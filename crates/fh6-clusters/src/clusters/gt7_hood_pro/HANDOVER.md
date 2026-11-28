# HANDOVER: Dash · Gt7HoodPro (catalog row 3)

**Thread / branch:** "Rebuild Gt7HoodPro dash", `claude/project-thread-h7qahw`, draft pull request #10  **Date:** 2026-11-28  **Result:** ready for merge (closing review done; the v3 boards re-exported at 07:39Z)

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

TJX on the closing before-and-after sheet (06:36Z, a pin on the new redline state's lit run): "this design is clean the two befores are so bad". Two more pins on the same sheet, on the red tile: "could be spaced better to the right not all the way", and on the speed digits: "could be spaced better to the left, not all the way". Both are done (below, "The closing spacing").

## After the direction check

### The changes (commit `9b0a5e4`, rendered on lavapipe 05:24Z)

| Group | Change | Evidence |
|---|---|---|
| 2 meaning | BRK and THR under the slots (Titillium Web 600, cap 22, static ink); thousand-rpm marks under the plinth placed by the car's limiter; the alert zone's unlit lenses tinted red, drawn live from the driver's shift point (LED 85 at the tokens' shift point; first baked into the plate, then moved to the dash after the audit's refuter found that the OSD's shift point moves it, test `the_alert_zone_follows_the_shift_point`); a rail under the steering dot with a centre notch and end stops | `review/p2_03_redline_strobe.png` |
| 3 moments | Alert LEDs carry a hot white core and their own bloom (threshold 0.2), so the alert run outshines the white run it replaces (test `the_alert_run_is_brighter_than_the_white_run`); the tile shows an up chevron over the next gear (cap 42); the hand-over slides the whole tile, digit kept, 90 px towards the gear while it shrinks and fades (no blank or squeezed frame, test `taking_the_suggested_gear_folds_the_tile_and_keeps_it_away`) | `review/p2_handover.png` (frames 41 to 47) |
| 4 readouts | Speed at synthetic weight 0.3 with a soft outer keyline ring (12 copies at 1.7 widths, alpha 0.25) on the speed and gear; the unit right-aligned under the last digit; the trip as whole units, a point and tenths with its unit (`0.0 mi`), right-aligned; neutral at full ink; tyre lenses lit from inside (a radial light, screened over the lens); pedal and boost fills at 0.7 of the ink, under the lit run | `review/p2_03_centre.png`, `review/p2_02_left.png` |
| 1 materials | Recipe: the zenith sky at 0.4 and the studio world at 0.5 of the hero rig's, and a window softbox on the key side (el 52°, half size 14°, so it never reaches the zenith); the light bar pillowed (10 px tall, 9 px round edge, a flat centre 28 px tall for the 26 px LED field); domed lenses (2 px, pill corners); half-round slots; domed telltale lamps; pillowed black-glass tyre lenses; a polished gunmetal boost arc with four engraved marks (−1, 0, 1, 2 bar); a car outline for the chip (narrower nose, mirrors, hips); deeper polished chamfers on every base. Rebuilt on the cloud lane through the PC | the PC's report; then the second and third passes below |
| 5 board | Zone table v3 boards drawn from the new numbers (`board_v3`), outlined for Penpot | Penpot page `07 · Gt7HoodPro`, v3 boards made at 07:05Z (ids in README.md and under "The closing spacing") |

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
| HP-1 (refuter's new finding) | The red alert zone baked at LED 85 ignores the driver's OSD shift point | new | fixed: the zone is drawn live from the shift point; the plate's lenses are all smoked again (next rebuild) |
| HP-2 (refuter's new finding) | The committed plate predates the recipe | new | the rebuild runs on the PC; a second rebuild follows for the live zone |

### The user-facing name

The OSD's cold reviewer, through the coordinator (05:46Z), flagged "GT7 Hood Pro" as a release risk: the name the user sees names another game. The dash is now shown as **Bonnet Line** (the low band of instruments that floats over the bonnet): `registry.rs`, `catalog.rs` (`display`), the OSD's theme names (`fh6-osd/src/model.rs`), `docs/TELEMETRY_USAGE_MATRIX.md`, and this module's README, spec and doc. The internal identifier stays `Gt7HoodPro` (theme, config value `"Gt7HoodPro"`, folder, assets and Penpot page), so saved configs and other threads are unaffected, and RESEARCH.md still names GT7 as the reference. CATALOG.md is the chain's file and is not edited here; its row keeps the identifier.

### The second and third materials passes

The second plate rebuild (PC, cloud lane, $0.059) brought the plate's p95 to 0.021 (was 0.044) and the static p95 to 0.190 (was 0.355), fixed the boost scale (p95 0.14), and kept the lens glints, the domes and the slots. It left two defects, both from the same physics: in a top-down orthographic render a flat polished face mirrors only the zenith, and a light on the key diagonal lights only the rounded ends of a long bar. So the bar's long top and bottom edges carried no highlight band, and the trip window came out flat mid grey (#3D3D3F) instead of dark glass. The thousand marks, at full static ink over the plate, measured 0.206 to 0.210, just over the 0.20 cap.

The third pass (commit `248cc98`) adds two thin strip lights straight up and straight down the screen (`TOP_STRIP`, `BOTTOM_STRIP` in the recipe, the bottom one dimmer so the bar keeps one key light), makes the trip window black glass, and draws the thousand marks at 0.8 of the static ink (`MARK_LIGHT`).

### The closing spacing

TJX's two pins on the closing sheet (06:45Z) moved the centre group apart (commit `b3f3039`, on the PC). The tile's centre went from x 211 to 280 (`SUG_C`, and its frame from x 156 to 225 in the recipe), short of the THR slot at x 442, and its sprite was rebuilt on the cloud lane. The speed's right edge went from x -26 to -62 and the gear's left edge from 30 to 44, so the gap round the divider opens evenly on both sides. The tile's hand-over slide grew from 90 to 159 px (`HAND_SLIDE`), so it still ends at the gear.

**The order of work:** this move went into the code first and onto the Penpot boards straight after (the v3 boards, made at 07:05Z, carry it); the post-direction-check changes of `9b0a5e4` also reached the boards only with v3. That is a no on closing line 30 that cannot be fixed afterwards (below). From here on the board comes first (TJX, 07:08Z: "never ever dont we always make in penpot first then code ?").

Re-measured on lavapipe at `b3f3039` (`hood_<state>.png`, the moved zones): gear 15.0 to 16.0:1 (needs 7), speed 9.4 to 10.4:1 (needs 4.5), tile 10.6:1 in `03` (the only state that shows it), trip 4.5:1. The gear change (`review/spacing_handover.png`, frames 40 to 48): "4" from frame 42, the tile slides left, shrinks and fades and is gone by frame 48, with no blank or doubled gear. Renders `review/spacing_02.png`, `review/spacing_03.png`.

## Shared places touched
- The user-facing name only: `crates/fh6-clusters/src/registry.rs` and `crates/fh6-clusters/src/catalog.rs` ("Bonnet Line"), `crates/fh6-osd/src/model.rs` (the theme's display name), `docs/TELEMETRY_USAGE_MATRIX.md`. One line each.
- Nothing in `fh6-kit`, `common/`, the hero library, `docs/specs/optics.md`, CATALOG.md (main's copy is kept on every merge) or the shared ledgers, which every build rewrites and the PC restored each time.

## Requests for shared code
1. **Hero library texture cache (RC-1):** the Blender cache keys a texture parameter on its path, so a changed texture at the same path does not reach the render. Hashing the file's bytes into the key would fix it.
2. **ComfyUI master copy (RC-2):** the Drive master of a ComfyUI texture is not copied under its ledger name; it should be, so the ledger and the masters folder agree.
3. **Review telemetry steering:** `steer` is 0 in every review state and sequence frame, so a steering indicator (this dash's dot) can never be checked from renders. A small steer value in `04_drift_braking` and a sweep in the sequence would let siblings check it.
4. **`common::VirtualLight` settled check:** a `settled()` accessor would let a dash's `animating()` keep the render loop drawing while the light's spring settles after a corner (rust-review finding 5).

## What sibling dashes must know
- **A flat polished face in a top-down ortho render mirrors only the zenith.** A bright overhead sky turns every flat black-glass face uniform grey; keep the zenith dark and light the faces from low angles.
- **Long horizontal edges need lights straight up and down the screen.** A window light on the key diagonal lights only a bar's rounded ends; thin strips at the edges' own azimuth light the long sides.
- **Bake nothing that a setting can move.** A red zone baked at a fixed LED ignores the driver's OSD shift point; draw it live from the shift point.
- **Cross-fade the light swing so coverage sums to one:** draw the rest render at `1 - swing` and add each swing render with `BlendMode::Add`, or anti-aliased edges pop.
- **A user-facing name must not name another game.** Keep the identifier, change only what the user reads.

## Visual review (closing round)

### Frozen expectations of the closing round (written 06:35Z on 2026-11-28; before it, of this round's renders only the 1:1 crops of `hood_02_mid_cruise` posted with the third materials pass and a 3x crop of the trip tile had been opened, and no full frame, state sheet or sequence frame)

The direction check's expectations stand, with these changes, each logged before re-checking:
- **New parts:** the steering rail (`rail` [350, -94, 6]) with its dot on the centre in every state (the review telemetry holds `steer` at 0, so F6's "off-centre in 04" cannot show: NOT ASSESSED by render, tested in code); thousand-rpm marks under the plinth, 8 of them for the 8700 rpm limiter; BRK and THR under the pedal bars; the trip tile reading "0.0 mi" (the view starts fresh); the speed unit right-aligned to the speed.
- **F5 changed:** the tile carries a chevron over its digit and a label under it. **F11 changed:** the lenses from the alert stage to the limiter carry a dim red zone in every state with a known limiter (drawn live from the shift point, about 0.22 of the warning colour), and in `03` the alert cores light in the shift colour; red elsewhere still only on a lit limiter lamp.
- **Key frames changed:** the tile no longer folds into the gear; from frame 42 it slides left, shrinks and fades over 6 to 8 frames, keeping its digit, and no frame shows it blank.
- **Glance:** the trip digits (a readout) reach 4.5:1 now that the window is dark glass; the unit and BOOST label remain static labels under the cap and under 4.5:1 (the same rule question as at the direction check).
- **Materials:** the bar's long top and bottom edges carry a highlight band; the trip window reads as dark glass; every lens is smoked.
- **Static:** p95 at or under 0.20 including the thousand marks (0.8 of the static ink).
- **Budget:** under 0.5 ms GPU at 4K on the PC (the second pass measured 0.24 ms p99).

### Closing-round cold look (06:38Z, `hood_02_mid_cruise_road.png` at full frame, its mirror `review/final_02_mirrored.png` and `review/hood_02_mid_cruise_luminance.png`, before the notes, spec, board or references)

A low band of separate black parts sits on the bonnet below the road, centred under the vanishing point. The eye goes first to the white run of the bar and the "4" right of a thin divider, then "118" with "mph" under it, then the cyan dot on a thin rail above the bar, then the red cells at the bar's right end. The bar reads as a glossy black capsule with a light rim along its top and bottom and a row of dark cells, the left half lit white. Under the bar sit eight short ticks. Left of the bar, from the edge in: a carbon car seen from above with four grey lenses, a round gauge labelled BOOST over a dark window reading "0.0 mi", a domed lamp, and a tall empty slot labelled BRK; right of the bar a slot labelled THR with a short white fill, a domed lamp, and a grey PORSCHE wordmark. Over the bright bonnet every glyph carries a black keyline, so "118", "4", "mph", BOOST, BRK and THR stay legible. In the luminance view the red zone is only a few cells slightly lighter than the unlit ones, and the white run, the gear and the speed are the three brightest masses; the labels, wordmark and rims are mid grey. The mirror reads the same way round. The domes, the slots' rims and the bar's capsule look like lit objects; the rail and the thousand ticks look flat (thin printed lines).

**Failure first.** Likeliest rejection: (b) the car chip's carbon and the lens domes are small, so materials show only at 1:1 (check on `review/p3_left.png`: the carbon flakes, the dome glints and the trip window's rim read at 1:1, and at full frame the chip reads as a textured car, not flat grey: holds). (a) Against GT7's meter (the reference), the band, the bar of dense ticks, the speed-divider-gear group and the side groups are where they are on the real meter: holds. (c) No backing panel: every part sits on true black with gaps (stretch: 0 px outside the envelope). (d) The capsule bar, domes and enamel tile are rendered hardware, not flat widgets; the rail and ticks are flat by design (printed scale). (e) The black glass is opaque hardware, not frosted: holds (V19 NOT ASSESSED).

### The cold reviewer, closing round (one `cold-reviewer` run at 06:41Z; packet `mockups/blind/47/`: A.png, glance_01 to 08 at seed 47, R1.jpg; logo zone masked)

- **Five-second test:** noticed first "the white lit part of the segmented bar", second "118" and the gear "4", third the car shape; it named the display's purpose as revs, speed, gear, pedals, boost and distance.
- **Unprompted recognition:** "Don't know."
- **Blind glance read** (key `review/blind_key_47.txt`): the gear right on 8 of 8 frames; the speed within 1 on 8 of 8 (75 / 75.4, 90 / 90.3, 118, 63 / 62.9, 95, 92 / 92.5, 0, 98); the zone right on 7 of 8: on glance_02 (frame 27, 8492 rpm, 98%: the limiter zone by the key's 97% line) it wrote "high or limiter; don't know which, because a sliver of red still shows at the bar's right end". That sliver is true data: at 98% the run has not reached the bar's end. The shift lights it read as all lit on 02 and 06 (the run in the shift colour) and dark elsewhere, matching each frame's crop.
- **Layout against R1:** "Same" in both orders, with the differences it named: R1 has about four small icons each side where ours has one (deliberate, F10), R1's bar is red where ours is white below the alert, and R1's gear shows a second red digit (our tile).

| # | Finding | Fixed? | Evidence or reason |
|---|---|---|---|
| 1 | The trip reads "0.0 mi" in every frame, even at 118 mph | not fixed | the view starts fresh and the review set covers one second; the counter is tested in code (`the_trip_counts_tenths_and_holds_at_its_limit`) |
| 2 | BOOST shows no number and no scale; the arc fill is faint | not fixed | GT7's round gauge is a thin scale with an icon and no digits (RESEARCH point 2); the fill sits at 0.7 of the ink to stay under the lit run (rank 4) |
| 3 | The car chip's mottled grey texture looks cheap and noisy | not fixed | forged carbon is the brief's material for the chip (the direction check's cold reviewer named it as carbon); recorded for the closing critique to weigh |
| 4 | The two small round lamps are too small to tell what they show; the right one turns red on 06 | not fixed | the slide and limiter telltales, 56 px; the limiter lamp lights red on the limiter by design (F11) |
| 5 | The cyan dot on the line above the bar has no label | not fixed | GT7's steering indicator has none either (RESEARCH point 6) |
| 6 | The ticks under the bar have no rpm numbers | not fixed | GT7's bar has none (RESEARCH point 5); the thousand marks are the scale |
| 7 | The white-to-shift-colour change is small in brightness and the bar's only shift cue; the tile is easier to see | not fixed | the strobe (10 Hz) is the bar's cue and a still shows one phase only; V7 settles the squint order |
| 8 | On glance_02 a sliver of red at the bar's end leaves it unclear whether the limiter is reached | not fixed | true data at 98% (see the blind read) |
| 9 | A soft grey haze lifts the black round the bar | not fixed | the lit run's bloom, which TJX praised on the direction-check sheet ("beautiful", "amazing"); stretch shows it is the glow only |
| 10 | The left side carries four items, the right two | not fixed | the logo zone at the right end was masked for the reviewer, as at the direction check |
| 11 | BRK, THR, BOOST and "mph" are small grey text, weak at glance size though readable | not fixed | static labels held under the 0.20 static cap; the rule question the pilot and PorscheAnalog reported |

### `design:design-critique` (closing round, findings only)

- **First impression:** the white lit run and the "118 | 4" block lead, then the tile when it shows; the band reads as one low instrument floating on the bonnet, recognisably GT7's layout (the cold reviewer called the layout the same as R1 in both orders). Correct order.
- **Hierarchy:** the squint sheet (`review/final_squint_sheet.png`) fades the labels, logo and side groups first and keeps the gear, the run and the tile; the strobe's on and off phases (frames 33 and 30) and each state differ plainly. The speed and gear keep their keylines over the bright bonnet (`hood_02_mid_cruise_road.png`).
- **Usability finding:** the shift state on the bar is a colour change plus the strobe; in a still the tile carries it. It reads by count (the run is full) and by the tile in the luminance and every colour-vision panel (`review/hood_03_redline_strobe_cvd_sheet.png`), so no change.
- **Consistency finding:** the rail and the thousand marks are flat printed lines beside rendered hardware. They are the scale and the steering track, printed on the plate like the real meter's; no change.
- **Consistency finding:** the labels (BOOST, BRK, THR, mph) and the wordmark share one grey under the static cap; the trip digits now use the speed's face at a heavier weight. Consistent with the family.
- **Material finding:** at 1:1 the bar's long edges now carry a highlight band, the domes and slots glint, the trip window is dark glass, and the car chip shows carbon flakes; at full frame the chip reads as textured rather than as named carbon (the cold reviewer's "mottled, grainy"). Recorded; the brief's material is kept.
- **What works:** the strobe and tile pairing; the tile's slide into the gear with no blank frame (`review/final_gear_change.png`); true black between every part; static parts under the cap.
- **Priority, if more time:** a finer carbon weave on the chip so it reads as carbon at glance size; nothing else rises above the recorded findings.

### `design:accessibility-review` (closing round; contrast, use of colour and flashing only)

- **1.4.3 / 1.4.11:** gear 14.1 to 16.1:1, speed 9.5 to 10.4:1, the rev bar 5.0 to 8.5:1, the tile 10.0:1, the trip 4.5:1 in all four states (glance gate, `review/closing_measure.txt` and the re-run after the trip change). The unit (2.3 to 2.5:1) and the BOOST label (2.0:1) are static labels under 4.5:1, held under the static cap: the open rule question.
- **1.4.1:** the shift state reads by count and the tile's presence, not by colour alone (cvd sheet).
- **2.3.1:** one flashing area, the LED run (758 x 26 px), 10 Hz at 3 on / 3 off; the worst 10-degree field holds 7950 of 21526 px2 (under 25%); no red flash. The reduced-flash mode, from the code (`RevBar::level`): the strobe keeps `STROBE_FLOOR` of the light in its off phase, and with `FrameInput::reduced_flash` set the run breathes slowly (`Strobe::breath`, never under the floor) instead of flashing at 10 Hz.
- No motion without data: the idle frames 6 to 9 are still.

### `rust-review` (with `rust-skills`, closing round, one run)

No unwrap, expect or panic outside tests, no per-frame allocation, and no unguarded division by a packet value; clippy clean for this module.

| # | Finding | Fixed? | Evidence or reason |
|---|---|---|---|
| 1 | The gear change was detected by `since_change() == 0.0`; a frame with dt 0 after a shift would read as a second change and restart the tile's hand-over | fixed | `changed` compares the shown gear before and after `HeroGear::update` |
| 2 | The recipe's `BOOST_BAR` was not in the drift test, so the baked boost marks could leave the live fill | fixed | `("BOOST_BAR", g.boost_bar)` added to `the_recipe_renders_the_same_band` |
| 3 | A master that fails to load was skipped silently in `write_previews` | fixed | prints which preview was skipped and why |
| 4 | `Geom::slot_rect(i, inner: bool)` was a bool parameter used across files | fixed | split into `slot_rect` and `slot_outer_rect`, like `sug_outer_rect` |
| 5 | `animating()` ignores the virtual light's spring, so the plate's cross-fade can settle on a frame the loop skips | not fixed | `common::VirtualLight` has no settled accessor and is shared code; logged under requests. While racing a packet arrives every frame |
| 6 | `light_weights`' doc said the rest render is drawn at 1; `blend` draws it at 1 minus the swing | fixed | doc corrected |

### V1 to V26 (closing round)

Renders on lavapipe in the cloud container at `b3f3039` (identical to `dae1263` apart from the goldens), images under `mockups/Gt7HoodPro/review/`.

| # | Answer | Evidence |
|---|---|---|
| V1 | yes | the closing round's expectations were frozen at 06:35Z, before this round's full frames, state sheet or sequence were opened (above) |
| V2 | yes | the cold look of 06:38Z, then each of TJX's five rejection patterns checked against a named crop (above) |
| V3 | yes | one `cold-reviewer` run at 06:41Z on `mockups/blind/47/`; its 11 findings are each recorded with what was done (above) |
| V4 | yes | first "the white lit part of the segmented bar" (the shift light's run), second "118" and the gear "4" |
| V5 | no, cannot be fixed afterwards | gear right 8 of 8, speed within 1 on 8 of 8, shift lights as each frame shows them, the rpm zone right on 7 of 8: on glance_02 (98%, the limiter zone by the key) it answered "high or limiter; don't know which". The sliver of unlit LEDs at 98% is true data, so nothing in the dash was changed for it |
| V6 | yes | `glance` on `hood_<state>.png`: gear 15.0 to 16.0:1 (needs 7), speed 9.4 to 10.4:1, the lit rev bar 5.0 to 8.5:1, the tile 10.6:1, the trip 4.5:1. The unit (2.3 to 2.5:1) and the BOOST label (2.0:1) are static labels held under the static cap, as at the direction check (the open rule question the pilot and PorscheAnalog reported) |
| V7 | yes | `review/final_squint_sheet.png`: idle, the white run, the alert run with the tile, and the strobe's off (frame 30) and on (frame 33) phases each differ plainly |
| V8 | yes | `geom.rs` `HOOD`: speed cap 58 px, trip 22, unit 22, tile digit 42; the gear 88 px (needs 36 in the hood view); the thinnest live stroke, the rail, 6 px; confirmed on `review/spacing_03.png` at 1:1 |
| V9 | yes | `flash --frames 30-41`: one flashing area, the LED run (4K 1541,1700 758x26); no digit is marked |
| V10 | yes | the squint sheet: labels, logo and the side groups fade first, then the speed, and the gear, run and tile last, in all four states |
| V11 | yes | `static` (PC, third pass): p95 0.191 under the 0.20 cap; the thousand marks p95 0.165. Over the cap: 27 px at the plinth's top-left corner (p95 0.27), the brightest point of the new long-edge highlight, a specular glint rather than an area; the steering dot and the digit strokes the four states share are live parts |
| V12 | yes | red only on the limiter lamp and the alert zone (the redline marker, 0.22 of the warning colour); the product page's red badge 1% outside the redline state and 5% in it, within budget; `review/hood_03_redline_strobe_cvd_sheet.png`: the shift state reads by count and by the tile in every panel |
| V13 | no, cannot be fixed afterwards | unprompted, logo masked: "Don't know." GT7's bonnet band has few cues of its own, and the dash gives up two on purpose (RESEARCH.md, "Deliberate changes"); the layout against R1 was "Same" in both orders |
| V14 | yes | the direction check's measured inventory (above), plus the closing changes measured on the render: 8 thousand marks for the 8700 rpm limiter, the tile at x 280, the speed's right edge at -62, the trip "0.0 mi" (`review/spacing_03.png`, `closing_measure.txt`) |
| V15 | yes | blurred to the same level, reference and render show the same masses (`review/dc_real_vs_render.png`); the cold reviewer called the layout the same as R1 in both orders |
| V16 | yes | the v3 boards re-exported from Penpot at 07:39Z (heavier trip digits added) against the `b3f3039` renders (`review/v16_centre.png`, ink boxes local to the anchor, render then board): speed x -271..-68 against -270..-69, the unit within 1 px, the divider the same pixels, the gear 45..128 on both, the tile 236..323 on both, the trip "0.0" -619..-560 on both with stems about 3.4 px on both. `diff --fit` of `03`: 3.82% of pixels over dE 3, all in the render's materials, bloom and glow, which the flat board does not draw. One finding: the board's speed digits are lighter than the render's (the render adds `SPEED_WEIGHT` 0.3 and the soft keyline); place and size match. The render is plainly richer than the board and the before render (`review/final_before_after.png`) |
| V17 | yes | at full frame the capsule bar reads as black glass, the plinth as black chrome, the domes and slots as glass, the chip as textured carbon, the tile as enamel, each lit from the one key (upper left) |
| V18 | yes | the materials checks for black glass, chrome, enamel and carbon: lit long edges (glass top 0.22, plinth top 0.11), the trip window dark glass (0.017), dome glints, no texture on vector shapes (`review/p3_left.png`, `review/p3_right.png`) |
| V19 | NOT ASSESSED | the dash has no clear glass; its glass is opaque black hardware |
| V20 | yes | `stretch` on all four states: 0 px outside the envelope plus 40 px; the corners off; the bloom the only glow round the run |
| V21 | yes | `motion`: no stall in either pull; the gear changes on frame 42 with the data; the run follows the rpm on the same frame |
| V22 | yes | `flash`: 10.0 Hz, 3 on / 3 off in all 18135 px of the strip, the worst 10-degree field 7950 of 21526 px2, the strip 26 px tall (limit 42), no red flash |
| V23 | yes | `review/spacing_handover.png` (frames 40 to 48): "4" from frame 42, never blank or doubled; the tile's slide and fade end within 6 frames |
| V24 | yes | the `motion` spikes at frames 1 (the start-up), 10 (the pull begins), 25 (the alert run) and 27 (the strobe starts lit) are each the data's; idle frames 6 to 9 are still |
| V25 | yes | `review/final_before_after.png`: the before's named failure (gull-wing ribbons that look nothing like GT7's meter) is gone; nothing the before did better is lost (checked on the sheet's crop pairs; TJX on the sheet: "the two befores are so bad") |
| V26 | yes | the four goldens were rewritten on lavapipe at `b3f3039`, opened and compared with the previous ones (`review/final_goldens.png`): every difference is the rebuild (the old goldens showed the gull-wing design); they passed twice without the update variable (commit `dae1263`) |

### What the review set could not show (fh6-visual-review section 8)
- The steering dot off centre: the review telemetry holds `steer` at 0 (tested in code; request 3 above).
- Flash rates from 3 to 6 Hz and the WCAG three-flashes-in-one-second count: the limiter hold is 12 frames.
- The reduced-flash mode: judged from the code (`RevBar::level`, `Strobe::breath`).
- The trip counting: the view starts fresh and the sequence is one second (tested in code).

## Before and after pair

- Before: `docs/handover-shots/cluster-gt7_hood_pro-before_02_mid_cruise.png` (the 2026-11-23 gull-wing build, full frame, kept before the work started).
- After: `docs/handover-shots/cluster-gt7_hood_pro-after_02_mid_cruise.png` (full frame, `b3f3039`).
- The four states at 1:1: `docs/handover-shots/cluster-gt7_hood_pro-0{1..4}_*.png`. The old build's shots of its parts were removed with it.

## Assets and masters

The runtime files, recipe, seeds, licences and master names are in [ASSETS.md](../../../../../assets/clusters/gt7_hood_pro/ASSETS.md):
- **ComfyUI:** the forged-carbon texture (seed 73101, 4096 px seamless), rendered into the plate's car chip; master `gt7_hood_pro_forged_carbon.png`.
- **Blender:** `plate_0.png` to `plate_2.png` (the band's hardware) and `suggest_0.png` to `suggest_2.png` (the tile), each at the light's rest azimuth and its ±25° swing, rendered at 2 px per 4K px; masters `plate/beauty_0..2.png` and `suggest/beauty_0..2.png` in the Drive folder `FH6 WheelDash v2 - 4K Masters/gt7_hood_pro/`.
- **Cloud:** every ComfyUI and Blender job ran on the cloud pod through `asset-forge --lane cloud`, driven from the PC, with `cloud down` after each batch.

## Changes from the real display and from the brief

[RESEARCH.md](RESEARCH.md), "Deliberate changes and why", lists each one with its reason. In short: the suggested gear is an upshift prompt from the shift point (FH6 sends no driving line); the steering dot takes the primary colour, not red; one telltale each side (slide and limiter), because FH6 sends no assists; no tyre heat or wear, fuel, compound or braking suggestion, because the verified telemetry carries none; a narrower band for the 1760 px hood envelope; the car's logo at the right end; and a 2027 production finish (a lit Blender plate and an enamel tile) over GT7's flat graphics. Against the brief: the redline board shows the alert LEDs with white hot cores in the dash's shift colour, because that is what the dash draws; the brief's "alert cores light red" is not followed.

## Verification (cloud container, `dae1263` plus this handover; Rust 1.97, the dash's crates per `-p`)

```
cargo check -q -p fh6-clusters -p asset-forge                      exit 0
cargo test -q -p fh6-clusters --lib gt7_hood_pro                    test result: ok. 25 passed; 0 failed
cargo test -q -p fh6-clusters --lib gt7_hood_pro (again)            test result: ok. 25 passed; 0 failed
cargo test -q -p asset-forge gt7_hood_pro                           test result: ok. 6 passed; 0 failed
cargo test -q -p fh6-clusters --test dash_gates                     test result: ok. 10 passed; 0 failed; finished in 52.44s
cargo clippy -q -p fh6-clusters -p asset-forge --all-targets        no finding in gt7_hood_pro files
rustfmt --edition 2021 --check (module and recipe)                 exit 0
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images gt7_hood_pro, then twice without it (lavapipe)   ok, ok
```
After merging `origin/main` (`c03f6ae`, then `f1f02a3` and `7a57273`; CATALOG.md kept as main has it): `cargo check -p fh6-clusters -p asset-forge` exit 0; `cargo test -p fh6-clusters --lib` 279 passed at the last merge (273 at the first); `cargo test -p fh6-kit --lib` 177 passed; asset-forge gt7_hood_pro 6 passed; `dash_gates` 10 passed (53.40 s); `cargo test -p fh6-osd` 64 passed; the Gt7HoodPro golden on lavapipe passed.

On the PC at `b3f3039` (rustc 1.100): lib tests, asset-forge tests, dash gates, `cargo clippy -p fh6-clusters -p asset-forge --all-targets -- -D warnings` and fmt pass.

## Performance

**GPU** (`preview-export`, layout `b3f3039`, RTX 3070 Ti through `gpu-lock`, hood view at 3840x2160; the catalog lists the hood view only): 01 idle 0.190 ms, 02 mid cruise 0.249 ms, 03 redline strobe 0.257 ms, 04 drift braking 0.246 ms; the 60-frame sequence p50 0.247 ms, p99 0.251 ms. Budget 0.5 ms.

**Live bench** (60 s, `mock-udp --scenario bmw_m4_cs` at 60 Hz):

Both runs at `dae1263` on the PC (RTX 3070 Ti, DX12, swapchain 1920x1080 Mailbox, frame latency 1, DWM 300 Hz), 60 s after a 2 s warm-up, one `gpu-lock` take for both; no error or warning in either app or `mock-udp` log, both exited 0, no frame idle-dimmed.

| Series (p50 / p99 / max ms) | Gt7HoodPro (`--cluster Gt7HoodPro`) | Scaffold baseline (`--cluster ValkyrieLmp1`, still ☐) |
|---|---|---|
| Frames presented | 8286 (138.1 fps) | 8435 (140.6 fps) |
| New packets shown | 3497 (58.3 per s) | 3499 (58.3 per s) |
| Draw calls per frame | 17.3 | 11.0 |
| Packet receive to present returned | 0.610 / 1.317 / 1.958 | 0.620 / 1.380 / 1.805 |
| CPU per frame | 0.601 / 0.778 / 4.377 | 0.604 / 0.772 / 1.415 |
| of which draw | 0.036 / 0.079 / 0.157 | 0.029 / 0.066 / 0.161 |
| of which kit render | 0.028 / 0.049 / 0.218 | 0.026 / 0.046 / 0.132 |
| Packet receive to next vblank | 2.210 / 4.246 / 4.583 | 2.315 / 4.228 / 4.604 |
| New packets presented before the first vblank | 80.9% | 80.1% |

The dash's own CPU share: 0.601 − 0.604 = −0.003 ms p50, within run-to-run noise, so effectively 0 against the 0.3 ms limit; its draw step is 0.007 ms p50 over the scaffold. The app total meets 1 ms at p99 (0.778 ms). The one 4.377 ms CPU maximum is a single present call of 3.736 ms (present p99 0.279 ms), a compositor or driver stall rather than dash work. Reports: `mockups/Gt7HoodPro/live_bench.md` and `live_bench_scaffold.md` on the PC (gitignored).

## Closing review

| # | Answer | Evidence |
|---|---|---|
| 1 | yes | the module reads no pad input (`git grep -n "0x1000\|0x2000\|0x4000\|0x8000\|buttons" crates/fh6-clusters/src/clusters/gt7_hood_pro` finds nothing) |
| 2 | yes | no stub or stand-in; every field is covered by the golden packets or derived from covered ones ("Telemetry" above) |
| 3 | yes | none outside tests (rust-review, closing round); no `unsafe` |
| 4 | yes | the search finds `format!` and `tracing::warn!` only in `hardware.rs` `Sprites::load` (called from `new`), `Box::new` only in `make`, and `vec!`, `format!` and `collect` only in `geom.rs` tests; `warm_frames_do_not_allocate` passes in `dash_gates` |
| 5 | yes | 0.190 to 0.257 ms per state, sequence p99 0.251 ms ("Performance") |
| 6 | yes | −0.003 ms p50 (0.601 with the dash, 0.604 on the scaffold), both p99 under 0.78 ms; "Performance" |
| 7 | yes | rpm, speed and gear go through the kit's `HeroGear` and `TabularSpeed` and the token springs only; the rev run follows rpm on the same frame. One hold: `backlash` keeps the head LED still while the engine bounces on the limiter (the run is full either way), which delays nothing outside that bounce |
| 8 | yes | the dash has no path of its own that draws; showing and hiding stay the app's |
| 9 | yes | no non-Rust source added; bpy lives as Rust strings in the recipe |
| 10 | yes | `git log origin/main..HEAD`: 24 commits, all by TJX, no co-author trailer; no real name, user path, personal email, legacy build or old design-tool name in the module, recipe or RESEARCH.md links |
| 11 | yes | nothing copied to the game folder or the mirror |
| 12 | yes | no new dependency (`Cargo.toml` and `Cargo.lock` unchanged against main) |
| 13 | yes | "Verification" |
| 14 | yes | the kit's premultiplied paths only; the sprites go through the kit's image loader, which premultiplies once; transparent is `(0,0,0,0)` |
| 15 | yes | in the hood view (the one the catalog lists) the band draws unscaled, and every size in `HOOD` is at least 1 px (the thinnest live stroke 6 px; the tick and LED widths are drift-tested against the recipe); G5 in `dash_gates` passes with no repairs; every division by `max_rpm`, `shift_rpm` or the band's size is guarded (`max(1.0)`, `MIN_LIMITER_RPM`) |
| 16 | yes | each sprite lookup has a procedural branch (`hardware.rs`), loaded once in `new`; `dash_gates` runs without assets |
| 17 | yes | MSDF text from the kit; `TabularSpeed` for the speed and tabular figures for the trip; a user display font replaces the numerals (`mod.rs`, the font choice in `new` and per frame) |
| 18 | yes | true black between every part, no box, no full-canvas fill (`stretch`: 0 px outside the envelope) |
| 19 | yes | no clock read; state in `&mut self`; the tests passed twice and the goldens twice without rewriting |
| 20 | yes | colours from palette roles and tokens; the plate's colours are baked by Blender (recorded in ASSETS.md) |
| 21 | yes | the dash draws no glass, reflective or spectral effect in code; the plate's reflections are Cycles renders |
| 22 | yes | V10 and the squint sheet, all four states |
| 23 | yes | V6 on the 4K hood renders |
| 24 | yes | drawn round (1920, 1760) inside the hood envelope; `stretch` 0 px outside; the hood view is the one the catalog lists |
| 25 | yes | RESEARCH.md: 17 points with sources, each matched or listed as a deliberate change |
| 26 | yes | separate pieces on black, each its own zone in `HOOD`; no backing panel (GT7's band has none) |
| 27 | yes | V17: the bar, plinth, domes, slots, carbon chip and enamel tile read as rendered objects at full frame; the ComfyUI carbon and the Blender plate and tile are visible surfaces |
| 28 | yes | V22 (the strobe) and V23 (the gear change) in `mockups/Gt7HoodPro/sequence/` |
| 29 | yes | palette roles, the user's font and the logo zone come from the frame input; no literal colour, font or brand name in `draw()` |
| 30 | no, cannot be fixed afterwards | RESEARCH.md, then the Penpot page, then ComfyUI, then Blender held for the build and each materials pass; but the post-direction-check changes (`9b0a5e4`) and TJX's spacing pins (`b3f3039`) went into the code before the boards, which caught up with the v3 boards at 07:05Z ("The closing spacing"). Reported in the thread and to the coordinator |
| 31 | yes | the page `07 · Gt7HoodPro` holds the four hood states (v3); ids in README.md and here |
| 32 | yes | the ComfyUI carbon and the Blender plate and tile, each with an ASSETS.md row |
| 33 | yes | rendered at 2 px per 4K px and box-filtered; runtime PNGs in git, masters in the Drive folder ("Assets and masters") |
| 34 | yes | every PC GPU job held `gpu.lock` (by lock file before 07:05Z, through `gpu-lock.exe` after); ComfyUI and Blender ran on the cloud pod with `cloud down` after each batch; the cloud goldens ran on lavapipe in the container, off both lanes. Spend in "Cloud spend" |
| 35 | yes | the direction check was posted with the first build's render; its self-review and TJX's words are above, and each change TJX asked for is done |
| 36 | yes | `mockups/Gt7HoodPro/index.html` from `preview-export` (the PC's copy carries the card's GPU times; the cloud copy shows the software renderer's) |
| 37 | yes | 60 s with `mock-udp` streaming, no error: 8286 frames, receive to present 0.610 / 1.317 ms, CPU per frame 0.601 / 0.778 ms; "Performance"; the sequence holds its digits (V9) and settles without a jump (V21, V24) |
| 38 | yes | findings and yes/no answers only |
| 39 | yes | V1 to V26 above, with V5 and V13 the reviewer's one-run noes that cannot be fixed afterwards, and V19 NOT ASSESSED (no clear glass) |

## Cloud spend

`asset-forge cloud spend` on the PC (07:25Z): this thread's sessions come to $0.47 (its own pod sessions $0.069, a pod session shared with the TeslaAdas thread $0.129, the second materials pass $0.059, the third pass with the moved tile $0.215). The lane stood at $2.90 of the $50 budget, far from the $40 warning; the volume's storage ($0.48) is shown and not counted. `cloud down` ran after each batch, and both cloud locks were released.

## Exit criteria
- [x] The introduction was posted, and the thread is not archived.
- [x] `RESEARCH.md` is written, with sources.
- [x] The Penpot page `07 · Gt7HoodPro` has the four hood states (v3); its page and board ids are in README.md and here.
- [x] A bespoke ComfyUI asset (forged carbon) and bespoke Blender assets (plate, tile) are in `assets/clusters/gt7_hood_pro/`, used, each with an ASSETS.md row; the before and after pair is above; the 4K masters are in the Drive folder.
- [x] The module is complete, with no TODOs, and the goldens pass.
- [x] 4K renders, the 60-frame sequence and the product page are in `mockups/Gt7HoodPro/`.
- [x] The live bench report is in this handover; the GPU time under 0.5 ms is recorded.
- [x] The direction check was posted and self-reviewed to a go, with TJX's words quoted.
- [x] Every closing line is yes, or reported as one that could not be fixed afterwards (30; V5 and V13).
- [x] V1 to V26 are answered above.
- [ ] Reported to the coordinator for the merge; the product page and the completed checklist posted in the thread.
- [x] Draft pull request #10 has been open since the branch's first push.
