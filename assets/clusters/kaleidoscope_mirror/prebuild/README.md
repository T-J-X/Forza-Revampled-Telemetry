# KaleidoscopeMirror pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild kaleidoscope_mirror` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/kaleidoscope_mirror/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `eyepiece_ring_380` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 396x396 at 1:1 (792x792 master) | kaleidoscope_mirror/prebuild/sprites/eyepiece_ring_380_0.png | the knurled black-chrome eyepiece ring round the 380 px figure (10 px width and 4 px knurl pitch proposed) | blender:hero prebuild sprite, Cycles 256 samples, ortho square 396 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
| `chip_glass_inclusions` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | kaleidoscope_mirror/prebuild/computed/chip_glass_inclusions_master.png | labelled grain map inside the eight chips; the mirror copies are the dash's dihedral group | computed: prebuild::surf SeedGlass, seed 510001: seed bubbles: in 40% of 110 px cells a ring of radius 7 to 20 px with a bright rim; faint wavy striae (stretched fbm, 4%) |
