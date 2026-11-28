# KineticLiquidMetal assets

Built by `cargo run --release -p asset-forge -- --masters kinetic_liquid_metal build --set kinetic_liquid_metal` (recipe `tools/asset-forge/src/recipes/kinetic_liquid_metal.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws procedural caps when the file is missing.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into `end_cap.png`) | seamless blasted-metal texture | 4096x4096 | master | comfyui:kinetic_liquid_metal::CAP_GRAIN (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=70727 prompt="extreme macro photograph of bead blasted black anodised aluminium, a fine uniform satin matte grain of tiny glass bead impacts, a completely flat metal surface filling the whole frame, top-down, even soft diffuse light, seamless tileable texture, physically based material scan" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of the caps' recessed faces (one repeat per 256 px). Master `cap_grain_neutral_f8510656fc00.png` |
| `end_cap.png` | machined end cap seen end-on: blasted black face, diamond-cut chamfer, turned black anodised land, reeded outer chamfer (sRGB, straight alpha) | 60x60 | 5 KB | blender:hero (kinetic_liquid_metal::cap_scene, lathe r 30 px, Cycles 384 samples, ortho at 4 px per 4K px, box-filtered to 1:1; hero rig at its rest key) | MIT (asset-forge) | both end caps of the lens-front tube, drawn inside a circle of the cap's radius; the left one carries the logo zone. Master `end_cap/beauty_0.png` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/kinetic_liquid_metal/`)

Copied by `--masters kinetic_liquid_metal`: the Blender outputs into `hero_kinetic_liquid_metal_end_cap/`, the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each for the tracker.

| Master | What it is |
|---|---|
| `kinetic_liquid_metal_cap_grain.png` | cap grain (ComfyUI, seamless 4096 px) |
| `cap_grain_neutral_f8510656fc00.png` | cap grain made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads |
| `beauty_0.png` | end cap, rest key |
| `normal.png` | end cap, tangent-space normal (16-bit) |
| `height.png` | end cap, height (16-bit) |
| `ao.png` | end cap, ambient occlusion (16-bit) |

6 preview(s) written.
