# GtrNismoMfd: research note

Build #30, catalog row 58, category Retro, Target `inspired`, family Modern Digital 2027, view cockpit. The real things are the Nissan GT-R (R35) combination meter, called "mesh-gear design gauges" by Nissan, and the Multi Function Meter (MFM) on the centre display, whose graphics were made with Polyphony Digital; the game cars are the 2020 and 2024 GT-R NISMO. Pre-researched 2026-11-28 by the thread "Research the upcoming dashes ahead" before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read that day from the linked source; **Seen** is a kind of Cited, read off a photograph at the linked URL (colours from photos of lit dials and LCDs are approximate); **Computed** shows its inputs and numbers (`python3 -I`, numpy); **Our reading** is a conclusion drawn for this dash. "Unconfirmed" names what step 1 must still check. Source keys (NIS-JP, CW-MFM and so on) are listed in section 10.

**How it differs from `SkylineR34Mfd`** (its note on `origin/claude/project-thread-tenxez`): the R34 is black faces with white *italic* numerals, crimson or orange-red needles, a navy MFD with green flat bars, salmon needles, cyan tiles and bold italic type. The R35 is upright bold numerals, a carbon-weave tach face inside a toothed "engaged gears" ring lit blue at the tooth roots, amber-orange lit needles, blue LCD windows, five pods in thick machined rings, and an MFM on true black with white upright type, needle-less arc gauges filled with a white-to-grey sector, 20 s history graphs and a red alert. Neither dash borrows the other's palette, font slant or tile language.

## 1. The real display: parts kept, measured, and departures

### 1.1 The combination meter (cluster)
| # | Fact | Label |
|---|---|---|
| C1 | "噛み合ったギヤをモチーフにしたコンビメーター": a meter whose motif is engaged gears; tachometer in the centre, a large shift (gear) indicator at its upper right; a machined-aluminium-look three-dimensional meter ring; a bold, heavy typeface ("骨太な書体") for legibility. | Cited NIS-JP |
| C2 | Five instruments, not three: speedometer (left), tachometer (centre, largest), transmission position indicator (a separate small round pod at the tach's upper right), fuel gauge and engine coolant temperature gauge (small round pods, right). The tachometer carries the upshift indicator; the vehicle information display sits in the tach. | Cited MAN09 p.81; Seen US24 |
| C3 | Tach face, 2024: dark carbon-weave print; white bold upright numerals 0 to 8 with "RPM×1000"; white bar ticks on a toothed ring whose tooth roots glow blue; red ticks from 7 to past 8. 2011: plain charcoal face with the same toothed relief, "x1000r/min". | Seen US24, CW11 |
| C4 | Tach geometry: about 25.6° per 1,000 rpm, linear, 0 at about −133° from 12 o'clock, 8,000 at about +72°, 5,000 near 12 o'clock (fit residual 3.6°; 2011 photo agrees within 3°). The red ticks run on to roughly 9,000 at about +98°, where the LCD window begins; no "9" is printed. | Computed from US24 numeral centres; scale end unconfirmed |
| C5 | The lower-right quadrant of the tach face is a blue backlit LCD: digital speed ("0 MPH", "0 km/h") large, odometer and trip (A/B) small, white digits. | Seen US24, CW11; MAN09 p.82 |
| C6 | Needles: long tapered amber-orange pointers (about #FF7A1A lit, with a paler hot core; redder about #FF4A1A in the daylight 2011 shot), no visible tail, over a large flat matte black hub cap. Fuel and temperature pods use the same needle and cap. | Seen US24, CW11 |
| C7 | Speedometer, US: 0 to 220 MPH in white outer numerals with an inner red km/h scale (20 to 340); zero sits at the lower right and the scale runs clockwise through 6, 9 and 12 o'clock to about 2 o'clock, a sweep near 320°. JDM: 340 km/h. | Seen US24 (angles approximate); 340 km/h in the VS17 image title |
| C8 | Gear pod: chrome-ringed round window, saturated blue LCD (about #1E6BFF) with one large white segmented glyph ("P"); unlit segments faintly visible. It blinks when the selected gear cannot be engaged in M. | Seen US24, CW11, CW-D; Cited MAN09 p.84 |
| C9 | Rings: thick machined aluminium rims with a bright outer chamfer and darker inner bevel, about 0.11 to 0.13 of the dial radius (25 to 30 px at Ø460). | Seen US24; Computed, approximate |
| C10 | Upshift indicator: three lamps on the tachometer, green, yellow, red. AUTO (factory default): yellow about 700 rpm before the red zone, red just before it. MANUAL: a set point from 3,000 to 6,300 rpm in 100 rpm steps; green flashes from about 500 rpm before it and lights steadily at it. Works in M only. | Cited MAN09 pp.85-86 |
| C11 | Instrument brightness is adjustable, stored separately for day and night. Meters illuminate at ignition ON; needles may move slightly after OFF. | Cited MAN09 pp.81, 87 |
| C12 | NISMO: "tachometer features a red design", carbon-like meter finish (2013); "Red-accented tachometer" (2024); Alcantara meter hood. Photos show a red tach chapter with white numerals round a dark centre, the speedometer staying black. | Cited NIS13, US24KIT; Seen NISMO24, GAW16 (low resolution; red face versus red band unconfirmed) |
| C13 | From the 2017 model every GT-R has a carbon-style meter finish. Redline 7,000 to 7,100 rpm. | Cited webcheck leads (2017 kit; Wikipedia) |
| C14 | Lighting source: a third-party "LED globe repair/upgrade" kit is sold for the R35 cluster, which suggests replaceable lamps; whether the factory lamps are incandescent or LED is unconfirmed. | Cited JJ |

### 1.2 The Multi Function Meter
| # | Fact | Label |
|---|---|---|
| M1 | 7-inch centre LCD to 2016, 8-inch wide LCD from 2017; started with the FUNCTION button; the first start after each engine start shows a "POLYPHONY DIGITAL & CLARION" logo screen. The 8-inch unit also shows the GT-R badge on dark. | Cited CW-MFM, NIS-JP17; Seen NIS-JP-MFD |
| M2 | Look: "black background, bold white numeral font, red accent colour"; designed like the car-data screen of Gran Turismo's replay. | Cited CW-MFM |
| M3 | 11 pages, named 1 2 3 4 A B C D E F G, changed by touch or a dial left of the wheel. Each page is divided into "palettes": small, and large (four small cells); six cells per page, so a page is six small, or one large plus two small (the article's "+4 small" contradicts its own screens; PRESET 1 and pages A to D show 1 large + 2 small). Only pages 1 to 4 are user-set. | Cited CW-MFM; Seen CW-P1, CW-A |
| M4 | 2012 list: water temp, engine oil temp and pressure, transmission oil temp and pressure, boost, speed, fuel/range, fuel flow, section consumption, front torque split, accel, brake, steering, cornering G, accel/braking G, absolute G, clock. Speed, torque split, accel, brake, steering and clock fit small palettes only. | Cited CW-MFM |
| M5 | Presets: page 1 boost (large) + engine oil temp + oil pressure; page 2 six small temperature and pressure gauges plus torque split. The torque split is a plain vertical bar that rises as more torque goes forward; it shows the car is mostly rear-driven. | Cited CW-MFM |
| M6 | Page A ACCELERATION: a 20 s line graph of longitudinal G, 0 to 0.5 G, time on x (20 → 0 s, now at the right), with boost and accel; the ideal trace is a smooth "Mt Fuji" hump. Page B BRAKING: the same, zero line inverted, with speed and brake. Page C STEERING: lateral G ±1 G across, time running down 0 to 20 s like an ECG, with speed and steering; above 0.5 G the trace turns red. | Cited CW-MFM, NIS-JP17 (20 s); Seen CW-A, CW-C |
| M7 | Page D GEAR POSITION ("ENG MAP INDICATOR"): six vertical tubes, one per gear, coloured by how suitable each gear is at the current speed, pale blue for good, toward red for too-high rpm, with a large segmented gear glyph and speed. | Cited CW-MFM; Seen CW-D |
| M8 | Page E FUEL ECONOMY (blocks stacked per minute over 20 min), F TIME LOGGER (stopwatch from START/STOP and MRK on the wheel; big amber digits), G DRIVERS NOTE (place names from navigation). The meter keeps measuring on hidden pages, so a history is full the moment it is shown. | Cited CW-MFM; Seen CW-F |
| M9 | Seen gauge drawing (2012 screens): arc gauges of about 222° with no needle; a white-to-grey radial fill (fine radial striping on the large one) sweeps from the scale start to the value; white tick marks, white numerals, the unit in the centre ("kgf/cm²", "%", "°C" with a big digital value); red-orange zone at the top of temperature scales. Boost: −1 to +1.5 kgf/cm² with 0 at 12 o'clock, two linear segments (−1..0 over 111°, 0..1.5 over 111°). Steering: a top arc L to R with a white arrowhead. Frames: rounded dark grey palettes with a lighter top bevel; captions white caps; page title large white caps at the bottom between round arrow buttons; round grey "coin" page tabs, the active one ringed red; clock amber top right; the current value of a graph is a yellow-green dot; red threshold lines (about 0.32 G on page A, ±0.5 G on page C). | Seen CW-P1, CW-A, CW-C, CW-D; angles Computed, approximate |
| M10 | Whether the 8-inch (2017+) MFM redrew these gauges is unconfirmed; Nissan's 2017 and 2019 to 2023 pages describe the same functions ("up to 6 items", 20 s G histories). | Cited NIS-JP17, NIS-JP19 |

**Better real target.** None needed: the R35 itself is right. But the brief's "three-dial cluster" and "three or six panels" are corrected above (C2, M3).

**Parts kept (inspired):** the toothed carbon tach with blue-lit tooth roots and machined ring, amber needles and black hubs, the blue LCD quadrant with digital speed, the round blue gear pod at the upper right, the three-colour upshift logic, the small fuel pod, and the MFM's palette grammar and gauge styles. **Departures:** section 9.

## 2. Reference photos
| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | 2024 US cluster lit, all five pods, carbon tach face, blue tooth-root glow | https://www.nissanusa.com/content/dam/Nissan/us/vehicles/gtr/2024/site-simp/gallery/interior/mosaic/2024-nissan-gtr-mesh-gear-design-gauges.jpg | https://www.nissanusa.com/vehicles/sports-cars/gt-r.html |
| 2 | 2011 tach, gear pod and fuel pod close up (toothed relief, needle, LCD) | https://asset.watch.impress.co.jp/img/car/docs/418/004/201.jpg | https://car.watch.impress.co.jp/img/car/docs/418/004/html/201.jpg.html |
| 3 | MFM PRESET 1: large boost arc, oil temp, oil pressure | https://asset.watch.impress.co.jp/img/car/docs/418/004/228.jpg | same gallery, 228.jpg.html |
| 4 | MFM page A ACCELERATION: 20 s G graph, boost and accel arcs | https://asset.watch.impress.co.jp/img/car/docs/418/004/221.jpg | 221.jpg.html |
| 5 | MFM page B BRAKING (not yet viewed) | https://asset.watch.impress.co.jp/img/car/docs/418/004/222.jpg | 222.jpg.html |
| 6 | MFM page C STEERING: vertical-time lateral G, red ±0.5 G lines | https://asset.watch.impress.co.jp/img/car/docs/418/004/223.jpg | 223.jpg.html |
| 7 | MFM page D GEAR POSITION: six gear tubes, segmented gear glyph | https://asset.watch.impress.co.jp/img/car/docs/418/004/224.jpg | 224.jpg.html |
| 8 | MFM page E fuel history (not yet viewed) | https://asset.watch.impress.co.jp/img/car/docs/418/004/225.jpg | 225.jpg.html |
| 9 | MFM customise screen (not yet viewed) | https://asset.watch.impress.co.jp/img/car/docs/418/004/229.jpg | 229.jpg.html |
| 10 | MFM page F TIME LOGGER, amber stopwatch digits, page tabs | probably https://asset.watch.impress.co.jp/img/car/docs/571/012/DSC_8213.jpg | https://car.watch.impress.co.jp/img/car/docs/571/012/html/DSC_8213.jpg.html |
| 11 | 2024 NISMO wheel with the red tach behind it | https://www.nissanusa.com/content/dam/Nissan/us/vehicles/gtr/2024/site-simp/nismo/carousel/2024-nissan-gt-r-nismo-alcantara-covered-steering-wheel.jpg | https://www.nissanusa.com/vehicles/sports-cars/gt-r/nismo.html |
| 12 | 2016 NISMO cabin, red tach face | https://images.guideautoweb.com/content/photos/14/36/143620_Nissan_GT-R_Nismo.jpg | https://www.guideautoweb.com/en/galleries/22898/2015-nissan-gt-r-and-2016-nissan-gt-r-nismo-the-latest-evolution/?im=6 |
| 13 | 2017 NISMO cockpit, Alcantara hood, MFM screen | https://asset.watch.impress.co.jp/img/car/docs/759/581/10.jpg | https://car.watch.impress.co.jp/img/car/docs/759/581/html/12.jpg.html |
| 14 | 8-inch centre display with the GT-R start badge | https://history.nissan.co.jp/GT-R/R35/1904/www-asia.nissan-cdn.net/content/dam/Nissan/jp/vehicles/gt-r/2203/interior/navi_audio/gt-r_2203_interior_navi_audio_pc_02.jpg.ximg.l_6_h.smart.jpg | https://history.nissan.co.jp/GT-R/R35/1904/interior/navi_audio.html |
| 15 | 2017 carbon cluster, 340 km/h speedometer (bot check blocked the reader) | unknown | https://www.vehiclesizes.com/cars/nissan/gt-r/gt-r-r35-facelift-2016-coupe-2017/images/1794-carbon-fiber-tachometer-speedometer-cluster-340kmh |

Drive `references` folder (with `SOURCES.md`): https://drive.google.com/drive/folders/1LMb4_yYfLh6wlFjzcu_Tjy7EOZKLIzmi . No image bytes were saved; the PC fetches the URLs above. Still wanted in step 1: a sharp NISMO tach close-up (2020 or 2024), a night shot of the cluster with the dimmer low, and one 8-inch MFM gauge page.

## 3. How the real thing behaves, with motion notes
- **Start-up.** Meters light at ignition ON (C11). A needle sweep is not documented anywhere read: unconfirmed, so the dash makes no sweep claim. The MFM shows its POLYPHONY DIGITAL & CLARION splash on its first opening after engine start (M1). Here: at `race_on`, the faces light from dark, then the MFM slots show a short Polyphony-style splash card once per session (text "GT-R" only, no third-party logos), then their gauges.
- **Idle.** Tach needle near the idle rpm; LCD shows speed 0; gear pod shows the gear; MFM arcs rest with a thin sliver of fill; histories hold their last 20 s.
- **Cruise.** Needles steady; graphs scroll at real time (20 s across the palette); the "now" dot rides the right edge (page A) or top (page C).
- **Pull to the limiter.** Upshift lamps: yellow about 700 rpm before the red zone, red just before it (AUTO, C10). The red zone ticks are printed, not lit. Boost arc fills to its value; page A traces the hump.
- **Limiter / shift strobe.** The real car shows only the steady red upshift lamp; nothing large flashes. The house strobe (lamp row only, 10 Hz at 50% duty, 2 Hz breath in gentle mode) is therefore a departure, kept small.
- **Gear change.** The pod's segmented glyph changes; the real transition speed is unconfirmed (LCD, so effectively instant with a short segment fade). A refused shift blinks the glyph (C8).
- **Braking.** Page B's inverted G trace; the brake bar fills.
- **Drift / lateral G.** Page C turns red above 0.5 G and stays red while above it (M6).
- **Reverse / neutral.** The pod shows R or N in segments; the LCD keeps the speed.
- **Shut-down.** Needles return; the fuel needle drops to E at LOCK (MAN09 p.84). Here: on a stale stream the host hides the dash.
- **Motion character.** Stepper-driven needles, smooth with no visible overshoot (Our reading, no source); MFM arcs and bars update live; numbers on the LCD update a few times a second (Our reading).

## 4. The maths
No row in "Real geometry and maths" for this dash. Mappings the unit tests can assert, all **Computed**:
- Tach angle θ(rpm) = −133° + 25.63° × rpm ÷ 1000 on the real 8,000 rpm face; the dash rescales so θ_max = +98° at `max_rpm` (θ = −133° + 231° × rpm ÷ max_rpm), and the red ticks start at `max_rpm` − 1,000 × (max_rpm ÷ 9,000) (the real 7,000 of 9,000 is 0.778).
- MFM boost: kgf/cm² = psi × 0.070307. θ = −111° × (−b) for b < 0, θ = 74° × b for b ≥ 0 (111° per 1.5), clamped to −1..+1.5. Review states: −2 psi → −0.141 (−15.6°), 5.6 psi → 0.394 (+29.1°), 16 psi → 1.125 (+83.3°).
- Upshift thresholds scaled to the car: red zone R = 0.778 × `max_rpm` (or the house `shift_rpm`, see 9.4); yellow at R − 700 × (max_rpm ÷ 9,000); red at R − 100 × (max_rpm ÷ 9,000).
- G in g = m/s² ÷ 9.80665; absolute G = √(g_lat² + g_long²).
- Gear tubes (page D replacement): ratio_g = speed ÷ rpm learned live per gear (`[f32; 10]`); predicted rpm in gear g = speed ÷ ratio_g; colour by predicted rpm ÷ R.
- Torque fallback = power × 60 ÷ (2π × rpm): 600 PS (441.3 kW) at 6,800 rpm gives 619.7 N·m.

## 5. Telemetry mapping
| Visual part | Input | Status | Transform |
|---|---|---|---|
| Tach needle, red zone | `rpm`, `max_rpm`, `idle_rpm` | verified | section 4 |
| Upshift lamps / lamp row | `rpm`, `shift_rpm` | verified | section 4 thresholds; MANUAL set point as a `config.extra` key |
| LCD digital speed | `speed` | verified | km/h or mph per config |
| LCD odometer and trip | `race_progress` (distance travelled) | decoded, offset confirmed at check C | km or miles; fallback ∫speed dt |
| Gear pod glyph | `gear` | verified | 0 = R, 11+ = N; >9 shows two segments |
| Fuel pod | `fuel` | decoded, offset confirmed at check C | 0..1 → E..F; hide the needle motion if constant |
| MFM BOOST | `boost` | verified | section 4 |
| MFM ACCEL / BRAKE | `throttle`, `brake` | verified | ÷255 → % |
| MFM STEERING arrow | `steer` | verified | ÷127 → ±full lock |
| MFM CORNERING / ACCEL-BRAKING / ABSOLUTE G, pages A to C histories | `g_lat`, `g_long` | verified | section 4; 20 s ring buffer |
| MFM SPEED | `speed` | verified | |
| TIME LOGGER replacement | `race_progress` (laps), `timestamp` | decoded at check C / verified | laps when present, else a session timer |
| Gear tubes | `speed`, `rpm`, `gear` | verified, derived | section 4 |
| Torque gauge | `torque` | decoded, offset confirmed at check C | fallback from power |
| Tyre heat pod | `tyre_temp` | decoded, offset confirmed at check C | unit (°C or °F) at check C |
| R/F slip bar | `wheel_speed` | decoded, offset confirmed at check C | rear mean ÷ front mean − 1 |

**What the game cannot send, and the replacements** (recommended first):
1. **Coolant temperature pod** (C2). (a) **Recommended: tyre heat pod**, the same C-to-H face fed by mean tyre temperature, a tyre pictogram in place of the coolant symbol, labelled tyres. (b) A small boost pod, the real car's most-watched MFM value pulled into the cluster.
2. **Engine and transmission oil temperature and pressure, water temp** (M4). (a) **Recommended: TORQUE (N·m) and POWER (kW or hp)** palettes in the real arc style. (b) Tyre temperature per corner as four small arcs on the oil-temp scale (70 to 150).
3. **Front torque split** (M5). (a) **Recommended: R/F SLIP**, the same plain vertical bar, driven by how much faster the rear wheels turn than the fronts (the very cue ATTESA reacts to), labelled slip, never torque; unlike the R34 dash it shows the raw cue, not a modelled split. (b) Drivetrain-aware: on a RWD car the bar shows rear slip only.
4. **Fuel flow and section consumption, fuel economy page E, range** (M4, M8). (a) **Recommended: page E becomes a WORK page**, one block per minute stacked over 20 min, each block the energy delivered that minute (∫power dt, kWh), the real page's Tetris grammar. (b) Fuel used per km from `fuel` and distance, labelled MODEL, only when the fuel value moves.
5. **Gear economy colours, page D** (M7). **Recommended: the tubes show rpm-if-in-this-gear-now** from the live per-gear ratios, pale blue in the good band, toward red near the red zone: the real page's lesson made exact for any car.
6. **TIME LOGGER from wheel buttons** (M8). **Recommended: automatic lap logging** from the lap fields (current, last, best in big amber digits), shown only when lap data is non-zero; otherwise a session timer started at `race_on`.
7. **DRIVERS NOTE place names** (M8). (a) **Recommended: a road trace**, the last 60 s of world position drawn as a thin white line in a palette, the road actually driven (not navigation). (b) A session log of peaks (top speed, max G, best 0 to 100) with their race time.
8. **Clock** (M4). Race time from `race_progress`, or local time only if the host exposes it (unconfirmed).
9. **TPMS** (2024 list). Tyre temperature, labelled as temperature.
10. **Vehicle information display messages, warnings, mode switches (R / Normal / Save, Comfort), outside temperature.** Dropped; the LCD keeps speed and distance only.

## 6. Edge cases
- `max_rpm` 0 or tiny: hold the needle at 0, hide lamps. Idle near max: lamps use `shift_rpm`, never fire at idle (require rpm rising past idle + 10% of range).
- 12,000+ rpm: the face rescales (numerals every 1 or 2 thousand, never more than 10 numerals); diesels (4,500): numerals every 500. EVs: no idle, gear 1; lamps from `shift_rpm`; boost arc shows 0 and the slot offers TORQUE instead.
- Reverse, neutral, gear > 9: segment glyphs R, N, two digits.
- mph and km/h, hp and kW: from config; the speedometer scale is not drawn (the dash has no speedometer dial unless the board adds one).
- Limiter bounce (about 50 rpm a frame): lamp hysteresis of 150 rpm; LCD digits held 0.1 s.
- Spikes and crashes: G clamps at the palette edge with the dot pinned; histories keep the true value for the peak readout.
- Airborne: G near zero; no special state. Stale or paused stream: the host hides the dash; histories freeze, not clear.
- Palette extremes and random palettes: needles and lamps follow roles, but faces stay physically believable (a print, never a glowing face); minimum contrast of numerals on face checked per palette.
- Font swap: numerals and MFM captions take the swappable font; the gear pod keeps segment glyphs (it is a display technology, not type).
- Reduced flash: lamp row breathes at 2 Hz; the page C red alert becomes a steady red frame line, not a red flood.
- Colour-only cues: upshift stages also differ by position (lamps light in sequence); the red alert also thickens the trace; colour-vision views checked at step 1.
- No turbo: boost arc rests at 0 or vacuum; offer to swap to TORQUE automatically after 30 s of no positive boost.

## 7. Signature moments
**Limiter / shift.**
1. **Recommended: the real upshift logic, scaled up.** The lamp row follows C10: outer lamps green, then yellow about 700 rpm before the red zone, red just before it, filling outside-in; on the limiter only the row strobes (10 Hz, 50% duty; 2 Hz breath gentle). Inside the tach, the three real lamps mirror it steadily.
2. Tooth-root glow: the blue light at the gear-tooth roots shifts to the `shift` role tooth by tooth from 7 to the end of scale as rpm passes it; steady, never strobing.

**Gear change.**
1. **Recommended: an LCD segment change.** Old segments fade over 40 ms while the new ones light at once, with the faint unlit segments always visible (C8): the real technology's own transition.
2. Page D echo: on each upshift the active gear tube steps right and its fill drops to the new predicted rpm, a small physical "click" beside the main glyph.

## 8. What it needs from the 4K library
Kit `gtr_nismo_mfd` (PR #18 branch): `carbon_twill_surround` (Blender 4096² tile, gloss 2×2 twill, the meter surround) and `tach_ring_460` (Blender sprite, machined alloy, three light keys). Step 1 finding: the real ring is about 25 to 30 px wide at Ø460, not the proposed 20 px (C9); re-run with about 26 px.

Shared parts to reuse: `alcantara_black` (MclarenSlimDisplay, the NISMO Alcantara meter hood), `lcd_backlight_mura` (SkylineR34Mfd, the blue LCD quadrant and gear pod), `antiglare_matte_grain` (SimInputTrace, the MFM screen face), `shift_led_32_off/on` (SimInputTrace, the lamp row), `pod_crystal_170` (ZNismoSport, the small pods' lenses), `spun_aluminium_chapter` (T50Floodlit, ring face brushing if the board wants it).

LIBRARY GAP: mesh-gear tooth ring, Blender sprite about Ø420 for a Ø460 dial, the toothed relief of the tach chapter with an emissive mask at the tooth roots for the blue glow; it is the R35's signature and no kit has it (blender-hero-assets).
LIBRARY GAP: carbon-weave face print, Blender 4096² tile, fine matte 2×2 weave (tows about 4 px at 4K), flatter and less glossy than `carbon_twill_surround`, for the tach face under the numerals (blender-hero-assets).
LIBRARY GAP: lit amber needle and matte black hub cap, Blender sprites (needle about 210 × 14 px, cap about Ø70) with translucent body and a hot core pass, tinted at run time (blender-hero-assets).
LIBRARY GAP: thick machined pod ring for the small pods (gear, fuel, heat), Blender sprite about Ø180 with a 22 px rim, three light keys; `small_dial_ring_150` is a thin chrome ring and does not match (blender-hero-assets).
LIBRARY GAP: MSDF atlas of an OFL segment font (DSEG7 Classic or similar) for the gear pod's segmented glyph with ghost segments (asset-forge MSDF).

## 9. Departures and open questions for the board
1. **Five pods versus the brief's six MFM slots.** The real cluster has speedometer, tach, gear pod, fuel and coolant pods (C2). Open: whether the board keeps a speedometer dial (the real car has one; the brief puts digital speed in the tach, which the real LCD quadrant also does, C5).
2. **The slot grammar.** The real page is six cells: six small, or one large plus two small (M3). Recommend each side be one MFM page of three small cells, or one large history graph plus one small, rather than six free slots of one size.
3. **Gear position.** The real gear pod sits at the tach's upper right (C1, C8). The house rule wants the hero gear on X = 1920. Recommend keeping the real position with a larger pod, and the handover says why; the alternative is the gear glyph centred in the dial.
4. **Gear glyph.** Real segmented LCD versus the brief's Saira Condensed 800: recommend segments (gap above) with the font swap applying to everything else.
5. **The 12-lamp row** is a departure from three small lamps (C10); its colours and timing follow the real logic.
6. **Red tach for NISMO.** Default preset: the NISMO red chapter (the game cars are NISMOs); the plain carbon face is a preset. Red here is a face print, so the red budget rule needs a note in the handover.
7. **Type.** Real numerals are bold upright (C1); recommend Saira (normal width) SemiBold upright for dial numerals and Titillium Web 600 or Barlow Semi Condensed 500 caps for MFM captions; step 1 compares crops of CW-P1 and US24.
8. **Customisable parts** (TJS 10:55), each staying physically believable, default first:

| Part | Default (period-correct) | Options | Maps to |
|---|---|---|---|
| Needle colour | amber-orange lit (about #FF7A1A) | red, white, palette | `primary` role, or `GtrNismoMfdNeedle` in `config.extra` |
| Face | NISMO red chapter on carbon | 2017+ carbon, 2011 plain charcoal | `GtrNismoMfdFace` |
| Tooth-root backlight | blue (about #2A6BFF) | white, amber, red, off | `accent` role, `GtrNismoMfdBacklight` |
| LCD colour | blue with white digits | amber, white, palette | `secondary` role, `GtrNismoMfdLcd` |
| Ring finish | machined aluminium | black chrome, satin titanium | `GtrNismoMfdRing` (sprite variant) |
| Lens tint | clear | smoked | `GtrNismoMfdLens` |
| MFM accent | red (alerts, active tab) | palette `accent` | `roles.warning` stays for alerts |
| Numeral font | Saira SemiBold | user font | font role `display` |
| Brightness | day and night levels | AppConfig brightness | as the real dimmer (C11) |
| Upshift mode | AUTO | MANUAL set point | `GtrNismoMfdUpshiftRpm` |
| Slots | real presets (page 1: boost large, oil-replacements small) | any gauge | `GtrNismoMfdSlot1..6` |

9. **Catalog correction for the lead:** the brief's "analogue cluster of speedometer, tachometer and fuel meter" misses the coolant pod and the separate gear pod (MAN09 p.81, US24); "three or six panels" is "six small or one large plus two small" (CW-MFM); the expected gauge styles are needle-less filled arcs, 20 s line graphs, vertical bars and digital values, not horizontal bars; `tach_ring_460`'s width should be about 26 px; fuel and torque are now offerable (88 fields).

## 10. Sources
**Nissan:** NIS-JP, cockpit page 2019 to 2023 https://history.nissan.co.jp/GT-R/R35/1904/interior/cockpit.html ; NIS-JP19, MFM page https://history.nissan.co.jp/GT-R/R35/1904/interior/navi_audio.html ; NIS-JP17, 2017 model page https://history.nissan.co.jp/GT-R/R35/1607/performance.html ; US24, GT-R overview and gallery https://www.nissanusa.com/vehicles/sports-cars/gt-r.html ; NISMO24 https://www.nissanusa.com/vehicles/sports-cars/gt-r/nismo.html ; NIS13, GT-R NISMO release https://global.nissannews.com/en/releases/131119-02-e ; launch release https://global.nissannews.com/en/releases/071024-01-e ; US24KIT https://usa.nissannews.com/en-US/releases/2024-nissan-gt-r-press-kit ; 2017 kit https://usa.nissannews.com/en-US/releases/us-2017-nissan-gt-r-press-kit .
**Manuals:** MAN09, 2009 GT-R owner's manual pp.81 to 88 https://www.manualsdir.com/manuals/192484/nissan-r35-d-2009-gt-r.html?page=85 (and ?page=81, 83, 84, 86, 87).
**Reviews and photos:** CW-MFM, Car Watch, Z. Nishikawa, "GT-Rのマルチファンクションディスプレイとは？" (2012-11-07) https://car.watch.impress.co.jp/docs/news/571012.html ; CW11, CW-P1, CW-A, CW-C, CW-D, Car Watch "写真で見る日産 GT-R EGOIST (2011)" gallery https://car.watch.impress.co.jp/img/car/docs/418/004/html/201.jpg.html ; CW-F https://car.watch.impress.co.jp/img/car/docs/571/012/html/DSC_8213.jpg.html ; Car Watch 2017 NISMO gallery https://car.watch.impress.co.jp/img/car/docs/759/581/html/12.jpg.html ; GAW16 https://www.guideautoweb.com/en/galleries/22898/2015-nissan-gt-r-and-2016-nissan-gt-r-nismo-the-latest-evolution/?im=6 ; VS17 (title only) https://www.vehiclesizes.com/cars/nissan/gt-r/gt-r-r35-facelift-2016-coupe-2017/images/1794-carbon-fiber-tachometer-speedometer-cluster-340kmh ; Gizmodo 2007 https://gizmodo.com/nissans-gt-r-data-dense-dashboard-explained-324672 ; JJ https://justjap.com/products/jjr-dash-instrument-cluster-light-globe-repair-upgrade-fits-nissan-r35-gtr .
**Leads checked:** `/mnt/project-files/dash-research/webcheck/GtrNismoMfd.md` (claims 1 to 14 hold, except the panel count and slot drawing styles corrected here), `nismo-skyline-clusters.json` item 8 (the recalled 0 to 9,000 tach with red from 7,000 is consistent with the photos, though no "9" is printed; the recalled right-hand "combined fuel and coolant dial with LCD" is wrong: fuel and coolant are separate pods and the LCD is in the tach; 11 pages confirmed by CW-MFM), `verify_nismo-skyline-clusters.json` (agrees).
