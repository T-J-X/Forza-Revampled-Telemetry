//! The one hold-to-toggle state machine (OSD open/close), shared by every caller.
//!
//! Default: hold View (`0x0020`) for 400 ms. The mask and time come from config
//! (`OsdToggleMask`, `OsdHoldMs`). A/B/X/Y are stripped from both the configured mask and the
//! pressed buttons, so the gearbox buttons can never open the OSD. A tap shorter than the hold
//! time does nothing (the game gets it). A hold fires once and re-arms on release.
use fh6_core::config::{default_osd_hold_ms, default_osd_toggle_mask, sanitize_toggle_mask, MASK_MANUAL_LOCK};
use fh6_core::AppConfig;

/// Which buttons to hold, and for how long.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HoldConfig {
    /// Buttons that must all be held. A/B/X/Y bits are ignored.
    pub toggle_mask: u16,
    /// Hold time in milliseconds.
    pub hold_ms: u64,
}

impl Default for HoldConfig {
    fn default() -> Self {
        Self {
            toggle_mask: default_osd_toggle_mask(),
            hold_ms: u64::from(default_osd_hold_ms()),
        }
    }
}

impl HoldConfig {
    /// The OSD toggle settings from config.
    pub fn from_config(cfg: &AppConfig) -> Self {
        Self {
            toggle_mask: cfg.osd_toggle_mask,
            hold_ms: u64::from(cfg.osd_hold_ms),
        }
    }
}

/// Hold-to-toggle detector. Feed it the pressed buttons and a monotonic time in ms.
#[derive(Debug, Clone)]
pub struct HoldToggle {
    target_mask: u16,
    hold_ms: u64,
    hold_start: Option<u64>,
    armed: bool,
}

impl Default for HoldToggle {
    fn default() -> Self {
        Self::new(HoldConfig::default())
    }
}

impl HoldToggle {
    /// A detector for `config`, with A/B/X/Y stripped from its mask.
    pub fn new(config: HoldConfig) -> Self {
        Self {
            target_mask: sanitize_toggle_mask(config.toggle_mask),
            hold_ms: config.hold_ms,
            hold_start: None,
            armed: true,
        }
    }

    /// The effective (sanitised) mask.
    pub fn mask(&self) -> u16 {
        self.target_mask
    }

    /// Returns true exactly once per hold, when the mask has been held for `hold_ms`.
    pub fn update(&mut self, pressed_mask: u16, now_ms: u64) -> bool {
        let pressed = pressed_mask & !MASK_MANUAL_LOCK;
        let full = (pressed & self.target_mask) == self.target_mask;
        if !full {
            // Released (or never fully pressed): a short tap fires nothing.
            self.hold_start = None;
            self.armed = true;
            return false;
        }
        if !self.armed {
            return false;
        }
        let start = *self.hold_start.get_or_insert(now_ms);
        if now_ms.saturating_sub(start) >= self.hold_ms {
            self.hold_start = None;
            self.armed = false;
            return true;
        }
        false
    }

    /// Applies new settings (for example after a config hot reload). A new mask needs a full
    /// release before it can fire, so a button still held across the change never re-fires.
    pub fn set_config(&mut self, config: HoldConfig) {
        let mask = sanitize_toggle_mask(config.toggle_mask);
        self.hold_ms = config.hold_ms;
        if mask != self.target_mask {
            self.target_mask = mask;
            self.hold_start = None;
            self.armed = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view_400() -> HoldToggle {
        HoldToggle::new(HoldConfig {
            toggle_mask: 0x0020,
            hold_ms: 400,
        })
    }

    #[test]
    fn hold_fires_at_threshold() {
        let mut s = view_400();
        assert!(!s.update(0x0020, 0));
        assert!(!s.update(0x0020, 399));
        assert!(s.update(0x0020, 400));
    }

    #[test]
    fn tap_below_threshold_ignored() {
        let mut s = view_400();
        s.update(0x0020, 0);
        s.update(0x0020, 200);
        s.update(0x0000, 300);
        assert!(!s.update(0x0020, 400), "the timer restarted on the new press");
    }

    #[test]
    fn no_refire_while_held() {
        let mut s = view_400();
        s.update(0x0020, 0);
        assert!(s.update(0x0020, 500));
        assert!(!s.update(0x0020, 600));
        assert!(!s.update(0x0020, 5000));
    }

    #[test]
    fn refire_after_release() {
        let mut s = view_400();
        s.update(0x0020, 0);
        assert!(s.update(0x0020, 500));
        s.update(0x0000, 600);
        s.update(0x0020, 700);
        assert!(s.update(0x0020, 1100));
    }

    #[test]
    fn partial_combo_does_not_fire() {
        let mut s = HoldToggle::new(HoldConfig {
            toggle_mask: 0x0030,
            hold_ms: 400,
        });
        s.update(0x0010, 0);
        assert!(!s.update(0x0010, 500));
    }

    #[test]
    fn zero_mask_falls_back_to_view() {
        let mut s = HoldToggle::new(HoldConfig {
            toggle_mask: 0,
            hold_ms: 400,
        });
        assert_eq!(s.mask(), 0x0020);
        s.update(0x0020, 600);
        assert!(s.update(0x0020, 1000));
    }

    #[test]
    fn abxy_never_fires() {
        let mut s = HoldToggle::new(HoldConfig {
            toggle_mask: 0x1000,
            hold_ms: 400,
        });
        assert_eq!(s.mask(), 0x0020, "A stripped from the mask");
        s.update(0xF000, 0);
        assert!(!s.update(0xF000, 5000), "holding A/B/X/Y alone never fires");
    }

    #[test]
    fn config_change_while_held_does_not_refire() {
        let mut s = view_400();
        s.update(0x0020, 0);
        assert!(s.update(0x0020, 400));
        s.set_config(HoldConfig {
            toggle_mask: 0x0020,
            hold_ms: 300,
        });
        assert!(!s.update(0x0020, 2000), "same mask, still held: no second fire");
        s.set_config(HoldConfig {
            toggle_mask: 0x0010 | 0x0020,
            hold_ms: 300,
        });
        assert!(!s.update(0x0030, 3000), "new mask needs a release first");
        s.update(0, 3100);
        s.update(0x0030, 3200);
        assert!(s.update(0x0030, 3500));
    }

    #[test]
    fn config_values_are_used() {
        let cfg = AppConfig {
            osd_toggle_mask: 0x0100 | 0x1000,
            osd_hold_ms: 250,
            ..AppConfig::default()
        };
        let hc = HoldConfig::from_config(&cfg);
        let mut s = HoldToggle::new(hc);
        assert_eq!(s.mask(), 0x0100);
        s.update(0x0100, 0);
        assert!(s.update(0x0100, 250));
    }
}
