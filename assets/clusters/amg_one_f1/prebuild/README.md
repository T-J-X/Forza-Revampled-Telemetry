# AmgOneF1 pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild amg_one_f1` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/amg_one_f1/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Twill carbon is Blender, never SDXL.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `antiglare_matte_grain` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | amg_one_f1/prebuild/textures/antiglare_matte_grain.png | labelled micro-grain over the screen face (luminance modulation of 4% at most); the pixel grid stays procedural | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=160001 prompt="macro photograph of the anti-glare matte coating of a switched-off black display screen, fine random etched micro-grain, uniform soft frosted surface, dark charcoal grey, flat top-down view, even diffuse light, seamless tileable texture, material scan" extra negative="pixel grid, subpixels, moire, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 4.0635 |
| `carbon_twill_rim` | swatch tile (Blender, seamless, 16-bit sRGB) + _normal + _rmh | 4096x4096 | amg_one_f1/prebuild/swatches/carbon_twill_rim.png | surface of the 780 px carbon rim arc (2x2 twill, 12 px tows) | blender:hero prebuild swatch, Cycles 256 samples, ortho top-down 2016 local px to 4096 px, hero rig at 16x span key -135°, Standard view; material {"color":[0.017999999225139618,0.017999999225139618,0.019999999552965164],"kind":"carbon","name":"carbon_twill_rim","rough":0.3199999928474426,"tow":12.0} |

## Left to the dash thread (needs the Penpot layout)

- the housing, the rim arc and the F1 shift-light caps: need the Penpot layout and the cap pitch
