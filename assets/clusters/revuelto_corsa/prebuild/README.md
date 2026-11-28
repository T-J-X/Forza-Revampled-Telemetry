# RevueltoCorsa pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild revuelto_corsa` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/revuelto_corsa/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `forged_composite_large_chips` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | revuelto_corsa/prebuild/computed/forged_composite_large_chips_master.png | the hexagon pod body | computed: prebuild::surf ForgedChips, seed 560001: forged carbon: about 1100 broad chopped flakes (380 to 700 px long, 110 to 200 px wide) laid in random order and direction, each with fine 6 px fibre striations along its length and its own tone 0.010 to 0.050; flat under resin; _direction map |

## Left to the dash thread (needs the Penpot layout)

- the hex-mesh grille: the hero library has no hexagon perforation yet
- the 300 px hexagon pod: the library has no hexagon outline yet, and the anodise tint is not in the brief
