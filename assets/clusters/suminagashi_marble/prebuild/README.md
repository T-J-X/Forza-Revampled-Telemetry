# SuminagashiMarble pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild suminagashi_marble` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/suminagashi_marble/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `urushi_lacquer` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | suminagashi_marble/prebuild/computed/urushi_lacquer_master.png | the tray rim's lacquer | computed: prebuild::surf Urushi, seed 420001: deep black with a warm undertone: 1200 px depth +-15%, brush drag stretched 10:1 at +-3% |
| `still_water_sheen` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | suminagashi_marble/prebuild/computed/still_water_sheen_master.png | greyscale height for the water's faint sheen normal; the marbling is the dash's area-preserving maps | computed: prebuild::surf StillWater, seed 420002: nine sine waves with whole wave numbers (exact tile), plus 90 px capillary fbm at 15% |
| `washi_paper_fibre` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | suminagashi_marble/prebuild/computed/washi_paper_fibre_master.png | labelled fibre texture under the labels | computed: prebuild::surf Washi, seed 420003: cloudy formation (300 px fbm, +-3%) with 2400 long kozo fibres (300 to 900 px, 1.5 to 3 px wide, translucent +7%) |

## Left to the dash thread (needs the Penpot layout)

- the gear stone (200 px) and the side stones (260x130): pebble geometry is modelled with the layout
