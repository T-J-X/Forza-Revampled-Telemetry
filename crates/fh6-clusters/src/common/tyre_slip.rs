//! TyreSlipQuad (COMPONENTS.md §12, tokens `components.tyreSlipQuad`): four tyres in a car
//! footprint.
//!
//! Each tyre is filled with a discrete step colour (never a continuous hue sweep), crossfaded
//! over `motion.durations.crossfade`: the thermal step when temperatures are supplied (tokens
//! `colorRoles.thermal`: cold < 70 °C, optimal < 100, hot < 112, overheated), otherwise the slip
//! step. Both ramps use the palette roles of `slipRamp`: `primary` -> `go` -> `caution` ->
//! `warning`. A vertical sheen shades the tread, a 1 px top-lit rim edges it, and a slip outline
//! appears in `caution` above a 0.08 slip ratio or `warning` above 0.18. Rank 4: no glow.
//!
//! **Telemetry honesty:** tyre slip and temperature are not in `tests/golden/packets`. The
//! caller supplies them and must verify their units (slip ratio magnitude, 0 = rolling; °C)
//! against real packets before wiring them to telemetry; this component never invents them.
use fh6_kit::math::finite_or;
use fh6_kit::{
    vec2, Align, Color, FontId, Gradient, Hysteresis, Painter, Palette, Radii, Rect, Shape, Stroke, TextStyle, Tokens,
    VAlign, Vec2,
};

/// Tyres in the order every array of this module uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Corner {
    /// Front left (index 0).
    FrontLeft,
    /// Front right (index 1).
    FrontRight,
    /// Rear left (index 2).
    RearLeft,
    /// Rear right (index 3).
    RearRight,
}

impl Corner {
    /// All four, in array order.
    pub const ALL: [Corner; 4] = [Corner::FrontLeft, Corner::FrontRight, Corner::RearLeft, Corner::RearRight];

    /// The array index.
    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }

    fn is_left(self) -> bool {
        matches!(self, Corner::FrontLeft | Corner::RearLeft)
    }

    fn is_front(self) -> bool {
        matches!(self, Corner::FrontLeft | Corner::FrontRight)
    }
}

/// Thermal step edges in °C (tokens `colorRoles.thermal.steps`: cold, optimal, hot, overheated).
pub const THERMAL_EDGES_C: [f32; 3] = [70.0, 100.0, 112.0];
/// Slip-ratio step edges for the fill without temperatures: grip, working band, sliding, lost.
pub const SLIP_EDGES: [f32; 3] = [0.04, 0.08, 0.18];
/// Slip outline edges (COMPONENTS.md §12): `caution` above 0.08, `warning` above 0.18.
pub const OUTLINE_EDGES: [f32; 2] = [0.08, 0.18];
/// A step only falls back once the value clears its edge by this much (°C).
const THERMAL_HYSTERESIS_C: f32 = 2.0;
/// A step only falls back once the value clears its edge by this much (slip ratio).
const SLIP_HYSTERESIS: f32 = 0.01;

/// One of the four discrete fill steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TyreStep {
    /// Cold / full grip: `primary`.
    #[default]
    Cool,
    /// Optimal / working slip: `go`.
    Optimal,
    /// Hot / sliding: `caution`.
    Hot,
    /// Overheated / grip lost: `warning`.
    Critical,
}

impl TyreStep {
    const ALL: [TyreStep; 4] = [TyreStep::Cool, TyreStep::Optimal, TyreStep::Hot, TyreStep::Critical];

    fn from_index(i: usize) -> Self {
        Self::ALL[i.min(3)]
    }

    /// The step colour from the palette (tokens `slipRamp` intent).
    #[must_use]
    pub fn color(self, palette: &Palette) -> Color {
        match self {
            TyreStep::Cool => palette.primary,
            TyreStep::Optimal => palette.roles.go,
            TyreStep::Hot => palette.roles.caution,
            TyreStep::Critical => palette.roles.warning,
        }
    }

    /// The thermal step for `celsius` with no history.
    #[must_use]
    pub fn from_celsius(celsius: f32) -> Self {
        Self::from_index(stepped(0, celsius, &THERMAL_EDGES_C, 0.0))
    }

    /// The slip step for a slip ratio (sign ignored) with no history.
    #[must_use]
    pub fn from_slip(slip: f32) -> Self {
        Self::from_index(stepped(0, slip.abs(), &SLIP_EDGES, 0.0))
    }
}

/// The slip outline state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SlipOutline {
    /// No outline.
    #[default]
    None,
    /// `caution` outline (slip ratio above 0.08).
    Caution,
    /// `warning` outline (slip ratio above 0.18).
    Warning,
}

/// The step index for `value` given the current step: rises at once, falls only after clearing
/// the edge by `hysteresis` (so a noisy channel never flickers between two colours).
fn stepped<const N: usize>(current: usize, value: f32, edges: &[f32; N], hysteresis: f32) -> usize {
    let up = edges.iter().filter(|e| value >= **e).count();
    if up >= current {
        return up;
    }
    edges.iter().filter(|e| value >= **e - hysteresis).count().min(current)
}

/// The four-tyre quad: per-tyre fill step with a crossfade, slip outline and shown temperature.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TyreSlipQuad {
    step: [usize; 4],
    from: [usize; 4],
    mix: [f32; 4],
    outline: [SlipOutline; 4],
    temps: [Hysteresis; 4],
    shown_temp: Option<[i64; 4]>,
    crossfade: f32,
}

impl TyreSlipQuad {
    /// A quad at rest (cool, no outline), crossfading over the tokens' `crossfade` duration.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            step: [0; 4],
            from: [0; 4],
            mix: [1.0; 4],
            outline: [SlipOutline::None; 4],
            temps: [Hysteresis::new(0.0, 0.6); 4],
            shown_temp: None,
            crossfade: tokens.durations.crossfade.max(1e-3),
        }
    }

    /// Advances with four caller-supplied slip ratios (FL, FR, RL, RR; sign ignored) and optional
    /// temperatures in °C. With temperatures the fill is thermal, without it is the slip step.
    /// The caller verifies the telemetry units first (see the module docs).
    pub fn update(&mut self, slip: [f32; 4], temps_c: Option<[f32; 4]>, dt: f32) {
        let dt = finite_or(dt, 0.0).max(0.0);
        let mut shown = [0i64; 4];
        for (i, raw) in slip.iter().enumerate() {
            let s = finite_or(*raw, 0.0).abs();
            let next = match temps_c {
                Some(t) if t[i].is_finite() => {
                    shown[i] = self.temps[i].update(t[i]);
                    stepped(self.step[i], t[i], &THERMAL_EDGES_C, THERMAL_HYSTERESIS_C)
                }
                // A non-finite temperature keeps the last step and shown value.
                Some(_) => {
                    shown[i] = self.shown_temp.map_or(0, |v| v[i]);
                    self.step[i]
                }
                None => stepped(self.step[i], s, &SLIP_EDGES, SLIP_HYSTERESIS),
            };
            if next != self.step[i] {
                self.from[i] = self.step[i];
                self.step[i] = next;
                self.mix[i] = 0.0;
            }
            self.mix[i] = (self.mix[i] + dt / self.crossfade).min(1.0);
            let level = match self.outline[i] {
                SlipOutline::None => 0,
                SlipOutline::Caution => 1,
                SlipOutline::Warning => 2,
            };
            self.outline[i] = match stepped(level, s, &OUTLINE_EDGES, SLIP_HYSTERESIS) {
                0 => SlipOutline::None,
                1 => SlipOutline::Caution,
                _ => SlipOutline::Warning,
            };
        }
        self.shown_temp = temps_c.map(|_| shown);
    }

    /// The current fill step of `corner`.
    #[must_use]
    pub fn step(&self, corner: Corner) -> TyreStep {
        TyreStep::from_index(self.step[corner.index()])
    }

    /// The slip outline of `corner`.
    #[must_use]
    pub fn outline(&self, corner: Corner) -> SlipOutline {
        self.outline[corner.index()]
    }

    /// The temperature shown beside `corner` (whole °C), when temperatures are supplied.
    #[must_use]
    pub fn shown_temp(&self, corner: Corner) -> Option<i64> {
        self.shown_temp.map(|t| t[corner.index()])
    }

    /// True while a step crossfade runs.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.mix.iter().any(|m| *m < 1.0)
    }

    /// The crossfaded fill colour of tyre `i`.
    fn fill(&self, i: usize, palette: &Palette) -> Color {
        let from = TyreStep::from_index(self.from[i]).color(palette);
        let to = TyreStep::from_index(self.step[i]).color(palette);
        from.lerp(to, self.mix[i])
    }

    /// Draws the quad in `rect` (the footprint: tyres at its corners, the body outline around
    /// them, 3% taller than `rect`).
    /// Temperatures, when supplied, are set outside `rect` beside each tyre in `font` at
    /// `metricSecondary` (allow about 2.4 caps of margin on both sides).
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette, font: FontId) {
        let roles = palette.roles;
        let w = rect.width();
        let h = rect.height();
        let tyre = vec2(w * 0.24, h * 0.30);
        let (y0, y1) = (rect.min.y, rect.max.y);

        // Body: a hairline car footprint around the wheels, rounder at the nose, lit from above;
        // the wheels sit proud of its sides and cut the line (see `draw_tyre`).
        let inset = tyre.x * 0.40;
        let body = Rect::from_xywh(rect.min.x + inset, y0 - h * 0.03, w - 2.0 * inset, h * 1.06);
        let bw = body.width();
        p.stroke(
            &Shape::round_rect(body, Radii::new(bw * 0.42, bw * 0.42, bw * 0.22, bw * 0.22)),
            Gradient::linear(body.at(0.5, 0.0), body.at(0.5, 1.0))
                .stop(0.0, roles.hairline.lerp(roles.rim_specular, 0.25))
                .stop(0.5, roles.hairline)
                .stop(1.0, roles.hairline.fade(0.55)),
            Stroke::rim_px(1.0),
        );

        let ts = p.tokens().type_scale;
        let temp_style = TextStyle::new(font, ts.metric_secondary).tabular();
        for corner in Corner::ALL {
            let i = corner.index();
            let x = if corner.is_left() { rect.min.x } else { rect.max.x - tyre.x };
            let y = if corner.is_front() {
                y0 + h * 0.20 - tyre.y * 0.5
            } else {
                y1 - h * 0.20 - tyre.y * 0.5
            };
            let tr = Rect::from_xywh(x, y, tyre.x, tyre.y);
            self.draw_tyre(p, tr, i, palette);

            if let Some(temps) = self.shown_temp {
                let step = TyreStep::from_index(self.step[i]);
                // Colour by exception: neutral until hot.
                let ink = match step {
                    TyreStep::Hot | TyreStep::Critical => step.color(palette).text_safe(),
                    _ => roles.ink_muted,
                };
                let align = if corner.is_left() { Align::Right } else { Align::Left };
                let style = temp_style.paint(ink).align(align, VAlign::CapMiddle);
                let pos = temp_anchor(corner, tr, ts.metric_secondary);
                p.number(pos, temps[i].clamp(-99, 999), 3, &style, None);
            }
        }
    }

    fn draw_tyre(&self, p: &mut Painter<'_>, tr: Rect, i: usize, palette: &Palette) {
        let roles = palette.roles;
        let r = tr.width() * 0.24;
        let shape = Shape::round_rect(tr, Radii::all(r));
        let c = self.fill(i, palette);
        // Wheel arch: OLED black around the tyre cuts the body line, leaving a little air.
        let air = tr.width() * 0.14;
        p.fill(
            &Shape::round_rect(tr.expand(air), Radii::all(r + air)),
            roles.oled_black,
        );
        // Vertical sheen: the tread curves away front and back, lit from above.
        p.fill(
            &shape,
            Gradient::linear(tr.at(0.5, 0.0), tr.at(0.5, 1.0))
                .stop(0.0, c.fade(0.50))
                .stop(0.24, c.lerp(roles.ink, 0.28))
                .stop(0.52, c.fade(0.88))
                .stop(1.0, c.fade(0.34)),
        );
        // Two circumferential grooves give the tread its material.
        let groove_w = (tr.width() * 0.035).max(p.device_px());
        for u in [0.36f32, 0.64] {
            let gx = tr.min.x + tr.width() * u;
            p.fill(
                &Shape::line(vec2(gx, tr.min.y + r * 0.6), vec2(gx, tr.max.y - r * 0.6), groove_w),
                roles.oled_black.with_alpha(0.42),
            );
        }
        p.stroke(
            &shape,
            Gradient::linear(tr.at(0.5, 0.0), tr.at(0.5, 1.0))
                .stop(0.0, roles.rim_specular.fade(0.8))
                .stop(0.45, roles.hairline)
                .stop(1.0, roles.hairline.fade(0.5)),
            Stroke::rim_px(1.0),
        );
        let outline = match self.outline[i] {
            SlipOutline::None => None,
            SlipOutline::Caution => Some(roles.caution),
            SlipOutline::Warning => Some(roles.warning),
        };
        if let Some(color) = outline {
            let ring = tr.expand(tr.width() * 0.09);
            let width = (tr.width() * 0.045).max(2.0 * p.device_px());
            p.stroke(
                &Shape::round_rect(ring, Radii::all(r + tr.width() * 0.09)),
                color,
                Stroke::new(width),
            );
        }
    }
}

/// Where a temperature label of `corner` would sit for a tyre rect `tyre` (helper for clusters
/// that lay out other parts around the quad).
#[must_use]
pub fn temp_anchor(corner: Corner, tyre: Rect, cap: f32) -> Vec2 {
    let room = cap * 0.6;
    if corner.is_left() {
        vec2(tyre.min.x - room, tyre.center().y)
    } else {
        vec2(tyre.max.x + room, tyre.center().y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn thermal_steps_follow_the_tokens() {
        assert_eq!(TyreStep::from_celsius(45.0), TyreStep::Cool);
        assert_eq!(TyreStep::from_celsius(70.0), TyreStep::Optimal);
        assert_eq!(TyreStep::from_celsius(105.0), TyreStep::Hot);
        assert_eq!(TyreStep::from_celsius(112.0), TyreStep::Critical);
        assert_eq!(TyreStep::from_slip(-0.2), TyreStep::Critical);
        assert_eq!(TyreStep::from_slip(0.05), TyreStep::Optimal);
    }

    #[test]
    fn steps_rise_at_once_and_fall_with_hysteresis() {
        assert_eq!(stepped(0, 101.0, &THERMAL_EDGES_C, 2.0), 2);
        // Just under the edge: hold the hot step.
        assert_eq!(stepped(2, 99.0, &THERMAL_EDGES_C, 2.0), 2);
        // Clear of the edge: fall.
        assert_eq!(stepped(2, 97.5, &THERMAL_EDGES_C, 2.0), 1);
        assert_eq!(stepped(3, 10.0, &THERMAL_EDGES_C, 2.0), 0);
    }

    #[test]
    fn colours_come_from_palette_roles() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "modernDigital");
        assert_eq!(TyreStep::Cool.color(&pal), pal.primary);
        assert_eq!(TyreStep::Optimal.color(&pal), pal.roles.go);
        assert_eq!(TyreStep::Hot.color(&pal), pal.roles.caution);
        assert_eq!(TyreStep::Critical.color(&pal), pal.roles.warning);
    }

    #[test]
    fn slip_outline_thresholds() {
        let t = Tokens::default();
        let mut q = TyreSlipQuad::new(&t);
        q.update([0.02, 0.09, 0.19, -0.25], None, DT);
        assert_eq!(q.outline(Corner::FrontLeft), SlipOutline::None);
        assert_eq!(q.outline(Corner::FrontRight), SlipOutline::Caution);
        assert_eq!(q.outline(Corner::RearLeft), SlipOutline::Warning);
        assert_eq!(q.outline(Corner::RearRight), SlipOutline::Warning, "sign ignored");
        // 0.075 is under the caution edge but within the hysteresis: the outline holds.
        q.update([0.02, 0.075, 0.19, 0.25], None, DT);
        assert_eq!(q.outline(Corner::FrontRight), SlipOutline::Caution);
        q.update([0.02, 0.05, 0.19, 0.25], None, DT);
        assert_eq!(q.outline(Corner::FrontRight), SlipOutline::None);
    }

    #[test]
    fn temperatures_drive_the_fill_and_crossfade() {
        let t = Tokens::default();
        let mut q = TyreSlipQuad::new(&t);
        q.update([0.0; 4], Some([60.0, 90.0, 105.0, 120.0]), DT);
        assert_eq!(q.step(Corner::FrontLeft), TyreStep::Cool);
        assert_eq!(q.step(Corner::FrontRight), TyreStep::Optimal);
        assert_eq!(q.step(Corner::RearLeft), TyreStep::Hot);
        assert_eq!(q.step(Corner::RearRight), TyreStep::Critical);
        assert_eq!(q.shown_temp(Corner::RearRight), Some(120));
        assert!(q.animating(), "crossfading into the new steps");
        let frames = (t.durations.crossfade / DT).ceil() as usize;
        for _ in 0..frames {
            q.update([0.0; 4], Some([60.0, 90.0, 105.0, 120.0]), DT);
        }
        assert!(!q.animating(), "crossfade done within {} s", t.durations.crossfade);
        // Slip-only mode: no temperatures shown.
        q.update([0.0; 4], None, DT);
        assert_eq!(q.shown_temp(Corner::FrontLeft), None);
    }

    #[test]
    fn non_finite_inputs_hold_state() {
        let t = Tokens::default();
        let mut q = TyreSlipQuad::new(&t);
        q.update([0.0; 4], Some([105.0; 4]), DT);
        q.update([f32::NAN; 4], Some([f32::NAN; 4]), DT);
        assert_eq!(q.step(Corner::FrontLeft), TyreStep::Hot);
        assert_eq!(q.shown_temp(Corner::FrontLeft), Some(105));
        assert_eq!(q.outline(Corner::FrontLeft), SlipOutline::None);
    }
}
