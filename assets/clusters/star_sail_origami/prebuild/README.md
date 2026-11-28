# StarSailOrigami pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild star_sail_origami` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/star_sail_origami/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `brushed_film_hairline` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | star_sail_origami/prebuild/swatches/brushed_film_hairline.png | optional facet sheen (use its normal); the fold is the dash's Miura-ori maths | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.85,"color":[0.9129999876022339,0.921999990940094,0.9240000247955322],"finish":"linear","grain":0.8,"kind":"metal","name":"brushed_film_hairline","rough":0.30000001192092896} |
