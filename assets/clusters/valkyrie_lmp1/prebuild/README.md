# ValkyrieLmp1 pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild valkyrie_lmp1` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/valkyrie_lmp1/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Forged carbon is SDXL (chopped-fibre marble is organic); the knurl is Blender, on the ring itself. Seed 90002 replaced 90001, which drew rough coal-like chunks with beige specks, not flat resin.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `forged_carbon_marble` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | valkyrie_lmp1/prebuild/textures/forged_carbon_marble.png | the yoke's forged-carbon marble (brief: albedo + normal, seamless): base colour, and a normal from its luminance | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=90002 prompt="macro photograph of a flat polished forged carbon fibre composite panel, smooth glossy clear resin surface, thin flattened black and dark grey carbon fibre strands in a swirling marbled pattern, satin sheen on the fibres, flat top-down view, even diffuse light, seamless tileable texture, material scan" extra negative="woven, twill, checker pattern, grid, rocks, stones, coal, crumpled, rough relief, cracks, beige, brown, white specks, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 0.8514 |

## Left to the dash thread (needs the Penpot layout)

- the 1440x420 forged-carbon yoke with angled shoulders: shape needs the Penpot board
- the knurled anodised encoder rings: the knurl only reads on a lathe ring, and the brief gives no ring diameter (render them as bezel sprites with a knurled outer chamfer once the layout sets the size)
