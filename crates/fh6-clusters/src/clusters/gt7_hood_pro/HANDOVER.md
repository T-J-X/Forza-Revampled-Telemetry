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

## Visual review (`fh6-visual-review`)

**Frozen expectations** (written at 04:34Z on 2026-11-28, before any render with the Blender plate existed; the only renders seen until then drew the procedural fallback):

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
