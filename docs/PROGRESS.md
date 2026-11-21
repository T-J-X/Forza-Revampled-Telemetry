# PROGRESS.md: status, backlog and decisions

Read this first and update it last on every task. The decision log is append-only.

## Current phase
The v2 build program is running. See [program/README.md](../program/README.md) for the roadmap, the dependency graph and the conductor protocol.

| Phase | Status |
|---|---|
| 1 Clean Rust repository | ✅ 2026-11-20 (public: T-J-X/Forza-Revampled-Telemetry) |
| 2 Program kit (`program/`, ONBOARDING, council agents 5–6) | ✅ 2026-11-20 |
| 3 Design system & art direction | ◐ 2026-11-21: all repo deliverables done on branch `claude/phase-3-design-system-449d2f` (PR T-J-X/Forza-Revampled-Telemetry#1), awaiting conductor review. Open: Figma library and render placement (blocked: Starter MCP cap, desktop MCP unavailable) and the "clearly better than references" bar (final verdict "matches"). Handover: [program/phases/03-design-system/HANDOVER.md](../program/phases/03-design-system/HANDOVER.md). |
| 4 Engine: a working exe | ◐ started 2026-11-20 |
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
1. `crates/fh6-app/tests/steam_launcher_tests.rs` doesn't compile: `assert_eq!` compares `&&str` with `&str`.
2. Clippy `collapsible_if` in `fh6-core` blocks `clippy -D warnings`.
3. `FH6TelemetryData` has no `torque` field, so `GlyphDotMatrix` derives torque from power and RPM. Decide whether to add the field (check the packet layout first) or drop torque from the design.
4. 50 compiler warnings, mostly unused dependencies such as `windows`, `tracing`, `thiserror` and `byteorder`.
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
1. ~~Which Figma file is the master?~~ Answered by Phase 3: the new file `0zOEwnJ8dEj1kQ7QwPGEH0` (docs/design/FIGMA.md). Neither legacy key is used.
2. What is Phase 2: fixing the known defects and making clippy green, Pre-05A, or Phase 8 cluster quality?
3. Figma is on a Starter plan: 3 pages, 1 variable mode, and an MCP call cap that stopped Phase 3 before the component library and render placement were finished. Upgrade the plan (Professional), or accept the HTML boards as the visual source and finish Figma by hand?
4. AGENTS.md (Design Council 3D) still quotes the old ADAS eye (0, 1.85 m, −3.4 m). Phase 3 raised it to (0, 3.4 m, −6.0 m) because the old camera hid the road behind the ego car. OK to update AGENTS.md?
5. Default shift colour: Phase 3 chose violet `#978DFC` (race convention, lifted to ≥ 7:1); brands whose `ShiftFlashHex` is white strobe white. Keep, or default to white?

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
| 2026-11-21 | Phase 3: the Figma master is `0zOEwnJ8dEj1kQ7QwPGEH0` (Starter: 3 pages organised with Sections, per-family palettes as variables because only 1 mode is allowed) | Created for v2; the legacy keys were never the v2 master |
| 2026-11-21 | Phase 3 visuals are pure-SVG HTML boards in `docs/design/boards/`, rendered to PNG with headless Edge; the generator script stays outside the repo | Figma's Starter MCP cap blocked rendering in Figma, and the repo is Rust-only |
| 2026-11-21 | `data/figma_tokens.json` schema v3 (3.2.0): colour roles, palette modes, OFL type scale, glow, glass, light, motion, layout, states, components, families, accessibility; legacy keys kept; `colors`/`grid` names avoided so `fh6-render::tokens` still parses | One token source for Figma, the kit and every cluster (docs/design/TOKENS.md) |
| 2026-11-21 | Default fonts: Saira Condensed (display, motorsport default), Titillium Web (labels), plus Barlow Semi Condensed, Michroma, Chakra Petch, Exo 2, B612 Mono per family; all OFL | Research (RESEARCH.md §9); Bahnschrift is a host font, not OFL |
| 2026-11-21 | Seven families: Classic Analog, Modern Digital 2027, Race Dash (new), Liquid Glass, Sci-Fi, Fluid / Generative, ADAS & HUD; `F35HelmetHmd` moves to ADAS & HUD | Research found a race-dash architecture no other family owns, and aviation HUD shares the ADAS grammar |
| 2026-11-21 | Limiter strobe 10 Hz, 50% duty, only on a shift strip ≤ 42 px tall; nothing larger flashes above 3 Hz; red washes are steady | WCAG 2.3.1 and the 60/120 Hz frame maths (RESEARCH.md principle 2) |
| 2026-11-21 | Shift lights fill outside-in on per-car, per-gear thresholds; unlit pods are neutral grey lenses; default shift colour violet `#978DFC` | Race convention; colour only for lit state |
| 2026-11-21 | Only glance ranks 1–3 may glow; one virtual light per cluster moves with lateral G and drives every rim | Research principles 5 and 6; critique round 1 |
| 2026-11-21 | ADAS chase eye raised to (0, 3.4, −6.0) m with 8° pitch | Critique: the old eye (0, 1.85, −3.4) hid the road behind the ego car |
| 2026-11-21 | Classic dials default to a symmetric 270° face; shift-at-12 is an optional per-car Track mode | A linear 270° face rotated to put shift at 12 o'clock turns lopsided |
| 2026-11-21 | Catalog: 29 rebuilds + 19 new concepts, pattern-setters LiquidGlass, Gt7HoodPro, PorscheAnalog, BoschDdu, TeslaAdas, RetroCyber, KineticLiquidMetal (build order in CATALOG.md) | Defaults first, then one pattern-setter per family so helpers exist before siblings |
