//! AMG MBUX Superscreen (`ClusterTheme::AmgMbuxSuperscreen`), catalog row 4: the driver display
//! of the E-Class generation that carries the MBUX Superscreen (W214, 2023 on, and the
//! Mercedes-AMG E 53), in its Sport (press: "Sporty") style, made "MBUX but better". Supercar
//! category, Modern Digital 2027 family, cockpit. Research: `RESEARCH.md`. Board: Penpot board v4
//! (page "07 · AmgMbuxSuperscreen", boards "v4 · <state>").
//!
//! There is no housing (TJS, 2026-11-28: "can be outside of a slab ... what MBUX is but better"):
//! every part floats on OLED black over a soft charcoal stage with no edge. The real display's
//! signatures stay (the red crescents, the side scales, G-Force and Engine data, the selector
//! stack); the middle is TJS's sky tunnel (08:18: "subtle clouds and horizon tunnel background and
//! the tunnel from the back to front being the rev limiter"). Zone table (local 4K px round the
//! cockpit anchor; `geom.rs` holds it as data):
//!
//! | Zone | Where | Size | Holds |
//! |---|---|---|---|
//! | Stage | ellipse round (0, −36) | 760 × 300 | a soft charcoal glow that ties the parts together, no edge |
//! | Crescents | round (0, −36), r 300 | 84 px at 9 and 3 o'clock, pointed tips, ±48° | halftone red, at half light at low revs and full near the shift point |
//! | Sky | the near ring's circle above the horizon y −58 | r 202 | a dusk sky with soft clouds (SDXL plate) that drift with the distance driven, warming to sunset near the shift point; it leans with lateral G (5° a g) about the horizon's centre |
//! | Rings | from (0, −60) r 64 out to the tunnel centre r 202 | 12, spaced in perspective | rank 1: the rev counter, lighting from the far end towards the driver; the nearest three red, the shift colour on the limiter; faint above the horizon so the sky shows through |
//! | Road | trapezoid y −58 .. 200, half widths 44 .. 214 | – | the road out of the tunnel with lane dashes that run with the distance driven, a red wash under braking, the horizon line on top; the rpm readout, cap 40, at (0, 56) |
//! | Speed | (0, −190) | cap 58 | rank 2: the speed above the thin speed line (y −153, ±150), on a soft dark halo; its unit to the right of it |
//! | Side scales | bands r 312 .. 392 round the tunnel centre, 150° .. 216° (mirrored right) | 21 ticks inward from the rim, numerals at r 344 | rank 4: torque (left) and power (right), the live fill along the rim, each live value under its band |
//! | G-Force | (−500, −52) | r 76 | rank 4: the felt force as a dot on red rings and a crosshair, over the rendered lens |
//! | Engine data | (500, −52) | r 76 | rank 4: boost on a round gauge |
//! | Gear | (660, 170), the selector stack at x 584 | cap 84; neighbours cap 18 | rank 1: the engaged gear, with the gears either side and a red `M` in a column, like the real selector |
//! | Logo | x −686 .. −506, y 164 .. 208 | – | rank 5: the car's logo or the custom name |
//!
//! Brightness follows the rank: the lit rings bloom; the gear and the speed are the brightest
//! type, in full ink without a glow; everything that never changes (labels, scale ticks and
//! numerals, units, the speed line, the logo) stays at the tokens' cap for static parts
//! (`STATIC_LUMA`), dimmed by its light and never by its alpha.
//!
//! Signature moments:
//! - **Limiter:** the three red rings turn the shift colour and hold steady (a still frame never
//!   reads as "no shift"), and the horizon line strobes in the shift colour at 10 Hz on its own
//!   clock, which starts lit; it is a 4 px line, so the flashing area stays under the tokens'
//!   strip height. The sky glows sunset orange. Nothing else flashes.
//! - **Upshift:** the new gear enters on the tokens' motion (`common::HeroGear`) and a ring of
//!   light runs once down the tunnel towards the driver, in 320 ms.
//! - **Braking:** the horizon turns red and a red wash runs down the road, like brake lights
//!   ahead.
//! - **Reduced flash:** the strobe becomes the tokens' 2 Hz breath, and the upshift ring is left
//!   out.
//!
//! Palette: the AMG MBUX dashes have their own default palette (tokens
//! `paletteModes.familyDefaults.amgMbux`, red like the real display; TJX chose it on
//! 2026-11-28). It is used while the host's palette is the factory or family one; a custom or
//! random palette replaces it.
//!
//! Telemetry is native or derived exactly: rpm (current and limiter), the tokens' shift point,
//! gear, speed (and the distance driven from it), brake, power and boost, acceleration (G), and
//! torque derived from power and rpm
//! (τ = P · 60 / (2π · rpm), rpm held at least at idle).
mod geom;
mod hardware;
#[cfg(test)]
mod tests;

use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use geom::{Geom, CRESCENT_SAMPLES, RED_RINGS, RINGS, SCALE_MAJOR_EVERY, SCALE_TICKS, SKY_SAMPLES};
use hardware::{crescent_band, Plates};

use fh6_kit::cluster::Setup;
use fh6_kit::{
    vec2, Affine, Align, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Gradient, Hysteresis, LayerOptions,
    Layout, Logo, Painter, Palette, PaletteSource, PowerUnit, Rect, Shape, SpeedUnit, Spring, Strobe, TextStyle,
    TickRing, VAlign, Vec2,
};

/// The tokens' family key of the AMG MBUX default palette.
const PALETTE_KEY: &str = "amgMbux";
/// The numeral face: a light, open grotesk close to the real display's corporate sans (OFL).
const NUMERAL_FAMILY: &str = "Barlow Semi Condensed";
/// Weights: the gear, the other numerals, the labels.
const GEAR_WEIGHT: u16 = 500;
const NUMERAL_WEIGHT: u16 = 400;
const LABEL_WEIGHT: u16 = 500;
/// The real display sets its speed and rpm in a slightly sloped light face (synthetic slant).
const NUMERAL_SLANT: f32 = 0.10;
/// The brightest that a part which never changes may show on black, as relative luminance
/// (tokens `motion.idle.staticLumaMaxPct`, 20%): labels, ticks, scale numerals, units, logo.
const STATIC_LUMA: f32 = 0.19;
/// The speed unit's light (relative luminance): over the sky it needs more than the static cap.
const SPEED_UNIT_LUMA: f32 = 0.75;
/// Relative luminance of the tokens' ink (#F4F7FB), to turn [`STATIC_LUMA`] into a light level.
const INK_LUMA: f32 = 0.92;
/// The crescents' light at rest and how it rises to full near the shift point (fill shares).
const CRESCENT_REST: f32 = 0.5;
const HEAT_FROM: f32 = 0.55;
const HEAT_TO: f32 = 0.95;
/// The sky: how far the clouds drift per metre driven (local px), and how far the horizon leans
/// per g of lateral force (radians, 5°), at most [`TILT_MAX`].
const SKY_DRIFT: f32 = 0.5;
const TILT_PER_G: f32 = 0.087_266;
const TILT_MAX: f32 = 0.14;
/// The sky warms towards sunset from this fill share; the warm veil's strongest opacity.
const WARM_FROM: f32 = 0.72;
const WARM_LEVEL: f32 = 0.55;
/// The road's lane dashes: how many on each side, the metres between two, and each dash's share
/// of the gap.
const LANE_DASHES: usize = 7;
const LANE_SPACING_M: f32 = 10.0;
const LANE_LEN: f32 = 0.45;
/// The red wash on the road and the horizon under braking, at full brake.
const BRAKE_WASH: f32 = 0.45;
/// A lit ring's soft halo: widths (times the line) and opacities, widest first.
const RING_HALO: [(f32, f32); 2] = [(6.0, 0.07), (3.0, 0.16)];
/// The rings' light when lit, far to near (share of the ink).
const RING_FAR_LIGHT: f32 = 0.55;
const RING_NEAR_LIGHT: f32 = 0.95;
/// The stage's light at its centre and at 60% of its radii (sRGB 8-bit).
const STAGE_CORE: [u8; 3] = [30, 26, 40];
const STAGE_MID: [u8; 3] = [17, 15, 25];
/// How much of the crescents goes into the bloom layer (their soft glow).
const CRESCENT_GLOW: f32 = 0.30;
/// The lavender glow on the scale bands (alpha at the rim).
const BAND_GLOW: f32 = 0.10;
/// The horizon line on the limiter: its height (local px) while it strobes.
const LIMITER_HORIZON: f32 = 4.0;
/// Share of the shift colour's light the strobing horizon keeps in the strobe's dark phase.
const STROBE_FLOOR: f32 = 0.2;
/// Added to the strobe's clock (s), so a phase edge that falls on a frame lands the same way.
const STROBE_LEAD: f32 = 1e-4;
/// The upshift ring: how long it runs down the tunnel (s) and its peak light.
const PULSE_S: f32 = 0.32;
const PULSE_LEVEL: f32 = 0.55;
/// Backlash of the ring fill (rpm): wider than the engine's bounce on the limiter (about 50 rpm
/// a frame), so the rings hold still there; a pull is drawn at most half of it behind.
const REV_BAND: f32 = 60.0;
/// How far a readout's raw input must move before its digits change (the speed's 0.6 rule):
/// rpm; torque and power in their units; boost in tenths of a psi or hundredths of a bar.
const RPM_HOLD: f32 = 100.0;
const TORQUE_HOLD: f32 = 4.0;
const POWER_HOLD: f32 = 3.0;
const BOOST_HOLD: f32 = 3.0;
/// The speed's hysteresis (tokens `motion.speedDisplay`).
const SPEED_HOLD: f32 = 0.6;
/// One mechanical horsepower (W), one newton metre in pound-feet, one bar in psi.
const WATTS_PER_HP: f32 = 745.7;
const LBFT_PER_NM: f32 = 0.737_562;
const PSI_PER_BAR: f32 = 14.503_77;
/// The side scales' first full value and the step a scale grows by when a value passes 95% of
/// it (its numerals are then quarters... thirds of a multiple of this step).
const SCALE_STEP: f32 = 300.0;
const SCALE_GROW_AT: f32 = 0.95;
/// Boost gauge range (bar) and sweep.
const BOOST_MIN_BAR: f32 = -1.0;
const BOOST_MAX_BAR: f32 = 2.0;
const BOOST_START: f32 = 2.356_194_5; // 135°
const BOOST_SWEEP: f32 = 4.712_389; // 270°
/// G-force widget: the outer ring's G.
const G_RING: f32 = 1.5;
/// Boost gauge ticks (major every fourth).
const BOOST_TICKS: u32 = 13;

/// Fonts resolved once.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    numerals: FontId,
    labels: FontId,
    wide: FontId,
    /// The host's display role at build time: a different one later means the user chose a
    /// font, and the numerals follow it.
    display_role: FontId,
}

/// Held readouts (digits that change only when the value moves past their hold).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Readouts {
    speed: Hysteresis,
    rpm: Hysteresis,
    torque: Hysteresis,
    power: Hysteresis,
    boost: Hysteresis,
    speed_shown: i64,
    rpm_shown: i64,
    torque_shown: i64,
    power_shown: i64,
    boost_shown: i64,
    /// The scales' full values (display units); they grow, never shrink, in one drive.
    torque_full: f32,
    power_full: f32,
    boost_bar: f32,
}

impl Readouts {
    fn new() -> Self {
        Self {
            speed: Hysteresis::new(0.0, SPEED_HOLD),
            rpm: Hysteresis::new(0.0, RPM_HOLD),
            torque: Hysteresis::new(0.0, TORQUE_HOLD),
            power: Hysteresis::new(0.0, POWER_HOLD),
            boost: Hysteresis::new(0.0, BOOST_HOLD),
            speed_shown: 0,
            rpm_shown: 0,
            torque_shown: 0,
            power_shown: 0,
            boost_shown: 0,
            torque_full: SCALE_STEP * 2.0,
            power_full: SCALE_STEP,
            boost_bar: 0.0,
        }
    }
}

/// The values a frame draws, in display units.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Drive {
    power_unit: PowerUnit,
    speed_unit: SpeedUnit,
    gear: Gear,
    /// Fill of the rings (0 = empty, 1 = the limiter).
    fill: f32,
    over_shift: bool,
    /// Light of the strobing rings (1 on, the floor off; the breath in reduced flash).
    strobe_light: f32,
    /// How far the horizon leans (radians), the brake (0..1), and the sky's scroll (local px)
    /// and the lane dashes' phase (0..1).
    tilt: f32,
    brake: f32,
    scroll: f32,
    lanes: f32,
    /// True while the car moves (the clouds and the dashes scroll).
    moving: bool,
    /// The felt force (screen convention) on the G widget, in g.
    g: Vec2,
    /// 0..1 progress of the upshift ring, or `None`.
    pulse: Option<f32>,
}

/// The AMG MBUX Superscreen dash.
#[derive(Debug, Clone)]
pub struct AmgMbuxSuperscreen {
    geom: Geom,
    faces: Faces,
    plates: Plates,
    /// The AMG default palette (built once) and the palette this frame draws with.
    own_palette: Palette,
    palette: Palette,
    logo: Logo,
    /// The ring fill, on the tokens' ribbon spring with feed-forward.
    fill: Spring,
    /// The sky's scroll (local px) and the lane dashes' phase.
    scroll: f32,
    lanes: f32,
    /// The rpm the rings follow after the backlash.
    rev: f32,
    gear: HeroGear,
    readouts: Readouts,
    /// Seconds since the shift point was reached (the strobe's own clock), while above it.
    strobe_t: f32,
    g_x: Spring,
    g_y: Spring,
    /// Seconds since the last upshift (the tunnel ring), large when none runs.
    since_upshift: f32,
    /// Light azimuth weight for the lit plates (0 = rest light, 1 = the swung light).
    swing: f32,
    drive: Drive,
    reduced_flash: bool,
}

impl AmgMbuxSuperscreen {
    /// Builds the dash: resolves its faces and loads its plates once.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t = setup.tokens;
        let display_role = setup.fonts.role(FontRole::Display);
        let text_role = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup
                .fonts
                .find(NUMERAL_FAMILY, GEAR_WEIGHT, false)
                .unwrap_or(display_role),
            numerals: setup
                .fonts
                .find(NUMERAL_FAMILY, NUMERAL_WEIGHT, false)
                .unwrap_or(display_role),
            labels: setup
                .fonts
                .find(NUMERAL_FAMILY, LABEL_WEIGHT, false)
                .unwrap_or(text_role),
            wide: setup.fonts.role(FontRole::Wide),
            display_role,
        };
        let own_palette = Palette::family(t, PALETTE_KEY);
        let geom = Geom::COCKPIT;
        Self {
            geom,
            faces,
            plates: Plates::load(setup),
            own_palette,
            palette: own_palette,
            logo: Logo::default(),
            fill: Spring::new(t.springs.ribbon, 0.0),
            scroll: 0.0,
            lanes: 0.0,
            rev: 0.0,
            gear: HeroGear::new(t),
            readouts: Readouts::new(),
            strobe_t: 0.0,
            g_x: Spring::new(t.springs.g_ball, 0.0),
            g_y: Spring::new(t.springs.g_ball, 0.0),
            since_upshift: PULSE_S * 2.0,
            swing: 0.0,
            drive: Drive {
                power_unit: PowerUnit::default(),
                speed_unit: SpeedUnit::default(),
                gear: Gear::Neutral,
                fill: 0.0,
                over_shift: false,
                strobe_light: 1.0,
                tilt: 0.0,
                brake: 0.0,
                scroll: 0.0,
                lanes: 0.0,
                moving: false,
                g: Vec2::ZERO,
                pulse: None,
            },
            reduced_flash: false,
        }
    }

    /// The numeral face this frame: the user's display font when they chose one.
    fn numeral_face(&self, chosen: FontId) -> FontId {
        if chosen == self.faces.display_role {
            self.faces.numerals
        } else {
            chosen
        }
    }

    /// Advances the readouts (held digits, scale fills, scale growth).
    fn update_readouts(&mut self, input: &FrameInput<'_>) {
        let s = input.signals;
        let car = input.telemetry;
        let r = &mut self.readouts;
        r.speed_shown = r.speed.update(s.speed.max(0.0)).clamp(0, 999);
        r.rpm_shown = r.rpm.update(s.rpm.max(0.0)).clamp(0, 99_999);
        // Power in the user's unit; torque from power and rpm (exact), in the unit that goes
        // with it (Nm with kW, lb-ft with hp, as the real display's markets do).
        let watts = car.power_watts.max(0.0);
        let omega = (s.rpm.max(s.idle_rpm).max(1.0)) * core::f32::consts::TAU / 60.0;
        let nm = watts / omega;
        let (power, torque) = match s.power_unit {
            PowerUnit::Kw => (watts / 1000.0, nm),
            PowerUnit::Hp => (watts / WATTS_PER_HP, nm * LBFT_PER_NM),
        };
        r.power_shown = r.power.update(power).clamp(0, 9999);
        r.torque_shown = r.torque.update(torque).clamp(0, 9999);
        r.power_full = grown(r.power_full, power);
        r.torque_full = grown(r.torque_full, torque);
        let bar = car.boost_psi / PSI_PER_BAR;
        r.boost_bar = bar;
        r.boost_shown = match s.unit {
            SpeedUnit::Mph => r.boost.update(car.boost_psi * 10.0),
            SpeedUnit::Kmh => r.boost.update(bar * 100.0),
        }
        .clamp(-9999, 9999);
    }
}

/// A scale's full value after `value`: unchanged, or grown by whole steps until `value` sits
/// under 95% of it.
fn grown(full: f32, value: f32) -> f32 {
    let mut full = full.max(SCALE_STEP);
    if value.is_finite() {
        let mut steps = 0;
        while value > full * SCALE_GROW_AT && steps < 64 {
            full += SCALE_STEP;
            steps += 1;
        }
    }
    full
}

/// Hermite smoothstep of `x` between `e0` and `e1`.
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A light level that keeps `color`'s coverage and puts it at relative luminance `luma`.
fn static_ink(ink: Color, luma: f32) -> Color {
    ink.intensity((luma / INK_LUMA).clamp(0.0, 1.0))
}

impl Cluster for AmgMbuxSuperscreen {
    fn update(&mut self, input: &FrameInput<'_>) {
        let s = input.signals;
        let dt = input.dt.max(0.0);
        self.palette = match input.palette.source {
            PaletteSource::Factory | PaletteSource::Family => self.own_palette,
            PaletteSource::Custom | PaletteSource::Random(_) => *input.palette,
        };
        self.logo.clone_from(input.logo);
        self.reduced_flash = input.reduced_flash;

        // The rings: rpm through a backlash, then the ribbon spring with feed-forward.
        let rpm = s.rpm.max(0.0);
        if rpm > self.rev + REV_BAND * 0.5 {
            self.rev = rpm - REV_BAND * 0.5;
        } else if rpm < self.rev - REV_BAND * 0.5 {
            self.rev = rpm + REV_BAND * 0.5;
        }
        let max_rpm = s.max_rpm.max(1.0);
        let target = (self.rev / max_rpm).clamp(0.0, 1.0);
        let fill = self
            .fill
            .update_tracking(target, s.rpm_rate / max_rpm, dt)
            .clamp(0.0, 1.0);

        // The limiter: the near red rings strobe on their own clock while above the shift point.
        let over_shift = s.shift_fraction >= 1.0;
        self.strobe_t = if over_shift { self.strobe_t + dt } else { 0.0 };
        let strobe_light = if self.reduced_flash {
            Strobe::breath(self.strobe_t)
        } else if Strobe::LIMITER.on(self.strobe_t + STROBE_LEAD) {
            1.0
        } else {
            STROBE_FLOOR
        };

        // Gear and the upshift ring.
        let before = self.gear.gear();
        self.gear
            .update(s.gear, if over_shift { self.strobe_t } else { 0.0 }, dt);
        if s.gear != before && self.gear.was_upshift() {
            self.since_upshift = 0.0;
        } else {
            self.since_upshift = (self.since_upshift + dt).min(PULSE_S * 2.0);
        }
        let pulse = (!self.reduced_flash && self.since_upshift < PULSE_S).then(|| self.since_upshift / PULSE_S);

        // The G widget's dot: the felt force on the tokens' gBall spring.
        let felt = crate::common::gball::felt_force(s.g_lat, s.g_long);
        let g = vec2(self.g_x.update(felt.x, dt), self.g_y.update(felt.y, dt));
        self.swing = (s.g_lat.abs() / G_RING).clamp(0.0, 1.0);

        // The sky drifts and the dashes run with the distance driven; the horizon leans with the
        // felt lateral force.
        let metres = s.speed_ms.max(0.0) * dt;
        let turn = self.geom.sky.width().max(1.0);
        self.scroll = (self.scroll + metres * SKY_DRIFT).rem_euclid(turn);
        self.lanes = (self.lanes + metres / LANE_SPACING_M).rem_euclid(1.0);
        let tilt = (g.x * TILT_PER_G).clamp(-TILT_MAX, TILT_MAX);

        self.update_readouts(input);
        self.drive = Drive {
            power_unit: s.power_unit,
            speed_unit: s.unit,
            gear: s.gear,
            fill,
            over_shift,
            strobe_light,
            tilt,
            brake: s.brake.clamp(0.0, 1.0),
            scroll: self.scroll,
            lanes: self.lanes,
            moving: s.speed_ms > 0.05,
            g,
            pulse,
        };
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = self.palette;
        let g = &self.geom;
        let chosen = p.fonts().role(FontRole::Display);
        let numerals = self.numeral_face(chosen);
        let gear_face = if chosen == self.faces.display_role {
            self.faces.gear
        } else {
            chosen
        };

        self.draw_stage(p);
        self.draw_sky(p, &pal);
        self.plates.draw_assembly(p, g, self.swing);
        self.plates.draw_crescents(p, g, &pal, self.heat());
        self.draw_scales(p, &pal, numerals);
        self.draw_widgets(p, &pal, numerals);
        self.draw_speed_line(p, &pal);
        logo_zone(p, g.logo, &self.logo, self.faces.wide, LogoVariant::Full, &pal);
        // The rev counter, outside the bloom layer: the bloom resolves over everything drawn
        // after it, so the rings' glow would shine through the road. Each lit ring carries its
        // own soft halo instead, and the road below covers both.
        self.draw_rings(p, &pal);

        // Rank 1 to 3 light, in one bloom layer: the crescents' soft red glow, the scales' live
        // fills and the G dot.
        p.layer(LayerOptions::bloom(), |p| {
            self.draw_crescent_glow(p, &pal);
            self.draw_scale_fills(p, &pal);
            let dot = g.g_widget + self.drive.g * (g.widget_r / G_RING);
            p.fill(&Shape::circle(dot, 7.0), pal.primary_text);
        });
        // The road over the rings (they light the tunnel walls, never the road), its dashes and
        // the horizon.
        self.plates.draw_road(p, g);
        self.draw_lanes(p, &pal);
        self.draw_horizon(p, &pal);
        // The gear is the brightest type, outside the bloom: its glow would spill past the
        // envelope's corner, and the real display's gear does not glow.
        self.gear.draw_with(p, g.gear, gear_face, g.gear_cap, pal.roles.ink);
        self.draw_stack(p, &pal);
        self.draw_speed(p, &pal, numerals);
        self.draw_rpm(p, &pal, numerals);
    }

    fn animating(&self) -> bool {
        !self.fill.settled(self.fill.value(), 1e-4)
            || self.gear.animating()
            || self.drive.over_shift
            || self.drive.pulse.is_some()
            || self.drive.moving
            || !self.g_x.settled(self.g_x.value(), 1e-4)
            || !self.g_y.settled(self.g_y.value(), 1e-4)
    }
}

impl AmgMbuxSuperscreen {
    /// The soft charcoal stage: an ellipse with a radial fall to nothing, so no edge shows.
    fn draw_stage(&self, p: &mut Painter<'_>) {
        let g = &self.geom;
        let [r0, g0, b0] = STAGE_CORE;
        let [r1, g1, b1] = STAGE_MID;
        p.scoped(|p| {
            p.translate(g.stage);
            p.transform(&Affine::scale_xy(g.stage_r.x, g.stage_r.y));
            p.fill(
                &Shape::circle(Vec2::ZERO, 1.0),
                Gradient::radial(Vec2::ZERO, 1.0)
                    .stop(0.0, Color::srgb8(r0, g0, b0, 255))
                    .stop(0.6, Color::srgb8(r1, g1, b1, 255))
                    .stop(1.0, Color::TRANSPARENT),
            );
        });
    }

    /// The crescents' light: dim at low revs, full red near the shift point.
    fn heat(&self) -> f32 {
        let k = smoothstep(HEAT_FROM, HEAT_TO, self.drive.fill);
        CRESCENT_REST + (1.0 - CRESCENT_REST) * k
    }

    /// The sky through the tunnel, leaned with lateral G: the clouds, a warm veil near the shift
    /// point and a vignette towards the tunnel's rim.
    fn draw_sky(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let d = &self.drive;
        let mut window = [Vec2::ZERO; SKY_SAMPLES];
        let n = g.sky_window(d.tilt, &mut window);
        if n < 3 {
            return;
        }
        let window = &window[..n];
        let pivot = g.horizon_pivot();
        p.scoped(|p| {
            p.rotate_about(d.tilt, pivot);
            self.plates.draw_sky(p, g, window, d.scroll);
            let shape = Shape::Polygon { points: window };
            let warm = smoothstep(WARM_FROM, 1.0, d.fill) * WARM_LEVEL;
            if warm > 1e-3 {
                let top = g.horizon - g.ring(RINGS - 1).1 * 2.0;
                p.fill(
                    &shape,
                    Gradient::linear(vec2(0.0, top), vec2(0.0, g.horizon))
                        .stop(0.0, Color::TRANSPARENT)
                        .stop(0.7, Color::srgb8(194, 50, 30, 255).with_alpha(0.7 * warm))
                        .stop(1.0, Color::srgb8(255, 122, 58, 255).with_alpha(warm)),
                );
            }
            let (c, r) = g.ring(RINGS - 1);
            let c = pivot + (c - pivot).rotate(-d.tilt);
            p.fill(
                &shape,
                Gradient::radial(c + vec2(0.0, r * 0.24), r)
                    .stop(0.45, Color::TRANSPARENT)
                    .stop(0.85, Color::BLACK.with_alpha(0.55))
                    .stop(1.0, Color::BLACK.with_alpha(0.9)),
            );
        });
        let _ = pal;
    }

    /// The paint of a ring line in `color`: faint above the horizon, where the sky shows through.
    fn ring_paint(&self, color: Color) -> Gradient {
        let g = &self.geom;
        Gradient::linear(vec2(0.0, g.horizon - g.ceiling_fade), vec2(0.0, g.horizon))
            .stop(0.0, color.fade(g.ceiling_floor))
            .stop(1.0, color)
    }

    /// The rev counter: the rings light from the far end towards the driver as the revs climb,
    /// the nearest [`RED_RINGS`] red; on the limiter they hold steady in the shift colour (the
    /// horizon line is the strobe).
    fn draw_rings(&self, p: &mut Painter<'_>, pal: &Palette) {
        let d = &self.drive;
        let lit = d.fill * RINGS as f32;
        for k in 0..RINGS {
            let red = k >= RINGS - RED_RINGS;
            let share = (lit - k as f32 + 0.5).clamp(0.0, 1.0);
            let u = (k as f32 + 1.0) / RINGS as f32;
            let color = if d.over_shift && red {
                pal.shift
            } else if share > 1e-3 {
                if red {
                    pal.primary_text.intensity(share)
                } else {
                    pal.roles
                        .ink
                        .intensity(share * (RING_FAR_LIGHT + (RING_NEAR_LIGHT - RING_FAR_LIGHT) * u))
                }
            } else {
                continue;
            };
            let (c, r) = self.geom.ring(k);
            let w = (1.3 + 2.2 * u) * if d.over_shift && red { 1.6 } else { 1.0 };
            for (width, alpha) in RING_HALO {
                p.fill(&Shape::ring(c, r, w * width), self.ring_paint(color.fade(alpha)));
            }
            p.fill(&Shape::ring(c, r, w), self.ring_paint(color));
        }
        self.draw_pulse(p, pal);
    }

    /// The lane dashes on the road, in perspective, running towards the driver with the
    /// distance driven.
    fn draw_lanes(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let span = g.floor_bottom - g.horizon;
        let half = |s: f32| g.floor_top_half + (g.floor_bottom_half - g.floor_top_half) * s * s;
        for i in 0..LANE_DASHES {
            let s0 = (i as f32 + self.drive.lanes) / LANE_DASHES as f32;
            let s1 = (s0 + LANE_LEN / LANE_DASHES as f32).min(1.0);
            if s1 <= s0 {
                continue;
            }
            let (y0, y1) = (g.horizon + span * s0 * s0, g.horizon + span * s1 * s1);
            // Fade in at the horizon and out at the bottom edge, so no dash pops.
            let light = (0.75 - 0.5 * s0) * smoothstep(0.0, 0.08, s0) * (1.0 - smoothstep(0.9, 1.0, s1));
            if light <= 1e-3 {
                continue;
            }
            let color = Color::srgb8(174, 182, 192, 255).intensity(light);
            for sx in [-1.0, 1.0] {
                p.fill(
                    &Shape::line(
                        vec2(sx * half(s0) * 0.5, y0),
                        vec2(sx * half(s1) * 0.5, y1),
                        1.2 + 1.6 * s0,
                    ),
                    color,
                );
            }
        }
        // Under braking a red wash runs down the road from the horizon, like brake lights ahead.
        let brake = self.drive.brake * BRAKE_WASH;
        if brake > 1e-3 {
            let f = g.floor();
            p.fill(
                &Shape::Polygon { points: &f },
                Gradient::linear(vec2(0.0, g.horizon), vec2(0.0, g.floor_bottom))
                    .stop(0.0, pal.primary_text.with_alpha(brake))
                    .stop(0.4, pal.primary_text.with_alpha(brake * 0.18))
                    .stop(1.0, Color::TRANSPARENT),
            );
        }
    }

    /// The horizon line, leaned with lateral G; it warms red under braking.
    fn draw_horizon(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let d = &self.drive;
        let half = g.ring(RINGS - 1).1 - 4.0;
        // On the limiter the horizon is the strobe: a thin line, so the flashing area stays
        // under the tokens' strip height (the rings hold steady in the shift colour).
        let (color, h) = if d.over_shift {
            (pal.shift.intensity(d.strobe_light), LIMITER_HORIZON)
        } else {
            let calm = Color::srgb8(242, 248, 255, 255);
            (calm.lerp(pal.primary_text.lerp(pal.roles.ink, 0.35), d.brake), 2.5)
        };
        p.scoped(|p| {
            p.rotate_about(d.tilt, g.horizon_pivot());
            p.fill(
                &Shape::rect(Rect::from_points(
                    vec2(-half, g.horizon - h * 0.5),
                    vec2(half, g.horizon + h * 0.5),
                )),
                Gradient::linear(vec2(-half, 0.0), vec2(half, 0.0))
                    .stop(0.0, Color::TRANSPARENT)
                    .stop(0.5, color)
                    .stop(1.0, Color::TRANSPARENT),
            );
        });
    }

    /// A faint copy of the crescents in the bloom layer: their soft red glow into the black, as
    /// the real display's crescents bleed light.
    fn draw_crescent_glow(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
        for side in [-1.0, 1.0] {
            let n = crescent_band(g, 0.0, 1.0, 1.0, side, &mut pts);
            p.fill(
                &Shape::Polygon { points: &pts[..n] },
                pal.primary.with_alpha(CRESCENT_GLOW * self.heat()),
            );
        }
    }

    /// The ring of light that runs once from the far end of the tunnel to the driver after an
    /// upshift (the road covers its lower part).
    fn draw_pulse(&self, p: &mut Painter<'_>, pal: &Palette) {
        let Some(t) = self.drive.pulse else {
            return;
        };
        let g = &self.geom;
        let e = 1.0 - (1.0 - t).powi(3);
        let (c1, r1) = g.ring(RINGS - 1);
        let c = g.ring_far + (c1 - g.ring_far) * e;
        let r = g.ring_far_r + (r1 - g.ring_far_r) * e;
        let light = PULSE_LEVEL * (1.0 - t).powf(1.5);
        p.fill(
            &Shape::ring(c, r, (5.0 * (1.0 - t) + 1.5).max(1.0)),
            self.ring_paint(pal.primary.lerp(pal.roles.ink, 0.3).with_alpha(light)),
        );
    }

    /// The scales: ticks inward from the band's rim, numerals on the band and each live value
    /// with its unit under its band (the bands themselves are on the assembly plate).
    fn draw_scales(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        // The real bands glow a faint lavender from their rim inwards.
        let mid = (g.band_in + g.band_out) * 0.5;
        for side in [-1.0_f32, 1.0] {
            let a0 = g.band_angle(0.0, side);
            let a1 = g.band_angle(1.0, side);
            p.fill(
                &Shape::arc(g.tunnel, mid, g.band_out - g.band_in, a0, a1 - a0, Cap::Butt),
                Gradient::radial(g.tunnel, g.band_out)
                    .stop(g.band_in / g.band_out, Color::TRANSPARENT)
                    .stop(1.0, Color::srgb8(170, 150, 225, 255).with_alpha(BAND_GLOW)),
            );
        }
        let r = &self.readouts;
        let ink = static_ink(pal.roles.ink, STATIC_LUMA);
        let mut num_style = TextStyle::new(self.faces.labels, g.scale_cap).paint(ink).tabular();
        num_style.slant = NUMERAL_SLANT;
        let unit_style = TextStyle::new(self.faces.labels, g.unit_cap)
            .paint(ink)
            .tracking(0.04)
            .align(Align::Center, VAlign::CapMiddle);
        let t0 = g.scale_t(0.0);
        let t1 = g.scale_t(1.0);
        for (side, full, shown) in [
            (-1.0_f32, r.torque_full, r.torque_shown),
            (1.0, r.power_full, r.power_shown),
        ] {
            // The kit's ticks take a positive sweep: start at the band's lower end on the left
            // and at its upper end on the right (the pattern is symmetric).
            let (a0, a1) = (g.band_angle(t0, side), g.band_angle(t1, side));
            let (start, sweep) = if a1 > a0 { (a0, a1 - a0) } else { (a1, a0 - a1) };
            let ticks = TickRing::new(g.tunnel, g.band_out - 1.0, start, sweep, SCALE_TICKS as u32)
                .minor(7.0, 1.2)
                .major(SCALE_MAJOR_EVERY as u32, 14.0, 2.2);
            p.fill(&Shape::Ticks(ticks), ink);
            let majors = SCALE_TICKS / SCALE_MAJOR_EVERY;
            for m in 0..=majors {
                let f = m as f32 / majors as f32;
                let pos = g.scale_point(f, g.scale_numeral_r, side);
                let value = (full * f).round() as i64;
                p.number(pos, value, digits(value), &num_style, None);
            }
            let mut value_style = TextStyle::new(numerals, g.value_cap).paint(pal.roles.ink).tabular();
            value_style.slant = NUMERAL_SLANT;
            let pos = if side < 0.0 { g.torque_value } else { g.power_value };
            let unit = if side < 0.0 {
                torque_unit(self.drive.power_unit)
            } else {
                power_unit(self.drive.power_unit)
            };
            p.number(pos, shown, digits(shown), &value_style, None);
            p.text(pos + vec2(0.0, g.value_unit_dy), unit, &unit_style);
        }
    }

    /// The scales' live fills along the bands' rims (inside the bloom layer).
    fn draw_scale_fills(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let r = &self.readouts;
        for (side, full, now) in [
            (-1.0_f32, r.torque_full, r.torque_shown as f32),
            (1.0, r.power_full, r.power_shown as f32),
        ] {
            let v = (now / full).clamp(0.0, 1.0);
            if v <= 1e-3 {
                continue;
            }
            let a0 = g.band_angle(g.scale_t(0.0), side);
            let a1 = g.band_angle(g.scale_t(v), side);
            p.fill(
                &Shape::arc(g.tunnel, g.band_out + 1.0, 6.0, a0, a1 - a0, Cap::Butt),
                pal.roles.ink.intensity(0.9),
            );
        }
    }

    /// The G-Force widget (left, over the rendered lens) and the engine-data widget with boost
    /// (right, over the rendered gauge).
    fn draw_widgets(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let ink = static_ink(pal.roles.ink, STATIC_LUMA);
        let title = TextStyle::new(self.faces.labels, g.label_cap)
            .paint(ink)
            .tracking(0.02)
            .align(Align::Center, VAlign::CapMiddle);
        let small = TextStyle::new(self.faces.labels, g.unit_cap)
            .paint(ink)
            .tracking(0.04)
            .align(Align::Center, VAlign::CapMiddle);
        let rw = g.widget_r;

        // G-Force: the lens, its red rim, rings at a third and two thirds, a crosshair; the dot
        // is drawn in the bloom layer.
        let c = g.g_widget;
        self.plates.draw_lens(p, g, pal, self.swing);
        p.text(c - vec2(0.0, rw + g.title_dy), "G-Force", &title);
        p.fill(&Shape::ring(c, rw, 2.5), pal.primary_text);
        for k in [0.66, 0.33] {
            p.fill(&Shape::ring(c, rw * k, 1.6), pal.primary.intensity(0.85));
        }
        let cross = pal.primary.intensity(0.6);
        p.fill(&Shape::line(c - vec2(rw, 0.0), c + vec2(rw, 0.0), 1.1), cross);
        p.fill(&Shape::line(c - vec2(0.0, rw), c + vec2(0.0, rw), 1.1), cross);

        // Engine data: boost on a 270° gauge, the value in the middle.
        let c = g.boost_widget;
        self.plates.draw_gauge(p, g, self.swing);
        p.text(c - vec2(0.0, rw + g.title_dy), "Engine data", &title);
        let ticks = TickRing::new(c, rw - 4.0, BOOST_START, BOOST_SWEEP, BOOST_TICKS)
            .minor(4.0, 1.1)
            .major(4, 9.0, 2.0);
        p.fill(&Shape::Ticks(ticks), ink);
        let span = BOOST_MAX_BAR - BOOST_MIN_BAR;
        let zero = -BOOST_MIN_BAR / span;
        let t = ((self.readouts.boost_bar - BOOST_MIN_BAR) / span).clamp(0.0, 1.0);
        let (a0, sw) = if t >= zero {
            (BOOST_START + BOOST_SWEEP * zero, BOOST_SWEEP * (t - zero))
        } else {
            (BOOST_START + BOOST_SWEEP * t, BOOST_SWEEP * (zero - t))
        };
        if sw > 1e-3 {
            p.fill(&Shape::arc(c, rw - 4.0, 5.0, a0, sw, Cap::Butt), pal.primary_text);
        }
        let (value, unit, decimals) = match self.drive.speed_unit {
            SpeedUnit::Mph => (self.readouts.boost_shown, "psi", 1),
            SpeedUnit::Kmh => (self.readouts.boost_shown, "bar", 2),
        };
        let mut style = TextStyle::new(numerals, g.value_cap).paint(pal.roles.ink).tabular();
        style.slant = NUMERAL_SLANT;
        draw_fixed(p, c + vec2(0.0, -3.0), value, decimals, &style);
        p.text(c + vec2(0.0, 27.0), unit, &small);
        p.text(c + vec2(0.0, rw + 29.0), "Boost", &small);
    }

    /// The thin speed line under the speed, fading out at its ends.
    fn draw_speed_line(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let ink = static_ink(pal.roles.ink, STATIC_LUMA * 0.9);
        p.fill(
            &Shape::rect(Rect::from_points(
                vec2(-g.line_half, g.line_y - 0.8),
                vec2(g.line_half, g.line_y + 0.8),
            )),
            Gradient::linear(vec2(-g.line_half, 0.0), vec2(g.line_half, 0.0))
                .stop(0.0, Color::TRANSPARENT)
                .stop(0.5, ink)
                .stop(1.0, Color::TRANSPARENT),
        );
    }

    /// The selector stack: the gears either side of the engaged one in a column, with a red `M`
    /// between them (the engaged gear itself is drawn large beside it).
    fn draw_stack(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let style = TextStyle::new(self.faces.labels, g.stack_cap)
            .paint(pal.roles.ink.intensity(0.55))
            .align(Align::Center, VAlign::CapMiddle);
        let (up, down) = neighbours(self.drive.gear);
        p.text(g.stack_row(-1.0), up, &style);
        p.text(g.stack_row(0.0), "M", &style.paint(pal.primary_text));
        p.text(g.stack_row(1.0), down, &style);
    }

    /// The speed and its unit.
    fn draw_speed(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        // A soft dark halo behind the digits keeps them clear of the rings and the sky.
        p.scoped(|p| {
            p.translate(g.speed);
            p.transform(&Affine::scale_xy(g.speed_cap * 3.6, g.speed_cap * 1.2));
            p.fill(
                &Shape::circle(Vec2::ZERO, 1.0),
                Gradient::radial(Vec2::ZERO, 1.0)
                    .stop(0.0, Color::srgb8(0, 0, 0, 240))
                    .stop(0.62, Color::srgb8(0, 0, 0, 215))
                    .stop(1.0, Color::TRANSPARENT),
            );
        });
        let mut style = TextStyle::new(numerals, g.speed_cap).paint(pal.roles.ink).tabular();
        style.slant = NUMERAL_SLANT;
        let v = self.readouts.speed_shown;
        p.number(g.speed, v, digits(v), &style, None);
        // The unit sits over the sky and the rings, so it is brighter than the other static
        // labels (the glance gate's 4.5:1 for a data label; a small area, named in the handover).
        let unit = TextStyle::new(self.faces.labels, g.speed_unit_cap)
            .paint(static_ink(pal.roles.ink, SPEED_UNIT_LUMA))
            .tracking(0.04)
            .align(Align::Left, VAlign::CapMiddle);
        p.text(g.speed_unit, speed_unit(self.drive.speed_unit), &unit);
    }

    /// The rpm readout in the tunnel floor.
    fn draw_rpm(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let mut style = TextStyle::new(numerals, g.rpm_cap).paint(pal.roles.ink).tabular();
        style.slant = NUMERAL_SLANT;
        let v = (self.readouts.rpm_shown / 10) * 10;
        let cells = if v >= 10_000 { 5 } else { 4 };
        p.number(g.rpm, v, cells, &style, None);
        let unit = TextStyle::new(self.faces.labels, g.unit_cap)
            .paint(static_ink(pal.roles.ink, STATIC_LUMA))
            .tracking(0.04);
        p.text(g.rpm_unit, "rpm", &unit);
    }
}

/// The gears above and below `gear` in the selector stack (R, N, 1 to 10), blank past the ends.
fn neighbours(gear: Gear) -> (&'static str, &'static str) {
    const ORDER: [&str; 12] = ["R", "N", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10"];
    let i = match gear {
        Gear::Reverse => 0,
        Gear::Neutral => 1,
        Gear::Forward(n) => usize::from(n.clamp(1, 10)) + 1,
    };
    let up = ORDER.get(i + 1).copied().unwrap_or("");
    let down = if i == 0 { "" } else { ORDER[i - 1] };
    (up, down)
}

/// How many digit cells `value` needs (1 to 5; a minus sign is not counted).
fn digits(value: i64) -> u8 {
    match value.unsigned_abs() {
        0..=9 => 1,
        10..=99 => 2,
        100..=999 => 3,
        1000..=9999 => 4,
        _ => 5,
    }
}

/// The speed unit as the real display prints it.
fn speed_unit(unit: SpeedUnit) -> &'static str {
    match unit {
        SpeedUnit::Mph => "mph",
        SpeedUnit::Kmh => "km/h",
    }
}

/// The power unit as the real display prints it.
fn power_unit(unit: PowerUnit) -> &'static str {
    match unit {
        PowerUnit::Kw => "kW",
        PowerUnit::Hp => "hp",
    }
}

/// The torque unit that goes with the power unit.
fn torque_unit(unit: PowerUnit) -> &'static str {
    match unit {
        PowerUnit::Kw => "Nm",
        PowerUnit::Hp => "lb-ft",
    }
}

/// Draws `value` (scaled by 10^`decimals`) as a fixed-point number centred on `pos`, with no
/// allocation.
fn draw_fixed(p: &mut Painter<'_>, pos: Vec2, value: i64, decimals: u8, style: &TextStyle) {
    let mut buf = [0u8; crate::common::race::FIXED_BUF];
    let text = crate::common::race::format_fixed(value, decimals, &mut buf);
    p.text(pos, text, style);
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(AmgMbuxSuperscreen::new(setup))
}

/// Tokens access for the tests (the default palette key).
#[cfg(test)]
pub(crate) fn own_palette(tokens: &fh6_kit::Tokens) -> Palette {
    Palette::family(tokens, PALETTE_KEY)
}
