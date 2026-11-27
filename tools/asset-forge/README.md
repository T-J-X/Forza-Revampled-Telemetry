# asset-forge

Reproducible asset pipeline for FH6 WheelDash. It rebuilds everything in `assets/` (textures, flipbooks, glTF cars, logos, fonts), writes the ledger (`assets/ASSETS.md`, `assets/assets.json`) and the review page (`assets/contact-sheet.html`).

```
cargo run --release -p asset-forge -- doctor
cargo run --release -p asset-forge -- build                      # every set
cargo run --release -p asset-forge -- build --set glass          # glass|fluid|scifi|classic|adas|fonts|logos|comfy
cargo run --release -p asset-forge -- build --set fluid --force  # ignore fingerprints
cargo run --release -p asset-forge -- build --only caustic       # jobs whose id contains the text
cargo run --release -p asset-forge -- sheet                      # rewrite ASSETS.md and the contact sheet
```

## How it works

- **Jobs and idempotence.** Each job (for example `glass/plates`) has a fingerprint: the bpy source, the parameters and seed, and a post-processing version. `assets/forge.lock.json` stores the fingerprint of the last good run. `build` skips a job whose fingerprint matches and whose files all exist, so a second `build` does nothing.
- **Blender (bpy as Rust strings).** The repo is Rust only, so each recipe is a `const &str` in `src/recipes/*.rs`. `blender::run` writes `PRELUDE` plus the recipe to `target/asset-forge/blender/<name>.py` (outside git) with a JSON parameter file, and runs `blender.exe -b --factory-startup --python-exit-code 3 -P <script> -- <params.json>`. Scripts print `FORGE-OK` when they finish; the log stays next to the script.
  - Data passes (normals, height, AO) are emission-only renders into 32-bit EXR, read back in Rust. An orthographic top-down camera makes world normals equal tangent-space normals.
  - Beauty renders use Cycles on the GPU (OptiX), OptiX denoise, AgX, Blender's bundled CC0 `studio.exr` and area lights.
  - Loops are exact: displacement samples 4D noise on a closed circle in noise space, so frame N equals frame 0.
  - Tileable materials use integer lattice frequencies and 4D-torus noise; the lit render uses only light at infinity, so it tiles too.
- **ComfyUI.** `workflows/sdxl_plate.json` is API-format workflow data with `{{placeholders}}`. `comfy.rs` starts the portable server if needed, queues the patched workflow, polls `/history` and downloads through `/view`. SDXL base 1.0 renders 1024², 4x-UltraSharp upscales to 4096²; Rust crushes near-black to `#000` and makes tiles seamless.
- **Rust maths.** Where an equation beats a render: refraction offsets and fresnel masks from the plate normals, lens caustics traced through the plate normals, tileable looping water caustics, void-and-cluster blue noise, Chladni plates, curl noise, the sheen arch, scanline / VFD / phosphor masks, and the vector reticle and road-decal sheets (SVG made in Rust, rasterised with resvg).
- **Logos.** `data/logos.json` pins each brand's source: an official Wikimedia Commons SVG (URL, licence, attribution), or a remade wordmark set in an OFL face from the font pack and outlined to paths. Each brand gets an SVG, a 4096 px full-colour PNG, a 4096 px white mono PNG and a 1024 px dial PNG.
- **Fonts.** Downloaded from github.com/google/fonts at one pinned commit, with each `OFL.txt`; `fonts.json` records weights, variable axes, roles, cap ratios and `tnum` support read from the font tables.

## GPU lanes

GPU work runs on one of two lanes, each with its own lock file (root `SKILLS.md`, section 5):

```
cargo run --release -p asset-forge -- build --set glass                          # local lane (the default)
cargo run --release -p asset-forge -- --lane cloud build --set glass             # the rented pod
cargo run --release -p asset-forge -- --lane cloud --masters liquid_glass build --set liquid_glass
cargo run --release -p asset-forge -- cloud up                                   # also: setup, down, status, spend, doctor, key
cargo run --release -p asset-forge -- --lane cloud selftest                      # or without --lane, on this PC
```

- **`--lane cloud`** (or `FH6_GPU_LANE=cloud`) sends the run's ComfyUI and Blender work to a rented 24 GB pod. `comfy.rs` talks to the pod's ComfyUI through an SSH forward on `127.0.0.1:8189`. `blender.rs` hands the script to `cloud/remote.rs`, which uploads the inputs the parameters name, runs the same script on the pod and copies the output folder back. Recipes need no change.
- **`--masters <snake_name>`** (or `FH6_MASTERS`) also copies each Blender output folder and each ComfyUI image of 2048 px or more to the dash's folder in the Google Drive masters folder. It works on both lanes.
- **`cloud ...`** manages the pod: set-up, start, delete, status, the spend log with its budget, a doctor, and saving a new API key. What runs on the pod (the idle watchdog, the installer, the services, the doctor) is Rust string constants in `src/cloud/pod.rs`, with every version pinned. The API key, the SSH key, the settings and the spend log live in `%USERPROFILE%\Documents\Dash_Tools\cloud\`, never in the repo.
- **`selftest`** runs the same three timed jobs on either lane.

The guide, with the costs, the limits that hold them down and the measurements, is `docs/cloud-gpu.md`. What the thread that built the lane checked is in `HANDOVER-cloud.md` next to this file.

## Formats (what `fh6-kit` loads)

| Kind | Format | Notes |
|---|---|---|
| Colour | PNG 8-bit sRGB | straight alpha (`alpha` in the ledger says `straight`, `none` or `additive`) |
| Normals, flow, offsets, masks | KTX2 `R8G8B8A8_UNORM`, zstd, full mips | linear; normal mips renormalised |
| Glass normal masters | PNG 16-bit | the brief's master format; the kit currently squashes 16-bit PNG to 8-bit sRGB, so use the KTX2 copy at runtime |
| Flipbooks | atlas, row-major from the top-left | layout in `assets.json` (`flipbook.cols/rows/frames/fps/cell`) |
| Cars | binary glTF | Y up, nose towards +Z, centred, ground at y = 0, ≤ 5k triangles |
| Vectors | SVG | outlined, no font dependency |

## Requirements

Blender 5.3 at `%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64\blender.exe` and ComfyUI portable at `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable\` with `sd_xl_base_1.0.safetensors` and `4x-UltraSharp.pth`. Override with `FH6_BLENDER`, `FH6_COMFYUI` and `FH6_COMFYUI_URL`. `doctor` checks all of it. Model files never go in git. The cloud lane needs the OpenSSH client that ships with Windows (`%SystemRoot%\System32\OpenSSH\`; `FH6_OPENSSH_DIR` overrides it) and the files `cloud setup` writes.

## Adding an asset

Add a job to the set in `src/sets/`: give it a fingerprint, render (Blender recipe, ComfyUI plate or Rust), then call `ctx.image(...)` or `ctx.file(...)` with a `Spec` (set, kind, colour space, alpha, generator, licence, clusters, QA). The ledger row, thumbnail and contact-sheet card follow automatically.
