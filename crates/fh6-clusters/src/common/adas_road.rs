//! AdasRoad: the 2.5D road scene of the ADAS & HUD family (COMPONENTS.md §10, tokens
//! `components.adasRoad`, `docs/design/families/adas-hud.md`), glance rank 4.
//!
//! A Tesla-style chase camera (eye `(0, 3.4, -6.0)` m, pitched 8 degrees down, 33 degree vertical
//! field of view, near plane 0.1 m) looks over the ego car at `z = 3.2` m onto a three-lane road
//! bent by `kappa = yawRate / max(v, 3 m/s)`: solid outer edges and dashed inner lines that scroll
//! with the distance travelled, a two-tone path carpet that steps down to 35% beyond the braking
//! distance `v^2 / (2 * 9 m/s^2)`, neutral traffic with the in-path lead car in FSD corner
//! brackets, 3-step blind-spot arcs and an underglow under the ego. Lines and the carpet dissolve
//! towards the horizon and the viewport's sides (no viewport box). Nothing writes to the bloom
//! buffer: the scene is rank 4.
//!
//! **Honesty:** the telemetry has no traffic, map or route data. The road ahead is a prediction
//! from the yaw rate, and the road users are a seeded ambient simulation ([`AdasRoad::new`]); a
//! cluster labels them as such and never presents them as sensor data.
//!
//! **Sign conventions** (`tests/golden/packets` carry no angular velocity, so they are not
//! verified there): [`AdasInput::yaw_rate`] is positive when turning right, matching
//! [`RoadCurve`]. [`AdasInput::from_signals`] passes `Signals::yaw_rate` through unchanged, so the
//! caller verifies the Forza sign once and negates it if needed. The ego's drift yaw uses only the
//! slip magnitude (the nose points into the bend, tokens `egoHeading`), so the slip sign does not
//! matter.
use fh6_kit::math::{finite_or, smoothstep, vec3, Vec3};
use fh6_kit::perspective::{Camera25, RoadCurve, MAX_KAPPA};
use fh6_kit::{
    vec2, Align, Color, FontRole, Glow, Gradient, Painter, Palette, Rect, Shape, Signals, Spring, Stroke, TextStyle,
    Tokens, VAlign, Vec2,
};

/// Chase-camera eye in world metres (tokens `components.adasRoad.eye`).
pub const EYE: Vec3 = vec3(0.0, 3.4, -6.0);
/// Downward camera pitch in degrees (tokens `components.adasRoad.pitchDeg`).
pub const PITCH_DEG: f32 = 8.0;
/// Vertical field of view in degrees (tokens `components.adasRoad.verticalFovDeg`).
pub const VERTICAL_FOV_DEG: f32 = 33.0;
/// Depth of the ego car's centre in metres (tokens `components.adasRoad.egoZ`).
pub const EGO_Z: f32 = 3.2;
/// Lane width in metres (tokens `components.adasRoad.laneWidthM`); the road has 3 lanes.
pub const LANE_WIDTH: f32 = 3.6;
/// Most road users simulated and drawn (tokens `colorRoles.adas.maxRoadUsers`).
pub const MAX_ROAD_USERS: usize = 6;
/// Speed floor of the curvature model in m/s (tokens `components.adasRoad.curvature`).
pub const MIN_CURVE_SPEED: f32 = 3.0;
/// Deceleration of the braking-distance estimate `v^2 / (2 a)` in m/s^2 (COMPONENTS §10).
pub const BRAKE_DECEL: f32 = 9.0;

/// Side fade width in local px (tokens `components.adasRoad.fadeEdgesPx`).
const FADE_EDGES_PX: f32 = 120.0;
/// Bottom and top fade widths in local px: the road dissolves into the cluster, no box edge.
const BOTTOM_FADE_PX: f32 = 44.0;
const TOP_FADE_PX: f32 = 16.0;
/// Carpet alpha at the ego (tokens `carpet.nearAlpha`) and the tone beyond the braking point
/// (tokens `carpet.brakeZoneAlphaPct`).
const CARPET_NEAR_ALPHA: f32 = 0.55;
const BRAKE_ZONE_TONE: f32 = 0.35;
/// Carpet half width (metres): the ego's track plus a hand's width.
const CARPET_HALF_WIDTH: f32 = 1.05;
/// Points per carpet side (2 x 32 = tokens `carpet.segments`).
const CARPET_SAMPLES: usize = 32;
/// Far end of the drawn road (metres); lines and carpet reach zero alpha there.
const Z_FAR: f32 = 85.0;
/// Depth where the lines start to dissolve towards the horizon.
const Z_FADE: f32 = 26.0;
/// Depth band where road users fade out towards the horizon.
const CAR_FADE: [f32; 2] = [50.0, 76.0];
/// Samples along each solid edge line.
const LINE_SAMPLES: usize = 36;
/// Edge and dash line widths at the ego's depth (local px; family spec: 2 px edge lines).
const EDGE_PX: f32 = 2.0;
const DASH_PX: f32 = 3.2;
/// Peak alpha of the neutral (ink) edge lines and dashes.
const EDGE_ALPHA: f32 = 0.42;
const DASH_ALPHA: f32 = 0.62;
/// Dash length and period (metres): 3 m lines with 9 m gaps.
const DASH_LEN: f32 = 3.0;
const DASH_PERIOD: f32 = 12.0;
/// Underglow Gaussian sigma as a fraction of the ego's width (tokens `underglow.sigmaPctOfCarWidth`).
const UNDERGLOW_SIGMA: f32 = 0.30;
/// Ego body: light neutral tinted 15% by the brand (family spec).
const EGO_TINT: f32 = 0.15;
/// Traffic luminance: `colorRoles.adas.neutralObject` at `neutralAlpha` over black, as an ink scale.
const TRAFFIC_LUMA: f32 = 0.22;
/// A blind-spot car's ghost body alpha (tokens `trafficRules`).
const GHOST_ALPHA: f32 = 0.30;
/// An adjacent-lane car nearer than `EGO_Z + 6 m` is a blind-spot ghost (tokens `trafficRules`).
const BLIND_SPOT_AHEAD: f32 = 6.0;
/// Longitudinal gaps (metres, centre to centre) of the 3 proximity steps: within 10 m, 6 m, 3 m.
const PROXIMITY_STEPS_M: [f32; 3] = [10.0, 6.0, 3.0];
/// Slip (degrees) and speed (m/s) above which the scene shows a drift (amber carpet).
const DRIFT_SLIP_DEG: f32 = 10.0;
const DRIFT_MIN_SPEED: f32 = 5.0;
/// Largest drift yaw of the ego model (degrees).
const MAX_EGO_YAW_DEG: f32 = 55.0;
/// Face shading: ambient plus Lambert from the virtual light.
const AMBIENT: f32 = 0.40;
const DIFFUSE: f32 = 0.60;
/// Ambient traffic loop in the adjacent lanes (z metres): a car leaving one end re-enters at the
/// other.
const WRAP_NEAR: f32 = -14.0;
const WRAP_FAR: f32 = 78.0;
/// The in-path lead car stays between these depths.
const LEAD_NEAR: f32 = EGO_Z + 12.0;
const LEAD_FAR: f32 = 62.0;
/// Ego speed (m/s) at which traffic moves at its full relative speed; slower, the traffic slows
/// with the ego, so a parked ego sees parked traffic.
const TRAFFIC_FULL_SPEED: f32 = 12.0;
/// Road users in a fresh simulation.
const DEFAULT_ROAD_USERS: usize = 5;

/// The underglow state under the ego (COMPONENTS §10: nominal, assist, limiter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Underglow {
    /// Nominal driving: the brand primary.
    #[default]
    Nominal,
    /// A driver assist is intervening: caution amber.
    Assist,
    /// On the rev limiter: the palette's shift colour.
    Limiter,
}

/// What the road scene reads each frame (plain values, so a cluster can adapt units and signs).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AdasInput {
    /// Speed in m/s.
    pub speed_ms: f32,
    /// Yaw rate in rad/s, positive turning right (verify the telemetry sign before relying on it).
    pub yaw_rate: f32,
    /// Body slip angle in degrees (only its magnitude is used).
    pub slip_deg: f32,
    /// Brake pedal 0..=1.
    pub brake: f32,
    /// Smoothed lateral G (swings the virtual light across the car bodies).
    pub g_lat: f32,
    /// Underglow state.
    pub underglow: Underglow,
}

impl AdasInput {
    /// The inputs from decoded [`Signals`]; the underglow shows the limiter at or above the shift
    /// rpm and is nominal otherwise (assists are not in the packet: see [`AdasInput::with_underglow`]).
    #[must_use]
    pub fn from_signals(sig: &Signals) -> Self {
        Self {
            speed_ms: sig.speed_ms,
            yaw_rate: sig.yaw_rate,
            slip_deg: sig.slip_deg,
            brake: sig.brake,
            g_lat: sig.g_lat,
            underglow: if sig.shift_fraction >= 1.0 {
                Underglow::Limiter
            } else {
                Underglow::Nominal
            },
        }
    }

    /// The same inputs with another underglow state.
    #[must_use]
    pub fn with_underglow(mut self, underglow: Underglow) -> Self {
        self.underglow = underglow;
        self
    }
}

/// The procedural proxy body of a car (2.5D shaded hulls until the Phase 6 glTF cars are wired).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CarKind {
    /// The ego coupé.
    Coupe,
    /// A sport sedan.
    Sedan,
    /// An SUV.
    Suv,
    /// A low supercar.
    Supercar,
}

/// Proportions of a proxy body.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Body {
    length: f32,
    width: f32,
    height: f32,
    /// Belt-line height as a fraction of `height`.
    shoulder: f32,
    /// Cabin base: rear and front ends as fractions of `length` from the centre.
    cabin_base: [f32; 2],
    /// Roof: rear and front ends as fractions of `length` from the centre.
    cabin_roof: [f32; 2],
    /// Cabin widths at the belt line and at the roof as fractions of `width`.
    cabin_width: [f32; 2],
}

impl CarKind {
    const TRAFFIC: [CarKind; 3] = [CarKind::Sedan, CarKind::Suv, CarKind::Supercar];

    fn body(self) -> Body {
        match self {
            CarKind::Coupe => Body {
                length: 4.5,
                width: 1.94,
                height: 1.30,
                shoulder: 0.56,
                cabin_base: [-0.40, 0.16],
                cabin_roof: [-0.25, -0.02],
                cabin_width: [0.86, 0.66],
            },
            CarKind::Sedan => Body {
                length: 4.8,
                width: 1.86,
                height: 1.44,
                shoulder: 0.58,
                cabin_base: [-0.36, 0.20],
                cabin_roof: [-0.24, 0.05],
                cabin_width: [0.88, 0.70],
            },
            CarKind::Suv => Body {
                length: 4.9,
                width: 1.96,
                height: 1.74,
                shoulder: 0.58,
                cabin_base: [-0.44, 0.22],
                cabin_roof: [-0.41, 0.08],
                cabin_width: [0.90, 0.80],
            },
            CarKind::Supercar => Body {
                length: 4.6,
                width: 2.02,
                height: 1.16,
                shoulder: 0.50,
                cabin_base: [-0.30, 0.18],
                cabin_roof: [-0.17, 0.0],
                cabin_width: [0.80, 0.58],
            },
        }
    }

    /// Body length in metres.
    #[must_use]
    pub fn length(self) -> f32 {
        self.body().length
    }
}

/// A body placed on the road.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Placed {
    x: f32,
    z: f32,
    /// Heading in radians, positive turned right.
    heading: f32,
    body: Body,
}

/// Faces of a hull whose corners are `[RL, RR, FR, FL]` at the base, then the same at the top:
/// roof, rear, front, left, right (the floor is never seen from the chase camera).
const FACES: [[usize; 4]; 5] = [[4, 5, 6, 7], [0, 1, 5, 4], [2, 3, 7, 6], [3, 0, 4, 7], [1, 2, 6, 5]];
/// Index of the roof face in [`FACES`].
const ROOF: usize = 0;
/// Index of the rear face in [`FACES`].
const REAR: usize = 1;

impl Placed {
    fn to_world(&self, lx: f32, ly: f32, lz: f32) -> Vec3 {
        let (s, c) = self.heading.sin_cos();
        vec3(self.x + lx * c + lz * s, ly, self.z - lx * s + lz * c)
    }

    /// The lower body and the cabin as two hulls.
    fn hulls(&self) -> [[Vec3; 8]; 2] {
        let b = self.body;
        let (hl, hw) = (b.length * 0.5, b.width * 0.5);
        let clearance = b.height * 0.08;
        let belt = b.height * b.shoulder;
        // The body side tucks in towards the belt line (tumblehome).
        let tw = hw * 0.94;
        let lower = [
            self.to_world(-hw, clearance, -hl),
            self.to_world(hw, clearance, -hl),
            self.to_world(hw, clearance, hl),
            self.to_world(-hw, clearance, hl),
            self.to_world(-tw, belt, -hl * 0.98),
            self.to_world(tw, belt, -hl * 0.98),
            self.to_world(tw, belt, hl * 0.95),
            self.to_world(-tw, belt, hl * 0.95),
        ];
        let (cw, rw) = (tw * b.cabin_width[0], tw * b.cabin_width[1]);
        let (cr, cf) = (b.cabin_base[0] * b.length, b.cabin_base[1] * b.length);
        let (rr, rf) = (b.cabin_roof[0] * b.length, b.cabin_roof[1] * b.length);
        let cabin = [
            self.to_world(-cw, belt, cr),
            self.to_world(cw, belt, cr),
            self.to_world(cw, belt, cf),
            self.to_world(-cw, belt, cf),
            self.to_world(-rw, b.height, rr),
            self.to_world(rw, b.height, rr),
            self.to_world(rw, b.height, rf),
            self.to_world(-rw, b.height, rf),
        ];
        [lower, cabin]
    }

    /// The ground footprint grown by `grow` metres, `[RL, RR, FR, FL]`.
    fn footprint(&self, grow: f32) -> [Vec3; 4] {
        let (hl, hw) = (self.body.length * 0.5 + grow, self.body.width * 0.5 + grow);
        [
            self.to_world(-hw, 0.0, -hl),
            self.to_world(hw, 0.0, -hl),
            self.to_world(hw, 0.0, hl),
            self.to_world(-hw, 0.0, hl),
        ]
    }
}

/// The outward unit normal and centre of `face` on `hull` (orientation fixed against the hull's
/// centre, so the corner winding does not matter).
fn face_normal(hull: &[Vec3; 8], face: &[usize; 4]) -> (Vec3, Vec3) {
    let v = face.map(|i| hull[i]);
    let center = (v[0] + v[1] + v[2] + v[3]) * 0.25;
    let hull_center = hull.iter().fold(Vec3::default(), |a, &b| a + b) * 0.125;
    let n = (v[2] - v[0]).cross(v[3] - v[1]);
    let n = if n.dot(center - hull_center) < 0.0 { n * -1.0 } else { n };
    (n.normalize_or_zero(), center)
}

/// How a car is painted.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Look {
    body: Color,
    glass: Color,
    alpha: f32,
    /// 1 px roof rim (transparent for none).
    rim: Color,
}

/// One ambient road user.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RoadUser {
    /// -1 left, 0 the ego's lane, 1 right.
    lane: i8,
    /// Depth of the car's centre (metres).
    z: f32,
    kind: CarKind,
}

/// A small deterministic generator (SplitMix64) for seeding the ambient traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..1`.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn index(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// The seeded ambient traffic: one in-path lead car that hovers ahead, the others cruising the
/// adjacent lanes at a per-lane speed relative to the ego and wrapping around.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Traffic {
    users: [RoadUser; MAX_ROAD_USERS],
    count: usize,
    /// Relative speeds (m/s) of the left lane, the lead car (magnitude) and the right lane.
    lane_speed: [f32; 3],
    /// Direction the lead car currently drifts in (+1 away, -1 closer).
    lead_dir: f32,
}

impl Traffic {
    fn seeded(seed: u64, count: usize) -> Self {
        let mut rng = SplitMix64(seed);
        let count = count.min(MAX_ROAD_USERS);
        let parked = RoadUser {
            lane: 0,
            z: LEAD_FAR,
            kind: CarKind::Sedan,
        };
        let mut users = [parked; MAX_ROAD_USERS];
        // The left lane overtakes the ego, the right lane is overtaken: both sides see cars pass.
        let lane_speed = [0.8 + 1.8 * rng.unit(), 0.5 + 1.0 * rng.unit(), -(0.8 + 1.8 * rng.unit())];
        if count > 0 {
            users[0] = RoadUser {
                lane: 0,
                z: LEAD_NEAR + 4.0 + (LEAD_FAR - LEAD_NEAR - 24.0) * rng.unit(),
                kind: CarKind::TRAFFIC[rng.index(3)],
            };
        }
        let others = count.saturating_sub(1);
        let first: i8 = if rng.unit() < 0.5 { -1 } else { 1 };
        let n_first = others.div_ceil(2);
        let span = WRAP_FAR - WRAP_NEAR;
        for (lane, n, start) in [(first, n_first, 1), (-first, others - n_first, 1 + n_first)] {
            let base = rng.unit() * span;
            for i in 0..n {
                // Even spacing per lane (>= 30 m with 3 cars), so same-lane cars never overlap.
                let jitter = (rng.unit() - 0.5) * 6.0;
                let z = WRAP_NEAR + (base + i as f32 * span / n as f32 + jitter).rem_euclid(span);
                users[start + i] = RoadUser {
                    lane,
                    z,
                    kind: CarKind::TRAFFIC[rng.index(3)],
                };
            }
        }
        Self {
            users,
            count,
            lane_speed,
            lead_dir: if rng.unit() < 0.5 { -1.0 } else { 1.0 },
        }
    }

    fn users(&self) -> &[RoadUser] {
        &self.users[..self.count]
    }

    fn update(&mut self, dt: f32, ego_speed: f32) {
        let step = (ego_speed / TRAFFIC_FULL_SPEED).clamp(0.0, 1.0) * dt;
        let span = WRAP_FAR - WRAP_NEAR;
        for u in self.users[..self.count].iter_mut() {
            if u.lane == 0 {
                u.z += self.lead_dir * self.lane_speed[1] * step;
                if u.z <= LEAD_NEAR {
                    self.lead_dir = 1.0;
                } else if u.z >= LEAD_FAR {
                    self.lead_dir = -1.0;
                }
                u.z = u.z.clamp(LEAD_NEAR, LEAD_FAR);
            } else {
                let speed = if u.lane < 0 { self.lane_speed[0] } else { self.lane_speed[2] };
                u.z = WRAP_NEAR + (u.z + speed * step - WRAP_NEAR).rem_euclid(span);
            }
        }
    }

    /// Index of the nearest car ahead in the ego's lane.
    fn lead(&self) -> Option<usize> {
        self.users()
            .iter()
            .enumerate()
            .filter(|(_, u)| u.lane == 0 && u.z > EGO_Z)
            .min_by(|a, b| a.1.z.total_cmp(&b.1.z))
            .map(|(i, _)| i)
    }

    /// Proximity step (0..=3) on the left and right.
    fn blind_spot(&self) -> [u8; 2] {
        let mut out = [0u8; 2];
        for u in self.users() {
            let side = match u.lane {
                -1 => 0,
                1 => 1,
                _ => continue,
            };
            out[side] = out[side].max(proximity_step(u.z - EGO_Z));
        }
        out
    }
}

/// The proximity step for a longitudinal gap (metres, centre to centre): 3 alongside, 0 clear.
fn proximity_step(gap: f32) -> u8 {
    let gap = gap.abs();
    if !gap.is_finite() || gap > PROXIMITY_STEPS_M[0] {
        0
    } else if gap > PROXIMITY_STEPS_M[1] {
        1
    } else if gap > PROXIMITY_STEPS_M[2] {
        2
    } else {
        3
    }
}

/// A step change shown as a crossfade (tokens `motion.durations.crossfade`).
#[derive(Debug, Clone, Copy, PartialEq)]
struct StepFade {
    from: u8,
    to: u8,
    /// Crossfade progress 0..=1.
    t: f32,
}

impl StepFade {
    const IDLE: StepFade = StepFade { from: 0, to: 0, t: 1.0 };

    fn update(&mut self, target: u8, dt: f32, duration: f32) {
        if target != self.to {
            self.from = if self.t >= 0.5 { self.to } else { self.from };
            self.to = target;
            self.t = 0.0;
        }
        self.t = (self.t + dt / duration.max(1e-3)).min(1.0);
    }

    fn running(&self) -> bool {
        self.t < 1.0
    }
}

/// Moves `value` towards `target` at a rate that covers 0..1 in `duration` seconds.
fn approach(value: f32, target: f32, dt: f32, duration: f32) -> f32 {
    let step = dt / duration.max(1e-3);
    value + (target - value).clamp(-step, step)
}

/// The road curvature (1/m) for a yaw rate (rad/s) and speed (m/s): `yawRate / max(v, 3 m/s)`,
/// clamped to the kit's 8 m radius; non-finite input gives a straight road.
#[must_use]
pub fn curvature(yaw_rate: f32, speed_ms: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).abs().max(MIN_CURVE_SPEED);
    (finite_or(yaw_rate, 0.0) / v).clamp(-MAX_KAPPA, MAX_KAPPA)
}

/// The ego model's yaw relative to the path (radians): the slip magnitude, turned into the bend
/// (`sign(kappa)`, tokens `egoHeading`); on a straight path the nose points against the slide.
#[must_use]
pub fn ego_yaw(kappa: f32, slip_deg: f32) -> f32 {
    let slip = finite_or(slip_deg, 0.0);
    let magnitude = slip.abs().min(MAX_EGO_YAW_DEG).to_radians();
    let dir = if kappa.abs() > 1e-3 { kappa.signum() } else { -slip.signum() };
    dir * magnitude
}

/// The braking distance (metres) at `speed_ms`: `v^2 / (2 * 9 m/s^2)`.
#[must_use]
pub fn braking_distance(speed_ms: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).max(0.0);
    v * v / (2.0 * BRAKE_DECEL)
}

/// The world depth of the ground seen at local row `y`, or [`Z_FAR`] at and above the horizon.
fn ground_z_at(cam: &Camera25, y: f32) -> f32 {
    let (s, c) = cam.pitch.sin_cos();
    let k = (y - cam.optical_center_y()) / cam.fy.max(1e-3);
    let den = k * c + s;
    if den <= 1e-4 {
        return Z_FAR;
    }
    cam.eye.z + cam.eye.y * (c - k * s) / den
}

/// Alpha towards the horizon.
fn depth_fade(z: f32) -> f32 {
    1.0 - smoothstep(Z_FADE, Z_FAR, z)
}

/// The per-frame projection shared by the scene's passes.
#[derive(Debug, Clone, Copy)]
struct Scene3 {
    cam: Camera25,
    road: RoadCurve,
    vp: Rect,
    /// Ground depth at the viewport's bottom edge.
    z_near: f32,
    /// Camera depth of the ego's ground point (line widths are 1:1 there).
    ego_depth: f32,
    fade_px: f32,
}

impl Scene3 {
    fn new(vp: Rect, kappa: f32) -> Self {
        let cam = AdasRoad::camera(&vp);
        let z_near = ground_z_at(&cam, vp.max.y).clamp(EYE.z + 1.0, EGO_Z);
        Self {
            cam,
            road: RoadCurve { kappa, tan_beta: 0.0 },
            vp,
            z_near,
            ego_depth: cam.depth(vec3(0.0, 0.0, EGO_Z)).max(0.5),
            fade_px: FADE_EDGES_PX.min(vp.width() * 0.25),
        }
    }

    /// A ground point at `offset` metres from the path centre.
    fn lane_point(&self, offset: f32, z: f32) -> Option<Vec2> {
        self.cam.project(vec3(self.road.center_x(z) + offset, 0.0, z))
    }

    /// The screen row of the ground at depth `z` (independent of x on flat ground).
    fn ground_y(&self, z: f32) -> f32 {
        self.cam
            .project(vec3(0.0, 0.0, z))
            .map_or(self.cam.horizon_y, |q| q.y)
    }

    /// Depth sample `i` of `n` from the viewport bottom to the far end, denser near the car.
    fn sample_z(&self, i: usize, n: usize) -> f32 {
        let t = i as f32 / (n.max(2) - 1) as f32;
        self.z_near + (Z_FAR - self.z_near) * t.powf(1.6)
    }

    /// A ground line `px` wide at the ego, in perspective at depth `z`.
    fn line_width(&self, px: f32, z: f32) -> f32 {
        let d = self.cam.depth(vec3(0.0, 0.0, z)).max(0.5);
        px * (self.ego_depth / d).clamp(0.4, 1.35)
    }

    fn side_fade(&self, q: Vec2) -> f32 {
        smoothstep(self.vp.min.x, self.vp.min.x + self.fade_px, q.x)
            * smoothstep(self.vp.max.x, self.vp.max.x - self.fade_px, q.x)
    }

    /// Side, bottom and top dissolve at `q`.
    fn edge_fade(&self, q: Vec2) -> f32 {
        self.side_fade(q)
            * smoothstep(self.vp.max.y, self.vp.max.y - BOTTOM_FADE_PX, q.y)
            * smoothstep(self.vp.min.y, self.vp.min.y + TOP_FADE_PX, q.y)
    }

    /// A car on the path at lateral `offset` (metres) and depth `z`, yawed `yaw` off the path.
    fn place(&self, offset: f32, z: f32, kind: CarKind, yaw: f32) -> Placed {
        Placed {
            x: self.road.center_x(z) + offset,
            z,
            heading: self.road.heading(z) + yaw,
            body: kind.body(),
        }
    }
}

/// The ADAS road scene: predicted road, path carpet, ambient traffic and the ego car
/// (COMPONENTS §10). Build one per cluster, feed it each frame with [`AdasRoad::update`] and draw
/// it into the scene band with [`AdasRoad::draw`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdasRoad {
    seed: u64,
    traffic: Traffic,
    kappa: Spring,
    kappa_target: f32,
    yaw: Spring,
    yaw_target: f32,
    brake: Spring,
    brake_target: f32,
    light: Spring,
    light_target: f32,
    speed: f32,
    /// Distance travelled modulo the dash period (scrolls the dashes).
    travelled: f32,
    drift: f32,
    assist: f32,
    limiter: f32,
    proximity: [StepFade; 2],
    crossfade: f32,
}

impl AdasRoad {
    /// A scene with the tokens' motion and a seeded ambient traffic simulation of 5 road users.
    ///
    /// The traffic is synthesised (the telemetry has none): the same `seed` and inputs always give
    /// the same cars, so mockups and goldens are reproducible.
    #[must_use]
    pub fn new(tokens: &Tokens, seed: u64) -> Self {
        Self {
            seed,
            traffic: Traffic::seeded(seed, DEFAULT_ROAD_USERS),
            kappa: Spring::new(tokens.springs.default, 0.0),
            kappa_target: 0.0,
            yaw: Spring::new(tokens.springs.default, 0.0),
            yaw_target: 0.0,
            brake: Spring::new(tokens.springs.snap, 0.0),
            brake_target: 0.0,
            light: Spring::new(tokens.springs.light, 0.0),
            light_target: 0.0,
            speed: 0.0,
            travelled: 0.0,
            drift: 0.0,
            assist: 0.0,
            limiter: 0.0,
            proximity: [StepFade::IDLE; 2],
            crossfade: tokens.durations.crossfade,
        }
    }

    /// The same scene re-seeded with `count` road users (at most [`MAX_ROAD_USERS`]; 0 for an
    /// empty road).
    #[must_use]
    pub fn with_road_users(mut self, count: usize) -> Self {
        self.traffic = Traffic::seeded(self.seed, count);
        self
    }

    /// The chase camera for a scene drawn into `viewport` (local px): the vertical field of view
    /// spans the viewport's height and the optical axis passes through its centre.
    #[must_use]
    pub fn camera(viewport: &Rect) -> Camera25 {
        let focal = 0.5 * viewport.height().max(1.0) / (0.5 * VERTICAL_FOV_DEG.to_radians()).tan();
        let pitch = PITCH_DEG.to_radians();
        let c = viewport.center();
        Camera25::chase(EYE, pitch, focal, focal, c.x, c.y - focal * pitch.tan())
    }

    /// Advances the scene by `dt` seconds: curvature, drift yaw, brake lights and light on the
    /// tokens' springs, dash scrolling and traffic from the speed, step crossfades.
    pub fn update(&mut self, input: &AdasInput, dt: f32) {
        let dt = finite_or(dt, 0.0).clamp(0.0, 0.25);
        let v = finite_or(input.speed_ms, 0.0).max(0.0);
        self.speed = v;
        self.travelled = (self.travelled + v * dt).rem_euclid(DASH_PERIOD);

        self.kappa_target = curvature(input.yaw_rate, v);
        self.kappa.update(self.kappa_target, dt);
        self.yaw_target = ego_yaw(self.kappa_target, input.slip_deg);
        self.yaw.update(self.yaw_target, dt);
        self.brake_target = finite_or(input.brake, 0.0).clamp(0.0, 1.0);
        self.brake.update(self.brake_target, dt);
        self.light_target = (finite_or(input.g_lat, 0.0) / 1.5).clamp(-1.0, 1.0);
        self.light.update(self.light_target, dt);

        let drifting = finite_or(input.slip_deg, 0.0).abs() >= DRIFT_SLIP_DEG && v >= DRIFT_MIN_SPEED;
        self.drift = approach(self.drift, f32::from(u8::from(drifting)), dt, self.crossfade);
        let assist = input.underglow == Underglow::Assist;
        let limiter = input.underglow == Underglow::Limiter;
        self.assist = approach(self.assist, f32::from(u8::from(assist)), dt, self.crossfade);
        self.limiter = approach(self.limiter, f32::from(u8::from(limiter)), dt, self.crossfade);

        self.traffic.update(dt, v);
        let steps = self.traffic.blind_spot();
        for (fade, step) in self.proximity.iter_mut().zip(steps) {
            fade.update(step, dt, self.crossfade);
        }
    }

    /// True while something moves without new telemetry (scrolling dashes, traffic, springs,
    /// crossfades), so the host keeps rendering between packets.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.speed > 0.05
            || !self.kappa.settled(self.kappa_target, 1e-5)
            || !self.yaw.settled(self.yaw_target, 1e-4)
            || !self.brake.settled(self.brake_target, 1e-3)
            || !self.light.settled(self.light_target, 1e-3)
            || self.proximity.iter().any(StepFade::running)
            || [(self.drift, 0.0), (self.assist, 0.0), (self.limiter, 0.0)]
                .iter()
                .any(|&(v, _)| v > 0.0 && v < 1.0)
    }

    /// The displayed road curvature in 1/m (positive bends right).
    #[must_use]
    pub fn curvature(&self) -> f32 {
        self.kappa.value()
    }

    /// The gap (metres, bumper to bumper) to the in-path lead car, if there is one.
    #[must_use]
    pub fn lead_gap(&self) -> Option<f32> {
        let lead = self.traffic.users()[self.traffic.lead()?];
        Some((lead.z - EGO_Z - 0.5 * (lead.kind.length() + CarKind::Coupe.length())).max(0.0))
    }

    /// The target proximity step (0 clear ..= 3 alongside) on the left and the right.
    #[must_use]
    pub fn blind_spot(&self) -> [u8; 2] {
        self.traffic.blind_spot()
    }

    /// Number of simulated road users.
    #[must_use]
    pub fn road_users(&self) -> usize {
        self.traffic.count
    }

    /// Draws the scene into `viewport` (local px; about 3.4:1, e.g. the family's scene band
    /// `y -50..240`). Lines and the carpet dissolve before the viewport's edges and cars are
    /// clipped to it, so nothing is drawn outside.
    pub fn draw(&self, p: &mut Painter<'_>, viewport: Rect, palette: &Palette) {
        if !(viewport.width() >= 64.0 && viewport.height() >= 32.0) {
            return;
        }
        let s = Scene3::new(viewport, self.kappa.value());
        self.draw_carpet(p, &s, palette);
        self.draw_lanes(p, &s, palette);
        self.draw_proximity(p, &s, palette);
        p.scoped(|p| {
            p.clip(viewport, 0.0);
            self.draw_cars(p, &s, palette);
        });
    }

    /// The path carpet: one polygon with a depth gradient, two tones split at the braking point.
    fn draw_carpet(&self, p: &mut Painter<'_>, s: &Scene3, pal: &Palette) {
        let mut pts = [Vec2::ZERO; 2 * CARPET_SAMPLES];
        let mut n = 0;
        for i in 0..CARPET_SAMPLES {
            let z = s.sample_z(i, CARPET_SAMPLES);
            if let Some(q) = s.lane_point(-CARPET_HALF_WIDTH, z) {
                pts[n] = q;
                n += 1;
            }
        }
        for i in (0..CARPET_SAMPLES).rev() {
            let z = s.sample_z(i, CARPET_SAMPLES);
            if let Some(q) = s.lane_point(CARPET_HALF_WIDTH, z) {
                pts[n] = q;
                n += 1;
            }
        }
        if n < 3 {
            return;
        }
        let tone = pal.primary.lerp(pal.roles.caution, self.drift.clamp(0.0, 1.0));
        let z_brake = EGO_Z + braking_distance(self.speed);
        let alpha = |z: f32| {
            let t = ((z - EGO_Z) / (Z_FAR - EGO_Z)).clamp(0.0, 1.0);
            let step = if z > z_brake { BRAKE_ZONE_TONE } else { 1.0 };
            CARPET_NEAR_ALPHA * (1.0 - t).powf(1.5) * step
        };
        let (y_near, y_far) = (s.ground_y(s.z_near), s.ground_y(Z_FAR));
        let span = y_far - y_near;
        if span.abs() < 1.0 {
            return;
        }
        let offset = |z: f32| (s.ground_y(z) - y_near) / span;
        // Stops along the depth: dissolve from the viewport's bottom edge, full at the ego, the
        // tone step at the braking point, zero at the far end (8 stops at most).
        let depths = [EGO_Z - 1.6, EGO_Z + 8.0, EGO_Z + 22.0, EGO_Z + 42.0, Z_FAR];
        let mut grad = Gradient::linear(vec2(0.0, y_near), vec2(0.0, y_far)).stop(0.0, tone.with_alpha(0.0));
        let mut split = !(z_brake > s.z_near && z_brake < Z_FAR);
        for z in depths {
            if !split && z_brake < z {
                let a = alpha(z_brake - 1e-3);
                grad = grad
                    .stop(offset(z_brake), tone.with_alpha(a))
                    .stop(offset(z_brake), tone.with_alpha(a * BRAKE_ZONE_TONE));
                split = true;
            }
            if z > s.z_near {
                let a = if z >= Z_FAR { 0.0 } else { alpha(z) };
                grad = grad.stop(offset(z), tone.with_alpha(a));
            }
        }
        p.fill(&Shape::Polygon { points: &pts[..n] }, grad);
    }

    /// Solid outer edges and dashed inner lines, perspective-tapered and dissolving.
    fn draw_lanes(&self, p: &mut Painter<'_>, s: &Scene3, pal: &Palette) {
        let ink = pal.roles.ink;
        for side in [-1.5f32, 1.5] {
            let offset = side * LANE_WIDTH;
            let mut prev: Option<(Vec2, f32)> = None;
            for i in 0..LINE_SAMPLES {
                let z = s.sample_z(i, LINE_SAMPLES);
                let Some(q) = s.lane_point(offset, z) else {
                    prev = None;
                    continue;
                };
                if let Some((a, za)) = prev {
                    let k = EDGE_ALPHA * depth_fade(0.5 * (za + z)) * s.edge_fade((a + q) * 0.5);
                    if k > 0.004 {
                        let line = Shape::Line {
                            a,
                            b: q,
                            width_a: s.line_width(EDGE_PX, za),
                            width_b: s.line_width(EDGE_PX, z),
                            cap: fh6_kit::Cap::Butt,
                        };
                        p.fill(&line, ink.fade(k));
                    }
                }
                prev = Some((q, z));
            }
        }
        // World-fixed dashes at multiples of the period, shifted by the distance travelled.
        let base = s.z_near - DASH_LEN;
        let first = base + (-self.travelled - base).rem_euclid(DASH_PERIOD);
        for side in [-0.5f32, 0.5] {
            let offset = side * LANE_WIDTH;
            let mut z0 = first;
            while z0 < Z_FAR {
                let (za, zb) = (z0.max(s.z_near), (z0 + DASH_LEN).min(Z_FAR));
                z0 += DASH_PERIOD;
                if zb <= za + 0.05 {
                    continue;
                }
                let (Some(a), Some(b)) = (s.lane_point(offset, za), s.lane_point(offset, zb)) else {
                    continue;
                };
                let k = DASH_ALPHA * depth_fade(0.5 * (za + zb)) * s.edge_fade((a + b) * 0.5);
                if k > 0.004 {
                    let dash = Shape::Line {
                        a,
                        b,
                        width_a: s.line_width(DASH_PX, za),
                        width_b: s.line_width(DASH_PX, zb),
                        cap: fh6_kit::Cap::Butt,
                    };
                    p.fill(&dash, ink.fade(k));
                }
            }
        }
    }

    /// 3-step blind-spot arcs on the ground beside the ego (one arc per step), crossfaded.
    fn draw_proximity(&self, p: &mut Painter<'_>, s: &Scene3, pal: &Palette) {
        let colors = [pal.accent_text, pal.roles.caution, pal.roles.warning];
        let ego_x = s.road.center_x(EGO_Z);
        for (fade, side) in self.proximity.iter().zip([-1.0f32, 1.0]) {
            for (level, weight) in [(fade.from, 1.0 - fade.t), (fade.to, fade.t)] {
                if level == 0 || weight <= 0.004 {
                    continue;
                }
                let color = colors[usize::from(level.min(3)) - 1];
                for ring in 0..level {
                    let radius = 1.75 + 0.42 * f32::from(ring);
                    let mut pts = [Vec2::ZERO; 15];
                    let mut n = 0;
                    for (i, slot) in pts.iter_mut().enumerate() {
                        let a = (-38.0 + 76.0 * i as f32 / 14.0).to_radians();
                        let world = vec3(ego_x + side * radius * a.cos(), 0.02, EGO_Z + radius * a.sin());
                        if let Some(q) = s.cam.project(world) {
                            *slot = q;
                            n += 1;
                        }
                    }
                    if n < pts.len() {
                        continue;
                    }
                    let ring_k = 1.0 - 0.22 * f32::from(ring);
                    let k = weight * ring_k * s.edge_fade(pts[7]);
                    let arc = Shape::Polyline {
                        points: &pts,
                        width: s.line_width(3.0 + f32::from(level), EGO_Z),
                        closed: false,
                    };
                    if level == 3 {
                        p.glow(&arc, color.fade(0.55 * k), Glow::new(7.0, 1.0));
                    }
                    p.fill(&arc, color.fade(k));
                }
            }
        }
    }

    /// Traffic, the lead car's brackets and the ego, painted far to near.
    fn draw_cars(&self, p: &mut Painter<'_>, s: &Scene3, pal: &Palette) {
        const EGO: usize = MAX_ROAD_USERS;
        let mut order = [(0.0f32, EGO); MAX_ROAD_USERS + 1];
        let users = self.traffic.users();
        for (slot, (i, u)) in order.iter_mut().zip(users.iter().enumerate()) {
            *slot = (u.z, i);
        }
        let n = users.len() + 1;
        order[n - 1] = (EGO_Z, EGO);
        // Insertion sort, farthest first (at most 7 entries; no allocation).
        for i in 1..n {
            let mut j = i;
            while j > 0 && order[j - 1].0 < order[j].0 {
                order.swap(j - 1, j);
                j -= 1;
            }
        }
        let g = self.light.value();
        let light = vec3(-0.35 - 0.4 * g, 1.0, -0.6).normalize_or_zero();
        let lead = self.traffic.lead();
        let neutral = pal.roles.ink.intensity(TRAFFIC_LUMA / 0.9);
        let in_path = pal.primary_text;
        let mut lead_marker: Option<(Rect, f32)> = None;
        for &(_, id) in &order[..n] {
            if id == EGO {
                self.draw_ego(p, s, pal, light);
                continue;
            }
            let u = users[id];
            let car = s.place(f32::from(u.lane) * LANE_WIDTH, u.z, u.kind, 0.0);
            let Some(center) = s.cam.project(vec3(car.x, 0.0, car.z)) else {
                continue;
            };
            let fade = (1.0 - smoothstep(CAR_FADE[0], CAR_FADE[1], u.z)) * s.side_fade(center);
            let is_lead = lead == Some(id);
            let ghost = u.lane != 0 && u.z < EGO_Z + BLIND_SPOT_AHEAD;
            let look = if is_lead {
                Look {
                    body: in_path,
                    glass: in_path.intensity(0.30),
                    alpha: fade,
                    rim: Color::TRANSPARENT,
                }
            } else {
                // A blind-spot car is a ghost body that appears once it is beside the ego.
                let ghost_k = if ghost {
                    GHOST_ALPHA * smoothstep(EGO_Z - 4.5, EGO_Z - 2.0, u.z)
                } else {
                    1.0
                };
                Look {
                    body: neutral,
                    glass: neutral.intensity(0.42),
                    alpha: fade * ghost_k,
                    rim: Color::TRANSPARENT,
                }
            };
            if look.alpha <= 0.004 {
                continue;
            }
            let bounds = draw_car(p, s, &car, &look, light);
            if let (true, Some(b)) = (is_lead, bounds) {
                lead_marker = Some((b, fade));
            }
        }
        if let (Some((bounds, fade)), Some(gap)) = (lead_marker, self.lead_gap()) {
            draw_lead_marker(p, bounds, gap, in_path.fade(fade));
        }
    }

    /// The ego: underglow, shaded body tinted by the brand, rim, brake lights.
    fn draw_ego(&self, p: &mut Painter<'_>, s: &Scene3, pal: &Palette, light: Vec3) {
        let ego = s.place(0.0, EGO_Z, CarKind::Coupe, self.yaw.value());
        // Underglow: a Gaussian of the footprint's distance, sigma 30% of the car's width.
        let mut foot = [Vec2::ZERO; 4];
        for (slot, v) in foot.iter_mut().zip(ego.footprint(0.06)) {
            match s.cam.project(v) {
                Some(q) => *slot = q,
                None => return,
            }
        }
        let width_px = s.cam.fx * ego.body.width / s.ego_depth;
        let glow_color = pal
            .primary
            .lerp(pal.roles.caution, self.assist.clamp(0.0, 1.0))
            .lerp(pal.shift, self.limiter.clamp(0.0, 1.0));
        p.glow(
            &Shape::Polygon { points: &foot },
            glow_color.fade(0.62),
            Glow {
                sigma: UNDERGLOW_SIGMA * width_px,
                intensity: 1.0,
                fill_inside: true,
            },
        );
        let body = pal.roles.ink.lerp(pal.primary, EGO_TINT);
        let look = Look {
            body,
            glass: body.intensity(0.24),
            alpha: 1.0,
            rim: pal.roles.rim_specular,
        };
        if draw_car(p, s, &ego, &look, light).is_none() {
            return;
        }
        self.draw_brake_lights(p, s, &ego, pal);
    }

    /// A full-width tail-light bar across the ego's rear: a dim running light, glowing when braking.
    fn draw_brake_lights(&self, p: &mut Painter<'_>, s: &Scene3, ego: &Placed, pal: &Palette) {
        let hull = ego.hulls()[0];
        let (n, c) = face_normal(&hull, &FACES[REAR]);
        if n.dot(s.cam.eye - c) <= 0.0 {
            return;
        }
        let b = ego.body;
        let belt = b.height * b.shoulder;
        let zr = -b.length * 0.5 * 0.99 - 0.01;
        let xw = b.width * 0.5 * 0.86;
        let corners = [
            ego.to_world(-xw, belt - 0.13, zr),
            ego.to_world(xw, belt - 0.13, zr),
            ego.to_world(xw, belt - 0.05, zr),
            ego.to_world(-xw, belt - 0.05, zr),
        ];
        let mut q = [Vec2::ZERO; 4];
        for (slot, v) in q.iter_mut().zip(corners) {
            match s.cam.project(v) {
                Some(x) => *slot = x,
                None => return,
            }
        }
        let brake = self.brake.value().clamp(0.0, 1.0);
        let bar = Shape::Polygon { points: &q };
        if brake > 0.02 {
            p.glow(
                &bar,
                pal.roles.warning.fade(0.9 * brake),
                Glow {
                    sigma: 4.0 + 12.0 * brake,
                    intensity: 1.0,
                    fill_inside: false,
                },
            );
        }
        p.fill(&bar, pal.roles.warning.intensity(0.30 + 0.70 * brake));
    }
}

/// Paints a car's visible faces (lower body, then cabin) shaded by `light`; returns its screen
/// bounds, or `None` if any corner is behind the near plane.
fn draw_car(p: &mut Painter<'_>, s: &Scene3, car: &Placed, look: &Look, light: Vec3) -> Option<Rect> {
    let hulls = car.hulls();
    let mut pts = [[Vec2::ZERO; 8]; 2];
    for (out, hull) in pts.iter_mut().zip(&hulls) {
        for (slot, v) in out.iter_mut().zip(hull) {
            *slot = s.cam.project(*v)?;
        }
    }
    let mut bounds = Rect::from_points(pts[0][0], pts[0][0]);
    for q in pts.iter().flatten() {
        bounds = bounds.union(&Rect::from_points(*q, *q));
    }
    for (h, hull) in hulls.iter().enumerate() {
        for (f, face) in FACES.iter().enumerate() {
            let (n, c) = face_normal(hull, face);
            if n.dot(s.cam.eye - c) <= 0.0 {
                continue;
            }
            let shade = AMBIENT + DIFFUSE * n.dot(light).max(0.0);
            let glass = h == 1 && f != ROOF;
            let base = if glass { look.glass } else { look.body };
            let quad = face.map(|i| pts[h][i]);
            let shape = Shape::Polygon { points: &quad };
            p.fill(&shape, base.intensity(shade).fade(look.alpha));
            if f == ROOF && look.rim.a > 0.0 {
                let lit = n.dot(light).max(0.0);
                p.stroke(&shape, look.rim.fade((0.18 + 0.32 * lit) * look.alpha), Stroke::rim_px(1.0));
            }
        }
    }
    Some(bounds)
}

/// FSD corner brackets around the lead car and its distance in the aviation face.
fn draw_lead_marker(p: &mut Painter<'_>, bounds: Rect, gap_m: f32, color: Color) {
    let r = bounds.expand(7.0);
    let arm = (r.width().min(r.height()) * 0.32).clamp(7.0, 22.0);
    let width = 2.5;
    let corners = [
        (r.min, 1.0f32, 1.0f32),
        (vec2(r.max.x, r.min.y), -1.0, 1.0),
        (vec2(r.min.x, r.max.y), 1.0, -1.0),
        (r.max, -1.0, -1.0),
    ];
    for (c, dx, dy) in corners {
        let pts = [vec2(c.x + dx * arm, c.y), c, vec2(c.x, c.y + dy * arm)];
        p.fill(
            &Shape::Polyline {
                points: &pts,
                width,
                closed: false,
            },
            color,
        );
    }
    let font = p.fonts().role(FontRole::Aviation);
    let cap = p.tokens().type_scale.unit;
    let style = TextStyle::new(font, cap)
        .paint(color)
        .align(Align::Left, VAlign::CapMiddle);
    let metres = finite_or(gap_m, 0.0).round().clamp(0.0, 999.0) as i64;
    let cells = if metres >= 100 {
        3
    } else if metres >= 10 {
        2
    } else {
        1
    };
    let pos = vec2(r.max.x + cap * 0.5, r.min.y + cap * 0.1);
    let set = p.number(pos, metres, cells, &style, None);
    p.text(vec2(set.max.x + cap * 0.3, pos.y), "m", &style);
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn vp() -> Rect {
        Rect::from_xywh(-500.0, -50.0, 1000.0, 290.0)
    }

    #[test]
    fn curvature_uses_the_speed_floor_and_clamps() {
        assert!((curvature(0.3, 30.0) - 0.01).abs() < 1e-6);
        // Below 3 m/s the speed floor keeps the curvature finite.
        assert!((curvature(0.3, 1.0) - 0.1).abs() < 1e-6);
        assert_eq!(curvature(5.0, 0.0), MAX_KAPPA);
        assert_eq!(curvature(-5.0, 0.0), -MAX_KAPPA);
        assert_eq!(curvature(f32::NAN, 20.0), 0.0);
    }

    #[test]
    fn braking_distance_is_v_squared_over_18() {
        assert!((braking_distance(30.0) - 50.0).abs() < 1e-4);
        assert_eq!(braking_distance(-4.0), 0.0);
    }

    #[test]
    fn ego_yaw_points_into_the_bend_whatever_the_slip_sign() {
        let right = ego_yaw(0.02, 25.0);
        assert!((right - 25f32.to_radians()).abs() < 1e-6);
        assert_eq!(ego_yaw(0.02, -25.0), right, "slip sign ignored in a bend");
        assert!(ego_yaw(-0.02, 25.0) < 0.0, "left bend yaws left");
        assert!((ego_yaw(0.0, 90.0).abs() - MAX_EGO_YAW_DEG.to_radians()).abs() < 1e-6, "clamped");
        assert!(ego_yaw(0.0, 10.0) < 0.0, "straight: nose against the slide");
    }

    #[test]
    fn camera_matches_the_tokens() {
        let vp = vp();
        let cam = AdasRoad::camera(&vp);
        assert_eq!(cam.eye, EYE);
        assert!((cam.pitch - PITCH_DEG.to_radians()).abs() < 1e-6);
        // The viewport spans the vertical field of view around the optical axis.
        let half = (0.5 * vp.height() / cam.fy).atan().to_degrees();
        assert!((2.0 * half - VERTICAL_FOV_DEG).abs() < 1e-3, "fov {}", 2.0 * half);
        assert!((cam.optical_center_y() - vp.center().y).abs() < 1e-3);
        // The ego sits in the bottom 40% of the band, the horizon in the upper part.
        let ego = cam.project(vec3(0.0, 0.0, EGO_Z)).expect("ego ground");
        let roof = cam.project(vec3(0.0, CarKind::Coupe.body().height, EGO_Z)).expect("ego roof");
        assert!(roof.y > vp.min.y + 0.6 * vp.height(), "ego roof {roof:?}");
        assert!(ego.y < vp.max.y);
        assert!(cam.horizon_y > vp.min.y && cam.horizon_y < vp.center().y);
        // A lead car 20 m ahead is fully above the ego's roof.
        let lead_base = cam.project(vec3(0.0, 0.0, EGO_Z + 20.0)).expect("lead");
        assert!(lead_base.y < roof.y, "lead {lead_base:?} above roof {roof:?}");
    }

    #[test]
    fn ground_rows_invert_the_projection() {
        let cam = AdasRoad::camera(&vp());
        for z in [1.5f32, EGO_Z, 12.0, 40.0] {
            let y = cam.project(vec3(0.0, 0.0, z)).expect("ground").y;
            assert!((ground_z_at(&cam, y) - z).abs() < 1e-2, "z {z}");
        }
        assert_eq!(ground_z_at(&cam, cam.horizon_y - 10.0), Z_FAR);
        let s = Scene3::new(vp(), 0.0);
        assert!(s.z_near > EYE.z && s.z_near < EGO_Z, "bottom row {}", s.z_near);
        assert!(s.line_width(2.0, EGO_Z) > s.line_width(2.0, 40.0), "lines taper with depth");
    }

    #[test]
    fn fades_dissolve_at_the_edges() {
        let s = Scene3::new(vp(), 0.0);
        assert_eq!(s.edge_fade(vec2(0.0, 100.0)), 1.0);
        assert_eq!(s.edge_fade(vec2(-500.0, 100.0)), 0.0);
        assert_eq!(s.edge_fade(vec2(0.0, 240.0)), 0.0);
        assert!(depth_fade(10.0) > 0.99 && depth_fade(Z_FAR) == 0.0);
    }

    #[test]
    fn traffic_is_deterministic_and_bounded() {
        let a = Traffic::seeded(7, 6);
        let b = Traffic::seeded(7, 6);
        assert_eq!(a, b);
        assert_ne!(Traffic::seeded(8, 6), a);
        assert_eq!(Traffic::seeded(7, 40).count, MAX_ROAD_USERS);
        assert_eq!(Traffic::seeded(7, 0).lead(), None);
        let mut t = a;
        let mut u = b;
        for _ in 0..6000 {
            t.update(DT, 30.0);
            u.update(DT, 30.0);
            for car in t.users() {
                match car.lane {
                    0 => assert!((LEAD_NEAR..=LEAD_FAR).contains(&car.z), "lead {}", car.z),
                    _ => assert!((WRAP_NEAR..=WRAP_FAR).contains(&car.z), "wrap {}", car.z),
                }
            }
            // Same-lane cars never overlap.
            for (i, x) in t.users().iter().enumerate() {
                for y in &t.users()[i + 1..] {
                    if x.lane == y.lane {
                        let d = (x.z - y.z).abs();
                        assert!(d > 5.0 && d < WRAP_FAR - WRAP_NEAR - 5.0, "overlap {x:?} {y:?}");
                    }
                }
            }
        }
        assert_eq!(t, u, "same inputs, same traffic");
        assert!(t.lead().is_some());
    }

    #[test]
    fn parked_ego_parks_the_traffic() {
        let mut t = Traffic::seeded(3, 6);
        let before = t;
        for _ in 0..120 {
            t.update(DT, 0.0);
        }
        assert_eq!(t, before);
    }

    #[test]
    fn proximity_steps() {
        assert_eq!(proximity_step(0.0), 3);
        assert_eq!(proximity_step(-2.9), 3);
        assert_eq!(proximity_step(5.0), 2);
        assert_eq!(proximity_step(-8.0), 1);
        assert_eq!(proximity_step(12.0), 0);
        assert_eq!(proximity_step(f32::NAN), 0);
    }

    #[test]
    fn step_fade_crossfades_in_150_ms() {
        let mut f = StepFade::IDLE;
        f.update(2, DT, 0.15);
        assert_eq!((f.from, f.to), (0, 2));
        assert!(f.running());
        for _ in 0..8 {
            f.update(2, DT, 0.15);
        }
        assert!(!f.running(), "done after 150 ms");
        f.update(3, DT, 0.15);
        f.update(1, DT, 0.15);
        assert_eq!((f.from, f.to), (2, 1), "retarget early keeps the shown step");
    }

    #[test]
    fn update_scrolls_dashes_and_settles() {
        let t = Tokens::default();
        let mut road = AdasRoad::new(&t, 11);
        let cruise = AdasInput {
            speed_ms: 30.0,
            yaw_rate: 0.12,
            ..AdasInput::default()
        };
        road.update(&cruise, 0.1);
        assert!((road.travelled - 3.0).abs() < 1e-4);
        assert!(road.animating());
        for _ in 0..120 {
            road.update(&cruise, DT);
        }
        assert!((road.curvature() - 0.004).abs() < 1e-4, "kappa {}", road.curvature());
        assert!(road.travelled < DASH_PERIOD);

        let mut parked = AdasRoad::new(&t, 11).with_road_users(0);
        for _ in 0..240 {
            parked.update(&AdasInput::default(), DT);
        }
        assert!(!parked.animating(), "a parked, settled scene stops rendering");
        assert_eq!(parked.lead_gap(), None);
    }

    #[test]
    fn drift_turns_the_carpet_and_the_glow_crossfades() {
        let t = Tokens::default();
        let mut road = AdasRoad::new(&t, 1);
        let drift = AdasInput {
            speed_ms: 20.0,
            yaw_rate: 0.5,
            slip_deg: -27.0,
            brake: 0.8,
            underglow: Underglow::Assist,
            ..AdasInput::default()
        };
        road.update(&drift, DT);
        assert!(road.drift > 0.0 && road.drift < 0.2);
        for _ in 0..12 {
            road.update(&drift, DT);
        }
        assert_eq!(road.drift, 1.0);
        assert_eq!(road.assist, 1.0);
        assert_eq!(road.limiter, 0.0);
        for _ in 0..60 {
            road.update(&drift, DT);
        }
        assert!(road.yaw.value() > 0.4, "nose into the right-hand bend: {}", road.yaw.value());
        assert!(road.brake.value() > 0.75);
    }

    #[test]
    fn from_signals_maps_the_limiter() {
        let sig = Signals {
            speed_ms: 40.0,
            yaw_rate: 0.1,
            slip_deg: 3.0,
            brake: 0.2,
            shift_fraction: 1.01,
            ..Signals::default()
        };
        let input = AdasInput::from_signals(&sig);
        assert_eq!(input.underglow, Underglow::Limiter);
        assert_eq!(input.speed_ms, 40.0);
        assert_eq!(input.with_underglow(Underglow::Assist).underglow, Underglow::Assist);
    }

    #[test]
    fn hull_faces_point_outwards() {
        let car = Placed {
            x: 0.0,
            z: 10.0,
            heading: 0.3,
            body: CarKind::Sedan.body(),
        };
        let [lower, _] = car.hulls();
        let (roof, _) = face_normal(&lower, &FACES[ROOF]);
        let (rear, _) = face_normal(&lower, &FACES[REAR]);
        assert!(roof.y > 0.9, "{roof:?}");
        assert!(rear.z < -0.8 && rear.x < 0.0, "rear turned with the heading: {rear:?}");
    }
}
