//! BoschDdu's bespoke assets (`assets/clusters/bosch_ddu/`, ledger
//! `assets/clusters/bosch_ddu/ASSETS.md`):
//!
//! - **Carbon weave (ComfyUI, SDXL + 4x-UltraSharp):** a macro of a raw carbon-fibre tow, made
//!   grey, seamless and high-passed, then laid into an exact 2×2 twill (SDXL draws filaments well
//!   but not a regular weave): crowned tows, grooves, the over/under dips, the filaments streaked
//!   along each tow. It is the texture the Blender carbon material is built from (tow colour and
//!   the bump under the clear coat), so the weave on the housing is a real weave, not a hatch.
//! - **Housing (Blender, Cycles):** the open-bottom U modelled in 3D at 1 Blender unit per 4K px:
//!   a short outer wall with a rounded edge, the 14 px flat carbon face, a rounded lip, the deep
//!   28 px 45° chamfer down to the cavity, and four Allen cap screws. Clear-coated carbon (the
//!   weave above) and machined steel screws, lit by a studio HDRI and a key light 35° above the
//!   plane. Rendered top-down and orthographic at 2 px per 4K px with a transparent film, once
//!   per key-light azimuth (−160° and −110°, the virtual light's ±25° swing); the cluster blends
//!   the two with lateral G.
//!
//! `cargo run --release -p asset-forge -- build --set bosch_ddu`. The SDXL render is cached in
//! the forge scratch by its fingerprint; the Blender renders re-run on every build. The files go
//! straight to the cluster's folder with its own ledger, so the shared `assets/ASSETS.md` is not
//! touched. The cluster keeps its procedural housing when the files are missing.
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
const DIR: &str = "assets/clusters/bosch_ddu";
/// Render scale: image px per 4K local px.
const SCALE: f32 = 2.0;
/// Key-light azimuths (degrees, screen convention: clockwise from 3 o'clock, y down).
const LIGHTS: [(&str, f32); 2] = [("l160", -160.0), ("l110", -110.0)];
/// The housing, as in the cluster's `Geom` (local 4K px, y down): half width, top, the open
/// bottom line, the top corner radius, the face and chamfer widths.
const HALF_W: f32 = 700.0;
/// See [`HALF_W`].
const TOP: f32 = -306.0;
/// See [`HALF_W`].
const OPEN: f32 = 234.0;
/// See [`HALF_W`].
const RADIUS: f32 = 96.0;
/// See [`HALF_W`].
const FACE: f32 = 14.0;
/// See [`HALF_W`].
const CHAMFER: f32 = 28.0;
/// Margin round the housing on the top, left and right (the render's rect in the cluster is
/// x ±(HALF_W + MARGIN), y TOP − MARGIN .. OPEN).
const MARGIN: f32 = 6.0;
/// The four Allen screws (local px, y down) and their head radius.
const BOLTS: [(f32, f32); 4] = [(-667.0, -273.0), (667.0, -273.0), (-693.0, 96.0), (693.0, 96.0)];
/// See [`BOLTS`].
const BOLT_R: f32 = 5.5;
/// One weave tile on the housing (local px): about 12 px tows.
const WEAVE_TILE: f32 = 192.0;
/// Blender seed and Cycles samples.
const BLENDER_SEED: u64 = 1204;
/// See [`BLENDER_SEED`].
const SAMPLES: u32 = 128;

/// The weave macro.
const WEAVE_PROMPT: &str = "extreme macro photograph of a raw carbon fibre tow, thousands of fine glossy black filaments with tiny specular glints and slight waviness, filling the whole frame, flat even studio lighting, seamless surface texture, monochrome black and grey, ultra detailed, sharp focus";
const WEAVE_NEGATIVE: &str = "text, letters, numbers, watermark, logo, frame, border, vignette, gradient, shadow, perspective, people, lowres, blurry, jpeg artifacts, color, weave pattern, grid, honeycomb";
const WEAVE_SEED: u64 = 1203;
/// Shipped weave size.
const WEAVE_SIZE: usize = 1024;

/// The housing: the open-bottom U as rings of an offset path, one ring per profile point. Params:
/// `half_w`, `top`, `open`, `radius`, `face`, `chamfer`, `margin`, `scale`, `bolts`, `bolt_r`,
/// `weave` (path), `weave_tile`, `lights` (list of `[name, azimuth_deg]`), `samples`, `prefix`.
pub const HOUSING: &str = r#"
sc = reset()
gpu_cycles(sc, samples=ARGS.get('samples', 128), denoise=True, bounces=8)
sc.render.film_transparent = True
view(sc, 'Standard')
X0, TOP, OPEN, R = float(ARGS['half_w']), float(ARGS['top']), float(ARGS['open']), float(ARGS['radius'])
FACE, CH, MARGIN, S = float(ARGS['face']), float(ARGS['chamfer']), float(ARGS['margin']), float(ARGS['scale'])
LEG_END = OPEN + 80.0
NA = 32

def u_path(d):
    # The outer boundary offset inwards by d (screen coords, y down), from the bottom of the left
    # leg, over the top, to the bottom of the right leg; returned in Blender coords (y up).
    r = max(R - d, 0.5)
    cxl, cxr, cy = -X0 + R, X0 - R, TOP + R
    pts = [(-X0 + d, LEG_END), (-X0 + d, cy)]
    for i in range(1, NA):
        a = math.pi + 0.5 * math.pi * i / NA
        pts.append((cxl + r * math.cos(a), cy + r * math.sin(a)))
    pts += [(cxl, TOP + d), (cxr, TOP + d)]
    for i in range(1, NA):
        a = 1.5 * math.pi + 0.5 * math.pi * i / NA
        pts.append((cxr + r * math.cos(a), cy + r * math.sin(a)))
    pts += [(X0 - d, cy), (X0 - d, LEG_END)]
    return [(x, -y) for (x, y) in pts]

ZF = 6.0
# Profile from the outer edge inwards: (offset, z). The outer wall, a rounded outer edge, the
# flat face, a rounded lip, the 45° chamfer and the cavity floor.
prof = [(0.0, -12.0), (0.0, ZF - 3.0), (0.4, ZF - 1.6), (1.2, ZF - 0.5), (2.6, ZF),
        (FACE - 1.4, ZF), (FACE - 0.5, ZF - 0.35), (FACE + 0.4, ZF - 1.1),
        (FACE + CH, ZF - 1.1 - (CH - 0.4)), (FACE + CH + 0.5, ZF - 1.1 - (CH - 0.4) - 0.2)]
bm = bmesh.new()
rings = [[bm.verts.new((x, y, z)) for (x, y) in u_path(d)] for (d, z) in prof]
for i in range(len(rings) - 1):
    a, b = rings[i], rings[i + 1]
    for k in range(len(a) - 1):
        bm.faces.new((a[k], a[k + 1], b[k + 1], b[k]))
bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
# Up-facing normals (seen from the top camera).
for f in bm.faces:
    if f.normal.z < -1e-4:
        f.normal_flip()
frame = mesh_obj('housing', bm)
for p in frame.data.polygons:
    p.use_smooth = True

# Clear-coated carbon: the SDXL weave drives the tow colour and the bump under the coat.
m, NT = new_mat('carbon')
bsdf = node(NT, 'ShaderNodeBsdfPrincipled')
tc = node(NT, 'ShaderNodeTexCoord')
mp = node(NT, 'ShaderNodeMapping')
t = 1.0 / float(ARGS['weave_tile'])
mp.inputs['Scale'].default_value = (t, t, 1.0)
lk(NT, tc.outputs['Object'], mp.inputs['Vector'])
tex = node(NT, 'ShaderNodeTexImage', interpolation='Cubic', extension='REPEAT')
tex.image = bpy.data.images.load(ARGS['weave'], check_existing=True)
tex.image.colorspace_settings.name = 'Non-Color'
lk(NT, mp.outputs[0], tex.inputs['Vector'])
col = node(NT, 'ShaderNodeMapRange')
col.inputs['To Min'].default_value = 0.006
col.inputs['To Max'].default_value = 0.032
lk(NT, tex.outputs['Color'], col.inputs['Value'])
lk(NT, col.outputs['Result'], bsdf.inputs['Base Color'])
rough = node(NT, 'ShaderNodeMapRange')
rough.inputs['To Min'].default_value = 0.42
rough.inputs['To Max'].default_value = 0.28
lk(NT, tex.outputs['Color'], rough.inputs['Value'])
lk(NT, rough.outputs['Result'], bsdf.inputs['Roughness'])
bump = node(NT, 'ShaderNodeBump')
bump.inputs['Strength'].default_value = 0.35
bump.inputs['Distance'].default_value = 0.6
lk(NT, tex.outputs['Color'], bump.inputs['Height'])
lk(NT, bump.outputs['Normal'], bsdf.inputs['Normal'])
bsdf.inputs['Coat Weight'].default_value = 1.0
bsdf.inputs['Coat Roughness'].default_value = 0.05
bsdf.inputs['Coat IOR'].default_value = 1.5
out = node(NT, 'ShaderNodeOutputMaterial')
lk(NT, bsdf.outputs[0], out.inputs['Surface'])
frame.data.materials.append(m)

# Allen cap screws: a machined button head and a black hex socket.
sm, SNT = new_mat('steel')
sb = node(SNT, 'ShaderNodeBsdfPrincipled')
sb.inputs['Base Color'].default_value = (0.32, 0.33, 0.35, 1.0)
sb.inputs['Metallic'].default_value = 1.0
sb.inputs['Roughness'].default_value = 0.22
so = node(SNT, 'ShaderNodeOutputMaterial')
lk(SNT, sb.outputs[0], so.inputs['Surface'])
dk, DNT = new_mat('socket')
db = node(DNT, 'ShaderNodeBsdfPrincipled')
db.inputs['Base Color'].default_value = (0.0, 0.0, 0.0, 1.0)
db.inputs['Roughness'].default_value = 0.9
do = node(DNT, 'ShaderNodeOutputMaterial')
lk(DNT, db.outputs[0], do.inputs['Surface'])
br = float(ARGS['bolt_r'])
for (x, y) in ARGS['bolts']:
    bpy.ops.mesh.primitive_uv_sphere_add(radius=br, segments=48, ring_count=24, location=(x, -y, ZF))
    head = bpy.context.active_object
    head.scale = (1.0, 1.0, 0.42)
    bpy.ops.object.shade_smooth()
    head.data.materials.append(sm)
    bpy.ops.mesh.primitive_cylinder_add(vertices=6, radius=br * 0.48, depth=br * 0.6, location=(x, -y, ZF + br * 0.42))
    sock = bpy.context.active_object
    sock.rotation_euler = (0.0, 0.0, math.radians(15.0))
    sock.data.materials.append(dk)

studio_world(sc, strength=0.3, rotation=0.6, visible=False)
span_x = 2.0 * (X0 + MARGIN)
y_top, y_bot = TOP - MARGIN, OPEN
span_y = y_bot - y_top
resolution(sc, round(span_x * S), round(span_y * S))
ortho_cam(sc, span_x, center=(0.0, -(y_top + y_bot) * 0.5), z=80.0, clip_end=300.0)
key = None
for (name, az) in ARGS['lights']:
    if key is not None:
        bpy.data.objects.remove(key, do_unlink=True)
    # Screen azimuth (y down) -> Blender (y up); the key sits 35° above the plane.
    a = math.radians(az)
    dx, dy = math.cos(a), -math.sin(a)
    dist = span_x
    el = math.radians(35.0)
    loc = (dx * dist * math.cos(el), dy * dist * math.cos(el), dist * math.sin(el))
    key = area_light('key', loc, (0.0, 0.0, 0.0), dist * 0.3, 1.1e7)
    save(sc, os.path.join(OUT, '%s_%s.png' % (ARGS['prefix'], name)), depth='8', mode='RGBA')
done()
"#;

/// Builds the weave and the housing renders, and writes the cluster's ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let mut rows = Vec::new();
    let scratch = ctx.scratch("bosch_ddu")?;

    // ComfyUI: the weave macro (cached by fingerprint).
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        WEAVE_PROMPT.as_bytes(),
        WEAVE_NEGATIVE.as_bytes(),
        &WEAVE_SEED.to_le_bytes(),
    ]);
    let raw_path = scratch.join("fibre_raw.png");
    let raw_fp_path = raw_path.with_extension("fp");
    let cached = raw_path.exists()
        && std::fs::read_to_string(&raw_fp_path)
            .map(|s| s == gen_fp)
            .unwrap_or(false);
    if !cached {
        println!("  comfyui carbon weave (SDXL)");
        let c = Comfy::connect(&ctx.paths)?;
        let mut params = Map::new();
        params.insert("prompt".into(), json!(WEAVE_PROMPT));
        params.insert("negative".into(), json!(WEAVE_NEGATIVE));
        params.insert("seed".into(), json!(WEAVE_SEED));
        params.insert("steps".into(), json!(32));
        params.insert("cfg".into(), json!(6.5));
        params.insert("width".into(), json!(1024));
        params.insert("height".into(), json!(1024));
        params.insert("prefix".into(), json!("fh6_bosch_ddu_fibre"));
        let wf = comfy::patch(&workflow, &params)?;
        let images = c.run(&wf, Duration::from_secs(900))?;
        let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
        img::write_file(&raw_path, bytes)?;
        img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
    }
    let raw = img::load(&raw_path)?.to_channels(3);
    let weave = weave_plate(&raw);
    let weave_rel = format!("{DIR}/carbon_weave.png");
    let bytes = img::save_png(&weave, &ctx.abs(&weave_rel), false, false)?;
    rows.push(format!(
        "| `carbon_weave.png` | grey height, tileable | {s}x{s} | {} KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={WEAVE_SEED} prompt=\"{WEAVE_PROMPT}\" → grey, seamless, high-pass, laid into a 2×2 twill of {TOWS} tows per tile with the filaments streaked along each tow, tile seam score {:.2} | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the carbon material of the Blender housing (tow colour and the bump under the clear coat) |",
        bytes / 1024,
        img::tile_seam_score(&weave),
        s = WEAVE_SIZE
    ));

    // Blender: the housing per key light, built on the weave.
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "half_w": HALF_W, "top": TOP, "open": OPEN, "radius": RADIUS,
        "face": FACE, "chamfer": CHAMFER, "margin": MARGIN, "scale": SCALE,
        "bolts": BOLTS.iter().map(|(x, y)| json!([x, y])).collect::<Vec<_>>(),
        "bolt_r": BOLT_R,
        "weave": ctx.abs(&weave_rel).to_string_lossy(),
        "weave_tile": WEAVE_TILE,
        "samples": SAMPLES,
        "prefix": "housing",
        "lights": LIGHTS.iter().map(|(n, a)| json!([n, a])).collect::<Vec<_>>(),
    });
    println!("  blender housing");
    blender::run(&ctx.paths, "bosch_ddu_housing", HOUSING, &params)?;
    for (name, az) in LIGHTS {
        let file = format!("housing_{name}.png");
        let im = img::load(&scratch.join(&file))?.to_channels(4);
        let rel = format!("{DIR}/{file}");
        let bytes = img::save_png(&im, &ctx.abs(&rel), false, true)?;
        rows.push(format!(
            "| `{file}` | colour, straight alpha | {}x{} | {} KB | blender:bosch_ddu::HOUSING open-bottom U {w}x{h} r={RADIUS} face={FACE} chamfer={CHAMFER} 45° light={az}° scale={SCALE} samples={SAMPLES} seed={BLENDER_SEED}, carbon from `carbon_weave.png` tiled at {WEAVE_TILE} px | MIT (asset-forge) | the housing; blended with the other light by lateral G |",
            im.w,
            im.h,
            bytes / 1024,
            w = 2.0 * HALF_W,
            h = OPEN - TOP,
        ));
    }

    let mut md = String::from(
        "# BoschDdu assets\n\nBuilt by `cargo run --release -p asset-forge -- build --set bosch_ddu` (recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`). The cluster draws its procedural housing when the files are missing. The housing renders cover x ±706, y −312 .. 234 in local 4K px at 2 px per px.\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (3 files + ASSETS.md)");
    Ok(())
}

/// Tows across one weave tile (a 2×2 twill repeats every 4 tows).
const TOWS: usize = 16;

/// The weave height: an exact 2×2 twill laid out in Rust, with the SDXL fibre macro streaked
/// along each tow. Each cell is one tow float, horizontal or vertical by the twill rule
/// `(i + j) mod 4 < 2`; it has a crowned cross-section, dips where it passes under its
/// neighbour, and carries the fibres (the macro sampled 12× compressed along the tow, so its
/// grain becomes filaments). Horizontal tows sit 20% brighter, as a light catches one tow
/// direction more than the other. Seamless by construction.
fn weave_plate(raw: &Img) -> Img {
    let n = WEAVE_SIZE;
    // The fibre source: luma, seamless, high-passed, 2..98% stretched to 0..1.
    let small = raw.resize(n, n, false);
    let grey = Img::from_fn(n, n, 1, |x, y| {
        let v = img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2));
        [v, 0.0, 0.0, 0.0]
    });
    let tile = img::make_seamless(&grey, 0.2);
    let low = tile.resize(n / 16, n / 16, true).resize(n, n, true);
    let mut hp: Vec<f32> = (0..n * n).map(|i| tile.data[i] - low.data[i]).collect();
    let mut sorted = hp.clone();
    sorted.sort_by(f32::total_cmp);
    let lo = sorted[(sorted.len() as f32 * 0.02) as usize];
    let hi = sorted[(sorted.len() as f32 * 0.98) as usize];
    let span = (hi - lo).max(1e-4);
    for v in &mut hp {
        *v = ((*v - lo) / span).clamp(0.0, 1.0);
    }
    let fibre = Img {
        w: n,
        h: n,
        c: 1,
        data: hp,
    };
    let cell = n as f32 / TOWS as f32;
    Img::from_fn(n, n, 3, |x, y| {
        let (fx, fy) = (x as f32 / cell, y as f32 / cell);
        let (i, j) = (fx.floor() as usize, fy.floor() as usize);
        let (u, v) = (fx.fract(), fy.fract());
        let horizontal = (i + j) % 4 < 2;
        // Across the tow: a crowned profile with a dark groove at the edges. Along it: the float
        // dips at the cell ends where the tow passes under (the float spans two cells).
        let (across, along) = if horizontal { (v, u) } else { (u, v) };
        let crown = (std::f32::consts::PI * across).sin().max(0.0).powf(0.3);
        let start = (i + j) % 4 == 0 || (i + j) % 4 == 2;
        let pos = if start { along * 0.5 } else { 0.5 + along * 0.5 };
        let dip = (std::f32::consts::PI * pos).sin().powf(0.25);
        // Fibres: the macro compressed 12× along the tow.
        let (sx, sy) = if horizontal {
            (x as f32 / 12.0, y as f32)
        } else {
            (x as f32, y as f32 / 12.0)
        };
        let f = fibre.sample(sx.rem_euclid(n as f32), sy.rem_euclid(n as f32), 0, true);
        let sheen = if horizontal { 0.2 } else { 0.0 };
        // The groove between tows stays at 30% (black grooves read as brick, not twill).
        let h = (0.3 + 0.4 * crown * dip + 0.22 * f * crown + sheen).clamp(0.0, 1.0);
        [h, h, h, 0.0]
    })
}
