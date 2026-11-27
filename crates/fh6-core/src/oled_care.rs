//! OLED care: a slow pixel orbit and an idle dimmer that protect the LG C1 panel from burn-in.
//!
//! The engine has no clock of its own. The render loop calls it once per iteration with that
//! iteration's `dt`, the same value
//! [`VisibilityController::update`](crate::visibility::VisibilityController::update) gets:
//! [`OledCareEngine::update`] for a frame that shows the dash, [`OledCareEngine::rest`] while
//! nothing is on screen. The result depends only on the inputs, so tests run it on a fixed clock.
//!
//! `fh6-app` applies the result (SKILLS.md law 6: the orbit and the dimmer run in the app, not
//! in clusters). [`OledCare::offset_px`] moves the whole dash by whole pixels. [`OledCare::dim`]
//! turns down the light of the dash; what the dash covers stays covered.
use crate::config::{AppConfig, MASK_MANUAL_LOCK};
use crate::telemetry::FH6TelemetryData;
use std::time::Duration;

/// How far the orbit moves the dash from its anchor on each axis, in pixels.
pub const ORBIT_AMPLITUDE_PX: i32 = 2;
/// Time for one horizontal sweep of the orbit.
pub const ORBIT_PERIOD_X: Duration = Duration::from_secs(180);
/// Time for one vertical sweep of the orbit.
pub const ORBIT_PERIOD_Y: Duration = Duration::from_secs(240);
/// The car and the controller must be idle this long before the dash starts to dim.
pub const IDLE_BEFORE_DIM: Duration = Duration::from_secs(30);
/// How fast an idle dash dims: this much of [`OledCare::dim`] per second.
pub const DIM_PER_SECOND: f32 = 0.9;
/// The dim floor used when `OledDimmerMinAlpha` is not a number.
pub const DEFAULT_MIN_ALPHA: f32 = 0.35;
/// At or above this speed (m/s) the car counts as moving.
pub const MOVING_SPEED_MS: f32 = 0.2;
/// The engine counts as revving above idle plus this margin.
pub const REV_MARGIN_RPM: f32 = 150.0;
/// The revving threshold never drops below this, for cars that report a low or zero idle.
pub const REV_FLOOR_RPM: f32 = 1100.0;

/// What the engine reads for a frame that shows the dash.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OledCareInputs {
    /// `OledPixelOrbitEnabled`.
    pub orbit_enabled: bool,
    /// `OledIdleDimmerEnabled`.
    pub dimmer_enabled: bool,
    /// `OledDimmerMinAlpha`: the lowest [`OledCare::dim`] of an idle dash (clamped to 0..=1).
    pub min_alpha: f32,
    /// Car speed in m/s.
    pub speed_ms: f32,
    /// Engine speed in rpm.
    pub rpm: f32,
    /// Engine idle speed in rpm.
    pub idle_rpm: f32,
    /// The gear byte of the packet. A change wakes the dash: the driver shifted.
    pub gear: u8,
    /// Controller buttons held. A/B/X/Y are ignored (manual gearbox lock).
    pub buttons: u16,
    /// The OSD is open: the dash stays at full brightness.
    pub osd_open: bool,
}

impl OledCareInputs {
    /// The inputs for one frame: the OLED keys of `cfg`, the telemetry being drawn, the
    /// controller buttons held and whether the OSD is open.
    #[must_use]
    pub fn from_frame(cfg: &AppConfig, data: &FH6TelemetryData, buttons: u16, osd_open: bool) -> Self {
        Self {
            orbit_enabled: cfg.oled_pixel_orbit_enabled,
            dimmer_enabled: cfg.oled_idle_dimmer_enabled,
            min_alpha: cfg.oled_dimmer_min_alpha,
            speed_ms: data.speed_ms,
            rpm: data.current_engine_rpm,
            idle_rpm: data.engine_idle_rpm,
            gear: data.gear,
            buttons,
            osd_open,
        }
    }
}

/// What the render loop applies to a frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OledCare {
    /// Offset to add to the cluster anchor, in whole pixels (x right, y down).
    pub offset_px: (i32, i32),
    /// How bright the dash is shown, from the dim floor up to 1 (not dimmed). It is on the scale
    /// of an opacity: at 0.35 a white part of the dash is as bright as white at 35% opacity.
    pub dim: f32,
}

/// Tracks the orbit position and how long the car has been idle.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OledCareEngine {
    orbit: Duration,
    idle: Duration,
    last_gear: Option<u8>,
}

impl OledCareEngine {
    /// An engine at the start of the orbit, with a dash that has just been used.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances by `dt` for a frame that shows the dash, and returns what to apply to it.
    #[must_use = "the offset and the dim do nothing until the frame applies them"]
    pub fn update(&mut self, inputs: OledCareInputs, dt: Duration) -> OledCare {
        // The orbit keeps time while it is switched off, so the dash does not come back at the
        // first position every time.
        self.orbit = self.orbit.saturating_add(dt);
        let offset_px = if inputs.orbit_enabled {
            orbit_offset(self.orbit)
        } else {
            (0, 0)
        };
        let shifted = self.last_gear.is_some_and(|gear| gear != inputs.gear);
        self.last_gear = Some(inputs.gear);
        let awake = !inputs.dimmer_enabled || inputs.osd_open || shifted || in_use(&inputs);
        self.idle = if awake {
            Duration::ZERO
        } else {
            self.idle.saturating_add(dt)
        };
        let dimming_for = self.idle.saturating_sub(IDLE_BEFORE_DIM).as_secs_f32();
        let dim = (1.0 - DIM_PER_SECOND * dimming_for).max(dim_floor(inputs.min_alpha));
        OledCare { offset_px, dim }
    }

    /// Advances by `dt` while the dash is not on screen (it is hidden, or there is no device to
    /// draw with). The orbit keeps time, and the idle wait starts again, so the dash comes back
    /// at full brightness.
    pub fn rest(&mut self, dt: Duration) {
        self.orbit = self.orbit.saturating_add(dt);
        self.idle = Duration::ZERO;
    }
}

/// The orbit position `elapsed` after its start: a 3:4 Lissajous figure, rounded to whole pixels
/// so 1 px rims stay crisp.
fn orbit_offset(elapsed: Duration) -> (i32, i32) {
    let amplitude = f64::from(ORBIT_AMPLITUDE_PX);
    let angle = |period: Duration| std::f64::consts::TAU * elapsed.as_secs_f64() / period.as_secs_f64();
    // Each value is within the amplitude, so the casts cannot saturate.
    (
        (amplitude * angle(ORBIT_PERIOD_X).sin()).round() as i32,
        (amplitude * angle(ORBIT_PERIOD_Y).cos()).round() as i32,
    )
}

/// Whether the driver is using the car or the controller this frame. A value that is not a
/// number says nothing, so it counts as idle.
fn in_use(inputs: &OledCareInputs) -> bool {
    let known = |v: f32| if v.is_finite() { v } else { 0.0 };
    let rev_limit = (known(inputs.idle_rpm) + REV_MARGIN_RPM).max(REV_FLOOR_RPM);
    inputs.buttons & !MASK_MANUAL_LOCK != 0
        || known(inputs.speed_ms) >= MOVING_SPEED_MS
        || known(inputs.rpm) > rev_limit
}

/// `OledDimmerMinAlpha` as a usable dim floor.
fn dim_floor(min_alpha: f32) -> f32 {
    if min_alpha.is_finite() {
        min_alpha.clamp(0.0, 1.0)
    } else {
        DEFAULT_MIN_ALPHA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Duration = Duration::from_millis(10);

    /// Stationary with the engine at idle and no button held: nothing keeps the dash awake.
    fn parked() -> OledCareInputs {
        OledCareInputs {
            orbit_enabled: true,
            dimmer_enabled: true,
            min_alpha: 0.35,
            speed_ms: 0.0,
            rpm: 800.0,
            idle_rpm: 800.0,
            gear: 1,
            buttons: 0,
            osd_open: false,
        }
    }

    /// Runs the engine for `time` in steps of `step` and returns the last frame's result.
    fn run(engine: &mut OledCareEngine, inputs: OledCareInputs, time: Duration, step: Duration) -> OledCare {
        let mut out = engine.update(inputs, Duration::ZERO);
        let mut elapsed = Duration::ZERO;
        while elapsed < time {
            out = engine.update(inputs, step);
            elapsed += step;
        }
        out
    }

    fn assert_dim(got: f32, want: f32, context: &str) {
        assert!((got - want).abs() < 1e-5, "{context}: dim {got}, expected {want}");
    }

    /// An engine whose car has been parked for a minute: its dash is dimmed to the floor.
    fn dimmed() -> OledCareEngine {
        let mut engine = OledCareEngine::new();
        let dim = run(&mut engine, parked(), Duration::from_secs(60), FRAME).dim;
        assert_dim(dim, 0.35, "parked for a minute");
        engine
    }

    #[test]
    fn orbit_follows_the_lissajous_on_a_fixed_clock() {
        // x = round(2 sin(2 pi t / 180)), y = round(2 cos(2 pi t / 240)), worked by hand. The
        // figure repeats every 720 s.
        for (seconds, want) in [
            (0, (0, 2)),
            (45, (2, 1)),
            (60, (2, 0)),
            (90, (0, -1)),
            (120, (-2, -2)),
            (135, (-2, -2)),
            (180, (0, 0)),
            (240, (2, 2)),
            (720, (0, 2)),
            (765, (2, 1)),
        ] {
            let mut engine = OledCareEngine::new();
            let got = engine.update(parked(), Duration::from_secs(seconds)).offset_px;
            assert_eq!(got, want, "at {seconds} s");
        }
    }

    #[test]
    fn orbit_reaches_two_pixels_each_way_and_no_further() {
        let mut engine = OledCareEngine::new();
        let (mut x_range, mut y_range) = ((0, 0), (0, 0));
        for _ in 0..720 {
            let (dx, dy) = engine.update(parked(), Duration::from_secs(1)).offset_px;
            x_range = (x_range.0.min(dx), x_range.1.max(dx));
            y_range = (y_range.0.min(dy), y_range.1.max(dy));
        }
        assert_eq!(x_range, (-2, 2));
        assert_eq!(y_range, (-2, 2));
    }

    #[test]
    fn orbit_is_the_same_at_any_frame_rate() {
        let mut engine = OledCareEngine::new();
        let got = run(&mut engine, parked(), Duration::from_secs(60), FRAME).offset_px;
        assert_eq!(got, (2, 0), "6000 frames of 10 ms land where one 60 s step does");
    }

    #[test]
    fn orbit_keeps_time_while_it_is_off() {
        let mut engine = OledCareEngine::new();
        let off = OledCareInputs {
            orbit_enabled: false,
            ..parked()
        };
        assert_eq!(engine.update(off, Duration::from_secs(45)).offset_px, (0, 0));
        let got = engine.update(parked(), Duration::ZERO).offset_px;
        assert_eq!(got, (2, 1), "switched back on 45 s into the orbit");
    }

    #[test]
    fn orbit_keeps_time_while_the_dash_is_not_on_screen() {
        let mut engine = OledCareEngine::new();
        engine.rest(Duration::from_secs(45));
        let got = engine.update(parked(), Duration::ZERO).offset_px;
        assert_eq!(got, (2, 1), "back on screen 45 s into the orbit");
    }

    #[test]
    fn dims_after_thirty_idle_seconds_at_the_set_rate() {
        // Full brightness for 30 s, then 0.9 per second down to the 0.35 floor.
        for (idle_ms, want) in [
            (29_990, 1.0),
            (30_000, 1.0),
            (30_500, 0.55),
            (30_700, 0.37),
            (31_000, 0.35),
            (600_000, 0.35),
        ] {
            let mut engine = OledCareEngine::new();
            let got = run(&mut engine, parked(), Duration::from_millis(idle_ms), FRAME).dim;
            assert_dim(got, want, &format!("after {idle_ms} ms idle"));
        }
    }

    #[test]
    fn dimming_is_the_same_at_any_frame_rate() {
        for step_ms in [10, 250] {
            let mut engine = OledCareEngine::new();
            let step = Duration::from_millis(step_ms);
            let got = run(&mut engine, parked(), Duration::from_millis(30_500), step).dim;
            assert_dim(got, 0.55, &format!("{step_ms} ms frames"));
        }
    }

    #[test]
    fn moving_revving_or_a_button_wakes_the_dash() {
        // Each row: what the driver does, the frame it gives, whether a dimmed dash wakes.
        let rolling = |speed_ms| OledCareInputs { speed_ms, ..parked() };
        let revving = |rpm, idle_rpm| OledCareInputs {
            rpm,
            idle_rpm,
            ..parked()
        };
        let holding = |buttons| OledCareInputs { buttons, ..parked() };
        for (label, inputs, wakes) in [
            ("rolling at 0.2 m/s", rolling(0.2), true),
            ("creeping at 0.19 m/s", rolling(0.19), false),
            (
                "1101 rpm over an 800 rpm idle (limit 1100)",
                revving(1101.0, 800.0),
                true,
            ),
            (
                "1100 rpm over an 800 rpm idle (limit 1100)",
                revving(1100.0, 800.0),
                false,
            ),
            (
                "2151 rpm over a 2000 rpm idle (limit 2150)",
                revving(2151.0, 2000.0),
                true,
            ),
            (
                "2150 rpm over a 2000 rpm idle (limit 2150)",
                revving(2150.0, 2000.0),
                false,
            ),
            ("Menu held", holding(0x0010), true),
            ("RB held", holding(0x0200), true),
        ] {
            let got = dimmed().update(inputs, FRAME).dim;
            assert_dim(got, if wakes { 1.0 } else { 0.35 }, label);
        }
    }

    #[test]
    fn waking_restarts_the_thirty_second_wait() {
        let mut engine = dimmed();
        let moving = OledCareInputs {
            speed_ms: 5.0,
            ..parked()
        };
        assert_dim(engine.update(moving, FRAME).dim, 1.0, "woken");
        let waited = run(&mut engine, parked(), Duration::from_secs(30), FRAME).dim;
        assert_dim(waited, 1.0, "30 s after waking");
        let dimming = run(&mut engine, parked(), Duration::from_millis(500), FRAME).dim;
        assert_dim(dimming, 0.55, "30.5 s after waking");
    }

    #[test]
    fn a_gear_change_wakes_the_dash() {
        let mut engine = dimmed();
        let second = OledCareInputs { gear: 2, ..parked() };
        assert_dim(engine.update(second, FRAME).dim, 1.0, "shifted to second");
        let waited = run(&mut engine, second, Duration::from_secs(30), FRAME).dim;
        assert_dim(waited, 1.0, "30 s after the shift");
        let dimming = run(&mut engine, second, Duration::from_millis(500), FRAME).dim;
        assert_dim(dimming, 0.55, "30.5 s after the shift");
    }

    #[test]
    fn gearbox_buttons_never_wake_the_dash() {
        // A, B, X and Y are the handbrake, the gearbox and rewind: the overlay never reads them.
        let mut engine = OledCareEngine::new();
        let shifting = OledCareInputs {
            buttons: 0xF000,
            ..parked()
        };
        let got = run(&mut engine, shifting, Duration::from_secs(60), FRAME).dim;
        assert_dim(got, 0.35, "A/B/X/Y held for a minute");
    }

    #[test]
    fn osd_keeps_the_dash_at_full_brightness() {
        let mut engine = OledCareEngine::new();
        let osd = OledCareInputs {
            osd_open: true,
            ..parked()
        };
        let got = run(&mut engine, osd, Duration::from_secs(60), FRAME).dim;
        assert_dim(got, 1.0, "OSD open for a minute");
        let closed = run(&mut engine, parked(), Duration::from_secs(30), FRAME).dim;
        assert_dim(closed, 1.0, "30 s after the OSD closed");
        let dimming = run(&mut engine, parked(), Duration::from_millis(500), FRAME).dim;
        assert_dim(dimming, 0.55, "30.5 s after the OSD closed");
    }

    #[test]
    fn a_dash_that_was_off_screen_comes_back_at_full_brightness() {
        let mut engine = dimmed();
        engine.rest(FRAME);
        assert_dim(engine.update(parked(), FRAME).dim, 1.0, "first frame back");
        let waited = run(&mut engine, parked(), Duration::from_millis(29_980), FRAME).dim;
        assert_dim(waited, 1.0, "just under 30 s after coming back");
        let dimming = run(&mut engine, parked(), Duration::from_millis(520), FRAME).dim;
        assert_dim(dimming, 0.541, "30.51 s after coming back");
    }

    #[test]
    fn dimmer_off_never_dims() {
        let mut engine = OledCareEngine::new();
        let off = OledCareInputs {
            dimmer_enabled: false,
            ..parked()
        };
        let got = run(&mut engine, off, Duration::from_secs(600), FRAME).dim;
        assert_dim(got, 1.0, "dimmer off, idle for 10 minutes");
    }

    #[test]
    fn dim_floor_is_clamped_and_a_bad_value_falls_back() {
        for (min_alpha, want) in [
            (0.5, 0.5),
            (0.0, 0.0),
            (-3.0, 0.0),
            (1.0, 1.0),
            (7.0, 1.0),
            (f32::NAN, 0.35),
            (f32::INFINITY, 0.35),
        ] {
            let mut engine = OledCareEngine::new();
            let inputs = OledCareInputs { min_alpha, ..parked() };
            let got = run(&mut engine, inputs, Duration::from_secs(120), FRAME).dim;
            assert_dim(got, want, &format!("OledDimmerMinAlpha {min_alpha}"));
        }
    }

    #[test]
    fn broken_telemetry_does_not_keep_the_dash_awake() {
        let mut engine = OledCareEngine::new();
        let broken = OledCareInputs {
            speed_ms: f32::NAN,
            rpm: f32::INFINITY,
            idle_rpm: f32::NAN,
            ..parked()
        };
        let got = run(&mut engine, broken, Duration::from_secs(60), FRAME).dim;
        assert_dim(got, 0.35, "NaN speed, infinite rpm, NaN idle");
    }

    #[test]
    fn inputs_come_from_the_config_and_the_drawn_packet() {
        let cfg = AppConfig {
            oled_pixel_orbit_enabled: false,
            oled_idle_dimmer_enabled: true,
            oled_dimmer_min_alpha: 0.5,
            ..AppConfig::default()
        };
        let data = FH6TelemetryData {
            speed_ms: 12.0,
            current_engine_rpm: 3000.0,
            engine_idle_rpm: 900.0,
            gear: 4,
            ..FH6TelemetryData::default()
        };
        assert_eq!(
            OledCareInputs::from_frame(&cfg, &data, 0x0100, true),
            OledCareInputs {
                orbit_enabled: false,
                dimmer_enabled: true,
                min_alpha: 0.5,
                speed_ms: 12.0,
                rpm: 3000.0,
                idle_rpm: 900.0,
                gear: 4,
                buttons: 0x0100,
                osd_open: true,
            }
        );
    }
}
