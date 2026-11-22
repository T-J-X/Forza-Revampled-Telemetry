//! ComfyUI plates: SDXL base 1.0 at 1024², 4x-UltraSharp to 4096², then post-processed in Rust
//! (true-black floor for OLED, seamless tiling where marked, verified seam). Prompts and seeds
//! live here; the workflow is `tools/asset-forge/workflows/sdxl_plate.json` (data).
use crate::comfy::{self, Comfy};
use crate::ctx::{Ctx, Format, Spec};
use crate::error::{tool_err, Result};
use crate::img;
use crate::ledger::fingerprint;
use serde_json::{json, Map};
use std::time::Duration;

struct Plate {
    set: &'static str,
    name: &'static str,
    prompt: &'static str,
    tile: bool,
    /// Mirror the top-left quadrant into a four-fold symmetric plate.
    mirror4: bool,
    seed: u64,
    /// Output size (longest side); 4096 for hero plates, 2048 where the plate is tiled or blurred.
    size: usize,
    clusters: &'static str,
}

const NEGATIVE: &str = "text, letters, watermark, logo, signature, frame, border, vignette, people, face, car, user interface, lowres, blurry, jpeg artifacts, noise, grain, oversaturated";

const PLATES: &[Plate] = &[
    Plate { set: "glass", name: "caustic_water", prompt: "macro photograph of bright water caustic light patterns projected on a pure black floor, thin luminous white filaments, high contrast, monochrome, crisp", tile: true, mirror4: false, seed: 1101, size: 2048, clusters: "LiquidGlass caustic overlay" },
    Plate { set: "glass", name: "caustic_prism", prompt: "refracted light caustics through cut crystal glass, delicate luminous network with prismatic rainbow fringes, pure black background, elegant, crisp", tile: true, mirror4: false, seed: 1102, size: 2048, clusters: "LiquidGlass, chromatic glass concepts" },
    Plate { set: "fluid", name: "plasma", prompt: "abstract plasma energy field, swirling electric violet and cyan filaments, fluid dynamics simulation, deep black background, ultra detailed, luminous", tile: true, mirror4: false, seed: 1201, size: 2048, clusters: "fluid / generative family, plasma concepts" },
    Plate { set: "fluid", name: "nebula", prompt: "deep space nebula, wisps of magenta and teal glowing gas, fine dust lanes, scattered tiny stars, pure black space, astrophotography, ultra detailed", tile: true, mirror4: false, seed: 1202, size: 2048, clusters: "fluid / generative family, space concepts" },
    Plate { set: "fluid", name: "ferrofluid_macro", prompt: "macro photograph of black ferrofluid spikes rising in a magnetic field, glossy liquid metal, cool blue studio rim light, pure black background, ultra detailed", tile: false, mirror4: false, seed: 1203, size: 2048, clusters: "Ferrofluid concept, fluid family" },
    Plate { set: "fluid", name: "liquid_chrome", prompt: "abstract liquid chrome flowing surface, mercury ripples and folds, mirror reflective silver, soft studio light, dark background, ultra detailed", tile: true, mirror4: false, seed: 1204, size: 2048, clusters: "LiquidMetal concepts, fluid family" },
    Plate { set: "fluid", name: "chromatic_prism", prompt: "abstract chromatic dispersion of white light through a glass prism, spectral rainbow bands fanning out on pure black, minimal, elegant, high contrast", tile: false, mirror4: false, seed: 1205, size: 2048, clusters: "Shibuya chromatic concept, sci-fi and fluid families" },
    Plate { set: "fluid", name: "chladni_sand", prompt: "Chladni figure, fine white sand on a black vibrating metal plate forming a symmetric nodal pattern, top view, high contrast, crisp", tile: false, mirror4: true, seed: 1206, size: 2048, clusters: "Chladni / resonance concept" },
    Plate { set: "scifi", name: "warp_streaks", prompt: "hyperspace warp jump, streaks of blue and violet light converging to a bright white center, radial motion, pure black background, cinematic", tile: false, mirror4: false, seed: 1301, size: 2048, clusters: "warp / FUI concepts, sci-fi family" },
    Plate { set: "scifi", name: "energy_core", prompt: "glowing fusion reactor energy core seen from the front, concentric rings of cyan light around a white hot center, sci-fi, centered, pure black background", tile: false, mirror4: false, seed: 1302, size: 2048, clusters: "energy-core FUI concepts, sci-fi family" },
    Plate { set: "scifi", name: "accretion_disk", prompt: "black hole with a glowing accretion disk, orange and white hot plasma ring, gravitational lensing arc, centered, pure black space, cinematic", tile: false, mirror4: false, seed: 1303, size: 2048, clusters: "ChronoEventHorizon concept" },
    Plate { set: "scifi", name: "holo_interference", prompt: "holographic interference pattern, iridescent cyan and magenta fine moire lines, dark background, abstract, crisp", tile: true, mirror4: false, seed: 1304, size: 2048, clusters: "hologram overlays, sci-fi family" },
    Plate { set: "scifi", name: "synth_sun", prompt: "retro synthwave sunset sun disc with horizontal cut stripes, neon magenta to orange gradient, pure black background, minimal flat graphic", tile: false, mirror4: false, seed: 1305, size: 2048, clusters: "SynthwaveOutrun" },
];

/// Builds every plate.
pub fn build(ctx: &mut Ctx) -> Result<()> {
    let wf_path = ctx.paths.forge_dir().join("workflows").join("sdxl_plate.json");
    let workflow = String::from_utf8_lossy(&crate::ctx::read(&wf_path)?).to_string();
    let mut comfy: Option<Comfy> = None;
    for p in PLATES {
        // The SDXL render is keyed by what feeds ComfyUI; the job also covers post-processing, so a
        // post change re-uses the cached raw render instead of re-running SDXL.
        let gen_fp = fingerprint(&[
            workflow.as_bytes(),
            p.prompt.as_bytes(),
            NEGATIVE.as_bytes(),
            &p.seed.to_le_bytes(),
        ]);
        let fp = fingerprint(&[
            b"comfy-v3",
            &[u8::from(p.mirror4)],
            workflow.as_bytes(),
            p.prompt.as_bytes(),
            NEGATIVE.as_bytes(),
            &p.seed.to_le_bytes(),
            &[u8::from(p.tile)],
            &p.size.to_le_bytes(),
        ]);
        let job = format!("comfy/{}", p.name);
        ctx.job(&job, &fp, |ctx| {
            let raw_path = ctx.scratch("comfy")?.join(format!("{}.png", p.name));
            let raw_fp_path = raw_path.with_extension("fp");
            let cached = raw_path.exists() && std::fs::read_to_string(&raw_fp_path).map(|s| s == gen_fp).unwrap_or(false);
            if !cached {
            if comfy.is_none() {
                comfy = Some(Comfy::connect(&ctx.paths)?);
            }
            let c = comfy.as_ref().ok_or_else(|| tool_err("comfyui", "not connected"))?;
            let mut params = Map::new();
            params.insert("prompt".into(), json!(p.prompt));
            params.insert("negative".into(), json!(NEGATIVE));
            params.insert("seed".into(), json!(p.seed));
            params.insert("steps".into(), json!(32));
            params.insert("cfg".into(), json!(6.5));
            params.insert("width".into(), json!(1024));
            params.insert("height".into(), json!(1024));
            params.insert("prefix".into(), json!(format!("fh6_{}", p.name)));
            let wf = comfy::patch(&workflow, &params)?;
            let images = c.run(&wf, Duration::from_secs(900))?;
            let bytes = images.first().ok_or_else(|| tool_err("comfyui", "no image"))?;
            img::write_file(&raw_path, bytes)?;
            img::write_file(&raw_fp_path, gen_fp.as_bytes())?;
            }
            let mut master = img::load(&raw_path)?.to_channels(3);
            img::crush_blacks(&mut master, 0.03);
            if p.mirror4 {
                master = img::mirror4(&master);
            }
            if p.tile {
                master = img::make_seamless(&master, 0.2);
            }
            // The 4096 px master stays out of git (assets/_masters/, rebuilt by this job); the repo
            // ships the plate at its listed size.
            let master_rel = format!("assets/_masters/{}/plate_{}_4k.png", p.set, p.name);
            img::save_png(&master, &ctx.abs(&master_rel), false, true)?;
            let plate = if master.w != p.size { master.resize(p.size, p.size, p.tile) } else { master };
            let mut qa = format!("SDXL 1024² → 4x-UltraSharp 4096² (master in {master_rel}, untracked) → {} px; blacks below 3% crushed to #000", p.size);
            if p.tile {
                qa.push_str(&format!("; made seamless, tile seam score {:.2}", img::tile_seam_score(&plate)));
            }
            let mut spec = Spec::new(
                p.set,
                "color",
                format!("comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed={} prompt=\"{}\"", p.seed, p.prompt),
                p.clusters,
            )
            .alpha("additive")
            .licence("CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge)")
            .qa(qa);
            if p.tile {
                spec = spec.tile();
            }
            ctx.image(&job, &format!("assets/{}/plate_{}.png", p.set, p.name), &plate, Format::Png8, spec)
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn workflow_patches_cleanly() {
        let wf = include_str!("../../workflows/sdxl_plate.json");
        let mut p = serde_json::Map::new();
        for (k, v) in [
            ("prompt", serde_json::json!("x")),
            ("negative", serde_json::json!("y")),
            ("seed", serde_json::json!(1)),
            ("steps", serde_json::json!(2)),
            ("cfg", serde_json::json!(3.0)),
            ("width", serde_json::json!(1024)),
            ("height", serde_json::json!(1024)),
            ("prefix", serde_json::json!("z")),
        ] {
            p.insert(k.into(), v);
        }
        let v = super::comfy::patch(wf, &p).unwrap_or_default();
        assert_eq!(v["3"]["inputs"]["seed"], serde_json::json!(1));
        assert!(!v.to_string().contains("{{"));
    }
}
