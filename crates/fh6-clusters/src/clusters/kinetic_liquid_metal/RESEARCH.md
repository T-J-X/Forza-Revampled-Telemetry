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
| Mercury's reflectance | conductor Fresnel with n + ik = √ε; ε = −18.46 + 13.50 i at 2.27 eV (546 nm) | Smith and Stromberg, J. Opt. Soc. Am. 56, 1539 (1966), as quoted by Arakawa, Inagaki and Williams, "Optical properties of metals by spectroscopic ellipsometry", [osti.gov/servlets/purl/6018359](https://www.osti.gov/servlets/purl/6018359) | n = 1.485, k = 4.546, R₀ = 0.779; the same paper's Drude fit gives R_N = 0.765 ± 0.008 at 2.27 eV |
| Borosilicate glass of the channel and the ampoule | n_d = 1.473, density 2.23 g cm⁻³ | Schott DURAN data sheet, [psec.uchicago.edu/glass/schott_DURAN_datasheet.pdf](https://psec.uchicago.edu/glass/schott_DURAN_datasheet.pdf) | the glass pieces use n_d 1.473 (`docs/specs/optics.md` §1) |

**Why vertical, not longitudinal, g.** The shared `common::liquid` tilts its surface by atan(gLat ÷ (1 + |gLong|)). Hydrostatics puts the vertical acceleration in the denominator (CodeCogs above): longitudinal acceleration tilts the surface about the other axis, which a screen-plane pool cannot show. This dash reads the vertical acceleration from the packet (`acceleration_y`, a verified field) and does not use `common::liquid`'s tilt. The handover asks for the shared fix.

**The pool's swing.** The surface's slope is the state of the U-tube column: it reaches the hydrostatic tilt through the column's own under-damped swing (ζ 0.35 from the tokens' `fluidSlosh`), and its ω₀ follows the effective gravity (√(2 g_eff ÷ l)). An upshift knocks it once.

## Deliberate changes

- **No ghost 8s behind the speed.** The brief asks for them; the family spec and the BoschDdu blind read (a ghost "888" read as real digits) say no. The family spec wins (order of authority 3).
- **The G vial shows the hydrostatic tilt, not gLat and gLong separately.** The brief maps gLong to the slug's length; a real vial's bubble does not change length with acceleration. The vial's bubble position is R_v × θ from the same tilt as the pool; longitudinal G is shown as a figure beside it.
- **Surface rumble and suspension ripples are left out.** The decoder does not expose rumble, and suspension travel is not verified against the golden packets (the 97% rule). The surfaces are moved only by verified acceleration and the gear change, so idle is still.
- **The limiter's drops come from a reservoir.** The column's length is the rpm and stays exact, so the drops that pinch off its tip (at 5 Hz, 1.89 × the capillary thread's radius) are drawn from the column's hidden reservoir and return to it when the pool drains after the shift. The mercury is kept in one ledger (`physics::Drips`), so none is made or lost.
- **Mercury, not chrome.** The thread's bands are the studio the metal mirrors, weighted by mercury's own reflectance at each angle across the thread (78% head-on, rising to 100% at grazing), never a flat grey.
