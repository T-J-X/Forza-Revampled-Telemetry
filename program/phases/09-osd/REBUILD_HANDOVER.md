# HANDOVER: the Liquid Glass OSD rebuild

**Thread:** "Build the Liquid Glass OSD"  **Branch:** `claude/project-thread-czxgs6` (draft PR #11)  **Date:** 2026-11-28
**Checklist:** [REBUILD_CHECKLIST.md](REBUILD_CHECKLIST.md) (41 lines, frozen 2026-11-27). Every line below is yes or no with its evidence; no scores. The Phase 9 handover ([HANDOVER.md](HANDOVER.md)) still describes the OSD's model, navigation and wiring, which this rebuild kept.

## What changed

The teal frosted slab is gone. The menu is light text on OLED black, and glass is the navigation only, as on Apple's Liquid Glass settings screens: the tab bar, the hint bar, the focus platter, the keyboard's key cap and the notice pill are `fh6_kit::OpticalGlass` pieces with the Liquid Glass cluster's own studio glass (`liquid_glass::studio_glass`), clear over black with crisp rims. Focus reads as light from inside the platter (an accent band set by its luminance, so any palette holds 3:1), and moves stretch like liquid on two springs. The camera rows show a rendered driving position (Blender layers over a ComfyUI alcantara) whose seat and wheel follow Seat height and Wheel offset, with the eye and the FOV fan drawn over it. Design: [docs/design/OSD.md](../../../docs/design/OSD.md); research: [docs/design/OSD_RESEARCH.md](../../../docs/design/OSD_RESEARCH.md); performance: [docs/perf/osd.md](../../../docs/perf/osd.md).

- **`crates/fh6-osd`:** `draw.rs` rebuilt (no panel, glass pieces, inner light, four inks, accent guard, echo, driving position, scroll rules, rim-crossing hide); `anim.rs` (edge springs, energise, parallax, scroll bar, tab-change snap); `drive.rs` (new: the rendered driving position and its fallback line drawing); `host.rs` (assets loaded when the kit is ready, `device_changed`, pools, allocation gates on wgpu's no-op device); `remap.rs` and `nav.rs` (D-pad left cancels, 30 s from the last change, separate cancel and time-out notices); `fit.rs` notice wording.
- **`crates/fh6-kit`:** a blurred layer is blurred only round what it drew (segment bounds and a scissor); a pool pass (`Kit::set_pools`, `pool.wgsl`) for the feathered black pools; `Painter::set_min_text_px` for thumbnails.
- **`crates/fh6-clusters`:** `liquid_glass::studio_glass` and `load_studio`, shared by the cluster and the OSD.
- **`tools/asset-forge`:** recipe set `osd` (`recipes/osd.rs`): the alcantara (ComfyUI) and five Cycles layers on the cloud lane.
- **`tools/preview-export`:** `--osd`, `--osd-state NAME`, `--no-timing`; 16 OSD states, a 180-frame motion sequence and the product page `mockups/osd/index.html`.
- **`crates/fh6-app`:** `render_loop.rs` loads the OSD's assets with each kit (`load_osd`) and passes the pools.

## Verification

Final PC run (RTX 3070 Ti Laptop GPU, DX12, driver 32.0.16.2124) at 195452b, under `gpu.lock`, with the GPU idle beforehand (1 to 2%, 1410 MHz, P0, about 48 °C):

```
cargo check --workspace                      exit 0
cargo clippy -p fh6-osd -p preview-export -p fh6-app -p fh6-kit --all-targets --no-deps -- -D warnings   clean (exit 0)
cargo test --workspace                       passed 1065, failed 0, ignored 1 (exit 0)
kit GPU tests (release): render_gpu 16 passed, glass_gpu 8 passed, fluid_gpu 2 passed
preview-export --osd (two runs): 16 states, 0.987–1.285 ms GPU median; no wgpu validation error or warning
preview-export --osd --no-assets --no-timing: exit 0, all 16 states and the sequence, no warning
live bench --demo --open-osd: CPU p50 0.899 / 0.817 / 0.841 ms (carousel / Colour / Fit), OSD closed 0.696
```

Then at 81856ae (the code of 6f1e700, after merging main's 72-dash re-cut): `cargo clippy -p fh6-osd -p preview-export --all-targets` 0 warnings; `cargo test -p fh6-osd -p preview-export` 64 and 14 passed; one timed `--osd` run 0.985–1.308 ms GPU median, within about 0.05 ms of 195452b. The earlier PC run at 4bbe604 (1010 tests, 0.975–1.289 ms) is in `docs/perf/osd.md`.

In this container (Linux, lavapipe for renders): `cargo test -p fh6-osd` 64 passed; `cargo test -p preview-export` 13 passed; `cargo test -p asset-forge` 165 passed and 4 failed, the four known `cloud::remote` tests that expect Windows paths (they pass on the PC, inside the 1010); `cargo clippy -p fh6-osd -p fh6-kit -p preview-export -p asset-forge --all-targets --no-deps -- -D warnings` clean apart from the toolchain's unknown-lint note `chunks_exact_to_as_chunks`, which predates the branch; `rustfmt --edition 2021 --config newline_style=Unix --check` clean on every changed `.rs` file. `cargo check --workspace` cannot run here (the Windows-only `wgc-bench` and the C build of `libdeflate-sys` for the Windows target), so the workspace check is the PC's.

**Since 4bbe604** (all covered by the runs above except the last merge of main, which touches no OSD code): b11602e (wording, the hold bar's position, the default-height line's gap), 378cfa5 (the scroll bar was clipped and is now drawn; rows hide while a platter rim crosses them; a tab change starts the platter on the new row; help rows 16 px taller; quiet ink 17% → 21%; remap cancel and time-out notices), the merge of main's `--no-assets` (626cf12), the 16 board renders (195452b), 3bf1634 (the dark ink on the accent and the last-resort accent read from the tokens), the merges of main's 72-dash re-cut and catalog notes (6f1e700, 407416e: no conflict in the OSD, 64 and 14 tests pass here), the board renders at the final head (eef705d), and a last merge of main (scaffold goldens and PorscheAnalog only; fh6-osd 64 and preview-export 14 tests pass here). None adds a pass or more than a few draws; 

## Closing review: the 41 lines

**A. Hard rules**

1. **yes.** `fh6-osd` binds none of A, B, X, Y: the only `0xF000` and `0x1000`–`0x8000` literals are in `nav.rs`'s tests (from line 501), which prove they are stripped. The remapper refuses them (`remap::capturable`), and the only text that names them says they are ignored ("A, B, X and Y stay with your gearbox: the menu never uses them"; the remapper's "A, B, X and Y are ignored").
2. **yes.** `grep -n "todo!|unimplemented!|TODO|FIXME|placeholder|stub" crates/fh6-osd/src/*.rs` finds nothing. Every row reads and writes the live `AppConfig`; the previews drive the same `OsdHost`.
3. **yes.** The workspace lint table denies `unwrap_used`, `expect_used` and `panic` outside tests, and clippy `-D warnings` is clean; `fh6-osd` keeps `#![forbid(unsafe_code)]`. The Rust review found none on the branch's added lines.
4. **yes.** `host.rs`'s gate runs a warm frame of `update` and `draw` on wgpu's no-op device and counts allocations with `allocation-counter`: zero (in the 64 passing tests). `tracing::` and `std::fs` appear only in `drive::load` and the studio load, which run once per kit. The review notes one first-use allocation, `defaults()` building an `AppConfig` the first time a slider is drawn (a `OnceLock`, once per process).
5. **yes.** All 16 states 0.987–1.285 ms GPU median at 3840×2160 on the PC at 195452b (two runs), and 0.985–1.308 ms at the final code (tables in `docs/perf/osd.md`), under 1.5 ms. Before the rebuild the same GPU measured 1.371–1.512 ms, and the Hood view 2.855 ms.
6. **yes.** `--demo --open-osd` ran without error in all four cases, at 4bbe604 and again at 195452b. CPU per frame and the render-thread rows are in `docs/perf/osd.md`. The OSD's own work ("build": navigate, prepare, anim) is p50 0.014–0.018 ms against main's 0.013–0.017, p99 0.041–0.045. Whole CPU per frame meets 1 ms at p50 (0.817–0.899 with the OSD open, 0.696 closed); its p99 is over 1 ms with the OSD open (1.246 / 1.159 / 1.092 at 195452b; 1.110 / 0.976 / 1.276 at 4bbe604), moving about ±0.15 ms run to run, and the spread is in upload, record and submit, not in the OSD's build or draw. The OSD only shows while the game is paused.
7. **yes.** The rebuild does not touch the visibility rules: `render_loop.rs`'s diff only adds `load_osd` (fonts and assets per kit), the kit counter and the blocker or pool under the live cluster. The OSD still draws only through the existing open path.
8. **yes.** No `.py` or other script file was added; the bpy is a Rust string in `recipes/osd.rs`. The one non-Rust source added is `crates/fh6-kit/src/shaders/pool.wgsl`, a shader beside the kit's five existing WGSL files.
9. **yes.** `git log origin/main..HEAD --no-merges` lists 24 commits, all `TJX <TJX@Ngage.com>`, with no co-author trailer. `git grep` for a `C:\Users\<name>` path, a real name or a personal email finds only the Saira font's OFL notice (the font project's own address). The research note's links are public product and documentation pages.
10. **yes.** Every PC report ends "nothing copied to the game folder"; the PC worked only in its own worktree.
11. **yes.** The only new dependency is a dev-dependency, `allocation-counter = "0.8.1"` (with wgpu's `noop` feature) for the allocation gates; its decision-log row is the 2026-11-28 "start-up loading … allocation gates" row in `docs/PROGRESS.md`.
12. **yes.** The PC's `cargo test --workspace` (1065 passed at 195452b) and clippy `-D warnings` are above, with the container's runs; rustfmt is clean. `cargo check --workspace` passed on the PC at 195452b.

**B. Rendering laws**

13. **yes.** The OSD never premultiplies by hand: every colour goes through the kit's `Color` and `Paint` (`with_alpha`, `fade`), which premultiply once in the shader; the pools are written premultiplied by `pool.wgsl` and the kit's tests `pools_are_the_same_picture_as_painting_them_first` and `pools_show_under_an_empty_frame_and_match_their_formula` pass on the card.
14. **yes.** The platter's bottom is held at least 40 px below its top (`focus_rect`), the tab fill at least its height wide, the scroll clamped to `0..max_scroll`, the driving position's scale clamped to 0.6..1 and its fade to 0..1; springs are fed finite targets only. `drive_fit` and the anim tests cover the edges.
15. **yes.** With no studio the glass mirrors the kit's analytic studio; with any driving-position layer missing the view draws its line drawing (`drive.rs`), proven by `preview-export --osd --no-assets`. Both load once per kit in `OsdHost::load_assets`, never in `update` or `draw`.
16. **yes.** All text is the kit's MSDF text in the font pack's `Text` and `Display` roles (`FontRole`), with the pack's 81 faces on the PC.
17. **yes.** No panel and no full-canvas fill: the column's darkness is a feathered black pool from the kit's pool pass, and the dim is the kit underlay. Over black the column area outside the text and glass is off (`stretch` on `06_fit_fov.png`).
18. **yes.** Motion runs on the host's monotonic `now` and the frame's `dt` (`OsdAnim::update(to, dt)`). The only wall-clock read in the crate is `model::new_seed`, which predates the rebuild and mixes the time into the Random palette's seed when the driver presses for a new palette; it is entropy, not time.
19. **yes.** Colours come from the palette roles and the tokens: inks from `roles.ink*`, the dark mark on the accent from `roles.oled_black`, the last-resort accent from the tokens' Liquid Glass primary (3bf1634). The hex literals left are in tests.
20. **yes.** Every glass piece is `OpticalGlass` from `osd_glass` (the cluster's studio glass with the OSD's clarity), drawn as `Shape::Blob` pieces in one glass layer; `grep` finds no legacy `Glass` or `glass_plate` in `fh6-osd`.

**C. OSD design**

21. **yes.** The only glass is the tab bar, the hint bar, the focus platter, the key cap and the notice pill (`panel()` builds exactly these `Piece`s); the cold reviewer noted "no panel (rows over brighter game)".
22. **yes.** Clear over black with no frost or tint of its own (the a11y and critique crops: "clear, with crisp rims"); the glass mirrors the shared studio; it materialises by thickness on open (`osd_glass(…, grown, …)` scales rise and bevel from 0), and the content fades with it; close is one frame.
23. **yes.** The pieces sit in fixed bands (tab bar y 228–324, rows 424–1736, hint bar 1802–1890, notice above the tab bar at y 110–170), and the platter is clipped to the rows' band; the 180-frame sequence shows no overlap in any frame.
24. **yes.** At rest, text inside the platter sits 30 px or more inside the rim vertically (help descenders 36 px after 378cfa5) and 52 px horizontally, and headings sit 65–101 px from it (a11y re-check: 69 to 101 px; critique 2: 65 to 97 px). In motion, a row or heading that a rim crosses hides (`rim_clear`, tested), so no text is seen in a moving rim (`seq_moves` frames 107–152).
25. **yes.** In every browse state the platter's lit band passes the blur gate (4.4 to 5.1 at 960×540, a11y re-check S1), and focus also reads by shape, rim and the band's luminance (critique 2: "focus survives without colour"); on the keyboard the solid focused key leads (7.21 to 9.92:1 against its neighbours).
26. **yes.** Labels and values 30 px cap, help and status 26, the gearbox line 26, keycaps 22, section headings 24 in sentence case (`LABEL_CAP`, `HELP_CAP`, `SECTION_CAP`, `HINT_CAP` and the keyboard's sizes). These meet the frozen line; they are under ten-foot guidance at 2.2 m, which is open below.
27. **yes.** Over the game every label and value is 9:1 or better (the column's pool leaves a median relative luminance of 0.002 behind the rows; a11y re-check). The weakest text is the echo label over the sky, now in label ink (378cfa5), and the header at 13.8:1 over the render's dimmed frame (4.76:1 computed for a bright frame).
28. **yes.** The platter and tab fill stretch on `fast` and `expressive` and settle with no overshoot past the next row (`a_move_down_stretches_then_settles`); with `ReducedFlash` both edges use `fast` (no stretch, tested); open is 200 ms; close is one frame.
29. **yes.** The energise light is a spring held up during hold-to-repeat (`energise_holds_while_repeating`: no pulsing, decays after release). `visual-review flash --frames 24-100` (the held adjust) prints yes for the WCAG count and for the 25% field; its strip-height line marks the echo's value digits and the live cluster's speed digits, which change with the data (57 px glyphs), not a flash.
30. **no.** The live cluster is not the strongest picture in every state. By summed light the column emits 1.3 to 1.6 times the cluster (critique 2, almost all of it label text), and the cold reviewer noticed the echo value, the focused row and the driving position before the cluster. Dimming unfocused labels to 75% (the critique's suggestion) would still leave the column brighter than the cluster, and a settings screen leads with the value being changed by design (OSD.md "My calls"). Not fixable inside this checklist without a design change; reported for TJX's check C.
31. **yes.** The camera rows show the rendered driving position: the seat moves with Seat height and the wheel with Wheel offset (`12_fit_seat_height_drive.png`, `13_fit_wheel_offset_drive.png`), the focused part is rim-lit in the accent, and with the layers missing the line drawing shows (`--no-assets`).

**D. Workflow and assets**

32. **no.** The order was not followed exactly. The research note came first (22:09), but a code spike rendered with the kit (e3383ff, 22:26) came before the Penpot page, whose boards were then made from the spike's renders (22:39); ComfyUI and Blender ran after that, before the driving-position build. Later Penpot changes are the final refresh (line 33); the assets did not need a new pass for it, because only the menu's layout changed, not the driving position.
33. **yes (after the merge).** The page `09 · OSD` (`6fe75c5e-bb22-8084-8008-fd8a2bcd3335`) holds all 16 states from the final renders in `docs/design/osd/` at eef705d: boards 01 to 11 were re-filled on 2026-11-28 at 08:49 GMT, after the merge (353b931), and boards 12 to 16 were added; the IDs are in `docs/design/PENPOT.md` and below. Penpot is shared between threads and was queued, so this came after the merge rather than holding it. The exports (3840×2160) are in the Drive masters folder under `osd/rebuild-2026-11-28/penpot/`.
34. **yes.** The ComfyUI alcantara (seed 90101) and the five Blender layers (seed 90211) are used in every camera-row state, each with its row in `assets/osd/ASSETS.md` (recipe, seed, licence, master).
35. **yes.** The masters are 4K-class (the alcantara 4096², the Blender layers at 4x the OSD's scale, 16-bit) in `G:\My Drive\FH6 WheelDash v2 - 4K Masters\osd\` (`comfyui/`, `osd_drive/`), listed in `assets/osd/ASSETS.md`; git has only the compressed runtime PNGs (21 to 101 KB).
36. **yes.** The PC's GPU steps held `gpu.lock`; the asset batches ran on the cloud lane through `asset-forge --lane cloud` under `gpu-cloud.lock`, each ended with `cloud down`. Cloud spend: this thread $0.333 of the project's $1.868 at the time of the second batch.
37. **no.** The first build's render was posted (22:20, "First look at the new OSD direction", and 22:39 with four states), but not as a `DIRECTION CHECK` post with its self-review (decision, findings, what was done). It happened, so it cannot be fixed now: the direction check's substance is the spike's own review in OSD.md "Approaches" and the decisions in `docs/PROGRESS.md`.
38. **yes.** `preview-export --osd` writes `mockups/osd/index.html` (the hero over the game, the driving-position crops, all 16 states, the GPU table against 1.5 ms).
39. **yes.** This handover holds findings and yes/no answers only.
40. **yes.** `## Visual review` below answers V1 to V26; the cold reviewer ran once with the neutral packet.
41. **yes.** `design:design-critique` (two rounds), `design:accessibility-review` (audit and re-check) and `rust-review` (two passes) ran on the final build; every finding is fixed or answered under "Review findings".

## Visual review (fh6-visual-review V1 to V26)

The OSD is a menu, not a dash, so the dash lines about gears, shift lights and the limiter are NOT ASSESSED for the OSD (the live cluster has its own review); the rest are read for the menu.

1. **no.** The expectations frozen before the build are the 41-line checklist (2026-11-27); no separate freeze was written before this round's renders were opened. A process line: reported, cannot be fixed now.
2. **no.** No cold description of the composite, its mirror and its luminance view was written before the notes at this round. Reported, cannot be fixed now.
3. **yes.** The cold reviewer ran once (05:40Z) on a neutral, logo-masked packet of 8 frames; each finding is recorded below as fixed or not.
4. **no.** Its first three: the echo's value, the focused row with the tab bar, the driving position; the cluster fourth. For the OSD the focused row is the lead, and it came second. The answer stays (line 30 carries it).
5. **yes** (read for the menu). The blind glance read named the tab, the focused row and its value correctly on all 8 frames, the remapper included.
6. **yes.** Every label and value over the game is 9:1 or better, the platter's lit band 4.33 to 4.89:1, the selected tab fill 3.21 to 3.85:1, switches and slots 3:1 or better (a11y re-check).
7. **NOT ASSESSED.** The OSD has no shift lights or strobe.
8. **yes.** The smallest OSD text is 22 px cap (keycaps), above 17.7 px; information strokes are 2 px or more (unfilled slider line 2 px, outlines 1 px at full contrast, hold track 14 px).
9. **NOT ASSESSED.** No limiter in the OSD's sequence; the live cluster's digits are the cluster's review.
10. **yes.** In the glance and squint views the platter's band and the echo persist longest and the headings and hints fade first (a11y re-check glance set; critique 2: "what draws the eye first: the platter's lit floor and the cluster's gear").
11. **NOT ASSESSED.** A menu's labels are static by nature; the brightness balance with the cluster is line 30.
12. **yes.** Red appears only as the cluster's redline and the warning role; the accent is moved 25° away from it. In the `cvd` views focus reads by shape, rim and luminance, and switches by knob position and "On"/"Off".
13. **no.** Asked for recognition, the cold reviewer answered "Don't know". The OSD recreates no real display; its benchmark is Apple's Liquid Glass settings screens, which it did not name.
14. **NOT ASSESSED.** No real display's feature inventory: the menu is benchmarked on principles (OSD_RESEARCH.md P1 to P12), answered in lines 21 to 29.
15. **NOT ASSESSED.** No reference display to compare masses with.
16. **yes (after the merge).** The Penpot boards are the final renders themselves (eef705d), so board and render match; the render is far richer than the Phase 9 boards (`before-osd-cluster-tab.png` against `review-07-cluster-height.png`). On OSD 09, where the grid shows, the focus platter outline was moved to y 712–920 to sit on the Hold time row, whose help line moved it down.
17. **yes.** At full frame the glass reads as glass with one key light (upper left) and the driving position's carbon, alcantara and satin read without zoom.
18. **yes.** The glass checks of `references/materials.md`: clear over black, rims that follow size, highlights that move with the parallax tilt, no glass on glass, one layer.
19. **yes.** Clear over black, crisp rims scaled per piece (bars and platter 36 px edge, key cap 20 px), highlights move between states with the platter's height, no piece sits on another, one layer with no seams.
20. **yes.** `stretch` on the over-black states: what is not black is text, glass, the pools' feather and the driving position; the corners are off; no banding steps in the pools (the pool pass is evaluated per pixel in float).
21. **yes.** `motion` over the 180 frames: no stalls; the platter edges move on the frame the input changes and settle without overshoot past the next row.
22. **yes.** `flash`: the WCAG count line is yes over the whole sequence and over the held adjust; the faster-than-3 Hz areas are the live cluster's needle and digits, the echo's value digits stepping with hold-to-repeat and the slider knob moving: moving content, not a flashing area.
23. **NOT ASSESSED.** No gear change in the OSD.
24. **yes.** Each `motion` spike is an input event: frames 29–56 the held adjust's value steps, 104 the move down, 131 the energise decay, 144 the tab change (the rows' crossfade).
25. **yes.** Against the Phase 9 OSD (`before-osd-cluster-tab.png`): the slab is gone and nothing it did better is lost (all rows, the carousel, the keyboard and the remapper remain, and GPU time dropped in every state).
26. **NOT ASSESSED.** The OSD has no goldens; the kit's goldens pass on the card (1010 tests).

## Review findings and what was done

**Cold reviewer (once, packet k7).** Fixed: the remapper's wording contradiction (b11602e), its hold bar alignment, the default-height line through the cluster, "saved in config only" wording (now "Camera patch is off: the game keeps its own camera"), the missing scroll indicator (it was clipped; 378cfa5). Not ours: "61 GPU 99%" is in the review backdrop. Answered: text keycaps instead of glyphs (open, below); small secondary text (open, below); tiny thumbnails (they keep the gear and shapes; text under 12 px is left out); the 10 Hz strobe option is the cluster's (ReducedFlash offers a gentle pulse); the echo moves with focus by design; the empty area under the rows is the end of the list; "TJX" is the custom driver name in the review config; "GT7 Hood Pro" names another game and was reported to the coordinator.

**Design critique round 2.** Fixed: text in a moving rim (`rim_clear`), the tab change lighting the wrong row (`OsdAnim::tab_changed` snaps), scrolled lists (scroll bar drawn, block-aligned scroll, no orphan heading, last row unfaded), the focused thumbnail's white ring (now the hairline), the remapper's combination next to its bar ("Holding View + L3"), the echo label in label ink, long echo strings at 84 px whatever they contain, the open slide written into the spec, the vertical clearance rule restated. Answered: the caret exists and blinks (the still caught its off phase); the pale accent keeps the driver's own low-chroma colour, with the inner light's chroma floor; thumbnails tinted by the platter band and similar at 178 px, and the driving view's plain dash and faint quilting, are left as they are; the dash-blocker outline's hard ends are the blocker's own shape; minus sign and degree spacing are the model's formatting (fix D's file) and are left for a copy pass.

**Accessibility re-check.** Fixed: the scroll bar, chevrons and quiet ink (17% → 21%, 2.95:1 → over 3:1), help lines 21–25 px from the rim (rows 16 px taller), the remapper's candidate far from its bar. Answered: unfocused slider fill and slot differ by thickness (4 px against 2 px), both over 3:1 against the column; the selected tab label's blur-gate figure fell to 2.1 to 2.2 while its WCAG contrast is 4.79 to 5.08:1; the violet stretch of the hue track (95 of 1,040 px) is the spectrum itself; the dash layer has no outline (it is a backdrop to the seat and wheel, which are outlined when lit); D-pad repeat speed has no setting.

**Rust review (second pass).** Fixed: the last row faded at the end of a scrolled list (`scroll_target` pads by `EDGE_FADE`, tested), the remap cancel and time-out notices (`RemapOutcome::TimedOut`, tested), `DPAD_LEFT` now `nav::LEFT`, the always-true scroll check, `OsdHost::pools`' doc. Left: `draw_row` at 137 lines; `#[allow]` over `#[expect]` in two places; `NAVY_HUES` is hue only (it also moves light blues such as #3B82F6 to the primary, then sapphire).

## Open (for TJX or a later phase)

- **Text size.** No text-size setting (WCAG 1.4.4), and the 30, 26, 24 and 22 px classes are under ten-foot guidance at 2.2 m on a 55-inch screen (labels about 15 arcmin against about 16). Both move every row and need the column to widen or the tabs to scroll.
- **Glyph keycaps.** The hint bar spells out "D-PAD", "LB RB" and "VIEW / MENU"; controller glyphs need an icon set.
- **Line 30.** Whether the live cluster must out-shine the menu by summed light, or the edited value may lead (the cold reviewer's order).
- **FOV direction (check C).** Whether the game's cockpit FOV is vertical or horizontal changes the fan's angle; check it in game.
- **The notice pill** ("Saved", "Restored") has no review state.

## Penpot

Page `09 · OSD` `6fe75c5e-bb22-8084-8008-fd8a2bcd3335`. Boards: OSD 01 `6fe75c5e-bb22-8084-8008-fd8aeed8fa18`, 02 `…fd8aeee0be9d`, 03 `…fd8aeee4d8b2`, 04 `…fd8aeee886b3`, 05 `…fd8aeeece75d`, 06 `…fd8aeef078b7`, 07 `…fd8aeef3fe24`, 08 `…fd8aeef78e1b`, 09 `…fd8aeefb62ed`, 10 `…fd8aeeff0834`, 11 `…fd8aef02ce42` (all `6fe75c5e-bb22-8084-8008-` prefixed); OSD 12 `347e5ae2-9e0b-80b1-8009-039932ddaebe`, 13 `…0399339dd4cb`, 14 `…03993439fd02`, 15 `…039934d498e8`, 16 `…0399358c001d` (all `347e5ae2-9e0b-80b1-8009-` prefixed, in a row at y 9600 and 12240 below the reference boards). Each board's render rectangle is the 960×540 final render from `docs/design/osd/` at eef705d, imported with `uploadMediaUrl` from its pinned raw GitHub URL and scaled ×4 (re-rendered after merging main: only the 72-dash catalog's count and names and the accent's dark ink changed from 195452b).
