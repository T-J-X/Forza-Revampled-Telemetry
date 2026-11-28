# AmgOneF1: research note

Build #16, catalog row 52, category Hypercar, Target `recreation`, family Race Dash, view cockpit only. It recreates the Mercedes-AMG ONE's free-standing driver display and the LED shift display on its steering wheel ([CATALOG.md, The five AMG dashes](../../../../../program/phases/07-cluster-factory/CATALOG.md#the-five-amg-dashes)). Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Telemetry updated 2026-11-28 for the 88-field decoder (main 60c99c9).

Labels: **Cited** means read on 2026-11-28 from the linked page. **Computed** means worked out here from cited or repo values, with the inputs given. **Our reading** is this note's own conclusion or idea, open to the direction check. "Unconfirmed" means it was looked for and not found, and says what step 1 must check.

**The headline gap, read first.** No text source opened describes what the ONE's driver display draws, and no picture could be looked at: every image URL returned a server error to the fetch tool and the shared web-fetch budget (100 calls an hour across all agents) ran out mid-session. The display's layout, rev-band form, typeface, colours, the shift-light count, colours and firing order therefore all stay **unconfirmed**. Step 1 must take frame grabs from the videos in section 2 before any layout work, as the brief's Risk line already demands. What this note gives is everything the text sources do confirm, the questions the frames must answer, and the ideas for what the game cannot send.

## Deliberate changes (the AMG rule: read first)

These hold whatever the frames show ([CATALOG.md, The five AMG dashes](../../../../../program/phases/07-cluster-factory/CATALOG.md#the-five-amg-dashes); shared notes in `amg_mbux_superscreen/RESEARCH.md`, branch `claude/project-thread-aig4a0`, "Shared AMG MBUX notes").

1. **Only the shift strip strobes.** If the real display turns an area red or flashes at the limiter, the dash keeps that area steady; only the rim row strobes, 10 Hz at 50% duty, a 2 Hz breath in reduced-flash mode ([TOKENS.md](../../../../../docs/design/TOKENS.md), limiter strobe).
2. **Deltas by role, with a sign.** Any lap or timer delta uses `go` and `warning` with a sign and a direction, never colour alone.
3. **No maker's marks.** The logo zone shows the driven car's logo or the custom name; no star, no AMG mark, no "ONE" wordmark drawn by the dash.
4. **Palette roles.** Colours come from the dash's default palette (the `amgMbux` family palette or a palette made for this dash once the frames show the ONE's own colours), never literal hex in `draw()`.
5. **Swappable font.** The real typeface is unconfirmed and almost certainly not freely licensed. The brief's Saira Condensed 600/800 and Titillium Web 600 are in `assets/fonts/`; the shared AMG notes suggest Barlow Semi Condensed so the five AMG dashes read as one maker. Section 9 asks the board to choose.
6. **No hybrid state is invented.** Battery charge, ERS deployment, EFC mode, the drive program, the aero state and DRS are not in the packet; none is drawn as a live value (brief, Telemetry line). The 88-field decoder adds none of them; whether `torque` or `fuel` carry any hybrid share on the ONE is a check C question (section 5). Section 5 replaces each.

## 1. The real display

| # | Feature | Label and source |
|---|---|---|
| R1 | Two free-standing, high-resolution 10-inch displays "with individual Mercedes-AMG ONE graphics": one in front of the driver, the other on the right of the centre console, tilted to the driver. | **Cited**, production press release, reprinted by [CarSifu](https://www.carsifu.my/news/mercedes-amg-one-makes-world-premiere) and [MarketScreener](https://www.marketscreener.com/quote/stock/MERCEDES-BENZ-GROUP-AG-436541/news/The-new-Mercedes-AMG-ONE-Formula-1-technology-for-the-road-40601788/); [MBUSA](https://www.mbusa.com/en/future-vehicles/mercedes-amg-one) |
| R2 | The instrument display "is in a slightly raised position in front of the driver"; "The most important information is displayed on the road in the line of vision above the steering wheel, so that the driver is not distracted." | **Cited**, CarSifu; [Project ONE show-car press kit, MBUSA PDF](https://www.mbusa.com/vcm/MB/DigitalAssets/pdfmb/about_us/Mercedes-AMG-Project-ONE-Showcar-Final.pdf) |
| R3 | The displays "are adapted with high-quality and weight-optimized solid metal components"; they sit on a "slim, wing-like profile of the dashboard" that "appears to be light and free-floating". | **Cited**, Project ONE press kit. This is the real reason the dash may have a machined housing (brief). |
| R4 | "The displays' rectangular basic form is echoed by the ventilation nozzles" and the centre console. | **Cited**, Project ONE press kit. **Our reading:** a plain rectangle with small radii, not a curved or cut-corner slab. |
| R5 | "The user interface is integrated as an autonomous concept." | **Cited**, Project ONE press kit. Its own UI, not MBUX menus: so the Supersport, Classic and TRACK PACE styles of the other four AMG dashes do not apply here. |
| R6 | The 2017 show car's driver screen "shows the car's rev counter and speed". | **Cited**, [CarWale](https://www.carwale.com/news/more-details-on-formula-1engined-amg-one-uncovered/) (show car, not production). |
| R7 | The centre screen is "the touchscreen from an A-Class with all manner of infotainment and lap time functions". | **Cited**, [Top Gear, interior](https://topgear.com/car-reviews/mercedes-benz/amg-one/interior). Lap timing lives on the centre screen, not (as far as sources say) on the driver display. |
| R8 | Steering wheel "flattened at the top and bottom", airbag; "The 'shift light' is displayed at the top of the steering wheel rim, as is usual in racing cars." | **Cited**, CarSifu |
| R9 | Show car: two controllers set "the driving modes and suspension setup, or the LED shift display in the upper steering wheel area". MBUSA: "LED shift display". Top Gear: "a strip of LEDs to warn when to change gear". mercedesblog: "a LED bar above the controls". | **Cited**, Project ONE press kit; MBUSA; Top Gear interior; [mercedesblog](https://mercedesblog.com/how-the-controls-are-organized-on-the-mercedes-amg-one-steering-wheel/). **Our reading:** the controllers can adjust the LED display, so the real light thresholds are probably driver-set; ours come from the car profile. Whether the bar sits in the rim or just under it on the wheel face is for the frames. |
| R10 | Wheel buttons by colour: blue main-beam flash, gold DRS (left), green Neutral, red boost for passing ("Pass"); a rotary at bottom right selects drive modes; a switch for MGU-K recuperation and one for the pit-lane limiter. | **Cited**, mercedesblog; Top Gear interior ("main beam flash, DRS, EPAS boost and selecting Neutral"); [The Drive](https://www.thedrive.com/news/watch-how-complicated-the-mercedes-amg-ones-drive-modes-are) |
| R11 | Drive programs: Race Safe (default), Race, EV, Race Plus, Strat 2, Individual. Aero states Highway, Track, Race DRS. Race+ offers "four electric flow control modes"; in Strat 2 "the car will use all of its battery power in a single lap". | **Cited**, CarSifu; [paultan](https://paultan.org/2022/06/01/mercedes-amg-one-finally-revealed-1063-ps-f1-1-6l-turbo-hybrid-v6-0-100-km-h-in-2-9-secs-352-km-h-top); The Drive |
| R12 | Race DRS: about 20% less downforce at a button press, "automatically deactivated when the driver hits the brakes or lateral acceleration is detected". | **Cited**, [Carscoops](https://www.carscoops.com/2022/06/mercedes-amg-one-debuts-with-1049-hp-f1-derived-hybrid-powertrain/) |
| R13 | 11,000 rpm limit; idle about 1,280 to 1,300 rpm; seven-speed single-clutch automated manual. | **Cited**, Project ONE kit (11,000); [Wikipedia](https://en.wikipedia.org/wiki/Mercedes-AMG_One) (1,280 idle, 7-speed); Top Gear review ("idled calmly at 1300rpm"); [Top Gear, driving](https://www.topgear.com/car-reviews/mercedes-benz/amg-one/driving) ("single clutch sequential"). mercedesblog's "8-speed dual-clutch" is wrong against the press material. |
| R14 | Reviewers saw "a flashing warning on the dashboard" and "a message about aero stuff on the screen" (when the speed limit of the aero state was reached). | **Cited**, [Top Gear NL](https://topgear.nl/autotest/mercedes-amg-one-review-kan-het-echt-een-f1-motor-in-een-straatauto/), Dutch, translated. So the real display does show aero-state messages. |
| R15 | The 2019 Mercedes F1 wheel, the benchmark the brief names: "A family of 15 LEDs form the shift lights". F1 wheels carry "a band of green and red lights to indicate ideal gear-shift timings" above the display. | **Cited**, [Top Gear, 2019 Mercedes F1 wheel](https://www.topgear.com/car-news/formula-one/boggle-your-mind-2019-mercedes-amg-f1-wheel); [Yahoo / F1 wheels explainer](https://uk.sports.yahoo.com/news/formula-1-steering-wheels-victory-at-the-flick-of-222434439.html) |

**Unconfirmed, for the frames** (each a step-1 question): F1 the screen's aspect and the housing's depth and radii; F2 where the gear sits and its size; F3 the rev counter's form (bar, arc, segmented band) and scale numerals; F4 speed position and units; F5 what the hybrid area shows (state of charge, deployment, EFC level) and where; F6 how the screen changes between Race Safe, Race, Race+ and Strat 2 (colour, layout or only a label); F7 the DRS and aero indicators; F8 the typeface and colours; F9 the LED count, colours and firing order, and whether the LEDs strobe at the limiter; F10 what the display shows during the 50 to 90 s pre-heat; F11 whether the game's own ONE cockpit (FH6 gameplay video, ref 15) copies the real screen, which would give step 1 a cheap second source. The brief's "a Formula 1 row of green, red and blue lights" is plausible from R15 but **unconfirmed** for the ONE.

**Better target?** No. The brief's target is right; it only needs the frames.

## 2. Reference photos

No image could be saved: every direct URL tried (refs 1, 4, 6, 7) returned a server error, and the shared fetch budget then ran out. All are listed for step 1. Captions are the pages' own; none was seen.

| # | What it shows (caption, not seen) | URL | Saved |
|---|---|---|---|
| 1 | Press photo 22C0211_054, steering wheel with the driver display behind it | https://gtspirit.com/wp-content/uploads/2022/06/22C0211_054.jpg | no |
| 2 | Press photo 22C0211_047, interior: both displays on the wing dash | https://gtspirit.com/wp-content/uploads/2022/06/22C0211_047.jpg | no |
| 3 | Press photo 22C0211_067, the MirrorCam roof screen | https://gtspirit.com/wp-content/uploads/2022/06/22C0211_067.jpg | no |
| 4 | Maker's site cockpit image, 1920x1080 | https://www.mercedes-amg.com/media/images/4bbde280c972d24528fdeef7083da799a31b8e82-1920x1080.jpg | no |
| 5 | Carscoops interior image | https://www.carscoops.com/wp-content/uploads/2022/05/2023-Mercedes-AMG-ONE-60.jpg | no |
| 6 | The Drive, two stills from the owner drive-mode article | https://www.thedrive.com/wp-content/uploads/2022/08/08/Mercedes-AMG-One-1-of-2.jpg | no |
| 7 | Top Gear review gallery (2025), interior among 18 | https://www.topgear.com/sites/default/files/2025/10/DSC03877.jpg | no |
| 8 | evo gallery, "interior" and "steering wheel" | https://www.evo.co.uk/mercedes/204868/production-specific-mercedes-amg-one-reveals-its-secrets-pictures | no |
| 9 | Video: owner on track, modes, Race+ entry, EFC, Pass | https://www.youtube.com/watch?v=YUtXNa2UdOU | no |
| 10 | Videos: start-up and warm-up | https://www.youtube.com/watch?v=C5mBu5W3O7Y, https://www.youtube.com/embed/OO9fCn1r7Hk | no |
| 11 | Video: "The ONE and ONLY POV out of the AMG ONE" | https://www.youtube.com/watch?v=pEn1W2Rn9y0 | no |
| 12 | Video: "2021 Mercedes-AMG One - POV TEST DRIVE" | https://www.youtube.com/watch?v=6mvmEz7yoQc | no |
| 13 | Video: "What It's Really Like To Own An F1-Powered AMG One" | https://www.youtube.com/watch?v=14zsF-Q9hS4 | no |
| 14 | Video: official Nordschleife record onboard | https://www.youtube.com/watch?v=0_VfzIDzM3U | no |
| 15 | Video: "Mercedes-AMG ONE Forza Horizon 6 Gameplay" (the game's cockpit) | https://www.youtube.com/watch?v=6PPlPhRvI5w | no |

Drive `references` folder (holds `SOURCES.md` only): https://drive.google.com/drive/folders/151LAw2Lc-72rpH19F55mdm9YOJ25RG7c. Frames to grab first: a POV at cruise (refs 11, 12), a pull to 11,000 with an upshift (refs 12, 14), the pre-heat start (ref 10), Race+ entry and Strat 2 (ref 9).

## 3. How the real thing behaves, with motion notes

| State | Real behaviour | Dash (Our reading unless marked) |
|---|---|---|
| Start-up | **Cited:** the car always starts silently on the front-axle motors; the engine joins only when the catalysts are hot, 50 to 60 s (Top Gear, Motor1) or 60 to 90 s (Motor1, [shut-down article](https://www.motor1.com/news/727380/amg-one-freezes-if-engine-prematurely-turned-off/)); full warm-up 5 to 8 min parked, 2 to 5 min driving. What the screen shows meanwhile is unconfirmed (F10). | A two-stage greeting: the screen wakes silent and dark-calm first, and the rim row stays dark until the first frame with rpm ≥ idle, when it sweeps once (the engine "joining"). See the boot idea in section 7. |
| Idle | **Cited:** about 1,300 rpm, rough ("sounds quite as broken at idle", Top Gear driving). | Rim dark (unlit LEDs neutral, alpha ≤ 22, TOKENS), rev band low, gear and speed steady. No idle wobble added. |
| Cruise | Unconfirmed. | Rim dark below stage 1; screen data only. |
| Pull to the limiter | **Cited:** "screamed through to 11,000rpm". LED strip warns when to shift (R9). | Rim fills on per-car, per-gear thresholds; attack 0 ms (TOKENS). |
| Limiter | Unconfirmed whether the real LEDs flash. | Only the rim row strobes (10 Hz, 50%); rev band and screen steady. |
| Gear change | **Cited:** single-clutch sequential; upshifts once called slow, now "surprisingly smooth and soft"; downshifts sharper (Top Gear driving). | The digit changes the frame the packet does: no animation that waits for a slow real shift (the Valkyrie note makes the same call for its slow AMT). Brief: row clears in 2 frames, gear enters 1.12 → 1.0. |
| Braking | **Cited:** full regeneration with a smooth hand-over to the discs; DRS cancels on braking (R12). | Idea "harvest" in section 5 uses brake and negative power honestly. |
| Lateral G | **Cited:** DRS also cancels on lateral acceleration (R12). | Section 5 idea D1 mirrors that logic for its own straight-line marker. |
| Reverse, neutral | Green Neutral button on the wheel (R10). | `R` and `N` in the gear slot; rim dark. |
| Shut-down | **Cited:** switching off before the engine is warm risks a lock-out (Motor1). | The app hides the dash when the stream stops (house rule); no shut-down scene. |

Timings: Mercedes documents none. The house timing applies (shared AMG notes): ribbon spring for fills with a 60 rpm backlash at the limiter, `HeroGear` for the gear, 10 Hz strobe, 2 Hz breath.

## 4. The maths

No row in "Real geometry and maths". The numbers the tests can assert are the light thresholds and the band scale. **Computed** with `python3 -I`, using TOKENS' fallback stages (0.75, 0.85, 0.93, 1.00 of shift rpm) and shiftPct 0.97:

| Car | max rpm | shift rpm | stage 1 / 2 / 3 / shift |
|---|---|---|---|
| AMG ONE | 11,000 | 10,670 | 8,002.5 / 9,069.5 / 9,923.1 / 10,670 |
| 6,000 rpm road car | 6,000 | 5,820 | 4,365 / 4,947 / 5,412.6 / 5,820 |
| diesel | 4,500 | 4,365 | 3,273.8 / 3,710.2 / 4,059.5 / 4,365 |
| 12,500 rpm engine | 12,500 | 12,125 | 9,093.8 / 10,306.2 / 11,276.2 / 12,125 |

- **A 15-LED row** (the Mercedes F1 count, R15; the ONE's own count is unconfirmed) spread evenly from stage 1 to shift on the ONE: thresholds 8,002, 8,193, 8,384, 8,574, 8,765, 8,955, 9,146, 9,336, 9,527, 9,717, 9,908, 10,098, 10,289, 10,479, 10,670 rpm, a step of 190.5 rpm. On the diesel the step is 77.9 rpm. The 120 rpm off-hysteresis (TOKENS) is wider than the diesel's step, so a falling pull can hold one LED more than the rising one; that is correct hysteresis, not a bug.
- **Rev band numbering** (brief: "numbered from `EngineMaxRpm` up to 11"): one numeral per 1,000 rpm, 0 to ceil(max ÷ 1000). On a 680 px slab with 24 px margins that is 57.5 px per 1,000 rpm at 11,000; 105.3 px at 6,000; 140.4 px at 4,500; 50.6 px at 12,500; 35.1 px at 18,000. Below about 40 px a numeral at a readable cap collides; the band should switch to every 2,000 rpm above 16,000 (Our reading).
- **The slab's shape.** 680 × 380 is 1.79:1, within 1% of 16:9 (1.78). A 10-inch 16:9 panel is about 221 × 125 mm (Computed). Whether the real panel is 16:9 is F1.
- **Strobe frames.** 10 Hz at 50% is 3 on / 3 off at 60 Hz and 6 / 6 at 120 Hz, but 7.2 at 144 Hz: the strobe must run on the timestamp clock, not a frame count.
- **Torque** (for idea P1): the packet's `torque` is now decoded (offset confirmed at check C; the page states no unit, and Forza Motorsport sends N·m). The derived T = P × 60 ÷ (2π × rpm) is the fallback and the check C cross-check: where both are read, they should agree in N·m. Example: 422 kW at 9,000 rpm is 447.8 N·m.

No catalog maths row to correct.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Rim shift lights | `rpm`, `shift_rpm`, `max_rpm`, gear | verified | per-car, per-gear thresholds (section 4), 120 rpm off-hysteresis, strobe at the limiter |
| Rev band | `rpm`, `idle_rpm`, `max_rpm` | verified | fill on the ribbon spring; numerals from max rpm |
| Gear | `gear` | verified | `R`, `N`, 1 to 10+, `HeroGear` |
| Speed | `speed`, `unit` | verified | tabular, `Hysteresis` on the shown digits |
| Power bar and value | `power` (power_watts) in `power_unit` | verified | kW or hp; bar against the learned peak for this car |
| Pedal traces | `throttle`, `brake` | verified | 0..1 traces, fixed ring buffer |
| Boost readout | `boost` (boost_psi) | verified | shy readout, hidden at 0 for NA cars |
| Acceleration timers | `speed_ms`, timestamp | derived | 0-100, 100-200 shared with `AmgMbuxTrackPace` |
| Torque (idea P1) | `torque` | decoded, offset confirmed at check C | N·m once check C confirms the unit (lb·ft when `power_unit` is hp is Our reading); fallback derived from power and rpm (formula above), hidden below idle |
| Straight-line marker (idea D1) | `throttle`, `brake`, `g_lat` | verified inputs, own logic | see below |
| Lap delta line (idea L1) | `race_progress` (current_lap, best_lap, last_lap, lap_number, distance_traveled) | decoded, offset confirmed at check C | live delta against the best lap at the same lap distance, signed, `go` / `warning` (rule 2); fallback the acceleration-timer row |
| Engine "joining" boot sweep and EV calm (section 3) | `cylinders`, `rpm`, `idle_rpm` | `cylinders` decoded, offset confirmed at check C; rpm verified | the ONE's 1.6 l V6 should read 6 (check C confirms); 0 means an EV (section 6) |
| ERS check (section 5 below) | `torque`, `power`, `fuel` | decoded, offset confirmed at check C (power verified) | diagnostic only at check C, nothing drawn from it |

### Decoded now, and whether the ONE's display shows it

These were in "What the game cannot send" before the 88-field decoder (2026-11-28):

- **Torque** (E1 below). The torque wing now shows the packet's own `torque`, not a derived figure; the derived value stays as the fallback and as the check C cross-check (section 4).
- **Lap times and the lap delta** (the real ones live on the centre screen, R7). **L1 (recommended), now real in races:** the brief's lap delta line is live. Keep one array, sized once, of the best lap's elapsed time against distance from the lap start (distance_traveled at the lap line subtracted, binned every 10 m, no per-frame allocation); the delta is current_lap minus the best lap's time at the same distance, shown with a sign and `go` (gaining) or `warning` (losing), never colour alone. At each lap line the row holds last lap minus best lap for 3 s. **L2 (fallback, kept):** in free roam, or until a first full lap exists, the row is the shared acceleration-timer row (0-100, 100-200) as before.
- **Fuel and tyre temperatures.** Decoded, but no source puts either on the ONE's driver display, so neither is drawn unless the frames (F5) show one. If they do, the readout maps to `fuel` or `tyre_temp`, labelled "decoded, offset confirmed at check C", with the edge cases of section 6.

**The ERS check (step 1, at check C).** The packet has no ERS, battery or EFC field, but two decoded fields could carry part of the hybrid on the ONE, and check C must look before anything is labelled: (1) drive the ONE flat out and compare `torque` with power × 60 ÷ (2π × rpm); if they agree, both are total figures and nothing separates the electric motors from the V6; if `torque` is steadily lower than the derived value, the gap is a lead (engine-only torque against total power), still never labelled ERS without a second proof; (2) watch `fuel` through the silent electric start, a hard lap and a long coast; if it rises under braking or behaves like a battery rather than a tank, report it, because that would make S3 below possible. Until both are answered, S1 and E1 stand.

### What the game cannot send

- **Battery state of charge.** **S1 (recommended): the "deploy" gauge becomes a live power gauge.** Where the real display shows the hybrid's energy, the dash shows current power as a fraction of the highest power seen for this car this session (learned live, reset on car change), as the bar the brief already wants, labelled with kW or hp. It is exact data, not a battery. **S2:** a "flat-out budget": the share of the current gear's run spent above 95% throttle, filling and draining like a charge bar, labelled as throttle time. **S3 (only if the ERS check shows `fuel` acting as a battery on the ONE):** that field drawn as the charge bar, labelled as the game's value.
- **ERS deployment and the four EFC modes.** **E1 (recommended): torque wing.** The decoded `torque` (decoded, offset confirmed at check C; derived fallback, section 4) beside power, so the screen shows the real "where the push is" figure the game does send. **E2:** a power-versus-rpm trace that draws this car's curve live as it is driven, the same envelope the EFC modes reshape in the real car; with `torque` decoded it can be a torque curve too.
- **Drive program (Race Safe to Strat 2).** **M1 (recommended): the name plate shows the car's own name** in the slot where the program would sit (brief, rank 5). **M2:** the slot shows the dash's own live state: `PIT` below 5 km/h, `PUSH` above 80% throttle, `COAST` on lift, named as the dash's states, not drive programs.
- **Aero state and DRS.** **D1 (recommended): a straight-line marker** lit while throttle ≥ 0.95, brake = 0 and |g_lat| < 0.15 g for 0.5 s, cancelled at once by brake or lateral g, exactly as the real Race DRS cancels (R12). Labelled `FLAT`, never `DRS`, so no DRS state is invented. **D2:** a downforce-feel bar from speed² (MODEL), labelled as such.
- **Regeneration.** **H1:** when brake > 0 and power is negative (engine braking, a real packet value), the power bar runs left of zero in a neutral role, labelled as engine braking, not harvest.
- *(Lap times moved to "Decoded now" above.)*
- **MirrorCam, media, navigation, warnings, coolant and oil temperatures.** Not in the 88 fields, and not on the driver display as far as sources say; nothing drawn. The aero-limit warning (R14) is not drawable without the aero state. (Fuel and tyre temperatures moved to "Decoded now" above.)

## 6. Edge cases

| Case | What the dash does |
|---|---|
| `max_rpm` 0 or tiny (< 500) | Rim and rev band hidden; gear and speed still shown. Clamp rpm to [0, max(max_rpm, idle_rpm + 1)] before any division. |
| idle near max | Thresholds below idle are lifted to idle + 1 step so the row never sits lit at idle. |
| 12,000+ rpm | Band numerals every 1,000 until 16,000, then every 2,000 (section 4). |
| diesel 4,500 | Works; LED step 78 rpm; hysteresis keeps it calm. |
| EV (single gear, no idle, odd rpm) | Known by `cylinders` 0 (decoded, offset confirmed at check C), with single gear and idle 0 as the fallback test. Row still follows rpm ÷ shift rpm; the gear slot shows `1` or the packet's gear; no limiter strobe if rpm never reaches shift; no boot sweep, since no engine joins. The ONE itself starts as an EV (R13 context), so an EV calm state suits this car; its own cylinders should read 6, not 0, even during the silent start. |
| Torque 0, negative or disagreeing | Below idle, or when `torque` reads 0 while power is above 0 for 0.5 s, the wing uses the derived value; negative torque on a lift is engine braking and shows left of zero like the power bar (H1); a steady gap to the derived value is reported by the ERS check, not hidden. |
| Fuel 0 or missing | Not drawn on this dash unless F5 shows a fuel readout; then 0 for 2 s with race on counts as not sent and the readout hides, never showing an empty tank. |
| No laps (free roam, `lap_number` and `current_lap` 0, best_lap 0) | The lap delta row is the acceleration-timer row (L2); a best_lap of 0 means no lap yet, never a 0:00.000 record. The first full lap fills the best-lap array; the delta shows from the second lap. |
| Lap reset or rewind | If current_lap or distance_traveled falls (a restart or a rewind, Y on the pad), the live delta hides until the next lap line; the stored best lap is kept. |
| Tyre temperatures before warm-up | Not drawn unless F5 shows them; if drawn, cold tyres take the neutral role, never a warning. |
| Reverse, neutral | `R`, `N`; rim dark; speed shown unsigned. |
| Gear above 10 | Two-digit glyph fits the gear's well (the ONE has 7, so the design must test 10+ anyway). |
| mph and km/h, hp and kW | From `unit` and `power_unit`; the label follows. |
| Limiter bounce (about 50 rpm a frame) | 120 rpm off-hysteresis on lights, `Hysteresis` on digits, strobe on its own clock. |
| Spikes and crashes, airborne | g inputs clamped for D1; power bar clamped to [−20%, 100%] of learned peak; learned peak ignores single-frame spikes (needs two frames). |
| Stale or paused stream | The app hides the dash. |
| Palette extremes, random palettes | Rim lights on `go` / `caution` / `warning` / `shift` roles; the validator keeps shift ≥ 60° from primary. |
| Font swap | All numerals follow the display font; the rim has no text. |
| Cockpit only | No hood variant listed. |
| Reduced flash | Strobe becomes the 2 Hz breath; the boot sweep becomes a fade. |
| Colour-vision | Position along the row and fill order carry the stage; the last LEDs or the full row being lit means shift, whatever the hue (brief Risk line). |

## 7. Signature moments

**Limiter / shift.** Only the rim row strobes, 10 Hz at 50%, 2 Hz breath in gentle mode.
- **L-A (recommended): the F1 fill, then the whole row.** The row fills in the real wheel's order once frames confirm it (F9); until then left to right, as a 15-LED F1 band (R15). At shift rpm every LED takes `shift`; at the limiter the row strobes and the screen holds still. Grounded in R8, R9 and R15.
- **L-B: outside-in, the family default** (TOKENS), kept only if the frames show the ONE fills from both ends.
- **L-C: the rim is ahead of the screen.** The rim arc sits detached above the slab (brief), with ±4 px parallax against the screen's ±2 (family), so at the limiter the strobe is the one thing moving in the driver's eye line.

**Gear change.**
- **G-A (recommended): the brief's clean cut.** On an upshift the row clears in 2 frames and the gear enters 1.12 → 1.0, nothing else moves. It suits the real single-clutch box: one decisive event, no slur.
- **G-B: the ignition sweep reused.** On the very first engine start (the boot) the rim sweeps once left to right in about 250 ms when rpm first reaches idle, the engine "joining" the electric start (section 3). Not used on gear changes, so it stays rare.
- **G-C: downshift tick.** On a downshift (sharper in the real car, Top Gear) the new gear enters from 0.92 → 1.0 instead, so up and down feel different without colour.

## 8. What it needs from the 4K library

From `tools/asset-forge/src/prebuild/kits.rs` and `assets/clusters/amg_one_f1/prebuild/manifest.json` (branch `claude/project-thread-ib67x9`, PR #18):
- `carbon_twill_rim`: Blender swatch, 4096 × 4096 with `_normal` and `_rmh`, 2×2 twill at 12 px tows: the surface of the 780 px rim arc.
- `antiglare_matte_grain`: computed, 4096 master: micro-grain over the screen face, luminance ±4% at most.
- Deferred by the kit: the housing, the rim arc and the shift-light caps (need the Penpot layout and the cap pitch).

Shared parts it can reuse: `shift_led_32_off` / `shift_led_32_on` (SimInputTrace and SimPitWall kits, 32 px caps tinted by role), `dark_anodised_carrier` and `black_anodised_alu` swatches for the real displays' "solid metal components" (R3), and the fonts in `assets/fonts/` (saira-condensed, titillium-web, barlow-semi-condensed).

LIBRARY GAP: a 34 px LED lens cap pair (off and on), Blender sprite at the rest key and two G-swing keys, to match the brief's 34 px rim row, unless the board accepts the shared 32 px caps (asset-forge prebuild, Blender lane).
LIBRARY GAP: a machined, bead-blasted aluminium swatch tile, 4096 × 4096 with normal and roughness, for the housing's metal edge that the press kit describes (R3); the kit has brushed, spun and anodised finishes but no blasted one (asset-forge prebuild, Blender swatch).

## 9. Departures and open questions for the board

1. **Everything on the screen waits for frames** (F1 to F11). Where frames cannot be found, the handover lists which parts are this thread's own design (brief Risk line).
2. **Rim or wheel face?** R8 says the rim; R9 says "upper steering wheel area" and "above the controls". The brief's detached carbon rim arc stays unless the frames show the LEDs on the face.
3. **LED count and colours.** 15 and green-red-blue are the F1 benchmark (R15), unconfirmed for the ONE.
4. **Font.** Saira Condensed (brief) or Barlow Semi Condensed (shared AMG notes). The ONE's UI is its own (R5), which argues for the brief's motorsport face.
5. **The lap delta line** is now kept (updated 2026-11-28): live from the decoded lap fields in races (section 5, L1), the acceleration-timer row in free roam (L2). The real ONE keeps lap timing on the centre screen (R7), so putting it on the driver display remains the brief's own choice.
6. **Drive-program slot** shows the car's name (M1); program states are not shown.
7. **DRS** becomes the `FLAT` marker (D1), never labelled DRS.
8. **No hybrid figures**: power and the decoded torque take the hybrid area (S1, E1). The packet still has no battery, ERS or EFC field; the ERS check at check C (section 5) decides whether S3 is possible.

## 10. Sources

Maker: [Project ONE show-car press kit (MBUSA PDF, 2017)](https://www.mbusa.com/vcm/MB/DigitalAssets/pdfmb/about_us/Mercedes-AMG-Project-ONE-Showcar-Final.pdf); production release reprinted by [CarSifu](https://www.carsifu.my/news/mercedes-amg-one-makes-world-premiere) and [MarketScreener](https://www.marketscreener.com/quote/stock/MERCEDES-BENZ-GROUP-AG-436541/news/The-new-Mercedes-AMG-ONE-Formula-1-technology-for-the-road-40601788/); [MBUSA product page](https://www.mbusa.com/en/future-vehicles/mercedes-amg-one).

Reviews and press: [Top Gear interior](https://topgear.com/car-reviews/mercedes-benz/amg-one/interior); [Top Gear driving](https://www.topgear.com/car-reviews/mercedes-benz/amg-one/driving); [Top Gear review](https://topgear.com/car-reviews/mercedes-benz/amg-one); [Top Gear NL](https://topgear.nl/autotest/mercedes-amg-one-review-kan-het-echt-een-f1-motor-in-een-straatauto/); [The Drive](https://www.thedrive.com/news/watch-how-complicated-the-mercedes-amg-ones-drive-modes-are); [Motor1 technical video](https://www.motor1.com/news/602971/mercedes-amg-one-technical-video/); [Motor1 shut-down](https://www.motor1.com/news/727380/amg-one-freezes-if-engine-prematurely-turned-off/); [mercedesblog](https://mercedesblog.com/how-the-controls-are-organized-on-the-mercedes-amg-one-steering-wheel/); [paultan](https://paultan.org/2022/06/01/mercedes-amg-one-finally-revealed-1063-ps-f1-1-6l-turbo-hybrid-v6-0-100-km-h-in-2-9-secs-352-km-h-top); [Carscoops](https://www.carscoops.com/2022/06/mercedes-amg-one-debuts-with-1049-hp-f1-derived-hybrid-powertrain/); [CarWale (show car)](https://www.carwale.com/news/more-details-on-formula-1engined-amg-one-uncovered/); [GTspirit first impressions](https://gtspirit.com/2022/06/04/first-impressions-mercedes-amg-one-road-legal-f1-car/); [Wikipedia](https://en.wikipedia.org/wiki/Mercedes-AMG_One).

Formula 1 benchmark: [Top Gear, 2019 Mercedes F1 wheel](https://www.topgear.com/car-news/formula-one/boggle-your-mind-2019-mercedes-amg-f1-wheel); [Yahoo / F1 wheels explainer](https://uk.sports.yahoo.com/news/formula-1-steering-wheels-victory-at-the-flick-of-222434439.html).

Videos for frame grabs: section 2, refs 9 to 15. Earlier leads: `/mnt/project-files/dash-research/amg-trackpace-one-superscreen.json`, item "Mercedes-AMG ONE displays".

Repo: `docs/design/TOKENS.md` (rev stages, strobe), `docs/design/families/race-dash.md`, `crates/fh6-kit/src/cluster.rs` (`Signals`), `docs/TELEMETRY_USAGE_MATRIX.md`.
