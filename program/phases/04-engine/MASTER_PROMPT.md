# Phase 4: engine, a working exe (MASTER PROMPT)

You lead Phase 4 of FH6 WheelDash v2, running in your own chat and worktree. Today `FH6_WheelDash.exe` draws nothing: there is no window, no UDP listener, no XInput and no frame loop, and `--auto-exit` quits after about 3.5 s. Your job is to make it a real, low-latency overlay. Phase 5 builds the full render kit on the GPU foundation you lay; you prove that foundation with one simple test cluster.

## Read first
`ONBOARDING.md`, root `AGENTS.md` and `SKILLS.md`, this folder's `AGENTS.md` and `SKILLS.md`, `docs/PROGRESS.md`, and these crates:
- `crates/fh6-app` (`main.rs`, `bin/steam_launcher.rs`, `tray.rs`, `single_instance.rs`, `process_watcher.rs`)
- `crates/fh6-core` (`udp.rs`, `config.rs`, `telemetry.rs`, `cars.rs`)
- `crates/fh6-input` (`xinput.rs`, `tracker.rs`, `hold_state.rs`)
- `crates/fh6-win` (`layered.rs`, `camera_patcher.rs`)

## Goal
Launching `FH6_WheelDash.exe` while `cargo run -p mock-udp` runs shows a GPU-rendered test cluster at the binnacle anchor:
- over the desktop
- click-through and always on top
- updating on every packet
- hidden within 1 frame when `IsRaceOn == 0` or when Forza isn't the foreground window

## Required work
1. **ADR first.** Use `engineering:architecture` to write `docs/adr/0001-gpu-overlay.md`: how a transparent, premultiplied-alpha, topmost, click-through window is presented with wgpu.
   - **Preferred route:** a wgpu DX12 surface on a DirectComposition visual (`SurfaceTargetUnsafe::CompositionVisual` or whatever the current wgpu equivalent is), with `CompositeAlphaMode::PreMultiplied`, on a `WS_EX_NOREDIRECTIONBITMAP | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` window.
   - Check the current wgpu and windows-crate APIs with context7 before writing code (97% rule).
   - Record the fallback and its latency cost.
   - Note that Forza must run borderless or windowed-fullscreen for any overlay to show. The configurator will tell users this later.
2. **New crate `crates/fh6-gpu`:**
   - wgpu device, queue and surface ownership
   - frame pacing on vsync (`PresentMode::Mailbox` if available, otherwise `Fifo`, with the lowest-latency setting)
   - a resize-safe surface
   - an offscreen render-to-texture path so tests and `preview-export` can render without a window
   - a minimal pipeline (a full-screen-triangle or quad shader) used by the test cluster
3. **Window and loop** in `fh6-win` and `fh6-app`:
   - Create the overlay window on the monitor Forza is on (primary by default) at native resolution and DPI-aware (per-monitor v2).
   - A Win32 message loop on the main thread.
   - The render loop wakes when a new telemetry packet arrives (an event or condvar from the UDP thread), or at least on every vsync.
   - **No queue:** always draw the latest packet.
4. **Telemetry:** start `fh6-core::udp::TelemetryEngine`.
   - Ports come from config (8000 primary, 5500 fallback, 5607 forward); remove the hardcoded ports.
   - Delivery is lock-free (ArcSwap) to the render thread.
5. **Visibility controller:**
   - Hide when `IsRaceOn == 0`, when the packet stream goes stale (more than 250 ms), or when the foreground window doesn't belong to `ForzaHorizon6.exe`. Check with `GetForegroundWindow` plus the process image name, cached and polled at 10 Hz or more.
   - Fade over about 120 ms. Hiding must be complete within 1 frame of the trigger, so a fade-out is allowed only while still racing.
   - A `--demo` flag, and the OSD's live dyno, can bypass this.
6. **XInput:** real `LoadLibraryW("xinput1_4.dll")`, falling back to `xinput9_1_0.dll`; `GetProcAddress("XInputGetState")`, plus ordinal 100 (`XInputGetStateEx`) for the Guide button when it is available.
   - Poll at 60 Hz or more on its own thread.
   - Unify the three hold-toggle implementations into one, in `fh6-input::hold_state`: default hold View `0x0020` for 400 ms, configurable, and ABXY stripped.
   - Keep the RB camera ring logic.
7. **Config:**
   - Find `config.json` next to the exe (`std::env::current_exe`), falling back to `data/config.json` in development.
   - Fix serde naming so `data/config.json` loads correctly (for example `StandardCockpitFOV`). Missing fields take `AppConfig::default()` values, never zero.
   - Hot-reload when the file changes, because Phase 10's configurator writes it.
   - Add tests that load `data/config.json` and assert real values.
8. **App shell:**
   - **Tray:** a real `Shell_NotifyIconW` with the menu Show/Hide overlay, Open configurator, Open OSD, Quit.
   - **Single instance:** keep it.
   - **Process watcher:** real; replace `process_exists_stub`.
   - Remove the 3.5 s auto-exit bug. `--auto-exit` quits only when the game process has actually exited.
9. **Steam launcher:** works as the launch option `"<install dir>\FH6_SteamLauncher.exe" %command%`.
   - Start WheelDash hidden, then the game with its arguments, wait for the game, then signal WheelDash to quit.
   - Remove the invalid `steam://rungameid/ForzaHorizon6` fallback.
   - Fix `crates/fh6-app/tests/steam_launcher_tests.rs` so it compiles.
10. **Safety:** fix `fh6-win::camera_patcher::bind_car_brand`. It uses lowercase keys and would wipe the real `car_brands.json` (which has `Cars` / `BrandPalettes`). Make it match the real schema, never truncate, and test it against a copy of the real file.
11. **Latency instrumentation:** timestamp each packet on receive and at present, then log p50/p99 for receive-to-present and the CPU frame time.
    - Add a `--stats` overlay text and a `--bench-seconds N` mode that writes `docs/perf/engine.md`.
    - Replace the fake `bench_dirty_rect`.
12. **Test cluster:** a deliberately simple GPU cluster at the configured cockpit anchor, driven by live telemetry: a speed number and a gear, plus an RPM bar. It proves the whole path. Real clusters arrive in Phases 5 and 7.

## Exit criteria
- `cargo check --workspace` and `cargo test --workspace` are fully green (the launcher test now compiles), and `cargo clippy --workspace -- -D warnings` is green.
- Run manually and record in the handover with screenshots (`docs/perf/` or `docs/handover-shots/`):
  - **Running the exe plus `mock-udp`:** the overlay shows and updates.
  - **`IsRaceOn = 0`:** it hides.
  - **Another window focused:** it hides.
- `docs/perf/engine.md` reports p50/p99 receive-to-present and CPU frame time at 3840x2160 (target: under 1 frame and under 1 ms CPU).
- The config loads correct values from `data/config.json` (tested) and hot-reloads.
- `bind_car_brand` is safe and tested.
- `docs/adr/0001-gpu-overlay.md` exists.

## Out of scope
Real cluster designs, the render kit (bloom, glass, text), the OSD, the camera patcher's FOV/seat fixes (Phase 8) and the configurator (Phase 10).
