//! Tach variants `arc`, `bar` and `chevron` (COMPONENTS.md §3), each with the steady limiter cue
//! of §5 (one non-flashing cue per family; the 10 Hz strobe stays on the shift strip).
//!
//! The three variants share one fill model: the rpm fraction from [`Signals`] on the tokens'
//! `ribbon` spring with velocity feed-forward (the shown fill only moves toward the measured rpm
//! and never passes it, so the lead cannot show revs the engine does not have), and a limiter
//! amount on the `snap` spring that switches on at the shift rpm and off 120 rpm below it, so the
//! cue never flickers while the engine bounces off the limiter. `update` and `draw` allocate
//! nothing.
use fh6_kit::{
    deg, vec2, Align, Axis, Bloom, Cap, Color, FontId, Glow, Gradient, LayerOptions, Paint, Painter, Palette, Rect,
    SegmentedBar, Shape, Signals, Spring, Stroke, TextStyle, TickRing, Tokens, VAlign, Vec2,
};
use std::f32::consts::TAU;

/// Start of the redline sector as a fraction of the limiter rpm (tokens
/// `components.tach.redlineFromRpmPct`).
pub const REDLINE_FROM_RPM_PCT: f32 = 0.92;
/// Scale ticks and numerals every this many rpm (tokens `components.tach.tickEvery`).
pub const TICK_EVERY_RPM: f32 = 1000.0;
/// Segments of the Race Dash rev bar (`docs/design/families/race-dash.md`, zone Z3).
pub const BAR_SEGMENTS: u32 = 45;
/// Gap between Race Dash rev-bar segments in local px (COMPONENTS.md §3).
pub const BAR_GAP: f32 = 5.0;

/// Limiter cue off-hysteresis in rpm (tokens `colorRoles.revStages`: 120 rpm off-hysteresis).
const LIMITER_OFF_HYSTERESIS_RPM: f32 = 120.0;
/// The `ShiftHold` cue holds the shift colour at this strength (Modern Digital limiter: 85%).
const SHIFT_HOLD_STRENGTH: f32 = 0.85;
/// Race Dash: lit rev segments below the stage thresholds are 72% of `primary`.
const BAR_BASE_STRENGTH: f32 = 0.72;
/// VFD ghost segments are 7% of their phosphor colour (COMPONENTS.md §3 and §15).
const GHOST_STRENGTH: f32 = 0.07;
/// Most thousands drawn on a scale (guards a corrupt limiter value).
const MAX_THOUSANDS: u32 = 30;
/// Most segments a bar or chevron run accepts.
const MAX_SEGMENTS: u32 = 120;
/// The arc fill's tail starts at this share of `primary`, so the ribbon brightens with the revs.
const ARC_TAIL_STRENGTH: f32 = 0.42;
/// The printed redline sector and its ticks show `warning` at this strength (a marking, not a
/// light: it stays below the lit fill).
const REDLINE_STRENGTH: f32 = 0.7;
/// Width (rpm fraction) of the soft blend from `primary` into the caution stage on the arc.
const ARC_STAGE_BLEND: f32 = 0.05;
/// The hot head of the arc fill whitens to this share of `ink` at the very tip.
const ARC_HEAD_WHITE: f32 = 0.6;
/// Share of `ink` in the leading-edge glow: a near-white sparkle reads as heat, while a glow in a
/// dark stage hue (warning red) reads as a smudge on black.
const ARC_HEAD_GLOW_WHITE: f32 = 0.75;
/// Share of white mixed into the core of a lit VFD chevron (phosphor core).
const CHEVRON_CORE_WHITE: f32 = 0.24;
/// Half height of the core band, as a fraction of the segment height.
const CHEVRON_CORE_HALF: f32 = 0.3;
/// Gap between chevron segments as a fraction of the segment pitch.
const CHEVRON_GAP: f32 = 0.24;
/// Horizontal lean of a slanted chevron per unit of height (tan 14 degrees).
const CHEVRON_LEAN: f32 = 0.249;
/// Depth of an arrow chevron's point per unit of height.
const CHEVRON_ARROW_DEPTH: f32 = 0.22;
/// Height of the first chevron as a fraction of the last (the staircase rise).
const CHEVRON_RISE: f32 = 0.3;
/// The lens glint on lit bar cells fades out over this fraction of the cell height from the top.
const BAR_GLINT_HEIGHT: f32 = 0.38;
/// Glint strength (share of `ink`) at the top edge of a lit bar cell.
const BAR_LIT_GLINT: f32 = 0.24;
/// Lit bar cells shade toward their bottom edge to this share of `oledBlack` (a top-lit lens).
const BAR_LENS_SHADE: f32 = 0.22;
/// Rim strength (share of `hairline`) on unlit bar cells: a grey lens, no colour.
const BAR_UNLIT_RIM: f32 = 0.6;
/// The peak-hold cell shows its stage colour at this strength.
const BAR_PEAK_STRENGTH: f32 = 0.55;

/// The steady limiter cue a tach shows (COMPONENTS.md §5): one per family, never a flash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LimiterCue {
    /// No cue on the tach; the family's cue lives elsewhere (for example the Sci-Fi gear bay).
    #[default]
    None,
    /// The lit fill holds the palette's `shift` colour at 85% (Modern Digital, Sci-Fi FUI).
    ShiftHold,
    /// Lit segments turn steady `ink`, so only the strip carries the shift colour (Race Dash).
    Ink,
}

impl LimiterCue {
    /// The colour the lit fill crossfades to at the limiter, if any.
    fn color(self, palette: &Palette) -> Option<Color> {
        match self {
            Self::None => None,
            Self::ShiftHold => Some(palette.shift.fade(SHIFT_HOLD_STRENGTH)),
            Self::Ink => Some(palette.roles.ink),
        }
    }

    /// Whether the cue keeps writing light to the bloom buffer. Steady ink does not: a white run
    /// blooming at the limiter would outshine the strobing strip, which must keep the focus.
    fn emissive(self) -> bool {
        !matches!(self, Self::Ink)
    }
}

/// The layer a lit fill draws in and the factor for its explicit glows: the tokens' bloom,
/// fading out while a non-emissive limiter cue (amount `k`) takes over the fill.
fn fill_layer(tokens: &Tokens, cue: LimiterCue, k: f32) -> (LayerOptions, f32) {
    let fade = if cue.emissive() {
        1.0
    } else {
        1.0 - finite(k).clamp(0.0, 1.0)
    };
    if fade <= 1e-3 {
        return (LayerOptions::default(), 0.0);
    }
    let bloom = Bloom {
        threshold: tokens.glow.bloom_threshold,
        intensity: tokens.glow.bloom_intensity * fade,
    };
    (
        LayerOptions {
            bloom: Some(bloom),
            ..LayerOptions::default()
        },
        fade,
    )
}

/// The fill state every variant shares: the rpm fraction on the `ribbon` spring and the limiter
/// amount on the `snap` spring.
///
/// The shown fill is the spring's value held between the previously shown fill and the measured
/// rpm fraction: it only ever moves toward the measured rpm and never passes it. Without that, the
/// feed-forward lead (a smoothed rpm rate that is still large after an upshift drop) and the
/// spring's momentum would show revs the engine does not have, for example a bar dipping three
/// cells below the new rpm for 200 ms after every upshift.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FillModel {
    level: Spring,
    shown: f32,
    limiter: Spring,
    target: f32,
    limiter_on: bool,
    max_rpm: f32,
    shift_ratio: f32,
}

/// `value` held within the span from `from` to `target` (either order).
fn approach(from: f32, value: f32, target: f32) -> f32 {
    let (lo, hi) = if from <= target { (from, target) } else { (target, from) };
    finite(value).clamp(lo, hi)
}

impl FillModel {
    fn new(tokens: &Tokens) -> Self {
        Self {
            level: Spring::new(tokens.springs.ribbon, 0.0),
            shown: 0.0,
            limiter: Spring::new(tokens.springs.snap, 0.0),
            target: 0.0,
            limiter_on: false,
            max_rpm: 0.0,
            shift_ratio: tokens.rev_stages.shift_pct.clamp(0.5, 1.0),
        }
    }

    fn update(&mut self, sig: &Signals, dt: f32) {
        let max_rpm = finite(sig.max_rpm).max(0.0);
        self.max_rpm = max_rpm;
        if max_rpm >= 1.0 && sig.shift_rpm >= 1.0 {
            self.shift_ratio = (sig.shift_rpm / max_rpm).clamp(0.0, 1.0);
        }
        self.target = finite(sig.rpm_fraction).clamp(0.0, 1.0);
        let rate = if max_rpm >= 1.0 {
            finite(sig.rpm_rate) / max_rpm
        } else {
            0.0
        };
        self.level.update_tracking(self.target, rate, dt);
        self.shown = approach(self.shown, self.level.value(), self.target);
        if sig.shift_rpm >= 1.0 {
            if sig.shift_fraction >= 1.0 {
                self.limiter_on = true;
            } else if sig.rpm < sig.shift_rpm - LIMITER_OFF_HYSTERESIS_RPM {
                self.limiter_on = false;
            }
        } else {
            self.limiter_on = false;
        }
        self.limiter.update(self.limiter_goal(), dt);
    }

    fn limiter_goal(&self) -> f32 {
        if self.limiter_on {
            1.0
        } else {
            0.0
        }
    }

    /// The shown rpm fraction, 0..=1.
    fn fraction(&self) -> f32 {
        self.shown.clamp(0.0, 1.0)
    }

    /// The limiter cue amount, 0..=1.
    fn limiter(&self) -> f32 {
        self.limiter.value().clamp(0.0, 1.0)
    }

    fn animating(&self) -> bool {
        !self.level.settled(self.target, 1e-4) || !self.limiter.settled(self.limiter_goal(), 1e-3)
    }

    /// Stage starts as rpm fractions: caution, warning and shift (the go stage keeps the base
    /// colour: no green for nominal).
    fn stage_fractions(&self, palette: &Palette) -> [f32; 3] {
        let s = self.shift_ratio;
        [palette.stages[1] * s, palette.stages[2] * s, s]
    }

    /// Thousands on the scale (0 when the limiter is unknown).
    fn thousands(&self) -> u32 {
        if self.max_rpm < TICK_EVERY_RPM {
            0
        } else {
            ((self.max_rpm / TICK_EVERY_RPM).floor() as u32).min(MAX_THOUSANDS)
        }
    }
}

/// `v`, or 0 when it is not finite.
fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// Segments lit at `fraction` with step lighting: a segment lights once the rpm reaches its top.
fn lit_segments(fraction: f32, count: u32) -> u32 {
    let lit = (finite(fraction).clamp(0.0, 1.0) * count as f32 + 1e-3).floor();
    (lit.max(0.0) as u32).min(count)
}

/// Digit cells for a scale numeral.
fn digits(value: u32) -> u8 {
    if value >= 10 {
        2
    } else {
        1
    }
}

/// Fills `shape` with `base`, crossfaded by `k` to the limiter cue colour when there is one.
fn fill_with_cue(p: &mut Painter<'_>, shape: &Shape<'_>, base: Paint, cue: Option<Color>, k: f32) {
    match cue {
        Some(color) if k > 1e-3 => {
            if k < 0.999 {
                p.fill(shape, base.fade(1.0 - k));
            }
            p.fill(shape, color.fade(k));
        }
        _ => p.fill(shape, base),
    }
}

/// The `arc` tach (Modern, Sci-Fi FUI): a 270-degree ring from 7:30, a redline sector from 92% of
/// the limiter, ticks every 1000 rpm, and a gradient fill (`primary` brightening into the rev
/// stages near the top) with a hot, glowing leading edge. The unlit track is neutral.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TachArc {
    fill: FillModel,
    start: f32,
    sweep: f32,
    cue: LimiterCue,
    light: f32,
}

impl TachArc {
    /// A 270-degree arc from `deg(135)` on the tokens' `ribbon` spring, with the `ShiftHold`
    /// limiter cue and the virtual light at rest (upper left).
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            fill: FillModel::new(tokens),
            start: deg(135.0),
            sweep: deg(270.0),
            cue: LimiterCue::ShiftHold,
            light: deg(-135.0),
        }
    }

    /// The same arc over another clockwise sweep (radians; clamped to 1 degree .. one turn).
    #[must_use]
    pub fn with_sweep(mut self, start: f32, sweep: f32) -> Self {
        self.start = finite(start);
        self.sweep = finite(sweep).clamp(deg(1.0), TAU);
        self
    }

    /// The same arc with another limiter cue.
    #[must_use]
    pub fn with_limiter_cue(mut self, cue: LimiterCue) -> Self {
        self.cue = cue;
        self
    }

    /// Advances with this frame's signals (`rpm_fraction`, `rpm_rate`, `shift_fraction`).
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        self.fill.update(sig, dt);
    }

    /// Sets the virtual light direction (`common::VirtualLight::angle`) for the track rim.
    pub fn set_light(&mut self, angle: f32) {
        self.light = finite(angle);
    }

    /// The shown rpm fraction, 0..=1.
    #[must_use]
    pub fn fraction(&self) -> f32 {
        self.fill.fraction()
    }

    /// The limiter cue amount, 0..=1 (crossfades on the `snap` spring).
    #[must_use]
    pub fn limiter(&self) -> f32 {
        self.fill.limiter()
    }

    /// The angle (radians, clockwise from 3 o'clock) of rpm fraction `fraction` on the scale.
    #[must_use]
    pub fn angle_at(&self, fraction: f32) -> f32 {
        self.start + self.sweep * finite(fraction).clamp(0.0, 1.0)
    }

    /// The leading edge of the fill on the arc's centre line, e.g. to place a light pool behind
    /// glass.
    #[must_use]
    pub fn tip(&self, center: Vec2, radius: f32) -> Vec2 {
        Vec2::polar(center, radius, self.angle_at(self.fraction()))
    }

    /// True while the fill or the limiter cue still moves without new telemetry.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.fill.animating()
    }

    /// The fill paint: a conic ramp from a dim `primary` tail to full `primary`, blending into the
    /// caution and warning stages near the top of the scale.
    fn fill_gradient(&self, center: Vec2, palette: &Palette) -> Gradient {
        let [caution, warning, _] = self.fill.stage_fractions(palette);
        let turn = self.sweep / TAU;
        let t = |f: f32| f * turn;
        let tail = palette.primary.fade(ARC_TAIL_STRENGTH);
        let caution_color = palette.rev_stage(palette.stages[1]);
        let warning_color = palette.rev_stage(palette.stages[2]);
        Gradient::conic(center, self.start)
            .stop(0.0, tail)
            .stop(t(caution.min(0.5)), palette.primary)
            .stop(t(caution - ARC_STAGE_BLEND), palette.primary)
            .stop(t(caution), caution_color)
            .stop(t(warning), warning_color)
            .stop(t(1.0), warning_color)
            // Past the sweep (only anti-aliasing reaches it) the ramp wraps back to the tail.
            .stop(t(1.0) + 1e-3, tail)
            .stop(1.0, tail)
    }

    /// Draws the arc around `center`: track centre line at `radius`, `thickness` wide; the
    /// redline sector and ticks sit just outside it. The fill and its head bloom (glance rank 3).
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, thickness: f32, palette: &Palette) {
        let roles = palette.roles;
        let half = thickness * 0.5;
        let outer = radius + half;

        // Unlit track: a neutral ghost with a 1 px rim lit by the virtual light.
        let track = Shape::arc(center, radius, thickness, self.start, self.sweep, Cap::Butt);
        p.fill(&track, roles.ink_ghost);
        let to_light = Vec2::from_angle(self.light);
        let rim = Gradient::linear(center + to_light * outer, center - to_light * outer)
            .stop(0.0, roles.rim_specular.fade(0.5))
            .stop(0.45, roles.hairline)
            .stop(1.0, roles.hairline.fade(0.2));
        p.stroke(&track, rim, Stroke::rim_px(1.0));

        // Redline sector just outside the track.
        let gap = (thickness * 0.35).max(4.0);
        let band = (thickness * 0.2).max(3.0);
        let red_from = self.angle_at(REDLINE_FROM_RPM_PCT);
        let red_sweep = self.sweep * (1.0 - REDLINE_FROM_RPM_PCT);
        p.fill(
            &Shape::arc(center, outer + gap + band * 0.5, band, red_from, red_sweep, Cap::Butt),
            roles.warning.fade(REDLINE_STRENGTH),
        );

        // Ticks every 1000 rpm (major) and at the half thousands (minor), red in the sector.
        let major_len = (thickness * 1.2).max(12.0);
        self.draw_ticks(p, center, outer + gap + major_len, major_len, palette);

        // The lit fill.
        let frac = self.fraction();
        if frac <= 1e-3 {
            return;
        }
        let k = self.fill.limiter();
        let cue = self.cue.color(palette);
        let fill_sweep = self.sweep * frac;
        let ribbon = self.fill_gradient(center, palette);
        let shape = Shape::arc(center, radius, thickness, self.start, fill_sweep, Cap::Butt);
        let mut head_color = ribbon.sample(frac * self.sweep / TAU);
        if let Some(c) = cue {
            head_color = head_color.lerp(c, k);
        }
        let tip_angle = self.start + fill_sweep;
        let head_len = (thickness * 2.4).min(fill_sweep * radius);
        let head_sweep = head_len / radius.max(1.0);
        let head = Shape::arc(center, radius, thickness, tip_angle - head_sweep, head_sweep, Cap::Butt);
        let head_from = Vec2::polar(center, radius, tip_angle - head_sweep);
        let tip = Vec2::polar(center, radius, tip_angle);
        let radial = Vec2::from_angle(tip_angle);
        let blade = Shape::line(
            tip - radial * (half * 0.92),
            tip + radial * (half * 0.92),
            (thickness * 0.16).max(2.0),
        );
        // Only the tight core lobe of `glow.medium`: the bloom pass already lays the wide halo,
        // and a second wide lobe on top of it smears the head into a blob.
        let core = p.tokens().glow.medium.lobes().first().copied();
        let (layer, glow_fade) = fill_layer(p.tokens(), self.cue, k);
        p.layer(layer, |p| {
            fill_with_cue(p, &shape, ribbon.into(), cue, k);
            // The leading glow is a point light around the blade: glowing the whole head would
            // outline it as a box over the fill behind it.
            if let Some(lobe) = core {
                let hot = head_color.lerp(roles.ink, ARC_HEAD_GLOW_WHITE);
                p.glow(&blade, hot, Glow::new(lobe.sigma, lobe.intensity * glow_fade));
            }
            // The hot head whitens toward the tip, ending in a crisp white blade.
            p.fill(
                &head,
                Gradient::linear(head_from, tip)
                    .stop(0.0, roles.ink.fade(0.0))
                    .stop(1.0, roles.ink.fade(ARC_HEAD_WHITE)),
            );
            p.fill(&blade, roles.ink);
        });
    }

    fn draw_ticks(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, major_len: f32, palette: &Palette) {
        let thousands = self.fill.thousands();
        if thousands == 0 {
            return;
        }
        let roles = palette.roles;
        let max = self.fill.max_rpm;
        let step = self.sweep * TICK_EVERY_RPM / max;
        let red_rpm = REDLINE_FROM_RPM_PCT * max;
        let majors = thousands + 1;
        let first_red = ((red_rpm / TICK_EVERY_RPM).ceil().max(0.0) as u32).min(majors);
        let minors = ((max / TICK_EVERY_RPM - 0.5).floor() + 1.0).clamp(0.0, MAX_THOUSANDS as f32) as u32;
        let first_red_minor = ((red_rpm / TICK_EVERY_RPM - 0.5).ceil().max(0.0) as u32).min(minors);
        let minor_len = major_len * 0.5;
        let major = |from: u32, to: u32| TickRun {
            center,
            radius,
            first: self.start,
            step,
            from,
            to,
            length: major_len,
            width: 3.0,
        };
        let minor = |from: u32, to: u32| TickRun {
            center,
            radius,
            first: self.start + step * 0.5,
            step,
            from,
            to,
            length: minor_len,
            width: 2.0,
        };
        // Majors carry the scale like the numerals (label ink); minors stay hairlines.
        major(0, first_red).draw(p, roles.ink_label);
        major(first_red, majors).draw(p, roles.warning.fade(REDLINE_STRENGTH));
        minor(0, first_red_minor).draw(p, roles.hairline);
        minor(first_red_minor, minors).draw(p, roles.warning.fade(REDLINE_STRENGTH * 0.6));
    }

    /// Draws the scale numerals (thousands) at `radius` around `center`, `cap` px tall, in the
    /// label ink (text-safe warning inside the redline sector). Optional: clusters may set their
    /// own numerals.
    pub fn draw_numerals(
        &self,
        p: &mut Painter<'_>,
        center: Vec2,
        radius: f32,
        font: FontId,
        cap: f32,
        palette: &Palette,
    ) {
        let thousands = self.fill.thousands();
        if thousands == 0 {
            return;
        }
        let max = self.fill.max_rpm;
        let base = TextStyle::new(font, cap)
            .paint(palette.roles.ink_label)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let red = base.paint(palette.roles.warning.text_safe());
        for k in 0..=thousands {
            let rpm = k as f32 * TICK_EVERY_RPM;
            let style = if rpm >= REDLINE_FROM_RPM_PCT * max - 1.0 {
                &red
            } else {
                &base
            };
            let at = Vec2::polar(center, radius, self.angle_at(rpm / max));
            p.number(at, i64::from(k), digits(k), style, None);
        }
    }
}

/// A run of evenly spaced radial ticks `from..to` (indices on a ring that starts at `first`).
#[derive(Debug, Clone, Copy)]
struct TickRun {
    center: Vec2,
    radius: f32,
    first: f32,
    step: f32,
    from: u32,
    to: u32,
    length: f32,
    width: f32,
}

impl TickRun {
    fn draw(&self, p: &mut Painter<'_>, color: Color) {
        if self.to <= self.from {
            return;
        }
        let count = self.to - self.from;
        let start = self.first + self.step * self.from as f32;
        let ring = TickRing::new(self.center, self.radius, start, self.step * (count - 1) as f32, count)
            .minor(self.length, self.width);
        p.fill(&Shape::Ticks(ring), color);
    }
}

/// Peak hold of the rev bar: once the lit run drops below the highest cell it reached, that cell
/// stays marked for `hold` seconds, then the marker falls back onto the run on a spring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PeakHold {
    level: Spring,
    hold: f32,
    left: f32,
}

impl PeakHold {
    fn update(&mut self, fraction: f32, count: u32, dt: f32) {
        let dt = finite(dt).max(0.0);
        if fraction >= self.level.value() || !self.visible(fraction, count) {
            // A new peak, or the run still lights the peak cell (a settling spring's sub-cell
            // overshoot is not a peak): the hold starts when the run falls below the cell.
            self.level.reset(fraction);
            self.left = self.hold;
        } else if self.left > 0.0 {
            self.left = (self.left - dt).max(0.0);
        } else {
            self.level.update(fraction, dt);
        }
    }

    /// True while the held cell sits above the lit run.
    fn visible(&self, fraction: f32, count: u32) -> bool {
        lit_segments(self.level.value(), count) > lit_segments(fraction, count)
    }
}

/// The `bar` tach (Race Dash): a segmented linear bar (45 segments, 5 px gaps) with step
/// lighting. Lit cells are 72% of `primary` and take the caution and warning stage colours above
/// the car's thresholds (Race Dash: 7300 / 8000 rpm); at the limiter they turn steady `ink`
/// without bloom, so the strobing strip keeps the focus. Lit cells are top-lit lenses (glint and
/// bottom shade); unlit cells are grey lenses. Optional peak hold.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TachBar {
    fill: FillModel,
    count: u32,
    gap: f32,
    cue: LimiterCue,
    peak: Option<PeakHold>,
}

impl TachBar {
    /// The Race Dash bar: 45 segments, 5 px gaps, the `Ink` limiter cue, no peak hold.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            fill: FillModel::new(tokens),
            count: BAR_SEGMENTS,
            gap: BAR_GAP,
            cue: LimiterCue::Ink,
            peak: None,
        }
    }

    /// The same bar with `count` segments (1..=120) and `gap` px between them.
    #[must_use]
    pub fn with_segments(mut self, count: u32, gap: f32) -> Self {
        self.count = count.clamp(1, MAX_SEGMENTS);
        self.gap = finite(gap).max(0.0);
        self
    }

    /// The same bar with another limiter cue.
    #[must_use]
    pub fn with_limiter_cue(mut self, cue: LimiterCue) -> Self {
        self.cue = cue;
        self
    }

    /// The same bar with a peak-hold cell: it holds for `durations.long` (500 ms), then falls on
    /// the `slow` spring.
    #[must_use]
    pub fn with_peak_hold(mut self, tokens: &Tokens) -> Self {
        self.peak = Some(PeakHold {
            level: Spring::new(tokens.springs.slow, 0.0),
            hold: tokens.durations.long,
            left: 0.0,
        });
        self
    }

    /// Advances with this frame's signals (`rpm_fraction`, `rpm_rate`, `shift_fraction`).
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        self.fill.update(sig, dt);
        let fraction = self.fill.fraction();
        let count = self.count;
        if let Some(peak) = &mut self.peak {
            peak.update(fraction, count, dt);
        }
    }

    /// Number of segments.
    #[must_use]
    pub fn count(&self) -> u32 {
        self.count
    }

    /// Lit segments (step lighting).
    #[must_use]
    pub fn lit(&self) -> u32 {
        lit_segments(self.fill.fraction(), self.count)
    }

    /// The limiter cue amount, 0..=1.
    #[must_use]
    pub fn limiter(&self) -> f32 {
        self.fill.limiter()
    }

    /// The held peak segment while it sits above the lit run, if peak hold is on.
    #[must_use]
    pub fn peak_segment(&self) -> Option<u32> {
        let peak = self.peak?;
        let top = lit_segments(peak.level.value(), self.count);
        (top > self.lit()).then(|| top - 1)
    }

    /// True while the fill, the limiter cue or a shown peak marker still moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.fill.animating() || self.peak.is_some_and(|pk| pk.visible(self.fill.fraction(), self.count))
    }

    /// Cell pitch and width in `rect`.
    fn cell_metrics(&self, rect: Rect) -> (f32, f32) {
        let n = self.count as f32;
        let cell = ((rect.width() - self.gap * (n - 1.0)) / n).max(1e-3);
        (cell + self.gap, cell)
    }

    /// The rectangle of segment `i` in `rect`.
    #[must_use]
    pub fn cell_rect(&self, rect: Rect, i: u32) -> Rect {
        let (pitch, cell) = self.cell_metrics(rect);
        Rect::from_xywh(rect.min.x + i as f32 * pitch, rect.min.y, cell, rect.height())
    }

    /// The gradient offset where cells lit at or above rpm fraction `f` begin: the left edge of
    /// the first segment that lights at `f` or later.
    fn stop_offset(&self, rect: Rect, f: f32) -> f32 {
        let (pitch, _) = self.cell_metrics(rect);
        let first = ((f * self.count as f32).ceil() - 1.0).clamp(0.0, self.count as f32);
        ((first * pitch - self.gap * 0.5) / rect.width().max(1e-3)).clamp(0.0, 1.0)
    }

    /// The lit-cell paint: a hard-stop ramp across the bar; each lit cell takes the colour at its
    /// centre, so a cell lights in the stage of the rpm at which it lights. Cells past the shift
    /// rpm stay in the warning stage: the shift colour belongs to the strip and the REV lamp,
    /// and at the limiter the lit run turns to the limiter cue instead (Race Dash: steady ink).
    fn paint(&self, rect: Rect, palette: &Palette) -> Gradient {
        let [caution, warning, _] = self.fill.stage_fractions(palette);
        let (c, w) = (self.stop_offset(rect, caution), self.stop_offset(rect, warning));
        let base = palette.primary.fade(BAR_BASE_STRENGTH);
        let caution_color = palette.rev_stage(palette.stages[1]);
        let warning_color = palette.rev_stage(palette.stages[2]);
        let y = rect.center().y;
        Gradient::linear(vec2(rect.min.x, y), vec2(rect.max.x, y))
            .stop(0.0, base)
            .stop(c, base)
            .stop(c, caution_color)
            .stop(w, caution_color)
            .stop(w, warning_color)
            .stop(1.0, warning_color)
    }

    /// The colour segment `i` shows when lit (before the limiter cue).
    #[must_use]
    pub fn segment_color(&self, rect: Rect, i: u32, palette: &Palette) -> Color {
        let cell = self.cell_rect(rect, i);
        let offset = (cell.center().x - rect.min.x) / rect.width().max(1e-3);
        self.paint(rect, palette).sample(offset)
    }

    /// Draws the bar in `rect` (segments fill left to right). Lit cells bloom (glance rank 3).
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette) {
        let roles = palette.roles;
        let (_, cell) = self.cell_metrics(rect);
        let radius = cell.min(rect.height()) * 0.14;
        let lit = self.lit() as f32;
        let bar = |r: Rect, radius: f32, unlit: Color| {
            Shape::Bar(SegmentedBar {
                rect: r,
                count: self.count,
                gap: self.gap,
                lit,
                radius,
                unlit,
                axis: Axis::Horizontal,
            })
        };

        // Unlit cells: grey lenses (ghost body, 1 px neutral rim); lit cells are skipped here.
        p.fill(&bar(rect, radius, roles.ink_ghost), Color::TRANSPARENT);
        p.stroke(
            &bar(rect, radius, roles.hairline.fade(BAR_UNLIT_RIM)),
            Color::TRANSPARENT,
            Stroke::rim_px(1.0),
        );

        let paint = self.paint(rect, palette);
        if lit > 0.0 {
            let k = self.fill.limiter();
            let cue = self.cue.color(palette);
            let lit_bar = bar(rect, radius, Color::TRANSPARENT);
            let (layer, _) = fill_layer(p.tokens(), self.cue, k);
            p.layer(layer, |p| {
                fill_with_cue(p, &lit_bar, paint.into(), cue, k);
            });
            // Lens optics on every lit cell (also the steady ink cue): a glint fading down from
            // the top edge and a shade toward the bottom, so a cell reads as a lit lens. Drawn
            // per cell: the kit samples a bar's paint once at each cell centre, so this gradient
            // on the bar shape would collapse to its (transparent) middle.
            let x = rect.min.x;
            let lens = Gradient::linear(vec2(x, rect.min.y), vec2(x, rect.max.y))
                .stop(0.0, roles.ink.fade(BAR_LIT_GLINT))
                .stop(BAR_GLINT_HEIGHT, roles.ink.fade(0.0))
                .stop(0.55, roles.oled_black.fade(0.0))
                .stop(1.0, roles.oled_black.fade(BAR_LENS_SHADE));
            for i in 0..self.lit() {
                p.fill(&Shape::round_rect(self.cell_rect(rect, i), radius), lens);
            }
        }

        if let Some(i) = self.peak_segment() {
            let cell = self.cell_rect(rect, i);
            let offset = (cell.center().x - rect.min.x) / rect.width().max(1e-3);
            let color = paint.sample(offset);
            let shape = Shape::round_rect(cell, radius);
            p.fill(&shape, color.fade(BAR_PEAK_STRENGTH));
            p.stroke(&shape, color, Stroke::rim_px(1.0));
        }
    }

    /// Draws the scale numerals (thousands) centred under `rect`, `cap` px tall, in the label ink.
    pub fn draw_scale(&self, p: &mut Painter<'_>, rect: Rect, font: FontId, cap: f32, palette: &Palette) {
        let thousands = self.fill.thousands();
        if thousands == 0 {
            return;
        }
        let max = self.fill.max_rpm;
        let style = TextStyle::new(font, cap)
            .paint(palette.roles.ink_label)
            .align(Align::Center, VAlign::CapTop)
            .tabular();
        let y = rect.max.y + cap * 0.9;
        for k in 0..=thousands {
            let x = rect.min.x + rect.width() * (k as f32 * TICK_EVERY_RPM / max);
            p.number(vec2(x, y), i64::from(k), digits(k), &style, None);
        }
    }
}

/// Colour zones of a chevron run, counted from the left (Sci-Fi: 14 cyan, 6 yellow, 4 magenta).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChevronZones {
    /// Segments in `primary` (phosphor cyan in the Sci-Fi family).
    pub primary: u32,
    /// Segments in `secondary` (caution yellow).
    pub secondary: u32,
    /// Segments in `accent` (magenta: the top tach zone).
    pub accent: u32,
}

impl Default for ChevronZones {
    fn default() -> Self {
        Self {
            primary: 14,
            secondary: 6,
            accent: 4,
        }
    }
}

impl ChevronZones {
    /// All segments.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.primary.saturating_add(self.secondary).saturating_add(self.accent)
    }
}

/// The outline of each chevron segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChevronForm {
    /// A parallelogram leaning forward 14 degrees (the RetroCyber staircase).
    #[default]
    Slant,
    /// An arrow block with a notch on the left and a point on the right.
    Arrow,
}

/// The `chevron` tach (Sci-Fi VFD): a staircase of 24 stepped segments rising left to right
/// (14 `primary`, 6 `secondary`, 4 `accent`), step lighting, lit segments in phosphor with a
/// whiter core and a bloom halo, unlit segments as ghosts at 7% of their colour. Steady at the
/// limiter by default (the family's cue is the gear bay).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TachChevron {
    fill: FillModel,
    zones: ChevronZones,
    form: ChevronForm,
    cue: LimiterCue,
    rise: f32,
}

impl TachChevron {
    /// The Sci-Fi run: 14 / 6 / 4 slanted segments, the first 30% of the last's height, no
    /// limiter cue.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            fill: FillModel::new(tokens),
            zones: ChevronZones::default(),
            form: ChevronForm::Slant,
            cue: LimiterCue::None,
            rise: CHEVRON_RISE,
        }
    }

    /// The same run with other colour zones (at least one segment, at most 120 in total).
    #[must_use]
    pub fn with_zones(mut self, zones: ChevronZones) -> Self {
        if (1..=MAX_SEGMENTS).contains(&zones.total()) {
            self.zones = zones;
        }
        self
    }

    /// The same run with another segment outline.
    #[must_use]
    pub fn with_form(mut self, form: ChevronForm) -> Self {
        self.form = form;
        self
    }

    /// The same run with another limiter cue.
    #[must_use]
    pub fn with_limiter_cue(mut self, cue: LimiterCue) -> Self {
        self.cue = cue;
        self
    }

    /// The same run with the first segment at `rise` of the last segment's height (0.05..=1;
    /// 1 is a flat run).
    #[must_use]
    pub fn with_rise(mut self, rise: f32) -> Self {
        self.rise = finite(rise).clamp(0.05, 1.0);
        self
    }

    /// Advances with this frame's signals (`rpm_fraction`, `rpm_rate`, `shift_fraction`).
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        self.fill.update(sig, dt);
    }

    /// Number of segments.
    #[must_use]
    pub fn count(&self) -> u32 {
        self.zones.total()
    }

    /// Lit segments (step lighting).
    #[must_use]
    pub fn lit(&self) -> u32 {
        lit_segments(self.fill.fraction(), self.count())
    }

    /// The limiter cue amount, 0..=1.
    #[must_use]
    pub fn limiter(&self) -> f32 {
        self.fill.limiter()
    }

    /// True while the fill or the limiter cue still moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.fill.animating()
    }

    /// The zone colour of segment `i` (before the limiter cue).
    #[must_use]
    pub fn segment_color(&self, i: u32, palette: &Palette) -> Color {
        if i < self.zones.primary {
            palette.primary
        } else if i < self.zones.primary + self.zones.secondary {
            palette.secondary
        } else {
            palette.accent
        }
    }

    /// The forward reach of a segment of height `h` beyond its base width.
    fn reach(&self, h: f32) -> f32 {
        match self.form {
            ChevronForm::Slant => CHEVRON_LEAN * h,
            ChevronForm::Arrow => CHEVRON_ARROW_DEPTH * h,
        }
    }

    /// The outline of segment `i` in `rect` (bottom-aligned, rising left to right) and its
    /// point count (4 for a slant, 6 for an arrow). The run exactly spans `rect`.
    #[must_use]
    pub fn segment(&self, i: u32, rect: Rect) -> ([Vec2; 6], usize) {
        let n = self.count().max(1) as f32;
        let h_max = rect.height().max(0.0);
        let available = (rect.width() - self.reach(h_max)).max(0.0);
        let pitch = available / (n - CHEVRON_GAP).max(1e-3);
        let w = pitch * (1.0 - CHEVRON_GAP);
        let t = if n > 1.0 { i as f32 / (n - 1.0) } else { 1.0 };
        let h = h_max * (self.rise + (1.0 - self.rise) * t.clamp(0.0, 1.0));
        let x = rect.min.x + i as f32 * pitch;
        let y = rect.max.y;
        let d = self.reach(h);
        match self.form {
            ChevronForm::Slant => (
                [
                    vec2(x, y),
                    vec2(x + w, y),
                    vec2(x + w + d, y - h),
                    vec2(x + d, y - h),
                    Vec2::ZERO,
                    Vec2::ZERO,
                ],
                4,
            ),
            ChevronForm::Arrow => (
                [
                    vec2(x, y),
                    vec2(x + w, y),
                    vec2(x + w + d, y - h * 0.5),
                    vec2(x + w, y - h),
                    vec2(x, y - h),
                    vec2(x + d, y - h * 0.5),
                ],
                6,
            ),
        }
    }

    /// Draws the run in `rect`. Lit segments bloom (glance rank 3): the bloom pass lays the short
    /// phosphor halo. The explicit `glow.phosphor` lobes are sized for thin 7-segment strokes; on
    /// 24 broad blocks their sub-unity edge reads as a dark outline, so the run does not add them.
    pub fn draw(&self, p: &mut Painter<'_>, rect: Rect, palette: &Palette) {
        let n = self.count();
        let lit = self.lit();
        for i in lit..n {
            let (pts, len) = self.segment(i, rect);
            let ghost = self.segment_color(i, palette).fade(GHOST_STRENGTH);
            p.fill(&Shape::Polygon { points: &pts[..len] }, ghost);
        }
        if lit == 0 {
            return;
        }
        let k = self.fill.limiter();
        let cue = self.cue.color(palette);
        let ink = palette.roles.ink;
        let (layer, _) = fill_layer(p.tokens(), self.cue, k);
        p.layer(layer, |p| {
            for i in 0..lit {
                let (pts, len) = self.segment(i, rect);
                let shape = Shape::Polygon { points: &pts[..len] };
                let mut color = self.segment_color(i, palette);
                if let Some(c) = cue {
                    color = color.lerp(c, k);
                }
                // Phosphor body in full hue with a whiter core band through the middle (the
                // filament's line), so a lit block reads as light, not paint.
                let bottom = (pts[0] + pts[1]) * 0.5;
                let top = vec2(bottom.x, bottom.y - (pts[0].y - rect_top(&pts[..len])));
                let hot = color.lerp(ink, CHEVRON_CORE_WHITE);
                p.fill(
                    &shape,
                    Gradient::linear(bottom, top)
                        .stop(0.0, color)
                        .stop(0.5 - CHEVRON_CORE_HALF, color)
                        .stop(0.5, hot)
                        .stop(0.5 + CHEVRON_CORE_HALF, color)
                        .stop(1.0, color),
                );
            }
        });
    }

    /// Draws the scale numerals (thousands) under `rect`, `cap` px tall, in the label ink.
    pub fn draw_scale(&self, p: &mut Painter<'_>, rect: Rect, font: FontId, cap: f32, palette: &Palette) {
        let thousands = self.fill.thousands();
        if thousands == 0 {
            return;
        }
        let max = self.fill.max_rpm;
        let style = TextStyle::new(font, cap)
            .paint(palette.roles.ink_label)
            .align(Align::Center, VAlign::CapTop)
            .tabular();
        let (first, _) = self.segment(0, rect);
        let (last, _) = self.segment(self.count().saturating_sub(1), rect);
        let span = last[1].x - first[0].x;
        let y = rect.max.y + cap * 0.9;
        for k in 0..=thousands {
            let x = first[0].x + span * (k as f32 * TICK_EVERY_RPM / max);
            p.number(vec2(x, y), i64::from(k), digits(k), &style, None);
        }
    }
}

/// The smallest y (the top) of a segment outline.
fn rect_top(pts: &[Vec2]) -> f32 {
    pts.iter().fold(f32::INFINITY, |top, v| top.min(v.y))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITER: f32 = 8700.0;
    const DT: f32 = 1.0 / 60.0;

    fn sig(tokens: &Tokens, rpm: f32) -> Signals {
        let shift = LIMITER * tokens.rev_stages.shift_pct;
        Signals {
            rpm,
            max_rpm: LIMITER,
            idle_rpm: 900.0,
            shift_rpm: shift,
            rpm_fraction: (rpm / LIMITER).clamp(0.0, 1.0),
            shift_fraction: rpm / shift,
            ..Signals::default()
        }
    }

    fn settle_fill(tokens: &Tokens, rpm: f32, frames: u32) -> FillModel {
        let mut f = FillModel::new(tokens);
        for _ in 0..frames {
            f.update(&sig(tokens, rpm), DT);
        }
        f
    }

    #[test]
    fn fill_settles_on_the_rpm_fraction() {
        let t = Tokens::default();
        let f = settle_fill(&t, 5200.0, 30);
        assert!(
            (f.fraction() - 5200.0 / LIMITER).abs() < 1e-3,
            "fraction {}",
            f.fraction()
        );
        assert!(!f.animating());
        assert_eq!(f.limiter(), 0.0);
    }

    #[test]
    fn feed_forward_keeps_a_climbing_fill_within_one_frame() {
        let t = Tokens::default();
        let mut f = FillModel::new(&t);
        // A steady 6000 rpm/s pull from 2000 rpm.
        let rate = 6000.0;
        let mut rpm = 2000.0;
        let mut s = sig(&t, rpm);
        for _ in 0..40 {
            rpm += rate * DT;
            s = Signals {
                rpm_rate: rate,
                ..sig(&t, rpm)
            };
            f.update(&s, DT);
        }
        let lag = (s.rpm_fraction - f.fraction()).abs();
        let one_frame = rate * DT / LIMITER;
        assert!(lag < one_frame, "lag {lag} vs one frame {one_frame}");
    }

    #[test]
    fn limiter_cue_has_off_hysteresis_and_never_flickers() {
        let t = Tokens::default();
        let shift = LIMITER * t.rev_stages.shift_pct;
        let mut f = settle_fill(&t, shift + 50.0, 30);
        assert!(f.limiter() > 0.99, "on at the shift rpm: {}", f.limiter());
        // Bouncing just under the shift rpm keeps it on.
        for _ in 0..30 {
            f.update(&sig(&t, shift - 60.0), DT);
        }
        assert!(f.limiter() > 0.99, "held within 120 rpm: {}", f.limiter());
        for _ in 0..30 {
            f.update(&sig(&t, shift - 400.0), DT);
        }
        assert!(f.limiter() < 0.01, "off below the hysteresis: {}", f.limiter());
    }

    #[test]
    fn limiter_crossfades_instead_of_snapping() {
        let t = Tokens::default();
        let shift = LIMITER * t.rev_stages.shift_pct;
        let mut f = settle_fill(&t, 5000.0, 30);
        f.update(&sig(&t, shift + 100.0), DT);
        let first = f.limiter();
        assert!(first > 0.0 && first < 0.9, "one frame in: {first}");
        assert!(f.animating());
    }

    #[test]
    fn step_lighting_counts_whole_segments() {
        assert_eq!(lit_segments(0.0, 45), 0);
        assert_eq!(lit_segments(0.5, 45), 22);
        assert_eq!(lit_segments(1.0, 45), 45);
        assert_eq!(lit_segments(8680.0 / LIMITER, 24), 23);
        assert_eq!(lit_segments(f32::NAN, 24), 0);
        assert_eq!(lit_segments(2.0, 24), 24);
    }

    #[test]
    fn arc_tip_and_angles_follow_the_sweep() {
        let t = Tokens::default();
        let mut arc = TachArc::new(&t);
        for _ in 0..40 {
            arc.update(&sig(&t, LIMITER * 0.5), DT);
        }
        let tip = arc.tip(Vec2::ZERO, 100.0);
        // Half of 270 degrees from 7:30 is 12 o'clock (y up is negative).
        assert!(tip.x.abs() < 0.5 && (tip.y + 100.0).abs() < 0.5, "tip {tip:?}");
        assert!((arc.angle_at(0.0) - deg(135.0)).abs() < 1e-5);
        assert!((arc.angle_at(1.0) - deg(405.0)).abs() < 1e-4);
    }

    #[test]
    fn arc_gradient_is_primary_mid_scale_and_staged_at_the_top() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "modernDigital");
        let mut arc = TachArc::new(&t);
        arc.update(&sig(&t, 1000.0), DT);
        let g = arc.fill_gradient(Vec2::ZERO, &pal);
        let at = |f: f32| g.sample(f * 0.75);
        assert_eq!(at(0.6), pal.primary);
        assert_eq!(at(0.98), pal.roles.warning);
        assert!(at(0.0).a < pal.primary.a, "the tail is dimmer than the body");
    }

    #[test]
    fn bar_cells_take_the_stage_of_the_rpm_they_light_at() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let mut bar = TachBar::new(&t);
        bar.update(&sig(&t, 3000.0), DT);
        let rect = Rect::from_xywh(-500.0, 0.0, 1000.0, 20.0);
        let shift_ratio = t.rev_stages.shift_pct;
        let n = bar.count();
        for i in 0..n {
            let lights_at = (i + 1) as f32 / n as f32;
            let want = if lights_at >= pal.stages[2] * shift_ratio {
                pal.roles.warning
            } else if lights_at >= pal.stages[1] * shift_ratio {
                pal.roles.caution
            } else {
                pal.primary.fade(BAR_BASE_STRENGTH)
            };
            assert_eq!(
                bar.segment_color(rect, i, &pal),
                want,
                "segment {i} lights at {lights_at}"
            );
        }
    }

    #[test]
    fn bar_cells_span_the_rect() {
        let t = Tokens::default();
        let bar = TachBar::new(&t);
        let rect = Rect::from_xywh(-500.0, 0.0, 1000.0, 20.0);
        let first = bar.cell_rect(rect, 0);
        let last = bar.cell_rect(rect, bar.count() - 1);
        assert!((first.min.x - rect.min.x).abs() < 1e-3);
        assert!((last.max.x - rect.max.x).abs() < 1e-2);
        let second = bar.cell_rect(rect, 1);
        assert!((second.min.x - first.max.x - BAR_GAP).abs() < 1e-3);
    }

    #[test]
    fn bar_peak_holds_then_falls() {
        let t = Tokens::default();
        let mut bar = TachBar::new(&t).with_peak_hold(&t);
        for _ in 0..30 {
            bar.update(&sig(&t, 8000.0), DT);
        }
        assert_eq!(bar.peak_segment(), None, "no marker while the peak is lit");
        let held = bar.lit() - 1;
        for _ in 0..12 {
            bar.update(&sig(&t, 5000.0), DT);
        }
        assert_eq!(bar.peak_segment(), Some(held), "held for 500 ms");
        assert!(bar.animating());
        for _ in 0..120 {
            bar.update(&sig(&t, 5000.0), DT);
        }
        assert_eq!(bar.peak_segment(), None, "falls back onto the run");
        assert!(!bar.animating());
    }

    #[test]
    fn chevron_zones_follow_the_family_roles() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "sciFi");
        let ch = TachChevron::new(&t);
        assert_eq!(ch.count(), 24);
        assert_eq!(ch.segment_color(0, &pal), pal.primary);
        assert_eq!(ch.segment_color(13, &pal), pal.primary);
        assert_eq!(ch.segment_color(14, &pal), pal.secondary);
        assert_eq!(ch.segment_color(19, &pal), pal.secondary);
        assert_eq!(ch.segment_color(20, &pal), pal.accent);
        assert_eq!(ch.segment_color(23, &pal), pal.accent);
        let ghost = pal.primary.fade(GHOST_STRENGTH);
        assert!((ghost.a - 0.07 * pal.primary.a).abs() < 1e-6);
    }

    #[test]
    fn chevron_staircase_rises_without_overlap_and_spans_the_rect() {
        let t = Tokens::default();
        let rect = Rect::from_xywh(-600.0, -40.0, 1200.0, 80.0);
        for form in [ChevronForm::Slant, ChevronForm::Arrow] {
            let ch = TachChevron::new(&t).with_form(form);
            let mut last_h = 0.0;
            for i in 0..ch.count() {
                let (pts, len) = ch.segment(i, rect);
                let pts = &pts[..len];
                let h = rect.max.y - rect_top(pts);
                assert!(h > last_h, "{form:?} segment {i} rises");
                last_h = h;
                assert!(pts.iter().all(|v| v.y <= rect.max.y + 1e-3 && v.y >= rect.min.y - 1e-3));
                if i + 1 < ch.count() {
                    let (next, next_len) = ch.segment(i + 1, rect);
                    let right = pts.iter().fold(f32::NEG_INFINITY, |m, v| m.max(v.x));
                    // The next segment starts after this one's base and reaches further forward.
                    assert!(next[0].x > pts[1].x, "{form:?} gap at the base of {i}");
                    let next_right = next[..next_len].iter().fold(f32::NEG_INFINITY, |m, v| m.max(v.x));
                    assert!(next_right > right);
                }
            }
            assert!((last_h - rect.height()).abs() < 1e-3);
            let (first, _) = ch.segment(0, rect);
            let (end, end_len) = ch.segment(ch.count() - 1, rect);
            let right = end[..end_len].iter().fold(f32::NEG_INFINITY, |m, v| m.max(v.x));
            assert!((first[0].x - rect.min.x).abs() < 1e-3);
            assert!((right - rect.max.x).abs() < 1e-2, "{form:?} right edge {right}");
        }
    }

    #[test]
    fn chevron_ignores_empty_zones() {
        let t = Tokens::default();
        let ch = TachChevron::new(&t).with_zones(ChevronZones {
            primary: 0,
            secondary: 0,
            accent: 0,
        });
        assert_eq!(ch.count(), 24);
    }

    #[test]
    fn limiter_cue_colours() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "modernDigital");
        assert_eq!(LimiterCue::None.color(&pal), None);
        assert_eq!(LimiterCue::Ink.color(&pal), Some(pal.roles.ink));
        let hold = LimiterCue::ShiftHold.color(&pal).map(|c| c.a);
        assert_eq!(hold, Some(SHIFT_HOLD_STRENGTH));
    }

    #[test]
    fn steady_ink_cue_stops_blooming_but_shift_hold_keeps_its_light() {
        let t = Tokens::default();
        let bloom = |cue: LimiterCue, k: f32| {
            let (layer, fade) = fill_layer(&t, cue, k);
            (layer.bloom.map(|b| b.intensity), fade)
        };
        let full = Some(t.glow.bloom_intensity);
        assert_eq!(bloom(LimiterCue::Ink, 0.0), (full, 1.0));
        assert_eq!(
            bloom(LimiterCue::Ink, 1.0),
            (None, 0.0),
            "a white run at the limiter never glares"
        );
        let (half, fade) = bloom(LimiterCue::Ink, 0.5);
        assert!(half.is_some_and(|i| (i - t.glow.bloom_intensity * 0.5).abs() < 1e-6) && (fade - 0.5).abs() < 1e-6);
        assert_eq!(bloom(LimiterCue::ShiftHold, 1.0), (full, 1.0));
        assert_eq!(bloom(LimiterCue::None, 1.0), (full, 1.0));
        assert_eq!(
            bloom(LimiterCue::Ink, f32::NAN),
            (full, 1.0),
            "a corrupt cue amount keeps the fill lit"
        );
    }

    #[test]
    fn bar_cells_past_the_shift_rpm_stay_in_the_warning_stage() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let bar = TachBar::new(&t);
        let rect = Rect::from_xywh(0.0, 0.0, 1000.0, 36.0);
        let top = bar.segment_color(rect, bar.count() - 1, &pal);
        assert_eq!(
            top, pal.roles.warning,
            "the shift colour belongs to the strip and the REV lamp"
        );
        assert_ne!(top, pal.shift);
    }

    #[test]
    fn a_settled_bar_with_peak_hold_stops_animating() {
        let t = Tokens::default();
        let mut bar = TachBar::new(&t).with_peak_hold(&t);
        // The ribbon spring overshoots by a fraction of a cell on the way up; that is no peak.
        for _ in 0..120 {
            bar.update(&sig(&t, 7400.0), DT);
        }
        assert_eq!(bar.peak_segment(), None);
        assert!(!bar.animating(), "nothing moves at a steady rpm");
    }
    /// Feeds engine rpm with the rate `SignalSmoother` derives from it (the per-frame
    /// difference with 40 ms smoothing), so the feed-forward sees what it sees in the game.
    struct Feed {
        fill: FillModel,
        rate: fh6_kit::Smoothed<f32>,
        last: f32,
    }

    impl Feed {
        fn settled(tokens: &Tokens, rpm: f32) -> Self {
            let mut feed = Self {
                fill: FillModel::new(tokens),
                rate: fh6_kit::Smoothed::new(0.0, 0.04),
                last: rpm,
            };
            for _ in 0..60 {
                feed.step(tokens, rpm);
            }
            feed
        }

        /// One frame at `rpm`; returns the shown fraction.
        fn step(&mut self, tokens: &Tokens, rpm: f32) -> f32 {
            let rate = self.rate.update((rpm - self.last) / DT, DT);
            self.last = rpm;
            self.fill.update(
                &Signals {
                    rpm_rate: rate,
                    ..sig(tokens, rpm)
                },
                DT,
            );
            self.fill.fraction()
        }
    }

    #[test]
    fn an_upshift_drop_never_shows_the_fill_below_the_new_rpm() {
        let t = Tokens::default();
        let mut feed = Feed::settled(&t, 8400.0);
        let target = 6000.0 / LIMITER;
        let mut frames_to_land = None;
        for frame in 0..40 {
            let shown = feed.step(&t, 6000.0);
            assert!(
                shown >= target - 1e-6,
                "frame {frame}: shown {shown} below the rpm {target}"
            );
            assert!(
                lit_segments(shown, BAR_SEGMENTS) >= lit_segments(target, BAR_SEGMENTS),
                "frame {frame}: a bar cell drops below the rpm"
            );
            if frames_to_land.is_none() && (shown - target).abs() < 1e-4 {
                frames_to_land = Some(frame + 1);
            }
        }
        assert!(
            frames_to_land.is_some_and(|n| n <= 3),
            "lands on the new rpm within 50 ms: {frames_to_land:?}"
        );
    }

    #[test]
    fn a_downshift_blip_never_shows_the_fill_above_the_rpm() {
        let t = Tokens::default();
        let mut feed = Feed::settled(&t, 5000.0);
        let target = 7500.0 / LIMITER;
        for frame in 0..40 {
            let shown = feed.step(&t, 7500.0);
            assert!(
                shown <= target + 1e-6,
                "frame {frame}: shown {shown} above the rpm {target}"
            );
        }
        assert!((feed.fill.fraction() - target).abs() < 1e-4);
    }

    #[test]
    fn a_lift_after_a_pull_falls_monotonically_and_keeps_up() {
        let t = Tokens::default();
        let mut feed = Feed::settled(&t, 3000.0);
        let mut rpm = 3000.0;
        let mut peak_target = 0.0f32;
        while rpm < 8000.0 {
            rpm += 6000.0 * DT;
            feed.step(&t, rpm);
            peak_target = peak_target.max(rpm / LIMITER);
        }
        let mut last = feed.fill.fraction();
        assert!(last <= peak_target + 1e-6, "never ahead of the pull");
        for frame in 0..30 {
            rpm -= 8000.0 * DT;
            let shown = feed.step(&t, rpm);
            assert!(
                shown <= last + 1e-6,
                "frame {frame}: the fill rises during a lift ({last} -> {shown})"
            );
            last = shown;
        }
        let lag = feed.fill.fraction() - rpm / LIMITER;
        let one_frame = 8000.0 * DT / LIMITER;
        assert!((0.0..one_frame).contains(&lag), "lag {lag} vs one frame {one_frame}");
    }

    #[test]
    fn a_pull_right_after_an_upshift_stays_honest_and_catches_up() {
        let t = Tokens::default();
        let mut feed = Feed::settled(&t, 8400.0);
        let mut rpm = 6000.0;
        let one_frame = 6000.0 * DT / LIMITER;
        for frame in 0..24 {
            let shown = feed.step(&t, rpm);
            let target = rpm / LIMITER;
            if frame >= 1 {
                assert!(
                    shown <= target + 1e-6,
                    "frame {frame}: shown {shown} ahead of the rpm {target}"
                );
            }
            if frame >= 7 {
                assert!(target - shown < one_frame, "frame {frame}: lag {}", target - shown);
            }
            rpm += 6000.0 * DT;
        }
    }

    #[test]
    fn a_fast_blip_ramp_is_tracked_without_lag() {
        let t = Tokens::default();
        let mut feed = Feed::settled(&t, 2000.0);
        let mut rpm = 2000.0;
        let rate = 20_000.0;
        for frame in 0..18 {
            rpm += rate * DT;
            let shown = feed.step(&t, rpm);
            let lag = rpm / LIMITER - shown;
            assert!(lag >= -1e-6, "frame {frame}: never ahead of the rpm ({lag})");
            if frame >= 3 {
                assert!(lag < 1e-3, "frame {frame}: lag {lag}");
            }
        }
    }

    #[test]
    fn scale_thousands_are_bounded() {
        let t = Tokens::default();
        let mut f = FillModel::new(&t);
        assert_eq!(f.thousands(), 0, "no scale before telemetry");
        f.update(&sig(&t, 1000.0), DT);
        assert_eq!(f.thousands(), 8);
        let mut s = sig(&t, 1000.0);
        s.max_rpm = 1.0e9;
        f.update(&s, DT);
        assert_eq!(f.thousands(), MAX_THOUSANDS);
    }
}
