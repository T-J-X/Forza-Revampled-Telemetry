//! VisionSpatialGlass's bespoke assets (`assets/clusters/vision_spatial_glass/`, ledger
//! `assets/clusters/vision_spatial_glass/ASSETS.md`). The panes, the blade and the puck are clear
//! optical glass computed by the kit (`fh6_kit::OpticalGlass`, `docs/specs/optics.md`); these
//! assets are what that glass reflects. The Liquid Glass family's asset rule allows only
//! environment, caustic and normal maps: nothing here is drawn inside the glass.
//!
//! Order (ComfyUI first, Blender last):
//! - **Cabin plate (ComfyUI, SDXL + 4x-UltraSharp):** a 4096 × 2048 panorama of the inside of a
//!   dark modern car cabin at night, seamless left to right. A dash's glass sits in a cabin, so
//!   its rims mirror a cabin: the windscreen, the A-pillars, the thin ambient light strips. The
//!   dash belongs to the Apple Vision category, where content is anchored in the world around
//!   the car (`CATALOG.md`), and this is the world it mirrors.
//! - **Cabin studio (Blender, Cycles):** the Liquid Glass family's studio (the horizon ring, the
//!   key strip and the far strip of `docs/specs/optics.md` §7, in the same directions as the
//!   kit's analytic studio, through the family's Blender script
//!   [`super::liquid_glass::STUDIO`]) around a cyclorama that carries the cabin plate, rendered
//!   as six cube faces and resampled here into one octahedral map, cleared within 44° of the
//!   viewer so the numerals' clear zone mirrors black.
//!
//! `cargo run --release -p asset-forge -- --lane cloud build --set vision_spatial_glass` (through
//! `gpu-lock run --lane cloud`). Every step is cached in the forge scratch by fingerprint. The
//! 4K masters are copied to the Drive masters folder when it is mounted.
use super::liquid_glass::{ENV_RANGE, STUDIO};
use crate::blender;
use crate::comfy::Comfy;
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::hero::sdxl::{self, Job};
use crate::hero::texture::Models;
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::json;
use std::f32::consts::FRAC_1_SQRT_2;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/vision_spatial_glass";
/// The Drive masters folder and this dash's subfolder.
const MASTERS: &str = r"G:\My Drive\FH6 WheelDash v2 - 4K Masters";
const MASTERS_SUB: &str = "vision_spatial_glass";

/// The cabin plate.
const PLATE_PROMPT: &str = "360 degree equirectangular panorama from the driver's seat inside a dark \
    modern electric sports car cabin at night, black leather and dark suede, a sweeping windscreen \
    with distant soft city lights and a dusk sky, slim A-pillars, a few thin cool ambient light \
    strips along the doors and dashboard, glossy black trim with faint reflections, low key, \
    cinematic, photographic, wide panorama";
const PLATE_NEGATIVE: &str = "people, hands, steering wheel close up, screen, text, logo, \
    watermark, bright daylight, sun, frame, border, vignette, cartoon, illustration, blurry, \
    jpeg artifacts, distorted";
const PLATE_SEED: u64 = 102_203;
/// Plate size (2:1).
const PLATE_SIZE: (usize, usize) = (4096, 2048);
/// Radiance of the plate's white on the cyclorama: the cabin shows in the rounded edges as faint
/// second reflections under the studio's own lights (the family's value).
const PLATE_RADIANCE: f32 = 0.30;
/// Softening of the plate in linear light (Gaussian sigma in plate pixels), so the plate's thin
/// light strips read as soft reflections in a few-pixel rim and not as scratches.
const PLATE_SOFTEN_PX: f32 = 6.0;

/// Octahedral map size.
const ENV_SIZE: usize = 4096;
/// Cube face size and field of view (wider than 90° so bilinear taps never leave a face).
const FACE_SIZE: usize = 4096;
const FACE_FOV_DEG: f32 = 94.0;
/// Directions closer than `VIEWER_CLEAR.0` degrees to the viewer are black, fading in by
/// `VIEWER_CLEAR.1`: a face tilted θ mirrors the direction 2θ off the view axis, so every surface
/// tilted under 22° mirrors black, and the numerals under the front pane keep their contrast.
const VIEWER_CLEAR: (f32, f32) = (44.0, 45.0);
/// Blender seed.
const BLENDER_SEED: u64 = 102_301;
/// Cycles samples per pixel of each face.
const STUDIO_SAMPLES: u32 = 256;

/// One emissive panel of the studio in the kit's environment frame (x right, y down, z at the
/// viewer). The two panels are the family's key strip and far strip (`docs/specs/optics.md` §7),
/// the same lights as the kit's analytic studio, so a missing map changes nothing but the cabin.
struct Panel {
    name: &'static str,
    c: [f32; 3],
    tx: [f32; 3],
    ty: [f32; 3],
    half: [f32; 2],
    radiance: [f32; 3],
    soft: f32,
}

const PANELS: [Panel; 2] = [
    Panel {
        name: "key",
        c: [-0.5417, -0.5417, 0.6428],
        tx: [FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0],
        ty: [0.4545, 0.4545, 0.7660],
        half: [0.75, 0.08],
        radiance: [8.0, 8.0, 8.0],
        soft: 0.02,
    },
    Panel {
        name: "strip",
        c: [0.6409, 0.6409, 0.4226],
        tx: [-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0],
        ty: [-0.2988, -0.2988, 0.9063],
        half: [1.30, 0.06],
        radiance: [4.2, 4.7, 5.2],
        soft: 0.02,
    },
];

/// The horizon ring (optics.md §7): radiance `RING_BASE` plus a lobe towards the key and a cooler
/// lobe opposite, from the plane of the screen to `RING_ELEV_DEG` above it.
const RING_ELEV_DEG: f32 = 9.0;
const RING_BASE: f32 = 0.9;
const RING_KEY: [f32; 3] = [5.0, 5.0, 5.0];
const RING_FAR: [f32; 3] = [1.98, 2.2, 2.46];
const RING_KEY_AZ_DEG: f32 = -135.0;
const RING_FAR_AZ_DEG: f32 = 45.0;
const RING_POWER: f32 = 6.0;

/// Cube faces in Blender's frame (x right, y up the screen, z at the viewer): name, forward, up.
const FACES: [(&str, [f32; 3], [f32; 3]); 6] = [
    ("px", [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ("nx", [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ("py", [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
    ("ny", [0.0, -1.0, 0.0], [0.0, 0.0, 1.0]),
    ("pz", [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
    ("nz", [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
];

/// Builds the cabin plate (ComfyUI), the cabin studio (Blender) and the environment map, copies
/// the masters to Drive and writes the cluster's ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("vision_spatial_glass")?;
    let masters = masters_dir();
    let mut rows = Vec::new();
    let mut master_rows = Vec::new();

    // 1. ComfyUI: the cabin plate (the picture on Blender's cyclorama).
    let plate_path = cabin_plate(ctx, &scratch)?;
    let plate_kb = std::fs::metadata(&plate_path).map_or(0, |m| m.len() / 1024);
    rows.push(format!(
        "| (no runtime file: baked into `cabin_env.png`) | cabin panorama, seamless left to right | {}x{} | {plate_kb} KB (master) | comfyui:vision_spatial_glass::cabin_plate (SDXL base 1.0 + refiner when installed, 1408x704, dpmpp_2m karras 32 steps cfg 6, seam column repainted at denoise 0.55, 4x-UltraSharp) seed={PLATE_SEED} prompt=\"{PLATE_PROMPT}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the picture on the cyclorama of the Blender cabin studio: the windscreen, pillars and ambient strips that the glass rims mirror. Master `cabin_plate_master.png` |",
        PLATE_SIZE.0, PLATE_SIZE.1,
    ));
    copy_master(
        masters.as_deref(),
        &plate_path,
        "cabin_plate_master.png",
        &mut master_rows,
    );

    // 2. Blender: the cabin studio as six cube faces.
    let soft_plate = soften_plate(&plate_path, &scratch)?;
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "size": FACE_SIZE,
        "fov": FACE_FOV_DEG,
        "samples": STUDIO_SAMPLES,
        "plate": soft_plate.to_string_lossy(),
        "plate_radiance": PLATE_RADIANCE,
        "panels": PANELS.iter().map(|p| json!({
            "name": p.name, "c": p.c, "tx": p.tx, "ty": p.ty, "half": p.half,
            "radiance": p.radiance, "soft": p.soft, "grid": false,
        })).collect::<Vec<_>>(),
        "ring": json!({
            "elev_deg": RING_ELEV_DEG, "base": RING_BASE, "key": RING_KEY, "far": RING_FAR,
            "key_az": RING_KEY_AZ_DEG, "far_az": RING_FAR_AZ_DEG, "power": RING_POWER,
        }),
        "faces": FACES.iter().map(|(n, f, u)| json!([n, f, u])).collect::<Vec<_>>(),
    });
    let plate_bytes = crate::ctx::read(&soft_plate)?;
    let fp = fingerprint(&[STUDIO.as_bytes(), params.to_string().as_bytes(), &plate_bytes]);
    let fp_path = scratch.join("studio.fp");
    let faces_exist = FACES
        .iter()
        .all(|(n, _, _)| scratch.join(format!("cabin_{n}.exr")).is_file());
    if faces_exist && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp) {
        println!("  = blender cabin studio (fresh)");
    } else {
        println!("  blender cabin studio ({FACE_SIZE} px x 6 faces)");
        blender::run(&ctx.paths, "vision_spatial_glass_studio", STUDIO, &params)?;
        // The family script names its faces `studio_<face>.exr`; this dash keeps them as
        // `cabin_<face>.exr` so the two studios never mix in a shared folder.
        for (n, _, _) in FACES {
            let from = scratch.join(format!("studio_{n}.exr"));
            let to = scratch.join(format!("cabin_{n}.exr"));
            std::fs::rename(&from, &to).map_err(|e| io_err(&from, e))?;
        }
        img::write_file(&fp_path, fp.as_bytes())?;
    }

    // 3. The octahedral environment map, and its 16-bit master (the same picture).
    let env_out = ctx.abs(&format!("{DIR}/cabin_env.png"));
    let env_master = scratch.join("cabin_env_master.png");
    let env_fp = fingerprint(&[
        fp.as_bytes(),
        format!("env v1 {ENV_SIZE} {ENV_RANGE} {VIEWER_CLEAR:?}").as_bytes(),
    ]);
    let env_fp_path = scratch.join("cabin_env.fp");
    let env_fresh =
        env_out.is_file() && env_master.is_file() && std::fs::read_to_string(&env_fp_path).is_ok_and(|s| s == env_fp);
    if env_fresh {
        println!("  = environment map (fresh)");
    } else {
        println!("  environment map ({ENV_SIZE} px octahedral)");
    }
    let mut faces = Vec::with_capacity(FACES.len());
    for (name, _, _) in FACES {
        let file = format!("cabin_{name}.exr");
        let path = scratch.join(&file);
        if !env_fresh {
            faces.push(img::load_exr(&path)?.to_channels(3));
        }
        copy_master(masters.as_deref(), &path, &file, &mut master_rows);
    }
    if !env_fresh {
        let env = environment(&faces);
        img::save_png(&encode_env(&env), &env_out, false, true)?;
        img::save_png(&env, &env_master, true, false)?;
        img::write_file(&env_fp_path, env_fp.as_bytes())?;
    }
    copy_master(
        masters.as_deref(),
        &env_master,
        "cabin_env_master.png",
        &mut master_rows,
    );
    let env_kb = std::fs::metadata(&env_out).map_or(0, |m| m.len() / 1024);
    rows.push(format!(
        "| `cabin_env.png` | environment, octahedral (z at the viewer in the centre), sRGB, white = radiance {ENV_RANGE} | {s}x{s} | {env_kb} KB | blender:liquid_glass::STUDIO run by vision_spatial_glass (Cycles, 6 faces {FACE_SIZE} px, fov {FACE_FOV_DEG}°, {STUDIO_SAMPLES} samples, seed={BLENDER_SEED}): horizon ring 0..{RING_ELEV_DEG}° (radiance {RING_BASE} to 5.9 towards the key), key strip 74x9° radiance 8, strip 105x7° radiance 4.7, cyclorama with the ComfyUI cabin plate (softened by a {PLATE_SOFTEN_PX} px Gaussian) at radiance {PLATE_RADIANCE}; resampled to octahedral, cleared within {}° of the viewer | CreativeML Open RAIL++-M output (SDXL plate); MIT (asset-forge) | what every pane, the blade and the puck reflect (`OpticalGlass` environment, gain {ENV_RANGE}). Masters `cabin_env_master.png` (the same picture, 16-bit linear) and the cube faces `cabin_px.exr` .. `cabin_nz.exr` |",
        VIEWER_CLEAR.0,
        s = ENV_SIZE
    ));

    // The cluster's own ledger.
    let mut md = String::from(
        "# Vision Spatial Glass assets\n\nBuilt by `cargo run --release -p asset-forge -- --lane cloud build --set vision_spatial_glass` (recipe `tools/asset-forge/src/recipes/vision_spatial_glass.rs`). The glass is computed by the kit (`docs/specs/optics.md`); this map is what it reflects. The cluster uses the kit's analytic studio when the file is missing.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/vision_spatial_glass/`)\n\n| File | What it is |\n|---|---|\n");
    for r in master_rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (1 file + ASSETS.md)");
    Ok(())
}

/// The masters folder for this dash, when the Drive folder is mounted.
fn masters_dir() -> Option<PathBuf> {
    let root = std::env::var_os("FH6_MASTERS").map_or_else(|| PathBuf::from(MASTERS), PathBuf::from);
    if !root.is_dir() {
        println!(
            "  masters folder {} is not mounted; masters stay in the forge scratch",
            root.display()
        );
        return None;
    }
    let dir = root.join(MASTERS_SUB);
    std::fs::create_dir_all(&dir).ok().map(|()| dir)
}

/// Copies a master to Drive (when mounted) and notes it for the ledger.
fn copy_master(masters: Option<&Path>, src: &Path, name: &str, rows: &mut Vec<String>) {
    let what = match name {
        "cabin_plate_master.png" => "the ComfyUI cabin panorama on the Blender cyclorama",
        "cabin_env_master.png" => {
            "`cabin_env.png` as 16-bit linear radiance (white = radiance 16): the same octahedral picture"
        }
        _ => "one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance",
    };
    let state = match masters {
        Some(dir) => match std::fs::copy(src, dir.join(name)) {
            Ok(_) => "",
            Err(e) => {
                eprintln!("  master {name} not copied: {e}");
                " (not copied: see the build log)"
            }
        },
        None => " (not copied: Drive folder not mounted)",
    };
    rows.push(format!("| `{name}` | {what}{state} |"));
}

/// The plate as the cyclorama shows it: blurred in linear light by [`PLATE_SOFTEN_PX`], cached by
/// fingerprint beside the plate.
fn soften_plate(plate: &Path, scratch: &Path) -> Result<PathBuf> {
    let out = scratch.join("cabin_plate_soft.png");
    let src = crate::ctx::read(plate)?;
    let fp = fingerprint(&[&src, format!("soften v1 {PLATE_SOFTEN_PX}").as_bytes()]);
    let fp_path = out.with_extension("fp");
    if out.is_file() && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp) {
        return Ok(out);
    }
    let mut linear = img::load(plate)?.to_channels(3);
    for v in &mut linear.data {
        *v = img::srgb_decode(*v);
    }
    let mut soft = crate::procedural::blur(&linear, PLATE_SOFTEN_PX, false);
    for v in &mut soft.data {
        *v = img::srgb_encode(*v);
    }
    img::save_png(&soft, &out, false, true)?;
    img::write_file(&fp_path, fp.as_bytes())?;
    Ok(out)
}

/// Generates the cabin panorama: txt2img near 1 MP, roll by half the width, repaint the seam
/// column, 4x upscale, resize to [`PLATE_SIZE`], and cross-fade the outer seam.
fn cabin_plate(ctx: &Ctx, scratch: &Path) -> Result<PathBuf> {
    let out = scratch.join("cabin_plate.png");
    let mut job = Job::new(PLATE_PROMPT, PLATE_NEGATIVE, PLATE_SEED);
    job.size = (1408, 704);
    job.steps = 32;
    job.prefix = "fh6_vision_spatial_glass_plate".into();
    let graph = serde_json::to_string(&sdxl::build(&job)).unwrap_or_default();
    let fp = fingerprint(&[graph.as_bytes(), b"x-seam v1"]);
    let fp_path = out.with_extension("fp");
    if out.is_file() && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp) {
        println!("  = comfyui cabin plate (fresh)");
        return Ok(out);
    }
    println!("  comfyui cabin plate (SDXL, {}x{})", PLATE_SIZE.0, PLATE_SIZE.1);
    let c = Comfy::connect(&ctx.paths)?;
    let models = Models::probe(&c);
    job.refiner = models.refiner.clone().map(|m| (m, 0.8));
    let decode = |bytes: Vec<Vec<u8>>, what: &str| -> Result<Img> {
        let b = bytes
            .into_iter()
            .next()
            .ok_or_else(|| tool_err("comfyui", format!("{what}: no image")))?;
        let tmp = scratch.join(format!("{what}.png"));
        img::write_file(&tmp, &b)?;
        img::load(&tmp)
    };
    let base = decode(c.run_retry(&sdxl::build(&job), Duration::from_secs(900))?, "plate_base")?.to_channels(3);
    // Move the left/right seam to the middle column and repaint a band round it.
    let (w, h) = (base.w, base.h);
    let rolled = Img::from_fn(w, h, 3, |x, y| {
        let sx = (x + w / 2) % w;
        [base.get(sx, y, 0), base.get(sx, y, 1), base.get(sx, y, 2), 1.0]
    });
    let mask = Img::from_fn(w, h, 3, |x, _| {
        let dx = (x as f32 + 0.5 - w as f32 * 0.5).abs() / (w as f32 * 0.06);
        let v = 1.0 - img::smoothstep(0.6, 1.0, dx);
        [v, v, v, 1.0]
    });
    let up = c.upload_image("fh6_vsg_plate_rolled.png", &img::encode_png(&rolled, false, true)?)?;
    let mk = c.upload_image("fh6_vsg_plate_seam.png", &img::encode_png(&mask, false, false)?)?;
    let mut seam = job.clone();
    seam.init = Some((up, 0.55));
    seam.mask = Some(mk);
    seam.upscale = Some(crate::paths::UPSCALER.into());
    seam.prefix = "fh6_vision_spatial_glass_plate_seam".into();
    let big = decode(
        c.run_retry(&sdxl::build(&seam), Duration::from_secs(900))?,
        "plate_seam",
    )?
    .to_channels(3);
    c.free();
    let plate = big.resize(PLATE_SIZE.0, PLATE_SIZE.1, false);
    // A last guard on the outer seam: cross-fade the edge columns with the half-shifted copy.
    let (pw, ph) = (plate.w, plate.h);
    let band = pw as f32 * 0.02;
    let guarded = Img::from_fn(pw, ph, 3, |x, y| {
        let d = (x as f32 + 0.5).min(pw as f32 - x as f32 - 0.5);
        let k = img::smoothstep(0.0, band, d);
        let sx = (x + pw / 2) % pw;
        let mut px = [0.0_f32; 4];
        for (ch, v) in px.iter_mut().enumerate().take(3) {
            *v = plate.get(x, y, ch) * k + plate.get(sx, y, ch) * (1.0 - k);
        }
        px
    });
    img::save_png(&guarded, &out, false, true)?;
    img::write_file(&fp_path, fp.as_bytes())?;
    Ok(out)
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// The direction (kit frame, unit) of octahedral coordinates `(u, v)` in 0..1: the inverse of
/// `oct_uv` in `shape.wgsl`.
fn oct_dir(u: f32, v: f32) -> [f32; 3] {
    let (ox, oy) = (u * 2.0 - 1.0, v * 2.0 - 1.0);
    let z = 1.0 - ox.abs() - oy.abs();
    let (x, y) = if z >= 0.0 {
        (ox, oy)
    } else {
        ((1.0 - oy.abs()).copysign(ox), (1.0 - ox.abs()).copysign(oy))
    };
    let len = (x * x + y * y + z * z).sqrt().max(1e-9);
    [x / len, y / len, z / len]
}

/// The forward mapping, as the shader does it (used by the round-trip test).
#[cfg(test)]
fn oct_uv(d: [f32; 3]) -> (f32, f32) {
    let l1 = d[0].abs() + d[1].abs() + d[2].abs();
    let n = [d[0] / l1, d[1] / l1, d[2] / l1];
    let (ox, oy) = if n[2] < 0.0 {
        ((1.0 - n[1].abs()).copysign(n[0]), (1.0 - n[0].abs()).copysign(n[1]))
    } else {
        (n[0], n[1])
    };
    (ox * 0.5 + 0.5, oy * 0.5 + 0.5)
}

/// Radiance from the cube faces along a kit-frame direction (bilinear).
fn sample_cube(faces: &[Img], d: [f32; 3]) -> [f32; 3] {
    // Kit frame (y down) to Blender's (y up).
    let b = [d[0], -d[1], d[2]];
    let mut best = 0;
    let mut best_dot = f32::MIN;
    for (i, (_, f, _)) in FACES.iter().enumerate() {
        let v = dot(b, *f);
        if v > best_dot {
            best_dot = v;
            best = i;
        }
    }
    let (_, f, up) = FACES[best];
    let right = cross(f, up);
    let k = (FACE_FOV_DEG.to_radians() * 0.5).tan();
    let s = dot(b, right) / best_dot / k;
    let t = dot(b, up) / best_dot / k;
    let face = &faces[best];
    let px = (s * 0.5 + 0.5) * face.w as f32 - 0.5;
    let py = (0.5 - t * 0.5) * face.h as f32 - 0.5;
    [
        face.sample(px + 0.5, py + 0.5, 0, false),
        face.sample(px + 0.5, py + 0.5, 1, false),
        face.sample(px + 0.5, py + 0.5, 2, false),
    ]
}

/// The runtime environment map: [`environment`]'s linear values sRGB-encoded for 8 bits.
fn encode_env(env: &Img) -> Img {
    let mut out = env.clone();
    for v in &mut out.data {
        *v = img::srgb_encode(*v);
    }
    out
}

/// Resamples the cube faces into the octahedral map (2x2 supersampled), clears the cone round
/// the viewer and stores linear radiance / [`ENV_RANGE`], clamped to 0..1.
fn environment(faces: &[Img]) -> Img {
    let n = ENV_SIZE;
    let (clear0, clear1) = (VIEWER_CLEAR.0.to_radians().cos(), VIEWER_CLEAR.1.to_radians().cos());
    Img::from_fn(n, n, 3, |x, y| {
        let mut acc = [0.0_f32; 3];
        for (dx, dy) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
            let d = oct_dir((x as f32 + dx) / n as f32, (y as f32 + dy) / n as f32);
            // 1 away from the viewer, 0 inside the clear cone (cosines fall as the angle grows).
            let keep = 1.0 - img::smoothstep(clear1, clear0, d[2]);
            let c = sample_cube(faces, d);
            for ch in 0..3 {
                acc[ch] += c[ch].max(0.0) * keep * 0.25;
            }
        }
        [
            (acc[0] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[1] / ENV_RANGE).clamp(0.0, 1.0),
            (acc[2] / ENV_RANGE).clamp(0.0, 1.0),
            1.0,
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn octahedral_mapping_round_trips() {
        for d in [
            [0.0_f32, 0.0, 1.0],
            [0.6, -0.3, 0.74],
            [-0.5417, -0.5417, 0.6428],
            [0.7, 0.7, -0.14],
            [-0.2, 0.3, -0.93],
            [1.0, 0.0, 0.0],
        ] {
            let len = dot(d, d).sqrt();
            let d = [d[0] / len, d[1] / len, d[2] / len];
            let (u, v) = oct_uv(d);
            let back = oct_dir(u, v);
            let err = (0..3).map(|i| (back[i] - d[i]).abs()).fold(0.0, f32::max);
            assert!(err < 1e-4, "{d:?} -> ({u}, {v}) -> {back:?}");
        }
        assert_eq!(oct_uv([0.0, 0.0, 1.0]), (0.5, 0.5));
    }

    #[test]
    fn the_studio_lights_match_the_family_and_stay_outside_the_clear_cone() {
        // The key strip's nearest edge must lie outside the 45° fade, so a numeral under a face
        // tilted less than 22° mirrors black (optics.md §7).
        for p in &PANELS {
            let len = dot(p.c, p.c).sqrt();
            assert!((len - 1.0).abs() < 1e-3, "{} centre is a unit vector", p.name);
            assert!(
                dot(p.c, p.tx).abs() < 1e-3 && dot(p.c, p.ty).abs() < 1e-3,
                "{} frame",
                p.name
            );
            // Nearest point of the strip to the viewer axis (z): step along ty towards +z.
            let toward = if p.ty[2] > 0.0 { -1.0 } else { 1.0 };
            let e = [
                p.c[0] + p.ty[0] * p.half[1] * toward,
                p.c[1] + p.ty[1] * p.half[1] * toward,
                p.c[2] + p.ty[2] * p.half[1] * toward,
            ];
            let cos = e[2] / dot(e, e).sqrt();
            assert!(
                cos.acos().to_degrees() > VIEWER_CLEAR.1,
                "{} reaches into the clear cone",
                p.name
            );
        }
    }

    #[test]
    fn the_clear_cone_is_black_in_the_map() {
        let faces: Vec<Img> = (0..6)
            .map(|_| Img::from_fn(8, 8, 3, |_, _| [16.0, 16.0, 16.0, 1.0]))
            .collect();
        let n = ENV_SIZE;
        // The centre texel of the octahedral map looks at the viewer.
        let d = oct_dir(0.5, 0.5);
        assert!(d[2] > 0.999);
        // `environment` is 4096² and slow in debug; check the per-texel rule on its inputs.
        let (clear0, clear1) = (VIEWER_CLEAR.0.to_radians().cos(), VIEWER_CLEAR.1.to_radians().cos());
        let keep = 1.0 - img::smoothstep(clear1, clear0, d[2]);
        assert_eq!(keep, 0.0);
        assert!(sample_cube(&faces, d)[0] > 15.9);
        assert_eq!(n, 4096);
    }
}
