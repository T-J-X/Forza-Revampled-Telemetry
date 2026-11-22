//! A small synthetic drivetrain: throttle raises revs, gears trade revs for speed, a limiter
//! bounces at the top. Produces Forza-shaped telemetry for previews without the game.
use fh6_core::FH6TelemetryData;

/// Gear ratios (overall, arbitrary units) for gears 1..=9.
const RATIOS: [f32; 9] = [3.2, 2.35, 1.82, 1.46, 1.21, 1.02, 0.87, 0.75, 0.65];
/// Speed (m/s) = rpm / (ratio * K).
const K: f32 = 115.0;
/// Limiter, idle and the limiter bounce.
const MAX_RPM: f32 = 8700.0;
const IDLE_RPM: f32 = 900.0;
const BOUNCE_RPM: f32 = 180.0;

/// The drivetrain state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sim {
    rpm: f32,
    speed: f32,
    gear: u8,
    throttle: f32,
    t: f32,
}

impl Default for Sim {
    fn default() -> Self {
        Self::new()
    }
}

impl Sim {
    /// Idling in first.
    #[must_use]
    pub fn new() -> Self {
        Self {
            rpm: IDLE_RPM,
            speed: 0.0,
            gear: 1,
            throttle: 0.0,
            t: 0.0,
        }
    }

    /// Selects gear `g` (1..=9); revs follow the road speed.
    pub fn shift(&mut self, g: u8) {
        let g = g.clamp(1, 9);
        self.gear = g;
        self.rpm = (self.speed * RATIOS[usize::from(g - 1)] * K).clamp(IDLE_RPM, MAX_RPM);
    }

    /// Advances `dt` seconds with the throttle held or not.
    pub fn step(&mut self, dt: f32, throttle_held: bool) {
        let dt = dt.clamp(0.0, 0.1);
        self.t += dt;
        let target = if throttle_held { 1.0 } else { 0.0 };
        self.throttle += (target - self.throttle) * (1.0 - (-dt / 0.06).exp());
        let ratio = RATIOS[usize::from(self.gear.clamp(1, 9) - 1)];
        // Revs climb faster in low gears; engine braking and drag when off throttle.
        let accel = self.throttle * 5200.0 * (ratio / 1.8).sqrt() - (1.0 - self.throttle) * 2600.0;
        self.rpm = (self.rpm + accel * dt).clamp(IDLE_RPM, MAX_RPM);
        if self.rpm >= MAX_RPM - 1.0 && self.throttle > 0.5 {
            // Limiter bounce.
            self.rpm = MAX_RPM - BOUNCE_RPM * (0.5 + 0.5 * (self.t * 60.0).sin());
        }
        self.speed = self.rpm / (ratio * K);
        if self.rpm <= IDLE_RPM + 1.0 {
            self.speed = (self.speed - 3.0 * dt).max(0.0);
        }
    }

    /// The packet the game would send.
    #[must_use]
    pub fn telemetry(&self) -> FH6TelemetryData {
        FH6TelemetryData {
            is_race_on: 1,
            engine_max_rpm: MAX_RPM,
            engine_idle_rpm: IDLE_RPM,
            current_engine_rpm: self.rpm,
            speed_ms: self.speed,
            velocity_z: self.speed,
            gear: self.gear,
            accel: (self.throttle * 255.0) as u8,
            acceleration_x: 4.0 * (self.t * 0.6).sin(),
            acceleration_z: self.throttle * 4.0 - 1.0,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn throttle_revs_up_to_the_limiter_and_shifting_drops_revs() {
        let mut s = Sim::new();
        for _ in 0..600 {
            s.step(1.0 / 120.0, true);
        }
        let t = s.telemetry();
        assert!(t.current_engine_rpm > MAX_RPM - BOUNCE_RPM - 1.0);
        s.shift(2);
        assert!(s.telemetry().current_engine_rpm < t.current_engine_rpm);
        for _ in 0..600 {
            s.step(1.0 / 120.0, false);
        }
        assert!(s.telemetry().current_engine_rpm < 2000.0);
    }
}
