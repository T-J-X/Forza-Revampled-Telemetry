# RevueltoCorsa pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild revuelto_corsa` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/revuelto_corsa/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.



| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `forged_composite_large_chips` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | revuelto_corsa/prebuild/textures/forged_composite_large_chips.png | the hexagon pod body | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=560001 prompt="macro photograph of forged carbon composite with large flat chopped carbon flakes, broad overlapping chips under glossy resin, black and graphite grey with soft fibre sheen, flat top-down view, even diffuse light, seamless tileable texture, material scan" extra negative="woven, twill, grid, checker pattern, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.5517 |

## Left to the dash thread (needs the Penpot layout)

- the hex-mesh grille: the hero library has no hexagon perforation yet
- the 300 px hexagon pod: the library has no hexagon outline yet, and the anodise tint is not in the brief
