//! GT7 Hood Pro (`ClusterTheme::Gt7HoodPro`) bespoke assets, `asset-forge build --set gt7_hood_pro`.
//!
//! - **Blender, `pod_lens.png`:** the gear pod as a sculpted OLED-black glass lens: a squircle
//!   slab (n 4, the pod's 232 × 184 proportion) with a 24-segment rolled bevel in a clear-coated
//!   black dielectric, lit by a long key strip from the upper left (the cluster's resting
//!   virtual light, azimuth −135°) and a cool rim strip from the lower right. Rendered top-down
//!   (orthographic, Cycles, transparent film) so the cluster lays it 1:1 over the pod: the bevel
//!   speculars and the faint sky reflection on the face are what make the pod read as glass
//!   hardware instead of a flat black shape.
//! - **ComfyUI, `ribbon_grain.png`:** a seamless, monochrome tile of fine horizontal light
//!   filaments (SDXL base 1.0 + 4x-UltraSharp, then blacks crushed for OLED, made seamless and
//!   reduced to luminance). The cluster scrolls it through the twin ribbons additively, tinted
//!   by the palette, so the light in the wings visibly flows towards the crest as the revs
//!   build.
//!
//! Both are optional at runtime: the cluster keeps its procedural pod and ribbons when
//! `assets/clusters/gt7_hood_pro/` is missing.
use crate::comfy::{self, Comfy};
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{tool_err, Result};
use crate::ledger::fingerprint;
use crate::{blender, img, recipes};
use serde_json::{json, Map};
use std::time::Duration;

/// Where the cluster finds its assets (repo-relative).
const DIR: &str = "assets/clusters/gt7_hood_pro";
const CLUSTERS: &str = "Gt7HoodPro";
/// sRGB level of the lens face taken back to black after the render.
const LENS_BLACK_FLOOR: f32 = 0.1;

/// The pod lens: a bevelled black-glass squircle, top-down. Params: `w`, `h` (units, 1 unit =
/// 100 local px), `thick`, `bevel`, `px_w`, `px_h`, `samples`, `seed`.
pub const POD_LENS: &str = r#"
sc = reset()
dev = gpu_cycles(sc, samples=int(ARGS['samples']), denoise=True, bounces=8)
view(sc, ARGS.get('view', 'AgX'), exposure=float(ARGS.get('exposure', 0.0)))
W, H = float(ARGS['w']), float(ARGS['h'])
pts = outline('squircle', W, H, n=4.0, segs=128)
ob = plate_mesh('pod', pts, float(ARGS['thick']), float(ARGS['bevel']), segs=24, profile=0.62)

m, nt = new_mat('oled_black_glass')
out = node(nt, 'ShaderNodeOutputMaterial')
bsdf = node(nt, 'ShaderNodeBsdfPrincipled')
bsdf.inputs['Base Color'].default_value = (0.003, 0.004, 0.006, 1.0)
bsdf.inputs['Roughness'].default_value = 0.16
bsdf.inputs['IOR'].default_value = 1.52
for name, value in (('Coat Weight', 0.7), ('Coat Roughness', 0.09), ('Coat IOR', 1.5)):
    try:
        bsdf.inputs[name].default_value = value
    except Exception as e:
        log('coat input', name, e)
lk(nt, bsdf.outputs[0], out.inputs['Surface'])
ob.data.materials.append(m)

# A dim studio world, a long key strip from the upper left (the cluster's resting light) and a
# cool, thinner rim strip from the lower right. No overhead light: the camera looks straight down
# at a glossy face, so anything above it would mirror as a hard shape; the satin coat lets the
# key graze the face as a soft gradient instead.
studio_world(sc, strength=float(ARGS.get('world', 0.25)), visible=False)
area_light('key', (-3.2, 3.0, 3.6), (0.0, 0.0, 0.0), 7.0, float(ARGS.get('key', 520.0)), size_y=0.5)
area_light('rim', (3.0, -2.6, 1.6), (0.0, 0.0, 0.0), 5.0, float(ARGS.get('rim', 160.0)), color=(0.62, 0.86, 1.0), size_y=0.25)

resolution(sc, ARGS['px_w'], ARGS['px_h'])
ortho_cam(sc, W * float(ARGS.get('margin', 1.04)), z=10.0, clip_end=40.0)
log('device', dev)
save(sc, os.path.join(OUT, 'pod_lens.png'), depth='16', mode='RGBA')
done()
"#;

/// The ComfyUI grain plate.
const GRAIN_PROMPT: &str = "seamless texture of fine horizontal luminous light filaments and streaks, anisotropic brushed light, thin parallel white and pale cyan lines of varying brightness, sparse sparkling grain, pure black background, macro photograph, crisp, high detail";
const GRAIN_NEGATIVE: &str = "text, letters, watermark, logo, signature, frame, border, vignette, people, face, car, user interface, lowres, blurry, jpeg artifacts, oversaturated, colour blotches, vertical lines, grid";
const GRAIN_SEED: u64 = 7301;
const GRAIN_SIZE: usize = 1024;

/// Builds both assets.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    pod_lens(ctx)?;
    ribbon_grain(ctx)
}

fn pod_lens(ctx: &mut Ctx) -> Result<()> {
    // 4x the hood pod (232 x 184 local px), so it stays crisp at the 1.4 user scale.
    let params = json!({
        "w": 2.32, "h": 1.84, "thick": 0.06, "bevel": 0.05, "margin": 1.04,
        "px_w": 928, "px_h": 736, "samples": 256, "seed": 7201,
        "key": 950.0, "rim": 280.0, "world": 0.06, "exposure": 0.0,
    });
    let fp = fingerprint(&[
        recipes::PRELUDE.as_bytes(),
        POD_LENS.as_bytes(),
        params.to_string().as_bytes(),
        b"post-v2-floor-0.1",
    ]);
    ctx.job("gt7_hood_pro/pod_lens", &fp, |ctx| {
        let out = ctx.scratch("gt7_hood_pro_pod")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "gt7_hood_pro_pod", POD_LENS, &p)?;
        let mut lens = img::load(&out.join("pod_lens.png"))?.to_channels(4);
        // The satin face catches a little key light; take the floor back to OLED black so only
        // the bevel and the top-left gradient read (alpha untouched).
        img::crush_blacks(&mut lens, LENS_BLACK_FLOOR);
        ctx.image(
            "gt7_hood_pro/pod_lens",
            &format!("{DIR}/pod_lens.png"),
            &lens,
            Format::Png8,
            Spec::new(
                "gt7_hood_pro",
                "color",
                "blender:gt7_hood_pro::POD_LENS (squircle n 4 slab, 24-segment rolled bevel, satin clear-coated black dielectric IOR 1.52 (roughness 0.16, coat 0.7 at 0.09); key strip from the upper left at 950 W, cool rim strip 280 W; orthographic top-down, Cycles 256 spp + denoise, AgX, transparent film) seed=7201",
                CLUSTERS,
            )
            .alpha("straight")
            .qa("face black floor 0.1 crushed to #000 (bevel speculars kept); 928x736 = 4x the hood pod (232 x 184 local px) plus a 2% margin each side; drawn 1:1 over the pod squircle"),
        )
    })
}

fn ribbon_grain(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let gen_fp = fingerprint(&[
        workflow.as_bytes(),
        GRAIN_PROMPT.as_bytes(),
        GRAIN_NEGATIVE.as_bytes(),
        &GRAIN_SEED.to_le_bytes(),
    ]);
    let fp = fingerprint(&[gen_fp.as_bytes(), &GRAIN_SIZE.to_le_bytes(), b"grain-post-v1"]);
    ctx.job("gt7_hood_pro/ribbon_grain", &fp, |ctx| {
        let raw_path = ctx.scratch("comfy")?.join("gt7_ribbon_grain.png");
        let raw_fp_path = raw_path.with_extension("fp");
        let cached = raw_path.exists()
            && std::fs::read_to_string(&raw_fp_path)
                .map(|s| s == gen_fp)
                .unwrap_or(false);
        if !cached {
            let c = Comfy::connect(&ctx.paths)?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(GRAIN_PROMPT));
            params.insert("negative".into(), json!(GRAIN_NEGATIVE));
            params.insert("seed".into(), json!(GRAIN_SEED));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1024));
            params.insert("height".into(), json!(1024));
            params.insert("prefix".into(), json!("fh6_gt7_ribbon_grain"));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
        }
        let mut master = img::load(&raw_path)?.to_channels(3);
        img::crush_blacks(&mut master, 0.04);
        let seamless = img::make_seamless(&master, 0.2);
        let small = if seamless.w != GRAIN_SIZE {
            seamless.resize(GRAIN_SIZE, GRAIN_SIZE, true)
        } else {
            seamless
        };
        // Luminance only: the cluster tints it with the palette.
        let grain = img::Img::from_fn(small.w, small.h, 3, |x, y| {
            let l = img::luma(small.get(x, y, 0), small.get(x, y, 1), small.get(x, y, 2));
            [l, l, l, 1.0]
        });
        let qa = format!(
            "SDXL 1024² → 4x-UltraSharp 4096² → {GRAIN_SIZE} px; blacks below 4% crushed to #000; seamless, tile seam score {:.2}; luminance only (tinted at runtime)",
            img::tile_seam_score(&grain)
        );
        ctx.image(
            "gt7_hood_pro/ribbon_grain",
            &format!("{DIR}/ribbon_grain.png"),
            &grain,
            Format::Png8,
            Spec::new(
                "gt7_hood_pro",
                "color",
                format!("comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={GRAIN_SEED} prompt=\"{GRAIN_PROMPT}\""),
                CLUSTERS,
            )
            .alpha("additive")
            .tile()
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa(qa),
        )
    })
}
