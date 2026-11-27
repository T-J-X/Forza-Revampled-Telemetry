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
use runpod::{Created, DataCenter, GpuType, Pod, PodRequest, Runpod, Window};
use serde::{Deserialize, Serialize};
use ssh::{Ssh, Tunnel};
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
    /// The most the lane may cost in total, in USD. At this total no pod starts. Only TJX
    /// raises it.
    pub budget_usd: f64,
    /// From this total on, every cloud job says how much of the budget is spent.
    pub budget_warn_usd: f64,
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
}

impl Spent {
    /// Puts the figures together. `logged` and `running` are this PC's own count; `billed_pods`
    /// and `billed_storage` are Runpod's, which run hours behind.
    #[must_use]
    pub fn from_parts(logged: f64, running: f64, billed_pods: f64, billed_storage: f64) -> Self {
        Self {
            pods: (logged + running).max(billed_pods),
            storage: billed_storage,
        }
    }

    /// Pods and storage together.
    #[must_use]
    pub fn total(&self) -> f64 {
        self.pods + self.storage
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
        let from_env = std::env::var("RUNPOD_API_KEY").ok().filter(|k| !k.trim().is_empty());
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
        std::fs::write(path, text + "\n").map_err(|e| io_err(path, e))
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
    home: Home,
    _tunnel: Tunnel,
}

/// The session of this process. It is opened by the first job that needs the pod and kept
/// until [`close`], so the forward and the pod's "in use" mark last for the whole run.
static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Runs `job` with the pod, starting the pod first when it is not up.
pub fn with_session<T>(paths: &Paths, job: impl FnOnce(&Session) -> Result<T>) -> Result<T> {
    let mut slot = SESSION
        .lock()
        .map_err(|_| tool_err("cloud", "the session lock is poisoned"))?;
    if slot.is_none() {
        *slot = Some(open(paths)?);
    }
    match slot.as_ref() {
        Some(session) => job(session),
        None => Err(tool_err("cloud", "the session did not open")),
    }
}

/// The pod's ComfyUI address on this PC, starting the pod when needed.
pub fn comfy_url(paths: &Paths) -> Result<String> {
    with_session(paths, |session| Ok(session.comfy_url.clone()))
}

/// Ends this process's session: the forward closes and the pod's idle clock starts. The pod
/// itself stays up for the next command until its watchdog or `cloud down` deletes it.
pub fn close() {
    let Ok(mut slot) = SESSION.lock() else { return };
    if let Some(session) = slot.take() {
        if let Some(mut state) = session.home.state() {
            state.touched = clock::now();
            let _ = session.home.save_state(Some(&state));
        }
    }
}

fn say(text: &str) {
    println!("  cloud: {text}");
}

/// Deletes the pod and books its session.
fn retire(api: &Runpod, home: &Home, state: &State, how: &str) -> Result<spend::Session> {
    api.terminate(&state.pod_id)?;
    let session = spend::Session::close(state, clock::now(), how);
    book(home, state, &session)?;
    Ok(session)
}

/// Writes a finished session to the spend log and forgets the pod.
fn book(home: &Home, state: &State, session: &spend::Session) -> Result<()> {
    spend::append(&home.spend_log(), session)?;
    home.save_state(None)?;
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

/// Adds up what the lane has cost: the spend log, the pod that is up now, and Runpod's own
/// billing. Runpod's figures are left out when they cannot be read; the log does not depend on
/// them.
fn spent(api: &Runpod, home: &Home, now: u64) -> Spent {
    let (logged, _) = spend::total(&spend::load(&home.spend_log()));
    let running = home.state().map_or(0.0, |state| {
        spend::cost(state.usd_per_hour, now.saturating_sub(state.started))
    });
    let window = Window::Months(BUDGET_MONTHS);
    Spent::from_parts(
        logged,
        running,
        api.pod_billing(window, None).unwrap_or(0.0),
        api.volume_billing(window).unwrap_or(0.0),
    )
}

/// The budget in one line, for status reports, warnings and refusals.
fn budget_line(config: &Config, spent: &Spent) -> String {
    format!(
        "${:.2} of the ${:.2} budget is spent (pods ${:.2}, volume storage ${:.2})",
        spent.total(),
        config.budget_usd,
        spent.pods,
        spent.storage
    )
}

/// Prints the budget line of `cloud status` and `cloud spend`.
fn print_budget(config: &Config, spent: &Spent) {
    let note = match config.budget(spent.total()) {
        Budget::Fine => String::new(),
        Budget::Warn => format!("; that is over the ${:.2} warning line", config.budget_warn_usd),
        Budget::Spent => "; it is used up, and no pod starts until TJX raises `budget_usd` in lane.json".to_owned(),
    };
    println!("budget: {}{note}", budget_line(config, spent));
}

/// Stops the lane when its budget is used up, and says so when it is close. A pod that is
/// still up at that point is deleted.
fn hold_to_budget(api: &Runpod, home: &Home, config: &Config, spent: &Spent) -> Result<()> {
    match config.budget(spent.total()) {
        Budget::Fine => Ok(()),
        Budget::Warn => {
            say(&format!("warning: {}", budget_line(config, spent)));
            Ok(())
        }
        Budget::Spent => {
            if let Some(state) = home.state() {
                // If the delete fails, the pod's own lifetime cap still ends it.
                let _ = retire(api, home, &state, "deleted: the budget is used up");
            }
            Err(tool_err(
                "cloud",
                format!(
                    "the cloud lane is closed: {}. Only TJX raises `budget_usd` in lane.json. Run the job on the local lane",
                    budget_line(config, spent)
                ),
            ))
        }
    }
}

/// Books a pod that is gone without `cloud down` (its watchdog deleted it, or the credit ran
/// out). The end time is an estimate; when Runpod already reports what it billed for the pod,
/// that amount replaces the estimate.
fn book_lost(api: &Runpod, home: &Home, config: &Config, state: &State) -> Result<()> {
    let ended = watchdog_end(state, config, clock::now());
    let mut session = spend::Session::close(state, ended, "the pod's watchdog (end time estimated)");
    if let Ok(billed) = api.pod_billing(Window::Days(3), Some(&state.pod_id)) {
        if billed > 0.0 {
            session.usd = (billed * 1000.0).round() / 1000.0;
            session.how = "the pod's watchdog (amount from Runpod's billing)".to_owned();
        }
    }
    say(&format!(
        "the last pod ({}) was deleted by its watchdog; booked {} and ${:.3}",
        state.pod_id,
        clock::span(session.seconds),
        session.usd
    ));
    book(home, state, &session)
}

/// Copies the spend log as CSV into the Drive masters folder (`shared\cloud-gpu-spend.csv`), so
/// it can be read from another machine. Skipped when the Drive folder is not mounted.
fn mirror_spend(home: &Home) {
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

fn state_for(pod: &Pod, lock: &Lock, started: u64, max_secs: u64) -> State {
    State {
        pod_id: pod.id.clone(),
        gpu: pod.gpu_id().to_owned(),
        data_center: pod.data_center_id.clone().unwrap_or_default(),
        usd_per_hour: pod.cost,
        started,
        touched: clock::now(),
        thread: lock.thread.clone(),
        job: lock.job.clone(),
        max_secs,
    }
}

/// Starts a new pod: the first GPU type in the settings that has room gets it. The pod's
/// lifetime cap is what the rest of the budget pays for, when that is less than `max_hours`.
fn create_pod(api: &Runpod, home: &Home, config: &Config, lock: &Lock, spent: &Spent) -> Result<(State, Pod)> {
    let Some(life) = config.pod_life_secs(spent.total()) else {
        return Err(tool_err(
            "cloud",
            format!(
                "too little of the budget is left to start a pod: {}. Only TJX raises `budget_usd` in lane.json. Run the job on the local lane",
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
    if balance.billing_mode != "PREPAID" {
        say("warning: this Runpod account is invoiced, so the credit balance is not a spending cap");
    }
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
            ],
            volume_id: config.volume_id.clone(),
            own_disk_gb: config.volume_gb,
        };
        match api.create_pod(&request)? {
            Created::Pod(pod) => {
                if pod.cost > config.max_usd_per_hour + 1e-6 {
                    api.terminate(&pod.id)?;
                    return Err(tool_err(
                        "cloud",
                        format!(
                            "{gpu} now costs ${:.2} an hour, over the ${:.2} limit in lane.json; the pod was deleted at once",
                            pod.cost, config.max_usd_per_hour
                        ),
                    ));
                }
                let state = state_for(&pod, lock, clock::now(), life);
                home.save_state(Some(&state))?;
                say(&format!(
                    "started pod {} on {gpu} in {} at ${:.2} an hour (balance ${:.2})",
                    pod.id, config.data_center, pod.cost, balance.balance_amount
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

/// Returns the pod to use: the one on record when it is still up, a pod of ours that this PC
/// has no record of, or a new one. Never more than one.
fn ensure_pod(api: &Runpod, home: &Home, config: &Config, lock: &Lock, spent: &Spent) -> Result<(State, Pod)> {
    if let Some(state) = home.state() {
        match api.pod(&state.pod_id)? {
            Some(pod) if pod.is_alive() => {
                let age = clock::now().saturating_sub(state.started);
                if worth_keeping(age, state.cap(config)) {
                    return Ok((state, pod));
                }
                say("the pod is close to its lifetime cap; replacing it before the job starts");
                retire(api, home, &state, "replaced before the lifetime cap")?;
            }
            Some(pod) => {
                // Stopped or failed: nothing is billed for compute, but it must not linger.
                api.terminate(&pod.id)?;
                book_lost(api, home, config, &state)?;
            }
            None => book_lost(api, home, config, &state)?,
        }
    }
    for stray in api.pods()?.into_iter().filter(|p| p.name == pod::POD_NAME) {
        if !stray.is_alive() {
            api.terminate(&stray.id)?;
            continue;
        }
        let Some(pod) = api.pod(&stray.id)? else { continue };
        let started = clock::parse_rfc3339(&pod.created_at).unwrap_or_else(clock::now);
        // Its own cap is not known here; the pod's watchdog holds the real one.
        let state = state_for(&pod, lock, started, config.max_secs());
        home.save_state(Some(&state))?;
        say(&format!("found pod {} already running; using it", pod.id));
        return Ok((state, pod));
    }
    create_pod(api, home, config, lock, spent)
}

/// Waits until the pod runs and its sshd takes the key.
fn wait_for_ssh(api: &Runpod, home: &Home, state: &State, mut pod: Pod) -> Result<Ssh> {
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
            book_lost(api, home, &home.config()?, state)?;
            return Err(tool_err(
                "cloud",
                format!("the pod stopped while starting (status {})", pod.status),
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
    // The pod's start command is bringing the services up; wait for its verdict.
    let probe =
        "if [ -f /run/forge/ready ]; then echo ready; elif [ -f /run/forge/failed ]; then echo failed; else echo wait; fi";
    let deadline = clock::now() + READY_TIMEOUT_SECS;
    loop {
        match ssh.check(probe)?.trim() {
            "ready" => return Ok(()),
            "wait" if clock::now() < deadline => std::thread::sleep(Duration::from_secs(4)),
            // It failed or is too slow: try once from here, with the output visible.
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
fn open_tunnel(ssh: &Ssh, port: u16) -> Result<(Tunnel, String)> {
    let url = format!("http://127.0.0.1:{port}");
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(5)))
        .http_status_as_error(true)
        .build()
        .into();
    let mut tunnel = ssh.tunnel(port, pod::COMFY_PORT)?;
    for _ in 0..60 {
        if net::get_bytes(&agent, &format!("{url}/system_stats")).is_ok() {
            return Ok((tunnel, url));
        }
        if !tunnel.alive() {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(tool_err(
        "cloud",
        format!("the SSH forward to the pod's ComfyUI did not open. Is port {port} on this PC in use?"),
    ))
}

fn open(paths: &Paths) -> Result<Session> {
    let home = Home::of(paths);
    let config = home.config()?;
    let lock = Lock::require(&paths.tools, Lane::Cloud)?;
    let api = Runpod::new(&home.api_key()?);
    let began = clock::now();
    let spent = spent(&api, &home, began);
    hold_to_budget(&api, &home, &config, &spent)?;
    let (state, pod) = ensure_pod(&api, &home, &config, &lock, &spent)?;
    let ssh = wait_for_ssh(&api, &home, &state, pod)?;
    make_ready(&ssh)?;
    let (tunnel, comfy_url) = open_tunnel(&ssh, config.tunnel_port)?;
    // Say how long the wait was, and whether it included starting a pod, so the start-up time
    // in the guide can be checked against a real run.
    let waited = clock::span(clock::now().saturating_sub(began));
    let how = if state.started >= began {
        "a new pod"
    } else {
        "the pod that was already up"
    };
    say(&format!("pod {} is ready after {waited} ({how})", state.pod_id));
    Ok(Session {
        ssh,
        comfy_url,
        home,
        _tunnel: tunnel,
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

const USAGE: &str = "asset-forge cloud setup [--dry-run] [--region EUROPE]... [--data-center <id>] [--volume-gb 30 | --no-volume] [--max-usd-per-hour 0.90] [--idle-minutes 10] [--max-hours 4] [--image <ref>] [--keep] | key [<api key>] | up | down | status | spend | doctor";

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

/// Saves a new API key (given as an argument, or as the first line of standard input) once
/// Runpod has accepted it. The key itself is never printed.
fn set_key(home: &Home, key: Option<&str>) -> Result<()> {
    let key = match key {
        Some(key) => key.trim().to_owned(),
        None => {
            let mut line = String::new();
            std::io::stdin()
                .read_line(&mut line)
                .map_err(|e| tool_err("cloud", format!("no key on standard input: {e}")))?;
            line.trim().to_owned()
        }
    };
    if key.is_empty() {
        return Err(ForgeError::Usage("asset-forge cloud key <api key>".to_owned()));
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
    // lane like any other job.
    Lock::require(&paths.tools, Lane::Cloud)?;
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
    let api = Runpod::new(&home.api_key()?);
    let Some(state) = home.state() else {
        // No record: still make sure nothing of ours is left running.
        let mut found = 0;
        for stray in api.pods()?.into_iter().filter(|p| p.name == pod::POD_NAME) {
            api.terminate(&stray.id)?;
            found += 1;
        }
        println!("cloud: no pod on record; deleted {found} unrecorded pod(s)");
        return Ok(());
    };
    let session = retire(&api, home, &state, "cloud down")?;
    println!(
        "cloud: deleted pod {} after {} (about ${:.3} at ${:.2} an hour)",
        session.pod_id,
        clock::span(session.seconds),
        session.usd,
        session.usd_per_hour
    );
    Ok(())
}

fn status(home: &Home) -> Result<()> {
    let config = home.config()?;
    let api = Runpod::new(&home.api_key()?);
    let balance = api.balance()?;
    println!("balance: ${:.2} ({})", balance.balance_amount, balance.billing_mode);
    println!(
        "limits: ${:.2} an hour at most; deleted after {} idle minutes or {} hours",
        config.max_usd_per_hour, config.idle_minutes, config.max_hours
    );
    match home.state() {
        None => println!("pod: none on record"),
        Some(state) => match api.pod(&state.pod_id)? {
            Some(pod) if pod.is_alive() => {
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
            }
            gone => {
                if let Some(stopped) = gone {
                    api.terminate(&stopped.id)?;
                }
                book_lost(&api, home, &config, &state)?;
                println!("pod: none (the last one deleted itself)");
            }
        },
    }
    let strays = api
        .pods()?
        .into_iter()
        .filter(|p| p.is_alive() && home.state().is_none_or(|s| s.pod_id != p.id))
        .count();
    if strays > 0 {
        println!("warning: {strays} other pod(s) are running on this account");
    }
    // Last, so a pod that turned out to be gone is already in the log.
    print_budget(&config, &spent(&api, home, clock::now()));
    Ok(())
}

fn spend_report(home: &Home) -> Result<()> {
    let sessions = spend::load(&home.spend_log());
    let (usd, seconds) = spend::total(&sessions);
    println!(
        "spend log: {} session(s), {}, ${usd:.3}",
        sessions.len(),
        clock::span(seconds)
    );
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
    let api = Runpod::new(&home.api_key()?);
    let window = Window::Months(BUDGET_MONTHS);
    println!(
        "Runpod's own billing (it runs hours behind): pods ${:.3}, volume storage ${:.3}",
        api.pod_billing(window, None)?,
        api.volume_billing(window)?
    );
    if let Ok(config) = home.config() {
        print_budget(&config, &spent(&api, home, clock::now()));
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
        let dir = std::env::temp_dir().join("forge_cloud_home_key");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let home = Home { dir: dir.clone() };
        if std::env::var_os("RUNPOD_API_KEY").is_none() {
            assert!(home.api_key().is_err());
            std::fs::write(home.key_file(), "  rpa_test_key  \nnotes\n").unwrap();
            assert_eq!(home.api_key().ok().as_deref(), Some("rpa_test_key"));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_is_saved_and_cleared() {
        let dir = std::env::temp_dir().join("forge_cloud_home_state");
        let _ = std::fs::remove_dir_all(&dir);
        let home = Home { dir: dir.clone() };
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
}
