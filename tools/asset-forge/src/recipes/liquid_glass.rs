//! Liquid Glass's bespoke assets (`assets/clusters/liquid_glass/`, ledger
//! `assets/clusters/liquid_glass/ASSETS.md`). The glass itself is maths in the kit
//! (`fh6_kit::OpticalGlass`, `docs/specs/optics.md`); these assets are what it reflects and what
//! it bends.
//!
//! Order (ComfyUI first, Blender last):
//! - **Studio plate (ComfyUI, SDXL + 4x-UltraSharp):** a 4096 × 2048 panorama of a dark studio
//!   with ceiling tube lights, seamless left to right. It is the picture on the cyclorama of the
//!   Blender step, so it is what the glass rims mirror between the big lights.
//! - **Studio environment (Blender, Cycles):** the studio the glass reflects, rendered as six
//!   cube faces from the cluster's position: the key softbox, the strip light and a warm kicker
//!   as emissive panels in exactly the directions of the kit's analytic studio, in front of a
//!   cyclorama that carries the ComfyUI plate. The faces are resampled here into one octahedral
//!   map (the layout `shape.wgsl` `oct_uv` reads).
//! - **Liquid light (ray traced here):** the caustic that sunlight draws on the floor of a
//!   rippled pool, by Snell's law. The cluster runs it through the rev ribbon under the glass
//!   rod, so the light reads as liquid and the lens has detail to magnify. An SDXL caustic was
//!   tried first and rejected: it did not tile and was not a caustic.
//!
//! `cargo run --release -p asset-forge -- build --set liquid_glass` (hold the GPU lock when the
//! ComfyUI plate or the Blender faces are stale; a build that finds both fresh uses no GPU).
//! Every step is cached in the forge scratch by fingerprint. The 4K masters are copied to the
//! Drive masters folder when it is mounted.
use crate::blender;
use crate::comfy::Comfy;
use crate::ctx::Ctx;
use crate::error::{io_err, tool_err, Result};
use crate::hero::sdxl::{self, Job};
use crate::hero::texture::Models;
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::procedural::Rng;
use serde_json::json;
use std::f32::consts::{FRAC_1_SQRT_2, TAU};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Output folder (repo-relative).
const DIR: &str = "assets/clusters/liquid_glass";
/// The Drive masters folder (override with `FH6_MASTERS`), and this dash's subfolder.
const MASTERS: &str = r"G:\My Drive\FH6 WheelDash v2 - 4K Masters";
const MASTERS_SUB: &str = "liquid_glass";

/// Size of the liquid-light tile, and vertical rays traced per texel side.
const LIGHT_SIZE: usize = 4096;
const LIGHT_RAYS: usize = 3;
const LIGHT_SEED: u64 = 77_101;
/// Refractive index of water (20 °C, sodium D line).
const WATER_IOR: f32 = 1.333;
/// The rippled surface: number of waves, their band in whole cycles per tile, the fall of
/// amplitude with wave number (`k^-RIPPLE_FALLOFF`) and the RMS slope.
const RIPPLE_WAVES: usize = 64;
const RIPPLE_BAND: (f32, f32) = (12.0, 72.0);
const RIPPLE_FALLOFF: f32 = 2.0;
const RIPPLE_SLOPE: f32 = 0.14;
/// Depth of the floor in mean focal depths of the surface (at 1 an average ripple focuses
/// exactly on the floor).
const POOL_FOCUS: f32 = 1.0;
/// Irradiance that 8-bit white stores, relative to the floor under still water. The cluster
/// multiplies its light colour by this.
pub const LIGHT_RANGE: f32 = 4.0;
/// Irradiance that white stores in the 16-bit master.
const LIGHT_MASTER_RANGE: f32 = 16.0;

/// The studio plate.
const PLATE_PROMPT: &str = "360 degree panorama of the inside of a large dark automotive photo studio \
    at night, matte black walls and ceiling, a few distant dim practical lights, lighting stands and \
    equipment silhouettes, thin haze, glossy black floor with faint reflections, low key, cinematic, \
    photographic, wide equirectangular panorama";
const PLATE_NEGATIVE: &str = "people, car, text, logo, watermark, bright window, daylight, frame, border, \
    vignette, cartoon, illustration, blurry, jpeg artifacts";
const PLATE_SEED: u64 = 77_203;
/// Plate size (2:1).
const PLATE_SIZE: (usize, usize) = (4096, 2048);
/// Radiance of the plate's white on the cyclorama: its ceiling lights show in the rounded edges as
/// faint second reflections, well under the studio's own lights.
const PLATE_RADIANCE: f32 = 0.30;

/// Octahedral map size.
const ENV_SIZE: usize = 4096;
/// Cube face size and field of view (wider than 90° so bilinear taps never leave a face).
const FACE_SIZE: usize = 2048;
const FACE_FOV_DEG: f32 = 94.0;
/// Radiance that 8-bit white stores. The cluster sets `OpticalGlass::env_gain` to this.
pub const ENV_RANGE: f32 = 16.0;
/// Directions closer than `VIEWER_CLEAR.0` degrees to the viewer are black, fading in by
/// `VIEWER_CLEAR.1`: a flat glass face mirrors exactly that direction, and anything there would
/// veil the numerals under it.
const VIEWER_CLEAR: (f32, f32) = (22.0, 34.0);
/// Blender seed.
const BLENDER_SEED: u64 = 77_301;

/// One emissive panel of the studio, in the kit's environment frame (x right, y down, z at the
/// viewer): centre direction, tangent axes, half size in tangent units, radiance and edge
/// softness. The first two match `studio()` in `shape.wgsl`.
struct Panel {
    name: &'static str,
    c: [f32; 3],
    tx: [f32; 3],
    ty: [f32; 3],
    half: [f32; 2],
    radiance: [f32; 3],
    soft: f32,
    grid: bool,
}

const PANELS: [Panel; 3] = [
    Panel {
        name: "key",
        c: [-0.5417, -0.5417, 0.6428],
        tx: [FRAC_1_SQRT_2, -FRAC_1_SQRT_2, 0.0],
        ty: [0.4545, 0.4545, 0.7660],
        half: [0.75, 0.08],
        radiance: [8.0, 8.0, 8.0],
        soft: 0.02,
        grid: false,
    },
    Panel {
        name: "strip",
        c: [0.6409, 0.6409, 0.4226],
        tx: [-FRAC_1_SQRT_2, FRAC_1_SQRT_2, 0.0],
        ty: [-0.2988, -0.2988, 0.9063],
        half: [1.30, 0.06],
        radiance: [4.2, 4.7, 5.2],
        soft: 0.02,
        grid: false,
    },
    Panel {
        name: "kicker",
        c: [-0.8529, 0.4924, 0.1736],
        tx: [-0.5, -0.8660, 0.0],
        ty: [0.1504, -0.0868, 0.9848],
        half: [0.20, 0.08],
        radiance: [3.4, 2.3, 1.2],
        soft: 0.03,
        grid: false,
    },
];

/// The horizon ring: a band of light from the plane of the screen to `RING_ELEV_DEG` above it, all
/// the way round. Its radiance is `RING_BASE` plus a lobe `cos^RING_POWER` towards the key (kit
/// azimuth `RING_KEY_AZ_DEG`) and a cooler lobe opposite it. A rounded edge mirrors the ring as one
/// thin line along its outline. Matches `studio()` in `shape.wgsl`.
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

/// The studio as six cube faces. Params: `size`, `fov`, `samples`, `plate` (path), `plate_radiance`,
/// `panels` (list of `{name, c, tx, ty, half, radiance, soft, grid}` in the kit frame), `ring`
/// (`{elev_deg, base, key, far, key_az, far_az, power}`), `faces` (list of `[name, forward, up]` in
/// Blender's frame).
pub const STUDIO: &str = r#"
import numpy as np
sc = reset()
gpu_cycles(sc, samples=int(ARGS.get('samples', 24)), denoise=False, bounces=1)
sc.render.film_transparent = False
view(sc, 'Standard')
N = int(ARGS['size'])
resolution(sc, N, N)

# A black world: every photon comes from a panel or the cyclorama.
w = bpy.data.worlds.new('void')
sc.world = w
try:
    w.use_nodes = True
except Exception:
    pass
wnt = w.node_tree
wnt.nodes.clear()
wout = node(wnt, 'ShaderNodeOutputWorld')
wbg = node(wnt, 'ShaderNodeBackground')
wbg.inputs['Color'].default_value = (0.0, 0.0, 0.0, 1.0)
wbg.inputs['Strength'].default_value = 0.0
lk(wnt, wbg.outputs[0], wout.inputs['Surface'])

def conv(v):
    # Kit frame (y down the screen) to Blender (y up).
    return Vector((v[0], -v[1], v[2]))

def smooth(e0, e1, x):
    t = np.clip((x - e0) / (e1 - e0), 0.0, 1.0)
    return t * t * (3.0 - 2.0 * t)

def panel_image(p):
    hx, hy = p['half']
    tw = 1024
    th = max(8, int(round(tw * hy / hx)))
    u = (np.arange(tw, dtype=np.float32) + 0.5) / tw
    v = (np.arange(th, dtype=np.float32) + 0.5) / th
    U, V = np.meshgrid(u, v)
    # Distance to the nearest edge in tangent units, for an even soft border.
    edge = np.minimum(np.minimum(U, 1.0 - U) * 2.0 * hx, np.minimum(V, 1.0 - V) * 2.0 * hy)
    fade = smooth(0.0, 2.0 * float(p['soft']), edge)
    # A diffuser is brighter in the middle than at its edges.
    hot = np.exp(-(((U - 0.5) / 0.46) ** 2 + ((V - 0.5) / 0.46) ** 2))
    val = (0.74 + 0.50 * hot) * fade
    if p.get('grid'):
        # The seams of the diffuser fabric and its egg crate.
        gu = np.abs(((U * 12.0) % 1.0) - 0.5) > 0.47
        gv = np.abs(((V * 4.0) % 1.0) - 0.5) > 0.47
        val = val * np.where(gu | gv, 0.93, 1.0)
    rgb = np.array(p['radiance'], dtype=np.float32)
    px = np.zeros((th, tw, 4), dtype=np.float32)
    px[..., 0:3] = val[..., None] * rgb[None, None, :]
    px[..., 3] = 1.0
    im = bpy.data.images.new('panel_' + p['name'], tw, th, alpha=False, float_buffer=True)
    im.colorspace_settings.name = 'Non-Color'
    im.pixels.foreach_set(px.ravel())
    im.pack()
    return im

def emit_image(name, image, strength):
    m, nt = new_mat(name)
    tex = node(nt, 'ShaderNodeTexImage')
    tex.image = image
    tex.extension = 'EXTEND'
    uv = node(nt, 'ShaderNodeUVMap')
    lk(nt, uv.outputs['UV'], tex.inputs['Vector'])
    emission_out(nt, tex.outputs['Color'], strength)
    return m

for p in ARGS['panels']:
    c, tx, ty = conv(p['c']), conv(p['tx']), conv(p['ty'])
    hx, hy = p['half']
    D = 6.0
    bm = bmesh.new()
    vs = [bm.verts.new(tuple((c + tx * (a * hx) + ty * (b * hy)) * D)) for (a, b) in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
    f = bm.faces.new(vs)
    uvl = bm.loops.layers.uv.new('UVMap')
    for loop, uv in zip(f.loops, ((0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0))):
        loop[uvl].uv = uv
    ob = mesh_obj('panel_' + p['name'], bm)
    ob.data.materials.append(emit_image('m_' + p['name'], panel_image(p), 1.0))

# The horizon ring: a band round the screen's normal, from the plane of the screen upwards.
ring = ARGS.get('ring')
if ring:
    RR, RSEG = 9.0, 256
    top = RR * math.tan(math.radians(float(ring['elev_deg'])))
    tw, th = 2048, 32
    az = (np.arange(tw, dtype=np.float32) + 0.5) / tw * 2.0 * math.pi
    def lobe(a0):
        return np.maximum(np.cos(az - math.radians(float(a0))), 0.0) ** float(ring['power'])
    rgb = (float(ring['base'])
           + np.array(ring['key'], dtype=np.float32)[None, :] * lobe(ring['key_az'])[:, None]
           + np.array(ring['far'], dtype=np.float32)[None, :] * lobe(ring['far_az'])[:, None])
    px = np.zeros((th, tw, 4), dtype=np.float32)
    px[..., 0:3] = rgb[None, :, :]
    px[..., 3] = 1.0
    im = bpy.data.images.new('ring', tw, th, alpha=False, float_buffer=True)
    im.colorspace_settings.name = 'Non-Color'
    im.pixels.foreach_set(px.ravel())
    im.pack()
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new('UVMap')
    col = []
    for i in range(RSEG + 1):
        a = 2.0 * math.pi * i / RSEG
        # Kit azimuth runs clockwise on the screen (y down); Blender's y is up.
        x, y = RR * math.cos(a), -RR * math.sin(a)
        col.append((bm.verts.new((x, y, 0.0)), bm.verts.new((x, y, top)), i / RSEG))
    for i in range(RSEG):
        (b0, t0, u0), (b1, t1, u1) = col[i], col[i + 1]
        f = bm.faces.new((b0, b1, t1, t0))
        for loop, uv in zip(f.loops, ((u0, 0.0), (u1, 0.0), (u1, 1.0), (u0, 1.0))):
            loop[uvl].uv = uv
    ob = mesh_obj('ring', bm)
    ob.data.materials.append(emit_image('m_ring', im, 1.0))

# The cyclorama: a cylinder round the screen's up axis carrying the ComfyUI studio plate. Its
# seam (u = 0) faces the viewer, where the map is cleared anyway.
plate_path = ARGS.get('plate', '')
if plate_path and os.path.isfile(plate_path):
    R, H, SEG = 14.0, 14.0, 192
    bm = bmesh.new()
    uvl = bm.loops.layers.uv.new('UVMap')
    ring = []
    for i in range(SEG + 1):
        a = 2.0 * math.pi * i / SEG
        x, z = R * math.sin(a), R * math.cos(a)
        ring.append((bm.verts.new((x, -H, z)), bm.verts.new((x, H, z)), i / SEG))
    for i in range(SEG):
        (b0, t0, u0), (b1, t1, u1) = ring[i], ring[i + 1]
        f = bm.faces.new((b0, b1, t1, t0))
        for loop, uv in zip(f.loops, ((u0, 0.0), (u1, 0.0), (u1, 1.0), (u0, 1.0))):
            loop[uvl].uv = uv
    cyc = mesh_obj('cyclorama', bm)
    im = bpy.data.images.load(plate_path, check_existing=True)
    cyc.data.materials.append(emit_image('m_cyclorama', im, float(ARGS.get('plate_radiance', 0.1))))
    log('cyclorama', plate_path)
else:
    log('no studio plate; panels only')

cd = bpy.data.cameras.new('cam')
cd.type = 'PERSP'
cd.sensor_fit = 'HORIZONTAL'
cd.angle = math.radians(float(ARGS['fov']))
cd.clip_start = 0.01
cd.clip_end = 1000.0
cam = bpy.data.objects.new('cam', cd)
link(cam)
sc.camera = cam
for (name, fwd, up) in ARGS['faces']:
    f, u = Vector(fwd), Vector(up)
    r = f.cross(u)
    cam.matrix_world = Matrix(((r.x, u.x, -f.x, 0.0), (r.y, u.y, -f.y, 0.0), (r.z, u.z, -f.z, 0.0), (0.0, 0.0, 0.0, 1.0)))
    save(sc, os.path.join(OUT, 'studio_%s.exr' % name), exr=True, mode='RGB')
done()
"#;

/// Builds the ComfyUI textures, the Blender studio and the environment map, copies the masters
/// to Drive and writes the cluster's ledger.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let out_dir = ctx.abs(DIR);
    std::fs::create_dir_all(&out_dir).map_err(|e| io_err(&out_dir, e))?;
    let scratch = ctx.scratch("liquid_glass")?;
    let masters = masters_dir();
    let mut rows = Vec::new();
    let mut master_rows = Vec::new();

    // 1. ComfyUI: the studio plate (the picture on Blender's cyclorama).
    let plate_path = studio_plate(ctx, &scratch)?;
    let plate_kb = std::fs::metadata(&plate_path).map_or(0, |m| m.len() / 1024);
    rows.push(format!(
        "| (no runtime file: baked into `studio_env.png`) | studio panorama, seamless left to right | {}x{} | {plate_kb} KB (master) | comfyui:liquid_glass::studio_plate (SDXL base 1.0 + refiner when installed, 1408x704, dpmpp_2m karras 32 steps cfg 6, seam column repainted at denoise 0.55, 4x-UltraSharp) seed={PLATE_SEED} prompt=\"{PLATE_PROMPT}\" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the picture on the cyclorama of the Blender studio: the ceiling lights, walls and floor that the glass rims mirror. Master `studio_plate_master.png` |",
        PLATE_SIZE.0, PLATE_SIZE.1,
    ));
    copy_master(
        masters.as_deref(),
        &plate_path,
        "studio_plate_master.png",
        &mut master_rows,
    );

    // 2. The liquid-light caustic (ray traced here; no GPU).
    let light_rel = format!("{DIR}/liquid_light.png");
    let light_master = scratch.join("liquid_light_master.png");
    let light_fp = fingerprint(&[format!(
        "caustic v1 {LIGHT_SIZE} {LIGHT_RAYS} {LIGHT_SEED} {WATER_IOR} {RIPPLE_WAVES} {RIPPLE_BAND:?} {RIPPLE_FALLOFF} {RIPPLE_SLOPE} {POOL_FOCUS} {LIGHT_RANGE} {LIGHT_MASTER_RANGE}"
    )
    .as_bytes()]);
    let light_fp_path = scratch.join("liquid_light.fp");
    let light_out = ctx.abs(&light_rel);
    if light_out.is_file()
        && light_master.is_file()
        && std::fs::read_to_string(&light_fp_path).is_ok_and(|s| s == light_fp)
    {
        println!("  = liquid light (fresh)");
    } else {
        println!(
            "  liquid light ({LIGHT_SIZE} px, {} rays, Snell caustic)",
            (LIGHT_SIZE * LIGHT_RAYS).pow(2)
        );
        let irr = caustic(LIGHT_SIZE, LIGHT_RAYS, LIGHT_SEED);
        let (lo, hi, mean) = stats(&irr.data);
        println!(
            "    irradiance: min {lo:.3}, mean {mean:.3}, max {hi:.1}; tile seam score {:.3}",
            img::tile_seam_score(&irr)
        );
        img::save_png(&encode_light(&irr, LIGHT_RANGE, true), &light_out, false, true)?;
        img::save_png(
            &encode_light(&irr, LIGHT_MASTER_RANGE, false),
            &light_master,
            true,
            false,
        )?;
        img::write_file(&light_fp_path, light_fp.as_bytes())?;
    }
    let light_kb = std::fs::metadata(&light_out).map_or(0, |m| m.len() / 1024);
    rows.push(format!(
        "| `liquid_light.png` | caustic irradiance, tileable, sRGB, white = {LIGHT_RANGE} x the floor under still water | {s}x{s} | {light_kb} KB | rust:liquid_glass::caustic (Snell's law at a rippled water surface, n = {WATER_IOR}: {RIPPLE_WAVES} waves of {}..{} cycles per tile, amplitude k^-{RIPPLE_FALLOFF}, RMS slope {RIPPLE_SLOPE}, floor at {POOL_FOCUS} mean focal depths, {LIGHT_RAYS}x{LIGHT_RAYS} vertical rays per texel weighted by Fresnel transmission, wrap-around splat) seed={LIGHT_SEED} | MIT (asset-forge) | the light in the rev ribbon under the glass rod. Master `liquid_light_master.png` |",
        RIPPLE_BAND.0,
        RIPPLE_BAND.1,
        s = LIGHT_SIZE
    ));
    copy_master(
        masters.as_deref(),
        &light_master,
        "liquid_light_master.png",
        &mut master_rows,
    );

    // 3. Blender: the studio as six cube faces.
    let params = json!({
        "out": scratch.to_string_lossy(),
        "seed": BLENDER_SEED,
        "size": FACE_SIZE,
        "fov": FACE_FOV_DEG,
        "samples": 24,
        "plate": plate_path.to_string_lossy(),
        "plate_radiance": PLATE_RADIANCE,
        "panels": PANELS.iter().map(|p| json!({
            "name": p.name, "c": p.c, "tx": p.tx, "ty": p.ty, "half": p.half,
            "radiance": p.radiance, "soft": p.soft, "grid": p.grid,
        })).collect::<Vec<_>>(),
        "ring": json!({
            "elev_deg": RING_ELEV_DEG, "base": RING_BASE, "key": RING_KEY, "far": RING_FAR,
            "key_az": RING_KEY_AZ_DEG, "far_az": RING_FAR_AZ_DEG, "power": RING_POWER,
        }),
        "faces": FACES.iter().map(|(n, f, u)| json!([n, f, u])).collect::<Vec<_>>(),
    });
    let plate_bytes = crate::ctx::read(&plate_path)?;
    let fp = fingerprint(&[STUDIO.as_bytes(), params.to_string().as_bytes(), &plate_bytes]);
    let fp_path = scratch.join("studio.fp");
    let faces_exist = FACES
        .iter()
        .all(|(n, _, _)| scratch.join(format!("studio_{n}.exr")).is_file());
    if faces_exist && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp) {
        println!("  = blender studio (fresh)");
    } else {
        println!("  blender studio ({FACE_SIZE} px x 6 faces)");
        blender::run(&ctx.paths, "liquid_glass_studio", STUDIO, &params)?;
        img::write_file(&fp_path, fp.as_bytes())?;
    }
    // 4. The octahedral environment map.
    let env_rel = format!("{DIR}/studio_env.png");
    let env_out = ctx.abs(&env_rel);
    let env_fp = fingerprint(&[
        fp.as_bytes(),
        format!("env v1 {ENV_SIZE} {ENV_RANGE} {VIEWER_CLEAR:?}").as_bytes(),
    ]);
    let env_fp_path = scratch.join("studio_env.fp");
    let env_fresh = env_out.is_file() && std::fs::read_to_string(&env_fp_path).is_ok_and(|s| s == env_fp);
    if env_fresh {
        println!("  = environment map (fresh)");
    } else {
        println!("  environment map ({ENV_SIZE} px octahedral)");
    }
    let mut faces = Vec::with_capacity(FACES.len());
    for (name, _, _) in FACES {
        let file = format!("studio_{name}.exr");
        let path = scratch.join(&file);
        if !env_fresh {
            faces.push(img::load_exr(&path)?.to_channels(3));
        }
        copy_master(masters.as_deref(), &path, &file, &mut master_rows);
    }
    if !env_fresh {
        img::save_png(&environment(&faces), &env_out, false, true)?;
        img::write_file(&env_fp_path, env_fp.as_bytes())?;
    }
    let env_kb = std::fs::metadata(&env_out).map_or(0, |m| m.len() / 1024);
    rows.push(format!(
        "| `studio_env.png` | environment, octahedral (z at the viewer in the centre), sRGB, white = radiance {ENV_RANGE} | {s}x{s} | {env_kb} KB | blender:liquid_glass::STUDIO (Cycles, 6 faces {FACE_SIZE} px, fov {FACE_FOV_DEG}°, 24 samples, seed={BLENDER_SEED}): horizon ring 0..{RING_ELEV_DEG}° (radiance {RING_BASE} to 5.9 towards the key), key strip 74x9° radiance 8, strip 105x7° radiance 4.7, warm kicker, cyclorama with the ComfyUI studio plate at radiance {PLATE_RADIANCE}; resampled to octahedral, cleared within {}° of the viewer | CreativeML Open RAIL++-M output (SDXL plate); MIT (asset-forge) | what every glass piece reflects (`OpticalGlass` environment, gain {ENV_RANGE}). Masters `studio_px.exr` .. `studio_nz.exr` |",
        VIEWER_CLEAR.0,
        s = ENV_SIZE
    ));

    // The cluster's own ledger.
    let mut md = String::from(
        "# Liquid Glass assets\n\nBuilt by `cargo run --release -p asset-forge -- build --set liquid_glass` (recipe `tools/asset-forge/src/recipes/liquid_glass.rs`). The glass is computed by the kit (`docs/specs/optics.md`); these files are what it reflects and bends. The cluster uses the kit's analytic studio and a plain light when a file is missing.\n\n## Runtime files (in git)\n\n| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |\n|---|---|---|---|---|---|---|\n",
    );
    for r in rows {
        let _ = writeln!(md, "{r}");
    }
    md.push_str("\n## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/liquid_glass/`)\n\n| File | What it is |\n|---|---|\n");
    for r in master_rows {
        let _ = writeln!(md, "{r}");
    }
    let ledger = ctx.abs(&format!("{DIR}/ASSETS.md"));
    img::write_file(&ledger, md.as_bytes())?;
    println!("  wrote {DIR}/ (2 files + ASSETS.md)");
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
        "liquid_light_master.png" => {
            "the ray-traced caustic as 16-bit linear irradiance (white = 16 x the floor under still water)"
        }
        "studio_plate_master.png" => "the ComfyUI studio panorama on the Blender cyclorama",
        _ => "one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance",
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

/// One wave of the rippled water surface: whole cycles per tile along x and y, height
/// amplitude (in tile widths) and phase.
#[derive(Debug, Clone, Copy)]
struct Wave {
    kx: f32,
    ky: f32,
    amp: f32,
    phase: f32,
}

/// The surface: [`RIPPLE_WAVES`] waves with wave numbers spread evenly in the logarithm over
/// [`RIPPLE_BAND`] and in direction, snapped to whole cycles per tile (so the surface tiles),
/// with heights scaled to [`RIPPLE_SLOPE`]: the mean of |grad h|² is the sum of (a 2π k)² / 2.
fn ripples(seed: u64) -> Vec<Wave> {
    let mut rng = Rng::new(seed);
    let (k0, k1) = RIPPLE_BAND;
    let mut waves = Vec::with_capacity(RIPPLE_WAVES);
    while waves.len() < RIPPLE_WAVES {
        let k = k0 * (k1 / k0).powf(rng.f32());
        let dir = rng.f32() * TAU;
        let (kx, ky) = ((k * dir.cos()).round(), (k * dir.sin()).round());
        let kk = (kx * kx + ky * ky).sqrt();
        if kk < 1.0 {
            continue;
        }
        waves.push(Wave {
            kx,
            ky,
            amp: kk.powf(-RIPPLE_FALLOFF),
            phase: rng.f32() * TAU,
        });
    }
    let mean_sq: f32 = waves
        .iter()
        .map(|w| (w.amp * TAU).powi(2) * (w.kx * w.kx + w.ky * w.ky) * 0.5)
        .sum();
    let gain = RIPPLE_SLOPE / mean_sq.sqrt().max(1e-9);
    for w in &mut waves {
        w.amp *= gain;
    }
    waves
}

/// Depth of the floor (tile widths). A surface of curvature κ between air and water focuses
/// vertical light at 1 / ((1 − 1/n) κ) below it; κ is the RMS mean curvature of the ripples,
/// half the RMS of the Laplacian: the square root of the sum of a² (2π k)⁴ / 2, halved.
fn pool_depth(waves: &[Wave]) -> f32 {
    let lap_sq: f32 = waves
        .iter()
        .map(|w| w.amp * w.amp * (TAU * TAU * (w.kx * w.kx + w.ky * w.ky)).powi(2) * 0.5)
        .sum();
    let curvature = 0.5 * lap_sq.sqrt();
    POOL_FOCUS / ((1.0 - 1.0 / WATER_IOR) * curvature.max(1e-9))
}

/// Reflectance of the air-to-water surface for unpolarised light (Fresnel's equations).
fn water_reflectance(cos_i: f32, cos_t: f32) -> f32 {
    let n = WATER_IOR;
    let rs = (cos_i - n * cos_t) / (cos_i + n * cos_t);
    let rp = (n * cos_i - cos_t) / (n * cos_i + cos_t);
    0.5 * (rs * rs + rp * rp)
}

/// The caustic on the floor of the pool under a sun at the zenith, as irradiance relative to
/// the floor under still water (1 channel, linear, mean close to 1). `rays`² vertical rays
/// per texel are refracted at the surface by Snell's law, weighted by the surface's Fresnel
/// transmission and splatted bilinearly where they land, with wrap-around: the map tiles
/// exactly, and light is conserved.
fn caustic(size: usize, rays: usize, seed: u64) -> Img {
    let waves = ripples(seed);
    let depth = pool_depth(&waves);
    let nw = waves.len();
    let total = size * rays;
    let eta = 1.0 / WATER_IOR;
    // Amplitude times cos and sin of each wave's phase along x, and cos and sin along y, at the
    // ray positions: every wave is then a product of an x entry and a y entry.
    let mut tab_x = vec![0.0_f32; total * nw * 2];
    let mut tab_y = vec![0.0_f32; total * nw * 2];
    for i in 0..total {
        let s = (i as f32 + 0.5) / total as f32;
        for (w, wave) in waves.iter().enumerate() {
            let (sx, cx) = (TAU * wave.kx * s + wave.phase).sin_cos();
            let (sy, cy) = (TAU * wave.ky * s).sin_cos();
            tab_x[(i * nw + w) * 2] = wave.amp * cx;
            tab_x[(i * nw + w) * 2 + 1] = wave.amp * sx;
            tab_y[(i * nw + w) * 2] = cy;
            tab_y[(i * nw + w) * 2 + 1] = sy;
        }
    }
    let still = 1.0 - water_reflectance(1.0, 1.0);
    let weight = 1.0 / ((rays * rays) as f32 * still);
    let mut acc = vec![0.0_f32; size * size];
    let n = size as i64;
    for j in 0..total {
        let ty = &tab_y[j * nw * 2..(j + 1) * nw * 2];
        let v = (j as f32 + 0.5) / total as f32;
        for i in 0..total {
            let tx = &tab_x[i * nw * 2..(i + 1) * nw * 2];
            let (mut h, mut gx, mut gy) = (0.0_f32, 0.0_f32, 0.0_f32);
            for (w, wave) in waves.iter().enumerate() {
                let (ac, as_) = (tx[2 * w], tx[2 * w + 1]);
                let (cy, sy) = (ty[2 * w], ty[2 * w + 1]);
                h += ac * cy - as_ * sy;
                let d = -TAU * (as_ * cy + ac * sy);
                gx += d * wave.kx;
                gy += d * wave.ky;
            }
            // The surface normal, and the refracted direction of a ray that travels straight
            // down: t = η d + (η cos θi − cos θt) n with d = (0, 0, −1).
            let inv = 1.0 / (gx * gx + gy * gy + 1.0).sqrt();
            let cos_i = inv;
            let cos_t = (1.0 - eta * eta * (1.0 - cos_i * cos_i)).sqrt();
            let bend = eta * cos_i - cos_t;
            let tz = -eta + bend * inv;
            let run = (depth + h) / -tz;
            let u = (i as f32 + 0.5) / total as f32 + bend * -gx * inv * run;
            let w_ = v + bend * -gy * inv * run;
            let flux = (1.0 - water_reflectance(cos_i, cos_t)) * weight;
            let (x, y) = (u * size as f32 - 0.5, w_ * size as f32 - 0.5);
            let (x0, y0) = (x.floor(), y.floor());
            let (fx, fy) = (x - x0, y - y0);
            let (x0, y0) = (x0 as i64, y0 as i64);
            let mut add = |xx: i64, yy: i64, k: f32| {
                acc[(yy.rem_euclid(n) * n + xx.rem_euclid(n)) as usize] += flux * k;
            };
            add(x0, y0, (1.0 - fx) * (1.0 - fy));
            add(x0 + 1, y0, fx * (1.0 - fy));
            add(x0, y0 + 1, (1.0 - fx) * fy);
            add(x0 + 1, y0 + 1, fx * fy);
        }
    }
    Img {
        w: size,
        h: size,
        c: 1,
        data: acc,
    }
}

/// Irradiance as a stored value: divided by `white`, clamped, sRGB-encoded for the runtime
/// tile (`srgb`) or left linear for the 16-bit master.
fn encode_light(irr: &Img, white: f32, srgb: bool) -> Img {
    let mut out = irr.clone();
    for v in &mut out.data {
        let x = (*v / white).clamp(0.0, 1.0);
        *v = if srgb { img::srgb_encode(x) } else { x };
    }
    out
}

/// Minimum, maximum and mean.
fn stats(values: &[f32]) -> (f32, f32, f32) {
    let mut lo = f32::MAX;
    let mut hi = f32::MIN;
    let mut sum = 0.0_f64;
    for &v in values {
        lo = lo.min(v);
        hi = hi.max(v);
        sum += f64::from(v);
    }
    (lo, hi, (sum / values.len().max(1) as f64) as f32)
}

/// Generates the studio panorama: txt2img near 1 MP, roll by half the width, repaint the seam
/// column, 4x upscale, resize to [`PLATE_SIZE`].
fn studio_plate(ctx: &Ctx, scratch: &Path) -> Result<PathBuf> {
    let out = scratch.join("studio_plate.png");
    let mut job = Job::new(PLATE_PROMPT, PLATE_NEGATIVE, PLATE_SEED);
    job.size = (1408, 704);
    job.steps = 32;
    job.prefix = "fh6_liquid_glass_plate".into();
    let graph = serde_json::to_string(&sdxl::build(&job)).unwrap_or_default();
    let fp = fingerprint(&[graph.as_bytes(), b"x-seam v1"]);
    let fp_path = out.with_extension("fp");
    if out.is_file() && std::fs::read_to_string(&fp_path).is_ok_and(|s| s == fp) {
        println!("  = comfyui studio plate (fresh)");
        return Ok(out);
    }
    println!("  comfyui studio plate (SDXL, {}x{})", PLATE_SIZE.0, PLATE_SIZE.1);
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
    let up = c.upload_image("fh6_lg_plate_rolled.png", &img::encode_png(&rolled, false, true)?)?;
    let mk = c.upload_image("fh6_lg_plate_seam.png", &img::encode_png(&mask, false, false)?)?;
    let mut seam = job.clone();
    seam.init = Some((up, 0.55));
    seam.mask = Some(mk);
    seam.upscale = Some(crate::paths::UPSCALER.into());
    seam.prefix = "fh6_liquid_glass_plate_seam".into();
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

/// Resamples the cube faces into the octahedral map (2x2 supersampled), clears the cone round
/// the viewer and stores sRGB-encoded radiance / [`ENV_RANGE`].
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
            img::srgb_encode(acc[0] / ENV_RANGE),
            img::srgb_encode(acc[1] / ENV_RANGE),
            img::srgb_encode(acc[2] / ENV_RANGE),
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
            assert!(
                (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v),
                "{d:?} -> {u}, {v}"
            );
            let back = oct_dir(u, v);
            let err = (0..3).map(|i| (back[i] - d[i]).abs()).fold(0.0, f32::max);
            assert!(err < 1e-4, "{d:?} -> ({u}, {v}) -> {back:?}");
        }
        // The viewer direction is the centre of the map.
        assert_eq!(oct_uv([0.0, 0.0, 1.0]), (0.5, 0.5));
    }

    #[test]
    fn every_direction_finds_its_cube_face() {
        // A face that is 1 on one cube face and 0 elsewhere: sampling along that face's forward
        // direction must read it, and the tangent axes must point the way the camera matrix does.
        for (i, (_, f, up)) in FACES.iter().enumerate() {
            let faces: Vec<Img> = (0..6)
                .map(|j| {
                    Img::from_fn(8, 8, 3, |x, y| {
                        if i == j {
                            [1.0, x as f32 / 7.0, 1.0 - y as f32 / 7.0, 1.0]
                        } else {
                            [0.0; 4]
                        }
                    })
                })
                .collect();
            // Blender frame to the kit frame.
            let kit = |b: [f32; 3]| [b[0], -b[1], b[2]];
            let centre = sample_cube(&faces, kit(*f));
            assert!(centre[0] > 0.99, "face {i} centre");
            let right = cross(*f, *up);
            let toward = |a: [f32; 3], k: f32| {
                let v = [f[0] + a[0] * k, f[1] + a[1] * k, f[2] + a[2] * k];
                let l = dot(v, v).sqrt();
                kit([v[0] / l, v[1] / l, v[2] / l])
            };
            assert!(
                sample_cube(&faces, toward(right, 0.5))[1] > centre[1],
                "face {i}: +right is +x in the image"
            );
            assert!(
                sample_cube(&faces, toward(*up, 0.5))[2] > centre[2],
                "face {i}: +up is the top of the image"
            );
        }
    }

    #[test]
    fn panels_are_orthonormal_frames() {
        for p in &PANELS {
            for (a, b) in [(p.c, p.tx), (p.c, p.ty), (p.tx, p.ty)] {
                assert!(dot(a, b).abs() < 2e-3, "{}: axes are perpendicular", p.name);
            }
            for a in [p.c, p.tx, p.ty] {
                assert!((dot(a, a) - 1.0).abs() < 2e-3, "{}: unit axis", p.name);
            }
            // Away from the viewer's clear cone, and in front of the screen.
            assert!(p.c[2] > 0.0 && p.c[2] < VIEWER_CLEAR.1.to_radians().cos(), "{}", p.name);
        }
    }

    #[test]
    fn the_caustic_conserves_light_and_tiles() {
        let irr = caustic(256, 2, LIGHT_SEED);
        assert_eq!((irr.w, irr.h, irr.c), (256, 256, 1));
        let (lo, hi, mean) = stats(&irr.data);
        // Every ray lands somewhere on the tile. Ripples reflect a little more than still water.
        assert!((0.97..=1.0).contains(&mean), "mean irradiance {mean}");
        // A caustic, not a flat field: dark water between bright filaments.
        assert!(lo < 0.7 && hi > 3.0, "range {lo}..{hi}");
        // The wrap edges are as smooth as the inside.
        assert!(img::tile_seam_score(&irr) < 1.6, "seam {}", img::tile_seam_score(&irr));
        // Deterministic.
        assert_eq!(irr, caustic(256, 2, LIGHT_SEED));
    }

    #[test]
    fn the_ripples_have_the_stated_slope_and_whole_cycles() {
        let waves = ripples(LIGHT_SEED);
        assert_eq!(waves.len(), RIPPLE_WAVES);
        let mean_sq: f32 = waves
            .iter()
            .map(|w| (w.amp * TAU).powi(2) * (w.kx * w.kx + w.ky * w.ky) * 0.5)
            .sum();
        assert!((mean_sq.sqrt() - RIPPLE_SLOPE).abs() < 1e-4);
        assert!(waves.iter().all(|w| w.kx.fract() == 0.0 && w.ky.fract() == 0.0));
        // Still water reflects ((n - 1) / (n + 1))² = 2.04%.
        assert!((water_reflectance(1.0, 1.0) - 0.0204).abs() < 2e-4);
        assert!(pool_depth(&waves) > 0.0);
    }

    #[test]
    fn light_encoding_clamps_at_white() {
        let irr = Img {
            w: 3,
            h: 1,
            c: 1,
            data: vec![0.0, LIGHT_RANGE * 0.5, LIGHT_RANGE * 3.0],
        };
        let lin = encode_light(&irr, LIGHT_RANGE, false);
        assert_eq!(lin.data, vec![0.0, 0.5, 1.0]);
        let enc = encode_light(&irr, LIGHT_RANGE, true);
        assert!((enc.data[1] - img::srgb_encode(0.5)).abs() < 1e-6 && (enc.data[2] - 1.0).abs() < 1e-6);
    }
}
