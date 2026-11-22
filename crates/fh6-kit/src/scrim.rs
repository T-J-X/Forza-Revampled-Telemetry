//! The OEM dash scrim and the binnacle ambient wash, drawn with the kit
//! (`docs/design/SCRIM_FIT.md` §2 and §3).
//!
//! Both draw in cluster local space (4K px at scale 1.0 around the anchor), so they follow the
//! cluster's anchor, height offset and scale. Draw them before the cluster: game frame, scrim,
//! wash, cluster, OSD.
//!
//! ```no_run
//! # use fh6_kit::{scrim, Painter, Color};
//! # use fh6_core::{AppConfig, FitView, ScrimFit};
//! # fn draw(p: &mut Painter<'_>, cfg: &AppConfig) {
//! let fit = ScrimFit::from_config(cfg, FitView::Cockpit);
//! scrim::draw_scrim(p, &fit);
//! let wash = scrim::Wash::from_config(cfg, p.palette().primary);
//! scrim::draw_wash(p, &fit, wash);
//! # }
//! ```
use crate::color::Color;
use crate::math::{vec2, Rect};
use crate::paint::Gradient;
use crate::painter::Painter;
use crate::shape::{Dome, Glow, Shape, Stroke};
use fh6_core::config::AppConfig;
use fh6_core::fit::ScrimFit;

/// The wash's lowest visible alpha (SCRIM_FIT §3: 12–35 of 255).
pub const WASH_ALPHA_MIN: f32 = 12.0 / 255.0;
/// The wash's highest alpha.
pub const WASH_ALPHA_MAX: f32 = 35.0 / 255.0;
/// Width of the lip line in device pixels (SCRIM_FIT §3: 1–2 px).
pub const WASH_LIP_PX: f32 = 1.5;
/// The lip only runs along the top of the dome: above `center.y - WASH_TOP_FRACTION * b`.
pub const WASH_TOP_FRACTION: f32 = 0.45;

/// The kit dome for a scrim fit.
#[must_use]
pub fn dome(fit: &ScrimFit) -> Dome {
    Dome {
        center: vec2(fit.center.0, fit.center.1),
        semi_axes: vec2(fit.semi_axes.0, fit.semi_axes.1),
        exponent_top: fit.exponent_top,
        exponent_bottom: fit.exponent_bottom,
        feather_top: fit.feather.0,
        feather_side: fit.feather.1,
        feather_bottom: fit.feather.2,
        void_half_width: fit.void_half_width,
        void_top: fit.void_top,
        void_feather: fit.void_feather,
    }
}

/// Draws the scrim: pure `#000000` at `opacity` in the core, smoothstep feather (dithered),
/// steering-column void open. Nothing is drawn at 0%.
pub fn draw_scrim(p: &mut Painter<'_>, fit: &ScrimFit) {
    if !fit.visible() {
        return;
    }
    let o = fit.opacity.clamp(0.0, 1.0);
    p.fill(&Shape::Dome(dome(fit)), Color::BLACK.with_alpha(o));
}

/// The binnacle ambient wash: a thin arc of the cluster colour along the dome's top edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wash {
    /// Colour (the cluster palette's `primary`).
    pub color: Color,
    /// Peak alpha, 0 (off) or [`WASH_ALPHA_MIN`]..=[`WASH_ALPHA_MAX`].
    pub alpha: f32,
}

impl Wash {
    /// The wash from `AmbientGlowEnabled` / `AmbientGlowIntensity` (0..1 maps onto alpha 12–35).
    /// `NightAmbientGlowEnabled` lifts the intensity to at least
    /// `NightAmbientGlowIntensityPct` / 100 ("brighter at night").
    #[must_use]
    pub fn from_config(cfg: &AppConfig, color: Color) -> Self {
        if !cfg.ambient_glow_enabled {
            return Self { color, alpha: 0.0 };
        }
        let mut k = if cfg.ambient_glow_intensity.is_finite() {
            cfg.ambient_glow_intensity.clamp(0.0, 1.0)
        } else {
            0.25
        };
        if cfg.night_ambient_glow_enabled {
            k = k.max((cfg.night_ambient_glow_intensity_pct.clamp(0, 100) as f32) / 100.0);
        }
        Self {
            color,
            alpha: WASH_ALPHA_MIN + (WASH_ALPHA_MAX - WASH_ALPHA_MIN) * k,
        }
    }
}

/// Draws the wash on top of the scrim. Its alpha scales with the scrim opacity (no dome, no
/// lip), and it is only ever a line and a faint halo, never a fill.
pub fn draw_wash(p: &mut Painter<'_>, fit: &ScrimFit, wash: Wash) {
    let a = wash.alpha.clamp(0.0, WASH_ALPHA_MAX) * fit.opacity.clamp(0.0, 1.0);
    if !fit.visible() || a <= 0.0 {
        return;
    }
    let d = dome(fit);
    let (cx, cy) = fit.center;
    let (sa, sb) = fit.semi_axes;
    let top = cy - sb;
    let clip_bottom = cy - WASH_TOP_FRACTION * sb;
    let feather = fit.feather.0.max(fit.feather.1);
    let clip = Rect::from_points(
        vec2(cx - sa - feather, top - feather),
        vec2(cx + sa + feather, clip_bottom),
    );
    // Fades out towards the ends of the arc so it never stops on a hard corner.
    let c = wash.color.with_alpha(1.0);
    let ramp = Gradient::linear(vec2(cx - sa, top), vec2(cx + sa, top))
        .stop(0.0, c.with_alpha(0.0))
        .stop(0.22, c.with_alpha(a * 0.6))
        .stop(0.5, c.with_alpha(a))
        .stop(0.78, c.with_alpha(a * 0.6))
        .stop(1.0, c.with_alpha(0.0));
    let shape = Shape::Dome(d);
    p.scoped(|p| {
        p.clip(clip, 0.0);
        p.glow(&shape, ramp, Glow::new(fit.feather.0 * 0.12, 0.5));
        p.stroke(&shape, ramp, Stroke::rim_px(WASH_LIP_PX));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::fit::FitView;

    #[test]
    fn dome_matches_the_fit() {
        let fit = ScrimFit::from_config(&AppConfig::default(), FitView::Cockpit);
        let d = dome(&fit);
        assert_eq!(d.center, vec2(0.0, -75.0));
        assert_eq!(d.semi_axes, vec2(640.0, 470.0));
        assert_eq!(d.feather_top, 96.0);
        assert_eq!(d.feather_side, 120.0);
        assert_eq!(d.feather_bottom, 140.0);
        assert_eq!(d.void_half_width, 110.0);
    }

    #[test]
    fn wash_alpha_stays_in_the_spec_band() {
        let mut cfg = AppConfig::default();
        let w = Wash::from_config(&cfg, Color::WHITE);
        assert!((WASH_ALPHA_MIN..=WASH_ALPHA_MAX).contains(&w.alpha));
        cfg.ambient_glow_intensity = 9.0;
        assert_eq!(Wash::from_config(&cfg, Color::WHITE).alpha, WASH_ALPHA_MAX);
        cfg.ambient_glow_intensity = f32::NAN;
        assert!(Wash::from_config(&cfg, Color::WHITE).alpha.is_finite());
        cfg.ambient_glow_intensity = 0.0;
        cfg.night_ambient_glow_enabled = true;
        cfg.night_ambient_glow_intensity_pct = 100;
        assert_eq!(Wash::from_config(&cfg, Color::WHITE).alpha, WASH_ALPHA_MAX);
        cfg.ambient_glow_enabled = false;
        assert_eq!(Wash::from_config(&cfg, Color::WHITE).alpha, 0.0);
    }
}
