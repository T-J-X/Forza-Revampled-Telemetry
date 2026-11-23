# RetroCyber assets

Built by `cargo run --release -p asset-forge -- build --set retro_cyber` (recipe `tools/asset-forge/src/recipes/retro_cyber.rs`). The cluster draws a procedural fallback for each when the file is missing.

| File | Kind | Size | Bytes | Generator (recipe + seed) | Licence | Used for |
|---|---|---|---|---|---|---|
| `housing_cockpit_l160.png` | colour, straight alpha | 2816x1056 | 475 KB | blender:retro_cyber::HOUSING view=cockpit cavity=1380x500 cut=58 ring=10 light=-160° scale=2 samples=96 seed=1907 | MIT (asset-forge) | the housing ring; blended with the other light by lateral G |
| `housing_cockpit_l110.png` | colour, straight alpha | 2816x1056 | 465 KB | blender:retro_cyber::HOUSING view=cockpit cavity=1380x500 cut=58 ring=10 light=-110° scale=2 samples=96 seed=1907 | MIT (asset-forge) | the housing ring; blended with the other light by lateral G |
| `housing_hood_l160.png` | colour, straight alpha | 3416x728 | 408 KB | blender:retro_cyber::HOUSING view=hood cavity=1680x336 cut=48 ring=10 light=-160° scale=2 samples=96 seed=1907 | MIT (asset-forge) | the housing ring; blended with the other light by lateral G |
| `housing_hood_l110.png` | colour, straight alpha | 3416x728 | 405 KB | blender:retro_cyber::HOUSING view=hood cavity=1680x336 cut=48 ring=10 light=-110° scale=2 samples=96 seed=1907 | MIT (asset-forge) | the housing ring; blended with the other light by lateral G |
| `phosphor_grain.png` | grey modulation, tileable | 1024x1024 | 213 KB | comfyui:workflows/sdxl_plate.json (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras 32 steps cfg 6.5) seed=1906 prompt="extreme macro photograph of the lit phosphor coating of a vacuum fluorescent display segment, fine powdery granular phosphor grain, soft mottled uneven glow, flat even lighting, monochrome grey, seamless surface texture, ultra detailed" → grey, seamless, high-pass, 0.88–1 display level, tile seam score 1.18 | CreativeML Open RAIL++-M output (SDXL); MIT (asset-forge) | tinted fill of every lit pod, chevron and column |
