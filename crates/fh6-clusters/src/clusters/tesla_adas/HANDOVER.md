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
