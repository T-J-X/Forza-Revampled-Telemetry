# HANDOVER: Fix phase G · Gates that mean something

**Chat / branch:** thread "Rust fix G", branch `claude/project-thread-ocd2bw` (PR #5), on main `8212145`  **Date:** 2026-11-28  **Result:** ✅ complete

Fix phase G of the Rust structural assessment of 2026-11-27, "Gates that mean something": the test findings and findings 16 and 18, plus four audit extras (allocation counting, the true MSRV, the `fh6-app` unsafe line and a `preview-export --no-assets` flag). A fix phase has no MASTER_PROMPT, so the exit criteria are the assessment's row for G, those findings, the extras the coordinator listed and the gates. D, E and F ran in parallel threads; G rebased onto E (`a8ef6a9`) and then D (`8212145`) before landing.

## Exit criteria
| Criterion | Met? | Evidence |
|---|---|---|
| One sensitivity and NaN test that every rebuilt dash plugs into (backlog G2, G3 and G5) | ✅ | `crates/fh6-clusters/tests/dash_gates.rs`: 10 tests over all 48 registry entries, on wgpu's no-op backend (no GPU, no lock). A dash plugs in through its row in `docs/TELEMETRY_USAGE_MATRIX.md` and, if it has threshold reads, its `DASH_SCENARIOS` entry; the cluster prompt says so |
| Float checks in the golden telemetry test | ✅ | `crates/fh6-core/tests/golden_telemetry.rs` compares every key of every golden `.json`, floats bit for bit, over every `.bin` in the folder (at least 20), and fails on a golden with no packet; `the_comparison_is_exact` proves a 1-ulp change fails |
| `parity-dump` fails on a missing golden | ✅ | A missing `.json` or an orphaned one fails the run; `--write-missing` writes a new packet's golden with `create_new` and never an existing one; 7 unit tests; `20/20 OK` on the repo goldens |
| The empty tests are removed | ✅ | Of the assessment's 7, fix E deleted 4 with their code (`fh6-clusters/src/lib.rs:617` and `:629`, `fh6-render`'s `pool.rs` and `assets.rs`) along with the v1-only sensitivity test. The 3 in `tests/component_instruments.rs` assert pixel positions and content today (`check_pixels`, `detail`), so they stay |
| A workspace `[lints]` table (finding 16) | ✅ | `[workspace.lints]` in `Cargo.toml`, inherited by all 18 members; clippy `-D warnings` passes with it (below) |
| `forbid(unsafe_code)` in the safe crates (finding 16) | ✅ | `fh6-core`, `fh6-kit`, `fh6-clusters`, `fh6-osd`, `fh6-gpu`, `fh6-app` (`fh6-render` went in fix E); `fh6-win`'s `camera_patcher` and `car_brands` modules too. `unsafe` is allowed only in `fh6-win` and on `fh6-input`'s `xinput` module |
| NaN and Inf sanitised once in the decoder (finding 18) | ✅ | `fh6_core::telemetry::decode`: NaN and the infinities become 0, the rest clamp to `±MAX_ABS` (1e7), in-range values come through bit for bit; tested at all 31 float offsets |
| `normalized_rpm` deleted (finding 18) | ✅ | Gone; `git grep normalized_rpm` finds only history |
| The G5 robustness test (finding 18) | ✅ | `g5_hostile_telemetry_and_odd_targets_stay_finite`; the mutation check below shows it catches a dash's own NaN |
| Allocation counting (audit extra) | ✅ | `warm_frames_do_not_allocate` with `allocation-counter` 0.8.1 (dev-dependency), and `the_allocation_counter_sees_allocations` proving the counter is live |
| The true MSRV (audit extra) | ✅ | `rust-version` and `clippy.toml` say 1.90; `cargo +1.90.0 check` passes for the Windows target and for asset-forge on the host (below). 1.87 never built: `ordered-float` 5.5.1 (through `wgpu-hal` 30.0.2) needs 1.90 |
| `fh6-app/src/lib.rs:3` tells the truth about `unsafe` (audit extra) | ✅ | It now says `unsafe` lives in `fh6-win` and `fh6-input`, and the crate forbids it; `fh6-win`'s crate doc says the same |
| `preview-export --no-assets` (audit extra) | ✅ | Landed after fix F, on 2026-11-28 ("After fix F" below): dashes and the OSD built with `assets: None`, no pack fonts, an empty car database, the custom name in the logo zone, output in `mockups/no-assets/`. The no-asset path itself is gated by `dashes_build_and_draw_without_assets` |
| fmt, clippy `-D warnings`, `cargo test --workspace` with the goldens and parity pass | ✅ | "Verification output" |

## What was built
**Telemetry (`crates/fh6-core`)**
- `src/telemetry.rs`: `le_f32` cleans every float in `decode` (NaN and ±Inf to 0, clamp to `±MAX_ABS`); `MAX_ABS` is public; `CurrentEngineRpm` is also kept at 0 or above. The module and `decode` docs state the contract exactly, including that zeros pass through so a division by a packet value still needs its own guard. `encode` is the inverse of `decode`, for tools and tests. `normalized_rpm` is deleted. Tests: hostile words, fill bytes and 64 seeded noise packets decode finite and bounded, in-range floats decode bit for bit at every one of the 31 offsets, `encode` inverts `decode`.
- `tests/golden_telemetry.rs`: rewritten. An exhaustive destructure of `FH6TelemetryData` lists every field, so a new field does not compile until the golden test covers it.

**The kit (`crates/fh6-kit`)**
- `src/scene.rs`: `Scene::fingerprint` (a hash of what a frame draws: visible instances with their paint and clip by content, point lists, draw runs, segments and mesh draws, bit for bit; zero-opacity items left out, mesh handles by first use, `-0.0` as `0.0`, list lengths) and `Scene::non_finite_count` (every float list the GPU reads, including the packed f16 bloom halves). `dropped` also counts a layer merged into the last segment when the segment list is full. Three unit tests.
- `src/math.rs`: `repairs()`, a per-thread count of every value the kit had to repair: each `finite_or` that replaces a NaN or an infinity.
- `src/painter.rs`: a non-finite transform or shape the painter ignores counts as a repair.
- `src/text.rs`: `Fonts::atlas_generation`, so a test compares frames only at one glyph atlas size.

**The dash gates (`crates/fh6-clusters/tests/dash_gates.rs`, new)**
- **G2 and G3** (`g2_g3_dashes_read_exactly_the_inputs_the_matrix_declares`): for each dash, in five common scenarios (the four review states and a lift off), its own `DASH_SCENARIOS` and every view the catalog designs it for, each of 26 inputs is switched in for 20 frames and back out, and four frames are fingerprinted against a build that never saw it. An input the row lists must change the frame; an input it does not list must not. Discrete inputs get several values (gear 2, 0 and 11; drivetrain 1 and 2; steer ±90); timestamps and positions advance every frame. Two fresh builds must draw the same frame.
- **G3, the matrix itself**: every registry entry has exactly one row and every row a registry entry, the dash name matches, the Inputs and Scenarios tables match the test's, and through serde each input's probe changes exactly the fields it names while every field belongs to one input.
- **G5** (`g5_hostile_telemetry_and_odd_targets_stay_finite`): whole hostile packets (NaN, infinities, `f32::MAX`, huge values, noise) with racing off and forced on; each word of a normal racing packet in turn at `f32::MAX`, `MIN`, 0 and a subnormal; 22 integer extremes and inconsistent readings; frame times 0, 1/60, 0.1, 0.25 and 5 s; cockpit, hood and chase, km/h with kW and reduced flash; targets 0x0, 1x1, 64x36, 3840x1 and 1x2160. A frame fails on a non-finite value, a dropped item, a kit repair or a panic.
- **Allocation** (`warm_frames_do_not_allocate`): after one pass of the review sequence, a perturbed second pass makes no heap allocation in `update`, `animating`, `begin_frame` or `draw`, in each designed view.
- **No assets** (`dashes_build_and_draw_without_assets`): every dash builds and draws with no asset folder, the hostile packets included.
- Each gate runs every dash on its own threads and kits and reports every failure at once (a panic is caught and reported per dash); the whole file takes about 50 s in a debug build.
- `Cargo.toml`: dev-dependencies wgpu (`noop`), `allocation-counter` 0.8.1 and `serde_json`.

**The matrix (`docs/TELEMETRY_USAGE_MATRIX.md`)**: rebuilt from what the gate observes, replacing the hand-audited v1 snapshot. It describes the step probe, the scenarios and how `car_ordinal` reaches a dash (the palette and `Logo.brand_name`). New this phase: `race_on` for Liquid Glass and Retro Cyber (their start shows) and `car_ordinal` for Porsche Analog (its peak reset on a new car), all intended behaviour.

**Lints and MSRV**
- `Cargo.toml`: `[workspace.lints]` denies `unsafe_code`, `unsafe_op_in_unsafe_fn`, and clippy's `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `dbg_macro` and `undocumented_unsafe_blocks`; allows `chunks_exact_to_as_chunks` for now (backlog). `rust-version = "1.90"`. Every member manifest has `[lints] workspace = true`.
- `clippy.toml`: `msrv = "1.90.0"`; tests may unwrap, expect, panic and `dbg!`. Integration-test helpers that need it allow the lints at the file root with a reason (`fh6-clusters/tests/golden_images.rs`, `tests/support/mod.rs`, `fh6-kit/tests/{render,glass,fluid}_gpu.rs`).
- The crate roots listed under the exit criteria `#![forbid(unsafe_code)]`; `fh6-input` allows `unsafe` on `pub mod xinput` only, with a reason.
- `tools/mock-udp/src/main.rs`: a failed bind prints a message and exits with 1 instead of panicking. `crates/fh6-clusters/tests/component_instruments.rs`: an `assert!` replaces a `panic!`.

**`tools/parity-dump/src/main.rs`**: the missing-golden and orphan checks, `--write-missing` with `create_new`, keys through a `BTreeMap`, fix D's value comparison kept through the rebase (see "My calls" 8), and 7 tests.

**After the rebase onto fix D**: the gates pass `Family::key()` to `Palette::family` and set the typed `SpeedUnit` and `PowerUnit`; G5's unit variant shows power in kW as well as speed in km/h.

**Docs**: `AGENTS.md` (toolchain line, the `parity-dump` row, the commands, directive 6, Agent 4's gates), `SKILLS.md` (MSRV; the dash gates need no GPU lock), `README.md` (MSRV), `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` (a dash owns its matrix row and `DASH_SCENARIOS` entries; the verify step and checklist lines 4, 13 and 15 run the gates), `docs/PROGRESS.md` (G2, G3 and G5 done, known defect 21, the lint backlog, seven decision-log rows, the state of the code after G).

## Verification output
### On the PC (rustc 1.100.0), commit `fd13cf5` (the tree that landed), 2026-11-28
In an isolated clone under the checkout's `target\fix-g\` with its own target folder. The GPU tests held `gpu.lock` from 05:50:57 to 05:56:30 UTC (an earlier wait, at `1f50de9`, ran from 23:48 to 00:34 UTC while fix F and the Retro Cyber thread held the lock, and then the session paused on the usage limit). Each command's last line as the PC session reported it:
```
$ cargo fmt --all -- --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.90s     (no lints)
$ cargo test -p fh6-clusters --test dash_gates
test result: ok. 10 passed; 0 failed; 0 ignored; finished in 42.59s
$ cargo test --workspace --no-fail-fast
exit 0; 966 passed, 0 failed, 1 ignored (the fluid.rs doctest), across 39 test binaries and 9 doc-test runs
golden_images: 49 passed in 129.24 s; dash_gates 10 passed; golden_telemetry 2 passed; parity_dump 7 passed
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK, 0 orphaned goldens
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.46s
```

An earlier PC pass at `1f50de9` (before the rebase onto fix D) had the same fmt, clippy, dash gates, parity and wasm results.

### Cloud checks (Linux container, rustc 1.100.1 and 1.90.0), final tree `9e03335` on main `8212145`
```
$ cargo fmt --all -- --check --config newline_style=Unix
exit 0
$ cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc --exclude asset-forge --keep-going -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s     (no lints)
$ cargo clippy -p asset-forge --all-targets -- -D warnings
exit 0
$ cargo +1.90.0 check --workspace --all-targets --target x86_64-pc-windows-msvc --exclude asset-forge
exit 0
$ cargo +1.90.0 check -p asset-forge --all-targets
exit 0
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.49s
$ cargo check -p web-preview --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.84s
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK, 0 orphaned goldens
$ cargo test --workspace --no-fail-fast --exclude fh6-app --exclude screen-capture --exclude stand-in-game --exclude wgc-bench
passed 810 failed 88 ignored 1     (dash_gates: 10 passed in 62.82 s under the parallel run; about 50 s alone)
```
The 88 failures are the container's known reds, the same as on main: the GPU tests (no adapter on the DX12 backend), four `asset-forge` `cloud::remote` tests (Windows paths) and one `fh6-win` camera test (CRLF).

### GPU tests and image goldens on a software GPU (Mesa lavapipe, Vulkan), branch against main
The container has no GPU, so both commits were built with wgpu's Vulkan backend on lavapipe, a local two-line change that was not committed (`vulkan` added to wgpu's features, `Backends::VULKAN` in `fh6-gpu/src/context.rs`). Lavapipe does not draw exactly like the PC's DX12 card (46 of the 49 golden tests miss the PC's images on it), so the check compares the branch with main on the same renderer:
```
main 8212145:  FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images     -> 49 passed (260 images written)
branch 9e03335 with main's lavapipe images:
  cargo test -p fh6-kit -p fh6-clusters --test render_gpu --test glass_gpu --test fluid_gpu --test guide_examples
    --test component_adas --test component_classic --test component_instruments --test component_liquid
    --test component_seg7 --test component_tach --test golden_images
  -> 84 passed, 0 failed (golden_images 49/49, the kit's GPU tests 18/18, the component tests 17/17)
branch 9e03335:  FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images     -> 260 images, 0 differ from main's byte for byte
```
So every golden frame the branch renders is byte-identical to main's on the same renderer. The PC's card then ran the same tests on `fd13cf5` (above): all pass against the committed goldens.

### The gates catch what they claim
- **A dash's own NaN** (mutation check): a NaN rotation planted in the shared scaffold's `draw` failed G5 for all 42 scaffold dashes (2,016 failure lines); the file was restored and `git diff` was empty.
- **Edge-only reads**: the first version of G2/G3 compared settled frames only. With the step probe the gate found the three undeclared reads above (Liquid Glass and Retro Cyber `race_on`, Porsche Analog `car_ordinal`) and the matrix now lists them.
- **Order independence**: each gate builds its own kits, compares at one atlas generation, and passes run alone (`cargo test -p fh6-clusters --test dash_gates g5`) as well as together.

## Visuals
No images. Nothing drawn changes: the decoder returns the same values for every in-range packet (the goldens are bit for bit), the kit's repair counter and fingerprint only count and hash, and the segment-overflow change only adds to `dropped`. On a software GPU every golden frame of the branch is byte-identical to main's ("Verification output").

## My calls
1. **A hostile float becomes 0 rather than the packet being rejected.** The rust-review pass offered both. The game sends one packet per frame and the matrix treats fields one by one, so a packet with one corrupt float still carries good gear, rpm and speed; a rejected packet would freeze the whole dash on the last frame. If phase C's real captures show non-finite floats in real packets, rejecting is a one-line change in `decode`.
2. **`MAX_ABS` is 1e7.** Real readings sit far below it (rpm, metres, newton-metres), and `MAX_ABS` cubed (about 1e21) stays far inside `f32`, so a dash's products stay finite.
3. **The gates compare scene fingerprints, not pixels.** On wgpu's no-op backend they need no GPU and no lock and take about a minute for all 48 dashes, so every dash thread can run them on every change. The cost: an image whose pixels change under the same handle does not show (`Kit::register_view`), which `Scene::fingerprint`'s doc says; no registry dash does that today.
4. **Bit-exact comparison, not quantised.** A sub-pixel move counts as a change. Quantising would hide a real but small read (a smoothed value early in its ramp), and the probe's four checks already include settled frames.
5. **The lint table is the subset of the BeautifulRust tables that enforces AGENTS.md's directives 2 and 6, all `deny`.** The full table raised 261 warnings on the pre-E tree (164 hidden lifetimes, 46 missing `Debug`, 23 `allow` without a reason, 27 others), 120 of them outside the v1 code E deleted, mostly in files other phases are editing. `allow_attributes_without_reason` is the next one worth adopting (backlog); every `allow` this phase added carries a reason already.
6. **`chunks_exact_to_as_chunks` is allowed.** It fires on about 40 calls in 21 files once the MSRV is 1.90; the change is style only and would touch files D, F and the dash threads are editing. Backlog.
7. **Tests may unwrap through `clippy.toml`, and integration-test helpers allow it at the file root.** clippy's `allow-*-in-tests` covers `#[test]` functions and `#[cfg(test)]` modules, not the helper functions of an integration-test file.
8. **`parity-dump` keeps both fixes for `preserve_order`.** `fh6-win`, and since fix D `fh6-core`, turn on `serde_json`'s `preserve_order`, so in a workspace build `parity-dump` wrote its keys in insertion order. G sent them through a `BTreeMap`; D, in parallel, made the comparison read values instead of text. After the rebase `parity-dump` does both: it compares values (D's) and writes new goldens with sorted keys (G's), with G's missing-golden, orphan and `create_new` checks and D's every-shipped-golden test.
9. **The matrix is rebuilt from what the gate observes**, and the old hand-audited v1 snapshot is replaced. Each change from the old rows was read against the dash's code before it went in.
10. **`--no-assets` waited for fix F** (exit criteria), so G landed first and the dash threads got the gates sooner. It follows the app's own no-asset path in `fh6-app/src/resources.rs`: the car database is empty too, so the OSD shows no brand palettes or car names.

## Known issues / debt
- **`Meshes::upload` never reuses a mesh** (known defect 21, found by this phase's review): each rebuild of a dash that uploads meshes leaks them (Tesla ADAS uploads 5 per build; the OSD carousel and a style switch rebuild). The fingerprint numbers mesh handles by first use, so the gates are not affected.
- **`Kit::render` is not in the allocation gate**: it needs a GPU. The host CPU phase measures the whole frame on the PC.
- **G2 cannot see an input read only through a texture** the dash rewrites under the same handle (My calls 3).
- **Threshold reads need a `DASH_SCENARIOS` entry.** The common scenarios cross the usual thresholds (redline, braking, lift off), but a lamp with its own threshold needs its dash to add a scenario, as Bosch DDU's four do. The cluster prompt tells each dash thread to.
- **The empty `## Visuals` evidence**: none needed; nothing drawn changed.
- **Backlog, `docs/PROGRESS.md` "Lints"**: `as_chunks` and `allow_attributes_without_reason`.

## What the next phases must know
- **Every dash thread:** run `cargo test -p fh6-clusters --test dash_gates` (no GPU, no lock, about a minute) and update your row in `docs/TELEMETRY_USAGE_MATRIX.md` in the same commit. A failure names the dash, the input, the scenario and the view. If your dash has a threshold the common scenarios do not cross, add a `DASH_SCENARIOS` entry (Bosch DDU's are the example).
- **Decoded telemetry** is finite and within `±fh6_core::telemetry::MAX_ABS`, but zeros pass through: guard divisions. Do not add NaN guards on packet values in a dash. `fh6_core::telemetry::encode` builds a packet from a struct for tests and tools.
- **`fh6_kit::math::repairs()`** counts, per thread, every value the kit repaired; a dash that computes a NaN shows up there, not on screen. **`Scene::fingerprint`** and **`Fonts::atlas_generation`** are the tools for "did this input reach the frame" in a test.
- **Lints:** new code gets the workspace table automatically (`[lints] workspace = true` in a new crate's manifest). `unwrap`, `expect` and `panic!` are compile errors under clippy outside tests; an integration-test helper allows them at the file root with a `reason`. A new safe crate adds `#![forbid(unsafe_code)]`.
- **MSRV 1.90**: do not use an API newer than Rust 1.90 (clippy's `incompatible_msrv` catches most).
- **`normalized_rpm` is gone**; use `current_engine_rpm / engine_max_rpm` with a guard, as the kit's signals do.
- **For F, D and the OSD thread:** rebase onto G. Your new code is under the lint table; `fh6-kit/tests/render_gpu.rs` gained a file-root `allow` for `expect_used` and `unwrap_used` (F edits that file).

## Closing review
A fixed list, written from the coordinator's brief, the assessment's row for G, `AGENTS.md` and `SKILLS.md`. Each line is yes or no, with its evidence. No scores.

How it was checked: a `rust-review` pass (with `rust-skills`) and an adversarial review of the dash gates by a second independent agent, both on the first version; every finding was fixed or is answered under "Review findings". A third independent agent started on the final diff and stopped on the plan's usage limit before reporting, so this list was then checked in this thread against the final tree, with the checks quoted.

**A. The brief**
1. One sensitivity and NaN test that every rebuilt dash plugs into. **Yes.** `dash_gates.rs`, 10 tests, all 48 registry entries; a dash plugs in through its matrix row (and `DASH_SCENARIOS` for its own thresholds), and the cluster prompt says so.
2. The gates catch what they claim. **Yes.** A planted NaN failed G5 on every scaffold dash; the step probe found three undeclared reads that the first version missed ("The gates catch what they claim").
3. Float checks in the golden telemetry test. **Yes.** Every field of every golden, floats bit for bit; a 1-ulp change fails (`the_comparison_is_exact`).
4. `parity-dump` fails on a missing golden, and never rewrites one. **Yes.** `a_missing_golden_fails_unless_asked_to_write_it`, `a_changed_packet_mismatches_and_is_never_rewritten`, `a_golden_without_its_packet_fails`; 20/20 on the repo goldens.
5. The empty tests are gone. **Yes.** Fix E deleted 4 with their code; the other 3 assert pixels and stay (exit criteria).
6. Finding 16: one lint table and `forbid(unsafe_code)` in the safe crates. **Yes.** `[workspace.lints]` in `Cargo.toml`, `[lints] workspace = true` in all 18 members; six crate roots and two `fh6-win` modules forbid `unsafe`.
7. Finding 18: NaN and Inf cleaned once in the decoder, `normalized_rpm` gone, G5 in place. **Yes.** `decode` tests at all 31 offsets; `git grep normalized_rpm` finds only history; G5 passes.
8. The audit extras. **Yes.** Allocation counting, MSRV 1.90 (checked with rustc 1.90.0) and the `fh6-app` doc line landed with G; `--no-assets` followed fix F ("After fix F").
9. G kept to its own files and rebased onto E and D before landing. **Yes.** The shared files it edits (the kit's scene, painter, maths and text; the matrix; the cluster prompt; rule files) are listed under "What was built"; D's `parity-dump` change was merged, not overwritten.

**B. Hard rules**
10. No code reads, binds or requires A, B, X or Y. **Yes.** No input code changed beyond moving the `unsafe` allow onto `fh6-input`'s `xinput` module. [AGENTS 4]
11. No placeholder, `todo!()` or stub added. **Yes.** The lint table now denies `todo` and `unimplemented`; clippy passes. [AGENTS 1, 2]
12. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. **Yes.** Denied by the lint table; clippy `-D warnings` passes for the Windows target and asset-forge. [AGENTS 6]
13. The per-frame path adds no heap allocation, file I/O, logging or locking. **Yes.** The repair count is a thread-local `Cell` behind a cold branch; `decode` adds a compare and a clamp per float on the UDP thread; `warm_frames_do_not_allocate` passes for every dash. [AGENTS 7]
14. The frame budget holds. **Yes.** No draw code changed beyond the cold repair branch and one early return for a non-finite shape (which was culled before). [AGENTS 7]
15. Telemetry still reaches the screen on the next frame. **Yes.** The UDP path and the render loop are unchanged; in-range packets decode bit for bit (goldens). [AGENTS 3]
16. Nothing new draws while the app hides the dash. **Yes.** [AGENTS 9]
17. No non-Rust source file was added. **Yes.** Rust, TOML and Markdown only. [AGENTS 11]
18. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path or a personal email. **Yes.** `git log --format='%an <%ae> | %(trailers:only)'` over the branch; a search of the diff found none.
19. Nothing was copied into the game folder or the mirror. **Yes.** [AGENTS 8]
20. A new dependency has a decision-log line. **Yes.** `allocation-counter` 0.8.1, wgpu `noop` and `serde_json` as dev-dependencies of `fh6-clusters`. [AGENTS 6]
21. fmt, clippy `-D warnings`, the workspace tests with the goldens and parity pass on the tree that lands. **Yes.** "Verification output": the PC run covered everything but the GPU tests; the GPU tests and image goldens ran on a software GPU, byte-identical to main's frames.

**C. Rendering laws**
22. Colour stays premultiplied linear end to end. **Yes.** No shader or blend state changed.
23. Off pixels stay `#000000`, with no full-canvas fill. **Yes.** Nothing drawn changed; the image goldens match main's.
24. Nothing new reads a clock. **Yes.** The gates' timestamps are synthetic.

**D. Thread rules**
25. The thread opened with an introduction and is not archived. **Yes.** [AGENTS 13]
26. The GPU lock was held for every local GPU job. **Yes.** The dash gates need none (no-op backend); the PC run waited for the lock before its GPU step and stopped there; the cloud GPU tests ran on a software renderer in the container, outside the PC lane. [SKILLS 5]
27. Review skills and independent agents ran, and their findings are applied or answered. **Yes.** "Review findings".
28. The review is yes/no only, with no scores anywhere. **Yes.** [AGENTS 12]
29. The handover follows `program/HANDOVER_TEMPLATE.md`, and `docs/PROGRESS.md`, `AGENTS.md` and `SKILLS.md` match the code. **Yes.**
30. The shared checkout's branch was never switched; the PC run used an isolated clone. **Yes.**

## Review findings
Three reviews. The first two read the first version (`e28c585` and `5c008a0` before the rebase); the third stopped on the usage limit before it reported (see "Closing review").

`rust-review` with `rust-skills` (diff and crate roots):
1. Must fix: production `unwrap()` at `fh6-clusters/src/lib.rs:545` and `:575` fails the new lint. Gone: that was the v1 half of `lib.rs`, deleted by fix E before G landed.
2. Must fix: the matrix parser broke on a CRLF checkout. Fixed: the matrix text is read with CRLF turned into LF before `table()` parses it.
3. Must fix: no decision-log line for the new dev-dependencies and the MSRV, and the cited `as_chunks` backlog line did not exist. Fixed: seven decision-log rows and the "Lints" backlog.
4. Should fix: `fh6-render` skipped the lint table. Gone: fix E deleted the crate.
5. Should fix: `AGENTS.md` and `SKILLS.md` still said MSRV 1.87, and 1.90 was inferred. Fixed, and checked with rustc 1.90.0.
6. Should fix: the decoder's doc promised that nothing downstream needs a guard. Fixed: it says zeros and subnormals pass, so divisions need a guard. Rejecting a non-finite packet instead was considered (My calls 1).
7. Should fix: G5 repeated one word in every float, ran in the cockpit view only, and tried no odd frame time. Fixed: the per-word sweep of a normal packet, cockpit, hood and chase, frame times 0, 0.1, 0.25 and 5 s. A NaN frame time is not tried: the host measures it with `Instant`, which cannot give one.
8. Should fix: G3 could not see reads of a rate of change. Fixed: timestamps and positions advance each frame, and the lift-off scenario moves rpm (`rpm_rate`).
9. Should fix: the fingerprint misses texture contents and had no list lengths. Lengths added; the texture case is a doc caveat (My calls 3).
10. Consider: `non_finite_count` skipped the f16 bloom halves. Fixed, with the test.
11. Consider: the `unsafe` allow was crate-wide in `fh6-input`. Fixed: on `pub mod xinput` only; two `fh6-win` modules forbid it.
12. Consider: `--write-missing` checked and then wrote. Fixed: `create_new`.
13. Consider: unit tests narrower than their names. Fixed: all 31 float offsets, 14 kinds of change in `fingerprint_follows_every_drawn_value`, a mesh in `non_finite_count_sees_every_value_list`.
14. Consider: one panicking dash stopped the whole gate, and the shared kit could be poisoned. Fixed: `catch_unwind` per dash and a kit per gate. `Kit::render` stays outside the allocation gate ("Known issues").
15. Consider: doc drift and an `assert!` followed by an unreachable `return` in `component_instruments.rs`. The drift is fixed. The `assert!` stays: it carries the message, and a file-root `allow(clippy::panic)` would let any other panic in that file through.

Adversarial review of the dash gates:
1. G5 never ran hostile floats with racing on. Fixed: every whole packet runs with `is_race_on` as decoded and forced to 1.
2. G3 was not sound: a constant probe from frame 0 misses edges, rates and thresholds; three rows were wrong. Fixed: the step probe, advancing timestamps and positions, several values for discrete inputs, every designed view, the lift-off scenario. The three rows now list `race_on` (Liquid Glass, Retro Cyber) and `car_ordinal` (Porsche Analog).
3. The field-coverage test did not check what it said. Fixed: through serde, each probe changes exactly its fields and every field has one input.
4. CRLF. Fixed in the parser. No `.gitattributes` was added: it would change how every thread's checkout writes Markdown.
5. The fingerprint hashed handles and kit-global state. Fixed: mesh handles by first use, comparisons at one atlas generation, a kit per gate. Images still hash by handle, and the image cache keys on a file's modification time, so an asset rewritten during a run could show as a change; the determinism check would name it.
6. The painter's sanitising hid a dash's own NaNs, and a segment overflow was not counted. Fixed: `fh6_kit::math::repairs` and `dropped`; G5 fails on either.
7. Whether G5 saw folder-font text depended on test order. Fixed: every gate draws with `draw_complete`, which loads each face first.
8. Bit-exact comparison counts invisible changes. Zero-opacity items are skipped and `-0.0` is `0.0`; positions are not quantised (My calls 4).
9. Other G5 gaps. Fixed: integer extremes and inconsistent readings, 8 noise packets, every view, km/h with kW and reduced flash, frame time 0.1, the no-asset fallback with hostile packets, `catch_unwind`, thin and tall targets. Not covered: brand palettes from `Palette::from_config` and a user display font (each dash's own goldens cover its palettes).
10. Allocation gate gaps. Fixed: a perturbed second pass, `animating()` and `begin_frame` measured, every designed view. Not covered: `Kit::render` (needs a GPU) and branches the review sequence never reaches.
11. Matrix parsing edge cases. Fixed: alignment separators, a header per table, code fences, the dash name and the Scenarios table checked. Escaped pipes are not handled; the matrix has none.
12. Doc errors (five against four scenarios, `car_ordinal` through the palette only). Fixed.

## After fix F: `preview-export --no-assets` (2026-11-28)
Commit `823e3eb` on main `7970472`. `--no-assets` builds every dash and the OSD with `assets: None` (in the main loop, `cluster.rs`, `fit.rs` including the scrims, and `osd.rs`), loads no pack fonts, gives the OSD an empty `CarDatabase` and shows only the custom name in the logo zone, which is what `fh6-app` does when `assets/` is missing. The reference backdrops still load. Its output goes to `mockups/no-assets/` unless `--out` says otherwise; it works with `--cluster` and `--fit`.
```
$ cargo fmt --all -- --check --config newline_style=Unix                      exit 0
$ cargo clippy -p preview-export --all-targets --target x86_64-pc-windows-msvc -- -D warnings      (no lints)
$ cargo test -p preview-export                                                 9 passed (one new: the flag parses with --cluster and --fit)
On the cloud's software GPU (lavapipe; the local Vulkan change of "Verification output", not committed):
$ preview-export --release --no-assets                    Export complete: 192 kit renders (with the OSD states and the scrims), 11 min 41 s
$ preview-export --release --no-assets --cluster PorscheAnalog                 Cluster review written, 35 s
$ preview-export --release --no-assets --fit                                   Fit review written, 12 s
```
Each run printed `assets: none (--no-assets): no pack fonts, no images, no logos` and `logo: no car, the custom name "TJX"`. Side by side with the normal review set, Porsche Analog falls back to plain bezels, the system face and "TJX" where the Porsche wordmark sits, and every reading still draws.

## Sign-off
TJX asked on 2026-11-27 (16:00 UTC) for threads to review their own work with the installed review skills and agents, and not to wait for a sign-off. This closing review stands in for it.
