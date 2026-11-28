//! AMG MBUX Superscreen (`ClusterTheme::AmgMbuxSuperscreen`), catalog row 4: the driver display
//! of the E-Class generation that carries the MBUX Superscreen (W214, 2023 on, and the
//! Mercedes-AMG E 53), in its Sport (press: "Sporty") style. Supercar category, Modern Digital
//! 2027 family, cockpit. Research: `RESEARCH.md`. Board: none yet (the rebuild follows it).
//!
//! The real display is one free-standing 12.3-inch screen (2.67:1), so this dash is that screen:
//! a black glass panel in a thin piano-black bezel (the Blender housing), and on it the Sport
//! style's red "tunnel" rev counter. Zone table (local 4K px round the cockpit anchor; `geom.rs`
//! holds it as data):
//!
//! | Zone | Where | Size | Holds |
//! |---|---|---|---|
//! | Housing | x ±720, y −314 .. 236 | 1440 × 550, r 30 | the display unit (Blender plate, or a procedural black panel) |
//! | Screen | x ±706, y −300 .. 222 | 1412 × 522, r 18 | the black glass every zone sits on |
//! | Crescents | round (0, −36), r 278 | 40 px at 9 and 3 o'clock, tapering to 0, ±52° | rank 3: the rev counter, filling bottom to top on both sides; a white notch at the shift point |
//! | Tunnel | rings receding from r 245 at (0, −36) to r 97 at (0, −62) | 6 rings | the tunnel's depth (Blender plate tinted by the palette, or procedural rings) |
//! | Disc | (0, −62) | r 92 | rank 1: the gear, cap 120, at the tunnel's vanishing point |
//! | Speed | (0, −230) | cap 60 | rank 2: the speed, between the crescents' tips; its unit under it at (0, −180) |
//! | Shift strip | y −286, x ±96 .. ±680 | 7 segments a side, 10 px tall | rank 1: along the glass's top edge, filling outside-in; only it strobes |
//! | Floor | trapezoid y 92 .. 196, half widths 128 .. 214 | – | rank 4: the rpm readout, cap 44, and its unit |
//! | Side scales | r 360 round the tunnel centre, 138° .. 218° (mirrored right), numerals inside at r 314 | 21 ticks | rank 4: current torque (left) and power (right), the live fill outside the line, each live value under its widget |
//! | G-force | (−566, −40) | r 84 | rank 4: the felt force as a dot on rings and a crosshair |
//! | Engine data | (566, −40) | r 84 | rank 4: boost on a round gauge |
//! | Logo | x −686 .. −486, y 156 .. 204 | – | rank 5: the car's logo or the custom name |
//! | Transmission | x 590 .. 686, y 156 .. 204 | – | rank 5: the `M` tag and the gear, as the real display's selector box |
//!
//! Brightness follows the rank: the gear and the lit shift segments bloom; the lit crescents
//! burn below them with a brighter head; the speed is the brightest digit outside the bloom;
//! everything that never changes (labels, scale ticks and numerals, units, the logo) stays at
//! the tokens' cap for static parts (`STATIC_LUMA`), dimmed by its light and never by its alpha.
//!
//! Signature moments:
//! - **Limiter:** from the shift point the 14 shift segments turn the shift colour and strobe at
//!   10 Hz on their own clock, which starts lit; the dark phase keeps a fifth of the light. The
//!   crescents hold full and steady with white-hot tips above the shift notch. Nothing else
//!   flashes (the real display's red flash of large areas is left out on purpose).
//! - **Upshift:** the new gear enters on the tokens' motion (`common::HeroGear`) and a ring of
//!   light runs once down the tunnel, from the disc out to the crescents, in 320 ms.
//! - **Reduced flash:** the strobe becomes the tokens' 2 Hz breath, and the tunnel ring is left
//!   out.
//!
//! Palette: the AMG MBUX dashes have their own default palette (tokens
//! `paletteModes.familyDefaults.amgMbux`, red like the real display; TJX chose it on
//! 2026-11-28). It is used while the host's palette is the factory or family one; a custom or
//! random palette replaces it.
//!
//! Telemetry is native or derived exactly: rpm (current and limiter), the tokens' shift point,
//! gear, speed, power and boost, acceleration (G), and torque derived from power and rpm
//! (τ = P · 60 / (2π · rpm), rpm held at least at idle).
mod geom;
mod hardware;
#[cfg(test)]
mod tests;

use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use geom::{Geom, CRESCENT_SAMPLES, SCALE_MAJOR_EVERY, SCALE_TICKS, SHIFT_PER_SIDE};
use hardware::Plates;

use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Gradient, Hysteresis, LayerOptions,
    Layout, Logo, Painter, Palette, PaletteSource, PowerUnit, Rect, Shape, SpeedUnit, Spring, Strobe, Stroke,
    TextStyle, TickRing, VAlign, Vec2,
};

/// The tokens' family key of the AMG MBUX default palette.
const PALETTE_KEY: &str = "amgMbux";
/// The numeral face: a light, open grotesk close to the real display's corporate sans (OFL).
const NUMERAL_FAMILY: &str = "Barlow Semi Condensed";
/// Weights: the gear, the other numerals, the labels.
const GEAR_WEIGHT: u16 = 400;
const NUMERAL_WEIGHT: u16 = 300;
const LABEL_WEIGHT: u16 = 500;
/// The real display sets its speed and rpm in a slightly sloped light face (synthetic slant).
const NUMERAL_SLANT: f32 = 0.10;
/// The brightest that a part which never changes may show on black, as relative luminance
/// (tokens `motion.idle.staticLumaMaxPct`, 20%): labels, ticks, scale numerals, units, logo.
const STATIC_LUMA: f32 = 0.19;
/// Relative luminance of the tokens' ink (#F4F7FB), to turn [`STATIC_LUMA`] into a light level.
const INK_LUMA: f32 = 0.92;
/// Light level of the crescents' unlit body and of the tunnel's rings (share of the primary).
const TRACK_LEVEL: f32 = 0.20;
const RING_LEVEL: f32 = 0.16;
/// Light of the lit crescent at its tail and at its head (share of the primary), and how much
/// of the head is drawn brighter (share of the crescent).
const LIT_TAIL: f32 = 0.62;
const LIT_HEAD: f32 = 1.0;
const HEAD_SHARE: f32 = 0.10;
/// How far the lit crescent above the shift notch leans to the ink (white-hot).
const HOT_MIX: f32 = 0.45;
/// Light of the vanishing disc at its centre, at rest and at the shift point.
const DISC_REST: f32 = 0.09;
const DISC_SHIFT: f32 = 0.16;
/// Share of the shift colour's light the segments keep in the strobe's dark phase.
const STROBE_FLOOR: f32 = 0.2;
/// Added to the strobe's clock (s), so a phase edge that falls on a frame lands the same way.
const STROBE_LEAD: f32 = 1e-4;
/// The upshift ring: how long it runs down the tunnel (s) and its peak light.
const PULSE_S: f32 = 0.32;
const PULSE_LEVEL: f32 = 0.55;
/// Backlash of the crescent fill (rpm): wider than the engine's bounce on the limiter (about
/// 50 rpm a frame), so the crescents hold still there; a pull is drawn at most half of it behind.
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
/// G-force widget: the outer ring's G and the inner ring's.
const G_RING: f32 = 1.5;
const G_INNER: f32 = 0.75;

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
    /// The live torque and power (display units), for the scale fills.
    torque_now: f32,
    power_now: f32,
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
            torque_now: 0.0,
            power_now: 0.0,
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
    /// Fill of the crescents (0 = empty, 1 = the limiter) and the shift notch on the same scale.
    fill: f32,
    notch: f32,
    /// Shift segments lit, from 0 to [`SHIFT_PER_SIDE`] (fractional).
    shift_level: f32,
    over_shift: bool,
    /// Light of the shift segments while strobing (1 on, the floor off; the breath in reduced
    /// flash).
    strobe_light: f32,
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
    /// The crescent fill, on the tokens' ribbon spring with feed-forward.
    fill: Spring,
    /// The rpm the crescents follow after the backlash.
    rev: f32,
    gear: HeroGear,
    readouts: Readouts,
    /// Seconds since the shift point was reached (the strobe's own clock), while above it.
    strobe_t: f32,
    g_x: Spring,
    g_y: Spring,
    /// Seconds since the last upshift (the tunnel ring), large when none runs.
    since_upshift: f32,
    /// Light azimuth weight for the housing plates (0 = rest light, 1 = the swung light).
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
                notch: 1.0,
                shift_level: 0.0,
                over_shift: false,
                strobe_light: 1.0,
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
        r.power_now = power;
        r.torque_now = torque;
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

        // The crescents: rpm through a backlash, then the ribbon spring with feed-forward.
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
        let notch = (s.shift_rpm / max_rpm).clamp(0.05, 1.0);

        // Shift segments: they light outside-in from the first rev stage to the shift point.
        let start = self.palette.stages[0];
        let level = ((s.shift_fraction - start) / (1.0 - start).max(1e-3)).clamp(0.0, 1.0);
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

        self.update_readouts(input);
        self.drive = Drive {
            power_unit: s.power_unit,
            speed_unit: s.unit,
            gear: s.gear,
            fill,
            notch,
            shift_level: level * SHIFT_PER_SIDE as f32,
            over_shift,
            strobe_light,
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

        self.plates.draw_housing(p, g, &pal, self.swing);
        self.draw_haze(p, &pal);
        self.plates.draw_tunnel(p, g, &pal);
        self.draw_crescent_tracks(p, &pal);
        self.draw_scales(p, &pal, numerals);
        self.draw_widgets(p, &pal, numerals);
        self.draw_disc(p, &pal);
        self.draw_footer(p, &pal);

        // Rank 1 to 3 light, in one bloom layer: the lit crescents, the upshift ring, the lit
        // shift segments and the gear.
        p.layer(LayerOptions::bloom(), |p| {
            self.draw_crescent_fill(p, &pal);
            self.draw_pulse(p, &pal);
            self.draw_shift(p, &pal);
            self.gear.draw_with(p, g.disc, gear_face, g.gear_cap, pal.roles.ink);
        });
        self.draw_unlit_shift(p, &pal);
        self.draw_speed(p, &pal, numerals);
        self.draw_rpm(p, &pal, numerals);
    }

    fn animating(&self) -> bool {
        !self.fill.settled(self.fill.value(), 1e-4)
            || self.gear.animating()
            || self.drive.over_shift
            || self.drive.pulse.is_some()
            || !self.g_x.settled(self.g_x.value(), 1e-4)
            || !self.g_y.settled(self.g_y.value(), 1e-4)
    }
}

impl AmgMbuxSuperscreen {
    /// A faint red haze in the glass round the tunnel, as the real screen's backlight bleeds.
    fn draw_haze(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let r = g.crescent_r * 1.55;
        let c = g.tunnel;
        p.fill(
            &Shape::rect(Rect::from_points(
                vec2(c.x - r, (c.y - r).max(g.screen.min.y)),
                vec2(c.x + r, (c.y + r).min(g.screen.max.y)),
            )),
            Gradient::radial(c, r)
                .stop(0.0, pal.primary.intensity(0.05))
                .stop(0.55, pal.primary.intensity(0.022))
                .stop(1.0, Color::TRANSPARENT),
        );
    }

    /// The crescents' unlit bodies and the shift notches.
    fn draw_crescent_tracks(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let track = pal.primary.intensity(TRACK_LEVEL);
        for side in [-1.0, 1.0] {
            let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
            let n = crescent_polygon(g, 0.0, 1.0, side, &mut pts);
            p.fill(&Shape::Polygon { points: &pts[..n] }, track);
            // The shift notch: a short white bar across the crescent.
            let a = g.crescent_angle(self.drive.notch);
            let dir = vec2(if side < 0.0 { a.cos() } else { -a.cos() }, a.sin());
            let outer = g.tunnel + dir * (g.crescent_r + 6.0);
            let inner = g.tunnel + dir * (g.crescent_r - g.crescent_width(self.drive.notch) - 6.0);
            p.fill(&Shape::line(inner, outer, 3.0), static_ink(pal.roles.ink, STATIC_LUMA));
        }
    }

    /// The lit crescents: the tail at [`LIT_TAIL`] of the primary, the head brighter, and the
    /// part above the shift notch white-hot.
    fn draw_crescent_fill(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let fill = self.drive.fill;
        if fill <= 1e-3 {
            return;
        }
        let head_from = (fill - HEAD_SHARE).max(0.0);
        let below = fill.min(self.drive.notch);
        for side in [-1.0, 1.0] {
            let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
            let n = crescent_polygon(g, 0.0, below.min(head_from).max(0.0), side, &mut pts);
            if n > 2 {
                p.fill(&Shape::Polygon { points: &pts[..n] }, pal.primary.intensity(LIT_TAIL));
            }
            let n = crescent_polygon(g, head_from.min(below), below, side, &mut pts);
            if n > 2 {
                p.fill(&Shape::Polygon { points: &pts[..n] }, pal.primary.intensity(LIT_HEAD));
            }
            if fill > self.drive.notch {
                let n = crescent_polygon(g, self.drive.notch, fill, side, &mut pts);
                if n > 2 {
                    p.fill(
                        &Shape::Polygon { points: &pts[..n] },
                        pal.primary.lerp(pal.roles.ink, HOT_MIX),
                    );
                }
            }
        }
    }

    /// The ring that runs down the tunnel after an upshift.
    fn draw_pulse(&self, p: &mut Painter<'_>, pal: &Palette) {
        let Some(t) = self.drive.pulse else {
            return;
        };
        let g = &self.geom;
        let e = 1.0 - (1.0 - t).powi(3);
        let c = g.disc + (g.tunnel - g.disc) * e;
        let r = g.disc_r + (g.crescent_r * 0.9 - g.disc_r) * e;
        let light = PULSE_LEVEL * (1.0 - t).powf(1.5);
        p.fill(
            &Shape::ring(c, r, (6.0 * (1.0 - t) + 1.5).max(1.0)),
            pal.primary.lerp(pal.roles.ink, 0.3).intensity(light),
        );
    }

    /// The scales: ticks, numerals and the live fill of torque (left) and power (right), with
    /// each live value under its scale.
    fn draw_scales(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let r = &self.readouts;
        let ink = static_ink(pal.roles.ink, STATIC_LUMA);
        let num_style = TextStyle::new(numerals, g.label_cap).paint(ink).tabular();
        let unit_style = TextStyle::new(self.faces.labels, g.label_cap).paint(ink).tracking(0.06);
        let sweep = g.scale_to - g.scale_from;
        for (side, full, now, shown) in [
            (-1.0_f32, r.torque_full, r.torque_now, r.torque_shown),
            (1.0, r.power_full, r.power_now, r.power_shown),
        ] {
            let (start, sw) = if side < 0.0 {
                (g.scale_from, sweep)
            } else {
                (deg(180.0) - g.scale_from, -sweep)
            };
            // The tick ring runs clockwise from its lower end on the left, from its upper end on
            // the right (the kit's ticks take a positive sweep; the pattern is symmetric).
            let tick_start = if side < 0.0 { start } else { start + sw };
            let ticks = TickRing::new(g.tunnel, g.scale_r, tick_start, sweep, SCALE_TICKS as u32)
                .minor(9.0, 2.0)
                .major(SCALE_MAJOR_EVERY as u32, 18.0, 3.0);
            p.fill(&Shape::Ticks(ticks), ink);
            // The scale's own line along the ticks' outer ends, and the live fill: a light band
            // just outside it, from zero to the value.
            p.fill(&Shape::arc(g.tunnel, g.scale_r, 1.5, start, sw, Cap::Butt), ink);
            let t = (now / full).clamp(0.0, 1.0);
            if t > 1e-3 {
                p.fill(
                    &Shape::arc(g.tunnel, g.scale_r + 9.0, 6.0, start, sw * t, Cap::Butt),
                    pal.secondary.lerp(pal.roles.ink, 0.5).intensity(0.85),
                );
            }
            let majors = SCALE_TICKS / SCALE_MAJOR_EVERY;
            for m in 0..=majors {
                let f = m as f32 / majors as f32;
                let pos = g.scale_point(f, g.scale_numeral_r, side);
                let value = (full * f).round() as i64;
                let cells = digits(value);
                p.number(pos, value, cells, &num_style, None);
            }
            // The live value (cap 34) with its unit.
            let value_style = TextStyle::new(numerals, 34.0)
                .paint(pal.roles.ink.intensity(0.86))
                .tabular();
            let pos = if side < 0.0 { g.torque_value } else { g.power_value };
            let unit = if side < 0.0 {
                torque_unit(self.drive.power_unit)
            } else {
                power_unit(self.drive.power_unit)
            };
            p.number(pos, shown, digits(shown), &value_style, None);
            p.text(
                pos + vec2(0.0, 34.0),
                unit,
                &unit_style.align(Align::Center, VAlign::CapMiddle),
            );
        }
    }

    /// The G-force widget (left) and the engine-data widget with boost (right).
    fn draw_widgets(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let ink = static_ink(pal.roles.ink, STATIC_LUMA);
        let title = TextStyle::new(self.faces.labels, g.label_cap)
            .paint(ink)
            .tracking(0.08)
            .align(Align::Center, VAlign::CapMiddle);
        let rw = g.widget_r;

        // G-force: rings at 0.75 and 1.5 g, a crosshair and the felt-force dot.
        let c = g.g_widget;
        p.text(c - vec2(0.0, rw + 30.0), "G-FORCE", &title);
        p.fill(&Shape::ring(c, rw, 2.0), pal.primary.intensity(0.55));
        p.fill(&Shape::ring(c, rw * G_INNER / G_RING, 2.0), pal.primary.intensity(0.30));
        p.fill(&Shape::line(c - vec2(rw, 0.0), c + vec2(rw, 0.0), 1.5), ink);
        p.fill(&Shape::line(c - vec2(0.0, rw), c + vec2(0.0, rw), 1.5), ink);
        let dot = c + self.drive.g * (rw / G_RING);
        p.fill(&Shape::circle(dot, 9.0), pal.primary_text);

        // Engine data: boost on a 270° gauge, the value in the middle.
        let c = g.boost_widget;
        p.text(c - vec2(0.0, rw + 30.0), "ENGINE DATA", &title);
        let track = Shape::arc(c, rw, 4.0, BOOST_START, BOOST_SWEEP, Cap::Butt);
        p.fill(&track, pal.primary.intensity(0.25));
        let span = BOOST_MAX_BAR - BOOST_MIN_BAR;
        let zero = -BOOST_MIN_BAR / span;
        let t = ((self.readouts.boost_bar - BOOST_MIN_BAR) / span).clamp(0.0, 1.0);
        let (a0, sw) = if t >= zero {
            (BOOST_START + BOOST_SWEEP * zero, BOOST_SWEEP * (t - zero))
        } else {
            (BOOST_START + BOOST_SWEEP * t, BOOST_SWEEP * (zero - t))
        };
        if sw > 1e-3 {
            p.fill(&Shape::arc(c, rw, 6.0, a0, sw, Cap::Butt), pal.primary_text);
        }
        let za = BOOST_START + BOOST_SWEEP * zero;
        let zdir = vec2(za.cos(), za.sin());
        p.fill(&Shape::line(c + zdir * (rw - 12.0), c + zdir * (rw + 8.0), 2.0), ink);
        let (value, unit, decimals) = match self.drive.speed_unit {
            SpeedUnit::Mph => (self.readouts.boost_shown, "psi", 1),
            SpeedUnit::Kmh => (self.readouts.boost_shown, "bar", 2),
        };
        let style = TextStyle::new(numerals, 32.0)
            .paint(pal.roles.ink.intensity(0.86))
            .tabular();
        draw_fixed(p, c + vec2(0.0, -6.0), value, decimals, &style);
        p.text(c + vec2(0.0, 34.0), unit, &title);
        p.text(c + vec2(0.0, rw + 30.0), "BOOST", &title);
    }

    /// The vanishing disc's soft light, brighter towards the shift point (steady, never a flash).
    fn draw_disc(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let k = (self.drive.fill / self.drive.notch.max(1e-3)).clamp(0.0, 1.0);
        let light = DISC_REST + (DISC_SHIFT - DISC_REST) * k;
        p.fill(
            &Shape::circle(g.disc, g.disc_r),
            Gradient::radial(g.disc, g.disc_r)
                .stop(0.0, pal.roles.ink.intensity(light))
                .stop(0.6, pal.roles.ink.intensity(light * 0.45))
                .stop(1.0, Color::TRANSPARENT),
        );
    }

    /// The bottom row: the logo zone and the transmission box.
    fn draw_footer(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        logo_zone(p, g.logo, &self.logo, self.faces.wide, LogoVariant::Full, pal);
        let ink = static_ink(pal.roles.ink, STATIC_LUMA);
        let box_r = g.transmission;
        p.stroke(&Shape::round_rect(box_r, 8.0), ink, Stroke::new(2.0));
        let style = TextStyle::new(self.faces.labels, 26.0)
            .paint(pal.roles.ink.intensity(0.86))
            .align(Align::Center, VAlign::CapMiddle);
        let c = box_r.center();
        p.text(c - vec2(22.0, 0.0), "M", &style.paint(pal.primary_text));
        p.text(c + vec2(18.0, 0.0), self.drive.gear.label(), &style);
    }

    /// The lit shift segments (inside the bloom layer).
    fn draw_shift(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        for i in 0..SHIFT_PER_SIDE {
            let Some(color) = self.segment_color(i, pal) else {
                continue;
            };
            for side in [-1.0, 1.0] {
                let r = g.shift_segment(i, side);
                p.fill(&Shape::round_rect(r, 2.0), color);
            }
        }
    }

    /// The unlit shift segments: a hairline along the glass's top edge.
    fn draw_unlit_shift(&self, p: &mut Painter<'_>, pal: &Palette) {
        let g = &self.geom;
        let line = static_ink(pal.roles.ink, STATIC_LUMA * 0.8);
        for i in 0..SHIFT_PER_SIDE {
            if self.segment_color(i, pal).is_some() {
                continue;
            }
            for side in [-1.0, 1.0] {
                let r = g.shift_segment(i, side);
                let y = r.center().y;
                p.fill(
                    &Shape::rect(Rect::from_points(vec2(r.min.x, y - 1.0), vec2(r.max.x, y + 1.0))),
                    line,
                );
            }
        }
    }

    /// The colour of shift segment `i` (0 = outermost), or `None` when it is unlit.
    fn segment_color(&self, i: usize, pal: &Palette) -> Option<Color> {
        let d = &self.drive;
        if d.over_shift {
            return Some(pal.shift.intensity(d.strobe_light));
        }
        (d.shift_level > i as f32 + 1e-4).then(|| {
            let k = (i as f32 + 1.0) / SHIFT_PER_SIDE as f32;
            pal.primary.intensity(0.7 + 0.3 * k)
        })
    }

    /// The speed and its unit.
    fn draw_speed(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let mut style = TextStyle::new(numerals, g.speed_cap).paint(pal.roles.ink).tabular();
        style.slant = NUMERAL_SLANT;
        let v = self.readouts.speed_shown;
        p.number(g.speed, v, digits(v), &style, None);
        let unit = TextStyle::new(self.faces.labels, g.label_cap)
            .paint(static_ink(pal.roles.ink, STATIC_LUMA))
            .tracking(0.06);
        p.text(g.speed_unit, speed_unit(self.drive.speed_unit), &unit);
    }

    /// The rpm readout in the tunnel floor.
    fn draw_rpm(&self, p: &mut Painter<'_>, pal: &Palette, numerals: FontId) {
        let g = &self.geom;
        let mut style = TextStyle::new(numerals, g.rpm_cap)
            .paint(pal.roles.ink.intensity(0.9))
            .tabular();
        style.slant = NUMERAL_SLANT;
        let v = (self.readouts.rpm_shown / 10) * 10;
        let cells = if v >= 10_000 { 5 } else { 4 };
        p.number(g.rpm, v, cells, &style, None);
        let unit = TextStyle::new(self.faces.labels, g.label_cap)
            .paint(static_ink(pal.roles.ink, STATIC_LUMA))
            .tracking(0.06);
        p.text(g.rpm_unit, "rpm", &unit);
    }
}

/// The outline of the left (`side` −1) or right crescent between fills `t0` and `t1`, into
/// `out`; returns the number of points (0 for an empty span).
fn crescent_polygon(g: &Geom, t0: f32, t1: f32, side: f32, out: &mut [Vec2; CRESCENT_SAMPLES * 2]) -> usize {
    let (t0, t1) = (t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0));
    if t1 - t0 < 1e-3 {
        return 0;
    }
    let n = CRESCENT_SAMPLES;
    for i in 0..n {
        let t = t0 + (t1 - t0) * i as f32 / (n - 1) as f32;
        out[i] = g.crescent_point(t, 0.0, side);
        out[2 * n - 1 - i] = g.crescent_point(t, 1.0, side);
    }
    2 * n
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
