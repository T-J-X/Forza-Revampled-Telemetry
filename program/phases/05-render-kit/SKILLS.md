# Phase 5 skills

| When | Skills / tools |
|---|---|
| Start | `rust-best-practices`, `engineering:architecture` (ADR 0002 render kit and cluster contract v2) |
| API and trait design, arenas, lifetimes | `rust-patterns` |
| Shader looks (glass, bloom, fluid) | the `penpot` MCP (read the Phase 3 hero boards and the glow / glass tokens), `fh6-hmi-design-rules` |
| Motion (springs, easing tokens) | `data/design_tokens.json` → `motion` and the family docs in `docs/design/families/` |
| Logo / SVG path | context7: `resvg`, `usvg`, `tiny-skia` (raster SVG only) |
| Tests (golden images, palette, logo fit) | `rust-testing` |
| Before the handover | `rust-review`, `/code-review`, `/simplify`; `engineering:documentation` for `docs/kit/README.md` |
| Optional fan-out | the Workflow tool for parallel shader R&D (at most 4 agents: bloom, glass, MSDF, fluid), then integrate here |

MCP: **context7**. Before first use, look up `wgpu`, `naga`/WGSL, `bytemuck`, `glam`, `image`/`ktx2`, `resvg`, `gltf`, `wasm-bindgen` and `web-sys` (WebGPU), and the MSDF crate or tool you choose.
