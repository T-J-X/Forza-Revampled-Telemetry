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

No photo has been seen yet: the container's proxy refuses every image host these pages use (section 7).

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

**Not established by any text source:** the MFD's background colour, bar and graph colours, digit style and the exact page layouts. Recalled (to confirm from photos): a dark navy-to-black background, thin horizontal bar meters with white numerals and small captions, and a red end on a bar past its red zone. No tile is drawn until a photo confirms this (catalog risk).

## 3. The stock meters

| # | Fact | Source |
|---|---|---|
| C1 | "A combined 10,000rpm rev counter / oil pressure gauge was flanked by a 180mph / 300kph speedometer. Smaller read outs for water temperature and fuel were located in each corner." The binnacle surround is brushed aluminium to match the centre console. | Cited: SCN |
| C2 | The Nür and M-Spec Nür speedometer reads to 300 km/h; the seller calls the standard JDM one 180 km/h. | Cited: NZP (seller page). Conflict with C1's 300 km/h for the standard car; **Our reading:** 180 km/h on the JDM standard car, 300 on the Nür. |
| C3 | The engine's redline is 8,000 rpm; 280 PS at 6,800 rpm. | Cited: JBS |
| C4 | The tachometer's red zone start, the face, numeral and needle colours and the night lighting of the stock meters. | Not established. Recalled: black faces, white numerals, red needles. |
| C5 | A boost gauge in the stock cluster. | Not established (SAU 41240 is unclear). Recalled: a boost gauge to the right of the tachometer. |
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

- Photos of the real MFD pages (bars, line graph, torque split, red zone, boot logo) and of the stock meters at day and night. The container's proxy refuses toprankglobal.jp, blogger, github.io, thedrive, wikimedia and imgur. Candidate pictures: Toprank `picture/documents/images/4-3.jpg` (ignition screen), `15-1.jpg` and `IMG_0019` to `IMG_0047`; GTR USA "R34 Vspec screen" and "R34 MFD normal to Vspec"; PMF gallery images 37 to 42 (42 is the tachometer); The Drive's Nismo meter photos; Supercar Nostalgia's interior photo. Route: the PC's built-in browser and a canvas crop (as Gt7HoodPro did), asked of the coordinator on 2026-11-28.
- The BNR34 owner's manual pages for the MFD and the meters (the Scribd copy "BNR34 Skyline GTR Multi Function Display", 5 pages, would not render as text).
