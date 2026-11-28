//! The TeslaAdas road scene: Tesla's driving visualisation, set on the Bayshore Route at night.
//!
//! A chase camera looks down on three lanes of a Japanese expressway. The road is drawn in road
//! paint, to the figures in `RESEARCH.md`: lane lines of 8 m with 12 m gaps, solid edge lines and
//! a round orange reflector on each edge every 50 m. The cars are Blender sprites rendered through
//! this same camera ([`EYE_HEIGHT`], [`EYE_BACK`], [`PITCH_DEG`] are the recipe's `CAM_H`, `CAM_D`
//! and `PITCH_DEG`), so a sprite's ground centre lands on its projected point at any scale.
//!
//! **What is measured and what is scenery.** The path's bend is `a_lat / v²`, the ego car's yaw is
//! the body slip angle, the brake light is the brake input, and the stop bar with 止まれ before it
//! stands where the car stops at its measured deceleration. The lanes, the reflectors and the
//! traffic are scenery: the packet has no map and no other cars, so the traffic is a seeded
//! ambient simulation that carries no numbers and never changes colour.
use crate::common::adas_road::{EGO_Z, LANE_WIDTH, MAX_DT, MIN_CURVE_SPEED};
use fh6_kit::cluster::Setup;
use fh6_kit::math::{finite_or, smoothstep, vec3};
use fh6_kit::perspective::Camera25;
use fh6_kit::{
    vec2, Affine, Align, BlendMode, Cap, Color, FontId, Glow, Gradient, ImageId, ImagePaint, LayerOptions, Painter,
    Palette, Rect, Shape, Smoothed, TextStyle, VAlign, Vec2,
};

/// The camera's height above the road (m; the recipe's `CAM_H`).
pub const EYE_HEIGHT: f32 = 3.2;
/// How far the camera sits behind the ego car's ground centre (m; the recipe's `CAM_D`).
pub const EYE_BACK: f32 = 7.2;
/// The camera's downward pitch (degrees; the recipe's `PITCH_DEG`).
pub const PITCH_DEG: f32 = 16.0;

/// Ego atlas (`ego.png`, and `ego_glow.png` / `ego_brake.png` at half size): cell size in atlas
/// px, px per metre at the ground centre, and where the ground centre sits in a cell.
const EGO_CELL: [f32; 2] = [1024.0, 640.0];
const EGO_S: f32 = 176.0;
const EGO_ANCHOR: [f32; 2] = [0.5, 0.46];
/// The ego atlas holds 11 yaws from -40° in steps of 8°, four to a row.
const EGO_YAW_MIN: f32 = -40.0;
const EGO_YAW_STEP: f32 = 8.0;
const EGO_CELLS: usize = 11;
const EGO_GRID: [usize; 2] = [4, 3];
/// Traffic atlases: rows are view elevations, columns are headings relative to the view ray.
const TRAFFIC_CELL: [f32; 2] = [336.0, 232.0];
const TRAFFIC_S: f32 = 64.0;
const TRAFFIC_ANCHOR: [f32; 2] = [0.5, 0.70];
const TRAFFIC_ELEVATIONS: [f32; 4] = [3.0, 6.0, 10.0, 16.0];
const TRAFFIC_HEADING_STEP: f32 = 10.0;
const TRAFFIC_COLS: usize = 5;
/// The parked image is 2048 x 1152, and its car's wheels stand this far down it.
const PARKED_ASPECT: f32 = 2048.0 / 1152.0;
const PARKED_FLOOR_Y: f32 = 0.70;
/// The parked car's floor is this much lighter than the road: the car stands alone on it.
const PARKED_FLOOR_LEVEL: f32 = 1.6;

/// Japanese expressway lane line: 8 m of paint in every 20 m (NEXCO; `RESEARCH.md`).
const DASH_ON: f32 = 8.0;
const DASH_PERIOD: f32 = 20.0;
/// Painted line width (m). The pages read give no figure; 0.15 m is our value.
const LINE_M: f32 = 0.15;
/// Edge reflectors stand every 50 m (`RESEARCH.md`), this far outside the edge line and this
/// high, with a reflector of this radius.
const REFLECTOR_EVERY: f32 = 50.0;
const REFLECTOR_OUT: f32 = 0.75;
const REFLECTOR_HEIGHT: f32 = 0.9;
const REFLECTOR_RADIUS: f32 = 0.05;
/// The reflectors' orange, dimmed so it stays scenery and is not read as the caution role.
const REFLECTOR_RGB: u32 = 0xF5_A6_23;
const REFLECTOR_LEVEL: f32 = 0.62;
/// The road repeats every 100 m: the lane lines' 20 m, the reflectors' 50 m and the slow-down
/// dashes' 2.5 m all divide it, so the distance travelled wraps without a jump.
const ROAD_PERIOD: f32 = 100.0;

/// How far the road is drawn (m), and how far on a bend (the heading's tangent stays below this).
const Z_FAR: f32 = 90.0;
const REACH_HEADING_TAN: f32 = 1.2;
const REACH_MIN: f32 = 24.0;
/// Solid lines are drawn in this many tapered pieces, a dash in this many.
const EDGE_PIECES: usize = 22;
const DASH_PIECES: usize = 4;
/// Lane paint and road surface levels (OLED: the road is a hint, not a plate).
const LINE_ALPHA: f32 = 0.62;
const EDGE_ALPHA: f32 = 0.46;
/// The road is lit as a pool round the ego car: full inside this share of the pool's radius,
/// then dissolving to nothing at its rim. Above the car the lane paint keeps its light this much
/// longer than the surface does, because the depth fade already takes it.
const POOL_CORE: f32 = 0.60;
const POOL_PAINT_UP: f32 = 1.7;
/// The under-car glow at its fullest: light on the road, not a lamp.
const GLOW_LEVEL: f32 = 0.55;
/// The cars' share of full white, so the gear and the speed stay the brightest things shown.
const CAR_LEVEL: f32 = 0.72;
/// How far a car's contact shadow reaches past its footprint, and how dark it is at its core.
const SHADOW_REACH: f32 = 1.22;
const SHADOW_DARK: f32 = 0.88;

/// From the ego's ground centre to its nose (m), and half the path band's width.
const NOSE: f32 = 2.45;
const PATH_HALF: f32 = 1.05;
const PATH_SAMPLES: usize = 14;
const PATH_ALPHA: f32 = 0.24;
/// The band's soft edge (the Gaussian's sigma, local px).
const PATH_SOFT_PX: f32 = 5.0;

/// The brake counts as on above this input.
const BRAKE_ON: f32 = 0.05;
/// The stop point uses at least this deceleration (m/s²) and shows below this distance (m) and
/// above this speed (m/s).
const STOP_DECEL_MIN: f32 = 2.0;
const STOP_MAX_M: f32 = 80.0;
const STOP_MIN_SPEED: f32 = 3.0;
/// Japan's stop line is 0.45 m deep.
const STOP_BAR_M: f32 = 0.45;
/// 止まれ before the stop bar: each character's width and length on the road, the gap between
/// characters and the gap to the bar (m). Road text is stretched along the road.
const STOP_TEXT: [&str; 3] = ["止", "ま", "れ"];
const STOP_CHAR_W: f32 = 1.6;
const STOP_CHAR_L: f32 = 2.4;
const STOP_CHAR_GAP: f32 = 0.5;
const STOP_TEXT_LEAD: f32 = 1.2;
/// The text is drawn while the stop point is between these distances (m): nearer, it would run
/// under the car; farther, it is a few pixels tall.
const STOP_TEXT_NEAR: (f32, f32) = (10.2, 11.6);
const STOP_TEXT_FAR: (f32, f32) = (24.0, 32.0);
/// The cap height the road text is set at before it is laid on the road (local px).
const STOP_TEXT_CAP: f32 = 72.0;
/// The slow-down dashes inside the ego lane's lines: width, length and period (m), and the
/// deceleration at which they are fully solid (m/s²).
const SLOW_W: f32 = 0.20;
const SLOW_L: f32 = 1.0;
const SLOW_PERIOD: f32 = 2.5;
const SLOW_INSET: f32 = 0.38;
const SLOW_MAX: usize = 26;
const SLOW_FULL_DECEL: f32 = 9.0;
const SLOW_REACH: f32 = 60.0;

/// The car stands still below the first speed and counts as moving again above the second (m/s).
const PARK_ON: f32 = 0.5;
const PARK_OFF: f32 = 1.5;
/// Time of the parked/driving cross-fade and of the brake light (s).
const PARK_FADE_S: f32 = 0.12;
const BRAKE_FADE_S: f32 = 0.08;
const STOP_FADE_S: f32 = 0.20;

/// Traffic: four road users; side lanes wrap between these depths and the lead car keeps
/// between its own two (m).
const USERS: usize = 4;
const WRAP_NEAR: f32 = -14.0;
const WRAP_FAR: f32 = 78.0;
const LEAD_NEAR: f32 = EGO_Z + 14.0;
const LEAD_FAR: f32 = 58.0;
/// The lead car's sprite is multiplied by this (linear light).
const LEAD_SHADE: f32 = 0.55;
/// Traffic flows at its full relative speed once the ego does this (m/s).
const TRAFFIC_FULL_SPEED: f32 = 12.0;
/// The traffic seed (catalog: ambient traffic is seeded, so two runs draw the same road).
pub const SEED: u64 = 0x7E57_A0A5;

/// One view's scene band and camera.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneGeom {
    /// The band (local px); every draw is clipped to it.
    pub viewport: Rect,
    /// Focal length (local px at 1 m).
    pub focal: f32,
    /// Where the horizon lands (local y).
    pub horizon_y: f32,
    /// Where the parked car's image is drawn.
    pub parked: Rect,
    /// The road surface's lift (ink alpha). 0 in the hood view: its legibility halo is already
    /// the surface, and the overlay's 8-bit premultiplied output would turn a faint lift over a
    /// dark halo into a grey panel.
    pub surface: f32,
}

impl SceneGeom {
    /// The chase camera for this band.
    #[must_use]
    pub fn camera(&self) -> Camera25 {
        Camera25::chase(
            vec3(0.0, EYE_HEIGHT, EGO_Z - EYE_BACK),
            PITCH_DEG.to_radians(),
            self.focal,
            self.focal,
            self.viewport.center().x,
            self.horizon_y,
        )
    }
}

/// A traffic body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Sedan,
    Kei,
    Sport,
}

impl Kind {
    fn index(self) -> usize {
        match self {
            Kind::Sedan => 0,
            Kind::Kei => 1,
            Kind::Sport => 2,
        }
    }
}

/// The scene's images (`assets/clusters/tesla_adas/`); each is optional and has a drawn fallback.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sprites {
    ego: Option<ImageId>,
    glow: Option<ImageId>,
    brake: Option<ImageId>,
    parked: Option<ImageId>,
    traffic: [Option<ImageId>; 3],
}

/// Loads the car sprites that exist.
pub fn load_sprites(setup: &mut Setup<'_>) -> Sprites {
    let mut image = |rel: &str| {
        let path = setup.assets?.join(rel);
        path.is_file()
            .then(|| setup.images.load_file(setup.device, setup.queue, &path, None).ok())
            .flatten()
    };
    Sprites {
        ego: image("clusters/tesla_adas/ego.png"),
        glow: image("clusters/tesla_adas/ego_glow.png"),
        brake: image("clusters/tesla_adas/ego_brake.png"),
        parked: image("clusters/tesla_adas/ego_parked.png"),
        traffic: [
            image("clusters/tesla_adas/traffic_sedan.png"),
            image("clusters/tesla_adas/traffic_kei.png"),
            image("clusters/tesla_adas/traffic_sport.png"),
        ],
    }
}

/// What the scene reads each frame (verified telemetry only).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SceneInput {
    /// Speed (m/s).
    pub speed_ms: f32,
    /// Lateral acceleration (m/s², positive to the right).
    pub a_lat: f32,
    /// Longitudinal acceleration (m/s², negative while braking).
    pub a_long: f32,
    /// Body slip angle (degrees, positive when the car slides to its right).
    pub slip_deg: f32,
    /// Brake input (0..1).
    pub brake: f32,
    /// The shift light's fill (0 below the first stage, 1 at the shift point).
    pub shift_level: f32,
    /// At or above the shift point.
    pub over_shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RoadUser {
    /// -1 left, 0 the ego's lane, 1 right.
    lane: i8,
    z: f32,
    kind: Kind,
}

/// SplitMix64: a small seeded generator for the traffic's layout.
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

    /// A value in 0..1.
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
}

/// The ambient traffic. Japan keeps left: the left lane is the slow one and falls behind the ego
/// car, the right lane is the passing lane and pulls ahead.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Traffic {
    users: [RoadUser; USERS],
    /// Speed relative to the ego car (m/s) of the left lane, the lead car and the right lane.
    lane_speed: [f32; 3],
    lead_dir: f32,
}

impl Traffic {
    fn seeded(seed: u64) -> Self {
        let mut rng = SplitMix64(seed);
        let lane_speed = [
            -(1.4 + 1.8 * rng.unit()),
            0.5 + 1.0 * rng.unit(),
            0.9 + 1.5 * rng.unit(),
        ];
        let lead_kind = match rng.next() % 3 {
            0 => Kind::Sedan,
            1 => Kind::Sport,
            _ => Kind::Kei,
        };
        let lead = RoadUser {
            lane: 0,
            z: LEAD_NEAR + 6.0 + 10.0 * rng.unit(),
            kind: lead_kind,
        };
        let span = WRAP_FAR - WRAP_NEAR;
        // The road opens with a slow car just ahead on the left and the passing car well ahead on
        // the right, so the first seconds already show keep-left traffic.
        let base = EGO_Z + 9.0 + 6.0 * rng.unit() - WRAP_NEAR;
        // The slow lane holds the kei box and a saloon, half the loop apart.
        let slow = if rng.unit() < 0.5 {
            [Kind::Kei, Kind::Sedan]
        } else {
            [Kind::Sedan, Kind::Kei]
        };
        let left = |i: usize, kind: Kind| RoadUser {
            lane: -1,
            z: WRAP_NEAR + (base + i as f32 * span * 0.5).rem_euclid(span),
            kind,
        };
        let right = RoadUser {
            lane: 1,
            z: EGO_Z + 26.0 + 16.0 * rng.unit(),
            kind: Kind::Sport,
        };
        Self {
            users: [lead, left(0, slow[0]), left(1, slow[1]), right],
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
}

/// Moves `value` toward `target` at a constant rate that covers 0..1 in `duration` seconds.
fn approach(value: f32, target: f32, dt: f32, duration: f32) -> f32 {
    let step = dt / duration.max(1e-3);
    if value < target {
        (value + step).min(target)
    } else {
        (value - step).max(target)
    }
}

/// The path's curvature (1/m, positive bends right) from lateral acceleration and speed. Below
/// [`MIN_CURVE_SPEED`] the speed floor stands in, so a parked car's noise bends nothing.
#[must_use]
pub fn path_curvature(a_lat: f32, speed_ms: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).max(MIN_CURVE_SPEED);
    (finite_or(a_lat, 0.0) / (v * v)).clamp(-0.08, 0.08)
}

/// Where the car stops at its measured deceleration (m ahead of its nose): `v² / (2 a)`, with a
/// floor on `a` so a light brake does not put the bar on the horizon.
#[must_use]
pub fn stop_distance(speed_ms: f32, a_long: f32) -> f32 {
    let v = finite_or(speed_ms, 0.0).max(0.0);
    let decel = (-finite_or(a_long, 0.0)).max(STOP_DECEL_MIN);
    v * v / (2.0 * decel)
}

/// How far the road is drawn on a bend of curvature `kappa`.
fn reach(kappa: f32) -> f32 {
    let k = kappa.abs();
    if k < 1e-5 {
        Z_FAR
    } else {
        (REACH_HEADING_TAN / k).clamp(REACH_MIN, Z_FAR)
    }
}

/// The ego atlas cell for a yaw in degrees (positive: nose to the right).
fn ego_cell(yaw_deg: f32) -> (usize, usize) {
    let i = ((finite_or(yaw_deg, 0.0) - EGO_YAW_MIN) / EGO_YAW_STEP)
        .round()
        .clamp(0.0, (EGO_CELLS - 1) as f32) as usize;
    (i % EGO_GRID[0], i / EGO_GRID[0])
}

/// How visible 止まれ is for a stop point `d` metres ahead of the nose.
fn stop_text_level(d: f32) -> f32 {
    smoothstep(STOP_TEXT_NEAR.0, STOP_TEXT_NEAR.1, d) * (1.0 - smoothstep(STOP_TEXT_FAR.0, STOP_TEXT_FAR.1, d))
}

/// One frame's projection: the camera, the path and the band.
#[derive(Debug, Clone, Copy)]
struct Frame {
    cam: Camera25,
    kappa: f32,
    vp: Rect,
    surface: f32,
    z_near: f32,
    z_far: f32,
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
            kappa,
            vp,
            surface: g.surface,
            z_near: z_near.clamp(cam.eye.z + 1.0, EGO_Z),
            z_far: reach(kappa),
        }
    }

    /// The path's lateral position at depth `z`: a parabola through the ego car.
    fn path_x(&self, z: f32) -> f32 {
        let d = z - EGO_Z;
        0.5 * self.kappa * d * d
    }

    /// The path's heading at depth `z` (radians, positive turns right).
    fn heading(&self, z: f32) -> f32 {
        (self.kappa * (z - EGO_Z)).atan()
    }

    /// A point `offset` metres right of the path and `height` above the road at depth `z`.
    fn point(&self, offset: f32, height: f32, z: f32) -> Option<Vec2> {
        self.cam.project(vec3(self.path_x(z) + offset, height, z))
    }

    fn ground(&self, offset: f32, z: f32) -> Option<Vec2> {
        self.point(offset, 0.0, z)
    }

    /// Local px per metre across the road at depth `z`.
    fn scale(&self, z: f32) -> f32 {
        self.cam.fx / self.cam.depth(vec3(0.0, 0.0, z)).max(0.5)
    }

    /// A painted width in px, never thinner than a pixel.
    fn paint_px(&self, metres: f32, z: f32) -> f32 {
        (metres * self.scale(z)).max(1.0)
    }

    /// The pool of light the road is drawn in: an ellipse from the horizon to the band's bottom
    /// edge and across its width (centre, radii).
    fn pool(&self) -> (Vec2, Vec2) {
        let top = self.cam.horizon_y.min(self.vp.max.y - 2.0);
        let ry = (self.vp.max.y - top) * 0.5;
        (vec2(self.vp.center().x, top + ry), vec2(self.vp.width() * 0.5, ry))
    }

    /// The scene dissolves toward the pool's rim and toward the far end of the road.
    fn fade(&self, q: Vec2, z: f32) -> f32 {
        let (c, r) = self.pool();
        let dy = (q.y - c.y) / r.y;
        let dy = if dy < 0.0 { dy / POOL_PAINT_UP } else { dy };
        let far = 1.0 - smoothstep(self.z_far * 0.55, self.z_far, z);
        pool_level(((q.x - c.x) / r.x).hypot(dy)) * far
    }

    /// A strip of road paint `width_m` wide along the path at `offset`, from `z0` to `z1`, in
    /// `pieces` tapered pieces spaced by `spacing` (1 = even, above 1 = denser near the car).
    #[allow(clippy::too_many_arguments)]
    fn paint(
        &self,
        p: &mut Painter<'_>,
        offset: f32,
        z0: f32,
        z1: f32,
        width_m: f32,
        color: Color,
        pieces: usize,
        spacing: f32,
    ) {
        let (z0, z1) = (z0.max(self.z_near), z1.min(self.z_far));
        if z1 <= z0 {
            return;
        }
        let n = pieces.max(1);
        let z_at = |i: usize| z0 + (z1 - z0) * (i as f32 / n as f32).powf(spacing);
        for i in 0..n {
            let (za, zb) = (z_at(i), z_at(i + 1));
            let (Some(a), Some(b)) = (self.ground(offset, za), self.ground(offset, zb)) else {
                continue;
            };
            // Each piece fades from its near end to its far end, so a long piece near the car
            // shows no step where the scene dissolves.
            // The fade is not a straight ramp, so the middle of the piece gets its own stop.
            let zm = (za + zb) * 0.5;
            let fm = self.ground(offset, zm).map_or(0.0, |m| self.fade(m, zm));
            let (fa, fb) = (self.fade(a, za), self.fade(b, zb));
            if fa.max(fb).max(fm) < 0.01 {
                continue;
            }
            let piece = Shape::Line {
                a,
                b,
                width_a: self.paint_px(width_m, za),
                width_b: self.paint_px(width_m, zb),
                cap: Cap::Butt,
            };
            if (fa - fb).abs() < 0.01 && (fa - fm).abs() < 0.01 {
                p.fill(&piece, color.fade(fa));
            } else {
                p.fill(
                    &piece,
                    Gradient::linear(a, b)
                        .stop(0.0, color.fade(fa))
                        .stop(0.5, color.fade(fm))
                        .stop(1.0, color.fade(fb)),
                );
            }
        }
    }
}

/// The pool's light at `d` of its radius: full to [`POOL_CORE`], then an eased fall to nothing.
/// The fall is squared because the scrim view adds light linearly, where a plain ramp reads as an
/// edge.
fn pool_level(d: f32) -> f32 {
    let v = 1.0 - smoothstep(POOL_CORE, 1.0, d);
    v * v
}

/// The road scene's state.
#[derive(Debug, Clone)]
pub struct Scene {
    sprites: Sprites,
    traffic: Traffic,
    /// Distance travelled along the road, wrapped to [`ROAD_PERIOD`] (m).
    travelled: f32,
    speed: f32,
    kappa: Smoothed<f32>,
    /// The ego sprite's yaw (degrees, positive: nose to the right).
    yaw: Smoothed<f32>,
    /// Measured deceleration (m/s², 0 while accelerating).
    decel: Smoothed<f32>,
    /// The stop point's distance ahead of the nose (m).
    stop: Smoothed<f32>,
    /// 0..1: the brake light, the stop bar and the parked view.
    braking: f32,
    stop_level: f32,
    parked: f32,
    is_parked: bool,
    glow: f32,
    over_shift: bool,
    started: bool,
}

impl Scene {
    /// A scene with the seeded traffic.
    #[must_use]
    pub fn new(sprites: Sprites) -> Self {
        Self {
            sprites,
            traffic: Traffic::seeded(SEED),
            travelled: 0.0,
            speed: 0.0,
            kappa: Smoothed::new(0.0, 0.18),
            yaw: Smoothed::new(0.0, 0.10),
            decel: Smoothed::new(0.0, 0.12),
            stop: Smoothed::new(STOP_MAX_M, 0.12),
            braking: 0.0,
            stop_level: 0.0,
            parked: 1.0,
            is_parked: true,
            glow: 0.0,
            over_shift: false,
            started: false,
        }
    }

    /// Advances the scene. The first frame snaps to its targets: the cluster is rebuilt on every
    /// view change and must not replay its fades.
    pub fn update(&mut self, input: &SceneInput, dt: f32) {
        let dt = finite_or(dt, 0.0).clamp(0.0, MAX_DT);
        let v = finite_or(input.speed_ms, 0.0).max(0.0);
        if self.is_parked {
            self.is_parked = v <= PARK_OFF;
        } else {
            self.is_parked = v < PARK_ON;
        }
        let moving = v > MIN_CURVE_SPEED;
        let kappa = if moving { path_curvature(input.a_lat, v) } else { 0.0 };
        let yaw = if moving {
            (-finite_or(input.slip_deg, 0.0)).clamp(EGO_YAW_MIN, -EGO_YAW_MIN)
        } else {
            0.0
        };
        let braking = if finite_or(input.brake, 0.0) > BRAKE_ON {
            1.0
        } else {
            0.0
        };
        let decel = (-finite_or(input.a_long, 0.0)).max(0.0);
        let stop = stop_distance(v, input.a_long).min(STOP_MAX_M * 2.0);
        let stop_on = if braking > 0.5 && v > STOP_MIN_SPEED && stop < STOP_MAX_M {
            1.0
        } else {
            0.0
        };
        let parked = if self.is_parked { 1.0 } else { 0.0 };
        if self.started {
            self.kappa.update(kappa, dt);
            self.yaw.update(yaw, dt);
            self.decel.update(decel, dt);
            self.stop.update(stop, dt);
            self.braking = approach(self.braking, braking, dt, BRAKE_FADE_S);
            self.stop_level = approach(self.stop_level, stop_on, dt, STOP_FADE_S);
            self.parked = approach(self.parked, parked, dt, PARK_FADE_S);
        } else {
            self.kappa.reset(kappa);
            self.yaw.reset(yaw);
            self.decel.reset(decel);
            self.stop.reset(stop);
            self.braking = braking;
            self.stop_level = stop_on;
            self.parked = parked;
            self.started = true;
        }
        self.speed = v;
        self.travelled = (self.travelled + v * dt).rem_euclid(ROAD_PERIOD);
        self.traffic.update(dt, v);
        self.glow = finite_or(input.shift_level, 0.0).clamp(0.0, 1.0);
        self.over_shift = input.over_shift;
    }

    /// True while the road scrolls or a fade runs between packets.
    #[must_use]
    pub fn animating(&self) -> bool {
        let mid = |x: f32| x > 0.0 && x < 1.0;
        self.speed > 0.05 || mid(self.braking) || mid(self.stop_level) || mid(self.parked)
    }

    /// Draws the scene into `g.viewport`. `kanji` is the Japanese face for the road text.
    pub fn draw(&self, p: &mut Painter<'_>, g: &SceneGeom, pal: &Palette, kanji: Option<FontId>) {
        let f = Frame::new(g, self.kappa.value());
        p.scoped(|p| {
            p.clip(g.viewport, 0.0);
            let road = 1.0 - self.parked;
            if road > 0.01 {
                p.scoped(|p| {
                    p.set_opacity(road);
                    self.draw_surface(p, &f, pal);
                    self.draw_lanes(p, &f, pal);
                    self.draw_reflectors(p, &f, pal);
                    self.draw_path(p, &f, pal);
                    self.draw_braking(p, &f, pal, kanji);
                    self.draw_glow(p, &f, pal);
                    self.draw_cars(p, &f, pal);
                });
            }
            if self.parked > 0.01 {
                self.draw_parked(p, g, pal);
            }
        });
    }

    /// The stop point's depth, when one is shown.
    fn stop_z(&self) -> Option<f32> {
        (self.stop_level > 0.01).then(|| EGO_Z + NOSE + self.stop.value())
    }

    /// The road surface: a faint lift of the three lanes inside the pool of light.
    fn draw_surface(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        if f.surface <= 0.0 {
            return;
        }
        let half = 1.5 * LANE_WIDTH + 0.4;
        let mut pts = [Vec2::ZERO; 2 * PATH_SAMPLES];
        let mut n = 0;
        for i in 0..PATH_SAMPLES {
            let z = f.z_near + (f.z_far - f.z_near) * (i as f32 / (PATH_SAMPLES - 1) as f32).powf(1.8);
            let (Some(l), Some(r)) = (f.ground(-half, z), f.ground(half, z)) else {
                continue;
            };
            pts[n] = l;
            pts[2 * PATH_SAMPLES - 1 - n] = r;
            n += 1;
        }
        if n < 2 {
            return;
        }
        // The two halves of the outline meet when every sample projected.
        if n < PATH_SAMPLES {
            return;
        }
        // One radial gradient, squashed into the pool's ellipse: the outline is stretched the
        // other way first, so the squash puts it back where it belongs.
        let (c, r) = f.pool();
        let k = r.x / r.y.max(1.0);
        for q in &mut pts {
            q.y = c.y + (q.y - c.y) * k;
        }
        let ink = pal.roles.ink;
        let mut ramp = Gradient::radial(c, r.x).stop(0.0, ink.with_alpha(f.surface));
        for i in 0..=5 {
            let d = POOL_CORE + (1.0 - POOL_CORE) * i as f32 / 5.0;
            ramp = ramp.stop(d, ink.with_alpha(f.surface * pool_level(d)));
        }
        p.scoped(|p| {
            p.transform(&squash(c, r));
            p.fill(&Shape::Polygon { points: &pts }, ramp);
        });
    }

    /// Lane paint: solid edge lines and the 8 m / 12 m lane lines, scrolling with the road.
    fn draw_lanes(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let ink = pal.roles.ink;
        for side in [-1.0f32, 1.0] {
            f.paint(
                p,
                side * 1.5 * LANE_WIDTH,
                f.z_near,
                f.z_far,
                LINE_M,
                ink.fade(EDGE_ALPHA),
                EDGE_PIECES,
                1.7,
            );
        }
        let phase = self.travelled.rem_euclid(DASH_PERIOD);
        let first = ((f.z_near - EGO_Z + phase) / DASH_PERIOD).floor() as i32 - 1;
        for side in [-1.0f32, 1.0] {
            for k in first..first + 7 {
                let z0 = EGO_Z + k as f32 * DASH_PERIOD - phase;
                if z0 > f.z_far {
                    break;
                }
                f.paint(
                    p,
                    side * 0.5 * LANE_WIDTH,
                    z0,
                    z0 + DASH_ON,
                    LINE_M,
                    ink.fade(LINE_ALPHA),
                    DASH_PIECES,
                    1.0,
                );
            }
        }
    }

    /// The edge reflectors: a post and a round orange reflector every 50 m on both sides.
    fn draw_reflectors(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let phase = self.travelled.rem_euclid(REFLECTOR_EVERY);
        let orange = Color::rgb_u32(REFLECTOR_RGB).intensity(REFLECTOR_LEVEL);
        let out = 1.5 * LANE_WIDTH + REFLECTOR_OUT;
        for k in 0..3 {
            let z = EGO_Z + k as f32 * REFLECTOR_EVERY - phase;
            if z < f.z_near || z > f.z_far {
                continue;
            }
            for side in [-1.0f32, 1.0] {
                let (Some(foot), Some(head)) = (f.ground(side * out, z), f.point(side * out, REFLECTOR_HEIGHT, z))
                else {
                    continue;
                };
                let alpha = f.fade(head, z);
                if alpha < 0.01 {
                    continue;
                }
                p.fill(
                    &Shape::line(foot, head, f.paint_px(0.05, z)),
                    pal.roles.ink.fade(0.10 * alpha),
                );
                let r = (REFLECTOR_RADIUS * f.scale(z)).max(1.6);
                let dot = Shape::circle(head, r);
                p.glow(&dot, orange.fade(alpha), Glow::new(r * 1.6, 0.55));
                p.fill(&dot, orange.fade(alpha));
            }
        }
    }

    /// The path: Tesla's wide band in the palette's primary, from the nose along the measured
    /// bend, ending at the stop bar when there is one.
    fn draw_path(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let near = EGO_Z + NOSE + 0.2;
        let far = self.stop_z().map_or(f.z_far, |z| z.min(f.z_far));
        if far <= near + 1.0 {
            return;
        }
        let (Some(a), Some(b)) = (f.ground(0.0, near), f.ground(0.0, far)) else {
            return;
        };
        let c = pal.primary;
        let end = if self.stop_level > 0.5 { 0.4 } else { 0.0 };
        let mut pts = [Vec2::ZERO; 2 * PATH_SAMPLES];
        for i in 0..PATH_SAMPLES {
            let z = near + (far - near) * (i as f32 / (PATH_SAMPLES - 1) as f32).powf(1.6);
            let (Some(l), Some(r)) = (f.ground(-PATH_HALF, z), f.ground(PATH_HALF, z)) else {
                return;
            };
            pts[i] = l;
            pts[2 * PATH_SAMPLES - 1 - i] = r;
        }
        // Filled, with a soft edge: light on the road, not a painted shape.
        p.glow(
            &Shape::Polygon { points: &pts },
            Gradient::linear(a, b)
                .stop(0.0, c.with_alpha(PATH_ALPHA))
                .stop(0.55, c.with_alpha(PATH_ALPHA * 0.55))
                .stop(1.0, c.with_alpha(PATH_ALPHA * end)),
            Glow {
                fill_inside: true,
                ..Glow::new(PATH_SOFT_PX, 1.0)
            },
        );
    }

    /// Braking, in Japanese road paint: the slow-down dashes inside the ego lane's lines (more
    /// solid the harder the measured braking), the stop bar at the predicted stop point and
    /// 止まれ before it.
    fn draw_braking(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette, kanji: Option<FontId>) {
        let ink = pal.roles.ink;
        if self.braking > 0.01 {
            let hard = (self.decel.value() / SLOW_FULL_DECEL).clamp(0.0, 1.0);
            let level = self.braking * (0.30 + 0.70 * hard);
            let end = self.stop_z().unwrap_or(EGO_Z + SLOW_REACH).min(EGO_Z + SLOW_REACH);
            let phase = self.travelled.rem_euclid(SLOW_PERIOD);
            for k in 0..SLOW_MAX {
                let z0 = EGO_Z + NOSE + 0.6 + k as f32 * SLOW_PERIOD - phase;
                if z0 < EGO_Z + NOSE {
                    continue;
                }
                if z0 + SLOW_L > end {
                    break;
                }
                for side in [-1.0f32, 1.0] {
                    f.paint(
                        p,
                        side * (0.5 * LANE_WIDTH - SLOW_INSET),
                        z0,
                        z0 + SLOW_L,
                        SLOW_W,
                        ink.fade(0.55 * level),
                        1,
                        1.0,
                    );
                }
            }
        }
        let Some(z) = self.stop_z() else {
            return;
        };
        if z >= f.z_far {
            return;
        }
        let half = 0.5 * LANE_WIDTH - 0.2;
        let corners = [
            f.ground(-half, z),
            f.ground(half, z),
            f.ground(half, z + STOP_BAR_M),
            f.ground(-half, z + STOP_BAR_M),
        ];
        if let [Some(a), Some(b), Some(c), Some(d)] = corners {
            let alpha = self.stop_level * f.fade((a + b) * 0.5, z);
            // Never thinner than two pixels: far away the bar is the whole message.
            let d = vec2(d.x, d.y.min(a.y - 2.0));
            let c = vec2(c.x, c.y.min(b.y - 2.0));
            p.fill(&Shape::Polygon { points: &[a, b, c, d] }, ink.fade(0.92 * alpha));
        }
        let text = self.stop_level * stop_text_level(self.stop.value());
        let Some(font) = kanji.filter(|_| text > 0.01) else {
            return;
        };
        let style = TextStyle::new(font, STOP_TEXT_CAP)
            .paint(ink.fade(0.85 * text))
            .align(Align::Center, VAlign::CapMiddle);
        let em = STOP_TEXT_CAP / p.fonts().cap_ratio(font).max(0.1);
        for (i, ch) in STOP_TEXT.iter().enumerate() {
            // 止 is the farthest from the driver, next to the bar; れ the nearest.
            let zc = z - STOP_TEXT_LEAD - STOP_CHAR_L * 0.5 - i as f32 * (STOP_CHAR_L + STOP_CHAR_GAP);
            let points = (
                f.ground(0.0, zc),
                f.ground(0.0, zc + STOP_CHAR_L * 0.5),
                f.ground(0.0, zc - STOP_CHAR_L * 0.5),
                f.ground(STOP_CHAR_W * 0.5, zc),
            );
            let (Some(c), Some(far), Some(near), Some(right)) = points else {
                continue;
            };
            // The road's plane near this character, as an affine map of the flat glyph: local x
            // runs across the lane, local y from the far edge to the near one.
            let ex = (right - c) * (2.0 / em);
            let ey = (near - far) * (1.0 / em);
            p.scoped(|p| {
                p.transform(&Affine {
                    m: [ex.x, ex.y, ey.x, ey.y, c.x, c.y],
                });
                p.text(Vec2::ZERO, ch, &style);
            });
        }
    }

    /// The ego sprite's rectangle: its ground centre on the projected point, at the camera's scale.
    fn ego_rect(f: &Frame) -> Option<Rect> {
        let q = f.ground(0.0, EGO_Z)?;
        let k = f.scale(EGO_Z) / EGO_S;
        let size = vec2(EGO_CELL[0] * k, EGO_CELL[1] * k);
        Some(Rect::from_xywh(
            q.x - EGO_ANCHOR[0] * size.x,
            q.y - EGO_ANCHOR[1] * size.y,
            size.x,
            size.y,
        ))
    }

    /// The glow under the ego car, the shift light's second voice: dark below the first shift
    /// stage, growing in the primary through the stages, the shift colour at the shift point.
    /// It is steady: only the shift line strobes.
    fn draw_glow(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let level = if self.over_shift {
            GLOW_LEVEL
        } else {
            self.glow * GLOW_LEVEL * 0.8
        };
        if level < 0.01 {
            return;
        }
        let Some(rect) = Self::ego_rect(f) else {
            return;
        };
        let color = if self.over_shift { pal.shift } else { pal.primary };
        match self.sprites.glow {
            Some(img) => p.layer(LayerOptions::bloom(), |p| {
                p.set_blend(BlendMode::Add);
                sprite(
                    p,
                    img,
                    EGO_GRID,
                    ego_cell(self.yaw.value()),
                    false,
                    rect,
                    color.fade(level),
                );
            }),
            None => {
                let c = rect.center();
                let pool = Rect::from_center_size(vec2(c.x, c.y + rect.height() * 0.10), rect.size() * 0.36);
                p.glow(
                    &Shape::round_rect(pool, pool.width() * 0.2),
                    color.fade(0.5 * level),
                    Glow {
                        fill_inside: true,
                        ..Glow::new(pool.width() * 0.22, 0.6)
                    },
                );
            }
        }
    }

    /// The cars, far to near: the seeded traffic in its grey and the ego car, then the ego's
    /// brake light added on top.
    fn draw_cars(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let mut order = [(0.0f32, 0usize); USERS + 1];
        for (slot, (i, u)) in order.iter_mut().zip(self.traffic.users.iter().enumerate()) {
            *slot = (u.z, i);
        }
        order[USERS] = (EGO_Z, USERS);
        order.sort_unstable_by(|a, b| b.0.total_cmp(&a.0));
        self.draw_shadows(p, f, pal);
        for (_, i) in order {
            if i == USERS {
                self.draw_ego(p, f, pal);
            } else {
                self.draw_road_user(p, f, pal, self.traffic.users[i]);
            }
        }
    }

    /// The contact shadows: each car darkens the lit road under it, so it stands on the road.
    /// Multiplied, so only the surface's light is taken and nothing is laid over black.
    fn draw_shadows(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        if f.surface <= 0.0 {
            return;
        }
        let black = pal.roles.oled_black;
        p.scoped(|p| {
            p.set_blend(BlendMode::Multiply);
            let ego = (0.0, EGO_Z, [0.97, 2.35]);
            let users = self.traffic.users.iter().map(|u| {
                let half = match u.kind {
                    Kind::Kei => [0.74, 1.70],
                    _ => [0.95, 2.30],
                };
                (f32::from(u.lane) * LANE_WIDTH, u.z, half)
            });
            for (offset, z, [half_w, half_l]) in std::iter::once(ego).chain(users) {
                if z < f.z_near - 2.0 || z > f.z_far {
                    continue;
                }
                let (Some(q), Some(near), Some(far)) = (
                    f.ground(offset, z),
                    f.ground(offset, z - half_l),
                    f.ground(offset, z + half_l),
                ) else {
                    continue;
                };
                let level = f.fade(q, z);
                let c = vec2(q.x, (near.y + far.y) * 0.5);
                let rx = half_w * f.scale(z) * SHADOW_REACH;
                let ry = (near.y - far.y) * 0.5 * SHADOW_REACH;
                if level < 0.05 || rx < 2.0 || ry < 1.0 {
                    continue;
                }
                contact_shadow(p, c, vec2(rx, ry), SHADOW_DARK * level, black);
            }
        });
    }

    fn draw_road_user(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette, u: RoadUser) {
        if u.z < f.z_near - 2.0 || u.z > f.z_far {
            return;
        }
        let offset = f32::from(u.lane) * LANE_WIDTH;
        let Some(q) = f.ground(offset, u.z) else {
            return;
        };
        let alpha = f.fade(q, u.z);
        if alpha < 0.01 {
            return;
        }
        let k = f.scale(u.z) / TRAFFIC_S;
        let size = vec2(TRAFFIC_CELL[0] * k, TRAFFIC_CELL[1] * k);
        let rect = Rect::from_xywh(
            q.x - TRAFFIC_ANCHOR[0] * size.x,
            q.y - TRAFFIC_ANCHOR[1] * size.y,
            size.x,
            size.y,
        );
        let Some(img) = self.sprites.traffic[u.kind.index()] else {
            // No sprite: a grey block of the car's footprint, so the traffic never disappears.
            let w = 1.8 * f.scale(u.z);
            let body = Rect::from_xywh(q.x - w * 0.5, q.y - w * 0.62, w, w * 0.72);
            p.fill(
                &Shape::round_rect(body, w * 0.12),
                pal.roles.ink_label.fade(0.55 * alpha),
            );
            return;
        };
        let x = f.path_x(u.z) + offset;
        let (dx, dz) = (x - f.cam.eye.x, u.z - f.cam.eye.z);
        let elevation = f.cam.eye.y.atan2(dx.hypot(dz)).to_degrees();
        let rel = (f.heading(u.z) - dx.atan2(dz)).to_degrees();
        let col = ((rel.abs() / TRAFFIC_HEADING_STEP).round() as usize).min(TRAFFIC_COLS - 1);
        let row = TRAFFIC_ELEVATIONS
            .iter()
            .enumerate()
            .min_by(|a, b| (a.1 - elevation).abs().total_cmp(&(b.1 - elevation).abs()))
            .map_or(0, |(r, _)| r);
        // One cell, opaque: a blend of two headings reads as a double exposure.
        sprite(
            p,
            img,
            [TRAFFIC_COLS, TRAFFIC_ELEVATIONS.len()],
            (col, row),
            rel < 0.0,
            rect,
            // Tesla draws the car ahead in a darker grey than the rest of the traffic.
            if u.lane == 0 {
                grey(LEAD_SHADE).fade(alpha)
            } else {
                grey(CAR_LEVEL).fade(alpha)
            },
        );
    }

    fn draw_ego(&self, p: &mut Painter<'_>, f: &Frame, pal: &Palette) {
        let Some(rect) = Self::ego_rect(f) else {
            return;
        };
        let cell = ego_cell(self.yaw.value());
        let warning = pal.roles.warning;
        match self.sprites.ego {
            Some(img) => sprite(p, img, EGO_GRID, cell, false, rect, grey(CAR_LEVEL)),
            None => {
                // No sprite: the car's plan as a white block with its dark glass, turned by the
                // same yaw the atlas would show.
                let m = f.scale(EGO_Z);
                let c = rect.min + vec2(rect.width() * EGO_ANCHOR[0], rect.height() * EGO_ANCHOR[1]);
                let body = Rect::from_xywh(c.x - 0.97 * m, c.y - 1.10 * m, 1.94 * m, 2.50 * m);
                p.scoped(|p| {
                    p.rotate_about(self.yaw.value().to_radians(), c);
                    p.fill(&Shape::round_rect(body, 0.30 * m), pal.roles.ink.fade(0.9));
                    p.fill(
                        &Shape::round_rect(body.inset(0.30 * m), 0.18 * m),
                        pal.roles.oled_black.with_alpha(0.85),
                    );
                    let y = body.max.y - 0.55 * m;
                    p.fill(
                        &Shape::line(vec2(body.min.x + 0.2 * m, y), vec2(body.max.x - 0.2 * m, y), 0.07 * m),
                        warning.fade(0.45 + 0.55 * self.braking),
                    );
                });
            }
        }
        if self.braking > 0.01 {
            if let Some(img) = self.sprites.brake {
                p.layer(LayerOptions::bloom(), |p| {
                    p.set_blend(BlendMode::Add);
                    sprite(p, img, EGO_GRID, cell, false, rect, warning.fade(self.braking));
                });
            }
        }
    }

    /// The floor under the parked car: a small pool of the road's light with the car's shadow
    /// in it, placed where the parked image's wheels stand.
    fn draw_floor(&self, p: &mut Painter<'_>, rect: Rect, surface: f32, pal: &Palette) {
        let c = vec2(rect.center().x, rect.min.y + rect.height() * PARKED_FLOOR_Y);
        let r = vec2(rect.width() * 0.56, rect.height() * 0.32);
        let ink = pal.roles.ink;
        let level = surface * PARKED_FLOOR_LEVEL * self.parked;
        let mut ramp = Gradient::radial(c, r.x).stop(0.0, ink.with_alpha(level));
        for i in 0..=5 {
            let d = 0.35 + 0.65 * i as f32 / 5.0;
            let v = 1.0 - smoothstep(0.35, 1.0, d);
            ramp = ramp.stop(d, ink.with_alpha(level * v * v));
        }
        p.scoped(|p| {
            p.transform(&squash(c, r));
            p.fill(&Shape::circle(c, r.x), ramp);
        });
        p.scoped(|p| {
            p.set_blend(BlendMode::Multiply);
            contact_shadow(
                p,
                c,
                vec2(rect.width() * 0.34, rect.height() * 0.11),
                SHADOW_DARK * self.parked,
                pal.roles.oled_black,
            );
        });
    }

    /// The parked view, as Tesla draws a car that stands still: the car alone, three-quarter front.
    fn draw_parked(&self, p: &mut Painter<'_>, g: &SceneGeom, pal: &Palette) {
        let fit = if g.parked.width() / g.parked.height().max(1.0) > PARKED_ASPECT {
            vec2(g.parked.height() * PARKED_ASPECT, g.parked.height())
        } else {
            vec2(g.parked.width(), g.parked.width() / PARKED_ASPECT)
        };
        let rect = Rect::from_center_size(g.parked.center(), fit);
        match self.sprites.parked {
            Some(img) => {
                if g.surface > 0.0 {
                    self.draw_floor(p, rect, g.surface, pal);
                }
                p.fill(
                    &Shape::rect(rect),
                    ImagePaint::new(img, rect).tint(grey(CAR_LEVEL).fade(self.parked)),
                );
            }
            None => {
                let body = Rect::from_center_size(rect.center(), vec2(fit.x * 0.62, fit.y * 0.34));
                p.fill(
                    &Shape::round_rect(body, body.height() * 0.45),
                    pal.roles.ink.fade(0.85 * self.parked),
                );
            }
        }
    }
}

/// A soft dark ellipse. Draw it with [`BlendMode::Multiply`]: it takes light from what is under
/// it and lays nothing over black.
fn contact_shadow(p: &mut Painter<'_>, center: Vec2, radii: Vec2, dark: f32, black: Color) {
    p.scoped(|p| {
        p.transform(&squash(center, radii));
        p.fill(
            &Shape::circle(center, radii.x),
            Gradient::radial(center, radii.x)
                .stop(0.0, black.with_alpha(dark))
                .stop(0.66, black.with_alpha(dark))
                .stop(0.84, black.with_alpha(dark * 0.45))
                .stop(1.0, black.with_alpha(0.0)),
        );
    });
}

/// A squash about `center` that turns a circle of radius `radii.x` into the ellipse `radii`.
fn squash(center: Vec2, radii: Vec2) -> Affine {
    Affine::translate(-center)
        .then(&Affine::scale_xy(1.0, radii.y / radii.x.max(1e-3)))
        .then(&Affine::translate(center))
}

/// An opaque neutral grey of linear level `k`: a sprite's tint.
fn grey(k: f32) -> Color {
    Color::linear(k, k, k, 1.0)
}

/// Draws one cell of an atlas into `rect`, mirrored left-right when asked.
fn sprite(
    p: &mut Painter<'_>,
    img: ImageId,
    grid: [usize; 2],
    cell: (usize, usize),
    mirror: bool,
    rect: Rect,
    tint: Color,
) {
    let (cu, cv) = (1.0 / grid[0] as f32, 1.0 / grid[1] as f32);
    let (u0, v0) = (cell.0 as f32 * cu, cell.1 as f32 * cv);
    let uv = if mirror {
        Rect {
            min: vec2(u0 + cu, v0),
            max: vec2(u0, v0 + cv),
        }
    } else {
        Rect::from_xywh(u0, v0, cu, cv)
    };
    p.fill(
        &Shape::rect(rect),
        ImagePaint {
            uv,
            ..ImagePaint::new(img, rect).tint(tint)
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geom() -> SceneGeom {
        SceneGeom {
            viewport: Rect::from_xywh(-360.0, -252.0, 720.0, 492.0),
            focal: 700.0,
            horizon_y: -226.0,
            parked: Rect::from_xywh(-290.0, -150.0, 580.0, 330.0),
            surface: 0.03,
        }
    }

    #[test]
    fn curvature_follows_lateral_acceleration_with_the_speed_floor() {
        assert_eq!(path_curvature(0.0, 30.0), 0.0);
        assert!((path_curvature(9.0, 30.0) - 0.01).abs() < 1e-6);
        assert!(
            path_curvature(-9.0, 30.0) < 0.0,
            "left acceleration bends the path left"
        );
        let parked = path_curvature(0.5, 0.0);
        assert!((parked - 0.5 / (MIN_CURVE_SPEED * MIN_CURVE_SPEED)).abs() < 1e-6);
        assert_eq!(path_curvature(f32::NAN, f32::NAN), 0.0);
    }

    #[test]
    fn the_stop_point_uses_the_measured_deceleration_with_a_floor() {
        assert!((stop_distance(20.0, -10.0) - 20.0).abs() < 1e-4);
        assert!((stop_distance(20.0, 0.0) - 100.0).abs() < 1e-3, "the floor is 2 m/s^2");
        assert_eq!(stop_distance(0.0, -5.0), 0.0);
        assert_eq!(stop_distance(f32::NAN, f32::NAN), 0.0);
    }

    #[test]
    fn the_ego_sits_where_the_recipe_rendered_it() {
        // The sprite is rendered with its ground centre straight ahead of the camera at EYE_BACK,
        // so the same point must project onto the band's centre line.
        let f = Frame::new(&geom(), 0.0);
        let q = f.ground(0.0, EGO_Z);
        assert!(q.is_some_and(|q| q.x.abs() < 1e-3 && q.y > 0.0 && q.y < 140.0), "{q:?}");
        // 176 px per metre in the atlas at this point: the car is about 1.94 m wide.
        let px_per_m = f.scale(EGO_Z);
        assert!((px_per_m - 700.0 / 7.803).abs() < 0.05, "{px_per_m}");
        // The tail's foot stays inside the band, above the power line.
        let tail = f.ground(0.0, EGO_Z - NOSE);
        assert!(tail.is_some_and(|t| t.y < geom().viewport.max.y - 30.0), "{tail:?}");
    }

    #[test]
    fn yaw_picks_the_nearest_atlas_cell() {
        assert_eq!(ego_cell(0.0), (1, 1), "straight ahead is the sixth cell");
        assert_eq!(ego_cell(-40.0), (0, 0));
        assert_eq!(ego_cell(40.0), (2, 2));
        assert_eq!(ego_cell(90.0), (2, 2), "clamped");
        assert_eq!(ego_cell(f32::NAN), (1, 1));
        assert_eq!(ego_cell(11.0), ego_cell(8.0));
    }

    #[test]
    fn traffic_is_seeded_keeps_left_and_never_overlaps_in_a_lane() {
        let mut a = Traffic::seeded(SEED);
        let b = Traffic::seeded(SEED);
        assert_eq!(a, b, "the same seed draws the same road");
        assert!(a.lane_speed[0] < 0.0, "the left lane is the slow lane and falls behind");
        assert!(a.lane_speed[2] > 0.0, "the right lane passes");
        assert!(a.users.iter().any(|u| u.kind == Kind::Kei && u.lane == -1));
        for _ in 0..4000 {
            a.update(1.0 / 60.0, 40.0);
            let left: Vec<f32> = a.users.iter().filter(|u| u.lane == -1).map(|u| u.z).collect();
            assert!(
                (left[0] - left[1]).abs() > 20.0,
                "the slow lane's cars keep apart: {left:?}"
            );
            for u in &a.users {
                assert!(u.z >= WRAP_NEAR && u.z <= WRAP_FAR, "{u:?}");
                if u.lane == 0 {
                    assert!(u.z >= LEAD_NEAR, "the lead car never reaches the ego");
                }
            }
        }
    }

    #[test]
    fn the_road_text_shows_only_where_it_can_be_read() {
        assert_eq!(stop_text_level(5.0), 0.0, "it would run under the car");
        assert_eq!(stop_text_level(16.0), 1.0);
        assert_eq!(stop_text_level(60.0), 0.0, "a few pixels tall");
        // The nearest character's near edge is ahead of the nose when the text appears.
        let run = STOP_TEXT_LEAD + 3.0 * STOP_CHAR_L + 2.0 * STOP_CHAR_GAP;
        assert!(STOP_TEXT_NEAR.0 > run, "{run}");
    }

    #[test]
    fn the_scene_snaps_on_its_first_frame_and_parks_with_hysteresis() {
        let input = |speed: f32| SceneInput {
            speed_ms: speed,
            a_lat: 0.0,
            a_long: 0.0,
            slip_deg: 0.0,
            brake: 0.0,
            shift_level: 0.0,
            over_shift: false,
        };
        let mut s = Scene::new(Sprites::default());
        s.update(&input(30.0), 1.0 / 60.0);
        assert_eq!(s.parked, 0.0, "a rebuilt cluster does not replay the fade");
        s.update(&input(1.0), 1.0 / 60.0);
        assert_eq!(s.parked, 0.0, "still moving at 1 m/s");
        for _ in 0..60 {
            s.update(&input(0.0), 1.0 / 60.0);
        }
        assert_eq!(s.parked, 1.0);
        s.update(&input(1.0), 1.0 / 60.0);
        assert_eq!(s.parked, 1.0, "a creep below 1.5 m/s keeps the parked view");
        let mut fresh = Scene::new(Sprites::default());
        fresh.update(&input(0.0), 1.0 / 60.0);
        assert_eq!(fresh.parked, 1.0);
        assert!(!fresh.animating());
    }

    #[test]
    fn the_road_wraps_without_a_jump() {
        for period in [DASH_PERIOD, REFLECTOR_EVERY, SLOW_PERIOD] {
            let n = ROAD_PERIOD / period;
            assert!((n - n.round()).abs() < 1e-6, "{period} m divides the road period");
        }
    }
}
