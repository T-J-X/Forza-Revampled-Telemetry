//! RetroCyber's bespoke assets (`assets/clusters/retro_cyber/`, ledger
//! `assets/clusters/retro_cyber/ASSETS.md`), for the Toyota Supra A70 Panoramic Digital Meter:
//!
//! - **Surround grain (ComfyUI, SDXL + 4x-UltraSharp):** a macro of bead-blasted anodised
//!   graphite, made seamless and high-passed. It is a build input only: Blender reads it as the
//!   surround's roughness and micro bump, so the satin face breaks the key light up like real
//!   blasted metal instead of a flat gradient.
//! - **Surround (Blender, Cycles):** the slim surround round the lens, modelled in 3D from the
//!   cluster's `Geom` (the lens's stepped outline, a low strip with a raised part joined by the
//!   kit's smooth union, and the surround width): a polished inner
//!   chamfer where the lens sits, a flat satin-graphite face and a rounded outer edge. It is
//!   rendered top-down and orthographic at 2 px per 4K px, once per view (cockpit and hood) and
//!   once per key-light azimuth (−160° and −110°, the virtual light's ±25° swing), with a
//!   transparent film. The cluster blends the two lights with lateral G.
//! - **Phosphor plate (ComfyUI, SDXL + 4x-UltraSharp):** a macro of lit VFD phosphor grain, made
//!   seamless, high-passed and mapped to an 88–100% display-level modulation (±6%). The cluster
//!   tiles it through every lit bar, pod and segment, so the phosphor glows unevenly, like the
//!   real thing, instead of as flat paint.
//!
//! `cargo run --release -p asset-forge -- build --set retro_cyber` (cloud: `--lane cloud` first).
//! The SDXL renders are cached in the forge scratch by their fingerprints; the Blender renders are
//! re-run on every build. The files are written straight to the cluster's folder with its own
//! ledger, so the shared `assets/ASSETS.md` is not touched.
use crate::blender;
use crate::comfy::{self, Comfy};
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use serde_json::{json, Map};
use std::fmt::Write as _;
use std::path::Path;
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/retro_cyber";
/// Render scale: image px per 4K local px.
const SCALE: f32 = 2.0;
/// Key-light azimuths (degrees, screen convention: clockwise from 3 o'clock, y down).
const LIGHTS: [(&str, f32); 2] = [("l160", -160.0), ("l110", -110.0)];
/// Margin round the surround in the render (local px; the cluster's `SURROUND_MARGIN`).
const MARGIN: f32 = 4.0;
/// Blender seed.
const BLENDER_SEED: u64 = 1907;

/// One view's lens outline, as in the cluster's `Geom` (`lens`, `lens_r`, `surround`,
/// `strip_top`, `hump`), in 4K local px.
struct ViewShape {
    name: &'static str,
    /// The lens's bounds: width and height.
    size: (f32, f32),
    /// Corner radius.
    r: f32,
    /// Surround width.
    ring: f32,
    /// The low strip's top edge, measured down from the lens's top.
    strip_drop: f32,
    /// The raised part's left and right edges, measured from the lens's centre.
    hump: (f32, f32),
}

const VIEWS: [ViewShape; 2] = [
    ViewShape {
        name: "cockpit",
        size: (1400.0, 514.0),
        r: 36.0,
        ring: 12.0,
        strip_drop: 132.0,
        hump: (-550.0, 340.0),
    },
    ViewShape {
        name: "hood",
        size: (1732.0, 358.0),
        r: 30.0,
        ring: 10.0,
        strip_drop: 68.0,
        hump: (-722.0, 442.0),
    },
];
/// Blend length of the outline's smooth union (the cluster's `LENS_BLEND`).
const LENS_BLEND: f32 = 24.0;
/// How far the raised part reaches into the strip (the cluster's `HUMP_OVERLAP`).
const HUMP_OVERLAP: f32 = 80.0;
/// Points on each ring of the surround's outline.
const OUTLINE_POINTS: u32 = 1440;

/// The phosphor plate.
const PLATE_PROMPT: &str = "extreme macro photograph of the lit phosphor coating of a vacuum fluorescent display segment, fine powdery granular phosphor grain, soft mottled uneven glow, flat even lighting, monochrome grey, seamless surface texture, ultra detailed";
const PLATE_NEGATIVE: &str = "text, letters, numbers, digits, watermark, logo, frame, border, vignette, gradient, shadow, people, lowres, blurry, jpeg artifacts, color";
const PLATE_SEED: u64 = 1906;
/// Shipped plate size (tiled at 80 local px in the cluster, so grains are 2–3 px).
const PLATE_SIZE: usize = 1024;
/// Display-level range of the modulation (darkest grain .. brightest).
const PLATE_RANGE: (f32, f32) = (0.88, 1.0);

/// The surround grain: materials only, never an object (SDXL draws whatever the prompt names).
const GRAIN_PROMPT: &str = "extreme macro photograph of bead blasted anodised graphite aluminium, fine uniform satin grain, tiny sparkling pits, flat even lighting, monochrome grey, seamless surface texture, ultra detailed";
const GRAIN_NEGATIVE: &str = "text, letters, numbers, watermark, logo, frame, border, vignette, gradient, shadow, scratches, objects, people, lowres, blurry, jpeg artifacts, color";
const GRAIN_SEED: u64 = 2708;
/// Grain texture size (a build input; Blender tiles it about every 40 local px).
const GRAIN_SIZE: usize = 1024;

/// The surround: a frame round the lens's stepped outline. Params: `w`, `h`, `r` (the lens's
/// bounds and corner radius), `strip_drop`, `hump` (`[x0, x1]`), `blend`, `overlap`, `points`
/// (the outline: the kit's smooth union of two rounded rectangles, `fh6_kit::shape`), `ring`,
/// `margin`, `scale`, `lights` (list of `[name, azimuth_deg]`), `samples`, `prefix`, `grain`
/// (path of the ComfyUI grain, or empty for procedural noise).
pub const HOUSING: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 96), denoise=True, bounces=6)
sc.render.film_transparent = True
view(sc, 'Standard')
W, H, R = float(ARGS['w']), float(ARGS['h']), float(ARGS['r'])
RING, MARGIN, S = float(ARGS['ring']), float(ARGS['margin']), float(ARGS['scale'])
DROP, K, OVERLAP = float(ARGS['strip_drop']), float(ARGS['blend']), float(ARGS['overlap'])
HX0, HX1 = float(ARGS['hump'][0]), float(ARGS['hump'][1])
POINTS = int(ARGS['points'])
# Screen coordinates (y down) round the lens's centre: the low strip and the raised part.
STRIP = (-W * 0.5, -H * 0.5 + DROP, W * 0.5, H * 0.5)
HUMP = (HX0, -H * 0.5, HX1, -H * 0.5 + DROP + OVERLAP)

def sd_rrect(x, y, b):
    hx, hy = (b[2] - b[0]) * 0.5, (b[3] - b[1]) * 0.5
    r = min(R, hx, hy)
    qx = abs(x - (b[0] + b[2]) * 0.5) - hx + r
    qy = abs(y - (b[1] + b[3]) * 0.5) - hy + r
    return math.hypot(max(qx, 0.0), max(qy, 0.0)) + min(max(qx, qy), 0.0) - r

def sdf(x, y):
    a, b = sd_rrect(x, y, STRIP), sd_rrect(x, y, HUMP)
    return min(a, b) - K * math.log1p(math.exp(-abs(a - b) / K))

# The outline at signed distance d: both pieces are convex and hold the origin O, so every ray
# from O crosses it once. The rays aim at points spaced evenly along the sharp-cornered outline,
# so corners and the far ends of the strip get as many points as the long edges. Bisection along
# each ray; y is flipped for Blender (y up).
OX, OY = (HX0 + HX1) * 0.5, (STRIP[1] + min(HUMP[3], STRIP[3])) * 0.5
CORNERS = [(STRIP[0], STRIP[3]), (STRIP[2], STRIP[3]), (STRIP[2], STRIP[1]), (HX1, STRIP[1]), (HX1, HUMP[1]),
           (HX0, HUMP[1]), (HX0, STRIP[1]), (STRIP[0], STRIP[1])]

def ray_angles():
    edges = [(CORNERS[i], CORNERS[(i + 1) % len(CORNERS)]) for i in range(len(CORNERS))]
    step = sum(math.dist(a, b) for (a, b) in edges) / POINTS
    out = []
    for (a, b) in edges:
        n = max(1, round(math.dist(a, b) / step))
        for j in range(n):
            x, y = a[0] + (b[0] - a[0]) * j / n, a[1] + (b[1] - a[1]) * j / n
            out.append(math.atan2(-(y - OY), x - OX))
    return sorted(out)

def trace(angles, d):
    pts = []
    reach = W + H
    for t in angles:
        dx, dy = math.cos(t), -math.sin(t)
        lo, hi = 0.0, reach
        for _ in range(40):
            mid = (lo + hi) * 0.5
            if sdf(OX + dx * mid, OY + dy * mid) < d:
                lo = mid
            else:
                hi = mid
        m = (lo + hi) * 0.5
        pts.append((OX + dx * m, OY + dy * m))
    return pts

def even_angles(angles, d):
    # Rays aimed at points spaced evenly along the outline at distance d (a ray that meets an
    # edge at a shallow angle lands far along it once the outline is offset).
    pts = trace(angles, d)
    n = len(pts)
    step = sum(math.dist(pts[i], pts[(i + 1) % n]) for i in range(n)) / POINTS
    out = []
    for i in range(n):
        a, b = pts[i], pts[(i + 1) % n]
        parts = max(1, math.ceil(math.dist(a, b) / step))
        for j in range(parts):
            x, y = a[0] + (b[0] - a[0]) * j / parts, a[1] + (b[1] - a[1]) * j / parts
            out.append(math.atan2(-(y - OY), x - OX))
    return out

BASE = ray_angles()
ALL = sorted(BASE + even_angles(BASE, RING) + even_angles(BASE, -1.0))
ANGLES, last = [], None
for (t, q) in zip(ALL, trace(ALL, 0.0)):
    # Points closer than this on the lens outline would make sliver faces.
    if last is None or math.dist(q, last) > 0.8:
        ANGLES.append(t)
        last = q

def rrect(d):
    return [(x, -y) for (x, y) in trace(ANGLES, d)]

# Profile from the lens edge outwards: (offset, z, polished). A polished chamfer rolls down to
# the lens, the satin face is flat, the outer edge is rounded over into a short wall.
prof = [(-1.0, -3.0, True), (0.0, 0.5, True), (0.5, 1.3, True), (1.4, 1.85, True), (2.4, 2.0, False),
        (RING - 3.0, 2.0, False), (RING - 1.4, 1.65, False), (RING - 0.4, 0.9, False), (RING, -0.2, False),
        (RING + 0.01, -6.0, False)]
bm = bmesh.new()
rings = []
for (d, z, _) in prof:
    rings.append([bm.verts.new((x, y, z)) for (x, y) in rrect(d)])
n = len(rings[0])
polished = []
for i in range(len(rings) - 1):
    a, b = rings[i], rings[i + 1]
    for k in range(n):
        j = (k + 1) % n
        f = bm.faces.new((a[k], a[j], b[j], b[k]))
        polished.append(prof[i][2] and prof[i + 1][2])
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
frame = mesh_obj('surround', bm)
for f in frame.data.polygons:
    f.use_smooth = True

# Satin anodised graphite: roughness and a micro bump from the ComfyUI grain (procedural noise
# without it).
m, NT = new_mat('graphite')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.075, 0.078, 0.085, 1.0)
bsdf.inputs['Metallic'].default_value = 1.0
tc = node(NT, 'ShaderNodeTexCoord')
mp = node(NT, 'ShaderNodeMapping')
mp.inputs['Scale'].default_value = (0.025, 0.025, 0.025)
lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
if ARGS.get('grain'):
    tex = node(NT, 'ShaderNodeTexImage')
    tex.image = bpy.data.images.load(ARGS['grain'])
    tex.image.colorspace_settings.name = 'Non-Color'
    tex.extension = 'REPEAT'
    lk(NT, mp.outputs[0], tex.inputs['Vector'])
    fac = tex.outputs['Color']
else:
    nz = node(NT, 'ShaderNodeTexNoise')
    nz.inputs['Scale'].default_value = 40.0
    nz.inputs['Detail'].default_value = 10.0
    lk(NT, mp.outputs[0], nz.inputs['Vector'])
    fac = nz.outputs['Fac']
mr = node(NT, 'ShaderNodeMapRange')
mr.inputs['To Min'].default_value = 0.30
mr.inputs['To Max'].default_value = 0.46
lk(NT, fac, mr.inputs['Value'])
lk(NT, mr.outputs['Result'], bsdf.inputs['Roughness'])
bump = node(NT, 'ShaderNodeBump')
bump.inputs['Strength'].default_value = 0.08
bump.inputs['Distance'].default_value = 0.05
lk(NT, fac, bump.inputs['Height'])
lk(NT, bump.outputs['Normal'], bsdf.inputs['Normal'])
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
frame.data.materials.append(m)

# Polished chamfer: dark chrome.
pm, PNT = new_mat('chamfer')
pb = node(PNT, 'ShaderNodeBsdfPrincipled')
pb.inputs['Base Color'].default_value = (0.32, 0.33, 0.35, 1.0)
pb.inputs['Metallic'].default_value = 1.0
pb.inputs['Roughness'].default_value = 0.07
po = node(PNT, 'ShaderNodeOutputMaterial')
lk(PNT, pb.outputs[0], po.inputs['Surface'])
frame.data.materials.append(pm)
for f, pol in zip(frame.data.polygons, polished):
    f.material_index = 1 if pol else 0

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

/// Runs the SDXL plate workflow for `prompt`, cached in `scratch` under `name` by its
/// fingerprint (workflow, prompts and seed), and returns the raw image.
fn sdxl_cached(ctx: &Ctx, scratch: &Path, name: &str, prompt: &str, negative: &str, seed: u64) -> Result<Img> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        prompt.as_bytes(),
        negative.as_bytes(),
        &seed.to_le_bytes(),
    ]);
    let raw_path = scratch.join(format!("{name}_raw.png"));
    let raw_fp_path = raw_path.with_extension("fp");
    let cached = raw_path.exists()
        && std::fs::read_to_string(&raw_fp_path)
            .map(|s| s == gen_fp)
            .unwrap_or(false);
    if !cached {
        println!("  comfyui {name} (SDXL)");
        let c = Comfy::connect(&ctx.paths)?;
        let mut params = Map::new();
        params.insert("prompt".into(), json!(prompt));
        params.insert("negative".into(), json!(negative));
        params.insert("seed".into(), json!(seed));
        params.insert("steps".into(), json!(32));
        params.insert("cfg".into(), json!(6.5));
        params.insert("width".into(), json!(1024));
        params.insert("height".into(), json!(1024));
        params.insert("prefix".into(), json!(format!("fh6_retro_cyber_{name}")));
        let wf = comfy::patch(&workflow, &params)?;
        let images = c.run(&wf, Duration::from_secs(900))?;
        let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
        img::write_file(&raw_path, bytes)?;
        img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
    }
    Ok(img::load(&raw_path)?.to_channels(3))
}

/// Builds the ComfyUI textures first, then the Blender surround that uses the grain, and writes
/// the cluster's ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("retro_cyber")?;
    let mut rows = Vec::new();

    // ComfyUI: the phosphor plate (shipped) and the surround grain (a Blender input).
    let raw = sdxl_cached(ctx, &scratch, "phosphor", PLATE_PROMPT, PLATE_NEGATIVE, PLATE_SEED)?;
    let plate = modulation(&raw, PLATE_SIZE, PLATE_RANGE);
    let rel = format!("{DIR}/phosphor_grain.png");
    let bytes = img::save_png(&plate, &ctx.abs(&rel), false, true)?;
    rows.push(format!(
        "| `phosphor_grain.png` | grey modulation, tileable | {s}x{s} | {} KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={PLATE_SEED} prompt=\"{PLATE_PROMPT}\" → grey, seamless, high-pass, {}–{} display level, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | tinted fill of every lit bar, pod and segment |",
        bytes / 1024,
        PLATE_RANGE.0,
        PLATE_RANGE.1,
        img::tile_seam_score(&plate),
        s = PLATE_SIZE
    ));
    let raw = sdxl_cached(ctx, &scratch, "grain", GRAIN_PROMPT, GRAIN_NEGATIVE, GRAIN_SEED)?;
    let grain = modulation(&raw, GRAIN_SIZE, (0.0, 1.0));
    let grain_path = scratch.join("surround_grain.png");
    img::save_png(&grain, &grain_path, false, true)?;
    let grain_note = format!(
        "surround grain comfyui:workflows/sdxl_plate.json seed={GRAIN_SEED} prompt=\"{GRAIN_PROMPT}\" → grey, seamless, high-pass, tile seam score {:.2}",
        img::tile_seam_score(&grain)
    );

    // Blender: the surround per view and light, its satin face broken up by the grain.
    for v in &VIEWS {
        let (view, (w, h), r, ring) = (v.name, v.size, v.r, v.ring);
        let params = json!({
            "out": scratch.to_string_lossy(),
            "seed": BLENDER_SEED,
            "w": w, "h": h, "r": r, "ring": ring,
            "strip_drop": v.strip_drop, "hump": [v.hump.0, v.hump.1],
            "blend": LENS_BLEND, "overlap": HUMP_OVERLAP, "points": OUTLINE_POINTS,
            "margin": MARGIN, "scale": SCALE,
            "samples": 96,
            "prefix": format!("surround_{view}"),
            "grain": grain_path.to_string_lossy(),
            "lights": LIGHTS.iter().map(|(n, a)| json!([n, a])).collect::<Vec<_>>(),
        });
        println!("  blender surround_{view}");
        blender::run(&ctx.paths, &format!("retro_cyber_surround_{view}"), HOUSING, &params)?;
        for (name, az) in LIGHTS {
            let file = format!("surround_{view}_{name}.png");
            let im = img::load(&scratch.join(&file))?.to_channels(4);
            let rel = format!("{DIR}/{file}");
            let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
            rows.push(format!(
                "| `{file}` | colour, straight alpha | {}x{} | {} KB | blender:retro_cyber::HOUSING view={view} lens={w}x{h} r={r} strip_drop={} hump={}..{} blend={LENS_BLEND} ring={ring} light={az}° scale={SCALE} samples=96 seed={BLENDER_SEED}; {grain_note} | MIT (asset-forge); grain CreativeML Open RAIL++-M output (SDXL) | the surround; blended with the other light by lateral G |",
                v.strip_drop,
                v.hump.0,
                v.hump.1,
                im.w,
                im.h,
                bytes / 1024
            ));
        }
    }

    // The cluster's own ledger.
    let mut md = String::from(
        "# RetroCyber assets\n\nBuilt by `cargo run --release -p asset-forge -- build --set retro_cyber` (recipe `tools/asset-forge/src/recipes/retro_cyber.rs`). The cluster draws a procedural fallback for each when the file is missing.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    let count = rows.len();
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ ({count} files + ASSETS.md)");
    Ok(())
}

/// A grey modulation from an SDXL macro: luma, seamless, high-passed (the 1/32-scale blur
/// removes lighting gradients), then the 2nd..98th percentile mapped to `range`.
fn modulation(raw: &Img, n: usize, range: (f32, f32)) -> Img {
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
        *v = range.0 + (range.1 - range.0) * k;
    }
    Img::from_fn(n, n, 3, |x, y| {
        let v = hp[y * n + x];
        [v, v, v, 0.0]
    })
}
