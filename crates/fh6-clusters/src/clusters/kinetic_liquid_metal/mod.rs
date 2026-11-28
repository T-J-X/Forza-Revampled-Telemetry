//! Kinetic Liquid Metal (`ClusterTheme::KineticLiquidMetal`), catalog row 27: the pattern-setter
//! of the Fluid / Generative family and the first Psychedelic dash (cockpit).
//!
//! A sealed borosilicate instrument, benchmarked on the lens-front mercury thermometer
//! (`RESEARCH.md`): a 240° glass channel whose front is a cylinder lens holds a thread of mercury
//! that is the tach, read at its meniscus; a domed ampoule holds a mercury pool under the gear;
//! eleven mercury beads are the shift strip; a spirit-level vial under the speed shows the
//! sideways pull. Every moving figure is the solution of the model in `physics` (the catalog's
//! "Real geometry and maths" row): the pool's surface is the hydrostatic plane
//! tan θ = a_lat ÷ (g + a_vert), reached through the swing of a U-tube column; its volume is
//! kept in the spherical ampoule; the meniscus is the 140° cap of mercury on glass; on the
//! limiter drops pinch off the thread by Rayleigh–Plateau break-up and fill the pool.
//!
//! Zone table (local 4K px round the cockpit anchor; `Geom`):
//!
//! | Zone | Element | Centre | Size | Rank |
//! |---|---|---|---|---|
//! | Beads | 11 mercury beads (shift strip) | x −320..320, y −300, 14 px sag | Ø 26, 40 px band | 1 |
//! | Ampoule | domed borosilicate sphere, gear inside | (0, −40) | R 140 (wall 8), rise 40 | 1 |
//! | Gear | Exo 2 at its heaviest instance (600) plus synthetic weight 0.5, for the brief's 800 | (0, −52) | cap 176 drawn, about 190 seen | 1 |
//! | Pool | mercury under the gear | in the ampoule | rest plane 92 below the centre | 4 (figure) |
//! | Channel | 240° lens-front tube, mercury thread | round (0, −40), 150° → 390° | r 212, glass 44 wide, thread 18 | 3 |
//! | Scale | etched marks every 500 rpm, numerals every 1000 | marks on the outer glass, numerals at r 166 | cap 24 | 5 |
//! | Caps | machined end caps; the lower-left one carries the logo | the channel's two ends | R 30 | 5 |
//! | Capillary | 1 px glass line from the right cap into the ampoule | (184, 66) → (107, 50) | thread 4 | 4 |
//! | Speed | Exo 2 300, tabular, no ghost 8s | (0, 162) | cap 92 | 2 |
//! | Vial | spirit-level G vial | (0, 228) | 440 × 16, bent to R 260 | 4 |
//! | Pedals | twin test tubes, dyed spirit | x −480 (throttle), −430 (brake) | 28 × 260 | 4 |
//! | Readouts | body slip, longitudinal G | x 480, y −70 and 50 | cap 44 | 4 |
//!
//! Draw order: a black core in the ampoule (so a bright cabin never washes out the gear), the
//! pool, the gear and its black keyline, the hot glow under the redline end, the thread with
//! its meniscus and drops, the pedal spirit and the vial's liquid, then one glass layer with
//! every glass piece (it bends all of that), then on top what no glass covers: the etched
//! scale, the caps, the beads, the speed and the readouts.
//!
//! Signature moments:
//! - **Limiter:** only the beads strobe (10 Hz, on the strip's own clock, outer beads steady in
//!   the shift colour). The meniscus swells and warms steadily to `shift`; drops pinch off the
//!   thread's tip at 5 Hz, run down the capillary and fill the pool, which rises the longer the
//!   limiter is held and drains back over 300 ms after the shift.
//! - **Upshift:** the old glyph sinks into the pool (0.92, out in 50 ms) while the new one
//!   surfaces from 1.12 with the 24 px slide on the `fast` spring; the pool takes one knock and
//!   swings once, under-damped, as the U-tube does.
//! - **Idle is still:** nothing moves without an input; the surfaces settle on their planes.
//!
//! Telemetry: rpm (current, idle, limiter), the tokens' shift point, gear, speed, throttle,
//! brake, lateral G, vertical acceleration (packet `acceleration_y`, verified), longitudinal G
//! and body slip (from velocity). Nothing is synthesised.
mod physics;

use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::liquid::LiquidMetal;
use crate::common::shift::LimiterCue;
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::FRAC_PI_2;
use fh6_kit::cluster::Setup;
use fh6_kit::optics::Dispersion;
use fh6_kit::{
    vec2, Align, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gear, Gradient, Hysteresis, ImageId, ImagePaint,
    LayerOptions, Layout, Logo, OpticalGlass, Painter, Palette, Piece, Rect, Shape, Signals, Spring, StackStr, Strobe,
    Stroke, TextStyle, Tokens, VAlign, Vec2,
};
use physics::{Drips, UTube};

/// Shift beads (catalog brief: eleven).
const BEADS: usize = 11;
/// Borosilicate 3.3 (Schott BOROFLOAT 33): n_d 1.47140, n_F − n_C 71.4 × 10⁻⁴, so
/// V_d = 0.4714 ÷ 0.00714 = 66.0 (`RESEARCH.md`).
const BOROSILICATE_ND: f64 = 1.4714;
const BOROSILICATE_VD: f64 = 66.0;
/// The capillary thread's radius (px): drops pinch off at 1.89 times this.
const THREAD_R: f32 = 4.0;
/// Drops a second on the limiter (catalog brief: 5 Hz), seconds a drop takes down the capillary,
/// and the pool's drain time after the shift (brief: 300 ms).
const DRIP_HZ: f32 = 5.0;
const DRIP_TRAVEL_S: f32 = 0.32;
const DRAIN_S: f32 = 0.3;
/// Most extra volume the pool takes, as drops (the pool then stands 22 px higher at most).
const POOL_DROPS: f32 = 36.0;
/// Contact angle of mercury on glass (degrees): the meniscus's cap.
const CONTACT_DEG: f32 = physics::MERCURY_CONTACT_DEG;
/// How much the meniscus swells on the limiter (share of its height).
const MENISCUS_SWELL: f32 = 0.6;
/// Smooth-union blend of the pinching drop and the thread (px; catalog brief: k 10).
const PINCH_K: f32 = 10.0;
/// The pool's knock on an upshift (slope per second): one swing of about 6°.
const UPSHIFT_KNOCK: f32 = 0.9;
/// The old glyph's sink on an upshift: scale and seconds (catalog brief: 0.92, 50 ms).
const SINK_SCALE: f32 = 0.92;
const SINK_S: f32 = 0.05;
/// Capillary length of the pool's edge meniscus (px): the surface drops towards the glass wall
/// over about this distance, as mercury's real capillary length does at the ampoule's scale.
const EDGE_LENGTH: f32 = 7.0;
/// Width of the backlash on the thread's rpm: wider than the engine's bounce on the limiter
/// (about 50 rpm a frame), so the meniscus holds still there.
const REV_BAND: f32 = 60.0;
/// Static parts (labels, numerals, marks) show at most this relative luminance on black (tokens
/// `motion.idle.staticLumaMaxPct`); at 0.19 a label still stands at 4.8:1.
const STATIC_LUMA: f32 = 0.19;
/// Synthetic weight on the gear (the pack instances Exo 2 up to 600; the brief asks for 800). It
/// also closes the seam the MTSDF leaves where the "4"'s bar and stem overlap in the font's
/// outline (fdsm has no overlap support), so the glyph reads as one solid piece.
const GEAR_WEIGHT: f32 = 0.5;
/// Smallest cap of a label (px): about 16 arcmin from 1.5 m.
const MIN_CAP: f32 = 22.0;
/// Opacity of the black core in the ampoule.
const CORE_ALPHA: f32 = 0.92;
/// Share of the shift colour a bead keeps in the strobe's dark phase.
const STROBE_FLOOR: f32 = 0.2;
/// Added to the strip's clock so a phase edge on a frame lands the same way every time.
const STROBE_LEAD: f32 = 1e-4;
/// Hysteresis of the slip readout (degrees) and the G readout (hundredths of a g).
const SLIP_HOLD: f32 = 0.8;
const G_HOLD: f32 = 2.5;
/// Brightness of the secondary readouts (share of the ink).
const DATA_LEVEL: f32 = 0.86;
/// Brightness of the pedal spirit (share of its role colour).
const SPIRIT_LEVEL: f32 = 0.7;
/// Brightness of the colour stripe the thread mirrors (share of `primary`): the tach (rank 3)
/// reads clearly and stays under the gear and the speed.
const STRIPE_LEVEL: f32 = 0.62;
/// Share of the token environment the glass mirrors: the empty channel stays a quiet outline.
const GLASS_ENV: f32 = 0.55;
/// Pieces of the thread drawn with their own bands (the light falls on each from its own side).
const THREAD_CHUNKS: usize = 8;
/// Points on the pool's outline: its surface and its arc.
const SURFACE_POINTS: usize = 13;
const ARC_POINTS: usize = 15;
/// Points on the meniscus cap.
const CAP_POINTS: usize = 9;

/// Geometry of the cockpit view (local 4K px round the anchor): the zone table above, as data.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    /// Centre of the ampoule and of the channel's circle.
    center: Vec2,
    ampoule_r: f32,
    ampoule_wall: f32,
    ampoule_rise: f32,
    ampoule_base: f32,
    gear: Vec2,
    gear_cap: f32,
    /// The pool's surface plane at rest, below the centre.
    pool_rest: f32,
    channel_r: f32,
    /// Half the glass rod's width, and the mercury thread's radius in it.
    channel_half: f32,
    bore: f32,
    channel_rise: f32,
    channel_base: f32,
    /// Channel start (rpm idle) and sweep, radians clockwise from 3 o'clock.
    start: f32,
    sweep: f32,
    numeral_r: f32,
    numeral_cap: f32,
    cap_r: f32,
    /// Where the capillary meets the ampoule's wall (angle from the centre).
    capillary_in: f32,
    bead_half_span: f32,
    bead_y: f32,
    bead_sag: f32,
    bead_r: f32,
    speed: Vec2,
    speed_cap: f32,
    vial: Vec2,
    vial_size: Vec2,
    vial_bend: f32,
    bubble_len: f32,
    tubes: [f32; 2],
    tube_top: f32,
    tube_bottom: f32,
    tube_half: f32,
    readout_x: f32,
    slip_y: f32,
    g_y: f32,
    readout_cap: f32,
}

const COCKPIT: Geom = Geom {
    center: vec2(0.0, -40.0),
    ampoule_r: 140.0,
    ampoule_wall: 8.0,
    ampoule_rise: 40.0,
    ampoule_base: 10.0,
    gear: vec2(0.0, -52.0),
    gear_cap: 176.0,
    pool_rest: 92.0,
    channel_r: 212.0,
    channel_half: 22.0,
    bore: 9.0,
    channel_rise: 22.0,
    channel_base: 6.0,
    start: 150.0 * core::f32::consts::PI / 180.0,
    sweep: 240.0 * core::f32::consts::PI / 180.0,
    numeral_r: 166.0,
    numeral_cap: 24.0,
    cap_r: 30.0,
    capillary_in: 40.0 * core::f32::consts::PI / 180.0,
    bead_half_span: 320.0,
    bead_y: -300.0,
    bead_sag: 14.0,
    bead_r: 13.0,
    speed: vec2(0.0, 162.0),
    speed_cap: 92.0,
    vial: vec2(0.0, 228.0),
    vial_size: vec2(440.0, 16.0),
    vial_bend: 260.0,
    bubble_len: 56.0,
    tubes: [-480.0, -430.0],
    tube_top: -150.0,
    tube_bottom: 110.0,
    tube_half: 14.0,
    readout_x: 480.0,
    slip_y: -70.0,
    g_y: 50.0,
    readout_cap: 44.0,
};

impl Geom {
    /// The channel's centre line at `angle`.
    fn on_channel(&self, angle: f32, dr: f32) -> Vec2 {
        Vec2::polar(self.center, self.channel_r + dr, angle)
    }

    /// The angle of the thread's tip at rpm fraction `f` (0 idle .. 1 limiter).
    fn angle_at(&self, f: f32) -> f32 {
        self.start + self.sweep * f.clamp(0.0, 1.0)
    }

    /// The two end caps: (left, right), each a little past the channel's ends.
    fn caps(&self) -> [Vec2; 2] {
        let past = (self.cap_r * 0.55) / self.channel_r;
        [
            self.on_channel(self.start - past, 0.0),
            self.on_channel(self.start + self.sweep + past, 0.0),
        ]
    }

    /// The capillary: from the right cap to the ampoule's wall.
    fn capillary(&self) -> (Vec2, Vec2) {
        let from = self.caps()[1];
        let to = Vec2::polar(self.center, self.ampoule_r - self.ampoule_wall * 0.5, self.capillary_in);
        (from, to)
    }

    /// The bead centres: evenly spaced in x on a shallow parabola (sag at the ends).
    fn bead_centers(&self) -> [Vec2; BEADS] {
        let mut out = [Vec2::ZERO; BEADS];
        let step = 2.0 * self.bead_half_span / (BEADS - 1) as f32;
        for (i, c) in out.iter_mut().enumerate() {
            let x = -self.bead_half_span + step * i as f32;
            let u = x / self.bead_half_span.max(1.0);
            *c = vec2(x, self.bead_y + self.bead_sag * u * u);
        }
        out
    }

    /// The inner radius of the ampoule (the pool's sphere).
    fn pool_r(&self) -> f32 {
        self.ampoule_r - self.ampoule_wall
    }

    /// The rect of the rank-4 figure: the ampoule.
    fn figure(&self) -> Rect {
        Rect::from_center_size(self.center, Vec2::splat(self.ampoule_r * 2.0))
    }

    /// The logo zone on the lower-left cap.
    fn logo_zone(&self) -> Rect {
        Rect::from_center_size(self.caps()[0], Vec2::splat(self.cap_r * 1.3))
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces the numerals).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    speed: FontId,
    metric: FontId,
    label: FontId,
    wide: FontId,
    role_display: FontId,
}

/// The beads' lights: `common::ShiftLights` on the strip's own clock, which starts at the shift
/// point (the pilot's `Strip`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Strip {
    lights: ShiftLights,
    since: Option<f32>,
    calm: bool,
}

impl Strip {
    fn new() -> Self {
        Self {
            lights: ShiftLights::new(BEADS as u32),
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

    /// The light of bead `i` this frame, or `None` when it is dark. Past the shift point the
    /// two outer beads stay lit in the shift colour (a still frame of the limiter never reads
    /// as "off"), the rest strobe and keep `STROBE_FLOOR` in the dark phase; in reduced-flash
    /// mode the breath scales the light.
    fn light(&self, i: usize, pal: &Palette) -> Option<Color> {
        if !self.lights.over_shift() {
            return self.lights.color_of(i, pal);
        }
        let outer = i == 0 || i == BEADS - 1;
        let level = if self.calm {
            Strobe::breath(self.clock()).max(STROBE_FLOOR)
        } else if outer || self.lights.color_of(i, pal).is_some() {
            1.0
        } else {
            STROBE_FLOOR
        };
        Some(pal.shift.intensity(level))
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

/// `ink` dimmed so that it shows at [`STATIC_LUMA`] on black. Never brightens; scales the light
/// and keeps the coverage (the overlay blends coverage in encoded space).
fn static_ink(ink: Color) -> Color {
    let seen = ink.luminance() * ink.a.clamp(0.0, 1.0);
    if seen > STATIC_LUMA {
        ink.intensity(STATIC_LUMA / seen)
    } else {
        ink
    }
}

fn finite(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// Loads one of this cluster's assets, when the assets folder is known and holds it.
fn load_asset(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join("kinetic_liquid_metal").join(name);
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The dash still draws: every asset has a procedural fallback.
            tracing::warn!(asset = name, %error, "a Kinetic Liquid Metal asset did not load");
            None
        }
    }
}

/// The light that the mercury mirrors at a point of a cylinder (the thread, or a bead's
/// silhouette) whose surface turns `phi` from the view axis, on the side that faces the key
/// light by `facing` (−1 away .. 1 towards): the studio's band in that direction times
/// mercury's reflectance at that angle (`physics::mercury_reflectance`).
///
/// A face turned φ mirrors the direction 2φ off the view axis. The studio is dark within 44° of
/// the axis (the floor), has a softbox strip round 64° on the key light's side and a far strip on
/// the other, and its lit floor fills in towards grazing (`docs/specs/optics.md` §7, the
/// family's `materials.liquidMetal` bands).
fn mirrored(metal: &LiquidMetal, stripe: Color, phi: f32, facing: f32) -> Color {
    let seen = (2.0 * phi.abs()).to_degrees();
    let r = physics::mercury_reflectance(phi.cos());
    let key = facing.clamp(0.0, 1.0);
    let far = (-facing).clamp(0.0, 1.0);
    let band = if seen < 44.0 {
        // Straight back towards the viewer: the colour stripe behind the bore that a lens-front
        // thermometer's mercury mirrors (US1819919), here in the palette's primary.
        metal.floor.lerp(stripe, 1.0 - (seen / 44.0).powi(2))
    } else if seen < 58.0 {
        let t = (seen - 44.0) / 14.0;
        metal.horizon.lerp(metal.falloff, t * (0.35 + 0.65 * key))
    } else if seen < 70.0 {
        // The softbox strip on the light's side, the far strip on the other.
        metal
            .falloff
            .lerp(metal.softbox, key)
            .lerp(metal.falloff, 1.0 - key.max(far * 0.5))
    } else if seen < 120.0 {
        metal.horizon.lerp(metal.falloff, 0.4)
    } else {
        metal.lower.lerp(metal.horizon, 0.55)
    };
    band.intensity(r).with_alpha(1.0)
}

/// Bands across a mercury thread running round `center` between `inner` and `outer`, lit from
/// `facing_out` (how much the outer flank faces the key light, −1..1): a concentric gradient,
/// so every piece of the thread and its meniscus joins without a seam.
fn thread_bands(metal: &LiquidMetal, stripe: Color, center: Vec2, inner: f32, outer: f32, facing_out: f32) -> Gradient {
    let outer = outer.max(inner + 1e-3);
    let mid = 0.5 * (inner + outer);
    let hw = 0.5 * (outer - inner);
    let mut g = Gradient::radial(center, outer);
    for k in 0..8 {
        let u = -0.97 + 1.94 * k as f32 / 7.0;
        let phi = u.clamp(-0.999, 0.999).asin();
        let facing = facing_out * u.signum();
        g = g.stop(
            ((mid + u * hw) / outer).clamp(0.0, 1.0),
            mirrored(metal, stripe, phi, facing),
        );
    }
    g
}

/// The Kinetic Liquid Metal cluster.
#[derive(Debug, Clone)]
pub struct KineticLiquidMetal {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    metal: LiquidMetal,
    glass: OpticalGlass,
    light: VirtualLight,
    /// The thread: rpm from idle (0) to the limiter (1).
    rev: Spring,
    rev_target: f32,
    rev_rpm: f32,
    gear: HeroGear,
    /// The glyph that sinks on an upshift, and how long it has been sinking.
    sinking: Option<(Gear, f32)>,
    speed: TabularSpeed,
    strip: Strip,
    cue: LimiterCue,
    /// The pool's surface: a U-tube column driven by the car's acceleration.
    slosh: UTube,
    a_lat: f32,
    a_vert: f32,
    drips: Drips,
    /// The pool's volume at rest (px³).
    pool_volume: f32,
    slip: Hysteresis,
    g_long: Hysteresis,
    /// The readouts' digits: slip in degrees, longitudinal G in hundredths.
    slip_shown: i64,
    g_shown: i64,
    fresh: bool,
    logo: Logo,
    /// Machined end-cap plate (Blender), when present.
    cap_plate: Option<ImageId>,
    label_cap: f32,
    tracking: f32,
    calm: bool,
}

impl KineticLiquidMetal {
    /// Builds the cluster: faces, springs, the physics and its assets.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let cap_plate = load_asset(setup, "end_cap.png");
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "fluidGenerative");
        let display = setup.fonts.role(FontRole::Display);
        let fluid = setup.fonts.role(FontRole::Fluid);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Exo 2", 800, false).unwrap_or(fluid),
            speed: setup.fonts.find("Exo 2", 300, false).unwrap_or(fluid),
            metric: setup.fonts.find("Exo 2", 600, false).unwrap_or(fluid),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let g = &COCKPIT;
        let drop = physics::drop_volume(physics::drop_radius(THREAD_R));
        let mut glass = OpticalGlass::new(&Dispersion::from_abbe(BOROSILICATE_ND, BOROSILICATE_VD));
        let op = &t.optics;
        glass.profile.exponent = op.edge_exponent;
        glass.light = [op.key_irradiance; 3];
        glass.roughness = op.key_roughness;
        glass.env_gain = [op.environment_gain * GLASS_ENV; 3];
        Self {
            faces,
            sig: Signals::default(),
            palette,
            metal: LiquidMetal::from_palette(&palette),
            glass,
            light: VirtualLight::new(t),
            rev: Spring::new(t.springs.ribbon, 0.0),
            rev_target: 0.0,
            rev_rpm: 0.0,
            gear: HeroGear::new(t),
            sinking: None,
            speed: TabularSpeed::new(),
            strip: Strip::new(),
            cue: LimiterCue::new(),
            slosh: UTube::from_period(t.springs.fluid_slosh.period, t.springs.fluid_slosh.zeta),
            a_lat: 0.0,
            a_vert: 0.0,
            drips: Drips::new(THREAD_R, DRIP_HZ, DRIP_TRAVEL_S, DRAIN_S, POOL_DROPS * drop),
            pool_volume: physics::cap_volume(g.pool_r(), g.pool_rest),
            slip: Hysteresis::new(0.0, SLIP_HOLD),
            g_long: Hysteresis::new(0.0, G_HOLD),
            slip_shown: 0,
            g_shown: 0,
            fresh: true,
            logo: Logo::default(),
            cap_plate,
            label_cap: (t.type_scale.unit * 1.2).max(MIN_CAP),
            tracking: t.type_scale.label_tracking,
            calm: false,
        }
    }

    /// Where `rpm` sits on the channel: idle is 0, the limiter is 1.
    fn rev_fraction(sig: &Signals, rpm: f32) -> f32 {
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let f = (rpm - sig.idle_rpm) / span;
        if f.is_finite() {
            f.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The pool's surface plane: its distance below the ampoule's centre along the effective
    /// gravity, which holds the pool's volume plus the drops that have landed.
    fn pool_plane(&self, g: &Geom) -> f32 {
        physics::cap_distance(g.pool_r(), self.pool_volume + self.drips.pool())
    }

    /// The colour stripe behind the thread that the mercury mirrors: the palette's primary at
    /// `STRIPE_LEVEL`.
    fn stripe(&self) -> Color {
        self.palette.primary.intensity(STRIPE_LEVEL).with_alpha(1.0)
    }

    /// The effective gravity's direction on screen (unit, y down): the pool's "down".
    fn down(&self) -> Vec2 {
        let t = self.slosh.tilt();
        vec2(-t.sin(), t.cos())
    }
}

impl Cluster for KineticLiquidMetal {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = if input.dt.is_finite() {
            input.dt.clamp(0.0, 0.25)
        } else {
            0.0
        };
        let sig = *input.signals;
        self.sig = sig;
        if self.palette != *input.palette {
            self.palette = *input.palette;
            self.metal = LiquidMetal::from_palette(&self.palette);
        }
        self.calm = input.reduced_flash;
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.metric = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }

        // The thread follows rpm through a backlash on the ribbon spring with feed-forward: a
        // pull moves it every frame, the bounce on the limiter leaves it still.
        let was_rpm = self.rev_rpm;
        self.rev_rpm = if self.fresh && sig.rpm.is_finite() {
            sig.rpm
        } else {
            backlash(self.rev_rpm, sig.rpm, REV_BAND)
        };
        self.rev_target = Self::rev_fraction(&sig, self.rev_rpm);
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let rate = if self.rev_rpm == was_rpm {
            0.0
        } else {
            finite(sig.rpm_rate)
        };
        self.rev.update_tracking(self.rev_target, rate / span, dt);

        self.strip.update(&sig, &self.palette, dt, input.reduced_flash);
        self.cue.update(self.strip.over_shift(), 0.25, dt);

        // The car's acceleration in g: lateral (smoothed, as every dash reads it) and vertical
        // (raw packet, verified field; gravity is not in it).
        self.a_lat = finite(sig.g_lat).clamp(-4.0, 4.0);
        self.a_vert = (finite(input.telemetry.acceleration_y) / physics::STANDARD_G).clamp(-4.0, 4.0);

        if self.fresh {
            // Built mid-race: the gear, the thread and the pool are shown as they are.
            self.gear.snap(sig.gear);
            self.rev.reset(self.rev_target);
            self.slosh.reset(physics::surface_tilt(self.a_lat, self.a_vert).tan());
            self.fresh = false;
        }
        let before = self.gear.gear();
        self.gear.update(sig.gear, self.cue.held(), dt);
        if self.gear.since_change() == 0.0 {
            if self.gear.was_upshift() {
                self.sinking = Some((before, 0.0));
                if !self.calm {
                    self.slosh.kick(UPSHIFT_KNOCK);
                }
            } else {
                self.sinking = None;
            }
        } else if let Some((old, t)) = self.sinking {
            let t = t + dt;
            self.sinking = (t < SINK_S).then_some((old, t));
        }

        self.slosh.step(self.a_lat, self.a_vert, dt);
        self.drips.step(self.strip.over_shift(), dt);
        self.speed.update(sig.speed);
        self.slip_shown = self.slip.update(finite(sig.slip_deg).abs());
        self.g_shown = self.g_long.update(finite(sig.g_long) * 100.0);

        // The light: lateral G swings it; the glass and the mercury's bands follow.
        self.light.update(sig.g_lat, dt);
        let op = &input.tokens.optics;
        self.glass.aim(self.light.angle(), op.key_elevation, Vec2::ZERO);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = &COCKPIT;
        let pal = self.palette;
        self.draw_core(p, g, &pal);
        self.draw_pool(p, g);
        self.draw_gear(p, g, &pal);
        self.draw_glow(p, g, &pal);
        self.draw_thread(p, g, &pal);
        self.draw_spirit(p, g, &pal);
        self.draw_glass(p, g);
        self.draw_scale(p, g, &pal);
        self.draw_caps(p, g, &pal);
        self.draw_beads(p, g, &pal);
        self.draw_readouts(p, g, &pal);
    }

    fn animating(&self) -> bool {
        self.strip.animating()
            || self.gear.animating()
            || self.cue.animating()
            || self.sinking.is_some()
            || self.drips.busy()
            || !self.rev.settled(self.rev_target, 0.001)
            || !self.slosh.settled(self.a_lat, self.a_vert, 1e-4)
    }

    fn figure_zone(&self, _layout: &Layout) -> Option<Rect> {
        Some(COCKPIT.figure())
    }
}

impl KineticLiquidMetal {
    /// A black core inside the ampoule, soft towards the glass, so the gear keeps its contrast
    /// over a bright cabin.
    fn draw_core(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let r = g.pool_r();
        p.fill(
            &Shape::circle(g.center, r),
            Gradient::radial(g.center, r)
                .stop(0.0, pal.roles.oled_black.with_alpha(CORE_ALPHA))
                .stop(0.8, pal.roles.oled_black.with_alpha(CORE_ALPHA))
                .stop(1.0, pal.roles.oled_black.with_alpha(0.0)),
        );
    }

    /// The mercury pool: the cap of the ampoule's sphere below the surface plane, its surface
    /// dropping towards the glass over the capillary length (mercury does not wet glass), lit by
    /// the studio's bands from the surface down.
    fn draw_pool(&self, p: &mut Painter<'_>, g: &Geom) {
        let r = g.pool_r();
        let d = self.pool_plane(g);
        if d >= r - 0.5 {
            return;
        }
        let down = self.down();
        let along = vec2(down.y, -down.x);
        let half = (r * r - d * d).max(0.0).sqrt();
        let surface_mid = g.center + down * d;
        let edge_drop = physics::meniscus_height(EDGE_LENGTH, CONTACT_DEG);
        let mut pts = [Vec2::ZERO; SURFACE_POINTS + ARC_POINTS];
        // The surface, from the left end to the right end.
        for (i, pt) in pts.iter_mut().take(SURFACE_POINTS).enumerate() {
            let s = -half + 2.0 * half * i as f32 / (SURFACE_POINTS - 1) as f32;
            let to_wall = (half - s.abs()).max(0.0);
            let drop = edge_drop * (-to_wall / EDGE_LENGTH).exp();
            *pt = surface_mid + along * s + down * drop;
        }
        // The arc of the sphere's section under the surface, right end round to the left.
        let a_right = (surface_mid + along * half - g.center).angle();
        let a_left = (surface_mid - along * half - g.center).angle();
        let mut sweep = a_left - a_right;
        if sweep < 0.0 {
            sweep += core::f32::consts::TAU;
        }
        for (i, pt) in pts.iter_mut().skip(SURFACE_POINTS).enumerate() {
            let k = (i + 1) as f32 / (ARC_POINTS + 1) as f32;
            *pt = Vec2::polar(g.center, r, a_right + sweep * k);
        }
        let depth = (r - d).max(1.0);
        p.fill(
            &Shape::Polygon { points: &pts },
            self.metal.depth_bands(surface_mid, down, depth),
        );
        // The surface's softbox line: the flat mercury mirrors the strip lying across the
        // studio, brightest where it faces the key light.
        let to_light = Vec2::from_angle(self.light.angle());
        let level = 0.55 + 0.45 * to_light.dot(-down).clamp(0.0, 1.0);
        p.fill(
            &Shape::Polyline {
                points: &pts[1..SURFACE_POINTS - 1],
                width: 2.5,
                closed: false,
            },
            self.metal.softbox.intensity(level * physics::mercury_reflectance(0.2)),
        );
    }

    /// Rank 1: the gear, with a black keyline that knocks it out of the pool; on an upshift the
    /// old glyph sinks into the pool while the new one surfaces.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let font = self.faces.gear;
        let keyline = |p: &mut Painter<'_>, label: &str, pos: Vec2, scale: f32, alpha: f32| {
            p.scoped(|p| {
                p.translate(pos);
                p.scale(scale);
                let mut style = TextStyle::new(font, g.gear_cap)
                    .paint(pal.roles.oled_black.with_alpha(alpha))
                    .align(Align::Center, VAlign::CapMiddle);
                style.weight = 1.0;
                p.text(Vec2::ZERO, label, &style);
            });
        };
        if let Some((old, t)) = self.sinking {
            let k = (t / SINK_S).clamp(0.0, 1.0);
            let scale = 1.0 + (SINK_SCALE - 1.0) * k;
            let pos = g.gear + vec2(0.0, 24.0 * k);
            let ink = pal.roles.ink.intensity(1.0 - k);
            p.scoped(|p| {
                p.translate(pos);
                p.scale(scale);
                let mut style = TextStyle::new(font, g.gear_cap)
                    .paint(ink)
                    .align(Align::Center, VAlign::CapMiddle);
                style.weight = GEAR_WEIGHT;
                p.text(Vec2::ZERO, old.label(), &style);
            });
        }
        let (dy, scale) = self.gear.entry();
        let pos = g.gear + vec2(0.0, dy);
        keyline(p, self.gear.gear().label(), pos, scale, 0.95);
        p.scoped(|p| {
            p.translate(pos);
            p.scale(scale);
            let mut style = TextStyle::new(font, g.gear_cap)
                .paint(self.gear.color(pal))
                .align(Align::Center, VAlign::CapMiddle);
            style.weight = GEAR_WEIGHT;
            p.text(Vec2::ZERO, self.gear.gear().label(), &style);
        });
    }

    /// The limiter's one steady cue besides the meniscus: a hot glow under the redline end of
    /// the channel that ramps in and stays (never flashes).
    fn draw_glow(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let heat = self.cue.level();
        if heat <= 0.01 {
            return;
        }
        let at = g.on_channel(g.angle_at(0.96), 0.0);
        let r = 58.0;
        p.fill(
            &Shape::circle(at, r),
            Gradient::radial(at, r)
                .stop(0.0, pal.shift.intensity(0.42 * heat))
                .stop(0.45, pal.shift.intensity(0.14 * heat))
                .stop(1.0, pal.shift.with_alpha(0.0).intensity(0.0)),
        );
    }

    /// Rank 3: the mercury thread from idle to the current rpm, its 140° meniscus (swelling and
    /// warming on the limiter), and on the limiter the drop pinching off the tip and the drops
    /// running down the capillary.
    fn draw_thread(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let c = g.center;
        let (inner, outer) = (g.channel_r - g.bore, g.channel_r + g.bore);
        let frac = self.rev.value().clamp(0.0, 1.0);
        let end = g.angle_at(frac);
        let to_light = Vec2::from_angle(self.light.angle());
        let heat = self.cue.level();
        // The capillary itself: a hairline of glass from the right cap into the ampoule.
        let (cap_from, cap_to) = g.capillary();
        p.fill(&Shape::line(cap_from, cap_to, 1.0), static_ink(pal.roles.hairline));
        let swept = end - g.start;
        if swept * g.channel_r >= 1.0 {
            // Opaque chunks, each with the bands of its own side of the light; concentric, so
            // the joints do not show.
            let step = swept / THREAD_CHUNKS as f32;
            for i in 0..THREAD_CHUNKS {
                let a0 = g.start + step * i as f32;
                let mid = a0 + step * 0.5;
                let facing = Vec2::from_angle(mid).dot(to_light);
                p.fill(
                    &Shape::arc(c, g.channel_r, 2.0 * g.bore, a0, step, Cap::Butt),
                    thread_bands(&self.metal, self.stripe(), c, inner, outer, facing),
                );
            }
            // The rounded foot at idle: the thread comes out of the left cap.
            // The meniscus: a spherical cap for mercury on glass, its height swelling on the
            // limiter, in the tip's own tangent frame.
            let tangent = Vec2::from_angle(end + FRAC_PI_2);
            let radial = Vec2::from_angle(end);
            let a = g.bore;
            let rise = physics::meniscus_height(a, CONTACT_DEG) * (1.0 + MENISCUS_SWELL * heat);
            let mut pts = [Vec2::ZERO; CAP_POINTS + 2];
            let tip = Vec2::polar(c, g.channel_r, end);
            for (i, pt) in pts.iter_mut().take(CAP_POINTS).enumerate() {
                let u = -1.0 + 2.0 * i as f32 / (CAP_POINTS - 1) as f32;
                // A circular cap through (±a, 0) and (0, rise).
                let rc = (a * a + rise * rise) / (2.0 * rise).max(1e-3);
                let h = (rc * rc - (u * a) * (u * a)).max(0.0).sqrt() - (rc - rise);
                *pt = tip + radial * (u * a) + tangent * h.max(0.0);
            }
            pts[CAP_POINTS] = tip + radial * a - tangent;
            pts[CAP_POINTS + 1] = tip - radial * a - tangent;
            let facing = radial.dot(to_light);
            let mut bands = thread_bands(&self.metal, self.stripe(), c, inner, outer, facing);
            if heat > 0.01 {
                // Warming: the cap's light leans to the shift colour, steadily.
                let warm = pal.shift.intensity(0.55 * heat);
                bands = Gradient::radial(c, outer)
                    .stop(inner / outer, self.metal.horizon.lerp(warm, heat))
                    .stop((g.channel_r) / outer, warm.lerp(self.metal.softbox, 0.25))
                    .stop(1.0, self.metal.horizon.lerp(warm, heat));
            }
            p.fill(&Shape::Polygon { points: &pts }, bands);
            // The pinch: on the limiter the next drop swells off the tip and joins the thread
            // through a smooth-union neck that thins until it breaks.
            if self.strip.over_shift() {
                let neck = self.drips.neck();
                let rd = physics::drop_radius(THREAD_R) * (0.4 + 0.6 * neck);
                let out = tip + tangent * (rise + rd * (0.6 + 1.2 * neck));
                let pieces = [
                    Piece::circle(tip + tangent * (rise * 0.5), THREAD_R),
                    Piece::circle(out, rd),
                ];
                p.fill(
                    &Shape::Blob {
                        pieces: &pieces,
                        smooth: PINCH_K * (1.0 - neck * 0.8),
                    },
                    self.metal.bead(out, rd, to_light),
                );
            }
        }
        // Drops in flight down the capillary.
        let rd = physics::drop_radius(THREAD_R);
        for f in self.drips.drops() {
            let pos = cap_from + (cap_to - cap_from) * f;
            let r = rd * (1.0 - 0.6 * ((f - 0.85) / 0.15).clamp(0.0, 1.0));
            p.fill(
                &Shape::circle(pos, r.max(1.0)),
                self.metal.bead(pos, r.max(1.0), to_light),
            );
        }
    }

    /// The pedal tubes' dyed spirit (concave meniscus: spirit wets glass) and the vial's spirit
    /// with its bubble at the highest point of the bent vial.
    fn draw_spirit(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let levels = [finite(self.sig.throttle), finite(self.sig.brake)];
        let colours = [pal.roles.go, pal.roles.warning];
        let inner = g.tube_half - 4.0;
        let height = g.tube_bottom - g.tube_top - inner * 2.0;
        for ((x, level), colour) in g.tubes.iter().zip(levels).zip(colours) {
            let level = level.clamp(0.0, 1.0);
            if level <= 0.005 {
                continue;
            }
            let top = g.tube_bottom - inner - height * level;
            let body = Rect::from_xywh(x - inner, top, inner * 2.0, g.tube_bottom - top);
            let ink = colour.intensity(SPIRIT_LEVEL);
            p.fill(&Shape::round_rect(body, inner), ink);
            // The concave meniscus: the spirit climbs the glass, so its surface dips in the
            // middle (a cap for a contact angle near 0°: a half circle).
            let dip = inner * 0.6;
            p.fill(
                &Shape::Circle {
                    center: vec2(*x, top - inner + dip),
                    radius: inner,
                },
                pal.roles.oled_black,
            );
        }
        // The vial: spirit along its length, the bubble where the effective gravity's "up" is.
        let vial = Rect::from_center_size(g.vial, g.vial_size);
        let inner_v = vial.inset(4.0);
        p.fill(
            &Shape::round_rect(inner_v, inner_v.height() * 0.5),
            pal.primary.intensity(0.10),
        );
        let travel = (inner_v.width() - g.bubble_len) * 0.5;
        let x = physics::bubble_offset(g.vial_bend, self.slosh.tilt()).clamp(-travel, travel);
        let bubble = Rect::from_center_size(vec2(g.vial.x + x, g.vial.y), vec2(g.bubble_len, inner_v.height()));
        p.fill(&Shape::round_rect(bubble, bubble.height() * 0.5), pal.roles.oled_black);
        p.stroke(
            &Shape::round_rect(bubble, bubble.height() * 0.5),
            pal.primary_text.intensity(0.55),
            Stroke::rim_px(1.5),
        );
    }

    /// One glass layer: the ampoule (a dome that magnifies the gear), the channel (a half rod
    /// that magnifies the thread, as a lens-front thermometer does), the vial and the two
    /// pedal tubes.
    fn draw_glass(&self, p: &mut Painter<'_>, g: &Geom) {
        let glass = self.glass;
        let ampoule = [Piece::circle(g.center, g.ampoule_r).dome(g.ampoule_rise, g.ampoule_base)];
        let channel = [
            Piece::arc(g.center, g.channel_r, g.channel_half * 2.0, g.start, g.sweep)
                .dome(g.channel_rise, g.channel_base),
        ];
        let vial = [Piece::pill(Rect::from_center_size(g.vial, g.vial_size)).dome(g.vial_size.y * 0.5, 3.0)];
        let tubes = [
            Piece::pill(Rect::from_xywh(
                g.tubes[0] - g.tube_half,
                g.tube_top,
                g.tube_half * 2.0,
                g.tube_bottom - g.tube_top,
            ))
            .dome(g.tube_half, 3.0),
            Piece::pill(Rect::from_xywh(
                g.tubes[1] - g.tube_half,
                g.tube_top,
                g.tube_half * 2.0,
                g.tube_bottom - g.tube_top,
            ))
            .dome(g.tube_half, 3.0),
        ];
        p.layer(LayerOptions::glass(0.0), |p| {
            for pieces in [&ampoule[..], &channel[..], &vial[..], &tubes[..1], &tubes[1..]] {
                p.fill(&Shape::Blob { pieces, smooth: 0.0 }, glass);
            }
        });
    }

    /// Rank 5: marks etched on the channel's outer glass every 500 rpm (longer every 1000), the
    /// red zone from the shift point, and the numerals inboard.
    fn draw_scale(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let sig = &self.sig;
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let ink = static_ink(pal.roles.ink);
        let outer = g.channel_r + g.channel_half - 3.0;
        let redline = Self::rev_fraction(sig, sig.shift_rpm);
        // The red zone: a band etched along the outer glass from the shift point.
        let red_from = g.angle_at(redline);
        let red_sweep = g.angle_at(1.0) - red_from;
        if red_sweep * outer >= 1.0 {
            p.fill(
                &Shape::arc(g.center, outer - 4.0, 4.0, red_from, red_sweep, Cap::Butt),
                pal.roles.warning.intensity(0.5),
            );
        }
        let first = ((sig.idle_rpm / 500.0).floor() as i32 + 1).clamp(1, 60);
        let label = TextStyle::new(self.faces.label, g.numeral_cap)
            .paint(ink)
            .align(Align::Center, VAlign::CapMiddle);
        for k in first..(first + 40) {
            let rpm = k as f32 * 500.0;
            let f = (rpm - sig.idle_rpm) / span;
            if !f.is_finite() || f > 1.0001 {
                break;
            }
            let a = g.angle_at(f);
            let major = k % 2 == 0;
            let len = if major { 14.0 } else { 7.0 };
            p.fill(
                &Shape::line(
                    Vec2::polar(g.center, outer - len, a),
                    Vec2::polar(g.center, outer, a),
                    if major { 2.0 } else { 1.5 },
                ),
                ink,
            );
            if major {
                let mut s: StackStr<4> = StackStr::new();
                if core::fmt::Write::write_fmt(&mut s, format_args!("{}", k / 2)).is_ok() {
                    p.text(Vec2::polar(g.center, g.numeral_r, a), s.as_str(), &label);
                }
            }
        }
    }

    /// The end caps (the Blender plate when present, else drawn), the logo on the lower-left one.
    fn draw_caps(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let to_light = Vec2::from_angle(self.light.angle());
        for (i, c) in g.caps().into_iter().enumerate() {
            let rect = Rect::from_center_size(c, Vec2::splat(g.cap_r * 2.0));
            match self.cap_plate {
                Some(img) => p.fill(&Shape::circle(c, g.cap_r), ImagePaint::new(img, rect)),
                None => {
                    // Machined dark metal: a soft lit flank toward the light, a knurled ring.
                    p.fill(
                        &Shape::circle(c, g.cap_r),
                        Gradient::radial(c + to_light * (g.cap_r * 0.35), g.cap_r * 1.4)
                            .stop(0.0, self.metal.falloff)
                            .stop(0.5, self.metal.horizon)
                            .stop(1.0, self.metal.floor),
                    );
                    p.stroke(
                        &Shape::circle(c, g.cap_r - 1.0),
                        static_ink(pal.roles.rim_specular),
                        Stroke::rim_px(1.0),
                    );
                }
            }
            if i == 0 {
                logo_zone(p, g.logo_zone(), &self.logo, self.faces.wide, LogoVariant::Dial, pal);
            }
        }
    }

    /// Rank 1: the eleven mercury beads. Dark mercury at rest; lit, each mirrors its stage
    /// colour with a soft halo.
    fn draw_beads(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let to_light = Vec2::from_angle(self.light.angle());
        for (i, c) in g.bead_centers().into_iter().enumerate() {
            match self.strip.light(i, pal) {
                Some(colour) => {
                    let halo = g.bead_r * 2.0;
                    p.fill(
                        &Shape::circle(c, halo),
                        Gradient::radial(c, halo)
                            .stop(0.0, colour.intensity(0.5))
                            .stop(0.5, colour.intensity(0.12))
                            .stop(1.0, colour.with_alpha(0.0).intensity(0.0)),
                    );
                    p.fill(
                        &Shape::circle(c, g.bead_r),
                        Gradient::radial(c + to_light * (g.bead_r * 0.35), g.bead_r * 1.35)
                            .stop(0.0, colour.lerp(pal.roles.ink, 0.35))
                            .stop(0.45, colour)
                            .stop(1.0, colour.intensity(0.55)),
                    );
                }
                None => {
                    p.fill(&Shape::circle(c, g.bead_r), self.metal.bead(c, g.bead_r, to_light));
                }
            }
        }
    }

    /// Rank 2 and 4: the speed with its unit, the body slip and the longitudinal G.
    fn draw_readouts(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        self.speed
            .draw(p, g.speed, self.faces.speed, g.speed_cap, pal.roles.ink, None);
        let unit = match self.sig.unit {
            fh6_kit::SpeedUnit::Mph => "MPH",
            fh6_kit::SpeedUnit::Kmh => "KM/H",
        };
        let label = TextStyle::new(self.faces.label, self.label_cap)
            .paint(static_ink(pal.roles.ink_label))
            .tracking(self.tracking)
            .align(Align::Left, VAlign::Baseline);
        let baseline = g.speed.y + g.speed_cap * 0.5;
        p.text(vec2(g.speed.x + g.speed_cap * 1.32, baseline), unit, &label);

        // Slip: steps to `caution` above 8°.
        let slip = self.slip_shown;
        let slip_ink = if slip > 8 {
            pal.roles.caution
        } else {
            pal.roles.ink.intensity(DATA_LEVEL)
        };
        let value = TextStyle::new(self.faces.metric, g.readout_cap)
            .paint(slip_ink)
            .align(Align::Left, VAlign::CapMiddle)
            .tabular();
        let r = p.number(vec2(g.readout_x, g.slip_y), slip.clamp(0, 99), 2, &value, None);
        let deg_style = TextStyle {
            valign: VAlign::CapTop,
            cap: g.readout_cap * 0.5,
            ..value
        };
        p.text(vec2(r.max.x + 4.0, g.slip_y - g.readout_cap * 0.5), "°", &deg_style);
        let micro = TextStyle {
            valign: VAlign::CapTop,
            align: Align::Left,
            ..label
        };
        p.text(vec2(g.readout_x, g.slip_y + g.readout_cap * 0.5 + 12.0), "SLIP", &micro);

        // Longitudinal G, signed, in hundredths.
        let hundredths = self.g_shown.clamp(-399, 399);
        let mut s: StackStr<8> = StackStr::new();
        let sign = if hundredths < 0 { "−" } else { "+" };
        let a = hundredths.unsigned_abs();
        let _ = core::fmt::Write::write_fmt(&mut s, format_args!("{sign}{}.{:02}", a / 100, a % 100));
        let gv = TextStyle {
            paint: pal.roles.ink.intensity(DATA_LEVEL).into(),
            ..value
        };
        p.text(vec2(g.readout_x, g.g_y), s.as_str(), &gv);
        p.text(vec2(g.readout_x, g.g_y + g.readout_cap * 0.5 + 12.0), "LONG G", &micro);
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(KineticLiquidMetal::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_zone_table_stays_inside_the_cockpit_envelope() {
        let g = &COCKPIT;
        let env = Rect::from_xywh(-720.0, -320.0, 1440.0, 560.0);
        let inside =
            |r: Rect| r.min.x >= env.min.x && r.min.y >= env.min.y && r.max.x <= env.max.x && r.max.y <= env.max.y;
        let outer = g.channel_r + g.channel_half;
        assert!(inside(Rect::from_center_size(g.center, Vec2::splat(outer * 2.0))));
        for c in g.bead_centers() {
            assert!(inside(Rect::from_center_size(c, Vec2::splat(g.bead_r * 2.0))));
        }
        assert!(inside(Rect::from_center_size(g.vial, g.vial_size)));
        assert!(inside(Rect::from_xywh(
            g.tubes[0] - g.tube_half,
            g.tube_top,
            1.0,
            g.tube_bottom - g.tube_top
        )));
        assert!(g.readout_x + 140.0 <= env.max.x);
        // The bead strip is a 40 px band (≤ 42 px strobe strip).
        assert!(g.bead_sag + 2.0 * g.bead_r <= 42.0);
    }

    #[test]
    fn the_beads_clear_the_channel_and_the_speed_clears_the_ampoule() {
        let g = &COCKPIT;
        let outer = g.channel_r + g.channel_half;
        for c in g.bead_centers() {
            assert!((c - g.center).length() - g.bead_r > outer + 4.0, "{c:?}");
        }
        let speed_top = g.speed.y - g.speed_cap * 0.5;
        assert!(speed_top > g.center.y + g.ampoule_r + 4.0);
        assert!(g.speed.y + g.speed_cap * 0.5 < g.vial.y - g.vial_size.y * 0.5);
    }

    #[test]
    fn the_pool_rests_below_the_gear_glyph() {
        let g = &COCKPIT;
        let baseline = g.gear.y + g.gear_cap * 0.5;
        assert!(
            g.center.y + g.pool_rest > baseline,
            "the pool surface stands below the glyph"
        );
        // Full of drops, the plane rises at most 26 px.
        let drop = physics::drop_volume(physics::drop_radius(THREAD_R));
        let full = physics::cap_distance(
            g.pool_r(),
            physics::cap_volume(g.pool_r(), g.pool_rest) + POOL_DROPS * drop,
        );
        assert!(g.pool_rest - full < 26.0, "{}", g.pool_rest - full);
    }

    #[test]
    fn the_gear_sits_where_the_ampoule_dome_tilts_under_22_degrees() {
        let g = &COCKPIT;
        // A dome over radius R rising H: h(s) = H √(1 − (1 − s/R)²) along the inset s.
        let r = g.ampoule_r;
        let h = g.ampoule_rise;
        let reach = g.gear_cap * 0.5 + (g.gear - g.center).length();
        let s = r - reach;
        let u = s / r;
        let slope = h / r * (1.0 - u) / (1.0 - (1.0 - u) * (1.0 - u)).max(1e-6).sqrt();
        assert!(slope.atan().to_degrees() < 22.0, "{}", slope.atan().to_degrees());
    }

    #[test]
    fn mercury_mirrors_dark_head_on_and_light_towards_grazing() {
        let pal = Palette::family(&Tokens::default(), "fluidGenerative");
        let metal = LiquidMetal::from_palette(&pal);
        let black = pal.roles.oled_black.with_alpha(1.0);
        let head_on = mirrored(&metal, black, 0.0, 1.0).luminance();
        let strip = mirrored(&metal, black, 32.0f32.to_radians(), 1.0).luminance();
        assert!(head_on < 0.02 && strip > 0.3, "{head_on} {strip}");
        // The far side of the light carries no softbox.
        assert!(mirrored(&metal, black, 32.0f32.to_radians(), -1.0).luminance() < strip * 0.5);
        // With a stripe behind the bore, head-on mirrors it at mercury's 78%.
        let stripe = pal.primary.with_alpha(1.0);
        let seen = mirrored(&metal, stripe, 0.0, 1.0).luminance();
        assert!((seen / stripe.luminance() - physics::mercury_reflectance(1.0)).abs() < 0.01);
    }
}
