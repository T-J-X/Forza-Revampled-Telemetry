# HANDOVER: Fix phase B · OLED care, OSD palette, render-thread guard

**Chat / branch:** thread "Phase B", committed on `main` with explicit paths (shared checkout)  **Date:** 2026-11-27  **Result:** ✅ complete, with two "no" lines in the closing review logged as follow-ups (the dimmed frame's GPU tail, and one GPU test run without the lock)

Fix phase B of the Rust structural assessment of 2026-11-27: findings 1, 4 and 20. A fix phase has no MASTER_PROMPT, so the exit criteria are the three findings, the scope the coordinator set (only `fh6-app` and `crates/fh6-core/src/oled_care.rs`) and the gates.

## Exit criteria
| Criterion | Met? | Evidence |
|---|---|---|
| Finding 1: the OLED orbit and the idle dimmer run from the render loop on the frame's `dt`, and both are tested with a fixed clock | ✅ | `fh6_core::oled_care` reads no clock; its 17 tests step it with fixed `dt`s. Real run at 3840×2160, parked: the dash moved exactly as the orbit predicts in every capture; after 30 s it dimmed to 35% of white (peak 251 → 88) while the scrim core stayed at (10, 10, 10) |
| Finding 4: the OSD keeps the car's brand colours over a paused game | ✅ | Real run: race a Ferrari SF90 (ordinal 3595), pause, open the OSD. Before: default cyan. After: Ferrari red (`docs/handover-shots/fix-b-osd-palette.png`) |
| Finding 20: a panic on the render thread never leaves a dead dash | ✅ | Real run with a test-only panic line: the panic is logged with its thread and location, the message loop ends at once, and the process exits with code 3. Before this phase the app ran on with an empty overlay and no log line |
| Only `fh6-app` and `crates/fh6-core/src/oled_care.rs` change | ✅ | `git show --stat`: those files, the rule files, this handover and two evidence images |
| `cargo check`, `test`, `clippy -D warnings`, `fmt --check`, parity and the wasm build are green | ✅ | Verification output below |

## What was built
- **`crates/fh6-core/src/oled_care.rs`** (rewritten). The engine has no clock and two verbs: `update(inputs, dt)` for a frame that shows the dash, which returns `OledCare { offset_px, dim }`, and `rest(dt)` while nothing is on screen.
  - Orbit: a 3:4 Lissajous figure, ±2 px, 180 s and 240 s sweeps, rounded to whole pixels. It keeps time while it is switched off and while the dash is off screen.
  - Dimmer: full brightness for 30 idle seconds, then down by 0.9 per second to `OledDimmerMinAlpha`. The dim depends only on the idle time, so it is the same at any frame rate.
  - Awake means: speed at or above 0.2 m/s, revs above max(idle + 150, 1100), a held button other than A/B/X/Y, a gear change, or the OSD open. A dash that was off screen comes back at full brightness.
  - Guards: A/B/X/Y are stripped inside the engine, a value that is not a number counts as idle, and the floor is clamped to 0..=1 (not a number: 0.35).
- **`crates/fh6-app/src/render_loop.rs`**
  - The loop calls the engine on every iteration with the loop's `dt`: `update` when the dash is shown, `rest` when it is hidden or the GPU device is being rebuilt.
  - `orbited` moves the layout anchor by the orbit and keeps the envelope inside the layout's safe zone. Scrim, wash and cluster move together. The `--stats` lines (also the OSD's stats row) move and dim with them (`stats_look`).
  - `dim_dash` is the dim: one `BlendMode::Multiply` fill, drawn after the cluster, over `lit_area` (the cluster's envelope, its glow reach, and the scrim's dome, whose top edge carries the wash). It scales the colour underneath and keeps its alpha, so black stays black and an opaque plate stays opaque: the scrim never lets the factory gauges through. A broken glow-reach token cannot turn the area inside out or make it infinite.
  - `OledCare::dim` is on the scale of an opacity. The fill's gain on light is its sRGB decode, so white lands at `dim` of its screen value (0.35 gives 89 of 255) and every colour loses the same share of its light.
  - A dimmed frame is drawn without bloom (`kit.set_bloom`), because the kit adds bloom after every draw and no draw can dim it.
  - `dyno_frame` puts the car last seen racing into the dyno's frame. The car's profile, its brand palette, its logo and the cluster all read the drawn packet, so they agree.
  - `PanicWatch` reports a panic when it is dropped during an unwind. It is the last local of `run`, so the report goes out first. The OSD state is declared before the loop state, so the swapchain is dropped before the config saver waits for its writer.
  - `Fade { opacity, care }` replaces a bare `f32` argument of `Loop::render`.
- **`crates/fh6-app/src/app.rs`**: the render thread's panic report posts a close, and the 1 s timer checks the flag as a backstop. At shutdown `join_render_thread` waits for a normal stop as long as it takes; once a panic is seen, the thread gets 2 s to finish unwinding and is then left behind. A panic shown only by the join (one raised during the thread's teardown) also counts. The exit code is 3.
- **`crates/fh6-app/src/logging.rs`**: a panic hook, installed once, writes every panic (thread, location, message) to the log.
- **`crates/fh6-app/tests/idle_dim_gpu.rs`** (new): the dim on a real GPU. It is a GPU job; the app's unit tests need no GPU.
- **Rule files:** `AGENTS.md` (the `fh6-core` and `fh6-app` rows), `SKILLS.md` (law 6, and the GPU job list names the new test), `docs/PROGRESS.md` (state of the code, known defects 16 to 18, two decision-log rows).

## Verification output
On main `942e818` plus this change, in a scratch clone with its own target folder. Real output, shortened to each command's last lines.
```
$ cargo fmt --all -- --check
exit 0
$ cargo check --workspace
   = note: `cargo::unused_dependencies` is set to `warn` by default
help: consider removing the dependency on `fh6-clusters`
warning: `web-preview` (manifest) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.60s
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
   = note: `cargo::unused_dependencies` is set to `warn` by default
help: consider removing the dependency on `fh6-clusters`
warning: `web-preview` (manifest) generated 1 warning
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.71s
exit 0
$ cargo test -p fh6-core oled_care
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 68 filtered out; finished in 0.02s
$ cargo test -p fh6-app --lib
test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.17s
$ cargo test --workspace
test idle_dim_turns_down_the_light_and_keeps_the_coverage ... ok
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
exit 0
(the 49 'test result' lines sum to 828 passed, 0 failed, 1 ignored)
$ cargo run -p parity-dump -- tests/golden/packets
OK tests/golden/packets\19.bin
OK tests/golden/packets\20.bin
20/20 OK
exit 0
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.08s
exit 0
```

### Real run (3840×2160, the stand-in game window, a scratch UDP sender at 60 Hz, Ferrari SF90 ordinal 3595, Bosch DDU)
On the final build (main `942e818` plus this change):

| Check | Result |
|---|---|
| Orbit, parked, captures 11, 22, 30 and 40 s after the first | moved (1, 0), (2, 0), (2, −1), (2, −1) px; the orbit predicts the same four |
| Idle dim, parked 26 s and 44 s | 26 s: full light. 34 s and 44 s: 0.1005 of the light in every lit pixel (p5 0.099, p95 0.101), peak 251 → 88 of 255, which is 35% of white on screen |
| Scrim core while dimmed | (10, 10, 10) in every capture: unchanged |
| Forced render-thread panic | the log lines below; the process exited by itself with code 3 |

```
ERROR a thread panicked thread="fh6-render" panic=panicked at crates\fh6-app\src\render_loop.rs:991:106:
INFO message loop ended; shutting down code=0
INFO render thread joined after a panic
INFO windows closed
ERROR the render thread panicked; FH6 WheelDash quit so no dead dash stays on screen
```

On the build before the closing-review fixes (which do not touch the palette or what the dim draws):

| Check | Result |
|---|---|
| OSD over a paused game | Ferrari red; before this phase, default cyan |
| Scaffold dash (ApertureNine, its gear digit in a bloom layer), parked | moved (1, −1) px as predicted; at 44 s the gear digit 255 → 88 and the speed digit 251 → 88 of 255 (35% of white); scrim core (10, 10, 10) unchanged |
| Where the dim acts (awake and dimmed captures, aligned) | only the dash's own light changes; no edge where the fill ends |

### Live bench (release, 30 s each, `mock-udp --scenario bmw_m4_cs`, 3840×2160 Mailbox, 120 Hz; base and change run in turn)
Run on main `7be8b37` with this change before the closing-review fixes, which do not touch the per-frame path apart from the stats lines (off in the bench).

| Build | CPU per frame p50 (ms) | p99 (ms) | build p50 (ms) | frames |
|---|---|---|---|---|
| base `7be8b37` | 0.761, 0.793, 0.783 (mean 0.779) | 1.602, 1.761, 1.685 | 0.007 | 2939, 3054, 3062 |
| this change | 0.785, 0.801, 0.755 (mean 0.780) | 1.735, 1.773, 1.645 | 0.007 | 3019, 2851, 3011 |

No measurable change. Both builds are over the 1 ms p99 target (known defect 10); the p99 of both is higher than the 1.23 ms of the morning runs, because other threads were using the machine. These are awake frames only: the bench's car always moves, so it never dims, and its "CPU per frame" leaves out recording the frame, where the dim is drawn (known defect 17).

### GPU cost of a dimmed frame (offscreen 3840×2160, layout scale 1, default dome, scrim 100% and wash, parked telemetry, RTX 3070 Ti Laptop GPU)
The dim is one fill of about 1.6 million pixels at 4K. With bloom off on both sides, so that only the fill differs (variants run in turn, 600 frames each):

| Cluster | no fill: GPU p50 / p99 (ms) | with the dim fill: GPU p50 / p99 (ms) |
|---|---|---|
| BoschDdu | 0.338 / 0.399 | 0.407 / 0.967 (18 of 600 frames over 0.5 ms) |
| ApertureNine (scaffold) | 0.180 / 0.182 | 0.249 / 0.252 |

Against the awake frame, which has bloom: ApertureNine 0.244 → 0.250, Gt7HoodPro 0.240 → 0.246, LiquidGlass 0.417 → 0.417, BoschDdu 0.338 → 0.407 (its bloom layer is dark when parked, so turning bloom off saves nothing there).

The dimmed Bosch DDU has a slow-frame tail: its p99 was 0.97 to 2.74 ms in five runs, and 3% to 10% of its frames went over 0.5 ms (77 of 800 in the run of the dim exactly as drawn; up to 2.9 ms). Every fill of that size gives it on that dash: Multiply or Normal blend, alpha 0 or 0.9, before or after the cluster. A fill of the cluster's envelope only (0.388 ms p50) gives none, and the Liquid Glass dash gives the same tail awake with no fill at all (p99 2.76 to 2.86 ms). In this harness the tail starts once a frame takes about 0.40 ms; I did not find its cause. It is limited to the dimmed state, while the car is parked and nothing on the dash moves, and the kit's light gain (known defect 16) removes the fill. A larger cluster scale or fit setting makes the fill larger; that was not measured.

## Visuals
- `docs/handover-shots/fix-b-idle-dim.png`: the Bosch DDU and a scaffold dash, awake and idle-dimmed (desktop captures of the real exe).
- `docs/handover-shots/fix-b-osd-palette.png`: the OSD over a paused game, before (default cyan) and after (Ferrari red).

## My calls
1. **The dim turns down light and keeps coverage.** The assessment named the frame opacity. That would fade the scrim and show the factory gauges behind an idle dash.
2. **0.35 is an opacity, as the key's name says.** The gain on light is the sRGB decode of the dim (0.35 gives 0.10 of the light), so white lands at 35% of its screen value. A plain gain of 0.35 on light looks like about 63% opacity, a much milder dim than the setting promises.
3. **A dimmed frame has no bloom.** A glowing part's halo cuts off when the dim starts, about 0.7 s before the dash reaches its floor. Left on, bloom keeps a glowing digit near full brightness (the GPU test checks that this still holds, so the switch can go when it stops holding). Only the kit can fade bloom smoothly (known defect 16).
4. **A gear change wakes the dash.** The driver's busiest buttons are A, B, X and Y, which the overlay may not read (AGENTS.md rule 4). The gear byte of the packet shows a shift without reading them.
5. **The orbit never leaves the safe zone.** A dash that sits flush against it orbits to one side only on that axis.
6. **The dyno's frames name the car last seen racing.** One value at the source instead of a second lookup beside the palette, so a cluster that reads the car (the ADAS ego car's brand tint) also gets it in the OSD live view.
7. **A drop guard, a close request and exit code 3, not `panic = "abort"`.** The workspace profile is outside this phase's files, and abort would skip the OSD's pending save. It is listed for TJX (known defect 18).
8. **Shutdown waits 2 s for a render thread that has panicked,** counted from when the panic is seen, then leaves without it. A normal quit still waits as long as it takes, as before.
9. **Every panic is logged,** on any thread and in both binaries. Only a render-thread panic ends the app.
10. **The GPU test is its own test binary** (`--test idle_dim_gpu`), so `cargo test -p fh6-app --lib` needs no GPU and no lock.
11. **The dim fill also covers the scrim's dome,** where the wash runs, so the wash is dimmed exactly once. It costs about 0.02 ms more GPU than a fill of the cluster alone; the cheaper choice would dim the wash's alpha and dim twice the strip where the two meet.
12. **The stats lines orbit and dim with the dash.** They are a diagnostic overlay, but the OSD can leave them on, and their fixed labels would burn in like any other.

## Known issues / debt
- **Bloom is switched off, not dimmed** (known defect 16). The right fix is a light gain in the kit's resolve (`Kit::set_light_gain`, beside `set_underlay`): no extra fill, bloom fades with the rest, and `dim_dash`, `lit_area` and the bloom switch go away. The kit is outside this phase, and the Liquid Glass pilot is editing it.
- **The dimmed Bosch DDU's GPU tail** ("GPU cost of a dimmed frame" above): over 0.5 ms at p99 in this harness while parked and dimmed.
- **The live bench leaves out the time to record a frame** (known defect 17). It was so before this phase. It matters for the 0.3 ms dash share of AGENTS.md rule 7; not changed here, because every dash thread compares against today's numbers.
- **No idle scenario.** Every `mock-udp` scenario keeps the car moving and the dyno never idles, so the benches never draw a dimmed frame. The real run used a scratch sender.
- **Only a render-thread panic is caught** (known defect 18). A render thread that hangs without panicking is not detected. A panic on another thread is logged, and the app carries on.
- **The overlay window cannot be hidden from the main thread** (`fh6-win` has no `hide()`). Today the dead frame goes when the dying thread releases the swapchain; the declaration order in `run` makes that early, and the process exit ends it in any case.
- **`OledDimmerMinAlpha` is not range-fixed by the config loader.** The engine clamps it, so a bad value is harmless, but the saved file can show a value that is not the one in use. The default 0.35 is in `config.rs` and in `oled_care::DEFAULT_MIN_ALPHA`.
- **In the OSD live view the orbit is clamped against the safe zone of the normal layout,** which does not apply there. The effect is an orbit to one side only.

## What the next phases must know
- **Clusters do nothing for OLED care.** The app moves the layout and dims the frame. A cluster must not read the wall clock, move itself, or dim itself when idle.
- **`OledCareEngine`:** call `update` once per iteration for a frame that shows the dash and `rest` otherwise, with the loop's `dt`. `OledCareInputs::from_frame(cfg, data, buttons, osd_open)` builds the inputs.
- **`render_loop::dim_dash(p, area, dim)`** is public for the GPU test. It needs bloom off to reach the floor.
- **`render_loop::spawn(setup, on_panic)`** takes the panic report. `RenderControl::render_failed()` is the flag. Exit code 3 means the render thread panicked.
- **Follow-ups for the coordinator:**
  1. Kit: a light gain in the resolve (replaces the fill, dims bloom, ends the dimmed frame's GPU tail).
  2. Layout: reserve the orbit's travel in the safe-zone clamp of `Layout::from_config`, so `orbited` needs no clamp of its own.
  3. `fh6-win`: `OverlayWindow::hide()`, called by the main thread when `render_failed()` turns true.
  4. Config (fix phase D): the range fix and one default for `OledDimmerMinAlpha`.
  5. Bench: count the frame recording in "CPU per frame"; add a parked scenario to `mock-udp`.
  6. A decision for TJX: `panic = "abort"` for release builds. It would cover every thread with less code; it skips destructors (an OSD edit younger than the 350 ms save delay is lost).
  7. For the kit owner: in an offscreen loop on this laptop GPU, frames from about 0.40 ms GPU up show a 3% to 10% tail near 2.8 ms (the Liquid Glass dash too). Worth a look before dashes are judged on p99.

## Closing review
A fixed list, written from the coordinator's brief, `AGENTS.md` and `SKILLS.md` (sections A and B of the dash checklist, read for this change). Each line is yes or no, with its evidence. No scores.

How it was checked: four agents each took a slice of the list and checked it against the code and the logs; a second agent then re-checked every verdict. They agreed on every line. What they found, and what changed because of it:
- A panic in the render thread's teardown, after its drop guard is gone, exited with code 0; the join's result now counts too.
- The 2 s shutdown limit only applied when the panic was seen before shutdown began; it now starts whenever the panic is seen.
- The stats lines neither moved nor dimmed; now they do.
- A broken glow-reach token could empty the dim area or make it infinite; it is now guarded.
- The verification output was summarised by hand and one line held a control character; it is now the real output.
- The change was gated on `7be8b37` while `main` had moved; it was moved onto `942e818`, the doc edits were re-applied to `main`'s text, and every gate ran again.
- Line 10 quoted 3% to 7% of frames over 0.5 ms; the run of the dim as drawn had 10%. Line 21 claimed more than the code shows; it is reworded.

**A. The brief**
1. Finding 1: the render loop drives `OledCareEngine` on every iteration with the loop's `dt` (`update` for a shown frame, `rest` otherwise), applies the orbit to the layout and the dim to the frame; `oled_care.rs` reads no clock, and its tests use a fixed clock. **Yes.** In `run`, `oled.rest(dt)` is in the device-lost and hidden branches and `oled.update(OledCareInputs::from_frame(..), dt)` is before the redraw gate: one call per iteration on every path. `orbited(layout, care.offset_px)` feeds the frame's layout; `Fade { care }` feeds `dim_dash` and the bloom switch. `oled_care.rs` has no `Instant` or `SystemTime`. 17 engine tests; real run: orbit as predicted in every capture, dim to 35% of white. [assessment finding 1]
2. Finding 4: over a paused game the OSD live view uses one car for the profile, the palette, the logo and the cluster. **Yes.** `dyno_frame` writes the car last seen racing into the dyno's frame, and both `CarConfig::resolve` calls, `ensure_palette`, the `SignalSmoother` and `FrameInput.telemetry` read `data.car_ordinal`. Test `dyno_frames_name_the_last_raced_car`; real run: Ferrari red over a paused game. [assessment finding 4]
3. Finding 20: when the render thread panics, the main thread learns of it at once, the panic is logged, nothing stays on screen and the app quits. **Yes.** `PanicWatch` (the last local of `run`) sets `render_failed` and posts a close; `slow_tick` quits on the flag; `join_render_thread` gives a panicked thread 2 s from when the panic is seen and counts a panic that only the join shows; exit code 3; `logging::log_panics`. Tests: `a_render_thread_panic_is_reported_once`, `a_panic_is_reported_before_the_loop_lets_go_of_what_it_owns`, `a_clean_render_thread_exit_is_not_a_failure`, the five `join_render_thread` tests, `a_panic_on_another_thread_reaches_the_log`. Real run: logged, exit code 3. [assessment finding 20]
4. Code changes only in `crates/fh6-app` and `crates/fh6-core/src/oled_care.rs`; the rest is the rule files, this handover and two images. **Yes.** `git show --stat` of the commit. [coordinator's scope]
5. No file the Liquid Glass pilot owns was touched. **Yes.** No path under `crates/fh6-kit`, `crates/fh6-clusters`, `tools/` or `assets/`. [coordinator's scope]

**B. Hard rules** (section A of the dash checklist, read for this change)
6. No code reads, binds or requires A, B, X or Y (`0xF000`). **Yes.** The buttons come from `InputShared::buttons()`, which `fh6-input` already strips; `in_use` strips `MASK_MANUAL_LOCK` again; test `gearbox_buttons_never_wake_the_dash`. [AGENTS 4]
7. No placeholder, `todo!()` or stub; every telemetry field the change reads is covered by the golden packets. **Yes.** It reads speed, the current and idle RPM, the gear and the car ordinal, all written by `mock-udp`. [AGENTS 1, 2]
8. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. **Yes.** They appear only in tests. [AGENTS 6]
9. The per-frame path adds no heap allocation, file I/O, logging or locking. **Yes.** The engine is arithmetic on `Copy` values; `orbited`, `lit_area`, `dim_dash`, `stats_look` and `dyno_frame` allocate nothing; the boxes and the hook are made once at start. [AGENTS 6, SKILLS 3.8]
10. The frame budget holds: no measurable CPU per frame, and under 0.5 ms GPU per frame at 3840×2160 in every state, the dimmed one included. **No.** CPU (awake frames): live bench p50 mean 0.779 ms (base) against 0.780 ms (this change); p99 over 1 ms in both (known defect 10). GPU: a dimmed frame is under 0.5 ms at p50 (Bosch DDU 0.407, scaffold 0.249), but the dimmed Bosch DDU's p99 is 0.97 to 2.74 ms in five offscreen runs, with 3% to 10% of frames over 0.5 ms (77 of 800 for the dim exactly as drawn). Every fill of the dim's size does this on that dash, a fill of the envelope only does not, and the Liquid Glass dash shows the same tail awake with no fill. Measured at layout scale 1 with the default dome only. The fix that removes the fill is the kit's light gain (known defect 16), outside this phase's files. It only happens while parked and dimmed, when nothing on the dash moves. Logged as an open follow-up for the render kit work (known defect 16). [AGENTS 7]
11. Telemetry still reaches the screen on the next frame: no smoothing, no queue, and the orbit and the dim never skip or delay a frame. **Yes.** The redraw gate and the loop's `break` and `continue` statements are unchanged; `care` is computed before the gate and adds no condition; a woken dash is at full brightness on the same frame. [AGENTS 7]
12. No new path draws while the app hides the dash. **Yes.** The hidden branch only adds `oled.rest(dt)`; the hide frame records nothing and clears to `(0,0,0,0)`. [AGENTS 9]
13. No non-Rust source file was added to the repo. **Yes.** The scratch scripts stayed in the scratch folder. [AGENTS 11]
14. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path, a personal email or a mention of the legacy build. **Yes.** The commit script stops on a trailer; a search of every changed file found none of the rest. [AGENTS 10, ONBOARDING 3]
15. Nothing was copied into the game folder or the mirror. **Yes.** [AGENTS 8]
16. No new dependency. **Yes.** No `Cargo.toml` or `Cargo.lock` change; every new `use` resolves to an existing dependency. [AGENTS 6]
17. `cargo check`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, parity 20/20 and the fh6-kit wasm build pass on the tree that lands. **Yes.** "Verification output" above, on `942e818` plus this change. [AGENTS 3]

**C. Rendering laws** (section B of the dash checklist)
18. Colour stays premultiplied linear end to end; alpha is never multiplied twice; transparent is `(0,0,0,0)`. **Yes.** The Multiply fill scales the colour underneath and keeps its alpha (blend state `Dst`, `OneMinusSrcAlpha`; alpha `Zero`, `One`); the wash is dimmed once, by the fill; the hide frame clears to `(0,0,0,0)`. GPU test: coverage unchanged on every sampled pixel. [SKILLS 3.1]
19. NaN and Inf never reach a shader. **Yes.** The engine treats a non-finite speed, rpm or idle as idle and a non-finite floor as 0.35; `is_dimmed` rejects any dim outside 0..1, NaN included; a non-finite or negative glow reach falls back to 0; the orbit is a whole number in ±2. Tests `broken_telemetry_does_not_keep_the_dash_awake`, `dim_floor_is_clamped_and_a_bad_value_falls_back`, `a_broken_glow_reach_leaves_the_cluster_itself_dimmed`. [SKILLS 3.2]
20. Off pixels stay `#000000`: the dim adds no light and no coverage, and there is no full-canvas fill. **Yes.** The fill only multiplies what is there, over `lit_area`, not the canvas; GPU test: "nothing" stays (0, 0) and the black scrim stays black. [SKILLS 3.6]
21. Nothing new in a cluster, the OLED engine or the render loop reads a clock. **Yes.** The engine takes `dt`, and the loop's `dt` is its existing tick. The only new clock reads are the shutdown limit in `app.rs` `join_render_thread` (main thread, after the loop has stopped) and its tests. [SKILLS 3.7]
22. The orbit and the dimmer run in `fh6-app`, not in a cluster. **Yes.** No cluster or kit file changed. [SKILLS 3.6]

**D. Thread rules**
23. The thread opened with an introduction and is not archived. **Yes.** The introduction is the thread's first message (13:39 UTC). [AGENTS 13, ONBOARDING 5]
24. The GPU lock was held for every GPU job and released after it. **No.** At 15:03 UTC the first (failing) run of the render-loop tests included the first version of the GPU dim test, a 960×540 offscreen render of a few seconds, without the lock. Every later GPU job (the GPU test, the timing runs, the real runs, the benches and `cargo test --workspace`) ran under the lock. It cannot be undone; it is recorded in the decision log. [AGENTS 15, SKILLS 5]
25. The rust-skills and rust-review passes, an independent review, `/code-review` and `/simplify` ran, and their findings are applied or listed. **Yes.** Applied in the code; the rest are in "Known issues" and the follow-ups. This closing review also ran as two independent passes over every line. [the person's rules, SKILLS 1]
26. The review is yes/no only, with no scores anywhere. **Yes.** [AGENTS 12]
27. The handover follows `program/HANDOVER_TEMPLATE.md`, and `docs/PROGRESS.md`, `AGENTS.md` and `SKILLS.md` match the code. **Yes.** Every template section is here; the doc edits were checked against the code by two agents and re-applied to `main`'s current text. [ONBOARDING 5]
28. The commit uses explicit paths in the shared checkout, and its branch was never switched. **Yes.** The commit holds only this change's paths; `main` was moved with a compare-and-swap of its ref, and other threads' uncommitted edits in the shared files stayed in place. [AGENTS 16]

## Sign-off
TJX asked on 2026-11-27 (16:00 UTC) for threads to review their own work with the installed review skills and agents, and not to wait for a sign-off. This closing review stands in for it. The coordinator logged the two "no" lines as follow-ups: the parked dim's GPU overrun goes with the render kit's light gain (known defect 16), and the one GPU test run without the lock is recorded in the decision log.
