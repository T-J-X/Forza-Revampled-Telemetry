# HANDOVER: Phase 4 · Engine, a working exe

**Chat / branch:** Phase 4 chat, `claude/phase-4-engine-b8ee7b`  **Date:** 2026-11-21  **Result:** ✅ complete

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence (command output, file, screenshot) |
|---|---|---|
| `cargo check --workspace` and `cargo test --workspace` fully green (the launcher test now compiles) | ✅ | `cargo test --workspace`: 182 passed, 0 failed. `crates/fh6-app/tests/steam_launcher_tests.rs` now tests `fh6_app::launcher::plan` (4 tests) |
| `cargo clippy --workspace -- -D warnings` green | ✅ | `cargo clippy --workspace --all-targets -- -D warnings` exits 0 (stricter than required: all targets) |
| Exe + `mock-udp`: the overlay shows and updates | ✅ | `docs/handover-shots/01-live-cluster-a.png` → `01-live-cluster-b.png` (84 mph / 3rd / 802 rpm → 97 mph / 4th / 6267 rpm, about 1 s apart), `02-stats.png` |
| `IsRaceOn = 0`: it hides | ✅ | `03-race-off-hidden.png`: the race-off stream replaced the racing stream mid-race, and the log reads `overlay hidden reason=Some(RaceOff)` at 16:37:29.706 (final code). `04-visible-again.png`: racing again |
| Another window focused: it hides | ✅ | `05-focus-lost-hidden.png` (final code: `overlay hidden reason=Some(NotForeground)` at 16:37:00.685). On the earlier build the log showed `foreground changed game_foreground=false` at 11:34:23.577285 and the hide at 11:34:23.578287, 1 ms later |
| `docs/perf/engine.md` reports p50/p99 receive-to-present and CPU frame time at 3840x2160 (target: under 1 frame and under 1 ms CPU) | ✅ | See `docs/perf/engine.md` (numbers repeated below) |
| Config loads correct values from `data/config.json` (tested) and hot-reloads | ✅ | `crates/fh6-core/tests/config_file.rs` (real values incl. `StandardCockpitFOV` 80, scales, ports; save round-trip without BOM; watcher reload that ignores a half-written file). Live: `06-hot-reload-before.png` → `07-hot-reload-after.png` (anchor 1835→1480, scale 1.45→1.0; log `config reloaded`) |
| `bind_car_brand` is safe and tested | ✅ | `crates/fh6-win/src/car_brands.rs`: 5 tests, run against a copy of the real `assets/car_brands/car_brands.json` (693 cars kept, palettes and key order unchanged, `TotalCars` updated, legacy lowercase file refused untouched, truncated file refused) |
| `docs/adr/0001-gpu-overlay.md` exists | ✅ | Written first; implementation notes added at the end |

## What was built
**ADR:** `docs/adr/0001-gpu-overlay.md`. wgpu 30 DX12 with `Dx12SwapchainKind::DxgiFromVisual` (a DirectComposition swapchain), `CompositeAlphaMode::PreMultiplied`, Mailbox (else Fifo), `desired_maximum_frame_latency = 1`, on a `WS_EX_NOREDIRECTIONBITMAP | LAYERED | TRANSPARENT | TOPMOST | TOOLWINDOW | NOACTIVATE` window. Fallback B (readback + `UpdateLayeredWindow`) is documented with its latency cost but not implemented.

**`crates/fh6-gpu` (new):**
- `GpuContext`: DX12 instance, hardware adapter, WARP fallback.
- `OverlaySurface`: premultiplied alpha required, `Bgra8Unorm`, resize, lost-surface handling, per-frame timings.
- `OffscreenTarget`: readback to RGBA for tests, the bench and `preview-export`.
- `QuadRenderer` / `QuadBatch`: one instanced draw of SDF rounded rects and seven-segment glyphs (`src/shaders/quad.wgsl`). Output is `E(c)·a`. The batch has fixed capacity and never reallocates.
- `seven_seg`: tabular segment text through a `StackStr` (no heap).
- `test_cluster`: speed, gear, a 30-LED RPM bar with shift/redline zones, RPM, throttle/brake bars, and the `--stats` lines.

**`crates/fh6-core`:**
- `config.rs`: struct-level `#[serde(default)]` with real defaults; renames `StandardCockpitFOV`, `ShowBonnetGTWidgets` and the other acronyms (old spellings accepted as aliases); sanitising (zero ports and scales, the ABXY mask, hold-time bounds); BOM-tolerant load; `resolve_config_path` (beside the exe, then `data/config.json` in a parent); `TelemetryPorts`; new key `UdpListenAllInterfaces` (default false).
- `config_watch.rs`: `ConfigWatcher` / `ConfigHandle`, polling mtime and size every 250 ms and publishing through `ArcSwap`.
- `udp.rs`: blocking receive (the old 10 ms sleep loop is gone), ports from config, `TelemetryFrame { data, received_at, seq, race_resumed_at }` through `ArcSwap`, a notify hook (`Thread::unpark`), a source-port lock across 8000/5500 (no doubled or reordered packets), forwarding to a precomputed address, loopback bind by default, and a start that leaves nothing half-running so the app's 1 s retry works.
- `visibility.rs`: `VisibilityController`. Rule order: `bypass` (only `--demo` / `--bench-seconds`) > not foreground > OSD live view (`live_preview`) > stale > race off. The live view therefore lifts race-off and stale but never the focus rule. Hard hides land on the next frame; a 120 ms fade-out only for a user hide while racing; a 120 ms fade-in.
- `atomic_file.rs`: unique temp, fsync, rename; orphaned temps from a crashed save are swept by the next save. `config.json`, `car_brands.json` and `car_profiles.json` all save through it.
- `telemetry.rs`: same offsets, no `unwrap`, `PACKET_LEN`, `thiserror` error; the dead duplicate `TelemetryEngine` stub was removed.

**`crates/fh6-input`:**
- `xinput.rs`: `LoadLibraryExW(System32)` for `xinput1_4.dll`, falling back to `xinput9_1_0.dll`. `XInputGetState`, plus ordinal 100 into a padded `XINPUT_STATE` buffer. Correct struct layout, including `dwPacketNumber`.
- `hold_state.rs`: `HoldToggle`, the only hold implementation.
- `tracker.rs`: the RB ring, the hold toggle, tab edges and Guide, with time passed in for tests.
- `poller.rs`: a 125 Hz thread publishing through `InputShared` atomics.
- The OSD crate's own View+Menu timer was removed (`fh6-osd/src/toggle.rs` now only applies a toggle plus the ESC auto-pause).

**`crates/fh6-win`:**
- `overlay.rs`: the overlay window, and `OverlayHandle` (raw-window-handle; keeps the HWND alive while a surface exists). An outside `WM_CLOSE` becomes a clean quit.
- `app_window.rs`: hidden message window, `Shell_NotifyIconW` tray with menu (Overlay visible, Open configurator, Open OSD, Quit), re-added on `TaskbarCreated`; timers; message loop. The window state is an `Rc` held by the window and by each message being dispatched, so a handler may even drop its own `AppWindow`; `WindowCtl` calls are no-ops once the window is destroyed.
- `foreground.rs`: `EVENT_SYSTEM_FOREGROUND` WinEvent hook plus a 10 Hz poll, with the process image name cached per window.
- `quit_signal.rs`: the `Local\FH6_WheelDash_Quit` event.
- `system.rs`: single instance, process list, `ShellExecuteW`, DWM timing.
- `monitor.rs`: per-monitor-v2 DPI and monitor rectangles.
- `capture.rs`: GDI screenshot.
- `stand_in.rs`: test window.
- `car_brands.rs`: safe binding.
- Removed: `layered.rs`, with `FastLayeredRenderer` and the fake `bench_dirty_rect`.

**`crates/fh6-app` (no `unsafe`):**
- `app.rs` wires the threads and owns shutdown order.
- `render_loop.rs`: wake on packet or each refresh while visible; always the latest packet; one transparent frame to hide (retried on the next refresh if the swapchain skipped it); anchor and scale from config scaled from 4K; RB camera 4 selects the bonnet anchor; stats; bench. GPU loss: on any wgpu error or lost device the device and swapchain are dropped (the window shows nothing) and recreated once a second, also while hidden.
- `latency.rs`: fixed sample rings and allocation-free percentiles.
- `perf_report.rs`: writes `docs/perf/engine.md`.
- `args.rs`, `launcher.rs`, `logging.rs`.
- `widget_server.rs`: `/api/state` JSON plus a no-script status page.
- `process_watcher.rs`: `--auto-exit` only after the game was seen and has exited.
- `bin/steam_launcher.rs`: start WheelDash `--auto-exit`, start `%command%`, wait (including a relaunched copy), then set the quit signal. The `steam://rungameid` fallback was removed.

**Tools:**
- `tools/bench-render`: an offscreen GPU bench, now in the workspace.
- `tools/screen-capture`: new.
- `tools/stand-in-game`: new; binary `fh6-stand-in`.
- `tools/mock-udp`: `--race-on`, the `menu_pause` scenario, and the unused dependency dropped.
- `tools/parity-dump`: CRLF-insensitive compare, exit code reflects the result.

**Lint debt:** `fh6-render`, `fh6-clusters`, `fh6-osd` and `preview-export` are now clippy-clean with lint-only edits. An adversarial diff review found no behaviour change. This also fixed a u8 overflow panic in `solaris_warp_core`.

**Docs:** ADR 0001, `docs/perf/engine.md`, `docs/handover-shots/*.png`, PROGRESS.md (status, defects, decision log) and the root AGENTS.md (workspace map; known reds are now none).

## Verification output
```
$ cargo test --workspace
... 182 passed; 0 failed (sum of all "test result" lines)

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.49s   (exit 0)

$ cargo run -p parity-dump -- tests/golden/packets
OK tests/golden/packets\20.bin
20/20 OK

$ cargo run --release -p bench-render -- --frames 1000     (offscreen, 3840x2160, 47 quads)
| Build quad batch (CPU)               | p50 0.002 | p99 0.004 ms |
| Upload + record + submit (CPU)       | p50 0.112 | p99 0.382 ms |
| Submit to GPU idle (GPU upper bound) | p50 0.084 | p99 2.374 ms |
```
Live overlay numbers (`FH6_WheelDash --bench-seconds 30`, 3840x2160 at 120 Hz, final code, quiet machine) are in `docs/perf/engine.md`:
```
Packet receive -> present returned   p50 0.452  p99 0.793 ms   (target: under one frame, 8.33 ms)  met
CPU per frame                        p50 0.411  p99 0.712 ms   (target: under 1 ms)                met
```
Smoke run on the final code (stand-in game, `mock-udp --scenario menu_pause`, a second app focused, then the stream stopped), log with `RUST_LOG=info,fh6_app=debug`:
```
04:20:02.483 DEBUG overlay hidden reason=Some(RaceOff) blank_presented=true
04:20:09.533 DEBUG overlay hidden reason=Some(NotForeground) blank_presented=true
04:20:18.921 INFO render loop stopped frames=418 packets=409
04:20:19.177 INFO windows closed
```

## Visuals
All screenshots are crops of the cluster area over the stand-in game window (`tools/stand-in-game`). No personal desktop content is in them.
- `docs/handover-shots/01-live-cluster-a.png`, `01-live-cluster-b.png`: live, updating.
- `docs/handover-shots/02-stats.png`: the `--stats` lines (p50/p99 receive-to-present, CPU p99, fps).
- `docs/handover-shots/03-race-off-hidden.png`: `IsRaceOn = 0`.
- `docs/handover-shots/04-visible-again.png`: racing again.
- `docs/handover-shots/05-focus-lost-hidden.png`: another app in front.
- `docs/handover-shots/06-hot-reload-before.png`, `07-hot-reload-after.png`: config edited while running.

How to repeat the manual checks:
```
cargo build --release -p fh6-app -p mock-udp -p stand-in-game -p screen-capture
target/release/fh6-stand-in                       # stand-in "game", takes focus
target/release/FH6_WheelDash --game-process fh6-stand-in.exe --stats
target/release/mock-udp --scenario bmw_m4_cs --duration 120     # shows, updates
target/release/mock-udp --scenario menu_pause --duration 30     # hides every other 4 s
copy target\release\fh6-stand-in.exe target\release\other-app.exe && target\release\other-app.exe   # focus lost: hides
target/release/screen-capture --region 1300,1560,1240,560 --out shot.png
```

## My calls
- **`windows` 0.58 → 0.62 for the whole workspace.** wgpu-hal 30 needs 0.62. Mixed versions broke the build through `gpu-allocator`.
- **The DirectComposition visual is created by wgpu** (`DxgiFromVisual`), not by our code. Less `unsafe`; `SurfaceTargetUnsafe::CompositionVisual` stays available if Phase 9 wants its own visual tree.
- **The swapchain stores sRGB-encoded premultiplied values (`E(c)·a`) in `Bgra8Unorm`.** DWM blends in encoded space. Phase 5 should composite in linear and resolve (ADR 0001).
- **Loopback bind by default** (`UdpListenAllInterfaces` opts in). Forza targets 127.0.0.1, and binding 0.0.0.0 triggered a Windows Firewall prompt for every new test binary.
- **XInput via `LoadLibraryExW(LOAD_LIBRARY_SEARCH_SYSTEM32)`** rather than plain `LoadLibraryW`. A DLL-planting guard.
- **Hidden means one fully transparent frame, then idle.** No `ShowWindow` from the render thread, which avoids cross-thread `SendMessage`. Visible means waking every refresh, so a stale stream hides within one frame of the 250 ms trigger.
- **Two ports, one stream.** When the same datagram arrives on 8000 and 5500 (mock-udp sends to both), the port that published last owns the stream until it has been silent for 50 ms; the other port's copies are dropped. Comparing payloads instead would drop a paused game's genuine repeats (and their forwards) and let a lagging thread reorder values.
- **The OSD live view is not a full bypass.** It lifts only race-off and stale, so the dash can animate on the pause screen but never over another app. `DemoMode` in `config.json` lifts nothing (neither visibility nor the RB camera-ring race guard); only `--demo` and `--bench-seconds` bypass.
- **GPU loss is survivable.** wgpu's default handlers panic; ours set a flag, the render loop drops the device and swapchain (the overlay shows nothing) and recreates them once a second. A failed surface configure is reported as `GpuError::DeviceFailed`, because wgpu panics if an unconfigured surface reaches `get_current_texture`.
- **The tray's "Open configurator" opens `config.json`** (which hot-reloads) until Phase 10. **"Open OSD"** and the View hold only toggle the OSD live view (`InputShared::osd_open`); the OSD itself is Phase 9.
- **All Win32 `unsafe` moved into fh6-win** (tray, single instance, process list), so fh6-app contains none.
- **Non-cockpit cameras use the cockpit anchor for now.** Camera ring index 4 uses the bonnet anchor and scale. Chase-cam routing is Phase 8.
- **The dyno feeds `--demo`, the bench and the OSD live view whenever the game is not actually driving** (stale stream, or the paused game's `IsRaceOn = 0` packets): `render_loop::uses_dyno`, data from `fh6_osd::LiveDynoGenerator`.
- **Screenshots are taken by a Rust tool (`screen-capture`) and cropped**, so no personal desktop content reaches the public repo.

## Code review
A full review of the phase diff found 2 High, 5 Medium and 10 Low issues; all were fixed in `c02286a`. An adversarial verification workflow (6 skeptic groups, 2 regression lenses) then checked those fixes; its residual findings were fixed in `83e9c65`:
- **High:** a failed surface configure during GPU recovery still reached `get_current_texture` on an unconfigured surface (a wgpu panic one frame later). Now `DeviceFailed` from `create`/`render`.
- **Medium:** the OSD live view drew the paused game's frozen packet instead of the dyno; a failed thread spawn left the UDP engine "running" with nothing bound, so the retry logged success forever; `car_profiles.json` deleted the old file before renaming; a handler that dropped its own `AppWindow` freed the state it was running from (now `Rc`, with a test).
- **Low:** a GPU loss while hidden went unnoticed until unpause; headless contexts swallowed wgpu errors (bench-render and the offscreen readback now fail on them); a skipped blank frame waited 50 ms; payload dedupe dropped a paused game's repeats and mishandled NaN; `DemoMode` still lifted the RB guard; orphaned temp files; stale `WindowCtl` HWND calls; a GDI brush leak in a class-registration race; no test for the save-side A/B/X/Y strip; stale handover text.
- Verified sound and unchanged: the XInput, hold-toggle, Guide, launcher and status-server fixes; no new per-frame allocation, lock or readback.

## Known issues / debt
- **CPU headroom is thin for Phase 5.** On a quiet machine the test cluster takes 0.411 ms p50 / 0.712 ms p99 CPU per frame (budget 1 ms), but the same run with builds compiling in the background read 1.278 ms p99.
  - The bulk is in wgpu's `Queue::write_buffer` and submit, plus the DXGI `present` call. Building the batch costs microseconds.
  - Offscreen, the same work is 0.11 ms p50, so per-frame swapchain overhead dominates.
  - Phase 5 should use a persistent mapped staging ring and a single encoder for all layers (ADR action 6), and re-measure with the real clusters.
- **The next-vsync rate in `engine.md` is indicative only.** It relies on DWM's reported vblank, which can lag on an idle desktop.
- **Nine `TODO(phase-7)` comments in fh6-clusters** mark computed-but-unused values found by the lint pass (PROGRESS defect 8).
- **The OSD is not drawn and ESC auto-pause is not wired into the app yet** (Phase 9). The configurator is Phase 10.
- **Fallback B (readback + `UpdateLayeredWindow`) is not implemented**, as decided in the ADR.
- **After a GPU recovery the dash reappears without its fade-in**, and the recreate may land on WARP if the hardware adapter is not back yet. Both predate the review and only matter after a device loss.
- **The widget status page still exists at `127.0.0.1:5301`.** It now uses no script; Phase 10 replaces it.

## What the next phases must know
- **Phase 5 (render kit):**
  - Build on `fh6_gpu::{GpuContext, OverlaySurface, OffscreenTarget}`; `fh6_gpu::wgpu` re-exports the exact wgpu version.
  - `OverlaySurface::render` takes a `QuadBatch`; replace or extend it with the kit's frame graph. Keep the contract: one present per frame, frame latency 1, transparent clear, premultiplied output.
  - The overlay format is `Bgra8Unorm`; offscreen targets use `Rgba8Unorm` with the same encoding.
  - `test_cluster` is throwaway; the real clusters replace it in `render_loop.rs` (`cluster_input` shows how anchors and scales are derived from config and the RB camera index).
- **Render-loop rules:**
  - Read telemetry with `TelemetryReader::latest()` (no lock).
  - The live config with `ConfigHandle::current()`.
  - The controller with the `InputShared` getters.
  - Never allocate, lock or read back in the loop.
- **Visibility:** `fh6_core::VisibilityController`. The OSD live view is `VisibilityInputs::live_preview`, fed from `InputShared::osd_open()`: it lifts race-off and stale only, never the focus rule, and `render_loop::uses_dyno` swaps in the dyno while the game is paused. Do not widen it into `bypass` (that is for `--demo` and the bench).
- **Phase 8 (cockpit fit):**
  - `camera_patcher.rs` is unchanged apart from `bind_car_brand`, which now takes `CarBinding` and returns `Result<BindOutcome, BindCarBrandError>`.
  - `tools/stand-in-game` gives a neutral background for fitting work, and `tools/screen-capture --region` makes crops.
- **Phase 10 (configurator):**
  - `widget_server` already binds 127.0.0.1:5301.
  - Writing `config.json` hot-reloads within 250 ms. Write it atomically (temp file + rename), as `AppConfig::save` does.
  - Telemetry port changes restart the UDP engine live.
- **Testing:**
  - `--game-process <exe>` points the focus rule at any process.
  - `--demo` bypasses visibility (the `DemoMode` config key does not).
  - `--bench-seconds N --bench-out <file>` writes the perf report.
  - `--log <file>` with `RUST_LOG=info,fh6_app=debug` logs hide reasons.
