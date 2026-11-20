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

## 3. Rust rendering and input laws

1. **No zero-size geometry.** tiny-skia returns `None` for zero-size rects, pixmaps, paths and gradients. Clamp to at least 1 px (`Rect::from_xywh(x, y, w.max(1.0), h.max(1.0))`) and handle the `Option`. Never `unwrap()` it.
2. **Premultiplied colour.** tiny-skia pixmaps are premultiplied RGBA. The DIB passed to `UpdateLayeredWindow` is premultiplied BGRA, so only swap R and B; never multiply alpha again.
3. **XInput masks.** Overlay, OSD and remapper code uses only the allowed masks in AGENTS.md. Strip `0xF000` (A/B/X/Y) from every input and every saved mask (`sanitize_toggle_mask`).
4. **Hybrid rendering.** Composite the cached PNG from `assets/` when it is present (`AssetCache::get_or_load`), and also run the procedural pass. With no asset, draw procedural only. A missing asset never panics.
5. **Fonts.** Noto Sans and Noto Sans JP (OFL licence) are embedded in `crates/fh6-osd/assets/fonts`. Load each face once. Tabular digits use a fixed advance (the `888` ghost grid) so numbers never shift sideways.
6. **OLED.** Use `#000000` where pixels must be off, or glass at alpha 28..64 with crisp 1 px rims. Never draw an opaque grey or navy box, and never fill the whole canvas. The orbit and dimmer (`fh6-core::oled_care`) run in `fh6-app`, not in clusters.
7. **Determinism.** Cluster rendering takes time from its inputs (`t_ms` / `dt`), not from a hidden wall clock. Sanitise NaN and Inf telemetry before drawing.
8. **Hot paths.** No file I/O, no allocation and no logging inside per-frame render code.
