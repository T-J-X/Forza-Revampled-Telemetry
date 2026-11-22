//! The four review states every mockup, golden render and critique uses (tokens `states`).
//! `limiterRpm` 8700, `maxRpm` 9000, `idleRpm` 900.
use fh6_core::FH6TelemetryData;

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
const NEUTRAL: u8 = 11;

/// The four states (tokens table; redline is captured on a strobe-on phase).
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
            engine_max_rpm: 8700.0,
            engine_idle_rpm: 900.0,
            current_engine_rpm: self.rpm,
            speed_ms: speed,
            velocity_x: speed * slip.sin(),
            velocity_z: speed * slip.cos(),
            acceleration_x: self.g_lat * G,
            acceleration_z: self.g_long * G,
            angular_velocity_y: if speed > 2.0 { self.g_lat * G / speed } else { 0.0 },
            gear: self.gear,
            accel: (self.throttle * 255.0).round() as u8,
            brake: (self.brake * 255.0).round() as u8,
            car_ordinal: 0,
            ..Default::default()
        }
    }
}
