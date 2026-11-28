# Gt7HoodPro research: Gran Turismo 7's Bonnet View meter

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Researched on 2026-11-28 from Polyphony Digital's own online manual (the race screen and the racing views, read in a browser because the pages are drawn by script) and from the community's decode of GT7's telemetry packet, which names the meter's data. Every point below names its source. The reference screenshots are copyrighted: they stay out of git (`mockups/references/` is ignored) and the links below point at the originals.

## What this dash recreates

**The compact meter that Gran Turismo 7 shows in its Bonnet View**, the view our hood camera matches. GT7 has four racing views: Cockpit, Normal (bumper), Bonnet and Chase ([GT7 manual, "Racing Views"](https://www.gran-turismo.com/gb/gt7/manual/race/03)). The meter changes with the view:

- In the **Normal and Chase views** GT7 draws its full meter: a round speedometer on the left, a round tachometer on the right, a trapezoid centre pod and the multi-function display under it ([GT7 manual, "The Race Screen"](https://www.gran-turismo.com/gb/gt7/manual/race/02), items 13 to 23). TJX assigned that meter to `Gt7ModularTripod` (CATALOG row 14, TJX override of 2026-11-23), so this dash does not copy it.
- In the **Bonnet View** the same information shrinks into **one low band across the bottom of the screen**, floating over the bonnet with no backing panel ([Racing Views](https://www.gran-turismo.com/gb/gt7/manual/race/03), the Bonnet View picture). That band is this dash.

The manual's own wording for the parts (race screen, items 13 to 23) names what each piece does; the Bonnet View picture shows how they are arranged when the meter is compact.

## What makes it recognisable

Measured in the Bonnet View picture of the manual (1920x1080) at 2.56x in the browser, then doubled to 4K and written relative to the screen's centre line, which is our hood anchor's x (1920). y grows downwards.

### Layout (left to right)
1. **Tyre diagram**, far left: a small grey car outline with four white tyre pills at its corners and the tyre compound's initials above and below. About 210x214 px at 4K, centred about 1130 px left of the centre line. The tyre frames redden with heat and the pills turn from white to red with wear (race screen item 20).
2. **Small round gauge with a box under it**: a thin semicircle scale (about 164 px across) with a fuel pump icon in it, and under it a translucent grey rounded box holding the odometer, "000007.7" in light digits (about 190x38 px). The fuel meter shows only in races with fuel use (item 15).
3. **Driver-assist telltales**: circled icons in a 2x2 grid (about 60 px circles). White means on, grey off, red engaged (items 22 and 23). The same grid sits on the right of the band.
4. **Brake bar**, left of the rev bar, and **throttle bar**, right of the gear: slim vertical bars inside a thin outline bracket that fill white with the pedal. Red shows how much ABS or traction control takes away (items 16 and 19).
5. **Rev bar**, centred on the screen: a straight horizontal bar of dense vertical ticks, about 800x30 px at 4K (roughly 100 ticks). Lit ticks run from the left end to the current revs; unlit ticks are dark grey on a darker strip.
6. **Steering dot**: a small red dot just above the rev bar's centre that slides left and right with the steering (item 13).
7. **Speed | gear block** under the rev bar: the speed in light, squared digits (cap about 54 px at 4K), right-aligned to a thin vertical divider on the centre line, with the unit ("mph" or "km/h") small under it; the gear in a bold squared digit (cap about 80 px) right of the divider (item 14).
8. **Suggested gear**: a red rounded box (about 86x70 px) right of the gear with a gear number in it, and the transmission tag ("AT" or "MT") under it. The packet decode calls this the assist's suggested gear to downshift to, with 15 meaning no suggestion, when the box is hidden ([PDTools `SimulatorPacket.cs`](https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs), `SuggestedGear`). In the race screen picture it reads 2 while the car is in 4th and the braking suggestion is shown.
9. **Multi-function display**, far right of the screen, outside the band: a paged panel with a title, an arrow and page dots (item 17).

### The rev bar and the shift indicator
10. The manual says the shift indicator **lights up as the revs get close to the red zone** (item 13). In the Bonnet View picture, near the top of second gear, **the whole lit run is red-pink**, not just its end: the bar changes colour all at once when the revs pass the alert point.
11. The packet carries a **minimum and maximum rpm for the rev alert** and a flag for **"rev limiter blink alert active"**: the indicator lights between the two and blinks at the limiter ([`SimulatorPacket.cs`](https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs): `MinAlertRPM`, `MaxAlertRPM`, `SimulatorFlags.RevLimiterBlinkAlertActive`).

### Colour, light and type
12. **Monochrome white and grey with red accents**, sampled from the picture (a JPEG over the scene, so approximate): lit ticks red-pink about `#C84F70`, unlit ticks about `#333132`, the suggested-gear box pure red about `#E8090E`, digits warm white about `#F9F6F1`.
13. **Squared digits with rounded corners**: thin for the speed, bold for the gear. GT7's face is its own; the pack's Orbitron (OFL) is the nearest squared face.
14. **No panel behind the band.** The white digits sit straight on the scene with a faint dark edge; the rev bar has its own dark strip.

### Behaviour
15. The braking suggestion turns the meter red with blinking text before corners (item 11).
16. Blind-spot semicircles light either side of the speed and gear when cars are alongside (item 14).
17. The handbrake telltale lights while the handbrake is on (item 15; the packet has a handbrake flag).

## How the dash matches each feature

| # | Real Bonnet View meter | This dash | Data |
|---|---|---|---|
| 1 | Tyre diagram, heat and wear | Forged-carbon car chip with four tyre lenses lit by **tyre load** (white only) | load from verified acceleration (weight transfer) |
| 2 | Fuel semicircle and odometer box | **Boost** semicircle (x100 kPa, -1 to 2, like GT7's boost gauge) labelled `BOOST`, and a **trip** box (five digits and tenths) | verified boost; trip integrated from verified speed |
| 3 | Assist telltales, 2x2 | Two lamps where verified data exists: **slide** (left, amber: body slip over 8° at 8 m/s and more) and **rev limiter** (right, red: 98.5% of the limiter), grey when off | velocity (slip); rpm |
| 4 | Brake and throttle bars, red for ABS or TC | Same bars in black-glass slots, white fill only | verified brake and throttle |
| 5 | Rev bar of dense ticks | Same bar: 95 LEDs behind smoked lenses in a black-glass light bar | rpm, limiter |
| 6 | Steering dot (red) | Same dot, in the palette's primary colour | verified steer |
| 7 | Speed, divider, gear | Same block: the tabular speed (with display hysteresis) right-aligned to the divider on the centre line, the gear right of it | verified speed and gear |
| 8 | Suggested gear box (downshift) | Same red box, showing **gear + 1 at the shift point** (gears 1 to 9), hidden otherwise; it pops up, and folds away into the gear when the driver takes it | rpm, gear, shift point |
| 9 | Multi-function display | Not in the band (outside the hood envelope; paging is `Gt7ModularTripod`'s) | none |
| 10, 11 | All-at-once alert colour, blink at the limiter | From the warning stage (93% of the shift point, GT7's `MinAlertRPM` analogue) the whole lit run takes the `shift` colour; from the shift point the run strobes at 10 Hz, starting lit, keeping a fifth of its light in the dark phase. The box never strobes (the tokens keep the strobe on the shift strip) | rpm, shift point |
| 12 | White, grey, red | `ink` for digits and lit LEDs (body at 78%, head at full), `shift` for the alert run, `warning` red for the box, the plate's smoked lenses for unlit LEDs, static parts held at the tokens' cap | palette roles |
| 13 | Squared digits | Michroma (speed and trip: wide squared digits with a plain zero), Orbitron 600 (gear and the box's digit), Titillium Web 600 (units and labels); a user display font replaces the numerals | fonts |
| 14 | No panel | No panel and no box behind anything: a black keyline round every glyph, the divider and the logo keeps them readable over bright road and sky, as GT7's digits carry a dark edge | none |

## Deliberate changes and why

- **Suggested gear is an upshift prompt.** GT7's comes from its driving-line assist, which FH6 does not send. The box keeps its look and meaning ("this is the gear to select") and is derived honestly: at the car's shift point it names the next gear. No transmission tag under it, because the telemetry does not say manual or automatic. It names nothing in neutral, reverse or 10th, and stays away after a gear change until the revs drop below the shift point.
- **The steering dot is not red.** It shows all the time, and the tokens keep red for the redline, braking and alarms (`colorRoles.budget`), so it takes the palette's primary colour.
- **Telltales: slide and limiter only.** GT7's grid shows assists, which FH6 does not send. The golden packets carry the clutch and handbrake only at zero, so they are not verified (AGENTS.md directive 1) and get no lamp. Body slip comes from verified velocity and the limiter from rpm, so those two are the lamps, one on each side of the band.
- **No red in the pedal bars, no tyre heat or wear, no fuel, no braking suggestion, no blind-spot lights.** None of that is in the verified telemetry (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`, telemetry honesty). Each slot is either dropped or shows the nearest verified quantity, and the table above names it.
- **No tyre compound initials.** There is no compound data.
- **The band is narrower.** GT7's band is about 1916 px wide at 4K without the multi-function display; the hood envelope is 1760 px, so the side groups sit closer to the centre, the telltales are single lamps, and the trip box holds five digits and tenths, one digit fewer than GT7's odometer.
- **The car's logo or custom name** sits at the right end of the band, where GT7 shows the multi-function display's title (project rule: each dash uses the brand logos).
- **The cockpit and chase views** draw the same band scaled into the cockpit envelope; the dash is made for the hood view.
- **2027 production finish.** GT7 draws flat graphics. Here the static parts are one Blender plate lit by the cluster's virtual light (three azimuths cross-faded with lateral G): the black-glass light bar on a polished black-chrome plinth with 95 smoked LED lenses, the forged-carbon car chip (a ComfyUI texture rendered into it) with its canopy and tyre lenses, the boost gauge's polished aluminium scale, the black-glass pedal slots, the telltale rings, the graphite-glass trip face, and the red enamel suggested-gear tile as its own sprite. The layout, colours, type and behaviour stay GT7's.

## Gaps (not verified)

- Colours and tick count come from one soft 1080p JPEG; the hues are estimates.
- The blink rate at the limiter is not documented; the dash uses the project's limiter strobe (`data/design_tokens.json`, `motion.limiterStrobe`).
- That the alert colour switches all at once is read from one picture at high revs; the manual says only that the indicator lights near the red zone.
- GT7's typeface is proprietary and unnamed.
- FH6 does not send the number of gears, so in a car's top gear below 10th the box still names the next gear at the shift point.
- The slide lamp's thresholds (8° on, 5° off, above 8 m/s) are set from the review states, not from a real car's stability-control lamp.
