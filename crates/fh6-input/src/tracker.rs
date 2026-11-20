use crate::xinput::XInputState;
use std::time::{Duration, Instant};

pub const XINPUT_GAMEPAD_DPAD_UP: u16 = 0x0001;
pub const XINPUT_GAMEPAD_DPAD_DOWN: u16 = 0x0002;
pub const XINPUT_GAMEPAD_DPAD_LEFT: u16 = 0x0004;
pub const XINPUT_GAMEPAD_DPAD_RIGHT: u16 = 0x0008;
pub const XINPUT_GAMEPAD_START: u16 = 0x0010;
pub const XINPUT_GAMEPAD_BACK: u16 = 0x0020;
pub const XINPUT_GAMEPAD_LEFT_THUMB: u16 = 0x0040;
pub const XINPUT_GAMEPAD_RIGHT_THUMB: u16 = 0x0080;
pub const XINPUT_GAMEPAD_LEFT_SHOULDER: u16 = 0x0100;
pub const XINPUT_GAMEPAD_RIGHT_SHOULDER: u16 = 0x0200;
pub const XINPUT_GAMEPAD_A: u16 = 0x1000;
pub const XINPUT_GAMEPAD_B: u16 = 0x2000;
pub const XINPUT_GAMEPAD_X: u16 = 0x4000;
pub const XINPUT_GAMEPAD_Y: u16 = 0x8000;

pub const MASK_MANUAL_LOCK: u16 = 0xF000;
pub const MASK_DPAD: u16 = 0x000F;
pub const MASK_VIEW_MENU: u16 = 0x0030;
pub const MASK_LB_RB: u16 = 0x0300;
pub const MASK_L3_R3: u16 = 0x00C0;

pub trait PadSource {
    fn poll_slot(&mut self, slot: usize) -> Option<XInputState>;
}

pub struct InputControllerTracker<S: PadSource> {
    pub controller_mode: String,
    pub is_osd_open: bool,
    pub config_osd_auto_pause: bool,
    prev_buttons: [u16; 4],
    latest_raw_buttons: u16,
    last_rb_press: Option<Instant>,
    view_menu_hold_start: Option<Instant>,
    view_menu_triggered: bool,
    last_tab_switch: Option<Instant>,
    last_studio_toggle: Option<Instant>,
    l3r3_was_held: bool,
    pad_source: S,
}

impl<S: PadSource> InputControllerTracker<S> {
    pub fn new(mode: &str, pad_source: S) -> Self {
        Self {
            controller_mode: if mode.is_empty() {
                "RB".to_string()
            } else {
                mode.to_string()
            },
            is_osd_open: false,
            config_osd_auto_pause: false,
            prev_buttons: [0; 4],
            latest_raw_buttons: 0,
            last_rb_press: None,
            view_menu_hold_start: None,
            view_menu_triggered: false,
            last_tab_switch: None,
            last_studio_toggle: None,
            l3r3_was_held: false,
            pad_source,
        }
    }

    fn safe_buttons(raw: u16) -> u16 {
        raw & !MASK_MANUAL_LOCK
    }

    pub fn poll(
        &mut self,
        current_camera_index: &mut usize,
        is_driving_active: bool,
        time_since_driving_resumed: f64,
        demo_mode: bool,
        is_settings_focused: bool,
    ) {
        let base_guard = (is_driving_active && time_since_driving_resumed >= 150.0) || demo_mode;
        let controller_guard_passed = base_guard;

        let mut combined_raw = 0u16;
        for slot in 0..4 {
            if let Some(state) = self.pad_source.poll_slot(slot) {
                let raw = state.Gamepad.wButtons;
                combined_raw |= raw;
                let safe = Self::safe_buttons(raw);
                let prev_safe = Self::safe_buttons(self.prev_buttons[slot]);

                let is_l3r3 = (safe & MASK_L3_R3) == MASK_L3_R3;
                let prev_l3r3 = (prev_safe & MASK_L3_R3) == MASK_L3_R3;
                if is_l3r3 && !prev_l3r3 && !self.l3r3_was_held {
                    let elapsed_ok = self
                        .last_studio_toggle
                        .map(|t| t.elapsed() >= Duration::from_millis(400))
                        .unwrap_or(true);
                    if elapsed_ok {
                        self.l3r3_was_held = true;
                        self.last_studio_toggle = Some(Instant::now());
                    }
                } else if !is_l3r3 {
                    self.l3r3_was_held = false;
                }

                if !is_settings_focused {
                    let is_view_menu = (safe & MASK_VIEW_MENU) == MASK_VIEW_MENU;
                    if is_view_menu {
                        if self.view_menu_hold_start.is_none() {
                            self.view_menu_hold_start = Some(Instant::now());
                        } else if !self.view_menu_triggered {
                            if let Some(start) = self.view_menu_hold_start {
                                if start.elapsed() >= Duration::from_millis(350) {
                                    self.view_menu_triggered = true;
                                    self.is_osd_open = !self.is_osd_open;
                                }
                            }
                        }
                    } else {
                        self.view_menu_hold_start = None;
                        self.view_menu_triggered = false;
                    }

                    if self.is_osd_open {
                        let lb_down = (safe & XINPUT_GAMEPAD_LEFT_SHOULDER) != 0;
                        let prev_lb = (prev_safe & XINPUT_GAMEPAD_LEFT_SHOULDER) != 0;
                        if lb_down && !prev_lb {
                            let elapsed_ok = self
                                .last_tab_switch
                                .map(|t| t.elapsed() >= Duration::from_millis(150))
                                .unwrap_or(true);
                            if elapsed_ok {
                                self.last_tab_switch = Some(Instant::now());
                            }
                        }
                        let rb_down = (safe & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        let prev_rb = (prev_safe & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        if rb_down && !prev_rb {
                            let elapsed_ok = self
                                .last_tab_switch
                                .map(|t| t.elapsed() >= Duration::from_millis(150))
                                .unwrap_or(true);
                            if elapsed_ok {
                                self.last_tab_switch = Some(Instant::now());
                            }
                        }
                    } else {
                        let _is_view_down = (safe & XINPUT_GAMEPAD_BACK) != 0;
                        let is_rb_down = (safe & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        let _is_lb_down = (safe & XINPUT_GAMEPAD_LEFT_SHOULDER) != 0;

                        let prev_rb_down = (prev_safe & XINPUT_GAMEPAD_RIGHT_SHOULDER) != 0;
                        if is_rb_down && !prev_rb_down {
                            let elapsed = self
                                .last_rb_press
                                .map(|t| t.elapsed())
                                .unwrap_or(Duration::from_secs(1));
                            if elapsed >= Duration::from_millis(150) && controller_guard_passed {
                                *current_camera_index = (*current_camera_index + 1) % 6;
                                self.last_rb_press = Some(Instant::now());
                            }
                        }
                    }
                }
                self.prev_buttons[slot] = raw;
            }
        }
        self.latest_raw_buttons = combined_raw;
    }

    pub fn get_raw_buttons(&self) -> u16 {
        self.latest_raw_buttons
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakePad {
        states: Vec<XInputState>,
        idx: usize,
    }

    impl FakePad {
        fn new(states: Vec<XInputState>) -> Self {
            Self { states, idx: 0 }
        }
    }

    impl PadSource for FakePad {
        fn poll_slot(&mut self, _slot: usize) -> Option<XInputState> {
            let s = self.states.get(self.idx).copied();
            if self.idx < self.states.len() {
                self.idx += 1;
            }
            s
        }
    }

    #[test]
    fn rb_rising_edge_camera_ring() {
        let mut tracker = InputControllerTracker::new(
            "RB",
            FakePad::new(vec![
                XInputState {
                    Gamepad: crate::xinput::XInputGamepad {
                        wButtons: 0,
                        ..Default::default()
                    },
                },
                XInputState {
                    Gamepad: crate::xinput::XInputGamepad {
                        wButtons: XINPUT_GAMEPAD_RIGHT_SHOULDER,
                        ..Default::default()
                    },
                },
            ]),
        );
        let mut cam = 0usize;
        tracker.poll(&mut cam, true, 200.0, false, false);
        tracker.poll(&mut cam, true, 200.0, false, false);
        assert_eq!(cam, 1);
    }

    #[test]
    fn manual_lock_masks_never_used() {
        let raw =
            XINPUT_GAMEPAD_A | XINPUT_GAMEPAD_B | XINPUT_GAMEPAD_X | XINPUT_GAMEPAD_Y | XINPUT_GAMEPAD_RIGHT_SHOULDER;
        let safe = InputControllerTracker::<FakePad>::safe_buttons(raw);
        assert_eq!(safe & MASK_MANUAL_LOCK, 0);
        assert_ne!(safe & XINPUT_GAMEPAD_RIGHT_SHOULDER, 0);
    }

    #[test]
    fn test_tracker_bonnet_autoselect() {
        use fh6_core::config::CameraViewMode;
        let idx = 4usize;
        let view = CameraViewMode::from_index(idx as i32);
        assert_eq!(view, CameraViewMode::Bonnet);
    }

    #[test]
    fn test_rb_closed_cycles_view() {
        let mut tracker = InputControllerTracker::new(
            "RB",
            FakePad::new(vec![
                XInputState { Gamepad: crate::xinput::XInputGamepad { wButtons: 0, ..Default::default() } },
                XInputState { Gamepad: crate::xinput::XInputGamepad { wButtons: XINPUT_GAMEPAD_RIGHT_SHOULDER, ..Default::default() } },
            ]),
        );
        let mut cam = 3usize;
        tracker.poll(&mut cam, true, 200.0, false, false);
        tracker.poll(&mut cam, true, 200.0, false, false);
        assert_eq!(cam, 4);
    }

    #[test]
    fn test_rb_open_switches_tab_only() {
        let mut tracker = InputControllerTracker::new(
            "RB",
            FakePad::new(vec![
                XInputState { Gamepad: crate::xinput::XInputGamepad { wButtons: XINPUT_GAMEPAD_RIGHT_SHOULDER, ..Default::default() } },
            ]),
        );
        // Simulate OSD open
        tracker.is_osd_open = true;
        let mut cam = 3usize;
        tracker.poll(&mut cam, true, 200.0, false, false);
        // Camera should NOT change when OSD is open
        assert_eq!(cam, 3);
    }
}
