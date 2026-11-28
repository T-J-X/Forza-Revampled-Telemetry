# RetroCyber research: the Toyota Supra A70 "Panoramic Digital Meter"

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Researched on 2026-11-27 and 2026-11-28 from Toyota's own catalogue and press release, owner teardowns and forum threads, and photos of working clusters (marked as such). Every point names its source. Where no source says something, the gap is listed at the end rather than guessed.

## What this dash recreates

**The Toyota Supra of the A70 generation (1986 to 1993): its optional "Panoramic Digital Meter", a full-width vacuum-fluorescent (VFD) instrument panel.** The catalog row asks for a Sci-Fi VFD dash; the A70's meter is the best-documented production VFD cluster of the 1980s, its car is in Forza Horizon 6 (the 1992 Toyota Supra 2.0 GT, ordinal 460 in `data/fh6_car_ordinals.json`), and it is the period cluster the plan names for the retro dashes ("period clusters such as the Skyline for retro", plan doc).

- Toyota's 1986 launch release names it: "an easy-to-read Panoramic Digital Meter" ([Toyota global newsroom, 1986](https://global.toyota/detail/7776580)).
- The Japanese launch catalogue describes **large VFD tubes in five colours (yellow, blue, red, green and orange)**, a **horizontal tachometer** ("水平基調のタコメーター") and large speed numerals ([Gazoo, Supra A70 catalogue PDF](https://gazoo.com/pages/contents/article/old_cars/190228/SUPRA70_Web_GAZOO_catalog.pdf)).
- The option and its later swap into other A70s are documented by owners ([I Heart Japanese Cars, "Supra Mk III digital gauge swap"](https://iheartjapanesecars.blogspot.com/2016/04/toyota-supra-mk-iii-digital-gauge-swap.html); [Supramania, "The LHD digidash bible"](https://www.supramania.com/threads/the-lhd-digidash-bible.193179/); [Wikipedia, Toyota Supra](https://en.wikipedia.org/wiki/Toyota_Supra)).

The runner-up was the Toyota Soarer Z20 "Space Vision Meter", a VFD seen through a half mirror so it floats ([Motor-Fan](https://motor-fan.jp/article/287448/); [Motor-Fan](https://motor-fan.jp/article/1673698/)). It is less documented and its Z20 car is not in FH6, so it was not chosen.

The photo the layout below is measured from: a working LHD (km/h) Panoramic Digital Meter on a bench, lit, showing 215 km/h ([YouTube, "Supra MA70 LHD Digital Dash Swap over 180km/h", thumbnail at full size](https://www.youtube.com/watch?v=pdmxq5RCrNY)). A second photo of a JDM (RHD) car in its dash confirms the horizontal bar tach, the row of warning lamps under the panel and the TEMS graphic (a Reddit photo, `i.redd.it/bj6o3veaq3gz.jpg`, seen through Bing's preview because the browser pane blocks i.redd.it).

## What makes it recognisable

### Layout
1. **One long, very low panel, about 6 to 1**, filling the binnacle from side to side behind one dark smoked lens. Behind the lens, separate VFD windows on black; the lens's lower edge carries the white printed labels (bench photo).
2. **Left to right on the LHD meter:** a **fuel** window (bars with F and E, a pump icon); the **centre window** with the tach across the top and the speed below; a narrow **temperature** window (one vertical bar, H and C); a window with **oil pressure** and **turbo boost** as two slanted bar gauges (bench photo). The JDM meter puts boost and oil at the left end next to the TEMS and door module (owner teardown, Supramania).
3. **No gear indicator on the manual car.** Automatic cars show the selector position in a separate block; the manual car's meter has none (Supramania; bench photo shows none).

### The tachometer
4. **Horizontal, rising to the right.** The tach is a row of thin upright bars whose tops follow a gentle curve that climbs from the 0 at the left to the 8 at the right, over a thin curved baseline; the bars thicken towards the top end (bench photo; catalogue "horizontal tach").
5. **Numerals 0 to 8 above the bars, following the same curve**, upright, about one cap height above the bar tops; "TACH ×1000r/min" printed at the top left of the window (bench photo).
6. **The red zone is red numerals (7, 8) under a red bracket** that starts near 6,500 rpm and turns down at its ends; the bars themselves stay green (bench photo).

### Speed and readouts
7. **The speed is the largest figure**: three green-blue seven-segment digits about half the window's height, with a small "km/h" to the right and "SPEED" printed under the window (bench photo; catalogue "large speed numerals"). The JDM meter reads to 399 km/h (owner teardown).
8. **A small trip readout left of the speed** ("0.9 km", labelled TWIN TRIP), and a small mode box at its left (bench photo).

### The bar gauges
9. **Fuel: two columns of thick horizontal segments**, F at the top and E at the bottom (bench photo).
10. **Temperature: one vertical bar in a narrow window**, its top cut on a slant, H and C (bench photo).
11. **Oil and turbo: slanted parallelogram bar gauges** whose segments lean to the right, with small scale figures beside them (0, 4, 8 kg/cm² for oil; −2 to +0.5 for boost, ×100 mmHg) and their icons and names printed below (bench photo).

### Colour, light and glass
12. **VFD phosphor green and blue-green on black**: the bars are yellow-green, the speed digits a bluer green, the numerals and printed labels near white, the red zone red; the catalogue lists yellow, blue, red, green and orange tubes (bench photo; catalogue).
13. **The panel is glass-fronted VFD packages behind a three-layer lens with a tinted acrylic filter**: off segments stay faintly visible as dark ghosts through the filter, and the lens shows the room's reflections across its whole width (owner teardown, Supramania; bench photo).
14. **Printed labels in a plain upright sans with small pictograms** (fuel pump, thermometer, oil can, turbo) (bench photo).

### Behaviour
15. **The panel lights segment by segment from black** when the ignition comes on; VFD segments switch, they do not fade like bulbs (owner videos of the swap; general VFD behaviour).

## How the dash matches each feature

| # | Feature of the real meter | In the dash | Match |
|---|---|---|---|
| 1 | One long low panel behind a smoked lens, separate VFD windows on black, labels printed on the lens | One wide lens (kit optical glass, clear with a smoked absorption, crisp rims) over separate black VFD windows; the labels are printed in the lens's lower band | matched; the panel is taller than 6:1 in the cockpit view (change A) |
| 2 | Left to right: fuel, centre (tach over speed), temperature, oil and turbo | Left to right: pedal bars, centre (tach over speed), slip bar, power and boost slanted bars | layout matched; contents change B |
| 3 | No gear indicator on the manual car | A gear window in the centre window, right of the speed under the red zone | deliberate change C |
| 4 | Horizontal tach, bars rising to the right on a curved baseline | The same, one bar per 250 rpm, the bar tops on a rising curve | matched |
| 5 | Numerals 0 to 8 on the same curve, "TACH ×1000r/min" | The same; the scale reads to the car's limiter rounded up to a whole thousand | matched |
| 6 | Red numerals and a red bracket over the red zone, green bars | The same; the bracket is the shift light (change D) | matched, change D |
| 7 | Big blue-green seven-segment speed, "km/h", "SPEED" | The same, ghost `888` behind it, the user's unit | matched |
| 8 | Small trip readout left of the speed | Lateral G, "0.9 G" in the same place, "G-LOAD" under it | change B |
| 9 | Fuel: two columns of thick segments | Throttle and brake columns | change B |
| 10 | Temperature: one vertical bar, slanted top | Body slip angle bar, slanted top | change B |
| 11 | Oil and turbo slanted bars with scale figures | Power and boost slanted bars with scale figures | power for oil (change B); boost matched |
| 12 | Green, blue-green, white labels, red zone | The family's phosphor roles: green bars, cyan digits, white labels, red zone, magenta limiter | matched (change E for magenta) |
| 13 | Glass-fronted VFDs behind a tinted filter: ghosts visible, lens reflections | 7 % ghosts, VFD grid mesh over lit phosphor only, kit optical glass with the studio's reflections | matched |
| 14 | Plain upright sans labels with pictograms | Titillium Web labels (OFL) with drawn pictograms | closest open match |
| 15 | Segments switch on from black at ignition | A sector boot: the windows light left to right, segments switching, never fading | matched |

## Deliberate changes and why

- **A. Proportion.** The real panel is about 6:1; the cockpit envelope is 1440 × 560 (2.6:1). The dash keeps the A70's order and its long low windows but stacks them taller so the speed, gear and tach reach the sizes the tokens and ISO 15008 ask for. The hood view (1760 × 400) is closer to the real proportion.
- **B. Contents of the side gauges.** Forza's telemetry carries no fuel level, coolant temperature, oil pressure or trip distance. The windows show verified fields instead, in the same shapes: throttle and brake (the fuel window's two columns), body slip angle (the temperature bar; `Signals::slip_deg`), engine power (the oil slant, from `power_watts`) and lateral G (the trip readout). Boost stays boost (`boost_psi`).
- **C. A gear window.** The rank order puts the gear first, and the manual A70 has none. It sits in the centre window under the red zone, where the bench photo shows empty black, as a seven-segment digit larger than the speed.
- **D. The red bracket is the shift light.** The A70 has no shift light. The red bracket over the red zone becomes a row of red segments that fill from the shift point and strobe on the limiter, so the shift cue sits exactly where the real meter marks its red zone.
- **E. Magenta on the limiter.** The Sci-Fi family's accent is reserved for the shift and the limiter (`docs/design/families/sci-fi.md`); the bracket's limiter strobe takes it, everything else keeps the A70's colours. The gear stays white and takes a soft glow on the limiter, because a magenta gear measured 6.3:1 in the tokens' blur gate, under the gear's 7:1.
- **F. The 2027 step.** A production 2027 VFD revival: the windows are real glass-fronted tube packages rendered in Blender (glass envelope, getter, filament wires, the grid mesh), behind a lens computed by the kit's optics (`docs/specs/optics.md`: Fresnel reflection, refraction at the rims, absorption in the smoked filter), lit by one studio light that swings with lateral G. The phosphor shows its grain and the grid mesh at 1:1, not only when zoomed. Whole tach bars switch as on the A70, and the next bar is dimmed by its grid duty to the rpm between two bars (modern VFD drivers set brightness per grid by duty), so the tach moves on every frame the rpm moves, never with a fade in time.

## Gaps (not verified)
- The exact number of tach bars per 1,000 rpm is not countable at the bench photo's resolution (about 4 to 5 per 1,000 rpm); the dash uses 4.
- The JDM meter's full left-end layout (TEMS, door and boost) is known only from a low-resolution preview; the dash follows the clear LHD photo.
- No source gives the VFD's phosphor wavelengths; the colours are taken from the photo and the family's roles.
- The lens's layer stack is from one owner teardown only.
