# SapphireMystery: research note

Build #35, catalog row 37, category Liquid Glass, Target `concept`, family Classic Analog, view cockpit. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read from the linked source on that date; **Computed** means worked out here from cited constants, with inputs and numbers given (`python3 -I` with numpy, the script's logic is restated in §4 so step 1 can re-run it); **Our reading** is a conclusion for this dash, open to the direction check. "Unconfirmed" marks something step 1 must check.

Telemetry is written for the 88-field decoder (main 60c99c9). The glass rule and `docs/specs/optics.md` bind every glass part; the kit's contact magnifier (`Profile::magnifier`, optics.md §4, main 1d456d2) is in place and §4.2 uses it.

## 1. The benchmark

**The benchmark is the Rotonde de Cartier Mysterious Hour** (calibre 9981 MC, 2013, and the same calibre in the Clé de Cartier Mysterious Hour of 2015): two hands that turn over an empty, see-through centre because each is carried by a clear, anti-reflective sapphire disc whose geared rim is hidden at the edge. It is the one object that is both a mystery dial and made of sapphire, which is exactly this dash's idea, so the visual review measures the floating lance and the invisible disc against it. Its ancestor, the Model A mystery clock, is the second reference for the look of hands floating in clear crystal.

| # | What makes it recognisable | Label and source |
|---|---|---|
| B1 | The hands "are mounted to transparent discs", which "appear totally clear when viewed straight-on", so the hands seem to float; the disc edges connect to gears hidden in the side or central supports, decorated "to draw the viewer's eye away from the mechanism". | **Cited**, [Sotheby's](https://www.sothebys.com/en/articles/the-enchanting-allure-of-cartiers-mystery-clocks) |
| B2 | 9981 MC: four sapphire discs "with an antireflective finish", each disc 0.56 g, its gear rim made in one piece with the sapphire by DRIE; the discs turn on fine pivots rather than in grooves; the disc module takes 58% of the movement's diameter "for record legibility"; the movement proper is a crescent beside it. | **Cited**, [WorldTempus 2013](https://en.worldtempus.com/article/watches/innovation-and-technology/cartier-to-keep-the-magic-alive-14218.html) |
| B3 | Clé version: "hands mounted on a pair of clear sapphire discs with teeth on their edge", the teeth "hidden underneath the dial". | **Cited**, [SJX 2015](https://watchesbysjx.com/2015/11/up-close-with-the-cle-de-cartier-mysterious-hour-review-photos-price.html) |
| B4 | Model A (1912 to 1913): diamond-set hands on two glass or rock-crystal discs, each in a toothed metal border; the borders are hidden in the hour circle; the movement sits in the base. "You can see straight through the clock." | **Cited**, [WorldTempus](https://en.worldtempus.com/article/watches/innovation-and-technology/cartier-to-keep-the-magic-alive-14218.html); [SJX 2025](https://watchesbysjx.com/2025/11/jp-morgan-cartier-mystery-clock-frodsham-phillips.html) |
| B5 | The cost of the trick on a wrist: "the illusion falls a bit flat when you get to see your own wrist through the mystery window". | **Cited**, [Worn & Wound](https://wornandwound.com/complications-the-mystery-dial/) |
| B6 | What the eye reads: nothing frames the hands. The only clues to the carrier are a faint edge and a tint where the coating catches light at an angle. | **Our reading** from B1, B2 and the AR-tint remark in [Gear Patrol](https://www.gearpatrol.com/watches/a35180867/watch-crystal-guide/); **unconfirmed** on photos: step 1 looks for the disc edge in photo 7 of §2 |

**Why the dash beats the watch at its own trick (Our reading).** B5 is the mystery watch's known weakness, and it is this dash's strength: what shows through the window is the live cockpit, not a wrist. Where Cartier hides the disc rim in an hour circle (B4), the dash has no ring at all, so its disc must be an optical object that is invisible face-on and shows only at its edge (§4.4).

**The others named in the brief, and what each lends:**
- **LeCoultre Galaxy (1940s):** "diamond markers act as the hour and minute hands and appear to float above the solid dial" (**Cited**, Worn & Wound). It is the precedent for floating dots as the indicator, so it backs the 13 floating lume dots and the floating indices. Later 1950s mystery dials (Longines, Zodiac Astrographic) carried dots and batons on discs (**Cited**, same).
- **Ulysse Nardin Freak (2001 on):** "no dial, no hands and no crown" (**Cited**, [Ulysse Nardin](https://www.ulysse-nardin.com/watches/freak/history)); the carousel "zips around the dial once every 60 minutes" and its bridge is the minute indicator, over an hour disc (**Cited**, [Gear Patrol](https://www.gearpatrol.com/watches/ulysse-nardin-freak-x-crystalium/)). It is the counter-example: the Freak's movement is the hand, while this dash hides everything but the hand. It lends one idea only, the hand with a counterweight tail read as one object turning about an invisible point.
- **Jaeger-LeCoultre:** its link is the LeCoultre Galaxy above. A JLC mystery history page was blocked by a bot check; **unconfirmed** beyond Worn & Wound.
- **The cabochon crown:** the Ballon Bleu's stone sits in "the winding crown, which appears to be floating within the round crown guard"; today's stone is synthetic spinel, earlier Cartier pieces used natural blue sapphire (**Cited**, [SJX 2018](https://watchesbysjx.com/2018/08/hands-on-with-the-cartier-ballon-bleu-42mm-automatic-in-steel.html)). A cabochon is a smooth unfaceted dome, typically 2 to 5 mm on a crown (**lead only**, [Skyjems](https://skyjems.ca/pages/encyclopedia-cabochon-crown), an unsigned page). So the dash's "sapphire cabochon" is a clear domed stone holding the gear, floating like the Ballon Bleu's, and the step-1 thread may give it a spinel-blue tint as an option (§5 customisation).

**A better target?** No car has a mystery dial. The Rotonde stays the benchmark.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | First Model A (1913), hands floating in rock crystal | https://watchesbysjx.com/wp-content/uploads/2025/10/cartier-model-a-mystery-clock-jp-morgan-phillips.jpg | [SJX 2025](https://watchesbysjx.com/2025/11/jp-morgan-cartier-mystery-clock-frodsham-phillips.html) |
| 2 | Same clock, second view | https://watchesbysjx.com/wp-content/uploads/2025/10/Phillips-Geneva-November-2025-JP-Morgan-JR-Cartier-Mystery-Clock-Model-A.jpg | same |
| 3 | Same clock, the movement in the base | https://watchesbysjx.com/wp-content/uploads/2025/10/Phillips-Geneva-November-2025-JP-Morgan-JR-Cartier-Mystery-Clock-Model-A-movement.jpg | same |
| 4 | A Model A, hands-on | https://watchesbysjx.com/wp-content/uploads/2016/03/ml_image.2570333.jpg | [SJX 2011](https://watchesbysjx.com/2011/12/the-cartier-model-a-mystery-clock.html) |
| 5 to 8 | Rotonde de Cartier Mysterious Hour Skeleton: front, second view, angled view, close view | https://www.ablogtowatch.com/wp-content/uploads/2017/09/Cartier-Rotonde-De-Cartier-Mysterious-Hour-Skeleton-aBlogtoWatch-01.jpg, `-04.jpg`, `-10.jpg`, `-18.jpg` (same folder) | [aBlogtoWatch](https://ablogtowatch.com/cartier-rotonde-de-cartier-mysterious-hour-skeleton-wrist-watch) |
| 9, 10 | Freak X Crystalium, close-up and lead | https://www.gearpatrol.com/wp-content/uploads/sites/2/2025/09/ulysse-nardin-freak-x-crystalium-close-up.webp, `...-lead.webp` | [Gear Patrol](https://www.gearpatrol.com/watches/ulysse-nardin-freak-x-crystalium/) |
| 11, 12 | Ballon Bleu 42 mm, the cabochon crown in its guard | https://watchesbysjx.com/wp-content/uploads/2018/08/Cartier-Ballon-Bleu-Blue-Dial-Automatic.jpg, `Cartier-Ballon-Bleu-automatic-blue-dial-1.jpg`, `Cartier-Ballon-Bleu-Blue-Dial-Automatic-3.jpg` | [SJX 2018](https://watchesbysjx.com/2018/08/hands-on-with-the-cartier-ballon-bleu-42mm-automatic-in-steel.html) |
| 13 | Strontium aluminate against zinc sulfide lume, lit and dark | lead figure, no direct URL | [Wikipedia, Super-LumiNova](https://en.wikipedia.org/wiki/Super-LumiNova) |
| 14 | Afterglow decay curves, SrAl2O4:Eu,Dy | Figure 1, no direct URL | [Van den Eeckhout et al. 2010](https://pmc.ncbi.nlm.nih.gov/articles/PMC5445854/) |
| 15 | Sapphire n_e against wavelength | page plot | [refractiveindex.info](https://refractiveindex.info/?shelf=main&book=Al2O3&page=Malitson-e) |

Drive `references` folder: https://drive.google.com/drive/folders/1Q7HeX3RhUd_F1lHteVk5udoIBu-GRnNS (holds `SOURCES.md`; no image bytes saved, the PC fetches the URLs above). The aBlogtoWatch and SJX image files have no captions, so step 1 confirms which shows what.

## 3. How the real thing behaves, with motion notes

A mystery dial has no start-up, limiter or gear change; its motion is a clock's. What it teaches is what moves and what stays still: the hands turn smoothly, the discs turn with them but cannot be seen turning (a rotationally symmetric clear disc looks the same at every angle), and nothing else moves (**Our reading** from B1 to B4). Each dash state below therefore takes its timing from the Classic Analog family and its light from the glass rule.

| State | What the dash does | Source |
|---|---|---|
| Start-up | The disc and cabochon materialise by growing thickness over 200 ms (never opacity); the hand does the family's first-start sweep, 350 ms up and 300 ms down, charging every index on the way so the dial is drawn once in lume and fades. | optics.md §2 "Materialise"; `families/classic-analog.md` (sweep); wake idea is the brief's |
| Idle | Hand on its spring; the two indices bracketing the tip stay lit; the wake has died; the crescent sits still at the rest light azimuth (−135°). Nothing else moves. | Brief; `common::VirtualLight` (optics.md §7) |
| Cruise | As idle; small rpm changes recharge the bracketing indices only. | Brief |
| Pull to the limiter | Hand climbs on the `needle` spring (T 0.07, ζ 0.75, feed-forward); a wake of at most 3 indices trails it (§4.5); the crescent drifts with lateral G. | Family spec; brief |
| Shift point and limiter | The 12 o'clock index charges in the shift colour; only the 13 floating dots strobe (10 Hz at 50% duty, 2 Hz breath in gentle mode); the disc rim lights steady in the shift colour (§7). | Catalog "Flash and telemetry"; brief |
| Gear change | The rpm drop sweeps the hand down and leaves a falling wake; the cabochon swaps the numeral (§7 for the moment itself). | Brief |
| Braking | Longitudinal G moves the G-dot and the index layers' parallax; nothing flashes. | Brief |
| Drift and lateral G | Light azimuth swings up to 25° at 1.5 g, so the crescent slides round the disc rim and the cabochon's highlight moves with it; index layers shift 0, 2, 4 px. | optics.md §7; brief |
| Reverse and neutral | Cabochon shows R or N; the hand stays live on rpm. | Our reading |
| Shut-down | The host hides the dash in one frame; no exit animation. | Liquid Glass family spec ("Hide: 1 frame") |

Real lume does not behave like the wake: it charges under light (excitation 200 to 450 nm) and glows for hours (**Cited**, Wikipedia; review §4.5). The 400 ms decay is a time-compressed design value, labelled MODEL on the dash.

## 4. The maths

### 4.1 Sapphire's two indices (Computed)
Sellmeier for the ordinary ray is optics.md [S1]; for the extraordinary ray, Malitson and Dodge 1972, B = 1.5039759, 0.55069141, 6.5927379 and C = 0.0740288², 0.1216529², 20.072248² µm² (**Cited**, [refractiveindex.info](https://refractiveindex.info/?shelf=main&book=Al2O3&page=Malitson-e), which itself gives n_e 1.7601 and R 0.075838 at 587.6 nm).

| λ (nm) | n_o | n_e | n_o − n_e |
|---|---|---|---|
| 587.56 (d) | 1.76817 | 1.76010 | 0.00808 |
| 611 (R channel) | 1.76694 | 1.75889 | 0.00805 |
| 549 (G channel) | 1.77051 | 1.76239 | 0.00812 |
| 464 (B channel) | 1.77775 | 1.76948 | 0.00827 |

R0 face-on: 0.07701 (o), 0.07584 (e). V_d: 72.31 (o), 72.99 (e). These match the brief's n_o 1.768, n_e 1.760 and optics.md's 1.7682 and 72.3.

### 4.2 The cabochon as a contact magnifier (Computed; checks the catalog row)
The row says "a hemisphere magnifies by n". At the centre, by the contact law m = n ÷ (n − (n − 1) t ÷ R) with t = R, it does: **m = 1.7682 (o) or 1.7601 (e)**. The row holds, but only paraxially. A trace of the kit's own model (orthographic view ray, Snell at the dome, straight to the plane, as `view_through` does) through the brief's Ø230 cabochon (R = 115, H = w = 115, b = 0, n_o):

| r | 0.05R | 0.2R | 0.4R | 0.6R | 0.8R | 0.9R | 0.95R |
|---|---|---|---|---|---|---|---|
| surface tilt | 2.8° | 11.5° | 23.5° | 36.8° | 53.0° | 64.0° | 71.6° |
| radial magnification dr/dx | 1.767 | 1.748 | 1.676 | 1.512 | 1.130 | 0.750 | 0.467 |

So a hemisphere shows its centre 1.77 times larger and its outer fifth squeezed below 1, and the family rule that numerals sit where the surface tilts under 22° (liquid-glass family spec, from the 44° dark cone) allows only r < 0.375R = 43 px. The brief's gear at cap 190 spans about ±95 px, out to 0.83R: under a hemisphere it would be bent badly and crossed by the rim's reflections. **Our reading:** keep the row's physics, but do not put the gear under a hemisphere. `Profile::magnifier(115, m, 1.7682)` gives the alternatives directly (Computed with the same trace):

| m | H (px) | tilt < 22° out to | dr/dx at centre, 0.5R, 0.8R |
|---|---|---|---|
| 1.10 | 52.6 | 0.662R = 76 px | 1.100, 1.097, 1.073 |
| 1.15 | 63.0 | 0.593R = 68 px | 1.150, 1.143, 1.096 |
| 1.25 | 78.0 | 0.512R = 59 px | 1.250, 1.230, 1.122 |
| 1.7682 (hemisphere) | 115.0 | 0.375R = 43 px | 1.768, 1.610, 1.130 |

**Recommended:** a ×1.15 oblate dome (H 63, b 0, k 2), the Liquid Glass pilot's own figure (optics.md §4 gives its gear dome 1.133). Draw the gear under it at cap 190 ÷ 1.15 ≈ 165 so it reads at cap 190, with the glyph's top and bottom near the 68 px flat-field limit; or keep the hemisphere for the lensing of the indices and the hand root and draw the gear after the glass (the pilot draws its readouts after the glass "so no rim bends them"). The board decides (§9). No dome here reaches ×2.75, so none folds (optics.md §4). The test to add: `Profile::magnifier(115, n, n)` is the hemisphere (H = w = 115, b = 0) and its traced centre magnification is n within 0.5%.

### 4.3 Birefringence is under a pixel (Computed)
Traced through the same hemisphere with n_e in place of n_o (the worst case, an a-cut stone seen across its axis), the two images differ by at most **0.20 px** inside 0.95R (0.199 px at 0.8R). For a c-cut stone (optical axis along the view, as optics-grade C-plane sapphire is cut, **Cited** [UQG Optics](https://www.uqgoptics.com/catalogue/optical-windows/sapphire/sapphire-windows-plates-cut-to-c-axis/)), the extraordinary ray inside travels at 10° to 39° from the axis, its index is 1/√(cos²θ ÷ n_o² + sin²θ ÷ n_e²) = 1.7679 to 1.7649, and the split is 0.004 to 0.070 px (ray walk-off neglected). Whether watch crystals are c-cut is **unconfirmed** (patent lead in §10). Either way the kit's "ordinary ray only" (optics.md §1) is right for this dash; the dash must not draw a double image.

### 4.4 The invisible disc (Computed)
- **Alpha.** The brief's "alpha 28" (disc) and "alpha 44" (cabochon) are flat tints, which the glass rule forbids. The kit's physics gives the disc's dimming instead: an uncoated flat sapphire face transmits 1 − 0.0770 = 0.923, which over the game is an encoded alpha of 1 − E(0.923) = 0.0346, about **9 of 255** (optics.md §8 alpha law). A single-layer MgF₂ quarter-wave coat at 550 nm reflects 0.13% and gives **0.15 of 255**, the "anti-reflective finish" of B2. Two uncoated faces of a real disc reflect 14.8%.
- **The face mirrors black.** The disc's flat top is tilted 0°, inside the 44° dark cone, so with the environment's far lights off it reflects nothing visible on OLED black (optics.md §7). It shows only by that small dimming and at its bevel.
- **The crescent.** With the key light at 40° elevation, the half vector stands 25° from the view axis, so the highlight sits where the bevel tilts 25°. For GGX the lobe falls to half at θ_h ≈ 0.644α (small-α form of `ggx_d`). Along the rim it spans azimuths where asin(sin 25° · sin Δφ) < 0.644α: **±5.2° at α = 0.06, a 47 px crescent on the R = 260 rim; ±13° at α = 0.15, about 120 px.** Across the rim, a k = 2 bevel of width w turns 1 ÷ (w cos θ) per px, so the lit band is about 0.07w wide: under a pixel for any bevel up to 14 px, and the kit's footprint filter (optics.md §7) spreads it to about 1 px. So "a 1 px crescent" follows from a thin disc (H = w of 2 to 4 px) and needs no painting. Computed with approximations (lobe half-width, no environment), so step 1 measures it on a render.
- **No light round the rim.** The kit's analytic studio and the pilot's map both have a horizon ring that draws "one thin line along the whole outline" (optics.md §7 table). This dash must set the environment's gain to zero (`OpticalGlass::env_gain`) or use an environment without the ring, so only the key light's crescent shows. That is what keeps it apart from `LiquidGlass`.

### 4.5 Lume (Computed and Cited)
- **The model** (the row): q ← q · e^(−dt ÷ τ), τ = 0.4 s, from `FrameInput` dt. Per frame: 0.95919 at 60 fps, 0.97938 at 120 fps. q falls to 0.5 in 277 ms, to 0.1 in 921 ms, to 0.02 in 1565 ms.
- **Catalog correction.** "Two half steps equal one whole step exactly" holds in real arithmetic, not in f32: over 100,000 random dt in 1 to 50 ms, exp(−dt ÷ τ) and exp(−dt ÷ 2τ)² were bit-equal only 24.6% of the time, up to 5 ulp apart, worst relative gap 3.3 × 10⁻⁷. The test must assert a relative tolerance of 1 × 10⁻⁶ (or work in f64). "q is 1 ÷ e after τ" holds: ten f32 steps of 40 ms give 0.3678798 against 0.3678794.
- **The wake length.** With 48 indices over the driven car's rpm range, the gap between indices in time is (max ÷ 48) ÷ rpm rate. Indices still at q ≥ 0.1 behind the hand:

| Max rpm | 1,000 rpm/s | 3,000 rpm/s | 8,000 rpm/s |
|---|---|---|---|
| 7,000 | 6 | 18 | 50 |
| 9,000 | 4 | 14 | 39 |
| 12,000 | 3 | 11 | 29 |

  So in a real pull the decay alone leaves a wake of 10 to 50 indices, and the brief's cap of 3 is what shapes it. With a hard cap the third index back sits at q 0.53 to 0.87 and the fourth vanishes: a step. **Our reading:** taper by rank instead, brightness = q · (1, 0.66, 0.33, 0) for the first, second, third and later indices behind the tip, so the wake ends softly at any rpm rate.
- **Real lume.** Super-LumiNova is strontium aluminate doped with europium and dysprosium; C3 glows at 515 nm and BGW9 at 485 nm; excitation 200 to 450 nm (**Cited**, Wikipedia). SrAl2O4:Eu,Dy glows at 520 nm for over 30 h, set by the slow release of trapped charge (**Cited**, Van den Eeckhout et al. 2010). The exact decay law (power law or a sum of exponentials) is **unconfirmed**; the dash's single exponential is a MODEL label either way.
- **Lume colour.** A Gaussian band of 80 nm FWHM (width **unconfirmed**) through the kit's colour-matching fit: 515 nm sits at xy (0.191, 0.506), outside sRGB, gamut-clipped to `#00FF93`; 485 nm at (0.132, 0.246), `#00C9FF`. The tokens' `colorRoles.lume.nightGreen` `#7DFF8A` and `nightBlue` `#5FD8FF` (`data/design_tokens.json`) are desaturated versions of exactly these hues, so the dash uses the token roles.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Floating lance angle | `rpm`, `max_rpm`, `shift_rpm` (`Signals`) | verified | Track-mode scale, shift rpm at 12 (`layout.classicDial.trackMode`); `needle` spring with feed-forward from `rpm_rate` |
| 48 index charges | `rpm` crossings, `FrameInput` dt | verified | Fixed `[f32; 48]`; set q = 1 when the tip passes, decay §4.5, rank taper |
| 13 floating dots | `rpm_fraction`, `shift_fraction` | verified | Thresholds per dot; strobe clock from the shift point |
| Cabochon numeral | `gear` | verified | R, N, 1 to 10, digits above 10 as is |
| Floating speed | `speed`, `unit` | verified | Tabular digits with the kit's `Hysteresis` |
| Crescent position and cabochon highlight | `g_lat` | verified | Azimuth −135° + 25° · clamp(g ÷ 1.5) (`common::VirtualLight`) |
| Index parallax 0, 2, 4 px | `g_lat`, `g_long` | verified | Offset ∝ felt G, clamped at 1.5 g |
| G-dot | `g_lat`, `g_long` | verified | `gBall` spring, ringless |
| Dimming pad | `OemDashOpaquePct` (fit core opacity) | config | alpha ≤ 140 × (1 − scrim) |
| Name zone | `custom_driver_name`, `show_logo` | config | Michroma, cap 12 |
| Materialise | `race_on` | verified | Thickness 0 → 1 over 200 ms |
| Option: torque-charged wake | `torque` | decoded, offset confirmed at check C | Charge q = torque ÷ peak torque learned live, so a pull paints the torque curve; fallback torque = power × 60 ÷ (2π rpm) |
| Option: lap-minute disc | `race_progress` (current_lap) | decoded, offset confirmed at check C | One lume dot on a second invisible disc, once round per 60 s; hidden when the field is 0 |

**What the game cannot send.** The benchmark's parts with no telemetry behind them:
1. **The time of day (the hour and minute discs).** Replacement, **recommended:** the lap-minute disc above, a second invisible disc carrying one lume dot that turns once a minute of the current lap, exactly as Cartier's minute disc turns once an hour; real data, hidden in free roam. Second: a session-time disc from the packet timestamp.
2. **Lume charged by daylight.** The game sends no ambient light level. Replacement, **recommended:** the hand is the light source, as the brief has it (the wake). Second: the torque-charged wake, where charge strength is real engine data.
3. **The crown as a control.** Replacement, **recommended:** the cabochon holds the gear. Second: none needed.

**Customisable without losing the look** (the owner, 10:55: "you can change for example needle color etc"):

| Part | Maps to | Default |
|---|---|---|
| Lance colour | palette `primary` (`primary_hex` in Custom mode) | family default `#FF7A1A` |
| Lume colour | `config.extra` `SapphireMysteryLume`: green, blue or primary | `colorRoles.lume.nightGreen` (C3) |
| Shift colour of the 12 index, dots and rim | palette `shift` (`shift_flash_hex`) | palette's own |
| Numeral and digit fonts | `cluster_font` (font roles `dial`, `display`) | Barlow Semi Condensed, Saira Condensed |
| Cabochon power | `config.extra` `SapphireMysteryLens`: 1.0 to 1.5 | 1.15 |
| Cabochon stone | `config.extra`: clear or spinel blue (Beer-Lambert tint, optics.md §6) | clear |
| Disc visibility | key-light irradiance and roughness (crescent length, §4.4) | α 0.06 |
| Wake time τ | `config.extra` `SapphireMysteryWakeMs`: 200 to 800 | 400 |
| Index style | `config.extra`: dots or batons | dots |

## 6. Edge cases

- **Max rpm 0 or tiny, idle near max:** hide the lance and wake until max rpm exceeds idle by 500 rpm; the 48 indices never divide by zero (guard the spacing).
- **12,000+ rpm and diesels at 4,500:** the scale follows max rpm; quarter numerals in thousands; the wake count changes (§4.5) but the rank taper keeps it at three.
- **EVs:** single gear, no idle: the cabochon shows 1 (or the game's value), the hand still reads rpm; shift dots use the kit's shift fraction.
- **Reverse, neutral, gear above 10:** R and N in the cabochon; above 10, two digits under the lens at the same cap (check the 68 px flat field).
- **mph and km/h, hp and kW:** speed follows `unit`; no power readout on this dash.
- **Limiter bounce (about 50 rpm a frame):** the lance follows on its spring; indices recharge only on a crossing with 1% hysteresis so the bracketing pair does not flicker; digits hold by `Hysteresis`.
- **Spikes, crashes, airborne:** G clamps at 1.5 g for light and parallax; the G-dot clamps at its edge; nothing flashes.
- **Stale or paused stream:** the app hides the dash; the charge array freezes with dt 0.
- **Palette extremes and random palettes:** the lume role is not the palette (unless the user picks it), so the wake stays readable; the lance must keep 3:1 against black; red is never a lume colour.
- **Font swap:** the cabochon's flat field (68 px at ×1.15) bounds the numeral; wide fonts shrink to fit.
- **Cockpit only:** over a bright windscreen at 0% scrim, the dimming pad (≤ 140) and a black core under the cabochon keep gear ≥ 7:1 and speed ≥ 4.5:1 (brief's blur gate).
- **Reduced flash:** dots breathe at 2 Hz; the rim's limiter light fades in over 150 ms instead of stepping.
- **Colour-only cues and colour vision:** the shift moment is also position (the 12 index) and the dot count, not hue alone.

## 7. Signature moments

**Limiter.** Only the 13 floating dots strobe (10 Hz, 50% duty; 2 Hz breath in gentle mode).
- **A (recommended): the disc shows itself by light, not paint.** At the limiter the environment's gain rises from 0 to a horizon-ring-only studio tinted the shift colour, so the rim, tilted 40.5° to 45° where it mirrors elevation 0° to 9° (optics.md §7), lights as one thin steady line round the whole Ø520 disc. It is the brief's "only moment the whole crystal can be seen", done with the kit's physics. Needs the environment in §8.
- **B:** a steady shift-colour ring drawn under the disc's edge, which the bevel refracts inwards by b√(n² − 1) (optics.md §4). Cheaper, but it reads as a lit ring under glass rather than the glass itself.

**Gear change.**
- **A (recommended): the cabochon refocuses.** The numeral swaps while the cabochon's power eases 1.15 → 1.0 → 1.15 over 180 ms on the `needle` spring, so the new gear swells into focus through the stone. Physical (only the dome height changes) and calm.
- **B: the falling wake.** No extra effect: the rpm drop already pulls the lance down and leaves a phosphor trail coming down the dial; the 12 index gives back its shift charge.
- Not chosen: a ripple through the dome, which is `LiquidGlass`'s upshift and would blur the siblings.

## 8. What it needs from the 4K library

The pre-build (PR #18, `PREBUILD.md` and `kits.rs` on `origin/claude/project-thread-ib67x9`) made **no kit** for this dash: "SapphireMystery's cabochon is optical glass, which the kit computes from the outline". No `assets/clusters/sapphire_mystery/prebuild/manifest.json` exists.
- **Already there:** `assets/glass/blue_noise_128.png` covers the brief's 64×64 dither tile; MSDF fonts for Saira, Saira Condensed, Barlow Semi Condensed and Michroma are in `assets/fonts/`; the lume roles are tokens.
- **Reusable from other kits:** PaganiHorologerie's `polished_steel_matcap` (PR #18) for the lance's metal edge and counterweight; the `liquid_glass` studio plate is **not** reusable as is, because its horizon ring breaks the "no light round the rims" rule.
- **Not needed:** the brief's "sapphire-dome normal/refraction pair" and "soft caustic crescent texture" (the kit computes both, and a painted crescent breaks the glass rule); the legacy `assets/glass/lens_circle_*` maps.

LIBRARY GAP: horizon-ring-only studio environment, octahedral, 2048² runtime and 4096² master, ring at elevation 0° to 9° only, everything else black; for the limiter rim light (§7 A); computed on the CPU in `tools/asset-forge/src/prebuild/` like the liquid-metal studio.
LIBRARY GAP: thin-film coating on `OpticalGlass` (optics.md §9 has the equation and `optics::thin_film_reflectance` exists, but the paint has no coating field), so an anti-reflective disc can be drawn per channel; no size; kit code (route d), scheduled by the coordinator. Without it the disc is uncoated and dims the game by about 9 of 255, which may be acceptable (§9).
LIBRARY GAP: optional applied-lume index sprite, 24 px at 1:1 (48 px master), a domed Super-LumiNova plot in its day colour `#E3EBC4` with a lit pass, so unlit indices read as physical plots over the cockpit; Blender.

## 9. Departures and open questions for the board

1. **No flat alpha.** The brief's alpha 28 and 44 become physics (§4.4): disc uncoated about 9/255 or coated, cabochon by Beer-Lambert if tinted. Reason: the glass rule.
2. **No painted crescent or rim.** The crescent is the key light on a 2 to 4 px bevel; the limiter rim is the horizon-ring environment (§7 A). Reason: glass rule, and distance from `LiquidGlass`.
3. **Cabochon power.** ×1.15 oblate dome with the gear under it at cap ≈ 165, or a hemisphere (×1.768) with the gear after the glass (§4.2). The board picks; the catalog row's "hemisphere" stays true for the hemisphere option only.
4. **What the cabochon lenses.** The brief says it lenses "the indices behind it", but at Ø230 in the centre of a Ø520 disc no index lies behind it unless the index rings pass inside r 115. Options: the lance's root and counterweight pass under the stone (the stone then hides the pivot by magnifying it, the mystery trick), or the inner parallax layer of indices is set inside the cabochon's radius.
5. **Wake shape.** Rank taper (1, 0.66, 0.33) instead of a hard cap of 3 (§4.5).
6. **Lume time.** τ 400 ms is a MODEL, real lume glows for hours (§4.5).
7. **Family spec drift.** `families/classic-analog.md` still lists "fresnel to 40/255, crescent 18/255, AR rim `#7B6CFF`"; optics.md §11 replaced these (the MgF₂ coat colour is `#9E51FF`). The lead should get the family spec fixed.
8. **Sweep and index count.** The brief fixes 48 indices and the shift rpm at 12 but not the sweep angle; the board sets it (the family default is 270°).

## 10. Sources

**Mystery dials**
- Sotheby's: https://www.sothebys.com/en/articles/the-enchanting-allure-of-cartiers-mystery-clocks
- WorldTempus 2013: https://en.worldtempus.com/article/watches/innovation-and-technology/cartier-to-keep-the-magic-alive-14218.html
- Worn & Wound 2018: https://wornandwound.com/complications-the-mystery-dial/
- SJX, Clé Mysterious Hour: https://watchesbysjx.com/2015/11/up-close-with-the-cle-de-cartier-mysterious-hour-review-photos-price.html
- SJX, first Model A: https://watchesbysjx.com/2025/11/jp-morgan-cartier-mystery-clock-frodsham-phillips.html
- SJX, Model A hands-on: https://watchesbysjx.com/2011/12/the-cartier-model-a-mystery-clock.html
- aBlogtoWatch, Rotonde Mysterious Hour Skeleton: https://ablogtowatch.com/cartier-rotonde-de-cartier-mysterious-hour-skeleton-wrist-watch
- Ulysse Nardin, Freak history: https://www.ulysse-nardin.com/watches/freak/history
- Gear Patrol, Freak X Crystalium: https://www.gearpatrol.com/watches/ulysse-nardin-freak-x-crystalium/
- Unread (bot check): https://www.watchprosite.com/digest/jaeger-lecoultre-mystery-galaxy-dials-history-guide/

**Crystals and cabochons**
- Gear Patrol, watch crystal guide: https://www.gearpatrol.com/watches/a35180867/watch-crystal-guide/
- SJX, Ballon Bleu 42 mm: https://watchesbysjx.com/2018/08/hands-on-with-the-cartier-ballon-bleu-42mm-automatic-in-steel.html
- Skyjems (lead only): https://skyjems.ca/pages/encyclopedia-cabochon-crown
- UQG Optics, C-cut sapphire: https://www.uqgoptics.com/catalogue/optical-windows/sapphire/sapphire-windows-plates-cut-to-c-axis/
- Unread lead, US patent 11054790: https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11054790

**Physics**
- Malitson and Dodge 1972, sapphire e-ray, via refractiveindex.info: https://refractiveindex.info/?shelf=main&book=Al2O3&page=Malitson-e
- `docs/specs/optics.md` §1, §2, §4, §7, §8, §9 and its sources [S1], [S5], [S7], [S9], [S28], [S35]
- Wikipedia, Super-LumiNova: https://en.wikipedia.org/wiki/Super-LumiNova
- K. Van den Eeckhout, P. F. Smet and D. Poelman, "Persistent Luminescence in Eu2+-Doped Compounds: A Review", Materials 3, 2536 (2010): https://pmc.ncbi.nlm.nih.gov/articles/PMC5445854/
