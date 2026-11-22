//! The seam between the OSD's Fit & camera tab and the cockpit-fit implementation (Phase 8).
//!
//! The OSD owns the rows and the live editing; the [`CockpitFit`] implementation owns what the
//! values mean outside `config.json`: writing the game's camera files (through the camera
//! patcher, only while the game is closed) and the scrim's parameter set. Phase 9 ships
//! [`ConfigOnlyFit`], which keeps every value in `config.json` and reports camera changes as
//! "applies on next launch"; Phase 8 replaces it with its patcher-backed implementation.
use fh6_core::AppConfig;

/// What happened to a camera change (FOV, seat height, wheel offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CameraApply {
    /// Saved; the game reads it on its next launch (the camera files are only written while
    /// the game is closed).
    #[default]
    NextLaunch,
    /// Applied to the camera files now (the game is not running).
    Applied,
    /// The camera files could not be found or written.
    Unavailable,
}

impl CameraApply {
    /// The notice shown under camera rows.
    #[must_use]
    pub fn notice(self) -> &'static str {
        match self {
            Self::NextLaunch => "Applies on next launch (camera file)",
            Self::Applied => "Saved to the camera file",
            Self::Unavailable => "Camera file not found: saved in config only",
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
/// I/O there (queue the work to a worker thread instead).
pub trait CockpitFit: Send {
    /// FOV, seat height or wheel offset changed in `cfg` (already saved to `config.json` by
    /// the OSD). Queue the camera-file write and say when it takes effect.
    fn camera_changed(&mut self, cfg: &AppConfig) -> CameraApply;

    /// The scrim rows shown under "Dash blocker" (Phase 8 adds height, arch, feather, corner
    /// softness).
    fn scrim_rows(&self) -> &'static [FitRow];

    /// Whether a "Resync camera" action is offered (Phase 8: re-aligns the RB camera ring).
    fn can_resync(&self) -> bool {
        false
    }

    /// Runs the resync action.
    fn resync(&mut self) {}
}

fn get_opacity(c: &AppConfig) -> f32 {
    c.oem_dash_opaque_pct as f32
}
fn set_opacity(c: &mut AppConfig, v: f32) {
    c.oem_dash_opaque_pct = v.round() as i32;
}
fn get_width(c: &AppConfig) -> f32 {
    c.oem_dash_width_scale
}
fn set_width(c: &mut AppConfig, v: f32) {
    c.oem_dash_width_scale = v;
}
fn get_offset(c: &AppConfig) -> f32 {
    c.oem_dash_binnacle_offset_y as f32
}
fn set_offset(c: &mut AppConfig, v: f32) {
    c.oem_dash_binnacle_offset_y = v.round() as i32;
}

/// The scrim rows bound to the existing `AppConfig` fields.
pub static CONFIG_SCRIM_ROWS: &[FitRow] = &[
    FitRow {
        label: "Dash blocker",
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
        label: "Blocker height offset",
        unit: " px",
        min: -200.0,
        max: 200.0,
        step: 4.0,
        decimals: 0,
        get: get_offset,
        set: set_offset,
    },
];

/// Phase 9's backend: values live in `config.json`; camera files are left to the patcher on
/// the next launch. It never touches the game folder.
#[derive(Debug, Default, Clone, Copy)]
pub struct ConfigOnlyFit;

impl CockpitFit for ConfigOnlyFit {
    fn camera_changed(&mut self, _cfg: &AppConfig) -> CameraApply {
        CameraApply::NextLaunch
    }

    fn scrim_rows(&self) -> &'static [FitRow] {
        CONFIG_SCRIM_ROWS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_rows_round_trip() {
        let mut cfg = AppConfig::default();
        for row in CONFIG_SCRIM_ROWS {
            let v = (row.min + row.max) * 0.5;
            (row.set)(&mut cfg, v);
            assert!(((row.get)(&cfg) - v).abs() <= row.step, "{}", row.label);
        }
    }
}
