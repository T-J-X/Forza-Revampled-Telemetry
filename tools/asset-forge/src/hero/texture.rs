//! ComfyUI texture work for hero parts: the per-material prompt cookbook, seamless tiling
//! textures (txt2img → half-offset → masked seam repaint → 4x upscale) and the detail pass over
//! a Blender beauty (img2img at low denoise, ControlNet depth when the model is installed).
//!
//! Every job is cached next to its output with a `.fp` fingerprint of the graph, so reruns are
//! free and a changed prompt or seed regenerates. Jobs run one at a time and VRAM is freed
//! afterwards so Blender can have the card back.
use super::sdxl::{self, Control, Job};
use crate::comfy::Comfy;
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::paths::Paths;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A cookbook entry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recipe {
    /// File stem.
    pub name: &'static str,
    /// Positive prompt.
    pub prompt: &'static str,
    /// Negative prompt (added to [`NEGATIVE`]).
    pub negative: &'static str,
    /// Seed.
    pub seed: u64,
}

/// What every texture prompt rejects.
pub const NEGATIVE: &str = "text, watermark, logo, signature, frame, border, vignette, perspective, \
    shadow gradient, lighting falloff, object, seam, blurry, jpeg artifacts, cartoon, illustration";

/// The texture cookbook. Metals, guilloché and carbon are better procedural in Blender (exact,
/// resolution-free, anisotropic); SDXL earns its place on organic surfaces.
pub const COOKBOOK: &[Recipe] = &[
    Recipe {
        name: "leather_black_pebble",
        prompt: "macro photograph of black full-grain automotive nappa leather, fine pebble grain, \
                 subtle natural creases, soft even diffuse studio light, flat top-down, seamless \
                 tileable texture, 8k material scan",
        negative: "stitching, seams, holes, perforation, shiny plastic",
        seed: 61_207,
    },
    Recipe {
        name: "leather_cognac",
        prompt: "macro photograph of cognac brown aniline leather, natural grain, rich tan colour \
                 variation, flat top-down, seamless tileable texture, even diffuse light, material scan",
        negative: "stitching, seams, holes, shiny plastic",
        seed: 61_311,
    },
    Recipe {
        name: "alcantara_black",
        prompt: "macro photograph of black alcantara suede microfibre, soft nap, faint directional \
                 sheen, flat top-down, seamless tileable texture, even diffuse light, material scan",
        negative: "stitching, seams, holes, leather pores",
        seed: 61_419,
    },
    Recipe {
        name: "enamel_grand_feu",
        prompt: "macro photograph of deep blue grand feu enamel watch dial, glassy depth, tiny \
                 suspended bubbles, flat top-down, seamless tileable texture, even light",
        negative: "numerals, hands, indices",
        seed: 61_523,
    },
    Recipe {
        name: "carbon_forged",
        prompt: "macro photograph of forged carbon fibre composite, marbled chopped fibre flakes \
                 under clear resin, flat top-down, seamless tileable texture, even light",
        negative: "woven pattern, twill",
        seed: 61_631,
    },
];

/// Finds a cookbook entry.
#[must_use]
pub fn recipe(name: &str) -> Option<Recipe> {
    COOKBOOK.iter().copied().find(|r| r.name == name)
}

/// Optional models found on this ComfyUI (refiner, ControlNet depth / canny).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Models {
    /// SDXL refiner checkpoint, when installed.
    pub refiner: Option<String>,
    /// ControlNet depth model.
    pub depth: Option<String>,
    /// ControlNet canny model.
    pub canny: Option<String>,
}

impl Models {
    /// Probes ComfyUI's model folders. Missing models just switch features off.
    #[must_use]
    pub fn probe(c: &Comfy) -> Self {
        let ckpts = c.models("checkpoints");
        let cns = c.models("controlnet");
        let pick = |list: &[String], keys: &[&str]| {
            list.iter()
                .find(|n| {
                    let l = n.to_ascii_lowercase();
                    keys.iter().all(|k| l.contains(k))
                })
                .cloned()
        };
        Self {
            refiner: pick(&ckpts, &["refiner"]),
            depth: pick(&cns, &["depth"]).or_else(|| pick(&cns, &["union"])),
            canny: pick(&cns, &["canny"]).or_else(|| pick(&cns, &["union"])),
        }
    }
}

fn cached(out: &Path, job: &Job) -> (bool, String) {
    let graph = serde_json::to_string(&sdxl::build(job)).unwrap_or_default();
    let fp = crate::ledger::fingerprint(&[graph.as_bytes()]);
    let fp_path = out.with_extension("fp");
    let fresh = out.is_file() && std::fs::read_to_string(&fp_path).is_ok_and(|s| s.trim() == fp);
    (fresh, fp)
}

fn stamp(out: &Path, fp: &str) -> Result<()> {
    let p = out.with_extension("fp");
    std::fs::write(&p, fp).map_err(|e| io_err(&p, e))
}

fn first_image(bytes: Vec<Vec<u8>>, what: &str) -> Result<Img> {
    let b = bytes
        .into_iter()
        .next()
        .ok_or_else(|| tool_err("comfyui", format!("{what}: no image")))?;
    let tmp = std::env::temp_dir().join(format!("forge_{what}.png"));
    std::fs::write(&tmp, &b).map_err(|e| io_err(&tmp, e))?;
    let img = img::load(&tmp)?;
    let _ = std::fs::remove_file(&tmp);
    Ok(img)
}

/// Rolls an image by half its size both ways (the seams move to the centre cross).
#[must_use]
pub fn half_offset(src: &Img) -> Img {
    let (hw, hh) = (src.w / 2, src.h / 2);
    let mut out = Img::new(src.w, src.h, src.c);
    for y in 0..src.h {
        for x in 0..src.w {
            for ch in 0..src.c {
                out.set(x, y, ch, src.get((x + hw) % src.w, (y + hh) % src.h, ch));
            }
        }
    }
    out
}

/// The repaint mask for the seam pass: white in a soft band round the centre cross.
#[must_use]
pub fn seam_mask(w: usize, h: usize, band: f32) -> Img {
    Img::from_fn(w, h, 3, |x, y| {
        let dx = (x as f32 + 0.5 - w as f32 * 0.5).abs() / (w as f32 * band);
        let dy = (y as f32 + 0.5 - h as f32 * 0.5).abs() / (h as f32 * band);
        let v = 1.0 - img::smoothstep(0.6, 1.0, dx.min(dy));
        [v, v, v, 1.0]
    })
}

/// Generates the cookbook texture `r` as a seamless tile of `size` px (sRGB PNG in `dir`).
pub fn make_texture(paths: &Paths, r: &Recipe, size: usize, dir: &Path) -> Result<PathBuf> {
    let out = dir.join(format!("{}.png", r.name));
    let mut job = Job::new(r.prompt, &format!("{NEGATIVE}, {}", r.negative), r.seed);
    job.prefix = format!("hero_{}", r.name);
    let (fresh, fp) = cached(&out, &job);
    if fresh {
        return Ok(out);
    }
    let c = Comfy::connect(paths)?;
    let models = Models::probe(&c);
    job.refiner = models.refiner.clone().map(|m| (m, 0.8));
    let base = first_image(c.run_retry(&sdxl::build(&job), Duration::from_secs(900))?, r.name)?;
    // Seam pass: roll by half, repaint the centre cross, keep everything else.
    let rolled = half_offset(&base.to_channels(3));
    let up = c.upload_image(
        &format!("{}_rolled.png", r.name),
        &img::encode_png(&rolled, false, true)?,
    )?;
    let mk = c.upload_image(
        &format!("{}_seam.png", r.name),
        &img::encode_png(&seam_mask(rolled.w, rolled.h, 0.08), false, false)?,
    )?;
    let mut seam = job.clone();
    seam.init = Some((up, 0.55));
    seam.mask = Some(mk);
    seam.upscale = Some(crate::paths::UPSCALER.into());
    seam.prefix = format!("hero_{}_seam", r.name);
    let tiled = first_image(c.run_retry(&sdxl::build(&seam), Duration::from_secs(900))?, r.name)?;
    c.free();
    let mut t = tiled.to_channels(3);
    if t.w != size || t.h != size {
        t = t.resize(size, size, true);
    }
    let t = img::make_seamless(&t, 0.04);
    std::fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    img::save_png(&t, &out, false, true)?;
    stamp(&out, &fp)?;
    println!("  texture {} (seam score {:.4})", r.name, img::tile_seam_score(&t));
    Ok(out)
}

/// The detail pass: SDXL img2img over a Blender beauty (flattened on OLED black) at low
/// `denoise`, guided by the depth pass through ControlNet when a depth model is installed.
/// The beauty's alpha is kept, so the result drops into the same `ImagePaint` rect.
pub fn detail_pass(
    paths: &Paths,
    beauty: &Path,
    depth: Option<&Path>,
    prompt: &str,
    denoise: f32,
    seed: u64,
    out: &Path,
) -> Result<PathBuf> {
    let src = img::load(beauty)?;
    let flat = Img::from_fn(src.w, src.h, 3, |x, y| {
        let a = src.get(x, y, 3);
        [src.get(x, y, 0) * a, src.get(x, y, 1) * a, src.get(x, y, 2) * a, 1.0]
    });
    // SDXL wants about a megapixel; work at that size and scale back.
    let k = (1_048_576.0 / (src.w * src.h) as f32).sqrt().min(1.0);
    let (w, h) = (
        ((src.w as f32 * k) as u32 / 8 * 8).max(64),
        ((src.h as f32 * k) as u32 / 8 * 8).max(64),
    );
    let mut job = Job::new(prompt, NEGATIVE, seed);
    job.size = (w, h);
    job.steps = 30;
    job.prefix = "hero_detail".into();
    let stem = out.file_stem().and_then(|s| s.to_str()).unwrap_or("detail").to_owned();
    job.init = Some((format!("{stem}_src.png"), denoise));
    let (fresh, fp) = cached(out, &job);
    if fresh {
        return Ok(out.to_path_buf());
    }
    let c = Comfy::connect(paths)?;
    let models = Models::probe(&c);
    let name = c.upload_image(&format!("{stem}_src.png"), &img::encode_png(&flat, false, true)?)?;
    job.init = Some((name, denoise));
    if let (Some(model), Some(d)) = (models.depth.clone(), depth) {
        let dn = c.upload_image(
            &format!("{stem}_depth.png"),
            &std::fs::read(d).map_err(|e| io_err(d, e))?,
        )?;
        job.control = Some(Control {
            model,
            image: dn,
            canny: false,
            strength: 0.65,
            end: 0.85,
        });
    }
    job.refiner = models.refiner.clone().map(|m| (m, 0.8));
    let res = first_image(c.run_retry(&sdxl::build(&job), Duration::from_secs(900))?, &stem)?;
    c.free();
    let res = res.resize(src.w, src.h, false);
    let done = Img::from_fn(src.w, src.h, 4, |x, y| {
        let a = src.get(x, y, 3);
        let un = |ch: usize| {
            if a > 1e-3 {
                (res.get(x, y, ch) / a).min(1.0)
            } else {
                0.0
            }
        };
        [un(0), un(1), un(2), a]
    });
    img::save_png(&done, out, true, true)?;
    stamp(out, &fp)?;
    Ok(out.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_offset_moves_the_corner_to_the_centre() {
        let src = Img::from_fn(4, 4, 1, |x, y| {
            [if x == 0 && y == 0 { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0]
        });
        let r = half_offset(&src);
        assert_eq!(r.get(2, 2, 0), 1.0);
        assert_eq!(r.get(0, 0, 0), 0.0);
    }

    #[test]
    fn seam_mask_is_white_on_the_cross_and_black_in_the_quadrants() {
        let m = seam_mask(256, 256, 0.08);
        assert!(m.get(128, 30, 0) > 0.99);
        assert!(m.get(30, 128, 0) > 0.99);
        assert!(m.get(30, 30, 0) < 0.01);
    }

    #[test]
    fn cookbook_is_unique_and_findable() {
        for r in COOKBOOK {
            assert_eq!(recipe(r.name), Some(*r));
            assert!(COOKBOOK.iter().filter(|o| o.seed == r.seed).count() == 1);
        }
    }
}
