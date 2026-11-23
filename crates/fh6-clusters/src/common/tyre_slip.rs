//! TyreSlipQuad (COMPONENTS.md §12, tokens `components.tyreSlipQuad`): four tyres in a car
//! footprint.
//!
//! Each tyre is filled with a discrete step colour (never a continuous hue sweep), crossfaded
//! over `motion.durations.crossfade`: the thermal step when temperatures are supplied (tokens
//! `colorRoles.thermal`: cold < 70 °C, optimal < 100, hot < 112, overheated), otherwise the slip
//! step. Both ramps use the palette roles of `slipRamp`: `primary` -> `go` -> `caution` ->
//! `warning` ([`TyreFill::Ramp`]); the Race Dash family colours only by exception
//! ([`TyreFill::Exception`]: cool and optimal stay a neutral lens). A vertical sheen shades the
//! tread, a 1 px top-lit rim edges it, and a slip outline appears in `caution` above a 0.08 slip
//! ratio or `warning` above 0.18. Rank 4: no glow.
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
    pub const ALL: [Corner; 4] = [
        Corner::FrontLeft,
        Corner::FrontRight,
        Corner::RearLeft,
        Corner::RearRight,
    ];

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
/// How far the sheen highlight moves the fill toward white.
const SHEEN_WHITE: f32 = 0.22;
/// Tyre size as a fraction of the footprint rect (width, height).
const TYRE_PCT: (f32, f32) = (0.24, 0.30);
/// Tyre centres sit this far (fraction of the footprint height) in from the top and bottom.
const AXLE_PCT: f32 = 0.20;
/// Air between a tyre and the body line (wheel arch), as a fraction of the tyre width.
const ARCH_AIR_PCT: f32 = 0.14;
/// The slip outline sits this far outside the tyre (fraction of the tyre width)...
const SLIP_RING_GAP_PCT: f32 = 0.09;
/// ...this wide, so it stays inside the arch air and never touches the body line.
const SLIP_RING_W_PCT: f32 = 0.045;

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

/// How the fill steps map to colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TyreFill {
    /// Every step coloured (`slipRamp`: `primary` -> `go` -> `caution` -> `warning`).
    #[default]
    Ramp,
    /// Colour only by exception (Race Dash): cool and optimal tyres are a neutral lens (the unlit
    /// lamp grey), hot is `caution`, critical is `warning`.
    Exception,
}

impl TyreFill {
    /// The fill colour of `step` from the palette.
    #[must_use]
    pub fn color(self, step: TyreStep, palette: &Palette) -> Color {
        match (self, step) {
            (TyreFill::Exception, TyreStep::Cool | TyreStep::Optimal) => palette.roles.ink_ghost,
            _ => step.color(palette),
        }
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
    fill: TyreFill,
    step: [usize; 4],
    from: [usize; 4],
    mix: [f32; 4],
    outline: [SlipOutline; 4],
    temps: [Hysteresis; 4],
    shown_temp: [Option<i64>; 4],
    crossfade: f32,
}

impl TyreSlipQuad {
    /// A quad at rest (cool, no outline) with the full colour ramp, crossfading over the tokens'
    /// `crossfade` duration.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            fill: TyreFill::Ramp,
            step: [0; 4],
            from: [0; 4],
            mix: [1.0; 4],
            outline: [SlipOutline::None; 4],
            temps: [Hysteresis::new(0.0, 0.6); 4],
            shown_temp: [None; 4],
            crossfade: tokens.durations.crossfade.max(1e-3),
        }
    }

    /// With a fill colour mapping.
    #[must_use]
    pub fn with_fill(mut self, fill: TyreFill) -> Self {
        self.fill = fill;
        self
    }

    /// Advances with four caller-supplied slip ratios (FL, FR, RL, RR; sign ignored) and optional
    /// temperatures in °C. With temperatures the fill is thermal, without it is the slip step.
    /// The caller verifies the telemetry units first (see the module docs). A non-finite
    /// temperature keeps that tyre's last step and reading (none if it never had one).
    pub fn update(&mut self, slip: [f32; 4], temps_c: Option<[f32; 4]>, dt: f32) {
        let dt = finite_or(dt, 0.0).max(0.0);
        for (i, raw) in slip.iter().enumerate() {
            let s = finite_or(*raw, 0.0).abs();
            let next = match temps_c {
                Some(t) if t[i].is_finite() => {
                    self.shown_temp[i] = Some(self.temps[i].update(t[i]));
                    stepped(self.step[i], t[i], &THERMAL_EDGES_C, THERMAL_HYSTERESIS_C)
                }
                // Never invent a reading: hold the last step and shown value.
                Some(_) => self.step[i],
                None => {
                    self.shown_temp[i] = None;
                    stepped(self.step[i], s, &SLIP_EDGES, SLIP_HYSTERESIS)
                }
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
        self.shown_temp[corner.index()]
    }

    /// True while a step crossfade runs.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.mix.iter().any(|m| *m < 1.0)
    }

    /// The crossfaded fill colour of tyre `i`.
    fn fill(&self, i: usize, palette: &Palette) -> Color {
        let from = self.fill.color(TyreStep::from_index(self.from[i]), palette);
        let to = self.fill.color(TyreStep::from_index(self.step[i]), palette);
        from.lerp(to, self.mix[i])
    }

    /// Draws the quad in `rect` (the footprint: tyres at its corners; the body outline runs 3%
    /// past its top and bottom). Temperatures, when supplied, are set outside `rect` beside each
    /// tyre in `font` at `metricSecondary` (allow about 2.4 caps of margin on both sides).
    /// Nothing is painted opaque, so the quad sits on OLED black, glass or the game alike.
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette, font: FontId) {
        let roles = palette.roles;
        let w = rect.width();
        let h = rect.height();
        let tyre = vec2(w * TYRE_PCT.0, h * TYRE_PCT.1);
        let (y0, y1) = (rect.min.y, rect.max.y);
        let front_y = y0 + h * AXLE_PCT - tyre.y * 0.5;
        let rear_y = y1 - h * AXLE_PCT - tyre.y * 0.5;
        let air = tyre.x * ARCH_AIR_PCT;

        // Body: a hairline car footprint around the wheels, rounder at the nose, lit from above.
        // The wheels sit proud of its sides, so the line is clipped to nose, waist and tail bands
        // that stop a little air short of each wheel arch: it never crosses a tyre, and no
        // opaque black is needed to hide it (that would punch holes in glass or the game).
        let inset = tyre.x * 0.40;
        let body = Rect::from_xywh(rect.min.x + inset, y0 - h * 0.03, w - 2.0 * inset, h * 1.06);
        let bw = body.width();
        let body_shape = Shape::round_rect(body, Radii::new(bw * 0.42, bw * 0.42, bw * 0.22, bw * 0.22));
        let body_paint = Gradient::linear(body.at(0.5, 0.0), body.at(0.5, 1.0))
            .stop(0.0, roles.hairline.lerp(roles.rim_specular, 0.25))
            .stop(0.5, roles.hairline)
            .stop(1.0, roles.hairline.fade(0.55));
        let margin = 4.0 * p.device_px();
        let (bx0, bx1) = (body.min.x - margin, body.max.x + margin);
        let bands = [
            (body.min.y - margin, front_y - air),
            (front_y + tyre.y + air, rear_y - air),
            (rear_y + tyre.y + air, body.max.y + margin),
        ];
        for (top, bottom) in bands {
            if bottom > top {
                p.scoped(|p| {
                    p.clip(Rect::from_xywh(bx0, top, bx1 - bx0, bottom - top), 0.0);
                    p.stroke(&body_shape, body_paint, Stroke::rim_px(1.0));
                });
            }
        }

        let ts = p.tokens().type_scale;
        let temp_style = TextStyle::new(font, ts.metric_secondary).tabular();
        for corner in Corner::ALL {
            let i = corner.index();
            let x = if corner.is_left() {
                rect.min.x
            } else {
                rect.max.x - tyre.x
            };
            let y = if corner.is_front() { front_y } else { rear_y };
            let tr = Rect::from_xywh(x, y, tyre.x, tyre.y);
            self.draw_tyre(p, tr, i, palette);

            if let Some(temp) = self.shown_temp[i] {
                let step = TyreStep::from_index(self.step[i]);
                // Colour by exception: neutral until hot.
                let ink = match step {
                    TyreStep::Hot | TyreStep::Critical => step.color(palette).text_safe(),
                    _ => roles.ink_muted,
                };
                let align = if corner.is_left() { Align::Right } else { Align::Left };
                let style = temp_style.paint(ink).align(align, VAlign::CapMiddle);
                let pos = temp_anchor(corner, tr, ts.metric_secondary);
                // Exactly as many cells as digits: a fixed 3-cell field would leave an empty
                // leading cell between a right-hand tyre and a two-digit reading, so the right
                // labels would sit further out than the left ones. The edge facing the tyre stays
                // fixed either way (right-aligned on the left, left-aligned on the right).
                let temp = temp.clamp(-99, 999);
                p.number(pos, temp, digit_cells(temp), &style, None);
            }
        }
    }

    fn draw_tyre(&self, p: &mut Painter<'_>, tr: Rect, i: usize, palette: &Palette) {
        let roles = palette.roles;
        let r = tr.width() * 0.24;
        let shape = Shape::round_rect(tr, Radii::all(r));
        let c = self.fill(i, palette);
        // Vertical sheen: the tread curves away front and back, lit from above. The highlight
        // keeps the fill's own opacity, so a neutral lens stays as quiet as its colour.
        let sheen = c.lerp(roles.ink.fade(c.a), SHEEN_WHITE);
        p.fill(
            &shape,
            Gradient::linear(tr.at(0.5, 0.0), tr.at(0.5, 1.0))
                .stop(0.0, c.fade(0.50))
                .stop(0.24, sheen)
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
            let gap = tr.width() * SLIP_RING_GAP_PCT;
            let width = (tr.width() * SLIP_RING_W_PCT).max(2.0 * p.device_px());
            p.stroke(
                &Shape::round_rect(tr.expand(gap), Radii::all(r + gap)),
                color,
                Stroke::new(width),
            );
        }
    }
}

/// Digit cells `value` occupies, its minus sign included (`5` -> 1, `98` -> 2, `-12` -> 3).
fn digit_cells(value: i64) -> u8 {
    let mut cells = if value < 0 { 2 } else { 1 };
    let mut rest = value.unsigned_abs() / 10;
    while rest > 0 {
        cells += 1;
        rest /= 10;
    }
    cells
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
    fn exception_fill_is_neutral_until_hot() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let f = TyreFill::Exception;
        assert_eq!(f.color(TyreStep::Cool, &pal), pal.roles.ink_ghost);
        assert_eq!(
            f.color(TyreStep::Optimal, &pal),
            pal.roles.ink_ghost,
            "never green for nominal"
        );
        assert_eq!(f.color(TyreStep::Hot, &pal), pal.roles.caution);
        assert_eq!(f.color(TyreStep::Critical, &pal), pal.roles.warning);
        assert_eq!(TyreFill::Ramp.color(TyreStep::Optimal, &pal), pal.roles.go);
        let q = TyreSlipQuad::new(&t).with_fill(f);
        assert_eq!(q.fill(0, &pal), pal.roles.ink_ghost);
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
    fn a_missing_first_reading_is_not_invented() {
        let t = Tokens::default();
        let mut q = TyreSlipQuad::new(&t);
        q.update([0.0; 4], Some([f32::NAN, 90.0, 90.0, f32::INFINITY]), DT);
        assert_eq!(q.shown_temp(Corner::FrontLeft), None, "no reading yet, not 0 °C");
        assert_eq!(q.shown_temp(Corner::FrontRight), Some(90));
        assert_eq!(q.shown_temp(Corner::RearRight), None);
        assert_eq!(q.step(Corner::FrontLeft), TyreStep::Cool);
    }

    #[test]
    fn temperature_labels_take_only_their_digits() {
        assert_eq!(digit_cells(0), 1);
        assert_eq!(digit_cells(9), 1);
        assert_eq!(digit_cells(98), 2);
        assert_eq!(digit_cells(104), 3);
        assert_eq!(digit_cells(-5), 2);
        assert_eq!(digit_cells(-99), 3);
        assert_eq!(digit_cells(i64::MIN), 20);
    }

    #[test]
    fn slip_ring_stays_inside_the_arch_air() {
        // The body line stops `ARCH_AIR_PCT` short of the tyre; the ring's outer edge must too.
        let outer = SLIP_RING_GAP_PCT + SLIP_RING_W_PCT * 0.5;
        assert!(outer < ARCH_AIR_PCT, "ring reaches {outer} of the tyre width");
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
