# Phase 7 skills (setup chat and cluster threads)

The steps run in this order (TJX, 2026-11-27). `CLUSTER_PROMPT.md` has the details.

| Step | Skills / tools |
|---|---|
| Start | `fh6-hmi-design-rules`, `aesthetic`, and `fh6-visual-review` (read once now: its first step freezes your expectations before the first render is opened) |
| 1. R&D on the web | web search and the built-in browser; real sources only (production displays, owner photos, press images). Output: `RESEARCH.md` in your module directory |
| 2. Penpot | the `penpot` MCP (Penpot's hosted server; TJX keeps the Penpot file open with ⋮ → MCP server connected): `high_level_overview` once, `penpot_api_info` (what the plugin API supports, for example tokens and themes), then `execute_code` (pages, boards, shapes, styles, tokens) and `export_shape`. The hosted server cannot import images from local paths and its export is limited. |
| Design to code | the zone table at the top of your `mod.rs` mirrors your Penpot board (same zones, sizes and positions in local 4K px) |
| 3. ComfyUI | `comfyui-hero-textures`, through asset-forge on the cloud pod (`--lane cloud`); queue for a cloud lock with `gpu-lock run --lane cloud` (root `SKILLS.md` section 5) |
| 4. Blender (final 3D) | `blender-hero-assets`, through asset-forge on the cloud pod (`--lane cloud`); queue for a cloud lock with `gpu-lock run --lane cloud`. The `Blender` MCP is for looking and checking |
| Masters | write the 4K masters to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\`; the Google Drive desktop app syncs them. `asset-forge --masters <snake_name>` copies its own Blender and ComfyUI outputs there |
| Rust | `rust-best-practices` (before any `.rs`), `rust-skills` (the 265-rule reference: open the rule you need), `rust-patterns`, `rust-testing` (goldens) |
| Review (self-review, AGENTS.md rule 12) | the direction check (step 5) and the closing review (step 10) in `CLUSTER_PROMPT.md`: a fixed checklist, no scores. `fh6-visual-review` and its `visual-review` tool (step 5: the lines marked DC; step 10: every line, under `## Visual review` in the handover), `design:design-critique` (step 5 and step 10), `design:accessibility-review` and `rust-review` (step 10), and `requesting-code-review`; the `cold-reviewer` agent is the one blind reviewer `fh6-visual-review` asks for, once at each review point. Each lists findings once; never ask one for a score |
| Review panel (step 10, mandatory before the merge; TJS, 2026-11-28) | the six-lead review panel: six fresh agents at once, one per lead of this folder's `AGENTS.md`, each loading its lead's skills (`design:design-critique`, `design:accessibility-review`, `blender-hero-assets` and `comfyui-hero-textures`, `rust-review` with `rust-skills`), and one checker, exactly as `fh6-visual-review`'s `references/panel.md` says. Parallel Agent calls, pre-approved (root `SKILLS.md` section 6). Findings and yes/no only, once, never rerun to change an answer |
| Progress posts | at each milestone and at least every 20 to 30 minutes: a short update with a screenshot made by `visual-review annotate`, sent with SendUserFile (TJX, 2026-11-27) |
| Other fan-out (optional, pre-approved, root `SKILLS.md` section 6) | parallel Agent calls or the Workflow tool for research or asset work, at most 4 agents; the review panel may also run at the direction check; findings only, no scores |

MCP: Penpot (`penpot`; run `ToolSearch "penpot"` to load it; if nothing shows, the session started before the server was registered: ask for a fresh session), context7 (kit dependencies when needed; no API key) and the built-in browser (`mockups/index.html`, served over loopback).
