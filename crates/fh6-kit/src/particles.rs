//! A CPU particle emitter with a fixed-capacity pool: sparks, exhaust embers, shift-light
//! flares, rain on the glass.
//!
//! The pool is allocated once in [`Emitter::new`]; [`Emitter::update`], [`Emitter::burst`] and
//! [`Emitter::draw`] never allocate. Live particles are kept packed at the front of the pool and
//! dead ones are swap-removed, so update cost is proportional to the live count. When the pool is
//! full, new spawns are dropped (never grown).
//!
//! The internal RNG is SplitMix64, seeded per emitter, so a given seed, config and sequence of
//! `dt` values reproduces the same particles exactly (handy for mockups and tests).
//!
//! Positions, speeds, sizes and gravity are in the painter's local units (per second where a
//! rate is involved). Draw the emitter inside the same transform it was updated in.
use crate::color::Color;
use crate::math::{finite_or, lerp, Vec2};
use crate::noise::curl2;
use crate::painter::Painter;
use crate::shape::{Glow, Shape};

/// Largest pool an emitter accepts.
pub const MAX_PARTICLES: usize = 65_536;
/// Longest `dt` one update integrates (larger steps are clamped, e.g. after a hitch).
pub const MAX_DT: f32 = 0.25;
/// The curl-noise clock wraps after this many seconds to keep `f32` precision.
const NOISE_TIME_WRAP: f32 = 4096.0;

/// Curl-noise turbulence applied to every particle (see [`crate::noise::curl2`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CurlNoise {
    /// Acceleration scale in local units per second squared.
    pub strength: f32,
    /// Spatial frequency: noise cells per local unit (`1 / eddy size`).
    pub scale: f32,
    /// How fast the field evolves (noise units per second).
    pub speed: f32,
}

impl Default for CurlNoise {
    fn default() -> Self {
        Self {
            strength: 40.0,
            scale: 0.02,
            speed: 0.5,
        }
    }
}

/// How an [`Emitter`] spawns, moves and draws particles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmitterConfig {
    /// Continuous emission rate in particles per second (0 = bursts only).
    pub rate: f32,
    /// Lifetime range in seconds `[min, max]`, uniformly distributed.
    pub lifetime: [f32; 2],
    /// Initial speed range in local units per second `[min, max]`.
    pub speed: [f32; 2],
    /// Emission direction in radians (clockwise from 3 o'clock on screen, like [`Vec2::from_angle`]).
    pub direction: f32,
    /// Full cone angle in radians around `direction` (`TAU` = all directions).
    pub spread: f32,
    /// Radius at birth, local units.
    pub size_start: f32,
    /// Radius at death, local units.
    pub size_end: f32,
    /// Colour at birth (linear premultiplied).
    pub color_start: Color,
    /// Colour at death (linear premultiplied; transparent for a fade-out).
    pub color_end: Color,
    /// Linear drag in 1/s: velocity decays as `v / (1 + drag * dt)` each step.
    pub drag: f32,
    /// Constant acceleration in local units per second squared (y is down).
    pub gravity: Vec2,
    /// Optional curl-noise turbulence.
    pub curl: Option<CurlNoise>,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            rate: 60.0,
            lifetime: [0.6, 1.2],
            speed: [60.0, 120.0],
            direction: -std::f32::consts::FRAC_PI_2,
            spread: 0.6,
            size_start: 4.0,
            size_end: 1.0,
            color_start: Color::WHITE,
            color_end: Color::TRANSPARENT,
            drag: 0.5,
            gravity: Vec2::ZERO,
            curl: None,
        }
    }
}

/// One live particle.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Particle {
    /// Position, local units.
    pub pos: Vec2,
    /// Velocity, local units per second.
    pub vel: Vec2,
    /// Seconds since birth.
    pub age: f32,
    /// Lifetime in seconds.
    pub life: f32,
}

impl Particle {
    /// Normalised age `0..=1`.
    #[must_use]
    pub fn t(&self) -> f32 {
        if self.life > 0.0 {
            (self.age / self.life).clamp(0.0, 1.0)
        } else {
            1.0
        }
    }
}

/// SplitMix64: a tiny, fast, well-distributed 64-bit generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)` with 24 bits of precision.
    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 * (1.0 / 16_777_216.0)
    }

    fn range(&mut self, r: [f32; 2]) -> f32 {
        lerp(r[0], r[1], self.next_f32())
    }
}

/// A fixed-capacity particle emitter.
#[derive(Debug, Clone)]
pub struct Emitter {
    /// Spawn, motion and look. May be changed between frames.
    pub config: EmitterConfig,
    particles: Vec<Particle>,
    capacity: usize,
    rng: SplitMix64,
    /// Fractional particles owed by the continuous rate.
    accumulator: f32,
    /// Curl-noise clock (wraps at `NOISE_TIME_WRAP`).
    time: f32,
}

/// The default RNG seed.
const DEFAULT_SEED: u64 = 0x00f6_6d15_ea5e_0001;

impl Emitter {
    /// An emitter with room for `capacity` live particles (clamped to `1..=MAX_PARTICLES`) and
    /// the default config and seed. This is the only allocation the emitter makes.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.clamp(1, MAX_PARTICLES);
        Self {
            config: EmitterConfig::default(),
            particles: Vec::with_capacity(capacity),
            capacity,
            rng: SplitMix64(DEFAULT_SEED),
            accumulator: 0.0,
            time: 0.0,
        }
    }

    /// Replaces the config.
    #[must_use]
    pub fn with_config(mut self, config: EmitterConfig) -> Self {
        self.config = config;
        self
    }

    /// Reseeds the RNG (same seed + same inputs = same particles).
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng = SplitMix64(seed);
        self
    }

    /// Most particles alive at once.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Particles alive now.
    #[must_use]
    pub fn len(&self) -> usize {
        self.particles.len()
    }

    /// True when no particle is alive.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.particles.is_empty()
    }

    /// The live particles (unordered).
    #[must_use]
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Kills every particle and resets the emission accumulator (keeps the RNG state).
    pub fn clear(&mut self) {
        self.particles.clear();
        self.accumulator = 0.0;
    }

    /// Advances every particle by `dt` seconds (clamped to `0..=MAX_DT`), removes the dead, then
    /// emits `rate * dt` new particles at `origin`. Allocation-free.
    pub fn update(&mut self, dt: f32, origin: Vec2) {
        let dt = finite_or(dt, 0.0).clamp(0.0, MAX_DT);
        if dt <= 0.0 {
            return;
        }
        let cfg = self.config;
        self.time = (self.time + dt * finite_or(cfg.curl.map_or(0.0, |c| c.speed), 0.0)) % NOISE_TIME_WRAP;
        let gravity = cfg.gravity.sanitized();
        let damping = 1.0 / (1.0 + finite_or(cfg.drag, 0.0).max(0.0) * dt);
        let curl = cfg.curl.map(|c| (finite_or(c.strength, 0.0), finite_or(c.scale, 0.0)));
        let time = self.time;

        let mut i = 0;
        while i < self.particles.len() {
            let p = &mut self.particles[i];
            p.age += dt;
            if p.age >= p.life {
                self.particles.swap_remove(i);
                continue;
            }
            let mut acc = gravity;
            if let Some((strength, scale)) = curl {
                acc += curl2(p.pos.x * scale, p.pos.y * scale, time) * strength;
            }
            p.vel = (p.vel + acc * dt) * damping;
            p.pos += p.vel * dt;
            i += 1;
        }

        let rate = finite_or(cfg.rate, 0.0).max(0.0);
        self.accumulator += rate * dt;
        let owed = self.accumulator.floor();
        self.accumulator -= owed;
        let n = owed.min(self.capacity as f32) as usize;
        for k in 0..n {
            // Spread births across the frame so a stream does not clump into rings.
            let back = dt * (k as f32 + self.rng.next_f32()) / n as f32;
            self.spawn(origin, back);
        }
    }

    /// Spawns up to `n` particles at `origin` immediately. Returns how many fit in the pool.
    pub fn burst(&mut self, n: usize, origin: Vec2) -> usize {
        let free = self.capacity - self.particles.len().min(self.capacity);
        let n = n.min(free);
        for _ in 0..n {
            self.spawn(origin, 0.0);
        }
        n
    }

    /// Spawns one particle at `origin`, pre-advanced by `back` seconds of free flight.
    fn spawn(&mut self, origin: Vec2, back: f32) {
        if self.particles.len() >= self.capacity {
            return;
        }
        let cfg = &self.config;
        let lo_life = finite_or(cfg.lifetime[0], 0.0).max(0.0);
        let hi_life = finite_or(cfg.lifetime[1], lo_life).max(lo_life);
        let lo_speed = finite_or(cfg.speed[0], 0.0);
        let hi_speed = finite_or(cfg.speed[1], lo_speed);
        let direction = finite_or(cfg.direction, 0.0);
        let spread = finite_or(cfg.spread, 0.0);

        let life = self.rng.range([lo_life, hi_life]);
        let speed = self.rng.range([lo_speed, hi_speed]);
        let angle = direction + (self.rng.next_f32() - 0.5) * spread;
        if life <= back {
            return;
        }
        let vel = Vec2::from_angle(angle) * speed;
        let origin = origin.sanitized();
        self.particles.push(Particle {
            pos: origin + vel * back,
            vel,
            age: back,
            life,
        });
    }

    /// Draws every live particle as a filled circle in its interpolated colour and size; with
    /// `glow = Some(sigma)` each also gets an outer glow of that sigma (local units) first.
    /// Allocation-free (draws go into the painter's frame arena).
    pub fn draw(&self, p: &mut Painter<'_>, glow: Option<f32>) {
        let cfg = &self.config;
        let s0 = finite_or(cfg.size_start, 0.0).max(0.0);
        let s1 = finite_or(cfg.size_end, 0.0).max(0.0);
        let sigma = glow.map(|s| finite_or(s, 0.0)).filter(|s| *s > 0.0);
        for particle in &self.particles {
            let t = particle.t();
            let color = cfg.color_start.lerp(cfg.color_end, t);
            let radius = lerp(s0, s1, t);
            if color.a <= 0.0 || radius <= 0.0 {
                continue;
            }
            let shape = Shape::circle(particle.pos, radius);
            if let Some(sigma) = sigma {
                p.glow(&shape, color, Glow::new(sigma, 1.0));
            }
            p.fill(&shape, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vec2;

    fn config() -> EmitterConfig {
        EmitterConfig {
            rate: 500.0,
            lifetime: [0.5, 1.0],
            speed: [50.0, 100.0],
            gravity: vec2(0.0, 98.0),
            curl: Some(CurlNoise::default()),
            ..EmitterConfig::default()
        }
    }

    #[test]
    fn count_never_exceeds_capacity_and_pool_never_grows() {
        let mut e = Emitter::new(100).with_config(config());
        let cap0 = e.particles.capacity();
        assert!(cap0 >= 100);
        for frame in 0..600 {
            e.update(1.0 / 60.0, vec2(0.0, 0.0));
            if frame % 50 == 0 {
                e.burst(1000, vec2(10.0, 10.0));
            }
            assert!(e.len() <= e.capacity(), "frame {frame}: {}", e.len());
            assert_eq!(e.particles.capacity(), cap0, "pool reallocated at frame {frame}");
        }
        assert_eq!(e.len(), 100, "a 500/s stream should keep a 100-particle pool full");
        assert_eq!(e.burst(10, Vec2::ZERO), 0);
    }

    #[test]
    fn capacity_is_clamped() {
        assert_eq!(Emitter::new(0).capacity(), 1);
        assert_eq!(Emitter::new(usize::MAX).capacity(), MAX_PARTICLES);
    }

    #[test]
    fn particles_die_after_their_lifetime() {
        let mut e = Emitter::new(64).with_config(EmitterConfig {
            rate: 0.0,
            lifetime: [0.4, 0.5],
            ..config()
        });
        assert_eq!(e.burst(32, Vec2::ZERO), 32);
        e.update(0.3, Vec2::ZERO);
        assert_eq!(e.len(), 32);
        assert!(e.particles().iter().all(|p| p.age > 0.0 && p.age < p.life));
        e.update(0.25, Vec2::ZERO);
        assert!(e.is_empty(), "{} particles outlived a 0.5 s max lifetime", e.len());
    }

    #[test]
    fn rate_emits_the_expected_count() {
        let mut e = Emitter::new(10_000).with_config(EmitterConfig {
            rate: 120.0,
            lifetime: [10.0, 10.0],
            ..EmitterConfig::default()
        });
        for _ in 0..60 {
            e.update(1.0 / 60.0, Vec2::ZERO);
        }
        assert!((119..=120).contains(&e.len()), "{}", e.len());
    }

    #[test]
    fn deterministic_with_the_same_seed() {
        let run = |seed: u64| {
            let mut e = Emitter::new(256).with_config(config()).with_seed(seed);
            for i in 0..120 {
                e.update(1.0 / 60.0, vec2(i as f32, 0.0));
                if i == 30 {
                    e.burst(40, vec2(5.0, 5.0));
                }
            }
            e.particles().to_vec()
        };
        let a = run(7);
        assert!(!a.is_empty());
        assert_eq!(a, run(7));
        assert_ne!(a, run(8));
    }

    #[test]
    fn hostile_inputs_stay_finite() {
        let mut e = Emitter::new(64).with_config(EmitterConfig {
            rate: f32::INFINITY,
            lifetime: [f32::NAN, 1.0],
            speed: [f32::NAN, f32::INFINITY],
            drag: f32::NAN,
            gravity: vec2(f32::NAN, 1.0),
            ..config()
        });
        e.update(f32::NAN, vec2(f32::NAN, 0.0));
        e.update(1.0 / 60.0, vec2(f32::NAN, 0.0));
        e.update(1e9, Vec2::ZERO);
        assert!(e.len() <= 64);
        for p in e.particles() {
            assert!(p.pos.is_finite() && p.vel.is_finite());
            assert!(p.age.is_finite() && p.life.is_finite());
        }
    }
}
