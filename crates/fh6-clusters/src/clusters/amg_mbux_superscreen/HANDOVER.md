# HANDOVER: Dash · AmgMbuxSuperscreen (catalog row 4, build #8)

This handover is written as the dash is built. The sections after the direction check fill in at the closing review.

## History

- First build (v1): a black display unit with a bezel and the gear in the disc, made before its Penpot board. TJS, 2026-11-28 07:07: "i feel this doesnt lock it in properly id can be outside of a slab btw it needs to be what mbux is but better this applies to all mbux"; 07:08: "never ever dont we always make in penpot first then code ?".
- Board v2 (no slab) and its Blender plates: TJS, 07:41: "It looks nothing like how it should".
- Board v3 (this build): measured from `w214_sport_full.jpg` and `w214_sport_tunnel.jpg`, imported into Penpot (page "07 · AmgMbuxSuperscreen", boards "v3 · <state>", 07:55), then the v3 plates and this code. The v1 expectations below were replaced before the v3 renders were opened.

## Direction check

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
