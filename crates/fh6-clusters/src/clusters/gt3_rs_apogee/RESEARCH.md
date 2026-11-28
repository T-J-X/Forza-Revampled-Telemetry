# GT3 RS Apogee: research note

Build #20, catalog row 54, category Supercar, Target `inspired`, family Classic Analog, view cockpit. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used below. **Cited**: read on 2026-11-28 in the source linked (most Porsche and spaceflight quotes were opened by the earlier web notes in `/mnt/project-files/dash-research/porsche-gt3rs-and-space-instruments.json` and re-checked by `verify_porsche-gt3rs-and-space-instruments.json`; where only the verify pass saw a page, the note says so). **Computed**: worked out here with `python3 -I` and numpy from the inputs shown. **Our reading**: a conclusion or proposal for this dash, open to the direction check. Anything marked *unconfirmed* is a job for step 1. The shared web-fetch budget ran out for most of this session, so several pages are cited from the verified earlier notes rather than re-opened.

## 1. The real display: parts kept, measured, and the departures

The real target is the **992.1 Porsche 911 GT3 RS** (world premiere August 2022), which is the game car `2023 Porsche 911 GT3 RS` (ordinal 3781; `data/car_profiles.json` gives it `CockpitAnchorY` 1830, `OemDashBinnacleOffsetY` 2, `OemDashWidthScale` 0.98). The 992.2 GT3 RS was still a prototype on 2026-11-28: Porsche's newsroom 911 page lists no such car, and evo (10 June 2026) only forecasts "a fully digital instrument panel" and "a Taycan-style curved display" for it (**Cited**, [newsroom 911 page](https://newsroom.porsche.com/en/products/911.html), [evo test-mule report](https://www.evo.co.uk/porsche/911-gt3-rs/207050/this-9922-porsche-911-gt3-rs-test-mule-might-be-hiding-a-secret); search titles of the same day still show only prototype and "about to get even better" stories). Step 1 re-checks whether a 992.2 GT3 RS has been shown; if it has, its cluster is the better real target for the turned scale.

### 1.1 What an owner recognises (feature inventory)

| # | Feature of the real car | Source | Kept as |
|---|---|---|---|
| K1 | **One physical rev counter in the middle**, "Seven-inch high-resolution screens flank the analogue tachometer". | **Cited**, [GT3 RS press kit (PCNA PDF)](https://newsroom.porsche.com/dam/jcr:46a23375-e7ee-4507-a577-d9761b784d33/992%20911%20GT3%20RS%20Press%20Kit%201.pdf) | The Ø500 rev counter in the middle with a detached part each side (attitude ball left, transfer plot right), standing where the two 7-inch screens stand. |
| K2 | **5,000 rpm at 12 o'clock** on the RS dial: "It's no coincidence that the 5000rpm marker is located at 12 o'clock on the rev counter. That's about where things get going." evo calls the cluster "a part-digital dash" and its driver's note praises the "ultra-simple dash display". | **Cited**, [evo GT3 RS review, updated 26 Nov 2025](https://www.evo.co.uk/porsche/911/gt3-rs) (re-read in the browser 2026-11-28) | Not kept: the dash turns the scale (K9). The classic position is an option (section 9). |
| K3 | **Scale to 10 with about 24° per 1,000 rpm**, a 240° sweep; 0 near 8 o'clock and 10 near 4 o'clock with 5 at 12. Porsche's GT3 document says the markings "run up to 10,000 rpm". | **Cited** for 0 to 10 and 24° ([PorscheAnalog RESEARCH.md item 4](../porsche_analog/RESEARCH.md)); "up to 10,000 rpm" in a Porsche PDF whose file is named for the GT3 RS but whose figures (375 kW at 8,400 rpm, six-speed manual) are the 992.1 GT3's, [PAG-911-GT3-RS-EN.pdf](https://newsroom.porsche.com/dam/jcr:fe2fb6d3-d0e3-44d8-8f2d-d82a66674718/PAG-911-GT3-RS-EN.pdf); end positions **Computed**: ±5 × 24° = ±120° from 12. No photo of the RS dial face was opened, so its numerals, needle colour and lettering are *unconfirmed*. | The 240° sweep and the whole-thousand numerals, scaled to the driven car. |
| K4 | **9,000 rpm limit, rated power at 8,500 rpm** (386 kW, 525 PS; the US kit says 517 hp). | **Cited**, press kit; the kW figure is in the verify pass | Real-car figures only. The dash's scale comes from the driven car's `max_rpm` (CATALOG, Flash and telemetry). |
| K5 | **Red zone starts at the maximum permissible engine speed**. | **Cited**, [911 Turbo / GT3 RS quick start guide](https://contentful-assets.porsche.com/u1ih0u5p8hpf/78zUvSYKKVeAm5ec7uOwKX/e9722f362f934da2e0b02318f76e671a/Porsche_MY25_911turbo_GT3RS_QSG.pdf) (cover says model year 2023) | Red band from `max_rpm` to the scale top. See open question 1: the brief's "red zone ends at 12 o'clock" runs against this. |
| K6 | **A digital window for speed, selector position and the engaged gear** (callout C "Digital Speedometer", next to B "Tachometer"). | **Cited**, quick start guide | Gear (rank 1) and tabular speed (rank 2) in the dial centre, the "planet". |
| K7 | **Shift assistant: two yellow bars beside the rev counter that fill as revs rise and flash blue at the best shift speed**; also "a motorsport-derived shift light". | **Cited**, press kit; [GT3 RS world premiere release](https://newsroom.porsche.com/en/2022/products/porsche-911-gt3-rs-world-premiere-29177.html); PAG PDF above. Bar shape, segment count, start rpm and flash rate are *unconfirmed* in every source found. | Two thin yellow bars beside the dial that fill with the revs and stay steady (departure D3). The flash goes to the docking-light strip. |
| K8 | **Track Screen**: one button strips both side screens to tyre pressure, oil pressure, oil temperature, water temperature and fuel level (the 992.2 GT3 adds tyre temperature). An owner reports the choice is stored with the drive mode. | **Cited**, press kit; premiere release; [992.2 GT3 cockpit kit](https://newsroom.porsche.com/en/press-kits/911-GT3/Cockpit-and-interior.html); [Rennlist owner thread](https://rennlist.com/forums/992-gt3-and-gt2rs-forum/1428063-gt3rs-suspension-programming-gt3-update.html) (caveat from the verify pass: that thread is RS software on a GT3) | The idea kept: side parts carry only what a driver needs at speed. Fuel and tyre temperature are decoded and can stay; tyre pressure, oil and water are absent from the packet and get replacements (section 5). |
| K9 | **The turned scale (992.2)**: "Depending on the display mode, the 9,000-rpm rev limit can be found in the traditional position or at the 12 o'clock mark." The rev counter is "especially easy to read thanks to its distinctive colour scheme". | **Cited**, 992.2 GT3 cockpit kit (wording from the verify pass) | The dash's founding idea: the top of the climb at 12 o'clock, the apogee. |
| K10 | **Four rotary controls and a DRS button on the wheel**; pressing a control opens its page as a graphic in the cluster. PASM top left, red ring (rebound and compression, front and rear, -4 to +4, default 0); PTV Plus lower left, blue (Coast and Power, -4 to +4); Drive Mode top right (Normal, Sport, Track); ESC/TC lower right, yellow ring (ESC On, Dynamic, Off; TC 4 to 7, 1 to 7, 0). Adjustments need Track mode. Jalopnik counts eight steps where Porsche's range gives nine. | **Cited**, press kit; [Jalopnik first drive](https://jalopnik.com/2023-porsche-911-gt3-rs-first-drive-992-generation-1849616412); [Jalopnik features explained](https://jalopnik.com/2023-porsche-911-gt3-rs-performance-features-explained-1849612711); [Rennlist how-to](https://www.rennlist.com/how-tos/slideshows/6-electronic-features-that-make-the-992-gt3-rs-extra-special-927999); evo | The four-corner arrangement and the "page appears when used" behaviour, rebuilt as event pages (section 5, idea R6). |
| K11 | **DRS**: opens by button, or by itself "among other things" above 62 mph (100 km/h) with more than 95 % accelerator travel; Jalopnik adds revs above 5,500 rpm and an exit above 0.9 g cornering; a second press or braking closes it; the wing flattens in about 0.3 s ("three-tenths of a second", evo); the swan-neck wing has "an operating range of 34 degrees, from low drag to full airbrake deployment" (evo). **Airbrake**: under heavy braking from high speed both wings go to maximum. Low downforce is the default. | **Cited**, press kit; Jalopnik features page; evo | Not in the packet. Replaced by a labelled MODEL "low-drag window" (section 5, R7). |
| K12 | **Yellow top-centre marking on the wheel**, "a visual indication of how far from center". | **Cited**, press kit | A yellow 12 o'clock index on the rev-counter ring, the apogee mark (Our reading). |
| K13 | Accent colours: red, yellow or blue inserts (Top Gear); "GT3 RS" script on headrests and sills; black leather, Race-Tex, satin carbon. | **Cited**, [Top Gear interior](https://topgear.com/car-reviews/porsche/911-gt3-rs/interior); press kit | Palette roles only; the crest comes from the logo zone. No literal colour in `draw()`. |
| K14 | Downforce 409 kg at 200 km/h and 860 kg at 285 km/h; the upper wing edge is higher than the roof. | **Cited**, premiere release | Not drawn as data (section 5, R8 is optional and MODEL). |

### 1.2 From spaceflight (the "out of this world" half)

| # | Feature | Source |
|---|---|---|
| S1 | The Apollo **FDAI** shows attitude "by rotating a ball", black on one side only (the "8-ball"), with **red circular regions at the poles** for gimbal lock, **three yellow needles** that say how to manoeuvre, and three short rate pointers on outer scales. "ROLL RATE", "PITCH RATE" and "YAW RATE" are printed in white beside the rate scales; the LM dial has red bands left and right; the LM unit is octagonal, the CM unit round; lighting is electroluminescent and an "OFF" flag shows loss of power. | **Cited**, [Shirriff, Inside the Apollo FDAI](https://www.righto.com/2025/06/inside-apollo-fdai.html) (re-opened 2026-11-28 for its photo list); [SPACE1 FDAI](https://space1.com/Artifacts/Apollo_Artifacts/FDAI/fdai.html); [NASSP CSM FDAI](https://nassp.space/index.php/CSM_FDAI) |
| S2 | The ball's graduations (line spacing, numerals), the colour of the non-black half and the edge on which each needle sits are **not** in any opened source. A related page speaks of a red roll warning zone "between the 255 and 285 degree index lines". | **Cited** absence; [SPACE1 Shuttle ADI](https://space1.com/Artifacts/Space_Shuttle_Artifacts/Controls_-_Displays/Space_Shuttle_ADI/space_shuttle_adi.html). Step 1 studies the photos in the Drive folder before the ball's texture is made. |
| S3 | Crew Dragon's docking page: a central reticle, a **green diamond** target over the docking adapter, ROLL, PITCH, YAW, RANGE and RATE readouts; docking succeeds when every green number is below 0.2. | **Cited**, [SpaceX ISS docking simulator](https://iss-sim.spacex.com) (via the earlier notes) |
| S4 | Orbits: a ground track is a sine-like trace; an orbit diagram's craft moves fastest at perigee and slowest at apogee. | **Cited**, [FAA, Describing Orbits](https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/III.4.1.4_Describing_Orbits.pdf) |
| S5 | A Hohmann transfer is the half ellipse tangent to two circular orbits, one burn at each tangent point. | Textbook result; **Computed** checks in section 4. A source page is still to be opened (*unconfirmed* citation): [Wikipedia, Hohmann transfer orbit](https://en.wikipedia.com/wiki/Hohmann_Orbit), [ASU MMAE 441 lecture 20](https://control.asu.edu/Classes/MMAE441/Spacecraft/441Lecture20.pdf). |

### 1.3 Told apart from `PorscheAnalog` (finished) and the Supercar siblings

`PorscheAnalog` already owns: five overlapping tubes, a hub cap and lacquer needle, a turned scale with each car's shift rpm at 12, eleven smoked lenses in the bezel within 65° of 12, a "+" beside the gear, a G tube with a felt-force plot, power and boost tubes, Barlow for every figure, and a Heritage option ([its RESEARCH.md](../porsche_analog/RESEARCH.md)). **Our reading** of what keeps Apogee apart: no tubes and no hub; the needle is a craft on the rim; shift light is a straight docking strip above the dial, not lenses in the bezel; the G display is a 3D ball, not a plot; the right part is an orbit diagram, not a gauge; type is Saira Condensed with B612 Mono micro-labels; materials are flight hardware (titanium, gold foil, ceramic tile), not watch finishes. Against `AmgMbuxSupersport` (one round rev counter between torque and power wings) the side parts here are round instruments with physical depth, not flat wings.

### 1.4 Departures (each kept for the handover)

- **D1** No five tubes and no hub: the car's two side screens become two instruments, and the dash does not repeat `PorscheAnalog`.
- **D2** The needle is a small craft riding the rim of a hubless ring.
- **D3** The shift-assistant bars stay steady yellow; the car flashes them blue. House rule: only the shift strip strobes.
- **D4** A straight strip of 12 docking lights above the dial carries the shift stages and the limiter strobe; the car has no such strip.
- **D5** Tyre pressure, oil pressure and temperature, water temperature, the wheel-dial pages, DRS and aero status, the clock and outside temperature are dropped (absent from the 88 fields) and replaced (section 5); fuel, tyre temperature and the lap timer are kept from decoded fields.
- **D6** Type is Saira Condensed and B612 Mono, not a Porsche face; the user's font swap replaces them.
- **D7** Colours come from palette roles; the yellow of the bars and the 12 o'clock index is a role (proposal: `caution` or `accent`, open question 4).

## 2. Reference photos

The image bytes could not be fetched (the shared fetch budget was exhausted and the cloud shell's curl is blocked). Rows 1 to 5 are direct image URLs that the PC fetches into the Drive folder later; the rest are pages, with the figure to take named in `SOURCES.md`.

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 1 | Apollo LM panel with its two octagonal FDAIs | https://static.righto.com/images/fdai/lm-panel.jpg | no |
| 2 | An Apollo FDAI with the case removed: ball, needles | https://static.righto.com/images/fdai/fdai-opened.jpg | no |
| 3 | FDAI dial close-up with traces of the red paint round the markings | https://static.righto.com/images/fdai/repaint.jpg | no |
| 4 | FDAI interior with the amplifier boards out (ball mechanism) | https://static.righto.com/images/fdai/fdai-opened2.jpg | no |
| 5 | Inside the ball, labelled parts | https://static.righto.com/images/fdai/ball-labeled.jpg | no |
| 6 | GT3 RS premiere gallery: cockpit, rev counter, wheel | https://newsroom.porsche.com/en/2022/products/porsche-911-gt3-rs-world-premiere-29177.html | no |
| 7 | 992.1 rev counter press close-up (dial geometry method) | https://newsroom.porsche.com/en/2019/products/porsche-911-rev-counter-analogue-eight-generations-992-timeless-machine-valencia-16966.html | no |
| 8 | 992.2 GT3 cockpit: turned rev counter, Track Screen, shift flash | https://newsroom.porsche.com/en/press-kits/911-GT3/Cockpit-and-interior.html | no |
| 9 | Owner photos of the RS cluster pages (damping, PTV+, ESC/TC, Aero line) | https://rennlist.com/forums/992-gt3-and-gt2rs-forum/1428063-gt3rs-suspension-programming-gt3-update.html | no |
| 10 | Owner photos of 992 clusters in use | https://rennlist.com/forums/992/1409631-whats-your-instrument-cluster-looks-like-3.html | no |
| 11 | GT3 RS interior and wheel dials in a review | https://www.carexpert.com.au/porsche/911/2024/gt3-rs/interior | no |
| 12 | Crew Dragon docking interface (reticle, green diamond) | https://iss-sim.spacex.com | no |
| 13 | Orbit figures 4.1.4-2 to 4.1.4-26 (elements, ground tracks) | https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/III.4.1.4_Describing_Orbits.pdf | no |

Drive `references` folder: https://drive.google.com/drive/folders/16naZFxXDjTUBi4VoWu70HszTcB5IiYIR (`FH6 WheelDash v2 - 4K Masters\gt3_rs_apogee\references\`, holds `SOURCES.md`; no image files yet).

## 3. How the real thing behaves, with motion notes

| State | The real car (with source) | Motion note for the dash (Our reading, token values from `data/design_tokens.json`) |
|---|---|---|
| Start-up, greeting | The 992 greets by driver profile ("Hallo FAHRER 1", **Cited** in PorscheAnalog item 16). A needle sweep on the GT3 RS is *unconfirmed*. | Family sweep 350 ms up, 300 ms down on the first race start only; the craft flies once round the rim, the ball rights itself on `gBall`. |
| Idle | Needle at idle; side screens static. | Craft at the idle angle; the transfer ellipse is drawn faint with the craft at perigee; nothing else moves (the ball holds level at 0 G). |
| Cruise | Track Screen values static unless they change. | Craft on `needle` spring (T 0.07, ζ 0.75, settles to 2 % in 63 ms, **Computed**); star-field drift only with speed. |
| Pull to the limiter | Yellow bars fill with the revs (**Cited**). DRS may open above 100 km/h at > 95 % throttle (**Cited**). | Bars fill; docking lights fill outside-in on the tokens' stages (0.75 `go`, 0.85 `caution`, 0.93 `warning`, 1.00 `shift`, hysteresis 120 rpm); the craft climbs the ellipse and dims toward apogee by vis-viva. |
| Shift point and limiter | Bars flash blue at the best shift speed (**Cited**); the limiter cuts at 9,000 (**Cited**). | Only the docking strip strobes, 10 Hz at 50 % (3 on / 3 off at 60 fps), release ≤ 50 ms; the apogee marker holds steady `shift`. |
| Gear change | PDK; the digital window updates the gear (**Cited**). Timing of the digit change *unconfirmed*. | Outgoing gear 50 ms, scale to 0.92 with fade; incoming from 1.12, slide 24 px, on `fast`. The ellipse-closing animation is section 7. |
| Braking | DRS closes on braking; airbrake sets both wings to maximum under heavy braking from high speed (**Cited**). | Ball pitches nose-down with `g_long` on `gBall`; retro-burn idea R7b. |
| Drift, lateral G | ESC/TC intervene by setting (**Cited** ranges); DRS exits above 0.9 g (Jalopnik). | Ball rolls with `g_lat`; prograde marker slides by β; the nose marker stays centred. |
| Reverse, neutral | Selector position shown in the window (**Cited**). | R or N in the centre; the plot shows the gear-1 circle alone and no ellipse; in R the craft parks at the idle angle and the ball keeps working. |
| Shut-down | *Unconfirmed*. | Paused or stale stream: the app hides the dash, so no shut-down animation is drawn. |

## 4. The maths

The catalog row ("Kepler orbits") is checked and holds. Constants and the computed values the unit tests can assert:

- **Ellipse from its focus** (the planet at the gear's centre): r(θ) = a(1 − e²) ÷ (1 + e cos θ), a = (r₁ + r₂) ÷ 2, e = (r₂ − r₁) ÷ (r₂ + r₁). **Computed** with r₁ = 100, r₂ = 140: e = 1/6, r(0) = 100.000, r(π) = 140.000, r(π/2) = 116.667, which equals the semi-latus rectum 2r₁r₂ ÷ (r₁ + r₂). Semi-minor axis b = √(r₁r₂) = 118.322. With r₁ = 60, r₂ = 120: e = 1/3, r(π/2) = 80.
- **Vis-viva**: v ∝ √(2/r − 1/a). **Computed**: v(r₁) ÷ v(r₂) = r₂ ÷ r₁ exactly (1.4 for 100 and 140; 2.0 for 60 and 120), the catalog's test, and also what angular momentum conservation gives (r₁v₁ = r₂v₂). Normalised brightness along the 100/140 arc at θ = 0, π/4, π/2, 3π/4, π: 1.0000, 0.9635, 0.8690, 0.7628, 0.7143.
- **The orbit radii are the learned speed per rpm** k_n = speed ÷ rpm in gear n. Then r₂ ÷ r₁ = k_{n+1} ÷ k_n = q, the gear step, and **the rpm drop on the upshift is exactly 1/q**, so the ellipse's perigee-to-apogee speed ratio equals the rpm ratio across the shift (**Computed**, Our reading: this is the physical reason the metaphor works). Eccentricity e = (q − 1) ÷ (q + 1): **Computed** for q = 1.15, 1.20, 1.30, 1.40, 1.58: e = 0.0698, 0.0909, 0.1304, 0.1667, 0.2248. Drawn with the next orbit at 140 px, the gap between the two circles is 18.3, 23.3, 32.3, 40.0 and 51.4 px. The GT3 RS's own seven ratios were not found in an opened source (*unconfirmed*; step 1 reads Porsche's technical data). evo says the RS has "shortened ratios and final drive" against the GT3 (**Cited**, evo review); auto motor und sport's test data give 386 kW at 8,500 rpm and a 7-speed dual-clutch box but no ratios ([ams Supertest data](https://www.auto-motor-und-sport.de/test/porsche-911-992-gt3-rs-supertest-nordschleife-schnellster-porsche/technische-daten/)). In the game the ratios are whatever the driven car's tune sets, so the dash learns them live anyway.
- **Correction to the brief's true anomaly.** The brief sets θ = π × (rpm − idle) ÷ (shift − idle). After an upshift the engine lands at shift ÷ q, not at idle, so the craft would restart high on the new ellipse. **Computed** with a 9,000 rpm limit, the default shift point 0.97 (8,730 rpm), idle 900: landing rpm 7,591, 7,275, 6,715, 6,236 and 5,525 for the five q above, which puts the craft at θ = 0.855π, 0.814π, 0.743π, 0.681π and 0.591π the moment the new ellipse opens. **Recommended fix:** θ = π × (rpm − r_entry,n) ÷ (shift − r_entry,n), with r_entry,1 = idle and r_entry,n = shift × k_{n−1} ÷ k_n once both are learned, clamped to [0, π]. It is still exact and monotonic in rpm, every pull starts at perigee and ends at apogee, and the extra test is "θ = 0 at the learned landing rpm". The catalog row stands; the brief's Telemetry line needs this change (the lead reports it).
- **The `fast` spring cannot make 300 ms.** **Computed** from `SpringSpec::omega` = 2π ÷ T: `fast` (T 0.168, ζ 0.9) settles to 2 % in 0.126 s with 0.15 % overshoot; `default` 0.177 s; `expressive` 0.192 s; `slow` (T 0.363) 0.272 s. The brief's "300 ms on the `fast` spring" is one or the other: either 300 ms on `slow`, or about 130 ms on `fast` (section 7 recommends the second, so the plot keeps pace with the gear glyph, which also uses `fast`).
- **Turned scale.** With a fixed 240° sweep, the scale top T (first whole thousand at least 500 above the limiter, as PorscheAnalog change G) and the shift rpm at 12, the angle clockwise from 12 is 240° × (rpm − shift) ÷ T. **Computed** for 9,000 / 8,730 / T = 10,000 (24° per 1,000): 0 rpm at −209.5° (about 5 o'clock), idle 900 at −187.9°, 5,000 at −89.5° (9 o'clock), the limiter at +6.5° and the top at +30.5° (1 o'clock). Since shift ÷ T is about 0.87 for most cars, the zero lands near 5 o'clock on almost every car, which matches the 992.2's turned Carrera GTS dial where 0 sits near 5 o'clock (PorscheAnalog item 10).

## 5. Telemetry mapping

Telemetry truth as of main `60c99c9` (2026-11-28 10:51): the decoder reads all 88 FH6 Data Out fields. **Verified** means long-verified against the golden packets (rpm, speed, gear, throttle, brake, steer, acceleration, local velocity, power, boost, timestamp). **Decoded, offset confirmed at check C** means the field is read (`crates/fh6-core/src/telemetry.rs`) but its offset is still to be confirmed in the real game; each such row names a cheap fallback from verified fields.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Rim craft (the needle) | `rpm`, `max_rpm`, `shift_rpm` (`Signals`) | verified | Angle from section 4 on the `needle` spring with `rpm_rate` feed-forward. |
| Red band, scale numerals | `max_rpm` | verified | Red from `max_rpm` to the top; numerals every 1,000 (2,000 above 12,000). |
| Docking lights (12, outside-in) | `shift_fraction` | verified | Tokens' stages and 120 rpm hysteresis; strobe at the limiter only. |
| Two yellow assist bars | `rpm`, learned `r_entry` | verified + derived | Fill = (rpm − r_entry,n) ÷ (shift − r_entry,n), the same fraction as θ ÷ π. |
| Gear numeral, active orbit | `gear` | verified | R, N, 1 to 10. |
| Tabular speed | `speed`, `unit` | verified | Integer, tabular figures, unit label in B612 Mono. |
| Orbit radii, "speed at shift" | k_n = `speed_ms` ÷ `rpm`, learned into `[f32; 10]` | derived | Learn only with clutch 0, throttle > 0.2, \|β\| < 3°, rpm steady; speed at shift = k_n × shift_rpm in the user's unit. |
| Craft on the ellipse, arc brightness | `rpm`, k table | derived | θ from section 4; brightness by vis-viva. |
| Attitude ball pitch and roll | `g_long`, `g_lat` | verified (acceleration) | Felt force on `gBall` (T 0.168, ζ 0.9); proposal 1 g = 30° of tilt, clamped at 60°. Option for the board: true body `pitch` and `roll` (decoded, check C), which on a flat circuit stay within a few degrees, so felt force stays the recommended driver. |
| **Three FDAI rate pointers** (ROLL RATE, PITCH RATE, YAW RATE on outer scales, as S1) | `angular_velocity_z`, `angular_velocity_x`, `angular_velocity_y` | decoded, check C (`yaw_rate` is already in `Signals`) | Full scale switchable like the Apollo 5 or 25 °/s switch (S1); proposal ±60 °/s for yaw, ±25 °/s for roll and pitch. Fallback for yaw: a_lat ÷ v (steady-state turn), labelled MODEL. This is the dash's most faithful FDAI part. |
| Prograde marker on the ball | `slip_deg` (β = atan2(v_x, v_z), faded below 3 m/s) | derived from verified velocity | Marker at azimuth β on the ball's equator, x = R sin β; the nose marker stays fixed. |
| Plume behind the craft | `throttle` | verified | Length and brightness; rank 4 luminance cap. |
| Retro-burn flare (R7b) | `brake`, `speed` | verified | Forward-pointing plume while braking above 150 km/h. |
| Mission elapsed time | `current_race_time` | decoded, check C | T+ mm:ss.s; fallback: timer from `race_on` and the timestamp. |
| Lap timer (the Chrono stopwatch's place) | `current_lap`, `last_lap`, `best_lap`, `lap_number` | decoded, check C | Current lap and the delta to best with a sign and `go` or `warning` role; hidden when the game reports no laps (free roam). |
| Tyre temperatures (Track Screen, 992.2 GT3) | `tire_temp_fl/fr/rl/rr` | decoded, check C | Four corner readouts or four ticks on the ball's cage; unit as the game sends it (step 1 confirms °F or °C). |
| Fuel level (Track Screen) | `fuel` (0 to 1) | decoded, check C | A "propellant" bar beside the plot; hidden when the event has no fuel use (constant 1). |
| Load-quadrant thrusters (R2) | `tire_combined_slip_*`, fallback felt force | decoded, check C | Four RCS pods round the ball brighten with each wheel's combined slip; fallback: sign and size of `g_lat` and `g_long`. |
| Torque on the orbit (optional) | `torque`, fallback power × 60 ÷ (2π rpm) | decoded, check C | Only if the board wants a power readout (open question 6). |
| Engine type for edge cases | `num_cylinders`, `drivetrain_type` | decoded, check C | 0 cylinders marks an EV (section 6). |
| Crest | `car_ordinal` → logo | decoded | Logo zone, rank 5. |

### What the game cannot send

Only what is absent from the 88 fields. Each gets a replacement the telemetry drives.

1. **Tyre pressure** (Track Screen). **R2a (recommended) Pressure slot becomes tyre temperature** (decoded), the item the 992.2 GT3 Track Screen added itself (K8). Second: the load-quadrant thrusters (R2).
2. **Oil pressure, oil temperature, water temperature.** **R3 (recommended) Heat-shield glow:** the ceramic-tile side parts warm at their leading edge with braking deceleration (|g_long| while `brake` > 0) and cool on a fixed time constant, labelled MODEL. Second: **burn timer**, seconds at full throttle in the current pull.
3. **Clock and outside temperature** (Speed & Assist). **R1 (recommended) Mission elapsed time** from `current_race_time`. Second: a **0 to 100 km/h (or 0 to 60 mph) launch timer** from speed and the timestamp.
4. **Assistance systems, ADAS, navigation.** **R5 (recommended) Star-field drift:** the faint star plate behind the parts streams by speed through a small parallax, the dash's "clouds and horizon". Second: nothing; the space stays black.
5. **The four wheel-dial pages (PASM, PTV+, Drive Mode, ESC/TC).** **R6 (recommended) Event pages:** as the car shows a page when a dial is turned, a small page opens for 2 s when an event happens: a drift page (peak β) when \|β\| > 10°, a launch page (0 to 100 time), a braking page (peak decel G), a lap page (lap time and delta). Second: four rank-5 corner readouts of session peaks.
6. **DRS and aero state ("Aero : Low").** **R7 (recommended) Low-drag window, MODEL:** a small "LOW DRAG" cue when the published conditions are met (speed > 100 km/h, throttle > 95 %, rpm > 5,500, \|g_lat\| < 0.9), worded as conditions met, never as the wing's state. **R7b** retro-burn flare for the airbrake condition (hard braking from high speed).
7. **Fuel range estimate.** **R4 (recommended) Δv to shift:** speed at shift minus current speed, the "propellant" left in this gear, exact once k_n is learned ("Δv 23 km/h"). The fuel bar itself is decoded and shown as above.
8. **Selector P, warnings and telltales.** R and N come from `gear`; no P, no telltales.
9. **Downforce (K14), optional.** **R8 MODEL** aero load scaled from Porsche's 409 kg at 200 km/h as 409 × (v ÷ 200)², only on the GT3 RS (ordinal 3781). **Computed**: it predicts 830.5 kg at 285 km/h against Porsche's 860 kg (ratio 2.031 against 2.103), because the active aero adds load. Not recommended for the first board.

## 6. Edge cases

- **`max_rpm` 0 or tiny:** `Signals` floors it at 1; below 1,500 the dash draws the ring without numerals, no ellipse, and the docking lights off.
- **Idle near max:** if shift − idle < 500 rpm, θ and the bars use the whole range from 0; the plot shows circles only.
- **12,000+ rpm engines:** numerals every 2,000; the sweep stays 240°. **Diesels (about 4,500):** top 5,000, numerals every 1,000, the same angles because shift ÷ top is similar.
- **EVs (one gear, no idle, odd rpm):** detected by `num_cylinders` = 0 (decoded, check C) or by a single learned gear; one orbit, no next gear, so no transfer ellipse: the craft circles the gear-1 orbit by rpm and the plot reads "SINGLE STAGE".
- **Reverse and neutral:** section 3.
- **Gear above 10:** `Gear::from_raw` maps it to N; the k table holds 10.
- **Unlearned gears:** the next orbit is a dashed circle labelled "–" until k_{n+1} is measured; nothing is guessed.
- **mph and km/h, hp and kW:** speed, Δv, speed at shift and the launch timer follow `unit`; no power readout is planned (open question 6).
- **Limiter bounce (about 50 rpm a frame):** the craft holds on its spring; docking-light hysteresis 120 rpm; digits refresh at 5 Hz.
- **Spikes and crashes:** the ball clamps at 60° tilt and the `gBall` spring rides out a single-frame spike; peaks ignore samples over 6 g.
- **Airborne:** G near zero, the ball levels; k_n is not learned (wheel speed is meaningless).
- **Stale or paused stream:** the app hides the dash.
- **Decoded fields not yet confirmed at check C:** every row so marked in section 5 falls back to its verified-field fallback, or hides, if its value is NaN, out of range (tyre temperature outside 0 to 400, fuel outside 0 to 1, rates over 10 rad/s) or frozen while the car moves.
- **No laps, no fuel use:** lap timer and fuel bar hide when `lap_number` and `current_lap` stay 0 or `fuel` stays at 1; their slots fall back to mission elapsed time and Δv to shift.
- **Palettes:** the ball's black half is true black, its light half capped at rank-4 luminance; a random palette with the shift role near the docking-light `go` colour still changes stage by position (outside-in), not by colour alone.
- **Font swap:** the gear and speed take the user's display font; B612 Mono micro-labels stay mono.
- **Cockpit only:** no hood variant listed.
- **Reduced-flash mode:** the docking strip breathes at 2 Hz (sine, 40 % depth).
- **Colour-vision:** stages read by how many lights are lit; the prograde marker differs from the nose marker by shape (circle with three ticks against a fixed wing bar), not colour.

## 7. Signature moments

**Limiter strobe (only the docking strip strobes):**
- **A (recommended) Docking capture.** The 12 lights fill outside-in like an approach, the last two at the centre lock in `shift` at the shift point, and at the limiter the whole strip strobes at 10 Hz, 50 % duty; the apogee marker stays steady `shift`. The tokens' default `shift` hue (#978DFC, a lifted violet-blue) is the nearest role to the car's blue flash.
- **B** Centre-pair strobe only (two lights), the rest held steady: calmer, still inside the 42 px strip rule.
- Gentle mode: 2 Hz sine breath at 40 % depth for either.

**Gear change (the transfer burn):**
- **A (recommended) Circularise and open.** On the upshift the ellipse closes into the next gear's circle on `fast` (about 130 ms, matching the gear glyph), the old inner circle fades, and the next ellipse opens outward from the new circle; with the θ fix of section 4 the craft is at perigee as it opens.
- **B** Two-burn flash: one-frame plume flare at 60 % (the tokens' `rimFlash`) at the apogee burn, and a second at perigee when the craft re-enters, the two Hohmann burns.
- **C** Downshift as the inverse transfer: the ellipse opens inward and the craft drops from apogee to the lower circle.

## 8. What it needs from the 4K library

The pre-build kit (PR #18 branch `claude/project-thread-ib67x9`, `assets/clusters/gt3_rs_apogee/prebuild/manifest.json`):
- `rev_counter_ring_500`: the Ø500 machined titanium ring, no hub, 22 px width proposed; three keys (rest, ±25° G swing). Step 1 confirms the width.
- `docking_light_off` and `docking_light_on`: 30 px caps for the 760 × 30 strip.
- `mli_foil_crinkle_height`: 4096² seamless height map for the gold-foil dial backing (the dash shades it with metal GOLD).
- `star_field_tile`: 4096² seamless faint stars, the reflection environment and the R5 drift plate.
- `heat_shield_tile_coating`: computed ceramic coating master for the side parts (the tile grid is geometry).
- Deferred by the kit: the attitude ball and its cage.

Shared parts it can reuse: `polished_steel_matcap` (PaganiHorologerie) for fasteners and the cage pivots; `chrome_matcap` (KineticLiquidMetal) for bright metal; `spun_titanium` swatch (LfaV10Nurburgring) for the ring's grain if re-rendered. Fonts are TTF in `assets/fonts/` and MSDF glyphs are generated at load (`fh6-kit/src/text.rs`), so no atlas is needed.

LIBRARY GAP: attitude ball sphere, Ø260 at 4K, about 2,000 triangles as glTF for the kit's mesh pass plus an equirectangular 4096 × 2048 albedo (black hemisphere, light-grey hemisphere at rank-4 luminance, graduation lines and red pole caps settled from photos 1 to 3) and a sprite fallback at three tilts; Blender.
LIBRARY GAP: gimbal cage for the ball, two thin titanium rings with pivots round a Ø260 ball (outer about Ø300, proposed), sprite keys at rest and ±25°; Blender.
LIBRARY GAP: the rim craft, about 36 px (proposed) small craft with an engine bell, at the rest key and two swing keys, for the hubless ring; Blender.
LIBRARY GAP: gold matcap and Fresnel table computed from gold's optical constants (as `chrome_matcap` was from chromium's), so the foil backing reads as gold, not yellow paint; asset-forge `prebuild/liquid.rs` route (CPU, exact).
LIBRARY GAP: safety-wired fastener sprite (the brief's flight-hardware ring), about 18 px, rest and swing keys; Blender.

## 9. Departures and open questions for the board

1. **Where the red zone sits.** The brief says the red zone ends at 12 o'clock; the car's red zone starts at the maximum permissible speed (K5) and the turned 992.2 dial starts its red band at 12 (PorscheAnalog item 10). With shift at 12, the limiter lands only +6.5° past 12 on a 9,000 rpm car (section 4). Proposal: shift at 12 (apogee), red from the limiter clockwise beyond it.
2. **θ mapping:** adopt the landing-rpm θ of section 4, or keep the brief's idle-based θ and accept a craft that restarts high on each new ellipse.
3. **Gear-change spring:** `fast` (about 130 ms) or a true 300 ms on `slow`.
4. **Yellow role** for the assist bars and the 12 o'clock index: `caution` or a family `accent`; must stay 25° of hue from `warning`.
5. **Classic face option** with 5,000 at 12 (K2), as the 992.1 RS; it breaks the apogee idea, so offer it only as an option or not at all.
6. **Power or boost:** the brief has none; the right part could show power on the orbit if the board wants it.
7. **R8 aero load:** keep off the first board.
8. **The transfer plot must not read as navigation:** no ground, no map lines, no lat/long, as the brief's risk line says.
9. **Rate pointers.** The brief's Telemetry line says packet pitch, roll and yaw are not used; with all 88 fields decoded (main `60c99c9`) the three FDAI rate pointers can run on the angular velocities once check C confirms them. Proposal: add them, with the a_lat ÷ v yaw fallback; the brief's line needs updating (the lead reports it).

## 10. Sources

**Porsche:** [GT3 RS press kit (PCNA PDF)](https://newsroom.porsche.com/dam/jcr:46a23375-e7ee-4507-a577-d9761b784d33/992%20911%20GT3%20RS%20Press%20Kit%201.pdf) · [GT3 RS world premiere](https://newsroom.porsche.com/en/2022/products/porsche-911-gt3-rs-world-premiere-29177.html) · [911 Turbo / GT3 RS quick start guide](https://contentful-assets.porsche.com/u1ih0u5p8hpf/78zUvSYKKVeAm5ec7uOwKX/e9722f362f934da2e0b02318f76e671a/Porsche_MY25_911turbo_GT3RS_QSG.pdf) · [PAG-911-GT3-RS-EN.pdf (992.1 GT3 figures)](https://newsroom.porsche.com/dam/jcr:fe2fb6d3-d0e3-44d8-8f2d-d82a66674718/PAG-911-GT3-RS-EN.pdf) · [992.2 GT3 cockpit and interior](https://newsroom.porsche.com/en/press-kits/911-GT3/Cockpit-and-interior.html) · [Newsroom 911 page](https://newsroom.porsche.com/en/products/911.html) · [What's the clock? (992 rev counter)](https://newsroom.porsche.com/en/2019/products/porsche-911-rev-counter-analogue-eight-generations-992-timeless-machine-valencia-16966.html)

**Reviews and owners:** [evo GT3 RS review (26 Nov 2025)](https://www.evo.co.uk/porsche/911/gt3-rs) · [ams Supertest technical data](https://www.auto-motor-und-sport.de/test/porsche-911-992-gt3-rs-supertest-nordschleife-schnellster-porsche/technische-daten/) · [evo GT3 RS interior and tech](https://www.evo.co.uk/porsche/911/gt3-rs/interior-and-tech) · [evo 992.2 GT3 RS test mule](https://www.evo.co.uk/porsche/911-gt3-rs/207050/this-9922-porsche-911-gt3-rs-test-mule-might-be-hiding-a-secret) · [evo 992.2 GT3 RS update](https://www.evo.co.uk/porsche/911-gt3-rs/207050/the-porsche-911-gt3-rs-is-about-to-get-even-better-updated-9922-version) · [Jalopnik first drive](https://jalopnik.com/2023-porsche-911-gt3-rs-first-drive-992-generation-1849616412) · [Jalopnik features explained](https://jalopnik.com/2023-porsche-911-gt3-rs-performance-features-explained-1849612711) · [Rennlist electronic features](https://www.rennlist.com/how-tos/slideshows/6-electronic-features-that-make-the-992-gt3-rs-extra-special-927999) · [Rennlist RS cluster software thread](https://rennlist.com/forums/992-gt3-and-gt2rs-forum/1428063-gt3rs-suspension-programming-gt3-update.html) · [Rennlist cluster photos](https://rennlist.com/forums/992/1409631-whats-your-instrument-cluster-looks-like-3.html) · [Top Gear interior](https://topgear.com/car-reviews/porsche/911-gt3-rs/interior) · [CarExpert interior](https://www.carexpert.com.au/porsche/911/2024/gt3-rs/interior)

**Spaceflight and orbits:** [Shirriff, Inside the Apollo FDAI](https://www.righto.com/2025/06/inside-apollo-fdai.html) · [SPACE1 FDAI](https://space1.com/Artifacts/Apollo_Artifacts/FDAI/fdai.html) · [SPACE1 Shuttle ADI](https://space1.com/Artifacts/Space_Shuttle_Artifacts/Controls_-_Displays/Space_Shuttle_ADI/space_shuttle_adi.html) · [NASSP CSM FDAI](https://nassp.space/index.php/CSM_FDAI) · [SpaceX docking simulator](https://iss-sim.spacex.com) · [FAA, Describing Orbits](https://www.faa.gov/sites/faa.gov/files/about/office_org/headquarters_offices/avs/III.4.1.4_Describing_Orbits.pdf) · [Wikipedia, Hohmann transfer orbit](https://en.wikipedia.com/wiki/Hohmann_Orbit) (not opened) · [ASU MMAE 441 lecture 20](https://control.asu.edu/Classes/MMAE441/Spacecraft/441Lecture20.pdf) (not opened)

**Repo:** `program/phases/07-cluster-factory/CATALOG.md` (brief, maths row) · `crates/fh6-clusters/src/clusters/porsche_analog/RESEARCH.md` · `docs/design/families/classic-analog.md` · `data/design_tokens.json` · `crates/fh6-kit/src/cluster.rs` (`Signals`) · `crates/fh6-kit/src/anim.rs` (springs) · PR #18 `PREBUILD.md`, `kits.rs`, `assets/clusters/gt3_rs_apogee/prebuild/manifest.json`
