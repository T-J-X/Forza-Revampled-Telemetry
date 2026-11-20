use fh6_core::config::sanitize_toggle_mask;

#[derive(Debug, Clone, Copy)]
pub struct HoldConfig {
    pub toggle_mask: u16,
    pub hold_ms: u64,
}

impl Default for HoldConfig {
    fn default() -> Self {
        Self {
            toggle_mask: 0x0020,
            hold_ms: 400,
        }
    }
}

#[derive(Debug, Default)]
pub struct OSDToggleHoldState {
    target_mask: u16,
    hold_ms: u64,
    hold_start: Option<u64>,
    armed: bool,
}

impl OSDToggleHoldState {
    const ABXY_MASK: u16 = 0xF000;

    pub fn new(config: HoldConfig) -> Self {
        let mask = sanitize_toggle_mask(config.toggle_mask);
        Self {
            target_mask: mask,
            hold_ms: config.hold_ms,
            hold_start: None,
            armed: true,
        }
    }

    fn sanitize_pressed(mask: u16) -> u16 {
        mask & !Self::ABXY_MASK
    }

    pub fn update(&mut self, pressed_mask: u16, now_ms: u64) -> Option<()> {
        let mask = Self::sanitize_pressed(pressed_mask);

        let full = (mask & self.target_mask) == self.target_mask;

        if full {
            if self.armed {
                if self.hold_start.is_none() {
                    self.hold_start = Some(now_ms);
                } else if let Some(start) = self.hold_start {
                    if now_ms.saturating_sub(start) >= self.hold_ms {
                        // fire once
                        self.hold_start = None;
                        self.armed = false;
                        return Some(());
                    }
                }
            }
            // still held, do not re-arm
        } else {
            // not fully pressed
            if self.hold_start.is_some() {
                // released before threshold -> tap, no event
                self.hold_start = None;
            }
            self.armed = true;
        }

        None
    }

    pub fn set_config(&mut self, config: HoldConfig) {
        self.target_mask = sanitize_toggle_mask(config.toggle_mask);
        self.hold_ms = config.hold_ms;
        self.hold_start = None;
        self.armed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hold_fires_at_threshold() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x0020, hold_ms: 400 });
        assert_eq!(s.update(0x0020, 0), None);
        assert_eq!(s.update(0x0020, 399), None);
        assert_eq!(s.update(0x0020, 400), Some(()));
    }

    #[test]
    fn tap_below_threshold_ignored() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x0020, hold_ms: 400 });
        s.update(0x0020, 0);
        s.update(0x0020, 200);
        // release
        s.update(0x0000, 300);
        // no event should have fired
        assert_eq!(s.update(0x0020, 400), None);
    }

    #[test]
    fn no_refire_while_held() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x0020, hold_ms: 400 });
        s.update(0x0020, 0);
        assert_eq!(s.update(0x0020, 500), Some(()));
        // still held
        assert_eq!(s.update(0x0020, 600), None);
    }

    #[test]
    fn refire_after_release() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x0020, hold_ms: 400 });
        s.update(0x0020, 0);
        assert_eq!(s.update(0x0020, 500), Some(()));
        // release
        s.update(0x0000, 600);
        s.update(0x0020, 700);
        assert_eq!(s.update(0x0020, 1100), Some(()));
    }

    #[test]
    fn partial_combo_does_not_fire() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x0030, hold_ms: 400 });
        s.update(0x0010, 0);
        assert_eq!(s.update(0x0010, 500), None);
    }

    #[test]
    fn zero_mask_falls_back_to_default() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0, hold_ms: 400 });
        // sanitize 0 -> 0x0020
        s.update(0, 0);
        assert_eq!(s.update(0, 500), None);
        s.update(0x0020, 600);
        assert_eq!(s.update(0x0020, 1000), Some(()));
    }

    #[test]
    fn abxy_bits_rejected_in_mask() {
        let mut s = OSDToggleHoldState::new(HoldConfig { toggle_mask: 0x1000, hold_ms: 400 });
        // sanitize strips ABXY, falls back to 0x0020
        s.update(0x1000, 0);
        // should not fire for 0x1000 alone
        assert_eq!(s.update(0x1000, 500), None);
    }
}
