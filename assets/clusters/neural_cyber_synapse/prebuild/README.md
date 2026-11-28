# NeuralCyberSynapse pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild neural_cyber_synapse` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/neural_cyber_synapse/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `soma_membrane_grain` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | neural_cyber_synapse/prebuild/textures/soma_membrane_grain.png | optional modulation inside the 260 px soma (a gradient is the fallback); the nerve front is the dash's Nagumo model | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=370001 prompt="macro photograph of a translucent biological cell membrane under a microscope, soft cytoplasm granules and faint fibrous texture, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture" extra negative="cell outlines, nucleus, stain colours, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.5039 |
