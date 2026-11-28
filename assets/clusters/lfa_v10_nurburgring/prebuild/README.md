# LfaV10Nurburgring pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild lfa_v10_nurburgring` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/lfa_v10_nurburgring/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `spun_titanium` | swatch disc (Blender, 16-bit sRGB, straight alpha) + _normal + _rmh | 4096x4096 | lfa_v10_nurburgring/prebuild/swatches/spun_titanium.png | the ring's circular grain and its tangential anisotropy | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.9,"color":[0.5419999957084656,0.4970000088214874,0.4490000009536743],"finish":"spun","grain":0.8,"kind":"metal","name":"spun_titanium","rough":0.25999999046325684} |
| `titanium_ring_480` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 496x496 at 1:1 (992x992 master) | lfa_v10_nurburgring/prebuild/sprites/titanium_ring_480_0.png | the floating 480 px titanium ring, 24 px wide (r_in 216), no face or pivot | blender:hero prebuild sprite, Cycles 256 samples, ortho square 496 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
