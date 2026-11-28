# AmgMbuxSuperscreen research: the W214 E-Class driver display, Sporty style

Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Researched on 2026-11-28 from Mercedes-Benz press releases, MBUSA owner's manual supplements, reviews and owner forums (marked as such), and one press-grade photo of the real display studied at full size. Every point names its source. Where no source says something, the gap is listed at the end and not guessed.

## Deliberate changes (read first)

These are the places where the dash departs from the real display on purpose. The catalog's AMG rules ask every AMG `RESEARCH.md` to open with them (`program/phases/07-cluster-factory/CATALOG.md`, "The five AMG dashes").

1. **No flashing area outside the shift strip.** The real Sporty display has no shift light at all. The house rule is that only a shift strip strobes, and nothing large flashes ([TOKENS.md](../../../../../docs/design/TOKENS.md), limiter strobe). So the dash adds a thin 14-segment strip along the glass's top edge. Only that strip strobes, at 10 Hz on its own clock, or with a 2 Hz breath in reduced flash. At the limiter the crescents hold full and steady.
2. **Deltas by role, with a sign.** This style shows no lap or speed delta. If an AMG dash shows one (TRACK PACE), faster and slower use the `go` and `warning` roles with a sign and a direction, never colour alone.
3. **No maker's marks.** The logo zone (bottom left) shows the driven car's logo, or the custom name. No AMG, Mercedes or star mark is drawn anywhere, and no asset carries one.
4. **Palette roles, not literal colours.** The red is the dash's own default palette, `paletteModes.familyDefaults.amgMbux` in `data/design_tokens.json`: primary `#E3122E`, secondary `#C3C8CF`, accent and shift `#FFFFFF`. TJX chose "red, like the real display" on 2026-11-28, past the tokens' 5% red budget. It applies while the host palette is factory or family; a custom or random palette replaces it, and the Blender tunnel plate is luminance-only so it recolours with the primary.
5. **A swappable font.** The real numerals are Mercedes's corporate sans, which is not freely licensed. The dash uses Barlow Semi Condensed (SIL OFL, the pack's `dial` face), light and slightly slanted. If the user picks a display font in the OSD, the numerals follow it.
6. **The gear moves into the tunnel.** The real display sets the gear small, in the selector stack at the bottom right ("R N D P"). The house rank order puts the gear first, so the dash draws it large in the tunnel's vanishing disc, where the real display leaves a grey glow. The bottom-right box keeps the real one's place and shows `M` and the gear.
7. **The crescents are the rev counter.** In the photo the two red crescents stand at full length while the car is parked (0 rpm), so they read as decoration around the speed and the rpm digits, not as a fill. The dash makes them fill bottom to top with the rpm, because a rev counter must move and the house rules ask for a fill that reads at a glance. How the real crescents move while driving is unverified (gap 1).
8. **Speed above, rpm below, both smaller than the gear.** The real display sets speed (top, about 12% of the screen height) and the rpm digits (in the floor, larger than the speed) as its biggest figures. Here the speed is rank 2 at cap 60, and the rpm is rank 4 at cap 44 under it.
9. **What the telemetry cannot fill is left out.** The real bottom row holds fuel, the clock, telltales, the outside temperature and the 48 V or high-voltage battery, and the real top row holds assistance icons. Forza sends none of these, so none are drawn. The engine-data widget shows boost (from the packet) where the real one shows oil temperature and pressure in kPa.

## What this dash recreates

**The driver display of the E-Class generation that carries the MBUX Superscreen (W214, 2023 on, and the Mercedes-AMG E 53 HYBRID), in its "Sporty" style.**

- The W214 interior has a 12.3-inch driver display that is "visually detached" from the Superscreen: a 14.4-inch central screen and an optional 12.3-inch passenger screen under one glass ([MBUSA, "A bridge between tradition and digitalization: the new E-Class"](https://media.mbusa.com/releases/release-1abd31f58af6a0ae7f6711444300f97d-a-bridge-between-tradition-and-digitalization-the-new-e-class); [MBUSA, "The new E-Class: dimensional concept and interior design"](https://media.mbusa.com/releases/the-new-e-class-dimensional-concept-and-interior-design); [paultan.org, W214 launch](https://paultan.org/2023/04/26/2024-w214-mercedes-benz-e-class-mild-hybrids-phevs-up-to-115-km-ev-range-mbux-superscreen); [ComputerBase, MBUX in the W214](https://www.computerbase.de/2023-04/mercedes-benz-e-klasse-w214-mbux-3/)).
- The driver display has "two display styles (Classic and Sporty) and three modes (Navigation, Assistance, Service)". "The 'Sporty' style is dominated by the color red, with a dynamically designed central tachometer." An optional 3D driver display "allows spatial scene perception with a real depth effect" ([MBUSA, E-Class release](https://media.mbusa.com/releases/release-1abd31f58af6a0ae7f6711444300f97d-a-bridge-between-tradition-and-digitalization-the-new-e-class)). The C-Class (W206) release uses the same words: "dominated by the colour red, with the dynamic design of the central rev counter" ([Brand Spur, C-Class release](https://brandspurng.com/2021/02/24/the-new-mercedes-benz-c-class-saloon-and-estate-this-is-how-inspiring-the-comfort-zone-can-be/)).
- The AMG E 53 HYBRID adds "AMG-specific and hybrid-specific displays" in the driver and centre screens ([MBUSA, AMG E 53 HYBRID](https://media.mbusa.com/releases/performance-and-efficiency-in-a-new-combination-the-mercedes-amg-e-53-hybrid); French press kit: [cp-amg-e53-hybrid-fr.pdf](https://media.mercedes-benz.fr/download/768cdcf4-eeee-45b1-9ab0-babc55fbe817/cp-amg-e53-hybrid-fr.pdf)). The CLE 53 supplement lists the AMG menus Classic, Understated, Service, Supersport and TRACK PACE ([MBUSA, MY24 AMG CLE Coupe owner's manual, p. 26](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf)). Sporty is the base-car style that every W214, AMG included, carries.

The 3D driver display is a separate option; owners discuss it in [mbworld, "3D instrument cluster"](https://mbworld.org/forums/e-class-w214/905475-3d-instrument-cluster.html) and [mbworld, "IP choices"](https://mbworld.org/forums/e-class-w214/923372-ip-choices.html) (owner forum). Reviews with photos of the screens: [CarExpert, AMG E 53 HYBRID](https://www.carexpert.com.au/car-reviews/2025-mercedes-amg-e53-hybrid-review); [AutoGuide, the MBUX driver displays of the GLC](https://cdn-fastly.autoguide.com/auto/manufacturers/mercedes-benz/2023-mercedes-benz-glc-300-exploring-the-mbux-driver-displays-44607835), which carries the same MBUX step; [jesmb.de](https://jesmb.de/15413/).

## What makes it recognisable

Studied at full size from a press-grade photo of a W214's driver display in the Sporty style, parked, with the US speed unit ([motaauto, 2024 E-Class](https://www.motaauto.com/wp-content/uploads/2024/09/2024-Mercedes-Benz-E-Class-2.jpg)); the local copy for review is `mockups/references/AmgMbuxSuperscreen/w214_sport_full.jpg` (git-ignored, never committed). Positions are given as shares of the screen.

### Layout
1. **Three zones.** A G-Force widget at the left, an Engine data widget at the right, and the "tunnel" rev counter in the middle, taking about 55% of the width.
2. **Two red crescents, "( )".** They are thick at 9 and 3 o'clock and taper to points, standing about 85 to 90% of the screen's height. They are saturated red in front, fading to a wine red in depth. A dotted halftone runs on their inner side.
3. **The tunnel.** Between the crescents, concentric rings shrink towards a dim grey-white disc a little above the centre, as if looking down a lit tube. The inner rings carry the same dotted halftone.
4. **The floor.** A dark trapezoid opens from the disc to the bottom edge, its edges drawn as thin lines in perspective. It holds the rpm figure ("0", large, slanted light digits) with "rpm" under it.
5. **Side scales outside the crescents.** Torque on the left, with numerals 600, 400, 200, 0 and −500 and "Nm" at the bottom. Power on the right, with numerals 240, 160, 80, 0 and −100 and "kW" at the bottom. Each is a thin grey arc of about 100° with fine ticks and its numerals on the inner side, between the arc and the crescent. Zero sits low, about two thirds of the way down, because the scales run negative for recuperation.
6. **Speed at the top centre.** It is thin, about 12% of the screen height, with "km/h" under it and a second readout "0 mph" in a small box. A thin horizontal line runs either side of it just below the digits, across the tunnel.
7. **Bottom row.** Fuel bar, clock, telltales, READY, P, outside temperature, battery bar, EL mode and the selector stack "R N D P" with the engaged position large.
8. **Widgets.** G-Force: two red rings with a crosshair and a red dot on a round dark lens, its title above. Engine data: a round gauge with a small scale (20, 140, 150), a pressure in kPa in the middle and an oil-can symbol, its title above.

### Colour, light and type
9. **One saturated red on true black.** Measured from the photo: about #E0102A to #FF2030 in front and #5A0010 in depth. Type is white for live values and grey for labels and scales. Nothing else is coloured apart from the green READY and the blue P and EL telltales.
10. **Light type.** The speed and rpm are a light, slightly slanted grotesque, and the labels are a regular weight of the same family ("G-Force", "Engine data", "Nm", "kW", "rpm"). No source names the typeface.

### Motion
11. **Not documented.** No source read describes how the crescents, rings or scales move while driving. A start-up animation exists in owner videos but was not opened (gap 1).

## How the dash matches each feature

| # | Feature of the real display | In the dash | Match |
|---|---|---|---|
| 1 | Three zones: G-Force, tunnel, Engine data | G-Force widget at x −566, tunnel at the centre (crescents at r 278), Engine data at x 566 | matched |
| 2 | Two tapered red crescents, about 85–90% of the height | Crescents of r 278 round (0, −36), 40 px at 9 and 3 o'clock with a sine taper to points over ±52°, about 80% of the glass height; unlit body in the primary at 20% | matched; they fill with rpm (change 7) |
| 3 | Rings receding to a grey disc above centre | Six rings from r 245 at (0, −36) to r 97 at (0, −62), dimming with depth; a grey-white radial glow in the disc | matched (Blender plate with the halftone next; procedural fallback today) |
| 4 | Dark floor wedge holding the rpm | A dark trapezoid from y 92 to 196 with red edges, the rpm at cap 44, slanted, with "rpm" under it | matched; the rpm is smaller (change 8) |
| 5 | Torque left and power right, thin arcs, numerals inside, zero low | Arcs of r 360 from 138° to 218° (80°), 21 ticks, numerals inside at r 314, a white live fill outside the line; units follow the user's power unit (Nm with kW, lb-ft with hp) | close: zero is at the bottom, because the telemetry has no recuperation, so the scale has no negative part; the span is 80° to keep the arcs on the lower glass |
| 6 | Thin speed at the top with lines either side | Speed at (0, −230), cap 60, light and slanted, its unit under it; the lines either side are replaced by the shift strip along the top edge | changed (change 1) |
| 7 | Bottom row and selector stack | The car's logo at the bottom left; a box at the bottom right with `M` and the gear | changed (changes 3, 6, 9) |
| 8 | G-Force rings and dot; Engine data gauge | G-Force: rings at 0.75 and 1.5 g, crosshair, the felt-force dot on the gBall spring. Engine data: a 270° boost gauge from −1 to 2 bar, the value in psi (mph) or bar (km/h) | matched; boost replaces oil (change 9) |
| 9 | One red on black, white and grey type | The `amgMbux` palette, static parts at the tokens' 20% luminance cap, live values in ink | matched (change 4) |
| 10 | Light slanted grotesque | Barlow Semi Condensed 300 with a 0.10 synthetic slant for speed and rpm, 400 for the gear, 500 for labels | close (change 5) |

## Shared AMG MBUX notes

For the four later AMG threads (`AmgMbuxSupersport`, `AmgMbuxClassic`, `AmgMbuxTrackPace`, `AmgOneF1`), to read before they search. What is here is sourced above or in the source named.

**Menu structure, by generation.**
- First-generation MBUX (A 35 / A 45, CLA 35 / 45, 2018 to 2019) has AMG styles Classic, Sport and Supersport, plus an AMG Performance menu: Warm-up, SETUP, G-meter, Engine data, and RACETIMER on cars without TRACK PACE ([A 45 / CLA 45 release](https://www.automotiveworld.com/news/the-new-mercedes-amg-a-45-4matic-and-cla-45-4matic-the-super-sportscars-in-the-compact-class-2/)). Sport is "a high-tech turbine look with decidedly sporty black/yellow contrasts" ([CLA Shooting Brake release](https://www.automotiveworld.com/news-releases/the-new-mercedes-benz-cla-shooting-brake-extravagant-and-practical-sports-car-with-load-space/)).
- Second-generation MBUX (2020 S-Class on; C 43 / C 63, SL R232, AMG GT C192, S 63) has the menu bar Understated (press: Discreet), Sport (press: Sporty), Classic, Navigation, Assistance and Service, plus the AMG-only Supersport and Track Pace ([S-Class "My MBUX"](https://media.mercedes-benz.ca/releases/release-97ed16fa779e41ce1c8368df0d064d7f-meet-the-s-class-digital-my-mbux-mercedes-benz-user-experience); [MBUSA, AMG C Sedan MY24 owner's manual](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20C%20Sedan%20Owners%20Manual.pdf)).
- The W214 step (E 53, CLE 53) has the base styles Classic and Sporty. The AMG supplement adds Supersport and TRACK PACE (sources under "What this dash recreates").
- The manual legends:
  - Supersport is current torque, centre content, current power.
  - TRACK PACE is G-meter, centre content, telemetry.
  - Classic is speedometer, tachometer, gear display, battery or coolant, outside temperature, centre content, time and fuel.

**Colour roles.** Sporty is "dominated by the color red". First-generation Sport is black and yellow. Manual mode shows a yellow `M`. Cold temperatures are blue, G-meter maxima red, and TRACK PACE deltas green or red (same sources). In this repo:
- Each AMG dash uses the `amgMbux` family palette as its default: primary red, secondary cool grey, accent and shift white.
- A dash whose real display is red (Sporty, Supersport) keeps red structure, past the 5% budget, on TJX's 2026-11-28 choice.
- A dash whose real display is not red (TRACK PACE, Classic) uses red only where the real one does.
- The `M` tag is drawn in the palette's `primary_text`, not a literal yellow, so it follows the palette.
- Shift is white, so the strip reads apart from the red.

**Type.**
- No source names the cluster's typeface; Mercedes's corporate type is not freely licensed.
- This dash uses Barlow Semi Condensed (OFL) in weights 300, 400 and 500, with a 0.10 synthetic slant on the speed and rpm. Labels are set in mixed case with 6–8% tracking, as the real "G-Force" and "Engine data" are. Here they are capitals for legibility at 4K and at 1.5 m.
- Later AMG dashes should reuse the same family, so that the five read as one maker's screens.
- The user's display font replaces the numerals.

**Motion timing.** None is documented by Mercedes. The house timing applies:
- the ribbon spring for fills, with a 60 rpm backlash on the limiter;
- `HeroGear` for gear changes;
- the gBall spring for the G dot;
- the limiter strobe at 10 Hz from a lit start, keeping a fifth of its light in the dark phase;
- a 2 Hz breath in reduced flash.

This dash's upshift moment is a ring running from the disc out to the crescents in 320 ms. A sibling should invent its own moment and not reuse it.

**Geometry.** The W214 driver display is 12.3 inches at about 2.67:1. On the 4K cockpit anchor this dash's glass is 1412 × 522 px in a 14 px bezel (x ±720, y −314 .. 236), inside the cockpit envelope. A sibling that recreates a 12.3-inch screen can take the same `housing` and `screen` rects from `geom.rs`.

## Round v5: weather and the road logo (research, 28 Nov)

TJS, 09:07: "The animated clouds should match the ingame weather so also rain would be cool as fuck or that I can turn on" and "And have the logo on the road".

**What the game sends.** The official FH6 Data Out page (support.forza.net, article 51744149102611, read 09:12) lists no weather, rain or wetness field. The only puddle channel is `WheelInPuddle{FrontLeft,FrontRight,RearLeft,RearRight}` at bytes 132, 136, 140 and 144. It is an S32 that the page defines as 1 in a puddle and 0 out of one, so it is a flag, not a depth (the Motorsport name `WheelInPuddleDepth` does not apply). `SurfaceRumble*` (148 to 160) is a force-feedback value with no stated range. fh6-core does not decode bytes 132 to 147 today. All 20 golden packets carry 0 there, so no recorded rain capture exists yet. The page's own field list adds up to 323 bytes against its stated 324, so check the offsets on a real capture before trusting them.

**What follows for Auto.** A puddle flag fires only while a tyre crosses standing water, so it cannot say "it is raining" frame by frame. Auto therefore keeps a wetness level from 0 to 1:
- each frame with any wheel flagged adds `dt / 2 s` (two seconds of puddle contact makes it fully wet);
- it decays with a 90 s time constant, so rain stays on between puddles and fades about three minutes after the last one;
- rain shows above 0.35 and clears below 0.15 (hysteresis, so it never flickers);
- the rain layer fades in and out over 4 s.

This needs the real-game check (session C): drive in FH6 rain and in clear weather, log the four flags, and confirm that rain on tarmac gives puddle hits often enough and that dry roads give none (fords, splashes). If dry roads trip it, raise the threshold. If rain gives too few hits, Auto stays off and the manual Rain switch carries the feature. The **setting** is `Weather: Auto | Clear | Rain`, per dash, with Clear as the default until the check passes.

**Rain, as drawn on board v5.**
- Streaks radiate from the vanishing point, and their length and speed scale with `speed_ms`.
- Beads sit on the near ribs below the horizon.
- The sky darkens by about a third, and the road takes a wet sheen. The clouds always stay. TJS, 09:16: "Just don't lose the clouds they look sick and will animate nice". Rain dims and greys them but never replaces them, and they keep drifting with speed in every weather.
- The streaks are thin (1 to 2 px) and grey-white at low alpha, and they change position, not brightness. That keeps them out of the flash audit's scope: they are motion, not flashing.

**Road logo.** The car's logo (from `fh6-kit::logo::draw`, never an AMG or Mercedes mark) is painted flat on the road near y 150 under a vertical squash of about 0.34. It is a faint satin decal that turns glossy when wet. It is static, so `static` must show it under the 20% cap. It replaces the bottom-left logo box.

### Rain from real physics (research, 28 Nov 10:40)

Study chart: `/mnt/project-files/amg_mbux_superscreen/research/rain_physics_study.png` (a chart, not a design).

**Fall speed.** The Foote and du Toit fit to Gunn and Kinzer (1949), as tabled in Bringi, Thurai and Baumgardner (AMT preprint amt-2017-401, Table 1), gives these speeds at sea level: 0.7 mm about 2.6 m/s, 1 mm about 3.9 to 4.0 m/s, 1.5 mm about 5.4 m/s, 2 mm about 6.5 m/s. Big drops top out near 9 m/s (the textbook Gunn and Kinzer ceiling, not read in this round). The dash uses 6.5 m/s for steady rain. A drizzle variant at 4 m/s is noted for later.

**Where the streaks come from.** In the car's frame a drop moves at (−v_x, −v_t, −v_z). So the rain's focus of expansion (the point the streaks fan out from) sits at elevation atan(v_t / v) above the horizon, and sideways at atan(v_x / v_z), which is the car's slip angle. With the tunnel's virtual focal length f = 312 px (the 180 px sky window spans about 30°), the focus sits at these heights:

| speed | source elevation (2 mm drops) | px above the horizon |
|---|---|---|
| parked | 90° | streaks fall straight down |
| 10 mph | 55° | 454 (off the top: near-vertical streaks) |
| 20 mph | 36° | 227 (at the top of the sky window) |
| 40 mph | 20° | 113 |
| 60 mph | 14° | 76 |
| 90 mph | 9° | 50 |
| 130 mph | 6° | 35 |
| 180 mph | 5° | 25 |

This is the dash's own data-driven idea for the rain. When you pull away the rain falls straight down, and as you speed up its source sinks towards the horizon and the streaks fan out into a warp. In a drift the source slides sideways by f·tan(slip) (83 px at 15°, 180 px at 30°), so the rain shows where the car is actually going, not where it points. Streak length follows the drop's speed across the screen over one frame of persistence, so streaks are short at the source and long at the tunnel's edge. Inputs: `speed_ms`, `velocity_x`, `velocity_z` (or `car_slip_angle_rad()`, already in fh6-core).

**Wet road: darker, and a mirror near the horizon.**
- Darker: Ångström's measurements, quoted by Lekner and Dorf ("Why some things are darker when wet", Applied Optics 27(7), 1988), put dry sand at 0.182 and wet sand at 0.091 albedo, and black mould at 0.141 dry and 0.084 wet. So the wet road plate is about 0.5 to 0.6 times as bright as the dry one.
- Mirror: a water film reflects by Fresnel (n 1.333, unpolarised). With the same f = 312 px, the reflectance at a given depth below the horizon is:

| below the horizon | incidence | reflectance |
|---|---|---|
| 5 px | 89.1° | 0.90 |
| 20 px | 86.3° | 0.67 |
| 40 px | 82.7° | 0.46 |
| 80 px | 75.6° | 0.23 |
| 120 px | 69.0° | 0.12 |
| 208 px (the road's foot) | 56.3° | 0.05 |

So the wet road shows the clouds mirrored in the horizon, strong just under it and fading to almost nothing at your feet. That is the same sky plate drawn flipped and scrolling with it, through a Fresnel ramp. The road logo picks up the same reflection, so it turns glossy. The clouds stay visible twice in rain (TJS: "Just don't lose the clouds").

**FH6's real weather (what the sources say).** Pure Xbox and The Drive (both 17 Mar 2026, quoting Playground) confirm four seasons. Winter asphalt is "generally" clear of snow, off-road routes are snowy, and the Alpine Region has snow all year. None of the sources read documents rain, puddles, storms, wet grip or mid-session weather changes. One seller blog claims "Tokyo downpours" without a source. So rain in FH6 is likely but unconfirmed. The real-game check must confirm it, and also check whether snow sets the puddle flag.

### Telemetry mapping (all zones, v5 plan)

| zone | input | rule |
|---|---|---|
| portal rings (rev counter) | `rpm / max_rpm`, shift fraction | rings light far to near; red rings from the shift point; horizon strobe on the limiter |
| sky drift | `speed_ms` | scroll by metres × 0.5, wrapping |
| horizon lean | `g_lat` | 5° per g, capped at 8° |
| sky warmth | shift fraction | warm veil from 72% |
| brake wash | `brake` | red wash on the road, red-tinted horizon |
| lane dashes | `speed_ms` | one dash per 10 m |
| crescents | rpm heat | intensity from 55% to 95% |
| torque and power bands | `power`, torque from power / ω | held values (no flicker) |
| G-Force widget | `g_lat`, `g_long` | dot |
| Engine data | `boost` | psi arc |
| **rain source** | `speed_ms`, `velocity_x/z` | elevation atan(6.5 / v), side f·tan(slip) |
| **rain density** | wetness (puddle accumulator) or the setting | 0 to 1 over 4 s |
| **wet road** | wetness | plate × 0.55, Fresnel sky mirror |
| **road logo** | car (logo crate), wetness | satin decal, glossy when wet |
| **housing** | `housing::shown` | optional slab; off by default |

### Edge cases (v5)

1. **Parked in rain** (speed under 0.5 m/s): the streaks become vertical falls with no fan, and the beads still sit on the ribs. The source formula needs a guard for v = 0, using the vertical limit and no division.
2. **Reverse:** `velocity_z` goes negative, so the source would flip behind the camera. Clamp to forward travel (use |v_z| and treat reverse as slow forward), and never draw streaks converging.
3. **Airborne:** puddle flags read 0 in the air. The decay is slow (90 s), so a jump never clears the rain.
4. **Fords and splashes on a dry day:** a long ford can push the wetness over the threshold. Two seconds of contact to saturate is short, so test it in the real game. If it trips, need 3 or more separate puddle hits within 60 s instead.
5. **Packets stop** (menu, pause, rewind, replay; the FH6 page says data flows only while driving): hold the wetness frozen and never decay it while no packets arrive. This needs a timestamp check, `timestamp_ms` not advancing.
6. **Car change or a new session:** keep the wetness (the weather didn't change).
7. **Setting changed mid-drive:** Clear forces 0 and Rain forces 1, each fading over 4 s. Auto resumes from the current accumulator.
8. **Reduced flash:** rain has no flashing. Streaks move but never blink, and drop beads fade in over 0.5 s or more, never popping.
9. **OLED:** streaks are thin, grey-white and always moving, so there's no static burn-in. The wet road is darker, which helps. The road logo is static and must stay under the 20% luminance cap.
10. **Budget:** streaks go in one batched draw (about 70 quads), the mirror reuses the sky image (no new texture), and nothing allocates per frame (a fixed array for streak state, refreshed by an index hash).
11. **Snow (winter, Alpine):** out of scope unless TJS asks. If snow sets the puddle flag, Auto would show rain in snow. Note that for the real-game check.
12. **Drizzle versus downpour:** with a flag-only input the intensity can't be measured. Density follows the wetness level only.

**Rejected idea, 09:16.** TJS asked for the crescent mesh to change colour as the rev limiter, then withdrew it: "Actually no the portal is fine for that". The portal rings stay the rev counter and limiter.

**Housing.** The OSD thread is adding `housing::shown(config, ClusterTheme::AmgMbuxSuperscreen)`, with MBUX dashes off by default. This dash draws an optional slab only when that returns true.

## Gaps (not guessed)

1. **Motion of the Sporty display while driving.** Whether the crescents fill, pulse or stay static is not described by any source read, and no driving video was opened. The dash's fill is an interpretation (change 7).
2. **The exact red.** It is measured from one photo with unknown white balance; the token is a reasoned value in that range.
3. **The typeface.** It is not documented anywhere opened.
4. **The AMG-specific Sporty content.** The E 53's "AMG-specific displays" are not itemised in the sources read; the dash shows the base Sporty layout.
5. **Halftone dot pitch and ring count.** These come from one photo at 1000 px wide; the Blender plate follows them approximately.
