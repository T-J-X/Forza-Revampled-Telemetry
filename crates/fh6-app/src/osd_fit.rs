//! The OSD's cockpit-fit backend on Phase 8's camera patcher.
//!
//! The OSD saves camera values to `config.json`; the camera sync service (`camera_sync`) picks
//! them up from the watched config and writes the game's camera files when `CameraPatchAuto` is
//! on (queued while the game runs). This backend only reads the published patch status to
//! choose the notice, and drives the camera-ring resync. No file I/O on the render thread.
use fh6_core::{AppConfig, CameraPatchHandle, CameraPatchState};
use fh6_input::InputShared;
use fh6_osd::{CameraApply, CockpitFit};
use std::sync::Arc;

/// The patcher-backed [`CockpitFit`].
#[derive(Debug)]
pub struct PatcherFit {
    camera: CameraPatchHandle,
    input: Arc<InputShared>,
}

impl PatcherFit {
    /// A backend reading `camera`'s status and resyncing through `input`.
    #[must_use]
    pub fn new(camera: CameraPatchHandle, input: Arc<InputShared>) -> Self {
        Self { camera, input }
    }
}

/// The notice for a camera change given the patch switch and the service's last status.
#[must_use]
pub fn camera_apply(patch_auto: bool, state: &CameraPatchState) -> CameraApply {
    if !patch_auto {
        return CameraApply::PatchOff;
    }
    match state {
        CameraPatchState::NoInstall | CameraPatchState::Failed(_) => CameraApply::Unavailable,
        // The OSD shows while the game runs: the service queues the write until it exits, and
        // the game reads the files on its next launch either way.
        _ => CameraApply::NextLaunch,
    }
}

impl CockpitFit for PatcherFit {
    fn camera_changed(&mut self, cfg: &AppConfig) -> CameraApply {
        camera_apply(cfg.camera_patch_auto, &self.camera.status().state)
    }

    fn can_resync(&self) -> bool {
        true
    }

    fn resync(&mut self, cfg: &AppConfig) {
        self.input.resync_camera(cfg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::CameraPatchStatus;

    #[test]
    fn notice_follows_the_switch_and_the_status() {
        assert_eq!(camera_apply(false, &CameraPatchState::Applied), CameraApply::PatchOff);
        assert_eq!(camera_apply(true, &CameraPatchState::Off), CameraApply::NextLaunch);
        assert_eq!(
            camera_apply(true, &CameraPatchState::QueuedUntilGameExits),
            CameraApply::NextLaunch
        );
        assert_eq!(
            camera_apply(true, &CameraPatchState::NoInstall),
            CameraApply::Unavailable
        );
        assert_eq!(
            camera_apply(true, &CameraPatchState::Failed("x".into())),
            CameraApply::Unavailable
        );
    }

    #[test]
    fn backend_reads_the_published_status_and_resyncs_the_ring() {
        let camera = CameraPatchHandle::new();
        let input = Arc::new(InputShared::new(0));
        let mut fit = PatcherFit::new(camera.clone(), Arc::clone(&input));
        let mut cfg = AppConfig {
            camera_patch_auto: true,
            resync_camera_index: 3,
            ..AppConfig::default()
        };
        camera.publish(CameraPatchStatus {
            state: CameraPatchState::NoInstall,
            ..CameraPatchStatus::default()
        });
        assert_eq!(fit.camera_changed(&cfg), CameraApply::Unavailable);
        cfg.camera_patch_auto = false;
        assert_eq!(fit.camera_changed(&cfg), CameraApply::PatchOff);

        assert!(fit.can_resync());
        fit.resync(&cfg);
        assert_eq!(input.camera_index(), 3);
    }
}
