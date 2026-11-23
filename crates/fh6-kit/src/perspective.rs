//! 2.5D helpers: a pinhole camera for ADAS road views and the road-curvature model, ported from
//! the v1 Tesla ADAS cluster and made allocation-free.
//!
//! World space is metres: +X right, +Y up, +Z forward. The ego car sits at the origin; the eye
//! defaults to `(0, 1.85, -3.4)` (AGENTS 3D), looking straight down +Z. A chase camera
//! ([`Camera25::chase`]) may also pitch its optical axis down; the horizon stays at `horizon_y`.
//! The same camera projects 2D road geometry ([`Camera25::project`]) and drives the mesh pass
//! ([`Camera25::view_proj`]), so glTF cars sit exactly on the drawn road.
use crate::math::{finite_or, vec2, vec3, Mat4, Rect, Vec2, Vec3};

/// Nearest depth (metres in front of the eye) that is projected (near-plane guard).
pub const NEAR_Z: f32 = 0.1;

/// A pinhole camera projecting onto local 2D coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera25 {
    /// Eye position in world metres.
    pub eye: Vec3,
    /// Horizontal focal length in local units (pixels per metre at 1 m depth).
    pub fx: f32,
    /// Vertical focal length in local units.
    pub fy: f32,
    /// Local x of the optical centre.
    pub center_x: f32,
    /// Local y of the horizon (where Z goes to infinity).
    pub horizon_y: f32,
    /// Downward pitch of the optical axis in radians (0 looks straight along +Z). The horizon
    /// stays at `horizon_y`; the optical centre sits `fy * tan(pitch)` below it.
    pub pitch: f32,
}

impl Camera25 {
    /// The ADAS camera with its eye at `(0, 1.85, -3.4)` and no pitch.
    #[must_use]
    pub fn adas(fx: f32, fy: f32, center_x: f32, horizon_y: f32) -> Self {
        Self {
            eye: vec3(0.0, 1.85, -3.4),
            fx,
            fy,
            center_x,
            horizon_y,
            pitch: 0.0,
        }
    }

    /// A chase camera at `eye` (metres) whose optical axis is pitched `pitch` radians down
    /// (tokens `components.adasRoad`: eye `(0, 3.4, -6.0)`, 8 degrees). `horizon_y` is where
    /// the horizon lands; non-finite inputs fall back to the level ADAS camera's values.
    #[must_use]
    pub fn chase(eye: Vec3, pitch: f32, fx: f32, fy: f32, center_x: f32, horizon_y: f32) -> Self {
        Self {
            eye: vec3(finite_or(eye.x, 0.0), finite_or(eye.y, 1.85), finite_or(eye.z, -3.4)),
            fx,
            fy,
            center_x,
            horizon_y,
            pitch: finite_or(pitch, 0.0).clamp(-1.2, 1.2),
        }
    }

    /// Local y of the optical centre (equal to `horizon_y` without pitch).
    #[must_use]
    pub fn optical_center_y(&self) -> f32 {
        self.horizon_y + self.fy * self.pitch.tan()
    }

    /// The camera-space depth of a world point (metres along the optical axis); the point is
    /// projected only when this exceeds [`NEAR_Z`].
    #[must_use]
    pub fn depth(&self, p: Vec3) -> f32 {
        let (s, c) = self.pitch.sin_cos();
        (p.z - self.eye.z) * c - (p.y - self.eye.y) * s
    }

    /// Projects a world point; `None` behind the near plane.
    #[must_use]
    pub fn project(&self, p: Vec3) -> Option<Vec2> {
        if self.pitch == 0.0 {
            let zc = p.z - self.eye.z;
            if zc.partial_cmp(&NEAR_Z) != Some(std::cmp::Ordering::Greater) || !zc.is_finite() {
                return None;
            }
            return Some(vec2(
                self.center_x + self.fx * (p.x - self.eye.x) / zc,
                self.horizon_y + self.fy * (self.eye.y - p.y) / zc,
            ));
        }
        let (s, c) = self.pitch.sin_cos();
        let (dy, dz) = (p.y - self.eye.y, p.z - self.eye.z);
        let zc = dz * c - dy * s;
        if zc.partial_cmp(&NEAR_Z) != Some(std::cmp::Ordering::Greater) || !zc.is_finite() {
            return None;
        }
        let yc = dy * c + dz * s;
        Some(vec2(
            self.center_x + self.fx * (p.x - self.eye.x) / zc,
            self.optical_center_y() - self.fy * yc / zc,
        ))
    }

    /// The view-projection matrix whose NDC, mapped onto `viewport` (local), reproduces
    /// [`Camera25::project`] exactly. Depth runs 0..1 from `NEAR_Z` to 500 m.
    #[must_use]
    pub fn view_proj(&self, viewport: &Rect) -> Mat4 {
        let (vw, vh) = (viewport.width().max(1e-3), viewport.height().max(1e-3));
        let vc = viewport.center();
        let far = 500.0;
        let range = far / (far - NEAR_Z);
        let sx = 2.0 * self.fx / vw;
        let sy = 2.0 * self.fy / vh;
        let ox = 2.0 * (self.center_x - vc.x) / vw;
        let oy = 2.0 * (vc.y - self.optical_center_y()) / vh;
        let proj = Mat4 {
            cols: [
                [sx, 0.0, 0.0, 0.0],
                [0.0, sy, 0.0, 0.0],
                [ox, oy, range, 1.0],
                [0.0, 0.0, -range * NEAR_Z, 0.0],
            ],
        };
        let view = Mat4::translation(vec3(-self.eye.x, -self.eye.y, -self.eye.z));
        if self.pitch == 0.0 {
            return proj.mul(&view);
        }
        // Pitch about +X: camera up/forward are the world axes tilted `pitch` down.
        let (s, c) = self.pitch.sin_cos();
        let tilt = Mat4 {
            cols: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c, -s, 0.0],
                [0.0, s, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        proj.mul(&tilt).mul(&view)
    }
}

/// The road ahead, bent by yaw rate and drift (spec: `kappa = omega_y / V`, slip `beta`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RoadCurve {
    /// Curvature in 1/m (positive bends right).
    pub kappa: f32,
    /// Tangent of the body slip angle.
    pub tan_beta: f32,
}

/// Speed below which curvature is not trusted (low-speed singularity guard), m/s.
pub const MIN_CURVE_SPEED: f32 = 2.0;
/// Largest curvature used (radius >= 8 m).
pub const MAX_KAPPA: f32 = 0.125;

impl RoadCurve {
    /// From yaw rate (rad/s), speed (m/s) and body slip angle (rad).
    #[must_use]
    pub fn from_motion(yaw_rate: f32, speed: f32, slip: f32) -> Self {
        let speed = finite_or(speed, 0.0);
        let kappa = if speed.abs() < MIN_CURVE_SPEED {
            0.0
        } else {
            (finite_or(yaw_rate, 0.0) / speed).clamp(-MAX_KAPPA, MAX_KAPPA)
        };
        Self {
            kappa,
            tan_beta: finite_or(slip, 0.0).clamp(-1.2, 1.2).tan(),
        }
    }

    /// Lateral offset of the lane centre at depth `z` metres.
    #[must_use]
    pub fn center_x(&self, z: f32) -> f32 {
        0.5 * self.kappa * z * z + self.tan_beta * z * 0.14
    }

    /// Fills `out` with the projected points of the line at lateral offset `offset` (metres from
    /// the lane centre) from `z_near` to `z_far`, spaced with a 1.25 power (denser near the car).
    /// Returns how many points were written (points behind the near plane are skipped).
    pub fn project_line(&self, cam: &Camera25, offset: f32, z_near: f32, z_far: f32, out: &mut [Vec2]) -> usize {
        let n = out.len();
        if n < 2 {
            return 0;
        }
        let mut written = 0;
        for i in 0..n {
            let t = i as f32 / (n - 1) as f32;
            let z = z_near + (z_far - z_near) * t.powf(1.25);
            if let Some(p) = cam.project(vec3(self.center_x(z) + offset, 0.0, z)) {
                out[written] = p;
                written += 1;
            }
        }
        written
    }

    /// Heading of the road at depth `z` (radians, positive turns right), for placing cars.
    #[must_use]
    pub fn heading(&self, z: f32) -> f32 {
        (self.kappa * z + self.tan_beta * 0.14).atan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_matches_v1_formula() {
        let cam = Camera25::adas(900.0, 900.0, 0.0, -100.0);
        let p = cam.project(vec3(1.0, 0.0, 10.0)).expect("in front");
        let zc = 10.0 + 3.4;
        assert!((p.x - 900.0 / zc).abs() < 1e-3);
        assert!((p.y - (-100.0 + 900.0 * 1.85 / zc)).abs() < 1e-3);
        assert!(cam.project(vec3(0.0, 0.0, -3.4)).is_none(), "near-plane guard");
    }

    #[test]
    fn view_proj_agrees_with_project() {
        let cam = Camera25::adas(700.0, 650.0, 30.0, -120.0);
        let vp = Rect::from_xywh(-400.0, -300.0, 800.0, 500.0);
        let m = cam.view_proj(&vp);
        for p in [vec3(0.0, 0.0, 5.0), vec3(-3.0, 1.2, 20.0), vec3(2.0, 0.5, 60.0)] {
            let want = cam.project(p).expect("visible");
            let c = m.project(p);
            let ndc = vec2(c[0] / c[3], c[1] / c[3]);
            let got = vec2(
                vp.center().x + ndc.x * vp.width() * 0.5,
                vp.center().y - ndc.y * vp.height() * 0.5,
            );
            assert!((got - want).length() < 1e-2, "{p:?}: {got:?} vs {want:?}");
            let depth = c[2] / c[3];
            assert!((0.0..=1.0).contains(&depth));
        }
    }

    #[test]
    fn pitched_chase_camera() {
        let level = Camera25::adas(700.0, 650.0, 30.0, -120.0);
        let same = Camera25::chase(level.eye, 0.0, 700.0, 650.0, 30.0, -120.0);
        assert_eq!(same, level, "zero pitch is the level ADAS camera");

        let pitch = 8f32.to_radians();
        let cam = Camera25::chase(vec3(0.0, 3.4, -6.0), pitch, 560.0, 560.0, 0.0, -60.0);
        // Far ground points converge on the horizon; the optical centre sits below it.
        let far = cam.project(vec3(0.0, 0.0, 1.0e5)).expect("far point");
        assert!((far.y - cam.horizon_y).abs() < 0.5, "horizon stays at horizon_y: {far:?}");
        assert!((cam.optical_center_y() - (-60.0 + 560.0 * pitch.tan())).abs() < 1e-3);
        // A point on the optical axis lands on the optical centre.
        let axis = vec3(0.0, 3.4 - 20.0 * pitch.sin(), -6.0 + 20.0 * pitch.cos());
        let on_axis = cam.project(axis).expect("on axis");
        assert!((on_axis.y - cam.optical_center_y()).abs() < 1e-2, "{on_axis:?}");
        assert!((cam.depth(axis) - 20.0).abs() < 1e-3);
        // The ego's ground point (z = 3.2 m) is below the optical centre, nearer ground lower.
        let ego = cam.project(vec3(0.0, 0.0, 3.2)).expect("ego");
        let near = cam.project(vec3(0.0, 0.0, 1.0)).expect("near ground");
        assert!(ego.y > cam.optical_center_y() && near.y > ego.y, "{ego:?} {near:?}");
        // Points that are behind the pitched near plane are rejected.
        assert!(cam.project(vec3(0.0, 3.4, -6.0)).is_none());

        // The mesh matrix reproduces the pitched projection.
        let vp = Rect::from_xywh(-400.0, -150.0, 800.0, 300.0);
        let m = cam.view_proj(&vp);
        for p in [vec3(0.0, 0.0, 3.2), vec3(-3.6, 1.2, 20.0), vec3(5.4, 0.0, 60.0)] {
            let want = cam.project(p).expect("visible");
            let c = m.project(p);
            let ndc = vec2(c[0] / c[3], c[1] / c[3]);
            let got = vec2(
                vp.center().x + ndc.x * vp.width() * 0.5,
                vp.center().y - ndc.y * vp.height() * 0.5,
            );
            assert!((got - want).length() < 1e-2, "{p:?}: {got:?} vs {want:?}");
            assert!((c[3] - cam.depth(p)).abs() < 1e-3, "w is the camera depth");
        }
    }

    #[test]
    fn low_speed_guard_and_lines() {
        let c = RoadCurve::from_motion(1.0, 0.5, 0.0);
        assert_eq!(c.kappa, 0.0);
        let curved = RoadCurve::from_motion(0.5, 20.0, 0.0);
        assert!(curved.center_x(40.0) > 0.0, "right turn bends right");
        let cam = Camera25::adas(900.0, 900.0, 0.0, 0.0);
        let mut pts = [Vec2::ZERO; 18];
        assert_eq!(curved.project_line(&cam, -1.8, 1.8, 44.0, &mut pts), 18);
        assert!(pts[0].y > pts[17].y, "near points are lower on screen");
    }
}
