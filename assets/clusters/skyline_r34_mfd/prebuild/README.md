# SkylineR34Mfd pre-built assets

Made before the dash thread by `cargo run --release -p asset-forge -- --lane cloud prebuild skyline_r34_mfd` (kit in `tools/asset-forge/src/prebuild/kits.rs`). These are layout-independent: the 4K masters are in the Drive folder `FH6 WheelDash v2 - 4K Masters/skyline_r34_mfd/prebuild/`, and `manifest.json` lists every file. Sprites are also here at 1:1 in `sprites/`, and computed maps (environment, matcaps, look-up tables) at their runtime size in `computed/`. Textures and swatches have no runtime copy yet: the dash's own recipe downsizes the master it picks to the size its Penpot layout needs.

Period LCD and plastics.

| Name | Kind | Size | Master (Drive) | Use | Generator |
|---|---|---|---|---|---|
| `tach_ring_440` | sprite (Blender, sRGB, straight alpha; _0 rest key -135°, _1/_2 swing -160°/-110°) | 456x456 at 1:1 (912x912 master) | skyline_r34_mfd/prebuild/sprites/tach_ring_440_0.png | the 440 px tach ring: matte period black plastic with a polished inner chamfer (14 px width proposed; step 1 checks the R34 ring) | blender:hero prebuild sprite, Cycles 256 samples, ortho square 456 local px at 2 px/px, hero rig, box-filtered to 1:1; parts ring |
| `lcd_backlight_mura` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | skyline_r34_mfd/prebuild/computed/lcd_backlight_mura_master.png | modulation layer for the gear and speed LCD windows and the MFD tiles; the pixel grid stays procedural | computed: prebuild::surf LcdMura, seed 120001: three-octave fbm, 1400 px cells: soft backlight unevenness +-15% on dark grey-green, no grain |
| `diagonal_wear_streaks` | surface texture (computed, seamless): colour 16-bit sRGB, plus _height (16-bit linear), _normal (tangent, green up) and _direction where the material has them; masters only, the dash downsizes | 4096x4096 master | skyline_r34_mfd/prebuild/computed/diagonal_wear_streaks_master.png | optional: faint wear over the MFD glass, fine streaks top left to bottom right at very low contrast (thread request, 28 Nov) | computed: prebuild::surf WearStreaks, seed 120002: fine wear streaks top left to bottom right: periodic fbm stretched along the diagonal (10 px across, about 680 px along, whole periods so it tiles), luminance 0.5 +- 0.03 |

## Left to the dash thread (needs the Penpot layout)

- the MFD tile housings and the dial-face print: layout and text
