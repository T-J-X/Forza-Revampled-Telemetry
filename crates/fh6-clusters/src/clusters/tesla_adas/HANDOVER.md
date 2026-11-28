# HANDOVER: Dash · TeslaAdas (catalog row 15, build #5, ADAS & HUD pattern-setter)

**Thread / branch:** "Rebuild the Tesla ADAS dash", `claude/project-thread-r828zb`  **Date:** 2026-11-28  **Result:** ◐ in progress (this file is filled as the work goes; the old build's handover is in git history).

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
- `docs/handover-shots/cluster-tesla_adas-*.png`: the before and after pair and the review crops.

## Telemetry
Every field the dash reads is one the golden packets cover, or is derived from one:
- **Read as they are (through `fh6_kit::Signals`):** rpm (current, limiter, idle), gear, speed, brake, the race flag.
- **Read from the packet:** power (W).
- **Derived:** lateral and longitudinal G from acceleration (the kit's smoothing); the road's curvature from lateral acceleration over the square of the speed, with a 3 m/s floor; the stop point from speed and measured deceleration (`v² / 2a`, with a 2 m/s² floor, shown up to 80 m); the shift stages from the limiter and `ShiftPointPct`; the power line's full scale from the highest power seen (110 kW floor).
- **Velocity:** read through the kit's body slip angle (`atan2(velocity_x, velocity_z)`), which turns the ego car's yaw cell. The matrix row lists `velocity` (gates G2 and G3 pass; with the dash's images present the gate sees the yaw change, so the no-sprite fallback car turns too and the dash reads the same inputs with or without assets).
- **Not read:** yaw rate, steer, throttle, suspension, wheel speeds, world position and every other offset the golden packets do not cover.
- **Not from telemetry at all:** the traffic. It is a seeded ambient simulation (`scene.rs`, `Traffic`), it carries no distance or closing-speed label, and nothing in the scene is drawn as a detection.

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
- M. (TJX, 28 Nov 2026: "make it japan inspired as we are in horizon 6") The road is a three-lane Japanese expressway at night that keeps left, with NEXCO lane lines, orange reflector posts, a kei box car in the slow lane, and a tunnel's ceiling lamps mirrored in the car's glass.
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
