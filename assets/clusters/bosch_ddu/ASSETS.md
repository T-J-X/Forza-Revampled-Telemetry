# BoschDdu assets

Built by `cargo run --release -p asset-forge -- --lane cloud --masters bosch_ddu build --set bosch_ddu` (recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws its procedural housing when the files are missing. The housing renders cover the runtime's `HOUSING_FRAME`, x −440..440, y −320..240 in local 4K px, at 1 px per px; `housing_1` is the key at the smaller azimuth, as the runtime blends it.

**Not built yet.** The recipe was rewritten for the rebuilt dash (the DDU 10 housing) and has not run on a GPU lane since. The rows below are what the next build writes; it rewrites this whole file with the real sizes, byte counts and brightness figures.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into the housing) | seamless bead-blast texture | 4096x4096 | master | comfyui:bosch_ddu::BEAD_BLAST (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=71207, then neutral (luminance, mean 0.5, deviation 0.08 over a 48 px window) | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the anodised body's colour and roughness variation and its faint bump (one repeat per 640 px) |
| `housing_0.png` | the DDU 10 housing: anodised body, black glass face, 10 + 5 + 5 smoked lenses, light sensor; key at -135° (sRGB, straight alpha) | 880x560 | not built yet | blender:hero (bosch_ddu::housing_scene, Cycles 384 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig at 0.6 power; body sheared 3 px for the driver's eye) seed=0 | MIT (asset-forge) | drawn at full strength under the live screen and LEDs; the rest render |
| `housing_1.png` | as `housing_0.png`; key at -160° | 880x560 | not built yet | as `housing_0.png` | MIT (asset-forge) | cross-faded in when the virtual light swings to the smaller angle |
| `housing_2.png` | as `housing_0.png`; key at -110° | 880x560 | not built yet | as `housing_0.png` | MIT (asset-forge) | cross-faded in when the virtual light swings to the larger angle |
