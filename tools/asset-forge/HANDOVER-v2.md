# Handover: asset pipeline v2 (hero library)

Branch `claude/asset-pipeline-v2`. Not merged, not pushed: the conductor merges.

## Why

PorscheAnalog and RetroCyber ran Blender and ComfyUI, but the result was invisible: luminance
maps added at 10–12 % over vector shapes, rendered top-down where flat faces reflect nothing.
TJX's verdict: it looks like vector UI. v2 renders the **physical hardware itself** at full
strength, registered 1:1 to the kit's local 4K px, and puts only live data in vectors.

## What landed

| Area | Files |
|---|---|
| Library | `tools/asset-forge/src/hero/` — `frame.rs` (camera matching, anchors), `rig.rs` (calibrated rigs, G-swing azimuths), `mat.rs` (PBR cookbook as data), `geom.rs` (lathe profiles with fillets, outlines, smooth union of pods, needle blades), `part.rs` (bezel, face, ring, crystal, dome, extrude, mesh, stitches), `bpy.rs` (the Blender side, a Rust string constant), `mod.rs` (`Scene`, `Passes`, `Rendered`, caching, EXR → PNG), `sdxl.rs` (ComfyUI graph builder), `texture.rs` (prompt cookbook, seamless textures, detail pass), `demo.rs` (the proof) |
| ComfyUI client | `src/comfy.rs`: `upload_image` (multipart), `free`, `models`, `run_retry` (frees VRAM and retries once on OOM) |
| CLI | `asset-forge hero-demo [--preview]`, `asset-forge hero-detail <beauty.png> <out.png> [--depth depth.png] [--denoise 0.25] [--seed N] [--prompt TEXT]`, `doctor` reports the optional models, `asset-forge hero-versus <board.png> <after.png> [out.png]` |
| Skills | `.claude/skills/blender-hero-assets/SKILL.md`, `.claude/skills/comfyui-hero-textures/SKILL.md` |
| Docs | `docs/kit/README.md` §12 (rendered hardware paragraph), `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` ("Use the hero library"), `assets/ASSETS.md` header via `ledger.rs::markdown`, `docs/PROGRESS.md` decision log |
| Proof | `docs/handover-shots/asset-pipeline-v2-{before,after,passes,materials,vs-penpot}.png` (the Penpot board is `07 · PorscheAnalog · 02_mid_cruise`, cropped to the cockpit envelope) |

No new dependency. No change to `assets/` content, the ledger rows or any cluster module.

## How a cluster uses it

```rust
let mut s = Scene::new("<cluster>_hw", Frame::envelope(&frame::COCKPIT));
s.add(Part::bezel("tach", [0.0, -14.0], 0.0, BezelSpec::ring(226.0, 18.0), ti_set));
let r = s.render(&ctx.paths, &ctx.scratch("<cluster>_hw")?)?;
// ctx.image(...) each of r.beauty[0..3], r.normal, r.ao … into assets/clusters/<name>/
```

Kit side: `ImagePaint { rect: frame rect }` at full strength under the vectors; needle sprite
rotated on top with a soft offset shadow; glass layer last; crossfade `beauty_0/1/2` by the
`VirtualLight` azimuth (−135° rest, −160°, −110°).

## Calibration notes (measured, keep them)

- Light power 1.0 straight above white diffuse (albedo 0.8) ≈ 0.8 linear at any distance
  (energy = power × 10 × d², Cycles, Standard view).
- The overhead "sky" must stay dim (0.14): a big source is mirrored directly by every flat face.
  At 0.9 every metal face clipped white and the vectors vanished (first preview run).
- Leather from SDXL: the 2048 tile reads as pebble grain at `tile_px` ≈ 150; at 520 it reads as
  quilted cushions.
- Needles: a polished (rough 0.08) blade mirrors the black void and disappears top-down; satin
  0.3 plus a wide lume insert reads.
- Blender 5.3 Principled sockets used: `Coat Weight/Roughness/Tint`, `Sheen Weight/Roughness`,
  `Thin Film Thickness/IOR`, `Transmission Weight`, `Subsurface Weight/Radius` (3-vector),
  `Anisotropic`, `Tangent`. `film_transparent_glass` keeps crystals reflection-only.

## Gates

See the commit message for the run: fmt, clippy `-D warnings`, test (workspace), parity 20/20.

## Open items / requests

1. **Models installed** (TJX approved the download in chat on 2026-11-23; sizes verified against
   Hugging Face, `asset-forge doctor` lists them):
   - `stabilityai/stable-diffusion-xl-refiner-1.0` → `sd_xl_refiner_1.0.safetensors`,
     6,075,981,930 bytes, CreativeML Open RAIL++-M, into `models/checkpoints/` (refiner hand-off).
   - `diffusers/controlnet-depth-sdxl-1.0-small` → `diffusion_pytorch_model.fp16.safetensors`,
     320,237,179 bytes, openrail++, saved as `controlnet-depth-sdxl-1.0-small.fp16.safetensors`
     in `models/controlnet/` (render → depth → refine).
   - `diffusers/controlnet-canny-sdxl-1.0-small` → same file name, 320,237,179 bytes,
     openrail++, saved as `controlnet-canny-sdxl-1.0-small.fp16.safetensors`.
   The code detects them by name (`texture::Models::probe`) and still works without them.
   Smoke test: `asset-forge hero-detail <beauty.png> <out.png> --depth <depth.png>` on the
   PorscheAnalog plate ran base + depth ControlNet + refiner at denoise 0.25: geometry and alpha
   kept, brushing and knurling sharper.
2. **Penpot side-by-side:** done from `07 · PorscheAnalog · 02_mid_cruise`. The demo's vectors are a stand-in (its tach runs 0 at lower-left); the board's dial layout belongs to the PorscheAnalog chat. Rerun any time: export the board with `export_shape`, then `asset-forge hero-versus <board.png> docs/handover-shots/asset-pipeline-v2-after.png`.
3. **Kit request:** a lit-sprite pass (normal + rmh + the `VirtualLight`) would replace the
   three-beauty crossfade with true relighting; the library already writes those maps.
4. **PorscheAnalog adoption** belongs to that chat: `hero::demo::{hardware, needle, glass}` are
   drop-in scenes matching its zone table.
