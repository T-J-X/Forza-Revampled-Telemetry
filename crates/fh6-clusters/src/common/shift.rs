//! ShiftLights pod optics and variants (COMPONENTS.md §4) and the RevLimiterStrobe behaviour (§5).
//!
//! [`super::ShiftLights`] owns the logic (outside-in fill onto the centre pod, 0 ms attack,
//! ≤ 50 ms release, the 10 Hz limiter strobe on the strip only, the reduced-flash breath). This
//! module adds the **optics** every family shares, tokens `components.shiftLights.pod`:
//! - lit: a 2-stop radial (centre 40% toward white, edge full hue), a 1 px inner rim at 50%
//!   white, and a glow; it writes to the bloom buffer (rank 1);
//! - unlit: a grey lens, white at alpha ≤ 22 with a 12% rim. Unlit pods are never coloured.
//!
//! Variants are layouts of the same pod: `glass16` pills in a strip, `race10` round-rects,
//! `analog15` jewels on an arc, `modern11` converging diamonds, `beads11` droplets and `vfdStrip`
//! (use the segmented [`fh6_kit::SegmentedBar`] for that one).
use super::ShiftLights;
use fh6_kit::{vec2, Cap, Color, Glow, Gradient, LayerOptions, Painter, Palette, Rect, Shape, Stroke, Vec2};

/// The lens shape of a pod.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PodShape {
    /// A pill (radius = half the height): Liquid Glass strips.
    #[default]
    Pill,
    /// A rounded rectangle (radius 35% of the height): Race Dash LED pods.
    RoundRect,
    /// A round jewel: analog dials.
    Jewel,
    /// A diamond (rotated square): Modern Digital converging pairs.
    Diamond,
}

/// How pods are drawn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PodStyle {
    /// Lens shape.
    pub shape: PodShape,
    /// Glow sigma (local px) of a lit pod.
    pub glow_sigma: f32,
    /// Glow intensity of a lit pod.
    pub glow: f32,
    /// Unlit lens alpha (tokens: ≤ 22 of 255). Colours blend in linear light, so 3% reads as
    /// about 19% grey on screen.
    pub unlit_alpha: f32,
    /// Alpha of the unlit lens's 1 px rim (linear; 0.12 reads as about 38% grey).
    pub unlit_rim: f32,
}

impl Default for PodStyle {
    fn default() -> Self {
        Self {
            shape: PodShape::Pill,
            glow_sigma: 10.0,
            glow: 0.55,
            unlit_alpha: 22.0 / 255.0,
            unlit_rim: 0.12,
        }
    }
}

impl PodStyle {
    /// The style with `shape`.
    #[must_use]
    pub fn with_shape(mut self, shape: PodShape) -> Self {
        self.shape = shape;
        self
    }
}

fn pod_shape(rect: Rect, shape: PodShape, pts: &mut [Vec2; 4]) -> Shape<'_> {
    match shape {
        PodShape::Pill => Shape::round_rect(rect, rect.height().min(rect.width()) * 0.5),
        PodShape::RoundRect => Shape::round_rect(rect, rect.height() * 0.35),
        PodShape::Jewel => Shape::circle(rect.center(), rect.width().min(rect.height()) * 0.5),
        PodShape::Diamond => {
            let c = rect.center();
            let (hw, hh) = (rect.width() * 0.5, rect.height() * 0.5);
            *pts = [
                vec2(c.x, c.y - hh),
                vec2(c.x + hw, c.y),
                vec2(c.x, c.y + hh),
                vec2(c.x - hw, c.y),
            ];
            Shape::Polygon { points: &pts[..] }
        }
    }
}

/// Draws one pod lens in `rect`: lit with `color` (bloomed optics) or unlit (`None`, a grey lens).
pub fn draw_pod(p: &mut Painter<'_>, rect: Rect, color: Option<Color>, style: &PodStyle, palette: &Palette) {
    let mut pts = [Vec2::ZERO; 4];
    let shape = pod_shape(rect, style.shape, &mut pts);
    let ink = palette.roles.ink;
    match color {
        Some(c) => {
            let center = rect.center();
            let r = rect.width().max(rect.height()) * 0.5;
            let core = c.lerp(ink.with_alpha(c.a), 0.4);
            p.glow(&shape, c, Glow::new(style.glow_sigma, style.glow));
            p.layer(LayerOptions::bloom(), |p| {
                p.fill(
                    &shape,
                    Gradient::radial(center, r).stop(0.0, core).stop(0.55, c).stop(1.0, c),
                );
            });
            p.stroke(&shape, ink.with_alpha(0.5), Stroke::inner(1.0));
        }
        None => {
            p.fill(&shape, ink.with_alpha(style.unlit_alpha));
            p.stroke(&shape, ink.with_alpha(style.unlit_rim), Stroke::rim_px(1.0));
        }
    }
}

impl ShiftLights {
    /// The fill level (0..1 of the strip, before the strobe).
    #[must_use]
    pub fn level(&self) -> f32 {
        self.level
    }

    /// True at or above the shift point (all pods take the shift colour and strobe).
    #[must_use]
    pub fn over_shift(&self) -> bool {
        self.over_shift
    }

    /// The colour of pod `i` this frame (`None` when unlit or strobed off).
    #[must_use]
    pub fn color_of(&self, i: usize, palette: &Palette) -> Option<Color> {
        self.pod_color(i, palette)
    }

    /// Draws the pods as a strip across `rect` (gap 30% of the pod width) with the shared optics.
    /// Keep `rect` ≤ 42 px tall: it is the only element that strobes.
    pub fn draw_strip(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette, style: &PodStyle) {
        let n = self.count as usize;
        // n w + (n - 1) 0.3 w = width.
        let w = rect.width() / (n as f32 + 0.3 * (n as f32 - 1.0));
        let gap = 0.3 * w;
        for i in 0..n {
            let pod = Rect::from_xywh(rect.min.x + i as f32 * (w + gap), rect.min.y, w, rect.height());
            draw_pod(p, pod, self.pod_color(i, palette), style, palette);
        }
    }

    /// Draws the pods centred on `centers` (one per pod, outermost first on the left), each
    /// `size` big: jewels on an arc, converging diamond pairs, beads on a path.
    pub fn draw_at(&self, p: &mut Painter<'_>, centers: &[Vec2], size: Vec2, palette: &Palette, style: &PodStyle) {
        for (i, c) in centers.iter().enumerate().take(self.count as usize) {
            let pod = Rect::from_center_size(*c, size);
            draw_pod(p, pod, self.pod_color(i, palette), style, palette);
        }
    }

    /// Fills `out` with `count` pod centres on an arc (analog jewels above a dial): around
    /// `center` at `radius`, from `start` over `sweep` (radians, clockwise from 3 o'clock).
    /// Returns how many were written.
    pub fn arc_centers(&self, center: Vec2, radius: f32, start: f32, sweep: f32, out: &mut [Vec2]) -> usize {
        let n = (self.count as usize).min(out.len());
        for (i, o) in out.iter_mut().enumerate().take(n) {
            let t = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.5 };
            *o = Vec2::polar(center, radius, start + sweep * t);
        }
        n
    }
}

/// The family's one steady limiter cue (§5): a 0..1 ramp that rises over `ramp_s` seconds once
/// the shift point is reached and falls back as fast when it is left. It never flashes; use it
/// to warm a rim, a meniscus or a wash.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LimiterCue {
    level: f32,
    held: f32,
}

impl LimiterCue {
    /// At rest.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances: `over` is true while at or above the shift point.
    pub fn update(&mut self, over: bool, ramp_s: f32, dt: f32) {
        let step = dt.max(0.0) / ramp_s.max(1e-3);
        if over {
            self.level = (self.level + step).min(1.0);
            self.held += dt.max(0.0);
        } else {
            self.level = (self.level - step * 2.0).max(0.0);
            self.held = 0.0;
        }
    }

    /// The steady cue level (0..1).
    #[must_use]
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Seconds spent continuously on the limiter (the 2 Hz gear pulse may start after 0.5 s).
    #[must_use]
    pub fn held(&self) -> f32 {
        self.held
    }

    /// True while the ramp is moving.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.level > 0.0 && self.level < 1.0
    }
}

/// Draws a capsule-shaped strip track under a pod row (a faint rail so the unlit strip still
/// reads as hardware). Optional.
pub fn draw_rail(p: &mut Painter<'_>, rect: Rect, palette: &Palette) {
    let y = rect.center().y;
    p.fill(
        &Shape::Line {
            a: vec2(rect.min.x, y),
            b: vec2(rect.max.x, y),
            width_a: 1.0,
            width_b: 1.0,
            cap: Cap::Round,
        },
        palette.roles.hairline.fade(0.6),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::{Signals, Tokens};

    #[test]
    fn strip_lights_outside_in_and_takes_the_shift_colour() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "liquidGlass");
        let mut s = ShiftLights::new(15);
        let sig = Signals {
            shift_fraction: 0.97,
            ..Signals::default()
        };
        s.update(&sig, &pal, 0.0, 1.0 / 60.0, false);
        assert!(s.color_of(0, &pal).is_some(), "outermost lit first");
        assert!(s.color_of(14, &pal).is_some());
        let over = Signals {
            shift_fraction: 1.02,
            ..Signals::default()
        };
        s.update(&over, &pal, 0.0, 1.0 / 60.0, false);
        assert!(s.over_shift());
        assert_eq!(s.color_of(7, &pal), Some(pal.shift));
    }

    #[test]
    fn limiter_cue_ramps_and_holds() {
        let mut c = LimiterCue::new();
        for _ in 0..30 {
            c.update(true, 0.25, 1.0 / 60.0);
        }
        assert!((c.level() - 1.0).abs() < 1e-6 && c.held() > 0.45);
        c.update(false, 0.25, 1.0 / 60.0);
        assert!(c.level() < 1.0 && c.held() == 0.0);
    }

    #[test]
    fn arc_centres_span_the_sweep() {
        let s = ShiftLights::new(15);
        let mut out = [Vec2::ZERO; 15];
        let n = s.arc_centers(Vec2::ZERO, 100.0, 0.0, std::f32::consts::PI, &mut out);
        assert_eq!(n, 15);
        assert!((out[0] - vec2(100.0, 0.0)).length() < 1e-3);
        assert!((out[14] - vec2(-100.0, 0.0)).length() < 1e-2);
    }
}
