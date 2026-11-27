# Liquid Glass assets

Built by `cargo run --release -p asset-forge -- build --set liquid_glass` (recipe `tools/asset-forge/src/recipes/liquid_glass.rs`). The glass is computed by the kit (`docs/specs/optics.md`); these files are what it reflects and bends. The cluster uses the kit's analytic studio and a plain light when a file is missing.

## Runtime files (in git)

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| (no runtime file: baked into `studio_env.png`) | studio panorama, seamless left to right | 4096x2048 | 6351 KB (master) | comfyui:liquid_glass::studio_plate (SDXL base 1.0 + refiner when installed, 1408x704, dpmpp_2m karras 32 steps cfg 6, seam column repainted at denoise 0.55, 4x-UltraSharp) seed=77203 prompt="360 degree panorama of the inside of a large dark automotive photo studio at night, matte black walls and ceiling, a few distant dim practical lights, lighting stands and equipment silhouettes, thin haze, glossy black floor with faint reflections, low key, cinematic, photographic, wide equirectangular panorama" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the picture on the cyclorama of the Blender studio: the ceiling lights, walls and floor that the glass rims mirror. Master `studio_plate_master.png` |
| `liquid_light.png` | caustic irradiance, tileable, sRGB, white = 4 x the floor under still water | 4096x4096 | 8059 KB | rust:liquid_glass::caustic (Snell's law at a rippled water surface, n = 1.333: 64 waves of 12..72 cycles per tile, amplitude k^-2, RMS slope 0.14, floor at 1 mean focal depths, 3x3 vertical rays per texel weighted by Fresnel transmission, wrap-around splat) seed=77101 | MIT (asset-forge) | the light in the rev ribbon under the glass rod. Master `liquid_light_master.png` |
| `studio_env.png` | environment, octahedral (z at the viewer in the centre), sRGB, white = radiance 16 | 4096x4096 | 1302 KB | blender:liquid_glass::STUDIO (Cycles, 6 faces 4096 px, fov 94°, 256 samples, seed=77301): horizon ring 0..9° (radiance 0.9 to 5.9 towards the key), key strip 74x9° radiance 8, strip 105x7° radiance 4.7, cyclorama with the ComfyUI studio plate (softened by a 6 px Gaussian) at radiance 0.3; resampled to octahedral, cleared within 44° of the viewer | CreativeML Open RAIL++-M output (SDXL plate); MIT (asset-forge) | what every glass piece reflects (`OpticalGlass` environment, gain 16). Masters `studio_env_master.png` (the same picture, 16-bit linear) and the cube faces `studio_px.exr` .. `studio_nz.exr` |

## 4K masters (Google Drive, `FH6 WheelDash v2 - 4K Masters/liquid_glass/`)

| File | What it is |
|---|---|
| `studio_plate_master.png` | the ComfyUI studio panorama on the Blender cyclorama |
| `liquid_light_master.png` | the ray-traced caustic as 16-bit linear irradiance (white = 16 x the floor under still water) |
| `studio_px.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_nx.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_py.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_ny.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_pz.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_nz.exr` | one cube face of the Blender studio, 32-bit float EXR, scene-linear radiance |
| `studio_env_master.png` | `studio_env.png` as 16-bit linear radiance (white = radiance 16): the same octahedral picture |
