# BugattiTourbillon pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild bugatti_tourbillon` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/bugatti_tourbillon/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Horology steel; the stripe pattern waits for a library addition.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `brushed_steel_linear` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | bugatti_tourbillon/prebuild/swatches/brushed_steel_linear.png | the bridges' base albedo and roughness (20% luminance at most in the kit) | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2048 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"aniso":0.85,"color":[0.671999990940094,0.6370000243186951,0.5849999785423279],"finish":"linear","grain":0.8,"kind":"metal","name":"brushed_steel_linear","rough":0.30000001192092896} |
| `jewel_lens_30` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 42x42 at 1:1 (84x84 master) | bugatti_tourbillon/prebuild/sprites/jewel_lens_30_0.png | the 11 jewels on the 30x740 strip and the jewel bearings: clear sapphire in a polished steel setting, tinted at run time | blender:hero prebuild sprite, Cycles 256 samples, ortho square 42 local px at 2 px/px, hero rig, box-filtered to 1:1; parts cap |

## Left to the dash thread (needs the Penpot layout)

- Côtes de Genève stripes: the hero library has no stripe pattern yet (add a guilloché 'stripes' mode first)
- the 210 px tourbillon cage glTF: part of the dash's 15k-triangle budget and its motion
