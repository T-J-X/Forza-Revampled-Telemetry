# BoschDdu assets

Built by `cargo run --release -p asset-forge -- --lane cloud --masters bosch_ddu build --set bosch_ddu` (recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws its procedural housing when the files are missing. The housing renders cover the runtime's `HOUSING_FRAME`, x −440..440, y −320..240 in local 4K px, at 1 px per px; `housing_1` is the key at the smaller azimuth, as the runtime blends it.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into the housing) | seamless bead-blast texture | 4096x4096 | master | comfyui:bosch_ddu::BEAD_BLAST (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=71207 prompt="extreme macro photograph of bead blasted anodised aluminium, fine uniform matte satin grain of tiny glass bead impact pits, a completely flat metal surface filling the whole frame, top-down, even soft diffuse light, seamless tileable texture, physically based material scan, neutral grey", then neutral (luminance, mean 0.5, deviation 0.08 over a 48 px window) | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the anodised body's colour and roughness variation and its faint bump (one repeat per 80 px). Master `bosch_ddu_bead_blast_neutral_a443e36f330a.png` |
| `housing_0.png` | the DDU 10 housing: anodised body, black glass face, 10 + 5 + 5 smoked lenses, light sensor; key at -135° (sRGB, straight alpha) | 880x560 | 62 KB | blender:hero (bosch_ddu::housing_scene, Cycles 384 samples, ortho at 2 px per 4K px over x -440..440, y -320..240, box-filtered to 1:1; the housing's rig (no softbox over the face, no HDRI) at 0.6 power; body sheared 3 px for the driver's eye) seed=0; luminance p95 0.019, screen p99 0.0011 | MIT (asset-forge) | drawn at full strength under the live screen and LEDs; the cluster cross-fades the azimuths with the virtual light. Master `housing/beauty_0.png` |
| `housing_1.png` | the DDU 10 housing: anodised body, black glass face, 10 + 5 + 5 smoked lenses, light sensor; key at -160° (sRGB, straight alpha) | 880x560 | 66 KB | blender:hero (bosch_ddu::housing_scene, Cycles 384 samples, ortho at 2 px per 4K px over x -440..440, y -320..240, box-filtered to 1:1; the housing's rig (no softbox over the face, no HDRI) at 0.6 power; body sheared 3 px for the driver's eye) seed=0; luminance p95 0.016, screen p99 0.0016 | MIT (asset-forge) | drawn at full strength under the live screen and LEDs; the cluster cross-fades the azimuths with the virtual light. Master `housing/beauty_1.png` |
| `housing_2.png` | the DDU 10 housing: anodised body, black glass face, 10 + 5 + 5 smoked lenses, light sensor; key at -110° (sRGB, straight alpha) | 880x560 | 61 KB | blender:hero (bosch_ddu::housing_scene, Cycles 384 samples, ortho at 2 px per 4K px over x -440..440, y -320..240, box-filtered to 1:1; the housing's rig (no softbox over the face, no HDRI) at 0.6 power; body sheared 3 px for the driver's eye) seed=0; luminance p95 0.023, screen p99 0.0009 | MIT (asset-forge) | drawn at full strength under the live screen and LEDs; the cluster cross-fades the azimuths with the virtual light. Master `housing/beauty_2.png` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/bosch_ddu/`)

Copied by `--masters bosch_ddu`: the Blender outputs into `hero_bosch_ddu_housing/`, the ComfyUI textures into `comfyui/`, and a `*_preview.png` beside them for the tracker.

| Master | What it is |
|---|---|
| `bosch_ddu_bead_blast.png` | bead-blast macro (ComfyUI, seamless 4096 px) |
| `bosch_ddu_bead_blast_neutral_a443e36f330a.png` | bead-blast macro made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads |
| `beauty_0.png` | housing, key -135° |
| `beauty_1.png` | housing, key -160° |
| `beauty_2.png` | housing, key -110° |

5 preview(s) written.
