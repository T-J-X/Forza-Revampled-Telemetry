# SkylineR34Mfd pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild skyline_r34_mfd` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/skyline_r34_mfd/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Period LCD and plastics.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `lcd_backlight_mura` | texture (ComfyUI, seamless, sRGB) | 4096x4096 | skyline_r34_mfd/prebuild/textures/lcd_backlight_mura.png | modulation layer for the gear and speed LCD windows and the MFD tiles; the pixel grid stays procedural | comfyui hero::texture::make_texture (SDXL base 1.0 + refiner, 1024², half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=120001 prompt="macro photograph of a dim late-1990s liquid crystal display panel, soft cloudy backlight mura and faint uneven brightness, very fine diffuser grain, dark grey-green, flat top-down view, even diffuse light, seamless tileable texture" extra negative="text, pixel grid, numerals, segments, objects, lighting falloff, perspective, seams, blur, cartoon", tile seam score 6.7470 |
| `tach_ring_440` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 456x456 at 1:1 (912x912 master) | skyline_r34_mfd/prebuild/sprites/tach_ring_440_0.png | the 440 px tach ring: matte period black plastic with a polished inner chamfer (14 px width proposed; step 1 checks the R34 ring) | blender:hero prebuild sprite, Cycles 256 samples, ortho square 456 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |

## Left to the dash thread (needs the Penpot layout)

- the MFD tile housings and the dial-face print: layout and text
