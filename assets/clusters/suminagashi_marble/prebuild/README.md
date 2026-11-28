# SuminagashiMarble pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild suminagashi_marble` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/suminagashi_marble/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `urushi_lacquer` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | suminagashi_marble/prebuild/textures/urushi_lacquer.png | the tray rim's lacquer | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=420001 prompt="macro photograph of polished black urushi lacquer, deep glossy black with very faint brush strokes and fine fibrous undercoat grain, flat top-down view, even diffuse light, seamless tileable texture" extra negative="gold leaf, maki-e, decoration, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.0075 |
| `still_water_sheen` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | suminagashi_marble/prebuild/textures/still_water_sheen.png | greyscale height for the water's faint sheen normal; the marbling is the dash's area-preserving maps | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=420002 prompt="macro photograph of the still surface of dark water, extremely faint shallow ripples and micro capillary waves, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture" extra negative="reflections, ink, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 7.0452 |
| `washi_paper_fibre` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | suminagashi_marble/prebuild/textures/washi_paper_fibre.png | labelled fibre texture under the labels | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=420003 prompt="macro photograph of handmade white washi paper, long translucent kozo fibres in a thin sheet, flat top-down view, even diffuse light, seamless tileable texture" extra negative="writing, ink, folds, stains, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 8.5147 |

## Left to the dash thread (needs the Penpot layout)

- the gear stone (200 px) and the side stones (260x130): pebble geometry is modelled with the layout
