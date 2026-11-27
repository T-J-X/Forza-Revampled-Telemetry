# Phase 7 skills (setup chat and cluster threads)

The steps run in this order (TJX, 2026-11-27). `CLUSTER_PROMPT.md` has the details.

| Step | Skills / tools |
|---|---|
| Start | `fh6-hmi-design-rules`, `aesthetic` |
| 1. R&D on the web | web search and the built-in browser; real sources only (production displays, owner photos, press images). Output: `RESEARCH.md` in your module directory |
| 2. Penpot | the `penpot` MCP (Penpot's hosted server; TJX keeps the Penpot file open with ⋮ → MCP server connected): `high_level_overview` once, `penpot_api_info` (what the plugin API supports, for example tokens and themes), then `execute_code` (pages, boards, shapes, styles, tokens) and `export_shape`. The hosted server cannot import images from local paths and its export is limited. |
| Design to code | the zone table at the top of your `mod.rs` mirrors your Penpot board (same zones, sizes and positions in local 4K px) |
| 3. ComfyUI | `comfyui-hero-textures`, through asset-forge; hold the GPU lock (`SKILLS.md` section 5) |
| 4. Blender (final 3D) | `blender-hero-assets`, through asset-forge; hold the GPU lock. The `Blender` MCP is for looking and checking |
| Masters | write the 4K masters to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\`; the Google Drive desktop app syncs them |
| Rust | `rust-best-practices` (before any `.rs`), `rust-patterns`, `rust-testing` (goldens) |
| Review | the closing review in `CLUSTER_PROMPT.md` (a fixed checklist, no scores), `rust-review`, `requesting-code-review`. The `ui-ux-designer` agent or `design:design-critique` may list findings once; never ask them for a score |
| Optional fan-out | the Workflow tool, at most 4 agents (for example findings from 3 director viewpoints plus a perf check); findings only, no scores |

MCP: Penpot (`penpot`; run `ToolSearch "penpot"` to load it; if nothing shows, the session started before the server was registered: ask for a fresh session), context7 (kit dependencies when needed; no API key) and the built-in browser (`mockups/index.html`, served over loopback).
