# LorenzButterfly pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild lorenz_butterfly` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/lorenz_butterfly/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `pad_rim_powder_coat` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | lorenz_butterfly/prebuild/computed/pad_rim_powder_coat_master.png | labelled grain on the rims of the true-black pads (the gear pad and the speed pad); the attractor is the dash's RK4 | computed: prebuild::surf PowderCoat, seed 330001: soft orange peel: two-octave fbm of 40 px cells, 600 px tone drift; no flecks |
