//! Review telemetry shared by the golden tests, `preview-export` and the cluster chats: the four
//! review states (tokens `states`), the 60-frame motion sequence, and a [`Driver`] that feeds a
//! cluster exactly as the overlay does (smoothed signals, real `dt`).
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_kit::{Cluster, FontChoice, FrameInput, Logo, Palette, SignalSmoother, Tokens, View};

/// One review state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReviewState {
    /// File stem, e.g. `02_mid_cruise`.
    pub name: &'static str,
    /// Engine rpm.
    pub rpm: f32,
    /// Raw gear byte (0 = R, 11 = N).
    pub gear: u8,
    /// Speed in mph.
    pub mph: f32,
    /// Throttle 0..=1.
    pub throttle: f32,
    /// Brake 0..=1.
    pub brake: f32,
    /// Lateral G.
    pub g_lat: f32,
    /// Longitudinal G.
    pub g_long: f32,
    /// Body slip angle in degrees.
    pub slip_deg: f32,
}

/// Neutral in the telemetry gear byte.
pub const NEUTRAL: u8 = 11;
/// The limiter rpm of every review state (`EngineMaxRpm`).
pub const LIMITER_RPM: f32 = 8700.0;
/// Idle rpm.
pub const IDLE_RPM: f32 = 900.0;

/// The four states (tokens `states`; redline is captured on a strobe-on phase).
pub const STATES: [ReviewState; 4] = [
    ReviewState {
        name: "01_idle",
        rpm: 900.0,
        gear: NEUTRAL,
        mph: 0.0,
        throttle: 0.0,
        brake: 0.0,
        g_lat: 0.0,
        g_long: 0.0,
        slip_deg: 0.0,
    },
    ReviewState {
        name: "02_mid_cruise",
        rpm: 5200.0,
        gear: 4,
        mph: 118.0,
        throttle: 0.42,
        brake: 0.0,
        g_lat: 0.18,
        g_long: 0.08,
        slip_deg: 1.0,
    },
    ReviewState {
        name: "03_redline_strobe",
        rpm: 8680.0,
        gear: 5,
        mph: 152.0,
        throttle: 1.0,
        brake: 0.0,
        g_lat: 0.05,
        g_long: 0.42,
        slip_deg: 0.0,
    },
    ReviewState {
        name: "04_drift_braking",
        rpm: 7100.0,
        gear: 3,
        mph: 74.0,
        throttle: 0.30,
        brake: 0.62,
        g_lat: -0.96,
        g_long: -0.58,
        slip_deg: 27.0,
    },
];

const G: f32 = 9.80665;
const MPH_TO_MS: f32 = 1.0 / 2.236_936_3;

impl ReviewState {
    /// The telemetry packet for this state.
    #[must_use]
    pub fn telemetry(&self) -> FH6TelemetryData {
        let speed = self.mph * MPH_TO_MS;
        let slip = self.slip_deg.to_radians();
        FH6TelemetryData {
            is_race_on: 1,
            engine_max_rpm: LIMITER_RPM,
            engine_idle_rpm: IDLE_RPM,
            current_engine_rpm: self.rpm,
            speed_ms: speed,
            velocity_x: speed * slip.sin(),
            velocity_z: speed * slip.cos(),
            acceleration_x: self.g_lat * G,
            acceleration_z: self.g_long * G,
            angular_velocity_y: if speed > 2.0 { self.g_lat * G / speed } else { 0.0 },
            power_watts: 310_000.0 * self.throttle * (self.rpm / LIMITER_RPM),
            boost_psi: 18.0 * self.throttle - 2.0,
            gear: self.gear,
            accel: (self.throttle * 255.0).round() as u8,
            brake: (self.brake * 255.0).round() as u8,
            car_ordinal: 0,
            ..Default::default()
        }
    }
}

/// Frames in the review motion sequence (60 Hz, one second).
pub const SEQUENCE_FRAMES: u32 = 60;
/// The sequence's frame time.
pub const SEQUENCE_DT: f32 = 1.0 / 60.0;

/// Frame `i` of the review motion sequence: idle (0–9), a full-throttle pull in 3rd to the
/// limiter (10–29), on the limiter so the strobe runs two cycles (30–41), the upshift to 4th with
/// the rpm drop (42–44), then pulling again in 4th (45–59).
#[must_use]
pub fn sequence_frame(i: u32) -> FH6TelemetryData {
    let i = i.min(SEQUENCE_FRAMES - 1) as f32;
    let smooth = |a: f32, b: f32, t: f32| {
        let t = t.clamp(0.0, 1.0);
        a + (b - a) * t * t * (3.0 - 2.0 * t)
    };
    let (rpm, gear, mph, throttle, g_long) = if i < 10.0 {
        (IDLE_RPM, NEUTRAL, 0.0, 0.0, 0.0)
    } else if i < 30.0 {
        let t = (i - 10.0) / 19.0;
        (smooth(2600.0, 8680.0, t), 3, smooth(38.0, 92.0, t), 1.0, 0.62)
    } else if i < 42.0 {
        // Bouncing off the limiter.
        let bounce = if (i as u32).is_multiple_of(2) { 8690.0 } else { 8640.0 };
        (bounce, 3, 92.5, 1.0, 0.05)
    } else if i < 45.0 {
        let t = (i - 42.0) / 2.0;
        (smooth(8600.0, 6420.0, t), 4, 93.0, 0.25, -0.05)
    } else {
        let t = (i - 45.0) / 14.0;
        (smooth(6420.0, 7350.0, t), 4, smooth(93.0, 103.0, t), 1.0, 0.48)
    };
    let speed = mph * MPH_TO_MS;
    FH6TelemetryData {
        is_race_on: 1,
        engine_max_rpm: LIMITER_RPM,
        engine_idle_rpm: IDLE_RPM,
        current_engine_rpm: rpm,
        speed_ms: speed,
        velocity_z: speed,
        acceleration_x: 0.12 * G,
        acceleration_z: g_long * G,
        angular_velocity_y: if speed > 2.0 { 0.12 * G / speed } else { 0.0 },
        power_watts: 310_000.0 * throttle * (rpm / LIMITER_RPM),
        boost_psi: 18.0 * throttle - 2.0,
        gear,
        accel: (throttle * 255.0).round() as u8,
        car_ordinal: 0,
        ..Default::default()
    }
}

/// What a [`Driver`] passes to every `update` besides the telemetry.
#[derive(Debug, Clone, Copy)]
pub struct DriveContext<'a> {
    /// Config (speed unit, names).
    pub config: &'a AppConfig,
    /// Palette.
    pub palette: &'a Palette,
    /// Tokens.
    pub tokens: &'a Tokens,
    /// Fonts.
    pub fonts: FontChoice,
    /// Logo zone content.
    pub logo: &'a Logo,
    /// View.
    pub view: View,
    /// Reduced-flash mode.
    pub reduced_flash: bool,
}

/// Feeds a cluster as the overlay does: smoothed signals and a running clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct Driver {
    smoother: SignalSmoother,
    t: f32,
}

impl Driver {
    /// A driver at t = 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seconds fed so far.
    #[must_use]
    pub fn t(&self) -> f32 {
        self.t
    }

    /// One `update` with `data`, then advances the clock by `dt`.
    pub fn step(&mut self, cluster: &mut dyn Cluster, data: &FH6TelemetryData, ctx: &DriveContext<'_>, dt: f32) {
        let signals = self.smoother.update(data, ctx.config, ctx.tokens, dt);
        cluster.update(&FrameInput {
            telemetry: data,
            signals: &signals,
            dt,
            t: self.t,
            config: ctx.config,
            palette: ctx.palette,
            tokens: ctx.tokens,
            fonts: ctx.fonts,
            logo: ctx.logo,
            view: ctx.view,
            reduced_flash: ctx.reduced_flash,
        });
        self.t += dt;
    }

    /// Settles `cluster` on a review state: `frames + 1` updates at `dt` (springs settle, the
    /// strobe lands on its on-phase for the redline state at the default 30 frames).
    pub fn settle(
        &mut self,
        cluster: &mut dyn Cluster,
        state: &ReviewState,
        ctx: &DriveContext<'_>,
        frames: u32,
        dt: f32,
    ) {
        let data = state.telemetry();
        for _ in 0..=frames {
            self.step(cluster, &data, ctx, dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_covers_idle_limiter_and_upshift() {
        let idle = sequence_frame(0);
        assert_eq!(idle.gear, NEUTRAL);
        let limiter = sequence_frame(35);
        assert!(limiter.current_engine_rpm >= 8600.0 && limiter.gear == 3);
        let after = sequence_frame(44);
        assert_eq!(after.gear, 4);
        assert!(after.current_engine_rpm < 6500.0);
        assert_eq!(sequence_frame(500).gear, sequence_frame(59).gear);
    }

    #[test]
    fn states_are_the_token_states() {
        assert_eq!(STATES.len(), 4);
        assert_eq!(STATES[2].name, "03_redline_strobe");
        assert!(STATES[2].rpm / LIMITER_RPM > 0.99);
    }
}
