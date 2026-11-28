# GtrNismoMfd pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild gtr_nismo_mfd` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/gtr_nismo_mfd/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Twill carbon is Blender, not SDXL.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `carbon_twill_surround` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | gtr_nismo_mfd/prebuild/swatches/carbon_twill_surround.png | the meter surround (2x2 twill) | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2080 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"color":[0.017999999225139618,0.017999999225139618,0.019999999552965164],"kind":"carbon","name":"carbon_twill_surround","rough":0.3199999928474426,"tow":10.0} |
| `tach_ring_460` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 476x476 at 1:1 (952x952 master) | gtr_nismo_mfd/prebuild/sprites/tach_ring_460_0.png | the 460 px chronograph-style tach ring: machined alloy top, polished chamfers (20 px width proposed; step 1 checks the R35 ring) | blender:hero prebuild sprite, Cycles 256 samples, ortho square 476 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
