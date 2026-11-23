//! TeslaAdas bespoke assets (`assets/clusters/tesla_adas/`), set `tesla_adas`:
//!
//! - **Underglow light pools** (Blender, Cycles): the Phase 6 ego coupe on a white ground with a
//!   downward area light under its floor, rendered through the cluster's own chase cameras
//!   (cockpit and hood: the same eye, pitch and focal length as `tesla_adas::COCKPIT` / `HOOD`).
//!   The car is invisible to the camera but still blocks light, so the pool carries the real
//!   tyre and sill shadows. Post: luminance → alpha (white RGB, tinted live by the palette).
//! - **Horizon haze plate** (ComfyUI: SDXL base 1.0 + 4x-UltraSharp): a thin band of atmospheric
//!   glow along a flat horizon, drawn behind the far road. Post: luminance → alpha, side and
//!   vertical dissolves baked in.
//!
//! Both keep a procedural fallback in the cluster (`setup.assets` may be `None`).
use crate::comfy::{self, Comfy};
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{tool_err, Result};
use crate::img::{self, Img};
use crate::ledger::fingerprint;
use crate::{blender, recipes};
use serde_json::{json, Map};
use std::time::Duration;

const CLUSTERS: &str = "TeslaAdas";
const SET: &str = "tesla_adas";

/// The underglow pool sprite's size in local 4K px (it is rendered at 2x).
pub const POOL_LOCAL: [usize; 2] = [480, 240];

/// bpy: the ego on a white ground lit only by an area light under its floor, seen from a chase
/// camera that reproduces `fh6_kit::perspective::Camera25` (eye, pitch, focal px), shifted so the
/// image centre is the ego's ground centre. Writes `pool_<view>.exr` (scene-linear) per view.
pub const POOL: &str = r#"
import math
P = ARGS
for v in P['views']:
    sc = reset()
    gpu_cycles(sc, samples=int(P['samples']), denoise=True, bounces=6)
    sc.render.film_transparent = True
    w = bpy.data.worlds.new('void')
    sc.world = w
    w.use_nodes = True
    bg = w.node_tree.nodes.get('Background')
    if bg is not None:
        bg.inputs['Strength'].default_value = 0.0
    ego_z = float(P['ego_z'])
    # glTF +Z (nose) imports as Blender -Y; kit depth z is Blender -y.
    bpy.ops.import_scene.gltf(filepath=P['glb'])
    for o in list(sc.objects):
        if o.parent is None and o.type in ('MESH', 'EMPTY'):
            o.location.y -= ego_z
    for o in sc.objects:
        if o.type == 'MESH':
            o.visible_camera = False
            o.visible_glossy = False
    # The ground: a matte white plane.
    bpy.ops.mesh.primitive_plane_add(size=80.0, location=(0.0, -ego_z, 0.0))
    ground = bpy.context.active_object
    m, nt = new_mat('ground')
    diffuse = nt.nodes.new('ShaderNodeBsdfDiffuse')
    diffuse.inputs['Color'].default_value = (0.8, 0.8, 0.8, 1.0)
    mout = nt.nodes.new('ShaderNodeOutputMaterial')
    nt.links.new(diffuse.outputs['BSDF'], mout.inputs['Surface'])
    ground.data.materials.append(m)
    # The underglow: a downward rectangle under the floor, inside the wheelbase and track.
    ld = bpy.data.lights.new('underglow', 'AREA')
    ld.shape = 'RECTANGLE'
    ld.size = float(P['light_w'])
    ld.size_y = float(P['light_l'])
    ld.energy = float(P['energy'])
    lo = bpy.data.objects.new('underglow', ld)
    lo.location = (0.0, -ego_z, float(P['light_h']))
    try:
        lo.visible_camera = False
    except Exception:
        pass
    link(lo)
    # The chase camera.
    ex, ey, ez = v['eye']
    pitch = math.radians(float(v['pitch_deg']))
    f = float(v['focal'])
    lw, lh = P['local']
    k = float(P['scale'])
    rx, ry = int(lw * k), int(lh * k)
    resolution(sc, rx, ry)
    view(sc, 'Standard')
    dy, dz = -ey, ego_z - ez
    s, c = math.sin(pitch), math.cos(pitch)
    zc = dz * c - dy * s
    yc = dy * c + dz * s
    below = -f * yc / zc
    cd = bpy.data.cameras.new('chase')
    cd.sensor_fit = 'HORIZONTAL'
    cd.sensor_width = 36.0
    cd.lens = f * k * 36.0 / rx
    cd.shift_x = 0.0
    cd.shift_y = -(below * k) / max(rx, ry)
    cd.clip_start = 0.05
    cd.clip_end = 400.0
    cam = bpy.data.objects.new('chase', cd)
    cam.location = (ex, -ez, ey)
    cam.rotation_euler = (math.radians(90.0) - pitch, 0.0, math.pi)
    link(cam)
    sc.camera = cam
    save(sc, os.path.join(OUT, 'pool_%s.exr' % v['name']), exr=True)
    log('pool', v['name'], 'below', below)
done()
"#;

/// The ComfyUI horizon plate.
const HAZE_PROMPT: &str = "wide panoramic night photograph of a flat empty plain, a thin band of soft glowing cold blue atmospheric haze along a perfectly straight distant horizon line, pure black sky above, pure black ground below, minimal, calm, cinematic";
const HAZE_NEGATIVE: &str = "text, letters, watermark, logo, signature, frame, border, people, car, road, buildings, trees, mountains, moon, stars, sun, lights, city, user interface, lowres, blurry, jpeg artifacts, oversaturated";
const HAZE_SEED: u64 = 1505;
/// The shipped plate size (the cluster draws it about 1440 x 220 local px).
const HAZE_SIZE: [usize; 2] = [1536, 384];

/// Builds the set.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    pools(ctx)?;
    haze(ctx)
}
/// Luminance → alpha over white, normalised to the 99.7th percentile, softened by a
/// down-and-up resample (`soften` = the reduction factor; 1 keeps it sharp), shaped by `gamma`
/// (< 1 lifts a light's falloff tail), with feathered edges.
fn to_light_mask(src: &Img, feather: [f32; 2], gamma: f32, soften: usize) -> Img {
    let mut lum = Img::from_fn(src.w, src.h, 1, |x, y| {
        let a = if src.c >= 4 {
            src.get(x, y, 3).clamp(0.0, 1.0)
        } else {
            1.0
        };
        [
            img::luma(src.get(x, y, 0), src.get(x, y, 1), src.get(x, y, 2)) * a,
            0.0,
            0.0,
            0.0,
        ]
    });
    if soften > 1 {
        let small = lum.resize((src.w / soften).max(2), (src.h / soften).max(2), false);
        lum = small.resize(src.w, src.h, false);
    }
    let mut sorted = lum.data.clone();
    sorted.sort_by(f32::total_cmp);
    let peak = sorted[((sorted.len() as f32 * 0.997) as usize).min(sorted.len() - 1)].max(1e-6);
    Img::from_fn(src.w, src.h, 4, |x, y| {
        let u = x as f32 / (src.w - 1).max(1) as f32;
        let v = y as f32 / (src.h - 1).max(1) as f32;
        let fx = img::smoothstep(0.0, feather[0], u) * img::smoothstep(0.0, feather[0], 1.0 - u);
        let fy = img::smoothstep(0.0, feather[1], v) * img::smoothstep(0.0, feather[1], 1.0 - v);
        let l = (lum.get(x, y, 0) / peak).clamp(0.0, 1.0);
        [1.0, 1.0, 1.0, l.powf(gamma) * fx * fy]
    })
}
fn pools(ctx: &mut Ctx) -> Result<()> {
    let glb = ctx.abs("assets/adas/ego_coupe.glb");
    let params = json!({
        "views": [
            {"name": "cockpit", "eye": [0.0, 3.4, -6.0], "pitch_deg": 8.0, "focal": 440.0},
            {"name": "hood", "eye": [0.0, 2.8, -6.5], "pitch_deg": 8.0, "focal": 400.0}
        ],
        "ego_z": 3.2,
        "local": POOL_LOCAL,
        "scale": 2.0,
        "light_w": 1.9,
        "light_l": 4.0,
        "light_h": 0.16,
        "energy": 520.0,
        "samples": 256,
        "seed": 151
    });
    let glb_bytes = crate::ctx::read(&glb)?;
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        POOL.as_bytes(),
        params.to_string().as_bytes(),
        &glb_bytes,
        b"pool-post-v3",
    ]);
    ctx.job("tesla_adas/underglow_pools", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_pools")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        p["glb"] = json!(glb.to_string_lossy());
        blender::run(&ctx.paths, "tesla_adas_pools", POOL, &p)?;
        for view in ["cockpit", "hood"] {
            let exr = img::load_exr(&out.join(format!("pool_{view}.exr")))?;
            let mask = to_light_mask(&exr, [0.12, 0.16], 0.7, 8);
            let gen = format!(
                "blender:tesla_adas::POOL {view} chase camera (Cycles 256 spp, OptiX denoise; ego_coupe.glb camera-invisible, area light 1.9 x 4.0 m at 0.16 m, 520 W; mask softened 8x, gamma 0.7) seed=151"
            );
            ctx.image(
                &format!("tesla_adas/underglow_pools/{view}"),
                &format!("assets/clusters/tesla_adas/underglow_pool_{view}.png"),
                &mask,
                Format::Png8,
                Spec::new(SET, "mask", gen, CLUSTERS)
                    .alpha("straight")
                    .space("linear")
                    .qa(format!(
                        "white RGB, alpha = ground light (99.7th percentile = 1), edges feathered; {} x {} local px centred on the ego's ground centre",
                        POOL_LOCAL[0], POOL_LOCAL[1]
                    )),
            )?;
        }
        Ok(())
    })
}

fn haze(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        HAZE_PROMPT.as_bytes(),
        HAZE_NEGATIVE.as_bytes(),
        &HAZE_SEED.to_le_bytes(),
        b"1536x640",
    ]);
    let fp = fingerprint(&[gen_fp.as_bytes(), b"haze-post-v2"]);
    ctx.job("tesla_adas/horizon_haze", &fp, |ctx| {
        let raw_path = ctx.scratch("comfy")?.join("tesla_adas_horizon_haze.png");
        let raw_fp_path = raw_path.with_extension("fp");
        let cached = raw_path.exists() && std::fs::read_to_string(&raw_fp_path).map(|s| s == gen_fp).unwrap_or(false);
        if !cached {
            let c = Comfy::connect(&ctx.paths)?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(HAZE_PROMPT));
            params.insert("negative".into(), json!(HAZE_NEGATIVE));
            params.insert("seed".into(), json!(HAZE_SEED));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1536));
            params.insert("height".into(), json!(640));
            params.insert("prefix".into(), json!("fh6_tesla_adas_horizon_haze"));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
        }
        // 6144 x 2560 from the upscaler: keep only the sky band from 18% of the height down to the
        // horizon line at 70% (the plain below it is bright and would read as the light), so the
        // plate's bottom edge is the horizon.
        let raw = img::load(&raw_path)?;
        let (top, bottom) = (raw.h * 18 / 100, raw.h * 70 / 100);
        let band = raw.crop(0, top, raw.w, bottom - top);
        let small = band.resize(HAZE_SIZE[0], HAZE_SIZE[1], false);
        let mut rgb = small.to_channels(3);
        img::crush_blacks(&mut rgb, 0.03);
        let mask = to_light_mask(&rgb, [0.18, 0.30], 1.0, 1);
        ctx.image(
            "tesla_adas/horizon_haze",
            "assets/clusters/tesla_adas/horizon_haze.png",
            &mask,
            Format::Png8,
            Spec::new(
                SET,
                "mask",
                format!(
                    "comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5, 1536 x 640) seed={HAZE_SEED} prompt=\"{HAZE_PROMPT}\""
                ),
                CLUSTERS,
            )
            .alpha("straight")
            .space("linear")
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa("sky band 18–70% of the 6144 x 2560 upscale (bottom edge = horizon) → 1536 x 384; blacks below 3% crushed; luminance → alpha over white (tinted live); sides and top/bottom feathered"),
        )
    })
}
