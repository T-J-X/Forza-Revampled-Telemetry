# HyperbolicTiles pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild hyperbolic_tiles` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/hyperbolic_tiles/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `disc_rim_ring_400` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 416x416 at 1:1 (832x832 master) | hyperbolic_tiles/prebuild/sprites/disc_rim_ring_400_0.png | the 400 px thin machined rim (8 px width proposed); one render, mirrored for both discs | blender:hero prebuild sprite, Cycles 256 samples, ortho square 416 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
| `etched_glass_grain` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | hyperbolic_tiles/prebuild/computed/etched_glass_grain_master.png | labelled sparkle map on the two disc faces; the tiling is the dash's Poincaré maths | computed: prebuild::surf EtchedGlass, seed 720001: uniform frosted grain: Worley soft bumps of 14 px cells, luminance 0.5 +- 0.03 |
