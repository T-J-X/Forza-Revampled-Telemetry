# VernierLoupe: research note

Build #17, catalog row 31, category Liquid Glass, Target `concept`, family Liquid Glass, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read from the linked source on that date; **Computed** means worked out here from cited constants, with the inputs and the numbers given (scripts in §4, `python3 -I` with numpy); **Our reading** is a conclusion for this dash, open to the direction check. "Unconfirmed" marks a lead that step 1 must open and check: the shared web-fetch budget ran out during this pass, so several pages were found by search but not read.

Telemetry updated 2026-11-28 for the 88-field decoder (main 60c99c9).

## 1. The benchmark

**The benchmark is the Rolex Cyclops lens**: a small, curved sapphire lens on the watch crystal at 3 o'clock, whose only job is to make one reading, the date, bigger than everything around it. It is the most widely known case of a piece of clear glass that *is* the instrument, which is the Liquid Glass category's line, and it is what the visual review measures the loupe against.

| # | What makes it recognisable | Label and source |
|---|---|---|
| B1 | A "curved lens set on the crystal at 3 o'clock, above the window", introduced in 1953 "to ease the legibility of the date", in "virtually scratchproof sapphire crystal" with a "double anti-reflective coating". | **Cited**, [rolex.com, Cyclops lens](https://rolex.com/watchmaking/features/cases/cyclops-lens) |
| B2 | Magnification 2.5×: "Rolex mentioned for a long time a 2.5x magnification on their website but this is no longer the case". First moulded in one piece with a Plexiglas crystal, later a separate sapphire piece glued on; anti-reflective coated since 2005. | **Cited**, [Monochrome Watches](https://monochrome-watches.com/the-rolex-cyclops-lens/) |
| B3 | A "raised bubble shape" on a flat crystal; the main complaint is that the bubble breaks the line of the watch. | **Cited**, [Swiss Watch Expo](https://www.swisswatchexpo.com/TheWatchClub/2020/05/01/rolex-date-and-cyclops-lens/); Monochrome Watches |
| B4 | What the eye reads: the same print appears at two sizes at once, small on the dial and 2.5× under the lens, with a clean step at the lens edge. The rest of the dial stays at 1×, which is what proves the lens is working. | **Our reading** from the sources above; to be confirmed on photos at step 1 |
| B5 | Outline of the Cyclops (round, oval or rounded rectangle) and its size in mm. | **Unconfirmed**: no page read gives them. Step 1 measures them on a press photo (leads: the Rolex page's images, the Monochrome close-ups in §2). |

**Why not the others named in the brief.** All three are kept as references for one part each, not as the benchmark:
- **The vernier** (caliper and sextant) is the reference for the rail. "N graduations of the indicating scale cover N − 1 graduations of the data scale", least count S/n; invented in 1631 by Pierre Vernier, also called a nonius after Pedro Nunes; the eye reads it by "vernier acuity", the ability to see which lines align, which "far exceeds the eye's optical resolution" (**Cited**, [Wikipedia, Vernier scale](https://en.wikipedia.org/wiki/Vernier_scale)). Under ideal conditions vernier acuity is about 3 arcseconds, against foveal cones about 30 arcseconds apart (**Cited**, [Wolfe et al., Sensation & Perception 4e, essay 3.1](https://learninglink.oup.com/public/files/cdn/wolfe4e/essay03.01.html)). A sextant's index arm carries its vernier and a reading magnifier along the arc (**unconfirmed**: leads in the [ION museum](https://www.ion.org/museum/Sextant-with-beveled-scale.cfm) and [Royal Museums Greenwich](https://www.rmg.co.uk/collections/objects/rmgc-object-43427)).
- **The contact magnifier** is the optical model. A dome magnifier is a plano-convex lens whose flat face lies on the page, giving 1.8× to 6× (**Cited**, [Wikipedia, Dome magnifier](https://en.wikipedia.org/wiki/Dome_magnifier)); the reading stone, "an approximately hemispherical lens that can be placed over text", in regular use from about 1000 AD (**Cited**, [Wikipedia, Reading stone](https://en.wikipedia.org/wiki/Reading_stone)). That is exactly the kit's contact model (optics.md §4), so the loupe is a reading stone that slides.
- **The jeweller's loupe** sets the vocabulary of power only: MP = 250 / f (f in mm); "as power increases, lens diameter and field of view decrease"; simple lenses "distort color on the outer fringes" (**Cited**, [Edmund Optics](https://www.edmundoptics.com/knowledge-center/application-notes/microscopy/magnifying-lenses-how-to-choose-a-magnifier/)). A loupe is held away from the work, so its maths is not ours.
- **A sliding magnifier on a scale** has real precedents: slide-rule cursor magnifiers ("Some magnifiers are placed directly on top of the cursor", to get "that fourth digit"; **Cited**, Koppany, [J. Oughtred Society 19.2](https://www.oughtred.org/jos/articles/V19.2_p28-31.pdf)) and rule magnifiers that slide along a steel rule (**unconfirmed**, [Lee Valley, Veritas rule magnifier](https://www.leevalley.com/en-us/shop/tools/hand-tools/marking-and-measuring/marking-accessories/45035-veritas-rule-magnifier)). The brief's iOS text loupe precedent: Apple removed it in iOS 13 and brought a smaller one back in iOS 15 (**Cited**, [MacRumors](https://www.macrumors.com/2021/06/09/ios-15-tidbits-loupe-magnifier-dial-picker-more/)).

**A better target?** No real car has a loupe tachometer. The Cyclops stays the benchmark; the step-1 thread may prefer the slide-rule magnifying cursor if its photos show a lens riding a scale more clearly.

## 2. Reference photos

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 1 | Rolex press image of the Cyclops on the crystal | https://media.rolex.com/q_auto/f_auto/rolexcom/watchmaking/features/watchmaking-cards/case/cyclops-lens/watchmaking-features-case-cyclop-lens_rvb-share.jpg | no (fetch failed) |
| 2 | Rolex Cyclops film poster frame | https://media.rolex.com/image/upload/q_auto:eco/f_auto/c_limit,w_1920/v1764321749/rolexcom/watchmaking/features/watchmaking-cards/case/cyclops-lens/video/watchmaking-features-cyclop-lens-film-posterframe | no |
| 3 | Cyclops hero photo, Monochrome Watches | https://monochrome-watches.com/app/uploads/2018/09/Rolex-Cyclops.jpg | no (fetch failed) |
| 4 | Cyclops close-up over the date | https://monochrome-watches.com/wp-content/uploads/2018/09/Rolex-Cyclops-1.jpg | no (budget out) |
| 5 | Cyclops detail 2 | https://monochrome-watches.com/app/uploads/2018/09/Rolex-Cyclops-2.jpg | no (fetch failed) |
| 6 | Cyclops detail 3 | https://monochrome-watches.com/app/uploads/2018/09/Rolex-Cyclops-3.jpg | no |
| 7 | Date and Cyclops header image | https://www.swisswatchexpo.com/thewatchclub/wp-content/uploads/2020/05/rolex-date.png | no |
| 8 | Vernier caliper diagram (Wikimedia) | https://en.wikipedia.org/wiki/File:Vernier_caliper.svg | no |
| 9 | Animated vernier reading | https://en.wikipedia.org/wiki/File:VernierscaleHow_a_vernier_scale_works.gif | no |
| 10 | Reading a caliper, animated | https://en.wikipedia.org/wiki/File:Using_the_caliper_new_en.gif | no |
| 11 | Sextant with bevelled scale, museum page | https://www.ion.org/museum/Sextant-with-beveled-scale.cfm | no |
| 12 | Small craft precision sextant, RMG | https://www.rmg.co.uk/collections/objects/rmgc-object-43427 | no |
| 13 | Rule magnifier sliding on a steel rule | https://www.leevalley.com/en-us/shop/tools/hand-tools/marking-and-measuring/marking-accessories/45035-veritas-rule-magnifier | no |

Drive `references` folder: https://drive.google.com/drive/folders/1RWwj7QNbNgKEKx4L15Cw2PAI_vahGkdZ (holds `SOURCES.md`; no image bytes came back this pass, so step 1 saves the photos, starting with 1, 4 and 8).

## 3. How the real thing behaves, with motion notes

A Cyclops does not move; a vernier moves only with its jaw. So every state below is the dash's own, grounded in how a lens over a scale behaves.

| State | What moves | What stays still | Notes |
|---|---|---|---|
| Start-up | Each lens materialises over 200 ms by growing H and b from 0 (optics.md §2 "Materialise"), so magnification grows in from ×1 to its value; the loupe glides from 0 to idle on the ribbon spring | the rail and its numerals | Family rule: never ramp opacity |
| Idle | nothing but rpm noise; the loupe sits at idle | everything else | Idle digits held by hysteresis |
| Cruise | the loupe tracks rpm (ribbon spring T 0.07 s, ζ 0.9, feed-forward; `data/design_tokens.json` `motion.springs.ribbon`) | the rail; the 4 px fill moves with rpm | Coarse reading = the fill; fine reading = the loupe |
| Pull to the limiter | loupe slides right; the 13 pods light in turn; in the last 300 rpm before the shift point the loupe's base thickens (swell, §7) | numerals | |
| Limiter / shift | only the pod strip strobes (10 Hz, 50% duty; 2 Hz breath in reduced flash); the loupe rim holds a steady ring in `shift` | the loupe position is held by hysteresis | 50 rpm a frame bounce, §6 |
| Gear change | the loupe whips back (§7), the gear lens ripples once at its rim, the numeral holds still | the rail | |
| Braking | loupe falls back on the same spring; the G-lens moves forward on the lattice | | |
| Drift / lateral G | the light azimuth swings up to 25° at 1.5 g (optics.md §7), so every rim line travels together; the G-lens moves sideways | | |
| Reverse / neutral | loupe parks at idle; gear lens shows R or N | | |
| Shut-down | the host hides the dash in one frame | | Stale stream hides the dash |

**Computed, the whip.** Inside a lens of magnification m centred at c, a scale point x is seen at c + m (x − c). With x fixed and c moving at v, the image moves at (1 − m) v: **the magnified ticks run backwards through the lens at 1.4 to 1.5 times the loupe's speed**. An upshift from 7,500 to 5,200 rpm on an 8,000 rpm scale (1200 px rail) moves the loupe 345 px; over 0.1 s that is 57.5 px a frame at 60 Hz and the image inside runs 80 px a frame the other way (0.2 s: 28.8 and 40 px). Magnified ticks are 36 to 37.5 px apart, so the scale inside the lens genuinely washes out while the loupe moves fast and comes back sharp when the spring settles (Our reading; the frame count depends on the real shift time, which step 1 measures in the demo). The brief calls the smear "radial"; the motion is along the rail, so the smear is linear along x, of length |1 − m(r)| v Δt, and shorter near the rim where m is lower (Our reading).

## 4. The maths

**The catalog row** (`## Real geometry and maths`): a contact magnifier (truncated ball lens), m = n ÷ (n − (n − 1) t ÷ R); least count = main division ÷ N; for n = 1.7682, t ÷ R = 1 gives 1.7682 and the aplanatic point gives n² = 3.1265; ten divisions over nine of 100 rpm read to 10 rpm.

**Check of the law (Computed).** For one spherical surface, object in glass at depth t below the vertex, image in air: n/t + 1/s' = (n − 1)/R, so s' = tR / ((n − 1)t − nR) and m = −n s'/t = n / (n − (n − 1) t/R). t = R gives n; t = R(1 + 1/n), the aplanatic point (object R/n beyond the centre; the R/n and nR conjugates are **Cited**, [GeoGebra, Aplanatic lens](https://geogebra.org/m/jHzzXcXW)), gives n². The image stays virtual while t/R < n/(n − 1) = 2.3018. **The row checks**: n_d = 1.7682 (Sellmeier of optics.md [S1], recomputed), n² = 3.1265, least count 100 ÷ 10 = 10 rpm. No correction to the row.

**It is already in the kit.** A truncated ball lens seen from above is a hemisphere of radius R over a base: `Profile { thickness: R, bevel: R, exponent: 2, base: t − R }` on a circle of radius R, and `view_through` traces it exactly (Snell at the top, straight to the content plane). The existing test `a_hemisphere_on_a_page_magnifies_by_its_index` covers only base 0.

| Centre magnification | t/R | base b/R | Round loupe Ø110: base, total t | Note |
|---|---|---|---|---|
| 1.7682 | 1.0000 | 0 | 0, 55.0 px | hemisphere, existing test |
| 2.40 (brief) | 1.3427 | 0.3427 | 18.8, 73.8 px | |
| **2.50 (Cyclops)** | 1.3811 | 0.3811 | 21.0, 76.0 px | most even field, below |
| 3.1265 (n², aplanatic) | 1.5656 | 0.5656 | 31.1, 86.1 px | |
| 3.20 (brief's swell) | 1.5825 | 0.5825 | 32.0, 87.0 px | past the aplanatic point |

**Edge distortion (Computed**, by tracing the kit's orthographic contact model at 20,000 to 200,000 radii; radial magnification = dr/dx):
- At ×2.4 the radial magnification is 2.40 at the centre, 2.39 at 0.6R, 2.35 at 0.7R, 2.24 at 0.8R and 1.87 at 0.9R: slight barrel at the edge, no fold. The rim shows the plane 0.495R from the centre (b√(n² − 1) = 0.500R, the optics.md rim rule, checks).
- **At ×2.5 the field is the most even of any centre magnification**: within 2.3% of the centre value out to 0.8R (×2.4: 6.9%; ×2.3: 13%). The Cyclops's published power is also the flattest view through a sapphire contact dome.
- At n² = 3.1265 the field is strongly pincushioned (radial 3.32 at 0.375R, 4.45 at 0.7R, 6.2 at 0.8R) and **folds at 0.903R**: beyond it the lens shows the same ticks again, mirrored. At ×3.2 the fold is at 0.889R; at ×2.8 at 0.984R (hidden in the rim); at ×2.6 there is none. The aplanatic point is free of spherical aberration for a point imaged through a cone of rays; under the kit's orthographic viewer what matters is distortion, and that is worst there.
- **Field of view.** A round loupe of Ø110 at ×2.4 shows ±27.4 px of the rail (Ø140: ±34.9; Ø170: ±42.4). The 22° clean zone of the family rule is r < R sin 22° = 0.375R: ±20.6 px on screen for Ø110, which shows only ±8.6 px of rail.

**The brief's 170×110 outline cannot magnify the ticks (Computed from optics.md §2).** The kit builds the surface from the distance to the outline. Inside a 170×110 capsule the distance depends only on y along the 60 px straight middle, so the surface there is a cylinder lying along the rail: it magnifies y by up to ×2.4 and x by exactly 1. The ticks would grow taller but not further apart. A rounded rectangle or an ellipse has the same ridge. Only a circle gives the same magnification in x and y; a cylinder standing across the rail (the long axis vertical) would magnify x only, like a rod magnifier.

**Oblate domes and the gear lens (Computed).** For k = 2 and H < w the cross-section is an ellipse whose apex radius is w²/H, so m = n / (n − (n − 1)(H + b) H / w²). The pilot's gear dome (w 160, H 72, b 24) gives 1.133, which matches its measured "about 1.15" (`liquid_glass/HANDOVER.md`). For the brief's gear (cap 200) the 22° rule needs tan θ = (H/w)(r/w)/√(1 − (r/w)²) ≤ tan 22° at the glyph edge: a lens of **diameter** 230 (w = 115) must have H ≤ 14 to 26 px and magnifies about 1.02×, while a lens of **radius** 230 allows H up to 171 to 193 px.

**Vernier geometry (Computed**; main division s = rail px × 100 ÷ max rpm):

| max rpm | s cockpit / hood | vernier span 9s | 10 rpm offset s/10 | same at ×2.4 |
|---|---|---|---|---|
| 4,500 | 26.7 / 33.3 px | 240 / 300 px | 2.67 / 3.33 px | 6.4 px |
| 8,000 | 15.0 / 18.8 px | 135 / 169 px | 1.50 / 1.88 px | 3.6 px |
| 12,000 | 10.0 / 12.5 px | 90 / 113 px | 1.00 / 1.25 px | 2.4 px |
| 20,000 | 6.0 / 7.5 px | 54 / 68 px | 0.60 / 0.75 px | 1.4 px |

On the LG C1 a 4K pixel is 0.277 mm (48"), 0.317 mm (55") or 0.375 mm (65") wide; at an assumed 0.9 m from the eye one pixel subtends 63 to 86 arcseconds, 20 to 30 times the 3-arcsecond vernier acuity. So a 1 px misalignment is readable even unmagnified (viewing distance is an assumption; step 1 takes it from the binnacle layout). **But a full 10-over-9 vernier never fits inside the loupe**: its span (54 to 300 px at 1×, 130 to 720 px magnified) is larger than what a Ø110 to Ø170 loupe shows (55 to 85 px of rail). See §9.

**Unit-test values** (no GPU): m(1) = 1.7682; m(1 + 1/n) = 3.1265 = n²; m(1.3811) = 2.500; the traced ×2.5 loupe within 2.5% of centre to 0.8R; the ×3.1265 trace folds between 0.90R and 0.91R; the pilot apex check 1.133; the least count 10 rpm; image velocity (1 − m) v.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Rail ticks and numerals | `max_rpm` | verified | 100 rpm ticks over 1200 px (hood 1500); rebuilt into the mip-mapped target only when max rpm changes |
| 4 px fill | `rpm` | verified | x = rail · rpm / max rpm, ribbon spring |
| Loupe position | `rpm`, `rpm_rate` | verified, derived | ribbon spring with feed-forward |
| Loupe swell | `shift_rpm` | derived (max rpm × shift %) | base b grows over the last 300 rpm (§7) |
| Coincident vernier line | `rpm` | verified | lit line index = floor(rpm mod 100 / 10), held by hysteresis |
| 13 pods | `rpm`, `shift_rpm` | verified, derived | stages by `shift_fraction`; strip strobes at the limiter |
| Operating band tint | `power` (packet `power_watts`, through `FrameInput.telemetry`), `rpm`, `torque` | power and rpm verified; `torque` decoded, offset confirmed at check C; peaks LEARNED | peak power and peak torque learned live into rpm bins, torque from the packet's `torque` with P · 60 / 2π rpm as the fallback; tint shows only after a wide-open pull |
| Gear lens | `gear` | verified | R, N, 1 to 10 |
| Speed | `speed`, unit | verified | tabular, ghost 8s |
| G-lens position | `g_lat`, `g_long` (acceleration x, z) | verified | px per g on the lattice, `gBall` spring |
| G-lens magnification | \|g\| | verified | base b from 0 (×1.77) to 0.343R (×2.4) at 2 g: lattice pitch 24 → 42 to 58 px (Computed) |
| G-lens rim state | `tyre_combined_slip` (brief: tyre combined slip) | decoded, offset confirmed at check C | the largest of the four wheels' combined slip, steps neutral, caution, warning at board-set values (MODEL thresholds until check C reads the field's scale in a slide; the page states none); fallback body slip β = atan2(v_x, v_z) (`slip_deg`, derived) at board-set angles |
| Light azimuth on all rims | `g_lat` | verified | −135° + 25° · clamp(g/1.5) (optics.md §7) |

**Brief corrections.** "Tyre combined slip" is now decoded (offset confirmed at check C), so the brief's "native fields only" holds: the rim reads the packet's combined slip, and body slip from local velocity is only the fallback. "Dots under it are magnified, so load reads as dot size" only works if the G-lens's thickness follows |g|; a lens of fixed thickness magnifies the same everywhere, so position alone would carry load. The brief's optional "shared sapphire dome normal map" must not be used: the kit computes the normal from the outline, and a baked normal map inside glass breaks the glass rule.

**What the game cannot send** (the benchmark's and the references' parts):
- **The date** (the Cyclops's reading). Replacement ideas: **(recommended) the Cyclops reads the gear**, so the fixed gear lens is the Cyclops of this dash, a raised bubble whose numeral steps up from the rail's scale; or the loupe carries a small learned "best shift" tick at the rpm where the next gear's power, learned live, crosses this one's.
- **A vernier's second scale on a moving jaw** has no input of its own. **(Recommended) the vernier plate rides with the loupe and its coincident line gives the tens of rpm**, a real reading from verified rpm; or a ghost loupe that stays where the last upshift happened (from rpm and gear), so the driver sees how close to the shift point they changed.
- **Warnings** (none in this brief or benchmark, so nothing to drop). The space below the rail is filled by the gear, speed and the G-lattice already.

**Decoded now, not drawn** (moved out of the list above on 2026-11-28): fuel, tyre temperatures, tyre slip and the lap and race data are all in the 88 fields (each "decoded, offset confirmed at check C"). Neither the brief nor the benchmark has a place for fuel, tyre temperatures or lap times, so nothing is added for them; tyre combined slip now drives the G-lens rim as the brief asked (table above), and `torque` feeds the operating band.
- **Porsche 550 Spyder operating band** and **Oblivion's dot lattice (GMUNK)**: unconfirmed this pass; step 1 finds a photo of the 550 tachometer's band and a still of the lattice before either is drawn.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| max rpm 0 or tiny | Signals clamp it to at least 1; below 1,000 rpm draw the rail with no ticks and the loupe parked at 0, no division by zero in s |
| Idle near max | the fill and loupe start near the right; ticks still at 100 rpm; pods stage from `shift_fraction` |
| 12,000+ rpm | s = 10 px at 12,000 and 6 px at 20,000: minor ticks every 200 rpm below 8 px spacing, so 2 px ticks never touch (mip-mapped target prevents shimmer) |
| Diesel 4,500 | s = 26.7 px; the loupe shows 2 divisions; vernier plate 240 px wide |
| EV (single gear, odd rpm; `cylinders` 0, decoded, offset confirmed at check C) | gear lens shows 1 steadily; rail from max rpm as sent; vernier still works; whip only fires on a gear change, so an EV never whips |
| Reverse, neutral | loupe parks at idle, no whip; R and N in the gear lens |
| Gear above 10 | `Gear::from_raw` maps it to N; nothing else |
| mph and km/h, hp and kW | speed digits follow `unit`; the learned band uses watts internally |
| Limiter bounce (about 50 rpm a frame) | loupe position and the coincident line held by `fh6_kit::Hysteresis` (the lit line would jump five places a frame otherwise) |
| Spikes, crashes, airborne | G-lens position clamped to the lattice edge; magnification clamped at ×2.4; airborne (g near 0) the G-lens returns to centre, and its rim goes neutral, because unloaded wheels spin up and their combined slip spikes without meaning a slide |
| Tyre combined slip 0 or missing | all four at exactly 0 while the car moves above 5 km/h for 1 s counts as not sent: the rim uses body slip β; at a standstill both are 0 and the rim is neutral; slip steps use hysteresis so a reading on a threshold does not flicker |
| Torque 0, negative or odd | the band learns only from frames with throttle above 0.9 and torque above 0; when `torque` reads 0 while power is above 0, the derived P · 60 / 2π rpm is used; the band never needs the unit, since only the rpm of the peak matters || Stale or paused stream | the host hides the dash |
| Palette extremes, random palettes | lensed ticks in `primary`, swell ring in `shift`; the rail stays at ≤ 20% luminance whatever the palette |
| Font swap | rail numerals and speed follow the user's font; the 22° clean zone is checked against the swapped glyph's bounds |
| Cockpit and hood | hood rail 1500 px (s grows 25%), halo alpha ≤ 0.55; the black core under the loupe keeps magnified ticks readable over a bright bonnet |
| Reduced flash | strip breathes at 2 Hz; no whip smear, no rim glint, no swell pulse |
| Colour-only cues, colour vision | the coincident line is brighter and longer, not only primary; the G-rim steps also change rim width |

## 7. Signature moments

**Limiter / shift.** (a) **Recommended:** only the 13-pod strip strobes (10 Hz, 50% duty, clock from the shift point); the loupe holds a steady `shift` ring and its base has swollen to the aplanatic n² = ×3.13, so the last ticks grow and the image folds at 0.90R into a thin mirrored band under the rim: a real optical effect, not a painted ring. (b) Swell only to ×2.8, where the fold sits at 0.98R inside the rim reflection, if the fold reads as a fault on the board. (c) The brief's ×3.2 is past the aplanatic point and folds at 0.89R; there is no physical reason for 3.2, so it is not recommended.

**Gear change.** (a) **Recommended:** the whip of §3, with the smear as the exposure of the moving image, length |1 − m(r)| v Δt along the rail, sharp again when the spring settles, then one glint round the rim from the key light. (b) The Cyclops step: the gear lens's base drops to zero and grows back over 150 ms, so the new numeral "zooms in" from ×1 to its magnification (materialise, optics.md §2). (c) The vernier plate snaps its coincident line to the new rpm in one frame and the whip carries only the loupe.

## 8. What it needs from the 4K library

**Its kit.** None: `kits.rs` and PREBUILD.md list VernierLoupe among the dashes with nothing to pre-build ("procedural only by their briefs"; branch `origin/claude/project-thread-ib67x9`). No `assets/clusters/vernier_loupe/prebuild/` exists.

**Shared parts it can reuse.** The Liquid Glass pilot's `assets/clusters/liquid_glass/studio_env.png` (octahedral studio, 4096², gain 16, black within 44° of the viewer), as CarplayIsland and others do; the kit's analytic studio as fallback; the font files already in `assets/fonts/` (Saira, Saira Condensed, Barlow Semi Condensed, Titillium Web, Michroma).

CLOSED (was a LIBRARY GAP): the contact magnifier is in the kit since main 1d456d2: `docs/specs/optics.md` §4 and `Profile::magnifier(radius, m, n)` with the helpers in `fh6_kit::optics`. The kit's finding: a ×2.5 sapphire loupe is the flattest choice, and folding starts at ×2.75. The dash uses that profile instead of its own law.
LIBRARY GAP: optics.md entry for the image velocity under a moving lens, (1 − m) v, and the exposure smear it gives; no size; needed only if the whip smear is kept; the dash thread in `optics.rs`.

## 9. Departures and open questions for the board

1. **Round loupe, not 170×110.** The kit's surface from a non-circular outline magnifies across the rail only (§4). Recommend a circle about Ø110 to Ø140, or a vertical cylinder if the board wants a rod.
2. **×2.5 instead of ×2.4.** The Cyclops's power and the most even field (2.3% to 0.8R). Swell to n² = ×3.13, not ×3.2.
3. **Where the vernier lives.** A full 10-over-9 vernier does not fit inside any loupe the brief allows. Recommend a vernier plate on the loupe's carriage just under the rail, at 1×, centred on the index, with the coincident line lit and magnified when it falls inside the loupe; or a 5-over-4 vernier (20 rpm least count, span 4s), which fits inside the rail a Ø140 to Ø170 loupe shows when max rpm is above about 6,900 to 5,700 (Computed: 4s ≤ 70 to 85 px).
4. **Gear lens size.** Is 230 px a diameter (then nearly flat, ×1.02) or a radius? The board decides, with the 22° rule (§4).
5. **G-lens rim from tyre combined slip** (updated 2026-11-28): the field is decoded, offset confirmed at check C, so the brief's input is kept and is no longer a departure; body slip is the fallback. Thresholds are MODEL until check C reads the field's scale, and the board picks one wheel rule (largest of four, or the rear axle for drift).
6. **Load as dot size** needs the G-lens base to follow |g| (§5); otherwise drop the claim.
7. **No baked normal map** in any lens (glass rule).
8. **Tell it apart from `LiquidGlass`.** The pilot already has a bead that rides its rev rod and magnifies the graduations, and 16 capsule cabochons. VernierLoupe must read differently at thumbnail size: one long hairline rail with a single strong lens (×2.5 against the pilot's 1.15), round pods, the vernier plate.
9. **The smear is not in optics.md.** Keep it only with the second gap above, or use option (b) of §7.

## 10. Sources

Benchmark and references
- Rolex, Cyclops lens: https://rolex.com/watchmaking/features/cases/cyclops-lens
- Monochrome Watches, The Rolex Cyclops lens: https://monochrome-watches.com/the-rolex-cyclops-lens/
- Swiss Watch Expo, Rolex date and Cyclops lens: https://www.swisswatchexpo.com/TheWatchClub/2020/05/01/rolex-date-and-cyclops-lens/
- Wikipedia, Vernier scale: https://en.wikipedia.org/wiki/Vernier_scale
- Wikipedia, Dome magnifier: https://en.wikipedia.org/wiki/Dome_magnifier
- Wikipedia, Reading stone: https://en.wikipedia.org/wiki/Reading_stone
- Edmund Optics, Magnifying lenses: https://www.edmundoptics.com/knowledge-center/application-notes/microscopy/magnifying-lenses-how-to-choose-a-magnifier/
- B. Koppany, "Magnifiers", Journal of the Oughtred Society 19.2 (2010): https://www.oughtred.org/jos/articles/V19.2_p28-31.pdf
- MacRumors, iOS 15 text loupe: https://www.macrumors.com/2021/06/09/ios-15-tidbits-loupe-magnifier-dial-picker-more/

Optics and perception
- GeoGebra, Aplanatic lens (R/n and nR conjugates): https://geogebra.org/m/jHzzXcXW
- J. M. Wolfe et al., Sensation & Perception 4e, essay 3.1 (vernier acuity): https://learninglink.oup.com/public/files/cdn/wolfe4e/essay03.01.html
- Repo: `docs/specs/optics.md` (§2 profile, §4 contact model, §7 studio and light), `crates/fh6-kit/src/optics.rs` (`Profile`, `view_through`), `data/design_tokens.json` (`motion.springs`), `crates/fh6-clusters/src/clusters/liquid_glass/HANDOVER.md`

Leads not read this pass (unconfirmed)
- ION museum sextant: https://www.ion.org/museum/Sextant-with-beveled-scale.cfm
- RMG small craft precision sextant: https://www.rmg.co.uk/collections/objects/rmgc-object-43427
- Lee Valley, Veritas rule magnifier: https://www.leevalley.com/en-us/shop/tools/hand-tools/marking-and-measuring/marking-accessories/45035-veritas-rule-magnifier
- Wikipedia, Solid immersion lens (Weierstrass sphere; its formulas did not extract): https://en.wikipedia.org/wiki/Solid_immersion_lens
