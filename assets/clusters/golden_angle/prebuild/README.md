# GoldenAngle pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild golden_angle` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/golden_angle/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `bead_inner_sparkle` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | golden_angle/prebuild/textures/bead_inner_sparkle.png | labelled sparkle grain inside the bead; the seed spiral is the dash's Vogel model | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=600001 prompt="macro photograph of tiny sparkling inclusions inside clear crystal, sparse bright glints and fine internal facets on black, monochrome, flat top-down view, even diffuse light, seamless tileable texture" extra negative="gemstones, shapes, colour, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.3759 |
| `seed_bead_26` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 38x38 at 1:1 (76x76 master) | golden_angle/prebuild/sprites/seed_bead_26_0.png | the 26 px domed crystal bead; one sprite serves every seed | blender:hero prebuild sprite, Cycles 256 samples, ortho square 38 local px at 2 px/px, hero rig, box-filtered to 1:1; parts cap |
