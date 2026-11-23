//! ComfyUI API graphs for SDXL, built in Rust instead of hand-edited JSON so every combination
//! stays valid: txt2img or img2img, optional refiner hand-off, optional ControlNet (depth from a
//! Blender pass, or canny computed in the graph), optional inpaint mask (the seamless seam
//! pass), tiled VAE for the 8 GB card, and an optional 4x upscale.
//!
//! Node and input names match ComfyUI 0.46 core nodes (`CheckpointLoaderSimple`,
//! `CLIPTextEncode`, `CLIPTextEncodeSDXLRefiner`, `KSamplerAdvanced`, `ControlNetLoader`,
//! `ControlNetApplyAdvanced`, `Canny`, `LoadImage`, `LoadImageMask`, `SetLatentNoiseMask`,
//! `VAEEncodeTiled`, `VAEDecodeTiled`, `UpscaleModelLoader`, `ImageUpscaleWithModel`).
use serde_json::{json, Map, Value};

/// A ControlNet conditioning.
#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    /// Model file in `models/controlnet`.
    pub model: String,
    /// Uploaded control image (a depth pass, or a beauty when `canny`).
    pub image: String,
    /// Run `Canny` on the image first.
    pub canny: bool,
    /// Strength (0.4–0.8 keeps the geometry without freezing the detail).
    pub strength: f32,
    /// Fraction of the schedule where the control stops (0.6–0.9).
    pub end: f32,
}

/// One generation job.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    /// Positive prompt.
    pub prompt: String,
    /// Negative prompt.
    pub negative: String,
    /// Seed (record it in the ledger).
    pub seed: u64,
    /// Total steps (base + refiner).
    pub steps: u32,
    /// CFG.
    pub cfg: f32,
    /// Output size before upscaling (multiples of 8; SDXL is happiest near 1 MP).
    pub size: (u32, u32),
    /// Base checkpoint.
    pub ckpt: String,
    /// Refiner checkpoint and the fraction of steps the base runs (0.8 is the SDXL default).
    pub refiner: Option<(String, f32)>,
    /// Img2img source (uploaded name) and denoise (0.15–0.35 for a detail pass over a render).
    pub init: Option<(String, f32)>,
    /// Inpaint mask (uploaded name; white = repaint) applied with `SetLatentNoiseMask`.
    pub mask: Option<String>,
    /// ControlNet.
    pub control: Option<Control>,
    /// Upscale model (e.g. `4x-UltraSharp.pth`).
    pub upscale: Option<String>,
    /// Use tiled VAE encode/decode (safe on 8 GB next to other GPU work).
    pub tiled: bool,
    /// Output filename prefix.
    pub prefix: String,
}

impl Job {
    /// A txt2img job with the repo's defaults (dpmpp_2m karras, 30 steps, CFG 6, 1024², tiled).
    #[must_use]
    pub fn new(prompt: &str, negative: &str, seed: u64) -> Self {
        Self {
            prompt: prompt.into(),
            negative: negative.into(),
            seed,
            steps: 30,
            cfg: 6.0,
            size: (1024, 1024),
            ckpt: crate::paths::SDXL_CKPT.into(),
            refiner: None,
            init: None,
            mask: None,
            control: None,
            upscale: None,
            tiled: true,
            prefix: "forge_hero".into(),
        }
    }
}

/// An API-format graph under construction.
#[derive(Debug, Default)]
struct Graph {
    nodes: Map<String, Value>,
}

impl Graph {
    fn add(&mut self, class: &str, inputs: Value) -> String {
        let id = (self.nodes.len() + 1).to_string();
        self.nodes
            .insert(id.clone(), json!({ "class_type": class, "inputs": inputs }));
        id
    }
}

fn o(id: &str, slot: u32) -> Value {
    json!([id, slot])
}

/// Builds the ComfyUI API workflow for `job`.
#[must_use]
pub fn build(job: &Job) -> Value {
    let mut g = Graph::default();
    let (w, h) = (job.size.0 / 8 * 8, job.size.1 / 8 * 8);
    let ck = g.add("CheckpointLoaderSimple", json!({ "ckpt_name": job.ckpt }));
    let mut pos = g.add("CLIPTextEncode", json!({ "text": job.prompt, "clip": o(&ck, 1) }));
    let mut neg = g.add("CLIPTextEncode", json!({ "text": job.negative, "clip": o(&ck, 1) }));
    let (mut pos_slot, mut neg_slot) = (0, 0);
    // Latent: empty, or the encoded init image (optionally masked for an inpaint pass).
    let mut start = 0u32;
    let latent = if let Some((image, denoise)) = &job.init {
        let li = g.add("LoadImage", json!({ "image": image }));
        let sc = g.add(
            "ImageScale",
            json!({ "image": o(&li, 0), "upscale_method": "lanczos", "width": w, "height": h, "crop": "disabled" }),
        );
        let enc = if job.tiled {
            g.add(
                "VAEEncodeTiled",
                json!({ "pixels": o(&sc, 0), "vae": o(&ck, 2), "tile_size": 512, "overlap": 64, "temporal_size": 64, "temporal_overlap": 8 }),
            )
        } else {
            g.add("VAEEncode", json!({ "pixels": o(&sc, 0), "vae": o(&ck, 2) }))
        };
        start = ((job.steps as f32) * (1.0 - denoise.clamp(0.01, 1.0))).round() as u32;
        match &job.mask {
            Some(mask) => {
                let lm = g.add("LoadImageMask", json!({ "image": mask, "channel": "red" }));
                g.add(
                    "SetLatentNoiseMask",
                    json!({ "samples": o(&enc, 0), "mask": o(&lm, 0) }),
                )
            }
            None => enc,
        }
    } else {
        g.add("EmptyLatentImage", json!({ "width": w, "height": h, "batch_size": 1 }))
    };
    if let Some(c) = &job.control {
        let li = g.add("LoadImage", json!({ "image": c.image }));
        let img = if c.canny {
            g.add(
                "Canny",
                json!({ "image": o(&li, 0), "low_threshold": 0.4, "high_threshold": 0.8 }),
            )
        } else {
            li
        };
        let cn = g.add("ControlNetLoader", json!({ "control_net_name": c.model }));
        let ap = g.add(
            "ControlNetApplyAdvanced",
            json!({
                "positive": o(&pos, pos_slot), "negative": o(&neg, neg_slot), "control_net": o(&cn, 0),
                "image": o(&img, 0), "strength": c.strength, "start_percent": 0.0, "end_percent": c.end,
                "vae": o(&ck, 2),
            }),
        );
        pos = ap.clone();
        neg = ap;
        pos_slot = 0;
        neg_slot = 1;
    }
    let base_end = job
        .refiner
        .as_ref()
        .map_or(10_000, |(_, f)| ((job.steps as f32) * f.clamp(0.3, 1.0)).round() as u32);
    let refining = job.refiner.is_some();
    let base = g.add(
        "KSamplerAdvanced",
        json!({
            "model": o(&ck, 0), "add_noise": "enable", "noise_seed": job.seed, "steps": job.steps,
            "cfg": job.cfg, "sampler_name": "dpmpp_2m", "scheduler": "karras",
            "positive": o(&pos, pos_slot), "negative": o(&neg, neg_slot), "latent_image": o(&latent, 0),
            "start_at_step": start, "end_at_step": base_end,
            "return_with_leftover_noise": if refining { "enable" } else { "disable" },
        }),
    );
    let mut samples = base;
    if let Some((refiner, _)) = &job.refiner {
        let rk = g.add("CheckpointLoaderSimple", json!({ "ckpt_name": refiner }));
        let rp = g.add(
            "CLIPTextEncodeSDXLRefiner",
            json!({ "ascore": 6.0, "width": w, "height": h, "text": job.prompt, "clip": o(&rk, 1) }),
        );
        let rn = g.add(
            "CLIPTextEncodeSDXLRefiner",
            json!({ "ascore": 2.5, "width": w, "height": h, "text": job.negative, "clip": o(&rk, 1) }),
        );
        samples = g.add(
            "KSamplerAdvanced",
            json!({
                "model": o(&rk, 0), "add_noise": "disable", "noise_seed": job.seed, "steps": job.steps,
                "cfg": job.cfg, "sampler_name": "dpmpp_2m", "scheduler": "karras",
                "positive": o(&rp, 0), "negative": o(&rn, 0), "latent_image": o(&samples, 0),
                "start_at_step": base_end.max(start), "end_at_step": 10_000,
                "return_with_leftover_noise": "disable",
            }),
        );
    }
    let decoded = if job.tiled {
        g.add(
            "VAEDecodeTiled",
            json!({ "samples": o(&samples, 0), "vae": o(&ck, 2), "tile_size": 512, "overlap": 64, "temporal_size": 64, "temporal_overlap": 8 }),
        )
    } else {
        g.add("VAEDecode", json!({ "samples": o(&samples, 0), "vae": o(&ck, 2) }))
    };
    let mut image = decoded;
    if let Some(model) = &job.upscale {
        let ul = g.add("UpscaleModelLoader", json!({ "model_name": model }));
        image = g.add(
            "ImageUpscaleWithModel",
            json!({ "upscale_model": o(&ul, 0), "image": o(&image, 0) }),
        );
    }
    g.add(
        "SaveImage",
        json!({ "images": o(&image, 0), "filename_prefix": job.prefix }),
    );
    Value::Object(g.nodes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class_count(v: &Value, class: &str) -> usize {
        v.as_object()
            .map(|m| m.values().filter(|n| n["class_type"] == class).count())
            .unwrap_or(0)
    }

    fn links_resolve(v: &Value) -> bool {
        let Some(m) = v.as_object() else { return false };
        m.values().all(|n| {
            n["inputs"].as_object().is_some_and(|ins| {
                ins.values().all(|i| match i.as_array() {
                    Some(a) if a.len() == 2 && a[0].is_string() => a[0].as_str().is_some_and(|id| m.contains_key(id)),
                    _ => true,
                })
            })
        })
    }

    #[test]
    fn txt2img_is_minimal_and_valid() {
        let v = build(&Job::new("brushed titanium", "text", 7));
        assert_eq!(class_count(&v, "EmptyLatentImage"), 1);
        assert_eq!(class_count(&v, "KSamplerAdvanced"), 1);
        assert_eq!(class_count(&v, "VAEDecodeTiled"), 1);
        assert!(links_resolve(&v));
    }

    #[test]
    fn full_chain_has_refiner_controlnet_mask_and_upscale() {
        let mut j = Job::new("leather", "plastic", 1);
        j.refiner = Some(("sd_xl_refiner_1.0.safetensors".into(), 0.8));
        j.init = Some(("beauty.png".into(), 0.3));
        j.mask = Some(("seam.png").into());
        j.control = Some(Control {
            model: "depth.safetensors".into(),
            image: "depth.png".into(),
            canny: false,
            strength: 0.6,
            end: 0.8,
        });
        j.upscale = Some("4x-UltraSharp.pth".into());
        let v = build(&j);
        assert_eq!(class_count(&v, "KSamplerAdvanced"), 2);
        assert_eq!(class_count(&v, "CLIPTextEncodeSDXLRefiner"), 2);
        assert_eq!(class_count(&v, "ControlNetApplyAdvanced"), 1);
        assert_eq!(class_count(&v, "SetLatentNoiseMask"), 1);
        assert_eq!(class_count(&v, "ImageUpscaleWithModel"), 1);
        assert!(links_resolve(&v));
        // Img2img at denoise 0.3 starts the base at step 21 of 30; the refiner takes over at 24.
        let base = v.as_object().and_then(|m| {
            m.values()
                .find(|n| n["class_type"] == "KSamplerAdvanced" && n["inputs"]["add_noise"] == "enable")
                .cloned()
        });
        let base = base.unwrap_or_default();
        assert_eq!(base["inputs"]["start_at_step"], 21);
        assert_eq!(base["inputs"]["end_at_step"], 24);
    }
}
