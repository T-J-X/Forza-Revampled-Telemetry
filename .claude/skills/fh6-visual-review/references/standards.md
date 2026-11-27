# Reference numbers

Sourced values behind the checklist, converted to this panel. Evidence: **cited** (read in the source), **summary** (a credible secondary source quoting it; ISO texts are paywalled), **derived** (arithmetic from cited inputs). Nothing here is a score: each number is a fixed line a render meets or does not.

## The panel and the eye

- 55-inch 16:9 LG C1: 1.218 m wide, so 1 px = 0.317 mm (derived; `docs/design/RESEARCH.md`).
- One pixel subtends 1.82 / 1.36 / 1.21 arcmin at 0.6 / 0.8 / 0.9 m; a 10-degree field is 331 / 442 / 497 px across (derived).
- With the eyes on the road (the screen centre, y = 1080), the cockpit anchor (1920, 1835) sits about 22 / 17 / 15 degrees below the gaze at 0.6 / 0.8 / 0.9 m, and the hood anchor (1920, 1760) about 20 / 15 / 13.5 degrees (derived).
- Visual acuity: about 1 arcmin (0.55 / 0.73 / 0.83 px), so 1 px is resolvable at every distance; anything thinner than 1 px at full contrast is not (cited: [AAO EyeWiki](https://eyewiki.aao.org/Visual_Acuity_Testing_in_Adults)).
- Claude's vision: 1:1 only up to 1568 px a side and about 1.15 MP on the standard tier (2576 px on some newer models); a 4K frame arrives at about 0.41x (cited: [Claude vision docs](https://platform.claude.com/docs/en/build-with-claude/vision)).

## Text size

| Rule | Value | On this panel | Evidence |
|---|---|---|---|
| ISO 15008:2017 dynamic text, cap height (H) | at least 20 arcmin recommended, 16 acceptable; judged from the rearmost eye point | 17.7 px (20') and 14.1 px (16') at 0.9 m, cockpit anchor, with the screen's tilt from the eye | summary: [You et al. 2021](https://doi.org/10.3390/info12050213) |
| Moving vehicle (TRL 2002, from the ISO 15008 drafts) | at least 24 arcmin; 15 only for static or non-critical text | 21.2 px at 0.9 m, cockpit | cited: [TRL PA3721/01 §5.2.3.5](https://trimis.ec.europa.eu/system/files/project/documents/20060728_165141_88073_UG340_Final_Report.pdf) |
| Glance legibility time | 114 ms at 20' (4 mm at 68.6 cm), 166 ms at 15'; light-on-dark about 40% slower than dark-on-light | 17.8 px cap at 0.9 m for a 150 ms read | cited: [Dobres et al. 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC5213401/) |
| Reading without looking down (Anstis, via Strasburger) | smallest recognisable letter = 0.031 + 0.046 x eccentricity (degrees) | about 40 px cap (cockpit) and 36 px (hood), almost the same at every distance | summary: [Strasburger et al. 2011](http://hans-strasburger.userweb.mwn.de/reprints/peripheral_vision_review.pdf) |
| Crowding (Bouma) | clear space at least about 0.5 x eccentricity | about 390 px sideways and 420 px above or below the gear (cockpit); less means a glance is needed | summary: Strasburger et al. 2011 |
| The repo's floor | `accessibility.minCapPx` 12 px cap | static micro-labels only | tokens |

ISO 15008's size rules cover dynamic information (more than two states). Units (mph, km/h, x1000 rpm) and quasi-static items are excluded (cited: [ISO 15008:2017 preview](https://cdn.standards.iteh.ai/samples/62784/ac1daa21f49545c69dc9a5b33259a059/ISO-15008-2017.pdf)). The token type scale already puts every dynamic readout at 30 px cap or more.

Secondary type metrics, for findings (summary: [Font Bakery's ISO 15008 checks](https://github.com/fonttools/fontbakery/tree/main/Lib/fontbakery/checks/iso15008), TRL 2002): stroke width 0.10 to 0.16 of the cap height (never below 0.08); H width 0.65 to 0.80 of its height; at least one stroke width between characters. The project's motorsport faces are condensed and some styles are light, so a reading outside these ranges is a finding to weigh (light weights should be set larger, as Apple also advises), not an automatic no.

## Contrast

| Rule | Value | Evidence |
|---|---|---|
| The tokens' blur gate | at 960x540 and sigma 1.5: gear at least 7:1, speed at least 4.5:1 | tokens `accessibility.blurGate`; RESEARCH principle 4 |
| WCAG 1.4.3 text | 4.5:1; 3:1 for large text (at 0.9 m on the cockpit anchor: an em of 27.1 px, or 21.1 px bold) | cited: [WCAG 2.2](https://www.w3.org/TR/WCAG22/); the size conversion is derived |
| WCAG 1.4.11 graphics and states | 3:1 (needle against face, lit against unlit segments, active against ghost digits) | cited: WCAG 2.2 |
| Android for Cars | 4.5:1 for text and icons that carry meaning | cited: [Design for Driving](https://developers.google.com/cars/design/design-foundations/visual-principles) |
| ISO 15008 (2003 clause, carried into 2017) | plain luminance ratio at least 5:1 at night, 3:1 by day, 2:1 in sunlight | cited: [ISO 15008:2003 preview](https://cdn.standards.iteh.ai/samples/34886/cad64a3350094f93a48fe3c610b71d7e/ISO-15008-2003.pdf) |
| Text over clear glass (Apple) | a dark dimming layer of about 35% over bright content | cited: [HIG Materials](https://developer.apple.com/design/human-interface-guidelines/materials) |

Renders are emissive and have no ambient light, so only the night case can be judged from a PNG; day and sunlight are NOT ASSESSED (summary: [TechnoTeam on ISO 15008 testing](https://www.technoteam.de/products/display_metrology/lmk_reflection/index_eng.html)).

## Flashing

- **WCAG 2.3.1 general flash:** a pair of opposing changes in relative luminance of at least 0.10 where the darker state is below 0.80. Content passes with no more than 3 flashes in any second, or when the concurrently flashing area is at most 0.006 sr (25% of a 10-degree field) at the closest viewing distance: 21,500 px² in a 331 px circle at 0.6 m (cited: [Understanding SC 2.3.1](https://www.w3.org/WAI/WCAG22/Understanding/three-flashes-or-below-threshold.html)). `visual-review flash` implements this.
- **Red flash:** a pair of opposing transitions involving a saturated red (R / (R+G+B) at least 0.8). WCAG 2.0, still used by PEAT: the change in (R - G - B) x 320 exceeds 20 for both transitions; WCAG 2.2 uses a u'v' difference above 0.2 instead (cited: WCAG 2.2 and 2.0 definitions). `flash` uses the 2.0 rule on linear values.
- **Counting:** one flash = two opposing transitions; more than 3 flashes a second = 7 transitions inside one second (cited: [HardingFPA](https://www.hardingfpa.com/technical-support/how-to-interpret-hardingfpa-results/); [PEAT](https://trace.umd.edu/?p=719)).
- **ITU-R BT.1702-3:** flashes whose leading edges are at least 334 ms apart (21 frames at 60 fps) are acceptable at any size; flashing that lasts more than 5 s may be a risk even when it passes (cited: [ITU-R BT.1702-3](https://www.itu.int/dms_pubrec/itu-r/rec/bt/R-REC-BT.1702-3-202311-I!!PDF-E.pdf)).
- **Regular patterns:** more than 5 light-dark stripe pairs are a risk when they move or flash over more than 25% of the screen; WCAG exempts fine balanced patterns under 0.1 degrees (3.3 px at 0.6 m) (cited: ITU-R BT.1702-3, WCAG 2.2). This is the check for scanlines and LED grids.
- **The tokens' strobe:** 10 Hz, 50% duty, on the shift strip only (at most 42 px tall at scale 1.0), nothing taller faster than 3 Hz (tokens `motion.limiterStrobe`). It follows race hardware, and the area rule above is what makes it legal. ISO 15008's draft guidance for road-car information displays is slower (attention blinking at 2 to 3 Hz, avoid 4 Hz or faster; summary via TRL 2002). That difference is a design-token decision (decision log, 2026-11-21: WCAG 2.3.1 and the frame maths), not something a dash thread changes; a thread that thinks it should change raises it with the coordinator.

## Glance and placement

- NHTSA visual-manual guidelines: no more than 15% of off-road glances over 2.0 s, a mean of 2.0 s or less, and at most 12 s in total; checking the speedometer and gauges is outside their scope, while trip computers and information centres are inside it (cited: [NHTSA 2013](https://www.govinfo.gov/content/pkg/FR-2013-04-26/html/2013-09883.htm)). These are task-level measures; renders can only check proxies: no scrolling text, no paging, every readout a single glance target.
- A driving display sits no more than 30 degrees below eye level: 1,093 px below y = 1080 at 0.6 m. The cockpit anchor is 755 px below it and the hood anchor 680 px (cited: NHTSA 2013 §V.C; JAMA; derived).
- JAMA: no more than 30 letters on screen while moving (each number or unit counts as one, punctuation as none), and no scrolling text (cited: [JAMA guideline v3.0](https://www.jama.or.jp/operation/safety/display_system/pdf/jama_guidelines_v30_en.pdf)).
- A response to an input within 250 ms (15 frames) is the general rule (cited: [AAM principles](https://www.autosinnovate.org/safety/driver-behavior/PDF_DF-T%20with%202006%20-Updates-prot.pdf)); the project's own rule is stricter: the new gear on the next frame.

## Not verified

The ISO 15008:2017 values for proportion, weight, spacing, twilight contrast, display mode and flashing (clauses 4.3 to 4.8, Annex B) were not found in any credible secondary source. Cite them only through the secondary values above, and never as ISO 15008:2017 text.
