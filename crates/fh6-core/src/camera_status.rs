//! The camera patch status shared between the camera sync service (fh6-app), the OSD and the
//! configurator. Lock-free reads (`ArcSwap`); the service publishes a new value only when the
//! state changes.
use arc_swap::ArcSwap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

/// Where the camera patch stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CameraPatchState {
    /// `CameraPatchAuto` is off and nothing was requested.
    #[default]
    Off,
    /// No game install was found (Steam libraries or `GameInstallDir`).
    NoInstall,
    /// The game files already match the config.
    UpToDate,
    /// The files were written (game closed); the next game start uses them.
    Applied,
    /// The game is running: the change waits and is written when it exits.
    QueuedUntilGameExits,
    /// The originals were restored from the backup.
    Restored,
    /// Something failed; the message says what (no file was half-written).
    Failed(String),
}

impl CameraPatchState {
    /// A short line for the OSD and the configurator.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Off => "Camera patch off",
            Self::NoInstall => "Game install not found",
            Self::UpToDate => "Camera files up to date",
            Self::Applied => "Camera files written; takes effect next game start",
            Self::QueuedUntilGameExits => "Game running: change applies when it exits (restart needed)",
            Self::Restored => "Original camera files restored",
            Self::Failed(m) => m.as_str(),
        }
    }

    /// Whether the user has to restart the game to see the change.
    #[must_use]
    pub fn restart_needed(&self) -> bool {
        matches!(self, Self::QueuedUntilGameExits)
    }
}

/// The published status.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CameraPatchStatus {
    /// State.
    pub state: CameraPatchState,
    /// The game install in use (empty when none).
    pub install_dir: String,
    /// The original backup folder (empty before the first change).
    pub backup_dir: String,
}

/// A request from the OSD or configurator to the camera sync service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CameraRequest {
    /// Nothing pending.
    None = 0,
    /// Apply the config now (or queue it while the game runs), even with `CameraPatchAuto` off.
    Apply = 1,
    /// Restore the original files from the first backup.
    Restore = 2,
}

/// Shared status plus a pending request. Clone freely.
#[derive(Debug, Clone, Default)]
pub struct CameraPatchHandle {
    status: Arc<ArcSwap<CameraPatchStatus>>,
    request: Arc<AtomicU8>,
}

impl CameraPatchHandle {
    /// A handle with status `Off` and no request.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The status as of now.
    #[must_use]
    pub fn status(&self) -> Arc<CameraPatchStatus> {
        self.status.load_full()
    }

    /// Publishes a status (the service).
    pub fn publish(&self, status: CameraPatchStatus) {
        if *self.status.load_full() != status {
            self.status.store(Arc::new(status));
        }
    }

    /// Asks the service to apply or restore.
    pub fn request(&self, req: CameraRequest) {
        self.request.store(req as u8, Ordering::Relaxed);
    }

    /// Takes the pending request (the service).
    pub fn take_request(&self) -> CameraRequest {
        match self.request.swap(0, Ordering::Relaxed) {
            1 => CameraRequest::Apply,
            2 => CameraRequest::Restore,
            _ => CameraRequest::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_are_taken_once() {
        let h = CameraPatchHandle::new();
        assert_eq!(h.take_request(), CameraRequest::None);
        h.request(CameraRequest::Restore);
        assert_eq!(h.take_request(), CameraRequest::Restore);
        assert_eq!(h.take_request(), CameraRequest::None);
    }

    #[test]
    fn status_publishes_and_explains() {
        let h = CameraPatchHandle::new();
        assert_eq!(h.status().state, CameraPatchState::Off);
        h.publish(CameraPatchStatus {
            state: CameraPatchState::QueuedUntilGameExits,
            ..CameraPatchStatus::default()
        });
        assert!(h.status().state.restart_needed());
        assert!(h.status().state.message().contains("restart"));
    }
}
