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
