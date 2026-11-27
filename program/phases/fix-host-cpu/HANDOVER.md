# HANDOVER: Fix phase · host CPU budget (known defects 10 and 17)

**Chat / branch:** `claude/project-thread-muqx44`  **Date:** 2026-11-27  **Result:** ✅ complete

## Exit criteria
| Criterion (from the coordinator's brief) | Met? | Evidence |
|---|---|---|
| The app meets under 1 ms CPU per frame (known defect 10, `AGENTS.md` directive 7) | ✅ | Live bench p99 0.873 (Bosch DDU), 0.883 (Liquid Glass), 0.783 ms (Aperture Nine); p50 0.62–0.68 ms. Baseline on main (b709602): p50 0.75–0.81, p99 1.45–1.64 ms, with less of the frame counted. `docs/perf/engine.md` |
| "CPU per frame" counts recording the frame (known defect 17, Phase B follow-up 4) | ✅ | The series is the render thread's whole iteration less the frame-latency wait, with rows for each part (`crates/fh6-app/src/render_loop.rs`, `perf_report.rs`) |
| A parked `mock-udp` scenario for the idle dim (Phase B follow-up 4) | ✅ | `mock-udp --scenario parked`; the report adds an idle-dimmed row when a frame was dimmed and prints the parked command when none was |
| No visual change | ✅ | `golden_images` 49/49 after the bloom margin fix; the kit's staging test compares 16 frames pixel for pixel with `write_buffer` |
| GPU stays under 0.5 ms per cluster | ✅ | `preview-export --cluster BoschDdu` sequence p50 0.239 / p99 0.249 ms (0.217 / 0.228 before) |

## What was built
- **Bench (defect 17).** `fh6-gpu::FrameTimings` splits `encode_submit` into `record` (swapchain view, encoder, recording, `finish`) and `submit`. `fh6-app::latency::FrameStats` gains `before_build`, `draw`, `record`, `kit`, `submit`, `after_present`, `cpu_frame_dimmed`, and passes and draws per frame; `SAMPLE_CAPACITY` is 16384 so a 60 s parked bench keeps every frame. `perf_report` prints them all, the kit's passes and draws per frame, and a reproduce block whose stream outlasts the bench.
- **`mock-udp --scenario parked`:** racing, stopped, 800 rpm idle with a slight wobble, no pedals.
- **Frame thread class (`fh6-win::system::raise_frame_thread`).** MMCSS "Games" (fallback `THREAD_PRIORITY_ABOVE_NORMAL`), the power-throttling opt-out, and on a hybrid CPU the thread's CPU sets limited to the highest efficiency class. A `!Send` guard undoes all three on drop. Feature `Win32_System_SystemInformation` added to the existing `windows` dependency. The render loop logs the class it got.
- **Release profile:** thin LTO, one codegen unit (`Cargo.toml`).
- **Kit upload staging ring (`fh6-kit/src/renderer.rs`, `Kit::frame_submitted`).** Three `MAP_WRITE | COPY_SRC` buffers; a frame records a copy from a ready slot into the upload buffer, writes the slot through its mapping and unmaps it; after submit the app calls `frame_submitted` and the kit maps the written slots again. With no ready slot it falls back to `Queue::write_buffer`. `bench-render` and the app call it.
- **Compute bloom (`fh6-kit/src/shaders/bloom.wgsl`).** The 5 down and 4 up levels are one compute pass of 9 dispatches, tap for tap the old fragment shaders (`fs_up` removed from `post.wgsl`). The pyramid textures gain `STORAGE_BINDING`. Each dispatch zeroes a 2-texel margin past its viewport, as the old passes' clear did.
- **Tests:** `a_staged_copy_covers_every_post_slot_and_stays_inside_the_upload` (unit), `recycled_upload_staging_draws_every_frame_exactly_like_write_buffer` (GPU, including fallback frames), `the_frame_thread_is_raised_and_reverted_on_its_own_thread`, `performance_cores_are_a_strict_subset_of_the_machine`, and the report test for the new rows and the parked variant.

## Verification output
On the test PC (RTX 3070 Ti Laptop, DX12), commit 8a999d4:
```
cargo test -p fh6-clusters --test golden_images   49 passed; 0 failed
cargo test -p fh6-kit                              121 + 2 + 8 + 1 + 6 + 1 passed, 1 ignored
cargo test --workspace --no-fail-fast              exit 0, every test binary ok
cargo clippy --workspace --all-targets -- -D warnings   clean (G, ca79536; 8a999d4 changes only the kit's bloom, clippy-clean)
cargo fmt --all -- --check                         clean
parity-dump tests/golden/packets                   20/20 OK
cargo build -p fh6-kit --target wasm32-unknown-unknown   ok
render thread scheduling class class=Mmcss full_speed=true performance_cores=12
```

Live bench, 25 s, `mock-udp --scenario bmw_m4_cs` at 60 Hz, stand-in focused, 1920 x 1080 at 300 Hz, quiet machine (load 19–24% average). ms, p50 / p99:

| Dash | Passes / draws | CPU per frame | view + encoder + kit + `finish` | `submit` | `present` | Receive -> present |
|---|---|---|---|---|---|---|
| Bosch DDU | 2.9 / 13.5 | 0.656 / 0.873 | 0.248 / 0.359 | 0.140 / 0.203 | 0.218 / 0.333 | 0.662 / 1.366 |
| Liquid Glass | 4.0 / 14.9 | 0.684 / 0.883 | 0.283 / 0.407 | 0.139 / 0.199 | 0.216 / 0.308 | 0.688 / 1.537 |
| Aperture Nine | 3.0 / 11.0 | 0.623 / 0.783 | 0.235 / 0.318 | 0.132 / 0.184 | 0.211 / 0.300 | 0.626 / 1.288 |

What each step was worth (interleaved runs on the PC): MMCSS, the throttling opt-out and thin LTO took the tail down first; the staging ring cut the kit's render from 0.13 to 0.03 ms p50; holding the thread on the performance cores cut the p99 by 0.2–0.3 ms; the compute bloom cut about 0.045 ms p50 (10–12 render passes per frame to 3–4).

GPU (`preview-export --cluster BoschDdu`, both from live worktrees): sequence p50 / p99 0.239 / 0.249 ms after, 0.217 / 0.228 ms before. Idle and cruise are unchanged (0.151–0.153 ms); the bloom-heavy states cost about 0.02 ms more.

## Visuals
None new: no pixel changed (`golden_images` 49/49). The bench reports are in `docs/perf/engine.md` and `docs/perf/render-kit.md`.

## My calls
1. **The bench's "CPU per frame" is the whole iteration less the latency wait,** not a sum of parts, so nothing between the parts can hide. The parts are listed under it, and the remainder (sample bookkeeping) is a few microseconds.
2. **MMCSS "Games" for the render thread.** Its work is short and latency-bound. MMCSS runs it in a real-time band, so it can take a performance core from the game for a frame's length (well under 1 ms); the doc of `raise_frame_thread` says so.
3. **Performance cores only when the CPU is hybrid.** On a CPU with one efficiency class the thread is left on every core.
4. **A staging ring rather than one persistently mapped buffer.** wgpu does not allow a buffer to stay mapped across a submit; three slots let the map callbacks of the previous frames land. `map_async` boxes its callback inside wgpu: one small allocation per frame that the kit's own code does not make (decision log).
5. **The fallback stays.** An app that never calls `frame_submitted` (the web preview, a test) draws exactly as before through `write_buffer`.
6. **Compute bloom despite 0.02 ms more GPU.** The CPU target was the miss; the GPU is at half its budget.
7. **A 2-texel zeroed margin rather than clearing whole textures.** A bilinear tap clamped to `src_max` reaches at most one texel past a level's viewport; the second covers the pyramid's rounding.
8. **No bloom-off preset and no change to the dashes' look.** Not needed to meet 1 ms.

## Known issues / debt
- The rest of the frame is wgpu and the driver (about 0.25 ms record and `finish`, 0.14 ms `submit`, 0.22 ms `present` at p50). Fewer glass layers per dash is the next lever if a dash comes close to its 0.3 ms share.
- The bench PC's display is 1920 x 1080 at 300 Hz, not the LG C1 at 4K 120 Hz. Host CPU barely depends on resolution, but the receive-to-vblank numbers are for 300 Hz.
- A parked bench (60 s with `--scenario parked`) was not run in this phase's final batch; the dimmed row is unit-tested and the scenario is wired.
- Known defect 16 (the idle dim cannot dim bloom) is unchanged.

## What the next phases must know
- Any host that renders with the kit should call `Kit::frame_submitted()` right after the queue submit that carried the render; without it the kit falls back to `write_buffer` (slower, same pixels).
- `fh6_win::system::raise_frame_thread()` returns a guard that must live on the thread it raised for as long as the thread renders.
- A dash's CPU share is now measured against an app total that fits: about 0.65 ms p50 and under 0.9 ms p99 with the dashes benched here.
- New bloom code must keep the zeroed margin; the hood-view goldens catch a missing one.

## Closing review
Against `AGENTS.md` and `SKILLS.md`, yes or no with evidence.

1. The app's CPU per frame is under 1 ms at p99 on the live bench. **Yes.** 0.78–0.88 ms on three dashes. [AGENTS 7]
2. The bench counts the frame's recording (defect 17). **Yes.** Whole iteration less the latency wait. [Phase B follow-up 4]
3. GPU stays under 0.5 ms per cluster. **Yes.** 0.249 ms p99 for the Bosch DDU sequence. [AGENTS 7]
4. No pixel changed. **Yes.** 49/49 goldens; the staging GPU test is pixel-equal. [SKILLS 9]
5. No per-frame heap allocation in the kit's or the app's render code. **Yes.** The bloom steps are a stack array; the staging ring reuses its buffers. wgpu's own `map_async` box is logged in the decision log. [AGENTS 6]
6. Every `unsafe` block has a `// SAFETY:` comment and stays in `fh6-win`. **Yes.** `system.rs` only. [AGENTS 6]
7. No `unwrap()`, `expect()` or `panic!` in new non-test code. **Yes.** Map failure marks a slot lost and logs a warning. [AGENTS 6]
8. No new dependency. **Yes.** One `windows` feature on the existing dependency; `Cargo.lock` unchanged. [AGENTS 6]
9. Render code does not call Win32. **Yes.** The thread class is in `fh6-win`; `fh6-app` calls its safe API. [AGENTS map]
10. No non-Rust source file added, beyond a WGSL shader in the kit's existing shader folder. **Yes.** [AGENTS 11]
11. Telemetry reaches the screen on the next frame with no new queue. **Yes.** Receive to present p50 0.63–0.69 ms.
12. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path or a personal email. **Yes.** [AGENTS 14]
13. Nothing was copied into the game folder. **Yes.** [AGENTS 8]
14. The GPU lock was held for every GPU job and released after each batch. **Yes**, after the coordinator's note: one earlier batch kept it idle until told; every batch since released it at its end. [SKILLS 5]
15. `cargo test --workspace`, clippy, fmt, parity and the wasm build pass. **Yes.** See Verification output.
16. An independent review ran on the bench, the thread class and the staging ring, and a second on the compute bloom, and their findings are applied or answered. **Yes.** Fixes in a10df39 and below.
17. The review is yes/no only, with no scores. **Yes.** [AGENTS 12]
18. `docs/PROGRESS.md`, `AGENTS.md`, `docs/kit/README.md` and the perf docs match the code. **Yes.**
