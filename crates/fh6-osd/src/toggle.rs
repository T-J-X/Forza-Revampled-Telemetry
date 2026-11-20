//! OSD open/close side effects (auto-pause ESC) and the live dyno generator.
//!
//! Hold detection itself lives in `fh6_input::hold_state::HoldToggle`, the single
//! implementation; the input thread calls [`OsdToggleHandler::apply_toggle`] when it fires.
use fh6_core::telemetry::FH6TelemetryData;

pub const VK_ESCAPE: u16 = 0x1B;

pub trait KeyboardInjector: Send + Sync {
    fn send_key_vk(&mut self, vk_code: u16);
}

#[derive(Default, Debug)]
pub struct MockKeyboardInjector {
    pub sent_keys: Vec<u16>,
}

impl MockKeyboardInjector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl KeyboardInjector for MockKeyboardInjector {
    fn send_key_vk(&mut self, vk_code: u16) {
        self.sent_keys.push(vk_code);
    }
}

/// Applies an OSD toggle: flips the open state and auto-pauses the game when opening mid-race.
pub struct OsdToggleHandler<K: KeyboardInjector> {
    pub keyboard_injector: K,
}

impl<K: KeyboardInjector> OsdToggleHandler<K> {
    pub fn new(injector: K) -> Self {
        Self {
            keyboard_injector: injector,
        }
    }

    /// Flips `is_osd_open`. Opening while `is_race_on == 1` sends ESC so the game pauses.
    pub fn apply_toggle(&mut self, is_race_on: i32, is_osd_open: &mut bool) {
        *is_osd_open = !*is_osd_open;
        if *is_osd_open && is_race_on == 1 {
            self.keyboard_injector.send_key_vk(VK_ESCAPE);
        }
    }
}

/// Generates live synthetic 60 FPS telemetry sweeps for dyno preview while paused.
#[derive(Debug)]
pub struct LiveDynoGenerator {
    pub phase: f32,
    pub max_rpm: f32,
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
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances simulation by dt seconds and returns synthetic telemetry.
    pub fn tick(&mut self, dt: f32) -> FH6TelemetryData {
        self.phase += dt * 0.8;
        if self.phase > 6.0 {
            self.phase -= 6.0;
        }

        let gear_idx = ((self.phase.floor() as i32) + 1).clamp(1, 6) as u8;
        let sub_phase = self.phase.fract(); // 0.0 .. 1.0 within the gear

        let rpm = self.idle_rpm + (self.max_rpm - self.idle_rpm) * (0.35 + 0.65 * sub_phase);
        let speed_mph = (gear_idx as f32) * 28.0 + sub_phase * 22.0;
        let speed_ms = speed_mph * 0.44704;

        let accel = if sub_phase > 0.95 { 0 } else { 255 };
        let brake = if sub_phase > 0.95 { 204 } else { 0 };
        let boost = if sub_phase > 0.3 { 16.5 * sub_phase } else { 2.0 };

        FH6TelemetryData {
            is_race_on: 0, // Paused in-game, but live in OSD
            engine_max_rpm: self.max_rpm,
            engine_idle_rpm: self.idle_rpm,
            current_engine_rpm: rpm,
            speed_ms,
            gear: gear_idx,
            accel,
            brake,
            boost_psi: boost,
            power_watts: 450_000.0 * (rpm / self.max_rpm),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_auto_pauses_only_when_racing() {
        let mut handler = OsdToggleHandler::new(MockKeyboardInjector::new());
        let mut is_open = false;

        handler.apply_toggle(1, &mut is_open);
        assert!(is_open, "OSD opens");
        assert_eq!(handler.keyboard_injector.sent_keys, vec![VK_ESCAPE], "ESC pauses the race");

        handler.apply_toggle(0, &mut is_open);
        assert!(!is_open, "OSD closes");
        handler.apply_toggle(0, &mut is_open);
        assert!(is_open);
        assert_eq!(handler.keyboard_injector.sent_keys.len(), 1, "no ESC when already paused");
    }

    #[test]
    fn test_live_dyno_generator() {
        let mut dyno = LiveDynoGenerator::new();
        let t1 = dyno.tick(0.016);
        assert!(t1.current_engine_rpm >= 950.0);
        assert!(t1.gear >= 1);
    }
}
