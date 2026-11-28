# AuroraVeil: research note

Build #32, catalog row 45, category Psychedelic, Target `concept`, family Sci-Fi (Design Council 3C, FUI dialect), view cockpit. A polar-night curtain whose 64 rays are the tach, rising from a 1 px field arc over the gear. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read on 2026-11-28 in the source linked; **Computed** means worked out here with `python3 -I` (numpy, scipy) from cited constants or from the brief's own numbers, with inputs and results given so the thread can re-run them; **Our reading** is a conclusion drawn for this dash, open to the direction check. Anything the dash draws that is modelled rather than exact is marked MODEL.

## 1. The benchmark

**One benchmark: a rayed auroral curtain (a discrete arc, "rayed band" in the observers' code) seen from the ground in the auroral zone, as in the US Air Force photograph "Aurora borealis over Eielson Air Force Base, Alaska" (ref. 1).** All-sky time-lapse and keograms (refs 8, 9) are the motion reference for the same object. What makes it recognisable:

| # | Feature | Source |
|---|---|---|
| B1 | **Pickets in a fence.** Rays stand in the arc "like pickets in a fence", lined up with the magnetic field, which is nearly vertical; seen overhead they are tight spirals "only a kilometer or two across". | **Cited**, Davis, Alaska Science Forum, [gi.alaska.edu](https://www.gi.alaska.edu/alaska-science-forum/auroral-rays) |
| B2 | **A sharp lower edge and a soft top.** Few oxygen atoms below about 100 km give the "abrupt-looking end of the lower edges of the curtains"; thinning oxygen makes the tops faint. | **Cited**, [Wikipedia, Aurora](https://en.wikipedia.org/wiki/Aurora), "Colours and wavelengths" |
| B3 | **Green body, red crown.** Green 557.7 nm between about 100 and 150 km; red 630.0 nm from 150 km up to about 250 km, rarely beyond 600 km, because below 150 km the red state is lost to collisions. | **Cited**, Atmospheric Optics, [atoptics.co.uk](https://www.atoptics.co.uk/blog/glowing-gases-aurorae/) |
| B4 | **Folds.** "The similarity to curtains is often enhanced by folds within the arcs"; arcs can break up into rapidly changing rayed features. | **Cited**, Wikipedia, "Forms" |
| B5 | **Five ground forms**, faintest to most visible: glow, patches, arcs, rays, coronas (rays that "diverge from one point"). Shape letters A arc, B band, C corona, D drapery, R rays; structure H homogeneous, R rayed, P pulsating. | **Cited**, Wikipedia, "Forms"; Bureau of Meteorology, [sws.bom.gov.au](https://www.sws.bom.gov.au/Educational/1/1/8) |
| B6 | **Black sky between.** Stars show "undiminished through the glow"; IBC I (1 kR) is as bright as the Milky Way, IBC IV (1000 kR) as the full moon. | **Cited**, Wikipedia, "Forms"; BoM IBC table |

**Our reading:** the benchmark is mostly black. Its light is a thin, near-vertical comb with a hard bottom edge and a long fading top, which suits an OLED and the brief's "the sky stays black". The visual review asks: does the render show a picket-fence comb, a sharp lower border, a fade upward and a fold, and nothing else lit? The brief's other references (Lucid Gravity's black glass, BMW Panoramic Vision's dark band) are restraint references for the housing, not the benchmark. No better real target was found: no production car shows an aurora figure.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | Benchmark: green rayed curtain over a lake, the lower border sharp, rays fading upward, a fold at left (public domain, USAF) | https://upload.wikimedia.org/wikipedia/commons/d/d3/Aurora_borealis_over_Eielson_Air_Force_Base%2C_Alaska.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Aurora_borealis_over_Eielson_Air_Force_Base,_Alaska.jpg) |
| 2 | Eight forms side by side, Iceland 2014, two exposure rows (30 to 20 s and 4 s): the naked-eye look versus the camera look | https://upload.wikimedia.org/wikipedia/commons/2/29/Aurora_shapes.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Aurora_shapes.jpg) |
| 3 | Rare strong red crown over green, Tromsø 2011 (B3) | https://upload.wikimedia.org/wikipedia/commons/1/1a/Red_and_green_auroras.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Red_and_green_auroras.jpg) |
| 4 | Corona: looking straight up into the divergence point, magenta, G5 storm, Sweden 2024 | https://upload.wikimedia.org/wikipedia/commons/e/e3/Magenta_G5_aurora_over_Tuntorp%2C_Lysekil_Municipality_11.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Magenta_G5_aurora_over_Tuntorp,_Lysekil_Municipality_11.jpg) |
| 5 | Corona overhead, Iowa, 10 May 2024 | https://upload.wikimedia.org/wikipedia/commons/4/4f/Auroral_Corona_Onawa_IA.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Auroral_Corona_Onawa_IA.jpg) |
| 6 | Purple aurora, Alaska (Geophysical Institute, CC BY-SA 4.0): the violet and pink end of the palette | https://upload.wikimedia.org/wikipedia/commons/1/12/Aurora2.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Aurora2.jpg) |
| 7 | Seen from the ISS, August 2017: the layered curtain edge-on, green below, red above | https://upload.wikimedia.org/wikipedia/commons/5/52/Aurora_australis_ISS.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Aurora_australis_ISS.jpg) |
| 8 | Keogram explainer: one all-sky night (2 s frames, Dawson City 2021) reduced to a time-by-latitude strip | https://upload.wikimedia.org/wikipedia/commons/d/d0/Keogram_explainer.gif | [Commons](https://commons.wikimedia.org/wiki/File:Keogram_explainer.gif) |
| 9 | Time-lapse of a curtain, northern Norway (video, 19 s) | https://upload.wikimedia.org/wikipedia/commons/4/4f/Aurora_timelapse.ogv | [Commons](https://commons.wikimedia.org/wiki/File:Aurora_timelapse.ogv) |
| 10 | Blue-through-red rays from southern England, 10 May 2024 | https://upload.wikimedia.org/wikipedia/commons/f/fa/AuroraBorealisOkeford20240510-01.jpg | [Commons](https://commons.wikimedia.org/wiki/File:AuroraBorealisOkeford20240510-01.jpg) |
| 11 | Green arcs over Vadsø, Norway (public domain) | https://upload.wikimedia.org/wikipedia/commons/f/f9/Polarlicht2012.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Polarlicht2012.jpg) |
| 12 | Magnetosphere schematic, the field lines that guide the rays (NASA, public domain) | https://upload.wikimedia.org/wikipedia/commons/b/bb/Structure_of_the_magnetosphere_LanguageSwitch.svg | [Commons](https://commons.wikimedia.org/wiki/File:Structure_of_the_magnetosphere_LanguageSwitch.svg) |

Drive `references` folder (with `SOURCES.md`): https://drive.google.com/drive/folders/16YxqscXV85I3EK_4TQLsj_0w-jdhLsNZ (`G:\My Drive\FH6 WheelDash v2 - 4K Masters\aurora_veil\references\`). No image bytes were saved here; the PC fetches the listed URLs. **Our reading on photos:** long exposures (ref. 2, top row) smear rays into smooth bands and saturate the colour; the 4 s row is nearer what the eye and the dash should show.

## 3. How the real thing behaves, with motion notes

### 3.1 The physics of time

| # | Fact | Source |
|---|---|---|
| T1 | 557.7 and 630.0 nm are forbidden transitions of atomic oxygen; their lifetimes, 0.7 s and 107 s, set "the graduality ... of flaring and fading". Other sources give 120 s and 110 s for the red line. | **Cited**, Wikipedia, "Colours"; Sandahl et al. 2008 §2 ([pdf](https://amisr.com/publications/media/pub-pdfs/2008_Sandahl_10.1016j.jastp.2008.08.016.pdf)); atoptics |
| T2 | The green lifetime "will obscure fast temporal variations" in TV frames. Nitrogen transitions are "much faster" (prompt). | **Cited**, Sandahl 2008; Wikipedia |
| T3 | Curls: typical size 1 to 2 km, preferred spacing 3 km, lifetime 0.25 to 0.75 s (Trondsen and Cogger 1998); Hallinan (1976): spacing about 8 km, lifetimes of a few seconds, drift 10 to 20 km/s. Rays can cross the sky at "one hundred times" jet speed. | **Cited**, Sandahl 2008 §3; gi.alaska.edu |
| T4 | Hallinan and Davis (1970) attributed curls to a Kelvin–Helmholtz instability driven by E×B drift shear; Trondsen and Cogger found arc width against wavelength consistent with it; Vogt et al. (1999) found growth faster than the shear alone explains, which an Alfvén wave could supply. | **Cited**, Sandahl 2008 §3 |
| T5 | Flickering aurora: typically 3 to 20 Hz. | **Cited**, Sandahl 2008 (Oguti 1981; Sakanoi and Fukunishi 2004) |
| T6 | Pulsating aurora: periods typically 2 to 20 s, with peak heights about 8 km lower. Measured at 3.3 Hz, patch on and off times are "erratic" and "not regularly periodic". | **Cited**, Wikipedia, "Changes with time"; Humberset et al. 2016, [arXiv 1612.00592](https://arxiv.org/abs/1612.00592) |
| T7 | Alfvén waves travelling Earthward accelerate the auroral electrons; their Poynting flux can power about a third of global auroral light, up to half before midnight. Lab measurements showed the resonant energy transfer. | **Cited**, Schroeder et al., Nat. Commun. 12, 3103 (2021), [PMC8184961](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8184961/) |
| T8 | A field line oscillating with a period of about six minutes made the aurora brighten and dim at the same pace (field-line resonance, THEMIS). | **Cited**, [NASA, 2016](https://www.nasa.gov/missions/themis/nasas-themis-sees-auroras-move-to-the-rhythm-of-earths-magnetic-field) |
| T9 | Substorm: quiet arcs turn active "in a few minutes" and recover over 1 to 3 hours; over a night forms run glow to corona and fade in reverse order. | **Cited**, Wikipedia, "Changes with time" |
| T10 | The ~3 Hz modulation inside pulsating patches named in this run's brief: **unconfirmed** (the sources read do not give it). Step 1 checks a pulsating-aurora review. | unconfirmed |

### 3.2 Every state of the dash, from the physics

| State | What the real aurora offers | Motion note for the dash (Our reading) |
|---|---|---|
| Start-up | A night runs glow, arc, rays, corona (T9) | The arc draws first as a 1 px line from the centre out (≤ 1 s, the family's sector boot), then the rays grow from both ends to the live rpm. Gear and speed live from frame 1. Skippable. |
| Idle | A quiet homogeneous arc | Short, still rays at the idle rpm; no drift, no pulsation (idle is still). Pulsating aurora (T6) is erratic, so it is never used as an idle animation. |
| Cruise | Steady rayed band | Ray heights hold; only distance moves the fold drift (brief). |
| Pull to limiter | Brightening and expansion before breakup | Heights grow from the ends inward; boost raises brightness on a log (IBC-like) scale. |
| Limiter | Breakup, corona | §7. Only the strip strobes. |
| Gear change | Energy arriving down the field line (T7) | §7. The reading drops at once; the glow decays with the real 0.74 s green lifetime. |
| Braking | none specific | Longitudinal G compresses the curtain on the fluidSlosh spring (brief). |
| Drift, lateral G | Shear makes curls and folds (T3, T4) | Lean up to 12 deg; slip seeds Kelvin–Helmholtz curls along the base (§4.4). |
| Reverse, neutral | none | §6. |
| Shut-down, stale stream | Fade in reverse order (T9) | The host hides the dash; nothing to draw. |

## 4. The maths and physics

### 4.1 Catalog row, checked

Row: q(y) = q_max · exp(1 − y − e^(−y)); tan I = 2 tan λ; colours from 557.7, 630.0 and 427.8 nm through `spectrum_to_linear_srgb`. Tests: the profile peaks at 1 for y = 0 and integrates to e; the dip is 76.88° at 65°.

- **Computed: both test values hold.** ∫ q dy = 2.718281828 = e (scipy quad over −20 to 60; exactly, with u = e^(−y) the integral is e ∫ e^(−u) du = e). tan I = 2 tan 65° = 4.289, I = 76.876°.
- **The form is the Chapman production function** for one wavelength absorbed in an isothermal exponential atmosphere, at zero zenith angle, with y the height above the peak in scale heights (Our reading from the standard form; step 1 cites a textbook for it, for example chapter 9 of the AFRL Handbook of Geophysics 1985, [ngdc.noaa.gov](https://www.ngdc.noaa.gov/stp/space-weather/online-publications/miscellaneous/afrl_publications/handbook_1985/Chptr09.pdf), not read in this run). Auroral electrons deposit energy with a different profile (Rees 1963; the Fang et al. 2010 parameterisation, [UCAR](https://data.ucar.edu/sq/dataset/electron-impact-ionization-a-new-parameterization-for-100-ev-to-1-mev-electrons)). So the ray profile is labelled MODEL. Its shape is right for the benchmark: a super-exponential fall below the peak (the sharp lower border, B2) and an e^(−y) tail above (the soft top). y must increase **upward**; flipped, the curtain has a soft bottom and a hard top, which no aurora shows.
- **Correction 1 (the catalog row needs fixing):** `spectrum_to_linear_srgb` cannot carry line spectra. It sums the colour matching functions on a 5 nm grid from 380 to 780 nm and balances the result for a reflectance under white light (`crates/fh6-kit/src/optics.rs`, line 302). 557.7 and 427.8 nm fall between samples (nearest 560 and 430), so a narrow line returns black; 630.0 hits a sample only by chance. The dash evaluates `cie_xyz(λ)` at each line, converts with `xyz_to_linear_srgb`, then maps into the sRGB gamut (§4.3). Suggested row text: "Colour of each emission line from `cie_xyz` at its wavelength, mapped into sRGB at constant OKLab hue."
- **Correction 2 (physics, not a number):** the blue 427.8 nm line is not a lower band. A statistical study of 57,907 simultaneous height pairs found green and blue both peak near 114 km (median 114.0 and 115.0 km; mean 114.84 and 116.55 km), "contrary to a common misconception that blue peaks below green"; blue is the more likely to peak high (above 130 km). **Cited**, Whiter et al., Ann. Geophys. 41, 1 (2023), [angeo](https://angeo.copernicus.org/articles/41/1/2023/angeo-41-1-2023.html). The red-violet lower border of intense displays comes from excited molecular nitrogen (**Cited**, atoptics; Wikipedia says blue and purple at the lower edges at the highest activity). So the violet belongs mixed through the green body, with a pink-violet fringe only at the bottom of bright rays.

### 4.2 Values the unit tests can assert (Computed)

| Quantity | Value |
|---|---|
| q(0), ∫q dy | 1, e = 2.718282 |
| q(−2), q(−1), q(−0.5), q(0.5), q(1), q(2), q(3), q(4) | 0.0124, 0.4876, 0.8618, 0.8989, 0.6922, 0.3213, 0.1288, 0.0489 |
| Half maximum at | y = −0.9852 and +1.4612 (FWHM 2.4464 scale heights) |
| 10% and 1% points | −1.5871 / +3.2644 and −2.0332 / +5.6015 |
| Light below height y (closed form) | exp(−e^(−y)), a Gumbel CDF: 0.3679 below the peak; 30% of the light lies below y = −0.1856, 90% below +2.2504 |
| Dip I and lean from vertical, 90° − I | 60°: 73.898° / 16.102°; 65°: 76.876° / 13.124°; **67°: 78.017° / 11.983°**; 70°: 79.686° / 10.314° |

- **Our reading: the lean.** The auroral zone is centred on 67° magnetic latitude (**Cited**, Wikipedia, "Characterisation"), where the field is 11.98° from vertical. That is the brief's "rays lean up to 12 deg" to 0.02°: the brief's maximum lean is the real field-line tilt at the centre of the auroral zone. Recommend latitude 67° as the model constant and the 12° cap as its consequence.
- **Our reading: why the rays still look vertical.** Arcs run east–west and the field tilts in the north–south plane, so a curtain seen broadside (as in ref. 1 and on the dash) shows its rays nearly upright; the tilt shows when looking along the arc. So rest is upright, and lateral G "turns the viewer" toward the arc's end, revealing up to the 12° tilt. This keeps "settles perfectly still" honest.
- **Ray profile on screen (Computed, proposal).** Put the arc at the lower 10% point (y = −1.587) and the reading edge (the ray top) at the upper 10% point (y = +3.264): 4.851 scale heights. At a 160 px ray the scale height is 32.98 px, the peak sits 52.3 px above the arc (32.7% of the ray, close to the brief's "lower 30% emissive") and the reading edge is a crisp palette-coloured tip at 160 px. The scale height scales with the rpm fraction, so the shape never changes, only its size.
- **Spring (Computed from `anim.rs`, period = undamped).** fluidSlosh T 0.62 s, ζ 0.35: ω = 10.134 rad/s, damped period 0.662 s, overshoot 30.9%, envelope τ 0.282 s, 2% settling 1.13 s (1.08 s simulated). Lean 12° over 160 px moves the tip 34.0 px.
- **Arc (Computed).** 1300 px span, 100 px sagitta: radius 2162.5 px, half-angle 17.49°, arc length 1320.4 px; 64 rays at 20.96 px pitch along the arc. If rays follow arc normals instead of the vertical, the end rays splay 17.5° outward: a corona seen from far off, an option for §7.

### 4.3 The emission colours (Computed with the kit's own `cie_xyz` fit)

| Line | CIE xy | Linear sRGB at Y = 1 (out of gamut) | Most saturated sRGB at the same OKLab hue | OKLCH hue | Nearest reserved token |
|---|---|---|---|---|---|
| O 557.7 nm green | 0.3603, 0.6369 | 0.294, 1.328, −0.168 | `#9CFF00` (L 0.90) | 132.1° | `go` `#2BE37A` at 151.9° |
| O 630.0 nm red | 0.7134, 0.2866 | 6.528, −0.536, −0.065 | `#FF0034` (L 0.63) | 23.4° | **`warning` `#FF2D3A` at 24.8°** |
| N₂⁺ 427.8 nm violet | 0.1647, 0.0061 | 18.2, −18.7, 145.3 | `#7600FF` (L 0.52) | 290.6° | **default `shift` `#7C5CFF` at 286.2°** |

- **The eye's weights** (ȳ at the line): 0.9955, 0.2637 and 0.009. For equal radiance the violet line is a hundred times dimmer than the green: physically the dash's violet is a faint tint, never a band.
- **Our reading: 557.7 nm is yellow-green**, not the emerald of camera photographs; the eye sees `#9CFF00`, cameras record bluer.
- **Conflict for the board:** the red line sits 1.4° of hue from the fixed `warning` red, which TOKENS.md keeps for redline, brake and alarms ("red is never structural", a palette primary within 15° of warning is rejected). The violet line sits 4.4° from the default `shift` colour, which the brief gives the ray tips at the shift rpm. §9 sets out the options.

### 4.4 Kelvin–Helmholtz curls (Computed)

The inviscid Rayleigh equation for the shear layer U(y) = U₀ tanh(y/δ), solved by finite differences (600 points on ±12δ): the fastest-growing wave has kδ = 0.445, growth rate 0.1897 U₀/δ, wavelength **14.12 δ**. Against T3, a 3 km curl spacing gives δ = 0.21 km and 8 km gives 0.57 km, the same order as the thin-arc widths Sandahl reviews (0.5 to 1.5 km gap, about 900 m Alfvénic arcs; **Cited**). For the dash: the base line's thickness sets δ, curls appear at 14.1 δ spacing, and they grow at 0.19 U/δ with U from the slip (§5). The thread re-checks kδ = 0.4446 and 0.1897 against Michalke (1964), whose values these match from memory only.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Ray reading height (tip) | `rpm`, `idle_rpm`, `max_rpm` | verified | f = (rpm − idle) ÷ (max − idle), clamped; 32 ray pairs light from both ends inward, each pair 3.125% of f; the tip is exact and never springs |
| Ray glow body | rpm via the tip | MODEL | Chapman profile scaled to the tip (§4.2); afterglow e^(−t/0.74 s) on fall only |
| Shift closing at the centre, tip colour | `shift_rpm` (Signals) | verified | steady at f ≥ shift fraction |
| Strip (16 pods) | `rpm`, `shift_rpm` | verified | family stages; limiter strobe (§7) |
| Gear, speed | `gear`, `speed` and its unit | verified | Saira 600 cap 200, Saira 300 cap 96 |
| Curtain shear and lift | `g_lat`, `g_long` (felt G = −acceleration) | verified | fluidSlosh; lean = 12° × clamp(g_lat ÷ 1.2 g) (the 1.2 g scale is a proposal) |
| Brightness | `boost` (psi) | verified | log scale, alpha ≤ 64 (brief); a car without boost uses `throttle` (verified) |
| Lower-border sharpness and violet fringe | `throttle` or `power` | verified | MODEL: harder precipitation reaches lower (the most energetic electrons reach about 80 km, atoptics; soft electrons below about 4 keV peak above 110 km, Whiter 2023) |
| KH curls along the base | body slip β = atan2(v_x, v_z) | derived from verified velocity | amplitude ∝ growth 0.19 U/δ with U from \|β\|, on a spring; zero below 3° slip |
| Fold drift | `distance_traveled` | decoded, offset confirmed at check C | fallback: integrate `speed_ms` × dt (exact enough); parked = still |
| Heading tape | `yaw` (orientation) | decoded, offset confirmed at check C | ticks every 15°; fallback: hide the tape (there is no verified heading) |
| Horizon line (§5.1) | `roll` | decoded, offset confirmed at check C | fallback: static line |
| Airborne | `g_vert` (acceleration_y), suspension | g_vert verified; suspension decoded, check C | §6 |

### 5.1 What the game cannot send

The benchmark is a natural light, so its "dropped parts" are the space weather and the scene around it. None of it may be faked.

| Real part | Replacement ideas (driven by telemetry) |
|---|---|
| Solar wind speed, IMF Bz, the Kp index | **(recommended) The car is the solar wind:** felt G drives shear and lift (brief) and power drives the "characteristic energy" (MODEL); no Kp readout is drawn. Second: a 0 to 9 ladder labelled RPM, never Kp. |
| The keogram scientists read the night by (ref. 8) | **(recommended) A drive keogram:** a thin rank-4 ribbon under speed and gear where each column is the curtain's height profile one second ago, scrolling left: the last 60 s of real rpm as an aurora scientist would plot it. No glow (rank 4). |
| Magnetic latitude and dip | **(recommended) A fixed model constant, 67°**, giving the 12° lean cap (§4.2). Second: tie latitude to car class (`car_class`, decoded, check C) so a faster class leans more: MODEL, weaker. |
| The landscape and lake under the arc (ref. 1) | **(recommended) A 1 px horizon silhouette** at ≤ 20% luminance that tilts with roll (check C) and is static until then: TJX's "subtle horizon" idea. Second: a faint reflection of the curtain in a "lake" band under the strip, mirrored live, alpha ≤ 16. |
| Stars | **(recommended) The `star_field_tile`** of the `Gt3RsApogee` kit at ≤ 8% luminance, static, masked to the sky above the arc. Second: none (pure OLED black). |
| Pulsating and flickering aurora (T5, T6) | Not reproduced: flicker is 3 to 20 Hz and pulsation is erratic, both forbidden as area motion. **(recommended)** The 10 Hz strip strobe is the only flicker, and it lies inside the natural 3 to 20 Hz band. |
| Navigation (heading is not navigation), media, phone, range, temperatures, warnings | Dropped: the benchmark has none and the brief needs none. Fuel is decoded (check C) but has no physical role in an aurora; leave it out. |

## 6. Edge cases

| Case | What the dash does |
|---|---|
| max_rpm 0 or below idle + 200 | f = 0; rays at the idle height; no shift closing; no division by zero (guard the span). |
| Idle near max | The 32 pairs still span idle to max; at least one pair lit whenever the engine runs. |
| 12,000+ rpm or diesel 4,500 | f is a fraction, so the curtain looks the same; the strip and shift fraction follow `shift_rpm`. |
| EV (single gear, odd rpm) | Same mapping; if max_rpm is implausible (≥ 25,000) the reading uses speed ÷ the learned top speed (MODEL), labelled. |
| Reverse, neutral | Glyph R or N; reverse mirrors the fold drift direction (distance runs back); neutral at idle is the still arc. |
| Gear above 10 | Glyph shows the number in fixed cells; the `[f32; 10]` table clamps. |
| mph and km/h, hp and kW | From `Signals.unit` and `power_unit`; no figure depends on the unit. |
| Limiter bounce ±50 rpm a frame | The tips follow exactly; the shift closing and tip colour use 2% hysteresis so the centre does not chatter. |
| Huge G (crash) | Lean clamps at 12°, lift at ±25%; the spring overshoot (31%) is clamped too. |
| Airborne | g_vert near 0: lift relaxes to rest; no special effect. |
| Stale or paused stream | The host hides the dash; springs freeze. |
| Palette extremes, random palettes | Tip and strip follow palette roles; the emission layer (if chosen, §9) is fixed and does not recolour. Test a black-ish primary: the tip keeps a 1 px rim at alpha ≥ 60. |
| User font swap | Cap-height sizing; the curtain does not depend on the font. |
| Reduced flash | Strip strobe becomes the 2 Hz breath; afterglow and curls stay (they are slow). |
| Colour-only cues, colour vision | Every stage is also a position: the closing at the centre and the pods. Red/green deuteranopia merges the green body and red crown, so the crown is never information. |

## 7. Signature moments

**Limiter and shift.** Only the 16-pod strip strobes: 10 Hz at 50% duty, a 2 Hz breath in reduced-flash mode.
1. **(recommended) Breakup.** At the shift rpm the last pair closes the curtain at the centre and every tip turns the palette `shift` colour, steady (brief). The strip's 10 Hz sits inside the real flicker band (T5), so the strobe reads as the aurora flickering at its edge. Nothing else flashes.
2. **Corona.** On the limiter the ray directions ease (fluidSlosh) from vertical to the arc normals, so the comb splays 17.5° at its ends toward a convergence point below the gear: the divergence point of refs 4 and 5. Steady; it relaxes when the rpm falls.
3. **Red crown.** Held on the limiter, a faint upper crown fills in with a slow time constant, like 630 nm with its 107 s lifetime scaled down (MODEL). Weakest: it needs a red that clashes with `warning` (§4.3).

**Gear change.**
1. **(recommended) Alfvén pulse.** An upshift sends one brightening down every ray from tip to base in about 120 ms (energy arriving down the field line, T7), then the green afterglow fades with the real 0.74 s lifetime while the tips drop instantly with the rpm. The brief's 1 px fold then travels along the base from the centre outward. Downshift: the fold travels inward and the pulse runs upward.
2. **Curl train.** The gear change seeds a Kelvin–Helmholtz wave at 14.1 δ spacing that rolls outward along the base and decays on the spring (§4.4).
3. **Afterglow only.** Just the 0.74 s decay of the old curtain behind the new, lower tips: the quietest option. It leaves 86.7%, 49.0% and 24.0% of the glow at 0.1, 0.5 and 1.0 s (Computed).

## 8. What it needs from the 4K library

- **This dash's kit:** none. `kits.rs` lists AuroraVeil among the dashes with nothing layout-independent to make, and PREBUILD.md says it "draws its figure from the model". No `assets/clusters/aurora_veil/prebuild/manifest.json` exists on the PR #18 branch.
- **Shared parts to reuse:** `star_field_tile` (ComfyUI, seed 200003, kit `Gt3RsApogee`) for the optional faint sky; `mat::piano_black()` in the hero library for a black-glass binnacle face; the Saira and Chakra Petch fonts are in `assets/fonts/`.
- **The bespoke rule** (one ComfyUI and one Blender asset, visibly used) is not met by the library today.

LIBRARY GAP: ComfyUI `polar_night_airglow_plate`, a 3840 px wide sky plate of a faint airglow band low over a black horizon with sparse stars, ≤ 8% luminance, seamless left to right; the dash's bespoke ComfyUI asset behind the curtain (never the figure); asset-forge prebuild, ComfyUI lane.
LIBRARY GAP: Blender swatch `black_glass_gloss`, a 4096² seamless tile of deep black glass with a faint polished sheen and the normal and roughness passes; the floating black-glass face of the brief's Lucid Gravity reference and the dash's bespoke Blender asset; asset-forge prebuild, Blender lane.
LIBRARY GAP: computed `aurora_line_colours`, a 16-bit linear table of the three line colours (§4.3) and a 256 × 1 ramp along the Chapman height from violet-tinted green to red, made exactly on the CPU from `cie_xyz`; layout-independent and shared with any dash using the colour law's emission clause; asset-forge prebuild, computed lane.

## 9. Departures and open questions for the board

1. **Which layer is the figure.** The colour law allows the emission colours "on the rank-4 figure, and only there", but the brief makes the curtain the rank-3 tach and the heading tape rank 4. Recommended: the tips (the reading) stay on palette roles; the glow body below them is declared the rank-4 figure and may take the emission colours. The handover names this.
2. **Emission colours or palette.** Recommended default: green body (`#9CFF00` mapped, or the palette ramp per the law) with the violet only as a tint; the red crown off, because 630.0 nm is 1.4° from `warning`. Offer `config.extra.aurora_colour = "palette" | "emission"`, default "emission" without the crown.
3. **The tip colour at shift.** The default `shift` violet is 4.4° from the 427.8 nm violet. If the body carries violet, the shift tip needs its higher luminance and the closing position to stay distinct.
4. **The heading tape.** Yaw is now decoded (check C). Recommended: draw it once check C confirms the offset, hidden until then, as the category note says.
5. **Catalog row text** (`spectrum_to_linear_srgb` for lines, §4.1): the lead reports the fix.
6. **Rays vertical or along the arc normals.** Recommended vertical at rest (broadside view, §4.2) and normals only at the corona moment.
7. **Lean scale.** 12° at 1.2 g is a proposal; check against the felt-G range of the test cars.
8. **Customisable without losing the physics:** palette or emission colours (above), curtain brightness (alpha cap 64 and lower), star field on or off, horizon on or off, keogram on or off, ray count (48 or 64), all as `config.extra` keys; font through the user font role. Defaults: emission body, stars off, horizon on, keogram on, 64 rays.

## 10. Sources

**Physics and observation**
- Whiter, Partamies, Gustavsson, Kauristie, "The altitude of green OI 557.7 nm and blue N2+ 427.8 nm aurora", Ann. Geophys. 41, 1–12 (2023): https://angeo.copernicus.org/articles/41/1/2023/angeo-41-1-2023.html
- Sandahl, Sergienko, Brändström, "Fine structure of optical aurora", JASTP 70, 2275–2292 (2008): https://amisr.com/publications/media/pub-pdfs/2008_Sandahl_10.1016j.jastp.2008.08.016.pdf
- Humberset et al., "Temporal characteristics and energy deposition of pulsating auroral patches", JGR (2016): https://arxiv.org/abs/1612.00592
- Schroeder et al., "Laboratory measurements of the physics of auroral electron acceleration by Alfvén waves", Nat. Commun. 12, 3103 (2021): https://www.ncbi.nlm.nih.gov/pmc/articles/PMC8184961/
- NASA, "THEMIS sees auroras move to the rhythm of Earth's magnetic field" (2016): https://www.nasa.gov/missions/themis/nasas-themis-sees-auroras-move-to-the-rhythm-of-earths-magnetic-field
- Davis, "Auroral rays", Alaska Science Forum (1981): https://www.gi.alaska.edu/alaska-science-forum/auroral-rays
- Atmospheric Optics, "Glowing gases: aurorae": https://www.atoptics.co.uk/blog/glowing-gases-aurorae/
- Wikipedia, "Aurora" (sections Characterisation, Altitude, Forms, Colours, Changes with time): https://en.wikipedia.org/wiki/Aurora
- Bureau of Meteorology, "Characterising the aurora" (forms code, IBC): https://www.sws.bom.gov.au/Educational/1/1/8
- IAGA Resolution 4 (1973), brightness reporting: https://www.iaga-aiga.org/index.php?id=res4-73

**Leads not read in this run (step 1)**
- AFRL Handbook of Geophysics (1985), ch. 9 (Chapman function): https://www.ngdc.noaa.gov/stp/space-weather/online-publications/miscellaneous/afrl_publications/handbook_1985/Chptr09.pdf
- Fang et al. 2010, electron impact ionisation parameterisation: https://data.ucar.edu/sq/dataset/electron-impact-ionization-a-new-parameterization-for-100-ev-to-1-mev-electrons
- Baranoski et al., simulating auroral dynamics (graphics): http://cuca.cs.uwaterloo.ca/resources/docs/aurora-dynamics.pdf
- Mesoscale auroral curls, GRL 2024: https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2023GL107889

**Images:** the twelve Wikimedia Commons pages in §2.

**Repository:** `program/phases/07-cluster-factory/CATALOG.md` (brief, category note, colour law, maths row); `docs/design/TOKENS.md` (warning, shift, go, strobe, budget); `docs/design/families/sci-fi.md`; `crates/fh6-kit/src/optics.rs` (`cie_xyz`, `spectrum_to_linear_srgb`); `crates/fh6-kit/src/anim.rs` and `tokens.rs` (fluidSlosh); `docs/TELEMETRY_USAGE_MATRIX.md`; PR #18 branch `PREBUILD.md` and `tools/asset-forge/src/prebuild/kits.rs`.
