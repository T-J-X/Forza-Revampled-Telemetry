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
| Design direction, design-system upkeep, design review | `design-superpowers:design`, `design-superpowers:ds-make`, `design-superpowers:design-review` (plugin; TJX enables it in the Claude app) | Skip it when the plugin is not enabled; `design:design-critique` still works. |
| Static concept art, posters, contact sheets | `canvas-design` (user scope) | Outputs go to `mockups/`, never into the repo tree as scripts. |
| Building a custom MCP server for this project | `mcp-builder` (user scope) | Only when a tool has no MCP and plain HTTP from Rust isn't enough. |
| Rendered hardware: bezels, dials, needles, crystals | `blender-hero-assets` | Through `tools/asset-forge`; the `Blender` MCP is for looking and checking. |
| Organic textures and detail passes | `comfyui-hero-textures` | ComfyUI must be running on `127.0.0.1:8188`. |
| Before merging or pushing a branch | `requesting-code-review`, `/code-review`, `/simplify` | |
| Cleaning up folders and finding duplicates | `file-organizer` | Never touch the deploy targets. |
| Designing, or pulling tokens and layouts from the design file | the `penpot` MCP: `high_level_overview` (once per chat), `penpot_api_info`, `execute_code`, `export_shape`; with `fh6-hmi-design-rules` | Penpot needs its file tab open and connected (`docs/design/PENPOT.md`). |
| Critiquing mockups in `mockups/` | the `ui-ux-designer` agent, or `design:design-critique` | |

If a skill doesn't load on its own, type `/<skill-name>`.

## 2. MCP servers

| MCP | Use for | Don't use for |
|---|---|---|
| **context7** (`.mcp.json`) | Crate docs before first use of an API: `windows` 0.58, `tiny-skia` 0.11, `ab_glyph`, `ttf-parser`, `serde`/`serde_json`, `arc-swap`, `png`, `thiserror` | Design decisions |
| **Penpot** (`penpot`, user scope) | All design work: boards, components, token sets mirrored from `data/design_tokens.json`, PNG/SVG checks with `export_shape`, and importing committed, pushed PNGs with `penpot.uploadMediaUrl` from this repo's raw GitHub URL pinned to a commit (`docs/design/PENPOT.md`) | Importing from a local path or `localhost` (blocked; only uncommitted renders need TJX to drag them in), or putting the MCP key in any file or in `.mcp.json` |
| **Blender** (`Blender`, user scope) | Hero renders and asset checks in a running Blender 5.3 with the MCP add-on (`blender-hero-assets`); repo assets are still built headless by `tools/asset-forge` | Saving `.blend` or script files into the repo |
| **ComfyUI** (HTTP at `127.0.0.1:8188`, no MCP) | Textures and detail passes driven from `tools/asset-forge` (`comfyui-hero-textures`); it must be running | Jobs that exceed 8 GB VRAM |
| **Picsart** (hosted connector, TJX signs in) | Generating, background removal, enhancing and vectorising images when Blender and ComfyUI don't fit; it spends Picsart credits and shows the cost first | Anything without TJX's go-ahead on the cost |
| **Built-in browser** | Opening `mockups/index.html` to inspect the 4K exports | Anything that needs a sign-in |

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
