//! View + Menu 350ms hold toggle handler, auto-pause ESC hook, and Live Dyno generator.
use fh6_core::telemetry::FH6TelemetryData;
use std::time::{Duration, Instant};

pub const VK_ESCAPE: u16 = 0x1B;
pub const MASK_VIEW_MENU: u16 = 0x0030; // 0x0020 (Back/View) | 0x0010 (Start/Menu)

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

pub struct OsdToggleHandler<K: KeyboardInjector> {
    pub hold_duration: Duration,
    pub view_menu_hold_start: Option<Instant>,
    pub view_menu_triggered: bool,
    pub keyboard_injector: K,
}

impl<K: KeyboardInjector> OsdToggleHandler<K> {
    pub fn new(injector: K) -> Self {
        Self {
            hold_duration: Duration::from_millis(350),
            view_menu_hold_start: None,
            view_menu_triggered: false,
            keyboard_injector: injector,
        }
    }

    /// Updates hold tracking. When View+Menu is held >= 350ms, toggles is_osd_open.
    /// If opening OSD and is_race_on == 1, sends VK_ESCAPE to auto-pause the game.
    pub fn update_hold(&mut self, raw_buttons: u16, is_race_on: i32, is_osd_open: &mut bool) -> bool {
        let is_view_menu = (raw_buttons & MASK_VIEW_MENU) == MASK_VIEW_MENU;
        if is_view_menu {
            if self.view_menu_hold_start.is_none() {
                self.view_menu_hold_start = Some(Instant::now());
            } else if !self.view_menu_triggered {
                if let Some(start) = self.view_menu_hold_start {
                    if start.elapsed() >= self.hold_duration {
                        self.view_menu_triggered = true;
                        *is_osd_open = !*is_osd_open;
                        // Auto-pause game iff opening OSD and race is currently active
                        if *is_osd_open && is_race_on == 1 {
                            self.keyboard_injector.send_key_vk(VK_ESCAPE);
                        }
                        return true;
                    }
                }
            }
        } else {
            self.view_menu_hold_start = None;
            self.view_menu_triggered = false;
        }
        false
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
    fn test_toggle_view_menu_hold_and_auto_pause() {
        let injector = MockKeyboardInjector::new();
        let mut handler = OsdToggleHandler::new(injector);
        handler.hold_duration = Duration::from_millis(50); // fast test

        let mut is_open = false;

        // Button not pressed
        handler.update_hold(0, 1, &mut is_open);
        assert!(!is_open);

        // View + Menu pressed
        handler.update_hold(MASK_VIEW_MENU, 1, &mut is_open);
        // Sleep past hold duration
        std::thread::sleep(Duration::from_millis(60));
        let triggered = handler.update_hold(MASK_VIEW_MENU, 1, &mut is_open);

        assert!(triggered);
        assert!(is_open, "OSD should open after 350ms hold");
        assert_eq!(
            handler.keyboard_injector.sent_keys,
            vec![VK_ESCAPE],
            "ESC should be sent on auto-pause"
        );

        // Release buttons
        handler.update_hold(0, 1, &mut is_open);

        // Toggle again when IsRaceOn == 0 (paused already)
        handler.update_hold(MASK_VIEW_MENU, 0, &mut is_open);
        std::thread::sleep(Duration::from_millis(60));
        let triggered_close = handler.update_hold(MASK_VIEW_MENU, 0, &mut is_open);

        assert!(triggered_close);
        assert!(!is_open, "OSD should close on second hold");
        // No second ESC because is_race_on == 0
        assert_eq!(handler.keyboard_injector.sent_keys.len(), 1);
    }

    #[test]
    fn test_live_dyno_generator() {
        let mut dyno = LiveDynoGenerator::new();
        let t1 = dyno.tick(0.016);
        assert!(t1.current_engine_rpm >= 950.0);
        assert!(t1.gear >= 1);
    }
}
