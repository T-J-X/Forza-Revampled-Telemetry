//! Small, allocation-free 2D/3D maths used by the painter and the 2.5D helpers.
//!
//! Screen convention: x to the right, y **down** (pixels). Angles are radians; with y down, a
//! positive angle turns **clockwise** on screen. `0` points to 3 o'clock, `PI / 2` to 6 o'clock.
use std::f32::consts::PI;
use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

/// Degrees to radians.
#[inline]
#[must_use]
pub fn deg(degrees: f32) -> f32 {
    degrees * (PI / 180.0)
}

/// `value` with NaN and infinities replaced by `fallback`.
#[inline]
#[must_use]
pub fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

/// Linear interpolation `a + (b - a) * t`.
#[inline]
#[must_use]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Hermite smoothstep of `x` between `e0` and `e1`.
#[inline]
#[must_use]
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A 2D vector or point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec2 {
    /// X component.
    pub x: f32,
    /// Y component (down on screen).
    pub y: f32,
}

/// Shorthand for [`Vec2::new`].
#[inline]
#[must_use]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { x, y }
}

impl Vec2 {
    /// The origin.
    pub const ZERO: Vec2 = vec2(0.0, 0.0);
    /// (1, 1).
    pub const ONE: Vec2 = vec2(1.0, 1.0);

    /// A vector from components.
    #[inline]
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Both components equal to `v`.
    #[inline]
    #[must_use]
    pub const fn splat(v: f32) -> Self {
        Self { x: v, y: v }
    }

    /// The unit vector at `angle` (clockwise from 3 o'clock on screen).
    #[inline]
    #[must_use]
    pub fn from_angle(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self { x: c, y: s }
    }

    /// `center + from_angle(angle) * radius`.
    #[inline]
    #[must_use]
    pub fn polar(center: Vec2, radius: f32, angle: f32) -> Self {
        center + Self::from_angle(angle) * radius
    }

    /// Dot product.
    #[inline]
    #[must_use]
    pub fn dot(self, o: Vec2) -> f32 {
        self.x * o.x + self.y * o.y
    }

    /// 2D cross product (z of the 3D cross).
    #[inline]
    #[must_use]
    pub fn cross(self, o: Vec2) -> f32 {
        self.x * o.y - self.y * o.x
    }

    /// Euclidean length.
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }

    /// Unit vector in the same direction, or zero for a zero vector.
    #[inline]
    #[must_use]
    pub fn normalize_or_zero(self) -> Self {
        let l = self.length();
        if l > 1e-12 {
            self * (1.0 / l)
        } else {
            Self::ZERO
        }
    }

    /// Rotated by 90 degrees clockwise on screen: (x, y) -> (-y, x).
    #[inline]
    #[must_use]
    pub fn perp(self) -> Self {
        Self { x: -self.y, y: self.x }
    }

    /// Rotated by `angle`.
    #[inline]
    #[must_use]
    pub fn rotate(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
        }
    }

    /// Angle of the vector (clockwise from 3 o'clock).
    #[inline]
    #[must_use]
    pub fn angle(self) -> f32 {
        self.y.atan2(self.x)
    }

    /// Component-wise minimum.
    #[inline]
    #[must_use]
    pub fn min(self, o: Vec2) -> Self {
        Self {
            x: self.x.min(o.x),
            y: self.y.min(o.y),
        }
    }

    /// Component-wise maximum.
    #[inline]
    #[must_use]
    pub fn max(self, o: Vec2) -> Self {
        Self {
            x: self.x.max(o.x),
            y: self.y.max(o.y),
        }
    }

    /// Linear interpolation towards `o`.
    #[inline]
    #[must_use]
    pub fn lerp(self, o: Vec2, t: f32) -> Self {
        self + (o - self) * t
    }

    /// Component-wise product.
    #[inline]
    #[must_use]
    pub fn mul_elem(self, o: Vec2) -> Self {
        Self {
            x: self.x * o.x,
            y: self.y * o.y,
        }
    }

    /// True when both components are finite.
    #[inline]
    #[must_use]
    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    /// Non-finite components replaced by 0.
    #[inline]
    #[must_use]
    pub fn sanitized(self) -> Self {
        Self {
            x: finite_or(self.x, 0.0),
            y: finite_or(self.y, 0.0),
        }
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    #[inline]
    fn add(self, o: Vec2) -> Vec2 {
        vec2(self.x + o.x, self.y + o.y)
    }
}
impl AddAssign for Vec2 {
    #[inline]
    fn add_assign(&mut self, o: Vec2) {
        self.x += o.x;
        self.y += o.y;
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    #[inline]
    fn sub(self, o: Vec2) -> Vec2 {
        vec2(self.x - o.x, self.y - o.y)
    }
}
impl SubAssign for Vec2 {
    #[inline]
    fn sub_assign(&mut self, o: Vec2) {
        self.x -= o.x;
        self.y -= o.y;
    }
}
impl Mul<f32> for Vec2 {
    type Output = Vec2;
    #[inline]
    fn mul(self, s: f32) -> Vec2 {
        vec2(self.x * s, self.y * s)
    }
}
impl Neg for Vec2 {
    type Output = Vec2;
    #[inline]
    fn neg(self) -> Vec2 {
        vec2(-self.x, -self.y)
    }
}
impl From<[f32; 2]> for Vec2 {
    #[inline]
    fn from(v: [f32; 2]) -> Self {
        vec2(v[0], v[1])
    }
}
impl From<(f32, f32)> for Vec2 {
    #[inline]
    fn from(v: (f32, f32)) -> Self {
        vec2(v.0, v.1)
    }
}

/// An axis-aligned rectangle given by its minimum and maximum corners.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    /// Top-left corner.
    pub min: Vec2,
    /// Bottom-right corner.
    pub max: Vec2,
}

impl Rect {
    /// A rectangle from its corners (any order).
    #[inline]
    #[must_use]
    pub fn from_points(a: Vec2, b: Vec2) -> Self {
        Self {
            min: a.min(b),
            max: a.max(b),
        }
    }

    /// A rectangle from x, y, width and height.
    #[inline]
    #[must_use]
    pub fn from_xywh(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self::from_points(vec2(x, y), vec2(x + w, y + h))
    }

    /// A rectangle centred on `center` with the given full size.
    #[inline]
    #[must_use]
    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let h = size * 0.5;
        Self::from_points(center - h, center + h)
    }

    /// An empty rectangle that any union replaces.
    pub const EMPTY: Rect = Rect {
        min: vec2(f32::MAX, f32::MAX),
        max: vec2(f32::MIN, f32::MIN),
    };

    /// Width.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    /// Height.
    #[inline]
    #[must_use]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    /// Size as a vector.
    #[inline]
    #[must_use]
    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Centre point.
    #[inline]
    #[must_use]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    /// True when the rectangle has no area.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        !(self.max.x > self.min.x && self.max.y > self.min.y)
    }

    /// Grown by `d` on every side (negative shrinks).
    #[inline]
    #[must_use]
    pub fn expand(&self, d: f32) -> Self {
        Self {
            min: self.min - Vec2::splat(d),
            max: self.max + Vec2::splat(d),
        }
    }

    /// Shrunk by `d` on every side.
    #[inline]
    #[must_use]
    pub fn inset(&self, d: f32) -> Self {
        self.expand(-d)
    }

    /// The smallest rectangle containing both.
    #[inline]
    #[must_use]
    pub fn union(&self, o: &Rect) -> Self {
        Self {
            min: self.min.min(o.min),
            max: self.max.max(o.max),
        }
    }

    /// The overlap of both (possibly empty).
    #[inline]
    #[must_use]
    pub fn intersect(&self, o: &Rect) -> Self {
        Self {
            min: self.min.max(o.min),
            max: self.max.min(o.max),
        }
    }

    /// True when `p` lies inside (edges included).
    #[inline]
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    /// A point at normalised coordinates (0,0 = top-left, 1,1 = bottom-right).
    #[inline]
    #[must_use]
    pub fn at(&self, u: f32, v: f32) -> Vec2 {
        vec2(lerp(self.min.x, self.max.x, u), lerp(self.min.y, self.max.y, v))
    }

    /// Moved by `d`.
    #[inline]
    #[must_use]
    pub fn translate(&self, d: Vec2) -> Self {
        Self {
            min: self.min + d,
            max: self.max + d,
        }
    }

    /// `[min.x, min.y, max.x, max.y]`.
    #[inline]
    #[must_use]
    pub fn to_array(&self) -> [f32; 4] {
        [self.min.x, self.min.y, self.max.x, self.max.y]
    }
}

/// A 2D affine transform: `p' = (a*x + c*y + tx, b*x + d*y + ty)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Affine {
    /// `[a, b, c, d, tx, ty]`.
    pub m: [f32; 6],
}

impl Default for Affine {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Affine {
    /// The identity transform.
    pub const IDENTITY: Affine = Affine {
        m: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// A translation.
    #[inline]
    #[must_use]
    pub fn translate(t: Vec2) -> Self {
        Self {
            m: [1.0, 0.0, 0.0, 1.0, t.x, t.y],
        }
    }

    /// A uniform scale about the origin.
    #[inline]
    #[must_use]
    pub fn scale(s: f32) -> Self {
        Self::scale_xy(s, s)
    }

    /// A non-uniform scale about the origin.
    #[inline]
    #[must_use]
    pub fn scale_xy(sx: f32, sy: f32) -> Self {
        Self {
            m: [sx, 0.0, 0.0, sy, 0.0, 0.0],
        }
    }

    /// A rotation about the origin (clockwise on screen for positive angles).
    #[inline]
    #[must_use]
    pub fn rotate(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            m: [c, s, -s, c, 0.0, 0.0],
        }
    }

    /// A rotation about `center`.
    #[inline]
    #[must_use]
    pub fn rotate_about(angle: f32, center: Vec2) -> Self {
        Self::translate(-center)
            .then(&Self::rotate(angle))
            .then(&Self::translate(center))
    }

    /// A horizontal shear (`x += k * y`), used for synthetic italics.
    #[inline]
    #[must_use]
    pub fn skew_x(k: f32) -> Self {
        Self {
            m: [1.0, 0.0, k, 1.0, 0.0, 0.0],
        }
    }

    /// `self` applied first, then `next`.
    #[inline]
    #[must_use]
    pub fn then(&self, next: &Affine) -> Self {
        let [a1, b1, c1, d1, x1, y1] = self.m;
        let [a2, b2, c2, d2, x2, y2] = next.m;
        Self {
            m: [
                a2 * a1 + c2 * b1,
                b2 * a1 + d2 * b1,
                a2 * c1 + c2 * d1,
                b2 * c1 + d2 * d1,
                a2 * x1 + c2 * y1 + x2,
                b2 * x1 + d2 * y1 + y2,
            ],
        }
    }

    /// Transforms a point.
    #[inline]
    #[must_use]
    pub fn apply(&self, p: Vec2) -> Vec2 {
        let [a, b, c, d, tx, ty] = self.m;
        vec2(a * p.x + c * p.y + tx, b * p.x + d * p.y + ty)
    }

    /// Transforms a direction (ignores translation).
    #[inline]
    #[must_use]
    pub fn apply_vector(&self, v: Vec2) -> Vec2 {
        let [a, b, c, d, _, _] = self.m;
        vec2(a * v.x + c * v.y, b * v.x + d * v.y)
    }

    /// The average linear scale (`sqrt(|det|)`): target pixels per local unit.
    #[inline]
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        let [a, b, c, d, _, _] = self.m;
        (a * d - b * c).abs().sqrt()
    }

    /// The inverse, or `None` when singular.
    #[must_use]
    pub fn inverse(&self) -> Option<Affine> {
        let [a, b, c, d, tx, ty] = self.m;
        let det = a * d - b * c;
        if det.abs() < 1e-12 || !det.is_finite() {
            return None;
        }
        let inv = 1.0 / det;
        let (ia, ib, ic, id) = (d * inv, -b * inv, -c * inv, a * inv);
        Some(Self {
            m: [ia, ib, ic, id, -(ia * tx + ic * ty), -(ib * tx + id * ty)],
        })
    }

    /// The bounding box of `r` after the transform.
    #[must_use]
    pub fn apply_rect(&self, r: &Rect) -> Rect {
        let p0 = self.apply(r.min);
        let p1 = self.apply(vec2(r.max.x, r.min.y));
        let p2 = self.apply(vec2(r.min.x, r.max.y));
        let p3 = self.apply(r.max);
        Rect {
            min: p0.min(p1).min(p2.min(p3)),
            max: p0.max(p1).max(p2.max(p3)),
        }
    }

    /// True when every coefficient is finite.
    #[inline]
    #[must_use]
    pub fn is_finite(&self) -> bool {
        self.m.iter().all(|v| v.is_finite())
    }
}

/// A column-major 4x4 matrix for the 2.5D and mesh passes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat4 {
    /// Columns, each `[x, y, z, w]`.
    pub cols: [[f32; 4]; 4],
}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// A 3D vector.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    /// X (right).
    pub x: f32,
    /// Y (up).
    pub y: f32,
    /// Z (forward, away from the eye in the kit's camera convention).
    pub z: f32,
}

/// Shorthand for a [`Vec3`].
#[inline]
#[must_use]
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { x, y, z }
}

impl Vec3 {
    /// Dot product.
    #[inline]
    #[must_use]
    pub fn dot(self, o: Vec3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    /// Cross product.
    #[inline]
    #[must_use]
    pub fn cross(self, o: Vec3) -> Vec3 {
        vec3(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }
    /// Length.
    #[inline]
    #[must_use]
    pub fn length(self) -> f32 {
        self.dot(self).sqrt()
    }
    /// Unit vector or zero.
    #[inline]
    #[must_use]
    pub fn normalize_or_zero(self) -> Vec3 {
        let l = self.length();
        if l > 1e-12 {
            self * (1.0 / l)
        } else {
            Vec3::default()
        }
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, o: Vec3) -> Vec3 {
        vec3(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, o: Vec3) -> Vec3 {
        vec3(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, s: f32) -> Vec3 {
        vec3(self.x * s, self.y * s, self.z * s)
    }
}

impl Mat4 {
    /// The identity.
    pub const IDENTITY: Mat4 = Mat4 {
        cols: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// `self * o` (apply `o` first, then `self`).
    #[must_use]
    pub fn mul(&self, o: &Mat4) -> Mat4 {
        let mut r = [[0.0f32; 4]; 4];
        for (c, col) in r.iter_mut().enumerate() {
            for (row, v) in col.iter_mut().enumerate() {
                *v = (0..4).map(|k| self.cols[k][row] * o.cols[c][k]).sum();
            }
        }
        Mat4 { cols: r }
    }

    /// A translation.
    #[must_use]
    pub fn translation(t: Vec3) -> Mat4 {
        let mut m = Self::IDENTITY;
        m.cols[3] = [t.x, t.y, t.z, 1.0];
        m
    }

    /// A uniform scale.
    #[must_use]
    pub fn scale(s: f32) -> Mat4 {
        let mut m = Self::IDENTITY;
        m.cols[0][0] = s;
        m.cols[1][1] = s;
        m.cols[2][2] = s;
        m
    }

    /// A rotation about +Y (yaw) by `angle` radians.
    #[must_use]
    pub fn rotation_y(angle: f32) -> Mat4 {
        let (s, c) = angle.sin_cos();
        Mat4 {
            cols: [
                [c, 0.0, -s, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s, 0.0, c, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// A left-handed view matrix (camera looks along +Z, +Y up) from `eye` towards `target`.
    #[must_use]
    pub fn look_at_lh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = (target - eye).normalize_or_zero();
        let s = up.cross(f).normalize_or_zero();
        let u = f.cross(s);
        Mat4 {
            cols: [
                [s.x, u.x, f.x, 0.0],
                [s.y, u.y, f.y, 0.0],
                [s.z, u.z, f.z, 0.0],
                [-s.dot(eye), -u.dot(eye), -f.dot(eye), 1.0],
            ],
        }
    }

    /// A left-handed perspective projection with depth in `0..1` (WebGPU convention).
    #[must_use]
    pub fn perspective_lh(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y * 0.5).tan();
        let range = far / (far - near);
        Mat4 {
            cols: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, range, 1.0],
                [0.0, 0.0, -range * near, 0.0],
            ],
        }
    }

    /// Transforms a point (w = 1) and returns clip-space `[x, y, z, w]`.
    #[must_use]
    pub fn project(&self, p: Vec3) -> [f32; 4] {
        let c = &self.cols;
        [
            c[0][0] * p.x + c[1][0] * p.y + c[2][0] * p.z + c[3][0],
            c[0][1] * p.x + c[1][1] * p.y + c[2][1] * p.z + c[3][1],
            c[0][2] * p.x + c[1][2] * p.y + c[2][2] * p.z + c[3][2],
            c[0][3] * p.x + c[1][3] * p.y + c[2][3] * p.z + c[3][3],
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-4
    }

    #[test]
    fn affine_composition_order() {
        let t = Affine::scale(2.0).then(&Affine::translate(vec2(10.0, 0.0)));
        assert!(close(t.apply(vec2(1.0, 1.0)), vec2(12.0, 2.0)));
        let inv = t.inverse().expect("invertible");
        assert!(close(inv.apply(vec2(12.0, 2.0)), vec2(1.0, 1.0)));
    }

    #[test]
    fn rotation_is_clockwise_on_screen() {
        let p = Affine::rotate(deg(90.0)).apply(vec2(1.0, 0.0));
        assert!(close(p, vec2(0.0, 1.0)), "3 o'clock turns to 6 o'clock, got {p:?}");
        assert!(close(Vec2::from_angle(deg(90.0)), vec2(0.0, 1.0)));
    }

    #[test]
    fn scale_factor_and_rect() {
        let t = Affine::rotate(0.7).then(&Affine::scale(3.0));
        assert!((t.scale_factor() - 3.0).abs() < 1e-4);
        let r = Affine::scale(2.0).apply_rect(&Rect::from_xywh(1.0, 1.0, 2.0, 2.0));
        assert_eq!(r, Rect::from_xywh(2.0, 2.0, 4.0, 4.0));
    }

    #[test]
    fn perspective_maps_near_far_to_unit_depth() {
        let p = Mat4::perspective_lh(1.0, 1.0, 0.1, 100.0);
        let n = p.project(vec3(0.0, 0.0, 0.1));
        let f = p.project(vec3(0.0, 0.0, 100.0));
        assert!((n[2] / n[3]).abs() < 1e-5);
        assert!((f[2] / f[3] - 1.0).abs() < 1e-5);
    }
}
