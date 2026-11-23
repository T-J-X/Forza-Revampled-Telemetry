//! The Tesla ADAS road scene (zone Z4): a chase-cam diorama of the predicted road with the
//! Phase 6 glTF clay cars drawn by the kit's mesh pass.
//!
//! It follows the family's scene rules (`docs/design/families/adas-hud.md`) and reuses the shared
//! model in `common::adas_road` (the curvature guard, the drift yaw, the camera constants, lane
//! width and ego depth). It is a private module rather than `AdasRoad` itself because the brief
//! needs what that component does not offer yet: glTF meshes, number-free lead brackets, the
//! measured brake stop line, the ego's dive and roll, and the upshift wake. The handover asks for
//! these to be folded back into `common::adas_road`.
//!
//! **Honesty.** The packet has no traffic, map or route data. The road ahead is predicted from the
//! path curvature `kappa = a_lat / v^2` (lateral acceleration and speed are verified against
//! `tests/golden/packets`; the angular velocity's sign is not), and the road users are a seeded
//! ambient simulation that carries no numbers. The stop line is real: it is the stopping distance
//! at the measured deceleration.
use crate::common::adas_road::{curvature, ego_yaw, EGO_Z, LANE_WIDTH, MAX_DT, MIN_CURVE_SPEED};
use fh6_kit::math::{finite_or, smoothstep, vec3, Mat4, Vec3};
use fh6_kit::mesh::{MeshData, MeshDraw, MeshId, MeshShading};
use fh6_kit::perspective::{Camera25, RoadCurve};
use fh6_kit::{
    vec2, Align, BlendMode, Cap, Color, FontId, Glow, Gradient, ImageId, ImagePaint, Painter, Palette, Rect, Shape,
    Spring, TextStyle, Tokens, VAlign, Vec2,
};

/// Standard gravity (m/s^2).
const G: f32 = 9.80665;
/// Road users simulated (one in-path lead car and three in the adjacent lanes; the Taycan's
/// 6-object cap counts the ego and leaves headroom).
const ROAD_USERS: usize = 4;
/// Carpet alpha at the ego and the tone beyond the stop point (family spec: 0.55, 35%).
const CARPET_NEAR_ALPHA: f32 = 0.40;
const BRAKE_ZONE_TONE: f32 = 0.35;
/// Carpet half width (metres): the ego's track plus a hand's width.
const CARPET_HALF_WIDTH: f32 = 1.05;
/// Points per carpet side.
const CARPET_SAMPLES: usize = 28;
/// Samples along each solid edge line.
const LINE_SAMPLES: usize = 30;
/// Far end of the drawn road (metres) on a straight.
const Z_FAR: f32 = 85.0;
/// The prediction stops where the road's heading reaches `atan(1.2)`, never nearer than 24 m.
const REACH_HEADING_TAN: f32 = 1.2;
const REACH_MIN: f32 = 24.0;
/// Edge and dash widths at the ego's depth (local px) and their peak alphas.
const EDGE_PX: f32 = 2.0;
const DASH_PX: f32 = 3.0;
const EDGE_ALPHA: f32 = 0.40;
const DASH_ALPHA: f32 = 0.58;
/// Dashes: 3 m lines every 12 m.
const DASH_LEN: f32 = 3.0;
const DASH_PERIOD: f32 = 12.0;
/// Side dissolve (local px, the brief's 120 px masks), bottom and top dissolves.
const FADE_EDGES_PX: f32 = 120.0;
const BOTTOM_FADE_PX: f32 = 40.0;
const TOP_FADE_PX: f32 = 18.0;
/// Height of the dissolve below the horizon as a fraction of the band.
const HORIZON_FADE_FRAC: f32 = 0.18;
/// Ground plane alpha at the ego (linear ink; about 6% luminance on screen).
const GROUND_ALPHA: f32 = 0.006;
/// Underglow alpha and its sigma as a fraction of the ego's on-screen width.
const UNDERGLOW_ALPHA: f32 = 0.40;
const UNDERGLOW_SIGMA: f32 = 0.22;
/// The baked underglow pool: its size in local px (asset-forge `tesla_adas::POOL_LOCAL`, centred on
/// the ego's ground centre) and its peak alpha.
const POOL_LOCAL: [f32; 2] = [480.0, 240.0];
const POOL_ALPHA: f32 = 0.75;
/// The pool atlas: 11 yaw cells from -50 to +50 degrees, 4 per row (asset-forge `POOL_YAWS`).
const POOL_CELLS: usize = 11;
const POOL_COLS: usize = 4;
const POOL_YAW_MIN: f32 = -50.0;
const POOL_YAW_STEP: f32 = 10.0;
/// The ComfyUI asphalt plate's light level (linear ink; about #0E1116 where the plate is full).
const ASPHALT_LEVEL: f32 = 0.010;
/// The road fill reaches this far past the outer edge lines (metres).
const ROAD_SHOULDER: f32 = 0.6;
/// Traffic contact shadows: a ground ellipse 1.1x the car's width and 0.95x its length, 50% black.
const SHADOW_SCALE: [f32; 2] = [1.1, 0.95];
const SHADOW_ALPHA: f32 = 0.5;
/// Ego body: light neutral tinted 15% by the brand (the brand-pearl twin).
const EGO_TINT: f32 = 0.15;
/// Traffic body (`#8C929C` at 0.70 over black, drawn opaque) and a blind-spot ghost's level.
const TRAFFIC_RGB: u32 = 0x8A_8E_96;
const TRAFFIC_LEVEL: f32 = 1.0;
const GHOST_LEVEL: f32 = 0.30;
/// An adjacent-lane car nearer than `EGO_Z + 6 m` is a ghost, solid over the next 4 m.
const BLIND_SPOT_AHEAD: f32 = 6.0;
const GHOST_RAMP: f32 = 4.0;
/// Longitudinal gaps (metres) of the 3 proximity steps.
const PROXIMITY_STEPS_M: [f32; 3] = [10.0, 6.0, 3.0];
/// Drift: on at 10 degrees and 5 m/s, off below 7 degrees or 4 m/s.
const DRIFT_ON: (f32, f32) = (10.0, 5.0);
const DRIFT_OFF: (f32, f32) = (7.0, 4.0);
/// Braking: the pedal threshold of the brake-dive moment and the decel floor of the stop line.
const BRAKE_ON: f32 = 0.05;
const STOP_DECEL_MIN: f32 = 2.0;
/// The reference deceleration of the carpet's tone step when not braking (COMPONENTS §10).
const REFERENCE_DECEL: f32 = 9.0;
/// The stop line and its label are hidden beyond this distance (metres).
const STOP_MAX_M: f32 = 120.0;
/// Dive: degrees of nose-down pitch per g of deceleration (the brief's x3 on a typical 1.3
/// degrees/g body pitch), clamped to 4 degrees down and 1.5 degrees of squat.
const DIVE_DEG_PER_G: f32 = 4.0;
const DIVE_MAX_DEG: f32 = 4.0;
const SQUAT_MAX_DEG: f32 = 1.5;
/// Roll: degrees per g of lateral acceleration, clamped.
const ROLL_DEG_PER_G: f32 = 2.5;
const ROLL_MAX_DEG: f32 = 3.0;
/// Upshift wake: duration (s) and band length (fraction of the carpet on screen).
const WAKE_S: f32 = 0.25;
const WAKE_BAND: f32 = 0.40;
/// Ambient traffic loop of the adjacent lanes (z metres) and the lead car's range.
const WRAP_NEAR: f32 = -14.0;
const WRAP_FAR: f32 = 78.0;
const LEAD_NEAR: f32 = EGO_Z + 14.0;
const LEAD_FAR: f32 = 58.0;
/// Ego speed (m/s) at which traffic moves at its full relative speed.
const TRAFFIC_FULL_SPEED: f32 = 12.0;
/// Seed of the ambient traffic (fixed, so mockups and goldens are reproducible).
pub const SEED: u64 = 0x7E57_A0A5;

/// One view's scene band and camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneGeom {
    /// The band (local px); the mesh pass and every draw are clipped to it.
    pub viewport: Rect,
    /// Camera eye (metres).
    pub eye: Vec3,
    /// Downward pitch (degrees).
    pub pitch_deg: f32,
    /// Focal length (local px per metre at 1 m).
    pub focal: f32,
    /// Horizon as a fraction of the band's height from its top.
    pub horizon_frac: f32,
    /// Which baked underglow pool matches this camera (0 cockpit, 1 hood; `assets/clusters/tesla_adas/`).
    pub pool: usize,
}

impl SceneGeom {
    /// The chase camera for this band.
    #[must_use]
    pub fn camera(&self) -> Camera25 {
        let vp = self.viewport;
        Camera25::chase(
            self.eye,
            self.pitch_deg.to_radians(),
            self.focal,
            self.focal,
            vp.center().x,
            vp.min.y + self.horizon_frac * vp.height(),
        )
    }
}

/// The four car models (ego coupe and three traffic bodies).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ego,
    Sedan,
    Suv,
    Supercar,
}

impl Kind {
    const TRAFFIC: [Kind; 3] = [Kind::Sedan, Kind::Suv, Kind::Supercar];

    /// Asset file and dimensions `[width, length, height]` (metres, `assets/ASSETS.md`).
    fn spec(self) -> (&'static str, [f32; 3]) {
        match self {
            Kind::Ego => ("adas/ego_coupe.glb", [1.92, 4.55, 1.33]),
            Kind::Sedan => ("adas/sport_sedan.glb", [1.87, 4.85, 1.47]),
            Kind::Suv => ("adas/suv.glb", [1.96, 4.85, 1.77]),
            Kind::Supercar => ("adas/supercar.glb", [2.02, 4.62, 1.19]),
        }
    }

    fn index(self) -> usize {
        match self {
            Kind::Ego => 0,
            Kind::Sedan => 1,
            Kind::Suv => 2,
            Kind::Supercar => 3,
        }
    }
}

/// An uploaded car model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CarModel {
    mesh: MeshId,
    /// `[width, length, height]` in metres.
    dims: [f32; 3],
    /// True when the Phase 6 glTF loaded (false: the kit's procedural proxy).
    pub gltf: bool,
}

/// Everything the scene loads once: the car models (the Phase 6 glTF when `assets` has it, else
/// the kit's procedural proxy), the tail-light quad, and this cluster's baked images (the Blender
/// underglow pool atlases per camera and the ComfyUI asphalt plate; each `None` falls back to the
/// procedural look).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneAssets {
    models: [CarModel; 4],
    lamp: MeshId,
    pools: [Option<ImageId>; 2],
    asphalt: Option<ImageId>,
}

/// Loads [`SceneAssets`].
pub fn load_assets(setup: &mut fh6_kit::cluster::Setup<'_>) -> SceneAssets {
    let kinds = [Kind::Ego, Kind::Sedan, Kind::Suv, Kind::Supercar];
    let models = kinds.map(|k| {
        let (file, dims) = k.spec();
        let glb = setup
            .assets
            .map(|a| a.join(file))
            .and_then(|path| std::fs::read(path).ok())
            .and_then(|bytes| MeshData::from_glb(&bytes).ok());
        let gltf = glb.is_some();
        let data = glb.unwrap_or_else(|| MeshData::car_proxy(dims[1], dims[0], dims[2]));
        CarModel {
            mesh: setup.meshes.upload(setup.device, setup.queue, &data),
            dims,
            gltf,
        }
    });
    let lamp = setup.meshes.upload(setup.device, setup.queue, &lamp_quad());
    let mut image = |rel: &str| {
        let path = setup.assets?.join(rel);
        path.is_file()
            .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
            .flatten()
    };
    SceneAssets {
        models,
        lamp,
        pools: [
            image("clusters/tesla_adas/underglow_pool_cockpit.png"),
            image("clusters/tesla_adas/underglow_pool_hood.png"),
        ],
        asphalt: image("clusters/tesla_adas/asphalt.png"),
    }
}

/// What the scene reads each frame (verified telemetry only).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SceneInput {
    /// Speed (m/s, a magnitude).
    pub speed_ms: f32,
    /// Lateral acceleration (m/s^2, positive to the right).
    pub a_lat: f32,
    /// Longitudinal acceleration (m/s^2, positive accelerating).
    pub a_long: f32,
    /// Body slip angle (degrees; only its magnitude is used).
    pub slip_deg: f32,
    /// Brake pedal 0..=1.
    pub brake: f32,
    /// Reverse gear.
    pub reverse: bool,
    /// Steady limiter cue 0..=1 (the underglow turns the shift colour).
    pub limiter: f32,
}

/// One ambient road user.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RoadUser {
    /// -1 left, 0 in path, 1 right.
    lane: i8,
    z: f32,
    kind: Kind,
}

/// SplitMix64, for seeding the traffic.
#[derive(Debug, Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }

    fn kind(&mut self) -> Kind {
        Kind::TRAFFIC[(self.next() % 3) as usize]
    }
}

/// The seeded traffic: a lead car hovering ahead in the path, one car in one adjacent lane and
/// two in the other, cruising at per-lane relative speeds and wrapping round.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Traffic {
    users: [RoadUser; ROAD_USERS],
    lane_speed: [f32; 3],
    lead_dir: f32,
}

impl Traffic {
    fn seeded(seed: u64) -> Self {
        let mut rng = SplitMix64(seed);
        let lane_speed = [
            0.8 + 1.6 * rng.unit(),
            0.5 + 1.0 * rng.unit(),
            -(0.8 + 1.6 * rng.unit()),
        ];
        let lead = RoadUser {
            lane: 0,
            z: LEAD_NEAR + 4.0 + (LEAD_FAR - LEAD_NEAR - 16.0) * rng.unit(),
            kind: rng.kind(),
        };
        let first: i8 = if rng.unit() < 0.5 { -1 } else { 1 };
        let span = WRAP_FAR - WRAP_NEAR;
        let base = rng.unit() * span;
        let one = RoadUser {
            lane: first,
            z: WRAP_NEAR + base,
            kind: rng.kind(),
        };
        let base2 = rng.unit() * span;
        let mut pair = [one; 2];
        for (i, u) in pair.iter_mut().enumerate() {
            *u = RoadUser {
                lane: -first,
                z: WRAP_NEAR + (base2 + i as f32 * span * 0.5).rem_euclid(span),
                kind: rng.kind(),
            };
        }
        Self {
            users: [lead, one, pair[0], pair[1]],
            lane_speed,
            lead_dir: if rng.unit() < 0.5 { -1.0 } else { 1.0 },
        }
    }

    fn update(&mut self, dt: f32, ego_speed: f32) {
        let step = (ego_speed / TRAFFIC_FULL_SPEED).clamp(0.0, 1.0) * dt;
        let span = WRAP_FAR - WRAP_NEAR;
        for u in &mut self.users {
            if u.lane == 0 {
                u.z += self.lead_dir * self.lane_speed[1] * step;
                if u.z <= LEAD_NEAR {
                    self.lead_dir = 1.0;
                } else if u.z >= LEAD_FAR {
                    self.lead_dir = -1.0;
                }
                u.z = u.z.clamp(LEAD_NEAR, LEAD_FAR);
            } else {
                let v = if u.lane < 0 {
                    self.lane_speed[0]
                } else {
                    self.lane_speed[2]
                };
                u.z = WRAP_NEAR + (u.z + v * step - WRAP_NEAR).rem_euclid(span);
            }
        }
    }

    /// Proximity step (0..=3) on the left and right.
    fn blind_spot(&self) -> [u8; 2] {
        let mut out = [0u8; 2];
        for u in &self.users {
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

/// The proximity step for a longitudinal gap (metres): 3 alongside, 0 clear.
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

/// Level of an adjacent-lane car at depth `z`: invisible behind the ego, a ghost beside it, solid
/// from `EGO_Z + 6 m` over the next 4 m (never popping).
fn adjacent_level(z: f32) -> f32 {
    let appear = smoothstep(EGO_Z - 4.5, EGO_Z - 2.0, z);
    let ahead = EGO_Z + BLIND_SPOT_AHEAD;
    let body = GHOST_LEVEL + (TRAFFIC_LEVEL - GHOST_LEVEL) * smoothstep(ahead, ahead + GHOST_RAMP, z);
    appear * body
}

/// A 150 ms crossfade between proximity steps.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StepFade {
    from: u8,
    to: u8,
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
}

/// Moves `value` towards `target`, covering 0..1 in `duration` seconds.
fn approach(value: f32, target: f32, dt: f32, duration: f32) -> f32 {
    let step = dt / duration.max(1e-3);
    value + (target - value).clamp(-step, step)
}

/// The path curvature (1/m) from lateral acceleration: `a_lat / v^2` with the shared speed floor
/// (`common::adas_road::curvature` of the equivalent yaw rate `a_lat / max(v, 3)`).
#[must_use]
pub fn path_curvature(a_lat: f32, speed_ms: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).abs();
    curvature(finite_or(a_lat, 0.0) / v.max(MIN_CURVE_SPEED), v)
}

/// The stopping distance (metres) at the measured deceleration, floored at 2 m/s^2:
/// `v^2 / (2 max(-a_long, 2))`.
#[must_use]
pub fn stop_distance(speed_ms: f32, a_long: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).abs();
    let decel = (-finite_or(a_long, 0.0)).max(STOP_DECEL_MIN);
    v * v / (2.0 * decel)
}

/// How far (metres of depth) the predicted road is drawn for a curvature.
fn reach(kappa: f32) -> f32 {
    let k = finite_or(kappa, 0.0).abs();
    if k < 1e-6 {
        return Z_FAR;
    }
    (REACH_HEADING_TAN / k).clamp(REACH_MIN, Z_FAR)
}

/// A rotation about +X by `a` radians (positive tips the nose, +Z, down).
fn rotation_x(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    Mat4 {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, c, s, 0.0],
            [0.0, -s, c, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

/// A rotation about +Z by `a` radians (positive leans the roof left, -X).
fn rotation_z(a: f32) -> Mat4 {
    let (s, c) = a.sin_cos();
    Mat4 {
        cols: [
            [c, s, 0.0, 0.0],
            [-s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

/// `m` applied to the point `v`.
fn transform(m: &Mat4, v: Vec3) -> Vec3 {
    let c = &m.cols;
    vec3(
        c[0][0] * v.x + c[1][0] * v.y + c[2][0] * v.z + c[3][0],
        c[0][1] * v.x + c[1][1] * v.y + c[2][1] * v.z + c[3][1],
        c[0][2] * v.x + c[1][2] * v.y + c[2][2] * v.z + c[3][2],
    )
}

/// Per-frame projection shared by the passes.
#[derive(Debug, Clone, Copy)]
struct Frame {
    cam: Camera25,
    road: RoadCurve,
    vp: Rect,
    z_near: f32,
    z_far: f32,
    ego_depth: f32,
    fade_px: f32,
    /// The baked underglow pool for this camera.
    pool: usize,
}

impl Frame {
    fn new(g: &SceneGeom, kappa: f32) -> Self {
        let cam = g.camera();
        let vp = g.viewport;
        // Ground depth at the band's bottom edge.
        let (s, c) = cam.pitch.sin_cos();
        let k = (vp.max.y - cam.optical_center_y()) / cam.fy.max(1e-3);
        let den = k * c + s;
        let z_near = if den > 1e-4 {
            cam.eye.z + cam.eye.y * (c - k * s) / den
        } else {
            EGO_Z
        };
        Self {
            cam,
            road: RoadCurve { kappa, tan_beta: 0.0 },
            vp,
            z_near: z_near.clamp(cam.eye.z + 1.0, EGO_Z),
            z_far: reach(kappa),
            ego_depth: cam.depth(vec3(0.0, 0.0, EGO_Z)).max(0.5),
            fade_px: FADE_EDGES_PX.min(vp.width() * 0.25),
            pool: g.pool,
        }
    }

    fn lane_point(&self, offset: f32, z: f32) -> Option<Vec2> {
        self.cam.project(vec3(self.road.center_x(z) + offset, 0.0, z))
    }

    fn ground_y(&self, z: f32) -> f32 {
        self.cam.project(vec3(0.0, 0.0, z)).map_or(self.cam.horizon_y, |q| q.y)
    }

    fn sample_z(&self, i: usize, n: usize, far: f32) -> f32 {
        let t = i as f32 / (n.max(2) - 1) as f32;
        self.z_near + (far - self.z_near) * t.powf(1.6)
    }

    fn line_width(&self, px: f32, z: f32) -> f32 {
        let d = self.cam.depth(vec3(0.0, 0.0, z)).max(0.5);
        px * (self.ego_depth / d).clamp(0.4, 1.35)
    }

    fn depth_fade(&self, z: f32) -> f32 {
        1.0 - smoothstep(self.z_far * 0.30, self.z_far, z)
    }

    fn car_fade(&self, z: f32) -> f32 {
        let r = self.z_far / Z_FAR;
        1.0 - smoothstep(50.0 * r, 76.0 * r, z)
    }

    fn side_fade(&self, q: Vec2) -> f32 {
        smoothstep(self.vp.min.x, self.vp.min.x + self.fade_px, q.x)
            * smoothstep(self.vp.max.x, self.vp.max.x - self.fade_px, q.x)
    }

    fn edge_fade(&self, q: Vec2) -> f32 {
        let horizon = self.cam.horizon_y;
        self.side_fade(q)
            * smoothstep(self.vp.max.y, self.vp.max.y - BOTTOM_FADE_PX, q.y)
            * smoothstep(self.vp.min.y, self.vp.min.y + TOP_FADE_PX, q.y)
            * smoothstep(horizon, horizon + HORIZON_FADE_FRAC * self.vp.height(), q.y)
    }

    /// The model transform of a car at lateral `offset` and depth `z` on the path.
    fn model(&self, offset: f32, z: f32, yaw: f32, lift: f32, dive: f32, roll: f32) -> Mat4 {
        let x = self.road.center_x(z) + offset;
        let heading = self.road.heading(z) + yaw;
        Mat4::translation(vec3(x, lift, z))
            .mul(&Mat4::rotation_y(heading))
            .mul(&rotation_x(dive))
            .mul(&rotation_z(roll))
    }

    /// Screen bounds of a car's bounding box under `model`.
    fn bounds(&self, model: &Mat4, dims: [f32; 3]) -> Option<Rect> {
        let (hw, hl, h) = (dims[0] * 0.5, dims[1] * 0.5, dims[2]);
        let mut out: Option<Rect> = None;
        for (x, y, z) in [
            (-hw, 0.0, -hl),
            (hw, 0.0, -hl),
            (hw, 0.0, hl),
            (-hw, 0.0, hl),
            (-hw, h, -hl),
            (hw, h, -hl),
            (hw, h, hl),
            (-hw, h, hl),
        ] {
            let q = self.cam.project(transform(model, vec3(x, y, z)))?;
            let r = Rect::from_points(q, q);
            out = Some(out.map_or(r, |o| o.union(&r)));
        }
        out
    }
}

/// The scene state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scene {
    models: [CarModel; 4],
    lamp: MeshId,
    pools: [Option<ImageId>; 2],
    asphalt: Option<ImageId>,
    traffic: Traffic,
    kappa: Spring,
    kappa_target: f32,
    yaw: Spring,
    yaw_target: f32,
    lamps: Spring,
    lamps_target: f32,
    dive: Spring,
    dive_target: f32,
    roll: Spring,
    roll_target: f32,
    light: Spring,
    light_target: f32,
    /// Depth past the ego's nose where the carpet steps down (metres; springs between the
    /// reference braking distance and the measured stop line).
    split: Spring,
    split_target: f32,
    speed: f32,
    travelled: f32,
    drifting: bool,
    drift: f32,
    limiter: f32,
    braking: bool,
    stop_alpha: f32,
    stop_m: f32,
    wake_t: f32,
    proximity: [StepFade; 2],
    crossfade: f32,
}

impl Scene {
    /// A scene with the tokens' motion and the seeded traffic.
    #[must_use]
    pub fn new(tokens: &Tokens, assets: SceneAssets) -> Self {
        let s = &tokens.springs;
        Self {
            models: assets.models,
            lamp: assets.lamp,
            pools: assets.pools,
            asphalt: assets.asphalt,
            traffic: Traffic::seeded(SEED),
            kappa: Spring::new(s.default, 0.0),
            kappa_target: 0.0,
            yaw: Spring::new(s.default, 0.0),
            yaw_target: 0.0,
            lamps: Spring::new(s.snap, 0.0),
            lamps_target: 0.0,
            dive: Spring::new(s.snap, 0.0),
            dive_target: 0.0,
            roll: Spring::new(s.default, 0.0),
            roll_target: 0.0,
            light: Spring::new(s.light, 0.0),
            light_target: 0.0,
            split: Spring::new(s.default, 0.0),
            split_target: 0.0,
            speed: 0.0,
            travelled: 0.0,
            drifting: false,
            drift: 0.0,
            limiter: 0.0,
            braking: false,
            stop_alpha: 0.0,
            stop_m: 0.0,
            wake_t: -1.0,
            proximity: [StepFade::IDLE; 2],
            crossfade: tokens.durations.crossfade,
        }
    }

    /// Starts the upshift wake up the carpet.
    pub fn wake(&mut self) {
        self.wake_t = 0.0;
    }

    /// True while a drift is shown (latched with hysteresis): the SLIP chip.
    #[must_use]
    pub fn drifting(&self) -> bool {
        self.drifting
    }

    /// Advances by `dt` seconds.
    pub fn update(&mut self, input: &SceneInput, dt: f32) {
        let dt = finite_or(dt, 0.0).clamp(0.0, MAX_DT);
        let v = finite_or(input.speed_ms, 0.0).abs();
        let dir = if input.reverse { -1.0 } else { 1.0 };
        self.speed = v;
        self.travelled = (self.travelled + dir * v * dt).rem_euclid(DASH_PERIOD);
        let a_lat = finite_or(input.a_lat, 0.0);
        let a_long = finite_or(input.a_long, 0.0);
        let slip = if input.reverse {
            0.0
        } else {
            finite_or(input.slip_deg, 0.0)
        };

        self.kappa_target = dir * path_curvature(a_lat, v);
        self.kappa.update(self.kappa_target, dt);
        self.yaw_target = ego_yaw(self.kappa_target, slip);
        self.yaw.update(self.yaw_target, dt);
        self.light_target = (a_lat / G / 1.5).clamp(-1.0, 1.0);
        self.light.update(self.light_target, dt);

        // Brake dive: the lamps light in the frame the pedal passes 5%; the body pitches by the
        // measured deceleration and rolls with the lateral load.
        let brake = finite_or(input.brake, 0.0).clamp(0.0, 1.0);
        self.braking = brake > BRAKE_ON && v > 0.5;
        self.lamps_target = if brake > BRAKE_ON { 0.6 + 0.4 * brake } else { 0.0 };
        if self.braking && self.lamps.value() < 0.6 {
            self.lamps.reset(self.lamps_target);
        }
        self.lamps.update(self.lamps_target, dt);
        self.dive_target = (-a_long / G * DIVE_DEG_PER_G).clamp(-SQUAT_MAX_DEG, DIVE_MAX_DEG);
        self.dive.update(self.dive_target, dt);
        self.roll_target = (a_lat / G * ROLL_DEG_PER_G).clamp(-ROLL_MAX_DEG, ROLL_MAX_DEG);
        self.roll.update(self.roll_target, dt);

        self.stop_m = stop_distance(v, a_long);
        let show_stop = self.braking && !input.reverse && self.stop_m <= STOP_MAX_M;
        self.stop_alpha = approach(self.stop_alpha, if show_stop { 1.0 } else { 0.0 }, dt, 0.12);
        self.split_target = if self.braking {
            self.stop_m
        } else {
            v * v / (2.0 * REFERENCE_DECEL)
        };
        self.split.update(self.split_target, dt);

        let s = slip.abs();
        self.drifting = if self.drifting {
            s >= DRIFT_OFF.0 && v >= DRIFT_OFF.1
        } else {
            s >= DRIFT_ON.0 && v >= DRIFT_ON.1
        };
        self.drift = approach(self.drift, f32::from(u8::from(self.drifting)), dt, self.crossfade);
        self.limiter = approach(self.limiter, input.limiter.clamp(0.0, 1.0), dt, self.crossfade);

        if self.wake_t >= 0.0 {
            self.wake_t += dt;
            if self.wake_t > WAKE_S {
                self.wake_t = -1.0;
            }
        }

        self.traffic.update(dt, v);
        let steps = self.traffic.blind_spot();
        for (fade, step) in self.proximity.iter_mut().zip(steps) {
            fade.update(step, dt, self.crossfade);
        }
    }

    /// True while anything moves without new telemetry.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.speed > 0.05
            || self.wake_t >= 0.0
            || !self.kappa.settled(self.kappa_target, 1e-5)
            || !self.yaw.settled(self.yaw_target, 1e-4)
            || !self.lamps.settled(self.lamps_target, 1e-3)
            || !self.dive.settled(self.dive_target, 1e-2)
            || !self.roll.settled(self.roll_target, 1e-2)
            || !self.light.settled(self.light_target, 1e-3)
            || !self.split.settled(self.split_target, 0.05)
            || self.proximity.iter().any(|f| f.t < 1.0)
            || [self.drift, self.limiter, self.stop_alpha]
                .iter()
                .any(|&v| v > 0.0 && v < 1.0)
    }

    /// Draws the scene into its band. `label` is the face of the stop-line distance.
    pub fn draw(&self, p: &mut Painter<'_>, g: &SceneGeom, pal: &Palette, label: FontId) {
        let vp = g.viewport;
        if !(vp.min.is_finite() && vp.max.is_finite() && vp.width() >= 64.0 && vp.height() >= 32.0) {
            return;
        }
        let f = Frame::new(g, self.kappa.value());
        p.scoped(|p| {
            p.clip(vp, 0.0);
            self.draw_ground(p, &f, pal);
            self.draw_carpet(p, &f, pal);
            self.draw_lanes(p, &f, pal);
            self.draw_stop_line(p, &f, pal, label);
            self.draw_proximity(p, &f, pal);
            self.draw_car_overlays(p, &f, pal);
        });
    }

    /// Ego nose position past its centre (metres).
    fn ego_nose(&self) -> f32 {
        self.models[Kind::Ego.index()].dims[1] * 0.5
    }

    /// The far end of the carpet: the road's end, or nearer where its edges enter the side fade.
    fn carpet_far(&self, f: &Frame) -> f32 {
        let min_far = (EGO_Z + 6.0).min(f.z_far);
        for i in 1..CARPET_SAMPLES {
            let z = carpet_z(i, f.z_far);
            let inside = [-CARPET_HALF_WIDTH, CARPET_HALF_WIDTH]
                .iter()
                .all(|&o| f.lane_point(o, z).is_some_and(|q| f.side_fade(q) >= 1.0));
            if !inside {
                return z.max(min_far);
            }
        }
        f.z_far
    }

    /// The carpet outline (left edge out, right edge back); returns the point count.
    fn carpet_outline(f: &Frame, far: f32, out: &mut [Vec2; 2 * CARPET_SAMPLES]) -> usize {
        let mut n = 0;
        for i in 0..CARPET_SAMPLES {
            if let Some(q) = f.lane_point(-CARPET_HALF_WIDTH, carpet_z(i, far)) {
                out[n] = q;
                n += 1;
            }
        }
        for i in (0..CARPET_SAMPLES).rev() {
            if let Some(q) = f.lane_point(CARPET_HALF_WIDTH, carpet_z(i, far)) {
                out[n] = q;
                n += 1;
            }
        }
        n
    }

    /// The road surface between the outer edges (the ComfyUI asphalt plate, near-black; or a faint
    /// gradient without it) and the underglow pool.
    fn draw_ground(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let mut pts = [Vec2::ZERO; 2 * CARPET_SAMPLES];
        let mut n = 0;
        let far = f.z_far * 0.8;
        let edge = 1.5 * LANE_WIDTH + ROAD_SHOULDER;
        for i in 0..CARPET_SAMPLES {
            if let Some(q) = f.lane_point(-edge, f.sample_z(i, CARPET_SAMPLES, far)) {
                pts[n] = q;
                n += 1;
            }
        }
        for i in (0..CARPET_SAMPLES).rev() {
            if let Some(q) = f.lane_point(edge, f.sample_z(i, CARPET_SAMPLES, far)) {
                pts[n] = q;
                n += 1;
            }
        }
        let (y0, y1) = (f.vp.max.y, f.ground_y(far));
        if n >= 3 && (y1 - y0).abs() > 1.0 {
            let road = Shape::Polygon { points: &pts[..n] };
            let ink = pal.roles.ink;
            match self.asphalt {
                // The plate's vanishing point at the horizon, its near edge at the band's bottom;
                // the road polygon cuts it to the carriageway (it is texture, lit near-black).
                Some(img) => {
                    let rect = Rect::from_points(vec2(f.vp.min.x, f.cam.horizon_y), f.vp.max);
                    p.fill(&road, ImagePaint::new(img, rect).tint(ink.intensity(ASPHALT_LEVEL)));
                }
                None => p.fill(
                    &road,
                    Gradient::linear(vec2(0.0, y0), vec2(0.0, y1))
                        .stop(0.0, ink.with_alpha(0.0))
                        .stop(0.25, ink.with_alpha(GROUND_ALPHA))
                        .stop(1.0, ink.with_alpha(0.0)),
                ),
            }
        }

        let color = pal
            .primary
            .lerp(pal.roles.caution, self.drift.clamp(0.0, 1.0))
            .lerp(pal.shift, self.limiter.clamp(0.0, 1.0));
        // Underglow, baked: the Blender light pool seen through this very camera at the nearest
        // two of 11 yaw angles (true perspective, tyre and sill shadows included), crossfaded and
        // tinted live. Light adds, so the crossfade is exact.
        if let Some(img) = self.pools[f.pool.min(1)] {
            let x = f.road.center_x(EGO_Z);
            let Some(c) = f.cam.project(vec3(x, 0.0, EGO_Z)) else {
                return;
            };
            let heading = (f.road.heading(EGO_Z) + self.yaw.value()).to_degrees();
            let pos = ((heading - POOL_YAW_MIN) / POOL_YAW_STEP).clamp(0.0, (POOL_CELLS - 1) as f32);
            let i0 = pos.floor() as usize;
            let i1 = (i0 + 1).min(POOL_CELLS - 1);
            let w1 = pos - i0 as f32;
            let rect = Rect::from_center_size(c, vec2(POOL_LOCAL[0], POOL_LOCAL[1]));
            let rows = POOL_CELLS.div_ceil(POOL_COLS);
            let (cu, cv) = (1.0 / POOL_COLS as f32, 1.0 / rows as f32);
            p.scoped(|p| {
                p.set_blend(BlendMode::Add);
                for (i, w) in [(i0, 1.0 - w1), (i1, w1)] {
                    if w <= 0.004 {
                        continue;
                    }
                    let uv = Rect::from_xywh((i % POOL_COLS) as f32 * cu, (i / POOL_COLS) as f32 * cv, cu, cv);
                    let paint = ImagePaint {
                        uv,
                        ..ImagePaint::new(img, rect).tint(color.fade(POOL_ALPHA * w))
                    };
                    p.fill(&Shape::rect(rect), paint);
                }
            });
            return;
        }
        let color = color.fade(UNDERGLOW_ALPHA);
        // Underglow fallback: a Gaussian round a ground ellipse under the ego (no bloom).
        let ego = self.models[Kind::Ego.index()];
        let model = f.model(0.0, EGO_Z, self.yaw.value(), 0.0, 0.0, 0.0);
        let (a, b) = (0.5 * ego.dims[0] * 1.22, 0.5 * ego.dims[1] * 1.10);
        let mut pool = [Vec2::ZERO; 24];
        for (i, slot) in pool.iter_mut().enumerate() {
            let t = core::f32::consts::TAU * i as f32 / 24.0;
            let Some(q) = f.cam.project(transform(&model, vec3(a * t.cos(), 0.0, b * t.sin()))) else {
                return;
            };
            *slot = q;
        }
        let width_px = f.cam.fx * ego.dims[0] / f.ego_depth;
        let (d0, d1) = (f.vp.max.y - BOTTOM_FADE_PX, f.vp.max.y);
        p.glow(
            &Shape::Polygon { points: &pool },
            Gradient::linear(vec2(0.0, d0), vec2(0.0, d1))
                .stop(0.0, color)
                .stop(1.0, color.with_alpha(0.0)),
            Glow {
                sigma: UNDERGLOW_SIGMA * width_px,
                intensity: 1.0,
                fill_inside: true,
            },
        );
    }

    /// The two-tone path carpet (primary, amber in a drift), stepping to 35% beyond the stop
    /// point, plus the upshift wake (an accent band climbing it in 250 ms, alpha only).
    fn draw_carpet(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let far = self.carpet_far(f);
        let mut pts = [Vec2::ZERO; 2 * CARPET_SAMPLES];
        let n = Self::carpet_outline(f, far, &mut pts);
        if n < 3 {
            return;
        }
        let (y_near, y_far) = (f.ground_y(EGO_Z), f.ground_y(far));
        let span = y_far - y_near;
        if span.abs() < 1.0 {
            return;
        }
        let offset = |z: f32| ((f.ground_y(z) - y_near) / span).clamp(0.0, 1.0);
        let tone = pal.primary.lerp(pal.roles.caution, self.drift.clamp(0.0, 1.0));
        let z_split = EGO_Z + self.ego_nose() + self.split.value().max(0.0);
        let alpha = |z: f32| {
            let t = ((z - EGO_Z) / (far - EGO_Z).max(1e-2)).clamp(0.0, 1.0);
            let step = if z > z_split { BRAKE_ZONE_TONE } else { 1.0 };
            CARPET_NEAR_ALPHA * (1.0 - t).powf(1.5) * step
        };
        let ahead = (far - EGO_Z).max(1e-2);
        let depths = [
            EGO_Z + 2.0f32.min(0.1 * ahead),
            EGO_Z + 0.12 * ahead,
            EGO_Z + 0.28 * ahead,
            EGO_Z + 0.52 * ahead,
            far,
        ];
        let mut grad = Gradient::linear(vec2(0.0, y_near), vec2(0.0, y_far)).stop(0.0, tone.with_alpha(0.0));
        let mut split = !(z_split > depths[0] && z_split < far);
        for z in depths {
            if !split && z_split < z {
                let a = alpha(z_split - 1e-3);
                grad = grad
                    .stop(offset(z_split), tone.with_alpha(a))
                    .stop(offset(z_split), tone.with_alpha(a * BRAKE_ZONE_TONE));
                split = true;
            }
            let a = if z >= far { 0.0 } else { alpha(z) };
            grad = grad.stop(offset(z), tone.with_alpha(a));
        }
        let shape = Shape::Polygon { points: &pts[..n] };
        p.fill(&shape, grad);

        if self.wake_t >= 0.0 {
            // The wake climbs the carpet evenly on screen (not in depth, where it would spend its
            // first frames under the ego and the rest as a sliver at the horizon).
            let k = (self.wake_t / WAKE_S).clamp(0.0, 1.0);
            let start = offset(EGO_Z + self.ego_nose());
            let o1 = start + (1.0 - start) * k;
            let (o0, o2) = ((o1 - WAKE_BAND).max(0.0), (o1 + 0.04).min(1.0));
            let a = 0.7 * (1.0 - k) * (1.0 - 0.4 * k);
            let accent = pal.accent.lerp(pal.roles.ink, 0.55);
            if o2 > o0 + 1e-3 && a > 0.004 {
                let band = Gradient::linear(vec2(0.0, y_near), vec2(0.0, y_far))
                    .stop(0.0, accent.with_alpha(0.0))
                    .stop(o0, accent.with_alpha(0.0))
                    .stop((o1 - 0.03).max(o0 + 1e-4), accent.with_alpha(a))
                    .stop(o1.max(o0 + 2e-4), accent.with_alpha((a * 1.4).min(0.9)))
                    .stop(o2.max(o1 + 3e-4), accent.with_alpha(0.0))
                    .stop(1.0, accent.with_alpha(0.0));
                p.fill(&shape, band);
            }
        }
    }

    /// Solid outer edges and dashed inner lines, perspective-tapered, dissolving to the horizon.
    fn draw_lanes(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let ink = pal.roles.ink;
        for side in [-1.5f32, 1.5] {
            let offset = side * LANE_WIDTH;
            let mut prev: Option<(Vec2, f32)> = None;
            for i in 0..LINE_SAMPLES {
                let z = f.sample_z(i, LINE_SAMPLES, f.z_far);
                let Some(q) = f.lane_point(offset, z) else {
                    prev = None;
                    continue;
                };
                if let Some((a, za)) = prev {
                    let k = EDGE_ALPHA * f.depth_fade(0.5 * (za + z)) * f.edge_fade((a + q) * 0.5);
                    if k > 0.004 {
                        let line = Shape::Line {
                            a,
                            b: q,
                            width_a: f.line_width(EDGE_PX, za),
                            width_b: f.line_width(EDGE_PX, z),
                            cap: Cap::Butt,
                        };
                        p.fill(&line, ink.fade(k));
                    }
                }
                prev = Some((q, z));
            }
        }
        let base = f.z_near - DASH_LEN;
        let first = base + (-self.travelled - base).rem_euclid(DASH_PERIOD);
        for side in [-0.5f32, 0.5] {
            let offset = side * LANE_WIDTH;
            let mut z0 = first;
            while z0 < f.z_far {
                let (za, zb) = (z0.max(f.z_near), (z0 + DASH_LEN).min(f.z_far));
                z0 += DASH_PERIOD;
                if zb <= za + 0.05 {
                    continue;
                }
                let (Some(a), Some(b)) = (f.lane_point(offset, za), f.lane_point(offset, zb)) else {
                    continue;
                };
                let k = DASH_ALPHA * f.depth_fade(0.5 * (za + zb)) * f.edge_fade((a + b) * 0.5);
                if k > 0.004 {
                    let dash = Shape::Line {
                        a,
                        b,
                        width_a: f.line_width(DASH_PX, za),
                        width_b: f.line_width(DASH_PX, zb),
                        cap: Cap::Butt,
                    };
                    p.fill(&dash, ink.fade(k));
                }
            }
        }
    }

    /// The caution stop line across the lanes at the measured stopping distance, with its
    /// distance in metres (only while braking, hidden beyond 120 m).
    fn draw_stop_line(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette, label: FontId) {
        if self.stop_alpha <= 0.004 {
            return;
        }
        let z = EGO_Z + self.ego_nose() + self.stop_m;
        if z >= f.z_far * 0.95 {
            return;
        }
        let edge = 1.5 * LANE_WIDTH;
        let mut pts = [Vec2::ZERO; 7];
        for (i, slot) in pts.iter_mut().enumerate() {
            let o = -edge + 2.0 * edge * i as f32 / 6.0;
            let Some(q) = f.lane_point(o, z) else {
                return;
            };
            *slot = q;
        }
        let k = self.stop_alpha * f.depth_fade(z) * f.edge_fade(pts[3]).max(0.35);
        let color = pal.roles.caution.fade(0.8);
        let w = f.line_width(3.0, z).max(2.0);
        for pair in pts.windows(2) {
            let fade = f.side_fade((pair[0] + pair[1]) * 0.5);
            p.fill(
                &Shape::Line {
                    a: pair[0],
                    b: pair[1],
                    width_a: w,
                    width_b: w,
                    cap: Cap::Butt,
                },
                color.fade(k * fade),
            );
        }
        // The distance, right of the in-path carpet on the line (Titillium, cap 16).
        let metres = self.stop_m.round().clamp(0.0, STOP_MAX_M) as i64;
        let cells = if metres >= 100 {
            3
        } else if metres >= 10 {
            2
        } else {
            1
        };
        let style = TextStyle::new(label, 16.0)
            .paint(color.fade(self.stop_alpha))
            .align(Align::Left, VAlign::Baseline)
            .tabular();
        // The distance, left of the line's left end (clear of the in-path car and its brackets).
        let width = p.fonts().tabular_cell(&style) * f32::from(cells) + 4.0 + p.measure("m", &style).width;
        let pos = vec2(pts[0].x - 12.0 - width, pts[0].y + 6.0);
        if f.vp.contains(pos) {
            let set = p.number(pos, metres, cells, &style, None);
            p.text(vec2(set.max.x + 4.0, pos.y), "m", &style);
        }
    }

    /// 3-step blind-spot arcs beside the ego, crossfaded: primary at 40%, caution at 70%, then
    /// caution at 90% alongside (no glow: the steps stay quieter than the brake bar). Red stays reserved for the brake bar and the redline.
    fn draw_proximity(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let colors = [
            pal.primary_text.fade(0.4),
            pal.roles.caution.fade(0.7),
            pal.roles.caution.fade(0.9),
        ];
        // Blind-spot arcs only while moving: a parked ego next to parked traffic is not a warning.
        let moving = smoothstep(3.0, 6.0, self.speed);
        if moving <= 0.004 {
            return;
        }
        let ego_x = f.road.center_x(EGO_Z);
        for (fade, side) in self.proximity.iter().zip([-1.0f32, 1.0]) {
            for (level, weight) in [(fade.from, 1.0 - fade.t), (fade.to, fade.t)] {
                if level == 0 || weight <= 0.004 {
                    continue;
                }
                let color = colors[usize::from(level.min(3)) - 1];
                for ring in 0..level {
                    let radius = 1.75 + 0.42 * f32::from(ring);
                    let mut pts = [Vec2::ZERO; 15];
                    let mut ok = true;
                    for (i, slot) in pts.iter_mut().enumerate() {
                        let a = (-38.0 + 76.0 * i as f32 / 14.0).to_radians();
                        let world = vec3(ego_x + side * radius * a.cos(), 0.02, EGO_Z + radius * a.sin());
                        match f.cam.project(world) {
                            Some(q) => *slot = q,
                            None => ok = false,
                        }
                    }
                    if !ok {
                        continue;
                    }
                    let k = moving * weight * (1.0 - 0.22 * f32::from(ring)) * f.edge_fade(pts[7]);
                    let width = f.line_width(3.0 + f32::from(level), EGO_Z);
                    // Two chunks of 7 segments (a long polyline evaluates its whole bounding box).
                    for chunk in [&pts[..8], &pts[7..]] {
                        let arc = Shape::Polyline {
                            points: chunk,
                            width,
                            closed: false,
                        };
                        p.fill(&arc, color.fade(k));
                    }
                }
            }
        }
    }

    /// The in-path lead car: the nearest one ahead in the ego's lane.
    fn lead(&self) -> Option<usize> {
        self.traffic
            .users
            .iter()
            .enumerate()
            .filter(|(_, u)| u.lane == 0 && u.z > EGO_Z)
            .min_by(|a, b| a.1.z.total_cmp(&b.1.z))
            .map(|(i, _)| i)
    }

    /// The model transform, body colour level and fade of road user `i` (`None` when hidden).
    fn traffic_pose(&self, f: &Frame, i: usize) -> Option<(Mat4, f32, f32)> {
        let u = self.traffic.users[i];
        let offset = f32::from(u.lane) * LANE_WIDTH;
        let c = f.cam.project(vec3(f.road.center_x(u.z) + offset, 0.0, u.z))?;
        let fade = f.car_fade(u.z) * f.side_fade(c);
        let level = if self.lead() == Some(i) {
            0.78
        } else if u.lane == 0 {
            TRAFFIC_LEVEL
        } else {
            adjacent_level(u.z)
        };
        (fade * level > 0.01).then(|| (f.model(offset, u.z, 0.0, 0.0, 0.0, 0.0), level, fade))
    }

    /// The ego's model transform: on the path, yawed by the drift, pitched by the measured
    /// deceleration and rolled by the lateral load (lifted by the sag, so the nose never sinks
    /// into the road).
    fn ego_pose(&self, f: &Frame) -> Mat4 {
        let ego = self.models[Kind::Ego.index()];
        let dive = self.dive.value().to_radians();
        let roll = self.roll.value().to_radians();
        let lift = 0.25 * ego.dims[1] * dive.abs().sin();
        f.model(0.0, EGO_Z, self.yaw.value(), lift, dive, roll)
    }

    /// The ego's tail-light bar in the ego's model space: `[half width, bottom, top, z]`.
    fn lamp_box(&self) -> [f32; 4] {
        let d = self.models[Kind::Ego.index()].dims;
        [d[0] * 0.5 * 0.62, d[2] * 0.56, d[2] * 0.62, -d[1] * 0.5 - 0.03]
    }

    /// The 2D overlays of the cars, drawn before the meshes: the traffic's contact shadows on the
    /// road, the brake light's spill (the body covers its inner part) and the lead car's FSD
    /// brackets.
    fn draw_car_overlays(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        for (i, u) in self.traffic.users.iter().enumerate() {
            let Some((m, level, fade)) = self.traffic_pose(f, i) else {
                continue;
            };
            let dims = self.models[u.kind.index()].dims;
            let (a, b) = (0.5 * dims[0] * SHADOW_SCALE[0], 0.5 * dims[1] * SHADOW_SCALE[1]);
            let mut pts = [Vec2::ZERO; 16];
            let mut ok = true;
            for (k, slot) in pts.iter_mut().enumerate() {
                let t = core::f32::consts::TAU * k as f32 / 16.0;
                match f.cam.project(transform(&m, vec3(a * t.cos(), 0.0, b * t.sin()))) {
                    Some(q) => *slot = q,
                    None => ok = false,
                }
            }
            if ok {
                let width_px = (pts[0] - pts[8]).length();
                p.glow(
                    &Shape::Polygon { points: &pts },
                    pal.roles.oled_black.fade(SHADOW_ALPHA * fade * level.min(1.0)),
                    Glow {
                        sigma: (0.12 * width_px).clamp(1.5, 10.0),
                        intensity: 1.0,
                        fill_inside: true,
                    },
                );
            }
        }
        let lamp = self.lamps.value().clamp(0.0, 1.0);
        if lamp > 0.02 {
            let m = self.ego_pose(f);
            let [hw, y0, y1, z] = self.lamp_box();
            let mut q = [Vec2::ZERO; 4];
            let mut ok = true;
            for (slot, (x, y)) in q.iter_mut().zip([(-hw, y0), (hw, y0), (hw, y1), (-hw, y1)]) {
                match f.cam.project(transform(&m, vec3(x, y, z))) {
                    Some(s) => *slot = s,
                    None => ok = false,
                }
            }
            if ok {
                p.glow(
                    &Shape::Polygon { points: &q },
                    pal.roles.warning.fade(0.85 * lamp),
                    Glow {
                        sigma: 5.0 + 11.0 * lamp,
                        intensity: 1.0,
                        fill_inside: true,
                    },
                );
            }
        }
        if let Some(i) = self.lead() {
            if let Some((m, _, fade)) = self.traffic_pose(f, i) {
                let dims = self.models[self.traffic.users[i].kind.index()].dims;
                if let Some(b) = f.bounds(&m, dims) {
                    draw_brackets(p, b, pal.primary_text.fade(fade));
                }
            }
        }
    }

    /// The traffic and the ego as clay meshes, then the ego's tail-light bar as an unlit quad.
    ///
    /// Drawn after the 2D scene (the meshes depth-test only against each other).
    pub fn draw_meshes(&self, p: &mut Painter<'_>, g: &SceneGeom, pal: &Palette) {
        let vp = g.viewport;
        if !(vp.min.is_finite() && vp.max.is_finite() && vp.width() >= 64.0 && vp.height() >= 32.0) {
            return;
        }
        let f = Frame::new(g, self.kappa.value());
        let view_proj = f.cam.view_proj(&f.vp);
        let gl = self.light.value();
        let light = vec3(-0.35 - 0.4 * gl, 1.0, -0.6).normalize_or_zero();
        let rim = pal.roles.rim_specular;
        let neutral = Color::rgb_u32(TRAFFIC_RGB);
        let lead = self.lead();
        let draw = |p: &mut Painter<'_>, mesh: MeshId, model: Mat4, color: Color, rim: Color, shading: MeshShading| {
            p.mesh(&MeshDraw {
                mesh,
                model,
                view_proj,
                eye: f.cam.eye,
                viewport: f.vp,
                color,
                rim,
                light,
                shading,
            });
        };
        for (i, u) in self.traffic.users.iter().enumerate() {
            let Some((m, level, fade)) = self.traffic_pose(&f, i) else {
                continue;
            };
            // The lead car is light clay with a 20% brand tint: its brackets mark it, so the
            // saturated hue stays with the path carpet.
            let body = if lead == Some(i) {
                pal.roles.ink.lerp(pal.primary, 0.2)
            } else {
                neutral
            };
            // Opaque and darkened (not translucent), so a body never shows its own back faces.
            draw(
                p,
                self.models[u.kind.index()].mesh,
                m,
                body.intensity(level).fade(fade),
                rim.fade(0.30 * fade * level.max(0.5)),
                MeshShading::Matcap,
            );
        }

        // The ego: a brand-pearl twin (light neutral, 15% brand, an accent-tinted pearl rim).
        let m = self.ego_pose(&f);
        let body = pal.roles.ink.lerp(pal.primary, EGO_TINT).intensity(0.92);
        draw(
            p,
            self.models[Kind::Ego.index()].mesh,
            m,
            body,
            rim.lerp(pal.accent, 0.35).fade(0.55),
            MeshShading::Matcap,
        );
        // The tail-light bar: a smoked lens, warning red from the frame the brake passes 5%.
        let lamp = self.lamps.value().clamp(0.0, 1.0);
        let [hw, y0, y1, z] = self.lamp_box();
        let bar = Mat4::translation(vec3(0.0, y0, z)).mul(&Mat4 {
            cols: [
                [hw, 0.0, 0.0, 0.0],
                [0.0, y1 - y0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        });
        let lens = pal.roles.ink.intensity(0.14);
        draw(
            p,
            self.lamp,
            m.mul(&bar),
            lens.lerp(pal.roles.warning, lamp),
            Color::TRANSPARENT,
            MeshShading::Unlit,
        );
    }
}

/// A unit quad for the tail-light bar: x -1..1, y 0..1 at z 0, facing -Z (the chase camera).
fn lamp_quad() -> MeshData {
    MeshData {
        positions: vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 0.0], [-1.0, 1.0, 0.0]],
        normals: vec![[0.0, 0.0, -1.0]; 4],
        indices: vec![0, 1, 2, 0, 2, 3],
    }
}

/// Carpet depth sample `i` from the ego to `far`, denser near the car.
fn carpet_z(i: usize, far: f32) -> f32 {
    let t = i as f32 / (CARPET_SAMPLES - 1) as f32;
    EGO_Z + (far - EGO_Z) * t.powf(1.6)
}

/// FSD lock brackets round the lead car: four corners, no number (the traffic is synthetic).
fn draw_brackets(p: &mut Painter<'_>, bounds: Rect, color: Color) {
    let r = bounds.expand(6.0);
    let arm = (r.width().min(r.height()) * 0.3).clamp(6.0, 18.0);
    for (c, dx, dy) in [
        (r.min, 1.0f32, 1.0f32),
        (vec2(r.max.x, r.min.y), -1.0, 1.0),
        (vec2(r.min.x, r.max.y), 1.0, -1.0),
        (r.max, -1.0, -1.0),
    ] {
        let pts = [vec2(c.x + dx * arm, c.y), c, vec2(c.x, c.y + dy * arm)];
        p.fill(
            &Shape::Polyline {
                points: &pts,
                width: 2.0,
                closed: false,
            },
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curvature_follows_lateral_acceleration_with_the_speed_floor() {
        // 0.96 g left at 33 m/s: a 116 m radius to the left.
        let k = path_curvature(-0.96 * G, 33.0);
        assert!((k + 0.96 * G / (33.0 * 33.0)).abs() < 1e-6, "{k}");
        assert_eq!(path_curvature(0.0, 40.0), 0.0);
        assert!(path_curvature(5.0, 0.0).is_finite());
        assert_eq!(path_curvature(f32::NAN, 20.0), 0.0);
    }

    #[test]
    fn the_stop_line_uses_the_measured_deceleration_with_a_floor() {
        // 30 m/s at -0.58 g: 79 m; coasting floors at 2 m/s^2 (225 m, beyond the 120 m cut).
        assert!((stop_distance(30.0, -0.58 * G) - 900.0 / (2.0 * 0.58 * G)).abs() < 1e-3);
        assert!((stop_distance(30.0, 0.5) - 225.0).abs() < 1e-3);
        assert_eq!(stop_distance(f32::NAN, -5.0), 0.0);
    }

    #[test]
    fn adjacent_cars_ghost_beside_the_ego_and_never_pop() {
        assert_eq!(adjacent_level(EGO_Z - 6.0), 0.0);
        assert!((adjacent_level(EGO_Z) - GHOST_LEVEL).abs() < 1e-6);
        assert!(adjacent_level(EGO_Z + BLIND_SPOT_AHEAD) <= GHOST_LEVEL + 1e-6);
        assert!((adjacent_level(EGO_Z + BLIND_SPOT_AHEAD + GHOST_RAMP) - TRAFFIC_LEVEL).abs() < 1e-6);
        let mut prev = adjacent_level(EGO_Z - 8.0);
        for i in 1..=300 {
            let a = adjacent_level(EGO_Z - 8.0 + i as f32 * 0.1);
            assert!((a - prev).abs() < 0.05);
            prev = a;
        }
    }

    #[test]
    fn traffic_is_seeded_and_never_overlaps_in_a_lane() {
        let mut t = Traffic::seeded(SEED);
        assert_eq!(t, Traffic::seeded(SEED));
        for _ in 0..3600 {
            t.update(1.0 / 60.0, 30.0);
            let u = &t.users;
            assert!(u[0].lane == 0 && (LEAD_NEAR..=LEAD_FAR).contains(&u[0].z));
            // The two cars sharing a lane stay half the loop apart.
            let span = WRAP_FAR - WRAP_NEAR;
            let gap = (u[2].z - u[3].z).rem_euclid(span);
            assert!(gap.min(span - gap) > 30.0, "same-lane gap {gap}");
        }
    }

    #[test]
    fn the_rotations_pitch_the_nose_down_and_roll_the_roof_left() {
        let nose = transform(&rotation_x(0.1), vec3(0.0, 0.0, 1.0));
        assert!(nose.y < 0.0, "positive dive tips the nose down");
        let roof = transform(&rotation_z(0.1), vec3(0.0, 1.0, 0.0));
        assert!(roof.x < 0.0, "positive roll leans the roof left");
        let yawed = transform(&Mat4::rotation_y(0.3), vec3(0.0, 0.0, 1.0));
        assert!(yawed.x > 0.0, "positive heading turns right, matching RoadCurve");
    }
}
