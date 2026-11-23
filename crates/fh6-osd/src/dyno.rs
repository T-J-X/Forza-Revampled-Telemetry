//! The live dyno: synthetic telemetry that sweeps the revs through six gears, so the cluster in
//! the OSD live view (and the demo, bench and previews) keeps moving while the game is paused.
use fh6_core::telemetry::FH6TelemetryData;

/// Seconds of sweep per full six-gear cycle at rate 1.
const CYCLE: f32 = 6.0;

/// Telemetry for the dyno sweep at `phase` (0..6, one unit per gear).
#[must_use]
pub fn dyno_telemetry(phase: f32) -> FH6TelemetryData {
    sweep(phase.rem_euclid(CYCLE), 9000.0, 850.0)
}

fn sweep(phase: f32, max_rpm: f32, idle_rpm: f32) -> FH6TelemetryData {
    let gear = ((phase.floor() as i32) + 1).clamp(1, 6) as u8;
    let sub = phase.fract();
    let rpm = idle_rpm + (max_rpm - idle_rpm) * (0.35 + 0.65 * sub);
    let speed_mph = f32::from(gear) * 28.0 + sub * 22.0;
    let lifting = sub > 0.95;
    FH6TelemetryData {
        is_race_on: 0,
        engine_max_rpm: max_rpm,
        engine_idle_rpm: idle_rpm,
        current_engine_rpm: rpm,
        speed_ms: speed_mph * 0.447_04,
        gear,
        accel: if lifting { 0 } else { 255 },
        brake: if lifting { 204 } else { 0 },
        boost_psi: if sub > 0.3 { 16.5 * sub } else { 2.0 },
        power_watts: 450_000.0 * (rpm / max_rpm),
        ..Default::default()
    }
}

/// A 120-point S-curve racing line (z 0..178.5 m) for trail previews.
#[must_use]
pub fn synthetic_trail() -> Vec<[f32; 3]> {
    (0..120)
        .map(|i| {
            let z = i as f32 * 1.5;
            [(z * 0.03).sin() * 15.0, 0.0, z]
        })
        .collect()
}

/// Generates the dyno sweep frame by frame (0.8 gears per second).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveDynoGenerator {
    /// Position in the sweep (0..6).
    pub phase: f32,
    /// Redline of the synthetic engine.
    pub max_rpm: f32,
    /// Idle of the synthetic engine.
    pub idle_rpm: f32,
}

impl Default for LiveDynoGenerator {
    fn default() -> Self {
        Self {
            phase: 0.0,
            max_rpm: 8500.0,
            idle_rpm: 950.0,
        }
    }
}

impl LiveDynoGenerator {
    /// A sweep starting in first gear.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances by `dt` seconds and returns the frame. Allocation-free.
    pub fn tick(&mut self, dt: f32) -> FH6TelemetryData {
        let dt = if dt.is_finite() { dt.clamp(0.0, 0.25) } else { 0.0 };
        self.phase = (self.phase + dt * 0.8).rem_euclid(CYCLE);
        sweep(self.phase, self.max_rpm, self.idle_rpm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dyno_sweeps_through_the_gears() {
        let mut dyno = LiveDynoGenerator::new();
        let first = dyno.tick(0.016);
        assert_eq!(first.gear, 1);
        assert!(first.current_engine_rpm >= 950.0);
        for _ in 0..200 {
            dyno.tick(0.016);
        }
        let later = dyno.tick(0.016);
        assert!(later.gear > 1, "gear {}", later.gear);
        assert!(dyno.tick(f32::NAN).current_engine_rpm.is_finite());
    }

    #[test]
    fn fixed_phase_is_repeatable() {
        assert_eq!(dyno_telemetry(2.5), dyno_telemetry(8.5));
        assert_eq!(synthetic_trail().len(), 120);
    }
}
