//! The seam between the OSD's Fit & camera tab and the cockpit-fit backend (Phase 8).
//!
//! The OSD owns the rows and the live editing; the [`CockpitFit`] implementation owns what the
//! values mean outside `config.json`: the game's camera files (written by Phase 8's camera sync
//! service, never while the game runs) and the camera-ring resync. The dash-blocker rows bind
//! straight to Phase 8's `OemDash*` keys ([`SCRIM_ROWS`]). [`ConfigOnlyFit`] keeps every value in
//! `config.json` (previews and tests); the app passes its patcher-backed implementation.
use fh6_core::AppConfig;

/// What a camera change (FOV, seat height, wheel offset) will do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraApply {
    /// Saved; the camera patch writes it to the game's files, which the game reads on its next
    /// launch (the files are never written while the game runs).
    #[default]
    NextLaunch,
    /// Written to the camera files now (the game is not running).
    Applied,
    /// The game install or its camera files could not be found or written.
    Unavailable,
    /// Saved in `config.json` only: the camera patch (`CameraPatchAuto`) is off.
    PatchOff,
}

impl CameraApply {
    /// The notice shown under camera rows.
    #[must_use]
    pub fn notice(self) -> &'static str {
        match self {
            Self::NextLaunch => "Applies on next launch (camera file)",
            Self::Applied => "Saved to the camera file",
            Self::Unavailable => "Camera file not found: saved in config only",
            Self::PatchOff => "Camera patch is off: saved in config only",
        }
    }
}

/// One numeric fit setting with its config binding (a scrim parameter, say).
#[derive(Debug, Clone, Copy)]
pub struct FitRow {
    /// Row label (sentence case).
    pub label: &'static str,
    /// Unit suffix shown after the value (`"%"`, `" px"`, `"×"`).
    pub unit: &'static str,
    /// Smallest value.
    pub min: f32,
    /// Largest value.
    pub max: f32,
    /// One D-pad step.
    pub step: f32,
    /// Decimals shown.
    pub decimals: u8,
    /// Reads the value from config.
    pub get: fn(&AppConfig) -> f32,
    /// Writes the (clamped, stepped) value to config.
    pub set: fn(&mut AppConfig, f32),
}

/// The cockpit-fit backend behind the Fit & camera tab.
///
/// Every method is called from the render thread: implementations must not block or do file
/// I/O there (the camera sync service does the writing on its own thread).
pub trait CockpitFit: Send {
    /// FOV, seat height, wheel offset or the camera patch switch changed in `cfg` (the OSD
    /// saves it to `config.json`). Says what the change will do.
    fn camera_changed(&mut self, cfg: &AppConfig) -> CameraApply;

    /// The rows shown under "Dash blocker".
    fn scrim_rows(&self) -> &'static [FitRow] {
        SCRIM_ROWS
    }

    /// Whether a "Resync camera" action is offered (re-aligns the RB camera ring).
    fn can_resync(&self) -> bool {
        false
    }

    /// Runs the resync action with the current config (`ResyncCameraIndex`).
    fn resync(&mut self, _cfg: &AppConfig) {}
}

fn get_opacity(c: &AppConfig) -> f32 {
    c.oem_dash_opaque_pct as f32
}
fn set_opacity(c: &mut AppConfig, v: f32) {
    c.oem_dash_opaque_pct = (v.round() as i32).clamp(0, 100);
}
fn get_width(c: &AppConfig) -> f32 {
    c.oem_dash_width_scale
}
fn set_width(c: &mut AppConfig, v: f32) {
    c.oem_dash_width_scale = v.clamp(0.5, 1.6);
}
fn get_height(c: &AppConfig) -> f32 {
    c.oem_dash_height_scale
}
fn set_height(c: &mut AppConfig, v: f32) {
    c.oem_dash_height_scale = v.clamp(0.5, 1.6);
}
fn get_arch(c: &AppConfig) -> f32 {
    c.oem_dash_top_arch * 100.0
}
fn set_arch(c: &mut AppConfig, v: f32) {
    c.oem_dash_top_arch = (v / 100.0).clamp(0.0, 1.0);
}
fn get_softness(c: &AppConfig) -> f32 {
    c.oem_dash_corner_softness * 100.0
}
fn set_softness(c: &mut AppConfig, v: f32) {
    c.oem_dash_corner_softness = (v / 100.0).clamp(0.0, 1.0);
}
fn get_offset_x(c: &AppConfig) -> f32 {
    c.oem_dash_offset_x as f32
}
fn set_offset_x(c: &mut AppConfig, v: f32) {
    c.oem_dash_offset_x = (v.round() as i32).clamp(-600, 600);
}
fn get_offset_y(c: &AppConfig) -> f32 {
    c.oem_dash_binnacle_offset_y as f32
}
fn set_offset_y(c: &mut AppConfig, v: f32) {
    c.oem_dash_binnacle_offset_y = (v.round() as i32).clamp(-600, 600);
}
fn get_feather(c: &AppConfig) -> f32 {
    c.oem_dash_feather_px
}
fn set_feather(c: &mut AppConfig, v: f32) {
    c.oem_dash_feather_px = v.clamp(8.0, 240.0);
}
fn get_void(c: &AppConfig) -> f32 {
    c.oem_dash_column_void_px
}
fn set_void(c: &mut AppConfig, v: f32) {
    c.oem_dash_column_void_px = v.clamp(0.0, 600.0);
}

/// The dash-blocker rows, bound to Phase 8's `OemDash*` keys with `AppConfig::sanitized`'s
/// ranges.
pub static SCRIM_ROWS: &[FitRow] = &[
    FitRow {
        label: "Blocker strength",
        unit: "%",
        min: 0.0,
        max: 100.0,
        step: 5.0,
        decimals: 0,
        get: get_opacity,
        set: set_opacity,
    },
    FitRow {
        label: "Blocker width",
        unit: "×",
        min: 0.50,
        max: 1.60,
        step: 0.02,
        decimals: 2,
        get: get_width,
        set: set_width,
    },
    FitRow {
        label: "Blocker height",
        unit: "×",
        min: 0.50,
        max: 1.60,
        step: 0.02,
        decimals: 2,
        get: get_height,
        set: set_height,
    },
    FitRow {
        label: "Crown arch",
        unit: "%",
        min: 0.0,
        max: 100.0,
        step: 5.0,
        decimals: 0,
        get: get_arch,
        set: set_arch,
    },
    FitRow {
        label: "Corner softness",
        unit: "%",
        min: 0.0,
        max: 100.0,
        step: 5.0,
        decimals: 0,
        get: get_softness,
        set: set_softness,
    },
    FitRow {
        label: "Blocker left / right",
        unit: " px",
        min: -600.0,
        max: 600.0,
        step: 8.0,
        decimals: 0,
        get: get_offset_x,
        set: set_offset_x,
    },
    FitRow {
        label: "Blocker up / down",
        unit: " px",
        min: -600.0,
        max: 600.0,
        step: 8.0,
        decimals: 0,
        get: get_offset_y,
        set: set_offset_y,
    },
    FitRow {
        label: "Edge feather",
        unit: " px",
        min: 8.0,
        max: 240.0,
        step: 4.0,
        decimals: 0,
        get: get_feather,
        set: set_feather,
    },
    FitRow {
        label: "Steering column gap",
        unit: " px",
        min: 0.0,
        max: 600.0,
        step: 10.0,
        decimals: 0,
        get: get_void,
        set: set_void,
    },
];

/// A backend that keeps every value in `config.json` and never touches the game folder
/// (preview renders and tests).
#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigOnlyFit;

impl CockpitFit for ConfigOnlyFit {
    fn camera_changed(&mut self, cfg: &AppConfig) -> CameraApply {
        if cfg.camera_patch_auto {
            CameraApply::NextLaunch
        } else {
            CameraApply::PatchOff
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrim_rows_round_trip_within_their_ranges() {
        let mut cfg = AppConfig::default();
        for row in SCRIM_ROWS {
            for v in [row.min, (row.min + row.max) * 0.5, row.max] {
                (row.set)(&mut cfg, v);
                assert!(((row.get)(&cfg) - v).abs() <= row.step, "{} at {v}", row.label);
            }
        }
    }

    #[test]
    fn scrim_rows_start_from_the_phase_8_defaults() {
        let cfg = AppConfig::default();
        for row in SCRIM_ROWS {
            let v = (row.get)(&cfg);
            assert!(
                v >= row.min && v <= row.max,
                "{} default {v} outside its range",
                row.label
            );
        }
    }

    #[test]
    fn every_notice_is_distinct() {
        let all = [
            CameraApply::NextLaunch,
            CameraApply::Applied,
            CameraApply::Unavailable,
            CameraApply::PatchOff,
        ];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.notice(), b.notice());
            }
        }
    }
}
