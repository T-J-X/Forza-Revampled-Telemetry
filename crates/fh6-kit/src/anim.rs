//! Animation: closed-form springs, frame-rate-independent smoothing, easing curves and timing
//! helpers. Everything advances from the frame's `dt` (never the wall clock), is exact at
//! 60/120/144 Hz and allocates nothing.
use crate::color::Color;
use crate::math::{finite_or, Vec2};
use std::f32::consts::TAU;

/// A spring described like the tokens: period `T` (seconds) and damping ratio `zeta`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpringSpec {
    /// Undamped period in seconds (`omega = 2 pi / T`).
    pub period: f32,
    /// Damping ratio: 1 = critically damped, below 1 overshoots.
    pub zeta: f32,
}

impl SpringSpec {
    /// A spring from period and damping ratio.
    #[must_use]
    pub const fn new(period: f32, zeta: f32) -> Self {
        Self { period, zeta }
    }

    /// A critically damped spring that settles in about `period`.
    #[must_use]
    pub const fn critical(period: f32) -> Self {
        Self { period, zeta: 1.0 }
    }

    fn omega(&self) -> f32 {
        TAU / finite_or(self.period, 0.1).max(1e-4)
    }
}

/// A damped spring on one value, advanced analytically (no integration error, any `dt`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spring {
    /// The spring's shape.
    pub spec: SpringSpec,
    value: f32,
    velocity: f32,
}

impl Spring {
    /// A spring resting at `value`.
    #[must_use]
    pub fn new(spec: SpringSpec, value: f32) -> Self {
        Self {
            spec,
            value: finite_or(value, 0.0),
            velocity: 0.0,
        }
    }

    /// Current value.
    #[must_use]
    pub fn value(&self) -> f32 {
        self.value
    }

    /// Current velocity (units per second).
    #[must_use]
    pub fn velocity(&self) -> f32 {
        self.velocity
    }

    /// Jumps to `value` at rest.
    pub fn reset(&mut self, value: f32) {
        self.value = finite_or(value, 0.0);
        self.velocity = 0.0;
    }

    /// True when within `eps` of `target` and nearly still.
    #[must_use]
    pub fn settled(&self, target: f32, eps: f32) -> bool {
        (self.value - target).abs() <= eps && self.velocity.abs() <= eps * 10.0
    }

    /// Advances `dt` seconds towards `target`; returns the new value.
    pub fn update(&mut self, target: f32, dt: f32) -> f32 {
        let target = finite_or(target, self.value);
        let dt = finite_or(dt, 0.0).clamp(0.0, 0.25);
        if dt <= 0.0 {
            return self.value;
        }
        let w = self.spec.omega();
        let z = finite_or(self.spec.zeta, 1.0).max(0.0);
        let x0 = self.value - target;
        let v0 = self.velocity;
        let (x, v) = if (z - 1.0).abs() < 1e-4 {
            let e = (-w * dt).exp();
            let c2 = v0 + w * x0;
            let x = (x0 + c2 * dt) * e;
            let v = (c2 - w * (x0 + c2 * dt)) * e;
            (x, v)
        } else if z < 1.0 {
            let wd = w * (1.0 - z * z).sqrt();
            let e = (-z * w * dt).exp();
            let (s, c) = (wd * dt).sin_cos();
            let c2 = (v0 + z * w * x0) / wd;
            let x = e * (x0 * c + c2 * s);
            let v = e * ((c2 * wd - z * w * x0) * c - (x0 * wd + z * w * c2) * s);
            (x, v)
        } else {
            let r = (z * z - 1.0).sqrt();
            let r1 = -w * (z - r);
            let r2 = -w * (z + r);
            let c2 = (v0 - r1 * x0) / (r2 - r1);
            let c1 = x0 - c2;
            let (e1, e2) = ((r1 * dt).exp(), (r2 * dt).exp());
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        };
        self.value = finite_or(target + x, target);
        self.velocity = finite_or(v, 0.0);
        self.value
    }

    /// Advances towards a target moving at `target_velocity` (units/s), with feed-forward so a
    /// steadily climbing needle does not lag (tokens: needle and ribbon springs).
    pub fn update_tracking(&mut self, target: f32, target_velocity: f32, dt: f32) -> f32 {
        let lead = 2.0 * finite_or(self.spec.zeta, 1.0) / self.spec.omega() * finite_or(target_velocity, 0.0);
        self.update(target + lead, dt)
    }
}

/// Values that can be smoothed.
pub trait Lerp: Copy {
    /// `self + (o - self) * t`.
    fn lerp_to(self, o: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp_to(self, o: Self, t: f32) -> Self {
        self + (o - self) * t
    }
}

impl Lerp for Vec2 {
    fn lerp_to(self, o: Self, t: f32) -> Self {
        self.lerp(o, t)
    }
}

impl Lerp for Color {
    fn lerp_to(self, o: Self, t: f32) -> Self {
        self.lerp(o, t)
    }
}

/// Exponential smoothing with a time constant, exact at any frame rate. Use it for noisy
/// telemetry inputs (G, slip, suspension) before they drive visuals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Smoothed<T: Lerp> {
    value: T,
    /// Time constant in seconds (63% of a step in `tau`).
    pub tau: f32,
    primed: bool,
}

impl<T: Lerp> Smoothed<T> {
    /// A smoother starting at `value` with time constant `tau` seconds.
    #[must_use]
    pub fn new(value: T, tau: f32) -> Self {
        Self {
            value,
            tau,
            primed: false,
        }
    }

    /// Current value.
    #[must_use]
    pub fn value(&self) -> T {
        self.value
    }

    /// Feeds a sample; the first sample is taken as-is.
    pub fn update(&mut self, sample: T, dt: f32) -> T {
        if !self.primed {
            self.value = sample;
            self.primed = true;
            return sample;
        }
        let tau = finite_or(self.tau, 0.0);
        let k = if tau <= 1e-6 {
            1.0
        } else {
            1.0 - (-finite_or(dt, 0.0).max(0.0) / tau).exp()
        };
        self.value = self.value.lerp_to(sample, k);
        self.value
    }

    /// Jumps to `value`.
    pub fn reset(&mut self, value: T) {
        self.value = value;
        self.primed = true;
    }
}

/// A displayed number that only changes when the input moves by at least `threshold`
/// (tokens: speed changes when `|v - shown| >= 0.6`, so the last digit never flickers).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Hysteresis {
    shown: f32,
    /// Minimum change.
    pub threshold: f32,
}

impl Hysteresis {
    /// Starts at `value`.
    #[must_use]
    pub fn new(value: f32, threshold: f32) -> Self {
        Self {
            shown: value.round(),
            threshold,
        }
    }

    /// Feeds a value; returns the integer to show.
    pub fn update(&mut self, value: f32) -> i64 {
        let value = finite_or(value, self.shown);
        if (value - self.shown).abs() >= self.threshold {
            self.shown = value.round();
        }
        self.shown as i64
    }
}

/// A CSS-style cubic Bezier easing baked into a 256-entry lookup table.
#[derive(Debug, Clone, PartialEq)]
pub struct Easing {
    lut: [f32; 256],
}

impl Easing {
    /// Bakes `cubic-bezier(x1, y1, x2, y2)`.
    #[must_use]
    pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let bez = |a: f32, b: f32, t: f32| {
            let u = 1.0 - t;
            3.0 * u * u * t * a + 3.0 * u * t * t * b + t * t * t
        };
        let mut lut = [0.0f32; 256];
        for (i, out) in lut.iter_mut().enumerate() {
            let x = i as f32 / 255.0;
            // Solve bez_x(t) = x by bisection (monotonic for x1, x2 in 0..1).
            let (mut lo, mut hi) = (0.0f32, 1.0f32);
            for _ in 0..30 {
                let mid = (lo + hi) * 0.5;
                if bez(x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0), mid) < x {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            *out = bez(y1, y2, (lo + hi) * 0.5);
        }
        Self { lut }
    }

    /// Linear.
    #[must_use]
    pub fn linear() -> Self {
        let mut lut = [0.0f32; 256];
        for (i, v) in lut.iter_mut().enumerate() {
            *v = i as f32 / 255.0;
        }
        Self { lut }
    }

    /// Parses `"cubic-bezier(a, b, c, d)"` or `"linear"`.
    #[must_use]
    pub fn parse(css: &str) -> Option<Self> {
        let s = css.trim();
        if s.eq_ignore_ascii_case("linear") {
            return Some(Self::linear());
        }
        let inner = s.strip_prefix("cubic-bezier(")?.strip_suffix(')')?;
        let mut v = [0.0f32; 4];
        let mut n = 0;
        for part in inner.split(',') {
            *v.get_mut(n)? = part.trim().parse().ok()?;
            n += 1;
        }
        (n == 4).then(|| Self::cubic_bezier(v[0], v[1], v[2], v[3]))
    }

    /// Eased value at `t` in 0..=1 (clamped), interpolating the table.
    #[must_use]
    pub fn eval(&self, t: f32) -> f32 {
        let x = finite_or(t, 0.0).clamp(0.0, 1.0) * 255.0;
        let i = x.floor() as usize;
        let f = x - i as f32;
        let a = self.lut[i.min(255)];
        let b = self.lut[(i + 1).min(255)];
        a + (b - a) * f
    }
}

/// A square-wave strobe (tokens: limiter 10 Hz, 50% duty; reduced-flash mode swaps it for a
/// 2 Hz sine breath).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Strobe {
    /// Frequency in Hz.
    pub hz: f32,
    /// Fraction of each period that is on.
    pub duty: f32,
}

impl Strobe {
    /// The limiter strobe.
    pub const LIMITER: Strobe = Strobe { hz: 10.0, duty: 0.5 };

    /// True when the strobe is lit at time `t` seconds.
    #[must_use]
    pub fn on(&self, t: f32) -> bool {
        let phase = (finite_or(t, 0.0) * self.hz).rem_euclid(1.0);
        phase < self.duty
    }

    /// Reduced-flash replacement: a 2 Hz breath between 0.35 and 1.
    #[must_use]
    pub fn breath(t: f32) -> f32 {
        0.675 + 0.325 * (finite_or(t, 0.0) * TAU * 2.0).sin()
    }
}

/// A one-shot timeline from 0 to 1 over `duration` seconds.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Tween {
    elapsed: f32,
    /// Length in seconds.
    pub duration: f32,
}

impl Tween {
    /// A tween of `duration` seconds, finished (call [`Tween::restart`] to run it).
    #[must_use]
    pub fn new(duration: f32) -> Self {
        Self {
            elapsed: duration,
            duration,
        }
    }

    /// Starts again from 0.
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }

    /// Advances and returns progress 0..=1.
    pub fn update(&mut self, dt: f32) -> f32 {
        self.elapsed = (self.elapsed + finite_or(dt, 0.0).max(0.0)).min(self.duration.max(0.0));
        self.progress()
    }

    /// Progress 0..=1.
    #[must_use]
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 {
            1.0
        } else {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        }
    }

    /// True while running.
    #[must_use]
    pub fn running(&self) -> bool {
        self.progress() < 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spring_is_frame_rate_independent() {
        let spec = SpringSpec::new(0.07, 0.75);
        let mut a = Spring::new(spec, 0.0);
        let mut b = Spring::new(spec, 0.0);
        for _ in 0..60 {
            a.update(1.0, 1.0 / 60.0);
        }
        for _ in 0..144 {
            b.update(1.0, 1.0 / 144.0);
        }
        assert!((a.value() - b.value()).abs() < 1e-4, "{} vs {}", a.value(), b.value());
    }

    #[test]
    fn spring_regimes_converge() {
        for zeta in [0.35, 1.0, 1.6] {
            let mut s = Spring::new(SpringSpec::new(0.2, zeta), 0.0);
            for _ in 0..240 {
                s.update(5.0, 1.0 / 120.0);
            }
            assert!(s.settled(5.0, 1e-3), "zeta {zeta}: {}", s.value());
        }
    }

    #[test]
    fn underdamped_overshoots_critical_does_not() {
        let mut under = Spring::new(SpringSpec::new(0.2, 0.35), 0.0);
        let mut crit = Spring::new(SpringSpec::critical(0.2), 0.0);
        let (mut max_u, mut max_c) = (0.0f32, 0.0f32);
        for _ in 0..240 {
            max_u = max_u.max(under.update(1.0, 1.0 / 120.0));
            max_c = max_c.max(crit.update(1.0, 1.0 / 120.0));
        }
        assert!(max_u > 1.05);
        assert!(max_c <= 1.0 + 1e-5);
    }

    #[test]
    fn feed_forward_removes_ramp_lag() {
        let spec = SpringSpec::new(0.07, 0.75);
        let mut plain = Spring::new(spec, 0.0);
        let mut ff = Spring::new(spec, 0.0);
        let rate = 6000.0; // rpm per second
        let dt = 1.0 / 120.0;
        let mut target = 0.0;
        for _ in 0..120 {
            target += rate * dt;
            plain.update(target, dt);
            ff.update_tracking(target, rate, dt);
        }
        let lag_plain = target - plain.value();
        let lag_ff = (target - ff.value()).abs();
        assert!(lag_ff < lag_plain * 0.5, "plain lag {lag_plain}, ff lag {lag_ff}");
        // Lag in time must be under one 60 Hz frame (tokens).
        assert!(lag_ff / rate < 1.0 / 60.0);
    }

    #[test]
    fn smoothing_and_hysteresis() {
        let mut s = Smoothed::new(0.0f32, 0.1);
        s.update(0.0, 0.0);
        let v = s.update(1.0, 0.1);
        assert!((v - (1.0 - (-1.0f32).exp())).abs() < 1e-5);
        let mut h = Hysteresis::new(100.0, 0.6);
        assert_eq!(h.update(100.4), 100);
        assert_eq!(h.update(100.7), 101);
    }

    #[test]
    fn easing_curves() {
        let e = Easing::parse("cubic-bezier(0.2, 0, 0, 1)").expect("parse");
        assert!(e.eval(0.0).abs() < 1e-4 && (e.eval(1.0) - 1.0).abs() < 1e-4);
        assert!(e.eval(0.5) > 0.5, "emphasized is front-loaded");
        assert!(Easing::parse("ease").is_none());
    }

    #[test]
    fn strobe_duty() {
        let s = Strobe::LIMITER;
        assert!(s.on(0.0) && s.on(0.049) && !s.on(0.051) && s.on(0.1));
    }
}
