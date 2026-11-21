//! Hot reload of `config.json`: a background thread polls the file and publishes changes.
//!
//! Readers call [`ConfigWatcher::current`], an `ArcSwap` load with no lock, so the render
//! thread can read the config every frame. A file that fails to parse (for example while the
//! configurator is half-way through writing it) is ignored and the previous config stays live.
use crate::config::AppConfig;
use arc_swap::ArcSwap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};

/// Called after a new config has been published.
pub type ConfigNotify = Arc<dyn Fn() + Send + Sync>;

/// Shared, lock-free handle to the live config.
#[derive(Clone)]
pub struct ConfigHandle {
    live: Arc<ArcSwap<AppConfig>>,
}

impl std::fmt::Debug for ConfigHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigHandle").finish_non_exhaustive()
    }
}

impl ConfigHandle {
    /// A handle holding a fixed config (no file behind it).
    pub fn fixed(cfg: AppConfig) -> Self {
        Self {
            live: Arc::new(ArcSwap::from_pointee(cfg)),
        }
    }

    /// The config as of now. One atomic increment; no lock, no allocation.
    pub fn current(&self) -> Arc<AppConfig> {
        self.live.load_full()
    }

    fn publish(&self, cfg: AppConfig) {
        self.live.store(Arc::new(cfg));
    }
}

/// Watches a config file and republishes it when its modification time or size changes.
pub struct ConfigWatcher {
    handle: ConfigHandle,
    path: PathBuf,
    running: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for ConfigWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigWatcher")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileStamp {
    modified: Option<SystemTime>,
    len: u64,
}

fn stamp(path: &Path) -> Option<FileStamp> {
    let meta = fs::metadata(path).ok()?;
    Some(FileStamp {
        modified: meta.modified().ok(),
        len: meta.len(),
    })
}

impl ConfigWatcher {
    /// Loads `path` (defaults if missing or invalid) and starts polling it every `interval`.
    ///
    /// # Errors
    /// Returns the I/O error if the watcher thread cannot be spawned.
    pub fn spawn(path: PathBuf, interval: Duration, notify: ConfigNotify) -> io::Result<Self> {
        let handle = ConfigHandle::fixed(AppConfig::load(&path));
        let running = Arc::new(AtomicBool::new(true));
        let thread = {
            let handle = handle.clone();
            let running = Arc::clone(&running);
            let path = path.clone();
            thread::Builder::new()
                .name("fh6-config-watch".into())
                .spawn(move || watch_loop(&path, interval, &handle, &running, &*notify))?
        };
        Ok(Self {
            handle,
            path,
            running,
            thread: Some(thread),
        })
    }

    /// The live config. See [`ConfigHandle::current`].
    pub fn current(&self) -> Arc<AppConfig> {
        self.handle.current()
    }

    /// A cloneable handle for other threads.
    pub fn handle(&self) -> ConfigHandle {
        self.handle.clone()
    }

    /// The file being watched.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

fn watch_loop(path: &Path, interval: Duration, handle: &ConfigHandle, running: &AtomicBool, notify: &dyn Fn()) {
    let mut last = stamp(path);
    while running.load(Ordering::Relaxed) {
        thread::park_timeout(interval);
        let now = stamp(path);
        if now == last {
            continue;
        }
        match AppConfig::try_load(path) {
            Ok(cfg) => {
                last = now;
                if *handle.current() != cfg {
                    handle.publish(cfg);
                    tracing::info!(path = %path.display(), "config reloaded");
                    notify();
                }
            }
            // Keep `last` unchanged so the next poll retries; the old config stays live.
            Err(err) => tracing::debug!(error = %err, "config change not loadable yet"),
        }
    }
}

impl Drop for ConfigWatcher {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            thread.thread().unpark();
            let _ = thread.join();
        }
    }
}
