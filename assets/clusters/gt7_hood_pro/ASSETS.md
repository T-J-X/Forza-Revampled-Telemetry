# Gt7HoodPro assets

Built by `cargo run --release -p asset-forge -- --masters gt7_hood_pro build --set gt7_hood_pro` (recipe `tools/asset-forge/src/recipes/gt7_hood_pro.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws a procedural fallback for every file that is missing.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into `plate_*.png`) | seamless forged-carbon texture | 4096x4096 | master | comfyui:gt7_hood_pro::CARBON (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=73101 prompt="macro photograph of forged carbon fibre composite, large marbled chopped carbon flakes in graphite grey and black under clear resin, glossy, flat top-down, filling the whole frame, even soft studio light, seamless tileable texture, physically based material scan" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of the tyre diagram's car chip (one repeat per 320 px). Master `gt7_hood_pro_forged_carbon.png` |
| `plate_0.png` | the band's static hardware: black-glass light bar on a polished plinth with 95 smoked LED lenses, carbon car chip with four tyre lenses, polished boost arc and its black-glass groove, two pedal slots, two telltale rings and the trip tile; covers x -880..880, y -100..140; key at -135° (sRGB, straight alpha) | 1760x240 | 72 KB | blender:hero (gt7_hood_pro::plate, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `plate/beauty_0.png` |
| `plate_1.png` | the band's static hardware: black-glass light bar on a polished plinth with 95 smoked LED lenses, carbon car chip with four tyre lenses, polished boost arc and its black-glass groove, two pedal slots, two telltale rings and the trip tile; covers x -880..880, y -100..140; key at -160° (sRGB, straight alpha) | 1760x240 | 77 KB | blender:hero (gt7_hood_pro::plate, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `plate/beauty_1.png` |
| `plate_2.png` | the band's static hardware: black-glass light bar on a polished plinth with 95 smoked LED lenses, carbon car chip with four tyre lenses, polished boost arc and its black-glass groove, two pedal slots, two telltale rings and the trip tile; covers x -880..880, y -100..140; key at -110° (sRGB, straight alpha) | 1760x240 | 76 KB | blender:hero (gt7_hood_pro::plate, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `plate/beauty_2.png` |
| `suggest_0.png` | the suggested-gear tile: red enamel on a black-chrome base; covers x 156..266, y -2..100; key at -135° (sRGB, straight alpha) | 110x102 | 5 KB | blender:hero (gt7_hood_pro::suggest, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `suggest/beauty_0.png` |
| `suggest_1.png` | the suggested-gear tile: red enamel on a black-chrome base; covers x 156..266, y -2..100; key at -160° (sRGB, straight alpha) | 110x102 | 5 KB | blender:hero (gt7_hood_pro::suggest, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `suggest/beauty_1.png` |
| `suggest_2.png` | the suggested-gear tile: red enamel on a black-chrome base; covers x 156..266, y -2..100; key at -110° (sRGB, straight alpha) | 110x102 | 5 KB | blender:hero (gt7_hood_pro::suggest, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at power 0.55) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light. Master `suggest/beauty_2.png` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/gt7_hood_pro/`)

Copied by `--masters gt7_hood_pro`: the Blender outputs into one sub-folder per job (`hero_gt7_hood_pro_<stem>`), the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each.

| Master | What it is |
|---|---|
| `gt7_hood_pro_forged_carbon.png` | forged carbon (ComfyUI, seamless 4096 px): the car chip's colour and relief |
| `beauty_0.png` | plate, key -135° |
| `beauty_1.png` | plate, key -160° |
| `beauty_2.png` | plate, key -110° |
| `beauty_0.png` | suggest, key -135° |
| `beauty_1.png` | suggest, key -160° |
| `beauty_2.png` | suggest, key -110° |

7 preview(s) written.
