# SKILLS.md: skills, MCPs and Rust rendering laws

This sits alongside [AGENTS.md](AGENTS.md), which holds the council, the hard rules and the commands. If the two conflict, AGENTS.md and TJX win.

## 1. Which skill when

| Situation | Load first | Notes |
|---|---|---|
| Any new feature, cluster style, OSD change or redesign | `brainstorming` | No code until TJX has approved the design. |
| Writing or reviewing any `.rs` file | `rust-best-practices` | Applies to every crate. |
| Ownership, traits, concurrency (UDP engine, `ArcSwap`, threads) | `rust-patterns` | |
| Adding or fixing tests, golden vectors, property tests | `rust-testing` | Golden packets live in `tests/golden/packets`. |
| Reviewing a crate, module or diff | `rust-review` | Gives a prioritised report with fixes. |
| Porting a reference algorithm (shader maths, kinematics) from another language | `rust-port` | |
| Any drawing: clusters, OSD, scrim, mockups | `fh6-hmi-design-rules` (project skill), then `aesthetic` | The project skill wins on anchors, OLED black and the rank order. |
| Web UI: the configurator page, the showcase site, HTML boards | `frontend-design` (project skill), with `aesthetic` | Page logic stays Rust → WASM (AGENTS.md rule 11). |
| Design-system upkeep and design findings | `design:design-system`, `design:design-critique` (the `design` plugin, enabled) | The `design-superpowers` plugin is not enabled on this account (checked 2026-11-27); don't rely on it. |
| Static concept art, posters, contact sheets | `canvas-design` (user scope) | Outputs go to `mockups/`, never into the repo tree as scripts. |
| Building a custom MCP server for this project | `mcp-builder` (user scope) | Only when a tool has no MCP and plain HTTP from Rust isn't enough. |
| Building any dash | the order in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`: R&D on the web → Penpot → ComfyUI → final 3D in Blender | All assets are 4K. Blender is the last image step. |
| Organic textures and environment plates (the ComfyUI step) | `comfyui-hero-textures` | ComfyUI must be running on `127.0.0.1:8188`. Hold the GPU lock (section 5). |
| Rendered hardware: bezels, dials, needles, crystals (the Blender step) | `blender-hero-assets` | Through `tools/asset-forge`; the `Blender` MCP is for looking and checking. Hold the GPU lock (section 5). |
| Before merging or pushing a branch | `requesting-code-review`, `/code-review`, `/simplify` | |
| Cleaning up folders and finding duplicates | `file-organizer` | Never touch the deploy targets. |
| Designing, or pulling tokens and layouts from the design file | the `penpot` MCP: `high_level_overview` (once per chat), `penpot_api_info`, `execute_code`, `export_shape`; with `fh6-hmi-design-rules` | Penpot needs its file tab open and connected (`docs/design/PENPOT.md`). |
| Reviewing a dash, the OSD or a phase | the fixed checklist ("Closing review" in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`), then TJX's sign-off | No scores and no critic loops (TJX, 2026-11-27). The `ui-ux-designer` agent or `design:design-critique` may list findings once, as input to the checklist; never ask them for a score. |

If a skill doesn't load on its own, type `/<skill-name>`.

Notes (checked 2026-11-27):
- `aesthetic` is edited in this repo: its numeric ratings and its "repeat until the score passes" steps are removed, and its design-tool pointers name Penpot. Don't update it from upstream without re-applying those edits.
- The `engineering` and `vercel` plugins are not enabled. Phase 10 and 11 name skills from them; TJX enables the plugin before that phase starts.

## 2. MCP servers

| MCP | Use for | Don't use for |
|---|---|---|
| **context7** (`.mcp.json`; it needs no API key) | Crate docs before first use of an API: `wgpu` 30, `windows` 0.62, `ttf-parser` 0.25, `rustybuzz` 0.20, `fdsm` 0.8, `resvg` 0.50, `gltf` 1.4, `image` 0.25, `png` 0.17, `ktx2` 0.5, `ureq` 3.1, `wasm-bindgen` / `web-sys`, `serde` / `serde_json`, `arc-swap`, `thiserror` 2.0, and `tiny-skia` 0.11 for the v1 clusters in `legacy/` | Design decisions |
| **Penpot** (`penpot`, user scope) | All design work: boards, components, token sets mirrored from `data/design_tokens.json`, PNG/SVG checks with `export_shape`, and importing committed, pushed PNGs with `penpot.uploadMediaUrl` from this repo's raw GitHub URL pinned to a commit (`docs/design/PENPOT.md`) | Importing from a local path or `localhost` (blocked; only uncommitted renders need TJX to drag them in), or putting the MCP key in any file or in `.mcp.json` |
| **Blender** (`Blender`, user scope) | Asset checks and test renders in a running Blender 5.3 with the MCP add-on (`blender-hero-assets`); repo assets are still built headless by `tools/asset-forge`. The MCP's Blender runs headless and its Cycles device defaults to CPU, so set OptiX in your code when you render through it | Saving `.blend` or script files into the repo |
| **ComfyUI** (HTTP at `127.0.0.1:8188`, no MCP) | Textures and environment plates driven from `tools/asset-forge` (`comfyui-hero-textures`); it must be running. Installed: SDXL base and refiner, the depth and canny ControlNets, 4x-UltraSharp. A 4096×4096 texture fits the card (58 s, 7.5 GB peak, measured 2026-11-27) | Generating above 1 megapixel before the upscale, or any job without the GPU lock |
| **Google Drive** (desktop app to write, connector to read) | The 4K masters. Write to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\` (or `shared\`) and the desktop app syncs it (check the drive letter first). Use the connector to list and check what arrived | Uploading through the connector (it refuses files over about 7 MB), or creating another Drive folder |
| **Picsart** (hosted connector; not connected as of 2026-11-27) | Only after TJX connects it and signs in: generating, background removal, enhancing and vectorising images when Blender and ComfyUI don't fit; it spends Picsart credits and shows the cost first | Assuming it is available, or anything without TJX's go-ahead on the cost |
| **Built-in browser** | Opening `mockups/index.html` and the product pages over loopback (`http://localhost:<port>/`). A `file://` page opens as a static snapshot and its images don't load, so serve the folder on `127.0.0.1` first (for example with ComfyUI's bundled Python: `python_embeded\python.exe -I -m http.server 5311 --bind 127.0.0.1 --directory mockups`), and stop the server after | Anything that needs a sign-in |

Token flow: `data/design_tokens.json` (the source of truth) → `fh6-render::tokens` / `fh6-kit` → every cluster, and mirrored into Penpot token sets for design work. Don't hardcode a hex colour or a typography value when a token exists. Add any missing token to `data/design_tokens.json` and log it in `docs/PROGRESS.md`.

## 3. Rust rendering and input laws (v2 GPU renderer)

1. **Premultiplied, linear colour end to end.**
   - `fh6-kit` works in premultiplied linear RGBA and outputs sRGB.
   - The overlay surface uses `CompositeAlphaMode::PreMultiplied`.
   - Never multiply alpha twice.
   - Transparent is `(0,0,0,0)`. That is OLED-off.
2. **No degenerate geometry.** Clamp sizes, radii and stroke widths to at least 1 px before handing them to the kit. Sanitise NaN and Inf telemetry before it reaches a shader.
3. **XInput masks.** Overlay, OSD and remapper code uses only the allowed masks in AGENTS.md. Strip `0xF000` (A/B/X/Y) from every input and every saved mask (`sanitize_toggle_mask`).
4. **Hybrid rendering.**
   - Use the textured look (Blender or ComfyUI assets from `assets/`) when the asset is present, and keep a procedural kit fallback.
   - A missing asset never panics.
   - Assets load once at startup, never per frame.
5. **Fonts.**
   - MSDF text from `fh6-kit`. The default is a motorsport OFL font from `assets/fonts/`, and users can add fonts in `fonts/` next to the exe.
   - Speed and RPM use tabular figures (a fixed advance), so numbers never shift sideways.
   - Noto stays as the fallback for missing glyphs (JP included).
6. **OLED.** Use `#000000` (transparent) where pixels must be off, or glass at alpha 28..64 with crisp 1 px rims. Never draw an opaque grey or navy box, and never fill the whole canvas. The orbit and dimmer (`fh6-core::oled_care`) run in `fh6-app`, not in clusters.
7. **Determinism and animation.**
   - Clusters get `dt` and `t` through `FrameInput`, and keep their springs and state in `&mut self`.
   - Never read the wall clock inside a cluster.
   - Golden tests render a fixed time sequence.
8. **Hot paths.** No file I/O, allocation, logging or locking inside per-frame update or draw code. Batch draws, and reuse buffers and arenas.
9. **Budget.** Under 0.5 ms GPU per cluster and under 1 ms CPU per frame at 3840x2160. Measure with `--stats` and record the numbers in handovers.
10. **Rust only.** See AGENTS.md rule 11.

## 4. Program phases

Each phase of the v2 program has its own skill list, with the load order and timing: `program/phases/<phase>/SKILLS.md`. Use it instead of the general table above. Interactive chats with TJX outside the program start with `brainstorming` for any new feature.

## 5. One GPU, one job

One 8 GB card serves every thread, so GPU jobs take turns (TJX, 2026-11-27).

1. Before any GPU job (ComfyUI, Blender, `asset-forge`, `preview-export`, the live demo, GPU tests), look for `%USERPROFILE%\Documents\Dash_Tools\gpu.lock`.
2. If it is absent, create it with your thread name, the start time (UTC) and the job, then run the job.
3. Delete it as soon as the job ends, also when the job fails.
4. If it exists, wait and work on a step that needs no GPU. If it is older than 30 minutes, ask its owner through the coordinator before removing it.

A cloud GPU is used only after TJX approves the provider and the cost.
