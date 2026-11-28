# Chrono Event Horizon: research note

Build #19, catalog row 22, category Sci-Fi, Target `concept`, family Sci-Fi (FUI / film dialect), view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used throughout. **Cited** means read on 2026-11-28 from the source linked. **Computed** means worked out here from cited equations with `python3 -I` and numpy/scipy; inputs and outputs are given so step 1 can re-run them (the scripts are short: the exact Schwarzschild orbit integral, a root find for image positions, nothing else). **Our reading** is a conclusion for this dash, open to the direction check. Anything marked **unconfirmed** was not read in a source today and step 1 must check it.

## 1. The benchmark

A `concept` dash names one real object for the visual review. **Ours is the Gargantua disk image from Interstellar, as published by its makers** in James, von Tunzelmann, Franklin and Thorne, "Gravitational lensing by spinning black holes in astrophysics, and in the movie Interstellar", Classical and Quantum Gravity 32, 065001 (2015), open access under CC BY 3.0 (**Cited**, [IOP supplementary page](https://iopscience.iop.org/article/10.1088/0264-9381/32/6/065001/data)). It is the image most people already picture when they hear "black hole", and its composition (a thin disk seen almost edge-on, lensed over and under a black shadow) is exactly the brief's.

What makes it recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | A razor-thin disk seen nearly edge-on crosses **in front** of the shadow as a straight bright band. | **Cited**: the EHT FAQ calls the film's disk "razor-thin" ([EHT FAQ](https://eventhorizontelescope.org/faq/how-realistic-are-movie-depictions-black-holes-eg-interstellar)); Luminet describes the film's disk "spreading above, below and in front of Gargantua" ([Luminet blog 3/6](https://blogs.futura-sciences.com/e-luminet/2016/01/10/warped-science-interstellar-36-accretion-disk-tidal-stress/)) |
| B2 | The far side of the disk is not hidden: its light is bent **over the top** of the shadow into a broad arch, and the underside shows **below** the shadow as a second, thinner arc. DNEG's own words: thin disks "get warped into rainbows of fire that stretch over and under the black hole". | **Cited**, [DNEG news page](https://www.dneg.com/news/gravitational-lensing-by-spinning-black-holes); Luminet blog 3/6 (secondary image "a highly deformed picture of the bottom of the disk") |
| B3 | A thin bright ring hugs the shadow edge: light that has circled the hole once or more. Successive rings "grow progressively fainter and thinner". | **Cited**, [NASA SVS 13326](https://svs.gsfc.nasa.gov/13326/) (photon ring text); [Johnson et al. 2020](https://arxiv.org/abs/1907.04329) abstract: subrings "becoming exponentially narrower but weaker" |
| B4 | **Symmetric brightness**: the film-makers left out the Doppler and gravitational shifts, so neither side is brighter. "the Doppler effect is switched off in the movie". | **Cited**, EHT FAQ; Luminet blog 3/6 (a conscious decision to leave out the Einstein and Doppler shifts "and have an accretion disk with the right shape but not the right lopsidedness") |
| B5 | Gargantua spins at a/M = 0.6 in the film, slowed from near-maximal; the paper adds the colour shifts and then the intensity shifts as separate steps. | **Cited**, [ar5iv of arXiv:1502.03808](https://ar5iv.arxiv.org/html/1502.03808) §1.2 |

**Our reading, and a better target for one rule.** The brief departs from Gargantua on purpose in one respect: it keeps Doppler **beaming as intensity only**. That matches two other real images better than Gargantua: Luminet's 1979 bolometric (black-and-white) image, "the first computer simulation of a black hole accretion disk", where one side is far brighter, and NASA Goddard's 2019 visualisation, where "the disk looks brighter on the left than it does on the right" (**Cited**, Luminet blog 3/6; NASA SVS 13326). The benchmark stays Gargantua for the **shape** (B1 to B3); Luminet 1979 and NASA 2019 are the reference for the **brightness asymmetry**. Both are Schwarzschild (non-spinning), which matches the catalog's maths row; Gargantua's 0.6 spin barely changes the silhouette at this inclination (**unconfirmed**: step 1 checks against the paper's §4 figures, which the reader could not reach today because the ar5iv text stops in §3.4.1).

**Why not the EHT images.** M87* is "an asymmetric bright emission ring with a diameter of 42±3 micro-as" around a central depression with "a flux ratio ~10:1", mass (6.5±0.7) × 10⁹ solar masses, at 1.3 mm; the asymmetry "can be explained in terms of relativistic beaming" (**Cited**, [EHT paper I, arXiv:1906.11238](https://arxiv.org/abs/1906.11238)). Sgr A* is "a bright, thick ring with a diameter of 51.8 ± 2.3 µas" with "modest azimuthal brightness asymmetry", and models disfavour inclinations above 50° (**Cited**, [arXiv:2311.08680](https://arxiv.org/abs/2311.08680)). Both are thick, blurred, near face-on rings: they prove the physics (ring, shadow, beaming) but do not show the disk-over-the-top look. They are the reference for the **Einstein-ring signature** only (§7).

## 2. Reference photos

No image bytes were saved (the lead's instruction of 2026-11-28: list URLs and the PC fetches them). Drive `references` folder: [chrono_event_horizon/references](https://drive.google.com/drive/folders/1aTiO6BvDOC-C2-48axGSlQ0ZkDhUo4cF), holding `SOURCES.md`.

| # | What it shows | Direct image URL where known | Source page |
|---|---|---|---|
| 1 | EHT M87* 2019: asymmetric ring, bright side at the bottom, dark centre | https://cdn.eso.org/images/large/eso1907a.jpg | https://eso.org/public/images/eso1907a |
| 2 | EHT Sgr A* 2022: thick ring with three bright knots | https://cdn.eso.org/images/large/eso2208-eht-mwa.jpg (pattern of #1; PC to confirm) | https://www.eso.org/public/images/eso2208-eht-mwa/ |
| 3 | Luminet 1979: the first simulated image, bolometric, strong one-sided beaming, disk over and under the shadow (CC BY-SA 4.0) | https://upload.wikimedia.org/wikipedia/commons/2/26/BH-JPL-A%26A1979.jpg | https://commons.wikimedia.org/wiki/File:BH-JPL-A%26A1979.jpg |
| 4 | NASA Goddard 2019, first frame of the 360° movie (thin disk, beaming, photon ring) | https://svs.gsfc.nasa.gov/vis/a010000/a013300/a013326/BH_Accretion_Disk_Sim_360_4k_Prores.00001_print.jpg | https://svs.gsfc.nasa.gov/13326/ |
| 5 | NASA Goddard 2019 annotated still ("highlights and explains various aspects") | page only | https://svs.gsfc.nasa.gov/13326/ |
| 6 | NASA Goddard 2019 edge-on loop and the photon-ring zoom GIFs | page only | https://svs.gsfc.nasa.gov/13326/ |
| 7 | Gargantua's disk as used in the film (paper §4 figures, CC BY 3.0) | page only; figure number unconfirmed | https://iopscience.iop.org/article/10.1088/0264-9381/32/6/065001 |
| 8 | Gargantua seen from slightly above the disk plane (film still; reference only, not for reuse) | page only, second figure | https://blogs.futura-sciences.com/e-luminet/2016/01/10/warped-science-interstellar-36-accretion-disk-tidal-stress/ |
| 9 | Marck 1996 colour simulation with Doppler and gravitational shifts (to show what the dash does **not** do: hue shifts) | page only, fourth figure | same Luminet page |
| 10 | Riazuelo's lensed star field (for the star-field idea in §5) | page only, first figure | same Luminet page |
| 11 | Flamm's paraboloid plot (for the G funnel) | page only | https://en.wikipedia.org/wiki/Schwarzschild_metric |
| 12 | DNEG movies of lensed star fields and critical curves | page only | https://www.dneg.com/news/gravitational-lensing-by-spinning-black-holes |

## 3. How the real thing behaves, with motion notes

The benchmark is a film shot and a family of simulations, not a car display, so most car states have no source; those rows are **Our reading**, grounded in the physics.

| State | What the real thing does | What the dash does |
|---|---|---|
| Start-up | No real start-up. NASA's movie orbits the camera; the disk is always there. | Brief: the shadow is live from frame 1 and the disk assembles in 6 angular sectors (Sci-Fi sector boot, ≤ 1 s, skippable). |
| Idle | The disk turns continuously; "Bright knots constantly form and dissipate", and differential rotation "stretches and shears the bright knots, producing light and dark lanes" (**Cited**, NASA SVS 13326). | Parked means frozen: the shear texture advects with distance travelled (§5), so idle is still. The family rule (idle visually still) wins over the real disk's constant motion. |
| Cruise | Inner annuli orbit faster than outer ones. | Texture advection per annulus at Ω(r) ∝ r^(−3/2) (Kepler, **Computed** below), scaled by speed: the inner edge laps the outer 7.9 times faster. |
| Pull to the limiter | No equivalent. | Mass is rpm: the shadow and lensing scale 0.8 → 1.15 while the lit arc grows clockwise from 7 o'clock. |
| Limiter | No equivalent. | Steady full Einstein ring in shift colour; only the curved 15-pod strip strobes (10 Hz, 50% duty; 2 Hz breath in gentle mode). |
| Gear change | Matter inside the innermost stable orbit cannot orbit: "stable circular orbit with r > 3rs" only (**Cited**, Wikipedia, Schwarzschild metric). | See §7: a knot plunges from the inner edge through the photon ring (recommended), or the brief's one-sector step-back. |
| Braking | None. | Felt G moves the test particle up the funnel (§5). |
| Drift / lateral G | Viewing angle changes which side approaches (NASA SVS: the asymmetry "disappears when we see the disk exactly face on"). | Yaw-rate sign picks the bright side; β tilts the funnel. The disk inclination stays fixed (78°): the board must not tilt the hole with G, or the shape stops reading. |
| Reverse / neutral | None. | Gear glyph R / N in the shadow; disk advection runs backwards in reverse (distance falls or the integrated speed is signed by gear). |
| Shut-down / stale | None. | The host hides the dash (AGENTS.md directive 9). |

**Our reading on timing.** The only real motion is shear: knots smear into arcs. Springs: the mass scale on a critical spring (no overshoot: a shadow that bounces reads as a bug); the bright-side swap as a 400 ms crossfade, never a flip.

## 4. The maths and physics

**Model (catalog row, Sci-Fi table): light bending round a Schwarzschild black hole.** All lengths in units of the Schwarzschild radius r_s = 2GM/c² (**Cited**, [Wikipedia, Schwarzschild metric](https://en.wikipedia.org/wiki/Schwarzschild_metric)). Photon orbit equation (u = 1/r): (du/dφ)² = 1/b² − u²(1 − u). The photon sphere is at r = 1.5 r_s (**Cited**, [Wikipedia, Photon sphere](https://en.wikipedia.org/wiki/Photon_sphere)); stable circular orbits need r > 3 r_s, the disk's inner edge (ISCO) (**Cited**, Schwarzschild metric page; also `docs/specs/optics.md` §12.1).

**Computed, the shadow edge.** A photon grazing the photon sphere has b = r/√(1 − r_s/r) at r = 1.5: b_c = 1.5/√(1/3) = (3√3/2) r_s = √27/2 r_s = **2.598076 r_s**, so the shadow is √3 = 1.732051 times the photon sphere. Matches the catalog.

**Computed, the deflection table α(b).** α(b) = 2∫₀^{u₀} du / √(1/b² − u²(1 − u)) − π, with u₀ the largest root of 1/b² = u²(1 − u), integrated with the substitution u = u₀(1 − t²) to remove the endpoint singularity.

| b / r_s | 100 | 50 | 20 | 10 | 5 | 3 | 2.7 | 2.6 |
|---|---|---|---|---|---|---|---|---|
| α (rad) | **0.020300** (0.02029997) | 0.041223 | 0.108104 | 0.236136 | 0.590396 | 1.719388 | 2.919396 | 6.810372 |
| weak field 2r_s/b | 0.020000 | 0.040000 | 0.100000 | 0.200000 | 0.400000 | 0.666667 | | |

- The catalog's α = 0.020300 rad at b = 100 r_s **checks** (the third-order series 2/b + 15π/(16b²) + 16/(3b³) gives 0.020300 too). **No correction to the maths row.**
- **Strong field (Cited form, Computed constant).** Bozza (2002) proves "the deflection angle always diverges logarithmically" at the minimum impact parameter ([arXiv:gr-qc/0208075](https://arxiv.org/abs/gr-qc/0208075)). Fitting the exact table: α + ln(b/b_c − 1) → −0.39985, −0.40019, −0.40023 at b/b_c − 1 = 10⁻⁴, 10⁻⁵, 10⁻⁶, so α ≈ −ln(b/b_c − 1) − 0.4002 near the edge. Each extra half-turn (Δα = π) therefore shrinks b − b_c by e^(−π) = 0.0432: that is why the photon ring is a 1 px line. Check from the table: α = π at b = 2.678479 and α = 3π at 2.598217, ratio of (b − b_c) 0.00175 against e^(−2π) = 0.00187 (still converging).
- **"Mass is rpm" is a zoom, and the paper says so.** The geometry has no scale but M, so one α(b/r_s) table serves every mass; NASA's visualisation is described as "mass invariant" (**Cited**, SVS 13326).

**Computed, where the disk images land (the numbers the board and the tests need).** Observer at infinity, disk inclination i from face-on. A screen point at polar angle a (0 = right, 90° = up) sees the disk point at angle ψ from the line of sight with tan ψ = −cot i / sin a; the primary image solves sweep(b, r) = ψ, the secondary ψ + π, on either orbit branch.

| Disk radius | Image | i = 78° | i = 80.05° |
|---|---|---|---|
| 3 r_s (ISCO) | right, primary | 3.466 | 3.466 |
| | top (far side, over the shadow), primary | 3.354 | 3.334 |
| | bottom (near side, in front), primary | 0.625 | 0.519 |
| | top, secondary (thin ring) | 2.634 | 2.632 |
| | bottom, secondary | 3.135 | 3.153 |
| 12 r_s | right, primary | 12.490 | 12.490 |
| | top, primary | 6.753 | 6.551 |
| | bottom, primary | 2.496 | 2.073 |
| | top / bottom, secondary | 2.677 / 4.830 | 2.674 / 4.956 |

(all b in r_s). Beloborodov's closed form 1 − cos α = (1 − cos ψ)(1 − r_s/r) ([arXiv:astro-ph/0201117](https://arxiv.org/abs/astro-ph/0201117)) is within 0.06% at the sides and front but **3.8% to 8.0% too high for the far-side arch** (b 3.483 vs 3.354 at ISCO, 7.293 vs 6.753 at 12 r_s). Our reading: the arch needs the exact table, so build a second, 2D table b(r, ψ) at load from the same integral (both branches), not the closed form.

**Computed, pixels.** Reading the brief's "230 px shadow" as the diameter at mass scale m = 1: r_s = 115 / 2.598076 = 44.26 px. With the disk's outer edge pinned at 550 px half-width (the 1100 px ellipse):

| m | r_s (px) | shadow Ø | ISCO half-width | outer edge r | top of far-side arch | thin ring at top |
|---|---|---|---|---|---|---|
| 0.80 | 35.4 | 184 | 123 | 15.04 r_s | 272 px | 93 to 95 px |
| 1.00 | 44.3 | 230 | 153 | 11.94 r_s | 298 px | 117 to 118 px |
| 1.15 | 50.9 | 264 | 176 | 10.32 r_s | 316 px | 134 to 136 px |

At m = 1 the outer-edge primary image runs (550, 0), (303, 175), (153, 265), (0, 298) over the top and (179, −104), (63, −109), (0, −110) in front; the ISCO edge (153, 0), (75, 129), (0, 148) over the top and (0, −28) in front.

- **Correction to the brief (not the maths row): 78° and 1100 × 190 disagree.** cos 78° × 1100 = 229 px, and lensing makes the near edge 220 px tall; a 190 px ellipse is i = arccos(190/1100) = **80.05°**. Pick one: 1100 × 220 at 78°, or keep 190 and say 80°. The catalog brief needs this fixed.
- **The arch is broad, not thin.** At 78° the far side's primary image fills 148 to 298 px above centre, a 150 px band, taller than the disk ellipse. The thin arcs are the secondary images (top 117 px, 2 px outside the 115 px shadow; bottom 139 to 214 px). The brief's "thin second arc" over the shadow is physically the n = 1 ring; the big Gargantua arch is the primary far side. The board should draw both (§9).
- **The near side crosses the gear.** The front band runs from 28 to 110 px below centre, over the lower half of a cap-200 numeral centred in the shadow.
- **"Einstein radius" is the wrong word for the scale.** An Einstein radius goes as √M; the shadow goes as M. If the shadow scales 0.8 to 1.15, an Einstein radius would scale 0.894 to 1.072; if the Einstein radius scaled 0.8 to 1.15, the mass would scale 0.64 to 1.32. Our reading: say "shadow radius (mass scale m) 0.8 to 1.15", since the shadow is what the eye measures.

**Computed, Doppler beaming (intensity only).** Circular Keplerian orbit, Ω = √(0.5/r³) in c/r_s; redshift g = √(1 − 1.5/r) / (1 ∓ Ω b sin i) for the approaching and receding sides (the standard thin-disk form; **unconfirmed** against Luminet 1979's equation, step 1 checks). With `docs/specs/optics.md` §12.1's law (D^(3−α)) already tested in the repo:

| r | local orbital speed | g approaching | g receding | I ratio g³ | g⁴ |
|---|---|---|---|---|---|
| 3 r_s | 0.500 c | 1.3128 | 0.4839 | 19.97 | 54.18 |
| 6 r_s | 0.316 c | 1.2461 | 0.6636 | 6.62 | 12.43 |
| 12 r_s | 0.213 c | 1.1808 | 0.7745 | 3.54 | 5.40 |

The brief's "receding side 60%" is a strong compression of a 5:1 to 54:1 effect. Recommended MODEL: brightness ∝ (g / g_app)^p with p = 0.512, which gives exactly 60% at the ISCO (ln 0.6 / ln(0.4839/1.3128)) and keeps the right shape with radius; labelled `MODEL` on the dash. Hue never changes (red stays reserved; Marck's blue-to-red disk is the thing not to do).

**Computed, the disk's own light (for §5).** Radiative efficiency at the ISCO η = 1 − √(8/9) = 0.0572 (binding energy of the last stable orbit; **unconfirmed** source, step 1 cites a textbook). Newtonian thin-disk flux F ∝ r⁻³(1 − √(3/r)) peaks at r = (3.5/3)² × 3 = **4.083 r_s** (MODEL; the relativistic Page-Thorne profile differs slightly, unconfirmed). Luminet: "The maximum brilliance comes from the inner regions close to the horizon" (**Cited**, blog 3/6).

**Computed, Flamm's paraboloid (the G funnel).** w = 2√(r_s(r − r_s)) (**Cited**, Schwarzschild metric page): w(1.5) = 1.4142, w(2) = 2, w(3) = 2.8284, w(5) = 4 r_s; slope dw/dr = √(r_s/(r − r_s)) = 1 at r = 2, 0.5 at r = 5. The same page warns it "should not be confused with a gravity well": no particle's worldline lies on it. The test particle on the funnel is therefore a **MODEL** read-out, not physics.

**Tests the dash can assert (no GPU).** b_c = 2.598076; α(100) = 0.020300 and α(10) = 0.236136, α(5) = 0.590396, α(3) = 1.719388; α + ln(b/b_c − 1) within 0.001 of −0.4002 at b/b_c − 1 = 10⁻⁵; table lookups at m = 0.8 and 1.15 equal the m = 1 table scaled; at 78° the far-side ISCO image at 3.354 and the 12 r_s image at 6.753 r_s (±0.5%); the 60% receding brightness at the ISCO; flux peak at 4.083 r_s; w(2) = 2, w(5) = 4.

**Checked against the EHT (Computed).** θ_g = GM/(Dc²) for M = 6.5 × 10⁹ M☉ (Cited) and D = 16.8 Mpc (**unconfirmed**, the paper's value) is 3.82 µas; the Schwarzschild shadow 2√27 θ_g = 39.7 µas, against the measured ring of 42 ± 3 µas: the ring sits just outside the shadow edge, as the photon ring should.

## 5. Telemetry mapping

"Long-verified" = verified against the golden packets. "Decoded, check C" = decoded on main 60c99c9, offset confirmed at check C at the PC.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Gear glyph in the shadow | `gear` | long-verified | R / N / 1 to 10; above 10 as digits |
| Speed (Saira, cap 100) | `speed`, unit | long-verified | tabular digits, hysteresis 0.5 unit |
| Lit arc of the disk (rank 3) | `rpm`, `idle_rpm`, `max_rpm`, shift rpm | long-verified | f = (rpm − idle)/(shift − idle), clockwise from 7 o'clock |
| Mass scale m (shadow, lensing, arch) | `rpm` | long-verified | m = 0.8 + 0.35 f, critical spring; r_s px = 44.26 m |
| 15-pod curved strip (rank 1) | `shift_fraction` | long-verified | outside-in fill; per-car, per-gear thresholds |
| Einstein-ring closure | `rpm` ≥ shift rpm | long-verified | lensed arc and photon ring complete, steady, shift colour |
| Bright (approaching) side | `angular_velocity_y` sign | decoded, check C | hysteresis ±0.05 rad/s, 400 ms crossfade; fallback: sign of `g_lat` (long-verified, a_lat = v·r in a steady turn) |
| Disk shear advection | `distance_traveled` | decoded, check C | phase += Δd × k(r), k ∝ Ω(r)/Ω(3); fallback: ∫ speed_ms dt from `timestamp` (long-verified; also covers sessions where the game's distance does not advance, unconfirmed) |
| Disk luminosity (overall gain) | `power` | long-verified | L = clamp(P / P_session_max), floor 0.35 so idle is visible; MODEL (accretion power) |
| Test particle on the funnel | `g_lat`, `g_long` (felt = −a) | long-verified | radius on w(r) from \|G\|, azimuth from the G vector, 500 ms trail |
| Light azimuth | felt G | long-verified | light spring |
| Funnel tilt by β (event) | rear `tire_slip_angle_rl/rr` | decoded, check C | mean of rears, event above 6°; fallback β = atan2(v_x, v_z) (long-verified velocity) |
| Photon ring 1 → 2 px | `boost` | long-verified | 0 to 1 bar maps 1 to 2 px |
| Plunging knot (gear change, §7) | `gear` change, `rpm` drop | long-verified | event only |

**What the game cannot send.** The benchmark is a film shot, so the dropped parts are the parts of Gargantua and the simulations that no field of the 88 drives.

1. **The lensed star field** behind Gargantua (and Riazuelo's lensed sky). There is no sky in the packet. **Replacement (recommended): a sparse star field lensed by the same α table, drifting with distance travelled**, so the hole visibly bends a sky that moves only when the car does (60 to 90 points, alpha ≤ 15%, nothing inside the shadow). Second idea: the stars are the **last lap's speed trace** wrapped round the sky (decoded `best_lap`/`last_lap` timing gates the sweep), rejected for rank creep.
2. **The hole's spin and the D-shaped Kerr shadow** (B5). No field is an angular momentum, and the maths row is Schwarzschild. **Replacement (recommended): the disk's rotation sense from yaw rate** (above), which is the one thing spin changes that a viewer reads. Second idea: throttle as frame-dragging, swirling the inner-edge texture faster (MODEL), not recommended because it fights the frozen-when-parked rule.
3. **The disk's temperature and colour** (Marck's colours, Luminet's "hottest" inner rings). **Replacement (recommended): engine power is the accretion luminosity**, and the radial brightness follows F(r) with its peak at 4.083 r_s, all in palette primary. Second idea: tyre temperatures (decoded, check C) as a hot spot that orbits at the ISCO when a tyre leaves its window, MODEL, event only.
4. **The jet and the EHT's knots** (M87*'s jet; Sgr A*'s three bright knots). **Replacement (recommended): surface rumble and rumble strips (decoded, check C) as short-lived bright knots** born in the disk that then shear into arcs, as NASA describes. Second idea: boost as a faint polar jet, rejected (a large glowing area breaks the flash and lit-area rules).

## 6. Edge cases

- **max_rpm 0 or tiny:** `Signals` clamps max to 1; f = 0, m = 0.8, no strip, no ring; the hole still draws (the shadow is not telemetry).
- **Idle near max:** f's denominator is shift − idle; guard ≥ 200 rpm, else use rpm/max.
- **12,000+ rpm and diesels (4,500):** f is a fraction, so the disk and mass scale span the same range; pods follow per-car shift.
- **EVs:** single gear, idle 0: f = rpm/shift; the plunge event never fires on a gear change, so it fires on nothing (no fake shifts).
- **Reverse:** glyph R, advection runs backward. **Neutral:** N, mass scale still follows rpm (blipping in neutral visibly swells the hole).
- **Gear above 10:** digits fit the 230 px shadow at m = 0.8 (184 px): check two-digit width at cap 200 on the board.
- **mph / km/h, hp / kW:** only speed digits change; power is normalised, so units never matter.
- **Limiter bounce (~50 rpm a frame):** ring closure with hysteresis (close at shift, open at shift − 150 rpm); m on its spring so the shadow does not throb.
- **Crashes and spikes (huge G):** the particle clamps at the funnel rim (2.5 g) and the trail shortens; β event capped.
- **Airborne:** G near zero puts the particle at the rim (weightless reads right); advection continues with speed.
- **Stale or paused stream:** the host hides the dash.
- **Palette extremes:** Doppler lifts primary toward white, so a white or very light primary loses the cue: then the receding side falls to 60% and the approaching side holds at 100% with no lift. A red primary hands structural roles to accent (red stays reserved).
- **Font swap:** gear and speed from the user's font; the shadow pad keeps 7:1 whatever the glyph.
- **Cockpit only:** no hood variant.
- **Reduced-flash mode:** strip breathes at 2 Hz; the Einstein ring is steady anyway; the plunge knot keeps its 150 ms.
- **Colour-only cues and colour vision:** the bright side is a luminance difference, not a hue, so it survives every colour-vision view; the ring closure changes geometry (an arc becomes a full ring), not only colour.
- **0% scrim:** the shadow is transparent on this overlay; only the feathered pad (alpha up to 140 × (1 − scrim)) makes it read, and it must hold the gear at 7:1 (blur gate).

## 7. Signature moments

**Limiter / shift.**
- A (brief, recommended): at shift the lensed arc and photon ring close into a full Einstein ring, steady in shift colour, while only the curved strip strobes at 10 Hz, 50% duty (2 Hz breath in gentle mode). Grounded in the EHT images: a complete ring around a dark centre is what a real black hole looks like (Cited, EHT papers).
- B: the ring is drawn as three nested subrings with spacing exaggerated from e^(−π) (MODEL: at true scale subring 2 sits 0.2 px from the edge), lighting one per 2% over shift. Richer, but it competes with the strip.
- C: the far-side arch thickens toward the shift as m approaches 1.15 (316 px top) and the disk's inner edge brightens to white. No new element, but weaker at a glance.

**Gear change.**
- A (recommended): **the plunge.** Matter cannot orbit inside 3 r_s (Cited). On each upshift one bright knot leaves the disk's inner edge, spirals once through the photon ring and vanishes into the shadow over 150 ms on the fast spring, while m falls with the rpm. It reads as the hole swallowing the old gear.
- B (brief): the shear texture steps back one sector on the fast spring as rpm falls. Subtle; good as a companion to A.
- C: photon-ring ringdown, a damped brightness oscillation of the ring for two cycles, after the way a disturbed black hole rings down (quasi-normal modes; constants **unconfirmed**, step 1 cites a review before using them).

## 8. What it needs from the 4K library

From PR #18 (`origin/claude/project-thread-ib67x9`, `PREBUILD.md` and `kits.rs`, `assets/clusters/chrono_event_horizon/prebuild/manifest.json`): one item.
- Texture `accretion_shear_filaments` (ComfyUI, SDXL, seed 190001, 4096², seamless, greyscale): the disk's material grain, wrapped polar onto the disk and tinted by the dash. The manifest's note: "Only the disk's material grain; the black-hole optics are the dash's maths." No runtime copy: the dash's recipe downsizes it.

Shared parts: `star_field_tile` (Gt3RsApogee kit, seed 200003) is a reflection environment, not usable for idea 5.1 because the lensed stars must be points the α table moves. MSDF atlases for Saira and Saira Condensed exist in `assets/fonts/`. The Phase 6 `assets/scifi/plate_accretion_disk.png` is a 2048 px SDXL picture of a whole black hole with an orange lensing arc: it draws the figure (forbidden: noise never draws the figure), it is not 4K and its hue is fixed, so it must not be used.

The shadow, lensing tables, photon ring, Doppler and funnel are analytic. Building b(r, ψ) at load is a few hundred thousand integrals; if that proves too slow on the PC, a precomputed table is the fallback:

LIBRARY GAP: `schwarzschild_image_table`, a 16-bit linear look-up of the primary and secondary image impact parameter b/r_s over disk radius 3 to 16 r_s and ψ 0 to 2π (about 512 × 512 × 2 channels), layout-independent because the geometry is scale-free; for the far-side arch where the closed form is 8% off; asset-forge `prebuild` computed map (CPU, exact, like the liquid-metal tables), tested against §4's values.

LIBRARY GAP: a flow map for `accretion_shear_filaments` (4096², RG, tangential flow with Keplerian shear Ω ∝ r^(−3/2) baked in polar form) so the grain shears like NASA's lanes instead of rotating rigidly; asset-forge computed map (CPU), not SDXL. Optional if the dash advects per annulus in code.

## 9. Departures and open questions for the board

1. **Inclination vs ellipse (§4):** 78° gives 1100 × 220; 1100 × 190 is 80°. The board picks one; the catalog brief gets fixed.
2. **The arch:** draw the far side's primary image as the broad Gargantua arch (148 to 298 px above centre) plus the thin n = 1 ring, or keep the brief's single thin arc. Recommended: both, the arch at the disk's brightness, the ring at 1 px. Without the arch it reads as Saturn, not Gargantua.
3. **Envelope height:** the arch top reaches 298 px (m = 1) to 316 px (m = 1.15) above centre; the cockpit envelope gives 320 px above the anchor. The strip "along the lensed arc" would sit outside it. Options: centre the hole about 60 px below the anchor, or cap the outer edge so the arch stays under about 260 px, or run the strip along the near-side band below.
4. **The lensing pass:** the brief's 1200 × 520 pass is 260 px tall each side; the arch needs about 640 px. Re-size the pass or clip the arch.
5. **Gear vs the front band:** the near side crosses 28 to 110 px below centre, through the numeral. Options: raise the gear in the shadow, let the band pass behind the dimming pad, or fade the band across the gear's box. Rank 1 legibility decides.
6. **Doppler exponent:** p = 0.512 for the brief's 60%, labelled MODEL; or show the true g³ with a floor. The visual review decides on the benchmark's look.
7. **The particle is a MODEL:** Flamm's surface is a picture of space, not a well (Cited); the funnel's label should say G, not "orbit".
8. **Sibling:** `GeodesicWell` (Apple Vision) also draws a gravity well and an "event-horizon ring". The funnel here must stay small (rank 4) and the hole's arch must lead, so the two cannot be mistaken at thumbnail size.
9. **The idle rule vs the real disk:** a real disk never stops; the dash freezes when parked (brief). Kept, and listed as a departure.

## 10. Sources

Papers: James, von Tunzelmann, Franklin, Thorne 2015, [IOP](https://iopscience.iop.org/article/10.1088/0264-9381/32/6/065001), [supplementary data and licence](https://iopscience.iop.org/article/10.1088/0264-9381/32/6/065001/data), [ar5iv](https://ar5iv.arxiv.org/html/1502.03808); EHT Collaboration 2019, [arXiv:1906.11238](https://arxiv.org/abs/1906.11238); EHT Collaboration 2022, [arXiv:2311.08680](https://arxiv.org/abs/2311.08680); Johnson et al. 2020, [arXiv:1907.04329](https://arxiv.org/abs/1907.04329); Bozza 2002, [arXiv:gr-qc/0208075](https://arxiv.org/abs/gr-qc/0208075); Beloborodov 2002, [arXiv:astro-ph/0201117](https://arxiv.org/abs/astro-ph/0201117) (equation from the search result title; step 1 reads the paper); Luminet 1979, A&A 75, 228, [ADS](https://ui.adsabs.harvard.edu/abs/1979A&A....75..228L/abstract).

Makers and agencies: [DNEG news page](https://www.dneg.com/news/gravitational-lensing-by-spinning-black-holes); [EHT FAQ on Interstellar](https://eventhorizontelescope.org/faq/how-realistic-are-movie-depictions-black-holes-eg-interstellar); [NASA SVS 13326](https://svs.gsfc.nasa.gov/13326/) and [its API record](https://svs.gsfc.nasa.gov/api/13326); [ESO eso1907a](https://eso.org/public/images/eso1907a); [ESO eso2208-eht-mwa](https://www.eso.org/public/images/eso2208-eht-mwa/).

Commentary and reference: [Luminet, The Warped Science of Interstellar 3/6](https://blogs.futura-sciences.com/e-luminet/2016/01/10/warped-science-interstellar-36-accretion-disk-tidal-stress/); [Wikimedia, Luminet 1979 image](https://commons.wikimedia.org/wiki/File:BH-JPL-A%26A1979.jpg); [Wikipedia, Schwarzschild metric](https://en.wikipedia.org/wiki/Schwarzschild_metric); [Wikipedia, Photon sphere](https://en.wikipedia.org/wiki/Photon_sphere).

Repo: `program/phases/07-cluster-factory/CATALOG.md` (brief, Sci-Fi maths row); `docs/design/families/sci-fi.md`; `docs/specs/optics.md` §12.1; `docs/TELEMETRY_USAGE_MATRIX.md` and `crates/fh6-core/src/telemetry.rs` on origin/main; PR #18 kit and manifest.
