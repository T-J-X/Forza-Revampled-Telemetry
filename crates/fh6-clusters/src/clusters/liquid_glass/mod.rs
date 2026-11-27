//! Liquid Glass (`ClusterTheme::LiquidGlass`), catalog row 1: the pilot of the Liquid Glass
//! family (cockpit and hood).
//!
//! Separate drops of optical glass float over the display, and every one is a lens. Each reads
//! as glass only because of how it bends and catches light (`fh6_kit::OpticalGlass`,
//! `docs/specs/optics.md`): there is no painted rim, no tinted slab and no texture inside the
//! glass. The light lives under the glass, and the glass bends it: the gear is seen through a
//! dome that really magnifies it. Research: `RESEARCH.md`. Board: Penpot page `07 · LiquidGlass`.
//!
//! Zone table (local 4K px round the anchor; the hood value in brackets where it differs):
//!
//! | Zone | Outline | Centre | Size | Glass (rise / edge / base) | Holds |
//! |---|---|---|---|---|---|
//! | Shift cabochons | 16 pills | x −540..540 [−585..585], 46 [33] above the rod | 56 × 22 [58 × 20] | dome 11 [10] / whole / 4 | rank 1: one light under each |
//! | Rev rod | arc band | through (±600, −214) and (0, −250) [(±760, −146), (0, −176)] | 22 [20] wide | half rod 11 [10] / whole / 5 | rank 3: the liquid-light ribbon, a graduation every 1000 rpm |
//! | Bead | disc on the rod | the head of the ribbon | r 20 [17] | dome / whole / 8 | the meniscus; hot on the limiter |
//! | Gear lens | disc | (0, −50) [(0, −12)] | r 160 [140] | dome 72 [63] / whole / 24 [21] | rank 1: the gear, drawn at cap 156 [136] and seen about 1.15 times larger |
//! | Speed capsule | pill | (−466, −50) [(−480, −12)] | 468 × 176 [440 × 156] | dome 30 [26] / whole / 20 [16] | rank 2: speed, cap 100 [88], and its unit |
//! | Info capsule | pill | (466, −50) [(480, −12)] | 468 × 144 [440 × 128] | dome 24 [20] / whole / 20 [16] | power and boost, cap 36 [32] |
//! | G lens | disc | (0, 178) under the gear lens [(192, 64), beside it] | r 52 [34] | dome r / 2 / whole / 12 | rank 4: the G light over a crosshair and a 0.5 g ring |
//! | Foot row | text | y 100 [110], under the capsules | – | – | rpm readout (left), logo zone (right) |
//!
//! Draw order: a black core under each piece that holds numerals (so a bright cabin or bonnet
//! never washes them out), then every light, then all the glass in one layer.
//!
//! Brightness follows the rank: the gear is the brightest thing on the dash (its lens is clear,
//! not tinted), the lit cabochons bloom, and the rev ribbon burns at half strength with a
//! brighter head, so it never outshines the gear.
//!
//! Signature moments:
//! - **Limiter:** only the cabochon strip strobes (10 Hz, `common::ShiftLights`), and the 16
//!   cabochons fuse into one bar of glass. The upper rim of the gear lens mirrors them. The
//!   steady cue is the bead: its light turns white with a warning halo. The ribbon and the
//!   numerals keep their colours.
//! - **Upshift:** a ripple runs out through the rim of the gear lens and wobbles its reflections,
//!   the lens swells by a seventh, and a light runs once round under its rim. The numeral itself
//!   stays still, for the glance that confirms the shift.
//! - **Braking:** the capsules reach for the gear lens and, by 0.75 g, fuse with it through
//!   thin necks of glass, on the tokens' slosh spring.
//! - **Light:** one rig lights every piece. Lateral G swings it up to 25° and acceleration
//!   leans it 6° per g, so highlights travel across all the pieces together.
//! - **Show:** the glass grows in over 200 ms (its thickness, never its opacity) and the
//!   capsules bud out of the gear lens.
//! - **Reduced flash:** the strobe becomes a 2 Hz breath (`common::ShiftLights`), and the ripple,
//!   the swell and the lap of light are left out.
//!
//! Telemetry is native: rpm (current, idle, limiter), the tokens' shift point, gear, speed,
//! acceleration, body slip (from velocity), power and boost. Nothing is synthesised.
use crate::common::hero_gear::HeroGear;
use crate::common::shift::LimiterCue;
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::{PI, TAU};
use fh6_kit::cluster::Setup;
use fh6_kit::logo::{self, LogoStyle};
use fh6_kit::optics::absorption_coefficient;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Bloom, Cap, Cluster, Environment, FontId, FontRole, FrameInput, Glow, Gradient,
    ImageId, ImagePaint, LayerOptions, Layout, Logo, OpticalGlass, Painter, Palette, Piece, Rect, Refraction, Ripple,
    Shape, Signals, Smoothed, Spring, Stroke, TextStyle, Tokens, Tween, VAlign, Vec2, View,
};

/// The 16 cabochons (catalog brief).
const PILLS: usize = 16;
/// Blend length of the gear puck and the side pills while they bud apart (px).
const BUD_SMOOTH: f32 = 20.0;
/// Blend length the cabochons reach on the limiter, as a share of the gap between two of them:
/// past 0.72 of the gap (gap / 2 ln 2) they touch, so at 0.9 they fuse into one bar.
const FUSE_REACH: f32 = 0.9;
/// Blend length of the bead and the rod: the bead rides the rod as a meniscus (px).
const BEAD_SMOOTH: f32 = 6.0;
/// Glass below the rim of the cabochons, the rod, the bead and the G lens (px).
const PILL_BASE: f32 = 4.0;
const ROD_BASE: f32 = 5.0;
const BEAD_BASE: f32 = 8.0;
const LENS_BASE: f32 = 12.0;
/// Blend length at full braking, as a share of the gap between the gear puck and a side pill.
/// Two outlines a gap apart first touch at gap / (2 ln 2), 0.72 of the gap; just past that the
/// side pills fuse with the puck through thin necks of glass.
const BRIDGE_REACH: f32 = 0.78;
/// Braking (g) at which the pieces start to reach for each other, and at which they have fused.
const BRIDGE_G: (f32, f32) = (0.2, 0.75);
/// Share by which the gear lens's rise grows at the peak of the upshift flex, and how long the
/// flex, the ripple and the lap of light take (s).
const FLEX: f32 = 0.15;
const FLEX_S: f32 = 0.42;
/// The upshift ripple on the gear lens: peak height and crest-to-crest distance (px, at the
/// cockpit size). It starts at `RIPPLE_FROM` of the radius and runs out past the rim, so it
/// wobbles the rim's reflections and leaves the numeral still: the glance that confirms a shift
/// lands on a sharp gear. The steepest slope is 2π × height / wavelength, about 8°.
const RIPPLE_HEIGHT: f32 = 0.6;
const RIPPLE_WAVE: f32 = 28.0;
const RIPPLE_FROM: f32 = 0.72;
/// How far the studio's light leans to the car's primary colour (0 white, 1 the pure hue): the
/// cabin's ambient lighting, mirrored in every rim.
const AMBIENT_TINT: f32 = 0.4;
/// Share of the token key light's irradiance on this dash: its glint on a lens stays dimmer than
/// the numeral under it.
const KEY_SHARE: f32 = 0.5;
/// Brightness of the rev ribbon (rank 3) as a share of the primary colour, and of its head: the
/// last `HEAD_PX` of the ribbon brighten to full, so the eye finds the rpm and the gear still
/// outshines the bar.
const RIBBON_LEVEL: f32 = 0.5;
const HEAD_PX: f32 = 120.0;
/// Share of the ribbon's light that the liquid-light caustic carries: a gentle shimmer that flows
/// with the engine, not a pattern.
const CAUSTIC_SHARE: f32 = 0.12;
/// Width of one repeat of the liquid-light tile (px): its caustic cells are then about 64 px,
/// wider than the ribbon, so the light swells and thins along it and never looks scratched.
const LIGHT_TILE: f32 = 2048.0;
/// Most graduations the rod can carry (one per 1000 rpm).
const MAX_GRADUATIONS: usize = 16;
/// Irradiance that white stores in `liquid_light.png`, and radiance that white stores in
/// `studio_env.png` (`assets/clusters/liquid_glass/ASSETS.md`).
const LIGHT_WHITE: f32 = 4.0;
const ENV_WHITE: f32 = 16.0;
/// Speed of the liquid light along the rod at the limiter (px/s); it scales with rpm.
const FLOW_SPEED: f32 = 90.0;
/// Opacity (linear) of the black core under a piece that holds numerals.
const CORE_ALPHA: f32 = 0.92;
/// Lateral or longitudinal G at which the light rig and the G light reach their limits.
const G_LIMIT: f32 = 1.5;

/// Geometry of one view (local 4K px round the anchor): the zone table above, as data.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    /// The rev rod is a circular arc through `(±rod_half_span, rod_end_y)` and `(0, rod_top_y)`.
    rod_half_span: f32,
    rod_end_y: f32,
    rod_top_y: f32,
    rod_half_width: f32,
    /// The cabochon centres run from `-pill_half_span` to `pill_half_span`, `pill_rise` above
    /// the rod's centre line.
    pill_half_span: f32,
    pill: Vec2,
    pill_rise: f32,
    /// Bead radius at the shift point (it is smaller at idle).
    bead_r: f32,
    gear: Vec2,
    gear_r: f32,
    /// Cap height the gear is drawn at; the dome shows it about a sixth larger.
    gear_cap: f32,
    /// The gear lens is a dome: its rise at the middle, and the glass under its rim.
    gear_rise: f32,
    gear_base: f32,
    /// The speed capsule (left, rank 2) and the info capsule (right): centres, sizes and the
    /// rise of each shallow dome. Speed is the taller of the two, as its rank asks.
    speed: Vec2,
    speed_size: Vec2,
    speed_rise: f32,
    info: Vec2,
    info_size: Vec2,
    info_rise: f32,
    /// Glass under the rim of both capsules.
    side_base: f32,
    speed_cap: f32,
    info_cap: f32,
    /// Half the distance between the two rows of the info capsule.
    info_row: f32,
    lens: Vec2,
    lens_r: f32,
    /// The middle of the foot row under the capsules.
    foot_y: f32,
}

const COCKPIT: Geom = Geom {
    rod_half_span: 600.0,
    rod_end_y: -214.0,
    rod_top_y: -250.0,
    rod_half_width: 11.0,
    pill_half_span: 540.0,
    pill: vec2(56.0, 22.0),
    pill_rise: 46.0,
    bead_r: 20.0,
    gear: vec2(0.0, -50.0),
    gear_r: 160.0,
    gear_cap: 156.0,
    gear_rise: 72.0,
    gear_base: 24.0,
    speed: vec2(-466.0, -50.0),
    speed_size: vec2(468.0, 176.0),
    speed_rise: 30.0,
    info: vec2(466.0, -50.0),
    info_size: vec2(468.0, 144.0),
    info_rise: 24.0,
    side_base: 20.0,
    speed_cap: 100.0,
    info_cap: 36.0,
    info_row: 26.0,
    lens: vec2(0.0, 178.0),
    lens_r: 52.0,
    foot_y: 100.0,
};

/// Hood: the scrim is off, so the row is wider and lower; the gear lens is as tall as the
/// envelope allows, and the G lens sits beside it.
const HOOD: Geom = Geom {
    rod_half_span: 760.0,
    rod_end_y: -146.0,
    rod_top_y: -176.0,
    rod_half_width: 10.0,
    pill_half_span: 585.0,
    pill: vec2(58.0, 20.0),
    pill_rise: 33.0,
    bead_r: 17.0,
    gear: vec2(0.0, -12.0),
    gear_r: 140.0,
    gear_cap: 136.0,
    gear_rise: 63.0,
    gear_base: 21.0,
    speed: vec2(-480.0, -12.0),
    speed_size: vec2(440.0, 156.0),
    speed_rise: 26.0,
    info: vec2(480.0, -12.0),
    info_size: vec2(440.0, 128.0),
    info_rise: 20.0,
    side_base: 16.0,
    speed_cap: 88.0,
    info_cap: 32.0,
    info_row: 22.0,
    lens: vec2(192.0, 64.0),
    lens_r: 34.0,
    foot_y: 110.0,
};

impl Geom {
    /// Radius of the rod's centre line: the circle through its two ends and its top.
    fn rod_radius(&self) -> f32 {
        let sag = (self.rod_end_y - self.rod_top_y).max(1e-3);
        (self.rod_half_span * self.rod_half_span + sag * sag) / (2.0 * sag)
    }

    /// Centre of that circle (far below the rod).
    fn rod_center(&self) -> Vec2 {
        vec2(0.0, self.rod_top_y + self.rod_radius())
    }

    /// Half the angle the rod spans, seen from [`Geom::rod_center`].
    fn rod_half_angle(&self) -> f32 {
        (self.rod_half_span / self.rod_radius()).clamp(-1.0, 1.0).asin()
    }

    /// The y of the rod's centre line at `x`.
    fn rod_y(&self, x: f32) -> f32 {
        let r = self.rod_radius();
        self.rod_top_y + r - (r * r - x * x).max(0.0).sqrt()
    }

    /// The point `f` (0..=1) of the way along the rod, from its left end to its right end.
    fn rod_point(&self, f: f32) -> Vec2 {
        let half = self.rod_half_angle();
        Vec2::polar(
            self.rod_center(),
            self.rod_radius(),
            deg(-90.0) - half + 2.0 * half * f.clamp(0.0, 1.0),
        )
    }

    /// The cabochon centres: evenly spaced in x, on an offset curve of the rod.
    fn pill_centers(&self) -> [Vec2; PILLS] {
        let mut out = [Vec2::ZERO; PILLS];
        let step = 2.0 * self.pill_half_span / (PILLS - 1) as f32;
        for (i, c) in out.iter_mut().enumerate() {
            let x = -self.pill_half_span + step * i as f32;
            *c = vec2(x, self.rod_y(x) - self.pill_rise);
        }
        out
    }

    /// The capsule on the left (`sign` −1, speed) or the right (`sign` 1, info) when `shown` of
    /// the way out of the gear lens: it starts as a disc inside the lens and ends at rest.
    fn side_rect(&self, sign: f32, shown: f32) -> Rect {
        let k = shown.clamp(0.0, 1.0);
        let (to, size) = if sign < 0.0 {
            (self.speed, self.speed_size)
        } else {
            (self.info, self.info_size)
        };
        let h = size.y;
        let from = vec2(sign * (self.gear_r - h * 0.5 - 2.0).max(0.0), self.gear.y);
        Rect::from_center_size(from.lerp(to, k), vec2(h + (size.x - h) * k, h))
    }

    /// The rise of the capsule on the side `sign`.
    fn side_rise(&self, sign: f32) -> f32 {
        if sign < 0.0 {
            self.speed_rise
        } else {
            self.info_rise
        }
    }

    /// The widest gap between the gear lens and a capsule at rest.
    fn side_gap(&self) -> f32 {
        let speed = -self.speed.x - self.speed_size.x * 0.5;
        let info = self.info.x - self.info_size.x * 0.5;
        (speed.max(info) - self.gear_r).max(0.0)
    }
}

/// The logo zone on the foot row, under the info capsule (tokens `layout.logoZone`: a wordmark
/// fits 240 × 48).
fn logo_zone(g: &Geom) -> Rect {
    Rect::from_center_size(vec2(g.info.x, g.foot_y), vec2(240.0, 48.0))
}

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
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
    /// The display role at creation, to notice a user font override.
    role_display: FontId,
}

/// The Liquid Glass cluster.
#[derive(Debug, Clone)]
pub struct LiquidGlass {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    /// This frame's glass: the token material under this frame's light rig.
    glass: OpticalGlass,
    /// The same glass without the lean to the palette's hue, for the pieces that hold numerals:
    /// white numerals stay white through 100 px of it.
    clear: OpticalGlass,
    light: VirtualLight,
    /// Lean of the light rig about the screen's x and y axes (radians).
    lean: [Spring; 2],
    lean_target: [f32; 2],
    /// The ribbon: rpm from idle (0) to the limiter (1).
    rev: Spring,
    rev_target: f32,
    gear: HeroGear,
    speed: TabularSpeed,
    shift: ShiftLights,
    cue: LimiterCue,
    /// The upshift flex and the ring light's lap.
    flex: Tween,
    /// The materialise ramp.
    show: Tween,
    /// How far the side pills reach for the gear puck (0 apart, 1 fused): hard braking.
    bridge: Spring,
    bridge_target: f32,
    spot: [Spring; 2],
    spot_target: [f32; 2],
    defocus: Smoothed<f32>,
    power_hp: Smoothed<f32>,
    boost: Smoothed<f32>,
    /// Phase of the liquid light along the rod (px, wrapped at the tile).
    flow: f32,
    logo: Logo,
    light_tile: Option<ImageId>,
    environment: Option<ImageId>,
    /// Cap height of labels and units, and of the rpm readout (tokens `type.unit` and
    /// `type.metricSecondary`): labels are at least 22 px, about 16 arcminutes at 1.5 m.
    label_cap: f32,
    rpm_cap: f32,
    tracking: f32,
    /// Reduced-flash mode: the strobe becomes a breath, and the ripple, the swell and the lap
    /// of light are left out.
    calm: bool,
}

/// Loads one of this cluster's assets, when the assets folder is known and holds it.
fn load_asset(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join("liquid_glass").join(name);
    if !path.is_file() {
        return None;
    }
    match setup.images.load_file(setup.device, setup.queue, &path, None) {
        Ok(image) => Some(image),
        Err(error) => {
            // The dash still draws: it falls back to the analytic studio or a plain ribbon.
            tracing::warn!(asset = name, %error, "a Liquid Glass asset did not load");
            None
        }
    }
}

impl LiquidGlass {
    /// Builds the cluster: faces, springs and its two assets (the liquid-light tile and the
    /// studio environment). Without them it draws a plain ribbon and the kit's analytic studio.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let light_tile = load_asset(setup, "liquid_light.png");
        let environment = load_asset(setup, "studio_env.png");
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "liquidGlass");
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
            // Medium: about 10 px strokes at cap 100, firm enough to read at a glance.
            speed: setup.fonts.find("Saira", 500, false).unwrap_or(display),
            metric: setup.fonts.find("Saira", 400, false).unwrap_or(display),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            wide: setup.fonts.role(FontRole::Wide),
            role_display: display,
        };
        let mut show = Tween::new(t.durations.materialize);
        show.restart();
        Self {
            faces,
            sig: Signals::default(),
            palette,
            view: View::Cockpit,
            glass: t.optical_glass(&palette),
            clear: t.optical_glass(&palette),
            light: VirtualLight::new(t),
            lean: [Spring::new(t.springs.light, 0.0), Spring::new(t.springs.light, 0.0)],
            lean_target: [0.0; 2],
            rev: Spring::new(t.springs.ribbon, 0.0),
            rev_target: 0.0,
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            shift: ShiftLights::new(PILLS as u32),
            cue: LimiterCue::new(),
            flex: Tween::new(FLEX_S),
            show,
            bridge: Spring::new(t.springs.fluid_slosh, 0.0),
            bridge_target: 0.0,
            spot: [Spring::new(t.springs.g_ball, 0.0), Spring::new(t.springs.g_ball, 0.0)],
            spot_target: [0.0; 2],
            defocus: Smoothed::new(0.0, 0.12),
            power_hp: Smoothed::new(0.0, 0.10),
            boost: Smoothed::new(0.0, 0.10),
            flow: 0.0,
            logo: Logo::default(),
            light_tile,
            environment,
            label_cap: (t.type_scale.unit * 1.2).max(22.0),
            rpm_cap: t.type_scale.metric_secondary,
            tracking: t.type_scale.label_tracking,
            calm: false,
        }
    }

    /// Where `rpm` sits on the rod: idle is 0, the limiter is 1.
    fn rev_fraction(sig: &Signals, rpm: f32) -> f32 {
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        let f = (rpm - sig.idle_rpm) / span;
        if f.is_finite() {
            f.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// How far the pieces have materialised (0..=1, eased out).
    fn shown(&self) -> f32 {
        let k = 1.0 - self.show.progress();
        1.0 - k * k * k
    }

    /// The upshift flex: 0 at rest, 1 at its peak.
    fn flexed(&self) -> f32 {
        if self.flex.running() {
            (PI * self.flex.progress()).sin()
        } else {
            0.0
        }
    }

    /// Bead brightness and size: 0.35 at idle rising to 1 at the shift point.
    fn bead_level(&self) -> f32 {
        let k = self.sig.rpm / self.sig.shift_rpm.max(1.0);
        0.35 + 0.65 * if k.is_finite() { k.clamp(0.0, 1.0) } else { 0.0 }
    }
}

impl Cluster for LiquidGlass {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        // Shown again after a menu or a pause: the glass grows in once more.
        if sig.race_on && !self.sig.race_on && !self.show.running() {
            self.show.restart();
        }
        self.sig = sig;
        self.palette = *input.palette;
        self.calm = input.reduced_flash;
        self.rev_target = Self::rev_fraction(&sig, sig.rpm);
        if input.view != self.view {
            self.view = input.view;
            self.rev.reset(self.rev_target);
        }
        // A user display font (FontChoice.display differs from the role) replaces the numerals.
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.metric = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }
        if self.logo != *input.logo {
            self.logo.clone_from(input.logo);
        }

        // The ribbon follows rpm on the tokens' ribbon spring, with feed-forward.
        let span = (sig.max_rpm - sig.idle_rpm).max(1.0);
        self.rev.update_tracking(self.rev_target, sig.rpm_rate / span, dt);
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.cue.update(self.shift.over_shift(), 0.25, dt);
        self.gear.update(sig.gear, self.cue.held(), dt);
        if self.gear.since_change() == 0.0 && self.gear.was_upshift() && !self.calm {
            self.flex.restart();
        }
        self.flex.update(dt);
        self.show.update(dt);
        self.speed.update(sig.speed);

        // The light rig: lateral G swings it, acceleration leans it.
        let op = &input.tokens.optics;
        self.light.update(sig.g_lat, dt);
        let lean = |g: f32| {
            if g.is_finite() {
                (g / G_LIMIT).clamp(-1.0, 1.0) * G_LIMIT * op.tilt_per_g
            } else {
                0.0
            }
        };
        self.lean_target = [lean(sig.g_long), lean(sig.g_lat)];
        self.lean[0].update(self.lean_target[0], dt);
        self.lean[1].update(self.lean_target[1], dt);
        // Liquid inertia: braking makes the side pills reach for the puck, on the slosh spring.
        self.bridge_target = if sig.g_long.is_finite() {
            ((-sig.g_long - BRIDGE_G.0) / (BRIDGE_G.1 - BRIDGE_G.0)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.bridge.update(self.bridge_target, dt);
        let mut glass = input.tokens.optical_glass(&self.palette);
        if let Some(env) = self.environment {
            glass.environment = Environment::Map(env);
            glass.env_gain = glass.env_gain.map(|g| g * ENV_WHITE);
        }
        // The studio's light leans to the car's colour, as a cabin's ambient lighting does.
        let hue = self.palette.primary;
        let peak = hue.r.max(hue.g).max(hue.b).max(1e-4);
        let lean_to = [hue.r, hue.g, hue.b].map(|c| 1.0 - AMBIENT_TINT * (1.0 - c / peak));
        for c in 0..3 {
            glass.env_gain[c] *= lean_to[c];
            glass.light[c] *= lean_to[c] * KEY_SHARE;
        }
        glass.refraction = Refraction::World;
        glass.aim(
            self.light.angle(),
            op.key_elevation,
            vec2(self.lean[0].value(), self.lean[1].value()),
        );
        glass.materialize = self.shown();
        self.glass = glass;
        // The pieces that hold numerals keep the token clarity but not its lean to the hue.
        self.clear = glass;
        self.clear.absorption = [absorption_coefficient(op.clarity, op.tint_depth); 3];

        // Felt force: accelerating right pushes the G light left; accelerating forward pushes
        // it back (down the screen), braking forward (up).
        let reach = geom(self.view).lens_r * 0.5;
        let felt = |g: f32| {
            if g.is_finite() {
                (g / G_LIMIT).clamp(-1.0, 1.0)
            } else {
                0.0
            }
        };
        self.spot_target = [-felt(sig.g_lat) * reach, felt(sig.g_long) * reach];
        self.spot[0].update(self.spot_target[0], dt);
        self.spot[1].update(self.spot_target[1], dt);
        // Grip is focus: body slip from 3° defocuses the light to a full ring by 18°.
        let slip = if sig.slip_deg.is_finite() {
            ((sig.slip_deg.abs() - 3.0) / 15.0).clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.defocus.update(slip * slip * (3.0 - 2.0 * slip), dt);

        let power = input.telemetry.power_watts;
        let boost = input.telemetry.boost_psi;
        self.power_hp.update(
            if power.is_finite() {
                (power / 745.7).max(0.0)
            } else {
                0.0
            },
            dt,
        );
        self.boost.update(if boost.is_finite() { boost } else { 0.0 }, dt);
        // The liquid light flows with the engine.
        self.flow = (self.flow + FLOW_SPEED * self.rev_target * dt).rem_euclid(LIGHT_TILE);
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        let heat = self.cue.level();
        self.draw_cores(p, g, &pal);
        self.draw_rev(p, g, &pal, heat);
        self.draw_shift(p, g, &pal);
        self.draw_gear(p, g, &pal, heat);
        self.draw_readouts(p, g, &pal);
        self.draw_g(p, g, &pal);
        self.draw_glass(p, g, heat);
    }

    fn animating(&self) -> bool {
        self.show.running()
            || self.flex.running()
            || self.shift.animating()
            || self.gear.animating()
            || self.cue.animating()
            || !self.rev.settled(self.rev_target, 0.001)
            || !self.bridge.settled(self.bridge_target, 0.002)
            || !self.lean[0].settled(self.lean_target[0], 0.001)
            || !self.lean[1].settled(self.lean_target[1], 0.001)
            || !self.spot[0].settled(self.spot_target[0], 0.05)
            || !self.spot[1].settled(self.spot_target[1], 0.05)
    }
}

impl LiquidGlass {
    /// A black core under each piece that holds numerals (and under the G lens), soft towards
    /// the rim: the numerals keep their contrast over a bright cabin or bonnet, and the rim
    /// stays clear.
    fn draw_cores(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let shown = self.shown();
        let black = pal.roles.oled_black.with_alpha(CORE_ALPHA * shown);
        let soft = |bevel: f32| Glow {
            sigma: bevel * 0.4,
            intensity: 1.0,
            fill_inside: true,
        };
        // The dome bends every sight line inwards, so nothing within about 16 px of its rim is
        // ever seen: the core stops there.
        p.glow(&Shape::circle(g.gear, (g.gear_r - 14.0).max(1.0)), black, soft(12.0));
        for sign in [-1.0, 1.0] {
            let face = g.side_rect(sign, shown).inset(14.0);
            if face.width() > 1.0 && face.height() > 1.0 {
                p.glow(&Shape::round_rect(face, face.height() * 0.5), black, soft(12.0));
            }
        }
        p.glow(&Shape::circle(g.lens, g.lens_r * 0.45), black, soft(g.lens_r * 0.5));
    }

    /// Rank 3: the scale hairline under the whole rod, the ribbon of liquid light from idle to
    /// the current rpm (warning-coloured past the shift point) and the light of the bead.
    fn draw_rev(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let roles = pal.roles;
        let c = g.rod_center();
        let r = g.rod_radius();
        let half = g.rod_half_angle();
        let (start, sweep) = (deg(-90.0) - half, 2.0 * half);
        let lane = ((g.rod_half_width - 3.0) * 2.0).max(1.0);
        let frac = self.rev.value().clamp(0.0, 1.0);
        let redline = Self::rev_fraction(&self.sig, self.sig.shift_rpm);
        p.fill(
            &Shape::arc(c, r, 2.0, start, sweep * redline, Cap::Butt),
            roles.hairline,
        );
        // The red zone fills the whole lane, dimly, so it reads before the light reaches it.
        p.fill(
            &Shape::arc(c, r, lane, start + sweep * redline, sweep * (1.0 - redline), Cap::Butt),
            roles.warning.fade(0.28),
        );
        // Graduations every 1000 rpm, like the marks on a thermometer: light where the rod is
        // empty, dark where the light has reached them. The rod and the bead magnify them.
        let span = (self.sig.max_rpm - self.sig.idle_rpm).max(1.0);
        let first = (self.sig.idle_rpm / 1000.0).floor().clamp(0.0, 100.0) as i32 + 1;
        let graduation = |p: &mut Painter<'_>, i: usize, lit: bool| {
            let rpm = (first + i as i32) as f32 * 1000.0;
            let f = (rpm - self.sig.idle_rpm) / span;
            if !f.is_finite() || f <= 0.0 || f >= 1.0 || (f <= frac) != lit {
                return;
            }
            let a = start + sweep * f;
            let (ink, width) = if lit {
                (roles.oled_black.with_alpha(0.85), 3.0)
            } else {
                (roles.ink.with_alpha(0.3), 2.0)
            };
            p.fill(
                &Shape::line(
                    Vec2::polar(c, r - lane * 0.5, a),
                    Vec2::polar(c, r + lane * 0.5, a),
                    width,
                ),
                ink,
            );
        };
        for i in 0..MAX_GRADUATIONS {
            graduation(p, i, false);
        }
        let quiet = LayerOptions {
            bloom: Some(Bloom {
                threshold: 0.7,
                intensity: 0.25,
            }),
            ..LayerOptions::default()
        };
        p.layer(quiet, |p| {
            if frac > 0.002 {
                // The body burns at `RIBBON_LEVEL`, its last `HEAD_PX` brighten to full, and a
                // narrow core runs along the middle as in a light pipe. With the tile, the
                // caustic carries `CAUSTIC_SHARE` of the light and flows with the engine.
                // The segments are opaque (light scaled, coverage kept), so where they meet
                // nothing doubles up.
                let share = if self.light_tile.is_some() { CAUSTIC_SHARE } else { 0.0 };
                let steady = 1.0 - share;
                let head = (HEAD_PX / (r * sweep).max(1.0)).min(frac);
                let body_end = (frac - head).max(0.0);
                let hue = |f: f32| if f > redline { roles.warning } else { pal.primary };
                let segment = |p: &mut Painter<'_>, from: f32, to: f32, color: fh6_kit::Color, cap: Cap| {
                    if to > from {
                        p.fill(
                            &Shape::arc(c, r, lane, start + sweep * from, sweep * (to - from), cap),
                            color.intensity(RIBBON_LEVEL * steady),
                        );
                    }
                };
                segment(p, 0.0, body_end.min(redline), pal.primary, Cap::Round);
                segment(p, redline, body_end, roles.warning, Cap::Butt);
                // The head: a gradient along its chord (the rod is nearly straight over it).
                let (h0, h1) = (g.rod_point(body_end), g.rod_point(frac));
                p.fill(
                    &Shape::arc(
                        c,
                        r,
                        lane,
                        start + sweep * body_end,
                        sweep * (frac - body_end),
                        Cap::Round,
                    ),
                    Gradient::linear(h0, h1)
                        .stop(0.0, hue(body_end).intensity(RIBBON_LEVEL * steady))
                        .stop(1.0, hue(frac).intensity(steady)),
                );
                let core = (lane * 0.3).max(1.0);
                let cool = frac.min(redline);
                p.fill(
                    &Shape::arc(c, r, core, start, sweep * cool, Cap::Round),
                    pal.primary.lerp(roles.ink, 0.3).fade(0.35),
                );
                if frac > redline {
                    p.fill(
                        &Shape::arc(c, r, core, start + sweep * redline, sweep * (frac - redline), Cap::Butt),
                        roles.warning.lerp(roles.ink, 0.3).fade(0.35),
                    );
                }
                if let Some(tile) = self.light_tile {
                    let rect = Rect::from_xywh(-self.flow, g.rod_top_y - LIGHT_TILE * 0.5, LIGHT_TILE, LIGHT_TILE);
                    p.scoped(|p| {
                        p.set_blend(BlendMode::Add);
                        p.fill(
                            &Shape::arc(c, r, lane, start, sweep * frac, Cap::Round),
                            ImagePaint::new(tile, rect)
                                .tint(pal.primary.intensity(LIGHT_WHITE * share * RIBBON_LEVEL))
                                .repeat(),
                        );
                    });
                }
            }
            for i in 0..MAX_GRADUATIONS {
                graduation(p, i, true);
            }
            // The bead's light: the primary hue, turning white with a warning halo on the
            // limiter. Radial gradients that end at zero, so no halo shows a cut edge on black.
            let level = self.bead_level();
            let at = g.rod_point(frac);
            if heat > 0.01 {
                let halo = g.bead_r * 2.2;
                p.fill(
                    &Shape::circle(at, halo),
                    Gradient::radial(at, halo)
                        .stop(0.0, roles.warning.fade(0.8 * heat))
                        .stop(0.3, roles.warning.fade(0.45 * heat))
                        .stop(0.65, roles.warning.fade(0.1 * heat))
                        .stop(1.0, roles.warning.with_alpha(0.0)),
                );
            }
            let glow = g.bead_r * 1.25;
            let light = pal.primary.lerp(roles.ink, heat);
            p.fill(
                &Shape::circle(at, glow),
                Gradient::radial(at, glow)
                    .stop(0.0, light.fade(level))
                    .stop(0.26, light.fade(level))
                    .stop(0.55, light.fade(0.3 * level))
                    .stop(1.0, light.with_alpha(0.0)),
            );
        });
    }

    /// Rank 1: the light under each cabochon (`common::ShiftLights` owns the stages, the fill
    /// order and the 10 Hz limiter strobe). An unlit cabochon is clear glass over black.
    fn draw_shift(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let centers = g.pill_centers();
        p.layer(LayerOptions::bloom(), |p| {
            for (i, c) in centers.iter().enumerate() {
                if let Some(color) = self.shift.color_of(i, pal) {
                    let lit = Rect::from_center_size(*c, g.pill - Vec2::splat(6.0));
                    p.fill(&Shape::round_rect(lit, lit.height() * 0.5), color);
                }
            }
        });
    }

    /// Rank 1: the gear under the clear middle of its dome. Nothing else is under the dome: its
    /// rim shows the studio and, along the top, the mirrored shift lights and rev ribbon. An
    /// upshift sends a light once round under the rim.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let roles = pal.roles;
        let ring_r = g.gear_r * 0.86;
        if self.flex.running() {
            let k = self.flex.progress();
            let tail = deg(80.0);
            p.layer(LayerOptions::bloom(), |p| {
                p.fill(
                    &Shape::arc(g.gear, ring_r, 6.0, deg(-90.0) + TAU * k - tail, tail, Cap::Round),
                    roles.ink.fade(1.0 - k),
                );
            });
        }
        p.scoped(|p| {
            p.set_opacity(self.shown());
            self.gear.draw(p, g.gear, self.faces.gear, g.gear_cap, pal);
            // It blooms only on the limiter, softly.
            if heat > 0.01 {
                p.layer(LayerOptions::bloom(), |p| {
                    let c = self.gear.color(pal).fade(0.35 * heat);
                    self.gear.draw_with(p, g.gear, self.faces.gear, g.gear_cap, c);
                });
            }
        });
    }

    /// Rank 2 and below: speed and its unit under the left pill, power and boost under the
    /// right pill, the rpm readout and the logo zone on the foot row.
    fn draw_readouts(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let shown = self.shown();
        p.scoped(|p| {
            p.set_opacity(shown);

            // Speed: floating light digits (no ghost 8s in this family), the unit on their
            // baseline. The group is centred in the pill.
            let sc = g.side_rect(-1.0, shown).center();
            let unit = TextStyle::new(self.faces.label, self.label_cap)
                .paint(roles.ink_muted)
                .tracking(self.tracking)
                .align(Align::Left, VAlign::Baseline);
            let unit_w = p.measure(self.sig.unit.label(), &unit).width;
            let gap = g.speed_cap * 0.09;
            let digits = TextStyle::new(self.faces.speed, g.speed_cap)
                .paint(roles.ink)
                .align(Align::Center, VAlign::CapMiddle)
                .tabular();
            let field = p.number(
                sc - vec2((unit_w + gap) * 0.5 + g.speed_cap * 0.12, 0.0),
                self.speed.value(),
                3,
                &digits,
                None,
            );
            p.text(
                vec2(field.max.x + gap, sc.y + g.speed_cap * 0.5),
                self.sig.unit.label(),
                &unit,
            );

            // Power and boost: the label at the left, the value and its unit at the right, one
            // baseline per row, all inside the capsule's clear middle.
            let info = g.side_rect(1.0, shown);
            let ic = info.center();
            let value = TextStyle::new(self.faces.metric, g.info_cap)
                .paint(roles.ink.fade(0.86))
                .align(Align::Right, VAlign::Baseline)
                .tabular();
            let small = TextStyle::new(self.faces.label, self.label_cap)
                .paint(roles.ink_label)
                .tracking(self.tracking)
                .align(Align::Left, VAlign::Baseline);
            let units_w = p.measure("PSI", &small).width.max(p.measure("HP", &small).width);
            let inset = info.height() * 0.4;
            let label_x = info.min.x + inset;
            let unit_x = info.max.x - inset - units_w;
            let right = unit_x - 10.0;
            let rows = [
                ic.y - g.info_row + g.info_cap * 0.5,
                ic.y + g.info_row + g.info_cap * 0.5,
            ];
            let hp = self.power_hp.value().round().clamp(0.0, 9999.0) as i64;
            p.number(vec2(right, rows[0]), hp, 4, &value, None);
            p.text(vec2(unit_x, rows[0]), "HP", &small);
            p.text(vec2(label_x, rows[0]), "POWER", &small);
            // Boost with one decimal: whole cells, a narrow point, then the tenth (no allocation).
            let cell = p.fonts().tabular_cell(&value);
            let b = self.boost.value().clamp(-99.9, 99.9);
            let tenths = (b * 10.0).round() as i64;
            let (whole, tenth) = (tenths.abs() / 10, tenths.abs() % 10);
            let tenth_r = p.number(vec2(right, rows[1]), tenth, 1, &value, None);
            let point = TextStyle {
                align: Align::Center,
                ..value
            };
            p.text(vec2(tenth_r.min.x - cell * 0.15, rows[1]), ".", &point);
            let whole_r = p.number(vec2(tenth_r.min.x - cell * 0.3, rows[1]), whole, 2, &value, None);
            if tenths < 0 {
                let digits = if whole >= 10 { 2.0 } else { 1.0 };
                p.text(vec2(whole_r.max.x - cell * (digits + 0.3), rows[1]), "-", &point);
            }
            p.text(vec2(unit_x, rows[1]), "PSI", &small);
            p.text(vec2(label_x, rows[1]), "BOOST", &small);

            // The foot row: the rpm readout (to the nearest 10) under the speed capsule, the
            // car's logo or the custom name under the info capsule, on one baseline.
            let rpm_cap = self.rpm_cap;
            let rpm_style = TextStyle::new(self.faces.metric, rpm_cap)
                .paint(roles.ink_muted)
                .align(Align::Right, VAlign::CapMiddle)
                .tabular();
            let rpm = if self.sig.rpm.is_finite() {
                ((self.sig.rpm / 10.0).round() * 10.0).clamp(0.0, 99_990.0) as i64
            } else {
                0
            };
            let rpm_label = TextStyle {
                valign: VAlign::Baseline,
                ..small
            };
            let label_w = p.measure("RPM", &rpm_label).width;
            let foot = vec2(g.speed.x, g.foot_y);
            let cells = p.number(
                foot + vec2((5.0 * p.fonts().tabular_cell(&rpm_style) - label_w - 10.0) * 0.5, 0.0),
                rpm,
                5,
                &rpm_style,
                None,
            );
            p.text(vec2(cells.max.x + 10.0, g.foot_y + rpm_cap * 0.5), "RPM", &rpm_label);
            // The logo zone, with the wordmark at the label size so its strokes hold on OLED.
            let style = LogoStyle {
                text: roles.ink_muted.into(),
                cap: self.label_cap,
                ..LogoStyle::new(self.faces.wide)
            };
            logo::draw(p, logo_zone(g), &self.logo, &style);
        });
    }

    /// Rank 4: the G light under the lens. It sits where the force is felt (1.5 g reaches 50%
    /// of the radius), a sharp point while the tyres grip and a ring while the car slides,
    /// over a 0.5 g reference ring.
    fn draw_g(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let shown = self.shown();
        let d = self.defocus.value().clamp(0.0, 1.0);
        let c = g.lens + vec2(self.spot[0].value(), self.spot[1].value());
        // The scale under the lens, which the dome magnifies: a crosshair and the 0.5 g ring.
        let scale = roles.ink.with_alpha(0.2 * shown);
        p.stroke(&Shape::circle(g.lens, g.lens_r * 0.5 / 3.0), scale, Stroke::rim_px(1.0));
        let arm = g.lens_r * 0.62;
        for d in [vec2(arm, 0.0), vec2(0.0, arm)] {
            p.fill(
                &Shape::line(g.lens - d, g.lens + d, 1.0),
                roles.ink.with_alpha(0.14 * shown),
            );
        }
        p.scoped(|p| {
            p.set_opacity(shown);
            p.set_blend(BlendMode::Add);
            let point = g.lens_r * 0.3;
            p.fill(
                &Shape::circle(c, point),
                Gradient::radial(c, point)
                    .stop(0.0, roles.ink.with_alpha(0.9 * (1.0 - d)))
                    .stop(0.4, pal.primary.with_alpha(0.35 * (1.0 - d)))
                    .stop(1.0, pal.primary.with_alpha(0.0)),
            );
            if d > 0.01 {
                let ring = g.lens_r * (0.1 + 0.16 * d);
                p.fill(
                    &Shape::ring(c, ring, (ring * 0.35).max(1.0)),
                    Gradient::radial(c, ring * 1.2)
                        .stop(0.6, pal.primary.with_alpha(0.0))
                        .stop(0.83, roles.ink.with_alpha(0.7 * d))
                        .stop(1.0, pal.primary.with_alpha(0.0)),
                );
            }
        });
    }

    /// All the glass, in one layer: the cabochons (fused on the limiter), the rod with its
    /// bead, the gear puck with the two side pills (one liquid group while they bud apart) and
    /// the G lens.
    fn draw_glass(&self, p: &mut Painter<'_>, g: &Geom, heat: f32) {
        let shown = self.shown();
        let glass = self.glass;
        let body = [
            Piece::circle(g.gear, g.gear_r).dome(g.gear_rise * (1.0 + FLEX * self.flexed()), g.gear_base),
            Piece::pill(g.side_rect(-1.0, shown)).dome(g.side_rise(-1.0), g.side_base),
            Piece::pill(g.side_rect(1.0, shown)).dome(g.side_rise(1.0), g.side_base),
        ];
        // The upshift ripple: a ring wave starts in the outer part of the gear lens and runs out
        // past its rim, growing in and dying away, so it wobbles the rim and not the numeral.
        let mut puck_glass = self.clear;
        if self.flex.running() {
            let k = self.flex.progress();
            let scale = g.gear_r / COCKPIT.gear_r;
            let wave = RIPPLE_WAVE * scale;
            let from = g.gear_r * RIPPLE_FROM;
            let grow = (k / 0.15).min(1.0);
            puck_glass.ripple = Ripple {
                center: g.gear,
                radius: from + k * (g.gear_r + wave - from),
                amplitude: RIPPLE_HEIGHT * scale * grow * grow * (3.0 - 2.0 * grow) * (1.0 - k),
                wavelength: wave,
            };
        }
        // Hard braking: the blend length grows until the capsules touch the gear lens.
        let reach = self.bridge.value().clamp(0.0, 1.25);
        let body_smooth = BUD_SMOOTH + (g.side_gap() * BRIDGE_REACH - BUD_SMOOTH).max(0.0) * reach;
        let half = g.rod_half_angle();
        let bead_r = g.bead_r * (0.72 + 0.28 * self.bead_level());
        let rod = [
            Piece::arc(
                g.rod_center(),
                g.rod_radius(),
                g.rod_half_width * 2.0,
                deg(-90.0) - half,
                2.0 * half,
            )
            .dome(g.rod_half_width, ROD_BASE),
            Piece::circle(g.rod_point(self.rev.value()), bead_r).dome(bead_r, BEAD_BASE),
        ];
        let mut pills = [Piece::circle(Vec2::ZERO, 1.0); PILLS];
        for (piece, c) in pills.iter_mut().zip(g.pill_centers()) {
            *piece = Piece::pill(Rect::from_center_size(c, g.pill)).dome(g.pill.y * 0.5, PILL_BASE);
        }
        let lens = [Piece::circle(g.lens, g.lens_r).dome(g.lens_r * 0.5, LENS_BASE)];
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &pills,
                    smooth: (2.0 * g.pill_half_span / (PILLS - 1) as f32 - g.pill.x) * FUSE_REACH * heat,
                },
                glass,
            );
            p.fill(
                &Shape::Blob {
                    pieces: &rod,
                    smooth: BEAD_SMOOTH,
                },
                glass,
            );
            if shown < 1.0 || reach > 0.005 {
                p.fill(
                    &Shape::Blob {
                        pieces: &body,
                        smooth: body_smooth,
                    },
                    puck_glass,
                );
            } else {
                // At rest the three are far apart: separate draws cover far fewer pixels.
                for (i, piece) in body.iter().enumerate() {
                    p.fill(
                        &Shape::Blob {
                            pieces: core::slice::from_ref(piece),
                            smooth: 0.0,
                        },
                        if i == 0 { puck_glass } else { self.clear },
                    );
                }
            }
            p.fill(
                &Shape::Blob {
                    pieces: &lens,
                    smooth: 0.0,
                },
                glass,
            );
        });
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(LiquidGlass::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::optics::{fresnel_dielectric, ripple_slope, transmittance, view_through, Dispersion, Profile};
    use fh6_kit::shape::blob_distance;

    const VIEWS: [&Geom; 2] = [&COCKPIT, &HOOD];

    /// The largest radius (or half height) at which a dome of this rise and half width tilts
    /// less than 22°, and how far from the middle the eye sees through that point.
    fn clear_zone(rise: f32, half: f32, base: f32) -> (f32, f32) {
        let dome = Profile {
            thickness: rise,
            bevel: half,
            base,
            exponent: 2.0,
        };
        let clear = (0..half as i32)
            .rev()
            .map(|r| r as f32)
            .find(|r| dome.normal(half - r, vec2(1.0, 0.0)).z >= deg(22.0).cos())
            .unwrap_or(0.0);
        let eta = Dispersion::SAPPHIRE.primaries()[1];
        let sight = view_through(&dome, half - clear, vec2(1.0, 0.0), eta);
        (clear, clear + sight.offset.x)
    }

    #[test]
    fn the_rod_passes_through_its_three_points() {
        for g in VIEWS {
            assert!((g.rod_y(0.0) - g.rod_top_y).abs() < 1e-2);
            assert!((g.rod_y(g.rod_half_span) - g.rod_end_y).abs() < 1e-2);
            let (left, mid, right) = (g.rod_point(0.0), g.rod_point(0.5), g.rod_point(1.0));
            assert!((left - vec2(-g.rod_half_span, g.rod_end_y)).length() < 0.05, "{left:?}");
            assert!((mid - vec2(0.0, g.rod_top_y)).length() < 0.05, "{mid:?}");
            assert!(
                (right - vec2(g.rod_half_span, g.rod_end_y)).length() < 0.05,
                "{right:?}"
            );
        }
    }

    #[test]
    fn every_piece_stays_inside_its_view_envelope() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            let inside = |r: Rect, what: &str| {
                assert!(env.contains(r.min) && env.contains(r.max), "{what} {r:?} in {env:?}");
            };
            for c in g.pill_centers() {
                inside(Rect::from_center_size(c, g.pill), "cabochon");
            }
            for f in [0.0, 0.5, 1.0] {
                inside(
                    Rect::from_center_size(g.rod_point(f), Vec2::splat(g.bead_r * 2.0)),
                    "bead",
                );
            }
            inside(Rect::from_center_size(g.gear, Vec2::splat(g.gear_r * 2.0)), "gear lens");
            inside(g.side_rect(-1.0, 1.0), "speed capsule");
            inside(g.side_rect(1.0, 1.0), "info capsule");
            inside(Rect::from_center_size(g.lens, Vec2::splat(g.lens_r * 2.0)), "G lens");
            inside(logo_zone(g), "logo zone");
        }
    }

    #[test]
    fn the_pieces_float_apart() {
        for g in VIEWS {
            // The cabochons clear the full-size bead, and each other.
            for c in g.pill_centers() {
                let gap = g.rod_y(c.x) - g.bead_r - (c.y + g.pill.y * 0.5);
                assert!(gap >= 4.0, "cabochon at x {} only {gap} px above the bead", c.x);
            }
            let step = 2.0 * g.pill_half_span / (PILLS - 1) as f32;
            assert!(step - g.pill.x >= 12.0, "cabochons {} px apart", step - g.pill.x);
            assert!(g.pill.y <= 42.0, "the strobing strip is at most 42 px tall");
            // The bead clears the gear lens.
            let lens_top = g.gear.y - g.gear_r;
            assert!(lens_top - (g.rod_y(0.0) + g.bead_r) >= 6.0);
            for sign in [-1.0, 1.0] {
                let side = g.side_rect(sign, 1.0);
                // At rest the smooth union moves the lens and a capsule by under a pixel
                // (k ln(1 + e^(-gap / k))), so drawing them apart does not show.
                let near = if sign < 0.0 { -side.max.x } else { side.min.x };
                let gap = near - g.gear_r;
                let bulge = BUD_SMOOTH * (-gap / BUD_SMOOTH).exp().ln_1p();
                assert!(bulge < 0.75, "a capsule merges at rest by {bulge} px");
                // Under full braking the blend length passes gap / (2 ln 2) and they fuse.
                let pieces = [Piece::circle(g.gear, g.gear_r), Piece::pill(side)];
                let mid = vec2(sign * (g.gear_r + gap * 0.5), g.gear.y);
                assert!(blob_distance(&pieces, BUD_SMOOTH, mid) > 0.0, "apart at rest");
                assert!(
                    blob_distance(&pieces, g.side_gap() * BRIDGE_REACH, mid) < 0.0,
                    "fused at full braking"
                );
                // The foot row sits under the capsules.
                assert!(g.foot_y - 24.0 >= side.max.y + 6.0, "foot row under the capsule");
                // The G lens touches neither capsule.
                let lens = Rect::from_center_size(g.lens, Vec2::splat(g.lens_r * 2.0));
                assert!(lens.max.x + 12.0 <= side.min.x || lens.min.x - 12.0 >= side.max.x);
            }
            // The G lens floats clear of the gear lens and of the logo zone.
            assert!((g.lens - g.gear).length() >= g.gear_r + g.lens_r + 12.0);
            let lens = Rect::from_center_size(g.lens, Vec2::splat(g.lens_r * 2.0));
            assert!(lens.max.x + 12.0 <= logo_zone(g).min.x);
            // Speed is the taller capsule (rank 2 over rank 5).
            assert!(g.speed_size.y > g.info_size.y);
        }
    }

    #[test]
    fn the_numerals_sit_under_the_clear_middle_of_each_lens() {
        for g in VIEWS {
            // The gear sits under the clear middle of its dome. Where the dome tilts less than
            // 22° it mirrors only the dark cone round the viewer (optics.md §7), so no
            // reflection crosses the glyph (Saira Condensed 700 is about 0.56 cap wide).
            let (clear, seen) = clear_zone(g.gear_rise, g.gear_r, g.gear_base);
            let half_diag = ((g.gear_cap * 0.28).powi(2) + (g.gear_cap * 0.5).powi(2)).sqrt();
            assert!(
                half_diag <= seen,
                "gear glyph {half_diag} under a clear zone that shows {seen} (radius {clear})"
            );
            // The dome magnifies: the glyph is seen 10% to 20% larger than it is drawn.
            let dome = Profile {
                thickness: g.gear_rise,
                bevel: g.gear_r,
                base: g.gear_base,
                exponent: 2.0,
            };
            let eta = Dispersion::SAPPHIRE.primaries()[1];
            let near = view_through(&dome, g.gear_r - 40.0, vec2(1.0, 0.0), eta);
            let magnified = 40.0 / (40.0 + near.offset.x);
            assert!((1.10..=1.20).contains(&magnified), "magnification {magnified}");
            // The capsules are shallow domes; their numerals stay inside the clear band too.
            let (_, speed) = clear_zone(g.speed_rise, g.speed_size.y * 0.5, g.side_base);
            assert!(
                g.speed_cap * 0.5 <= speed,
                "speed cap {} under a clear band that shows {speed}",
                g.speed_cap
            );
            let (_, info) = clear_zone(g.info_rise, g.info_size.y * 0.5, g.side_base);
            assert!(
                g.info_row + g.info_cap * 0.5 <= info,
                "info rows reach {} in a clear band of {info}",
                g.info_row + g.info_cap * 0.5
            );
        }
    }

    #[test]
    fn brightness_follows_the_rank() {
        // The gear is seen through clear glass: at least 85% of its light reaches the eye in
        // every channel. The tinted glass would take more of the red than the blue.
        let t = Tokens::default();
        let pal = Palette::family(&t, "liquidGlass");
        for g in VIEWS {
            let path = g.gear_rise + g.gear_base;
            let eta = Dispersion::SAPPHIRE.primaries();
            let clear = absorption_coefficient(t.optics.clarity, t.optics.tint_depth);
            let tinted = t.optical_glass(&pal).absorption;
            for c in 0..3 {
                let seen = (1.0 - fresnel_dielectric(1.0, eta[c])) * transmittance(clear, path);
                assert!(seen >= 0.85, "channel {c}: {seen}");
                assert!(transmittance(tinted[c], path) <= transmittance(clear, path) + 1e-6);
            }
        }
        // The rev ribbon burns at half strength with a full-strength head.
        const { assert!(RIBBON_LEVEL <= 0.6 && CAUSTIC_SHARE < 0.2) };
        // Labels are at least 22 px tall.
        assert!((t.type_scale.unit * 1.2).max(22.0) >= 22.0);
    }

    #[test]
    fn the_upshift_ripple_leaves_the_numeral_still() {
        // Snell's displacement of what lies under a slope θ, through the depth of the dome at r.
        let eta = Dispersion::SAPPHIRE.primaries()[1];
        for g in VIEWS {
            let scale = g.gear_r / COCKPIT.gear_r;
            let (wave, from) = (RIPPLE_WAVE * scale, g.gear_r * RIPPLE_FROM);
            let glyph = ((g.gear_cap * 0.28).powi(2) + (g.gear_cap * 0.5).powi(2)).sqrt();
            let mut strongest = 0.0_f32;
            for step in 0..=100 {
                let k = step as f32 / 100.0;
                let grow = (k / 0.15).min(1.0);
                let amplitude = RIPPLE_HEIGHT * scale * grow * grow * (3.0 - 2.0 * grow) * (1.0 - k);
                let radius = from + k * (g.gear_r + wave - from);
                for r in 0..g.gear_r as i32 {
                    let r = r as f32;
                    let tilt = ripple_slope(amplitude, wave, radius, r).atan();
                    let inside = (tilt.sin() / eta).asin();
                    let shift = (g.gear_rise + g.gear_base) * (tilt - inside).abs().tan();
                    if r <= glyph {
                        assert!(shift <= 1.5, "k {k}: the numeral moves {shift} px at r {r}");
                    } else {
                        strongest = strongest.max(tilt.abs());
                    }
                }
            }
            // Out in the rim the wave is plain to see.
            assert!(strongest.to_degrees() > 4.0, "{}", strongest.to_degrees());
        }
    }

    #[test]
    fn the_capsules_bud_out_of_the_gear_lens() {
        for g in VIEWS {
            for sign in [-1.0, 1.0] {
                // At the start each is a disc inside the lens.
                let start = g.side_rect(sign, 0.0);
                assert!((start.width() - start.height()).abs() < 1e-3);
                let reach = (start.center() - g.gear).length() + start.height() * 0.5;
                assert!(reach <= g.gear_r, "the bud starts inside the lens ({reach})");
                // At the end it is the capsule of the zone table.
                let end = g.side_rect(sign, 1.0);
                let (centre, size) = if sign < 0.0 {
                    (g.speed, g.speed_size)
                } else {
                    (g.info, g.info_size)
                };
                assert!((end.center() - centre).length() < 1e-3);
                assert!((end.size() - size).length() < 1e-3);
            }
        }
    }

    #[test]
    fn the_ribbon_runs_from_idle_to_the_limiter() {
        let sig = Signals {
            rpm: 900.0,
            idle_rpm: 900.0,
            max_rpm: 8700.0,
            ..Signals::default()
        };
        assert_eq!(LiquidGlass::rev_fraction(&sig, sig.rpm), 0.0);
        assert_eq!(LiquidGlass::rev_fraction(&sig, 8700.0), 1.0);
        assert!((LiquidGlass::rev_fraction(&sig, 4800.0) - 0.5).abs() < 1e-6);
        // Bad telemetry never reaches a shader.
        assert_eq!(LiquidGlass::rev_fraction(&sig, f32::NAN), 0.0);
        assert_eq!(LiquidGlass::rev_fraction(&Signals::default(), 0.0), 0.0);
    }
}
