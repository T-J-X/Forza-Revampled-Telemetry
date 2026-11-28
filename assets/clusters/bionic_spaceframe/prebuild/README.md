# BionicSpaceframe pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild bionic_spaceframe` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/bionic_spaceframe/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `brushed_titanium_strut` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | bionic_spaceframe/prebuild/swatches/brushed_titanium_strut.png | the struts' surface (base 12% luminance at most); the truss is the dash's Michell geometry | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.85,"color":[0.5419999957084656,0.4970000088214874,0.4490000009536743],"finish":"linear","grain":0.8,"kind":"metal","name":"brushed_titanium_strut","rough":0.30000001192092896} |
