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

The brief asks for "rear tyre combined slip". The decoder (`crates/fh6-core/src/telemetry.rs`) has no combined-slip field, and FH6's `TireSlipAngle` is **normalised** ("0 means 100% grip and |angle| > 1.0 means loss of grip") [S23], is not in degrees, and is not verified against the golden packets (`docs/kit/README.md`). So the wedge's amber edge comes from body kinematics instead.

The lateral velocity of a point a distance b behind the centre of mass, on a rigid body yawing at rate r, is v_x − r·b (rigid-body kinematics). The rear axle's slip angle is then

β_r = atan2(v_x − r·b, v_z)

with b the centre-of-mass-to-rear-axle distance. The packet has no wheelbase; b = 1.35 m is our assumption (about half of a 2.7 m wheelbase) and is named in the handover. In a steady drift the rear axle slides more than the body, so β_r > β is the "rear is out" cue. The sign of r against v_x (FH6's axes are not stated as left- or right-handed [S23]) is checked against the golden packets before it is used: a car turning right with positive v_x must give β_r of the same sign. Until then, the amber edge is a no-claim cue and the handover says so.

### 6.3 Yaw rate and counter-steer

- r (angular velocity y, rad/s) is the wedge's feed-forward: the wedge's spring target leads by r·τ with τ about one display frame, so the wedge does not lag the slide (the brief's "feed-forward").
- Counter-steer: steer is normalised to −1..1 and drawn as a tick against the wedge. No wheel angle is claimed, since the packet gives the input, not the road-wheel angle.

### 6.4 The score: a DOSS-style slide score

DOSS's formula is not published [S2, S3, S5, S6]. DriftBox's 0 to 10 grade uses average angle and average lateral G between sector gates [S9]; VDrift adds distance, a length bonus, max speed and max angle [S12]. **Our model**, built from the four things DOSS is cited to judge (B7), each from verified fields, scored per slide (a run by the DriftBox rule, B3):

| Term | What it measures | From |
|---|---|---|
| A, angle | the slide's mean \|β\| against a 45° reference | velocity |
| S, speed | the mean speed in the slide against 100 km/h | speed |
| K, stability | 1 − the RMS of β's deviation from its 0.5 s running mean, against 10° | velocity |
| W, swing-back | the peak \|dβ/dt\| through each change of side, against 120°/s | velocity |

Score = 100 × (0.35·min(A, 1.2) + 0.25·min(S, 1.2) + 0.25·K + 0.15·min(W, 1.2)), clamped to 0..120 and shown to two decimals. The caps let a big slide pass 100, as D1GP's do (B6). Straightening up ends the run and freezes its score (B4). The weights are ours; the handover lists them as a call, and the unit tests assert the score for fixed traces without a GPU.

### 6.5 The SPIN rule

SPIN shows when \|β\| > 100°, or \|β\| > 45° below 10 km/h (B5). It voids the run's score, as a spin "deducts big points" in D1GP [S1].

### 6.6 What is verified

Verified: rpm, speed, gear, pedals, acceleration, velocity (the kit's lessons, `docs/kit/README.md`). Steer, the hand brake and angular velocity are decoded raw fields; the dash uses them as inputs only (a tick, a label, a feed-forward), and the handover names them as not yet checked against the golden packets. No tyre slip, suspension or weather field is read.

## 7. Parts the game can't drive, and what replaces them

From the benchmark and the category, these parts need data the packet does not send (fill rule, CLUSTER_PROMPT "The bar"):

| Real part | Why it is dropped | Replacement ideas (the board shows each; the best is picked there) |
|---|---|---|
| **Tsuiso proximity and comparative angle** (Formula Drift UDSM, D1 chase scoring) [S1, S7] | No other car is in the packet. | (a) **Ghost lead:** the last slide's β trace replays as a faint ghost wedge, and the live wedge is scored against it, so the driver chases their own best. (b) A "match" ring that closes as the live angle meets the ghost's angle at the same time into the slide. |
| **Course line, clipping and touch zones**, and the off-course tyre count [S1, S5, S7] | No track geometry; the puddle and rumble flags are not decoded (project memory, FH6 weather note). | (a) **Line ribbon:** the car's own path from `position`, drawn as a short top-down ribbon whose width is β, so a smooth slide draws a smooth ribbon. (b) Yaw-rate curvature (`yaw_curvature`) as the arc the car is carving. |
| **Sector gates and the cumulative score** (DriftBox Sectors mode) [S9] | No sector gates without a track. | (a) **Chain:** consecutive slides linked without straightening (β never under 4° for more than 1 s) add into a chain total, shown beside the slide score. (b) A best-of-session score. |
| **Formula Drift's "Decel Lights"** (brake lights for the chase car) [S8] | They are rear lights for the car behind. | The brake input warms the visor's lower lip, so a trail-braked entry shows on the visor. |
| **"Steady engine sound"** (D1 judges) [S1] | No audio. | rpm steadiness in the slide (the RMS of rpm about its running mean) feeds the meniscus of the light pipe: a held throttle gives a calm meniscus, a bouncing limiter a ragged one. |

## 8. Departures from the brief, and why

- **Tint by absorption, not alpha 56 to 64.** The category note and optics.md §11 rule out a flat alpha (§3).
- **Combined slip replaced by β_r** (§6.2): the field is not in the decoder and FH6's slip angle is normalised and unverified.
- **Score added.** The brief names the dash "a drift scoring tool" but gives it no score; the benchmark's score and its two-decimal D1GP format are added (B4, B6).
- **Run rule.** The score's run follows the DriftBox (5°, 25 km/h); the wedge keeps the brief's 8° / 4° hysteresis and its 5 m/s floor, so the score starts a little before the wedge shows.
- **SPIN** added from the benchmark (B5).
- **Signed angle** (B9): the real meter is unsigned.

## 9. The pre-built library, and what this dash still needs

`PREBUILD.md` (pre-build branch, PR #18) lists DriftGlass among the dashes with nothing to pre-build ("procedural only by their briefs"), so it has no kit. The asset rules still ask every dash for at least one bespoke ComfyUI asset and one bespoke Blender asset, visibly used. **Our reading**, to be decided on the board:

- Blender (hero library): the visor shell as a real polycarbonate part, with the eyeport's rubber gasket and the visor's pivot plates at the flanks, rendered under a studio rig at three azimuths for the G-driven light, plus its normal and thickness maps (the thickness map feeds the Beer-Lambert tint).
- ComfyUI: the gasket's moulded rubber and the pivot plates' carbon, as visible surfaces at 4096 px; or a smoked-polycarbonate micro-scratch normal map.
- From other kits, if they fit: the carbon twill swatches and Blender bezels that the library shares.

These wait for LIBRARY DONE (the rule at the top of CLUSTER_PROMPT.md).

## 10. Sources

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
