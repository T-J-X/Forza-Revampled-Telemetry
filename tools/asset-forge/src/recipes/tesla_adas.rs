//! TeslaAdas bespoke assets (`assets/clusters/tesla_adas/`), set `tesla_adas`:
//!
//! - **Underglow light-pool atlases** (Blender, Cycles): the Phase 6 ego coupe on a white ground
//!   with a downward area light under its floor, rendered through the cluster's own chase cameras
//!   (cockpit and hood: the same eye, pitch and focal length as `tesla_adas::COCKPIT` / `HOOD`) at
//!   11 yaw angles from −50° to +50°. The car is invisible to the camera but still blocks light,
//!   so each pool carries the real tyre and sill shadows in true perspective for that yaw. Post:
//!   luminance → alpha over white with a hard black point, softened; 4 × 3 atlas cells.
//! - **Asphalt plate** (ComfyUI: SDXL base 1.0 + 4x-UltraSharp): a dark road surface receding to a
//!   central vanishing point, used as the road fill between the edge lines. Post: the part below
//!   the horizon, luminance → alpha over white, a near-to-far fade baked in.
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

/// One pool cell's size in local 4K px (rendered at [`POOL_SCALE`]x).
pub const POOL_LOCAL: [usize; 2] = [480, 240];
/// Render scale of a pool cell.
const POOL_SCALE: f32 = 1.5;
/// The yaw angles of the atlas cells (degrees, positive = nose to the right), row-major in a
/// [`POOL_COLS`]-wide grid.
pub const POOL_YAWS: [f32; 11] = [-50.0, -40.0, -30.0, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0, 40.0, 50.0];
/// Atlas columns (3 rows hold the 11 cells).
pub const POOL_COLS: usize = 4;

/// bpy: per view and yaw, the ego (yawed about its centre, light included) on a white ground lit
/// only by an area light under its floor, seen from a chase camera that reproduces
/// `fh6_kit::perspective::Camera25` (eye, pitch, focal px), shifted so the image centre is the
/// ego's ground centre. Writes `pool_<view>_<i>.exr` (scene-linear).
pub const POOL: &str = r#"
import math
P = ARGS
for v in P['views']:
    for i, yaw_deg in enumerate(P['yaws']):
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
        yaw = math.radians(float(yaw_deg))
        # glTF +Z (nose) imports as Blender -Y; kit depth z is Blender -y; a positive kit heading
        # turns the nose towards +x, a positive rotation about Blender +Z.
        bpy.ops.import_scene.gltf(filepath=P['glb'])
        for o in list(sc.objects):
            if o.parent is None and o.type in ('MESH', 'EMPTY'):
                o.location.y -= ego_z
                o.rotation_euler.z += yaw
        for o in sc.objects:
            if o.type == 'MESH':
                o.visible_camera = False
                o.visible_glossy = False
        bpy.ops.mesh.primitive_plane_add(size=80.0, location=(0.0, -ego_z, 0.0))
        ground = bpy.context.active_object
        m, nt = new_mat('ground')
        diffuse = nt.nodes.new('ShaderNodeBsdfDiffuse')
        diffuse.inputs['Color'].default_value = (0.8, 0.8, 0.8, 1.0)
        mout = nt.nodes.new('ShaderNodeOutputMaterial')
        nt.links.new(diffuse.outputs['BSDF'], mout.inputs['Surface'])
        ground.data.materials.append(m)
        ld = bpy.data.lights.new('underglow', 'AREA')
        ld.shape = 'RECTANGLE'
        ld.size = float(P['light_w'])
        ld.size_y = float(P['light_l'])
        ld.energy = float(P['energy'])
        lo = bpy.data.objects.new('underglow', ld)
        lo.location = (0.0, -ego_z, float(P['light_h']))
        lo.rotation_euler = (0.0, 0.0, yaw)
        try:
            lo.visible_camera = False
        except Exception:
            pass
        link(lo)
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
        save(sc, os.path.join(OUT, 'pool_%s_%d.exr' % (v['name'], i)), exr=True)
done()
"#;

const ASPHALT_PROMPT: &str = "photograph of an empty dark asphalt road at night seen from a low camera behind a car, the road surface fills the frame and recedes straight to a vanishing point at the top centre, fine aggregate texture, faint cold blue sheen, moody, minimal";
const ASPHALT_NEGATIVE: &str = "lane markings, white lines, yellow lines, arrows, text, letters, watermark, logo, car, vehicle, people, buildings, trees, street lights, sky, sun, moon, lowres, blurry, jpeg artifacts, oversaturated";
const ASPHALT_SEED: u64 = 1507;
/// The shipped plate size (the cluster maps it over the road from the horizon to the band's
/// bottom).
const ASPHALT_SIZE: [usize; 2] = [1536, 512];

/// Builds the set.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    pools(ctx)?;
    asphalt(ctx)
}

/// Luminance → alpha over white: softened by a down-and-up resample (`soften` = the reduction
/// factor; 1 keeps it sharp), normalised to the 99.7th percentile, everything at or below
/// `black` (after normalising) set to exactly 0, shaped by `gamma` (< 1 lifts a light's falloff
/// tail), and multiplied by `shape(u, v)` (feathers and fades; `u`, `v` in 0..1).
fn to_light_mask(src: &Img, soften: usize, black: f32, gamma: f32, shape: impl Fn(f32, f32) -> f32) -> Img {
    let mut lum = Img::from_fn(src.w, src.h, 1, |x, y| {
        let a = if src.c >= 4 { src.get(x, y, 3).clamp(0.0, 1.0) } else { 1.0 };
        [img::luma(src.get(x, y, 0), src.get(x, y, 1), src.get(x, y, 2)) * a, 0.0, 0.0, 0.0]
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
        let l = (lum.get(x, y, 0) / peak).clamp(0.0, 1.0);
        let l = ((l - black) / (1.0 - black)).max(0.0);
        [1.0, 1.0, 1.0, l.powf(gamma) * shape(u, v)]
    })
}

/// Feathered edges: `fx` / `fy` of the width / height fade in from each side.
fn feather(fx: f32, fy: f32) -> impl Fn(f32, f32) -> f32 {
    move |u, v| {
        img::smoothstep(0.0, fx, u)
            * img::smoothstep(0.0, fx, 1.0 - u)
            * img::smoothstep(0.0, fy, v)
            * img::smoothstep(0.0, fy, 1.0 - v)
    }
}

fn pools(ctx: &mut Ctx) -> Result<()> {
    let glb = ctx.abs("assets/adas/ego_coupe.glb");
    let params = json!({
        "views": [
            {"name": "cockpit", "eye": [0.0, 3.4, -6.0], "pitch_deg": 8.0, "focal": 440.0},
            {"name": "hood", "eye": [0.0, 2.8, -6.5], "pitch_deg": 8.0, "focal": 400.0}
        ],
        "yaws": POOL_YAWS,
        "ego_z": 3.2,
        "local": POOL_LOCAL,
        "scale": POOL_SCALE,
        "light_w": 1.9,
        "light_l": 4.0,
        "light_h": 0.16,
        "energy": 520.0,
        "samples": 192,
        "seed": 151
    });
    let glb_bytes = crate::ctx::read(&glb)?;
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        POOL.as_bytes(),
        params.to_string().as_bytes(),
        &glb_bytes,
        b"pool-atlas-v1",
    ]);
    ctx.job("tesla_adas/underglow_pools", &fp, |ctx| {
        let out = ctx.scratch("tesla_adas_pools")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        p["glb"] = json!(glb.to_string_lossy());
        blender::run(&ctx.paths, "tesla_adas_pools", POOL, &p)?;
        let (cw, ch) = (
            (POOL_LOCAL[0] as f32 * POOL_SCALE) as usize,
            (POOL_LOCAL[1] as f32 * POOL_SCALE) as usize,
        );
        let rows = POOL_YAWS.len().div_ceil(POOL_COLS);
        for view in ["cockpit", "hood"] {
            let mut atlas = Img::new(cw * POOL_COLS, ch * rows, 4);
            for i in 0..POOL_YAWS.len() {
                let exr = img::load_exr(&out.join(format!("pool_{view}_{i}.exr")))?;
                let cell = to_light_mask(&exr, 10, 0.04, 0.7, feather(0.12, 0.16));
                atlas.paste(&cell, (i % POOL_COLS) * cw, (i / POOL_COLS) * ch);
            }
            let gen = format!(
                "blender:tesla_adas::POOL {view} chase camera at yaw -50..50 step 10 (Cycles 192 spp, OptiX denoise; ego_coupe.glb camera-invisible, area light 1.9 x 4.0 m at 0.16 m, 520 W) seed=151"
            );
            ctx.image(
                &format!("tesla_adas/underglow_pools/{view}"),
                &format!("assets/clusters/tesla_adas/underglow_pool_{view}.png"),
                &atlas,
                Format::Png8,
                Spec::new(SET, "mask", gen, CLUSTERS)
                    .alpha("straight")
                    .space("linear")
                    .qa(format!(
                        "{POOL_COLS} x {rows} atlas of {cw} x {ch} cells (yaw -50..50 step 10, row-major); white RGB, alpha = ground light: softened 10x, 99.7th percentile = 1, black point 4% (true 0 below), gamma 0.7, edges feathered; each cell {} x {} local px centred on the ego's ground centre",
                        POOL_LOCAL[0], POOL_LOCAL[1]
                    )),
            )?;
        }
        Ok(())
    })
}

fn asphalt(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        ASPHALT_PROMPT.as_bytes(),
        ASPHALT_NEGATIVE.as_bytes(),
        &ASPHALT_SEED.to_le_bytes(),
        b"1536x640",
    ]);
    let fp = fingerprint(&[gen_fp.as_bytes(), b"asphalt-post-v2"]);
    ctx.job("tesla_adas/asphalt", &fp, |ctx| {
        let raw_path = ctx.scratch("comfy")?.join("tesla_adas_asphalt.png");
        let raw_fp_path = raw_path.with_extension("fp");
        let cached = raw_path.exists() && std::fs::read_to_string(&raw_fp_path).map(|s| s == gen_fp).unwrap_or(false);
        if !cached {
            let c = Comfy::connect(&ctx.paths)?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(ASPHALT_PROMPT));
            params.insert("negative".into(), json!(ASPHALT_NEGATIVE));
            params.insert("seed".into(), json!(ASPHALT_SEED));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1536));
            params.insert("height".into(), json!(640));
            params.insert("prefix".into(), json!("fh6_tesla_adas_asphalt"));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
        }
        // 6144 x 2560 from the upscaler: the road below the horizon (at about 47%). SDXL painted a
        // centre line despite the negative prompt, so keep the line-free left 46% and mirror it
        // into a symmetric plate (the cluster draws its own lane lines).
        let raw = img::load(&raw_path)?;
        let top = raw.h * 50 / 100;
        let half = raw.w * 46 / 100;
        let road = raw.crop(0, top, half, raw.h - top);
        let sym = Img::from_fn(half * 2, road.h, road.c, |x, y| {
            let sx = if x < half { x } else { 2 * half - 1 - x };
            let mut px = [0.0; 4];
            for (ch, v) in px.iter_mut().enumerate().take(road.c) {
                *v = road.get(sx, y, ch);
            }
            px
        });
        let small = sym.resize(ASPHALT_SIZE[0], ASPHALT_SIZE[1], false);
        // Texture only, the lighting comes from the cluster: near (bottom) full, far (top) 0.
        let mask = to_light_mask(&small, 1, 0.0, 1.0, |u, v| {
            img::smoothstep(0.0, 0.85, v) * img::smoothstep(0.0, 0.08, u) * img::smoothstep(0.0, 0.08, 1.0 - u)
        });
        ctx.image(
            "tesla_adas/asphalt",
            "assets/clusters/tesla_adas/asphalt.png",
            &mask,
            Format::Png8,
            Spec::new(
                SET,
                "mask",
                format!(
                    "comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5, 1536 x 640) seed={ASPHALT_SEED} prompt=\"{ASPHALT_PROMPT}\""
                ),
                CLUSTERS,
            )
            .alpha("straight")
            .space("linear")
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa("the line-free left 46% of the lower half of the 6144 x 2560 upscale, mirrored into a symmetric plate → 1536 x 512; luminance → alpha over white (tinted live to near-black), a far-to-near fade (0 at the top) and side feathers baked in"),
        )
    })
}
