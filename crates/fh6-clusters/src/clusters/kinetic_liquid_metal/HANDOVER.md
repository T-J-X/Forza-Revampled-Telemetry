# HANDOVER: Dash · KineticLiquidMetal (catalog row 27, build #7)

Draft: filled in as the thread works. Sections still to come are listed at the end.

## Direction check

### Frozen expectations (written 2026-11-28 06:55 UTC, before the round's render, the first one on the PC's GPU with the Blender end caps, was opened)

The first build's lavapipe renders (v1, posted 06:45 UTC) were already looked at while fixing the gear's crack, so they are not the round's render. The direction check reviews the next render: the same code on the PC's card with `end_cap.png`.

Rank order (fh6-hmi-design-rules 3): (1) the gear inside the ampoule and the eleven shift beads, (2) the speed digits, (3) the mercury thread and its meniscus (the tach), (4) the mercury pool, the test-tube pedals, the G vial, slip and longitudinal G, (5) the glass, the etched scale and numerals, the end caps, the labels and the logo.

Feature inventory of the real instrument (from `RESEARCH.md`), each with the tolerance the render must meet:

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | A thin bright mercury thread in a sealed glass bore; the reading is the end of the thread | lens-front thermometer (US1819919) | the thread is a continuous bright line from idle to the current rpm; its end within 1° of the rpm's angle on the scale |
| 2 | The reading face of the tube is a lens that magnifies the thread | US1819919, US1590613 | the channel's glass visibly widens the thread compared with the bare capillary (its seen width at least 1.3× the 18 px bore) |
| 3 | A colour stripe behind the bore, reflected by the mercury so the thread looks coloured | US1819919 | the thread's bands carry the palette primary, never pure grey; `probe` hue of the thread within 30° of the primary's hue |
| 4 | Etched graduations, longer every tenth, numbered beside the tube | the lab instrument; OIML R 7 | a mark every 500 rpm, the 1,000 rpm marks longer, numerals 1 to 8 inboard of the channel, upright |
| 5 | A convex mercury meniscus | contact angle 140° | the thread's end is a rounded cap, not a flat cut or a point; cap height about 0.47 of the bore's half width |
| 6 | Mercury is a mirror of measured strength | R₀ 0.779, rising to 1 at grazing | the thread, the beads and the pool show light and dark bands of a studio, never a flat grey fill; brightest at the edges of the thread |
| 7 | A spirit-level vial whose bubble sits at the highest point | spirit level | the bubble moves toward the side the free surface rises; in `04_drift_braking` (−0.96 g) visibly off centre, in `02_mid_cruise` near centre |
| 8 | The free surface tilts by atan(a_lat ÷ (g + a_vert)) | hydrostatics | `04_drift_braking`: pool surface at 44° ± 5° (rising on the side away from the turn), `02_mid_cruise` within 5° of level |
| 9 | Borosilicate glass, clear | DURAN / BOROFLOAT, n_d 1.47 | no tint, frost or fill inside the glass; thin rims only; the gear under the dome magnified a little, not distorted past reading |
| 10 | Machined end caps where the tube is sealed | lab manometers | two caps at the channel's ends with a recessed dark face, a bright chamfer ring and a darker outer ring, visible without zoom |

Key frames of the sequence: the limiter is reached at frame 29 or 30, the strobe runs 30 to 41, the gear changes at 42.

Blur levels: the tool's defaults (glance 960x540 at sigma 1.5, squint at its default), the same at the closing review.

TJX's rejection patterns to check against named crops: (a) it looks nothing like the real instrument (for a concept dash, the benchmark); (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real instrument does not have; (d) flat modern widgets instead of the "2027 future" bar; (e) murky, tinted or frosted glass.
