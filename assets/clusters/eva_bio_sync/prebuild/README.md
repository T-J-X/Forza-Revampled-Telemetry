# EvaBioSync pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild eva_bio_sync` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/eva_bio_sync/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `cell_nucleus_grain` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | eva_bio_sync/prebuild/computed/cell_nucleus_grain_master.png | optional modulation of the lit-cell gradient | computed: prebuild::surf Chromatin, seed 640001: soft granules (26 px Worley, varied) over 400 px fbm; grey 0.35 to 0.6, no outlines |
