# ADR 0002: The fh6-kit render kit and cluster contract v2

**Status:** Accepted
**Date:** 2026-11-21
**Deciders:** Phase 5 (Rust Systems lead, with the Glass/Optical and GT7/Telemetry directors), under TJX's v2 program pre-approval

## Context
Phase 4 delivered a GPU overlay (ADR 0001): wgpu 30 on DX12, a DirectComposition swapchain with premultiplied alpha, frame latency 1, and a one-pipeline quad renderer. Every cluster chat of Phases 7 and 11 now needs a toolkit that makes premium visuals short to write and cheap to run:
- crisp vector shapes at any user scale (0.70–1.40 on a 4K OLED), needles, tick rings, ribbons and LED strips;
- light: bloom that is "earned" per element, glows, shadows, blur and the Liquid Glass material;
- motorsport typography: kerning, tabular digits that never jump, user fonts that keep their optical size;
- motion: springs exact at 60/120/144 Hz, smoothing for noisy telemetry;
- effects: GPU fluid, particles, noise, a 2.5D road camera and glTF cars;
- one code path natively and in the browser configurator (WebGPU, Phase 10).

Budgets: under 0.5 ms GPU for a typical cluster at 3840x2160 and under 1 ms CPU per frame in total (Phase 4 left CPU p99 at 1.28 ms, mostly present and submit).

## Decision

### 1. One immediate-mode painter over an arena, one uber-shader
- `Painter` methods (`fill`, `stroke`, `glow`, `shadow`, `text`, `number`, `image`, `nine_slice`, `mesh`, `layer`, `clip`, transforms) append fixed-size records to a `Scene` arena whose vectors are allocated once at their capacity and cleared each frame. A warm frame performs no heap allocation (tested).
- Every 2D primitive is a 144-byte instance drawn by **one** WGSL pipeline (`shape.wgsl`) that evaluates an analytic signed distance per pixel: rounded rectangles with per-corner radii, circles, arcs with round or butt caps, tapered capsules (needles), polylines, even-odd polygons, tick rings, chevrons, segmented bars, seven-segment digits, MTSDF glyphs and 9-slice images. Coverage modes reuse the same distance: fill, stroke (inner / centred / outer, in local units or exact device pixels), Gaussian glow and erf drop shadow.
- Consecutive instances that share blend mode and texture become one instanced draw call. A cluster is typically 10–40 draws whatever its shape count.
- Anti-aliasing needs no derivatives: the CPU passes device pixels per local unit with each instance, so the shader is valid in non-uniform control flow (a WebGPU requirement for an uber-shader).

### 2. Linear premultiplied composition on a work rectangle, one resolve
- The scene, emissive (bloom) and layer targets are `Rgba16Float`, linear, premultiplied, as ADR 0001 action 4 asked. The final pass encodes `E(c)·a` into the swapchain (`Bgra8Unorm`) or the offscreen target (`Rgba8Unorm`); light beyond coverage (bloom, rims) is written as additive emission, which DWM composites over the game.
- All intermediate targets are sized to the **work rectangle**: the union of what was drawn (plus the bloom margin), snapped to 16 px. A cluster costs what it covers, not 8.3 MP.

### 3. Layers as segments
`Painter::layer(LayerOptions { bloom, blur, backdrop, opacity }, |p| ..)` splits the frame into segments:
- **bloom**: instances write `colour × intensity × soft-threshold` into the emissive target (MRT); a dual-filter (Kawase) pyramid of 5 levels blooms them once per frame;
- **blur**: the segment renders into its own target, is Gaussian-blurred on a downsampled pyramid level and composited back;
- **backdrop**: the scene so far is copied and blurred (frost) before the segment, and glass samples both. Glass writes the *difference* it makes (additive blend), so translucent glass over a transparent overlay stays exact without dual-source blending.

### 4. Text: MTSDF atlas at load, kerning measured once
- Glyphs are generated with `fdsm` (pure-Rust msdfgen: MTSDF with error correction) at 64 px/em and an 8 px range into a 2048-wide shelf-packed atlas that grows to 4096 tall. The preload charset is generated in parallel at font load; other characters are generated after the frame they first appear in.
- Kerning pairs (GPOS and `kern`) are measured once per face with `rustybuzz` for the glyph set clusters use; layout at draw time is hash lookups only. Tabular digits use fixed 0.60 em cells (tokens), centred.
- Sizes are cap heights; faces with cap ratios outside 0.60–0.78 are clamped, so a user font keeps the layout.

### 5. Cluster contract v2
```rust
pub trait Cluster {
    fn update(&mut self, input: &FrameInput<'_>);
    fn draw(&self, p: &mut Painter<'_>, layout: &Layout);
    fn animating(&self) -> bool { false }
}
```
- `FrameInput`: raw telemetry, decoded and smoothed `Signals`, `dt`, `t`, config, palette, tokens, font choice, logo and names, view, reduced-flash flag.
- `Layout`: anchor, scale, height offset, view (cockpit, hood, chase), envelope and safe zone.
- `ClusterTheme` stays the identifier. `fh6_clusters::registry::REGISTRY` is the one table a cluster registers in (`ClusterEntry { theme, name, family, make }`); the Phase 7 setup chat generates it. Themes without a kit implementation resolve to their family's sample.
- `fh6-render`'s `ClusterRenderer`, `StubRenderer` and `registry.rs` are removed; the 29 v1 clusters keep compiling behind `fh6_clusters::LegacyClusterRenderer` as references for their rebuilds.

### 6. Host-agnostic
`fh6-kit` depends on `wgpu` (re-exported), not on `fh6-gpu`: the host passes a device and queue. Natively `fh6-gpu` provides them (DX12 + DirectComposition); in the browser `tools/web-preview` requests a WebGPU adapter. Readback helpers (`fh6_kit::testing`) are native-only.

### 7. One upload per frame
Instances, paints, clips, polyline points, per-segment globals and post-pass parameters are packed into one CPU staging vector and written with a single `Queue::write_buffer` into one GPU buffer; regions are bound with dynamic offsets. This replaces Phase 4's per-buffer uploads (ADR 0001 action 6). A persistent mapped ring was considered and not needed: one write per frame costs microseconds at cluster sizes.

## Options considered
| Option | Why not |
|---|---|
| Tessellate paths on the CPU (lyon) and draw triangles with MSAA | Per-frame CPU tessellation of moving needles and arcs, MSAA cost at 4K, and no free glow/shadow/rim from the same geometry |
| Vello (compute 2D renderer) | Excellent quality, but its own scene model, heavier compute passes for a few dozen shapes, and no control over the premultiplied/linear resolve or our glass and bloom layers |
| Bitmap fonts or runtime SDF via a C++ msdfgen | Bitmap text blurs at user scales; C++ breaks the Rust-only rule |
| One pipeline per primitive | More draw calls and state changes for no quality gain |

## Consequences
- Easier: a cluster gets bloom, glass, crisp text and springs in a few lines (`docs/kit/README.md`); the same code renders the overlay, `preview-export`, golden tests and the WebGPU preview.
- Harder: everything visual goes through one large shader; new primitives are added there (kinds and paint kinds are versioned constants shared by Rust and WGSL).
- Glass refracts our own earlier layers by default. `Refraction::World` additionally samples a captured game frame when the app attaches one (`Kit::set_refraction_source`); capture is off by default and its cost is in `docs/perf/render-kit.md`.
- Revisit the work-rectangle model if a cluster ever covers most of the screen (the OSD in Phase 9 may want a separate visual).
