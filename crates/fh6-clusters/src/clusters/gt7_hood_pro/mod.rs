//! GT7 Hood Pro (`ClusterTheme::Gt7HoodPro`), catalog row 3: the Modern Digital 2027 family's
//! pattern-setter and the default hood theme (hood, build #2).
//!
//! A frameless "gull-wing horizon" hung from one 1 px hairline. Two mirrored light ribbons
//! (7 → 24 px, power 1.6, primary → white core) climb from the wingtips to a crest above the
//! gear. Each wing is a circular arc whose tangent is level at the tip, so the pair reads as a
//! gull wing: flat outboard, rising steeply into a kinked crest. Idle sits at the tips and the
//! car's shift rpm at the crest.
//! - Rank 1: 11 converging diamonds (28 px, the centre 34–36 px) above the crest, and the gear
//!   (Saira Condensed 800) in an OLED-black squircle pod with a light-driven 1 px rim.
//! - Rank 2: the 3-cell tabular speed (Saira Condensed 600, no ghost 8s: see HANDOVER) under the pod.
//! - Rank 3: the twin ribbons, each led by a comet orb (the meniscus bead of
//!   `common::ribbon`, sized by the orb intensity law 0.35 + 0.65 × rpm / shift rpm). The left
//!   wing carries the rpm scale (Titillium Web 600, every 1000 rpm, ticks every 500); the right
//!   wing runs at 70% with ticks only, so the two never read as duplicate tachs.
//! - Rank 4: four slip capsules on the pod's corners (the pod reads as the car from above), the
//!   G puck on the right (`common::GBall`) and the THR / BRK capsules on the left.
//!
//! Signature moments:
//! - **Apex fuse:** as the revs build the two comet orbs race inward. When they come within
//!   2 r + 10 px a neck grows between them (the look of a k 10 px smooth union). At the car's
//!   shift rpm they meet on the crest and a steady 6 px blade in `shift` rises from the crest to
//!   the centre diamond, timed with the outside-in diamond fill.
//! - **Limiter:** only the diamonds strobe (10 Hz, 50% duty, `common::ShiftLights`); the blade,
//!   the fused orb and the pod rim hold a steady `shift`.
//! - **Gear change:** the blade splits and both comets drop back outward on the `ribbon` spring
//!   with feed-forward; the glyph does the tokens' 1.12 → 1.0 entry and a light sweeps across the
//!   pod rim (left to right on an upshift, right to left on a downshift).
//! - **Light:** lateral G swings the virtual light on every rim and shifts the three depth
//!   planes by 0 / 2 / 4 px (parallax).
//!
//! Brief: `spec.md`. Telemetry is native: rpm (current, idle, limiter), the shift rpm (limiter ×
//! the shift point, the OSD's `ShiftPointPct` when set), gear, speed, throttle, brake, smoothed
//! acceleration (G puck, light) and body slip (from velocity). Tyre temperatures, slip ratios
//! and lap data are not decoded by `fh6-core` (and not in `tests/golden/packets`), so the slip
//! capsules are driven by body slip (rears β, fronts β / 2) and the lap line is omitted.
//!
//! # Zone table (local 4K px around the anchor; hood anchor (1920, 1760))
//!
//! | Zone | Element | Hood | Cockpit |
//! |---|---|---|---|
//! | Z1 | 11 diamonds | y −200, 28 px (centre 34), pitch 72 | y −262, 28 px (centre 36), pitch 72 |
//! | Z1 | gear pod (squircle n 4) | 232 × 184 at (0, −22), gear cap 140 | 250 × 210 at (0, −50), cap 150 |
//! | Z2 | speed (3 cells) | cap 80 at (0, 124) | cap 96 at (0, 128) |
//! | Z3 | gull-wing ribbons | tips (±820, −60), crest (0, −146), 7 → 24 px | tips (±680, −80), crest (0, −206) |
//! | Z3 | apex blade | 6 px, crest → centre diamond | same |
//! | Z4 | slip capsules | 18 × 48 at (±131, −22 ± 46) | 20 × 56 at (±142, −50 ± 56) |
//! | Z4 | G puck · THR/BRK | r 58 at (620, 72) · x −656 / −584, y 14..130 | r 66 at (540, 70), readout under it · x −576 / −504 |
//! | Z5 | rpm numerals | Titillium 600 cap 22 under the left wing | cap 24 |
use crate::common::gball::GBall;
use crate::common::glass_plate::{squircle, squircle_point};
use crate::common::hero_gear::HeroGear;
use crate::common::ribbon::{PathRibbon, RibbonStyle};
use crate::common::shift::LimiterCue;
use crate::common::speed::TabularSpeed;
use crate::common::tyre_slip::{Corner, SlipOutline, TyreSlipQuad, TyreStep};
use crate::common::{ShiftLights, VirtualLight};
use core::f32::consts::TAU;
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, BlendMode, Bloom, Cap, Cluster, Color, FontId, FontRole, FrameInput, Gradient, ImageId,
    ImagePaint, ImageWrap, LayerOptions, Layout, Painter, Palette, Rect, Shadow, Shape, Signals, Smoothed, Spring,
    Stroke, TextStyle, Tokens, VAlign, Vec2, View,
};

/// The diamond strip (`modern11`).
const DIAMONDS: u32 = 11;
/// Pod exponent (squircle, Apple's continuous curvature).
const POD_N: f32 = 4.0;
/// Reach of the apex fuse's "smooth union" (px past the two orb radii).
const FUSE_K: f32 = 10.0;
/// The ribbons stop just short of the crest, so the head colour never reaches the ribbon's
/// redline stop (the fused crest is drawn by the blade instead).
const FILL_MAX: f32 = 0.999;
/// Hairline alpha (46 of 255).
const HAIRLINE_ALPHA: f32 = 46.0 / 255.0;
/// The right wing runs at this brightness (ticks, no numerals).
const RIGHT_GAIN: f32 = 0.7;
/// The blade rises over this ramp once the shift rpm is reached (s).
const LOCK_RAMP_S: f32 = 0.06;
/// The pod-rim light sweep on a gear change (s).
const SWEEP_S: f32 = 0.2;
/// Half width of the sweeping highlight on the pod rim.
const SWEEP_HALF: f32 = 0.524; // 30°, a 60° streak
/// The diamonds' strobe off phase keeps 25% of the shift colour.
const STROBE_FLOOR: f32 = 0.25;
/// The diamonds hold full `shift` this long at the shift point before the strobe starts (s).
const STROBE_HOLD_S: f32 = 0.1;
/// Share of each wing (from the tip) that keeps its own colour under the lock; the rest turns
/// `shift`.
const LOCK_FROM: f32 = 0.55;
/// The Blender pod lens covers the pod plus a 2% margin on each side.
const POD_LENS_MARGIN: f32 = 1.04;
/// The ribbon grain tile in local px (stretched along the wing, so several filaments cross the
/// ribbon's width).
const GRAIN_TILE: Vec2 = vec2(640.0, 240.0);
/// Grain flow speed (local px/s): at idle, plus this much more at the shift rpm.
const GRAIN_FLOW: [f32; 2] = [80.0, 900.0];
/// Peak opacity of the additive grain at the ribbon head.
const GRAIN_ALPHA: f32 = 0.9;
/// How much the grain dims the ribbon body before adding its filaments (at the head).
const GRAIN_DIM: f32 = 0.38;
/// Opacity of the soft pads under the text groups (speed; on the hood also G and pedals).
const TEXT_PAD: f32 = 0.2;
/// Hood wing channel under the unlit span.
const CHANNEL_UNLIT: f32 = 0.1;
/// How far the full channel runs ahead of the comet (local px; longer leaves a dark knot ahead
/// of the orb where the two blurred layers meet).
const CHANNEL_LEAD: f32 = 10.0;
/// Hood: the unlit diamond lens, and the dark lens behind a lit diamond.
const DIAMOND_LENS: f32 = 0.2;
const LIT_LENS: f32 = 0.5;
/// Hood wing channel: extra width over the ribbon, blur and opacity.
const CHANNEL_PAD: f32 = 8.0;
const CHANNEL_BLUR: f32 = 4.0;
const CHANNEL_ALPHA: f32 = 0.38;
/// Hood keyline: opacity of the black copies, and their offset per unit of cap height (tokens
/// `type.keyline` 6 px at an 84 px cap).
const KEYLINE_ALPHA: f32 = 0.85;
const KEYLINE_PER_CAP: f32 = 0.07;
/// Hood: the slip capsules sit on a dark lens.
const HOOD_LENS: f32 = 0.5;
/// Width of the apex blade (px).
const BLADE_W: f32 = 8.0;
/// Smallest cap of the micro-labels (MPH, THR, BRK): the hood has no scrim behind them.
const MICRO_MIN_CAP: f32 = 18.0;
/// Most scale ticks kept (every 500 rpm from idle to the shift rpm).
const MAX_TICKS: usize = 48;
/// Parallax of the mid (pod, tyres, G) and front (gear, speed) planes at 1.5 g (px).
const PARALLAX_PX: [f32; 2] = [2.0, 4.0];
/// Samples of each wing hairline.
const HAIR_POINTS: usize = 65;
/// Segments per hairline polyline chunk (keeps each bounding box small).
const HAIR_CHUNK: usize = 8;

/// Geometry of one view (local 4K px around the anchor).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Geom {
    /// The left wingtip (the right one mirrors it).
    tip: Vec2,
    /// The crest, where the wings meet (x = 0).
    crest: Vec2,
    /// Ribbon width at the tip and at the crest.
    wing_w: [f32; 2],
    /// Comet orb radius at the shift rpm.
    bead_r: f32,
    diamond_y: f32,
    diamond_pitch: f32,
    diamond: f32,
    diamond_centre: f32,
    pod: Vec2,
    pod_size: Vec2,
    gear_cap: f32,
    speed: Vec2,
    speed_cap: f32,
    /// Slip capsule size, the |x| of its centre and the half distance between the front and
    /// rear centres.
    tyre: Vec2,
    tyre_dx: f32,
    tyre_dy: f32,
    gball: Vec2,
    gball_r: f32,
    /// The G readout's baseline centre (beside the puck on the hood, under it in the cockpit).
    gread: Vec2,
    /// THR and BRK capsule centre x, the capsules' top and bottom y and their width.
    pedal_x: [f32; 2],
    pedal_y: [f32; 2],
    pedal_w: f32,
    numeral_cap: f32,
    /// Hood: black keylines on the speed digits (no scrim behind the cluster).
    keyline: bool,
}

const HOOD: Geom = Geom {
    tip: vec2(-820.0, -60.0),
    crest: vec2(0.0, -146.0),
    wing_w: [7.0, 24.0],
    bead_r: 13.0,
    diamond_y: -200.0,
    diamond_pitch: 72.0,
    diamond: 28.0,
    diamond_centre: 34.0,
    pod: vec2(0.0, -22.0),
    pod_size: vec2(232.0, 184.0),
    gear_cap: 140.0,
    speed: vec2(0.0, 124.0),
    speed_cap: 80.0,
    tyre: vec2(18.0, 48.0),
    tyre_dx: 131.0,
    tyre_dy: 46.0,
    gball: vec2(620.0, 72.0),
    gball_r: 58.0,
    gread: vec2(752.0, 87.0),
    pedal_x: [-656.0, -584.0],
    pedal_y: [14.0, 130.0],
    pedal_w: 12.0,
    numeral_cap: 22.0,
    keyline: true,
};

const COCKPIT: Geom = Geom {
    tip: vec2(-680.0, -80.0),
    crest: vec2(0.0, -206.0),
    wing_w: [7.0, 24.0],
    bead_r: 15.0,
    diamond_y: -262.0,
    diamond_pitch: 72.0,
    diamond: 28.0,
    diamond_centre: 36.0,
    pod: vec2(0.0, -50.0),
    pod_size: vec2(250.0, 210.0),
    gear_cap: 150.0,
    speed: vec2(0.0, 128.0),
    speed_cap: 96.0,
    tyre: vec2(20.0, 56.0),
    tyre_dx: 142.0,
    tyre_dy: 46.0,
    gball: vec2(540.0, 70.0),
    gball_r: 66.0,
    gread: vec2(540.0, 194.0),
    pedal_x: [-576.0, -504.0],
    pedal_y: [10.0, 136.0],
    pedal_w: 14.0,
    numeral_cap: 24.0,
    keyline: false,
};

/// One wing as a circular arc: centre, radius, start angle and sweep (radians, clockwise from
/// 3 o'clock), running from the tip to the crest.
#[derive(Debug, Clone, Copy, PartialEq)]
struct WingArc {
    center: Vec2,
    radius: f32,
    start: f32,
    sweep: f32,
}

impl Geom {
    /// The wing arc on the `side` (-1 left, +1 right): level at the tip, through the crest.
    fn wing(&self, side: f32) -> WingArc {
        let tip = vec2(self.tip.x.abs() * side, self.tip.y);
        let dx = (self.crest.x - tip.x).abs();
        let h = (tip.y - self.crest.y).max(1.0);
        let radius = (dx * dx + h * h) / (2.0 * h);
        let center = vec2(tip.x, tip.y - radius);
        let start = deg(90.0);
        let end = (self.crest.y - center.y).atan2(self.crest.x - center.x);
        WingArc {
            center,
            radius,
            start,
            sweep: end - start,
        }
    }

    /// The wing centre line's y at `x` (either side).
    #[cfg(test)]
    fn wing_y(&self, x: f32) -> f32 {
        let a = self.wing(if x < 0.0 { -1.0 } else { 1.0 });
        let dx = (x - a.center.x).abs().min(a.radius);
        a.center.y + (a.radius * a.radius - dx * dx).sqrt()
    }

    /// The ribbon width at `x` (the fill's width curve, by the share of the wing's span).
    #[cfg(test)]
    fn wing_width(&self, x: f32) -> f32 {
        let t = (1.0 - x.abs() / self.tip.x.abs().max(1.0)).clamp(0.0, 1.0);
        self.wing_w[0] + (self.wing_w[1] - self.wing_w[0]) * t.powf(1.6)
    }

    /// Diamond `i`'s centre and size (outermost first, left to right).
    fn diamond_rect(&self, i: usize) -> Rect {
        let mid = (DIAMONDS as usize - 1) / 2;
        let x = (i as f32 - mid as f32) * self.diamond_pitch;
        let s = if i == mid { self.diamond_centre } else { self.diamond };
        Rect::from_center_size(vec2(x, self.diamond_y), vec2(s, s))
    }

    /// The slip capsule of `corner`.
    fn tyre_rect(&self, corner: Corner) -> Rect {
        let left = matches!(corner, Corner::FrontLeft | Corner::RearLeft);
        let front = matches!(corner, Corner::FrontLeft | Corner::FrontRight);
        let x = if left { -self.tyre_dx } else { self.tyre_dx };
        let y = if front {
            self.pod.y - self.tyre_dy
        } else {
            self.pod.y + self.tyre_dy
        };
        Rect::from_center_size(vec2(x, y), self.tyre)
    }

    /// Numerals and ticks stay off the pod and its capsules (|x| below this).
    fn scale_clear_x(&self) -> f32 {
        self.tyre_dx + self.tyre.x * 0.5 + 24.0
    }

    /// The pod's outline y (top half) at `x`, for the clearance tests.
    #[cfg(test)]
    fn pod_top(&self, x: f32) -> f32 {
        let a = self.pod_size.x * 0.5;
        let b = self.pod_size.y * 0.5;
        let u = ((x - self.pod.x).abs() / a).min(1.0).powf(POD_N);
        self.pod.y - b * (1.0 - u).max(0.0).powf(1.0 / POD_N)
    }
}

fn geom(view: View) -> &'static Geom {
    match view {
        View::Hood => &HOOD,
        View::Cockpit | View::Chase => &COCKPIT,
    }
}

/// The faces this cluster sets (resolved once; a user display font replaces gear and speed).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    speed: FontId,
    metric: FontId,
    numeral: FontId,
    label: FontId,
    role_display: FontId,
}

/// One view's two wing ribbons (left, right).
#[derive(Debug, Clone, Copy, PartialEq)]
struct Wings {
    side: [PathRibbon; 2],
    /// The hairline under each wing, sampled once (drawn in short chunks: one arc of a
    /// 4000 px circle is evaluated over a full-screen quad).
    hair: [[Vec2; HAIR_POINTS]; 2],
}

impl Wings {
    fn new(tokens: &Tokens, g: &Geom) -> Self {
        let make = |side: f32, gain: f32| {
            let a = g.wing(side);
            PathRibbon::arch(
                tokens,
                a.center,
                a.radius,
                a.start,
                a.sweep,
                RibbonStyle {
                    width_start: g.wing_w[0],
                    width_end: g.wing_w[1],
                    power: 1.6,
                    redline: 0.0,
                    track_width: 0.0,
                    gain,
                },
            )
        };
        let side = [make(-1.0, 1.0), make(1.0, RIGHT_GAIN)];
        let mut hair = [[Vec2::ZERO; HAIR_POINTS]; 2];
        for (h, w) in hair.iter_mut().zip(side.iter()) {
            for (i, q) in h.iter_mut().enumerate() {
                *q = w.point_at(i as f32 / (HAIR_POINTS - 1) as f32);
            }
        }
        Self { side, hair }
    }
}

/// One scale tick: its fraction along a wing and its numeral (0 for a 500 rpm minor tick).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Tick {
    frac: f32,
    label: u8,
}

/// The GT7 Hood Pro cluster.
#[derive(Debug, Clone)]
pub struct Gt7HoodPro {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    hood: Wings,
    cockpit: Wings,
    ticks: [Tick; MAX_TICKS],
    n_ticks: usize,
    /// The (idle, shift) rpm the ticks were laid out for.
    tick_key: (f32, f32),
    light: VirtualLight,
    /// Smoothed felt lateral force (-1..1) for the parallax, and its target.
    parallax: Smoothed<f32>,
    parallax_target: f32,
    shift: ShiftLights,
    lock: LimiterCue,
    gear: HeroGear,
    speed: TabularSpeed,
    gball: GBall,
    tyres: TyreSlipQuad,
    tyre_fill: [Smoothed<Color>; 4],
    tyre_target: [Color; 4],
    throttle: Spring,
    brake: Spring,
    /// Seconds since the pod-rim light sweep started (negative: none running).
    sweep_t: f32,
    sweep_up: bool,
    reduced_flash: bool,
    /// The Blender pod lens (`assets/clusters/gt7_hood_pro/pod_lens.png`), when present.
    pod_lens: Option<ImageId>,
    /// The ComfyUI ribbon grain tile (`ribbon_grain.png`), when present.
    grain: Option<ImageId>,
    /// How far the grain has flowed (local px, wrapped to the tile width).
    grain_phase: f32,
    micro_cap: f32,
    tracking: f32,
}

impl Gt7HoodPro {
    /// Builds the cluster: faces and the two views' wing ribbons (everything is procedural).
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let palette = Palette::family(t, "modernDigital");
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 800, false).unwrap_or(display),
            speed: setup.fonts.find("Saira Condensed", 600, false).unwrap_or(display),
            metric: setup.fonts.find("Saira Condensed", 600, false).unwrap_or(display),
            numeral: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            label: setup.fonts.find("Titillium Web", 600, false).unwrap_or(text),
            role_display: display,
        };
        let clear = palette.roles.ink.with_alpha(2.0 / 255.0);
        let pod_lens = load_asset(setup, "pod_lens.png");
        let grain = load_asset(setup, "ribbon_grain.png");
        Self {
            faces,
            sig: Signals::default(),
            palette,
            view: View::Hood,
            hood: Wings::new(t, &HOOD),
            cockpit: Wings::new(t, &COCKPIT),
            ticks: [Tick::default(); MAX_TICKS],
            n_ticks: 0,
            tick_key: (-1.0, -1.0),
            light: VirtualLight::new(t),
            parallax: Smoothed::new(0.0, 0.08),
            parallax_target: 0.0,
            shift: ShiftLights::new(DIAMONDS),
            lock: LimiterCue::new(),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            gball: GBall::new(t),
            tyres: TyreSlipQuad::new(t),
            tyre_fill: [Smoothed::new(clear, 0.05); 4],
            tyre_target: [clear; 4],
            throttle: Spring::new(t.springs.snap, 0.0),
            brake: Spring::new(t.springs.snap, 0.0),
            sweep_t: -1.0,
            sweep_up: true,
            reduced_flash: false,
            pod_lens,
            grain,
            grain_phase: 0.0,
            micro_cap: t.type_scale.micro_label.max(MICRO_MIN_CAP),
            tracking: t.type_scale.label_tracking,
        }
    }

    fn wings(&self) -> &Wings {
        match self.view {
            View::Hood => &self.hood,
            View::Cockpit | View::Chase => &self.cockpit,
        }
    }

    fn wings_mut(&mut self) -> &mut Wings {
        match self.view {
            View::Hood => &mut self.hood,
            View::Cockpit | View::Chase => &mut self.cockpit,
        }
    }

    /// The wing fill: idle at the tips, the shift rpm at the crest (never past it).
    fn wing_target(sig: &Signals) -> f32 {
        let span = (sig.shift_rpm - sig.idle_rpm).max(1.0);
        ((sig.rpm - sig.idle_rpm) / span).clamp(0.0, 1.0) * FILL_MAX
    }

    /// Lays the scale out again when the car's idle or shift rpm changes (no allocation).
    fn layout_ticks(&mut self, idle: f32, shift: f32) {
        if (idle, shift) == self.tick_key {
            return;
        }
        self.tick_key = (idle, shift);
        self.n_ticks = 0;
        let span = shift - idle;
        if !(span.is_finite() && span > 500.0) {
            return;
        }
        let mut k = 1u32;
        while self.n_ticks < MAX_TICKS {
            let rpm = k as f32 * 500.0;
            k += 1;
            if rpm <= idle {
                continue;
            }
            if rpm >= shift {
                break;
            }
            let label = if k % 2 == 1 { (rpm / 1000.0) as u8 } else { 0 };
            self.ticks[self.n_ticks] = Tick {
                frac: (rpm - idle) / span * FILL_MAX,
                label,
            };
            self.n_ticks += 1;
        }
    }

    /// The colour a slip capsule fills to for `step` (alarm by exception: clear until hot).
    fn tyre_color(step: TyreStep, pal: &Palette) -> Color {
        match step {
            TyreStep::Cool | TyreStep::Optimal => pal.roles.ink.with_alpha(2.0 / 255.0),
            TyreStep::Hot => pal.roles.caution.fade(0.06),
            TyreStep::Critical => pal.roles.warning.fade(0.08),
        }
    }
}

impl Cluster for Gt7HoodPro {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        self.sig = sig;
        self.palette = *input.palette;
        let target = Self::wing_target(&sig);
        if input.view != self.view {
            self.view = input.view;
            for w in &mut self.wings_mut().side {
                w.snap(target);
            }
        }
        // A user display font (FontChoice.display differs from the role) replaces the numerals.
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.speed = input.fonts.display;
            self.faces.metric = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }

        let span = (sig.shift_rpm - sig.idle_rpm).max(1.0);
        let rate = sig.rpm_rate / span * FILL_MAX;
        for w in &mut self.wings_mut().side {
            w.update(target, rate, dt);
        }
        self.layout_ticks(sig.idle_rpm, sig.shift_rpm);

        self.light.update(sig.g_lat, dt);
        self.parallax_target = (-sig.g_lat / 1.5).clamp(-1.0, 1.0);
        self.parallax.update(self.parallax_target, dt);

        self.reduced_flash = input.reduced_flash;
        // The grain flows towards the crest, faster as the revs build.
        self.grain_phase = (self.grain_phase + dt * (GRAIN_FLOW[0] + GRAIN_FLOW[1] * target)) % GRAIN_TILE.x;
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.lock.update(self.shift.over_shift(), LOCK_RAMP_S, dt);
        self.gear.update(sig.gear, self.lock.held(), dt);
        if self.gear.since_change() == 0.0 {
            self.sweep_t = 0.0;
            self.sweep_up = self.gear.was_upshift();
        } else if self.sweep_t >= 0.0 {
            self.sweep_t += dt;
            if self.sweep_t > SWEEP_S {
                self.sweep_t = -1.0;
            }
        }
        self.speed.update(sig.speed);
        self.gball.update(&sig, dt);

        // Body slip (verified, from velocity) stands in for tyre slip: the rears carry β, the
        // fronts half of it; 8° / 18° of body slip reach the caution / warning outlines.
        let beta = if sig.slip_deg.is_finite() {
            sig.slip_deg.abs()
        } else {
            0.0
        };
        let (front, rear) = (beta / 200.0, beta / 100.0);
        self.tyres.update([front, front, rear, rear], None, dt);
        for c in Corner::ALL {
            let i = c.index();
            self.tyre_target[i] = Self::tyre_color(self.tyres.step(c), &self.palette);
            self.tyre_fill[i].update(self.tyre_target[i], dt);
        }
        self.throttle.update(sig.throttle, dt);
        self.brake.update(sig.brake, dt);
    }

    fn draw(&self, p: &mut Painter<'_>, layout: &Layout) {
        let g = geom(self.view);
        let pal = self.palette;
        let shift_px = self.parallax.value();

        // The feathered pads behind gear and speed go first, under the ribbons' bloom, so they
        // never cut a dark silhouette into it (alpha <= 0.55, the hood brief's 140).
        // The pod's pad hugs the pod (a 4 px spread, 8 px sigma: about a 24 px feather) instead of
        // a radial pool, so it never rises as a dark dome over the apex.
        p.shadow(
            &squircle(g.pod, g.pod_size, POD_N),
            Shadow {
                offset: Vec2::ZERO,
                sigma: 8.0,
                spread: 4.0,
                color: pal.roles.oled_black.with_alpha(0.45),
            },
        );
        // Soft rounded-rectangle pads (text box + 24 px, 32 px feather, 0.20, clipped to the
        // envelope) under the speed and, on the hood (no scrim, bright road and sky behind), under
        // the G and pedal groups, plus the soft dark channel under the wings.
        let env = layout.envelope;
        let half = g.speed_cap * 0.5;
        soft_pad(
            p,
            Rect::from_points(
                vec2(g.speed.x - g.speed_cap * 1.3, g.speed.y - half),
                vec2(g.speed.x + g.speed_cap * 2.1, g.speed.y + half),
            ),
            env,
            TEXT_PAD,
            &pal,
        );
        if g.keyline {
            let reach = g.gball_r * 1.06;
            soft_pad(
                p,
                Rect::from_points(g.gball - Vec2::splat(reach), vec2(g.gread.x + 64.0, g.gball.y + reach)),
                env,
                TEXT_PAD,
                &pal,
            );
            soft_pad(
                p,
                Rect::from_points(
                    vec2(g.pedal_x[0] - 36.0, g.pedal_y[0]),
                    vec2(g.pedal_x[1] + 36.0, g.pedal_y[1] + 32.0),
                ),
                env,
                TEXT_PAD,
                &pal,
            );
            self.draw_channel(p, &pal);
        }

        // Back plane (0 px): hairline, scale, ribbons, comets, the apex fuse and the diamonds.
        self.draw_scale(p, g, &pal);
        self.draw_wings(p, g, &pal);
        self.draw_diamonds(p, g, &pal);

        // Mid plane (2 px): pod, slip capsules, pedals and the G puck.
        p.scoped(|p| {
            p.translate(vec2(shift_px * PARALLAX_PX[0], 0.0));
            self.draw_pod(p, g, &pal);
            self.draw_tyres(p, g, &pal);
            self.draw_pedals(p, g, &pal);
            self.gball.draw_meter(p, g.gball, g.gball_r, &pal);
            self.gball
                .draw_readout(p, g.gread, &pal, self.faces.metric, self.faces.label);
        });

        // Front plane (4 px): gear and speed.
        p.scoped(|p| {
            p.translate(vec2(shift_px * PARALLAX_PX[1], 0.0));
            self.draw_gear(p, g, &pal);
            self.draw_speed(p, g, &pal);
        });
    }

    fn animating(&self) -> bool {
        self.wings().side.iter().any(PathRibbon::animating)
            || self.shift.animating()
            || self.lock.animating()
            || self.gear.animating()
            || self.gball.animating()
            || self.tyres.animating()
            || self.sweep_t >= 0.0
            || (self.grain.is_some() && self.wings().side[0].fraction() > 0.002)
            || (self.parallax.value() - self.parallax_target).abs() > 1e-3
            || !self.throttle.settled(self.sig.throttle, 1e-3)
            || !self.brake.settled(self.sig.brake, 1e-3)
            || (0..4).any(|i| {
                let (a, b) = (self.tyre_fill[i].value(), self.tyre_target[i]);
                (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() + (a.a - b.a).abs() > 2e-3
            })
    }
}

impl Gt7HoodPro {
    /// The 1 px hairline both wings hang from, the ticks under both wings (the right at 70%)
    /// and the rpm numerals under the left wing.
    fn draw_scale(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let px = p.device_px();
        let wings = self.wings();
        for hair in &wings.hair {
            let mut i = 0;
            while i < HAIR_POINTS - 1 {
                let end = (i + HAIR_CHUNK).min(HAIR_POINTS - 1);
                p.fill(
                    &Shape::Polyline {
                        points: &hair[i..=end],
                        width: px,
                        closed: false,
                    },
                    roles.ink.with_alpha(HAIRLINE_ALPHA),
                );
                i = end;
            }
        }
        let clear = g.scale_clear_x();
        let numeral = TextStyle::new(self.faces.numeral, g.numeral_cap)
            .paint(roles.ink_muted)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        for (k, w) in wings.side.iter().enumerate() {
            let gain = if k == 0 { 1.0 } else { RIGHT_GAIN };
            for tick in &self.ticks[..self.n_ticks] {
                let at = w.point_at(tick.frac);
                if at.x.abs() < clear {
                    continue;
                }
                let tan = w.tangent_at(tick.frac);
                let mut n = vec2(-tan.y, tan.x);
                if n.y < 0.0 {
                    n = -n;
                }
                let half = w.width_at(tick.frac) * 0.5;
                let (len, ink) = if tick.label > 0 {
                    (12.0, roles.ink_label)
                } else {
                    (7.0, roles.hairline)
                };
                let a = at + n * (half + 5.0);
                p.fill(&Shape::line(a, a + n * len, 2.0), ink.fade(gain));
                if k == 0 && tick.label > 0 {
                    let c = a + n * (len + 6.0 + g.numeral_cap * 0.6);
                    let cells = if tick.label >= 10 { 2 } else { 1 };
                    if g.keyline {
                        keyline_number(p, c, i64::from(tick.label), cells, &numeral, roles.oled_black);
                    }
                    p.number(c, i64::from(tick.label), cells, &numeral, None);
                }
            }
        }
    }

    /// The twin ribbons, the comet orbs, the apex fuse and the shift blade.
    fn draw_wings(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let wings = self.wings();
        let lock = self.lock.level();
        let ribbons = LayerOptions {
            bloom: Some(Bloom {
                threshold: 0.3,
                intensity: 0.35,
            }),
            ..LayerOptions::default()
        };
        // The drawn fill never passes FILL_MAX: the ribbon spring may overshoot its target, and
        // the last stop of the ribbon's colour ramp is the warning role.
        let fill = [
            wings.side[0].fraction().min(FILL_MAX),
            wings.side[1].fraction().min(FILL_MAX),
        ];
        p.layer(ribbons, |p| {
            for (w, &f) in wings.side.iter().zip(fill.iter()) {
                // Under the lock the white-hot inner fill retracts to LOCK_FROM, and the shift-tinted
                // arch takes its place (drawn after the cores, in its own bloom).
                let e = lock * (2.0 - lock);
                let shown = if f > LOCK_FROM { f + (LOCK_FROM - f) * e } else { f };
                w.draw_fill(p, shown, pal, 0.0);
            }
        });

        // Comet orbs: size and glow follow the orb intensity law; while locked they shrink so
        // the shift-coloured arch, not a white core, carries the moment.
        let sig = &self.sig;
        let intensity = 0.35 + 0.65 * (sig.rpm / sig.shift_rpm.max(1.0)).clamp(0.0, 1.0);
        let r = g.bead_r * (0.25 + 0.75 * intensity) * (1.0 - 0.35 * lock);
        let core = r * (0.55 + 0.25 * self.throttle.value().clamp(0.0, 1.0));
        let light = Vec2::polar(Vec2::ZERO, 1.0, self.light.angle());
        let heads = [wings.side[0].point_at(fill[0]), wings.side[1].point_at(fill[1])];
        // Apex fuse: a neck grows between the orbs once they are within 2 r + k of each other.
        let d = (heads[1] - heads[0]).length();
        let reach = 2.0 * r + FUSE_K;
        let u = ((reach - d) / reach).clamp(0.0, 1.0);
        let u = u * u * (3.0 - 2.0 * u);
        if u > 0.001 {
            p.layer(LayerOptions::bloom(), |p| {
                let w = r * (0.5 + 0.9 * u);
                p.fill(
                    &Shape::Line {
                        a: heads[0],
                        b: heads[1],
                        width_a: w,
                        width_b: w,
                        cap: Cap::Round,
                    },
                    roles.ink.lerp(pal.primary, 0.35).fade(u),
                );
            });
        }
        for (k, w) in wings.side.iter().enumerate() {
            let gain = if k == 0 { 1.0 } else { RIGHT_GAIN };
            w.draw_bead(p, fill[k].max(0.004), r, core, light, pal, intensity * gain, 0.0);
        }
        // The lock tint goes over the fill, the cores and their bloom (outside any bloom
        // layer), so the inner arch reads as `shift`, not as a white-hot core.
        if lock > 0.004 {
            p.layer(ribbons, |p| {
                for (w, &f) in wings.side.iter().zip(fill.iter()) {
                    Self::draw_arch_lock(p, w, f, lock, pal);
                }
            });
        }

        // The lock: a steady blade in `shift` from the crest to the centre diamond, and the
        // fused orb turning to the shift colour (never strobing).
        if lock > 0.004 {
            let e = 1.0 - (1.0 - lock) * (1.0 - lock);
            // The fused orb sits between the two heads (they lag the rpm on the ribbon spring) and
            // only shows once the neck has joined them.
            let mid = (heads[0] + heads[1]) * 0.5;
            let orb = lock * u;
            let bottom = mid.y.min(g.crest.y + 8.0);
            let h = (bottom - g.diamond_y) * e;
            p.layer(LayerOptions::bloom(), |p| {
                p.fill(
                    &Shape::round_rect(
                        Rect::from_xywh(g.crest.x - BLADE_W * 0.5, bottom - h, BLADE_W, h),
                        BLADE_W * 0.5,
                    ),
                    Gradient::linear(vec2(0.0, bottom), vec2(0.0, bottom - h))
                        .stop(0.0, pal.shift.lerp(roles.ink, 0.3).fade(lock))
                        .stop(1.0, pal.shift.fade(lock)),
                );
                p.fill(
                    &Shape::circle(mid, r * 1.05),
                    Gradient::radial(mid, r * 1.05)
                        .stop(0.0, roles.ink.fade(orb))
                        .stop(0.45, pal.shift.lerp(roles.ink, 0.25).fade(orb))
                        .stop(1.0, pal.shift.fade(0.0)),
                );
            });
        }
        // The ComfyUI light grain flowing through both ribbons towards the crest (last, so it
        // shows over the lock tint too).
        if let Some(grain) = self.grain {
            for (k, (w, &f)) in wings.side.iter().zip(fill.iter()).enumerate() {
                self.draw_grain(p, w, f, grain, k, lock, pal);
            }
        }
    }

    /// The colour of diamond `i` this frame: the shared outside-in fill, and on the limiter
    /// the strobe between the full shift colour and 25% (the strip never vanishes).
    ///
    /// The strobe runs on the lock's own clock: the strip holds full `shift` for
    /// [`STROBE_HOLD_S`] when the shift point is reached, then strobes at 10 Hz, 50% duty, so
    /// the shift instruction always lands as a flash-on, never as a dimming. Reduced-flash mode
    /// keeps the shared 2 Hz breath.
    fn diamond_color(&self, i: usize, pal: &Palette) -> Option<Color> {
        if !self.shift.over_shift() || self.reduced_flash {
            return self.shift.color_of(i, pal);
        }
        let t = self.lock.held() - STROBE_HOLD_S;
        let on = t < 0.0 || (t * 10.0).fract() < 0.5;
        Some(if on { pal.shift } else { pal.shift.fade(STROBE_FLOOR) })
    }

    /// The 11 converging diamonds; only they strobe at the limiter. A lit diamond is light, not
    /// a plate: an additive Gaussian-like halo with no edge, then a bloomed core with a 30% white
    /// centre and a 1 px inner rim. Unlit diamonds are near-black lenses with a faint rim.
    fn draw_diamonds(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let mut pts = [[Vec2::ZERO; 4]; DIAMONDS as usize];
        for (i, q) in pts.iter_mut().enumerate() {
            let r = g.diamond_rect(i);
            let (c, h) = (r.center(), r.width() * 0.5);
            *q = [
                vec2(c.x, c.y - h),
                vec2(c.x + h, c.y),
                vec2(c.x, c.y + h),
                vec2(c.x - h, c.y),
            ];
        }
        for (i, q) in pts.iter().enumerate() {
            if self.diamond_color(i, pal).is_none() {
                let shape = Shape::Polygon { points: q };
                let lens = if g.keyline {
                    roles.oled_black.with_alpha(DIAMOND_LENS)
                } else {
                    roles.ink.with_alpha(1.0 / 255.0)
                };
                p.fill(&shape, lens);
                p.stroke(&shape, roles.ink.with_alpha(0.12), Stroke::rim_px(1.0));
            }
        }
        // Hood: a dark lens behind each lit diamond (1.3x, 0.5 black), so additive light still
        // reads over a bright sky.
        if g.keyline {
            for (i, q) in pts.iter().enumerate() {
                if self.diamond_color(i, pal).is_some() {
                    let c = g.diamond_rect(i).center();
                    let mut big = [Vec2::ZERO; 4];
                    for (b, v) in big.iter_mut().zip(q.iter()) {
                        *b = c + (*v - c) * 1.3;
                    }
                    p.fill(&Shape::Polygon { points: &big }, roles.oled_black.with_alpha(LIT_LENS));
                }
            }
        }
        p.scoped(|p| {
            p.set_blend(BlendMode::Add);
            for i in 0..DIAMONDS as usize {
                if let Some(c) = self.diamond_color(i, pal) {
                    let r = g.diamond_rect(i);
                    let (center, rad) = (r.center(), r.width() * 1.15);
                    p.fill(
                        &Shape::circle(center, rad),
                        Gradient::radial(center, rad)
                            .stop(0.0, c.fade(0.35))
                            .stop(0.22, c.fade(0.12))
                            .stop(0.45, c.fade(0.03))
                            .stop(0.7, c.fade(0.006))
                            .stop(1.0, c.fade(0.0)),
                    );
                }
            }
        });
        p.layer(LayerOptions::bloom(), |p| {
            for (i, q) in pts.iter().enumerate() {
                if let Some(c) = self.diamond_color(i, pal) {
                    let r = g.diamond_rect(i);
                    let shape = Shape::Polygon { points: q };
                    p.fill(
                        &shape,
                        Gradient::radial(r.center(), r.width() * 0.5)
                            .stop(0.0, c.lerp(roles.ink.with_alpha(c.a), 0.3))
                            .stop(0.6, c)
                            .stop(1.0, c),
                    );
                }
            }
        });
        for (i, q) in pts.iter().enumerate() {
            if let Some(c) = self.diamond_color(i, pal) {
                p.stroke(
                    &Shape::Polygon { points: q },
                    roles.ink.fade(0.4 * c.a),
                    Stroke::inner(1.0),
                );
            }
        }
    }

    /// Light segments along the pod's squircle rim, centred on outline angle `mid` with half
    /// width `half` (radians): a tapered streak in `color`, 0 at both ends.
    #[allow(clippy::too_many_arguments)]
    fn rim_streak(p: &mut Painter<'_>, g: &Geom, mid: f32, half: f32, width: f32, color: Color, alpha: f32) {
        const STEPS: usize = 10;
        let mut prev = squircle_point(g.pod, g.pod_size, POD_N, mid - half, 0.0);
        for i in 1..=STEPS {
            let a = mid - half + 2.0 * half * i as f32 / STEPS as f32;
            let q = squircle_point(g.pod, g.pod_size, POD_N, a, 0.0);
            let k = (i as f32 - 0.5) / STEPS as f32;
            let taper = (k * core::f32::consts::PI).sin();
            p.fill(
                &Shape::Line {
                    a: prev,
                    b: q,
                    width_a: width,
                    width_b: width,
                    cap: Cap::Butt,
                },
                color.with_alpha(alpha * taper * taper),
            );
            prev = q;
        }
    }

    /// Inner arch at the lock: from 55% of the wing to its head the ribbon crossfades to `shift`
    /// at 85% (opaque round-capped segments: overlapping opaque caps never show as dots, and butt caps
    /// left anti-aliased seams).
    fn draw_arch_lock(p: &mut Painter<'_>, w: &PathRibbon, f: f32, lock: f32, pal: &Palette) {
        const FROM: f32 = LOCK_FROM;
        const STEPS: usize = 24;
        if f <= FROM {
            return;
        }
        let at = |k: usize| FROM + (f - FROM) * k as f32 / STEPS as f32;
        let tint = |t: f32| {
            let s = ((t - FROM) / 0.2).clamp(0.0, 1.0);
            let s = s * s * (3.0 - 2.0 * s);
            w.color_at(t, pal, 0.0).lerp(pal.shift, 0.85 * lock * s)
        };
        let mut prev = w.point_at(at(0));
        for k in 1..=STEPS {
            let (t0, t1) = (at(k - 1), at(k));
            let q = w.point_at(t1);
            p.fill(
                &Shape::Line {
                    a: prev,
                    b: q,
                    width_a: w.width_at(t0),
                    width_b: w.width_at(t1),
                    cap: Cap::Round,
                },
                Gradient::linear(prev, q).stop(0.0, tint(t0)).stop(1.0, tint(t1)),
            );
            prev = q;
        }
    }

    /// The OLED-black pod with its feathered dimming pad, light-driven rim, the steady limiter
    /// rim and the gear-change light sweep.
    fn draw_pod(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let lock = self.lock.level();
        let pod = squircle(g.pod, g.pod_size, POD_N);
        match self.pod_lens {
            // The Blender lens (black glass, rolled bevel, baked speculars) laid 1:1 over the pod;
            // the texture carries a 2% margin on each side.
            Some(lens) => {
                let rect = Rect::from_center_size(g.pod, g.pod_size * POD_LENS_MARGIN);
                p.fill(&pod, ImagePaint::new(lens, rect));
            }
            None => p.fill(&pod, roles.oled_black.with_alpha(0.92)),
        }
        let light = Vec2::polar(Vec2::ZERO, 1.0, self.light.angle());
        let reach = g.pod_size.length() * 0.5;
        let calm = 1.0 - lock;
        p.stroke(
            &pod,
            Gradient::linear(g.pod + light * reach, g.pod - light * reach)
                .stop(0.0, roles.rim_specular.with_alpha(0.6 * calm))
                .stop(0.45, pal.primary.with_alpha(0.24 * calm))
                .stop(1.0, pal.primary.with_alpha(0.07 * calm)),
            Stroke::rim_px(1.0),
        );
        // A specular crescent towards the virtual light (it swings with lateral G).
        let la = self.light.angle();
        Self::rim_streak(p, g, la, deg(28.0), 2.0, roles.rim_specular, 0.55 * calm);
        if lock > 0.004 {
            p.stroke(&pod, pal.shift.fade(0.85 * lock), Stroke::rim_px(1.5));
        }
        if self.sweep_t >= 0.0 {
            let k = (self.sweep_t / SWEEP_S).clamp(0.0, 1.0);
            let e = 1.0 - (1.0 - k) * (1.0 - k);
            // Over the top of the rim: 9 o'clock → 12 → 3 o'clock (reversed on a downshift).
            let along = if self.sweep_up { e } else { 1.0 - e };
            let a = deg(180.0) + deg(180.0) * along;
            let fade = (1.0 - k).min(k * 8.0).clamp(0.0, 1.0);
            Self::rim_streak(p, g, a, SWEEP_HALF, 3.0, roles.ink, fade);
        }
    }

    /// The four slip capsules on the pod's corners: clear lenses with a lit rim until a tyre is
    /// in trouble, then a step fill and an amber / red outline.
    fn draw_tyres(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let light = Vec2::polar(Vec2::ZERO, 1.0, self.light.angle());
        for c in Corner::ALL {
            let r = g.tyre_rect(c);
            let shape = Shape::round_rect(r, r.width() * 0.5);
            if g.keyline {
                p.fill(&shape, roles.oled_black.with_alpha(HOOD_LENS));
            }
            p.fill(&shape, self.tyre_fill[c.index()].value());
            let reach = r.height() * 0.5;
            p.stroke(
                &shape,
                Gradient::linear(r.center() + light * reach, r.center() - light * reach)
                    .stop(0.0, roles.rim_specular.fade(0.7))
                    .stop(0.5, roles.ink_label)
                    .stop(1.0, roles.hairline),
                Stroke::rim_px(1.0),
            );
            let outline = match self.tyres.outline(c) {
                SlipOutline::None => None,
                SlipOutline::Caution => Some(roles.caution),
                SlipOutline::Warning => Some(roles.warning),
            };
            if let Some(color) = outline {
                let gap = 5.0;
                p.stroke(
                    &Shape::round_rect(r.expand(gap), r.width() * 0.5 + gap),
                    color,
                    Stroke::new(2.5),
                );
            }
        }
    }

    /// THR (ink) and BRK (warning) capsules, filling from the bottom, with micro-labels.
    fn draw_pedals(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let label = TextStyle::new(self.faces.label, self.micro_cap)
            .paint(roles.ink.fade(0.8))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        let h = g.pedal_y[1] - g.pedal_y[0];
        let pedals = [
            (g.pedal_x[0], self.throttle.value(), roles.ink.fade(0.85), "THR"),
            (g.pedal_x[1], self.brake.value(), roles.warning.fade(0.9), "BRK"),
        ];
        for (x, v, color, name) in pedals {
            let track = Rect::from_xywh(x - g.pedal_w * 0.5, g.pedal_y[0], g.pedal_w, h);
            let radius = g.pedal_w * 0.5;
            p.fill(&Shape::round_rect(track, radius), roles.ink.with_alpha(1.0 / 255.0));
            p.stroke(&Shape::round_rect(track, radius), roles.hairline, Stroke::rim_px(1.0));
            let v = v.clamp(0.0, 1.0);
            if v > 0.005 {
                let fh = (h * v).max(g.pedal_w);
                let fill = Rect::from_xywh(track.min.x, track.max.y - fh, g.pedal_w, fh);
                p.fill(&Shape::round_rect(fill, radius), color);
            }
            let at = vec2(x, g.pedal_y[1] + 12.0);
            if g.keyline {
                keyline_text(p, at, name, &label, roles.oled_black);
            }
            p.text(at, name, &label);
        }
    }

    /// Rank 1: the gear in its pod (clipped, so the 1.12 entry never leaves it), blooming softly
    /// and a little more at the limiter.
    fn draw_gear(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let lock = self.lock.level();
        p.scoped(|p| {
            // Inset 16 px, so the 1.12 entry is cut inside the pod and never meets its rim.
            p.clip(
                Rect::from_center_size(g.pod, g.pod_size - Vec2::splat(32.0)),
                g.pod_size.y * 0.26,
            );
            self.gear.draw(p, g.pod, self.faces.gear, g.gear_cap, pal);
            let glow = self.gear.color(pal).fade(0.1 + 0.3 * lock + self.gear.flash() * 0.5);
            p.layer(LayerOptions::bloom(), |p| {
                self.gear.draw_with(p, g.pod, self.faces.gear, g.gear_cap, glow);
            });
        });
    }

    /// Rank 2: the tabular speed and its unit on the baseline; on the hood the
    /// digits get a black keyline and a feathered pad (the scrim is 0% there).
    fn draw_speed(&self, p: &mut Painter<'_>, g: &Geom, pal: &Palette) {
        let roles = pal.roles;
        let style = TextStyle::new(self.faces.speed, g.speed_cap)
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        let value = self.speed.value();
        // No ghost 8s: the kit sets low-alpha text far brighter than its linear alpha (even
        // 1/255 reads as a real 8 on OLED black), so the field is exactly as wide as the digits
        // and stays centred on the gear axis (see HANDOVER).
        let cells = speed_cells(value);
        if g.keyline {
            let key = style.paint(roles.oled_black.with_alpha(0.85));
            let r = g.speed_cap * 0.045;
            for i in 0..8 {
                let o = Vec2::polar(Vec2::ZERO, r, i as f32 * TAU / 8.0);
                p.number(g.speed + o, value, cells, &key, None);
            }
        }
        let cells = p.number(g.speed, value, cells, &style.paint(roles.ink), None);
        let unit = TextStyle::new(self.faces.label, self.micro_cap)
            .paint(roles.ink.fade(0.8))
            .tracking(self.tracking)
            .align(Align::Left, VAlign::Baseline);
        let at = vec2(cells.max.x + 14.0, g.speed.y + g.speed_cap * 0.5);
        if g.keyline {
            keyline_text(p, at, self.sig.unit.label(), &unit, roles.oled_black);
        }
        p.text(at, self.sig.unit.label(), &unit);
    }

    /// Hood: a soft dark channel under each wing (opaque round-capped segments in a blurred
    /// layer, so the joins never double up). Under the lit span, plus [`CHANNEL_LEAD`] past the comet, it
    /// sits at [`CHANNEL_ALPHA`] so the ribbon (and the `shift` arch at the lock) keeps its
    /// contrast over a bright sky; under the unlit span it is a faint [`CHANNEL_UNLIT`] trace,
    /// so the empty part of the tach never out-contrasts the lit part.
    fn draw_channel(&self, p: &mut Painter<'_>, pal: &Palette) {
        let wings = self.wings();
        for (lit, opacity) in [(true, CHANNEL_ALPHA), (false, CHANNEL_UNLIT)] {
            let opts = LayerOptions {
                blur: Some(CHANNEL_BLUR),
                opacity,
                ..LayerOptions::default()
            };
            p.layer(opts, |p| {
                for (hair, w) in wings.hair.iter().zip(wings.side.iter()) {
                    let edge = w.fraction().min(FILL_MAX) + CHANNEL_LEAD / w.length().max(1.0);
                    for i in 1..HAIR_POINTS {
                        let (t0, t1) = (
                            (i - 1) as f32 / (HAIR_POINTS - 1) as f32,
                            i as f32 / (HAIR_POINTS - 1) as f32,
                        );
                        if (t0 < edge) != lit {
                            continue;
                        }
                        p.fill(
                            &Shape::Line {
                                a: hair[i - 1],
                                b: hair[i],
                                width_a: w.width_at(t0) + CHANNEL_PAD,
                                width_b: w.width_at(t1) + CHANNEL_PAD,
                                cap: Cap::Round,
                            },
                            pal.roles.oled_black,
                        );
                    }
                }
            });
        }
    }
}

/// A soft rounded-rectangle pad under a text group: `rect` grown by 24 px, a 32 px feather
/// (Gaussian edge, no visible rim), black at `alpha`, clipped to the view's envelope so it
/// never bleeds past the cluster.
fn soft_pad(p: &mut Painter<'_>, rect: Rect, envelope: Rect, alpha: f32, pal: &Palette) {
    let r = rect.expand(24.0);
    p.scoped(|p| {
        p.clip(envelope, 0.0);
        p.shadow(
            &Shape::round_rect(r, r.height().min(r.width()) * 0.5),
            Shadow {
                offset: Vec2::ZERO,
                sigma: 16.0,
                spread: 0.0,
                color: pal.roles.oled_black.with_alpha(alpha),
            },
        );
    });
}

/// Hood keyline: eight black copies of a text run around `pos` (tokens `type.keyline`), so a
/// micro-label reads over a bright road or sky. The radius follows the cap height.
fn keyline_text(p: &mut Painter<'_>, pos: Vec2, text: &str, style: &TextStyle, black: Color) {
    let key = style.paint(black.with_alpha(KEYLINE_ALPHA));
    let r = (style.cap * KEYLINE_PER_CAP).max(1.5);
    for i in 0..8 {
        p.text(pos + Vec2::polar(Vec2::ZERO, r, i as f32 * TAU / 8.0), text, &key);
    }
}

/// [`keyline_text`] for a tabular number.
fn keyline_number(p: &mut Painter<'_>, pos: Vec2, value: i64, cells: u8, style: &TextStyle, black: Color) {
    let key = style.paint(black.with_alpha(KEYLINE_ALPHA));
    let r = (style.cap * KEYLINE_PER_CAP).max(1.5);
    for i in 0..8 {
        p.number(
            pos + Vec2::polar(Vec2::ZERO, r, i as f32 * TAU / 8.0),
            value,
            cells,
            &key,
            None,
        );
    }
}

/// Loads `name` from `assets/clusters/gt7_hood_pro/` when the assets folder is known and the
/// file exists (the cluster draws its procedural look otherwise).
fn load_asset(setup: &mut Setup<'_>, name: &str) -> Option<ImageId> {
    let path = setup.assets?.join("clusters").join("gt7_hood_pro").join(name);
    if !path.is_file() {
        return None;
    }
    setup.images.load_file(setup.device, setup.queue, &path, None).ok()
}

impl Gt7HoodPro {
    /// The light grain inside one wing's fill, so the ribbon reads as a tube of light filaments
    /// rather than a flat stroke: the tube body is first dimmed by [`GRAIN_DIM`], then the
    /// repeating tile adds its filaments back on top (both butt-capped, so nothing stacks into
    /// dots). It flows towards the crest (the left wing's grain moves right, the right wing's
    /// left) and is tinted by the palette; under the lock the inner arch's grain turns `shift`.
    #[allow(clippy::too_many_arguments)]
    fn draw_grain(
        &self,
        p: &mut Painter<'_>,
        w: &PathRibbon,
        f: f32,
        grain: ImageId,
        side: usize,
        lock: f32,
        pal: &Palette,
    ) {
        const STEPS: usize = 32;
        if f <= 0.002 {
            return;
        }
        let gain = if side == 0 { 1.0 } else { RIGHT_GAIN };
        let dir = if side == 0 { -1.0 } else { 1.0 };
        let tile = Rect::from_xywh(dir * self.grain_phase, 0.0, GRAIN_TILE.x, GRAIN_TILE.y);
        let mut prev = w.point_at(0.0);
        for k in 1..=STEPS {
            let (t0, t1) = (f * (k - 1) as f32 / STEPS as f32, f * k as f32 / STEPS as f32);
            let q = w.point_at(t1);
            p.fill(
                &Shape::Line {
                    a: prev,
                    b: q,
                    width_a: w.width_at(t0) * 0.8,
                    width_b: w.width_at(t1) * 0.8,
                    cap: Cap::Butt,
                },
                pal.roles
                    .oled_black
                    .with_alpha(GRAIN_DIM * (0.35 + 0.65 * t1 / f.max(1e-3))),
            );
            prev = q;
        }
        p.scoped(|p| {
            p.set_blend(BlendMode::Add);
            let mut prev = w.point_at(0.0);
            for k in 1..=STEPS {
                let (t0, t1) = (f * (k - 1) as f32 / STEPS as f32, f * k as f32 / STEPS as f32);
                let q = w.point_at(t1);
                let lit = pal.primary.lerp(pal.roles.ink, 0.45);
                let s = ((t1 - LOCK_FROM) / 0.2).clamp(0.0, 1.0) * lock;
                // Brighter towards the head, as the ribbon's own light is.
                let a = GRAIN_ALPHA * gain * (0.35 + 0.65 * t1 / f.max(1e-3));
                let mut paint = ImagePaint::new(grain, tile).tint(lit.lerp(pal.shift, s).fade(a));
                paint.wrap = ImageWrap::Repeat;
                p.fill(
                    &Shape::Line {
                        a: prev,
                        b: q,
                        width_a: w.width_at(t0) * 0.8,
                        width_b: w.width_at(t1) * 0.8,
                        cap: Cap::Butt,
                    },
                    paint,
                );
                prev = q;
            }
        });
    }
}

/// Digit cells for a speed (1–3; the readout clamps to 0..=999).
fn speed_cells(value: i64) -> u8 {
    match value {
        v if v >= 100 => 3,
        v if v >= 10 => 2,
        _ => 1,
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(Gt7HoodPro::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_stays_inside_the_view_envelopes() {
        let t = Tokens::default();
        for (g, env) in [(&COCKPIT, t.layout.cockpit_envelope), (&HOOD, t.layout.hood_envelope)] {
            let inside = |p: Vec2, what: &str| assert!(env.contains(p), "{what} {p:?} outside {env:?}");
            inside(g.tip - vec2(g.wing_w[0], 0.0), "left wingtip");
            inside(vec2(-g.tip.x + g.wing_w[0], g.tip.y), "right wingtip");
            for i in 0..DIAMONDS as usize {
                let r = g.diamond_rect(i);
                inside(r.min, "diamond");
                inside(r.max, "diamond");
            }
            let pod = Rect::from_center_size(g.pod, g.pod_size);
            inside(pod.min, "pod");
            inside(pod.max, "pod");
            inside(g.speed + vec2(0.0, g.speed_cap * 0.5), "speed baseline");
            inside(g.speed - vec2(0.0, g.speed_cap * 0.5), "speed cap top");
            // The G puck: crosshair 1.06 r, a ball held at the limit 1.18 r.
            inside(g.gball - Vec2::splat(g.gball_r * 1.18), "G puck");
            inside(g.gball + Vec2::splat(g.gball_r * 1.18), "G puck");
            inside(g.gread + vec2(64.0, 0.0), "G readout");
            inside(g.gread - vec2(64.0, 32.0), "G readout");
            inside(
                vec2(g.pedal_x[0] - g.pedal_w, g.pedal_y[1] + 12.0 + 16.0),
                "pedal label",
            );
            for c in Corner::ALL {
                let r = g.tyre_rect(c).expand(5.0);
                inside(r.min, "tyre");
                inside(r.max, "tyre");
            }
        }
    }

    #[test]
    fn wings_meet_at_the_crest_and_stay_clear_of_the_pod() {
        let t = Tokens::default();
        for g in [&COCKPIT, &HOOD] {
            let w = Wings::new(&t, g);
            for side in &w.side {
                assert!((side.point_at(1.0) - g.crest).length() < 1.0);
                assert!((side.point_at(0.0).y - g.tip.y).abs() < 1.0);
                // Level at the tip, rising towards the crest.
                assert!(side.tangent_at(0.0).y.abs() < 0.05);
            }
            // The full-size orb on the wing clears the pod and the front capsules by >= 8 px.
            let mut x = -g.tip.x.abs();
            while x <= g.tip.x.abs() {
                let bottom = g.wing_y(x) + (g.wing_width(x) * 0.5).max(g.bead_r);
                if x.abs() <= g.pod_size.x * 0.5 {
                    assert!(g.pod_top(x) - bottom >= 8.0, "pod at x {x}: {}", g.pod_top(x) - bottom);
                }
                for c in [Corner::FrontLeft, Corner::FrontRight] {
                    let r = g.tyre_rect(c).expand(5.0);
                    if x >= r.min.x && x <= r.max.x {
                        assert!(r.min.y - bottom >= 8.0, "capsule at x {x}");
                    }
                }
                x += 4.0;
            }
            // The blade has room between the crest ribbon and the centre diamond, and the strobing
            // strip stays <= 42 px tall.
            let top = g.crest.y - g.wing_w[1] * 0.5;
            let diamond_bottom = g.diamond_y + g.diamond_centre * 0.5;
            assert!(top - diamond_bottom >= 20.0);
            assert!(g.diamond_centre <= 42.0);
        }
    }

    #[test]
    fn the_scale_runs_idle_to_shift() {
        let sig = Signals {
            rpm: 900.0,
            idle_rpm: 900.0,
            shift_rpm: 8439.0,
            max_rpm: 8700.0,
            ..Signals::default()
        };
        assert_eq!(Gt7HoodPro::wing_target(&sig), 0.0);
        let at_shift = Signals { rpm: 8439.0, ..sig };
        assert!((Gt7HoodPro::wing_target(&at_shift) - FILL_MAX).abs() < 1e-6);
        let over = Signals { rpm: 8700.0, ..sig };
        assert!((Gt7HoodPro::wing_target(&over) - FILL_MAX).abs() < 1e-6);
    }
}
