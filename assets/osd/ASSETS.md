# OSD assets

Built by `cargo run --release -p asset-forge -- --lane cloud --masters osd build --set osd` (recipe `tools/asset-forge/src/recipes/osd.rs`). The driving-position view of the Fit & camera tab: the OSD draws these layers back to front (dash, wheel, seat), moves the seat with the seat height and the wheel with its offset, and tints a rim pass with the accent while its row has focus. With any file missing it draws the vector schematic instead.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into the seat and the wheel) | seamless alcantara texture | 4096x4096 | master | comfyui:osd::NAP (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=90101 prompt="extreme macro photograph of black alcantara suede microfibre upholstery, dense soft nap with faint directional brushing and gentle light and dark patches where the nap lies differently, a completely flat fabric surface filling the whole frame, top-down, even soft diffuse light, seamless tileable texture, physically based material scan" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the upholstery of the seat's quilted centre and the wheel's rim (one repeat per 0.24 m). Master `comfyui/osd_alcantara_neutral_6151636539c3.png` |
| `drive_dash.png` | the dash cut away at the driver's side (satin black, the cut a lighter satin), fixed; the full studio light (sRGB, straight alpha) | 388x338 | 21 KB | blender:osd::DRIVE (Cycles 256 samples, OptiX denoise, orthographic at yaw 24° pitch 8°, 2200 px per m box-filtered to 550) seed=90211 | MIT (asset-forge) | drawn by the OSD at (552, 152) from the view's top-left. Master `osd_drive/dash.png` |
| `drive_wheel.png` | the D-shaped wheel (alcantara rim, carbon spokes) and its column, cut where it enters the dash; moves with the wheel offset; the full studio light (sRGB, straight alpha) | 276x240 | 23 KB | blender:osd::DRIVE (Cycles 256 samples, OptiX denoise, orthographic at yaw 24° pitch 8°, 2200 px per m box-filtered to 550) seed=90211 | MIT (asset-forge) | drawn by the OSD at (376, 194) from the view's top-left. Master `osd_drive/wheel.png` |
| `drive_wheel_rim.png` | the D-shaped wheel (alcantara rim, carbon spokes) and its column, cut where it enters the dash; moves with the wheel offset; the rim light alone, white (the OSD tints it) (sRGB, straight alpha) | 276x240 | 24 KB | blender:osd::DRIVE (Cycles 256 samples, OptiX denoise, orthographic at yaw 24° pitch 8°, 2200 px per m box-filtered to 550) seed=90211 | MIT (asset-forge) | drawn by the OSD at (376, 194) from the view's top-left. Master `osd_drive/wheel_rim.png` |
| `drive_seat.png` | the carbon bucket seat (quilted alcantara centre, leather bolsters); moves with the seat height; the full studio light (sRGB, straight alpha) | 492x572 | 101 KB | blender:osd::DRIVE (Cycles 256 samples, OptiX denoise, orthographic at yaw 24° pitch 8°, 2200 px per m box-filtered to 550) seed=90211 | MIT (asset-forge) | drawn by the OSD at (0, 0) from the view's top-left. Master `osd_drive/seat.png` |
| `drive_seat_rim.png` | the carbon bucket seat (quilted alcantara centre, leather bolsters); moves with the seat height; the rim light alone, white (the OSD tints it) (sRGB, straight alpha) | 492x572 | 57 KB | blender:osd::DRIVE (Cycles 256 samples, OptiX denoise, orthographic at yaw 24° pitch 8°, 2200 px per m box-filtered to 550) seed=90211 | MIT (asset-forge) | drawn by the OSD at (0, 0) from the view's top-left. Master `osd_drive/seat_rim.png` |

The view is 957x572 px at 550 px per m (orthographic, yaw 24°, pitch 8°). The driver's eye at seat height 0, (-0.02, 0, 0.66) m from the H-point, lands at (193.45, 151.41) px; `crates/fh6-osd/src/drive.rs` mirrors these numbers.

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/osd/`)

Copied by `--masters osd`: the Blender outputs into `osd_drive/` (16-bit PNG, 4x the OSD's scale), both textures into `comfyui/` (the ComfyUI lane also leaves its raw output there under ComfyUI's own name the first time it runs), and a `*_preview.png` beside each for the tracker.

| Master | What it is |
|---|---|
| `comfyui/osd_alcantara.png` | alcantara (ComfyUI, seamless 4096 px) |
| `comfyui/osd_alcantara_neutral_6151636539c3.png` | alcantara made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads |
| `osd_drive/dash.png` | dash, the full studio light |
| `osd_drive/wheel.png` | wheel, the full studio light |
| `osd_drive/wheel_rim.png` | wheel, the rim light alone, white (the OSD tints it) |
| `osd_drive/seat.png` | seat, the full studio light |
| `osd_drive/seat_rim.png` | seat, the rim light alone, white (the OSD tints it) |
| `osd_drive/view.png` | the whole driving position at seat height 0 and wheel offset 0 (the hero still) |

8 preview(s) written.
