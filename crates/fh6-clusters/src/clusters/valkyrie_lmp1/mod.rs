//! Valkyrie LMP1 (`ClusterTheme::ValkyrieLmp1`), catalog row 13: Hypercar category, Race Dash
//! family, cockpit and hood, target `recreation` (the Aston Martin Valkyrie's wheel OLED).
//!
//! The real car has no binnacle: everything sits on one OLED screen in the carbon wheel, with ten
//! blue shift lights above it (1 + 8 + 1) and rotary dials beside it (`RESEARCH.md`). The dash
//! takes those parts, not the wheel's shape: a forged-carbon wheel face around a true-black
//! window, the ten lights across the top, a huge round rev counter numbered to 12 that fills the
//! window, the gear at its centre, a digital speed on its left, power and a 4 s engineer trace on
//! its right, the TC and ABS lamps under the gear, and a knurled encoder on each shoulder carrying
//! a data arc (pedals left, power share right).
//!
//! The wheel face is the Blender render (`assets/clusters/valkyrie_lmp1/wheel_{view}_k{0,1,2}.png`,
//! recipe `tools/asset-forge/src/recipes/valkyrie_lmp1.rs`): forged carbon under a clear coat,
//! the anodised window bezel, the LED partition and pod housings, the unlit LED caps and the
//! knurled encoders, one render per key-light azimuth (−135°, −160°, −110°) crossfaded by the
//! virtual light (lateral G swings it 25°). Without the files it draws a flat procedural face.
//!
//! Glance ranks:
//! 1. The gear (`race` Saira Condensed 800, cap 176) at the dial's centre, and the ten shift
//!    lights: 76 × 30 lenses, filling outside-in (the two shoulder singles first, then the eight's
//!    pairs onto the centre) in `primary`, all ten in `shift` at the shift point, and at the
//!    limiter only the strip strobes (10 Hz, `ShiftLights`).
//! 2. Tabular speed (cap 112, ghost 8s at 2%) left of the dial, its unit under it.
//! 3. The rev counter: a 240° arc of r 204 from 0 to the scale top (the car's limiter rounded up
//!    to the next thousand: 12 on an 11,100 rpm car), lit in `primary`, numbered every thousand.
//!    The band from the shift point to the scale top is `warning` at 55%; on the limiter it takes
//!    a steady `shift` and the lit arc turns ink, so only the strip flashes.
//! 4. Power (in `signals.power_unit`) and the engineer trace (throttle in `go`, brake in `warning`,
//!    steer in ink, the last 4 s; 3 s in the hood view); the encoders' arcs.
//! 5. The TC and ABS lamps (derived), the labels, the wheel face.
//!
//! Telemetry: rpm (current, limiter, the shift point), gear, speed, pedals, steer, power, lateral
//! and longitudinal G, body slip (from velocity) and drivetrain are decoded. Derived (labelled so
//! in `HANDOVER.md`): the TC and ABS lamps and the power share (of the highest power seen in this
//! car). Lap, sector and tyre data are not decoded, so nothing here pretends to show them.
//!
//! Zone table (local 4K px around the cockpit anchor (1920, 1835); the envelope is
//! x −720..720, y −320..240):
//!
//! | Zone | Element | Geometry |
//! |---|---|---|
//! | Z5 | wheel face | x ±560 at y −300, ±700 from −196 to 120, ±520 at 212 |
//! | Z5 | window | x ±440, y −214..192, r 28; opaque black |
//! | Z1 | partition + eight pods | x ±408, y −286..−240; pods 76 × 30 at x −350 + 100·i, y −263 |
//! | Z1 | shoulder singles | (±560, −263), 76 × 30 |
//! | Z3 | rev counter | centre (0, 14), r 204, width 16, from 150° over 240° clockwise |
//! | Z3 | numerals | r 148, cap 26 |
//! | Z1 | gear | cap middle (0, 16), cap 176 |
//! | Z2 | speed | cap middle (−341, 12), cap 112; unit cap top at y 78 |
//! | Z4 | power | right edge x 396, cap middle −64, cap 56 |
//! | Z4 | trace | x 252..416, y 18..132 |
//! | Z5 | lamps | 88 × 36 at (±58, 166) |
//! | Z4 | encoders | (±604, 4): knurl r 59..73, well r 92, data arc r 90 (brake r 78) |
//!
//! Hood: the same layout at [`HOOD_SCALE`] around the hood anchor; the wheel face is rendered at
//! that scale into the hood envelope.
use crate::common::hero_gear::HeroGear;
use crate::common::race::{AlarmColumn, AlarmLamp, AlarmSide};
use crate::common::speed::TabularSpeed;
use crate::common::{ShiftLights, VirtualLight};
use fh6_kit::cluster::Setup;
use fh6_kit::{
    deg, vec2, Align, Bloom, Cap, Cluster, Color, FontId, FontRole, FrameInput, Glow, Gradient, Hysteresis, ImageId,
    ImagePaint, LayerOptions, Layout, Painter, Palette, PowerUnit, Rect, Shape, Signals, Smoothed, Stroke, TextStyle,
    TickRing, Tokens, VAlign, Vec2, View,
};

/// Shift lights: one on each shoulder and eight in the centre partition.
const PODS: u32 = 10;
/// Scale of the layout in the hood view (the recipe renders the hood face at this scale).
const HOOD_SCALE: f32 = 0.7;
/// The wheel face's parallax at full lateral G (px). The data on the screen stays.
const PARALLAX_FACE: f32 = 4.0;
/// The Blender renders' key-light azimuths (degrees), in file order k0, k1, k2.
const BAKED_LIGHTS: [f32; 3] = [-135.0, -160.0, -110.0];
/// The renders' brightness on screen as a linear share (the studio render is lit for a
/// monitor; in the cockpit the static structure stays under 20% on screen).
const BAKED_GAIN: f32 = 0.55;
/// The rpm at or above this share of the limiter counts as on the limiter.
const LIMITER_PCT: f32 = 0.985;
/// The red band's opacity below the limiter.
const RED_BAND_ALPHA: f32 = 0.55;
/// The speed's ghost 8s as seen on screen.
const GHOST_8: f32 = 0.02;
/// Unlit lenses as seen on screen (the procedural face only; the render carries its own caps).
const UNLIT_LENS: f32 = 9.0 / 255.0;
/// The lit pods' share of the tokens' bloom intensity (an LED behind a lens).
const POD_BLOOM: f32 = 0.4;
/// Labels (RPM x1000, POWER, PEDALS) as seen on screen.
const LABEL_ALPHA: f32 = 0.47;
/// Units (KM/H, KW) as seen on screen.
const UNIT_ALPHA: f32 = 0.62;
/// Rev numerals.
const NUMERAL_ALPHA: f32 = 0.75;
/// Unlit lamp labels.
const LAMP_LABEL_ALPHA: f32 = 0.35;
/// Trace samples (64 points, 63 segments over 4 s).
const TRACE_POINTS: usize = 64;
/// The trace's span in the cockpit and in the hood (s).
const TRACE_S: [f32; 2] = [4.0, 3.0];
/// Segments per trace polyline chunk (keeps each bounding box small).
const TRACE_CHUNK: usize = 8;
/// Power smoothing (s) and the readout's hysteresis (display units).
const POWER_TAU: f32 = 0.10;
/// See [`POWER_TAU`].
const POWER_STEP: f32 = 2.0;
/// The power share counts only once the car has made this much power (W).
const PEAK_MIN_W: f32 = 1000.0;
/// Watts per mechanical horsepower.
const W_PER_HP: f32 = 745.7;
/// Numerals by thousand (no formatting in the draw path).
const NUMERALS: [&str; 21] = [
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
];

/// Derived lamp thresholds (as BoschDdu derives them; labelled derived in `HANDOVER.md`).
mod lamp {
    /// Minimum speed (m/s) before any slip or lock-up reading counts.
    pub const MIN_SPEED_MS: f32 = 4.0;
    /// TC: throttle above this.
    pub const TC_THROTTLE: f32 = 0.2;
    /// TC: body slip above this (deg) and growing faster than [`TC_SLIP_RATE`].
    pub const TC_SLIP_DEG: f32 = 3.0;
    /// TC: the slip onset rate (deg/s) that reads as power oversteer.
    pub const TC_SLIP_RATE: f32 = 8.0;
    /// ABS: brake above this.
    pub const ABS_BRAKE: f32 = 0.3;
    /// ABS: deceleration (g) above this reads as threshold braking.
    pub const ABS_DECEL_G: f32 = 0.85;
    /// Forza `DrivetrainType` 0 is front-wheel drive: no power oversteer to catch.
    pub const FWD: i32 = 0;
}

const LAMPS: [AlarmLamp; 2] = [AlarmLamp::TC, AlarmLamp::ABS];

/// Geometry (local 4K px around the cockpit anchor). See the zone table in the module docs.
mod g {
    use fh6_kit::{vec2, Rect, Vec2};

    /// The wheel face outline (the recipe's yoke), clockwise from the top left.
    pub const YOKE: [Vec2; 8] = [
        vec2(-560.0, -300.0),
        vec2(560.0, -300.0),
        vec2(700.0, -196.0),
        vec2(700.0, 120.0),
        vec2(520.0, 212.0),
        vec2(-520.0, 212.0),
        vec2(-700.0, 120.0),
        vec2(-700.0, -196.0),
    ];
    /// The window and its corner radius.
    pub const WINDOW: Rect = Rect {
        min: vec2(-440.0, -214.0),
        max: vec2(440.0, 192.0),
    };
    /// See [`WINDOW`].
    pub const WINDOW_R: f32 = 28.0;
    /// The anodised bezel's width round the window.
    pub const BEZEL: f32 = 6.0;
    /// The LED partition (the eight).
    pub const PARTITION: Rect = Rect {
        min: vec2(-408.0, -286.0),
        max: vec2(408.0, -240.0),
    };
    /// A pod lens.
    pub const POD: Vec2 = vec2(76.0, 30.0);
    /// The pods' centre line.
    pub const POD_Y: f32 = -263.0;
    /// The shoulder singles' x.
    pub const SINGLE_X: f32 = 560.0;
    /// A single's housing.
    pub const SINGLE_HOUSING: Vec2 = vec2(92.0, 46.0);
    /// The rev counter.
    pub const DIAL_C: Vec2 = vec2(0.0, 14.0);
    /// See [`DIAL_C`].
    pub const DIAL_R: f32 = 204.0;
    /// See [`DIAL_C`].
    pub const DIAL_W: f32 = 16.0;
    /// Start (radians clockwise from 3 o'clock: lower left) and sweep, in degrees.
    pub const DIAL_START: f32 = 150.0;
    /// See [`DIAL_START`].
    pub const DIAL_SWEEP: f32 = 240.0;
    /// The red band outside the arc.
    pub const RED_R: f32 = 220.0;
    /// See [`RED_R`].
    pub const RED_W: f32 = 5.0;
    /// Tick outer radius and lengths (major, minor).
    pub const TICK_R: f32 = 192.0;
    /// See [`TICK_R`].
    pub const TICK_MAJOR: f32 = 22.0;
    /// See [`TICK_R`].
    pub const TICK_MINOR: f32 = 12.0;
    /// Numerals.
    pub const NUMERAL_R: f32 = 148.0;
    /// See [`NUMERAL_R`].
    pub const NUMERAL_CAP: f32 = 26.0;
    /// The RPM x1000 label (cap middle).
    pub const RPM_LABEL: Vec2 = vec2(0.0, 126.0);
    /// Small labels.
    pub const LABEL_CAP: f32 = 16.0;
    /// The gear (cap middle).
    pub const GEAR: Vec2 = vec2(0.0, 16.0);
    /// See [`GEAR`].
    pub const GEAR_CAP: f32 = 176.0;
    /// The speed (cap middle) and its unit (cap top).
    pub const SPEED: Vec2 = vec2(-341.0, 12.0);
    /// See [`SPEED`].
    pub const SPEED_CAP: f32 = 112.0;
    /// See [`SPEED`].
    pub const UNIT: Vec2 = vec2(-330.0, 78.0);
    /// See [`SPEED`].
    pub const UNIT_CAP: f32 = 24.0;
    /// Power: right edge and cap middle of the value; the unit's left edge; the label (cap top).
    pub const POWER: Vec2 = vec2(396.0, -64.0);
    /// See [`POWER`].
    pub const POWER_CAP: f32 = 56.0;
    /// See [`POWER`].
    pub const POWER_UNIT: Vec2 = vec2(408.0, -46.0);
    /// See [`POWER`].
    pub const POWER_UNIT_CAP: f32 = 20.0;
    /// See [`POWER`].
    pub const POWER_LABEL: Vec2 = vec2(316.0, -128.0);
    /// The trace box.
    pub const TRACE: Rect = Rect {
        min: vec2(252.0, 18.0),
        max: vec2(416.0, 132.0),
    };
    /// The trace label (cap top).
    pub const TRACE_LABEL: Vec2 = vec2(334.0, 146.0);
    /// Lamps (centres) and lens size.
    pub const LAMPS: [Vec2; 2] = [vec2(-58.0, 166.0), vec2(58.0, 166.0)];
    /// See [`LAMPS`].
    pub const LAMP: Vec2 = vec2(88.0, 36.0);
    /// Encoder centres.
    pub const ENC: [Vec2; 2] = [vec2(-604.0, 4.0), vec2(604.0, 4.0)];
    /// Knurl ring radii (inner, outer) and the well's radius.
    pub const KNURL: [f32; 2] = [59.0, 73.0];
    /// See [`KNURL`].
    pub const WELL_R: f32 = 92.0;
    /// Data arcs: outer (throttle, power share) and inner (brake) radii, width, start and sweep
    /// (degrees, clockwise from 3 o'clock: from lower left round over the top to lower right).
    pub const ARC_R: [f32; 2] = [90.0, 78.0];
    /// See [`ARC_R`].
    pub const ARC_W: f32 = 6.0;
    /// See [`ARC_R`].
    pub const ARC_START: f32 = 135.0;
    /// See [`ARC_R`].
    pub const ARC_SWEEP: f32 = 270.0;
    /// Encoder labels (cap top, below the encoder).
    pub const ENC_LABEL_DY: f32 = 118.0;

    /// The ten pod centres in strip order: the left single, the eight, the right single.
    pub fn pods() -> [Vec2; 10] {
        let mut out = [Vec2::ZERO; 10];
        out[0] = vec2(-SINGLE_X, POD_Y);
        for (i, c) in out.iter_mut().enumerate().skip(1).take(8) {
            *c = vec2(-350.0 + 100.0 * (i - 1) as f32, POD_Y);
        }
        out[9] = vec2(SINGLE_X, POD_Y);
        out
    }

    /// The wheel face's bounds.
    pub fn yoke_bounds() -> Rect {
        Rect {
            min: vec2(-700.0, -300.0),
            max: vec2(700.0, 212.0),
        }
    }
}

/// Faces resolved at build time.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Faces {
    gear: FontId,
    digits: FontId,
    label: FontId,
    role_display: FontId,
}

/// The engineer trace: throttle, brake and steer (−1..1), sampled at a fixed rate into a ring.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Trace {
    /// `[throttle, brake, steer]` per sample.
    samples: [[f32; 3]; TRACE_POINTS],
    /// Index of the newest sample.
    head: usize,
    /// Time since the newest sample (s).
    since: f32,
}

impl Trace {
    fn new() -> Self {
        Self {
            samples: [[0.0; 3]; TRACE_POINTS],
            head: 0,
            since: 0.0,
        }
    }

    /// The sampling period: the cockpit span over the ring's segments.
    fn period() -> f32 {
        TRACE_S[0] / (TRACE_POINTS - 1) as f32
    }

    fn update(&mut self, sample: [f32; 3], dt: f32) {
        self.since += dt.max(0.0);
        let period = Self::period();
        // Catch up at most a full ring after a long frame.
        let mut steps = 0;
        while self.since >= period && steps < TRACE_POINTS {
            self.since -= period;
            self.head = (self.head + 1) % TRACE_POINTS;
            self.samples[self.head] = sample;
            steps += 1;
        }
        if steps == TRACE_POINTS {
            self.since = 0.0;
        }
        // The newest point follows the input between samples.
        self.samples[self.head] = sample;
    }

    /// Sample `k` back from the newest (0 = newest).
    fn back(&self, k: usize) -> [f32; 3] {
        self.samples[(self.head + TRACE_POINTS - k % TRACE_POINTS) % TRACE_POINTS]
    }

    /// True while any shown sample differs from the newest (the trace still scrolls).
    fn moving(&self) -> bool {
        let now = self.samples[self.head];
        self.samples
            .iter()
            .any(|s| (s[0] - now[0]).abs() + (s[1] - now[1]).abs() + (s[2] - now[2]).abs() > 1e-4)
    }
}

/// The TC and ABS states (derived).
fn lamp_conditions(sig: &Signals, drivetrain: i32, slip_rate: f32) -> [bool; 2] {
    let moving = sig.speed_ms > lamp::MIN_SPEED_MS;
    let slip = sig.slip_deg.abs();
    let tc = moving
        && drivetrain != lamp::FWD
        && sig.throttle > lamp::TC_THROTTLE
        && slip > lamp::TC_SLIP_DEG
        && slip_rate > lamp::TC_SLIP_RATE;
    let abs = moving && sig.brake > lamp::ABS_BRAKE && -sig.g_long >= lamp::ABS_DECEL_G;
    [tc, abs]
}

/// The scale top in thousands: the limiter rounded up (12 on an 11,100 rpm car).
fn scale_top(max_rpm: f32) -> u32 {
    let k = if max_rpm.is_finite() {
        (max_rpm / 1000.0).ceil()
    } else {
        8.0
    };
    // 4..=20 keeps the numerals table and the tick ring sane on odd data.
    (k.max(4.0) as u32).min(20)
}

/// A dial fraction's angle (radians).
fn dial_angle(t: f32) -> f32 {
    deg(g::DIAL_START + g::DIAL_SWEEP * t.clamp(0.0, 1.0))
}

/// The Valkyrie LMP1 cluster.
#[derive(Debug, Clone)]
pub struct ValkyrieLmp1 {
    faces: Faces,
    sig: Signals,
    palette: Palette,
    view: View,
    light: VirtualLight,
    shift: ShiftLights,
    gear: HeroGear,
    speed: TabularSpeed,
    power_w: Smoothed<f32>,
    power_shown: Hysteresis,
    /// The power readout's integer (from `power_shown`).
    power_value: i64,
    power_peak_w: f32,
    car: i32,
    lamps: AlarmColumn,
    prev_slip: f32,
    trace: Trace,
    /// The Blender wheel face per view (cockpit, hood), three light keys each, when all load.
    plates: [Option<[ImageId; 3]>; 2],
    envelopes: [Rect; 2],
    tracking: f32,
}

impl ValkyrieLmp1 {
    /// Builds the cluster: faces, components and the optional Blender wheel face.
    #[must_use]
    pub fn new(setup: &mut Setup<'_>) -> Self {
        let t: &Tokens = setup.tokens;
        let display = setup.fonts.role(FontRole::Display);
        let text = setup.fonts.role(FontRole::Text);
        let faces = Faces {
            gear: setup.fonts.find("Saira Condensed", 800, false).unwrap_or(display),
            digits: setup.fonts.find("Saira Condensed", 700, false).unwrap_or(display),
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
        let mut plate = |view: &str| match (
            load(&format!("clusters/valkyrie_lmp1/wheel_{view}_k0.png")),
            load(&format!("clusters/valkyrie_lmp1/wheel_{view}_k1.png")),
            load(&format!("clusters/valkyrie_lmp1/wheel_{view}_k2.png")),
        ) {
            (Some(a), Some(b), Some(c)) => Some([a, b, c]),
            _ => None,
        };
        let plates = [plate("cockpit"), plate("hood")];
        Self {
            faces,
            sig: Signals::default(),
            palette: Palette::family(t, "raceDash"),
            view: View::Cockpit,
            light: VirtualLight::new(t),
            shift: ShiftLights::new(PODS),
            gear: HeroGear::new(t),
            speed: TabularSpeed::new(),
            power_w: Smoothed::new(0.0, POWER_TAU),
            power_shown: Hysteresis::new(0.0, POWER_STEP),
            power_value: 0,
            power_peak_w: 0.0,
            car: 0,
            lamps: AlarmColumn::new(t, AlarmSide::Bottom, &LAMPS),
            prev_slip: 0.0,
            trace: Trace::new(),
            plates,
            envelopes: [t.layout.cockpit_envelope, t.layout.hood_envelope],
            tracking: t.type_scale.label_tracking,
        }
    }

    /// The wheel face's parallax (−1..1) from the light's swing.
    fn parallax(&self) -> f32 {
        ((self.light.angle() - deg(BAKED_LIGHTS[0])) / deg(25.0)).clamp(-1.0, 1.0)
    }

    fn view_index(&self) -> usize {
        usize::from(self.view == View::Hood)
    }

    fn on_limiter(&self) -> bool {
        self.sig.max_rpm > 1.0 && self.sig.rpm >= self.sig.max_rpm * LIMITER_PCT
    }

    /// The power share of the car's highest power seen (0..1).
    fn power_share(&self) -> f32 {
        if self.power_peak_w < PEAK_MIN_W {
            0.0
        } else {
            (self.power_w.value() / self.power_peak_w).clamp(0.0, 1.0)
        }
    }
}

impl Cluster for ValkyrieLmp1 {
    fn update(&mut self, input: &FrameInput<'_>) {
        let dt = input.dt.max(0.0);
        let sig = *input.signals;
        self.sig = sig;
        self.palette = *input.palette;
        self.view = input.view;
        if input.fonts.display != self.faces.role_display {
            self.faces.gear = input.fonts.display;
            self.faces.digits = input.fonts.display;
            self.faces.role_display = input.fonts.display;
        }

        self.light.update(sig.g_lat, dt);
        self.shift.update(&sig, &self.palette, input.t, dt, input.reduced_flash);
        self.gear.update(sig.gear, 0.0, dt);
        self.speed.update(sig.speed);

        // A new car starts a new power peak.
        if sig.car_ordinal != self.car {
            self.car = sig.car_ordinal;
            self.power_peak_w = 0.0;
        }
        let watts = input.telemetry.power_watts;
        let w = self
            .power_w
            .update(if watts.is_finite() { watts.max(0.0) } else { 0.0 }, dt);
        self.power_peak_w = self.power_peak_w.max(w);
        let shown = match sig.power_unit {
            PowerUnit::Hp => w / W_PER_HP,
            PowerUnit::Kw => w / 1000.0,
        };
        self.power_value = self.power_shown.update(shown).clamp(0, 9999);

        let slip = sig.slip_deg.abs();
        let slip_rate = if dt > 1e-4 { (slip - self.prev_slip) / dt } else { 0.0 };
        self.prev_slip = slip;
        self.lamps
            .update(&lamp_conditions(&sig, input.telemetry.drivetrain_type, slip_rate), dt);

        let steer = f32::from(input.telemetry.steer) / 127.0;
        self.trace.update(
            [
                sig.throttle.clamp(0.0, 1.0),
                sig.brake.clamp(0.0, 1.0),
                steer.clamp(-1.0, 1.0),
            ],
            dt,
        );
    }

    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = self.palette;
        let vi = self.view_index();
        let scale = if vi == 1 { HOOD_SCALE } else { 1.0 };
        let par = (-self.parallax() * PARALLAX_FACE * scale).round();
        // The wheel face, rendered at the view's scale into its envelope.
        p.scoped(|p| {
            p.translate(vec2(par, 0.0));
            match self.plates[vi] {
                Some(keys) => self.draw_plate(p, keys, self.envelopes[vi]),
                None => p.scoped(|p| {
                    p.scale(scale);
                    Self::draw_face(p, &pal);
                }),
            }
        });
        p.scoped(|p| {
            p.scale(scale);
            // The opaque black screen (the game never shows through it).
            p.fill(&Shape::round_rect(g::WINDOW, g::WINDOW_R), pal.roles.oled_black);
            self.draw_pods(p, &pal);
            self.draw_rev_counter(p, &pal);
            self.draw_readouts(p, &pal);
            self.draw_trace(p, &pal);
            self.draw_lamps(p, &pal);
            p.scoped(|p| {
                p.translate(vec2(par / scale, 0.0));
                self.draw_encoders(p, &pal);
            });
        });
    }

    fn animating(&self) -> bool {
        self.shift.animating() || self.gear.animating() || self.lamps.animating() || self.trace.moving()
    }
}

impl ValkyrieLmp1 {
    /// The Blender face: key k0 under the crossfaded swing key, dimmed to the cockpit level in
    /// the tint (no separate dimming fill).
    fn draw_plate(&self, p: &mut Painter<'_>, keys: [ImageId; 3], rect: Rect) {
        let light = self.light.angle();
        let (other, t) = if light < deg(BAKED_LIGHTS[0]) {
            (
                keys[1],
                (light - deg(BAKED_LIGHTS[0])) / deg(BAKED_LIGHTS[1] - BAKED_LIGHTS[0]),
            )
        } else {
            (
                keys[2],
                (light - deg(BAKED_LIGHTS[0])) / deg(BAKED_LIGHTS[2] - BAKED_LIGHTS[0]),
            )
        };
        let t = t.clamp(0.0, 1.0);
        let shape = Shape::rect(rect);
        let k = BAKED_GAIN;
        p.fill(&shape, ImagePaint::new(keys[0], rect).tint(Color::linear(k, k, k, 1.0)));
        if t > 1e-3 {
            let kt = k * t;
            p.fill(&shape, ImagePaint::new(other, rect).tint(Color::linear(kt, kt, kt, t)));
        }
    }

    /// The procedural face (no render): a flat carbon-dark yoke with a faint lit top edge, the
    /// window bezel, the partition and the single housings, and the encoder wells.
    fn draw_face(p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        let ink = roles.ink;
        let b = g::yoke_bounds();
        p.fill(
            &Shape::Polygon { points: &g::YOKE },
            Gradient::linear(b.at(0.5, 0.0), b.at(0.5, 1.0))
                .stop(0.0, Color::srgb8(20, 21, 24, 255))
                .stop(1.0, Color::srgb8(10, 11, 12, 255)),
        );
        let bezel = g::WINDOW.expand(g::BEZEL);
        p.fill(
            &Shape::round_rect(bezel, g::WINDOW_R + g::BEZEL),
            Color::srgb8(5, 5, 6, 255),
        );
        p.stroke(
            &Shape::round_rect(bezel, g::WINDOW_R + g::BEZEL),
            ink.with_alpha(0.08),
            Stroke::rim_px(1.0),
        );
        let housing = |p: &mut Painter<'_>, r: Rect| {
            p.fill(&Shape::round_rect(r, 14.0), Color::srgb8(22, 23, 26, 255));
            p.stroke(&Shape::round_rect(r, 14.0), ink.with_alpha(0.12), Stroke::rim_px(1.0));
        };
        housing(p, g::PARTITION);
        for x in [-g::SINGLE_X, g::SINGLE_X] {
            housing(p, Rect::from_center_size(vec2(x, g::POD_Y), g::SINGLE_HOUSING));
        }
        for c in g::ENC {
            p.fill(&Shape::circle(c, g::WELL_R), Color::srgb8(6, 6, 7, 255));
            p.stroke(&Shape::circle(c, g::WELL_R), ink.with_alpha(0.08), Stroke::rim_px(1.0));
            let mid = (g::KNURL[0] + g::KNURL[1]) * 0.5;
            p.fill(
                &Shape::ring(c, mid, g::KNURL[1] - g::KNURL[0]),
                Color::srgb8(58, 30, 33, 255),
            );
            p.fill(&Shape::circle(c, g::KNURL[0] - 6.0), Color::srgb8(16, 17, 20, 255));
        }
        for c in g::pods() {
            let lens = Rect::from_center_size(c, g::POD);
            p.fill(&Shape::round_rect(lens, 7.0), ink.with_alpha(UNLIT_LENS));
        }
    }

    /// The ten lit lenses: `primary` below the shift point, `shift` (strobing on the limiter) at
    /// and above it, from `ShiftLights`' outside-in state.
    fn draw_pods(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        let bloom = LayerOptions {
            bloom: Some(Bloom {
                threshold: p.tokens().glow.bloom_threshold,
                intensity: p.tokens().glow.bloom_intensity * POD_BLOOM,
            }),
            ..LayerOptions::default()
        };
        let over = self.shift.over_shift();
        for (i, c) in g::pods().into_iter().enumerate() {
            let Some(stage) = self.shift.color_of(i, pal) else {
                continue;
            };
            let color = if over { stage } else { pal.primary };
            let lens = Rect::from_center_size(c, g::POD);
            let shape = Shape::round_rect(lens, 7.0);
            p.glow(&shape, color, Glow::new(6.0, 0.25));
            p.layer(bloom, |p| {
                p.fill(
                    &shape,
                    Gradient::radial(c, lens.width() * 0.5)
                        .stop(0.0, color.lerp(roles.ink.with_alpha(color.a), 0.4))
                        .stop(0.55, color)
                        .stop(1.0, color.fade(0.8)),
                );
            });
            let glint = Rect::from_xywh(lens.min.x + 12.0, lens.min.y + 3.0, lens.width() - 24.0, 2.0);
            p.fill(&Shape::round_rect(glint, 1.0), roles.ink.with_alpha(0.3));
        }
    }

    /// The round rev counter: track, red band, lit arc, ticks, numerals and the label.
    fn draw_rev_counter(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        let sig = &self.sig;
        let top = scale_top(sig.max_rpm);
        let span = top as f32 * 1000.0;
        let frac = |rpm: f32| if rpm.is_finite() { rpm / span } else { 0.0 };
        let sweep = deg(g::DIAL_SWEEP);
        let start = deg(g::DIAL_START);
        let limiter = self.on_limiter();
        p.fill(
            &Shape::arc(g::DIAL_C, g::DIAL_R, g::DIAL_W, start, sweep, Cap::Butt),
            Color::srgb8(14, 20, 26, 255),
        );
        // The band from the shift point to the scale top.
        let shift_rpm = if sig.shift_rpm > 1.0 { sig.shift_rpm } else { span };
        let f0 = frac(shift_rpm).clamp(0.0, 1.0);
        if f0 < 1.0 {
            let band = if limiter {
                pal.shift
            } else {
                roles.warning.with_alpha(RED_BAND_ALPHA)
            };
            p.fill(
                &Shape::arc(
                    g::DIAL_C,
                    g::RED_R,
                    g::RED_W,
                    dial_angle(f0),
                    sweep * (1.0 - f0),
                    Cap::Butt,
                ),
                band,
            );
        }
        let f = frac(sig.rpm).clamp(0.0, 1.0);
        if f > 1e-3 {
            let lit = if limiter { roles.ink } else { pal.primary };
            let arc = Shape::arc(g::DIAL_C, g::DIAL_R, g::DIAL_W, start, sweep * f, Cap::Butt);
            p.glow(&arc, lit, Glow::new(10.0, 0.3));
            p.fill(&arc, lit);
        }
        // Ticks every 500 rpm, majors every thousand.
        p.fill(
            &Shape::Ticks(TickRing::new(g::DIAL_C, g::TICK_R, start, sweep, top * 2 + 1).minor(g::TICK_MINOR, 2.0)),
            roles.ink.with_alpha(0.4),
        );
        p.fill(
            &Shape::Ticks(TickRing::new(g::DIAL_C, g::TICK_R, start, sweep, top + 1).minor(g::TICK_MAJOR, 4.0)),
            roles.ink.with_alpha(0.8),
        );
        let numeral = TextStyle::new(self.faces.digits, g::NUMERAL_CAP)
            .paint(roles.ink.with_alpha(NUMERAL_ALPHA))
            .align(Align::Center, VAlign::CapMiddle)
            .tabular();
        for (k, s) in NUMERALS.iter().enumerate().take(top as usize + 1) {
            let at = Vec2::polar(g::DIAL_C, g::NUMERAL_R, dial_angle(k as f32 / top as f32));
            p.text(at, s, &numeral);
        }
        let label = TextStyle::new(self.faces.label, g::LABEL_CAP)
            .paint(roles.ink.with_alpha(LABEL_ALPHA))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapMiddle);
        p.text(g::RPM_LABEL, "RPM x1000", &label);
    }

    /// Gear, speed and power.
    fn draw_readouts(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        p.layer(LayerOptions::bloom(), |p| {
            self.gear.draw(p, g::GEAR, self.faces.gear, g::GEAR_CAP, pal);
        });
        self.speed.draw(
            p,
            g::SPEED,
            self.faces.digits,
            g::SPEED_CAP,
            roles.ink,
            Some(roles.ink.with_alpha(GHOST_8)),
        );
        let unit = TextStyle::new(self.faces.label, g::UNIT_CAP)
            .paint(roles.ink.with_alpha(UNIT_ALPHA))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(g::UNIT, self.sig.unit.label(), &unit);

        let value = TextStyle::new(self.faces.digits, g::POWER_CAP)
            .paint(roles.ink)
            .align(Align::Right, VAlign::CapMiddle)
            .tabular();
        p.number(g::POWER, self.power_value, 4, &value, None);
        let unit = TextStyle::new(self.faces.label, g::POWER_UNIT_CAP)
            .paint(roles.ink.with_alpha(UNIT_ALPHA))
            .tracking(self.tracking)
            .align(Align::Left, VAlign::CapMiddle);
        let unit_label = match self.sig.power_unit {
            PowerUnit::Hp => "HP",
            PowerUnit::Kw => "KW",
        };
        p.text(g::POWER_UNIT, unit_label, &unit);
        let label = TextStyle::new(self.faces.label, g::LABEL_CAP)
            .paint(roles.ink.with_alpha(LABEL_ALPHA))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        p.text(g::POWER_LABEL, "POWER", &label);
    }

    /// The engineer trace: the last 4 s (3 s in the hood), newest on the right.
    fn draw_trace(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        let b = g::TRACE;
        p.stroke(&Shape::rect(b), roles.ink.with_alpha(0.12), Stroke::rim_px(1.0));
        let vi = self.view_index();
        // Segments shown: the span over the sampling period.
        let segs = ((TRACE_S[vi] / Trace::period()).round() as usize).clamp(1, TRACE_POINTS - 1);
        let mut pts = [Vec2::ZERO; TRACE_POINTS];
        let channels: [(usize, Color, f32); 3] = [
            (2, roles.ink.with_alpha(0.45), 2.0),
            (0, roles.go, 3.0),
            (1, roles.warning, 3.0),
        ];
        for (ch, color, width) in channels {
            for (j, pt) in pts.iter_mut().enumerate().take(segs + 1) {
                let s = self.trace.back(segs - j);
                let v = if ch == 2 { 0.5 + 0.5 * s[2] } else { s[ch] };
                let x = b.min.x + b.width() * j as f32 / segs as f32;
                let y = b.max.y - v.clamp(0.0, 1.0) * b.height();
                *pt = vec2(x, y);
            }
            let mut i = 0;
            while i < segs {
                let end = (i + TRACE_CHUNK).min(segs);
                p.fill(
                    &Shape::Polyline {
                        points: &pts[i..=end],
                        width,
                        closed: false,
                    },
                    color,
                );
                i = end;
            }
        }
        let label = TextStyle::new(self.faces.label, g::LABEL_CAP)
            .paint(roles.ink.with_alpha(LABEL_ALPHA))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        let text = if vi == 1 {
            "THR  BRK  STR · 3 S"
        } else {
            "THR  BRK  STR · 4 S"
        };
        p.text(g::TRACE_LABEL, text, &label);
    }

    /// TC and ABS: dark lenses with shy labels, lit in their tone with a dark label.
    fn draw_lamps(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        for (i, (lamp, c)) in self.lamps.lamps().iter().zip(g::LAMPS).enumerate() {
            let lens = Rect::from_center_size(c, g::LAMP);
            let shape = Shape::round_rect(lens, 9.0);
            let on = self.lamps.is_lit(i);
            let (fill, ink) = if on {
                (lamp.tone.color(pal), roles.oled_black)
            } else {
                (Color::srgb8(11, 12, 14, 255), roles.ink.with_alpha(LAMP_LABEL_ALPHA))
            };
            p.fill(&shape, fill);
            let style = TextStyle::new(self.faces.label, 18.0)
                .paint(ink)
                .tracking(self.tracking)
                .align(Align::Center, VAlign::CapMiddle);
            p.text(c, lamp.label, &style);
        }
    }

    /// The encoders' data arcs and labels: pedals (throttle outer, brake inner) on the left,
    /// power share on the right.
    fn draw_encoders(&self, p: &mut Painter<'_>, pal: &Palette) {
        let roles = pal.roles;
        let start = deg(g::ARC_START);
        let sweep = deg(g::ARC_SWEEP);
        let arc = |p: &mut Painter<'_>, c: Vec2, r: f32, f: f32, color: Color| {
            if f > 0.01 {
                p.fill(&Shape::arc(c, r, g::ARC_W, start, sweep * f.min(1.0), Cap::Butt), color);
            }
        };
        let label = TextStyle::new(self.faces.label, g::LABEL_CAP)
            .paint(roles.ink.with_alpha(LABEL_ALPHA))
            .tracking(self.tracking)
            .align(Align::Center, VAlign::CapTop);
        for (i, c) in g::ENC.into_iter().enumerate() {
            p.fill(
                &Shape::arc(c, g::ARC_R[0], g::ARC_W, start, sweep, Cap::Butt),
                roles.ink.with_alpha(0.1),
            );
            if i == 0 {
                arc(p, c, g::ARC_R[0], self.sig.throttle, roles.go);
                arc(p, c, g::ARC_R[1], self.sig.brake, roles.warning);
                p.text(c + vec2(0.0, g::ENC_LABEL_DY), "PEDALS", &label);
            } else {
                arc(p, c, g::ARC_R[0], self.power_share(), pal.primary);
                p.text(c + vec2(0.0, g::ENC_LABEL_DY), "POWER %", &label);
            }
        }
    }
}

/// The registry constructor (`registry::REGISTRY`).
pub fn make(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(ValkyrieLmp1::new(setup))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_stays_inside_the_envelopes_and_clear() {
        let t = Tokens::default();
        let env = t.layout.cockpit_envelope;
        let hood = t.layout.hood_envelope;
        for v in g::YOKE {
            assert!(env.contains(v), "yoke point {v:?} in the cockpit envelope");
            assert!(
                hood.contains(v * HOOD_SCALE),
                "scaled yoke point {v:?} in the hood envelope"
            );
        }
        for c in g::ENC {
            let r = Rect::from_center_size(c, Vec2::splat(g::WELL_R * 2.0));
            assert!(
                env.contains(r.min) && env.contains(r.max),
                "encoder {c:?} in the envelope"
            );
            assert!(
                r.max.x.abs().min(r.min.x.abs()) > g::WINDOW.max.x,
                "encoder {c:?} outside the window"
            );
        }
        // The dial is the largest element in the window and at least 40% of its width (F2), and
        // its red band stays inside the window.
        assert!(g::DIAL_R * 2.0 >= 0.4 * g::WINDOW.width());
        let outer = g::RED_R + g::RED_W * 0.5;
        assert!(
            g::DIAL_C.y - outer >= g::WINDOW.min.y + 4.0,
            "dial clear of the window top"
        );
        // The gear is centred on the dial (F8) and clear of the lowest numerals.
        assert!((g::GEAR.x - g::DIAL_C.x).abs() <= 10.0);
        let low = Vec2::polar(g::DIAL_C, g::NUMERAL_R, dial_angle(0.0));
        assert!(
            low.x.abs() - g::NUMERAL_CAP > g::GEAR_CAP * 0.33,
            "numeral 0 clear of the gear"
        );
        // The strobing strip is <= 42 px tall; pods sit in the partition or on the shoulders.
        assert!(g::POD.y <= 42.0);
        let pods = g::pods();
        for c in &pods[1..9] {
            let r = Rect::from_center_size(*c, g::POD);
            assert!(r.min.x > g::PARTITION.min.x && r.max.x < g::PARTITION.max.x);
            assert!(r.min.y > g::PARTITION.min.y && r.max.y < g::PARTITION.max.y);
        }
        assert!(
            (pods[4].x + pods[5].x).abs() < 1e-3,
            "the centre pair straddles the gear"
        );
        // Speed, power, trace and lamps inside the window, clear of the dial's band.
        for r in [
            g::TRACE,
            Rect::from_center_size(g::LAMPS[0], g::LAMP),
            Rect::from_center_size(g::LAMPS[1], g::LAMP),
        ] {
            assert!(
                g::WINDOW.contains(r.min) && g::WINDOW.contains(r.max),
                "{r:?} in the window"
            );
        }
        let speed_right = g::SPEED.x + g::SPEED_CAP * 0.6 * 1.5;
        assert!(speed_right < g::DIAL_C.x - outer, "speed clear of the dial");
        assert!(g::TRACE.min.x > g::DIAL_C.x + outer, "trace clear of the dial");
    }

    #[test]
    fn scale_top_rounds_the_limiter_up() {
        assert_eq!(scale_top(11_100.0), 12);
        assert_eq!(scale_top(8_000.0), 8);
        assert_eq!(scale_top(f32::NAN), 8);
        assert_eq!(scale_top(1_000.0), 4);
        assert_eq!(scale_top(90_000.0), 20);
    }

    #[test]
    fn trace_scrolls_at_a_fixed_rate_and_settles() {
        let mut tr = Trace::new();
        assert!(!tr.moving());
        tr.update([1.0, 0.0, 0.0], Trace::period() * 1.01);
        assert!(tr.moving(), "a new value shows against the old ones");
        for _ in 0..(TRACE_POINTS + 2) {
            tr.update([1.0, 0.0, 0.0], Trace::period());
        }
        assert!(!tr.moving(), "once the ring holds one value it stops");
        assert_eq!(tr.back(0), [1.0, 0.0, 0.0]);
        // A long frame never loops forever.
        tr.update([0.0, 1.0, 0.0], 1.0e6);
        assert_eq!(tr.back(0), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn lamps_follow_their_sources() {
        let mut sig = Signals {
            speed_ms: 30.0,
            ..Signals::default()
        };
        assert_eq!(lamp_conditions(&sig, 1, 0.0), [false, false]);
        sig.brake = 0.8;
        sig.g_long = -1.1;
        assert_eq!(lamp_conditions(&sig, 1, 0.0), [false, true]);
        sig.brake = 0.0;
        sig.g_long = 0.3;
        sig.throttle = 0.9;
        sig.slip_deg = 6.0;
        assert_eq!(lamp_conditions(&sig, 1, 20.0), [true, false]);
        assert_eq!(
            lamp_conditions(&sig, lamp::FWD, 20.0),
            [false, false],
            "no TC on front drive"
        );
    }
}
