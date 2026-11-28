# Skyline R34 MFD: research note

Written 2026-11-28 by the SkylineR34Mfd dash thread (build #12), before the Penpot board. **Cited** means read from the named source that day. **Recalled** means from memory, not yet confirmed. **Our reading** is a conclusion drawn for this dash. Nothing here is a layout: the design is made on the Penpot board `07 · SkylineR34Mfd`.

The game car is the 2000 Nissan Skyline GT-R V-Spec II (BNR34). Target `inspired`: the MFD and the meters keep their named parts, and every departure is listed in section 6.

## 1. Sources read

| Key | Source | Date read |
|---|---|---|
| JAL | Jalopnik, "The R34 GT-R's Multifunction Display Was Secretly Its Coolest Feature" (2020-11-16) https://jalopnik.com/the-r34-gt-rs-multifunction-display-was-secretly-its-co-1845687935 | 2026-11-28 |
| EVO | evo, "Nissan R34 Skyline GT-R multifunction display: Art of Speed" https://cdn2.evo.co.uk/nissan/gt-r/203810/nissan-r34-skyline-gt-r-multifunction-display-art-of-speed | 2026-11-28 (web check) |
| TR1 | Toprank, "How to use the R34 Nissan Skyline GT-R MFD: Part 1" (2020-11-27) https://toprankglobal.jp/news/92 | 2026-11-28 |
| TR2 | Toprank news 60 (2019-10-05), MFD screen failure and repair https://toprankglobal.jp/news/60 | 2026-11-28 |
| GUS | GTR USA blog, "R34 MFD self-diagnosis" (2023-05) https://www.gtrusablog.com/2023/05/r34-mfd-multi-function-display-self.html | 2026-11-28 |
| SAU | SAU forums: 223920 (MFD info, page 2), 429651 (V-spec MFD lost intake and exhaust temperatures), 41240 (MFD unit) https://sau.com.au/forums/topic/223920-r34-gtr-mfd-info/page/2 | 2026-11-28 |
| PMF | pedromiguelferreira.github.io, "Nissan Skyline R34 GT-R" https://pedromiguelferreira.github.io/nissan-skyline-r34-gt-r | 2026-11-28 |
| SCN | Supercar Nostalgia, "Nissan R34 Skyline GT-R: a historical and technical appraisal" (2021-06-14) https://supercarnostalgia.com/blog/nissan-r34-skyline-gt-r | 2026-11-28 |
| SPH | Speedhunters, R34 MFD screen replacement https://www.speedhunters.com/?p=385340 | 2026-11-28 |
| TG | Top Gear, Skyline R34 interior (2019-04-19) https://www.topgear.com/car-reviews/nissan/skyline-r34/interior | 2026-11-28 |
| PH / GIZ | PistonHeads (2007-10-19) and Gizmodo (2007-10-23) on the R35's display | 2026-11-28 |
| NZP | NizmoPartsPlug, R34 V-Spec II Nür 300 km/h speedometer (seller page) | 2026-11-28 |
| NIS | Nismo reproduction meters: Philkotse (2022-08-25), The Drive (2022-08-31), Japanese Nostalgic Car, paultan.org | 2026-11-28 |
| JBS | jbskyline.net, BNR34 engine figures https://www.jbskyline.net/r34/gtr/index2.php | 2026-11-28 (web check) |
| WEB | The catalog's web check, `/mnt/project-files/dash-research/webcheck/SkylineR34Mfd.md`, and the planning notes `nismo-skyline-clusters.json` | 2026-11-28 |

Photos: 34 reference photos were downloaded on the person's PC on 2026-11-28 (the container's proxy refuses these image hosts) into the Drive masters folder `skyline_r34_mfd\references\`, with `VISUAL_NOTES.md` (sampled colours) beside them. Small crops are in the project files at `skyline_r34_mfd/references/`. Section 8 is drawn from them; **Seen** marks a fact read off a photo.

## 2. The MFD

| # | Fact | Source |
|---|---|---|
| M1 | A 5.8-inch full-colour LCD mounted on top of the centre of the dashboard, "crowning the dashboard". | Cited: JAL, EVO |
| M2 | Controls: Display, Return, Menu and Mode buttons and a four-way joystick with a centre select. | Cited: JAL, TR1, PMF |
| M3 | Three information displays, stepped with MODE; MENU then SELECT chooses what each shows; DISP sets the display. | Cited: TR1 |
| M4 | The main page shows up to seven readouts as bar meters with their numbers. | Cited: JAL |
| M5 | A single-readout page draws a line graph of the previous 30 seconds. | Cited: JAL; EVO ("30-second increments") |
| M6 | A separate dial shows the front and rear power split (the ATTESA E-TS torque split). | Cited: JAL ("a dial showing how much power goes to the front wheels"); EVO ("torque split") |
| M7 | Readouts: boost, throttle opening, injector duty, oil temperature, water temperature; the V-spec adds intake and exhaust (EGT) temperatures. | Cited: SAU 223920 ("5 rows" on the standard car), JAL, EVO. Conflict: PMF says seven on the standard car plus two on the V-spec; SAU 429651 says both have the temperature rows and need a sensor. **Our reading:** the V-Spec II in the game has seven rows. |
| M8 | RED ZONE mode: the driver sets a threshold for boost, injector, oil temperature, water temperature and (V-spec) exhaust temperature; past it the bar turns red. It only monitors. | Cited: TR1, JAL |
| M9 | SHIFT UP mode: "the red LED of shift indicator" in the MFD housing flashes at an rpm the driver sets with the joystick (Toprank's example: 5,000 rpm). | Cited: TR1, JAL |
| M10 | Start-up: the GT-R logo shows at ignition-on and the readouts appear when it clears. A failing screen shows the logo "like burning". | Cited: TR1, TR2 |
| M11 | Boost scale: 1.2 bar on the stock unit, 2.0 bar on the Nismo V2 unit. Temperatures in °C; self-diagnosis minimums "Exhaust is 400c and Intake will be 5c". | Cited: PMF, GUS, SAU 429651 |
| M12 | Some readings show no number until the engine reaches temperature. | Cited: WEB (Toprank) |
| M13 | Data could be offloaded over a serial cable to a laptop; evo says the extremes of a drive were stored. | Cited: JAL, EVO. GUS ties data logging to the Nismo unit. |
| M14 | Nismo's final MFD version adds a lap timer and G-sensors (and an oil-temperature scale of 70 to 150 °C). | Cited: EVO, GUS |
| M15 | Hardware: Sharp screen on early cars, Toshiba from Series 2; early ("zenki") panels have a satin finish and get red spots at the edges; screen deterioration is a very common fault. | Cited: SCN, SPH, TR1 |
| M16 | A TV button exists; on a car without the tuner it shows a no-tuner message. | Cited: SPH |
| M17 | Top Gear (2019): "Nowadays, it looks like a cheap computer game." | Cited: TG |
| M18 | Polyphony Digital designed the R35's display (2007), not the R34's. The maker of the R34 screens' graphics is not established. | Cited: PH, GIZ; WEB |

The MFD's colours, type and page layouts were not in any text source; they are in section 8, seen on photos.

## 3. The stock meters

| # | Fact | Source |
|---|---|---|
| C1 | "A combined 10,000rpm rev counter / oil pressure gauge was flanked by a 180mph / 300kph speedometer. Smaller read outs for water temperature and fuel were located in each corner." The binnacle surround is brushed aluminium to match the centre console. | Cited: SCN |
| C2 | The Nür and M-Spec Nür speedometer reads to 300 km/h; the seller calls the standard JDM one 180 km/h. | Cited: NZP (seller page). Conflict with C1's 300 km/h for the standard car; **Our reading:** 180 km/h on the JDM standard car, 300 on the Nür. |
| C3 | The engine's redline is 8,000 rpm; 280 PS at 6,800 rpm. | Cited: JBS |
| C4 | Black faces, white italic numerals and ticks, orange-red needles (about #E0552E), black hub caps; the tachometer's ticks turn red-orange from 8 with a red band 8 to 10 on the outer edge. | Seen (section 8.10) |
| C5 | No boost gauge in the stock cluster: four dials, small water temperature left, tachometer centre-left, speedometer (0 to 180 km/h) centre-right, small fuel right, on a silver panel. Boost lives on the MFD. | Seen (section 8.10); corrects the recalled layout in the first brief |
| C6 | Nismo reproduction meters (2022): BNR34 black face with orange numerals and orange lighting, or white face with black numerals and orange lighting; 320 km/h speedometer and 11,000 rpm tachometer in two stages. The single-turbo R34's version is matte black to 300 km/h and 10,000 rpm. | Cited: NIS |

**Our reading:** the dial scale follows `EngineMaxRpm` (house rule), and the stock 10,000 rpm face with its oil-pressure sub-dial is the period reference. The Nismo orange lighting is a strong second palette ("Nismo night") for the palette roles.

## 4. What the game sends

From `docs/TELEMETRY_USAGE_MATRIX.md`: rpm, max and idle rpm, gear, speed, throttle, brake, clutch, handbrake, steer, G on three axes, velocity, yaw rate, orientation, tyre slip angle ×4, suspension travel ×4 (normalised and metres), car ordinal, drivetrain, power and boost. There is no fuel, no temperature, no oil pressure, no torque per axle and no lap data.

## 5. Dropped parts and the ideas that replace them

Each real part the telemetry cannot drive is dropped (never drawn with fake data) and its space goes to one of these, all driven by the game. These are ideas for the board, not a layout; the board picks.

| Real part (source) | Why dropped | Ideas for its space (telemetry) |
|---|---|---|
| Front torque split dial (M6) | No torque per axle | **ATTESA E-TS, modelled.** The real system sends torque forward (0:100 up to 50:50) as the rear wheels slip more than the fronts. A small model fed by rear-minus-front slip angle, longitudinal G and throttle draws a split dial that moves like the real one in a slide. Labelled as modelled in the README. A row in "Real geometry and maths" if the board keeps it. |
| Injector duty row (M7) | Not sent | **Load:** power as a share of the highest power seen in this car this session, the closest honest stand-in; or clutch, the R34's manual-gearbox signature. |
| Oil and water temperature rows (M7) | Not sent | **Chassis page:** four suspension-travel bars in the MFD's own bar style (front left to rear right), with red zones near bump stop; or tyre slip bars per corner. |
| Intake and exhaust temperatures, V-spec (M7) | Not sent | **Nismo G page (M14):** a friction circle of lateral and longitudinal G with a fading trace, the Nismo MFD's own addition. |
| Oil pressure sub-dial in the tachometer (C1) | Not sent | **Boost sub-dial** in the same place, to 1.2 bar like the stock MFD (M11), with its red zone; or a throttle and brake pair. |
| Water temperature and fuel corner gauges (C1) | Not sent | **Steer and yaw:** a small steering angle and yaw-rate pair; or a peak-hold boost and speed memory, as the MFD stored a drive's extremes (M13). |
| Warm-up blanking (M12) | No temperatures | After `race_on`, each readout shows "- - -" until its first real reading arrives, then fills in turn: the real car's warm-up behaviour, driven by the race start. |
| TV, navigation (M16) | Not sent | Nothing in the tiles. Space goes to the friction circle or the chassis page. |
| Single red shift LED (M9) | Replaced, house rule | The catalog's 12-lamp row (a departure); the real red LED can be kept as its final, centre lamp. |
| Lap timer (M14) | No lap data | Dropped; the line graph's 30 s history carries the "drive record" idea. |

Ideas for the empty space round the tiles, from the car: a Wangan or Bayshore night horizon behind the tiles that brightens with speed; the line graph's boost history drawn as a mountain ridge (the "Godzilla" spine) that scrolls; the GT-R boot logo at the race start (M10) as the signature start-up, burning in and clearing as the readouts appear.

## 6. Departures from the real car (for the handover)

- The gear window: the real car has no gear readout (WEB).
- The 12-lamp row in place of one red shift LED in the MFD housing (M9).
- The MFD leaves the dash top and its pages float round the tachometer as tiles (the brief's twist).
- Every dropped readout in section 5 and its replacement.
- The tachometer scale follows `EngineMaxRpm`, not a fixed 10,000 rpm.
- Red zones use the `warning` role, not the MFD's own red, so the dial's red needle and band stay inside the red budget.
- The line graph keeps the real 30 s (M5); the brief's 600 samples means 20 Hz.

## 7. Still needed

- A photo of the F-TORQUE gauge page itself (only its SELECT tile is seen). The board designs the modelled ATTESA dial in the MFD's own dial style (8.4).
- Speedhunters' MFD photos (the PC could not reach speedhunters.com).
- The BNR34 owner's manual pages for the MFD and the meters (the Scribd copy "BNR34 Skyline GTR Multi Function Display", 5 pages, would not render as text).

## 8. Seen on the photos (2026-11-28)

From the photos in the Drive folder; colours were sampled from photos of an LCD and are approximate. The full notes, with every sampled value, are `VISUAL_NOTES.md` in that folder.

**8.1 Panel.** Sharp 5.8-inch, 400 × 234 px (caption of a replacement-screen kit). Soft, anti-aliased low-resolution graphics. Background very dark navy-black (#050507 on a straight close-up of the original screen; #18191A to #2A2934 in room light). Fine diagonal streaks on every photo, most likely screen wear.

**8.2 Type and chrome.** Bold italic wide sans throughout (Eurostile / Helvetica Bold Oblique character); captions in caps; numerals strongly slanted with smaller units (kg/cm², mmHg, %, ℃); missing data "- - -". Thin white 1 px cell outlines. Selection: a gold rounded outline (#E3C69C, up to #FFEF99). Buttons and tiles: light cyan rounded rectangles with a vertical gradient (#C7E9FE to #CBF3FD), white bold italic text.

**8.3 Seven-row page.** Rows BOOST, THROTTLE, INJECTOR, OIL-TEMP, W-TEMP, EXH-TEMP, INT-TEMP (5 rows without the V-spec's last two), each a white-outlined box, pitch 0.13 of the screen height. Columns: caption 0.03 to 0.24, a white line pictogram 0.24 to 0.36 (turbo snail, throttle butterfly, injector spray, oil can and thermometer, coolant, exhaust, intercooler), bar 0.36 to 0.69, value 0.69 to 0.96 right-aligned. Bars: flat solid green fill from the left (about #30D072, highlight #52E78B), no segments or ticks; the BOOST bar has a white zero line a quarter in, vacuum left of it.

**8.4 Round dials (twin page and history page).** 270° sweep from 6 o'clock clockwise to 3 o'clock; the lower-right quadrant is cut from the ring and holds the sensor pictogram and a quarter-sector PEAK box (teal #1DA1AD to navy #105687 gradient, peak value in white italic above "PEAK"). Ring: thick bevelled grey band (#B9B8B5, highlight #E9EBEA) with white block ticks, red ticks in the red zone. Hub: a large flat pale-grey disc, a third of the radius. Needle: thin salmon-red line (#E48670 to #F08F7C); the boost dial also shows a white second needle. A flat green "locus" pie slice (#30D072) covers the span the value has swept, switchable on the DISPLAY page (ゾーンOFF). Boost scale −6 to −2 ×100 mmHg, then 0 to 1.2 kg/cm², 0 at 9 o'clock.

**8.5 History page.** Left half a line graph: y-axis 70 to 150 (oil) with a tick at each label, 9 horizontal and 4 vertical thin grid lines, x labels "30 20 10 0 sec", time running right to left (now at the right); a thin green trace. Right half one round dial as 8.4.

**8.6 RED ZONE page.** Five rows: a cyan caption button with its pictogram, an empty white-outlined bar with a red threshold marker, the threshold value ("1.20kg/cm²", "100.0%", "130℃", "119℃", "970℃"); END and RESET buttons.

**8.7 SHIFT UP page.** A ring dial without ticks, 5 to 10 ×1000 r/min over 270°, a green sector up to the set point (about 8,000 in the photo); a cyan pill with ▲ 回転数 ▼ and lower-case "up" and "down". The housing's REV. LED (red) is the shift-up light.

**8.8 MENU and SELECT.** MENU: three cyan tiles SELECT, RED ZONE, SHIFT UP (the SHIFT UP tile shows the gate "1 3 5 / 2 4 6 R"). SELECT: a 4 × 2 grid of cyan tiles with grey bevelled circle pictograms: BOOST, OIL-TEMP, F-TORQUE, VOLT / THROTTLE, INJECTOR, EXH-TEMP, INT-TEMP. Twin pages pair two of them (boost + oil temp; F-torque + oil temp; injector + exhaust temp). VOLT confirms the battery-voltage reading the web check left open.

**8.9 Start logo and housing.** The GT-R emblem centred on dark navy: silver-lavender "GT" over a larger deep-red "R" (#71252A) with a light outline. (An orange version on one photo is a failing screen, not a design state.) The pod is charcoal (#1B1C20): screen in the left 70% behind a deep bezel step; on the right the "REV." LED, a TV button, "PUSH ENTER/RESET" and a chrome-ringed joystick; DISP, RETURN, MENU and MODE rocker buttons below.

**8.10 Stock meters.** Four dials on a silver panel: small water temperature (left), tachometer 0 to 10 ×1000 r/min with red-orange ticks and a red outer band from 8 and an oil-pressure sub-dial (0 4 8 kg/cm²) in the bottom of its face, speedometer 0 to 180 km/h with an LCD odometer window, small fuel (right). Matte black faces, white italic numerals, orange-red needles, black hub caps.

**8.11 Nismo meters.** Same layout; tachometer 0 to 11 with a red band from about 9.2; speedometer 0 to 320 km/h; "nismo" wordmark over the tach hub and the red GT-R "R" over the speedo hub. Black face with yellow numerals (#FEFB60), or white face with black upright numerals and red needles.

**Our reading for the board.** The MFD's language (navy-black glass, white hairline cells, green flat fills, bevelled grey rings with a cut-away PEAK quadrant, salmon needles, cyan tiles with gold selection, bold italic type) is distinctive and owner-recognisable; the tiles take it directly, redrawn crisp at 4K with a faint 400 × 234 pixel grid as texture. The tachometer takes the stock face (black, white italic numerals, orange-red needle, oil-pressure sub-dial position reused for boost). The PEAK box is the MFD's own "stored extremes" and becomes the peak tick of the signature moment. The MFD's red zone ticks give the house `warning` its place.

## 9. ATTESA E-TS Pro, for the modelled torque-split idea (2026-11-28)

Sources: Wikipedia "ATTESA"; DSPORT, "ATTESA Explained" pages 2 and 3 (2018); Haltech, "Nissan ATTESA Control"; Full-Race ETS-Pro install guide. None is a Nissan service manual; no OEM control law, response time or MFD gauge scale is published in them.

| # | Fact | Source |
|---|---|---|
| A1 | Front share runs from 0:100 under acceleration to at most 50:50. | Cited: Wikipedia, Haltech |
| A2 | R33 and R34 hold a base pressure of about 25 to 30 psi through a failsafe solenoid, "about 10-percent" front bias, which also lets the clutch reach peak pressure faster. | Cited: DSPORT p.2 |
| A3 | Mechanism: a chain-driven multi-plate wet clutch in the transfer case, ATF at 0 to about 300 psi from an on/off pump, a PWM duty solenoid at 200 Hz; front torque follows actuator pressure. | Cited: Wikipedia, DSPORT p.3, Haltech |
| A4 | Inputs: wheel speeds from the ABS sensors (front torque when a rear wheel turns 5% or more faster than the fronts), a three-axis G sensor under the console (lateral and longitudinal), throttle, engine speed, reverse and brake. More wheelspin means more front torque. | Cited: Wikipedia, DSPORT p.2 |
| A5 | The harder the car is cornered, the less the front wheels are engaged (factory oversteer bias); the G sensors reduce output in braking and turning; the handbrake sets the target to zero. | Cited: Wikipedia; Full-Race (aftermarket Auto mode); Haltech |
| A6 | The R34 samples the rear wheel speeds 1,000 times a second (R33: 100). | Cited: Wikipedia, DSPORT p.2 |
| A7 | Tuner controllers structure it as a base duty table plus corrections from wheel slip, G and steering angle (8 × 8 tables). | Cited: Haltech |

**Our reading, a model the game can drive.** The game sends no wheel speeds, only slip angles per tyre, G, throttle, brake, handbrake and speed (section 4). A faithful, honest model: front share `f = clamp(0.10 + k_s·max(0, s_r − s_f) + k_a·max(0, a_long)·throttle − k_y·|a_lat|, 0, 0.5)`, forced to 0 with the handbrake, where `s_r` and `s_f` are the mean rear and front slip angles (rear stepping out is the R34's typical cue for torque forward), then a first-order lag to stand in for the clutch's pressure ramp. The gains are ours (no OEM values exist) and are listed as such; the model, the 0.10 floor, the 0.5 ceiling, the handbrake rule and the lateral-G reduction come from A1, A2, A4 and A5, and unit tests can assert them without a GPU. Display as "F-TORQUE" in the MFD's own dial style (8.4), 0 to 50 %, since no photo shows the real page.
