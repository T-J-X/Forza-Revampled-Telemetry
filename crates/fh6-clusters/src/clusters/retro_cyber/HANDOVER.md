# HANDOVER: Dash · RetroCyber (catalog row 19)

**Thread / branch:** "Rebuild RetroCyber as the Supra A70 dash" (slot D), `claude/project-thread-191tnq`, draft PR #9  **Date:** 2026-11-28  **Result:** DRAFT. Written while the build is finished; the closing review, the goldens, the live bench and the Penpot import are still to come, and each section marked *pending* is filled in at the close.

Findings and yes/no answers only; there are no scores in this file. TJX is hands-off (AGENTS.md rule 12): the self-review stands in for TJX's direction check and sign-off. TJX's own words in the thread are quoted where they decided something.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| The introduction was posted, and the thread is not archived | yes | the thread's first message; the thread is open |
| `RESEARCH.md` is written, with sources | yes | [RESEARCH.md](RESEARCH.md): the A70 Panoramic Digital Meter, its features with sources, the match table, deliberate changes A to G, the gaps |
| The design board is the Penpot page `07 · RetroCyber`, 4 states per view, its IDs here and in the README | *pending* | the eight state boards and the motion notes are drawn from `Geom` (outlined text); the import waits for the Penpot queue |
| A bespoke ComfyUI asset and a bespoke Blender asset are used, with ledger rows and 4K masters on Drive; the before and after pair is here | yes | [assets/clusters/retro_cyber/ASSETS.md](../../../../../assets/clusters/retro_cyber/ASSETS.md); "Assets and masters"; "Before and after pair" |
| The module is complete, with no TODOs, and goldens pass | *pending* | 15 unit tests pass; goldens are a PC job |
| 4K renders, the 60-frame sequence and the product page are in `mockups/RetroCyber/` | *pending* | the final `preview-export` run |
| The live bench report and the GPU time under 0.5 ms are recorded | *pending* | "GPU budget"; the re-time after the mesh zones runs on the PC |
| The direction check was posted and self-reviewed, with its findings here | yes | "Direction check" (posted 28 Nov 04:20 UTC) |
| Every line of the closing review is yes, or a line that could not be fixed afterwards is reported | *pending* | "Closing review" |
| The visual review (V1 to V26) is here, each line with its evidence | *pending* | "Visual review" |
| The handover is written, the job is reported for the merge | *pending* | this file |

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
| (next) side-window zones on their segments | *pending* | *pending* | *pending* | *pending* |

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
*Pending: the closing round.* Re-checked on the cloud render at 9447c0f with the new surround art: glance all yes (cockpit gear 8.7 / 9.9 / 11.6 / 10.7:1, speed 8.4 / 9.3 / 9.3 / 8.7:1, G 6.6 / 6.6 / 6.6 / 6.1:1); static yes; stretch 0 px not off outside the envelope; flash, frames 30 to 41: one area, 10 Hz, 3 on / 3 off, worst field 4,834 of 21,526 px², tallest 38 px (yes); motion: no stalls.

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
*Pending: the closing review's skills and agents.*
