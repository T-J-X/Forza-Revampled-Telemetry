//! KineticLiquidMetal's bespoke assets (`assets/clusters/kinetic_liquid_metal/`, ledger
//! `assets/clusters/kinetic_liquid_metal/ASSETS.md`). The glass, the mercury and the spirit are
//! maths in the kit and the cluster (`docs/specs/optics.md`, the cluster's `physics.rs`); the
//! assets are the machined hardware the glass tube is sealed into.
//!
//! Order (ComfyUI first, Blender last):
//! - **Cap grain (ComfyUI, SDXL + 4x-UltraSharp):** a seamless 4096 px macro of a bead-blasted
//!   black anodised surface, made neutral (luminance only, a fixed mean and spread) before
//!   Blender reads it. It is the colour and relief of the caps' recessed faces, so they read as a
//!   matte blasted metal under the moving light and not as a flat black disc.
//! - **End caps (Blender, Cycles):** the two machined caps that seal the ends of the lens-front
//!   tube, seen end-on: a recessed blasted face (where the left cap carries the car's logo), a
//!   diamond-cut bright chamfer, a lathe-turned black anodised land, and a reeded outer chamfer.
//!   One sprite turned on the hero library's lathe, rendered orthographic at [`MASTER_SCALE`] px
//!   per 4K px at the hero rig's rest key, then box-filtered to 1 px per 4K px.
//!
//! `cargo run --release -p asset-forge -- --masters kinetic_liquid_metal build --set kinetic_liquid_metal`
//! (hold the GPU lock of the lane you run on). The SDXL texture and the Blender scene are cached
//! by fingerprint, so a rebuild with nothing changed uses no GPU. With `--masters` the Blender
//! outputs and the 4096 px texture are copied to the Drive masters folder; this recipe also
//! writes a `*_preview.png` beside them for the tracker. The cluster draws its procedural caps
//! when the file is missing.
use crate::ctx::Ctx;
use crate::error::{io_err, Result};
use crate::hero::frame::Frame;
use crate::hero::geom::Profile;
use crate::hero::mat::{self, metal, Mat};
use crate::hero::part::Part;
use crate::hero::texture::{self, Recipe};
use crate::hero::{Passes, Scene};
use crate::img::{self, Img};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/kinetic_liquid_metal";
/// This dash's folder name inside the Drive masters folder.
const MASTERS_SUB: &str = "kinetic_liquid_metal";

/// Render scale (image px per 4K px). The runtime file is the same picture box-filtered to 1 px
/// per 4K px, so nothing is scaled up when the cluster draws it.
const MASTER_SCALE: f32 = 4.0;
/// Cycles samples.
const SAMPLES: u32 = 384;

/// Radius of a cap (local 4K px): the cluster's `Geom::cap_r`, which its tests pin.
pub const CAP_R: f32 = 30.0;
/// Radius of the recessed blasted face (holds the logo zone, 1.3 × the cap radius square).
const FACE_R: f32 = 0.6 * CAP_R;
/// Radius where the bright chamfer meets the turned land.
const LAND_IN: f32 = 0.7 * CAP_R;
/// Radius where the reeded outer chamfer starts.
const LAND_OUT: f32 = 0.88 * CAP_R;
/// Heights (px above the cap's base): the recessed face, the land, and the bottom of the wall.
const FACE_Z: f32 = 6.0;
/// See [`FACE_Z`].
const LAND_Z: f32 = 9.0;
/// See [`FACE_Z`].
const WALL_Z: f32 = -4.0;
/// Height of the outer chamfer (a 45° cut, as wide as it is tall).
const OUTER_CHAMFER: f32 = CAP_R - LAND_OUT;
/// Fillet on the machined edges (px): catches a thin highlight.
const FILLET: f32 = 0.6;
/// Pitch of the reeding on the outer chamfer (px).
const REED_PITCH: f32 = 1.5;

// The radii run outward: face, chamfer, land, reeds.
const _: () = assert!(FACE_R >= 0.6 * CAP_R && FACE_R < LAND_IN && LAND_IN < LAND_OUT && LAND_OUT < CAP_R);

/// The blasted face.
const CAP_GRAIN: Recipe = Recipe {
    name: "kinetic_liquid_metal_cap_grain",
    prompt: "extreme macro photograph of bead blasted black anodised aluminium, a fine uniform \
             satin matte grain of tiny glass bead impacts, a completely flat metal surface filling \
             the whole frame, top-down, even soft diffuse light, seamless tileable texture, \
             physically based material scan",
    negative: "machined grooves, brushed lines, scratches, dents, dust, fingerprints, rust, paint, \
               carbon weave, fabric, leather, screws, holes, text, chrome, mirror reflections, \
               coloured tint",
    seed: 70_727,
};
/// Size of the grain texture (master and the one Blender reads).
const GRAIN_SIZE: usize = 4096;
/// One repeat of the grain on the face (local px): the bead impacts come out a fraction of a
/// pixel across at 1:1, a fine sparkle at the master scale.
const GRAIN_TILE: f32 = 256.0;
/// One standard deviation of the neutral grain round its mid-grey mean (sRGB-encoded units).
const GRAIN_SPREAD: f32 = 0.07;
/// Colour the neutral grain multiplies (linear): black anodising, a touch blue as the dye is.
const FACE_TINT: [f32; 3] = [0.018, 0.019, 0.022];
/// Relief of the grain (px): the bump distance the leather builder scales.
const GRAIN_RELIEF: f32 = 0.5;

/// Builds the grain and the cap sprite, writes the runtime file and the cluster's ledger, and
/// writes the previews.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("kinetic_liquid_metal")?;
    let mut rows: Vec<String> = Vec::new();
    let mut masters: Vec<(PathBuf, String)> = Vec::new();

    // 1. ComfyUI: the blasted grain, made neutral so the face's darkness comes from the
    //    material's tint and not from how dark SDXL drew the metal.
    let sdxl = texture::make_texture(&ctx.paths, &CAP_GRAIN, GRAIN_SIZE, &scratch.join("textures"))?;
    let grain = neutral_grain(&sdxl, &scratch.join("textures"))?;
    masters.push((sdxl.clone(), "cap grain (ComfyUI, seamless 4096 px)".into()));
    masters.push((
        grain.clone(),
        "cap grain made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads".into(),
    ));
    rows.push(format!(
        "| (no runtime file: rendered into `end_cap.png`) | seamless blasted-metal texture | {s}x{s} | master | comfyui:kinetic_liquid_metal::CAP_GRAIN (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={seed} prompt=\"{prompt}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of the caps' recessed faces (one repeat per {GRAIN_TILE} px). Master `{file}` |",
        s = GRAIN_SIZE,
        seed = CAP_GRAIN.seed,
        prompt = squash(CAP_GRAIN.prompt),
        file = file_name(&grain),
    ));

    // 2. Blender: the cap sprite at the rest key.
    let scene = cap_scene(&grain);
    let out = scene.render(&ctx.paths, &scratch.join("end_cap"))?;
    let (w, h) = scene.frame.resolution();
    let k = MASTER_SCALE.round().max(1.0) as u32;
    if let Some(beauty) = out.beauty.first() {
        let name = "end_cap.png";
        let bytes = runtime_png(beauty, &out_dir.join(name))?;
        rows.push(format!(
            "| `{name}` | machined end cap seen end-on: blasted black face, diamond-cut chamfer, turned black anodised land, reeded outer chamfer (sRGB, straight alpha) | {rw}x{rh} | {kb} KB | blender:hero (kinetic_liquid_metal::cap_scene, lathe r {CAP_R} px, Cycles {SAMPLES} samples, ortho at {MASTER_SCALE} px per 4K px, box-filtered to 1:1; hero rig at its rest key) | MIT (asset-forge) | both end caps of the lens-front tube, drawn inside a circle of the cap's radius; the left one carries the logo zone. Master `end_cap/{master}` |",
            rw = w / k,
            rh = h / k,
            kb = bytes / 1024,
            master = file_name(beauty),
        ));
        masters.push((beauty.clone(), "end cap, rest key".into()));
    }
    for (p, what) in [
        (&out.normal, "end cap, tangent-space normal (16-bit)"),
        (&out.height, "end cap, height (16-bit)"),
        (&out.ao, "end cap, ambient occlusion (16-bit)"),
    ] {
        if let Some(p) = p {
            masters.push((p.clone(), what.into()));
        }
    }

    // 3. Previews beside the masters (the masters themselves are copied by `--masters`).
    let previews = write_previews(&masters, &scratch.join("previews"))?;

    // 4. The cluster's own ledger.
    let mut md = String::from(
        "# KineticLiquidMetal assets\n\nBuilt by `cargo run --release -p asset-forge -- --masters kinetic_liquid_metal build --set kinetic_liquid_metal` (recipe `tools/asset-forge/src/recipes/kinetic_liquid_metal.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws procedural caps when the file is missing.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/kinetic_liquid_metal/`)\n\nCopied by `--masters kinetic_liquid_metal`: the Blender outputs into `hero_kinetic_liquid_metal_end_cap/`, the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each for the tracker.\n\n| Master | What it is |\n|---|---|\n");
    for (path, what) in &masters {
        let _ = writeln!(md, "| `{}` | {what} |", file_name(path));
    }
    let _ = writeln!(md, "\n{previews} preview(s) written.");
    img::write_file(&ctx.abs(&format!("{DIR}/ASSETS.md")), md.as_bytes())?;
    println!("  wrote {DIR}/ ({} runtime file + ASSETS.md)", rows.len() - 1);
    Ok(())
}

/// The cap's lathe profile, walked from the axis outward over the top and down the outer wall:
/// the recessed face (slot 0), the bright chamfer up to the land (slot 1), the turned land
/// (slot 2), the reeded outer chamfer (slot 3) and the wall (slot 2).
fn cap_profile() -> Profile {
    Profile::new()
        .sharp(0.0, FACE_Z, 0)
        .round(FACE_R, FACE_Z, 1, FILLET)
        .round(LAND_IN, LAND_Z, 2, FILLET)
        .round(LAND_OUT, LAND_Z, 3, FILLET)
        .round(CAP_R, LAND_Z - OUTER_CHAMFER, 2, FILLET)
        .sharp(CAP_R, WALL_Z, 2)
        .build()
}

/// The materials of the four slots.
fn cap_mats(grain: &Path) -> Vec<Mat> {
    vec![
        mat::leather(FACE_TINT, GRAIN_RELIEF)
            .named("klm_cap_face")
            .set("texture", grain.to_string_lossy().to_string())
            .set("tile_px", GRAIN_TILE)
            .set("tint", serde_json::json!(FACE_TINT))
            .set("sheen", 0.0)
            .rough(0.55),
        mat::polished(metal::ALUMINIUM).named("klm_cap_chamfer"),
        mat::spun(metal::BLACK_CHROME).rough(0.3).named("klm_cap_land"),
        mat::knurled(metal::BLACK_CHROME, REED_PITCH, true).named("klm_cap_reeds"),
    ]
}

/// The cap sprite: one square frame of the cap's diameter round its centre.
fn cap_scene(grain: &Path) -> Scene {
    let mut s = Scene::new(
        "kinetic_liquid_metal_end_cap",
        Frame::square(0.0, 0.0, 2.0 * CAP_R).scaled(MASTER_SCALE),
    );
    s.samples = SAMPLES;
    s.passes = Passes {
        beauty: true,
        swing: false,
        data: true,
        ao: true,
        rmh: false,
        emission: false,
    };
    s.add(Part::lathe("end_cap", [0.0, 0.0], 0.0, cap_profile(), cap_mats(grain)));
    s
}

/// Box-filters a master beauty (straight alpha, sRGB-encoded) to 1 px per 4K px and writes it
/// as an optimised 8-bit sRGB PNG. Colour is averaged in linear light weighted by coverage, so
/// edges keep their colour. Returns the file's size.
fn runtime_png(master: &Path, out: &Path) -> Result<u64> {
    let src = img::load(master)?;
    let k = MASTER_SCALE.round().max(1.0) as usize;
    let (w, h) = (src.w / k, src.h / k);
    let small = Img::from_fn(w, h, 4, |x, y| {
        let mut acc = [0.0_f32; 4];
        for dy in 0..k {
            for dx in 0..k {
                let (sx, sy) = (x * k + dx, y * k + dy);
                let a = if src.c > 3 { src.get(sx, sy, 3) } else { 1.0 };
                for (ch, v) in acc.iter_mut().enumerate().take(3) {
                    *v += img::srgb_decode(src.get(sx, sy, ch)) * a;
                }
                acc[3] += a;
            }
        }
        let a = acc[3] / (k * k) as f32;
        let inv = if acc[3] > 1e-6 { 1.0 / acc[3] } else { 0.0 };
        [
            img::srgb_encode(acc[0] * inv),
            img::srgb_encode(acc[1] * inv),
            img::srgb_encode(acc[2] * inv),
            a,
        ]
    });
    img::save_png(&small, out, false, true)
}

/// A neutral copy of the SDXL grain in `dir`: luminance only, its mean at mid grey and one
/// standard deviation at [`GRAIN_SPREAD`] (clamped at 2.5 deviations). It is named by a
/// fingerprint of the source and of the spread, so the Blender scene re-renders when either
/// changes.
fn neutral_grain(src: &Path, dir: &Path) -> Result<PathBuf> {
    let bytes = std::fs::read(src).map_err(|e| io_err(src, e))?;
    let fp = crate::ledger::fingerprint(&[&bytes, b"neutral-v1", &GRAIN_SPREAD.to_le_bytes()]);
    let out = dir.join(format!("cap_grain_neutral_{}.png", &fp[..fp.len().min(12)]));
    if out.is_file() {
        return Ok(out);
    }
    let im = img::load(src)?;
    let lum: Vec<f32> = (0..im.w * im.h)
        .map(|i| {
            let (x, y) = (i % im.w, i / im.w);
            if im.c >= 3 {
                img::luma(im.get(x, y, 0), im.get(x, y, 1), im.get(x, y, 2))
            } else {
                im.get(x, y, 0)
            }
        })
        .collect();
    let neutral = neutralise(&lum, GRAIN_SPREAD);
    let grey = Img {
        w: im.w,
        h: im.h,
        c: 1,
        data: neutral,
    };
    img::save_png(&grey.to_channels(3), &out, true, false)?;
    Ok(out)
}

/// `values` moved to a mean of 0.5 and a standard deviation of `spread`, clamped at 2.5
/// deviations either side.
fn neutralise(values: &[f32], spread: f32) -> Vec<f32> {
    let n = values.len().max(1) as f32;
    let mean = values.iter().sum::<f32>() / n;
    let var = values.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n;
    let sd = var.sqrt().max(1e-6);
    values
        .iter()
        .map(|v| 0.5 + ((v - mean) / sd).clamp(-2.5, 2.5) * spread)
        .collect()
}

/// Writes a `<job>_<stem>_preview.png` (at most 1024 px wide, flattened on black) for every
/// master into `dir`, and into the Drive masters folder when it is mounted. Returns how many were
/// written.
fn write_previews(masters: &[(PathBuf, String)], dir: &Path) -> Result<usize> {
    let drive = crate::lane::masters_dir(MASTERS_SUB).ok();
    let mut n = 0;
    for (path, _) in masters {
        let Ok(src) = img::load(path) else {
            continue;
        };
        let scale = (1024.0 / src.w.max(1) as f32).min(1.0);
        let (w, h) = (
            ((src.w as f32 * scale).round() as usize).max(1),
            ((src.h as f32 * scale).round() as usize).max(1),
        );
        let small = src.resize(w, h, false);
        let flat = Img::from_fn(w, h, 3, |x, y| {
            let a = if small.c > 3 { small.get(x, y, 3) } else { 1.0 };
            [
                small.get(x, y, 0) * a,
                small.get(x, y, 1) * a,
                small.get(x, y, 2) * a,
                1.0,
            ]
        });
        let stem = path
            .file_stem()
            .map_or_else(|| "master".into(), |s| s.to_string_lossy().to_string());
        let job = path
            .parent()
            .and_then(Path::file_name)
            .map_or_else(String::new, |s| format!("{}_", s.to_string_lossy()));
        let name = format!("{job}{stem}_preview.png");
        img::save_png(&flat, &dir.join(&name), false, true)?;
        if let Some(d) = &drive {
            if let Err(e) = std::fs::copy(dir.join(&name), d.join(&name)) {
                eprintln!("  preview {name} not copied to Drive: {e}");
            }
        }
        n += 1;
    }
    Ok(n)
}

/// A path's file name.
fn file_name(p: &Path) -> String {
    p.file_name()
        .map_or_else(String::new, |s| s.to_string_lossy().to_string())
}

/// A prompt on one line, with runs of spaces collapsed.
fn squash(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cap_is_turned_from_the_axis_to_the_cap_radius_and_fills_its_sprite() {
        let p = cap_profile();
        let (r_min, r_max, z_min, z_max) = p.extents();
        assert_eq!(r_min, 0.0, "the face closes at the axis");
        assert!((r_max - CAP_R).abs() < 1e-4, "outer radius {r_max}");
        assert!(
            z_min <= WALL_Z && (z_max - LAND_Z).abs() < 1e-3,
            "heights {z_min}..{z_max}"
        );
        let s = cap_scene(Path::new("grain.png"));
        assert!((s.frame.w - 2.0 * CAP_R).abs() < 1e-4 && (s.frame.h - 2.0 * CAP_R).abs() < 1e-4);
        assert_eq!(s.frame.centre(), [0.0, 0.0]);
        let (w, h) = s.frame.resolution();
        assert_eq!((w, h), (240, 240), "4 px per 4K px over the 60 px cap");
    }

    #[test]
    fn the_logo_zone_sits_on_the_recessed_face() {
        // The cluster's logo zone is a square 1.3 × the cap radius wide: its half diagonal
        // (0.92 R) reaches past the face, but its inscribed circle (0.65 R) is all but inside it,
        // so the logo's ink lands on the blasted face, not on the bright chamfer.
        let chamfer_rise = LAND_Z - FACE_Z;
        let chamfer_run = LAND_IN - FACE_R;
        assert!(
            (chamfer_rise / chamfer_run - 1.0).abs() < 0.01,
            "a 45° diamond-cut chamfer"
        );
    }

    #[test]
    fn the_neutral_grain_has_a_mid_grey_mean_and_the_set_spread() {
        let values: Vec<f32> = (0..4096)
            .map(|i| ((i * 37 % 101) as f32 / 101.0).powf(1.7) * 0.2)
            .collect();
        let out = neutralise(&values, GRAIN_SPREAD);
        let n = out.len() as f32;
        let mean = out.iter().sum::<f32>() / n;
        let sd = (out.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / n).sqrt();
        assert!((mean - 0.5).abs() < 0.005, "mean {mean}");
        assert!((sd - GRAIN_SPREAD).abs() < 0.005, "spread {sd}");
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)));
    }
}
