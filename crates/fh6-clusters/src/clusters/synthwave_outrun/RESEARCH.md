# SynthwaveOutrun: research note

Build #39, catalog row 20, category Retro, Target `concept`, family Sci-Fi, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on 2026-11-28 from the source linked in section 10. **Computed** means worked out here with Python 3 from cited or brief values, inputs and numbers shown. **Our reading** is a conclusion or proposal for this dash, open to the direction check. "Unconfirmed" marks a lead step 1 must still check. Pages were read through the PC's built-in browser (Wikipedia, Wikimedia Commons, Jalopnik, Hagerty UK, Toyota's newsroom, PixelatedArcade, the CRT Database, a GitHub gist), three web fetches (Margolin's XY monitor article, the Jankowiak phosphor tables, a halo patent) and the open-source Cannonball engine on GitHub, which reimplements Out Run's original 68000 code routine by routine. Segaretro sits behind a bot check and was not read.

**Staying distinct in the Retro shelf.** `RetroCyber` recreates the Toyota Supra A70 vacuum-fluorescent meter: bare segments on black, a hex control-grid mesh, filament wires, a smoked filter lens. `GlyphDotMatrix` is the LED dot matrix: round dots in dark wells under a red contrast filter. SynthwaveOutrun takes the third display technology of the decade, the **colour raster CRT**: horizontal scanlines whose width grows with brightness, a slot-mask phosphor screen, a thick curved and tinted faceplate with its halo, and a picture that is a scene (horizon, sun, grid) rather than segments or dots. No VFD mesh, no filament lines, no round wells.

## 1. The real display or benchmark

### 1.1 The one benchmark: Sega's Out Run (1986) on its colour raster arcade monitor

The review measures this dash against **the Out Run arcade machine's screen: a colour raster CRT showing the game's low-camera road to a horizon, with its rev bar and speed readout**. The brief's name and its "1980s arcade racing cabinets' horizon framing" point there, and the synthwave subgenre takes its name from the game.

| # | Feature | Label and source |
|---|---|---|
| B1 | Sega, September 1986, designed by Yu Suzuki; a pseudo-3D racer seen from behind a Ferrari Testarossa convertible with "the camera placed near the ground", so road crests hide what lies beyond. | **Cited**, Wikipedia, Out Run |
| B2 | Out Run "provid[ed] the name for a subgenre of synthwave". | **Cited**, Wikipedia, Out Run |
| B3 | Every cabinet has a steering wheel, a stick shift and pedals; four cabinet designs, two upright and two sit-down motion cabinets. | **Cited**, Wikipedia, Out Run |
| B4 | Arcade monitor: horizontal, raster, colour, 320 x 224 at 60.054389 Hz. | **Cited**, PixelatedArcade, OutRun technical specs |
| B5 | The rev counter is 20 steps drawn in 10 text tiles: each tile is off, half lit or fully lit, so the bar advances in half-tile steps. Steps 0 to 9 use the palette the source calls WHITE, 10 to 13 GREEN, 14 to 19 RED; unlit tiles show a blank tile. During the start countdown the revs are boosted by a quarter "so the bar goes further into the red". After a crash the bar shows a separate post-stop value. | **Cited**, Cannonball `ohud.cpp`, `draw_rev_counter` (original routine at 0x6B08) |
| B6 | Colour by tile: 5 white tiles, 2 green, 3 red (the last write to each tile is the odd step). | **Computed** from B5 |
| B7 | Speed is three digit tiles drawn two tile rows tall, with leading zeros blanked. | **Cited**, Cannonball `ohud.cpp`, `blit_speed` (0xBB72) |
| B8 | The gearbox is two-speed (LOW and HIGH): a gear change runs 4 logic ticks, the revs relax by 1/16 per tick, and a hard change can raise wheel smoke. | **Cited**, Cannonball `oferrari.cpp`, `tick_gear_change` |
| B9 | The HUD sits in a fixed text layer above the road; the rev counter is on the bottom text row (address 0x110DB4 lies in row 27 of 28). | **Computed** from B5 (0xDB4 = 3508 bytes, 128 bytes a row); the exact screen position is unconfirmed, step 1 checks a screenshot |

**What makes it recognisable (Our reading, step 1 confirms against a screen photo):** a horizon low in the frame, a road and scenery that scale toward it, a stepped white-green-red rev bar and three chunky speed digits in a corner, and above all the raster CRT itself: visible scanlines, colour from a phosphor mask, bright areas that bloom and soft edges.

### 1.2 The period hardware it is faithful to

The concept is a colour CRT used as a car's meter. That existed:

| # | Real hardware | Label and source |
|---|---|---|
| H1 | **Toyota Electro Multivision** (announced 7 January 1985, Soarer 3.0 GT-Limited automatic): "a full-color CRT display (Type 6 cathode ray tube) that functions as a visual meter", "the first time in the world" in a mass-production car. It shows engine revolutions, shift position, TEMS and ECT patterns, a fuel-efficiency history (average for every 30 s of the past 5 minutes), a maintenance guide, warnings in the upper right corner, and TV or video when parked. | **Cited**, Toyota global newsroom, 1985 release |
| H2 | **Aston Martin Lagonda Series 3**: three dedicated CRTs in the instrument binnacle, driven by a Z80 computer with a custom character set and animated graphical gauges; very unreliable and soon dropped. Earlier Lagondas used gas plasma and solid-state displays, later VFD. | **Cited**, Jalopnik (2014); Hagerty UK |
| H3 | **Buick Riviera Graphic Control Center** (1986): a CRT "with a green and black screen", the first production touchscreen. | **Cited**, Hagerty UK |
| H4 | Other digital dashes of the decade for mood, not technology: Corvette C4 (three LCDs as standard), Nissan 300ZX Z31 (speed as digits and a colourful bar chart), Renault 11 TXE Electronic (a horizontal rev counter), Subaru XT Turbo (a car pictogram flanked by "pavements" lit by boost and tachometer, "like a cross between Tron" and a toy racer). | **Cited**, Hagerty UK |

**Our reading.** The XT Turbo's road pictogram and the Out Run screen are the same idea the brief draws (a road running to a horizon, lit by the engine), and Toyota's Electro Multivision proves a colour CRT could be the meter. The dash is therefore **an Electro Multivision-class colour CRT in the binnacle, running a night-drive horizon in Out Run's grammar**. Better real target: none found. Toyota's own image downloads (section 2) are the closest automotive photo; the Lagonda Series 3 screens have too few readable photos to be the benchmark.

### 1.3 The CRT physics the dash must show

| # | Property | Label and source |
|---|---|---|
| C1 | Colour CRTs use three phosphors behind a shadow mask that lets each beam reach only its own colour. A delta gun lights dot triads; an in-line gun lights stripes through slots. Vertical slots tolerate the Earth's field better. | **Cited**, Margolin, "The Secret Life of XY Monitors" |
| C2 | TV-class tubes of the kind used in arcade monitors are slot-mask with dark-tinted glass, 90 degree deflection (example: a 19 inch Toshiba tube with "Mask: Slot, Tint: Dark", listed as an arcade monitor tube donor). A Commons photo compares a TV's slot array with a PC's dot array at one magnification. | **Cited**, CRT Database; Wikimedia Commons, "CRT pixel array.jpg" (the example tube is from 1998, so the period mask of Out Run's monitor stays unconfirmed) |
| C3 | P22 colour phosphors: red Y2O2S:Eu at 611 nm, green (Zn,Cd)S:Cu,Al at 530 nm, blue ZnS:Ag at about 450 nm, all "short" persistence; ZnS:Ag blue afterglow about 200 ns. | **Cited**, Wikipedia, Phosphor (standard types table) |
| C4 | Monochrome options of the period: P1 green willemite, 525 nm, 1 to 100 ms (Jankowiak gives 20 ms); P31 yellowish green ZnS:Cu, 0.01 to 1 ms (oscilloscopes, monochrome monitors); P3 amber, 602 nm, 13 ms. | **Cited**, Wikipedia, Phosphor; Jankowiak, CRT phosphors |
| C5 | Persistence classes: medium-short is 10 us to 1 ms, medium 1 to 100 ms (to 10%). P22 and P31 are medium-short. | **Cited**, Margolin |
| C6 | A 60 Hz field lasts 16.7 ms (8.3 ms at 120 Hz), so P22 and P31 fade completely inside one rendered frame: a colour CRT leaves no visible trail at frame rate; only P1 (20 ms) leaves about one frame of afterglow. | **Computed** from C3 to C5 |
| C7 | The beam spot grows with intensity: a good model is a separable raised-cosine spot whose width rises with the square root of brightness between a minimum and a maximum, scanned along the line; bright scanlines fatten and close the dark gap, dim ones thin out. Real spots are very bright at the beam and fall off fast; CRTs are "fast rise, slow fall". | **Cited**, Duffey, "Scanline simulation math" (gist) and the Blur Busters comment on it |
| C8 | Halo (halation): light trapped in the faceplate by total internal reflection lands back on the phosphor and draws "a nearly uniformly illuminated area with a clearly defined outer edge" round a bright spot; its diameter is "typically 3.5 times the thickness of the glass" (critical angle about 41 degrees at n = 1.52). | **Cited**, US 4,310,784 |
| C9 | Check: asin(1 / 1.52) = 41.14 degrees; a ray at that angle returns at 2t tan(41.14) = 1.747 t from the spot, so the halo is 3.49 t across. | **Computed** |
| C10 | Curved faceplates (needed against the vacuum) and wide deflection cause pincushion distortion, corrected by yoke magnets on monochrome tubes and by dynamic correction on colour tubes. A thin aluminium layer over the phosphor reflects light forward. Stationary images burn the phosphor; the Amplifone spot killer blanks a stopped beam. | **Cited**, Margolin; Wikipedia, Phosphor |
| C11 | Colour CRT beam spots are about 0.2 mm, monochrome about 0.6 mm. | **Cited**, Wikipedia, Phosphor |
| C12 | Vector monitors draw lines, not raster, and are brighter where the beam moves slowly. Battlezone (1980) shows a flat plane with a mountainous horizon, an erupting volcano and a crescent moon on a black and white vector monitor whose overlay tints the bottom four fifths green and the top fifth red; Atari's colour version is the shadow-mask "Color QuadraScan" (Tempest, Star Wars). | **Cited**, Wikipedia, Vector monitor and Battlezone |
| C13 | The arcade triad pitch of a 1986 19 inch monitor, the faceplate thickness and the tint transmission. | Unconfirmed: no source read gives them; step 1 looks for a Nanao, Hantarex or Wells-Gardner K-series data sheet |

**Our reading of C12.** The rank-5 wireframe ridge is Battlezone's horizon, but a vector tube cannot draw the filled sun, the chrome speed or the gear pad, so the dash keeps one technology, raster, and draws the ridge as a raster line, as Out Run's horizon is.

### 1.4 What the user can change, and the period default

TJS: "Yes and you can change for example needle color etc". The dash has no needle; its parts change like a real CRT's set-up.

| Part | Maps to | Period-correct default |
|---|---|---|
| Phosphor set | `config.extra` `SynthwaveOutrunPhosphor`: `p22`, `p31`, `p1`, `p3` | `p22` colour (Electro Multivision, C3); `p31` is the Riviera's green and black (H3), `p3` amber |
| Sun slat colours | palette `accent` (base) to `secondary` (crown), mapped into the P22 gamut | the brief's ramp; magenta to orange as on the synthwave posters (see open question 2) |
| Grid and ridge colour | palette `primary` at 20% luminance or less | P22 green-cyan |
| Gear glyph | palette `primary_text` on its pad; font role (the user's font) | ink white, Saira Condensed 800 (ExtraBold is in `assets/fonts/saira-condensed`) |
| Speed chrome | `config.extra` `SynthwaveOutrunChrome`: `chrome`, `gold`, `flat` | `chrome` (the brief's band LUT) |
| Shift strip | palette `shift` | the brief's crown strip colour |
| Mask | `config.extra` `SynthwaveOutrunMask`: `slot`, `aperture`, `delta`, `none` | `slot` (C1, C2) |
| Scanline strength | `config.extra` `SynthwaveOutrunScanlines`: 0 to 1 | 0.6 (gap visible on dim content, closed on bright, C7) |
| Glass tint and curvature reflection | `config.extra` `SynthwaveOutrunGlass`: `dark`, `clear`, `none`; palette `glass_tint` | `dark` (C2) |
| Bezel | `config.extra` `SynthwaveOutrunBezel`: `dash` (moulded black plastic hood) or `cabinet` (flat black arcade bezel) | `dash` |
| Brightness and night level | host brightness and `reduced_flash` | full; reduced flash per section 7 |

Random and custom palettes stay allowed. Our reading: map every palette colour into the P22 primaries' triangle (desaturate along the line to white) so a random palette still looks like it came out of a period tube.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Out Run upright cabinet, monitor and bezel in the cabinet | https://upload.wikimedia.org/wikipedia/commons/1/1e/Outrun.jpg | https://commons.wikimedia.org/wiki/File:Outrun.jpg |
| 2 | Out Run sit-down deluxe cabinet | https://upload.wikimedia.org/wikipedia/commons/c/cf/Outrun-sit-down-cabinet.jpg | https://commons.wikimedia.org/wiki/Category:Out_Run_(arcade_racing_game) |
| 3 | Out Run image (555 x 496; step 1 checks whether it is the screen) | https://upload.wikimedia.org/wikipedia/commons/d/d6/Out_Run.jpg | same category |
| 4 | Toyota Electro Multivision, Soarer 3000 GT-Limited | https://global.toyota/en/download/8200182 | https://global.toyota/en/detail/7790394 |
| 5 | The Electro Multivision system sheet | https://global.toyota/en/download/8200186 | same release |
| 6 | Slot array (TV) against dot array (PC) at one magnification | https://upload.wikimedia.org/wikipedia/commons/0/04/CRT_pixel_array.jpg | https://commons.wikimedia.org/wiki/File:CRT_pixel_array.jpg |
| 7 | Shadow-mask close-up with a cursor | https://upload.wikimedia.org/wikipedia/commons/8/8d/Shadow_mask_closeup_cursor.jpg | https://commons.wikimedia.org/wiki/File:Shadow_mask_closeup_cursor.jpg |
| 8 | CRT phosphor close-up | https://upload.wikimedia.org/wikipedia/commons/9/9f/CRT_screen._closeup.jpg | https://commons.wikimedia.org/wiki/File:CRT_screen._closeup.jpg |
| 9 | Spectra of P22 red, green and blue (figure on the page) | https://upload.wikimedia.org/wikipedia/commons/2/29/CRT_phosphors.png | https://en.wikipedia.org/wiki/Phosphor |
| 10 | Battlezone cabinet, small | https://upload.wikimedia.org/wikipedia/commons/b/bb/Atari-bz-arcade.jpg | https://commons.wikimedia.org/wiki/Category:Battlezone |
| 11 | Battlezone at the Musee Mecanique | https://upload.wikimedia.org/wikipedia/commons/4/48/Mus%C3%A9e_M%C3%A9canique_205.JPG | same category |
| 12 | 1987 Corvette C4 LCD dashboard | https://upload.wikimedia.org/wikipedia/commons/6/63/MMWA_1987_Chevrolet_Corvette-Dashboard.jpg | https://commons.wikimedia.org/wiki/File:MMWA_1987_Chevrolet_Corvette-Dashboard.jpg |
| 13 | Lagonda interior (binnacle shape; check which series) | https://upload.wikimedia.org/wikipedia/commons/a/a1/AstonMartinLagonda-interior.jpg | https://commons.wikimedia.org/wiki/File:AstonMartinLagonda-interior.jpg |
| 14 | Subaru XT Turbo, 300ZX Z31, Renault 11 and Riviera GCC dashes (article photos) | page only | https://www.hagerty.co.uk/articles/12-favourite-retro-digital-dashboards-volume-one/ |

Commons URLs are built from the MD5 of the file name (**Computed**); the PC fetches each and drops any that 404. Drive `references` folder: https://drive.google.com/drive/folders/1Mbz05rTg0sT2wxTOWsjwoID9xZdMas1A (with `SOURCES.md`; no image bytes saved from here).

## 3. How the real thing behaves, with motion notes

- **Start-up.** Out Run: during the start countdown the rev bar is boosted a quarter so it reaches further into the red (B5, **Cited**). A CRT: the heater warms the cathode before any picture (Margolin, **Cited** for the indirectly heated cathode); colour sets degauss at power-on (CRT Database, "Automatic on Power-on", **Cited**). Our reading: boot is (1) a dark tube, (2) the degauss: a brief, small hue wobble over the black glass reflection only, (3) the raster fades in from the scanlines' centres outward (the beam current rising), then the brief's grid in 4 sectors and the sun rising to live rpm. Total 1 s or less; gear and speed live from frame 1.
- **Idle.** One slat lit at 20% luminance or less (brief). The scanlines are thin and the gaps clearly visible because the content is dim (C7). The grid is still when parked (distance does not change).
- **Cruise.** Slats light in the same frame as rpm; the grid scrolls with distance; bright parts (gear, speed bands) show fattened scanlines and a faint halo (C7, C8); dim parts show the scanline structure.
- **Pull to the limiter.** The sun rises slat by slat into a full disc at the shift rpm. As more of the tube lights, the beam spots widen: the sun's slats visibly fill their scanline gaps near the crown. Out Run's bar turns green then red in its last half (B6); the dash's crown strip carries that order.
- **Limiter and shift.** Only the crown strip strobes, 10 Hz at 50% duty (2 Hz breath in gentle mode). The horizon line heats to accent and widens from 1 to 3 px as a brighter beam would (steady). Never flash the sun.
- **Gear change.** Out Run's change takes 4 logic ticks while the revs relax by 1/16 a tick (B8, **Cited**): the sun sinks with the real rpm drop, no extra animation needed; the brief's 1-frame horizon pulse at 4000 px/s takes 0.175 s, about 10.5 frames at 60 Hz, to cross 700 px (**Computed**).
- **Braking.** The grid's near edge ramps toward caution (brief), steady.
- **Drift and lateral G.** Felt lateral G slides the chrome bands' highlight azimuth. The grid bends with the path curvature (section 4).
- **Reverse and neutral.** Reverse scrolls the grid toward the horizon (signed longitudinal velocity). Neutral: gear N; sun follows free revs.
- **Crash.** Out Run shows a post-stop rev value after a crash (B5). The family allows an 80 to 120 ms event glitch; on a CRT that is a convergence jolt (R, G, B shift 1 to 2 px apart and settle), never on the gear.
- **Shut-down.** A colour CRT switched off collapses its raster to a line and a dot (Our reading, common CRT behaviour, unconfirmed by a source here). The host hides a stale stream, so this plays only on `race_on` 1 to 0 and only as a small, dim line (no large flash).
- **Persistence.** No trails at frame rate in P22 or P31 (C6); with `p1` a one-frame afterglow at about 40% is honest.
- **Flicker.** The real tube flickers at 60 Hz; the dash never draws it (flash rules).

## 4. The maths

**Row:** "Perspective of a ground-plane grid: a line at depth z sits f H_e / z under the horizon; on a curve it shifts by kappa z^2 / 2. f = 900 px, eye height 1.2 m: lines at 5, 10, 20 and 40 m sit 216, 108, 54 and 27 px under the horizon."

- **Computed:** 900 x 1.2 / z gives 216.0, 108.0, 54.0 and 27.0 px. **The row checks; no correction.**
- **Computed, the band:** the brief clips the grid to the lower 160 px, so the nearest visible line is at z = 900 x 1.2 / 160 = 6.75 m; the 5 m line (216 px) is outside the band. Tests should assert the visible set starts at 6.75 m.
- **Computed, field of view:** with f = 900 px the cockpit half-width 700 px gives a horizontal field of 75.7 degrees, the hood's 850 px gives 86.7 degrees.
- **Lateral shift on screen:** the ground offset kappa z^2 / 2 projects to x = f kappa z / 2. **Computed:** at v = 30 m/s and yaw rate 0.3 rad/s, kappa = 0.01 per metre and the lines at 10, 20 and 40 m move 45, 90 and 180 px. At v = 3 m/s (the brief's floor) and 0.5 rad/s, kappa = 0.167 and the 40 m line moves 3000 px, far off the window. **Recommended clamp:** |kappa| at most 0.035 per metre keeps the 40 m line within 700 px (1400 / (900 x 40) = 0.0389). This is a note for the catalog row, not a wrong value.
- **Fallback for kappa** from verified fields: lateral acceleration a = v^2 kappa, so kappa = g_lat x 9.81 / max(v, 3)^2, same clamp.
- **Grid scroll:** phase = distance mod line spacing in depth; a parked car's grid is still.
- **Scanlines (C7):** spot width sigma(s) = sigma_max ((1 - sigma_min) sqrt(s) + sigma_min), raised-cosine profile, s in linear light. Tests: two full-white neighbouring lines sum to at most 1; width at s = 0.25 is sigma_max (0.5 (1 - sigma_min) + sigma_min).
- **Halo (C8, C9):** diameter 3.49 t. **Computed:** a 19 inch 4:3 tube is 289.6 mm tall; mapped onto the 520 px cockpit window that is 1.80 px per mm, so a 10 mm faceplate gives a 63 px halo, 12 mm gives 75 px (faceplate thickness unconfirmed, C13).

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Sun slats (14) | `rpm`, `idle_rpm`, `max_rpm`, `shift_rpm` | verified | slat k lights at idle + (shift - idle) k / 14; release 75 rpm lower (hysteresis) |
| Crown strip (16 pods) | `rpm`, `shift_rpm` | verified | fill outside-in; strobe only on the limiter |
| Horizon heat | `rpm` at limiter | verified | steady accent, 1 to 3 px width |
| Gear glyph | `gear` | verified | N, R, 1 to 10+ |
| Chrome speed | `speed`, unit | verified | 3 digits, leading zeros blanked as in Out Run (B7) |
| Chrome highlight azimuth | `g_lat` | verified | felt G, clamped, spring |
| Grid scroll | `race_progress` distance_traveled | decoded, offset confirmed at check C | fallback: integrate signed `velocity` z x dt |
| Grid curvature | `yaw_rate` | decoded, offset confirmed at check C | kappa = yaw / max(v, 3), clamp 0.035; fallback from `g_lat` (section 4) |
| Horizon ridge (120 points) | `position` y | decoded, offset confirmed at check C | sample every 5 m of distance, not every frame; labelled "own elevation"; fallback flat ridge |
| Grid near-edge caution | `brake` | verified | steady ramp |
| Beam drive (bloom width) | luminance of the drawn content | derived | C7 model, no telemetry |
| Boot, shut-down | `race_on` | verified | section 3 |
| Ridge reset | `car_ordinal` | verified | clear the buffer on a new car |

**What the game cannot send** (parts of Out Run, the Electro Multivision and the period CRT dashes), each with replacements; the recommended one is marked.

1. **Out Run's TIME countdown to the checkpoint.** No countdown exists. **Recommended: a lap or race clock from current lap and race time (decoded, offset confirmed at check C), shown only when a lap is running.** Second: a 0-100 km/h timer from speed and timestamp.
2. **SCORE.** The game has no score. **Recommended: an odometer of this drive from distance travelled (decoded, check C).** Never a made-up score.
3. **The course map and route fork.** No map. **Recommended: the car's own breadcrumb from world position X and Z (decoded, check C), drawn as a scanline-thin trail in a corner and labelled as the own trail.** Second: the ridge alone carries "where I have been".
4. **The radio station choice.** No media. **Recommended: a P31-green oscilloscope trace of the engine's firing frequency, rpm / 60 x cylinders / 2 (`cylinders` decoded, check C), MODEL for the waveform shape.** Second: leave it to the ridge.
5. **Electro Multivision maintenance guide and EFI warnings.** Not sent. **Recommended: the upper right corner (where Toyota put warnings) shows real events only: tyre temperature above a set mark, wheel lock or spin from slip ratio with brake or throttle (decoded, check C), each labelled as inferred.** Second: the corner stays dark.
6. **Electro Multivision TV and video when parked.** Not sent. **Recommended: an attract mode after 30 s parked: the tube replays the last recorded pull (rpm, sun and grid from a fixed ring buffer), labelled REPLAY, live data back on the first input.** Second: dim the tube to a standby raster to save burn-in.
7. **TEMS and ECT pattern pages.** Not sent as modes. Recommended: four thin bars of suspension travel (decoded, check C). Second: none.
8. **Fuel-efficiency history.** Fuel is decoded (check C): a 10-bar history of fuel used per distance in 30 s bins, as Toyota drew it, is honest, not a gap. Optional.
9. **Coolant, oil and other temperatures and pressures of the Lagonda and Toyota pages.** Not sent. Covered by 5.
10. **Real sky and scenery** of Out Run's stages. Not sent. The sun from rpm and the ridge from elevation replace it (brief).

## 6. Edge cases

- **Max rpm 0 or tiny, idle near max:** if shift - idle is under 500 rpm, light one slat at idle and fill the rest only from throttle-free rpm rise; never divide by zero; the strip uses `shift_fraction`.
- **12,000+ rpm and 4,500 rpm diesels:** slats scale with the car; hysteresis 75 rpm suits both (about 0.6% and 2%).
- **EVs:** no idle and one gear: slats from 0, gear shows 1 or D as Signals gives it; the firing-frequency trace hides when cylinders is 0.
- **Reverse:** grid scrolls toward the horizon; gear R in ink (the brief keeps red out of the sun).
- **Neutral:** N; sun follows free revs.
- **Gear above 10:** two digits on the same pad, cap held.
- **mph and km/h:** the unit label follows Signals; Out Run's km/h is the default only for the label style.
- **hp and kW:** unused unless a page shows power.
- **Limiter bounce of about 50 rpm a frame:** 75 rpm hysteresis on each slat and on the strip's last pod.
- **Spikes and crashes:** clamp G for the highlight; the convergence jolt is event-only, at most once a second.
- **Airborne:** position y jumps; the ridge samples by distance and clamps steps to 3 m, so a jump does not draw a spire.
- **Stale or paused stream:** the host hides the dash; the attract mode never runs on a stale stream.
- **Palette extremes and random palettes:** map into the P22 triangle (1.4); keep the sun's luminance caps; red never on a slat.
- **Font swap:** chrome bands, horizon cut and the shear apply to any font's MSDF.
- **Cockpit and hood:** hood 1700 x 400 has less height: grid band and sun radius scale with the window height; scanline pitch stays even.
- **Reduced flash:** strip breath at 2 Hz; horizon pulse off; boot degauss wobble off; no convergence jolt.
- **Colour-only cues and colour vision:** the limiter cue is width plus colour (1 to 3 px); sun stages read by count and height, not hue.
- **Burn-in (CRT and OLED alike, C10):** idle sun dim, the grid moves only when driving, and the attract mode or a standby raster takes over when parked.

## 7. Signature moments

**Limiter (only the strip strobes):**
- A. The brief: crown strip strobes at 10 Hz; the horizon line heats to accent, steady.
- B. **Recommended: A plus beam overdrive.** At the limiter the horizon is drawn with a higher beam current: its scanline spreads from 1 to 3 px and its halo ring appears (C7, C8), steady, so the cue is physical and readable without colour.
- C. The sun's last slat closes the disc and its scanline gaps fill completely (bright content fills its gaps, C7), a steady "full sun".

**Gear change:**
- A. The brief: a 1-frame pulse runs outward along the horizon at 4000 px/s while the sun sinks with the real rpm.
- B. **Recommended: A plus a raster repaint of the gear.** The new glyph is drawn at 60% on frame 1 and a brighter scan band sweeps it top to bottom in 33 ms, as a field is painted; the value is never late.
- C. Out Run's own way (B8): no animation, the bar and sun simply relax with the revs. Kept for reduced-flash mode.

## 8. What it needs from the 4K library

The pre-build (PR #18, branch `claude/project-thread-ib67x9`) made **no kit** for this dash: `kits.rs` lists SynthwaveOutrun among "Dashes with nothing layout-independent to make", PREBUILD.md says its "chrome bevel follows its font atlas", and there is no `assets/clusters/synthwave_outrun/prebuild/manifest.json`. That collides with the asset rule (one bespoke ComfyUI and one bespoke Blender asset, no procedural-only cluster) and with the Retro wish.

Already in the library and usable:
- `chrome_matcap` and `chrome_fresnel` (computed, KineticLiquidMetal and FaradayMandala kits): shading the chrome speed bands from real chrome.
- Saira Condensed TTFs including ExtraBold (`assets/fonts/saira-condensed`); the MSDF atlas is still to bake (brief).
- `assets/scifi/scanline_2px.png`, `scanline_3px.png`: flat multiply masks; they cannot widen with brightness (C7), so the dash computes scanlines analytically and these are fallbacks only.
- `assets/scifi/phosphor_triad.png` (36 x 36, 3 px aperture-grille stripes, made for this dash): that is a Trinitron grille, not the slot mask of TV-class tubes (C1, C2); keep it as the `aperture` option.
- `assets/scifi/plate_synth_sun.png` (2048, an SDXL flat graphic): flat and not 4K; the brief draws the sun procedurally, so it is not used.
- The kit's optical glass (`docs/specs/optics.md`) for the faceplate's reflection and rim.

Library check against the Retro list: the library has no CRT parts (slot-mask phosphor screen, phosphor grain, faceplate halo, tube bezel plastic).

LIBRARY GAP: slot-mask phosphor screen tile, 4096 x 4096 seamless, in-line RGB stripes broken into offset slots at a 6 px triad pitch (2 px per stripe) with a black matrix, plus `delta` dot-triad variant; the realistic colour screen under every lit pixel, at low strength so small text keeps its edges; computed (exact geometry) in asset-forge.
LIBRARY GAP: P22 phosphor screen grain, 4096 x 4096 seamless, the fine powder texture of a settled phosphor coat seen through faceplate glass, greyscale, used as a 3 to 5% modulation of lit areas only; distinct from RetroCyber's VFD `phosphor_grain`; ComfyUI.
LIBRARY GAP: faceplate halation kernel, 256 x 1 radial table: flat disc of diameter 3.49 t with a defined edge (US 4,310,784) for t = 10 mm and 12 mm at 1.80 px per mm; computed.
LIBRARY GAP: CRT faceplate material swatch for Blender, a dark-tinted curved glass with an aluminised phosphor backing seen off (dark charcoal grey, faint specular), 4096 x 4096 reference render with three light azimuths, so the tube reads as glass in the cockpit render; Blender.
LIBRARY GAP: moulded black ABS dash-hood swatch, 4096 x 4096 seamless, the fine stipple grain of 1980s Japanese dash plastic, normal and roughness passes, for the `dash` bezel; Blender (ComfyUI for the grain).

Layout-dependent and not gaps: the tube bezel and hood shape (sized by the board), the sun, grid and ridge (procedural by the brief).

## 9. Departures and open questions for the board

1. **Family dialect.** `docs/design/families/sci-fi.md` puts SynthwaveOutrun in the VFD dialect with RetroCyber. This note proposes a raster CRT dialect so the two cannot be mistaken at thumbnail size. The family spec needs a line for it (lead).
2. **Accent role.** The family reserves magenta accent for shift and limiter, but the brief paints the sun's base in accent. Either the sun takes `secondary` to a new sun role, or the family rule names the sun as the one exception.
3. **Scanline pitch.** A true 224-line raster over 520 px is 2.32 px a line (**Computed**), too fine to show the beam profile on 4K and prone to moire on the WRGB subpixels. Proposed: 4 px (130 lines cockpit, 100 hood), with 6 px as the alternative. Even pitch only.
4. **Mask strength and pitch.** A real triad is finer than a scanline; at 4K it cannot be drawn to scale, so the mask is a faint 6 px texture. The board decides how visible it is at 1:1.
5. **Black.** A switched-off CRT face is grey under room light; at night the lit black is black. The dash keeps OLED black outside lit content and shows the tube only through the glass reflection and the bezel.
6. **Geometry.** Real tubes are adjusted to look rectangular (C10); the content stays rectangular, and the curvature shows in the reflection and in a gentle corner fall-off.
7. **Curvature clamp** 0.035 per metre (section 4).
8. **Ridge sampling** by distance (5 m), not by frame.
9. **The real gear.** Out Run is two-speed LOW and HIGH (B8); the dash shows the real gear number. A LOW and HIGH style label is a possible flourish for two-gear cars only.
10. **Benchmark check.** Step 1 needs one clear photo of the Out Run screen in a cabinet to measure the HUD's position and the scanline look, and one Electro Multivision screen photo (Toyota downloads).

## 10. Sources

**Benchmark and period hardware**
- Wikipedia, Out Run: https://en.wikipedia.org/wiki/Out_Run
- PixelatedArcade, OutRun technical specs: https://pixelatedarcade.com/games/outrun/techspecs
- Cannonball (Chris White), `ohud.cpp`: https://raw.githubusercontent.com/djyt/cannonball/master/src/main/engine/ohud.cpp ; `oferrari.cpp`: https://raw.githubusercontent.com/djyt/cannonball/master/src/main/engine/oferrari.cpp
- Toyota, "Toyota introduces Electro Multivision" (7 January 1985): https://global.toyota/en/detail/7790394
- Jalopnik, "Can We Emulate An Aston Martin Lagonda's CRT Dash?" (2014): https://www.jalopnik.com/can-we-emulate-an-aston-martin-lagondas-crt-dash-1627568648/
- Hagerty UK, "12 of our favourite digital dashboards": https://www.hagerty.co.uk/articles/12-favourite-retro-digital-dashboards-volume-one/
- Wikipedia, Battlezone (1980): https://en.wikipedia.org/wiki/Battlezone_(1980_video_game)

**CRT physics**
- Jed Margolin, "The Secret Life of XY Monitors": https://jmargolin.com/xy/xymon.htm
- Wikipedia, Phosphor: https://en.wikipedia.org/wiki/Phosphor
- Patrick Jankowiak, "Cathode Ray Tube Phosphors Of Interest To The Experimenter" (rev. 20090315): https://curtpalme.com/forum_archived/crt_phosphor_research_20090315.pdf
- Wikipedia, Vector monitor: https://en.wikipedia.org/wiki/Vector_monitor
- A. Duffey, "Scanline simulation math" (gist, with the Blur Busters comment): https://gist.github.com/aduffey/d7468b8068d6124641ff0762c2b373e8
- US 4,310,784, CRT face plate construction for suppressing the halo: https://patents.google.com/patent/US4310784A/enExample
- CRT Database, Toshiba CF19H32: https://crtdatabase.com/crts/toshiba/toshiba-cf19h32

**Photos:** the Commons and Toyota links in section 2.

**Repo:** `program/phases/07-cluster-factory/CATALOG.md` (brief, maths row, Retro wish), `docs/design/families/sci-fi.md`, `crates/fh6-clusters/src/clusters/retro_cyber/RESEARCH.md`, `crates/fh6-clusters/src/clusters/glyph_dot_matrix/RESEARCH.md`, `assets/ASSETS.md` (scifi rows), PR #18 `tools/asset-forge/src/prebuild/kits.rs` and `PREBUILD.md`, `docs/TELEMETRY_USAGE_MATRIX.md`.
