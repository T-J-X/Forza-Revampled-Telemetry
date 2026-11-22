# HANDOVER: Phase 8 · Cockpit fit

**Chat / branch:** `claude/phase-8-cockpit-fit-7daeb6`  **Date:** 2026-11-22  **Result:** ✅ complete (two open items need TJX or the game; see Known issues)

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence |
|---|---|---|
| `cargo check`, `cargo test` and `cargo clippy` are green across the workspace | ✅ | Verification output below: check clean, 325 tests pass and 0 fail, clippy `-D warnings` clean, fmt clean, parity 20/20, wasm build OK |
| Fit renders exist in `mockups/fit/`, and a design critique has been done and passed | ✅ | `cargo run --release -p preview-export -- --fit` writes 18 composites (3 clusters × 0/50/100% × heights 0/+170) and 2 low-seat frames, plus crops, thumbs and `index.html`. The `ui-ux-designer` critique verdict was **PASS-WITH-NOTES**. Its notes 1 and 7 (dome riding the height offset, heavy wash at larger scale) are fixed; the rest are under Known issues |
| Camera patcher tests pass on copies: cockpit FOV key, seat `Param.y`, wheel offset, backup and restore | ✅ | `fh6-win::camera_patcher` tests copy `data/CameraSettings.ini` and `data/CameraPhysics.xml` into a temp install: `writes_the_real_cockpit_fov_key_and_keeps_every_other_byte`, `writes_the_seat_param_y_in_drivercam_only`, `writes_the_wheel_offset`, `backs_up_once_and_restores_in_one_call`, `nothing_is_written_while_the_game_runs`, plus 4 more. `data/` is unchanged (`git status data/` is clean) |
| Per-car profile overrides work end to end in `--demo` when `car_ordinal` is switched via `mock-udp --ordinal` | ✅ | Live `FH6_WheelDash --demo --game-process fh6-stand-in.exe` with a scratch `car_profiles.json`. `mock-udp --ordinal 4122` showed the default fit (`docs/handover-shots/phase8-car-4122-no-profile.png`). `--ordinal 3595` showed the cluster 160 px higher, the scrim at 40% and the dome 1.3× wider (`phase8-car-3595-profile.png`). The log shows `car profile applied car_ordinal=3595`. Unit test: `render_loop::car_config_switches_with_the_ordinal_and_caches` |
| The hide rules still pass | ✅ | Live, without `--demo`: race on showed the cluster and scrim (`phase8-race-on-scrim.png`); `--race-on 0` hid everything (`phase8-race-off-hidden.png`). The `fh6-core::visibility` and `render_loop::uses_dyno` tests pass. The scrim is drawn inside the same frame as the cluster, so it hides in the same frame |

## What was built
- **Dome scrim on the kit**
  - `fh6_kit::Shape::Dome` / `Dome`: a superellipse with separate top and bottom exponents and a direction-dependent feather (top / side / bottom corners). Its coverage is a smoothstep fall-off with ±0.5 LSB triangular dither and a round-topped steering-column void cut out.
  - In the shader: `sd_dome_core`, `dome_feather`, `sd_dome_void` and a dedicated fill path in `shape.wgsl`.
  - `fh6_kit::scrim`: `dome`, `draw_scrim`, `draw_wash` and `Wash`, with the wash a 1.5 px lip plus a 3 px halo along the top arc, alpha 12–35.
- **Fit model in `fh6_core::fit`**
  - `ScrimFit::from_config(cfg, FitView)`: the SCRIM_FIT §2 dome at anchor + (0, −75), 640 × 470, n = 2.6, feather 96/120/140, column void 220 px from anchor + 300. Opacity is 0 in hood and chase.
  - `ScrimFit::coverage` is the CPU mirror of the shader.
  - `ViewPlacement`, `clamp_anchor` and `FitView`.
- **Placement:** `fh6_kit::Layout::from_config` now uses the per-view anchor, height, scale and safe zone, and clamps the envelope on screen (`View::fit_view`).
- **Config:** new keys in `AppConfig` (listed below), with `AppConfig::sanitized` and migration of the old `extra.ClusterHeightOffsetPx`.
- **Per-car profiles**
  - `fh6_core::profiles` is rewritten: `CarProfile` is an all-`Option` overlay, `CarProfilesFile` v2 (`data/car_profiles.json`), with `effective_config`, `ProfilesHandle` and hot reload in `ConfigWatcher`.
  - `render_loop::CarConfig` re-resolves only when the config, the profiles or the car change.
- **Render loop:** draws scrim, then wash, then cluster in Cockpit; scrim 0 in Hood; the chase HUD only when `ChaseHudEnabled`, using `ChaseHudStyle`.
- **Resync**
  - `fh6_input::ResyncConfig`: hold LB + RB for `ResyncHoldMs` with the OSD closed. A/B/X/Y are never read.
  - `InputShared::resync_camera(&cfg)` for the OSD item.
- **Camera patcher, rewritten in `fh6-win`**
  - `camera_patcher`: `CameraPatch`, `patch_settings_text`, `patch_physics_text`, `apply`, `restore`, `ensure_backup` and `original_backup`.
  - `steam`: a pure-Rust VDF parser, `library_paths`, `find_game_install`, and the registry `SteamPath`.
  - `fh6-app::camera_sync`: an opt-in service that queues while the game runs.
  - `fh6_core::camera_status`: `CameraPatchHandle`, `CameraPatchState` and `CameraRequest`.
  - `GET /api/camera` and a status-page row on `127.0.0.1:5301`.
- **Legacy CPU twins:** the tiny-skia `oem_scrim::scrim` (`LocalFrame`, `render_oem_scrim`, `render_scrim_fit`) and `ambient_wash` (`BinnacleAmbientWash::render_for`) were rewritten on `ScrimFit`. The old 0.34-height scrim and `dome_path.rs` are gone.
- **Tools**
  - `preview-export --fit` (`tools/preview-export/src/fit.rs`).
  - The kit scrim cards `Scrim/OemScrim_{0,50,100}pct.png`.
  - A `bench-render` scrim scenario.
  - `Painter` now trims instance quads to an axis-aligned clip (not glass).

## Verification output
```
cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s)
    (one cargo manifest note: tools/web-preview `unused dependency fh6-clusters`, unchanged from main)
cargo test --workspace            -> 41 test binaries, 325 passed, 0 failed (--no-fail-fast; fh6-win quit_signal can collide with another session running the overlay or its tests at the same time: it uses a system-wide named event; 3/3 alone)
cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)   (no warnings)
cargo fmt --all -- --check        -> clean
cargo run -p parity-dump -- tests/golden/packets   -> 20/20 OK
cargo build -p fh6-kit -p fh6-clusters --target wasm32-unknown-unknown -> Finished
cargo run --release -p bench-render -- --frames 300 (RTX 3070 Ti Laptop, 3840x2160)
  Liquid Glass sample (bloom on)               GPU p50 0.213 p99 0.220 ms | 19 draws
  Liquid Glass sample + OEM scrim 100% + wash  GPU p50 0.380 p99 0.390 ms | 19 draws
```

## Visuals
- `mockups/fit/index.html` and its frames (gitignored; regenerate with `cargo run --release -p preview-export -- --fit`, about 1 minute).
- Committed: `docs/handover-shots/phase8-fit-*.png` (fit thumbs) and `docs/handover-shots/phase8-car-*.png` / `phase8-race-*.png` (live captures).

## My calls
- **The dome is drawn in cluster local space.** It follows the anchor and scale, but it cancels the cockpit height offset, so it stays on the factory binnacle when the user raises the cluster. The critique showed it otherwise rising into the windscreen. SCRIM_FIT §2 and §4 are updated.
- **Car profiles are an `Option` overlay.** A missing key means "inherit"; v1 zeros used to collapse the width to 0. Styles were dropped from the shipped profiles. The per-car seat (`CockpitOffsetY`) is recorded but not applied live: the seat is a camera-file value and needs a game restart.
- **Per-view scale multiplies `ClusterScale`.** The product is clamped to 0.70–1.40, and chase defaults to 0.8.
- **Camera file mapping**
  - `CockpitCam\FOV` ← `WheelLessCockpitFOV` (the overlay's cockpit view [3]).
  - `Driver\FOV` ← `StandardCockpitFOV`.
  - Seat: the last DriverCam `ConstantOne / None / CarSpaceXYZOffset` layer gets `Param.y` ← `DriverSeatHeightM`, with `Param.x/z` ← `CockpitOffsetX/Z`.
  - Wheel: `Driver\DefaultOffsetFromSteeringWheelY` ← `SteeringWheelOffsetYM` only; X and Z are untouched. The old patcher wrote Z from the seat offset.
  - Hood: only the INI `HoodCam\` keys. The hood XML layer is no longer written.
  - FOVs are clamped to the file's 40–110.
  - Formatting keeps each value's decimals; CRLF, the trailing newline and every other byte are preserved; files are written UTF-8 without BOM, atomically.
- **Backups and auto-apply**
  - Backups go to `%LOCALAPPDATA%\FH6WheelDash\backups\<YYYYMMDD-HHMMSS UTC>\`. Only the first backup is kept as "the original"; restore uses it.
  - `CameraPatchAuto` defaults to **false**: nothing touches the game folder unless TJX turns it on or the OSD/configurator sends `CameraRequest::Apply` (deploy guard).
- **Resync:** the RB edge that starts the LB + RB chord still advances the ring, which is presumably what the game does with that RB. Once held, the index is set outright to `ResyncCameraIndex` (default 3). The other two options, suppressing that RB edge or starting from a different camera, are a config change away if TJX's game behaves differently.
- **`--fit` uses Analog at user scale 1.25 as the third cluster.** Only two kit clusters exist until Phase 7, and the v1 Bosch DDU renderer ignores the `Layout` anchor (it drew a grey box mid-windscreen). Two low-seat frames were added.
- **New dependency:** `image` 0.25 with `jpeg`, in `preview-export` only, to decode the reference JPEGs. Logged in PROGRESS. The `windows` crate gained the `Win32_System_Registry` feature (same crate).
- **Painter clip trim:** clipped quads are trimmed to the clip when the transform has no rotation, and glass is excluded because its optics use the full bounds. This took the scrim and wash from +0.25 ms to +0.17 ms GPU.

## Known issues / debt
- **Critique notes not acted on**
  - *(3) Vent and centre-screen overlap at default width in the stock-seat photo.* SCRIM_FIT's own numbers conflict here: sides at 1280…2560 against a vent at 1300–1460 and a screen at about 2390+ in this photo. The width stays as shipped, per the MASTER_PROMPT ("default width works as shipped"). Tune per car with `OemDashWidthScale` / `OemDashOffsetX` after real captures.
  - *(2) Dome scales linearly with the cluster.* This is as specified.
  - *(4) 50% looks muddy.* The opacity is linear as specified. A perceptual curve is a one-line change in `ScrimFit::from_config` if TJX wants it.
  - *(5) The Liquid Glass sample has a teal glow disc behind the capsule.* Phase 7 owns that.
  - *(6) The column void exposes the shifter in the low-seat view when raised.* Set `OemDashColumnVoidPx` to 0 per car.
  - *(9) Near-black banding needs a check on the C1 panel itself.*
- **Per-car reference captures (SCRIM_FIT §5)** need the real game at the patched seat and FOV for 3 cars. They were not possible here; the profile fields to store the results exist.
- **Camera patcher open questions (97% rule; need TJX)**
  - Does FH6 use `CockpitCam\FOV` or `Driver\FOV` for the wheel-less view [3]? Both are written, from two config keys.
  - Does the LB + RB chord's RB still change the in-game camera?
  - Nothing was written to the real game folder.
- **Scrim cost:** the dome fill is about 0.15 ms GPU at 4K (a full-dome superellipse with `pow`). It fits the 0.5 ms per-cluster budget with Liquid Glass (0.38 ms). An early-out inside the core would halve it if a heavier cluster needs the room.
- **Focus-loss hide** was not re-run live in this phase. It is covered by the visibility unit tests and the Phase 4 shot `docs/handover-shots/05-focus-lost-hidden.png`, and nothing in that path changed.

## What the next phases must know
**Phase 9 (OSD) and Phase 10 (configurator): config keys** (PascalCase in `config.json`, all hot-reloaded, all clamped by `AppConfig::sanitized`)

| Key | Default | Range / meaning |
|---|---|---|
| `OemDashOpaquePct` | 100 | 0–100, core opacity in Cockpit |
| `OemDashWidthScale`, `OemDashHeightScale` | 1.0 | 0.5–1.6 × the 640 × 470 dome |
| `OemDashTopArch` | 0.7 | 0–1 → exponent 4 − 2·v (0 flat crown, 1 round) |
| `OemDashCornerSoftness` | 0.7 | 0–1, same mapping for the lower half |
| `OemDashOffsetX`, `OemDashBinnacleOffsetY` | 0 | ±600 px, dome offset from anchor + (0, −75) |
| `OemDashFeatherPx` | 96 | 8–240 (sides × 1.25, bottom × 1.46) |
| `OemDashColumnVoidPx` | 220 | 0–600, void width (0 = none) |
| `AmbientGlowEnabled`, `AmbientGlowIntensity` | true, 0.25 | wash alpha 12 + 23·intensity (of 255) |
| `NightAmbientGlowEnabled`, `NightAmbientGlowIntensityPct` | false, 0 | lifts the wash intensity to at least pct/100 |
| `CockpitAnchorX/Y`, `BonnetAnchorX/Y`, `ChaseAnchorX/Y` | 1920/1835, 1920/1760, 1920/1900 | 4K px |
| `CockpitHeightOffsetPx`, `BonnetHeightOffsetPx`, `ChaseHeightOffsetPx` | 0 | ±240, positive raises (the dome does not move) |
| `ClusterScale` × `CockpitClusterScale` / `BonnetClusterScale` / `ChaseClusterScale` | 1.0 × 1.0 / 1.0 / 0.8 | product clamped 0.70–1.40 |
| `CockpitSafeZone`, `BonnetSafeZone`, `ChaseSafeZone` | `{SidePx:160, TopPx:0, BottomPx:40}` | the envelope is clamped inside |
| `ChaseHudEnabled`, `ChaseHudStyle` | true, "" | "" = the cockpit style |
| `ResyncCameraIndex`, `ResyncHoldMs` | 3, 1000 | 0–5; 500–3000 |
| `CameraPatchAuto`, `GameInstallDir` | false, "" | "" = Steam lookup |
| `WheelLessCockpitFOV`, `StandardCockpitFOV`, `BonnetFOV`, `ChaseNearFOV`, `ChaseFarFOV`, `BumperFOV` | 80, 80, 95, 62, 68, 78 | clamped 40–110 when written |
| `DriverSeatHeightM`, `SteeringWheelOffsetYM`, `CockpitOffsetX/Z` | −0.045, −0.035, 0 / −0.5 | metres |

**APIs**
- **Scrim preview in the OSD:** `ScrimFit::from_config(&cfg, FitView::Cockpit)`, then `fh6_kit::scrim::draw_scrim(p, &fit)` and `draw_wash(p, &fit, Wash::from_config(&cfg, palette.primary))` inside `kit.begin_frame(&layout)`, before the cluster.
- **Resync item:** call `input.resync_camera(&cfg)` (`fh6_input::InputShared`).
- **Camera status:** create one `fh6_core::CameraPatchHandle` (today `app.rs` creates it and hands it to `camera_sync::spawn` and `widget_server::spawn`; pass the same clone into the OSD).
  - Read `handle.status()`: a `CameraPatchStatus` with `state`, `install_dir` and `backup_dir`. `state.message()` is a ready one-line text, and `state.restart_needed()` is true for `QueuedUntilGameExits`.
  - Send `handle.request(CameraRequest::Apply)` or `CameraRequest::Restore`.
  - Over HTTP: `GET http://127.0.0.1:5301/api/camera` returns `{state, message, restartNeeded, installDir, backupDir}`.
- **Per-car profiles:** `data/car_profiles.json` v2, `{"Version":"2.0.0","Profiles":{"<ordinal>":{CarOrdinal, CarName, …any key from the table above as an override}}}`.
  - `CarProfilesFile::upsert` and `save` (atomic, no BOM) are what the configurator's "save for this car" should call.
  - The watcher hot-reloads it, and the render loop applies it when `PerCarProfileEnabled` is true.

**Gotchas**
- The repo's `.rs` files are CRLF (`rustfmt.toml` `newline_style = "Windows"`). Regex edits with `\n` silently miss.
- `ScrimFit.center` / `void_top` are in cluster local units around the *drawn* origin, which is the anchor after the height offset.
