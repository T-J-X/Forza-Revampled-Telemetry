# ChronoEventHorizon pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild chrono_event_horizon` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/chrono_event_horizon/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Only the disk's material grain; the black-hole optics are the dash's maths.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `accretion_shear_filaments` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | chrono_event_horizon/prebuild/textures/accretion_shear_filaments.png | greyscale shear texture wrapped polar onto the 1100x190 disk and tinted by the dash; the lensing itself is the dash's deflection table, never this texture | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=190001 prompt="macro photograph of fine sheared luminous smoke filaments all streaming in one horizontal direction, turbulent wisps of varied brightness on black, monochrome grey, flat top-down view, even diffuse light, seamless tileable texture" extra negative="vortex, spiral, galaxy, stars, text, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 1.2458 |
