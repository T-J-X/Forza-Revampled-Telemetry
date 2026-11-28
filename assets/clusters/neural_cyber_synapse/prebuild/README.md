# NeuralCyberSynapse pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild neural_cyber_synapse` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/neural_cyber_synapse/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `soma_membrane_grain` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | neural_cyber_synapse/prebuild/computed/soma_membrane_grain_master.png | optional modulation inside the 260 px soma (a gradient is the fallback); the nerve front is the dash's Nagumo model | computed: prebuild::surf Membrane, seed 370001: faint fibres (60 px noise stretched 4:1, warped) with soft granules of 34 px; grey 0.38 to 0.6 |
