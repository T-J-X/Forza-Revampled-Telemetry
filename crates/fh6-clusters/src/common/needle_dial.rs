//! Classic `needleDial` tach (COMPONENTS.md §3, `docs/design/families/classic-analog.md`).
//!
//! A horology-grade rev counter for a transparent OLED: a true-black cavity with a faint domed
//! sheen toward the virtual light, a chapter ring with ticks every 250 / 500 / 1000 rpm, `dial`
//! numerals in thousands, an optional operating band and the redline sector. The lance needle
//! (14 -> 3 px) has a lit ridge whose two flanks swap brightness as it crosses the light, a white
//! lume core on the emissive layer, a screen-space drop shadow cast away from the light and a
//! polished hub whose ring turns `warning` from rev stage 3. Motion runs on the tokens' `needle`
//! spring with feed-forward; LFA-style ghost needles follow fast sweeps when enabled.
//!
//! The lower dead zone of the default 270° face is left empty for the logo zone
//! ([`NeedleDial::logo_zone`]), where clusters call `fh6_kit::logo::draw`.
use super::bezel::lit_rim;
use fh6_kit::math::{finite_or, smoothstep};
use fh6_kit::{
    deg, vec2, Align, Cap, Color, FontRole, Gradient, LayerOptions, Painter, Palette, Rect, Shadow, Shape, Signals,
    Spring, Stroke, TextStyle, TickRing, Tokens, VAlign, Vec2,
};
use std::f32::consts::{PI, TAU};

/// Face radius the token sizes are specified for (family layout: bezel 242–262, so face R 242).
/// Every length scales with `radius / REFERENCE_RADIUS`, so satellites reuse the dial.
pub const REFERENCE_RADIUS: f32 = 242.0;
/// Redline sector start as a fraction of the limiter (tokens `components.tach.redlineFromRpmPct`).
pub const REDLINE_FROM_PCT: f32 = 0.92;
/// Minor tick spacing (rpm); the red sector and numerals snap to it.
const MINOR_STEP_RPM: f32 = 250.0;
/// Upper bound of the scale (rpm), so a corrupt limiter can never build thousands of ticks.
const MAX_FULL_SCALE_RPM: f32 = 20_000.0;
/// Numerals `0` to `20` (thousands), so drawing never formats a string.
const DIGITS: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19",
    "20",
];
/// Above this many thousands the numerals step by 2000 rpm (tight faces stay legible).
const MAX_NUMERALS_AT_1000: u32 = 12;

/// Tick ring outer end, inset from the face edge (tokens `stroke`).
const TICK_INSET: f32 = 8.0;
/// Minor / mid / major ticks: length and width (tokens `stroke.tickLength`, `stroke.tick`).
const TICK_MINOR: (f32, f32) = (14.0, 2.0);
/// See [`TICK_MINOR`].
const TICK_MID: (f32, f32) = (22.0, 3.0);
/// See [`TICK_MINOR`].
const TICK_MAJOR: (f32, f32) = (34.0, 5.0);
/// Operating band and red sector width (tokens `layout.classicDial.operatingBand`: 9 px).
const BAND_WIDTH: f32 = 9.0;
/// Operating band opacity (family: band at 35%).
const BAND_ALPHA: f32 = 0.35;
/// Gap between the major ticks and the numerals.
const NUMERAL_GAP: f32 = 10.0;

/// Needle length as a fraction of the radius (tokens `stroke.needle.lengthPctOfRadius`).
const NEEDLE_LENGTH_PCT: f32 = 0.92;
/// Needle base and tip widths, and the tail beyond the hub (tokens `stroke.needle`).
const NEEDLE_BASE: f32 = 14.0;
/// See [`NEEDLE_BASE`].
const NEEDLE_TIP: f32 = 3.0;
/// See [`NEEDLE_BASE`].
const NEEDLE_TAIL: f32 = 10.0;
/// Hub cap radius as a fraction of the face radius.
const HUB_PCT: f32 = 0.10;
/// Lume core: where it starts and ends along the needle, and its share of the needle width.
const LUME_FROM: f32 = 0.30;
/// See [`LUME_FROM`].
const LUME_TO: f32 = 0.93;
/// See [`LUME_FROM`].
const LUME_WIDTH: f32 = 0.30;
/// Black keyline around the lume core (family: 2 px keyline).
const LUME_KEYLINE: f32 = 1.5;
/// Drop shadow: offset away from the light, sigma and opacity (family: 3 px, σ 2, 40%).
const SHADOW_OFFSET: f32 = 3.0;
/// See [`SHADOW_OFFSET`].
const SHADOW_SIGMA: f32 = 2.0;
/// See [`SHADOW_OFFSET`].
const SHADOW_ALPHA: f32 = 0.40;
/// Virtual light elevation (tokens `light.elevationDeg`).
const LIGHT_ELEVATION_DEG: f32 = 40.0;
/// Tilt of each needle flank away from the face plane (a lance ridge).
const FLANK_TILT_DEG: f32 = 38.0;
/// Flank luminance (fraction of `ink` over black) in full shadow and full light.
const FLANK_DARK: f32 = 0.16;
/// See [`FLANK_DARK`].
const FLANK_LIT: f32 = 0.82;

/// Afterimage (tokens `motion.afterimage`): raw needle below 90°/s, all ghosts from 180°/s.
const AFTERIMAGE_FROM_DEG_S: f32 = 90.0;
/// See [`AFTERIMAGE_FROM_DEG_S`].
const AFTERIMAGE_FULL_DEG_S: f32 = 180.0;
/// Number of ghost needles (tokens `motion.afterimage.ghosts`, a `[f32; 4]` ring buffer).
pub const GHOSTS: usize = 4;
/// Ghost sampling interval: one 60 Hz frame, independent of the display rate.
const GHOST_STEP: f32 = 1.0 / 60.0;
/// Opacity of the newest ghost at full strength.
const GHOST_ALPHA: f32 = 0.30;

/// The steady limiter wash ramps in from this fraction of the limiter (family: LFA precedent).
const WASH_FROM: f32 = 0.92;
/// Peak wash opacity at the face edge.
const WASH_ALPHA: f32 = 0.22;
/// Domed sapphire crescent opacity (family: crescent 18/255).
const CRESCENT_ALPHA: f32 = 18.0 / 255.0;
/// Peak opacity of the face's domed sheen toward the light.
const DOME_ALPHA: f32 = 0.045;

/// How the scale is laid out on the face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialMode {
    /// Symmetric linear 270° face: 0 at 7:30, full scale at 4:30 (tokens `defaultSweep`).
    #[default]
    Symmetric,
    /// Porsche track rev counter: the scale is rotated so the shift rpm stands at 12 o'clock
    /// (tokens `layout.classicDial.trackMode`).
    Track,
}

/// The ghost-needle trail of fast sweeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Afterimage {
    /// A single needle.
    #[default]
    Off,
    /// Lexus LFA style: up to four ghost needles above 180°/s.
    Lfa,
}

/// The colour of the needle's lume core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lume {
    /// White lume core (COMPONENTS.md §3).
    #[default]
    White,
    /// Lume insert in the palette's `primary` (text-safe), as in the family hero.
    Primary,
}

/// The family's one steady limiter cue on the face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LimiterCue {
    /// A steady red wash that ramps in from 92% of the limiter; it never flashes.
    #[default]
    RedWash,
    /// No cue on the face.
    None,
}

/// Who draws the hub over the needle pivot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HubStyle {
    /// The dial draws a polished hub cap with the stage-3 ring.
    #[default]
    Cap,
    /// The cluster covers the pivot itself (for example a sapphire gear lens); read
    /// [`NeedleDial::hub_heat`] to colour its ring.
    External,
}

/// An operating band (peak torque to peak power) as fractions of the limiter rpm, from the car
/// profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OperatingBand {
    /// Start, as a fraction of the limiter.
    pub from: f32,
    /// End, as a fraction of the limiter.
    pub to: f32,
}

impl OperatingBand {
    /// A band between two limiter fractions (ordered and clamped to `0..=1.1`).
    #[must_use]
    pub fn new(from: f32, to: f32) -> Self {
        let a = finite_or(from, 0.0).clamp(0.0, 1.1);
        let b = finite_or(to, 0.0).clamp(0.0, 1.1);
        Self {
            from: a.min(b),
            to: a.max(b),
        }
    }
}

/// Options of a [`NeedleDial`]; the default is the family's symmetric face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialConfig {
    /// Symmetric or track layout.
    pub mode: DialMode,
    /// The operating band, when the car profile knows it.
    pub band: Option<OperatingBand>,
    /// Ghost needles on fast sweeps.
    pub afterimage: Afterimage,
    /// Lume core colour.
    pub lume: Lume,
    /// Steady limiter cue on the face.
    pub limiter_cue: LimiterCue,
    /// Hub drawn by the dial or by the cluster.
    pub hub: HubStyle,
    /// Red sector start as a fraction of the limiter (snapped down to a 250 rpm tick).
    pub redline_pct: f32,
}

impl Default for DialConfig {
    fn default() -> Self {
        Self {
            mode: DialMode::Symmetric,
            band: None,
            afterimage: Afterimage::Off,
            lume: Lume::White,
            limiter_cue: LimiterCue::RedWash,
            hub: HubStyle::Cap,
            redline_pct: REDLINE_FROM_PCT,
        }
    }
}

/// The rpm-to-angle mapping of a dial face.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DialScale {
    /// Angle of 0 rpm (radians, clockwise from 3 o'clock).
    pub start: f32,
    /// Angle from 0 rpm to full scale (radians).
    pub sweep: f32,
    /// Full-scale rpm: the limiter rounded up to the next 1000.
    pub full_rpm: f32,
}

impl DialScale {
    /// The scale for `mode` with the car's limiter and shift rpm.
    #[must_use]
    pub fn new(mode: DialMode, limiter_rpm: f32, shift_rpm: f32) -> Self {
        let full_rpm = full_scale(limiter_rpm);
        let sweep = deg(270.0);
        let start = match mode {
            DialMode::Symmetric => deg(135.0),
            // tokens: angle = 270 + (rpm - shiftRpm) * 270 / maxRpm (screen degrees).
            DialMode::Track => deg(270.0 - finite_or(shift_rpm, 0.0).clamp(0.0, full_rpm) / full_rpm * 270.0),
        };
        Self {
            start,
            sweep,
            full_rpm,
        }
    }

    /// The angle of `rpm` (radians), clamped a little past full scale for spring overshoot.
    #[must_use]
    pub fn angle(&self, rpm: f32) -> f32 {
        self.start + self.sweep * (finite_or(rpm, 0.0) / self.full_rpm).clamp(0.0, 1.02)
    }

    /// Degrees of needle travel per rpm.
    #[must_use]
    pub fn degrees_per_rpm(&self) -> f32 {
        self.sweep.to_degrees() / self.full_rpm
    }

    /// The angle in the middle of the dead zone the needle never crosses.
    #[must_use]
    pub fn dead_zone_angle(&self) -> f32 {
        self.start + self.sweep + (TAU - self.sweep) * 0.5
    }

    /// The logo zone of a face of `radius` at `center`: a wordmark rectangle (≤ 220 x 44, tokens
    /// `layout.logoZone`) in the lower dead zone of the symmetric face, or a crest square
    /// (≤ 96) in the dead zone of a rotated track face. The needle never crosses it.
    #[must_use]
    pub fn logo_zone(&self, center: Vec2, radius: f32) -> Rect {
        let radius = finite_or(radius, 0.0).max(0.0);
        let gap = self.dead_zone_angle();
        if Vec2::from_angle(gap).y > 0.97 {
            let size = vec2((radius * 0.84).min(220.0), (radius * 0.19).min(44.0));
            Rect::from_center_size(center + vec2(0.0, radius * 0.60), size)
        } else {
            let side = (radius * 0.36).min(96.0);
            Rect::from_center_size(Vec2::polar(center, radius * 0.58, gap), Vec2::splat(side))
        }
    }
}

/// Full-scale rpm: the limiter rounded up to the next 1000, within 1000..=20000.
fn full_scale(limiter_rpm: f32) -> f32 {
    let thousands = (finite_or(limiter_rpm, 0.0).max(0.0) / 1000.0).ceil();
    (thousands * 1000.0).clamp(1000.0, MAX_FULL_SCALE_RPM)
}

/// Sizes of a face of a given radius (local px).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geometry {
    k: f32,
    tick_outer: f32,
    numeral_bound: f32,
    needle_length: f32,
    needle_tail: f32,
    needle_base: f32,
    needle_tip: f32,
    hub: f32,
}

impl Geometry {
    fn new(radius: f32) -> Self {
        let radius = finite_or(radius, REFERENCE_RADIUS).max(1.0);
        let k = (radius / REFERENCE_RADIUS).clamp(0.3, 3.0);
        let tick_outer = radius - TICK_INSET * k;
        let hub = radius * HUB_PCT;
        Self {
            k,
            tick_outer,
            numeral_bound: tick_outer - (TICK_MAJOR.0 + NUMERAL_GAP) * k,
            needle_length: radius * NEEDLE_LENGTH_PCT,
            needle_tail: hub + NEEDLE_TAIL * k,
            needle_base: NEEDLE_BASE * k,
            needle_tip: NEEDLE_TIP * k,
            hub,
        }
    }

    /// Full needle width at `s` px from the pivot (the needle tapers from the tail end).
    fn needle_width(&self, s: f32) -> f32 {
        let span = (self.needle_tail + self.needle_length).max(1.0);
        let t = ((s + self.needle_tail) / span).clamp(0.0, 1.0);
        self.needle_base + (self.needle_tip - self.needle_base) * t
    }
}

/// Brightness (0..=1) of the two flanks of a ridged needle at `angle` under the light at
/// azimuth `light`: `(right flank, left flank)` seen along the needle. They swap as the
/// needle crosses the light vector (the family's dauphine facet swap).
fn flank_levels(angle: f32, light: f32) -> (f32, f32) {
    let n = Vec2::from_angle(angle).perp();
    let l = Vec2::from_angle(light);
    let (sin_e, cos_e) = deg(LIGHT_ELEVATION_DEG).sin_cos();
    let (sin_t, cos_t) = deg(FLANK_TILT_DEG).sin_cos();
    let up = cos_t * sin_e;
    let side = n.dot(l) * cos_e * sin_t;
    ((up + side).clamp(0.0, 1.0), (up - side).clamp(0.0, 1.0))
}

/// An opaque neutral at `level` of `ink` over `oled_black` (metal facets, hub steel).
fn metal(palette: &Palette, level: f32) -> Color {
    palette.roles.oled_black.lerp(palette.roles.ink, level.clamp(0.0, 1.0))
}

/// The classic needle dial: animation state plus the face and needle drawing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NeedleDial {
    config: DialConfig,
    needle: Spring,
    heat: Spring,
    heat_target: f32,
    target_rpm: f32,
    limiter_rpm: f32,
    shift_rpm: f32,
    stage3: f32,
    numeral_cap: f32,
    ghosts: [f32; GHOSTS],
    ghost_head: usize,
    ghost_clock: f32,
    ghost_strength: f32,
    primed: bool,
}

impl NeedleDial {
    /// A dial on the tokens' `needle` spring (and `snap` for the hub ring), numerals at
    /// `type.scale.dialNumeral`.
    #[must_use]
    pub fn new(tokens: &Tokens, config: DialConfig) -> Self {
        Self {
            config,
            needle: Spring::new(tokens.springs.needle, 0.0),
            heat: Spring::new(tokens.springs.snap, 0.0),
            heat_target: 0.0,
            target_rpm: 0.0,
            limiter_rpm: 0.0,
            shift_rpm: 0.0,
            stage3: tokens.rev_stages.stage3,
            numeral_cap: tokens.type_scale.dial_numeral,
            ghosts: [0.0; GHOSTS],
            ghost_head: 0,
            ghost_clock: 0.0,
            ghost_strength: 0.0,
            primed: false,
        }
    }

    /// The options.
    #[must_use]
    pub fn config(&self) -> &DialConfig {
        &self.config
    }

    /// Replaces the options (for example a new operating band when the car changes).
    pub fn set_config(&mut self, config: DialConfig) {
        self.config = config;
    }

    /// Advances with this frame's signals: rpm with feed-forward from `rpm_rate`, the limiter
    /// and shift rpm for the scale, and `shift_fraction` for the stage-3 hub ring.
    pub fn update(&mut self, sig: &Signals, dt: f32) {
        let dt = finite_or(dt, 0.0).max(0.0);
        self.limiter_rpm = finite_or(sig.max_rpm, 0.0).max(0.0);
        self.shift_rpm = finite_or(sig.shift_rpm, 0.0).max(0.0);
        self.target_rpm = finite_or(sig.rpm, 0.0).max(0.0);
        if !self.primed {
            self.needle.reset(self.target_rpm);
            self.ghosts = [self.target_rpm; GHOSTS];
            self.primed = true;
        }

        // The ring buffer holds the needle as it was one 60 Hz frame ago, two, three, four.
        self.ghost_clock += dt;
        if self.ghost_clock + 1e-4 >= GHOST_STEP {
            self.ghost_clock = (self.ghost_clock - GHOST_STEP).clamp(0.0, GHOST_STEP);
            self.ghost_head = (self.ghost_head + GHOSTS - 1) % GHOSTS;
            self.ghosts[self.ghost_head] = self.needle.value();
        }

        self.needle
            .update_tracking(self.target_rpm, finite_or(sig.rpm_rate, 0.0), dt);

        self.ghost_strength = match self.config.afterimage {
            Afterimage::Off => 0.0,
            Afterimage::Lfa => {
                let deg_per_s = self.needle.velocity().abs() * self.scale().degrees_per_rpm();
                smoothstep(AFTERIMAGE_FROM_DEG_S, AFTERIMAGE_FULL_DEG_S, deg_per_s)
            }
        };

        self.heat_target = if finite_or(sig.shift_fraction, 0.0) >= self.stage3 {
            1.0
        } else {
            0.0
        };
        self.heat.update(self.heat_target, dt);
    }

    /// True while the needle, its ghosts or the hub ring still move without new telemetry.
    #[must_use]
    pub fn animating(&self) -> bool {
        !self.needle.settled(self.target_rpm, 0.5)
            || self.ghost_strength > 0.0
            || !self.heat.settled(self.heat_target, 1e-3)
    }

    /// The face's current scale.
    #[must_use]
    pub fn scale(&self) -> DialScale {
        DialScale::new(self.config.mode, self.limiter_rpm, self.shift_rpm)
    }

    /// The rpm the needle shows (spring value).
    #[must_use]
    pub fn needle_rpm(&self) -> f32 {
        self.needle.value()
    }

    /// The needle angle (radians, clockwise from 3 o'clock).
    #[must_use]
    pub fn needle_angle(&self) -> f32 {
        self.scale().angle(self.needle.value())
    }

    /// The stage-3 hub ring mix, 0 (neutral) to 1 (`warning`), on the `snap` spring.
    #[must_use]
    pub fn hub_heat(&self) -> f32 {
        self.heat.value().clamp(0.0, 1.0)
    }

    /// The steady limiter wash, 0 to 1 from 92% of the limiter (0 with [`LimiterCue::None`]).
    #[must_use]
    pub fn wash(&self) -> f32 {
        match self.config.limiter_cue {
            LimiterCue::None => 0.0,
            LimiterCue::RedWash => {
                let frac = self.needle.value() / self.limiter_rpm.max(1.0);
                smoothstep(WASH_FROM, 1.0, frac)
            }
        }
    }

    /// The red sector start: `redline_pct` of the limiter, snapped down to a 250 rpm tick.
    #[must_use]
    pub fn redline_rpm(&self) -> f32 {
        let rpm = self.limiter_rpm * finite_or(self.config.redline_pct, REDLINE_FROM_PCT).clamp(0.5, 1.0);
        (rpm / MINOR_STEP_RPM).floor() * MINOR_STEP_RPM
    }

    /// The logo zone for a face of `radius` at `center` (see [`DialScale::logo_zone`]).
    #[must_use]
    pub fn logo_zone(&self, center: Vec2, radius: f32) -> Rect {
        self.scale().logo_zone(center, radius)
    }

    /// The `i`-th newest ghost (rpm), `i < GHOSTS`.
    fn ghost(&self, i: usize) -> f32 {
        self.ghosts[(self.ghost_head + i) % GHOSTS]
    }

    /// Draws the whole dial (face, then needle and hub) at `center` with face `radius`, lit
    /// from `light` (radians, screen convention, e.g. `VirtualLight::angle()`).
    pub fn draw(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, palette: &Palette, light: f32) {
        self.draw_face(p, center, radius, palette, light);
        self.draw_needle(p, center, radius, palette, light);
    }

    /// Draws the face: true-black cavity with a domed sheen, the steady limiter wash, the
    /// sapphire crescent, operating band, red sector, chapter ring, ticks and numerals.
    pub fn draw_face(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, palette: &Palette, light: f32) {
        let g = Geometry::new(radius);
        let radius = finite_or(radius, REFERENCE_RADIUS).max(1.0);
        let k = g.k;
        let roles = palette.roles;
        let scale = self.scale();
        let l = Vec2::from_angle(light);
        let face = Shape::circle(center, radius);

        // Cavity: true black, a faint dome of light toward the light source.
        p.fill(&face, roles.oled_black);
        p.fill(
            &face,
            Gradient::radial(center + l * (radius * 0.35), radius * 1.25)
                .stop(0.0, roles.ink.with_alpha(DOME_ALPHA))
                .stop(0.55, roles.ink.with_alpha(DOME_ALPHA * 0.3))
                .stop(1.0, roles.ink.with_alpha(0.0)),
        );

        // The family's steady limiter cue: a red wash from the rim inwards, never flashing.
        let wash = self.wash();
        if wash > 0.0 {
            p.fill(
                &face,
                Gradient::radial(center, radius)
                    .stop(0.0, roles.warning.with_alpha(WASH_ALPHA * wash * 0.25))
                    .stop(0.55, roles.warning.with_alpha(WASH_ALPHA * wash * 0.45))
                    .stop(1.0, roles.warning.with_alpha(WASH_ALPHA * wash)),
            );
        }

        // Domed sapphire: a thin crescent of reflected light on the light side.
        let crescent = Gradient::conic(center, light)
            .stop(0.0, roles.ink.with_alpha(CRESCENT_ALPHA))
            .stop(0.06, roles.ink.with_alpha(CRESCENT_ALPHA * 0.5))
            .stop(0.14, roles.ink.with_alpha(0.0))
            .stop(0.86, roles.ink.with_alpha(0.0))
            .stop(0.94, roles.ink.with_alpha(CRESCENT_ALPHA * 0.5))
            .stop(1.0, roles.ink.with_alpha(CRESCENT_ALPHA));
        p.fill(&Shape::ring(center, radius - 3.0 * k, 4.0 * k), crescent);

        // Operating band and red sector share the chapter-ring track under the ticks.
        let track = g.tick_outer - BAND_WIDTH * 0.5 * k;
        if let Some(band) = self.config.band {
            let a0 = scale.angle(band.from * self.limiter_rpm);
            let a1 = scale.angle(band.to * self.limiter_rpm);
            if a1 > a0 {
                p.fill(
                    &Shape::arc(center, track, BAND_WIDTH * k, a0, a1 - a0, Cap::Butt),
                    palette.primary.with_alpha(BAND_ALPHA),
                );
            }
        }
        let red_rpm = self.redline_rpm();
        let red_a = scale.angle(red_rpm);
        let end_a = scale.angle(scale.full_rpm);
        if end_a > red_a {
            p.fill(
                &Shape::arc(center, track, BAND_WIDTH * k, red_a, end_a - red_a, Cap::Butt),
                roles.warning.with_alpha(0.85),
            );
        }

        // Chapter ring: a 1 px hairline along the tick ends.
        p.fill(
            &Shape::arc(center, g.tick_outer + 2.0 * k, p.device_px(), scale.start, scale.sweep, Cap::Butt),
            roles.hairline,
        );

        // Ticks every 250 / 500 / 1000 rpm; ticks inside the red sector turn warning.
        let red_t = ((scale.angle(red_rpm - MINOR_STEP_RPM * 0.5) - scale.start) / TAU).clamp(0.0, 1.0);
        let tick_paint = |base: Color| {
            Gradient::conic(center, scale.start)
                .stop(0.0, base)
                .stop(red_t, base)
                .stop(red_t, roles.warning)
                .stop(1.0, roles.warning)
        };
        let full = scale.full_rpm;
        for (step, (len, width), base) in [
            (MINOR_STEP_RPM, TICK_MINOR, roles.ink_muted),
            (500.0, TICK_MID, roles.ink_muted),
            (1000.0, TICK_MAJOR, roles.ink),
        ] {
            let count = (full / step).round() as u32 + 1;
            p.fill(
                &Shape::Ticks(
                    TickRing::new(center, g.tick_outer, scale.start, scale.sweep, count).minor(len * k, width * k),
                ),
                tick_paint(base),
            );
        }

        // Numerals in thousands, each pulled in by its own radial half-extent so the gap to the
        // ticks is even all round.
        let cap = self.numeral_cap * k;
        let base_style = TextStyle::new(p.fonts().role(FontRole::Dial), cap).align(Align::Center, VAlign::CapMiddle);
        let thousands = (full / 1000.0).round() as u32;
        let step = if thousands > MAX_NUMERALS_AT_1000 { 2 } else { 1 };
        for n in (0..=thousands).step_by(step) {
            let Some(digits) = DIGITS.get(n as usize) else {
                break;
            };
            let rpm = n as f32 * 1000.0;
            let a = scale.angle(rpm);
            let dir = Vec2::from_angle(a);
            let half_w = p.measure(digits, &base_style).width * 0.5;
            let half_h = cap * 0.5;
            let r = g.numeral_bound - (dir.x.abs() * half_w + dir.y.abs() * half_h);
            let ink = if rpm >= red_rpm { roles.warning } else { roles.ink };
            p.text(Vec2::polar(center, r, a), digits, &base_style.paint(ink));
        }
    }

    /// Draws the needle (ghosts, shadow, lit ridge, lume core) and, with [`HubStyle::Cap`],
    /// the polished hub with its stage-3 ring.
    pub fn draw_needle(&self, p: &mut Painter<'_>, center: Vec2, radius: f32, palette: &Palette, light: f32) {
        let g = Geometry::new(radius);
        let k = g.k;
        let roles = palette.roles;
        let scale = self.scale();
        let l = Vec2::from_angle(light);
        let lume = match self.config.lume {
            Lume::White => roles.ink,
            Lume::Primary => palette.primary_text,
        };
        let needle_at = |a: f32| Shape::needle(center, a, g.needle_length, g.needle_tail, g.needle_base, g.needle_tip);

        // LFA afterimage: the last positions at falling opacity.
        if self.ghost_strength > 0.0 {
            let visible = (self.ghost_strength * GHOSTS as f32).ceil() as usize;
            for i in 0..visible.min(GHOSTS) {
                let fade = 1.0 - i as f32 / GHOSTS as f32;
                let alpha = GHOST_ALPHA * self.ghost_strength * fade;
                p.fill(&needle_at(scale.angle(self.ghost(i))), lume.with_alpha(alpha));
            }
        }

        let a = scale.angle(self.needle.value());
        let dir = Vec2::from_angle(a);
        let needle = needle_at(a);

        // Screen-space drop shadow, cast away from the light.
        p.shadow(
            &needle,
            Shadow::new(
                -l * (SHADOW_OFFSET * k),
                SHADOW_SIGMA * k,
                roles.oled_black.with_alpha(SHADOW_ALPHA),
            ),
        );

        // The lit ridge: two flanks split along the axis, each shaded by its facing.
        let (right, left) = flank_levels(a, light);
        let flank = |level: f32| metal(palette, FLANK_DARK + (FLANK_LIT - FLANK_DARK) * level);
        let n = dir.perp();
        let half = g.needle_base * 0.5;
        p.fill(
            &needle,
            Gradient::linear(center + n * half, center - n * half)
                .stop(0.0, flank(right))
                .stop(0.5, flank(right))
                .stop(0.5, flank(left))
                .stop(1.0, flank(left)),
        );

        // Lume core in a black keyline; the lume is the emissive part (glance rank 3).
        let s0 = g.needle_length * LUME_FROM;
        let s1 = g.needle_length * LUME_TO;
        let w0 = g.needle_width(s0) * LUME_WIDTH;
        let w1 = g.needle_width(s1) * LUME_WIDTH;
        let keyline = LUME_KEYLINE * k;
        p.fill(
            &Shape::Line {
                a: center + dir * (s0 - keyline),
                b: center + dir * (s1 + keyline),
                width_a: w0 + keyline * 2.0,
                width_b: w1 + keyline * 2.0,
                cap: Cap::Round,
            },
            roles.oled_black,
        );
        let core = Shape::Line {
            a: center + dir * s0,
            b: center + dir * s1,
            width_a: w0,
            width_b: w1,
            cap: Cap::Round,
        };
        p.layer(LayerOptions::bloom(), |p| p.fill(&core, lume));

        if self.config.hub == HubStyle::Cap {
            self.draw_hub(p, center, &g, palette, light);
        }
    }

    /// The polished hub cap, its 1 px lit rim, the stage-3 ring and the pivot jewel.
    fn draw_hub(&self, p: &mut Painter<'_>, center: Vec2, g: &Geometry, palette: &Palette, light: f32) {
        let k = g.k;
        let roles = palette.roles;
        let l = Vec2::from_angle(light);
        let hub = Shape::circle(center, g.hub);
        p.shadow(
            &hub,
            Shadow::new(-l * (SHADOW_OFFSET * k), 3.0 * k, roles.oled_black.with_alpha(0.55)),
        );
        p.fill(
            &hub,
            Gradient::radial(center + l * (g.hub * 0.45), g.hub * 1.45)
                .stop(0.0, metal(palette, 0.62))
                .stop(0.32, metal(palette, 0.24))
                .stop(0.75, metal(palette, 0.08))
                .stop(1.0, metal(palette, 0.04)),
        );
        p.stroke(
            &hub,
            lit_rim(center, light, roles.rim_specular, roles.hairline.with_alpha(0.06)),
            Stroke::rim_px(1.0),
        );
        let ring = roles.hairline.lerp(roles.warning, self.hub_heat());
        p.fill(&Shape::ring(center, g.hub + 4.0 * k, 2.5 * k), ring);
        let jewel = Shape::circle(center, g.hub * 0.26);
        p.fill(&jewel, roles.oled_black);
        p.stroke(
            &jewel,
            lit_rim(center, light + PI, roles.rim_specular.with_alpha(0.5), roles.hairline.with_alpha(0.0)),
            Stroke::rim_px(1.0),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIMITER: f32 = 8700.0;
    const SHIFT: f32 = 8439.0;

    fn signals(rpm: f32, rpm_rate: f32) -> Signals {
        Signals {
            rpm,
            rpm_rate,
            max_rpm: LIMITER,
            shift_rpm: SHIFT,
            rpm_fraction: rpm / LIMITER,
            shift_fraction: rpm / SHIFT,
            ..Signals::default()
        }
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).rem_euclid(TAU).min((b - a).rem_euclid(TAU)) < 1e-3
    }

    #[test]
    fn full_scale_rounds_the_limiter_up() {
        assert_eq!(full_scale(8700.0), 9000.0);
        assert_eq!(full_scale(7000.0), 7000.0);
        assert_eq!(full_scale(7001.0), 8000.0);
        assert_eq!(full_scale(0.0), 1000.0);
        assert_eq!(full_scale(f32::NAN), 1000.0);
        assert_eq!(full_scale(90_000.0), MAX_FULL_SCALE_RPM);
    }

    #[test]
    fn symmetric_face_sweeps_270_from_half_past_seven() {
        let s = DialScale::new(DialMode::Symmetric, LIMITER, SHIFT);
        assert!(close(s.angle(0.0), deg(135.0)));
        assert!(close(s.angle(9000.0), deg(405.0)));
        assert!(close(s.angle(4500.0), deg(270.0)), "mid-scale stands at 12 o'clock");
        assert!(close(s.dead_zone_angle(), deg(90.0)), "dead zone at 6 o'clock");
    }

    #[test]
    fn track_mode_puts_the_shift_rpm_at_twelve() {
        let s = DialScale::new(DialMode::Track, LIMITER, SHIFT);
        assert!(close(s.angle(SHIFT), deg(270.0)));
        assert!((s.sweep - deg(270.0)).abs() < 1e-5);
        let z = s.logo_zone(Vec2::ZERO, REFERENCE_RADIUS);
        assert!(z.width() <= 96.0 && (z.width() - z.height()).abs() < 1e-3, "crest square");
    }

    #[test]
    fn redline_snaps_to_a_tick() {
        let t = Tokens::default();
        let mut d = NeedleDial::new(&t, DialConfig::default());
        d.update(&signals(900.0, 0.0), 1.0 / 60.0);
        assert_eq!(d.redline_rpm(), 8000.0, "0.92 x 8700 = 8004 snaps to 8000");
    }

    /// Points along the needle (hub to tip, both edges) for every 50 rpm never enter the logo
    /// zone, and the numeral boxes stay clear of it.
    #[test]
    fn logo_zone_is_never_crossed() {
        for radius in [150.0, REFERENCE_RADIUS, 320.0] {
            let g = Geometry::new(radius);
            let s = DialScale::new(DialMode::Symmetric, LIMITER, SHIFT);
            let zone = s.logo_zone(Vec2::ZERO, radius);
            assert!(zone.width() > 0.0 && zone.height() > 0.0);
            let mut rpm = 0.0;
            while rpm <= s.full_rpm * 1.02 {
                let a = s.angle(rpm);
                let dir = Vec2::from_angle(a);
                let n = dir.perp();
                let mut d = -g.needle_tail;
                while d <= g.needle_length {
                    let hw = g.needle_width(d) * 0.5 + 2.0;
                    for side in [-1.0, 0.0, 1.0] {
                        let pt = dir * d + n * (hw * side);
                        assert!(!zone.contains(pt), "needle at {rpm} rpm crosses the logo zone (R {radius})");
                    }
                    d += 2.0;
                }
                rpm += 50.0;
            }
            // Numeral boxes (two-digit worst case: 1.25 cap wide).
            let cap = Tokens::default().type_scale.dial_numeral * g.k;
            for n in 0..=9 {
                let a = s.angle(n as f32 * 1000.0);
                let dir = Vec2::from_angle(a);
                let (hw, hh) = (cap * 0.62, cap * 0.5);
                let r = g.numeral_bound - (dir.x.abs() * hw + dir.y.abs() * hh);
                let b = Rect::from_center_size(dir * r, vec2(hw * 2.0, hh * 2.0));
                assert!(b.intersect(&zone).is_empty(), "numeral {n} overlaps the logo zone (R {radius})");
            }
        }
    }

    #[test]
    fn needle_tracks_a_climb_with_feed_forward() {
        let t = Tokens::default();
        let mut d = NeedleDial::new(&t, DialConfig::default());
        let rate = 3000.0;
        let dt = 1.0 / 60.0;
        let mut rpm = 2000.0;
        for _ in 0..90 {
            rpm += rate * dt;
            d.update(&signals(rpm, rate), dt);
        }
        let lag = (rpm - d.needle_rpm()).abs();
        assert!(lag < rate * dt, "lag {lag} rpm is under one 60 Hz frame of travel");
    }

    #[test]
    fn afterimage_only_on_fast_sweeps() {
        let t = Tokens::default();
        let cfg = DialConfig {
            afterimage: Afterimage::Lfa,
            ..DialConfig::default()
        };
        let dt = 1.0 / 60.0;
        // A slow climb (1500 rpm/s = 45 deg/s on a 9000 face): no ghosts.
        let mut slow = NeedleDial::new(&t, cfg);
        let mut rpm = 2000.0;
        for _ in 0..60 {
            rpm += 1500.0 * dt;
            slow.update(&signals(rpm, 1500.0), dt);
        }
        assert_eq!(slow.ghost_strength, 0.0);
        // A blip (30000 rpm/s = 900 deg/s): all four ghosts, trailing behind the needle.
        let mut fast = NeedleDial::new(&t, cfg);
        let mut rpm = 1500.0;
        for _ in 0..10 {
            rpm += 30_000.0 * dt;
            fast.update(&signals(rpm, 30_000.0), dt);
        }
        assert!(fast.ghost_strength > 0.99, "strength {}", fast.ghost_strength);
        assert!(fast.animating());
        for i in 0..GHOSTS - 1 {
            assert!(fast.ghost(i) > fast.ghost(i + 1), "ghost {i} is newer than ghost {}", i + 1);
        }
        assert!(fast.ghost(0) < fast.needle_rpm());
        // Off: never.
        let mut off = NeedleDial::new(&t, DialConfig::default());
        off.update(&signals(1500.0, 0.0), dt);
        off.update(&signals(6000.0, 30_000.0), dt);
        assert_eq!(off.ghost_strength, 0.0);
    }

    #[test]
    fn hub_ring_heats_from_stage_three_only() {
        let t = Tokens::default();
        let mut d = NeedleDial::new(&t, DialConfig::default());
        let dt = 1.0 / 60.0;
        for _ in 0..30 {
            d.update(&signals(SHIFT * (t.rev_stages.stage3 - 0.02), 0.0), dt);
        }
        assert!(d.hub_heat() < 1e-3, "below stage 3: {}", d.hub_heat());
        for _ in 0..30 {
            d.update(&signals(SHIFT * (t.rev_stages.stage3 + 0.01), 0.0), dt);
        }
        assert!(d.hub_heat() > 0.99, "stage 3: {}", d.hub_heat());
    }

    #[test]
    fn red_wash_ramps_in_steadily_from_92_percent() {
        let t = Tokens::default();
        let dt = 1.0 / 60.0;
        let wash_at = |rpm: f32, cue: LimiterCue| {
            let mut d = NeedleDial::new(
                &t,
                DialConfig {
                    limiter_cue: cue,
                    ..DialConfig::default()
                },
            );
            for _ in 0..60 {
                d.update(&signals(rpm, 0.0), dt);
            }
            d.wash()
        };
        assert_eq!(wash_at(LIMITER * 0.90, LimiterCue::RedWash), 0.0);
        let mid = wash_at(LIMITER * 0.96, LimiterCue::RedWash);
        assert!(mid > 0.2 && mid < 0.8, "ramp, not a switch: {mid}");
        assert!(wash_at(LIMITER, LimiterCue::RedWash) > 0.99);
        assert_eq!(wash_at(LIMITER, LimiterCue::None), 0.0);
    }

    #[test]
    fn flanks_swap_as_the_needle_crosses_the_light() {
        let light = deg(-135.0);
        // Needle perpendicular to the light, either way: one flank lit, the other dark.
        let (r1, l1) = flank_levels(light + deg(90.0), light);
        let (r2, l2) = flank_levels(light - deg(90.0), light);
        assert!(r1 < 0.1 && l1 > 0.9, "({r1}, {l1})");
        assert!(r2 > 0.9 && l2 < 0.1, "({r2}, {l2})");
        // Pointing at the light: both flanks equal (the swap moment).
        let (r3, l3) = flank_levels(light, light);
        assert!((r3 - l3).abs() < 1e-4);
    }

    #[test]
    fn needle_settles_and_stops_animating() {
        let t = Tokens::default();
        let mut d = NeedleDial::new(&t, DialConfig::default());
        d.update(&signals(900.0, 0.0), 1.0 / 60.0);
        for _ in 0..120 {
            d.update(&signals(5200.0, 0.0), 1.0 / 60.0);
        }
        assert!((d.needle_rpm() - 5200.0).abs() < 0.5);
        assert!(!d.animating());
    }

    #[test]
    fn operating_band_is_ordered() {
        let b = OperatingBand::new(0.85, 0.55);
        assert_eq!((b.from, b.to), (0.55, 0.85));
        let c = OperatingBand::new(f32::NAN, 2.0);
        assert_eq!((c.from, c.to), (0.0, 1.1));
    }
}
