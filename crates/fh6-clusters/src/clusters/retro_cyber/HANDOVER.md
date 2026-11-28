# HANDOVER: Dash · RetroCyber (catalog row 19)

**Thread / branch:** "Rebuild RetroCyber as the Supra A70 dash" (slot D), `claude/project-thread-191tnq`, draft PR #9  **Date:** 2026-11-28  **Result:** finished; the closing review is below, and the job is reported for the merge.

Findings and yes/no answers only; there are no scores in this file. TJX is hands-off (AGENTS.md rule 12): the self-review stands in for TJX's direction check and sign-off. TJX's own words in the thread are quoted where they decided something.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| The introduction was posted, and the thread is not archived | yes | the thread's first message; the thread is open |
| `RESEARCH.md` is written, with sources | yes | [RESEARCH.md](RESEARCH.md): the A70 Panoramic Digital Meter, its features with sources, the match table, deliberate changes A to G, the gaps |
| The design board is the Penpot page `07 · RetroCyber`, 4 states per view, its IDs here and in the README | yes | "Penpot board" below; the boards came after the code (Penpot was held by other threads), and the one later code change (f5a33f2) was put on the boards too |
| A bespoke ComfyUI asset and a bespoke Blender asset are used, with ledger rows and 4K masters on Drive; the before and after pair is here | yes | [assets/clusters/retro_cyber/ASSETS.md](../../../../../assets/clusters/retro_cyber/ASSETS.md); "Assets and masters"; "Before and after pair" |
| The module is complete, with no TODOs, and goldens pass | yes | 17 unit tests and dash_gates pass; the 8 goldens pass (rendered and checked in the cloud, f5a33f2) |
| 4K renders, the 60-frame sequence and the product page are in `mockups/RetroCyber/` | yes | `preview-export --cluster RetroCyber` on the PC at 43f7d8a (product page with the PC's GPU timings) and in the cloud at f5a33f2 |
| The live bench report and the GPU time under 0.5 ms are recorded | yes | "GPU budget": every state 0.423 to 0.487 ms at a1fd67c; live CPU share +0.032 ms |
| The direction check was posted and self-reviewed, with its findings here | yes | "Direction check" (posted 28 Nov 04:20 UTC) |
| Every line of the closing review is yes, or a line that could not be fixed afterwards is reported | yes | V5, V13 and V15 reported under "Visual review" |
| The visual review (V1 to V26) is here, each line with its evidence | yes | "Visual review" |
| The handover is written, the job is reported for the merge | yes | this file; READY FOR MERGE posted in the thread |

## What was built
- `crates/fh6-clusters/src/clusters/retro_cyber/`
  - `mod.rs`: the zone tables for both views (`Geom`, with the stepped lens outline: a low strip across the width and a raised part over the centre window, joined by smooth shoulders), the tach scale (`Scale`: one bar per 250 rpm, numerals on the rising curve, red from the thousand nearest the shift point), the six-pod shift bracket, the pedal, slip, power and boost gauges, the lateral G readout, the sector boot, the limiter hold (`LimiterHold`), the surround's ring cover (`Geom::ring_cover`) and the mesh zones (`Geom::mesh_zones`), the cluster (`update`, `draw`) and 15 unit tests.
  - `RESEARCH.md`, `README.md`, `spec.md` (the catalog brief, superseded by `RESEARCH.md` where they differ), this file.
- `crates/fh6-clusters/src/common/seg7.rs`: two emission modes, `Emission::Halo` and `Emission::SoftHalo` (the bloom modes' lobes as a local glow, with no bloom pass), with a test. Shared code: see "Shared places touched".
- `tools/asset-forge/src/recipes/retro_cyber.rs`: the ComfyUI phosphor plate and surround grain (SDXL), the Blender surround per view and key light, and the cluster's `ASSETS.md`.
- `assets/clusters/retro_cyber/`: `phosphor_grain.png`, `surround_{cockpit,hood}_{l160,l110}.png` and `ASSETS.md`. The octagon `housing_*` renders of the old design are deleted.

## Telemetry
- **Through `fh6_kit::Signals`:** rpm and its limiter and shift point (bars, bracket, red zone), gear, speed (in the user's unit), throttle, brake, the race flag (boot), the body slip angle from the velocity (`Signals::slip_deg`), lateral G (the kit's smoothing; readout and light swing), the power unit.
- **From the packet:** `power_watts` (power gauge, in hp or kW) and `boost_psi` (boost gauge).
- Forza sends no fuel, coolant or oil temperature, oil pressure or trip distance, so the A70's windows for those carry the verified fields above in the same shapes (change B).

## GPU budget
Measured on the PC's RTX 3070 Ti Laptop GPU (median of three interleaved rounds with PorscheAnalog as the control; 0.5 ms per cluster at 4K is the budget):

| Build | Cockpit states | Hood states | Sequence p50 / p99 | Control (PorscheAnalog p50) |
|---|---|---|---|---|
| 7930b12 (first build after the direction check) | 0.66 to 0.74 ms | | 0.729 / 0.737 ms | |
| 1b3ce71 (ring cover, no bloom, limiter hold, new surround art) | 0.518 to 0.536 ms | 0.471 to 0.484 ms | 0.529 / 0.845 ms | 0.392 ms |
| 9447c0f (mesh zones; preview-export now also draws a logo in the zone, +2 draws) | 0.482 / 0.489 / 0.501 / 0.490 ms | 0.433 / 0.438 / 0.447 / 0.439 ms | 0.492 / 0.501 ms | 0.391 ms |
| a1fd67c (side-window zones on their segments) | 0.471 / 0.477 / 0.487 / 0.477 ms | 0.423 / 0.428 / 0.437 / 0.428 ms | 0.480 / 0.486 ms | 0.391 ms |

Every state is under 0.5 ms from a1fd67c (the redline cockpit state, the highest, at 0.487 ms). ecf889e (the rust review's fixes) changes no draw.

**Live bench (CPU share, directive 7):** the PC, 28 Nov 06:42 to 06:48 UTC, mock-udp `bmw_m4_cs` at 60 Hz, `FH6_WheelDash --demo --cluster <Name> --stats --bench-seconds 60`, A B A B against the scaffold dash LfaV10Nurburgring, built at a1fd67c. CPU per frame p50 / p99: RetroCyber 0.725 / 1.266 and 0.657 / 0.965 ms, scaffold 0.625 / 0.944 and 1.059 / 1.427 ms. The clean pair (no other build running in either run) gives a share of +0.032 ms, under 0.3 ms; the other pairings were loaded by other threads' builds (+0.100 and -0.402 ms) and are under it too. The dash's own draw part is 0.053 to 0.059 ms p50 (scaffold 0.031 to 0.050), 18.6 draw calls against 11.0. Receive to present stayed under one 300 Hz frame at p99 in every run. ecf889e only removes work from `draw` (cached mesh zones, hoisted colours), so the share at the close is at most this.

What the cuts were, from the per-layer timing at e003c20 (surround 0.221, glass 0.205, mesh and scanlines 0.128, speed bloom 0.094 ms): the surround's art, face, rims and lens black are drawn on twelve rectangles over its ring only (about 17% of the panel); the speed and gear glow with local halos and no frame blooms; the grid mesh and the scanlines go only where phosphor can light. Drawing the lens glass on its rim band alone was tried and dropped: the kit never trims a glass draw under a clip, so twelve clipped glass draws were twice as slow on lavapipe. The p99 spikes (single frames about 0.3 ms slower, seen only in variants with a glass layer) are the kit thread "Speed up the kit's glass layer"'s to explain.

## Direction check
Run 28 Nov 2026, 03:55 to 04:20 UTC, on lavapipe renders in the container and one cold-reviewer run on the PC. Decision: **go with changes**.

### Frozen expectations (written 03:20 UTC, before the first new render)
Rank order: 1 gear (white VFD seven-segment, centre window under the red zone) and the red-zone bracket shift light; 2 speed (three cyan seven-segment digits, ghost 888); 3 the horizontal rising tach bars; 4 pedal columns, slip bar, power and boost slants, G readout; 5 numerals, printed labels, lens, surround.

Feature inventory, reference → tolerance:
- F1 one long low panel, separate VFD windows on black behind one lens → windows visibly separate (at least 12 px of black between), one lens outline over all
- F2 order left to right: two-column bar window, centre (tach over speed), narrow vertical bar, two slanted bars → same order, no other windows
- F4 tach bars: thin upright bars, tops on a curve rising left to right, taller to the right → the right end's bar at least 3 times the left one's height; 4 bars per 1000 rpm
- F5 numerals 0 to N above the bars on the same curve, "TACH ×1000r/min" top left → each numeral within ±8 px of its bar's x; label present
- F6 red numerals in the red zone and the bracket over it; bars stay primary → red only on numerals from the red-zone thousand and on the bracket (tolerance change below)
- F7 speed the largest figure in the centre window below the tach, unit right, "SPEED" below → speed cap at least 2.5 times the numeral cap
- F8 small readout left of the speed → present, cap at most 0.4 times the speed cap (tolerance change below)
- F9 two columns of thick horizontal segments → 2 columns, at least 8 rows
- F10 one vertical bar with a slanted top in a narrow window → width at most half its height
- F11 two slanted parallelogram bars leaning right with scale figures → lean 10 to 25°, figures present
- F12 colours: bars green or cyan, digits cyan, labels near-white but dim, red zone red → hue within 20° of the palette roles
- F13 ghosts visible through the filter, lens reflections → unlit segments at 7% (±2%), lens rim crisp, no tint over black
- F15 boot lights the windows left to right → sequence frames 0 to 9 show only the settled state
- Key frames: the limiter at frame 29 or 30; strobe 30 to 41 (bracket only); gear change at 42.

### Tolerance changes (logged before re-checking)
- 04:05 UTC, F8: "cap at most 0.4 times the speed cap" became "at most 0.7 times and visibly smaller than the speed". Why: the glance gate needs every information zone at 4.5:1 after the blur, and the shared seg7 stroke is a fixed 10.5% of the cap, so the G readout measured 3.9 to 4.4:1 at cap 44 (cockpit) and 36 (hood). It is now cap 62 (0.48 of 128) and 66 (0.69 of 96).
- 05:12 UTC, F6: red is also on the printed red-zone line under the baseline and its limiter tick. Why: the cold reviewer's finding 5 and the frontal bench photo, whose red zone is a marked section of the scale, not numerals alone.

### Cold look (04:10 UTC, `02_mid_cruise_cockpit.png` at full frame, its mirror and its luminance view)
Not blind to the references (they were studied for `RESEARCH.md`) or to earlier renders of this build.
1. One wide, low rounded rectangle with a thin grey metal rim sits low in the centre of the frame, in the black scrim, below the windscreen; it spans about 37% of the frame width.
2. Inside it, behind a dark glass face, are separate black windows with faint outlines: a narrow one at the left, a large centre one, a narrow one and a wider one at the right.
3. The eye goes first to the white seven-segment "4" right of centre, then to the cyan "118" with ghost 8s, then to the cyan bar row that climbs from left to right across the top of the centre window.
4. The bar row is lit to just past the 5; the numerals 0 to 9 sit above it on the same rising curve, 8 and 9 in red; a dim red bracket with dark pods sits above 7 to 9.
5. Small readouts: "0.2 G" in yellow seven-segment left in the centre window, "MPH" right of the speed, "45°" over the narrow right window, "200 HP" and "20 psi" over the wide right window.
6. Left window: two columns of ten segments, the left lit green four high, the right dark red ghosts. Narrow right window: one column of dark yellow ghosts with a slanted top. Wide right window: two slanted yellow bars lit about half way.
7. Labels in grey caps under the windows: THR BRK, G-LOAD, SPEED, GEAR, SLIP, POWER, BOOST; "TACH ×1000r/min" and the logo letters at the top left of the centre window.
8. Colours: cyan bars and speed, white gear, yellow G and right-hand bars, green throttle, red numerals and bracket rail, grey labels, on black.
9. The lit segments carry a fine hexagonal mesh and read as lit phosphor; the rim reads as metal with a highlight on its top-left curve; the windows and labels read flat.
10. In the luminance view the gear and speed are the brightest marks, then the lit tach bars, then the G digits and the yellow bars; labels and numerals are mid grey.

Premortem (the likeliest reason TJX rejects it): "a neat 2D VFD mock-up in a thin frame, not the Supra's glass panel in a real binnacle: the surround is too thin to show its material at a glance, the windows are flat outlines, and the black right of the gear and over the tach's left end looks sparse."

Rejection patterns against named crops: (a) not like the real car: the layout follows the A70; the A70 has no gear and a longer panel (changes A and C), left to the cold reviewer. (b) materials only at 2×: the surround's grain reads only at 2×; at 1× it reads as a metal rim. (c) slab with blobs: one panel behind one lens is what the A70 has; no. (d) flat modern widgets: phosphor, ghosts and lens read as hardware; the window outlines were flat 1 px rims; partly. (e) murky glass: the lens interior measured black after the stretch fix; no.

### Measurements (03:55 to 04:20 UTC)
- glance, first run: gear 8.5 / 9.7 / 6.3 / 10.4:1 (redline no: magenta gear), G readout 3.9 to 4.2:1 (no). Fixed in be612fd and 7930b12 (the gear stays white; G cap 62 / 66); re-run all yes: cockpit gear 8.5 / 9.7 / 11.5 / 10.4:1, speed 8.3 / 9.3 / 9.3 / 8.7:1, G 5.1 / 5.0 / 5.1 / 4.7:1; hood gear 9.1 to 10.7:1, speed 7.5 to 8.1:1, G 4.6 to 5.1:1.
- flash, frames 30 to 41: one area (the bracket), 10 Hz, 3 on / 3 off, worst field 3,872 of 21,526 px² (yes), tallest 32 px (yes).
- motion, first run: stalls at frames 29, 47, 57 and 59 (the rpm moved less than one bar). Fixed in be612fd (the next bar dimmed by duty): no stalls.
- static: p95 at or below 0.2, dimmer than the live data: yes.
- stretch, first run: the whole lens face at 4/255 (a haze). Fixed in e003c20 (key light 30%, roughness 0.015, studio 1.2%): face 0/255, rims keep the highlight.
- before and after: the before's main failure, an octagon tube with chevrons that copies no real car, is gone. What it did better: the dot G-scope and the DRIFT and SLIP lamps; the slip bar keeps slip but loses the lamps (the A70 has none).

### Cold reviewer (one run on the PC, 04:13 UTC, seed 1928, logo zone masked; R1 a frontal left-hand-drive bench photo, R2 an angled right-hand-drive photo)
- Five-second test: first the white "4", second the teal "118", third the teal rev bar.
- Recognition: "Don't know." (V13 no; stays no.)
- Blind read against the key: gear 8 of 8, speed within 1 mph 8 of 8, rpm zone 8 of 8.
- R1: "Same" in both orders. R2: "Different" in both orders (that car's left-end turbo module and longer lower strip).

| # | Finding | Outcome |
|---|---|---|
| 1 | Flat tablet frame with an even grey rim, no hood, lens, depth or curve | fixed in e093d1b: the lens and surround follow the A70's stepped panel; the Blender surround follows the same outline (re-rendered 1b3ce71, checked at 1:1) |
| 2 | Rounded boxes round the windows read as UI cards | fixed in 126a98d: no window boxes; one printed divider line under the windows with a riser in each gap |
| 3 | Empty corners above the side columns | fixed in e093d1b: the side windows sit under the low strip, the centre window under the raised part |
| 4 | Shift strip small and thin | fixed in 126a98d: six pods, 30 px (cockpit) and 20 px (hood) tall |
| 5 | Red zone marked only by two numerals | fixed in 126a98d: a printed red line under the baseline and a limiter tick |
| 6 | Gear looks raised; its ghost is teal while lit is white | fixed in 126a98d and 1aa00e5: white ghost; matte off the limiter, a soft local glow on it |
| 7 | G digits look outlined | fixed in 126a98d: the readout is drawn in front of the mesh; glance 6.1 to 6.9:1 |
| 8 | G reads 0.1 at idle; boost has 2 bars lit at idle with no zero mark | boost: a zero tick and "0" (vacuum below it is the A70's own scale); G 0.1 is the review telemetry's idle lateral G |
| 9 | Power top figure differs between frames | by design: the power scale re-ranges to the car only with the throttle under 0.1 |
| 10 | Speed, gear and rpm disagree between frames | the review telemetry (sequence against state data), not the dash |
| 11 | BOOST touches the rim | fixed in 126a98d: POWER left-aligned, BOOST right-aligned |
| 12 | Grey labels small and mid grey | the static cap (V11, 20% luminance); labels raised to cap 18 (cockpit) and 16 (hood) |

### DC answers
V1 yes, V2 yes, V3 yes (findings above), V4 yes (the gear first), V6 yes after the fixes, V10 yes, V13 no (stays no), V14 yes for layout, geometry and colour, V15 yes against R1, no against R2 (another car's panel), V16 NOT ASSESSED at the direction check (the board came later, out of order: Penpot was held by other threads), V17 partly (the surround's material at 1× only after the stepped re-render), V19 yes after e003c20, V20 yes after e003c20.

## Visual review
Closing round on 28 Nov, on cloud renders (lavapipe) of a1fd67c, ecf889e and f5a33f2; the PC rendered the cold reviewer's packet at 43f7d8a. Working notes are in `mockups/RetroCyber/review/direction_check.md` and `expectations.md` (gitignored).

### Closing cold look (written before the notes, the spec, the board or the references were opened)
A long low black panel sits in the steering-column gap; its outline steps up in the middle into a raised strip with rounded shoulders, inside a thin graphite ring with a light top edge. The eye goes to the white "4", then the teal "118", then the teal tach bars rising left to right, then the amber "0.2 G", the yellow slants and the green column. At 1:1 the lit segments carry a fine dot mesh and a soft glow and read as lit phosphor; the ring reads as a rounded dark metal edge; the labels, the divider and its risers read as flat print. Wide true black sits above the tach's left half and above the slip and power windows. In the luminance view the gear and the speed are brightest, then the lit bars, then the G digits and the slants. Premortem: "a clean 2D display in a thin ring, with the shift pods looking added on." Rejection patterns: (a) the cold reviewer's question; (b) no (the mesh is VFD detail at 1:1; the ring shows a highlight at 1x); (c) no (one panel behind one lens, as the A70; no window boxes); (d) partly (the unlit pods read as dark tinted boxes: their ghost state, kept); (e) no (the lens face is 0/255).

### Cold reviewer (one run on the PC at 07:00 UTC, seed 1929, packet rendered at 43f7d8a, logo zone masked; R1 the frontal bench photo, R2 the angled right-hand-drive photo)
- Five-second test: the white "4" first, the teal "118 MPH" second, the tach with the red 8 and 9 third.
- Recognition: "Don't know."
- Blind read against `blind_key_1929`: gear right 8 of 8, speed within 1 mph 8 of 8, shift lights as the frame shows them 8 of 8 (checked on 1:1 crops), rpm zone right on 5 of 8 and not given on 3 (frames 28 and 29 at the limiter, "high or limiter, don't know"; frame 17 at 51%, "right on the low/mid line"). It took the scale's end (9) as the limiter, because the limiter tick was too small to find at glance size.
- R1 and R2: "Different" in both orders. Shared with R1: the TACH legend over a tach across the top with red at the high end, the large speed under it with its unit and SPEED label, a small labelled readout left of the speed, a vertical bar at the far left and one right of centre, two slanted bars with a zero on the right, labels along a bottom line. Different: the gear digit and the shift strip (changes C and D), the straight rising tach against R1's curved arc (change A's proportions), the closed stepped housing against R1's longer strip.

| # | Finding | Outcome |
|---|---|---|
| 1 | The shift lights look dark at the top of the rev range (frames 28 and 29) | fixed in f5a33f2: the strobe was on the wall clock and opened with its off phase as the limiter arrived; it is now timed from the limiter, so frames 27 to 29 are lit, then 3 off and 3 on (flash audit below) |
| 2 | The shift strip is small, crowded on the 7 and 8, with a red outline only on its top and left | not fixed: six pods at 30 px (changed after the direction check); the rail and its turned-down ends are the A70 bracket's print (the right end sits behind the 9) |
| 3 | No colour change in the red zone on the bars | not fixed: the A70's bars stay one colour; the red zone is print (F6) |
| 4 | The redline mark is tiny | fixed in f5a33f2: the red-zone line at 5 px and the limiter tick at 6 px, half as tall again |
| 5 | The gear (white, matte) and the speed (teal, glowing) look like two displays | not fixed: deliberate rank, the gear is the hero and white for 7:1 (DC fix) |
| 6 | The speed's glow softens its digits | not fixed: the phosphor halo; the speed read on every glance |
| 7 | The boost "0–" mark is unclear, and boost shows three lit segments at idle | not fixed: the vacuum part of the A70's turbo scale, under its zero tick |
| 8 | Small grey labels | not fixed: print at the static cap (V11); labels at cap 18 and 16 |
| 9 | The housing is lopsided and its frame plain | not fixed: the raised part sits over the tach, as on the A70 panel; the ring's material reads at 1:1 |
| 10 | G-LOAD reads 0.1 at rest | the review telemetry's 0.12 g lateral input, not the dash |

### V1 to V26
| Line | Answer | Evidence |
|---|---|---|
| V1 | yes | closing expectations appended at 06:38 UTC before the closing cold look (renders of a1fd67c had been opened for pixel diffs and the 1:1 surround check, said in the note) |
| V2 | yes | the closing cold look above, with patterns a to e against named crops |
| V3 | yes | one cold-reviewer run, each finding recorded above as fixed or not fixed |
| V4 | yes | the gear first, the speed second |
| V5 | no, stays no | gear, speed and lights right on 8 of 8; the rpm zone not given on 3 of 8. Fixed in the dash where it can be: the limiter tick and red-zone line made bolder (f5a33f2) |
| V6 | yes | glance after the blur: cockpit gear 8.7 / 9.9 / 11.6 / 10.7:1, speed 8.4 / 9.3 / 9.3 / 8.7:1, G 6.6 / 6.6 / 6.6 / 6.1:1; hood gear 9.1 to 10.7:1, speed 7.5 to 8.1:1, G 4.6 to 5.1:1 |
| V7 | yes | squint views per state: bracket dark, partly lit green and amber, all magenta (on phase), and dark (off phase) are each distinct |
| V8 | yes | dynamic readouts: gear cap 190 / 150, speed 128 / 96, G 62 / 66 (cockpit / hood), all at least 17.7 and the gear at least 40 and 36; the scale figures (cap 15 / 13) are print that re-ranges only per car, like a dial's numerals |
| V9 | yes | the limiter-hold flash map marks only the bracket; the tach tip holds (LimiterHold) |
| V10 | yes | glance to squint: labels and ring fade first, then the slants and pedals, then the tach, gear and speed last |
| V11 | yes | `static` over 4 states and the sequence: static p95 0.157, live p95 0.646. Areas over the cap are live readouts the review data never changes: the first tach bars lit at idle rpm, the G readout's "0." (lateral G held at 0.12 g), and boost's vacuum segments |
| V12 | yes | product page badges: red 0 to 1%, white 1%; red only on the red-zone print, the brake column and the reverse gear; in protanopia and deuteranopia the red zone reads by position (the right end, the line and tick), throttle and brake by column and label, shift stages by count |
| V13 | no, stays no | "Don't know." (the direction check's answer was the same) |
| V14 | yes | F1 windows 20 / 20 / 18 px apart (cockpit) and 20 px (hood), stepped outline; F2 order by test; F4 4 bars per 1000 rpm, last bar 6.5x (cockpit) and 4.7x (hood) the first; F5 numerals at their bars' x; F6 red from the shift thousand plus the bracket, line and tick (logged change); F7 speed 4.9x and 4.4x the numeral cap; F8 G 0.48x and 0.69x the speed (logged change); F9 2 columns of 10; F10 slip window 0.26 and 0.35 of its height; F11 lean 12 deg with figures; F12 hues as at the direction check (palette unchanged); F13 ghosts 7%, lens face 0/255; F15 the boot runs only on the first race start, idle frames settled |
| V15 | no | the cold reviewer's blurred-mass comparison gave "Different" against R1 and R2 in both orders this round (the direction check gave "Same" against R1); the differences it names are the logged changes A, C and D. The reference photos stay on the PC, so this thread's own blurred sheet was not made |
| V16 | yes | `mockups/RetroCyber/review/board_vs_render.png` (the board's own export of cockpit 02 against the render): every zone in the same place at the same size (the speed's lit pixels start at x 1440, y 1685 in both); the render adds the graphite ring and its highlight, the phosphor mesh and glow, the ghosts and the lens, so it reads clearly richer than the flat board and the before render |
| V17 | yes | full frame: graphite ring with one highlight, clear lens over black, lit phosphor with mesh |
| V18 | yes | the materials checks that apply (lens black level, rim highlight, phosphor plate, surround at 1:1 with no seams); dropped: metal dials, needles, glass drops |
| V19 | yes | one OpticalGlass lens over the whole panel: clear over black (0/255), rims crisp, its key light swings with lateral G between 02 and 04, never over other glass |
| V20 | yes | `stretch`: 0 px not off outside the envelope plus 40 in every state; frame corners off |
| V21 | yes | gear changes on frame 42, the strobe opens on frame 27 where the limiter is reached, bars switch whole with the next one at its duty, `motion` 0 stalls |
| V22 | yes | `flash --frames 30-41` at f5a33f2: one area (the bracket), 10 Hz, 3 on / 3 off, worst field 4,840 of 21,526 px2, tallest 38 px; 3 to 4.3 Hz and the one-second count NOT ASSESSED (12 frames) |
| V23 | yes | frames 39 to 50 at 1:1: "3" to "4" on frame 42, no blank or doubled frame, no morph (VFD switch) |
| V24 | yes | `motion`: one spike at frame 10, the first pull frame after idle (rpm and speed jump in the data) |
| V25 | yes | the before's octagon tube that copied no car is gone; the before's DRIFT/SLIP lamps are not kept (the A70 has none; slip stays as a bar) |
| V26 | yes | the 8 goldens opened and checked at 676d428 and f5a33f2 (the second changes only the red-zone line) |

Penpot board (28 Nov, 07:15 to 07:35 UTC): file `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, page `07 · RetroCyber` `6fe75c5e-bb22-8084-8008-fd924684ddca`. Boards (3840x2160): cockpit 01 idle `347e5ae2-9e0b-80b1-8009-03889f3834ca`, 02 mid cruise `347e5ae2-9e0b-80b1-8009-038896a7afaf`, 03 redline strobe `347e5ae2-9e0b-80b1-8009-03889ff4af02`, 04 drift braking `347e5ae2-9e0b-80b1-8009-0388a0e0841c`; hood 01 `347e5ae2-9e0b-80b1-8009-0388a19f0a16`, 02 `347e5ae2-9e0b-80b1-8009-0388a8b4027a`, 03 `347e5ae2-9e0b-80b1-8009-0388a9a8f9e2`, 04 `347e5ae2-9e0b-80b1-8009-0388aaa292ab`; motion notes (1920x1080, editable text) `347e5ae2-9e0b-80b1-8009-03890a6cdf00`. The first design's eight boards are kept on the page as "superseded · ..." below the new grid.

Lines that could not be fixed afterwards: V5 (the zone on 3 of 8 glances), V13 (the car not named), V15 (different arrangement in both orders, from logged changes).

## Before and after pair
- Before: `mockups/_before/RetroCyber/02_mid_cruise.png` (the octagon tube on `main`); copied to `docs/handover-shots/` at the close.
- After: the final render at the close.

## Assets and masters
Runtime files (in git) with recipe, seed and licence: [assets/clusters/retro_cyber/ASSETS.md](../../../../../assets/clusters/retro_cyber/ASSETS.md).
- `phosphor_grain.png`: ComfyUI (SDXL base + 4x-UltraSharp), seed 1906, the tinted fill of every lit bar, pod and segment.
- `surround_{cockpit,hood}_{l160,l110}.png`: Blender, seed 1907, over a ComfyUI graphite grain (seed 2708); two key lights per view, cross-faded by lateral G. Built on the cloud 4090 for the first outline (28 Nov 03:45, $0.097) and on the PC's own card for the stepped outline (28 Nov 05:27, TJX chose "local": no cost).
- Masters: the Drive folder `FH6 WheelDash v2 - 4K Masters/retro_cyber/` (the asset build's `--masters` copy, 7 files per view).

## Changes from the real display and from the brief
From `RESEARCH.md`, "Deliberate changes and why":
- A. The panel is taller than the A70's 6:1 in the cockpit view, to fill the steering-column gap at a readable size.
- B. The fuel, temperature, oil and trip windows carry pedals, slip, power and lateral G (no such data in the telemetry).
- C. A gear window under the red zone (the manual A70 has none; the gear is rank 1 here).
- D. The bracket over the red zone is the shift light: six pods that fill from the shift point and strobe on the limiter.
- E. Magenta for the limiter (the family's shift colour).
- F. The 2027 step: phosphor grain, grid mesh and filaments over lit phosphor, a Blender graphite surround, the kit's optical lens; whole bars with the next one dimmed by duty; a limiter hold; halos without bloom.
- G. The windows have no drawn edges; a printed divider line under them with a riser in each gap.

## Shared places touched
- `crates/fh6-clusters/src/common/seg7.rs`: `Emission::Halo` and `Emission::SoftHalo` (additive; no existing caller changes) with a test.
- `docs/handover-shots/cluster-retro_cyber-*.png` (at the close).
- Not touched: `CATALOG.md` (main's copy kept on every merge), `fh6-kit`, the shared asset ledgers.

## Requests for shared code
1. **Glass under a clip** (now the kit thread "Speed up the kit's glass layer"): the painter never trims a glass draw, so a dash cannot limit its lens to the rim band where all of its visible effect is; and the glass pass shows single-frame spikes of about 0.3 ms.
2. **Emissive target only when something blooms:** every fill writes both the scene and the emissive target; a frame with no bloom could skip the second.
3. **`common::seg7::draw_scanlines` phase:** the lines start at each rectangle's own top pixel. A dash that draws scanlines over several rectangles has to align them itself (`mod.rs` does, with a copy of the private period rule); an origin argument would remove that copy.
4. **asset-forge:** the surround rows' arguments were positional and landed in the wrong columns (fixed in c24f52f); named arguments in every recipe's ledger rows would prevent it elsewhere.

## What sibling dashes must know
- **A full-panel fill costs about 0.05 ms at 4K** on the PC card (two Rgba16Float targets per fill). Draw big surrounds on a ring cover and multiply layers only where there is light.
- **Glass is never trimmed by a clip**: clipping a glass shape into pieces multiplies its cost.
- **The glass's flat interior is not free of effect**: it passes about 91% of what is under it (Fresnel and absorption), so removing glass from an area brightens it.
- **A VFD tach on the limiter flickers at 30 Hz** if the next bar's duty follows the bouncing rpm: hold the peak (`LimiterHold`).
- **Penpot swaps lettering for a stand-in serif**: outline the text from `assets/fonts/` before importing an SVG board; the board generator reads the dash's own `Geom` (dumped by a local test), so the board and the code cannot drift.

## Review findings
- design:design-critique (once): one colour role per data family, states distinct at a glance; the hood view's G readout is nearly as loud as the speed at 0.39x (within the logged F8 tolerance, kept); the empty black over the tach's left end is the A70's panel shape. No changes.
- design:accessibility-review (once, limited to 1.4.3, 1.4.11, 1.4.1, 2.3.1 and undriven motion): contrast yes, labels about 7:1, use of colour yes, flashing yes, reduced-flash mode from the code (`ShiftLights` holds the pods lit with a 2 Hz breath; RetroCyber's own strobe defers to it), no undriven motion. No changes.
- rust-review (once, a fresh-context agent): fixed in ecf889e: the power scale never grew past its first value (now the pull's peak per car, re-ranged on the lift, with a test), the scale figure truncated at 1000 or more, mesh zones rebuilt every frame (now cached per view and scale), per-cell colour maths (hoisted), the halo reach copied token constants (now seg7's glow scale, with a token test). Not fixed: the SDXL cache fingerprint leaves out steps and cfg (changing it would invalidate the committed plate's key; a follow-up for the recipe), the recipe's hand-copied geometry (asset-forge cannot depend on fh6-clusters; the 1:1 surround check confirms they agree), the display font revert (a pattern shared with LiquidGlass and Gt7HoodPro; a request for shared code), the boot self-test's per-sector redraw (0.47 s once per launch), pedantic clippy nits.
- cold-reviewer (once): see "Visual review".
