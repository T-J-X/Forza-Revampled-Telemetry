# TeslaAdas assets

Built by `cargo run --release -p asset-forge -- build --set tesla_adas` from `tools/asset-forge/src/recipes/tesla_adas.rs`. Don't edit these files by hand: change the recipe and rebuild. The ledger rows are also in `assets/ASSETS.md`. The cluster has a procedural fallback for each file, used when `setup.assets` is `None` or the file is missing.

| File | Tool | Recipe and seed | Size | Used for | Fallback |
|---|---|---|---|---|---|
| `underglow_pool_cockpit.png` | Blender 5.3 (Cycles, 256 spp, OptiX denoise) | `tesla_adas::POOL`, seed 151. The Phase 6 `adas/ego_coupe.glb` on a white ground, invisible to the camera but still blocking light. Downward area light 1.9 × 4.0 m at 0.16 m, 520 W. Chase camera eye (0, 3.4, −6), 8° pitch, focal 440 px (the cluster's cockpit camera), shifted so the image centre is the ego's ground centre. Post: luminance → alpha over white, softened 8×, gamma 0.7, edges feathered. | 960 × 480 (480 × 240 local px) | The ego's underglow in the cockpit view. Tinted live: primary, amber in a drift, limiter violet. It carries the real tyre and sill shadows. | A Gaussian round a ground ellipse |
| `underglow_pool_hood.png` | Blender 5.3 | The same, through the hood camera: eye (0, 2.8, −6.5), focal 400 px. | 960 × 480 | The ego's underglow in the hood view | The same Gaussian |
| `horizon_haze.png` | ComfyUI (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras, 32 steps, cfg 6.5, 1536 × 640) | Seed 1505; prompt in the recipe (`HAZE_PROMPT`). Post: the sky band from 18 to 70% of the 6144 × 2560 upscale (its bottom edge is the horizon), → 1536 × 384, blacks below 3% crushed, luminance → alpha over white, sides and top / bottom feathered. | 1536 × 384 | A thin band of atmosphere on the horizon behind the far road, tinted primary at 16%. | None (the horizon is plain black) |

Licences: the Blender renders are MIT (asset-forge); the SDXL output is under the CreativeML Open RAIL++-M licence, with the asset-forge code MIT.
