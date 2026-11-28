//! Bosch DDU (`ClusterTheme::BoschDdu`), catalog row 12: the Race Dash family's pattern-setter
//! (cockpit only).
//!
//! The Bosch Motorsport DDU 10 as BMW fits it to the M4 GT3 (`RESEARCH.md`): one black glass
//! face edge to edge in an anodised aluminium body with an arched top, large top corners and
//! small bottom ones; ten shift LEDs across the brow in groups of 2, 6 and 2, five LEDs down each
//! side, and a 7" screen with the round rev gauge in the middle, a horizon line under it, the
//! gear in a beam of light that runs from the horizon to the screen's foot, and four quarter
//! panels. The housing is the Blender render (`hardware.rs`, recipe
//! `tools/asset-forge/src/recipes/bosch_ddu.rs`) with a procedural fallback; everything lit is
//! drawn here. No Bosch marks: the chin carries the car's logo and the custom name instead.
//! Board: Penpot page `07 · BoschDdu`.
//!
//! Zone table (local 4K px round the cockpit anchor; the hood view draws the table's large parts
//! at `HOOD_SCALE`, 0.68, see below):
//!
//! | Zone | Part | Where | Holds |
//! |---|---|---|---|
//! | H | housing | x ±405, y −316..232 (arch crest −316, corners −306), top corners R 120, bottom R 44 | anodised body, black glass face, light sensor (359, 24) R 8 |
//! | L·T | top LEDs | 10 lenses 28 × 10, centres y −284 on the arch, x ±19, ±57, ±95 (6) and ±161, ±199 (2 + 2) | rank 1: shift lights, left to right from the first rev stage to the shift point (the tenth lights at it), two green, six amber, two red (the palette's stage colours), then all ten in the shift lamp (`shift`, lifted toward ink to at least the amber stage's light, so that state is the brow's brightest for any palette) with a hotter core; the middle six strobe at 10 Hz from the shift point |
//! | L·S | side LEDs | 5 lenses 28 × 10 per side at x ±359, centres y −70 up to −222 (pitch 38) | left: brake (fifths, `warning`), on the brake bar's side; right: body slip (one per 2.5°, `caution`); a halo, no bloom |
//! | S | screen | x ±312, y −204..170 | true black |
//! | Z | horizon | y 9, across both panel columns and the gauge's disc | a tinted line with a soft band of light over it: it splits the rows, cuts the disc flat and carries the beam's apex |
//! | R | rev gauge | centre (0, −39), scale ring R 115 over 270° from 7:30, the lit band R 91..113 inside it, numerals outside at R 137 (cap 20) | rank 3: the rpm as a band in `primary` with a needle at its head (cross-faded to ink over the shift point, so only the LEDs carry `shift`); the limiter's tick wider than the others and reaching further in (its outer end where theirs is); the rpm digits (cap 36) and "rpm" in the disc |
//! | G | gear beam | trapezoid y 9..170, ±32 at the horizon, ±120 at the foot | rank 1: the gear (cap 132, every gear in ink) in a beam of `primary` light, whose two edge lines turn `warning` in reverse |
//! | Q1 | PEDALS | x −312..−156, y −204..−15 | throttle and brake bars: 28 fine lines, a marker at the value |
//! | Q2 | G-FORCE | x −312..−156, y 9..170 | a full ring (R 54, band 14) at (−234, 108) on a dim track (half `ink_ghost`), filled clockwise from 12 o'clock by the felt force of the shared `GBall` (full at 1.5 g), a bead at the force's bearing, the total G (cap 26) and its unit inside |
//! | Q3 | ENGINE | x 156..312, y −204..−15 | power (cap 28, `signals.power_unit`), boost (psi; its value in ink, its label the panel's one accent line: the text-safe accent at the print's light), the shift rpm boxed |
//! | Q4 | SPEED | x 156..312, y 9..170 | rank 2: tabular speed (cap 58, no ghost 8s: at a glance they read as digits) and its unit |
//! | C | chin | y 170..232 | the custom name (left zone), the car's logo zone (right); the hood view keeps a logo image only |
//!
//! The four quarter panels can be swapped or hidden with the config key `BoschDduPanels`: every
//! panel fits every slot (the slots mirror about the gauge). A top panel's header stands on a
//! tinted rule that turns up at the gauge; a bottom panel's stands under the horizon. A panel
//! uses three type sizes: its header and labels (cap 15: 16 arcminutes at the 0.9 m seat is
//! 14.1 px), one big value and one small value; a row's unit follows its label in lower case,
//! and the three values that stand alone (rpm, speed, total G) carry theirs under the digits
//! (cap 16).
//!
//! Sizes against the tokens: the real DDU 10 leads here (`RESEARCH.md`), so the gear (cap 132,
//! 0.35 of the 374 px screen against `type.ratios.raceGearToPanelHeight` 0.45), the speed
//! (cap 58) and the LED row (426 px) are under the family's `layout.zones` minimums (150, 84,
//! 720), which describe the earlier wide concept; gear and figures are larger on the screen than
//! the real display's, and a test pins the three sizes. The catalog lists the cockpit view only.
//! The hood view is an uncatalogued fallback at `HOOD_SCALE` that draws only what stays over
//! the size floors there (one decision, `shows`): the housing, the LEDs, the horizon, the rev
//! band with its ticks and digits, the gear in its beam, the speed's digits and a logo image on
//! the chin. The other three panels (ENGINE goes whole, although its power figure alone would be
//! 19 px), every header, label and unit, the rev numerals and the chin's wordmarks are left out.
//!
//! Every static part tinted with `primary` or `warning` (the beam at rest, the header rules and
//! the horizon, the red zone, the shift box) is held at or under the static cap of 0.2 relative
//! luminance whatever the palette (`static_alpha`); the printed parts sit under it by their share
//! of the label role.
//!
//! Signature moments:
//! - **Shift lights:** the brow fills left to right: LED 1 in the frame the palette's first rev
//!   stage is crossed (0.75 of the shift rpm in the tokens), one more every ninth of the range
//!   from there to the shift point, so nine are lit just under it and the tenth lights at it.
//!   Each LED has its group's colour as BMW sets the M4 GT3's: two green, six amber, two red
//!   (the palette's three stage colours). At the shift point all ten turn the shift lamp with a
//!   hotter core, the brow's brightest state for any palette (lens and halo), and the middle six strobe at 10 Hz on their own clock
//!   (the strobe's first frame is lit) while the outer pairs stay lit, as the shared
//!   `ShiftLights` strobes from the shift point too (the tokens' text says "at the limiter").
//!   The LEDs go out within 50 ms of the rpm falling. Over the shift point the band cross-fades
//!   to ink in 0.1 s (held at least 0.35 s, no bloom while ink). Over the shift point, or on a
//!   limiter bounce just under it, the rpm digits and the band show the highest rpm of the last
//!   0.3 s, so neither the digits nor the band's head flicker on a bounce; an upshift's drop
//!   releases them at once. With reduced flash the middle six breathe at 2 Hz between 0.6
//!   and full, from the peak, instead of strobing.
//! - **Gear change:** the token glyph motion inside the beam; the beam flares once.
//! - **Light:** lateral G swings one key light 25° across the housing (three renders,
//!   cross-faded).
//!
//! The first frame, a race-flag change and the first frame after a gap take the car's state as
//! it is (no latch, peak, fade or glyph motion carries over, and the G ball and the virtual light
//! are put at rest on that frame's force, so nothing drifts after it). Telemetry: rpm, limiter and shift point,
//! gear, speed, pedals, smoothed G and body slip (derived from the verified velocity) from
//! `Signals`; power and boost from the packet. Boost reads in psi, the packet's own unit,
//! whatever the speed unit (the config has no pressure unit yet). Every readout shows its raw
//! value on the frame it moves, held by `fh6_kit::Hysteresis` (rpm by 30, power by 1 % and at
//! least 3, boost by 0.3 psi, total G by 0.03).
mod hardware;

use crate::common::gball::GBall;
use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::race::{draw_fixed, saturate_to_cells};
use crate::common::speed::TabularSpeed;
use crate::common::VirtualLight;
use core::f32::consts::{PI, TAU};
use fh6_kit::cluster::Setup;
use fh6_kit::logo::LogoStyle;
use fh6_kit::{
    vec2, Align, Bloom, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, Hysteresis,
    LayerOptions, Layout, Logo, Painter, Palette, PowerUnit, Rect, Shape, Signals, SpeedUnit, Strobe, Stroke,
    TextStyle, Tokens, Tween, VAlign, Vec2, View,
};
use hardware::Sprites;

/// The hood view draws the cockpit table's large parts at this scale (housing 548 px tall → 373,
/// inside the hood's 400 px envelope); [`shows`] says what it leaves out.
const HOOD_SCALE: f32 = 0.68;
/// Top LEDs.
const TOP_LEDS: usize = 10;
/// The outer LEDs at each end of the brow (the 2 of the 2-6-2 groups) stay steady over the
/// shift point while the middle six strobe.
const OUTER: usize = 2;
/// LEDs in each side column.
const SIDE_LEDS: usize = 5;
/// Body slip per side LED (degrees), and the fall back a lit LED needs before it goes out.
const SLIP_STEP_DEG: f32 = 2.5;
const SLIP_HYST_DEG: f32 = 0.5;
/// Brake below this is off (pedal noise); a lit first LED goes out only [`BRAKE_HYST`] of this
/// under it, so a trigger resting at the dead zone does not flicker it.
const BRAKE_DEAD: f32 = 0.02;
/// The shift lights' release (s): a full strip is dark this long after the rpm has dropped
/// (tokens `motion.shiftLights.releaseMaxMs` 50), three frames at 60 Hz.
const RELEASE_S: f32 = 0.05;
/// Once over the shift point the lights stay over until the rpm falls this far under it
/// (tokens `hysteresisRpm`).
const SHIFT_HYSTERESIS_RPM: f32 = 120.0;
/// A gap in updates at least this long (s) ends in a snapped frame: take the car's state as it
/// is. The host caps `dt` (`fh6-app` `MAX_DT`, 0.1 s), so this must stay under that cap for a
/// gap to show in `dt`. The cluster's own clock (`FrameInput::t`, f32 seconds) is not used: after
/// about 12 days its step passes this value and every step would read as a gap.
const RESUME_DT: f32 = 0.09;
/// The rpm digits show the highest rpm of a window this long (s), kept in two buckets of half
/// its length ([`PeakWindow`]), while [`holds_peak`] says so: over the shift point, or on a
/// limiter bounce just under it. A bounce is 50 to 150 ms, so the window always holds a whole
/// bounce and the digits stay on its peak instead of dipping and coming back. The band and its
/// needle take the same held rpm, or their head would jump between the bounce's top and bottom
/// on every frame of a limiter hold (a 30 Hz flash in the flash audit).
const PEAK_HOLD_S: f32 = 0.3;
/// A limiter bounce falls at most this far (rpm) under its peak: within [`ARC_HOLD_S`] of the
/// rpm last reaching the shift point (its latch's band) the digits keep the peak while the rpm
/// stays this close to it, even when the bounce drops out of the shift lights' latch (a shift
/// point at the limiter, or a deep bounce). An upshift drops far more, so it releases the digits
/// at once.
const BOUNCE_RPM: f32 = 500.0;
/// The readouts' hysteresis (the kit's `Hysteresis` on the raw value, as the lessons of
/// `CLUSTER_PROMPT.md` set it): rpm by 30 (in tens), power by 1 % and at least 3 of its unit,
/// boost by 0.3 psi (3 tenths of a psi), each in the unit it is shown in.
const RPM_HYST_TENS: f32 = 3.0;
const POWER_HYST_SHARE: f32 = 0.01;
const POWER_HYST_MIN: f32 = 3.0;
const BOOST_HYST_TENTHS: f32 = 3.0;
/// The total G's hysteresis (hundredths of a g): the limiter's bounce moves the longitudinal G
/// a little every frame.
const G_HYST_CENTI: f32 = 3.0;
/// The digit cells of each readout, which also bound the value it shows (`saturate_to_cells`):
/// rpm and the shift point (up to 99,999), power (9,999) and the total G (9.99 g).
const RPM_CELLS: u8 = 5;
const POWER_CELLS: u8 = 4;
const G_CELLS: u8 = 3;
/// A limiter under this (rpm) was not sent: `Signals` reports a missing one as 1. The scale's
/// top when no limiter is known (rpm).
const LIMITER_MIN_RPM: f32 = 500.0;
const DEFAULT_SCALE_TOP: f32 = 8000.0;
/// The G ring, a full circle: the felt force that fills it (g), where its fill starts (12
/// o'clock, running clockwise), its band's width (px; 0.23 of the outer radius, towards the DDU
/// fuel ring's heavy band), the bead at the force's bearing (radius, px) and the force at which
/// the bead is fully lit (g).
const G_FULL: f32 = 1.5;
const G_START: f32 = 1.5 * PI;
const G_BAND: f32 = 14.0;
const G_BEAD: f32 = 3.5;
const G_BEAD_FULL: f32 = 0.15;
/// The G ring's unlit track: this share of `ink_ghost` (0.03 relative luminance, 1.6:1 over
/// black), so the ring's form reads without lifting the light round the G value.
const G_TRACK_SHARE: f32 = 0.5;
/// A snapped frame settles a new G ball and the virtual light on the frame's force in this many
/// updates of this length (s): `Spring::update` takes at most 0.25 s a step, and 2 s is twelve
/// periods of the tokens' `gBall` spring, thirteen of the `light` spring and four lengths of the
/// ball's trail.
const SETTLE_STEPS: usize = 8;
const SETTLE_DT: f32 = 0.25;
/// A lit brake LED goes out only when the brake has fallen this far under its step (share of a
/// step; of the dead zone for the first LED), so a pedal resting on a fifth does not flicker it.
const BRAKE_HYST: f32 = 0.15;
/// The beam's flare on a gear change (s, linear fade) and its peak.
const FLARE_S: f32 = 0.25;
const FLARE_PEAK: f32 = 0.6;
/// The gear beam's light at rest (alpha of `primary`, or of `warning` for the edges in reverse;
/// static parts, held under [`STATIC_CAP`] by [`static_alpha`]): its body at the apex, fading out
/// towards the foot, and the bands of light inside its two edges with their width (px). The
/// flare on a gear change lifts them for a moment.
const BEAM_APEX: f32 = 0.3;
const BEAM_EDGE: f32 = 0.34;
const BEAM_BAND: f32 = 22.0;
/// The gear's bloom: far under the token default (0.8). The kit's bloom is a pass over the
/// whole frame that no clip bounds, so this level, not a clip, is what limits the gear's light
/// on the chin under the screen.
const GEAR_BLOOM: f32 = 0.05;
/// The horizon's soft band of light: alpha of `primary` at the line over the panels and inside
/// the rev gauge's disc (held under [`STATIC_CAP`]), and the height of each band (px). The disc's is the lower one, so the
/// "rpm" unit stands on black over it.
const HORIZON_GLOW: f32 = 0.07;
const DISC_GLOW: f32 = 0.18;
const HORIZON_BAND: f32 = 24.0;
const DISC_BAND: f32 = 16.0;
/// Header rules and the horizon line are tinted: this share of `primary` (held under
/// [`STATIC_CAP`]), drawn opaque.
const RULE_TINT: f32 = 0.3;
/// The static cap (relative luminance over black): no part that never moves is brighter, the
/// burn-in rule.
const STATIC_CAP: f32 = 0.2;
/// Labels at cap 15 sit this far (px) from their line, so their cap edges (7.5 px either side of
/// the cap middle) land on whole pixels. The values keep their lines.
const LABEL_NUDGE: f32 = -0.5;
/// A header rule's 45° end at the gauge (px), and how far the rule's middle sits under the
/// header's cap (px): with the header's half cap it lands the 2 px rule on whole pixels.
const KICK: f32 = 22.0;
const RULE_DROP: f32 = 8.5;
/// A lit LED's hot centre: this share of ink mixed into its colour ([`lens_core`]; tokens
/// `components.shiftLights.pod.core`: 40 % toward white), and the share on the brow over the
/// shift point. With the shift lamp ([`shift_lamp`]) at least as bright as the amber stage, ten
/// lit lenses then give more light than the fullest staged fill, lenses and halos counted, for
/// any palette (a test integrates what `led` paints for several).
const LED_CORE: f32 = 0.4;
const SHIFT_CORE: f32 = 0.8;
/// A lit LED's inner rim, 1 device px: alpha of ink (tokens `pod.innerRimAlphaPct` 50). It keeps
/// neighbouring lit lenses apart.
const LED_RIM: f32 = 0.5;
/// Reduced flash: the middle six breathe at this rate (Hz) by this depth under full (tokens
/// `motion.limiterStrobe.reducedFlash`: 2 Hz, 40 %), starting at the peak.
const BREATH_HZ: f32 = 2.0;
const BREATH_DEPTH: f32 = 0.4;
/// The rev gauge's lit band: its strength under the scale ring and at its inner edge, and the
/// bright edge under the ring: its strength at the band's start (its head is at full strength)
/// and its width (px).
const ARC_BODY: f32 = 0.55;
const ARC_INNER: f32 = 0.1;
const ARC_TAIL: f32 = 0.5;
const ARC_HOT: f32 = 7.0;
/// The bloom of the band's bright edge while the band is `primary`, under the token default
/// (0.8) for the numerals beside it. It falls to none as the band turns ink.
const ARC_BLOOM: f32 = 0.35;
/// The band over the shift point: its cross-fade from `primary` to ink and back (s), and how
/// long it stays ink at least once the shift point is reached (s), so brushing the shift point
/// never flickers the band.
const ARC_FADE_S: f32 = 0.1;
const ARC_HOLD_S: f32 = 0.35;
/// The ticks across the scale ring (px): how far each reaches inside and outside the ring and
/// its width (the line's ends are round, half the width further on). The limiter's tick is wider
/// and starts much further inside, so it differs by shape and not by its red alone; outside the
/// ring its round end stops where the other ticks' ends do, clear of the numeral beside it.
const TICK_IN: f32 = 4.0;
const TICK_OUT: f32 = 5.0;
const TICK_W: f32 = 2.0;
const LIMIT_TICK_IN: f32 = 10.0;
const LIMIT_TICK_W: f32 = 3.5;
const LIMIT_TICK_OUT: f32 = TICK_OUT - (LIMIT_TICK_W - TICK_W) * 0.5;
/// The shift box's inner padding at each end (px), and the air between a row's label and its
/// unit.
const SHIFT_PAD: f32 = 6.0;
const ROW_GAP: f32 = 4.0;
/// The red zone on the scale ring (px wide), and its strength (alpha of `warning`; a static part,
/// held under [`STATIC_CAP`]).
const RED_BAND: f32 = 5.0;
const RED_LEVEL: f32 = 0.75;
/// The printed parts (numerals, labels, the boxed shift point, the chin) draw at this share of
/// the `ink_label` role: 0.19 relative luminance, so they keep 4.5:1 over black (4.8:1) and stay
/// under the static cap (0.2), dimmer than every live value; a test holds both. The role itself
/// (alpha 120 of 255) is over the cap. Unlit segments are `ink_ghost`.
const PRINT_SHARE: f32 = 0.44;
/// The hairlines (the disc's ring, the bars' two rules, the minor ticks) are one tier under the
/// print: this share of the `hairline` role (0.11 relative luminance, 3.2:1 over black),
/// drawn opaque so crossing strokes show no brighter joints.
const HAIRLINE_SHARE: f32 = 0.6;
/// A logo image carries no label alpha of its own, so it draws at this opacity: a white logo
/// then stays under the static cap (0.2).
const LOGO_SHARE: f32 = 0.19;
/// The outline of the box round the shift point (alpha of `primary`; a static part, held under
/// [`STATIC_CAP`]). The box has no fill: the printed digits stand on black.
const SHIFT_BOX: f32 = 0.4;
/// A pedal bar: its fine lines, their width, how far the marker stands over the bar, and how far
/// a bar's label sits over its top (px; with [`LABEL_NUDGE`] in it, so THR and BRK share their
/// lines with POWER and BOOST).
const HATCH: usize = 28;
const HATCH_W: f32 = 2.0;
const MARKER_OVER: f32 = 4.0;
const BAR_LABEL: f32 = 12.5;

/// Numerals 0 to 20 (thousands), so drawing never formats a string.
const NUMERALS: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
];

/// The zone table as data (cockpit, local 4K px round the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
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
    /// The horizon line's y: the rows' split, the disc's flat foot and the beam's apex.
    horizon: f32,
    rev_c: Vec2,
    /// The lit band's middle radius and its width; the scale ring runs just outside it.
    rev_r: f32,
    rev_w: f32,
    rev_start: f32,
    rev_sweep: f32,
    num_r: f32,
    num_cap: f32,
    rpm_cap: f32,
    rpm_at: Vec2,
    /// The units that stand under their value (rpm, speed, total G).
    unit_cap: f32,
    /// The beam's half widths at the horizon and at the screen's foot.
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
    /// Offsets below a panel's header line (px): a panel's first row label, the pedal bars'
    /// tops, the G ring's centre, the power figure, the boost row, the shift row's middle, the
    /// speed and its unit.
    row_dy: f32,
    bar_h: f32,
    thr_dy: f32,
    brk_dy: f32,
    g_dy: f32,
    g_r: f32,
    g_value_cap: f32,
    power_dy: f32,
    power_cap: f32,
    boost_dy: f32,
    value_cap: f32,
    shift_dy: f32,
    shift_h: f32,
    speed_dy: f32,
    speed_cap: f32,
    unit_dy: f32,
    /// The chin's two zones: the custom name (left) and the car's logo (right).
    name: Rect,
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
        min: Vec2 { x: -312.0, y: -204.0 },
        max: Vec2 { x: 312.0, y: 170.0 },
    },
    top_led_x: [-199.0, -161.0, -95.0, -57.0, -19.0, 19.0, 57.0, 95.0, 161.0, 199.0],
    top_led_y: -284.0,
    top_led: Vec2 { x: 28.0, y: 10.0 },
    side_led_x: 359.0,
    side_led_y0: -70.0,
    side_led_pitch: 38.0,
    side_led: Vec2 { x: 28.0, y: 10.0 },
    horizon: 9.0,
    rev_c: Vec2 { x: 0.0, y: -39.0 },
    rev_r: 102.0,
    rev_w: 22.0,
    rev_start: 0.75 * PI,
    rev_sweep: 1.5 * PI,
    num_r: 137.0,
    num_cap: 20.0,
    rpm_cap: 36.0,
    rpm_at: Vec2 { x: 0.0, y: -54.0 },
    unit_cap: 16.0,
    beam_half: (32.0, 120.0),
    gear_at: Vec2 { x: 0.0, y: 94.0 },
    gear_cap: 132.0,
    left: (-312.0, -156.0),
    right: (156.0, 312.0),
    head_top: -182.0,
    head_bottom: 24.0,
    head_cap: 15.0,
    label_cap: 15.0,
    row_dy: 34.0,
    bar_h: 18.0,
    thr_dy: 46.0,
    brk_dy: 108.0,
    g_dy: 84.0,
    g_r: 54.0,
    g_value_cap: 26.0,
    power_dy: 66.0,
    power_cap: 28.0,
    boost_dy: 96.0,
    value_cap: 18.0,
    shift_dy: 128.0,
    shift_h: 26.0,
    speed_dy: 81.0,
    speed_cap: 58.0,
    unit_dy: 128.0,
    name: Rect {
        min: Vec2 { x: -330.0, y: 184.0 },
        max: Vec2 { x: -150.0, y: 220.0 },
    },
    logo: Rect {
        min: Vec2 { x: 150.0, y: 184.0 },
        max: Vec2 { x: 330.0, y: 220.0 },
    },
};

/// One quarter panel's place: its x range, its header line, its side of the gauge and its row
/// (the top row's headers mirror about the gauge and stand on a rule; the bottom row's are both
/// left-aligned).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Slot {
    x: (f32, f32),
    head: f32,
    right_side: bool,
    top: bool,
}

/// Quarter panels, in the order of the `BoschDduPanels` list: left top, left bottom, right top,
/// right bottom.
const PANELS: usize = 4;

impl Geom {
    /// Quarter panel `i` (see [`PANELS`]).
    fn slot(&self, i: usize) -> Slot {
        let right_side = i >= 2;
        let top = i.is_multiple_of(2);
        Slot {
            x: if right_side { self.right } else { self.left },
            head: if top { self.head_top } else { self.head_bottom },
            right_side,
            top,
        }
    }

    /// The shift row's y range in an ENGINE panel at `s`.
    fn shift_row(&self, s: Slot) -> (f32, f32) {
        let middle = s.head + self.shift_dy;
        (middle - self.shift_h * 0.5, middle + self.shift_h * 0.5)
    }

    /// The scale ring's radius: just outside the lit band.
    fn ring_r(&self) -> f32 {
        self.rev_r + self.rev_w * 0.5 + 2.0
    }

    /// The radius of the disc inside the lit band, which holds the rpm digits.
    fn disc_r(&self) -> f32 {
        self.rev_r - self.rev_w * 0.5 - 5.0
    }

    /// The housing's body below the arch.
    fn body(&self) -> Rect {
        Rect::from_points(vec2(-self.half_w, self.top), vec2(self.half_w, self.bottom))
    }

    /// The whole housing, arch included.
    #[cfg(test)]
    fn housing(&self) -> Rect {
        Rect::from_points(vec2(-self.half_w, self.top - self.arch), vec2(self.half_w, self.bottom))
    }

    /// Half the flat span between the top corners, which the arch bridges.
    fn arch_span(&self) -> f32 {
        self.half_w - self.corner_top
    }

    /// The radius of the arch's circle: it rises `arch` over the span between the top corners.
    fn arch_radius(&self) -> f32 {
        let span = self.arch_span();
        (span * span + self.arch * self.arch) / (2.0 * self.arch)
    }

    /// How far the arch drops below its crest at `x` (0 at the middle).
    fn arch_drop(&self, x: f32) -> f32 {
        let r = self.arch_radius();
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

    /// The gear beam's corners: apex left, apex right, foot right, foot left.
    fn beam(&self) -> [Vec2; 4] {
        let (t, b) = self.beam_half;
        let foot = self.screen.max.y;
        [
            vec2(-t, self.horizon),
            vec2(t, self.horizon),
            vec2(b, foot),
            vec2(-b, foot),
        ]
    }
}

/// What a quarter panel shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    Pedals,
    GForce,
    Engine,
    Speed,
    Off,
}

impl Content {
    fn parse(value: &str) -> Option<Content> {
        let v = value.trim();
        [
            ("pedals", Content::Pedals),
            ("g", Content::GForce),
            ("gforce", Content::GForce),
            ("engine", Content::Engine),
            ("speed", Content::Speed),
            ("off", Content::Off),
            ("hidden", Content::Off),
        ]
        .into_iter()
        .find_map(|(name, content)| v.eq_ignore_ascii_case(name).then_some(content))
    }
}

/// The dash's own option, from the config's extra key `BoschDduPanels` (hot-reloaded; set by
/// hand in `config.json` until the OSD and the configurator offer it): the four quarter panels'
/// contents as a comma-separated list in the order left top, left bottom, right top, right
/// bottom, each `Pedals`, `G`, `Engine`, `Speed` or `Off` (default `Pedals,G,Engine,Speed`). A
/// missing or unknown entry keeps its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Options {
    panels: [Content; PANELS],
}

impl Default for Options {
    fn default() -> Self {
        Self {
            panels: [Content::Pedals, Content::GForce, Content::Engine, Content::Speed],
        }
    }
}

impl Options {
    /// Parses the list; an empty one keeps every default.
    fn parse(list: &str) -> Self {
        let mut opts = Self::default();
        for (slot, name) in opts.panels.iter_mut().zip(list.split(',')) {
            if let Some(content) = Content::parse(name) {
                *slot = content;
            }
        }
        opts
    }
}

/// What the hood view decides on: a quarter panel by its content, or the fine print (every
/// header, label and unit, the rev numerals and the chin's wordmarks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Panel(Content),
    Print,
}

/// True when `part` is drawn in `view`. The cockpit draws everything; the hood view, at
/// [`HOOD_SCALE`], keeps only what stays over the size floors there and leaves the rest out
/// whole: the PEDALS, G-FORCE and ENGINE panels (ENGINE although its power figure alone would be
/// 19 px: its labels and boxed shift point could not come with it), and the fine print (caps 15
/// and 16 would be 10.2 and 10.9 px, under the tokens' smallest size; the rev numerals' cap 20
/// would be 13.6 px, under the 17.7 px of a readout; the chin's wordmark, the kit's cap 18,
/// would be 12.2 px at best, at the smallest size's edge, and under it whenever a long name is
/// shrunk to its zone).
/// The SPEED panel keeps its digits.
fn shows(view: View, part: Part) -> bool {
    view != View::Hood || part == Part::Panel(Content::Speed)
}

/// A static part's alpha of `color`: `alpha`, or less where that would pass [`STATIC_CAP`] over
/// black, so a light palette colour cannot lift a part that never moves over the cap.
fn static_alpha(color: Color, alpha: f32) -> f32 {
    alpha.min(STATIC_CAP / color.luminance().max(1e-3))
}

/// `color` at a static part's alpha ([`static_alpha`]).
fn static_tint(color: Color, alpha: f32) -> Color {
    color.with_alpha(static_alpha(color, alpha))
}

/// The colour of a lit lens's hot centre: `core` of ink mixed into `color`, at the lens's own
/// level (a breathing lens's centre breathes with it).
fn lens_core(color: Color, core: f32, pal: &Palette) -> Color {
    color.lerp(pal.roles.ink.fade(color.a), core)
}

/// The brow's lamp over the shift point: `shift`, lifted toward ink until it gives at least the
/// amber stage's light (a dark or saturated shift colour, such as `#FF1744`, gives less), so ten
/// lit lenses always outshine the fullest staged fill. A shift colour as bright as amber or
/// brighter is kept as it is.
fn shift_lamp(pal: &Palette) -> Color {
    let gap = (light(pal.roles.caution) - light(pal.shift)).max(0.0);
    let room = (light(pal.roles.ink) - light(pal.shift)).max(1e-3);
    pal.shift.lerp(pal.roles.ink, (gap / room).clamp(0.0, 1.0))
}

/// The halo round a lit lens: the first lobe of the tokens' medium glow.
fn led_halo(t: &Tokens) -> Option<Glow> {
    t.glow
        .medium
        .lobes()
        .first()
        .map(|lobe| Glow::new(lobe.sigma, lobe.intensity))
}

/// True when the shift point's limiter is known: `Signals` reports a missing limiter as 1 and
/// derives the shift rpm from it.
fn limiter_known(sig: &Signals) -> bool {
    sig.max_rpm > LIMITER_MIN_RPM && sig.shift_rpm > 0.0
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

/// The ten brow LEDs: they fill left to right from the palette's first rev stage to the shift
/// point (0 ms attack, a release of at most 50 ms), turn `shift` together at the shift point
/// and strobe from there.
#[derive(Debug, Clone, Copy, PartialEq)]
struct TopLeds {
    /// Share of the strip lit (0..=1): LED `i` (0 at the left) is lit from `(i + 1) / 10`.
    level: f32,
    over: bool,
    /// The strobe's own clock (s): from the frame the shift point is reached.
    strobe_t: f32,
    strobe_on: bool,
    breath: f32,
    /// True while the level falls toward its target (the release).
    releasing: bool,
}

impl TopLeds {
    fn new() -> Self {
        Self {
            level: 0.0,
            over: false,
            strobe_t: 0.0,
            strobe_on: true,
            breath: 1.0,
            releasing: false,
        }
    }

    /// Feeds the rpm. A car that sends no limiter or shift point leaves the strip dark.
    fn update(&mut self, sig: &Signals, pal: &Palette, dt: f32, calm: bool, snap: bool) {
        if snap {
            // A snapped frame starts from the car's state: no latch or strobe phase carries over.
            self.over = false;
            self.strobe_t = 0.0;
        }
        let known = limiter_known(sig);
        let rpm = sig.rpm.max(0.0);
        let frac = if known { sig.shift_fraction.max(0.0) } else { 0.0 };
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
        } else if known && frac >= start {
            // LED 1 at the first stage, one more every ninth of the range, the tenth at the
            // shift point: reaching it adds a light as well as the colour. Under the shift point
            // the level stays half a rank under the tenth, which lights only with the shift state.
            let range = ((frac - start) / (1.0 - start).max(1e-3)).clamp(0.0, 1.0);
            let staged = (1.0 + (TOP_LEDS - 1) as f32 * range) / TOP_LEDS as f32;
            staged.min((TOP_LEDS as f32 - 0.5) / TOP_LEDS as f32)
        } else {
            0.0
        };
        self.level = if target >= self.level || snap {
            target
        } else {
            // A linear release: even a full strip is dark within [`RELEASE_S`].
            (self.level - dt.max(0.0) / RELEASE_S).max(target)
        };
        self.releasing = self.level > target;
        if calm {
            // A breath from the peak: full on the frame the shift point is reached.
            self.strobe_on = true;
            let dip = 0.5 - 0.5 * (self.strobe_t * TAU * BREATH_HZ).cos();
            self.breath = 1.0 - BREATH_DEPTH * dip;
        } else {
            self.strobe_on = Strobe::LIMITER.on(self.strobe_t);
            self.breath = 1.0;
        }
    }

    /// The colour of LED `i` (0 at the left), or `None` when it is dark or strobed off. LED `i`
    /// lights when the rpm reaches `stage 1 + (1 − stage 1) × i / 9` of the shift rpm, so the
    /// first lights as stage 1 is crossed and the tenth at the shift point. A lit LED has its
    /// group's colour (the lenses' 2-6-2 groups, as BMW sets the M4 GT3's): the palette's three
    /// stage colours in turn; over the shift point all are the shift lamp ([`shift_lamp`]).
    fn color_of(&self, i: usize, pal: &Palette) -> Option<Color> {
        let rank = (i + 1) as f32 / TOP_LEDS as f32;
        if self.level < rank - 1e-4 {
            return None;
        }
        if self.over {
            // The outer pairs stay lit, so the strobe's dark phase never reads as "no shift";
            // the middle six strobe.
            let lamp = shift_lamp(pal);
            if !(OUTER..TOP_LEDS - OUTER).contains(&i) {
                return Some(lamp);
            }
            return self.strobe_on.then(|| lamp.fade(self.breath));
        }
        let stage = if i < OUTER {
            pal.roles.go
        } else if i < TOP_LEDS - OUTER {
            pal.roles.caution
        } else {
            pal.roles.warning
        };
        Some(stage)
    }

    /// The share of ink in a lit lens's hot centre: the shift state is the brow's brightest.
    fn core(&self) -> f32 {
        if self.over {
            SHIFT_CORE
        } else {
            LED_CORE
        }
    }

    /// True while the strip changes without new telemetry: the strobe or breath over the shift
    /// point, or a release under way. A steady staged fill is still.
    fn animating(&self) -> bool {
        self.over || self.releasing
    }
}

/// The rev band's colour over the shift point: it cross-fades between `primary` and ink in
/// [`ARC_FADE_S`] instead of stepping, and once the shift point is reached it stays ink for at
/// least [`ARC_HOLD_S`].
#[derive(Debug, Clone, Copy, PartialEq)]
struct ArcInk {
    /// Share of ink in the band (0 is `primary`, 1 is ink).
    mix: f32,
    /// Time since the shift point was last reached (s), counted up to [`ARC_HOLD_S`].
    since: f32,
}

impl ArcInk {
    fn new() -> Self {
        Self {
            mix: 0.0,
            since: ARC_HOLD_S,
        }
    }

    /// Feeds whether the lights are over the shift point, and whether they got there on this
    /// frame (`reached`). A snapped frame takes the state as it is, with no fade.
    fn update(&mut self, over: bool, reached: bool, dt: f32, snap: bool) {
        if snap {
            self.mix = if over { 1.0 } else { 0.0 };
            self.since = if over { 0.0 } else { ARC_HOLD_S };
            return;
        }
        self.since = if reached {
            0.0
        } else {
            (self.since + dt).min(ARC_HOLD_S)
        };
        let held = self.mix > 0.0 && self.since < ARC_HOLD_S;
        let step = dt / ARC_FADE_S;
        self.mix = if over || held {
            (self.mix + step).min(1.0)
        } else {
            (self.mix - step).max(0.0)
        };
    }

    /// True while the band has ink to lose: in its fade, or held with the rpm back under the
    /// shift point.
    fn animating(&self, over: bool) -> bool {
        self.mix > 0.0 && (self.mix < 1.0 || !over)
    }
}

/// The highest rpm of the last [`PEAK_HOLD_S`], for the rpm digits at the shift point: a
/// running maximum in two buckets of half that length, so a peak is kept for at least half the
/// window and at most all of it, and nothing sinks or allocates.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PeakWindow {
    /// The last full bucket's peak, and the peak of the bucket being filled.
    prev: f32,
    cur: f32,
    /// How long the current bucket has been filled (s).
    t: f32,
}

impl PeakWindow {
    fn new() -> Self {
        Self {
            prev: 0.0,
            cur: 0.0,
            t: 0.0,
        }
    }

    /// Feeds the rpm and returns the window's peak, never under `rpm`. A snapped frame starts
    /// the window again at `rpm`.
    fn update(&mut self, rpm: f32, dt: f32, snap: bool) -> f32 {
        if snap {
            *self = Self {
                prev: rpm,
                cur: rpm,
                t: 0.0,
            };
            return rpm;
        }
        self.t += dt;
        if self.t >= PEAK_HOLD_S * 0.5 {
            self.prev = self.cur;
            self.cur = rpm;
            self.t = 0.0;
        }
        self.cur = self.cur.max(rpm);
        self.prev.max(self.cur)
    }
}

/// True when the rpm digits keep the window's `peak`: over the shift point, or, still holding
/// it, within [`ARC_HOLD_S`] of the rpm last reaching the shift point (`since_reached`: the last
/// frame it was inside the lights' latch band, [`SHIFT_HYSTERESIS_RPM`] under the shift point,
/// so a bounce whose later peaks stop just under a shift point at the limiter still counts) and
/// within [`BOUNCE_RPM`] of the peak. Once released the digits follow the rpm until the lights
/// are over the shift point again, so an old peak never comes back.
fn holds_peak(over: bool, holding: bool, since_reached: f32, peak: f32, rpm: f32) -> bool {
    over || (holding && since_reached < ARC_HOLD_S && peak - rpm <= BOUNCE_RPM)
}

/// The rpm digits: 10 rpm steps with 30 rpm of hysteresis on the raw rpm (or on the peak while
/// [`holds_peak`] says so), in a field of 3 to 5 cells.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RpmDigits {
    peak: PeakWindow,
    holding: bool,
    /// Time since the rpm was last in the shift point's latch band (s), counted up to
    /// [`ARC_HOLD_S`].
    since_reached: f32,
    digits: Hysteresis,
    /// The rpm the band and the digits follow: the window's peak while held, else the live rpm.
    held: f32,
    /// The rpm shown, and the field's cells.
    shown: i64,
    cells: u8,
}

impl RpmDigits {
    fn new() -> Self {
        Self {
            peak: PeakWindow::new(),
            holding: false,
            since_reached: ARC_HOLD_S,
            digits: Self::start(0.0),
            held: 0.0,
            shown: 0,
            cells: 3,
        }
    }

    /// The digits' hysteresis, started at `rpm`.
    fn start(rpm: f32) -> Hysteresis {
        Hysteresis::new(rpm / 10.0, RPM_HYST_TENS)
    }

    /// Feeds the live rpm, the lights' latch and whether the rpm is in the latch's band.
    fn update(&mut self, rpm: f32, over: bool, near: bool, dt: f32, snap: bool) {
        self.since_reached = if near {
            0.0
        } else if snap {
            ARC_HOLD_S
        } else {
            (self.since_reached + dt).min(ARC_HOLD_S)
        };
        let peak = self.peak.update(rpm, dt, snap);
        self.holding = holds_peak(over, self.holding && !snap, self.since_reached, peak, rpm);
        let held = if self.holding { peak } else { rpm };
        self.held = held;
        if snap {
            self.digits = Self::start(held);
        }
        self.shown = saturate_to_cells(self.digits.update(held / 10.0) * 10, RPM_CELLS).max(0);
        self.cells = rpm_cells(self.cells, self.shown, snap);
    }
}

/// Everything the rpm drives: the shift lights, the rev band's colour and the rpm digits, fed in
/// that order from one place.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Rev {
    leds: TopLeds,
    /// The rev band's colour over the shift point.
    arc: ArcInk,
    digits: RpmDigits,
}

impl Rev {
    fn new() -> Self {
        Self {
            leds: TopLeds::new(),
            arc: ArcInk::new(),
            digits: RpmDigits::new(),
        }
    }

    fn update(&mut self, sig: &Signals, pal: &Palette, dt: f32, calm: bool, snap: bool) {
        let was_over = self.leds.over;
        self.leds.update(sig, pal, dt, calm, snap);
        let over = self.leds.over;
        self.arc.update(over, over && !was_over, dt, snap);
        let rpm = sig.rpm.max(0.0);
        let near = limiter_known(sig) && rpm >= sig.shift_rpm - SHIFT_HYSTERESIS_RPM;
        self.digits.update(rpm, over, near, dt, snap);
    }
}

/// The readouts held by hysteresis besides the rpm: the total G in hundredths, power in the
/// user's unit and boost in tenths of a psi, each in the unit it is shown in.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Readouts {
    g_total: Hysteresis,
    power: Hysteresis,
    boost: Hysteresis,
}

impl Readouts {
    /// Started at these values (zeros for a new cluster, the car's state on a snapped frame).
    fn start(g_centi: f32, power: f32, boost_tenths: f32) -> Self {
        Self {
            g_total: Hysteresis::new(g_centi, G_HYST_CENTI),
            power: Hysteresis::new(power, POWER_HYST_MIN),
            boost: Hysteresis::new(boost_tenths, BOOST_HYST_TENTHS),
        }
    }
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
    opts: Options,
    /// The `BoschDduPanels` text `opts` was parsed from (parsed again only when it changes).
    opts_text: String,
    /// The shift lights, the rev band's colour and the rpm digits.
    rev: Rev,
    /// Side columns lit: left (brake), right (slip).
    brake_lit: usize,
    slip_lit: usize,
    gear: HeroGear,
    flare: Tween,
    speed: TabularSpeed,
    gball: GBall,
    /// The total G, power and boost as held by hysteresis, and as shown.
    readouts: Readouts,
    g_shown: i64,
    power_shown: i64,
    boost_shown: i64,
    /// The custom name alone, for the chin's left zone (kept in step with the frame's logo).
    name: Logo,
    light: VirtualLight,
    /// True while the virtual light moved on the last update (its spring still settling).
    light_moving: bool,
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
            opts: Options::default(),
            opts_text: String::new(),
            rev: Rev::new(),
            brake_lit: 0,
            slip_lit: 0,
            gear: HeroGear::new(t),
            flare: Tween::new(FLARE_S),
            speed: TabularSpeed::new(),
            gball: GBall::new(t),
            readouts: Readouts::start(0.0, 0.0, 0.0),
            g_shown: 0,
            power_shown: 0,
            boost_shown: 0,
            name: Logo::default(),
            light: VirtualLight::new(t),
            light_moving: false,
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The scale's top (rpm): the limiter rounded up to a thousand ([`DEFAULT_SCALE_TOP`] when
    /// none is known).
    fn scale_top(&self) -> f32 {
        if limiter_known(&self.sig) {
            (self.sig.max_rpm / 1000.0).ceil().clamp(1.0, 20.0) * 1000.0
        } else {
            DEFAULT_SCALE_TOP
        }
    }

    /// The angle of `rpm` on the rev gauge.
    fn rev_angle(&self, g: &Geom, rpm: f32) -> f32 {
        g.rev_start + g.rev_sweep * (rpm / self.scale_top()).clamp(0.0, 1.0)
    }
}

/// The slip column's count for `value` in steps of `step`, latched so a value sitting on a step
/// never flickers: it rises at a step and falls only `hyst` under it.
fn latched(lit: usize, value: f32, step: f32, hyst: f32, max: usize) -> usize {
    let v = value.abs();
    let up = ((v / step).floor() as usize).min(max);
    if up >= lit {
        up
    } else {
        (((v + hyst) / step).floor() as usize).min(lit)
    }
}

/// The brake column's count for `brake` (0..=1): one LED per started fifth above the dead zone,
/// latched so a pedal resting on a fifth never flickers: an LED lights as its fifth starts and
/// goes out only [`BRAKE_HYST`] of a step under it.
fn brake_latched(lit: usize, brake: f32) -> usize {
    // The dead zone is latched as well: a lit first LED holds a little under it.
    let dead = if lit > 0 {
        BRAKE_DEAD * (1.0 - BRAKE_HYST)
    } else {
        BRAKE_DEAD
    };
    if brake < dead {
        return 0;
    }
    let steps = brake.clamp(0.0, 1.0) * SIDE_LEDS as f32;
    let up = (steps.ceil() as usize).min(SIDE_LEDS);
    if up >= lit {
        up
    } else {
        ((steps + BRAKE_HYST).ceil() as usize).min(lit)
    }
}

/// A G ball at rest on the force of `sig`, its trail collapsed onto it, for a snapped frame: a
/// new ball starts in the centre and would drift to the force over the next frames. (The shared
/// `GBall` has no snap of its own; this settles it on its own spring, on the stack.)
fn settled_gball(tokens: &Tokens, sig: &Signals) -> GBall {
    let mut ball = GBall::new(tokens);
    for _ in 0..SETTLE_STEPS {
        ball.update(sig, SETTLE_DT);
    }
    ball
}

/// The virtual light at rest on the lateral G of `sig`, for a snapped frame, as
/// [`settled_gball`] puts the ball at rest.
fn settled_light(tokens: &Tokens, sig: &Signals) -> VirtualLight {
    let mut light = VirtualLight::new(tokens);
    for _ in 0..SETTLE_STEPS {
        light.update(sig.g_lat, SETTLE_DT);
    }
    light
}

/// The rpm field's digit cells: as many as the value needs (3 to 5), so the centred digits stay
/// on the gauge's axis. The field grows at once and shrinks only 50 rpm under the step, so it
/// moves rarely.
fn rpm_cells(cells: u8, shown: i64, snap: bool) -> u8 {
    let need = match shown {
        ..=999 => 3,
        1000..=9999 => 4,
        _ => 5,
    };
    let held = match cells {
        5 => shown > 9949,
        4 => shown > 949,
        _ => false,
    };
    if snap || need >= cells || !held {
        need
    } else {
        cells
    }
}

impl Cluster for BoschDdu {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        let race_edge = sig.race_on != self.sig.race_on;
        // A gap on the cluster's clock or in `dt`: the clock sees it whatever the host's cap.
        let gap = input.dt >= RESUME_DT;
        let snap = self.fresh || race_edge || gap;
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
            // The host empties the logo when `ShowLogo` is off, so the name goes with it.
            self.name.custom_name.clone_from(&input.logo.custom_name);
        }
        let panels = input
            .config
            .extra
            .get("BoschDduPanels")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if panels != self.opts_text {
            panels.clone_into(&mut self.opts_text);
            self.opts = Options::parse(panels);
        }

        // The gear first. Snapped frames show the engaged gear at rest.
        if snap {
            self.gear.snap(sig.gear);
            self.flare = Tween::new(FLARE_S);
        } else {
            let changed = sig.gear != self.gear.gear();
            self.gear.update(sig.gear, 0.0, dt);
            if changed && !self.calm {
                self.flare.restart();
            }
        }
        self.flare.update(dt);

        // Shift lights, the rev band's colour and the rpm digits (over the shift point and on a
        // limiter bounce they and the band show the peak of the last 0.3 s, so a bounce never
        // moves them), then the side columns.
        self.rev.update(&sig, &self.palette, dt, self.calm, snap);
        if snap {
            self.slip_lit = 0;
            self.brake_lit = 0;
        }
        self.slip_lit = latched(self.slip_lit, sig.slip_deg, SLIP_STEP_DEG, SLIP_HYST_DEG, SIDE_LEDS);
        self.brake_lit = brake_latched(self.brake_lit, sig.brake);

        // Speed, G (the shared ball, on the tokens' spring), power and boost.
        self.speed.update(sig.speed);
        if snap {
            self.gball = settled_gball(input.tokens, &sig);
        }
        self.gball.update(&sig, dt);
        let g_total = sig.g_lat.hypot(sig.g_long) * 100.0;
        let watts = input.telemetry.power_watts.max(0.0);
        let power = match sig.power_unit {
            PowerUnit::Hp => watts / 745.7,
            PowerUnit::Kw => watts / 1000.0,
        };
        // Boost in tenths of a psi, the packet's unit, whatever the speed unit.
        let boost = input.telemetry.boost_psi.clamp(-15.0, 60.0) * 10.0;
        if snap {
            self.readouts = Readouts::start(g_total, power, boost);
        }
        let r = &mut self.readouts;
        self.g_shown = saturate_to_cells(r.g_total.update(g_total), G_CELLS).max(0);
        r.power.threshold = (self.power_shown as f32 * POWER_HYST_SHARE).max(POWER_HYST_MIN);
        self.power_shown = saturate_to_cells(r.power.update(power), POWER_CELLS).max(0);
        self.boost_shown = r.boost.update(boost);

        // The virtual light, put at rest on a snapped frame like the G ball.
        if snap {
            self.light = settled_light(input.tokens, &sig);
        }
        let before = self.light.angle();
        self.light.update(sig.g_lat, dt);
        self.light_moving = (self.light.angle() - before).abs() > 1e-4;
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &GEOM;
        let pal = &self.palette;
        p.scoped(|p| {
            if self.view == View::Hood {
                p.scale(HOOD_SCALE);
            }
            let w = hardware::light_weights(self.light.angle());
            hardware::contact_shadow(p, g, pal);
            if !self.sprites.draw(p, w) {
                hardware::fallback(p, g, pal, w[2] - w[1]);
            }
            // The screen is true black whatever the housing under it (a render's glass leaves up
            // to 0.004 there).
            p.fill(&Shape::rect(g.screen), pal.roles.oled_black);
            self.draw_leds(p, g, pal);
            Self::draw_horizon(p, g, pal);
            self.draw_rev(p, g, pal);
            self.draw_gear(p, g, pal);
            for (i, &content) in self.opts.panels.iter().enumerate() {
                if !shows(self.view, Part::Panel(content)) {
                    continue;
                }
                let slot = g.slot(i);
                match content {
                    Content::Pedals => self.draw_pedals(p, g, slot, pal),
                    Content::GForce => self.draw_g(p, g, slot, pal),
                    Content::Engine => self.draw_engine(p, g, slot, pal),
                    Content::Speed => self.draw_speed(p, g, slot, pal),
                    Content::Off => {}
                }
            }
            self.draw_chin(p, g, pal);
        });
    }

    fn animating(&self) -> bool {
        // The G ball counts only where it is drawn (its trail is never drawn here).
        let ball_drawn = shows(self.view, Part::Panel(Content::GForce)) && self.opts.panels.contains(&Content::GForce);
        self.rev.leds.animating()
            || self.rev.arc.animating(self.rev.leds.over)
            || self.gear.animating()
            || self.flare.running()
            || (ball_drawn && self.gball.animating())
            || self.light_moving
    }
}

/// The ink of the printed parts: [`PRINT_SHARE`] of the label role.
fn print_ink(pal: &Palette) -> Color {
    pal.roles.ink_label.fade(PRINT_SHARE)
}

/// The light of `color` over black: its relative luminance at its opacity.
fn light(color: Color) -> f32 {
    color.luminance() * color.a
}

/// The palette's text-safe accent at the printed parts' light, for the one label that follows
/// its value's colour (BOOST).
fn accent_print(pal: &Palette) -> Color {
    let accent = pal.primary_text;
    accent.fade((light(print_ink(pal)) / light(accent).max(1e-3)).min(1.0))
}

/// The hairlines' paint: [`HAIRLINE_SHARE`] of the `hairline` role, opaque.
fn hairline(pal: &Palette) -> Color {
    solid(pal.roles.hairline.fade(HAIRLINE_SHARE))
}

/// The horizon's band of light: none `band` px over the line, `level` of `primary` at the line
/// (held under the static cap).
fn horizon_glow(g: &Geom, band: f32, level: f32, pal: &Palette) -> Gradient {
    Gradient::linear(vec2(0.0, g.horizon - band), vec2(0.0, g.horizon))
        .stop(0.0, pal.primary.with_alpha(0.0))
        .stop(1.0, static_tint(pal.primary, level))
}

/// The tint of the header rules and the horizon line.
fn rule_tint(pal: &Palette) -> Color {
    solid(static_tint(pal.primary, RULE_TINT))
}

/// The red zone on the scale ring and the limiter's tick.
fn red_zone(pal: &Palette) -> Color {
    solid(static_tint(pal.roles.warning, RED_LEVEL))
}

/// The outline of the box round the shift point.
fn shift_box(pal: &Palette) -> Color {
    static_tint(pal.primary, SHIFT_BOX)
}

/// The gear beam at rest: the alpha of its body at the apex, and its edge bands' colour (in
/// `warning` in reverse) with their alpha, before the flare. The parts are capped as they stack
/// at the apex (the beam is drawn from under the horizon rule, see `draw_gear`, so it lies on
/// black): the body, then an edge over it, stays under [`STATIC_CAP`] (source-over), not only
/// each part on its own.
fn beam_rest(pal: &Palette, reverse: bool) -> (f32, Color, f32) {
    let edge = if reverse { pal.roles.warning } else { pal.primary };
    let apex = static_alpha(pal.primary, BEAM_APEX);
    let body = beam_stack(pal.primary.luminance(), apex, 0.0);
    (apex, edge, stacked_alpha(edge, BEAM_EDGE, body))
}

/// `color`'s alpha for a static part laid over a background of light `under`: `alpha`, or less
/// where the result would pass [`STATIC_CAP`]. A colour no lighter than the background cannot
/// lift it, so it keeps `alpha` (under the part's own cap).
fn stacked_alpha(color: Color, alpha: f32, under: f32) -> f32 {
    let own = static_alpha(color, alpha);
    let lift = color.luminance() - under;
    if lift <= 1e-4 {
        return own;
    }
    own.min(((STATIC_CAP - under) / lift).max(0.0))
}

/// The light of a layer of luminance `l` at alpha `a` over a background of light `under`.
fn beam_stack(l: f32, a: f32, under: f32) -> f32 {
    a * l + (1.0 - a) * under
}

/// The G ring's unlit track: [`G_TRACK_SHARE`] of `ink_ghost`.
fn g_track(pal: &Palette) -> Color {
    pal.roles.ink_ghost.fade(G_TRACK_SHARE)
}

/// `color` as an opaque stroke of the same light over black, so strokes that cross or join show
/// no brighter spot (the screen under them is black).
fn solid(color: Color) -> Color {
    Color { a: 1.0, ..color }
}

impl BoschDdu {
    /// True when the fine print is drawn: not in the hood view ([`shows`]).
    fn fine_print(&self) -> bool {
        shows(self.view, Part::Print)
    }

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

    /// One LED lens: lit as one soft oval of light with a hot centre ([`lens_core`]), a 1 px
    /// inner rim and a tight halo ([`led_halo`]), or an unlit lens in `ink_ghost` (left to the housing render when it loaded: its
    /// lenses are the real hardware). Only the shift lights (`bloom`, rank 1) feed the bloom.
    fn led(&self, p: &mut Painter<'_>, rect: Rect, lit: Option<Color>, core: f32, bloom: bool, pal: &Palette) {
        let r = rect.width().min(rect.height()) * 0.5;
        let shape = Shape::round_rect(rect, r);
        match lit {
            Some(color) => {
                if let Some(halo) = led_halo(p.tokens()) {
                    p.glow(&shape, color, halo);
                }
                // The hot centre and the rim fade with the lens, so a breathing LED (reduced
                // flash) breathes as a whole.
                let hot = lens_core(color, core, pal);
                let lens = Gradient::radial(rect.center(), rect.width() * 0.5)
                    .stop(0.0, hot)
                    .stop(1.0, color);
                if bloom {
                    p.layer(LayerOptions::bloom(), |p| p.fill(&shape, lens));
                } else {
                    p.fill(&shape, lens);
                }
                p.stroke(&shape, pal.roles.ink.with_alpha(LED_RIM * color.a), Stroke::rim_px(1.0));
            }
            None if self.sprites.loaded() => {}
            None => p.fill(&shape, pal.roles.ink_ghost),
        }
    }

    /// The brow's shift lights and the two side columns.
    fn draw_leds(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let leds = &self.rev.leds;
        let core = leds.core();
        for i in 0..TOP_LEDS {
            self.led(p, g.top_led_rect(i), leds.color_of(i, pal), core, true, pal);
        }
        for i in 0..SIDE_LEDS {
            // The brake on the left, the side its bar is on. Each column keeps one colour, and
            // slip is a caution, never red: red is kept for the limiter and the brake.
            let brake = (i < self.brake_lit).then_some(pal.roles.warning);
            self.led(p, g.side_led_rect(-1.0, i), brake, LED_CORE, false, pal);
            let slip = (i < self.slip_lit).then_some(pal.roles.caution);
            self.led(p, g.side_led_rect(1.0, i), slip, LED_CORE, false, pal);
        }
    }

    /// The horizon over both panel columns: a tinted line with a soft band of light over it (the
    /// rev gauge's disc draws its own stretch).
    fn draw_horizon(p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let glow = horizon_glow(g, HORIZON_BAND, HORIZON_GLOW, pal);
        for (x0, x1) in [g.left, g.right] {
            let band = Rect::from_points(vec2(x0, g.horizon - HORIZON_BAND), vec2(x1, g.horizon));
            p.fill(&Shape::rect(band), glow);
            p.fill(
                &Shape::rect(Rect::from_points(vec2(x0, g.horizon - 1.0), vec2(x1, g.horizon + 1.0))),
                rule_tint(pal),
            );
        }
    }

    /// The round rev gauge as the DDU draws it: a hairline scale ring with ticks across it and
    /// numerals outside, the lit band inside the ring with a needle at its head, and the digits
    /// in the disc, which the horizon cuts flat.
    fn draw_rev(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let c = g.rev_c;
        let ink = pal.roles.ink;
        let top = self.scale_top();
        let print = solid(print_ink(pal));
        let rule = hairline(pal);
        let red = red_zone(pal);
        let (ring, disc) = (g.ring_r(), g.disc_r());
        let fine = self.fine_print();
        // The disc: black glass in a thin ring, cut flat by the horizon, whose light gathers in
        // the [`DISC_BAND`] px at its foot: it starts under the "rpm" unit's descender, so the
        // unit stands on black.
        p.scoped(|p| {
            p.clip(
                Rect::from_points(vec2(c.x - disc, c.y - disc), vec2(c.x + disc, g.horizon)),
                0.0,
            );
            p.fill(&Shape::circle(c, disc), horizon_glow(g, DISC_BAND, DISC_GLOW, pal));
            p.stroke(&Shape::circle(c, disc), rule, Stroke::inner(1.5));
        });
        let drop = g.horizon - c.y;
        let half = (disc * disc - drop * drop).max(0.0).sqrt();
        p.fill(
            &Shape::rect(Rect::from_points(
                vec2(c.x - half, g.horizon - 1.0),
                vec2(c.x + half, g.horizon + 1.0),
            )),
            rule_tint(pal),
        );
        // The scale ring, red from the shift point to the scale's end.
        p.fill(&Shape::arc(c, ring, 2.0, g.rev_start, g.rev_sweep, Cap::Butt), print);
        let known = limiter_known(&self.sig);
        let max = self.sig.max_rpm;
        let shift = self.sig.shift_rpm;
        if known && shift < top {
            let a0 = self.rev_angle(g, shift);
            p.fill(
                &Shape::arc(c, ring, RED_BAND, a0, self.rev_angle(g, top) - a0, Cap::Butt),
                red,
            );
        }
        // The band follows the rpm the digits show (the live rpm, or a limiter bounce's peak).
        // Over the shift point it turns ink, so only the LEDs carry the shift colour (the
        // family's rule): a short cross-fade, never a step.
        let rpm = self.rev.digits.held.max(0.0);
        let mix = self.rev.arc.mix;
        if rpm > 0.0 {
            let lit = pal.primary.lerp(ink, mix);
            let head = self.rev_angle(g, rpm);
            let sweep = head - g.rev_start;
            let (r0, r1) = (g.rev_r - g.rev_w * 0.5, g.rev_r + g.rev_w * 0.5);
            // Its body fades towards the disc.
            let body = Gradient::radial(c, r1)
                .stop(0.0, lit.fade(ARC_INNER))
                .stop(r0 / r1, lit.fade(ARC_INNER))
                .stop(1.0, lit.fade(ARC_BODY));
            p.fill(&Shape::arc(c, g.rev_r, g.rev_w, g.rev_start, sweep, Cap::Butt), body);
            // Its edge under the ring is bright, and brightens towards its head as the DDU's
            // does.
            let ramp = Gradient::conic(c, g.rev_start)
                .stop(0.0, lit.fade(ARC_TAIL))
                .stop((sweep / TAU).clamp(0.001, 1.0), lit)
                .stop(1.0, lit);
            let hot = Shape::arc(c, r1 - ARC_HOT * 0.5, ARC_HOT, g.rev_start, sweep, Cap::Butt);
            // Its bloom goes as the band turns ink: an ink band has no bloom layer, so the
            // numerals beside it keep their contrast.
            let bloom = ARC_BLOOM * (1.0 - mix);
            if bloom > 0.0 {
                let glow = LayerOptions {
                    bloom: Some(Bloom {
                        intensity: bloom,
                        ..Bloom::default()
                    }),
                    ..LayerOptions::default()
                };
                p.layer(glow, |p| p.fill(&hot, ramp));
            } else {
                p.fill(&hot, ramp);
            }
            // The needle: a white line at the band's head, from the disc's ring out across the
            // scale ring (never into the disc, where the rpm digits are).
            p.fill(
                &Shape::line(Vec2::polar(c, disc, head), Vec2::polar(c, ring + 6.0, head), 3.0),
                ink,
            );
        }
        // Ticks across the ring every thousand with three between them (one on a scale over
        // 10,000), numerals outside (not in the hood view). From the limiter on a tick is red,
        // wider and reaches further in; outside the ring it ends where the others do.
        let thousands = (top / 1000.0).round() as usize;
        let (step, parts) = if thousands > 10 { (2, 2) } else { (1, 4) };
        let style = self.figures(g.num_cap, print_ink(pal), Align::Center);
        for k in 0..=thousands {
            let at = k as f32 * 1000.0;
            let a = self.rev_angle(g, at);
            let numeral = if fine && k % step == 0 { NUMERALS.get(k) } else { None };
            let (inside, outside, width, tick) = if known && at >= max - 1.0 {
                (LIMIT_TICK_IN, LIMIT_TICK_OUT, LIMIT_TICK_W, red)
            } else {
                (TICK_IN, TICK_OUT, TICK_W, print)
            };
            p.fill(
                &Shape::line(
                    Vec2::polar(c, ring - inside, a),
                    Vec2::polar(c, ring + outside, a),
                    width,
                ),
                tick,
            );
            if k < thousands {
                for part in 1..parts {
                    let a = self.rev_angle(g, at + 1000.0 * part as f32 / parts as f32);
                    p.fill(
                        &Shape::line(Vec2::polar(c, ring, a), Vec2::polar(c, ring + 4.0, a), 1.5),
                        rule,
                    );
                }
            }
            if let Some(label) = numeral {
                p.text(Vec2::polar(c, g.num_r, a), label, &style);
            }
        }
        // The rpm digits and their unit in the disc, centred on the gauge's axis.
        let digits = self.figures(g.rpm_cap, ink, Align::Center);
        let rpm_digits = &self.rev.digits;
        p.number(g.rpm_at, rpm_digits.shown, rpm_digits.cells, &digits, None);
        if fine {
            let unit = self.label(g.unit_cap, print_ink(pal), Align::Center);
            p.text(vec2(g.rpm_at.x, g.rpm_at.y + g.rpm_cap * 0.5 + 15.0), "rpm", &unit);
        }
    }

    /// The gear in its beam, like a road running away from the horizon: light at the apex that
    /// fades out towards the foot, and a band of light inside each edge. Every gear is ink;
    /// reverse shows in the beam's two edge bands, which turn `warning`.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let beam = g.beam();
        let flare = if self.flare.running() {
            1.0 + FLARE_PEAK * (1.0 - self.flare.progress())
        } else {
            1.0
        };
        let foot = g.screen.max.y;
        let reverse = self.gear.gear() == Gear::Reverse;
        let (apex_rest, edge_ink, edge_rest) = beam_rest(pal, reverse);
        let apex = (apex_rest * flare).min(1.0);
        let body = Gradient::linear(vec2(0.0, g.horizon), vec2(0.0, foot))
            .stop(0.0, pal.primary.with_alpha(apex))
            .stop(0.45, pal.primary.with_alpha(apex * 0.25))
            .stop(1.0, pal.primary.with_alpha(0.0));
        // The beam starts under the horizon rule's bottom edge, so its light never stacks on the
        // rule's (the static cap holds for the stack, `beam_rest`).
        p.save();
        p.clip(
            Rect::from_points(vec2(g.screen.min.x, g.horizon + 1.0), g.screen.max),
            0.0,
        );
        p.fill(&Shape::Polygon { points: &beam }, body);
        // The edge bands: each a strip inside its edge, at one strength along the edge and
        // fading inwards along the edge's normal.
        let edge = (edge_rest * flare).min(1.0);
        let along = (beam[2] - beam[1]).normalize_or_zero();
        for (side, top, base) in [(1.0, beam[1], beam[2]), (-1.0, beam[0], beam[3])] {
            // The inward normal of this edge, and the strip's width along x.
            let normal = vec2(-side * along.y, along.x);
            let across = vec2(-side * BEAM_BAND / along.y.max(0.1), 0.0);
            let strip = [top, base, base + across, top + across];
            let fade = Gradient::linear(top, top + normal * BEAM_BAND)
                .stop(0.0, edge_ink.with_alpha(edge))
                .stop(1.0, edge_ink.with_alpha(0.0));
            p.fill(&Shape::Polygon { points: &strip }, fade);
        }
        p.restore();
        // The shared glyph's own colours are left out: its red R would stand over the beam's
        // colour, so reverse is ink like every gear (neutral at the glyph's 72 %).
        let ink = if reverse { pal.roles.ink } else { self.gear.color(pal) };
        // The clip keeps the glyph itself on the screen while it slides in on a gear change. It
        // cannot bound the bloom, which the kit adds over the whole frame: [`GEAR_BLOOM`] keeps
        // that light low instead.
        let glow = LayerOptions {
            bloom: Some(Bloom {
                intensity: GEAR_BLOOM,
                ..Bloom::default()
            }),
            ..LayerOptions::default()
        };
        p.scoped(|p| {
            p.clip(g.screen, 0.0);
            p.layer(glow, |p| {
                self.gear.draw_with(p, g.gear_at, self.faces.gear, g.gear_cap, ink);
            });
        });
    }

    /// A quarter panel's caps header, at the screen's edge of its panel (the bottom right one at
    /// the gauge's side, as the DDU sets SPEED). A top panel's header stands on a tinted rule
    /// that runs from the screen's edge and turns up at 45° at the gauge; a bottom panel stands
    /// under the horizon, which is its rule.
    fn header(&self, p: &mut Painter<'_>, g: &Geom, s: Slot, text: &str, pal: &Palette) {
        if !self.fine_print() {
            return;
        }
        let (x, y) = (s.x, s.head);
        let (at, align) = if s.right_side && s.top {
            (x.1, Align::Right)
        } else {
            (x.0, Align::Left)
        };
        p.text(
            vec2(at, y + LABEL_NUDGE),
            text,
            &self.label(g.head_cap, print_ink(pal), align),
        );
        if !s.top {
            return;
        }
        // A right-hand panel has the gauge on its left.
        let (edge_end, gauge_end, dir) = if s.right_side {
            (x.1, x.0, -1.0)
        } else {
            (x.0, x.1, 1.0)
        };
        let ry = y + g.head_cap * 0.5 + RULE_DROP;
        let rule = rule_tint(pal);
        p.fill(&Shape::line(vec2(edge_end, ry), vec2(gauge_end, ry), 2.0), rule);
        p.fill(
            &Shape::line(vec2(gauge_end, ry), vec2(gauge_end + dir * KICK, ry - KICK), 2.0),
            rule,
        );
    }

    /// A horizontal bar as the DDU draws it: a strip of fine lines between two hairlines, lit up
    /// to `value` (0..=1), with a taller ink marker at the value.
    fn bar(p: &mut Painter<'_>, x: (f32, f32), y: f32, h: f32, value: f32, lit: Color, pal: &Palette) {
        let rule = hairline(pal);
        let w = x.1 - x.0;
        p.fill(&Shape::rect(Rect::from_xywh(x.0, y, w, 1.0)), rule);
        p.fill(&Shape::rect(Rect::from_xywh(x.0, y + h - 1.0, w, 1.0)), rule);
        let v = value.clamp(0.0, 1.0);
        let pitch = (w - HATCH_W) / (HATCH as f32 - 1.0);
        for i in 0..HATCH {
            let on = (i as f32 + 0.5) / HATCH as f32 <= v;
            let line = Rect::from_xywh(x.0 + i as f32 * pitch, y + 3.0, HATCH_W, h - 6.0);
            p.fill(&Shape::rect(line), if on { lit } else { pal.roles.ink_ghost });
        }
        if v > 0.0 {
            let marker = Rect::from_xywh(
                x.0 + v * (w - HATCH_W) - 0.5,
                y - MARKER_OVER,
                HATCH_W + 1.0,
                h + 2.0 * MARKER_OVER,
            );
            p.fill(&Shape::rect(marker), pal.roles.ink);
        }
    }

    /// PEDALS: throttle and brake.
    fn draw_pedals(&self, p: &mut Painter<'_>, g: &Geom, s: Slot, pal: &Palette) {
        self.header(p, g, s, "PEDALS", pal);
        let label = self.row_label(g.label_cap, print_ink(pal));
        let (thr, brk) = (s.head + g.thr_dy, s.head + g.brk_dy);
        p.text(vec2(s.x.0, thr - BAR_LABEL), "THR", &label);
        Self::bar(p, s.x, thr, g.bar_h, self.sig.throttle, pal.primary, pal);
        p.text(vec2(s.x.0, brk - BAR_LABEL), "BRK", &label);
        Self::bar(p, s.x, brk, g.bar_h, self.sig.brake, pal.roles.warning, pal);
    }

    /// G-FORCE (rank 4: no bloom), in the form of the DDU's fuel ring: a full ring, filled
    /// clockwise from 12 o'clock by the felt force of the shared `GBall` (its spring, its
    /// felt-force rule and its hold at the limit), a bead on the ring at the force's bearing
    /// (any bearing: the ring has no gap), and the total G with its unit inside.
    fn draw_g(&self, p: &mut Painter<'_>, g: &Geom, s: Slot, pal: &Palette) {
        self.header(p, g, s, "G-FORCE", pal);
        let c = vec2((s.x.0 + s.x.1) * 0.5, s.head + g.g_dy);
        p.fill(&Shape::ring(c, g.g_r, G_BAND), g_track(pal));
        let ball = self.gball.ball();
        let force = ball.length();
        let share = (force / G_FULL).clamp(0.0, 1.0);
        if share > 0.0 {
            let lit = Shape::arc(c, g.g_r, G_BAND, G_START, TAU * share, Cap::Butt);
            p.fill(&lit, pal.primary);
        }
        // The bead fades in with the force: a force near zero has no bearing to show.
        let bead = (force / G_BEAD_FULL).clamp(0.0, 1.0);
        if bead > 0.0 {
            let at = c + ball.normalize_or_zero() * g.g_r;
            p.fill(&Shape::circle(at, G_BEAD), pal.roles.ink.fade(bead));
        }
        let value = self.figures(g.g_value_cap, pal.roles.ink, Align::Center);
        draw_fixed(p, vec2(c.x, c.y - 4.0), self.g_shown, 2, &value);
        let unit = self.label(g.unit_cap, print_ink(pal), Align::Center);
        p.text(vec2(c.x, c.y + g.g_value_cap * 0.5 + 11.0), "g", &unit);
    }

    /// A row's label with its unit after it in lower case, on the line `at` (nudged by
    /// [`LABEL_NUDGE`]), in `ink`.
    fn row(&self, p: &mut Painter<'_>, g: &Geom, at: Vec2, label: &str, unit: &str, ink: Color) {
        let style = self.row_label(g.label_cap, ink);
        let w = p.measure(label, &style).width;
        let y = at.y + LABEL_NUDGE;
        p.text(vec2(at.x, y), label, &style);
        p.text(vec2(at.x + w + ROW_GAP, y), unit, &style);
    }

    /// ENGINE: power in the user's unit, boost in psi (its label is the panel's accent line, as
    /// the DDU colours one line of its list; its value is ink, which reads at a glance), and the
    /// shift point boxed.
    fn draw_engine(&self, p: &mut Painter<'_>, g: &Geom, s: Slot, pal: &Palette) {
        let (x0, x1) = s.x;
        self.header(p, g, s, "ENGINE", pal);
        let power_unit = match self.sig.power_unit {
            PowerUnit::Hp => "hp",
            PowerUnit::Kw => "kW",
        };
        self.row(p, g, vec2(x0, s.head + g.row_dy), "POWER", power_unit, print_ink(pal));
        let big = self.figures(g.power_cap, pal.roles.ink, Align::Right);
        p.number(vec2(x1, s.head + g.power_dy), self.power_shown, POWER_CELLS, &big, None);

        // Boost: the label and its unit on the left in the text-safe accent at the printed
        // parts' light, the value on the right in ink (vacuum reads 0).
        let boost_y = s.head + g.boost_dy;
        self.row(p, g, vec2(x0, boost_y), "BOOST", "psi", accent_print(pal));
        let small = self.figures(g.value_cap, pal.roles.ink, Align::Right);
        draw_fixed(p, vec2(x1, boost_y), self.boost_shown.max(0), 1, &small);

        // The shift point in an outlined box, as the DDU boxes its key value. It never changes
        // while a car is driven, so it is printed at the labels' level, on black.
        let (y0, y1) = g.shift_row(s);
        let cy = (y0 + y1) * 0.5;
        let print = print_ink(pal);
        p.text(vec2(x0, cy + LABEL_NUDGE), "SHIFT", &self.row_label(g.label_cap, print));
        let shift = saturate_to_cells((self.sig.shift_rpm / 10.0).round() as i64 * 10, RPM_CELLS).max(0);
        let (cells, field) = if shift > 9999 { (5, "00000") } else { (4, "0000") };
        let printed = self.figures(g.value_cap, print, Align::Right);
        let w = p.measure(field, &printed).width + 2.0 * SHIFT_PAD;
        let boxed = Shape::round_rect(Rect::from_points(vec2(x1 - w, y0), vec2(x1, y1)), 4.0);
        p.stroke(&boxed, shift_box(pal), Stroke::inner(2.0));
        p.number(vec2(x1 - SHIFT_PAD, cy), shift, cells, &printed, None);
    }

    /// SPEED: tabular digits flush right, as every value in a panel is (no ghost 8s: at a glance
    /// a ghost read as a digit), and the unit in lower case under their last digit.
    fn draw_speed(&self, p: &mut Painter<'_>, g: &Geom, s: Slot, pal: &Palette) {
        self.header(p, g, s, "SPEED", pal);
        let digits = self.figures(g.speed_cap, pal.roles.ink, Align::Right);
        p.number(vec2(s.x.1, s.head + g.speed_dy), self.speed.value(), 3, &digits, None);
        if !self.fine_print() {
            return;
        }
        let unit = self.label(g.unit_cap, print_ink(pal), Align::Right);
        let text = match self.sig.unit {
            SpeedUnit::Mph => "mph",
            SpeedUnit::Kmh => "km/h",
        };
        p.text(vec2(s.x.1, s.head + g.unit_dy), text, &unit);
    }

    /// The chin: the custom name in the left zone, the car's logo zone on the right, both
    /// printed (at the labels' level; a logo image at [`LOGO_SHARE`]). The name is the wordmark
    /// of the tokens' logo zone (the `wide` face, fitted to its zone by the kit) and goes when
    /// the logo is switched off. The hood view keeps a logo image only: a wordmark there would be
    /// under the size floor.
    fn draw_chin(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        if !self.fine_print() {
            if let Some(image) = self.logo.image {
                // The image alone (empty names allocate nothing), so no wordmark stands in.
                let only = Logo {
                    image: Some(image),
                    ..Logo::default()
                };
                p.scoped(|p| {
                    p.set_opacity(LOGO_SHARE);
                    logo_zone(p, g.logo, &only, self.faces.wide, LogoVariant::Full, pal);
                });
            }
            return;
        }
        // While the car's logo is still loading the logo zone stays empty, so no stand-in shows
        // and then swaps; the name keeps its own zone.
        let loading = self.logo.loading;
        // Without a logo image or brand the logo zone shows the custom name itself: never twice.
        let zone_shows_name = !loading && self.logo.image.is_none() && self.logo.brand_name.trim().is_empty();
        if !zone_shows_name {
            p.scoped(|p| {
                p.set_opacity(PRINT_SHARE);
                let style = LogoStyle {
                    text: pal.roles.ink_label.into(),
                    ..LogoStyle::new(self.faces.wide)
                };
                fh6_kit::logo::draw(p, g.name, &self.name, &style);
            });
        }
        if loading {
            return;
        }
        p.scoped(|p| {
            p.set_opacity(if self.logo.image.is_some() {
                LOGO_SHARE
            } else {
                PRINT_SHARE
            });
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
        assert!(g.rev_c.x - outer > g.left.1, "numerals clear of the left panels");
        assert!(g.rev_c.x + outer < g.right.0, "numerals clear of the right panels");
        // The horizon cuts the disc under the rpm unit and over the band's ends, and the beam
        // starts on it inside the gauge's open bottom, clear of the band's ends.
        // The unit's descender (5 px under its cap foot) ends over the disc's band of light.
        let unit_foot = g.rpm_at.y + g.rpm_cap * 0.5 + 15.0 + g.unit_cap * 0.5;
        assert!(
            unit_foot + 5.0 <= g.horizon - DISC_BAND,
            "the unit over the disc's light"
        );
        assert!(g.horizon < g.rev_c.y + g.disc_r(), "the horizon cuts the disc");
        let end = Vec2::polar(g.rev_c, g.ring_r(), g.rev_start);
        assert!(end.y > g.horizon, "the band starts under the horizon");
        let beam = g.beam();
        let at_end =
            beam[0].x.abs() + (beam[3].x.abs() - beam[0].x.abs()) * ((end.y - g.horizon) / (s.max.y - g.horizon));
        assert!(at_end + 16.0 < end.x.abs(), "beam clear of the band's end");
        assert!(
            beam[2].x < g.right.0 && beam[3].x > g.left.1,
            "beam clear of the panels"
        );
        // The gear (at its 1.12 entry) stays on the screen and under the horizon (its width
        // against the beam's is measured in `panel_rows_fit_the_narrowest_slot`).
        assert!(g.gear_at.y + g.gear_cap * 0.56 <= s.max.y, "gear on the screen");
        assert!(g.gear_at.y - g.gear_cap * 0.56 >= g.horizon, "gear under the horizon");
        // The top row's header rules end clear of the numerals, on the screen.
        let rule_y = g.head_top + g.head_cap * 0.5 + RULE_DROP;
        assert_eq!(rule_y.fract(), 0.0, "the 2 px rule on whole pixels");
        // Cap-15 labels sit half a pixel up, so their cap edges land on whole pixels, and THR
        // and BRK share their lines with POWER and BOOST.
        for head in [g.head_top, g.head_bottom] {
            for line in [head, head + g.row_dy, head + g.boost_dy, head + g.shift_dy] {
                let edge = line + LABEL_NUDGE - g.label_cap * 0.5;
                assert_eq!(edge.fract(), 0.0, "a label's cap edge at {edge}");
            }
            assert_eq!(
                head + g.row_dy + LABEL_NUDGE,
                head + g.thr_dy - BAR_LABEL,
                "THR on POWER's line"
            );
            assert_eq!(
                head + g.boost_dy + LABEL_NUDGE,
                head + g.brk_dy - BAR_LABEL,
                "BRK on BOOST's line"
            );
        }
        assert_eq!(g.head_cap, g.label_cap, "headers take the labels' nudge");
        // The limiter's tick: wider and longer than the others, its round outer end where theirs
        // is, so it stands no closer to the numeral beside it.
        assert_eq!(LIMIT_TICK_OUT + LIMIT_TICK_W * 0.5, TICK_OUT + TICK_W * 0.5);
        const { assert!(LIMIT_TICK_W > TICK_W && LIMIT_TICK_IN + LIMIT_TICK_OUT > TICK_IN + TICK_OUT) };
        assert!(
            g.ring_r() - LIMIT_TICK_IN - LIMIT_TICK_W * 0.5 > g.disc_r(),
            "the limiter's tick stays out of the disc"
        );
        let kick_end = vec2(g.left.1 + KICK, rule_y - KICK);
        assert!(kick_end.y > s.min.y, "the rule's end on the screen");
        assert!(
            (kick_end - g.rev_c).length() > outer,
            "the rule's end clear of the numerals"
        );
        assert!(
            s.max.y < g.logo.min.y && g.logo.max.y < g.bottom,
            "logo zone on the chin"
        );
        assert!(g.logo.max.x < g.half_w - g.corner_bottom, "logo clear of the corner");
        assert!(
            g.shift_row(g.slot(2)).1 < g.horizon - HORIZON_BAND,
            "the shift box clears the horizon's light"
        );
        // ISO 15008 minimums: every dynamic readout at least 17.7 px cap.
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
        // The recorded exception to the family's zone minimums (150, 84, 720; the module doc):
        // the real DDU 10's proportions lead, and these three sizes are held where they are.
        assert_eq!(g.gear_cap, 132.0);
        assert_eq!(g.speed_cap, 58.0);
        assert_eq!(g.top_led_x[TOP_LEDS - 1] - g.top_led_x[0] + g.top_led.x, 426.0);
        // Labels and units: 16 arcminutes at 0.9 m is 14.1 px (the review's standards table);
        // the tokens' smallest size is 12. The sizes that are the tokens' own stay bound to them.
        let t = Tokens::default();
        for cap in [g.head_cap, g.label_cap, g.unit_cap] {
            assert!(cap >= 14.1 && cap >= t.type_scale.nano);
        }
        assert_eq!(g.unit_cap, t.type_scale.micro_label);
        assert_eq!(g.num_cap, t.type_scale.unit);
        // The rank order in size: gear, then speed, then the rpm digits.
        assert!(g.gear_cap > g.speed_cap && g.speed_cap > g.rpm_cap);
        // The chin's zones mirror about the centreline.
        assert_eq!(g.name.min.x, -g.logo.max.x);
        assert_eq!(g.name.max.x, -g.logo.min.x);
    }

    #[test]
    fn every_panel_fits_every_slot_vertically() {
        let g = &GEOM;
        // The tallest content of each kind ends above the horizon's band of light (top row) or
        // the screen's foot (bottom row), whichever slot it is put in.
        // The G ring stays under the header's rule, and its value and unit inside it.
        let reach = g.g_r + G_BAND * 0.5;
        assert!(
            g.g_dy - reach >= g.head_cap * 0.5 + RULE_DROP + 2.0,
            "ring under the rule"
        );
        let inside = g.g_r - G_BAND * 0.5;
        let unit_foot = g.g_value_cap * 0.5 + 11.0 + g.unit_cap * 0.5;
        assert!(unit_foot + 6.0 <= inside, "the G unit inside the ring");
        let lowest = [
            g.brk_dy + g.bar_h + MARKER_OVER,
            g.g_dy + reach,
            g.shift_dy + g.shift_h * 0.5,
            g.unit_dy + g.unit_cap,
        ];
        for i in 0..PANELS {
            let s = g.slot(i);
            let floor = if s.top {
                g.horizon - HORIZON_BAND
            } else {
                g.screen.max.y
            };
            for dy in lowest {
                assert!(
                    s.head + dy <= floor,
                    "slot {i}: {dy} px under the header passes {floor}"
                );
            }
            assert!(s.x.0 >= g.screen.min.x && s.x.1 <= g.screen.max.x);
        }
    }

    #[test]
    fn panels_option_swaps_and_hides() {
        assert_eq!(Options::parse(""), Options::default());
        let o = Options::parse("speed, OFF, bogus");
        assert_eq!(
            o.panels,
            [Content::Speed, Content::Off, Content::Engine, Content::Speed],
            "unknown and missing entries keep their defaults"
        );
        assert_eq!(Options::parse("g,gforce,hidden,pedals").panels[2], Content::Off);
    }

    /// The signals of a car at `rpm` with this limiter and shift point, as `Signals` derives
    /// them.
    fn signals_at(rpm: f32, max_rpm: f32, shift_rpm: f32) -> Signals {
        Signals {
            rpm,
            max_rpm,
            shift_rpm,
            shift_fraction: rpm / shift_rpm,
            ..Signals::default()
        }
    }

    fn signals(rpm: f32) -> Signals {
        signals_at(rpm, 8000.0, 7760.0)
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
        let lit_at =
            |leds: &TopLeds| -> Vec<bool> { (0..TOP_LEDS).map(|i| leds.color_of(i, &pal).is_some()).collect() };
        // The first LED lights as stage 1 is crossed, and one more every ninth of the range.
        let start = pal.stages[0];
        leds.update(&signals(7760.0 * (start + 0.001)), &pal, 1.0 / 60.0, false, false);
        assert_eq!(lit_at(&leds).iter().filter(|l| **l).count(), 1, "LED 1 at stage 1");
        assert_eq!(leds.color_of(0, &pal), Some(pal.roles.go));
        let mid = start + (1.0 - start) * 0.55;
        leds.update(&signals(7760.0 * mid), &pal, 1.0 / 60.0, false, false);
        let lit = lit_at(&leds);
        assert_eq!(
            lit.iter().filter(|l| **l).count(),
            5,
            "five at 55 % of the range: {lit:?}"
        );
        assert!(lit.windows(2).all(|w| w[0] || !w[1]), "no gaps: {lit:?}");
        // Nine are lit in their groups' colours just under the shift point (2 green, 6 amber,
        // 1 red): the tenth is kept for the shift point itself.
        leds.update(&signals(7760.0 * 0.99), &pal, 1.0 / 60.0, false, false);
        let colors: Vec<Option<Color>> = (0..TOP_LEDS).map(|i| leds.color_of(i, &pal)).collect();
        let count = |c: Color| colors.iter().filter(|x| **x == Some(c)).count();
        assert_eq!(
            (count(pal.roles.go), count(pal.roles.caution), count(pal.roles.warning)),
            (2, 6, 1),
            "{colors:?}"
        );
        assert_eq!(colors[TOP_LEDS - 1], None, "LED 10 waits for the shift point");
        assert_eq!(leds.core(), LED_CORE);
        // Over the shift point all ten are the shift lamp on the strobe's first frame and the
        // second (frames 0 and 1 at 60 Hz), and a middle one is dark on frame 4, in the second
        // half of the 6-frame cycle.
        let lamp = shift_lamp(&pal);
        leds.update(&signals(7800.0), &pal, 1.0 / 60.0, false, false);
        assert!((0..TOP_LEDS).all(|i| leds.color_of(i, &pal) == Some(lamp)));
        assert_eq!(leds.core(), SHIFT_CORE, "the shift state has the hotter core");
        assert!(leds.animating(), "the strobe runs without new telemetry");
        leds.update(&signals(7900.0), &pal, 1.0 / 60.0, false, false);
        assert_eq!(leds.color_of(4, &pal), Some(lamp), "still lit on the second frame");
        for _ in 0..3 {
            leds.update(&signals(7900.0), &pal, 1.0 / 60.0, false, false);
        }
        assert!(
            leds.color_of(4, &pal).is_none(),
            "a middle LED strobed off on the fifth frame, mid-phase (6 frames per cycle)"
        );
        assert_eq!(leds.color_of(0, &pal), Some(lamp), "the outer pairs stay lit");
        assert_eq!(leds.color_of(9, &pal), Some(lamp));
        // A dip inside the hysteresis keeps it over; an upshift's drop clears it at once.
        leds.update(&signals(7700.0), &pal, 1.0 / 60.0, false, false);
        assert!(leds.over);
        leds.update(&signals(5200.0), &pal, 1.0 / 60.0, false, false);
        assert!(!leds.over);
        // An upshift's drop from a full strip leaves no LED lit after 50 ms (three frames).
        leds.update(&signals(7800.0), &pal, 1.0 / 60.0, false, false);
        for frame in 0..3 {
            leds.update(&signals(5200.0), &pal, 1.0 / 60.0, false, false);
            if frame < 2 {
                assert!(leds.color_of(0, &pal).is_some(), "the release has a short tail");
                assert!(leds.animating(), "the host draws the release");
            }
        }
        assert!(
            (0..TOP_LEDS).all(|i| leds.color_of(i, &pal).is_none()),
            "dark 50 ms after the drop"
        );
        assert!(!leds.animating(), "a dark strip is still");
        // A steady staged fill changes nothing between packets.
        leds.update(&signals(7000.0), &pal, 1.0 / 60.0, false, false);
        leds.update(&signals(7000.0), &pal, 1.0 / 60.0, false, false);
        assert!(
            leds.color_of(0, &pal).is_some() && !leds.animating(),
            "a staged fill is still"
        );
        // A snapped frame inside the hysteresis band starts from the staged fill, not the latch:
        // nine lit, the tenth dark.
        leds.update(&signals(7800.0), &pal, 1.0 / 60.0, false, false);
        leds.update(&signals(7700.0), &pal, 0.5, false, true);
        assert!(!leds.over, "a snap clears the latch");
        assert_eq!(leds.color_of(8, &pal), Some(pal.roles.warning));
        assert_eq!(leds.color_of(9, &pal), None);
        // A car that sends no shift point leaves the strip dark.
        let unknown = Signals {
            rpm: 7000.0,
            ..Signals::default()
        };
        leds.update(&unknown, &pal, 1.0 / 60.0, false, true);
        assert!((0..TOP_LEDS).all(|i| leds.color_of(i, &pal).is_none()));
        assert!(!limiter_known(&unknown));
    }

    #[test]
    fn each_led_lights_at_its_share_of_the_shift_rpm() {
        // LED i (0..9) lights when rpm / shift rpm reaches stage 1 + (1 − stage 1) × i / 9: the
        // first at the tokens' first rev stage (0.75), the tenth exactly at the shift point.
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let start = pal.stages[0];
        assert_eq!(start, t.rev_stages.stage1);
        assert_eq!(start, 0.75, "the tokens' first stage");
        let shift_rpm = signals(0.0).shift_rpm;
        for i in 0..TOP_LEDS {
            let threshold = start + (1.0 - start) * i as f32 / (TOP_LEDS - 1) as f32;
            let mut leds = TopLeds::new();
            leds.update(&signals(shift_rpm * (threshold - 0.002)), &pal, 1.0 / 60.0, false, true);
            assert!(leds.color_of(i, &pal).is_none(), "LED {i} dark just under {threshold}");
            if i > 0 {
                assert!(
                    leds.color_of(i - 1, &pal).is_some(),
                    "the LED before {i} is lit by then"
                );
            }
            leds.update(&signals(shift_rpm * threshold), &pal, 1.0 / 60.0, false, false);
            assert!(leds.color_of(i, &pal).is_some(), "LED {i} lit at {threshold}");
            if i + 1 < TOP_LEDS {
                assert!(leds.color_of(i + 1, &pal).is_none(), "LED {} not yet", i + 1);
                assert!(!leds.over);
            } else {
                assert!(leds.over, "the tenth lights with the shift state");
                assert_eq!(leds.color_of(i, &pal), Some(shift_lamp(&pal)));
            }
        }
        // A hair under the shift point the tenth is still dark: it lights with the shift state.
        let mut leds = TopLeds::new();
        leds.update(&signals(shift_rpm * 0.99999), &pal, 1.0 / 60.0, false, true);
        assert!(!leds.over);
        assert_eq!(
            leds.color_of(TOP_LEDS - 1, &pal),
            None,
            "LED 10 waits for the shift point"
        );
        assert!(leds.color_of(TOP_LEDS - 2, &pal).is_some());
    }

    /// The light `led` paints for one lit lens of `size` in `color` (px² of relative luminance
    /// over black), integrated on a quarter-pixel grid: the radial fill from its hot centre
    /// ([`lens_core`]) to its colour, the 1 px inner rim over it, and the halo outside it
    /// ([`led_halo`]: the kit's glow, `intensity × exp(−d² / 2σ²)` outside the shape out to 3σ).
    /// The bloom is left out: it follows the fill.
    fn lens_light(size: Vec2, color: Color, core: f32, pal: &Palette, t: &Tokens) -> f32 {
        let halo = led_halo(t).expect("the medium glow has a lobe");
        let half = size * 0.5;
        let r = half.x.min(half.y);
        let reach = halo.sigma * 3.0;
        let hot = lens_core(color, core, pal);
        let rim = pal.roles.ink.with_alpha(LED_RIM * color.a);
        let step = 0.25;
        let (nx, ny) = (
            ((half.x + reach) / step).ceil() as i32,
            ((half.y + reach) / step).ceil() as i32,
        );
        let mut sum = 0.0;
        for iy in -ny..ny {
            for ix in -nx..nx {
                let p = vec2((ix as f32 + 0.5) * step, (iy as f32 + 0.5) * step);
                // The round rect's signed distance (corner radius `r`, as `led` draws it).
                let q = vec2(p.x.abs(), p.y.abs()) - half + Vec2::splat(r);
                let d = q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - r;
                let lit = if d > 0.0 {
                    halo.intensity * (-0.5 * (d / halo.sigma) * (d / halo.sigma)).exp() * light(color)
                } else {
                    let fill = light(hot.lerp(color, (p.length() / half.x).min(1.0)));
                    if d > -1.0 {
                        // The rim, source-over the fill (premultiplied).
                        light(rim) + fill * (1.0 - rim.a)
                    } else {
                        fill
                    }
                };
                sum += lit * step * step;
            }
        }
        sum
    }

    /// The family's palette, and palettes with a darker or more saturated shift colour (the
    /// default brand's `#FF1744`, the tokens' `#7C5CFF`) and a white one.
    fn shift_palettes(t: &Tokens) -> Vec<Palette> {
        let family = Palette::family(t, "raceDash");
        let mut out = vec![family];
        for hex in ["#FF1744", "#7C5CFF", "#FFFFFF"] {
            let shift = Color::from_hex(hex).expect("hex");
            out.push(Palette::new(
                t,
                [family.primary, family.secondary, family.accent, shift],
                fh6_kit::PaletteSource::Custom,
            ));
        }
        out
    }

    #[test]
    fn the_brow_is_brightest_over_the_shift_point() {
        // Ten lit lenses over the shift point give more light than the fullest staged fill, the
        // halos counted, for every palette.
        let t = Tokens::default();
        let size = GEOM.top_led;
        for pal in shift_palettes(&t) {
            let brow = |leds: &TopLeds| -> f32 {
                (0..TOP_LEDS)
                    .filter_map(|i| leds.color_of(i, &pal))
                    .map(|c| lens_light(size, c, leds.core(), &pal, &t))
                    .sum()
            };
            let mut leds = TopLeds::new();
            let mut staged: f32 = 0.0;
            for step in 0..=200 {
                // 0.70 to 0.999 of the shift rpm: every staged fill.
                let frac = 0.70 + 0.299 * step as f32 / 200.0;
                leds.update(&signals(7760.0 * frac), &pal, 1.0 / 60.0, false, true);
                assert!(!leds.over);
                staged = staged.max(brow(&leds));
            }
            leds.update(&signals(7760.0), &pal, 1.0 / 60.0, false, false);
            assert!(leds.over);
            let over = brow(&leds);
            let shift = pal.shift.to_srgb8();
            assert!(
                over > staged,
                "shift {shift:?}: reaching the shift point adds light: {over} against the fullest staged brow {staged}"
            );
            // The lamp is the shift colour where that is bright enough, and never dimmer than
            // amber.
            let lamp = shift_lamp(&pal);
            assert!(light(lamp) >= light(pal.roles.caution) - 1e-4 || lamp == pal.shift);
            assert!(light(lamp) >= light(pal.shift) - 1e-4);
        }
        // The outer pairs alone (the strobe's dark phase) do not have to beat the staged fill:
        // the cue there is the strobe itself.
    }

    #[test]
    fn reduced_flash_breathes_from_the_peak_and_never_goes_dark() {
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let lamp = shift_lamp(&pal);
        let mut calm = TopLeds::new();
        calm.update(&signals(7000.0), &pal, 1.0 / 60.0, true, true);
        let mut levels = Vec::new();
        for _ in 0..60 {
            calm.update(&signals(7900.0), &pal, 1.0 / 60.0, true, false);
            assert!(calm.strobe_on, "reduced flash never strobes");
            assert_eq!(calm.color_of(4, &pal), Some(lamp.fade(calm.breath)));
            assert_eq!(calm.color_of(0, &pal), Some(lamp), "the outer pairs stay full");
            levels.push(calm.breath);
        }
        assert_eq!(levels[0], 1.0, "the cue starts at its peak");
        let floor = levels.iter().copied().fold(f32::MAX, f32::min);
        let peak = levels.iter().copied().fold(f32::MIN, f32::max);
        assert!(
            (floor - 0.6).abs() < 0.005 && peak <= 1.0,
            "0.6 to 1.0: {floor}..{peak}"
        );
        // 2 Hz: back at the peak after half a second, at the floor a quarter of a second in.
        assert!((levels[30] - 1.0).abs() < 0.01 && (levels[15] - 0.6).abs() < 0.01);
        // The lens's hot centre fades with it, and so does what the lens paints.
        let dim = lamp.fade(0.6);
        assert!(
            (lens_core(dim, SHIFT_CORE, &pal).a - 0.6).abs() < 1e-6,
            "the core keeps the lens's level"
        );
        let size = GEOM.top_led;
        let full = lens_light(size, lamp, SHIFT_CORE, &pal, &t);
        // 0.6 of its light, a hair more where the 1 px rim lies over a dimmer lens (source-over
        // is not linear in the lens's level).
        let dimmed = lens_light(size, dim, SHIFT_CORE, &pal, &t);
        assert!(dimmed < full * 0.62, "{dimmed} against {full}");
    }

    #[test]
    fn the_rev_band_cross_fades_to_ink_and_holds_it() {
        let dt = 1.0 / 60.0;
        let mut arc = ArcInk::new();
        arc.update(false, false, dt, true);
        assert_eq!(arc.mix, 0.0);
        assert!(!arc.animating(false));
        // Reaching the shift point: a fade over about 0.1 s, never a one-frame step.
        let mut last = 0.0;
        let mut frames = 0;
        arc.update(true, true, dt, false);
        while arc.mix < 1.0 {
            assert!(
                arc.mix - last <= dt / ARC_FADE_S + 1e-6,
                "no step: {last} to {}",
                arc.mix
            );
            assert!(arc.animating(true));
            last = arc.mix;
            frames += 1;
            arc.update(true, false, dt, false);
        }
        assert!((5..=7).contains(&frames), "about 0.1 s at 60 Hz: {frames} frames");
        // A brush of the shift point (two frames over it) still gives ink for at least 0.35 s
        // from the frame it was reached, then fades back.
        let mut arc = ArcInk::new();
        arc.update(true, true, dt, false);
        arc.update(true, false, dt, false);
        let mut ink_frames = 2;
        loop {
            arc.update(false, false, dt, false);
            if arc.mix <= 0.0 {
                break;
            }
            assert!(arc.animating(false), "the host keeps drawing until the ink is gone");
            ink_frames += 1;
            assert!(ink_frames < 120, "the hold ends");
        }
        let seconds = ink_frames as f32 * dt;
        assert!(
            (ARC_HOLD_S..=ARC_HOLD_S + ARC_FADE_S + 2.0 * dt).contains(&seconds),
            "ink for {seconds} s"
        );
        // After a long stay over the shift point the band fades back at once.
        let mut arc = ArcInk::new();
        arc.update(true, true, dt, false);
        for _ in 0..60 {
            arc.update(true, false, dt, false);
        }
        assert_eq!(arc.mix, 1.0);
        assert!(!arc.animating(true));
        arc.update(false, false, dt, false);
        assert!(arc.mix < 1.0 && arc.mix > 0.0);
        // A snapped frame takes the state as it is.
        arc.update(true, false, 0.5, true);
        assert_eq!(arc.mix, 1.0);
        arc.update(false, false, 0.5, true);
        assert_eq!(arc.mix, 0.0);
    }

    #[test]
    fn side_columns_latch_their_steps() {
        assert_eq!(latched(0, 2.4, 2.5, 0.5, 5), 0);
        assert_eq!(latched(0, -5.1, 2.5, 0.5, 5), 2, "either direction of slip");
        assert_eq!(latched(2, 4.8, 2.5, 0.5, 5), 2, "held within the hysteresis");
        assert_eq!(latched(2, 4.4, 2.5, 0.5, 5), 1);
        assert_eq!(latched(0, 40.0, 2.5, 0.5, 5), 5, "at most five");
        // The brake: one LED per started fifth; a pedal resting on a fifth (the packet's byte
        // toggling 51 and 52 of 255) keeps its LED.
        assert_eq!(brake_latched(0, 0.01), 0, "dead zone");
        assert_eq!(brake_latched(0, 0.05), 1);
        assert_eq!(brake_latched(0, 52.0 / 255.0), 2);
        assert_eq!(brake_latched(2, 51.0 / 255.0), 2, "held within the hysteresis");
        assert_eq!(
            brake_latched(2, 0.16),
            1,
            "out once the pedal is clearly under the step"
        );
        assert_eq!(brake_latched(3, 1.0), 5);
        assert_eq!(brake_latched(5, 0.0), 0, "released at once");
        // The dead zone (0.02, between bytes 5 and 6) is latched too: a trigger resting on byte
        // 5 keeps a lit LED and lights none.
        assert_eq!(brake_latched(0, 6.0 / 255.0), 1);
        assert_eq!(brake_latched(1, 5.0 / 255.0), 1, "held at the dead zone");
        assert_eq!(brake_latched(0, 5.0 / 255.0), 0);
        assert_eq!(brake_latched(1, 4.0 / 255.0), 0, "out under it");
    }

    #[test]
    fn rpm_field_follows_the_value_and_moves_rarely() {
        assert_eq!(rpm_cells(3, 900, false), 3);
        assert_eq!(rpm_cells(3, 1000, false), 4, "grows at once");
        assert_eq!(rpm_cells(4, 960, false), 4, "held 50 rpm under the step");
        assert_eq!(rpm_cells(4, 940, false), 3);
        assert_eq!(rpm_cells(4, 10_000, false), 5);
        assert_eq!(rpm_cells(5, 9960, false), 5);
        assert_eq!(rpm_cells(5, 9940, false), 4);
        assert_eq!(rpm_cells(5, 800, false), 3, "a big drop takes what the value needs");
        assert_eq!(rpm_cells(5, 9960, true), 4, "a snap takes the value as it is");
    }

    #[test]
    fn rpm_digits_never_change_on_a_limiter_bounce() {
        // A sawtooth limiter bounce from 9000 rpm whose later peaks vary, fed through the path
        // the dash runs (the lights' latch, the band's clock, the digits' gate): the digits (10
        // rpm steps, 30 rpm of hysteresis) must not change at all. A shift point at the limiter
        // (`ShiftPointPct` 1.00) and a deep bounce both fall out of the lights' 120 rpm latch on
        // every bounce, so only the digits' own gate holds them.
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let dt = 1.0 / 60.0;
        /// The peak of bounce `n`.
        type Peaks = fn(usize) -> f32;
        let shapes: [(usize, Peaks); 4] = [
            // 100 ms bounces, later peaks 10 rpm lower.
            (6, |bounce| if bounce == 0 { 9000.0 } else { 8990.0 }),
            // 150 ms bounces (the longest), later peaks 10 rpm lower.
            (9, |bounce| if bounce == 0 { 9000.0 } else { 8990.0 }),
            // 100 ms and 150 ms bounces whose peaks alternate.
            (6, |bounce| if bounce % 2 == 0 { 9000.0 } else { 8990.0 }),
            (9, |bounce| [9000.0, 8990.0, 8980.0, 8990.0][bounce % 4]),
        ];
        // (shift point, bounce depth): the default 0.97, a shift point at the limiter, and a
        // deep bounce at each.
        let cars = [(8730.0, 240.0), (9000.0, 240.0), (8730.0, 500.0), (9000.0, 500.0)];
        for (shift_rpm, depth) in cars {
            for (case, (period, top)) in shapes.into_iter().enumerate() {
                let mut rev = Rev::new();
                let mut shown = Vec::new();
                let mut band = Vec::new();
                let mut dropped_out = false;
                for frame in 0..480 {
                    let phase = (frame % period) as f32 / period as f32;
                    let rpm = top(frame / period) - depth * phase;
                    rev.update(&signals_at(rpm, 9000.0, shift_rpm), &pal, dt, false, frame == 0);
                    dropped_out |= !rev.leds.over;
                    shown.push(rev.digits.shown);
                    band.push(rev.digits.held);
                }
                assert!(
                    shown.iter().all(|&v| v == 9000),
                    "shift {shift_rpm}, depth {depth}, case {case}: the digits never change: {shown:?}"
                );
                // The band's head takes the same held rpm: it stays on the bounces' peaks
                // (8980 to 9000) instead of following each bounce down.
                assert!(
                    band.iter().all(|&v| (8980.0..=9000.0).contains(&v)),
                    "shift {shift_rpm}, depth {depth}, case {case}: the band holds the peak: {band:?}"
                );
                if shift_rpm == 9000.0 || depth > 400.0 {
                    assert!(
                        dropped_out,
                        "shift {shift_rpm}, depth {depth}: the bounce leaves the latch"
                    );
                }
            }
        }
        // An upshift's drop releases the digits on its first frame, and an old peak never comes
        // back as the rpm climbs again.
        let mut rev = Rev::new();
        rev.update(&signals_at(9000.0, 9000.0, 9000.0), &pal, dt, false, true);
        rev.update(&signals_at(9000.0, 9000.0, 9000.0), &pal, dt, false, false);
        assert_eq!(rev.digits.shown, 9000);
        rev.update(&signals_at(6500.0, 9000.0, 9000.0), &pal, dt, false, false);
        assert_eq!(rev.digits.shown, 6500, "an upshift shows the new rpm at once");
        rev.update(&signals_at(8600.0, 9000.0, 9000.0), &pal, dt, false, false);
        assert_eq!(
            rev.digits.shown, 8600,
            "released until the shift point is reached again"
        );
        // A climb that has not reached the shift point is never held.
        let mut rev = Rev::new();
        rev.update(&signals_at(8000.0, 9000.0, 8730.0), &pal, dt, false, true);
        rev.update(&signals_at(8600.0, 9000.0, 8730.0), &pal, dt, false, false);
        rev.update(&signals_at(8400.0, 9000.0, 8730.0), &pal, dt, false, false);
        assert_eq!(rev.digits.shown, 8400);
    }

    #[test]
    fn the_digits_gate_holds_a_bounce_and_releases_an_upshift() {
        // Over the shift point the peak is held; under it only while still holding, soon after
        // the shift point was reached and close to the peak.
        assert!(holds_peak(true, false, ARC_HOLD_S, 9000.0, 8000.0));
        assert!(holds_peak(false, true, 0.1, 9000.0, 9000.0 - BOUNCE_RPM));
        assert!(
            !holds_peak(false, true, 0.1, 9000.0, 9000.0 - BOUNCE_RPM - 1.0),
            "an upshift"
        );
        assert!(!holds_peak(false, true, ARC_HOLD_S, 9000.0, 8900.0), "too long after");
        assert!(!holds_peak(false, false, 0.1, 9000.0, 8900.0), "released already");
        // The readout saturates to its five cells.
        let mut digits = RpmDigits::new();
        digits.update(1.0e7, false, false, 1.0 / 60.0, true);
        assert_eq!(digits.shown, 99_999);
    }

    #[test]
    fn the_peak_window_forgets_within_its_length() {
        let dt = 1.0 / 60.0;
        let mut window = PeakWindow::new();
        assert_eq!(window.update(9000.0, dt, true), 9000.0, "a snap takes the rpm");
        // A peak is kept for at least half the window and at most all of it (a frame of slack
        // for the bucket's edge).
        let mut held = 0;
        while window.update(8000.0, dt, false) > 8000.0 {
            held += 1;
            assert!(held < 60, "the peak is dropped");
        }
        let seconds = held as f32 * dt;
        assert!(
            (PEAK_HOLD_S * 0.5 - dt..=PEAK_HOLD_S + dt).contains(&seconds),
            "held {seconds} s"
        );
        // A rising rpm is followed at once.
        assert_eq!(window.update(8500.0, dt, false), 8500.0);
        assert_eq!(window.update(3000.0, dt, true), 3000.0, "a snap starts again");
    }

    #[test]
    fn a_snapped_frame_puts_the_g_ball_at_rest() {
        // A new ball starts in the centre and would drift to the force over eight or so frames
        // after a snapped frame; the settled one is on the force with its trail collapsed.
        let t = Tokens::default();
        let sig = Signals {
            g_lat: 0.9,
            g_long: -0.4,
            ..Signals::default()
        };
        let mut ball = settled_gball(&t, &sig);
        let rest = crate::common::gball::felt_force(sig.g_lat, sig.g_long);
        assert!((ball.ball() - rest).length() < 1e-4, "{:?} at {rest:?}", ball.ball());
        assert!(!ball.animating(), "nothing left to settle");
        for _ in 0..12 {
            ball.update(&sig, 1.0 / 60.0);
            assert!((ball.ball() - rest).length() < 1e-4, "no drift");
            assert!(!ball.animating());
        }
        assert!(
            SETTLE_STEPS as f32 * SETTLE_DT >= 4.0 * crate::common::gball::TRAIL_SECONDS,
            "the settle outlasts the trail"
        );
    }

    #[test]
    fn a_snapped_frame_puts_the_virtual_light_at_rest() {
        // Built mid-corner, the housing is lit from the corner's side at once: the settled light
        // no longer moves, as a light run long on the same force does not.
        let t = Tokens::default();
        let sig = Signals {
            g_lat: 0.9,
            ..Signals::default()
        };
        let mut light = settled_light(&t, &sig);
        let mut long = VirtualLight::new(&t);
        for _ in 0..600 {
            long.update(sig.g_lat, 1.0 / 60.0);
        }
        assert!((light.angle() - long.angle()).abs() < 1e-4, "at rest on the force");
        let before = light.angle();
        light.update(sig.g_lat, 1.0 / 60.0);
        assert!((light.angle() - before).abs() <= 1e-4, "nothing left to settle");
    }

    #[test]
    fn printed_parts_keep_their_contrast_under_the_static_cap() {
        // Relative luminance over OLED black: at least 4.5:1 (WCAG 1.4.3) and at most the static
        // cap of 0.2 (the burn-in rule for parts that never move).
        let t = Tokens::default();
        let pal = Palette::family(&t, "raceDash");
        let contrast = |c: Color| (light(c) + 0.05) / 0.05;
        let print = print_ink(&pal);
        assert!(contrast(print) >= 4.5, "print at {}:1", contrast(print));
        assert!(light(print) <= STATIC_CAP, "print at {}", light(print));
        // The BOOST label takes the accent at the same light; its value is ink, the brightest
        // text there is, so it reads at a glance.
        let accent = accent_print(&pal);
        assert!((light(accent) - light(print)).abs() < 0.005);
        assert!(contrast(accent) >= 4.5 && light(accent) < light(pal.primary_text));
        assert!(
            light(accent) < light(pal.roles.ink),
            "the BOOST value outshines its label"
        );
        // The hairlines sit one tier under the print and still read as lines (3:1).
        let rule = hairline(&pal);
        assert_eq!(rule.a, 1.0, "opaque, so crossings show no joints");
        assert!(light(rule) < light(print) * 0.75, "hairlines at {}", light(rule));
        assert!(contrast(rule) >= 3.0, "hairlines at {}:1", contrast(rule));
        // The G ring's unlit track: half the light it had, still a form on black (1.5:1).
        let track = g_track(&pal);
        assert!(light(track) <= light(pal.roles.ink_ghost) * 0.5 + 1e-6);
        assert!(contrast(track) >= 1.5, "the G track at {}:1", contrast(track));
        // A white logo image at its opacity stays under the cap.
        const { assert!(LOGO_SHARE <= STATIC_CAP) };
    }

    #[test]
    fn tinted_static_parts_stay_under_the_cap_for_any_palette() {
        // The parts tinted with `primary` or `warning` that never move, through the helpers
        // `draw` uses, on the family's palette, the default brand's cyan, a brand green and a
        // white primary.
        let t = Tokens::default();
        let family = Palette::family(&t, "raceDash");
        let mut palettes = vec![family];
        for hex in ["#00E5FF", "#76FF03", "#FFFFFF"] {
            let primary = Color::from_hex(hex).expect("hex");
            palettes.push(Palette::new(
                &t,
                [primary, family.secondary, family.accent, family.shift],
                fh6_kit::PaletteSource::Custom,
            ));
        }
        let cap = STATIC_CAP + 1e-5;
        for pal in palettes {
            let hex = pal.primary.to_srgb8();
            for reverse in [false, true] {
                let (apex, edge, edge_alpha) = beam_rest(&pal, reverse);
                assert!(light(pal.primary.with_alpha(apex)) <= cap, "{hex:?}: beam apex");
                assert!(light(edge.with_alpha(edge_alpha)) <= cap, "{hex:?}: beam edges");
                // As they stack at the apex: the body on black, an edge over it.
                let body = beam_stack(pal.primary.luminance(), apex, 0.0);
                let top = beam_stack(edge.luminance(), edge_alpha, body);
                assert!(body <= cap && top <= cap, "{hex:?}: beam stack {body} / {top}");
            }
            for (part, color) in [
                ("header rules and horizon", rule_tint(&pal)),
                ("red zone", red_zone(&pal)),
                ("shift box", shift_box(&pal)),
                ("horizon light", static_tint(pal.primary, HORIZON_GLOW)),
                ("disc light", static_tint(pal.primary, DISC_GLOW)),
            ] {
                assert!(light(color) <= cap, "{hex:?}: {part} at {}", light(color));
            }
        }
        // The family's own palette keeps its levels: the cap only bites on light colours.
        assert_eq!(static_alpha(family.primary, SHIFT_BOX), SHIFT_BOX);
        // The beam keeps its apex; its edges give up a little where they lie over the body.
        let (apex, _, edge) = beam_rest(&family, false);
        assert!(apex == BEAM_APEX && edge > 0.8 * BEAM_EDGE, "{apex} {edge}");
    }

    #[test]
    fn the_hood_view_keeps_only_parts_over_the_floors() {
        // The one decision `draw` follows: the cockpit draws everything; the hood view keeps the
        // SPEED panel (its digits) and leaves out the other panels and all fine print (headers,
        // labels, units, rev numerals and the chin's wordmarks).
        let contents = [
            Content::Pedals,
            Content::GForce,
            Content::Engine,
            Content::Speed,
            Content::Off,
        ];
        for content in contents {
            assert!(shows(View::Cockpit, Part::Panel(content)));
            assert_eq!(
                shows(View::Hood, Part::Panel(content)),
                content == Content::Speed,
                "{content:?}"
            );
        }
        assert!(shows(View::Cockpit, Part::Print));
        assert!(!shows(View::Hood, Part::Print));
        // What it keeps (the rpm digits, the gear and the speed's digits) stays over a
        // readout's 17.7 px at HOOD_SCALE.
        let g = &GEOM;
        for cap in [g.rpm_cap, g.gear_cap, g.speed_cap] {
            assert!(cap * HOOD_SCALE >= 17.7, "cap {cap} in the hood view");
        }
        // What it leaves out as fine print would fall under the floors: labels and units under
        // the tokens' smallest size, the rev numerals and the panels' small values under 17.7 px.
        let t = Tokens::default();
        for cap in [g.head_cap, g.label_cap, g.unit_cap] {
            assert!(
                cap * HOOD_SCALE < t.type_scale.nano,
                "cap {cap} is over its floor in the hood view: draw it there"
            );
        }
        for cap in [g.num_cap, g.value_cap, g.g_value_cap] {
            assert!(
                cap * HOOD_SCALE < 17.7,
                "cap {cap} is over its floor in the hood view: draw it there"
            );
        }
        // The recorded exceptions. ENGINE goes whole although its power figure alone would clear
        // the floor, since its labels and boxed shift point could not come with it. The chin's
        // wordmark (the kit's logo cap 18) would stand at the smallest size's edge, 12.2 px, and
        // under it whenever the kit shrinks a long name to its zone, so it goes with the print.
        assert!(g.power_cap * HOOD_SCALE >= 17.7);
        let fonts = fh6_kit::Fonts::new().expect("embedded font");
        let wordmark = LogoStyle::new(fonts.role(FontRole::Wide)).cap * HOOD_SCALE;
        assert!(
            (wordmark - 12.24).abs() < 0.01 && wordmark < t.type_scale.nano + 0.5,
            "{wordmark} px"
        );
    }

    #[test]
    fn panel_rows_fit_the_narrowest_slot() {
        // Every panel can sit in every slot, so its widest row must fit the narrowest one.
        let mut fonts = fh6_kit::Fonts::new().expect("embedded font");
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/fonts");
        fonts.load_dir(&dir).expect("assets/fonts");
        let label = fonts.find("Titillium Web", 600, false).expect("label face");
        let figures = fonts.find("Saira Semi Condensed", 600, false).expect("figures face");
        assert_eq!(
            fonts.desc(figures).map(|d| d.weight),
            Some(600),
            "the SemiBold face is in the pack"
        );
        // Pack faces load their glyphs lazily; measure the real advances.
        let gear_face = fonts.find("Saira Condensed", 700, false).expect("gear face");
        fonts.ensure_preloaded(label);
        fonts.ensure_preloaded(figures);
        fonts.ensure_preloaded(gear_face);
        let t = Tokens::default();
        let g = &GEOM;
        assert_eq!(g.left.1 - g.left.0, g.right.1 - g.right.0, "the slots mirror");
        assert_eq!(g.left.1, -g.right.0, "about the gauge");
        let narrow = g.right.1 - g.right.0;
        let width = |text: &str, style: &TextStyle| fonts.measure(text, style).width;
        let head = TextStyle::new(label, g.head_cap).tracking(t.type_scale.label_tracking);
        for text in ["PEDALS", "G-FORCE", "ENGINE", "SPEED"] {
            assert!(width(text, &head) <= narrow, "header {text}");
        }
        // A value keeps at least this much air from the label and unit before it.
        const AIR: f32 = 4.0;
        let row = TextStyle::new(label, g.label_cap);
        let small = TextStyle::new(figures, g.value_cap).tabular();
        let boost = width("BOOST", &row) + ROW_GAP + width("psi", &row) + AIR + width("60.0", &small);
        assert!(boost <= narrow, "BOOST row: {boost} px in {narrow}");
        let shift = width("SHIFT", &row) + AIR + SHIFT_PAD + 5.0 * fonts.tabular_cell(&small) + SHIFT_PAD;
        assert!(
            shift <= narrow,
            "SHIFT row with five boxed cells: {shift} px in {narrow}"
        );
        assert!(g.value_cap + 6.0 <= g.shift_h, "the shift point inside its box");
        let power = TextStyle::new(figures, g.power_cap).tabular();
        assert!(4.0 * fonts.tabular_cell(&power) <= narrow, "four power cells");
        assert!(width("POWER", &row) + ROW_GAP + width("kW", &row) <= narrow);
        let speed = TextStyle::new(figures, g.speed_cap).tabular();
        assert!(3.0 * fonts.tabular_cell(&speed) <= narrow, "three speed cells");
        // The G ring stays in the panel, and its value (0.00 to 9.99, bounded by `G_CELLS`)
        // inside the ring.
        assert!((g.g_r + G_BAND * 0.5) * 2.0 <= narrow);
        let g_value = TextStyle::new(figures, g.g_value_cap).tabular();
        let chord = {
            let (r, up) = (g.g_r - G_BAND * 0.5, 4.0 + g.g_value_cap * 0.5);
            2.0 * (r * r - up * up).sqrt()
        };
        assert!(width("9.99", &g_value) + 4.0 <= chord, "the G value inside its ring");
        // Every gear's label fits inside the beam at the gear's middle line: a single glyph even
        // at its 1.12 entry, the two-digit "10" at rest (its entry passes the beam's edges there
        // for the moment of the change).
        let beam = g.beam();
        let along = (g.gear_at.y - g.horizon) / (beam[3].y - g.horizon);
        let beam_w = 2.0 * (beam[1].x + (beam[2].x - beam[1].x) * along);
        for raw in 0..=11u8 {
            let label = Gear::from_raw(raw).label();
            let entry = if label.len() > 1 { 1.0 } else { 1.12 };
            let w = width(label, &TextStyle::new(gear_face, g.gear_cap * entry));
            assert!(w <= beam_w, "gear {label} ({w} px) in the beam ({beam_w} px)");
        }
        // The rpm digits (five cells at most) stay inside the disc.
        let rpm = TextStyle::new(figures, g.rpm_cap).tabular();
        let inner = g.disc_r();
        assert!(
            5.0 * fonts.tabular_cell(&rpm) <= 2.0 * inner - 8.0,
            "five rpm cells in the disc"
        );
    }
}
