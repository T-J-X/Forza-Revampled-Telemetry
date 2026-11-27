//! Values that hold still on the limiter, as a car's instruments do.
//!
//! - [`Readout`]: a digital readout takes a value at a few hertz and only when it has moved more
//!   than 0.6 of its last digit, so a reading that wobbles every frame (power and boost bouncing
//!   off the limiter, G settling after a lift) never flickers a digit.
//! - [`LimiterHold`]: the rev counter's needle holds the peak of a bounce off the limiter, as a
//!   damped needle does, instead of buzzing with it; a gear change (the cluster calls
//!   [`LimiterHold::release`]), a drop of the band or more, or a lift out of the band releases it
//!   at once.

/// How often a readout may take a new value (Hz). Five keeps a step visible within 0.2 s and
/// changes a digit at most once in the twelve frames of a limiter hold.
const REFRESH_HZ: f32 = 5.0;
/// A new value is taken only when it is this many steps away from the shown one (the same rule
/// as the tabular speed).
const HYSTERESIS: f32 = 0.6;
/// A drop of less than this (rpm) near the limiter is a bounce off it; a bigger drop releases the
/// hold at once. A limiter at or below this counts as none known.
pub(super) const LIMITER_BAND: f32 = 200.0;
/// Once no new peak has come for [`BOUNCE_HOLD_S`], the held peak sinks this fast (rpm/s)
/// towards the rpm, so a held needle never lies for long.
const PIN_DECAY: f32 = 400.0;
/// How long a bounce keeps the peak exactly still and the needle's feed-forward off (s): a
/// limiter bounce re-peaks every frame or two.
const BOUNCE_HOLD_S: f32 = 0.1;

/// What the needle is fed: the rpm, or the peak of a bounce off the limiter.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct LimiterHold {
    pin: f32,
    bounce: f32,
    /// Time since the rpm last reached the held peak (s).
    since_peak: f32,
}

impl LimiterHold {
    /// The needle's target rpm and target velocity (rpm/s) this frame, for `rpm` moving at
    /// `rate` with the limiter at `limiter`. Within [`LIMITER_BAND`] of the limiter a drop
    /// smaller than the band holds the last peak exactly while the bounce goes on (sinking
    /// slowly once it stops) with the feed-forward off.
    pub fn feed(&mut self, rpm: f32, rate: f32, limiter: f32, dt: f32) -> (f32, f32) {
        let rpm = if rpm.is_finite() { rpm } else { 0.0 };
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        // No limiter known (a menu, or a car that sends none): nothing is held.
        let near = limiter.is_finite() && limiter > LIMITER_BAND && rpm >= limiter - LIMITER_BAND;
        let drop = self.pin - rpm;
        if near && drop > 0.0 && drop < LIMITER_BAND {
            self.since_peak += dt;
            if self.since_peak > BOUNCE_HOLD_S {
                self.pin = (self.pin - PIN_DECAY * dt).max(rpm);
            }
            self.bounce = BOUNCE_HOLD_S;
        } else {
            // An upshift, or the rpm leaving the band (a lift): the hold ends at once, so the
            // needle is never snapped onto a falling rpm.
            if drop >= LIMITER_BAND || !near {
                self.bounce = 0.0;
            }
            self.pin = rpm;
            self.since_peak = 0.0;
            self.bounce = (self.bounce - dt).max(0.0);
        }
        let rate = if self.bounce > 0.0 || !rate.is_finite() {
            0.0
        } else {
            rate
        };
        (self.pin, rate)
    }

    /// True while a bounce off the limiter is being held.
    pub fn holding(&self) -> bool {
        self.bounce > 0.0
    }

    /// Ends any hold at once (a gear change: the rpm that follows is a new gear's, never a
    /// bounce), so the next frame feeds the rpm as it is.
    pub fn release(&mut self) {
        *self = Self::default();
    }
}

/// One readout: the value on show, in steps of its last digit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Readout {
    shown: f32,
    since: f32,
    primed: bool,
}

impl Readout {
    /// A readout with nothing on show yet (the first sample is taken as it is).
    pub const fn new() -> Self {
        Self {
            shown: 0.0,
            since: 0.0,
            primed: false,
        }
    }

    /// The value on show.
    pub fn value(&self) -> f32 {
        self.shown
    }

    /// Feeds a sample whose last shown digit is `step` (1 for whole horsepower, 0.1 for psi to
    /// one decimal); returns the value on show. A sample that is not finite is ignored.
    pub fn update(&mut self, sample: f32, step: f32, dt: f32) -> f32 {
        if !sample.is_finite() {
            return self.shown;
        }
        let step = if step.is_finite() && step > 0.0 { step } else { 1.0 };
        if !self.primed {
            self.shown = quantise(sample, step);
            self.primed = true;
            self.since = 0.0;
            return self.shown;
        }
        self.since += if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        if self.since >= 1.0 / REFRESH_HZ {
            self.since = 0.0;
            if (sample - self.shown).abs() >= HYSTERESIS * step {
                self.shown = quantise(sample, step);
            }
        }
        self.shown
    }
}

/// `v` rounded to a whole number of `step`s.
fn quantise(v: f32, step: f32) -> f32 {
    (v / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn the_first_sample_shows_at_once() {
        let mut r = Readout::new();
        assert_eq!(r.update(414.6, 1.0, DT), 415.0);
    }

    #[test]
    fn a_value_bouncing_every_frame_never_changes_a_digit() {
        let mut r = Readout::new();
        r.update(414.0, 1.0, DT);
        let mut changes = 0;
        let mut last = r.value();
        for i in 0..120 {
            // Both samples round to 415 but stay within 0.6 of the 414 on show: only the
            // hysteresis keeps the digit still.
            let v = if i % 2 == 0 { 414.52 } else { 414.58 };
            let shown = r.update(v, 1.0, DT);
            if shown != last {
                changes += 1;
                last = shown;
            }
        }
        assert_eq!(changes, 0, "a 0.5 hp wobble moved the digits");
    }

    #[test]
    fn a_step_shows_within_one_refresh_and_digits_change_at_most_five_times_a_second() {
        let mut r = Readout::new();
        r.update(100.0, 1.0, DT);
        let mut changes = 0;
        let mut last = r.value();
        let mut frames_to_show = None;
        for i in 0..60 {
            // A climb of 3 hp per frame.
            let shown = r.update(100.0 + 3.0 * (i + 1) as f32, 1.0, DT);
            if shown != last {
                changes += 1;
                last = shown;
                frames_to_show.get_or_insert(i + 1);
            }
        }
        assert!(changes <= 5, "{changes} changes in one second");
        assert!(frames_to_show.is_some_and(|f| f <= 13), "{frames_to_show:?}");
    }

    #[test]
    fn a_bounce_off_the_limiter_holds_the_needle_still_and_an_upshift_releases_it() {
        let mut h = LimiterHold::default();
        // The pull up to the limiter: the rpm and its rate pass straight through.
        assert_eq!(h.feed(8000.0, 3000.0, 8700.0, DT), (8000.0, 3000.0));
        assert_eq!(h.feed(8680.0, 3000.0, 8700.0, DT), (8680.0, 3000.0));
        // The review sequence's bounce: 8,690 and 8,640 on alternate frames.
        let mut targets = Vec::new();
        for i in 0..12 {
            let (rpm, rate) = if i % 2 == 0 {
                (8690.0, 3000.0)
            } else {
                (8640.0, -3000.0)
            };
            let (t, r) = h.feed(rpm, rate, 8700.0, DT);
            // The first frame still climbs; from the first drop on, the feed-forward stays off.
            if i > 0 {
                assert_eq!(r, 0.0, "frame {i}: feed-forward on during a bounce");
                assert!(h.holding(), "frame {i}: the bounce is not held");
            }
            targets.push(t);
        }
        // While the bounce goes on the needle is held exactly at its peak.
        assert!(
            targets.iter().all(|t| *t == 8690.0),
            "the held needle moved: {targets:?}"
        );
        // The upshift: a drop of more than the band releases the needle at once.
        let (t, r) = h.feed(7510.0, -60_000.0, 8700.0, DT);
        assert_eq!((t, r), (7510.0, -60_000.0));
    }

    #[test]
    fn a_held_peak_sinks_once_the_bounce_stops() {
        let mut h = LimiterHold::default();
        h.feed(8690.0, 0.0, 8700.0, DT);
        // The rpm settles 60 below the peak with no new peak: held for 0.1 s, then sinking.
        let mut last = 8690.0;
        for i in 0..30 {
            let (t, _) = h.feed(8630.0, 0.0, 8700.0, DT);
            assert!(t <= last && t >= 8630.0, "frame {i}: {t}");
            last = t;
        }
        assert!(last < 8690.0 - 50.0, "still held at {last} after half a second");
    }

    #[test]
    fn a_gear_change_ends_the_hold_at_once() {
        let mut h = LimiterHold::default();
        h.feed(8690.0, 0.0, 8700.0, DT);
        h.feed(8640.0, -3000.0, 8700.0, DT);
        assert!(h.holding());
        // The review sequence's upshift frame: 8,600 rpm is a drop of only 90, but the gear has
        // changed, so the cluster releases the hold and the needle takes the new gear's rpm.
        h.release();
        assert!(!h.holding());
        assert_eq!(h.feed(8600.0, -6000.0, 8700.0, DT), (8600.0, -6000.0));
    }

    #[test]
    fn a_lift_out_of_the_band_ends_the_hold_at_once() {
        let mut h = LimiterHold::default();
        h.feed(8690.0, 0.0, 8700.0, DT);
        h.feed(8640.0, -3000.0, 8700.0, DT);
        assert!(h.holding());
        // A lift to 8,495: a drop under the band, but out of it.
        let (t, r) = h.feed(8495.0, -6000.0, 8700.0, DT);
        assert_eq!((t, r), (8495.0, -6000.0));
        assert!(!h.holding(), "the hold outlived the lift");
    }

    #[test]
    fn far_from_the_limiter_nothing_is_held() {
        let mut h = LimiterHold::default();
        h.feed(5000.0, 0.0, 8700.0, DT);
        assert_eq!(h.feed(4900.0, -500.0, 8700.0, DT), (4900.0, -500.0));
        assert_eq!(h.feed(f32::NAN, f32::NAN, f32::NAN, DT), (0.0, 0.0));
        // With no limiter known, an idle wobble is shown as it is.
        let mut idle = LimiterHold::default();
        idle.feed(900.0, 0.0, 0.0, DT);
        assert_eq!(idle.feed(860.0, -2400.0, 0.0, DT), (860.0, -2400.0));
    }

    #[test]
    fn steps_and_bad_samples_are_safe() {
        let mut r = Readout::new();
        assert_eq!(r.update(1.234, 0.1, DT), 1.2);
        assert_eq!(r.update(f32::NAN, 0.1, DT), 1.2);
        assert_eq!(r.update(5.0, f32::NAN, 1.0), 5.0);
    }
}
