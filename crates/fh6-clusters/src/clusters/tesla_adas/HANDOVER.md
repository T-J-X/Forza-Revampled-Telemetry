# HANDOVER: Dash · TeslaAdas (catalog row 15, build #5, ADAS & HUD pattern-setter)

**Thread / branch:** "Rebuild the Tesla ADAS dash", `claude/project-thread-r828zb`  **Date:** 2026-11-28  **Result:** ✓ closing review done, ready for merge (the old build's handover is in git history).

Findings and yes/no answers only; there are no scores in this file.

## Direction check

### Frozen expectations (written 2026-11-28 04:58 UTC, before the first render with every car image was opened)

Recorded honestly: while the code was written, development renders with placeholder cars were looked at (to fix a baked-shadow box, label spacing and the hood road surface). The round reviewed here is the first with the ego, the parked car and the three traffic bodies.

Rank order (fh6-hmi-design-rules 3): (1) the gear and the shift line at the shift point, (2) the speed, (3) the power line and the revs filling the shift line, (4) the road scene (ego car, path, traffic) and the G-meter, (5) labels, the logo, the R · N · D row and the rpm and power figures.

Feature inventory of the real display (from `RESEARCH.md`; positions are fractions of the cluster envelope, as R1 and R2 give them as fractions of the screen), each with the tolerance the render must meet:

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | Three parts: a card on the left, the road scene in the centre, a figure column on the right | R1: map 0 to 35 %, scene centre, speed at 81 % | left group centred at 10 to 30 % of the width, the scene's centre at 45 to 55 %, the figure column at 75 to 90 % |
| 2 | Vanishing point above the car, near the top of the scene | R1: (53 %, 20 %) | x 47 to 53 % of the width; within the top quarter of the scene's height |
| 3 | The ego car seen from behind and above, small | R1: (50 %, 73 %), about 8 % of the width wide | x 48 to 52 %; y 55 to 80 % of the height; 8 to 14 % of the width wide |
| 4 | Thin pale lane lines, small grey traffic | R1 | lane lines dimmer than the speed digits; traffic paint has no hue (`probe`) |
| 5 | The biggest white figure in the right column with a small grey unit under it | R1: speed digits 16 to 17 % of the height | deliberate change A: the gear is the biggest (at least 25 % of the height), the speed 14 to 18 %, its unit under it in grey |
| 6 | Drive-mode letters in a row at the column's foot, the selected one lit | R1: (77 to 80 %, 93 %) | R · N · D at 85 to 95 % of the height, the current letter the brightest of the three |
| 7 | Power meter: one thin centre-zero line along the bottom centre | R1: x 32 to 62 % | inside x 30 to 70 %, at 88 % of the height or lower, at most 6 px thick |
| 8 | A blue path band the width of the car, leading away from it | R3 | the palette's primary; 0.9 to 1.3 car widths at the nose |
| 9 | A glossy car with a crisp glass roof and lit tail lamps | R3 | the roof glass darker than the paint, the tail bar lit red, highlights on the body at 1:1 |
| 10 | A soft glow on the road under the car | R3: blue, while FSD drives | deliberate change H: none in `02_mid_cruise`, the shift colour in `03_redline_strobe` |
| 11 | Parked: the car alone, large, three-quarter front | R2 | `01_idle` shows it at 30 % of the width or more |
| 12 | No lit screen rectangle | deliberate change G | `stretch` finds no rectangle edge |
| 13 | Track Mode G-meter: rings, the current point, the shaded peak history | manual text | rings and dot in every state; the shaded history in `04_drift_braking` |
| 14 | Japan: lane lines 8 m on, 12 m off | NEXCO | gaps longer than dashes at the same depth; 2 : 3 in the code's constants |
| 15 | Japan: round orange reflectors on both edges | Kuruma News | one or more on each side in `02_mid_cruise`, orange (`probe`) |
| 16 | Japan: keep-left traffic with a kei box in the slow lane | RESEARCH M | a box-shaped car in the left lane in `02_mid_cruise` |
| 17 | Japan: two-language labels, Japanese above, Latin below and smaller | guide signs | three labels, each with both lines legible at 1:1 |
| 18 | Japan: a stop bar with 止まれ before it under hard braking near a stop | RESEARCH O | not in the four review states; shown in one extra capture (38 mph at −0.95 g) |

Key frames of the sequence: the limiter is reached at frame 29 or 30, the strobe runs 30 to 41, the gear changes at 42. Blur levels: the tool's glance view (960 × 540, sigma 1.5) and its squint view.

TJX's rejection patterns to check against named crops: (a) it looks nothing like the real car's display; (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real display does not have; (d) flat modern widgets instead of the "2027 future" bar; (e) murky, tinted or frosted glass.

### Cold look (05:19 UTC, round t7, `02_mid_cruise_cockpit.png`, its mirror and its luminance view)

Sixteen short grey bars run across the top, the middle four darker. Left: a thin grey TJX, a two-line label (Japanese over TRACK MODE), a circle with a cross, two rings, a dashed outer ring and a blue ball just off centre, "0.20 G", and two small two-line labels over 5200 and 104. Centre: a dark grey road in perspective with three lanes, white dashed lane lines and solid edge lines; the grey ends in soft vertical edges left and right and a straight cut at the bottom. On it a white car from behind and above with orange streaks in its roof glass, a red light bar and a plate with green blocks; a blue band from its nose to a smaller, darker car ahead; a white box van in the left lane with upright red lamps and a yellow plate; two small cars far ahead; orange dots on thin posts at both edges. Right: a white 4 (the tallest thing), 118 with a small grey MPH at its lower right, and R N D with D brighter. A thin grey line runs along the bottom centre with a brighter white part right of a tick. The eye goes to the white car first (the largest bright mass and the only orange), then the 4, then 118, the van and the circle. Values read: 4, 118 MPH, 0.20 G, 5200, 104, D. The cars read as lit objects; the road, the bars, the meter and the figures are flat.

Premortem: "a toy car on a grey slab: the white car out-shouts the gear, and the road is a grey panel."
- (a) nothing like the real display: the three-part layout with a chase-view car and a blue path is Tesla's (confirmed against R1 below).
- (b) materials only at 2x: the glass reflections and body shading show at full-frame scale. Not this.
- (c) one slab: **finding**, the road surface is a grey slab (sRGB 46 over the scrim) with side edges and a cut bottom.
- (d) flat widgets: **finding**, the G-meter and the shift pills are plain vector, as on the real display (Tesla's own UI is flat); the 2027 bar is carried by the scene.
- (e) murky glass: the dash has no glass.

### Cold reviewer (one run, 05:19–05:22 UTC, packet `mockups/blind/15/`, logo zone masked)

Five-second test: first "the big white 4 and 118", second "the white car with the blue strip". Recognition: "Don't know. The Japanese labels and the yellow plates point to a Japanese-market display, but I cannot name a maker or car." Blind glance read against the key: gear right on 8 of 8 frames, speed right to within 1 on 8 of 8, rpm zone right on 7 (glance_06, on the limiter in the strobe's dark phase, was read "don't know; my reading is limiter", which is right but unsure).

Its findings, each fixed (round t8) or not:
| Finding | Status |
|---|---|
| The road is a flat grey slab cut by a rectangle (sides and bottom) | Fixed: the road is a pool of light that fades on every side (`Frame::pool`, squared fall so the scrim's linear add shows no edge) |
| The car floats: no shadow, no contact with the road | Fixed: contact shadows under every car, multiplied into the road's light (nothing laid over black); the parked car stands on a small floor with its shadow |
| The blue path is an opaque, hard-edged trapezoid | Fixed: 0.24 instead of 0.42, drawn as a filled glow with a 5 px soft edge |
| The under-car glow is a flat saturated puddle wider than the car | Fixed in part: at most 0.55 of the shift colour; its shape is the Blender light pass and stays |
| glance_04: two cars superimposed (the parked view fading over the road) | Fixed: the cross-fade takes 0.12 s, not 0.45 s, so no still catches a double car |
| Speed digits ghost-doubled in 03, 04, 05 | Same cause (the 0.45 s fade), fixed |
| N is grey while the 0 under it is white | Fixed: neutral is drawn in full ink |
| The speed hangs under the right edge of the gear | Fixed: the speed and the unit are centred under the gear, as on the real display |
| Unlit R and N nearly invisible | Fixed: 0.8 of the label ink (4.3–4.9:1 in the blur gate) |
| The rev pre-fill is colourless and low in contrast | Fixed: an opaque grey (`REV_LEVEL` 0.24) that reaches 3.2–3.5:1 at cruise in both views |
| The car is a smooth, toy-like blob; the roof reads as wood grain | Not fixed in this round: the car model pass (shut lines, glass reflection scale) is the next asset job |
| The rear plate looks like placeholder pixels | Not fixed: deliberate, no plate may hold a readable number (`ASSETS.md`) |
| No P in R N D, no warnings, battery, clock | Not fixed: deliberate, Tesla's driving view shows R N D only, and the game sends no warnings, battery or clock |
| Japanese labels with MPH and HP | Not fixed: the units follow the player's setting (km/h and kW when set) |
| TRACK MODE beside public-road traffic | Not fixed: Tesla's Track Mode is the screen that shows a G-meter; the traffic is ambient scenery |
| The far end of the road stops at the top of the scene | Not fixed: it fades by depth (`Frame::fade` far term), the reviewer was not sure |
| The orange posts are unexplained | Not fixed: they are Japanese expressway edge reflectors (RESEARCH.md change N) |

### Measurements (round t8, after the fixes)

- `glance`, cockpit over the scrim (`<state>_cockpit.png`): gear 18.0–18.8:1 and speed 12.5–13.8:1 in all four states; rpm 6.4–7.0, power 5.1–6.5, R·N·D 4.6–4.9; G value 6.0–7.1:1 once drawn in full ink (it was 4.1:1 in drift before); shift line 1.5:1 at idle (dark by design: no revs), 3.5:1 at cruise, 4.3:1 strobe, 9.8:1 drift.
- `glance`, hood (`hood_<state>.png`): gear 17.7–18.5:1, speed 11.3–12.7:1, rpm 6.4–6.9, power 5.5–6.6, G 5.8–6.8:1 (value in full ink; the status figures at a 24 px cap), R·N·D 4.3–4.7, shift 1.3:1 at idle, 3.2 cruise, 3.8 strobe, 7.8 drift.
- `flash --frames 30-41`: the shift strip 3 on / 3 off at 10 Hz; worst 10-degree field yes; strip height yes. The rpm and power digits flickered at 30 Hz on the limiter: fixed with display hysteresis (1.5 steps of 50 rpm, 4 of power); what is left is the G readout falling with the data and the scenery scrolling.
- `motion`: 0 stalls; idle frames 6–9 still; spikes at 27 (the first shift stage) and 50, 53, 56 (the lane lines passing at 100 mph), each a data event.
- `stretch`: 0 px not off outside the envelope plus 40 px.
- Reference and render (`review/ref_vs_render.png`, both orders): the same arrangement of masses, info block left, road with the ego car from behind in the centre, the speed right, R N D bottom right, the power line bottom centre. Changes: a gear figure over the speed (the brief's rank order), a G-meter where Tesla has the map (no map in the packet), and a larger car (a closer chase camera so the Blender car reads at 1.4 m).
- Board against render (`review/board_before_after.png`): every board zone sits where the board puts it; the render is clearly richer than the board and the before render.

### DC answers and decision

- V1 yes (frozen 04:58, before the round was opened). V2 yes (cold look and premortem above, each pattern checked). V3 yes (one run, each finding recorded).
- V4 yes: the reviewer named the gear and speed first.
- V6 yes for gear and speed in every state and view; the other zones as listed above.
- V10 yes: `review/02_mid_cruise_cockpit_squint.png` keeps the 4, 118 and the car; labels and the logo go first.
- V13 no: the reviewer could not name Tesla ("Don't know … a Japanese-market display"). It stays no (rule 2); the layout pairing with R1 was named "the same overall arrangement".
- V14 yes for layout, geometry and colour (frozen inventory, measured above).
- V15 yes (both orders). V16 yes. V17 yes: paint, glass, rubber, lamps nameable at full frame, one overhead key. V19 NOT ASSESSED: no glass on this dash. V20 yes.

Decision: **go with changes.** Changes made (t8): the road pool and contact shadows, calmer cars, the softer path and glow, the short parked fade, centred figures, full-ink figures, the rev grey, the figure hysteresis. Still to do before the closing review: the car model pass (shut lines and the scale of the roof's lamp reflections) in the next cloud asset run.

## Closing review

Round t9 (`09ae6da`: the car images with shut lines from the second cloud run), reviewed 06:05–06:40 UTC, then one fix (the glass reflections, `c0cbd13`) from the cold reviewer's findings. The expectations frozen at the direction check were not edited.

### Cold reviewer (the second and last run, 06:11–06:13 UTC, packet `mockups/blind/16/`)

Five-second test: first "the big 4", second "118 MPH", third the road with the white car. Recognition: "Don't know. A.png shows no maker name or logo." Blind glance read against the key (`review/blind_key_16.txt`): gear right on 8 of 8, speed right to within 1 on 8 of 8, rpm zone right on 8 of 8; the shift lights as each frame shows them on 8 of 8 (glance_07 is frame 33, the strobe's dark phase, read "dark"; the reader added "from the lights alone I would have read it as low"). The glance frames were written without the logo mask, so the driven car's logo (the review telemetry's car, a Porsche) showed in them; A.png, on which the recognition question was asked, was masked.

| Finding | Status |
|---|---|
| The ego car's rear glass has "an odd brown/orange striped texture that looks like wood grain" (also found at the direction check) | Fixed (`c0cbd13`): the tunnel light card is desaturated to 0.15 in the car shots, so the glass shows soft neutral streaks (`/mnt/project-files/shots/tesla_adas/12_rear_glass_neutral_test.png`, CPU Cycles test; the cloud run and the goldens follow) |
| Unlit shift pills are mid-grey, the middle four darker; lit and unlit hard to tell apart | Not fixed: the grey is the rev fill (the only rpm graphic, as Tesla has no tach; it must reach 3:1, change C), filling from the outside in, so the dark middle four are the revs not yet reached. The stage colours replace it with hue and at 9.8:1 |
| glance_07: all lights dark at 8650 rpm reads as low revs | Not fixed: the strobe's dark phase is the token strobe (`motion.limiterStrobe`, 3 on / 3 off); the rpm figure carries the value through it |
| No tach scale and no limiter mark | Not fixed: deliberate (change C) |
| The ego car looks matte and clay-like | Not fixed: Tesla's own visualisation draws a smooth, light grey model (R3); the shut lines and lamp reflections are what the model pass added |
| The van is a toy-like box with a flat yellow plate | Not fixed: the kei box is the RESEARCH M motif; the plate may hold no readable number |
| The blue path is a flat even fill; the road plain dark grey | Not fixed: the path is Tesla's flat Autopilot band (R1, R3); the road is the pool of light from the direction check |
| The parked car looks flat next to R2 | Not fixed: R2 is lit daytime studio art; the parked car is the same night model |
| The thin line at the bottom has no label | Not fixed: Tesla's power meter carries no label either (feature 7) |
| Labels, the logo and the rpm and power figures are small and grey | Not fixed: rank 5; caps measured on the 1:1 crop at 17.7 px or more for every figure (V8) |
| A gear digit beside R N D, no P | Not fixed: deliberate (change A); P shows when parked |
| The cockpit photo behind the cluster, no frame of its own | Not fixed: change G, no screen rectangle |

### Second opinions (once each)

- `design:design-critique` (on `02_mid_cruise_crop.png`, `04_drift_braking_crop.png`, the cvd sheets and the before/after sheet): the eye goes to the 4, then 118, then the car, which is the rank order. The orange rear glass was the one element out of rank (the only warm hue in the frame at cruise, on the brightest object): fixed as above. The red tail bar reads at cruise and grows brighter under braking, which is the brake signal; position (the car) and brightness carry it without the hue. The hood static labels and the G-meter rings are brighter than the 0.2 static cap (see V11). Consistency: the shift pills, the meter, the figures and the chips share one type family and one grey ladder; the Japanese lines sit above the Latin ones everywhere.
- `design:accessibility-review` (contrast, use of colour, flashing, motion without data): contrast as V6; colour as V12 (every state reads in the luminance-only panels: the shift stages by count, the limiter by all pills lit, braking by the tail bar's brightness, drift by the path's bend and the meter's ball); flashing as V22, and the reduced-flash mode is the shared `ShiftLights` (`input.reduced_flash` passed in `update`, which holds the strip lit instead of strobing); motion: nothing moves without data (`motion`: idle frames 6–9 still, the lane lines scroll with the speed).
- `rust-review` (the dash's `mod.rs` and `scene.rs`, the kit's `preload_text`; clippy with `-W clippy::pedantic` run): no must-fix and no should-fix. Every float-to-index cast is clamped or guarded before it indexes (`ego_cell` clamps to the atlas, `PeakTrace::record` takes `rem_euclid` and `% PEAK_BINS`, the traffic column takes `min`); no `unwrap`, `expect` or `panic!` outside tests; the remaining pedantic lints are precision casts of small counts to `f32`, single-letter geometry names and one `unused_self` on `draw_surface` (kept so every `draw_*` has the same shape). `Fonts::preload_text` is documented, allocates only when the cluster is built and changes no existing behaviour.

### Visual review V1–V26

- V1 yes: frozen 04:58 UTC, unchanged.
- V2 yes: cold look and premortem at the direction check; for t9 each rejection pattern was checked again on `02_mid_cruise_crop.png`: (a) the layout pairing with R1 holds; (b) the shut lines and glass show at 1:1, no texture needs 2x; (c) the road is a pool of light, not a slab; (d) the scene carries the 2027 bar; (e) no glass panels.
- V3 yes: one run, every finding in the table above.
- V4 yes: "the big 4" first, "118 MPH" second.
- V5 yes: gear, speed and rpm zone right on all 8 frames; the shift lights as the frames show them.
- V6 yes: gear 17.7–18.8:1 and speed 11.3–13.8:1 in all four states of both views; rpm, power, G and R·N·D 4.3:1 or more; the lit shift pills 3.2:1 or more. Two zones below 3:1 carry no information at the time: the shift line at idle (1.5 cockpit, 1.3 hood; no revs to show) and the power line at idle (1.2; zero power).
- V7 yes: in the squint view each stage (2, 5, all pills) and the strobe's on and off frames differ from each other and from idle.
- V8 yes: from the `Geom` tables, confirmed on 1:1 crops: the gear 233 px (cockpit) and 205 px (hood) cap, the speed 108 / 90 px, the G, rpm and power figures 24 px or more, labels 17.7 px or more; strokes 1 px or more at full contrast.
- V9 yes: after the hysteresis, `flash --frames 30-41` marks no digit.
- V10 yes: `review/02_mid_cruise_cockpit_squint.png`: labels and the logo go first, the gear and the speed last.
- V11 yes with explanations: `static` over the four states (cockpit, round t9): every area over the 0.2 cap is live content that lands on the same pixels in all four states (the gear digits N, 4, 5, 3 overlapping, the rpm figure's trailing 00, the speed digits, and the idle parked car overlapping the cruising ego car), not structure. Hood: the logo (0.30, the shared `logo_zone`, the kit's label ink), the two-language labels (0.29–0.31) and the G-meter rings (0.31) are over the cap. Explained, not changed: they use the palette's `ink_label`, which they need for 4.5:1 over the cockpit scrim (0.2 would give 4.5:1 over black but not over the scrim's lift), and they are well under the live data (p95 0.93). Large near-white static areas: none.
- V12 yes: `04_drift_braking_cockpit_cvd_sheet.png`, `03_redline_strobe_cockpit_cvd_sheet.png`, `hood_04_drift_braking_cvd_sheet.png`: every colour state reads in every panel by count, position or brightness; red only on the tail lamps (brighter when braking) and the rev line's limiter end.
- V13 no: the reviewer could not name Tesla in either run ("Don't know", both times). It cannot be fixed afterwards and stays no.
- V14 yes: the frozen inventory was measured at the direction check; t9 changed only the car images (shut lines, dimmer glass), so lines 3, 9 and 11 were checked again on the 1:1 crop and still match.
- V15 yes, V16 yes (direction check, unchanged since).
- V17 yes: paint, glass, rubber, the lit lamps nameable at full frame, one overhead key.
- V18 yes for the checks that apply (car paint, glass, lamps); no bezel, crystal, carbon or Liquid Glass on this dash.
- V19 NOT ASSESSED: no glass pieces on this dash.
- V20 yes: `stretch` 0 px not off outside the envelope plus 40 px; corners off; no banding in the road's fall (squared).
- V21 yes: shift stages, strobe, gear and digits change on the data's frame; nothing trails or overshoots; `motion` reports 0 stalls.
- V22 yes: worst 10-degree field yes, strip height yes, the shift strip 3 on / 3 off at 10 Hz; the other flashing area is the G readout falling with the data and the scenery (area 3).
- V23 yes: frames 39–50 at 1:1 (`/tmp` sheet, recorded here): 3 on 39–41, 4 from 42 (the data changes at 42), no blank or doubled frame, no morph.
- V24 yes: the `motion` spikes are frame 27 (the first shift stage) and 50, 53, 56 (lane lines passing at 93 mph); no shimmer, crawl or bloom pop on the 1:1 frames.
- V25 yes: `review/before_after/02_mid_cruise_cockpit_vs_TeslaAdas-02_mid_cruise_cockpit_side.png`: the before's main failure (a generic pod HUD with nothing of Tesla's layout) is gone; nothing it did better is lost (its gear is still the biggest figure; its blind-spot arcs and lock box are removed on purpose, change E).
- V26 yes: the eight goldens were made on the PC's card from the third run's car images (`goldens_v4`, the golden test passes there with them) and each was opened here before it was committed: idle (the parked car three-quarter front, N, 0 MPH, the line dark), cruise (the car from behind on the blue path, the kei van left, three cars ahead, 4 and 118), redline (all 16 pills violet, the violet glow under the car, 5 and 152), drift braking (the road bending right, the car turned with its brake bar lit, the outer three pills green each side, 3, 74 and 1.12 G), and the four hood versions. Every difference from the previous goldens (the old build's) is the rebuild.

### After the fix (round t10, the third run's car images)

`docs/handover-shots/cluster-tesla_adas-after-02_mid_cruise.png`: the rear and roof glass shows soft grey streaks with no orange; nothing else changed (the code is the same, the goldens are new). Re-checked on the 1:1 crop: the car's place, size and lamps (inventory lines 3, 9, 11) unchanged.

### GPU and CPU

- `preview-export --cluster TeslaAdas` on the PC's card (06:53–06:55 UTC, gpu.lock held): cockpit 01 0.081 ms, 02 0.222, 03 0.321, 04 0.354; hood 01 0.268, 02 0.287, 03 0.401, 04 0.434; the 60-frame sequence p50 0.316 ms, p99 0.327 ms. Every figure is under the 0.5 ms per cluster.
- Live bench on the PC (07:1x UTC, through `gpu-lock.exe run`, release build, 60 s each, the stand-in window in front, `mock-udp --scenario bmw_m4_cs` at 60 Hz, `--bench-out` to a scratch folder so the tracked `docs/perf/engine.md` stayed as it is; no panic, warning or error in either run):

  | | Frames | CPU per frame p50 / p99 / max | Receive to present p50 / p99 | Draw calls per frame |
  |---|---|---|---|---|
  | TeslaAdas | 8059 (134 fps, 3267 packets) | 0.729 / 1.317 / 16.25 ms | 0.759 / 1.880 ms | 19.3 |
  | Scaffold (AbyssalShearBloom) | 8189 (136 fps, 3496 packets) | 0.582 / 0.796 / 1.396 ms | 0.594 / 1.227 ms | 11.0 |

  The dash's own share (criterion 6): 0.729 − 0.582 = **0.147 ms p50, under the 0.3 ms limit.** Receive to present meets one display frame (3.33 ms) at p99.
  The first run's app total, 1.317 ms CPU p99 with a 16 ms max, was over the 1 ms target, so the bench was run again (07:30–07:34 UTC, three runs back to back in one `gpu-lock.exe run`, same setup):

  | | Frames | CPU per frame p50 / p99 / max | Receive to present p50 / p99 | Draw calls | p99: upload+record+submit / swapchain, encoder, finish / `Queue::submit` / present |
  |---|---|---|---|---|---|
  | TeslaAdas, run 2 | 8086 | 0.675 / 0.894 / 1.444 ms | 0.690 / 1.435 ms | 19.3 | 0.540 / 0.360 / 0.210 / 0.326 ms |
  | BoschDdu (a finished dash whose lights also bloom) | 8051 | 0.623 / 0.829 / 1.371 ms | 0.629 / 1.298 ms | 13.5 | 0.504 / 0.335 / 0.197 / 0.306 ms |
  | TeslaAdas, run 3 | 8332 | 0.664 / 0.850 / 1.341 ms | 0.675 / 1.426 ms | 19.3 | 0.510 / 0.344 / 0.203 / 0.315 ms |

  **The app total is under 1 ms p99 in both repeat runs (0.894 and 0.850 ms).** The first run's p99 came from a few slow present calls (present max 15.36 ms there, 0.83 and 0.77 ms in the repeats). TeslaAdas costs 0.04–0.05 ms more than BoschDdu at p50 and p99; its own share against the scaffold is 0.082–0.093 ms p50 in the repeats. Draw calls: `preview-export` shows 10 at cruise (fewer than the scaffold) and 20–21 only in the states where something blooms; the difference is the kit's bloom dispatches, which run once per frame whenever any layer blooms (the shared shift lights included), so merging the car atlases would save almost nothing and was not done. `AppConfig` has no switch that turns bloom off, so no bloom-off run was made.

### Assets, masters and cloud spend

- Three cloud runs on the RTX 4090 lane: the first (all jobs, the tunnel plate) $0.129; the second (cars with shut lines) about $0.138; the third (cars with the neutral card) about $0.124. This thread's lane spend is $0.391; the lane total after the third run was $2.64 of the $50. The $40 warning never printed. Every pod was taken down after its batch.
- Masters and runtime copies in the Drive folder `tesla_adas/` (`runtime_v4/`, `goldens_v4/`, `renders_v4/`), listed in `assets/clusters/tesla_adas/ASSETS.md` with each committed file's SHA-256; every file fetched to the cloud was checked against the SHA-256 the PC reported before it was committed.

### Board and code (TJX, 28 Nov 07:09: Penpot first, then code)

The board came first (page `07 · TeslaAdas`, before any of this build's code), and the code has not drifted from it: the direction check laid the board over the render and every zone sat where the board puts it (V16). After the direction check nothing in the layout changed; the closing review's one change is inside the Blender car images (the light card's saturation), which the board does not draw. So the board needed no update before the change.

### Before and after

`docs/handover-shots/cluster-tesla_adas-before-02_mid_cruise.png` (the dash on `main` before the rebuild) against `cluster-tesla_adas-after-02_mid_cruise.png`, with the other three states and the hood cruise view beside them (`after-01_idle`, `after-03_redline_strobe`, `after-04_drift_braking`, `after-hood_02_mid_cruise`).

## What was built
- `crates/fh6-clusters/src/clusters/tesla_adas/`
  - `mod.rs`: the zone tables (`Geom`, one for the cockpit and one for the hood), the cluster (`update`, `draw`), the shift line (16 thin pills along the top that fill with revs from the outside in, take the palette's stage colours and strobe at the shift point), the left card (logo zone, the two-language Track Mode label, the G-meter with its peak history in 24 direction bins, the rpm and power figures), the right column (gear, speed with its unit, the R · N · D row), the centre-zero power line, and the hood view's legibility halos. 6 unit tests (the layout inside both envelopes with the zone minimums, the halos per view, the rev fill, the shift stages, the peak history, the Japanese glyphs preloaded).
  - `scene.rs`: the road scene. One fixed chase camera (3.2 m up, 7.2 m behind, 16° down); a three-lane expressway with lane lines 8 m on and 12 m off, solid edge lines and an orange reflector post every 50 m on both sides; curvature from the measured lateral acceleration with a speed floor; the path band; under braking the slow-down dashes and, at the predicted stop point, a stop bar with 止まれ written before it; four seeded road users that keep left; the ego car from an 11-cell yaw atlas with its glow and brake-lamp layers; the parked view with hysteresis; a procedural fallback for every image. 8 unit tests (curvature, the stop point, the ego's place against the recipe, the yaw cell, the seeded traffic, where the road text shows, the first frame and the parked hysteresis, the road's wrap).
  - `RESEARCH.md`, `README.md`, this file. `spec.md` is the catalog brief and is unchanged.
- `tools/asset-forge/src/recipes/tesla_adas.rs`: the jobs `font` (checks the vendored font subset against its recorded hash), `night` (the ComfyUI plate, with everything but the lights taken to black), `ego` (11 yaws, each with a glow and a brake-lamp pass), `parked` and `traffic` (three bodies at four elevations and five headings), the atlases and this dash's `ASSETS.md`. 3 unit tests.
- `assets/clusters/tesla_adas/`: eight PNGs, the Noto Sans JP subset with its licence, and `ASSETS.md`. The old build's ten images are removed.
- `crates/fh6-kit/src/text.rs`: `Fonts::preload_text` (8 lines; see "Shared places touched").
- `docs/TELEMETRY_USAGE_MATRIX.md`: this dash's row.
- `tests/golden/images/TeslaAdas_*.png`: the eight goldens (four states, cockpit and hood).
- `docs/handover-shots/cluster-tesla_adas-*.png`: the before and after pair and the other states (the old build's shots are removed; they are in git history).

## Telemetry
Every field the dash reads is one the golden packets cover, or is derived from one:
- **Read as they are (through `fh6_kit::Signals`):** rpm (current, limiter, idle), gear, speed, brake, the race flag.
- **Read from the packet:** power (W).
- **Derived:** lateral and longitudinal G from acceleration (the kit's smoothing); the road's curvature from lateral acceleration over the square of the speed, with a 3 m/s floor; the stop point from speed and measured deceleration (`v² / 2a`, with a 2 m/s² floor, shown up to 80 m); the shift stages from the limiter and `ShiftPointPct`; the power line's full scale from the highest power seen (110 kW floor).
- **Velocity:** read through the kit's body slip angle (`atan2(velocity_x, velocity_z)`), which turns the ego car's yaw cell. The matrix row lists `velocity` (gates G2 and G3 pass; with the dash's images present the gate sees the yaw change, so the no-sprite fallback car turns too and the dash reads the same inputs with or without assets).
- **From the full Data Out list (rework of 2026-11-28, below):** the driving line and the AI brake difference, the rumble-strip and puddle flags, combined tyre slip, the lap data. Their offsets follow the official page's field order and are unconfirmed until real-game check C.
- **Not read:** yaw rate, steer, throttle, suspension, wheel speeds, slip ratio, surface rumble, tyre temperature (no unit on the page), torque, fuel, world position.
- **Not from telemetry at all:** the traffic. It is a seeded ambient simulation (`scene.rs`, `Traffic`), it carries no distance or closing-speed label, and nothing in the scene is drawn as a detection.

## Rework on the full telemetry (2026-11-28, thread "Rework Tesla ADAS dash for real telemetry")

TJS, 28 Nov 10:52: "Yeah that can be reweworked ASAP to be accurate". The decoder on `main` (60c99c9) now reads all 88 Data Out fields; this rework puts them in the scene where they fit and keeps the parts the game sends nothing for (traffic, lanes) as the seeded scenery they were.

| Field(s) | What it drives | Where |
|---|---|---|
| `normalized_driving_line` | the blue path leaves the nose straight and merges onto the game's driving line over 18 m (full scale 127 = one lane, 3.5 m, our display scale) | `scene.rs` `draw_path`, `driving_line_offset` |
| `normalized_ai_brake_difference` | the path ahead turns caution, then warning red, and brightens, as Forza's braking line does; full at 96 of 127 | `draw_path`, `brake_cue` |
| `wheel_on_rumble_strip_*` | the ego lane's line on that side (left wheels: left) turns solid red beside the car, its light falling off over 29 m (Tesla's lane departure warning); held 0.35 s so kerb hits in a row read as one warning | `draw_departure` |
| `wheel_in_puddle_*` | each side's wheels in a puddle start or lengthen a wet patch (a soft sheen ellipse with the overhead lights' streak) that then scrolls back with the road; 8 patches kept | `update_puddles`, `draw_puddles` |
| `tire_combined_slip_*` | past 1.0 (Forza's normalised edge of grip) a glow under that wheel, caution to warning by 1.6, placed at the wheel (Model S track and wheelbase) and turned with the car's yaw | `draw_grip`, `grip_level` |
| `current_race_time`, `current_lap`, `lap_number`, `race_position` | while a race runs (any of the first three non-zero) the card's TRACK MODE label becomes Track Mode's lap timer: the lap time (m:ss.t, cap 20) and "LAP n · Pp" under it | `mod.rs` `LapTimer`, `draw_lap` |

Unverified and named so check C can flip them: `LINE_SIGN` (positive reading taken as the car right of the line), `BRAKE_CUE_SIGN` (positive taken as the game braking harder than the driver), `LINE_FULL_M`, and the lap number counting from 0 on the first lap (Forza Motorsport's convention). Times are taken in seconds.

**Unchanged at zero.** With every new field at zero the frame is bit-identical to the merged build: the scene fingerprints of the five gate scenarios (cruise, idle, redline, drift braking, lift off) at the gate's four check frames, in both views, are the same before and after (40 of 40). The goldens are not rewritten.

**Speed.** CPU, no-op kit in the cloud container, update plus draw, 3000 frames after a warm-up: 26.5 to 28.0 µs with the new fields at zero, 30.5 to 30.6 µs with all of them active, so about 3 to 4 µs added to the 0.147 ms share measured on the PC. GPU, lavapipe A/B in the container (`preview-export`, every new feature forced on, two runs each): +1 draw and +29 instances, and a frame-time change inside lavapipe's run-to-run noise (cockpit drift 61.4 / 58.8 ms off against 70.0 / 62.9 ms on; hood drift 68.7 / 81.7 against 77.6 / 73.8; cruise unchanged). On the card the worst state was hood drift at 0.434 ms; the PC's bench confirms the new figure at check C.

**Renders** (lavapipe, `mockups/TeslaAdas/telemetry/`, both views): `05_driving_line`, `06_brake_cue`, `07_rumble_left`, `08b_puddle_in`, `08_puddles`, `09_grip_drift`, `10_race_lap`; the sheet is `docs/handover-shots/cluster-tesla_adas-telemetry-rework.png`.

**No new assets.** Penpot, ComfyUI and Blender work waits for the 4K library's LIBRARY DONE (CLUSTER_PROMPT.md); nothing here needed any.

## Changes from the real display and from the brief
From `RESEARCH.md`, "Deliberate changes and why" and "Japan-inspired":
- A. The gear leads and the speed follows (Tesla shows the speed biggest and no gear).
- B. A shift light Tesla does not have, drawn as 16 thin pills where Tesla's telltale row sits.
- C. Tesla's centre-zero power meter is the power display; the revs are carried by the shift line.
- D. Only verified data: no speed limit, cruise or Autopilot state, map, range, telltales or turn signals.
- E. The traffic is a seeded simulation with no distance labels.
- F. The power meter's left side fills green only when the packet reports negative power.
- G. No lit screen rectangle: every part floats on true black.
- H. One generic ego coupe, rendered in Blender; the under-car glow shows the shift colour and the drift, not an Autopilot state; the camera is fixed.
- I. No start-up show.
- J. The driven car's logo, small and grey, at the top of the left card.
- K. Track Mode's G-meter fills the card that holds Tesla's map.
- L. Noto Sans (OFL) for every figure and label.
- M. (TJX, 28 Nov 2026: "make it japan inspired as we are in horizon 6") The road is a three-lane Japanese expressway at night that keeps left, with NEXCO lane lines, orange reflector posts, a kei box car in the slow lane, and a tunnel's ceiling lamps mirrored, neutral, in the car's glass.
- N. Two-language labels, Japanese above and Latin below.
- O. Braking is written on the road: slow-down dashes and a stop bar with 止まれ.
- P. Left out on purpose: speed-limit roundels, route numbers, destination names and tourist motifs.

Against the catalog brief (`spec.md`): the brief's ten pods, gear pill with neighbour ghosts, SLIP / LOCK / LIM chips, blind-spot arcs, lead-car lock boxes, glTF mesh cars, suspension-driven ride height and per-corner detail are not built. The real display has none of them (`CLUSTER_PROMPT.md`: "Where the brief invents a concept the real car doesn't have, follow the real car"), and the lock boxes and arcs would present simulated traffic as detections. AGENTS.md's Agent 3D text still describes that older concept.

## Shared places touched
- `crates/fh6-kit/src/text.rs`: one new method, `Fonts::preload_text(font, text)` (8 lines, no change to existing behaviour). The kit preloads a fixed Latin set per face; a face for another script would rasterise its glyphs on the first frame it is drawn, which allocates in a warm frame (the warm-frame allocation gate) and shows the label one frame late. The method lets a dash name the characters it draws when it is built. It is a kit change made by a dash thread, so the coordinator should look at it at the merge.
- `docs/TELEMETRY_USAGE_MATRIX.md`: this dash's row only.
- `docs/handover-shots/cluster-tesla_adas-*.png`.
- Nothing in `common/`, the registry, the catalog table, the hero library, `docs/specs/optics.md`, `docs/PROGRESS.md` or the shared ledgers (`assets/ASSETS.md`, `assets/assets.json`, `assets/forge.lock.json`, `assets/contact-sheet.html`), which every build rewrites and this thread restored each time. The asset-forge registration lines for the `tesla_adas` set were already there from the old build.

## Requests for shared code
1. **OSD: a label and a switch for simulated traffic.** The road users are an ambient simulation. The OSD should say so beside this dash ("simulated traffic") and offer a switch that turns them off; the dash would read it as a config extra key. Until then the handover and `README.md` say it.
2. **`common::adas_road`:** this dash now uses four of its constants (`EGO_Z`, `LANE_WIDTH`, `MAX_DT`, `MIN_CURVE_SPEED`) and none of its functions. If no other ADAS dash takes the rest, the module can shrink to those constants.
3. **A perspective image paint in the kit.** Road text (止まれ) is drawn as glyphs scaled per character by depth, which reads at the sizes shown but is not a true ground projection. A paint that maps an image or a text run through a homography would make road markings exact for every road-scene dash.
4. **The 8-bit premultiplied resolve and dark overlays.** `post.wgsl`'s `fs_resolve` writes `encode(covered / a) * a`, which is right for what DWM blends, but it means black with alpha laid over faint content makes that content brighter on screen. A note in `docs/kit/README.md` would save the next dash the search (see "What sibling dashes must know").
5. **asset-forge: light-only plates.** `lights_only` (keep an image's light, take the rest to black, keep each pixel's hue) lives in this recipe; any dash that hangs a ComfyUI plate as a light card needs the same step, so it could move to `img`.
6. **AGENTS.md, Agent 3D:** its text describes the old concept (lock boxes, proximity arcs, a gear pill, brand-tinted ego). It should follow the rebuilt dash, or say that the catalog and `RESEARCH.md` lead.
7. **Device folder tools for cloud threads:** this thread's session was never offered the device folder tools, so it could not stage PC files. The route that worked, every time: the PC's Remote Control session copies the files into the dash's Drive masters folder, the cloud thread downloads them with the Drive connector and checks each SHA-256 before committing.

## What sibling dashes must know
- **Judge the cockpit view on `<state>_cockpit.png`, never on the over-black `<state>.png`.** Over black the resolve's display-space alpha keeps a 3 % ink surface at sRGB 7; in the cockpit composite the opaque scrim sits under the cluster and everything adds linearly, so the same surface shows at sRGB 46 and a lane line goes from 154 to 199. A faint layer that looks right over black is a grey slab in the real view. Fade such a layer with an eased (squared) fall, because a straight ramp reads as an edge once added linearly.
- **Contact shadows go in with the multiply blend.** Multiplied into the road's light they take light only where there is some and lay nothing over black, so they cannot make the box described next.
- **Do not bake a shadow or any black-with-alpha layer into a sprite that sits over faint content.** The kit's 8-bit resolve treats alpha in display space, so a soft black shadow over a 3 % road surface shows as a lighter box round the car. Render sprites without a shadow catcher (a hidden ground plane still bounces light), and keep a hood view's halo and a faint surface from overlapping.
- **A ComfyUI plate can light a Blender scene instead of being drawn.** Hung over the car as an emission card that the camera does not see, it puts the plate's lights into glass and paint. Reflections of a card only reach a near-horizontal roof when the lights sit almost overhead; lights off to the side miss it.
- **SDXL draws a tunnel in perspective whatever the prompt says about a flat view.** Plan the card for the picture SDXL gives, or prompt for a subject that is flat by nature.
- **A second script needs its glyphs preloaded** (`Fonts::preload_text`) and a font subset: Noto Sans JP Medium cut down to the characters drawn is 160 KB; the full face is over 5 MB. Keep the list of characters in one constant and test that every string drawn is inside it.
- **Ghost digits behind a proportional-looking number misread** ("74" over ghost 8s read as "874" at a glance in this family). This family draws no ghost digits and hangs the unit from the field's right edge.
- **Seeded traffic must open the scene in a believable order** (a slow car just ahead on the left, a passing car further ahead on the right), or the first frame shows cars stacked at one depth.
- **Fade lane paint per piece with a gradient**, not per line: a long edge line with one alpha pops at the far end as the road bends.
- **Shut lines can be cut in the shader.** A lofted body has no panel gaps; bands in object space (across the top for the boot lid and bonnet, down the sides for the doors, across the tail for the bumper line), dark and with no clear coat, read as gaps at about a pixel wide in the dash.
- **Hold the small figures through the limiter.** The limiter swings the revs and the power every frame; a figure that follows each step flickers its last digits at 30 Hz (the flash audit marks it). Display hysteresis of 1.5 steps fixes it without lag the eye sees.
- **The cloud container can render Blender previews on the CPU** (bpy 5.3.0 in a venv) and the dash on a software GPU, which is enough to iterate on a recipe before the paid run; ComfyUI is not available there, so the plate needs a stand-in until the cloud run.

## Penpot

File `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, page `07 · TeslaAdas` `b568ed3f-64cd-806f-8008-fd974581b17e`. Boards (exported at 3840 × 2160 to the Drive folder `tesla_adas/boards/`):

| Board | ID |
|---|---|
| cockpit 1 idle | `347e5ae2-9e0b-80b1-8009-0355cb14872f` |
| cockpit 2 cruise | `347e5ae2-9e0b-80b1-8009-035605af3009` |
| cockpit 3 redline | `347e5ae2-9e0b-80b1-8009-03562540c162` |
| cockpit 4 drift-braking | `347e5ae2-9e0b-80b1-8009-0356659edb39` |
| hood 1 idle | `347e5ae2-9e0b-80b1-8009-035681f73b9f` |
| hood 2 cruise | `347e5ae2-9e0b-80b1-8009-0356855632f5` |
| hood 3 redline | `347e5ae2-9e0b-80b1-8009-03568907d954` |
| hood 4 drift-braking | `347e5ae2-9e0b-80b1-8009-03568caa256e` |
