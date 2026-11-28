# KoenigseggGyro pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild koenigsegg_gyro` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/koenigsegg_gyro/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `carbon_twill_12` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | koenigsegg_gyro/prebuild/swatches/carbon_twill_12.png | the rotating ring band | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2016 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"color":[0.017999999225139618,0.017999999225139618,0.019999999552965164],"kind":"carbon","name":"carbon_twill_12","rough":0.3199999928474426,"tow":12.0} |
| `brushed_titanium` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | koenigsegg_gyro/prebuild/swatches/brushed_titanium.png | the inner rim | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.85,"color":[0.5419999957084656,0.4970000088214874,0.4490000009536743],"finish":"linear","grain":0.8,"kind":"metal","name":"brushed_titanium","rough":0.30000001192092896} |
| `instrument_ring_540` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 556x556 at 1:1 (1112x1112 master) | koenigsegg_gyro/prebuild/sprites/instrument_ring_540_0.png | the 540 px instrument ring: carbon band, brushed titanium rim, polished chamfers (30 px width proposed); the kit turns it with steering | blender:hero prebuild sprite, Cycles 256 samples, ortho square 556 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
