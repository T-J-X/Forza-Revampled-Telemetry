//! Geometry for hero parts, computed in Rust (testable) and handed to Blender as plain data.
//!
//! Units are **local 4K px** (the kit's drawing units at user scale 1.0): x right, y **down**,
//! z toward the viewer. Blender receives the numbers unchanged and converts with `PX` (Blender
//! units per px) in [`super::bpy::PARTS`].
//!
//! - **Lathe profiles** ([`Profile`]): `(r, z)` polylines revolved round the part's axis. Bezels,
//!   rings, dial faces, crystals, hub caps, LED caps and counterweights are all lathes. Each
//!   point carries the material slot of the band that starts at it and whether its edge is
//!   sharp, so one bezel can be polished on the chamfer and brushed on the flat.
//! - **Outlines** ([`Outline`]): closed CCW polygons for extruded parts (housings, pods,
//!   surrounds, visors), including a smooth union of circles for binnacle shrouds.
//! - **Meshes** ([`Mesh`]): explicit vertices and faces (needles).
use serde_json::{json, Value};
use std::f32::consts::{PI, TAU};

/// One profile point: radius and height (px), the material slot of the band from this point to
/// the next, and whether the edge at this point stays sharp in shading.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProfilePoint {
    /// Distance from the axis (px, ≥ 0).
    pub r: f32,
    /// Height above the part's base (px).
    pub z: f32,
    /// Material slot of the band that starts here.
    pub mat: u8,
    /// Keep a crisp edge here (no smoothing across it).
    pub sharp: bool,
}

/// A lathe profile, ordered so the revolved surface faces outward (walk it with the solid on
/// your left in the `(r, z)` plane: inner wall upward, top outward, outer wall downward).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Profile {
    /// The points.
    pub points: Vec<ProfilePoint>,
    /// Fillets waiting for [`Profile::build`]: (point index, radius px).
    pending: Vec<(usize, f32)>,
}

impl Profile {
    /// Starts an empty profile.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a sharp point.
    #[must_use]
    pub fn sharp(mut self, r: f32, z: f32, mat: u8) -> Self {
        self.points.push(ProfilePoint {
            r: r.max(0.0),
            z,
            mat,
            sharp: true,
        });
        self
    }

    /// Appends a point with a fillet of `radius` px (0 keeps it sharp).
    #[must_use]
    pub fn round(self, r: f32, z: f32, mat: u8, radius: f32) -> Self {
        self.with_fillet(r, z, mat, radius)
    }

    fn with_fillet(mut self, r: f32, z: f32, mat: u8, radius: f32) -> Self {
        // The fillet radius rides on `sharp = false` + a pending radius list resolved in `build`.
        self.points.push(ProfilePoint {
            r: r.max(0.0),
            z,
            mat,
            sharp: radius <= 0.0,
        });
        self.pending.push((self.points.len() - 1, radius));
        self
    }

    /// Resolves the fillets: every point added with [`Profile::round`] is replaced by an arc.
    #[must_use]
    pub fn build(self) -> Profile {
        let mut radii = vec![0.0_f32; self.points.len()];
        for (i, r) in &self.pending {
            if let Some(slot) = radii.get_mut(*i) {
                *slot = *r;
            }
        }
        Profile {
            points: fillet(&self.points, &radii, 6),
            pending: Vec::new(),
        }
    }

    /// The radial and height extents `(r_min, r_max, z_min, z_max)`.
    #[must_use]
    pub fn extents(&self) -> (f32, f32, f32, f32) {
        let mut e = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for p in &self.points {
            e.0 = e.0.min(p.r);
            e.1 = e.1.max(p.r);
            e.2 = e.2.min(p.z);
            e.3 = e.3.max(p.z);
        }
        e
    }

    /// JSON for Blender: `[[r, z, mat, sharp], ...]`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::Array(self.points.iter().map(|p| json!([p.r, p.z, p.mat, p.sharp])).collect())
    }
}

/// Replaces corner `i` by a circular arc of `radii[i]` px (clamped to half the shorter
/// neighbouring segment). End points are never filleted. Arc points take the corner's material
/// and are smooth; the arc's last point keeps the material of the outgoing band.
#[must_use]
pub fn fillet(points: &[ProfilePoint], radii: &[f32], segs_per_quarter: usize) -> Vec<ProfilePoint> {
    let n = points.len();
    let mut out = Vec::with_capacity(n * 4);
    for i in 0..n {
        let b = points[i];
        let rad = radii.get(i).copied().unwrap_or(0.0);
        if i == 0 || i + 1 == n || rad <= 0.0 {
            out.push(b);
            continue;
        }
        let a = points[i - 1];
        let c = points[i + 1];
        let (ux, uz) = (a.r - b.r, a.z - b.z);
        let (vx, vz) = (c.r - b.r, c.z - b.z);
        let (la, lc) = ((ux * ux + uz * uz).sqrt(), (vx * vx + vz * vz).sqrt());
        if la < 1e-4 || lc < 1e-4 {
            out.push(b);
            continue;
        }
        let (ux, uz, vx, vz) = (ux / la, uz / la, vx / lc, vz / lc);
        let cos = (ux * vx + uz * vz).clamp(-1.0, 1.0);
        let theta = cos.acos();
        if theta < 1e-3 || (PI - theta) < 1e-3 {
            out.push(b);
            continue;
        }
        let half = theta * 0.5;
        let mut t = rad / half.tan();
        t = t.min(0.5 * la.min(lc));
        let rho = t * half.tan();
        let (t1r, t1z) = (b.r + ux * t, b.z + uz * t);
        let (t2r, t2z) = (b.r + vx * t, b.z + vz * t);
        let (bx, bz) = (ux + vx, uz + vz);
        let bl = (bx * bx + bz * bz).sqrt().max(1e-6);
        let d = rho / half.sin();
        let (or, oz) = (b.r + bx / bl * d, b.z + bz / bl * d);
        let a1 = (t1z - oz).atan2(t1r - or);
        let a2 = (t2z - oz).atan2(t2r - or);
        let mut sweep = a2 - a1;
        while sweep > PI {
            sweep -= TAU;
        }
        while sweep < -PI {
            sweep += TAU;
        }
        let steps = ((sweep.abs() / (PI * 0.5)) * segs_per_quarter as f32).ceil().max(2.0) as usize;
        for k in 0..=steps {
            let ang = a1 + sweep * k as f32 / steps as f32;
            out.push(ProfilePoint {
                r: (or + rho * ang.cos()).max(0.0),
                z: oz + rho * ang.sin(),
                mat: b.mat,
                sharp: false,
            });
        }
    }
    out
}

/// A machined instrument bezel (all px). Material slots: 0 = the polished inner chamfer, 1 = the
/// brushed top, 2 = the walls (dark, in shadow), 3 = the outer chamfer (knurled or polished).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BezelSpec {
    /// Radius of the opening (where the dial face shows).
    pub r_in: f32,
    /// Outer radius.
    pub r_out: f32,
    /// Height of the top above the dial plane.
    pub height: f32,
    /// How far the inner wall drops below the dial plane (hides the face's edge).
    pub drop: f32,
    /// Width of the polished 45° inner chamfer (the bright ring that reads as "machined").
    pub chamfer_in: f32,
    /// Width of the outer chamfer.
    pub chamfer_out: f32,
    /// Fillet on every machined edge (catches a thin highlight).
    pub fillet: f32,
    /// A step down on the top's outer half (0 = flat top), for a two-level bezel.
    pub step: f32,
}

impl BezelSpec {
    /// A bezel `width` px wide around an opening of `r_in` px, proportioned like a machined
    /// motorsport ring (height 0.55 × width, chamfers 0.28 and 0.18 × width).
    #[must_use]
    pub fn ring(r_in: f32, width: f32) -> Self {
        Self {
            r_in,
            r_out: r_in + width,
            height: width * 0.55,
            drop: width * 0.4,
            chamfer_in: width * 0.28,
            chamfer_out: width * 0.18,
            fillet: (width * 0.04).max(0.6),
            step: 0.0,
        }
    }

    /// The lathe profile.
    #[must_use]
    pub fn profile(&self) -> Profile {
        let w = self.r_out - self.r_in;
        let ci = self.chamfer_in.clamp(0.0, w * 0.45);
        let co = self.chamfer_out.clamp(0.0, w * 0.3);
        let h = self.height.max(ci + 0.5);
        let f = self.fillet;
        let mut p = Profile::new()
            .sharp(self.r_in, -self.drop, 2)
            .round(self.r_in, h - ci, 0, f)
            .round(self.r_in + ci, h, 1, f);
        if self.step > 0.0 {
            let mid = self.r_in + ci + (w - ci - co) * 0.55;
            p = p.round(mid, h, 1, f).round(mid + self.step * 0.5, h - self.step, 1, f);
            p = p.round(self.r_out - co, h - self.step, 3, f);
            p = p.round(self.r_out, h - self.step - co, 2, f);
        } else {
            p = p.round(self.r_out - co, h, 3, f).round(self.r_out, h - co, 2, f);
        }
        p.sharp(self.r_out, -self.drop, 2).build()
    }
}

/// A sapphire or mineral crystal: flat or domed top, bevelled edge, flat underside (a closed
/// solid, so Cycles refracts through it). Slot 0 = the glass.
#[must_use]
pub fn crystal(r: f32, thickness: f32, dome: f32, edge_bevel: f32) -> Profile {
    let mut p = Profile::new();
    let n = 24;
    let top = |rr: f32| thickness + dome * (1.0 - (rr / r).powi(2)).max(0.0);
    let inner = (r - edge_bevel).max(r * 0.5);
    for k in 0..=n {
        let rr = inner * k as f32 / n as f32;
        p.points.push(ProfilePoint {
            r: rr,
            z: top(rr),
            mat: 0,
            sharp: false,
        });
    }
    p.points.push(ProfilePoint {
        r,
        z: (thickness - edge_bevel).max(thickness * 0.4),
        mat: 0,
        sharp: false,
    });
    p.points.push(ProfilePoint {
        r,
        z: 0.0,
        mat: 0,
        sharp: true,
    });
    p.points.push(ProfilePoint {
        r: 0.0,
        z: 0.0,
        mat: 0,
        sharp: true,
    });
    p
}

/// A flat dial face of radius `r` with a raised chapter ring step at `chapter` (0 = none).
/// Slot 0 = the face, slot 1 = the chapter ring.
#[must_use]
pub fn dial_face(r: f32, chapter: f32, chapter_h: f32) -> Profile {
    let mut p = Profile::new().sharp(0.0, 0.0, 0);
    if chapter > 0.0 && chapter < r {
        p = p
            .sharp(chapter, 0.0, 0)
            .round(chapter, chapter_h, 1, chapter_h * 0.3)
            .sharp(r, chapter_h, 1);
        p.sharp(r, -2.0, 1).build()
    } else {
        p.sharp(r, 0.0, 0).sharp(r, -2.0, 0).build()
    }
}

/// A domed hub cap or LED lens: radius `r`, total height `h`, a cylindrical collar `collar` px
/// tall under the dome. Slot 0 = dome, slot 1 = collar.
#[must_use]
pub fn dome(r: f32, h: f32, collar: f32) -> Profile {
    let mut p = Profile::new();
    let n = 20;
    let dh = (h - collar).max(0.5);
    for k in 0..=n {
        let a = (k as f32 / n as f32) * PI * 0.5;
        p.points.push(ProfilePoint {
            r: r * a.sin(),
            z: collar + dh * a.cos(),
            mat: 0,
            sharp: false,
        });
    }
    if collar > 0.0 {
        p.points.push(ProfilePoint {
            r,
            z: 0.0,
            mat: 1,
            sharp: true,
        });
    }
    p.points.push(ProfilePoint {
        r,
        z: -1.0,
        mat: 1,
        sharp: true,
    });
    p
}

/// A closed CCW outline in local px (x right, y down).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outline {
    /// The vertices.
    pub points: Vec<[f32; 2]>,
}

impl Outline {
    /// A rounded rectangle centred on `(cx, cy)`.
    #[must_use]
    pub fn rrect(cx: f32, cy: f32, w: f32, h: f32, r: f32, segs: usize) -> Self {
        let r = r.min(w * 0.5).min(h * 0.5).max(0.0);
        let (hx, hy) = (w * 0.5 - r, h * 0.5 - r);
        let mut points = Vec::new();
        for (sx, sy, a0) in [
            (1.0, 1.0, 0.0),
            (-1.0, 1.0, 90.0),
            (-1.0, -1.0, 180.0),
            (1.0, -1.0, 270.0_f32),
        ] {
            for i in 0..=segs {
                let a = (a0 + 90.0 * i as f32 / segs as f32).to_radians();
                points.push([cx + sx * hx + r * a.cos(), cy + sy * hy + r * a.sin()]);
            }
        }
        Self { points }.dedup()
    }

    /// The smooth union of circles `(cx, cy, r)`, filleted with blend width `k` px, sampled at
    /// `n` angles from `(ox, oy)` (the shape must be star-shaped from there).
    #[must_use]
    pub fn circle_union(circles: &[(f32, f32, f32)], k: f32, origin: (f32, f32), n: usize) -> Self {
        let sdf = |x: f32, y: f32| -> f32 {
            let mut d = f32::MAX;
            for &(cx, cy, r) in circles {
                let di = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r;
                d = if d == f32::MAX { di } else { smin(d, di, k) };
            }
            d
        };
        let reach = circles
            .iter()
            .map(|&(cx, cy, r)| ((cx - origin.0).powi(2) + (cy - origin.1).powi(2)).sqrt() + r + k)
            .fold(0.0_f32, f32::max)
            * 1.5;
        let mut points = Vec::with_capacity(n);
        for i in 0..n {
            let a = TAU * i as f32 / n as f32;
            let (dx, dy) = (a.cos(), a.sin());
            // March out from the origin to the last inside sample, then bisect the crossing.
            let steps = 400;
            let mut lo = 0.0;
            for s in 1..=steps {
                let t = reach * s as f32 / steps as f32;
                if sdf(origin.0 + dx * t, origin.1 + dy * t) <= 0.0 {
                    lo = t;
                }
            }
            let mut hi = lo + reach / steps as f32;
            let mut lo2 = lo;
            for _ in 0..30 {
                let mid = 0.5 * (lo2 + hi);
                if sdf(origin.0 + dx * mid, origin.1 + dy * mid) <= 0.0 {
                    lo2 = mid;
                } else {
                    hi = mid;
                }
            }
            points.push([origin.0 + dx * lo2, origin.1 + dy * lo2]);
        }
        Self { points }
    }

    /// Signed area in the y-down plane (positive = clockwise on screen = CCW in Blender's y-up).
    #[must_use]
    pub fn area(&self) -> f32 {
        let n = self.points.len();
        let mut s = 0.0;
        for i in 0..n {
            let [x0, y0] = self.points[i];
            let [x1, y1] = self.points[(i + 1) % n];
            s += x0 * y1 - x1 * y0;
        }
        s * 0.5
    }

    /// Bounding box `(x0, y0, x1, y1)`.
    #[must_use]
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let mut b = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &[x, y] in &self.points {
            b.0 = b.0.min(x);
            b.1 = b.1.min(y);
            b.2 = b.2.max(x);
            b.3 = b.3.max(y);
        }
        b
    }

    fn dedup(mut self) -> Self {
        self.points
            .dedup_by(|a, b| (a[0] - b[0]).abs() + (a[1] - b[1]).abs() < 1e-4);
        if self.points.len() > 2 {
            let (f, l) = (self.points[0], self.points[self.points.len() - 1]);
            if (f[0] - l[0]).abs() + (f[1] - l[1]).abs() < 1e-4 {
                self.points.pop();
            }
        }
        self
    }

    /// JSON for Blender: `[[x, y], ...]`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        Value::Array(self.points.iter().map(|p| json!(p)).collect())
    }
}

/// Polynomial smooth minimum (blend width `k`).
#[must_use]
pub fn smin(a: f32, b: f32, k: f32) -> f32 {
    if k <= 0.0 {
        return a.min(b);
    }
    let h = (k - (a - b).abs()).max(0.0) / k;
    a.min(b) - h * h * k * 0.25
}

/// An explicit mesh in local px: vertices `[x, y, z]` and polygons (vertex indices, CCW seen
/// from +z), each polygon with a material slot.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// Vertices.
    pub verts: Vec<[f32; 3]>,
    /// Polygons.
    pub faces: Vec<Vec<u32>>,
    /// Material slot per polygon.
    pub mats: Vec<u8>,
    /// Shade smooth (false = flat facets).
    pub smooth: bool,
}

impl Mesh {
    /// JSON for Blender.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "verts": self.verts, "faces": self.faces, "mats": self.mats, "smooth": self.smooth })
    }
}

/// A faceted dauphine blade along the screen's −y axis, pivot at the origin. The blade runs from
/// `base` px to `tip` px from the pivot (both measured up the screen; negative values point
/// down, which is how counterweights are made). Two top facets meet in a ridge so one side
/// catches the key and the other falls dark: that is what makes a needle read as metal.
/// `half_w` = half width at the widest point, found at `widest` (0..1) of the way from base to
/// tip; `ridge` = ridge height; `lume_w` > 0 adds a raised lume strip from 45% to 90%.
/// Slot 0 = facets, slot 1 = lume, slot 2 = underside.
#[must_use]
pub fn blade(base: f32, tip: f32, half_w: f32, widest: f32, ridge: f32, lume_w: f32) -> Mesh {
    let at = |t: f32| -(base + (tip - base) * t);
    let yw = at(widest.clamp(0.05, 0.9));
    let ys = at(widest.clamp(0.05, 0.9) * 0.35);
    let verts = vec![
        [0.0, at(1.0), 0.0],       // 0 tip
        [-half_w, yw, 0.0],        // 1 left shoulder
        [0.0, yw, ridge],          // 2 ridge peak
        [half_w, yw, 0.0],         // 3 right shoulder
        [0.0, at(0.0), 0.0],       // 4 base point
        [-half_w * 0.75, ys, 0.0], // 5 left base shoulder
        [half_w * 0.75, ys, 0.0],  // 6 right base shoulder
        [0.0, ys, ridge * 0.8],    // 7 base ridge
    ];
    let mut faces: Vec<Vec<u32>> = vec![
        vec![0, 2, 1],
        vec![0, 3, 2],
        vec![1, 2, 7, 5],
        vec![2, 3, 6, 7],
        vec![5, 7, 4],
        vec![7, 6, 4],
    ];
    let mut mats = vec![0u8; faces.len()];
    faces.push(vec![0, 1, 5, 4, 6, 3]);
    mats.push(2);
    let mut mesh = Mesh {
        verts,
        faces,
        mats,
        smooth: false,
    };
    if lume_w > 0.0 {
        // Ridge height along the centre line between the peak (at `widest`) and the tip.
        let w = widest.clamp(0.05, 0.9);
        let z_at = |t: f32| {
            if t <= w {
                ridge
            } else {
                ridge * (1.0 - (t - w) / (1.0 - w))
            }
        };
        let b = mesh.verts.len() as u32;
        let lw = lume_w * 0.5;
        for &(t, sx) in &[(0.45_f32, -1.0_f32), (0.45, 1.0), (0.9, 1.0), (0.9, -1.0)] {
            mesh.verts.push([sx * lw * (1.15 - t * 0.5), at(t), z_at(t) + 0.6]);
        }
        mesh.faces.push(vec![b, b + 1, b + 2, b + 3]);
        mesh.mats.push(1);
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezel_profile_spans_the_ring_and_rises_to_its_height() {
        let b = BezelSpec::ring(226.0, 18.0);
        let p = b.profile();
        let (r0, r1, z0, z1) = p.extents();
        assert!((r0 - 226.0).abs() < 1e-3, "r_min {r0}");
        assert!((r1 - 244.0).abs() < 1e-3, "r_max {r1}");
        assert!(z0 < 0.0 && (z1 - b.height).abs() < 0.2, "z {z0}..{z1}");
        // Fillets add smooth arc points; the first and last stay sharp.
        assert!(p.points.len() > 12);
        assert!(p.points[0].sharp && p.points[p.points.len() - 1].sharp);
        assert!(p.points.iter().any(|q| !q.sharp));
        // Every point is inside the ring.
        assert!(p.points.iter().all(|q| q.r >= 226.0 - 1e-3 && q.r <= 244.0 + 1e-3));
    }

    #[test]
    fn fillet_keeps_points_on_both_tangents() {
        let pts = [
            ProfilePoint {
                r: 0.0,
                z: 0.0,
                mat: 0,
                sharp: true,
            },
            ProfilePoint {
                r: 10.0,
                z: 0.0,
                mat: 0,
                sharp: true,
            },
            ProfilePoint {
                r: 10.0,
                z: 10.0,
                mat: 0,
                sharp: true,
            },
        ];
        let out = fillet(&pts, &[0.0, 2.0, 0.0], 6);
        // The corner (10, 0) is gone; the arc starts at (8, 0) and ends at (10, 2).
        assert!(out.iter().all(|p| !((p.r - 10.0).abs() < 1e-3 && p.z.abs() < 1e-3)));
        assert!(out.iter().any(|p| (p.r - 8.0).abs() < 1e-3 && p.z.abs() < 1e-3));
        assert!(out
            .iter()
            .any(|p| (p.r - 10.0).abs() < 1e-3 && (p.z - 2.0).abs() < 1e-3));
        // Every arc point is 2 px from the centre (8, 2).
        for p in &out[1..out.len() - 1] {
            let d = ((p.r - 8.0).powi(2) + (p.z - 2.0).powi(2)).sqrt();
            assert!((d - 2.0).abs() < 1e-3, "arc radius {d}");
        }
    }

    #[test]
    fn crystal_and_dome_close_on_the_axis() {
        let c = crystal(96.0, 6.0, 4.0, 1.5);
        assert_eq!(c.points[0].r, 0.0);
        assert_eq!(c.points[c.points.len() - 1].r, 0.0);
        assert!((c.points[0].z - 10.0).abs() < 1e-4);
        let d = dome(12.0, 8.0, 2.0);
        assert!(d.points[0].r.abs() < 1e-4 && (d.points[0].z - 8.0).abs() < 1e-4);
    }

    #[test]
    fn circle_union_covers_every_circle_and_fills_the_waist() {
        let circles = [(-416.0, 24.0, 196.0), (0.0, -14.0, 260.0), (416.0, 24.0, 196.0)];
        let o = Outline::circle_union(&circles, 60.0, (0.0, 0.0), 720);
        let (x0, y0, x1, y1) = o.bounds();
        assert!(x0 <= -611.0 && x1 >= 611.0, "x {x0}..{x1}");
        assert!(y0 <= -273.0 && y1 >= 219.0, "y {y0}..{y1}");
        // The fillet pushes the waist between two circles beyond the bare circles.
        let waist = o
            .points
            .iter()
            .filter(|p| (p[0] - 230.0).abs() < 6.0 && p[1] < 0.0)
            .map(|p| p[1])
            .fold(f32::MAX, f32::min);
        let bare = -14.0 - (260.0_f32.powi(2) - 230.0_f32.powi(2)).sqrt();
        assert!(waist < bare, "waist {waist} vs bare {bare}");
        assert!(o.area().abs() > 300_000.0);
    }

    #[test]
    fn rrect_is_closed_without_duplicates() {
        let o = Outline::rrect(0.0, 0.0, 100.0, 40.0, 10.0, 8);
        let (x0, y0, x1, y1) = o.bounds();
        assert!((x0 + 50.0).abs() < 1e-3 && (x1 - 50.0).abs() < 1e-3);
        assert!((y0 + 20.0).abs() < 1e-3 && (y1 - 20.0).abs() < 1e-3);
        let f = o.points[0];
        let l = o.points[o.points.len() - 1];
        assert!((f[0] - l[0]).abs() + (f[1] - l[1]).abs() > 1e-3);
    }

    #[test]
    fn blade_points_up_with_a_raised_ridge() {
        let m = blade(104.0, 208.0, 9.0, 0.25, 4.0, 6.0);
        let min_y = m.verts.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
        let max_z = m.verts.iter().map(|v| v[2]).fold(f32::MIN, f32::max);
        assert!((min_y + 208.0).abs() < 1e-3);
        assert!((max_z - 4.0).abs() < 1e-3);
        assert_eq!(m.mats.iter().filter(|&&s| s == 1).count(), 1, "one lume insert");
        assert_eq!(m.faces.len(), m.mats.len());
        assert!(m.faces.iter().flatten().all(|&i| (i as usize) < m.verts.len()));
    }
}
