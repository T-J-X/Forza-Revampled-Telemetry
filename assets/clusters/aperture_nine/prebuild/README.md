# ApertureNine pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild aperture_nine` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/aperture_nine/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `brushed_black_titanium_blade` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | aperture_nine/prebuild/swatches/brushed_black_titanium_blade.png | the iris blades' anisotropic lobe (18% luminance at most); the iris geometry is the dash's nine-disc maths | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.85,"coat":1.0,"coat_rough":0.2,"color":[0.05000000074505806,0.05000000074505806,0.054999999701976776],"finish":"linear","grain":0.8,"kind":"metal","name":"brushed_black_titanium_blade","rough":0.30000001192092896} |
