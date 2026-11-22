//! Abstract fluid set: looping liquid-metal and ferrofluid flipbooks (Blender Cycles, Geometry
//! Nodes displacement on a closed 4D-noise loop) with matching normal and flow atlases, studio
//! matcaps, Chladni plates and tileable curl-noise flow.
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{ForgeError, Result};
use crate::img::{self, Img};
use crate::ktx2w::Encoding;
use crate::ledger::{fingerprint, Flipbook};
use crate::procedural as pr;
use crate::{blender, recipes};
use serde_json::json;

const CLUSTERS: &str = "fluid / generative family (LiquidMetal, Ferrofluid, MercuryPool concepts), LiquidGlass meniscus orb";
const FRAMES: usize = 64;
const CELL: usize = 512;
const COLS: usize = 8;
/// Cell size of the packed normal + flow atlas (2048² for 64 frames).
const DATA_CELL: usize = 256;

/// Builds every fluid job.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    for (style, note) in [
        ("mercury_blob", "chrome blob, 3 octaves of looped 4D noise on an icosphere"),
        ("ferrofluid", "glossy black dome with looped 4D Voronoi spikes (Rosensweig-style)"),
        ("mercury_pool", "top-down mercury pool surface, looped 4D noise height"),
    ] {
        flipbook(ctx, style, note)?;
    }
    matcaps(ctx)?;
    maths(ctx)?;
    Ok(())
}

fn flipbook(ctx: &mut Ctx, style: &str, note: &str) -> Result<()> {
    let params = json!({ "style": style, "frames": FRAMES, "cell": CELL, "samples": 96, "seed": 51 });
    let body = format!("{}\n{}", recipes::fluid::FLUID_HELPERS, recipes::fluid::FLIPBOOK);
    let fp = fingerprint(&[recipes::PRELUDE.as_bytes(), body.as_bytes(), params.to_string().as_bytes(), b"post-v2"]);
    let job = format!("fluid/{style}");
    ctx.job(&job, &fp, |ctx| {
        let out = ctx.scratch(&format!("fluid_{style}"))?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, &format!("fluid_{style}"), &body, &p)?;
        let mut beauty = Vec::with_capacity(FRAMES);
        let mut packed = Vec::with_capacity(FRAMES);
        for f in 0..FRAMES {
            let b = img::load(&out.join(format!("beauty_{f:04}.png")))?;
            let d = img::load_exr(&out.join(format!("data_{f:04}.exr")))?;
            let (n, h) = crate::sets::glass::unpack_data(&d);
            let fl = img::flow_from_height(&h, 0, false);
            // RG = normal.xy (z = sqrt(1 - x² - y²)), BA = flow.xy; coverage comes from the beauty alpha.
            let p = Img::from_fn(n.w, n.h, 4, |x, y| [n.get(x, y, 0), n.get(x, y, 1), fl.get(x, y, 0), fl.get(x, y, 1)]);
            packed.push(p.resize(DATA_CELL, DATA_CELL, false));
            beauty.push(b);
        }
        let loop_score = img::loop_seam_score(&beauty);
        if loop_score > 2.0 {
            return Err(ForgeError::Check {
                asset: job.clone(),
                message: format!("loop seam score {loop_score:.2} > 2.0"),
            });
        }
        let flip = Flipbook { cols: COLS as u32, rows: (FRAMES / COLS) as u32, frames: FRAMES as u32, fps: 30, cell: CELL as u32 };
        let gen = format!("blender:fluid::FLIPBOOK style={style} ({note}), {FRAMES} f, Cycles 96 spp + OptiX denoise seed=51");
        let qa = format!("loop seam score {loop_score:.2} (1.0 = as smooth as any step)");
        ctx.image(
            &format!("{job}/beauty"),
            &format!("assets/fluid/{style}_beauty_8x8.png"),
            &img::atlas(&beauty, COLS),
            Format::Png8,
            Spec::new("fluid", "flipbook", gen.clone(), CLUSTERS).alpha("straight").flip(flip).qa(qa.clone()),
        )?;
        let data_flip = Flipbook { cell: DATA_CELL as u32, ..flip };
        ctx.image(
            &format!("{job}/normal_flow"),
            &format!("assets/fluid/{style}_normal_flow_8x8.ktx2"),
            &img::atlas(&packed, COLS),
            Format::Ktx2(Encoding::Linear),
            Spec::new("fluid", "data", gen, CLUSTERS)
                .alpha("none")
                .flip(data_flip)
                .qa("RG = camera-space normal.xy (0.5-centred, z = sqrt(1-x²-y²)); BA = divergence-free flow along displacement contours (0.5-centred); coverage = beauty alpha"),
        )?;
        Ok(())
    })
}

fn matcaps(ctx: &mut Ctx) -> Result<()> {
    let items = json!([
        {"name": "matcap_chrome", "color": [0.93, 0.93, 0.95], "roughness": 0.03},
        {"name": "matcap_mercury", "color": [0.78, 0.79, 0.81], "roughness": 0.05},
        {"name": "matcap_satin_steel", "color": [0.82, 0.82, 0.84], "roughness": 0.26},
        {"name": "matcap_black_glass", "color": [0.01, 0.01, 0.012], "roughness": 0.04, "metallic": 0.0, "coat": 1.0},
        {"name": "matcap_ferro", "color": [0.012, 0.012, 0.014], "roughness": 0.16, "metallic": 0.25, "coat": 1.0}
    ]);
    let params = json!({ "items": items, "size": 1024, "samples": 256, "seed": 52 });
    let body = format!("{}\n{}", recipes::fluid::FLUID_HELPERS, recipes::fluid::MATCAPS);
    let fp = fingerprint(&[recipes::PRELUDE.as_bytes(), body.as_bytes(), params.to_string().as_bytes(), b"post-v1"]);
    ctx.job("fluid/matcaps", &fp, |ctx| {
        let out = ctx.scratch("fluid_matcaps")?;
        let mut p = params.clone();
        p["out"] = json!(out.to_string_lossy());
        blender::run(&ctx.paths, "fluid_matcaps", &body, &p)?;
        for it in items.as_array().into_iter().flatten() {
            let name = it["name"].as_str().unwrap_or_default();
            let m = img::load(&out.join(format!("{name}.png")))?;
            ctx.image(
                &format!("fluid/matcaps/{name}"),
                &format!("assets/fluid/{name}.png"),
                &m,
                Format::Png8,
                Spec::new(
                    "fluid",
                    "color",
                    "blender:fluid::MATCAPS (sphere in Blender's bundled CC0 studio.exr + 4 softboxes) seed=52",
                    "fluid family surface shading, classic hubs, ADAS car paint (kit mesh pass matcap)",
                )
                .alpha("straight"),
            )?;
        }
        Ok(())
    })
}

fn maths(ctx: &mut Ctx) -> Result<()> {
    let fp = fingerprint(&[b"fluid-maths-v1", include_bytes!("../procedural.rs")]);
    ctx.job("fluid/maths", &fp, |ctx| {
        for (i, (n, m)) in [(3.0f32, 5.0f32), (4.0, 7.0), (5.0, 9.0), (2.0, 11.0)].iter().enumerate() {
            let c = pr::chladni(2048, *n, *m, 0.035);
            let c = pr::blur(&c, 0.7, false);
            let enc = Img::from_fn(c.w, c.h, 1, |x, y| [img::srgb_encode(c.get(x, y, 0)), 0.0, 0.0, 0.0]);
            ctx.image(
                &format!("fluid/maths/chladni_{i}"),
                &format!("assets/fluid/chladni_{}_{}.png", *n as u32, *m as u32),
                &enc,
                Format::Png8,
                Spec::new(
                    "fluid",
                    "color",
                    format!("rust:procedural::chladni modes ({n},{m}): cos(nπx)cos(mπy) − cos(mπx)cos(nπy) = 0"),
                    "Chladni / resonance concepts, fluid family idle state",
                )
                .alpha("additive"),
            )?;
        }
        let curl = pr::curl_noise(1024, 6, 61);
        ctx.image(
            "fluid/maths/curl_noise",
            "assets/fluid/curl_noise_flow.ktx2",
            &curl,
            Format::Ktx2(Encoding::Linear),
            Spec::new("fluid", "flow", "rust:procedural::curl_noise (curl of tileable fBm) seed=61", "GPU fluid advection seed, fluid family")
                .tile()
                .qa(format!("tile seam score {:.2}", img::tile_seam_score(&curl))),
        )?;
        Ok(())
    })
}
