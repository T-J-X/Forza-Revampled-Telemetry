# LorenzButterfly: research note

Build #33, catalog row 60, category Abstract, target `concept`, family Fluid / Generative, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used below: **Cited** means read from the named source on 2026-11-28 (link given). **Computed** means worked out here with `python3 -I` and numpy from cited constants; the inputs and numbers are given so the dash's unit tests can assert them (scripts: fourth-order Runge-Kutta, step 0.005 unless stated). **Our reading** is a conclusion or idea for this dash, open to the direction check.

## 1. The real display or benchmark

A `concept` dash names one real object. **Ours is the featured Wikimedia Commons plot "Lorenz attractor yb.svg"**, the yellow-on-blue butterfly that the Wikipedia article on the Lorenz system grew up with (it is built from that article's lead image, "Lorenz system r28 s10 b2-6666.png"). Its file page calls it "An icon of chaos theory", a projection of one trajectory with the "canonical" values r = 28, σ = 10, b = 8/3 (**Cited**, [Commons file page](https://commons.wikimedia.org/wiki/File:Lorenz_attractor_yb.svg)). It is the picture most people mean by "the butterfly", which is why it beats the original 1963 figure as the measuring stick.

What makes it recognisable (**Our reading**, from the plot and checked against the maths in section 4):
1. **Two lobes, two holes.** Each wing is a tight spiral round an empty eye. The eyes are the resting points C±; the trajectory never comes nearer to them than about 3.5 units (**Computed**, closest approach in 8,000 samples of the attractor), so each wing has a clean dark hole of about 100 px at this dash's 30 px per unit.
2. **One line, never crossing itself in 3D.** It is a single thin curve; the apparent crossings are projection. A glowing trail with depth sorting (the brief's "far to near") keeps that honest.
3. **Proportions.** In x-z the figure is about 38 units wide (x from -19.3 to +19.3) and 46 units tall (z from 1.7 to 47.4), with a waist at x = 0 near z = 15 to 25 where the two wings hand over (**Computed**, 64 trajectories, 500 time units each).
4. **Dense outer rings, sparse switching strands.** Most loops stay on one wing; the handovers are fewer and wider. Lorenz's own Figure 1 shows the same irregular lobe switching in time (**Cited**, Lorenz 1963 p. 137).

The **historical anchor** is Lorenz's Figure 2 of 1963: projections on the X-Y and Y-Z planes of the trajectory from iteration 1400 to 1900, integrated with a step of 0.01 from (0, 1, 0) (**Cited**, [Lorenz 1963](https://scispace.com/pdf/deterministic-nonperiodic-flow-1k20vjsax3.pdf) pp. 136-137). The **physical twin** is the Malkus waterwheel, a leaking-bucket wheel whose motion obeys the Lorenz equations: at low inflow it turns steadily one way, at higher inflow it reverses chaotically (**Cited**, [Wikipedia, Malkus waterwheel](https://en.wikipedia.org/wiki/Malkus_waterwheel)); Illing et al. built one from a 16.5 inch BMX wheel and 56 syringes with a magnetic brake and synchronised a Lorenz model to it (**Cited**, [arXiv 1202.5508](https://ar5iv.labs.arxiv.org/html/1202.5508) §II). **Our reading:** the waterwheel is a lovely story (steady rotation at cruise, reversals near the shift point) but its look is a wheel, not the figure; the plot stays the benchmark. No better real target was found.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | The benchmark: "Lorenz attractor yb.svg" | https://upload.wikimedia.org/wikipedia/commons/5/5b/Lorenz_attractor_yb.svg (to confirm) | [Commons](https://commons.wikimedia.org/wiki/File:Lorenz_attractor_yb.svg) |
| 2 | Its source PNG, the Wikipedia lead image | to confirm | [Commons](https://commons.wikimedia.org/wiki/File:Lorenz_system_r28_s10_b2-6666.png) |
| 3 | Lorenz 1963 Fig. 2, X-Y and Y-Z projections | PDF p. 137 | [scispace](https://scispace.com/pdf/deterministic-nonperiodic-flow-1k20vjsax3.pdf) |
| 4 | Lorenz 1963 Fig. 4, the map of successive z maxima | PDF p. 138 | same |
| 5 | Lorenz 1963 Fig. 1, Y against time | PDF p. 137 | same |
| 6 | Solutions at ρ = 13, 14, 15, 28 (spiral versus butterfly) | in the article gallery (to confirm) | [Wikipedia](https://en.wikipedia.org/wiki/Lorenz_system) |
| 7 | Bifurcation diagram with 13.926, 24.06, 24.74 | https://arxiv.org/html/2009.11017/x1.png (to confirm) | [arXiv 2009.11017](https://arxiv.org/html/2009.11017) |
| 8 | Basins of C± at r = 20 | .../x2.png (to confirm) | same |
| 9 | Basins at r = 24.5, three attractors | .../x4.png (to confirm) | same |
| 10 | x(t) while r drifts down through 24.06 | .../x6.png (to confirm) | same |
| 11 | Malkus waterwheel simulation figure | to confirm | [Wikipedia](https://en.wikipedia.org/wiki/Malkus_waterwheel) |
| 12 | A real Malkus-Lorenz waterwheel (photo, Fig. 1) | to confirm | [ar5iv 1202.5508](https://ar5iv.labs.arxiv.org/html/1202.5508) |
| 13 | Y-Z phase plot of the Lorenz 1963 model | [Physiome PNG](https://models.physiomeproject.org/w/randall/Lorenz_1963/file/c8321a0c35604ad741c146993945a20baf087223/Phase%20space%20trajectory%20yz.png) | [Physiome](https://models.physiomeproject.org/w/randall/Lorenz_1963) |

Drive `references` folder: https://drive.google.com/drive/folders/1Zlaim1FFMNR8x27A8unePgbKMQXrAe3J (holds `SOURCES.md`; no image bytes saved, the PC fetches the URLs).

## 3. How the real thing behaves, with motion notes

The benchmark is a still plot, so "behaviour" here is the flow's own, at the brief's mapping ρ = 28 × rpm ÷ shift rpm and clock of one model unit per 50 m (floor 0.5 unit/s). All times **Computed** unless marked.

| State | ρ | What the equations do | Motion note |
|---|---|---|---|
| Engine off / start-up | 0 | Origin is the only attractor (**Cited**, Lorenz 1963 p. 135: all roots negative for r < 1). Points slide to the origin. | One still light at (0, +215). On start-up at idle (e.g. 800 of a 6,790 rpm shift point, ρ = 3.30), a point held 1e-6 off the z axis needs ln(10⁶) ÷ λ₊ to lift off, λ₊ = 1.80: **7.7 units = 385 m, or 15 s parked**. See section 6. |
| Idle | 1.35 to 5 | Eyes appear at ρ = 1; spirals from ρ = 1.3456. Decay of a spiral: Re λ = -0.93 at ρ = 5, 0.71 turns per e-fold, loop period 1.51 units. | Eyes near the origin; anything that moves settles within about 5 units (250 m or 10 s parked). Still. |
| Cruise | 8 to 14 | Re λ = -0.60 at ρ = 10 (1.65 turns per e-fold, period 1.02 units); e-fold length 84 m. | When the revs change, the eyes slide along the parabola and the points chase them in a few loops. Two still lights at steady cruise. |
| Mid to high revs | 14 to 24 | Transient chaos from 13.926 (**Cited**, Doedel et al.: ρ_hom ≈ 13.9265; "preturbulence"). Re λ = -0.155 at ρ = 20 (8.95 turns per e-fold), -0.023 at ρ = 24 (67 turns). | Settling slows sharply: at ρ = 20 a 0.5 unit offset needs about 17 units (875 m) to fall below 1 px; at ρ = 24 about 120 units (6 km). The spirals are the figure here. |
| Shift window | 24.06 to 24.74 | Chaotic attractor born at 24.0579 beside the stable eyes; about 20% of a sphere of starts ends in an eye at r = 24.5 (**Cited**, Cantisán et al. §2, Fig. 4). | A point already flying stays flying; a point near its eye stays there. |
| Pull to the limiter | 24.74 to 28 | Eyes become unstable in a subcritical Hopf at 470 ÷ 19 = 24.7368 (**Cited**, Lorenz 1963 p. 136; Doedel et al.). Outward e-fold at the eye: 126 units at ρ = 25, 10.6 units (532 m) at ρ = 28. | **The wings are slow to appear** (section 4.4): from a settled eye, the first switch to the other wing comes after 13 to 58 units (0.65 to 2.9 km) of driving. |
| Limiter | 28 (clamped) | Butterfly; largest Lyapunov exponent 0.906 per unit. | Only the lamps strobe; eyes steady `shift`. |
| Upshift / lift | falls below 24 | Catch rule of the brief: each point set 0.5 units from its eye, trail fades over 300 ms. Without it, the median capture at ρ = 20 is 19 units (0.97 km) and at ρ = 24 only 20 of 300 points are captured in 2,000 units. | Wings fold in about one loop; spirals then decay at that ρ's rate. With the parameter drifting, chaos outlives the bifurcation: drifting down at 10⁻³ per unit, capture comes at a mean r = 23.07 (**Cited**, Cantisán et al. §3, Fig. 7), which supports catching on the gear change rather than waiting. |
| Braking / downshift | rises | A downshift raises ρ; same physics as a pull. | Blip on downshift may cross 24.74 briefly; the slow onset means no wing flicker. |
| Lateral G | any | The view turns about the vertical (z) axis, 12° per g. | A turn never changes a height, so the rpm numerals stay put. |
| Reverse / neutral | from rpm | ρ follows the revs; revving in neutral at rest runs the clock at its floor. | See section 6. |

## 4. The maths / physics

### 4.1 The model and constants
- x′ = σ(y − x), y′ = x(ρ − z) − y, z′ = xy − βz; Lorenz printed X′ = −σX + σY, Y′ = −XZ + rX − Y, Z′ = XY − bZ (equations 25 to 27), with σ = 10 and b = 8/3 "following Saltzman (1962)" and r = 28 "the slightly supercritical value" (**Cited**, Lorenz 1963 pp. 135-136). σ is the Prandtl number, ρ the Rayleigh number ratio, β a geometric factor (**Cited**, [Wikipedia](https://en.wikipedia.org/wiki/Lorenz_system)).
- Resting points C± = (±√(β(ρ − 1)), ±√(β(ρ − 1)), ρ − 1); Lorenz gives (6√2, 6√2, 27) at r = 28 (**Cited**). **Computed:** x = ±8.485281, z = 27 at ρ = 28; (4.898979, 4.898979, 9) at ρ = 10.
- Stability bound ρ_H = σ(σ + β + 3) ÷ (σ − β − 1) = 470 ÷ 19 = 24.736842 (**Cited**, Lorenz eq. 34 "r = 470/19 = 24.74"; **Computed** 24.736842105).
- ρ_hom = 13.9265, ρ_het = 24.0579 (**Cited**, Doedel, Krauskopf and Osinga; they credit the homoclinic explosion to Sparrow 1982 and the 24.06 estimate to Kaplan and Yorke). Sparrow's book itself was not opened: step 1 may confirm the values there.
- Lyapunov exponents at ρ = 28: 0.90563, 0, −14.57219, Kaplan-Yorke dimension 2.06215 (**Cited**, [Sprott](https://sprott.physics.wisc.edu/chaos/lespec.htm)). **Computed** (QR method, 1,000 units): 0.9008, −0.0037, −14.5637, sum −13.66666 against the exact −(σ + 1 + β) = −13.66667, dimension 2.0619. The tests should assert the sum exactly and λ₁ within 0.02.
- Tucker proved, by interval arithmetic and normal forms, that the classical Lorenz system has a strange attractor (Smale's 14th problem), Found. Comput. Math. 2 (2002) 53-117 (**Cited**, [MathWorld news](https://mathworld.wolfram.com/news/2002-02-13/smale14th/), [Tucker's page](https://www2.math.uu.se/~watuc425/main/rodes.html)); his sanity check integrates with standard fourth-order Runge-Kutta, as this dash does.

### 4.2 Eigenvalues the tests can assert (Computed)
| ρ | origin λ₊ | eye: real λ | eye: complex pair | turns per e-fold | e-fold in metres |
|---|---|---|---|---|---|
| 5 | 2.882 | −11.809 | −0.92872 ± 4.1476i | 0.71 | 54 |
| 10 | 5.466 | −12.476 | −0.59550 ± 6.1742i | 1.65 | 84 |
| 14 | 7.160 | −12.878 | −0.39449 ± 7.3270i | 2.96 | 127 |
| 20 | 9.341 | −13.357 | −0.15479 ± 8.7087i | 8.95 | 323 |
| 24 | 10.632 | −13.622 | −0.02253 ± 9.4896i | 67.0 | 2,219 |
| 28 | 11.828 | −13.855 | +0.09396 ± 10.1945i | (outward) | 532 |

Spirals begin at ρ = 1.34562. The catalog's "9 turns at ρ = 20, 67 at ρ = 24" and "spirals from 1.35" check.

### 4.3 The catalog row, rechecked (Computed)
- ρ_H = 24.7368 = 470 ÷ 19: **checks**. Eyes at x = ±8.4853, z = 27 at ρ = 28: **checks**. Butterfly over the last 1 − 24.7368 ÷ 28 = 11.65% of the revs: **checks** (11.7%); 24.0579 ÷ 28 = 85.9%: **checks** ("from 86%").
- Mirror symmetry bit for bit: two points (x, y, z) and (−x, −y, z) in float32 stayed exact negatives after 200,000 steps (1,000 units). **Our reading:** round-to-nearest is sign-symmetric, so this holds with or without fused multiply-adds as long as both points run the same instruction sequence. **Our reading:** the safest form is to integrate one point and draw the other as its mirror; the brief's "no FMA" is then unnecessary.
- Settling from 0.5 units off the eye in any direction: 60 of 60 at ρ = 5, 14, 20, 23 and 24 (**checks**), and still 60 of 60 at 24.3, 24.5 and 24.7 (very slowly).
- Transient capture times from points on the ρ = 28 butterfly (capture = within 0.5 of an eye, 300 points): median 6.9 units (0.34 km) at ρ = 14.5, 11.6 units at 18, 19.4 units (0.97 km) at 20, 49 units (2.45 km) at 22; at 24 only 20 of 300 in 2,000 units. The catalog's "median of 1 km at ρ = 20, most never settle at 24": **checks**.
- Gear pad: 14.5% of trail samples fall inside the Ø200 pad at (0, −40): **checks** ("about 14%").
- **Width at rest and turned: needs a small fix.** With 64 trajectories of 500 units, |x| reaches 19.34, so the figure spans **±580 px** at rest (catalog ±565), **±703 px** at 12° (catalog ±683), and the bound x_max cos 12° + y_max sin 12° = 24.47 units gives **±734 px** (catalog ±715), which is 14 px outside the ±720 cockpit envelope. Rare excursions only, but the catalog row should carry the new numbers and the dash should either use 29 px per unit or fade the trail over the last 20 px of the envelope.
- **Hood: needs a fix.** At the brief's 40 px per unit the figure spans ±773 px at rest, ±883 at 8° and ±937 at 12° (bound ±979), outside the hood envelope's ±880. Either **36 px per unit** (bound ±881 at 12°, at the edge; 35 px gives ±857) or a hood turn limit of about 7.5°. Vertically the hood figure is 7 × 45.6 = 320 px of the 400 available: fine.
- Cockpit heights: z from 1.7 to 47.4 maps to Y = +200 to −206 at 8.9 px per unit about +215, inside the envelope (−320 to +240) and clear of the lamps at −296. About 10% of trail samples sit below Y = +100, near the speed pad.

### 4.4 Integration step and the clock (Computed)
- **Stability.** The largest Jacobian eigenvalue modulus on the attractor is 21.8 (at the origin it is 22.8 at ρ = 28). RK4's real-axis limit is 2.785, so h must stay below about 0.12; empirically no point blew up at h = 0.13 and 40 of 200 did at 0.15. The brief's h = 0.005 sits 25 times inside the limit.
- **Accuracy.** Against h = 0.000625: error 5.4e-6 after one unit at h = 0.005, 1.5e-4 at 0.01, 4.7e-3 at 0.02. Any step departs from the reference by a whole unit after 6 to 15 units (that is chaos, λ₁ = 0.906, not error); the figure's shape does not depend on it.
- **Steps per frame** (clock = max(v ÷ 50 m, 0.5 unit/s)): 0 to 90 km/h: 0.5 u/s, 1.67 steps a frame at 60 Hz, 0.83 at 120 Hz; 200 km/h: 3.7 and 1.85; 300 km/h: 5.6 and 2.8; 500 km/h: 9.3 and 4.6. Use a fixed step with a remainder accumulator, never a variable step, so the mirror and the tests stay deterministic. A whole frame as one step (up to 0.046 units) would still be stable but visibly less accurate.
- **The trail's chord.** 1,000 samples over 20 units is one sample per 0.02 units; on the butterfly the chord between samples is 21 px median and up to 92 px, so a polyline of raw samples looks faceted on the fast outer loops. Store the vector field with each sample (it is free from RK4's k₁) and draw cubic Hermite segments, or keep every step (4,000 samples). **Our reading:** Hermite.
- **The wings come late (the main finding).** At ρ = 28 a point rests on an unstable eye and leaves it by only e per 10.6 units. Ramping ρ from a settled eye to 28 and holding it: first switch to the other wing after 5.6 to 8.1 units from ρ = 10 with a fast ramp (0.5 to 1 unit), 13 to 58 units from ρ = 18 or 22. With the brief's clock a pull lasts about 1 to 2 units (2 to 4 s at 100 km/h), so **during a normal pull and shift the butterfly is never drawn**: the point only loops, widening, round its eye, and the upshift catches it. The butterfly appears only after long running at 86% revs or more (a top-speed run). Section 7 gives fixes; the catalog's Signature line describes a figure the driver would rarely see, and the lead should know.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| ρ (eyes, figure, lamps) | `rpm`, `Signals::shift_rpm` (= max rpm × the shift % setting, default 0.97) | verified | ρ = clamp(28 × rpm ÷ shift_rpm, 0, 28); note `Signals` has one shift rpm, not one per gear as the brief says |
| Eyes on the parabola, numerals | ρ | verified | closed form x = ±√(β(ρ − 1)), z = ρ − 1; numerals at the height of their rpm |
| Model clock | `speed`, `timestamp` | verified | Δt_model = max(v × Δt ÷ 50 m, 0.5 × Δt), Δt from the packet timestamp (u32 ms, wraps after 49.7 days) |
| Clock, alternative | `race_progress.distance_traveled` | decoded, offset confirmed at check C | Δdistance ÷ 50 m; fall back to speed × Δt when it does not advance (free roam may not count it) |
| Catch | `gear` | verified | gear change, or ρ < 24 with a point more than 3 units from its eye |
| Gear numeral | `gear` | verified | R and N as `Signals` encodes them (step 1 confirms) |
| Speed | `speed` | verified | km/h or mph from `AppConfig` |
| View turn | `g_lat` | verified | 12° per g, clamped to ±1 g, through a `ribbon` spring; hood limit per 4.3 |
| Curve brightness | `throttle` | verified | the brief's mapping; capped at 30% of the gear's luminance |
| Wing hue (idea) | sign of x of each point | computed from the model | colour law allows hue for sense of rotation: x is the convection roll's speed and its sign the roll's direction (**Cited**, Cantisán et al. §2: C± "correspond to convection rolls with the two possible directions of rotation") |
| Optional | `yaw_rate`, `torque`, `cylinders` | decoded, offset confirmed at check C | not needed; section 9 Q5 |

**What the game cannot send.** The benchmark is a plot, so nothing on it is missing data, but its physical meaning is: the convection cell's temperature field (y and z are temperature modes, **Cited**, Cantisán et al. §2) and the waterwheel's inflow.
- **The convection cell. Replacement ideas:** (a) **recommended:** the two wings take two hues from the palette ramp, one per sense of rotation of the roll, so a switch of wing is a visible reversal of the convection, as the Malkus wheel reverses; (b) a faint ghost of the Lorenz map (Lorenz Fig. 4: each new z maximum against the last) as rank-5 dots under the speed pad, filling only in the butterfly. Both are the model, not data: labelled MODEL.
- **The waterwheel's inflow. Replacement idea:** the throttle already sets the curves' brightness; nothing more is needed.

## 6. Edge cases

- **Max rpm 0 or tiny:** shift_rpm ≤ 1 gives ρ = 0 (one still light); `Signals` already floors the divisor at 1.
- **Idle near max (shift % high, rpm range narrow):** ρ at idle may be above 14; the figure then rests in slow spirals at idle. Fine, it is still.
- **12,000+ rpm engines and diesels at 4,500:** ρ is a fraction of shift rpm, so the figure is the same; only the numeral column changes.
- **EVs** (idle 0, single gear): no gear change ever catches the points; the "ρ < 24 and more than 3 units out" rule does it on a lift. The start-up lift-off (15 s parked) is worse with idle 0: ρ = 0 until moving.
- **Start-up lift-off:** seed each point 0.5 units from its eye whenever ρ first exceeds 1.5 (the same rule as the catch), rather than relying on the 1e-6 nudge, so the eyes are lit at once.
- **Reverse, neutral, gear above 10:** ρ comes from rpm only; a change into or out of R or N counts as a gear change for the catch.
- **mph and km/h, hp and kW:** the clock uses metres internally; display units only change the speed numeral.
- **Limiter bounce of about 50 rpm a frame:** ρ jitters by about 0.2 at the top, harmless to the integration; the lamps use the shared hysteresis.
- **Spikes, crashes, huge G:** clamp the turn to ±1 g through a spring; a huge speed spike could ask for many steps: cap at 64 steps a frame and drop the remainder.
- **Airborne:** speed holds, clock runs; nothing special.
- **Stale or paused stream:** the app hides the dash; on return, do not integrate the missing time (cap Δt at 0.1 s).
- **Palette extremes, random palettes:** the figure's hue ramp comes from primary, secondary and accent; eyes use `shift` only at the limiter; true-black pads stay black.
- **Font swap:** numerals sit in a fixed column, so a wider font needs the column's right edge fixed, not its left.
- **Cockpit and hood:** hood at 36 px per unit or a 7.5° turn limit (4.3).
- **Reduced flash:** only the lamps change (2 Hz breath); the figure never flashes.
- **Colour only and colour vision:** the wing hues are a bonus; the eyes' height on the parabola and the lamps carry the revs without colour.
- **NaN:** reset a non-finite point to 0.5 units from its eye.

## 7. Signature moments

**Limiter / shift strobe** (only the shift strip strobes, 10 Hz at 50% duty, 2 Hz breath in gentle mode):
- **L1 (recommended, the brief's):** the ten lamps strobe; both eyes hold a steady `shift` glow; the figure keeps drawing, capped at 30% of the gear.
- **L2:** as L1, and at the limiter the trail's head (the newest 0.5 units) takes the `shift` hue, steady, so the newest loops read as "past the shift point" without any flash.

**Gear change:**
- **G1 (the brief's):** the catch: each point set 0.5 units from its eye along the line to where it was, the trail fades over 300 ms, the wings fold in about one loop (0.62 to 0.72 units).
- **G2 (recommended, adds a release):** G1, plus a **release** when ρ rises through 24.74: each point is set 10 units from its eye in the eye's unstable plane (the real and imaginary parts of the outward eigenvector). **Computed:** both wings are then drawn within a median of 2.3 to 3.1 units (p90 3.5 to 5.7) at ρ = 24.8 to 28, against 13 to 58 units without it; a 6 unit kick gives 5.5 units. Pair it with a faster clock in the shift window, one unit per 50 engine revolutions capped at 3 units/s (2.3 u/s at 7,000 rpm, a wing loop every 0.27 s, both wings in about 1 s). Release and catch are mirror rules: the equations draw everything in between. It needs the catalog's maths row amended.
- **G3:** keep the distance clock and no release (honest and calm), and accept that the butterfly appears only on long runs above 86% revs; the pull itself is shown by widening spirals. Least risk, weakest signature.

## 8. What it needs from the 4K library

From the PR #18 branch (`kits.rs`, `assets/clusters/lorenz_butterfly/prebuild/manifest.json`): one computed item, `pad_rim_powder_coat` (PowderCoat surface, seed 330001, 4096² master with height and normal), the labelled grain on the rims of the gear pad and the speed pad. Nothing else; the attractor is the dash's RK4.

Shared parts it can reuse: `jewel_lamp_28_off` and `_on` from the `hakosuka_s20` kit (28 px jewel lenses, made for 10 lamps on 740 × 28; this dash's lamps are 10 on 760 × 28, so the lens fits, only the pitch differs). The Exo 2 and Barlow Semi Condensed fonts are in `assets/fonts/` (variable `Exo2[wght].ttf`), so the MSDF atlases are a pipeline run, not a gap.

LIBRARY GAP: soft glow sprite for the two eyes, 512 px master (128 px runtime), a radially exact falloff with straight alpha, tinted by the kit; the brief assigns it to ComfyUI, but a computed Gaussian-plus-halo is exact and cheaper (asset-forge computed).
LIBRARY GAP: a bespoke Blender asset, which CLUSTER_PROMPT's asset rules require of every cluster while the brief says "Blender: none"; the gear pad's rim bevel at Ø200 (216 px sprite, rest and two G-swing keys) in powder coat over true black is layout-independent because the brief states its size (Blender, hero library).
LIBRARY GAP: the fine grain for the black pads' rims that the brief gives to ComfyUI is covered by the computed `pad_rim_powder_coat`, but a ComfyUI asset is still required by the same rule; a 4096² seamless soft-grain plate for the pads' faces at 2% modulation would satisfy it (ComfyUI SDXL).

## 9. Departures and open questions for the board

1. **The clock and the release (4.4, 7 G2).** The brief's distance clock makes the signature butterfly almost never visible during a pull. The board picks G2 (release plus a faster window clock) or G3; the catalog's maths row and Signature line change with G2.
2. **Hood scale.** 36 px per unit, or a 7.5° turn limit, to stay inside ±880 px (4.3). The cockpit's rare ±734 px bound wants 29 px per unit or an edge fade.
3. **One shift rpm.** `Signals` gives one shift rpm for the car, not one per gear; the brief's "for the gear" needs either that or a learned per-gear table.
4. **Mirror by construction.** Integrate one point and mirror it, instead of two points and a no-FMA rule.
5. **Optional inputs.** `distance_traveled` as the clock (decoded, offset confirmed at check C) only if free roam advances it; yaw rate could drive the turn instead of lateral G, but lateral G is verified and reads the same in steady corners.
6. **Wing hues** by sense of rotation (5): allowed by the colour law; the board checks the ramp at ρ = 28 and that red stays reserved.
7. **Sibling separation.** `HarmonographDrive` (Psychedelic, build #41) draws a closed or slowly precessing Lissajous-like ink figure on a 640 × 80 table under the speed; this dash's figure is open, three-dimensional, fills the envelope and is mostly still at cruise (two lights). `GoldenAngle` and `HopfFibration` are not curves in time. Thumbnail test: two holes and two lights.
8. **What a user can change** without losing the model: trail hue ramp (palette primary, secondary, accent), eye glow colour (`accent`, `shift` at the limiter), trail width (2 to 4 px, `config.extra`), trail length (10 to 20 units, `config.extra`), the turn per g (`config.extra`, 0 to 12°). Defaults are the brief's.

## 10. Sources

Primary maths
- E. N. Lorenz, "Deterministic Nonperiodic Flow", J. Atmos. Sci. 20 (1963) 130-141: https://scispace.com/pdf/deterministic-nonperiodic-flow-1k20vjsax3.pdf
- E. J. Doedel, B. Krauskopf, H. M. Osinga, "Global invariant manifolds in the transition to preturbulence in the Lorenz system" (preprint): https://research-information.bris.ac.uk/ws/files/3021048/dko_preturb_preprint.pdf
- J. Cantisán, J. M. Seoane, M. A. F. Sanjuán, "Transient dynamics of the Lorenz system with a parameter drift", arXiv:2009.11017: https://arxiv.org/html/2009.11017
- J. C. Sprott, "Lyapunov Exponent Spectrum Software": https://sprott.physics.wisc.edu/chaos/lespec.htm
- W. Tucker, "A Rigorous ODE Solver and Smale's 14th Problem", Found. Comput. Math. 2 (2002) 53-117: https://www2.math.uu.se/~watuc425/main/rodes.html; MathWorld news: https://mathworld.wolfram.com/news/2002-02-13/smale14th/
- C. Sparrow, The Lorenz Equations (Springer, 1982), not opened: https://link.springer.com/book/9780387907758

Benchmark and figures
- Wikimedia Commons, "Lorenz attractor yb.svg": https://commons.wikimedia.org/wiki/File:Lorenz_attractor_yb.svg
- Wikipedia, "Lorenz system": https://en.wikipedia.org/wiki/Lorenz_system
- Physiome Model Repository, Lorenz 1963: https://models.physiomeproject.org/w/randall/Lorenz_1963

Waterwheel
- Wikipedia, "Malkus waterwheel": https://en.wikipedia.org/wiki/Malkus_waterwheel
- L. Illing et al., "Experiments with a Malkus-Lorenz water wheel", arXiv:1202.5508: https://ar5iv.labs.arxiv.org/html/1202.5508

Repo
- `program/phases/07-cluster-factory/CATALOG.md` (brief, maths row), `docs/TELEMETRY_USAGE_MATRIX.md`, `crates/fh6-kit/src/cluster.rs` (`Signals::shift_rpm`), PR #18 branch `PREBUILD.md`, `kits.rs` and this dash's prebuild manifest.
