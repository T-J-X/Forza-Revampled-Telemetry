# Vision Spatial Glass assets

Built by `cargo run --release -p asset-forge -- --lane cloud build --set vision_spatial_glass` (recipe `tools/asset-forge/src/recipes/vision_spatial_glass.rs`). The glass is computed by the kit (`docs/specs/optics.md`); this map is what it reflects. The cluster uses the kit's analytic studio when the file is missing.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: baked into `cabin_env.png`) | cabin panorama, seamless left to right | 4096x2048 | 6468 KB (master) | comfyui:vision_spatial_glass::cabin_plate (SDXL base 1.0 + refiner when installed, 1408x704, dpmpp_2m karras 32 steps cfg 6, seam column repainted at denoise 0.55, 4x-UltraSharp) seed=102203 prompt="360 degree equirectangular panorama from the driver's seat inside a dark modern electric sports car cabin at night, black leather and dark suede, a sweeping windscreen with distant soft city lights and a dusk sky, slim A-pillars, a few thin cool ambient light strips along the doors and dashboard, glossy black trim with faint reflections, low key, cinematic, photographic, wide panorama" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the picture on the cyclorama of the Blender cabin studio: the windscreen, pillars and ambient strips that the glass rims mirror. Master `cabin_plate_master.png` |
| `cabin_env.png` | environment, octahedral (z at the viewer in the centre), sRGB, white = radiance 16 | 4096x4096 | 1056 KB | blender:liquid_glass::STUDIO run by vision_spatial_glass (Cycles, 6 faces 4096 px, fov 94°, 256 samples, seed=102301): horizon ring 0..9° (radiance 0.9 to 5.9 towards the key), key strip 74x9° radiance 8, strip 105x7° radiance 4.7, cyclorama with the ComfyUI cabin plate (softened by a 6 px Gaussian) at radiance 0.3; resampled to octahedral, cleared within 44° of the viewer | CreativeML Open RAIL++-M output (SDXL plate); MIT (asset-forge) | what every pane, the blade and the puck reflect (`OpticalGlass` environment, gain 16). Masters `cabin_env_master.png` (the same picture, 16-bit linear) and the cube faces `cabin_px.exr` .. `cabin_nz.exr` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/vision_spatial_glass/`)

| File | What it is |
|---|---|
| `cabin_plate_master.png` | the ComfyUI cabin panorama on the Blender cyclorama |
| `cabin_px.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_nx.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_py.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_ny.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_pz.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_nz.exr` | one cube face of the Blender cabin studio, 32-bit float EXR, scene-linear radiance |
| `cabin_env_master.png` | `cabin_env.png` as 16-bit linear radiance (white = radiance 16): the same octahedral picture |
