# HANDOVER: Dash · Gt7HoodPro (catalog row 3)

**Thread / branch:** "Rebuild Gt7HoodPro dash", `claude/project-thread-h7qahw`, draft pull request #10  **Date:** 2026-11-28  **Result:** ◐ at work (this file is filled in as each step is done)

This replaces the handover of the 2026-11-23 build (the gull-wing ribbons; that file is in the git history). The dash is rebuilt from the ground up as Gran Turismo 7's Bonnet View meter. Findings and yes/no answers only; there are no scores in this file.

## What was built
- **Research:** [RESEARCH.md](RESEARCH.md): what makes GT7's Bonnet View meter recognisable (17 points with sources), how the dash matches each one, the deliberate changes and the gaps.
- **Dash:** `mod.rs` (the `Cluster`, the rev bar, the suggested-gear tile, the readouts), `geom.rs` (the zone table `HOOD` and the fit into another envelope) and `hardware.rs` (the sprites, the light's cross-fade weights, the procedural fallbacks and the lamp icons).
- **Asset recipe:** `tools/asset-forge/src/recipes/gt7_hood_pro.rs`: a ComfyUI forged-carbon texture, then the Blender plate and the suggested-gear tile, each at the virtual light's rest azimuth and its ±25° swing.
- **Guard against drift:** the test `the_recipe_renders_the_same_band` in `geom.rs` reads the recipe's constants and fails when the recipe and the zone table disagree.
- **Matrix:** the dash's row in `docs/TELEMETRY_USAGE_MATRIX.md`.

## Telemetry
Through `fh6_kit::Signals`, all covered by the golden packets or derived from them:
- **Read as they are:** rpm (current and limiter), gear, speed, throttle, brake, steer and boost.
- **Derived:** the shift point from the limiter rpm and the tokens; lateral and longitudinal G from acceleration (the tyre loads by weight transfer, and the virtual light); body slip from velocity (the slide lamp); the trip from the speed.
- **Not read:** tyre heat and wear, fuel, assists, the handbrake, the clutch and the driving-line suggestion. GT7 shows them; the verified telemetry does not carry them, so each slot is dropped or shows the nearest verified quantity (RESEARCH.md, "Deliberate changes").

The matrix row changed with the rebuild. Added: `throttle`, `brake`, `steer`, `g_lat`, `g_long`, `velocity` and `boost`, each read on purpose (pedal bars, steering dot, tyre loads and the light, the slide lamp, the boost gauge). `race_on` is not listed: the dash draws the same with it on or off, and only the trip stops counting while the race is paused.

## Direction check

The first build with the plate (commit `c7a27a2`), reviewed once with `fh6-visual-review` steps 1 to 6 and `design:design-critique`. Renders: lavapipe in the cloud container for the review set, and the PC's RTX 3070 Ti for the GPU time (0.162 to 0.220 ms per state, sequence p50 0.214 ms and p99 0.218 ms at 4K).

### Frozen expectations (written at 04:34Z on 2026-11-28, before any render with the Blender plate existed; the only renders seen until then drew the procedural fallback)

- **Rank order:** 1 the gear, the lit LED run with its strobe, and the suggested-gear tile; 2 the speed; 3 the rev bar as a tach (where the head LED is); 4 the side groups (pedals, steering dot, tyre lenses, boost, trip, telltales); 5 the unit and label, the divider, the logo and the hardware's rims.
- **Fade order in the blur ladder:** the labels, divider, logo and rims go first; then the side groups; then the body of the LED run; the speed; the gear, the alert run and the tile last.
- **Feature inventory** (RESEARCH.md points 1 to 14; frame coordinates at 4K, the hood anchor is (1920, 1760)), each with its tolerance:
  - F1 One low band and no panel: at least nine separate parts with true black between them; `probe` of the gaps reads 100% off.
  - F2 Rev bar: a straight bar of dense ticks centred on x 1920 within 2 px; GT7's is about 800 px wide with about 100 ticks; ours is 788 px with 95 lenses (width within 40 px, tick count within 10).
  - F3 Lit LEDs run from the left end to the revs: 9 or 10 lit in `01_idle`, 56 ± 1 in `02_mid_cruise`, 94 or 95 in `03_redline_strobe`, 77 ± 1 in `04_drift_braking`.
  - F4 Speed, divider, gear: the speed ends 26 px left of x 1920 (within 4 px), the divider is on x 1920 (within 1 px), the gear starts 30 px right of it (within 4 px); the speed's cap is 58 px (GT7 about 54, within 6) and the gear's 88 px (GT7 about 80, within 10); the unit sits under the speed.
  - F5 Suggested gear: a red rounded tile right of the gear, 94 × 86 px (GT7 about 86 × 70; width within 12), showing in `03_redline_strobe` only among the four states, with the next gear (6) in it.
  - F6 Steering dot: above the rev bar, within 340 px of x 1920, off-centre in `04_drift_braking`.
  - F7 Pedal bars: brake left of the rev bar and throttle right of it, white fills; the brake fill shows in `04_drift_braking`.
  - F8 Tyre diagram: the leftmost group, a car seen from above with four lenses at its corners.
  - F9 Round gauge with a box under it: an arc scale about 120 px across (GT7 about 164: a deliberate change, the band is narrower) with a box 200 px wide under it (GT7 about 190, within 20) holding five digits and tenths.
  - F10 Telltales: one lamp each side of the rev bar (GT7 has a 2 × 2 grid each side: a deliberate change), 48 to 58 px across (GT7 about 60).
  - F11 Colour: digits and lit LEDs are the ink (white) below the alert; in `03_redline_strobe` the whole lit run is one colour, the palette's `shift`, with no white LED left in it; the tile's face is red (hue within 12° of the `warning` role's); nothing else is red in `01`, `02` and `04` except a lit limiter lamp.
  - F12 Squared digits: thin and wide for the speed, bold for the gear.
- **Materials at glance scale** (the full 4K hood frame, no zoom): the light bar reads as black glass with one highlight line from the key light, and its unlit lenses as separate cells; the plinth, slot bases and rings as dark chrome whose chamfers glint; the car chip as forged carbon, with flakes; the boost scale as polished metal; the tile as a raised red enamel key. The highlights sit differently in `04_drift_braking` (the light has swung) than in `02_mid_cruise`.
- **Key frames of the sequence** (limiter 8700 rpm, shift point 97% = 8439 rpm, alert from 93% of that = 7848 rpm): the run is white up to frame 24 and takes the shift colour from frame 25 (7985 rpm); the strobe and the tile start at frame 27 (8492 rpm) and hold through 41 at 3 frames on and 3 off, starting lit; no digit changes from 30 to 41; the gear changes at 42, where the tile brightens for that one frame and folds away within 6 frames; from 43 the run is white again and has fallen back by frame 45.
- **Static parts:** at or under 0.20 relative luminance (p95), the polished boost scale included.
- **Budget:** under 0.5 ms of GPU time at 4K with the three renders cross-faded.
- **Blur levels:** the tool's defaults (the 960 × 540 blur gate at sigma 1.5, and its squint view).
- **TJX's recorded rejections to check against a named crop:** (a) it looks nothing like the real display; (b) faint textures on vector shapes, or materials that show only when zoomed; (c) one slab with blobs stuck on, or a backing panel the real display lacks; (d) flat modern widgets, not a 2027 instrument; (e) murky, tinted or frosted glass.
- **Known in advance to be NOT ASSESSED:** V19 (the dash has no clear glass piece: its black glass is opaque hardware); the cockpit composite (a hood-only dash has none, so the hood frame and its road composite stand in).

### Cold look (written 05:07Z on 2026-11-28 from `hood_02_mid_cruise.png` at full frame, its mirror `review/dc_02_mirrored.png` and `review/hood_02_mid_cruise_luminance.png`, before the notes, the board or the reference were opened)
1. One low band sits at the bottom centre of a black frame, a little under half the frame's width; nothing else is drawn.
2. The eye goes to the white lit run in the horizontal bar and the white "4" first, then to "118", then to the cyan dot above the bar, then to the left group (a car shape, an arc, "BOOST", "00000.0"), then to the white fill in the right vertical slot, and last to the letters "TJX" at the right end.
3. Values read: 118 mph, gear 4, a lit run ending a little right of the bar's middle, trip 00000.0, the right slot filled about two fifths from the bottom, the left slot empty, a short bright stroke near the arc's top left.
4. Colours: white and greys only, with one cyan dot. No red.
5. Parts that look lit: the bar's rounded frame has a pale outline and a soft white glow round its lit half; the car shape shows a mottled pattern of grey flakes; the arc has a bright rim.
6. Parts that look flat: the bar's face and its unlit half are an even dark grey with faint cells; the two slots are dark grey bars; the two round lamps are grey discs with a pictogram; the trip box is a mid-grey box with a pale outline; the four tyre pills are pale grey rounded rectangles.
7. The left group is dense and the right end holds only "TJX", with a gap between it and the right lamp.
8. In the mirror the weight sits on the tyre side. The speed and gear straddle the centre line, so the block of digits hangs left of the bar's middle in 02 (three speed digits against one gear digit).
9. In the luminance view the lit run, the gear and the speed are the brightest; the tyre pills and trip digits come next and are brighter than the labels; the cyan dot is a mid grey.

**The failure first.** If TJX rejected this render, the likeliest reason is that the hardware is dull grey: at glance size the bar, slots, lamps, trip box and tyre pills read as flat grey widgets, and only the carbon chip shows a material.

**TJX's recorded rejections against named crops** (`review/hood_02_mid_cruise_tile_r0c0.png`, `_r0c1.png`, `review/dc_states.png`, `review/dc_board_before_after.png`):
- (a) Nothing like the real display: no. The parts and their order match the reference strip (the cold reviewer said "the same layout" in both orders).
- (b) Materials that show only when zoomed: **yes, this applies.** At glance size only the carbon chip and the boost arc show a material; the light bar's face is an even dark grey (about #333) with no reflection, and the chamfers' highlights are 1 to 2 px.
- (c) One slab or a backing panel: no. Nine separate parts with true black between them.
- (d) Flat modern widgets: **yes, this applies** to the slots, the lamps, the trip box and the tyre pills, which read as grey rounded rectangles.
- (e) Murky or frosted glass: **yes in part.** The black glass reads as lifted dark grey, where piano black should be near #000000 with a crisp reflection band (materials T1).

### Cold reviewer (one `cold-reviewer` run at 05:08Z; packet `mockups/blind/31/`: A.png, glance_01 to 08 at seed 31, R1.jpg; logo zone masked)
- **Five-second test:** it noticed "the lit white part of the horizontal bar" first, "the '118' and the big '4'" second and "the cyan dot above the bar" third, and said the display is "a driving instrument cluster: engine revs as a bar, road speed in mph, the current gear, with boost, tyres and distance as side information".
- **Recognition, unprompted:** "Don't know."
- **Blind glance read** against the key (`review/blind_key_31.txt`): gear right on 8 of 8 frames, rpm zone right on 8 of 8 (on frame 45, 74%, it answered "mid, right at the edge of high"), speed within 1 on 8 of 8. It read the alert as "all lit (whole bar turned violet, red box, right icon red)" and found no separate row of shift lights.
- **Layout against the reference:** "the same layout, with differences in detail" in both orders. The differences it listed: one icon a side against GT7's four; a white fill and cyan dot against GT7's pink-red fill and red dot; our bar takes a larger share of the width; our speed digits are nearly as tall as the gear.

| # | Its finding | What was done |
|---|---|---|
| 1 | The car shape "reads as a textured pill, not a car" | Open: a car outline for the chip (nose, cabin glass, tail) in the next plate |
| 2 | Tyre blocks are "four identical flat grey rounded rectangles with no state shown" | Open: domed lenses in the plate, and the load drawn as light inside the lens |
| 3 | The odometer "00000.0" looks like a placeholder; its point floats; its digits are a third type style | Open: no leading zeros, a narrow cell for the point |
| 4 | The boost arc has no ticks or numbers, and its label is dim | Open: engraved ticks in the plate; the label is a static label (rule question, V6) |
| 5 | Both round icons are a dark symbol on a dark disc | Open: a domed lens with a clearer pictogram; off stays dim on purpose (as GT7's unlit telltales) |
| 6 | The two vertical bars have no label, and the empty one "looks like an unused slot" | Open: BRK and THR labels under the slots |
| 7 | The rev bar has no numbers and no marked red zone; the unlit ticks are too dark to show the bar's length | Open: a lens highlight on every unlit LED, thousand marks and a marked alert zone |
| 8 | The cyan dot is "a single isolated dot with no evident meaning" | Open: a thin rail with a centre notch and end stops under the dot |
| 9 | A grey haze lifts the black round the lit run | Open: a tighter bloom |
| 10 | The left group holds five parts and the right two; the right fifth is empty | Not a fault of the dash: the logo zone at the right end was masked for the reviewer |
| 11 | The violet alert fill "looks dimmer than the white fill", so the limiter state stands out less | Open: a hot core in each alert LED |
| 12 | "3" and the red-boxed "4" can be read as "34" | Open: an up chevron over a smaller digit in the tile; GT7 shows the same pairing |
| 13 | Frame 43 shows "a red square with no digit in it", frame 45 "a narrow red sliver" | Open: the hand-over becomes a short fade towards the gear, with no squeezed box |
| 14 | In neutral the "0" sits against the divider while "mph" stays further left | Open: the unit right-aligned to the divider, under the last digit |
| 15 | The "N" looks greyer and thinner than the numeric gears | Open: neutral at full ink |
| 16 | At glance size the odometer, the label, the icons and the carbon cannot be read | Expected for ranks 4 and 5; the materials pass answers the carbon |

### Measurements (05:06Z to 05:12Z; `review/dc_measure.txt`)
- **Glance gate** (hood states 01 to 04): gear 11.3 / 16.1 / 14.3 / 15.0 :1 (needs 7); speed 8.0 / 8.9 / 8.7 / 8.6 :1 (needs 4.5); rev bar 4.9 / 7.8 / 3.4 / 7.5 :1 (a graphic, needs 3); the tile in 03 11.3:1; trip 4.5:1 in all four. The unit (1.8 to 2.3:1) and the BOOST label (2.0:1) are static labels held under the 0.20 cap, and fall under 4.5:1 in the 960 x 540 blur: the rule question the pilot and PorscheAnalog reported.
- **Flash, frames 30 to 41:** one area, the LED run (758 x 26 px), 10.0 Hz, 3 on / 3 off; the worst 10-degree field holds 6474 of 21526 px2: yes; tallest block 26 px against the 42 px strip: yes; red flashes 0. Frames 27 to 41 add 3 px on the tile's edge while it pops up (27 to 29).
- **Motion:** 0 stalls; 3 spikes, each an intended event (frame 10 the pull starts, 25 the alert colour, 27 the strobe and the tile).
- **Static:** p95 0.355, over the cap. Areas over it: the trip digits (0.670; live, but the review set covers one second, so they never change), the boost scale's polished rim (0.352, max 0.836: to fix in the plate), the steering dot (0.545; live, the sequence holds the steer at 0).
- **Stretch:** 0 px not off outside the hood envelope plus 40 px in all four states; the frame's corners are off.
- **Diff against the before render:** the whole envelope changes (a new design).

### Feature inventory, measured (layout, geometry and colour lines)
| # | Frozen | Measured | Result |
|---|---|---|---|
| F1 | at least nine separate parts, black between | chip and lenses, arc, trip, two lamps, two slots, bar, readout, tile, logo; gaps probe as off | matched |
| F2 | bar centred on x 1920 within 2 px; 95 lenses | centre 1920; 95 cells at an 8 px pitch (plate), 95 lit runs counted in 03 | matched |
| F3 | lit LEDs 9 or 10 / 56 ± 1 / 94 or 95 / 77 ± 1 | 10 / 57 / 95 / 78 | matched |
| F4 | speed ends 26 px left of 1920 (± 4); divider on 1920; gear starts 30 px right (± 4); caps 58 and 88 | speed ink ends at 1888 (32 px left); divider 1919 to 1920; gear ink starts at 1951 ("4") to 1957 (31 to 37 px right); caps 60 and 88 | the caps and divider match; the two gaps are 2 to 3 px outside the frozen tolerance, because each digit sits centred in its tabular cell (logged, not re-frozen) |
| F5 | red tile right of the gear, 94 x 86, only in 03, showing 6 | red face 88 x 81 at x 2087 to 2174 (the base rim makes 94 x 86); "6"; 0 red pixels in 01, 02 and 04 | matched |
| F6 | steering dot above the bar, off-centre in 04 | on x 1920 in all four states: the review telemetry carries steer 0 (the code audit, NS-5) | no: the travel is covered by the unit test `the_steering_dot_spans_its_travel`; a steer value in `04_drift_braking` would change the shared review telemetry and every dash's goldens, so it is a request for the coordinator |
| F7 | brake left, throttle right, white fills; brake fill in 04 | as frozen (`review/dc_states.png`) | matched |
| F8 | tyre diagram leftmost | leftmost | matched in place; its car shape is finding 1 |
| F9 | arc about 120 px across, 200 px box under it | arc 124 px, box 207 px | matched |
| F10 | one lamp a side, 48 to 58 px | 57 px | matched (a deliberate change from GT7's grids) |
| F11 | ink below the alert; whole run `shift` in 03; tile red; no other red in 01, 02, 04 | lit LEDs #EDEFF3 in 01, 02, 04; #A89DFF on all 95 in 03; tile face #FF575F; 0 red pixels elsewhere | matched |
| F12 | squared digits, thin wide speed, bold gear | Michroma and Orbitron 600 | matched |

Materials at glance scale (frozen): the bar as black glass with one highlight line: **no**; plinth, slot bases and rings as dark chrome with glinting chamfers: **no at glance size** (1 to 2 px lines); the car chip as forged carbon with flakes: yes; the boost scale as polished metal: yes, and too bright for the static cap; the tile as a raised red enamel key: yes; highlights sit differently in 04 than in 02: yes (the plinth's top chamfer lights along its whole length in 04).

### The reference and the board
- **Coarse** (`review/dc_real_vs_render_coarse.png`, both blurred alike): the same arrangement of masses, in both orders: a tyre block at the left, a round gauge with a box, a bar across the middle with digits under it, an upright bar each side.
- **Board against render** (`review/board/..._blend.png`, `review/dc_board_before_after.png`): every zone sits where the board has it (the blend doubles only the trip digits, by about 3 px, and the logo's letter weight). **The render does not look clearly richer than the board**: apart from the carbon chip and the glow, the two look alike.
- **Over a bright road** (`review/dc_02_road_centre.png`): the thin speed digits are white on near-white and are carried by their black keyline alone, so they read as outlined digits.

### `design:design-critique` (findings only)
- The eye lands on the lit run and the gear first, which is right, and the alert state (violet run, red tile) cannot be missed.
- The hardware reads as grey interface widgets, not as a 2027 instrument: this is the largest gap to the bar.
- The tyre pills and the trip digits are brighter than their rank (4): they outshine the labels and pull the eye left.
- Three numeral styles sit in one band (thin Michroma, bold Orbitron, and a trip that looks outlined); the trip should recede.
- Over a bright scene the thin speed needs a stronger dark edge than a 2 px keyline.
- What works: the fidelity of the layout, the true black between parts, the alignment on the centre line, and the legibility of gear and speed (11:1 and 8:1 at the glance gate).

### DC answers and decision
| Line | Answer | Evidence |
|---|---|---|
| V1 | yes | the expectations above, committed (`e6172d5`) before the first plate render existed |
| V2 | yes | the cold look and the five rejections above |
| V3 | yes | one `cold-reviewer` run, its 16 findings listed with what is done |
| V4 | yes | first the lit run of the rev bar, second "the '118' and the big '4'" |
| V6 | yes for the gear and the speed in all four states; **no** for the unit and the BOOST label (the rule question) | glance numbers above |
| V10 | **no** | `review/hood_03_redline_strobe_squint.png`: the labels, divider and logo go first and the gear, the tile and the lit run last, as frozen; but the thin speed fades before the tyre pills, the trip box and a full pedal bar (rank 4) |
| V13 | **no** | "Don't know." One run; it is not asked again |
| V14 | yes for layout, geometry and colour, with two gaps 2 to 3 px outside tolerance (logged) | the inventory above |
| V15 | yes | the same arrangement in both orders, by the reviewer and on the coarse sheet |
| V16 | yes for the zones; **no** for "clearly richer than the board" | `review/dc_board_before_after.png` |
| V17 | **no** | the bar, slots, lamps, trip box and tyre pills cannot be named as a material at glance size |
| V19 | NOT ASSESSED | no clear glass piece; the black glass is opaque hardware, judged by T1 and V17 |
| V20 | yes | 0 px outside the envelope, corners off; the bloom's haze round the run is finding 9 |

**Decision: go with changes.** The concept holds: the layout, order and behaviour match GT7's band, and gear, speed and the alert read at a glance. Every no has a fix that keeps the concept:
1. **A second materials pass** (V16, V17, rejections b, d, e): black glass that is black, crowned so it carries one reflection band; wider polished chamfers; a lens highlight on every unlit LED; domed lamp and tyre lenses; a dark glass trip window; a boost scale under the static cap; a car outline for the chip; engraved boost ticks.
2. **Meaning for the unlabelled parts** (findings 6, 7, 8): BRK and THR labels, a rail under the steering dot, thousand marks and a marked alert zone on the rev bar.
3. **The signature moments** (findings 11, 12, 13): a hot core in the alert LEDs, an up chevron in the tile, and a hand-over with no blank or squeezed frame.
4. **Readout details** (V10, findings 3, 9, 14, 15, the bright-road crop): heavier speed strokes and dimmer tyre lenses, trip and pedal fills, so the speed outlasts the side groups in the squint; the trip without leading zeros, a tighter bloom, the unit under the last digit, neutral at full ink, a soft dark edge behind the digits.
5. **The board** is redrawn for the parts that change, and the plate is rebuilt after it (the order of work).

V13 stays no for this review point: GT7's bonnet band has few cues of its own, and the dash gives up two of them on purpose (a white run and a primary-colour dot, where GT7's are pink-red and red; RESEARCH.md, "Deliberate changes").

### TJX's words on the direction-check sheet (2026-11-28, 05:17Z to 05:18Z)

TJX answered in the thread, on `dc_board_before_after.png`: "i like what you made". Pinned on the old gull-wing render (the before row's gear pod): "shit". Pinned on the new rev bar (the lit LED run in the black-glass bar) and on the run's head: "nrautiful" (beautiful) and "amazing". So the lit run and its bloom stay as they were; the cold reviewer's "bloom haze" finding does not outrank TJX's words, and only the alert run's bloom changed (below).

## After the direction check

### The changes (commit `9b0a5e4`, rendered on lavapipe 05:24Z)

| Group | Change | Evidence |
|---|---|---|
| 2 meaning | BRK and THR under the slots (Titillium Web 600, cap 22, static ink); thousand-rpm marks under the plinth placed by the car's limiter; the alert zone's lenses smoked red in the plate (from LED 85, where the run takes the shift colour, test `the_alert_zone_starts_where_the_run_changes_colour`); a rail under the steering dot with a centre notch and end stops | `review/p2_03_redline_strobe.png` |
| 3 moments | Alert LEDs carry a hot white core and their own bloom (threshold 0.2), so the alert run outshines the white run it replaces (test `the_alert_run_is_brighter_than_the_white_run`); the tile shows an up chevron over the next gear (cap 42); the hand-over slides the whole tile, digit kept, 90 px towards the gear while it shrinks and fades (no blank or squeezed frame, test `taking_the_suggested_gear_folds_the_tile_and_keeps_it_away`) | `review/p2_handover.png` (frames 41 to 47) |
| 4 readouts | Speed at synthetic weight 0.3 with a soft outer keyline ring (12 copies at 1.7 widths, alpha 0.25) on the speed and gear; the unit right-aligned under the last digit; the trip as whole units, a point and tenths with its unit (`0.0 mi`), right-aligned; neutral at full ink; tyre lenses lit from inside (a radial light, screened over the lens); pedal and boost fills at 0.7 of the ink, under the lit run | `review/p2_03_centre.png`, `review/p2_02_left.png` |
| 1 materials | Recipe: the zenith sky at 0.4 and the studio world at 0.5 of the hero rig's, and a window softbox on the key side (el 52°, half size 14°, so it never reaches the zenith); the light bar pillowed (10 px tall, 9 px round edge, a flat centre 28 px tall for the 26 px LED field); domed lenses (2 px, pill corners); half-round slots; domed telltale lamps; pillowed black-glass tyre lenses; a polished gunmetal boost arc with four engraved marks (−1, 0, 1, 2 bar); a car outline for the chip (narrower nose, mirrors, hips); deeper polished chamfers on every base. Rebuilt on the cloud lane through the PC | the PC's report (pending) |
| 5 board | Zone table v3 boards drawn from the new numbers (`board_v3`), outlined for Penpot | Penpot page `07 · Gt7HoodPro` (pending) |

### Code audit (workflow `gt7-hood-pro-code-audit`: four read-only auditors, each followed by one refuter; findings only)

| Id | Finding | Refuter | Action |
|---|---|---|---|
| HP-1, NS-1 | No off-hysteresis at the shift point or the alert: an rpm on the threshold flashed the tile and restarted the strobe | real (with a guard for an unknown limiter) | fixed: both release 120 rpm under their thresholds, only for a known limiter (test `the_shift_point_holds_through_a_dip_and_an_unknown_limiter_stays_dark`) |
| HP-2 | The run's release takes 100 to 150 ms to drain a large drop | not real (the tokens' release is met as the project reads it) | kept |
| HP-3, RL-5, NS-9 | Clearing the user's display font never brought the dash's own faces back | real | fixed: the faces from `new()` are kept and restored |
| RL-6 | A wide user face was set with no tracking, so its digits overlapped | | fixed: a user face's tracking is its widest digit less the kit's cell |
| NS-2, RL-4 | The tile's 10 (in 9th) overflowed the face | real | fixed: a proportional run fitted to the face |
| NS-3, RL-3 | The trip was set 5 px right of centre | real | gone with the right-aligned trip (a `number` field needs the bias, a `text` run does not) |
| NS-4 | A dash built past the shift point played the tile's pop on its first frame | real | fixed |
| NS-5 | Frozen F6 cannot hold: the review telemetry has no steering | real | F6 is recorded no above; unit test added; the coordinator is asked about a steer value in the review state |
| NS-6 | The trip restarts on every camera or theme change (the app builds the cluster again) | real | the wording now says "since the view was entered"; a session distance would need the host |
| NS-7 | The slide lamp lit while reversing in a straight line | real | fixed: off in reverse |
| NS-8 | A limiter of 0 rpm made the whole dash a permanent shift alarm | real | fixed: an unknown limiter keeps the bar dark, with no alert, tile or limiter lamp |
| RL-1, RL-2 | The tile's sprite cross-fade doubled its bloom and every anti-aliased edge thickened during a swing | | fixed: the rest render at 1 − swing and the swing render added, so coverage sums to one (`hardware::blend`); spec.md names the tile as the second bloom layer |
| RL-7 | Long wordmarks shrank far under the label floor | | fixed: set in the label face at 22 px when the wide face would go under it |
| RL-8 | The boost scale was a static part far over the cap | | the arc is now polished gunmetal; measured after the rebuild |
| RL-9 | Pedal and boost fills burned brighter than the lit run | | fixed: 0.7 of the ink (test `the_side_groups_burn_below_the_lit_run`) |
| RC-1 | A changed carbon texture would not reach the plate (the Blender cache keys on its path) | pending | the texture does not change in this rebuild; logged as a request for the hero library (hash the bytes of texture parameters) |
| RC-2 | The ComfyUI master is not copied to Drive under its ledger name | pending | to fix with RC-1 |
| RC-3 | The tile's baked red is not exactly the `warning` role after the render's light | pending | kept baked (enamel under light is not a flat role colour); the comments say so |
| RC-4 | Two radii were bare literals the drift test could not see | | fixed: `TICK_R`, `SUG_R` named and checked (commit `59c8a0b`) |

