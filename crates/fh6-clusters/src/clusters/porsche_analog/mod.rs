//! Porsche Analog (`ClusterTheme::PorscheAnalog`), catalog row 7: the Classic Analog family's
//! pattern-setter (cockpit, build #3) and the reference for rendered physical hardware.
//!
//! The Porsche 911 (992) five-tube cluster (`RESEARCH.md`): the rev counter in the middle, the
//! speed and G tubes tucked behind it, the power and boost tubes tucked behind those, as on the
//! 992.2's Classic view. Every tube is Blender-rendered hardware (`hardware.rs`, recipe
//! `tools/asset-forge/src/recipes/porsche_analog.rs`) drawn at full strength, the live data is
//! drawn on top, and the crystals and the gear's magnifier are optical glass
//! (`fh6_kit::OpticalGlass`, `docs/specs/optics.md`). Board: Penpot page `07 · PorscheAnalog`.
//!
//! Zone table (local 4K px round the cockpit anchor; the hood draws the same table at
//! [`HOOD_SCALE`]):
//!
//! | Zone | Part | Centre | Size | Holds |
//! |---|---|---|---|---|
//! | T | rev counter | (0, −40) | bezel R 248, dial R 214, raised scale ring R 150–214, hub cap R 20 | ranks 1 and 3: the shift ring (11 lenses at R 231 in the bezel, from 205° over 130°, so only the approach to the shift point and the red zone carry lenses), the scale (240°, ticks to R 204, numerals tucked 5 px inside the bars, cap 32), the needle (tail 26, tip 200), the logo zone on the hub's vertical (160 × 20, 100 below the hub when that clears the window; else 120 × 18, 104 below; else 160 × 20, 84 above) and "1/min × 1000" under it |
//! | T·W | gear window | the middle of the scale's dead zone, 116 from the hub, reaching across the numeral-free part of the scale ring | porthole R 70, rim R 76 | rank 1: the gear (cap 96, "10" at 0.76 of it, magnified about 5 % by the low sapphire cyclops) and, from the shift point, the "+" upshift cue beside it in the same window, as on the Carrera |
//! | L | left tube | (−396, −8), 36 px behind the rev counter's bezel | bezel R 184, dial R 160, scale ring R 112–160 | rank 2 by default: speed (pointer R 126–156, numerals cap 17 tucked inside the bars, digits cap 66) |
//! | R | right tube | (396, −8) | as L | rank 4 by default: G (felt force), throttle and brake; the driver greeting while parked |
//! | FL / FR | outer tubes | (∓612, 16) | bezel R 108, dial R 92 | power and boost by default, read on the visible outer crescent |
//!
//! Every tube's content can be swapped or hidden from the config (see [`Options`]).
//!
//! Signature moments:
//! - **Plumb line:** the shift ring fills from both sides up to 12 o'clock, driven by the
//!   needle's own rpm while it rises, so in the track face the last lens lights in the frame the
//!   needle stands vertical under it. On the limiter every flank lens turns steady white and
//!   only the three lenses at 12 strobe (10 Hz, a band 22 px tall); "+" lights beside the gear
//!   window. A bounce off the limiter holds the needle at its peak, as a damped needle does.
//! - **The scale turns:** when a car with a different shift point arrives, the scale (and the
//!   needle with it) turns on the tokens' slow spring to put the new shift rpm at 12 o'clock.
//! - **Gear change:** the token glyph motion under the magnifier, and the porthole's rim flashes
//!   once; after an upshift the shift ring stays dark until the rpm has fallen, so a new gear
//!   never shows under a "shift again" strobe.
//! - **Light:** lateral G swings one key light 25° per 1.5 g across every bezel, needle and
//!   crystal (the hardware is rendered at three azimuths and cross-faded).
//!
//! The host builds the cluster again on every view change, and the OSD's dyno frames, menus and
//! pauses interrupt it, so the first frame of a cluster, of a race-flag change and after a gap
//! takes the car's state as it is (needle, pointers, gear, greeting, scale, readouts) instead of
//! animating into it.
//!
//! Telemetry is native: rpm (current, limiter), the shift point, gear, speed, throttle, brake
//! and smoothed G from `Signals`, and power and boost from the packet. Nothing is synthesised.
//! The small digital readouts (power, boost, G) refresh at a few hertz with hysteresis, as a
//! car's display does (`readout.rs`).
mod hardware;
mod readout;
mod scale;
mod tubes;

use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::{FRAC_PI_2, PI, TAU};
use fh6_core::AppConfig;
use fh6_kit::cluster::Setup;
use fh6_kit::{
    vec2, Align, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, ImageId, LayerOptions,
    Layout, Logo, OpticalGlass, Painter, Palette, Piece, Rect, Shadow, Shape, Signals, Smoothed, SpeedUnit, Spring,
    SpringSpec, Stroke, TextStyle, Tokens, Tween, VAlign, Vec2, View,
};
use hardware::{Kind, Sprites};
use readout::{LimiterHold, Readout};
use scale::{DialScale, Mode};

/// The shift ring's lenses (odd, so the last one lit is the one at 12 o'clock).
const LEDS: usize = 11;
/// Lenses within this angle of 12 o'clock strobe on the limiter: the middle three, a band 22 px
/// tall (the strobe rule: at most 42 px).
const CREST_DEG: f32 = 16.0;
/// While the needle holds a limiter bounce, within this many rpm of the held peak it rests on
/// it (under a degree on the scale).
const HOLD_SNAP: f32 = 30.0;
/// Clear space between a numeral and the inner end of the bars (px).
const NUM_GAP: f32 = 5.0;
/// The hood view draws the cockpit table at this scale (496 px tall → 357 px, inside the hood's
/// 400 px envelope).
pub const HOOD_SCALE: f32 = 0.72;
/// After an upshift the shift ring stays dark for this long (s), unless the rpm has fallen below
/// the shift point sooner.
const UPSHIFT_QUIET_S: f32 = 0.3;
/// For this long after any gear change (s) the needle follows the rpm on its spring alone: the
/// rpm steps on a shift, and the kit's rpm rate (smoothed over 40 ms) still carries the old
/// slope, so its feed-forward would push the needle past the new rpm.
const SHIFT_LEAD_OFF_S: f32 = 0.15;
/// Once the shift ring is over the shift point it stays over until the rpm falls this far
/// under it (the tokens' `hysteresisRpm`, COMPONENTS.md), so an rpm hovering at the shift point
/// never flashes the whole ring.
const SHIFT_HYSTERESIS_RPM: f32 = 120.0;
/// A frame at least this long (s) follows a gap in the dash's updates (the host caps `dt` at
/// 0.1 s and skips updates while the dash is hidden for a menu or a pause): the dash takes the
/// car's state as it is, as on its first frame.
const RESUME_DT: f32 = 0.09;
/// A two-character gear ("10") is drawn at this share of the gear cap, so it stays inside the
/// magnifier's view and clear of the "+".
const TWO_DIGIT_GEAR: f32 = 0.76;
/// A cluster built mid-race settles the gear glyph's entry motion in this many steps of this
/// length (s) before its first draw.
const GEAR_SETTLE_STEPS: u32 = 30;
const GEAR_SETTLE_DT: f32 = 1.0 / 60.0;
/// Below this throttle the driver has lifted, and the power and boost scales may re-range.
const RANGE_LIFT: f32 = 0.2;
/// The window rim's flash on a gear change (s, linear fade).
const RIM_FLASH_S: f32 = 0.22;
/// Period of the spring that turns the scale when the shift point changes (s; no overshoot).
const TURN_PERIOD_S: f32 = 0.9;
/// Lateral or longitudinal G at which the G dot reaches the outer ring.
const G_LIMIT: f32 = 1.5;
/// Speed (display units) and gear that count as parked: the right tube greets the driver.
const PARKED_SPEED: f32 = 1.0;
/// On-screen levels of the static marks (linear coverage over black): numerals and major ticks,
/// half ticks, minor ticks.
const LEVEL_NUMERAL: f32 = 0.86;
const LEVEL_HALF: f32 = 0.62;
const LEVEL_MINOR: f32 = 0.42;
/// How far the Heritage numerals lean from the `go` role to `ink` (a pale 356 green).
const HERITAGE_INK: f32 = 0.42;
/// The glow of a lit shift lens (sigma, intensity): tight, so the strobe never spills far.
const LENS_GLOW: (f32, f32) = (6.0, 0.4);
/// The magnifier's share of the studio's key light and reflections: its glint stays under the
/// gear's ink, so the brightest thing in the window is the gear and not the glass over it.
const CYCLOPS_LIGHT: f32 = 0.45;
/// The logo zone and its legend keep this far from the window's rim (px).
const LOGO_CLEAR: f32 = 4.0;
/// The light round the hub cap's edge (sigma, intensity): a thin ring in the needle's colour.
const HUB_GLOW: (f32, f32) = (4.0, 0.3);

/// Numerals 0 to 20 (thousands), so drawing never formats a string.
const NUMERALS: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
];

/// The zone table as data (cockpit, local 4K px round the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    tach: Vec2,
    tach_open: f32,
    tach_outer: f32,
    /// Inner edge of the raised scale ring.
    scale_in: f32,
    tick_out: f32,
    /// Tick (length, width): every 1,000, every 500 and every 100 rpm.
    major: (f32, f32),
    half: (f32, f32),
    minor: (f32, f32),
    /// The red band on the outer edge of the scale: radius and width.
    red_band: (f32, f32),
    /// The numerals' cap (each sits tucked inside the bars by its own width).
    num_cap: f32,
    hub_r: f32,
    needle_tail: f32,
    needle_tip: f32,
    needle_half_w: f32,
    /// The shift ring: radius, lens size (along × across), first angle and sweep (degrees).
    led_r: f32,
    led_size: Vec2,
    led_start_deg: f32,
    led_sweep_deg: f32,
    /// The gear window: a round porthole in the dead zone (distance of its centre from the hub,
    /// its radius, and the outer radius of its rendered rim).
    window_r: f32,
    porthole: f32,
    porthole_rim: f32,
    gear_cap: f32,
    /// The "+" upshift cue beside the gear, inside the window: its offset from the window's
    /// centre in porthole radii, and its cap.
    plus_at: Vec2,
    plus_cap: f32,
    /// The logo zone on the hub's vertical: the distance of its centre from the hub below the
    /// hub (where it sits when that is clear of the window) and above it, its size, a narrower
    /// zone a little lower that clears a window lower down, and the legend's drop below it.
    logo_below: f32,
    logo_above: f32,
    logo_size: Vec2,
    logo_below_narrow: f32,
    logo_narrow: Vec2,
    legend_drop: f32,
    /// Tube centres: far left, left, right, far right.
    tubes: [Vec2; 4],
    side_open: f32,
    side_outer: f32,
    side_scale_in: f32,
    side_tick_out: f32,
    side_num_cap: f32,
    speed_cap: f32,
    pointer: (f32, f32),
    far_open: f32,
    far_outer: f32,
    far_arc_r: f32,
    far_pointer: (f32, f32),
    /// Crystal cross-sections (rise, rounded edge, glass below the rim): rev counter, side tubes.
    tach_crystal: (f32, f32, f32),
    side_crystal: (f32, f32, f32),
    /// The magnifier over the gear window: rise and glass below its rim.
    cyclops: (f32, f32),
}

const GEOM: Geom = Geom {
    tach: vec2(0.0, -40.0),
    tach_open: 214.0,
    tach_outer: 248.0,
    scale_in: 150.0,
    tick_out: 204.0,
    major: (20.0, 6.0),
    half: (13.0, 3.0),
    minor: (8.0, 2.0),
    red_band: (198.0, 5.0),
    num_cap: 32.0,
    hub_r: 20.0,
    needle_tail: 26.0,
    needle_tip: 200.0,
    needle_half_w: 4.5,
    led_r: 231.0,
    led_size: vec2(36.0, 12.0),
    led_start_deg: 205.0,
    led_sweep_deg: 130.0,
    window_r: 116.0,
    porthole: 70.0,
    porthole_rim: 76.0,
    gear_cap: 96.0,
    plus_at: vec2(0.74, -0.30),
    plus_cap: 24.0,
    logo_below: 100.0,
    logo_above: 84.0,
    logo_size: vec2(160.0, 20.0),
    logo_below_narrow: 104.0,
    logo_narrow: vec2(120.0, 18.0),
    legend_drop: 24.0,
    tubes: [
        vec2(-612.0, 16.0),
        vec2(-396.0, -8.0),
        vec2(396.0, -8.0),
        vec2(612.0, 16.0),
    ],
    side_open: 160.0,
    side_outer: 184.0,
    side_scale_in: 112.0,
    side_tick_out: 152.0,
    side_num_cap: 17.0,
    speed_cap: 66.0,
    pointer: (126.0, 156.0),
    far_open: 92.0,
    far_outer: 108.0,
    far_arc_r: 76.0,
    far_pointer: (68.0, 88.0),
    tach_crystal: (8.0, 9.0, 6.0),
    side_crystal: (6.0, 8.0, 5.0),
    cyclops: (21.0, 6.0),
};

impl Geom {
    /// Angle of shift lens `i` (radians).
    fn led_angle(&self, i: usize) -> f32 {
        (self.led_start_deg + self.led_sweep_deg * i as f32 / (LEDS - 1) as f32).to_radians()
    }

    /// True for the lenses that strobe on the limiter.
    fn crest(&self, i: usize) -> bool {
        (self.led_angle(i) + FRAC_PI_2 - TAU)
            .abs()
            .min((self.led_angle(i) + FRAC_PI_2).abs())
            <= CREST_DEG.to_radians()
    }

    /// The gear window's centre for a scale: in the middle of the dead zone, where Porsche puts
    /// its readout, so the needle never crosses it.
    fn window_centre(&self, scale: &DialScale) -> Vec2 {
        Vec2::polar(self.tach, self.window_r, scale.dead_centre())
    }

    /// The "+" cue's centre: beside the gear, inside the window.
    fn plus_centre(&self, scale: &DialScale) -> Vec2 {
        self.window_centre(scale) + self.plus_at * self.porthole
    }

    /// The logo zone, drawn upright on the hub's vertical: below the hub when it and its legend
    /// clear the window there (full width, else a narrower zone a little lower), else above it
    /// (the classic face, as on the real dial, and a track face whose window sits low). Below,
    /// the needle crosses it only near idle; above, on the track face, it stands under the
    /// needle at the shift point, where the 992.2 puts its model badge.
    fn logo_zone(&self, scale: &DialScale) -> Rect {
        let window = self.window_centre(scale);
        let clear = |zone: Rect| {
            distance_to_rect(window, zone).min(distance_to_rect(window, self.legend_box(zone)))
                > self.porthole_rim + LOGO_CLEAR
        };
        let below = Rect::from_center_size(self.tach + vec2(0.0, self.logo_below), self.logo_size);
        let narrow = Rect::from_center_size(self.tach + vec2(0.0, self.logo_below_narrow), self.logo_narrow);
        if clear(below) {
            below
        } else if clear(narrow) {
            narrow
        } else {
            Rect::from_center_size(self.tach - vec2(0.0, self.logo_above), self.logo_size)
        }
    }

    /// The box "1/min × 1000" takes under the logo `zone` (its text measures under 110 × 14).
    fn legend_box(&self, zone: Rect) -> Rect {
        Rect::from_center_size(zone.center() + vec2(0.0, self.legend_drop), LEGEND_SIZE)
    }
}

/// The most room "1/min × 1000" takes at the tokens' nano cap (px).
const LEGEND_SIZE: Vec2 = vec2(110.0, 14.0);

/// The shortest distance from `p` to the rectangle `r` (0 inside it).
fn distance_to_rect(p: Vec2, r: Rect) -> f32 {
    let q = vec2(p.x.clamp(r.min.x, r.max.x), p.y.clamp(r.min.y, r.max.y));
    (q - p).length()
}

/// Where numeral `label` sits at `angle` so that its box stays [`NUM_GAP`] inside radius `inner`
/// round `c`. Its width is measured in tabular cells (no shaping; a little wide for a "1", so
/// the gap only grows), its height is the style's cap.
fn tucked(p: &Painter<'_>, c: Vec2, inner: f32, angle: f32, label: &str, style: &TextStyle) -> Vec2 {
    let width = p.fonts().tabular_cell(style) * label.len() as f32;
    let reach = angle.cos().abs() * width * 0.5 + angle.sin().abs() * style.cap * 0.5;
    Vec2::polar(c, inner - NUM_GAP - reach, angle)
}

/// Every numeral (1) or every other one (2): two-digit numerals `spacing` px apart along their
/// arc need about one and a half tabular cells of ink each and two gaps between them.
fn numeral_step(spacing: f32, cell: f32) -> usize {
    if spacing < cell * 1.5 + NUM_GAP * 2.0 {
        2
    } else {
        1
    }
}

/// What a tube shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Content {
    Speed,
    GForce,
    Power,
    Boost,
    Off,
}

impl Content {
    fn parse(value: Option<&str>, default: Content) -> Content {
        match value.map(str::trim) {
            Some(v) if v.eq_ignore_ascii_case("speed") => Content::Speed,
            Some(v) if v.eq_ignore_ascii_case("g") || v.eq_ignore_ascii_case("gforce") => Content::GForce,
            Some(v) if v.eq_ignore_ascii_case("power") => Content::Power,
            Some(v) if v.eq_ignore_ascii_case("boost") => Content::Boost,
            Some(v) if v.eq_ignore_ascii_case("off") || v.eq_ignore_ascii_case("hidden") => Content::Off,
            _ => default,
        }
    }
}

/// The dash's own options, from the config's extra keys (hot-reloaded; the configurator and the
/// OSD set them):
/// - `PorscheAnalogScale`: `Track` (default, the shift rpm at 12) or `Classic`.
/// - `PorscheAnalogFarLeft`, `PorscheAnalogLeft`, `PorscheAnalogRight`, `PorscheAnalogFarRight`:
///   `Speed`, `G`, `Power`, `Boost` or `Off` (defaults Power, Speed, G, Boost).
/// - `PorscheAnalogNumerals`: `White` (default) or `Heritage` (the 356 green).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Options {
    mode: Mode,
    tubes: [Content; 4],
    heritage: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: Mode::Track,
            tubes: [Content::Power, Content::Speed, Content::GForce, Content::Boost],
            heritage: false,
        }
    }
}

impl Options {
    fn from_config(cfg: &AppConfig) -> Self {
        let get = |k: &str| cfg.extra.get(k).and_then(|v| v.as_str());
        let d = Self::default();
        Self {
            mode: match get("PorscheAnalogScale").map(str::trim) {
                Some(v) if v.eq_ignore_ascii_case("classic") => Mode::Classic,
                _ => Mode::Track,
            },
            tubes: [
                Content::parse(get("PorscheAnalogFarLeft"), d.tubes[0]),
                Content::parse(get("PorscheAnalogLeft"), d.tubes[1]),
                Content::parse(get("PorscheAnalogRight"), d.tubes[2]),
                Content::parse(get("PorscheAnalogFarRight"), d.tubes[3]),
            ],
            heritage: get("PorscheAnalogNumerals").is_some_and(|v| v.trim().eq_ignore_ascii_case("heritage")),
        }
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces the numerals).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    dial: FontId,
    gear: FontId,
    speed: FontId,
    label: FontId,
    wide: FontId,
    /// The display role at creation, to notice a user font override.
    role_display: FontId,
}

/// What the logo zone shows ([`zone_art`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZoneArt<I> {
    /// Nothing: the logo is turned off, or the car's dial is still loading.
    Nothing,
    /// An image, engraved on the dial: the car's logo, else this dash's maker wordmark.
    Image(I),
    /// The brand or custom name ([`logo_zone`]).
    Names,
}

/// What the logo zone shows: `image` (the car's logo, [`Logo::image`]), else `maker` (the
/// maker's wordmark when the car is one of theirs), else the names in `logo`. While the car's
/// dial is loading the zone stays empty, so neither the maker's wordmark nor a name shows first
/// and the car's dial a moment later. The images are any handle, so tests need no kit.
fn zone_art<I>(image: Option<I>, maker: Option<I>, logo: &Logo) -> ZoneArt<I> {
    let off = image.is_none() && logo.brand_name.trim().is_empty() && logo.custom_name.trim().is_empty();
    if off || logo.loading {
        return ZoneArt::Nothing;
    }
    match image.or(maker) {
        Some(image) => ZoneArt::Image(image),
        None => ZoneArt::Names,
    }
}

/// The maker whose wordmark (`assets/logos/porsche_dial.png`) this dial carries when the car is
/// one of theirs (compared with the car's brand name in `update`, never in `draw`).
const MAKER: &str = "Porsche";

/// The Porsche Analog cluster.
#[derive(Debug, Clone)]
pub struct PorscheAnalog {
    faces: Faces,
    /// The faces chosen at creation, put back when a user's display font is cleared.
    own_faces: Faces,
    sprites: Sprites,
    /// The dash maker's wordmark, printed in the logo zone when the car is the maker's and the
    /// host sends no logo image of its own.
    maker: Option<ImageId>,
    /// True when the car's brand is the dash's maker.
    maker_car: bool,
    /// True until the first update: the first frame takes the car's state as it is.
    fresh: bool,
    sig: Signals,
    palette: Palette,
    view: View,
    opts: Options,
    calm: bool,
    logo: Logo,
    /// The driver's name from the config, for the greeting (it shows whether or not the logo
    /// does; copied only when it changes).
    driver: String,
    /// The scale this car should have, the animated start angle (radians, unwrapped) and where
    /// it is turning to.
    scale: DialScale,
    turn: Spring,
    scale_goal: f32,
    turn_ready: bool,
    /// The needle (rpm), with feed-forward, and what it is fed (the peak of a bounce off the
    /// limiter, else the rpm).
    needle: Spring,
    hold: LimiterHold,
    shift: ShiftLights,
    /// The strobe's own clock (s): from the frame the shift point is reached, so that frame and
    /// the next two are lit; and whether the ring is over the shift point (latched, with
    /// [`SHIFT_HYSTERESIS_RPM`]).
    strobe_t: f32,
    was_over: bool,
    /// True while the rpm is over the ring's first stage (or the ring is over the shift point):
    /// below it every lens is dark at once, whatever tail the shared ring's release leaves.
    ring_lit: bool,
    /// Set by an upshift, cleared once the rpm falls below the shift point (`upshift_quiet`).
    quiet: bool,
    gear: HeroGear,
    rim_flash: Tween,
    speed: TabularSpeed,
    speed_needle: Spring,
    g: [Spring; 2],
    g_target: [f32; 2],
    power_hp: Smoothed<f32>,
    power_peak: f32,
    boost_psi: Smoothed<f32>,
    boost_peak: f32,
    /// The printed tops of the power (hp) and boost (psi) scales: they follow the peaks only
    /// while the driver is off the throttle, so a scale never re-ranges mid-pull.
    power_top: f32,
    boost_top: f32,
    /// The digits on show for power (hp), boost (psi) and the G magnitude (g).
    power_digits: Readout,
    boost_digits: Readout,
    g_digits: Readout,
    car: i32,
    /// 1 while parked (the greeting shows), 0 while driving.
    greet: Spring,
    greet_target: f32,
    light: VirtualLight,
    glass: OpticalGlass,
    show: Tween,
    micro_cap: f32,
    unit_cap: f32,
    /// The smallest text the tokens allow (cap px).
    nano_cap: f32,
    tracking: f32,
}

impl PorscheAnalog {
    /// Builds the cluster: faces, springs, the rendered hardware and the maker's wordmark.
    /// Every missing file has a procedural fallback.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let sprites = Sprites::load(setup);
        let maker = hardware::load(setup, &["logos", "porsche_dial.png"]);
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "classicAnalog");
        let display = setup.fonts.role(FontRole::Display);
        let dial_role = setup.fonts.role(FontRole::Dial);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            dial: setup
                .fonts
                .find("Barlow Semi Condensed", 600, false)
                .unwrap_or(dial_role),
            gear: setup.fonts.find("Barlow Semi Condensed", 700, false).unwrap_or(display),
            speed: setup.fonts.find("Barlow Semi Condensed", 600, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let opts = Options::default();
        let scale = DialScale::new(opts.mode, 0.0, 0.0);
        let mut show = Tween::new(t.durations.materialize);
        show.restart();
        let turn_spec = SpringSpec {
            period: TURN_PERIOD_S,
            zeta: 1.0,
        };
        Self {
            faces,
            own_faces: faces,
            sprites,
            maker,
            maker_car: false,
            fresh: true,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            opts,
            calm: false,
            logo: Logo::default(),
            driver: String::new(),
            scale,
            turn: Spring::new(turn_spec, scale.start),
            scale_goal: scale.start,
            turn_ready: false,
            needle: Spring::new(t.springs.needle, 0.0),
            hold: LimiterHold::default(),
            shift: ShiftLights::new(LEDS as u32),
            strobe_t: 0.0,
            was_over: false,
            ring_lit: false,
            quiet: false,
            gear: HeroGear::new(t),
            rim_flash: Tween::new(RIM_FLASH_S),
            speed: TabularSpeed::new(),
            speed_needle: Spring::new(t.springs.needle, 0.0),
            g: [Spring::new(t.springs.g_ball, 0.0), Spring::new(t.springs.g_ball, 0.0)],
            g_target: [0.0; 2],
            power_hp: Smoothed::new(0.0, 0.10),
            power_peak: 0.0,
            boost_psi: Smoothed::new(0.0, 0.10),
            boost_peak: 0.0,
            power_top: 0.0,
            boost_top: 0.0,
            power_digits: Readout::new(),
            boost_digits: Readout::new(),
            g_digits: Readout::new(),
            car: i32::MIN,
            greet: Spring::new(t.springs.default, 0.0),
            greet_target: 0.0,
            light: VirtualLight::new(t),
            glass: t.optical_glass(&palette),
            show,
            micro_cap: t.type_scale.micro_label,
            unit_cap: t.type_scale.unit,
            nano_cap: t.type_scale.nano,
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The scale as drawn this frame (turned by the animation).
    fn shown_scale(&self) -> DialScale {
        self.scale.turned(self.turn.value())
    }

    /// The rpm the needle shows this frame.
    fn needle_rpm(&self) -> f32 {
        self.needle.value()
    }

    /// True from an upshift until the rpm first falls below the shift point (at most
    /// [`UPSHIFT_QUIET_S`]): the shift ring and the "+" stay dark so the new gear never shows
    /// under a "shift again" cue, and light again as soon as the rpm says so.
    fn upshift_quiet(&self) -> bool {
        self.quiet
    }

    /// How far the parts have materialised (0..=1, eased out).
    fn shown(&self) -> f32 {
        let k = 1.0 - self.show.progress();
        1.0 - k * k * k
    }

    /// The numeral ink: white, or the Heritage green (from the palette roles).
    fn numeral_ink(&self, pal: &Palette) -> Color {
        if self.opts.heritage {
            pal.roles.go.lerp(pal.roles.ink, HERITAGE_INK)
        } else {
            pal.roles.ink
        }
    }

    /// The needles' lacquer: the palette's primary, or white with the Heritage numerals, as on
    /// the 356-style dials.
    fn needle_ink(&self, pal: &Palette) -> Color {
        if self.opts.heritage {
            pal.roles.ink
        } else {
            pal.primary_text
        }
    }

    /// The colour of shift lens `i` this frame. On the limiter only the crest strobes; the flanks
    /// turn steady to the shift colour, so the limiter differs from "nearly there" in every frame,
    /// the strobe's dark phase included.
    fn lens_color(&self, g: &Geom, i: usize, pal: &Palette) -> Option<Color> {
        if !self.ring_lit {
            return None;
        }
        if self.shift.over_shift() && !g.crest(i) {
            return Some(pal.shift);
        }
        self.shift.color_of(i, pal)
    }

    /// Feeds the needle: the rpm with its feed-forward, or the held peak of a bounce off the
    /// limiter (`readout::LimiterHold`). Just after a gear change the feed-forward is off
    /// ([`SHIFT_LEAD_OFF_S`]).
    fn feed_needle(&mut self, sig: &Signals, dt: f32) {
        let (target, rate) = self.hold.feed(sig.rpm, sig.rpm_rate, sig.max_rpm, dt);
        let after_shift = self.gear.since_change() < SHIFT_LEAD_OFF_S;
        let rate = if after_shift { 0.0 } else { rate };
        if self.hold.holding() && (self.needle.value() - target).abs() < HOLD_SNAP {
            // Against the stop the needle comes to rest on the held peak at once, instead of
            // ringing round it for a few frames (a 15 Hz flicker on the limiter).
            self.needle.reset(target);
        } else {
            let before = self.needle.value() - target;
            self.needle.update_tracking(target, rate, dt);
            // Chasing the step of a shift, the needle comes to rest on the rpm rather than
            // swinging past it (the spring's ring after a 2,000 rpm drop is about 55 rpm).
            if after_shift && before * (self.needle.value() - target) < 0.0 {
                self.needle.reset(target);
            }
        }
    }
}

impl Cluster for PorscheAnalog {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        // The dash takes the car's state as it is, instead of animating into it from where it
        // was, on its first frame, on a change of the race flag (the OSD's dyno frames are the
        // only race-off frames it sees: menus and pauses hide it and skip its updates) and on
        // the first frame after such a gap.
        let race_edge = sig.race_on != self.sig.race_on;
        let snap = self.fresh || race_edge || input.dt >= RESUME_DT;
        self.fresh = false;
        let unit_edge = sig.unit != self.sig.unit;
        // Back from the OSD's dyno frames: the glass grows in once more.
        if race_edge && sig.race_on && !self.show.running() {
            self.show.restart();
        }
        self.sig = sig;
        self.palette = *input.palette;
        self.calm = input.reduced_flash;
        self.view = input.view;
        self.opts = Options::from_config(input.config);
        if input.fonts.display != self.faces.role_display {
            // The user's display font replaces every numeral (the gear, the speed and the
            // dials); clearing it brings back this dash's own faces.
            self.faces = if input.fonts.display == self.own_faces.role_display {
                self.own_faces
            } else {
                Faces {
                    gear: input.fonts.display,
                    speed: input.fonts.display,
                    dial: input.fonts.display,
                    role_display: input.fonts.display,
                    ..self.own_faces
                }
            };
        }
        // Copied only when they change (a car of another brand, or an edited name).
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
            self.maker_car = self.logo.brand_name.trim().eq_ignore_ascii_case(MAKER);
        }
        if self.driver != input.config.custom_driver_name {
            self.driver.clone_from(&input.config.custom_driver_name);
        }

        // The scale for this car; when it changes mid-race the scale turns to the new shift point.
        let target = DialScale::new(self.opts.mode, sig.max_rpm, sig.shift_rpm);
        if !self.turn_ready || snap {
            self.turn.reset(target.start);
            self.scale_goal = target.start;
            self.turn_ready = true;
        } else if target != self.scale {
            let from = self.turn.value();
            self.scale_goal = from + scale::shortest_turn(from, target.start);
        }
        self.scale = target;
        self.turn.update(self.scale_goal, dt);

        // The gear first: a gear change ends any limiter hold and, for a moment, the needle's
        // feed-forward. Snapped frames show the engaged gear at rest, with no entry motion, no
        // rim flash and no upshift quiet: the glyph's motion is settled before the first draw.
        let rpm = if sig.rpm.is_finite() { sig.rpm.max(0.0) } else { 0.0 };
        if snap {
            for _ in 0..GEAR_SETTLE_STEPS {
                self.gear.update(sig.gear, 0.0, GEAR_SETTLE_DT);
            }
        } else {
            let changed = sig.gear != self.gear.gear();
            // No limiter pulse: on this dash the limiter is the lenses' job.
            self.gear.update(sig.gear, 0.0, dt);
            if changed {
                self.quiet = self.gear.was_upshift();
                self.hold.release();
                if !self.calm {
                    self.rim_flash.restart();
                }
            }
        }
        if snap || rpm < sig.shift_rpm || self.gear.since_change() >= UPSHIFT_QUIET_S {
            self.quiet = false;
        }

        // The needle: rpm on the tokens' needle spring with feed-forward (held through a bounce
        // off the limiter).
        if snap {
            self.hold = LimiterHold::default();
            self.needle.reset(rpm);
        }
        self.feed_needle(&sig, dt);

        // The shift ring follows the needle while it rises (the plumb line) and the raw rpm when
        // it falls, so an upshift clears it at once. It needs a known limiter (a car that sends
        // none leaves it dark), it stays over the shift point until the rpm falls
        // [`SHIFT_HYSTERESIS_RPM`] under it, and its strobe runs on its own clock from the frame
        // the shift point is reached.
        let shown = self.needle_rpm();
        let feed = if rpm >= shown { shown } else { rpm };
        let known = sig.max_rpm.is_finite()
            && sig.max_rpm > readout::LIMITER_BAND
            && sig.shift_rpm.is_finite()
            && sig.shift_rpm > 0.0;
        let frac = if known { feed / sig.shift_rpm } else { 0.0 };
        let over = known
            && if self.was_over {
                feed >= sig.shift_rpm - SHIFT_HYSTERESIS_RPM
            } else {
                frac >= 1.0
            };
        self.strobe_t = if over && self.was_over { self.strobe_t + dt } else { 0.0 };
        self.was_over = over;
        self.ring_lit = over || frac > self.palette.stages[0];
        let ring = Signals {
            rpm,
            shift_fraction: if over { frac.max(1.0) } else { frac },
            ..sig
        };
        self.shift
            .update(&ring, &self.palette, self.strobe_t, dt, input.reduced_flash);
        self.rim_flash.update(dt);
        self.show.update(dt);

        // Speed, its pointer, G (felt force, at most 1.5 g in any direction), power and boost.
        self.speed.update(sig.speed);
        if snap || unit_edge {
            // A unit change switches the face at once, so the pointer does too.
            self.speed_needle.reset(sig.speed);
        }
        self.speed_needle.update(sig.speed, dt);
        let finite = |g: f32| if g.is_finite() { g } else { 0.0 };
        // Clamped by length, so a combined load never pushes the dot past the outer ring.
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
        let car_change = sig.car_ordinal != self.car;
        if car_change {
            self.car = sig.car_ordinal;
            self.power_peak = 0.0;
            self.boost_peak = 0.0;
            self.power_top = 0.0;
            self.boost_top = 0.0;
        }
        let power = input.telemetry.power_watts;
        let hp = if power.is_finite() {
            (power / 745.7).max(0.0)
        } else {
            0.0
        };
        let boost = input.telemetry.boost_psi;
        let psi = if boost.is_finite() {
            boost.clamp(-15.0, 60.0)
        } else {
            0.0
        };
        if snap || car_change {
            // Nothing of the last car's (or the dyno's) power carries over into this one's peak.
            self.power_hp.reset(hp);
            self.boost_psi.reset(psi);
            self.power_digits = Readout::new();
            self.boost_digits = Readout::new();
            self.g_digits = Readout::new();
        }
        let hp = self.power_hp.update(hp, dt);
        let psi = self.boost_psi.update(psi, dt);
        // The peaks are learned only in a race (the OSD's dyno frames would teach them its own
        // car), and the printed scale tops follow them only off the throttle, so a scale never
        // re-ranges mid-pull: on the first pull after a build or a car change the gauge pegs at
        // its end, as a real one does, and the scale is printed for the car at the first lift.
        if sig.race_on {
            self.power_peak = self.power_peak.max(hp);
            self.boost_peak = self.boost_peak.max(psi);
        }
        if snap || unit_edge || self.power_top == 0.0 || sig.throttle < RANGE_LIFT {
            self.power_top = tubes::power_top(self.power_peak, sig.unit);
            self.boost_top = tubes::boost_top(self.boost_peak, sig.unit);
        }
        // The digits refresh at a few hertz, one step of the last shown digit at a time.
        let (power_step, boost_step) = match sig.unit {
            SpeedUnit::Mph => (1.0, 0.1),
            SpeedUnit::Kmh => (1.0 / tubes::HP_TO_KW, 0.1 / tubes::PSI_TO_BAR),
        };
        self.power_digits.update(self.power_hp.value(), power_step, dt);
        self.boost_digits.update(self.boost_psi.value(), boost_step, dt);
        let (g_mag, _) = self.g_reading();
        self.g_digits.update(g_mag, 0.01, dt);

        // Parked in neutral: the right tube greets the driver. Once the car moves the greeting
        // goes at once (the spring jumps to the half-way point, where the greeting has gone and
        // the live content starts to fade in), as the real car drops it when you drive off.
        self.greet_target = if sig.speed.abs() < PARKED_SPEED && sig.gear == Gear::Neutral {
            1.0
        } else {
            0.0
        };
        if snap {
            self.greet.reset(self.greet_target);
        } else if self.greet_target == 0.0 && self.greet.value() > 0.5 {
            self.greet.reset(0.5);
        }
        self.greet.update(self.greet_target, dt);

        // The light rig and the glass.
        self.light.update(sig.g_lat, dt);
        let op = &input.tokens.optics;
        let mut glass = input.tokens.optical_glass(&self.palette);
        glass.aim(self.light.angle(), op.key_elevation, Vec2::ZERO);
        glass.materialize = self.shown();
        self.glass = glass;
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &GEOM;
        let pal = self.palette;
        let w = hardware::light_weights(self.light.angle());
        let hood = self.view == View::Hood;
        p.save();
        if hood {
            p.scale(HOOD_SCALE);
        }
        // Back to front: the outer tubes, the side tubes and their crystals, then the rev
        // counter over them (its bezel covers the side crystals' inner edges) and its glass.
        for slot in [0usize, 3, 1, 2] {
            self.draw_tube(p, g, slot, &pal, w);
        }
        self.draw_side_glass(p, g);
        self.draw_tach(p, g, &pal, w);
        self.draw_tach_glass(p, g);
        p.restore();
    }

    fn animating(&self) -> bool {
        self.show.running()
            || self.rim_flash.running()
            || (self.ring_lit && self.shift.animating())
            || self.gear.animating()
            || !self.turn.settled(self.scale_goal, 1e-4)
            || !self.needle.settled(self.sig.rpm, 1.0)
            || !self.speed_needle.settled(self.sig.speed, 0.05)
            || !self.g[0].settled(self.g_target[0], 0.002)
            || !self.g[1].settled(self.g_target[1], 0.002)
            || !self.greet.settled(self.greet_target, 0.002)
    }
}

impl PorscheAnalog {
    /// The rev counter: hardware, scale, logo zone, gear window, shift ring, needle and hub cap.
    fn draw_tach(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, w: [f32; 3]) {
        let c = g.tach;
        let scale = self.shown_scale();
        // A soft contact shadow where the rev counter's bezel overhangs the side tubes.
        p.shadow(
            &Shape::circle(c, g.tach_outer - 2.0),
            Shadow::new(vec2(0.0, 6.0), 12.0, pal.roles.oled_black.with_alpha(0.8)),
        );
        if !self.sprites.draw(p, Kind::Tach, c, w) {
            hardware::fallback_tube(p, c, g.tach_open, g.tach_outer, g.scale_in, pal);
        }
        // Printed parts are always there at full strength: only the glass materialises.
        self.draw_scale(p, g, &scale, pal);
        self.draw_logo(p, g, pal);
        self.draw_window(p, g, &scale, pal, w);
        self.draw_leds(p, g, pal);
        self.draw_needle(p, g, &scale, pal, w);
    }

    /// Ticks every 100 rpm (a bar every 1,000, a half tick every 500), the red band and red
    /// ticks from the limiter, and the numerals.
    fn draw_scale(&self, p: &mut Painter<'_>, g: &Geom, scale: &DialScale, pal: &Palette) {
        let c = g.tach;
        let ink = pal.roles.ink;
        let red = pal.roles.warning;
        // No limiter known (a menu, or a car that sends none): no red band, as the hold does.
        let limit = if self.sig.max_rpm.is_finite() && self.sig.max_rpm > readout::LIMITER_BAND {
            self.sig.max_rpm
        } else {
            scale.top
        };
        let hundreds = (scale.top / 100.0).round().clamp(0.0, 200.0) as u32;
        for k in 0..=hundreds {
            let rpm = k as f32 * 100.0;
            let a = scale.angle(rpm);
            let (len, width, level) = if k % 10 == 0 {
                (g.major.0, g.major.1, LEVEL_NUMERAL)
            } else if k % 5 == 0 {
                (g.half.0, g.half.1, LEVEL_HALF)
            } else {
                (g.minor.0, g.minor.1, LEVEL_MINOR)
            };
            let color = if rpm >= limit - 1.0 { red } else { ink.with_alpha(level) };
            p.fill(
                &Shape::Line {
                    a: Vec2::polar(c, g.tick_out - len, a),
                    b: Vec2::polar(c, g.tick_out, a),
                    width_a: width.max(1.0),
                    width_b: width.max(1.0),
                    cap: Cap::Butt,
                },
                color,
            );
        }
        // The red band from the limiter to the top, on the outer edge inside the ticks' ends.
        if limit < scale.top {
            let a0 = scale.angle(limit);
            p.fill(
                &Shape::arc(
                    c,
                    g.red_band.0,
                    g.red_band.1,
                    a0,
                    scale.angle(scale.top) - a0,
                    Cap::Butt,
                ),
                red,
            );
        }
        let style = TextStyle::new(self.faces.dial, g.num_cap)
            .paint(self.numeral_ink(pal).with_alpha(LEVEL_NUMERAL))
            .align(Align::Center, VAlign::CapMiddle);
        let inner = g.tick_out - g.major.0;
        // Every thousand, or every other one where two-digit numerals would run together (a
        // face to 14,000 or more).
        let spacing = (inner - NUM_GAP - g.num_cap * 0.5) * scale.per_rpm() * 1000.0;
        let step = numeral_step(spacing, p.fonts().tabular_cell(&style));
        for k in (0..=scale.thousands()).step_by(step) {
            let Some(label) = NUMERALS.get(k as usize) else {
                break;
            };
            let a = scale.angle(k as f32 * 1000.0);
            let at = tucked(p, c, inner, a, label, &style);
            p.text(at, label, &style);
        }
    }

    /// The logo zone (the host's logo image, else this dash's maker wordmark on the maker's own
    /// cars, else the brand or custom name; nothing when the logo is turned off or while the
    /// car's dial is loading, [`zone_art`]) and
    /// "1/min × 1000" under it. Its side of the hub follows the car's scale, not the turning
    /// one, so it never jumps in the middle of a turn.
    fn draw_logo(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let zone = g.logo_zone(&self.scale);
        let maker = if self.maker_car { self.maker } else { None };
        match zone_art(self.logo.image, maker, &self.logo) {
            ZoneArt::Nothing => {}
            ZoneArt::Image(image) => {
                // An image, printed on the dial: engraved (a dark copy 1 px below-right first).
                let ink = pal.roles.ink.with_alpha(215.0 / 255.0);
                let shadow = fh6_kit::logo::LogoStyle {
                    tint: pal.roles.oled_black.with_alpha(0.8),
                    ..fh6_kit::logo::LogoStyle::new(self.faces.wide)
                };
                let only = Logo {
                    image: Some(image),
                    ..Logo::default()
                };
                let off = Rect::from_points(zone.min + vec2(1.0, 1.0), zone.max + vec2(1.0, 1.0));
                fh6_kit::logo::draw(p, off, &only, &shadow);
                let style = fh6_kit::logo::LogoStyle {
                    tint: ink,
                    ..fh6_kit::logo::LogoStyle::new(self.faces.wide)
                };
                fh6_kit::logo::draw(p, zone, &only, &style);
            }
            ZoneArt::Names => logo_zone(p, zone, &self.logo, self.faces.wide, LogoVariant::Dial, pal),
        }
        let legend = TextStyle::new(self.faces.label, self.nano_cap)
            .paint(pal.roles.ink_label)
            .tracking(self.tracking * 0.5)
            .align(Align::Center, VAlign::CapMiddle);
        p.text(zone.center() + vec2(0.0, g.legend_drop), "1/min × 1000", &legend);
    }

    /// The gear window in the dead zone: a black porthole, the gear, the "+" cue from the shift
    /// point, and the porthole's rendered rim (flashing once on a gear change).
    fn draw_window(&self, p: &mut Painter<'_>, g: &Geom, scale: &DialScale, pal: &Palette, w: [f32; 3]) {
        let c = g.window_centre(scale);
        // The porthole is black; its depth is the rendered rim and the magnifier over it.
        p.fill(&Shape::circle(c, g.porthole), pal.roles.oled_black);
        // Printed under the magnifier, not glowing (bloom fringed the glyph through the glass),
        // and held inside the porthole while it slides in on a gear change.
        // Neutral at full ink here (the shared glyph dims it): parked, the gear still leads.
        let gear = self.gear.gear();
        let ink = match gear {
            Gear::Neutral => pal.roles.ink,
            _ => self.gear.color(pal),
        };
        let cap = if gear.label().len() > 1 {
            g.gear_cap * TWO_DIGIT_GEAR
        } else {
            g.gear_cap
        };
        p.scoped(|p| {
            p.clip(Rect::from_center_size(c, Vec2::splat(g.porthole * 2.0)), g.porthole);
            self.gear.draw_with(p, c, self.faces.gear, cap, ink);
        });
        // Porsche's upshift cue (manual gearbox: "+" beside the gear once every segment is red),
        // never over a gear just shifted into.
        if self.shift.over_shift() && !self.upshift_quiet() {
            let style = TextStyle::new(self.faces.gear, g.plus_cap)
                .paint(pal.shift)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(g.plus_centre(scale), "+", &style);
        }
        if !self.sprites.draw_window(p, c, w) {
            p.stroke(
                &Shape::circle(c, g.porthole + 2.0),
                pal.roles.rim_specular.with_alpha(0.45),
                Stroke::rim_px(1.5),
            );
        }
        if self.rim_flash.running() {
            let k = 1.0 - self.rim_flash.progress();
            p.stroke(
                &Shape::circle(c, g.porthole + 2.0),
                pal.roles.ink.with_alpha(0.6 * k),
                Stroke::new(2.5),
            );
        }
    }

    /// The shift ring's lit lenses (the rendered plate holds the unlit smoked lenses). Each lens
    /// glows tightly and feeds no bloom, so no haze hangs over an unlit lens and the strobe stays
    /// inside its band.
    fn draw_leds(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let c = g.tach;
        let fallback = !self.sprites.has(Kind::Tach);
        // Dark for the moment after an upshift while the rpm is still over the shift point.
        let quiet = self.upshift_quiet();
        for i in 0..LEDS {
            let a = g.led_angle(i);
            let at = Vec2::polar(c, g.led_r, a);
            let lens = Rect::from_center_size(Vec2::ZERO, g.led_size);
            let color = if quiet { None } else { self.lens_color(g, i, pal) };
            if color.is_none() && !fallback {
                continue;
            }
            p.scoped(|p| {
                p.translate(at);
                p.rotate_about(a + FRAC_PI_2, Vec2::ZERO);
                let shape = Shape::round_rect(lens, g.led_size.y * 0.5);
                match color {
                    Some(col) => {
                        p.glow(&shape, col, Glow::new(LENS_GLOW.0, LENS_GLOW.1));
                        p.fill(&shape, col);
                    }
                    None => p.fill(&shape, pal.roles.ink.with_alpha(6.0 / 255.0)),
                }
            });
        }
    }

    /// The needle (rendered lacquer tinted with the palette, or a vector blade) with its shadow,
    /// then the polished hub cap over its root.
    fn draw_needle(&self, p: &mut Painter<'_>, g: &Geom, scale: &DialScale, pal: &Palette, w: [f32; 3]) {
        let c = g.tach;
        let a = scale.angle(self.needle_rpm());
        let dir = Vec2::polar(Vec2::ZERO, 1.0, a);
        let tail = c - dir * g.needle_tail;
        let tip = c + dir * g.needle_tip;
        let blade = Shape::Line {
            a: tail,
            b: tip,
            width_a: g.needle_half_w * 2.0,
            width_b: 1.5,
            cap: Cap::Round,
        };
        // The needle floats above the dial: its shadow falls away from the light.
        let away = Vec2::polar(Vec2::ZERO, 6.0, self.light.angle() + PI);
        p.shadow(&blade, Shadow::new(away, 3.0, pal.roles.oled_black.with_alpha(0.75)));
        let tint = self.needle_ink(pal);
        if !self.sprites.draw_needle(p, c, a, self.light.angle(), tint) {
            p.fill(&blade, tint);
            // A lit ridge down the middle.
            p.fill(
                &Shape::Line {
                    a: c,
                    b: c + dir * (g.needle_tip - 6.0),
                    width_a: 1.4,
                    width_b: 1.0,
                    cap: Cap::Round,
                },
                pal.roles.ink.with_alpha(0.55),
            );
        }
        // The illuminated hub: the needle's light leaks round the cap's edge.
        p.glow(
            &Shape::circle(c, g.hub_r + 1.0),
            tint,
            Glow::new(HUB_GLOW.0, HUB_GLOW.1),
        );
        if !self.sprites.draw(p, Kind::Hub, c, w) {
            p.fill(
                &Shape::circle(c, g.hub_r),
                Gradient::radial(c + vec2(-6.0, -7.0), g.hub_r * 1.4)
                    .stop(0.0, pal.roles.ink.with_alpha(0.9))
                    .stop(0.6, pal.roles.ink.with_alpha(0.35))
                    .stop(1.0, pal.roles.ink.with_alpha(0.12)),
            );
        }
    }

    /// The side tubes' crystals, drawn before the rev counter so its bezel covers their inner
    /// edges where the tubes tuck behind it.
    fn draw_side_glass(&self, p: &mut Painter<'_>, g: &Geom) {
        let (sr, sb, sbase) = g.side_crystal;
        let mut crystals = [Piece::circle(g.tubes[1], g.side_open).section(sr, sb, sbase); 2];
        let mut n = 0;
        for slot in [1usize, 2] {
            if self.opts.tubes[slot] != Content::Off {
                crystals[n] = Piece::circle(g.tubes[slot], g.side_open).section(sr, sb, sbase);
                n += 1;
            }
        }
        if n == 0 {
            return;
        }
        let glass = self.glass;
        p.layer(LayerOptions::glass(0.0), |p| {
            for piece in &crystals[..n] {
                p.fill(
                    &Shape::Blob {
                        pieces: core::slice::from_ref(piece),
                        smooth: 0.0,
                    },
                    glass,
                );
            }
        });
    }

    /// The rev counter's crystal with the sapphire magnifier set into it over the gear window:
    /// one glass layer, the magnifier drawn last so it takes the crystal's place in its
    /// footprint (a lens set into the cover glass, never glass stacked on glass), lit more
    /// softly than the crystal so its glint never outshines the gear.
    fn draw_tach_glass(&self, p: &mut Painter<'_>, g: &Geom) {
        let glass = self.glass;
        let (tr, tb, tbase) = g.tach_crystal;
        let crystal = [Piece::circle(g.tach, g.tach_open).section(tr, tb, tbase)];
        let mut lens = glass;
        lens.light = lens.light.map(|l| l * CYCLOPS_LIGHT);
        lens.env_gain = lens.env_gain.map(|e| e * CYCLOPS_LIGHT);
        let window = g.window_centre(&self.shown_scale());
        let cyclops = [Piece::circle(window, g.porthole).dome(g.cyclops.0, g.cyclops.1)];
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &crystal,
                    smooth: 0.0,
                },
                glass,
            );
            p.fill(
                &Shape::Blob {
                    pieces: &cyclops,
                    smooth: 0.0,
                },
                lens,
            );
        });
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(PorscheAnalog::new(setup))
}

#[cfg(test)]
mod tests;
