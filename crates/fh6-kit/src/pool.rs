//! Soft black pools under a frame: [`Pool`], its GPU form and the CPU mirror of
//! `shaders/pool.wgsl` (`Kit::set_pools`).

use crate::math::{vec2, Rect, Vec2};
use bytemuck::{Pod, Zeroable};

/// Most pools one frame can have (`Kit::set_pools` keeps the first this many).
pub const MAX_POOLS: usize = 6;

/// Where an ellipse's fall-off bends, as a share of its radius, and the share of the core alpha
/// left there.
const KNEE: f32 = 0.55;
const KNEE_ALPHA: f32 = 0.75;
/// How far a rounded rectangle's feather is drawn past its edge, in sigmas. Three sigmas would
/// be under one 8-bit step of alpha, but not far enough: the resolve encodes light up to the
/// alpha under it as coverage, so faint light (a bloom halo at 3/255) is held back by an alpha
/// of 0.001 and released where the pool stops, which draws the pool's bounds as a straight
/// line in the halo. At four sigmas 0.003% of the core alpha is left and the step is under a
/// tenth of an 8-bit level.
const REACH_SIGMAS: f32 = 4.0;

/// A soft black pool under everything a frame draws: black at `alpha` in its core, fading to
/// nothing, in target pixels.
///
/// It is the same picture as drawing the pool first with the painter (a radial gradient on an
/// ellipse, or a [`crate::Shadow`]), and what is drawn later lies over it in the same way. The
/// difference is the cost: a painted shape blends into the scene and the bloom target, which
/// reads and writes 32 bytes for every pixel it covers, so a pool the size of a 4K menu column
/// costs about 0.15 ms of GPU time. `Kit::set_pools` writes every pool in one pass that blends
/// nothing and reads nothing, before the frame's first layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pool {
    /// A feathered ellipse: `alpha` at the centre, three quarters of it at 55% of the radius and
    /// nothing at the edge, in two straight lines.
    Ellipse {
        /// Centre (target pixels).
        center: Vec2,
        /// Radii (target pixels).
        radii: Vec2,
        /// Alpha at the centre (0..1).
        alpha: f32,
    },
    /// The soft shadow of a rounded rectangle: `alpha` deep inside, half of it on the outline,
    /// with the fall-off of a [`crate::Shadow`] of this `sigma` and no offset or spread.
    RoundRect {
        /// The rectangle (target pixels).
        rect: Rect,
        /// Corner radius (target pixels).
        radius: f32,
        /// Softness (target pixels).
        sigma: f32,
        /// Alpha deep inside (0..1).
        alpha: f32,
    },
}

impl Pool {
    /// True when the pool can darken something: a finite shape with some area and alpha.
    #[must_use]
    pub fn is_visible(&self) -> bool {
        let finite = |v: Vec2| v.x.is_finite() && v.y.is_finite();
        match *self {
            Pool::Ellipse { center, radii, alpha } => {
                finite(center) && finite(radii) && radii.x > 0.0 && radii.y > 0.0 && alpha.is_finite() && alpha > 0.0
            }
            Pool::RoundRect {
                rect,
                radius,
                sigma,
                alpha,
            } => {
                finite(rect.min)
                    && finite(rect.max)
                    && !rect.is_empty()
                    && radius.is_finite()
                    && sigma.is_finite()
                    && alpha.is_finite()
                    && alpha > 0.0
            }
        }
    }

    /// The pixels the pool is drawn into (target pixels): outside them it is clear.
    #[must_use]
    pub fn bounds(&self) -> Rect {
        match *self {
            Pool::Ellipse { center, radii, .. } => Rect::from_points(center - radii, center + radii),
            Pool::RoundRect { rect, sigma, .. } => rect.expand(REACH_SIGMAS * sigma.max(0.0)),
        }
    }

    /// The pool's alpha at `p` (target pixels): what `shaders/pool.wgsl` writes for it.
    #[must_use]
    pub fn alpha_at(&self, p: Vec2) -> f32 {
        if !self.is_visible() {
            return 0.0;
        }
        match *self {
            Pool::Ellipse { center, radii, alpha } => {
                let q = p - center;
                let t = vec2(q.x / radii.x, q.y / radii.y).length();
                let alpha = alpha.clamp(0.0, 1.0);
                if t >= 1.0 {
                    0.0
                } else if t <= KNEE {
                    alpha * (1.0 - (1.0 - KNEE_ALPHA) * t / KNEE)
                } else {
                    alpha * KNEE_ALPHA * (1.0 - t) / (1.0 - KNEE)
                }
            }
            Pool::RoundRect {
                rect,
                radius,
                sigma,
                alpha,
            } => {
                let half = rect.size() * 0.5;
                let d = sd_round_box(p - rect.center(), half, radius);
                let sigma = sigma.max(1e-4);
                alpha.clamp(0.0, 1.0) * 0.5 * (1.0 - erf(d / (sigma * std::f32::consts::SQRT_2)))
            }
        }
    }

    /// The alpha of `pools` together at `p` (target pixels): each lies over the ones before it,
    /// as black layers do.
    #[must_use]
    pub fn combined_alpha(pools: &[Pool], p: Vec2) -> f32 {
        1.0 - pools.iter().fold(1.0, |keep, pool| keep * (1.0 - pool.alpha_at(p)))
    }

    /// The pool as the shader reads it.
    pub(crate) fn gpu(&self) -> GpuPool {
        match *self {
            Pool::Ellipse { center, radii, alpha } => GpuPool {
                shape: [center.x, center.y, radii.x, radii.y],
                style: [0.0, 0.0, alpha.clamp(0.0, 1.0), KIND_ELLIPSE],
            },
            Pool::RoundRect {
                rect,
                radius,
                sigma,
                alpha,
            } => {
                let (c, half) = (rect.center(), rect.size() * 0.5);
                GpuPool {
                    shape: [c.x, c.y, half.x, half.y],
                    style: [radius.max(0.0), sigma.max(1e-4), alpha.clamp(0.0, 1.0), KIND_ROUND_RECT],
                }
            }
        }
    }
}

const KIND_ELLIPSE: f32 = 1.0;
const KIND_ROUND_RECT: f32 = 2.0;

/// One pool in the uniform block: `shape` is the centre and the half size, `style` the corner
/// radius, the sigma, the core alpha and the kind.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuPool {
    shape: [f32; 4],
    style: [f32; 4],
}

/// The pool pass's uniform block (`Pools` in `shaders/pool.wgsl`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub(crate) struct PoolBlock {
    /// Work origin in target pixels.
    pub(crate) origin: [f32; 2],
    pub(crate) count: u32,
    pub(crate) pad: u32,
    pub(crate) pools: [GpuPool; MAX_POOLS],
}

/// Rounded box distance (the shader's `sd_round_box`).
fn sd_round_box(p: Vec2, half: Vec2, radius: f32) -> f32 {
    let r = radius.clamp(0.0, half.x.min(half.y));
    let q = vec2(p.x.abs() - half.x + r, p.y.abs() - half.y + r);
    vec2(q.x.max(0.0), q.y.max(0.0)).length() + q.x.max(q.y).min(0.0) - r
}

/// The error function as the shaders approximate it (Abramowitz and Stegun 7.1.26, within
/// about 5e-4).
fn erf(x: f32) -> f32 {
    let a = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * a);
    let poly = ((((1.061_405_4 * t - 1.453_152_1) * t + 1.421_413_8) * t - 0.284_496_74) * t + 0.254_829_6) * t;
    (1.0 - poly * (-a * a).exp()).copysign(x)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ELLIPSE: Pool = Pool::Ellipse {
        center: Vec2 { x: 400.0, y: 300.0 },
        radii: Vec2 { x: 200.0, y: 100.0 },
        alpha: 0.4,
    };

    fn column() -> Pool {
        Pool::RoundRect {
            rect: Rect::from_xywh(1000.0, 100.0, 600.0, 900.0),
            radius: 60.0,
            sigma: 90.0,
            alpha: 0.78,
        }
    }

    #[test]
    fn the_block_fits_one_uniform_slot() {
        assert_eq!(size_of::<GpuPool>(), 32);
        assert_eq!(size_of::<PoolBlock>(), 16 + 32 * MAX_POOLS);
        assert!(size_of::<PoolBlock>() <= 256);
        assert_eq!(size_of::<PoolBlock>() % 16, 0);
    }

    #[test]
    fn an_ellipse_falls_in_two_straight_lines_to_nothing_at_its_edge() {
        let at = |t: f32| ELLIPSE.alpha_at(vec2(400.0 + 200.0 * t, 300.0));
        assert!((at(0.0) - 0.4).abs() < 1e-6);
        assert!((at(0.55) - 0.3).abs() < 1e-5, "three quarters at the knee");
        assert!((at(0.275) - 0.35).abs() < 1e-5, "halfway to the knee");
        assert!((at(0.775) - 0.15).abs() < 1e-5, "halfway from the knee to the edge");
        assert_eq!(at(1.0), 0.0);
        assert_eq!(at(1.5), 0.0);
        // The same share of the radius on the short axis gives the same alpha.
        assert!((ELLIPSE.alpha_at(vec2(400.0, 300.0 - 55.0)) - 0.3).abs() < 1e-5);
    }

    #[test]
    fn a_rounded_rectangle_is_half_as_dark_on_its_outline_and_gone_four_sigmas_out() {
        let pool = column();
        assert!((pool.alpha_at(vec2(1300.0, 550.0)) - 0.78).abs() < 1e-3, "the core");
        assert!(
            (pool.alpha_at(vec2(1000.0, 550.0)) - 0.39).abs() < 1e-3,
            "the left edge"
        );
        assert!(
            (pool.alpha_at(vec2(1300.0, 1000.0)) - 0.39).abs() < 1e-3,
            "the bottom edge"
        );
        // Where its bounds end, too little is left to hold back even faint light (REACH_SIGMAS).
        let out = pool.alpha_at(vec2(1000.0 - 360.0, 550.0));
        assert!(out < 1e-4, "at the edge of its bounds: {out}");
        assert!(pool.bounds().contains(vec2(1000.0 - 359.0, 550.0)));
        assert!(!pool.bounds().contains(vec2(1000.0 - 361.0, 550.0)));
    }

    #[test]
    fn pools_lie_over_each_other_like_black_layers() {
        let a = ELLIPSE;
        let b = Pool::Ellipse {
            center: vec2(500.0, 300.0),
            radii: vec2(200.0, 100.0),
            alpha: 0.5,
        };
        let p = vec2(450.0, 300.0);
        let (pa, pb) = (a.alpha_at(p), b.alpha_at(p));
        assert!(pa > 0.0 && pb > 0.0);
        assert!((Pool::combined_alpha(&[a, b], p) - (pa + pb * (1.0 - pa))).abs() < 1e-6);
        assert_eq!(Pool::combined_alpha(&[], p), 0.0);
    }

    #[test]
    fn a_pool_that_cannot_show_is_not_visible() {
        assert!(ELLIPSE.is_visible());
        assert!(column().is_visible());
        let gone = [
            Pool::Ellipse {
                center: vec2(0.0, 0.0),
                radii: vec2(10.0, 10.0),
                alpha: 0.0,
            },
            Pool::Ellipse {
                center: vec2(0.0, 0.0),
                radii: vec2(0.0, 10.0),
                alpha: 0.5,
            },
            Pool::Ellipse {
                center: vec2(f32::NAN, 0.0),
                radii: vec2(10.0, 10.0),
                alpha: 0.5,
            },
            Pool::RoundRect {
                rect: Rect::EMPTY,
                radius: 4.0,
                sigma: 9.0,
                alpha: 0.5,
            },
            Pool::RoundRect {
                rect: Rect::from_xywh(0.0, 0.0, 10.0, 10.0),
                radius: 4.0,
                sigma: f32::INFINITY,
                alpha: 0.5,
            },
        ];
        for pool in gone {
            assert!(!pool.is_visible(), "{pool:?}");
            assert_eq!(pool.alpha_at(vec2(1.0, 1.0)), 0.0);
        }
    }

    #[test]
    fn the_error_function_matches_its_known_values() {
        for (x, want) in [
            (0.0, 0.0),
            (0.5, 0.520_5),
            (1.0, 0.842_7),
            (2.0, 0.995_32),
            (-1.0, -0.842_7),
        ] {
            assert!((erf(x) - want).abs() < 6e-4, "erf({x}) = {}", erf(x));
        }
    }
}
