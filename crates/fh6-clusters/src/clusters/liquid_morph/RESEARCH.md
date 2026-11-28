# Liquid Morph: research note

Build #24, catalog row 56, category Abstract, Target `concept`, family Liquid Glass, view cockpit. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read on 2026-11-28 in the source linked; **Computed** means worked out here from cited or brief values with `python3 -I`, numpy and scipy (inputs and numbers are given so they can be re-run); **Our reading** is a conclusion or idea for this dash, open to the direction check. Where a point could not be confirmed it says "unconfirmed" and what step 1 must check. The earlier web check (`/mnt/project-files/dash-research/webcheck/LiquidMorph.md`) has already been folded into the catalog brief; this note builds on that brief, not on the older copy in this folder's `spec.md`.

## 1. The benchmark

A `concept` dash names one real object. **Ours is the Liquid Glass morph of iOS 26 as Apple's own SwiftUI sample shows it: the Landmarks badges that flow out of a toolbar button inside one `GlassEffectContainer` and are "re-absorbed gracefully" into it** (WWDC25 session 323, chapter "Liquid Glass effects", 17:57), with the eraser and pencil pair of Apple's article "Applying Liquid Glass to custom views" as its smallest case. `LiquidGlass` already takes Liquid Glass's lensing and travelling highlight as its reference; this dash takes only the morph, which is what tells the two apart. A physical second reference, for the thread moment only, is a liquid jet breaking into drops (Rayleigh's and Savart's plates, section 2).

What makes the benchmark recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | Several glass shapes render as one: a container "combines multiple Liquid Glass shapes into a single shape that can morph individual shapes into one another". | **Cited**, [GlassEffectContainer](https://developer.apple.com/documentation/swiftui/glasseffectcontainer) |
| B2 | Merging depends on distance, not on contact: "As shapes near one another, their paths start to blend into one another. The higher the spacing, the sooner blending begins." | **Cited**, same page |
| B3 | The spacing is a length with a stated rule: in Apple's example the container spacing is 40 and the eraser morphs into the pencil "when the eraser's nearest edge is less than or equal to the container's spacing"; a container spacing larger than the layout spacing makes shapes "blend together at rest". | **Cited**, [Applying Liquid Glass to custom views](https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views) |
| B4 | Two kinds of appearance: shapes inside the spacing morph (`matchedGeometry`); shapes farther apart `materialize`, and "the system applies more than opacity changes". | **Cited**, same article |
| B5 | Objects "materialize in and out by gradually modulating the light bending and lensing" instead of fading. | **Cited**, [WWDC25 219 transcript](https://nonstrict.eu/wwdcindex/wwdc2025/219/) (Apple's video: https://developer.apple.com/videos/play/wwdc2025/219/) |
| B6 | Lensing, not scatter: the material "dynamically bends, shapes, and concentrates light", where earlier materials "scattered light". | **Cited**, WWDC25 219 |
| B7 | Motion "more like a lightweight liquid", with "an inherent gel-like flexibility"; controls "continually shape shift" between states on one floating plane. | **Cited**, WWDC25 219 |
| B8 | Highlights from lights in an environment "travel around the material, defining its silhouette", and "in some cases, the lighting responds to device motion". | **Cited**, WWDC25 219 |
| B9 | Larger glass reads as thicker: it gets "more pronounced lensing and refraction effects" when it morphs to a larger size. | **Cited**, WWDC25 219 |
| B10 | "Glass can not sample other glass", so nearby glass belongs in one container; avoid glass on glass. | **Cited**, [WWDC25 323 transcript](https://nonstrict.eu/wwdcindex/wwdc2025/323/); WWDC25 219 |
| B11 | Two variants that "should never be mixed": Regular (adaptive) and Clear, which "is permanently more transparent" and "needs a dimming layer" for legibility. | **Cited**, WWDC25 219 |
| B12 | Apple's public framing: "translucent and behaves like glass in the real world", controls "dynamically morph as users need more options". | **Cited**, [Apple Newsroom, 9 June 2025](https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/) |

**Our reading.** Three things carry the recognition, and the visual review should check them on the `02_mid_cruise` render and the motion sequence: (1) separate rounded glass bodies that join through a smooth neck with no crease and part again, all on one plane, (2) joining starts before contact, at a stated distance (B2, B3), which is exactly what the kit's smooth union does (section 4), and (3) a new body arrives by its lensing growing, never by a fade (B5, and the family's materialise rule). Apple's spacing and our blend length play the same role; they are not the same number (section 4).

**Departures forced by the house rules (Our reading).** Apple's glass "blurs content behind it" (B1 article); the glass rule forbids frost, so this dash keeps Apple's morph and drops the blur. Apple's Regular variant adapts its tint and flips light and dark; the dash is Clear only (B11), and its "dimming layer" is the fixed black core under each numeral. Apple's drops are rounded rectangles and capsules on a phone; ours are drops on OLED black.

**Better target?** No. Nothing found shows clear glass bodies merging by distance more directly than Apple's container. The liquid jet stays the reference for the thread, not the benchmark.

## 2. Reference photos

No image bytes were saved (no tool here saves them); the PC fetches the URLs in `SOURCES.md`.

| # | What it shows | Direct image URL (where known) | Source page |
|---|---|---|---|
| 1 | Apple's Liquid Glass hero across devices | https://www.apple.com/newsroom/images/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/article/Apple-WWDC25-Liquid-Glass-hero-250609_big.jpg.large.jpg | Apple Newsroom, 9 Jun 2025 |
| 2 | macOS Tahoe desktop, the clear look (Clear variant over a picture) | https://www.apple.com/newsroom/images/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/article/Apple-WWDC25-Liquid-Glass-Home-Screen-clear-look-250609_big.jpg.large.jpg | same |
| 3 | The same desktop with a dark tint (what this dash does not do) | https://www.apple.com/newsroom/images/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/article/Apple-WWDC25-Liquid-Glass-Home-Screen-dark-tint-250609_big.jpg.large.jpg | same |
| 4 | Newsroom tile image | https://www.apple.com/newsroom/images/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/tile/Apple-WWDC25-Liquid-Glass-hero-250609-lp.jpg.og.jpg | same |
| 5 | Landmarks badges morphing out of the toolbar button and re-absorbed (the benchmark), frames from 17:57 to 21:31 | video frames; HD video linked on the page | https://developer.apple.com/videos/play/wwdc2025/323/ |
| 6 | Lensing, materialise and controls shape-shifting, Dynamics chapter 1:29 to 6:00 | video frames | https://developer.apple.com/videos/play/wwdc2025/219/ |
| 7 | Eraser morphing into the pencil, container spacing 40 (the article's second video) | embedded video | https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views |
| 8 | Savart's 1833 figure of a 6 mm jet breaking into main and satellite drops (Fig. 7) | PDF page 4 to 6 | Eggers and Villermaux 2008, https://people.maths.bris.ac.uk/~majge/rpp8_3_036601.pdf |
| 9 | A dripping drop of 1887 forming a satellite (Fig. 8) and Rayleigh's 1891 plate (Fig. 9) | PDF page 5 | same |
| 10 | Three liquids detaching from a nozzle (lead figure) and the jet's intermediate stage with radii of curvature | page figures | https://en.wikipedia.org/wiki/Plateau%E2%80%93Rayleigh_instability |
| 11 | Bottom view of a drop's neck opening into a bath, capillary waves ahead of the front (Fig. 3a) | HTML figure | Deblais et al. 2024, https://arxiv.org/html/2402.00500v2 |
| 12 | X-ray images of two water drops coalescing, the toroidal bulge in the bridge | page figures | https://www.aps.anl.gov/APS-Science-Highlight/2008-04-24/how-two-drops-become-one |
| 13 | A drop on a lotus leaf, and the plot of drop shape against radius over capillary length (micro and macro drops) | page figures | https://en.wikipedia.org/wiki/Capillary_length |

Drive `references` folder: https://drive.google.com/drive/folders/1esuFLdxQgqRhBxSaUaJvOjIXNLFpSC4l (inside `liquid_morph`, https://drive.google.com/drive/folders/1kcuwiJ8GnqtufCaFEGhfVKqLWcaoSFzp), holding `SOURCES.md`.

## 3. How the real thing behaves, with motion notes

Apple publishes no durations or springs for the glass morph; the timings below are the tokens' (`data/design_tokens.json`) unless marked.

| State | The benchmark | This dash (Our reading unless marked) |
|---|---|---|
| Start-up | Glass materialises by lensing (B5). | Every drop materialises over 200 ms (`durations.materialize`) by growing thickness and base from 0; the ten small drops first, then the three large ones, all in place. No ignition sweep through the thread: the thread is kept for the shift point. |
| Idle | Resting shapes stay apart unless the spacing exceeds the layout gap (B3). | Three large drops apart (blend length 30 px at zero G, visible gap 56.5 px, section 4); ten small drops dark and apart; still. |
| Cruise | Calm. | Same as idle; only the speed digits and the channel's light change. A light throttle does not join the row (it joins at +0.17 g). |
| Pull to the limiter | Controls shape-shift as content changes (B7). | The row joins under acceleration past +0.17 g; each shift stage lit makes the thread grow inwards from both ends by one drop (section 4). |
| Shift point and limiter | Interaction "illuminates from within" (WWDC25 219). | At the shift point the ten drops are one thread 18 px thick. On the limiter only the thread strobes (section 7). |
| Gear change | Badges re-absorbed into the button (B1). | The thread breaks back into ten drops along the Rayleigh–Plateau wavelength; the numeral changes behind a still core (section 7). |
| Braking | None. | Blend length falls to 20 px at −0.5 g: the row is three drops with the widest gaps (63.4 px). This is the opposite of the family's braking moment (section 9, item 3). |
| Drift and lateral G | Highlights travel round the silhouette with device motion (B8). | The family light swing (25° at 1.5 g); the two outer drops' glass leans sideways 8 px per g, clamped at 12 px; the cores and numerals stay still. |
| Reverse and neutral | None. | Neutral: the gear drop carries "N" and the row stays as G sets it. Reverse: "R"; the row behaves the same (G under reversing is small). |
| Shut-down | Materialise out. | The host hides the dash in 1 frame (family rule); no farewell. |

**Real-liquid timing, for the springs (Cited and Computed).** Two drops joining open their bridge as R(τ) = A (γR₀/ρ)^(1/4) τ^(1/2), A between 1 and 1.25 measured and 1.62 simulated ([Deblais et al. 2024](https://arxiv.org/html/2402.00500v2), eq. 1): fast at first, then slowing. A thread pinching off thins as h_min = 0.7 (γ(t₀ − t)²/ρ)^(1/3) ([Eggers and Villermaux 2008](https://people.maths.bris.ac.uk/~majge/rpp8_3_036601.pdf), §1.2): slow at first, then a snap. **Our reading:** joining eases out (a square-root ramp) and parting eases in (a two-thirds-power snap), so the glass reads as liquid rather than as a cross-fade. Neither is a spring in the tokens; both are short curves the thread can bake into the tokens' 256-entry easing LUTs.

## 4. The maths

**Smooth union (optics.md §2, kit `shape.rs`).** smin_k(a, b) = −k ln(e^(−a/k) + e^(−b/k)), natural logarithm, k a length. Two outlines a gap g apart touch when the field midway is zero: g/2 − k ln 2 = 0, so k = g ÷ (2 ln 2) = 0.7213 g (**Cited**, optics.md; **Computed** below). Quilez writes the same function with log₂ and exp₂ and k as a sharpness ([iquilezles.org/articles/smin](https://iquilezles.org/articles/smin)); optics.md [S25] already notes the rescale. In the kit a `Shape::Blob` holds up to 64 pieces (`MAX_BLOB_PIECES`, `shape.rs`), so the brief's worry about thirteen pieces is answered. Its cross-section (thickness, bevel, base) is blended between pieces by the same softmax weights as the distance (`blob_field` in `shape.wgsl`), so a thin thread and a fat drop in one group meet without a step (**Our reading from the code**).

**The three large drops (Computed, on the axis y = −30, gear disc r 125 at the origin, a 280 × 150 stadium centred at +330).**

| Blend length k (px) | Low-passed g_long | Visible gap on the axis (px) | Neck height midway (px) |
|---|---|---|---|
| 20 | −0.5 g | 63.35 | apart |
| 30 | 0 g (cruise) | 56.50 | apart |
| 40 | +0.10 g | 39.83 | apart |
| 45 | +0.15 g | 21.63 | apart |
| 46.80 | +0.166 g | 3.50 | apart |
| 46.888 | +0.1689 g | 0 (touch) | 0 |
| 50 | +0.20 g | joined | 47.7 |
| 60 | +0.30 g | joined | 99.4 |

The brief's map (20 px at −0.5 g, 30 px at 0, 60 px at +0.3 g, linear between) puts the touch at +0.1689 g, as it says. Two findings: (1) the gap closes as a snap, from 40 px at k 40 to nothing at k 46.9, so a car hovering near +0.17 g would flicker between one body and three, and the low-pass alone does not stop it; (2) at k 60 the gear drop's top rises 3.2 px (−155 to −158.2) because the union swells every outline a little, so the gear core must clear that. **Our reading, recommended:** a joined/apart state with hysteresis: join when the low-passed g passes +0.19 g, part only below +0.12 g; while joined the blend length is held at least 47.5 px, and on parting the neck pinches over about 120 ms on the two-thirds curve before the gap opens. Low-pass g_long on the tokens' `slow` spring (T 0.363 s, ζ 0.9) over `Signals::g_long`, which is already smoothed.

**Why the thread cannot be made by blend length (Computed, ten discs Ø34 at 81 px pitch, gap 47 px).** The drops touch at k = 47 ÷ (2 ln 2) = 33.90 px. Raising k on the drops alone swells the whole row first: at k 30 the row is 49.2 px tall, at the touch 57.2 px, and the neck jumps straight to about 34 px, never passing 18. That breaks the family's 42 px cap for the strobing strip in both directions. **Recommended (Our reading):** the thread is its own piece, a capsule of radius 9 (`Outline::Capsule` exists in the kit), in the same blob as the ten drops with a small blend length (3 to 8 px keeps the strip at 33.8 to 34.0 px tall, computed). As each stage lights, a lit drop is absorbed into the capsule growing from its end, and since the shift lights fill outside in, two threads grow from both ends and meet in the middle at the shift point.

**Rayleigh–Plateau (Cited, then Computed).** The growth rate of a ripple of wavenumber k on an inviscid thread of radius R is ω² = (σ ÷ ρR³) · kR · I₁(kR) ÷ I₀(kR) · (1 − k²R²), unstable only for kR < 1 ([MIT 18.357 lecture 11](https://ocw.mit.edu/courses/18-357-interfacial-phenomena-fall-2010/d77da8d5f1bb8b69a62682bc0dbc5845_MIT18_357F10_Lecture11.pdf), eqs. 11.30 to 11.33; [Wikipedia](https://en.wikipedia.org/wiki/Plateau%E2%80%93Rayleigh_instability)). The lecture gives kR = 0.697, λ ≈ 9.02 R and a break-up time of about 2.91 √(ρR³/σ); Eggers and Villermaux give 9.01 R and k_m R ≈ 0.7. **Computed** (scipy, bounded minimiser): the maximum is at kR = 0.697019, λ = 9.0144 R, ω_max² = 0.117881 σ/(ρR³), so the e-folding time is 2.9126 √(ρR³/σ). The catalog's 9.015 R rounds 9.0146 (from 0.697); 9.014 is the exact figure and 9.01 the safe one to print.

| Test value (R = 9 px) | Computed |
|---|---|
| Wavelength λ | 81.13 px (catalog 81.1) |
| Drop radius from one wavelength of thread, (¾ R² λ)^(1/3) | 1.8909 R = 17.02 px (catalog 17.0) |
| Ten wavelengths against the drops' centre span plus one pitch (729 + 81) | 811.3 px against 810 px: the drops re-form within 1.3 px of where they sat |
| Thread length holding ten drops of r 17.02 (cylinder plus two hemispherical ends) | 799.3 px of cylinder, 817.3 px overall (catalog "about 817"); 814.7 px for drops of exactly 17 |
| One drop's volume against one pitch of thread, as half-bodies (the kit's hemispherical profile, H = w = r) | 10,322.7 against 10,322.4 px³: the 3D law carries to the domed glass because both halves are halves |
| Thread radius that keeps the span at 763 px with the same volume | 9.30 px (18.6 px thick), whose wavelength would be 83.9 px, not 81 |
| Linear growth from a 5% ripple to pinch | ln 20 ÷ 0.34334 = 8.73 e-folding units; a 200 ms break-up sets the MODEL time unit at 22.9 ms |

**Satellite drops.** A real thread also leaves a smaller drop between two main drops (Savart's figure; Eggers and Villermaux §1.2) and main drops come out larger than the wavelength predicts because the thread does not break everywhere at once ([Carnevale et al. 2023](https://arxiv.org/abs/2307.07033)). Neither paper read here gives a size ratio (**unconfirmed**; step 1 can look in Eggers and Villermaux §3.3). The dash leaves satellites out and labels the break-up MODEL.

**Drop shapes (Cited and Our reading).** Drops smaller than the capillary length are spherical caps; larger ones are "flattened by gravity" ([Wikipedia, Capillary length](https://en.wikipedia.org/wiki/Capillary_length)). That is what the numerals need: the ten small drops stay round domes, and the three large ones are puddles with flat tops. **Computed** from the brief's sizes: a gear numeral of cap 160 (width about 80, half-diagonal about 89 px) needs a flat top of radius 89 on the Ø250 drop, so a bevel of at most 36 px (at most 57.6 px if only the family's 22° rule is applied, since a circular bevel tilts under 22° once 0.625 of the bevel in). The speed drop (half height 75) with numerals of cap 90 that the glass may lean 12 px across needs a bevel of at most 18 px (28.8 px under the 22° rule). A puddle's height of about 2 l_c sin(θ/2) is the textbook form (de Gennes, Brochard-Wyart and Quéré, *Capillarity and Wetting Phenomena*, chapter 2: **unconfirmed**, not opened); the kit's superellipse edge stands in for it, labelled MODEL.

**Catalog check.** The maths row's numbers hold: 46.89 px, 81.1 px, 17.0 px. Two corrections the lead may want: "a wavelength of 9.015 R" is 9.014 R (or "9.01 R"); and the row's tests should add "the ten drops never pass 42 px tall, joined or apart", because the blend-length route breaks it (above).

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Ten drops and the thread | `shift_fraction` (rpm ÷ shift rpm), shift stages from `revStages` | verified | stage n lights drops n from the ends inward; a lit drop is absorbed into the end capsule on the `snap` spring (T 0.102 s) |
| Channel light | `rpm_fraction` | verified | light runs from the left under the 1100 × 36 channel on the `ribbon` spring with feed-forward; ticks from the car's max rpm |
| Limiter strobe | rpm ≥ shift rpm | verified | thread only, 10 Hz, 50% duty, starting lit; 2 Hz breath when `reduced_flash` |
| Blend length of the three large drops | `g_long` | verified (smoothed) | low-pass on `slow`, map 20, 30, 60 px at −0.5, 0, +0.3 g, hysteresis state (section 4) |
| Gear numeral | `gear` | verified | `Gear::label`; incoming glyph on `fast` |
| Speed | `speed`, `unit` | verified | tabular Saira 300, `Hysteresis` 0.6 |
| Power and its peak | `power` (W), `power_unit` | verified | kW or hp by `power_unit`; peak held until the next pull begins (throttle rises past 0.9 after falling below 0.3, or a gear change) |
| Outer drops' lean | `g_lat` | verified (smoothed) | 8 px per g, clamped to 12 px, on `parallax` (ω 12 rad/s) |
| Studio light swing | `g_lat` | verified | family rule, 25° at 1.5 g |
| Materialise | `race_on` | verified | 200 ms thickness ramp |
| Optional: torque in the power drop | `torque` | decoded, offset confirmed at check C | fallback torque = power × 60 ÷ (2π rpm) |

**What the game cannot send** (parts of the benchmark with no telemetry), each with a replacement:

1. **Touch: glass that "illuminates from within" under the finger and spreads to nearby glass.** (a) **Recommended:** the throttle is the touch: a throttle stab past 0.3 in 200 ms lights the inside of the speed and power drops from the side nearest the gear, spreading in 150 ms (`crossfade`), never over a numeral's core. (b) The brake lights the gear drop's lower rim in `warning`, steady.
2. **Content scrolling under the glass (the reason Apple's glass lenses anything).** (a) **Recommended:** the channel's rev light is the content: as it passes under the gear drop's base the drop really bends it, so the glass is seen working. (b) The speed drop bends a faint tick scale that moves with distance travelled (decoded, offset confirmed at check C; fallback the integral of speed).
3. **Menus popping open out of a button.** (a) **Recommended:** at a standstill (speed 0, idle) the power drop shows the last pull's peak and, when that pull ran from rest, its 0 to 100 km/h (0 to 60 mph) time from speed and the timestamp, then re-absorbs it into one figure when the car moves. (b) On a gear change the outgoing gear's numeral is drawn, small, in the thread's middle drop for 300 ms.
4. **Adaptive tint and the light/dark flip of the Regular variant.** (a) **Recommended:** none: Clear only (B11), with the black cores as Apple's dimming layer. (b) The light rig's lean to the palette's primary rises with throttle by at most 10%.
5. **Device-motion highlights (B8).** (a) **Recommended:** the car's G does it: the family light swing and the 6° per g tilt. (b) Yaw rate (decoded, offset confirmed at check C; fallback a_lat ÷ V, labelled MODEL) turns the environment by up to 10°.
6. **Badges appearing as the user earns them.** (a) **Recommended:** the race position (decoded, check C) appears as a small drop that materialises out of the power drop when it changes, and is re-absorbed 2 s later; hidden outside a race. (b) Lap number in the same way.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| Max rpm 0 or tiny, shift ≤ idle | no stage lights, the thread never forms, the channel is dark; no NaN in `shift_fraction` |
| Idle near max | stages light on a blip; the thread forms and breaks fast, which is correct |
| 12,000+ rpm and 4,500 rpm diesels | the scale is the car's; stages follow `shift_fraction`, so nothing changes |
| EV (one gear, idle 0) | no gear-change break-up: when rpm falls below the first stage the thread breaks the same way, so the moment survives |
| Reverse | "R"; the row follows G as usual |
| Neutral | "N"; a blip in neutral still forms the thread (the engine is at the shift point) |
| Gear above 10 | `Gear::from_raw` gives Neutral above 10 |
| mph and km/h, hp and kW | `unit` and `power_unit` from `Signals`, tabular digits |
| Limiter bounce of about 50 rpm a frame | the thread holds once formed; it breaks only below 0.95 of the shift rpm or on a gear change, so it never chatters; digits use `Hysteresis` |
| Crash spike or airborne landing | g_long clamped to [−1.5, +1.5] g before the low-pass; the join state's hysteresis stops a one-frame join; lean clamps at 12 px |
| Hovering near +0.17 g | the join/part hysteresis (section 4) |
| Stale or paused stream | the app hides the dash; on return, no materialise unless `race_on` rose |
| Palette extremes and random palettes | rims come from the studio, so glass reads on any palette; the thread's `shift` colour is the only hue that changes |
| Font swap | cores are fixed sizes; flat tops sized for the widest default glyph plus 10% |
| Cockpit only | no hood variant |
| Reduced flash | breath at 2 Hz; the throttle glow (5.1a) and the break-up's travelling highlight are left out; Apple's Reduce Motion "disables any elastic properties", so the break-up becomes a 150 ms cross of thickness |
| Colour-only cues | the shift point is a shape (one thread), not only a colour |

## 7. Signature moments

**Limiter.** (A) **Recommended:** the thread alone strobes, 10 Hz at 50% duty from a clock starting lit at the shift point, 2 Hz breath in gentle mode; between flashes it keeps its glass and only its light below goes dark, so its shape stays visible. (B) The thread's light strobes while its surface carries a standing ripple at the Rayleigh wavelength, held below the 5° slope limit, so it looks about to break.

**Gear change.** (A) **Recommended:** the thread breaks: a ripple of wavelength 81.13 px grows on it, the troughs pinch on the two-thirds curve, and ten drops of radius 17 re-form over the ten seats in about 200 ms (MODEL time unit 22.9 ms), left to right if the car's upshift is fast or all at once otherwise; the gear core stays still. (B) The break-up starts from the middle and runs to both ends, the reverse of how it filled. (C) On a downshift the large row jolts: blend length steps by +8 px for 150 ms, so the three drops reach for each other once.

## 8. What it needs from the 4K library

The pre-build (PR #18, `PREBUILD.md`) made **no kit** for this dash: it reuses "the existing `liquid_glass` studio plate (seed 77203)", and "LiquidMorph's drops ... are optical glass, which the kit computes from the outline". Reusable now: `assets/clusters/liquid_glass/studio_env.png` (Blender studio, octahedral 4096², black within 44° of the viewer) and its ComfyUI panorama master, and the kit's analytic studio as the fallback. Do not reuse `liquid_light.png` under the channel: the liquid-light caustic under a rod is `LiquidGlass`'s mark (section 9, item 6). Fonts Saira, Saira Condensed and Titillium Web are in `assets/fonts`.

LIBRARY GAP: a bespoke ComfyUI studio panorama, 4096 × 2048, seamless left to right, darker and with fewer practicals than the pilot's, so the drops' rims stay thin lines on black; ComfyUI SDXL plus 4x-UltraSharp, as the pilot's `studio_plate`.
LIBRARY GAP: a bespoke Blender studio environment, octahedral 4096², the family's horizon ring, key and far strips carrying that panorama, cleared within 44° of the viewer; Blender Cycles through the pilot's recipe.
LIBRARY GAP: a ray-traced caustic tile for the light a drop throws on the black beneath it: a unit sapphire drop (n 1.768) as a dome and as a flat-topped puddle, lit by the key light at rest and at ±25°, 4096² master each; Rust in asset-forge on the CPU (Snell's law, as `liquid_glass::caustic`), scaled per drop at run time from a single-channel irradiance tile.
LIBRARY GAP (optional, needs an optics.md addition first): a 4096 × 1 table of a sessile macrodrop's edge (height against distance in from the rim, from the Young–Laplace equation) to replace the superellipse edge on the three large drops; computed on the CPU in asset-forge. The brief's Blender normal and thickness pair is not recommended: a baked map cannot follow an outline that merges.

## 9. Departures and open questions for the board

1. **The thread is a capsule, not a blend.** Raising blend length on ten drops breaks the 42 px cap (49 to 57 px) and jumps the neck past 18 px (section 4).
2. **The thread is 817 px wide, the drops 763 px.** Keeping volume with an 18 px thread needs 817 px; keeping 763 px needs an 18.6 px thread and moves the wavelength to 83.9 px. Recommended: 817, as the brief's Risk line already allows.
3. **Braking parts the row; the family joins it.** `liquid-glass.md` says "the side pieces reach for the centre piece and fuse" under braking. This brief does the opposite (one body under power), which helps tell it from `LiquidGlass`; the brief outranks the family on the instrument idea, and the handover must say so.
4. **At cruise the drops sit 2.2 blend lengths apart, not 3** (family layout rule). They are one blob group, so the rule's reason (separate drawing would show) does not apply; the swelling is 4.25 px per side and is the "blending begins" of B2.
5. **Join hysteresis** at +0.19 g up and +0.12 g down, with a pinch on parting; the brief has a low-pass only.
6. **Telling it apart.** From `LiquidGlass`: no cabochons over a rod, no caustic ribbon, no G lens; the shift strip becomes a thread by Rayleigh–Plateau, which `LiquidGlass`'s fused bar is not. From `VisionPetals`: its petals never share a blob group and overlap as separate lenses, ours always merge. From `KineticLiquidMetal`: chrome with an oil film versus clear glass with no colour inside; its drops spill from a ribbon tip on the limiter, ours are the shift lights. The thumbnail must read as a row of three clear drops over a line of ten.
7. **Flat tops.** The large drops are puddles with bevels of at most 36 px (gear) and 18 px (speed and power) if numerals stay unbent.
8. **No frost and no adaptive tint**, though Apple's glass has both (section 1).
9. **Satellite drops left out**, and the break-up timing is MODEL.

## 10. Sources

**Apple**
- Apple Newsroom, "Apple introduces a delightful and elegant new software design", 9 Jun 2025: https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/
- WWDC25 session 219, "Meet Liquid Glass": https://developer.apple.com/videos/play/wwdc2025/219/ ; transcript read at https://nonstrict.eu/wwdcindex/wwdc2025/219/
- WWDC25 session 323, "Build a SwiftUI app with the new design": https://developer.apple.com/videos/play/wwdc2025/323/ ; transcript read at https://nonstrict.eu/wwdcindex/wwdc2025/323/
- SwiftUI, GlassEffectContainer: https://developer.apple.com/documentation/swiftui/glasseffectcontainer
- SwiftUI, "Applying Liquid Glass to custom views": https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views

**Liquids**
- MIT 18.357 Interfacial Phenomena, lecture 11 (Rayleigh–Plateau): https://ocw.mit.edu/courses/18-357-interfacial-phenomena-fall-2010/d77da8d5f1bb8b69a62682bc0dbc5845_MIT18_357F10_Lecture11.pdf
- J. Eggers and E. Villermaux, "Physics of liquid jets", Rep. Prog. Phys. 71, 036601 (2008): https://people.maths.bris.ac.uk/~majge/rpp8_3_036601.pdf
- Wikipedia, "Plateau–Rayleigh instability": https://en.wikipedia.org/wiki/Plateau%E2%80%93Rayleigh_instability
- L. H. Carnevale, P. Deuar, Z. Che and P. E. Theodorakis, "Liquid thread breakup and the formation of satellite droplets", Phys. Fluids 35, 074108 (2023): https://arxiv.org/abs/2307.07033
- J. Eggers, J. R. Lister and H. A. Stone, "Coalescence of liquid drops" (1999): https://ar5iv.labs.arxiv.org/html/physics/9903017
- A. Deblais et al., "Early stages of drop coalescence" (2024), eq. 1: https://arxiv.org/html/2402.00500v2
- K. Fezzaa and Y. Wang, Phys. Rev. Lett. 100, 104501 (2008), as reported by Argonne APS: https://www.aps.anl.gov/APS-Science-Highlight/2008-04-24/how-two-drops-become-one
- Wikipedia, "Capillary length" (micro and macro drops): https://en.wikipedia.org/wiki/Capillary_length
- P.-G. de Gennes, F. Brochard-Wyart and D. Quéré, *Capillarity and Wetting Phenomena* (Springer, 2004), chapter 2 (not opened)

**Maths and the kit**
- I. Quilez, "Smooth minimum": https://iquilezles.org/articles/smin
- `docs/specs/optics.md` §2 (smooth union, materialise, ripple slope limit), §4 (contact model), §7 (studio, 44° dark cone); `crates/fh6-kit/src/shape.rs` (`smooth_union`, `MAX_BLOB_PIECES`, `Outline::Capsule`); `crates/fh6-kit/src/shaders/shape.wgsl` (`blob_field`); `docs/design/families/liquid-glass.md`; `data/design_tokens.json` (springs, durations)
