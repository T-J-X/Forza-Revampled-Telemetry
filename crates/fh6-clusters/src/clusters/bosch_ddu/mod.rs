//! Bosch DDU (`ClusterTheme::BoschDdu`), catalog row 12: the Race Dash family's pattern-setter
//! (cockpit, build #4).
//!
//! An honest race display in a machined carbon housing: an open-bottom U of 2×2 carbon twill
//! (14 px face) with a deep 28 px 45° chamfer into a true-black cavity, four Allen bolts, the
//! chamfer's lit and shadowed flanks following the virtual light (lateral G swings it 25°), and a
//! bonded AR cover glass (a faint crescent and a 1 px tinted rim).
//!
//! The housing is the Blender render (`assets/clusters/bosch_ddu/housing_l160.png` and
//! `housing_l110.png`, recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`): the U modelled in 3D,
//! clear-coated carbon built on the ComfyUI carbon-fibre macro laid into a 2×2 twill
//! (`carbon_weave.png`), machined Allen screws, one render per key-light azimuth crossfaded by
//! lateral G. Without the files it draws the procedural housing (twill, facet-lit chamfer, rims,
//! bolts). The rev cells, the G bar and the lamps are drawn here from the shared components'
//! state (`TachBar`, `SignedBar`, `AlarmColumn`) without bloom or glow.
//!
//! Glance ranks:
//! 1. Ten 84 × 36 microprism LED pods in a recessed light bar, filling outside-in onto the centre
//!    pair; and the gear (`race` Saira Condensed 600, cap 240) under them.
//! 2. Tabular speed (cap 112, ghost 8s) on the left, its unit under it.
//! 3. The 45-segment rev bar, numbered in thousands from the car's limiter.
//! 4. The signed longitudinal-G bar above the gear (accelerating in ink, braking in `warning`);
//!    the derived load quad under the gear; boost and power tiles on the right.
//! 5. Six alarm lamps, three under each column, lit without bloom (the shared lamp keeps its soft
//!    distance glow), labels shy until lit.
//!
//! Signature moments:
//! - **Shift lights:** the pods light in the frame each threshold is crossed, green → amber → red,
//!   outer pair first, landing on the centre pair above the gear; at the shift point all ten take
//!   `shift`, and at the limiter only the pod strip strobes (10 Hz, 50% duty, `ShiftLights`).
//! - **Pod spill:** the one steady limiter cue. The upper carbon chamfer takes a non-emissive
//!   `shift` reflection (one hot spot over each pod, 35% peak) that ramps in over 150 ms, as if the
//!   pods spill onto the bevel. It never flashes. The lit rev segments turn steady ink, so only
//!   the strip and LIM carry `shift`.
//! - **Shift event page:** on every upshift the right tiles swap to the rpm the shift was made at
//!   and how far it was early or late against the shift point, for 4 s (a Bosch event page). The
//!   swap is stepped over 150 ms and never shows two pages at once. The gear itself changes with
//!   the tokens' entry motion only.
//! - **Parallax:** lateral G moves the housing ±4 px and the cover glass ±2 px; the data stays.
//!
//! Telemetry: rpm (current, limiter, the shift point), gear, speed, longitudinal and lateral G,
//! pedals, handbrake, clutch, body slip (from velocity), drivetrain, power and boost are decoded
//! and golden-tested. Derived (labelled so in `HANDOVER.md`): the TC, ABS, SLIP and LNCH lamps and
//! the load quad (weight transfer from G). Tyre temperatures, slip ratios, fuel and lap data are
//! not in the decoder yet, so nothing here pretends to show them.
//!
//! Zone table (local 4K px around the cockpit anchor (1920, 1835); the envelope is
//! x −720..720, y −320..240):
//!
//! | Zone | Element | Geometry |
//! |---|---|---|
//! | Z5 | housing (open-bottom U) | x ±700, y −306 → open at 234; top corners r 96; face 14, chamfer 28 |
//! | Z5 | cavity | x ±658, top −264, corners r 54 |
//! | Z1 | light bar + 10 pods | bar x ±556, y −252..−204; pods 84 × 36 at x −495 + 110·i, y −228 |
//! | Z3 | rev bar | x ±500, y −196..−172, 45 segments, numerals cap 16 under it |
//! | Z4 | long-G bar | centre (0, −120), ±260, 8% of the gear cap tall |
//! | Z1 | gear | centre (0, 46), cap 240 |
//! | Z4 | load quad | 2 × 2 tyres 28 × 24 at x ±17, y 190 / 220 |
//! | Z2 | speed | centre (−400, 10), cap 112; unit cap-top at y 84 |
//! | Z4 | tiles | x 260..560: y −80..35 and 45..160, value cap 56 |
//! | Z4 | dividers | x ±210, y −80..160, fading at both ends |
//! | Z5 | lamps | rows x −620..−260 and 260..620 at y 180 |
//! | Z5 | bolts | (±667, −273) and (±693, 96), r 5.5 |
//!
//! Hood: the catalog lists this cluster for the cockpit only. Forced into the hood view it draws
//! the same layout at [`HOOD_SCALE`], which fits the hood envelope; the housing is opaque, so it
//! stays legible over a bright bonnet.
use crate::common::bezel::{chamfer_level, lit_rim, on_screen};
use crate::common::hero_gear::HeroGear;
use crate::common::race::{AlarmColumn, AlarmLamp, AlarmSide, DataTile, LampTone, TileAlert, TileFormat, TileFrame};
use crate::common::shift::LimiterCue;
use crate::common::signed_bar::{AccelTone, SignedBar, SignedSource};
use crate::common::speed::TabularSpeed;
use crate::common::tach::TachBar;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::{FRAC_1_SQRT_2, PI, TAU};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Bloom, Cluster, Color, FontId, FontRole, FrameInput, Gear, Glow, Gradient, ImageId,
    ImagePaint, LayerOptions, Layout, Paint, Painter, Palette, Radii, Rect, Shape, Signals, Smoothed, SpeedUnit,
    Spread, Stroke, TextStyle, Tokens, VAlign, Vec2, View,
};

/// Pods in the light bar (catalog brief: ten).
const PODS: u32 = 10;
/// Scale of the whole cluster when it is forced into the hood view.
const HOOD_SCALE: f32 = 0.7;
/// The pod-spill cue ramps to full over this long (brief: 150 ms).
const SPILL_RAMP_S: f32 = 0.15;
/// The pod-spill reflection's peak opacity. The brief asks 60%; at 60% the reflection read as
/// an emitter in the critique, so each hot spot peaks at 35%.
const SPILL_PEAK: f32 = 0.35;
/// How long the shift event page holds (brief: 4 s).
const EVENT_HOLD_S: f32 = 4.0;
/// The stepped page swap (brief: 150 ms): 3 steps of 25 ms out, then 3 in.
const EVENT_STEP_S: f32 = 0.025;
/// Steps each way.
const EVENT_STEPS: f32 = 3.0;
/// The rpm the shift was made at is the peak over this window before the gear changed (the
/// rpm can drop a frame or two before the gear byte does).
const SHIFT_PEAK_WINDOW_S: f32 = 0.25;
/// A shift within this many rpm of the shift point reads as on time (neutral ink).
const SHIFT_WINDOW_RPM: f32 = 150.0;
/// Housing parallax at full lateral G (px), and the cover glass's.
const PARALLAX_HOUSING: f32 = 4.0;
/// See [`PARALLAX_HOUSING`].
const PARALLAX_GLASS: f32 = 2.0;
/// The cover glass crescent: 18/255 as seen on screen (brief).
const CRESCENT_LEVEL: f32 = 18.0 / 255.0;
/// Brightness of the twill texture before it is multiplied by itself (the squared weave sits
/// near 5% on screen, the tows near 7%: static structure stays well under 20%).
const TWILL_TINT: f32 = 1.0;
/// The Blender housing renders (`assets/clusters/bosch_ddu/housing_l160.png`, `housing_l110.png`,
/// recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`): the local rect they cover (the housing
/// with a 6 px margin, down to the open line) and their key-light azimuths (degrees).
const BAKED_RECT: Rect = Rect {
    min: vec2(-706.0, -312.0),
    max: vec2(706.0, 234.0),
};
/// See [`BAKED_RECT`].
const BAKED_LIGHTS: [f32; 2] = [-160.0, -110.0];
/// The renders' brightness on screen as a linear share (the studio render is lit for a
/// monitor; in the cockpit the static structure stays under 20% on screen).
const BAKED_GAIN: f32 = 0.55;
/// Twill tile size for the Phase 6 texture (local px).
const TWILL_TILE: f32 = 208.0;
/// Procedural twill: weave pitch and tow contrast (linear coverage of ink over black).
const WEAVE: f32 = 12.0;
/// See [`WEAVE`].
const WEAVE_ALPHA: f32 = 0.02;
/// The twill line's brightness on screen (the diagonal steps of the weave).
const TWILL_LINE: f32 = 0.07;
/// Diagonal sheen across the carbon face (linear coverage of ink).
const SHEEN: f32 = 0.03;
/// A chamfer facet's brightness across its width, as a share of its facing level (lip, a third
/// of the way down, base). The lip catches the light; the base sinks into the cavity.
const CHAMFER_PROFILE: [f32; 3] = [1.0, 0.55, 0.2];
/// Share of the COMPONENTS.md facing levels (lit 55%, shadow 18%) the chamfer shows: the full
/// levels on a 28 px band read as a grey plastic frame on OLED black.
const CHAMFER_GAIN: f32 = 0.9;
/// Unlit lenses, cells and tracks (pods, rev cells, G-bar track, lamps, tyres) as seen on screen:
/// 9/255. At the ghost role they read as grey keys and a scroll bar on true black.
const UNLIT_LENS: f32 = 9.0 / 255.0;
/// The lit pods' share of the tokens' bloom intensity (rank 1, but an LED behind a lens, not a
/// glowing button).
const POD_BLOOM: f32 = 0.4;
/// Pitch of the microprism rings in a pod lens (local px).
const POD_RING_PITCH: f32 = 5.0;
/// Nominal rev segments: `primary` pulled this far toward the label grey (the bar is rank 3; at
/// full saturation 26 blue cells outshout the gear at cruise).
const REV_DESAT: f32 = 0.35;
/// Unlit rank-5 labels (lamps) as seen on screen, under the unit label.
const LAMP_LABEL: f32 = 0.35;
/// A load-quad tyre's body on screen: a little above the unlit lenses so the empty part of the
/// tyre still shows its outline without a stroke.
const TYRE_BODY: f32 = 14.0 / 255.0;
/// The speed's ghost 8s as seen on screen (3%): brighter, an unused leading 8 reads as a digit
/// (74 mph read as 874).
const GHOST_8: f32 = 0.02;

/// Derived lamp thresholds (proposed; labelled derived in `HANDOVER.md`).
mod lamp {
    /// SLIP: body slip angle (deg) above this at speed.
    pub const SLIP_DEG: f32 = 10.0;
    /// SLIP / TC / ABS: minimum speed (m/s) before any slip or lock-up reading counts.
    pub const MIN_SPEED_MS: f32 = 4.0;
    /// TC: throttle above this (race-dash.md: 20%).
    pub const TC_THROTTLE: f32 = 0.2;
    /// TC: body slip above this (deg) and growing faster than [`TC_SLIP_RATE`].
    pub const TC_SLIP_DEG: f32 = 3.0;
    /// TC: the slip onset rate (deg/s) that reads as power oversteer.
    pub const TC_SLIP_RATE: f32 = 8.0;
    /// ABS: brake above this (race-dash.md: 30% in the brief).
    pub const ABS_BRAKE: f32 = 0.3;
    /// ABS: deceleration (g) above this reads as threshold braking.
    pub const ABS_DECEL_G: f32 = 0.85;
    /// LIM: rpm at or above this share of the limiter.
    pub const LIM_PCT: f32 = 0.985;
    /// HB: handbrake above this.
    pub const HANDBRAKE: f32 = 0.1;
    /// LNCH: standing still (m/s), throttle above this with the brake, handbrake or clutch held.
    pub const LAUNCH_SPEED_MS: f32 = 0.5;
    /// See [`LAUNCH_SPEED_MS`].
    pub const LAUNCH_THROTTLE: f32 = 0.5;
    /// Forza `DrivetrainType` 0 is front-wheel drive: no power oversteer to catch.
    pub const FWD: i32 = 0;
}

/// Weight transfer per g as a share of the axle (centre of mass height / wheelbase, typical of a
/// road car) and per g as a share of the side (height / track width).
const LOAD_LONG_PER_G: f32 = 0.18;
/// See [`LOAD_LONG_PER_G`].
const LOAD_LAT_PER_G: f32 = 0.30;

/// Geometry (local 4K px around the anchor). See the zone table in the module docs.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    /// The housing's outer bounds; `max.y` is the open bottom line.
    housing: Rect,
    top_radius: f32,
    face: f32,
    chamfer: f32,
    light_bar: Rect,
    pod_first_x: f32,
    pod_pitch: f32,
    pod_y: f32,
    pod_size: Vec2,
    bar: Rect,
    scale_cap: f32,
    g_bar: Vec2,
    g_bar_half: f32,
    gear: Vec2,
    gear_cap: f32,
    quad: Vec2,
    tyre: Vec2,
    quad_step: Vec2,
    speed: Vec2,
    speed_cap: f32,
    unit_y: f32,
    tiles: [Rect; 2],
    tile_cap: f32,
    divider_x: f32,
    divider_y: [f32; 2],
    lamps: [Rect; 2],
    bolts: [Vec2; 4],
    bolt_r: f32,
    /// The U's legs fade into the column over this length above the open line.
    leg_fade: f32,
}

const COCKPIT: Geom = Geom {
    housing: Rect {
        min: vec2(-700.0, -306.0),
        max: vec2(700.0, 234.0),
    },
    top_radius: 96.0,
    face: 14.0,
    chamfer: 28.0,
    light_bar: Rect {
        min: vec2(-556.0, -252.0),
        max: vec2(556.0, -204.0),
    },
    pod_first_x: -495.0,
    pod_pitch: 110.0,
    pod_y: -228.0,
    pod_size: vec2(84.0, 36.0),
    bar: Rect {
        min: vec2(-500.0, -196.0),
        max: vec2(500.0, -172.0),
    },
    scale_cap: 16.0,
    g_bar: vec2(0.0, -120.0),
    g_bar_half: 260.0,
    gear: vec2(0.0, 46.0),
    gear_cap: 240.0,
    quad: vec2(0.0, 205.0),
    tyre: vec2(28.0, 24.0),
    quad_step: vec2(34.0, 30.0),
    speed: vec2(-400.0, 10.0),
    speed_cap: 112.0,
    unit_y: 84.0,
    tiles: [
        Rect {
            min: vec2(260.0, -80.0),
            max: vec2(560.0, 35.0),
        },
        Rect {
            min: vec2(260.0, 45.0),
            max: vec2(560.0, 160.0),
        },
    ],
    tile_cap: 56.0,
    divider_x: 210.0,
    divider_y: [-80.0, 160.0],
    lamps: [
        Rect {
            min: vec2(-620.0, 180.0),
            max: vec2(-260.0, 230.0),
        },
        Rect {
            min: vec2(260.0, 180.0),
            max: vec2(620.0, 230.0),
        },
    ],
    bolts: [
        vec2(-667.0, -273.0),
        vec2(667.0, -273.0),
        vec2(-693.0, 96.0),
        vec2(693.0, 96.0),
    ],
    bolt_r: 5.5,
    leg_fade: 96.0,
};

impl Geom {
    /// The housing extended below the open line (its bottom corners are clipped away).
    fn outer(&self) -> Rect {
        Rect::from_points(self.housing.min, vec2(self.housing.max.x, self.housing.max.y + 120.0))
    }

    /// The outer bounds inset by `d` on the top, left and right (the open bottom stays).
    fn inset_u(&self, d: f32) -> Rect {
        let o = self.outer();
        Rect::from_points(o.min + vec2(d, d), vec2(o.max.x - d, o.max.y))
    }

    /// The edge between the face and the chamfer (the lip).
    fn lip(&self) -> Rect {
        self.inset_u(self.face)
    }

    /// The cavity (the chamfer's base).
    fn cavity(&self) -> Rect {
        self.inset_u(self.face + self.chamfer)
    }

    fn lip_radius(&self) -> f32 {
        self.top_radius - self.face
    }

    fn cavity_radius(&self) -> f32 {
        self.top_radius - self.face - self.chamfer
    }

    /// The centres of the two top corners (left, right).
    fn corner_centers(&self) -> (Vec2, Vec2) {
        let h = self.housing;
        let r = self.top_radius;
        (vec2(h.min.x + r, h.min.y + r), vec2(h.max.x - r, h.min.y + r))
    }

    /// The hard clip region of each band piece (same order as [`Geom::band`]); they tile the
    /// housing and meet on the corner centres' lines, which sit on whole pixels at the anchor.
    fn band_clips(&self) -> [Rect; 5] {
        let h = self.housing;
        let (cl, cr) = self.corner_centers();
        let top = h.min.y - 4.0;
        [
            Rect::from_points(vec2(h.min.x - 4.0, top), cl),
            Rect::from_points(vec2(cr.x, top), vec2(h.max.x + 4.0, cr.y)),
            Rect::from_points(vec2(cl.x, top), vec2(cr.x, cl.y)),
            Rect::from_points(vec2(h.min.x - 4.0, cl.y), vec2(cl.x, h.max.y)),
            Rect::from_points(cr, vec2(h.max.x + 4.0, h.max.y)),
        ]
    }

    /// The band of the U from `d0` to `d1` px in from the outer edge, as five small shapes (no
    /// fill covers the whole housing): the left and right corner arcs, the top run and the left
    /// and right legs down to the open line. The straight runs reach `overlap` px into the corners:
    /// opaque passes use 1 px so the anti-aliased joints don't let the game show through (two
    /// half-covered edges add up to 75%); additive passes use 0 so the joint is not lit twice.
    fn band(&self, d0: f32, d1: f32, overlap: f32) -> [Shape<'static>; 5] {
        let h = self.housing;
        let r = self.top_radius;
        let (cl, cr) = self.corner_centers();
        let mid = r - (d0 + d1) * 0.5;
        let w = d1 - d0;
        let q = PI * 0.5;
        let o = overlap;
        // The arcs overrun their quarter by `overlap` px at each end too.
        let e = o / (r - d1).max(1.0);
        [
            Shape::arc(cl, mid, w, PI - e, q + 2.0 * e, fh6_kit::Cap::Butt),
            Shape::arc(cr, mid, w, PI * 1.5 - e, q + 2.0 * e, fh6_kit::Cap::Butt),
            Shape::rect(Rect::from_points(
                vec2(cl.x - o, h.min.y + d0),
                vec2(cr.x + o, h.min.y + d1),
            )),
            Shape::rect(Rect::from_points(
                vec2(h.min.x + d0, cl.y - o),
                vec2(h.min.x + d1, h.max.y),
            )),
            Shape::rect(Rect::from_points(
                vec2(h.max.x - d1, cr.y - o),
                vec2(h.max.x - d0, h.max.y),
            )),
        ]
    }

    /// Pod centres, outermost left first.
    fn pod_centers(&self) -> [Vec2; PODS as usize] {
        let mut out = [Vec2::ZERO; PODS as usize];
        for (i, c) in out.iter_mut().enumerate() {
            *c = vec2(self.pod_first_x + self.pod_pitch * i as f32, self.pod_y);
        }
        out
    }
}

/// Fills `shape` inside the hard (not anti-aliased) clip `clip`: band pieces meet on clip edges, so
/// their joints are neither doubled nor left half-covered.
fn clipped(p: &mut Painter<'_>, clip: Rect, shape: &Shape<'_>, paint: impl Into<Paint>) {
    p.scoped(|p| {
        p.clip(clip, 0.0);
        p.fill(shape, paint);
    });
}

/// U-shaped round rect (top corners only).
fn u_shape(rect: Rect, r: f32) -> Shape<'static> {
    Shape::round_rect(rect, Radii::new(r, r, 0.0, 0.0))
}

/// The faces this cluster sets (resolved once; a user display font replaces the numerals).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    speed: FontId,
    value: FontId,
    label: FontId,
    /// The display role at creation, to notice a user font override.
    role_display: FontId,
}

/// The right column's page.
#[derive(Debug, Clone, Copy, PartialEq)]
struct EventPage {
    /// Seconds since the page opened (negative: closed).
    t: f32,
    /// Peak rpm over the last [`SHIFT_PEAK_WINDOW_S`].
    peak_rpm: f32,
    peak_age: f32,
    last_gear: Gear,
    /// The last shift was at or past the shift point.
    late: bool,
}

impl EventPage {
    fn new() -> Self {
        Self {
            t: -1.0,
            peak_rpm: 0.0,
            peak_age: 0.0,
            last_gear: Gear::Neutral,
            late: false,
        }
    }

    /// The opacities of the normal page and the event page. The swap is stepped and never
    /// overlaps two pages of text: the outgoing page steps out in three 25 ms steps, then the
    /// incoming one steps in (150 ms in all); after the hold the same happens in reverse.
    fn levels(&self) -> (f32, f32) {
        if self.t < 0.0 {
            return (1.0, 0.0);
        }
        let half = EVENT_STEP_S * EVENT_STEPS;
        // 1 → 0 then 0 → 1 over `2 × half`, in steps.
        let swap = |t: f32| -> (f32, f32) {
            if t < half {
                let steps = ((t / EVENT_STEP_S).floor() + 1.0).min(EVENT_STEPS);
                (1.0 - steps / EVENT_STEPS, 0.0)
            } else {
                let steps = (((t - half) / EVENT_STEP_S).floor() + 1.0).min(EVENT_STEPS);
                (0.0, steps / EVENT_STEPS)
            }
        };
        if self.t < EVENT_HOLD_S {
            let (out, inn) = swap(self.t);
            (out, inn)
        } else {
            let (out, inn) = swap(self.t - EVENT_HOLD_S);
            (inn, out)
        }
    }

    fn running(&self) -> bool {
        self.t >= 0.0
    }
}

/// The lamps' derived conditions, evaluated from decoded fields.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct LampInputs {
    slip_rate: f32,
}

/// Left lamps (assists, derived): TC, ABS, SLIP.
const LEFT_LAMPS: [AlarmLamp; 3] = [AlarmLamp::TC, AlarmLamp::ABS, AlarmLamp::new("SLIP", LampTone::Caution)];
/// Right lamps: LIM (the limiter, `shift`), HB (handbrake), LNCH (launch armed, derived).
const RIGHT_LAMPS: [AlarmLamp; 3] = [
    AlarmLamp::LIM,
    AlarmLamp::new("HB", LampTone::Caution),
    AlarmLamp::new("LNCH", LampTone::Caution),
];

/// The left lamp states: TC, ABS, SLIP.
fn left_conditions(sig: &Signals, drivetrain: i32, slip_rate: f32) -> [bool; 3] {
    let moving = sig.speed_ms > lamp::MIN_SPEED_MS;
    let slip = sig.slip_deg.abs();
    let tc = moving
        && drivetrain != lamp::FWD
        && sig.throttle > lamp::TC_THROTTLE
        && slip > lamp::TC_SLIP_DEG
        && slip_rate > lamp::TC_SLIP_RATE;
    let abs = moving && sig.brake > lamp::ABS_BRAKE && -sig.g_long >= lamp::ABS_DECEL_G;
    let slipping = moving && slip >= lamp::SLIP_DEG;
    [tc, abs, slipping]
}

/// The right lamp states: LIM, HB, LNCH.
fn right_conditions(sig: &Signals) -> [bool; 3] {
    let lim = sig.max_rpm > 1.0 && sig.rpm >= sig.max_rpm * lamp::LIM_PCT;
    let hb = sig.handbrake > lamp::HANDBRAKE;
    let held = sig.brake > lamp::ABS_BRAKE || sig.handbrake > lamp::ABS_BRAKE || sig.clutch > 0.5;
    let launch = sig.speed_ms < lamp::LAUNCH_SPEED_MS
        && matches!(sig.gear, Gear::Forward(_))
        && sig.throttle > lamp::LAUNCH_THROTTLE
        && held;
    [lim, hb, launch]
}

/// Per-corner load (1 = static share) from the smoothed G: FL, FR, RL, RR.
fn corner_loads(g_long: f32, g_lat: f32) -> [f32; 4] {
    let gl = if g_long.is_finite() {
        g_long.clamp(-2.5, 2.5)
    } else {
        0.0
    };
    let gt = if g_lat.is_finite() { g_lat.clamp(-2.5, 2.5) } else { 0.0 };
    // Accelerating moves load rearwards; accelerating to the right (a right turn) moves it to
    // the left (outside) tyres.
    let front = (0.5 - LOAD_LONG_PER_G * gl).clamp(0.0, 1.0);
    let left = (0.5 + LOAD_LAT_PER_G * gt).clamp(0.0, 1.0);
    let rear = 1.0 - front;
    let right = 1.0 - left;
    [
        front * left * 4.0,
        front * right * 4.0,
        rear * left * 4.0,
        rear * right * 4.0,
    ]
}

/// The Bosch DDU cluster.
#[derive(Debug, Clone)]
pub struct BoschDdu {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    light: VirtualLight,
    shift: ShiftLights,
    spill: LimiterCue,
    bar: TachBar,
    g_bar: SignedBar,
    gear: HeroGear,
    speed: TabularSpeed,
    boost: DataTile,
    power_hp: DataTile,
    power_kw: DataTile,
    shift_at: DataTile,
    shift_late: DataTile,
    shift_early: DataTile,
    power_w: Smoothed<f32>,
    event: EventPage,
    left: AlarmColumn,
    right: AlarmColumn,
    lamp_in: LampInputs,
    prev_slip: f32,
    twill: Option<ImageId>,
    /// The Blender housing renders (the −160° and −110° key lights), when both load.
    housing: Option<[ImageId; 2]>,
    micro_cap: f32,
    tracking: f32,
}

impl BoschDdu {
    /// Builds the cluster: faces, components and the optional Phase 6 carbon twill.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 600, false).unwrap_or(display),
            speed: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
            value: setup.fonts.find("Saira Condensed", 600, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            role_display: display,
        };
        let mut load = |rel: &str| {
            setup.assets.and_then(|a| {
                let path = a.join(rel);
                path.is_file()
                    .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
                    .flatten()
            })
        };
        // The Blender housing first; the Phase 6 twill only feeds the procedural fallback.
        let housing = match (
            load("clusters/bosch_ddu/housing_l160.png"),
            load("clusters/bosch_ddu/housing_l110.png"),
        ) {
            (Some(a), Some(b)) => Some([a, b]),
            _ => None,
        };
        let twill = if housing.is_some() {
            None
        } else {
            load("classic/carbon_twill_lit.png")
        };
        let open = TileFrame::Open;
        Self {
            faces,
            sig: Signals::default(),
            palette: Palette::family(t, "raceDash"),
            view: View::Cockpit,
            light: VirtualLight::new(t),
            shift: ShiftLights::new(PODS),
            spill: LimiterCue::new(),
            bar: TachBar::new(t),
            g_bar: SignedBar::new(t, SignedSource::LongitudinalG { full_scale_g: 1.5 })
                .with_accel_tone(AccelTone::Ink)
                .with_height_pct(0.08),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            boost: DataTile::new("BOOST PSI", TileFormat::Fixed { decimals: 1 }).with_frame(open),
            power_hp: DataTile::new("POWER HP", TileFormat::Integer { cells: 4 }).with_frame(open),
            power_kw: DataTile::new("POWER KW", TileFormat::Integer { cells: 4 }).with_frame(open),
            shift_at: DataTile::new("SHIFT RPM", TileFormat::Integer { cells: 5 }).with_frame(open),
            shift_late: DataTile::new("LATE BY RPM", TileFormat::Integer { cells: 4 }).with_frame(open),
            shift_early: DataTile::new("EARLY BY RPM", TileFormat::Integer { cells: 4 }).with_frame(open),
            power_w: Smoothed::new(0.0, 0.10),
            event: EventPage::new(),
            left: AlarmColumn::new(t, AlarmSide::Bottom, &LEFT_LAMPS),
            right: AlarmColumn::new(t, AlarmSide::Bottom, &RIGHT_LAMPS),
            lamp_in: LampInputs::default(),
            prev_slip: 0.0,
            twill,
            housing,
            micro_cap: t.type_scale.micro_label,
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The housing's parallax (−1..1) from the light's swing (lateral G on the light spring).
    fn parallax(&self) -> f32 {
        ((self.light.angle() - deg(-135.0)) / deg(25.0)).clamp(-1.0, 1.0)
    }

    fn update_event(&mut self, dt: f32) {
        let sig = self.sig;
        // Peak rpm over the window before the gear changes.
        if sig.rpm >= self.event.peak_rpm || self.event.peak_age > SHIFT_PEAK_WINDOW_S {
            self.event.peak_rpm = sig.rpm;
            self.event.peak_age = 0.0;
        } else {
            self.event.peak_age += dt;
        }
        let upshift = match (self.event.last_gear, sig.gear) {
            (Gear::Forward(a), Gear::Forward(b)) => b > a,
            _ => false,
        };
        self.event.last_gear = sig.gear;
        if upshift {
            let at = self.event.peak_rpm;
            self.shift_at.update(Some(at));
            // Early or late against the shift point, as a magnitude under its own label (a tile
            // cannot sign a positive number). Colour only by exception: outside the window it
            // turns `caution`, never `warning` (red is for alarms).
            let delta = at - sig.shift_rpm;
            self.event.late = delta >= 0.0;
            let alert = if delta.abs() > SHIFT_WINDOW_RPM {
                TileAlert::Caution
            } else {
                TileAlert::Normal
            };
            for tile in [&mut self.shift_late, &mut self.shift_early] {
                tile.update(Some(delta.abs()));
                tile.set_alert(alert);
            }
            self.event.t = 0.0;
            self.event.peak_rpm = sig.rpm;
            self.event.peak_age = 0.0;
        } else if self.event.t >= 0.0 {
            self.event.t += dt;
            if self.event.t > EVENT_HOLD_S + 2.0 * EVENT_STEP_S * EVENT_STEPS {
                self.event.t = -1.0;
            }
        }
    }
}

impl Cluster for BoschDdu {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        self.sig = sig;
        self.palette = *input.palette;
        self.view = input.view;
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.value = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }

        self.light.update(sig.g_lat, dt);
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.spill.update(self.shift.over_shift(), SPILL_RAMP_S, dt);
        self.bar.update(&sig, dt);
        self.g_bar.update(&sig, dt);
        self.gear.update(sig.gear, 0.0, dt);
        self.speed.update(sig.speed);

        let boost = input.telemetry.boost_psi;
        self.boost.update(Some(if boost.is_finite() { boost } else { 0.0 }));
        let watts = input.telemetry.power_watts;
        let w = self
            .power_w
            .update(if watts.is_finite() { watts.max(0.0) } else { 0.0 }, dt);
        self.power_hp.update(Some(w / 745.7));
        self.power_kw.update(Some(w / 1000.0));
        self.update_event(dt);

        let slip = sig.slip_deg.abs();
        self.lamp_in.slip_rate = if dt > 1e-4 { (slip - self.prev_slip) / dt } else { 0.0 };
        self.prev_slip = slip;
        let drivetrain = input.telemetry.drivetrain_type;
        self.left
            .update(&left_conditions(&sig, drivetrain, self.lamp_in.slip_rate), dt);
        self.right.update(&right_conditions(&sig), dt);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &COCKPIT;
        let pal = self.palette;
        p.scoped(|p| {
            if self.view == View::Hood {
                p.scale(HOOD_SCALE);
            }
            let par = self.parallax();
            let light = self.light.angle();
            p.scoped(|p| {
                // Whole pixels: the band pieces meet on hard clip edges that must sit on pixel
                // boundaries (a 1 px step at 4K is below what the eye tracks).
                p.translate(vec2((-par * PARALLAX_HOUSING).round(), 0.0));
                self.draw_housing(p, g, &pal, light);
            });
            self.draw_light_bar(p, g, &pal);
            self.draw_data(p, g, &pal);
            p.scoped(|p| {
                p.translate(vec2(-par * PARALLAX_GLASS, 0.0));
                Self::draw_cover_glass(p, g, &pal, light);
            });
        });
    }

    fn animating(&self) -> bool {
        self.shift.animating()
            || self.spill.animating()
            || self.bar.animating()
            || self.g_bar.animating()
            || self.gear.animating()
            || self.event.running()
            || self.left.animating()
            || self.right.animating()
    }
}

impl BoschDdu {
    /// The carbon U, drawn as five small band pieces (two corner arcs, the top run, two legs) so
    /// no fill covers the whole housing: the body, the squared-contrast twill, the diagonal sheen
    /// on the face, the chamfer facets lit by facing and sinking toward the base, the pod spill,
    /// rims, bolts and the leg fade.
    fn draw_housing(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, light: f32) {
        let roles = pal.roles;
        let ink = roles.ink;
        let c = g.housing.center();
        let band = g.face + g.chamfer;
        p.scoped(|p| {
            // The open bottom: everything below the line is cut.
            p.clip(
                Rect::from_points(
                    g.housing.min - vec2(4.0, 4.0),
                    vec2(g.housing.max.x + 4.0, g.housing.max.y),
                ),
                0.0,
            );
            // The true-black cavity (opaque: the game never shows through the screen).
            p.fill(&u_shape(g.cavity(), g.cavity_radius()), roles.oled_black);
            let pieces = g.band(0.0, band, 2.0);
            let clips = g.band_clips();
            for (s, k) in pieces.iter().zip(clips) {
                clipped(p, k, s, roles.oled_black);
            }
            if let Some(baked) = self.housing {
                // The Blender housing (ComfyUI weave, clear coat, chamfer, bolts), one render
                // per key-light azimuth, crossfaded by the light's swing.
                let t = ((light - deg(BAKED_LIGHTS[0])) / deg(BAKED_LIGHTS[1] - BAKED_LIGHTS[0])).clamp(0.0, 1.0);
                for (s, k) in pieces.iter().zip(clips) {
                    clipped(p, k, s, ImagePaint::new(baked[0], BAKED_RECT));
                }
                if t > 1e-3 {
                    for (s, k) in pieces.iter().zip(clips) {
                        clipped(
                            p,
                            k,
                            s,
                            ImagePaint::new(baked[1], BAKED_RECT).tint(Color::WHITE.fade(t)),
                        );
                    }
                }
                // Static structure stays under 20% on screen: a uniform darkening of the render.
                let dim = roles.oled_black.with_alpha(1.0 - BAKED_GAIN);
                for (s, k) in pieces.iter().zip(clips) {
                    clipped(p, k, s, dim);
                }
                p.stroke(
                    &u_shape(g.cavity(), g.cavity_radius()),
                    pal.glass_tint.with_alpha(0.07),
                    Stroke::rim_px(1.0),
                );
            } else {
                // Carbon: the twill texture times itself (squares the weave contrast, so the tows
                // read on a near-black resin), or the procedural 2×2 twill.
                match self.twill {
                    Some(img) => {
                        let tile = Rect::from_center_size(c, Vec2::splat(TWILL_TILE));
                        let paint = ImagePaint::new(img, tile).repeat();
                        for (s, k) in pieces.iter().zip(clips) {
                            clipped(p, k, s, paint.tint(ink.with_alpha(TWILL_TINT)));
                        }
                        p.set_blend(BlendMode::Multiply);
                        for (s, k) in pieces.iter().zip(clips) {
                            clipped(p, k, s, paint);
                        }
                        p.set_blend(BlendMode::Normal);
                    }
                    None => {
                        for (dir, pitch, alpha) in [
                            (vec2(1.0, 1.0), WEAVE, WEAVE_ALPHA),
                            (vec2(1.0, -1.0), WEAVE * 2.0, WEAVE_ALPHA * 0.6),
                        ] {
                            let d = dir.normalize_or_zero() * pitch;
                            let paint = Gradient::linear(c, c + d)
                                .spread(Spread::Repeat)
                                .stop(0.0, ink.with_alpha(0.0))
                                .stop(0.35, ink.with_alpha(alpha))
                                .stop(0.5, ink.with_alpha(alpha))
                                .stop(0.85, ink.with_alpha(0.0))
                                .stop(1.0, ink.with_alpha(0.0));
                            for (s, k) in pieces.iter().zip(clips) {
                                clipped(p, k, s, paint);
                            }
                        }
                    }
                }
                // The twill line: the diagonal steps a 2×2 weave makes, lit by facing (the tows
                // running toward the light catch it), about ±9/255 over the resin.
                let l = Vec2::from_angle(light);
                let catch = 0.6 + 0.4 * l.dot(vec2(FRAC_1_SQRT_2, FRAC_1_SQRT_2)).abs();
                let line = ink.with_alpha(on_screen(TWILL_LINE * catch));
                let d = vec2(1.0, 1.0).normalize_or_zero() * WEAVE * 2.0;
                let twill_line = Gradient::linear(c, c + d)
                    .spread(Spread::Repeat)
                    .stop(0.0, line.with_alpha(0.0))
                    .stop(0.45, line.with_alpha(0.0))
                    .stop(0.55, line)
                    .stop(0.95, line)
                    .stop(1.0, line.with_alpha(0.0));
                p.set_blend(BlendMode::Add);
                for (s, k) in pieces.iter().zip(clips) {
                    clipped(p, k, s, twill_line);
                }
                p.set_blend(BlendMode::Normal);
                // Diagonal sheen on the face from the light's corner (additive).
                let ext = g.outer().size().length() * 0.5;
                let sheen = Gradient::linear(c + l * ext, c - l * ext)
                    .stop(0.0, ink.with_alpha(SHEEN * 0.5))
                    .stop(0.16, ink.with_alpha(SHEEN))
                    .stop(0.42, ink.with_alpha(0.0))
                    .stop(1.0, ink.with_alpha(0.0));
                p.set_blend(BlendMode::Add);
                let face = g.band(0.0, g.face, 2.0);
                for (s, k) in face.iter().zip(clips) {
                    clipped(p, k, s, sheen);
                }
                p.set_blend(BlendMode::Normal);

                self.draw_chamfer(p, g, pal, light);

                // Rims: the outer edge and the lip catch the light; the cavity edge is the cover
                // glass's 1 px AR tint.
                p.stroke(
                    &u_shape(g.outer(), g.top_radius),
                    lit_rim(
                        c,
                        light,
                        roles.rim_specular.with_alpha(0.32),
                        roles.hairline.with_alpha(0.06),
                    ),
                    Stroke::rim_px(1.0),
                );
                p.stroke(
                    &u_shape(g.lip(), g.lip_radius()),
                    lit_rim(
                        c,
                        light,
                        roles.rim_specular.with_alpha(0.25),
                        roles.hairline.with_alpha(0.04),
                    ),
                    Stroke::rim_px(1.0),
                );
                p.stroke(
                    &u_shape(g.cavity(), g.cavity_radius()),
                    pal.glass_tint.with_alpha(0.07),
                    Stroke::rim_px(1.0),
                );

                for b in g.bolts {
                    allen_bolt(p, b, g.bolt_r, pal, light);
                }
            }
            self.draw_spill(p, g, pal);

            // The legs dissolve into the steering column above the open line.
            let open = g.housing.max.y;
            let cav = g.cavity();
            for (x0, x1) in [
                (g.housing.min.x - 4.0, cav.min.x + 1.0),
                (cav.max.x - 1.0, g.housing.max.x + 4.0),
            ] {
                let r = Rect::from_points(vec2(x0, open - g.leg_fade), vec2(x1, open));
                p.fill(
                    &Shape::rect(r),
                    Gradient::linear(r.at(0.5, 0.0), r.at(0.5, 1.0))
                        .stop(0.0, roles.oled_black.with_alpha(0.0))
                        .stop(0.5, roles.oled_black.with_alpha(0.75))
                        .stop(1.0, roles.oled_black),
                );
            }
        });
    }

    /// The chamfer facets: each takes the light by its facing (additive, uniform along the
    /// straight runs, turning with a conic round the corners), then sinks toward the cavity
    /// under a black overlay across its width. The pod spill sits on the upper run.
    fn draw_chamfer(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, light: f32) {
        let roles = pal.roles;
        let ink = roles.ink;
        let black = roles.oled_black;
        let [corner_l, corner_r, top, leg_l, leg_r] = g.band(g.face, g.face + g.chamfer, 2.0);
        let [k_cl, k_cr, k_top, k_ll, k_lr] = g.band_clips();
        let level = |normal: Vec2| ink.with_alpha(on_screen(chamfer_level(normal, light) * CHAMFER_GAIN));
        let [k0, k1, k2] = CHAMFER_PROFILE;
        let shade = |k: f32| black.with_alpha((1.0 - on_screen(k)).clamp(0.0, 1.0));
        let (cl, cr) = g.corner_centers();
        let r_lip = g.lip_radius();
        let r_base = g.cavity_radius();
        let base_t = r_base / r_lip.max(1.0);
        let mid_t = 1.0 - (1.0 - base_t) * 0.35;
        let n = |a: f32| -Vec2::from_angle(a);
        let q = PI * 0.5;

        p.set_blend(BlendMode::Add);
        clipped(p, k_top, &top, level(vec2(0.0, 1.0)));
        clipped(p, k_ll, &leg_l, level(vec2(1.0, 0.0)));
        clipped(p, k_lr, &leg_r, level(vec2(-1.0, 0.0)));
        for (shape, center, start, k) in [(corner_l, cl, PI, k_cl), (corner_r, cr, PI * 1.5, k_cr)] {
            clipped(
                p,
                k,
                &shape,
                Gradient::conic(center, start)
                    .stop(0.0, level(n(start)))
                    .stop(0.125, level(n(start + q * 0.5)))
                    .stop(0.25, level(n(start + q)))
                    .stop(1.0, level(n(start + q))),
            );
        }
        p.set_blend(BlendMode::Normal);

        // Across the width: the lip keeps its light, the base sinks into the cavity.
        let across = |from: Vec2, to: Vec2| {
            Gradient::linear(from, to)
                .stop(0.0, shade(k0))
                .stop(0.35, shade(k1))
                .stop(1.0, shade(k2))
        };
        let lip = g.lip();
        let cav = g.cavity();
        clipped(p, k_top, &top, across(vec2(0.0, lip.min.y), vec2(0.0, cav.min.y)));
        clipped(p, k_ll, &leg_l, across(vec2(lip.min.x, 0.0), vec2(cav.min.x, 0.0)));
        clipped(p, k_lr, &leg_r, across(vec2(lip.max.x, 0.0), vec2(cav.max.x, 0.0)));
        for (shape, center, k) in [(corner_l, cl, k_cl), (corner_r, cr, k_cr)] {
            clipped(
                p,
                k,
                &shape,
                Gradient::radial(center, r_lip)
                    .stop(base_t, shade(k2))
                    .stop(mid_t, shade(k1))
                    .stop(1.0, shade(k0)),
            );
        }
    }

    /// The pod spill: a non-emissive reflection of each pod on the upper chamfer, one hot spot
    /// over every pod (falling off over 44 px), strongest at the chamfer's base.
    fn draw_spill(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let lip = g.lip();
        let cav = g.cavity();
        let spill = self.spill.level();
        if spill > 1e-3 {
            let col = pal.shift.with_alpha(SPILL_PEAK * spill);
            for c in g.pod_centers() {
                let r = Rect::from_points(vec2(c.x - 48.0, lip.min.y), vec2(c.x + 48.0, cav.min.y));
                let at = vec2(c.x, cav.min.y);
                p.fill(
                    &Shape::rect(r),
                    Gradient::radial(at, 44.0)
                        .stop(0.0, col)
                        .stop(0.45, col.fade(0.35))
                        .stop(1.0, col.with_alpha(0.0)),
                );
            }
        }
    }

    /// The recessed light bar and its ten microprism pods.
    fn draw_light_bar(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let bar = g.light_bar;
        let radius = bar.height() * 0.3;
        // A recessed channel, no outline: its floor rises from black under the top lip to a
        // faint lit bottom lip (6% on screen).
        p.fill(
            &Shape::round_rect(bar, radius),
            Gradient::linear(bar.at(0.5, 0.0), bar.at(0.5, 1.0))
                .stop(0.0, roles.oled_black)
                .stop(0.7, roles.oled_black)
                .stop(1.0, roles.ink.with_alpha(on_screen(0.06))),
        );
        // The ten LEDs behind microprism lenses, from `ShiftLights`' state (outside-in fill, stage
        // colours, the 10 Hz limiter strobe). No outlines: a lens is a two-stop radial (a core 40%
        // toward white), fine concentric prism rings and one top glint; lit lenses take a tight
        // 6 px glow and a reduced bloom, so they read as LEDs behind glass, not glowing buttons.
        let bloom = LayerOptions {
            bloom: Some(Bloom {
                threshold: p.tokens().glow.bloom_threshold,
                intensity: p.tokens().glow.bloom_intensity * POD_BLOOM,
            }),
            ..LayerOptions::default()
        };
        for (i, c) in g.pod_centers().into_iter().enumerate() {
            let lens = Rect::from_center_size(c, g.pod_size);
            let shape = Shape::round_rect(lens, lens.height() * 0.35);
            let half = lens.width() * 0.5;
            let rings = |a: f32| {
                Gradient::radial(c, POD_RING_PITCH)
                    .spread(Spread::Repeat)
                    .stop(0.0, roles.ink.with_alpha(0.0))
                    .stop(0.7, roles.ink.with_alpha(0.0))
                    .stop(0.85, roles.ink.with_alpha(a))
                    .stop(1.0, roles.ink.with_alpha(0.0))
            };
            let glint = Rect::from_xywh(lens.min.x + 14.0, lens.min.y + 3.0, lens.width() - 28.0, 2.0);
            match self.shift.color_of(i, pal) {
                Some(color) => {
                    p.glow(&shape, color, Glow::new(6.0, 0.25));
                    p.layer(bloom, |p| {
                        p.fill(
                            &shape,
                            Gradient::radial(c, half)
                                .stop(0.0, color.lerp(roles.ink.with_alpha(color.a), 0.4))
                                .stop(0.55, color)
                                .stop(1.0, color.fade(0.8)),
                        );
                    });
                    p.fill(&shape, rings(0.10));
                    p.fill(&Shape::round_rect(glint, 1.0), roles.ink.with_alpha(0.3));
                }
                None => {
                    p.fill(&shape, roles.ink.with_alpha(on_screen(UNLIT_LENS)));
                    p.fill(&shape, rings(on_screen(0.05)));
                    p.fill(&Shape::round_rect(glint, 1.0), roles.ink.with_alpha(on_screen(0.12)));
                }
            }
        }
    }

    /// Everything on the screen: rev bar, G bar, gear, load quad, speed, dividers, tiles, lamps.
    fn draw_data(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        // Rank 3: the rev bar and its numerals from the car's limiter.
        self.draw_rev_bar(p, g, pal);
        self.bar.draw_scale(p, g.bar, self.faces.value, g.scale_cap, pal);
        // Rank 4: the long-G bar above the gear.
        self.draw_g_bar(p, g, pal);
        // Rank 1: the gear, a crisp TFT glyph (no bloom: a halo on OLED black reads as a game HUD).
        self.gear.draw(p, g.gear, self.faces.gear, g.gear_cap, pal);
        self.draw_quad(p, g, pal);

        // Rank 2: speed with ghost 8s, the unit under it.
        self.speed.draw(
            p,
            g.speed,
            self.faces.speed,
            g.speed_cap,
            roles.ink,
            Some(roles.ink.with_alpha(on_screen(GHOST_8))),
        );
        self.speed.draw_unit(
            p,
            vec2(g.speed.x, g.unit_y),
            self.sig.unit.label(),
            self.faces.label,
            self.micro_cap,
            self.tracking,
            pal,
        );

        // Hairline dividers, fading at both ends: data is grouped by alignment, never boxed.
        for x in [-g.divider_x, g.divider_x] {
            let r = Rect::from_points(vec2(x - 0.5, g.divider_y[0]), vec2(x + 0.5, g.divider_y[1]));
            p.fill(
                &Shape::rect(r),
                Gradient::linear(r.at(0.5, 0.0), r.at(0.5, 1.0))
                    .stop(0.0, roles.hairline.with_alpha(0.0))
                    .stop(0.25, roles.hairline)
                    .stop(0.75, roles.hairline)
                    .stop(1.0, roles.hairline.with_alpha(0.0)),
            );
        }

        // Rank 4 tiles, with the stepped swap to the shift event page.
        let (normal, ev) = self.event.levels();
        let power = match self.sig.unit {
            SpeedUnit::Mph => &self.power_hp,
            SpeedUnit::Kmh => &self.power_kw,
        };
        let (value, label) = (self.faces.value, self.faces.label);
        if normal > 0.0 {
            p.scoped(|p| {
                p.set_opacity(normal);
                self.boost.draw(p, g.tiles[0], g.tile_cap, pal, value, label);
                power.draw(p, g.tiles[1], g.tile_cap, pal, value, label);
            });
        }
        if ev > 0.0 {
            p.scoped(|p| {
                p.set_opacity(ev);
                self.shift_at.draw(p, g.tiles[0], g.tile_cap, pal, value, label);
                let delta = if self.event.late {
                    &self.shift_late
                } else {
                    &self.shift_early
                };
                delta.draw(p, g.tiles[1], g.tile_cap, pal, value, label);
            });
        }

        // Rank 5: the alarm lamps (no bloom, no glow), labels shy until lit.
        self.draw_lamps(p, g.lamps[0], &self.left, pal);
        self.draw_lamps(p, g.lamps[1], &self.right, pal);
    }

    /// The rev bar's cells, drawn here from `TachBar`'s honest fill model (lit count, stage
    /// colour per cell, limiter amount) without its bloom: the rank-3 bar is a lit TFT, and a
    /// cyan halo over its numerals read as a game HUD. Unlit cells are fill-only at 9/255; lit
    /// cells are top-lit lenses (a glint fading down, a shade at the bottom). At the limiter the
    /// lit run crossfades to steady ink, so only the strip and LIM carry `shift`.
    fn draw_rev_bar(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let lit = self.bar.lit();
        let k = self.bar.limiter().clamp(0.0, 1.0);
        let first = self.bar.cell_rect(g.bar, 0);
        let radius = first.width().min(first.height()) * 0.14;
        let unlit = roles.ink.with_alpha(on_screen(UNLIT_LENS));
        // `TachBar` lights nominal cells at 72% of `primary`; only those are pulled toward grey,
        // and the caution and warning stages keep their full colour (colour by exception). Flat
        // cells, no gloss.
        let nominal = pal.primary.fade(0.72);
        let quiet = nominal.lerp(roles.ink_label.fade(nominal.a), REV_DESAT);
        for i in 0..self.bar.count() {
            let cell = Shape::round_rect(self.bar.cell_rect(g.bar, i), radius);
            if i < lit {
                let stage = self.bar.segment_color(g.bar, i, pal);
                let base = if stage == nominal { quiet } else { stage };
                p.fill(&cell, base.lerp(roles.ink, k));
            } else {
                p.fill(&cell, unlit);
            }
        }
        // The scale's unit at the bar's end (the last whole thousand sits short of the limiter).
        let ts = p.tokens().type_scale;
        let style = TextStyle::new(self.faces.label, ts.nano)
            .paint(roles.ink_label)
            .tracking(ts.label_tracking)
            .align(Align::Right, VAlign::CapTop);
        p.text(vec2(g.bar.max.x, g.bar.max.y + g.scale_cap * 0.9), "×1000", &style);
    }

    /// The long-G bar, from `SignedBar`'s spring: a 4 px track at 12%, ±1 g ticks, a notch the
    /// bar's height + 4 px, and a flat 8 px fill with square ends (ink accelerating, `warning`
    /// braking) that fades in from a tenth of full scale.
    fn draw_g_bar(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let c = g.g_bar;
        let half = g.g_bar_half;
        let faint = roles.ink.with_alpha(on_screen(0.12));
        p.fill(&Shape::rect(Rect::from_center_size(c, vec2(half * 2.0, 4.0))), faint);
        // Full scale is 1.5 g: the ±1 g ticks sit at two thirds.
        for s in [-1.0, 1.0] {
            let x = c.x + s * half / 1.5;
            p.fill(
                &Shape::rect(Rect::from_center_size(vec2(x, c.y), vec2(2.0, 12.0))),
                faint,
            );
        }
        p.fill(
            &Shape::rect(Rect::from_center_size(c, vec2(2.0, 8.0 + 4.0))),
            roles.ink.with_alpha(on_screen(0.35)),
        );
        let a = self.g_bar.opacity();
        if a > 0.0 {
            let v = self.g_bar.value();
            let len = v.abs() * half;
            let (x0, color) = if v >= 0.0 {
                (c.x, roles.ink)
            } else {
                (c.x - len, roles.warning)
            };
            p.fill(&Shape::rect(Rect::from_xywh(x0, c.y - 4.0, len, 8.0)), color.fade(a));
        }
    }

    /// One row of alarm lamps from an `AlarmColumn`'s state, in the same slots as its bottom-row
    /// layout. Unlit: fill-only lenses at 9/255 with a 1 px top glint. Lit: the lamp colour with a
    /// core 40% toward white and a 1 px inner rim, and no glow (the brief: lit without bloom).
    fn draw_lamps(&self, p: &mut Painter<'_>, rect: Rect, column: &AlarmColumn, pal: &Palette) {
        let roles = pal.roles;
        let ts = p.tokens().type_scale;
        let lamps = column.lamps();
        let slot = rect.width() / 5.0;
        let (w, h) = (slot * 0.72, slot * 0.72 * 30.0 / 88.0);
        let first = rect.center().x - slot * lamps.len() as f32 * 0.5;
        for (i, lamp) in lamps.iter().enumerate() {
            let cx = first + slot * (i as f32 + 0.5);
            let lens = Rect::from_xywh(cx - w * 0.5, rect.min.y, w, h);
            let shape = Shape::round_rect(lens, h * 0.35);
            let on = column.is_lit(i);
            let color = lamp.tone.color(pal);
            if on {
                p.fill(
                    &shape,
                    Gradient::radial(lens.center(), w * 0.5)
                        .stop(0.0, color.lerp(roles.ink, 0.4))
                        .stop(0.55, color)
                        .stop(1.0, color),
                );
                p.stroke(&shape, roles.ink.with_alpha(0.5), Stroke::inner(1.0));
            } else {
                p.fill(&shape, roles.ink.with_alpha(on_screen(UNLIT_LENS)));
                let glint = Rect::from_xywh(lens.min.x + h * 0.35, lens.min.y + 1.0, w - h * 0.7, 1.0);
                p.fill(&Shape::rect(glint), roles.ink.with_alpha(on_screen(0.12)));
            }
            let ink = if on {
                color.text_safe()
            } else {
                roles.ink.with_alpha(on_screen(LAMP_LABEL))
            };
            let style = TextStyle::new(self.faces.label, (h * 0.55).min(ts.micro_label).max(ts.nano))
                .paint(ink)
                .tracking(ts.label_tracking)
                .align(Align::Center, VAlign::CapTop);
            p.text(vec2(cx, lens.max.y + h * 0.45), lamp.label, &style);
        }
    }

    /// The derived load quad: four tyres seen from above, brighter with more load; an unloaded
    /// (lifting) or overloaded corner turns `caution`.
    fn draw_quad(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let loads = corner_loads(self.sig.g_long, self.sig.g_lat);
        let offsets = [vec2(-0.5, -0.5), vec2(0.5, -0.5), vec2(-0.5, 0.5), vec2(0.5, 0.5)];
        for (load, o) in loads.iter().zip(offsets) {
            let c = g.quad + vec2(o.x * g.quad_step.x, o.y * g.quad_step.y);
            let r = Rect::from_center_size(c, g.tyre);
            let radius = g.tyre.x * 0.18;
            // The tyre: an unlit cell (no outline), filled from the bottom in proportion to its
            // load (full at twice the static share).
            p.fill(
                &Shape::round_rect(r, radius),
                roles.ink.with_alpha(on_screen(TYRE_BODY)),
            );
            let share = (load * 0.5).clamp(0.0, 1.0);
            // Nominal load is a quiet 20%; a loaded corner (over 1.3 × static) lifts to 60%; an
            // unloaded or overloaded corner turns clean `caution`, the SLIP lamp's amber.
            let color = if *load < 0.25 || *load > 1.9 {
                roles.caution
            } else if *load > 1.3 {
                roles.ink.with_alpha(on_screen(0.6))
            } else {
                roles.ink.with_alpha(on_screen(0.2))
            };
            // A lifting wheel (under a quarter of its static load) is flagged by the whole cell in
            // `caution`; otherwise the fill rises with the load, and under a quarter of the cell it
            // is left out rather than drawn as a meaningless sliver.
            if *load < 0.25 {
                p.fill(&Shape::round_rect(r, radius), color);
            } else if share >= 0.25 {
                let fill = Rect::from_points(vec2(r.min.x, r.max.y - r.height() * share), r.max);
                p.fill(&Shape::round_rect(fill, radius), color);
            }
        }
        let style = TextStyle::new(self.faces.label, p.tokens().type_scale.nano)
            .paint(roles.ink_label)
            .tracking(self.tracking)
            .align(Align::Right, VAlign::CapMiddle);
        p.text(
            vec2(g.quad.x - g.quad_step.x * 0.5 - g.tyre.x * 0.5 - 14.0, g.quad.y),
            "TYRE LOAD",
            &style,
        );
    }

    /// The bonded AR cover glass: an 18/255 crescent along the top of the cavity on the light's
    /// side (additive, very faint). It stays above the light bar, so it never smudges the lenses.
    fn draw_cover_glass(p: &mut Painter<'_>, g: &Geom, pal: &Palette, light: f32) {
        let cav = g.cavity();
        let ink = pal.roles.ink;
        p.scoped(|p| {
            p.clip(
                Rect::from_points(cav.min, vec2(cav.max.x, g.light_bar.min.y - 2.0)),
                g.cavity_radius(),
            );
            // A very shallow arc of a large circle (it drops 5 px over ±300 px), offset toward
            // the light.
            let lx = Vec2::from_angle(light).x;
            let r = 9000.0;
            let half = 0.067;
            let center = vec2(lx * 260.0, cav.min.y + 5.0 + r);
            let a = ink.with_alpha(on_screen(CRESCENT_LEVEL));
            let start = -PI * 0.5 - half;
            p.fill(
                &Shape::arc(center, r, 6.0, start, 2.0 * half, fh6_kit::Cap::Butt),
                Gradient::conic(center, start)
                    .stop(0.0, a.with_alpha(0.0))
                    .stop(half / TAU, a)
                    .stop(2.0 * half / TAU, a.with_alpha(0.0))
                    .stop(1.0, a.with_alpha(0.0)),
            );
        });
    }
}

/// An Allen bolt: a lit dome head and a black hex socket with a rim lit from the other side.
fn allen_bolt(p: &mut Painter<'_>, at: Vec2, r: f32, pal: &Palette, light: f32) {
    let roles = pal.roles;
    let ink = roles.ink;
    let l = Vec2::from_angle(light);
    let head = Shape::circle(at, r);
    p.fill(
        &head,
        Gradient::linear(at + l * r, at - l * r)
            .stop(0.0, ink.with_alpha(on_screen(0.42)))
            .stop(1.0, ink.with_alpha(on_screen(0.08))),
    );
    p.stroke(&head, roles.oled_black.with_alpha(0.8), Stroke::outer(1.0));
    let mut hex = [Vec2::ZERO; 6];
    for (i, v) in hex.iter_mut().enumerate() {
        *v = Vec2::polar(at, r * 0.5, deg(15.0 + 60.0 * i as f32));
    }
    let socket = Shape::Polygon { points: &hex };
    p.fill(&socket, roles.oled_black);
    p.stroke(
        &socket,
        lit_rim(
            at,
            light + PI,
            roles.rim_specular.with_alpha(0.4),
            roles.hairline.with_alpha(0.0),
        ),
        Stroke::rim_px(1.0),
    );
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(BoschDdu::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_stays_inside_the_envelopes_and_clear() {
        let t = Tokens::default();
        let g = &COCKPIT;
        let env = t.layout.cockpit_envelope;
        assert!(
            env.contains(g.housing.min) && env.contains(g.housing.max),
            "housing in the cockpit envelope"
        );
        // Forced into the hood view, the scaled layout fits the hood envelope.
        let hood = t.layout.hood_envelope;
        assert!(
            hood.contains(g.housing.min * HOOD_SCALE) && hood.contains(g.housing.max * HOOD_SCALE),
            "scaled housing {:?} in {hood:?}",
            g.housing
        );
        let cav = g.cavity();
        // Everything on the screen stays inside the cavity, clear of its walls by >= 20 px.
        for r in [g.light_bar, g.bar, g.tiles[0], g.tiles[1], g.lamps[0], g.lamps[1]] {
            assert!(
                r.min.x >= cav.min.x + 20.0 && r.max.x <= cav.max.x - 20.0,
                "{r:?} inside the cavity"
            );
            assert!(
                r.min.y >= cav.min.y + 10.0 && r.max.y <= g.housing.max.y,
                "{r:?} inside the cavity"
            );
        }
        // The light bar clears the cavity's top corners.
        assert!(
            g.light_bar.max.x <= cav.max.x - g.cavity_radius(),
            "light bar clear of the corner"
        );
        // The strobing strip is <= 42 px tall and the pods sit inside the light bar.
        assert!(g.pod_size.y <= 42.0);
        for c in g.pod_centers() {
            let pod = Rect::from_center_size(c, g.pod_size);
            assert!(pod.min.x > g.light_bar.min.x && pod.max.x < g.light_bar.max.x);
            assert!(pod.min.y > g.light_bar.min.y && pod.max.y < g.light_bar.max.y);
        }
        // Rev numerals clear the G bar; the gear (at its 1.12 entry) clears the G bar.
        let numerals_bottom = g.bar.max.y + g.scale_cap * 0.9 + g.scale_cap;
        let g_half_h = g.gear_cap * 0.08 * 0.5;
        assert!(
            g.g_bar.y - g_half_h - numerals_bottom >= 12.0,
            "numerals clear of the G bar"
        );
        assert!(
            g.gear.y - g.gear_cap * 0.5 * 1.12 >= g.g_bar.y + g_half_h,
            "gear clear of the G bar"
        );
        assert!(
            g.gear.y - g.gear_cap * 0.5 - (g.g_bar.y + g_half_h) >= 36.0,
            "air between the G bar and the gear"
        );
        let quad_top = g.quad.y - g.quad_step.y * 0.5 - g.tyre.y * 0.5;
        assert!(
            quad_top >= g.gear.y + g.gear_cap * 0.5 + 8.0,
            "load quad clear of the gear"
        );
        assert!(
            quad_top + g.quad_step.y + g.tyre.y <= g.housing.max.y,
            "load quad above the open line"
        );
        // The pods' centre pair sits over the gear.
        let c = g.pod_centers();
        assert!((c[4].x + c[5].x).abs() < 1e-3);
        // Bolts sit on the carbon face.
        for b in g.bolts {
            let on_face = b.x.abs() + g.bolt_r <= g.housing.max.x && b.x.abs() - g.bolt_r >= g.lip().max.x - 30.0;
            assert!(on_face, "bolt {b:?} on the face");
        }
    }

    #[test]
    fn event_page_steps_in_holds_and_steps_out() {
        let mut e = EventPage::new();
        assert_eq!(e.levels(), (1.0, 0.0));
        // Never two pages at once, and the whole swap takes 150 ms.
        let mut t = 0.0;
        while t < EVENT_HOLD_S + 0.3 {
            e.t = t;
            let (normal, event) = e.levels();
            assert!(normal == 0.0 || event == 0.0, "pages overlap at {t}");
            t += 0.005;
        }
        e.t = 0.0;
        assert!(
            (e.levels().0 - 2.0 / 3.0).abs() < 1e-6,
            "the normal page steps out first"
        );
        e.t = 0.149;
        assert_eq!(e.levels(), (0.0, 1.0));
        e.t = 1.0;
        assert_eq!(e.levels(), (0.0, 1.0));
        e.t = EVENT_HOLD_S + 0.01;
        assert!(
            (e.levels().1 - 2.0 / 3.0).abs() < 1e-6,
            "the event page steps out first"
        );
        e.t = EVENT_HOLD_S + 0.149;
        assert_eq!(e.levels(), (1.0, 0.0));
    }

    #[test]
    fn lamps_follow_their_sources() {
        let base = Signals {
            speed_ms: 30.0,
            max_rpm: 8000.0,
            rpm: 5000.0,
            gear: Gear::Forward(3),
            ..Signals::default()
        };
        assert_eq!(left_conditions(&base, 1, 0.0), [false; 3]);
        assert_eq!(right_conditions(&base), [false; 3]);
        // Power oversteer onset in a RWD car lights TC, not in a FWD car.
        let pow = Signals {
            throttle: 0.8,
            slip_deg: 5.0,
            ..base
        };
        assert!(left_conditions(&pow, 1, 20.0)[0]);
        assert!(!left_conditions(&pow, lamp::FWD, 20.0)[0]);
        // Threshold braking lights ABS.
        let brake = Signals {
            brake: 0.9,
            g_long: -1.1,
            ..base
        };
        assert!(left_conditions(&brake, 1, 0.0)[1]);
        // A drift lights SLIP.
        let drift = Signals {
            slip_deg: -24.0,
            ..base
        };
        assert!(left_conditions(&drift, 1, 0.0)[2]);
        // LIM at the limiter, HB with the handbrake, LNCH armed at a standstill.
        let lim = Signals { rpm: 7990.0, ..base };
        assert!(right_conditions(&lim)[0]);
        let hb = Signals { handbrake: 1.0, ..base };
        assert!(right_conditions(&hb)[1]);
        let launch = Signals {
            speed_ms: 0.0,
            throttle: 1.0,
            brake: 1.0,
            gear: Gear::Forward(1),
            ..base
        };
        assert!(right_conditions(&launch)[2]);
    }

    #[test]
    fn load_moves_with_g() {
        let still = corner_loads(0.0, 0.0);
        for l in still {
            assert!((l - 1.0).abs() < 1e-6);
        }
        let braking = corner_loads(-1.0, 0.0);
        assert!(braking[0] > 1.0 && braking[2] < 1.0, "braking loads the front");
        let right_turn = corner_loads(0.0, 1.0);
        assert!(
            right_turn[0] > 1.0 && right_turn[1] < 1.0,
            "a right turn loads the left"
        );
        let total: f32 = corner_loads(0.7, -0.9).iter().sum();
        assert!((total - 4.0).abs() < 1e-4, "load is conserved");
        let nan = corner_loads(f32::NAN, f32::INFINITY);
        assert!(nan.iter().all(|l| l.is_finite()));
    }
}
