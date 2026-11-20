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
| Before merging or pushing a branch | `requesting-code-review`, `/code-review`, `/simplify` | |
| Cleaning up folders and finding duplicates | `file-organizer` | Never touch the deploy targets. |
| Pulling tokens or layouts from Figma | `figma:figma-use`, `figma:figma-design-to-code` | Figma needs `/mcp` authorisation first. |
| Critiquing mockups in `mockups/` | the `ui-ux-designer` agent, or `design:design-critique` | |

If a skill doesn't load on its own, type `/<skill-name>`.

## 2. MCP servers

| MCP | Use for | Don't use for |
|---|---|---|
| **context7** (`.mcp.json`) | Crate docs before first use of an API: `windows` 0.58, `tiny-skia` 0.11, `ab_glyph`, `ttf-parser`, `serde`/`serde_json`, `arc-swap`, `png`, `thiserror` | Design decisions |
| **Figma** | Reading tokens, layout and exports from the master file into `data/figma_tokens.json` | Writing to Figma without TJX asking. The master file key is an open question in `docs/PROGRESS.md`. |
| **Built-in browser** | Opening `mockups/index.html` to inspect the 4K exports | Anything that needs a sign-in |

Token flow: Figma → `data/figma_tokens.json` → `fh6-render::tokens` → every cluster. Don't hardcode a hex colour or a typography value when a token exists. Add any missing token to `data/figma_tokens.json` and log it in `docs/PROGRESS.md`.

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
