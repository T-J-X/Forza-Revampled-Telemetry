//! Auto-pause: opening the OSD mid-race taps ESC so the game pauses under the menu, and
//! closing it taps ESC again to resume, but only if this OSD paused the game and it is still
//! paused. Key injection sits behind [`KeyboardInjector`] so tests use a mock; the real
//! `SendInput` implementation lives in `fh6-win`.
use std::time::Duration;

/// Virtual-key code of Escape.
pub const VK_ESCAPE: u16 = 0x1B;

/// How long after closing to wait before resuming: Menu is also Forza's own pause button, so
/// a Menu press that closes the OSD may already have resumed the game by itself.
pub const RESUME_DELAY: Duration = Duration::from_millis(250);

/// Sends a key press and release to the foreground window.
pub trait KeyboardInjector: Send {
    /// Taps `vk` (down, then up). Returns false when the system rejected the input.
    fn tap_key(&mut self, vk: u16) -> bool;
}

/// An injector that never sends anything (no-op hosts, previews).
#[derive(Debug, Default, Clone, Copy)]
pub struct NullInjector;

impl KeyboardInjector for NullInjector {
    fn tap_key(&mut self, _vk: u16) -> bool {
        false
    }
}

/// Records taps instead of sending them.
#[derive(Debug, Default, Clone)]
pub struct MockInjector {
    /// Every key tapped, in order.
    pub taps: Vec<u16>,
}

impl KeyboardInjector for MockInjector {
    fn tap_key(&mut self, vk: u16) -> bool {
        self.taps.push(vk);
        true
    }
}

/// What auto-pause needs to know about the game at an open or close edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PauseInputs {
    /// Racing: a fresh stream with `IsRaceOn == 1`.
    pub racing: bool,
    /// The game has the foreground (ESC would go to it).
    pub game_foreground: bool,
    /// `OsdAutoPauseGame`.
    pub auto_pause: bool,
}

/// Remembers whether this OSD paused the game.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AutoPause {
    paused_by_us: bool,
    resume_at: Option<Duration>,
}

impl AutoPause {
    /// A fresh state (nothing paused).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// True when this OSD paused the game and has not resumed it yet.
    #[must_use]
    pub fn paused_by_us(&self) -> bool {
        self.paused_by_us
    }

    /// The OSD opened at `now`. Taps ESC only while racing in the foreground with auto-pause
    /// on; never in menus or when already paused. Returns true when ESC was sent.
    pub fn on_open(&mut self, inj: &mut dyn KeyboardInjector, inputs: PauseInputs) -> bool {
        self.resume_at = None;
        if self.paused_by_us {
            // Reopened before the delayed resume: the game is still paused by us.
            return false;
        }
        if inputs.auto_pause && inputs.racing && inputs.game_foreground && inj.tap_key(VK_ESCAPE) {
            self.paused_by_us = true;
            return true;
        }
        false
    }

    /// The OSD closed at `now`: schedules the resume check (see [`RESUME_DELAY`]).
    pub fn on_close(&mut self, now: Duration) {
        if self.paused_by_us {
            self.resume_at = Some(now + RESUME_DELAY);
        }
    }

    /// Call every frame. When a resume is due it taps ESC if the game is still paused (not
    /// racing) and in the foreground. Returns true when ESC was sent.
    pub fn poll(&mut self, inj: &mut dyn KeyboardInjector, now: Duration, inputs: PauseInputs) -> bool {
        let Some(at) = self.resume_at else {
            return false;
        };
        if inputs.racing {
            // The player resumed the game themselves (Menu is Forza's pause button too).
            self.resume_at = None;
            self.paused_by_us = false;
            return false;
        }
        if now < at {
            return false;
        }
        self.resume_at = None;
        self.paused_by_us = false;
        inputs.game_foreground && inj.tap_key(VK_ESCAPE)
    }

    /// True while a resume is scheduled.
    #[must_use]
    pub fn resume_pending(&self) -> bool {
        self.resume_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RACING: PauseInputs = PauseInputs {
        racing: true,
        game_foreground: true,
        auto_pause: true,
    };
    const PAUSED: PauseInputs = PauseInputs {
        racing: false,
        game_foreground: true,
        auto_pause: true,
    };
    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn pauses_while_racing_and_resumes_after_close() {
        let mut inj = MockInjector::default();
        let mut ap = AutoPause::new();
        assert!(ap.on_open(&mut inj, RACING));
        assert_eq!(inj.taps, vec![VK_ESCAPE]);
        ap.on_close(MS(1000));
        assert!(!ap.poll(&mut inj, MS(1100), PAUSED), "waits for the resume delay");
        assert!(ap.poll(&mut inj, MS(1260), PAUSED));
        assert_eq!(inj.taps, vec![VK_ESCAPE, VK_ESCAPE]);
        assert!(!ap.paused_by_us());
    }

    #[test]
    fn never_sends_esc_in_menus_or_with_auto_pause_off() {
        let mut inj = MockInjector::default();
        let mut ap = AutoPause::new();
        assert!(!ap.on_open(&mut inj, PAUSED));
        ap.on_close(MS(10));
        assert!(!ap.poll(&mut inj, MS(1000), PAUSED));
        let off = PauseInputs {
            auto_pause: false,
            ..RACING
        };
        assert!(!ap.on_open(&mut inj, off));
        let background = PauseInputs {
            game_foreground: false,
            ..RACING
        };
        assert!(!ap.on_open(&mut inj, background));
        assert!(inj.taps.is_empty());
    }

    #[test]
    fn no_resume_when_the_player_already_resumed() {
        let mut inj = MockInjector::default();
        let mut ap = AutoPause::new();
        ap.on_open(&mut inj, RACING);
        ap.on_close(MS(0));
        assert!(!ap.poll(&mut inj, MS(100), RACING));
        assert!(!ap.poll(&mut inj, MS(1000), PAUSED));
        assert_eq!(inj.taps.len(), 1, "only the pause");
    }

    #[test]
    fn reopening_before_the_resume_keeps_the_pause() {
        let mut inj = MockInjector::default();
        let mut ap = AutoPause::new();
        ap.on_open(&mut inj, RACING);
        ap.on_close(MS(0));
        assert!(!ap.on_open(&mut inj, PAUSED));
        assert!(!ap.resume_pending());
        assert!(ap.paused_by_us());
        assert_eq!(inj.taps.len(), 1);
    }

    #[test]
    fn null_injector_reports_failure() {
        let mut ap = AutoPause::new();
        assert!(!ap.on_open(&mut NullInjector, RACING));
        assert!(!ap.paused_by_us());
    }
}
