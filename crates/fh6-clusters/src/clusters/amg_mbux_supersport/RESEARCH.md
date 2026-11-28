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
6. **No housing slab.** The real 12.3-inch screen sits in a cowl or visor (SL: "integrated into a three-dimensional visor", NVIDIA; "recessed and surrounded by a shroud", DriveLife). The dash draws no screen rectangle or bezel: the display's parts float on OLED black, "what MBUX is but better" (TJS, 2026-11-28 07:07, all MBUX dashes). Build note: every translucent part drawn over the overlay's clear pixels uses `bezel::over_clear(level)` (main 9c2eb1c); `bezel::on_screen` is only right over an opaque body and comes out about 2.5 times too dim here. Any part that sits on the switchable housing uses `bezel::level_alpha(level, housing::shown(...))`.
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
4. **The lit rev line crackles and ripples like a fuse.** Where the fire has passed, the scale line is a jagged yellow-white ember core that ripples along its length with small sparks spitting off it, brightest at the wall and thinning to nothing at the vanishing point. A second, fainter flame edge runs along the floor edge under the jet, so the jet reads as a burning slab between two ragged flame edges (`gt63_c192_supersport_idle_01.jpg` at 3x; pointed out by TJS, 2026-11-28 09:22).
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

## Board v3 notes (TJS, 2026-11-28 09:52; to draw after LIBRARY DONE)

TJS on board v2: "We don't need the top rev limiter but we're getting close look back at the Mercedes ripple and give some space depth the mph and number I front of the gear".

1. **No upshift strip.** The 12-cell strip above the speed goes. The shift cue is the real one's: the fire jet entering the red gate, the gate glowing steady and the wall's front burning white. With no strip, nothing on this dash strobes at all (the house rule allows less flashing, never more); the limiter is a steady state. Logged as a deliberate change in the handover.
2. **The real ripple, studied at 2x** (`gt63_c192_supersport_idle_01.jpg`, the left scale from 0 to 4). It is not a jagged zig-zag, as v2 drew it. It is a liquid plasma ribbon: a white-yellow core about as thick as a numeral stroke, with soft orange edges, whose thickness swells and pinches in smooth, uneven bulges (blob-like, a few per scale segment), slightly wavy in height by a pixel or two, brightest and thickest at the wall and tapering to a hairline at 0. The floor-edge flame under it is the same ribbon, fainter and calmer. The jet's dot volume lies mostly between the two ribbons (below the scale line), densest and brightest next to the line and the wall; above the line the dots are much fainter. The wall itself is a smooth, saturated orange arc with a soft glow, not a white stroke; it whitens only where the ribbon meets it.
   - Elevated: the bulges travel outward from the mesh like pulses in a hose (speed from rpm, size from throttle), and a burst of bulges runs out on a sharp throttle stab.
3. **Space and depth for the speed and the gear.** Both float in front of the tunnel as objects in their own layer: a soft dark shadow cast onto the tunnel behind them (offset down and toward the vanishing point), a faint blue rim light from the tunnel on their lower edges, and a parallax shift against the tunnel under lateral and longitudinal g (a few pixels; the tunnel moves, the figures hold), so they read as standing in the space between the driver and the mesh. The speed sits nearer (larger, bold) and the gear further in, at the mesh.

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

## Real behaviour study (second pass, 2026-11-28 10:45)

Read from four more stills in the Drive reference folder (GT 63 G-force page, SL 55 G-force centre, GT 63 S E drive-program menu, SL 63 idle). Pixel positions are measured on the photos, so they are approximate.

1. **The walls never sit at the vanishing point while the engine runs.** At idle (parked, about 700 rpm) both walls stand just inside the `1` tick, about 0.7 to 0.9 on the ×1000 scale. The thin bright front rim is a crisp arc; behind it a dotted orange fan reaches back to the mesh, and a flat orange glow runs along the scale line from the mesh to the wall.
2. **The rev scale is not linear.** Measured on the SL 55 still: 0 to 1 and 1 to 2 each take about 55 px, while every 2,000 rpm step above that takes about 80 px. So the 0 to 2,000 band is stretched to about 1.4 times a 2,000 step, which gives idle and pull-away room to move. Labels are 0 (small), 1 (small), 2, 4, 6, 8 (large, 8 in red). Minor ticks every 500 rpm.
3. **The red part starts at about 6,800 of 8,000 (85%)** and is a thicker red bar with an end cap, not a colour change of the thin line.
4. **Torque and power arcs** (SL 55): thin vertical arcs outside the tunnel. Each arc's top value is the car's rated peak (516 lb ft and 469 hp: the SL 55's real ratings), the live value is an orange numeral that sits about half way up, the unit sits at the bottom. The arc's own fill rises from 0. The units follow the car's region settings (lb ft and HP in the US, Nm and kW in Europe). The plug-in hybrid adds an inner `CHARGE` scale (0 to 300 Nm, 0 to 150 kW) for recuperation; dropped (no telemetry).
5. **The wings can hold panels instead of arcs.** The GT 63 G-force page puts a "G-Force" panel in the left wing (polar target, red dot, peak values in red on each axis, the total in g at the bottom) and a three-row panel in the right wing (Speed, Gas Pedal, Brake Pedal, each a label and a value). The panels are flat translucent slabs with a soft blue top edge, angled to follow the tunnel wall.
6. **Centre content sits on the vanishing point.** The G-force centre (SL 55) is a small polar target with rings at 0.5, 1.0 and 1.5 g, four axis values round it and the total below, labelled "G-Force". The drive-program pop-up (GT 63 S E) is a 2×3 grid of mode tiles labelled `Setup`. Trip data (SL 63) is two numbers either side of a short line with the unit. A column of nine small dots on the right edge marks the page in the vertical menu, with the current one in pink.
7. **Speed is a thin italic numeral** with a small unit below and two long hairlines either side that fade at their ends. TJS asked for it much bolder (09:12); the line pair stays and becomes the shelf the bold numeral stands on.
8. **The floor** is a bright halftone plane with lane lines converging on the vanishing point, brightest at the front edge, which is a curved cyan line. Low dark blue hills sit behind the scales in both wings.
9. **The fire colour shifts with the drive program**: redder in the SL 63 idle still (Race flag shown), oranger in the GT 63 S E still (Sport+). We keep one blue colourway, so the flame's heat follows load instead (see the motion notes).

## Motion notes (what moves, what drives it)

Real timing is still a gap until the video frames come back; these are the behaviours the dash will have, each tied to a signal. Durations are proposals for the board.

| Element | Moves with | Behaviour |
|---|---|---|
| Fire walls (both) | `signals.rpm` through the scale map below | Stand at the current rpm, mirrored. Critically damped spring, about 60 ms, fed forward by `signals.rpm_rate` so they do not lag a fast pull. |
| Plasma ribbon on the scale line | rpm and throttle | Smooth swelling blobs travel outward from the mesh to the wall; travel speed from rpm, blob height from throttle, thickest at the wall, a hairline at 0. White-yellow core, orange edge. |
| Second ribbon on the floor edge | same, at 40% | Fainter echo, half a blob out of phase. |
| Jets from the mesh | rpm fraction | The dotted fan behind each wall brightens and lengthens as rpm rises; above the red start the jets burn hotter (core towards yellow-white). No flashing anywhere (change 1). |
| Sparks | throttle, `signals.g_lat` | Rate follows throttle, drift follows lateral g. On an upshift they are drawn back into the mesh with the jet. |
| Upshift | `signals.gear` rises | The walls fall back to the new rpm on the spring (no snap); the rings rush past once (about 180 ms) and the new gear arrives from the horizon. |
| Downshift | gear falls | Walls jump forward on the spring with a short flare at the rim (about 120 ms), no rush. |
| Torque and power arcs | `telemetry.power_watts`, rpm | Fill rises with the live value; a hold tick marks the session peak and decays only on a car change. |
| Horizon | g_lat, g_long | Rolls with lateral g, lifts and drops with longitudinal g, capped at a few degrees and pixels. |
| Hills and clouds | throttle × rpm fraction | Lit from behind by load; calm at a cruise, bright through a pull. |
| Speed and gear | g (parallax) | Float in front with a shadow and a blue rim light; 0 to 4 px parallax from g. |

## Telemetry mapping (exact fields)

The dash gets `FrameInput` (`crates/fh6-kit/src/cluster.rs:355`): the raw packet `telemetry: &FH6TelemetryData` and the smoothed `signals: &Signals` (`cluster.rs:197`). The packet decoder already makes every float finite and bounded, but values can still be 0 (see the edge cases).

| Element | Source | Formula |
|---|---|---|
| Wall position | `signals.rpm`, `signals.max_rpm` | `x = scale(rpm)`, the piecewise map below. `max_rpm` is already `max(1.0)` and rpm is clamped to `0..=1.1·max_rpm`. |
| Scale end and labels | `signals.max_rpm` | `end = ceil(max_rpm / 1000)` thousand. Labels 0, 1, 2, then every 2 (every 4 above an end of 12). |
| Red bar | `signals.max_rpm` | From `0.85·end` to `end`, as measured on the real scale. Not tied to `shift_rpm`, which feeds nothing here because there is no strip. |
| Ribbon speed and height | `signals.rpm_fraction`, `signals.throttle` | speed ∝ rpm_fraction; height ∝ throttle (0..=1). |
| Speed | `signals.speed`, `signals.unit` | Rounded to a whole number with 0.6 hysteresis; unit text from `unit`. |
| Gear | `signals.gear` | `Gear::label()`: `R`, `N`, `1` to `10`. Raw 0 is reverse, 11 and up are neutral. `M` beside it always. |
| Power | `telemetry.power_watts` | `/ 745.7` for HP or `/ 1000` for kW, from `signals.power_unit`. |
| Torque | power and rpm | `τ = P · 60 / (2π · max(rpm, idle_rpm, 500))` in Nm; × 0.7376 for lb ft when power is in HP. Labelled derived. |
| Arc tops | session peaks | Held maximum of power and torque since the last `car_ordinal` change, rounded up to a clean step; starts at a guess of 400 kW and 700 Nm. |
| G (horizon and centre page) | `signals.g_lat`, `signals.g_long` | Already smoothed (60 ms). Total = `hypot(g_lat, g_long)`. |
| Sparks drift | `signals.g_lat` | Sideways drift ∝ g_lat. |
| Load light on the hills | throttle, rpm_fraction | `throttle · rpm_fraction`, eased over 400 ms. |
| Shown at all | `signals.race_on` | Off-race: scene idles at rpm with no sparks. |

**The scale map.** With `e = end` in thousands and `k = 1.4` (the measured stretch), the 0 to 2,000 band takes `k` units, every later 2,000 takes 1 unit:

- `rpm ≤ 2000`: `u = k · rpm / 2000`
- above: `u = k + (rpm − 2000) / 2000`
- normalised `s = u / (k + (e·1000 − 2000) / 2000)`, then `x = mesh_x + s · (front_x − mesh_x)` along the scale line.

For an end of 2 or less (rare) the map is linear.

## Edge cases (behaviour decided now)

| Case | What the dash does |
|---|---|
| `engine_max_rpm` is 0 or tiny | `signals.max_rpm` is 1, so the scale would collapse. If `max_rpm < 1500`, draw the scale with an end of 8 and park the walls at 0; no NaN. |
| `idle_rpm > max_rpm` (corrupt) | Use idle only as the torque floor; clamp it to `max_rpm`. |
| rpm above the limiter (to 1.1×) | Walls stop at the scale end; the extra 10% only heats the jets. |
| rpm below idle, or 0 (engine off, stall) | Walls ease back to the mesh; the ribbon becomes a hairline; jets dark. |
| Reverse | Gear shows `R`; the floor's lane lines run toward the viewer instead of away (slowly). Speed still positive. |
| Neutral | Gear `N`; walls follow rpm as normal (a free rev is shown). |
| Gear change mid-spring | The upshift rush only plays once per change; a second change within 250 ms restarts it, no stacking. |
| `power_watts ≤ 0` (engine braking, coasting) | Power and torque show 0; arcs empty; peaks untouched. |
| Power spike on a load jump | Peaks take a value only after it holds for 150 ms. |
| Car change (`car_ordinal` changes) | Peaks reset, scale rebuilt from the new `max_rpm`, walls glide to the new position. |
| Unit change (mph/km/h, HP/kW) live | Text swaps on the next frame; no animation. |
| Speed over 999 | Clamp the display to 999. |
| `race_on` false (menus, pause) | Hold the last frame dimmed to the static cap; no sparks or ribbon motion. |
| Very high max rpm (EV, 18,000+) | The label step rises to 4 (or 5 at an end above 20) so labels never collide. |
| Frame drop (large `dt`) | Springs step with `dt` clamped to 50 ms. |
| Reduced flash on | Already nothing flashes; the spark rate halves. |

## Ideas for the board

- **The burning, rippling fuse, made better** (TJS, 2026-11-28 09:22-09:23: "Can you see that almost fire spark it has on the line", "Rippling", "Intergaetr it and make it better"). The real line crackles; ours ripples as a travelling wave from the mesh out to the wall (wave speed and height from rpm), spits sparks whose rate follows throttle and whose drift follows lateral g, glows hotter (orange to yellow-white) as load rises, and leaves a short cooling afterglow when revs fall. On an upshift the sparks are sucked back into the mesh with the jet.

Leads for step 2, each driven by verified telemetry. None is a design yet; the board chooses and draws.

- **The tunnel is the rev counter, back to front.** This is the real display's own idea (the fire walls travel from the vanishing point to the viewer as rpm rises) and TJS's image at 08:18. Ours can go further: real depth for the walls (Blender), the front of the tunnel reaching the viewer exactly at the shift rpm, and the red segment as a gate the walls pass through.
- **Subtle clouds on the horizon** (the real one has faint hills and a glow there), lit by load (throttle × rpm), so the sky behind the wings is calm at a cruise and bright through a pull.
- **The dyno in the wings** (catalog brief): torque rises, peaks and falls while power keeps climbing, each leaving a hold tick at its maximum. Both are real curves from `power` and `rpm`.
- **G-force as the horizon's tilt and pitch**: lateral acceleration rolls the horizon line, longitudinal moves it, so the G page lives in the whole scene and not only in a small widget.
- **Gear change as a pass through the tunnel**: on an upshift the rings rush past and the new gear arrives from the horizon.

## Gaps (not guessed)

1. **Motion while driving (blue colourway).** Behaviour now decided in the motion notes; real timing still open. How fast the fire walls travel, whether they flicker, and what happens at the red segment and the limiter. No driving still was found; owner-video frames are the next step.
2. **A gear change.** Where the gear is shown in Supersport (none of the photos shows a gear figure in the centre; the manual's example puts it in the centre content) and how it changes.
3. **The scale's ends.** The GT 63 and CLE 53 both end at 8 with red from about 7; the scale for other engines is not seen. The dash scales from the driven car's max rpm anyway.
4. **The red colourway at the limiter**: whether the crescents pulse. The dash does not flash them (change 1).
5. **The typeface** (not documented anywhere opened).

## Sources still to open

- Press galleries for the AMG GT C192, SL R232 and C 63 S E Performance (Mercedes media), and reviews with interior photos ([CarExpert](https://www.carexpert.com.au/car-reviews/2025-mercedes-amg-e53-hybrid-review), [carsales C 63](https://www.carsales.com.au/editorial/details/mercedes-amg-c-63-s-e-performance-2023-review-138799), [Digital Trends C 63](https://www.digitaltrends.com/cars/2025-mercedes-amg-c63-s-e-performance-plug-in-hybrid-review/), [evo C 63](https://www.evo.co.uk/mercedes/c63/205529/mercedes-amg-c63-s-e-performance-2023-review), [vehiclesizes AMG GT cluster photo](https://www.vehiclesizes.com/cars/mercedes-benz/amg-gt/amg-gt-c192-coupe-2024/images/41546-digital-instrument-cluster-touchscreen-quilted-leather)).
- Owner videos for driving frames: [SL 55 display options](https://www.youtube.com/watch?v=w-lHb2vPOv8), [GLC 63 S E launch](https://www.youtube.com/watch?v=MzRyBQHS-SI), [S-Class 3D mode](https://www.youtube.com/watch?v=qgIbDSRLuX0) (a sister style).
- Shared notes from `AmgMbuxSuperscreen` (`amg_mbux_superscreen/RESEARCH.md`, "Shared AMG MBUX notes", on its branch until it merges): read; palette, type and motion timing are taken from there.
