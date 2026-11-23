---
name: comfyui-hero-textures
description: ComfyUI (SDXL) texture and detail work for FH6 WheelDash hero assets through asset-forge - SDXL base + refiner, ControlNet depth/canny guided by Blender passes (render → ControlNet → refine), img2img detail passes over Blender beauties, seamless tiling, 4x upscale, the per-material prompt/negative cookbook, seeds and the ledger, and 8 GB VRAM safety. Use whenever a cluster needs an organic texture (leather, alcantara, enamel, forged carbon), a generative detail pass over a render, or any ComfyUI job.
---

# ComfyUI hero textures

Code: `tools/asset-forge/src/hero/sdxl.rs` (graph builder) and `hero/texture.rs` (cookbook,
seamless textures, detail pass); HTTP in `src/comfy.rs`. Run everything through `asset-forge`
with the **Bash** tool, never PowerShell.

## What SDXL is for (and what it is not)

- **Yes:** organic surfaces Blender can't fake convincingly — leather grain, alcantara nap,
  grand-feu enamel depth, forged carbon — and a low-denoise detail pass over a Blender beauty.
- **No:** metals, guilloché, carbon twill, anything that must be exact, symmetric or relit.
  Those are procedural in Blender (`blender-hero-assets`). SDXL never draws a whole cluster: it
  invents numerals and breaks geometry.

## Models

Installed: `sd_xl_base_1.0.safetensors`, `4x-UltraSharp.pth`, `sd_xl_refiner_1.0.safetensors`,
`controlnet-depth-sdxl-1.0-small.fp16.safetensors` and `controlnet-canny-sdxl-1.0-small.fp16.safetensors`
(the last three from the official stabilityai and diffusers Hugging Face orgs; `asset-forge doctor`
lists what is present). Any further download needs TJX's explicit OK.
`texture::Models::probe` switches features on when the files appear in
`models/checkpoints` / `models/controlnet`; nothing breaks when they're missing.

## Graphs (`sdxl::Job` → `sdxl::build`)

One builder covers every combination, so graphs are always valid:

- **txt2img:** `Job::new(prompt, negative, seed)` — dpmpp_2m karras, 30 steps, CFG 6, 1024².
- **Refiner hand-off:** `job.refiner = Some((name, 0.8))` — base `KSamplerAdvanced` stops at 80 %
  with leftover noise, refiner (`CLIPTextEncodeSDXLRefiner`, ascore 6 / 2.5) finishes.
- **img2img:** `job.init = Some((uploaded, denoise))` → `start_at_step = steps × (1 − denoise)`.
- **Inpaint / seam repaint:** `job.mask = Some(uploaded_mask)` → `SetLatentNoiseMask` (white =
  repaint).
- **ControlNet:** `job.control = Some(Control { model, image, canny, strength, end })` —
  depth from Blender's `depth.png` (near = white), or `canny: true` to edge-detect a beauty in
  graph. Strength 0.55–0.7, end 0.8–0.9.
- **Upscale:** `job.upscale = Some("4x-UltraSharp.pth")`.
- **Tiled VAE:** `job.tiled = true` (default) — `VAEEncodeTiled` / `VAEDecodeTiled` 512/64.

## Render → ControlNet → refine

1. Blender renders the part with `Passes::ALL` (`beauty_0.png`, `depth.png`, `normal.png`).
2. `texture::detail_pass(paths, beauty, Some(depth), prompt, 0.25, seed, out)` (CLI:
   `asset-forge hero-detail <beauty.png> <out.png> --depth <depth.png>`): flattens the
   beauty on OLED black, works near 1 MP, img2img at denoise 0.2–0.35, ControlNet depth when
   installed (else plain img2img), refiner when installed, scales back, **restores the beauty's
   alpha** so the result drops into the same `ImagePaint` rect.
3. Compare against the plain beauty at 100 % zoom: keep the pass only if it adds real micro-detail
   without moving an edge. Above 0.35 denoise SDXL starts redrawing geometry — don't.

## Seamless tiling (`texture::make_texture`)

1. txt2img 1024² (refiner if present).
2. `half_offset` in Rust (no core ComfyUI roll node) → seams move to the centre cross.
3. Masked img2img (denoise 0.55) with `seam_mask(w, h, 0.08)` repaints only the cross.
4. 4x upscale (4096²), resize to the target, `img::make_seamless` band 0.04 as a final guard,
   `img::tile_seam_score` printed — reject anything above the existing comfy set's QA line.

Use the tile in Blender with `mat::leather(..).set("texture", path).set("tile_px", 520.0)`.

## Prompt cookbook (`texture::COOKBOOK`)

Shared negative `texture::NEGATIVE`: text, watermark, logo, frame, perspective, lighting falloff,
objects, seams, blur, cartoon. Per material:

| Name | Prompt core | Extra negative | Seed |
|---|---|---|---|
| `leather_black_pebble` | black full-grain nappa, fine pebble grain, flat top-down, even diffuse, tileable | stitching, perforation, plastic | 61207 |
| `leather_cognac` | cognac aniline, natural grain, tan variation | stitching, plastic | 61311 |
| `alcantara_black` | black alcantara microfibre, soft nap, directional sheen | leather pores | 61419 |
| `enamel_grand_feu` | deep blue grand-feu enamel, glassy depth, suspended bubbles | numerals, hands | 61523 |
| `carbon_forged` | forged carbon, marbled chopped flakes under resin | woven, twill | 61631 |

Prompt rules: always "macro photograph … flat top-down … even diffuse light … seamless tileable
texture"; lighting comes from Blender, so the texture must be flat-lit. Never put a brand or
text in a prompt.

## Seeds and the ledger

- Every job is cached with a `.fp` fingerprint of its graph next to the output: same prompt,
  seed and models → no GPU work.
- Record seed, prompt name, models and licence (SDXL output: CreativeML Open RAIL++-M, as the
  existing comfy rows do) via `Spec::licence` in `ctx.image`, so
  `assets/ASSETS.md` stays reproducible.

## 8 GB VRAM safety

- One GPU job at a time, across Blender and ComfyUI. Never start ComfyUI work while a Cycles
  render is running from the same recipe.
- `Comfy::run_retry` frees VRAM (`POST /free`) and retries once on an out-of-memory error;
  `texture.rs` calls `free()` after each job so Blender gets the card back.
- Tiled VAE always; keep generation near 1 MP and upscale with the 4x model, never generate at
  2048².
- If ComfyUI must be started, `Comfy::connect` launches the portable build; it is shared with
  other chats, so never kill it.
