# Drift Glass: research note

Written 2026-11-28 for build #13 (catalog row 5, Motorsport category, target `concept`, Modern Digital 2027 family, cockpit). Every statement marked **Cited** was read from the source named, on that date. Statements marked **Our reading** are conclusions drawn for this dash and are open to the direction check. The optics equations are in [docs/specs/optics.md](../../../../../docs/specs/optics.md); this note covers the look, the behaviour and the maths.

This note holds research and plans only. The layout is designed on the Penpot page `07 · DriftGlass`; no layout is written here.

## 1. The bar and the benchmark

DriftGlass is a concept: no car ships with it. Its brief makes it a drift scoring tool whose housing is a smoked helmet visor. The real benchmark it is measured against is the **Racelogic DriftBox**, the GPS and yaw-rate meter that D1 Grand Prix's DOSS judging system runs on, and the scoring screens that D1GP and Formula Drift broadcast from it.

- **Cited:** DOSS uses in-car telemetry from a "Racelogic DriftBox" and judges entry speed, angle, speed of transition and the smoothness of the angle; "the points are instantly displayed on the digital screens" [S3].
- **Cited:** DOSS was introduced in 2013 and is "judged by a machine"; speed, angle and stability are scored individually [S2]. It measures speed, angle, angle stability and the sharpness of the swing-back (振り返し) with GPS and an angular-velocity sensor [S5, S6].
- **Cited:** D1GP solo runs are scored to two decimals and can pass 100: 99.10, 99.48 and 100.60 at Aichi in May 2026 [S4].
- **Cited:** Formula Drift's 2026 Universal Drift Scoring Method (Race Data Labs) scores line and angle "entirely" by machine, about 80% of a qualifying score, and measures proximity, comparative angle and comparative speed of two cars plus a deceleration pattern; the data is animated live into the broadcast [S7, S8].

An owner of a DriftBox, or a D1GP viewer, should recognise in DriftGlass: the live drift angle as the big number of the slide, the peak angle held after it, the speed beside it, a score per slide with two decimals, and the SPIN call.

## 2. What the benchmark does, feature by feature

| # | Feature | Source | In DriftGlass |
|---|---|---|---|
| B1 | **Drift angle** is the angle between where the car points and where it goes. | Cited: DriftBox quick start [S11]; VBOX [S10]; VDrift [S12] | Matched: β, §6.1 |
| B2 | **Live angle, peak angle held, and speed** on the main screen; the peak is held until reset. | Cited: DriftBox guide, screens 1 and 2 [S9] | Matched: the live wedge, the scribed peak tick and the `PEAK` readout. The hold ends by itself after the slide (no reset button: the cluster has no input). |
| B3 | **A run starts above 5° of drift and 25 km/h and ends when either falls below.** | Cited: DriftBox guide, Practice mode [S9] | Matched for the score's run (§6.4). The wedge keeps the brief's 8° / 4° hysteresis for its look (§8). |
| B4 | **Score** combines average drift angle and average lateral G per sector, 0 to 10; faster and more angle score higher. Straightening up inside a sector zeroes it. | Cited: DriftBox guide [S9]; datasheet lists "Score: Current & Total" [S13] | Changed: a DOSS-style score out of 100 to two decimals, as D1GP shows it (B6, §6.4). |
| B5 | **SPIN!** shows when drift exceeds 100°, or 45° below 10 km/h. | Cited: DriftBox guide [S9] | Matched: the same rule, §6.5. |
| B6 | **Scores to two decimals that can pass 100.** | Cited: D1GP Aichi 2026 [S4] | Matched. |
| B7 | **Angle, speed, angle stability and swing-back sharpness** are what DOSS scores. | Cited [S5, S6] | Matched: the four terms of the score, §6.4. |
| B8 | **10 Hz GPS with a temperature-compensated yaw sensor**; drift angle resolution 0.01°, accuracy 1°. | Cited: datasheet [S13] | Ours reads the game's velocity at frame rate. The readout shows whole degrees, since 1° is the real meter's accuracy. |
| B9 | The DriftBox's angle is **unsigned**; no page shows which side the car slides to. | Cited [S9, S11, S13] | Changed: DriftGlass shows the side, because the wedge opens toward it (a 2027 elevation). |

### 2.1 What the reference photos show

Photos saved 2026-11-28 to the Drive masters folder `drift_glass\references\` (12 files, with `sources.md`); they stay out of git. Read by eye from the files named.

| # | Photo | What it shows | Our reading for DriftGlass |
|---|---|---|---|
| P1 | `01_driftbox_unit_drift_screen.jpg` (Racelogic product photo) | A monochrome LCD headed DRIFT ANGLE in three cells: speed on the left (`54` over `KMH`), the live angle large in the centre (`24°`, heavy condensed bold), the peak on the right (`34°` over `PEAK`), and a segmented bar strip along the bottom. | The DriftBox's signature is the triad speed · live angle · peak, with the live angle the largest figure. DriftGlass keeps the triad and its order of size; the house rank rule still puts the gear above all three. |
| P2 | `04_d1gp_doss_solo_score_96_6.jpg` (D1GP 2024 Rd.10 Odaiba, broadcast frame) | Black screen, white 1 px grid, white condensed sans caps. A table ZONE · LINE OFF · JUDGE AREA · Sec. Point with five sector points to two decimals (25.17, 19.93, 19.63, 14.16, 18.74); entry speed `113.8 km/h` to one decimal; a total `96.6` in very large yellow; a red cell (`Z1`) marks a zone deduction; RANK in a gold box. The five sector points sum to 97.63, so the shown total is the sector sum less deductions. | DOSS shows sector points to two decimals and the total to one. DriftGlass shows the slide score to two decimals (as the press reports, B6) and the chain total to one, like the board's total. The deduction cue is a red cell on black: in DriftGlass red stays reserved for the limiter, so SPIN uses the `caution` role and the handover says why. |
| P3 | `05_d1gp_doss_solo_score_91_3.jpg` | The same screen with a judge-area deduction (J3 in red), 105.76 km/h, total 91.3. | Same reading as P2; the speed shows two decimals here, so DOSS's decimals vary by field. |
| P4 | `07_fd_rdl_udsm_gauge_closeup.jpg` (Formula Drift 2026, Race Data Labs review tool) | A round gauge: a speed arc 0 to 150 mph with one bar per driver in their colour (blue, magenta), each driver's speed large (`43 mph`) with the angle under it (`50°`), an acceleration-delta bar each (`+1.2 ft/s`), and a green PROX bar (`PROX 5.2 ft`) along the bottom. Thin white ticks, translucent dark centre panel. | The modern benchmark pairs speed with angle in one stack and compares two runs side by side. That is the ghost-lead idea (§7): the live slide in the palette's primary, the ghost in a dimmer secondary, and a match bar in place of PROX. |
| P5 | `10`, `11`, `12_visor_bell_panovision_*.jpg`, `08_visor_stilo_st5_dark_smoke.jpg` | Visor cutouts at three-quarter view: a constant-thickness shell, a curved top edge with a slight lip, flanks that wrap back to round pivot bosses, a pull tab low on one side. The flank seen obliquely is visibly darker than the face seen square-on, in light smoke and dark smoke alike; the iridium one adds a bright mirror sheen across the upper half. | Confirms V3: the darkening toward the flanks is path length (Beer-Lambert), not a painted gradient. The pivot bosses and the pull tab are the visor's recognisable hardware and are candidates for the Blender part (§9). |

### 2.2 Shift lights that behave like the real hardware

The Motorsport category asks for shift lights that behave like real hardware. MoTeC's shift light modules, common in pro drift and race cars, are the reference read.

| # | Finding | Source | In DriftGlass |
|---|---|---|---|
| L1 | The SLM has 8 full-colour LEDs, each with its own colour, intensity and purpose; the SLM-C has 8 fixed-colour LEDs as a 3-stage shift light plus warning lights. | Cited: MoTeC SLM and SLM-C datasheets [S28, S29] | The brief's 15 pills keep the stage logic; the count is the brief's. |
| L2 | **Stage thresholds are engine speeds, optionally per gear**, and once a stage is lit it stays lit until the engine speed drops below that stage's speed (hysteresis). | Cited: MoTeC CTN0030, CTN0031 [S30, S31] | Matched: stages from the kit's shift stages, held until rpm falls below the stage's own threshold, so a bouncing limiter does not flicker the stage lamps. |
| L3 | **Warnings override shift lights** on the same LEDs ("a warning light can override a shift light"; MoTeC advises giving warning lights priority). On the SLM-C, LEDs 1 and 8, the two ends, are the warning lights and LEDs 2 to 7 the shift lights. | Cited [S28, S30, S31] | Matched: the two end pills double as warning lamps in the `caution` role for SPIN and the over-rotation warning, and win over the shift stages while lit. Red stays reserved for the limiter. |
| L4 | Brightness is set by the driver, not by an ambient sensor. | Cited [S30, S31] | The pills follow the app's brightness; no auto-dimming of their own. |
| L5 | No source read gives the flash rate at the shift point or the limiter behaviour. | Cited [S28 to S31] | The flash rate follows the house flash rules (WCAG 2.3.1 audit) and the kit's strobe; the outer pills stay steady while the middle ones strobe (lesson from the BoschDdu rebuild). |

## 3. The visor: what a smoked race visor is

| # | Finding | Source |
|---|---|---|
| V1 | Shark's electrochromic IRID visor goes from 64% to 20% transmission (Clear), 38% to 12% (Dark Smoke, measured) and 29% to 8% (Blue mirror, an "iridium look"). It darkens in under 1 s and clears in under 3 s. | Cited: 1000PS test, Aug 2026 [S14] |
| V2 | Retail "light smoke" visors are sold as 30% or 50%, and "most iridium visors are comparable in colour to a light smoke visor"; "dark smoke visors are generally 80% tinted". The retailer's labels contradict each other, so the percentages are unreliable. | Cited: Chromeburner [S15, S16] |
| V3 | **Our reading:** a smoked visor is a curved polycarbonate shell of near-constant thickness, tinted in the mass. Toward the flanks the line of sight crosses the shell obliquely, so the light path is longer and the flanks look darker; that is Beer-Lambert absorption ([optics.md §6](../../../../../docs/specs/optics.md#6-absorption)), not a gradient. A light-smoke visor at about 50% transmission at normal incidence is the target: dark enough to read as smoked, clear enough that the game shows through. |
| V4 | **Our reading:** an iridium visor's colour is a thin-film reflection ([optics.md §9](../../../../../docs/specs/optics.md#9-thin-film-coating)). It is a candidate for the visor's top band, where a real helmet's light hits first; the colour must come from the equation and the palette accent must not be painted on. |

The brief's "alpha 56 to 64, glassTint" tint is a flat alpha. [optics.md §11](../../../../../docs/specs/optics.md#11-what-this-corrects) replaces such tints with Beer-Lambert over the path length, and the category note says the same ("built by absorption on the kit's glass and not by a flat alpha"). DriftGlass follows the category note (§8).

## 4. Tinted Liquid Glass

| # | Finding | Source |
|---|---|---|
| T1 | iOS 26.1's "Tinted" option "increases the opacity of Liquid Glass UI elements and improves contrast"; Clear is the default look. It is a binary switch, with no slider. | Cited: MacRumors [S17]; Daring Fireball [S19] |
| T2 | In Dark mode Tinted uses a darker background and keeps white text; on the Lock Screen it lowers the overall brightness so the time and the passcode prompt read. | Cited: TidBITS [S18] |
| T3 | **Our reading:** for a cockpit overlay on an OLED, "Tinted" means the glass darkens what is behind it so white numerals hold their contrast over a bright scene. That is the visor's job: it is a contrast device first and a decoration second. |

## 5. AR head-up restraint

| # | Finding | Source |
|---|---|---|
| H1 | The S-Class AR HUD makes navigation arrows and speed warnings seem "painted on the road ahead", from a DMD with 1.3 million mirrors. | Cited: CarAdvice [S20]; Mercedes-Benz [S22] |
| H2 | Mercedes' AR arrows are blue and can run over cars, pedestrians and trees, "slightly spoiling the illusion". | Cited: IEEE Spectrum (EQS, 2022) [S21] |
| H3 | No source read gives the S-Class HUD's image distance, field of view or palette. The often-quoted 10 to 40 m range is Panasonic's AR HUD 2.0, not Mercedes'. | Cited [S21] |
| H4 | **Our reading:** what DriftGlass takes from AR-HUD symbology is sparseness: a few thin symbols, each tied to a direction in the world (the travel direction, the body heading), drawn only while they mean something. The wedge and the G comet are world directions, so they follow this. |

## 6. Telemetry and maths

### 6.1 Body slip angle β

**Cited:** FH6 Data Out gives velocity "in the car's local space; X = right, Y = up, Z = forward", angular velocity in rad/s with Y as yaw, steer from −127 (full left) to 127 (full right) and the hand brake as 0 to 255 [S23]. The packet has no drift field and no weather field [S23].

β = atan2(v_x, v_z), the angle from the body's forward axis to the velocity. This is `FH6TelemetryData::car_slip_angle_rad` (it already fades β to zero below 3 m/s), and `Signals::slip_deg` springs it. Velocity is a verified field; β is derived from it.

### 6.2 Rear-axle slip, in place of the brief's tyre "combined slip"

*Superseded the same day: the study (§6.7 F4) showed β_r restates the yaw rate, and the full decoder (§6.8) brought the real combined slip. Kept as the record of what was tried.*

The brief asks for "rear tyre combined slip". The decoder (`crates/fh6-core/src/telemetry.rs`) has no combined-slip field, and FH6's `TireSlipAngle` is **normalised** ("0 means 100% grip and |angle| > 1.0 means loss of grip") [S23], is not in degrees, and is not verified against the golden packets (`docs/kit/README.md`). So the wedge's amber edge comes from body kinematics instead.

The lateral velocity of a point a distance b behind the centre of mass, on a rigid body yawing at rate r, is v_x − r·b (rigid-body kinematics). The rear axle's slip angle is then

β_r = atan2(v_x − r·b, v_z)

with b the centre-of-mass-to-rear-axle distance. The packet has no wheelbase; b = 1.35 m is our assumption (about half of a 2.7 m wheelbase) and is named in the handover. In a steady drift the rear axle slides more than the body, so β_r > β is the "rear is out" cue.

**The sign of r is not checkable from the repo.** FH6's axes are not stated as left- or right-handed [S23], and the golden packets in `tests/golden/packets/*.json` carry velocity and acceleration but no angular velocity (checked 2026-11-28). So r's sign comes from a verified field instead: the lateral acceleration of a body moving at speed v on a path of curvature κ is a_x = v²κ, and the path's turn rate is v·κ = a_x / v = r + dβ/dt. In a held drift dβ/dt is near zero, so r_est = a_x / v (verified fields only) has the sign r must have. The dash uses the packet's r only while its sign agrees with r_est above a small threshold, and r_est otherwise; the real-game check at the PC (memory "C") confirms the sign, and the handover records the result.

### 6.3 Yaw rate and counter-steer

- r (angular velocity y, rad/s) is the wedge's feed-forward: the wedge's spring target leads by r·τ with τ about one display frame, so the wedge does not lag the slide (the brief's "feed-forward").
- Counter-steer: steer is normalised to −1..1 and drawn as a tick against the wedge. No wheel angle is claimed, since the packet gives the input, not the road-wheel angle.

### 6.4 The score: a DOSS-style slide score

DOSS's formula is not published [S2, S3, S5, S6]. DriftBox's 0 to 10 grade uses average angle and average lateral G between sector gates [S9]; VDrift adds distance, a length bonus, max speed and max angle [S12]. **Our model**, built from the four things DOSS is cited to judge (B7), each from verified fields, scored per slide. A run starts by the DriftBox rule (B3: over 5° and 25 km/h) and ends after 300 ms under 4°, or under 25 km/h, so that a transition through zero stays inside one run (E7):

| Term | What it measures | From |
|---|---|---|
| A, angle | the slide's mean \|β\| against a 45° reference | velocity |
| S, speed | the mean speed in the slide against 100 km/h | speed |
| K, stability | 1 − the RMS of β's deviation from its 0.5 s running mean, against 10° | velocity |
| W, swing-back | the peak \|dβ/dt\| through each change of side, against 180°/s (§6.7 F3) | velocity |

Score = 100 × (0.35·min(A, 1.2) + 0.25·min(S, 1.2) + 0.25·K + 0.15·min(W, 1.2)), clamped to 0..120 and shown to two decimals. The caps let a big slide pass 100, as D1GP's do (B6). Straightening up ends the run and freezes its score (B4). The weights are ours; the handover lists them as a call, and the unit tests assert the score for fixed traces without a GPU.

### 6.5 The SPIN rule

SPIN shows when \|β\| > 100°, or \|β\| > 45° below 10 km/h (B5). It voids the run's score, as a spin "deducts big points" in D1GP [S1].

### 6.6 What is verified

Verified: rpm, speed, gear, pedals, acceleration, velocity (the kit's lessons, `docs/kit/README.md`). Steer, the hand brake and angular velocity are decoded raw fields; the dash uses them as inputs only (a tick, a label, a feed-forward), and the handover names them as not yet checked against the golden packets. The fields the full decoder added (§6.8) are raw until the real-game check; the dash reads the rears' combined slip and slip ratio (the visor's smoke), the rear rumble-strip flags (a kerb tick) and the distance travelled (slide length), and no suspension, temperature or weather field.

### 6.7 Study: what a real drift does to these numbers

No source read gives a measured drift-entry or transition rate in deg/s [S24 to S27], and the repo's drive scenarios and review states are synthetic (`tools/mock-udp`, `fh6_clusters::review`; the drift review state is a fixed 27°). So the numbers below come from a trace built on the real figures that the sources do give, through the kinematic identity a_y = V(β̇ + r) (the rearranged lateral equation of the single-track model in [S24, S26]; with the path turn rate a_y / V, r = a_y / V − β̇).

**Cited real figures.** A production Mercedes-AMG GT 63 S drifting under automated control held a target sideslip of 0.6 rad (34°), peaked at 0.66 rad (38°), with a peak yaw rate of 0.95 rad/s and speeds of 12.6 to 14.4 m/s; Stanford's MARTY held 0.7 rad (40°) with yaw rates up to 2 rad/s at 10.8 to 13.8 m/s [S24]. A BMW M3 held 30° on a skid pad [S25]. MARTY with a professional driver counter-steered about 25° at the wheel [S26]. Stanford reports sideslip "as high as 45 degrees" [S27]. In these sources β and r have opposite signs in a held drift (an example equilibrium: r = 0.6 rad/s, β = −0.7 rad, V = 13.8 m/s [S24]).

**The trace** (`/mnt/project-files/drift_glass/research/study_slip_trace.png`, made 2026-11-28 from a script kept outside the repo): straight, entry to −38° over 0.6 s, a held slide with ±2° of wander, a transition to +38° over 0.6 s, a held slide, exit over 0.8 s. Path turn rate ±0.95 rad/s at 13.5 m/s, so a held lateral acceleration of about 1.26 g.

| Finding | Number | What changes |
|---|---|---|
| F1 | Through the transition β crosses the ±4° band in about 50 ms, at a peak β rate near 190°/s, with the yaw rate reversing at up to about 200°/s. | The 300 ms end rule (E7) keeps a transition inside one run, with a margin of six times. Confirmed. |
| F2 | **The brief's 8° / 4° wedge hysteresis closes the wedge for about 70 ms mid-transition**, then reopens it: a blink at exactly the signature moment. | The wedge follows the run's rule instead: it opens above 8° and closes only after 300 ms under 4°. Through a transition it sweeps across the centre without closing (as §12 says). The brief's hysteresis is kept for the opening edge. |
| F3 | The swing-back term saturated: 190°/s against the 120°/s reference. | The reference becomes 180°/s, so a fast, clean transition earns full marks and a faster one only reaches the cap. |
| F4 | In the held slide, β_r − β stays at 4 to 6° all the time. It is r·b / V by construction (§6.2), so it restates the yaw rate and says nothing about grip. | **β_r is dropped as the amber edge's input** (kept here as considered). The amber edge becomes an **over-rotation warning**: lit while \|β\| is over 55° and still growing faster than 30°/s, the moment before a spin (SPIN comes at 100°). A first rule (over 30° and growing faster than 60°/s) lit falsely at the end of every entry and transition in the trace, where β still grows at up to 119°/s past 30°; held drifts in the sources sit at 30 to 45° [S24, S25, S27], so 55° is past any held angle. It needs only β̇, from verified velocity, and no sign of r. |
| F5 | The sample slide (38°, 48 km/h, steady) scores 75.30: angle 0.73, speed 0.47, stability 0.80, swing-back at the cap. | Speeds in Forza drifting run well above this test's 48 km/h, so the 100 km/h speed reference stays; the direction check revisits the weights with the review states. |

### 6.8 The full decoder (main 60c99c9, 2026-11-28)

The decoder now reads all 88 FH6 fields. Their offsets and units still wait for the real-game check at the PC; until then each is a raw input and the handover names it. What they change for DriftGlass:

| Field | FH6's definition | Use in DriftGlass |
|---|---|---|
| `tire_combined_slip_rl/rr` | normalised: 0 is full grip, \|slip\| > 1.0 is loss of grip [S23] | **Restores the brief's cue, in a form that means something.** In a held drift both rears are past 1.0 all the time (they must be, to slide), so a lit/unlit amber edge would just be on for the whole slide. Instead the rears' combined slip above 1.0 sets the **density of the visor's smoke**: the brief's "smoke = tint + frost noise" frost grows from nothing at 1.0 to its full 3.5% at 2.0, so the smoked visor fogs as the tyres smoke. A clean grip corner leaves it clear. |
| `tire_slip_ratio_rl/rr` | normalised longitudinal slip, same scale [S23] | Wheelspin: the larger of the rears' slip ratio above 1.0 feeds the same smoke, so a burnout or a clutch kick fogs the visor before the car even rotates. |
| `wheel_on_rumble_strip_*` | 1 on a rumble strip, 0 off | **The DOSS "LINE OFF" column, in part.** Count of rear wheels on a kerb, shown as a shy tick on the fan's rim while it holds. It is a kerb, not off-course, so the score is not docked for it. |
| `surface_rumble_*` | the surface's force-feedback rumble per wheel | A candidate for "off course" (grass and gravel rumble harder than tarmac). Its scale is unknown until the real-game check, so it is noted and not used. |
| `normalized_driving_line` | the game's driving-line value, i8 | A candidate for the course-line idea (§7). A drift line is not the racing line the game draws, so it is not scored; at most it tints the line ribbon. Decided on the board. |
| `distance_traveled` | metres travelled | **Slide length**, VDrift's base score [S12]: the distance covered within a run, shown with `PEAK` after the slide. It does not enter the DOSS-style score, whose four terms stay as cited (B7). |
| `tire_temp_rl/rr` | no unit stated (Forza titles have sent Fahrenheit) | Considered and left out: a drift heats the rears steadily, and a temperature readout would be one more number on a dash that is about angle. |
| `lap_number`, `current_race_time`, `best_lap` | race data | Not shown. A free-roam drift has no laps; the chain and the best slide of the session stand in for a sector total (§7). |
| `wheel_in_puddle_*` | 1 in a puddle, 0 out | Not used (no weather in Data Out; project memory "FH6 weather"). |

The over-rotation warning of §6.7 F4 stays: it is the one cue that warns before a spin, and it needs no unverified field.

## 7. Parts the game can't drive, and what replaces them

From the benchmark and the category, these parts need data the packet does not send (fill rule, CLUSTER_PROMPT "The bar"):

| Real part | Why it is dropped | Replacement ideas (the board shows each; the best is picked there) |
|---|---|---|
| **Tsuiso proximity and comparative angle** (Formula Drift UDSM, D1 chase scoring) [S1, S7] | No other car is in the packet. | (a) **Ghost lead:** the last slide's β trace replays as a faint ghost wedge, and the live wedge is scored against it, so the driver chases their own best. (b) A "match" ring that closes as the live angle meets the ghost's angle at the same time into the slide. |
| **Course line, clipping and touch zones**, and the off-course tyre count [S1, S5, S7] | No track geometry. The full decoder (§6.8) adds rumble-strip flags (a kerb, not off-course) and a driving-line value (the game's racing line, not a drift line). | (a) **Line ribbon:** the car's own path from `position`, drawn as a short top-down ribbon whose width is β, so a smooth slide draws a smooth ribbon. (b) Yaw-rate curvature (`yaw_curvature`) as the arc the car is carving. (c) A kerb tick from the rear rumble-strip flags, the nearest real cue to DOSS's LINE OFF. |
| **Sector gates and the cumulative score** (DriftBox Sectors mode) [S9] | No sector gates without a track. | (a) **Chain:** consecutive slides linked without straightening (β never under 4° for more than 1 s) add into a chain total, shown beside the slide score. (b) A best-of-session score. |
| **Formula Drift's "Decel Lights"** (brake lights for the chase car) [S8] | They are rear lights for the car behind. | The brake input warms the visor's lower lip, so a trail-braked entry shows on the visor. |
| **"Steady engine sound"** (D1 judges) [S1] | No audio. | rpm steadiness in the slide (the RMS of rpm about its running mean) feeds the meniscus of the light pipe: a held throttle gives a calm meniscus, a bouncing limiter a ragged one. |

## 8. Departures from the brief, and why

- **Tint by absorption, not alpha 56 to 64.** The category note and optics.md §11 rule out a flat alpha (§3).
- **Combined slip moves from an amber edge to the visor's smoke** (§6.8): with the full decoder the field exists, but both rears are past 1.0 for the whole of any drift, so an on/off edge would carry no news; its excess over 1.0 sets the frost density instead. The early-warning job goes to the over-rotation warning (§6.7 F4), since the rear-axle slip β_r turned out to restate the yaw rate.
- **Score added.** The brief names the dash "a drift scoring tool" but gives it no score; the benchmark's score and its two-decimal D1GP format are added (B4, B6).
- **Run rule.** The score's run follows the DriftBox (5°, 25 km/h); the wedge keeps the brief's 8° / 4° hysteresis and its 5 m/s floor, so the score starts a little before the wedge shows.
- **SPIN** added from the benchmark (B5).
- **Signed angle** (B9): the real meter is unsigned.

## 9. The pre-built library, and what this dash still needs

`PREBUILD.md` (pre-build branch, PR #18) lists DriftGlass among the dashes with nothing to pre-build ("procedural only by their briefs"), so it has no kit. The asset rules still ask every dash for at least one bespoke ComfyUI asset and one bespoke Blender asset, visibly used. **Our reading**, to be decided on the board:

- Blender (hero library): the visor shell as a real polycarbonate part, with the eyeport's rubber gasket and the visor's pivot plates at the flanks, rendered under a studio rig at three azimuths for the G-driven light, plus its normal and thickness maps (the thickness map feeds the Beer-Lambert tint).
- ComfyUI: the gasket's moulded rubber and the pivot plates' carbon, as visible surfaces at 4096 px; or a smoked-polycarbonate micro-scratch normal map.
- From other kits, if they fit: the carbon twill swatches and Blender bezels that the library shares.

These wait for LIBRARY DONE (the rule at the top of CLUSTER_PROMPT.md); TJS added on 2026-11-28 09:28 that Penpot, ComfyUI and Blender all wait for it.

**Earlier assets found.** The Drive holds Phase 6 Blender renders of a DriftGlass visor from 2026-11-23 (`visor_l.png`, `visor_c.png`, `visor_r.png`, 2528 × 728 at 128 samples, seed 1010, three light keys at azimuths −9°, 0° and 9°), made from a bpy script in an old worktree. They predate the 2026-11-27 rules (all assets 4K and 1:1 at the 3840 × 2160 frame; Penpot before assets), so they are not reused as the final plate. Their three-key approach (one render per light azimuth, swapped by lateral G) matches the brief's facet swap and is worth keeping in the new Blender recipe.

## 10. Telemetry mapping

Every input the dash reads, what it drives, and how it is held. "Verified" follows §6.6. Spring names are the tokens' `motion.springs` (`data/design_tokens.json`): `snap` T 0.102 s ζ 1, `fast` T 0.168 s ζ 0.9, `ribbon` T 0.07 s ζ 0.9 with feed-forward, `light` T 0.15 s ζ 1, `gBall` T 0.168 s ζ 0.9, parallax ω 12 rad/s for 0 / 2 / 4 px per g. Thresholds are kept in m/s and degrees inside, whatever unit the driver shows.

| Element | Input | Derived as | Motion | Held by | Verified |
|---|---|---|---|---|---|
| Gear | `gear` | `Gear::from_raw` (0 = R, 1 to 10, else N) | glyph change on the `fast` spring plus the family's rim light sweep | none: it changes on the frame the gear does | yes |
| Speed | `signals.speed` in the driver's unit | as sent | none (AGENTS 7) | `Hysteresis` of 1 unit, on the raw value | yes |
| Shift pills | `shift_fraction`, `rpm`, `max_rpm` | the kit's `ShiftLights` stages from the tokens; the two end pills are warning lamps for SPIN and over-rotation and win while lit (§2.2 L3) | lamps step; the strobe runs on the strip's own clock (time since the shift point) | each stage held until rpm falls below its own threshold (§2.2 L2) | yes |
| Light pipe and meniscus | `rpm_fraction` (idle to limiter), `throttle`, rpm steadiness | meniscus position from rpm, its brightness from throttle, its ripple from the RMS of rpm about its 0.5 s running mean (§7) | `ribbon` with `rpm_rate` feed-forward | peak-held rpm above the shift point (lesson: limiter flicker) | yes |
| Drift-angle readout | `velocity_x`, `velocity_z` | β of §6.1, computed here from the raw velocity and unwrapped across ±180° (edge case E3) | none: the digits are the live value | whole degrees, `Hysteresis` of 1° (B8: the real meter is accurate to 1°) | yes |
| Slip wedge | β, `angular_velocity_y` | opening = β; target led by r·dt (feed-forward) | `snap` | opens above 8° (brief); closes after 300 ms under 4°, so it never blinks in a transition (§6.7 F2); hidden below 5 m/s | β yes; r raw |
| Wedge colour band | \|β\| | 8 to 20° primary, 20 to 40° accent, over 40° caution (brief) | 150 ms crossfade (brief) | a 2° hysteresis at each band edge, so a slide at 20° does not blink between bands (our addition) | yes |
| Peak scribe and `PEAK` | max \|β\| of the run | moves outward only during the run | appears with the run end | tick held 2 s; `PEAK nn°` shown 1.5 s after the slide ends (brief) | yes |
| Counter-steer tick | `steer` | steer / 127, no wheel angle claimed | `ribbon` | none | raw |
| G comet | `g_lat`, `g_long` (the kit smooths them over 60 ms) | direction and length on the fan's pivot | `gBall` | none | yes |
| Visor smoke (frost density) | `tire_combined_slip_rl/rr`, `tire_slip_ratio_rl/rr` | frost from 0 at slip 1.0 to the brief's 3.5% at 2.0 (§6.8) | 150 ms fade | none | raw (decoder 60c99c9) |
| Kerb tick | `wheel_on_rumble_strip_rl/rr` | rear wheels on a kerb | 100 ms fade | none | raw |
| Slide length | `distance_traveled` | metres within the run, shown with `PEAK` | with the run's end | 1.5 s | raw |
| Over-rotation edge | β, β̇ | lit while \|β\| is over 55° and growing faster than 30°/s (§6.7 F4) | 150 ms fade | 100 ms minimum on-time | yes (from velocity) |
| HB label | `handbrake` | lit above 0.1 | 100 ms fade | none | raw |
| Visor light and facet swap | `g_lat` | light azimuth from lateral G | `light` | none | yes |
| Parallax | `g_lat`, `g_long` | 0 / 2 / 4 px per g by layer | parallax ω | none | yes |
| Visor lower lip | `brake`; the limiter | warms with brake (the decel-light idea, §7); steady `shift` at the limiter (brief) | 100 ms fade | none | yes |
| Slide score | the run's statistics (§6.4) | computed at the run's end | shown at the run's end | until the next run starts | yes (from β, speed) |
| Chain total | consecutive runs (§7) | the sum of a chain's slide scores, one decimal | shown with the slide score | until a straight of over 1 s at under 4° | yes |
| Ghost | the last run's β(t) | sampled at 30 Hz into a fixed buffer made in `new()` (20 s, 600 values) | replays from the next run's start | the last finished run | yes |
| SPIN | β, speed | §6.5 | appears at once | 1.5 s, and voids the run | yes |

## 11. Edge cases

| # | Case | What happens | Our plan |
|---|---|---|---|
| E1 | Standing still or crawling (under 1 m/s) | `car_slip_angle_rad` returns 0; β is undefined at zero speed. | Wedge and readout hidden below 5 m/s (brief); the score's run needs 25 km/h (B3). |
| E2 | Reverse gear, or rolling backwards | v_z < 0, so β is near ±180°. | No wedge, readout or SPIN while the gear is R; rolling backwards in a forward gear above 10 km/h is a SPIN by B5. |
| E3 | β crosses ±180° | The kit's `Signals::slip_deg` is smoothed by a linear lerp, which crosses 0 on the way from +179° to −179°: for a frame or two it shows a small angle and could end a run falsely. | Compute β from the raw velocity in the dash, unwrapped (take the step of least magnitude between frames), and latch SPIN, which comes first above 100°. |
| E4 | Airborne over a jump | v_y grows; a_x falls to near 0, so r_est = a_x / v carries no sign. | β from v_x and v_z still holds. The sign gate keeps the last agreed sign while \|r_est\| is under 0.2 rad/s; the score's run is frozen while \|a_y\| < 0.3 g, so a jump neither ends nor scores a run. |
| E5 | Donuts at low speed | Large β below 10 km/h. | SPIN by B5, as the DriftBox calls it; no score. |
| E6 | Grip cornering at the limit | Body slip of 2 to 6° in fast grip corners can cross the 5° run start. | A run only scores if its peak reached 8° (the wedge was shown); otherwise it is dropped silently. |
| E7 | Transition through zero (left to right) | β passes through 0, below 4° for a moment. | The run ends only after 300 ms under 4° (brief) or under 25 km/h, so a transition stays one run and its swing-back rate is scored (B7). The §12 study checks 300 ms against real transition times. |
| E8 | Bouncing off the limiter mid-slide | rpm flickers about 50 rpm a frame. | Only the pills strobe; the light pipe takes the peak-held rpm; the meniscus ripple is the steadiness cue, not a flicker. With `reduced_flash` the pills hold steady. |
| E9 | Gear change mid-slide | | The gear's change motion never disturbs the wedge or the run. |
| E10 | `max_rpm` 0, or idle above max | | The kit's `Signals` guards `shift_fraction` and `rpm_fraction`; the meniscus clamps to 0..1. |
| E11 | Hostile packets | `decode` returns finite floats within bounds; zeros pass through. | atan2(0, 0) is 0; every division (score terms, r_est) guards its denominator. The G5 gate must pass with no repairs. |
| E12 | Pause, menu or a stale stream | The host hides the dash (AGENTS 9). | On the first frame back, if `dt` exceeds 0.5 s, the run, the peak and the chain reset; the ghost keeps the last finished run. |
| E13 | A slide longer than the ghost buffer | | The ghost keeps the first 20 s of the run. |
| E14 | Units | The driver may show mph or km/h. | Thresholds stay in m/s and degrees; only the shown speed changes unit. |

## 12. Motion notes

What each moment does, in time. The look of each moment is drawn on the Penpot board.

| Moment | Trigger | Behaviour |
|---|---|---|
| Idle | race on, speed under 5 m/s | The visor rests with its light centred; the pills, the wedge and the readouts are dark; gear and speed show. Nothing moves. |
| Entry | \|β\| rises past 8° above 5 m/s | The wedge snaps open from the travel axis to the body heading on the `snap` spring, led by the yaw rate so it does not trail the slide. The angle readout appears; its first band colour is primary. |
| Hold | in the slide | The wedge tracks β. Crossing a band edge crossfades over 150 ms. The peak scribe moves outward only. The counter-steer tick and the G comet live on the same pivot. |
| Transition | β changes sign within the run | The wedge sweeps through the centre to the other side without closing; the swing-back rate is recorded for the score. |
| Exit | \|β\| under 4° for 300 ms | The wedge closes on `snap`. The peak tick stays 2 s, then fades. `PEAK nn°` shows for 1.5 s. The slide score appears, and the chain total if the slide was chained. |
| Spin | §6.5 | `SPIN` in the caution role; the wedge closes at once; the run's score shows as a dash; 1.5 s. |
| Limiter | rpm at the limiter | Only the pill strip strobes, on its own clock, with its outer pills steady in the shift colour (lesson: a dark still frame reads as "no shift"); the visor lip holds a steady `shift`. With `reduced_flash`, no strobe. |
| Gear change | `gear` changes | The family's glyph motion on the `fast` spring and a light sweep across the lozenge rim. |
| Cornering G | any | The visor's light azimuth follows lateral G on the `light` spring, so the bright facet swaps sides in corners; layers shift 0 / 2 / 4 px per g. |

## 13. Sources

Read 2026-11-28.

- [S1] D1 Grand Prix, "What's D1 Grand Prix": https://d1gp.co.jp/02_what/whats_index-e.html
- [S2] D1 Grand Prix, "Competition Method" (2015): https://d1gp.co.jp/02_what/whats_comp-e.html
- [S3] Speedhunters, D1GP Tsukuba (about 2018): https://www.speedhunters.com/?p=388407
- [S4] AUTOSPORT web via carview, D1GP Aichi, 2026-06-05: https://carview.yahoo.co.jp/news/detail/e7c55ed39ba2464bbf14e042d201692d0782ab18/?mode=top
- [S5] ASCII, D1GP Autopolis, 2020-11-12: https://weekly.ascii.jp/elem/000/004/033/4033633/
- [S6] MONOist, "Motorsport primer (10)", 2021-10-18: https://monoist.itmedia.co.jp/mn/spv/2110/18/news033_3.html
- [S7] RACER, Formula Drift 2026 format changes, 2026-02-23: https://racer.com/2026/02/23/formula-drift-announces-major-2026-competition-format-changes
- [S8] Formula DRIFT news, the same announcement: https://news.formulad.com/?p=56122
- [S9] Racelogic, DriftBox user guide, "Drift Mode": https://support-centre.racelogic.co.uk/motorsport/discontinued-legacy/performance-meters/performancebox-driftbox/ug/08-db-drift-mode/
- [S10] VBOX Automotive, "Measuring attitude angles": https://vboxautomotive.co.uk/en/how-does-it-work-measuring-attitude-angles
- [S11] Racelogic, DriftBox quick start guide: https://www.vboxmotorsport.co.uk/downloads/manuals/DriftBox-quickstart.pdf
- [S12] VDrift, "Drift scoring": https://raw.githubusercontent.com/VDrift/vdrift/master/docs/Drift_scoring.md
- [S13] Racelogic, DriftBox datasheet (RLDB): https://www.vboxmotorsport.co.uk/downloads/datasheets/RLDB_Data.pdf
- [S14] 1000PS, Shark IRID visor test, 2026-08-10: https://www.1000ps.com/en-gb/article/3014289/shark-irid-a-prueba-tintado-en-menos-de-1-segundo
- [S15] Chromeburner, Shark Race-R Pro GP 50% light smoke visor: https://www.chromeburner.com/shark-visor-race-r-pro-gp-50-smoke
- [S16] Chromeburner, Shark Race-R Pro GP 30% light smoke visor: https://www.chromeburner.com/au/en/shark-visor-race-r-pro-gp-light-smoke
- [S17] MacRumors, iOS 26.1 transparency option: https://www.macrumors.com/2025/10/20/ios-26-1-transparency-option-liquid-glass/
- [S18] TidBITS, iOS 26.1 optional opacity: https://tidbits.com/2025/10/21/ios-26-1-to-add-optional-opacity-to-liquid-glass/
- [S19] Daring Fireball, iOS 26.1 beta 4 Tinted option: https://daringfireball.net/linked/2025/10/21/ios-26-1-beta-4-liquid-glass-tinted-option
- [S20] CarAdvice, Mercedes-Benz AR head-up display: https://www.caradvice.com.au/news/mercedes-benz-unveils-augmented-reality-heads-up-display/
- [S21] IEEE Spectrum, augmented-reality car HUDs: https://spectrum.ieee.org/augmented-reality-car-hud
- [S22] Mercedes-Benz, "Augmented reality": https://www.mercedes-benz.co.nz/passengercars/brand/mercedes-me-magazine/innovation/articles/augmented-reality.html
- [S23] Forza, "Forza Horizon 6 Data Out Documentation": https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation
- [S28] MoTeC, SLM datasheet CDS18120: https://cdn.shopify.com/s/files/1/0798/0249/files/CDS18120_SLM_Shift_Light_Module_2.pdf
- [S29] MoTeC, SLM-C datasheet CDS18122: https://cdn.shopify.com/s/files/1/0332/8881/4637/files/CDS18122_SLM-C_Club_Shift_Light_Module.pdf
- [S30] MoTeC, CTN0030 "Configuring an SLM-C in M1 Tune": https://cdn.shopify.com/s/files/1/0798/0249/files/CTN0030_Configuring_an_SLM-C_in_M1_Tune.pdf
- [S31] MoTeC, CTN0031 "Configuring an SLM in M1 Tune": https://cdn.shopify.com/s/files/1/0798/0249/files/CTN0031_Configuring_an_SLM_in_M1_Tune_2.pdf
- [S24] Werner et al., "Drifting in the Future" (ICRA 2026; Mercedes-AMG GT 63 S, with MARTY's figures in its Table II): https://arxiv.org/pdf/2606.27914
- [S25] Meijer, Bertipaglia, Shyrokau, NMPC drifting of a BMW M3 Competition (2024): https://arxiv.org/pdf/2405.10859
- [S26] Righetti et al., phase-plane analysis of MARTY drift data: https://www.iris.unina.it/retrieve/4f72239f-ac09-49e0-9d9d-f219bc8842a1/paper_062.pdf
- [S27] Goh and Gerdes, "Simultaneous stabilization and tracking of basic automobile drifting trajectories" (IV 2016): https://ddl.stanford.edu/publications/conference-proceeding/simultaneous-stabilization-and-tracking-basic-automobile
