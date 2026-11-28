# HANDOVER: Fix phase F · Load the brand logos

**Chat / branch:** thread "Rust fix F", branch `claude/project-thread-q6crd5` (PR #3), on main `3661d52`  **Date:** 2026-11-28  **Result:** ✅ complete

Fix phase F of the Rust structural assessment of 2026-11-27, "Load the brand logos": finding 6. Every `Logo` was built with `image: None`, so the logo zone showed a text wordmark everywhere, although Phase 6 ships a logo for every brand. The fix the assessment asked for: "load the logo for the car's brand when the car changes, inside `ensure_palette` and never per frame, and pass its `ImageId` in `Logo.image`. Do the same in `preview-export`." A fix phase has no MASTER_PROMPT, so the exit criteria are that fix, the plan's design question for F ("which logo variant to load and when, within 8 GB of video memory"), the scope the coordinator set (F's own files: `render_loop.rs`, `fh6-kit`'s `logo.rs` and `images.rs`, and `preview-export`) and the gates. D, E and G ran in parallel threads; F merged E (`a8ef6a9`), D (`8212145`), G (`fd13cf5`) and main's docs and cloud-lane commits (`4a6838f`, `7648f02`) and the branch rule 17 commits (`a965f43` to `3661d52`) before landing.

## Exit criteria
| Criterion | Met? | Evidence |
|---|---|---|
| The live dash passes the car's brand logo in `Logo.image` | ✅ | `fh6_kit::logo::CarLogo`, driven by `render_loop.rs` (`ensure_palette`, `poll_logos`); the PC's live runs logged `brand logo ready brand="BMW M"`, and `fix-f-live-stepped.png` shows the BMW M logo where the tach said "BMW M" |
| It loads when the car changes, inside `ensure_palette`, never per frame | ✅ | On a steady frame `ensure_palette` returns at its cache check, before the logo code, and `poll_logos` returns after one comparison (`CarLogo::busy`) unless a dial is decoding or uploading |
| `preview-export` does the same | ✅ | `--car ORDINAL` (default 3667, the 2021 Porsche 911 GT3; 0 for none) in every mode; its "logo:" line names the dial it loaded |
| The variant and the timing fit in 8 GB of video memory | ✅ | The dial variant, uploaded from 512 px down: at most 1.33 MiB a brand, and at most 47.4 MiB for all 91 dial files (46.0 MiB for the 90 brand dials; the generic emblem never loads) (`every_dial_decodes_into_the_capped_chain_and_fits_video_memory`) |
| A brand's first car no longer stalls the render thread | ✅ | The decode runs on the `fh6-logos` worker (9.9 and 13.2 ms for BMW M on the PC). The render thread then uploads one step a frame: BMW M's 545 KiB went in over 14 frames, 1.42 to 1.45 ms in all, and no step took more than 0.14 ms (third PC run). The whole load took 9.75 ms in one frame at first, and the upload alone 0.58 to 0.90 ms in one frame once the worker decoded |
| A missing, broken or oversized dial never panics, and the zone shows the brand's name | ✅ | `a_dial_that_fails_shows_the_name_and_is_never_read_again`, `a_broken_dial_is_an_image_error_and_its_car_shows_the_name`, `an_oversized_dial_is_refused_from_its_header`, `a_missing_dial_file_is_an_io_error`, `a_dial_the_image_store_refuses_counts_as_failed`, `a_stopped_worker_fails_what_it_was_asked_and_everything_after`, `a_dial_that_does_not_arrive_shows_the_name_meanwhile_and_the_dial_if_it_comes` |
| The zone never shows the name first and the dial a moment later, and the greeting never says "DRIVER 1" while the car's names are known | ✅ | `Logo::loading` (the names are set, the zone draws nothing): `a_car_shows_nothing_while_its_dial_loads_then_the_dial`, the live GPU test's empty-zone render, `the_logo_zone_stays_empty_while_the_cars_dial_loads`, `a_parked_car_greets_its_brand_while_the_dial_loads_never_driver_1` |
| Nothing else that is drawn changes | ✅ | The 49 goldens pass unchanged on the PC (fourth run, at `daabe89` and `138b030`); on a software GPU the branch's 260 golden frames are byte-identical to main's; `--car 0` renders are identical by SHA-256 to the base's (20 of 20 PNGs, first PC run) |
| fmt, clippy `-D warnings`, `cargo test --workspace` with the goldens, the dash gates, parity, the MSRV check and the wasm build pass | ✅ | "Verification output" |

## What was built
- **`crates/fh6-kit/src/logo.rs`**
  - `LogoIndex` reads `assets/logos/logos.json` once. `car_brand(cars, ordinal)` gives the brand a car shows (its `BrandKey` when the index lists it, else its make through the index's aliases; the generic emblem `DEFAULT` is never a brand) and the name the wordmark falls back to (the brand palette's name, else the index's). Dial paths that leave the logo folder are dropped.
  - `decode_dial` reads a brand's dial, refuses a file over 1024 px from its PNG header (`LogoError::TooLarge`), decodes and premultiplies it, and keeps the mip levels from 512 px down (`DIAL_TEXTURE_SIDE`). It needs no device. `load_dial` decodes and uploads in one call, for tools.
  - `DialLoader`: a worker thread named `fh6-logos` decodes on request, and the loader keeps one slot per brand for the current kit (asked, failed or loaded). `dial(brand)` answers `Ready(ImageId)`, `Missing` or `Loading` without blocking. `poll` is one frame's share of the upload: it takes the next decoded dial and copies it into the kit a slice at a time until `STEP_BUDGET_MS` (0.05 ms) has gone, one dial at a time, and a decode nobody asked for on the current kit is dropped. A dial being uploaded owns its half-made texture, and the step that finishes a dial ends the poll (that frame also logs the dial and frees its texels). `busy` says whether a dial is decoding or uploading; `new_kit` forgets the slots and the dial being uploaded, and is called both when the device is lost (the half-made texture holds the removed Direct3D 12 device, which must be gone before a new one can be made) and when the new kit is made.
  - `CarLogo`: what the logo zone shows for the car on screen, over a `DialLoader`. `show(brand, custom_name)` on a car change (`None` when the zone is off), `poll` while `busy`, `logo()` for the frame. While the car's dial loads, `logo()` already has the car's names and is marked `Logo::loading`, so the zone draws nothing and a greeting can still name the brand; the zone shows the dial when it is in, shows the brand's name when the car has no dial or it failed, and shows the name after `NAME_AFTER_FRAMES` (600) polls if the dial has still not arrived (the dial replaces it if it comes). A car never gets another car's dial.
  - `draw` fits a dial into its zone by the size of its file (`Images::source_size`), so a dial with an odd side keeps its shape.
- **`crates/fh6-kit/src/cluster.rs`**: `Logo` gains `loading`. `logo::draw` draws nothing while it is set, and a dash with its own logo-zone code must do the same.
- **`crates/fh6-kit/src/images.rs`**: `MipChain` (premultiplied levels built on the CPU, with the size of the picture they were made from), `decode_png_mips`, `png_size` (reads the header only), `Images::upload_mips` (a whole chain), `Images::upload_step` (the texture on the first call, then one slice of at most `UPLOAD_STEP_BYTES`, 128 KiB, per call; `upload_mips` is a loop over it) and `Images::source_size` (for layout; `size` stays the texel size, which the shader picks the mip level from). `upload_premultiplied_srgb8` goes through the same chain, with the same texels as before.
- **`crates/fh6-clusters/src/common/label.rs`**: with an image, `logo_zone` tints it white at alpha 215 over a dark copy, as the components spec's dial variant says. It set only the wordmark's paint, so an image would have drawn full white twice. No dash passes an image through it today: PorscheAnalog, its only caller, draws the host's image itself (`draw_logo`, as it did before the phase) and calls the helper for the name, so the helper's image tint is covered by its unit test alone. Wordmarks draw as before.
- **`crates/fh6-clusters/src/clusters/porsche_analog`**: the greeting's name (`tubes::greeting_name`) and what the logo zone shows (`zone_art`) are pure functions with tests, and the zone draws nothing while `Logo::loading` is set: without that, a Porsche would have drawn the dash's own Porsche wordmark first and the car's dial a moment later. Before `Logo::loading`, the parked greeting said "HELLO, DRIVER 1" until a brand's dial was in, because the empty `Logo` had no brand name (fix D had emptied the default driver name).
- **`crates/fh6-app`**: `KitResources::logos` loads the index with the other resources; a missing assets folder, an unreadable or empty index and an unreadable car list each log one warning that says what the zone then shows. The render loop holds a `CarLogo`: `ensure_palette` calls `show` on a car change, `poll_logos` calls `poll` only while it is busy, and `lose_gpu` and `adopt` call `new_kit`. A generic car now shows the custom name; its wordmark said "STANDARD", the default palette's name.
- **`tools/preview-export`**: `--car ORDINAL`, default 3667 (the 2021 Porsche 911 GT3), `0` for none. Every mode (the states, `--cluster`, `--fit` and the OSD) draws that car's logo, loaded once per run with `load_dial`, reads the car list once, and prints a "logo:" line that says exactly what the zone shows (the dial, or why not and which name).
- **Tests, 50 new**: 35 in `logo.rs` (a broken dial of allowed size, the lookup order on the shipped car list and on a small one, the generic emblem, path safety, a broken index, every car resolves, the names, every dial fits the capped chain and the memory bound, an oversized dial by either side and from the header alone, a missing file, the `DialLoader`'s cache, stepped upload, frame budget (with a fake clock), one dial at a time, failure and new-kit rules, and what `CarLogo` shows, including a setting changed while a read hangs), 5 in `images.rs` (the full and the capped chain, the header, the source size, the slices), 1 in `label.rs`, 2 in PorscheAnalog (the greeting and the logo zone while a dial loads), 4 in `preview-export` (the arguments, the car choice and the "logo:" lines), and 3 GPU tests in `crates/fh6-kit/tests/render_gpu.rs` (decode, upload and draw; the 512 px chain draws within 1 of 255 of the full one for six dials in six zone sizes; the live path over the worker for three cars, polls submitted between frames, the largest dial over more than one poll, and nothing in the zone while it loads).
- **Docs**: `AGENTS.md` (the `fh6-kit`, `fh6-app` and `preview-export` rows), `SKILLS.md` (law 4: a brand's dial is the one asset that loads after start-up, decoded off the render thread and uploaded in steps over the frames that follow), `docs/kit/README.md` (section 9: where `FrameInput::logo` comes from; `source_size`), `docs/PROGRESS.md` (state of the code, known defect 22, backlog and decision log).

## Verification output
### On the PC, fourth run (rustc 1.100.0), commits `daabe89` and `138b030`: the tree that lands, less the docs
In the same worktree, with its own target folder; every GPU job held `gpu.lock`, released at the end. `daabe89` has `Logo::loading`, the device-loss fix and main's commits up to `7648f02`; `138b030` adds a Porsche's wordmark while its dial loads, `adopt`'s palette reset, the broken-dial test and the live screenshot. After it, only main's docs commits and this handover were merged.
```
$ cargo test --workspace --no-fail-fast                       (daabe89)
exit 0; 47 'test result' lines: 1016 passed, 0 failed, 1 ignored (the fluid.rs doctest)
golden_images: 49 passed; dash_gates: 10 passed
a_cars_brand_logo_decodes_uploads_and_draws: ok
a_dial_from_512_px_down_draws_the_same_pixels_as_the_full_chain: ok
a_cars_logo_loads_on_the_worker_and_uploads_over_frames: ok
the_logo_zone_stays_empty_while_the_cars_dial_loads: ok   (named the_logo_zone_shows_no_name_while_the_cars_dial_loads from d1b2876)
a_parked_car_greets_its_brand_while_the_dial_loads_never_driver_1: ok
$ cargo fmt --all -- --check ; cargo clippy --workspace --all-targets -- -D warnings
exit 0 ; exit 0
$ cargo run --release -p preview-export -- --cluster PorscheAnalog ; ... --cluster LiquidGlass
logo: car 3667 (2021 Porsche 911 GT3), logos/porsche_dial.png
      (172 of 172 PNGs identical by SHA-256 to the third run's, Liquid Glass's 92 included)
$ cargo test --workspace --no-fail-fast                       (138b030)
exit 0; 1017 passed, 0 failed, 1 ignored; golden_images: 49 passed; dash_gates: 10 passed
$ cargo clippy --workspace --all-targets -- -D warnings       (138b030)
exit 0
$ cargo run --release -p preview-export -- --cluster PorscheAnalog   (138b030)
logo: car 3667 (2021 Porsche 911 GT3), logos/porsche_dial.png
      (80 of 80 PNGs identical by SHA-256 to daabe89's)
```
The fourth run's benches above were taken while other threads compiled, so the PC then ran an A/B bench: `3d9f02c` (A, the third run's commit) and `138b030` (B) alternating, both built in release before any bench and nothing built during them, each with its own PorscheAnalog bench config, mock-udp `bmw_m4_cs` at 60 Hz, 25 s. All six exited 0, loaded the BMW M dial and logged no warning or error.

| Run | Commit | CPU per frame p50 / p99 / worst | fps | rustc or cargo running |
|---|---|---|---|---|
| A1 | `3d9f02c` | 1.257 / 1.686 / 16.064 ms | 126.5 | 12 |
| B1 | `138b030` | 0.786 / 1.357 / 1.622 ms | 137.2 | 10 |
| A2 | `3d9f02c` | 0.712 / 0.975 / 1.287 ms | 139.4 | 3 |
| B2 | `138b030` | 0.746 / 1.482 / 7.774 ms | 136.7 | 3 |
| A3 | `3d9f02c` | 0.766 / 1.324 / 1.611 ms | 136.0 | 5 |
| B3 | `138b030` | 0.703 / 0.901 / 1.709 ms | 138.9 | 2 |

A1 and B1 ran under a heavy compile. In the four quieter runs the p50 is 0.712 and 0.766 ms for A and 0.746 and 0.703 ms for B, and the p99 and worst frame jump about in both commits. So `138b030` costs no more than `3d9f02c` under the same load; both sit above the third run's 0.634 ms p50 because the PC is busier now, and the third run's clean numbers stand as the budget evidence.

### On the PC, third run (rustc 1.100.0), commit `3d9f02c`: the stepped upload, merged with fix D and fix G
In the worktree under `%USERPROFILE%\Documents\fh6-worktrees\fix-f`, with its own target folder. The GPU jobs held `gpu.lock` (another thread had it first, so the build ran while waiting), and it was released at the end.
```
$ cargo test --workspace --no-fail-fast
exit 0; 47 'test result' lines: 1003 passed, 0 failed, 1 ignored (the fluid.rs doctest)
golden_images: 49 passed; dash_gates: 10 passed
a_cars_brand_logo_decodes_uploads_and_draws: ok
a_dial_from_512_px_down_draws_the_same_pixels_as_the_full_chain: ok
a_cars_logo_loads_on_the_worker_and_uploads_over_frames: ok
$ cargo fmt --all -- --check
exit 0
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0
$ cargo run --release -p preview-export -- --cluster PorscheAnalog
logo: car 3667 (2021 Porsche 911 GT3), logos/porsche_dial.png
      (80 of 80 PNGs identical by SHA-256 to the second run's)
$ cargo run --release -p preview-export -- --cluster LiquidGlass
logo: car 3667 (2021 Porsche 911 GT3), logos/porsche_dial.png
      (56 of 92 PNGs differ in bytes from the second run's, the same 56 files that differed between the first two runs;
       01_idle: mean dE 0.00, max dE 0.4, no pixel over dE 3; hood_03_redline_strobe: max dE 0.1)
```
PorscheAnalog draws the Porsche dial in its logo zone, so its byte-identical renders show that a dial uploaded in slices and laid out by its file's size draws exactly what the second run drew (the Porsche dial is 1024 x 128, so its 512 px level is exactly half the file).

The live bench (PorscheAnalog over `mock-udp` with the `bmw_m4_cs` scenario at 60 Hz, the stand-in game window in front, `--bench-seconds 25`), run twice with no `rustc` or `cargo` process on the PC. Both logs hold these logo lines and no warning or error (the times are rounded to three places here):
```
INFO brand logo index loaded brands=91
INFO brand logo ready brand="BMW M" decode_ms=9.900 upload_ms=1.454 worst_step_ms=0.139 frames=14 kib=545
INFO brand logo ready brand="BMW M" decode_ms=13.239 upload_ms=1.422 worst_step_ms=0.120 frames=14 kib=545     (the second run)
```
| CPU per frame | p50 | p99 | worst | Frames a second | Receive to present, p99 |
|---|---|---|---|---|---|
| First run | 0.634 ms | 0.828 ms | 1.255 ms | 135.9 | 1.469 ms |
| Second run | 0.649 ms | 0.877 ms | 1.692 ms | 137.8 | 1.477 ms |

What the upload line says. BMW M's dial is 14 steps (the texture, four slices for its 512 x 205 level and one for each of the nine levels below), and it took 14 frames: a step cost about 0.10 ms on this card (1.42 to 1.45 ms over 14 steps, and 0.12 to 0.14 ms for the slowest), which is more than the 0.05 ms a poll may spend before it stops, so every frame took exactly one step. The mean is close to the slowest although six of the 13 slices are under 2 KiB, so a step's cost is mostly the fixed cost of one `Queue::write_texture` call, not the bytes it copies. A frame that uploads is therefore 0.10 to 0.14 ms longer than its neighbours, for 14 frames here and for 11 to 19 across the 91 dials (their step counts); before the stepped upload, one frame was 0.58 to 0.90 ms longer.

What the table says. Both runs meet 1 ms at p99, with p50 and p99 where the host CPU phase left them (0.656 and 0.873 ms, `docs/perf/engine.md`): steady frames cost what they did. The bench clears its statistics 2 s after the render loop starts, so when the stream is already running, as in the bench's own recipe, the dial's frames fall in that warm-up. If they fell inside the measured span instead, 14 frames are 0.4% of each run's about 3,400 and could not move its p99. The worst frames (1.255 and 1.692 ms) are of the size the bench showed before this phase's upload existed (1.208 ms after the host CPU phase), and a step adds at most 0.14 ms to a frame.

### Cloud checks (Linux container, rustc 1.100.0 and 1.90.0; Windows code cross-checked), `d1b2876`
```
$ cargo fmt --all -- --check --config newline_style=Unix
exit 0
$ cargo clippy --workspace --all-targets --exclude asset-forge --target x86_64-pc-windows-msvc -- -D warnings
exit 0     (no lints, with fix G's workspace lint table)
$ cargo clippy -p asset-forge --all-targets -- -D warnings
exit 0
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
exit 0
$ cargo check -p web-preview --target wasm32-unknown-unknown
exit 0
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK, 0 orphaned goldens
$ cargo +1.90.0 check --workspace --all-targets --target x86_64-pc-windows-msvc --exclude asset-forge
exit 0
$ cargo +1.90.0 check -p asset-forge --all-targets
exit 0
$ cargo test --workspace --no-fail-fast --exclude fh6-app --exclude screen-capture --exclude stand-in-game --exclude wgc-bench
exit 101; 38 'test result' lines: 857 passed, 91 failed, 1 ignored; dash_gates: 10 passed
```
The 91 failures are the container's known reds: 86 GPU tests (every GPU test binary; there is no DX12 adapter here, and three of them are this phase's), four `asset-forge` `cloud::remote` tests and one `fh6-win` camera test. Fix G's cloud run had 88, before this phase's three GPU tests. All 46 `logo::` and `images::` unit tests pass.

### GPU tests and image goldens on a software GPU (Mesa lavapipe, Vulkan), branch against main
The container has no DX12 adapter, so the GPU tests ran on lavapipe in a scratch worktree with a local two-line change (wgpu's `vulkan` feature and `Backends::VULKAN`) that is not committed. Lavapipe does not draw the PC card's pixels, so the comparison is the branch against main on the same renderer: main's goldens were rendered there (`FH6_UPDATE_GOLDEN=1`), then the branch's tests ran against them. The same run at `3d9f02c` against `fd13cf5` gave the same result; the commits after `f71ca14` change no drawing code that the goldens reach (PorscheAnalog's zone for a maker car and the render loop's `adopt`).
```
main 4a6838f:  golden_images, FH6_UPDATE_GOLDEN=1      49 passed (260 images)
branch f71ca14 (Logo::loading) against main's images:
  fh6-kit       render_gpu 10, glass_gpu 8, fluid_gpu 2, guide_examples 1                  all passed
  fh6-clusters  component_adas 2, classic 2, instruments 4, liquid 3, seg7 2, tach 4       all passed
  fh6-clusters  golden_images                                                              49 passed
branch f71ca14, FH6_UPDATE_GOLDEN=1, byte comparison:  260 images, 0 differ from main's
```
All 87 tests of the 11 GPU test binaries pass, and every golden frame the branch draws is byte-identical to main's: the phase changes nothing a dash draws without a car. A dash's own images still upload in whole levels (`upload_chain`); only a dial goes through `upload_step`, and the kit's GPU tests check it two ways: a dial uploaded in slices from 512 px down matches the same file's full chain uploaded in whole levels (`load_png`) within 1 of 255 per channel, for six dials in six zone sizes, and a dial uploaded over frames draws byte-identical pixels to one uploaded in one call, for three dials.

### The tests catch what they claim
- With `logo::draw` laying the dial out by the texture's size again (the second review's finding 4), `a_dial_from_512_px_down_draws_the_same_pixels_as_the_full_chain` fails: "HENNESSEY in a 240x48 zone: a channel differs by 10". Across the test's six dials and six zones the worst channel difference without the fix is 43 (ABARTH, 955 x 1024, in a 500 px zone) and 33 for BMW M in the app-sized zone; with it, 0 or 1 everywhere. `docs/handover-shots/fix-f-odd-dial.png` shows the BMW M pair.
- After the third review's fixes, each of these changes to the code fails a unit test: the poll not ending at the step that finishes a dial, `show(None)` keeping the brand it waited for, the slowest step recorded as the last one, the budget counted per step instead of per frame, two dials uploading at once, a slice of zero rows, and `new_kit` keeping the upload. With the budget switched off (`f64::INFINITY`), the live GPU test fails on the software GPU: "BMW: the whole dial went up in one poll".
- With `CarLogo::show` leaving `loading` off while a dial loads (so the name would show first), five `CarLogo` tests fail; with `zone_art` drawing the name while loading, `the_logo_zone_shows_no_name_while_the_cars_dial_loads` fails.

### Earlier PC runs, on earlier commits
- **First run, `b8a10ec` (the load on the render thread):** `cargo test --workspace` 956 passed, 0 failed; the 49 goldens passed; `--car 0` renders identical by SHA-256 to the base's (20 of 20 PNGs). The live run logged `brand logo ready brand="BMW M" ms=9.751`: the first BMW's dial cost 9.75 ms on the render thread, on the frame the car changed. That is what moved the decode to a worker.
- **Second run, `b29922e` (the worker, merged with fix E):** 933 passed, 0 failed (fix E removed the v1 tests); the 49 goldens passed; PorscheAnalog's 80 renders were byte-identical to the first run's, so the 512 px chain draws what the full chain drew. The live bench logged `decode_ms=18.32 upload_ms=0.895 kib=545` and, on a rerun, `decode_ms=18.52 upload_ms=0.578`: the whole upload still landed in one frame, on top of a frame of about 0.70 ms. That is what the second review's finding 1 was about, and what the stepped upload removes.

## Visuals
All in `docs/handover-shots/`, the renders as cockpit envelopes at 1:1 (`02_mid_cruise`):
- `fix-f-porsche-before.png`, `fix-f-porsche-after.png`: PorscheAnalog before (the custom name "TJX") and after (the 2021 911 GT3's Porsche dial on the tach face).
- `fix-f-porsche-ferrari.png`: the same with `--car 3595`, the 2020 Ferrari SF90 Stradale.
- `fix-f-liquid-glass-before.png`, `fix-f-liquid-glass-after.png`: Liquid Glass, the same pair.
- `fix-f-live-before.png`, `fix-f-live-after.png`: the live app on the PC (PorscheAnalog over the BMW M4 scenario), before (the "BMW M" text on the tach) and after (the BMW M logo), at `b8a10ec`.
- `fix-f-live-worker.png`: the live app at `b29922e`, with the dial decoded on the worker.
- `fix-f-live-stepped.png`: the live app at `3d9f02c` during the third run's first bench (PorscheAnalog over the BMW M4 scenario, 115 MPH in 4th), with the BMW M dial uploaded over 14 frames and laid out by its file's size.
- `fix-f-live-final.png`: the live app at `daabe89` during the fourth run (PorscheAnalog over the BMW M4 scenario), with the BMW M emblem on the tach.
- `fix-f-emblems.png`: PorscheAnalog's logo zone at 2x for a Porsche (a wordmark), a Toyota and a BMW (emblems), on a software GPU.
- `fix-f-odd-dial.png`: the BMW M dial (1024 x 411) in the app-sized zone at 3x, full chain against the 512 px chain, before and after the layout fix, with the difference map (a software GPU render).

## Visual review
The phase changes what one zone draws: the logo zone of the two dashes that draw it (Liquid Glass and PorscheAnalog) now holds the brand's dial instead of a wordmark. The lines of the `fh6-visual-review` checklist that apply:

- **V11 (static parts).** NOT ASSESSED over the four states: the state renders are on the PC, and `static` was not run. The logo is a static part; its ink is as bright as the wordmark it replaces (`probe` of the zone: the brightest pixel is relative luminance 0.763 before and after in PorscheAnalog, 0.190 in Liquid Glass), but it covers more of the zone (p95 0.036 before, 0.408 after in PorscheAnalog; 0.010 and 0.190 in Liquid Glass). PorscheAnalog's white at alpha 215 is the components spec's dial variant; PorscheAnalog sets that ink in its own `draw_logo`, and a dash that wants it dimmer passes a dimmer `LogoStyle::tint` to `logo::draw`.
- **V20 (near-black).** Yes for the logo zone: `stretch` of Liquid Glass's zone shows faint pixels only as the thin edge of the letters, with no rectangle around the dial's transparent area, no haze and no speckle (88.4% off, 3.2% faint, 8.4% lit).
- **V25 (before and after).** Yes. `diff` before against after changes only the logo zone: PorscheAnalog 137 x 48 px at 4K (1860, 1856), 0.16% of the envelope over dE 3; Liquid Glass 221 x 27 px at (2275, 1921), 0.44%. Both are within the golden tolerance, and nothing outside the zone moved.
- **V26 (goldens).** Yes. No golden changed: the golden harness draws without a car, all 49 pass on the PC, and on a software GPU every golden frame is byte-identical to main's.
- **V1 to V10, V12 to V19, V21 to V24.** NOT ASSESSED: they judge a dash's design (the glance read, the hierarchy, the real display, the materials and the motion), which this phase does not change, and the cold reviewer's packet masks the logo zone by design.

## My calls
1. **The dial variant.** Phase 6 made each brand's dial variant for small sizes (mono, 1024 px at most); the full logos are 4096 px and in colour. The components spec draws the dial tinted.
2. **Load once per brand, on the car change.** The trigger is `ensure_palette`'s change check, as the finding asked; a brand seen again in the same kit costs a slot lookup.
3. **Decode on a worker thread.** On the render thread the first BMW M dial took 9.75 ms on the PC, and the first independent review measured 0.9 to about 10 ms across the 91 dials (median 1.7 ms) on its machine: past the 1 ms budget on that frame. The worker reads, decodes, premultiplies and builds the mips.
4. **Upload from 512 px down.** The widest logo zone (Liquid Glass's 240 x 48 less its padding) draws a wide dial at most 320 px across at user scale 1.4 on a 4K target, and a square one at most 51 px, so the 1024 px level was never sampled. The kit picks the level from the drawn size, and the levels kept are the same texels as in the full chain. A brand costs at most 1.33 MiB of video memory instead of 5.33; all 91 together 47.4 MiB instead of 189.7.
5. **Upload a step a frame, on the render thread.** With the worker, the whole upload still landed in one frame (0.58 to 0.90 ms on the PC, the second review's finding 1). `Images::upload_step` now creates the texture on its first call and writes one slice of at most 128 KiB on each later one, and a poll takes steps until 0.05 ms has gone (`STEP_BUDGET_MS`), always at least one. On the PC a step costs about 0.10 ms, and 0.14 ms at worst, so a frame takes exactly one: BMW M's 14 steps took 14 frames and 1.42 to 1.45 ms in all, and no frame carried more than 0.14 ms of it (third PC run). The 128 KiB slice and the 0.05 ms were chosen before that run and were not tuned after it: the budget only matters on a machine where a step is cheaper than it. Two other fixes were weighed and left. One staging buffer for the ten levels still copies up to 1.33 MiB in one frame. Uploading from the worker would put `Queue::write_texture` on a second thread: wgpu-core takes the queue's pending-writes lock for the whole call, the copy included, and the render thread's `submit` takes the same lock, so the stall would move into `submit` instead of going away.
6. **A dial is laid out by its file's size.** 45 of the 91 dials have one odd side, so their 512 px level is half a texel short of half the file, and a zone fitted to the texture drew them at a slightly different shape (the second review's finding 4). `MipChain` carries the picture's size, `Images::source_size` reports it for layout, and `Images::size` stays the texel size, because the shader picks the mip level from it.
7. **The zone is empty while the dial loads, and shows the name if the dial does not come.** On the PC the decode took 10 to 18 ms and the upload 14 frames, about 0.1 s at the bench's 136 frames a second; the 91 dials are 11 to 19 steps each. That is about the length of the dash's 120 ms fade-in: the car normally changes in the game's menus, while the dash is hidden, and the loader is asked on the first frame drawn with the new car. Showing the name first would flash text that the logo then replaces. The zone gets the car's names at once, marked `Logo::loading`, rather than an empty `Logo` (call 14). After 600 polls without the dial (`NAME_AFTER_FRAMES`: 10 s at 60 frames a second, about 4.4 s at the PC's 136, 2 s at 300) the name shows, and the dial replaces it if it still arrives (the second review's finding 5). A brand without a dial, or whose dial fails, shows its name at once.
8. **What the zone shows is decided in the kit** (`CarLogo`), not in the render loop: the rules that keep a stale or another car's dial off the screen are tested on any platform without a GPU (the second review's finding 2).
9. **A dial over 1024 px is refused from its header**, before decoding: one of the 4096 px full logos named by mistake in `logos.json` would otherwise cost an 85 MiB texture.
10. **No per-dash "shows a logo" flag** (the first review's finding 2). With the worker and the cap, a dash that draws no logo pays one decode off the render thread and up to 1.33 MiB per brand. The OSD's carousel draws other dashes with the same frame input, so a flag on the active dash would leave their thumbnails without the logo, and a flag that defaults to off would be a trap for every new dash that draws one.
11. **`preview-export --car` sets only the logo** (the first review's finding 3). The review states keep the family palettes and car 0's telemetry that every dash is reviewed in; pairing the car's palette as the app does (`ColorMode` "AutoBrand") would change every preview's colours and is a change to `preview-export`'s design, outside F's scope. It is in the backlog under P5.
12. **A generic car shows the custom name**, not the default palette's theme name ("STANDARD").
13. **The device loss drops the dial being uploaded.** `lose_gpu` calls `CarLogo::new_kit` before it drops the device, because a half-made dial texture holds the device (wgpu-core's `Texture` keeps an `Arc` of it), and Direct3D 12 makes one device per adapter: while a removed device lives, `D3D12CreateDevice` fails and wgpu leaves that adapter out, so the app could not make its new device on the same card. The texture lives inside the upload it belongs to, so nothing else can keep it (the third review's finding 1).
14. **`Logo::loading`, not an empty `Logo`, while a dial loads** (the pre-landing review). An empty `Logo` hid the brand's name from everything that reads it, so PorscheAnalog's parked greeting said "HELLO, DRIVER 1" until the dial was in and then the brand. With `loading`, the names are there from the first frame and only the logo zone waits; `logo::draw` honours it, and PorscheAnalog's own zone code does too.
15. **No new dependency.** The worker is `std::thread` with `std::sync::mpsc`; the timing uses the kit's wasm-safe `clock::Stamp`.

## Known issues / debt
- **`preview-export --car` changes only the logo** (call 11): the palette and the telemetry still name the family and car 0, so a preview pairs, for example, the Porsche dial with Liquid Glass's family colours. Backlog P5.
- **The live dash's logo appears about 0.1 s after the first frame drawn with a brand's first car** (calls 5 and 7): a decode of 10 to 18 ms, then 11 to 19 frames of upload at one step a frame, each about 0.10 ms longer than a steady frame (0.14 ms at worst on the PC). A step's cost is mostly the fixed cost of one `Queue::write_texture` call ("Verification output"), so smaller slices would not make those frames much cheaper, and larger ones would make them fewer. A lever that was not measured: copy the levels into one staging buffer, mapped at creation, a slice a frame (a plain memory copy), then record every `copy_buffer_to_texture` in one frame. It is in the backlog under "Brand logos".
- **`preview-export --no-assets`** is fix G's audit extra; it waited for this phase to land because both edit `preview-export`'s `main.rs`.
- **Liquid Glass's renders were not byte-identical between the PC runs** (56 of 92 PNGs between the first two, max dE 0.4, no pixel over dE 3), with or without a logo. PorscheAnalog's renders, whose PNGs go through the same upload path, are byte-identical. The likeliest cause is that Liquid Glass's GPU render is not bit-exact from run to run on the PC's card; this was not confirmed by rendering twice at one commit. The third run repeated it: the same 56 files differ from the second run's (01_idle again by at most dE 0.4, with no pixel over dE 3), while PorscheAnalog's 80 are again byte-identical. Between the third and fourth runs all 92 were identical, so the difference comes and goes between runs. The goldens' tolerance (dE 3 on at most 0.5% of pixels) covers it, and on a software GPU every golden frame is byte-identical to main's.
- **Emblem dials are small on PorscheAnalog.** Its logo zone is a 160 x 20 wordmark slot, so a square or near-square dial draws about 15 px tall at 4K: the BMW roundel reads as a badge, but lettering inside an emblem (Toyota's) does not, where these cars showed their brand's name across the zone before the phase (`fix-f-emblems.png`). A taller zone for emblems, or the name for dials whose `logos.json` kind is an emblem, is the dash's call. Backlog "Brand logos".
- **What no test covers.** `MipUpload::fits` (it restarts an upload whose chain changed shape) is a guard no caller can reach now that the texture lives in its upload. The render loop's wiring (`ensure_palette`'s call to `show`, `poll_logos`, `lose_gpu` and `adopt` calling `new_kit`) runs in no test, because the loop needs a window and a DX12 device; the PC's live runs exercise `show` and `poll`, and the device-loss path was read, not run (the pre-landing review's finding 4, left: moving the refresh decision out of the loop to test it would change `render_loop.rs` beyond this fix for three lines of wiring).
- **The Formula Drift dial has transparent padding**, so it draws smaller in the zone than other dials. An asset fix (crop the dial in its Phase 6 recipe), not code.
- **After a device loss, the OSD carousel's thumbnail dashes keep image ids from the old device** (found by both reviews, pre-existing, not reproduced; known defect 22): `crates/fh6-osd/src/host.rs` keeps the thumbnails while the carousel's order is unchanged, and `fh6-app`'s `lose_gpu` and `adopt` cannot reach them. PorscheAnalog's own Porsche wordmark is one of those ids. The live dash and its logo are not affected (`adopt` drops the dash and calls `CarLogo::new_kit`). The fix the review proposed is an `OsdHost::new_kit()` that clears the thumbnails, called from `adopt`; `fh6-osd` is outside this phase's files and the OSD thread is working in it.

## What the next phases must know
- **A dash gets the logo in `FrameInput::logo` and only draws it** (`fh6_kit::logo::draw`, or `common::label::logo_zone` with a `LogoVariant`). While a brand's dial loads, `Logo::loading` is set and the names are already filled: the logo zone draws nothing (`logo::draw` does so by itself; a dash with its own logo-zone code must check `loading`, as PorscheAnalog's `zone_art` does), and other text such as a greeting may use the names. With `ShowLogo` off the whole `Logo` is empty. Build a `Logo` with `..Logo::default()`. For a car without a brand logo, or whose dial failed, `image` is `None` and the wordmark falls back to the brand name, then the custom name.
- **Lay an image out by `Images::source_size`**, not `size`, wherever the image can be a capped chain (`logo::draw` does). `size` is the texel size of the top level.
- **A host owns one `CarLogo`**: `show` on a car change, `poll` while `busy`, `logo()` for the frame, `new_kit` when the device is lost and again when the new kit is made. Nothing in it blocks. A host that holds any other wgpu object of a lost device must drop it too, for the Direct3D 12 reason in call 13.
- **Tools** load a dial with `LogoIndex::load_dial`, or any chain with `Images::upload_mips` (all at once: call it once per run). `preview-export --car ORDINAL` picks the car (default 3667; 0 renders as before the phase).
- **The video memory a session can spend on logos** is at most 46.0 MiB (the 90 brand dials, each once per kit; the generic emblem never loads), 1.33 MiB per brand.
- **GPU tests can run in the cloud on Mesa lavapipe** (`apt-get install mesa-vulkan-drivers`, wgpu's `vulkan` feature and `Backends::VULKAN` as a local change that is never committed). It does not draw the PC card's pixels, so compare a branch with main on it, as fix G's handover and this one do.

## Closing review
A fixed list, written from the coordinator's brief, `AGENTS.md` and `SKILLS.md`. Each line is yes or no, with its evidence. No scores.

How it was checked: a `rust-review` pass and a `rust-skills` pass on the diff, three independent agents, each given the code and the scope and not this thread's conclusions (one on the first version, one on the worker commit, and one on the fixes to the second one's findings), and a pre-landing review of five readers (the merges with fix D in the crates and in the tools, the driver's view of every sequence, and the claims in the code, the commits and this handover), each finding of which a second agent then checked. Each ran once. Their findings and what was done are under "Review findings".

**A. The brief**
1. The logo for the car's brand loads when the car changes, inside `ensure_palette`, never per frame, and its `ImageId` is passed in `Logo.image`. **Yes.** `render_loop.rs` `ensure_palette` calls `CarLogo::show` after its cache check; `poll_logos` calls `CarLogo::poll` while a dial is decoding or uploading, and the frame input takes `CarLogo::logo`.
2. `preview-export` does the same. **Yes.** `car_logo` in `tools/preview-export/src/main.rs`, in every mode; the "logo:" lines in "Verification output".
3. The variant and the timing were chosen within 8 GB of video memory. **Yes.** My calls 1, 2 and 4; at most 46.0 MiB for the 90 brand dials.
4. F kept to its files and told D, E and G where they overlapped. **Yes.** Beside the listed files, these changed: the kit's GPU tests (`crates/fh6-kit/tests/render_gpu.rs`, three tests); `fh6-kit`'s `clock.rs` (a comment); `fh6-app`'s `resources.rs`, which loads the logo index with the other start-up resources; `common/label.rs`, only for the case of an image, which no caller passes (PorscheAnalog, its only caller, draws the host's image itself and calls the helper for the name, which draws as before); `fh6-kit`'s `cluster.rs`, for `Logo::loading`; PorscheAnalog, whose greeting and own logo-zone code had to honour `loading` (pure helpers with tests; its goldens are byte-identical); and the unregistered kit sample's logo tint (one line). The coordinator is told about `Logo::loading`, because every dash thread builds `Logo`s. E, D and G were merged in as they landed, with the car-database calls ported to D's typed API, and G is told when F lands, so that its `--no-assets` flag can follow.

**B. Hard rules**
5. No code reads, binds or requires A, B, X or Y. **Yes.** No input code changed. [AGENTS 4]
6. No placeholder, `todo!()` or stub added. **Yes.** [AGENTS 2]
7. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. **Yes.** Fix G's workspace lint table denies them, and Windows clippy passes with `-D warnings`; a failed spawn, decode, upload or channel logs and falls back to the name. [AGENTS 6]
8. The per-frame path adds no heap allocation, file I/O, logging or locking. **Yes.** A steady frame runs one comparison in `poll_logos` and returns at `ensure_palette`'s cache check. The channel is polled and slices are written only while a dial is decoding or uploading; the allocation, the file I/O and the one log line per brand happen on a car change, on the worker, on the frame a dial's last slice goes in, inside wgpu on each of a dial's 11 to 19 upload steps (its staging copy), and on the one frame a name shows after 600 polls. [AGENTS 6, 7; SKILLS 3.4, 3.8]
9. The phase does not make the frame budget worse. **Yes.** A steady frame adds one comparison, and the third PC run's benches met 1 ms at p99 (0.828 and 0.877 ms; p50 0.634 and 0.649 ms), where the host CPU phase left them. An A/B bench on the PC under the same load found `138b030` no slower than `3d9f02c` (quiet-run p50 0.746 and 0.703 ms against 0.712 and 0.766 ms). While a brand's dial uploads, a frame carries one step: about 0.10 ms, and 0.14 ms at worst on the PC, for 11 to 19 frames, once per brand per kit. The first version spent 9.75 ms in one frame. [AGENTS 7]
10. Telemetry still reaches the screen on the next frame. **Yes.** No wait was added; the loader never blocks. [AGENTS 7]
11. No new path draws while the app hides the dash. **Yes.** [AGENTS 9]
12. A missing asset never panics, and assets load once. **Yes.** Each brand's dial loads once per kit; failures show the name (tests listed under the exit criteria). [SKILLS 3.4]
13. No non-Rust source file was added. **Yes.** [AGENTS 11]
14. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path or a personal email. **Yes.** `git log --format='%an <%ae> | %(trailers:only)'`; a search of the diff found none. [AGENTS 10]
15. Nothing was copied into the game folder or the mirror. **Yes.** [AGENTS 8]
16. No dependency was added. **Yes.** The phase's diff does not touch `Cargo.toml` or `Cargo.lock`. [AGENTS 6]
17. fmt, check, clippy `-D warnings`, `cargo test --workspace` with the goldens and the dash gates, parity 20/20, the MSRV check and the wasm builds pass on the tree that lands. **Yes.** The PC's fourth run (1017 passed, 0 failed; the 49 goldens and the dash gates) and the cloud checks at `d1b2876`, in "Verification output". After them only docs, a screenshot and main's docs commits were added.

**C. Rendering laws**
18. Colour stays premultiplied linear end to end. **Yes.** The dial is premultiplied once on the CPU and uploaded as sRGB, as before (`upload_premultiplied_srgb8` goes through the same chain); no shader or blend state changed. [SKILLS 3.1]
19. Off pixels stay `#000000`, with no full-canvas fill. **Yes.** V20 above; the dial's transparent texels are zero.
20. Nothing new reads a clock in a cluster. **Yes.** The worker times its decode and the loader its upload steps with `clock::Stamp`, for the step budget and the log; clusters are unchanged. [SKILLS 3.7]

**D. Thread rules**
21. The thread opened with an introduction and is not archived. **Yes.** [AGENTS 13]
22. The GPU lock was held for every GPU job and released after it. **Yes.** The PC runs took `gpu.lock` for the tests, `preview-export` and the live bench, waited while other threads held it, and released it at the end. The cloud's GPU tests ran on a software renderer in the container, which the PC's lock does not cover. [SKILLS 5]
23. A `rust-review` pass and independent reviews ran, and their findings are applied or listed. **Yes.** "Review findings".
24. The review is yes/no only, with no scores anywhere, and no reviewer was run again to get a different answer. **Yes.** [AGENTS 12]
25. The handover follows `program/HANDOVER_TEMPLATE.md`, and `docs/PROGRESS.md`, `AGENTS.md` and `SKILLS.md` match the code. **Yes.**
26. The shared checkout's branch was never switched; the PC runs used a worktree. **Yes.** [AGENTS 16]

## Review findings
`rust-review` and `rust-skills` (diff `21e8a06..b8a10ec`, library code held to the API rules): no must-fix finding. Clippy pedantic on the changed lines raised only casts between `u32` and `f32` on image sizes (bounded at 1024), `single_match_else` and missing doc backticks; the repo's gate is clippy's default set with `-D warnings`, so none was changed. The first commit's memory figure (179 MB for 88 brands) was for the full chains; the 91 dials come to 189.7 MiB full and 47.4 MiB capped, and the docs give the capped figures.

**First independent review** (`21e8a06..b8a10ec`, with a timing harness for all 91 dials):
1. The first dial of a brand decoded and uploaded on the render thread, inside the frame (about 6 ms for BMW on its machine, 0.9 to about 10 ms across the dials; 9.75 ms for BMW M on the PC). **Fixed** in `31f9335`: the `fh6-logos` worker decodes.
2. Dashes that draw no logo still load it. **Not changed**, with the reason in my call 10.
3. `preview-export --car` changes the logo and nothing else. **Documented** (my call 11, known issues, backlog P5).
4. A brand seen again was not free (one last-brand cache: every swap repeated a metadata call and an info log). **Fixed**: one slot per brand per kit; a brand already tried costs a slot lookup: no I/O, no log and no allocation in the loader (a car change still allocates the car's names).
5. Nothing bounded a dial's size at run time. **Fixed**: the header check refuses a dial over 1024 px before decoding.
6. No handover or PROGRESS entry. **Done**: this file and `docs/PROGRESS.md`.
- Pre-existing, outside the change: the OSD thumbnails after a device loss (known defect 22), and the render loop's cache keyed on a raw `*const AppConfig`. The second is fixed by fix D (`Arc::ptr_eq`).
- Its yes/no answers: no old-device logo id survives a device loss in the live dash; a steady frame does no logo work; no path leaves the logo folder, panics or hangs on a bad index; `ShowLogo` and A → B → A behave; car 0 renders as before; nothing else still builds an image-less `Logo` for a car.

**Second independent review** (the worker, `33580f3..31f9335`, with a harness that replays the render loop's logo steps over the real loader and worker, mutation runs against the shipped tests, and a wasm probe under node):
1. The frame a dial lands on still ran past the 1 ms budget (the upload alone took 0.58 to 0.90 ms on a frame of about 0.70 ms), and nothing limited a frame to one upload: three car changes before the decodes landed gave three uploads in one frame. **Fixed** in `3d9f02c`: one dial at a time, a slice of at most 128 KiB per step, steps until 0.05 ms has gone in a frame, and a decode nobody asked for on the current kit is dropped instead of uploaded (my call 5; `a_dial_uploads_a_step_per_frame_and_is_ready_when_the_last_is_in`, `a_frame_with_time_left_takes_more_steps`, `dials_upload_one_at_a_time_and_a_car_is_only_handed_its_own`, `a_decode_nobody_asked_for_on_the_new_kit_is_dropped`). On the PC the BMW M dial now takes 14 frames, 1.42 to 1.45 ms in all, and no frame carries more than 0.14 ms of it.
2. The rules that stop a stale or wrong dial from showing were untested: keeping the loaded slots across `new_kit`, or handing any landed dial to whichever car waits, passed every test, and nothing tested the render loop's side. **Fixed**: the logic moved into the kit as `CarLogo` with its own tests, and `a_new_kit_forgets_the_dials_of_the_old_one`, `a_new_kit_drops_the_dial_being_uploaded` and `dials_upload_one_at_a_time_and_a_car_is_only_handed_its_own` cover the loader's side.
3. `an_oversized_dial_is_refused_from_its_header` checked neither the height nor that the header alone decides. **Fixed**: the test adds a 1 x 1025 file and a 1025 x 1025 file cut to 48 bytes, which must be `TooLarge` and not a PNG error.
4. A dial with an odd side was drawn at a slightly different shape (HENNESSEY, 1024 x 95: 29.36 px high instead of 29.68 in Liquid Glass's zone), because the zone was fitted to the capped texture's size; 45 of the 91 dials have an odd side. **Fixed** (my call 6): `Images::source_size`; the GPU test now draws six dials, three of them odd-sided, in six zone sizes, and fails without the fix.
5. No timeout on a decode: one that hangs kept the zone empty. **Fixed** (my call 7): the name shows after `NAME_AFTER_FRAMES` polls, and a late dial still lands (`a_dial_that_does_not_arrive_shows_the_name_meanwhile_and_the_dial_if_it_comes`).
6. The OSD carousel's thumbnails keep image ids from the old kit after a device loss. **Not fixed here**: pre-existing and in `crates/fh6-osd`, outside this phase's files; it is known defect 22, with the review's proposed fix.
- Its yes/no answers: no dial can be drawn with an image id from a kit or device that no longer exists; a steady frame does no new allocation, I/O, logging, locking or channel work; the zone cannot end up wrong for the current car in any sequence it walked; nothing can panic, deadlock, leak a thread or spin; the wasm32 build works, and there every brand shows its name. `preview-export`: no finding.

**Third independent review** (the fixes, `3d9f02c`, with 43 mutants of the upload and loader code run against the tests, and a read-back probe on a software GPU; it could not check DX12 or Windows):
1. `lose_gpu` dropped the device but not a half-uploaded dial's texture, which holds the device; Direct3D 12 gives an adapter no new device while a removed one exists. **Fixed** in `16cc3f7` (my call 13): the texture lives in its upload, and `lose_gpu` calls `CarLogo::new_kit` before it drops the device.
2. No test failed when the frame budget was switched off (`f64::INFINITY`). **Fixed**: the live GPU test now asserts that a dial over 1 MiB takes more than one poll, and fails with "BMW: the whole dial went up in one poll" under that mutant on the software GPU.
3. Nothing tested that dials decoded before the first frame still upload one at a time. **Fixed**: `dials_decoded_before_the_first_frame_still_upload_one_at_a_time`.
4. The frame that finishes a dial also logged it, freed its texels and could start the next dial, past "the budget plus one step". **Fixed**: that step ends the poll (`the_frame_that_finishes_a_dial_starts_no_other`), and the budget is for the whole frame, tested with a fake clock (`the_budget_is_for_the_whole_frame_not_for_each_step`).
5. `show` set the wait back to 0 for a car shown again. **Already fixed** in `3b9d425`.
6. `clock.rs`'s header did not say the upload budget uses it. **Fixed.**
7. Test gaps: the slice test could hang on a zero-row slice (**fixed**: it asserts rows > 0); `show(None)` while a read hangs (**fixed**: `a_logo_zone_switched_off_while_a_read_hangs_stays_empty`); `new_kit` keeping the half-made texture (**gone by construction**: the texture lives in the upload); the live GPU test neither submitted between polls nor counted them (**fixed**); `MipUpload::fits` is always true for the chains callers pass (**not covered**, under known issues).
- Of the 43 mutants, 32 failed a test, 1 hung and 10 passed; six of the ten were findings 2, 3 and 7, and every mutant of those kinds now fails a test.

**Pre-landing review** (five readers at `063c15d` and `16cc3f7`, each finding checked by a second agent, which confirmed all 50 as real; for six it adjusted the proposed fix, and the fixes below follow the adjusted ones):
- *The merge with fix D, crates.* 1: with fix D's empty default driver name, PorscheAnalog greeted a parked car "HELLO, DRIVER 1" until the brand's dial was in. **Fixed** in `f71ca14` (my call 14: `Logo::loading`). 2: `show` and `waited` (already fixed in `3b9d425`). 3: `car_brand` was tested only on the shipped car list. **Fixed**: `a_car_brand_follows_its_key_then_its_make_then_names_nothing`. 4: `ensure_palette` runs in no test. **Not changed** (known issues: what no test covers). 5 to 7: no warning for a missing assets folder or an empty index, and two warnings that misstated what the zone then shows. **Fixed** in `8aa1264`. 8: `GENERIC_BRAND` and `DEFAULT_BRAND_KEY` were two literals. **Fixed**: one is the other.
- *The merge with fix D, tools.* Eight findings in `preview-export` (the car list read once per mode, the "logo:" line's wording for each case, the argument parser, the `AGENTS.md` row). **All fixed** in `3a1eb51`, with four unit tests.
- *The driver's view.* The parked greeting (as above), and for a Porsche on PorscheAnalog an empty zone while the dial loaded although the dash holds the same wordmark. **Fixed** in `d1b2876`: a maker car shows the dash's wordmark at once. A half-uploaded texture after a device loss (finding 1 of the third review). `adopt` relied on the cluster rebuild to show the car again. **Fixed**: it resets the palette cache. The dial loads whichever dash is active. **Kept** (call 10), now said in `ensure_palette`. The kit sample tinted an image white and its name `ink_label`. **Fixed.** Emblem dials are small in PorscheAnalog's 160 x 20 zone (18 brands, 196 cars, under 24 units wide). **Checked, not changed**: rendered for a Toyota and a BMW (`fix-f-emblems.png`), the BMW roundel draws about 15 x 15 px at 4K and reads as the badge, while the Toyota emblem's own lettering does not read. The zone's size is PorscheAnalog's design (the 992's wordmark slot), so this is left to that dash, under known issues and the backlog.
- *The claims in the code and the commits.* Ten claims stated more than the code does: the upload pace ("a few slices per frame", "about 0.1 s"), "the zone is empty until the dial is in" against the name after 600 frames, "once per brand" (per kit), "a square dial at most 102 px" (51 px), "the same pixels" for a test that allows 1 of 255, `fit`'s padding before the optical lift, `wordmark`'s length before upper-casing, `preview-export` loading a dial "as the app loads it", and "every render shows". **Fixed** in `8aa1264` (the last in `3a1eb51`). One commit message says "179 MB" for 178.8 MiB; commit messages are not rewritten.
- *The claims in this handover* (its draft, against the code at `063c15d`). Twelve findings: the drafts described an older tree (recounted and restated against the tree that lands), the upload pace (as above), main had moved on (merged), the 512 px test's tolerance and which uploads it compares, three rule citations (B6, B8, B10), the square dial's 51 px, a pointer to "logo:" lines the draft did not hold, two places B8 left out where memory is allocated, "91 brands" for the 91 dial files (the generic emblem never loads: 46.0 MiB for the 90 brand dials), V11's advice to set a tint in a call that has none, and no test of a broken dial of allowed size. **All fixed**; the last with `a_broken_dial_is_an_image_error_and_its_car_shows_the_name`.

## Sign-off
TJX asked on 2026-11-27 (16:00 UTC) for threads to review their own work with the installed review skills and agents, and not to wait for a sign-off. This closing review stands in for it.
