//! Physical parts: the "render physical part" helper. Each constructor builds one piece of
//! instrument hardware at a position in local px, with its materials per slot. A [`super::Scene`]
//! renders any set of parts together, so bezels shadow faces and crystals refract dials.
use super::frame::PX;
use super::geom::{self, BezelSpec, Mesh, Outline, Profile};
use super::mat::Mat;
use serde_json::{json, Value};

/// The geometry of a part.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A profile revolved round the part's z axis.
    Lathe(Profile),
    /// A closed outline (absolute local px) extruded `thick` px with a rounded top edge of
    /// `bevel` px (the pillow edge of a leather hood or a machined housing).
    Extrude {
        /// The outline.
        outline: Outline,
        /// Height (px).
        thick: f32,
        /// Top-edge round-over (px).
        bevel: f32,
    },
    /// An explicit mesh relative to the part's position.
    Mesh(Mesh),
}

/// One part.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    /// Object name.
    pub name: String,
    /// Position (local px); ignored by [`Shape::Extrude`], whose outline is absolute.
    pub at: [f32; 2],
    /// Base height (px above the dial plane).
    pub z: f32,
    /// Rotation (degrees clockwise on screen).
    pub rot: f32,
    /// Geometry.
    pub shape: Shape,
    /// Material per slot.
    pub mats: Vec<Mat>,
}

impl Part {
    fn new(name: &str, at: [f32; 2], z: f32, shape: Shape, mats: Vec<Mat>) -> Self {
        Self {
            name: name.into(),
            at,
            z,
            rot: 0.0,
            shape,
            mats,
        }
    }

    /// Any lathe profile.
    #[must_use]
    pub fn lathe(name: &str, at: [f32; 2], z: f32, profile: Profile, mats: Vec<Mat>) -> Self {
        Self::new(name, at, z, Shape::Lathe(profile), mats)
    }

    /// A machined bezel. Slots: inner chamfer (polished), top (brushed / spun), walls (dark),
    /// outer chamfer (knurled or polished).
    #[must_use]
    pub fn bezel(name: &str, at: [f32; 2], z: f32, spec: BezelSpec, mats: [Mat; 4]) -> Self {
        Self::lathe(name, at, z, spec.profile(), mats.to_vec())
    }

    /// A flat dial face of radius `r`.
    #[must_use]
    pub fn face(name: &str, at: [f32; 2], z: f32, r: f32, mat: Mat) -> Self {
        Self::lathe(name, at, z, geom::dial_face(r, 0.0, 0.0), vec![mat])
    }

    /// A dial face with a raised chapter ring from `chapter` px out to `r`.
    #[must_use]
    pub fn chaptered_face(
        name: &str,
        at: [f32; 2],
        z: f32,
        r: f32,
        chapter: f32,
        step: f32,
        face: Mat,
        ring: Mat,
    ) -> Self {
        Self::lathe(name, at, z, geom::dial_face(r, chapter, step), vec![face, ring])
    }

    /// A plain ring (flat annulus `h` px tall with rounded edges): bearing rings, collars.
    #[must_use]
    pub fn ring(name: &str, at: [f32; 2], z: f32, r_in: f32, r_out: f32, h: f32, mat: Mat) -> Self {
        let f = ((r_out - r_in) * 0.3).min(h * 0.45);
        let p = Profile::new()
            .sharp(r_in, 0.0, 0)
            .round(r_in, h, 0, f)
            .round(r_out, h, 0, f)
            .sharp(r_out, 0.0, 0)
            .build();
        Self::lathe(name, at, z, p, vec![mat])
    }

    /// A crystal (sapphire or glass): `t` px thick, `dome` px of crown, bevelled edge.
    #[must_use]
    pub fn crystal(name: &str, at: [f32; 2], z: f32, r: f32, t: f32, dome: f32, mat: Mat) -> Self {
        Self::lathe(
            name,
            at,
            z,
            geom::crystal(r, t, dome, (t * 0.5).min(r * 0.1)),
            vec![mat],
        )
    }

    /// A dome on a collar: hub caps, LED lenses, jewel caps. Slots: dome, collar.
    #[must_use]
    pub fn dome(name: &str, at: [f32; 2], z: f32, r: f32, h: f32, collar: f32, mats: [Mat; 2]) -> Self {
        Self::lathe(name, at, z, geom::dome(r, h, collar), mats.to_vec())
    }

    /// An extruded housing or surround.
    #[must_use]
    pub fn extrude(name: &str, outline: Outline, z: f32, thick: f32, bevel: f32, mat: Mat) -> Self {
        Self::new(name, [0.0, 0.0], z, Shape::Extrude { outline, thick, bevel }, vec![mat])
    }

    /// A mesh part (needles, stitches).
    #[must_use]
    pub fn mesh(name: &str, at: [f32; 2], z: f32, mesh: Mesh, mats: Vec<Mat>) -> Self {
        Self::new(name, at, z, Shape::Mesh(mesh), mats)
    }

    /// Rotated `deg` clockwise on screen about its position.
    #[must_use]
    pub fn rotated(mut self, deg: f32) -> Self {
        self.rot = deg;
        self
    }

    /// Highest point (px above the dial plane), for the height pass range.
    #[must_use]
    pub fn top(&self) -> f32 {
        self.z
            + match &self.shape {
                Shape::Lathe(p) => p.extents().3,
                Shape::Extrude { thick, .. } => *thick,
                Shape::Mesh(m) => m.verts.iter().map(|v| v[2]).fold(0.0, f32::max),
            }
    }

    /// JSON for Blender (BU, Blender's y-up).
    #[must_use]
    pub fn to_json(&self) -> Value {
        let mats: Vec<&str> = self.mats.iter().map(Mat::name).collect();
        let base = json!({
            "name": self.name, "loc": [self.at[0] * PX, -self.at[1] * PX, self.z * PX],
            "rot": -self.rot.to_radians(), "mats": mats,
        });
        let extra = match &self.shape {
            Shape::Lathe(p) => json!({
                "type": "lathe",
                "profile": p.points.iter().map(|q| json!([q.r * PX, q.z * PX, q.mat, q.sharp])).collect::<Vec<_>>(),
                "segs": if p.extents().1 > 400.0 { 512 } else { 256 },
            }),
            Shape::Extrude { outline, thick, bevel } => json!({
                "type": "extrude",
                "outline": outline.points.iter().map(|p| [p[0] * PX, -p[1] * PX]).collect::<Vec<_>>(),
                "thick": thick * PX, "bevel": bevel * PX,
            }),
            Shape::Mesh(m) => json!({
                "type": "mesh",
                "verts": m.verts.iter().map(|v| [v[0] * PX, -v[1] * PX, v[2] * PX]).collect::<Vec<_>>(),
                "faces": m.faces, "fmats": m.mats, "smooth": m.smooth,
            }),
        };
        let mut out = base;
        if let (Some(o), Some(e)) = (out.as_object_mut(), extra.as_object()) {
            for (k, v) in e {
                o.insert(k.clone(), v.clone());
            }
        }
        out
    }
}

/// Saddle-stitch threads along a closed outline: one small raised capsule every `pitch` px,
/// `len` px long and `width` px wide, `height` px proud. Returns one mesh in absolute local px.
#[must_use]
pub fn stitches(outline: &Outline, pitch: f32, len: f32, width: f32, height: f32) -> Mesh {
    let pts = &outline.points;
    let n = pts.len();
    let mut mesh = Mesh {
        smooth: true,
        ..Mesh::default()
    };
    if n < 3 || pitch <= 0.0 {
        return mesh;
    }
    let seg_len = |i: usize| {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
    };
    let total: f32 = (0..n).map(seg_len).sum();
    let count = (total / pitch).floor().max(1.0) as usize;
    let step = total / count as f32;
    let (mut i, mut acc) = (0usize, 0.0f32);
    for k in 0..count {
        let s = k as f32 * step;
        while i < n && acc + seg_len(i) < s {
            acc += seg_len(i);
            i += 1;
        }
        let (a, b) = (pts[i % n], pts[(i + 1) % n]);
        let l = seg_len(i % n).max(1e-4);
        let t = (s - acc) / l;
        let (dx, dy) = ((b[0] - a[0]) / l, (b[1] - a[1]) / l);
        let (cx, cy) = (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t);
        let (nx, ny) = (-dy, dx);
        let p = |u: f32, v: f32, z: f32| [cx + dx * u + nx * v, cy + dy * u + ny * v, z];
        let base = mesh.verts.len() as u32;
        let (hl, hw) = (len * 0.5, width * 0.5);
        mesh.verts.extend([
            p(-hl, 0.0, 0.0),
            p(-hl * 0.6, -hw, 0.0),
            p(hl * 0.6, -hw, 0.0),
            p(hl, 0.0, 0.0),
            p(hl * 0.6, hw, 0.0),
            p(-hl * 0.6, hw, 0.0),
            p(-hl * 0.6, 0.0, height),
            p(hl * 0.6, 0.0, height),
        ]);
        for f in [
            vec![0, 1, 6],
            vec![1, 2, 7, 6],
            vec![2, 3, 7],
            vec![3, 4, 7],
            vec![4, 5, 6, 7],
            vec![5, 0, 6],
        ] {
            mesh.faces.push(f.into_iter().map(|v: u32| base + v).collect());
            mesh.mats.push(0);
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::mat;

    #[test]
    fn bezel_json_is_in_blender_units_with_y_flipped() {
        let m = mat::polished(mat::metal::TITANIUM);
        let p = Part::bezel(
            "tach_bezel",
            [0.0, -14.0],
            0.0,
            BezelSpec::ring(226.0, 18.0),
            [m.clone(), m.clone(), m.clone(), m],
        );
        let j = p.to_json();
        assert_eq!(j["type"], "lathe");
        assert!((j["loc"][1].as_f64().unwrap_or(0.0) - 0.14).abs() < 1e-6);
        assert!(j["profile"].as_array().is_some_and(|a| a.len() > 10));
        assert!(p.top() > 9.0);
    }

    #[test]
    fn stitches_follow_the_outline_at_their_pitch() {
        let o = Outline::rrect(0.0, 0.0, 400.0, 200.0, 40.0, 16);
        let m = stitches(&o, 12.0, 8.0, 2.5, 1.5);
        let per = 6;
        let count = m.faces.len() / per;
        // Perimeter ≈ 2(320 + 120) + 2π·40 ≈ 1131 px → about 94 stitches.
        assert!((90..=96).contains(&count), "{count}");
        assert_eq!(m.faces.len(), m.mats.len());
    }
}
