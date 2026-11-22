//! The camera sync service: keeps the game's camera files in line with the config, only while
//! the game is closed, and publishes a status for the OSD and the configurator.
//!
//! It writes only when `CameraPatchAuto` is on or the OSD/configurator asked for it
//! ([`CameraRequest::Apply`]); restore is [`CameraRequest::Restore`]. While the game runs a
//! change is queued (status "restart needed") and written as soon as the game exits.
use fh6_core::{AppConfig, CameraPatchHandle, CameraPatchState, CameraPatchStatus, CameraRequest};
use fh6_win::camera_patcher::{self, ApplyOutcome, CameraFiles, CameraPatch};
use std::path::{Path, PathBuf};

/// The service's state between ticks (pure logic; the thread supplies the world).
#[derive(Debug, Default)]
pub struct CameraSync {
    /// The last patch the files were brought to (or found at).
    synced: Option<CameraPatch>,
    /// A change is waiting for the game to exit.
    queued: bool,
    /// The last status published.
    status: CameraPatchStatus,
}

/// What a tick can see.
#[derive(Debug, Clone, Copy)]
pub struct World<'a> {
    /// The game install, if one was found.
    pub install: Option<&'a Path>,
    /// Where backups go (`None` when `%LOCALAPPDATA%` is unset).
    pub backup_root: Option<&'a Path>,
    /// Whether the game process is running.
    pub game_running: bool,
}

impl CameraSync {
    /// A service that has not synced anything yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// One step: handles a request, applies or queues the config, returns the status.
    pub fn tick(&mut self, cfg: &AppConfig, request: CameraRequest, world: World<'_>) -> &CameraPatchStatus {
        let install_dir = world.install.map(|p| p.display().to_string()).unwrap_or_default();
        let wants = cfg.camera_patch_auto || request == CameraRequest::Apply || self.queued;
        let state = match (request, world.install, world.backup_root) {
            (CameraRequest::Restore, Some(install), Some(root)) => {
                let files = CameraFiles::for_install(install);
                match camera_patcher::restore(&files, root, world.game_running) {
                    Ok(_) => {
                        self.synced = None;
                        self.queued = false;
                        CameraPatchState::Restored
                    }
                    Err(err) => CameraPatchState::Failed(err.to_string()),
                }
            }
            _ if !wants => {
                if matches!(
                    self.status.state,
                    CameraPatchState::Restored | CameraPatchState::Applied
                ) {
                    self.status.state.clone()
                } else {
                    CameraPatchState::Off
                }
            }
            (_, None, _) => CameraPatchState::NoInstall,
            (_, Some(_), None) => CameraPatchState::Failed("LOCALAPPDATA is not set; no backup folder".into()),
            (_, Some(install), Some(root)) => {
                let patch = CameraPatch::from_config(cfg);
                if self.synced == Some(patch) && !self.queued && request != CameraRequest::Apply {
                    self.status.state.clone()
                } else {
                    let files = CameraFiles::for_install(install);
                    match camera_patcher::apply(&files, &patch, root, world.game_running) {
                        Ok(ApplyOutcome::UpToDate) => {
                            self.synced = Some(patch);
                            self.queued = false;
                            CameraPatchState::UpToDate
                        }
                        Ok(ApplyOutcome::Applied(_)) => {
                            self.synced = Some(patch);
                            self.queued = false;
                            CameraPatchState::Applied
                        }
                        Ok(ApplyOutcome::Queued) => {
                            self.queued = true;
                            CameraPatchState::QueuedUntilGameExits
                        }
                        Err(err) => {
                            self.queued = false;
                            self.synced = Some(patch);
                            CameraPatchState::Failed(err.to_string())
                        }
                    }
                }
            }
        };
        let backup_dir = world
            .backup_root
            .and_then(camera_patcher::original_backup)
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        self.status = CameraPatchStatus {
            state,
            install_dir,
            backup_dir,
        };
        &self.status
    }
}

/// Starts the service thread: one tick every `period`, plus whenever a request arrives.
///
/// # Errors
/// Returns the I/O error if the thread cannot be spawned.
#[cfg(windows)]
pub fn spawn(
    config: fh6_core::ConfigHandle,
    handle: CameraPatchHandle,
    game_process: String,
    period: std::time::Duration,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new()
        .name("fh6-camera-sync".into())
        .spawn(move || {
            let mut sync = CameraSync::new();
            let backup_root = camera_patcher::default_backup_root();
            let mut install: Option<(String, Option<PathBuf>)> = None;
            loop {
                let cfg = config.current();
                let request = handle.take_request();
                let active = cfg.camera_patch_auto || request != CameraRequest::None || sync.queued;
                if active {
                    // Look the install up once per `GameInstallDir` value.
                    if install.as_ref().is_none_or(|(o, _)| *o != cfg.game_install_dir) {
                        let found = fh6_win::steam::find_game_install(&cfg.game_install_dir);
                        tracing::info!(install = ?found, "game install");
                        install = Some((cfg.game_install_dir.clone(), found));
                    }
                }
                let world = World {
                    install: install.as_ref().and_then(|(_, p)| p.as_deref()),
                    backup_root: backup_root.as_deref(),
                    game_running: active && fh6_win::system::is_process_running(&game_process),
                };
                let before = handle.status();
                let status = sync.tick(&cfg, request, world).clone();
                if *before != status {
                    tracing::info!(state = status.state.message(), "camera patch");
                }
                handle.publish(status);
                std::thread::park_timeout(period);
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Copies of the repo's `data/` camera files in a temp install (never the game's).
    fn sandbox(name: &str) -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let root = std::env::temp_dir().join(format!(
            "fh6_camsync_{name}_{}_{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        let cam = fh6_win::steam::camera_dir(&root.join("game"));
        fs::create_dir_all(&cam).unwrap();
        let data = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
        for f in [camera_patcher::SETTINGS_FILE, camera_patcher::PHYSICS_FILE] {
            fs::copy(data.join(f), cam.join(f)).unwrap();
        }
        root
    }

    fn cfg(auto: bool, fov: f32) -> AppConfig {
        AppConfig {
            camera_patch_auto: auto,
            wheel_less_cockpit_fov: fov,
            ..AppConfig::default()
        }
    }

    #[test]
    fn off_by_default_and_never_writes() {
        let root = sandbox("off");
        let game = root.join("game");
        let backups = root.join("backups");
        let before = fs::read(CameraFiles::for_install(&game).settings).unwrap();
        let mut s = CameraSync::new();
        let world = World {
            install: Some(&game),
            backup_root: Some(&backups),
            game_running: false,
        };
        assert_eq!(
            s.tick(&cfg(false, 99.0), CameraRequest::None, world).state,
            CameraPatchState::Off
        );
        assert_eq!(fs::read(CameraFiles::for_install(&game).settings).unwrap(), before);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn queues_while_running_and_applies_when_the_game_exits() {
        let root = sandbox("queue");
        let game = root.join("game");
        let backups = root.join("backups");
        let mut s = CameraSync::new();
        let running = World {
            install: Some(&game),
            backup_root: Some(&backups),
            game_running: true,
        };
        let st = s.tick(&cfg(true, 99.0), CameraRequest::None, running);
        assert_eq!(st.state, CameraPatchState::QueuedUntilGameExits);
        assert!(st.state.restart_needed());
        // Auto switched off meanwhile: the queued change still lands once the game exits.
        let closed = World {
            game_running: false,
            ..running
        };
        assert_eq!(
            s.tick(&cfg(false, 99.0), CameraRequest::None, closed).state,
            CameraPatchState::Applied
        );
        let text = fs::read_to_string(CameraFiles::for_install(&game).settings).unwrap();
        assert!(text.contains("CockpitCam\\FOV 99\r\n"));
        assert!(!s
            .tick(&cfg(false, 99.0), CameraRequest::None, closed)
            .backup_dir
            .is_empty());
        // Restore in one request.
        assert_eq!(
            s.tick(&cfg(false, 99.0), CameraRequest::Restore, closed).state,
            CameraPatchState::Restored
        );
        let text = fs::read_to_string(CameraFiles::for_install(&game).settings).unwrap();
        assert!(text.contains("CockpitCam\\FOV 80\r\n"));
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn explicit_apply_works_with_auto_off_and_missing_install_is_reported() {
        let root = sandbox("apply");
        let game = root.join("game");
        let backups = root.join("backups");
        let mut s = CameraSync::new();
        let world = World {
            install: Some(&game),
            backup_root: Some(&backups),
            game_running: false,
        };
        assert_eq!(
            s.tick(&cfg(false, 90.0), CameraRequest::Apply, world).state,
            CameraPatchState::Applied
        );
        let none = World { install: None, ..world };
        assert_eq!(
            s.tick(&cfg(true, 90.0), CameraRequest::None, none).state,
            CameraPatchState::NoInstall
        );
        fs::remove_dir_all(&root).ok();
    }
}
