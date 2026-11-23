//! Tach variants `arc`, `bar` and `chevron` (COMPONENTS.md §3), each with the steady limiter cue
//! of §5 (one non-flashing cue per family; the 10 Hz strobe stays on the shift strip).
//!
//! The three variants share one fill model: the rpm fraction from [`Signals`] on the tokens'
//! `ribbon` spring with velocity feed-forward, and a limiter amount on the `snap` spring that
//! switches on at the shift rpm and off 120 rpm below it, so the cue never flickers while the
//! engine bounces off the limiter. `update` and `draw` allocate nothing.
use fh6_kit::{
    deg, vec2, Align, Axis, Cap, Color, FontId, Glow, Gradient, LayerOptions, Paint, Painter, Palette, Rect,
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
/// Width (rpm fraction) of the soft blend from `primary` into the caution stage on the arc.
const ARC_STAGE_BLEND: f32 = 0.05;
/// The hot head of the arc fill whitens to this share of `ink` at the very tip.
const ARC_HEAD_WHITE: f32 = 0.6;
/// Share of white mixed into the core of a lit VFD chevron (phosphor core).
const CHEVRON_CORE_WHITE: f32 = 0.38;
/// Gap between chevron segments as a fraction of the segment pitch.
const CHEVRON_GAP: f32 = 0.24;
/// Horizontal lean of a slanted chevron per unit of height (tan 14 degrees).
const CHEVRON_LEAN: f32 = 0.249;
/// Depth of an arrow chevron's point per unit of height.
const CHEVRON_ARROW_DEPTH: f32 = 0.22;
/// Height of the first chevron as a fraction of the last (the staircase rise).
const CHEVRON_RISE: f32 = 0.3;
/// Lens glint band on top of bar cells, as a fraction of the cell height.
const BAR_GLINT_HEIGHT: f32 = 0.3;
/// Glint strength (share of `ink`) on lit bar cells.
const BAR_LIT_GLINT: f32 = 0.2;
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
}

/// The fill state every variant shares: the rpm fraction on the `ribbon` spring and the limiter
/// amount on the `snap` spring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FillModel {
    level: Spring,
    limiter: Spring,
    target: f32,
    limiter_on: bool,
    max_rpm: f32,
    shift_ratio: f32,
}

impl FillModel {
    fn new(tokens: &Tokens) -> Self {
        Self {
            level: Spring::new(tokens.springs.ribbon, 0.0),
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
        self.level.value().clamp(0.0, 1.0)
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
            roles.warning.fade(0.7),
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
        let blade = Shape::line(tip - radial * (half * 0.92), tip + radial * (half * 0.92), (thickness * 0.16).max(2.0));
        let glow = p.tokens().glow.medium;
        p.layer(LayerOptions::bloom(), |p| {
            fill_with_cue(p, &shape, ribbon.into(), cue, k);
            for lobe in glow.lobes() {
                p.glow(&head, head_color, Glow::new(lobe.sigma, lobe.intensity));
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
        major(0, first_red).draw(p, roles.hairline.fade(1.0));
        major(first_red, majors).draw(p, roles.warning.fade(0.7));
        minor(0, first_red_minor).draw(p, roles.hairline.fade(0.6));
        minor(first_red_minor, minors).draw(p, roles.warning.fade(0.45));
    }

    /// Draws the scale numerals (thousands) at `radius` around `center`, `cap` px tall, in the
    /// label ink (text-safe warning inside the redline sector). Optional: clusters may set their
    /// own numerals.
    pub fn draw_numerals(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, font: FontId, cap: f32, palette: &Palette) {
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

/// Peak hold of the rev bar: holds the highest fraction for `hold` seconds, then falls on a
/// spring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PeakHold {
    level: Spring,
    hold: f32,
    left: f32,
}

impl PeakHold {
    fn update(&mut self, fraction: f32, dt: f32) {
        let dt = finite(dt).max(0.0);
        if fraction >= self.level.value() {
            self.level.reset(fraction);
            self.left = self.hold;
        } else if self.left > 0.0 {
            self.left = (self.left - dt).max(0.0);
        } else {
            self.level.update(fraction, dt);
        }
    }

    fn animating(&self, fraction: f32) -> bool {
        self.left > 0.0 || !self.level.settled(fraction, 1e-4)
    }
}

/// The `bar` tach (Race Dash): a segmented linear bar (45 segments, 5 px gaps) with step
/// lighting. Lit cells are 72% of `primary` and take the caution, warning and shift stage
/// colours above the car's thresholds; at the limiter they turn steady `ink`. Unlit cells are
/// grey lenses. Optional peak hold.
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
        if let Some(peak) = &mut self.peak {
            peak.update(fraction, dt);
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

    /// True while the fill, the limiter cue or the peak hold still moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.fill.animating() || self.peak.is_some_and(|pk| pk.animating(self.fill.fraction()))
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
    /// centre, so a cell lights in the stage of the rpm at which it lights.
    fn paint(&self, rect: Rect, palette: &Palette) -> Gradient {
        let [caution, warning, shift] = self.fill.stage_fractions(palette);
        let (c, w, s) = (
            self.stop_offset(rect, caution),
            self.stop_offset(rect, warning),
            self.stop_offset(rect, shift),
        );
        let base = palette.primary.fade(BAR_BASE_STRENGTH);
        let caution_color = palette.rev_stage(palette.stages[1]);
        let warning_color = palette.rev_stage(palette.stages[2]);
        let shift_color = palette.rev_stage(1.0);
        let y = rect.center().y;
        Gradient::linear(vec2(rect.min.x, y), vec2(rect.max.x, y))
            .stop(0.0, base)
            .stop(c, base)
            .stop(c, caution_color)
            .stop(w, caution_color)
            .stop(w, warning_color)
            .stop(s, warning_color)
            .stop(s, shift_color)
            .stop(1.0, shift_color)
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
        let glint = Rect::from_xywh(rect.min.x, rect.min.y, rect.width(), rect.height() * BAR_GLINT_HEIGHT);
        let glint_radius = radius.min(glint.height() * 0.5);
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
            p.layer(LayerOptions::bloom(), |p| {
                fill_with_cue(p, &lit_bar, paint.into(), cue, k);
            });
            // A lens glint across the top of each lit cell.
            p.fill(&bar(glint, glint_radius, Color::TRANSPARENT), roles.ink.fade(BAR_LIT_GLINT));
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
        self.primary
            .saturating_add(self.secondary)
            .saturating_add(self.accent)
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
/// whiter core and a tight glow, unlit segments as ghosts at 7% of their colour. Steady at the
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

    /// Draws the run in `rect`. Lit segments bloom with the phosphor glow (glance rank 3).
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
        let phosphor = p.tokens().glow.phosphor;
        p.layer(LayerOptions::bloom(), |p| {
            for i in 0..lit {
                let (pts, len) = self.segment(i, rect);
                let shape = Shape::Polygon { points: &pts[..len] };
                let mut color = self.segment_color(i, palette);
                if let Some(c) = cue {
                    color = color.lerp(c, k);
                }
                for lobe in phosphor.lobes() {
                    p.glow(&shape, color, Glow::new(lobe.sigma, lobe.intensity));
                }
                // Phosphor core: whiter along the middle of the segment, full hue at its edges.
                let bottom = (pts[0] + pts[1]) * 0.5;
                let top = vec2(bottom.x, bottom.y - (pts[0].y - rect_top(&pts[..len])));
                p.fill(
                    &shape,
                    Gradient::linear(bottom, top)
                        .stop(0.0, color)
                        .stop(0.5, color.lerp(ink, CHEVRON_CORE_WHITE))
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
        assert!((f.fraction() - 5200.0 / LIMITER).abs() < 1e-3, "fraction {}", f.fraction());
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
            s = Signals { rpm_rate: rate, ..sig(&t, rpm) };
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
            let want = if lights_at >= shift_ratio {
                pal.rev_stage(1.0)
            } else if lights_at >= pal.stages[2] * shift_ratio {
                pal.roles.warning
            } else if lights_at >= pal.stages[1] * shift_ratio {
                pal.roles.caution
            } else {
                pal.primary.fade(BAR_BASE_STRENGTH)
            };
            assert_eq!(bar.segment_color(rect, i, &pal), want, "segment {i} lights at {lights_at}");
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
