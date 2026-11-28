//! Real `gpu-lock` processes sharing one folder: the order of the queue and the hand-over gap,
//! holders and waiters whose process dies, a lock file made by hand, the time limit, `take`,
//! `release`, `cancel`, `status`, and the cloud lane's two slots.
#![allow(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test harness: a setup step that fails should stop the test with its message"
)]

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime};

const GPU_LOCK: &str = env!("CARGO_BIN_EXE_gpu-lock");

/// Not a test: the job the other tests queue. It sleeps for `GPU_LOCK_TEST_SLEEP_MS`.
#[test]
#[ignore = "the job the other tests run; it only sleeps"]
fn sleeper() {
    let ms = std::env::var("GPU_LOCK_TEST_SLEEP_MS")
        .ok()
        .and_then(|ms| ms.parse().ok())
        .unwrap_or(0);
    std::thread::sleep(Duration::from_millis(ms));
}

/// One line of the log.
#[derive(Debug)]
struct Event {
    /// Milliseconds since midnight UTC.
    ms: i64,
    event: String,
    thread: String,
}

/// A lock folder of its own for one test.
struct Folder(PathBuf);

impl Folder {
    fn new(name: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("gpu-lock-it-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(GPU_LOCK);
        command.args(args).env("GPU_LOCK_DIR", &self.0).stdin(Stdio::null());
        command
    }

    /// The job's command line: this test binary, running only `sleeper`.
    fn job() -> Vec<String> {
        let me = std::env::current_exe().unwrap().display().to_string();
        vec![
            me,
            "sleeper".into(),
            "--exact".into(),
            "--ignored".into(),
            "--quiet".into(),
        ]
    }

    /// A queued run of `thread` on `lane` whose job sleeps `ms`.
    fn run(&self, thread: &str, lane: &str, ms: u64, extra: &[&str]) -> Child {
        let mut command = self.command(&["run", "--lane", lane, "--thread", thread, "--job", "sleep"]);
        command
            .args(extra)
            .arg("--")
            .args(Self::job())
            .env("GPU_LOCK_TEST_SLEEP_MS", ms.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        command.spawn().unwrap()
    }

    fn output(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// A `take` of `thread`, started in the background as a thread starts it.
    fn take(&self, thread: &str, extra: &[&str]) -> Child {
        let mut command = self.command(&["take", "--thread", thread, "--job", "renders"]);
        command.args(extra).stdout(Stdio::null()).stderr(Stdio::null());
        command.spawn().unwrap()
    }

    /// `wait` for `thread`'s take, for at most half a minute.
    fn wait_locked(&self, thread: &str) -> Output {
        self.output(&["wait", "--thread", thread, "--wait-minutes", "0.5"])
    }

    fn holder(&self, slot: &str) -> Option<String> {
        let text = fs::read_to_string(self.0.join(slot)).ok()?;
        text.lines()
            .find_map(|line| line.strip_prefix("thread: ").map(str::to_owned))
    }

    fn waiting(&self, lane: &str, thread: &str) -> bool {
        let Ok(entries) = fs::read_dir(self.0.join("gpu-queue").join(lane)) else {
            return false;
        };
        entries.filter_map(Result::ok).any(|entry| {
            entry.path().extension().is_some_and(|ext| ext == "ticket")
                && fs::read_to_string(entry.path())
                    .is_ok_and(|text| text.lines().any(|line| line == format!("thread: {thread}")))
        })
    }

    fn log(&self) -> Vec<Event> {
        let text = fs::read_to_string(self.0.join("gpu-queue").join("log.txt")).unwrap_or_default();
        text.lines()
            .filter_map(|line| {
                let mut words = line.split(' ');
                let time = words.next()?;
                let _lane = words.next()?;
                let _slot = words.next()?;
                let event = words.next()?.to_owned();
                let thread = line.split('"').nth(1)?.to_owned();
                let clock = time.get(11..23)?;
                let hours: i64 = clock.get(0..2)?.parse().ok()?;
                let minutes: i64 = clock.get(3..5)?.parse().ok()?;
                let seconds: i64 = clock.get(6..8)?.parse().ok()?;
                let millis: i64 = clock.get(9..12)?.parse().ok()?;
                let ms = ((hours * 60 + minutes) * 60 + seconds) * 1000 + millis;
                Some(Event { ms, event, thread })
            })
            .collect()
    }

    fn event(&self, event: &str, thread: &str) -> Option<i64> {
        self.log()
            .into_iter()
            .find(|e| e.event == event && e.thread == thread)
            .map(|e| e.ms)
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn wait_until(what: &str, timeout: Duration, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(
            start.elapsed() < timeout,
            "timed out after {timeout:?} waiting for {what}"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn finish(child: &mut Child, timeout: Duration) -> i32 {
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return status.code().unwrap_or(-1);
        }
        if start.elapsed() > timeout {
            let _ = child.kill();
            panic!("a gpu-lock run did not end within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

/// The gap from one event to a later one, in milliseconds (a midnight in between is allowed for).
fn gap(from: i64, to: i64) -> i64 {
    let gap = to - from;
    if gap < -43_200_000 {
        gap + 86_400_000
    } else {
        gap
    }
}

const LONG: Duration = Duration::from_secs(30);

#[test]
fn the_lock_goes_to_whoever_asked_first_the_moment_it_is_released() {
    let folder = Folder::new("fifo");
    let mut a = folder.run("A", "local", 1500, &[]);
    wait_until("A to hold the lock", LONG, || {
        folder.holder("gpu.lock").as_deref() == Some("A")
    });
    let mut b = folder.run("B", "local", 400, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    let mut c = folder.run("C", "local", 400, &[]);
    wait_until("C to wait", LONG, || folder.waiting("local", "C"));
    for child in [&mut a, &mut b, &mut c] {
        assert_eq!(finish(child, LONG), 0);
    }
    let took: Vec<String> = folder
        .log()
        .into_iter()
        .filter(|e| e.event == "took")
        .map(|e| e.thread)
        .collect();
    assert_eq!(took, ["A", "B", "C"]);
    for (before, after) in [("A", "B"), ("B", "C")] {
        let released = folder.event("released", before).unwrap();
        let took = folder.event("took", after).unwrap();
        let gap = gap(released, took);
        assert!(
            (0..1000).contains(&gap),
            "{after} took the lock {gap} ms after {before} released it"
        );
    }
    assert_eq!(folder.holder("gpu.lock"), None);
}

#[test]
fn a_holder_whose_process_tree_dies_frees_the_lock_at_once() {
    let folder = Folder::new("crash");
    let mut a = folder.run("A", "local", 15_000, &[]);
    wait_until("A to hold the lock", LONG, || {
        folder.holder("gpu.lock").as_deref() == Some("A")
    });
    let mut b = folder.run("B", "local", 200, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    let _ = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &a.id().to_string()])
        .output();
    let _ = a.kill();
    let _ = a.wait();
    let start = Instant::now();
    wait_until("B to take over", LONG, || folder.event("took", "B").is_some());
    assert!(start.elapsed() < Duration::from_secs(3), "{:?}", start.elapsed());
    assert_eq!(finish(&mut b, LONG), 0);
    assert!(folder.event("stale", "A").is_some(), "the takeover is in the log");
}

#[test]
fn a_job_whose_supervisor_was_killed_keeps_the_lock_until_it_ends() {
    let folder = Folder::new("orphan");
    let mut a = folder.run("A", "local", 3000, &[]);
    wait_until("A to hold the lock", LONG, || {
        fs::read_to_string(folder.0.join("gpu.lock")).is_ok_and(|text| text.contains("job_pid:"))
    });
    let mut b = folder.run("B", "local", 200, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    // Only the supervisor dies; its job goes on using the GPU.
    a.kill().unwrap();
    let _ = a.wait();
    std::thread::sleep(Duration::from_millis(1000));
    assert!(folder.event("took", "B").is_none(), "B waits while A's job still runs");
    wait_until("B to take over after the job ends", LONG, || {
        folder.event("took", "B").is_some()
    });
    assert_eq!(finish(&mut b, LONG), 0);
}

#[test]
fn a_waiter_whose_process_dies_leaves_the_queue() {
    let folder = Folder::new("waiter");
    let mut a = folder.run("A", "local", 1500, &[]);
    wait_until("A to hold the lock", LONG, || {
        folder.holder("gpu.lock").as_deref() == Some("A")
    });
    let mut b = folder.run("B", "local", 200, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    let mut c = folder.run("C", "local", 200, &[]);
    wait_until("C to wait", LONG, || folder.waiting("local", "C"));
    b.kill().unwrap();
    let _ = b.wait();
    assert_eq!(finish(&mut a, LONG), 0);
    assert_eq!(finish(&mut c, LONG), 0);
    let took: Vec<String> = folder
        .log()
        .into_iter()
        .filter(|e| e.event == "took")
        .map(|e| e.thread)
        .collect();
    assert_eq!(took, ["A", "C"]);
}

#[test]
fn a_lock_file_made_by_hand_counts_until_it_is_15_minutes_old() {
    let folder = Folder::new("hand");
    let lock = folder.0.join("gpu.lock");
    fs::write(
        &lock,
        "thread: the old way\nstarted: 2026-11-28T00:00:00Z\njob: by hand\n",
    )
    .unwrap();
    let mut args = vec!["run", "--thread", "N", "--job", "j", "--wait-minutes", "0.02", "--"];
    let job = Folder::job();
    args.extend(job.iter().map(String::as_str));
    let out = folder.output(&args);
    assert_eq!(out.status.code(), Some(3), "it gave up waiting");
    assert_eq!(folder.holder("gpu.lock").as_deref(), Some("the old way"));
    let file = fs::File::options().write(true).open(&lock).unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(16 * 60))
        .unwrap();
    drop(file);
    let mut args = vec!["run", "--thread", "N", "--job", "j", "--"];
    args.extend(job.iter().map(String::as_str));
    let out = folder.output(&args);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(folder.holder("gpu.lock"), None, "released after the job");
    assert!(folder.event("stale", "the old way").is_some());
}

#[test]
fn the_time_limit_stops_the_job_and_frees_the_lock() {
    let folder = Folder::new("limit");
    let start = Instant::now();
    let mut a = folder.run("A", "local", 30_000, &["--minutes", "0.05"]);
    assert_eq!(finish(&mut a, LONG), 5);
    assert!(start.elapsed() < Duration::from_secs(15), "{:?}", start.elapsed());
    assert_eq!(folder.holder("gpu.lock"), None);
}

#[test]
fn take_holds_until_release_and_a_run_of_the_same_thread_goes_at_once() {
    let folder = Folder::new("take");
    let mut take = folder.take("T", &["--minutes", "1"]);
    let out = folder.wait_locked("T");
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("LOCKED gpu.lock"));
    assert_eq!(folder.holder("gpu.lock").as_deref(), Some("T"));
    let mut b = folder.run("B", "local", 100, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    let start = Instant::now();
    let mut own = folder.run("T", "local", 100, &[]);
    assert_eq!(finish(&mut own, LONG), 0);
    assert!(start.elapsed() < Duration::from_secs(10));
    assert!(b.try_wait().unwrap().is_none(), "B still waits");
    let out = folder.output(&["release", "--thread", "T"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("RELEASED gpu.lock"));
    assert_eq!(finish(&mut take, LONG), 0, "the take ends when it is released");
    assert_eq!(finish(&mut b, LONG), 0);
}

#[test]
fn a_take_stopped_before_its_turn_gives_up_its_place() {
    let folder = Folder::new("caller");
    let mut a = folder.run("A", "local", 2000, &[]);
    wait_until("A to hold the lock", LONG, || {
        folder.holder("gpu.lock").as_deref() == Some("A")
    });
    let mut take = folder.take("T", &["--minutes", "1"]);
    wait_until("T to wait", LONG, || folder.waiting("local", "T"));
    take.kill().unwrap();
    let _ = take.wait();
    // The next look at the queue (any waiter, or `status`) finds the place dead and removes it.
    let out = folder.output(&["status", "--lane", "local"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("nobody waits for the local lane"), "{text}");
    assert!(!folder.waiting("local", "T"), "the dead place is removed");
    assert_eq!(finish(&mut a, LONG), 0);
    std::thread::sleep(Duration::from_secs(1));
    assert_eq!(
        folder.holder("gpu.lock"),
        None,
        "nobody holds the lock for a take that is gone"
    );
}

#[test]
fn a_take_ends_at_its_time_limit() {
    let folder = Folder::new("takelimit");
    let start = Instant::now();
    let mut take = folder.take("T", &["--minutes", "0.05"]);
    assert_eq!(finish(&mut take, LONG), 5);
    assert!(start.elapsed() < Duration::from_secs(15), "{:?}", start.elapsed());
    assert_eq!(folder.holder("gpu.lock"), None);
}

#[test]
fn wait_fails_when_the_thread_has_no_take() {
    let folder = Folder::new("nowait");
    let out = folder.output(&["wait", "--thread", "Nobody"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn release_lets_go_only_of_the_named_thread() {
    let folder = Folder::new("release");
    let mut take = folder.take("T1", &["--minutes", "1"]);
    assert_eq!(folder.wait_locked("T1").status.code(), Some(0));
    let out = folder.output(&["release", "--thread", "T2"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("nothing to let go of"));
    assert_eq!(folder.holder("gpu.lock").as_deref(), Some("T1"));
    let out = folder.output(&["release", "--thread", "T1"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(finish(&mut take, LONG), 0);
    assert_eq!(folder.holder("gpu.lock"), None);
}

#[test]
fn cancel_stops_a_running_job_and_frees_the_lock() {
    let folder = Folder::new("cancel");
    let mut a = folder.run("A", "local", 20_000, &[]);
    wait_until("A to hold the lock", LONG, || {
        folder.holder("gpu.lock").as_deref() == Some("A")
    });
    let out = folder.output(&["cancel", "--thread", "A"]);
    assert_eq!(out.status.code(), Some(0), "{}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("RELEASED gpu.lock"));
    assert_eq!(finish(&mut a, LONG), 4);
    assert_eq!(folder.holder("gpu.lock"), None);
}

#[test]
fn the_cloud_lane_runs_two_jobs_at_once() {
    let folder = Folder::new("cloud");
    let mut a = folder.run("A", "cloud", 2500, &[]);
    wait_until("A to hold a cloud lock", LONG, || {
        folder.holder("gpu-cloud.lock").as_deref() == Some("A")
    });
    let mut b = folder.run("B", "cloud", 2500, &[]);
    wait_until("B to hold the second cloud lock", LONG, || {
        folder.holder("gpu-cloud-2.lock").as_deref() == Some("B")
    });
    let mut c = folder.run("C", "cloud", 100, &[]);
    wait_until("C to wait", LONG, || folder.waiting("cloud", "C"));
    std::thread::sleep(Duration::from_millis(400));
    assert!(folder.event("took", "C").is_none(), "both cloud locks are held");
    for child in [&mut a, &mut b, &mut c] {
        assert_eq!(finish(child, LONG), 0);
    }
    assert!(folder.event("took", "C").is_some());
}

#[test]
fn status_shows_the_holder_and_the_line_in_order() {
    let folder = Folder::new("status");
    let mut take = folder.take("T", &["--minutes", "1"]);
    assert_eq!(folder.wait_locked("T").status.code(), Some(0));
    let mut b = folder.run("B", "local", 100, &[]);
    wait_until("B to wait", LONG, || folder.waiting("local", "B"));
    let mut c = folder.run("C", "local", 100, &[]);
    wait_until("C to wait", LONG, || folder.waiting("local", "C"));
    let out = folder.output(&["status", "--lane", "local"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("gpu.lock: held by \"T\""), "{text}");
    let first = text.find("1. \"B\"").unwrap_or(usize::MAX);
    let second = text.find("2. \"C\"").unwrap_or(usize::MAX);
    assert!(first < second && second < usize::MAX, "{text}");
    let out = folder.output(&["release", "--thread", "T"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(finish(&mut take, LONG), 0);
    assert_eq!(finish(&mut b, LONG), 0);
    assert_eq!(finish(&mut c, LONG), 0);
}

#[test]
fn a_command_that_cannot_start_frees_the_lock() {
    let folder = Folder::new("nostart");
    let out = folder.output(&[
        "run",
        "--thread",
        "A",
        "--job",
        "j",
        "--",
        "gpu-lock-test-no-such-program",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(folder.holder("gpu.lock"), None);
    assert!(folder.event("released", "A").is_some());
}
