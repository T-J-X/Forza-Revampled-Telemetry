# GT7 Hood Pro assets

Built by `cargo run --release -p asset-forge -- build --set gt7_hood_pro` (recipe `tools/asset-forge/src/recipes/gt7_hood_pro.rs`). Both are optional at runtime: without them the cluster draws its procedural pod and ribbons.


| Path | Kind | Space / alpha | Dims | Size | Generator | Licence | Clusters | QA |
|---|---|---|---|---|---|---|---|---|
| `assets/clusters/gt7_hood_pro/pod_lens.png` | color | srgb / straight | 928x736 | 37 KB | blender:gt7_hood_pro::POD_LENS (squircle n 4 slab, 24-segment rolled bevel, satin clear-coated black dielectric IOR 1.52 (roughness 0.16, coat 0.7 at 0.09); key strip from the upper left at 950 W, cool rim strip 280 W; orthographic top-down, Cycles 256 spp + denoise, AgX, transparent film) seed=7201 | MIT (asset-forge) | Gt7HoodPro | face black floor 0.1 crushed to #000 (bevel speculars kept); 928x736 = 4x the hood pod (232 x 184 local px) plus a 2% margin each side; drawn 1:1 over the pod squircle |
| `assets/clusters/gt7_hood_pro/ribbon_grain.png` | color, tileable | srgb / additive | 1024x1024 | 85 KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed=7301 prompt="seamless texture of fine horizontal luminous light filaments and streaks, anisotropic brushed light, thin parallel white and pale cyan lines of varying brightness, sparse sparkling grain, pure black background, macro photograph, crisp, high detail" | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | Gt7HoodPro | SDXL 1024² → 4x-UltraSharp 4096² → 1024 px; blacks below 4% crushed to #000; seamless, tile seam score 139.94; luminance only (tinted at runtime) |

**Used for:** `pod_lens.png` is laid 1:1 over the gear pod (the sculpted black-glass face and rolled bevel); `ribbon_grain.png` scrolls through both wing ribbons as light filaments (tinted by the palette, `shift` on the locked inner arch).
