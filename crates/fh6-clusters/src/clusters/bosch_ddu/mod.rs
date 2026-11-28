//! Bosch DDU (`ClusterTheme::BoschDdu`), catalog row 12: the Race Dash family's pattern-setter
//! (cockpit only).
//!
//! The Bosch Motorsport DDU 10 as BMW fits it to the M4 GT3 (`RESEARCH.md`): one black glass
//! face edge to edge in an anodised aluminium body with an arched top, large top corners and
//! small bottom ones; ten shift LEDs across the brow in groups of 2, 6 and 2, five LEDs down each
//! side, and a 7" screen with the round rev gauge in the middle, the gear in a glowing beam under
//! it and four quarter panels. The housing is the Blender render (`hardware.rs`, recipe
//! `tools/asset-forge/src/recipes/bosch_ddu.rs`) with a procedural fallback; everything lit is
//! drawn here. No Bosch marks: the chin carries the car's logo and the driver's name instead.
//! Board: Penpot page `07 · BoschDdu`.
//!
//! Zone table (local 4K px round the cockpit anchor; the hood view draws the same table at
//! [`HOOD_SCALE`]):
//!
//! | Zone | Part | Where | Holds |
//! |---|---|---|---|
//! | H | housing | x ±405, y −316..232 (arch crest −316, corners −306), top corners R 120, bottom R 44 | anodised body, black glass face, light sensor (359, 24) R 8 |
//! | L·T | top LEDs | 10 lenses 38 × 12, centres y −270 on the arch, x ±22, ±66, ±110 (6) and ±206, ±250 (2 + 2) | rank 1: shift lights, left to right through the palette's stages, then all `shift`; 10 Hz strobe on the limiter |
//! | L·S | side LEDs | 5 lenses 10 × 26 per side at x ±359, centres y −70 up to −222 (pitch 38) | left: body slip (one per 2.5°, `caution`); right: brake (fifths, `warning`) |
//! | S | screen | x ±312, y −224..150 | true black |
//! | R | rev gauge | centre (0, −59), track R 111..125 over 270° from 7:30, numerals outside at R 146 (cap 20) | rank 3: rpm arc in `primary`, `shift` over the shift point; the rpm digits (cap 48) and "rpm" in the disc |
//! | G | gear beam | trapezoid y 14..150, ±44 at the top, ±112 at the foot | rank 1: the gear (cap 104) in a beam of `primary` light |
//! | Q1 | PEDALS | x −312..−176, y −224..−90 | throttle and brake bars, 12 segments |
//! | Q2 | G-FORCE | x −312..−176, y −34..150 | combined G: ring R 40 (1.5 g) at (−244, 54), the value (cap 22) under it |
//! | Q3 | ENGINE | x 168..312, y −224..−48 | power (cap 34, `signals.power_unit`), boost, the shift rpm boxed |
//! | Q4 | SPEED | x 168..312, y −34..150 | rank 2: tabular speed (cap 52, no ghost 8s: at a glance they read as digits) and its unit |
//! | C | chin | y 150..232 | the driver's name (left), the car's logo zone (right) |
//!
//! Signature moments:
//! - **Shift lights:** the brow fills left to right in the frame each stage is crossed, green,
//!   amber, red; at the shift point all ten turn `shift`, and on the limiter the middle six strobe
//!   at 10 Hz on their own clock (the strobe's first frame is lit) while the outer pairs stay lit. The rpm digits hold the bounce's peak
//!   so no digit flickers on the limiter. Reduced flash breathes instead.
//! - **Gear change:** the token glyph motion inside the beam; the beam flares once.
//! - **Light:** lateral G swings one key light 25° across the housing (three renders,
//!   cross-faded).
//!
//! The first frame, a race-flag change and the first frame after a gap take the car's state as
//! it is. Telemetry: rpm, limiter and shift point, gear, speed, pedals, smoothed G and body slip
//! (derived from the verified velocity) from `Signals`; power and boost from the packet.
mod hardware;

use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::speed::TabularSpeed;
use crate::common::VirtualLight;
use core::f32::consts::{PI, TAU};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    vec2, Align, Cluster, Color, FontId, FontRole, FrameInput, Glow, Gradient, Hysteresis, LayerOptions, Layout, Logo,
    Painter, Palette, PowerUnit, Rect, Shape, Signals, SpeedUnit, Spring, Strobe, Stroke, TextStyle, Tokens, Tween,
    VAlign, Vec2, View,
};
use hardware::Sprites;

/// The hood view draws the cockpit table at this scale (housing 548 px tall → 373, inside the
/// hood's 400 px envelope).
pub const HOOD_SCALE: f32 = 0.68;
/// Top LEDs.
const TOP_LEDS: usize = 10;
/// The outer LEDs at each end of the brow (the 2 of the 2-6-2 groups) stay steady on the
/// limiter while the middle six strobe.
const OUTER: usize = 2;
/// LEDs in each side column.
const SIDE_LEDS: usize = 5;
/// Body slip per side LED (degrees), and the fall back a lit LED needs before it goes out.
const SLIP_STEP_DEG: f32 = 2.5;
const SLIP_HYST_DEG: f32 = 0.5;
/// Brake below this is off (pedal noise).
const BRAKE_DEAD: f32 = 0.02;
/// The shift lights' release time constant (s; tokens: release <= 50 ms).
const RELEASE_TAU: f32 = 0.025;
/// Once over the shift point the lights stay over until the rpm falls this far under it
/// (tokens `hysteresisRpm`).
const SHIFT_HYSTERESIS_RPM: f32 = 120.0;
/// A frame at least this long (s) follows a gap in updates: take the car's state as it is.
const RESUME_DT: f32 = 0.09;
/// A cluster built mid-race settles the gear glyph's entry in this many steps before its first
/// draw.
const GEAR_SETTLE_STEPS: u32 = 30;
const GEAR_SETTLE_DT: f32 = 1.0 / 60.0;
/// While over the shift point the rpm digits show the highest rpm of the last this-many seconds
/// (a limiter bounce is 50 to 150 ms), so no digit flickers.
const PEAK_HOLD_S: f32 = 0.3;
/// The small engine readouts refresh at this rate (s), as a race display's do.
const READOUT_S: f32 = 0.2;
/// G at the ring's edge.
const G_LIMIT: f32 = 1.5;
/// The beam's flare on a gear change (s, linear fade) and its peak.
const FLARE_S: f32 = 0.25;
const FLARE_PEAK: f32 = 0.6;
/// The beam's light at its foot (alpha of `primary`) and its two edge lines.
const BEAM_FOOT: f32 = 0.26;
const BEAM_EDGE: f32 = 0.55;
/// Lit LED bloom (sigma, intensity): tight, so the strobe never spills far.
const LED_GLOW: (f32, f32) = (5.0, 0.45);
/// An unlit LED: a smoked lens over a dark emitter (linear level).
const LED_OFF: f32 = 0.018;
/// The accent light pooled low in the rev gauge's disc (alpha of `primary`).
const DISC_POOL: f32 = 0.16;
/// The lit arc's brightness at its start (its head is at full strength).
const ARC_TAIL: f32 = 0.5;
/// How far the pointer at the arc's head reaches into the disc (px): not at all, so it never
/// touches the rpm digits.
const POINTER_IN: f32 = 0.0;
/// The red zone's width on the rev gauge's inner edge (px).
const RED_BAND: f32 = 6.0;
/// The rev gauge's unlit track (alpha of `ink`).
const TRACK_LEVEL: f32 = 0.10;
/// The printed parts (alpha of `ink`): numerals, labels and rules. Numerals and labels sit at
/// about 0.19 relative luminance, so they keep 4.5:1 over black and stay under the static cap
/// (0.2), dimmer than every live value.
const LEVEL_NUMERAL: f32 = 0.195;
const LEVEL_LABEL: f32 = 0.195;
const LEVEL_RULE: f32 = 0.14;
/// The chin's logo zone draws at this share of its own ink (it is printed, like the labels).
const LOGO_LEVEL: f32 = 0.44;

/// Numerals 0 to 20 (thousands), so drawing never formats a string.
const NUMERALS: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
];

/// The zone table as data (cockpit, local 4K px round the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Geom {
    half_w: f32,
    /// The top corners' y (the arch's crest is `arch` above it) and the bottom edge.
    top: f32,
    bottom: f32,
    arch: f32,
    corner_top: f32,
    corner_bottom: f32,
    sensor: Vec2,
    sensor_r: f32,
    screen: Rect,
    top_led_x: [f32; TOP_LEDS],
    top_led_y: f32,
    top_led: Vec2,
    side_led_x: f32,
    side_led_y0: f32,
    side_led_pitch: f32,
    side_led: Vec2,
    rev_c: Vec2,
    rev_r: f32,
    rev_w: f32,
    rev_start: f32,
    rev_sweep: f32,
    num_r: f32,
    num_cap: f32,
    rpm_cap: f32,
    rpm_at: Vec2,
    beam_top: f32,
    beam_half: (f32, f32),
    gear_at: Vec2,
    gear_cap: f32,
    /// Quarter panels: left (x0, x1), right (x0, x1), top row's header y and bottom row's.
    left: (f32, f32),
    right: (f32, f32),
    head_top: f32,
    head_bottom: f32,
    head_cap: f32,
    label_cap: f32,
    bar_h: f32,
    thr_y: f32,
    brk_y: f32,
    g_c: Vec2,
    g_r: f32,
    g_value_y: f32,
    g_value_cap: f32,
    power_y: f32,
    power_cap: f32,
    boost_y: f32,
    value_cap: f32,
    shift_box: Rect,
    speed_at: Vec2,
    speed_cap: f32,
    unit_y: f32,
    name_at: Vec2,
    name_cap: f32,
    logo: Rect,
}

const GEOM: Geom = Geom {
    half_w: 405.0,
    top: -306.0,
    bottom: 232.0,
    arch: 10.0,
    corner_top: 120.0,
    corner_bottom: 44.0,
    sensor: Vec2 { x: 359.0, y: 24.0 },
    sensor_r: 8.0,
    screen: Rect {
        min: Vec2 { x: -312.0, y: -224.0 },
        max: Vec2 { x: 312.0, y: 150.0 },
    },
    top_led_x: [-250.0, -206.0, -110.0, -66.0, -22.0, 22.0, 66.0, 110.0, 206.0, 250.0],
    top_led_y: -270.0,
    top_led: Vec2 { x: 38.0, y: 12.0 },
    side_led_x: 359.0,
    side_led_y0: -70.0,
    side_led_pitch: 38.0,
    side_led: Vec2 { x: 10.0, y: 26.0 },
    rev_c: Vec2 { x: 0.0, y: -59.0 },
    rev_r: 118.0,
    rev_w: 14.0,
    rev_start: 0.75 * PI,
    rev_sweep: 1.5 * PI,
    num_r: 146.0,
    num_cap: 20.0,
    rpm_cap: 48.0,
    rpm_at: Vec2 { x: 0.0, y: -76.0 },
    beam_top: 14.0,
    beam_half: (44.0, 112.0),
    gear_at: Vec2 { x: 0.0, y: 84.0 },
    gear_cap: 104.0,
    left: (-312.0, -176.0),
    right: (168.0, 312.0),
    head_top: -208.0,
    head_bottom: -26.0,
    head_cap: 15.0,
    label_cap: 13.0,
    bar_h: 14.0,
    thr_y: -160.0,
    brk_y: -112.0,
    g_c: Vec2 { x: -244.0, y: 54.0 },
    g_r: 40.0,
    g_value_y: 122.0,
    g_value_cap: 22.0,
    power_y: -142.0,
    power_cap: 34.0,
    boost_y: -104.0,
    value_cap: 18.0,
    shift_box: Rect {
        min: Vec2 { x: 168.0, y: -84.0 },
        max: Vec2 { x: 312.0, y: -52.0 },
    },
    speed_at: Vec2 { x: 244.0, y: 48.0 },
    speed_cap: 52.0,
    unit_y: 86.0,
    name_at: Vec2 { x: -318.0, y: 192.0 },
    name_cap: 16.0,
    logo: Rect {
        min: Vec2 { x: 150.0, y: 170.0 },
        max: Vec2 { x: 330.0, y: 214.0 },
    },
};

impl Geom {
    /// The housing's body below the arch.
    fn body(&self) -> Rect {
        Rect::from_points(vec2(-self.half_w, self.top), vec2(self.half_w, self.bottom))
    }

    /// The whole housing, arch included.
    #[cfg(test)]
    fn housing(&self) -> Rect {
        Rect::from_points(vec2(-self.half_w, self.top - self.arch), vec2(self.half_w, self.bottom))
    }

    /// How far the arch drops below its crest at `x` (0 at the middle).
    fn arch_drop(&self, x: f32) -> f32 {
        let span = self.half_w - self.corner_top;
        let r = (span * span + self.arch * self.arch) / (2.0 * self.arch);
        r - (r * r - x * x).max(0.0).sqrt()
    }

    /// Top LED `i`'s lens, following the arch.
    fn top_led_rect(&self, i: usize) -> Rect {
        let x = self.top_led_x[i];
        Rect::from_center_size(vec2(x, self.top_led_y + self.arch_drop(x)), self.top_led)
    }

    /// Side LED `i` (0 at the bottom) of the left (`side` −1) or right (+1) column.
    fn side_led_rect(&self, side: f32, i: usize) -> Rect {
        let c = vec2(
            side * self.side_led_x,
            self.side_led_y0 - self.side_led_pitch * i as f32,
        );
        Rect::from_center_size(c, self.side_led)
    }

    /// The gear beam's corners: top left, top right, foot right, foot left.
    fn beam(&self) -> [Vec2; 4] {
        let (t, b) = self.beam_half;
        let foot = self.screen.max.y;
        [
            vec2(-t, self.beam_top),
            vec2(t, self.beam_top),
            vec2(b, foot),
            vec2(-b, foot),
        ]
    }
}

/// The faces this cluster sets (a user display font replaces the figures).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    figures: FontId,
    label: FontId,
    wide: FontId,
    role_display: FontId,
}

/// The ten brow LEDs: they fill left to right through the palette's stages (0 ms attack,
/// 25 ms release), turn `shift` together at the shift point and strobe on the limiter.
#[derive(Debug, Clone, Copy, PartialEq)]
struct TopLeds {
    /// Share of the strip lit (0..=1).
    level: f32,
    over: bool,
    /// The strobe's own clock (s): from the frame the shift point is reached.
    strobe_t: f32,
    strobe_on: bool,
    breath: f32,
}

impl TopLeds {
    fn new() -> Self {
        Self {
            level: 0.0,
            over: false,
            strobe_t: 0.0,
            strobe_on: true,
            breath: 1.0,
        }
    }

    /// Feeds the rpm. A car that sends no limiter or shift point leaves the strip dark.
    fn update(&mut self, sig: &Signals, pal: &Palette, dt: f32, calm: bool, snap: bool) {
        let known = sig.max_rpm.is_finite() && sig.max_rpm > 500.0 && sig.shift_rpm.is_finite() && sig.shift_rpm > 0.0;
        let rpm = if sig.rpm.is_finite() { sig.rpm.max(0.0) } else { 0.0 };
        let frac = if known { rpm / sig.shift_rpm } else { 0.0 };
        let over = known
            && if self.over {
                rpm >= sig.shift_rpm - SHIFT_HYSTERESIS_RPM
            } else {
                frac >= 1.0
            };
        self.strobe_t = if over && self.over { self.strobe_t + dt } else { 0.0 };
        self.over = over;
        let start = pal.stages[0];
        let target = if over {
            1.0
        } else {
            ((frac - start) / (1.0 - start).max(1e-3)).clamp(0.0, 1.0)
        };
        if target >= self.level || snap {
            self.level = target;
        } else {
            let k = 1.0 - (-dt.max(0.0) / RELEASE_TAU).exp();
            self.level += (target - self.level) * k;
            if self.level < 1e-3 {
                self.level = 0.0;
            }
        }
        if calm {
            self.strobe_on = true;
            self.breath = Strobe::breath(self.strobe_t);
        } else {
            self.strobe_on = Strobe::LIMITER.on(self.strobe_t);
            self.breath = 1.0;
        }
    }

    /// The colour of LED `i` (0 at the left), or `None` when it is dark or strobed off.
    fn color_of(&self, i: usize, pal: &Palette) -> Option<Color> {
        let rank = (i + 1) as f32 / TOP_LEDS as f32;
        if self.level <= 0.0 || self.level < rank - 1e-4 {
            return None;
        }
        if self.over {
            // The outer pairs stay lit, so the strobe's dark phase never reads as "no shift";
            // the middle six strobe.
            if !(OUTER..TOP_LEDS - OUTER).contains(&i) {
                return Some(pal.shift);
            }
            return self.strobe_on.then(|| pal.shift.fade(self.breath));
        }
        let start = pal.stages[0];
        Some(pal.rev_stage(start + rank * (1.0 - start) - 1e-4))
    }

    fn animating(&self) -> bool {
        self.over || (self.level > 0.0 && self.level < 1.0)
    }
}

/// A small readout that refreshes every [`READOUT_S`], as a race display's do.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Readout {
    shown: i64,
    timer: f32,
    fresh: bool,
}

impl Readout {
    fn new() -> Self {
        Self {
            shown: 0,
            timer: 0.0,
            fresh: true,
        }
    }

    /// Feeds `value` in display steps (already scaled: tenths for one decimal).
    fn update(&mut self, value: f32, dt: f32) {
        let v = if value.is_finite() { value.round() as i64 } else { 0 };
        self.timer += dt.max(0.0);
        if self.fresh || self.timer >= READOUT_S {
            self.shown = v;
            self.timer = 0.0;
            self.fresh = false;
        }
    }
}

/// Writes `value / 10^decimals` with `decimals` places into `buf` (no allocation).
fn fixed(buf: &mut [u8; 16], value: i64, decimals: u32) -> &str {
    let neg = value < 0;
    let mut v = value.unsigned_abs();
    let mut i = buf.len();
    let mut digits = 0;
    loop {
        if decimals > 0 && digits == decimals {
            i -= 1;
            buf[i] = b'.';
        }
        i -= 1;
        buf[i] = b'0' + (v % 10) as u8;
        v /= 10;
        digits += 1;
        if v == 0 && digits > decimals {
            break;
        }
        if i < 3 {
            break;
        }
    }
    if neg {
        i -= 1;
        buf[i] = b'-';
    }
    core::str::from_utf8(&buf[i..]).unwrap_or("")
}

/// The Bosch DDU cluster.
#[derive(Debug, Clone)]
pub struct BoschDdu {
    faces: Faces,
    own_faces: Faces,
    sprites: Sprites,
    fresh: bool,
    sig: Signals,
    palette: Palette,
    view: View,
    calm: bool,
    logo: Logo,
    driver: String,
    leds: TopLeds,
    /// Side columns lit: left (slip), right (brake).
    slip_lit: usize,
    brake_lit: usize,
    gear: HeroGear,
    flare: Tween,
    speed: TabularSpeed,
    rpm_digits: Hysteresis,
    rpm_shown: i64,
    /// The rpm the gauge's arc shows: the rpm, or the bounce's peak over the shift point.
    arc_rpm: f32,
    peak: f32,
    peak_t: f32,
    g: [Spring; 2],
    g_target: [f32; 2],
    g_digits: Readout,
    power: Readout,
    boost: Readout,
    light: VirtualLight,
    tracking: f32,
}

impl BoschDdu {
    /// Builds the cluster: faces, springs and the rendered housing (with a procedural fallback).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let sprites = Sprites::load(setup);
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let figures = setup
            .fonts
            .find("Saira Semi Condensed", 600, false)
            .or_else(|| setup.fonts.find("Saira Condensed", 600, false))
            .unwrap_or(display);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
            figures,
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let palette = Palette::family(t, "raceDash");
        Self {
            faces,
            own_faces: faces,
            sprites,
            fresh: true,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            calm: false,
            logo: Logo::default(),
            driver: String::new(),
            leds: TopLeds::new(),
            slip_lit: 0,
            brake_lit: 0,
            gear: HeroGear::new(t),
            flare: Tween::new(FLARE_S),
            speed: TabularSpeed::new(),
            rpm_digits: Hysteresis::new(0.0, 3.0),
            rpm_shown: 0,
            arc_rpm: 0.0,
            peak: 0.0,
            peak_t: 0.0,
            g: [Spring::new(t.springs.default, 0.0), Spring::new(t.springs.default, 0.0)],
            g_target: [0.0; 2],
            g_digits: Readout::new(),
            power: Readout::new(),
            boost: Readout::new(),
            light: VirtualLight::new(t),
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The scale's top (rpm): the limiter rounded up to a thousand (8,000 when none is known).
    fn scale_top(&self) -> f32 {
        let max = self.sig.max_rpm;
        if max.is_finite() && max > 500.0 {
            (max / 1000.0).ceil().clamp(1.0, 20.0) * 1000.0
        } else {
            8000.0
        }
    }

    /// The angle of `rpm` on the rev gauge.
    fn rev_angle(&self, g: &Geom, rpm: f32) -> f32 {
        g.rev_start + g.rev_sweep * (rpm / self.scale_top()).clamp(0.0, 1.0)
    }
}

/// The side column count for `value` in steps of `step`, latched so a value sitting on a step
/// never flickers: it rises at a step and falls only `hyst` under it.
fn latched(lit: usize, value: f32, step: f32, hyst: f32, max: usize) -> usize {
    let v = if value.is_finite() { value.abs() } else { 0.0 };
    let up = ((v / step).floor() as usize).min(max);
    if up >= lit {
        up
    } else {
        (((v + hyst) / step).floor() as usize).min(lit)
    }
}

impl Cluster for BoschDdu {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        let race_edge = sig.race_on != self.sig.race_on;
        let snap = self.fresh || race_edge || input.dt >= RESUME_DT;
        self.fresh = false;
        self.sig = sig;
        self.palette = *input.palette;
        self.calm = input.reduced_flash;
        self.view = input.view;
        if input.fonts.display != self.faces.role_display {
            self.faces = if input.fonts.display == self.own_faces.role_display {
                self.own_faces
            } else {
                Faces {
                    gear: input.fonts.display,
                    figures: input.fonts.display,
                    role_display: input.fonts.display,
                    ..self.own_faces
                }
            };
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }
        if self.driver != input.config.custom_driver_name {
            self.driver.clone_from(&input.config.custom_driver_name);
        }

        // The gear first. Snapped frames show the engaged gear at rest.
        if snap {
            for _ in 0..GEAR_SETTLE_STEPS {
                self.gear.update(sig.gear, 0.0, GEAR_SETTLE_DT);
            }
            self.flare = Tween::new(FLARE_S);
        } else {
            let changed = sig.gear != self.gear.gear();
            self.gear.update(sig.gear, 0.0, dt);
            if changed && !self.calm {
                self.flare.restart();
            }
        }
        self.flare.update(dt);

        // Shift lights and the side columns.
        self.leds.update(&sig, &self.palette, dt, self.calm, snap);
        self.slip_lit = latched(self.slip_lit, sig.slip_deg, SLIP_STEP_DEG, SLIP_HYST_DEG, SIDE_LEDS);
        let brake = if sig.brake.is_finite() {
            sig.brake.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.brake_lit = if brake < BRAKE_DEAD {
            0
        } else {
            ((brake * SIDE_LEDS as f32).ceil() as usize).min(SIDE_LEDS)
        };

        // The rpm digits: 10 rpm steps with 30 rpm of hysteresis; over the shift point they hold
        // the bounce's peak.
        let rpm = if sig.rpm.is_finite() { sig.rpm.max(0.0) } else { 0.0 };
        if snap || rpm >= self.peak || self.peak_t >= PEAK_HOLD_S {
            self.peak = rpm;
            self.peak_t = 0.0;
        } else {
            self.peak_t += dt;
        }
        let digits = if self.leds.over { self.peak } else { rpm };
        self.arc_rpm = digits;
        if snap {
            self.rpm_digits = Hysteresis::new(digits / 10.0, 3.0);
        }
        self.rpm_shown = self.rpm_digits.update(digits / 10.0) * 10;

        // Speed, G (felt force), power and boost.
        self.speed.update(sig.speed);
        let finite = |v: f32| if v.is_finite() { v } else { 0.0 };
        let felt = vec2(-finite(sig.g_lat), finite(sig.g_long)) * (1.0 / G_LIMIT);
        let reach = felt.length();
        let felt = if reach > 1.0 { felt * (1.0 / reach) } else { felt };
        self.g_target = [felt.x, felt.y];
        for (axis, target) in self.g.iter_mut().zip(self.g_target) {
            if snap {
                axis.reset(target);
            }
            axis.update(target, dt);
        }
        let g_mag = (finite(sig.g_lat).powi(2) + finite(sig.g_long).powi(2)).sqrt();
        let watts = finite(input.telemetry.power_watts).max(0.0);
        let power = match sig.power_unit {
            PowerUnit::Hp => watts / 745.7,
            PowerUnit::Kw => watts / 1000.0,
        };
        let psi = finite(input.telemetry.boost_psi).clamp(-15.0, 60.0);
        let boost = match sig.unit {
            SpeedUnit::Mph => psi * 10.0,
            SpeedUnit::Kmh => psi * 0.068_947_6 * 100.0,
        };
        if snap {
            self.g_digits = Readout::new();
            self.power = Readout::new();
            self.boost = Readout::new();
        }
        self.g_digits.update(g_mag * 100.0, dt);
        self.power.update(power, dt);
        self.boost.update(boost, dt);

        self.light.update(sig.g_lat, dt);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &GEOM;
        let pal = self.palette;
        p.save();
        if self.view == View::Hood {
            p.scale(HOOD_SCALE);
        }
        let w = hardware::light_weights(self.light.angle());
        if !self.sprites.draw(p, w) {
            hardware::fallback(p, g, &pal, w[2] - w[1]);
        }
        self.draw_leds(p, g, &pal);
        self.draw_rev(p, g, &pal);
        self.draw_gear(p, g, &pal);
        self.draw_pedals(p, g, &pal);
        self.draw_g(p, g, &pal);
        self.draw_engine(p, g, &pal);
        self.draw_speed(p, g, &pal);
        self.draw_chin(p, g, &pal);
        p.restore();
    }

    fn animating(&self) -> bool {
        self.leds.animating()
            || self.gear.animating()
            || self.flare.running()
            || !self.g[0].settled(self.g_target[0], 0.002)
            || !self.g[1].settled(self.g_target[1], 0.002)
    }
}

impl BoschDdu {
    fn label(&self, cap: f32, ink: Color, align: Align) -> TextStyle {
        TextStyle::new(self.faces.label, cap)
            .paint(ink)
            .tracking(self.tracking)
            .align(align, VAlign::CapMiddle)
    }

    /// A row label inside a panel: no tracking, so label and value share the narrow panel.
    fn row_label(&self, cap: f32, ink: Color) -> TextStyle {
        TextStyle::new(self.faces.label, cap)
            .paint(ink)
            .align(Align::Left, VAlign::CapMiddle)
    }

    fn figures(&self, cap: f32, ink: Color, align: Align) -> TextStyle {
        TextStyle::new(self.faces.figures, cap)
            .paint(ink)
            .align(align, VAlign::CapMiddle)
            .tabular()
    }

    /// One LED lens: lit with a tight bloom and a hot core, or a smoked dark lens (left to the
    /// housing render when it loaded: its lenses are the real hardware).
    fn led(&self, p: &mut Painter<'_>, rect: Rect, lit: Option<Color>) {
        let r = rect.width().min(rect.height()) * 0.5;
        let shape = Shape::round_rect(rect, r);
        match lit {
            Some(color) => {
                p.glow(&shape, color, Glow::new(LED_GLOW.0, LED_GLOW.1));
                p.layer(LayerOptions::bloom(), |p| p.fill(&shape, color));
                let core = Rect::from_center_size(rect.center(), rect.size() * 0.5);
                let cr = core.width().min(core.height()) * 0.5;
                p.fill(&Shape::round_rect(core, cr), color.lerp(Color::WHITE, 0.45));
            }
            None if self.sprites.loaded() => {}
            None => {
                p.fill(&shape, Color::linear(LED_OFF, LED_OFF, LED_OFF * 1.1, 1.0));
                p.stroke(&shape, Color::WHITE.with_alpha(0.05), Stroke::inner(1.0));
            }
        }
    }

    /// The brow's shift lights and the two side columns.
    fn draw_leds(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        for i in 0..TOP_LEDS {
            self.led(p, g.top_led_rect(i), self.leds.color_of(i, pal));
        }
        for i in 0..SIDE_LEDS {
            // Slip is a caution, never red: red is kept for the limiter and the brake.
            let slip = (i < self.slip_lit).then_some(pal.roles.caution);
            self.led(p, g.side_led_rect(-1.0, i), slip);
            let brake = (i < self.brake_lit).then_some(pal.roles.warning);
            self.led(p, g.side_led_rect(1.0, i), brake);
        }
    }

    /// The round rev gauge: track, lit arc, thousand ticks, numerals outside and the digits.
    fn draw_rev(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let c = g.rev_c;
        let ink = pal.roles.ink;
        let top = self.scale_top();
        p.fill(
            &Shape::arc(c, g.rev_r, g.rev_w, g.rev_start, g.rev_sweep, fh6_kit::Cap::Butt),
            ink.with_alpha(TRACK_LEVEL),
        );
        // The red zone from the shift point to the scale's end, on the track's inner edge.
        let max = self.sig.max_rpm;
        let shift = self.sig.shift_rpm;
        if shift > 500.0 && shift < top {
            let a0 = self.rev_angle(g, shift);
            p.fill(
                &Shape::arc(
                    c,
                    g.rev_r - g.rev_w * 0.5 - 5.0,
                    RED_BAND,
                    a0,
                    self.rev_angle(g, top) - a0,
                    fh6_kit::Cap::Butt,
                ),
                pal.roles.warning,
            );
        }
        // The dial's glass disc: a faint pool of the accent light low in the disc and a thin
        // inner ring, so the gauge has depth without a panel fill.
        let inner = g.rev_r - g.rev_w * 0.5 - 12.0;
        let pool = Gradient::radial(vec2(c.x, c.y + inner * 0.55), inner * 1.25)
            .stop(0.0, pal.primary.with_alpha(DISC_POOL))
            .stop(1.0, pal.primary.with_alpha(0.0));
        p.fill(&Shape::circle(c, inner), pool);
        p.stroke(&Shape::circle(c, inner), ink.with_alpha(LEVEL_RULE), Stroke::inner(1.5));
        let rpm = self.arc_rpm;
        if rpm > 0.0 {
            let lit = if self.leds.over { pal.shift } else { pal.primary };
            let head = self.rev_angle(g, rpm);
            let sweep = head - g.rev_start;
            // The arc brightens towards its head, as the DDU's does.
            let ramp = Gradient::conic(c, g.rev_start)
                .stop(0.0, lit.fade(ARC_TAIL))
                .stop((sweep / TAU).clamp(0.001, 1.0), lit)
                .stop(1.0, lit);
            let arc = Shape::arc(c, g.rev_r, g.rev_w, g.rev_start, sweep, fh6_kit::Cap::Butt);
            p.layer(LayerOptions::bloom(), |p| p.fill(&arc, ramp));
            // The pointer: a white line across the track at the arc's head, reaching into the disc.
            p.fill(
                &Shape::line(
                    Vec2::polar(c, inner - POINTER_IN, head),
                    Vec2::polar(c, g.rev_r + g.rev_w * 0.5, head),
                    3.0,
                ),
                ink,
            );
        }
        // Ticks every thousand across the track's outer edge, numerals outside.
        let thousands = (top / 1000.0).round() as usize;
        let step = if thousands > 10 { 2 } else { 1 };
        let style = self.figures(g.num_cap, ink.with_alpha(LEVEL_NUMERAL), Align::Center);
        let outer = g.rev_r + g.rev_w * 0.5;
        for k in 0..=thousands {
            let a = self.rev_angle(g, k as f32 * 1000.0);
            let red = max.is_finite() && max > 500.0 && k as f32 * 1000.0 >= max - 1.0;
            let tick = if red {
                pal.roles.warning
            } else {
                ink.with_alpha(LEVEL_NUMERAL)
            };
            p.fill(
                &Shape::line(Vec2::polar(c, outer + 2.0, a), Vec2::polar(c, outer + 9.0, a), 2.0),
                tick,
            );
            if k < thousands {
                let half = self.rev_angle(g, k as f32 * 1000.0 + 500.0);
                p.fill(
                    &Shape::line(
                        Vec2::polar(c, outer + 2.0, half),
                        Vec2::polar(c, outer + 6.0, half),
                        1.5,
                    ),
                    ink.with_alpha(LEVEL_RULE),
                );
            }
            if k % step == 0 {
                if let Some(label) = NUMERALS.get(k) {
                    p.text(Vec2::polar(c, g.num_r, a), label, &style);
                }
            }
        }
        // The rpm digits and their unit in the disc.
        let cells = if top > 9999.0 { 5 } else { 4 };
        let digits = self.figures(g.rpm_cap, ink, Align::Center);
        p.number(g.rpm_at, self.rpm_shown, cells, &digits, None);
        let unit = self.label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Center);
        p.text(vec2(g.rpm_at.x, g.rpm_at.y + g.rpm_cap * 0.5 + 22.0), "rpm", &unit);
    }

    /// The gear in its beam of light.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let beam = g.beam();
        let flare = if self.flare.running() {
            FLARE_PEAK * (1.0 - self.flare.progress())
        } else {
            0.0
        };
        let foot = (BEAM_FOOT * (1.0 + flare)).min(1.0);
        let light = Gradient::linear(vec2(0.0, g.beam_top), vec2(0.0, g.screen.max.y))
            .stop(0.0, pal.primary.with_alpha(0.0))
            .stop(0.55, pal.primary.with_alpha(foot * 0.35))
            .stop(1.0, pal.primary.with_alpha(foot));
        p.fill(&Shape::Polygon { points: &beam }, light);
        let edge = Gradient::linear(vec2(0.0, g.beam_top), vec2(0.0, g.screen.max.y))
            .stop(0.0, pal.primary.with_alpha(0.0))
            .stop(1.0, pal.primary.with_alpha(BEAM_EDGE));
        p.fill(&Shape::line(beam[0], beam[3], 2.0), edge);
        p.fill(&Shape::line(beam[1], beam[2], 2.0), edge);
        p.layer(LayerOptions::bloom(), |p| {
            self.gear.draw(p, g.gear_at, self.faces.gear, g.gear_cap, pal);
        });
    }

    /// A quarter panel's caps header over a rule whose inner end turns down at 45°.
    fn header(
        &self,
        p: &mut Painter<'_>,
        g: &Geom,
        x: (f32, f32),
        y: f32,
        text: &str,
        pal: &Palette,
        inner_right: bool,
    ) {
        let style = self.label(
            g.head_cap,
            pal.roles.ink.with_alpha(LEVEL_LABEL),
            if inner_right { Align::Left } else { Align::Right },
        );
        let at = if inner_right { x.0 } else { x.1 };
        p.text(vec2(at, y), text, &style);
        let ry = y + g.head_cap * 0.5 + 8.0;
        let rule = pal.roles.ink.with_alpha(LEVEL_RULE);
        let kick = 8.0;
        let (outer, inner) = if inner_right { (x.0, x.1) } else { (x.1, x.0) };
        let dir = if inner_right { 1.0 } else { -1.0 };
        let bend = inner - dir * kick;
        p.fill(&Shape::line(vec2(outer, ry), vec2(bend, ry), 2.0), rule);
        p.fill(&Shape::line(vec2(bend, ry), vec2(inner, ry + kick), 2.0), rule);
    }

    /// A horizontal bar of 12 segments, filled continuously to `value` (0..=1).
    fn bar(p: &mut Painter<'_>, x: (f32, f32), y: f32, h: f32, value: f32, lit: Color, pal: &Palette) {
        const SEGMENTS: usize = 12;
        const GAP: f32 = 3.0;
        let w = (x.1 - x.0 - GAP * (SEGMENTS as f32 - 1.0)) / SEGMENTS as f32;
        let v = if value.is_finite() { value.clamp(0.0, 1.0) } else { 0.0 } * SEGMENTS as f32;
        for i in 0..SEGMENTS {
            let x0 = x.0 + i as f32 * (w + GAP);
            let seg = Rect::from_xywh(x0, y, w, h);
            p.fill(&Shape::rect(seg), pal.roles.ink.with_alpha(TRACK_LEVEL));
            let f = (v - i as f32).clamp(0.0, 1.0);
            if f > 0.0 {
                p.fill(&Shape::rect(Rect::from_xywh(x0, y, w * f, h)), lit);
            }
        }
    }

    /// PEDALS: throttle and brake.
    fn draw_pedals(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.header(p, g, g.left, g.head_top, "PEDALS", pal, false);
        let label = self.label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Left);
        p.text(vec2(g.left.0, g.thr_y - 14.0), "THR", &label);
        Self::bar(p, g.left, g.thr_y, g.bar_h, self.sig.throttle, pal.primary, pal);
        p.text(vec2(g.left.0, g.brk_y - 14.0), "BRK", &label);
        Self::bar(p, g.left, g.brk_y, g.bar_h, self.sig.brake, pal.roles.warning, pal);
    }

    /// G-FORCE: the felt force as a dot in a 1.5 g ring, and its size.
    fn draw_g(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.header(p, g, g.left, g.head_bottom, "G-FORCE", pal, false);
        let c = g.g_c;
        let rule = pal.roles.ink.with_alpha(LEVEL_RULE);
        p.stroke(&Shape::circle(c, g.g_r), rule, Stroke::new(2.0));
        p.stroke(
            &Shape::circle(c, g.g_r / G_LIMIT),
            pal.roles.ink.with_alpha(TRACK_LEVEL),
            Stroke::new(1.5),
        );
        p.fill(
            &Shape::line(vec2(c.x - g.g_r, c.y), vec2(c.x + g.g_r, c.y), 1.0),
            pal.roles.ink.with_alpha(TRACK_LEVEL),
        );
        p.fill(
            &Shape::line(vec2(c.x, c.y - g.g_r), vec2(c.x, c.y + g.g_r), 1.0),
            pal.roles.ink.with_alpha(TRACK_LEVEL),
        );
        let dot = c + vec2(self.g[0].value(), self.g[1].value()) * (g.g_r - 6.0);
        p.layer(LayerOptions::bloom(), |p| p.fill(&Shape::circle(dot, 6.0), pal.primary));
        let mut buf = [0u8; 16];
        let text = fixed(&mut buf, self.g_digits.shown, 2);
        let value = self.figures(g.g_value_cap, pal.roles.ink, Align::Right);
        p.text(vec2(c.x + 18.0, g.g_value_y), text, &value);
        let unit = self.label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Left);
        p.text(vec2(c.x + 24.0, g.g_value_y + 2.0), "g", &unit);
    }

    /// ENGINE: power in the user's unit, boost, and the shift point boxed.
    fn draw_engine(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let (x0, x1) = g.right;
        self.header(p, g, g.right, g.head_top, "ENGINE", pal, true);
        let label = self.label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Left);
        let unit = self.label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Right);
        let ly = g.power_y - g.power_cap * 0.5 - 12.0;
        p.text(vec2(x0, ly), "POWER", &label);
        p.text(
            vec2(x1, ly),
            match self.sig.power_unit {
                PowerUnit::Hp => "hp",
                PowerUnit::Kw => "kW",
            },
            &unit,
        );
        let big = self.figures(g.power_cap, pal.roles.ink, Align::Right);
        p.number(vec2(x1, g.power_y), self.power.shown.clamp(0, 9999), 4, &big, None);

        // Boost: the label and its unit on the left, the value on the right (vacuum reads 0).
        let row = self.row_label(g.label_cap, pal.roles.ink.with_alpha(LEVEL_LABEL));
        let label_w = p.measure("BOOST", &row).width;
        p.text(vec2(x0, g.boost_y), "BOOST", &row);
        let (decimals, boost_unit) = match self.sig.unit {
            SpeedUnit::Mph => (1, "psi"),
            SpeedUnit::Kmh => (2, "bar"),
        };
        p.text(vec2(x0 + label_w + 6.0, g.boost_y), boost_unit, &row);
        let small = self.figures(g.value_cap, pal.roles.ink, Align::Right);
        let mut buf = [0u8; 16];
        p.text(
            vec2(x1, g.boost_y),
            fixed(&mut buf, self.boost.shown.max(0), decimals),
            &small,
        );

        // The shift point, boxed in the accent colour as the DDU boxes its key value.
        let b = g.shift_box;
        p.stroke(
            &Shape::round_rect(b, 4.0),
            pal.primary.with_alpha(0.8),
            Stroke::inner(2.0),
        );
        let cy = b.center().y;
        p.text(vec2(b.min.x + 8.0, cy), "SHIFT", &row);
        let shift = if self.sig.shift_rpm.is_finite() {
            ((self.sig.shift_rpm / 10.0).round() as i64 * 10).clamp(0, 99_999)
        } else {
            0
        };
        let cells = if shift > 9999 { 5 } else { 4 };
        p.number(vec2(b.max.x - 8.0, cy), shift, cells, &small, None);
    }

    /// SPEED: tabular digits (no ghost 8s: at a glance a ghost read as a digit), and the unit in
    /// lower case.
    fn draw_speed(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.header(p, g, g.right, g.head_bottom, "SPEED", pal, true);
        // Flush right, as every value on this side is; the unit sits under the last digit.
        let digits = self.figures(g.speed_cap, pal.roles.ink, Align::Right);
        p.number(vec2(g.right.1, g.speed_at.y), self.speed.value(), 3, &digits, None);
        let unit = self.label(g.label_cap + 2.0, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Right);
        let text = match self.sig.unit {
            SpeedUnit::Mph => "mph",
            SpeedUnit::Kmh => "km/h",
        };
        p.text(vec2(g.right.1, g.unit_y), text, &unit);
    }

    /// The chin: the driver's name on the left, the car's logo zone on the right.
    fn draw_chin(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let name = self.driver.trim();
        // Without a logo image or brand the logo zone shows the custom name: never twice.
        let zone_text = if self.logo.image.is_some() {
            ""
        } else if !self.logo.brand_name.trim().is_empty() {
            self.logo.brand_name.trim()
        } else {
            self.logo.custom_name.trim()
        };
        if !name.is_empty() && !name.eq_ignore_ascii_case(zone_text) {
            let style = self.label(g.name_cap, pal.roles.ink.with_alpha(LEVEL_LABEL), Align::Left);
            p.text(g.name_at, name, &style);
        }
        p.scoped(|p| {
            p.set_opacity(LOGO_LEVEL);
            logo_zone(p, g.logo, &self.logo, self.faces.wide, LogoVariant::Full, pal);
        });
    }
}

/// Builds the cluster for the registry.
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(BoschDdu::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn housing_fits_both_envelopes() {
        let t = Tokens::default();
        let h = GEOM.housing();
        let env = t.layout.cockpit_envelope;
        assert!(env.contains(h.min) && env.contains(h.max), "{h:?} in {env:?}");
        let hood = t.layout.hood_envelope;
        assert!(
            hood.contains(h.min * HOOD_SCALE) && hood.contains(h.max * HOOD_SCALE),
            "scaled {h:?} in {hood:?}"
        );
        let frame = hardware::HOUSING_FRAME;
        assert!(
            env.contains(frame.min) && env.contains(frame.max),
            "sprite frame in the envelope"
        );
    }

    #[test]
    fn leds_sit_on_the_brow_and_borders() {
        let g = &GEOM;
        let s = g.screen;
        for i in 0..TOP_LEDS {
            let r = g.top_led_rect(i);
            assert!(
                r.max.y < s.min.y - 20.0 && r.min.y > g.top - g.arch + 20.0,
                "top LED {i} {r:?} on the brow"
            );
            assert!(r.height() <= 42.0, "the strobing strip is at most 42 px tall");
        }
        for i in 0..SIDE_LEDS {
            for side in [-1.0, 1.0] {
                let r = g.side_led_rect(side, i);
                assert!(
                    r.min.x.abs().min(r.max.x.abs()) > s.max.x + 20.0,
                    "side LED clear of the screen"
                );
                assert!(r.max.x.abs() < g.half_w - 20.0, "side LED inside the housing");
                // Clear of the top corner's curve.
                let corner = vec2(side * (g.half_w - g.corner_top), g.top + g.corner_top);
                let far = vec2(r.center().x + side * r.width() * 0.5, r.min.y);
                assert!(
                    far.y > corner.y || (far - corner).length() < g.corner_top - 16.0,
                    "side LED {i} inside the corner"
                );
            }
        }
        assert!(
            (g.sensor.y - g.top) / (g.bottom - g.top) > 0.55,
            "the sensor sits in the lower right border"
        );
    }

    #[test]
    fn screen_zones_stay_on_the_screen_and_apart() {
        let g = &GEOM;
        let s = g.screen;
        // The rev numerals stay on the screen and clear of the quarter panels.
        let outer = g.num_r + g.num_cap * 0.5 + 4.0;
        assert!(g.rev_c.y - outer >= s.min.y, "numerals under the screen's top");
        assert!(g.rev_c.x - outer > g.left.1 - 4.0, "numerals clear of the left panels");
        // The beam starts inside the gauge's open bottom, clear of the track's ends.
        let end = Vec2::polar(g.rev_c, g.rev_r + g.rev_w * 0.5, g.rev_start);
        let beam = g.beam();
        let at_end =
            beam[0].x.abs() + (beam[3].x.abs() - beam[0].x.abs()) * ((end.y - g.beam_top) / (s.max.y - g.beam_top));
        assert!(at_end + 16.0 < end.x.abs(), "beam clear of the track's end");
        assert!(
            beam[2].x < g.right.0 && beam[3].x > g.left.1,
            "beam clear of the panels"
        );
        // The gear (at its 1.12 entry) stays inside the beam and on the screen.
        assert!(g.gear_at.y + g.gear_cap * 0.56 <= s.max.y, "gear on the screen");
        assert!(
            g.gear_at.y - g.gear_cap * 0.56 >= g.beam_top,
            "gear under the beam's top"
        );
        assert!(
            s.max.y < g.logo.min.y && g.logo.max.y < g.bottom,
            "logo zone on the chin"
        );
        assert!(g.logo.max.x < g.half_w - g.corner_bottom, "logo clear of the corner");
        let box_ok = g.shift_box.max.y < g.head_bottom - g.head_cap;
        assert!(box_ok, "the shift box clears the SPEED header");
        // ISO 15008 minimums: every dynamic readout at least 17.7 px cap, the gear at least 40.
        for cap in [
            g.num_cap,
            g.rpm_cap,
            g.value_cap,
            g.power_cap,
            g.speed_cap,
            g.g_value_cap,
        ] {
            assert!(cap >= 17.7);
        }
        assert!(g.gear_cap >= 40.0);
    }

    fn signals(rpm: f32) -> Signals {
        Signals {
            rpm,
            max_rpm: 8000.0,
            shift_rpm: 7760.0,
            shift_fraction: rpm / 7760.0,
            ..Signals::default()
        }
    }

    #[test]
    fn shift_lights_fill_left_to_right_then_strobe() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let mut leds = TopLeds::new();
        leds.update(&signals(1000.0), &pal, 1.0 / 60.0, false, true);
        assert!(
            (0..TOP_LEDS).all(|i| leds.color_of(i, &pal).is_none()),
            "dark at low rpm"
        );
        let mid = pal.stages[0] + (1.0 - pal.stages[0]) * 0.5;
        leds.update(&signals(7760.0 * mid), &pal, 1.0 / 60.0, false, false);
        let lit: Vec<bool> = (0..TOP_LEDS).map(|i| leds.color_of(i, &pal).is_some()).collect();
        assert!(lit[0] && !lit[TOP_LEDS - 1], "fills from the left: {lit:?}");
        assert!(lit.windows(2).all(|w| w[0] || !w[1]), "no gaps: {lit:?}");
        // Over the shift point: all ten in `shift` on the first frame, then the 10 Hz strobe.
        leds.update(&signals(7800.0), &pal, 1.0 / 60.0, false, false);
        assert!((0..TOP_LEDS).all(|i| leds.color_of(i, &pal) == Some(pal.shift)));
        for _ in 0..3 {
            leds.update(&signals(7900.0), &pal, 1.0 / 60.0, false, false);
        }
        assert!(
            leds.color_of(4, &pal).is_none(),
            "a middle LED strobed off on the fourth frame (6 frames per cycle)"
        );
        assert_eq!(leds.color_of(0, &pal), Some(pal.shift), "the outer pairs stay lit");
        assert_eq!(leds.color_of(9, &pal), Some(pal.shift));
        // A dip inside the hysteresis keeps it over; an upshift's drop clears it at once.
        leds.update(&signals(7700.0), &pal, 1.0 / 60.0, false, false);
        assert!(leds.over);
        leds.update(&signals(5200.0), &pal, 1.0 / 60.0, false, false);
        assert!(!leds.over);
        // Reduced flash never goes dark.
        let mut calm = TopLeds::new();
        for _ in 0..12 {
            calm.update(&signals(7900.0), &pal, 1.0 / 60.0, true, false);
            assert!(calm.color_of(4, &pal).is_some());
        }
    }

    #[test]
    fn side_columns_latch_their_steps() {
        assert_eq!(latched(0, 2.4, 2.5, 0.5, 5), 0);
        assert_eq!(latched(0, -5.1, 2.5, 0.5, 5), 2, "either direction of slip");
        assert_eq!(latched(2, 4.8, 2.5, 0.5, 5), 2, "held within the hysteresis");
        assert_eq!(latched(2, 4.4, 2.5, 0.5, 5), 1);
        assert_eq!(latched(0, 40.0, 2.5, 0.5, 5), 5, "at most five");
        assert_eq!(latched(3, f32::NAN, 2.5, 0.5, 5), 0);
    }

    #[test]
    fn fixed_formats_without_allocating() {
        let mut b = [0u8; 16];
        assert_eq!(fixed(&mut b, 124, 2), "1.24");
        assert_eq!(fixed(&mut b, 5, 2), "0.05");
        assert_eq!(fixed(&mut b, 124, 1), "12.4");
        assert_eq!(fixed(&mut b, -32, 1), "-3.2");
        assert_eq!(fixed(&mut b, 0, 1), "0.0");
        assert_eq!(fixed(&mut b, 712, 0), "712");
    }

    #[test]
    fn readout_refreshes_at_its_rate() {
        let mut r = Readout::new();
        r.update(100.0, 0.0);
        assert_eq!(r.shown, 100);
        r.update(110.0, 0.1);
        assert_eq!(r.shown, 100, "held between refreshes");
        r.update(120.0, 0.11);
        assert_eq!(r.shown, 120);
    }
}
