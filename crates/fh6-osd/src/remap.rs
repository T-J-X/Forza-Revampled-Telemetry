//! The OSD button remapper: hold the new button or combination for one second to bind it.
//! A, B, X and Y and the D-pad are ignored (the gearbox and the menu's own navigation); LB or
//! RB alone is refused because they also switch tabs and drive the camera ring. D-pad left
//! cancels at once, and the time limit restarts whenever the held buttons change.
use fh6_core::config::{sanitize_toggle_mask, MASK_MANUAL_LOCK};
use std::time::Duration;

/// How long the new combination must be held.
pub const COMMIT_HOLD: Duration = Duration::from_millis(1000);
/// Capture gives up after this long with no change in the held buttons and no commit.
pub const TIMEOUT: Duration = Duration::from_secs(30);
/// D-pad bits (menu navigation; never part of the toggle).
pub const MASK_DPAD: u16 = 0x000F;
/// D-pad left: cancels the capture.
pub const DPAD_LEFT: u16 = crate::nav::LEFT;
/// LB.
pub const LB: u16 = 0x0100;
/// RB.
pub const RB: u16 = 0x0200;

/// Buttons the remapper listens to.
#[must_use]
pub fn capturable(buttons: u16) -> u16 {
    buttons & !MASK_MANUAL_LOCK & !MASK_DPAD
}

/// Why a combination is refused, if it is.
#[must_use]
pub fn refusal(mask: u16) -> Option<&'static str> {
    match mask {
        0 => Some("Hold a button"),
        LB | RB => Some("Pair LB or RB with another button"),
        _ => None,
    }
}

/// How a capture ended (or that it continues).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemapOutcome {
    /// Still listening.
    Pending,
    /// Bound: the sanitised mask to save.
    Committed(u16),
    /// D-pad left cancelled it.
    Cancelled,
    /// Nothing bindable was held for [`COMMIT_HOLD`] within [`TIMEOUT`] of the last change.
    TimedOut,
}

/// A running capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RemapCapture {
    started: Duration,
    candidate: u16,
    stable_since: Duration,
    /// Shown under the row (refusals, progress).
    pub message: Option<&'static str>,
}

impl RemapCapture {
    /// Starts listening at `now`.
    #[must_use]
    pub fn start(now: Duration) -> Self {
        Self {
            started: now,
            candidate: 0,
            stable_since: now,
            message: None,
        }
    }

    /// The combination currently held (capturable bits).
    #[must_use]
    pub fn candidate(&self) -> u16 {
        self.candidate
    }

    /// Progress of the current hold, 0..1.
    #[must_use]
    pub fn progress(&self, now: Duration) -> f32 {
        if self.candidate == 0 || refusal(self.candidate).is_some() {
            return 0.0;
        }
        (now.saturating_sub(self.stable_since).as_secs_f32() / COMMIT_HOLD.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Seconds left before the capture times out (it restarts when the held buttons change).
    #[must_use]
    pub fn remaining(&self, now: Duration) -> f32 {
        TIMEOUT.saturating_sub(now.saturating_sub(self.started)).as_secs_f32()
    }

    /// Feeds the held buttons at `now`.
    pub fn update(&mut self, held: u16, now: Duration) -> RemapOutcome {
        if held & DPAD_LEFT != 0 {
            return RemapOutcome::Cancelled;
        }
        let mask = capturable(held);
        if mask != self.candidate {
            self.candidate = mask;
            self.stable_since = now;
            self.started = now;
            self.message = if mask == 0 { None } else { refusal(mask) };
        }
        if mask != 0 && refusal(mask).is_none() && now.saturating_sub(self.stable_since) >= COMMIT_HOLD {
            return RemapOutcome::Committed(sanitize_toggle_mask(mask));
        }
        if now.saturating_sub(self.started) >= TIMEOUT {
            return RemapOutcome::TimedOut;
        }
        RemapOutcome::Pending
    }
}

/// Writes a mask's button names into `out` (`"View + L3"`).
pub fn mask_label(mask: u16, out: &mut crate::model::Line) {
    use std::fmt::Write as _;
    const NAMES: [(u16, &str); 8] = [
        (0x0020, "View"),
        (0x0010, "Menu"),
        (0x0040, "L3"),
        (0x0080, "R3"),
        (LB, "LB"),
        (RB, "RB"),
        (0x0400, "Guide"),
        (MASK_DPAD, "D-pad"),
    ];
    let mut first = true;
    for (bit, name) in NAMES {
        if mask & bit != 0 {
            let _ = write!(out, "{}{name}", if first { "" } else { " + " });
            first = false;
        }
    }
    if first {
        let _ = out.write_str("None");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    #[test]
    fn commits_after_a_steady_hold() {
        let mut cap = RemapCapture::start(MS(0));
        assert_eq!(cap.update(0x0040 | 0x0080, MS(100)), RemapOutcome::Pending);
        assert_eq!(cap.update(0x0040 | 0x0080, MS(900)), RemapOutcome::Pending);
        assert_eq!(cap.update(0x0040 | 0x0080, MS(1100)), RemapOutcome::Committed(0x00C0));
    }

    #[test]
    fn abxy_and_dpad_are_never_captured() {
        let mut cap = RemapCapture::start(MS(0));
        // A, B, X, Y and D-pad up, down and right (left cancels).
        for t in (0..3000).step_by(100) {
            assert_eq!(cap.update(0xF00B, MS(t)), RemapOutcome::Pending, "{t}");
        }
        assert_eq!(cap.candidate(), 0);
        // ABXY held together with View: only View is bound.
        let mut cap = RemapCapture::start(MS(0));
        cap.update(0xF020, MS(0));
        assert_eq!(cap.update(0xF020, MS(1000)), RemapOutcome::Committed(0x0020));
    }

    #[test]
    fn lone_bumper_is_refused_and_the_capture_times_out() {
        let mut cap = RemapCapture::start(MS(0));
        cap.update(RB, MS(0));
        assert_eq!(cap.update(RB, MS(5000)), RemapOutcome::Pending);
        assert_eq!(cap.message, Some("Pair LB or RB with another button"));
        assert_eq!(cap.update(RB, MS(29_900)), RemapOutcome::Pending);
        assert_eq!(cap.update(RB, MS(30_000)), RemapOutcome::TimedOut);
        let mut cap = RemapCapture::start(MS(0));
        cap.update(RB | 0x0020, MS(0));
        assert_eq!(cap.update(RB | 0x0020, MS(1000)), RemapOutcome::Committed(0x0220));
    }

    #[test]
    fn dpad_left_cancels_and_a_change_restarts_the_limit() {
        let mut cap = RemapCapture::start(MS(0));
        assert_eq!(cap.update(0x0040, MS(25_000)), RemapOutcome::Pending);
        assert!(
            (cap.remaining(MS(25_000)) - 30.0).abs() < 1e-3,
            "restarted on the change"
        );
        cap.update(0, MS(25_500));
        assert_eq!(
            cap.update(0, MS(50_000)),
            RemapOutcome::Pending,
            "30 s from the last change"
        );
        assert_eq!(cap.update(DPAD_LEFT, MS(50_100)), RemapOutcome::Cancelled);
    }

    #[test]
    fn labels_name_the_buttons() {
        let mut line = crate::model::Line::new();
        mask_label(0x0060, &mut line);
        assert_eq!(line.as_str(), "View + L3");
    }
}
