//! The physics behind Kinetic Liquid Metal: its row of "Real geometry and maths" in
//! `program/phases/07-cluster-factory/CATALOG.md`, with every source in `RESEARCH.md`.
//!
//! Pure functions and small state machines, no drawing and no GPU, so each value the catalog
//! row names is asserted by a unit test here:
//!
//! - **Hydrostatics of a free surface.** In a container that accelerates sideways by `a_lat`
//!   and upwards by `a_vert`, the free surface is a plane normal to the effective gravity, tilted
//!   by tan θ = a_lat ÷ (g + a_vert) ([`surface_tilt`]). 1.0 g tilts it 45.00°, 0.5 g 26.57°.
//! - **A U-tube column.** A liquid column of length `l` swinging in a U-tube obeys
//!   x″ + 2ζω₀x′ + ω₀²x = −a_t with ω₀ = √(2g ÷ l) ([`UTube`]). Its steady state is the tilt
//!   above, so the pool's surface reaches the hydrostatic tilt through the column's own
//!   under-damped swing. The tokens' `fluidSlosh` period of 0.62 s is a column of l = 0.191 m
//!   ([`column_length`]).
//! - **Volume.** The ampoule is a sphere; the mercury below a plane at distance `d` from its
//!   centre has volume π h² (3r − h) ÷ 3 with h = r − d ([`cap_volume`]). That volume does not
//!   depend on the plane's direction, so a tilting surface keeps its distance from the centre
//!   and only turns ([`Pool`]).
//! - **The meniscus.** Mercury meets glass at a contact angle of about 140°, so its surface in
//!   a bore of radius a is a convex spherical cap of half-angle φ = θc − 90°, radius
//!   a ÷ sin φ = a ÷ |cos θc| and height a (1 − cos φ) ÷ sin φ ≈ 0.466 a ([`meniscus_height`]).
//! - **Rayleigh–Plateau dripping.** A thread of radius R is unstable to wavelengths longer than
//!   its circumference; the fastest mode has kR = 0.697, so it breaks into drops λ = 9.01 R
//!   apart, each holding one wavelength of thread: radius (¾ R² λ)^(1/3) = 1.89 R
//!   ([`drop_radius`]).
//! - **Mercury's reflectance.** A metal reflects by the conductor Fresnel equations with a
//!   complex index n + ik; for liquid mercury at 546 nm n = 1.49, k = 4.55, so it reflects 78%
//!   at normal incidence ([`conductor_reflectance`], `docs/specs/optics.md` §9b).
use core::f32::consts::{PI, TAU};

/// Standard gravity (m/s²).
pub const STANDARD_G: f32 = 9.806_65;
/// Effective gravity never drops under this share of g: a crest or a jump can leave the car
/// nearly weightless, and the surface's tilt is then undefined (the liquid floats).
pub const MIN_EFFECTIVE_G: f32 = 0.1;
/// The fastest-growing Rayleigh–Plateau mode of an inviscid thread, kR (MIT 18.357 lecture 11,
/// eq. 11.32: λ ≈ 9.02 R).
pub const RAYLEIGH_PLATEAU_KR: f32 = 0.697;
/// Contact angle of mercury on clean glass (degrees), measured through the liquid.
pub const MERCURY_CONTACT_DEG: f32 = 140.0;
/// Liquid mercury at 546 nm: ε = −18.46 + 13.50 i (Smith and Stromberg 1966, quoted by Arakawa,
/// Inagaki and Williams), so n + ik = √ε.
pub const MERCURY_EPS: (f32, f32) = (-18.46, 13.50);

/// The tilt of a free surface (radians, positive when it rises on the left of the screen) in a
/// container accelerating `a_lat_g` to the right and `a_vert_g` upwards, both in g:
/// tan θ = a_lat ÷ (g + a_vert). The effective gravity is held at [`MIN_EFFECTIVE_G`] or more.
#[must_use]
pub fn surface_tilt(a_lat_g: f32, a_vert_g: f32) -> f32 {
    let lat = finite(a_lat_g);
    let vert = (1.0 + finite(a_vert_g)).max(MIN_EFFECTIVE_G);
    lat.atan2(vert)
}

/// The length (m) of a U-tube column whose undamped period is `period` s at gravity `g`:
/// ω₀ = √(2g ÷ l), so l = 2g ÷ ω₀² = g T² ÷ 2π².
#[must_use]
pub fn column_length(period: f32, g: f32) -> f32 {
    let w = TAU / period.max(1e-6);
    2.0 * g / (w * w)
}

/// The undamped angular frequency of a U-tube column of length `l` m under gravity `g`.
#[must_use]
pub fn column_omega(l: f32, g: f32) -> f32 {
    (2.0 * g.max(0.0) / l.max(1e-6)).sqrt()
}

/// The free surface's slope as a U-tube column: x″ + 2ζω₀x′ + ω₀²x = −a_t.
///
/// The state is the surface slope s = tan θ, which is −2x ÷ w for a U-tube of width `w` whose
/// limbs stand at ±x: the forcing −a_t of the column becomes ω₀² · a_lat ÷ (g + a_vert), so the
/// steady slope is the hydrostatic one ([`surface_tilt`]). ω₀ follows the effective gravity
/// (√(2 g_eff ÷ l)). Each step is the exact solution over `dt` with the forcing held, so the
/// motion is the same at 60, 120 or 144 Hz.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UTube {
    /// Column length (m).
    pub length: f32,
    /// Damping ratio ζ.
    pub zeta: f32,
    slope: f32,
    rate: f32,
}

impl UTube {
    /// A column of `length` m with damping `zeta`, at rest and level.
    #[must_use]
    pub fn new(length: f32, zeta: f32) -> Self {
        Self {
            length: length.max(1e-4),
            zeta: zeta.clamp(0.0, 0.999),
            slope: 0.0,
            rate: 0.0,
        }
    }

    /// The column whose period at 1 g is `period` s (the tokens' `fluidSlosh` spring).
    #[must_use]
    pub fn from_period(period: f32, zeta: f32) -> Self {
        Self::new(column_length(period, STANDARD_G), zeta)
    }

    /// The surface slope tan θ now.
    #[must_use]
    pub fn slope(&self) -> f32 {
        self.slope
    }

    /// The surface tilt θ now (radians, positive when it rises on the left).
    #[must_use]
    pub fn tilt(&self) -> f32 {
        self.slope.atan()
    }

    /// The slope's rate of change (1/s).
    #[must_use]
    pub fn rate(&self) -> f32 {
        self.rate
    }

    /// Jumps to `slope`, still.
    pub fn reset(&mut self, slope: f32) {
        self.slope = finite(slope);
        self.rate = 0.0;
    }

    /// Adds `impulse` to the slope's rate: a knock that sets the surface swinging.
    pub fn kick(&mut self, impulse: f32) {
        self.rate += finite(impulse);
    }

    /// The undamped angular frequency at `a_vert_g` (g upwards).
    #[must_use]
    pub fn omega(&self, a_vert_g: f32) -> f32 {
        let g_eff = (1.0 + finite(a_vert_g)).max(MIN_EFFECTIVE_G) * STANDARD_G;
        column_omega(self.length, g_eff)
    }

    /// Advances `dt` s with the car accelerating `a_lat_g` right and `a_vert_g` up.
    pub fn step(&mut self, a_lat_g: f32, a_vert_g: f32, dt: f32) {
        let dt = finite(dt).clamp(0.0, 0.25);
        if dt <= 0.0 {
            return;
        }
        let target = surface_tilt(a_lat_g, a_vert_g).tan();
        let w = self.omega(a_vert_g);
        let z = self.zeta;
        let wd = w * (1.0 - z * z).sqrt();
        let x0 = self.slope - target;
        let v0 = self.rate;
        let e = (-z * w * dt).exp();
        let (s, c) = (wd * dt).sin_cos();
        let c2 = (v0 + z * w * x0) / wd.max(1e-6);
        let x = e * (x0 * c + c2 * s);
        let v = e * ((c2 * wd - z * w * x0) * c - (x0 * wd + z * w * c2) * s);
        self.slope = finite(target + x);
        self.rate = finite(v);
    }

    /// True when within `eps` of the hydrostatic slope for this acceleration and nearly still.
    #[must_use]
    pub fn settled(&self, a_lat_g: f32, a_vert_g: f32, eps: f32) -> bool {
        let target = surface_tilt(a_lat_g, a_vert_g).tan();
        (self.slope - target).abs() <= eps && self.rate.abs() <= eps * 10.0
    }
}

/// The volume of a sphere of radius `r` below a plane at signed distance `d` from its centre
/// (`d` > 0 below the centre): π h² (3r − h) ÷ 3 with h = r − d, clamped to the sphere.
#[must_use]
pub fn cap_volume(r: f32, d: f32) -> f32 {
    let r = r.max(0.0);
    let h = (r - d).clamp(0.0, 2.0 * r);
    PI * h * h * (3.0 * r - h) / 3.0
}

/// The plane distance `d` whose cap holds `volume` in a sphere of radius `r` (the inverse of
/// [`cap_volume`], by bisection to a hundredth of a pixel; 24 halvings of the diameter).
#[must_use]
pub fn cap_distance(r: f32, volume: f32) -> f32 {
    let r = r.max(1e-3);
    let v = finite(volume).clamp(0.0, 4.0 / 3.0 * PI * r * r * r);
    // cap_volume falls as d rises from −r to r.
    let (mut lo, mut hi) = (-r, r);
    for _ in 0..24 {
        let mid = 0.5 * (lo + hi);
        if cap_volume(r, mid) > v {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// The height of a convex mercury meniscus in a bore of radius `a`, for contact angle
/// `contact_deg` (over 90°): the surface is a spherical cap of half-angle φ = θc − 90° (flat at
/// 90°, a hemisphere at 180°), so its sphere has radius a ÷ sin φ and it rises
/// a (1 − cos φ) ÷ sin φ above its rim.
#[must_use]
pub fn meniscus_height(a: f32, contact_deg: f32) -> f32 {
    let phi = (contact_deg - 90.0).clamp(0.0, 90.0).to_radians();
    if phi < 1e-4 {
        return 0.0;
    }
    a.max(0.0) * (1.0 - phi.cos()) / phi.sin()
}

/// The radius of the cap's sphere for the same meniscus: a ÷ |cos θc|.
#[must_use]
pub fn meniscus_radius(a: f32, contact_deg: f32) -> f32 {
    let c = contact_deg.to_radians().cos().abs().max(1e-3);
    a.max(0.0) / c
}

/// The fastest-growing Rayleigh–Plateau wavelength of a thread of radius `r`: 2π r ÷ 0.697.
#[must_use]
pub fn drop_spacing(r: f32) -> f32 {
    TAU * r.max(0.0) / RAYLEIGH_PLATEAU_KR
}

/// The radius of the drop one wavelength of thread of radius `r` makes:
/// (4/3) π r_d³ = π r² λ, so r_d = (¾ r² λ)^(1/3) = 1.89 r.
#[must_use]
pub fn drop_radius(r: f32) -> f32 {
    (0.75 * r * r * drop_spacing(r)).cbrt()
}

/// The volume of a drop of radius `r`.
#[must_use]
pub fn drop_volume(r: f32) -> f32 {
    4.0 / 3.0 * PI * r * r * r
}

/// The complex index n + ik of a metal with permittivity ε = ε₁ + iε₂: (n + ik)² = ε.
#[must_use]
pub fn index_from_permittivity(eps1: f32, eps2: f32) -> (f32, f32) {
    let m = eps1.hypot(eps2);
    (((m + eps1) * 0.5).max(0.0).sqrt(), ((m - eps1) * 0.5).max(0.0).sqrt())
}

/// Unpolarised reflectance of a metal of complex index n + ik, from air, at incidence cosine
/// `cos_i` (the exact conductor Fresnel equations; `docs/specs/optics.md` §9b). At normal
/// incidence it is ((n − 1)² + k²) ÷ ((n + 1)² + k²).
#[must_use]
pub fn conductor_reflectance(cos_i: f32, n: f32, k: f32) -> f32 {
    let c = cos_i.clamp(0.0, 1.0);
    let c2 = c * c;
    let s2 = 1.0 - c2;
    let n2k2 = n * n - k * k - s2;
    let a2b2 = (n2k2 * n2k2 + 4.0 * n * n * k * k).sqrt();
    let a = ((a2b2 + n2k2) * 0.5).max(0.0).sqrt();
    let rs = (a2b2 + c2 - 2.0 * a * c) / (a2b2 + c2 + 2.0 * a * c).max(1e-9);
    let t = c2 * a2b2 + s2 * s2;
    let u = 2.0 * a * c * s2;
    let rp = rs * (t - u) / (t + u).max(1e-9);
    (0.5 * (rs + rp)).clamp(0.0, 1.0)
}

/// Liquid mercury's reflectance at incidence cosine `cos_i` (546 nm).
#[must_use]
pub fn mercury_reflectance(cos_i: f32) -> f32 {
    let (n, k) = index_from_permittivity(MERCURY_EPS.0, MERCURY_EPS.1);
    conductor_reflectance(cos_i, n, k)
}

/// The spirit-level bubble's offset along a vial bent to radius `vial_radius` px: the vial's
/// lowest point follows the effective gravity, so the bubble (the highest point) sits
/// `vial_radius` × θ from the middle, on the side the car accelerates towards.
#[must_use]
pub fn bubble_offset(vial_radius: f32, tilt: f32) -> f32 {
    vial_radius * finite(tilt)
}

/// Mercury on the limiter: drops pinch off the column tip, run down the capillary and join the
/// pool; after the shift the extra mercury drains back over `drain_s`. The volumes are kept in
/// one ledger (reservoir, drops in flight, pool), so mercury is neither made nor lost
/// ([`Drips::total`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Drips {
    /// Thread radius at the tip (px).
    pub thread: f32,
    /// Drops a second while the limiter is held.
    pub rate_hz: f32,
    /// Seconds a drop takes down the capillary.
    pub travel_s: f32,
    /// Seconds the pool takes to drain back after the limiter.
    pub drain_s: f32,
    /// Most extra volume the pool takes (px³); no drop forms once it is full.
    pub capacity: f32,
    /// Mercury still in the column's reservoir that can drip (px³).
    reservoir: f32,
    /// Extra mercury in the pool (px³).
    pool: f32,
    /// Progress (0..1) of each drop in flight down the capillary; negative is an empty slot.
    flight: [f32; MAX_DROPS],
    /// Time since the last drop pinched off.
    since: f32,
    /// Volume a second flowing back once the limiter ends.
    drain_rate: f32,
}

/// Drops that can be in flight at once (5 Hz over a 0.6 s run is 3).
pub const MAX_DROPS: usize = 6;

impl Drips {
    /// A ledger with `capacity` px³ of mercury in reserve, none in flight and none in the pool.
    #[must_use]
    pub fn new(thread: f32, rate_hz: f32, travel_s: f32, drain_s: f32, capacity: f32) -> Self {
        Self {
            thread: thread.max(0.5),
            rate_hz: rate_hz.max(0.1),
            travel_s: travel_s.max(0.01),
            drain_s: drain_s.max(0.01),
            capacity: capacity.max(0.0),
            reservoir: capacity.max(0.0),
            pool: 0.0,
            flight: [-1.0; MAX_DROPS],
            since: 0.0,
            drain_rate: 0.0,
        }
    }

    /// The volume of one drop.
    #[must_use]
    pub fn drop_volume(&self) -> f32 {
        drop_volume(drop_radius(self.thread))
    }

    /// Mercury in reserve, in flight and in the pool: constant.
    #[must_use]
    pub fn total(&self) -> f32 {
        self.reservoir + self.in_flight() + self.pool
    }

    /// Mercury in flight.
    #[must_use]
    pub fn in_flight(&self) -> f32 {
        self.flight.iter().filter(|f| **f >= 0.0).count() as f32 * self.drop_volume()
    }

    /// Extra mercury in the pool (px³).
    #[must_use]
    pub fn pool(&self) -> f32 {
        self.pool
    }

    /// Each drop in flight as its progress down the capillary (0 at the tip, 1 at the pool).
    pub fn drops(&self) -> impl Iterator<Item = f32> + '_ {
        self.flight.iter().copied().filter(|f| *f >= 0.0)
    }

    /// How far the next drop has pinched (0 just after one fell, 1 at the pinch).
    #[must_use]
    pub fn neck(&self) -> f32 {
        (self.since * self.rate_hz).clamp(0.0, 1.0)
    }

    /// True while a drop is in flight or the pool still holds extra mercury.
    #[must_use]
    pub fn busy(&self) -> bool {
        self.pool > 0.0 || self.flight.iter().any(|f| *f >= 0.0)
    }

    /// Advances `dt` s; `limiter` is true while the engine sits on the limiter.
    pub fn step(&mut self, limiter: bool, dt: f32) {
        let dt = finite(dt).clamp(0.0, 0.25);
        let v = self.drop_volume();
        // Drops in flight move on; those that reach the pool merge into it.
        for f in &mut self.flight {
            if *f >= 0.0 {
                *f += dt / self.travel_s;
                if *f >= 1.0 {
                    *f = -1.0;
                    self.pool += v;
                }
            }
        }
        if limiter {
            self.drain_rate = 0.0;
            self.since += dt;
            let room = self.reservoir >= v;
            if self.since * self.rate_hz >= 1.0 && room {
                if let Some(slot) = self.flight.iter_mut().find(|f| **f < 0.0) {
                    *slot = 0.0;
                    self.reservoir -= v;
                    self.since = 0.0;
                }
            }
            if !room {
                self.since = self.since.min(1.0 / self.rate_hz);
            }
        } else {
            self.since = 0.0;
            if self.pool > 0.0 {
                // Drains back linearly over `drain_s` from the most the pool has held since the
                // limiter ended (a drop still in flight then lands and drains with the rest).
                self.drain_rate = self.drain_rate.max(self.pool / self.drain_s);
                let back = (self.drain_rate * dt).min(self.pool);
                self.pool -= back;
                self.reservoir += back;
            } else {
                self.drain_rate = 0.0;
            }
        }
    }
}

fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn one_g_tilts_the_surface_45_degrees_and_half_a_g_26_57() {
        assert!(close(surface_tilt(1.0, 0.0).to_degrees(), 45.00, 0.005));
        assert!(close(surface_tilt(0.5, 0.0).to_degrees(), 26.57, 0.005));
        assert!(close(surface_tilt(-1.0, 0.0).to_degrees(), -45.00, 0.005));
    }

    #[test]
    fn vertical_g_not_longitudinal_g_sits_under_the_tilt() {
        // 1 g sideways while 1 g up (a compression): tan θ = 1 / 2.
        assert!(close(surface_tilt(1.0, 1.0).to_degrees(), 26.57, 0.005));
        // Nearly weightless: the effective gravity is held, so the tilt stays defined.
        assert!(surface_tilt(0.5, -1.0).is_finite());
    }

    #[test]
    fn the_slosh_period_of_0_62_s_is_a_column_of_0_191_m() {
        let l = column_length(0.62, STANDARD_G);
        assert!(close(l, 0.191, 0.0005), "l = {l}");
        let w = column_omega(l, STANDARD_G);
        assert!(close(TAU / w, 0.62, 1e-5));
    }

    #[test]
    fn the_column_settles_on_the_hydrostatic_tilt_and_is_still_at_idle() {
        let mut u = UTube::from_period(0.62, 0.35);
        for _ in 0..600 {
            u.step(0.5, 0.0, 1.0 / 60.0);
        }
        assert!(close(u.tilt().to_degrees(), 26.57, 0.01));
        for _ in 0..600 {
            u.step(0.0, 0.0, 1.0 / 60.0);
        }
        assert!(u.settled(0.0, 0.0, 1e-4));
    }

    #[test]
    fn the_column_swings_at_its_damped_period_and_overshoots() {
        // ζ = 0.35: one overshoot of e^(−πζ/√(1−ζ²)) ≈ 30.9%.
        let mut u = UTube::from_period(0.62, 0.35);
        let target = surface_tilt(0.5, 0.0).tan();
        let mut peak: f32 = 0.0;
        for _ in 0..240 {
            u.step(0.5, 0.0, 1.0 / 240.0);
            peak = peak.max(u.slope());
        }
        let overshoot = peak / target - 1.0;
        let expect = (-PI * 0.35 / (1.0 - 0.35f32 * 0.35).sqrt()).exp();
        assert!(close(overshoot, expect, 0.01), "{overshoot} vs {expect}");
    }

    #[test]
    fn the_step_is_the_same_at_any_frame_rate() {
        let mut a = UTube::from_period(0.62, 0.35);
        let mut b = a;
        for _ in 0..60 {
            a.step(0.8, 0.1, 1.0 / 60.0);
        }
        for _ in 0..144 {
            b.step(0.8, 0.1, 1.0 / 144.0);
        }
        assert!(close(a.slope(), b.slope(), 1e-4));
    }

    #[test]
    fn a_kick_swings_and_dies_away() {
        let mut u = UTube::from_period(0.62, 0.35);
        u.kick(3.0);
        u.step(0.0, 0.0, 0.05);
        assert!(u.slope() > 0.05);
        for _ in 0..600 {
            u.step(0.0, 0.0, 1.0 / 60.0);
        }
        assert!(u.settled(0.0, 0.0, 1e-4));
    }

    #[test]
    fn volume_is_conserved_while_the_surface_turns() {
        // A plane at distance d from the centre cuts the same cap whatever its direction, so
        // the pool keeps its volume at any tilt: check by integrating the sphere in slices
        // perpendicular to a tilted axis.
        let r = 132.0;
        let d = 92.0;
        let v = cap_volume(r, d);
        for tilt_deg in [0.0f32, 15.0, 45.0, 70.0] {
            let t = tilt_deg.to_radians();
            // Monte Carlo on a fixed lattice: points of the sphere beyond the plane.
            let n = 80;
            let step = 2.0 * r / n as f32;
            let mut inside = 0u32;
            for i in 0..n {
                for j in 0..n {
                    for k in 0..n {
                        let p = [
                            -r + (i as f32 + 0.5) * step,
                            -r + (j as f32 + 0.5) * step,
                            -r + (k as f32 + 0.5) * step,
                        ];
                        if p[0] * p[0] + p[1] * p[1] + p[2] * p[2] > r * r {
                            continue;
                        }
                        // Down along (sin t, cos t, 0).
                        if p[0] * t.sin() + p[1] * t.cos() > d {
                            inside += 1;
                        }
                    }
                }
            }
            let measured = inside as f32 * step * step * step;
            assert!(close(measured / v, 1.0, 0.03), "tilt {tilt_deg}: {measured} vs {v}");
        }
    }

    #[test]
    fn cap_distance_inverts_cap_volume() {
        let r = 132.0;
        for d in [-100.0f32, 0.0, 60.0, 92.0, 120.0] {
            let v = cap_volume(r, d);
            assert!(close(cap_distance(r, v), d, 0.02), "{d}");
        }
        assert!(close(cap_volume(r, -r), 4.0 / 3.0 * PI * r * r * r, 1.0));
    }

    #[test]
    fn a_mercury_meniscus_rises_0_466_of_the_bore_radius() {
        assert!(close(meniscus_height(1.0, MERCURY_CONTACT_DEG), 0.466, 0.001));
        assert!(close(meniscus_radius(1.0, MERCURY_CONTACT_DEG), 1.305, 0.001));
        // A meniscus meeting the wall square (90°) is flat; one that never wets (180°) is a
        // hemisphere.
        assert!(close(meniscus_height(1.0, 90.0), 0.0, 1e-6));
        assert!(close(meniscus_height(1.0, 180.0), 1.0, 1e-5));
    }

    #[test]
    fn a_thread_pinches_at_kr_0_697_into_drops_of_1_89_r() {
        let r = 4.0;
        assert!(close(drop_spacing(r) / r, 9.01, 0.01));
        assert!(close(drop_radius(r) / r, 1.89, 0.005));
        // A drop holds exactly one wavelength of thread.
        assert!(close(drop_volume(drop_radius(r)), PI * r * r * drop_spacing(r), 0.01));
    }

    #[test]
    fn mercury_reflects_78_percent_head_on_and_all_at_grazing() {
        let (n, k) = index_from_permittivity(MERCURY_EPS.0, MERCURY_EPS.1);
        assert!(close(n, 1.485, 0.002) && close(k, 4.546, 0.002), "{n} {k}");
        assert!(close(n * n - k * k, MERCURY_EPS.0, 1e-3) && close(2.0 * n * k, MERCURY_EPS.1, 1e-3));
        let r0 = ((n - 1.0).powi(2) + k * k) / ((n + 1.0).powi(2) + k * k);
        assert!(close(mercury_reflectance(1.0), r0, 1e-5));
        assert!(close(r0, 0.779, 0.002));
        assert!(close(mercury_reflectance(0.0), 1.0, 1e-5));
        // A metal's reflectance dips a little before grazing (the principal angle), never
        // under 0.7 for mercury.
        for i in 0..=20 {
            assert!(mercury_reflectance(i as f32 / 20.0) > 0.7);
        }
        // The dielectric limit (k = 0) matches the ordinary Fresnel value.
        let glass = conductor_reflectance(1.0, 1.5, 0.0);
        assert!(close(glass, 0.04, 1e-4));
    }

    #[test]
    fn the_drip_ledger_conserves_mercury() {
        let mut d = Drips::new(4.0, 5.0, 0.35, 0.3, 40.0 * drop_volume(drop_radius(4.0)));
        let total = d.total();
        let mut drops = 0;
        let mut was = 0.0;
        for i in 0..300 {
            d.step(i < 120, 1.0 / 60.0);
            assert!(close(d.total(), total, total * 1e-5));
            let flown = d.reservoir;
            if flown < was {
                drops += 1;
            }
            was = flown;
        }
        // 2 s on the limiter at 5 Hz: 10 drops.
        assert!((9..=10).contains(&drops), "{drops}");
        // Drained back.
        assert!(!d.busy());
    }

    #[test]
    fn the_pool_drains_back_in_300_ms_after_the_shift() {
        let mut d = Drips::new(4.0, 5.0, 0.2, 0.3, 40.0 * drop_volume(drop_radius(4.0)));
        for _ in 0..120 {
            d.step(true, 1.0 / 60.0);
        }
        for _ in 0..12 {
            d.step(false, 1.0 / 60.0);
        }
        // Drops still in flight land; whatever reached the pool is gone 0.3 s after it ends.
        for _ in 0..30 {
            d.step(false, 1.0 / 60.0);
        }
        assert!(d.pool() < 1e-3, "{}", d.pool());
    }

    #[test]
    fn the_bubble_runs_towards_the_acceleration() {
        let t = surface_tilt(1.0, 0.0);
        assert!(close(bubble_offset(240.0, t), 240.0 * PI / 4.0, 1e-3));
    }
}
