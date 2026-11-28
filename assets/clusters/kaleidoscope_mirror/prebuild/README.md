# KaleidoscopeMirror pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild kaleidoscope_mirror` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/kaleidoscope_mirror/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `chip_glass_inclusions` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | kaleidoscope_mirror/prebuild/textures/chip_glass_inclusions.png | labelled grain map inside the eight chips; the mirror copies are the dash's dihedral group | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=510001 prompt="macro photograph of clear antique glass with tiny seed bubbles and faint wavy striae, monochrome grey on black, flat top-down view, even diffuse light, seamless tileable texture" extra negative="cracks, chips, colour, edges, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.4985 |
| `eyepiece_ring_380` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 396x396 at 1:1 (792x792 master) | kaleidoscope_mirror/prebuild/sprites/eyepiece_ring_380_0.png | the knurled black-chrome eyepiece ring round the 380 px figure (10 px width and 4 px knurl pitch proposed) | blender:hero prebuild sprite, Cycles 256 samples, ortho square 396 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
