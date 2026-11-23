# Phase 7 skills (setup chat and cluster chats)

| Step | Skills / tools |
|---|---|
| Start | `fh6-hmi-design-rules`, `aesthetic`, `design-superpowers:design` |
| Penpot design | the `penpot` MCP (Penpot's hosted server; TJX keeps a Penpot file open with File → MCP Server connected): `penpot_api_info` first (what the plugin API supports, for example tokens and themes), then `execute_code` (pages, boards, shapes, styles, tokens), `high_level_overview`, `export_shape`. The hosted server cannot import images from local paths and its export is limited. The `figma:*` skills do not apply. |
| Design to code | the zone table at the top of your `mod.rs` mirrors your Penpot board (same zones, sizes and positions in local 4K px) |
| Assets | asset-forge (Blender and ComfyUI); `aesthetic` for texture art direction |
| Rust | `rust-best-practices` (before any `.rs`), `rust-patterns`, `rust-testing` (goldens) |
| Review | `design:design-critique`, the `ui-ux-designer` agent, `design-superpowers:design-review`, `rust-review`, `requesting-code-review` |
| Optional fan-out | the Workflow tool, at most 4 agents (for example parallel critique from 3 director viewpoints plus a perf check) |

MCP: Penpot (`penpot`; run `ToolSearch "penpot"` to load it — if nothing shows, the session started before the server was registered: ask for a fresh session, do not fall back to Figma), context7 (kit dependencies when needed) and the built-in browser (`mockups/index.html`).
