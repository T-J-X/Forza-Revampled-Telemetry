# OSD performance

## Liquid Glass rebuild (2026-11-28)

Measured on the development machine's RTX 3070 Ti Laptop GPU (DX12, driver 32.0.16.2124) at 3840x2160 with release builds, at 4bbe604 (the rebuild after the design-critique and accessibility fixes, merged with main). Every GPU step ran under `gpu.lock` with the GPU idle beforehand (1 to 2% utilisation, 1410 MHz, P0, about 48 °C). Later commits change only wording, two small layout fixes and main's logo loading; they move no pass or draw.

Targets: under 1.5 ms GPU for the whole OSD frame at 4K (Phase 9), and the app under 1 ms CPU per frame.

### Summary

| Item | Result | Target |
|---|---|---|
| GPU, whole OSD frame, 16 states (`preview-export --osd`, two runs) | 0.975–1.289 ms median; worst 06 Fit & camera FOV and 12 seat height, 1.29 ms | < 1.5 ms: **met** in every state |
| GPU, the 11 Phase 9 states before the rebuild (main 2f85973, same GPU, one session, 2026-11-27) | 1.371–1.512 ms median; Hood view 2.855 ms (over budget) | |
| CPU per frame, OSD open (`--demo --open-osd`, 12 s): carousel / Colour tab / Fit & camera tab | p50 0.903 / 0.797 / 0.820 ms; p99 1.110 / 0.976 / 1.276 ms | < 1 ms: **p50 met; p99 met on Colour, not on the carousel or Fit** |
| CPU per frame, OSD closed (same runs) | p50 0.681 ms, p99 0.874 ms | < 1 ms: met |
| The OSD's own render-thread work ("build": navigate, prepare, anim) | p50 0.012–0.017 ms, p99 0.047–0.063 ms | no larger than before: main measured p50 0.013–0.017 and p99 0.040–0.060 ms; p99 at this size moves 0.03 ms run to run (the branch gave 0.054–0.084 in an earlier run) |

The OSD only shows while the game is paused, so a p99 over 1 ms never costs a racing frame. The Fit & camera run's p99 (1.276, max 2.119) comes from one or two hitches in upload, record and submit (max 1.763 ms), not from the steady state; its p50 matches the others. Against main at the same commit (44565a2 against fd13cf5, eight alternating runs), the rebuild is 0.10–0.12 ms cheaper at p50 with the OSD open and equal with it closed: it renders in 5 passes where the old OSD took 9.

### Final check (195452b and the final code)

The same machine and conditions, after the closing-review fixes. At 195452b: two timed runs (a, b), `cargo test --workspace` 1065 passed, clippy `-D warnings` clean, kit GPU tests pass, `--no-assets` clean. At 81856ae (the code after merging main's 72-dash re-cut, the same OSD code as the final head): one timed run (c). Every state has 6 passes and 3 segments (02: 27 and 8); there are 2 more draws than at 4bbe604 (5 more on the carousel), from the scroll bar and the remap card's held-buttons line. No wgpu validation error or warning in any run. Median / minimum in ms:

| State | Run a | Run b | Run c (final code) | Draws |
|---|---|---|---|---|
| 01 Cluster tab, carousel | 1.212 / 1.175 | 1.215 / 1.176 | 1.202 / 1.169 | 76 |
| 02 Cluster tab, Hood view | 0.992 / 0.977 | 0.987 / 0.979 | 0.985 / 0.974 | 79 |
| 03 Colour, custom hue | 1.089 / 1.060 | 1.117 / 1.036 | 1.129 / 1.060 | 24 |
| 04 Colour, random | 1.095 / 1.060 | 1.112 / 1.075 | 1.132 / 1.033 | 24 |
| 05 Type & logo | 1.066 / 1.041 | 1.094 / 1.035 | 1.114 / 1.110 | 24 |
| 06 Fit & camera, FOV | 1.248 / 1.226 | 1.285 / 1.255 | 1.308 / 1.252 | 31 |
| 07 Fit, cluster height | 1.135 / 1.099 | 1.128 / 1.098 | 1.165 / 1.108 | 24 |
| 08 Fit, dash blocker | 1.191 / 1.149 | 1.195 / 1.153 | 1.215 / 1.161 | 23 |
| 09 Controls | 1.109 / 1.083 | 1.111 / 1.084 | 1.159 / 1.079 | 24 |
| 10 Keyboard | 1.263 / 0.997 | 1.056 / 0.989 | 1.083 / 1.030 | 24 |
| 11 Remap | 1.020 / 0.995 | 1.034 / 1.001 | 1.037 / 1.001 | 24 |
| 12 Fit, seat height | 1.193 / 1.185 | 1.252 / 1.220 | 1.304 / 1.242 | 31 |
| 13 Fit, wheel offset | 1.272 / 1.227 | 1.272 / 1.233 | 1.297 / 1.227 | 31 |
| 14 Accent navy | 1.087 / 1.067 | 1.094 / 1.057 | 1.096 / 1.057 | 24 |
| 15 Accent violet | 1.094 / 1.057 | 1.094 / 1.059 | 1.120 / 1.049 | 24 |
| 16 Accent pale | 1.094 / 1.024 | 1.096 / 1.057 | 1.112 / 1.066 | 24 |

Worst state 1.308 ms (06, run c), under 1.5 ms. Run a's 10 Keyboard median is one noisy run (its minimum is 0.997).

Live bench at 195452b (`--demo --open-osd`, 12 s, p50 / p99 / max ms):

| Case | CPU per frame | build | draw | upload, record, submit | Frames, draws, passes |
|---|---|---|---|---|---|
| a, carousel | 0.899 / 1.246 / 1.516 | 0.018 / 0.045 / 0.118 | 0.130 / 0.220 / 0.306 | 0.503 / 0.687 / 0.904 | 1198, 49.0, 5 |
| b, Colour tab | 0.817 / 1.159 / 1.520 | 0.014 / 0.041 / 0.076 | 0.068 / 0.117 / 0.140 | 0.485 / 0.659 / 0.813 | 1164, 17.8, 5 |
| c, Fit & camera tab | 0.841 / 1.092 / 1.514 | 0.018 / 0.045 / 0.101 | 0.086 / 0.145 / 0.188 | 0.489 / 0.649 / 0.951 | 1197, 24.8, 5 |
| d, OSD closed | 0.696 / 0.941 / 1.022 | 0.005 / 0.030 / 0.062 | 0.031 / 0.066 / 0.091 | 0.414 / 0.566 / 0.658 | 1187, 13.7, 3 |

p50 matches 4bbe604 within 0.02 ms. p99 moved by up to 0.18 ms each way (a and b up, c down), which is this laptop's run-to-run spread; the OSD's own build stays at p99 0.041–0.045 ms.

### GPU per state at 4bbe604 (generated by `preview-export --osd`)

Median / minimum in ms for two back-to-back runs (a, b), then draws, passes and layer segments. 600 warm-up frames before the first timed state, 100 between states, then 20 frames timed one by one.

| State | Run a | Run b | Draws | Passes | Segments |
|---|---|---|---|---|---|
| 01 Cluster tab, carousel | 1.207 / 1.201 | 1.188 / 1.172 | 71 | 6 | 3 |
| 02 Cluster tab, Hood view | 0.975 / 0.971 | 0.998 / 0.978 | 79 | 27 | 8 |
| 03 Colour, custom hue | 1.110 / 1.072 | 1.089 / 1.062 | 22 | 6 | 3 |
| 04 Colour, random | 1.105 / 1.048 | 1.098 / 1.062 | 22 | 6 | 3 |
| 05 Type & logo | 1.094 / 1.047 | 1.058 / 1.050 | 22 | 6 | 3 |
| 06 Fit & camera, FOV and driving position | 1.289 / 1.237 | 1.289 / 1.237 | 29 | 6 | 3 |
| 07 Fit & camera, cluster height | 1.142 / 1.097 | 1.120 / 1.086 | 22 | 6 | 3 |
| 08 Fit & camera, dash blocker | 1.198 / 1.149 | 1.189 / 1.146 | 21 | 6 | 3 |
| 09 Controls | 1.134 / 1.128 | 1.119 / 1.068 | 22 | 6 | 3 |
| 10 On-screen keyboard | 1.039 / 1.013 | 1.030 / 1.021 | 22 | 6 | 3 |
| 11 Remap capture | 1.044 / 1.002 | 1.034 / 0.998 | 22 | 6 | 3 |
| 12 Fit & camera, seat height | 1.286 / 1.281 | 1.278 / 1.271 | 29 | 6 | 3 |
| 13 Fit & camera, wheel offset | 1.284 / 1.178 | 1.264 / 1.220 | 29 | 6 | 3 |
| 14 Accent navy (falls back to the primary) | 1.105 / 1.058 | 1.095 / 1.056 | 22 | 6 | 3 |
| 15 Accent violet | 1.077 / 1.064 | 1.095 / 1.094 | 22 | 6 | 3 |
| 16 Accent pale mint | 1.096 / 1.057 | 1.095 / 1.057 | 22 | 6 | 3 |

No wgpu validation error or warning appeared in either run or in the kit's GPU tests (render_gpu 13, glass_gpu 8, fluid_gpu 2, all passing on the card).

### What it took

1. **A blurred layer is blurred only round what it drew** (`fh6-kit`, segment bounds and a scissor through the blur pyramid). The Hood view's two soft-shadow layers were blurred across the whole 4K target: 2.8 → 1.2 ms.
2. **The feathered black pools moved into one kit pass** (`Kit::set_pools`, `pool.wgsl`). Painting four large feathered shapes cost about 0.4 ms (an A/B on the card: 12 seat height 1.43 ms with them, 1.006 without); evaluating them in one pass costs about 0.2 ms. Every state dropped 0.12–0.22 ms.
3. **The driving position is cheap.** Its five rendered layers, the rim pass, the fan and the legend together cost about 0.1 ms (12 with everything else off: 0.895 ms against 1.006 with only the pools off).
4. **A longer warm-up for the first timed state.** The carousel's early 1.66 ms reading was the laptop GPU still clocking up after 100 frames; with 600 it reads 1.2 ms.

### Reproduce

```
cargo build --release -p preview-export -p fh6-app -p stand-in-game -p mock-udp
target/release/preview-export --osd                      # GPU table and mockups/osd/ (hold gpu.lock)
target/release/fh6-stand-in.exe                          # separate terminal
target/release/FH6_WheelDash.exe --demo --open-osd --game-process fh6-stand-in.exe --config <copy of data/config.json> --bench-seconds 12 [--pad-script <file>]
```

The pad timeline starts at process start and the render loop about 6 s later, so a script that switches tabs presses RB at 7.4 s or later (`0 0 / 7400 0x0200 / 7500 0` for the first tab to the right).

# Phase 9 (the OSD before the rebuild)

All numbers on the development machine: RTX 3070 Ti Laptop GPU (DX12), 3840x2160, release builds, 2026-11-22 (GPU table re-measured 2026-11-23 after the merge with Phases 6 and 8).

Target (Phase 9 MASTER_PROMPT): under 1.5 ms GPU for the whole OSD at 4K.

## Summary

| Item | Result | Target |
|---|---|---|
| GPU, whole OSD frame (live cluster, glass panel, rows, focus bloom, underlay dim, scrims) | 1.26–1.32 ms median over the 11 states, in the pack fonts; carousel (with 5 live thumbnails) 1.32 ms | < 1.5 ms: **met** (0.18 ms headroom) |
| GPU, same frames before the optimisations below | 3.2 ms (8.0 ms with the carousel) | |
| CPU per frame, OSD open (`--demo --open-osd`, 12 s) | 0.99 ms p50, 2.24 ms p99 | < 1 ms: **p50 met, p99 not met** |
| CPU per frame, dash only (same run conditions) | 0.76 ms p50, 1.48 ms p99 | Phase 4/5 debt (p99) |
| OSD cost on the render thread (navigate, prepare, anim, thumbnails update) | 0.013 ms p50, 0.061 ms p99 ("build" column) | |

The extra CPU with the OSD open is almost all `Kit::render` packing and wgpu recording of a larger scene (0.76 ms p50 vs 0.57 ms dash-only). The OSD only shows while the game is paused, so the p99 overshoot never costs racing frames; it is recorded as a follow-up with the existing Phase 4 p99 debt.

## GPU per state (generated by `preview-export`)

Whole-frame GPU time from timestamp queries around every pass (shape passes, glass backdrop, bloom, resolve). 100 frames are submitted back to back first so the laptop GPU clocks up, then 20 frames are timed one by one. Median and minimum in ms. One run on the merged branch (Phase 6 font pack loaded: 81 faces, 9 roles; Phase 8 blocker rows and the live blocker preview in 08), with the GPU at about 20% from other work. 3 layer segments per frame (live cluster glass, panel glass), 27 draw calls (35 with the carousel).

| State | GPU median | GPU min | draws |
|---|---|---|---|
| 01 Cluster tab, carousel focused | 1.316 | 1.269 | 35 |
| 02 Cluster tab, editing the Hood view | 1.302 | 1.298 | 35 |
| 03 Colour, custom, hue slider | 1.271 | 1.265 | 27 |
| 04 Colour, random | 1.267 | 1.212 | 27 |
| 05 Type & logo | 1.263 | 1.177 | 27 |
| 06 Fit & camera, FOV + schematic | 1.297 | 1.245 | 27 |
| 07 Fit & camera, cluster height | 1.277 | 1.245 | 27 |
| 08 Fit & camera, dash blocker + live blocker preview | 1.297 | 1.287 | 27 |
| 09 Controls | 1.287 | 1.202 | 27 |
| 10 On-screen keyboard | 1.300 | 1.218 | 27 |
| 11 Remap capture | 1.287 | 1.253 | 27 |

**Measurement caveat.** This GPU is shared with other work on the machine. During one session another process held it at 98% (127 W, 87 °C) and the same frames read 2.3–3.5 ms; with the GPU at 11–16% they read as above. `preview-export` now prints the minimum next to the median so contention is visible: when the two are close (as in the table), the number is clean.

## What it took to get under 1.5 ms

1. **Thumbnails skip glass backdrops.** Each carousel thumbnail is a real cluster drawing into the main scene. Its glass layer would snapshot and blur the whole 4K work rectangle (1.2 ms each). The new `Painter::set_backdrops(false)` keeps their tint, rims, sheen and fresnel without the backdrop: 8.0 → 2.7 ms.
2. **The dim moved into the resolve.** A full-screen black shape blends into two HDR work targets at 4K (about 0.9 ms). The new `Kit::set_underlay(alpha)` clears the target to premultiplied black and composites it under everything in the resolve pass the frame already runs: 1.64 → 0.76 ms with otherwise identical content.
3. **Warm-up before timing.** Timing single frames that each wait for readback let the laptop GPU drop clocks between frames (2–6 ms noise). Burst first, then time.

With the panel's own backdrop blur kept (the brief asks for blur behind the panel), the critique's legibility fixes (black under the panel, feathered scrims behind the focus echo and the live cluster) and bloom on the focus ring and tab underline, the frame settled at 1.18–1.32 ms. The round 2 fixes (the focused row grows to hold its help line, and the schematic is wider) moved it to 1.28–1.48 ms. The FOV schematic state (06) is the most expensive, with little headroom; its wedge and scrim are the first things to trim if a later phase adds to the OSD.

Measured costs of individual pieces (back-to-back A/B on an idle GPU): panel backdrop blur 0.25 ms, bloom 0.2 ms, panel black base 0.07 ms, both scrims about 0.2 ms.

## Reproduce

```
cargo build --release -p preview-export -p fh6-app -p stand-in-game -p mock-udp
cargo run --release -p preview-export                     # GPU table, mockups/osd/
target/release/fh6-stand-in.exe                           # separate terminal
target/release/FH6_WheelDash.exe --demo --open-osd --game-process fh6-stand-in.exe --config <copy of data/config.json> --bench-seconds 12 --bench-out osd-open.md
```

Use a copy of `config.json` for `--config`: OSD edits are saved to whichever file the overlay was started with.
