# PROGRESS.md: status, backlog and decisions

Read this first and update it last on every task. The decision log is append-only.

## Current phase
The v2 build program is running. See [program/README.md](../program/README.md) for the roadmap, the dependency graph and the conductor protocol.

| Phase | Status |
|---|---|
| 1 Clean Rust repository | ✅ 2026-11-20 (public: T-J-X/Forza-Revampled-Telemetry) |
| 2 Program kit (`program/`, ONBOARDING, council agents 5–6) | ✅ 2026-11-20 |
| 3 Design system & art direction | ◐ started 2026-11-20 |
| 4 Engine: a working exe | ✅ 2026-11-21 (branch `claude/phase-4-engine-b8ee7b`, see `program/phases/04-engine/HANDOVER.md`) |
| 5–11 | ☐ waiting on dependencies |

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

## State of the code (after Phase 4, 2026-11-21)
| Check | Result |
|---|---|
| `cargo check --workspace` | ✅ no warnings |
| `cargo test --workspace` | ✅ 167 passed, 0 failed (the launcher test compiles again) |
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
8. Nine `TODO(phase-7)` comments in `fh6-clusters` (and one in `fh6-osd/src/render.rs`) mark values that are computed but never drive the visual. Examples: `_speed` in chrono_event_horizon, `_rpm_ratio` in f35_helmet_hmd, `_level` in kinetic_liquid_metal. The Phase 4 lint pass found them; they are for the cluster rebuilds to resolve.
9. The OSD is not drawn yet. The hold toggle (View 400 ms) and the tray's "Open OSD" only switch the live-view bypass on and off (Phase 9). The tray's "Open configurator" opens `config.json`, whose changes hot-reload, until the Phase 10 web configurator exists.
5. `fh6-render::oem_scrim::mount` (mount, visor lip, hub cap) is still a no-op.
6. `fh6-render::pool::PaintPool` is a cache stub with no real paint objects.
7. `assets/*.png` duplicates `assets/textures/4k/`. Pick one location when `AssetCache` is wired up.

## Backlog (not yet scheduled)
**Pre-05A: Liquid Glass OSD and cockpit optics**
- 60/40 OSD: the live dyno on the left and a glass control panel (alpha 22..36, 1 px rim `#FFFFFF`→`#38BDF8`) on the right.
- Low bucket seat and FOV kinematics: `Param.y = -0.045`, `DefaultOffsetFromSteeringWheelY = -0.035`, and an FOV slider from 45° to 110°, applied through `camera_patcher` without restarting the game.
- Per-camera cluster routing: Cam 3 gets its cluster plus the scrim; Cam 4 gets its cluster with the scrim at 0%; the chase cams get a floating HUD. Routing swaps automatically on RB.
- A contoured binnacle ambient wash with inverse-square falloff, alpha 12..35 at night.
- A per-cluster scaler (0.70× to 1.40×) with optical centre lock, plus typography presets.

**P5: mockups and contact sheet**
- Keep `preview-export` working: 29 clusters × 4 states, the 0/50/100% scrim, the OSD and the `index.html` gallery.
- Add the in-cockpit compositor (Phase 8 spec P8-4), using `assets/reference/cockpit_with_wheel.jpg`.

**P6: gates, benchmark, deploy**
- P6-1: golden telemetry replay through the decoder; it must match `tests/golden/packets/*.json` field for field.
- P6-2: visual regression, pixel-diffed against the approved `mockups/` baselines. Checks: no text clipping, true-black alpha cut-outs.
- P6-3: `bench-render` over 1,000 frames at 4K; mean must be under 1.8 ms and P99 under 2.4 ms.
- P6-4: a Rust deploy script with SHA256 parity to the two deploy targets in AGENTS.md. It is held for TJX's explicit GO.

**P7: launcher, remapper, Game Bar**
- P7-1: `FH6_WheelDash.exe` (standalone, tray, process watcher) and `FH6_SteamLauncher.exe` (wraps the game launch with `--auto-exit`).
- P7-2: hold-View remapper in OSD tab 3 (200 to 1000 ms in 50 ms steps). A tap under 200 ms passes through to the game. A/B/X/Y can never be selected.
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
- G4: OSD dispatch. A test exists (`crates/fh6-osd/tests/g4_dispatch.rs`).
- G5: robustness against NaN, Inf and zero-size inputs.

## Open questions for TJX
1. Which Figma file is the master: `ZW8mpHivoqjl8p7eLlvdBi` or `FsEcAnDwfFf340i8zwbBup`?
2. What is Phase 2: fixing the known defects and making clippy green, Pre-05A, or Phase 8 cluster quality?

## Decision log (append only)
| Date | Decision | Why |
|---|---|---|
| 2026-11-20 | The repo starts fresh as a Rust-only workspace at the root, with new git history | A clean base for Claude Code work and a new GitHub repo |
| 2026-11-20 | Claude Code does the work; the `program/` kit and this tracker replace the old task cards | One tracker and one set of rules |
| 2026-11-20 | `car_brands.json` is embedded with `include_str!` | Removes a hardcoded absolute path; rendering never depends on the working directory |
| 2026-11-20 | `preview-export` writes to `mockups/`, which is gitignored | Keeps about 150 MB of 4K PNGs out of git; they can be regenerated at any time |
| 2026-11-20 | Runtime data moved to `data/`; reference photos to `assets/reference/` | A clear layout. At runtime the app still reads `config.json` from its working directory |
| 2026-11-20 | `fh6-render::pool::FigmaGdiPool` renamed to `PaintPool` | Neutral naming; it was only used inside its own file |
| 2026-11-20 | MCPs: context7 plus Figma only | Enough for crate docs and design tokens |
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
