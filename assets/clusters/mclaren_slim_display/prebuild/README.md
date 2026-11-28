# MclarenSlimDisplay pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild mclaren_slim_display` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/mclaren_slim_display/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `antiglare_matte_grain` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | mclaren_slim_display/prebuild/computed/antiglare_matte_grain_master.png | labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural | computed: prebuild::surf Antiglare, seed 160001: Worley soft bumps of 22 px cells over a 256 px low swell; luminance 0.5 +- 0.04 |
| `alcantara_black` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | mclaren_slim_display/prebuild/computed/alcantara_black_master.png | the dash top the slab folds into (alcantara(...).set("texture")) | computed: prebuild::surf Alcantara, seed 650002: microfibre nap: 18 px noise stretched 5:1 along x (+-6%), 700 px sheen drift (+-5%); low contrast |
