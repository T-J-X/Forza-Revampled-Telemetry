# BoschDdu assets

Built by `cargo run --release -p asset-forge -- build --set bosch_ddu` (recipe `tools/asset-forge/src/recipes/bosch_ddu.rs`). The cluster draws its procedural housing when the files are missing. The housing renders cover x ±706, y −312 .. 234 in local 4K px at 2 px per px.

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| `carbon_weave.png` | grey height, tileable | 1024x1024 | 198 KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed=1203 prompt="extreme macro photograph of a raw carbon fibre tow, thousands of fine glossy black filaments with tiny specular glints and slight waviness, filling the whole frame, flat even studio lighting, seamless surface texture, monochrome black and grey, ultra detailed, sharp focus" → grey, seamless, high-pass, laid into a 2×2 twill of 16 tows per tile with the filaments streaked along each tow, tile seam score 11.99 | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | the carbon material of the Blender housing (tow colour and the bump under the clear coat) |
| `housing_l160.png` | colour, straight alpha | 2824x1092 | 748 KB | blender:bosch_ddu::HOUSING open-bottom U 1400x540 r=96 face=14 chamfer=28 45° light=-160° scale=2 samples=128 seed=1204, carbon from `carbon_weave.png` tiled at 192 px | MIT (asset-forge) | the housing; blended with the other light by lateral G |
| `housing_l110.png` | colour, straight alpha | 2824x1092 | 710 KB | blender:bosch_ddu::HOUSING open-bottom U 1400x540 r=96 face=14 chamfer=28 45° light=-110° scale=2 samples=128 seed=1204, carbon from `carbon_weave.png` tiled at 192 px | MIT (asset-forge) | the housing; blended with the other light by lateral G |
