# PROGRESS.md: status, backlog and decisions

Read this first and update it last on every task. The decision log is append-only.

## Current phase
The v2 build program is running. See [program/README.md](../program/README.md) for the roadmap, the dependency graph and the conductor protocol.

| Phase | Status |
|---|---|
| 1 Clean Rust repository | ✅ 2026-11-20 (public: T-J-X/Forza-Revampled-Telemetry) |
| 2 Program kit (`program/`, ONBOARDING, council agents 5–6) | ✅ 2026-11-20 |
| 3 Design system & art direction | ✅ merged 2026-11-22 (see `program/phases/03-design-system/HANDOVER.md`; HTML boards are the reference; finished in the Penpot master file 2026-11-23, see `docs/design/PENPOT.md`) |
| 4 Engine: a working exe | ✅ 2026-11-21 (branch `claude/phase-4-engine-b8ee7b`, see `program/phases/04-engine/HANDOVER.md`) |
| 5 GPU render kit | ✅ 2026-11-22 (branch `claude/phase-5-render-kit-d6906d`, see `program/phases/05-render-kit/HANDOVER.md`) |
| 6 Assets, logos & fonts | ✅ 2026-11-22 (branch `claude/objective-euclid-13487b`, see `program/phases/06-assets/HANDOVER.md`) |
| 8 Cockpit fit | ✅ 2026-11-22 (branch `claude/phase-8-cockpit-fit-7daeb6`, see `program/phases/08-cockpit-fit/HANDOVER.md`) |
| 9 Controller-first Liquid Glass OSD | ✅ 2026-11-23 (branch `claude/phase-9-osd-1fcc97`, merged with Phases 6 and 8; see `program/phases/09-osd/HANDOVER.md`) |
| 7 Cluster factory: setup (07-00) | ✅ 2026-11-23 (branch `claude/exciting-napier-8fa678`, see `program/phases/07-cluster-factory/HANDOVER.md`): 48 scaffolds, registry, `common/` helpers, per-cluster tooling, the LiquidGlass pilot; cluster chats next (Gt7HoodPro, PorscheAnalog, BoschDdu) |
| 7 Cluster chats, 10, 11 | ☐ waiting |

## Runtime audit (2026-11-20): why v2 exists
- **The exe draws nothing.** `fh6-app` never starts UDP, creates no window, has no render loop and no real XInput. `layered.rs` is a stub, and `--auto-exit` quits after about 3.5 s. (Phase 4)
- **Clusters are crude:**
  - Stroked block digits (`fh6-render/text.rs`), no gradients or blur.
  - 19 of 30 fill the whole canvas opaque.
  - Fixed scale, centred on the canvas instead of the binnacle.
  - `AssetCache` never decodes, so no PNG is ever drawn.
  - (Phases 5 and 7)
- **Config:**
  - Serde renaming breaks `StandardCockpitFOV`-style keys, so FOVs load as 0.
  - Missing fields become 0 (`cluster_scale` = 0).
  - Colour fields are never read.
  - (Phase 4)
- **Camera patcher:**
  - Misses `CockpitCam\FOV` and never writes seat height. (Phase 8)
  - `bind_car_brand` uses lowercase keys and would wipe `car_brands.json`. (Phase 4 safety fix)
- **Visibility:** nothing hides the overlay in menus or out of focus. (Phase 4)
- **OSD:** the dyno is a fixed frame, ESC injection is mocked, and LB/RB tab switching is a no-op. (Phase 9)
- **Logos and brand colours:** no logo support, and brand palettes are never applied at runtime. (Phases 5, 6 and 7)

## State of the code (after the host CPU phase, 2026-11-27)
| Check | Result |
|---|---|
| `cargo test --workspace --no-fail-fast` | ✅ 895 passed, 0 failed (1 ignored; `golden_images` 49/49) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit --target wasm32-unknown-unknown` | ✅ |

What the host CPU phase changed (known defects 10 and 17; `program/phases/fix-host-cpu/HANDOVER.md`):
- **The app meets 1 ms CPU per frame.** Live bench p99 0.78–0.88 ms and p50 0.62–0.68 ms on three dashes (was 1.11–1.19 ms p99 with less of the frame counted). The render thread runs in MMCSS "Games", without power throttling and on the performance cores; release builds use thin LTO; the kit uploads through mapped staging buffers; bloom is one compute pass.
- **The bench counts the whole frame.** "CPU per frame" is the render thread's whole iteration less the latency wait, broken into its parts, with passes and draws per frame and an idle-dimmed row; `mock-udp --scenario parked` drives the idle dim.

## State of the code (after fix phase B, 2026-11-27)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings (the `tools/web-preview` manifest note, as on main) |
| `cargo test --workspace` | ✅ 828 passed, 0 failed (1 ignored) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit --target wasm32-unknown-unknown` | ✅ |

What fix phase B added (findings 1, 4 and 20 of the 2026-11-27 Rust assessment; `program/phases/fix-b-oled-palette-guard/HANDOVER.md`):
- **OLED care runs.** `fh6_core::oled_care` has no clock: the render loop steps it with its own `dt`. The pixel orbit moves the layout by whole pixels; after 30 idle seconds the idle dim turns the dash's light down to `OledDimmerMinAlpha` (the look of that opacity) and leaves the black scrim as it is.
- **The OSD keeps the car's colours.** The dyno's frames name the car last seen racing, so the profile, the brand palette, the logo and the cluster agree over a paused game.
- **A render-thread panic ends the app.** It is logged with its thread and location, the main thread is told at once, and the app quits with exit code 3 instead of running on with a dead overlay.

## State of the code (after the Phase 7 setup, 2026-11-23)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings (the `tools/web-preview` manifest note, as on main) |
| `cargo test --workspace` | ✅ 597 passed, 0 failed (1 ignored), merged with Phase 9 |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit -p fh6-clusters --target wasm32-unknown-unknown` | ✅ |

What the Phase 7 setup added:
- **Cluster factory:** `fh6_clusters::catalog` (48 rows as data), the generated `registry`, one module per cluster under `clusters/<snake>/` (scaffold until its chat builds it), v1 clusters moved to `legacy/`.
- **Shared components:** 17 modules in `fh6_clusters::common` (glass plate, light ribbon, shift lights, hero gear, tabular speed, labels, tachs, needle dial, bezel, ADAS road, G-ball, tyre slip, signed bar, race tiles, seven-segment, liquid metal), documented in `docs/kit/README.md` §12.
- **Per-cluster tooling:** `preview-export --cluster <Name>` (4 states per view, crops, composites, 60-frame sequence, GPU p50 / p99), a golden test per catalog row (260 goldens), `FH6_WheelDash --demo --cluster <Name>`, `web-preview ?cluster=<Name>`.
- **Kit:** (font-pack loading is Phase 9's, adopted in the merge) `Scene::missing_glyph_count`, the glass-first backdrop cleared without `CLEAR_TEXTURE`.
- **Pilot:** Liquid Glass rebuilt on the kit (cockpit sequence GPU p50 0.227 / p99 0.238 ms; hood 0.44 ms); critique a conditional pass at 7.5/10 (glass material debt in shared code).
## State of the code (after Phase 9, 2026-11-23)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings (the `tools/web-preview` manifest note, as on main) |
| `cargo test --workspace` | ✅ 366 passed, 0 failed, 1 ignored |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit --target wasm32-unknown-unknown` | ✅ |

What Phase 9 added:
- **`crates/fh6-osd`:** the controller-first Liquid Glass OSD (spec `docs/design/OSD.md`): 5-tab row model, navigation with hold-to-repeat, on-screen keyboard, remapper, auto-pause behind `KeyboardInjector`, debounced saver, live dyno, carousel thumbnails, and the `CockpitFit` seam.
- **App wiring:** open and close edges in the render loop, the live view over the paused game, `PatcherFit` on Phase 8's camera patcher, nine blocker rows with a live blocker preview, per-view cluster height, and a "profile overrides" note for per-car settings.
- **Kit:** `Kit::set_underlay`, `Painter::set_backdrops(false)`, and font-pack loading from `assets/fonts/fonts.json` (variable-font instances, deferred glyph preload, cached kerning plans).
- **Input:** press/release latches, the hold toggle suspended while the OSD has focus, `--pad-script` and `--open-osd` for manual runs without a controller.
- **Perf:** 1.26–1.32 ms GPU for the whole OSD at 4K (target 1.5); CPU p50 0.99 ms, p99 2.24 ms with the OSD open (defect 13). See `docs/perf/osd.md`.

## State of the code (after Phase 8, 2026-11-22)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings (one cargo manifest note: `tools/web-preview` unused dependency `fh6-clusters`, as on main) |
| `cargo test --workspace` | ✅ 325 passed, 0 failed |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit -p fh6-clusters --target wasm32-unknown-unknown` | ✅ |

What Phase 8 added:
- **Dome scrim on the kit:** `Shape::Dome`, plus the `fh6_kit::scrim` draw and wash functions. It sits on the binnacle anchor, with an SDF feather, dither and the column void.
- **Fit model:** `fh6_core::fit` (`ScrimFit`, `ViewPlacement`, `clamp_anchor`), with per-view anchor, height, scale and safe zone and an on-screen clamp in `Layout::from_config`.
- **Per-car profiles:** `car_profiles.json` v2 is an `Option` overlay, hot-reloaded and applied by car ordinal in the render loop.
- **Views:** routing for cockpit, hood and the chase HUD, and the LB + RB camera resync.
- **Camera patcher**
  - The rewrite writes `CockpitCam\FOV`, the seat `Param.y` and the wheel offset, preserves CRLF, writes atomically, and refuses while the game runs.
  - It finds the install through Steam `libraryfolders.vdf`, keeps a timestamped backup, and restores in one call.
  - An opt-in sync service drives it, and `GET /api/camera` reports its status.
- **Fit review:** `preview-export --fit` writes to `mockups/fit/`.
- **Perf:** scrim + wash cost +0.17 ms GPU at 4K (Liquid Glass + scrim 0.38 ms).

## State of the code (after Phase 5, 2026-11-22)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings |
| `cargo test --workspace` | ✅ 282 passed, 0 failed (incl. GPU render tests, golden images, guide examples) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |
| `cargo build -p fh6-kit -p fh6-clusters --target wasm32-unknown-unknown` | ✅ |

What Phase 5 added:
- `crates/fh6-kit`: the GPU render kit (ADR 0002, author's guide `docs/kit/README.md`): arena-backed `Painter`, one SDF uber-shader (rects, circles, arcs, needles, polylines, polygons, tick rings, chevrons, LED bars, seven-segment, MTSDF glyphs, 9-slice), gradients, images (PNG, KTX2, SVG), blend modes, bloom, layer blur, Liquid Glass with optional world refraction, MTSDF text with kerning and tabular figures, closed-form springs, the palette engine, token binding (v3.2), 2.5D camera, glTF mesh pass, GPU fluid, particles, noise, golden-image helpers.
- Cluster contract v2 (`fh6_kit::Cluster`, `FrameInput`, `Layout`) and the registry table `fh6_clusters::registry::REGISTRY`; `fh6-render`'s `ClusterRenderer`, `StubRenderer` and `registry.rs` are gone (v1 clusters compile behind `LegacyClusterRenderer`).
- Two showcase clusters (Liquid Glass and analog samples) drawn live by `FH6_WheelDash` with the car's brand palette.
- `preview-export` renders on the GPU (real warm-up, box-downsampled thumbnails, measured badges); `bench-render` benchmarks the kit; new `tools/web-preview` (WebGPU page) and `tools/wgc-bench`.
- Perf: `docs/perf/render-kit.md` (GPU 0.21–0.26 ms per sample cluster at 4K; live CPU p50 0.49–0.58 ms, p99 1.11–1.19 ms).

## State of the code (after Phase 4, 2026-11-21)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings |
| `cargo test --workspace` | ✅ 182 passed, 0 failed (the launcher test compiles again) |
| `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean |
| `cargo fmt --all -- --check` | ✅ clean (whole workspace formatted once in Phase 4) |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |

What Phase 4 added:
- `FH6_WheelDash.exe` is a real overlay. It runs on a wgpu 30 DX12 DirectComposition swapchain (premultiplied alpha, Mailbox, frame latency 1) on a click-through, topmost, no-activate window (ADR 0001).
- A render thread wakes per packet and draws a GPU test cluster at the cockpit/bonnet anchor.
- Hide rules: hidden within one frame on `IsRaceOn == 0`, a stale stream or focus loss.
- Real XInput, tray, single instance, hot-reloaded config, `--auto-exit` and the Steam launcher.
- Latency stats (`--stats`, `--bench-seconds`); numbers are in `docs/perf/engine.md`.
- New crate `crates/fh6-gpu`.
- New tools: `tools/screen-capture` and `tools/stand-in-game`. `tools/bench-render` is now a real GPU benchmark in the workspace.

## State of the code (baseline 2026-11-20)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ passes, with 50 warnings (mostly unused dependencies) |
| `cargo test --workspace --exclude fh6-app` + `cargo test -p fh6-app --bins` | ✅ 103 passed, 0 failed |
| `cargo test --workspace` | ❌ `crates/fh6-app/tests/steam_launcher_tests.rs` doesn't compile (E0277, `&str` vs `str`) |
| `cargo clippy --workspace -- -D warnings` | ❌ 1 error: a collapsible `if` in `fh6-core` |
| `parity-dump tests/golden/packets` | ✅ 20/20 OK |

What exists:
- 7 crates and 4 tools.
- 29 `ClusterTheme` styles, all registered and rendering in `preview-export`.
- OSD with 4 tabs, a live dyno preview and the hold-to-toggle on View `0x0020` / 400 ms.
- The OEM domed scrim and the Bézier-cowl ambient wash.
- The camera INI/XML patcher.
- The UDP engine with ports 8000 / 5500 / 5607.
- XInput tracker with the A/B/X/Y lock.
- Single-instance guard, tray, process watcher and `/api/state` widget server.

## Known defects
1. ~~`crates/fh6-app/tests/steam_launcher_tests.rs` doesn't compile~~ Fixed in Phase 4.
2. ~~Clippy `collapsible_if` in `fh6-core` blocks `clippy -D warnings`~~ Fixed in Phase 4; the whole workspace is clippy-clean.
3. `FH6TelemetryData` has no `torque` field, so `GlyphDotMatrix` derives torque from power and RPM. Decide whether to add the field (check the packet layout first) or drop torque from the design.
4. ~~50 compiler warnings~~ Fixed in Phase 4.
8. Nine `TODO(phase-7)` comments in `fh6-clusters` mark values that are computed but never drive the visual. Examples: `_speed` in chrono_event_horizon, `_rpm_ratio` in f35_helmet_hmd, `_level` in kinetic_liquid_metal. The Phase 4 lint pass found them; they are for the cluster rebuilds to resolve.
9. ~~The OSD is not drawn yet~~ Fixed in Phase 9 (`docs/design/OSD.md`). The tray's "Open configurator" opens `config.json`, whose changes hot-reload, until the Phase 10 web configurator exists.
10. ~~Live CPU per frame p99 is 1.11–1.19 ms with the kit (target 1 ms; p50 0.49–0.58 ms). The kit's own work is under 0.05 ms; the tail is wgpu submit (about 10 µs per render pass, 11–18 passes) and `present`. See `docs/perf/render-kit.md` for levers (bloom-off preset, fewer backdrop passes).~~ Fixed in the host CPU phase (2026-11-27): p99 0.78–0.88 ms and p50 0.62–0.68 ms with the whole frame counted (`docs/perf/engine.md`, `program/phases/fix-host-cpu/HANDOVER.md`).
11. `fh6-gpu`'s `QuadRenderer` and `test_cluster` no longer draw anything in the app, but `OverlaySurface` (`surface.rs`) and the offscreen target still build a `QuadRenderer`; remove them when nothing needs a quad baseline (optional fix phase E of the 2026-11-27 Rust assessment).
12. `Camera25` has no pitch; the Phase 3 ADAS chase eye (0, 3.4, −6.0) m with 8° pitch needs a pitch term before TeslaAdas is rebuilt (the eye itself is a public field).
5. `fh6-render::oem_scrim::mount` (mount, visor lip, hub cap) is still a no-op.
6. `fh6-render::pool::PaintPool` is a cache stub with no real paint objects.
7. `assets/*.png` duplicates `assets/textures/4k/`. Pick one location when `AssetCache` is wired up.
13. OSD-open CPU per frame p99 is 2.24 ms (p50 0.99 ms; dash only 0.76 / 1.48 ms), measured before the merge. The OSD shows only while the game is paused, so racing frames are not affected; it shares the Phase 4/5 p99 levers (`docs/perf/osd.md`).
14. Phase 9 review items not yet verified (found by the branch review, not reproduced): clearing the custom name may come back after a reload (the config migration restores the default name); a seat height set in the OSD may be reverted on reload when the file still has a legacy `CockpitOffsetY`; a face is marked preloaded even when the glyph atlas is full (its text then falls back to Noto Sans); the open-edge ESC judges "racing" from a packet up to 250 ms old. See `program/phases/09-osd/HANDOVER.md`.
15. The pack fonts have no CJK glyphs; a custom name or font family in CJK draws as missing glyphs. The on-screen keyboard only types Latin, so only hand-edited configs hit it.
16. The OLED idle dim cannot dim bloom: the kit adds bloom after every draw. A dimmed frame is drawn without bloom, so a glowing part's halo cuts off when the dim starts. The dim itself is a fill of about 1.6 million pixels: 0.07 ms more GPU on a dimmed frame, and a slow-frame tail on the dimmed Bosch DDU (p99 up to 2.7 ms offscreen; `program/phases/fix-b-oled-palette-guard/HANDOVER.md`). The fix for both is a light gain in the kit's resolve (`Kit::set_light_gain`, beside `set_underlay`), which replaces the fill.
17. ~~The live bench's "CPU per frame" leaves out recording the frame: the scrim, the cluster's `draw()` and the OSD are recorded between the end of `build` and the start of `render_with`, and neither counts them. The 0.3 ms dash share of `AGENTS.md` directive 7 is measured with it.~~ Fixed in the host CPU phase: "CPU per frame" is the render thread's whole iteration less the frame-latency wait, and the report breaks it into the loop's checks, build, draw, kit render, `finish`, `submit`, `present` and the staging hand-back.
18. Only a panic on the render thread is caught. A render thread that hangs without panicking is not detected, and a panic on another thread is logged while the app carries on. `panic = "abort"` for release builds would cover every thread; it is TJX's decision (it skips the OSD's pending save).

## Backlog (not yet scheduled)
**Pre-05A: Liquid Glass OSD and cockpit optics**
- 60/40 OSD: the live dyno on the left and a glass control panel (alpha 22..36, 1 px rim `#FFFFFF`→`#38BDF8`) on the right.
- Low bucket seat and FOV kinematics: `Param.y = -0.045`, `DefaultOffsetFromSteeringWheelY = -0.035`, and an FOV slider from 45° to 110°, applied through `camera_patcher` without restarting the game.
- Per-camera cluster routing: Cam 3 gets its cluster plus the scrim; Cam 4 gets its cluster with the scrim at 0%; the chase cams get a floating HUD. Routing swaps automatically on RB.
- A contoured binnacle ambient wash with inverse-square falloff, alpha 12..35 at night.
- A per-cluster scaler (0.70× to 1.40×) with optical centre lock, plus typography presets.

**P5: mockups and contact sheet**
- Keep `preview-export` working: the 48 catalog clusters × 4 states (`--legacy` adds the 29 v1 clusters), the 0/50/100% scrim, the OSD and the `index.html` gallery.
- Add the in-cockpit compositor (Phase 8 spec P8-4), using `assets/reference/cockpit_with_wheel.jpg`.

**P6: gates, benchmark, deploy**
- P6-1: golden telemetry replay through the decoder; it must match `tests/golden/packets/*.json` field for field.
- P6-2: visual regression, pixel-diffed against the approved `mockups/` baselines. Checks: no text clipping, true-black alpha cut-outs.
- P6-3: `bench-render` over 1,000 frames at 4K; mean must be under 1.8 ms and P99 under 2.4 ms. (Superseded: the budget is AGENTS.md directive 7, under 1 ms CPU per frame and under 0.5 ms GPU per cluster.)
- P6-4: a Rust deploy script with SHA256 parity to the two deploy targets in AGENTS.md. It is held for TJX's explicit GO.

**P7: launcher, remapper, Game Bar**
- P7-1: `FH6_WheelDash.exe` (standalone, tray, process watcher) and `FH6_SteamLauncher.exe` (wraps the game launch with `--auto-exit`).
- ~~P7-2: hold-View remapper~~ Done in Phase 9 (Controls tab: OSD button remapper, hold time 200–1000 ms; A/B/X/Y never captured).
- P7-3: Xbox Guide button (`XInputGetStateEx`, ordinal 100, `0x0400`) and a Game Bar companion over a loopback HTTP control server.

**Phase 8: materials and shaders**
Bring every cluster up to `docs/specs/phase-8-materials-and-shaders.md`:
- no full-canvas fills
- every gauge reacts to live telemetry
- the OSD dyno renders the selected cluster
- `AssetCache` blitting
- Liquid Glass optics (P8-1)
- reactive sci-fi kinematics (P8-2)
- motorsport rebuilds (P8-3)

**Gates G1 to G5**
- G1: a scan for forbidden patterns.
- G2: per-cluster telemetry sensitivity. Every input in `docs/TELEMETRY_USAGE_MATRIX.md` that a cluster claims must change its output.
- G3: matrix integrity.
- G4: OSD dispatch. The v1 test (`g4_dispatch.rs`) went with the v1 OSD renderer in Phase 9; the OSD now draws the cluster through `fh6_clusters::registry`, covered by `fh6-osd` unit tests (`host`, `model`) and `preview-export`'s OSD states.
- G5: robustness against NaN, Inf and zero-size inputs.

## Open questions for TJX
1. ~~Which design file is the master?~~ Answered 2026-11-23: the Penpot master file (docs/design/PENPOT.md).
2. What is Phase 2: fixing the known defects and making clippy green, Pre-05A, or Phase 8 cluster quality?
3. ~~The first design tool's plan limits stopped Phase 3 before the component library and render placement were finished.~~ Answered 2026-11-23: TJX moved design work to Penpot on an unlimited plan (docs/design/PENPOT.md). The blocked work was finished in the Penpot master file on 2026-11-23 (file ID and board IDs in docs/design/PENPOT.md).
4. ~~AGENTS.md (Design Council 3D) still quotes the old ADAS eye (0, 1.85 m, −3.4 m). Phase 3 raised it to (0, 3.4 m, −6.0 m) because the old camera hid the road behind the ego car. OK to update AGENTS.md?~~ Closed 2026-11-27: AGENTS.md now quotes the eye the code uses, as part of the rule-file fixes TJX accepted that day.
5. Default shift colour: Phase 3 chose violet `#978DFC` (race convention, lifted to ≥ 7:1); brands whose `ShiftFlashHex` is white strobe white. Keep, or default to white?
6. (Phase 8) Which INI key sets the FOV of the wheel-less cockpit view [3] in FH6, `CockpitCam\FOV` or `Driver\FOV`? The patcher writes `CockpitCam\FOV` ← `WheelLessCockpitFOV` and `Driver\FOV` ← `StandardCockpitFOV`.
7. (Phase 8) With LB held, does RB still change the in-game camera? The LB + RB resync assumes it does (the ring advances on that RB, then the hold sets `ResyncCameraIndex`).
8. (Phase 8) The scrim at the default width overlaps the A-pillar vent and the left edge of the centre screen in the stock-seat reference. Tune per car after real low-seat captures (SCRIM_FIT §5), or narrow the default?
9. ~~(Cloud GPU lane) The 30 GB network volume is 26.7 GB full. Job folders no longer use it, so it grows only with uploaded inputs, which are cleared after 14 days. Leave it at 30 GB ($2.10 a month), or grow it at $0.07 a month for each GB? `asset-forge cloud doctor` warns under 3 GB free.~~ Answered 2026-11-27: TJX chose 100 GB, and the volume was grown to 100 GB that day ($7 a month). Runpod cannot shrink a volume.

## Decision log (append only)
| Date | Decision | Why |
|---|---|---|
| 2026-11-20 | The repo starts fresh as a Rust-only workspace at the root, with new git history | A clean base for Claude Code work and a new GitHub repo |
| 2026-11-20 | Claude Code does the work; the `program/` kit and this tracker replace the old task cards | One tracker and one set of rules |
| 2026-11-20 | `car_brands.json` is embedded with `include_str!` | Removes a hardcoded absolute path; rendering never depends on the working directory |
| 2026-11-20 | `preview-export` writes to `mockups/`, which is gitignored | Keeps about 150 MB of 4K PNGs out of git; they can be regenerated at any time |
| 2026-11-20 | Runtime data moved to `data/`; reference photos to `assets/reference/` | A clear layout. At runtime the app still reads `config.json` from its working directory |
| 2026-11-20 | The `fh6-render::pool` pool type renamed to `PaintPool` | Neutral naming; it was only used inside its own file |
| 2026-11-20 | MCPs: context7 plus the design tool's MCP only | Enough for crate docs and design tokens |
| 2026-11-20 | The v2 program (Phases 2–11) is approved: GPU renderer on wgpu, all 29 clusters rebuilt plus at least 15 next-gen concepts, one chat per cluster, and a fully automatic chain run by a conductor chat | Phase 1 audit: the exe renders nothing and the designs are crude |
| 2026-11-20 | Real 4K car logos ship in the repo | TJX accepts the trademark risk |
| 2026-11-20 | Motorsport default font from an OFL pack; users can add their own fonts | TJX's request; the OFL licence is safe for a public repo |
| 2026-11-20 | The repo is Rust only. Skill eval fixtures (C/Python/shell) and the TS hook plugin are untracked. Blender bpy lives inside the Rust tool `asset-forge` | TJX: "this is all rust", lightweight, 0 ms |
| 2026-11-20 | Commits are authored by `TJX <TJX@Ngage.com>`, with no co-author trailer | TJX's instruction for the public repo |
| 2026-11-21 | Phase 4: overlay presented with wgpu 30 on DX12 through a DirectComposition swapchain (`Dx12SwapchainKind::DxgiFromVisual`), premultiplied alpha, Mailbox, frame latency 1 | ADR 0001: the only transparent Windows route with no CPU copy |
| 2026-11-21 | New dependency `wgpu` 30.0.2 (default features off; `dx12`, `wgsl`, `std`, `parking_lot`) | The GPU renderer for v2; pure Rust (no C build) |
| 2026-11-21 | New dependency `bytemuck` 1.21 (`derive`) | Casts instance arrays to bytes for buffer uploads without `unsafe`; already in wgpu's tree |
| 2026-11-21 | New dependency `raw-window-handle` 0.6 (fh6-win) | Hands the overlay HWND to wgpu through a lifetime-safe `OverlayHandle` |
| 2026-11-21 | New dependency `tracing-subscriber` 0.3 (`fmt`, `env-filter`; fh6-app) | Logs to stderr or `--log <file>` with `RUST_LOG` filtering |
| 2026-11-21 | `serde_json` gains `preserve_order` (pulls `indexmap`) via fh6-win | `bind_car_brand` rewrites `car_brands.json` without reordering its keys |
| 2026-11-21 | Workspace moves from `windows` 0.58 to 0.62 | `wgpu-hal` 30 uses 0.62; with 0.58 in the workspace Cargo put `gpu-allocator` on 0.58 and wgpu failed to build |
| 2026-11-21 | MSRV raised to 1.87 (`clippy.toml`, `workspace.package.rust-version`) | wgpu 30 requires it |
| 2026-11-21 | `tools/bench-render` joins the workspace (its own `[workspace]` removed) and benchmarks the GPU path offscreen | One lockfile and target dir; the old tiny-skia stub measured nothing |
| 2026-11-21 | New tools `tools/screen-capture` (DPI-aware GDI capture to PNG) and `tools/stand-in-game` (solid focusable window) | Repeatable manual checks and screenshots without capturing personal desktop content |
| 2026-11-21 | Telemetry binds 127.0.0.1 by default; `UdpListenAllInterfaces: true` listens on every interface | Forza's Data Out targets 127.0.0.1; loopback needs no firewall rule |
| 2026-11-21 | All Win32 `unsafe` lives in fh6-win (tray/app window, single instance, process list, shell, DWM timing moved out of fh6-app); fh6-app has no `unsafe` | Phase 4 AGENTS rule |
| 2026-11-21 | XInput DLLs load with `LoadLibraryExW(LOAD_LIBRARY_SEARCH_SYSTEM32)` | Stricter than `LoadLibraryW`: a planted DLL in the working folder is never loaded |
| 2026-11-21 | One hold-toggle implementation: `fh6_input::HoldToggle` (config mask/ms, A/B/X/Y stripped); the tracker's View+Menu 350 ms and the OSD's own timer were removed | Phase 4 requirement; one behaviour everywhere |
| 2026-11-21 | Tray "Open configurator" opens `config.json` (hot-reloaded) until Phase 10 | An honest, working action now; the web configurator replaces it |
| 2026-11-21 | Whole workspace run through `cargo fmt --all` once | `fmt --check` is a gate; the tree had drift from before v2 |
| 2026-11-21 | Per-frame wgpu internals (`Queue::write_buffer` staging, `create_view`, the command encoder) allocate inside wgpu; our frame code allocates nothing | Accepted for Phase 4 (CPU p50 well under budget); Phase 5 moves to a persistent mapped staging ring (ADR 0001 action 6) |
| 2026-11-21 | Visibility: `--demo`/`--bench-seconds` bypass every rule; the OSD live view lifts only the race-off and stale rules, never the foreground rule; `DemoMode` in config.json no longer bypasses anything | AGENTS directive 9 (never visible in menus or over other apps); review finding |
| 2026-11-21 | UDP: the port that published last owns the stream until silent for 50 ms; the other port's copies are dropped (replaces payload/timestamp dedupe) | Payload compare dropped a paused game's genuine repeats and their forwards, and a lagging thread could reorder values; review verification |
| 2026-11-21 | wgpu errors and device loss set a flag instead of panicking; the render loop drops and recreates the device once a second; a failed surface configure is `GpuError::DeviceFailed` | wgpu's default handlers panic the render thread; an unconfigured surface panics in `get_current_texture` |
| 2026-11-21 | `AppWindow` state is an `Rc` shared by the window and each dispatched message | A handler dropping its own window freed the state it was running from; safe API must stay sound |
| 2026-11-21 | Phase 3 started its design file in a tool whose plan limits (3 pages, 1 variable mode, an MCP call cap) blocked the build | Superseded 2026-11-23 by the Penpot master file (docs/design/PENPOT.md) |
| 2026-11-21 | Phase 3 visuals are pure-SVG HTML boards in `docs/design/boards/`, rendered to PNG with headless Edge; the generator script stays outside the repo | The first design tool's MCP cap blocked rendering there, and the repo is Rust-only. Superseded 2026-11-23: design moved to Penpot (docs/design/PENPOT.md); the boards stay the reference until the Penpot master file is built |
| 2026-11-21 | `data/design_tokens.json` schema v3 (3.2.0): colour roles, palette modes, OFL type scale, glow, glass, light, motion, layout, states, components, families, accessibility; legacy keys kept; `colors`/`grid` names avoided so `fh6-render::tokens` still parses | One token source for the design file, the kit and every cluster (docs/design/TOKENS.md) |
| 2026-11-21 | Default fonts: Saira Condensed (display, motorsport default), Titillium Web (labels), plus Barlow Semi Condensed, Michroma, Chakra Petch, Exo 2, B612 Mono per family; all OFL | Research (RESEARCH.md §9); Bahnschrift is a host font, not OFL |
| 2026-11-21 | Seven families: Classic Analog, Modern Digital 2027, Race Dash (new), Liquid Glass, Sci-Fi, Fluid / Generative, ADAS & HUD; `F35HelmetHmd` moves to ADAS & HUD | Research found a race-dash architecture no other family owns, and aviation HUD shares the ADAS grammar |
| 2026-11-21 | Limiter strobe 10 Hz, 50% duty, only on a shift strip ≤ 42 px tall; nothing larger flashes above 3 Hz; red washes are steady | WCAG 2.3.1 and the 60/120 Hz frame maths (RESEARCH.md principle 2) |
| 2026-11-21 | Shift lights fill outside-in on per-car, per-gear thresholds; unlit pods are neutral grey lenses; default shift colour violet `#978DFC` | Race convention; colour only for lit state |
| 2026-11-21 | Only glance ranks 1–3 may glow; one virtual light per cluster moves with lateral G and drives every rim | Research principles 5 and 6; critique round 1 |
| 2026-11-21 | ADAS chase eye raised to (0, 3.4, −6.0) m with 8° pitch | Critique: the old eye (0, 1.85, −3.4) hid the road behind the ego car |
| 2026-11-21 | Classic dials default to a symmetric 270° face; shift-at-12 is an optional per-car Track mode | A linear 270° face rotated to put shift at 12 o'clock turns lopsided |
| 2026-11-21 | Catalog: 29 rebuilds + 19 new concepts, pattern-setters LiquidGlass, Gt7HoodPro, PorscheAnalog, BoschDdu, TeslaAdas, RetroCyber, KineticLiquidMetal (build order in CATALOG.md) | Defaults first, then one pattern-setter per family so helpers exist before siblings |
| 2026-11-22 | Render kit = one immediate-mode `Painter` over a fixed-capacity arena and one SDF uber-shader; linear premultiplied `Rgba16Float` composition on a work rectangle; layers as segments (bloom, blur, glass backdrop); one upload per frame (ADR 0002) | Premium looks at 0.2–0.3 ms GPU and under 0.05 ms kit CPU; the same code runs natively and in WebGPU |
| 2026-11-22 | Dependency: `fdsm` 0.8 (pure-Rust msdfgen; pulls `nalgebra` and `image` 0.25 without default features) for MTSDF glyphs; `ttf-parser` 0.25 directly | No C++ msdfgen; crisp text at every user scale |
| 2026-11-22 | Dependency: `rustybuzz` 0.20 to measure kerning pairs (GPOS and `kern`) once per face | Modern OFL faces only kern through GPOS; layout at draw time stays table lookups |
| 2026-11-22 | Dependencies: `resvg` 0.50 (no default features; SVG logos), `gltf` 1.4 (`utils` only; ADAS cars), `ktx2` 0.5 + `ruzstd` 0.9 (KTX2 textures, zstd supercompression), `png` (workspace) | Pure Rust; the asset formats Phase 6 delivers |
| 2026-11-22 | Dependencies (`tools/web-preview`, wasm32 only): `wasm-bindgen` 0.2, `wasm-bindgen-futures` 0.4, `js-sys` 0.3, `web-sys` 0.3; workspace `wgpu` gains the `webgpu` feature (a no-op on native) | The WebGPU preview Phase 10 builds on; `wasm-bindgen-cli` 0.2.130 installed to match |
| 2026-11-22 | `windows` features for Windows.Graphics.Capture (`Graphics_Capture`, `Graphics_DirectX_Direct3D11`, `Win32_Graphics_Direct3D`, `Win32_Graphics_Direct3D11`, `Win32_Graphics_Dxgi_Common`, `Win32_System_WinRT_Direct3D11`, `Win32_System_WinRT_Graphics_Capture`) | Optional glass refraction source, off by default (`GlassRefraction`) |
| 2026-11-22 | v2 layouts use only `ClusterScale` (0.70–1.40); `CockpitScale` / `BonnetScale` are v1 authoring multipliers and do not apply to kit clusters | At 1.45 the token envelope runs off the bottom of a 4K screen |
| 2026-11-22 | Phase 6: `tools/asset-forge` joins the workspace; bpy recipes are Rust `const &str`, ComfyUI workflows are JSON data; every job is fingerprinted (FNV-1a of recipe + params) in `assets/forge.lock.json`, so `build` is idempotent | Exit criterion; Rust-only repo; Phase 7 chats re-run single jobs |
| 2026-11-22 | New dependency `image` 0.25 (asset-forge; `png`, `exr` only) | Reads Blender's float EXR passes and PNGs; pure Rust |
| 2026-11-22 | New dependency `resvg` 0.50 (asset-forge; `text`, `system-fonts`, `memmap-fonts`) | Same SVG rasteriser the kit uses: logos, reticle and decal sheets render identically offline and at runtime |
| 2026-11-22 | New dependency `ureq` 3.1 (asset-forge; `json`, `native-tls`, `gzip`) | Blocking HTTP for the ComfyUI API, Commons and google/fonts; native-tls uses Windows SChannel so no C crypto build |
| 2026-11-22 | New dependency `oxipng` 10.1 (asset-forge; `parallel`) | Lossless PNG compression for the repo budget. Its `libdeflater` dependency compiles C at build time; accepted because it is a dev tool, never shipped in the exe |
| 2026-11-22 | New dependency `ruzstd` 0.9 (asset-forge) | Pure-Rust zstd for KTX2 supercompression (only its Fastest level exists; fine for the budget) |
| 2026-11-22 | New dependency `ttf-parser` 0.25 (asset-forge) | Reads weights, axes, cap height and `tnum` from the font pack into `fonts.json` |
| 2026-11-22 | New dev-dependency `ktx2` 0.5 (asset-forge tests) | Reads back our KTX2 writer's output with an independent parser |
| 2026-11-22 | Data maps (normals, flow, height, refraction, fresnel) ship as KTX2 RGBA8 UNORM with mips + zstd; 16-bit PNG normals ship only for the four glass plates | The kit's PNG path decodes to 8-bit sRGB, which would gamma-bend data; KTX2 UNORM is loaded as-is |
| 2026-11-22 | Fluid flipbooks: 64 frames, 512 px beauty cells (8×8 atlas, 4096²) plus a packed 256 px data atlas (RG normal.xy, BA flow.xy) | 1024 px cells are 256 MB of VRAM per flipbook uncompressed; 512 holds up at cluster size |
| 2026-11-22 | ComfyUI plates commit at 2048; the 4K masters stay in gitignored `assets/_masters/` and rebuild by script | Repo budget (~400 MB); the plates are backdrops behind glass and blur |
| 2026-11-22 | The Liquid Glass lens caustic is traced in Rust over the Blender plate normals instead of Cycles MNEE caustics | MNEE rendered near-black after 7 minutes; the tracer is exact, fast and reproducible |
| 2026-11-22 | Logos: Commons SVG first (57 brands); Ferrari, Lamborghini and Porsche use their official wordmarks (emblems are not on Commons); 34 brands are remade outlined wordmarks in OFL fonts; mono/dial variants use per-brand modes (silhouette, knockout, ink) and optional crops | 100% coverage with honest licensing; per-brand modes keep internal detail at dial size |
| 2026-11-22 | Dependency (`tools/preview-export` only): `image` 0.25 with the `jpeg` feature (pure Rust, via zune-jpeg) to decode the reference photos for `--fit` | The fit review composites over `assets/reference/*.jpg`; the kit already uses `image` without features |
| 2026-11-22 | `windows` feature `Win32_System_Registry` in `fh6-win` | Reads Steam's `SteamPath` to find `libraryfolders.vdf` |
| 2026-11-22 | The OEM dome stays on the factory binnacle when the user changes the cluster height; it still follows the anchor and the cluster scale | The fit critique showed the dome rising into the windscreen at +170 px; the binnacle does not move. SCRIM_FIT §2 and §4 updated |
| 2026-11-22 | Car profiles are an `Option` overlay (`car_profiles.json` v2); styles are not shipped per car | Missing keys inherit instead of collapsing to 0; profiles carry fit only |
| 2026-11-22 | The camera patcher writes only while the game is closed and only when `CameraPatchAuto` is on or the OSD/configurator asks; the first backup is the original | Deploy guard and safe restore; a running game queues the change ("restart needed") |
| 2026-11-22 | Phase 9: the OSD runs on the render thread (`fh6_osd::OsdHost`). Input arrives as press/release latches from the input thread; an edit lands in a working `AppConfig` (a new `Arc` per edit) and shows on that frame; a saver thread writes `config.json` 350 ms after the last edit; after close the working config is drawn until the watcher reloads an equal value (2 s at most) | Every input shows on the next frame, with no file I/O and no lock on the render thread |
| 2026-11-22 | The OSD dims the game through `Kit::set_underlay` (0.62 black, applied in the resolve pass) plus feathered black pools; it does not blur the game | The overlay cannot blur a game it does not capture; a full-screen dim shape cost 0.9 ms GPU at 4K, the underlay is free |
| 2026-11-22 | `Painter::set_backdrops(false)` for the OSD carousel thumbnails | Each glass backdrop snapshots and blurs the 4K work rectangle (1.2 ms); thumbnails keep tint, rims and sheen without it |
| 2026-11-22 | Auto-pause sits behind `fh6_osd::KeyboardInjector` (`fh6_win::keyboard::SendInputKeyboard`, scan-code ESC). ESC goes out on the open edge only while racing, with the game in front and `OsdAutoPauseGame` on; on close it resumes 250 ms later only if this code paused the game and it is still paused | Never an ESC in a menu or over another window; testable with a mock |
| 2026-11-22 | OSD config keys kept in `extra` (hot-reloaded, unknown to older builds): `ReducedFlash`, `GlassRefraction`, `PaletteSeed`, `ClusterFont`, `ShowLogo`, `ShowStats`, `ShiftPointPct`. The cluster height and the chase cluster use Phase 8's typed keys (`CockpitHeightOffsetPx` and siblings, `ChaseHudStyle`) | New settings without a config version bump; one owner per key after the Phase 8 merge |
| 2026-11-22 | The OSD's Fit & camera tab binds Phase 8: nine `OemDash*` rows, cluster height for the view being edited, a `CameraPatchAuto` switch, and Resync through `InputShared::resync_camera`. The OSD never writes camera files; Phase 8's sync service does, and only while the game is closed | Deploy guard; one writer for the game folder |
| 2026-11-22 | The OSD edits the global config. With a per-car profile the rows show the global values, the live view shows what the car really gets, and a row the profile overrides says so. Saving for one car is the configurator's job (Phase 10, `CarProfilesFile::upsert`) | A row that silently did nothing would look broken; per-car editing needs a UI the OSD does not have |
| 2026-11-22 | `Fonts::load_dir` reads the font pack as Phase 6 ships it: `assets/fonts/fonts.json`, one folder per family, variable fonts as one face per weight (300/400/500/600 within the axis), glyph preload deferred until a face is used. A face takes the manifest's family name, else its typographic family (name ID 16) | The Phase 5 loader read only flat folders, and registered each static weight as its own family ("Saira Condensed Thin"), so token roles would have bound to the wrong weights |
| 2026-11-22 | Kerning is measured with two shape plans compiled once per face (Latin, and no script) instead of `rustybuzz::shape`, which compiles a plan per pair; a test proves the tables are identical | 4,624 pairs per face |
| 2026-11-22 | Dev profile: `opt-level = 3` for `rustybuzz`, `ttf-parser`, `fdsm` and `nalgebra` | Loading one font face took 5 s in a debug build (shaping and MTSDF rasterising); the kit's text tests went from 34.6 s to 2.1 s |
| 2026-11-22 | `fh6_input::PadScript` and `--pad-script`, plus `--open-osd` | Repeatable manual runs and screenshots without a physical controller; the script goes through the same tracker, so A/B/X/Y are stripped |
| 2026-11-22 | Phase 9 adds no dependency | — |
| 2026-11-22 | The Phase 9 OSD page was not built in the design file: the first design tool's MCP call cap returned on the first write. `docs/design/boards/osd/index.html` and `docs/design/OSD.md` are the reference | Same block as Phase 3. Superseded 2026-11-23: the OSD page is built in Penpot (docs/design/PENPOT.md) |
| 2026-11-23 | Phase 9 merged with main (Phases 6 and 8) before the final gates: the OSD uses the Phase 8 `CameraPatchHandle` (`fh6-app::osd_fit::PatcherFit`), the nine `OemDash*` blocker rows, per-view cluster height and the typed `ChaseHudStyle`; it draws in the Phase 6 font pack | Main moved under the phase; the OSD binds to the real cockpit-fit model instead of the Phase 9 stand-ins |
| 2026-11-23 | OSD rows edit the global config; the live view shows the car's profile on top, and an overridden row says so | Per-car editing needs a save target per row; Phase 10 owns per-car profiles in the configurator |
| 2026-11-23 | Branch review fixes: no OSD (and no ESC) while the overlay is hidden from the tray; the dyno keeps the last raced car's profile; the Cockpit FOV row reads the wheel-less FOV the patcher writes; the chase glide starts from `ChaseAnchorX`; no per-frame assets clone; the car-profile log line fires only on a car or profiles change | Found by an adversarially verified review of the branch |
| 2026-11-23 | Design tool moves to Penpot. Penpot's official hosted MCP is registered at user scope as `penpot` (key kept out of the repo and `.mcp.json`); setup, limits (the file tab must stay open, no local-path image import, limited export) and the master-file plan are in `docs/design/PENPOT.md`. `data/design_tokens.json` stays the token source of truth | The first design tool's plan capped MCP calls; TJX has an unlimited Penpot plan |
| 2026-11-23 | Phase 7: cluster chats own only `clusters/<snake>/`, their goldens, assets, recipe and their own `CATALOG.md` status cell; `catalog.rs`, `registry.rs`, `ClusterTheme`, `common/`, the harness, kit and tools are frozen for them | Up to 3 parallel cluster chats merge without conflicts |
| 2026-11-23 | Dependencies (`tools/preview-export`): `tracing` and `tracing-subscriber` (workspace versions, already used by `fh6-app`) | wgpu validation errors were invisible in headless renders; no new crate enters the lockfile |
| 2026-11-23 | Dev profile: `rustybuzz`, `ttf-parser`, `fdsm`, `png`, `miniz_oxide`, `fdeflate` built at opt-level 3 | Golden and component tests load the 64-face font pack in seconds instead of minutes (one font test took 244 s) |
| 2026-11-23 | Merging main (Phase 9): Phase 9's `Fonts::load_dir` (manifest, variable instances, deferred preload) replaces the Phase 7 lazy loader; Phase 7 keeps `Scene::missing_glyph_count` and the review tools' repeat-until-complete glyph loop | One loader; Phase 9's reads the pack as Phase 6 ships it |
| 2026-11-23 | v1 tiny-skia clusters moved to `fh6-clusters/src/legacy/`; every catalog row has a kit module (scaffold until built) and a golden | `clusters/<x>.rs` and `clusters/<x>/mod.rs` cannot coexist; v1 stays reachable until each rebuild lands |
| 2026-11-23 | Phase 7 designs in Penpot (TJX switched to an unlimited Penpot plan): one page `07 · <Name>` per cluster; `data/design_tokens.json` stays the token source of truth; the HTML board stays alongside | The hosted Penpot MCP has no call cap |
| 2026-11-23 | Penpot master file built (file `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, connected as "New File 1"; TJX renames it to "FH6 WheelDash v2 — Design System"): tokens v3.2 as 22 sets and 489 tokens with `Family` (7) and `Glass` (2) theme groups, 95 library colours, 17 typographies, the 16 COMPONENTS.md components (12 variant sets, bound to tokens), 7 templates, 28 heroes, the OSD, the scrim fit and the configurator direction. Pages are numbered by owning phase (00–04, 08, 09, 10); Phase 7 cluster chats add `07 · <Name>`. Renders are imported with `penpot.uploadMediaUrl` from this public repo's raw GitHub URLs pinned to a commit. IDs in `docs/design/PENPOT.md` | Finishes the Phase 3 design-system build; server-side URL import removes the hand-dragging step for committed images |
| 2026-11-23 | Asset pipeline v2: physical cluster hardware is rendered by the shared `tools/asset-forge/src/hero/` library (calibrated rig with a dim overhead sky, pixel-exact ortho frames, measured-F0 PBR cookbook, lathe/extrude/mesh parts, beauty at 3 key azimuths + normal/height/AO/rmh/emission) and drawn at full strength under the vectors; ComfyUI graphs are built in Rust (`hero::sdxl`) with refiner, ControlNet, seamless seam repaint and tiled VAE. No new dependency | Luminance maps at 10–12 % over vectors were invisible; a bright overhead source whitened every metal face. One library keeps every cluster on the same light and scale |
| 2026-11-27 | Penpot is the only design tool. The token file is `data/design_tokens.json` (loader `fh6-render::tokens::DesignTokens`; the `fh6-render` helpers are `DesignKit` and `DesignLayout`). `docs/design/BOARDS.md` documents the HTML boards. `.mcp.json` keeps context7 only. Skills added: `frontend-design` at project scope, `canvas-design` and `mcp-builder` at user scope (all from `anthropics/skills`); phase skill lists point at the `penpot` MCP, `blender-hero-assets` and `comfyui-hero-textures` | TJX: the workspace is Penpot only. The two user-scope skills ship Python and fonts, which rule 11 keeps out of the repo. No crate dependency changed |
| 2026-11-27 | Tool validation (thread 3): `.mcp.json` drops the placeholder context7 key (context7 needs none and failed with it in every session); `CLUSTER_PROMPT.md` moves to R&D → Penpot → ComfyUI → Blender, with a fixed 36-line closing review and TJX's own direction check and sign-off (no scores, no proxy reviewer); the `prompt-engineer` agent is removed and `aesthetic` loses its score loop; GPU jobs hold `%USERPROFILE%\Documents\Dash_Tools\gpu.lock`; 4K masters go to the Drive folder through the desktop app (`G:\My Drive\FH6 WheelDash v2 - 4K Masters\`), because the Drive connector stops at about 7 MB | TJX's rules of 2026-11-27. Measured on the 8 GB card: 4096×4096 SDXL in 58 s at 7.5 GB, 3840×2160 Cycles glass in 15 s at 4.1 GB |
| 2026-11-27 | Skill added: `rust-skills` (leonardomso/rust-skills 1.5.1, MIT, 265 rules in 26 categories, Markdown only) at `.claude/skills/rust-skills/`, copied from the earlier local setup. `SKILLS.md` gains "How to call them" and section 6 on multi-agent workflows (ultracode): findings and yes/no verdicts only, the GPU lock, worktree isolation, one Penpot writer | TJX asked on 2026-11-27 whether the earlier Rust rule set was still there; it was not. The earlier port-process skill is left out: the port is finished and its rules are in `AGENTS.md` |
| 2026-11-27 | Rule files brought up to date in one commit. `AGENTS.md` gains global directives 12 to 16 (TJX's 2026-11-27 rules: checklist reviews and sign-off, threads, the dash order and 4K, one GPU job, merging) and loses its stale facts (MSRV 1.87, `windows` 0.62 and wgpu, 125 Hz polling, the directive 7 budget, the `asset-forge` row, the `fh6-render` and `fh6-core` rows, Saira Condensed, the ADAS eye the code uses). `ONBOARDING.md` section 2 lists the nine crates and section 5 is the thread flow. `SKILLS.md` drops the unconnected image connector, names `fh6-kit::tokens` as the live loader and gives one atomic lock command. `CLUSTER_PROMPT.md` puts a first build before the direction check and defines 4K | TJX accepted the tool validation proposals and the Rust assessment's doc-drift table on 2026-11-27; an independent review of commits 913fafd and 75a4b70 found the remaining gaps |
| 2026-11-27 | Docs follow-up from a read-only audit (five agents, each finding checked by a second one): the GPU job list lives only in `SKILLS.md` section 5 and names every `cargo test` that renders; the dash checklist in `CLUSTER_PROMPT.md` grows to 38 lines that a thread can answer (GPU and CPU split, no-lag line, per-view lines, evidence named), the handover is drafted before the closing review, and a dash writes its own `assets/clusters/<snake_name>/ASSETS.md` and never commits the shared ledger files; `program/HANDOVER_TEMPLATE.md` gains the dash sections; the golden packets are described as synthetic; the declared MSRV 1.87 is marked untested because the locked dependencies need about 1.90; stale Penpot, README, kit-guide and phase 10 and 11 lines are corrected | The audit confirmed 36 findings after commit 00c04c7; the ones that need TJX or a Rust thread are listed in the tool validation report |
| 2026-11-27 | The dash-thread limit is 2 at once in all four places that state it (`program/README.md` twice, `CATALOG.md`, the phase 7 `MASTER_PROMPT.md`); they said 3 and 6 before | One 8 GB GPU serves every thread, and the plan of 2026-11-27 allows two or three dash threads open at once; the coordinator set 2 |
| 2026-11-27 | A dash has a hard CPU limit for its own share: under 0.3 ms per frame, measured as the live bench's "CPU per frame" p50 with the dash minus the same number for a scaffold dash (`AGENTS.md` directive 7, checklist line 6 of `CLUSTER_PROMPT.md`, `SKILLS.md` law 9) | TJX agreed to a hard limit on 2026-11-27 ("it needs to be responsive and fast"); the coordinator set 0.3 ms. The app's host code still misses the 1 ms total (known defect 10), which a separate phase fixes. For scale: the kit's own work is 0.045 ms p50, and 18 render passes cost about 0.23 ms of submit time in `bench-render` |
| 2026-11-27 | `docs/specs/optics.md` replaces card P8-1 as the source for every glass, reflective and spectral effect. The kit gains `OpticalGlass`, `Shape::Blob` with `Piece` (a smooth union), the token block `glass.optics` (synthetic sapphire, n_d 1.768) and a CPU reference `fh6_kit::optics` with one test per equation; the legacy `Glass` stays until its users (the OSD and a few components) are rebuilt. No new dependency | TJX asked for physically correct optics. Card P8-1 gave sapphire the index of crown glass and painted its rim, fresnel and dispersion as fixed effects |
| 2026-11-27 | Liquid Glass rebuild (thread 1), first build. The dash is separate pieces of `fh6_kit::OpticalGlass` (cabochons, rod and bead, gear puck, two pills, G lens) over black, with a black core under each flat face and all light under the glass; the first pilot's capsule is gone. Rounded edges are 20 to 26 px so highlights are thin lines. No new dependency | TJX rejected the first pilot ("a dark slab with blobs") and asked for separate floating pieces that read as glass by how they bend and catch light. The direction check is open |
| 2026-11-27 | Kit: the analytic studio is a horizon ring (0 to 9 degrees above the screen, all round), a thin key strip and a thin far strip; reflections and the key light's lobe are filtered over the pixel footprint (an anisotropic sampler for environment maps at binding 10 of the shape pipeline; roughness widened with sigma^2 = 1 / 2 pi and a clamp of 0.18). `docs/specs/optics.md` §7 has the equations and sources | Clear glass on a dark ground shows only where an edge mirrors a light (dark-field lighting); a broad softbox made thick white crescents. Without footprint filtering a thin highlight on a tight edge flickers |
| 2026-11-27 | Kit: `Shape::Arc` and `Outline::Arc` have tight bounds (the arc's extent, not its whole circle), and the PNG loader skips colour arithmetic for opaque and flat texels (pixels identical; unit-tested against the slow path) | A shallow arc of a 5000 px circle covered the whole 4K frame and cost 0.5 ms GPU; two 4096 px textures took seconds to load |
| 2026-11-27 | Liquid Glass assets: the studio environment is rendered in Blender with a ComfyUI panorama on its cyclorama; the liquid-light tile is ray traced in `asset-forge` (Snell's law at a rippled water surface), not generated | The SDXL caustic did not tile and was not a caustic; a traced one is exact and seamless by construction. ComfyUI's bespoke asset in this dash is the panorama |
| 2026-11-27 | Liquid Glass, second build: every piece is a dome, so every piece is a lens. The gear is drawn at cap 156 under a dome that magnifies it about 1.15 times; an upshift sends a ripple across it (kit `Ripple`: a ring wave added to the surface height, held under 5 degrees over a numeral); the side capsules are shallow domes; the rev rod has a graduation every 1000 rpm; hard braking raises the blend length until the capsules fuse with the gear lens; the studio light leans 40% to the palette's primary colour. No colour splitting (sapphire this thin separates the primaries by under a pixel) and no light pool under the pieces (it made navy faces) | TJX on the first render: "exactly we want 2027 future not anything less just modern in all fairness". Flat-faced pills read as today's phone widgets; lenses that really magnify and a surface that really ripples do not |
| 2026-11-27 | Cloud GPU lane (thread "Set up cloud GPU lane"): `asset-forge` gains `--lane cloud`, `--masters <snake_name>`, `selftest` and the `cloud` commands (`src/cloud/`, `src/lane.rs`, `src/selftest.rs`). A rented Runpod pod (RTX 4090 first, data centre EU-RO-1) with a 30 GB network volume runs the same Blender 5.3.0, ComfyUI v0.46.0, torch build, five models and 101 pinned Python packages as this PC. asset-forge starts it through Runpod's REST API, reaches it over SSH only, and deletes it. Limits: $0.90 an hour, 10 idle minutes, 4 hours, one pod, and a budget of $50 for the whole lane with a warning from $40. The GPU lock rule has two lanes (`gpu.lock`, `gpu-cloud.lock`; `SKILLS.md` section 5, `AGENTS.md` directive 15). Guide: `docs/cloud-gpu.md`; handover: `tools/asset-forge/HANDOVER-cloud.md`. No new dependency: ureq and serde were there, and SSH is the OpenSSH client that ships with Windows | TJX approved renting a 24 GB GPU on 2026-11-27, then turned on auto-pay and set $50 for all clusters. Measured that day: SDXL 4096² in 9.1 s against 27.1 s on the local card once the model is loaded, a 4K Cycles glass render in 12.6 s against 32.4 s, a new pod ready in about 3 minutes, an idle pod gone 10 minutes after its last use. The set-up and its tests cost $0.86 |
| 2026-11-27 | Fix phase B (OLED care, OSD palette, render-thread guard): `fh6_core::oled_care` is rewritten without a clock (`update(inputs, dt)` for a shown frame, `rest(dt)` otherwise) and the render loop drives it. The orbit moves the layout anchor by whole pixels inside the safe zone. The idle dim is one `BlendMode::Multiply` fill after the cluster over the cluster, its glow reach and the scrim's dome: it scales light and keeps coverage, so the black scrim never opens; its gain is the sRGB decode of `OledDimmerMinAlpha`, so 0.35 looks like 35% opacity, as the key's name says; a dimmed frame has no bloom. A gear change wakes the dash. No new dependency | Finding 1 named the frame opacity, which would fade the scrim and show the factory gauges. A plain gain of 0.35 on light looks like about 63% opacity. No draw can reach bloom (known defect 16) |
| 2026-11-27 | Fix phase B: the dyno's frames carry the car last seen racing (`dyno_frame`), so the OSD live view over a paused game keeps the car's profile, palette and logo. A render-thread panic is reported by a drop guard as its unwind starts, closes the app (exit code 3, a 2 s limit on the join) and is logged by a panic hook for every thread | Finding 4: the palette took the dyno's ordinal 0 while the profile took the last raced car. Finding 20: the app ran on with a dead overlay and no log line. `panic = "abort"` is outside the phase (the workspace profile) and would skip the OSD's pending save (known defect 18). Open follow-up for the render kit work: the parked dim's GPU overrun on the Bosch DDU (known defect 16). One early run of the phase's GPU test (15:03 UTC, a few seconds of offscreen rendering) went without the GPU lock; every later GPU job held it |
| 2026-11-27 | Visual review (thread "Add visual review skills for dashes"): a new workspace tool, `tools/visual-review` (CPU only, no GPU lock), and the project skill `fh6-visual-review` (`.claude/skills/`, markdown only), wired into `CLUSTER_PROMPT.md` (step 5, step 10 and closing-review line 39), the handover template (`## Visual review`), `SKILLS.md`, `AGENTS.md` and `ONBOARDING.md`. No new crate: the tool uses `image` 0.25, `resvg` 0.50 (with its `text` feature) and `fh6-kit`, all already in the lockfile, and `fh6-clusters` only in its tests, so a dash module mid-edit never stops it building | TJX went hands-off on reviews, and nothing installable fitted: every candidate scored designs, reviewed web pages or code, needed Python or Node, or sent images to a service. Claude sees a 4K frame only at about 0.4x, so threads need 1:1 tiles and measurements (contrast at the blur gate, the golden harness's delta-E, a WCAG 2.3.1 flash audit, dial unwraps) to review their own renders without fooling themselves |
| 2026-11-27 | Cloud first and a shared pod: every asset-forge ComfyUI and Blender job runs on the cloud pod, which takes two runs at once (lock files `gpu-cloud.lock` and `gpu-cloud-2.lock`); the local card keeps `preview-export`, the goldens and the live bench (`SKILLS.md` section 5, `AGENTS.md` directive 15). One run at a time starts the pod (`start.lock`); each run marks itself in `runs\`, has its own job folders, upload names and forward port; ComfyUI keeps 6 GB free (`--reserve-vram 6`) and a Blender job waits for 6 GB free; `cloud down` leaves the pod up while another run uses it. The $50 budget counts GPU time only; the 400 GB volume ($28 a month) is shown beside it. No new dependency | TJX: "cant we do more ona 4090" and "we have 400gb and £50 in the cloud on a 4090"; TJX grew the volume to 400 GB and chose GPU time only for the budget. One shared pod costs no extra start or idle time, unlike a second pod |
| 2026-11-27 | Release builds use thin LTO and one codegen unit (`[profile.release]` in the workspace `Cargo.toml`); `panic` stays `unwind` (known defect 18) | Host CPU phase: wgpu's submit and encoder paths gain from cross-crate inlining; a release build is slower to link, which only release builds pay |
| 2026-11-27 | The kit uploads each frame through a ring of 3 `MAP_WRITE` staging buffers it maps again after submit (`Kit::frame_submitted`), with `Queue::write_buffer` as the fallback when no slot is ready. `map_async` boxes its callback, so each frame makes one small heap allocation inside wgpu; the kit's own code allocates nothing | `write_buffer` created a staging resource and mapped it on every frame (kit render 0.13 ms p50 to 0.035 ms). A test draws 16 frames both ways, including fallback frames, and checks they are pixel-equal |
| 2026-11-27 | The render thread joins MMCSS "Games" (fallback `THREAD_PRIORITY_ABOVE_NORMAL`), opts out of power throttling and, on a hybrid CPU, is held on the performance cores (`fh6_win::system::raise_frame_thread`; the guard undoes all three) | Its work is short and latency-bound; on the test PC the p99 fell by 0.2–0.3 ms. MMCSS can take a performance core from the game for the length of a frame (well under 1 ms) |
| 2026-11-27 | The kit's bloom pyramid is one compute pass (9 dispatches, `shaders/bloom.wgsl`, tap for tap the old fragment shaders) instead of 9 render passes | Each render pass costs about 15–20 µs of host CPU in wgpu (live: about 0.045 ms less CPU per frame, 3 to 4 passes instead of 10 to 12). Each dispatch also zeroes a 2-texel margin past its viewport, as the passes' clear did, so stale texels never bleed in (without it 16 hood-view goldens changed); all 49 goldens pass. It costs about 0.02 ms more GPU on bloom-heavy frames (Bosch DDU sequence p50 0.239 against 0.217 ms), well inside the 0.5 ms budget |
