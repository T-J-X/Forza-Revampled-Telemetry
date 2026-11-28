#![forbid(unsafe_code)]
//! `gpu-lock`: the first-come-first-served queue for the GPU lock files that the threads share
//! (`SKILLS.md` section 5; how it works: `tools/gpu-lock/README.md`).
//!
//! TJX, 2026-11-28: "as soon as gpu lock is off the next task that needs the gpu uses it". Every
//! GPU job waits in its lane's queue, and the lock goes to the first in line within one poll
//! (0.2 s) of its release. A holder whose process dies frees the lock at once, and every hold has
//! a time limit, so a crashed or forgotten holder cannot block the others.

mod clock;
mod queue;

use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

use queue::{Hold, Home, Kind, Lane, Place, Slot, Tick, View, Wait, Who};

const USAGE: &str = "\
gpu-lock: the first-come-first-served queue for the GPU lock files (SKILLS.md section 5)

  gpu-lock run --thread <name> --job <what> [--lane local|cloud] [--minutes N] [--wait-minutes N] -- <command> [args...]
      Waits its turn, holds the lane's lock file for exactly as long as the command runs, and
      exits with the command's exit code. Start it in the background: the wait can be long.
      --minutes is the time limit (default 15, at most 240): at the limit the command and
      everything it started are stopped, and the lock is released. A run of a thread that holds
      the lane with `take` runs at once.

  gpu-lock take --thread <name> --job <what> [--lane local|cloud] [--minutes N] [--wait-minutes N]
      Waits its turn, prints LOCKED, and holds the lock until `release` or the time limit
      (default 15 minutes, at most 240). For several commands, or the Blender MCP. It keeps
      running while it holds, so start it in the background, then use `wait`.

  gpu-lock wait --thread <name> [--lane local|cloud] [--wait-minutes N]
      Returns with LOCKED as soon as the thread's `take` holds the lock.

  gpu-lock release --thread <name> [--lane local|cloud]
      Lets go of the thread's `take` (also one still waiting) and of a lock file it made by hand.

  gpu-lock cancel --thread <name> [--lane local|cloud]
      Leaves the queue for every run and take of the thread, and stops its running jobs.

  gpu-lock status [--lane local|cloud]
      Who holds each lock file, and who waits, in order.

The local lane is the PC's card (gpu.lock); the cloud lane is the pod (gpu-cloud.lock and
gpu-cloud-2.lock). The files are in %USERPROFILE%\\Documents\\Dash_Tools\\, and the log of every take
and release is gpu-queue\\log.txt there.

Exit codes: the command's own for `run`; 2 for a wrong command line; 3 when --wait-minutes ran
out; 4 when cancelled or released before or during the job; 5 when the time limit stopped it;
6 when another run's lock file took its slot (it stopped so that one job uses the card).
";

const EXIT_FAILED: i32 = 1;
const EXIT_USAGE: i32 = 2;
const EXIT_GAVE_UP: i32 = 3;
const EXIT_CANCELLED: i32 = 4;
const EXIT_LIMIT: i32 = 5;
const EXIT_LOST: i32 = 6;

const RUN_MINUTES: f64 = 15.0;
const TAKE_MINUTES: f64 = 15.0;
const MAX_MINUTES: f64 = 240.0;

/// How long `wait` looks for a `take` that was started a moment before it.
const WAIT_FOR_TAKE_TO_JOIN: Duration = Duration::from_secs(5);

/// A failure, with the exit code it ends the program with.
#[derive(Debug)]
struct Fail {
    code: i32,
    message: String,
}

impl Fail {
    fn usage(message: impl Into<String>) -> Self {
        Self {
            code: EXIT_USAGE,
            message: message.into(),
        }
    }

    fn io(doing: &str, error: &std::io::Error) -> Self {
        Self {
            code: EXIT_FAILED,
            message: format!("{doing}: {error}"),
        }
    }
}

impl fmt::Display for Fail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// The command line after the command's name.
#[derive(Debug, Default)]
struct Opts {
    lane: Option<Lane>,
    thread: Option<String>,
    job: Option<String>,
    minutes: Option<f64>,
    wait_minutes: Option<f64>,
    command: Vec<String>,
}

impl Opts {
    fn lane(&self) -> Lane {
        self.lane.unwrap_or(Lane::Local)
    }

    fn lanes(&self) -> Vec<Lane> {
        self.lane.map_or_else(|| Lane::ALL.to_vec(), |lane| vec![lane])
    }

    fn thread(&self) -> Result<String, Fail> {
        self.thread
            .as_deref()
            .map(|thread| Who::new(thread, "").thread)
            .filter(|thread| !thread.is_empty())
            .ok_or_else(|| Fail::usage("--thread <name> is required: the name of your thread"))
    }

    fn who(&self) -> Result<Who, Fail> {
        let thread = self.thread()?;
        let job = self
            .job
            .as_deref()
            .map(str::trim)
            .filter(|job| !job.is_empty())
            .ok_or_else(|| Fail::usage("--job <what> is required: what the job is"))?;
        Ok(Who::new(&thread, job))
    }

    /// The time `--wait-minutes` runs out, in seconds since the epoch.
    fn give_up_at(&self) -> Option<u64> {
        self.wait_minutes
            .map(|minutes| clock::now_secs().saturating_add(secs(minutes)))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match dispatch(&args) {
        Ok(code) => code,
        Err(fail) => {
            eprintln!("gpu-lock: {fail}");
            fail.code
        }
    };
    std::process::exit(code);
}

fn dispatch(args: &[String]) -> Result<i32, Fail> {
    let Some((command, rest)) = args.split_first() else {
        print!("{USAGE}");
        return Ok(EXIT_USAGE);
    };
    if matches!(command.as_str(), "-h" | "--help" | "help") {
        print!("{USAGE}");
        return Ok(0);
    }
    let opts = parse(rest, command == "run")?;
    let home = Home::from_env()
        .ok_or_else(|| Fail::usage("neither USERPROFILE nor HOME is set, so the lock folder is unknown"))?;
    match command.as_str() {
        "run" => run(&home, &opts),
        "take" => take(&home, &opts),
        "wait" => wait_for_take(&home, &opts),
        "release" => let_go(&home, &opts, Scope::Takes),
        "cancel" => let_go(&home, &opts, Scope::Everything),
        "status" => status(&home, &opts),
        other => Err(Fail::usage(format!("unknown command `{other}`; see `gpu-lock --help`"))),
    }
}

fn parse(args: &[String], takes_command: bool) -> Result<Opts, Fail> {
    let mut opts = Opts::default();
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if arg == "--" {
            if !takes_command {
                return Err(Fail::usage("only `run` takes a command after `--`"));
            }
            opts.command = rest.cloned().collect();
            break;
        }
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| Fail::usage(format!("{arg} needs a value")))
        };
        match arg.as_str() {
            "--lane" => {
                let text = value()?;
                opts.lane = Some(
                    Lane::parse(&text).ok_or_else(|| Fail::usage(format!("--lane is local or cloud, not `{text}`")))?,
                );
            }
            "--thread" => opts.thread = Some(value()?),
            "--job" => opts.job = Some(value()?),
            "--minutes" => opts.minutes = Some(minutes(&value()?, "--minutes")?),
            "--wait-minutes" => opts.wait_minutes = Some(minutes(&value()?, "--wait-minutes")?),
            other => return Err(Fail::usage(format!("unknown option `{other}`; see `gpu-lock --help`"))),
        }
    }
    Ok(opts)
}

fn minutes(text: &str, flag: &str) -> Result<f64, Fail> {
    let minutes: f64 = text
        .trim()
        .parse()
        .map_err(|_| Fail::usage(format!("{flag} takes a number of minutes, not `{text}`")))?;
    if minutes.is_nan() || minutes <= 0.0 || minutes > MAX_MINUTES {
        return Err(Fail::usage(format!("{flag} is more than 0 and at most {MAX_MINUTES}")));
    }
    Ok(minutes)
}

/// Whole seconds, at least one.
fn secs(minutes: f64) -> u64 {
    Duration::try_from_secs_f64(minutes * 60.0).map_or(1, |span| span.as_secs().max(1))
}

/// Prints where a waiting run stands: when its place or the holders change, and once a minute.
#[derive(Default)]
struct Progress {
    last: String,
    at: u64,
}

impl Progress {
    fn show(&mut self, view: &View, id: &str) {
        let Some(place) = view.place_of(id) else {
            return;
        };
        let holders: Vec<(&str, &queue::SlotLock)> = view
            .slots
            .iter()
            .filter_map(|(name, slot)| match slot {
                Slot::Held(lock) => Some((*name, lock)),
                Slot::Stale(..) | Slot::Free => None,
            })
            .collect();
        let key = format!(
            "{place} {}",
            holders
                .iter()
                .map(|(name, lock)| format!("{name}={}", lock.who.thread))
                .collect::<Vec<_>>()
                .join(",")
        );
        let now = clock::now_secs();
        if key == self.last && now < self.at.saturating_add(60) {
            return;
        }
        let held = holders
            .iter()
            .map(|(name, lock)| {
                format!(
                    "{name} held by \"{}\" for {} ({})",
                    lock.who.thread,
                    clock::span(lock.held),
                    lock.who.job
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let line = if place == 1 {
            "next in line".to_owned()
        } else {
            format!("{} ahead in line", place - 1)
        };
        let separator = if held.is_empty() { "" } else { "; " };
        // Never block or fail on a full or closed output pipe: the queue behind this run waits on it.
        let _ = writeln!(
            std::io::stdout(),
            "gpu-lock: waiting for the {} lane, {line}{separator}{held}",
            view.lane.name()
        );
        self.last = key;
        self.at = now;
    }
}

/// Waits in line for `place`, showing progress. A wait that ends without the lock is an error
/// with its own exit code.
fn wait_in_line(home: &Home, place: &mut Place, limit: u64, give_up_at: Option<u64>) -> Result<Hold, Fail> {
    let id = place.id().to_owned();
    let lane = place.lane;
    let mut progress = Progress::default();
    let waited = queue::wait_turn(home, place, limit, give_up_at, &mut |view| progress.show(view, &id))
        .map_err(|error| Fail::io("taking the lock", &error))?;
    match waited {
        Wait::Took(hold) => Ok(hold),
        Wait::Asked => {
            home.log(lane, "-", "left", &place.who, "cancelled or released before its turn");
            Err(Fail {
                code: EXIT_CANCELLED,
                message: "cancelled before its turn; nothing ran".to_owned(),
            })
        }
        Wait::GaveUp => {
            home.log(lane, "-", "left", &place.who, "gave up waiting (--wait-minutes)");
            Err(Fail {
                code: EXIT_GAVE_UP,
                message: "gave up waiting (--wait-minutes); nothing ran".to_owned(),
            })
        }
    }
}

fn run(home: &Home, opts: &Opts) -> Result<i32, Fail> {
    let who = opts.who()?;
    let lane = opts.lane();
    let Some((program, args)) = opts.command.split_first() else {
        return Err(Fail::usage("run needs the command after `--`"));
    };
    let limit = secs(opts.minutes.unwrap_or(RUN_MINUTES));
    if let Some(slot) = queue::scan(home, lane, None).held_by_hand_or_take(&who.thread) {
        println!("gpu-lock: \"{}\" already holds {slot}; running at once", who.thread);
        let until = queue::scan(home, lane, None)
            .slots
            .iter()
            .find(|(name, _)| *name == slot)
            .and_then(|(_, state)| match state {
                Slot::Held(lock) => lock.until,
                Slot::Stale(..) | Slot::Free => None,
            });
        let mut child = spawn(program, args)?;
        return under_own_hold(&mut child, until);
    }
    let mut place =
        Place::join(home, lane, who, Kind::Run, None).map_err(|error| Fail::io("joining the queue", &error))?;
    home.log(
        lane,
        "-",
        "queued",
        &place.who,
        &format!("run, limit {}", clock::span(limit)),
    );
    let hold = wait_in_line(home, &mut place, limit, opts.give_up_at())?;
    println!(
        "LOCKED {} for \"{}\": {} (limit {}, until {}; released when the command ends)",
        hold.slot,
        place.who.thread,
        place.who.job,
        clock::span(limit),
        clock::hhmm(hold.until())
    );
    let mut child = match spawn(program, args) {
        Ok(child) => child,
        Err(fail) => {
            hold.release(home, &place, "the command did not start");
            return Err(fail);
        }
    };
    hold.set_job(home, &place, child.id());
    supervise(home, &mut place, &hold, &mut child, limit)
}

/// Runs the job to its end, its time limit, or a `cancel`, and releases the lock.
fn supervise(home: &Home, place: &mut Place, hold: &Hold, child: &mut Child, limit: u64) -> Result<i32, Fail> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                hold.release(home, place, &format!("the job ended ({})", describe(status)));
                return Ok(code_of(status));
            }
            Ok(None) => {}
            Err(error) => {
                stop_tree(child);
                hold.release(home, place, "lost track of the job");
                return Err(Fail::io("watching the job", &error));
            }
        }
        match hold.tick(home, place) {
            Tick::Keep => std::thread::sleep(queue::POLL),
            Tick::Asked => {
                stop_tree(child);
                hold.release(home, place, "cancelled; the job was stopped");
                eprintln!("gpu-lock: cancelled; the job was stopped and {} released", hold.slot);
                return Ok(EXIT_CANCELLED);
            }
            Tick::Limit => {
                stop_tree(child);
                hold.release(home, place, "time limit; the job was stopped");
                eprintln!(
                    "gpu-lock: the {} time limit was reached; the job was stopped and {} released (raise it with --minutes)",
                    clock::span(limit),
                    hold.slot
                );
                return Ok(EXIT_LIMIT);
            }
            Tick::Lost => {
                stop_tree(child);
                eprintln!(
                    "gpu-lock: another run's lock file is in {}; the job was stopped so only one job uses it",
                    hold.slot
                );
                return Ok(EXIT_LOST);
            }
        }
    }
}

/// Waits in line, then holds the lock until `release` or the time limit. This process is the
/// holder, so it is started in the background; `wait` says when it holds.
fn take(home: &Home, opts: &Opts) -> Result<i32, Fail> {
    let who = opts.who()?;
    let lane = opts.lane();
    let limit = secs(opts.minutes.unwrap_or(TAKE_MINUTES));
    if let Some(slot) = queue::scan(home, lane, None).held_by_hand_or_take(&who.thread) {
        println!("LOCKED {slot} (\"{}\" already holds it)", who.thread);
        return Ok(0);
    }
    let mut place =
        Place::join(home, lane, who, Kind::Take, None).map_err(|error| Fail::io("joining the queue", &error))?;
    home.log(
        lane,
        "-",
        "queued",
        &place.who,
        &format!("take, limit {}", clock::span(limit)),
    );
    let hold = wait_in_line(home, &mut place, limit, opts.give_up_at())?;
    println!(
        "LOCKED {} for \"{}\": {} (until {}); held until `gpu-lock release --thread \"{}\"`",
        hold.slot,
        place.who.thread,
        place.who.job,
        clock::hhmm(hold.until()),
        place.who.thread
    );
    loop {
        match hold.tick(home, &mut place) {
            Tick::Keep => std::thread::sleep(queue::POLL),
            Tick::Asked => {
                hold.release(home, &place, "released");
                println!("RELEASED {}", hold.slot);
                return Ok(0);
            }
            Tick::Limit => {
                hold.release(home, &place, "time limit");
                eprintln!(
                    "gpu-lock: the {} time limit was reached, so {} was released (raise it with --minutes)",
                    clock::span(limit),
                    hold.slot
                );
                return Ok(EXIT_LIMIT);
            }
            Tick::Lost => {
                eprintln!(
                    "gpu-lock: another run's lock file is in {}; this take no longer holds it",
                    hold.slot
                );
                return Ok(EXIT_LOST);
            }
        }
    }
}

/// A `run` under its own thread's `take` (or hand-made lock): it ends at that hold's time limit.
fn under_own_hold(child: &mut Child, until: Option<u64>) -> Result<i32, Fail> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(code_of(status)),
            Ok(None) => {}
            Err(error) => {
                stop_tree(child);
                return Err(Fail::io("watching the job", &error));
            }
        }
        if until.is_some_and(|until| clock::now_secs() >= until) {
            stop_tree(child);
            eprintln!("gpu-lock: the take this run went under reached its time limit; the job was stopped");
            return Ok(EXIT_LIMIT);
        }
        std::thread::sleep(queue::POLL);
    }
}

/// Returns as soon as the thread's `take` holds a lock file.
fn wait_for_take(home: &Home, opts: &Opts) -> Result<i32, Fail> {
    let thread = opts.thread()?;
    let lanes = opts.lanes();
    let give_up_at = opts.give_up_at();
    let started = std::time::Instant::now();
    let mut progress = Progress::default();
    loop {
        let mut in_line = false;
        for &lane in &lanes {
            let view = queue::scan(home, lane, None);
            if let Some(slot) = view.held_by_hand_or_take(&thread) {
                println!("LOCKED {slot} (held by \"{thread}\")");
                return Ok(0);
            }
            let waiting = view
                .waiting()
                .find(|ticket| ticket.who.thread == thread && ticket.kind == Kind::Take)
                .map(|ticket| ticket.id.clone());
            if let Some(id) = waiting {
                in_line = true;
                progress.show(&view, &id);
            }
        }
        if !in_line && started.elapsed() > WAIT_FOR_TAKE_TO_JOIN {
            return Err(Fail {
                code: EXIT_FAILED,
                message: format!(
                    "\"{thread}\" has no take in line or holding; start `gpu-lock take` first, in the background"
                ),
            });
        }
        if give_up_at.is_some_and(|at| clock::now_secs() >= at) {
            eprintln!("gpu-lock: gave up waiting (--wait-minutes); the take is still in line");
            return Ok(EXIT_GAVE_UP);
        }
        std::thread::sleep(queue::POLL);
    }
}

/// What `release` and `cancel` let go of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scope {
    /// The thread's `take`s and hand-made lock files.
    Takes,
    /// Every run and take of the thread, and its hand-made lock files.
    Everything,
}

fn let_go(home: &Home, opts: &Opts, scope: Scope) -> Result<i32, Fail> {
    let thread = opts.thread()?;
    let mut asked: Vec<(Lane, String, String)> = Vec::new();
    let mut said = 0;
    for lane in opts.lanes() {
        let view = queue::scan(home, lane, None);
        for ticket in &view.tickets {
            if ticket.who.thread != thread || (scope == Scope::Takes && ticket.kind != Kind::Take) {
                continue;
            }
            queue::ask_release(home, lane, &ticket.id);
            let what = view.slot_of(&ticket.id).map_or_else(
                || format!("its place in the {} queue", lane.name()),
                |(slot, _)| slot.to_owned(),
            );
            asked.push((lane, ticket.id.clone(), what));
        }
        for (name, slot) in &view.slots {
            let lock = match slot {
                Slot::Held(lock) | Slot::Stale(lock, _) => lock,
                Slot::Free => continue,
            };
            if lock.ticket.is_none() && lock.who.thread == thread && std::fs::remove_file(home.slot(name)).is_ok() {
                home.log(
                    lane,
                    name,
                    "released",
                    &lock.who,
                    "a lock file made by hand, removed by its thread",
                );
                println!("RELEASED {name} (a lock file made by hand)");
                said += 1;
            }
        }
    }
    // The holders and waiters look every 0.2 s; give them a few seconds to go.
    let mut left = asked.clone();
    for _ in 0..50 {
        left.retain(|(lane, id, _)| {
            queue::scan(home, *lane, None)
                .tickets
                .iter()
                .any(|ticket| &ticket.id == id)
        });
        if left.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    for (_, id, what) in &asked {
        if left.iter().all(|(_, still, _)| still != id) {
            println!("RELEASED {what}");
            said += 1;
        }
    }
    if !left.is_empty() {
        let names: Vec<&str> = left.iter().map(|(_, _, what)| what.as_str()).collect();
        return Err(Fail {
            code: EXIT_FAILED,
            message: format!("asked to let go, but still held after 5 s: {}", names.join(", ")),
        });
    }
    if said == 0 {
        println!("nothing to let go of for \"{thread}\"");
    }
    Ok(0)
}

fn status(home: &Home, opts: &Opts) -> Result<i32, Fail> {
    let now = clock::now_secs();
    for lane in opts.lanes() {
        let view = queue::scan(home, lane, None);
        for (name, slot) in &view.slots {
            match slot {
                Slot::Free => println!("{name}: free"),
                Slot::Held(lock) => {
                    let until = lock.until.map_or_else(
                        || " (made by hand)".to_owned(),
                        |until| format!(", until {}", clock::hhmm(until)),
                    );
                    println!(
                        "{name}: held by \"{}\" for {}: {}{until}",
                        lock.who.thread,
                        clock::span(lock.held),
                        lock.who.job
                    );
                }
                Slot::Stale(lock, why) => println!(
                    "{name}: \"{}\" ({}) no longer counts: {}; the first in line takes it over",
                    lock.who.thread,
                    lock.who.job,
                    why.reason()
                ),
            }
        }
        let waiting: Vec<&queue::Ticket> = view.waiting().collect();
        if waiting.is_empty() {
            println!("  nobody waits for the {} lane", lane.name());
        }
        for (at, ticket) in waiting.iter().enumerate() {
            println!(
                "  {}. \"{}\" waiting {} ({}): {}",
                at + 1,
                ticket.who.thread,
                clock::span(now.saturating_sub(ticket.joined)),
                if ticket.kind == Kind::Take { "take" } else { "run" },
                ticket.who.job
            );
        }
    }
    Ok(0)
}

fn spawn(program: &str, args: &[String]) -> Result<Child, Fail> {
    Command::new(program).args(args).spawn().map_err(|error| Fail {
        code: EXIT_FAILED,
        message: format!("could not start `{program}`: {error}"),
    })
}

fn code_of(status: ExitStatus) -> i32 {
    status.code().unwrap_or(EXIT_FAILED)
}

fn describe(status: ExitStatus) -> String {
    status
        .code()
        .map_or_else(|| "no exit code".to_owned(), |code| format!("exit code {code}"))
}

/// Stops a job and everything it started (`cargo` starts the program it runs, for example).
fn stop_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let taskkill = std::env::var_os("SystemRoot").map_or_else(
            || PathBuf::from("taskkill.exe"),
            |root| PathBuf::from(root).join("System32").join("taskkill.exe"),
        );
        let _ = Command::new(taskkill)
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn a_run_takes_its_command_after_the_double_dash() {
        let opts = parse(
            &args(&[
                "--thread",
                "T",
                "--job",
                "j",
                "--lane",
                "cloud",
                "--",
                "cargo",
                "--minutes",
                "3",
            ]),
            true,
        )
        .unwrap();
        assert_eq!(opts.lane(), Lane::Cloud);
        assert_eq!(opts.command, args(&["cargo", "--minutes", "3"]));
        assert_eq!(opts.minutes, None);
        assert_eq!(opts.who().unwrap(), Who::new("T", "j"));
    }

    #[test]
    fn wrong_command_lines_are_refused() {
        assert!(parse(&args(&["--lane", "moon"]), true).is_err());
        assert!(parse(&args(&["--minutes", "0"]), true).is_err());
        assert!(parse(&args(&["--minutes", "241"]), true).is_err());
        assert!(parse(&args(&["--minutes", "NaN"]), true).is_err());
        assert!(parse(&args(&["--thread"]), true).is_err());
        assert!(parse(&args(&["--", "cargo"]), false).is_err());
        assert!(parse(&args(&["--colour", "red"]), false).is_err());
        let opts = parse(&args(&["--job", "j"]), false).unwrap();
        assert!(opts.who().is_err(), "--thread is required");
        let opts = parse(&args(&["--thread", " ", "--job", "j"]), false).unwrap();
        assert!(opts.who().is_err(), "a blank thread name is refused");
    }

    #[test]
    fn minutes_become_whole_seconds() {
        assert_eq!(secs(60.0), 3600);
        assert_eq!(secs(0.05), 3);
        assert_eq!(secs(0.001), 1);
        assert_eq!(parse(&args(&["--minutes", "0.5"]), false).unwrap().minutes, Some(0.5));
    }

    #[test]
    fn without_a_lane_status_and_release_look_at_both() {
        let opts = parse(&args(&["--thread", "T"]), false).unwrap();
        assert_eq!(opts.lanes(), Lane::ALL.to_vec());
        assert_eq!(opts.lane(), Lane::Local);
    }
}
