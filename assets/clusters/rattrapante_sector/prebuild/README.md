# RattrapanteSector pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild rattrapante_sector` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/rattrapante_sector/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `spun_titanium_arc_rail` | swatch disc (Blender, 16-bit sRGB, straight alpha) + _normal + _rmh | 4096x4096 | rattrapante_sector/prebuild/swatches/spun_titanium_arc_rail.png | the arc rail's tangential grain | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.9,"color":[0.5419999957084656,0.4970000088214874,0.4490000009536743],"finish":"spun","grain":0.8,"kind":"metal","name":"spun_titanium_arc_rail","rough":0.25999999046325684} |
