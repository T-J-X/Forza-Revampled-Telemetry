# Faraday Mandala: research note

Written 2026-11-28 for build #73 (catalog row 73, Abstract, target `concept`, Fluid / Generative family, cockpit). Step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`. Every statement marked **Cited** was read from the source named, on that date. **Computed** means the thread worked it out from cited constants (the script's numbers are given so they can be checked). **Our reading** is a conclusion drawn for this dash, open to the direction check.

This note holds facts, maths and ideas only. The layout, the hierarchy and the look are designed on the Penpot page `07 · FaradayMandala` (step 2), not here.

## 1. The benchmark

A `concept` dash names one real object for the visual review to measure against. **Ours is the laboratory Faraday-wave photograph**: a shallow dish of liquid shaken vertically, photographed from straight above under a ring of lamps, phase-locked to the subharmonic response, so the standing pattern appears frozen. The canonical examples are the plates in Edwards and Fauve (1994) and Kudrolli and Gollub (1996).

What makes those photographs recognisable (each **Cited**):

| # | Feature | Source |
|---|---|---|
| B1 | Light comes from an axisymmetric **ring of lamps** round the dish with the camera on the axis looking down; Kudrolli and Gollub used 58 lamps with the camera at the centre, Edwards and Fauve a ring of 30 × 15 W lamps on a 14.5 cm radius with the camera 128 to 165 cm above. | Edwards & Fauve, JFM 278 (1994) §3, §3.5, [ifremer.fr](https://data-ww3.ifremer.fr/BIB/Edwards_Fauve_JFM1994.pdf); Kudrolli & Gollub, Physica D 97 (1996) §3, [uvm.edu](https://pdodds.w3.uvm.edu/files/papers/others/1996/kudrolli1996b.pdf) |
| B2 | The image is **reflected light, not height**: a surface element is bright only where its slope sends a lamp's light into the camera. In Kudrolli and Gollub's rig, bright regions are where the surface slope is 4.2°. Everything else is dark. | Kudrolli & Gollub §3 |
| B3 | Exposures are **phase-locked** to the forcing: Kudrolli and Gollub expose for one fluid period (two drive cycles); Douady used bicolour stroboscopy, alternating two colours each half period, because the waves oscillate at half the excitation frequency. | Kudrolli & Gollub §3; Douady, JFM 221 (1990) §3.2 and fig. 14, [utoronto.ca](https://www.physics.utoronto.ca/~phy326/far/Douady.pdf) |
| B4 | The pattern is a **crystal of light**: squares, hexagons, stripes, superlattices and quasi-patterns of bright cells on black, with point defects and grain boundaries where the order breaks. | Kudrolli & Gollub table 1, §4.4, figs 6, 10; Edwards & Fauve abstract |

**Our reading:** the benchmark is a near-black picture, lit only by thin bright contours where the surface tilts toward the lamps. That suits an OLED and the family rule that lit area stays low. It also gives the dash a physical reason for its light: the ring that lights the metal is a real part of the rig, not decoration.

## 2. The physics

### 2.1 Faraday's observation and the f ÷ 2 response

| # | Fact | Source |
|---|---|---|
| F1 | Faraday read "On a peculiar class of acoustical figures; and on certain forms assumed by groups of particles upon vibrating elastic surfaces" to the Royal Society on 12 May 1831 (Phil. Trans. 121). He describes "the well-known and peculiar crispations which form on water at the centres of vibration" (para. 43) and saw them with white of egg (para. 49) but not in oil, which gathered into lenses at the edges (para. 45). | **Cited**, OCR of Phil. Trans. 121, [jimclifford.ca](https://jimclifford.ca/philosophical-transactions-ocr-1665-1869/articles/jstor-107936/) |
| F1b | **On mercury.** A plate of amalgamated tin or copper "is flooded with the fluid metal" and "the crispations are formed with great facility, and of extreme beauty"; "A film quickly covers the metal", and a layer of ink, whose acid "removed all film", or "a little very dilute acetic or nitric acid" keeps it clean (paras 77, 78). Under dilute nitric acid "the rectangular arrangement is constantly obtained", "always at angles of 45° to its edges" (para 85). | **Cited**, same OCR |
| F1c | **Squares, and the half rate.** "the heaps were equidistant, and in rows parallel or at right angles to each other" (para 81). "there were two sets regularly and alternately arranged, the one set rising as the other descended" (para 98). "Each heap ... recurs or is re-formed in two complete vibrations of the sustaining surface" (para 101). | **Cited**, same OCR |
| F2 | Faraday noticed that the waves oscillate at half the excitation frequency; Matthiessen (1868) claimed they oscillate at the excitation frequency; Rayleigh (1883) agreed with Faraday. | **Cited**, Douady 1990 §1, p. 383 |
| F3 | Benjamin and Ursell (1954) showed the linearised equations are equivalent to a system of Mathieu equations, one per surface eigenmode, each parametrically excited. | **Cited**, Kumar & Tuckerman, JFM 279 (1994) §1, [espci.fr](https://blog.espci.fr/laurette/files/2018/01/KT1994JFM279.pdf); Douady §1 |
| F4 | The tongues of harmonic and subharmonic response alternate; the lowest tongue is subharmonic, so at onset the surface answers at half the forcing frequency (for 100 Hz forcing the first three tongues answer at 50, 100 and 150 Hz). Viscosity smooths the bottom of the tongues and, because dissipation grows with k, raises the threshold for larger k. | **Cited**, Kumar & Tuckerman §5, p. 58–60, fig. 1 |
| F5 | For forcing a cos(2ωt) the critical standing waves respond with frequency ω, and ζ(t + π/ω) = −ζ(t): half a response period later the surface is the negative of itself. | **Cited**, Edwards & Fauve §2.4 |
| F6 | π-phase defects in hexagon patterns are "associated with the subharmonic nature of the waves". | **Cited**, Kudrolli & Gollub §4.4, fig. 6 |

### 2.2 Dispersion and the wavelength

| # | Fact | Source |
|---|---|---|
| D1 | ω₀² = k tanh(kh) (g + σk²/ρ) for a layer of depth h; gravity waves where gk dominates, capillary waves where (σ/ρ)k³ dominates. | **Cited**, Douady §2.1, p. 385; Edwards & Fauve eq. 2 |
| D2 | Viscous damping of a surface mode δ = 2νk². | **Cited**, Edwards & Fauve §2.1; Toronto PHY326 manual p. 4, [utoronto.ca](https://www.physics.utoronto.ca/~phy326/far/far.pdf); Chen & Viñals write γ = 2νk₀²/ω₀, [arXiv patt-sol/9702002](https://arxiv.org/pdf/patt-sol/9702002) |
| D3 | Galinstan: density 6.44 g/cm³, viscosity 2.4 mPa·s at 20 °C, surface tension 0.535–0.718 N/m in air (by maker); above 1 ppm oxygen its skin oxidises to Ga₂O₃. | **Cited** by the `KineticLiquidMetal` thread from the Geratherm data sheet §9 ([umn.edu](https://apps.mnc.umn.edu/pub/msds/galinstan_fluid.pdf)) and [Wikipedia, Galinstan](https://en.wikipedia.org/wiki/Galinstan); its RESEARCH.md, table "Galinstan's bulk properties" |
| D4 | A Faraday experiment on a gallium-indium alloy (EGaIn: 6,280 kg/m³, ν 5.13 × 10⁻⁷ m²/s) covered the metal with 0.5 M NaOH "to prevent the oxidation of the liquid metal surface so that it behaves as a simple fluid"; the interfacial tension was then 0.456 N/m and the Faraday threshold 3.2 g at 40 Hz. | **Cited**, Tang, Zhao & Liu, [arXiv 2001.04641](https://arxiv.org/pdf/2001.04641) §II |

**Computed** (g 9.80665, ρ 6,440 kg/m³, σ 0.534 N/m, h 5 mm, ν = 2.4 mPa·s ÷ ρ = 3.73 × 10⁻⁷ m²/s; four cylinders; response at f ÷ 2 with f = rpm ÷ 60 × cylinders ÷ 2; k found by bisection on D1):

| rpm | drive f | response | λ | cells across 60 mm | px per λ at Ø760 | viscous decay 1/δ | onset a_c ≈ 4δω₀ ÷ (k tanh kh) |
|---|---|---|---|---|---|---|---|
| 800 | 26.7 Hz | 13.3 Hz | 17.44 mm | 3.4 | 221 | 10.3 s | 0.010 g |
| 1,500 | 50 Hz | 25 Hz | 10.30 mm | 5.8 | 130 | 3.6 s | 0.029 g |
| 3,000 | 100 Hz | 50 Hz | 6.14 mm | 9.8 | 78 | 1.28 s | 0.098 g |
| 6,000 | 200 Hz | 100 Hz | 3.79 mm | 15.8 | 48 | 0.49 s | 0.32 g |
| 9,000 | 300 Hz | 150 Hz | 2.87 mm | 20.9 | 36 | 0.28 s | 0.63 g |

- **A correction to the catalog's maths row.** Its 800 rpm value, λ = 17.91 mm, is the deep-water value (tanh(kh) set to 1). With the brief's own 5 mm depth, kh = 1.8 there and tanh(kh) = 0.947, so λ = 17.44 mm. The 3,000 and 9,000 rpm values (6.14 and 2.87 mm) are deep enough that the depth does not matter, and they check. The capillary-gravity crossover λ = 2π √(σ ÷ ρg) = 18.27 mm checks. The unit tests will assert 17.44; the handover asks the coordinator to fix the catalog row.
- **Sensitivity to σ.** At the top of galinstan's range (0.718 N/m) the wavelengths are 18.70, 6.76 and 3.17 mm: about 10% longer. The brief's 0.534 N/m is kept; it sits near the oxide-free value for gallium alloys under electrolyte (0.456 N/m for EGaIn, D4).
- **The crossover sits at idle.** At 800 rpm the wave is 17.44 mm against a crossover of 18.27 mm: idle is the mixed gravity-capillary regime, and from about 1,000 rpm upward the waves are capillary. That matters for which pattern the lab would see (§2.3).
- **The onset rises with frequency.** A pure-viscous estimate gives a threshold of 0.01 g at idle and 0.63 g at 9,000 rpm (Mathieu with linear damping, p_c = 4δ/ω₀; Kumar & Tuckerman warn that this simple model underestimates the threshold at small viscosity, F4, so these are lower bounds). In a real dish, friction at the walls and bottom dominates (Douady fig. 19 caption: "dissipation principally depends on the friction at the walls, including the bottom").

### 2.3 Which pattern a real surface chooses

| # | Fact | Source |
|---|---|---|
| P1 | Single-frequency forcing, silicone oils 0.04–1.0 cm²/s, 0.3 cm deep: hexagons at all viscosities below 25 Hz drive; squares between 25 and 50 Hz for ν ≤ 0.5 cm²/s; above 50 Hz squares for ν ≤ 0.2 and stripes for ν ≥ 0.3. | **Cited**, Kudrolli & Gollub table 1, §4.1 |
| P2 | High viscosity gives stripes; low viscosity gives squares in the capillary regime (large frequencies); at lower frequencies, in the mixed gravity-capillary regime, hexagons, eight-fold and other patterns are predicted. In Binks and van de Water's experiment the square-to-hexagon change came near 35 Hz and the hexagon-to-eight-fold change near 29 Hz (predicted 35.4 and 28.7 Hz). | **Cited**, Chen & Viñals abstract, fig. 1 and text |
| P3 | Weakly damped capillary waves favour squares for damping γ < 0.2; near the crossover the full equations gave an eight-fold quasi-pattern. | **Cited**, Zhang & Viñals, [arXiv patt-sol/9603005](https://arxiv.org/pdf/patt-sol/9603005), eq. 7, figs 2, 3 |
| P4 | The twelve-fold quasi-pattern of Edwards and Fauve needs **two-frequency** forcing, f(t) = a[cos χ cos(mωt) + sin χ cos(nωt + φ)] with m, n coprime and one of them odd; the quadratic terms that bring hexagons (three waves) and the twelve-fold pattern (six waves) arise only then. Single-frequency theory gives lines, squares and 2N-fold patterns. | **Cited**, Edwards & Fauve abstract, §2.4, §2.6, eq. 13; its twelve-fold field ζ = A sin(ωt) Σ₁⁶ e^{ik_j·x} + c.c. (eq. 23, read from the paper's summary, to check against the PDF) |
| P5 | Beyond stripes, squares and hexagons, experiments have shown quasipatterns, superlattices, oscillons and double hexagons. | **Cited**, Skeldon & Rucklidge, [arXiv 1504.01553](https://arxiv.org/pdf/1504.01553) §1 |
| P6 | A pattern written as η(r) = Σ A_i cos(k_i · r + φ_i); superlattices of squares (√2 × √2) and hexagons (√3 × √3) were seen above the square onset at 9.5 Hz in a 0.7 mm silicone-oil layer. | **Cited**, Wagner, Müller & Knorr, [arXiv patt-sol/9911001](https://arxiv.org/pdf/patt-sol/9911001), eq. 1, figs 2, 4 |

**Our reading for galinstan.** Faraday's own liquid metal, mercury kept clean under dilute acid, gave squares every time (F1b, F1c). Its kinematic viscosity (0.0037 cm²/s) is ten times lower than the thinnest oil in P1, so it is the weakly damped case of P3. At idle, near the crossover, the lab would expect hexagons or an eight-fold quasi-pattern (P2, P3); through the capillary revs it would expect squares (P1, P3). A twelve-fold mandala needs a second forcing frequency (P4). So the brief's climb (squares → hexagons → twelve-fold) is the dash's own mapping, as the brief already says, and it runs in roughly the opposite order to the lab. §4 sets out the choice for the board.

### 2.4 Amplitude above onset

| # | Fact | Source |
|---|---|---|
| A1 | Reduced forcing ε = (a − a_c) ÷ a_c. With the amplitude equation τ ∂A_i/∂t = εA_i − Σ Γ(θ_ij)|A_j|² A_i, the steady amplitude grows as √ε. | **Cited**, Wernet et al., "Amplitude measurements of Faraday waves", [ar5iv nlin/0008036](https://ar5iv.labs.arxiv.org/html/nlin/0008036) §III.B.2, eq. 3 (√ε is the steady state of that equation) |
| A2 | At cubic order the bifurcation is supercritical; Zhang and Viñals find amplitudes ∝ ε^½; Wagner et al. saw "no noticeable hysteresis for the primary onset". | **Cited**, Edwards & Fauve §2.4; Zhang & Viñals after eq. 7; Wagner et al. |

### 2.5 Order breaking into chaos

| # | Fact | Source |
|---|---|---|
| C1 | Hexagons melt continuously as the forcing rises: near-perfect at ε = 0.4, disordered at 0.6, spatiotemporal chaos at 0.8 (ν 0.2 cm²/s, 22.5 Hz). | **Cited**, Kudrolli & Gollub §5.1, figs 10, 14 |
| C2 | In a small circular cell the surface takes the cell's own modes, S(r, θ, t) = Σ a_lm(t) J_l(k_lm r) cos(lθ), with J_l′(k R) = 0 at the wall; l counts the angular maxima and m the nodal circles. Ciliberto and Gollub saw at least 30 modes (l, m from 1 to 7), such as (7, 2) and (4, 3), in a 6.35 cm radius, 1 cm deep cell driven near 16 Hz; where two modes' stability curves cross, the modes compete and the surface goes chaotic with a slow oscillation over 100 times slower than the drive. | **Cited**, Ciliberto & Gollub, PRL 52, 922 (1984), eq. 1, figs 1–3, [core.ac.uk](https://core.ac.uk/download/229116239.pdf); model in Il Nuovo Cimento D 6, 309 (1985), [core.ac.uk](https://core.ac.uk/download/229115768.pdf) |
| C3 | Large aspect ratio is needed to study patterns without the side walls; moderate-viscosity patterns in large containers are not strongly bound by the walls. | **Cited**, Chen & Viñals introduction; Skeldon & Rucklidge §2 |
| C4 | At the container wall a meniscus oscillates at the driving frequency and sends out waves of its own; in the EGaIn experiment the Faraday threshold was taken as the point where the meniscus-wave pattern breaks. | **Cited**, Tang, Zhao & Liu §II, §IV.1 |

**Computed:** with a 30 mm radius, kR is 10.8 at 800 rpm (3.4 cells across) and 65.6 at 9,000 rpm. At idle the dish is a small-aspect-ratio cell where the circular modes of C2 lead, and at high revs it is a large-aspect-ratio layer where the lattice of §2.3 leads. **Computed (the dish's own modes near idle, four cylinders, R 30 mm, from J_l′(kR) = 0):** the nearest modes are (l, m) = (8, 1), (4, 2), (2, 3) at 700 rpm (kR 9.58); (9, 1), (5, 2), (3, 3) at 800 rpm (kR 10.81); (10, 1), (6, 2), (1, 4) at 900 rpm; (2, 4), (0, 4), (7, 2) at 1,000 rpm; (9, 2), (6, 3), (13, 1) at 1,200 rpm. The modes sit about 0.2–0.6 apart in kR, so as the revs rise the dish passes from one mode to the next every 50 to 100 rpm, near-degenerate pairs and all, which is where Ciliberto and Gollub saw modes compete (C2). The pinned post makes the true modes annular (J_l and Y_l terms); the plain-disc forms are a close guide away from the post. Figure: `/mnt/project-files/faraday_mandala/research_idle_modes_and_amplitude.png` (a maths study, not a design).

**Our reading:** a true mandala (a Bessel mode J_l(kr) cos lθ: l petals and m rings) is what the real dish would show near idle.

## 3. The maths the code will assert (no GPU)

From the catalog's "Real geometry and maths" row, checked here:
- f = rpm ÷ 60 × cylinders ÷ 2; response f ÷ 2; k from D1 with ρ 6,440, σ 0.534, h 5 mm, g 9.80665.
- λ(800 rpm) = 17.44 mm (not 17.91, §2.2), λ(3,000) = 6.14 mm, λ(9,000) = 2.87 mm; crossover 18.27 mm.
- ζ = A √ε Σ cos(k e_i · x + φ_i) with 2, 3 or 6 unit vectors spaced 90°, 120° (equivalently 60°) or 30°: **Computed** with all φ_i = 0, ζ is unchanged by a 90° turn for squares, 60° for hexagons and 30° for the twelve-fold pattern (largest error over 200 random points 2 × 10⁻¹⁴); an eight-fold option (four vectors at 45°) is unchanged by 45°.
- Hexagons with φ_i = 0 are asymmetric: ζ runs from +3 to −1.5. Half a response period later (F5) the surface is −ζ, from +1.5 to −3: the "up" hexagons become the "down" honeycomb. A strobe at the crest shows one; a strobe half a period later shows the other.
- **The 36 px cap and the cylinder count.** The brief caps the pattern at 36 px per wavelength (2.874 mm at 12.67 px/mm). **Computed:** that wavelength is reached at 9,000 rpm with four cylinders, 12,000 with three, 7,200 with five, 6,000 with six, 4,500 with eight, 3,600 with ten and 3,000 with twelve. A V12 would freeze the pattern from 3,000 rpm to the limiter. §4, idea M3, gives a fix that keeps λ exact.

## 4. Choices and ideas for the Penpot board

The board picks among these; none is a layout.

**Pattern mapping (the brief marks it as the dash's own):**
- **M1, the brief's climb:** squares → hexagons → twelve-fold over the three thirds of the revs, with 300 rpm crossfades. Visually it gets busier with the revs. Departure from the lab (§2.3): the order is reversed, and the twelve-fold pattern needs two-frequency forcing.
- **M2, the lab's order:** the dish's own circular mode (a mandala of petals and rings, C2) at idle, hexagons or eight-fold near the crossover, squares through the capillary revs (P1–P3), and chaos past the limiter (C1). Honest, but the top of the revs is the plainest pattern.
- **M3, the camera zoom (applies to M1 or M2):** the field of view of the "photograph" is set once per car from `EngineMaxRpm` and the cylinder count, so the 36 px wavelength lands at the car's own max rpm. λ stays exact in millimetres; only the magnification changes, as a lab camera's would. This removes the frozen pattern above 3,000 rpm on a V12.
- **Recommended for the board:** M1 with M3 as the signature. **Not** M2's circular mode at idle: the sibling check below rules it out. Idle with the throttle shut is a still mirror, as the brief says; with throttle at low revs the lattice stands at its longest wavelength. Every departure goes in the handover.

**Sibling check (2026-11-28, CATALOG.md briefs).** `CymaticResonanceField` (build #14, Psychedelic) draws circular-plate modes J_m(k r) cos mθ with sand on the nodal lines, stepped through nine (m, n) figures with the revs. A Bessel-mode mandala at idle would make this dash read as that one, which the brief's Risk line forbids ("must not read as a plate figure"). So this dash keeps to lattices and quasi-patterns of plane waves, on a reflective metal with no nodal lines drawn. `ChronoMultiverse` (build #23) owns the ferrofluid spikes along a crown arc and specular-only black fluid; this dash's metal is a bright mirror lit by its ring, with no spikes. `KineticLiquidMetal` (build #7) owns the single oil-slick blob and its thin-film colour; this dash shows no film colour.

**Light (from the benchmark):**
- **L1, the rev ring is the lamp ring.** In the photographs the only light is a ring of lamps round the dish (B1, B2). If the lit arc of the rpm ring is that ring, the metal reflects only the lit part of it, so the pattern's highlights fill in round the dish as the revs climb. The reflection follows from the surface normal and the kit's studio environment, no painted glow.
- **L1, computed (what a ring light lights).** A mirror facet sends a ring seen at angle α off the vertical into an overhead eye only when its tilt is α/2 (within the ring's own angular width). So: (a) with the pattern standing but its steepest tilt below α/2, the metal stays black; (b) as the amplitude reaches α/2, the steepest flanks light first, as small rings round each cell; (c) above it each flank splits into two thin contours that move apart. For hexagons with the ring at 16° and a ±1.2° band, the lit share of the dish is 0 at 0.3 and 0.6 of the catching tilt, 13.6% at 0.9, 29% at 1.2 and 10.8% at 2.0 (`/mnt/project-files/faraday_mandala/research_ring_light_contours.png`, a study of the optics, not a design). Kudrolli and Gollub's rig lit facets at 4.2° (B2). **Our reading:** the throttle then reads as light growing out of black, and the lit share stays far under the family's limit; the ring angle is chosen so the catching tilt falls near mid-throttle, and the peak lit share (about 30%) is one more input for the OLED luminance budget at the board.
- **L2, the G-light** of the brief (lateral G turns the key light's azimuth) layered on L1.

**Signature moments:**
- **Limiter:** the phases drift apart into chaos (C1), smooth and brightness-capped; only the lamp row strobes (house rule).
- **Gear change (the brief does not define one):**
  - **G1, the half-period flip:** the strobe jumps half a response period, so the surface shows −ζ (F5): the hexagons turn into their honeycomb, the squares shift by half a cell. One clean inversion per shift, eased, not a flash.
  - **G2, regrowth from the rim:** the throttle lifts, the old pattern decays (D2), and the new wavelength grows back in from the wall, where the meniscus waves start (C4), as a ring front toward the post.
  - Recommended: G2 with G1 as the moment the clutch bites; the board shows both.
- **Braking (throttle shut, so the brief has a flat mirror):** **K1, strobe detuning.** A strobe locked to the old frequency while the revs fall sees the pattern beat slowly between ζ and −ζ (the lab's way of watching a pattern move). The beat rate is the change in response frequency, capped at 1 Hz and smooth, so it never flashes. Or **K2:** braking raises the damping, and the last of the pattern dies from the rim inward.

**Telemetry the benchmark has that the game does not send, and what replaces it:**
- The rig's function generator shows the drive frequency and the accelerometer shows Γ = a ÷ g. Both are honest here: f follows from rpm and the cylinder setting, and ε from the throttle. **Idea R1:** a micro-label of the live response frequency in hertz (and the cylinder setting), the dash's own readout. The game does not send a cylinder count (the brief's OSD setting `FaradayMandalaCylinders` covers it).
- Nothing in the benchmark needs fake data, so nothing is dropped empty.

**Material (galinstan under electrolyte):** D3 and D4 mean an honest dish keeps the metal clean under an electrolyte or an inert gas, or the oxide skin stops the waves. **Idea S1:** a thin, clear cover layer over the metal (the house's optical glass, `docs/specs/optics.md`) that also gives a crisp rim. Optional; the board decides.

## 5. Library parts (pre-build kit, PR #18)

From `assets/clusters/faraday_mandala/prebuild/manifest.json` (branch `claude/project-thread-ib67x9`), all computed on the CPU, masters on the Drive in `faraday_mandala/prebuild/`:
- `liquid_metal_studio_env` (octahedral, 2048 runtime, 4096 master): what the metal mirrors.
- `liquid_metal_env_band` (256 × 1 HDR): the cheap band lookup for 2D metal shading.
- `mercury_matcap` (1024 runtime, 4096 master): one lookup from the wave normal. Mercury stands in for galinstan (no measured galinstan constants in refractiveindex.info).
- `mercury_fresnel` (256 × 1): reflectance over cos θ.
- `chrome_matcap`: the rim and the post.

**What the kit leaves to this dash:** the wave field and its normals (live maths, cheaper than any flipbook), and the rim hardware (its size comes from the layout). The Blender rim, post and the ComfyUI brushed finish are made after the board and after LIBRARY DONE, through `tools/asset-forge/src/recipes/faraday_mandala.rs` on the cloud lane.

**Render path (settled 2026-11-28).** `Cluster::update` gets no device or queue, so a dash cannot run a compute pass of its own per frame. On this dash's request the kit gained a live standing-wave surface and a conductor mode (main 266f471, PR #26; `docs/specs/optics.md` §2–3, `docs/kit/README.md` §4): `OpticalGlass::metal(Conductor::mercury())` with `glass.standing = StandingWaves::symmetric(center, k, count, amplitude, turn).still(r, w)`, at most `MAX_STANDING_WAVES` = 6 waves, the wavenumber and `waves_mut()` set each frame, and `StandingWaves::sample(p)` as the CPU mirror for the unit tests.

**The crossfades fit in six waves.** The twelve-fold set is six directions 30° apart (0°, 30°, …, 150°). The squares' two directions (0°, 90°) and the hexagons' three (0°, 60°, 120°) are subsets of it. So every band, and every crossfade between bands, is the same six waves with different amplitudes: squares → hexagons uses four (0°, 60°, 90°, 120°), hexagons → twelve-fold fades in the three odd ones. No second draw is needed. An eight-fold option (45° steps) is not a subset and would need its own set; that is one more reason to keep M1's twelve-fold.

A pre-shaded texture scaled by k stays the fallback only (exact for the periodic squares and hexagons, but not for the amplitude, the phase drift or the crossfades).

## 6. Departures (kept for the handover)

1. The catalog's 800 rpm wavelength corrected to 17.44 mm (§2.2).
2. The pattern bands are the dash's own mapping and do not follow the lab (§2.3, §4).
3. The twelve-fold pattern is shown under single-frequency forcing, which the lab does not produce (P4).
4. The onset is not computed per frequency: ε comes from the throttle, as the brief says, though the real threshold rises about 60-fold from idle to 9,000 rpm (§2.2).
5. The viscous decay (10 s at idle) is far slower than a dash may be; the decay used on screen is a design time, named in the handover.
6. Mercury's optics stand in for galinstan's (§5).

## 7. The cylinder count is in the packet

**Cited:** the official Forza Horizon 6 Data Out documentation lists `NumCylinders` (S32, "Number of cylinders in the engine") right after `DrivetrainType`, which puts it at bytes 228–231 by the field list (the page gives no offsets; its fields add to 323 bytes against a stated 324). It also lists `CarClass`, `CarPerformanceIndex`, `CarGroup` and `Torque`, and says nothing of what `NumCylinders` holds for electric or rotary cars. [support.forza.net, Forza Horizon 6 Data Out Documentation](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation)

**Checked here:** the shared decoder (`crates/fh6-core/src/telemetry.rs`) reads 212 (`car_ordinal`) and 224 (`drivetrain_type`) but not 228. All 20 golden packets in `tests/golden/packets` hold 0 at 216–231 (class, PI and cylinders): they are synthetic, so they cannot verify the field.

**So:** the brief's premise ("the packet has no cylinder count") is wrong for FH6. **Decoded since main 60c99c9** (2026-11-28, on this thread's request): `FH6TelemetryData::num_cylinders` (le_i32 at 228), matrix key `cylinders`. The dash takes it when it is in 1..=16; the OSD setting `FaradayMandalaCylinders` stays as the fallback (0 or out of range, an EV, a rotary) and as the user's override. The real-game check C still has to confirm the offset and what EVs and rotaries send; until then the golden packets (all 0 there) exercise only the fallback. `cylinders` joins this dash's row in `docs/TELEMETRY_USAGE_MATRIX.md` with the code, when `dash_gates` can observe it.

## 8. Telemetry mapping (verified fields only, `fh6_kit::Signals`)

| Input | Drives | Range and guard |
|---|---|---|
| `rpm`, `idle_rpm`, `max_rpm`, `shift_rpm` | wavenumber (§2.2), the pattern band, the rpm ring, the lamps, the limiter | `max_rpm` can be 0 and `idle_rpm` can exceed it (decoder note): clamp rpm to [0, max(max_rpm, idle_rpm + 1)] before any division; below 1 rpm the surface is flat |
| cylinders (`num_cylinders` when 1..=16, else the OSD setting, default 4) | f = rpm ÷ 60 × cylinders ÷ 2, and the camera zoom of idea M3 | integer 1..=16; anything else falls back to 4 |
| `throttle` | ε = throttle − 0.05, amplitude ∝ √ε (A1, A2) | ε ≤ 0 gives a flat mirror; amplitude capped so the steepest slope stays below the still-zone's |
| `rpm_rate` | idea K1 (strobe detuning while the revs fall) | beat capped at 1 Hz, eased |
| `gear` | the numeral; a change starts the gear moment (G1/G2) | neutral and reverse are a still dish with the numeral only |
| `speed` + `unit` | the tabular speed | `fh6_kit::Hysteresis` on the raw value |
| `g_lat` | the key light's azimuth | spring from the tokens, no extra smoothing of rpm, speed or gear (closing review line 7) |
| `g_long` | the dish's mean level tilt on `fluidSlosh` | clamped tilt |

Not used: slip, suspension, tyre temperatures, fuel, laps and `Torque` (not verified against golden packets; `Torque` is not decoded).

## 9. Real behaviour and motion notes

- **Strobed, never animated at the wave's own rate.** 13 to 150 Hz responses (4 cylinders, 800 to 9,000 rpm) are 0.2 to 2.5 times the display rate, so drawing the real oscillation would alias into flicker. The surface is drawn at its crest (B3), so on screen only the wavelength, the amplitude and the phases move.
- **Growth and decay.** From A1, τ dA/dt = εA − …: the pattern grows at a rate ε ÷ τ and decays at about 1 ÷ τ once ε < 0. Real τ (viscous, §2.2) runs 10 s at idle to 0.28 s at 9,000 rpm, too slow for a dash at low revs. On screen: growth over about 120 ms and decay over about 250 ms, both faster at high revs, in the same direction as the physics. Named as a departure (§6, item 5).
- **Wavelength changes are instant.** k follows rpm each frame with no smoothing of its own (closing review line 7); the cells shrink and grow smoothly because rpm itself is continuous, and a gear change is the one jump (handled by G1/G2).
- **Chaos without flash.** C1's melt goes ordered → defects → chaos as ε rises; on the limiter the dash drifts the six phases apart at a capped rate (e.g. ≤ 0.6 rad/s per wave, a design value) with brightness capped, so the WCAG 2.3.1 flash audit stays clean. Only the lamp row strobes.
- **Inversion (G1) is a crossfade of sign,** A → −A over about 80 ms, not a cut. **Computed (motion study):** a pure ring light shows only the slope's size, and |∇(−ζ)| = |∇ζ|, so under the benchmark's lighting alone the flip is invisible; and half-way through, A = 0, the dish is a flat mirror for a frame or two. So what the driver sees is the pattern sinking to a mirror and standing again with the key light on the other flanks: the flip only reads through the directional key light and the studio reflection, which the board must show. Figure: `/mnt/project-files/faraday_mandala/research_motion_chaos_and_flip.png`.
- **Chaos, computed:** drifting the six phases of the twelve-fold set apart at up to 0.6 rad/s melts the order visibly within about 1 s and fully by about 2 s, while the mean slope (what the ring light turns into brightness) stays within 1% and its 99th percentile within 3% of the ordered state, so the melt adds no flash (same figure).
- **The still zone** round the post (the brief's 1.6 × the numeral's cap) uses the kit's `still(r, w)` ease, so the gear always sits on a flat mirror (blur gate).

## 10. Edge cases

| Case | What happens | Why |
|---|---|---|
| `max_rpm` = 0 or `idle_rpm` ≥ `max_rpm` | ring and lamps hidden, surface flat, gear and speed still shown | the decoder passes these as sent |
| rpm above `max_rpm` | treated as the limiter; k clamped at the max-rpm wavenumber | k must not exceed the 36 px cap |
| Electric car (`num_cylinders` 0, to confirm in check C) | idea E1: the forcing becomes the motor's own electrical frequency, f = rpm ÷ 60 × pole pairs (an OSD setting, default 4: the 48-slot, 8-pole machine a Chalmers traction-motor thesis calls "a well-established traction motor baseline", read from a summary), so the same physics runs on the motor | an EV has no firing pulses. **Cited:** a PMSM's cogging torque repeats with N_r, "the least common multiple of the slot number NS and 2NP" (Li et al., IEEE TIE 2018, §II-B, eq. 9c, [nsf.gov](https://par.nsf.gov/servlets/purl/10149926)); p and f_e as defined in the Chalmers thesis nomenclature ([chalmers.se](https://odr.chalmers.se/bitstreams/0a345826-631e-439f-824a-f1227a382506/download)). Which harmonic of f_e dominates the ripple was not confirmed by these sources, so E1 uses f_e itself and names the choice |
| Rotary engine | f = rpm ÷ 60 × rotors (a two-rotor fires like a four-cylinder four-stroke); the cylinder field's value for a rotary is unknown (§7), so the OSD setting covers it | **Cited:** a Wankel rotor's three faces each run their own four-stroke cycle and "one power pulse is produced at each revolution of the shaft" ([Wikipedia, Wankel engine](https://en.wikipedia.org/wiki/Wankel_engine), working principle) |
| Very many cylinders (V12, W16) | idea M3's zoom keeps the climb; without it the pattern freezes from 3,000 rpm (V12) | §3 |
| Throttle feathering at 1–5% | ε ≤ 0: a flat mirror, no flicker at the threshold (hysteresis of 2% on ε) | A2 says the onset itself is not hysteretic; the 2% is a design guard, named |
| Neutral, reverse, paused, stale stream | flat mirror; the app hides the dash as for every dash | closing review line 8 |
| Reduced flash | the limiter's phase drift halves and the lamp strobe follows the kit's reduced-flash rule | house rule |
| No assets (`setup.assets` is `None`) | the kit's studio environment (`Environment::Studio`) instead of the kit map; the metal still reads | closing review line 16 |

## 11. Not yet checked

- Faraday 1831, paras 106–110, were not readable verbatim; nothing from them is cited.
- Edwards and Fauve eq. 23 and their §5.1 (order above p = 0.01) were read from a summary of the PDF, not verbatim.
