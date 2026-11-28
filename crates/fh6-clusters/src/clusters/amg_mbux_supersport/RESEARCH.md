# AmgMbuxSupersport research: the AMG-only Supersport menu of second-generation MBUX

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Started 2026-11-28. Text sources first (press releases and MBUSA owner's-manual supplements), then photographs and owner-video frame grabs of the real display. Every point names its source. Where no source says something, it is listed under Gaps and not guessed. No design is made here: the layout is drawn in Penpot (step 2) from what this file records.

Status: **text sources and the photo pass done** (30 photos, 21 of them second-generation, gathered on the PC into `mockups/references/AmgMbuxSupersport/`, git-ignored, and mirrored to the Drive masters folder `amg_mbux_supersport/references/` with a `sources.md` naming each photo's page). They are third-party photographs, for study only: never committed, shipped or traced. **Still open:** a driving frame at mid-revs and at the limiter in the blue colourway, and a gear change (owner-video stills).

## Deliberate changes (read first)

The catalog's AMG rules ask every AMG `RESEARCH.md` to open with these (`program/phases/07-cluster-factory/CATALOG.md`, "The five AMG dashes").

1. **No flashing area outside the shift strip.** The real cars flash display segments (the RACE START segments "flicker rapidly", CLE and C-Class supplements). In this dash only the shift strip strobes: 10 Hz at 50% duty, or a 2 Hz breath in reduced flash ([TOKENS.md](../../../../../docs/design/TOKENS.md), limiter strobe). Nothing large flashes.
2. **Deltas by role, with a sign.** Supersport shows no lap delta. Any faster or slower cue uses the `go` and `warning` roles with a sign and a direction, never colour alone.
3. **No maker's marks.** The logo zone shows the driven car's logo or the custom name. No AMG, Mercedes or star mark is drawn, and no asset carries one.
4. **Palette roles, not literal colours.** The dash uses the shared `amgMbux` family palette (`paletteModes.familyDefaults.amgMbux`, set by `AmgMbuxSuperscreen`), red structure allowed past the 5% red budget where the real display is red (TJS, 2026-11-28 06:24, "Red, like the real display"). The yellow manual `M` is drawn in a palette role, not a literal yellow.
5. **A swappable font.** Mercedes's cluster typeface is not freely licensed and no source names it. The dash uses the family the other AMG dashes share (Barlow Semi Condensed, OFL), and the user's display font replaces the numerals.
6. **No housing slab.** The real 12.3-inch screen sits in a cowl or visor (SL: "integrated into a three-dimensional visor", NVIDIA; "recessed and surrounded by a shroud", DriveLife). The dash draws no screen rectangle or bezel: the display's parts float on OLED black, "what MBUX is but better" (TJS, 2026-11-28 07:07, all MBUX dashes).
7. **What the telemetry cannot drive is dropped, and its space gets a bold idea of our own.** Temperatures, set-up, consumption, trip, audio, navigation and the status rows have no telemetry, so they go. The centre pages kept are the ones the packet can drive: gear, G-force, engine data (power, torque, boost). The freed space is filled with a telemetry-driven idea drawn on the Penpot board (TJS, 2026-11-28 08:18: "You just drop them and fill with something else like imagine subtle clouds and horizon tunnel background and the tunnel from the back to front being the rev limiter"; `CLUSTER_PROMPT.md`, "The bar", 09940b2). Candidate ideas, for the board to decide, are under "Ideas for the board" below.

## What this dash recreates

**The AMG-only "Supersport" menu of the second-generation MBUX driver display**: the 12.3-inch landscape screen of the Mercedes-AMG GT (C192, 2024 on), SL (R232, 2022 on), C 43 / C 63 S E Performance (W206), S 63 E Performance and the W214-step CLE 53 / E 53.

- **The menu exists on these cars.** DriveLife lists the SL 63's eight menus as "Understated, Sport, Classic, Navigation, Assistance, Service, Supersport and Track Pace" ([DriveLife, 2023 SL63 review](https://www.drivelife.co.nz/2024/04/2023-mercedes-amg-sl63-4matic-car-review/)). The CLE 53 supplement lists Classic, Understated, Service, Supersport and TRACK PACE ([MBUSA, MY24 AMG CLE Coupe supplement](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf)).
- **The legend, combustion cars** (CLE 53, p. 26): 1 Current torque, 2 Center display content (example: gear display), 3 Current power output ([MBUSA, CLE supplement](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf)).
- **The legend, plug-in hybrids** (C 63 S E Performance, pp. 35–36): 1 Current torque and recuperation display, 2 Center display content (example: selected gear and electric drive support display), 3 Current power and recuperation display ([MBUSA, MY24 AMG C Sedan supplement](https://Mbusa.Com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20C%20Sedan%20Owners%20Manual.pdf)).
- **Centre content is chosen from a vertical menu.** "The AMG-specific Supersport style offers the option of displaying various contents via a vertical menu structure" (Mercedes press, recorded with its link in `/mnt/project-files/dash-research/amg-display-styles.json`). The supplements list: transmission position / gear display, temperatures (transmission oil, engine oil, coolant; hybrids: battery and motor), engine data (current power output and current torque), set-up (drive, AMG DYNAMICS, suspension, sound), G-force, trip and total distance, From Start / From Reset, consumption, audio, navigation (CLE and C-Class supplements).
- **The rev counter on the 2024 AMG GT runs inside-to-outside.** "A new heads-up-display mirrors the instrument screen, both with an interesting inside-to-outside style rev counter" ([MBWorld, refreshed AMG GT tech details](https://mbworld.org/?p=52580)). The photographs confirm it ("What the photographs show", item 3).
- **Flames that answer the throttle.** On the C 63 S E Performance, testers preferred the "Supersport design", with "little orange flames" that react to throttle input; others chose Classic for its "more easily readable trad rev counter and larger gear indicator" ([evo, C 63 S E Performance review](https://www.evo.co.uk/mercedes/c63/205529/mercedes-amg-c63-s-e-performance-2023-review)). So the real Supersport has a throttle-driven fire element, is less of a plain rev counter than Classic, and sets the gear smaller. The photographs show them: the flames are the two fire walls that stand on the rev scales (item 4).
- **Head-up display pairing.** The HUD has "unique Race and Supersport settings" focused on vehicle and engine speed ([carsales, C 63 S E Performance launch review](https://www.carsales.com.au/editorial/details/mercedes-amg-c-63-s-e-performance-2023-review-138799)).
- **The screen hardware.** SL: "a fully digital 12.3-inch high-resolution instrument cluster, which is integrated into a three-dimensional visor" ([NVIDIA blog, AMG SL MBUX](https://blogs.nvidia.com/blog/mercedes-amg-sl-mbux-intelligent-performance)). S-Class step: 2400 × 900 px (`amg-display-styles.json`).

### The first-generation Supersport, for lineage

Mercedes describes the first-generation Supersport (A 45 / CLA 45, 2019; AMG GT 4-Door, 2018) as "a central, round rev counter" with "additional information presented in the form of bars to the left and right of the rev counter", drawn "with a three-dimensional perspective" so that they "reach far into the background to an artificial horizon" ([A 45 / CLA 45 release, Automotive World](https://www.automotiveworld.com/news/the-new-mercedes-amg-a-45-4matic-and-cla-45-4matic-the-super-sportscars-in-the-compact-class-2/)). In manual mode "a prominent prompt to change up a gear ('shift light')" appears, and the gear shows a yellow `M` (same family of releases, in `amg-display-styles.json`). The catalog brief's "wings to an artificial horizon" comes from this text. The second generation drops the round dial and keeps the depth: the bars became a whole tunnel with the horizon at its end.

## What the photographs show (second generation)

Studied at full size from the reference set (file names from `sources.md`, each with its page there). The second-generation Supersport is **not** a round dial: it is a perspective tunnel. It comes in two colourways, chosen by the drive program.

### The blue colourway (Comfort, Sport; the press and review photos)
From `gt63_c192_supersport_idle_01.jpg` (AMG GT 63, [Car and Driver gallery](https://www.caranddriver.com/photos/g45781348/2024-mercedes-amg-gt-63-drive-gallery/)), `cle53_c236_supersport_idle_01.jpg` (CLE 53, [cforcars](https://cforcars.com/news/introducing-the-mercedes-amg-cle-53-coupe/)) and `gt63_c192_supersport_gforce_page_01.jpg` (AMG GT 63, [MotorWeek](https://motorweek.org/road-tests/track-tests/2024-mercedes-amg-gt-63-coupe/)).

1. **A tunnel seen from inside.** Concentric arched ribs recede to a vanishing point a little above the screen's centre. The walls carry a fine dotted halftone; a dark horizon band with faint hill silhouettes sits at the vanishing point.
2. **A lit floor.** A blue perspective floor of long lanes (about seven lane lines) runs from the horizon to the bottom band, brightest near the viewer.
3. **The rev counter is two horizontal scales that run inside-to-outside.** A thin line on each side of the vanishing point, numbered from the centre outwards, mirrored: 0, 1, 2, 4, 6, 8 with "x1000/min" under the outer end. The spacing grows outwards, as perspective would place equal steps receding into the tunnel. The last segment is red (about 7 to 8 on the GT 63's scale), with red end ticks. This is MBWorld's "inside-to-outside style rev counter".
4. **The lit rev line crackles and ripples like a fuse.** Where the fire has passed, the scale line is a jagged yellow-white ember core that ripples along its length with small sparks spitting off it, brightest at the wall (`gt63_c192_supersport_idle_01.jpg`; pointed out by TJS, 2026-11-28 09:22).
5. **The fire walls are the needle.** On each scale stands a glowing orange-to-red wall, a cross-section of the tunnel lit like flame (evo's "little orange flames"), with a bright yellow-white rim on its leading edge and a halftone glow behind. The wall stands at the current rpm, so as revs rise the two walls travel out from the vanishing point towards the viewer, back to front. At idle on the CLE 53 they stand just inside 1; the GT 63 photo shows them near 4.
5. **Torque and power are thin outer arcs.** Outside the tunnel, a tall thin arc on each side: torque on the left ("Nm", 0 to 800 on the GT 63, 0 to 600 on the CLE 53), power on the right ("kW", 0 to 430 or 330). Each has a small live value in orange or red near its top ("0") and a short lit bar segment at its zero. The hybrids label the right side POWER and CHARGE.
6. **Speed at the top centre.** A light, slanted numeral ("0", about 6% of the screen height) with "km/h" or "mph" under it, between two thin horizontal lines, flanked by assistance icons.
7. **A bottom band** with the maker's logo in the middle, fuel and range left, outside and coolant temperature right, the time, telltales and the R N D P stack at the far right.
8. **The G-Force page** turns both outer areas into panels: a G-Force target with rings at 0.5, 1.0 and 1.5 g, a red dot and red maxima on the left; speed, gas-pedal % and brake-pedal % on the right. The tunnel and its fire walls stay in the middle.

### The red colourway (Sport+, Race)
From `sl55_r232_supersport_sportplus_red_idle_730rpm_01.jpg` (SL 55, [CarBuzz](https://carbuzz.com/reviews/driven-2022-mercedes-amg-sl-55-proves-that-you-dont-need-the-63/)), `sl63_r232_supersport_sportplus_red_1500rpm_01.jpg` (SL 63, [mercedesblog](https://mercedesblog.com/mercedes-amg-sl-63-4matic-test-drive-gt-or-sports-car-or-both/)) and `glc63se_x254_supersport_red_6400rpm_launch_yt_01.jpg` (GLC 63 S E, [YouTube launch video](https://www.youtube.com/watch?v=MzRyBQHS-SI), a frame with a timing overlay).

9. **Red crescent rings round a lit sun.** The tunnel becomes nested red crescents, "( )", thick at 9 and 3 o'clock, with a bright white-violet glowing disc at the horizon and a dark mountain horizon line. The walls keep the dotted halftone.
10. **A large rpm readout** in the floor ("730", "1500", "6400", light slanted figures with "rpm" or "1/min" under it), larger than the speed.
11. **Torque and power as thin vertical arcs** close to the tunnel, with a glowing white dot at the live value ("0 lb ft", "0 HP"). Under load they fill with a red bar and a live value at the top (the 6400 rpm frame: about 118 and 79 against scales to 945 and 345).
12. **At high revs the red floods outwards**: in the 6400 rpm frame the crescents are brighter and wider, and their inner rim turns white. Whether any of it flashes is not visible in a still (gap 6).
13. **Side lists**: temperatures (engine oil, transmission oil, coolant, and on hybrids battery and motor) on the left; the drive-program stack (Race, D, Pro or Master, Sport+, Powerful, ESP) on the right.

### What carries across both
- No round dial, no needle: the rpm is shown by where the light stands in depth (blue) or by how much red floods the rings (red), plus a figure.
- Thin, light, slanted numerals; small grey labels; one warm hue (orange-red) for live engine state on a cool ground (blue or black).
- Dotted halftone on every lit surface, and a horizon with a glow at the vanishing point.
- The real screen sits in a visor with a bezel; our dash drops it (change 6).

## Telling it apart from AmgMbuxSuperscreen

Its sibling's v4 board (Penpot, 2026-11-28; `amg_mbux_superscreen/board_v4/states_v4.png` in the project files) is a red-on-black sky tunnel: two tall red halftone crescents, grey round rings that light back to front, a road floor, a round G-Force widget left and a round boost gauge right, grey scale bands with numerals. At thumbnail size Supersport must read differently. What the real Supersport gives us that the sibling lacks:

- **Horizontal, mirrored rev scales** crossing the tunnel at the horizon, numbered outwards, instead of round rings.
- **Two orange fire walls** that travel along those scales, warm amber and yellow-white instead of a red fill.
- **A wide, lit lane floor** (blue in the real default colourway) instead of a dark road.
- **Thin outer torque and power arcs** instead of heavy grey bands, and no side widgets in the default page (the G-Force page replaces the outer arcs only when chosen).

**TJS chose the blue colourway on 2026-11-28 at 08:56** ("The top blue is what we're after", on the reference sheet whose top row is the AMG GT 63 and CLE 53 blue photos). So the board's lead is the real default colourway: a cool, dark-blue tunnel with fire walls, which no sibling uses. The red Sport+ look (crescents round a horizon sun, big rpm figure) is close to the sibling's red; if the dash carries it at all, it is the second look the driver picks, with the sun as its own mark. This weighs against the shared red `amgMbux` default (TJS chose red for the Sporty style on 2026-11-28, and the family note allows red "where the real display is red"): Supersport's real default is blue with orange, so the board proposes its own family default for this dash and logs the reason.

## Behaviour the manuals document

- **Refused shift.** "If the engine speed is too high or too low, you will not be able to change gear using the steering wheel paddle shifters", and segments "light up red on the driver's display" (CLE p. 13; C-Class p. 21).
- **Red area.** "Shift up before the engine speed reaches the red area in the tachometer" (CLE p. 13; C-Class p. 21). The fuel supply is cut "when the red mark on the rev counter (overrevving range) is reached" ([Mercedes online manual, A-Class family](https://moba.i.mercedes-benz.com/baix/cars/118.3_mbux-high_2021_b/en_GB/page/ID_87d5596c493762d8354ae365689e58e4-eb9741d4571e5e22354ae365157ca12b-en-GB.html)).
- **Upshift bar.** "If you activate manual gearshifting, the upshift bar will appear on the multifunction or head-up display", and "the transmission position (manual) and the current gear will appear on the transmission position display" ([Mercedes online manual, AMG GT 4-Door family](https://moba.i.mercedes-benz.com/baix/cars/257.3_audio_2020_b/en_GB/page/ID_87d5596c493762d8354ae365689e58e4-7f8789aee8f0bf57ac190d2b518cb50b-en-GB.html)).
- **RACE START.** "The segments on the driver display will flicker rapidly" (CLE p. 18). This dash does not flash them (change 1).
- **G-meter maxima** stay as red marks until reset (TRACK PACE and first-generation G-meter; `amg-display-styles.json`).
- **Head-up display Supersport context** (CLE p. 28): over-rev protection, current speed, selected gear with gearshift options for manual shifting, current engine speed.

## Telemetry check (what the packet can drive)

Verified fields per `CATALOG.md` ("Flash and telemetry"): rpm (current, idle, max), speed, gear, throttle, brake, steer, acceleration, local velocity, power, boost, timestamp.

| Real element | Packet source | Note |
|---|---|---|
| Current torque | derived: τ = P · 60 / (2π · rpm), rpm clamped to idle | torque is not a verified field; label it as derived in the handover |
| Current power output | power, in `signals.power_unit` | verified |
| Recuperation part of each scale | none | combustion layout only; no negative part |
| Rev counter, red area, upshift bar | rpm, idle, max | the scale comes from the driven car's max rpm |
| Gear display and `M` | gear | `M` always on (TJS drives manual) |
| Refused-shift red segments | not sent | left out, or inferred only if a later field allows it |
| G-force page | acceleration | verified |
| Engine data page | power, derived torque, boost | verified (boost) |
| Temperatures, set-up, trip, consumption, audio, navigation | none | left out (change 7) |

## Ideas for the board

Leads for step 2, each driven by verified telemetry. None is a design yet; the board chooses and draws.

- **The tunnel is the rev counter, back to front.** This is the real display's own idea (the fire walls travel from the vanishing point to the viewer as rpm rises) and TJS's image at 08:18. Ours can go further: real depth for the walls (Blender), the front of the tunnel reaching the viewer exactly at the shift rpm, and the red segment as a gate the walls pass through.
- **Subtle clouds on the horizon** (the real one has faint hills and a glow there), lit by load (throttle × rpm), so the sky behind the wings is calm at a cruise and bright through a pull.
- **The dyno in the wings** (catalog brief): torque rises, peaks and falls while power keeps climbing, each leaving a hold tick at its maximum. Both are real curves from `power` and `rpm`.
- **G-force as the horizon's tilt and pitch**: lateral acceleration rolls the horizon line, longitudinal moves it, so the G page lives in the whole scene and not only in a small widget.
- **Gear change as a pass through the tunnel**: on an upshift the rings rush past and the new gear arrives from the horizon.

## Gaps (not guessed)

1. **Motion while driving (blue colourway).** How fast the fire walls travel, whether they flicker, and what happens at the red segment and the limiter. No driving still was found; owner-video frames are the next step.
2. **A gear change.** Where the gear is shown in Supersport (none of the photos shows a gear figure in the centre; the manual's example puts it in the centre content) and how it changes.
3. **The scale's ends.** The GT 63 and CLE 53 both end at 8 with red from about 7; the scale for other engines is not seen. The dash scales from the driven car's max rpm anyway.
4. **The red colourway at the limiter**: whether the crescents pulse. The dash does not flash them (change 1).
5. **The typeface** (not documented anywhere opened).

## Sources still to open

- Press galleries for the AMG GT C192, SL R232 and C 63 S E Performance (Mercedes media), and reviews with interior photos ([CarExpert](https://www.carexpert.com.au/car-reviews/2025-mercedes-amg-e53-hybrid-review), [carsales C 63](https://www.carsales.com.au/editorial/details/mercedes-amg-c-63-s-e-performance-2023-review-138799), [Digital Trends C 63](https://www.digitaltrends.com/cars/2025-mercedes-amg-c63-s-e-performance-plug-in-hybrid-review/), [evo C 63](https://www.evo.co.uk/mercedes/c63/205529/mercedes-amg-c63-s-e-performance-2023-review), [vehiclesizes AMG GT cluster photo](https://www.vehiclesizes.com/cars/mercedes-benz/amg-gt/amg-gt-c192-coupe-2024/images/41546-digital-instrument-cluster-touchscreen-quilted-leather)).
- Owner videos for driving frames: [SL 55 display options](https://www.youtube.com/watch?v=w-lHb2vPOv8), [GLC 63 S E launch](https://www.youtube.com/watch?v=MzRyBQHS-SI), [S-Class 3D mode](https://www.youtube.com/watch?v=qgIbDSRLuX0) (a sister style).
- Shared notes from `AmgMbuxSuperscreen` (`amg_mbux_superscreen/RESEARCH.md`, "Shared AMG MBUX notes", on its branch until it merges): read; palette, type and motion timing are taken from there.
