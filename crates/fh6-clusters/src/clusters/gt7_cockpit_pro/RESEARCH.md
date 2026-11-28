# Gt7CockpitPro: research note

Build #22, catalog row 55, category Motorsport, Target `inspired`, family Modern Digital 2027, view cockpit. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on 2026-11-28 from the source linked. **Computed** means worked out here from cited or measured inputs, with the numbers shown. **Our reading** is a conclusion or proposal for this dash, open to the direction check. Measurements of GT7's pictures were taken in the PC's browser from the manual's 1920 x 1080 JPEGs shown at 2x (so 1 picture pixel at 2x is 1 px at 4K); one screenshot pixel was 1.28 px at 4K, so every size below carries about ±3 px of error, and hues are JPEG estimates.

This note extends the finished sibling's note, [`../gt7_hood_pro/RESEARCH.md`](../gt7_hood_pro/RESEARCH.md) (17 recognisable points of the Bonnet View meter, colours, type, the packet's `MinAlertRPM`/`MaxAlertRPM`/`SuggestedGear`). Those points are not repeated here; read that note first.

## 1. The real display: what this dash keeps and where it departs

### 1.1 What GT7 really shows in its Cockpit View (new finding)
1. **Cited.** GT7 has four racing views, Cockpit, Normal, Bonnet and Chase, switched with R1 ([GT7 manual, "Racing Views"](https://www.gran-turismo.com/gb/gt7/manual/race/03)). The manual says the race screen's details "will vary depending on the current view" ([The Race Screen](https://www.gran-turismo.com/gb/gt7/manual/race/02)).
2. **Cited (the manual's four view pictures, read in the browser).** In the **Cockpit View** picture GT7 draws the **same compact low band as in the Bonnet View**, across the bottom of the screen over the steering wheel: tyre diagram (with a "W" compound initial), fuel semicircle E to F with the pump icon, odometer box reading 000007.3, four assist circles (two lit red as engaged), the brake bar, the speed `44` over `mph` right-aligned to a divider, the gear `2`, a red suggested-gear box reading `1` with `AT` under it, the right-hand assist icons, and the MFD at the far right titled "Track Map". The **Chase View** picture shows the same band too (`122 | 4`, red box). Only the **Normal View** picture shows the full meter (two round dials, the trapezoid pod, the MFD with ◀ ▶ and page dots). Picture URLs are in section 2.
3. **Our reading, and a correction to the sibling note.** `gt7_hood_pro/RESEARCH.md` says the full meter shows "in the Normal and Chase views". The manual's Chase View picture shows the compact band, so the full meter is a Normal View feature only. Nothing in the hood dash depends on this, but `Gt7ModularTripod`'s step 1 should know it.
4. **Our reading.** So the real "GT7 cockpit meter" is the hood dash's meter. That supports the brief's aim ("pressing RB between cockpit and hood changes the camera and not the language"), and it also means a literal copy would duplicate `Gt7HoodPro`, which already draws its band in the cockpit view scaled to fit (`gt7_hood_pro/geom.rs`, `fit`). This dash earns its place by re-composing GT7's parts for the taller 1440 x 560 binnacle, not by recreating a different GT7 screen.

### 1.2 Parts kept, measured
| # | Part | Measurement (4K px) | Label |
|---|---|---|---|
| K1 | **Speed, divider, gear** on the screen's centre line | Cockpit picture: divider at x ≈ 1931 (centre 1920, within error); speed cap ≈ 49, gear cap ≈ 82 | Computed from the Cockpit View picture (38 and 64 screenshot px x 1.28) |
| K2 | **The sizes hold across views** | Bonnet: speed ≈ 54, gear ≈ 80 (hood note); Cockpit: ≈ 49 and ≈ 82. Same within measuring error | Our reading from both pictures: GT7 does not resize the readout when the camera changes. This is the cited basis for the brief's "handover" |
| K3 | **Suggested-gear box** right of the gear, transmission tag under it | ≈ 90 x 64 in the Cockpit picture (hood note: ≈ 86 x 70) | Computed |
| K4 | **The rev strip is a shallow arc, not a straight bar** | Normal View pod at 2x: tick strip chord ≈ 803, sagitta ≈ 22 (ends lower than the middle). The Bonnet and Cockpit pictures show the same gentle bow | Computed: R = (c²/4 + s²)/(2s) = 3711 px, half-angle 6.2°, sagitta/chord 0.027. **New:** the hood dash drew a straight bar |
| K5 | **The trapezoid pod** behind speed and gear | Normal View: about 803 wide at the top, about 188 tall, short vertical sides then chamfered lower corners, translucent grey. In the Cockpit picture the same pod shows **solid translucent red** because the braking suggestion is on ("the meter panel will turn red", race screen item 11) | Computed (approximate corners); Cited for the red panel |
| K6 | **Steering dot** above the strip's centre, red; a grey centre mark beside it in the Normal View picture | dot ≈ 8 px | Cited (item 13); size Computed |
| K7 | **Brake bar** left of the pod: white fill, ABS reduction in red above it | in the Cockpit picture the bar is about half white, the upper part red | Cited (items "Brake Pressure", "Throttle") |
| K8 | **Surface water indicator**: a bar whose length is how wet the track is | far left in the Normal View picture, a small ticked vertical scale | Cited ("Surface Water Indicator") |
| K9 | **Monochrome white and grey, red for alerts** | as the hood note | Cited there |

### 1.3 Departures (each with its reason)
- **D1. Taller composition.** GT7's band is about 1916 px wide; the cockpit envelope is 1440 x 560. The band cannot sit in one row at full size, so the parts stack round the pod. Reason: binnacle fit and distinctness from `Gt7HoodPro`.
- **D2. The arch.** The brief keeps a "pointed arch" of two ribbon limbs from the 2026-11-23 hood brief. The finished `Gt7HoodPro` no longer has ribbons, diamonds or a squircle: it is a straight 95-LED tick bar (HANDOVER.md, "What was built"). **Our reading:** build the arch from GT7's own arc (K4) and the hood dash's LED-lens language, bent much further for the taller envelope (section 4), so the two dashes share the lens, the alert colour and the readout, and differ in silhouette.
- **D3. No round dials and no paged MFD.** Both belong to `Gt7ModularTripod` (CATALOG, TJX override of 2026-11-23). The request that launched this research described "detached round gauges" and a "D-pad paged centre panel": that is the Tripod's Normal View meter, so it is left out here (section 9, Q1).
- **D4. Type follows the finished hood dash, not the brief.** The brief names Saira Condensed 800 (cap 190) and 600 (cap 100). The hood dash shipped Michroma for the speed and Orbitron 600 for the gear (its RESEARCH.md, row 13). For "the camera changes, not the language" the cockpit takes the hood's faces and caps (section 9, Q2).
- **D5. Red stays reserved.** The steering dot takes the primary colour, as in the hood dash; red is for the alert run's neighbours, the suggestion tile, braking and alarms.
- **D6. 2027 hardware.** GT7 draws flat graphics; this dash shares the hood dash's black-glass, black-chrome and forged-carbon parts (family rule).

## 2. Reference photos
| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | GT7 **Cockpit View** with the compact band over the wheel, red braking panel, `44 | 2`, box `1`, MFD "Track Map" | https://www.gran-turismo.com/images/c/i1FGJ2FCkZaqlEE.jpg | [Racing Views](https://www.gran-turismo.com/gb/gt7/manual/race/03) |
| 2 | GT7 **Normal View**: the full meter, trapezoid pod with the arched tick strip, round dials, MFD | https://www.gran-turismo.com/images/c/i1iYlkuZCwBxdz.jpg | same |
| 3 | GT7 **Bonnet View**: the compact band, the arched strip lit red-pink near the shift point | https://www.gran-turismo.com/images/c/i1vStJCga1tqJuB.jpg | same |
| 4 | GT7 **Chase View**: the compact band again (`122 | 4`) | https://www.gran-turismo.com/images/c/i1eOkILgEsxJw.jpg | same |
| 5 | GT7 race screen, every item numbered 1 to 25 | https://www.gran-turismo.com/images/c/i16HBxihYqSTHO.jpeg | [The Race Screen](https://www.gran-turismo.com/gb/gt7/manual/race/02) |
| 6 | GT7 MFD pages (first of nine pictures, probably TCS) | https://www.gran-turismo.com/images/c/i1p3rCOIZ7S1hG.jpg | [The Multi-Function Display](https://www.gran-turismo.com/gb/gt7/manual/race/04) |
| 7 | GT7 MFD, second picture (probably torque distribution) | https://www.gran-turismo.com/images/c/i1DCktyPUHTylSH.jpg | same |
| 8 | GT7 MFD, sixth picture (probably Radar) | https://www.gran-turismo.com/images/c/i1xA6ggV8CaXd.jpg | same |
| 9 | GT7 MFD, last picture (Session Best layout) | https://www.gran-turismo.com/images/c/i1ZQd1qp86FXz8E.jpg | same |
| 10 | Porsche 911 (992.2) 12.6-inch curved cluster, rev counter view with shift speed at 12 o'clock | page gallery "The new 911 - studio" | [Porsche newsroom, 911 interior](https://newsroom.porsche.com/en/press-kits/911/Interior.html) |
| 11 | Formula 1 wheel: 15 shift LEDs above the central display | page figure | [F1 Chronicle](https://f1chronicle.com/how-does-an-f1-steering-wheel-work) |
| 12 | GT7 in-car race-car dashes in the cockpit view (Corvette and NSX Gr.4 best case) | page figure, post #1 | [GTPlanet thread](https://www.gtplanet.net/forum/threads/dashboard-info-for-vr-users-and-cockpit-cam-drivers.429610) |

The pictures are copyrighted: they stay out of git (`mockups/references/` is ignored). Drive `references` folder (holds `SOURCES.md`; the PC fetches the listed URLs): https://drive.google.com/drive/folders/1VhOvSKLnY9U_E05mAoe0f_Ud1cLPUkBG

## 3. How the real thing behaves, with motion notes
| State | GT7 behaviour | Source | For this dash (Our reading) |
|---|---|---|---|
| Start-up | Not documented; the race screen fades in with the HUD at the start | unconfirmed (step 1: a video of a race start) | hardware lit first, then the readout fades in within `bootMax` 1000 ms |
| Idle | A few ticks lit from the left end; the readout steady | Cited (pictures) | lit ticks from the left foot of the arch |
| Cruise | White run grows with rpm; the speed changes, the gear holds | Cited (pictures) | the arch fills on the `ribbon` spring (T 0.07 s, ζ 0.9, feed-forward) |
| Pull to the limiter | "Lights up as the engine revolution gets close to the red zone"; in the Bonnet picture the **whole lit run** turns red-pink at once | Cited (item 13); hood note point 10 | whole lit run takes `shift` from the warning stage, as the hood dash |
| Limiter | The packet has `RevLimiterBlinkAlertActive` ("Rev Limiting is active"), between `MinAlertRPM` and `MaxAlertRPM` | Cited ([PDTools `SimulatorPacket.cs`](https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs)) | blink rate undocumented: use the house strobe on the shift strip only |
| Gear change | The suggested-gear box (a downshift suggestion, 15 = none) disappears when no gear is suggested; `AT`/`MT` tag under it | Cited (packet; pictures) | as the hood dash: the tile shows gear + 1 at the shift point and folds into the gear when taken |
| Braking | **"Red text will blink and the meter panel will turn red"** before a corner; the brake bar shows ABS reduction in red | Cited (item 11, Brake Pressure) | panel tint driven by `normalized_ai_brake_difference`, steady, never blinking (section 5) |
| Drift / lateral G | Countersteer assist and stability telltales turn red when engaged | Cited (assist items) | slide lamp from body slip β, as the hood dash; light on rims follows lateral G |
| Reverse / neutral | Not shown in the pictures | unconfirmed | `R` in `warning`, `N` at full ink (hood finding 15) |
| Shut-down / pause | Packet flags `Paused` and `LoadingOrProcessing` | Cited (packet) | the host hides the dash on a stale stream |

The cockpit's own displays are drawn too: the in-car wheel screen in the Cockpit picture reads "85" and "110". **Cited:** the packet decode says GT7 "will always send 85" for water and "110" for oil temperature, so even GT7 shows those as constants.

## 4. The maths
No row in "Real geometry and maths". The modelled figure is the arch; these values are for the unit tests.
- **GT7's arc (Computed, from K4):** chord 803 px, sagitta 22 px → radius 3711 px, half-angle 6.21°.
- **The brief's arch (Computed):** feet at (±690, +170), crest at (0, −270) from the cockpit anchor: rise/chord 0.319 against GT7's 0.027. Straight limbs would be 818.4 px long at 32.5° from horizontal. One circle through the feet and the crest has R 761.0 px, half-angle 65.0° and arc length 1728 px.
- **Tick pitch on that arc (Computed):** GT7's pitch is about 8 px (803 px / about 100 ticks). On 1728 px: 95 ticks give 18.2 px, 100 give 17.3, 121 give 14.3; keeping GT7's 8 px pitch would need about 216 ticks. The board picks; 95 keeps the hood dash's count (lit count tests can then reuse its expectations: 10, 57, 95, 78 in the four states).
- **Lit ticks:** n = round(N x rpm / max_rpm), as the hood dash (`fraction`), where max_rpm ≥ 1000; shift point = 0.97 x max_rpm (Signals `shift_rpm`), warning stage at 93% of it.
- **Strobe (Computed):** 10 Hz at 50% → 3 frames on, 3 off at 60 Hz; 6 and 6 at 120 Hz; 7.2 and 7.2 at 144 Hz (use the clock, not frame counts).
- **Hood band in the cockpit (Computed):** 1440 / 1760 = 0.818, so the hood dash in the cockpit view shows the gear at about 72 px and the speed at about 49 px. If this dash holds GT7's sizes (K2), its readout is larger than the hood dash's own cockpit fallback.
- **Fallbacks (Computed):** torque = P x 60 / (2π rpm): 300 kW at 7000 rpm → 409.3 N·m. Body slip β = atan2(v_x, v_z): v_x 3, v_z 20 m/s → 8.53°. Boost: 1 psi = 0.06895 x 100 kPa (14.7 psi → 1.014). Speed: 1 m/s = 2.236936 mph = 3.6 km/h. Tyre temperature: Forza titles have sent °F (telemetry.rs doc), so 180 °F = 82.2 °C.

## 5. Telemetry mapping
| Visual part | Input | Status | Transform |
|---|---|---|---|
| Arch tick run, alert colour, strobe | `rpm`, `max_rpm`, Signals `shift_rpm` | verified | n = round(N x rpm/max_rpm); whole run `shift` from 93% of the shift point; strobe from the shift point |
| Speed with unit | `speed` | verified | m/s → mph or km/h, tabular, display hysteresis |
| Gear, suggestion tile | `gear`, rpm | verified, derived | tile = gear + 1 at the shift point (gears 1 to 9), as the hood dash |
| Steering dot | `steer` | verified | −127..127 → rail travel, τ 30 ms |
| Brake and throttle bars | `brake`, `throttle` | verified | 0..1 fill, white |
| Red share of the bars (GT7's ABS/TC reduction) | `tyre_slip_ratio` per wheel, `drivetrain` | decoded, offset confirmed at check C | **MODEL:** brake bar's top shows front lock (slip ratio < −0.15 under brake); throttle bar's top shows driven-wheel spin (> 0.15 under throttle) |
| Pod panel tint (braking suggestion) | `driving_line` (`normalized_ai_brake_difference`) | decoded, offset confirmed at check C | steady warm tint ramp while the value says brake; fallback: none (hidden) |
| Tyre diagram frames (heat) | `tyre_temp` x4 | decoded, offset confirmed at check C | quiet ink when in band, thermal step only off-optimal (family rule) |
| Tyre lenses (GT7: wear) | `tyre_combined_slip` x4 | decoded, offset confirmed at check C | glow above 0.08, stronger above 0.18; fallback: load by weight transfer from verified G (hood dash) |
| Fuel semicircle | `fuel` | decoded, offset confirmed at check C | 0..1 → E..F; if the game holds it constant, the semicircle shows boost as the hood dash |
| Odometer box | `race_progress` (`distance_traveled`) | decoded, offset confirmed at check C | metres → miles or km, one decimal; fallback: integrate verified speed (hood dash) |
| Surface water bar | `puddle` x4 | decoded, offset confirmed at check C | mean depth of the four wheels → bar length |
| Handbrake telltale | `handbrake` | decoded, offset confirmed at check C | lamp while > 0.1 |
| Slide lamp | `velocity` | verified | β over 8° on, 5° off, above 8 m/s |
| Light on rims, parallax | `g_lat`, `g_long` | verified | `light` spring, 0/2/4 px |
| Shy lap line | `race_progress` (current, last, best lap, lap number) | decoded, offset confirmed at check C | shown only when lap number > 0; free roam: acceleration timers from speed and timestamp |

**What the game cannot send** (each dropped part, with replacements; ★ = recommended):
- **Blind-spot semicircles** (other cars alongside). ★ **Kerb half-moons:** the left or right semicircle lights while that side's wheels are on a rumble strip (`rumble_strip`, decoded, offset confirmed at check C), so the cue keeps its place and its "something beside you" meaning. Second: the semicircles show lateral G, the outer one filling.
- **Assist telltales ABS, TCS, ASM, auto-drive, countersteer** (no assist flags in FH6). ★ **Grip lamps:** LOCK (front slip ratio under brake) and SPIN (driven-wheel slip ratio under throttle), MODEL, plus the slide lamp; grey when quiet, as GT7's "off". Second: one drivetrain glyph (FWD/RWD/AWD from `drivetrain`) lit when the driven axle slips.
- **Track map, radar, weather radar MFD pages** (no traffic, map belongs to `AmgMbuxTrackPace`). ★ **Horizon under the arch:** a faint artificial horizon behind the pod that tilts with `orientation` roll and lifts with pitch (decoded, offset confirmed at check C), so the cockpit dash shows the car's attitude where GT7 shows the world. Second: a G trace under the pod (the brief's G puck with its 500 ms trail).
- **Brake balance, fuel map, torque split pages** (driver settings, not sent). ★ **Power and torque at the right foot of the arch**, shy tabular figures (`power` verified; `torque` decoded, offset confirmed at check C, fallback the formula in section 4). Second: the engine's cylinder count engraved on the arch's foot (`cylinders`, decoded), a static label that changes with the car.
- **Lights-on indicator, nitro and overtake indicator**: ★ drop the lamp and give the space to the handbrake lamp; second: a boost semicircle for turbo cars.
- **Laps-to-go estimate, water and oil temperature**: not in the 88 fields (GT7 itself sends constants). ★ Leave out; the lap line holds the real lap data.

## 6. Edge cases
- **Max rpm 0 or tiny (< 1000):** arch dark, no alert, no tile (hood constant `MIN_LIMITER_RPM`).
- **Idle near max:** lit run starts high; the warning stage still needs 93% of the shift point, so no false alert at idle.
- **12,000+ rpm and diesels at 4,500:** the scale follows max_rpm; any numerals come from it, rounded to 1000s (hood `MARK_RPM` 1000 to 20,000).
- **EVs:** single gear, no idle, odd rpm. The arch still follows max_rpm. Suppress the suggestion tile when `cylinders` reads 0 (decoded, offset confirmed at check C; that 0 means electric is unconfirmed, step 1 checks it), otherwise an EV at its shift point would be offered a gear 2 it does not have.
- **Reverse, neutral, gear above 10:** `R` in `warning`, `N` at full ink, gears above 10 shown as numerals with no tile (hood `TOP_GEAR` 10).
- **mph and km/h, hp and kW:** units from Signals; tabular digits so the width never jumps.
- **Limiter bounce (≈ 50 rpm a frame):** shift hysteresis 120 rpm (hood constant) on the alert and tile; digits do not change during the strobe.
- **Spikes and crashes, airborne:** G light clamped; airborne (all four suspension near full droop, decoded) holds the horizon idea still.
- **Stale or paused stream:** the host hides the dash.
- **Palette extremes:** all colours from roles; the alert run must stay distinct from the white run by brightness as well as hue (hood finding 11: hot core).
- **Font swap:** numerals take the user's display font; the divider and tile size from the cell, not the glyph.
- **Cockpit only:** this dash's view is cockpit; the hood view stays `Gt7HoodPro`. The scrim hides the game's dash, so no dimming pad.
- **Reduced flash:** the 2 Hz breath on the strip only; the panel tint never flashes in any mode (GT7 blinks its braking text; this dash does not).
- **Colour-only cues:** the alert also has the tile and the strobe; kerb half-moons are left/right by position; grip lamps carry letters.

## 7. Signature moments
**Limiter / shift strobe** (only the shift strip strobes, 10 Hz at 50%, 2 Hz breath in gentle mode):
1. ★ **Crest strobe:** the whole lit arch takes `shift` at the warning stage (GT7's all-at-once colour), and from the shift point the strobing run is the tick strip itself, as in the hood dash, so the two views strobe the same part. Grounded in K4 and hood point 10.
2. **Crest-only strobe:** the arch holds steady `shift` and only the top 11 ticks over the gear strobe, like the F1 wheel's 15 LEDs above its display (Cited, F1 Chronicle). Smaller flash area, less like GT7.
3. **Porsche crest:** the shift point is pinned at 12 o'clock of the arch, as Porsche rotates its rev counter so the needle is at 12 o'clock at shift speed (Cited, Porsche newsroom); the run meets the crest exactly at the shift point and the strobe starts there.

**Gear change:**
1. ★ **Tile hand-off:** the red suggestion tile, showing gear + 1, slides into the gear when it is taken and folds away within 6 frames, then the run drops back down both limbs on the `ribbon` spring. Same moment as the hood dash, so RB does not change it.
2. **Crest split:** at the change the lit run parts at the crest and falls down both limbs to the new rpm, the brief's "blade splits" moment without a blade.
3. **Pod pulse:** the pod's rim light sweeps once across (family "light sweep across the pod rim"), 150 ms, no flash.

## 8. What it needs from the 4K library
- **Kit made for this dash:** none. Cited: `kits.rs` on the PR #18 branch lists Gt7CockpitPro among dashes with nothing layout-independent to make, and PREBUILD.md says it "shares Gt7HoodPro's parts".
- **Shared parts to reuse (on main, `assets/clusters/gt7_hood_pro/`):** the forged-carbon master (`gt7_hood_pro_forged_carbon.png`, ComfyUI seed 73101) for the car chip; the hood recipe's materials (black glass, black chrome, smoked LED lens, red enamel tile, gunmetal arc) in `tools/asset-forge/src/recipes/gt7_hood_pro.rs`; the suggest tile (`suggest_0..2.png`, 110 x 102) if the board keeps the hood's tile size. The plate itself (`plate_0..2.png`, 1760 x 240) is layout-dependent and cannot be reused.
- **Still needed (layout-dependent, not gaps):** the arch's LED carrier and lenses on the board's curve, the pod, the arch's two machined feet, relit for the cockpit.

LIBRARY GAP: one smoked LED lens cap sprite, 24 x 32 px at 4K, three light azimuths (-110°, -135°, -160°), so an arch of any radius can be built from instanced lenses instead of one baked plate; Blender (asset-forge hero, the hood recipe's lens material).

LIBRARY GAP: a seamless 4096 px anti-glare micro-matrix texture for the black-glass carrier (the brief's "fine matrix under the ribbons re-made at the cockpit scale"); ComfyUI SDXL, then sampled into the Blender carrier.

LIBRARY GAP: MSDF atlases for Michroma and Orbitron 600 if the cockpit follows the hood's faces (D4), checked for an outer distance band wide enough for the 6/18 px keyline; font tool of the kit.

## 9. Departures and open questions for the board
1. **Round dials and paged MFD left out.** They are `Gt7ModularTripod`'s; the launch request described them, the catalog brief does not. The coordinator should confirm (the Tripod keeps the Normal View meter).
2. **Type: the hood's faces (Michroma, Orbitron 600) or the brief's Saira Condensed?** The handover between views argues for the hood's.
3. **Readout size:** hold GT7's sizes (gear ≈ 88, speed ≈ 60, as the hood dash and as GT7 does across views, K2) or grow toward the brief's caps 190 and 100? Holding is the cited behaviour; growing uses the taller envelope.
4. **Arch rise:** the brief's 0.319 rise/chord is twelve times GT7's 0.027. A rise near 0.15 might read as GT7's bow made taller; the board decides, keeping the brief's 60 px true black between the arch and the gear.
5. **Tick count:** 95 (the hood's, reusing its tests) or a pitch-led count (section 4).
6. **Pod:** GT7's trapezoid (K5) behind speed and gear, invisible until the braking tint warms it, would be the dash's quiet nod to the Cockpit View picture; it must not become a backing slab (hood rejection c).
7. **Brief's diamonds:** the finished hood has none; drop them so the views match, or keep them and break the handover.
8. **New decoded fields** (tyre temps, fuel, distance, puddles, rumble, AI brake difference) all wait for check C; each has a fallback above so the board can design them now.
9. **Correction for the hood note:** GT7's Chase View shows the compact band, not the full meter (1.1, point 3).

## 10. Sources
**Polyphony Digital (GT7 online manual), read in the browser**
- Racing Views: https://www.gran-turismo.com/gb/gt7/manual/race/03 (view pictures listed in section 2)
- The Race Screen: https://www.gran-turismo.com/gb/gt7/manual/race/02
- The Multi-Function Display: https://www.gran-turismo.com/gb/gt7/manual/race/04
- Display Settings: https://gran-turismo.com/us/gt7/manual/drivingoption/05

**GT7 packet decode**
- PDTools `SimulatorPacket.cs`: https://github.com/Nenkai/PDTools/blob/master/PDTools.SimulatorInterface/SimulatorPacket.cs

**Shift-light and rev-counter references**
- Porsche newsroom, the new 911 interior: https://newsroom.porsche.com/en/press-kits/911/Interior.html
- F1 Chronicle, how an F1 steering wheel works: https://f1chronicle.com/how-does-an-f1-steering-wheel-work

**Community**
- GTPlanet, dashboard info for VR and cockpit-cam drivers: https://www.gtplanet.net/forum/threads/dashboard-info-for-vr-users-and-cockpit-cam-drivers.429610

**Repo**
- `crates/fh6-clusters/src/clusters/gt7_hood_pro/` (RESEARCH.md, HANDOVER.md, mod.rs, geom.rs); `assets/clusters/gt7_hood_pro/ASSETS.md`; `docs/TELEMETRY_USAGE_MATRIX.md`; `crates/fh6-core/src/telemetry.rs`; `data/design_tokens.json`; PR #18 branch `tools/asset-forge/src/prebuild/kits.rs` and `program/phases/07-cluster-factory/PREBUILD.md`.
