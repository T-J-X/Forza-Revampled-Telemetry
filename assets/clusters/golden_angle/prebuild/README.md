# GoldenAngle pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild golden_angle` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/golden_angle/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `seed_bead_26` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 38x38 at 1:1 (76x76 master) | golden_angle/prebuild/sprites/seed_bead_26_0.png | the 26 px domed crystal bead; one sprite serves every seed | blender:hero prebuild sprite, Cycles 256 samples, ortho square 38 local px at 2 px/px, hero rig, box-filtered to 1:1; parts cap |
| `bead_inner_sparkle` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | golden_angle/prebuild/computed/bead_inner_sparkle_master.png | labelled sparkle grain inside the bead; the seed spiral is the dash's Vogel model | computed: prebuild::surf Sparkle, seed 600001: sparse glints: a soft Gaussian flake in 35% of 42 px Worley cells, brightness r^3, 3 to 5 px wide |
