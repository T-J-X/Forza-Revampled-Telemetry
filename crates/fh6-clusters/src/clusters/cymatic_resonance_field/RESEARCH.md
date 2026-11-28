# CymaticResonanceField: research note

Build #14, catalog row 28, category Psychedelic, Target `concept`, family Fluid / Generative, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used throughout: **Cited** means read on that date from the linked source; **Computed** means worked out here with numpy and scipy (`python3 -I`), with the inputs and numbers given so a unit test can repeat them; **Our reading** is a conclusion drawn for this dash, open to the direction check. Anything marked **unconfirmed** is a lead step 1 must check. The web-fetch budget for the session ran out part way through this research, so several sources below were found by title only and are marked so.

## 1. The benchmark

A `concept` dash names one real object for the visual review. **Ours is the modern lecture-hall round Chladni plate: a black-painted metal disc held at its centre, with dry white sand gathered into thin lines on the parts of the plate that do not move.** The University of Toronto's nonlinear physics group shows the archetype: a 70 cm round plate of 0.125 inch (3.2 mm) aluminium, painted black, held at the centre and bowed, photographed with 10-spoke and 14-spoke sand figures (**Cited**, [Toronto, Chladni patterns in vibrated plates](https://www.physics.utoronto.ca/~nonlin/chladni.html)). Chladni's own engraved figures of 1787 are the historical reference for the catalogue of shapes.

What makes the benchmark recognisable:

| # | Feature | Source |
|---|---|---|
| B1 | **Dark plate, light sand.** The plate is matte black; the sand is the only light thing in the picture. | **Cited**, Toronto page (plates "painted black") |
| B2 | **The lines are where nothing moves.** "The sand is thrown off the moving regions and piles up at the nodes"; "only at or near the node is the sand stationary". | **Cited**, Toronto page; J. Wolfe, [UNSW, Chladni patterns](https://newt.phys.unsw.edu.au/jw/chladni.html) |
| B3 | **A round plate draws spokes and rings.** Its figures are straight nodal diameters through the centre and concentric nodal circles; a figure with m diameters shows 2m spokes (the Toronto "10-spoke" and "14-spoke" photos are m = 5 and m = 7). UNSW labels its circular-plate photos by mode, (2,0) and (0,1). | **Cited**, Toronto and UNSW image captions; spoke count **Our reading** from the mode shape cos mθ |
| B4 | **Held at one point, driven at one point.** Toronto's round plate is held at the centre and bowed at the rim; its square, stadium and violin plates are driven from the centre with a shaker. UNSW drives its plates with a 0.2 g magnet in an oscillating coil field, tuned to the resonance. | **Cited**, Toronto; UNSW |
| B5 | **A catalogue of discrete figures.** Chladni published 11 plates and 166 figures in *Entdeckungen über die Theorie des Klanges* (1787) and gave a rule for circular plates. Each figure belongs to one resonance; between resonances the sand does not form a figure. | **Cited**, [APS News, July 2017](https://www.aps.org/apsnews/2017/07/first-experiments-chladni-figures); [Public Domain Review, Chladni Figures (1787)](https://publicdomainreview.org/collection/chladni-figures-1787/) |
| B6 | **Hooke came first.** Robert Hooke bowed a flour-covered glass plate in 1680; Chladni perfected the method, mostly with sand. | **Cited**, Public Domain Review; APS News |

**Our reading.** The benchmark is a near-black picture with thin pale lines, which suits the OLED and the family rule that lit area stays low. Its identity is geometric: straight spokes meeting at a quiet centre, and rings. That identity is what keeps this dash apart from its siblings: `FaradayMandala` draws bright slope highlights on a reflective liquid (its lit cells are where the surface tilts), `KineticLiquidMetal` a chrome blob with thin-film colour, `ChronoMultiverse` ferrofluid spikes. Ours draws the **zero set** of the mode, in **dry particles**, on a **matte solid plate**, with no reflection doing the drawing. Step 1 should keep that contrast explicit on the sheet, because FaradayMandala's idle state is also a Bessel-mode mandala (its note §2.5, C2).

No better real target turned up. Hans Jenny's *Cymatics* (1967), named in the brief, is the source of the word; what its photographs show (sand and liquids on driven plates, from memory) was not opened or checked here (**unconfirmed**), and liquid figures belong to FaradayMandala, so only Jenny's sand-on-plate work is relevant.

## 2. Reference photos

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 1 | Round black aluminium plate, 70 cm, centre-held, bowed: 10-spoke figure (m = 5) | https://www.physics.utoronto.ca/nonlinear/chladni/round_10spoke.jpg | no |
| 2 | The same plate, 14-spoke figure (m = 7) | https://www.physics.utoronto.ca/nonlinear/chladni/round_14spoke.jpg | no |
| 3 | Toronto apparatus schematic (plate, centre mount, driver) | https://www.physics.utoronto.ca/nonlinear/chladni/chladni_app.gif | no |
| 4 | Uniform circular plate, mode (2,0): two crossed diameters | https://newt.phys.unsw.edu.au/jw/graphics/Mode.20.CP.GIF | no |
| 5 | Uniform circular plate, mode (0,1): one nodal circle | https://newt.phys.unsw.edu.au/jw/graphics/Mode.01.CP.GIF | no |
| 6 | Chladni 1787, plate of nodal-line figures (Public Domain Review scan) | https://pdr-assets.b-cdn.net/collections/chladni-figures-1787/35835910874_1b7b4201e7_o.png | no |
| 7 | Chladni 1787, a second plate of figures | https://pdr-assets.b-cdn.net/collections/chladni-figures-1787/36501469562_d783356518_o.png | no |
| 8 | Square black plate, centre-driven by a shaker, 1225.0 Hz (the square-plate fallback figure) | https://www.physics.utoronto.ca/nonlinear/chladni/square_1225.0.jpg | no |
| 9 | Square plate, 3139.7 Hz: a dense figure, for the moiré risk | https://www.physics.utoronto.ca/nonlinear/chladni/square_3139.7.jpg | no |
| 10 | A real sand photograph on a Chladni plate (COMSOL blog) | https://cdn.comsol.com/wordpress/2018/08/chladni-pattern-2.png | no |
| 11 | Leissa, *Vibration of Plates* (NASA SP-160, 1969), ch. 2, Tables 2.5 and 2.7: free circular plate eigenvalues and nodal-circle radii | https://apps.dtic.mil/sti/tr/pdf/ADA307623.pdf | no (a PDF table, linked) |

Drive `references` folder: https://drive.google.com/drive/folders/1ME-yh7PTN2vhE5R4Hnf6RaG57sQy_OIS (`SOURCES.md` is there with the same list). No image file could be saved: the Toronto and Public Domain Review images returned a server error, and the shared web-fetch budget was exhausted for the UNSW ones. Step 1 saves rows 1, 2, 4, 5 and 6 first; they are the benchmark.

## 3. How the real thing behaves, with motion notes

A Chladni plate has no start-up screen or gear; these are the plate's own behaviours, then what each driving state should do on the dash.

- **R1, figures exist only at resonance.** The drive is tuned to a mode's frequency (UNSW: "an oscillating force whose frequency is tuned to the resonance of the mode"). Off resonance the plate barely moves and the sand stays where it lies. **Cited**, UNSW.
- **R2, the old figure persists.** Sand that is not thrown does not move, so a figure stays drawn after the drive stops. **Our reading** from B2: the sand only moves where the plate's acceleration throws it (§4.4).
- **R3, re-forming takes the sand some time.** Grains migrate by repeated hops, so a new figure gathers over many drive cycles. No source with a measured migration time for sand in air was found (**unconfirmed**: step 1 looks for one, for example in Rossing's work or a lecture-demonstration measurement). Dorrestijn et al. 2007 is not that source: it studies particles in a liquid at the nanoscale, where nanoparticles go to the nodes and micron-sized particles to the antinodes, "in contrast with the macroscopic observations in the gaseous environment" (**Cited**, abstract, [Purdue e-Pubs](https://docs.lib.purdue.edu/nanopub/275), PRL 98, 026102). The brief's 300 ms re-form is a design time, probably faster than real sand (unconfirmed), and goes in the handover as a departure.
- **R4, fine powder goes the other way.** Faraday (1831) studied "certain forms assumed by groups of particles upon vibrating elastic surfaces" (title **Cited** through the FaradayMandala note's OCR source). When very light particles are sprinkled on a resonating horizontal plate they form inverse Chladni patterns: instead of going to the nodal lines they are dragged to the antinodes by the air currents the vibration induces; the cycle-averaged Lagrangian velocity of the air points to the antinodes, while the time-averaged Eulerian velocity points the other way, toward the nodal lines (**Cited**, abstract of H. J. van Gerner, K. van der Weele, M. A. van der Hoef and D. van der Meer, "Air-induced inverse Chladni patterns", J. Fluid Mech. 689 (2011) 203 to 220, [Cambridge Core](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/airinduced-inverse-chladni-patterns/2A1779B2D6EB029119AC8C640BAF4B5F)). **Our reading:** an optional second, finer particle population would give the dash an "inverse" layer at the antinodes; it is risky next to FaradayMandala and is left to the board (§9, Q6).

State by state on the dash (Our reading unless marked):

| State | Plate behaviour |
|---|---|
| Start-up / greeting | The plate is clean and dark; on the first packet the grains are sprinkled (seeded by blue noise) and stay where they fall, because idle throttle is below the hop threshold. A single short drive pulse settles them onto the idle mode in about 300 ms, once. |
| Idle | Still. Throttle near 0 gives Γ below 1 (§4.4): nothing hops, the last figure stays drawn, crisp but not refreshed. Idle is still, as the category demands. |
| Cruise | Light throttle: Γ a little above 1, grains hop only near the antinodes, the lines are broad and soft. |
| Pull to the limiter | Full throttle: Γ high, the resting band narrows to a razor line. The mode steps up through its bands as rpm rises (§4.3), each step a re-form. |
| Limiter / shift | The plate holds the top mode, (6,0): twelve spokes. Grains warm steadily toward `shift` (no strobe on the figure). Only the tine strip strobes (§7). |
| Gear change | An upshift drops rpm, so the mode steps down a band: one throw (all grains airborne, Γ forced above 1 everywhere for one hop of about 120 ms) and the grains land on the new mode's lines. |
| Braking | Throttle 0, so no hop; the brief's gLong creep along the tilt is a body force that moves grains only while they are airborne, so on a lift with no hop, nothing creeps. Option: braking adds a small hop from deceleration (§5). |
| Drift / lateral G | Steer and gLat turn θ0, the angle of the nodal diameters (§4.5). On a real plate the bow and the damping finger set that angle. |
| Reverse / neutral | Neutral: drive off, figure persists, mode index frozen. Reverse: the same plate, the glyph R; no special figure. |
| Shut-down | Stream stale or `race_on` false: the app hides the dash (house rule). No plate animation. |

## 4. The maths and physics

### 4.1 The model, and the catalog row checked

Kirchhoff–Love plate, D∇⁴w + ρh ∂²w/∂t² = 0, D = Eh³ ÷ (12(1 − ν²)). For a circular plate of radius a the modes are W(r, θ) = [J_m(λr/a) + C·I_m(λr/a)] cos m(θ − θ₀), with ω = (λ²/a²)√(D/ρh). The free edge needs zero radial bending moment, R″ + ν(R′/r − m²R/r²) = 0, and zero Kirchhoff shear, (∇²R)′ − (1 − ν)m²(R′ − R/r)/r² = 0, at r = a. The 2 × 2 determinant in (1, C) is the frequency equation; C follows from the moment condition. **Cited** for the setting (Leissa, NASA SP-160, §2.1.3: free edge, M_r(a) = 0 and V_r(a) = 0, the frequency equation in J_n and I_n; λ² = ωa²√(ρ/D)), the equations as written **Computed** and checked against Leissa's numbers below. The brief's J_m(k r) cos mθ is a membrane, as the catalog row already says.

**Computed** (scipy `jv`, `iv`, bisection then `brentq`, ν = 0.30; m = nodal diameters, n = nodal circles counted as zeros of R inside the plate):

| Mode (m, n) | λ² | λ | C | nodal circle radius r/a | ratio to (2,0) |
|---|---|---|---|---|---|
| (2,0) | 5.3583 | 2.31481 | 0.213424 | none | 1.0000 |
| (0,1) | 9.0031 | 3.00052 | −0.085675 | 0.6799 | 1.6802 |
| (3,0) | 12.4390 | 3.52689 | 0.091142 | none | 2.3214 |
| (1,1) | 20.4746 | 4.52488 | −0.019217 | 0.7811 | 3.8211 |
| (4,0) | 21.8352 | 4.67281 | 0.042801 | none | 4.0750 |
| (5,0) | 33.4949 | 5.78748 | 0.021071 | none | 6.2510 |
| (2,1) | 35.2601 | 5.93802 | −0.005579 | 0.8224 | 6.5804 |
| (0,2) | 38.4432 | 6.20026 | 0.003151 | 0.3909, 0.8415 | 7.1745 |
| (6,0) | 47.3780 | 6.88317 | 0.010682 | none | 8.8419 |

- **The catalog row checks.** λ² = 5.3583 (2,0), 9.0031 (0,1), 12.4390 (3,0) for ν = 0.30 are confirmed to four decimals. No correction is needed to the maths row.
- **Against the published table (ν = 0.33).** The same code gives 5.2620 (2,0), 12.2439 (3,0), 21.5272 (4,0), 33.0618 (5,0), 46.8087 (6,0), 35.2425 (2,1), 52.9210 (3,1), 9.0689 (0,1), 20.5131 (1,1). Leissa's Table 2.5 (ν = 0.33, "values true within 2 percent") prints 5.253, 12.23, 21.6, 33.1, 46.2, 35.25 and 52.91 for the same modes; every value agrees within 1.3%. His Table 2.7 nodal radii begin 0.680, 0.841, 0.391; ours for ν = 0.33 are 0.6795 for (0,1) and 0.8412, 0.3906 for (0,2). **Cited** (Leissa, read through the DTIC OCR, whose table layout is scrambled: step 1 confirms the cell positions on the scan) and **Computed**.
- **The rigid modes.** A free plate's (0,0) and (1,0) are rigid-body motions (λ = 0), which is why the list starts at (2,0). **Computed** (no elastic root below 2.3 for m = 0, 1).
- **Symmetry the tests can assert.** W(r, θ + π/m) = −W(r, θ): **Computed** for (3,0) at r = 0.7, error 5.6 × 10⁻¹⁷. Mode (m, n) has m nodal diameters (2m spokes) and n nodal circles, as the catalog row says.

### 4.2 Chladni's law

f = C(m + 2n)^p, with p roughly 2 for flat circular plates and 1.4 to 2.4 for cymbals and bells. **Cited**, [Wikipedia, Chladni's law](https://en.wikipedia.org/wiki/Chladni%27s_law) and `docs/specs/optics.md` §12.2 [S20]; the primary source is T. D. Rossing, "Chladni's law for vibrating plates", Am. J. Phys. 50 (1982) 271 (by title, **unconfirmed**). **Computed** from the table above: p = 1.957 over the n = 0 family (m = 2 to 8), 1.875 over the n = 1 family, 1.842 over the first 14 modes together. The law is a fit, not the model: the dash uses the exact λ², and a test can assert that the fitted p over the n = 0 family lies between 1.9 and 2.0.

### 4.3 Which mode sounds: mapping rpm to a figure

- The brief: 9 discrete bands across idle → shift with 120 rpm hysteresis. `optics.md` §12.2: drive at the engine's firing frequency and pick the integer mode whose frequency is nearest. The two can be made the same thing.
- **Computed:** the nine lowest modes span λ² 5.3583 to 47.378, a ratio of **8.842**. A petrol car from 800 rpm idle to a 6,790 rpm shift point (7,000 × 0.97) spans 8.49. So one honest plate constant nearly fits the whole climb.
- **Recommended mapping (Our reading): tune the plate to the car.** Set the plate constant once per car so the top mode (6,0) sounds at `shift_rpm`; the target is λ²* = 47.378 × rpm ÷ shift_rpm, and the mode is the nearest λ² in log distance, with the band edges at geometric midpoints and 120 rpm hysteresis. This is a real plate whose thickness or radius has been chosen for the car (ω ∝ h ÷ a²), the frequency stays linear in rpm, and no cylinder count is needed. **Computed** band edges (rpm) for shift 6,790: 995, 1,517, 2,287, 3,030, 3,876, 4,925, 5,276, 6,116; idle 800 sits in (2,0). The narrowest band, (2,1), is 351 rpm wide, so the 120 rpm hysteresis fits. For a diesel (idle 800, shift 4,365) idle already sits in (0,1), whose band starts at 640; for a 12,000 rpm engine (idle 1,000, shift 11,640) idle lies below the (2,0) centre and is clamped to it.
- **Alternative M2 (the brief's literal reading):** nine equal bands in log rpm from idle to shift. Every car shows all nine figures, but the spacing no longer follows the plate's own frequencies. The board picks; the handover names the choice.
- **Real plate scale, for the motion notes.** **Computed** (aluminium, E 69 GPa, ρ 2,700 kg/m³, ν 0.33): the Toronto plate (r 0.35 m, h 3.175 mm) has its (2,0) at 33.6 Hz; a 1.5 mm aluminium plate tuned so (2,0) answers a four-cylinder's 26.7 Hz firing at 800 rpm has r = 0.270 m. The constants are textbook values, not read from a source here (**unconfirmed**).

### 4.4 The sand: when grains hop and how thin the lines are

- **Model (Our reading, to be sourced at step 1):** a grain on the plate leaves the surface when the plate's downward acceleration exceeds g, so with W normalised to max |W| = 1 and Γ = Aω² ÷ g (peak acceleration at the antinode over g), grains hop where Γ|W| > 1 and rest where Γ|W| ≤ 1. Repeated hops carry them down the gradient of W², toward the nodal lines. No source for this hop model was read (**unconfirmed**); step 1 finds one (R3).
- **Computed: the fraction of the plate where grains rest** (801 × 801 grid over the disc):

| Mode | Γ = 1.5 | Γ = 3 | Γ = 10 | Γ = 30 |
|---|---|---|---|---|
| (2,0) | 0.851 | 0.550 | 0.225 | 0.093 |
| (0,1) | 0.801 | 0.390 | 0.116 | 0.039 |
| (0,2) | 0.962 | 0.665 | 0.175 | 0.058 |
| (6,0) | 0.919 | 0.725 | 0.456 | 0.297 |

- So the brief's "soft and fuzzy on a lift, razor-thin at full throttle" is the physics: throttle sets Γ, and the resting band narrows as 1 ÷ (Γ|∇W|). Γ ≤ 1 means no hop at all, which gives the still idle.
- **The quiet centre.** For high m, J_m(λr) grows like r^m, so the middle of the plate barely moves: at Γ = 30 almost 30% of the (6,0) plate still holds sand, mostly a central disc where the spokes merge. **Our reading:** this is what the gear sits on; the levitating numeral above a calm centre is physically right, and the board can size the gear's dimming pad from it.
- **Airtime.** **Computed:** a hop of 120 ms (the brief) needs a launch speed of 0.588 m/s and rises 17.7 mm. On a 0.27 m plate drawn 1,100 px across, that is 31.7 px on screen at a 62° tilt (sin 62° of the normal) and 34.6 px at 74.2°. At 26.7 Hz it needs A = 3.5 mm (Γ = 10); at 300 Hz, A = 0.31 mm (Γ = 113).

### 4.5 The angle of the figure

For m ≥ 1, cos mθ and sin mθ have the same frequency on a perfect disc, so the figure may sit at any θ₀; on a real plate the bowing point and the damping finger fix it (B4). An m = 0 figure has nothing to turn. **Our reading** from the mode shape. So steer and gLat turning θ₀ (±20°) is honest, and it reads as "the bow moving round the rim".

### 4.6 The centre clamp changes the ring modes

The brief puts a clamp at the centre, and the benchmark plate is held there (B4). **Computed** (annular plate, clamped at r = b, free at r = a, J, Y, I and K terms, 4 × 4 determinant, ν = 0.30): for m ≥ 3 the clamp changes nothing (12.439, 21.835, 33.495, 47.378 at b/a = 0.05, as free); (2,0) moves from 5.358 to 5.424; but the ring modes change completely: a new umbrella mode at λ² 3.923 (m = 0) and a rocking mode at 2.831 (m = 1) appear, and the first one-ring mode moves from 9.003 to 22.642. At b/a = 0.10 the umbrella mode is 4.237, which matches the value we remember for Leissa's annular clamped-free table (**unconfirmed**: step 1 reads it). **Our reading:** a centre-driven plate (a shaker under the clamp) excites only the m = 0 modes by symmetry, and the spokes appear when the rim is bowed, which is why the Toronto round plate is bowed. The board picks one honest story: (a) the free-plate modes with the clamp as a decorative crest boss (a departure), or (b) the clamped-centre modes, solved at load by the same code with b from the crest's size. Recommended: (b), since the solver is the same and the spoke modes, which carry the identity, are unchanged.

### 4.7 The tilt in the brief is inconsistent

**Computed:** a circle seen with its normal at 62° to the view axis projects to minor ÷ major = cos 62° = 0.4695, so a 1,100 px plate is 1,100 × 516, not 1,100 × 300. The brief's 1,100 × 300 is a tilt of 74.17°; the hood's 1,500 × 220 is 81.57°. This is an error in the brief's layout numbers, not in the maths row; the lead reports it, and the board picks one of the two.

### 4.8 Values the unit tests will assert (no GPU)

λ² for (2,0), (0,1), (3,0) at ν = 0.30 as above (5.3583, 9.0031, 12.4390) and the six further values of §4.1; zero bending moment and zero Kirchhoff shear at r = a for each solved mode (residual below 1e-6 of the term scale); the nodal radii 0.6799 (0,1) and 0.3909, 0.8415 (0,2); W(θ + π/m) = −W; the count of sign changes along a circle at r = 0.9 equals 2m; under the recommended mapping the mode picked at shift_rpm is (6,0), and at 0.5 × shift_rpm (λ²* = 23.689) it is (4,0) (log distance 0.082, against 0.146 to (1,1) and 0.346 to (5,0)); Γ ≤ 1 gives zero hops; `chladni_square(m = n)` is zero everywhere (already in `fh6_kit::optics`).

## 5. Telemetry mapping

Since main 60c99c9 the decoder reads all 88 FH6 Data Out fields (`crates/fh6-core/src/telemetry.rs`). Fields marked "long-verified" match the golden packets; the newly decoded ones are labelled "decoded, offset confirmed at check C" (the real-game check at the PC), each with a fallback from the long-verified fields.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Near-rim tach ribbon | `rpm`, `max_rpm` | long-verified | `ribbon` spring (T 0.07, ζ 0.9) with feed-forward; exact |
| Mode index (the figure) | `rpm`, `shift_rpm`, `idle_rpm` | long-verified | λ²* = 47.378 × rpm ÷ shift_rpm, nearest mode by log, 120 rpm hysteresis (§4.3) |
| Tine strip (15 tuning forks) | `rpm`, per-gear thresholds | long-verified; thresholds learned live | outside-in, 0 ms attack; limiter strobe 10 Hz |
| Gear glyph and re-tune throw | `gear` | long-verified | glyph changes in frame 1; a change starts one forced hop |
| Speed | `speed`, unit | long-verified | tabular, hysteresis on digits |
| Line sharpness (Γ) | `throttle` | long-verified | Γ = 1 + k × throttle (k a design constant, say 20), so throttle 0 is no hop |
| Figure angle θ₀ | `steer`, `g_lat` | long-verified | `light` spring (T 0.15, ζ 1), ±20°, only for m ≥ 1 |
| Grain creep along the tilt | `g_long` | long-verified | a body force on airborne grains only |
| Grain hop impulses, per corner | `surface_rumble_fl/fr/rl/rr` (bytes 148 to 160) | decoded, offset confirmed at check C | each wheel's rumble throws a local hop at one of four sprinkle points on the rim (front pair on the far rim, rear pair on the near rim), amplitude ∝ rumble, zero on smooth tarmac, so idle is still. **Fallback:** a whole-plate hop where `acceleration_y` (long-verified) departs from 1 g by more than a threshold. |
| Kerb strike | `wheel_on_rumble_strip_*` (116 to 128, a 0/1 flag) | decoded, offset confirmed at check C | the flag's rising edge throws one sharper hop at that corner's sprinkle point. **Fallback:** the `acceleration_y` hop. |
| Bump hops (the brief's suspension velocity) | `normalized_suspension_travel_*` (68 to 80), differentiated over the timestamp | decoded, offset confirmed at check C | a hop at that corner when \|d(travel)/dt\| passes a threshold. **Fallback:** the `acceleration_y` hop. |
| Airborne hold | `normalized_suspension_travel_*` all near full extension | decoded, offset confirmed at check C | no new hops while airborne; the landing throws once. **Fallback:** `acceleration_y` near 0 g. |
| Drive-frequency micro-label (optional) | `rpm`, `num_cylinders` (228) | decoded, offset confirmed at check C | f = rpm ÷ 60 × cylinders ÷ 2 in Hz, exact for a four-stroke piston engine; shown beside the "(m, n)" label. **Fallback:** the label is hidden and only "(m, n)" shows. |
| Grain warmth at the limiter | `rpm` ≥ `shift_rpm` | long-verified | steady ramp to `shift`, no strobe |

Not used: tyre slip, slip ratio, temperatures, fuel, laps, position, `torque` (the plate does not need them). `wheel_in_puddle_*` is decoded too (a flag, not a depth); an optional idea for the board is that a wet corner damps its sprinkle point's hops (damp sand clumps), labelled MODEL.

**What the game cannot send.** With all 88 fields decoded, every input the brief names is in the packet (surface rumble and suspension above). What remains absent is the benchmark's own bench equipment:

- **The signal generator's frequency readout.** **Replacement, recommended: a micro-label of the live mode, "(m, n)"**, which is exact. Second idea: the engine's firing frequency in hertz from `num_cylinders` (above), exact once check C confirms the field; the plate's own resonance frequency would be MODEL, because it depends on the plate constant the dash chose.
- **The bow and the damping finger** that set the figure's angle. **Replacement, recommended:** steer and gLat turn θ₀ (§4.5), drawn as a small rim mark where the "bow" sits. Second idea: the rim mark follows the body slip angle β = atan2(v_x, v_z) in a drift (long-verified velocity).
- **The amplitude knob.** **Replacement:** throttle (above), exact.

Nothing is faked: the figure is MODEL (a real plate's modes chosen by rpm) and the dash labels it so in its handover.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| `max_rpm` 0 or tiny, or `idle_rpm` ≥ `max_rpm` | Ribbon and tines hidden; the figure holds (2,0) still; gear and speed still drawn. Clamp rpm to [0, max(max_rpm, idle_rpm + 1)] before any division. |
| Idle near max | The recommended mapping needs only shift_rpm, so it still works; idle just sits in a higher band. |
| 12,000+ rpm engines | Idle clamps to (2,0); the climb still ends at (6,0) on the shift point. |
| Diesels (4,500) | Idle starts at (0,1): the climb is shorter, and honest. |
| EVs (single gear, no idle, odd rpm) | The same mapping on motor rpm; no gear change, so no throw except on bumps; if rpm is 0 at a standstill the figure holds. `num_cylinders` is presumably 0 (unconfirmed until check C): the Hz label is hidden. |
| Rotary or odd cylinder counts | Show the Hz label only for `num_cylinders` in 1..=16; anything else hides it. What the field holds for a rotary is unknown until check C. |
| Reverse / neutral | Neutral freezes the mode index (no hop from the drive; road hops still apply); reverse is the glyph R on the same plate. |
| Gear above 10 | `Gear::from_raw` gives neutral; treat as neutral. |
| mph and km/h | Speed only; the plate does not read speed. |
| hp and kW | Not used. |
| Limiter bounce (about 50 rpm a frame) | Inside the 120 rpm hysteresis, so the figure does not flicker between bands; digits use hysteresis. |
| Spikes and crashes (huge G) | Cap θ₀ at ±20° and every hop impulse (rumble, kerb, suspension, `acceleration_y`) at one throw per 300 ms; a crash re-scatters the grains once, then they settle. |
| Surface rumble or suspension wrong or zero at check C | Switch the hop source to the `acceleration_y` fallback; the plate still reads, still idle-still. |
| Airborne | Suspension at full extension (or `acceleration_y` near 0 g in the fallback) reads as weightless: hold all grains; the landing throws them once. |
| Stale or paused stream | The app hides the dash. |
| Palette extremes and random palettes | Grains are `inkMuted` tinted 18% `primary` (brief); with a dark primary keep the grains' luminance floor so lines stay visible; the colour law allows hue only for a model quantity (mode number), so a mode-number tint is optional. |
| Font swap | Gear, speed and ribbon numerals use palette roles and the swappable font; the "(m, n)" and Hz labels use `text`. |
| Cockpit and hood | Hood plate 1,500 × 220 (tilt 81.6°, §4.7): at that tilt the spokes foreshorten strongly, so modes with rings read best; the board checks legibility. |
| Reduced flash | Tines breathe at 2 Hz; the re-tune throw and road hops stay (motion, not flash) but their brightness change is capped. |
| Colour-only cues, colour vision | Shift is a tine count; the mode is a shape; nothing relies on hue. |

## 7. Signature moments

**Limiter (only the tine strip strobes).**
- **L1, recommended: the plate holds (6,0) and the forks ring.** The twelve-spoke figure stays razor-thin and still; the grains warm steadily to `shift`; the 15 tines strobe at 10 Hz, 50% duty (2 Hz breath in reduced flash). The tuning forks are the honest strobe: a fork's tines are what rings.
- **L2: the quiet centre grows.** As rpm passes shift, Γ is held at its maximum and the spokes reach further in toward the gear (§4.4), a steady change, not a flash.

**Gear change.**
- **G1, recommended: the re-tune throw (the brief).** Frame 1: the glyph changes. Then one hop of about 120 ms with every grain airborne (a 32 to 35 px rise on screen, §4.4), and the grains land on the new, lower mode. The figure re-forms in about 300 ms (a design time; no measured time for real sand was found, R3).
- **G2: the bow moves.** On a change the rim mark (the "bow") jumps to the new mode's antinode, and the figure turns by π ÷ (2m) as it lands, so each gear reads as a fresh bowing.
- **G3: a ring sweep.** For an m = 0 landing, the grains arrive from the rim inward along the gradient of W², so the new circle draws itself as a closing ring.

## 8. What it needs from the 4K library

There is **no pre-build kit** for this dash: `PREBUILD.md` and `kits.rs` on `claude/project-thread-ib67x9` list none, and say "CymaticResonanceField's rim ... [is an] optional profile strip, and the brief's analytic bevel is the default". There is no `assets/clusters/cymatic_resonance_field/prebuild/manifest.json`.

Shared parts it can reuse (from the `kinetic_liquid_metal` and `faraday_mandala` kits): `chrome_matcap` and `chrome_fresnel` for the centre clamp and any bright metal, `liquid_metal_studio_env` and `liquid_metal_env_band` for what the plate's polished rim mirrors.

Computed at load by the dash (not a library item): the λ and C table, 512-sample radial profiles and their gradients in an RGBA16F LUT, the clamped-centre solve.

LIBRARY GAP: black-chrome (or black anodised aluminium) plate swatch, a 4096² radial disc with normal, roughness and metal passes and a fine lapped or sunray finish, so the dark plate reads as machined hardware under the matte sand; Blender (asset-forge prebuild swatch).
LIBRARY GAP: tuning-fork tine sprites, 10 × 36 px per the brief, at the rest key and the two G-swing keys (2 px per 4K px master, box-filtered to 1:1), lit and unlit; Blender (prebuild sprite).
LIBRARY GAP: 128² blue-noise tile for grain seeding and dither, computed exactly on the CPU (void-and-cluster); asset-forge prebuild (computed).
LIBRARY GAP: grain macro texture, a seamless 4096² master of fine dry white quartz sand on black, used only as per-grain sparkle and size variation (it never places a grain: colour law rule 5); ComfyUI (SDXL), which also meets the "one bespoke ComfyUI asset" rule.

Layout-dependent and not a gap: the centre clamp boss and crest (its size comes from the board), the rim bevel if a profile strip is used.

## 9. Departures and open questions for the board

1. **Tilt.** 62° and 1,100 × 300 disagree (§4.7); pick one.
2. **Mapping.** Tune the plate to the car (recommended, §4.3) or nine equal log bands (the brief's literal reading).
3. **Centre clamp.** Free-plate modes with a decorative boss, or the clamped-centre modes (recommended, §4.6). Either way the ring modes are named in the handover.
4. **Re-form time.** 300 ms is a design time; real sand is probably slower, but no measured time was found (R3, unconfirmed).
5. **Road hops.** Surface rumble, rumble-strip flags and suspension velocity drive the grain hops (§5), decoded since main 60c99c9 with offsets confirmed at check C; vertical acceleration is the fallback if check C fails.
6. **Inverse layer.** A second, finer powder at the antinodes (R4) is physical and pretty, but it lights the antinodes, which is FaradayMandala's language; recommended off.
7. **Render path.** 20k grains each frame: `Cluster::update` gets no device, so either a CPU loop over 20k grains drawn through a new instanced-points kit paint (route d, shared code) or nine settled density textures built at load (route b) with a CPU particle overlay only during a throw. The thread asks the coordinator early.
8. **Hood legibility.** At 81.6° the spokes are nearly edge-on; the hood may prefer ring-bearing modes or a flatter view.

## 10. Sources

Benchmark and photographs
- University of Toronto, nonlinear physics, "Chladni patterns in vibrated plates": https://www.physics.utoronto.ca/~nonlin/chladni.html
- J. Wolfe, UNSW, "Chladni patterns": https://newt.phys.unsw.edu.au/jw/chladni.html
- Public Domain Review, "Chladni Figures (1787)": https://publicdomainreview.org/collection/chladni-figures-1787/
- APS News, "This Month in Physics History", July 2017: https://www.aps.org/apsnews/2017/07/first-experiments-chladni-figures
- COMSOL blog, "How do Chladni plates make it possible to visualize sound?": https://www.comsol.com/blogs/how-do-chladni-plates-make-it-possible-to-visualize-sound

Plate theory
- A. W. Leissa, *Vibration of Plates*, NASA SP-160 (1969), ch. 2, Tables 2.5 and 2.7: https://apps.dtic.mil/sti/tr/pdf/ADA307623.pdf
- Wikipedia, "Chladni's law": https://en.wikipedia.org/wiki/Chladni%27s_law
- `docs/specs/optics.md` §12.2 and its sources [S18] P. Bourke, http://paulbourke.net/geometry/chladni/, [S19] https://en.wikipedia.org/wiki/Vibration_of_plates, [S20] as above

Particles on a vibrating plate (abstracts read)
- H. J. van Gerner, K. van der Weele, M. A. van der Hoef and D. van der Meer, "Air-induced inverse Chladni patterns", J. Fluid Mech. 689 (2011) 203 to 220: https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/airinduced-inverse-chladni-patterns/2A1779B2D6EB029119AC8C640BAF4B5F
- M. Dorrestijn et al., "Chladni figures revisited based on nanomechanics", PRL 98, 026102 (2007), particles in liquid, not sand in air: https://docs.lib.purdue.edu/nanopub/275

Found by title only (step 1 reads them)
- T. D. Rossing, "Chladni's law for vibrating plates", Am. J. Phys. 50 (1982) 271
- "Point-driven modern Chladni figures with symmetry breaking" (open access): https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6052176/
- H. Jenny, *Kymatik / Cymatics* (1967)
- M. Faraday, "On a peculiar class of acoustical figures; and on certain forms assumed by groups of particles upon vibrating elastic surfaces", Phil. Trans. 121 (1831), OCR: https://jimclifford.ca/philosophical-transactions-ocr-1665-1869/articles/jstor-107936/ (cited in the FaradayMandala note)

Repo
- `program/phases/07-cluster-factory/CATALOG.md` (brief, row 28, maths row), `docs/design/families/fluid-generative.md`, `data/design_tokens.json` (springs, limiter strobe), `crates/fh6-kit/src/cluster.rs` (`Signals`, `FrameInput`), `crates/fh6-core/src/telemetry.rs` (decoded offsets), `tests/golden/packets/*.bin`
- PREBUILD.md and `kits.rs` on `origin/claude/project-thread-ib67x9`; FaradayMandala and KineticLiquidMetal notes on `origin/claude/project-thread-hqrmcu` and `-nrluim`
