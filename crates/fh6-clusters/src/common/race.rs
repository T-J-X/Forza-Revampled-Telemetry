//! Race Dash alarm lamps and data tiles (COMPONENTS.md §14, `docs/design/families/race-dash.md`).
//!
//! - [`AlarmColumn`]: up to five LED-lens alarm lamps on one side of the screen (or in a row
//!   under it). A lamp lights only from a real condition the cluster evaluates ([`tc_condition`],
//!   [`abs_condition`], [`rev_condition`], [`tyre_condition`]). Lit lamps take `caution` (or the
//!   palette `shift` colour for REV) with a soft distance glow, never bloom (ranks 4–5); unlit
//!   lamps are a grey lens. Labels are shy: neutral ink until their lamp lights.
//! - [`DataTile`]: a value right-aligned in tabular figures under a micro-label at 25–30% of the
//!   value height, framed by a 1 px hairline (no fill) or left open above a fading divider.
//! - [`format_fixed`] and [`draw_fixed`]: allocation-free fixed-point numbers for tiles and
//!   readouts (`Painter::number` covers integers only).
use fh6_kit::math::finite_or;
use fh6_kit::{
    vec2, Align, Color, FontId, Glow, Gradient, Hysteresis, Painter, Palette, Rect, Shape, Signals, Stroke, TextStyle,
    Tokens, VAlign, Vec2,
};

/// Lamps per alarm column (COMPONENTS.md §14: 5 per side).
pub const MAX_LAMPS: usize = 5;

/// Lens height as a fraction of its width (the Race Dash pods are 88 x 30).
const LAMP_ASPECT: f32 = 30.0 / 88.0;
/// Lens corner radius as a fraction of its height (tokens `components.shiftLights.pod.radiusPctOfHeight`).
const LAMP_RADIUS_PCT: f32 = 0.35;
/// Opacity of the unlit lens rim (COMPONENTS.md §4: a 12% rim on a grey lens).
const UNLIT_RIM_ALPHA: f32 = 0.12;
/// Opacity of the lit inner rim (tokens `components.shiftLights.pod.innerRimAlphaPct`).
const LIT_RIM_ALPHA: f32 = 0.50;
/// How far the lit core moves toward white (tokens `components.shiftLights.pod.core`: 40%).
const CORE_WHITE: f32 = 0.40;
/// Opacity of the top glint on a lit lens.
const GLINT_ALPHA: f32 = 0.30;
/// Lamps count as lit above this level.
const LIT_LEVEL: f32 = 0.5;

/// TC lamp: rear slip ratio above this with throttle (race-dash.md "Alarm lamps").
pub const TC_REAR_SLIP: f32 = 0.15;
/// TC lamp: throttle above this fraction.
pub const TC_THROTTLE: f32 = 0.20;
/// ABS lamp: front slip ratio above this with the brake applied (the slip outline's caution edge).
pub const ABS_FRONT_SLIP: f32 = 0.08;
/// ABS lamp: brake above this fraction.
pub const ABS_BRAKE: f32 = 0.20;
/// TYRE lamp: any tyre at or above this temperature in °C (tokens `colorRoles.thermal`, overheated).
pub const TYRE_OVERHEAT_C: f32 = 112.0;

/// The colour a lit lamp takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LampTone {
    /// Assists and soft limits (`caution`): TC, ABS, FUEL, TYRE.
    Caution,
    /// Hard faults (`warning`).
    Warning,
    /// The limiter (palette `shift`): REV.
    Shift,
}

impl LampTone {
    /// The lit colour from the palette.
    #[must_use]
    pub fn color(self, palette: &Palette) -> Color {
        match self {
            LampTone::Caution => palette.roles.caution,
            LampTone::Warning => palette.roles.warning,
            LampTone::Shift => palette.shift,
        }
    }

    /// True for alarm tones, which outrank the shift lights while lit (race-dash.md).
    #[must_use]
    pub fn is_alarm(self) -> bool {
        matches!(self, LampTone::Caution | LampTone::Warning)
    }
}

/// One alarm lamp: its shy label and the tone it lights in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlarmLamp {
    /// Upper-case micro-label (at most about 5 characters).
    pub label: &'static str,
    /// Lit colour.
    pub tone: LampTone,
}

impl AlarmLamp {
    /// FUEL: fuel below 10%. `fh6-core` does not decode fuel yet: verify the field against
    /// `tests/golden/packets` before wiring a source.
    pub const FUEL: AlarmLamp = AlarmLamp::new("FUEL", LampTone::Caution);
    /// TC: rear slip above [`TC_REAR_SLIP`] with throttle above [`TC_THROTTLE`] ([`tc_condition`]).
    pub const TC: AlarmLamp = AlarmLamp::new("TC", LampTone::Caution);
    /// ABS: front slip above [`ABS_FRONT_SLIP`] with brake above [`ABS_BRAKE`] ([`abs_condition`]).
    pub const ABS: AlarmLamp = AlarmLamp::new("ABS", LampTone::Caution);
    /// REV: the limiter ([`rev_condition`]); lights in the palette's shift colour.
    pub const REV: AlarmLamp = AlarmLamp::new("REV", LampTone::Shift);
    /// TYRE: any tyre overheated ([`tyre_condition`]).
    pub const TYRE: AlarmLamp = AlarmLamp::new("TYRE", LampTone::Caution);

    /// A lamp with `label` lighting in `tone`.
    #[must_use]
    pub const fn new(label: &'static str, tone: LampTone) -> Self {
        Self { label, tone }
    }
}

/// TC condition: rear slip ratio above [`TC_REAR_SLIP`] while the throttle is above
/// [`TC_THROTTLE`]. The caller supplies the rear slip ratio and must verify the tyre slip units
/// against `tests/golden/packets` first (tyre fields are not in the golden set).
#[must_use]
pub fn tc_condition(rear_slip: f32, throttle: f32) -> bool {
    finite_or(rear_slip, 0.0).abs() > TC_REAR_SLIP && finite_or(throttle, 0.0) > TC_THROTTLE
}

/// ABS condition: front slip ratio above [`ABS_FRONT_SLIP`] while the brake is above
/// [`ABS_BRAKE`]. The caller supplies the front slip ratio (verify its units first).
#[must_use]
pub fn abs_condition(front_slip: f32, brake: f32) -> bool {
    finite_or(front_slip, 0.0).abs() > ABS_FRONT_SLIP && finite_or(brake, 0.0) > ABS_BRAKE
}

/// REV condition: at or past the shift rpm (`shift_fraction >= 1`, the limiter strobe zone).
#[must_use]
pub fn rev_condition(sig: &Signals) -> bool {
    sig.shift_fraction >= 1.0
}

/// TYRE condition: any of four caller-supplied temperatures (°C, units verified by the caller)
/// at or above [`TYRE_OVERHEAT_C`].
#[must_use]
pub fn tyre_condition(temps_c: &[f32; 4]) -> bool {
    temps_c.iter().any(|t| t.is_finite() && *t >= TYRE_OVERHEAT_C)
}

/// Where an alarm column sits, which decides its arrangement and label side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlarmSide {
    /// A column on the left of the screen; labels sit to the right of the lamps (inboard).
    Left,
    /// A column on the right of the screen; labels sit to the left of the lamps (inboard).
    Right,
    /// A row under the screen, centred; labels sit under the lamps.
    Bottom,
}

/// Up to [`MAX_LAMPS`] alarm lamps with token optics: 0 ms attack, a release within
/// `motion.durations.micro`, steady while lit (lamps never strobe; only the shift strip does).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlarmColumn {
    side: AlarmSide,
    lamps: [AlarmLamp; MAX_LAMPS],
    count: usize,
    level: [f32; MAX_LAMPS],
    release_tau: f32,
}

impl AlarmColumn {
    /// A column on `side` with the first [`MAX_LAMPS`] of `lamps` (only lamps with a real
    /// source: race-dash.md "Don't keep lamps that can never light").
    #[must_use]
    pub fn new(tokens: &Tokens, side: AlarmSide, lamps: &[AlarmLamp]) -> Self {
        let mut slots = [AlarmLamp::new("", LampTone::Caution); MAX_LAMPS];
        let count = lamps.len().min(MAX_LAMPS);
        slots[..count].copy_from_slice(&lamps[..count]);
        Self {
            side,
            lamps: slots,
            count,
            level: [0.0; MAX_LAMPS],
            // Half the micro duration as the time constant: dark within about 2 tau.
            release_tau: (tokens.durations.micro * 0.5).max(1e-3),
        }
    }

    /// The lamps in this column.
    #[must_use]
    pub fn lamps(&self) -> &[AlarmLamp] {
        &self.lamps[..self.count]
    }

    /// Advances: `active[i]` lights lamp `i` this frame (missing entries are off).
    pub fn update(&mut self, active: &[bool], dt: f32) {
        let k = 1.0 - (-finite_or(dt, 0.0).max(0.0) / self.release_tau).exp();
        for (i, level) in self.level.iter_mut().enumerate().take(self.count) {
            if active.get(i).copied().unwrap_or(false) {
                *level = 1.0;
            } else {
                *level -= *level * k;
                if *level < 1e-3 {
                    *level = 0.0;
                }
            }
        }
    }

    /// True while lamp `i` is lit.
    #[must_use]
    pub fn is_lit(&self, i: usize) -> bool {
        i < self.count && self.level[i] > LIT_LEVEL
    }

    /// True while an alarm lamp (caution or warning tone) is lit: warnings outrank the shift
    /// lights, so a cluster may calm its strip while this holds.
    #[must_use]
    pub fn outranks_shift(&self) -> bool {
        (0..self.count).any(|i| self.lamps[i].tone.is_alarm() && self.is_lit(i))
    }

    /// True while a lamp is still fading out.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.level[..self.count].iter().any(|l| *l > 0.0 && *l < 1.0)
    }

    /// Draws the lamps in `rect`: a column (left or right) uses five equal slots from the top,
    /// a bottom row centres its lamps. Label caps come from the tokens (`microLabel`, never
    /// below `nano`).
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette, label_font: FontId) {
        let ts = p.tokens().type_scale;
        let soft = p.tokens().glow.soft;
        let slots = MAX_LAMPS as f32;
        let (lens_w, lens_h) = match self.side {
            AlarmSide::Left | AlarmSide::Right => {
                let slot = rect.height() / slots;
                let w = (slot * 0.55 / LAMP_ASPECT).min(rect.width() * 0.5);
                (w, w * LAMP_ASPECT)
            }
            AlarmSide::Bottom => {
                let w = rect.width() / slots * 0.72;
                (w, w * LAMP_ASPECT)
            }
        };
        let cap = (lens_h * 0.55).clamp(ts.nano, ts.micro_label);
        let gap = lens_h * 0.45;
        for i in 0..self.count {
            let lamp = self.lamps[i];
            let level = self.level[i];
            let (lens, label_pos, align, valign) = match self.side {
                AlarmSide::Left | AlarmSide::Right => {
                    let slot = rect.height() / slots;
                    let cy = rect.min.y + slot * (i as f32 + 0.5);
                    if self.side == AlarmSide::Left {
                        let lens = Rect::from_xywh(rect.min.x, cy - lens_h * 0.5, lens_w, lens_h);
                        (lens, vec2(lens.max.x + gap, cy), Align::Left, VAlign::CapMiddle)
                    } else {
                        let lens = Rect::from_xywh(rect.max.x - lens_w, cy - lens_h * 0.5, lens_w, lens_h);
                        (lens, vec2(lens.min.x - gap, cy), Align::Right, VAlign::CapMiddle)
                    }
                }
                AlarmSide::Bottom => {
                    let slot = rect.width() / slots;
                    let first = rect.center().x - slot * self.count as f32 * 0.5;
                    let cx = first + slot * (i as f32 + 0.5);
                    let lens = Rect::from_xywh(cx - lens_w * 0.5, rect.min.y, lens_w, lens_h);
                    (lens, vec2(cx, lens.max.y + gap), Align::Center, VAlign::CapTop)
                }
            };
            let color = lamp.tone.color(palette);
            draw_lens(p, lens, color, level, palette, &soft);
            // Shy label: neutral until lit, then the lamp's colour lifted for text.
            let ink = if level > 0.0 {
                palette.roles.ink_label.lerp(color.text_safe(), level)
            } else {
                palette.roles.ink_label
            };
            let style = TextStyle::new(label_font, cap)
                .paint(ink)
                .tracking(ts.label_tracking)
                .align(align, valign);
            p.text(label_pos, lamp.label, &style);
        }
    }
}

/// One LED lens (tokens `components.shiftLights.pod`): unlit = a grey lens with a 12% rim; lit =
/// a 2-stop radial core 40% toward white, a 1 px inner rim at 50% white, a top glint and a soft
/// distance glow (not bloom).
fn draw_lens(
    p: &mut Painter<'_>,
    lens: Rect,
    color: Color,
    level: f32,
    palette: &Palette,
    soft: &fh6_kit::tokens::GlowLevel,
) {
    let roles = palette.roles;
    let r = lens.height() * LAMP_RADIUS_PCT;
    let shape = Shape::round_rect(lens, r);
    if level <= 0.0 {
        // Grey lens: a vertical falloff so the unlit pod reads as glass, not a flat chip.
        p.fill(
            &shape,
            Gradient::linear(lens.at(0.5, 0.0), lens.at(0.5, 1.0))
                .stop(0.0, roles.ink_ghost)
                .stop(1.0, roles.ink_ghost.fade(0.45)),
        );
        p.stroke(&shape, roles.ink.with_alpha(UNLIT_RIM_ALPHA), Stroke::rim_px(1.0));
        return;
    }
    for lobe in soft.lobes() {
        p.glow(&shape, color.fade(level), Glow::new(lobe.sigma, lobe.intensity));
    }
    let core = color.lerp(roles.ink, CORE_WHITE);
    p.fill(
        &shape,
        Gradient::radial(lens.center(), lens.height() * 1.6)
            .stop(0.0, core.fade(level))
            .stop(1.0, color.fade(level)),
    );
    p.stroke(&shape, roles.ink.with_alpha(LIT_RIM_ALPHA * level), Stroke::rim_px(1.0));
    let glint = Rect::from_xywh(
        lens.min.x + r,
        lens.min.y + lens.height() * 0.14,
        (lens.width() - 2.0 * r).max(0.0),
        lens.height() * 0.16,
    );
    p.fill(
        &Shape::round_rect(glint, glint.height() * 0.5),
        roles.ink.with_alpha(GLINT_ALPHA * level),
    );
}

/// Size of the buffer [`format_fixed`] writes into: a sign, 20 digits and the point fit.
pub const FIXED_BUF: usize = 24;
/// Most decimals [`format_fixed`] places.
pub const MAX_DECIMALS: u8 = 6;

/// Formats `scaled / 10^decimals` into `buf` without allocating, e.g. `(-1234, 2)` -> `-12.34`
/// and `(5, 2)` -> `0.05`. `decimals` is clamped to [`MAX_DECIMALS`]; 0 gives an integer.
#[must_use]
pub fn format_fixed(scaled: i64, decimals: u8, buf: &mut [u8; FIXED_BUF]) -> &str {
    let decimals = usize::from(decimals.min(MAX_DECIMALS));
    let mut mag = scaled.unsigned_abs();
    let mut i = FIXED_BUF;
    let mut written = 0;
    loop {
        if decimals > 0 && written == decimals {
            i -= 1;
            buf[i] = b'.';
        }
        i -= 1;
        buf[i] = b'0' + (mag % 10) as u8;
        mag /= 10;
        written += 1;
        if mag == 0 && written > decimals {
            break;
        }
    }
    if scaled < 0 {
        i -= 1;
        buf[i] = b'-';
    }
    std::str::from_utf8(&buf[i..]).unwrap_or("")
}

/// Draws `scaled / 10^decimals` at `pos` with `style` (use tabular figures so the digits sit in
/// fixed cells). Allocation-free. Returns the set bounds.
pub fn draw_fixed(p: &mut Painter<'_>, pos: Vec2, scaled: i64, decimals: u8, style: &TextStyle) -> Rect {
    let mut buf = [0u8; FIXED_BUF];
    let text = format_fixed(scaled, decimals, &mut buf);
    p.text(pos, text, style)
}

/// Label cap height as a fraction of the value cap (COMPONENTS.md §14: 25–30%).
pub const TILE_LABEL_RATIO: f32 = 0.28;
/// Padding inside an outlined tile as a fraction of the value cap.
const TILE_PAD: f32 = 0.24;
/// Gap between label and value as a fraction of the value cap.
const TILE_GAP: f32 = 0.30;
/// Corner radius of an outlined tile (tokens `radius.xs`).
const TILE_RADIUS: f32 = 6.0;

/// How a tile formats its value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileFormat {
    /// A whole number in `cells` fixed digit cells (`Painter::number`).
    Integer {
        /// Digit cells (the field width never changes).
        cells: u8,
    },
    /// A fixed-point number with `decimals` places (at most [`MAX_DECIMALS`]).
    Fixed {
        /// Places after the point.
        decimals: u8,
    },
}

impl TileFormat {
    fn decimals(self) -> u8 {
        match self {
            TileFormat::Integer { .. } => 0,
            TileFormat::Fixed { decimals } => decimals.min(MAX_DECIMALS),
        }
    }
}

/// A tile's frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileFrame {
    /// A 1 px hairline outline, lit slightly brighter at the top (no fill).
    #[default]
    Outline,
    /// No box: a 1 px divider on top that fades at both ends (race-dash.md: align, don't box).
    Open,
}

/// A tile's exception state: colour appears only when something is wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TileAlert {
    /// Neutral ink.
    #[default]
    Normal,
    /// `caution` value and frame.
    Caution,
    /// `warning` value and frame.
    Warning,
}

/// A Race Dash data tile: micro-label top-left, value right-aligned in tabular figures, 0.6-step
/// hysteresis on the last shown digit so it never flickers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DataTile {
    label: &'static str,
    format: TileFormat,
    frame: TileFrame,
    alert: TileAlert,
    shown: Hysteresis,
    value: Option<i64>,
}

impl DataTile {
    /// A tile labelled `label` (upper case) showing values in `format`, with an outline frame.
    #[must_use]
    pub fn new(label: &'static str, format: TileFormat) -> Self {
        Self {
            label,
            format,
            frame: TileFrame::Outline,
            alert: TileAlert::Normal,
            shown: Hysteresis::new(0.0, 0.6),
            value: None,
        }
    }

    /// With a frame style.
    #[must_use]
    pub fn with_frame(mut self, frame: TileFrame) -> Self {
        self.frame = frame;
        self
    }

    /// Feeds the value (`None` or a non-finite value shows `--`).
    pub fn update(&mut self, value: Option<f32>) {
        let scale = 10f32.powi(i32::from(self.format.decimals()));
        self.value = value.filter(|v| v.is_finite()).map(|v| self.shown.update(v * scale));
    }

    /// Sets the exception state.
    pub fn set_alert(&mut self, alert: TileAlert) {
        self.alert = alert;
    }

    /// The shown value in units of the last decimal place (`None` when there is no data).
    #[must_use]
    pub fn shown(&self) -> Option<i64> {
        self.value
    }

    /// The tile height that fits a value of cap `value_cap` (label, gap and padding included).
    #[must_use]
    pub fn preferred_height(tokens: &Tokens, value_cap: f32) -> f32 {
        let label = (value_cap * TILE_LABEL_RATIO).max(tokens.type_scale.nano);
        value_cap * (2.0 * TILE_PAD + TILE_GAP) + label + value_cap
    }

    /// Draws the tile in `rect` with the value at cap `value_cap` (Race Dash: `race` figures).
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        rect: Rect,
        value_cap: f32,
        palette: &Palette,
        value_font: FontId,
        label_font: FontId,
    ) {
        let ts = p.tokens().type_scale;
        let roles = palette.roles;
        let label_cap = (value_cap * TILE_LABEL_RATIO).max(ts.nano);
        let alert = match self.alert {
            TileAlert::Normal => None,
            TileAlert::Caution => Some(roles.caution),
            TileAlert::Warning => Some(roles.warning),
        };
        let pad_x = match self.frame {
            TileFrame::Outline => value_cap * TILE_PAD,
            TileFrame::Open => 0.0,
        };
        let pad_y = value_cap * TILE_PAD;
        match self.frame {
            TileFrame::Outline => {
                let shape = Shape::round_rect(rect, TILE_RADIUS);
                let base = alert.map_or(roles.hairline, |c| c.with_alpha(0.55));
                // Lit from above (the virtual light sits high): brighter top edge, dim base.
                let top = base.lerp(roles.rim_specular, 0.22);
                p.stroke(
                    &shape,
                    Gradient::linear(rect.at(0.5, 0.0), rect.at(0.5, 1.0))
                        .stop(0.0, top)
                        .stop(0.35, base)
                        .stop(1.0, base.fade(0.55)),
                    Stroke::rim_px(1.0),
                );
            }
            TileFrame::Open => {
                let line = Rect::from_xywh(rect.min.x, rect.min.y, rect.width(), p.device_px());
                let mid = alert.map_or(roles.hairline, |c| c.with_alpha(0.55));
                p.fill(
                    &Shape::rect(line),
                    Gradient::linear(line.at(0.0, 0.5), line.at(1.0, 0.5))
                        .stop(0.0, mid.fade(0.0))
                        .stop(0.2, mid)
                        .stop(0.8, mid)
                        .stop(1.0, mid.fade(0.0)),
                );
            }
        }
        let label_ink = alert.map_or(roles.ink_label, Color::text_safe);
        let label_style = TextStyle::new(label_font, label_cap)
            .paint(label_ink)
            .tracking(ts.label_tracking)
            .align(Align::Left, VAlign::CapTop);
        p.text(vec2(rect.min.x + pad_x, rect.min.y + pad_y), self.label, &label_style);

        let value_pos = vec2(rect.max.x - pad_x, rect.max.y - pad_y);
        let value_ink = alert.map_or(roles.ink, Color::text_safe);
        let style = TextStyle::new(value_font, value_cap)
            .paint(value_ink)
            .align(Align::Right, VAlign::Baseline)
            .tabular();
        match (self.value, self.format) {
            (None, _) => {
                p.text(value_pos, "--", &style.paint(roles.ink_label));
            }
            (Some(v), TileFormat::Integer { cells }) => {
                p.number(value_pos, v, cells, &style, None);
            }
            (Some(v), TileFormat::Fixed { decimals }) => {
                draw_fixed(p, value_pos, v, decimals, &style);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixed(v: i64, d: u8) -> String {
        let mut buf = [0u8; FIXED_BUF];
        format_fixed(v, d, &mut buf).to_owned()
    }

    #[test]
    fn fixed_point_formatting() {
        assert_eq!(fixed(0, 0), "0");
        assert_eq!(fixed(42, 0), "42");
        assert_eq!(fixed(-7, 0), "-7");
        assert_eq!(fixed(1234, 2), "12.34");
        assert_eq!(fixed(5, 2), "0.05");
        assert_eq!(fixed(-5, 1), "-0.5");
        assert_eq!(fixed(100, 1), "10.0");
        assert_eq!(fixed(0, 3), "0.000");
        assert_eq!(fixed(i64::MIN, 0), "-9223372036854775808");
        assert_eq!(fixed(i64::MAX, 6), "9223372036854.775807");
        assert_eq!(fixed(12, 9), "0.000012", "decimals clamp to {MAX_DECIMALS}");
    }

    #[test]
    fn tile_hysteresis_and_missing_data() {
        let mut t = DataTile::new("BOOST", TileFormat::Fixed { decimals: 1 });
        assert_eq!(t.shown(), None);
        t.update(Some(14.26));
        assert_eq!(t.shown(), Some(143));
        // Less than 0.6 of the last digit: unchanged.
        t.update(Some(14.32));
        assert_eq!(t.shown(), Some(143));
        t.update(Some(14.40));
        assert_eq!(t.shown(), Some(144));
        t.update(Some(f32::NAN));
        assert_eq!(t.shown(), None);
        let mut i = DataTile::new("KW", TileFormat::Integer { cells: 4 });
        i.update(Some(311.4));
        assert_eq!(i.shown(), Some(311));
    }

    #[test]
    fn lamps_attack_instantly_and_release_within_micro() {
        let tokens = Tokens::default();
        let mut c = AlarmColumn::new(&tokens, AlarmSide::Left, &[AlarmLamp::TC, AlarmLamp::ABS]);
        c.update(&[true, false], 1.0 / 120.0);
        assert!(c.is_lit(0), "same-frame attack");
        assert!(!c.is_lit(1));
        assert!(c.outranks_shift());
        let mut t = 0.0;
        while c.is_lit(0) {
            c.update(&[false, false], 1.0 / 120.0);
            t += 1.0 / 120.0;
        }
        assert!(t <= tokens.durations.micro + 1e-4, "released in {t} s");
        for _ in 0..60 {
            c.update(&[], 1.0 / 60.0);
        }
        assert!(!c.animating());
    }

    #[test]
    fn rev_lamp_does_not_outrank_shift() {
        let tokens = Tokens::default();
        let mut c = AlarmColumn::new(&tokens, AlarmSide::Right, &[AlarmLamp::REV, AlarmLamp::TYRE]);
        c.update(&[true, false], 1.0 / 60.0);
        assert!(c.is_lit(0));
        assert!(!c.outranks_shift(), "the limiter lamp is not an alarm");
        let pal = Palette::family(&tokens, "raceDash");
        assert_eq!(LampTone::Shift.color(&pal), pal.shift);
        assert_eq!(LampTone::Caution.color(&pal), pal.roles.caution);
    }

    #[test]
    fn column_keeps_at_most_five_lamps() {
        let tokens = Tokens::default();
        let many = [AlarmLamp::TC; 7];
        let c = AlarmColumn::new(&tokens, AlarmSide::Bottom, &many);
        assert_eq!(c.lamps().len(), MAX_LAMPS);
    }

    #[test]
    fn alarm_conditions() {
        assert!(tc_condition(0.2, 0.5));
        assert!(!tc_condition(0.2, 0.1), "no TC without throttle");
        assert!(!tc_condition(0.1, 0.9));
        assert!(abs_condition(-0.09, 0.62), "slip sign is ignored");
        assert!(!abs_condition(0.09, 0.1));
        assert!(tyre_condition(&[90.0, 95.0, 113.0, 99.0]));
        assert!(!tyre_condition(&[90.0, f32::NAN, 111.0, 99.0]));
        let sig = Signals {
            shift_fraction: 1.02,
            ..Signals::default()
        };
        assert!(rev_condition(&sig));
    }

    #[test]
    fn tile_height_grows_with_the_value() {
        let tokens = Tokens::default();
        let small = DataTile::preferred_height(&tokens, 40.0);
        let big = DataTile::preferred_height(&tokens, 80.0);
        assert!(big > small && small > 40.0);
    }
}
