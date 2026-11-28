# Gt7ModularTripod: research note

Build #40, catalog row 14, category Motorsport, Target `recreation`, family Race Dash, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on 2026-11-28 from the source linked. **Computed** means worked out here from cited or measured inputs, with the numbers shown. **Our reading** is a conclusion or proposal for this dash, open to the direction check. Measurements were taken in the PC's browser from the GT7 manual's 1920 x 1080 Normal View JPEG shown at natural size, where one screenshot pixel was 1.28 picture pixels, so 2.56 px at 4K; every size carries about ±5 px of error at 4K and hues are JPEG estimates. The MFD page pictures are 1920 x 800 crops shown about 1.43 times larger than the Normal View, so only their proportions are used.

**The three GT7 dashes, kept apart.** `Gt7HoodPro` (finished) is GT7's compact Bonnet View band: one low row, straight 95-LED bar, no dials, no MFD. `Gt7CockpitPro` (pre-researched) re-composes that band round a tall arch for the cockpit and leaves out "the round dials and paged MFD", which "belong to `Gt7ModularTripod`" (its section 1.3, D3). This dash is the third and largest: **the full meter GT7 draws only in its Normal View**, two detached round dials either side of the trapezoid pod, with the D-pad-paged Multi-Function Display under the pod. The local reference `mockups/references/gt7_multifunction_meter.webp` does not exist in this container (searched); the manual's own pictures stand in for it and match the catalog's description point for point.

## 1. The real display: feature inventory

**Cited**, for the views: GT7 shows "all the information that can be shown in Normal view" in its race screen picture, and "the details shown will vary depending on the current view" ([The Race Screen](https://www.gran-turismo.com/gb/gt7/manual/race/02)). The Cockpit, Bonnet and Chase pictures show only the compact band ([Racing Views](https://www.gran-turismo.com/gb/gt7/manual/race/03); `gt7_cockpit_pro/RESEARCH.md` 1.1). So this meter is GT7's Normal View meter.

Positions are at 4K relative to the screen's centre line (the pod's divider), y down from the top of the pod. Pictures are numbered as in section 2.

### Geometry and layout
1. **Two detached round dials, no panel behind them.** Speedometer left, tachometer right, the scene visible through both (pictures 1 to 6). **Computed** (picture 2): outer diameter about 190 screenshot px, so **486 px at 4K**; centres at x **-632** and **+635** (symmetric within error, call it ±634); centres **351 px below the pod's top edge**.
2. **Sweep and gap.** **Computed** (picture 2): the "0" and the end numeral sit 42° either side of straight down (dx 57, dy 63 screenshot px from the centre), so the open gap at the bottom is **84°** and the sweep **276°**, starting lower left and running clockwise.
3. **The dials have no needle: the scale ring fills.** **Our reading from pictures 2 and 3** (step 1 confirms with a video): each dial's outer ring is a dark translucent band, and a thick white band fills it from zero to the current value. In picture 2 the speed fill ends at 10:30 for 67 mph on a 0 to 200 scale (67/200 x 276° = 92° from the start, which is the 10:30 position, **Computed**); in picture 3 the speed fill stops at 128 of 280 km/h and the dark band continues to 280. This corrects the catalog's "thin white scale": the scale is thin, the fill is a thick band about 15 px at 4K over a track about 5 px (**Computed**, 6 and 2 screenshot px).
4. **Speedometer face.** Light numerals inside the ring: picture 2 (a Gr.3 car, mph) reads 0 to 200 every 20 (11 numerals, 27.6° apart); pictures 1 and 3 (km/h) read 0 to 280 every 40 (8 numerals, 39.4° apart); minor ticks between. Unit label (`mph` or `km/h`) above centre. **Cited:** the packet carries `CalculatedMaxSpeed`, "possible max speed achievable using the current transmission settings" ([PDTools `SimulatorPacket.cs`](https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs)); **Our reading:** GT7 sets each car's speedo end from it, which is why the two cars differ.
5. **Inset fuel semicircle and odometer box** in the speedometer's lower half: a white semicircle E to F with the pump icon at its centre, and under it a dark rounded box with six digits and a tenth in light figures (`082,882.3`, `000007.5`). A circled handbrake icon sits right of the dial's centre, grey when off. **Cited:** "The fuel meter is displayed for races with fuel consumption activated. The odometer shows how far you've driven in each car. The handbrake indicator lights up when the handbrake is in use."
6. **Tachometer face.** Numerals every 1000 rpm with `x1000rpm` (0 to 8 in pictures 1, 3, 4; 0 to 10 in picture 2), a **red band on the outer ring from the red zone to the end** (9 to 10 in picture 2, 7 to 8 in pictures 1 and 5), a headlight icon left of centre. **Cited:** "The light indicator lights up when the car's lights are on. The boost gauge displays the boost pressure for turbo or superchargers."
7. **Inset boost sub-gauge** in the tach's lower half: a small ring -1, 0, 1, 2 with `x100 kPa` and a turbo icon (pictures 1, 3, 4). It is **absent** in picture 2 (an unboosted Gr.3 car). **Cited:** the packet flag `HasTurbo` and "TurboBoost ... 2.0 = 1 x 100kPa" (PDTools). **Our reading:** the sub-gauge shows only on boosted cars.
8. **The trapezoid pod.** **Computed** (picture 2): **806 px** wide at the top, **172 px** tall, short vertical sides, then chamfers to a bottom edge about **678 px** wide; translucent mid-grey. Inside each end, faint concentric half-rings are the **blindside indicators**, which light red on the side a car is close (Cited, item 14; picture 5 shows the left one lit).
9. **The tick strip** sits on the pod's top edge as a gentle arc of about 100 dense vertical ticks (`gt7_cockpit_pro/RESEARCH.md` K4: chord 803, sagitta 22, radius 3711). Unlit ticks dark grey.
10. **Speed | gear block** in the pod: thin squared speed digits right-aligned to a thin divider on the centre line, unit under them; bold squared gear right of it. **Computed** (picture 2): speed cap **46 px**, gear cap **82 px** (the hood note measured 54 and 80 in the Bonnet View: the same within error, so GT7 keeps the readout size across views).
11. **Suggested-gear box** right of the gear with the transmission tag (`AT`, `MT`) under it: **87 x 72 px** (Computed). Red with a white digit when a gear is suggested (pictures 2, 4, 6); **a dark empty slot when none is** (picture 3, new: the slot never disappears, it goes dark).
12. **Steering dot** above the strip's centre (red), with a small grey centre mark (Cited, item 13; picture 2).
13. **Faint grey arrows** just outside the pod's lower corners, pointing outwards (pictures 2 to 6). **Unconfirmed:** the manual does not name them; step 1 checks a video (they may belong to the blindside indicator).
14. **The MFD** under the pod: a title in light caps (`Track Map`, `TCS`, `Radar`, `Session Best`) between two round dark buttons with white ◀ ▶ triangles; buttons **558 px** apart and the title **225 px** below the pod's top (Computed); a translucent dark field below with the page content; a row of page dots at the bottom, pitch **69 px**, the current dot white and the rest dark grey (Computed, picture 2). **Cited:** "You can switch between the different features by pressing the left and right directional buttons", and up and down adjust the value on TCS, torque distribution, brake balance and fuel map ([The Multi-Function Display](https://www.gran-turismo.com/gb/gt7/manual/race/04)).
15. **The MFD pages and their own accent colours.** Cited pages: TCS, Front/Rear Torque Distribution, Brake Balance (needs a brake controller), Fuel Map (with laps-left estimate), Track Map, Radar, Weather Radar, Session Best / Car Record. The page count changes with the car: five dots in pictures 1 and 3, six in 2, eight in 4 (Computed by counting). **New:** each adjustable page has its own accent: TCS a **blue** arc LOW to HIGH with a big squared level digit and ▲ ▼ buttons (picture 3); torque distribution a **yellow** arc with `35:65` FRONT / REAR (picture 4); Radar is white rings and a crosshair with cars as pills, the player's car red (picture 5); Session Best is a 6-row table with `LAST`, `BEST` on a green tag, `OPT`, red plus and blue minus deltas (picture 6). A GTPlanet thread of April 2026 lists the same pages and calls the MFD a "floating pageable informational display, inside most cars" ([GTPlanet](https://www.gtplanet.net/forum/threads/suggestion-request-qol-changes-improvements-to-gt7-psvr2-mfd-hud.437093/)).
16. **Brake and throttle bars** flank the MFD: slim vertical bars in a thin bracket with three ticks, white fill with the pedal; **Cited:** red shows ABS reduction on the brake bar and traction-control reduction on the throttle bar (items 16 and 19).
17. **Telltale rows** under each dial: three circled icons, left ABS, auto-drive brakes, auto-drive steering; right TCS, ASM, countersteer. White on, grey off, **solid red disc engaged** (Cited, items 22 and 23; picture 2 shows the right three engaged).
18. **Tyre diagram** far left (car outline, four tyre pills, compound initials front and rear) with the **surface water bar** in a bracket beside it (Cited, items 20 and 21; picture 3).
19. **Extent.** **Computed:** the two dials span **1756 px** at 4K (±878), 46% of the screen width; from the steering dot to the telltale row the meter is **724 px** tall. The tyre block adds about 140 px on the left.

### Colour behaviour
20. **Monochrome white and grey; red for alerts; one accent per MFD page.** **New (Our reading from pictures 1 to 6, step 1 confirms with a video):** the tick strip is **dark until the alert range**, then lights red-pink from its left end, and **the tach's fill turns red at the same moment**. Picture 2 (tach fill white to about 8.7 of 10, red band from 9) and picture 1 show an unlit strip; pictures 3, 4 and 6 (tach fill red) show the strip lit red over 57%, about 45% and about 35%. **Cited:** the packet sends `MinAlertRPM` and `MaxAlertRPM`, "minimum rpm to which the rev limiter shows an alert" and its maximum, and the flag `RevLimiterBlinkAlertActive` (PDTools). **Our reading:** the strip's lit length is the fraction of the way from `MinAlertRPM` to `MaxAlertRPM`. This differs from `Gt7HoodPro`, whose strip shows a white run from idle and turns red at the warning stage; the hood note took its reading from one Bonnet View picture already in the alert.
21. **Braking suggestion turns the pod red** (Cited, item 11: "the meter panel will turn red"; picture 5 shows the whole pod tinted red).
22. Sampled hues (JPEG estimates): strip and tach alert red about `#E0405A`, TCS accent blue about `#2E8BDA`, torque accent yellow about `#E4D23A`, BEST tag green about `#2EA043`, panels grey at roughly 40% over the scene.

### Typography
23. Squared, rounded-corner digits throughout (thin for speed, odometer and MFD values, bold for gear and the TCS level), light grotesque numerals on the dials. GT7's face is proprietary; the finished `Gt7HoodPro` uses Michroma (speed), Orbitron 600 (gear) and Titillium Web 600 (labels) as the nearest open faces.

## 2. Reference photos
| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Race screen, items 1 to 25 numbered: both dials, pod, MFD "Course Map", bars, telltales | https://www.gran-turismo.com/images/c/i16HBxihYqSTHO.jpeg | [The Race Screen](https://www.gran-turismo.com/gb/gt7/manual/race/02) |
| 2 | Normal View, the main measuring picture: mph dial, 0 to 10 tach, Track Map, red box `1` AT | https://www.gran-turismo.com/images/c/i1iYlkuZCwBxdz.jpg | [Racing Views](https://www.gran-turismo.com/gb/gt7/manual/race/03) |
| 3 | MFD TCS page; strip lit red 57%, tach fill red, empty suggested-gear slot | https://www.gran-turismo.com/images/c/i1p3rCOIZ7S1hG.jpg | [The Multi-Function Display](https://www.gran-turismo.com/gb/gt7/manual/race/04) |
| 4 | MFD Torque Distribution page (yellow), eight page dots | https://www.gran-turismo.com/images/c/i1DCktyPUHTylSH.jpg | same |
| 5 | MFD Radar page; pod tinted red; left blindside lit | https://www.gran-turismo.com/images/c/i1xA6ggV8CaXd.jpg | same |
| 6 | MFD Session Best page | https://www.gran-turismo.com/images/c/i1ZQd1qp86FXz8E.jpg | same |
| 7 | Bonnet View band (Gt7HoodPro's target, for telling apart) | https://www.gran-turismo.com/images/c/i1vStJCga1tqJuB.jpg | [Racing Views](https://www.gran-turismo.com/gb/gt7/manual/race/03) |
| 8 | Cockpit View band (Gt7CockpitPro's reference) | https://www.gran-turismo.com/images/c/i1FGJ2FCkZaqlEE.jpg | same |
| 9 | Chase View band | https://www.gran-turismo.com/images/c/i1eOkILgEsxJw.jpg | same |
| 10 | The other five MFD page pictures (Brake Balance, Fuel Map, Track Map, Weather Radar, Session Best layout) | URLs not captured; pictures 3, 4, 5, 7, 8 on the page | [The Multi-Function Display](https://www.gran-turismo.com/gb/gt7/manual/race/04) |

The pictures are copyrighted and stay out of git. Drive `references` folder (holds `SOURCES.md`; the PC fetches the listed URLs): https://drive.google.com/drive/folders/1Px635W_CTbj-ij0kmPILJ1u_CcurElB7

## 3. How the real thing behaves, with motion notes
| State | GT7 | Source | For this dash (Our reading) |
|---|---|---|---|
| Start-up | Not documented | unconfirmed (step 1: a race-start video) | rings draw on first (track, then numerals), fills sweep 0 to max and back once inside `bootMax` 1000 ms, MFD fades in on its last page |
| Idle | Fills short; strip dark; suggested slot dark | pictures | strip fully dark at idle, the cleanest GT7 cue |
| Cruise | Speed and rpm fills grow; nothing else moves | pictures 1, 2 | fills on a critically damped spring (τ about 70 ms, as the hood dash's ribbon); numerals and rings static |
| Pull to the limiter | From `MinAlertRPM` the strip lights red from the left and the tach fill turns red | packet; pictures 3, 4, 6 | strip lit fraction = alert progress; tach fill takes `shift` at the same instant |
| Limiter | `RevLimiterBlinkAlertActive` flag; blink rate undocumented | packet | only the strip strobes (10 Hz, 50%); tach fill holds steady `shift` |
| Gear change | Suggested box goes dark when no suggestion; gear digit changes | pictures 2, 3 | gear swaps in one frame; strip drops dark as revs fall below the alert start |
| Braking | Pod turns red, red text blinks; brake bar shows ABS cut in red | items 11, 16 | pod tint from `normalized_ai_brake_difference`, steady, never blinking |
| Drift / lateral G | Countersteer and ASM telltales go solid red | items 22, 23; picture 2 | SLIDE telltale from body slip β |
| Reverse / neutral | Not pictured | unconfirmed | `R` in `warning`, `N` at full ink, as the hood dash |
| MFD page change | Left/right buttons step pages; dots move | manual race/04 | page content cross-slides 1 page width on the `default` spring (237 ms); the title and dots change on the same frame; never auto-pages during the limiter |
| Shut-down / pause | Packet flags `Paused`, `LoadingOrProcessing` | packet | the host hides the dash on a stale stream |

## 4. The maths
No row in "Real geometry and maths". Values for the unit tests (all **Computed**):
- **Dial angle:** θ(v) = θ0 + 276° x clamp(v / v_end, 0, 1), θ0 = 222° clockwise from 12 o'clock (gap 84° centred on 6 o'clock). 67 mph on 200 → 92.5° past θ0.
- **Numeral pitch:** 276° / n: 200 mph by 20 → 27.6°; 280 km/h by 40 → 39.4°; 320 km/h by 40 → 34.5°; tach 10 → 27.6°, 8 → 34.5°.
- **Ring arc length** at the fill's mid-radius 235.5 px: 2π x 235.5 x 276/360 = **1134 px**.
- **Speedo end without `CalculatedMaxSpeed`:** v_top = max over learned gears of (speed/rpm)_g x max_rpm (the catalog's live `[f32; 10]` table), end = step x ceil(1.05 v_top / step), step 40 km/h or 20 mph. Example: 0.0095 m/s per rpm x 8000 = 76 m/s = 273.6 km/h → **320 km/h**, 170.0 mph → **180 mph**. Until a gear is learned, use a table by `car_class` (MODEL, decoded, offset confirmed at check C); the end never shrinks for the same `car_ordinal`.
- **Tach end** = ceil(max_rpm / 1000) x 1000; red band from `shift_rpm` (0.97 x max_rpm) to the end.
- **Alert progress** (GT7's `MinAlertRPM` to `MaxAlertRPM`): lo = 0.93 x shift_rpm, hi = shift_rpm, f = clamp((rpm - lo)/(hi - lo), 0, 1). Max 8000 → shift 7760, lo 7216.8: 7000 → 0, 7300 → 0.153, 7500 → 0.521, 7760 → 1.
- **Boost sub-gauge:** ×100 kPa = psi x 6.894757 / 100; 14.7 psi → 1.014; the gauge's -1 to 2 spans -14.50 to 29.01 psi.
- **Units:** 1 m/s = 2.236936 mph = 3.6 km/h. Torque fallback 300 kW at 7000 rpm → 409.3 N·m. β = atan2(v_x, v_z).
- **Fit:** the meter is 1756 x 724. Cockpit envelope 1440 x 560: uniform scale min(0.820, 0.773) = **0.773** (gear cap 63 px). Hood envelope 1760 x 400: uniform scale **0.552** (gear cap 45 px), too small, so the hood needs a re-arranged layout (section 9, Q3).

## 5. Telemetry mapping
| Visual part | Input | Status | Transform |
|---|---|---|---|
| Speed fill, numerals, end | `speed`, learned gear table | verified, derived | section 4; tabular digits with hysteresis |
| Speed digits, unit | `speed`, Signals `unit` | verified | m/s → mph or km/h |
| Tach fill, numerals, red band | `rpm`, `max_rpm`, `shift_rpm` | verified | fill white below lo, `shift` from lo |
| Tick strip lit length, strobe | `rpm`, `shift_rpm` | verified | f from section 4; strobe at the shift point |
| Gear | `gear` | verified | `R`, `N`, 1 to 10+ |
| Suggested-gear box | `gear`, `rpm` | derived | gear + 1 at the shift point (as the hood dash), dark slot otherwise |
| Steering dot | `steer` | verified | −127..127 → ±rail |
| Brake, throttle bars | `brake`, `throttle` | verified | 0..1 fill |
| Red share of the bars | `tyre_slip_ratio` x4, `drivetrain` | decoded, offset confirmed at check C | MODEL: front lock under brake, driven-wheel spin under throttle |
| Boost sub-gauge | `boost` | verified | ×100 kPa; shown once the car has ever exceeded +0.1 bar this session |
| Fuel semicircle | `fuel` | decoded, offset confirmed at check C | 0..1 → E..F; hidden while it never changes (free roam often sends 1) |
| Odometer box | `race_progress` (distance) | decoded, offset confirmed at check C | metres → km or miles, 1 decimal; fallback ∫ speed dt |
| Handbrake icon | `handbrake` | decoded, offset confirmed at check C | lit above 0.1 |
| Pod red tint | `driving_line` (AI brake difference) | decoded, offset confirmed at check C | steady tint while it says brake |
| Tyre pills, frames | `tyre_combined_slip`, `tyre_temp` x4 | decoded, offset confirmed at check C | pills by slip, frames redden off-band; fallback load from G |
| Surface water bar | `puddle` x4 | decoded, offset confirmed at check C | mean of four wheels |
| Telltales | see below | mixed | section 5 replacements |
| MFD pages | see below | mixed | section 5 replacements |

**What the game cannot send** (★ recommended). Lights, assists, other cars, the map, weather and the driver's in-race settings are absent from the 88 fields.
- **Blindside half-rings** (other cars). ★ **Kerb half-rings:** the side's rings light while that side's wheels are on a rumble strip (`rumble_strip`, decoded, offset confirmed at check C), keeping position and "something beside you". Second: rings fill with lateral G on the outside of the turn.
- **Headlight icon.** ★ **Drivetrain glyph** (FWD, RWD, AWD from `drivetrain`, decoded) in its place, lit when the driven axle slips. Second: cylinder count (`cylinders`) as a static label.
- **Six assist telltales.** ★ **Grip telltales in GT7's three-circle rows:** left LOCK (front slip ratio under brake), HBK (handbrake), KERB; right SPIN (driven slip under throttle), SLIDE (β over 8°), LIM (limiter); grey quiet, white armed, solid red disc when engaged, as GT7. LOCK and SPIN are MODEL. Second: fewer circles, two per side.
- **MFD Track Map.** ★ **TRAIL page:** the last 60 s of `position` x/z (decoded, offset confirmed at check C) drawn as GT7's course line, car-centred and heading-up with GT7's crosshair, the car a red arrow. It is a trace of where the car has been, not a map, so it does not tread on `AmgMbuxTrackPace`. Second: a G trace.
- **MFD Radar.** ★ **G-G page in the radar's own rings:** rings at 0.5, 1.0 and 1.5 g with the crosshair, the felt-force dot (−acceleration) as the red car pill, a 500 ms trail. Verified data.
- **MFD TCS (blue arc).** ★ **SLIP page:** the same blue arc LOW to HIGH showing driven-axle slip ratio, the big squared digit the peak of the last 2 s ×10 (MODEL). Second: body slip β in degrees.
- **MFD Torque Distribution (yellow arc).** ★ **POWER page:** the same yellow arc for power as a share of the session peak, `412 kW : 520 N·m` in GT7's `35:65` style (`power` verified, `torque` decoded with the formula fallback, unit from `power_unit`).
- **MFD Brake Balance.** ★ **BRAKE page:** an observed lock-bias slider, which axle reaches slip −0.15 first under braking (MODEL), labelled as observed, never as a setting.
- **MFD Fuel Map and laps estimate.** ★ **FUEL page:** fuel level and fuel per km from `fuel` and distance (derived, MODEL once it has 1 km of data); no laps-left estimate the game does not give. Hidden when fuel never changes.
- **MFD Weather Radar.** ★ **SURFACE page:** four wheel cells for puddle, surface rumble and kerb (decoded, offset confirmed at check C).
- **MFD Session Best.** ★ **LAP page in GT7's table:** current, last and best lap, lap number and position (`race_progress`, decoded, offset confirmed at check C); shown only when lap number > 0. Second (free roam): 0 to 100 km/h and 100 to 0 timers from speed and timestamp in the same LAST/BEST rows.
- **Paging input.** GT7 pages with the D-pad; FH6 sends no button presses. ★ An app key or wheel button, if `AppConfig` gets one, else `config.extra` key `Gt7ModularTripodPage` sets the page and events may jump to one (LOCK at heavy braking jumps to BRAKE once per 5 s). Open question Q2.

## 6. Edge cases
- **Max rpm 0 or under 1000:** tach shows ring and numerals 0 to 8 dark, no fill, no alert, strip dark, no suggestion.
- **Idle near max:** f uses lo = 0.93 x shift, so the strip lights only near the top; if idle ≥ lo, hold the strip dark until rpm rises 2% above idle.
- **12,000+ rpm:** 12 to 20 numerals at 23° to 13.8°; above 12 show every 2000. **Diesels at 4,500:** 5 numerals at 55°, fine.
- **EVs:** no idle, one gear; tach still follows max_rpm; no boost sub-gauge; suggestion off if `cylinders` reads 0 (that 0 means electric is unconfirmed, step 1 checks).
- **Reverse, neutral, gear above 10:** as the hood dash; speed fill uses |v|.
- **mph, km/h, hp, kW:** speedo numerals and end recomputed when the unit changes; POWER page follows `power_unit`.
- **Limiter bounce (about 50 rpm a frame):** 120 rpm hysteresis on the alert start and the suggestion; the tach fill stays red through the bounce.
- **Spikes, crashes, airborne:** G-G dot clamped at 2 g; trail paused while airborne (four wheels near full droop).
- **Stale or paused stream:** the host hides the dash; the MFD remembers its page.
- **Palette extremes and random palettes:** page accents come from `accent` and `secondary`, so a palette with near-equal hues still separates them by lightness; the alert stays `shift`, red stays `warning`.
- **Font swap:** the user's display font replaces speed, gear and MFD numerals; dial numerals keep their light role so the rings do not crowd.
- **Cockpit and hood:** both listed; see section 4 fit and Q3.
- **Reduced flash:** strip breathes at 2 Hz; pod tint and telltales never flash (GT7 blinks its brake text and nitro; this dash does not).
- **Colour-only cues:** telltales carry letters; alert also shows by the strip lighting at all (dark to lit), not only by hue; kerb rings by side.

## 7. Signature moments
**Limiter / shift strobe** (only the shift strip strobes, 10 Hz at 50%, 2 Hz breath in gentle mode):
1. ★ **GT7's dark-to-red strip:** the strip is dark all the way to the alert start, then fills red from the left as the revs climb (f from section 4) while the tach's fill turns `shift`; at the shift point the strip strobes. It is GT7's own behaviour (point 20) and unlike either sibling, whose strips run white from idle.
2. **Mirror fill:** the strip fills from both ends to the centre over the alert range, as the Race Dash family's outside-in fill, landing over the gear at the shift point. Family-true, less GT7.
3. **Ring wipe:** at the shift point the tach's red band sweeps once from the red zone start to the end (150 ms), then the strip strobes. Ties the dial to the strip.

**Gear change:**
1. ★ **Slot hand-off:** the red suggested box empties into a dark slot as the new gear lands, the gear digit swaps in one frame, and the strip drops dark (revs fall under the alert start), so the whole meter goes calm in one beat.
2. **Fill settle:** the tach fill falls to the new rpm on a slightly underdamped spring (ζ 0.8) so the drop reads as a physical sweep.
3. **Dot step:** the page dots tick once at a gear change on the POWER page only, showing the new gear's power share. Small and optional.

## 8. What it needs from the 4K library
- **Kit made for this dash:** none. **Cited:** PREBUILD.md on the PR #18 branch says "Gt7ModularTripod (translucent panels, thin lines) has no hardware", and `kits.rs` has no entry.
- **Shared parts to reuse (on main, `assets/clusters/gt7_hood_pro/`):** the red enamel suggested-gear tile `suggest_0..2.png` (110 x 102 at three azimuths; GT7's box here is 87 x 72, so scale 0.79 or re-render); the hood recipe's materials (black glass, black chrome, smoked LED lens) in `tools/asset-forge/src/recipes/gt7_hood_pro.rs`; the forged-carbon master (seed 73101) only if the tyre diagram gets a car chip. The plate (1760 x 240) is layout-bound and not reusable.
- **Race Dash rule:** materials serve realism only (anti-glare matrix, lens caps, carbon); GT7 has no housing, so the material work lands on the translucent panels, the dial rings and the strip.
- **Layout-dependent (not gaps):** the pod's glass slab at its board size, the MFD field, the bars' brackets.

LIBRARY GAP: a seamless 4096 px smoked anti-glare micro-matrix texture for GT7's translucent pod and MFD panels, so they read as tinted display glass at glance scale and not as flat grey; ComfyUI SDXL, then sampled into a Blender glass swatch.

LIBRARY GAP: a dial ring sprite at Ø486 px 4K (2 px per px master), a satin dark track 5 px wide with a fine machined outer lip, three light azimuths (−110°, −135°, −160°), so both dials and the 1134 px fill track are rendered hardware under the vector fill; Blender hero.

LIBRARY GAP: a circled telltale icon set (LOCK, SPIN, SLIDE, HBK, KERB, LIM, drivetrain FWD/RWD/AWD, handbrake, fuel pump, turbo) drawn in GT7's thin-ring style, SVG to the kit's icon atlas with on, off and engaged states; Penpot.

LIBRARY GAP: one smoked LED lens cap sprite (24 x 32 px, three azimuths) for the tick strip, the same gap `Gt7CockpitPro` lists; one sprite serves both; Blender with the hood recipe's lens material.

## 9. Departures and open questions for the board
1. **No needle.** GT7 fills the ring (point 3); the catalog says "thin white scale". Keep GT7's fill; the board confirms after step 1's video check.
2. **Paging input.** FH6 has no D-pad events. Choose between an app key or wheel button (needs an `AppConfig` field, a coordinator ask), a fixed page from `config.extra`, or event jumps. Recommendation: config page plus event jumps, rate-limited.
3. **Hood layout.** Uniform scale gives a 45 px gear. Proposal: in the hood view the dials shrink to about Ø365 and rise level with the pod, the MFD sits under the pod (pod 200 + MFD 160 = 360 px of 400), the tyre block drops. The catalog's old "satellites at ±700 px" note is from the dropped capsule concept and should not be used.
4. **Strip behaviour differs from `Gt7HoodPro`.** This dash keeps GT7's dark-until-alert strip (point 20). The hood dash should hear of the finding; whether it changes is the lead's call.
5. **Accent colours.** GT7 uses a different accent per page (blue, yellow, green tag). The family wants one accent: map adjustable pages to `accent`, the BEST tag to `secondary`, and keep red for alerts. The Race Dash `shift` default is purple (`#B27CFF`), GT7's alert is red-pink: the board picks per palette.
6. **No housing.** The family language says "a machined housing carries a top LED bar"; TJX's override says no backing panel. The override wins; the strip's lens caps and the panel glass carry the materials.
7. **Arrows at the pod corners** (point 13) unconfirmed: draw nothing until step 1 identifies them.
8. **Tyre block:** keep (decoded tyre temps and slip, with fallbacks) or drop for width in the cockpit envelope.
9. **Customisable without losing the look** (TJS, "you can change for example needle color etc."): ring fill colour → `ink` (default warm white `#F9F6F1`); alert colour of strip and tach → `shift` (default GT7 red-pink `#E0405A`); page accents → `accent` and `secondary` (defaults blue `#2E8BDA`, green `#2EA043`); suggested box → `warning` (default `#E8090E`); panel opacity → `config.extra` `Gt7ModularTripodPanel` (default 0.40); numerals → the user's display font (default Michroma and Orbitron as the hood dash); speed unit and power unit → `AppConfig`; start page → `Gt7ModularTripodPage` (default TRAIL, GT7's Track Map slot).
10. **Catalog corrections for the lead:** the brief's "thin white scale" should read "thin scale with a thick white fill band, no needle"; "red arc from the shift zone" is right, and the whole tach fill also turns red in the alert range; the old "Hood [4]: satellites move to ±700 px at 6° yaw" line belongs to the dropped concept.

## 10. Sources
**Polyphony Digital, GT7 online manual (read in the PC's browser)**
- The Race Screen: https://www.gran-turismo.com/gb/gt7/manual/race/02
- The Multi-Function Display: https://www.gran-turismo.com/gb/gt7/manual/race/04
- Racing Views: https://www.gran-turismo.com/gb/gt7/manual/race/03
- Display Settings (no meter options found): https://www.gran-turismo.com/gb/gt7/manual/drivingoption/05
- Pictures: the nine image URLs in section 2

**GT7 packet decode**
- PDTools `SimulatorPacket.cs` (read from GitHub raw): https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs

**Community**
- GTPlanet, MFD pages and the floating MFD, April 2026: https://www.gtplanet.net/forum/threads/suggestion-request-qol-changes-improvements-to-gt7-psvr2-mfd-hud.437093/

**Repo**
- `crates/fh6-clusters/src/clusters/gt7_hood_pro/` (RESEARCH.md, HANDOVER.md), `gt7_cockpit_pro/RESEARCH.md`, `assets/clusters/gt7_hood_pro/ASSETS.md`, `docs/design/families/race-dash.md`, `docs/TELEMETRY_USAGE_MATRIX.md`, `crates/fh6-kit/src/tokens.rs` (envelopes), PR #18 branch `program/phases/07-cluster-factory/PREBUILD.md` and `tools/asset-forge/src/prebuild/kits.rs`.
