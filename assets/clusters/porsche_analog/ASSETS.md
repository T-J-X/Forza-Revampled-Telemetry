# PorscheAnalog assets

Built by `cargo run --release -p asset-forge -- --masters porsche_analog build --set porsche_analog` (recipe `tools/asset-forge/src/recipes/porsche_analog.rs`, hero library `tools/asset-forge/src/hero/`). The cluster draws a procedural fallback for every file that is missing.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: rendered into the tube sprites) | seamless dial texture | 4096x4096 | master | comfyui:porsche_analog::DIAL_GRAIN (SDXL base 1.0 + refiner when installed, 1024², dpmpp_2m karras, half-offset seam repaint at denoise 0.55, 4x-UltraSharp) seed=70701 prompt="extreme macro photograph of matte black powder coat paint, fine uniform orange peel texture with tiny soft pigment flecks, a completely flat painted surface filling the whole frame, top-down, even soft diffuse light, seamless tileable texture, physically based material scan" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the colour and relief of every dial face in the Blender scenes (one repeat per 768 px). Master `porsche_analog_dial_grain_neutral_580672d04e70.png` |
| `tach_0.png` | rev counter: graphite-titanium bezel with the shift ring's 11 smoked lenses, raised scale ring, darker spun inner zone; key at -135° (sRGB, straight alpha) | 512x512 | 158 KB | blender:hero (porsche_analog::tach, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `tach/beauty_0.png` |
| `tach_1.png` | rev counter: graphite-titanium bezel with the shift ring's 11 smoked lenses, raised scale ring, darker spun inner zone; key at -160° (sRGB, straight alpha) | 512x512 | 161 KB | blender:hero (porsche_analog::tach, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `tach/beauty_1.png` |
| `tach_2.png` | rev counter: graphite-titanium bezel with the shift ring's 11 smoked lenses, raised scale ring, darker spun inner zone; key at -110° (sRGB, straight alpha) | 512x512 | 156 KB | blender:hero (porsche_analog::tach, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `tach/beauty_2.png` |
| `side_0.png` | side tube (speed and G): graphite-titanium bezel, raised scale ring, dial well; key at -135° (sRGB, straight alpha) | 384x384 | 93 KB | blender:hero (porsche_analog::side, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `side/beauty_0.png` |
| `side_1.png` | side tube (speed and G): graphite-titanium bezel, raised scale ring, dial well; key at -160° (sRGB, straight alpha) | 384x384 | 94 KB | blender:hero (porsche_analog::side, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `side/beauty_1.png` |
| `side_2.png` | side tube (speed and G): graphite-titanium bezel, raised scale ring, dial well; key at -110° (sRGB, straight alpha) | 384x384 | 92 KB | blender:hero (porsche_analog::side, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `side/beauty_2.png` |
| `far_0.png` | outer tube (power and boost): graphite-titanium bezel and dial; key at -135° (sRGB, straight alpha) | 232x232 | 41 KB | blender:hero (porsche_analog::far, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `far/beauty_0.png` |
| `far_1.png` | outer tube (power and boost): graphite-titanium bezel and dial; key at -160° (sRGB, straight alpha) | 232x232 | 42 KB | blender:hero (porsche_analog::far, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `far/beauty_1.png` |
| `far_2.png` | outer tube (power and boost): graphite-titanium bezel and dial; key at -110° (sRGB, straight alpha) | 232x232 | 41 KB | blender:hero (porsche_analog::far, Cycles 256 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `far/beauty_2.png` |
| `hub_0.png` | the satin black-chrome domed hub cap over the needle's pivot; 48 px square; key at -135° (sRGB, straight alpha) | 48x48 | 3 KB | blender:hero (porsche_analog::hub, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `hub/beauty_0.png` |
| `hub_1.png` | the satin black-chrome domed hub cap over the needle's pivot; 48 px square; key at -160° (sRGB, straight alpha) | 48x48 | 3 KB | blender:hero (porsche_analog::hub, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `hub/beauty_1.png` |
| `hub_2.png` | the satin black-chrome domed hub cap over the needle's pivot; 48 px square; key at -110° (sRGB, straight alpha) | 48x48 | 3 KB | blender:hero (porsche_analog::hub, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `hub/beauty_2.png` |
| `window_0.png` | the gear window's machined graphite-titanium rim; 164 px square; key at -135° (sRGB, straight alpha) | 164x164 | 11 KB | blender:hero (porsche_analog::window, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `window/beauty_0.png` |
| `window_1.png` | the gear window's machined graphite-titanium rim; 164 px square; key at -160° (sRGB, straight alpha) | 164x164 | 11 KB | blender:hero (porsche_analog::window, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `window/beauty_1.png` |
| `window_2.png` | the gear window's machined graphite-titanium rim; 164 px square; key at -110° (sRGB, straight alpha) | 164x164 | 11 KB | blender:hero (porsche_analog::window, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `window/beauty_2.png` |
| `needle_0.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at -135° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_0.png` |
| `needle_1.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at -90° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_1.png` |
| `needle_2.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at -45° (sRGB, straight alpha) | 24x236 | 2 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_2.png` |
| `needle_3.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at 0° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_3.png` |
| `needle_4.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at 45° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_4.png` |
| `needle_5.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at 90° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_5.png` |
| `needle_6.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at 135° (sRGB, straight alpha) | 24x236 | 2 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_6.png` |
| `needle_7.png` | rev counter needle, white lacquer (tinted by the cluster), pointing up from the pivot; frame x -12..12, y -206..30 round the pivot; key at 180° (sRGB, straight alpha) | 24x236 | 1 KB | blender:hero (porsche_analog::needle, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `needle/beauty_7.png` |
| `pointer_0.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at -135° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_0.png` |
| `pointer_1.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at -90° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_1.png` |
| `pointer_2.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at -45° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_2.png` |
| `pointer_3.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at 0° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_3.png` |
| `pointer_4.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at 45° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_4.png` |
| `pointer_5.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at 90° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_5.png` |
| `pointer_6.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at 135° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_6.png` |
| `pointer_7.png` | side-tube pointer, white lacquer (tinted), pointing up from the pivot; frame x -8..8, y -160..-122 round the pivot; key at 180° (sRGB, straight alpha) | 16x38 | 0 KB | blender:hero (porsche_analog::pointer, Cycles 192 samples, ortho at 2 px per 4K px, box-filtered to 1:1; hero rig) | MIT (asset-forge) | drawn at full strength; the cluster cross-fades the azimuths with the virtual light (and, for a needle, with its angle). Master `pointer/beauty_7.png` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/porsche_analog/`)

Copied by `--masters porsche_analog`: the Blender outputs into one sub-folder per job (`hero_porsche_analog_<stem>`), the ComfyUI master into `comfyui/`, and a `*_preview.png` beside each for the tracker.

| Master | What it is |
|---|---|
| `porsche_analog_dial_grain.png` | dial grain (ComfyUI, seamless 4096 px) |
| `porsche_analog_dial_grain_neutral_580672d04e70.png` | dial grain made neutral (luminance only, mid-grey mean, fixed spread): the file Blender reads |
| `beauty_0.png` | tach, key -135° |
| `beauty_1.png` | tach, key -160° |
| `beauty_2.png` | tach, key -110° |
| `normal.png` | tach, tangent-space normal (16-bit) |
| `height.png` | tach, height (16-bit) |
| `ao.png` | tach, ambient occlusion (16-bit) |
| `beauty_0.png` | side, key -135° |
| `beauty_1.png` | side, key -160° |
| `beauty_2.png` | side, key -110° |
| `normal.png` | side, tangent-space normal (16-bit) |
| `height.png` | side, height (16-bit) |
| `ao.png` | side, ambient occlusion (16-bit) |
| `beauty_0.png` | far, key -135° |
| `beauty_1.png` | far, key -160° |
| `beauty_2.png` | far, key -110° |
| `normal.png` | far, tangent-space normal (16-bit) |
| `height.png` | far, height (16-bit) |
| `ao.png` | far, ambient occlusion (16-bit) |
| `beauty_0.png` | hub, key -135° |
| `beauty_1.png` | hub, key -160° |
| `beauty_2.png` | hub, key -110° |
| `normal.png` | hub, tangent-space normal (16-bit) |
| `height.png` | hub, height (16-bit) |
| `beauty_0.png` | window, key -135° |
| `beauty_1.png` | window, key -160° |
| `beauty_2.png` | window, key -110° |
| `normal.png` | window, tangent-space normal (16-bit) |
| `height.png` | window, height (16-bit) |
| `beauty_0.png` | needle, key -135° |
| `beauty_1.png` | needle, key -90° |
| `beauty_2.png` | needle, key -45° |
| `beauty_3.png` | needle, key 0° |
| `beauty_4.png` | needle, key 45° |
| `beauty_5.png` | needle, key 90° |
| `beauty_6.png` | needle, key 135° |
| `beauty_7.png` | needle, key 180° |
| `normal.png` | needle, tangent-space normal (16-bit) |
| `height.png` | needle, height (16-bit) |
| `beauty_0.png` | pointer, key -135° |
| `beauty_1.png` | pointer, key -90° |
| `beauty_2.png` | pointer, key -45° |
| `beauty_3.png` | pointer, key 0° |
| `beauty_4.png` | pointer, key 45° |
| `beauty_5.png` | pointer, key 90° |
| `beauty_6.png` | pointer, key 135° |
| `beauty_7.png` | pointer, key 180° |
| `normal.png` | pointer, tangent-space normal (16-bit) |
| `height.png` | pointer, height (16-bit) |

50 preview(s) written.
