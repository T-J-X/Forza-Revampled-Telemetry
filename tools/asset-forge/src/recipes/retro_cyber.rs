//! RetroCyber's bespoke assets (`assets/clusters/retro_cyber/`, ledger
//! `assets/clusters/retro_cyber/ASSETS.md`):
//!
//! - **Housing (Blender, Cycles):** the chamfer45 VFD housing modelled in 3D: a 45° outer bevel,
//!   a flat gunmetal face with an engraved groove, a rounded inner lip and four slotted pan-head
//!   screws, in brushed gunmetal. It is rendered top-down and orthographic at 2 px per 4K px,
//!   once per view (cockpit and hood) and once per key-light azimuth (−160° and −110°, the
//!   virtual light's ±25° swing), with a transparent film. The cluster blends the two lights
//!   with lateral G.
//! - **Phosphor plate (ComfyUI, SDXL + 4x-UltraSharp):** a macro of lit VFD phosphor grain, made
//!   seamless, high-passed and mapped to an 88–100% display-level modulation (±6%). The cluster tiles
//!   it through every lit pod, chevron and column, so the phosphor glows unevenly, like the
//!   real thing, instead of as flat paint.
//!
//! `cargo run --release -p asset-forge -- build --set retro_cyber`. The SDXL render is cached in
//! the forge scratch by its fingerprint; the Blender renders are re-run on every build (≈ 20 s).
//! The files are written straight to the cluster's folder with its own ledger, so the shared
//! `assets/ASSETS.md` is not touched.
use crate::blender;
use crate::comfy::{self, Comfy};
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::{json, Map};
use std::fmt::Write as _;
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/retro_cyber";
/// Render scale: image px per 4K local px.
const SCALE: f32 = 2.0;
/// Key-light azimuths (degrees, screen convention: clockwise from 3 o'clock, y down).
const LIGHTS: [(&str, f32); 2] = [("l160", -160.0), ("l110", -110.0)];
/// The housing ring and bevel share (must match `RING` / `BEVEL_PCT` in the cluster).
const RING: f32 = 10.0;
/// See [`RING`].
const BEVEL_PCT: f32 = 0.38;
/// Margin round the housing in the render (local px).
const MARGIN: f32 = 4.0;
/// Blender seed.
const BLENDER_SEED: u64 = 1907;

/// The two views' cavities: (name, width, height, corner cut), as in the cluster's `Geom`.
const VIEWS: [(&str, f32, f32, f32); 2] = [("cockpit", 1380.0, 500.0, 58.0), ("hood", 1680.0, 336.0, 48.0)];

/// The phosphor plate.
const PLATE_PROMPT: &str = "extreme macro photograph of the lit phosphor coating of a vacuum fluorescent display segment, fine powdery granular phosphor grain, soft mottled uneven glow, flat even lighting, monochrome grey, seamless surface texture, ultra detailed";
const PLATE_NEGATIVE: &str = "text, letters, numbers, digits, watermark, logo, frame, border, vignette, gradient, shadow, people, lowres, blurry, jpeg artifacts, color";
const PLATE_SEED: u64 = 1906;
/// Shipped plate size (tiled at 80 local px in the cluster, so grains are 2–3 px).
const PLATE_SIZE: usize = 1024;
/// Display-level range of the modulation (darkest grain .. brightest).
const PLATE_RANGE: (f32, f32) = (0.88, 1.0);

/// The housing: a 3D chamfer45 frame around an octagonal cavity. Params: `w`, `h`, `cut`,
/// `ring`, `bevel_pct`, `margin`, `scale`, `lights` (list of `[name, azimuth_deg]`),
/// `samples`, `prefix`.
pub const HOUSING: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 96), denoise=True, bounces=6)
sc.render.film_transparent = True
view(sc, 'Standard')
W, H, CUT = float(ARGS['w']), float(ARGS['h']), float(ARGS['cut'])
RING, BP, MARGIN, S = float(ARGS['ring']), float(ARGS['bevel_pct']), float(ARGS['margin']), float(ARGS['scale'])
G = 2.0 - math.sqrt(2.0)

def octa(d):
    a, b, c = W * 0.5 + d, H * 0.5 + d, CUT + d * G
    return [(-a + c, b), (a - c, b), (a, b - c), (a, -b + c), (a - c, -b), (-a + c, -b), (-a, -b + c), (-a, b - c)]

FACE = 0.9
face_end = RING * (1.0 - BP)
# Profile from the cavity outwards: (offset, z). A rounded lip, the flat face with an engraved
# groove at 55% of the face, the 45° outer bevel and a short wall.
prof = [(-0.6, -2.0), (0.0, 0.2), (0.35, 0.7), (0.8, FACE),
        (face_end * 0.50, FACE), (face_end * 0.55, FACE - 0.35), (face_end * 0.60, FACE),
        (face_end, FACE), (RING, FACE - RING * BP), (RING + 0.01, -6.0)]
bm = bmesh.new()
rings = []
for (d, z) in prof:
    rings.append([bm.verts.new((x, y, z)) for (x, y) in octa(d)])
for i in range(len(rings) - 1):
    a, b = rings[i], rings[i + 1]
    for k in range(8):
        j = (k + 1) % 8
        bm.faces.new((a[k], a[j], b[j], b[k]))
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
frame = mesh_obj('housing', bm)
for f in frame.data.polygons:
    f.use_smooth = False

# Brushed gunmetal: metallic, roughness streaked along each facet by stretched noise.
m, NT = new_mat('gunmetal')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.11, 0.115, 0.125, 1.0)
bsdf.inputs['Metallic'].default_value = 1.0
tc = node(NT, 'ShaderNodeTexCoord')
mp = node(NT, 'ShaderNodeMapping')
mp.inputs['Scale'].default_value = (0.04, 3.0, 1.0)
lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
nz = node(NT, 'ShaderNodeTexNoise')
nz.inputs['Scale'].default_value = 6.0
nz.inputs['Detail'].default_value = 8.0
lk(NT, mp.outputs[0], nz.inputs['Vector'])
mr = node(NT, 'ShaderNodeMapRange')
mr.inputs['To Min'].default_value = 0.22
mr.inputs['To Max'].default_value = 0.42
lk(NT, nz.outputs['Fac'], mr.inputs['Value'])
lk(NT, mr.outputs['Result'], bsdf.inputs['Roughness'])
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
frame.data.materials.append(m)

# Slotted pan-head screws near each end of the top and bottom faces.
sm, SNT = new_mat('screw')
sb = node(SNT, 'ShaderNodeBsdfPrincipled')
sb.inputs['Base Color'].default_value = (0.16, 0.165, 0.17, 1.0)
sb.inputs['Metallic'].default_value = 1.0
sb.inputs['Roughness'].default_value = 0.25
so = node(SNT, 'ShaderNodeOutputMaterial')
lk(SNT, sb.outputs[0], so.inputs['Surface'])
dk, DNT = new_mat('slot')
db = node(DNT, 'ShaderNodeBsdfPrincipled')
db.inputs['Base Color'].default_value = (0.0, 0.0, 0.0, 1.0)
db.inputs['Roughness'].default_value = 0.9
do = node(DNT, 'ShaderNodeOutputMaterial')
lk(DNT, db.outputs[0], do.inputs['Surface'])
r = RING * 0.2
inset = CUT + RING * 1.4
dy = H * 0.5 + face_end * 0.5
for (x, y) in ((-W * 0.5 + inset, dy), (W * 0.5 - inset, dy), (-W * 0.5 + inset, -dy), (W * 0.5 - inset, -dy)):
    bpy.ops.mesh.primitive_uv_sphere_add(radius=r, segments=32, ring_count=16, location=(x, y, FACE))
    head = bpy.context.active_object
    head.scale = (1.0, 1.0, 0.35)
    bpy.ops.object.shade_smooth()
    head.data.materials.append(sm)
    bpy.ops.mesh.primitive_cube_add(size=1.0, location=(x, y, FACE + r * 0.32))
    slot = bpy.context.active_object
    slot.scale = (r * 1.5, r * 0.28, r * 0.25)
    slot.rotation_euler = (0.0, 0.0, math.radians(-30.0))
    slot.data.materials.append(dk)

studio_world(sc, strength=0.35, rotation=0.0, visible=False)
span_x = W + 2.0 * (RING + MARGIN)
span_y = H + 2.0 * (RING + MARGIN)
resolution(sc, round(span_x * S), round(span_y * S))
ortho_cam(sc, span_x, z=60.0, clip_end=200.0)
key = None
for (name, az) in ARGS['lights']:
    if key is not None:
        bpy.data.objects.remove(key, do_unlink=True)
    # Screen azimuth (y down) -> Blender (y up); the light sits 30° above the plane.
    a = math.radians(az)
    dx, dy2 = math.cos(a), -math.sin(a)
    dist = span_x
    loc = (dx * dist * math.cos(math.radians(30.0)), dy2 * dist * math.cos(math.radians(30.0)), dist * math.sin(math.radians(30.0)))
    key = area_light('key', loc, (0.0, 0.0, 0.0), dist * 0.35, 2.4e7)
    save(sc, os.path.join(OUT, '%s_%s.png' % (ARGS['prefix'], name)), depth='8', mode='RGBA')
done()
"#;

/// Builds the housing renders and the phosphor plate, and writes the cluster's ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let mut rows = Vec::new();

    // Blender: housing per view and light.
    let scratch = ctx.scratch("retro_cyber")?;
    for (view, w, h, cut) in VIEWS {
        let params = json!({
            "out": scratch.to_string_lossy(),
            "seed": BLENDER_SEED,
            "w": w, "h": h, "cut": cut,
            "ring": RING, "bevel_pct": BEVEL_PCT, "margin": MARGIN, "scale": SCALE,
            "samples": 96,
            "prefix": format!("housing_{view}"),
            "lights": LIGHTS.iter().map(|(n, a)| json!([n, a])).collect::<Vec<_>>(),
        });
        println!("  blender housing_{view}");
        blender::run(&ctx.paths, &format!("retro_cyber_housing_{view}"), HOUSING, &params)?;
        for (name, az) in LIGHTS {
            let file = format!("housing_{view}_{name}.png");
            let im = img::load(&scratch.join(&file))?.to_channels(4);
            let rel = format!("{DIR}/{file}");
            let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
            rows.push(format!(
                "| `{file}` | colour, straight alpha | {}x{} | {} KB | blender:retro_cyber::HOUSING view={view} cavity={w}x{h} cut={cut} ring={RING} light={az}° scale={SCALE} samples=96 seed={BLENDER_SEED} | MIT (asset-forge) | the housing ring; blended with the other light by lateral G |",
                im.w,
                im.h,
                bytes / 1024
            ));
        }
    }

    // ComfyUI: the phosphor plate.
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        PLATE_PROMPT.as_bytes(),
        PLATE_NEGATIVE.as_bytes(),
        &PLATE_SEED.to_le_bytes(),
    ]);
    let raw_path = scratch.join("phosphor_raw.png");
    let raw_fp_path = raw_path.with_extension("fp");
    let cached = raw_path.exists()
        && std::fs::read_to_string(&raw_fp_path)
            .map(|s| s == gen_fp)
            .unwrap_or(false);
    if !cached {
        println!("  comfyui phosphor plate (SDXL)");
        let c = Comfy::connect(&ctx.paths)?;
        let mut params = Map::new();
        params.insert("prompt".into(), json!(PLATE_PROMPT));
        params.insert("negative".into(), json!(PLATE_NEGATIVE));
        params.insert("seed".into(), json!(PLATE_SEED));
        params.insert("steps".into(), json!(32));
        params.insert("cfg".into(), json!(6.5));
        params.insert("width".into(), json!(1024));
        params.insert("height".into(), json!(1024));
        params.insert("prefix".into(), json!("fh6_retro_cyber_phosphor"));
        let wf = comfy::patch(&workflow, &params)?;
        let images = c.run(&wf, Duration::from_secs(900))?;
        let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
        img::write_file(&raw_path, bytes)?;
        img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
    }
    let raw = img::load(&raw_path)?.to_channels(3);
    let plate = phosphor_plate(&raw);
    let rel = format!("{DIR}/phosphor_grain.png");
    let bytes = img::save_png(&plate, &ctx.abs(&rel), false, true)?;
    rows.push(format!(
        "| `phosphor_grain.png` | grey modulation, tileable | {s}x{s} | {} KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={PLATE_SEED} prompt=\"{PLATE_PROMPT}\" → grey, seamless, high-pass, {}–{} display level, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | tinted fill of every lit pod, chevron and column |",
        bytes / 1024,
        PLATE_RANGE.0,
        PLATE_RANGE.1,
        img::tile_seam_score(&plate),
        s = PLATE_SIZE
    ));

    // The cluster's own ledger.
    let mut md = String::from(
        "# RetroCyber assets\n\nBuilt by `cargo run --release -p asset-forge -- build --set retro_cyber` (recipe `tools/asset-forge/src/recipes/retro_cyber.rs`). The cluster draws a procedural fallback for each when the file is missing.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (5 files + ASSETS.md)");
    Ok(())
}

/// The phosphor modulation from the SDXL macro: luma, seamless, high-passed (the 1/32-scale
/// blur removes lighting gradients), then the 2nd..98th percentile mapped to [`PLATE_RANGE`].
fn phosphor_plate(raw: &Img) -> Img {
    let n = PLATE_SIZE;
    let small = raw.resize(n, n, false);
    let grey = Img::from_fn(n, n, 1, |x, y| {
        let v = img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2));
        [v, 0.0, 0.0, 0.0]
    });
    let tile = img::make_seamless(&grey, 0.2);
    let low = tile.resize(n / 32, n / 32, true).resize(n, n, true);
    let mut hp: Vec<f32> = (0..n * n).map(|i| tile.data[i] - low.data[i]).collect();
    let mut sorted = hp.clone();
    sorted.sort_by(f32::total_cmp);
    let lo = sorted[(sorted.len() as f32 * 0.02) as usize];
    let hi = sorted[(sorted.len() as f32 * 0.98) as usize];
    let span = (hi - lo).max(1e-4);
    for v in &mut hp {
        let k = ((*v - lo) / span).clamp(0.0, 1.0);
        *v = PLATE_RANGE.0 + (PLATE_RANGE.1 - PLATE_RANGE.0) * k;
    }
    Img::from_fn(n, n, 3, |x, y| {
        let v = hp[y * n + x];
        [v, v, v, 0.0]
    })
}
