# ChronoMultiverse: research note

Build #23, catalog row 29, category Psychedelic, Target `concept`, family Fluid / Generative, views cockpit and hood. Since 08:52 on 2026-11-28 this dash holds the ferrofluid crown: Rosensweig spikes along a crown arc, the gear as a pole piece. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read on that date in the source linked; **Computed** means worked out here with `python3 -I -c` and numpy from cited constants, with inputs and results given so the thread can re-run them; **Our reading** is a conclusion drawn for this dash, open to the direction check.

## 1. The benchmark

**One benchmark: Sachiko Kodama and Minako Takeno, "Protrude, Flow" (created 2000, shown at SIGGRAPH 2001).** What makes it recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | Black magnetic fluid in a dish under electromagnets; the spikes are "pointed mountains" that rise and fall, "occasionally ... pliable organic shapes, sometimes ... flowing particle streams". | Cited, SIGGRAPH archive artist statement, [history.siggraph.org](https://history.siggraph.org/artwork/sachiko-kodama-minako-takeno-protrude-flow/) |
| B2 | **Sound drives the field.** A microphone above the room picks up sound and voices; a computer turns it into the electromagnet voltage, so "magnetic fluid pulsates according to the sound". | Cited, same page |
| B3 | Six electromagnets; a camera films the fluid and projects it on a wide screen (the piece is 4 m × 4 m × 7 m). | Cited, same page; Kodama, "Dynamic Ferrofluid Sculpture", CACM 51(6), 2008, [cacm.acm.org](https://cacm.acm.org/research/dynamic-ferrofluid-sculpture/) |
| B4 | The later "ferrofluid sculpture" technique (Morpho Tower, 2006): one electromagnet with a sculpted iron core; the fluid climbs the iron and its spikes ride the edge of the spiral, growing and shrinking with the coil current, the surface passing from "a soft fluid, a minute moss, spiky shark's teeth" to "a hard iron surface". | Cited, CACM 2008 |
| B5 | Ferrofluids appear black; their spikes form "along magnetic field lines when the magnetic surface force exceeds the stabilizing effects of the fluid weight and surface tension". | Cited, CACM 2008 |

**Our reading.** "Protrude, Flow" is the closest real object to this dash: a live input (sound there, the engine here) sets a coil current, and the black fluid answers with spikes. The brief's gear "pole piece whose outline bristles with micro-spikes" is Kodama's Morpho Tower technique (B4): spikes gather where an iron edge concentrates the field. Ring-shaped spike rows trapped along the sharp edge of a cylindrical electromagnet are also a lab result (Rupp et al. 2003, as cited by Vieu and Walter, [arXiv 1711.09942](https://ar5iv.arxiv.org/html/1711.09942) §1). The physics benchmark for the figure itself is the laboratory Rosensweig pattern (§4, photos 11 to 13). No better real target was found.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Ferrofluid in a dish over a neodymium magnet: a dense field of spikes, side-lit (CC BY 2.0) | https://upload.wikimedia.org/wikipedia/commons/d/d9/Ferrofluid_in_magnetic_field.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_in_magnetic_field.jpg) |
| 2 | Large spikes, close (GFDL 1.2) | https://upload.wikimedia.org/wikipedia/commons/c/c4/Ferrofluid_large_spikes.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_large_spikes.jpg) |
| 3 | Spikes on a reflective glass plate in a strong, steep field (GFDL 1.2) | https://upload.wikimedia.org/wikipedia/commons/c/c9/Ferrofluid_large_spikes2.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_large_spikes2.jpg) |
| 4 | Ferrofluid on glass over a magnet (GFDL 1.2) | https://upload.wikimedia.org/wikipedia/commons/7/7c/Ferrofluid_Magnet_under_glass.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_Magnet_under_glass.jpg) |
| 5 | Fluid collected at the poles of a strong magnet, under an oily surface (GFDL 1.2) | https://upload.wikimedia.org/wikipedia/commons/b/b7/Ferrofluid_poles.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_poles.jpg) |
| 6 | Macro of ferrofluid under a magnet: specular-only highlights on black (CC BY 3.0) | https://upload.wikimedia.org/wikipedia/commons/d/d8/Ferrofluid_close.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_close.jpg) |
| 7 | Spike field, 3456 px (CC BY 2.0) | https://upload.wikimedia.org/wikipedia/commons/6/66/Ferrofluid_spikes.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_spikes.jpg) |
| 8 | Spikes with the magnet underneath (CC BY-SA 2.0) | https://upload.wikimedia.org/wikipedia/commons/a/aa/Ferrofluid_spikes_%28magnet_placed_underneath_the_fluid%29.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_spikes_(magnet_placed_underneath_the_fluid).jpg) |
| 9 | A series titled 1, 7 and 17 spikes (public domain): the reference for the ferro-drops' spike count (contents not yet viewed; step 1 confirms) | https://upload.wikimedia.org/wikipedia/commons/0/0a/Ferrofluid1spike.JPG , https://upload.wikimedia.org/wikipedia/commons/9/9b/Ferrofluid7spike.JPG , https://upload.wikimedia.org/wikipedia/commons/f/fb/Ferrofluid17spike.JPG | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid7spike.JPG) |
| 10 | Simulated ferrofluid shapes for several surface tensions and field strengths (CC BY-SA 4.0) | https://upload.wikimedia.org/wikipedia/commons/9/98/Ferrofluid_simulations_for_different_parameters_of_surface_tension_and_magnetic_field_strengths.png | [Commons](https://commons.wikimedia.org/wiki/File:Ferrofluid_simulations_for_different_parameters_of_surface_tension_and_magnetic_field_strengths.png) |
| 11 | Laboratory Rosensweig pattern photo (Fig. 1a) and X-ray maps of hexagons, mixed states and squares (Fig. 5) | page figures | [Gollwitzer, Rehberg, Richter, arXiv nlin/0607059](https://ar5iv.labs.arxiv.org/html/nlin/0607059) |
| 12 | Hexagonal peaks of EMG 909 in a 120 mm vessel (Fig. 1) | page figure | [Lange et al., arXiv 1603.01073](https://ar5iv.arxiv.org/html/1603.01073) |
| 13 | 3D rendering of the measured surface at B = 20.1068 mT, contours 1 mm apart (Fig. 3) | page figure | [Gollwitzer et al., arXiv nlin/0604009](https://ar5iv.labs.arxiv.org/html/nlin/0604009) |
| 14 | "Protrude, Flow" (2001), installation photograph (Fig. 1); copyrighted, reference only | page figure | [CACM 2008](https://cacm.acm.org/research/dynamic-ferrofluid-sculpture/) |
| 15 | Morpho Tower (2006), spikes riding a spiral iron edge (Fig. 2); copyrighted, reference only | page figure | [CACM 2008](https://cacm.acm.org/research/dynamic-ferrofluid-sculpture/) |

Drive `references` folder (with `SOURCES.md`): https://drive.google.com/drive/folders/1at7Atuv89bfMM7sWIgKPOkOLuuaAaPW8 (`G:\My Drive\FH6 WheelDash v2 - 4K Masters\chrono_multiverse\references\`). No image bytes were saved here; the PC fetches the listed URLs.

## 3. How the real thing behaves, with motion notes

| # | Behaviour | Label and source |
|---|---|---|
| R1 | Below the critical induction the surface stays flat; at B_c "a hexagonal array of peaks appears" (APG 512A: B_c = 16.7 mT measured). | Cited, nlin/0607059 §3.2 |
| R2 | The hexagon branch is transcritical: the amplitude jumps from zero to a finite height at onset and, on the way down, the spikes survive below B_c to a fold, so the onset is hysteretic. Amplitude A_H = [γ(1 + ε) + √(γ²(1 + ε)² + 4εg_h)] ÷ (2g_h), ε = (B² − B_c²) ÷ B_c²; fitted γ = 0.281, g_h = 0.062 for EMG 901 (χ 2.2). The hysteresis widens with susceptibility. | Cited, nlin/0607059 eq. 1, Fig. 3a |
| R3 | For APG 512A (χ 1.17) the hysteresis is small, about 0.2 mT. | Cited by the `KineticLiquidMetal` note from Gollwitzer et al., NJP 11, 053016 (2009), [uni-bayreuth.de](https://epub.uni-bayreuth.de/3966/1/Gollwitzer_2009_New_J._Phys._11_053016.pdf); step 1 re-reads it |
| R4 | Hexagons give way to squares at higher field: local squares from 20.7 mT, the whole dish square by 27.4 mT; on the way down hexagons return at 29.5 mT (first) and 23.7 mT (all), higher than they left, an inverse hysteresis ("proteresis"). | Cited, nlin/0607059 §3.2, Fig. 7 |
| R5 | The measured wavenumber falls a little as the field rises (k_c = 0.63 mm⁻¹ for comparison), and in the square state it is lower than in the hexagonal state. | Cited, nlin/0607059 Fig. 7 and text |
| R6 | A local field pulse (+0.68 mT on 8.91 mT, EMG 901) makes one stationary spike with a circular dip round it, a "ferrosoliton", which stays for days after the pulse is gone; its width equals the lattice period. | Cited, nlin/0607059 §2, Fig. 3 |
| R7 | The crests are static: "liquid crests which persist without motion". | Cited, nlin/0607059 §1 |
| R8 | Growth is fast in a thin fluid and slow in a thick one: in APG E32 cooled to 10 °C (η 4.48 Pa·s) the pattern takes about 60 s to grow; with η ≈ 4 mPa·s it was measured with a 7 kHz Hall-sensor array. | Cited, Lange et al. 1603.01073 §I |
| R9 | Under shaking the spike crystal melts once a spike's rms displacement passes 0.14 of the lattice wavelength, for hexagons and squares alike. | Cited by the `KineticLiquidMetal` note from Boyer and Falcon, [arXiv 0909.3504](https://arxiv.org/pdf/0909.3504) |
| R10 | In "Protrude, Flow" the field follows the room's sound, so the spikes pulse with it (B2). | Cited, SIGGRAPH archive |

**Computed timings (inviscid, deep layer, APG 512A: ρ 1236 kg/m³, σ 30.57 mN/m).** Linear growth at k_c: s = √(2σk_c³ε ÷ ρ) = 111 √ε s⁻¹, so the e-folding time is 90 ms at ε = 0.01, 28 ms at 0.1 and 13 ms at 0.5 (APG 512A's 120 mPa·s slows this; a lower bound). The free ripple at k_c has ω₀ = √(2gk_c) = 111.1 rad/s (17.7 Hz), and k_c is exactly the wavenumber of the slowest capillary-gravity wave, c_min = (4gσ ÷ ρ)^¼ = 0.176 m/s.

**State by state (Our reading, from R1 to R10):**
- **Start-up / greeting:** the field ramps up: flat black pool, then at ε = 0 the whole hexagon field jumps up at once (R2), not a slow rise.
- **Idle:** the field sits just under or at threshold, so the pool is flat or holds low still spikes (R7). Nothing moves while the inputs are constant.
- **Cruise:** spikes stand at the height the field gives, motionless; only the front moves with rpm.
- **Pull to the limiter:** the field climbs; height grows as A_H(ε) while the spacing stays put (R2, R5), so the spikes get steeper.
- **Limiter:** past ε ≈ 0.5 to 1.7 the hexagons turn to squares (R4); see §7.
- **Gear change:** a field dip on the upper branch slumps the spikes and they recover on the same branch if the dip stays above the fold (R2); see §7.
- **Braking / lift-off:** the field falls; spikes shrink and, below the fold, collapse to a flat mirror. They do not return until ε rises past 0 again (R2).
- **Drift / lateral G:** the pool's mean level tilts hydrostatically; spikes stay along the field (B5). Hard shaking past 0.14 λ melts the lattice (R9).
- **Reverse / neutral:** no field change from the engine; the pool holds its last stable state or relaxes flat.
- **Shut-down:** the field ramps down; spikes persist to the fold, then fall flat together.

## 4. The maths and physics

**Model (catalog row "ChronoMultiverse").** Cowley and Rosensweig's linear analysis for a deep inviscid layer: ρs² = −ρgk − σk³ + μ₀(μ ÷ μ₀ − 1)² ÷ (1 + μ₀ ÷ μ) · H₀² k² (Cited, Abou, Néron de Surgy and Wesfreid eq. 1, [arXiv physics/9706005](https://ar5iv.labs.arxiv.org/html/physics/9706005)). Onset: k_c = √(ρg ÷ σ) = 1 ÷ l_c, independent of the field (same source, eq. 2; Gollwitzer et al. nlin/0604009 eq. 2.7). Critical magnetisation M_c² = (2 ÷ μ₀)(1 + 1 ÷ r₀)√(gρσ), r₀ = √(μ_c μ_t) ÷ μ₀ for a nonlinear M(H), with B_c = μ₀(H + M) (Cited, nlin/0604009 eqs 2.5, 2.6, after Rosensweig 1985 §7.1). Near onset the surface is ζ = A Σ cos(k_i · x), |k_i| = k_c, three vectors 120° apart; the fundamental Fourier mode fits the measured spike shape within 10% up to ε = 0.4 (Cited, nlin/0604009 abstract). Above that the tips are sharper than the cosine sum: MODEL harmonics, named as such.

**Computed (g 9.80665; μ₀ 4π × 10⁻⁷; linear law M = χH):**

| Fluid (source) | ρ, σ, χ | k_c | λ_c = 2π√(σ ÷ ρg) | hexagon spacing 2λ_c ÷ √3 | M_c | B_c, linear law |
|---|---|---|---|---|---|---|
| APG 512A (nlin/0604009 Table 1) | 1236, 30.57 mN/m, 1.172 | 0.6297 mm⁻¹ | 9.978 mm | 11.522 mm | 6,689 A/m | 15.58 mT |
| EMG 901 (nlin/0607059 Fig. 2) | 1406, 25.0, 2.2 | 0.7426 | 8.461 | 9.769 | 6,228 | 11.38 |
| APG E32 at 10 °C (1603.01073 Table I) | 1168, 30.9, 3.74 | 0.6088 | 10.320 | 11.916 | 6,022 | 9.59 |

- **Checks against sources.** k_c and λ_c for APG 512A match the cited 0.629 mm⁻¹ and 9.98 mm. The linear B_c (15.58 mT) is below the nonlinear-law prediction of 17.22 mT (Cited, nlin/0604009 §2.4) and the measured 16.7 mT (R1): the effective susceptibility at 16 mT is lower than χ₀. For APG E32 the paper's semi-infinite value is 10.5 mT against 9.59 here. So **the unit tests assert λ_c, k_c and the spacing, not a B_c from the linear law**; if a field readout is drawn it uses B_c of the reference fluid as a labelled constant.
- **Magnetic Bond number.** Jamin et al. define Bo_m ≡ B²R ÷ (μ₀γ) for a drop of radius R (Cited, [arXiv 1604.07686](https://arxiv.org/html/1604.07686)). For a layer the length is l_c (Our reading), giving Bo_m = B² ÷ (μ₀√(ρgσ)), or in the magnetisation form Bo_M = μ₀M² ÷ √(ρgσ). **Computed:** the onset is Bo_M,c = 2(1 + 1 ÷ μ_r) exactly, which is 2.921 for APG 512A, 2.625 for EMG 901 and 2.422 for APG E32 (equivalently Bo_m,c = 2μ_r(μ_r + 1) ÷ χ² = 10.03 for APG 512A). The dash's dimensionless field is then ε = Bo_M ÷ Bo_M,c − 1. Note: the ferrogel review prints B_c² = 2μ₀μ_r(μ_r + 1)(√(σρg) + G) ÷ (μ_r − 1) ([arXiv 0811.1526](https://ar5iv.labs.arxiv.org/html/0811.1526) eq. 1, read through ar5iv); with G = 0 it matches the value here only with (μ_r − 1)² in the denominator, so the exponent was likely lost in rendering. Step 1 checks the PDF.
- **Hexagon lattice, computed.** With k_i at 0°, 120°, 240°: k₁ + k₂ + k₃ = (−2 × 10⁻¹⁶, 3 × 10⁻¹⁶); ζ unchanged by a 60° turn about a peak (largest error over 200 random points 2.5 × 10⁻¹⁴); ζ runs from +3.000 at the peaks to −1.500, so A > 0 gives sharp-topped spikes over broad troughs, the right way up. Nearest peaks lie at 30°, 90° and 150° (perpendicular to the k_i) at a = 2λ ÷ √3 = 1.1547 λ, where ζ = 3.000; at 0°, 60° and 120° the same distance gives ζ = −1.205.
- **Side-on, computed.** Seen along a lattice row, the rows behind project between the front row's spikes: the apparent pitch is a ÷ 2 = λ ÷ √3 (5.76 mm for APG 512A), alternate spikes one row further back. A crown arc seen side-on is therefore two staggered ranks, not one row. Our reading: the back rank dimmer and lower, which gives depth for free.
- **Amplitude, computed with R2's EMG 901 fit (A read as mm; step 1 confirms the unit against Fig. 3a):** A_H = 4.53 at ε = 0, 4.92 at 0.05, 5.29 at 0.1, 5.98 at 0.2, 7.83 at 0.5, 10.59 at 1.0 (B = 1.414 B_c). The fold is at ε_F = −0.2025 (B = 0.893 B_c) with A = 1.81. **The jump at onset is 43% of the height at ε = 1:** the brief's "spike height 40 to 100%" is this curve. For APG 512A the 0.2 mT hysteresis (R3) is ε_F ≈ −0.024.
- **Spike sharpness, computed (EMG 901 spacing 9.769 mm, cone through peak and neighbour trough).** Full apex angle 139° at the fold, 94° at ε = 0, 64° at 0.5, 50° at 1.0; height ÷ spacing 0.19 to 1.08. The brief's 38° to 18° needs height ÷ spacing 1.45 to 3.16, the tall slender spikes of a steep magnet (photos 2, 3), not a uniform-field lattice. See §9, item 3.
- **Squares.** Square spacing equals λ (nlin/0607059 §3.3: the hexagonal peaks are "wider spaced" by a factor read here as 2 ÷ √3 = 1.155, the symbol lost in rendering). At equal wavelength, the change to squares packs the spikes 13.4% closer, a little less in practice because the square wavenumber is lower (R5). For APG 512A the change runs from ε = 0.536 (20.7 mT) to 1.692 (27.4 mT); Boyer and Falcon give B* = 1.45 (ε = 1.10).
- **Effective gravity, computed.** λ_c ∝ g_eff^(−½) and M_c ∝ g_eff^(¼): at g_eff = 0.5 g the spacing grows by 41% and the threshold falls 16%; at 2 g the spacing shrinks to 0.707 and the threshold rises 19%.

**What the tests can assert (proposed):** k₁ + k₂ + k₃ = 0; ζ invariant under 60°; peak spacing 2λ_c ÷ √3 = 11.522 mm and λ_c = 9.978 mm for APG 512A; λ_c independent of ε; Bo_M,c = 2(1 + 1 ÷ μ_r) = 2.921 at χ 1.172; A_H(0) = γ ÷ g_h = 4.532 and ε_F = −0.2025 for γ 0.281, g_h 0.062; ζ ≡ 0 for ε < ε_F, and for ε < 0 on the rising branch.

**Catalog row: no value is wrong; three additions are needed** (the lead reports them). (1) The row has no onset value or fluid: add M_c² = (2 ÷ μ₀)(1 + 1 ÷ r₀)√(ρgσ), APG 512A as the reference fluid, λ_c = 9.978 mm, spacing 11.522 mm, and the magnetic Bond number onset 2.921. (2) "λ_c does not depend on the field" holds for the linear onset only; measured k falls slightly with field (R5) and depends on layer depth (Abou et al.); the tests assert the linear value, labelled as such. (3) The brief's "lattice compressing toward the front like a field gradient" runs against R5 (more field, slightly longer wavelength); the honest compression is the hexagon-to-square packing (13.4% at most) or g_eff.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Spike front position along the crown | `rpm`, `idle_rpm`, `max_rpm` | verified | exact rpm fraction on the `ribbon` spring with feed-forward |
| Field ε (spike height through A_H(ε), with the hysteresis loop) | `throttle` | verified | `snap` spring, 2% deadband, then ε from throttle; height from R2's curve, never linear |
| Extra field (steeper spikes at fixed spacing) | `boost` | verified | adds to ε, capped; sharpness follows from height ÷ spacing (§4) |
| Pool mean-level tilt; spikes stay along the field | `g_lat`, `g_vert` | verified (acceleration) | tan θ = a_lat ÷ (g + a_vert) on `fluidSlosh`; the "lean" is the spikes against the tilted pool |
| Lattice spacing and threshold over crests and dips | `g_vert` | verified (acceleration); sign of the field confirmed at step 1 | g_eff = g + a_vert, λ ∝ g_eff^(−½), clamped 0.5 g to 2 g |
| Foreshortening under braking | `g_long` | verified | design mapping, named |
| Pole caps (shift strip) | `rpm`, `shift_rpm`, per-gear table | verified / derived | per-gear thresholds learned live |
| Pole-piece gear and the collapse wave | `gear`, optionally `clutch` | gear verified; clutch decoded, offset confirmed at check C | glyph swaps in frame 1; the clutch, when present, is the field interruption |
| Speed digits | `speed`, `unit` | verified | tabular, `Hysteresis` |
| Ferro-drop spike count (0 to 7) | `tyre_combined_slip` | decoded, offset confirmed at check C | per wheel; fallback body slip β = atan2(v_x, v_z) for all four |
| Ferro-drop rim tint | `tyre_temp` | decoded, offset confirmed at check C; unit unconfirmed (Forza titles have sent °F) | 4 steps; Our reading: magnetic susceptibility falls with temperature (Langevin), so a hot drop needs more slip to spike, which is an honest coupling |
| Pool shiver on kerbs (optional) | `rumble_strip`, `surface_rumble` | decoded, offset confirmed at check C | a small vertical-forcing term, zero on smooth road so idle stays still |

**What the game cannot send** (from the benchmark, and from the lab rig):
1. **The room's sound** that drives "Protrude, Flow" (B2). The game sends no audio. **Replacement (recommended): the engine is the voice.** Field ε = throttle plus boost, so the fluid "pulsates" with the engine as the installation does with the room. Second idea: the firing frequency f = rpm ÷ 60 × `cylinders` ÷ 2 (decoded, offset confirmed at check C) as a faint shimmer of the front spike only, capped so it never flickers.
2. **The gaussmeter and coil current** of a lab rig. **Replacement: a micro-readout of the MODEL field**, B = B_c √(1 + ε) in mT for the reference fluid, labelled MODEL. Second idea: the pole caps double as the coil's current steps.
3. **The projection screen** of the installation (B3). **Replacement:** none needed as a part; the dash itself is the screen. The camera's hard top light becomes the G-moved softbox of the family.
4. Fuel, range and the like are not in the benchmark. **Optional idea:** `fuel` (decoded, offset confirmed at check C) as the pool's depth, which in a thin layer shifts k_c slightly (Abou et al.); MODEL, off by default.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| `max_rpm` 0 or tiny, `idle_rpm` ≥ `max_rpm` | front hidden, pool flat, gear and speed shown; clamp before any division |
| Idle near max | the front's range collapses; keep a minimum span, spikes still from throttle |
| 12,000+ rpm or a 4,500 rpm diesel | the front is a fraction of the car's own scale; spacing never changes with rpm, so nothing freezes or crowds |
| EV (one gear, no idle, odd rpm) | front from rpm ÷ max; no firing shimmer (cylinders 0); field from throttle as usual |
| Reverse, neutral, gear above 10 | R and N on the pole piece; no collapse wave for N to R; gears above 10 show as the number, the per-gear table clamps at 10 |
| mph and km/h, hp and kW | units from `Signals`; no figure depends on them |
| Limiter bounce of about 50 rpm a frame | the front follows rpm exactly; the hex-to-square limiter state uses its own proteresis (R4), so it does not flicker; digits use hysteresis |
| Spikes and crashes (huge G) | g_eff clamped to 0.5 to 2 g; tilt clamped; past 0.14 λ the lattice melts (R9) and re-forms, never tearing off the screen |
| Airborne (g_eff toward 0) | physically the threshold falls to zero; the clamp holds the lattice at its 0.5 g spacing and the pool floats level |
| Stale or paused stream | the app hides the dash; on return the pool starts flat and the field ramps (start-up) |
| Palette extremes, random palettes | the fluid is true black with specular from the studio; only the ridge tint and pole caps use palette roles, checked for contrast against #000 |
| Font swap | the pole-piece outline is built from the glyph's outline, so micro-spikes follow any font; the gear keeps an ink fill |
| Cockpit and hood | hood flattens the crown (brief); the side-on two-rank view keeps reading at the lower rise |
| Reduced flash | pole caps breathe at 2 Hz; the field never pulses visibly at the limiter |
| Colour-only cues and colour vision | slip reads by spike count on the drops, not colour; temperature tint has a matching step in rim width |

## 7. Signature moments

**Limiter (only the pole caps strobe: 10 Hz at 50% duty, 2 Hz breath in gentle mode).**
- **L1, the ferrosoliton crown (recommended).** At the limiter the redline end of the crown gets a local field boost and one tall spike forms there with its circular dip, as in R6; it stays, lit steadily in `shift`, until the revs fall back below the fold. This grounds the brief's "lead spikes fuse into one tall crown spike" in a measured object.
- **L2, hexagons to squares.** Past the second threshold (R4) the lattice changes symmetry and the spikes pack up to 13.4% closer; leaving the limiter it changes back at a different point (proteresis). A real pattern change with no brightness change.

**Gear change.**
- **G1, the brief's collapse wave, on the upper branch (recommended).** A dip in ε travels along the crown; each spike slumps to 30% and recovers. **Computed:** 30% of A(ε = 1) is 3.2 against the fold's 1.81, so the slump stays on the spike branch and the spikes never vanish, as the physics requires for a recovery without re-nucleation. The real wave speed (0.176 m/s) would take seconds over a crown of 63 spikes, so the 168 ms travel is a design time, named.
- **G2, clutch interruption.** When `clutch` is present, the field drops below the fold for the clutch's duration: the lattice falls flat at once and re-erupts as a front from the pole piece when the clutch bites (growth tens of ms, §3). Fallback to G1 when the clutch is not sent.
- **G3, the pole piece hands over.** The micro-spike fringe migrates from the old glyph's edge to the new (Morpho Tower, B4), over the `fast` spring (T 0.168 s).

## 8. What it needs from the 4K library

**The pre-build kit (PR #18 branch `claude/project-thread-ib67x9`):** none. `tools/asset-forge/src/prebuild/kits.rs` lists ChronoMultiverse among the "Dashes with nothing layout-independent to make have no kit" (header, line 12), and `PREBUILD.md` says it "draw[s] their figure from the model". There is no `assets/clusters/chrono_multiverse/prebuild/manifest.json`. **So the prebuild memory's plan of ferrofluid parts (a black matcap and Rosensweig spike sprites) is not in kits.rs.** Spike sprites would also break the category's rule 5 (baked assets never draw the figure).

**Shared parts it can reuse:**
- `liquid_metal_studio_env` (octahedral, 2048 runtime, 4096 master) and `liquid_metal_env_band` (256 × 1 HDR), from the `KineticLiquidMetal` and `FaradayMandala` kits: the studio the black fluid mirrors.
- `polished_steel_matcap` (Blender, 1012 px, `PaganiHorologerie` kit): a candidate for the pole piece if it reads as steel.
- `assets/glass/blue_noise_128.png` (the brief's 128² blue noise) and `assets/fonts/exo-2/` (the brief's Exo 2).
- Phase 6 `assets/fluid/matcap_ferro.png` (1024², Blender studio with four softboxes) and `plate_ferrofluid_macro.png` (2048 px, SDXL): references only. The plate is below 4K, and the matcap uses a different studio from the computed env, so its highlights would not match.
- Phase 6 `ferrofluid_beauty_8x8.png` and its normal flipbook: drawn from "looped 4D Voronoi spikes (Rosensweig-style)" (`assets/ASSETS.md`), not the model, so it cannot be the brief's fallback (rule 5).
- The kit's `StandingWaves` (3 waves = hexagons, CPU mirror `sample`) evaluates ζ exactly, but from above; this dash is side-on solid spikes (rule 6), so it needs the mesh pass or computed facets.

LIBRARY GAP: computed `ferrofluid_matcap` (1024 runtime, 4096 master) and `ferrofluid_fresnel` (256 × 1) from the shared studio, for a black, opaque, weakly reflecting dielectric; made on the CPU by `tools/asset-forge/src/prebuild/liquid.rs`, which today knows only Chrome and Mercury. Needs the fluid's optical constants first: one source gives n ≈ 2.9 + 0.015i at 633 nm without naming the sample ([EPJ Web Conf. 185, 09007](https://www.epj-conferences.org/articles/epjconf/abs/2018/20/epjconf_mism2017_09007/epjconf_mism2017_09007.html)), which would mean about 24% head-on reflectance, against about 4% for an oil carrier near n 1.5. Step 1 reads Figs 2 to 4 of that paper before the matcap is made.

Layout-dependent, not gaps: the pole-piece glyph mesh (built from the font outline), the crown pool and its trough, and any coil or housing hardware.

## 9. Departures and open questions for the board

1. **Spike height is A_H(ε), not linear in throttle.** It jumps to 43% at onset and has a fold below it; this matches the brief's 40 to 100% while making the hysteresis real.
2. **Lattice compression toward the front** (brief) is replaced by the hex-to-square packing or g_eff, because measured wavelength grows slightly with field (R5).
3. **Apex angles 38° to 18°** (brief) are steeper than a uniform-field lattice reaches (94° to 50°, §4). Either accept them as a steep-magnet look (photos 2, 3) and name it, or keep the lab proportions. The board decides; the blind glance read should judge which reads as ferrofluid.
4. **Lean under gLat**: the pool tilts and the spikes stay along the field. This is a reversal of the brief's "shear on fluidSlosh" in the screen frame; the board shows both.
5. **The fallback flipbook** of the brief is dropped: the existing one is not the model (rule 5), and the live surface is cheap.
6. **Two staggered ranks** along the crown (side-on projection, §4), not one row.
7. **Gear-change wave speed** is a design time (168 ms), not 0.176 m/s.
8. **Telemetry now decoded:** combined slip replaces the interim slip angle; tyre temperature needs its unit confirmed at check C.
9. **Thumbnail distinctness:** `FaradayMandala` (hqrmcu) is a bright mercury mirror seen from above, lit by a lamp ring, with oscillating standing-wave lattices (squares, hexagons, twelve-fold) strobed at f ÷ 2. `KineticLiquidMetal` (nrluim) is one chrome blob with star modes and an oil-film sheen. This dash is black, opaque, static spikes seen side-on, with no film colour and no oscillation. Never show the hexagon field from above: that is the mandala's view and the same Σcos.
10. **Open:** the optical constants of the fluid (§8); the sign of `acceleration_y`; whether the EMG 901 amplitude fit is in mm.

## 10. Sources

**Benchmark and art**
- SIGGRAPH History Archives, "Protrude, flow": https://history.siggraph.org/artwork/sachiko-kodama-minako-takeno-protrude-flow/
- Kodama, "Dynamic Ferrofluid Sculpture: Organic Shape-Changing Art Forms", CACM 51(6), 79 to 81 (2008): https://cacm.acm.org/research/dynamic-ferrofluid-sculpture/

**Physics**
- Abou, Néron de Surgy, Wesfreid, dispersion relation in a ferrofluid layer: https://ar5iv.labs.arxiv.org/html/physics/9706005
- Gollwitzer, Matthies, Richter, Rehberg, Tobiska, surface topography of a magnetic fluid: https://ar5iv.labs.arxiv.org/html/nlin/0604009
- Gollwitzer, Rehberg, Richter, via hexagons to squares in ferrofluids: https://ar5iv.labs.arxiv.org/html/nlin/0607059
- Lange, Gollwitzer, Maretzki, Rehberg, Richter, retarding the growth of the Rosensweig instability: https://ar5iv.arxiv.org/html/1603.01073
- Gollwitzer, Krekhova, Lattermann, Rehberg, Richter, surface instabilities and magnetic soft matter: https://ar5iv.labs.arxiv.org/html/0811.1526
- Jamin, Djama, Bacri, Falcon, tuning the resonant frequencies of a drop (magnetic Bond number): https://arxiv.org/html/1604.07686
- Vieu and Walter, shape and fission instabilities of ferrofluids: https://ar5iv.arxiv.org/html/1711.09942
- Gollwitzer et al., NJP 11, 053016 (2009), via the `KineticLiquidMetal` note: https://epub.uni-bayreuth.de/3966/1/Gollwitzer_2009_New_J._Phys._11_053016.pdf
- Boyer and Falcon, two-dimensional melting of a crystal of ferrofluid spikes, via the same note: https://arxiv.org/pdf/0909.3504
- Yerin, Lykhmanova, Erina, complex refractive index of concentrated magnetic fluids: https://www.epj-conferences.org/articles/epjconf/abs/2018/20/epjconf_mism2017_09007/epjconf_mism2017_09007.html

**Photos** (Wikimedia Commons, licences in §2): the ten Commons pages listed in §2.

**Repo** (read, not changed): `program/phases/07-cluster-factory/CATALOG.md` (brief, maths row, rules), `CLUSTER_PROMPT.md`, `docs/design/families/fluid-generative.md`, `docs/TELEMETRY_USAGE_MATRIX.md`, `docs/kit/README.md` (StandingWaves, conductors, meshes), `assets/ASSETS.md`, `data/design_tokens.json`; PR #18 branch `PREBUILD.md` and `kits.rs`; sibling notes on `origin/claude/project-thread-nrluim` and `origin/claude/project-thread-hqrmcu`.
