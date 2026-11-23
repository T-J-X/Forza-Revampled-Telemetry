//! Hero asset library (asset pipeline v2): physical instrument hardware rendered in Blender at
//! full strength and registered 1:1 to the kit's local 4K px, plus ComfyUI texture and detail
//! passes. Cluster recipes call this instead of writing their own bpy.
//!
//! ```ignore
//! let mut s = Scene::new("porsche_hw", Frame::envelope(&frame::COCKPIT));
//! s.rig = Rig::hero(1300.0, -135.0);
//! s.add(Part::bezel("tach", [0.0, -14.0], 0.0, BezelSpec::ring(226.0, 18.0), [...]));
//! let out = s.render(&paths, &dir)?; // beauty_0..2.png, normal.png, height.png, depth.png …
//! ```
//!
//! - [`frame`]: camera matching (ortho, pixel-exact; or a lens), anchors and envelopes.
//! - [`rig`]: calibrated studio rigs in the kit's light convention, plus the G-swing azimuths.
//! - [`mat`]: the PBR cookbook as data (metals by measured F0, finishes, glass, leather …).
//! - [`geom`]: lathe profiles with fillets, outlines, needle blades, all in px.
//! - [`part`]: bezels, faces, rings, crystals, domes, housings, needles, stitches.
//! - [`bpy`]: the Blender side, a Rust string constant.
//! - [`sdxl`] and [`texture`]: ComfyUI graphs (txt2img, refiner, ControlNet, seamless, detail
//!   pass) and the per-material prompt cookbook.
pub mod bpy;
pub mod demo;
pub mod frame;
pub mod geom;
pub mod mat;
pub mod part;
pub mod rig;
pub mod sdxl;
pub mod texture;

use crate::error::{io_err, Result};
use crate::img::{self, Img};
use crate::paths::Paths;
use frame::Frame;
use mat::Mat;
use part::Part;
use rig::Rig;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Which passes to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Passes {
    /// Beauty at the rest key.
    pub beauty: bool,
    /// Beauty at every extra key azimuth too (the G swing).
    pub swing: bool,
    /// Normal, height and ControlNet depth.
    pub data: bool,
    /// Ambient occlusion.
    pub ao: bool,
    /// Roughness / metallic / height.
    pub rmh: bool,
    /// Emitters only (lume, LEDs), for an Add layer.
    pub emission: bool,
}

impl Passes {
    /// Every pass.
    pub const ALL: Passes = Passes {
        beauty: true,
        swing: true,
        data: true,
        ao: true,
        rmh: true,
        emission: true,
    };
    /// Just the rest-key beauty (previews, swatches).
    pub const BEAUTY: Passes = Passes {
        beauty: true,
        swing: false,
        data: false,
        ao: false,
        rmh: false,
        emission: false,
    };
}

/// A hero scene: parts, a frame, rigs and passes.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    /// Name (scratch script and log names).
    pub name: String,
    /// What the camera sees.
    pub frame: Frame,
    /// The rest rig.
    pub rig: Rig,
    /// Extra key azimuths for the swing passes (each re-aims the rest rig's lights).
    pub swing: Vec<f32>,
    /// The parts.
    pub parts: Vec<Part>,
    /// Cycles samples for beauty.
    pub samples: u32,
    /// Passes.
    pub passes: Passes,
    /// AO distance (px).
    pub ao_px: f32,
}

/// Where a render's passes landed.
#[derive(Debug, Clone, PartialEq)]
pub struct Rendered {
    /// The output folder.
    pub dir: PathBuf,
    /// Beauty plates, rest key first, then one per swing azimuth.
    pub beauty: Vec<PathBuf>,
    /// Tangent-space normal map (RGBA16, OpenGL convention: +G = up the screen).
    pub normal: Option<PathBuf>,
    /// Height (16-bit grey + alpha).
    pub height: Option<PathBuf>,
    /// ControlNet depth (8-bit grey, near = white, black background).
    pub depth: Option<PathBuf>,
    /// Ambient occlusion (16-bit grey + alpha).
    pub ao: Option<PathBuf>,
    /// Roughness, metallic, height (RGBA16).
    pub rmh: Option<PathBuf>,
    /// Emitters only (RGBA16).
    pub emission: Option<PathBuf>,
}

impl Scene {
    /// An empty scene with the hero rig sized to the frame.
    #[must_use]
    pub fn new(name: &str, frame: Frame) -> Self {
        let [cx, cy] = frame.centre();
        Self {
            name: name.into(),
            frame,
            rig: Rig::hero(frame.w.max(frame.h), rig::SWING[0]).at(cx, cy, 0.0),
            swing: rig::SWING[1..].to_vec(),
            parts: Vec::new(),
            samples: 256,
            passes: Passes::ALL,
            ao_px: 30.0,
        }
    }

    /// Adds a part.
    pub fn add(&mut self, p: Part) -> &mut Self {
        self.parts.push(p);
        self
    }

    /// Every distinct material used by the parts (by name; first definition wins).
    #[must_use]
    pub fn materials(&self) -> Vec<Mat> {
        let mut out: Vec<Mat> = Vec::new();
        for p in &self.parts {
            for m in &p.mats {
                if !out.iter().any(|o| o.name() == m.name()) {
                    out.push(m.clone());
                }
            }
        }
        out
    }

    /// The rest rig plus one rig per swing azimuth (the key, sky and rims all turn with it).
    #[must_use]
    pub fn rigs(&self) -> Vec<Rig> {
        let rest = self.rig.clone();
        let base = rest.lights.first().map_or(rig::SWING[0], |l| l.az);
        let mut v = vec![rest.clone()];
        for &az in &self.swing {
            let mut r = rest.clone();
            for l in &mut r.lights {
                l.az += az - base;
            }
            r.hdri_rot += az - base;
            v.push(r);
        }
        v
    }

    /// Height range (px) for the data passes: the dial plane minus a margin up to the top part.
    #[must_use]
    pub fn height_range(&self) -> [f32; 2] {
        let top = self.parts.iter().map(Part::top).fold(1.0, f32::max);
        [-10.0, top]
    }

    /// The parameters Blender reads.
    #[must_use]
    pub fn to_json(&self, out: &Path) -> Value {
        let [h0, h1] = self.height_range();
        json!({
            "out": out.to_string_lossy(),
            "frame": self.frame.to_json(),
            "rigs": self.rigs().iter().map(Rig::to_json).collect::<Vec<_>>(),
            "materials": self.materials().iter().map(Mat::to_json).collect::<Vec<_>>(),
            "parts": self.parts.iter().map(Part::to_json).collect::<Vec<_>>(),
            "samples": self.samples,
            "passes": {
                "beauty": self.passes.beauty, "swing": self.passes.swing, "data": self.passes.data,
                "ao": self.passes.ao, "rmh": self.passes.rmh, "emission": self.passes.emission,
            },
            "height_range": [h0 * frame::PX, h1 * frame::PX],
            "ao_distance": self.ao_px * frame::PX,
        })
    }

    /// Renders into `out` (skipped when the same scene was already rendered there), then
    /// converts the EXR data passes to PNG. GPU jobs run one at a time; a failed render is
    /// retried once (another chat may have been holding the 8 GB card).
    pub fn render(&self, paths: &Paths, out: &Path) -> Result<Rendered> {
        std::fs::create_dir_all(out).map_err(|e| io_err(out, e))?;
        let params = self.to_json(out);
        let text = serde_json::to_string(&params).unwrap_or_default();
        let fp = crate::ledger::fingerprint(&[
            bpy::HERO.as_bytes(),
            crate::recipes::PRELUDE.as_bytes(),
            text.as_bytes(),
        ]);
        let fp_path = out.join(".fp");
        let n_beauty = if self.passes.swing { 1 + self.swing.len() } else { 1 };
        let beauty: Vec<PathBuf> = (0..n_beauty).map(|i| out.join(format!("beauty_{i}.png"))).collect();
        let fresh =
            std::fs::read_to_string(&fp_path).is_ok_and(|s| s.trim() == fp) && beauty.iter().all(|p| p.is_file());
        if !fresh {
            let name = format!("hero_{}", self.name);
            if let Err(first) = crate::blender::run(paths, &name, bpy::HERO, &params) {
                eprintln!("  {name}: retrying once after: {first}");
                std::thread::sleep(std::time::Duration::from_secs(20));
                crate::blender::run(paths, &name, bpy::HERO, &params)?;
            }
            convert_passes(out)?;
            std::fs::write(&fp_path, &fp).map_err(|e| io_err(&fp_path, e))?;
        }
        let some = |n: &str| Some(out.join(n)).filter(|p| p.is_file());
        Ok(Rendered {
            dir: out.to_path_buf(),
            beauty,
            normal: some("normal.png"),
            height: some("height.png"),
            depth: some("depth.png"),
            ao: some("ao.png"),
            rmh: some("rmh.png"),
            emission: some("emission.png"),
        })
    }
}

/// Un-premultiplies an EXR emission pass (Cycles writes associated alpha).
fn straight(img: &Img) -> Img {
    Img::from_fn(img.w, img.h, 4, |x, y| {
        let a = img.get(x, y, 3);
        let k = if a > 1e-4 { 1.0 / a } else { 0.0 };
        [img.get(x, y, 0) * k, img.get(x, y, 1) * k, img.get(x, y, 2) * k, a]
    })
}

/// Turns `data.exr`, `ao.exr` and `rmh.exr` into the PNGs the kit and ComfyUI use.
fn convert_passes(out: &Path) -> Result<()> {
    let data = out.join("data.exr");
    if data.is_file() {
        let d = straight(&img::load_exr(&data)?);
        let normal = Img::from_fn(d.w, d.h, 4, |x, y| {
            let (nx, ny) = (d.get(x, y, 0) * 2.0 - 1.0, d.get(x, y, 1) * 2.0 - 1.0);
            let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
            let a = d.get(x, y, 3);
            if a < 1e-4 {
                [0.5, 0.5, 1.0, 0.0]
            } else {
                [nx * 0.5 + 0.5, ny * 0.5 + 0.5, nz * 0.5 + 0.5, a]
            }
        });
        img::save_png(&normal, &out.join("normal.png"), true, false)?;
        let height = Img::from_fn(d.w, d.h, 2, |x, y| {
            [d.get(x, y, 2).clamp(0.0, 1.0), d.get(x, y, 3), 0.0, 0.0]
        });
        img::save_png(&height, &out.join("height.png"), true, false)?;
        let depth = Img::from_fn(d.w, d.h, 3, |x, y| {
            let v = (d.get(x, y, 2).clamp(0.0, 1.0) * 0.85 + 0.15) * d.get(x, y, 3).clamp(0.0, 1.0);
            [v, v, v, 1.0]
        });
        img::save_png(&depth, &out.join("depth.png"), false, false)?;
    }
    let ao = out.join("ao.exr");
    if ao.is_file() {
        let d = straight(&img::load_exr(&ao)?);
        let g = Img::from_fn(d.w, d.h, 2, |x, y| {
            [d.get(x, y, 0).clamp(0.0, 1.0), d.get(x, y, 3), 0.0, 0.0]
        });
        img::save_png(&g, &out.join("ao.png"), true, false)?;
    }
    let rmh = out.join("rmh.exr");
    if rmh.is_file() {
        let d = straight(&img::load_exr(&rmh)?);
        img::save_png(&d, &out.join("rmh.png"), true, false)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hero::geom::BezelSpec;

    #[test]
    fn scene_json_dedupes_materials_and_turns_every_light_for_the_swing() {
        let mut s = Scene::new("t", Frame::square(0.0, 0.0, 600.0));
        let ti = mat::polished(mat::metal::TITANIUM);
        s.add(Part::bezel(
            "b",
            [0.0, 0.0],
            0.0,
            BezelSpec::ring(226.0, 18.0),
            [ti.clone(), ti.clone(), ti.clone(), ti],
        ));
        s.add(Part::face("f", [0.0, 0.0], 0.0, 226.0, mat::sunray([0.05, 0.05, 0.05])));
        let j = s.to_json(Path::new("out"));
        assert_eq!(j["materials"].as_array().map(Vec::len), Some(2));
        assert_eq!(j["rigs"].as_array().map(Vec::len), Some(3));
        let rigs = s.rigs();
        for (r, az) in rigs.iter().zip(rig::SWING) {
            assert!((r.lights[0].az - az).abs() < 1e-4);
        }
        // The rim stays opposite the key in every swing rig.
        for r in &rigs {
            assert!(((r.lights[3].az - r.lights[0].az) - 180.0).abs() < 1e-4);
        }
    }
}
