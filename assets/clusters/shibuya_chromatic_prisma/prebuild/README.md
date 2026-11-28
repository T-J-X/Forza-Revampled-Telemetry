# ShibuyaChromaticPrisma pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild shibuya_chromatic_prisma` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/shibuya_chromatic_prisma/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

The prism dispersion is the dash's optics; this is only the wet street grain.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `wet_asphalt_ripple` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | shibuya_chromatic_prisma/prebuild/computed/wet_asphalt_ripple_master.png | greyscale height for the mirror-ripple normal map | computed: prebuild::surf WetAsphalt, seed 260001: aggregate: Worley stones of 16 px, tones 0.040 to 0.060 (dark, low contrast); height = 35% stones + 65% shallow 260 px ripples |
