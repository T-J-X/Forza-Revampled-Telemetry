//! The queue's files.
//!
//! The lock files (the lane's *slots*) are where they have always been, in `Dash_Tools\`:
//! `gpu.lock` for the PC's card, `gpu-cloud.lock` and `gpu-cloud-2.lock` for the cloud pod. Their
//! first three lines (`thread`, `started`, `job`) are the ones asset-forge reads; the queue adds
//! `ticket`, `took_unix`, `until`, `until_unix`, `kind` and `pid`, and the holder touches the file
//! every 10 s as its heartbeat.
//!
//! Each lane has a folder, `Dash_Tools\gpu-queue\<lane>\`, with one *place* for every run that
//! waits for a slot or holds one:
//!
//! - `<id>.ticket`: who waits, for what, since when (text, read by everyone);
//! - `<id>.live`: locked by the run's process for as long as it lives, so a run whose process
//!   dies, whatever killed it, leaves the queue and frees its slot at once;
//! - `<id>.release`: asks the run to let go (`release`, `cancel`, or the next in line taking
//!   over the slot of a holder that is hung past its time limit).
//!
//! An id is the time the run joined in nanoseconds, then its process id, both zero-padded, so
//! the names sort in the order the runs joined. Only the first waiting run in that order takes a
//! free slot, so the lock goes to whoever asked first, within one poll of its release.
//!
//! The process that waits is the process that holds: `run` holds while its command runs, and
//! `take` holds until `release`. Nothing is left running in the background, which matters on
//! Windows, where a process started from another inherits its open handles, and a leftover
//! holder would keep the caller's output pipe open.

use std::cell::Cell;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::clock;

/// How often a waiting or holding run looks at the queue.
pub const POLL: Duration = Duration::from_millis(200);

/// A lock file made by hand (no ticket) counts as held until it is this old.
pub const HAND_MADE_STALE_SECS: u64 = 15 * 60;

/// A holder touches its lock file this often, as its heartbeat.
pub const BEAT_SECS: u64 = 10;

/// A lock file whose heartbeat is this old belongs to a holder that hung or froze.
pub const NO_BEAT_STALE_SECS: u64 = 120;

/// A holder this far past its time limit is taken to be hung, and the next in line takes over.
pub const PAST_LIMIT_GRACE_SECS: u64 = 120;

/// The two GPUs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lane {
    /// The PC's card: one job at a time.
    Local,
    /// The rented cloud pod: two jobs at once.
    Cloud,
}

impl Lane {
    /// Both lanes, for `status` and `release`.
    pub const ALL: [Self; 2] = [Self::Local, Self::Cloud];

    /// `local` or `cloud`, in any case.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "local" => Some(Self::Local),
            "cloud" => Some(Self::Cloud),
            _ => None,
        }
    }

    /// The name used in commands, folders and the log.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Cloud => "cloud",
        }
    }

    /// The lane's lock files, in the order they are handed out (asset-forge reads the same names).
    #[must_use]
    pub fn slots(self) -> &'static [&'static str] {
        match self {
            Self::Local => &["gpu.lock"],
            Self::Cloud => &["gpu-cloud.lock", "gpu-cloud-2.lock"],
        }
    }
}

/// How a run holds its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// For exactly as long as one command runs.
    Run,
    /// From `take` until `release` or the time limit.
    Take,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::Take => "take",
        }
    }

    fn parse(text: &str) -> Self {
        if text.trim() == "take" {
            Self::Take
        } else {
            Self::Run
        }
    }
}

/// Who asks for the GPU, and for what. Both are kept to one line of at most 120 characters,
/// with no double quotes, so they cannot break the files or the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Who {
    /// The thread's name, as the thread gives it.
    pub thread: String,
    /// What the job is.
    pub job: String,
}

impl Who {
    /// Cleans both texts.
    #[must_use]
    pub fn new(thread: &str, job: &str) -> Self {
        Self {
            thread: one_line(thread),
            job: one_line(job),
        }
    }
}

fn one_line(text: &str) -> String {
    let cleaned: String = text
        .chars()
        .map(|c| match c {
            '"' => '\'',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    cleaned.trim().chars().take(120).collect()
}

/// Where the lock files and the queue live: `GPU_LOCK_DIR` when it is set (the tests use it),
/// else `%USERPROFILE%\Documents\Dash_Tools` (or `$HOME/Documents/Dash_Tools`).
#[derive(Clone, Debug)]
pub struct Home {
    tools: PathBuf,
}

impl Home {
    /// The folder from the environment, or `None` when no home folder is set.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        if let Some(dir) = std::env::var_os("GPU_LOCK_DIR").filter(|dir| !dir.is_empty()) {
            return Some(Self::new(PathBuf::from(dir)));
        }
        let profile = std::env::var_os("USERPROFILE")
            .filter(|dir| !dir.is_empty())
            .or_else(|| std::env::var_os("HOME").filter(|dir| !dir.is_empty()))?;
        Some(Self::new(PathBuf::from(profile).join("Documents").join("Dash_Tools")))
    }

    /// A queue in `tools`.
    #[must_use]
    pub fn new(tools: PathBuf) -> Self {
        Self { tools }
    }

    /// A lock file.
    #[must_use]
    pub fn slot(&self, name: &str) -> PathBuf {
        self.tools.join(name)
    }

    /// A lane's queue folder.
    #[must_use]
    pub fn queue(&self, lane: Lane) -> PathBuf {
        self.tools.join("gpu-queue").join(lane.name())
    }

    /// The log of every take, release, time limit and stale lock.
    #[must_use]
    pub fn log_file(&self) -> PathBuf {
        self.tools.join("gpu-queue").join("log.txt")
    }

    /// Appends one line to the log. The log is for people, so a failed write is ignored.
    pub fn log(&self, lane: Lane, slot: &str, event: &str, who: &Who, detail: &str) {
        let separator = if detail.is_empty() { "" } else { ": " };
        let line = format!(
            "{} {} {slot} {event} \"{}\" ({}){separator}{detail}\n",
            clock::utc_ms(clock::now_ms()),
            lane.name(),
            who.thread,
            who.job
        );
        let path = self.log_file();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

/// A new place's id: the time in nanoseconds and the process id, so ids sort in joining order.
#[must_use]
pub fn new_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    format!("{nanos:024}-{:010}", std::process::id())
}

/// Ids are digits and dashes only, so a hand-edited file cannot point outside the queue.
#[must_use]
pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 40 && id.bytes().all(|b| b.is_ascii_digit() || b == b'-')
}

fn place_file(dir: &Path, id: &str, extension: &str) -> PathBuf {
    dir.join(format!("{id}.{extension}"))
}

/// The value of a `key: value` line.
fn field<'t>(text: &'t str, key: &str) -> Option<&'t str> {
    text.lines()
        .find_map(|line| line.strip_prefix(key)?.strip_prefix(':').map(str::trim))
}

/// Whether the process that locked a file still lives.
enum Life {
    Alive,
    /// Nobody holds it; when the file exists, the handle that now holds its lock comes along.
    Dead(Option<File>),
}

fn life(path: &Path) -> Life {
    match File::open(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Life::Dead(None),
        // A file that cannot be opened counts as held: the queue never frees a slot on a guess.
        Err(_) => Life::Alive,
        Ok(file) => match file.try_lock() {
            Ok(()) => Life::Dead(Some(file)),
            Err(TryLockError::WouldBlock | TryLockError::Error(_)) => Life::Alive,
        },
    }
}

/// Removes a dead place. `held` is the lock of its `.live` file, which this process now holds,
/// so nobody else removes it at the same time.
fn clean_up(dir: &Path, id: &str, held: Option<File>) {
    for extension in ["ticket", "release"] {
        let _ = fs::remove_file(place_file(dir, id, extension));
    }
    let _ = fs::remove_file(place_file(dir, id, "live"));
    drop(held);
}

/// Whether a process with this id runs (`tasklist` on Windows, `/proc` elsewhere). A check that
/// cannot run counts the process as running: the queue never frees a slot on a guess.
#[must_use]
pub fn process_alive(pid: u32) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let tasklist = std::env::var_os("SystemRoot").map_or_else(
            || PathBuf::from("tasklist.exe"),
            |root| PathBuf::from(root).join("System32").join("tasklist.exe"),
        );
        match std::process::Command::new(tasklist)
            .args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
        {
            Ok(out) => String::from_utf8_lossy(&out.stdout).contains(&format!("\"{pid}\"")),
            Err(_) => true,
        }
    }
    #[cfg(not(windows))]
    {
        Path::new(&format!("/proc/{pid}")).exists()
    }
}

/// Asks a place to let go.
pub fn ask_release(home: &Home, lane: Lane, id: &str) {
    if valid_id(id) {
        let _ = File::create(place_file(&home.queue(lane), id, "release"));
    }
}

/// A place in the queue, as others see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ticket {
    /// The place's id.
    pub id: String,
    /// Who waits or holds.
    pub who: Who,
    /// `run` or `take`.
    pub kind: Kind,
    /// When it joined, in seconds since the epoch (0 if its ticket could not be read).
    pub joined: u64,
    /// It was asked to let go and is leaving.
    pub leaving: bool,
}

fn read_ticket(dir: &Path, id: &str) -> Ticket {
    let text = fs::read_to_string(place_file(dir, id, "ticket")).unwrap_or_default();
    Ticket {
        id: id.to_owned(),
        who: Who::new(
            field(&text, "thread").unwrap_or("(unreadable)"),
            field(&text, "job").unwrap_or("(unreadable)"),
        ),
        kind: field(&text, "kind").map_or(Kind::Run, Kind::parse),
        joined: field(&text, "joined").and_then(|v| v.parse().ok()).unwrap_or(0),
        leaving: place_file(dir, id, "release").exists(),
    }
}

/// A lock file, read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotLock {
    /// Who holds it.
    pub who: Who,
    /// When, as written.
    pub started: String,
    /// The holder's place; `None` for a lock file made by hand.
    pub ticket: Option<String>,
    /// The holder's time limit, in seconds since the epoch.
    pub until: Option<u64>,
    /// How the holder holds it; `None` for a lock file made by hand.
    pub kind: Option<Kind>,
    /// The process id of the job a `run` started.
    pub job_pid: Option<u32>,
    /// Seconds since the file was written.
    pub age: u64,
    /// How long it has been held: since `took_unix`, or the file's age for a hand-made file.
    pub held: u64,
    /// The whole text, to check that the file is unchanged before it is taken over.
    pub text: String,
}

impl SlotLock {
    /// Reads the fields; missing ones stay empty.
    #[must_use]
    pub fn parse(text: &str, age: u64) -> Self {
        Self {
            who: Who::new(field(text, "thread").unwrap_or(""), field(text, "job").unwrap_or("")),
            started: field(text, "started").unwrap_or("").to_owned(),
            ticket: field(text, "ticket").filter(|id| valid_id(id)).map(str::to_owned),
            until: field(text, "until_unix").and_then(|v| v.parse().ok()),
            kind: field(text, "kind").map(Kind::parse),
            job_pid: field(text, "job_pid").and_then(|v| v.parse().ok()),
            age,
            held: field(text, "took_unix")
                .and_then(|v| v.parse::<u64>().ok())
                .map_or(age, |took| clock::now_secs().saturating_sub(took)),
            text: text.to_owned(),
        }
    }
}

/// The text of a lock file the queue writes.
#[must_use]
pub fn lock_text(who: &Who, took: u64, id: &str, until: u64, kind: Kind) -> String {
    format!(
        "thread: {}\nstarted: {}\njob: {}\nticket: {id}\ntook_unix: {took}\nuntil: {}\nuntil_unix: {until}\nkind: {}\npid: {}\n",
        who.thread,
        clock::utc(took),
        who.job,
        clock::utc(until),
        kind.name(),
        std::process::id()
    )
}

/// Why a lock file no longer counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stale {
    /// The process that held it is gone.
    HolderGone,
    /// Its holder is hung: past its time limit by more than [`PAST_LIMIT_GRACE_SECS`].
    PastLimit,
    /// Its holder has not touched it for [`NO_BEAT_STALE_SECS`]: hung, or its process froze.
    NoHeartbeat,
    /// A lock file made by hand that is older than [`HAND_MADE_STALE_SECS`].
    HandMadeOld,
}

impl Stale {
    /// Words for the log and `status`.
    #[must_use]
    pub fn reason(self) -> &'static str {
        match self {
            Self::HolderGone => "its holder's process is gone",
            Self::PastLimit => "its holder is hung past its time limit",
            Self::NoHeartbeat => "its holder has had no heartbeat for 2 minutes",
            Self::HandMadeOld => "it was made by hand and is over 15 minutes old",
        }
    }
}

/// A slot's state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Slot {
    /// No lock file.
    Free,
    /// A lock file that counts.
    Held(SlotLock),
    /// A lock file that no longer counts: the first in line takes it over.
    Stale(SlotLock, Stale),
}

/// A lane's queue and slots at one moment.
#[derive(Clone, Debug)]
pub struct View {
    /// The lane.
    pub lane: Lane,
    /// The live places, in the order they joined.
    pub tickets: Vec<Ticket>,
    /// The lane's slots, in the order they are handed out.
    pub slots: Vec<(&'static str, Slot)>,
}

impl View {
    fn lock_of(slot: &Slot) -> Option<&SlotLock> {
        match slot {
            Slot::Held(lock) | Slot::Stale(lock, _) => Some(lock),
            Slot::Free => None,
        }
    }

    fn named(&self, id: &str) -> bool {
        self.slots
            .iter()
            .filter_map(|(_, slot)| Self::lock_of(slot))
            .any(|lock| lock.ticket.as_deref() == Some(id))
    }

    /// The places that wait for a slot, in the order they joined.
    pub fn waiting(&self) -> impl Iterator<Item = &Ticket> {
        self.tickets
            .iter()
            .filter(|ticket| !ticket.leaving && !self.named(&ticket.id))
    }

    /// A waiting place's position in line, from 1.
    #[must_use]
    pub fn place_of(&self, id: &str) -> Option<usize> {
        self.waiting().position(|ticket| ticket.id == id).map(|at| at + 1)
    }

    /// The slot a place holds, and the lock file it wrote.
    #[must_use]
    pub fn slot_of(&self, id: &str) -> Option<(&'static str, &SlotLock)> {
        self.slots.iter().find_map(|(name, slot)| match slot {
            Slot::Held(lock) if lock.ticket.as_deref() == Some(id) => Some((*name, lock)),
            Slot::Held(_) | Slot::Stale(..) | Slot::Free => None,
        })
    }

    /// A slot `thread` holds with `take` or by hand; a `run` of the same thread uses that hold.
    #[must_use]
    pub fn held_by_hand_or_take(&self, thread: &str) -> Option<&'static str> {
        self.slots.iter().find_map(|(name, slot)| match slot {
            Slot::Held(lock) if lock.who.thread == thread && lock.kind != Some(Kind::Run) => Some(*name),
            Slot::Held(_) | Slot::Stale(..) | Slot::Free => None,
        })
    }
}

/// Looks at a lane: removes the places whose process is gone, and judges every slot. `me` is the
/// caller's own place, which is alive by definition.
#[must_use]
pub fn scan(home: &Home, lane: Lane, me: Option<&str>) -> View {
    let dir = home.queue(lane);
    let mut ids: Vec<String> = fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| {
                    let name = entry.file_name();
                    name.to_str()
                        .and_then(|name| name.strip_suffix(".ticket"))
                        .filter(|id| valid_id(id))
                        .map(str::to_owned)
                })
                .collect()
        })
        .unwrap_or_default();
    ids.sort();
    let mut tickets = Vec::with_capacity(ids.len());
    for id in ids {
        if me != Some(id.as_str()) {
            if let Life::Dead(held) = life(&place_file(&dir, &id, "live")) {
                clean_up(&dir, &id, held);
                continue;
            }
        }
        tickets.push(read_ticket(&dir, &id));
    }
    let now = clock::now_secs();
    let slots = lane
        .slots()
        .iter()
        .map(|&name| (name, judge(home, name, now, &tickets, me)))
        .collect();
    View { lane, tickets, slots }
}

fn judge(home: &Home, name: &str, now: u64, tickets: &[Ticket], me: Option<&str>) -> Slot {
    let path = home.slot(name);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Slot::Free,
        // Unreadable for a moment (being written): judged below as a young hand-made file.
        Err(_) => String::new(),
    };
    let age = fs::metadata(&path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|written| SystemTime::now().duration_since(written).ok())
        .map_or(0, |since| since.as_secs());
    let lock = SlotLock::parse(&text, age);
    match lock.ticket.as_deref() {
        Some(id) => {
            let alive = me == Some(id) || tickets.iter().any(|ticket| ticket.id == id);
            // A job whose supervisor was killed still uses the GPU: it keeps the slot.
            let orphan = !alive && lock.job_pid.is_some_and(process_alive);
            if orphan {
                Slot::Held(lock)
            } else if !alive {
                Slot::Stale(lock, Stale::HolderGone)
            } else if lock
                .until
                .is_some_and(|until| now > until.saturating_add(PAST_LIMIT_GRACE_SECS))
            {
                Slot::Stale(lock, Stale::PastLimit)
            } else if me != Some(id) && age > NO_BEAT_STALE_SECS {
                Slot::Stale(lock, Stale::NoHeartbeat)
            } else {
                Slot::Held(lock)
            }
        }
        None if age > HAND_MADE_STALE_SECS => Slot::Stale(lock, Stale::HandMadeOld),
        None => Slot::Held(lock),
    }
}

/// This run's place in a lane's queue. Dropping it leaves the queue.
#[derive(Debug)]
pub struct Place {
    dir: PathBuf,
    id: String,
    /// Who waits.
    pub who: Who,
    /// How it will hold its slot.
    pub kind: Kind,
    /// The lane it waits for.
    pub lane: Lane,
    joined: u64,
    live: Option<File>,
}

impl Place {
    /// Joins the end of `lane`'s queue. `id` is given by `take`, which made the id for its holder.
    ///
    /// # Errors
    ///
    /// When the queue folder or the place's files cannot be written.
    pub fn join(home: &Home, lane: Lane, who: Who, kind: Kind, id: Option<String>) -> io::Result<Self> {
        let dir = home.queue(lane);
        fs::create_dir_all(&dir)?;
        let id = id.filter(|id| valid_id(id)).unwrap_or_else(new_id);
        // `.live` first, locked, so the place is alive before anyone can see its ticket.
        let live = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(place_file(&dir, &id, "live"))?;
        live.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => io::Error::other("another process holds this place"),
            TryLockError::Error(error) => error,
        })?;
        let place = Self {
            dir,
            id,
            who,
            kind,
            lane,
            joined: clock::now_secs(),
            live: Some(live),
        };
        // On an error here, dropping `place` removes the `.live` file again.
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(place_file(&place.dir, &place.id, "ticket"))?
            .write_all(place.ticket_text().as_bytes())?;
        Ok(place)
    }

    /// The place's id.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// When it joined, in seconds since the epoch.
    #[must_use]
    pub fn joined(&self) -> u64 {
        self.joined
    }

    /// Whether `release`, `cancel` or a takeover asked it to let go.
    #[must_use]
    pub fn asked_to_let_go(&self) -> bool {
        place_file(&self.dir, &self.id, "release").exists()
    }

    /// Writes the place's files again if someone deleted them (or the whole queue folder), so a
    /// waiting run keeps its place and a holder stays visible to the others.
    ///
    /// # Errors
    ///
    /// When the files cannot be written again.
    pub fn repair(&mut self) -> io::Result<()> {
        let live = place_file(&self.dir, &self.id, "live");
        if !live.exists() {
            fs::create_dir_all(&self.dir)?;
            let file = OpenOptions::new().write(true).create_new(true).open(&live)?;
            file.try_lock().map_err(|error| match error {
                TryLockError::WouldBlock => io::Error::other("another process holds this place"),
                TryLockError::Error(error) => error,
            })?;
            self.live = Some(file);
        }
        let ticket = place_file(&self.dir, &self.id, "ticket");
        if !ticket.exists() {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&ticket)?
                .write_all(self.ticket_text().as_bytes())?;
        }
        Ok(())
    }

    fn ticket_text(&self) -> String {
        format!(
            "thread: {}\njob: {}\nkind: {}\njoined: {}\npid: {}\n",
            self.who.thread,
            self.who.job,
            self.kind.name(),
            self.joined,
            std::process::id()
        )
    }
}

impl Drop for Place {
    fn drop(&mut self) {
        for extension in ["ticket", "release"] {
            let _ = fs::remove_file(place_file(&self.dir, &self.id, extension));
        }
        if let Some(live) = self.live.take() {
            let _ = live.unlock();
            drop(live);
        }
        let _ = fs::remove_file(place_file(&self.dir, &self.id, "live"));
    }
}

/// A slot that a run holds.
#[derive(Debug)]
pub struct Hold {
    /// The lock file's name.
    pub slot: &'static str,
    /// When it was taken, in seconds since the epoch.
    pub took: u64,
    until: Cell<u64>,
    job_pid: Cell<Option<u32>>,
    last_tick: Cell<u64>,
    warned: Cell<bool>,
    beat: Cell<u64>,
}

/// What a holder does next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tick {
    /// Keep holding.
    Keep,
    /// `release`, `cancel` or a takeover asked it to let go.
    Asked,
    /// Its time limit is reached.
    Limit,
    /// Another run's lock file is in its slot: it must stop, so that one job uses the slot.
    Lost,
}

impl Hold {
    fn new(slot: &'static str, took: u64, until: u64) -> Self {
        Self {
            slot,
            took,
            until: Cell::new(until),
            job_pid: Cell::new(None),
            last_tick: Cell::new(took),
            warned: Cell::new(false),
            beat: Cell::new(took),
        }
    }

    /// The time limit, in seconds since the epoch.
    #[must_use]
    pub fn until(&self) -> u64 {
        self.until.get()
    }

    fn text(&self, place: &Place) -> String {
        let mut text = lock_text(&place.who, self.took, place.id(), self.until.get(), place.kind);
        if let Some(pid) = self.job_pid.get() {
            text.push_str(&format!("job_pid: {pid}\n"));
        }
        text
    }

    fn write(&self, home: &Home, place: &Place) {
        let _ = fs::write(home.slot(self.slot), self.text(place));
    }

    /// Records the process id of the job `run` started, so that a job whose supervisor was
    /// killed keeps the slot until the job itself has ended.
    pub fn set_job(&self, home: &Home, place: &Place, pid: u32) {
        self.job_pid.set(Some(pid));
        self.write(home, place);
    }

    /// Checks the hold once, and beats. A lock file deleted by hand (the old `rm -f`) is written
    /// again, and so are the place's own files. After the PC slept, the time limit moves on by
    /// the time asleep, so a job is never stopped for time it did not run.
    pub fn tick(&self, home: &Home, place: &mut Place) -> Tick {
        if place.asked_to_let_go() {
            return Tick::Asked;
        }
        let now = clock::now_secs();
        let gap = now.saturating_sub(self.last_tick.get());
        self.last_tick.set(now);
        if gap > 10 {
            self.until.set(self.until.get().saturating_add(gap));
            self.write(home, place);
            home.log(
                place.lane,
                self.slot,
                "woke",
                &place.who,
                &format!("the limit moves on by {}", clock::span(gap)),
            );
        }
        if now >= self.until.get() {
            return Tick::Limit;
        }
        if place.repair().is_err() && !self.warned.replace(true) {
            home.log(
                place.lane,
                self.slot,
                "warning",
                &place.who,
                "its place in the queue was deleted and could not be written again",
            );
        }
        let path = home.slot(self.slot);
        match fs::read_to_string(&path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                if let Ok(mut file) = OpenOptions::new().write(true).create_new(true).open(&path) {
                    let _ = file.write_all(self.text(place).as_bytes());
                    home.log(
                        place.lane,
                        self.slot,
                        "rewrote",
                        &place.who,
                        "the lock file was deleted while it was held",
                    );
                }
            }
            Ok(text) if !text.is_empty() && SlotLock::parse(&text, 0).ticket.as_deref() != Some(place.id()) => {
                let other = SlotLock::parse(&text, 0);
                home.log(
                    place.lane,
                    self.slot,
                    "lost",
                    &place.who,
                    &format!(
                        "\"{}\" ({}) holds the slot now; this run stops",
                        other.who.thread, other.who.job
                    ),
                );
                return Tick::Lost;
            }
            Ok(_) => {
                if now >= self.beat.get().saturating_add(BEAT_SECS) {
                    self.beat.set(now);
                    if let Ok(file) = OpenOptions::new().write(true).open(&path) {
                        let _ = file.set_modified(SystemTime::now());
                    }
                }
            }
            Err(_) => {}
        }
        Tick::Keep
    }

    /// Lets go: removes the lock file if it is still this run's, and logs why.
    pub fn release(&self, home: &Home, place: &Place, why: &str) {
        let path = home.slot(self.slot);
        let ours =
            fs::read_to_string(&path).is_ok_and(|text| SlotLock::parse(&text, 0).ticket.as_deref() == Some(place.id()));
        if ours {
            let _ = fs::remove_file(&path);
        }
        let held = clock::now_secs().saturating_sub(self.took);
        home.log(
            place.lane,
            self.slot,
            "released",
            &place.who,
            &format!("{why}, after {}", clock::span(held)),
        );
    }
}

/// Takes the first free or stale slot for `place`, if `place` is first in line.
///
/// # Errors
///
/// When a free slot's lock file cannot be written.
pub fn try_take(
    home: &Home,
    view: &View,
    place: &Place,
    limit_secs: u64,
    trust_beats: bool,
) -> io::Result<Option<Hold>> {
    if view.waiting().next().map(|ticket| ticket.id.as_str()) != Some(place.id()) {
        return Ok(None);
    }
    for (name, slot) in &view.slots {
        let path = home.slot(name);
        match slot {
            Slot::Held(_) => continue,
            Slot::Free => {}
            // Just after this PC wakes from sleep every heartbeat looks old: give the holders time to beat.
            Slot::Stale(_, Stale::NoHeartbeat) if !trust_beats => continue,
            Slot::Stale(lock, why) => {
                // Take it over only if it is still the file that was judged stale.
                if fs::read_to_string(&path).ok().as_deref() != Some(lock.text.as_str()) {
                    continue;
                }
                if matches!(why, Stale::PastLimit | Stale::NoHeartbeat) {
                    if let Some(id) = lock.ticket.as_deref() {
                        ask_release(home, view.lane, id);
                    }
                }
                match fs::remove_file(&path) {
                    Ok(()) => {}
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(_) => continue,
                }
                home.log(
                    view.lane,
                    name,
                    "stale",
                    &lock.who,
                    &format!(
                        "{} ({} old); taken over by \"{}\"",
                        why.reason(),
                        clock::span(lock.held),
                        place.who.thread
                    ),
                );
            }
        }
        let took = clock::now_secs();
        let until = took.saturating_add(limit_secs);
        let text = lock_text(&place.who, took, place.id(), until, place.kind);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                if let Err(error) = file.write_all(text.as_bytes()) {
                    drop(file);
                    let _ = fs::remove_file(&path);
                    return Err(error);
                }
                let waited = took.saturating_sub(place.joined());
                home.log(
                    view.lane,
                    name,
                    "took",
                    &place.who,
                    &format!(
                        "after {} in line; limit {}",
                        clock::span(waited),
                        clock::span(limit_secs)
                    ),
                );
                return Ok(Some(Hold::new(name, took, until)));
            }
            // A lock file made by hand got there first: wait for it like any other.
            // A lock file made by hand got there first, or Windows still has the old file
            // "delete pending" (access denied): try again on the next look.
            Err(_) => continue,
        }
    }
    Ok(None)
}

/// How a wait in line ended.
#[derive(Debug)]
pub enum Wait {
    /// The run's turn came, and it holds this slot.
    Took(Hold),
    /// `release` or `cancel` asked it to leave the queue.
    Asked,
    /// It waited as long as it was allowed to.
    GaveUp,
}

/// Waits in line until `place` holds a slot. `give_up_at` is a time in seconds since the epoch;
/// `report` sees every look at the queue, for progress messages.
///
/// # Errors
///
/// When the place's files or a free slot's lock file cannot be written.
pub fn wait_turn(
    home: &Home,
    place: &mut Place,
    limit_secs: u64,
    give_up_at: Option<u64>,
    report: &mut dyn FnMut(&View),
) -> io::Result<Wait> {
    let mut last_look = clock::now_secs();
    let mut woke_at = 0;
    loop {
        if place.asked_to_let_go() {
            return Ok(Wait::Asked);
        }
        let now = clock::now_secs();
        if now > last_look.saturating_add(10) {
            woke_at = now;
        }
        last_look = now;
        let trust_beats = now >= woke_at.saturating_add(BEAT_SECS * 2);
        place.repair()?;
        let view = scan(home, place.lane, Some(place.id()));
        if let Some(hold) = try_take(home, &view, place, limit_secs, trust_beats)? {
            return Ok(Wait::Took(hold));
        }
        if give_up_at.is_some_and(|at| clock::now_secs() >= at) {
            return Ok(Wait::GaveUp);
        }
        report(&view);
        std::thread::sleep(POLL);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn home(name: &str) -> Home {
        let dir = std::env::temp_dir().join(format!("gpu-lock-unit-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Home::new(dir)
    }

    fn done(home: &Home) {
        let _ = fs::remove_dir_all(&home.tools);
    }

    fn who(thread: &str) -> Who {
        Who::new(thread, "a job")
    }

    #[test]
    fn the_lanes_have_the_lock_files_asset_forge_reads() {
        assert_eq!(Lane::Local.slots(), ["gpu.lock"]);
        assert_eq!(Lane::Cloud.slots(), ["gpu-cloud.lock", "gpu-cloud-2.lock"]);
        assert_eq!(Lane::parse(" Cloud "), Some(Lane::Cloud));
        assert_eq!(Lane::parse("moon"), None);
    }

    #[test]
    fn names_and_jobs_stay_on_one_line_without_quotes() {
        let who = Who::new("  a \"thread\"\nname ", &"x".repeat(300));
        assert_eq!(who.thread, "a 'thread' name");
        assert_eq!(who.job.len(), 120);
    }

    #[test]
    fn ids_sort_in_the_order_places_joined() {
        let first = new_id();
        std::thread::sleep(Duration::from_millis(2));
        let second = new_id();
        assert!(first < second, "{first} {second}");
        assert!(valid_id(&first));
        assert!(!valid_id("../gpu.lock"));
        assert!(!valid_id(""));
    }

    #[test]
    fn a_lock_file_keeps_the_three_lines_asset_forge_reads_first() {
        let text = lock_text(&who("T"), 1_795_846_800, "1-2", 1_795_850_400, Kind::Take);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "thread: T");
        assert_eq!(lines[1], "started: 2026-11-28T06:20:00Z");
        assert_eq!(lines[2], "job: a job");
        let lock = SlotLock::parse(&text, 5);
        assert_eq!(lock.ticket.as_deref(), Some("1-2"));
        assert_eq!(lock.until, Some(1_795_850_400));
        assert_eq!(lock.kind, Some(Kind::Take));
        assert_eq!(lock.who, who("T"));
    }

    #[test]
    fn a_hand_made_lock_has_no_ticket_and_counts_until_it_is_old() {
        let home = home("hand");
        fs::write(home.slot("gpu.lock"), "thread: old\nstarted: x\njob: y\n").unwrap();
        let view = scan(&home, Lane::Local, None);
        assert!(matches!(&view.slots[0].1, Slot::Held(lock) if lock.ticket.is_none()));
        let file = File::options().write(true).open(home.slot("gpu.lock")).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(HAND_MADE_STALE_SECS + 5))
            .unwrap();
        drop(file);
        let view = scan(&home, Lane::Local, None);
        assert!(matches!(view.slots[0].1, Slot::Stale(_, Stale::HandMadeOld)));
        done(&home);
    }

    #[test]
    fn a_lock_whose_place_is_gone_is_stale_at_once() {
        let home = home("gone");
        let text = lock_text(
            &who("T"),
            clock::now_secs(),
            "123-456",
            clock::now_secs() + 600,
            Kind::Run,
        );
        fs::write(home.slot("gpu.lock"), text).unwrap();
        let view = scan(&home, Lane::Local, None);
        assert!(matches!(view.slots[0].1, Slot::Stale(_, Stale::HolderGone)));
        done(&home);
    }

    #[test]
    fn a_lock_with_no_heartbeat_for_two_minutes_is_stale_but_not_just_after_a_wake() {
        let home = home("beat");
        let holder = Place::join(&home, Lane::Local, who("H"), Kind::Run, None).unwrap();
        let view = scan(&home, Lane::Local, Some(holder.id()));
        let _hold = try_take(&home, &view, &holder, 600, true).unwrap().unwrap();
        let waiter = Place::join(&home, Lane::Local, who("W"), Kind::Run, None).unwrap();
        assert!(matches!(scan(&home, Lane::Local, None).slots[0].1, Slot::Held(_)));
        let file = File::options().write(true).open(home.slot("gpu.lock")).unwrap();
        file.set_modified(SystemTime::now() - Duration::from_secs(NO_BEAT_STALE_SECS + 5))
            .unwrap();
        drop(file);
        let view = scan(&home, Lane::Local, Some(waiter.id()));
        assert!(matches!(view.slots[0].1, Slot::Stale(_, Stale::NoHeartbeat)));
        assert!(
            try_take(&home, &view, &waiter, 60, false).unwrap().is_none(),
            "just woken: wait"
        );
        let hold = try_take(&home, &view, &waiter, 60, true).unwrap().unwrap();
        assert_eq!(hold.slot, "gpu.lock");
        assert!(holder.asked_to_let_go(), "the hung holder is told to let go");
        drop(holder);
        drop(waiter);
        done(&home);
    }

    #[test]
    fn places_leave_when_dropped_and_the_first_in_line_takes_the_slot() {
        let home = home("order");
        let first = Place::join(&home, Lane::Local, who("A"), Kind::Run, None).unwrap();
        std::thread::sleep(Duration::from_millis(2));
        let second = Place::join(&home, Lane::Local, who("B"), Kind::Run, None).unwrap();
        let view = scan(&home, Lane::Local, Some(second.id()));
        assert_eq!(view.place_of(first.id()), Some(1));
        assert_eq!(view.place_of(second.id()), Some(2));
        // Only the first in line may take the free slot.
        assert!(try_take(&home, &view, &second, 60, true).unwrap().is_none());
        let hold = try_take(&home, &view, &first, 60, true).unwrap().unwrap();
        assert_eq!(hold.slot, "gpu.lock");
        let view = scan(&home, Lane::Local, None);
        assert_eq!(view.slot_of(first.id()).map(|(name, _)| name), Some("gpu.lock"));
        assert_eq!(view.place_of(second.id()), Some(1));
        hold.release(&home, &first, "done");
        drop(first);
        let view = scan(&home, Lane::Local, None);
        assert_eq!(view.slots[0].1, Slot::Free);
        assert_eq!(view.tickets.len(), 1);
        drop(second);
        assert!(scan(&home, Lane::Local, None).tickets.is_empty());
        done(&home);
    }

    #[test]
    fn a_place_whose_process_is_gone_is_cleaned_up() {
        let home = home("dead");
        let dir = home.queue(Lane::Local);
        fs::create_dir_all(&dir).unwrap();
        // A ticket and an unlocked `.live`: what a killed process leaves behind.
        fs::write(
            place_file(&dir, "1-1", "ticket"),
            "thread: X\njob: y\nkind: run\njoined: 1\n",
        )
        .unwrap();
        fs::write(place_file(&dir, "1-1", "live"), "").unwrap();
        let view = scan(&home, Lane::Local, None);
        assert!(view.tickets.is_empty());
        assert!(!place_file(&dir, "1-1", "ticket").exists());
        assert!(!place_file(&dir, "1-1", "live").exists());
        done(&home);
    }

    #[test]
    fn a_run_of_the_same_thread_uses_its_take_or_hand_made_lock() {
        let home = home("own");
        fs::write(home.slot("gpu.lock"), "thread: T\nstarted: x\njob: y\n").unwrap();
        let view = scan(&home, Lane::Local, None);
        assert_eq!(view.held_by_hand_or_take("T"), Some("gpu.lock"));
        assert_eq!(view.held_by_hand_or_take("U"), None);
        done(&home);
    }

    #[test]
    fn the_time_limit_and_a_request_end_a_hold() {
        let home = home("tick");
        let mut place = Place::join(&home, Lane::Local, who("A"), Kind::Take, None).unwrap();
        let view = scan(&home, Lane::Local, Some(place.id()));
        let hold = try_take(&home, &view, &place, 600, true).unwrap().unwrap();
        assert_eq!(hold.tick(&home, &mut place), Tick::Keep);
        // A lock file deleted by hand is written again.
        fs::remove_file(home.slot("gpu.lock")).unwrap();
        assert_eq!(hold.tick(&home, &mut place), Tick::Keep);
        assert!(home.slot("gpu.lock").exists());
        // So are the place's own files, if someone deletes the queue folder.
        fs::remove_dir_all(home.queue(Lane::Local)).unwrap();
        assert_eq!(hold.tick(&home, &mut place), Tick::Keep);
        assert_eq!(
            scan(&home, Lane::Local, None).slot_of(place.id()).map(|(name, _)| name),
            Some("gpu.lock")
        );
        ask_release(&home, Lane::Local, place.id());
        assert_eq!(hold.tick(&home, &mut place), Tick::Asked);
        let past = Hold::new("gpu.lock", clock::now_secs() - 5, clock::now_secs() - 1);
        fs::remove_file(place_file(&home.queue(Lane::Local), place.id(), "release")).unwrap();
        assert_eq!(past.tick(&home, &mut place), Tick::Limit);
        hold.release(&home, &place, "done");
        assert!(!home.slot("gpu.lock").exists());
        drop(place);
        done(&home);
    }
}
