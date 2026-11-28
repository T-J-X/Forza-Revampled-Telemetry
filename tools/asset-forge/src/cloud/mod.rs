//! The cloud GPU lane: a rented Runpod pod with a 24 GB card that runs the same ComfyUI and
//! the same Blender as this PC. asset-forge starts it, sends it jobs over SSH and deletes it.
//!
//! How it keeps the cost down:
//! - a pod only exists while a job needs one; `cloud down` deletes it, and the pod's own
//!   watchdog deletes it after `idle_minutes` without use or `max_hours` in total, even if
//!   this PC is off ([`pod`]);
//! - no pod may cost more than `max_usd_per_hour`;
//! - every session goes into the spend log ([`spend`]);
//! - the lane as a whole may cost `budget_usd` and no more: at that total no pod starts, and a
//!   pod that starts near it is given only the lifetime the rest of the budget pays for.
//!
//! Everything personal stays outside the repo, in `Dash_Tools\cloud\`: the API key
//! (`runpod.key`, or the `RUNPOD_API_KEY` variable), the SSH key pair, the settings
//! (`lane.json`), the running pod (`state.json`) and the spend log (`spend.jsonl`).
//!
//! ```text
//! asset-forge cloud setup      # once: pick the data centre, create the disk, install everything
//! asset-forge cloud up         # start the pod now (jobs also start it on demand)
//! asset-forge cloud down       # delete the pod and book the session
//! asset-forge cloud status | spend | doctor
//! asset-forge --lane cloud build --set <name>     # any job, on the pod
//! ```

pub mod clock;
pub mod pod;
pub mod remote;
pub mod runpod;
pub mod spend;
pub mod ssh;

use crate::error::{io_err, tool_err, ForgeError, Result};
use crate::lane::{Lane, Lock};
use crate::net;
use crate::paths::Paths;
use runpod::{Api, Created, DataCenter, GpuType, Pod, PodRequest, Runpod, Window};
use serde::{Deserialize, Serialize};
use ssh::{Ssh, Tunnel};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

/// GPU types to ask for, fastest first, as text their names contain. The RTX 4090 is the first
/// choice; the two Blackwell workstation cards are what the secure cloud stocks beside it; the
/// older 24 GB and 48 GB cards follow. The L4 is left out: it is slower than the local card.
pub const GPU_WISHES: &[&str] = &[
    "RTX 4090",
    "RTX PRO 4500",
    "RTX PRO 4000",
    "RTX 3090",
    "RTX A5000",
    "RTX A6000",
    "A40",
];
/// The lane exists for 24 GB and more.
pub const MIN_VRAM_GB: u32 = 24;
/// The top of the price range TJX approved on 2026-11-27 ($0.34 to $0.89 an hour).
pub const DEFAULT_MAX_USD_PER_HOUR: f64 = 0.90;
/// Runpod's price for a standard network volume, per GB and month.
pub const VOLUME_USD_PER_GB_MONTH: f64 = 0.07;
/// The most the lane may cost in total. TJX set it on 2026-11-27, after turning on auto-pay
/// ("all clusters should cost under $50"), so the credit balance no longer limits the spend.
pub const DEFAULT_BUDGET_USD: f64 = 50.0;
/// From this total on, every cloud job says how much of the budget is spent.
pub const DEFAULT_BUDGET_WARN_USD: f64 = 40.0;
/// How many months of Runpod's billing history count towards the budget.
const BUDGET_MONTHS: u32 = 24;
/// A pod is not started when the rest of the budget pays for less than this.
const MIN_POD_SECS: u64 = 10 * 60;
/// A pod this close to its lifetime cap is replaced before a job starts on it (or within a
/// quarter of the cap, when the budget made the cap short).
const RECYCLE_MARGIN_SECS: u64 = 30 * 60;
/// A pod that another run is using is kept, not replaced, while at least this much of its
/// life is left.
const MIN_SHARED_LEFT_SECS: u64 = 5 * 60;
/// How long a new pod may take to come up (image pull included) before it is given up on.
const START_TIMEOUT_SECS: u64 = 15 * 60;
/// How long the pod's services may take to report ready.
const READY_TIMEOUT_SECS: u64 = 8 * 60;

/// The lane's settings, kept in `lane.json`. `cloud setup` writes it; it is plain JSON and may
/// be edited by hand.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// `SECURE` (Runpod's own data centres; needed for the network volume and for SSH).
    pub cloud: String,
    /// The data centre the volume lives in; pods start there.
    pub data_center: String,
    /// The network volume, or `None` to give every pod a fresh disk and install on each start.
    pub volume_id: Option<String>,
    /// The volume's size in GB (or the fresh disk's).
    pub volume_gb: u32,
    /// GPU type ids to try, first choice first.
    pub gpus: Vec<String>,
    /// No pod may cost more than this per hour.
    pub max_usd_per_hour: f64,
    /// The container image.
    pub image: String,
    /// The container disk in GB (system libraries and temporary files).
    pub container_disk_gb: u32,
    /// Lowest host CUDA version.
    pub min_cuda: String,
    /// The pod deletes itself after this many minutes without use.
    pub idle_minutes: u32,
    /// The pod deletes itself at this age, whatever it is doing.
    pub max_hours: u32,
    /// The port on this PC that forwards to the pod's ComfyUI. The local ComfyUI keeps 8188.
    pub tunnel_port: u16,
    /// The most the lane may cost in total, in USD. At this total no pod starts, and every job
    /// runs on the local lane: TJX chose that over raising it (2026-11-28). Only TJX raises it.
    pub budget_usd: f64,
    /// From this total on, every cloud job says how much of the budget is spent.
    pub budget_warn_usd: f64,
    /// Whether the volume's storage counts towards the budget. TJX chose GPU time only on
    /// 2026-11-27, after growing the volume to 400 GB: storage is shown, not counted.
    pub budget_counts_storage: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cloud: "SECURE".to_owned(),
            data_center: String::new(),
            volume_id: None,
            volume_gb: 30,
            gpus: Vec::new(),
            max_usd_per_hour: DEFAULT_MAX_USD_PER_HOUR,
            image: pod::DEFAULT_IMAGE.to_owned(),
            container_disk_gb: 20,
            min_cuda: pod::MIN_CUDA.to_owned(),
            idle_minutes: 10,
            max_hours: 4,
            tunnel_port: 8189,
            budget_usd: DEFAULT_BUDGET_USD,
            budget_warn_usd: DEFAULT_BUDGET_WARN_USD,
            budget_counts_storage: false,
        }
    }
}

/// Where the lane stands against its budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Budget {
    /// Under the warning line.
    Fine,
    /// At or over the warning line: jobs run, and each says how much is spent.
    Warn,
    /// At or over the budget: no pod starts.
    Spent,
}

impl Config {
    fn idle_secs(&self) -> u64 {
        u64::from(self.idle_minutes.max(2)) * 60
    }

    fn max_secs(&self) -> u64 {
        u64::from(self.max_hours.max(1)) * 3600
    }

    /// Where a total of `spent_usd` stands against the budget. A total that is not a number
    /// counts as spent, so a damaged figure can never open the lane.
    #[must_use]
    pub fn budget(&self, spent_usd: f64) -> Budget {
        if spent_usd.is_nan() || self.budget_usd.is_nan() || spent_usd >= self.budget_usd {
            Budget::Spent
        } else if spent_usd >= self.budget_warn_usd {
            Budget::Warn
        } else {
            Budget::Fine
        }
    }

    /// How long a new pod may live when `spent_usd` is already gone: `max_hours`, or the time
    /// the rest of the budget pays for at the price cap when that is shorter. The pod's own
    /// watchdog enforces it, so the budget holds even if this PC is off. `None` when the rest
    /// pays for less than [`MIN_POD_SECS`].
    #[must_use]
    pub fn pod_life_secs(&self, spent_usd: f64) -> Option<u64> {
        if self.budget(spent_usd) == Budget::Spent || self.max_usd_per_hour.is_nan() || self.max_usd_per_hour <= 0.0 {
            return None;
        }
        let paid_for = ((self.budget_usd - spent_usd) / self.max_usd_per_hour * 3600.0).floor();
        let cap = self.max_secs();
        // `paid_for` is positive here; anything at or over the cap (infinity included) is the cap.
        let secs = if paid_for >= cap as f64 { cap } else { paid_for as u64 };
        (secs >= MIN_POD_SECS).then_some(secs)
    }
}

/// What the lane has cost so far, in USD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spent {
    /// Pod time: the spend log plus the pod that is up now, or Runpod's own billing when that
    /// is higher (it also counts a pod that was not started through asset-forge).
    pub pods: f64,
    /// Volume storage, as far as Runpod has billed it.
    pub storage: f64,
    /// Lines of the spend log that are not a session. While there are any, the budget cannot
    /// be counted and no pod starts.
    pub damaged: usize,
}

impl Spent {
    /// Puts the figures together. `logged` and `running` are this PC's own count; `billed_pods`
    /// and `billed_storage` are Runpod's, which run hours behind.
    #[must_use]
    pub fn from_parts(logged: f64, running: f64, billed_pods: f64, billed_storage: f64) -> Self {
        Self {
            pods: (logged + running).max(billed_pods),
            storage: billed_storage,
            damaged: 0,
        }
    }

    /// Pods and storage together.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.pods + self.storage
    }

    /// What counts towards the budget: pod time, plus storage when the settings say so.
    #[must_use]
    pub fn counted(&self, config: &Config) -> f64 {
        if config.budget_counts_storage {
            self.total()
        } else {
            self.pods
        }
    }
}

/// The pod that is up right now, kept in `state.json` until its session is booked.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    /// The pod's id.
    pub pod_id: String,
    /// Its GPU type.
    pub gpu: String,
    /// Its data centre.
    pub data_center: String,
    /// Its hourly rate in USD.
    pub usd_per_hour: f64,
    /// When it was created (Unix seconds).
    pub started: u64,
    /// When asset-forge last finished using it (Unix seconds).
    pub touched: u64,
    /// The thread that held the cloud lock when it was started.
    pub thread: String,
    /// The job named in that lock.
    pub job: String,
    /// The age at which its watchdog deletes it, in seconds: `max_hours`, or less when the
    /// budget was nearly spent. 0 in a record written before this field existed.
    #[serde(default)]
    pub max_secs: u64,
}

impl State {
    /// The pod's lifetime cap in seconds.
    fn cap(&self, config: &Config) -> u64 {
        if self.max_secs > 0 {
            self.max_secs
        } else {
            config.max_secs()
        }
    }
}

/// The lane's folder next to the tools.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Home {
    dir: PathBuf,
}

impl Home {
    /// `Dash_Tools\cloud`.
    #[must_use]
    pub fn of(paths: &Paths) -> Self {
        Self {
            dir: paths.tools.join("cloud"),
        }
    }

    fn key_file(&self) -> PathBuf {
        self.dir.join("runpod.key")
    }

    /// The private SSH key for pods (the public half is next to it with `.pub`).
    #[must_use]
    pub fn ssh_key(&self) -> PathBuf {
        self.dir.join("id_ed25519")
    }

    fn config_file(&self) -> PathBuf {
        self.dir.join("lane.json")
    }

    fn state_file(&self) -> PathBuf {
        self.dir.join("state.json")
    }

    /// The spend log.
    #[must_use]
    pub fn spend_log(&self) -> PathBuf {
        self.dir.join("spend.jsonl")
    }

    fn known_hosts(&self, pod_id: &str) -> PathBuf {
        self.dir.join(format!("known_hosts_{}", crate::lane::safe_name(pod_id)))
    }

    /// The Runpod API key: the `RUNPOD_API_KEY` variable, or the first line of `runpod.key`.
    pub fn api_key(&self) -> Result<String> {
        self.api_key_from(std::env::var("RUNPOD_API_KEY").ok())
    }

    /// [`Home::api_key`] with the variable's value passed in.
    fn api_key_from(&self, env: Option<String>) -> Result<String> {
        let from_env = env.filter(|k| !k.trim().is_empty());
        let key = from_env.or_else(|| std::fs::read_to_string(self.key_file()).ok());
        key.as_deref()
            .and_then(|k| k.lines().next())
            .map(|k| k.trim().to_owned())
            .filter(|k| !k.is_empty())
            .ok_or_else(|| {
                tool_err(
                    "cloud",
                    format!(
                        "no Runpod API key. Save it as the only line of {} (or set RUNPOD_API_KEY)",
                        self.key_file().display()
                    ),
                )
            })
    }

    fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
        let text = std::fs::read_to_string(path).map_err(|e| io_err(path, e))?;
        serde_json::from_str(&text).map_err(|source| ForgeError::Json {
            context: path.display().to_string(),
            source,
        })
    }

    fn write_json<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        std::fs::create_dir_all(&self.dir).map_err(|e| io_err(&self.dir, e))?;
        let text = serde_json::to_string_pretty(value).map_err(|source| ForgeError::Json {
            context: path.display().to_string(),
            source,
        })?;
        // Written to a file of this process's own and then renamed over the old one, so a run
        // that reads the file while another run writes it never sees half of it.
        let part = path.with_extension(format!("{}.part", std::process::id()));
        std::fs::write(&part, text + "\n").map_err(|e| io_err(&part, e))?;
        std::fs::rename(&part, path).map_err(|e| {
            let _ = std::fs::remove_file(&part);
            io_err(path, e)
        })
    }

    /// The settings, or an error that says to run `cloud setup`.
    pub fn config(&self) -> Result<Config> {
        let path = self.config_file();
        if !path.is_file() {
            return Err(tool_err(
                "cloud",
                "the cloud lane is not set up on this PC. Run `asset-forge cloud setup` first",
            ));
        }
        Self::read_json(&path)
    }

    fn save_config(&self, config: &Config) -> Result<()> {
        self.write_json(&self.config_file(), config)
    }

    /// The pod on record, if any.
    #[must_use]
    pub fn state(&self) -> Option<State> {
        Self::read_json(&self.state_file()).ok()
    }

    fn save_state(&self, state: Option<&State>) -> Result<()> {
        let path = self.state_file();
        match state {
            Some(state) => self.write_json(&path, state),
            None if path.exists() => std::fs::remove_file(&path).map_err(|e| io_err(&path, e)),
            None => Ok(()),
        }
    }
}

fn price(gpu: &GpuType, cloud: &str) -> f64 {
    if cloud.eq_ignore_ascii_case("COMMUNITY") {
        gpu.price.community
    } else {
        gpu.price.secure
    }
}

/// The catalog's GPU types that fit the lane, in the order of `wishes`: at least
/// [`MIN_VRAM_GB`], offered on `cloud`, and priced at or under `cap` per hour.
#[must_use]
pub fn pick_gpus(catalog: &[GpuType], wishes: &[String], cloud: &str, cap: f64) -> Vec<GpuType> {
    let mut picked: Vec<GpuType> = Vec::new();
    for wish in wishes {
        let wish = wish.to_ascii_lowercase();
        for gpu in catalog {
            let named = gpu.name.to_ascii_lowercase().contains(&wish) || gpu.id.to_ascii_lowercase().contains(&wish);
            let offered = cloud.eq_ignore_ascii_case("COMMUNITY") || gpu.secure;
            let cost = price(gpu, cloud);
            let fits = gpu.memory >= MIN_VRAM_GB && offered && cost > 0.0 && cost <= cap + 1e-9;
            if named && fits && !picked.iter().any(|p| p.id == gpu.id) {
                picked.push(gpu.clone());
            }
        }
    }
    picked
}

fn stock_in(gpu: &GpuType, data_center: &str) -> u8 {
    gpu.data_centers
        .iter()
        .find(|d| d.id == data_center)
        .map_or(0, |d| runpod::stock_rank(&d.availability))
}

/// Picks the data centre for the lane. With `need_volume`, only data centres that take
/// standard network volumes count. A data centre where the first-choice GPU is in medium or
/// high stock wins; then the earlier region in `regions`; then more stock of the first choice;
/// then more stock of the fallbacks. `None` when no data centre has any of the GPUs.
#[must_use]
pub fn pick_data_center(
    gpus: &[GpuType],
    centers: &[DataCenter],
    regions: &[String],
    need_volume: bool,
) -> Option<String> {
    let first = gpus.first()?;
    centers
        .iter()
        .filter(|dc| !need_volume || dc.network_volume_types.iter().any(|t| t == "STANDARD"))
        .filter_map(|dc| {
            let first_stock = stock_in(first, &dc.id);
            let rest: u32 = gpus.iter().skip(1).map(|g| u32::from(stock_in(g, &dc.id))).sum();
            if u32::from(first_stock) + rest == 0 {
                return None;
            }
            let region = regions
                .iter()
                .position(|r| r.eq_ignore_ascii_case(&dc.region))
                .map_or(0, |at| regions.len() - at);
            Some(((first_stock >= 2, region, first_stock, rest), dc.id.clone()))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, id)| id)
}

/// A pod that is up, reachable and ready, with the forward to its ComfyUI open.
#[derive(Debug)]
pub struct Session {
    /// The pod's SSH address.
    pub ssh: Ssh,
    /// The pod's ComfyUI as seen from this PC.
    pub comfy_url: String,
    /// A name for this run, unique among the runs that share the pod. This run's job folders
    /// and uploads on the pod carry it, so two runs never touch each other's files.
    pub tag: String,
    home: Home,
    _tunnel: Tunnel,
    _user: UserLock,
}

impl Session {
    /// Marks the pod on record as used now. If this run is killed, the estimate of when the pod
    /// ended starts from the last of these marks.
    fn touch(&self) {
        if let Some(mut state) = self.home.state() {
            state.touched = clock::now();
            let _ = self.home.save_state(Some(&state));
        }
    }
}

/// The session of this process. It is opened by the first job that needs the pod and kept
/// until [`close`], so the forward and the pod's "in use" mark last for the whole run.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Runs `job` with the pod, starting the pod first when it is not up. The session stays locked
/// while `job` runs, so `job` must not call this function again.
pub fn with_session<T>(paths: &Paths, job: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
    let mut slot = SESSION
        .lock()
        .map_err(|_| tool_err("cloud", "the session lock is poisoned"))?;
    if slot.is_none() {
        *slot = Some(open(paths)?);
    }
    let Some(session) = slot.as_ref() else {
        return Err(tool_err("cloud", "the session did not open"));
    };
    session.touch();
    let result = job(session);
    session.touch();
    result
}

/// The pod's ComfyUI address on this PC and this run's tag, starting the pod when needed.
pub fn comfy_endpoint(paths: &Paths) -> Result<(String, String)> {
    with_session(paths, |session| Ok((session.comfy_url.clone(), session.tag.clone())))
}

/// Ends this process's session: the forward closes and the pod's idle clock starts. The pod
/// itself stays up for the next command until its watchdog or `cloud down` deletes it.
pub fn close() {
    let Ok(mut slot) = SESSION.lock() else { return };
    if let Some(session) = slot.take() {
        session.touch();
    }
}

/// This run's tag: a name unique among the runs that share the pod, made from the process id.
#[must_use]
pub fn run_tag() -> String {
    format!("run{}", std::process::id())
}

/// Windows' `ERROR_SHARING_VIOLATION`: another process has the file open and shares nothing.
const SHARING_VIOLATION: i32 = 32;

/// Opens `path` so that no other handle can open it while this one lives. Windows closes the
/// handle by itself when the process ends, so a run that crashed leaves nothing held. (Other
/// systems open the file without this guard; the tool runs on Windows.)
fn open_exclusive(path: &Path) -> std::io::Result<std::fs::File> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt as _;
        // Share nothing: every other open of the file fails while this handle lives.
        options.share_mode(0);
    }
    options.open(path)
}

/// True when an open failed because another handle holds the file.
fn held_elsewhere(e: &std::io::Error) -> bool {
    e.raw_os_error() == Some(SHARING_VIOLATION)
}

/// The longest a run waits for another run to finish starting the pod: a first install on an
/// empty volume took 21 minutes.
const START_WAIT_SECS: u64 = 30 * 60;

/// Held by the one run that is starting, checking or installing the pod. Other runs wait for
/// it, so two runs never start two pods, run the installer at the same time or overwrite each
/// other's record. It is let go as soon as the pod is ready, and the runs then share the pod.
#[derive(Debug)]
struct StartLock {
    _file: std::fs::File,
}

impl StartLock {
    fn path(home: &Home) -> PathBuf {
        home.dir.join("start.lock")
    }

    /// Takes the lock now, or `None` while another run holds it.
    fn try_take(home: &Home) -> Result<Option<Self>> {
        match open_exclusive(&Self::path(home)) {
            Ok(file) => Ok(Some(Self { _file: file })),
            Err(e) if held_elsewhere(&e) => Ok(None),
            Err(e) => Err(io_err(&Self::path(home), e)),
        }
    }

    /// Takes the lock, waiting while another run starts the pod.
    fn take(home: &Home) -> Result<Self> {
        let deadline = clock::now() + START_WAIT_SECS;
        let mut told = false;
        loop {
            if let Some(lock) = Self::try_take(home)? {
                return Ok(lock);
            }
            if clock::now() > deadline {
                return Err(tool_err(
                    "cloud",
                    "another asset-forge run has been starting the pod for 30 minutes; `asset-forge cloud status` shows the pod",
                ));
            }
            if !told {
                say("waiting for another run to finish starting the pod");
                told = true;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    }
}

/// Held by each run for as long as it uses the pod, one file per run in `runs\`, so a command
/// can tell whether other runs still use the pod before it deletes or replaces it.
#[derive(Debug)]
struct UserLock {
    file: Option<std::fs::File>,
    path: PathBuf,
}

impl UserLock {
    fn folder(home: &Home) -> PathBuf {
        home.dir.join("runs")
    }

    /// Marks this process as a run that uses the pod.
    fn take(home: &Home) -> Result<Self> {
        let path = Self::folder(home).join(format!("{}.lock", std::process::id()));
        let file = open_exclusive(&path).map_err(|e| io_err(&path, e))?;
        Ok(Self { file: Some(file), path })
    }

    /// How many runs other than `own` use the pod now. Files left by runs that ended are
    /// removed on the way.
    fn others(home: &Home, own: Option<&Self>) -> usize {
        let Ok(entries) = std::fs::read_dir(Self::folder(home)) else {
            return 0;
        };
        let mut count = 0;
        for path in entries.filter_map(|e| e.ok().map(|e| e.path())) {
            if own.is_some_and(|own| own.path == path) {
                continue;
            }
            match open_exclusive(&path) {
                Ok(file) => {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                }
                Err(e) if held_elsewhere(&e) => count += 1,
                Err(_) => {}
            }
        }
        count
    }
}

impl Drop for UserLock {
    fn drop(&mut self) {
        // The handle first: Windows refuses to delete a file that is still open this way.
        drop(self.file.take());
        let _ = std::fs::remove_file(&self.path);
    }
}

/// The port on this PC for the forward: `preferred` when it is free, else any free one, so a
/// second run that shares the pod gets a forward of its own.
fn free_port(preferred: u16) -> Result<u16> {
    if std::net::TcpListener::bind(("127.0.0.1", preferred)).is_ok() {
        return Ok(preferred);
    }
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map(|address| address.port())
        .map_err(|e| tool_err("cloud", format!("no free port on this PC for the forward: {e}")))
}

fn say(text: &str) {
    println!("  cloud: {text}");
}

/// The hourly rate a pod is booked at: its own price, or the price cap when the answer gave
/// none, so a missing figure never books a pod as free.
fn rate_of(pod: &Pod, config: &Config) -> f64 {
    if pod.cost.is_finite() && pod.cost > 0.0 {
        pod.cost
    } else {
        config.max_usd_per_hour
    }
}

/// The age at which a pod's watchdog deletes it: what the pod was created with, or `max_hours`
/// when the answer did not include its environment.
fn life_of(pod: &Pod, config: &Config) -> u64 {
    pod.env_number("FORGE_MAX_SECS")
        .filter(|secs| *secs > 0)
        .unwrap_or_else(|| config.max_secs())
}

/// The lock fields for a pod that this PC has no record of.
fn no_record() -> Lock {
    Lock {
        thread: "(no record on this PC)".to_owned(),
        ..Lock::default()
    }
}

/// Deletes a pod that is up and books its session up to now.
fn retire(api: &dyn Api, home: &Home, state: &State, how: &str) -> Result<spend::Session> {
    api.terminate(&state.pod_id)?;
    let session = spend::Session::close(state, clock::now(), how);
    book(home, state, &session)?;
    Ok(session)
}

/// Writes a finished session to the spend log, and forgets the pod when it is the one on
/// record. A pod that the log already has is not booked again.
fn book(home: &Home, state: &State, session: &spend::Session) -> Result<()> {
    spend::append(&home.spend_log(), session)?;
    if home.state().is_some_and(|recorded| recorded.pod_id == state.pod_id) {
        home.save_state(None)?;
    }
    let _ = std::fs::remove_file(home.known_hosts(&state.pod_id));
    mirror_spend(home);
    Ok(())
}

/// When the pod's watchdog deleted it: `touched` plus the idle time, or the lifetime cap,
/// whichever came first, plus one look of the watchdog, and never later than now. The first
/// pod that was left idle (2026-11-27) was gone 10 minutes and 4 to 15 seconds after its last
/// use, so the estimate is at most half a minute long.
fn watchdog_end(state: &State, config: &Config, now: u64) -> u64 {
    let idle_end = state.touched.max(state.started) + config.idle_secs() + pod::WATCHDOG_POLL_SECS;
    let life_end = state.started + state.cap(config) + pod::WATCHDOG_POLL_SECS;
    now.min(idle_end).min(life_end)
}

/// Books a pod that ended without `cloud down`: its watchdog deleted it, or it stopped. The end
/// is estimated from its last use. When Runpod has already billed more for it, that amount is
/// booked instead, so the log never holds less than the pod cost.
fn book_lost(api: &dyn Api, home: &Home, config: &Config, state: &State) -> Result<()> {
    let ended = watchdog_end(state, config, clock::now());
    let mut session = spend::Session::close(state, ended, "the pod's watchdog (end time estimated)");
    if let Ok(billed) = api.pod_billing(Window::Days(3), Some(&state.pod_id)) {
        let billed = (billed * 1000.0).round() / 1000.0;
        if billed > session.usd {
            session.usd = billed;
            session.how = "the pod's watchdog (amount from Runpod's billing)".to_owned();
        }
    }
    say(&format!(
        "pod {} ended without `cloud down`; booked {} and ${:.3}",
        state.pod_id,
        clock::span(session.seconds),
        session.usd
    ));
    book(home, state, &session)
}

/// Looks at the pod on record. One that is still up is returned. One that is gone, or stopped,
/// is deleted for good and booked up to its estimated end (never up to now), and its record is
/// cleared.
fn settle(api: &dyn Api, home: &Home, config: &Config) -> Result<Option<(State, Pod)>> {
    let Some(state) = home.state() else {
        return Ok(None);
    };
    match api.pod(&state.pod_id)? {
        Some(pod) if pod.is_alive() => Ok(Some((state, pod))),
        Some(pod) => {
            // Stopped or failed: no compute is billed, but it must not linger.
            api.terminate(&pod.id)?;
            book_lost(api, home, config, &state)?;
            Ok(None)
        }
        None => {
            book_lost(api, home, config, &state)?;
            Ok(None)
        }
    }
}

/// Adds up what the lane has cost: the spend log, the pod that is up now (`live`, which the
/// caller has checked is still running), and Runpod's own billing. Runpod's figures are left
/// out when they cannot be read; the log does not depend on them.
fn spent(api: &dyn Api, home: &Home, live: Option<&State>, now: u64) -> Spent {
    let (sessions, damaged) = spend::load_checked(&home.spend_log());
    let (logged, _) = spend::total(&sessions);
    let running = live.map_or(0.0, |state| {
        spend::cost(state.usd_per_hour, now.saturating_sub(state.started))
    });
    let window = Window::Months(BUDGET_MONTHS);
    let mut spent = Spent::from_parts(
        logged,
        running,
        api.pod_billing(window, None).unwrap_or(0.0),
        api.volume_billing(window).unwrap_or(0.0),
    );
    spent.damaged = damaged;
    spent
}

/// The budget in one line, for status reports, warnings and refusals.
fn budget_line(config: &Config, spent: &Spent) -> String {
    if config.budget_counts_storage {
        format!(
            "${:.2} of the ${:.2} budget is spent (pods ${:.2}, volume storage ${:.2})",
            spent.counted(config),
            config.budget_usd,
            spent.pods,
            spent.storage
        )
    } else {
        format!(
            "${:.2} of the ${:.2} budget is spent on GPU time (volume storage so far ${:.2}, shown, not counted)",
            spent.counted(config),
            config.budget_usd,
            spent.storage
        )
    }
}

/// Prints the budget line of `cloud status` and `cloud spend`.
fn print_budget(config: &Config, spent: &Spent) {
    let note = if spent.damaged > 0 {
        format!(
            "; the spend log has {} damaged line(s), so no pod starts until they are fixed",
            spent.damaged
        )
    } else {
        match config.budget(spent.counted(config)) {
            Budget::Fine => String::new(),
            Budget::Warn => format!("; that is over the ${:.2} warning line", config.budget_warn_usd),
            Budget::Spent => "; it is used up, so every job runs on the local lane, as TJX chose".to_owned(),
        }
    };
    println!("budget: {}{note}", budget_line(config, spent));
}

/// Stops the lane when its budget is used up or cannot be counted, and says so when it is
/// close. When the budget is used up, the pod that is up (`live`) is deleted and booked up to
/// now, unless `others` runs still use it: its deadline already keeps it within the budget.
fn hold_to_budget(
    api: &dyn Api,
    home: &Home,
    config: &Config,
    spent: &Spent,
    live: Option<&State>,
    others: usize,
) -> Result<()> {
    if spent.damaged > 0 {
        return Err(tool_err(
            "cloud",
            format!(
                "{} has {} line(s) that are not a session, so the budget cannot be counted. Fix or remove them, then run the job again",
                home.spend_log().display(),
                spent.damaged
            ),
        ));
    }
    match config.budget(spent.counted(config)) {
        Budget::Fine => Ok(()),
        Budget::Warn => {
            say(&format!("warning: {}", budget_line(config, spent)));
            Ok(())
        }
        Budget::Spent => {
            if let Some(state) = live.filter(|_| others == 0) {
                // If the delete fails, the pod's own lifetime cap still ends it.
                let _ = retire(api, home, state, "deleted: the budget is used up");
            }
            Err(tool_err(
                "cloud",
                format!(
                    "the cloud lane is closed: {}. TJX chose the local lane from here on: take `gpu.lock` (SKILLS.md section 5) and run the same command without `--lane cloud`",
                    budget_line(config, spent)
                ),
            ))
        }
    }
}

/// Copies the spend log as CSV into the Drive masters folder (`shared\cloud-gpu-spend.csv`), so
/// it can be read from another machine. Skipped when the Drive folder is not mounted.
fn mirror_spend(home: &Home) {
    // The tests book sessions in folders of their own; they must never write the real copy.
    if cfg!(test) {
        return;
    }
    let Some(root) = crate::lane::masters_root() else {
        return;
    };
    let shared = root.join("shared");
    if std::fs::create_dir_all(&shared).is_ok() {
        let _ = std::fs::write(
            shared.join("cloud-gpu-spend.csv"),
            spend::csv(&spend::load(&home.spend_log())),
        );
    }
}

fn state_for(pod: &Pod, lock: &Lock, started: u64, max_secs: u64, usd_per_hour: f64) -> State {
    State {
        pod_id: pod.id.clone(),
        gpu: pod.gpu_id().to_owned(),
        data_center: pod.data_center_id.clone().unwrap_or_default(),
        usd_per_hour,
        started,
        touched: clock::now(),
        thread: lock.thread.clone(),
        job: lock.job.clone(),
        max_secs,
    }
}

/// Starts a new pod: the first GPU type in the settings that has room gets it. The pod's
/// lifetime cap is what the rest of the budget pays for, when that is less than `max_hours`,
/// and its deadline is set in its environment, so its own watchdog enforces both.
fn create_pod(api: &dyn Api, home: &Home, config: &Config, lock: &Lock, spent: &Spent) -> Result<(State, Pod)> {
    let Some(life) = config.pod_life_secs(spent.counted(config)) else {
        return Err(tool_err(
            "cloud",
            format!(
                "too little of the budget is left to start a pod: {}. TJX chose the local lane from here on: take `gpu.lock` (SKILLS.md section 5) and run the same command without `--lane cloud`",
                budget_line(config, spent)
            ),
        ));
    };
    if life < config.max_secs() {
        say(&format!(
            "the rest of the budget pays for {} of pod time; this pod deletes itself at that age",
            clock::span(life)
        ));
    }
    let balance = api.balance()?;
    let public_key_file = home.ssh_key().with_extension("pub");
    let public_key = std::fs::read_to_string(&public_key_file).map_err(|e| io_err(&public_key_file, e))?;
    let template = pod::template(&config.image, config.container_disk_gb);
    let template_id = api.put_template(api.template_id(pod::TEMPLATE_NAME)?.as_deref(), &template)?;
    let mut last = String::from("the settings list no GPU type");
    for gpu in &config.gpus {
        let request = PodRequest {
            name: pod::POD_NAME.to_owned(),
            template_id: template_id.clone(),
            gpu_id: gpu.clone(),
            cloud: config.cloud.clone(),
            data_center: config.data_center.clone(),
            min_cuda: config.min_cuda.clone(),
            env: vec![
                ("PUBLIC_KEY".to_owned(), public_key.trim().to_owned()),
                ("FORGE_IDLE_SECS".to_owned(), config.idle_secs().to_string()),
                ("FORGE_MAX_SECS".to_owned(), life.to_string()),
                ("FORGE_DEADLINE".to_owned(), (clock::now() + life).to_string()),
            ],
            volume_id: config.volume_id.clone(),
            own_disk_gb: config.volume_gb,
        };
        match api.create_pod(&request)? {
            Created::Pod(pod) => {
                let state = state_for(&pod, lock, clock::now(), life, rate_of(&pod, config));
                if pod.cost > config.max_usd_per_hour + 1e-6 {
                    retire(api, home, &state, "deleted at once: over the price cap")?;
                    return Err(tool_err(
                        "cloud",
                        format!(
                            "{gpu} now costs ${:.2} an hour, over the ${:.2} limit in lane.json; the pod was deleted at once",
                            pod.cost, config.max_usd_per_hour
                        ),
                    ));
                }
                if let Err(e) = home.save_state(Some(&state)) {
                    // A pod with no record would run unbooked; delete it before giving up.
                    let _ = api.terminate(&pod.id);
                    return Err(e);
                }
                say(&format!(
                    "started pod {} on {gpu} in {} at ${:.2} an hour (balance ${:.2})",
                    pod.id, config.data_center, state.usd_per_hour, balance.balance_amount
                ));
                return Ok((state, *pod));
            }
            Created::Unavailable(why) => {
                say(&format!("no {gpu} in {} right now ({why})", config.data_center));
                last = why;
            }
        }
    }
    Err(tool_err(
        "cloud",
        format!(
            "no pod could be started in {} ({last}). Try again later, or run the job on the local lane",
            config.data_center
        ),
    ))
}

/// True when a pod of this age, with this lifetime cap, still has enough time left to start a
/// job on: more than [`RECYCLE_MARGIN_SECS`], or more than a quarter of a short cap.
fn worth_keeping(age_secs: u64, cap_secs: u64) -> bool {
    age_secs + RECYCLE_MARGIN_SECS.min(cap_secs / 4) < cap_secs
}

/// Deals with pods of ours that this PC has no record of (a run that crashed before it saved
/// one, or a pod started by hand). The first one that is up and within the price cap is
/// adopted; every other one that is up is deleted and booked from its creation, and stopped
/// ones are deleted. `skip` is a pod this run has just deleted, which the provider's list may
/// still show.
fn adopt_strays(
    api: &dyn Api,
    home: &Home,
    config: &Config,
    lock: &Lock,
    skip: Option<&str>,
) -> Result<Option<(State, Pod)>> {
    let mut adopted = None;
    let listed = api.pods()?;
    for id in listed
        .iter()
        .filter(|p| p.name == pod::POD_NAME && Some(p.id.as_str()) != skip)
        .map(|p| p.id.as_str())
    {
        let Some(pod) = api.pod(id)? else { continue };
        if !pod.is_alive() {
            api.terminate(&pod.id)?;
            continue;
        }
        let started = clock::parse_rfc3339(&pod.created_at).unwrap_or_else(clock::now);
        let fits = pod.cost <= config.max_usd_per_hour + 1e-6;
        if adopted.is_none() && fits {
            let state = state_for(&pod, lock, started, life_of(&pod, config), rate_of(&pod, config));
            home.save_state(Some(&state))?;
            say(&format!("found pod {} already running; using it", pod.id));
            adopted = Some((state, pod));
        } else {
            let state = state_for(
                &pod,
                &no_record(),
                started,
                life_of(&pod, config),
                rate_of(&pod, config),
            );
            say(&format!(
                "deleting pod {}: one pod at a time, within the price cap",
                pod.id
            ));
            retire(api, home, &state, "deleted: an extra pod with no record")?;
        }
    }
    Ok(adopted)
}

/// Returns the pod to use: `live` (the pod on record, still up) when it has time left, else a
/// pod of ours that this PC has no record of, else a new one. Never more than one.
fn ensure_pod(
    api: &dyn Api,
    home: &Home,
    config: &Config,
    lock: &Lock,
    spent: &Spent,
    live: Option<(State, Pod)>,
    others: usize,
) -> Result<(State, Pod)> {
    let mut retired = None;
    if let Some((state, pod)) = live {
        let age = clock::now().saturating_sub(state.started);
        let cap = state.cap(config);
        if worth_keeping(age, cap) {
            return Ok((state, pod));
        }
        if others > 0 {
            // Another run is using it, so it is not replaced under that run.
            let left = cap.saturating_sub(age);
            if left < MIN_SHARED_LEFT_SECS {
                return Err(tool_err(
                    "cloud",
                    format!(
                        "the pod ends in {} and another run is using it; run the job again once it has ended",
                        clock::span(left)
                    ),
                ));
            }
            say(&format!(
                "the pod ends in {}; another run is using it, so it is kept",
                clock::span(left)
            ));
            return Ok((state, pod));
        }
        say("the pod is close to its lifetime cap; replacing it before the job starts");
        retire(api, home, &state, "replaced before the lifetime cap")?;
        retired = Some(state.pod_id);
    }
    if let Some(found) = adopt_strays(api, home, config, lock, retired.as_deref())? {
        return Ok(found);
    }
    create_pod(api, home, config, lock, spent)
}

/// Everything before the pod is reached: books a pod on record that has ended, holds the lane
/// to its budget, and returns the pod to use, starting one when needed.
fn prepare(api: &dyn Api, home: &Home, config: &Config, lock: &Lock, now: u64, others: usize) -> Result<(State, Pod)> {
    let live = settle(api, home, config)?;
    let spent = spent(api, home, live.as_ref().map(|(state, _)| state), now);
    hold_to_budget(api, home, config, &spent, live.as_ref().map(|(state, _)| state), others)?;
    ensure_pod(api, home, config, lock, &spent, live, others)
}

/// Waits until the pod runs and its sshd takes the key.
fn wait_for_ssh(api: &dyn Api, home: &Home, state: &State, mut pod: Pod) -> Result<Ssh> {
    let deadline = clock::now() + START_TIMEOUT_SECS;
    let mut told = false;
    loop {
        if pod.status == "RUNNING" {
            if let Some(direct) = &pod.ssh.direct {
                let ssh = Ssh {
                    host: direct.host.clone(),
                    port: direct.port,
                    user: direct.username.clone(),
                    key: home.ssh_key(),
                    known_hosts: home.known_hosts(&pod.id),
                };
                if ssh.reachable() {
                    return Ok(ssh);
                }
            }
        }
        if !pod.is_alive() {
            retire(api, home, state, "deleted: it stopped while starting")?;
            return Err(tool_err(
                "cloud",
                format!("the pod stopped while starting (status {}); it was deleted", pod.status),
            ));
        }
        if clock::now() > deadline {
            retire(api, home, state, "deleted: it did not start in time")?;
            return Err(tool_err(
                "cloud",
                format!(
                    "the pod did not come up within {} minutes; it was deleted",
                    START_TIMEOUT_SECS / 60
                ),
            ));
        }
        if !told {
            say("waiting for the pod to boot");
            told = true;
        }
        std::thread::sleep(Duration::from_secs(5));
        pod = api
            .pod(&state.pod_id)?
            .ok_or_else(|| tool_err("cloud", "the pod disappeared while starting"))?;
    }
}

/// Installs the pod's disk when it is new or its pins are out of date, then waits for ComfyUI
/// and Blender to report ready.
fn make_ready(ssh: &Ssh) -> Result<()> {
    let have = ssh.check("cat /workspace/forge/provisioned 2>/dev/null || true")?;
    if have.trim() != pod::stamp() {
        say("installing ComfyUI, the models and Blender on the pod's disk (first use, or a pin changed)");
        if !ssh.run_live(&pod::provision_script())? {
            return Err(tool_err("cloud", "the pod's set-up failed; the lines above say where"));
        }
        return start_services(ssh);
    }
    // The pod's start command is bringing the services up; wait for its verdict. A "ready"
    // left by an earlier session counts only while ComfyUI still answers.
    let probe = format!(
        "if [ -f /run/forge/ready ]; then if curl -fsS -m 5 http://127.0.0.1:{}/system_stats >/dev/null 2>&1; then echo ready; else echo dead; fi; elif [ -f /run/forge/failed ]; then echo failed; else echo wait; fi",
        pod::COMFY_PORT
    );
    let deadline = clock::now() + READY_TIMEOUT_SECS;
    loop {
        match ssh.check(&probe)?.trim() {
            "ready" => return Ok(()),
            "wait" if clock::now() < deadline => std::thread::sleep(Duration::from_secs(4)),
            // It failed, died or is too slow: start it from here, with the output visible.
            _ => return start_services(ssh),
        }
    }
}

/// Starts ComfyUI and installs Blender's libraries from this side, and reports why if it fails.
fn start_services(ssh: &Ssh) -> Result<()> {
    if ssh.run_live(&pod::services_script())? {
        return Ok(());
    }
    let log = ssh.check("tail -n 20 /run/forge/apt.log /workspace/forge/comfyui.log 2>/dev/null || true")?;
    Err(tool_err("cloud", format!("the pod's services did not start\n{log}")))
}

/// Opens the forward to the pod's ComfyUI and waits until it answers through it.
fn open_tunnel(ssh: &Ssh, preferred: u16) -> Result<(Tunnel, String)> {
    // A port nothing listens on, so no forward of another run, or of an earlier one, can answer
    // in the pod's place.
    let port = free_port(preferred)?;
    let url = format!("http://127.0.0.1:{port}");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .http_status_as_error(true)
        .build()
        .into();
    let mut tunnel = ssh.tunnel(port, pod::COMFY_PORT)?;
    for _ in 0..60 {
        if !tunnel.alive() {
            break;
        }
        if net::get_bytes(&agent, &format!("{url}/system_stats")).is_ok() {
            return Ok((tunnel, url));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(tool_err(
        "cloud",
        format!("the SSH forward to the pod's ComfyUI on port {port} did not open"),
    ))
}

fn open(paths: &Paths) -> Result<Session> {
    let home = Home::of(paths);
    let config = home.config()?;
    let lock = Lock::require(&paths.tools, Lane::Cloud)?;
    let user = UserLock::take(&home)?;
    let api = Runpod::new(&home.api_key()?);
    let began = clock::now();
    let start = StartLock::take(&home)?;
    let others = UserLock::others(&home, Some(&user));
    let (state, pod) = prepare(&api, &home, &config, &lock, began, others)?;
    let ssh = wait_for_ssh(&api, &home, &state, pod)?;
    make_ready(&ssh)?;
    // The pod is ready: other runs may start using it from here.
    drop(start);
    let (tunnel, comfy_url) = open_tunnel(&ssh, config.tunnel_port)?;
    // Say how long the wait was, and whether it included starting a pod, so the start-up time
    // in the guide can be checked against a real run.
    let waited = clock::span(clock::now().saturating_sub(began));
    let how = if state.started >= began {
        "a new pod"
    } else {
        "the pod that was already up"
    };
    let shared = if others > 0 {
        format!(", shared with {others} other run(s)")
    } else {
        String::new()
    };
    say(&format!("pod {} is ready after {waited} ({how}{shared})", state.pod_id));
    Ok(Session {
        ssh,
        comfy_url,
        tag: run_tag(),
        home,
        _tunnel: tunnel,
        _user: user,
    })
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|at| args.get(at + 1))
        .map(String::as_str)
}

fn number<T: std::str::FromStr>(args: &[String], flag: &str) -> Result<Option<T>> {
    match flag_value(args, flag) {
        None => Ok(None),
        Some(text) => text
            .parse::<T>()
            .map(Some)
            .map_err(|_| ForgeError::Usage(format!("{flag} needs a number, not {text}"))),
    }
}

const USAGE: &str = "asset-forge cloud setup [--dry-run] [--region EUROPE]... [--data-center <id>] [--volume-gb 30 | --no-volume] [--max-usd-per-hour 0.90] [--idle-minutes 10] [--max-hours 4] [--image <ref>] [--keep] | key | up | down | status | spend | doctor";

/// The `cloud` commands.
pub fn cli(paths: &Paths, args: &[String]) -> Result<()> {
    let home = Home::of(paths);
    match args.first().map(String::as_str) {
        Some("setup") => setup(paths, &home, &args[1..]),
        Some("key") => set_key(&home, args.get(1).map(String::as_str)),
        Some("up") => with_session(paths, |_| Ok(())).and_then(|()| status(&home)),
        Some("down") => down(&home),
        Some("status") => status(&home),
        Some("spend") => spend_report(&home),
        Some("doctor") => doctor(paths),
        _ => Err(ForgeError::Usage(USAGE.to_owned())),
    }
}

/// Saves a new API key once Runpod has accepted it. The key is read from standard input (the
/// command asks for it), so it stays out of the command line, the shell's history and cargo's
/// "Running" line. A key given as an argument still works, with a warning. The key itself is
/// never printed.
fn set_key(home: &Home, key: Option<&str>) -> Result<()> {
    let key = match key {
        Some(key) => {
            say("warning: a key on the command line stays in the shell's history, and `cargo run` prints the line. Next time run `asset-forge cloud key` and paste the key when it asks");
            key.trim().to_owned()
        }
        None => {
            eprint!("Paste the Runpod API key and press Enter: ");
            let _ = std::io::stderr().flush();
            let mut line = String::new();
            std::io::stdin()
                .read_line(&mut line)
                .map_err(|e| tool_err("cloud", format!("no key on standard input: {e}")))?;
            line.trim().to_owned()
        }
    };
    if key.is_empty() {
        return Err(ForgeError::Usage(
            "asset-forge cloud key   (then paste the key when it asks)".to_owned(),
        ));
    }
    let balance = Runpod::new(&key).balance()?;
    let path = home.key_file();
    std::fs::create_dir_all(&home.dir).map_err(|e| io_err(&home.dir, e))?;
    std::fs::write(&path, format!("{key}\n")).map_err(|e| io_err(&path, e))?;
    println!(
        "key saved to {}: the account has ${:.2} of credit ({})",
        path.display(),
        balance.balance_amount,
        balance.billing_mode
    );
    if std::env::var_os("RUNPOD_API_KEY").is_some() {
        println!("note: RUNPOD_API_KEY is set in this environment and takes precedence over the file");
    }
    Ok(())
}

/// One-time set-up: picks the GPU types and the data centre, creates the network volume and the
/// template, installs everything on a first pod, checks it and deletes the pod. It is safe to
/// run again: what exists is kept. `--dry-run` prints the choices and creates nothing.
fn setup(paths: &Paths, home: &Home, args: &[String]) -> Result<()> {
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let dry_run = has("--dry-run");
    let no_volume = has("--no-volume");
    let api = Runpod::new(&home.api_key()?);
    let balance = api.balance()?;
    println!(
        "account: ${:.2} of credit, billing mode {}",
        balance.balance_amount, balance.billing_mode
    );

    let mut config = home.config().unwrap_or_default();
    if let Some(cap) = number::<f64>(args, "--max-usd-per-hour")? {
        config.max_usd_per_hour = cap;
    }
    if let Some(minutes) = number::<u32>(args, "--idle-minutes")? {
        config.idle_minutes = minutes;
    }
    if let Some(hours) = number::<u32>(args, "--max-hours")? {
        config.max_hours = hours;
    }
    if let Some(gb) = number::<u32>(args, "--volume-gb")? {
        config.volume_gb = gb;
    }
    if let Some(image) = flag_value(args, "--image") {
        image.clone_into(&mut config.image);
    }
    let mut regions: Vec<String> = args
        .windows(2)
        .filter(|w| w[0] == "--region")
        .map(|w| w[1].to_ascii_uppercase())
        .collect();
    if regions.is_empty() {
        regions = vec!["EUROPE".to_owned(), "NORTH_AMERICA".to_owned()];
    }

    let wishes: Vec<String> = GPU_WISHES.iter().map(|w| (*w).to_owned()).collect();
    let gpus = pick_gpus(
        &api.gpus(&config.cloud)?,
        &wishes,
        &config.cloud,
        config.max_usd_per_hour,
    );
    if gpus.is_empty() {
        return Err(tool_err(
            "cloud",
            format!(
                "Runpod lists no {MIN_VRAM_GB} GB GPU at or under ${:.2} an hour on the {} cloud",
                config.max_usd_per_hour, config.cloud
            ),
        ));
    }

    // A volume that exists decides the data centre. Otherwise the stock does.
    let volumes = api.volumes()?;
    let existing = volumes
        .iter()
        .find(|v| config.volume_id.as_deref() == Some(v.id.as_str()))
        .or_else(|| volumes.iter().find(|v| v.name == pod::VOLUME_NAME))
        .filter(|_| !no_volume);
    let create_volume = existing.is_none() && !no_volume;
    match existing {
        Some(volume) => {
            config.volume_id = Some(volume.id.clone());
            config.volume_gb = volume.size;
            config.data_center.clone_from(&volume.data_center);
            println!(
                "volume: {} ({} GB in {}), already there",
                volume.id, volume.size, volume.data_center
            );
        }
        None => {
            config.volume_id = None;
            config.data_center = match flag_value(args, "--data-center") {
                Some(id) => id.to_owned(),
                None => pick_data_center(&gpus, &api.data_centers()?, &regions, !no_volume)
                    .ok_or_else(|| tool_err("cloud", "no data centre has any of the GPUs in stock right now"))?,
            };
        }
    }
    config.gpus = gpus
        .iter()
        .filter(|g| g.data_centers.iter().any(|d| d.id == config.data_center))
        .map(|g| g.id.clone())
        .collect();
    println!("data centre: {}", config.data_center);
    for gpu in &gpus {
        println!(
            "gpu: {} ({} GB) ${:.2} an hour, stock in {}: {}",
            gpu.id,
            gpu.memory,
            price(gpu, &config.cloud),
            config.data_center,
            gpu.data_centers
                .iter()
                .find(|d| d.id == config.data_center)
                .map_or("not offered", |d| d.availability.as_str())
        );
    }
    if config.gpus.is_empty() {
        return Err(tool_err(
            "cloud",
            format!("{} offers none of the GPUs", config.data_center),
        ));
    }
    let monthly = f64::from(config.volume_gb) * VOLUME_USD_PER_GB_MONTH;
    if create_volume {
        println!(
            "volume: a new {} GB network volume in {}, about ${monthly:.2} a month until it is deleted",
            config.volume_gb, config.data_center
        );
    } else if no_volume {
        println!(
            "volume: none; every pod installs onto a fresh {} GB disk",
            config.volume_gb
        );
    }
    if dry_run {
        println!("dry run: nothing was created");
        return Ok(());
    }

    // From here on things cost money, and the set-up ends by starting a pod, so it needs the
    // lane like any other job, and the budget must not be used up.
    Lock::require(&paths.tools, Lane::Cloud)?;
    let live = settle(&api, home, &config)?;
    let spent_now = spent(&api, home, live.as_ref().map(|(state, _)| state), clock::now());
    hold_to_budget(
        &api,
        home,
        &config,
        &spent_now,
        live.as_ref().map(|(state, _)| state),
        UserLock::others(home, None),
    )?;
    ssh::ensure_key(&home.ssh_key())?;
    if create_volume {
        let volume = api.create_volume(pod::VOLUME_NAME, config.volume_gb, &config.data_center)?;
        println!("volume: created {}", volume.id);
        config.volume_id = Some(volume.id);
    }
    home.save_config(&config)?;
    println!("settings: {}", home.config_file().display());

    let result = doctor(paths);
    close();
    if !has("--keep") {
        down(home)?;
    }
    result
}

fn down(home: &Home) -> Result<()> {
    close();
    let config = home.config().unwrap_or_default();
    let api = Runpod::new(&home.api_key()?);
    down_with(&api, home, &config, UserLock::others(home, None))
}

fn print_retired(session: &spend::Session) {
    println!(
        "cloud: deleted pod {} after {} (about ${:.3} at ${:.2} an hour)",
        session.pod_id,
        clock::span(session.seconds),
        session.usd,
        session.usd_per_hour
    );
}

/// Deletes the pod on record and every other pod of ours that is up, and books each one. A pod
/// on record that has already ended is booked up to its estimated end, not up to now. While
/// `others` runs still use the pod, nothing is deleted: the last run's idle clock ends it.
fn down_with(api: &dyn Api, home: &Home, config: &Config, others: usize) -> Result<()> {
    if others > 0 {
        println!(
            "cloud: {others} other run(s) still use the pod, so it stays up; it deletes itself {} minutes after the last run ends",
            config.idle_minutes
        );
        return Ok(());
    }
    let mut deleted = 0;
    let mut skip = None;
    if let Some((state, _)) = settle(api, home, config)? {
        print_retired(&retire(api, home, &state, "cloud down")?);
        deleted += 1;
        skip = Some(state.pod_id);
    }
    let listed = api.pods()?;
    for id in listed
        .iter()
        .filter(|p| p.name == pod::POD_NAME && Some(&p.id) != skip.as_ref())
        .map(|p| p.id.as_str())
    {
        let Some(pod) = api.pod(id)? else { continue };
        if !pod.is_alive() {
            api.terminate(&pod.id)?;
            continue;
        }
        let started = clock::parse_rfc3339(&pod.created_at).unwrap_or_else(clock::now);
        let state = state_for(
            &pod,
            &no_record(),
            started,
            life_of(&pod, config),
            rate_of(&pod, config),
        );
        print_retired(&retire(
            api,
            home,
            &state,
            "cloud down (a pod with no record on this PC)",
        )?);
        deleted += 1;
    }
    if deleted == 0 {
        println!("cloud: no pod was up");
    }
    Ok(())
}

fn status(home: &Home) -> Result<()> {
    let config = home.config()?;
    let api = Runpod::new(&home.api_key()?);
    status_with(&api, home, &config)
}

fn status_with(api: &dyn Api, home: &Home, config: &Config) -> Result<()> {
    let balance = api.balance()?;
    println!("balance: ${:.2} ({})", balance.balance_amount, balance.billing_mode);
    println!(
        "limits: ${:.2} an hour at most; deleted after {} idle minutes or {} hours",
        config.max_usd_per_hour, config.idle_minutes, config.max_hours
    );
    let live = settle(api, home, config)?;
    match &live {
        None => println!("pod: none up"),
        Some((state, pod)) => {
            let age = clock::now().saturating_sub(state.started);
            println!(
                "pod: {} {} on {} in {}, up {}, about ${:.3} so far at ${:.2} an hour",
                pod.id,
                pod.status,
                state.gpu,
                state.data_center,
                clock::span(age),
                spend::cost(state.usd_per_hour, age),
                state.usd_per_hour
            );
            println!("     started for: {} / {}", state.thread, state.job);
            println!("     runs using it from this PC: {}", UserLock::others(home, None));
        }
    }
    let ours = live.as_ref().map(|(state, _)| state.pod_id.as_str());
    let strays = api
        .pods()?
        .into_iter()
        .filter(|p| p.is_alive() && Some(p.id.as_str()) != ours)
        .count();
    if strays > 0 {
        println!("warning: {strays} other pod(s) are running on this account");
    }
    print_budget(
        config,
        &spent(api, home, live.as_ref().map(|(state, _)| state), clock::now()),
    );
    Ok(())
}

fn spend_report(home: &Home) -> Result<()> {
    let api = Runpod::new(&home.api_key()?);
    let config = home.config().ok();
    // A pod on record that has ended is booked first, so the log below is complete.
    let live = match &config {
        Some(config) => settle(&api, home, config)?,
        None => None,
    };
    let (sessions, damaged) = spend::load_checked(&home.spend_log());
    let (usd, seconds) = spend::total(&sessions);
    println!(
        "spend log: {} session(s), {}, ${usd:.3}",
        sessions.len(),
        clock::span(seconds)
    );
    if damaged > 0 {
        println!("warning: {damaged} line(s) of the log are not a session");
    }
    let mut threads: Vec<&str> = sessions.iter().map(|s| s.thread.as_str()).collect();
    threads.sort_unstable();
    threads.dedup();
    for thread in threads {
        let own: Vec<spend::Session> = sessions.iter().filter(|s| s.thread == thread).cloned().collect();
        let (usd, seconds) = spend::total(&own);
        println!(
            "  {thread}: {} session(s), {}, ${usd:.3}",
            own.len(),
            clock::span(seconds)
        );
    }
    let window = Window::Months(BUDGET_MONTHS);
    println!(
        "Runpod's own billing (it runs hours behind): pods ${:.3}, volume storage ${:.3}",
        api.pod_billing(window, None)?,
        api.volume_billing(window)?
    );
    if let Some(config) = &config {
        print_budget(
            config,
            &spent(&api, home, live.as_ref().map(|(state, _)| state), clock::now()),
        );
    }
    let balance = api.balance()?;
    println!("balance: ${:.2} ({})", balance.balance_amount, balance.billing_mode);
    mirror_spend(home);
    println!("log: {}", home.spend_log().display());
    Ok(())
}

/// One line of the pod's doctor report: the value after `key=`.
fn field<'a>(report: &'a str, key: &str) -> &'a str {
    report
        .lines()
        .find_map(|l| l.strip_prefix(key).and_then(|rest| rest.strip_prefix('=')))
        .unwrap_or_default()
        .trim()
}

/// A pod's doctor report turned into checks: the label, whether it passed, and the detail.
#[must_use]
pub fn doctor_checks(report: &str) -> Vec<(String, bool, String)> {
    let mut checks = Vec::new();
    let gpu = field(report, "gpu");
    let vram_mib = gpu
        .split(',')
        .nth(1)
        .and_then(|v| v.trim().split(' ').next())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    checks.push(("gpu".to_owned(), vram_mib >= 23_000, gpu.to_owned()));
    let blender = field(report, "blender");
    checks.push((
        "blender".to_owned(),
        blender.starts_with(&format!("Blender {}", pod::BLENDER_VERSION)),
        blender.to_owned(),
    ));
    let optix = field(report, "cycles_optix");
    checks.push(("cycles optix".to_owned(), !optix.is_empty(), optix.to_owned()));
    let cuda = field(report, "cycles_cuda");
    checks.push(("cycles cuda".to_owned(), !cuda.is_empty(), cuda.to_owned()));
    let stats: serde_json::Value = serde_json::from_str(field(report, "comfyui")).unwrap_or_default();
    let version = stats["system"]["comfyui_version"].as_str().unwrap_or_default();
    let device = stats["devices"][0]["name"].as_str().unwrap_or("?");
    checks.push((
        "comfyui".to_owned(),
        format!("v{version}") == pod::COMFY_TAG,
        format!("{version} on {device}"),
    ));
    let torch = field(report, "torch");
    let want = pod::TORCH
        .split(' ')
        .next()
        .and_then(|t| t.strip_prefix("torch=="))
        .unwrap_or_default();
    checks.push((
        "torch".to_owned(),
        !want.is_empty() && torch.starts_with(want),
        torch.to_owned(),
    ));
    for model in pod::MODELS {
        let line = format!("{} {}/{}", model.bytes, model.folder, model.file);
        let found = report.lines().any(|l| l.strip_prefix("model=") == Some(line.as_str()));
        checks.push((
            model.file.to_owned(),
            found,
            if found {
                format!("{} bytes", model.bytes)
            } else {
                "missing or the wrong size".to_owned()
            },
        ));
    }
    let installed = installed_packages(report);
    let mut wrong: Vec<String> = Vec::new();
    let mut pinned = 0;
    for (name, version) in pod::pins() {
        pinned += 1;
        match installed.get(name) {
            Some(found) if found == version => {}
            Some(found) => wrong.push(format!("{name} is {found}, pinned {version}")),
            None => wrong.push(format!("{name} is missing")),
        }
    }
    let detail = if installed.is_empty() {
        "the pod gave no package list".to_owned()
    } else if wrong.is_empty() {
        format!("{pinned} pinned, all at their version")
    } else {
        format!("{} of {pinned} differ: {}", wrong.len(), wrong.join("; "))
    };
    checks.push((
        "python packages".to_owned(),
        !installed.is_empty() && wrong.is_empty(),
        detail,
    ));
    let stamp = field(report, "stamp");
    checks.push(("pins".to_owned(), stamp == pod::stamp(), stamp.to_owned()));
    checks
}

/// The pod's Python packages from its doctor report, as name to version. Names are made
/// lowercase with hyphens, the way the pin lists spell them.
fn installed_packages(report: &str) -> std::collections::BTreeMap<String, String> {
    field(report, "packages")
        .split_whitespace()
        .filter_map(|entry| entry.split_once("=="))
        .map(|(name, version)| (name.to_ascii_lowercase().replace('_', "-"), version.to_owned()))
        .collect()
}

/// Packages on the pod that no pin covers, apart from the three the torch line installs.
/// They are reported, not failed: they cannot change what a pinned package does.
#[must_use]
pub fn unpinned_packages(report: &str) -> Vec<String> {
    let pinned: std::collections::BTreeSet<&str> = pod::pins().map(|(name, _)| name).collect();
    installed_packages(report)
        .into_iter()
        .filter(|(name, _)| !pinned.contains(name.as_str()))
        .filter(|(name, _)| !matches!(name.as_str(), "torch" | "torchvision" | "torchaudio"))
        .map(|(name, version)| format!("{name}=={version}"))
        .collect()
}

/// How full the pod's disk is, from the megabytes its doctor report counted. `None` when the
/// report has no count.
#[must_use]
pub fn disk_line(report: &str, volume_gb: u32) -> Option<String> {
    let used_mb: f64 = field(report, "disk_mb").parse().ok()?;
    let used_gb = used_mb / 1024.0;
    let free_gb = f64::from(volume_gb) - used_gb;
    let note = if free_gb < 3.0 {
        " (low: a 4K render set can need 1 to 2 GB)"
    } else {
        ""
    };
    Some(format!(
        "{used_gb:.1} of {volume_gb} GB used, {free_gb:.1} GB free{note}"
    ))
}

/// Checks the pod the way `doctor` checks this PC, and fails when a check does.
fn doctor(paths: &Paths) -> Result<()> {
    let report = with_session(paths, |session| session.ssh.check(&pod::doctor_script()))?;
    let checks = doctor_checks(&report);
    for (label, pass, detail) in &checks {
        println!("{} {label}: {detail}", if *pass { "✓" } else { "✗" });
    }
    let volume_gb = Home::of(paths).config().map_or(0, |c| c.volume_gb);
    match disk_line(&report, volume_gb) {
        Some(line) => println!("· disk: {line}"),
        None => println!("· disk: the pod gave no size"),
    }
    let extra = unpinned_packages(&report);
    if !extra.is_empty() {
        println!("· packages without a pin: {}", extra.join(" "));
    }
    let stops = field(&report, "stops");
    if !stops.is_empty() {
        println!("· last watchdog deletions: {stops}");
    }
    let services = field(&report, "services");
    if !services.is_empty() {
        println!("· last start of the services by a pod: {services}");
    }
    // This run's own forward is one; more means a forward of an earlier run is still marking
    // the pod as in use.
    let tunnels = field(&report, "tunnels");
    if !tunnels.is_empty() {
        println!("· forwards keeping the pod in use: {tunnels}");
    }
    if checks.iter().all(|(_, pass, _)| *pass) {
        println!("cloud doctor: all checks passed");
        Ok(())
    } else {
        Err(tool_err("cloud doctor", "one or more checks failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runpod::{DcStock, Price};

    fn gpu(id: &str, memory: u32, secure_price: f64, stock: &[(&str, &str)]) -> GpuType {
        GpuType {
            id: id.to_owned(),
            name: id.trim_start_matches("NVIDIA ").to_owned(),
            memory,
            secure: true,
            price: Price {
                secure: secure_price,
                community: secure_price / 2.0,
            },
            data_centers: stock
                .iter()
                .map(|(dc, level)| DcStock {
                    id: (*dc).to_owned(),
                    availability: (*level).to_owned(),
                })
                .collect(),
        }
    }

    fn dc(id: &str, region: &str, volumes: bool) -> DataCenter {
        DataCenter {
            id: id.to_owned(),
            name: id.to_owned(),
            region: region.to_owned(),
            network_volume_types: if volumes {
                vec!["STANDARD".to_owned()]
            } else {
                Vec::new()
            },
        }
    }

    fn wishes() -> Vec<String> {
        GPU_WISHES.iter().map(|w| (*w).to_owned()).collect()
    }

    fn catalog() -> Vec<GpuType> {
        vec![
            gpu(
                "NVIDIA RTX A5000",
                24,
                0.27,
                &[("EU-RO-1", "HIGH"), ("US-TX-3", "HIGH")],
            ),
            gpu("NVIDIA GeForce RTX 5090", 32, 1.39, &[("EU-RO-1", "HIGH")]),
            gpu(
                "NVIDIA GeForce RTX 4090",
                24,
                0.89,
                &[("EU-RO-1", "MEDIUM"), ("US-TX-3", "HIGH"), ("EU-NO-VOL", "HIGH")],
            ),
            gpu("NVIDIA GeForce RTX 4080", 16, 0.50, &[("EU-RO-1", "HIGH")]),
            gpu("NVIDIA GeForce RTX 3090", 24, 0.50, &[("US-TX-3", "LOW")]),
        ]
    }

    #[test]
    fn gpus_come_in_wish_order_with_24_gb_under_the_cap() {
        let picked = pick_gpus(&catalog(), &wishes(), "SECURE", 0.90);
        let ids: Vec<&str> = picked.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(
            ids,
            ["NVIDIA GeForce RTX 4090", "NVIDIA GeForce RTX 3090", "NVIDIA RTX A5000"]
        );
    }

    #[test]
    fn a_lower_cap_drops_the_dearer_gpus() {
        let picked = pick_gpus(&catalog(), &wishes(), "SECURE", 0.30);
        assert_eq!(picked.len(), 1);
        assert_eq!(picked[0].id, "NVIDIA RTX A5000");
        assert!(pick_gpus(&catalog(), &wishes(), "SECURE", 0.10).is_empty());
    }

    #[test]
    fn the_data_centre_needs_volumes_and_prefers_the_first_region() {
        let gpus = pick_gpus(&catalog(), &wishes(), "SECURE", 0.90);
        let centers = [
            dc("US-TX-3", "NORTH_AMERICA", true),
            dc("EU-RO-1", "EUROPE", true),
            dc("EU-NO-VOL", "EUROPE", false),
        ];
        let regions = vec!["EUROPE".to_owned(), "NORTH_AMERICA".to_owned()];
        // Medium stock in the preferred region beats high stock elsewhere.
        assert_eq!(
            pick_data_center(&gpus, &centers, &regions, true).as_deref(),
            Some("EU-RO-1")
        );
        // Without the volume requirement the high-stock European site wins.
        assert_eq!(
            pick_data_center(&gpus, &centers, &regions, false).as_deref(),
            Some("EU-NO-VOL")
        );
        // The other region first.
        let us_first = vec!["NORTH_AMERICA".to_owned()];
        assert_eq!(
            pick_data_center(&gpus, &centers, &us_first, true).as_deref(),
            Some("US-TX-3")
        );
    }

    #[test]
    fn a_data_centre_without_any_of_the_gpus_is_never_picked() {
        let gpus = pick_gpus(&catalog(), &wishes(), "SECURE", 0.90);
        let centers = [dc("AP-JP-1", "ASIA", true)];
        assert_eq!(pick_data_center(&gpus, &centers, &["EUROPE".to_owned()], true), None);
        assert_eq!(pick_data_center(&[], &centers, &[], true), None);
    }

    #[test]
    fn low_first_choice_stock_loses_to_usable_stock_in_another_region() {
        let gpus = vec![gpu(
            "NVIDIA GeForce RTX 4090",
            24,
            0.89,
            &[("EU-RO-1", "LOW"), ("US-TX-3", "HIGH")],
        )];
        let centers = [dc("EU-RO-1", "EUROPE", true), dc("US-TX-3", "NORTH_AMERICA", true)];
        let regions = vec!["EUROPE".to_owned(), "NORTH_AMERICA".to_owned()];
        assert_eq!(
            pick_data_center(&gpus, &centers, &regions, true).as_deref(),
            Some("US-TX-3")
        );
    }

    fn state() -> State {
        State {
            pod_id: "p1".into(),
            gpu: "NVIDIA GeForce RTX 4090".into(),
            data_center: "EU-RO-1".into(),
            usd_per_hour: 0.89,
            started: 1_000_000,
            touched: 1_000_900,
            thread: "t".into(),
            job: "j".into(),
            max_secs: 0,
        }
    }

    #[test]
    fn the_budget_warns_at_its_warning_line_and_closes_at_its_end() {
        let config = Config::default();
        assert_eq!(config.budget(0.81), Budget::Fine);
        assert_eq!(config.budget(39.99), Budget::Fine);
        assert_eq!(config.budget(40.0), Budget::Warn);
        assert_eq!(config.budget(49.99), Budget::Warn);
        assert_eq!(config.budget(50.0), Budget::Spent);
        assert_eq!(config.budget(63.0), Budget::Spent);
        // A figure that is not a number never opens the lane.
        assert_eq!(config.budget(f64::NAN), Budget::Spent);
        let broken = Config {
            budget_usd: f64::NAN,
            ..Config::default()
        };
        assert_eq!(broken.budget(1.0), Budget::Spent);
    }

    #[test]
    fn a_new_pod_lives_no_longer_than_the_rest_of_the_budget_pays_for() {
        let config = Config::default();
        // Plenty left: the usual four hours.
        assert_eq!(config.pod_life_secs(0.81), Some(4 * 3600));
        // $2 left at the $0.90 price cap: 8000 seconds.
        assert_eq!(config.pod_life_secs(48.0), Some(8000));
        // $0.20 left: 800 seconds, still worth a pod.
        assert_eq!(config.pod_life_secs(49.80), Some(800));
        // $0.10 left: under ten minutes, so no pod.
        assert_eq!(config.pod_life_secs(49.90), None);
        assert_eq!(config.pod_life_secs(50.0), None);
        assert_eq!(config.pod_life_secs(f64::NAN), None);
        // A pod can never outspend what is left, whatever GPU it gets.
        for spent in [0.0, 12.5, 46.3, 49.2, 49.8] {
            let secs = config.pod_life_secs(spent).unwrap();
            let worst = spend::cost(config.max_usd_per_hour, secs);
            assert!(spent + worst <= config.budget_usd + 0.001, "{spent} + {worst}");
        }
        // A price cap of nothing cannot be divided by.
        let free = Config {
            max_usd_per_hour: 0.0,
            ..Config::default()
        };
        assert_eq!(free.pod_life_secs(1.0), None);
    }

    #[test]
    fn the_spend_is_this_pcs_count_or_runpods_whichever_is_higher_plus_storage() {
        // Runpod's billing is behind: the log and the running pod count.
        let early = Spent::from_parts(0.814, 0.05, 0.0, 0.0);
        assert!((early.total() - 0.864).abs() < 1e-9);
        // Runpod has billed more than the log holds (a pod started outside asset-forge).
        let more = Spent::from_parts(3.0, 0.0, 4.5, 0.21);
        assert!((more.pods - 4.5).abs() < 1e-9);
        assert!((more.total() - 4.71).abs() < 1e-9);
    }

    #[test]
    fn a_pod_keeps_its_own_lifetime_cap_and_old_records_fall_back_to_the_settings() {
        let config = Config::default();
        assert_eq!(state().cap(&config), 4 * 3600);
        let short = State {
            max_secs: 1200,
            ..state()
        };
        assert_eq!(short.cap(&config), 1200);
        // A record from before the field existed still reads.
        let old: State = serde_json::from_str(
            r#"{"pod_id":"p0","gpu":"g","data_center":"EU-RO-1","usd_per_hour":0.89,"started":1,"touched":2,"thread":"t","job":"j"}"#,
        )
        .unwrap();
        assert_eq!(old.max_secs, 0);
        // The watchdog's estimate ends at the pod's own cap.
        let mut busy = short;
        busy.touched = busy.started + 1100;
        assert_eq!(watchdog_end(&busy, &config, 2_000_000), 1_000_000 + 1200 + 30);
    }

    #[test]
    fn a_pod_is_replaced_only_when_little_of_its_life_is_left() {
        // The usual cap of four hours: half an hour of margin.
        assert!(worth_keeping(12_000, 14_400));
        assert!(!worth_keeping(12_700, 14_400));
        // A cap the budget cut to 20 minutes: a quarter of it, so the pod is not replaced at once.
        assert!(worth_keeping(0, 1200));
        assert!(worth_keeping(800, 1200));
        assert!(!worth_keeping(950, 1200));
    }

    #[test]
    fn a_watchdog_deletion_is_booked_at_last_use_plus_the_idle_time() {
        let config = Config::default();
        // Idle: last use at +900 s, 10 idle minutes, one look of the watchdog.
        assert_eq!(watchdog_end(&state(), &config, 2_000_000), 1_000_900 + 600 + 30);
        // Never later than now.
        assert_eq!(watchdog_end(&state(), &config, 1_001_000), 1_001_000);
        // Never later than the lifetime cap.
        let mut busy = state();
        busy.touched = 1_000_000 + 4 * 3600;
        assert_eq!(watchdog_end(&busy, &config, 2_000_000), 1_000_000 + 4 * 3600 + 30);
    }

    #[test]
    fn settings_survive_a_round_trip_and_fill_in_missing_fields() {
        let config = Config {
            data_center: "EU-RO-1".into(),
            volume_id: Some("vol1".into()),
            gpus: vec!["NVIDIA GeForce RTX 4090".into()],
            ..Config::default()
        };
        let text = serde_json::to_string(&config).unwrap();
        assert_eq!(serde_json::from_str::<Config>(&text).unwrap(), config);
        let sparse: Config = serde_json::from_str(r#"{"data_center":"US-TX-3"}"#).unwrap();
        assert_eq!(sparse.data_center, "US-TX-3");
        assert_eq!(sparse.idle_minutes, 10);
        assert_eq!(sparse.tunnel_port, 8189);
        assert!((sparse.max_usd_per_hour - 0.90).abs() < 1e-9);
        // A settings file from before the budget gets the budget TJX set.
        assert!((sparse.budget_usd - 50.0).abs() < 1e-9);
        assert!((sparse.budget_warn_usd - 40.0).abs() < 1e-9);
    }

    #[test]
    fn the_limits_have_floors() {
        let config = Config {
            idle_minutes: 0,
            max_hours: 0,
            ..Config::default()
        };
        assert_eq!(config.idle_secs(), 120);
        assert_eq!(config.max_secs(), 3600);
    }

    #[test]
    fn the_api_key_is_the_first_line_of_the_key_file() {
        let home = test_home("key");
        assert!(home.api_key_from(None).is_err());
        std::fs::write(home.key_file(), "  rpa_test_key  \nnotes\n").unwrap();
        assert_eq!(home.api_key_from(None).ok().as_deref(), Some("rpa_test_key"));
        // The variable wins over the file; an empty one does not count.
        assert_eq!(
            home.api_key_from(Some("rpa_env".into())).ok().as_deref(),
            Some("rpa_env")
        );
        assert_eq!(
            home.api_key_from(Some("  ".into())).ok().as_deref(),
            Some("rpa_test_key")
        );
        let _ = std::fs::remove_dir_all(&home.dir);
    }

    #[test]
    fn state_is_saved_and_cleared() {
        let home = test_home("state");
        let dir = home.dir.clone();
        assert_eq!(home.state(), None);
        home.save_state(Some(&state())).unwrap();
        assert_eq!(home.state(), Some(state()));
        home.save_state(None).unwrap();
        assert_eq!(home.state(), None);
        assert!(home.config().is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn report(optix: &str) -> String {
        let models: Vec<String> = pod::MODELS
            .iter()
            .map(|m| format!("model={} {}/{}", m.bytes, m.folder, m.file))
            .collect();
        // The package list as `uv pip freeze` prints it: every pin, plus the torch line's three.
        let mut packages: Vec<String> = pod::pins().map(|(n, v)| format!("{n}=={v}")).collect();
        packages.push("torch==2.14.1+cu130".to_owned());
        packages.push("torchvision==0.29.1+cu130".to_owned());
        packages.push("torchaudio==2.11.0+cu130".to_owned());
        format!(
            "gpu=NVIDIA GeForce RTX 4090, 24564 MiB, 580.95.05\nblender=Blender 5.3.0\ncycles_optix={optix}\ncycles_cuda=NVIDIA GeForce RTX 4090\ncomfyui={{\"system\":{{\"comfyui_version\":\"0.46.0\"}},\"devices\":[{{\"name\":\"cuda:0 NVIDIA GeForce RTX 4090\"}}]}}\ntorch=2.14.1+cu130\npackages={}\n{}\nstamp={}\ndisk_mb=27033\nstops=\n",
            packages.join(" "),
            models.join("\n"),
            pod::stamp()
        )
    }

    #[test]
    fn a_complete_pod_passes_every_check() {
        let checks = doctor_checks(&report("NVIDIA GeForce RTX 4090"));
        let failed: Vec<&String> = checks.iter().filter(|c| !c.1).map(|c| &c.0).collect();
        assert!(failed.is_empty(), "{failed:?}");
        assert_eq!(checks.len(), 6 + pod::MODELS.len() + 2);
        assert!(unpinned_packages(&report("x")).is_empty());
    }

    #[test]
    fn a_package_at_another_version_or_missing_fails_the_package_check() {
        // The state of the first pod before the pins: kornia and transformers newer than this PC.
        let drifted = report("x")
            .replace("kornia==0.9.0", "kornia==0.9.2")
            .replace("transformers==5.21.0", "transformers==5.24.1")
            .replace(" triton==3.8.0", "");
        let checks = doctor_checks(&drifted);
        let packages = checks.iter().find(|c| c.0 == "python packages").unwrap();
        assert!(!packages.1);
        assert!(packages.2.starts_with("3 of 101 differ: "), "{}", packages.2);
        assert!(packages.2.contains("kornia is 0.9.2, pinned 0.9.0"));
        assert!(packages.2.contains("transformers is 5.24.1, pinned 5.21.0"));
        assert!(packages.2.contains("triton is missing"));
        // Nothing else fails because of it.
        assert_eq!(checks.iter().filter(|c| !c.1).count(), 1);
    }

    #[test]
    fn a_report_without_a_package_list_fails_rather_than_passes() {
        let none = report("x")
            .lines()
            .filter(|l| !l.starts_with("packages="))
            .collect::<Vec<_>>()
            .join("\n");
        let packages = doctor_checks(&none)
            .into_iter()
            .find(|c| c.0 == "python packages")
            .unwrap();
        assert!(!packages.1);
        assert_eq!(packages.2, "the pod gave no package list");
    }

    #[test]
    fn package_names_are_compared_in_one_spelling_and_extras_are_listed() {
        // uv prints hyphens; an underscore or capital must not read as a missing package.
        let odd = report("x").replace(" pydantic-core==", " Pydantic_Core==") + "packages_note=unused\n";
        assert!(doctor_checks(&odd).iter().all(|c| c.1));
        let extra = report("x").replace("packages=", "packages=left-over==1.0 ");
        assert_eq!(unpinned_packages(&extra), ["left-over==1.0"]);
        assert!(doctor_checks(&extra).iter().all(|c| c.1));
    }

    #[test]
    fn the_disk_line_counts_against_the_volume_and_warns_when_it_is_nearly_full() {
        assert_eq!(
            disk_line("disk_mb=27033\n", 30).as_deref(),
            Some("26.4 of 30 GB used, 3.6 GB free")
        );
        let tight = disk_line("disk_mb=28672\n", 30).unwrap();
        assert!(tight.starts_with("28.0 of 30 GB used, 2.0 GB free (low"), "{tight}");
        assert_eq!(disk_line("disk=x\n", 30), None);
    }

    #[test]
    fn a_pod_without_optix_or_a_model_fails_its_check() {
        let checks = doctor_checks(&report(""));
        let failed: Vec<&str> = checks.iter().filter(|c| !c.1).map(|c| c.0.as_str()).collect();
        assert_eq!(failed, ["cycles optix"]);
        let short = report("x").replace("model=6938078334", "model=1234");
        let failed: Vec<String> = doctor_checks(&short)
            .into_iter()
            .filter(|c| !c.1)
            .map(|c| c.0)
            .collect();
        assert_eq!(failed, ["sd_xl_base_1.0.safetensors"]);
        assert!(doctor_checks("").iter().all(|c| !c.1));
    }

    // The code that spends money, run against a provider that keeps its pods in memory.

    use std::cell::{Cell, RefCell};

    /// A provider that keeps its pods in memory, so the code that starts, deletes and books pods
    /// can be tested without spending anything.
    #[derive(Default)]
    struct Fake {
        pods: RefCell<Vec<Pod>>,
        created: RefCell<Vec<PodRequest>>,
        terminated: RefCell<Vec<String>>,
        /// The hourly price of a pod it creates.
        price: Cell<f64>,
        /// What it reports as billed for one pod, and for all pods together.
        billed_pod: Cell<f64>,
        billed_all: Cell<f64>,
        /// Every delete fails.
        refuse_terminate: Cell<bool>,
        /// Deleted pods still show, as running, in the list and when fetched (a lagging list).
        lagging: Cell<bool>,
        deleted: RefCell<Vec<Pod>>,
    }

    fn fake_pod(id: &str, status: &str, cost: f64, created: u64) -> Pod {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": pod::POD_NAME,
            "status": status,
            "cost": cost,
            "dataCenterId": "EU-RO-1",
            "gpu": { "id": "NVIDIA GeForce RTX 4090" },
            "createdAt": clock::rfc3339(created),
        }))
        .unwrap()
    }

    impl Fake {
        fn with(pods: Vec<Pod>) -> Self {
            let fake = Self::default();
            fake.price.set(0.89);
            *fake.pods.borrow_mut() = pods;
            fake
        }

        fn env_of(&self, at: usize, name: &str) -> Option<String> {
            self.created.borrow()[at]
                .env
                .iter()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.clone())
        }
    }

    impl Api for Fake {
        fn balance(&self) -> Result<runpod::Balance> {
            Ok(runpod::Balance {
                balance_amount: 9.0,
                billing_mode: "PREPAID".into(),
            })
        }

        fn template_id(&self, _name: &str) -> Result<Option<String>> {
            Ok(Some("tpl".into()))
        }

        fn put_template(&self, _existing: Option<&str>, _template: &serde_json::Value) -> Result<String> {
            Ok("tpl".into())
        }

        fn pods(&self) -> Result<Vec<Pod>> {
            let mut list = self.pods.borrow().clone();
            if self.lagging.get() {
                list.extend(self.deleted.borrow().iter().cloned());
            }
            Ok(list)
        }

        fn pod(&self, id: &str) -> Result<Option<Pod>> {
            let up = self.pods.borrow().iter().find(|p| p.id == id).cloned();
            let stale = || {
                self.lagging
                    .get()
                    .then(|| self.deleted.borrow().iter().find(|p| p.id == id).cloned())
                    .flatten()
            };
            Ok(up.or_else(stale))
        }

        fn create_pod(&self, request: &PodRequest) -> Result<Created> {
            let id = format!("new{}", self.created.borrow().len() + 1);
            self.created.borrow_mut().push(request.clone());
            let mut pod = fake_pod(&id, "RUNNING", self.price.get(), clock::now());
            let env: serde_json::Map<String, serde_json::Value> = request
                .env
                .iter()
                .map(|(key, value)| (key.clone(), serde_json::json!(value)))
                .collect();
            pod.env = serde_json::Value::Object(env);
            self.pods.borrow_mut().push(pod.clone());
            Ok(Created::Pod(Box::new(pod)))
        }

        fn terminate(&self, id: &str) -> Result<()> {
            if self.refuse_terminate.get() {
                return Err(tool_err("fake", "the delete was refused"));
            }
            let mut pods = self.pods.borrow_mut();
            if let Some(at) = pods.iter().position(|p| p.id == id) {
                self.deleted.borrow_mut().push(pods.remove(at));
            }
            self.terminated.borrow_mut().push(id.to_owned());
            Ok(())
        }

        fn pod_billing(&self, _window: Window, pod: Option<&str>) -> Result<f64> {
            Ok(if pod.is_some() {
                self.billed_pod.get()
            } else {
                self.billed_all.get()
            })
        }

        fn volume_billing(&self, _window: Window) -> Result<f64> {
            Ok(0.0)
        }
    }

    /// A lane folder of its own for one test, with an SSH public key in it.
    fn test_home(name: &str) -> Home {
        let dir = std::env::temp_dir().join(format!("forge_cloud_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("id_ed25519.pub"), "ssh-ed25519 AAAA test\n").unwrap();
        Home { dir }
    }

    fn test_config() -> Config {
        Config {
            data_center: "EU-RO-1".into(),
            gpus: vec!["NVIDIA GeForce RTX 4090".into()],
            ..Config::default()
        }
    }

    fn test_lock() -> Lock {
        Lock {
            thread: "test thread".into(),
            started: String::new(),
            job: "test job".into(),
        }
    }

    fn record(id: &str, started: u64, touched: u64) -> State {
        State {
            pod_id: id.into(),
            gpu: "NVIDIA GeForce RTX 4090".into(),
            data_center: "EU-RO-1".into(),
            usd_per_hour: 0.89,
            started,
            touched,
            thread: "t".into(),
            job: "j".into(),
            max_secs: 4 * 3600,
        }
    }

    /// Puts an earlier session of `usd` in the log.
    fn booked(home: &Home, id: &str, usd: f64) {
        let mut session = spend::Session::close(&record(id, 1_000_000, 1_000_000), 1_003_600, "earlier");
        session.usd = usd;
        spend::append(&home.spend_log(), &session).unwrap();
    }

    fn logged(home: &Home) -> Vec<spend::Session> {
        spend::load(&home.spend_log())
    }

    fn cleanup(home: &Home) {
        let _ = std::fs::remove_dir_all(&home.dir);
    }

    #[test]
    fn a_pod_that_ended_long_ago_is_booked_to_its_estimated_end_not_to_now() {
        // The review's case: the watchdog deleted the pod, and nothing ran for three days.
        let home = test_home("ended_long_ago");
        let fake = Fake::with(Vec::new());
        let now = clock::now();
        let started = now - 3 * 24 * 3600;
        home.save_state(Some(&record("gone1", started, started + 600))).unwrap();
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        let log = logged(&home);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].pod_id, "gone1");
        assert_eq!(log[0].seconds, 600 + 600 + 30, "last use, ten idle minutes, one look");
        assert!(log[0].usd < 0.31, "{}", log[0].usd);
        // The lane stays open, and the new pod gets the whole lifetime.
        assert_eq!(state.pod_id, "new1");
        assert_eq!(fake.env_of(0, "FORGE_MAX_SECS").as_deref(), Some("14400"));
        cleanup(&home);
    }

    #[test]
    fn a_pod_that_is_up_is_counted_up_to_now_and_kept() {
        let home = test_home("up");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("up1", "RUNNING", 0.89, now - 3600)]);
        home.save_state(Some(&record("up1", now - 3600, now - 60))).unwrap();
        let live = settle(&fake, &home, &test_config()).unwrap();
        let counted = spent(&fake, &home, live.as_ref().map(|(s, _)| s), now);
        assert!((counted.total() - 0.89).abs() < 0.001, "{}", counted.total());
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        assert_eq!(state.pod_id, "up1");
        assert!(fake.created.borrow().is_empty());
        assert!(logged(&home).is_empty());
        cleanup(&home);
    }

    #[test]
    fn at_the_budget_the_pod_that_is_up_is_deleted_and_none_starts() {
        let home = test_home("budget_used");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("up2", "RUNNING", 0.89, now - 3600)]);
        booked(&home, "old", 49.5);
        home.save_state(Some(&record("up2", now - 3600, now - 60))).unwrap();
        let err = prepare(&fake, &home, &test_config(), &test_lock(), now, 0)
            .unwrap_err()
            .to_string();
        assert!(err.contains("the cloud lane is closed"), "{err}");
        assert!(err.contains("run the same command without `--lane cloud`"), "{err}");
        assert_eq!(*fake.terminated.borrow(), ["up2"]);
        assert!(fake.created.borrow().is_empty());
        assert_eq!(home.state(), None);
        assert_eq!(logged(&home).len(), 2);
        cleanup(&home);
    }

    #[test]
    fn near_the_budget_a_new_pod_lives_only_as_long_as_the_rest_pays_for() {
        let home = test_home("budget_near");
        let fake = Fake::with(Vec::new());
        booked(&home, "old", 48.0);
        let now = clock::now();
        prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        // $2 left at the $0.90 price cap.
        assert_eq!(fake.env_of(0, "FORGE_MAX_SECS").as_deref(), Some("8000"));
        let deadline: u64 = fake.env_of(0, "FORGE_DEADLINE").unwrap().parse().unwrap();
        assert!(deadline >= now + 8000 && deadline <= clock::now() + 8000, "{deadline}");
        assert_eq!(home.state().unwrap().max_secs, 8000);
        cleanup(&home);
    }

    #[test]
    fn with_under_ten_minutes_of_budget_left_the_job_is_sent_to_the_local_lane() {
        let home = test_home("budget_short");
        let fake = Fake::with(Vec::new());
        // $0.10 left pays for 400 s at the $0.90 price cap, under the 10-minute minimum.
        booked(&home, "old", 49.9);
        let err = prepare(&fake, &home, &test_config(), &test_lock(), clock::now(), 0)
            .unwrap_err()
            .to_string();
        assert!(err.contains("too little of the budget is left to start a pod"), "{err}");
        assert!(err.contains("run the same command without `--lane cloud`"), "{err}");
        assert!(fake.created.borrow().is_empty());
        assert_eq!(home.state(), None);
        cleanup(&home);
    }

    #[test]
    fn a_new_pod_over_the_price_cap_is_deleted_at_once_and_booked() {
        let home = test_home("over_cap");
        let fake = Fake::with(Vec::new());
        fake.price.set(1.25);
        let err = prepare(&fake, &home, &test_config(), &test_lock(), clock::now(), 0)
            .unwrap_err()
            .to_string();
        assert!(err.contains("over the $0.90 limit"), "{err}");
        assert_eq!(*fake.terminated.borrow(), ["new1"]);
        assert_eq!(home.state(), None);
        let log = logged(&home);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].pod_id, "new1");
        cleanup(&home);
    }

    #[test]
    fn a_pod_without_a_price_is_booked_at_the_price_cap() {
        let home = test_home("no_price");
        let fake = Fake::with(Vec::new());
        fake.price.set(0.0);
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), clock::now(), 0).unwrap();
        assert!((state.usd_per_hour - 0.90).abs() < 1e-9);
        cleanup(&home);
    }

    #[test]
    fn one_unrecorded_pod_is_adopted_and_every_other_is_deleted_and_booked() {
        let home = test_home("strays");
        let now = clock::now();
        let fake = Fake::with(vec![
            fake_pod("s1", "RUNNING", 0.89, now - 600),
            fake_pod("s2", "RUNNING", 0.89, now - 300),
            fake_pod("s3", "EXITED", 0.89, now - 900),
        ]);
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        assert_eq!(state.pod_id, "s1");
        assert_eq!(home.state().unwrap().pod_id, "s1");
        assert_eq!(*fake.terminated.borrow(), ["s2", "s3"]);
        assert!(fake.created.borrow().is_empty());
        let log = logged(&home);
        assert_eq!(log.len(), 1, "the stopped pod costs no compute and is not booked");
        assert_eq!(log[0].pod_id, "s2");
        assert!(log[0].seconds >= 300);
        cleanup(&home);
    }

    #[test]
    fn an_unrecorded_pod_over_the_price_cap_is_deleted_not_adopted() {
        let home = test_home("stray_over_cap");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("s4", "RUNNING", 1.50, now - 300)]);
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        assert_eq!(state.pod_id, "new1");
        assert_eq!(*fake.terminated.borrow(), ["s4"]);
        cleanup(&home);
    }

    #[test]
    fn a_pod_replaced_near_its_cap_is_not_taken_back_from_a_lagging_list() {
        let home = test_home("replaced");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("old", "RUNNING", 0.89, now - 14_000)]);
        fake.lagging.set(true);
        home.save_state(Some(&record("old", now - 14_000, now - 100))).unwrap();
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 0).unwrap();
        assert_eq!(state.pod_id, "new1");
        assert_eq!(*fake.terminated.borrow(), ["old"]);
        let log = logged(&home);
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].pod_id, "old");
        cleanup(&home);
    }

    #[test]
    fn a_refused_delete_keeps_the_record_so_the_next_command_tries_again() {
        let home = test_home("refused");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("up3", "RUNNING", 0.89, now - 3600)]);
        fake.refuse_terminate.set(true);
        booked(&home, "old", 49.5);
        home.save_state(Some(&record("up3", now - 3600, now - 60))).unwrap();
        assert!(prepare(&fake, &home, &test_config(), &test_lock(), now, 0).is_err());
        assert_eq!(home.state().unwrap().pod_id, "up3");
        assert!(down_with(&fake, &home, &test_config(), 0).is_err());
        assert_eq!(home.state().unwrap().pod_id, "up3");
        assert_eq!(logged(&home).len(), 1, "nothing is booked for a pod that is still up");
        cleanup(&home);
    }

    #[test]
    fn cloud_down_books_an_ended_pod_to_its_estimate_and_deletes_every_other_pod() {
        let home = test_home("down");
        let now = clock::now();
        let started = now - 3 * 24 * 3600;
        let fake = Fake::with(vec![fake_pod("s5", "RUNNING", 0.89, now - 900)]);
        home.save_state(Some(&record("gone2", started, started + 600))).unwrap();
        down_with(&fake, &home, &test_config(), 0).unwrap();
        let log = logged(&home);
        assert_eq!(log.len(), 2);
        assert_eq!((log[0].pod_id.as_str(), log[0].seconds), ("gone2", 1230));
        assert_eq!(log[1].pod_id, "s5");
        assert!(log[1].seconds >= 900);
        assert_eq!(*fake.terminated.borrow(), ["s5"]);
        assert_eq!(home.state(), None);
        cleanup(&home);
    }

    #[test]
    fn a_new_pod_whose_record_cannot_be_saved_is_deleted() {
        let home = test_home("unsaved");
        // A folder where the record should go makes writing it fail.
        std::fs::create_dir_all(home.dir.join("state.json")).unwrap();
        let fake = Fake::with(Vec::new());
        assert!(prepare(&fake, &home, &test_config(), &test_lock(), clock::now(), 0).is_err());
        assert_eq!(*fake.terminated.borrow(), ["new1"]);
        cleanup(&home);
    }

    #[test]
    fn a_damaged_spend_log_stops_the_lane() {
        let home = test_home("damaged");
        std::fs::write(home.spend_log(), "not a session\n").unwrap();
        let fake = Fake::with(Vec::new());
        let err = prepare(&fake, &home, &test_config(), &test_lock(), clock::now(), 0)
            .unwrap_err()
            .to_string();
        assert!(err.contains("cannot be counted"), "{err}");
        assert!(fake.created.borrow().is_empty());
        cleanup(&home);
    }

    #[test]
    fn an_ended_pod_is_booked_at_runpods_bill_only_when_that_is_higher() {
        let now = clock::now();
        let started = now - 3 * 24 * 3600;
        for (billed, expected) in [(0.05, 0.304), (0.75, 0.75)] {
            let home = test_home(&format!("billed_{billed}"));
            let fake = Fake::with(Vec::new());
            fake.billed_pod.set(billed);
            home.save_state(Some(&record("gone3", started, started + 600))).unwrap();
            assert!(settle(&fake, &home, &test_config()).unwrap().is_none());
            let log = logged(&home);
            assert!((log[0].usd - expected).abs() < 1e-9, "{billed}: {}", log[0].usd);
            cleanup(&home);
        }
    }

    #[test]
    fn runpods_total_counts_when_it_is_above_the_log() {
        let home = test_home("billed_total");
        let fake = Fake::with(Vec::new());
        booked(&home, "old", 3.0);
        fake.billed_all.set(4.5);
        assert!((spent(&fake, &home, None, clock::now()).total() - 4.5).abs() < 1e-9);
        cleanup(&home);
    }

    #[cfg(windows)]
    #[test]
    fn only_one_run_at_a_time_starts_the_pod() {
        let home = test_home("start_lock");
        let first = StartLock::try_take(&home).unwrap();
        assert!(first.is_some());
        assert!(StartLock::try_take(&home).unwrap().is_none(), "a second run waits");
        drop(first);
        assert!(StartLock::try_take(&home).unwrap().is_some());
        cleanup(&home);
    }

    #[cfg(windows)]
    #[test]
    fn the_runs_that_use_the_pod_are_counted_and_ended_ones_cleared() {
        let home = test_home("users");
        let own = UserLock::take(&home).unwrap();
        assert_eq!(UserLock::others(&home, Some(&own)), 0);
        assert_eq!(UserLock::others(&home, None), 1);
        // Another run: a lock file held by a handle of its own.
        let other_path = UserLock::folder(&home).join("999999.lock");
        let other = open_exclusive(&other_path).unwrap();
        assert_eq!(UserLock::others(&home, Some(&own)), 1);
        // That run ends: its file is no longer held, and the next count removes it.
        drop(other);
        assert_eq!(UserLock::others(&home, Some(&own)), 0);
        assert!(!other_path.exists());
        // A run's own file goes when it ends.
        let own_path = own.path.clone();
        drop(own);
        assert!(!own_path.exists());
        cleanup(&home);
    }

    #[test]
    fn a_shared_pod_is_not_deleted_at_the_budget_while_another_run_uses_it() {
        let home = test_home("shared_budget");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("up4", "RUNNING", 0.89, now - 3600)]);
        booked(&home, "old", 49.5);
        home.save_state(Some(&record("up4", now - 3600, now - 60))).unwrap();
        let err = prepare(&fake, &home, &test_config(), &test_lock(), now, 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("the cloud lane is closed"), "{err}");
        assert!(fake.terminated.borrow().is_empty(), "the other run's pod stays up");
        assert_eq!(home.state().unwrap().pod_id, "up4");
        cleanup(&home);
    }

    #[test]
    fn a_shared_pod_near_its_cap_is_kept_and_one_about_to_end_is_refused() {
        let now = clock::now();
        // 400 s left: kept for the second run, not replaced under the first.
        let home = test_home("shared_near_cap");
        let fake = Fake::with(vec![fake_pod("old2", "RUNNING", 0.89, now - 14_000)]);
        home.save_state(Some(&record("old2", now - 14_000, now - 100))).unwrap();
        let (state, _) = prepare(&fake, &home, &test_config(), &test_lock(), now, 1).unwrap();
        assert_eq!(state.pod_id, "old2");
        assert!(fake.terminated.borrow().is_empty());
        assert!(fake.created.borrow().is_empty());
        cleanup(&home);
        // 100 s left: the job is refused instead.
        let home = test_home("shared_ending");
        let fake = Fake::with(vec![fake_pod("old3", "RUNNING", 0.89, now - 14_300)]);
        home.save_state(Some(&record("old3", now - 14_300, now - 100))).unwrap();
        let err = prepare(&fake, &home, &test_config(), &test_lock(), now, 1)
            .unwrap_err()
            .to_string();
        assert!(err.contains("another run is using it"), "{err}");
        assert!(fake.terminated.borrow().is_empty());
        cleanup(&home);
    }

    #[test]
    fn cloud_down_leaves_the_pod_up_while_another_run_uses_it() {
        let home = test_home("shared_down");
        let now = clock::now();
        let fake = Fake::with(vec![fake_pod("up5", "RUNNING", 0.89, now - 600)]);
        home.save_state(Some(&record("up5", now - 600, now - 60))).unwrap();
        down_with(&fake, &home, &test_config(), 1).unwrap();
        assert!(fake.terminated.borrow().is_empty());
        assert_eq!(home.state().unwrap().pod_id, "up5");
        assert!(logged(&home).is_empty());
        cleanup(&home);
    }

    #[test]
    fn the_budget_counts_gpu_time_and_shows_storage_apart_unless_told_otherwise() {
        let spent = Spent::from_parts(3.0, 0.0, 0.0, 5.0);
        let gpu_only = Config::default();
        assert!(!gpu_only.budget_counts_storage);
        assert!((spent.counted(&gpu_only) - 3.0).abs() < 1e-9);
        assert!(budget_line(&gpu_only, &spent).contains("$3.00 of the $50.00 budget is spent on GPU time"));
        assert!(budget_line(&gpu_only, &spent).contains("volume storage so far $5.00, shown, not counted"));
        let both = Config {
            budget_counts_storage: true,
            ..Config::default()
        };
        assert!((spent.counted(&both) - 8.0).abs() < 1e-9);
        assert!(budget_line(&both, &spent).starts_with("$8.00 of the $50.00 budget is spent"));
    }

    #[test]
    fn a_free_port_is_found_when_the_preferred_one_is_taken() {
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let busy = taken.local_addr().unwrap().port();
        let port = free_port(busy).unwrap();
        assert_ne!(port, busy);
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok());
    }
}
