# Phase 5: GPU render kit (MASTER PROMPT)

You lead Phase 5 of FH6 WheelDash v2, running in your own chat and worktree. Phase 4 delivered `crates/fh6-gpu` (device, surface, offscreen path) and a live overlay loop. Your job is the toolkit every cluster chat builds with. It must make premium visuals easy and cheap: a cluster author should get bloom, glass, crisp text and spring animation in a few lines.

## Read first
- `ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, and this folder's `AGENTS.md` / `SKILLS.md`.
- `program/phases/04-engine/HANDOVER.md` and `docs/adr/0001-gpu-overlay.md`.
- If they are merged: `docs/design/TOKENS.md`, `data/design_tokens.json` and `docs/design/families/*.md` from Phase 3. If Phase 3 isn't merged yet, build to the needs listed below and leave token binding generic.
- `docs/specs/phase-8-materials-and-shaders.md`: the glass and kinematics maths to implement.
- What to keep: `crates/fh6-render/src/oem_scrim/ambient_wash.rs`, the road projection in `crates/fh6-clusters/src/clusters/tesla_adas.rs` (lines 25–204) and `crates/fh6-osd/src/text.rs`. Bring them across where useful.

## Goal
Build `crates/fh6-kit`, a GPU 2D/2.5D drawing kit on wgpu and WGSL, with these properties:
- one batched scene per frame
- under 0.5 ms GPU for a typical cluster at 3840x2160
- under 1 ms CPU in total
- runs natively and as WASM/WebGPU

## Required features
1. **Scene and batching.** An immediate-mode `Painter` API that records into a per-frame arena (no per-frame heap growth once warm) and renders in as few draw calls as possible. Clipping, transforms and layers are supported.
2. **SDF shapes,** analytically anti-aliased at any scale:
   - rounded rects with per-corner radii
   - circles, rings and arcs (start/end, thickness, round or butt caps)
   - tick rings
   - tapered needles with hub
   - polylines and paths (strokes and fills)
   - chevrons and segmented bars
   - inner and outer strokes for 1 px specular rims
3. **Paint:** solid; linear, radial and conic gradients with up to 8 stops; images and textures (PNG and KTX2); 9-slice; blend modes normal, add, screen and multiply. Colour stays premultiplied and linear end to end, with sRGB output.
4. **Light and material passes:**
   - dual-filter (Kawase) bloom with threshold and intensity per layer
   - outer glow and drop shadow
   - Gaussian blur for UI layers
   - a **glass material**: tint, frosted noise, a fresnel rim with a 1 px specular edge, an arched top sheen, an optional caustic texture, and an optional refraction input texture
   - an optional `refraction_source`: Windows.Graphics.Capture of the Forza window, sampled behind glass. It stays off by default, and the handover must give its perf cost.
5. **Text:** MSDF glyph atlas, generated at build time (`msdf-atlas-gen` style, or a Rust MSDF crate checked with context7) or at load.
   - Kerning, tracking, and tabular figures (fixed advance) for speed and RPM.
   - Weight and italic variants.
   - Fonts load from `assets/fonts/` (Phase 6 pack) and a user `fonts/` folder next to the exe.
   - Bundle a motorsport default chosen from Phase 3 tokens, using an OFL font. Until Phase 6 merges, fall back to the embedded Noto.
6. **Animation:** `Spring` (critically damped, configurable), easing curves from tokens, `Smoothed<T>` for telemetry inputs, and per-cluster persistent state. Each cluster gets `&mut self` across frames, plus `dt` and `t`.
7. **Effects helpers:**
   - a GPU 2D fluid solver (stable fluids, small grid, compute shader) for fluid clusters
   - a particle emitter
   - noise (simplex, curl)
   - a 2.5D perspective helper
   - a glTF mesh pass for ADAS cars: unlit or matcap and rim-lit, with depth
8. **Cluster contract v2.** Replace `fh6-render::ClusterRenderer` with:
   - `trait Cluster { fn update(&mut self, input: &FrameInput); fn draw(&self, p: &mut Painter, layout: &Layout); }`
   - `FrameInput` holds telemetry, smoothed values, `dt`, `t`, config, palette (factory, custom or random), logo handle, custom name and font choice.
   - `Layout` holds the anchor, scale, height offset, view (cockpit, hood or chase) and safe-zone rects.
   - Remove `StubRenderer` and `registry.rs`.
   - Keep `ClusterTheme` as the identifier and add an extensible registry. Each cluster module registers itself through one table that the Phase 7 setup chat generates.
9. **Palette engine:** factory (brand palette by ordinal from `car_brands.json`), custom (user hex values) and randomise (seeded, harmonious, OLED-safe), mapped onto token roles.
10. **Logo zone:** load a logo (PNG or SVG through resvg) and fit it into a rect with optical centering and brand-safe padding. Fall back to the brand name, then to the custom name, set in a chosen font.
11. **Offscreen and tests:**
    - Port `tools/preview-export` to the kit: real warm-up with `dt`, thumbnails downscaled from the 4K render, and computed (not hardcoded) badges.
    - Golden-image tests with a perceptual tolerance in `tests/golden/images/`.
12. **WASM:** `fh6-kit` and the cluster crate build for `wasm32-unknown-unknown` with the WebGPU backend. Provide `tools/web-preview/`, a minimal page rendering a sample cluster from synthetic telemetry, so Phase 10 can build on it.
13. **Showcase clusters:** port 2 demo clusters onto the kit to prove it: a glass sample and an analog sample. They don't need to be final designs.

## Exit criteria
- `cargo check`, `cargo test` and `cargo clippy -D warnings` are green across the workspace, including the golden-image tests.
- `cargo build -p fh6-kit --target wasm32-unknown-unknown` succeeds. `tools/web-preview` renders in the built-in browser; attach a screenshot.
- `docs/perf/render-kit.md` gives GPU and CPU timings at 4K for: the 2 demo clusters, bloom on and off, glass with and without refraction, and fluid on.
- `preview-export` produces 4K renders and real thumbnails of the demo clusters, and they render in the live overlay.
- `docs/kit/README.md` is an author's guide with copy-paste examples for every feature, ready for the cluster chats.

## Out of scope
Final cluster designs (Phase 7), the OSD (Phase 9) and final textures (Phase 6).
