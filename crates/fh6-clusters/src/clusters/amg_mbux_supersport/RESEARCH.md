# AmgMbuxSupersport research: the AMG-only Supersport menu of second-generation MBUX

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Started 2026-11-28. Text sources first (press releases and MBUSA owner's-manual supplements), then photographs and owner-video frame grabs of the real display. Every point names its source. Where no source says something, it is listed under Gaps and not guessed. No design is made here: the layout is drawn in Penpot (step 2) from what this file records.

Status: **text sources done; photo and frame-grab pass in progress** (the cloud container cannot download images, so the reference set is being gathered on the PC into `mockups/references/AmgMbuxSupersport/`, git-ignored, and mirrored to the Drive masters folder `amg_mbux_supersport/references/`).

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
- **The rev counter on the 2024 AMG GT runs inside-to-outside.** "A new heads-up-display mirrors the instrument screen, both with an interesting inside-to-outside style rev counter" ([MBWorld, refreshed AMG GT tech details](https://mbworld.org/?p=52580)). This is the one text source on the second-generation rev counter's shape; the frame grabs must confirm it.
- **Flames that answer the throttle.** On the C 63 S E Performance, testers preferred the "Supersport design", with "little orange flames" that react to throttle input; others chose Classic for its "more easily readable trad rev counter and larger gear indicator" ([evo, C 63 S E Performance review](https://www.evo.co.uk/mercedes/c63/205529/mercedes-amg-c63-s-e-performance-2023-review)). So the real Supersport has a throttle-driven fire element, is less of a plain rev counter than Classic, and sets the gear smaller. The frame grabs must show where the flames sit and how they move.
- **Head-up display pairing.** The HUD has "unique Race and Supersport settings" focused on vehicle and engine speed ([carsales, C 63 S E Performance launch review](https://www.carsales.com.au/editorial/details/mercedes-amg-c-63-s-e-performance-2023-review-138799)).
- **The screen hardware.** SL: "a fully digital 12.3-inch high-resolution instrument cluster, which is integrated into a three-dimensional visor" ([NVIDIA blog, AMG SL MBUX](https://blogs.nvidia.com/blog/mercedes-amg-sl-mbux-intelligent-performance)). S-Class step: 2400 × 900 px (`amg-display-styles.json`).

### The first-generation Supersport, for lineage

Mercedes describes the first-generation Supersport (A 45 / CLA 45, 2019; AMG GT 4-Door, 2018) as "a central, round rev counter" with "additional information presented in the form of bars to the left and right of the rev counter", drawn "with a three-dimensional perspective" so that they "reach far into the background to an artificial horizon" ([A 45 / CLA 45 release, Automotive World](https://www.automotiveworld.com/news/the-new-mercedes-amg-a-45-4matic-and-cla-45-4matic-the-super-sportscars-in-the-compact-class-2/)). In manual mode "a prominent prompt to change up a gear ('shift light')" appears, and the gear shows a yellow `M` (same family of releases, in `amg-display-styles.json`). The catalog brief's "wings to an artificial horizon" comes from this text. Whether the second-generation screen keeps it is a frame-grab question (gap 1).

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

- **The horizon is the rev limiter.** The first-generation bars "reach far into the background to an artificial horizon". That horizon can carry the engine: a tunnel of rings running from the horizon to the front as rpm rises, the front ring reaching the viewer at the shift point (TJS's own image, 08:18).
- **Subtle clouds on the horizon**, lit by load (throttle × rpm), so the sky behind the wings is calm at a cruise and bright through a pull.
- **The dyno in the wings** (catalog brief): torque rises, peaks and falls while power keeps climbing, each leaving a hold tick at its maximum. Both are real curves from `power` and `rpm`.
- **G-force as the horizon's tilt and pitch**: lateral acceleration rolls the horizon line, longitudinal moves it, so the G page lives in the whole scene and not only in a small widget.
- **Gear change as a pass through the tunnel**: on an upshift the rings rush past and the new gear arrives from the horizon.

## Gaps (not guessed; to close from the frame grabs)

1. **The second-generation layout.** Whether the centre is still a round rev counter, what "inside-to-outside" looks like, and whether the torque and power bars still run in perspective to an artificial horizon.
2. **The shape of the torque and power elements**, their scales, numerals and units, and where their digital values sit.
3. **Where the digital speed sits**, and its size against the gear.
4. **The rev scale**: numerals, tick pattern, the red area's start, and the upshift bar's shape and place on the cluster.
5. **Colour and material**: how much of it is red, the greys, any depth or rendered metal, glow.
6. **Motion**: how the rev counter, the bars and the gear move while driving and at a gear change.
7. **The typeface** (not documented anywhere opened).
8. **The centre pages**: what the G-force and engine-data pages look like in Supersport.

## Sources still to open

- Press galleries for the AMG GT C192, SL R232 and C 63 S E Performance (Mercedes media), and reviews with interior photos ([CarExpert](https://www.carexpert.com.au/car-reviews/2025-mercedes-amg-e53-hybrid-review), [carsales C 63](https://www.carsales.com.au/editorial/details/mercedes-amg-c-63-s-e-performance-2023-review-138799), [Digital Trends C 63](https://www.digitaltrends.com/cars/2025-mercedes-amg-c63-s-e-performance-plug-in-hybrid-review/), [evo C 63](https://www.evo.co.uk/mercedes/c63/205529/mercedes-amg-c63-s-e-performance-2023-review), [vehiclesizes AMG GT cluster photo](https://www.vehiclesizes.com/cars/mercedes-benz/amg-gt/amg-gt-c192-coupe-2024/images/41546-digital-instrument-cluster-touchscreen-quilted-leather)).
- Owner videos of the SL 63, AMG GT 63 and C 63 cluster modes for frame grabs (being gathered on the PC).
- Shared notes from `AmgMbuxSuperscreen` (`amg_mbux_superscreen/RESEARCH.md`, "Shared AMG MBUX notes", on its branch until it merges): read; palette, type and motion timing are taken from there.
