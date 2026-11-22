//! Saves OSD edits to `config.json` off the render thread: edits are debounced (350 ms) so a
//! held slider writes once, and flushed when the OSD closes or the saver drops.
use fh6_core::AppConfig;
use std::path::PathBuf;
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Quiet time before a pending edit is written.
pub const DEBOUNCE: Duration = Duration::from_millis(350);

enum Msg {
    Save(Arc<AppConfig>),
    Flush,
}

/// The saver thread's handle. Sending never blocks the caller.
#[derive(Debug)]
pub struct ConfigSaver {
    tx: Option<Sender<Msg>>,
    thread: Option<JoinHandle<()>>,
}

impl std::fmt::Debug for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Msg::Save(_) => "Save",
            Msg::Flush => "Flush",
        })
    }
}

impl ConfigSaver {
    /// Starts the saver for `path`.
    ///
    /// # Errors
    /// Returns the I/O error if the thread cannot be spawned.
    pub fn spawn(path: PathBuf) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel::<Msg>();
        let thread = thread::Builder::new().name("fh6-config-saver".into()).spawn(move || {
            let mut pending: Option<Arc<AppConfig>> = None;
            loop {
                let msg = if pending.is_some() {
                    rx.recv_timeout(DEBOUNCE)
                } else {
                    rx.recv().map_err(|_| RecvTimeoutError::Disconnected)
                };
                match msg {
                    Ok(Msg::Save(cfg)) => pending = Some(cfg),
                    Ok(Msg::Flush) | Err(RecvTimeoutError::Timeout) => write(&path, pending.take()),
                    Err(RecvTimeoutError::Disconnected) => {
                        write(&path, pending.take());
                        break;
                    }
                }
            }
        })?;
        Ok(Self {
            tx: Some(tx),
            thread: Some(thread),
        })
    }

    /// Queues `cfg` to be written after the debounce.
    pub fn save(&self, cfg: Arc<AppConfig>) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(Msg::Save(cfg));
        }
    }

    /// Writes any pending edit now.
    pub fn flush(&self) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(Msg::Flush);
        }
    }
}

fn write(path: &PathBuf, cfg: Option<Arc<AppConfig>>) {
    let Some(cfg) = cfg else { return };
    match cfg.save(path) {
        Ok(()) => tracing::debug!(path = %path.display(), "OSD settings saved"),
        Err(err) => tracing::warn!(error = %err, path = %path.display(), "OSD settings not saved"),
    }
}

impl Drop for ConfigSaver {
    fn drop(&mut self) {
        // Dropping the sender ends the thread after it writes what is pending.
        self.tx = None;
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debounced_saves_write_the_last_value_without_bom() {
        let dir = std::env::temp_dir().join(format!("fh6-osd-saver-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("config.json");
        {
            let saver = ConfigSaver::spawn(path.clone()).expect("spawn");
            for i in 0..5 {
                let cfg = AppConfig {
                    osd_hold_ms: 300 + i * 50,
                    ..AppConfig::default()
                };
                saver.save(Arc::new(cfg));
            }
        }
        let bytes = std::fs::read(&path).expect("written");
        assert_ne!(bytes.get(..3), Some(&[0xEF, 0xBB, 0xBF][..]), "no BOM");
        let back = AppConfig::from_json_str(std::str::from_utf8(&bytes).expect("utf-8")).expect("parses");
        assert_eq!(back.osd_hold_ms, 500);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
