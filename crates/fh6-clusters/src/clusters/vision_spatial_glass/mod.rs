//! Vision Spatial Glass (`ClusterTheme::VisionSpatialGlass`), catalog row 2: the lead of the
//! Apple Vision category, in the Liquid Glass family, cockpit only.
//!
//! A frameless spatial stack at four depths, floating in a dim visionOS sky: two glass wings toed
//! in towards the driver, a clear front pane with the gear and the speed, a glass attitude puck
//! hovering under it, and a glass blade of 15 shift pods frontmost. Behind them all runs the rev
//! tunnel: 16 rounded rings receding to a vanishing point behind the pane, which fill from the
//! back to the front as the engine winds up, so the limiter is the ring nearest the driver.
//! Research: `RESEARCH.md`. Board: Penpot page `07 · VisionSpatialGlass` (boards 01 to 06).
//!
//! Zone table (local 4K px round the anchor, y down; the envelope is x −720..720, y −320..240):
//!
//! | Zone | Outline | Where | Glass (rise / edge / base) | Holds |
//! |---|---|---|---|---|
//! | Sky | horizon and floor glow | horizon through the vanishing point (0, −45) | – | ambient: tilts with felt G |
//! | Rev tunnel | 16 rounded-rect rings | centred on (0, −45); nearest 700 × 270 half size, radius 90 | – | rank 3: rpm, back to front |
//! | Wings | tapered capsules, toed in 14° | x ±296..±700, half height 95 inside, 120 outside | 18 / 30 / 8 | rank 4–5: the rpm numerals, labels and the logo zone |
//! | Front pane | rounded rect | x −280..280, y −246..150, radius 72 | 28 / 24 / 14 | rank 1: the gear (cap 176), centred on the vanishing point, so the tunnel runs into it; rank 2: the speed (cap 74) |
//! | Puck | disc | (0, 170), r 54 | dome 27 / whole / 8 | rank 4: felt G bead, brake caution arcs, slip halo |
//! | Blade | pill | x −520..520, y −312..−268 | dome 22 / whole / 4 | rank 1: 15 shift pods, outside-in |
//!
//! Draw order: sky, tunnel, black cores under the pane and the puck, lights (gear, speed, puck
//! bead, pods), all the glass in one layer (glass cannot bend glass), then the wing labels and
//! numerals and the rims' glints over the glass, crisp, where no rim bends them.
//!
//! Signature moments:
//! - **Limiter:** the blade strobes at 10 Hz on its own clock (`common::ShiftLights`), lit at the
//!   moment the shift point is reached, and the whole lit tunnel holds a steady shift colour, as
//!   do the wings' outer slab edges.
//! - **Z-hop upshift:** a glint lights the wings' rims, then the pane's 40 ms later, then the
//!   blade's 40 ms after that; the pane recedes 6 px and the new gear enters from 1.12.
//! - **Depth:** felt lateral G shears the stack by 0/6/12/18 px per g (wings, pane, puck,
//!   blade) and the tunnel and sky the other way, so depth is read, not drawn.
//! - **Show:** the panes materialise back to front over the tokens' materialise time.
//!
//! Telemetry: rpm and the car's idle, shift and limiter rpm (tunnel, numerals, pods), gear and
//! speed (pane), felt G (parallax, horizon tilt, puck bead and tilt), brake (puck caution arcs),
//! body slip (puck halo) and a yaw rate derived from lateral G over speed (wing toe ±2°). Pitch,
//! roll and the game's angular velocity are not verified in the golden packets, so they are not
//! read (`RESEARCH.md`).
use crate::clusters::liquid_glass::{self, aim_studio, studio_glass};
use crate::common::hero_gear::HeroGear;
use crate::common::shift::LimiterCue;
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::{FRAC_PI_2, PI};
use fh6_kit::cluster::Setup;
use fh6_kit::logo::{self, LogoStyle};
use fh6_kit::math::Affine;
use fh6_kit::optics::absorption_coefficient;
use fh6_kit::shape::Outline;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Cap, Cluster, Color, FontId, FontRole, FrameInput, Glow, Gradient, ImageId,
    LayerOptions, Layout, Logo, OpticalGlass, Painter, Palette, Piece, Rect, Shape, Signals, Smoothed, Spring,
    SpringSpec, Strobe, Stroke, TextStyle, Tokens, Tween, VAlign, Vec2,
};

/// Rings in the rev tunnel.
const RINGS: usize = 16;
/// Shift pods on the blade (catalog brief).
const PODS: usize = 15;
/// Scale of the farthest ring against the nearest, and the exponent that spaces them: rings
/// crowd towards the vanishing point as in a real perspective.
const RING_FAR: f32 = 0.18;
const RING_POW: f32 = 1.6;
/// Stroke of the farthest and the nearest ring (px).
const RING_STROKE: (f32, f32) = (2.0, 6.0);
/// Light of an unlit ring: white at 4.5% coverage, a touch under the board's 6% so the wing
/// numerals stand clear of the rings.
const RING_IDLE: f32 = 0.045;
/// Share of `RING_IDLE` the nearest unlit ring loses: the unlit tunnel fades towards the driver,
/// so its outermost ring never reads as a frame round the whole stack.
const RING_IDLE_FADE: f32 = 0.7;
/// Brightness of the lit tunnel (rank 3) as a share of its colour, so the gear outshines it.
const TUNNEL_LEVEL: f32 = 0.75;
/// The ring the rpm is reaching (the head of the fill) burns brighter than the rest, leaning this
/// far to white, and this much wider (still under the speed, rank 2), so the tunnel reads as a gauge at a glance and not as decor.
const HEAD_LEVEL: f32 = 0.85;
const HEAD_WHITE: f32 = 0.3;
const HEAD_WIDEN: f32 = 2.0;
/// Light of a wing numeral the rpm has passed, as a share of its ring's colour leaned to white.
const NUMERAL_LIT: f32 = 0.9;
/// Board v3's tubes of light: the halo under each lit ring (light share and extra width), the
/// hot core on the near half, the depth fog's share at the far end.
const RING_HALO_LEVEL: f32 = 0.18;
const RING_HALO_WIDEN: f32 = 14.0;
const RING_CORE_LEVEL: f32 = 0.55;
const FOG_FAR: f32 = 0.55;
/// The gear-change warp: one ring of light from the vanishing point to the driver over
/// `WARP_S`, at `WARP_LEVEL`, widening by `WARP_WIDEN` px as it nears.
const WARP_S: f32 = 0.22;
const WARP_LEVEL: f32 = 0.9;
const WARP_WIDEN: f32 = 6.0;
/// How far the kit's bloom spreads past a lit stroke (measured with `visual-review stretch` on the
/// redline frame: light of 4 of 255 still 150 px out). Rings nearer the envelope than this are drawn
/// without bloom, so no light reaches past the envelope plus 40 px.
const BLOOM_REACH: f32 = 160.0;
/// The gear's wide glow in the tunnel's colour (board v3), as a share of that colour, and its
/// radius as a share of the gear's cap.
const GEAR_GLOW: f32 = 0.1;
const GEAR_GLOW_R: f32 = 0.95;
/// The nearest lit rings lean to this pale sky blue (board: primary at the back, #C7E9FF at the
/// front).
const NEAR_TINT: u32 = 0xC7_E9_FF;
/// Lateral parallax per g of felt force (px): wings, pane, puck, blade; the tunnel and the sky
/// move the other way by `BACK_SHEAR`.
const SHEAR: [f32; 4] = [0.0, 6.0, 12.0, 18.0];
const BACK_SHEAR: f32 = -6.0;
/// Period of the parallax spring (s): ω 12 rad/s, critically damped (catalog brief).
const SHEAR_PERIOD: f32 = 2.0 * PI / 12.0;
/// Felt G at which the parallax, the horizon tilt and the puck reach their limits.
const G_LIMIT: f32 = 1.5;
/// Horizon tilt per g of felt lateral force (degrees).
const HORIZON_TILT: f32 = 4.0;
/// Reach of the puck's bead per g (px) and its limit.
const BEAD_PER_G: f32 = 40.0;
const BEAD_REACH: f32 = 40.0;
/// Wing toe from the derived yaw rate: ±2° at `YAW_FULL` rad/s, as the change it makes to the
/// outer half height (px per degree at the wing's 450 px length).
const YAW_FULL: f32 = 0.6;
const TOE_DEG: f32 = 2.0;
const TOE_PX_PER_DEG: f32 = 450.0 * PI / 180.0 * 0.5;
/// Speed under which the yaw rate is not derived (m/s): lateral G over a crawl is noise.
const YAW_MIN_SPEED: f32 = 3.0;
/// Standard gravity (m/s²).
const G0: f32 = 9.806_65;
/// Z-hop timeline (s): each layer's glint lights `HOP_STEP` after the one behind it and holds
/// for `GLINT_S`; the pane recedes `RECEDE_PX` over `RECEDE_S`.
const HOP_STEP: f32 = 0.04;
const GLINT_S: f32 = 0.02;
const HOP_S: f32 = 2.0 * HOP_STEP + GLINT_S;
const GLINT_LEVEL: f32 = 0.6;
const RECEDE_PX: f32 = 6.0;
const RECEDE_S: f32 = 0.2;
/// Opacity (linear) of the black core under the pane: the lit rings behind it still glow
/// faintly through, so the tunnel's vanishing point and its first rings stay readable (board
/// 02), while the digits keep their contrast.
const CORE_ALPHA: f32 = 0.92;
/// The brightest a part that never changes may show on black (tokens
/// `motion.idle.staticLumaMaxPct` is 20%): labels, units, numerals and the logo.
const STATIC_LUMA: f32 = 0.19;
/// Smallest cap height of a label (px): about 16 arcmin at 1.5 m on the 55-inch panel.
const MIN_CAP: f32 = 22.0;
/// Cap height of the tunnel's rpm numerals on the wings (px).
const NUMERAL_CAP: f32 = 36.0;
/// Width of the rpm backlash (as the Liquid Glass pilot): wider than the engine's bounce on the
/// limiter, so the tunnel holds still there.
const REV_BAND: f32 = 60.0;
/// Strip clock lead and dark-phase floor (as the pilot).
const STROBE_LEAD: f32 = 1e-4;
const STROBE_FLOOR: f32 = 0.2;
/// Brightness of the floor glow, the horizon and the clouds: ambient, far under the static cap.
const SKY_LEVEL: f32 = 0.05;
/// The stratus streaks over the horizon: x, height above the horizon (negative is up), half
/// length and brightness. They are flattened by `CLOUD_FLAT` and wrap `CLOUD_WRAP` px past the
/// envelope's ends, drifting `CLOUD_DRIFT` px per second per m/s of speed.
const CLOUDS: [(f32, f32, f32, f32); 6] = [
    (-520.0, -120.0, 300.0, 1.0),
    (-80.0, -160.0, 360.0, 0.8),
    (420.0, -110.0, 280.0, 1.0),
    (-300.0, -205.0, 260.0, 0.6),
    (250.0, -230.0, 320.0, 0.55),
    (640.0, -175.0, 240.0, 0.7),
];
const CLOUD_FLAT: f32 = 0.09;
const CLOUD_WRAP: f32 = 260.0;
const CLOUD_DRIFT: f32 = 0.4;
/// Speed streaks: light dashes that run along the tunnel's four corner rails towards the driver.
/// `STREAKS` per rail, each `STREAK_LEN` of the rail long; they cover the rail once in
/// `1 / (speed_ms * STREAK_RATE)` seconds (about 1.3 s at 40 m/s), so the tunnel flows with
/// the car's speed and stands still when it does.
const STREAKS: usize = 5;
const STREAK_LEN: f32 = 0.07;
const STREAK_RATE: f32 = 0.019;
const STREAK_LEVEL: f32 = 0.55;
/// Light of the faint rails the streaks run on.
const RAIL_IDLE: f32 = 0.035;
/// Speed under which the streaks stop (m/s).
const STREAK_MIN_SPEED: f32 = 0.5;
/// How much throttle lifts the floor glow, and how far the brake warms it to the warning red.
const FLOOR_THROTTLE: f32 = 0.9;
const FLOOR_BRAKE: f32 = 0.6;
/// Vertical squash of the floor glow: the floor seen at a grazing angle.
const FLOOR_SQUASH: f32 = 0.38;

/// The cockpit geometry: the zone table above, as data.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    vanish: Vec2,
    /// Half size and corner radius of the nearest ring.
    ring_half: Vec2,
    ring_radius: f32,
    /// Each wing (left shown; the right is mirrored): the inner and outer centres of its tapered
    /// capsule, and the half heights there.
    wing_inner: Vec2,
    wing_outer: Vec2,
    wing_inner_r: f32,
    wing_outer_r: f32,
    pane: Rect,
    pane_radius: f32,
    core_inset: f32,
    gear_baseline: f32,
    gear_cap: f32,
    speed_baseline: f32,
    speed_cap: f32,
    puck: Vec2,
    puck_r: f32,
    blade: Rect,
    pod: Vec2,
    pod_first_x: f32,
    pod_pitch: f32,
    /// Label rows on the wings (cap middle) and the logo zone's centre (right wing).
    label_y: f32,
    numeral_dy: f32,
    logo: Vec2,
    envelope: Rect,
}

const COCKPIT: Geom = Geom {
    vanish: vec2(0.0, -45.0),
    ring_half: vec2(700.0, 270.0),
    ring_radius: 90.0,
    wing_inner: vec2(-391.0, 0.0),
    wing_outer: vec2(-580.0, 0.0),
    wing_inner_r: 95.0,
    wing_outer_r: 120.0,
    pane: Rect {
        min: vec2(-280.0, -246.0),
        max: vec2(280.0, 150.0),
    },
    pane_radius: 72.0,
    core_inset: 22.0,
    gear_baseline: 108.0,
    gear_cap: 236.0,
    speed_baseline: -144.0,
    speed_cap: 64.0,
    puck: vec2(0.0, 170.0),
    puck_r: 54.0,
    blade: Rect {
        min: vec2(-520.0, -312.0),
        max: vec2(520.0, -268.0),
    },
    pod: vec2(54.0, 24.0),
    pod_first_x: -485.0,
    pod_pitch: 69.3,
    label_y: 70.0,
    numeral_dy: 36.0,
    logo: vec2(475.0, -78.0),
    envelope: Rect {
        min: vec2(-720.0, -320.0),
        max: vec2(720.0, 240.0),
    },
};

impl Geom {
    /// Scale of ring position `u` (0 the farthest ring, `RINGS − 1` the nearest; fractional in
    /// between) against the nearest ring.
    fn ring_scale(u: f32) -> f32 {
        let k = (u / (RINGS - 1) as f32).clamp(0.0, 1.0);
        RING_FAR + (1.0 - RING_FAR) * k.powf(RING_POW)
    }

    /// Half size and corner radius of ring `u`.
    fn ring(&self, u: f32) -> (Vec2, f32) {
        let s = Self::ring_scale(u);
        (self.ring_half * s, self.ring_radius * s)
    }

    /// Stroke width of ring `i`.
    fn ring_stroke(i: usize) -> f32 {
        Self::ring_stroke_at(i as f32)
    }

    /// Stroke width at ring position `u` (fractional between rings).
    fn ring_stroke_at(u: f32) -> f32 {
        let k = (u / (RINGS - 1) as f32).clamp(0.0, 1.0);
        RING_STROKE.0 + (RING_STROKE.1 - RING_STROKE.0) * k
    }

    /// The corner of the nearest ring the rails aim at: the middle of its corner arc, which every
    /// ring's corner shares the direction of, since they all scale about the vanishing point.
    fn rail_corner(&self) -> Vec2 {
        let inset = self.ring_radius * (1.0 - core::f32::consts::FRAC_1_SQRT_2);
        self.ring_half - Vec2::splat(inset)
    }

    /// The black core under the pane's clear middle.
    fn core(&self) -> Rect {
        self.pane.inset(self.core_inset)
    }

    /// True when ring `i` lies wholly behind the pane's core, so it is not drawn while unlit.
    fn ring_hidden(&self, i: usize) -> bool {
        let (half, _) = self.ring(i as f32);
        let w = Self::ring_stroke(i) * 0.5;
        let outer = Rect::from_center_size(self.vanish, (half + Vec2::splat(w)) * 2.0);
        let core = self.core();
        outer.min.x >= core.min.x && outer.max.x <= core.max.x && outer.min.y >= core.min.y && outer.max.y <= core.max.y
    }

    /// True when ring position `u` can sit in a bloom layer: the bloom's reach round its outer
    /// edge, at the furthest backdrop shear, stays inside the envelope plus 40 px.
    fn ring_blooms(&self, u: f32) -> bool {
        let (half, _) = self.ring(u);
        let reach = Self::ring_stroke_at(u) * 0.5 + BLOOM_REACH;
        let slack = BACK_SHEAR.abs() * G_LIMIT;
        let outer = Rect::from_center_size(self.vanish, (half + Vec2::splat(reach)) * 2.0);
        let room = self.envelope.expand(40.0);
        outer.min.x - slack >= room.min.x
            && outer.max.x + slack <= room.max.x
            && outer.min.y >= room.min.y
            && outer.max.y <= room.max.y
    }

    /// The pod centres on the blade, left to right.
    fn pod_centers(&self) -> [Vec2; PODS] {
        let y = self.blade.center().y;
        core::array::from_fn(|i| vec2(self.pod_first_x + self.pod_pitch * i as f32, y))
    }

    /// The x at which the tunnel's numeral for ring position `u` sits on the left wing: just
    /// inside the ring's left edge.
    fn numeral_x(&self, u: f32) -> f32 {
        let (half, _) = self.ring(u);
        -half.x + 24.0
    }

    /// True when a numeral centred at left-wing `x` sits on the wing and clear of the pane.
    fn numeral_fits(&self, x: f32) -> bool {
        x <= self.pane.min.x - NUMERAL_CAP && x >= self.wing_outer.x - self.wing_outer_r + NUMERAL_CAP
    }
}

/// The faces this cluster sets (a user display font replaces the numerals).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    speed: FontId,
    numeral: FontId,
    label: FontId,
    wide: FontId,
    role_display: FontId,
}

/// The blade's 15 pod lights: `common::ShiftLights` on a clock that starts at the shift point
/// (as the Liquid Glass pilot's strip), so the strobe starts lit.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Strip {
    lights: ShiftLights,
    since: Option<f32>,
    calm: bool,
}

impl Strip {
    fn new() -> Self {
        Self {
            lights: ShiftLights::new(PODS as u32),
            since: None,
            calm: false,
        }
    }

    fn update(&mut self, sig: &Signals, palette: &Palette, dt: f32, reduced_flash: bool) {
        let over = sig.shift_fraction >= 1.0;
        let dt = if dt.is_finite() { dt.max(0.0) } else { 0.0 };
        self.since = match (over, self.since) {
            (false, _) => None,
            (true, None) => Some(0.0),
            (true, Some(t)) => Some((t + dt).rem_euclid(1.0)),
        };
        self.calm = reduced_flash;
        self.lights.update(sig, palette, self.clock(), dt, reduced_flash);
    }

    fn clock(&self) -> f32 {
        self.since.map_or(0.0, |t| t + STROBE_LEAD)
    }

    fn over_shift(&self) -> bool {
        self.lights.over_shift()
    }

    fn animating(&self) -> bool {
        self.lights.animating()
    }

    /// The light under pod `i`; past the shift point the dark phase keeps `STROBE_FLOOR`.
    fn light(&self, i: usize, pal: &Palette) -> Option<Color> {
        if !self.lights.over_shift() {
            return self.lights.color_of(i, pal);
        }
        let level = if self.calm {
            Strobe::breath(self.clock()).max(STROBE_FLOOR)
        } else if self.lights.color_of(i, pal).is_some() {
            1.0
        } else {
            STROBE_FLOOR
        };
        Some(pal.shift.intensity(level))
    }
}

/// `ink` dimmed so it shows at no more than [`STATIC_LUMA`] on black through `transmission`.
/// It scales the light and keeps the coverage (the overlay blends coverage in encoded space).
fn static_ink(ink: Color, transmission: f32) -> Color {
    let seen = ink.luminance() * ink.a.clamp(0.0, 1.0) * transmission.clamp(0.05, 1.0);
    if seen > STATIC_LUMA {
        ink.intensity(STATIC_LUMA / seen)
    } else {
        ink
    }
}

/// `held` moved just far enough that `raw` lies within half of `band` of it.
fn backlash(held: f32, raw: f32, band: f32) -> f32 {
    if raw.is_finite() {
        held.clamp(raw - band * 0.5, raw + band * 0.5)
    } else {
        held
    }
}

/// `g` as a share of [`G_LIMIT`] (−1..=1); not a number reads 0.
fn felt(g: f32) -> f32 {
    if g.is_finite() {
        (g / G_LIMIT).clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Yaw rate (rad/s) derived from lateral G over speed, as a steady turn has it (`a = v ω`). The
/// game's own angular velocity is not verified in the golden packets.
fn derived_yaw(g_lat: f32, speed_ms: f32) -> f32 {
    if !g_lat.is_finite() || !speed_ms.is_finite() || speed_ms < YAW_MIN_SPEED {
        return 0.0;
    }
    g_lat * G0 / speed_ms
}

/// Where `rpm` sits between idle (0) and the limiter (1).
fn rev_fraction(sig: &Signals, rpm: f32) -> f32 {
    let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
    let f = (rpm - sig.idle_rpm) / span;
    if f.is_finite() {
        f.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Loads this cluster's environment (`cabin_env.png`), or the family's studio without it.
fn load_environment(setup: &mut Setup<'_>) -> Option<ImageId> {
    let own = setup
        .assets
        .map(|a| a.join("clusters").join("vision_spatial_glass").join("cabin_env.png"));
    if let Some(path) = own.filter(|p| p.is_file()) {
        match setup.images.load_file(setup.device, setup.queue, &path, None) {
            Ok(image) => return Some(image),
            Err(error) => tracing::warn!(%error, "the Vision Spatial Glass cabin did not load"),
        }
    }
    liquid_glass::load_studio(setup)
}

/// One glass piece's outline as a tapered capsule (the wings).
fn tapered(a: Vec2, b: Vec2, radius_a: f32, radius_b: f32) -> Piece {
    Piece::new(Outline::Capsule {
        a,
        b,
        radius_a,
        radius_b,
    })
}

/// The Vision Spatial Glass cluster.
#[derive(Debug, Clone)]
pub struct VisionSpatialGlass {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    glass: OpticalGlass,
    clear: OpticalGlass,
    light: VirtualLight,
    lean: [Spring; 2],
    lean_target: [f32; 2],
    /// Felt lateral force (share of `G_LIMIT`) on the parallax spring.
    shear: Spring,
    shear_target: f32,
    /// Felt force on the puck: x lateral, y longitudinal (share of `G_LIMIT`).
    bead: [Spring; 2],
    bead_target: [f32; 2],
    /// Wing toe from the derived yaw rate (−1..=1).
    toe: Smoothed<f32>,
    halo: Smoothed<f32>,
    /// Phase of the speed streaks along the rails (0..1).
    flow: f32,
    /// How far the clouds have drifted (px, wrapped).
    cloud_px: f32,
    /// Throttle and brake on the floor glow, smoothed so a stab never pops it.
    floor_throttle: Smoothed<f32>,
    floor_brake: Smoothed<f32>,
    /// Tunnel fill: rpm from idle (0) to the limiter (1).
    rev: Spring,
    rev_target: f32,
    rev_rpm: f32,
    gear: HeroGear,
    speed: TabularSpeed,
    strip: Strip,
    cue: LimiterCue,
    hop: Tween,
    /// The gear-change warp down the tunnel.
    warp: Tween,
    recede: Tween,
    show: Tween,
    fresh: bool,
    logo: Logo,
    environment: Option<ImageId>,
    label_cap: f32,
    tracking: f32,
    calm: bool,
}

impl VisionSpatialGlass {
    /// Builds the cluster: faces, springs and the cabin environment (the family's studio, or the
    /// kit's analytic studio, when it is missing).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let environment = load_environment(setup);
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "liquidGlass");
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
            speed: setup.fonts.find("Saira", 400, false).unwrap_or(display),
            numeral: setup.fonts.find("Saira", 500, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let mut show = Tween::new(t.durations.materialize);
        show.restart();
        let parallax = SpringSpec::critical(SHEAR_PERIOD);
        Self {
            faces,
            sig: Signals::default(),
            palette,
            glass: t.optical_glass(&palette),
            clear: t.optical_glass(&palette),
            light: VirtualLight::new(t),
            lean: [Spring::new(t.springs.light, 0.0), Spring::new(t.springs.light, 0.0)],
            lean_target: [0.0; 2],
            shear: Spring::new(parallax, 0.0),
            shear_target: 0.0,
            bead: [Spring::new(t.springs.g_ball, 0.0), Spring::new(t.springs.g_ball, 0.0)],
            bead_target: [0.0; 2],
            toe: Smoothed::new(0.0, 0.25),
            halo: Smoothed::new(0.0, 0.12),
            flow: 0.0,
            cloud_px: 0.0,
            floor_throttle: Smoothed::new(0.0, 0.15),
            floor_brake: Smoothed::new(0.0, 0.08),
            rev: Spring::new(t.springs.ribbon, 0.0),
            rev_target: 0.0,
            rev_rpm: 0.0,
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            strip: Strip::new(),
            cue: LimiterCue::new(),
            hop: Tween::new(HOP_S),
            warp: Tween::new(WARP_S),
            recede: Tween::new(RECEDE_S),
            show,
            fresh: true,
            logo: Logo::default(),
            environment,
            label_cap: (t.type_scale.unit * 1.2).max(MIN_CAP),
            tracking: t.type_scale.label_tracking,
            calm: false,
        }
    }

    /// How far piece `depth` (0 wings, 1 pane, 2 puck, 3 blade) has materialised: back to front,
    /// each over most of the show and a little after the one behind it.
    fn shown(&self, depth: usize) -> f32 {
        let lag = 0.15 * depth as f32;
        let k = ((self.show.progress() - lag) / (1.0 - 0.45)).clamp(0.0, 1.0);
        if !self.show.running() {
            return 1.0;
        }
        let k = 1.0 - k;
        1.0 - k * k * k
    }

    /// Lateral offset of depth `depth` (index into `SHEAR`), or of the tunnel and sky (`None`).
    fn shear_px(&self, depth: Option<usize>) -> f32 {
        let k = depth.map_or(BACK_SHEAR, |d| SHEAR[d]);
        self.shear.value() * G_LIMIT * k
    }

    /// The glint level on depth `layer` (0 wings, 1 pane, 2 blade) during the Z-hop.
    fn glint(&self, layer: usize) -> f32 {
        if !self.hop.running() {
            return 0.0;
        }
        let t = self.hop.progress() * HOP_S - HOP_STEP * layer as f32;
        if (0.0..GLINT_S).contains(&t) {
            GLINT_LEVEL
        } else {
            0.0
        }
    }

    /// How far the pane has receded (px).
    fn receded(&self) -> f32 {
        if self.recede.running() {
            RECEDE_PX * (PI * self.recede.progress()).sin()
        } else {
            0.0
        }
    }

    /// The pane this frame: sheared, and inset while it recedes.
    fn pane(&self, g: &Geom) -> Rect {
        g.pane
            .translate(vec2(self.shear_px(Some(1)), 0.0))
            .inset(self.receded())
    }

    /// The left (`sign` −1) or right (`sign` 1) wing's glass piece this frame.
    fn wing(&self, g: &Geom, sign: f32) -> Piece {
        // Toe-in follows the yaw: turning right swings the right wing further in.
        let toe = self.toe.value().clamp(-1.0, 1.0) * TOE_DEG * TOE_PX_PER_DEG * sign;
        let dx = self.shear_px(Some(0));
        let mirror = |v: Vec2| vec2(v.x * -sign + dx, v.y);
        tapered(
            mirror(g.wing_inner),
            mirror(g.wing_outer),
            g.wing_inner_r,
            (g.wing_outer_r + toe).max(g.wing_inner_r),
        )
        .section(18.0, 30.0, 8.0)
    }
}

impl Cluster for VisionSpatialGlass {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = if input.dt.is_finite() {
            input.dt.clamp(0.0, 0.25)
        } else {
            0.0
        };
        let sig = *input.signals;
        if sig.race_on && !self.sig.race_on && !self.show.running() {
            self.show.restart();
        }
        self.sig = sig;
        self.palette = *input.palette;
        self.calm = input.reduced_flash;
        let was_rpm = self.rev_rpm;
        self.rev_rpm = if self.fresh && sig.rpm.is_finite() {
            sig.rpm
        } else {
            backlash(self.rev_rpm, sig.rpm, REV_BAND)
        };
        self.rev_target = rev_fraction(&sig, self.rev_rpm);
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.numeral = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let rate = if self.rev_rpm == was_rpm { 0.0 } else { sig.rpm_rate };
        self.rev.update_tracking(self.rev_target, rate / span, dt);
        self.strip.update(&sig, &self.palette, dt, input.reduced_flash);
        self.cue.update(self.strip.over_shift(), 0.25, dt);
        if self.fresh {
            self.gear.snap(sig.gear);
            self.rev.reset(self.rev_target);
            self.fresh = false;
        }
        self.gear.update(sig.gear, self.cue.held(), dt);
        if self.gear.since_change() == 0.0 && self.gear.was_upshift() && !self.calm {
            self.hop.restart();
            self.warp.restart();
            self.recede.restart();
        }
        self.hop.update(dt);
        self.warp.update(dt);
        self.recede.update(dt);
        self.show.update(dt);
        self.speed.update(sig.speed);

        // Felt force: accelerating right pushes the stack left.
        self.shear_target = -felt(sig.g_lat);
        self.shear.update(self.shear_target, dt);
        self.bead_target = [-felt(sig.g_lat), felt(sig.g_long)];
        for (s, t) in self.bead.iter_mut().zip(self.bead_target) {
            s.update(t, dt);
        }
        self.toe
            .update((derived_yaw(sig.g_lat, sig.speed_ms) / YAW_FULL).clamp(-1.0, 1.0), dt);
        let slip = if sig.slip_deg.is_finite() {
            ((sig.slip_deg.abs() - 3.0) / 15.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.halo.update(slip * slip * (3.0 - 2.0 * slip), dt);
        let unit = |x: f32| if x.is_finite() { x.clamp(0.0, 1.0) } else { 0.0 };
        self.floor_throttle.update(unit(sig.throttle), dt);
        self.floor_brake.update(unit(sig.brake), dt);
        let speed = if sig.speed_ms.is_finite() {
            sig.speed_ms.max(0.0)
        } else {
            0.0
        };
        if speed >= STREAK_MIN_SPEED {
            self.flow = (self.flow + speed * STREAK_RATE * dt).rem_euclid(1.0);
            self.cloud_px = (self.cloud_px + speed * CLOUD_DRIFT * dt).rem_euclid(1.0e4);
        }

        // One light on every rim: lateral G swings it, acceleration leans it.
        let op = &input.tokens.optics;
        self.light.update(sig.g_lat, dt);
        self.lean_target = [
            felt(sig.g_long) * G_LIMIT * op.tilt_per_g,
            felt(sig.g_lat) * G_LIMIT * op.tilt_per_g,
        ];
        for (s, t) in self.lean.iter_mut().zip(self.lean_target) {
            s.update(t, dt);
        }
        let mut glass = studio_glass(
            input.tokens.optical_glass(&self.palette),
            self.environment,
            self.palette.primary,
        );
        aim_studio(
            &mut glass,
            self.light.angle(),
            op.key_elevation,
            vec2(self.lean[0].value(), self.lean[1].value()),
        );
        self.glass = glass;
        self.clear = glass;
        self.clear.absorption = [absorption_coefficient(op.clarity, op.tint_depth); 3];
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &COCKPIT;
        let pal = self.palette;
        let heat = self.cue.level();
        self.draw_sky(p, g, &pal);
        self.draw_tunnel(p, g, &pal, heat);
        self.draw_streaks(p, g, &pal);
        self.draw_cores(p, g, &pal);
        self.draw_pane_lights(p, g, &pal, heat);
        self.draw_puck(p, g, &pal);
        self.draw_pods(p, g, &pal);
        self.draw_glass(p, g);
        self.draw_wing_text(p, g, &pal);
        self.draw_rims(p, g, &pal, heat);
    }

    fn animating(&self) -> bool {
        self.show.running()
            || self.sig.speed_ms >= STREAK_MIN_SPEED
            || self.hop.running()
            || self.warp.running()
            || self.recede.running()
            || self.strip.animating()
            || self.gear.animating()
            || self.cue.animating()
            || !self.rev.settled(self.rev_target, 0.001)
            || !self.shear.settled(self.shear_target, 0.002)
            || !self.lean[0].settled(self.lean_target[0], 0.001)
            || !self.lean[1].settled(self.lean_target[1], 0.001)
            || !self.bead[0].settled(self.bead_target[0], 0.002)
            || !self.bead[1].settled(self.bead_target[1], 0.002)
    }
}

impl VisionSpatialGlass {
    /// The dim visionOS sky: a floor glow under the horizon and soft clouds over it, all tilted
    /// by felt lateral G about the vanishing point, then the horizon line itself.
    fn draw_sky(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let v = g.vanish + vec2(self.shear_px(None), 0.0);
        let tilt = deg(self.shear.value() * G_LIMIT * HORIZON_TILT);
        let env = g.envelope;
        // Throttle lifts the floor glow; the brake warms it towards the warning red.
        let floor = pal
            .primary
            .lerp(roles.warning, FLOOR_BRAKE * self.floor_brake.value())
            .intensity(0.6 + FLOOR_THROTTLE * self.floor_throttle.value());
        p.scoped(|p| {
            // The sky never leaves the envelope, whatever the tilt or the drift.
            p.clip(env, 0.0);
            p.transform(&Affine::rotate_about(tilt, v));
            // The floor glow: brightest at the horizon under the vanishing point, fading out
            // sideways and down (a radial gradient squashed to the floor's perspective).
            let squash = FLOOR_SQUASH;
            p.scoped(|p| {
                p.transform(
                    &Affine::translate(-v)
                        .then(&Affine::scale_xy(1.0, squash))
                        .then(&Affine::translate(v)),
                );
                p.fill(
                    &Shape::rect(Rect {
                        min: vec2(env.min.x, v.y),
                        max: vec2(env.max.x, v.y + (env.max.y - v.y) / squash),
                    }),
                    Gradient::radial(v, env.max.x)
                        .stop(0.0, floor.intensity(SKY_LEVEL * 1.6))
                        .stop(0.45, floor.intensity(SKY_LEVEL * 0.5))
                        .stop(1.0, floor.with_alpha(0.0)),
                );
            });
            // Dusk stratus: long thin streaks of cloud above the horizon, drifting slowly with the
            // car's speed (far away, so a fraction of a pixel per metre).
            let cloud = roles.ink.lerp(pal.primary, 0.45);
            for (i, &(x, y, half, k)) in CLOUDS.iter().enumerate() {
                let span = env.width() + 2.0 * CLOUD_WRAP;
                let drift = (x - self.cloud_px * (0.6 + 0.2 * i as f32) - env.min.x + CLOUD_WRAP).rem_euclid(span);
                let cx = env.min.x - CLOUD_WRAP + drift;
                // Fade out towards the envelope's ends, so a streak never pops as it wraps.
                let edge = ((env.max.x - cx.abs()) / CLOUD_WRAP).clamp(0.0, 1.0);
                if edge <= 0.0 {
                    continue;
                }
                let at = vec2(cx, v.y + y);
                p.scoped(|p| {
                    p.transform(
                        &Affine::translate(-at)
                            .then(&Affine::scale_xy(1.0, CLOUD_FLAT))
                            .then(&Affine::translate(at)),
                    );
                    p.fill(
                        &Shape::circle(at, half),
                        Gradient::radial(at, half)
                            .stop(0.0, cloud.intensity(SKY_LEVEL * k * edge))
                            .stop(0.45, cloud.intensity(SKY_LEVEL * 0.4 * k * edge))
                            .stop(1.0, cloud.with_alpha(0.0)),
                    );
                });
            }
            let line = roles.ink.lerp(pal.primary, 0.5);
            p.fill(
                &Shape::line(vec2(env.min.x, v.y), vec2(env.max.x, v.y), 2.0),
                Gradient::linear(vec2(env.min.x, v.y), vec2(env.max.x, v.y))
                    .stop(0.0, line.with_alpha(0.0))
                    .stop(0.5, line.intensity(0.18))
                    .stop(1.0, line.with_alpha(0.0)),
            );
        });
    }

    /// Rank 3: the rev tunnel. Ring `i` lights once the rpm passes `(i + 1) / RINGS` of the span,
    /// back to front; the ring being reached fades in. Lit rings lean from the primary colour to
    /// a pale sky blue towards the driver, rings past the shift point take the shift colour, and
    /// on the limiter the whole lit tunnel holds the shift colour.
    fn draw_tunnel(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let roles = pal.roles;
        let v = g.vanish + vec2(self.shear_px(None), 0.0);
        let fill = self.rev.value().clamp(0.0, 1.0) * RINGS as f32;
        let shift_at = rev_fraction(&self.sig, self.sig.shift_rpm) * RINGS as f32;
        let near = Color::rgb_u32(NEAR_TINT);
        let mut lit: [Option<Color>; RINGS] = [None; RINGS];
        for (i, slot) in lit.iter_mut().enumerate() {
            let on = (fill - i as f32).clamp(0.0, 1.0);
            if on <= 0.0 {
                continue;
            }
            let depth = i as f32 / (RINGS - 1) as f32;
            let hue = if (i as f32 + 1.0) > shift_at + 1e-3 {
                pal.shift
            } else {
                pal.primary.lerp(near, depth)
            };
            *slot = Some(hue.lerp(pal.shift, heat).intensity(TUNNEL_LEVEL * on * fog(i)));
        }
        for (i, _) in lit.iter().enumerate().filter(|(_, c)| c.is_none()) {
            if !g.ring_hidden(i) {
                ring(p, g, v, i as f32, roles.ink.with_alpha(idle_ring(i)), 0.0);
            }
        }
        let head = lit.iter().rposition(Option::is_some);
        // Tubes of light (board v3): a soft wide halo under each lit ring, then the ring, then a
        // thin hot core on the nearer half of the tunnel.
        for (i, color) in lit.iter().enumerate() {
            if let Some(c) = color {
                ring(
                    p,
                    g,
                    v,
                    i as f32,
                    c.intensity(RING_HALO_LEVEL).fade(0.6),
                    RING_HALO_WIDEN,
                );
            }
        }
        // Each lit ring, then the warp: in the bloom layer where its glow stays inside the
        // envelope, plain where it does not.
        let warp = self.warp.running().then(|| {
            let k = self.warp.progress();
            let u = k * k * (RINGS - 1) as f32;
            let color = roles.ink.lerp(pal.primary, 0.25).intensity(WARP_LEVEL * (1.0 - k));
            (u, color, WARP_WIDEN * k)
        });
        let strokes = |p: &mut Painter<'_>, bloomed: bool| {
            for (i, color) in lit.iter().enumerate() {
                // Lit rings behind the pane are drawn too: they glow through its core.
                let Some(c) = color else { continue };
                let u = i as f32;
                if g.ring_blooms(u) != bloomed {
                    continue;
                }
                if Some(i) == head {
                    let on = (fill - u).clamp(0.0, 1.0);
                    let full = c.intensity(HEAD_LEVEL / (TUNNEL_LEVEL * on * fog(i)).max(1e-3));
                    ring(p, g, v, u, full.lerp(roles.ink, HEAD_WHITE).fade(on), HEAD_WIDEN);
                } else {
                    ring(p, g, v, u, *c, 0.0);
                }
            }
            // The gear change: a ring of white light rushes from the vanishing point to the
            // driver, the whole tunnel in one stroke.
            if let Some((u, color, widen)) = warp {
                if g.ring_blooms(u) == bloomed {
                    ring(p, g, v, u, color, widen);
                }
            }
        };
        p.layer(LayerOptions::bloom(), |p| strokes(p, true));
        strokes(p, false);
        // Thin hot cores on the nearer half, without bloom.
        for (i, _) in lit.iter().enumerate().filter(|(_, c)| c.is_some()) {
            let depth = i as f32 / (RINGS - 1) as f32;
            if depth > 0.5 {
                let core = roles.ink.intensity(RING_CORE_LEVEL * (depth - 0.5) * 2.0);
                ring(p, g, v, i as f32, core, 1.2 - Geom::ring_stroke(i));
            }
        }
    }

    /// The tunnel's four corner rails, from the vanishing point to the nearest ring's corners, with
    /// light streaks running along them towards the driver at the car's speed. Each streak brightens
    /// and lengthens as it nears, as a light passing a camera does.
    fn draw_streaks(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let v = g.vanish + vec2(self.shear_px(None), 0.0);
        let near = Color::rgb_u32(NEAR_TINT);
        let corner = g.rail_corner();
        let at = |u: f32, sx: f32, sy: f32| {
            v + vec2(corner.x * sx, corner.y * sy) * Geom::ring_scale(u * (RINGS - 1) as f32)
        };
        let quads = [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)];
        for (sx, sy) in quads {
            p.fill(
                &Shape::line(at(0.0, sx, sy), at(1.0, sx, sy), 1.5),
                roles.ink.with_alpha(RAIL_IDLE),
            );
        }
        if self.sig.speed_ms < STREAK_MIN_SPEED || !self.sig.speed_ms.is_finite() {
            return;
        }
        p.layer(LayerOptions::bloom(), |p| {
            for k in 0..STREAKS {
                let u = (self.flow + k as f32 / STREAKS as f32).rem_euclid(1.0);
                let tail = (u - STREAK_LEN).max(0.0);
                // Fade in from the vanishing point and out at the near end, so no streak pops.
                let level = STREAK_LEVEL * (u / 0.25).min(1.0) * ((1.0 - u) / 0.12).min(1.0);
                if level <= 0.005 || u - tail < 1e-3 {
                    continue;
                }
                let color = pal.primary.lerp(near, u).intensity(level);
                for (sx, sy) in quads {
                    p.fill(&Shape::line(at(tail, sx, sy), at(u, sx, sy), 1.5 + 2.5 * u), color);
                }
            }
        });
    }

    /// Black cores under the pane and the puck, soft at the rim, so the numerals and the bead
    /// keep their contrast over the tunnel and the cabin.
    fn draw_cores(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let black = pal.roles.oled_black.with_alpha(CORE_ALPHA);
        let soft = Glow {
            sigma: 6.0,
            intensity: 1.0,
            fill_inside: true,
        };
        let core = self.pane(g).inset(g.core_inset);
        p.glow(
            &Shape::round_rect(core, (g.pane_radius - g.core_inset).max(1.0)),
            black,
            soft,
        );
        let puck = g.puck + vec2(self.shear_px(Some(2)), 0.0);
        p.glow(&Shape::circle(puck, g.puck_r - 10.0), black, soft);
    }

    /// Rank 1 and 2 on the untilted pane: the gear above the speed. The gear blooms softly only
    /// on the limiter.
    fn draw_pane_lights(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let dx = self.shear_px(Some(1));
        let gear_at = vec2(dx, g.gear_baseline - g.gear_cap * 0.5);
        // The gear sits in the tunnel's light (board v3): a wide faint glow in the colour of the
        // lit tunnel, under the white numeral.
        let fill = self.rev.value().clamp(0.0, 1.0);
        if fill > 0.01 {
            let near = Color::rgb_u32(NEAR_TINT);
            let tint = pal.primary.lerp(near, fill).lerp(pal.shift, heat);
            let r = g.gear_cap * GEAR_GLOW_R;
            p.scoped(|p| {
                p.set_blend(BlendMode::Add);
                p.fill(
                    &Shape::circle(gear_at, r),
                    Gradient::radial(gear_at, r)
                        .stop(0.0, tint.intensity(GEAR_GLOW * fill.sqrt()))
                        .stop(1.0, tint.with_alpha(0.0)),
                );
            });
        }
        self.gear.draw(p, gear_at, self.faces.gear, g.gear_cap, pal);
        if heat > 0.01 {
            p.layer(LayerOptions::bloom(), |p| {
                let c = self.gear.color(pal).fade(0.35 * heat);
                self.gear.draw_with(p, gear_at, self.faces.gear, g.gear_cap, c);
            });
        }
        let speed_at = vec2(dx, g.speed_baseline - g.speed_cap * 0.5);
        let digits = TextStyle::new(self.faces.speed, g.speed_cap)
            .paint(pal.roles.ink)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let field = p.number(speed_at, self.speed.value(), 3, &digits, None);
        // The unit sits on the speed's baseline, just right of its three cells, in the static ink.
        let unit = TextStyle::new(self.faces.label, self.label_cap)
            .paint(static_ink(pal.roles.ink, 1.0))
            .tracking(self.tracking)
            .align(Align::Left, VAlign::Baseline);
        p.text(vec2(field.max.x + 12.0, g.speed_baseline), self.sig.unit.label(), &unit);
    }

    /// Rank 4 under the puck: a tilted reference ring (felt G tilts the puck as a gimbal), the
    /// bead where the force is felt with its halo spreading as the body slips, and three caution
    /// arcs round the rim that step on with the brake.
    fn draw_puck(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let c = g.puck + vec2(self.shear_px(Some(2)), 0.0);
        let (bx, by) = (self.bead[0].value(), self.bead[1].value());
        // The gimbal: rolled by lateral force, foreshortened by longitudinal force.
        let roll = deg(-bx * 12.0);
        let squash = 1.0 - 0.35 * by.abs();
        p.scoped(|p| {
            p.transform(
                &Affine::translate(-c)
                    .then(&Affine::scale_xy(1.0, squash))
                    .then(&Affine::rotate(roll))
                    .then(&Affine::translate(c)),
            );
            p.stroke(
                &Shape::circle(c, g.puck_r * 0.62),
                roles.ink.with_alpha(0.22),
                Stroke::new(2.0),
            );
            p.fill(
                &Shape::line(c - vec2(g.puck_r * 0.62, 0.0), c + vec2(g.puck_r * 0.62, 0.0), 1.5),
                roles.ink.with_alpha(0.16),
            );
        });
        let mut reach = vec2(bx, by) * (G_LIMIT * BEAD_PER_G);
        let len = reach.length();
        if len > BEAD_REACH {
            reach = reach * (BEAD_REACH / len);
        }
        let at = c + reach;
        let d = self.halo.value().clamp(0.0, 1.0);
        p.scoped(|p| {
            p.set_blend(BlendMode::Add);
            let r = g.puck_r * 0.36;
            p.fill(
                &Shape::circle(at, r),
                Gradient::radial(at, r)
                    .stop(0.0, roles.ink.with_alpha(0.95 * (1.0 - 0.6 * d)))
                    .stop(0.3, roles.ink.with_alpha(0.6 * (1.0 - 0.6 * d)))
                    .stop(0.6, pal.primary.with_alpha(0.4))
                    .stop(1.0, pal.primary.with_alpha(0.0)),
            );
            if d > 0.01 {
                let ring_r = g.puck_r * (0.18 + 0.3 * d);
                p.fill(
                    &Shape::ring(at, ring_r, (ring_r * 0.3).max(1.0)),
                    Gradient::radial(at, ring_r * 1.2)
                        .stop(0.6, pal.primary.with_alpha(0.0))
                        .stop(0.83, roles.ink.with_alpha(0.7 * d))
                        .stop(1.0, pal.primary.with_alpha(0.0)),
                );
            }
        });
        let brake = if self.sig.brake.is_finite() {
            self.sig.brake.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let steps = caution_steps(brake);
        let rim = g.puck_r + 10.0;
        for k in 0..3 {
            let a = deg(50.0 + 40.0 * k as f32) - deg(14.0);
            let color = if k < steps {
                roles.caution
            } else {
                roles.ink.with_alpha(0.08)
            };
            p.fill(&Shape::arc(c, rim, 4.0, a, deg(28.0), Cap::Round), color);
        }
    }

    /// Rank 1: the light under each pod (`common::ShiftLights`). Unlit pods are a faint outline
    /// of glass over black.
    fn draw_pods(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let dx = self.shear_px(Some(3));
        let lit = g.pod - Vec2::splat(6.0);
        for c in g.pod_centers() {
            let r = Rect::from_center_size(c + vec2(dx, 0.0), lit);
            p.fill(&Shape::round_rect(r, r.height() * 0.5), pal.roles.ink.with_alpha(0.05));
        }
        for (i, c) in g.pod_centers().iter().enumerate() {
            if let Some(color) = self.strip.light(i, pal) {
                let r = Rect::from_center_size(*c + vec2(dx, 0.0), lit);
                p.fill(&Shape::round_rect(r, r.height() * 0.5), color);
            }
        }
    }

    /// All the glass, in one layer, back to front: the wings, the pane, the puck and the blade.
    /// Each grows in in its turn on show (its thickness, never its opacity).
    fn draw_glass(&self, p: &mut Painter<'_>, g: &Geom) {
        let with = |glass: OpticalGlass, depth: usize| OpticalGlass {
            materialize: self.shown(depth),
            ..glass
        };
        let wings = [self.wing(g, -1.0), self.wing(g, 1.0)];
        let pane = self.pane(g);
        let pane = [Piece::round_rect(pane, g.pane_radius).section(28.0, 24.0, 14.0)];
        let puck = [Piece::circle(g.puck + vec2(self.shear_px(Some(2)), 0.0), g.puck_r).dome(g.puck_r * 0.5, 8.0)];
        let blade =
            [Piece::pill(g.blade.translate(vec2(self.shear_px(Some(3)), 0.0))).dome(g.blade.height() * 0.5, 4.0)];
        p.layer(LayerOptions::glass(0.0), |p| {
            for w in &wings {
                p.fill(
                    &Shape::Blob {
                        pieces: core::slice::from_ref(w),
                        smooth: 0.0,
                    },
                    with(self.glass, 0),
                );
            }
            p.fill(
                &Shape::Blob {
                    pieces: &pane,
                    smooth: 0.0,
                },
                with(self.clear, 1),
            );
            p.fill(
                &Shape::Blob {
                    pieces: &puck,
                    smooth: 0.0,
                },
                with(self.glass, 2),
            );
            p.fill(
                &Shape::Blob {
                    pieces: &blade,
                    smooth: 0.0,
                },
                with(self.glass, 3),
            );
        });
    }

    /// Over the glass, on the wings: the tunnel's rpm numerals (one per 1000 rpm, just inside
    /// their ring, red from the shift point), the labels and the logo zone.
    fn draw_wing_text(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        // Drawn over the glass, so nothing dims them on the way to the eye.
        let ink = static_ink(roles.ink, 1.0);
        let red = static_ink(roles.warning, 1.0);
        let near = Color::rgb_u32(NEAR_TINT);
        let dx = self.shear_px(Some(0));
        let numeral = TextStyle::new(self.faces.numeral, NUMERAL_CAP)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let span = (self.sig.max_rpm - self.sig.idle_rpm).max(1.0);
        let first = (self.sig.idle_rpm / 1000.0).floor().clamp(0.0, 99.0) as i64 + 1;
        let last = (self.sig.max_rpm / 1000.0).floor().clamp(0.0, 99.0) as i64;
        let y = g.vanish.y + g.numeral_dy;
        for n in first..=last {
            let rpm = n as f32 * 1000.0;
            let u = (rpm - self.sig.idle_rpm) / span * (RINGS - 1) as f32;
            let x = g.numeral_x(u);
            if !u.is_finite() || !g.numeral_fits(x) {
                continue;
            }
            // Once the rpm passes it, a numeral lights in its ring's colour: the wings read as
            // the tunnel's scale.
            let over_shift = rpm >= self.sig.shift_rpm;
            let paint = if self.rev_rpm >= rpm {
                let hue = if over_shift {
                    pal.shift
                } else {
                    pal.primary.lerp(near, u / (RINGS - 1) as f32)
                };
                hue.lerp(roles.ink, 0.5).intensity(NUMERAL_LIT)
            } else if over_shift {
                red
            } else {
                ink
            };
            let style = numeral.paint(paint);
            for sign in [-1.0, 1.0] {
                p.number(vec2(x * -sign + dx, y), n, 1, &style, None);
            }
        }
        let label = TextStyle::new(self.faces.label, self.label_cap)
            .paint(ink)
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapMiddle);
        let wing_mid = (g.wing_inner.x + g.wing_outer.x) * 0.5;
        p.text(vec2(wing_mid + dx, g.label_y), "RPM \u{d7}1000", &label);
        p.text(vec2(-wing_mid + dx, g.label_y), "RPM \u{d7}1000", &label);
        let style = LogoStyle {
            tint: ink,
            text: ink.into(),
            cap: self.label_cap,
            ..LogoStyle::new(self.faces.wide)
        };
        logo::draw(
            p,
            Rect::from_center_size(g.logo + vec2(dx, 0.0), vec2(220.0, 44.0)),
            &self.logo,
            &style,
        );
    }

    /// The rims' moments over the glass: the wings' outer slab edges (a steady shift-colour line
    /// on the limiter) and the Z-hop glints, wing to pane to blade.
    fn draw_rims(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let roles = pal.roles;
        let edge = static_ink(roles.ink, 1.0).intensity(0.5);
        let wings = [self.wing(g, -1.0), self.wing(g, 1.0)];
        let sliver = |p: &mut Painter<'_>, w: &Piece, color: Color| {
            if let Outline::Capsule { b, radius_b, .. } = w.outline {
                let start = if b.x < 0.0 { PI - 0.55 } else { -0.55 };
                p.fill(&Shape::arc(b, radius_b - 4.0, 3.0, start, 1.1, Cap::Round), color);
            }
        };
        for w in &wings {
            sliver(p, w, edge);
        }
        // The rims sit at the envelope's edge, where a bloom would spread light past it, so their
        // heat and glints get a drawn halo instead: a wide faint stroke under the bright one.
        if heat > 0.01 {
            for w in &wings {
                if let Outline::Capsule { b, radius_b, .. } = w.outline {
                    let start = if b.x < 0.0 { PI - 0.6 } else { -0.6 };
                    let halo = pal.shift.intensity(heat * 0.35).fade(0.5);
                    p.fill(&Shape::arc(b, radius_b - 8.0, 12.0, start, 1.2, Cap::Round), halo);
                }
                sliver(p, w, pal.shift.intensity(heat));
            }
        }
        let glints = [self.glint(0), self.glint(1), self.glint(2)];
        if glints.iter().all(|&k| k <= 0.0) {
            return;
        }
        let glint = roles.ink.intensity(1.0);
        // Each glint twice: a wide faint pass inward (the halo), then the bright 3 px line.
        for (width, level, inset) in [(10.0, 0.25, 6.0), (3.0, 1.0, 2.0)] {
            if glints[0] > 0.0 {
                for w in &wings {
                    if let Outline::Capsule { b, radius_b, .. } = w.outline {
                        let start = if b.x < 0.0 { PI - 1.0 } else { -1.0 };
                        p.fill(
                            &Shape::arc(b, radius_b - inset, width, start, 2.0, Cap::Round),
                            glint.fade(glints[0] * level),
                        );
                    }
                }
            }
            if glints[1] > 0.0 {
                p.stroke(
                    &Shape::round_rect(self.pane(g).inset(inset), g.pane_radius - inset),
                    glint.fade(glints[1] * level),
                    Stroke::new(width),
                );
            }
            if glints[2] > 0.0 {
                let blade = g.blade.translate(vec2(self.shear_px(Some(3)), 0.0)).inset(inset);
                p.stroke(
                    &Shape::round_rect(blade, blade.height() * 0.5),
                    glint.fade(glints[2] * level),
                    Stroke::new(width),
                );
            }
        }
    }
}

/// Depth fog on lit ring `i` (board v3): far rings fainter, near rings brighter.
fn fog(i: usize) -> f32 {
    let depth = i as f32 / (RINGS - 1) as f32;
    FOG_FAR + (1.0 - FOG_FAR) * depth
}

/// Coverage of unlit ring `i`: brightest at the vanishing point, fading towards the driver.
fn idle_ring(i: usize) -> f32 {
    let depth = i as f32 / (RINGS - 1) as f32;
    RING_IDLE * (1.0 - RING_IDLE_FADE * depth)
}

/// How many of the puck's 3 caution arcs the brake lights: one from 10%, two from 40%, three
/// from 70%.
fn caution_steps(brake: f32) -> usize {
    [0.1, 0.4, 0.7].iter().filter(|&&t| brake >= t).count()
}

/// Strokes the ring at position `u` round `v` (`widen` px wider than its own stroke) as four straight sides and
/// four corner arcs, so each draw covers only its own band and not the whole ring's box.
fn ring(p: &mut Painter<'_>, g: &Geom, v: Vec2, u: f32, color: Color, widen: f32) {
    let (half, r) = g.ring(u);
    let w = (Geom::ring_stroke_at(u) + widen).max(0.5);
    let (hx, hy) = (half.x, half.y);
    let r = r.min(hx).min(hy);
    for (a, b) in [
        (vec2(-hx + r, -hy), vec2(hx - r, -hy)),
        (vec2(-hx + r, hy), vec2(hx - r, hy)),
        (vec2(-hx, -hy + r), vec2(-hx, hy - r)),
        (vec2(hx, -hy + r), vec2(hx, hy - r)),
    ] {
        if (b - a).length() > 0.5 {
            p.fill(&Shape::line(v + a, v + b, w), color);
        }
    }
    for (c, start) in [
        (vec2(-hx + r, -hy + r), PI),
        (vec2(hx - r, -hy + r), -FRAC_PI_2),
        (vec2(hx - r, hy - r), 0.0),
        (vec2(-hx + r, hy - r), FRAC_PI_2),
    ] {
        if r > 0.5 {
            p.fill(&Shape::arc(v + c, r, w, start, FRAC_PI_2, Cap::Butt), color);
        }
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(VisionSpatialGlass::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: &Geom = &COCKPIT;

    #[test]
    fn every_piece_stays_inside_the_envelope() {
        let env = G.envelope;
        let inside =
            |r: Rect| r.min.x >= env.min.x && r.max.x <= env.max.x && r.min.y >= env.min.y && r.max.y <= env.max.y;
        let (half, _) = G.ring((RINGS - 1) as f32);
        let w = Geom::ring_stroke(RINGS - 1) * 0.5;
        assert!(inside(Rect::from_center_size(G.vanish, (half + Vec2::splat(w)) * 2.0)));
        assert!(inside(G.pane));
        assert!(inside(G.blade));
        assert!(inside(Rect::from_center_size(
            G.puck,
            Vec2::splat(2.0 * (G.puck_r + 12.0))
        )));
        let toe_r = G.wing_outer_r + TOE_DEG * TOE_PX_PER_DEG;
        let wing = Rect {
            min: vec2(G.wing_outer.x - toe_r, -toe_r),
            max: vec2(G.wing_inner.x + G.wing_inner_r, toe_r),
        };
        assert!(inside(wing), "wing {wing:?} with toe and shear");
    }

    #[test]
    fn the_wings_toe_in_their_far_edges_shorter() {
        assert!(G.wing_inner_r < G.wing_outer_r);
        let toe = (G.wing_outer_r - G.wing_inner_r)
            .atan2(G.wing_inner.x - G.wing_outer.x)
            .to_degrees();
        assert!((4.0..=16.0).contains(&toe), "taper {toe}°");
        // Outer tip at x −700 (board); the inner tip stops short of the pane (glass never sits on
        // glass), even with the pane sheared to its limit.
        assert!((G.wing_outer.x - G.wing_outer_r + 700.0).abs() < 0.5);
        let pane_reach = G.pane.min.x - SHEAR[1] * G_LIMIT;
        assert!(
            G.wing_inner.x + G.wing_inner_r < pane_reach - 4.0,
            "wing tip meets the pane"
        );
    }

    #[test]
    fn the_rings_recede_to_the_vanishing_point() {
        let mut last = 0.0;
        for i in 0..RINGS {
            let s = Geom::ring_scale(i as f32);
            assert!(s > last, "ring {i} scale {s}");
            last = s;
        }
        assert!((Geom::ring_scale((RINGS - 1) as f32) - 1.0).abs() < 1e-6);
        assert!(G.ring_hidden(0), "the farthest ring hides behind the pane");
        assert!(!G.ring_hidden(RINGS - 1));
    }

    #[test]
    fn the_pods_sit_on_the_blade_evenly() {
        let c = G.pod_centers();
        for w in c.windows(2) {
            assert!((w[1].x - w[0].x - G.pod_pitch).abs() < 1e-3);
        }
        for p in c {
            let r = Rect::from_center_size(p, G.pod);
            assert!(r.min.x >= G.blade.min.x && r.max.x <= G.blade.max.x, "pod {r:?}");
            assert!(r.min.y >= G.blade.min.y && r.max.y <= G.blade.max.y, "pod {r:?}");
        }
        assert!((c[PODS / 2].x).abs() < 1.0, "the middle pod is centred");
    }

    #[test]
    fn every_digit_on_the_pane_sits_in_its_flat_clear_middle() {
        let clear = G.pane.inset(24.0);
        let gear_top = G.gear_baseline - G.gear_cap;
        assert!(gear_top >= clear.min.y && G.gear_baseline <= clear.max.y);
        let speed_top = G.speed_baseline - G.speed_cap;
        // Board v4: the speed reads on top, under the shift lights, the hero gear below it.
        assert!(speed_top >= clear.min.y && G.speed_baseline < gear_top);
    }

    #[test]
    fn the_wing_numerals_land_on_the_wings_clear_of_the_pane() {
        // A 800 to 8000 rpm car: 5, 6 and 7 fit; 8 sits on the outer rim and is left out.
        let span = 7200.0;
        let x = |rpm: f32| G.numeral_x((rpm - 800.0) / span * (RINGS - 1) as f32);
        assert!(!G.numeral_fits(x(4000.0)), "4 would sit under the pane");
        for rpm in [5000.0, 6000.0, 7000.0] {
            assert!(G.numeral_fits(x(rpm)), "{rpm} at {}", x(rpm));
        }
    }

    #[test]
    fn the_brake_steps_the_caution_arcs() {
        assert_eq!(caution_steps(0.0), 0);
        assert_eq!(caution_steps(0.15), 1);
        assert_eq!(caution_steps(0.5), 2);
        assert_eq!(caution_steps(1.0), 3);
    }

    #[test]
    fn the_yaw_is_derived_from_lateral_g_over_speed() {
        assert_eq!(derived_yaw(1.0, 0.5), 0.0);
        assert!((derived_yaw(1.0, G0) - 1.0).abs() < 1e-6);
        assert_eq!(derived_yaw(f32::NAN, 20.0), 0.0);
    }

    #[test]
    fn static_ink_never_brightens_and_caps_white() {
        let white = Color::linear(1.0, 1.0, 1.0, 1.0);
        assert!(static_ink(white, 1.0).luminance() <= STATIC_LUMA + 1e-4);
        let dim = white.intensity(0.05);
        assert_eq!(static_ink(dim, 1.0), dim);
    }
}
