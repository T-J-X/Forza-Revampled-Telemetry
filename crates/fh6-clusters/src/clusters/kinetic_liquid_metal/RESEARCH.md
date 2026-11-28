# Kinetic Liquid Metal: research

Catalog row 27, build #7, Psychedelic category, target `concept`, Fluid / Generative family, cockpit. Written 2026-11-28 from a web search, as step 1 of `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` asks of every dash whatever its target.

## The benchmark

A `concept` names one real object that the visual review compares it with. This dash is benchmarked against the **lens-front mercury-in-glass thermometer and the laboratory mercury U-tube manometer**: a sealed borosilicate tube with a thin bright thread of mercury inside, read at its meniscus against etched graduations, the tube's front face moulded into a cylinder lens that magnifies the thread.

What makes that instrument recognisable, and where the dash keeps it:

| Feature of the real instrument | Source | In the dash |
|---|---|---|
| A thin mercury thread in a sealed glass bore; the reading is the end of the thread | US patent 1,819,919 (Palmer, 1931), [patents.google.com/patent/US1819919](https://patents.google.com/patent/US1819919) | The tach is the mercury column in the 240° channel; the tip of its meniscus is the exact rpm |
| The reading face of the tube is formed into a lens "to provide the observer with a magnified image" | US1819919; US patent 1,590,613 (Benedict, 1926) has "a magnifying section disposed directly over the mercury tube", [patents.google.com/patent/US1590613](https://patents.google.com/patent/US1590613) | The channel is a half rod of the kit's optical glass over the thread: it magnifies the thread by real refraction (`docs/specs/optics.md` §4) |
| A colour stripe (white enamel, or red glass) behind the bore, reflected by the mercury so the thread looks coloured | US1819919: "Behind the bore is a color stripe 5 which may be white enamel" | The studio the mercury mirrors carries a stripe in the palette's `primary`: the thread's tint toward `primary` (tokens `materials.liquidMetal.tintMix` 0.18) is that reflection, not paint |
| Etched graduations, longer every tenth, numbered beside the tube | the lab instrument itself; OIML R 7 for clinical thermometers, [oiml.org/en/files/pdf_r/r007-e79.pdf](https://www.oiml.org/en/files/pdf_r/r007-e79.pdf) | A mark every 500 rpm on the glass, longer every 1000, numerals inboard of the channel |
| A convex mercury meniscus (mercury does not wet glass) | Cambridge fluid-properties notes: the contact angle "is greater than 90°" for mercury, so the level is depressed, [www-mdp.eng.cam.ac.uk … node18](https://www-mdp.eng.cam.ac.uk/web/library/enginfo/aerothermal_dvd_only/aero/fprops/propsoffluids/node18.html); 140° is the textbook value for mercury on soda-lime glass, [byjus.com](https://byjus.com/question-answer/mercury-has-an-angle-of-contact-equal-to-140deg-with-soda-lime-glass-a-narrow) | The meniscus is a spherical cap for θc = 140°: 0.466 of the bore radius tall (`physics::meniscus_height`) |
| A spirit-level vial: a curved, part-filled tube whose bubble sits at the highest point | Wikipedia, Spirit level: vials have "a slight upward curve, so that the bubble naturally rests in the center, the highest point", filled "usually [with] a colored spirit or alcohol", [en.wikipedia.org/wiki/Spirit_level](https://en.wikipedia.org/wiki/Spirit_level) | The G vial under the speed: its bubble runs R_v × θ along a vial bent to radius R_v, θ being the hydrostatic tilt |

The brief's other reference, the **Lexus LFA meter**, is the reason the instrument is a liquid drawn live rather than a mechanism: "no conventional rev counter would be able to live with the unloaded V10's ability to go from idle to 9000rpm in 0.6sec", so Lexus drew it on a TFT (evo, "Lexus LFA rev counter – Art of Speed", [evo.co.uk](https://www.evo.co.uk/lexus/lfa/202607/lexus-lfa-rev-counter-art-of-speed)). The dash keeps that idea: the column follows rpm on the tokens' `ribbon` spring with feed-forward, so the thread is never behind the engine. Its look is not copied. Terminator 2 is a register for the material only.

## The physics, with sources (catalog row "Real geometry and maths")

| Model | Equation | Source | Value the tests assert |
|---|---|---|---|
| Free surface under acceleration | tan θ = a_H ÷ (a_V + g) | CodeCogs, "Fluid masses subjected to acceleration along an inclined plane", [codecogs.com/page/1443](https://www.codecogs.com/page/1443) (and tan θ = a ÷ g for a purely horizontal acceleration, [codecogs.com/page/1441](https://www.codecogs.com/page/1441)) | 1.0 g → 45.00°, 0.5 g → 26.57° |
| U-tube column | ρSL ÿ + bẏ + 2ρSg y = 0, so ω₀ = √(2g ÷ L) | Orjuela et al., "damped oscillations in U-tubes", eq. 1, [arxiv.org/abs/2402.09866](https://arxiv.org/pdf/2402.09866) | T = 0.62 s (tokens `fluidSlosh`) ⇔ L = g T² ÷ 2π² = 0.191 m |
| Volume of the pool | V = π h² (3r − h) ÷ 3, h = r − d (a spherical cap) | geometry of the sphere | the volume below a tilted surface is unchanged at 0°, 15°, 45° and 70° (lattice count within 3%) |
| Rayleigh–Plateau break-up | fastest mode kR₀ = 0.697, λ_max ≈ 9.02 R₀; unstable only for kR₀ < 1; break-up time ≈ 2.91 √(ρR₀³ ÷ σ) | MIT 18.357 Interfacial Phenomena, lecture 11, eqs. 11.30 to 11.33, [ocw.mit.edu](https://ocw.mit.edu/courses/18-357-interfacial-phenomena-fall-2010/d77da8d5f1bb8b69a62682bc0dbc5845_MIT18_357F10_Lecture11.pdf) | λ = 2π R ÷ 0.697 = 9.01 R; one wavelength of thread per drop gives r_d = (¾ R² λ)^(1/3) = 1.89 R (derived here: the lecture gives λ, and volume conservation gives the radius) |
| Mercury's reflectance | conductor Fresnel with n + ik = √ε (`docs/specs/optics.md` §3, conductors); ε = −18.46 + 13.50 i at 2.27 eV (546 nm) | Smith and Stromberg, J. Opt. Soc. Am. 56, 1539 (1966), as quoted by Arakawa, Inagaki and Williams, "Optical properties of metals by spectroscopic ellipsometry", [osti.gov/servlets/purl/6018359](https://www.osti.gov/servlets/purl/6018359) | n = 1.485, k = 4.546, R₀ = 0.779; the same paper's Drude fit gives R_N = 0.765 ± 0.008 at 2.27 eV |
| Borosilicate glass of the channel, the ampoule, the vial and the test tubes | borosilicate 3.3: n_d = 1.47140, n_F − n_C = 71.4 × 10⁻⁴, so V_d = 66.0; density 2.23 g cm⁻³ | SCHOTT BOROFLOAT 33 technical details, [schott.com](https://www.schott.com/en-gb/products/borofloat/technical-details) (DURAN tubing, also borosilicate 3.3, lists n_d 1.473, [psec.uchicago.edu](https://psec.uchicago.edu/glass/schott_DURAN_datasheet.pdf)) | every glass piece is `Dispersion::from_abbe(1.4714, 66.0)` (`docs/specs/optics.md` §1); R₀ = 3.6% |

**Why vertical, not longitudinal, g.** The shared `common::liquid` tilts its surface by atan(gLat ÷ (1 + |gLong|)). Hydrostatics puts the vertical acceleration in the denominator (CodeCogs above): longitudinal acceleration tilts the surface about the other axis, which a screen-plane pool cannot show. This dash reads the vertical acceleration from the packet (`acceleration_y`, a verified field) and does not use `common::liquid`'s tilt. The handover asks for the shared fix.

**The pool's swing.** The surface's slope is the state of the U-tube column: it reaches the hydrostatic tilt through the column's own under-damped swing (ζ 0.35 from the tokens' `fluidSlosh`), and its ω₀ follows the effective gravity (√(2 g_eff ÷ l)). An upshift knocks it once.

## Deliberate changes

- **No ghost 8s behind the speed.** The brief asks for them; the family spec and the BoschDdu blind read (a ghost "888" read as real digits) say no. The family spec wins (order of authority 3).
- **The G vial shows the hydrostatic tilt, not gLat and gLong separately.** The brief maps gLong to the slug's length; a real vial's bubble does not change length with acceleration. The vial's bubble position is R_v × θ from the same tilt as the pool; longitudinal G is shown as a figure beside it.
- **Surface rumble and suspension ripples are left out.** The decoder does not expose rumble, and suspension travel is not verified against the golden packets (the 97% rule). The surfaces are moved only by verified acceleration and the gear change, so idle is still.
- **The limiter's drops come from a reservoir.** The column's length is the rpm and stays exact, so the drops that pinch off its tip (at 5 Hz, 1.89 × the capillary thread's radius) are drawn from the column's hidden reservoir and return to it when the pool drains after the shift. The mercury is kept in one ledger (`physics::Drips`), so none is made or lost.
- **Mercury, not chrome.** The thread's bands are the studio the metal mirrors, weighted by mercury's own reflectance at each angle across the thread (78% head-on, rising to 100% at grazing), never a flat grey.

## Direction: the Ferrofluid crown (TJX, 2026-11-28)

TJX found the mercury-thermometer reading "not abstract enough" and "boring" (08:13 UTC). Of three abstract directions this dash takes the **Ferrofluid crown**; the other two (an oil-slick chrome blob and a Faraday mandala) become clusters of their own (TJX 08:20: "Let's have all 3 as seperate clusters"). The benchmark for the crown is the Rosensweig, or normal-field, instability: a ferrofluid layer under a vertical magnetic field breaks into a lattice of spikes once the field passes a threshold. The facts below are what the board and the code may draw on; how they are laid out is the Penpot board's job.

| Fact | Source | Values |
|---|---|---|
| The spikes' spacing is set by gravity and surface tension, not by the field: k_c = √(ρg ÷ γ), the inverse capillary length, so λ_c = 2π √(γ ÷ ρg) | Abou, Néron de Surgy and Wesfreid, "Dispersion relation in a ferrofluid layer of any thickness and viscosity in a normal magnetic field", eq. 2, [arxiv.org/abs/physics/9706005](https://ar5iv.labs.arxiv.org/html/physics/9706005); Boyer and Falcon, "Two-dimensional melting of a crystal of ferrofluid spikes", §1, [arxiv.org/abs/0909.3504](https://arxiv.org/pdf/0909.3504) | APG 512A (γ 30.57 mN/m, ρ 1236 kg/m³): λ_c = 9.98 mm, worked here from Gollwitzer et al.'s Table 1; Boyer and Falcon's aqueous fluid: 13.4 mm as stated |
| The linear dispersion relation: ρs² = −ρgk − γk³ + μ₀(μ/μ₀ − 1)² ÷ (1 + μ₀/μ) · H₀² k² | Abou et al., eq. 1 | the field term grows as k², so above threshold a band round k_c grows and the wavenumber stays at k_c |
| Spike height grows as √(B* − 1), B* = B ÷ B_c | Boyer and Falcon §1 ("data can be well fitted by √(B*−1)"), after Gailitis (1977) and Bacri and Salin (1984) | no prefactor given; measured central heights of a single spike run to about 7–15 mm (Gollwitzer et al., Fig. 9, read from the axis) |
| Pattern selection: hexagons at onset, squares above a second threshold | Abou et al., introduction; Boyer and Falcon §1: hexagons for 1 < B* < 1.45, squares above 1.45 | lattice wavelength λ = (√3 ÷ 2)δ for hexagons and λ = δ for squares, δ being the spike-to-spike distance |
| The onset is hysteretic in a wide layer (transcritical), barely so in a small dish | Gollwitzer, Spyropoulos, Papathanasiou, Boudouvis and Richter, New J. Phys. 11, 053016 (2009), §§1, 5, [epub.uni-bayreuth.de](https://epub.uni-bayreuth.de/3966/1/Gollwitzer_2009_New_J._Phys._11_053016.pdf) | ΔB 0.2 mT measured (0.17 mT in a near-infinite layer, Gollwitzer et al. 2007); onset of a single central spike near 16 mT for APG 512A |
| The spike crystal melts when shaken: a spike's rms displacement past 0.14 of the lattice wavelength (Lindemann ratio γ_m = 0.14 ± 0.02, the same for hexagons and squares), through a hexatic-like phase | Boyer and Falcon, pp. 1–3 | d_m = 1.6–2.6 mm rms, rising with B*; no hysteresis in melting |
| The fluid in those experiments | Gollwitzer et al., Table 1 | APG 512A: γ 30.57 mN/m, ρ 1236 kg/m³, η 120 mPa·s, M_S 28.7 kA/m, χ₀ 1.2023, contact angle 10° |

Ideas the research suggests for the telemetry (for the board to pick from, not a layout):
- **rpm as the field.** B* runs from 1 at idle to just past 1.45 at the limiter, so spike height follows √(B* − 1) and the crown snaps from hexagons to squares exactly on the limiter: a real pattern change as the signature moment, with no flashing.
- **The shift point as the onset's hysteresis.** Spikes rise at one field and fall at a slightly lower one, which matches the beads' 120 rpm off-hysteresis.
- **Lateral G and the surface as shaking.** Displacement of the spikes from their lattice sites follows the effective-gravity tilt and the U-tube swing; past 0.14 λ the crystal melts into a disordered field, a physical picture of a slide that the telemetry does drive (verified acceleration only, the 97% rule).
- **Spacing never changes with revs** (λ_c is independent of the field), which keeps the lattice still and readable at a glance: only height and symmetry move.
- **The ferrofluid's own look.** An oil-based colloid of magnetite: a near-black, glossy dielectric surface (a reflectance of a few percent head-on, rising to total at grazing by Fresnel), which shows the studio only on the spikes' flanks: an OLED-black instrument whose light is all reflection. The thread checks the oil's refractive index for `docs/specs/optics.md` before using it.

### The sibling check, and the oil-slick chrome blob

The catalog's sibling check found that `ChronoMultiverse` (build #23) is already a ferrofluid crown: Rosensweig spikes along a crown arc, with the gear as a pole piece. A second crown would look the same at thumbnail size. This thread has asked TJX to move this dash to the second direction, an **oil-slick chrome blob**: a free drop of liquid metal that pulses, stretches and splits, whose skin carries the colours of a thin film. `ChronoMultiverse` keeps the crown, and the facts above are free for its thread. The facts below support the blob. They hold whichever direction TJX picks, and they keep this dash's own maths row (hydrostatics, U-tube, Rayleigh–Plateau).

| Fact | Source | Values |
|---|---|---|
| A drop shaken at frequency Ω answers at Ω ÷ 2 (parametric, a Mathieu resonance) in star-shaped azimuthal modes; Rayleigh's 2D relation ωₙ = √(n(n² − 1)σ ÷ ρR³) (eq. 1); the flattened-puddle form ω²ₙ,ₘ = (n² − 1) k σ J′ₙ(kR) ÷ (ρR² Jₙ(kR)) (eq. 22) | Dong, Liu, Xu, Wang and Wang, "Complete modes of star-shaped oscillating drops", eqs. 1, 15–19, 22, §III, [arxiv.org/abs/1812.10630](https://ar5iv.arxiv.org/html/1812.10630) | water, R 10.8 mm: drive 60 Hz gives n = 4–7; 90 Hz n = 3, 5–9; 120 Hz n = 3–11. k₁ depends only on the drive (0.8, 1.0, 1.4 mm⁻¹) |
| The same star modes appear without a prescribed drive, in Leidenfrost drops (n = 2 to 13 for water), the vapour pressure pulsing at twice the drop's frequency; 2πR = nλ_d | Ma and Burton, "Self-organized oscillations of Leidenfrost drops", introduction, Fig. 1, Table 1, [arxiv.org/abs/1802.06136](https://ar5iv.labs.arxiv.org/html/1802.06136) | which modes appear depends on viscosity |
| Electrocapillarity: γ(V) = γ_pzc − ½ C (V − V_pzc)², valid without reactions; by electrochemical oxidation a gallium alloy's tension falls from about 500 mN/m to near 0 at about 0.8 V, reversibly | Song, Daniels, Kiani, Rashidnadimi and Dickey, "Interfacial tension modulation of liquid metal via electrochemical oxidation", eq. 1 and abstract, [arxiv.org/abs/2102.02383](https://arxiv.org/pdf/2102.02383) | EGaIn 624 mN/m, gallium 708 mN/m in air (water 72); EGaIn in 1 M NaOH ~120 mN/m at 0.5 V; bare-metal capillary length about 3 mm; conventional electrocapillarity alone moves pure Ga only 630 → 540 mN/m; native oxide 1–3 nm |
| Galinstan deformed by electrocapillarity in NaOH at over 120 mm/s, with the Young–Lippmann form γ = γ₀ − ½ C V²_EDL | Gough et al., "Rapid electrocapillary deformation of liquid metal with reversible shape retention", Micro and Nano Systems Letters 3:4 (2015), eq. 1, abstract, [doi.org/10.1186/s40486-015-0017-z](https://link.springer.com/article/10.1186/s40486-015-0017-z) | oxidative spreading 2–3 mm/s; retraction after the bias is removed takes tens of seconds |
| Self-fuelled liquid-metal motors: EGaIn with aluminium in NaOH, driven by an internal galvanic current and hydrogen bubbles | Yao, Sheng and Liu, "Injectable spontaneous generation of tremendous self-fueled liquid metal droplet motors in a moment", [arxiv.org/abs/1504.02851](https://arxiv.org/pdf/1504.02851) | 1 mm droplets at 10–30 mm/s, alive for over half an hour |
| A thin film reflects the colour where its two reflections interfere constructively; with one half-wave phase flip, 2 n d cos θ_t = (m + ½) λ | thin-film interference, [physics.info/thin-films](https://physics.info/thin-films/summary.shtml), [phys.libretexts.org, Thin Films I](https://phys.libretexts.org/Workbench/PhysClips_Light/04%3A_Interference/4.03%3A_Thin_Films_I) | the oil-slick bands run through the orders as the film thins; the colour shifts with the viewing angle through cos θ_t |

Ideas the research suggests for the telemetry (for the board to pick from, not a layout):
- **rpm as the drive.** The blob is shaken by the engine and answers at half its frequency in star modes. The mode number climbs with rpm through the real sequence (n = 3 at idle up to 11 near the limiter), so the outline is a star whose points count up the revs: a figure that is the solution of a named equation, readable as "more points, more revs", with the exact rpm on its own scale.
- **Throttle as the voltage.** Electrocapillarity lowers the tension, so the blob slackens, spreads and its points deepen under throttle, and it pulls back round when you lift, within the measured speeds.
- **The limiter.** At the top the tension nearly vanishes (the oxidation regime) and the drop spreads into fingers or pinches off satellites (Rayleigh–Plateau, this dash's own drop radius of 1.89 R), never a flash.
- **G as the tilt.** The blob slides and leans with the hydrostatic tilt this dash already computes, and a self-fuelled motor's run is the picture of a drift.
- **The oil slick.** A film on the metal shows interference bands whose order follows its thickness; the thickness can follow the blob's stretch, so the colours flow over the skin as it moves and lie still at idle. Colour then comes only from the model, as the Psychedelic category asks.
- No source here gives galinstan's density or viscosity; the thread takes them from a materials datasheet before using them.

| Fact | Source | Values |
|---|---|---|
| Galinstan's bulk properties | Geratherm "Galinstan fluid" safety data sheet §9, [apps.mnc.umn.edu](https://apps.mnc.umn.edu/pub/msds/galinstan_fluid.pdf); Wikipedia, Galinstan, with its cited papers, [en.wikipedia.org/wiki/Galinstan](https://en.wikipedia.org/wiki/Galinstan) | density 6.44 g/cm³; viscosity 2.4 mPa·s at 20 °C; melting point −19 °C; conductivity 3.46 × 10⁶ S/m; surface tension 0.535–0.718 N/m in air, with oxide (by maker); eutectic 68.5 Ga, 21.5 In, 10.0 Sn wt%; above 1 ppm oxygen its skin oxidises to Ga₂O₃, and unlike mercury it then wets glass; "higher reflectivity and lower density than mercury" |
| A large non-wetting drop is flattened by gravity to a puddle about 2 ℓ_c tall, ℓ_c = √(σ ÷ ρg) | Yim, Bouillant, Quéré and Gallaire, "Leidenfrost flows: instabilities and symmetry breakings", §§I, II.B, [arxiv.org/abs/2012.10408](https://arxiv.org/pdf/2012.10408) | H_max ≈ 2 ℓ_c once R > ℓ_c (the θ = 180° case); galinstan ℓ_c ≈ 3 mm from the values above (Song et al. give "~3 mm" for bare liquid metal). A puddle of a few millimetres seen from above is the blob's honest scale: its outline carries the star modes |
| Gravity drainage of a rigid film: ∂h/∂t = −(ρg ÷ 4μ) h² ∂h/∂z, so h ∼ √(z ÷ t) | Mysels (1959) as quoted by Aurégan and Deike, "Drainage and lifetime of thin liquid films", §1, [arxiv.org/abs/2411.03908](https://arxiv.org/pdf/2411.03908) | mobile films drain differently (their eq. 1, h ∝ z ÷ t); the oil film's thickness, and so its colour order, thins from the drop's crown towards its rim over time and is renewed by motion |

**The kit's film tables, checked.** The pre-build kit (`assets/clusters/kinetic_liquid_metal/prebuild/`, `oil_on_mercury_film`) was checked here against an independent Airy sum at normal incidence (oil n 1.47 on mercury n + ik = 1.485 + 4.546i, `docs/specs/optics.md` §9): at 300 nm red dips (0.60 against 0.69 green and 0.78 blue; the table's 140, 179, 201 of 255 follow the same order), and at 450 nm green dips, as in the table. **What it means for the look:** an oil film on a metal this reflective only tints it. Across 0–1200 nm the largest spread between the red, green and blue reflectances is 0.19 for n 1.47, against 0.36 for a film of n 1.9 and 0.52 for n 2.4. So an oil slick on liquid metal is a pastel sheen, not a saturated rainbow; saturated colour on a metal needs a higher-index film, as in the temper colours of an oxide (gallium's own skin is Ga₂O₃, and its index needs a source before the dash uses it). The board takes this into account when it chooses how much colour the skin carries.

## Real behaviour of the drop, worked through (for the board and the code)

**Sources added.** Mercury beating heart: a mercury drop in an acid electrolyte with an oxidiser; an iron nail almost touching it makes it oscillate, because oxidation near the nail lowers the tension locally while reduction restores it over the whole drop (Wikipedia, [Mercury beating heart](https://en.wikipedia.org/wiki/Mercury_beating_heart); first seen by Volta and Henry in 1800). The gallium version "continuously oscillates between a dull flat puddle and a shiny sphere" for about 30 minutes ([Indiana University chemistry demos](https://www.chem.indiana.edu/faculty-research/faculty-resources/chemistry-demos/demo/the-gallium-beating-heart)). Viscous decay of drop modes, Lamb (1932) §355 as given by Padrino, Funada and Joseph, "Purely irrotational theories for the viscous effects on the oscillations of drops and bubbles", eqs. 2.24, 3.12, 3.16 ([umn.edu](https://dept.aem.umn.edu/people/faculty/joseph/archive/docs/946-sphere-short.pdf)): the decay rate of mode ℓ is (ℓ − 1)(2ℓ + 1)ν ÷ a², so τ = a² ÷ ((ℓ − 1)(2ℓ + 1)ν). The beating-heart pages give no shape counts or beat rates; those come from Dong et al. above.

**The numbers for a galinstan drop** (σ 0.535 N/m, ρ 6440 kg/m³, μ 2.4 mPa·s, so ν = 3.73 × 10⁻⁷ m²/s; worked here):
- The drive. An engine's firing frequency is proportional to rpm. A drop answers at half its drive. For a four-cylinder car the half-frequency is rpm ÷ 60 Hz, 15 Hz at a 900 rpm idle. The packet carried no cylinder count when this was written, so the dash folded that constant into the drop's size. Superseded: the decoder now reads `num_cylinders`, see "The cylinder count arrives" below.
- The size. Setting the n = 3 mode at 15 Hz with Rayleigh's 2D relation gives R = 6.08 mm. That is a puddle (ℓ_c = 2.91 mm, so R ≈ 2 ℓ_c, the same ratio Dong et al. used for water), which is the shape that shows star modes from above.
- The modes. n is the mode whose frequency is nearest the half-drive, so n(n² − 1) = 24 (rpm ÷ idle)². For idle 900 and limiter 8700, n runs 3 → 13: 3 at 900, 4 at 1500, 5 at 2200, 6 at 3000, 8 at 3900, 9 at 4900, 10 at 5900, 11 at 7000, 13 at 8200–8700 (figure `star_modes_study.png` in the project files). A car with a narrower band tops out lower (idle 1000, limiter 7000: n ≈ 11), which is real and fine.
- Frequencies and ring-down: n = 3 at 15 Hz rings for τ = 7.1 s; n = 6 at 44 Hz for 1.5 s; n = 8 at 69 Hz for 0.83 s; n = 10 at 96 Hz for 0.52 s; n = 13 at 143 Hz for 0.31 s.

**What that means on a 60 Hz OLED, and the honest way to show it.**
- The star modes oscillate at 15–143 Hz, faster than the display. A real drop shows them only under a stroboscope, which is how the papers photograph them. The dash draws the drop **stroboscopically, at the phase of its peak**, so the star stands still while the drive is steady. Idle is still, as the house rules ask, and the figure is a true sample of the motion, not a stand-in.
- A change of mode is a real transient: the old mode rings down with its Lamb τ and the new one grows. At high n this takes 0.3–0.8 s, a lively morph. At low n it would take seconds, too slow for a gauge, so the morph is capped (a deliberate change, listed at the end of this section).
- An upshift kicks every mode at once and the drop rings, then settles back to its stroboscopic star. That is the upshift's signature, with no flash.

**Telemetry mapping (verified fields only, the 97% rule):**

| Packet input | Drives | Model |
|---|---|---|
| `current_engine_rpm`, `engine_idle_rpm`, `engine_max_rpm` | the number of star points n, and the exact rpm on the tach | n(n² − 1) = 24 (rpm ÷ idle)², nearest integer, 120 rpm hysteresis on each step |
| `accel` (throttle) | how deep the points are | electrocapillary tension γ(V) = γ₀ − ½CV²; the amplitude grows as the tension falls |
| the limiter (rpm within 1% of max) | fingering and satellites | the tension falls towards zero (the oxidation regime); points pinch off as drops of radius 1.89 × the neck (Rayleigh–Plateau), with volume conserved in one ledger |
| `acceleration_x`, `acceleration_y` (lateral and vertical G) | the drop's lean and slide | tan θ = a_lat ÷ (g + a_vert), reached through the U-tube swing (ζ 0.35) |
| `acceleration_z` (longitudinal G) | the signed G figure, and the film's drainage direction | figure in hundredths; drainage h ∼ √(z ÷ t) runs away from the deceleration |
| `gear` | the numeral, and the ring impulse on a change | all modes kicked once; each decays with its own τ |
| `speed_ms` | the tabular speed | display units, 0.6-unit hysteresis |
| `brake` | the brake column (rank 4) | level ∝ brake |
| `velocity_x`, `velocity_z` | body slip, the figure that turns amber above 8° | atan of the lateral over the forward velocity |

**Edge cases to design for:**
- Engine off or rpm 0: no drive; the drop relaxes to a round puddle (n = 0) and stays still.
- Bad car data: idle ≥ max, or idle 0. Clamp the ratio, keep n between 2 and 13, never divide by zero.
- Neutral and reverse: the void shows N or R. Reverse never drives a negative mode.
- Rev-hanging on the limiter: the satellites accumulate. With volume conserved the main drop shrinks a little, and the satellites rejoin it within 300 ms after the shift. The pool can never empty.
- Rapid shifts (under 100 ms apart): ring impulses add up but are clamped, so the numeral void always stays readable.
- Drift at −1 g lateral: the drop leans 44° and slides about half a radius, clamped at 0.6 R so the glass boss under the gear never uncovers (see "The gear void, made physically honest").
- Hostile telemetry (G5 in the dash gates): non-finite values are rejected by the decoder, and any zero still gets its own guard.
- Menus and pause: the dash hides within a frame. On return the drop starts from rest, not from a stale state.
- A car change: idle and max change, so the mapping is recomputed. The drop re-forms with one ring instead of snapping.

**Deliberate changes, liquid-metal version:**
- The drop is drawn stroboscopically (see above), not at its real 15–143 Hz.
- Mode changes are capped at about 0.25 s, against Lamb's 0.3–7 s, so the gauge stays instant. The real τ is kept for the upshift's ring-down at high n.
- ~~The cylinder count is folded into the drop's size.~~ Superseded by "The cylinder count arrives" below.

## The sheen and the ring-down, computed (research figure 02)

Figure `film_and_ringdown.png` in the project files. Worked here with the Airy sum for a single film on a metal: r = (r₀₁ + r₁₂e^{2iβ}) ÷ (1 + r₀₁r₁₂e^{2iβ}), β = 2π n₁ d cosθ₁ ÷ λ, s and p averaged, the substrate at the kit's mercury constant (n 1.485, k 4.546), CIE 1931 colour matching (Wyman, Sloan and Shirley's 2013 fit) under 6504 K light.

**What the sheen can and cannot be.**
- The native oxide on gallium alloys is 0.5–5 nm thick in air, about 3 nm at steady state (RSC Nanoscale Horizons 2024 review, "Interface of gallium-based liquid metals: oxide skin, wetting, and applications", section 2, [rsc.org](https://pubs.rsc.org/en/content/articlehtml/2024/nh/d4nh00067f); an HRSTEM image there shows 2.2 nm on EGaIn). That is far under the first interference order, so **a real galinstan drop shows no colour from its skin**. It reads as a bright grey mirror, a little less white than mercury.
- Colour needs a film tens to hundreds of nanometres thick: an oil slick on the metal, or a thick anodic oxide. The board's sheen is therefore an oil film, as the direction already says, and its thickness is what drains.
- Brightness minima (photopic Y) head-on, oil n 1.47: 76, 264, 455, 645 nm. Oxide n 1.9 (an assumed value for amorphous Ga₂O₃, not checked against a source here): 56, 202, 349, 495 nm. The metal's phase shift moves the first minimum off the quarter-wave value, which is why these are not evenly spaced from zero.
- Oil stays pastel: Y swings only 0.60–0.78. An oxide film swings 0.44–0.79 and is more saturated. So a pastel oil-slick sheen is the physics, not a styling choice, and any strong saturated band would be wrong.
- At 60° the same minima move to 100, 335, 569, 803 nm (oil). The bands stretch towards thicker film, so at the drop's rim a film shows the colour a thinner film shows head-on. The rim runs one colour lower in the sequence than the centre, which the shader gets for free from cosθ₁. The kit's `oil_on_mercury_film` table (x = thickness, y = cosθ) already holds exactly this.

**Motion notes (timings for the board's "motion + material notes" panel and for the code).**
- Upshift kick. Every mode gets one impulse and rings down with its own Lamb τ for R 6.08 mm: n 3 7.1 s, n 6 1.5 s, n 8 0.83 s, n 10 0.52 s, n 13 0.31 s. Low in the rev range a kick lingers as a slow wobble; near the limiter the drop snaps back almost at once. The amplitude is clamped so the gear void stays readable.
- Mode change while revving. Capped at about 0.25 s (the deliberate change above). The new star grows from the old one's crests, never a cross-fade.
- Limiter. On entry the tension ramps towards zero over about 80 ms, the points finger outward, and necks pinch off at the Rayleigh–Plateau ratio (drop radius 1.89 × neck). On exit the satellites coalesce back within 300 ms, with volume kept in one ledger.
- Lean and slide. The U-tube swing at ζ 0.35 overshoots once by about 30% and settles in about a second, which reads as weight.
- Film drainage. Mysels h ∼ √(z ÷ t): after a brake the thick film runs towards the deceleration side, so the colour bands slide forward and thin out over a few seconds. This is the one slow motion in the dash, and it is ambient: it carries no reading the driver needs.
- Idle. Stroboscopic sampling holds the n = 3 star still. Only the film's slow drainage moves, under the house rule for idle motion.

## The cylinder count arrives (decoder main 60c99c9)

The decoder now reads all 88 FH6 fields, including `num_cylinders` (matrix key `cylinders`). That removes the one fudge in the mapping, and it gives the drop the engine's character for free. Real-game check C still has to confirm the offset and units, so the code keeps a fallback.

**The model, now complete.** A four-stroke engine fires cyl ÷ 2 times per revolution, so the firing frequency is rpm × cyl ÷ 120 Hz, and the drop answers at half of it: f = rpm × cyl ÷ 240 Hz. The drop keeps one fixed, real size (R 6.08 mm, the galinstan puddle above), so with Rayleigh's 2D relation n(n² − 1) = 0.1067 f² (f in Hz). For a four-cylinder at 900 rpm that is 15 Hz and n = 3, the same as before.

**What each engine layout draws** (worked here, nearest mode):

| Engine | Idle → n | Limiter → n |
|---|---|---|
| I3, idle 900, limiter 6500 | 11.2 Hz → 2 (a rocking oval) | 81 Hz → 9 |
| I4, 900 / 8700 | 15.0 Hz → 3 | 145 Hz → 13 |
| flat-6, 950 / 9000 | 23.8 Hz → 4 | 225 Hz → 18 |
| V8 cross-plane, 750 / 7000 | 25.0 Hz → 4 | 233 Hz → 18 |
| V8 flat-plane, 900 / 8500 | 30.0 Hz → 5 | 283 Hz → 20 |
| V10, 1000 / 8700 | 41.7 Hz → 6 | 363 Hz → 24 |
| V12, 900 / 9000 | 45.0 Hz → 6 | 450 Hz → 28, capped at 24 |

A small three-cylinder idles as a slow oval and never gets past nine points. A V12 sits at six points at idle and blooms into a fine crown. The driver can tell the engine from the drop before the car moves, and every one of those shapes is the mode a real drop would pick.

**Design consequences.**
- Legibility cap: n ≤ 24. On a drop about 400 px across that leaves about 50 px per point, which still reads at 2 m. Only V12s above about 7700 rpm hit it. This is a deliberate change.
- Lamb ring-down at the new top: n 18 lasts 0.15 s, n 24 lasts 0.084 s. A high-revving big engine settles almost instantly after an upshift, while a small engine wobbles. That is real, and it fits the car.
- The gear void must stay readable at n 24: its radius stays under the inner radius of the crests (R × (1 − a)), the same rule as before.
- The tach's meniscus is unchanged. It shows the exact rpm whatever n is.

**Edge cases this adds.**
- `num_cylinders` 0: an electric car (no firing) or a missing value. Fall back to the old normalised ladder n(n² − 1) = 24 (rpm ÷ idle)², a "virtual four" at the car's own idle, so an EV still gets a living drop.
- Out-of-range values (over 16, or still unconfirmed before check C): clamp to 1–16. A one-cylinder engine is valid (f = rpm ÷ 240) and gives a calm drop.
- Rotary engines: what FH6 reports for them is unknown until check C. Whatever count it sends, the model holds, because a rotor fires once per rotor revolution.
- A car change mid-session: cyl, idle and max all change together, and the drop re-forms with one ring.

**Code-phase notes.** Add `cylinders` to the KineticLiquidMetal row of `docs/TELEMETRY_USAGE_MATRIX.md` in the same commit as the code that reads it, because the G2 gate fails on a listed input the dash does not read. Unit tests will cover: I4 at 900 → 3; V12 at 9000 → capped 24; cyl 0 → the fallback; cyl 40 → clamped; idle ≥ max → no panic.

## The limiter, frame by frame (research figure 03)

Figure `limiter_study.png` in the project files.

**Source.** Eaker, Hight, O'Regan, Dickey and Daniels, "Oxidation-mediated fingering in liquid metals", arXiv 1703.03011 ([ar5iv](https://ar5iv.labs.arxiv.org/html/1703.03011)). A drop of gallium alloy in electrolyte stays a smooth sessile drop for 0.2 V < E < 0.8 V (their scale, against Ag/AgCl with a 1.5 V offset). Above 0.8 V it spreads and "becomes unstable to undulations along its perimeter" (their Regime B), reaching about 20× its first area. The outline is fractal, D = 1.30 ± 0.05 for every drop volume, "smaller than the value observed in viscous fingering (D = 1.7)". After about 10 s of spreading it contracts again at higher potentials, and pieces that lose electrical contact "bead up into a sphere". The paper gives no finger wavelength and no retraction speed.

**What the dash takes from it.**
- The limiter is the oxidising regime: tension falls towards zero and the star turns into fingers. The finger outline is drawn with a measured target, D ≈ 1.3, not by eye.
- The study outline is r(θ) = R[1 + a cos 13θ + Σₖ k^−(H+½) cos(kθ + φₖ)] for k 14–159, with H 0.55 and fixed random phases (one seed per car, so it never flickers). It is floored softly above the gear void and rescaled so the area stays constant. Box counting on the drawn outline gives D 1.04 for the plain star, 1.15 at 40 ms, 1.25 at 80 ms and 1.24 at pinch-off. That is close to the measured 1.30 at this sampling, and well under viscous fingering's 1.7. So the shape reads as liquid metal, not as paint.
- Timeline (a deliberate compression of the real 10 s): 0 ms star at n 13 (or the car's n), 40 ms points deepen, 80 ms fingers and tip-splitting, 120 ms the 13 longest tips pinch off as satellites of radius 1.89 × the neck. Each tip that leaves shortens by the satellite's volume.
- The drop's area is held constant, against the real 20× spread, so it stays inside its zone and the gear void stays clear at every frame.
- On the shift the satellites coalesce back within 300 ms (they "bead up" while apart, so they are drawn as true circles), and the drop returns to its stroboscopic star.

**Cost note for the code phase.** 146 harmonics × 1024 angle samples is about 150k multiply-adds per frame, under the 0.3 ms CPU budget only if it runs during the limiter alone. The plan: bake r(θ) into a 1024 × 1 texture on the CPU, only while the limiter is active, with 48 harmonics (enough for D ≈ 1.25 at 4K, to be checked with the same box count in a unit test). The shader reads that texture for the outline.

## The gear void, made physically honest

The board punches the gear numeral through the drop as a void with a bright lip. Free liquid will not hold a hole like that, so this works out what keeps one open.

**What real liquid does with a hole.**
- Holes in a layer of liquid have a critical size. Taylor and Michael (1973), "On making holes in a sheet of fluid", J. Fluid Mech. 58 ([cambridge.org](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/on-making-holes-in-a-sheet-of-fluid/EE136E61382830357E5A9694172166AB)), blew holes in water and mercury layers with air jets: "there exists a critical hole size above which larger holes grow and below which smaller holes heal" (as summarised by Bostwick, Dijksman and Shearer, "Wetting dynamics of a collapsing fluid hole", [arXiv 1512.07826](https://arxiv.org/pdf/1512.07826)). The size is of the order of the layer's depth. My estimate for galinstan on glass, at a 140° contact angle, is a depth of 2ℓ_c sin(θ ÷ 2) = 5.5 mm, close to the drop's own radius. A free hole in this drop would either heal or tear the drop into a ring.
- A ring of liquid is unstable too. Yao and Bowick, "The shrinking instability of toroidal liquid droplets in the Stokes flow regime" ([ar5iv 1011.3437](https://ar5iv.labs.arxiv.org/html/1011.3437)): "the torus shrinks to close its interior hole, eventually becoming a single ball", unless the Rayleigh instability gets there first and splits a thin ring into droplets.

**The honest fix: a glass boss.** The gear sits on a clear, flat-topped glass boss at the centre of the drop. Galinstan under its acid film does not wet glass (θ ≈ 140°, the mercury-on-glass figure already in the sources), so the metal wraps around the boss and pulls back from it with a convex, rolled meniscus, the same roll as the drop's outer edge. The void is real: it is the boss's footprint, held open by the solid. Nothing about it needs a deliberate change.

**What the lip looks like, and why it is bright.**
- The rolled edge turns the surface normal from straight up, on the drop's top, through horizontal, to past horizontal where it meets the glass. A narrow band of the edge therefore reflects every elevation of the studio, from the ceiling down to the floor, and the softbox lands on it as a thin, bright line. This is the "bright meniscus lip" on the board, and it has the same physical cause as the bright outer rim of any mercury puddle.
- The roll is about ℓ_c wide (2.9 mm, about half the drop's radius at the real scale). The softbox highlight inside it is much narrower: about ℓ_c × (the softbox's angular size ÷ 90°). For a 15° softbox that is about a sixth of the roll. On a lip about 40 px wide on the 4K canvas, that gives a highlight line of 6–7 px.
- Mercury's reflectance rises from 0.78 head-on towards 1 at grazing (the kit's conductor Fresnel), so the outer, more grazing part of the roll is the brightest. The lip's line sits on the side of the roll towards the boss.
- Seen through the boss, the glass top shows the OLED black and the gear numeral. The numeral is drawn on the boss itself, so it never moves with the drop. The drop's lean and slide move the metal around the boss, and the lip thins on one side and fattens on the other. That reads as weight, and the numeral stays still for reading.
- Limit: the drop's lean must never uncover the boss on one side (that would happen past about a 0.6 R slide). The slide is clamped there; at −1 g lateral the 44° lean slides it about 0.5 R, so it fits.

## Reference index for the Penpot board session

Look at these before designing, by topic:
- Star-shaped drops and their mode shapes: Dong et al., arXiv 1812.10630 (complete star modes and the half-frequency response); Ma and Burton, arXiv 1802.06136, Fig. 1 (star-shaped Leidenfrost drops seen from above).
- Liquid metal changing shape with voltage: Song, Daniels, Kiani, Rashidnadimi and Dickey, arXiv 2102.02383 (tension falling with oxidation); Micro and Nano Systems Letters 3:4 (2015), doi 10.1186/s40486-015-0017-z (electrocapillary shape change).
- The limiter's fingers: Eaker et al., arXiv 1703.03011 (fractal fingering, D 1.30).
- The beating heart (a drop pulsing between shapes): Wikipedia, "Mercury beating heart"; Indiana University's gallium beating heart demo.
- The edge and lip, and the puddle's height: Yim, Bouillant, Quéré and Gallaire, arXiv 2012.10408; MIT 18.357 lecture 11 (puddles and ℓ_c).
- Thin-film colour on the metal: figure 02 in the project files, and the kit's `oil_on_mercury_film` table.
- Figures from this research: `star_modes_study.png`, `film_and_ringdown.png`, `limiter_study.png` in the project files under kinetic_liquid_metal/research/.
