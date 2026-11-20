# Phase 8: cockpit fit (MASTER PROMPT)

You lead Phase 8 of FH6 WheelDash v2, running in your own chat and worktree. TJX drives mostly in cockpit view. The OEM dash blocker must fit the real binnacle so our clusters sit in it like factory hardware, and the seat, FOV and cluster height must be easy to set.

## Read first
`ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`, the handovers of Phases 4 and 5, `docs/design/SCRIM_FIT.md` (from Phase 3), `docs/kit/README.md`, `crates/fh6-render/src/oem_scrim/*`, `crates/fh6-win/src/camera_patcher.rs`, `data/CameraSettings.ini`, `data/CameraPhysics.xml`, and the photos in `assets/reference/`.

## Required work
1. **The dash blocker, rebuilt on the kit** (`crates/fh6-render/src/oem_scrim/` or a new `fh6-scrim` module):
   - The dome shape follows `SCRIM_FIT.md`. Centre it on the same binnacle anchor the clusters use; today it sits at 0.34 of the height while the clusters sit at 0.845, so fix that.
   - The default width works as shipped (today `oem_dash_width_scale` defaults to 0, so the dome falls back to its 420 px minimum).
   - True feathering via SDF distance falloff.
   - A steering-column void mask.
   - Opacity from 0 to 100%; the OLED core is true black.
   - **Parameters in config:** width, height, top-arch curvature, Y offset, feather, corner softness.
   - **Per-car overrides by car ordinal:** `data/car_profiles.json` extended. The car changes automatically when telemetry's `car_ordinal` changes.
   - The ambient wash stays (it works) and is ported to the kit.
2. **Cluster placement:**
   - `Layout` anchor X/Y, **cluster height** (a vertical offset), scale and a safe zone, all per view (cockpit, hood, chase).
   - Clamp so a cluster never leaves the screen.
   - Per-car overrides go through the same profile.
3. **Camera patcher, made correct and safe** (`fh6-win`):
   - Use the real `CockpitCam\FOV` key in `CameraSettings.ini`.
   - Write the seat height (`Param.y` of the DriverCam fixed offset), the wheel offset (`DefaultOffsetFromSteeringWheelY`) and the per-view FOVs.
   - Find the game install: the Steam library folders via `libraryfolders.vdf` (pure-Rust parse), with a config override.
   - Before the first change, back up the original game files into `%LOCALAPPDATA%\FH6WheelDash\backups\` with a timestamp. Restore is one call.
   - **Write files while the game is closed.** Detect whether the game is running. If it is, queue the change and apply it when the game exits, or tell the user a restart is needed (the OSD and configurator show this).
   - **Tests on copies only.** Writing to the real game folder still needs TJX's GO, so tests must use copies of `data/` files.
4. **Views:**
   - The RB camera ring stays.
   - Add a **resync** action: an OSD item, plus hold LB+RB for 1 s (never ABXY).
   - Route views: Cockpit → cockpit cluster plus scrim; Hood → hood cluster with the scrim at 0; Chase → optional floating HUD.
5. **Fit tooling:**
   - `cargo run -p preview-export -- --fit` composites the scrim and a chosen cluster over `assets/reference/cockpit_with_wheel.jpg` at 4K, so fit can be judged against a real cockpit.
   - Do this for 3 clusters, 3 opacities and 2 heights.

## Exit criteria
- Workspace check, test and clippy are green.
- The fit renders are in `mockups/fit/`, with a design critique done and passed.
- The camera patcher passes tests on copies (cockpit FOV key, seat `Param.y`, wheel offset, backup and restore).
- Per-car profile overrides work end to end in `--demo` when `car_ordinal` is switched through `mock-udp --ordinal`.
- Hide rules still pass.
