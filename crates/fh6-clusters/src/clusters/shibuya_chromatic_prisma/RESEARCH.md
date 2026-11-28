# ShibuyaChromaticPrisma: research note

Build #26, catalog row 26, category Liquid Glass, Target `concept`, family Sci-Fi, view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on 2026-11-28 from the linked source. **Computed** means worked out here with `python3 -I` and numpy from cited constants; the inputs and the numbers are given so step 1 can re-run them. **Our reading** is a conclusion or proposal for this dash, open to the board and the direction check. "Unconfirmed" marks what step 1 must still check.

## 1. The real display or benchmark

**The benchmark: a photograph of a flint-glass prism dispersing a mercury-vapour lamp on black** (Wikimedia Commons, "Light dispersion of a mercury-vapor lamp with a flint glass prism IPNr°0125.jpg", photo taken 18 June 2009, CC BY-SA 3.0 AT, 2309 x 2818 px). **Cited**: [Commons file page](https://commons.wikimedia.org/wiki/File:Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_IPNr%C2%B00125.jpg). It is used on the English Wikipedia pages "Dispersive prism", "Visible spectrum" and "Spectroscopy" (same page, file usage list).

Why this one and not the white-light renders. The famous Commons render of a dispersing prism carries a curator's note that it "should not be used to illustrate how a prism disperses light": the colours must start to separate inside the glass, not at the exit face (**Cited**, [Spigget render page](https://commons.wikimedia.org/wiki/File:Dispersive_Prism_Illustration_by_Spigget.jpg)). Wikipedia says the same of the Dark Side of the Moon cover, which splits the beam "only after leaving the prism's rear facet" (**Cited**, [Dispersive prism](https://en.wikipedia.org/wiki/Dispersive_prism)). A real photograph cannot make that mistake. A mercury lamp is also the closest real analogue of what this dash bends: a display's light is three narrow bands (optics.md section 5), and a mercury lamp is a few narrow lines, so both throw separate lines with black between them, not a smeared rainbow.

What makes the benchmark recognisable (to be confirmed against the photo at step 1):

| # | Feature | Label |
|---|---|---|
| B1 | A clear triangular block of flint glass on black, readable only by its edges and faces catching light (dark-field), with no frame or painted outline. | Our reading of the photo; optics.md section 7 cites dark-field lighting [S21] |
| B2 | Discrete coloured lines, not a continuous band: low-pressure mercury emits at 404.7, 435.8, 546.1 and 578.2 nm in the visible. | **Cited**, [RP Photonics, mercury vapour lamps](https://www.rp-photonics.com/mercury_vapor_lamps.html) |
| B3 | Blue and violet are turned most, red least, so the order of the lines is fixed by physics. | **Cited**, [Dispersive prism](https://en.wikipedia.org/wiki/Dispersive_prism) |
| B4 | The spread is a few degrees, not a fan of 30 degrees: through N-SF11 at 60 degrees, set for minimum deviation at 546.1 nm, the lines leave at 74.741, 71.955, 67.263 and 66.534 degrees (404.7 to 578.2 nm), so green to yellow is only 0.73 degrees apart. | **Computed** (Sellmeier of N-SF11, section 4) |
| B5 | Most dispersing prisms are equilateral, and flint is used for strong visible dispersion. | **Cited**, [Dispersive prism](https://en.wikipedia.org/wiki/Dispersive_prism); [Edmund Optics 50 mm N-SF11 equilateral prism](https://www.edmundoptics.com/p/50mm-n-sf11-uncoated-equilateral-prism/9863/) (product exists) |

Mood references kept from the brief, not the benchmark:
- **Shibuya on a wet night.** Commons holds CC BY 2.0 photographs of the Shibuya crossing "during a rainy evening with many umbrellas and reflections on the ground" (**Cited**, [Tokyo (16230379595).jpg](https://commons.wikimedia.org/wiki/File:Tokyo_%2816230379595%29.jpg)). What they show (our reading, step 1 confirms on the photos): signs mirrored in the road as long vertical streaks, not as sharp images. Section 4.4 gives the physics.
- **Blade Runner 2049 screens (Territory Studio).** The director's words for the look were "abstract, organic, optical, physical"; the studio brought "optical lenses, old school projections, microfiche" into the studio, and its morgue screen has stages that "shunt across physically as optical lenses", built as "looping and holding states triggered on set" (**Cited**, [vfxblog, 2017](https://vfxblog.com/2017/11/08/a-visual-journey-through-the-screen-graphics-of-blade-runner-2049/)). The catalog's phrase "degradation as register" was not found in that source; the source speaks of "degradation of all everything mechanical" as the look reference. Step 1 should quote the source, not the phrase.
- **Cyberpunk 2077's glitch as event.** Only search titles were found (a GDC talk "User Interface in Cyberpunk 2077", [gdcvault](https://gdcvault.com/play/1034160/User-Interface-in-Cyberpunk-2077)). Unconfirmed; the Sci-Fi family spec already holds the rule (glitch and aberration only as events).

**A better target?** None found. A prism spectroscope (collimator, prism, telescope; [Commons "Simple spectroscope.jpg"](https://commons.wikimedia.org/wiki/File:Simple_spectroscope.jpg)) is the same bench with a slit and a scale. Our reading: borrow its slit and scale (section 5) and keep the mercury-lamp photograph as the benchmark.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Benchmark: flint prism dispersing a mercury lamp, black ground | https://upload.wikimedia.org/wikipedia/commons/1/1f/Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_IPNr%C2%B00125.jpg (opened, loads 2309 x 2818) | [file page](https://commons.wikimedia.org/wiki/File:Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_IPNr%C2%B00125.jpg) |
| 2 | Same lamp and flint prism, second photograph | https://upload.wikimedia.org/wikipedia/commons/a/a4/Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_PNr%C2%B00124.jpg | [file page](https://commons.wikimedia.org/wiki/File:Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_PNr%C2%B00124.jpg) |
| 3 | Same lamp through an Amici direct-vision prism (lines with no net turn) | https://upload.wikimedia.org/wikipedia/commons/4/44/Light_dispersion_of_a_mercury-vapor_lamp_with_an_Amici_direct-vision_prism_PNr%C2%B00122.jpg | [file page](https://commons.wikimedia.org/wiki/File:Light_dispersion_of_a_mercury-vapor_lamp_with_an_Amici_direct-vision_prism_PNr%C2%B00122.jpg) |
| 4 | Counter-example: render that splits only at the exit face (flagged on Commons) | https://upload.wikimedia.org/wikipedia/commons/0/0b/Dispersive_Prism_Illustration_by_Spigget.jpg | [file page](https://commons.wikimedia.org/wiki/File:Dispersive_Prism_Illustration_by_Spigget.jpg) |
| 5 | Newton's experimentum crucis, photographed (two prisms) | https://upload.wikimedia.org/wikipedia/commons/a/a7/Newton%27s_Experimentum_Crucis_%28Grusche_2015%29.jpg | [file page](https://commons.wikimedia.org/wiki/File:Newton%27s_Experimentum_Crucis_%28Grusche_2015%29.jpg) |
| 6 | Recombination of a spectrum into white (engraving) | https://upload.wikimedia.org/wikipedia/commons/7/7e/Recombination_of_spectrum_from_white_light_Wellcome_M0010895.jpg | [file page](https://commons.wikimedia.org/wiki/File:Recombination_of_spectrum_from_white_light_Wellcome_M0010895.jpg) |
| 7 | High- and low-dispersion prisms compared | https://upload.wikimedia.org/wikipedia/commons/e/ef/Prisms_with_high_and_low_dispersion.png | [file page](https://commons.wikimedia.org/wiki/File:Prisms_with_high_and_low_dispersion.png) |
| 8 | Real prism photo, flat spectrum | https://upload.wikimedia.org/wikipedia/commons/f/fa/Prism_flat_rainbow_%28cropped%29.jpg | [file page](https://commons.wikimedia.org/wiki/File:Prism_flat_rainbow_%28cropped%29.jpg) |
| 9 | Prism spectroscope (collimator, prism, telescope) | https://upload.wikimedia.org/wikipedia/commons/1/1a/Simple_spectroscope.jpg | [file page](https://commons.wikimedia.org/wiki/File:Simple_spectroscope.jpg) |
| 10 | Fraunhofer's prisms (museum objects) | https://upload.wikimedia.org/wikipedia/commons/1/14/Fraunhofer_prisms.jpg | [file page](https://commons.wikimedia.org/wiki/File:Fraunhofer_prisms.jpg) |
| 11 | Shibuya crossing, rainy evening, reflections on the ground | https://upload.wikimedia.org/wikipedia/commons/5/5f/Tokyo_%2816230379595%29.jpg | [file page](https://commons.wikimedia.org/wiki/File:Tokyo_%2816230379595%29.jpg) |
| 12 | Shibuya crossing in the rain with umbrellas | https://upload.wikimedia.org/wikipedia/commons/9/9f/Tokyo_%2816204487106%29.jpg | [file page](https://commons.wikimedia.org/wiki/File:Tokyo_%2816204487106%29.jpg) |
| 13 | Shibuya crossing at night (its uploader titled it "Blade Runner") | https://upload.wikimedia.org/wikipedia/commons/3/33/Shibuya_Crossing_at_night_%2814738088849%29.jpg | [file page](https://commons.wikimedia.org/wiki/File:Shibuya_Crossing_at_night_%2814738088849%29.jpg) |
| 14 | Blade Runner 2049 morgue and baseline screens (optical lenses, stepped states) | page figures only | [vfxblog](https://vfxblog.com/2017/11/08/a-visual-journey-through-the-screen-graphics-of-blade-runner-2049/) |

Image URLs 2 to 13 follow Commons' MD5 path rule from the file name; only URL 1 was opened. Drive `references` folder (holds `SOURCES.md`; the PC fetches the images): https://drive.google.com/drive/folders/15PdjRJh_S_yZcEPAeHsAzQnvt-otvVoi

## 3. How the real thing behaves, with motion notes

A prism has no states of its own. It responds at once, it never lags, and it moves only when its geometry or its light changes. Our reading for each state, using the instrument of section 4:

| State | What moves | What stays still |
|---|---|---|
| Start-up | The 3-state machine of the family (initial, action, loop): the slit opens over about 400 ms and the white beam runs from the slit into the glass at once (light has no travel time on screen). The fan appears with the beam, exact from the first frame. Gear and speed live from frame 1. | The prism body, which is lit by the studio (section 4.5) before the beam. |
| Idle | Prism at minimum deviation for green; the fan is at its shortest rpm length. The three bands nearly overlap (section 4.2). | Everything else. No breathing or drift: "never always-on" (brief). |
| Cruise | Fan length follows rpm; the beam's brightness follows throttle. Grip keeps the prism at rest. | Prism angle, bands, mirror (dry). |
| Pull to the limiter | Fan length runs out to the end of the 1 px rpm ruler; the strip fills. | Prism angle if the car has grip. |
| Limiter / shift | Fan at the ruler's end, steady, in the shift colour; only the strip strobes (section 7). | The fan and the prism do not flash. |
| Gear change | Section 7. | Bands do not split for a gear change. |
| Braking | Beam brightness falls with throttle (and brake may dim the slit further, section 5); the fan shortens with rpm. | Prism, unless the rear steps out. |
| Drift / lateral G | Body slip turns the prism toward grazing emergence: the spread grows, the bands dim by Fresnel loss, and past the critical angle blue, then green, then red go out (section 4.3). Lateral G swings the key light (optics.md section 7). | The digits: they never split. |
| Reverse / neutral | Glyph R or N. The prism stays at minimum deviation (no slip sign in reverse is trusted, section 6). | Fan at idle length. |
| Shut-down / stale stream | The app hides the dash. If a fade is drawn, the slit closes: beam and fan go dark together. | |

Spring constants: the prism angle follows body slip, which `Signals` already smooths with an 80 ms time constant (`crates/fh6-kit/src/cluster.rs`). **Cited** from the repo. No extra spring is needed on the prism; the fan length can use the rpm smoothing that exists.

## 4. The maths and physics

### 4.1 The model and its constants

Deviation of a prism of apex A for a ray meeting its first face at theta1 (optics.md section 10, `fh6_kit::optics::prism_deviation`):

delta = theta1 + theta2 - A, with sin theta1 = n sin theta1', theta2' = A - theta1', sin theta2 = n sin theta2'.

Minimum deviation (symmetric passage): n = sin((A + delta) / 2) / sin(A / 2). Angular dispersion there: d delta / d n = 2 sin(A/2) / sqrt(1 - n^2 sin^2(A/2)) (Born and Wolf, via [Dispersive prism](https://en.wikipedia.org/wiki/Dispersive_prism); the formula is the derivative of the line above, checked numerically below).

Glasses (Sellmeier, lambda in micrometres):

| Glass | B1, B2, B3 | C1, C2, C3 (um^2) | n_d, V_d | Source |
|---|---|---|---|---|
| N-BK7 | 1.03961212, 0.231792344, 1.01046945 | 0.00600069867, 0.0200179144, 103.560653 | 1.5168, 64.17 | optics.md S2, in the kit as `Dispersion::CROWN_BK7` |
| N-SF11 | 1.73759695, 0.313747346, 1.89878101 | 0.013188707, 0.0623068142, 155.23629 | 1.78472, 25.68 | **Cited**, [refractiveindex.info, SCHOTT Zemax catalogue 2017-01-20b](https://refractiveindex.info/?shelf=specs&book=SCHOTT-optical&page=N-SF11); not in the kit |
| Sapphire (o) | optics.md section 1 | | 1.7682, 72.3 | `Dispersion::SAPPHIRE` |

N-SF11's extinction coefficient at 587.6 nm is k = 3.13e-8 (absorption 0.0067 per cm), so by Beer-Lambert (optics.md section 6) the prism is effectively colourless (**Cited**, same page). "Tint comes from absorption only" therefore means no visible tint at all for either glass.

### 4.2 Computed values (A = 60 degrees; channels 611, 549, 464 nm from optics.md section 5)

| Quantity | N-BK7 | N-SF11 | Sapphire |
|---|---|---|---|
| n at R, G, B | 1.51587, 1.51857, 1.52410 | 1.78136, 1.79135, 1.81380 | 1.76694, 1.77051, 1.77775 |
| delta_min at the d line | **38.647** | 66.343 | 64.279 |
| F to C spread, each at its own minimum | **0.709** (0.7090) | 3.939 | 1.308 |
| F to C spread at the d-line incidence | 0.710 (0.7097) | 4.001 | 1.313 |
| theta1 for minimum deviation of G | 49.401 | 63.595 | 62.283 |
| Deviation R, G, B at that incidence | 38.565, 38.803, 39.291 | 65.930, 67.190, 70.242 | 64.129, 64.566, 65.471 |
| B to R spread there | 0.726 | 4.312 | 1.342 |
| Same from d delta / d n x (n_B - n_R) | 0.724 | 4.179 | 1.332 |

All in degrees, **Computed**. n = 1.5 gives delta_min = 37.1808 degrees, which the kit's test already asserts. The sapphire indices match optics.md section 5 (1.7669, 1.7705, 1.7778).

**Catalog correction.** The maths row says "N-BK7 gives 38.647° with an F to C spread of 0.708°". 38.647 holds. The spread computes to 0.709 degrees (0.7090 with each line at its own minimum, 0.7097 at the d line's incidence), so 0.708 is a rounding slip: the row should read 0.709 and say which definition it uses. The lead reports it.

### 4.3 Slip turns the prism (the category note), and what that does

Our reading of the instrument: the white beam is fixed, rising into the left face at 19.40 degrees above the horizontal, which is the N-BK7 minimum-deviation incidence for a point-up prism with a level base (49.401 - 30 = 19.401). At grip the green band leaves the right face falling at -19.40 degrees: the symmetric picture every reader knows. Body slip turns the prism by phi, so theta1 = 49.401 - phi.

| phi | theta1 | Deviation R, G, B | Green leaves at | Split at a 400 px fan (G-R, B-G, B-R px) | Blue transmission |
|---|---|---|---|---|---|
| 0 | 49.40 | 38.565, 38.803, 39.291 | -19.40 | 1.7, 3.4, 5.1 | 0.88 |
| 10 | 39.40 | 40.420, 40.726, 41.361 | -21.32 | 2.1, 4.4, 6.6 | 0.86 |
| 15 | 34.40 | 43.743, 44.156, 45.027 | -24.75 | 2.9, 6.1, 9.0 | about 0.79 |
| 18 | 31.40 | 47.881, 48.505, 49.885 | -29.10 | 4.4, 9.6, 14.0 | about 0.6 |
| 19 | 30.40 | 50.282, 51.126, 53.147 | -31.72 | 5.9, 14.1, 20.0 | about 0.5 |
| 19.6 | 29.80 | 52.425, 53.627, 57.597 | -34.23 | 8.4, 27.7, 36.1 | under 0.3 |

**Computed** (N-BK7; transmission is the product of the exact Fresnel transmission at both faces, optics.md section 3; the 0.6 and 0.5 rows are read between computed points at theta1 = 31 and 30.5). Total internal reflection at the second face starts at phi = 19.661 (blue), 20.079 (green) and 20.283 (red). The other turning direction narrows the spread (0.630 degrees at phi = -20), so the instrument should turn the prism by the size of the slip, not its sign.

What this gives the brief, all exact:
- **"Coincident with grip"** is nearly true, not exactly: at minimum deviation the three bands sit 1.7 and 3.4 px apart at the tip of a 400 px fan, inside a 6 px band core, so they read as one white ray with faint coloured edges. A prism is never free of dispersion. If the board wants exactly one white ray, see departure 2.
- **"Up to 18 px per band"** happens near phi = 19.3 to 19.6 (B-G 17.6 to 27.7 px), within half a degree of the critical angle. To make the split grow evenly with slip, solve phi from a target split at load (a table of B-R against phi, inverted): the prism is still really at that angle, so the split stays physics, not a pixel offset. Our proposal: target B-R = 5.1 + 31.0 x clamp((|beta| - 2 deg) / 28 deg, 0, 1) px at full fan length, which reaches 36.1 px at a 30 degree body slip. Constants are a design choice for the board.
- **The fan's width grows with rpm by itself**: the split in pixels is fan length x angular spread, so a longer rpm fan is a wider fan. The brief's "length and opening grow with rpm" needs no extra rule.
- **The split starts inside the glass.** Inside N-BK7 at grip the R and B rays differ by 0.179 degrees, 0.44 px over a 140 px path: invisible, but the drawn bands should leave from a common point at the entry face, never fan out from the exit face (the benchmark's lesson, section 1).
- **Bands dim as they spread** (Fresnel loss rises toward grazing exit), which keeps the colour budget in check during a drift without a hand-tuned fade.
- **Spin-out blackout.** Past phi = 19.661 the blue band is totally reflected and vanishes, then green at 20.079, then red at 20.283. A real, exact event; section 7 uses it.

N-SF11 is the classic dispersing glass but a poor fit here: its minimum spread is already 4.3 degrees (30 px at 400 px, so never "one ray"), and its working range before blue's critical angle is under 9.5 degrees of turn (blue total reflection at theta1 = 54.145). Our reading: build the fan in N-BK7, which the kit already has; keep N-SF11 for an optional rich mode. Adding N-SF11 to the kit means adding it to optics.md section 1 first, with this source and a test (the optics rule).

### 4.4 The wet-road mirror

- The game sends `WheelInPuddle*` as 1 in a puddle, 0 out: a flag, not a depth (**Cited**, [FH6 Data Out documentation](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation); the decoder says the same). The brief's "WheelInPuddleDepth" does not exist in FH6. Mirror strength = wheels in puddle / 4 (`wheels_in_puddle()`), eased over the brief's 200 ms.
- Water reflects 2.0% face-on and much more near grazing: 21.2% at 75 degrees, 34.8% at 80, 58.4% at 85, 80.4% at 88 (**Computed**, exact Fresnel, n = 1.333 from optics.md section 1). Our reading: weight the mirror by the value at one stated viewing angle (85 degrees gives 0.58), labelled MODEL.
- A rippled surface turns each light into a glitter pattern of many glints. For a high source its angular length is four times the steepest wave slope, its width-to-length ratio is the sine of the source's elevation, and a source level with the observer gives half the size with the same ratio (**Cited**, J. A. Shaw, Optics and Photonics News 10(3), 1999, [NOAA copy](https://psl.noaa.gov/outreach/education/science/glitter/)). So lights low on a wet street become long thin vertical streaks. Our reading: streak length = 2 x 4 x s_max (source at eye height) with s_max from `SurfaceRumble` (MODEL scale), width = length x sin(elevation) with a fixed low elevation, over the pre-built `wet_asphalt_ripple` height map.

### 4.5 The prism body is a block lying on the screen

The beam runs in the screen plane through a triangular block of glass seen from above. The block is drawn by `OpticalGlass` (outline, profile, Fresnel, studio, optics.md sections 2 to 8): its bevel catches the studio as thin light lines, and the gear sits under its flat top, where the contact model leaves numerals unbent. The in-plane beam is the dash's own CPU trace of section 10 (route a of the catalog). Both use the same index table.

### 4.6 What the unit tests should assert (no GPU)

1. n = 1.5, A = 60: delta_min = 37.18 (kit test, keep).
2. N-BK7 at the d line: delta_min = 38.647 within 0.001; F to C spread 0.709 within 0.001.
3. N-BK7 at theta1 = 49.401: deviations 38.565, 38.803, 39.291 within 0.002; B-R spread 0.726.
4. Blue goes out first: `prism_deviation` is None for blue at phi > 19.661 while red still transmits up to phi = 20.283.
5. The split in px is fan length x (delta_B - delta_R) in radians: 400 px at phi = 0 gives 5.1 px.
6. Bands are 0 apart when the dash's "exact one ray" option (departure 2) is on.
7. Mirror strength is 0, 0.25, 0.5, 0.75, 1 for 0 to 4 puddle flags; water Fresnel at 85 degrees = 0.584.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Fan length, strip, ruler | `rpm`, `max_rpm`, `idle_rpm` | verified | fraction f = (rpm - idle) / (max - idle), guarded; length L = L0 + (Lmax - L0) f |
| Gear glyph in the prism | `gear` | verified | `fh6_kit::Gear`: 0 = R, 1 to 10 forward, anything else N |
| Speed above the beam | `speed` | verified | km/h or mph by the user's unit |
| Beam brightness | `throttle` | verified | luminance 0.35 + 0.65 x throttle (proposal); brake may close the slit to 60% |
| Prism turn phi | body slip beta = atan2(v_x, v_z) from `velocity` | derived from verified fields | phi from the target-split table of 4.3, gated by speed (already in `car_slip_angle_rad`) |
| Prism turn, extra lead | `yaw_rate` | decoded, offset confirmed at check C | optional: add k x yaw rate x sign agreement; fallback a_lat / V |
| "Grip" gate | rear `tyre_slip` (normalised: 0 grip, above 1 loss of grip) | decoded, offset confirmed at check C | phi forced to 0 while both rear values are below 1; fallback |beta| < 2 degrees |
| Four tyre beams | `tyre_combined_slip` x4 (normalised) | decoded, offset confirmed at check C | each a small prism of the same function, turned by clamp((|s| - 0.5) / 1.0) x 19.6 degrees; fallback: rears from beta, fronts from steer against yaw |
| Mirror strength | `puddle` x4 (flags) | decoded, offset confirmed at check C | count / 4, eased 200 ms |
| Mirror ripple | `surface_rumble` x4 | decoded, offset confirmed at check C | streak length, MODEL; fallback: none (mirror stays still) |
| Kerb ripple in the mirror | `rumble_strip` x4 | decoded, offset confirmed at check C | short ring per flagged wheel (idea, section 5 list) |
| Key light azimuth | `g_lat` | verified | optics.md section 7: -135 + 25 x clamp(g / 1.5) degrees |
| Glass lean | `g_lat`, `g_long` | verified | 6 degrees per g (`tiltDegPerG`) |

Note for the lead: the FH6 page now says "Torque in newton-meters" and "Angular velocity ... (rad/s)", while `telemetry.rs` says the page "states no unit" for torque. This dash does not use torque.

**What the game cannot send.** The benchmark and the mood references have these parts with no telemetry:

1. **The spectroscope's wavelength scale.** No spectrum is sent. **Replacement (recommended): the 1 px rpm ruler of the brief, drawn as a spectroscope reticle** (fine ticks every 500 rpm on the driven car's scale, a long tick at the shift rpm), so the fan's tip reads rpm. Second idea: a gear-ratio scale whose ticks are each gear's speed per rpm, learned live into a `[f32; 10]` table, so the fan tip sits on the tick of the gear you are in.
2. **The street's neon signs and billboards (media).** **Replacement (recommended): the dash's own light is the neon.** The strip pods and the fan's tip are mirrored in the wet road as glitter streaks (4.4), so the street glows only with what the car drives. Second idea: when the car stands still in neutral, the beam's reflection in the mirror lays the white bars of a scramble crossing (stripe count = gear ratio table entries learned so far), off as soon as speed exceeds 1 m/s.
3. **Rain and weather.** Not sent. **Replacement (recommended): puddle flags open the mirror, rumble flags ripple it** (table above). Second idea: wheel rotation speed against car speed (wheel slip) sends spray streaks across the mirror on the spinning wheel's side, MODEL.
4. **Traffic, pedestrians, signals.** Not sent; nothing replaces them on the bench. Our reading: leave them out; the four tyre beams are the only small figures.
5. **The lamp's own lines (mercury).** The dash's "lamp" is white. **Replacement (recommended): the three display channels are the lines** (611, 549, 464 nm, optics.md section 5). Second idea: the beam's colour is the palette primary and only its non-zero channels appear in the fan (a cyan beam throws only green and blue), which is exact.

## 6. Edge cases

- **Max rpm 0 or tiny, idle near or above max**: f is clamped to [0, 1] with a guarded denominator; fan at L0, strip dark. `decode` already guarantees finite floats.
- **12,000+ rpm and 4,500 rpm diesels**: the ruler's ticks follow the car's own max; tick spacing switches 500 or 1000 rpm so ticks stay at least 12 px apart.
- **EVs**: idle 0, one gear. f from 0; glyph shows the gear the game sends; no gear-change moment fires when gear never changes.
- **Reverse**: beta flips sign meaning; keep phi = 0 below 3 m/s and in reverse.
- **Neutral, gear above 10**: `fh6_kit::Gear` maps every byte other than 0 to 10 to N, so a gear above 10 shows N; the N glyph sits in the same cap height as the digits.
- **mph and km/h, hp and kW**: speed by the user's unit; no power readout.
- **Limiter bounce of about 50 rpm a frame**: the fan tip at the ruler end uses 1.5% hysteresis so it does not flicker; the shift colour latches until rpm falls 3% below the shift point.
- **Spikes and crashes (huge G)**: beta is already smoothed; clamp phi to 19.6 degrees except in the blackout event; G-light clamps at 1.5 g.
- **Airborne**: velocity stays sane but tyre slip is meaningless; gate phi by the rear tyre flags or keep the last value.
- **Stale or paused stream**: the app hides the dash.
- **Palette extremes and random palettes**: the bands are channels, so a palette never changes their order; a beam colour with a zero channel drops that band (5.5). Warning red is never a band colour by role.
- **The user's font swap**: gear and speed use the swappable font; the slit, ticks and bands are geometry.
- **Cockpit only**: no hood variant.
- **Reduced-flash mode**: strip 2 Hz breath; nothing else flashes in either mode.
- **Colour-only cues and colour vision**: the split is read by width and position, not hue; blue always sits furthest from the apex. The blackout reads as the fan shortening band by band.

## 7. Signature moments

Limiter and shift (only the strip strobes, 10 Hz at 50% duty, 2 Hz breath in gentle mode):
- **A. Steady at the ruler's end (recommended).** The fan reaches the shift tick and takes the shift colour, steady; the strip strobes. This is the brief and keeps the glass calm.
- **B. The slit narrows.** Above the shift rpm the slit closes to a hairline over 120 ms, so the beam and fan thin to a single bright line (steady, no flash), and reopen on the upshift. Exact in spirit: a narrower slit is a sharper spectrum.

Gear change:
- **A. Experimentum crucis (recommended).** For 120 ms a second, inverted prism image slides behind the first and the three bands recombine into one white ray (Newton's recombination, references 5 and 6), then it slides away and the fan reopens at the new rpm. The new glyph appears under the flat top at the moment of white.
- **B. Shutter.** The beam is cut for 60 ms at the slit and comes back; the fan relights from the entry face outwards.
- **C. Lensed entry.** The new digit enters from under the bevel and the contact model bends it as it crosses onto the flat top (optics.md section 4), never split by colour.

Drift event: **the spin-out blackout** (4.3). Above a body slip of about 40 degrees, phi passes the critical angles and the fan goes out blue, green, red, then returns in reverse order as the car straightens. It is an event, never on at grip.

## 8. What it needs from the 4K library

Pre-built for this dash (PR #18, `kits.rs` and `assets/clusters/shibuya_chromatic_prisma/prebuild/manifest.json` on branch `claude/project-thread-ib67x9`): one computed map, `wet_asphalt_ripple` (WetAsphalt, seed 260001, 4096 x 4096 master: Worley stones of 16 px with tones 0.040 to 0.060, height 35% stones plus 65% shallow 260 px ripples; colour, height and normal). It is the street grain under the mirror's streaks. Kit note: "The prism dispersion is the dash's optics; this is only the wet street grain."

Shared parts it can reuse: the Liquid Glass studio (`assets/clusters/liquid_glass/studio_env.png`, octahedral, cleared within 44 degrees of the viewer) for what the prism's bevel mirrors; the Saira Condensed TTFs in `assets/fonts/saira-condensed/` (the kit builds MSDF glyphs at load, so no atlas is needed); the shared `shift_led_32_off` / `_on` caps from SimInputTrace's kit if the board accepts 32 px caps in 36 px pods.

Still needed (the cluster prompt requires one bespoke ComfyUI and one bespoke Blender asset):

LIBRARY GAP: rain-night street panorama, 4096 x 2048 equirectangular, ComfyUI (SDXL): a dark wet city street at night with distant neon at low radiance, for the prism's environment to mirror and as the cyclorama of the Blender environment below; the bespoke ComfyUI asset.
LIBRARY GAP: rain-night octahedral environment, 4096 x 4096, Blender (Cycles, the Liquid Glass STUDIO rig with the panorama above on the cyclorama, cleared within 44 degrees of the viewer): what the prism's bevel and faces mirror, replacing the studio so the glass reads as on a street.
LIBRARY GAP: slit collimator sprite, about 160 x 56 px (size proposed), Blender, rest key and two G-swing keys: the machined slit jaws the white beam leaves from (the spectroscope's collimator end); the bespoke Blender asset.
LIBRARY GAP: 36 px pod caps off and on, 44 px masters, Blender: the 15-pod crown strip at the brief's 36 px, unless the board takes the shared 32 px caps.

The prism, the beam, the bands and the mirror are computed and are not gaps.

## 9. Departures and open questions for the board

1. **The brief's glass words break the glass rule.** "Tint alpha 44", "1 px rim", "fresnel band" and "neon-tube edges" are all ruled out (CATALOG, glass rule; category note: neon "never as tubes on the glass"). The prism is clear `OpticalGlass` in N-BK7; its edges are drawn by the profile normal and the environment.
2. **One white ray at grip.** A prism always disperses (4.3). Options: (a) accept the 5 px fringe at grip (recommended, exact); (b) a recombining second prism behind the first that slides away with slip, exact zero split at grip (experimentum crucis); (c) a variable-angle liquid wedge whose apex goes from 0 at grip to 60 degrees in a drift (the principle of Canon's Vari-Angle Prism stabiliser; unconfirmed, step 1 reads a source such as the [TV Technology article](https://www.tvtechnology.com/news/camera-lens-image-stabilization-part-2)).
3. **"yaw x 6 + beta x 18 px" becomes a prism angle.** optics.md section 10 forbids made-up channel offsets; slip turns the prism and the pixels follow.
4. **"Rear TireSlipAngle" is normalised in FH6** (0 grip, above 1 loss of grip), not an angle. Use it as the grip gate; beta is the angle.
5. **"WheelInPuddleDepth" is a flag.** Mirror opacity steps by wheels in puddle.
6. **Band colours.** The brief puts the bands in primary, secondary and accent. The maths row says the split is "the deviation at the three channel wavelengths". Recommended: bands are the R, G, B components of the beam's colour (exact); alternative: tint each channel band with a palette role in the order of deviation. The R band versus "warning red is never used" needs a decision.
7. **Glass choice.** N-BK7 (in the kit) recommended; N-SF11 only as an optional mode after it is added to optics.md.
8. **Geometry check.** A 280 px equilateral prism is 242.5 px tall; the fan falls at 19.4 degrees at grip and about 34 degrees at full slip, so a 400 px fan drops 133 to 225 px. The board checks this fits the 1440 x 560 cockpit envelope from (1200, 1515) with the strip at the crown.

## 10. Sources

Benchmark and photos: [Commons, mercury lamp and flint prism IPNr°0125](https://commons.wikimedia.org/wiki/File:Light_dispersion_of_a_mercury-vapor_lamp_with_a_flint_glass_prism_IPNr%C2%B00125.jpg); [Commons, Spigget render and its flaw note](https://commons.wikimedia.org/wiki/File:Dispersive_Prism_Illustration_by_Spigget.jpg); [Commons, Category: Dispersion in prisms](https://commons.wikimedia.org/wiki/Category:Dispersion_in_prisms); [Commons, Tokyo (16230379595)](https://commons.wikimedia.org/wiki/File:Tokyo_%2816230379595%29.jpg); the other file pages in section 2.

Optics and materials: [Wikipedia, Dispersive prism](https://en.wikipedia.org/wiki/Dispersive_prism) (citing Born and Wolf, Principles of Optics, 7th ed., pp. 190 to 193); [refractiveindex.info, SCHOTT N-SF11](https://refractiveindex.info/?shelf=specs&book=SCHOTT-optical&page=N-SF11); [Edmund Optics, 50 mm N-SF11 equilateral prism](https://www.edmundoptics.com/p/50mm-n-sf11-uncoated-equilateral-prism/9863/); [RP Photonics, mercury vapour lamps](https://www.rp-photonics.com/mercury_vapor_lamps.html); J. A. Shaw, "Glittering light on water", Optics and Photonics News 10(3), 43 to 45 (1999), [NOAA copy](https://psl.noaa.gov/outreach/education/science/glitter/); `docs/specs/optics.md` sections 1 to 10 and its sources S2, S6, S7, S21, S23.

Telemetry: [Forza Horizon 6 Data Out documentation](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation); `crates/fh6-core/src/telemetry.rs`; `crates/fh6-kit/src/cluster.rs`; `docs/TELEMETRY_USAGE_MATRIX.md`.

Film and game: [vfxblog, screen graphics of Blade Runner 2049 (Territory Studio notes)](https://vfxblog.com/2017/11/08/a-visual-journey-through-the-screen-graphics-of-blade-runner-2049/); [GDC Vault, User Interface in Cyberpunk 2077](https://gdcvault.com/play/1034160/User-Interface-in-Cyberpunk-2077) (title only, unread).

Variable prism lead (unread): [TV Technology, camera lens image stabilisation part 2](https://www.tvtechnology.com/news/camera-lens-image-stabilization-part-2).
