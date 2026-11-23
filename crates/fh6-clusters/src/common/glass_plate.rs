//! GlassPlate (COMPONENTS.md §8): the one Liquid Glass slab a cluster floats in.
//!
//! A squircle (superellipse, exponent 4–4.2) or pill of the kit's [`Glass`] material: tint from
//! the palette's `glass_tint` (alpha ≤ 44 for the regular variant), frost, self-lensing, fresnel
//! band, the 1 px gradient rim and 1 px inner rim, and the crown sheen. The rim's light comes
//! from the virtual light, so the crescent and sheen swing with lateral G. It materialises in
//! 200 ms by ramping lensing, fresnel and rim (never opacity); hiding is the host's 1-frame job.
use crate::common::VirtualLight;
use fh6_kit::{
    Color, Dome, Glass, Gradient, LayerOptions, Painter, Palette, Rect, Refraction, Shape, Tokens, Tween, Vec2,
};

/// A squircle (superellipse) centred on `center` with full `size` and exponent `n` (2 = ellipse,
/// 4–4.2 = Apple's continuous-curvature squircle). Works with fill, stroke, glow, shadow and the
/// glass material (it is the kit's dome shape with no feather and no void).
#[must_use]
pub fn squircle(center: Vec2, size: Vec2, n: f32) -> Shape<'static> {
    Shape::Dome(Dome {
        center,
        semi_axes: size * 0.5,
        exponent_top: n,
        exponent_bottom: n,
        feather_top: 0.0,
        feather_side: 0.0,
        feather_bottom: 0.0,
        void_half_width: 0.0,
        void_top: 0.0,
        void_feather: 0.0,
    })
}

/// A point on the squircle outline at parameter `theta` (radians, clockwise from 3 o'clock),
/// inset by `inset` local units along the approximate normal. Use it to run light pipes, glints
/// and beads along the rim.
#[must_use]
pub fn squircle_point(center: Vec2, size: Vec2, n: f32, theta: f32, inset: f32) -> Vec2 {
    let (s, c) = theta.sin_cos();
    let e = 2.0 / n.max(1.0);
    let half = size * 0.5;
    let x = c.signum() * c.abs().powf(e) * half.x;
    let y = s.signum() * s.abs().powf(e) * half.y;
    if inset == 0.0 {
        return center + Vec2::new(x, y);
    }
    // Normal of |x/a|^n + |y/b|^n = 1 is (n x^(n-1) / a^n, n y^(n-1) / b^n).
    let nx = x.signum() * (x.abs() / half.x.max(1e-3)).powf(n - 1.0) / half.x.max(1e-3);
    let ny = y.signum() * (y.abs() / half.y.max(1e-3)).powf(n - 1.0) / half.y.max(1e-3);
    let len = (nx * nx + ny * ny).sqrt().max(1e-6);
    center + Vec2::new(x - nx / len * inset, y - ny / len * inset)
}

/// Which glass variant a cluster uses (one per cluster, never mixed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GlassVariant {
    /// Regular: tint alpha from tokens `glass.tintAlpha.default` (44).
    #[default]
    Regular,
    /// Clear: the minimum tint (28), for plates that sit over busy layers.
    Clear,
    /// Tinted: the maximum tint (64), for OSD-like panels.
    Tinted,
}

/// The glass slab state: virtual light and the materialise ramp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlassPlate {
    variant: GlassVariant,
    light: VirtualLight,
    materialize: Tween,
    glass: Glass,
    frost_sigma: f32,
}

impl GlassPlate {
    /// A plate that starts materialising now.
    #[must_use]
    pub fn new(tokens: &Tokens, palette: &Palette, variant: GlassVariant) -> Self {
        let mut materialize = Tween::new(tokens.durations.materialize);
        materialize.restart();
        let mut plate = Self {
            variant,
            light: VirtualLight::new(tokens),
            materialize,
            glass: tokens.glass(palette.glass_tint),
            frost_sigma: 18.0,
        };
        plate.retint(tokens, palette);
        plate
    }

    /// Sets the frost blur sigma of the backdrop layer (local units; default 18).
    #[must_use]
    pub fn with_frost_sigma(mut self, sigma: f32) -> Self {
        self.frost_sigma = sigma.max(0.0);
        self
    }

    fn retint(&mut self, tokens: &Tokens, palette: &Palette) {
        let mut g = tokens.glass(palette.glass_tint);
        let alpha = match self.variant {
            GlassVariant::Regular => tokens.glass.tint_alpha[1],
            GlassVariant::Clear => tokens.glass.tint_alpha[0],
            GlassVariant::Tinted => tokens.glass.tint_alpha[2],
        };
        g.tint = palette.glass_tint.with_alpha8(alpha);
        // The rim's coloured stops follow the palette (tokens: "accent role may replace
        // #38BDF8"), so a factory, custom or random palette re-colours the glass edge too.
        g.rim_mid = palette.primary.with_alpha(tokens.glass.rim[1].a);
        g.rim_bottom = palette
            .primary
            .lerp(palette.roles.oled_black, 0.35)
            .with_alpha(tokens.glass.rim[2].a);
        g.refraction = Refraction::World;
        self.glass = g;
    }

    /// Advances the light (smoothed lateral G) and the materialise ramp, and follows palette
    /// changes.
    pub fn update(&mut self, g_lat: f32, dt: f32, tokens: &Tokens, palette: &Palette) {
        self.light.update(g_lat, dt);
        self.materialize.update(dt);
        if self.glass.tint.with_alpha(1.0) != palette.glass_tint.with_alpha(1.0) {
            self.retint(tokens, palette);
        } else {
            let mid = palette.primary.with_alpha(tokens.glass.rim[1].a);
            if self.glass.rim_mid != mid {
                self.retint(tokens, palette);
            }
        }
    }

    /// Restarts the 200 ms materialise ramp (call when the cluster is shown again).
    pub fn rematerialize(&mut self) {
        self.materialize.restart();
    }

    /// 0..1 progress of the materialise ramp.
    #[must_use]
    pub fn materialized(&self) -> f32 {
        self.materialize.progress()
    }

    /// The light azimuth (radians), for rims, sheens and speculars drawn outside the glass.
    #[must_use]
    pub fn light_angle(&self) -> f32 {
        self.light.angle()
    }

    /// The light direction as a unit vector pointing *towards* the light (screen space, y down).
    #[must_use]
    pub fn light_dir(&self) -> Vec2 {
        let a = self.light.angle();
        Vec2::new(a.cos(), a.sin())
    }

    /// True while the materialise ramp runs.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.materialize.running()
    }

    /// The material for this frame: lensing, fresnel and rim ramped by the materialise
    /// progress; the rim, crescent and sheen follow the light. `rim_boost` (0..1) brightens the
    /// rim steadily (the family's limiter cue) without blooming it.
    #[must_use]
    pub fn material(&self, rim_boost: f32) -> Glass {
        let m = self.materialize.progress();
        let k = 1.0 + 0.35 * rim_boost.clamp(0.0, 1.0);
        let fade = |c: Color| c.with_alpha((c.a * m * k).min(0.95));
        Glass {
            lens: self.glass.lens * m,
            fresnel: self.glass.fresnel * m,
            rim_top: fade(self.glass.rim_top),
            rim_mid: fade(self.glass.rim_mid),
            rim_bottom: fade(self.glass.rim_bottom),
            inner_rim: fade(self.glass.inner_rim),
            sheen: self.glass.sheen * m,
            light_angle: self.light.angle(),
            ..self.glass
        }
    }

    /// Draws the slab in its own glass layer. Draw it **before** the content that sits on it.
    pub fn draw(&self, p: &mut Painter<'_>, shape: &Shape<'_>, rim_boost: f32) {
        let glass = self.material(rim_boost);
        p.layer(LayerOptions::glass(self.frost_sigma), |p| {
            p.fill(shape, glass);
        });
    }
}

/// A dimming pad (tokens `glass.dimmingPad`): a soft black radial pool under a hero glyph so it
/// reads at full contrast over anything behind the glass. Never a visible disc.
pub fn dimming_pad(p: &mut Painter<'_>, center: Vec2, radius: f32, strength: f32, palette: &Palette) {
    let black = palette.roles.oled_black;
    p.fill(
        &Shape::circle(center, radius),
        Gradient::radial(center, radius)
            .stop(0.0, black.with_alpha(0.55 * strength.clamp(0.0, 1.0)))
            .stop(0.6, black.with_alpha(0.32 * strength.clamp(0.0, 1.0)))
            .stop(1.0, black.with_alpha(0.0)),
    );
}

/// A legibility halo for the hood view (the scrim is 0% there): a wide, soft black pool behind
/// `rect` at alpha ≤ 0.55, so glass and digits read over a bright bonnet.
pub fn legibility_halo(p: &mut Painter<'_>, rect: Rect, alpha: f32, palette: &Palette) {
    let a = alpha.clamp(0.0, 0.55);
    let c = rect.center();
    let grow = rect.height() * 0.6;
    let outer = rect.expand(grow);
    p.fill(
        &Shape::round_rect(outer, outer.height() * 0.5),
        Gradient::radial(c, outer.width() * 0.5)
            .stop(0.0, palette.roles.oled_black.with_alpha(a))
            .stop(0.55, palette.roles.oled_black.with_alpha(a * 0.8))
            .stop(1.0, palette.roles.oled_black.with_alpha(0.0)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::{vec2, Palette, Tokens};

    #[test]
    fn squircle_points_lie_on_the_outline() {
        let size = vec2(1240.0, 300.0);
        for i in 0..16 {
            let th = i as f32 * std::f32::consts::TAU / 16.0;
            let q = squircle_point(Vec2::ZERO, size, 4.0, th, 0.0);
            let s = (q.x / 620.0).abs().powi(4) + (q.y / 150.0).abs().powi(4);
            assert!((s - 1.0).abs() < 1e-3, "theta {th}: {s}");
            let inner = squircle_point(Vec2::ZERO, size, 4.0, th, 14.0);
            let si = (inner.x / 620.0).abs().powi(4) + (inner.y / 150.0).abs().powi(4);
            assert!(si < 1.0, "inset points are inside");
        }
    }

    #[test]
    fn materialise_ramps_lensing_not_opacity() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "liquidGlass");
        let mut g = GlassPlate::new(&t, &pal, GlassVariant::Regular);
        let start = g.material(0.0);
        assert!(start.lens < 1e-3 && start.tint.a > 0.1);
        for _ in 0..30 {
            g.update(0.0, 1.0 / 60.0, &t, &pal);
        }
        let done = g.material(0.0);
        assert!(done.lens > 1.0 && !g.animating());
        assert!(g.material(1.0).rim_top.a >= done.rim_top.a);
    }

    #[test]
    fn rim_follows_the_palette() {
        let t = Tokens::default();
        let a = Palette::family(&t, "liquidGlass");
        let b = Palette::family(&t, "classicAnalog");
        let mut g = GlassPlate::new(&t, &a, GlassVariant::Regular);
        g.update(0.0, 0.016, &t, &b);
        let (got, want) = (g.material(0.0).rim_mid.with_alpha(1.0), b.primary.with_alpha(1.0));
        assert!(
            (got.r - want.r).abs() + (got.g - want.g).abs() + (got.b - want.b).abs() < 1e-3,
            "{got:?} vs {want:?}"
        );
    }
}
