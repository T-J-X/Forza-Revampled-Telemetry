# EvaBioSync pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild eva_bio_sync` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/eva_bio_sync/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `cell_nucleus_grain` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | eva_bio_sync/prebuild/textures/cell_nucleus_grain.png | optional modulation of the lit-cell gradient | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=640001 prompt="macro photograph of translucent cell nucleus material under a microscope, soft granular chromatin texture, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture" extra negative="cell outlines, hexagons, stain colours, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.5620 |
