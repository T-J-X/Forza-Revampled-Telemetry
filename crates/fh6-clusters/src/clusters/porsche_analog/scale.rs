//! The rev counter's scale: Porsche's 240° face, either classic (0 at 8 o'clock, the middle
//! thousand at 12) or turned so that the shift rpm stands at 12 o'clock (the 992.2's
//! rev-counter mode, `RESEARCH.md` §10).
//!
//! Angles are radians, clockwise from 3 o'clock (the kit's convention).
use core::f32::consts::{FRAC_PI_2, PI, TAU};

/// Sweep of the scale from 0 rpm to its top: Porsche's 992.1 face runs 0 to 8 over about 240°
/// (30° per 1,000 rpm) and the GT3's 0 to 10 over the same arc (`RESEARCH.md` §4).
pub(super) const SWEEP: f32 = 240.0 * PI / 180.0;
/// The classic face starts at 8 o'clock (150° clockwise from 3 o'clock), so it is symmetric
/// about 12.
pub(super) const CLASSIC_START: f32 = 150.0 * PI / 180.0;
/// The lowest and highest top a scale may have (rpm): a corrupt limiter never builds thousands
/// of ticks.
const TOP_MIN: f32 = 3_000.0;
const TOP_MAX: f32 = 20_000.0;

/// How the scale is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum Mode {
    /// The 992.2 rev-counter mode: the shift rpm at 12 o'clock.
    #[default]
    Track,
    /// The 992.1 face: 0 at 8 o'clock, the top at 4 o'clock.
    Classic,
}

/// One car's scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DialScale {
    /// The top of the scale (rpm): the whole thousand above the limiter.
    pub top: f32,
    /// Angle of 0 rpm.
    pub start: f32,
}

impl DialScale {
    /// The scale of a car with limiter `max_rpm` and shift point `shift_rpm` in `mode`.
    pub fn new(mode: Mode, max_rpm: f32, shift_rpm: f32) -> Self {
        let top = top_for(max_rpm);
        let start = match mode {
            Mode::Classic => CLASSIC_START,
            Mode::Track => {
                let shift = if shift_rpm.is_finite() {
                    shift_rpm.clamp(0.0, top)
                } else {
                    top
                };
                -FRAC_PI_2 - SWEEP * shift / top
            }
        };
        Self { top, start }
    }

    /// The same scale turned to start at `start` (the animated rotation).
    pub fn turned(self, start: f32) -> Self {
        Self { start, ..self }
    }

    /// Radians per rpm.
    pub fn per_rpm(&self) -> f32 {
        SWEEP / self.top
    }

    /// The needle's angle at `rpm` (held a little past the top so it never wraps round).
    pub fn angle(&self, rpm: f32) -> f32 {
        let r = if rpm.is_finite() {
            rpm.clamp(0.0, self.top * 1.04)
        } else {
            0.0
        };
        self.start + r * self.per_rpm()
    }

    /// Angle of the top of the scale.
    pub fn end(&self) -> f32 {
        self.start + SWEEP
    }

    /// Direction of the middle of the numeral-free sector the needle never enters.
    pub fn dead_centre(&self) -> f32 {
        self.end() + (TAU - SWEEP) * 0.5
    }

    /// Number of whole thousands on the face (the last numeral).
    pub fn thousands(&self) -> u32 {
        (self.top / 1000.0).round().clamp(0.0, 20.0) as u32
    }
}

/// The red band runs at least this far past the limiter (rpm), so it reads as a band and not a
/// hook.
const RED_MIN: f32 = 500.0;

/// The top of the scale for a limiter: the first whole thousand at least [`RED_MIN`] above it
/// (9,000 → 10,000 as on the GT3; 7,500 → 8,000 as on the Carrera; 8,700 → 10,000).
pub(super) fn top_for(max_rpm: f32) -> f32 {
    if !max_rpm.is_finite() || max_rpm <= 0.0 {
        return TOP_MIN;
    }
    (((max_rpm + RED_MIN) / 1000.0).ceil() * 1000.0).clamp(TOP_MIN, TOP_MAX)
}

/// The shortest signed turn from angle `a` to angle `b` (radians, in −π..π).
pub(super) fn shortest_turn(a: f32, b: f32) -> f32 {
    let d = (b - a + PI).rem_euclid(TAU) - PI;
    if d.is_finite() {
        d
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        shortest_turn(a, b).abs() < 1e-4
    }

    #[test]
    fn the_top_leaves_a_red_band_of_at_least_500_rpm() {
        assert_eq!(top_for(8700.0), 10_000.0);
        assert_eq!(top_for(9000.0), 10_000.0);
        assert_eq!(top_for(7500.0), 8000.0);
        assert_eq!(top_for(6400.0), 7000.0);
        for max in [3000.0, 5999.0, 6501.0, 8499.0, 8501.0, 12_345.0] {
            assert!(top_for(max) - max >= RED_MIN, "{max}");
        }
        assert_eq!(top_for(f32::NAN), TOP_MIN);
        assert_eq!(top_for(-5.0), TOP_MIN);
        assert_eq!(top_for(1.0e9), TOP_MAX);
    }

    #[test]
    fn track_mode_puts_the_shift_rpm_at_twelve() {
        for (max, shift) in [(8700.0, 8439.0), (9000.0, 8730.0), (7500.0, 7275.0), (6000.0, 5820.0)] {
            let s = DialScale::new(Mode::Track, max, shift);
            assert!(close(s.angle(shift), -FRAC_PI_2), "{max}: {}", s.angle(shift));
            // The scale still runs 240° and the needle reaches its top before the dead zone.
            assert!((s.end() - s.start - SWEEP).abs() < 1e-5);
        }
    }

    #[test]
    fn the_classic_face_matches_the_992_dial() {
        // An 8,000 rpm face: 0 near 8 o'clock, 4 at 12, 8 near 4 o'clock, 30° per 1,000.
        let s = DialScale::new(Mode::Classic, 7500.0, 7275.0);
        assert_eq!(s.thousands(), 8);
        assert!(close(s.angle(0.0), 150f32.to_radians()));
        assert!(close(s.angle(4000.0), -FRAC_PI_2));
        assert!(close(s.angle(8000.0), 30f32.to_radians()));
        assert!((s.per_rpm() * 1000.0 - 30f32.to_radians()).abs() < 1e-6);
        // The dead zone is centred at 6 o'clock.
        assert!(close(s.dead_centre(), FRAC_PI_2));
    }

    #[test]
    fn the_needle_never_enters_the_dead_zone() {
        for mode in [Mode::Track, Mode::Classic] {
            for max in [4000.0, 6500.0, 8700.0, 9000.0, 12_000.0, 19_999.0] {
                let s = DialScale::new(mode, max, max * 0.97);
                let dead = s.dead_centre();
                // The whole scale, then what the host can send past it (rpm up to 1.1 of the
                // limiter, and a needle's feed-forward a little below zero): the clamp holds the
                // needle 4 % past the top and at 0.
                let past = [s.top * 1.05, s.top * 1.2, max * 1.1, -500.0];
                let samples = (0..=200).map(|i| s.top * 1.04 * i as f32 / 200.0).chain(past);
                for rpm in samples {
                    let off = shortest_turn(dead, s.angle(rpm)).abs();
                    // Half the dead zone is 60°; the 4% overrun eats under 10° of it.
                    assert!(off > 50f32.to_radians(), "{mode:?} {max} rpm {rpm}: {off}");
                }
            }
        }
    }

    #[test]
    fn non_finite_inputs_stay_on_the_face() {
        let s = DialScale::new(Mode::Track, f32::INFINITY, f32::NAN);
        assert!(s.start.is_finite() && s.top.is_finite());
        assert!(s.angle(f32::NAN).is_finite());
        assert_eq!(shortest_turn(f32::NAN, 1.0), 0.0);
    }
}
