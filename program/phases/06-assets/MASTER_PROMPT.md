# Phase 6: assets, logos & fonts (MASTER PROMPT)

You lead Phase 6 of FH6 WheelDash v2, running in your own chat and worktree. The clusters must look like 2027 hardware and beyond: real Liquid Glass, real abstract fluid, out-of-this-world sci-fi materials, horology-grade analog dials, and the car's own logo in 4K. Procedural shaders alone won't get there. **Blender and ComfyUI are your main tools.** Build reusable, scripted asset pipelines so the cluster chats in Phase 7 can make more assets on demand.

## Read first
`ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`, `program/phases/03-design-system/HANDOVER.md`, `docs/design/*` (tokens, families, heroes, catalog asset notes) and `program/phases/07-cluster-factory/CATALOG.md`. If Phase 5 is merged, also read `docs/kit/README.md` for the texture formats the kit loads.

## Tools
- **Blender 5.3 (headless):** `%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64\blender.exe -b -P <script.py> -- <args>`. Use Cycles with GPU (OptiX/CUDA on the RTX 3070 Ti).
- **ComfyUI portable:** `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable\run_nvidia_gpu.bat`, API on `http://127.0.0.1:8188` (`/prompt`, `/history`, `/view`).
  - It has no models yet. Install the pre-approved ones: SDXL base 1.0 (`stabilityai/stable-diffusion-xl-base-1.0`, file `sd_xl_base_1.0.safetensors`, about 6.9 GB) into `models/checkpoints`, and 4x-UltraSharp (about 67 MB) into `models/upscale_models`.
  - Download only from those official sources. 8 GB VRAM: use fp16 and tiled VAE / upscale for 4K.

## Required work
1. **Pipelines in the repo: `tools/asset-forge`, a Rust binary crate.** The repo is Rust only, so add no `.py`, `.ps1` or shell files (see the ONBOARDING rule).
   - **Blender recipes:** each is a Rust module that embeds its bpy script as a `const &str` (`r#"..."#`). The script is parameterised and deterministic (seeded). asset-forge writes it to a temp file and runs `blender.exe -b -P <tmp> -- <args>`.
   - **ComfyUI:** asset-forge queues API-format workflow JSON from `tools/asset-forge/workflows/*.json` (data, not code), with parameters patched in. It goes over HTTP using a pure-Rust client such as `ureq`, then polls `/history`, downloads through `/view` and upscales to 4K.
   - **Commands:** `cargo run -p asset-forge -- build [--set glass|fluid|scifi|classic|adas|logos|fonts]` rebuilds into `assets/` and is idempotent. `-- doctor` checks the Blender and ComfyUI paths and the models.
   - **Model and tool files stay out of git:** the `.gitignore` covers ComfyUI models and Blender caches.
2. **Liquid Glass set** (Blender, with ComfyUI for caustic variety):
   - glass plate normal maps (bevelled, rounded-rect and pill shapes) at 4K
   - refraction offset maps
   - caustic light textures (Cycles caustics or ComfyUI) as tileable 2K/4K
   - fresnel rim masks
   - frosted micro-noise
   - an arched sheen gradient
   - a "liquid" droplet and meniscus normal set
   - Format: 16-bit PNG for normals, KTX2 or PNG for colour.
3. **Abstract fluid set** for the fluid and generative clusters:
   - Blender liquid-metal and mercury simulations (mantaflow, or metaballs and displacement) rendered as looping **flipbooks** (for example 64 frames, 1024² cells) plus matching **normal and flow maps**. The GPU fluid solver from Phase 5 can then advect real-time telemetry-driven motion on top.
   - ComfyUI abstract plates: plasma, nebula, ferrofluid, chromatic prisms, Chladni patterns. Upscale to 4K, then make them seamless.
   - Every loop must loop seamlessly, with a verified seam.
4. **Sci-fi and out-of-this-world set:** holographic grids, warp tunnels, energy cores, HUD reticle sheets and scanline / VFD phosphor masks. Use ComfyUI for colour plates and Blender for geometric precision. Mark each asset's alpha and premultiplication.
5. **Classic and motorsport materials** (Blender procedural):
   - brushed titanium, guilloché (several patterns), carbon twill 2x2 and forged carbon, anodised aluminium, enamel dial faces, sunburst
   - bezel rings at several radii, with normal and AO
   - Alcantara and leather for OSD backdrops
6. **ADAS models:** low-poly stylised glTF cars (sport sedan, SUV, supercar, hypercar and an ego coupé), each at most 5k triangles with clean normals for rim-lit rendering. Add a lane and road decal sheet.
7. **Logos:** a 4K logo for **every brand** in `assets/car_brands/car_brands.json` (`BrandPalettes` keys, plus any makes in `Cars`).
   - Prefer the official SVG from Wikimedia Commons or the manufacturer's press kit. Keep the SVG and render a 4096 px transparent PNG.
   - Variants: full colour, monochrome white, and a "dial" variant tweaked for small sizes on dark dials.
   - If no good SVG exists, rebuild it as SVG (trace and clean up) or make a tasteful wordmark, and mark it as remade.
   - Write the brand-to-file map to `assets/logos/logos.json`, keyed exactly like `car_brands.json`, with aliases.
   - TJX accepted the trademark risk of shipping these in the public repo. Add `assets/logos/NOTICE.md`: trademarks belong to their owners, and they are used to identify the car.
8. **Fonts:** download OFL motorsport fonts from github.com/google/fonts: the Phase 3 picks plus at least 8 options (for example Saira and Saira Condensed, Titillium Web, Orbitron, Chakra Petch, Rajdhani, Michroma, Oxanium, Barlow Condensed, Exo 2, Teko). Keep each `OFL.txt`. Write `assets/fonts/fonts.json` with name, files, weights, the default flag, and tabular-figure support.
9. **CC0 fill-ins** (ambientCG and Poly Haven), only where Blender or ComfyUI can't do better.
10. **Ledger:** `assets/ASSETS.md` has one row per asset: path, generator (script plus seed, workflow plus seed, or source URL), licence, size and intended clusters. Keep the repo lean:
    - compress (oxipng; KTX2 where the kit supports it)
    - stay under about 400 MB in total
    - list anything larger in the handover; it can be left out of git and rebuilt by script
11. **Contact sheet:** `assets/contact-sheet.html` shows every asset grouped by set, for review in the built-in browser. Run `design:design-critique` on the liquid glass, fluid and sci-fi sets, and redo anything that looks cheap.

## Exit criteria
- `tools/asset-forge` builds, `-- doctor` passes, and `-- build` rebuilds at least one asset of every set.
- `git ls-files` lists no `.py`, `.ps1`, `.sh`, `.js`, `.ts` or `.c` files.
- These are present in `assets/`: Liquid Glass set, fluid flipbooks plus flow and normal maps, the sci-fi set, classic materials, ADAS glTF cars, logos for 100% of brands (with `logos.json`), the font pack (with `fonts.json`), and `ASSETS.md` with a licence for every file.
- The contact sheet has been reviewed, with critique notes and fixes in the handover.
- Total repo growth is reported in the handover.

## Out of scope
Rust code, except tiny loaders if Phase 5 lacks a format (coordinate through the handover instead), and the cluster designs themselves.
