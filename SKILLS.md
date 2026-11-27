# SKILLS.md: skills, MCPs and Rust rendering laws

This sits alongside [AGENTS.md](AGENTS.md), which holds the council, the hard rules and the commands. If the two conflict, AGENTS.md and TJX win.

## 1. Which skill when

| Situation | Load first | Notes |
|---|---|---|
| Any new feature, cluster style, OSD change or redesign | `brainstorming` | No code until TJX has approved the design. |
| Writing or reviewing any `.rs` file | `rust-best-practices` | Applies to every crate. |
| Looking up a specific Rust rule: ownership, errors, memory, unsafe, async, API design, performance, anti-patterns | `rust-skills` (265 rules in 26 categories; MIT, by leonardomso, v1.5.1, restored from the earlier setup on 2026-11-27) | Its `SKILL.md` is the index: open only the rule files you need. The repo is edition 2021 with MSRV 1.87, so skip a rule that needs a newer edition or compiler. AGENTS.md wins where they differ. |
| Ownership, traits, concurrency (UDP engine, `ArcSwap`, threads) | `rust-patterns` | |
| Adding or fixing tests, golden vectors, property tests | `rust-testing` | Golden packets live in `tests/golden/packets`. |
| Reviewing a crate, module or diff | `rust-review`, with `rust-skills` as the rule reference | Gives a prioritised report with fixes. |
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
| Reviewing a dash | the fixed checklist ("Closing review" in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`), then TJX's sign-off | No scores and no critic loops (AGENTS.md rule 12). The `ui-ux-designer` agent or `design:design-critique` may list findings once, as input to the checklist; never ask them for a score. |
| Reviewing the OSD or a phase | a fixed yes/no checklist written at the start of the thread from AGENTS.md and SKILLS.md (sections A and B of the dash checklist apply as they are), then TJX's sign-off | The same rule: no scores and no critic loops. |

### How to call them
- **A skill:** call the Skill tool with its name, for example `rust-review`, or `design:design-critique` for a plugin skill. A person types `/<skill-name>`. Project skills live in `.claude/skills/` and are listed to every new session when it starts.
- **An MCP tool:** when a session lists the MCP tools by name only, load them first with ToolSearch, all in one call, then call them. For example `select:mcp__penpot__high_level_overview,mcp__penpot__execute_code,mcp__penpot__export_shape` or `select:mcp__context7__resolve-library-id,mcp__context7__query-docs`. The Blender tools are named `mcp__Blender__*`. The Google Drive tools carry an account ID in their names, so find them with the ToolSearch query `drive search_files`.
- **ComfyUI and headless Blender** have no MCP tool: run them through `cargo run --release -p asset-forge -- <command>` with the Bash tool.
- If a skill or tool is missing from the session's list, the session started before it was registered: ask for a fresh session.

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
| **Built-in browser** | Opening `mockups/index.html` and the product pages over loopback (`http://localhost:5311/`). A `file://` page opens as a static snapshot and its images don't load, so serve the folder on `127.0.0.1` first. From the repo root in the Bash tool, ComfyUI's bundled Python does it: `"$USERPROFILE/Documents/Dash_Tools/ComfyUI_windows_portable/python_embeded/python.exe" -I -m http.server 5311 --bind 127.0.0.1 --directory mockups`. Stop that server when you are done | Anything that needs a sign-in |

Token flow: `data/design_tokens.json` (the source of truth) → `fh6-kit::tokens` (the live loader; `fh6-render::tokens` serves only the v1 clusters) → every cluster, and mirrored into Penpot token sets for design work. Don't hardcode a hex colour or a typography value when a token exists. Add any missing token to `data/design_tokens.json` and log it in `docs/PROGRESS.md`.

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
   - Noto Sans stays as the fallback for missing glyphs. The font pack has no CJK face yet, so Japanese text does not render until one is added.
6. **OLED.** Use `#000000` (transparent) where pixels must be off, or glass at alpha 28..64 with crisp 1 px rims. Never draw an opaque grey or navy box, and never fill the whole canvas. The orbit and dimmer (`fh6-core::oled_care`) belong in `fh6-app`, not in clusters. Nothing calls them yet (fix phase B of the 2026-11-27 Rust assessment wires them).
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

1. Before any GPU job (ComfyUI, Blender, `asset-forge`, `preview-export`, the live demo, GPU tests), take the lock `%USERPROFILE%\Documents\Dash_Tools\gpu.lock`. Take it with this one command in the Bash tool, which creates the file only if it does not exist, so two threads cannot both get it:
   `( set -o noclobber; printf 'thread: %s\nstarted: %s\njob: %s\n' "<thread name>" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "<job>" > "$USERPROFILE/Documents/Dash_Tools/gpu.lock" ) 2>/dev/null && echo LOCKED || cat "$USERPROFILE/Documents/Dash_Tools/gpu.lock"`
2. If it prints `LOCKED`, run the job. If it prints another thread's lock instead, wait and work on a step that needs no GPU.
3. Release it as soon as the job ends, also when the job fails: `rm -f "$USERPROFILE/Documents/Dash_Tools/gpu.lock"`. Only the thread named in the file releases it.
4. If a lock is older than 30 minutes, ask its owner through the coordinator before removing it.

A cloud GPU is used only after TJX approves the provider and the cost.

## 6. Multi-agent workflows (ultracode)

`ultracode` is Claude Code's opt-in for multi-agent workflows (the Workflow tool): one session plans the work, fans it out to several agents and has other agents check the results. It is not a skill or a plugin, and nothing needs installing.

- **How it turns on:** only TJX can switch it on, by typing the word `ultracode` in a message to that session or by turning it on for the session. The session then sees a confirmation from the system. A note from the coordinator or another thread does not switch it on.
- **When to use it** (TJX, 2026-11-27: "when needed"): work too wide for one context, such as a codebase audit, a cross-crate review where each finding is checked by a second agent, a broad research sweep, or a change across many files. A single dash's design work doesn't need it.
- **Rules here:**
  1. Agents return findings and yes/no verdicts. No scored judge panels, and no loop that runs until a score passes.
  2. The GPU lock still holds: workflow agents never run GPU jobs in parallel.
  3. Agents that write files in parallel use worktree isolation, and none switches the branch of the shared checkout.
  4. One Penpot tab: at most one agent writes to Penpot.
  5. Keep a workflow under 10 agents unless TJX asked for more.
- **Model:** the coordinator sets each thread's model. TJX allowed the largest model with ultracode when a task needs it (2026-11-27).
