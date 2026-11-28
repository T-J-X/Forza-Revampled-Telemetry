# HANDOVER: Phase 7 · Vision Spatial Glass (catalog row 2, build #10)

Thread "Build VisionSpatialGlass dash", branch `claude/project-thread-cx5bbu`, PR #20. Draft in progress: sections fill as the steps finish.

Penpot: page `07 · VisionSpatialGlass` (id `347e5ae2-9e0b-80b1-8009-03925c3ac4cb`); boards 01 idle `…0395f8a50672`, 02 mid cruise `…0395fb624082`, 03 redline strobe `…0395fe127365`, 04 drift braking `…039600f1a577`, 05 motion notes `…03951ba3137e`, 06 ideas `…0396040b493e` (every id starts `347e5ae2-9e0b-80b1-8009-`). Exports on Drive `vision_spatial_glass/penpot_v2/`.

## Exit criteria

## What was built
A frameless spatial stack in a dim visionOS sky (`mod.rs` zone table): a 16-ring rev tunnel that fills back to front, a tilting horizon with a floor glow and clouds, two toed-in glass wings carrying the rpm numerals and labels, a clear front pane with the gear and speed, a glass attitude puck and a 15-pod glass blade. What the board changed from the brief: `RESEARCH.md` section 9.

## Telemetry
`race_on`, `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `brake`, `g_lat`, `g_long`, `velocity` (the matrix row; `dash_gates` G2/G3 pass at e794797). Pitch, roll, angular velocity and tyre slip are not read (unverified); yaw is derived as lateral G over speed.

## Direction check

### Frozen expectations (written 28 Nov 08:58 UTC, before the direction-check render round)
The first-build renders (88d171d to e794797) were opened for the first-build post and the floor fix; the direction check runs on the next round, rendered with the cabin environment, which is not yet opened.

Rank order: 1 gear (white, cap 200, centre of the front pane) and the 15-pod blade (top, frontmost); 2 speed (cap 88, under the gear on the same pane); 3 the lit rev tunnel; 4 the puck (bead, caution arcs, halo) and the wing numerals; 5 labels, logo, unlit rings, sky.

Feature inventory, benchmark (a visionOS window with ornaments, RESEARCH.md section 2) and board → tolerance:
- F1 frameless glass panes floating over black, no backing slab → no panel behind the stack; black between the pieces except the sky and tunnel
- F2 depth order back to front: sky, tunnel, wings, pane, puck, blade → each later piece drawn over the earlier; overlap of pane and wings at most 12% of the wing area
- F3 the front pane holds every digit, untilted → gear and speed inside the pane's flat middle (inset 24)
- F4 wings toed in, far (inner) edges shorter → inner half height 95 ±4, outer 120 ±8 (with yaw toe)
- F5 the blade an ornament floating above the window, 15 pods, outside-in → 15 pods on 69.3 px centres ±1, lit from the outer pods first
- F6 rev tunnel: 16 rings to a vanishing point at (0, −45) behind the pane; fill from the back → rings lit in index order; ring 15's half size 700 × 270 ±2
- F7 shift colour: rings past the shift point and the whole lit tunnel at the limiter; the wings' outer slab edges steady shift colour at the limiter → hue within 20° of `palette.shift`
- F8 sky: horizon through the vanishing point, tilting 4°/g, floor glow below, clouds above → all under 10% luminance; no hard rectangle edge
- F9 glass clear sapphire, thin rims, no tint or frost (optics.md) → black stays black through the glass; rims at most ~4 px
- F10 puck: bead moves with felt G (40 px/g, at most 40), 3 caution arcs from brake 10/40/70%, halo from body slip → bead offset in `04_drift_braking` towards the felt force; arcs lit per the state's brake
- F11 one light on every rim, swinging with lateral G → highlights move between `02_mid_cruise` and `04_drift_braking`
- F12 wing numerals: one per 1000 rpm inside its ring, red from the shift point, clear of the pane → no numeral within 30 px of the pane
- Key frames: the limiter at frame 29 or 30; blade strobe 30 to 41 with the tunnel steady; gear change at 42 with the Z-hop glints (wings 42, pane ~44, blade ~47 at 60 fps).
- TJS rejection patterns to check: (a) not like a visionOS window; (b) materials only at zoom; (c) one slab with blobs; (d) flat widgets, not the 2027 bar; (e) murky, tinted or frosted glass; and his own words on this dash: "This is ass" on the flat outline draft (flatness, no depth).

### Tolerance changes (logged before re-checking)
- 09:20 UTC, F2: "overlap of pane and wings at most 12%" became "no overlap": V19 says glass never sits on glass, and the cold reviewer saw the left wing "cut by the central box". The wings' inner tips moved from x ±250 to ±296.
- 09:20 UTC, F12: the right wing's numerals keep "RPM ×1000" (not the speed unit, which moved next to the speed): the reviewer read "MPH 6 7 8" as a nonsense speed scale.

### Cold look (09:01 UTC, `02_mid_cruise_cockpit.png` at full frame, its mirror and its luminance view; before the board or notes were opened)
A wide, low stack sits in the steering-column gap over the dark cockpit. The eye goes to the white "4" first, then "118" under it, then the blue-lit rings round the central pane, then the slim bar of grey pods along the top. The central pane is a rounded rectangle of clear glass with a bright upper rim; the two side pills are glass with rim highlights from the cabin and carry grey "8 7 6" / "6 7 8" and small grey labels. Behind everything, nested rounded rectangles recede to the centre: two near the pane are blue, the rest faint grey. A small glass ball with a blue light sits under the pane. The luminance view keeps the same order: gear, speed, the lit rings, pods, labels. The glass reads as lit objects (rims, reflections); the faint outer rings read flat. Likeliest TJS rejection: (c) the outermost unlit ring reads as a frame round a slab. Checked on `review/02_mid_cruise_cockpit_crop_1160_1475_1520x640.png`: measured at 8 of 255 at most, then faded further towards the driver (`RING_IDLE_FADE`). (a) visionOS: floating clear panes and an ornament bar, yes; (b) no textures; (d) not flat widgets: the depth tunnel; (e) the glass is clear over black (`stretch`).

### Measurements (09:01 to 09:30 UTC, lavapipe, renders at f38e0ff)
- `glance`, gear 7:1 and speed 4.5:1 over `<state>_cockpit.png`: idle 12.5 / 9.2, mid cruise 15.0 / 9.7, redline 15.0 / 6.9, drift 15.4 / 5.4, all yes. Lit pods 3.1:1 (redline, strobe on) and 9.9:1 (drift) against 3:1, yes. The wing numerals measure 2.3 to 3.3:1: they are scale labels at the static cap of 19% (tokens `staticLumaMaxPct`), which by the 0.05 offset of the contrast formula cannot reach 4.5:1 over a dark of 0.004; logged as my call, and they now light in their ring's colour once the rpm passes them.
- `flash --frames 30-41`: only the 15 pods flash, each 10 Hz, 3 on / 3 off; worst 10-degree field yes; strip height 35 px against 42, yes; 3 to 6 Hz and the one-second count NOT ASSESSED (12-frame hold).
- `motion`: no stalls, no spikes; idle frames 1 to 9 move (the 200 ms show materialise, expected).
- `static`: the wing labels, numerals and logo measured 0.21 to 0.225 (static_ink assumed glass transmission, but they are drawn over the glass); fixed to 0.19 at 1.0 transmission.
- `stretch` on `02_mid_cruise.png`: 0 px not off outside the envelope plus 40 px.

### Cold reviewer (one run, 09:12 UTC, seed 1101, logo zone masked; R1 Apple Newsroom visionOS App Store window with its tab-bar ornament, R2 Apple Newsroom WWDC25 Liquid Glass hero)
- Q1: "First: the gear "4". Second: the speed "118". Third: the nested blue tunnel of rectangles, with the glass pills close behind it." Purpose: gear and speed with shift lights; "I saw no moving pointer or bar on them, so I don't know how they show a value."
- Q2: "don't know".
- Q3 (blind read, key `review/blind_key_1101.txt`): gear right on 8 of 8; speed within 1 on 8 of 8 (92/92.5, 75/75.4, 55/54.6, 0, 51/50.8, 0, 83/82.7, 97/96.9); shift lights right on 8 of 8 (limiter all lit; frames 22, 24, 51 partly lit; idle and early pull dark); rpm zone "don't know" on 7 of 8 (limiter named from the lights only).
- Q4: R1 and R2 "different", both orders; "Neither photo shows a real car display."
- Q5 findings and what was done: (1) the wing pills give no reading, "MPH 6 7 8" makes no sense → fixed: numerals light as the rpm passes them, the head ring of the tunnel burns brighter and wider, both wings say "RPM ×1000", the unit moved beside the speed (f38e0ff). (2) rpm zone unreadable → same fix; to be re-checked by the closing review's blind read (V5 is not asked again at this point). (3) limiter pods violet like the glowing frame → not fixed: the shift colour is the family's limiter colour, and the tunnel holding it is the board's signature moment. (4) nested rectangles busy behind the speed → partly fixed: the unlit tunnel fades towards the driver and the core over the speed is 0.92. (5) speed thinner than the gear → not fixed: rank 2 under rank 1 by design. (6) the idle "0" low and right of the N → not fixed: three tabular cells keep the digits still; the N is the shared `HeroGear` neutral grey. (7) the dark pod bar barely visible → not fixed: unlit pods stay dark by the family rule. (8) labels small and grey → not fixed: the static cap. (9) the left pill cut by the central box → fixed (wings clear of the pane). (10) cockpit photo at the frame edges → the review composite, not the dash. (11) the ball's purpose unclear → not fixed: the attitude puck (felt G, brake arcs, slip halo); it is rank 4. (12) the cluster is small in the frame → the glance frames show the whole 4K screen, by design.

### DC answers
- V1 yes: expectations frozen at 08:58, before the cabin-environment round (rendered 09:00).
- V2 yes: the cold look above, written before opening the board and the notes; rejection patterns checked on the named crop.
- V3 yes: one fresh-context `cold-reviewer` run, findings recorded above.
- V4 yes: "First: the gear "4"."
- V6 yes: gear and speed pass in all four states; the numerals are static scale labels at the static cap (my call, above).
- V10 yes: the squint view (`review/04_drift_braking_cockpit_squint.png`) keeps the gear and the lit pods, then the tunnel and the speed; labels and unlit rings go first.
- V13 no: "don't know". Not fully blind (the agent read AGENTS.md, section 8 of the skill), and a car cluster built on a visionOS window is unlikely to be named from the image.
- V14 yes for layout, geometry and colour: the unit tests pin the zone table (envelope, toe, rings, pods, pane digits, numerals, wing gap); F1 to F12 matched except the logged F2 change.
- V15 no: blurred, R1 is one large window with a side ornament; the render is a central pane with two wings, a top ornament bar and a tunnel. Related masses (floating pane, ornament), different arrangement: the concept is a car cluster, not a copy of the window.
- V16 yes: `review/board_vs_render.png`. Every board zone is where `Geom` puts it; departures logged in RESEARCH section 9 (tapered wings for quads, labels under the numerals, wings clear of the pane). The render is richer than the board (real glass, cabin reflections, the glowing tunnel inside the pane).
- V17 yes: the reviewer named the "glass box" and "glass pills" unprompted; rims follow one light.
- V19 yes: the glass bends the tunnel and sky behind it, stays clear over black, rims crisp; highlights move between mid cruise and drift (`VirtualLight`); no glass on glass since f38e0ff.
- V20 yes: `stretch`, 0 px outside the envelope.

### Decision: **go with changes**
V13 and V15 are no for the concept itself, which the rule calls a no-go. TJS's own words on this render set win (AGENTS.md rule 12): "This is on path to being insane holy! Love this how it's going" (28 Nov 08:53 UTC, on the first-build sheet). The changes listed under Q5 are done (f38e0ff).

## Visual review

## Before and after pair
Before: the scaffold, `/mnt/project-files/vision_spatial_glass/before_02_mid_cruise_cockpit.png` (lavapipe, kept at the start).

## Assets and masters
`assets/clusters/vision_spatial_glass/cabin_env.png` from `tools/asset-forge/src/recipes/vision_spatial_glass.rs` (ComfyUI cabin panorama, seed 102_203; Blender studio, seed 102_301), when the cloud run lands. Without it the dash uses the Liquid Glass studio.

## Changes from the real display and from the brief
`RESEARCH.md` section 9.

## Shared places touched
`docs/TELEMETRY_USAGE_MATRIX.md` (this dash's row only).

## Requests for shared code

## What sibling dashes must know
- A tapered `Outline::Capsule` (different `radius_a` and `radius_b`) is one glass piece that reads as a toed-in pane; no quad outline is needed.
- Strokes of large rounded rectangles are drawn as four lines and four corner arcs, so each draw covers only its band.

## Review findings
