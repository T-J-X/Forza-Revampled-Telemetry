# HyperbolicTiles pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild hyperbolic_tiles` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/hyperbolic_tiles/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `etched_glass_grain` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | hyperbolic_tiles/prebuild/textures/etched_glass_grain.png | labelled sparkle map on the two disc faces; the tiling is the dash's Poincaré maths | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=720001 prompt="macro photograph of finely acid-etched frosted glass, uniform fine matte grain, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture" extra negative="scratches, fingerprints, patterns, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.1423 |
| `disc_rim_ring_400` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 416x416 at 1:1 (832x832 master) | hyperbolic_tiles/prebuild/sprites/disc_rim_ring_400_0.png | the 400 px thin machined rim (8 px width proposed); one render, mirrored for both discs | blender:hero prebuild sprite, Cycles 256 samples, ortho square 416 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
