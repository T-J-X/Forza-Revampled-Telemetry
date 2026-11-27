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
//! | Shift cabochons | 16 pills | x −450..450 [−525..525], 34 [30] above the rod | 46 × 18 [52 × 16] | dome 9 [8] / whole / 4 | rank 1: one light under each |
//! | Rev rod | arc band | through (±600, −214) and (0, −250) [(±760, −146), (0, −176)] | 22 [20] wide | half rod 11 [10] / whole / 5 | rank 3: the liquid-light ribbon |
//! | Bead | disc on the rod | the head of the ribbon | r 20 [17] | dome / whole / 8 | the meniscus; hot on the limiter |
//! | Gear lens | disc | (0, −50) [(0, −28)] | r 160 [116] | dome 72 [52] / whole / 24 [18] | rank 1: the gear, drawn at cap 156 [112] and magnified about 1.15 times; the ring light |
//! | Speed pill | pill | (−436, −50) [(−452, −28)] | 388 × 172 [372 × 148] | dome 30 [26] / whole / 20 [16] | rank 2: speed, cap 88 [76], and its unit |
//! | Info pill | pill | (436, −50) [(452, −28)] | as the speed pill | as the speed pill | power and boost, cap 40 [34] |
//! | G lens | disc | (0, 180) [(0, 138)] | r 44 [36] | dome r / 2 / whole / 12 | rank 4: the G light |
//! | Foot row | text | y 100, under the side pills | – | – | rpm readout (left), logo zone (right) |
//!
//! Draw order: a black core under each piece that holds numerals (so a bright cabin or bonnet
//! never washes them out), then every light, then all the glass in one layer.
//!
//! Signature moments:
//! - **Limiter:** only the cabochon strip strobes (10 Hz, `common::ShiftLights`), and the 16
//!   cabochons fuse into one bar of glass. The steady cue is the bead: its light turns white
//!   with a warning halo. The ribbon and the numerals keep their colours.
//! - **Upshift:** a ripple runs across the gear lens from its middle to its rim and bends the
//!   numeral for a moment, the lens swells (its rise grows by a seventh) and the ring light
//!   runs once round it.
//! - **Braking:** the side pills reach for the gear puck and, at 1 g, fuse with it through
//!   thin necks of glass, on the tokens' slosh spring.
//! - **Light:** one rig lights every piece. Lateral G swings it up to 25° and acceleration
//!   leans it 6° per g, so highlights travel across all the pieces together.
//! - **Show:** the glass grows in over 200 ms (its thickness, never its opacity) and the side
//!   pills bud out of the gear puck.
//!
//! Telemetry is native: rpm (current, idle, limiter), the tokens' shift point, gear, speed,
//! acceleration, body slip (from velocity), power and boost. Nothing is synthesised.
use crate::common::hero_gear::HeroGear;
use crate::common::label::{logo_zone, LogoVariant};
use crate::common::shift::LimiterCue;
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::{PI, TAU};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Bloom, Cap, Cluster, Environment, FontId, FontRole, FrameInput, Glow, Gradient,
    ImageId, ImagePaint, LayerOptions, Layout, Logo, OpticalGlass, Painter, Palette, Piece, Rect, Refraction, Ripple,
    Shape, Signals, Smoothed, Spring, Stroke, TextStyle, Tokens, Tween, VAlign, Vec2, View,
};

/// The 16 cabochons (catalog brief).
const PILLS: usize = 16;
/// Blend length of the gear puck and the side pills while they bud apart (px).
const BUD_SMOOTH: f32 = 22.0;
/// Blend length the cabochons reach on the limiter, where they fuse into one bar (px).
const FUSE_SMOOTH: f32 = 10.0;
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
const BRIDGE_G: (f32, f32) = (0.25, 1.0);
/// share by which the gear lens's rise grows at the peak of the upshift flex, and how long the
/// flex, the ripple and the ring light's lap take (s).
const FLEX: f32 = 0.15;
const FLEX_S: f32 = 0.42;
/// The upshift ripple on the gear lens: peak height and crest-to-crest distance (px). The
/// steepest slope is 2π × height / wavelength, under 5°: through 96 px of glass that moves the
/// numeral's edges by 3 px, a wobble that never costs the gear its shape.
const RIPPLE_HEIGHT: f32 = 0.5;
const RIPPLE_WAVE: f32 = 40.0;
/// How far the studio's light leans to the car's primary colour (0 white, 1 the pure hue): the
/// cabin's ambient lighting, mirrored in every rim.
const AMBIENT_TINT: f32 = 0.4;
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
    /// The gear puck is a dome: its rise at the middle, and the glass under its rim.
    gear_rise: f32,
    gear_base: f32,
    /// Centre of the info pill; the speed pill mirrors it.
    side: Vec2,
    side_size: Vec2,
    /// The side pills are shallow domes: the rise along their middle, and the glass under the rim.
    side_rise: f32,
    side_base: f32,
    speed_cap: f32,
    info_cap: f32,
    /// Half the distance between the two rows of the info pill.
    info_row: f32,
    lens: Vec2,
    lens_r: f32,
    /// The middle of the foot row under the side pills.
    foot_y: f32,
}

const COCKPIT: Geom = Geom {
    rod_half_span: 600.0,
    rod_end_y: -214.0,
    rod_top_y: -250.0,
    rod_half_width: 11.0,
    pill_half_span: 450.0,
    pill: vec2(46.0, 18.0),
    pill_rise: 34.0,
    bead_r: 20.0,
    gear: vec2(0.0, -50.0),
    gear_r: 160.0,
    gear_cap: 156.0,
    gear_rise: 72.0,
    gear_base: 24.0,
    side: vec2(436.0, -50.0),
    side_size: vec2(388.0, 172.0),
    side_rise: 30.0,
    side_base: 20.0,
    speed_cap: 88.0,
    info_cap: 40.0,
    info_row: 30.0,
    lens: vec2(0.0, 180.0),
    lens_r: 44.0,
    foot_y: 100.0,
};

/// Hood: the scrim is off, so the row is wider and lower and the pieces are a little thinner.
const HOOD: Geom = Geom {
    rod_half_span: 760.0,
    rod_end_y: -146.0,
    rod_top_y: -176.0,
    rod_half_width: 10.0,
    pill_half_span: 525.0,
    pill: vec2(52.0, 16.0),
    pill_rise: 30.0,
    bead_r: 17.0,
    gear: vec2(0.0, -28.0),
    gear_r: 116.0,
    gear_cap: 112.0,
    gear_rise: 52.0,
    gear_base: 18.0,
    side: vec2(452.0, -28.0),
    side_size: vec2(372.0, 148.0),
    side_rise: 26.0,
    side_base: 16.0,
    speed_cap: 76.0,
    info_cap: 34.0,
    info_row: 26.0,
    lens: vec2(0.0, 138.0),
    lens_r: 36.0,
    foot_y: 100.0,
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

    /// The side pill on the left (`sign` −1, speed) or the right (`sign` 1, info) when `shown`
    /// of the way out of the gear puck: it starts as a disc inside the puck and ends at rest.
    fn side_rect(&self, sign: f32, shown: f32) -> Rect {
        let k = shown.clamp(0.0, 1.0);
        let h = self.side_size.y;
        let from = vec2(sign * (self.gear_r - h * 0.5 - 2.0).max(0.0), self.gear.y);
        let to = vec2(sign * self.side.x, self.side.y);
        Rect::from_center_size(from.lerp(to, k), vec2(h + (self.side_size.x - h) * k, h))
    }
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
    micro_cap: f32,
    tracking: f32,
}

/// Loads one of this cluster's assets, when the assets folder is known and holds it.
fn load_asset(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join("liquid_glass").join(name);
    if !path.is_file() {
        return None;
    }
    setup.images.load_file(setup.device, setup.queue, &path, None).ok()
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
            speed: setup
                .fonts
                .find("Saira", 300, false)
                .or_else(|| setup.fonts.find("Saira", 200, false))
                .unwrap_or(display),
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
            micro_cap: t.type_scale.micro_label,
            tracking: t.type_scale.label_tracking,
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
        if self.gear.since_change() == 0.0 && self.gear.was_upshift() {
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
            glass.light[c] *= lean_to[c];
        }
        glass.refraction = Refraction::World;
        glass.aim(
            self.light.angle(),
            op.key_elevation,
            vec2(self.lean[0].value(), self.lean[1].value()),
        );
        glass.materialize = self.shown();
        self.glass = glass;

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
        p.fill(
            &Shape::arc(c, r, 2.0, start + sweep * redline, sweep * (1.0 - redline), Cap::Butt),
            roles.warning.fade(0.6),
        );
        // Graduations every 1000 rpm, like the marks on a thermometer: light where the rod is
        // empty, dark where the light has reached them. The rod and the bead magnify them.
        let span = (self.sig.max_rpm - self.sig.idle_rpm).max(1.0);
        let first = (self.sig.idle_rpm / 1000.0).floor() as i32 + 1;
        let graduation = |p: &mut Painter<'_>, i: usize, lit: bool| {
            let rpm = (first + i as i32) as f32 * 1000.0;
            let f = (rpm - self.sig.idle_rpm) / span;
            if !f.is_finite() || f <= 0.0 || f >= 1.0 || (f <= frac) != lit {
                return;
            }
            let a = start + sweep * f;
            let ink = if lit {
                roles.oled_black.with_alpha(0.6)
            } else {
                roles.ink.with_alpha(0.3)
            };
            p.fill(
                &Shape::line(
                    Vec2::polar(c, r - lane * 0.5, a),
                    Vec2::polar(c, r + lane * 0.5, a),
                    2.0,
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
                intensity: 0.4,
            }),
            ..LayerOptions::default()
        };
        p.layer(quiet, |p| {
            if frac > 0.002 {
                // With the tile the steady fill carries 76% of the light and the caustic the rest.
                let steady = if self.light_tile.is_some() { 0.76 } else { 1.0 };
                let cool = frac.min(redline);
                p.fill(
                    &Shape::arc(c, r, lane, start, sweep * cool, Cap::Round),
                    pal.primary.fade(steady),
                );
                if frac > redline {
                    p.fill(
                        &Shape::arc(c, r, lane, start + sweep * redline, sweep * (frac - redline), Cap::Butt),
                        roles.warning.fade(steady),
                    );
                }
                if let Some(tile) = self.light_tile {
                    let rect = Rect::from_xywh(-self.flow, g.rod_top_y - LIGHT_TILE * 0.5, LIGHT_TILE, LIGHT_TILE);
                    p.scoped(|p| {
                        p.set_blend(BlendMode::Add);
                        p.fill(
                            &Shape::arc(c, r, lane, start, sweep * frac, Cap::Round),
                            ImagePaint::new(tile, rect)
                                .tint(pal.primary.intensity(LIGHT_WHITE * (1.0 - steady)))
                                .repeat(),
                        );
                    });
                }
            }
            for i in 0..MAX_GRADUATIONS {
                graduation(p, i, true);
            }
            // The bead's light: the primary hue, turning white with a warning halo on the limiter.
            let level = self.bead_level();
            let at = g.rod_point(frac);
            let core = g.bead_r * 0.32;
            if heat > 0.01 {
                p.glow(
                    &Shape::circle(at, core),
                    roles.warning,
                    Glow {
                        sigma: g.bead_r * 0.6,
                        intensity: 0.8 * heat,
                        fill_inside: true,
                    },
                );
            }
            p.glow(
                &Shape::circle(at, core),
                pal.primary.lerp(roles.ink, heat),
                Glow {
                    sigma: g.bead_r * 0.3,
                    intensity: level,
                    fill_inside: true,
                },
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

    /// Rank 1: the gear under the clear middle of its dome, and the ring light further out, which
    /// the dome bends to its rim. An upshift sends a bright head once round the ring.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette, heat: f32) {
        let roles = pal.roles;
        let ring_r = g.gear_r * 0.8;
        p.fill(
            &Shape::ring(g.gear, ring_r, 4.0),
            pal.primary.fade(0.6 * self.bead_level() * self.shown()),
        );
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
            let unit = TextStyle::new(self.faces.label, self.micro_cap)
                .paint(roles.ink_label)
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

            // Power and boost: label, value and unit on one baseline per row.
            let ic = g.side_rect(1.0, shown).center();
            let k = g.info_cap / 40.0;
            let value = TextStyle::new(self.faces.metric, g.info_cap)
                .paint(roles.ink.fade(0.86))
                .align(Align::Right, VAlign::Baseline)
                .tabular();
            let small = TextStyle::new(self.faces.label, self.micro_cap * 0.85)
                .paint(roles.ink_label)
                .tracking(self.tracking)
                .align(Align::Left, VAlign::Baseline);
            let right = ic.x + 70.0 * k;
            let label_x = ic.x - 138.0 * k;
            let rows = [
                ic.y - g.info_row + g.info_cap * 0.5,
                ic.y + g.info_row + g.info_cap * 0.5,
            ];
            let hp = self.power_hp.value().round().clamp(0.0, 9999.0) as i64;
            p.number(vec2(right, rows[0]), hp, 4, &value, None);
            p.text(vec2(right + 10.0 * k, rows[0]), "HP", &small);
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
            p.text(vec2(right + 10.0 * k, rows[1]), "PSI", &small);
            p.text(vec2(label_x, rows[1]), "BOOST", &small);

            // The foot row: the rpm readout (to the nearest 10) under the speed pill, the car's
            // logo or the custom name under the info pill.
            let rpm_cap = self.micro_cap * 1.5;
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
            let foot = vec2(-g.side.x, g.foot_y);
            let cells = p.number(
                foot + vec2((5.0 * p.fonts().tabular_cell(&rpm_style) - label_w - 10.0) * 0.5, 0.0),
                rpm,
                5,
                &rpm_style,
                None,
            );
            p.text(vec2(cells.max.x + 10.0, g.foot_y + rpm_cap * 0.5), "RPM", &rpm_label);
            let zone = Rect::from_center_size(vec2(g.side.x, g.foot_y), vec2(220.0, 44.0));
            logo_zone(p, zone, &self.logo, self.faces.wide, LogoVariant::Full, pal);
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
        p.stroke(
            &Shape::circle(g.lens, g.lens_r * 0.5 / 3.0),
            roles.ink.with_alpha(0.12 * shown),
            Stroke::rim_px(1.0),
        );
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
            Piece::pill(g.side_rect(-1.0, shown)).dome(g.side_rise, g.side_base),
            Piece::pill(g.side_rect(1.0, shown)).dome(g.side_rise, g.side_base),
        ];
        // The upshift ripple: a ring wave runs from the middle of the gear puck out past its
        // rim and dies away as it goes. It bends the numeral for a moment, as a liquid would.
        let mut puck_glass = glass;
        if self.flex.running() {
            let k = self.flex.progress();
            let scale = g.gear_r / COCKPIT.gear_r;
            puck_glass.ripple = Ripple {
                center: g.gear,
                radius: k * (g.gear_r + RIPPLE_WAVE * scale),
                amplitude: RIPPLE_HEIGHT * scale * (1.0 - k),
                wavelength: RIPPLE_WAVE * scale,
            };
        }
        // Hard braking: the blend length grows until the side pills touch the puck.
        let reach = self.bridge.value().clamp(0.0, 1.25);
        let gap = (g.side.x - g.side_size.x * 0.5 - g.gear_r).max(0.0);
        let body_smooth = BUD_SMOOTH + (gap * BRIDGE_REACH - BUD_SMOOTH).max(0.0) * reach;
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
                    smooth: FUSE_SMOOTH * heat,
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
                        if i == 0 { puck_glass } else { glass },
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
    use fh6_kit::optics::{view_through, Dispersion, Profile};
    use fh6_kit::shape::blob_distance;

    const VIEWS: [&Geom; 2] = [&COCKPIT, &HOOD];

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
            inside(Rect::from_center_size(g.gear, Vec2::splat(g.gear_r * 2.0)), "gear puck");
            inside(g.side_rect(-1.0, 1.0), "speed pill");
            inside(g.side_rect(1.0, 1.0), "info pill");
            inside(Rect::from_center_size(g.lens, Vec2::splat(g.lens_r * 2.0)), "G lens");
            inside(
                Rect::from_center_size(vec2(g.side.x, g.foot_y), vec2(220.0, 44.0)),
                "logo zone",
            );
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
            // The bead clears the gear puck, the puck clears the side pills and the G lens.
            let puck_top = g.gear.y - g.gear_r;
            assert!(puck_top - (g.rod_y(0.0) + g.bead_r) >= 6.0);
            let side = g.side_rect(1.0, 1.0);
            // At rest the smooth union moves the puck and a side pill by under a pixel
            // (k ln(1 + e^(-gap / k))), so drawing them apart does not show.
            let gap = side.min.x - g.gear_r;
            let bulge = BUD_SMOOTH * (-gap / BUD_SMOOTH).exp().ln_1p();
            assert!(bulge < 0.75, "the side pills merge at rest by {bulge} px");
            // Under full braking the blend length passes gap / (2 ln 2) and they fuse.
            let pieces = [Piece::circle(g.gear, g.gear_r), Piece::pill(side)];
            let mid = vec2(g.gear_r + gap * 0.5, g.gear.y);
            assert!(blob_distance(&pieces, BUD_SMOOTH, mid) > 0.0, "apart at rest");
            assert!(
                blob_distance(&pieces, gap * BRIDGE_REACH, mid) < 0.0,
                "fused at full braking"
            );
            assert!((g.lens.y - g.lens_r) - (g.gear.y + g.gear_r) >= 12.0);
            // The foot row sits under the side pills and above the envelope's bottom.
            assert!(g.foot_y - 22.0 >= side.max.y + 6.0);
        }
    }

    #[test]
    fn the_numerals_sit_under_the_flat_faces() {
        for g in VIEWS {
            // The gear sits under the clear middle of its dome. Where the dome tilts less than
            // 22° it mirrors only the dark cone round the viewer (optics.md §7), so no
            // reflection crosses the glyph (Saira Condensed 700 is about 0.56 cap wide).
            let dome = Profile {
                thickness: g.gear_rise,
                bevel: g.gear_r,
                base: g.gear_base,
                exponent: 2.0,
            };
            let eta = Dispersion::SAPPHIRE.primaries()[1];
            let clear = (0..g.gear_r as i32)
                .rev()
                .map(|r| r as f32)
                .find(|r| dome.normal(g.gear_r - r, vec2(1.0, 0.0)).z >= deg(22.0).cos())
                .unwrap_or(0.0);
            // What the eye sees at the edge of the clear zone lies this far from the middle.
            let sight = view_through(&dome, g.gear_r - clear, vec2(1.0, 0.0), eta);
            let seen = clear + sight.offset.x;
            let half_diag = ((g.gear_cap * 0.28).powi(2) + (g.gear_cap * 0.5).powi(2)).sqrt();
            assert!(
                half_diag <= seen,
                "gear glyph {half_diag} under a clear zone that shows {seen} (radius {clear})"
            );
            // The dome magnifies: the glyph is seen 10% to 20% larger than it is drawn.
            let near = view_through(&dome, g.gear_r - 40.0, vec2(1.0, 0.0), eta);
            let magnified = 40.0 / (40.0 + near.offset.x);
            assert!((1.10..=1.20).contains(&magnified), "magnification {magnified}");
            // The side pills are shallow domes. Their numerals stay inside the band that
            // tilts less than 22°, so no reflection crosses them either.
            let half = g.side_size.y * 0.5;
            let pillow = Profile {
                thickness: g.side_rise,
                bevel: half,
                base: g.side_base,
                exponent: 2.0,
            };
            let clear = (0..half as i32)
                .rev()
                .map(|y| y as f32)
                .find(|y| pillow.normal(half - y, vec2(0.0, 1.0)).z >= deg(22.0).cos())
                .unwrap_or(0.0);
            let sight = view_through(&pillow, half - clear, vec2(0.0, 1.0), eta);
            let seen = clear + sight.offset.y;
            assert!(
                g.speed_cap * 0.5 <= seen,
                "speed cap {} under a clear band that shows {seen}",
                g.speed_cap
            );
            assert!(g.info_row + g.info_cap * 0.5 <= seen);
        }
    }

    #[test]
    fn the_side_pills_bud_out_of_the_puck() {
        for g in VIEWS {
            for sign in [-1.0, 1.0] {
                // At the start each is a disc inside the puck.
                let start = g.side_rect(sign, 0.0);
                assert!((start.width() - start.height()).abs() < 1e-3);
                let reach = (start.center() - g.gear).length() + start.height() * 0.5;
                assert!(reach <= g.gear_r, "the bud starts inside the puck ({reach})");
                // At the end it is the pill of the zone table.
                let end = g.side_rect(sign, 1.0);
                assert!((end.center() - vec2(sign * g.side.x, g.side.y)).length() < 1e-3);
                assert!((end.size() - g.side_size).length() < 1e-3);
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
