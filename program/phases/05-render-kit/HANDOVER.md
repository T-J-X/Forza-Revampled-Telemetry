# HANDOVER: Phase 5 · GPU render kit

**Chat / branch:** Phase 5 chat, `claude/phase-5-render-kit-d6906d` (origin/main with Phase 3 and Phase 4's fixes merged in)  **Date:** 2026-11-22  **Result:** ✅ complete (one perf target partly met, see Known issues)

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence |
|---|---|---|
| `cargo check`, `cargo test` and `cargo clippy -D warnings` green across the workspace, including the golden-image tests | ✅ | 282 passed, 0 failed; clippy `--workspace --all-targets -D warnings` clean; fmt clean; parity 20/20. Goldens: `crates/fh6-clusters/tests/golden_images.rs` vs `tests/golden/images/*.png` |
| `cargo build -p fh6-kit --target wasm32-unknown-unknown` succeeds; `tools/web-preview` renders in the built-in browser (screenshot) | ✅ | Both `fh6-kit` and `fh6-clusters` build for wasm32. `docs/handover-shots/phase5-web-preview-idle.jpg`, `phase5-web-preview-throttle.jpg` (Space held: revs, ribbon, throttle bar) |
| `docs/perf/render-kit.md` gives GPU and CPU timings at 4K for: the 2 demo clusters, bloom on and off, glass with and without refraction, fluid on | ✅ | GPU 0.259 ms (Liquid Glass) / 0.209 ms (analog) p50 with bloom, 0.164 / 0.171 without; glass plate 0.096 / 0.097 ms without / with refraction source; fluid solver 0.281 ms; WGC capture 0.44 ms CPU per frame; live overlay CPU and latency |
| `preview-export` produces 4K renders and real thumbnails of the demo clusters, and they render in the live overlay | ✅ | `mockups/kit/<Theme>/<state>.png` (+ `_crop`), `mockups/_thumbs/kit/...` box-downsampled from the 4K frame, badges measured per render; live: `docs/handover-shots/phase5-live-liquid-glass.png`, `phase5-live-analog.png`, `phase5-live-glass-refraction.png` (FH6_WheelDash + mock-udp + stand-in game, BMW palette from the car ordinal) |
| `docs/kit/README.md` is an author's guide with copy-paste examples for every feature | ✅ | Every snippet is compiled and rendered by `crates/fh6-kit/tests/guide_examples.rs` |

### Required features
| # | Feature | Status |
|---|---|---|
| 1 | Scene and batching: arena `Painter`, few draw calls, clipping, transforms, layers | ✅ `scene.rs`, `painter.rs`; no growth after warm-up (test `warm_frames_do_not_grow_the_arena`); 11–19 draw calls per sample cluster |
| 2 | SDF shapes: per-corner rounded rects, circles, rings, arcs (round/butt), tick rings, tapered needles with hub, polylines, polygons, chevrons, segmented bars, inner/outer strokes, 1 px rims | ✅ `shape.rs`, `shaders/shape.wgsl`; plus seven-segment digits |
| 3 | Paint: solid, linear/radial/conic (8 stops), images PNG + KTX2, 9-slice, blend normal/add/screen/multiply, linear premultiplied with sRGB output | ✅ `paint.rs`, `images.rs`; KTX2 supports RGBA8 sRGB/UNORM and BC7 (when the device has BC), zstd supercompression |
| 4 | Light and material: Kawase bloom per layer, glow, drop shadow, Gaussian layer blur, glass (tint, frost, fresnel rim + 1 px edge, crown sheen, caustic, refraction input), optional WGC `refraction_source` off by default with its cost | ✅ `renderer.rs`, `fh6-win/src/wgc.rs`, `fh6-app/src/refraction.rs` (`"GlassRefraction": true`) |
| 5 | Text: MSDF atlas at load, kerning, tracking, tabular figures, weight and italic, fonts from `assets/fonts/` and the user `fonts/` folder, motorsport default from tokens with Noto fallback | ✅ `text.rs` (MTSDF via `fdsm`, kerning via `rustybuzz`), roles bound from tokens `type.fonts` (Saira Condensed / Titillium Web …) once Phase 6's faces are present |
| 6 | Animation: critically damped (configurable) `Spring`, easing from tokens, `Smoothed<T>`, per-cluster state with `dt` and `t` | ✅ `anim.rs` (closed form, feed-forward), `Hysteresis`, `Tween`, `Strobe` |
| 7 | Effects: GPU stable fluids, particles, noise (simplex, curl), 2.5D helper, glTF mesh pass | ✅ `fluid.rs` + `shaders/fluid.wgsl`, `particles.rs`, `noise.rs` (CPU + WGSL), `perspective.rs`, `mesh.rs` + `shaders/mesh.wgsl` |
| 8 | Cluster contract v2; `StubRenderer` and `registry.rs` removed; extensible registry | ✅ `fh6_kit::{Cluster, FrameInput, Layout}`, `fh6_clusters::registry::REGISTRY`; v1 clusters behind `LegacyClusterRenderer` |
| 9 | Palette engine: factory (brand by ordinal), custom, random (seeded, harmonious, OLED-safe) on token roles | ✅ `palette.rs` |
| 10 | Logo zone: PNG or SVG (resvg), optical centring, brand-safe padding, brand-name then custom-name fallback | ✅ `logo.rs`, `images.rs` |
| 11 | Offscreen and tests: preview-export ported, golden images with perceptual tolerance | ✅ `tools/preview-export`, `fh6_kit::testing` (OKLab ΔE), `tests/golden/images/` |
| 12 | WASM: kit and cluster crate for wasm32/WebGPU; `tools/web-preview` | ✅ |
| 13 | Two showcase clusters (glass, analog) | ✅ `crates/fh6-clusters/src/kit/{liquid_glass,analog}.rs` |

## What was built
- **`crates/fh6-kit`** (new): `math`, `color` (sRGB, OKLab/OKLCH, text-safe lift to ≥ 7:1), `paint`, `shape`, `scene` (arena, GPU layouts), `painter`, `renderer` (work-rectangle targets, segments, bloom, blur, glass backdrop, single upload, timestamps), `images`, `text`, `anim`, `tokens` (v3.2 binding), `palette`, `logo`, `cluster` (contract v2, `SignalSmoother`), `perspective`, `mesh`, `fluid`, `particles`, `noise`, `testing`, `kit` (facade), `clock` (wasm-safe stopwatch); shaders `shape.wgsl`, `post.wgsl`, `mesh.wgsl`, `fluid.wgsl`. Tests: unit tests, `tests/render_gpu.rs`, `tests/fluid_gpu.rs`, `tests/guide_examples.rs`.
- **`crates/fh6-clusters`**: `registry.rs`, `common/` (ShiftLights, GearGlyph, SpeedReadout, VirtualLight), `kit/liquid_glass.rs`, `kit/analog.rs`, `tests/golden_images.rs`; `LegacyClusterRenderer`.
- **`crates/fh6-render`**: removed `registry.rs`, `renderer.rs` and the `ClusterRenderer` trait; `dispatch::style_to_theme` is public.
- **`crates/fh6-gpu`**: `OverlaySurface::render_with` (the kit's present path, with Phase 4's `DeviceFailed` guard); optional `TIMESTAMP_QUERY` and `TEXTURE_COMPRESSION_BC` features.
- **`crates/fh6-app`**: the render loop draws the registry cluster with the kit (theme from config + camera ring, palette from the car ordinal, `animating()` keeps frames coming while springs settle); `resources.rs` (tokens, brand palettes, `assets/`, user `fonts/`); `refraction.rs` (WGC source).
- **`crates/fh6-win`**: `wgc.rs` (Windows.Graphics.Capture, pipelined readback, downscale).
- **Tools**: `preview-export` (GPU), `bench-render` (kit benchmark), `web-preview` (new), `wgc-bench` (new).
- **Docs**: `docs/adr/0002-render-kit.md`, `docs/kit/README.md`, `docs/perf/render-kit.md`, AGENTS.md map, PROGRESS.md.

## Verification output
```
$ cargo test --workspace
... 282 passed; 0 failed (sum of all "test result" lines)

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.93s

$ cargo fmt --all -- --check        (clean)
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.68s
$ cargo build -p fh6-clusters --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.81s

$ cargo run --release -p bench-render -- --frames 600      (3840x2160, RTX 3070 Ti Laptop, DX12)
| Liquid Glass sample (bloom on)  | record 0.006/0.012 | render 0.039/0.151 | GPU 0.259/0.274 | 19 draws |
| Analog sample (bloom on)        | record 0.005/0.015 | render 0.032/0.082 | GPU 0.209/0.213 | 11 draws |
| Glass plate + refraction source | ...                                      | GPU 0.097/0.098 |          |

$ FH6_WheelDash --game-process fh6-stand-in.exe --bench-seconds 25   (+ mock-udp, 120 Hz)
Liquid Glass: receive->present p50 0.639 p99 2.049 ms; CPU/frame p50 0.568 p99 1.111 ms
Analog:       receive->present p50 0.571 p99 1.836 ms; CPU/frame p50 0.493 p99 1.123 ms
```

## Visuals
- Live overlay: `docs/handover-shots/phase5-live-liquid-glass.png`, `phase5-live-analog.png`, `phase5-live-glass-refraction.png`.
- Exports: `docs/handover-shots/phase5-export-liquid-glass-redline.png`, `phase5-export-analog-drift.png`; the full set in `mockups/` (`cargo run --release -p preview-export`, gallery `mockups/index.html`).
- Browser: `docs/handover-shots/phase5-web-preview-idle.jpg`, `phase5-web-preview-throttle.jpg`.
- Goldens: `tests/golden/images/{LiquidGlass,PorscheAnalog}_{cruise,drift}.png`.

## My calls
- **One uber-shader with analytic SDFs** rather than CPU tessellation or Vello: free AA at any scale, glow/shadow/rims from the same distance, one pipeline per blend mode (ADR 0002).
- **No derivatives in the shader**: the CPU passes device pixels per local unit with each instance, so the uber-shader is valid in WebGPU's non-uniform control flow.
- **Work rectangle**: intermediate targets cover only what was drawn (+ bloom margin); this is why a 4K cluster costs ~0.2 ms.
- **Glass writes its delta (additive)** so translucent glass over a transparent overlay composites exactly without dual-source blending. Glass samples the scene as it was when its layer began (documented rule: draw glass first in its layer).
- **MTSDF at load (fdsm) and kerning measured once (rustybuzz)** instead of shaping per frame: layout is table lookups, no allocation. Missing glyphs appear one frame later.
- **Single per-frame upload** (all regions packed, dynamic offsets) instead of a mapped staging ring: one `write_buffer` per frame is measured at microseconds.
- **v2 layouts ignore `CockpitScale`/`BonnetScale`** (v1 multipliers; 1.45 pushes the token envelope off a 4K screen); only `ClusterScale` (0.70–1.40) applies.
- **Bloom intensity curve**: emissive uses a soft threshold on the straight colour's peak channel (not luminance), so saturated reds and blues bloom as designers expect.
- **WGC pipelined readback** (one frame behind) by default: the blocking map costs up to 8 ms p99.
- **Clusters render every refresh while `animating()`** (springs, strobes), so needles move at 120 Hz between 60 Hz packets.
- **`Gear` labels**: `R`, `N`, `1`–`10` (raw byte 0 = R, 11+ = N), as Phase 4's test cluster decoded.
- **context7 was unavailable** (invalid API key) all session; every crate API was checked against its source in the cargo registry instead.
- Unknown cluster themes fall back to their family's sample (analog family → analog sample, everything else → Liquid Glass) so every config value shows a working cluster until Phase 7.

## Known issues / debt
- **Live CPU p99 1.11–1.19 ms** (target 1 ms; p50 0.49–0.58 ms, half the budget). The kit itself is < 0.05 ms; the tail is wgpu submit (~10 µs per render pass; 11–18 passes) and `present`. Levers: a bloom-off performance preset (2 passes), fewer backdrop passes, Phase 4's earlier debt on submit/present. Details in `docs/perf/render-kit.md`.
- **Fluid** costs about 0.3 ms GPU on top of its cluster (256x128, 20 Jacobi); keep grids small.
- **KTX2 loader** is untested against a real file (no KTX2 assets exist yet); PNG and SVG paths are tested.
- **`Camera25` has no pitch**: Phase 3's ADAS chase eye (0, 3.4, −6.0) m with 8° pitch needs one before the TeslaAdas rebuild (the eye is a public field; default stays AGENTS' (0, 1.85, −3.4)).
- **No user-font override UI yet**: `Kit::font_choice(Some(id))` exists; the config key and OSD/configurator choice arrive in Phases 9–10.
- **Logos**: the zone falls back to the brand name today (no logo images until Phase 6).
- **`fh6-gpu` quad pipeline and `test_cluster`** are unused by the app now (only `fh6-gpu`'s own tests); remove when nothing needs that baseline.
- **web-preview loader**: `index.html` has a one-line module import of the generated `pkg/web_preview.js`; all page logic is Rust. `pkg/` is gitignored and built by `web-preview-build`.
- Scrim, ambient wash and OSD still render with tiny-skia in `preview-export` (Phases 8–9).

## What the next phases must know
- **Start here:** `docs/kit/README.md` (author's guide, every snippet compiled), then `crates/fh6-clusters/src/kit/*.rs` and `common/`.
- **Register a cluster:** one row in `crates/fh6-clusters/src/registry.rs`; `ClusterTheme` stays the config identifier.
- **Coordinates:** local = 4K px at user scale 1.0 around the anchor; y down; angles clockwise from 3 o'clock. Fit the view envelope (`layout.envelope`).
- **Colours only from `p.palette()` / `p.tokens()`**; text and thin lines use `primary_text` / `accent_text`.
- **Bloom is a layer** (`LayerOptions::bloom()`), only for glance ranks 1–3. **Glass** goes first in a `LayerOptions::glass(sigma)` layer; `refraction: Refraction::World` uses the capture when enabled.
- **Checks:** `preview-export` (badges: GPU, draws, colour budget, transparency), goldens (`FH6_UPDATE_GOLDEN=1` to rewrite, review PNGs), `bench-render`, `FH6_WheelDash --demo --stats`, `web-preview`.
- **Phase 6:** put the OFL faces in `assets/fonts/` (families as tokens `type.fonts`); `Kit::bind_font_roles` binds them at startup. Logos load with `Images::load_file` (PNG/SVG/KTX2); KTX2 BC7 needs the device's BC feature (requested when available).
- **Phase 8:** `RoadCurve`/`Camera25` for ADAS, scrim and ambient wash still tiny-skia; chase-cam routing (`View::Chase`) uses the cockpit anchor.
- **Phase 9 (OSD):** the kit's glass, blur and text are ready; the OSD probably wants its own `Kit::begin_frame` with a full-screen `Layout` (the work rectangle then covers more of the screen).
- **Phase 10:** `tools/web-preview` is the base (WebGPU device, canvas surface, kit, synthetic drivetrain, Space/1–9 input); tokens are embedded with `include_str!`.
- **Deps added** (decision log): fdsm, rustybuzz, ttf-parser, resvg, gltf, ktx2, ruzstd, image (no default features); web-preview: wasm-bindgen, wasm-bindgen-futures, js-sys, web-sys; wgpu `webgpu` feature; WGC `windows` features. `wasm-bindgen-cli` 0.2.130 is installed and must match `Cargo.lock`.
