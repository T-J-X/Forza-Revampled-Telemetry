//! When the dash may be on screen, and how it fades.
//!
//! Rules (Agent 1, HMI/UX):
//! - Hard hide, complete on the very next frame: `IsRaceOn == 0` (menus, pause), a stale
//!   stream (no packet for 250 ms), or Forza not being the foreground window.
//! - Soft hide, a ~120 ms fade-out: only while still racing, when the user hides the overlay.
//! - Show: a ~120 ms fade-in.
//! - Live preview (the OSD live dyno) lifts the race-off and stale rules so the dash animates
//!   while the game is paused, but never the foreground rule.
//! - Bypass (`--demo`, `--bench-seconds`, for testing) keeps the dash visible regardless.
use std::time::Duration;

/// Default fade duration for showing, and for a user-requested hide while racing.
pub const FADE: Duration = Duration::from_millis(120);

/// What the visibility controller sees this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VisibilityInputs {
    /// The latest packet has `IsRaceOn == 1`.
    pub race_on: bool,
    /// A packet arrived within the stale window.
    pub stream_fresh: bool,
    /// The foreground window belongs to the game process.
    pub game_foreground: bool,
    /// The user hid the overlay (tray "Hide overlay").
    pub user_hidden: bool,
    /// The OSD live view is open: ignore the race-off and stale rules, keep the focus rule.
    pub live_preview: bool,
    /// Test modes (`--demo`, `--bench-seconds`): ignore every gate above.
    pub bypass: bool,
}

/// Why the dash is hidden, for logs and the stats line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HideReason {
    /// `IsRaceOn == 0`: menu, pause or loading.
    RaceOff,
    /// No packet within the stale window.
    Stale,
    /// Another window has the focus.
    NotForeground,
    /// Hidden from the tray.
    User,
}

/// The controller's verdict for one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Visibility {
    /// Opacity to draw with, 0..=1. Zero means present a fully transparent frame.
    pub opacity: f32,
    /// `None` while the dash should be shown.
    pub hidden_because: Option<HideReason>,
    /// True while a fade is in progress, so the loop keeps rendering every vsync.
    pub animating: bool,
}

/// Tracks opacity across frames.
#[derive(Debug, Clone)]
pub struct VisibilityController {
    opacity: f32,
    fade: Duration,
}

impl Default for VisibilityController {
    fn default() -> Self {
        Self::new(FADE)
    }
}

impl VisibilityController {
    /// A controller that starts hidden and fades over `fade`.
    pub fn new(fade: Duration) -> Self {
        Self { opacity: 0.0, fade }
    }

    /// Current opacity without advancing time.
    pub fn opacity(&self) -> f32 {
        self.opacity
    }

    /// Advances by `dt` and returns this frame's opacity.
    pub fn update(&mut self, inputs: VisibilityInputs, dt: Duration) -> Visibility {
        let hard_hide = if inputs.bypass {
            None
        } else if !inputs.game_foreground {
            Some(HideReason::NotForeground)
        } else if inputs.live_preview {
            None
        } else if !inputs.stream_fresh {
            Some(HideReason::Stale)
        } else if !inputs.race_on {
            Some(HideReason::RaceOff)
        } else {
            None
        };
        if let Some(reason) = hard_hide {
            self.opacity = 0.0;
            return Visibility {
                opacity: 0.0,
                hidden_because: Some(reason),
                animating: false,
            };
        }

        let hidden_because = (inputs.user_hidden && !inputs.bypass).then_some(HideReason::User);
        let target = if hidden_because.is_some() { 0.0 } else { 1.0 };
        let step = if self.fade.is_zero() {
            1.0
        } else {
            (dt.as_secs_f32() / self.fade.as_secs_f32()).clamp(0.0, 1.0)
        };
        self.opacity = if self.opacity < target {
            (self.opacity + step).min(target)
        } else {
            (self.opacity - step).max(target)
        };
        Visibility {
            opacity: self.opacity,
            hidden_because,
            animating: (self.opacity - target).abs() > f32::EPSILON,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Duration = Duration::from_micros(16_667);

    fn racing() -> VisibilityInputs {
        VisibilityInputs {
            race_on: true,
            stream_fresh: true,
            game_foreground: true,
            user_hidden: false,
            live_preview: false,
            bypass: false,
        }
    }

    fn fade_fully_in(ctl: &mut VisibilityController) {
        for _ in 0..20 {
            ctl.update(racing(), FRAME);
        }
        assert_eq!(ctl.opacity(), 1.0);
    }

    #[test]
    fn fades_in_over_about_120_ms() {
        let mut ctl = VisibilityController::default();
        let first = ctl.update(racing(), FRAME);
        assert!(first.opacity > 0.0 && first.opacity < 1.0);
        assert!(first.animating);
        let mut frames = 1;
        while ctl.update(racing(), FRAME).animating {
            frames += 1;
        }
        // 120 ms at 60 Hz is 7.2 frames.
        assert!((7..=9).contains(&frames), "took {frames} frames");
    }

    #[test]
    fn race_off_hides_on_the_next_frame() {
        let mut ctl = VisibilityController::default();
        fade_fully_in(&mut ctl);
        let v = ctl.update(
            VisibilityInputs {
                race_on: false,
                ..racing()
            },
            FRAME,
        );
        assert_eq!(v.opacity, 0.0);
        assert_eq!(v.hidden_because, Some(HideReason::RaceOff));
    }

    #[test]
    fn focus_loss_and_stale_hide_immediately() {
        for (inputs, reason) in [
            (
                VisibilityInputs {
                    game_foreground: false,
                    ..racing()
                },
                HideReason::NotForeground,
            ),
            (
                VisibilityInputs {
                    stream_fresh: false,
                    ..racing()
                },
                HideReason::Stale,
            ),
        ] {
            let mut ctl = VisibilityController::default();
            fade_fully_in(&mut ctl);
            let v = ctl.update(inputs, FRAME);
            assert_eq!(v.opacity, 0.0, "{reason:?}");
            assert_eq!(v.hidden_because, Some(reason));
        }
    }

    #[test]
    fn user_hide_fades_out_while_racing() {
        let mut ctl = VisibilityController::default();
        fade_fully_in(&mut ctl);
        let v = ctl.update(
            VisibilityInputs {
                user_hidden: true,
                ..racing()
            },
            FRAME,
        );
        assert!(v.opacity > 0.0 && v.opacity < 1.0, "soft fade, got {}", v.opacity);
        assert_eq!(v.hidden_because, Some(HideReason::User));
    }

    #[test]
    fn live_preview_lifts_race_and_stale_but_not_focus() {
        let mut ctl = VisibilityController::new(Duration::ZERO);
        let paused = VisibilityInputs {
            race_on: false,
            stream_fresh: false,
            game_foreground: true,
            live_preview: true,
            ..VisibilityInputs::default()
        };
        assert_eq!(ctl.update(paused, FRAME).opacity, 1.0, "OSD dyno animates while paused");
        let unfocused = VisibilityInputs {
            game_foreground: false,
            ..paused
        };
        let v = ctl.update(unfocused, FRAME);
        assert_eq!(v.opacity, 0.0, "never over other apps");
        assert_eq!(v.hidden_because, Some(HideReason::NotForeground));
    }

    #[test]
    fn bypass_ignores_every_gate() {
        let mut ctl = VisibilityController::new(Duration::ZERO);
        let v = ctl.update(
            VisibilityInputs {
                bypass: true,
                user_hidden: true,
                ..VisibilityInputs::default()
            },
            FRAME,
        );
        assert_eq!(v.opacity, 1.0);
        assert_eq!(v.hidden_because, None);
    }
}
