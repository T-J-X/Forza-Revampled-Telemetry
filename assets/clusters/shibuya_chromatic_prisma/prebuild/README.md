# ShibuyaChromaticPrisma pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild shibuya_chromatic_prisma` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/shibuya_chromatic_prisma/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

The prism dispersion is the dash's optics; this is only the wet street grain.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `wet_asphalt_ripple` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | shibuya_chromatic_prisma/prebuild/textures/wet_asphalt_ripple.png | greyscale height for the mirror-ripple normal map | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=260001 prompt="macro photograph of wet dark asphalt road surface under a thin film of rain water, fine aggregate grain and shallow puddle ripples, greyscale, flat top-down view, even diffuse light, seamless tileable texture" extra negative="road markings, reflections of objects, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 2.7619 |
