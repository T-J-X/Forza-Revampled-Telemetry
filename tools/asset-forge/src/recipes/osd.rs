//! The OSD's bespoke assets (`assets/osd/`, ledger `assets/osd/ASSETS.md`): the driving
//! position the Fit & camera tab shows while a camera row has focus (`docs/design/OSD.md`),
//! as rendered hardware the OSD moves live: the seat with the seat height, the wheel with its
//! offset, the dash fixed.
//!
//! Order (ComfyUI first, Blender last):
//! - **Alcantara (ComfyUI, SDXL + 4x-UltraSharp):** a seamless 4096 px macro of black alcantara
//!   nap, made neutral ([`img::neutral_luma`]: luminance only, a fixed mean and spread) before
//!   Blender reads it. It is the upholstery of the seat's quilted centre and of the wheel's rim,
//!   where it drives the colour, the nap's roughness and a fine bump.
//! - **Driving position (Blender, Cycles):** a carbon racing bucket seat (a lofted shell, a
//!   quilted alcantara centre and leather bolsters), a D-shaped wheel with carbon spokes on a
//!   22° column, and a cutaway of the dash the column runs into, in metres round the seat's
//!   H-point. Rendered orthographic from the front left at [`MASTER_SCALE`] times the OSD's
//!   scale, one layer at a time: the wheel with the dash as a holdout, so the column is cut
//!   where it enters the dash; a pass lit by the rim light alone for the seat and the wheel,
//!   which the OSD tints with the accent when their row has focus; and the whole view as the
//!   hero still.
//!
//! The layers are box-filtered to the OSD's scale and cropped to fixed rectangles, which
//! `crates/fh6-osd/src/drive.rs` mirrors (its tests check the same numbers). The build fails if
//! a layer has pixels outside its rectangle.
//!
//! `cargo run --release -p asset-forge -- --lane cloud --masters osd build --set osd` (hold the
//! lane's GPU lock). The SDXL texture is cached by its fingerprint and the renders by theirs, so
//! a rebuild with nothing changed uses no GPU. With `--masters` the Blender outputs and the
//! 4096 px texture are copied to the Drive masters folder `osd\`, with a `*_preview.png` beside
//! each for the tracker.
use crate::blender;
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::hero::texture::{self, Recipe};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::{json, Value};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Output folder (repo-relative).
const DIR: &str = "assets/osd";
/// The OSD's folder name inside the Drive masters folder.
const MASTERS_SUB: &str = "osd";
/// Blender's job name (and the masters sub-folder its outputs are copied to).
const JOB: &str = "osd_drive";

/// The OSD's scale: 4K px per metre of the driving position (0.55 px per mm of seat travel).
pub const PX_PER_M: f32 = 550.0;
/// The masters are rendered at this multiple of the OSD's scale and box-filtered down.
const MASTER_SCALE: usize = 4;
/// Cycles samples of every render (OptiX denoised).
const SAMPLES: u32 = 256;
/// Cycles seed.
const SEED: u64 = 90_211;
/// The camera, orthographic: turned this far from a side view toward the front of the car, and
/// tilted down this far, so the seat's padding and the wheel's depth show.
pub const YAW_DEG: f32 = 24.0;
pub const PITCH_DEG: f32 = 8.0;
/// The view's window on the image plane (m): left, bottom, right, top.
pub const FRAME: [f32; 4] = [-0.37, -0.11, 1.37, 0.93];

/// The seat's centre line from the cushion's front edge to the top of the shell (x forward, z
/// up, metres from the H-point), and each section's half width and bolster height.
const SEAT_PATH: [[f32; 2]; 10] = [
    [0.44, 0.06],
    [0.30, 0.0],
    [0.14, -0.035],
    [0.0, -0.05],
    [-0.10, 0.03],
    [-0.16, 0.20],
    [-0.22, 0.42],
    [-0.27, 0.62],
    [-0.30, 0.78],
    [-0.31, 0.88],
];
const SEAT_W: [f32; 10] = [0.24, 0.255, 0.26, 0.26, 0.265, 0.27, 0.275, 0.26, 0.19, 0.12];
const SEAT_H: [f32; 10] = [0.07, 0.11, 0.12, 0.14, 0.18, 0.18, 0.16, 0.13, 0.09, 0.02];
/// The shell's thickness, and the padding's inset from the shell and its thickness.
const SHELL: f32 = 0.022;
const PAD_INSET: f32 = 0.024;
const PAD: f32 = 0.05;
/// The driver's eye at seat height 0 (m). The OSD draws it, moves it with the seat height and
/// aims the FOV fan from it; the head behind it clears the backrest's padding.
pub const EYE: [f32; 3] = [-0.02, 0.0, 0.66];
/// The wheel at offset 0: hub centre (m), the column's angle above horizontal, the rim's radius,
/// its flat bottom below the hub and the rim tube's radius.
const HUB: [f32; 3] = [0.50, 0.0, 0.40];
const COLUMN_DEG: f32 = 22.0;
const RIM_R: f32 = 0.18;
const RIM_FLAT: f32 = -0.14;
const RIM_TUBE: f32 = 0.02;
/// The hub's radius, and where it runs along the column (toward the driver is positive).
const HUB_R: f32 = 0.066;
const HUB_SPAN: [f32; 2] = [-0.10, -0.026];
/// The column shroud: from and to along the column, its width and thickness, and how far it
/// sits below the column's axis (m).
const SHROUD: [f32; 5] = [-0.12, -0.38, 0.10, 0.085, -0.012];
/// The dash's profile at the cut (x, z), where the cut sits across the car (y, on the door side
/// of the column so the column enters the dash's face) and how deep the dash runs from it.
const DASH: [[f32; 2]; 16] = [
    [0.82, 0.14],
    [0.80, 0.24],
    [0.785, 0.40],
    [0.775, 0.50],
    [0.745, 0.575],
    [0.755, 0.605],
    [0.85, 0.635],
    [0.96, 0.625],
    [1.04, 0.578],
    [1.14, 0.558],
    [1.24, 0.548],
    [1.27, 0.52],
    [1.25, 0.44],
    [1.12, 0.30],
    [1.00, 0.16],
    [0.88, 0.12],
];
const DASH_CUT: f32 = -0.06;
const DASH_DEPTH: f32 = 0.45;
// The dash's cut sits on the door side of the column's shroud, so the shroud enters the dash's
// face and not its cut; the eye sits behind the wheel (checked when the crate compiles).
const _: () = {
    assert!(DASH_CUT < -SHROUD[2] * 0.5);
    assert!(EYE[0] < HUB[0] - RIM_R);
};
/// One repeat of the alcantara (m) and the quilting's diamond (m).
const NAP_TILE: f32 = 0.24;
const QUILT: f32 = 0.05;
/// Light power (W): the key softbox, the overhead sky, the cool rim strip and the warm kicker;
/// the studio world's strength (camera-invisible, for curved reflections); and how much the
/// rim light is boosted in the rim-only passes.
const LIGHTS: [f32; 4] = [80.0, 40.0, 300.0, 20.0];
const WORLD: f32 = 0.12;
const RIM_GAIN: f32 = 2.5;

/// One layer the OSD draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer {
    /// Blender's name for it (`<name>.png`, `<name>_rim.png`).
    pub name: &'static str,
    /// Its crop in OSD px from the view's top-left: x, y, width, height.
    pub rect: [usize; 4],
    /// Whether it has a rim-light-only pass.
    pub rim: bool,
    /// What it is (the ledger's "Kind").
    what: &'static str,
}

/// The layers, back to front as the OSD draws them.
pub const LAYERS: [Layer; 3] = [
    Layer {
        name: "dash",
        rect: [552, 152, 388, 338],
        rim: false,
        what: "the dash cut away at the driver's side (satin black, the cut a lighter satin), fixed",
    },
    Layer {
        name: "wheel",
        rect: [376, 194, 276, 240],
        rim: true,
        what: "the D-shaped wheel (alcantara rim, carbon spokes) and its column, cut where it enters the dash; moves with the wheel offset",
    },
    Layer {
        name: "seat",
        rect: [0, 0, 492, 572],
        rim: true,
        what: "the carbon bucket seat (quilted alcantara centre, leather bolsters); moves with the seat height",
    },
];

/// Alpha (0..1) a layer may leave outside its rectangle at the OSD's scale (render noise).
const LEAK: f32 = 1.5 / 255.0;

/// The ComfyUI upholstery. The prompt names only the fabric: no seat, no stitching, so SDXL
/// draws a flat surface and not upholstery with seams.
const NAP: Recipe = Recipe {
    name: "osd_alcantara",
    prompt: "extreme macro photograph of black alcantara suede microfibre upholstery, dense soft \
             nap with faint directional brushing and gentle light and dark patches where the nap \
             lies differently, a completely flat fabric surface filling the whole frame, top-down, \
             even soft diffuse light, seamless tileable texture, physically based material scan",
    negative: "stitching, seams, quilting, diamond pattern, piping, holes, perforation, leather \
               pores, grain, wrinkles, folds, creases, woven fabric, knit, carbon fibre, plastic, \
               glossy reflections",
    seed: 90_101,
};
/// Size of the alcantara texture (master and the one Blender reads).
const NAP_SIZE: usize = 4096;
/// One standard deviation of the neutral nap round its mid-grey mean, and the window (radius,
/// texels) it is evened out over: the nap's patches show without turning into a pattern.
const NAP_SPREAD: f32 = 0.14;
const NAP_WINDOW: usize = 64;
/// Version of the nap normalisation; it is part of the neutral file's name, so a change here
/// re-renders the scenes that read it.
const NAP_NORMALISE: &[u8] = b"osd neutral alcantara v1";

/// Builds the alcantara, renders the driving position, writes the OSD's layers and the ledger,
/// and writes the previews.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("osd")?;
    let mut rows: Vec<String> = Vec::new();
    let mut masters: Vec<(PathBuf, String)> = Vec::new();

    // 1. ComfyUI: the alcantara (the colour, roughness and fine relief of the seat's centre and
    //    the wheel's rim), made neutral so its brightness comes from the material.
    let textures = scratch.join("textures");
    let sdxl = texture::make_texture(&ctx.paths, &NAP, NAP_SIZE, &textures)?;
    let nap = neutral_nap(&sdxl, &textures)?;
    masters.push((sdxl, "alcantara (ComfyUI, seamless 4096 px)".into()));
    masters.push((
        nap.clone(),
        "alcantara made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads".into(),
    ));
    rows.push(format!(
        "| (no runtime file: rendered into the seat and the wheel) | seamless alcantara texture | {s}x{s} | master | comfyui:osd::NAP (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed={seed} prompt=\"{prompt}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the upholstery of the seat's quilted centre and the wheel's rim (one repeat per {NAP_TILE} m). Master `{file}` |",
        s = NAP_SIZE,
        seed = NAP.seed,
        prompt = squash(NAP.prompt),
        file = file_name(&nap),
    ));

    // 2. Blender: every layer, the rim passes and the hero still, in one scene.
    let render_dir = scratch.join(JOB);
    let params = params(&render_dir, &nap);
    let fp = fingerprint(&[
        DRIVE.as_bytes(),
        params.to_string().as_bytes(),
        &crate::ctx::read(&nap)?,
    ]);
    let fp_path = render_dir.join("drive.fp");
    let fresh = outputs().iter().all(|n| render_dir.join(n).is_file())
        && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp);
    let (vw, vh) = view_size();
    if fresh {
        println!("  = driving position (fresh)");
    } else {
        println!(
            "  driving position ({}x{} px, {SAMPLES} samples, {} renders)",
            vw * MASTER_SCALE,
            vh * MASTER_SCALE,
            outputs().len()
        );
        blender::run(&ctx.paths, JOB, DRIVE, &params)?;
        img::write_file(&fp_path, fp.as_bytes())?;
    }

    // 3. The OSD's layers: each pass box-filtered to the OSD's scale and cropped.
    for layer in &LAYERS {
        for (suffix, light) in passes(layer) {
            let master = render_dir.join(format!("{}{suffix}.png", layer.name));
            let small = img::box_down(&img::load(&master)?, MASTER_SCALE);
            if (small.w, small.h) != (vw, vh) {
                return Err(tool_err(
                    "osd",
                    format!(
                        "{} is {}x{} at the OSD's scale, not {vw}x{vh}",
                        file_name(&master),
                        small.w,
                        small.h
                    ),
                ));
            }
            let crop = crop_layer(&small, layer)?;
            let name = format!("drive_{}{suffix}.png", layer.name);
            let bytes = img::save_png(&crop, &out_dir.join(&name), false, true)?;
            let [x, y, w, h] = layer.rect;
            rows.push(format!(
                "| `{name}` | {what}; {light} (sRGB, straight alpha) | {w}x{h} | {kb} KB | blender:osd::DRIVE (Cycles {SAMPLES} samples, OptiX denoise, orthographic at yaw {YAW_DEG}° pitch {PITCH_DEG}°, {master_px} px per m box-filtered to {PX_PER_M}) seed={SEED} | MIT (asset-forge) | drawn by the OSD at ({x}, {y}) from the view's top-left. Master `{JOB}/{master_name}` |",
                what = layer.what,
                kb = bytes / 1024,
                master_px = PX_PER_M * MASTER_SCALE as f32,
                master_name = file_name(&master),
            ));
            masters.push((master, format!("{}, {light}", layer.name)));
        }
    }
    masters.push((
        render_dir.join("view.png"),
        "the whole driving position at seat height 0 and wheel offset 0 (the hero still)".into(),
    ));

    // 4. Previews beside the masters (the masters themselves are copied by `--masters`).
    let previews = crate::lane::write_previews(&masters, &scratch.join("previews"), MASTERS_SUB)?;

    // 5. The OSD's own ledger.
    let mut md = String::from(
        "# OSD assets\n\nBuilt by `cargo run --release -p asset-forge -- --lane cloud --masters osd build --set osd` (recipe `tools/asset-forge/src/recipes/osd.rs`). The driving-position view of the Fit & camera tab: the OSD draws these layers back to front (dash, wheel, seat), moves the seat with the seat height and the wheel with its offset, and tints a rim pass with the accent while its row has focus. With any file missing it draws the vector schematic instead.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in &rows {
        let _ = writeln!(md, "{r}");
    }
    let [ex, ey] = view_px(EYE);
    let _ = writeln!(
        md,
        "\nThe view is {vw}x{vh} px at {PX_PER_M} px per m (orthographic, yaw {YAW_DEG}°, pitch {PITCH_DEG}°). The driver's eye at seat height 0, ({}, {}, {}) m from the H-point, lands at ({ex:.2}, {ey:.2}) px; `crates/fh6-osd/src/drive.rs` mirrors these numbers.",
        EYE[0], EYE[1], EYE[2]
    );
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/osd/`)\n\nCopied by `--masters osd`: the Blender outputs into `osd_drive/` (16-bit PNG, 4x the OSD's scale), the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each for the tracker.\n\n| Master | What it is |\n|---|---|\n");
    for (path, what) in &masters {
        let _ = writeln!(md, "| `{}` | {what} |", file_name(path));
    }
    let _ = writeln!(md, "\n{previews} preview(s) written.");
    img::write_file(&ctx.abs(&format!("{DIR}/ASSETS.md")), md.as_bytes())?;
    println!("  wrote {DIR}/ ({} runtime files + ASSETS.md)", rows.len() - 1);
    Ok(())
}

/// A point (m, round the H-point) on the image plane (m): x right, y up, as the bpy camera.
#[must_use]
pub fn project(p: [f32; 3]) -> [f32; 2] {
    let (psi, phi) = (YAW_DEG.to_radians(), PITCH_DEG.to_radians());
    let r = [psi.cos(), psi.sin(), 0.0];
    let u = [-psi.sin() * phi.sin(), psi.cos() * phi.sin(), phi.cos()];
    let dot = |a: [f32; 3]| a[0] * p[0] + a[1] * p[1] + a[2] * p[2];
    [dot(r), dot(u)]
}

/// Where a point lands in the view at the OSD's scale (px from the view's top-left, y down).
#[must_use]
pub fn view_px(p: [f32; 3]) -> [f32; 2] {
    let [x, y] = project(p);
    [(x - FRAME[0]) * PX_PER_M, (FRAME[3] - y) * PX_PER_M]
}

/// The view's size at the OSD's scale (px).
#[must_use]
pub fn view_size() -> (usize, usize) {
    (
        ((FRAME[2] - FRAME[0]) * PX_PER_M).round() as usize,
        ((FRAME[3] - FRAME[1]) * PX_PER_M).round() as usize,
    )
}

/// A layer's passes: the beauty, and the rim-light-only pass when it has one.
fn passes(layer: &Layer) -> Vec<(&'static str, &'static str)> {
    let mut out = vec![("", "the full studio light")];
    if layer.rim {
        out.push(("_rim", "the rim light alone, white (the OSD tints it)"));
    }
    out
}

/// Every file the render writes.
fn outputs() -> Vec<String> {
    let mut out: Vec<String> = LAYERS
        .iter()
        .flat_map(|l| passes(l).into_iter().map(move |(s, _)| format!("{}{s}.png", l.name)))
        .collect();
    out.push("view.png".into());
    out
}

/// Blender's parameters: the geometry, camera and lights above, and the neutral nap.
fn params(out: &Path, nap: &Path) -> Value {
    json!({
        "out": out.to_string_lossy(),
        "seed": SEED,
        "samples": SAMPLES,
        "yaw": YAW_DEG,
        "pitch": PITCH_DEG,
        "frame": FRAME,
        "px_per_m": PX_PER_M * MASTER_SCALE as f32,
        "seat_path": SEAT_PATH,
        "seat_w": SEAT_W,
        "seat_h": SEAT_H,
        "shell": SHELL,
        "pad_inset": PAD_INSET,
        "pad": PAD,
        "hub": HUB,
        "column_deg": COLUMN_DEG,
        "rim_r": RIM_R,
        "rim_flat": RIM_FLAT,
        "rim_tube": RIM_TUBE,
        "hub_r": HUB_R,
        "hub_span": HUB_SPAN,
        "shroud": SHROUD,
        "dash": DASH,
        "dash_cut": DASH_CUT,
        "dash_depth": DASH_DEPTH,
        "suede": nap.to_string_lossy(),
        "suede_tile": NAP_TILE,
        "quilt": QUILT,
        "lights": LIGHTS,
        "world": WORLD,
        "rim_gain": RIM_GAIN,
        "layers": LAYERS.iter().map(|l| l.name).collect::<Vec<_>>(),
        "rim_layers": LAYERS.iter().filter(|l| l.rim).map(|l| l.name).collect::<Vec<_>>(),
        "composite": true,
        "strict_frame": true,
    })
}

/// Crops `layer`'s rectangle out of one of its passes at the OSD's scale. Fails when the pass
/// has pixels outside the rectangle (the OSD would cut them off) or nothing inside it.
fn crop_layer(small: &Img, layer: &Layer) -> Result<Img> {
    let [x0, y0, w, h] = layer.rect;
    if small.c < 4 || x0 + w > small.w || y0 + h > small.h {
        return Err(tool_err(
            "osd",
            format!(
                "layer {}: its rectangle {:?} does not fit a {}x{} RGBA pass",
                layer.name, layer.rect, small.w, small.h
            ),
        ));
    }
    let mut inside = 0.0_f32;
    for y in 0..small.h {
        for x in 0..small.w {
            let a = small.get(x, y, 3);
            if (x0..x0 + w).contains(&x) && (y0..y0 + h).contains(&y) {
                inside = inside.max(a);
            } else if a > LEAK {
                return Err(tool_err(
                    "osd",
                    format!(
                        "layer {} has pixels outside its rectangle {:?} at ({x}, {y}); grow the rectangle here and in fh6-osd's drive.rs",
                        layer.name, layer.rect
                    ),
                ));
            }
        }
    }
    if inside < 0.5 {
        return Err(tool_err("osd", format!("layer {} rendered empty", layer.name)));
    }
    Ok(small.crop(x0, y0, w, h))
}

/// A neutral copy of the SDXL alcantara in `dir` ([`img::neutral_luma`]), named by a fingerprint
/// of the source and of these settings, so the render re-runs when either changes.
fn neutral_nap(src: &Path, dir: &Path) -> Result<PathBuf> {
    let bytes = std::fs::read(src).map_err(|e| io_err(src, e))?;
    let fp = fingerprint(&[
        &bytes,
        NAP_NORMALISE,
        &NAP_SPREAD.to_le_bytes(),
        &NAP_WINDOW.to_le_bytes(),
    ]);
    let out = dir.join(format!("osd_alcantara_neutral_{}.png", &fp[..12]));
    if out.is_file() {
        return Ok(out);
    }
    let flat = img::neutral_luma(&img::load(src)?, NAP_WINDOW, NAP_SPREAD);
    img::save_png(&flat, &out, false, true)?;
    Ok(out)
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

/// The driving position (bpy). The geometry, the camera and the lights come from the
/// parameters; the materials are tuned here.
const DRIVE: &str = r#"
# The OSD's driving position (asset-forge recipe osd::DRIVE): a racing bucket seat, the steering
# wheel and its column, and a cutaway of the dash, in metres round the seat's H-point (x forward,
# y to the car's right, z up). Rendered orthographic from the front left, one layer at a time, so
# the OSD can move the seat with the seat height and the wheel with its offset.
sc = reset()
dev = gpu_cycles(sc, samples=int(ARGS['samples']), denoise=True, bounces=10)
view(sc, 'Standard')

YAW = math.radians(float(ARGS['yaw']))
PITCH = math.radians(float(ARGS['pitch']))

def v3(x, y, z):
    return Vector((x, y, z))

# ----- materials ---------------------------------------------------------------------------

def principled(name, color, rough, metal=0.0, coat=0.0, coat_rough=0.05, sheen=0.0, spec=0.5):
    m, nt = new_mat(name)
    out = node(nt, 'ShaderNodeOutputMaterial')
    b = node(nt, 'ShaderNodeBsdfPrincipled')
    b.inputs['Base Color'].default_value = (color[0], color[1], color[2], 1.0)
    b.inputs['Roughness'].default_value = rough
    b.inputs['Metallic'].default_value = metal
    for k, v in (('Coat Weight', coat), ('Coat Roughness', coat_rough), ('Sheen Weight', sheen),
                 ('Specular IOR Level', spec)):
        try:
            b.inputs[k].default_value = v
        except Exception as e:
            log('input', k, e)
    lk(nt, b.outputs[0], out.inputs['Surface'])
    return m, nt, b

def carbon(name):
    # Satin carbon composite: a 2x2 twill of 6 mm tows as a faint bump and roughness change
    # under a clear coat (at the OSD's scale the weave reads as a fine texture, not a pattern).
    m, nt, b = principled(name, (0.007, 0.007, 0.008), 0.32, coat=1.0, coat_rough=0.06)
    tc = node(nt, 'ShaderNodeTexCoord')
    ck = node(nt, 'ShaderNodeTexChecker')
    ck.inputs['Scale'].default_value = 1.0 / 0.006
    ck.inputs['Color1'].default_value = (1, 1, 1, 1)
    ck.inputs['Color2'].default_value = (0, 0, 0, 1)
    lk(nt, tc.outputs['Object'], ck.inputs['Vector'])
    rr = node(nt, 'ShaderNodeMapRange')
    lk(nt, ck.outputs['Fac'], rr.inputs['Value'])
    rr.inputs['To Min'].default_value = 0.26
    rr.inputs['To Max'].default_value = 0.40
    lk(nt, rr.outputs['Result'], b.inputs['Roughness'])
    bump = node(nt, 'ShaderNodeBump')
    bump.inputs['Strength'].default_value = 0.08
    bump.inputs['Distance'].default_value = 0.0004
    lk(nt, ck.outputs['Fac'], bump.inputs['Height'])
    lk(nt, bump.outputs['Normal'], b.inputs['Normal'])
    return m

def suede(name, color, quilt=0.0):
    # Alcantara: the neutral ComfyUI nap drives the colour (x0.45 .. x1.55 round the material's,
    # 1.0 at its mid-grey mean), the nap's roughness and a fine bump; with `quilt` (the diamond's
    # size in m) the padding rises in pillows between darker seams.
    m, nt, b = principled(name, color, 0.86, sheen=0.9, spec=0.25)
    try:
        b.inputs['Sheen Roughness'].default_value = 0.35
        b.inputs['Sheen Tint'].default_value = (0.9, 0.92, 1.0, 1.0)
    except Exception as e:
        log('sheen', e)
    uv = node(nt, 'ShaderNodeUVMap')
    uv.uv_map = 'phys'
    height = None
    tex_path = ARGS.get('suede')
    if tex_path and os.path.isfile(tex_path):
        img = bpy.data.images.load(tex_path, check_existing=True)
        img.colorspace_settings.name = 'Non-Color'
        tx = node(nt, 'ShaderNodeTexImage')
        tx.image = img
        tx.interpolation = 'Cubic'
        mp = node(nt, 'ShaderNodeMapping')
        tile = float(ARGS['suede_tile'])
        mp.inputs['Scale'].default_value = (1.0 / tile, 1.0 / tile, 1.0)
        lk(nt, uv.outputs['UV'], mp.inputs['Vector'])
        lk(nt, mp.outputs[0], tx.inputs['Vector'])
        lum = node(nt, 'ShaderNodeRGBToBW')
        lk(nt, tx.outputs['Color'], lum.inputs['Color'])
        gain = node(nt, 'ShaderNodeMapRange')
        lk(nt, lum.outputs['Val'], gain.inputs['Value'])
        gain.inputs['From Min'].default_value = 0.0
        gain.inputs['From Max'].default_value = 1.0
        gain.inputs['To Min'].default_value = 0.45
        gain.inputs['To Max'].default_value = 1.55
        mix = node(nt, 'ShaderNodeVectorMath', operation='SCALE')
        mix.inputs[0].default_value = (color[0], color[1], color[2])
        lk(nt, gain.outputs['Result'], mix.inputs['Scale'])
        lk(nt, mix.outputs[0], b.inputs['Base Color'])
        rough = node(nt, 'ShaderNodeMapRange')
        lk(nt, lum.outputs['Val'], rough.inputs['Value'])
        rough.inputs['To Min'].default_value = 0.95
        rough.inputs['To Max'].default_value = 0.70
        lk(nt, rough.outputs['Result'], b.inputs['Roughness'])
        height = lum.outputs['Val']
    else:
        log('no alcantara texture: plain suede')
    bump_in = height
    if quilt > 0.0:
        sep = node(nt, 'ShaderNodeSeparateXYZ')
        lk(nt, uv.outputs['UV'], sep.inputs[0])
        def tri(a_sign):
            # Distance to the nearest seam of one diagonal family: 0 at a seam, 0.5 mid-cell.
            s = math_node(nt, 'ADD' if a_sign > 0 else 'SUBTRACT', sep.outputs['X'], sep.outputs['Y'])
            s = math_node(nt, 'DIVIDE', s, None, vb=quilt)
            f = math_node(nt, 'FRACT', s)
            g = math_node(nt, 'SUBTRACT', None, f, va=1.0)
            return math_node(nt, 'MINIMUM', f, g)
        d = math_node(nt, 'MINIMUM', tri(1), tri(-1))
        d = math_node(nt, 'MULTIPLY', d, None, vb=2.0)
        pillow = math_node(nt, 'POWER', d, None, vb=0.8)
        if bump_in is not None:
            fine = math_node(nt, 'MULTIPLY', bump_in, None, vb=0.08)
            bump_in = math_node(nt, 'ADD', pillow, fine)
        else:
            bump_in = pillow
        # The seams sit a little darker (the stitch's shadow).
        seam = math_node(nt, 'MINIMUM', math_node(nt, 'MULTIPLY', d, None, vb=6.0), None, vb=1.0)
        dark = node(nt, 'ShaderNodeMix', data_type='RGBA')
        lk(nt, seam, dark.inputs['Factor'])
        dark.inputs[6].default_value = (color[0] * 0.35, color[1] * 0.35, color[2] * 0.35, 1.0)
        src = b.inputs['Base Color'].links[0].from_socket if b.inputs['Base Color'].is_linked else None
        if src is not None:
            lk(nt, src, dark.inputs[7])
        else:
            dark.inputs[7].default_value = (color[0], color[1], color[2], 1.0)
        lk(nt, dark.outputs[2], b.inputs['Base Color'])
    if bump_in is not None:
        bump = node(nt, 'ShaderNodeBump')
        bump.inputs['Strength'].default_value = 0.3 if quilt > 0.0 else 0.25
        bump.inputs['Distance'].default_value = 0.003 if quilt > 0.0 else 0.0006
        lk(nt, bump_in, bump.inputs['Height'])
        lk(nt, bump.outputs['Normal'], b.inputs['Normal'])
    return m

def leather(name, color):
    m, nt, b = principled(name, color, 0.42, coat=0.25, coat_rough=0.3, spec=0.45)
    tc = node(nt, 'ShaderNodeTexCoord')
    vo = node(nt, 'ShaderNodeTexVoronoi')
    vo.inputs['Scale'].default_value = 900.0
    lk(nt, tc.outputs['Object'], vo.inputs['Vector'])
    bump = node(nt, 'ShaderNodeBump')
    bump.inputs['Strength'].default_value = 0.15
    bump.inputs['Distance'].default_value = 0.0003
    lk(nt, vo.outputs['Distance'], bump.inputs['Height'])
    lk(nt, bump.outputs['Normal'], b.inputs['Normal'])
    return m

M_CARBON = carbon('osd_carbon')
M_SUEDE_Q = suede('osd_suede_quilted', (0.020, 0.020, 0.022), quilt=float(ARGS['quilt']))
M_SUEDE = suede('osd_suede', (0.018, 0.018, 0.020))
M_LEATHER = leather('osd_leather', (0.009, 0.009, 0.010))
M_DASH, _, _ = principled('osd_dash', (0.006, 0.006, 0.007), 0.6, spec=0.35)
M_SECTION, _, _ = principled('osd_section', (0.016, 0.016, 0.018), 0.38, spec=0.4)
M_TRIM, _, _ = principled('osd_trim', (0.30, 0.30, 0.32), 0.18, metal=1.0)
M_SHROUD, _, _ = principled('osd_shroud', (0.006, 0.006, 0.007), 0.4, spec=0.35)

# ----- seat --------------------------------------------------------------------------------

def path_frames(pts):
    # Tangent and inward normal (toward the occupant) at each point of an x-z polyline.
    out = []
    n = len(pts)
    for i in range(n):
        a = pts[max(i - 1, 0)]
        b = pts[min(i + 1, n - 1)]
        t = Vector((b[0] - a[0], 0.0, b[1] - a[1])).normalized()
        out.append((t, Vector((t.z, 0.0, -t.x))))
    return out

def u_profile(w, h):
    # Lateral s and inward n of one cross-section, wing tip to wing tip.
    return [(-w * 0.90, h), (-w, h * 0.55), (-w * 0.90, h * 0.14), (-w * 0.50, 0.0), (0.0, 0.0),
            (w * 0.50, 0.0), (w * 0.90, h * 0.14), (w, h * 0.55), (w * 0.90, h)]

SEAT_PATH = [tuple(p) for p in ARGS['seat_path']]
SEAT_W = [float(w) for w in ARGS['seat_w']]
SEAT_H = [float(h) for h in ARGS['seat_h']]

def loft(name, pts, widths, heights, inset, mats_for_col):
    frames = path_frames(pts)
    bm = bmesh.new()
    phys = bm.loops.layers.uv.new('phys')
    rows = []
    arc = 0.0
    for i, ((px, pz), (t, nrm)) in enumerate(zip(pts, frames)):
        if i > 0:
            arc += math.hypot(px - pts[i - 1][0], pz - pts[i - 1][1])
        row = []
        for (s, n) in u_profile(widths[i], heights[i]):
            p = Vector((px, 0.0, pz)) + nrm * (n + inset) + Vector((0.0, s, 0.0))
            row.append((bm.verts.new(p), s, arc))
        rows.append(row)
    for i in range(len(rows) - 1):
        for j in range(len(rows[i]) - 1):
            q = [rows[i][j], rows[i + 1][j], rows[i + 1][j + 1], rows[i][j + 1]]
            f = bm.faces.new([c[0] for c in q])
            f.material_index = mats_for_col(j)
            f.smooth = True
            for loop, c in zip(f.loops, q):
                loop[phys].uv = (c[1], c[2])
    bm.normal_update()
    return mesh_obj(name, bm)

def finish(ob, thickness, offset, levels, mats):
    for m in mats:
        ob.data.materials.append(m)
    so = ob.modifiers.new('solid', 'SOLIDIFY')
    so.thickness = thickness
    so.offset = offset
    so.use_even_offset = True
    so.material_offset_rim = 0
    ss = ob.modifiers.new('subd', 'SUBSURF')
    ss.levels = levels
    ss.render_levels = levels
    return ob

shell = loft('seat_shell', SEAT_PATH, SEAT_W, SEAT_H, 0.0, lambda j: 0)
finish(shell, float(ARGS['shell']), -1.0, 3, [M_CARBON])
# The padding inside the shell: quilted alcantara down the middle, smooth leather on the bolsters.
pad = loft('seat_pad', SEAT_PATH[:-1], [w - 0.03 for w in SEAT_W[:-1]], [h * 0.82 for h in SEAT_H[:-1]],
           float(ARGS['pad_inset']), lambda j: 0 if 2 <= j <= 5 else 1)
finish(pad, float(ARGS['pad']), 1.0, 3, [M_SUEDE_Q, M_LEATHER])
SEAT = [shell, pad]

# ----- steering wheel ----------------------------------------------------------------------

HUB = v3(*ARGS['hub'])
COL = math.radians(float(ARGS['column_deg']))
C_AX = v3(-math.cos(COL), 0.0, math.sin(COL))   # along the column, toward the driver
B_AX = v3(math.sin(COL), 0.0, math.cos(COL))    # the wheel's "up" in its plane
Y_AX = v3(0.0, 1.0, 0.0)
R = float(ARGS['rim_r'])
FLAT = float(ARGS['rim_flat'])
HUB_R = float(ARGS['hub_r'])
HUB_SPAN = ARGS['hub_span']
SHROUD = ARGS['shroud']   # from, to (along the column), width, thickness, drop below the axis

def wheel_pt(a, b, c=0.0):
    return HUB + Y_AX * a + B_AX * b + C_AX * c

def rim_curve():
    cd = bpy.data.curves.new('rim', 'CURVE')
    cd.dimensions = '3D'
    cd.bevel_depth = float(ARGS['rim_tube'])
    cd.bevel_resolution = 6
    cd.fill_mode = 'FULL'
    sp = cd.splines.new('POLY')
    pts = []
    for i in range(240):
        th = 2.0 * math.pi * i / 240
        a, b = R * math.cos(th), R * math.sin(th)
        if b < FLAT:
            b = FLAT - (FLAT - b) * 0.12  # a flat bottom with a soft belly
        pts.append(wheel_pt(a, b))
    sp.points.add(len(pts) - 1)
    for p, q in zip(sp.points, pts):
        p.co = (q.x, q.y, q.z, 1.0)
    sp.use_cyclic_u = True
    ob = bpy.data.objects.new('rim', cd)
    link(ob)
    ob.data.materials.append(M_SUEDE)
    return ob

def box_between(name, p0, p1, wide, thick, up, mat, bevel=0.004):
    # A bevelled bar from p0 to p1; `up` is the bar's thin direction.
    d = p1 - p0
    x = d.normalized()
    z = up.normalized()
    y = z.cross(x).normalized()
    z = x.cross(y).normalized()
    bm = bmesh.new()
    bmesh.ops.create_cube(bm, size=1.0)
    for v in bm.verts:
        v.co = Vector((v.co.x * d.length, v.co.y * wide, v.co.z * thick))
    mat4 = Matrix((x, y, z)).transposed().to_4x4()
    mat4.translation = (p0 + p1) * 0.5
    bmesh.ops.transform(bm, matrix=mat4, verts=bm.verts)
    ob = mesh_obj(name, bm)
    bv = ob.modifiers.new('bevel', 'BEVEL')
    bv.width = bevel
    bv.segments = 4
    for f in ob.data.polygons:
        f.use_smooth = True
    ob.data.materials.append(mat)
    return ob

def cylinder(name, p0, p1, r, mat, verts=48, bevel=0.003):
    d = p1 - p0
    bm = bmesh.new()
    bmesh.ops.create_cone(bm, cap_ends=True, segments=verts, radius1=r, radius2=r, depth=d.length)
    m = d.normalized().to_track_quat('Z', 'Y').to_matrix().to_4x4()
    m.translation = (p0 + p1) * 0.5
    bmesh.ops.transform(bm, matrix=m, verts=bm.verts)
    ob = mesh_obj(name, bm)
    bv = ob.modifiers.new('bevel', 'BEVEL')
    bv.width = bevel
    bv.segments = 3
    bv.limit_method = 'ANGLE'
    for f in ob.data.polygons:
        f.use_smooth = True
    ob.data.materials.append(mat)
    return ob

DISH = -0.04
rim = rim_curve()
spokes = [
    box_between('spoke_l', wheel_pt(-0.05, 0.0, DISH), wheel_pt(-R + 0.012, -0.012, 0.0), 0.062, 0.02, C_AX, M_CARBON),
    box_between('spoke_r', wheel_pt(0.05, 0.0, DISH), wheel_pt(R - 0.012, -0.012, 0.0), 0.062, 0.02, C_AX, M_CARBON),
    box_between('spoke_b', wheel_pt(0.0, -0.05, DISH), wheel_pt(0.0, FLAT + 0.008, 0.0), 0.07, 0.02, C_AX, M_CARBON),
]
hub = cylinder('hub', wheel_pt(0.0, 0.0, HUB_SPAN[0]), wheel_pt(0.0, 0.0, HUB_SPAN[1]), HUB_R, M_SHROUD)
cap = cylinder('hub_cap', wheel_pt(0.0, 0.0, HUB_SPAN[1] - 0.004), wheel_pt(0.0, 0.0, HUB_SPAN[1] + 0.006), HUB_R * 0.7, M_TRIM)
column = cylinder('column', wheel_pt(0.0, 0.0, DISH - 0.05), wheel_pt(0.0, 0.0, SHROUD[0] - 0.04), 0.028, M_TRIM)
shroud = box_between('shroud', wheel_pt(0.0, SHROUD[4], SHROUD[0]), wheel_pt(0.0, SHROUD[4], SHROUD[1]),
                     SHROUD[2], SHROUD[3], B_AX, M_SHROUD, bevel=0.035)
WHEEL = [rim, hub, cap, column, shroud] + spokes

# ----- dash (cut away at the driver's side) ------------------------------------------------

DASH = [tuple(p) for p in ARGS['dash']]
DASH_CUT = float(ARGS['dash_cut'])

def dash_obj():
    bm = bmesh.new()
    vs = [bm.verts.new((x, DASH_CUT, z)) for (x, z) in DASH]
    f = bm.faces.new(vs)
    bm.normal_update()
    res = bmesh.ops.extrude_face_region(bm, geom=[f])
    moved = [g for g in res['geom'] if isinstance(g, bmesh.types.BMVert)]
    bmesh.ops.translate(bm, vec=(0.0, float(ARGS['dash_depth']), 0.0), verts=moved)
    bm.normal_update()
    for face in bm.faces:
        face.smooth = True
        # The cut reads as a section: a lighter satin face.
        face.material_index = 1 if all(abs(v.co.y - DASH_CUT) < 1e-6 for v in face.verts) else 0
    ob = mesh_obj('dash', bm)
    bv = ob.modifiers.new('bevel', 'BEVEL')
    bv.width = 0.014
    bv.segments = 5
    bv.limit_method = 'ANGLE'
    ob.modifiers.new('wn', 'WEIGHTED_NORMAL')
    ob.data.materials.append(M_DASH)
    ob.data.materials.append(M_SECTION)
    return ob

DASHES = [dash_obj()]

# ----- camera, lights, layers --------------------------------------------------------------

def cam_axes():
    r = v3(math.cos(YAW), math.sin(YAW), 0.0)
    u = v3(-math.sin(YAW) * math.sin(PITCH), math.cos(YAW) * math.sin(PITCH), math.cos(PITCH))
    return r, u, u.cross(r)  # image right, image up, view direction (into the scene)

r_ax, u_ax, d_ax = cam_axes()
F = ARGS['frame']  # image-plane rectangle (m): left, bottom, right, top
centre = r_ax * ((F[0] + F[2]) * 0.5) + u_ax * ((F[1] + F[3]) * 0.5)
cd = bpy.data.cameras.new('cam')
cd.type = 'ORTHO'
cd.ortho_scale = F[2] - F[0]
cd.sensor_fit = 'HORIZONTAL'
cd.clip_start = 0.01
cd.clip_end = 40.0
cam = bpy.data.objects.new('cam', cd)
cam.location = centre - d_ax * 12.0
cam.rotation_euler = (math.pi * 0.5 - PITCH, 0.0, YAW)
link(cam)
sc.camera = cam
px_per_m = float(ARGS['px_per_m'])
resolution(sc, round((F[2] - F[0]) * px_per_m), round((F[3] - F[1]) * px_per_m))

WORLD = float(ARGS['world'])
studio_world(sc, strength=WORLD, visible=False)
target = (0.25, 0.0, 0.35)
P = ARGS['lights']  # key, sky, rim, kick
# Key: a big softbox high on the camera side; sky: overhead; rim: a cool strip behind the far
# side; kicker: warm and low in front.
COLORS = {'key': (1.0, 1.0, 1.0), 'sky': (1.0, 1.0, 1.0), 'rim': (0.70, 0.85, 1.0), 'kick': (1.0, 0.86, 0.72)}
LIGHTS = {
    'key': area_light('key', (1.0, -2.2, 2.0), target, 1.6, float(P[0]), color=COLORS['key']),
    'sky': area_light('sky', (0.3, 0.2, 2.8), target, 2.6, float(P[1]), color=COLORS['sky'], size_y=1.4),
    'rim': area_light('rim', (-1.2, 2.2, 1.4), target, 0.35, float(P[2]), color=COLORS['rim'], size_y=2.4),
    'kick': area_light('kick', (2.2, -1.2, 0.15), target, 0.8, float(P[3]), color=COLORS['kick']),
}
BASE = {k: v.data.energy for k, v in LIGHTS.items()}

LAYERS = {'seat': SEAT, 'wheel': WHEEL, 'dash': DASHES}
# The column runs into the dash, so the wheel's renders keep the dash as a holdout: it cuts the
# column where it enters and stays out of the picture.
HOLDOUTS = {'wheel': ['dash']}

def only(name):
    keep = HOLDOUTS.get(name, [])
    for k, obs in LAYERS.items():
        for ob in obs:
            ob.hide_render = (k != name) and (k not in keep)
            ob.is_holdout = k in keep

def lights(mode):
    # 'beauty': the whole rig. 'rim': the rim strip alone, white and boosted, for the pass the
    # OSD tints with its accent.
    for k, l in LIGHTS.items():
        if mode == 'beauty':
            l.data.energy = BASE[k]
            l.data.color = COLORS[k]
        elif k == 'rim':
            l.data.energy = BASE[k] * float(ARGS['rim_gain'])
            l.data.color = (1.0, 1.0, 1.0)
        else:
            l.data.energy = 0.0
    sc.world.node_tree.nodes['Background'].inputs['Strength'].default_value = WORLD if mode == 'beauty' else 0.0

def layer_bounds(obs):
    # Image-plane bounds (m) of the evaluated geometry: x along r_ax, y along u_ax.
    dg = bpy.context.evaluated_depsgraph_get()
    lo = [1e9, 1e9]
    hi = [-1e9, -1e9]
    for ob in obs:
        ev = ob.evaluated_get(dg)
        me = ev.to_mesh()
        for v in me.vertices:
            w = ev.matrix_world @ v.co
            x, y = w.dot(r_ax), w.dot(u_ax)
            lo = [min(lo[0], x), min(lo[1], y)]
            hi = [max(hi[0], x), max(hi[1], y)]
        ev.to_mesh_clear()
    return lo + hi

for k, obs in LAYERS.items():
    b = layer_bounds(obs)
    inside = b[0] >= F[0] and b[1] >= F[1] and b[2] <= F[2] and b[3] <= F[3]
    log('bounds', k, ' '.join('%.4f' % v for v in b), 'inside' if inside else 'OUTSIDE')
    if not inside and ARGS.get('strict_frame', False):
        raise SystemExit('layer %s leaves the frame' % k)

log('device', dev, 'res', sc.render.resolution_x, sc.render.resolution_y)
for layer in ARGS['layers']:
    only(layer)
    lights('beauty')
    save(sc, os.path.join(OUT, layer + '.png'), depth='16', mode='RGBA')
    if layer in ARGS['rim_layers']:
        lights('rim')
        save(sc, os.path.join(OUT, layer + '_rim.png'), depth='16', mode='RGBA')
if ARGS.get('composite'):
    for obs in LAYERS.values():
        for ob in obs:
            ob.hide_render = False
            ob.is_holdout = False
    lights('beauty')
    save(sc, os.path.join(OUT, 'view.png'), depth='16', mode='RGBA')
done()
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// Image-plane bounds (left, bottom, right, top) of `points`, grown by `grow`.
    fn bounds(points: &[[f32; 3]], grow: f32) -> [f32; 4] {
        points
            .iter()
            .map(|&p| project(p))
            .fold([f32::MAX, f32::MAX, f32::MIN, f32::MIN], |b, [x, y]| {
                [
                    b[0].min(x - grow),
                    b[1].min(y - grow),
                    b[2].max(x + grow),
                    b[3].max(y + grow),
                ]
            })
    }

    /// A layer's rectangle on the image plane (left, bottom, right, top).
    fn rect_m(l: &Layer) -> [f32; 4] {
        let [x, y, w, h] = l.rect.map(|v| v as f32 / PX_PER_M);
        [FRAME[0] + x, FRAME[3] - y - h, FRAME[0] + x + w, FRAME[3] - y]
    }

    fn layer(name: &str) -> Layer {
        LAYERS.iter().copied().find(|l| l.name == name).unwrap_or(LAYERS[0])
    }

    fn inside(b: [f32; 4], r: [f32; 4]) -> bool {
        b[0] >= r[0] && b[1] >= r[1] && b[2] <= r[2] && b[3] <= r[3]
    }

    /// The seat's loft control points (shell and padding), as the bpy `loft` builds them.
    fn seat_points() -> Vec<[f32; 3]> {
        let mut out = Vec::new();
        let n = SEAT_PATH.len();
        for (inset, sections, dw, kh) in [(0.0, n, 0.0, 1.0), (PAD_INSET, n - 1, 0.03, 0.82)] {
            for i in 0..sections {
                // The padding lofts along the path without its last point, as in bpy.
                let (a, b) = (SEAT_PATH[i.saturating_sub(1)], SEAT_PATH[(i + 1).min(sections - 1)]);
                let (tx, tz) = (b[0] - a[0], b[1] - a[1]);
                let len = tx.hypot(tz);
                let nrm = [tz / len, -tx / len];
                let (w, h) = (SEAT_W[i] - dw, SEAT_H[i] * kh);
                for (s, k) in [(-0.9, h), (-1.0, h * 0.55), (-0.9, h * 0.14), (-0.5, 0.0), (0.0, 0.0)] {
                    for side in [-1.0, 1.0] {
                        let off = k + inset;
                        out.push([
                            SEAT_PATH[i][0] + nrm[0] * off,
                            side * s * w,
                            SEAT_PATH[i][1] + nrm[1] * off,
                        ]);
                    }
                }
            }
        }
        out
    }

    /// The wheel's axes: along the column toward the driver, and "up" in the wheel's plane.
    fn wheel_axes() -> ([f32; 3], [f32; 3]) {
        let c = COLUMN_DEG.to_radians();
        ([-c.cos(), 0.0, c.sin()], [c.sin(), 0.0, c.cos()])
    }

    fn wheel_pt(a: f32, b: f32, c: f32) -> [f32; 3] {
        let (cx, bx) = wheel_axes();
        [
            HUB[0] + bx[0] * b + cx[0] * c,
            HUB[1] + a,
            HUB[2] + bx[2] * b + cx[2] * c,
        ]
    }

    /// The shroud's far-end corners (the part that enters the dash).
    fn shroud_end() -> Vec<[f32; 3]> {
        let [_, to, wide, thick, drop] = SHROUD;
        let mut out = Vec::new();
        for a in [-wide * 0.5, wide * 0.5] {
            for b in [drop - thick * 0.5, drop + thick * 0.5] {
                out.push(wheel_pt(a, b, to));
            }
        }
        out
    }

    /// Whether (x, z) lies inside the dash's profile (even-odd rule).
    fn in_dash(x: f32, z: f32) -> bool {
        let mut inside = false;
        for i in 0..DASH.len() {
            let (a, b) = (DASH[i], DASH[(i + 1) % DASH.len()]);
            if (a[1] > z) != (b[1] > z) && x < a[0] + (z - a[1]) / (b[1] - a[1]) * (b[0] - a[0]) {
                inside = !inside;
            }
        }
        inside
    }

    #[test]
    fn every_layer_fits_its_rectangle_and_the_rectangles_fit_the_view() {
        let (vw, vh) = view_size();
        assert_eq!((vw, vh), (957, 572), "the OSD mirrors this size");
        for l in &LAYERS {
            let [x, y, w, h] = l.rect;
            assert!(x + w <= vw && y + h <= vh && w > 0 && h > 0, "{}: {:?}", l.name, l.rect);
        }
        // The seat: its loft's control hull, grown by the shell and the padding (subdivision
        // only shrinks a surface inside its hull).
        let seat = bounds(&seat_points(), SHELL + 0.003);
        assert!(
            inside(seat, rect_m(&layer("seat"))),
            "seat {seat:?} in {:?}",
            rect_m(&layer("seat"))
        );
        // The wheel: the rim's centre line grown by the tube, the hub's end circles and the
        // shroud's corners.
        let mut wheel = Vec::new();
        for i in 0..240 {
            let th = std::f32::consts::TAU * i as f32 / 240.0;
            let (mut b, a) = (RIM_R * th.sin(), RIM_R * th.cos());
            if b < RIM_FLAT {
                b = RIM_FLAT - (RIM_FLAT - b) * 0.12;
            }
            wheel.push(wheel_pt(a, b, 0.0));
        }
        let mut wheel_b = bounds(&wheel, RIM_TUBE);
        let mut parts = Vec::new();
        for i in 0..64 {
            let th = std::f32::consts::TAU * i as f32 / 64.0;
            for c in HUB_SPAN {
                parts.push(wheel_pt(HUB_R * th.cos(), HUB_R * th.sin(), c));
            }
        }
        let [from, to, wide, thick, drop] = SHROUD;
        for c in [from, to] {
            for a in [-wide * 0.5, wide * 0.5] {
                for b in [drop - thick * 0.5, drop + thick * 0.5] {
                    parts.push(wheel_pt(a, b, c));
                }
            }
        }
        let p = bounds(&parts, 0.0);
        wheel_b = [
            wheel_b[0].min(p[0]),
            wheel_b[1].min(p[1]),
            wheel_b[2].max(p[2]),
            wheel_b[3].max(p[3]),
        ];
        assert!(
            inside(wheel_b, rect_m(&layer("wheel"))),
            "wheel {wheel_b:?} in {:?}",
            rect_m(&layer("wheel"))
        );
        // The dash: a prism of its profile between the cut and its depth.
        let prism: Vec<[f32; 3]> = DASH
            .iter()
            .flat_map(|&[x, z]| [[x, DASH_CUT, z], [x, DASH_CUT + DASH_DEPTH, z]])
            .collect();
        let dash = bounds(&prism, 0.0);
        assert!(
            inside(dash, rect_m(&layer("dash"))),
            "dash {dash:?} in {:?}",
            rect_m(&layer("dash"))
        );
    }

    #[test]
    fn the_column_runs_into_the_dash() {
        for p in shroud_end() {
            assert!(
                in_dash(p[0], p[2]),
                "the shroud's end {p:?} is inside the dash's profile"
            );
        }
        // The shroud starts in front of the dash, so a length of it shows.
        let start = wheel_pt(0.0, SHROUD[4], SHROUD[0]);
        assert!(!in_dash(start[0], start[2]));
    }

    #[test]
    fn the_eye_sits_in_the_seat_and_sees_over_the_wheel() {
        // Above the rim's top and behind the wheel.
        let (_, up) = wheel_axes();
        let rim_top = HUB[2] + up[2] * RIM_R + RIM_TUBE;
        assert!(EYE[2] > rim_top + 0.05, "eye {} over rim {rim_top}", EYE[2]);
        // In front of the backrest's padding at its height, with room for the head (about
        // 0.17 m from the eye to the back of the head).
        let i = SEAT_PATH.iter().position(|p| p[1] > EYE[2]).unwrap_or(1).max(1);
        let (a, b) = (SEAT_PATH[i - 1], SEAT_PATH[i]);
        let back_x = a[0] + (EYE[2] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
        let pad_front = back_x + (PAD_INSET + PAD) * 0.97;
        assert!(EYE[0] - pad_front > 0.17, "head room {}", EYE[0] - pad_front);
        // Inside the view, under the seat's top.
        let [x, y] = project(EYE);
        assert!(x > FRAME[0] && x < FRAME[2] && y > FRAME[1] && y < FRAME[3]);
        assert!(EYE[2] < SEAT_PATH[SEAT_PATH.len() - 1][1]);
    }

    #[test]
    fn the_numbers_the_osd_mirrors_hold() {
        // crates/fh6-osd/src/drive.rs checks the same values.
        let [ex, ey] = view_px(EYE);
        assert!(
            (ex - 193.45).abs() < 0.02 && (ey - 151.41).abs() < 0.02,
            "eye at ({ex}, {ey})"
        );
        assert_eq!(
            LAYERS.map(|l| l.rect),
            [[552, 152, 388, 338], [376, 194, 276, 240], [0, 0, 492, 572]]
        );
        assert_eq!(LAYERS.map(|l| l.rim), [false, true, true]);
        assert!((PX_PER_M - 550.0).abs() < f32::EPSILON);
        assert!((YAW_DEG - 24.0).abs() < f32::EPSILON && (PITCH_DEG - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn blender_gets_every_parameter_the_script_reads() {
        let params = params(Path::new("out"), Path::new("nap.png"));
        let mut rest = DRIVE;
        let mut seen = 0;
        while let Some(i) = rest.find("ARGS['") {
            rest = &rest[i + 6..];
            let key = rest.split('\'').next().unwrap_or_default();
            assert!(
                params.get(key).is_some(),
                "the script reads ARGS['{key}'] but the recipe does not pass it"
            );
            seen += 1;
        }
        assert!(seen > 20, "{seen} parameters read");
        assert_eq!(outputs().len(), 6);
        assert!(outputs().contains(&"wheel_rim.png".to_owned()) && !outputs().contains(&"dash_rim.png".to_owned()));
    }

    #[test]
    fn a_layer_that_leaves_its_rectangle_is_refused() {
        let (vw, vh) = view_size();
        let seat = layer("seat");
        let mut pass = Img::new(vw, vh, 4);
        for y in 10..300 {
            for x in 20..200 {
                pass.set(x, y, 3, 1.0);
            }
        }
        let crop = crop_layer(&pass, &seat).map(|c| (c.w, c.h)).ok();
        assert_eq!(crop, Some((492, 572)));
        // A pixel past the rectangle's right edge, or an empty pass, fails the build.
        pass.set(600, 40, 3, 0.5);
        assert!(crop_layer(&pass, &seat).is_err());
        assert!(crop_layer(&Img::new(vw, vh, 4), &seat).is_err());
        // Faint render noise outside is allowed.
        let mut noisy = Img::new(vw, vh, 4);
        noisy.set(100, 100, 3, 1.0);
        noisy.set(900, 500, 3, 1.0 / 255.0);
        assert!(crop_layer(&noisy, &seat).is_ok());
    }
}
