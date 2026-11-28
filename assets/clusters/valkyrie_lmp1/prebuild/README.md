# ValkyrieLmp1 pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild valkyrie_lmp1` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/valkyrie_lmp1/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Forged carbon is computed (chopped tow flakes laid along a flow field, exact tile; the SDXL marble failed review); the knurl is Blender, on the ring itself.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `encoder_ring_knurled_184` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 200x200 at 1:1 (400x400 master) | valkyrie_lmp1/prebuild/sprites/encoder_ring_knurled_184_0.png | the two shoulder encoders at (±604, 4) in 4K px (0.7 scale in the hood view): one sprite, mirrored; a 3 px diamond knurl on the ring's face | blender:hero prebuild sprite, Cycles 256 samples, ortho square 200 local px at 2 px/px, hero rig, box-filtered to 1:1; parts well, knurl, cap |
| `forged_carbon_marble` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | valkyrie_lmp1/prebuild/computed/forged_carbon_marble_master.png | the yoke's forged-carbon marble (brief: albedo + normal, seamless): base colour, and a normal from its luminance | computed: prebuild::surf ForgedMarble, seed 90002: forged carbon: about 9000 chopped tow flakes (150 to 300 px long, 18 to 40 px wide, ends cut square) laid in random order and direction, bent along a slow periodic flow (the marble swirl), each with fine 6 px fibre striations along its length and its own tone 0.010 to 0.050; flat under resin (no relief); _direction = fibre direction for anisotropic sheen |

## Left to the dash thread (needs the Penpot layout)

- the 1440x420 forged-carbon yoke with angled shoulders: shape needs the Penpot board
