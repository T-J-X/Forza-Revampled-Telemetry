# BoschDdu research: the Bosch Motorsport DDU 10 race display

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Researched on 2026-11-28 from Bosch Motorsport's product pages, data sheets and operation manuals, the makers' own documents for the cars that ship the display, and the licensed sim manuals of those cars (marked as such). Every point names its source. Where no source says something, the gap is listed at the end rather than guessed.

## What this dash recreates

**The Bosch Motorsport DDU 10 ("Display and Data Unit"), as fitted in the BMW M4 GT3.** It is the current flagship of Bosch's two race displays: Bosch's display category lists only the DDU 10 and the smaller DDU 11 ([Bosch Motorsport, Displays](https://www.bosch-motorsport.com/products/displays/)), and its Australian distributor calls the DDU 9 "older generation ... phased out in favour of our current generation DDU 10 and 11" ([Bosch Motorsport Shop AU, AMG GT3 Evo](https://www.bosch-motorsport-shop.com.au/amg-gt3-evo)).

- **DDU 10:** a 7-inch colour display, 800 × 480 px, with an ambient light sensor, **20 freely configurable RGB LEDs, 10 across the top and 5 down each side**, IP67, 198 × 134 × 35 mm, 875 g, 12 user pages in page sets ([DDU 10 product page](https://www.bosch-motorsport.com/products/displays/display-ddu-10/); [DDU 10 data sheet](https://www.bosch-motorsport.com/media/catalog_content/downloads_catalog/pdf_catalog/data_sheet_70827659_display_ddu_10.pdf)).
- **DDU 11:** 6.5 inches, 800 × 480 px, **8 LEDs across the top and 5 down each side**, 203 × 128 × 38.5 mm, 649 g ([DDU 11 product page](https://www.bosch-motorsport.com/products/displays/display-ddu-11/); [DDU 11 operation manual](https://www.bosch-motorsport.com/media/catalog_content/downloads_catalog/pdf_catalog/operation_manual_323436043_display_ddu_11.pdf), pp. 8 and 44).
- **The car:** BMW's own M4 GT3 brochure lists "Bosch DDU 10" under displays, with the "BMW M Track Cockpit" menu set up "via the display and steering wheel", and shows the unit lit in the car ([BMW M4 GT3 information brochure](https://s3.amazonaws.com/themes.limelightplatform.com/client/bmw-na/files/Other/BMW+M4+GT3+Infobrochure.pdf), p. 7). McLaren's 720S GT3X product guide names the same unit ([McLaren 720S GT3X product guide](https://view.pagetiger.com/csmnvfz/McLaren720SGT3XProductGuide.pdf)), and the licensed Ferrari 296 GT3 sim manual says the car "features a Bosch DDU 10 display" ([Ferrari 296 GT3 manual, sim](https://s100.iracing.com/wp-content/uploads/2024/06/Ferrari-296-GT3-V4.pdf)). The display is the same unit in every car; only its pages are the team's.

## What makes it recognisable

### The hardware
1. **One rounded housing, black glass face edge to edge.** Bosch's own press image of the DDU 10 (the product page's main picture, [image](https://www.bosch-motorsport.com/media/catalog_content/images_catalog/jpg_catalog/9007199325544587_web.jpg), studied at full size in the built-in browser) shows a black glass front that covers the screen and the whole bezel, a gently arched top edge, large rounded top corners, straight sides, smaller bottom corners, and a satin grey aluminium side wall about a quarter of the face's height deep. The BMW brochure photo shows the same light edge in the car (brochure p. 7).
2. **Proportions.** The housing is 198 × 134 mm (1.48:1); a 7-inch 5:3 panel is about 152 × 91 mm, so the screen fills about 77 % of the width and 68 % of the height (data sheet). In the press image the border above the screen (the LED brow) is about 15 % of the face's height and the border below it about 13 %.
3. **Ten LEDs across the top, in three groups: 2, 6, 2.** In the press image the top row runs along the arched brow with a clear gap after the second and before the ninth LED; each LED is a small horizontal pill behind the glass; an unlit LED reads as a short grey dash (press image; data sheet: "10 on top").
4. **Five LEDs down each side of the screen,** short horizontal pills in the upper half of each side border (press image; data sheet: "5 on each side"). In the press image the lower two on the left and the lower three on the right are lit blue.
5. **Print on the glass:** "Motorsport" bottom left and the Bosch logo bottom right (press image; BMW brochure p. 7 shows "DDU 10" bottom left in the car). A small round ambient-light sensor sits in the right border (press image; data sheet "ambient light sensor").

### The screen (Bosch's own demonstration page)
Both Bosch press images, the DDU 10 above and the DDU 11 ([image](https://www.bosch-motorsport.com/media/catalog_content/images_catalog/png_catalog/336956299_web.png)), show the same page, which is how the display is pictured everywhere:

6. **A round rev gauge in the middle:** a dark disc, a 270° scale from 0 at about half past seven to the top value at about half past four, numerals in thousands outside the ring, a thick arc that fills clockwise in the accent colour, and the rpm as a large number in the disc with "rpm" under it (DDU 10: "2174 rpm", cyan arc on a 0 to 6 scale; DDU 11: "7104 rpm", red sweep on 0 to 8).
7. **The gear, large and white, at the bottom centre inside a glowing perspective beam:** a trapezoid of light that widens towards the bottom edge of the screen, like a road running away, overlapping the bottom of the rev gauge (DDU 10: "3" in a cyan beam; DDU 11: "4 GEAR" in a red beam).
8. **Four quarter panels round it:** top left "TEMP" with two horizontal bar gauges (oil and water icons, a marker on each bar); bottom left "FUEL" with a ring gauge and the percentage inside; top right "LAP TIMES" with ACTUAL (stopwatch icon, the largest figure), DIFF_LAPTIME (small, in the accent colour) and PREDICTED (in a highlighted box); bottom right "SPEED" with a large number and the unit under it (both press images).
9. **Panel headers:** a short caps label over a thin rule that ends in an angled corner, the right-hand headers right-aligned (press images).
10. **One accent colour per theme** on black (cyan on the DDU 10 image, red on the DDU 11), white figures, grey labels and scale (press images). RaceCon's elements are boxes, horizontal and vertical bars, large and small gauges in "Classic" and "Modern" styles, values and texts, enumerations, images and alarm icons, and any of them can change colour when its value crosses a limit with hysteresis ([DDU 10 operation manual](https://www.bosch-motorsport.com/media/catalog_content/downloads_catalog/pdf_catalog/operation_manual_70834315_display_ddu_10.pdf)).

### Shift lights and alarms
11. **Shift lights on the top row, from an rpm channel with per-gear thresholds and hysteresis;** the LEDs follow the six-step brightness switch (DDU 10 operation manual §9.5; data sheet).
12. **Real calibrations:** McLaren's lights step "from left to right: Green, Yellow, Orange, Red and then all illuminate in Blue", in steps that let the driver anticipate the blue, and with the pit limiter on the "left and right-hand LEDs flash white" (McLaren product guide pp. 42 and 45). The DDU 11 press image shows the top row graded green, yellow, orange, red from left to right. The licensed BMW M4 GT3 sim manual describes 2 green, 6 yellow, 2 orange, then all red at the shift point, and a magenta pit-limiter flash ([BMW M4 GT3 manual, sim](https://s100.iracing.com/wp-content/uploads/2024/07/BMW-M4-GT3-Manual_V3.pdf), p. 7).
13. **The side columns carry driver aids:** in the BMW M4 GT3 the side LEDs show traction-control intervention, from one green LED up to five red (BMW M4 GT3 sim manual, p. 7).
14. **Alarms are opaque boxes drawn over the page in four fixed alarm areas,** one alarm per area at a time, several rotating every 1.5 s, blinking at a set rate (DDU 10 operation manual; DDU 11 operation manual pp. 70 to 77).
15. **Start-up:** the Bosch logo, then the configured page (DDU 11 operation manual p. 13).

### Type
16. No Bosch document names the display's face; the manuals offer only a font size per element (DDU 10 and DDU 11 operation manuals). In both press images the figures are a plain, low-contrast geometric sans of regular width with a flat-based 2 and a straight 7, the labels short caps, units lower case ("rpm", "mph", "kph").

## How the dash matches each feature

| # | Feature of the real display | In the dash | Match |
|---|---|---|---|
| 1 | Black glass face edge to edge, arched top, large top corners, aluminium side wall | One Blender-rendered housing: bonded black glass over the whole face, arched brow, top corners R 120, bottom R 44, a bead-blasted anodised aluminium body seen from the driver's eye above it (its top face shows as a band) | matched |
| 2 | 198 × 134 mm, screen 77 % × 68 %, brow 15 %, chin 13 % | Housing 810 × 548 px (1.48:1), screen 624 × 374 px (5:3), brow 92 px, chin 82 px | matched |
| 3 | 10 top LEDs in groups 2-6-2, horizontal pills, unlit grey | 10 rendered smoked lenses on the brow's arch, groups 2-6-2, lit by live vector light | matched |
| 4 | 5 LEDs down each side, upper half | 5 lenses per side between 19 % and 47 % of the face's height | matched |
| 5 | "Motorsport" and the Bosch logo printed on the chin, light sensor right | The chin carries the car's logo bottom right and the custom name or model bottom left; the sensor is rendered | changed (A) |
| 6 | Round rev gauge, 270° scale, numerals outside, accent arc, rpm figure in the disc | The same, the scale numbered to the first thousand at or above the car's limiter; arc in the palette's primary | matched |
| 7 | Big white gear in a glowing perspective beam at the bottom centre | The same; the gear is larger than on the demo page (B) | matched, larger |
| 8 | Four quarter panels: TEMP bars, FUEL ring, LAP TIMES list, SPEED | Same four panels and forms: PEDALS bars (throttle, brake), G ring (combined G), ENGINE list (power, boost, the car's shift rpm boxed), SPEED | changed (C) |
| 9 | Caps headers over a rule with an angled end | The same | matched |
| 10 | One accent colour on black, white figures, grey labels; limits recolour values | The palette's primary as the accent; boost and G recolour past their limits | matched |
| 11 | Top-row shift lights from rpm, per-gear thresholds, hysteresis | The palette's three stages over the 10 lenses left to right, the tokens' hysteresis; the shift point from the car's limiter (D) | matched |
| 12 | Green, yellow, orange, red, then all one colour; limiter flash | Stages go, caution, warning, then all ten in `shift`; on the limiter the top row strobes at the tokens' 10 Hz | matched |
| 13 | Side columns show driver aids, 1 green up to 5 red | Left column: body slip (derived, like a TC column); right column: brake pressure | matched, sources changed (E) |
| 14 | Opaque alarm boxes in fixed areas | Not drawn: no alarm source is decoded (F) | changed (F) |
| 15 | Bosch logo at start-up | Not drawn (A) | changed |
| 16 | Plain geometric sans, caps labels, lower-case units | Saira (regular width) for figures, Titillium Web caps for labels, lower-case units | matched |

## Deliberate changes and why
- **A. No Bosch marks.** The chin's two print zones become the dash's logo zone (the car's logo, bottom right, where Bosch prints its logo) and the name zone (the custom name or the dash's name, bottom left, where "Motorsport" is printed). Every dash shows its car's logo (CLUSTER_PROMPT, "Personalisation"), and the overlay carries no supplier trademarks.
- **B. The gear leads.** The demo page's gear is about a fifth of the screen's height; here it is the largest figure (rank 1, `fh6-hmi-design-rules` 3), still inside its beam.
- **C. Honest panels.** The telemetry has no temperatures, fuel or lap times (`docs/kit/README.md` §13), so each quarter keeps its form with data the packet carries: TEMP's two bars show throttle and brake; FUEL's ring shows combined G; LAP TIMES' list shows power (the large figure), boost (the small accent line) and the car's shift rpm (the boxed line, where PREDICTED sits); SPEED stays.
- **D. Thresholds from the car.** The shift point is the car's limiter times the tokens' shift share (`Signals::shift_rpm`); the telemetry has no per-gear table.
- **E. Side columns.** Traction control is not in the packet, so the left column shows body slip from velocity (`Signals::slip_deg`, labelled derived in the handover), one LED per 2.5° up to five; the right column shows brake pressure in fifths. Both stay dark when nothing is happening.
- **F. No alarm boxes.** There is no oil, water, fuel or tyre source behind an alarm, and the family rule says no lamp without a packet field (`docs/design/families/race-dash.md`).
- **G. The 2027 step.** The real unit's housing is rendered as physical hardware (Blender, with a ComfyUI anodised-aluminium texture) and seen from the driver's eye, the LEDs light through smoked lenses with a tight glow, and the screen keeps OLED black where the real panel shows dark grey.

## Gaps
- Housing material and finish, glass type and bonding, screen brightness and colour depth: not stated in the DDU 10 or DDU 11 sources; the satin grey edge is read from the press image.
- The real race pages of the BMW M4 GT3 and Ferrari 296 GT3 are known only from licensed sim manuals; no lit race-page photo from BMW, Ferrari or Bosch was found.
- The display's face is not named anywhere.
- The size of an LED lens is read from the press image only.
