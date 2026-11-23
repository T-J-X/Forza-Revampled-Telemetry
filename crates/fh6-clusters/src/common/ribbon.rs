//! Tach `ribbon` (COMPONENTS.md §3): a light ribbon along any path, with the Liquid Glass light
//! pipe (a ribbon inset along a glass outline) as its main use.
//!
//! The path is sampled once into a fixed array (no allocation per frame) and parameterised by
//! arc length, so the fill, the meniscus bead, glints and comets can all be placed at a
//! fraction of the length. The fill runs on the tokens' `ribbon` spring with feed-forward (it
//! lags the rpm by less than one 60 Hz frame), its width grows `w0 → w1` with a power curve, and
//! its colour runs primary → white towards the tip, with the redline share in the warning role.
use crate::common::glass_plate::squircle_point;
use fh6_kit::{deg, Cap, Color, Glow, Gradient, LayerOptions, Painter, Palette, Shape, Spring, Tokens, Vec2};

/// Samples along a ribbon path.
pub const RIBBON_POINTS: usize = 129;
/// Segments per polyline chunk of the track (keeps each chunk's bounding box small).
const TRACK_CHUNK: usize = 8;

/// How the ribbon is drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RibbonStyle {
    /// Width at the start of the path (local px).
    pub width_start: f32,
    /// Width at the end of the path.
    pub width_end: f32,
    /// Width curve exponent (tokens: 1.6 for the arch).
    pub power: f32,
    /// Share of the path at the end that is the redline (0.10 = last 10%).
    pub redline: f32,
    /// Track (unlit path) width; 0 draws no track.
    pub track_width: f32,
    /// Brightness of the lit fill (1 = the palette colours; lower it so a long ribbon never
    /// outshines the gear).
    pub gain: f32,
}

impl Default for RibbonStyle {
    /// The Liquid Glass light pipe: 6 → 14 px, power 1.6, redline the last 10%, a 2 px track.
    fn default() -> Self {
        Self {
            width_start: 6.0,
            width_end: 14.0,
            power: 1.6,
            redline: 0.10,
            track_width: 2.0,
            gain: 1.0,
        }
    }
}

/// A path ribbon with its fill spring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathRibbon {
    pts: [Vec2; RIBBON_POINTS],
    cum: [f32; RIBBON_POINTS],
    fill: Spring,
    target: f32,
    /// The style.
    pub style: RibbonStyle,
}

impl PathRibbon {
    fn from_fn(tokens: &Tokens, style: RibbonStyle, f: impl Fn(f32) -> Vec2) -> Self {
        let mut pts = [Vec2::ZERO; RIBBON_POINTS];
        for (i, p) in pts.iter_mut().enumerate() {
            *p = f(i as f32 / (RIBBON_POINTS - 1) as f32);
        }
        let mut cum = [0.0; RIBBON_POINTS];
        for i in 1..RIBBON_POINTS {
            cum[i] = cum[i - 1] + (pts[i] - pts[i - 1]).length();
        }
        Self {
            pts,
            cum,
            fill: Spring::new(tokens.springs.ribbon, 0.0),
            target: 0.0,
            style,
        }
    }

    /// A circular arch around `center` from `start` over `sweep` (radians, clockwise from 3
    /// o'clock), e.g. the 270° tach from `deg(135.0)`.
    #[must_use]
    pub fn arch(tokens: &Tokens, center: Vec2, radius: f32, start: f32, sweep: f32, style: RibbonStyle) -> Self {
        Self::from_fn(tokens, style, |t| Vec2::polar(center, radius, start + sweep * t))
    }

    /// A light pipe inset `inset` px inside a squircle outline (`center`, full `size`,
    /// exponent `n`), running from outline angle `from` to `to` (radians, clockwise from 3
    /// o'clock; `to` may exceed `from` by more than π, e.g. lower-left `deg(150)` over the crown
    /// to lower-right `deg(390)`).
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn squircle_rim(
        tokens: &Tokens,
        center: Vec2,
        size: Vec2,
        n: f32,
        inset: f32,
        from: f32,
        to: f32,
        style: RibbonStyle,
    ) -> Self {
        // Sample densely in angle, then resample by arc length so the samples are even.
        let mut dense = [Vec2::ZERO; 4 * RIBBON_POINTS];
        let m = dense.len();
        for (i, p) in dense.iter_mut().enumerate() {
            let t = i as f32 / (m - 1) as f32;
            *p = squircle_point(center, size, n, from + (to - from) * t, inset);
        }
        let mut dcum = [0.0f32; 4 * RIBBON_POINTS];
        for i in 1..m {
            dcum[i] = dcum[i - 1] + (dense[i] - dense[i - 1]).length();
        }
        let total = dcum[m - 1].max(1e-3);
        Self::from_fn(tokens, style, |t| {
            let s = t * total;
            let i = dcum.partition_point(|&c| c < s).clamp(1, m - 1);
            let seg = (dcum[i] - dcum[i - 1]).max(1e-6);
            dense[i - 1].lerp(dense[i], ((s - dcum[i - 1]) / seg).clamp(0.0, 1.0))
        })
    }

    /// Total path length (local px).
    #[must_use]
    pub fn length(&self) -> f32 {
        self.cum[RIBBON_POINTS - 1]
    }

    /// Point at fraction `t` (0..1) of the length.
    #[must_use]
    pub fn point_at(&self, t: f32) -> Vec2 {
        let s = t.clamp(0.0, 1.0) * self.length();
        let i = self.cum.partition_point(|&c| c < s).clamp(1, RIBBON_POINTS - 1);
        let seg = (self.cum[i] - self.cum[i - 1]).max(1e-6);
        self.pts[i - 1].lerp(self.pts[i], ((s - self.cum[i - 1]) / seg).clamp(0.0, 1.0))
    }

    /// Unit tangent (direction of travel) at fraction `t`.
    #[must_use]
    pub fn tangent_at(&self, t: f32) -> Vec2 {
        let a = self.point_at((t - 0.004).max(0.0));
        let b = self.point_at((t + 0.004).min(1.0));
        (b - a).normalize_or_zero()
    }

    /// Width at fraction `t`.
    #[must_use]
    pub fn width_at(&self, t: f32) -> f32 {
        let s = &self.style;
        s.width_start + (s.width_end - s.width_start) * t.clamp(0.0, 1.0).powf(s.power.max(0.1))
    }

    /// Advances the fill towards `fraction` (0..1) with the rate `rate` (fraction per second)
    /// as feed-forward.
    pub fn update(&mut self, fraction: f32, rate: f32, dt: f32) {
        self.target = fraction.clamp(0.0, 1.0);
        self.fill.update_tracking(self.target, rate, dt);
    }

    /// Snaps the fill (no motion), e.g. for a hood/cockpit switch.
    pub fn snap(&mut self, fraction: f32) {
        self.target = fraction.clamp(0.0, 1.0);
        self.fill.reset(self.target);
    }

    /// The shown fill fraction (spring value, clamped).
    #[must_use]
    pub fn fraction(&self) -> f32 {
        self.fill.value().clamp(0.0, 1.0)
    }

    /// True while the fill spring is still moving.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.fill.settled(self.target, 1e-4)
    }

    /// The fill colour at `t` (0..1 along the path): primary → a whiter primary towards the
    /// redline, then the warning role over the redline share. `heat` (0..1) warms every stop
    /// towards the shift colour (the steady limiter ramp).
    #[must_use]
    pub fn color_at(&self, t: f32, palette: &Palette, heat: f32) -> Color {
        let red_from = 1.0 - self.style.redline.clamp(0.0, 0.5);
        if t >= red_from {
            // The redline share stays in the warning role at any heat.
            return dim(palette.roles.warning, self.style.gain);
        }
        let k = (t / red_from.max(1e-3)).clamp(0.0, 1.0);
        let base = palette
            .primary
            .lerp(palette.primary.lerp(palette.roles.ink, 0.55), k * k);
        dim(base.lerp(palette.shift, heat.clamp(0.0, 1.0) * 0.6), self.style.gain)
    }

    /// Draws the unlit track (a hairline the full length), with the redline share faintly in
    /// the warning role.
    ///
    /// The path is drawn in short chunks: a polyline's distance field is evaluated over its
    /// whole bounding box for every segment, so one 128-segment polyline spanning a 1240 px
    /// capsule cost about 0.36 ms of GPU at 4K; 16 chunks with small boxes cost almost nothing.
    pub fn draw_track(&self, p: &mut Painter<'_>, palette: &Palette) {
        let w = self.style.track_width;
        if w <= 0.0 {
            return;
        }
        let red_from = 1.0 - self.style.redline;
        let red_i = self
            .cum
            .partition_point(|&c| c < red_from * self.length())
            .min(RIBBON_POINTS - 1);
        let mut i = 0;
        while i < RIBBON_POINTS - 1 {
            let in_red = i >= red_i;
            // A chunk never straddles the redline boundary.
            let limit = if in_red { RIBBON_POINTS - 1 } else { red_i.max(i + 1) };
            let end = (i + TRACK_CHUNK).min(limit).min(RIBBON_POINTS - 1);
            p.fill(
                &Shape::Polyline {
                    points: &self.pts[i..=end],
                    width: w,
                    closed: false,
                },
                if in_red {
                    palette.roles.warning.with_alpha(0.35)
                } else {
                    palette.roles.ink_ghost
                },
            );
            i = end;
        }
    }

    /// Draws the lit fill from 0 to `fraction` as tapered segments with per-segment gradients
    /// (seamless colour along a curved path). Call inside a bloom layer when the ribbon is a
    /// rank-3 element.
    pub fn draw_fill(&self, p: &mut Painter<'_>, fraction: f32, palette: &Palette, heat: f32) {
        let f = fraction.clamp(0.0, 1.0);
        if f <= 0.0005 {
            return;
        }
        let end = f * self.length();
        for i in 1..RIBBON_POINTS {
            let s0 = self.cum[i - 1];
            if s0 >= end {
                break;
            }
            let s1 = self.cum[i].min(end);
            let total = self.length().max(1e-3);
            let (t0, t1) = (s0 / total, s1 / total);
            let a = self.pts[i - 1];
            let seg = (self.cum[i] - s0).max(1e-6);
            let b = a.lerp(self.pts[i], ((s1 - s0) / seg).clamp(0.0, 1.0));
            let (c0, c1) = (self.color_at(t0, palette, heat), self.color_at(t1, palette, heat));
            p.fill(
                &Shape::Line {
                    a,
                    b,
                    width_a: self.width_at(t0),
                    width_b: self.width_at(t1),
                    cap: Cap::Round,
                },
                Gradient::linear(a, b).stop(0.0, c0).stop(1.0, c1),
            );
        }
    }

    /// Draws a meniscus bead at fraction `t`: a liquid orb (white core → accent → primary)
    /// fused to the ribbon by a short tapered neck (a smooth union in look), with a specular
    /// dot facing the light (`light_dir` points towards the light). `core` is the core radius
    /// (tokens: 16 → 22 px with throttle), `hot` (0..1) is the limiter's hot meniscus (white core,
    /// warning halo; steady, never flashing).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_bead(
        &self,
        p: &mut Painter<'_>,
        t: f32,
        radius: f32,
        core: f32,
        light_dir: Vec2,
        palette: &Palette,
        intensity: f32,
        hot: f32,
    ) {
        let tip = self.point_at(t);
        let dir = self.tangent_at(t);
        // The neck only reaches back along the lit part of the path (none at the very start).
        let back = (radius * 1.6).min(t.clamp(0.0, 1.0) * self.length());
        let neck_from = tip - dir * back;
        let w_neck = self.width_at(t);
        let roles = palette.roles;
        let hot = hot.clamp(0.0, 1.0);
        let halo = palette.primary.lerp(roles.warning, hot);
        let i = intensity.clamp(0.0, 1.0);
        p.layer(LayerOptions::bloom(), |p| {
            // Neck: the ribbon swelling into the bead.
            if back > 2.0 {
                p.fill(
                    &Shape::Line {
                        a: neck_from,
                        b: tip,
                        width_a: w_neck,
                        width_b: radius * 1.5,
                        cap: Cap::Round,
                    },
                    Gradient::linear(neck_from, tip)
                        .stop(0.0, self.color_at(t, palette, 0.0))
                        .stop(1.0, halo.lerp(roles.ink, 0.3)),
                );
            }
            p.fill(
                &Shape::circle(tip, radius),
                Gradient::radial(tip, radius)
                    .stop(0.0, roles.ink)
                    .stop(
                        (core / radius.max(1.0)).clamp(0.1, 0.95) * 0.5,
                        roles.ink.lerp(palette.accent, 0.35),
                    )
                    .stop(0.72, halo.with_alpha(0.7 * (0.35 + 0.65 * i)))
                    .stop(1.0, halo.with_alpha(0.0)),
            );
        });
        // Only the hot meniscus gets an extra halo; otherwise the bloom of the core is enough
        // (a soft primary glow outside the glass reads as a dark smudge on OLED black).
        if hot > 0.01 {
            p.glow(&Shape::circle(tip, core * 0.5), halo, Glow::new(8.0, 0.45 * hot));
        }
        // Specular: a tiny highlight offset towards the light.
        let spec = tip + light_dir * (radius * 0.42);
        p.fill(&Shape::circle(spec, radius * 0.16), roles.rim_specular);
    }
}

/// Darkens `c` by `gain` (0..1) but keeps its alpha, so overlapping round caps of opaque segments
/// never show as dots.
fn dim(c: Color, gain: f32) -> Color {
    let g = gain.clamp(0.0, 1.0);
    Color {
        r: c.r * g,
        g: c.g * g,
        b: c.b * g,
        a: c.a,
    }
}

/// Default sweep for an arc tach that starts at 7:30 (`deg(135)`) and runs 270°.
#[must_use]
pub fn arch_270() -> (f32, f32) {
    (deg(135.0), deg(270.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::{vec2, Palette, Tokens};

    #[test]
    fn arch_length_and_points() {
        let t = Tokens::default();
        let (s, w) = arch_270();
        let r = PathRibbon::arch(&t, Vec2::ZERO, 200.0, s, w, RibbonStyle::default());
        let expected = 200.0 * w;
        assert!((r.length() - expected).abs() / expected < 0.002);
        let start = r.point_at(0.0);
        assert!((start - Vec2::polar(Vec2::ZERO, 200.0, s)).length() < 0.5);
        let mid = r.point_at(0.5);
        assert!((mid - vec2(0.0, -200.0)).length() < 1.0, "{mid:?}");
    }

    #[test]
    fn squircle_rim_is_evenly_sampled() {
        let t = Tokens::default();
        let r = PathRibbon::squircle_rim(
            &t,
            Vec2::ZERO,
            vec2(1240.0, 300.0),
            4.0,
            14.0,
            deg(150.0),
            deg(390.0),
            RibbonStyle::default(),
        );
        let step = r.length() / (RIBBON_POINTS - 1) as f32;
        for i in 1..RIBBON_POINTS {
            let d = r.cum[i] - r.cum[i - 1];
            assert!((d - step).abs() < step * 0.25, "uneven step {d} vs {step}");
        }
        // Over the crown: the middle of the pipe is at the top.
        assert!(r.point_at(0.5).y < -100.0);
    }

    #[test]
    fn fill_tracks_with_feed_forward_and_settles() {
        let t = Tokens::default();
        let mut r = PathRibbon::arch(&t, Vec2::ZERO, 200.0, 0.0, 3.0, RibbonStyle::default());
        for _ in 0..60 {
            r.update(0.6, 0.0, 1.0 / 60.0);
        }
        assert!((r.fraction() - 0.6).abs() < 0.01 && !r.animating());
        let pal = Palette::family(&t, "liquidGlass");
        assert_eq!(r.color_at(0.95, &pal, 0.0), pal.roles.warning);
        assert!(r.width_at(1.0) > r.width_at(0.0));
    }
}
