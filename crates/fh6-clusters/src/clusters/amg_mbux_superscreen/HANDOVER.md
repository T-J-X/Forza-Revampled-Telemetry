# HANDOVER: Dash · AmgMbuxSuperscreen (catalog row 4, build #8)

This handover is written as the dash is built. The sections after the direction check fill in at the closing review.

## Direction check

### Frozen expectations (written 2026-11-28 07:10 UTC, before the direction-check renders were made or opened)

Rank order (fh6-hmi-design-rules 3): (1) the gear in the disc and the shift strip on the limiter, (2) the speed at the top centre, (3) the red crescents filling with rpm, and the rpm figure on the floor, (4) the torque and power scales, the G-Force and boost widgets, (5) the housing, the tunnel rings, labels, units, the logo and the transmission box.

Feature inventory of the real display (from `RESEARCH.md`, "What makes it recognisable"), each with the tolerance the render must meet:

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | Three zones: G-Force left, tunnel middle, Engine data right | `w214_sport_full.jpg` | both widgets centred within ±40 px of the screen's vertical centre line; the tunnel 50 to 60% of the glass width (crescents' outer edges) |
| 2 | Two red crescents "( )", thick at 9 and 3 o'clock, tapering to points | same | both present and mirrored within 2 px; points at both ends; standing 75 to 90% of the glass height; saturated red in front |
| 3 | Rings receding to a dim grey-white disc a little above the centre | `w214_sport_tunnel.jpg` | at least 5 rings visible, each nested; the disc's centre above the tunnel centre; the disc the brightest non-type area inside the crescents |
| 4 | Dark floor trapezoid from the disc to the bottom edge with thin perspective edges, holding the rpm and "rpm" | same | a trapezoid wider at the bottom, darker than the rings beside it, edges visible as thin lines; the rpm figure and "rpm" inside it |
| 5 | Torque left, power right: thin grey arcs with fine ticks, numerals on the inner side between arc and crescent, unit at the bottom | `w214_sport_full.jpg` | both arcs present, numerals between arc and crescent with no overlap, a unit label at each lower end; zero low on the scale (deliberate change: zero at the bottom) |
| 6 | Thin light speed at the top centre with its unit under it | same | the speed centred within 4 px, light weight, the unit under it; lines either side replaced by the shift strip (change 1) |
| 7 | Bottom row and the selector stack | same | changed by design: logo bottom left, transmission box bottom right (changes 3, 6, 9) |
| 8 | G-Force rings, crosshair and dot; a round Engine data gauge | same | G-Force: two rings, a crosshair, one dot; boost gauge round with a scale and a centred value; a title above each |
| 9 | One saturated red on true black; white live values, grey labels and scales | same; measured #E0102A to #FF2030 in front | probe of the crescent's lit front: hue within 350° to 360° (or 0° to 5°), saturation above 85%; background corners off; labels and scales visibly greyer than live values |
| 10 | Light, slightly slanted grotesque for speed and rpm | same | visibly slanted, a light weight; no serif |

Key frames of the sequence: the limiter is reached at frame 29 or 30, the strobe runs 30 to 41, the gear changes at 42.

TJX's rejection patterns to check against named crops: (a) it looks nothing like the real car's display; (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real display does not have; (d) flat modern widgets instead of the "2027 future" bar; (e) murky, tinted or frosted glass (this dash has no glass elements on the screen; the cover glass is the housing's only glass).
