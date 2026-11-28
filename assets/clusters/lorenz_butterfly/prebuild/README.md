# LorenzButterfly pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild lorenz_butterfly` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/lorenz_butterfly/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `pad_rim_powder_coat` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | lorenz_butterfly/prebuild/textures/pad_rim_powder_coat.png | labelled grain on the rims of the true-black pads (the gear pad and the speed pad); the attractor is the dash's RK4 | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=330001 prompt="macro photograph of matte black powder-coat paint, very fine uniform grain with tiny soft flecks, near black, flat top-down view, even diffuse light, seamless tileable texture" extra negative="orange peel bumps, scratches, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.6691 |
