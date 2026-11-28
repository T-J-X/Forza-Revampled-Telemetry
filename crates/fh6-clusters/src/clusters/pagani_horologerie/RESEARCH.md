# PaganiHorologerie: research note

Build #34, catalog row 8, category Hypercar, Target `recreation`, family Classic Analog, view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

**Labels.** **Cited** means the fact was read on the named page on 2026-11-28 (the link is given). **Photo** means it was seen in a named reference photo (section 2) opened in the PC browser on 2026-11-28; the crops were small (a main dial is about 180 px wide in photo 1), so every Photo point says what step 1 must confirm at full size. **Computed** means it was worked out with `python3`, inputs and numbers shown. **Our reading** is a conclusion or idea for this dash, open to the direction check. "Unconfirmed" says what no source settled. The leads file `hypercar-clusters.json` was written from memory and was used only as a list of things to check: its claim that Hagerty reports a black anodised cluster is **not on the Hagerty page** (read today), so the brief's black anodised case rests on nothing.

**Which Pagani.** The category note asks step 1 to name the car. **Our reading: the Utopia (2022) cluster is the target.** It is the only Pagani whose maker says every dial "subtly reveals part of its mechanism as if it were revealing the skeleton movement" ([Pagani, Utopia page](https://www.pagani.com/pagani-utopia/)), which is the brief's idea. The game has no Utopia but has the 2009 Zonda Cinque, 2010 Zonda R, 2016 Huayra and 2021 Huayra R (`assets/car_brands/car_brands.json`). The Zonda and Huayra share the Utopia's grammar (two large round dials, rpm and speed, around a small central display), so a Zonda or Huayra driver still recognises the family (section 1.3).

## 1. The real display

### 1.1 Feature inventory (Utopia)

| # | Feature | Label and source |
|---|---|---|
| R1 | **All instruments analogue, each showing part of its mechanism**: "each of the easy-to-read dials subtly reveals part of its mechanism as if it were revealing the skeleton movement". "No screens apart from the minimal display in front of the driver." | Cited: [Pagani, Utopia page](https://www.pagani.com/pagani-utopia/); [Pagani press contents PDF](https://pagani.com/app/uploads/2022/09/Pagani-Utopia_Contents_ENG.pdf) |
| R2 | **The mechanism shows through small portholes**: "The analogue instruments feature small portholes through which you can view their inner workings, as you would the back of a fine Swiss watch." | Cited: [evo, R. Meaden, 2025](https://www.evo.co.uk/pagani/utopia/207610/pagani-utopia-2025-review-ps22m-huayra-successor-nails-the-hypercar-formula). Photo 1: one round porthole low in each main dial, near 6 o'clock, with a toothed wheel inside. Confirm what moves in it |
| R3 | **Binnacle: a floating crescent of machined aluminium** that moves with the steering column. "Floating crescent-shaped instrument binnacle" (evo); "brings the instrument binnacle along for the ride" (Top Gear). | Cited: evo; [Top Gear interior, O. Marriage, 2024](https://www.topgear.com/car-reviews/pagani/utopia/interior). Photo 1: the bezels and the crescent carry fine linear fluting (fly-cut ribs) in bright aluminium, with a brow over the screen |
| R4 | **Two main dials at the edges, the screen between.** "That screen ... has elbowed the rev counter and speedo to the edges"; "a single screen, elbowing its way to prominence between the two glorious main dials"; "a digital display between the analog speedometer and tachometer". | Cited: Top Gear interior; [Top Gear big read, 2024](https://www.topgear.com/car-news/big-reads/driving-a-baroque-theatre-testing-v12-manual-pagani-utopia); [CarExpert](https://www.carexpert.com.au/car-news/pagani-utopia-revealed) |
| R5 | **Rev counter on the LEFT, speedometer on the RIGHT** (the brief has them the other way). Left dial numerals run 1 to 8 with a red sector from about 7; right dial numerals rise in 20 km/h steps (60, 80, 100 at the top, 260 to 320 down the right). | Photo 1 (Utopia Roadster press photo). Unconfirmed on the coupe and on left-hand versus right-hand drive: step 1 checks photo 2 at full size |
| R6 | **Dial face**: a dark outer chapter ring carrying white numerals and ticks; inside it a large recessed centre in brushed or spun silver metal falling to a polished domed hub; a small PAGANI plaque on the upper centre; the porthole below. | Photo 1; Our reading of a small crop, confirm finish (spun, sunray or bead-blasted) at full size |
| R7 | **Needles**: slim, silver-white, long, from the domed polished hub; both rest near 7 o'clock at zero. | Photo 1. Tail length, counterweight and tip colour unconfirmed |
| R8 | **Rev counter geometry** (Our reading from photo 1): 0 near 7 o'clock, 3 at about 10, 5 just past 12, 7 near 3 o'clock, so about 30° per 1,000 rpm and about 240° for 0 to 8,000. | Computed from that reading: 6,700 rpm sits at +51° from 12; unconfirmed |
| R9 | **The screen** (portrait-ish TFT, colour): photo 1 shows date and time, tyre temperature and pressure for each corner round a car outline ("26 °C 2.25 bar"), a small bar scale "70° 100° 130°", a gear or speed figure; photo 2 shows "COMFORT", trip and consumption. Motor1: it carries navigation, tyre pressure and temperature, powertrain information and audio; Top Gear: dynamic info, mood lighting and CarPlay, run by a clickwheel. | Photo 1, 2; Cited: [Motor1 interior, A. Carmone, 2024](https://www.motor1.com/news/735286/pagani-utopia-interior-technology-analysis/); Top Gear interior |
| R10 | **Typography**: "bespoke typefaces" throughout. The dial numerals look like an upright regular-weight sans with open forms. | Cited: [Motor1 first drive, K. Kinard, 2024](https://www.motor1.com/reviews/729242/pagani-utopia-review-first-drive/). The closest font match is unconfirmed; step 1 measures stroke and width from photo 1 |
| R11 | **Night lighting**: instruments lit cool white with a lavender-blue cast; a glowing light line outlines the console pod; lit rings round the tunnel controls. Pagani: "a luminous aura that creates a pleasant and relaxing environment"; CarExpert: "illumination surrounding all the main control pods". | Photo 3, 4; Cited: Pagani PDF; CarExpert. Colour temperature is Our reading |
| R12 | **Binnacle details**: a PAGANI wordmark under the screen and a round button marked LIFTER (the nose lift) under the left dial. | Photo 1 |
| R13 | **Four more round gauges** in a row on the centre stack above a row of toggles with a red centre switch, then three climate rotaries labelled DIRECTION, TEMPERATURE and FAN SPEED. GENROQ: "the tachometer, oil temperature gauge and other meters are analogue". Which four quantities they read is unconfirmed beyond oil temperature. | Photo 2; Cited: [Motor-Fan / GENROQ, 2024](https://motor-fan.jp/article/98786/) |
| R14 | **Materials round the dials**: "Almost everything you look at is machined aluminium", 777 machined parts; parts "anodised or polished ... more akin to jewellery". | Cited: Top Gear interior |
| R15 | **Exposed gear linkage** and a red START button at the base of the stack. | Cited: Pagani page ("the gear lever mechanism is still exposed"); Motor1 first drive; Photo 4, 5 |
| R16 | **Engine facts the scale needs**: 6.0 l twin-turbo V12, 635 kW (864 PS) at 6,000 rpm, 1,100 Nm from 2,800 to 5,900 rpm, limiter 6,700 rpm, 7-speed Xtrac manual or automated manual, rear drive, top speed 217 mph. | Cited: Pagani PDF; Motor1 first drive; evo |
| R17 | **Drive modes** Comfort, Sport, Race, Wet, plus Supersoft for the dampers. | Cited: evo |

### 1.2 What no source settles (step 1 checks the photos and a video)
- Speedometer end value (20 km/h steps suggest 360 or 400 km/h; **Computed** 217 mph = 349.2 km/h) and the rev counter's red-sector start.
- What turns inside the portholes, and whether it moves with the needle (a geared train) or is decorative.
- Needle tail, counterweight and tip; whether the numerals are printed, applied or backlit; the face colour of other cars (photo 2's coupe dial faces read darker and bluer than photo 1's).
- Start-up behaviour: no source describes a needle sweep or a light-up sequence.

### 1.3 Lineage (Zonda and Huayra, both in the game)
- **Zonda C12 S and Cinque**: "a simple oval binnacle", large analogue road and engine speed dials, a fuel gauge to the left, water temperature to the right, warning lights at the right, and "a digital screen" at the base of the binnacle; natural aluminium on the C12 S, gloss black on the Cinque. Cited: [Supercar Nostalgia, C12 S](https://supercarnostalgia.com/blog/pagani-zonda-c12-s); [Cinque](https://supercarnostalgia.com/blog/pagani-zonda-cinque).
- **Huayra**: "a round tachometer and speedometer dial set on either side of a small, flat multi-function display", "other smaller dials set below", "Everything is brightly polished metal", day contrast "rather lousy", at night "a rainbow's choice of colors". Cited: [Autoblog 2014 review as reposted on ClubLexus](https://www.clublexus.com/forums/car-chat/758268-review-2014-pagani-huayra.html).
- **Our reading:** the constant Pagani grammar for 25 years is two big round dials round a small screen, satellites for fuel and temperature, everything in bright machined metal. The Utopia adds the portholes and the crescent. Building the Utopia keeps every in-game Pagani recognisable.

## 2. Reference photos

All are Pagani press images. Photos 1 to 5 were viewed; 6 to 10 are listed for the PC and step 1 (content unknown).

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Roadster driver's view: crescent binnacle, rev counter left, speedometer right, TFT with tyre data, LIFTER button, portholes | https://cdn.motor1.com/images/mgl/JOBe2K/s1/pagani-utopia-roadster-020300-volante.jpg | [Motor1 735286](https://www.motor1.com/news/735286/pagani-utopia-interior-technology-analysis/) |
| 2 | Coupe dashboard by day: screen in COMFORT, four console gauges, toggles, three climate dials | https://www.netcarshow.com/Pagani-Utopia-2023-1600-0f.jpg | [NetCarShow Utopia](https://www.netcarshow.com/pagani/2023-utopia/) |
| 3 | Coupe cabin at night: cool white dials, lit outline round the console pod | https://www.netcarshow.com/Pagani-Utopia-2023-1600-0e.jpg | same |
| 4 | Gear linkage under light, lit control rings, red START | https://www.netcarshow.com/Pagani-Utopia-2023-1600-10.jpg | same |
| 5 | Gear lever and open gate from above | https://www.netcarshow.com/Pagani-Utopia-2023-1600-11.jpg | same |
| 6 to 10 | Motor1 gallery, unviewed | https://cdn.motor1.com/images/mgl/W88O91/s1/pagani_utopia_01.jpg, .../AkkeK2/s1/pagani_utopia_04.jpg, .../g44LAg/s1/pagani_utopia_10.jpg, .../y22Mml/s1/pagani_utopia_11.jpg, .../JOOv36/s1/pagani_utopia_13.jpg | Motor1 735286 |

Video to find at step 1: Pagani's C10 film (https://www.pagani.com/app/uploads/2022/09/C10_sito_auto.mp4, not watched) and owner start-up and night videos on YouTube.

Drive `references` folder: https://drive.google.com/drive/folders/1RAwEJtOsIX4bxEe6NpNDbqlTMQFsSSHs (holds `SOURCES.md` with every URL; no image bytes saved, the PC fetches them).

## 3. How the real thing behaves

| State | What the sources say | Label |
|---|---|---|
| Start-up | One press of the red START button. No needle sweep or light-up is reported. | Cited (Motor1 first drive); sweep unconfirmed |
| Idle | Physical needles hold; the screen shows mode, trip and tyre data. | Photo 2; Our reading |
| Cruise | "Your peripheral vision flutters and dances with whirring dials"; "seeing needles and dials and levers flick about". Nothing animates on its own beyond the needles and whatever moves in the portholes. | Cited (Top Gear big read) |
| Pull to the limiter | V12 to the 6,700 rpm limiter; the rev needle enters the red sector. No shift lights exist. | Cited (Pagani PDF; evo); Photo 1 for the red |
| Gear change | Manual: the driver moves the exposed linkage; the rev needle drops and the screen's gear figure changes. Automated manual: single clutch, so a short torque interruption. | Cited (Top Gear, evo); display behaviour Our reading |
| Braking | Speed needle falls; nothing else reported. | Our reading |
| Drift, lateral G | The car slides progressively in Race mode with ESC on; no G display is reported. | Cited (evo; Motor1 first drive) |
| Reverse, neutral | A very wide-angle rear camera exists (Top Gear); where it shows and how the cluster marks R and N is unreported. | Cited camera; rest unconfirmed |
| Night | Cool white dials, a glowing outline round the control pods. | Photo 3; Cited (CarExpert) |
| Shut-down | A violin exit tune composed by the designer plays when the car is switched off. Needle return unreported. | Cited (Top Gear interior; GENROQ) |

**Motion notes (Our reading).** Real stepper-driven needles: quick, stop clean, no visible bounce. Keep the family `needle` spring (T 0.07, ζ 0.75, feed-forward) but no overshoot past a scale end. Whatever turns in the porthole should turn only when the needle moves, so idle is still.

## 4. The maths

No row in "Real geometry and maths". The modelled figures are the scale, the date discs and the torque fallback.

- **Torque fallback, Computed:** τ = P × 60 ÷ (2π n). With Pagani's figures, 635 kW at 6,000 rpm gives 1,010.6 Nm and at 5,900 rpm 1,027.8 Nm; 1,100 Nm at 5,900 rpm would need 679.6 kW. So the published plateau and peak power do not quite agree; a test may assert the formula, not the 1,100 Nm plateau.
- **Rev scale (Our reading of photo 1, to confirm):** angle = −150° + 30° × rpm/1,000 from 12 o'clock; 6,700 rpm sits at +51°, 8,000 at +90°. The dash scales to the driven car: end value = next whole thousand above `max_rpm`, red from the car's redline (fallback `max_rpm` × 0.97, family rule).
- **Grande-date discs (brief):** a ten-position units disc steps 36° per digit; the tens disc shows blank or 1. R and N are extra positions; a test asserts gear 10 shows "1" and "0".
- **Brief's bezel ratchet, Computed:** 24 detents in 600 ms is 15° and 25 ms per detent (40 detents a second). It is motion, not flashing, so the flash audit does not apply.
- **Wheel radius, Computed** (for any wheel-driven porthole idea): front 265/35 R21 r = 0.3594 m, rear 325/30 R22 r = 0.3769 m; the rear wheel turns 73.7 rad/s (11.73 rps) at 100 km/h.

## 5. Telemetry mapping

Keys are the Inputs table of `docs/TELEMETRY_USAGE_MATRIX.md`. "Verified" = long-verified against the golden packets; "Decoded" = decoded, offset confirmed at check C.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Rev needle (left dial) | `rpm`, `idle_rpm`, `max_rpm` | Verified | section 4 scale; needle spring with feed-forward |
| Red sector, shift cue | `max_rpm`, car profile | Verified | redline from the car profile, fallback limiter × 0.97 |
| Speed needle (right dial) | `speed` | Verified | km/h or mph; 20 km/h (or 10 mph) numerals; scale end per car |
| Gear (centre window) | `gear` | Verified | R in warning, N at 72% ink, 1 to 10+ |
| Speed digits | `speed` | Verified | tabular, user units |
| Porthole wheel | `rpm` or `wheel_speed` with `drivetrain` | Verified / Decoded | turns only as its needle turns (geared to the needle angle), or with the driven wheels; fallback `speed` ÷ r |
| Four small console-style dials (if the board keeps them, 5 below) | `fuel`, `tyre_temp`, `boost`, `torque` | Decoded / Verified (boost) | 0 to 1 reserve; axle-mean °C; psi or bar; torque fallback section 4 |
| Tyre corner figures (screen echo) | `tyre_temp` | Decoded | four corners, °C |
| Light over the metal, parallax | `g_lat`, `g_long` | Verified | light azimuth; window 0, dials 2, case 4 px (brief) |
| Shift strip | `rpm` + thresholds | Verified | limiter strobe, section 7 |
| Pedal fans (brief) | `throttle`, `brake` | Verified | go and warning roles |
| Bezel turn on car change (brief) | `car_ordinal` | Verified (host) | only on a new car |
| Gear-change edge | `gear`, `clutch` | Verified / Decoded | edge on gear; clutch confirms |

**What the game cannot send, and its replacement** (one recommended in each):
1. **Navigation, CarPlay, audio and mood-lighting menus on the screen.** **Bold idea (recommended): the screen's portrait aperture becomes the brief's twin-disc date window holding the gear**, framed like the TFT so the layout still reads as Utopia. Second: a small "movement" view in that aperture, a balance wheel whose beat rate follows rpm, so the centre is mechanical like the rest.
2. **Tyre pressures.** Temperatures are decoded, pressures are not. **Recommended: keep the four corners round a car outline with temperatures only**, the one screen page the game can drive truthfully. Second: per-wheel combined slip (`tyre_combined_slip`, decoded) as four small arcs.
3. **Oil and water temperatures** (the console gauges, the Zonda's satellites). **Recommended: four small dials in the console's row style reading fuel, boost, tyre temperature and torque**; for a non-turbo car the boost dial reads lateral G. Second: two satellites only (fuel left, tyre temperature right), as the Zonda lays them out.
4. **Drive mode (COMFORT, SPORT, RACE, WET).** **Recommended: a small mode word chosen by what the car is doing, labelled MODEL** (calm cruise reads COMFORT, sustained high throttle and lateral G read RACE). Second: drop it.
5. **Date, time and trip.** Dropped; the date window takes the space (the date complication is the brief's nod to it). Distance travelled (`race_progress`, decoded) can fill a trip line if the board wants one.
6. **Warning lights.** Dropped; nothing to show truthfully.

## 6. Edge cases
- **Max rpm 0 or tiny:** needle at rest, scale 0 to 8, strip off until `max_rpm` > 1,500. **Idle near max:** shift rpm at least idle + 10% of the range.
- **12,000+ rpm:** numerals in thousands to 13 or more; with 30° per 1,000 the sweep would pass 360°, so the scale compresses to the 240° arc (step 1 fixes the real arc first). **Diesels (≈4,500):** 0 to 5 with half-thousand ticks.
- **EVs:** no idle, one gear; the needle tracks motor rpm on its own scale; the date window shows the single gear and does not jump; the porthole wheel follows the road.
- **Reverse:** R in warning; the porthole wheel runs backwards if it follows the wheels. **Neutral:** N at 72% ink. **Gear above 10:** the tens disc carries it (brief).
- **Units:** km/h numerals in 20s, mph in 10s or 20s; hp or kW for the torque or power dial per user setting.
- **Limiter bounce (≈50 rpm a frame):** needle on its spring, strip thresholds with 150 rpm hysteresis; no rpm digits.
- **Crashes and huge G:** clamp light swing and parallax. **Airborne:** wheel-driven parts overspeed; cap their drawn rate below the aliasing limit (Bugatti note, section 4).
- **Stale or paused stream:** the app hides the dash. **Random palettes:** needles, lume and the shift fill stay on roles; red only for the red sector and warning.
- **Font swap:** gear and speed digits follow the user's font; dial numerals use the `dial` role on MSDF.
- **Reduced-flash mode:** the strip breathes at 2 Hz. **Colour-vision:** needle position, strip length and the window digits carry every cue.
- **Cockpit only;** no hood variant.

## 7. Signature moments

**Limiter (only the shift strip strobes, 10 Hz at 50% duty, 2 Hz breath in gentle mode).** The real car has no shift lights, so the strip is the one house addition.
- A (recommended): **a fluted aluminium rail along the binnacle brow** (R3), its lamps set in the ribs; it strobes at the limiter while the rev needle's tip and the red sector stay steady.
- B: the brief's 12 lume cartouches in a straight bridge rail joining the dial crowns.

**Gear change.**
- A (recommended): **the grande-date jump** from the brief, in the screen's aperture: the units disc jumps like a big date at midnight, the old numeral leaves upward, the new one rises from 1.12 on the fast spring. It is the date complication standing in for the dropped screen, so it is both house idea and Utopia layout. Drop the 1-frame 60% rim flash if the flash audit counts it.
- B: **the porthole wheel takes one visible step** on each shift, so the movement answers the linkage the driver just moved (R15).
- C: the bezel ratchet on a car change only (brief), kept separate from the gear change.

## 8. What it needs from the 4K library

Kit `pagani_horologerie` (PR #18 branch, `tools/asset-forge/src/prebuild/kits.rs`; `assets/clusters/pagani_horologerie/prebuild/manifest.json`):
- Swatch `clous_de_paris_p12` (Blender tile, 4096²): the brief's dial guilloché. The photos show no clous de Paris on Utopia dials (Our reading), so keep it as an optional face finish only.
- Swatch `rose_guilloche_p24` (Blender disc): secondary guilloché; same caveat.
- Swatch `black_anodised_alu` (Blender tile): the brief's case chamfer. The real binnacle is bright machined aluminium (R3, R14), so this suits a dark variant, not the default.
- Sprite `polished_steel_matcap` (1012 px, three G keys): the needles and the polished domed hubs (R6, R7). Fits the real car directly.

Shared parts it can reuse: `spun_aluminium_chapter` (T50Floodlit) for the spun silver dial centres (R6); `brushed_steel_linear` (BugattiTourbillon) for linear brushing; `chrome_matcap` (KineticLiquidMetal) as a brighter hub option.

LIBRARY GAP: fluted machined aluminium (fly-cut parallel ribs, about 6 to 8 px pitch at 1:1, with anisotropic highlight) swatch, 4096² seamless with normal and roughness, for the crescent binnacle, the bezels and the recommended shift rail (R3); Blender, asset-forge swatch.
LIBRARY GAP: porthole movement vignette: a jewelled toothed wheel with a bridge and screws seen through a Ø120 px aperture, sprite at 1:1 with rest and two G-swing keys plus a separate wheel layer that can rotate, for R2; Blender (or the parametric watch-wheel recipe the Bugatti note asked for).
LIBRARY GAP: spun or sunray conical dial-centre normal map (a recessed cone falling to the hub), 2048² disc with mips, so the silver centre reads as a recess under the moving light (R6); computed (exact analytic cone) in asset-forge.

## 9. Departures and open questions for the board
1. **Dial sides.** The brief puts speed left and tach right; photo 1 shows the tach left and the speedometer right. For a recreation the car wins: swap them, after step 1 confirms on the coupe.
2. **Case.** The brief's tonneau case drawn as a 1 px chamfer around black vs the real crescent of fluted bright aluminium. Keep the house rule (no plate, no opaque fill) but draw the crescent's outline and fluting as light-raked edges, not a black tonneau.
3. **Dial faces.** The brief's clous-de-Paris faces vs the real dark chapter ring, recessed silver centre and porthole. Recommended: the real face; guilloché is Bugatti's listed finish and would blur the two dashes.
4. **Hands.** The brief's openworked primary-colour hands vs slim silver-white needles on domed polished hubs. Recommended: real needles, colour user-changeable (below).
5. **Centre.** The brief's date window replaces the real TFT (whose content the game cannot drive); keep it, framed in the TFT's portrait proportions.
6. **Retrograde pedal fans and the rotating bezel** are inventions with no counterpart. Recommended: keep the pedal fans small or replace them with the four console-style dials (5, item 3); keep the bezel ratchet only on a car change, or print the scale per car without turning it.
7. **Crest.** Brief: crest under the tach's 6. Real: PAGANI plaque in the dial centre and wordmark under the screen. The logo zone shows the driven car's logo there instead.
8. **Typeface.** Real "bespoke typefaces"; the brief's Saira Condensed and Barlow Semi Condensed are defaults to compare with photo 1.
9. **Lighting.** Default backlight cool white with a slight lavender cast (R11), not the family orange lume.
10. **Distinct from siblings.** `BugattiTourbillon` is three dials, skeletonised bridges, jewels and concentric hands; `KoenigseggGyro` a rotating ring; `RattrapanteSector` a split-hand fan; `AmgMbuxClassic` two tube dials on a screen. This dash is two bright-metal dials in a crescent, a central date window and portholes.

**Customisable without losing the realism** (TJS, 10:55: "you can change for example needle color etc"):

| Part | Maps to | Period-correct default |
|---|---|---|
| Needle colour | palette `primary` (needle role) | silver-white |
| Chapter ring and numeral colour | palette `ink` / face roles | dark ring, white numerals |
| Backlight colour and brightness | palette `secondary` plus `AppConfig` brightness | cool white, slight lavender |
| Numeral font | font role `dial` (and `display` for gear and speed) | upright regular sans |
| Bezel finish | `config.extra` finish: fluted bright, black anodised, polished | fluted bright aluminium |
| Dial-centre finish | `config.extra`: spun silver, clous de Paris, rose guilloché | spun silver |
| Porthole on or off | `config.extra` | on |
| Speed units, power units | `AppConfig` | user setting |

## 10. Sources

**Pagani**
- https://www.pagani.com/pagani-utopia/
- https://pagani.com/app/uploads/2022/09/Pagani-Utopia_Contents_ENG.pdf
- https://www.pagani.com/press/pagani-utopia-unveiled-at-the-national-science-technology-museum-in-milan-surrounded-by-original-drawings-by-leonardo-da-vinci/

**Reviews and press**
- https://www.topgear.com/car-reviews/pagani/utopia/interior (O. Marriage, 2024)
- https://www.topgear.com/car-news/big-reads/driving-a-baroque-theatre-testing-v12-manual-pagani-utopia (O. Marriage, 2024)
- https://www.evo.co.uk/pagani/utopia/207610/pagani-utopia-2025-review-ps22m-huayra-successor-nails-the-hypercar-formula (R. Meaden, 2025)
- https://www.motor1.com/news/735286/pagani-utopia-interior-technology-analysis/ (A. Carmone, 2024)
- https://www.motor1.com/reviews/729242/pagani-utopia-review-first-drive/ (K. Kinard, 2024)
- https://www.carexpert.com.au/car-news/pagani-utopia-revealed (D. Fung, 2022)
- https://www.hagerty.com/media/news/paganis-v-12-utopia-a-steampunk-dream-with-a-gated-manual/ (G. Jarvis, 2022)
- https://motor-fan.jp/article/98786/ (GENROQ, 2024)
- https://www.guideautoweb.com/en/articles/71581/we-went-inside-the-pagani-utopia-at-monterey-car-week/ (2023)

**Lineage**
- https://supercarnostalgia.com/blog/pagani-zonda-c12-s
- https://supercarnostalgia.com/blog/pagani-zonda-cinque
- https://www.clublexus.com/forums/car-chat/758268-review-2014-pagani-huayra.html (Autoblog review repost)
- https://www.gearpatrol.com/cars/a424228/15-greatest-automotive-instrument-clusters-of-all-time/

**Photos**: section 2 (Motor1 CDN and NetCarShow, Pagani press images).

**Repo**: `program/phases/07-cluster-factory/CATALOG.md` (brief, category, flash rules); `docs/design/families/classic-analog.md`; `docs/TELEMETRY_USAGE_MATRIX.md`; `docs/design/RESEARCH.md` §4; `crates/fh6-clusters/src/clusters/bugatti_tourbillon/RESEARCH.md`; PR #18 branch `origin/claude/project-thread-ib67x9` (`PREBUILD.md`, `kits.rs`, the kit manifest); `assets/car_brands/car_brands.json`.
