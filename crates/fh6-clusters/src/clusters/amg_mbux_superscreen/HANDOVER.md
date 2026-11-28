# HANDOVER: Dash · AmgMbuxSuperscreen (catalog row 4, build #8)

This handover is written as the dash is built. The sections after the direction check fill in at the closing review.

## History

- First build (v1): a black display unit with a bezel and the gear in the disc, made before its Penpot board. TJS, 2026-11-28 07:07: "i feel this doesnt lock it in properly id can be outside of a slab btw it needs to be what mbux is but better this applies to all mbux"; 07:08: "never ever dont we always make in penpot first then code ?".
- Board v2 (no slab) and its Blender plates: TJS, 07:41: "It looks nothing like how it should".
- Board v3: measured from `w214_sport_full.jpg` and `w214_sport_tunnel.jpg`, imported into Penpot (page "07 · AmgMbuxSuperscreen", boards "v3 · <state>", 07:55), then the v3 plates and code (881a126). TJS, 08:17-08:19: "Okay so its still not 100% the same is it", "You just drop them and fill with something else like imagine subtle clouds and horizon tunnel background and the tunnel from the back to front being the rev limiter", "So many ideas you had when researching why am I doing this".
- Board v4 (this build): the sky tunnel. The W214's signature parts stay (crescents, side scales, G-Force, Engine data, selector stack); the middle is TJS's idea plus the thread's own: a dusk sky with clouds inside the tunnel that drifts with the distance driven and leans with lateral G, twelve rings that light from the far end towards the driver as the rev counter (the nearest three red, strobing on the limiter), lane dashes, a sunset sky near the shift and a red wash under braking. The six shift segments, the disc and the rpm line on the crescents are gone. Penpot boards "v4 · <state>" (08:40), cloud plates d22a3f5 (SDXL sky, ring ribs, road), code 711db53. TJS on the first v4 render, 08:55: "It's so close to perfect".

## Direction check

### Frozen expectations, round v4 (written 2026-11-28 08:57 UTC)

Honest note first: the first v4 render (`render_v4_first_states.png`, 08:50) was opened before this list was written, to find a bloom spill through the road (fixed: the rings now draw outside the bloom layer with their own halo). So V1 is answered no for this round too. Nothing below is edited after this point.

Rank order: (1) the engaged gear and the ring rev counter's red zone and limiter strobe, (2) the speed, (3) the lit rings below the red zone and the rpm figure, (4) the torque and power scales, G-Force and boost, (5) the sky, the crescents, the road, labels and the logo.

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | Three zones with no housing (kept from v3) | `w214_sport_full.jpg` | no rectangle or panel edge in `stretch`; widgets within ±40 px of the tunnel centre's height |
| 2 | Two red crescents, mirrored, pointed tips | same | mirrored within 2 px; hue 350° to 5°; at least half light at idle, full at the limiter |
| 3 | The rings light back to front with rpm | board v4 | lit count within one ring of round(12 · rpm/limiter) in each state; the nearest three red; on the limiter the two nearest strobe 3 on / 3 off and the third holds |
| 4 | The sky through the tunnel, above the horizon only | board v4 | no sky pixel below the horizon line on the road; the clouds read without zoom; the top of the window darker than the horizon |
| 5 | The horizon leans with lateral G | board v4 | 0° within 0.5° in `01_idle`; in `04_drift_braking` the lean is under 8° and its sign matches the G dot's side |
| 6 | The road with lane dashes and edge lines | board v4 | dashes in perspective, none over the rpm digits; the road covers the rings' light |
| 7 | Sunset near the shift, red wash under braking | board v4 | warm veil visible in `03`, none in `01`/`02`; red horizon and road wash in `04` only |
| 8 | Side scales, G-Force, Engine data, selector stack (kept from v3) | `w214_sport_full.jpg` | as the v3 rows 5, 7 and 8 |
| 9 | Speed above a thin line, rpm on the road | board v4 | `glance` yes for speed (4.5:1) in all four states over the cockpit composite |
| 10 | Soft stage, corners off | same | `stretch` smooth fall, corners off, 0 px outside the envelope + 40 |

Key frames: the limiter at frame 29 or 30, the strobe 30 to 41, the gear change at 42. TJX's rejection patterns (a) to (e) as in the v3 list below, plus (f) "not 100% the same": every replaced part must read as a deliberate idea, never a near-miss copy.


### Frozen expectations (written 2026-11-28 08:20 UTC, before the v3 renders were opened; they replace the v1 list of 07:10)

Correction to the heading above: this list was written at about 08:10 UTC, after one container preview of `02_mid_cruise` (1x container plates, opened at 08:07 to check that the plates load) had been looked at. So V1 is answered no for this round. Nothing below was edited after that.

Rank order (fh6-hmi-design-rules 3): (1) the engaged gear bottom right and the shift strip on the limiter, (2) the speed above the speed line, (3) the rpm line up the crescents and the rpm figure on the road, (4) the torque and power scales, the G-Force and boost widgets, (5) the crescents' red body, the tunnel, labels, units, the gear stack's neighbours and the logo.

Feature inventory of the real display, with the tolerance the render must meet:

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | Three zones: G-Force left, tunnel middle, Engine data right, no housing | `w214_sport_full.jpg` | widgets centred within ±40 px of the tunnel centre's height; no rectangle or panel edge anywhere (stretch) |
| 2 | Two wide red crescents "( )", flat in the middle, pointed tips, always lit | same | mirrored within 2 px; width at 9 o'clock 0.24 to 0.30 of the outer radius; red at every rpm, idle included |
| 3 | A shallow tunnel: thin grey ring lines, the light at the end brightest at the horizon, two red arcs beside it | `w214_sport_tunnel.jpg` | 4 to 6 ring lines, each thin (under 4 px); the light's brightest row at the horizon; a red arc each side rising from the horizon |
| 4 | The road from the horizon to the bottom, edges fading towards the viewer, the rpm and "rpm" on it | same | a trapezoid wider at the bottom; edge lines brightest at the horizon; the rpm inside it |
| 5 | Dark scale bands with a silver rim on the outer edge, ticks inward from the rim, numerals on the band | `w214_sport_full.jpg` | both bands present, rim visibly brighter than the band, numerals between band edge and ticks with no overlap |
| 6 | Light slanted speed above a thin line, unit under it | same | centred within 4 px, light weight, slanted, a thin line under it running towards the crescents |
| 7 | The selector stack bottom right | same (R N D stack) | the engaged gear large with the neighbours in a column beside it and a red `M` |
| 8 | G-Force: red rings, crosshair, dot over a dark lens; Engine data: round gauge with ticks and a centred value | same | two inner rings and a rim, a crosshair, one dot; gauge round with a scale; mixed-case titles above |
| 9 | One saturated red on black; white live values, grey labels | same | crescent front hue 350° to 5°, saturation above 85%; corners off |
| 10 | A soft dark stage behind the display, no hard edge | same (the screen's black glow) | stretch shows a smooth fall with no edge or banding steps |

Key frames of the sequence: the limiter is reached at frame 29 or 30, the strobe runs 30 to 41, the gear changes at 42.

TJX's rejection patterns to check against named crops: (a) it looks nothing like the real car's display; (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real display does not have; (d) flat modern widgets instead of the "2027 future" bar; (e) murky, tinted or frosted glass (only the G-Force lens is glass here: dark smoked glass, like the real lens).
