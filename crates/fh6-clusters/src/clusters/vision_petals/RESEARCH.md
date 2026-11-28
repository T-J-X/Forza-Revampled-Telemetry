# Vision Petals: research note

Build #15, catalog row 51, category Abstract, Target `concept`, family Liquid Glass, view cockpit. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE. Labels: **Cited** means read on 2026-11-28 in the source linked; **Computed** means worked out here from cited or brief values with `python3 -I` and numpy (inputs and numbers are given so they can be re-run); **Our reading** is a conclusion or idea for this dash, open to the direction check. Where a point could not be confirmed it says "unconfirmed" and what step 1 must check.

Telemetry updated 2026-11-28 for the 88-field decoder (main 60c99c9).

## 1. The benchmark

A `concept` dash names one real object. **Ours is the Breathe animation of Apple Watch** (the Breathe app of watchOS 3, 2016; since watchOS 8 the Breathe session of the Mindfulness app), read through Apple's own patent for it, US 11,069,255 "Fluctuating progress indicator". Apple Vision Pro's Mindfulness flower is a second reference for the depth idea, not the benchmark: it is a 3D object round the viewer, and this dash is a flat figure on a screen.

What makes the benchmark recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | Several equal circles arranged round one centre point, with overlapping areas: "circular shapes aligned around a center point ... may have at least some overlapping areas". | **Cited**, US 11,069,255 (Apple Inc., priority 10 June 2016, granted 20 July 2021), [uspto.report](https://uspto.report/patent/grant/11069255) |
| B2 | The count is not fixed: the patent's example shows six elements for a one-minute, seven-breath session and "eight variable visual elements" for two minutes. A 2017 rebuild says the app adds a circle for each minute added. | **Cited**, same patent (FIG. 3, elements 320); [CSS-Tricks](https://css-tricks.com/?p=262519), G. Graham, 20 Nov 2017 |
| B3 | At the bottom of a breath it is "a simple circle": the elements are not visible as separate shapes. | **Cited**, patent |
| B4 | It grows while rotating one way on the inhale and shrinks while rotating back on the exhale. The patent is not consistent about which way (one passage says counterclockwise on the grow, another clockwise), and it gives no angle. | **Cited**, patent. The amount is **unconfirmed**: the CSS-Tricks rebuild turns 180° (scale 0.15 + rotate 180° to scale 1, 4 s each way); step 1 measures it from Apple's own video. |
| B5 | Overlaps are darker: the elements "may be partially transparent such that areas where the variable visual elements 320 overlap may be darker than other areas", and "the center ... may appear darker than the outer edges". | **Cited**, patent |
| B6 | Pace: seven breaths a minute by default (the patent's example, with inhale to exhale 1 to 1.5, so 3.4 s in and 5.1 s out), adjustable from 4 to 10 a minute; sessions of 1 to 5 minutes. | **Cited**, patent; [MacRumors](https://www.macrumors.com/2016/06/20/watchos-3-apple-watch-breathe-app/), T. Hardwick, 20 Jun 2016; [Apple Support](https://support.apple.com/en-jo/guide/watch/apd371dfe3d7/26/watchos/26); [Apple Newsroom](https://www.apple.com/newsroom/2016/06/apple-previews-watchos-3/). **Computed:** 60 ÷ 7 = 8.571 s a breath; ÷ 2.5 gives 3.429 s and 5.143 s. |
| B7 | Colour on the Watch: "several overlapping blue circles". The common rebuild uses six 125 px circles, alternately `#61bea2` and `#529ca0`, on black, with `mix-blend-mode: screen`. | **Cited**, [Wikipedia](https://en.wikipedia.org/wiki/Mindfulness_(Apple)); CSS-Tricks. The screen blend (lighter overlaps) is the rebuild's choice and disagrees with B5; which one the shipped Watch shows is **unconfirmed** until step 1 looks at Apple's press video frame by frame. |
| B8 | How far the circles travel. In the rebuild circle 1 ends at translate(−35, −50) px with R = 62.5 px. | **Computed:** offset √(35² + 50²) = 61.03 px, so d ÷ R = 0.977 at full inhale, at 55.0° from the horizontal (a perfect hexagon would be 60°). So the rebuild's circles very nearly pass through the centre at the top of the breath, which is the brief's d = R. |
| B9 | Vision Pro: "a sphere made out of petal-like objects that spread apart when instructed to inhale"; "an orb flower petal of sorts opens up in the center"; "the pedals will separate and surround you"; the view "gradually gets darker at the start of your session"; with Follow Your Breathing, "visual animations and sounds follow the rhythm of your breathing", and the animation "held steady, waiting for me to exhale". | **Cited**, Wikipedia; J. Krol, TheStreet via [Blue Mountain Eagle](https://bluemountaineagle.com/?p=228339), 12 Feb 2024; [Apple Support, Vision Pro](https://support.apple.com/guide/apple-vision-pro/devae0eb1c57/visionos); D. Dundee, [MacStories visionOS 2 review](https://www.macstories.net/stories/visionos-2-the-macstories-review/6/) |
| B10 | Seed of Life: "Six congruent circles have centers equally spaced on the circumference of a central circle", all of one radius. Our flower at N = 6, d = R is that figure without its central circle. | **Cited**, [MathWorld](https://mathworld.wolfram.com/SeedofLife.html) |

**Our reading.** Three things carry the recognition, and the visual review should check them: (1) it is one circle at rest and a ring of equal circles at full breath, (2) it turns as it opens, and (3) overlap depth reads as a change of light. B5 is a lucky match for this dash: Apple's own description of the overlaps (partly transparent, darker where they stack, darkest at the centre) is exactly what stacked clear glass does (section 4). Nothing in Apple's flower is glass, and a petal per gear is this dash's own idea.

**Correction for the catalog brief (the lead reports it).** The brief says "Apple publishes no circle count" and "The circles are opaque teal and green that add up where they overlap". Apple's patent gives counts in its example (six at one minute, eight at two) and describes partly transparent circles that get darker where they overlap; the opaque, additive look is the CSS rebuild's. Suggested wording: "Apple's patent shows six circles for a one-minute session and eight for two, partly transparent and darker where they overlap; the well-known rebuild draws six opaque circles that add up on black."

**Better target?** No. Research found nothing closer to a gear-counted flower; the Watch animation stays the benchmark.

## 2. Reference photos

No image bytes came back: WebFetch returned a server error for each of the five direct image URLs tried (rows 1, 3, 4, 5, 9), so the folder holds `SOURCES.md` only. Step 1 should save rows 1, 4, 5, 7 and 9 from a browser.

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 1 | Apple press image of the Breathe screen, watchOS 3 | https://www.apple.com/newsroom/images/product/os/watchos/standard/watchos3_breathe_inline.jpg.large.jpg | no (fetch failed) |
| 2 | Apple's downloadable Breathe press assets (zip) | https://www.apple.com/newsroom/images/product/os/watchos/standard/WatchOS3-Breathe.zip | no (archive) |
| 3 | MacRumors photo of the Breathe flower on the Watch | https://images.macrumors.com/t/j6OrSh9Q5m5owepU_0-BGprVTvE=/1600x/article-new/2016/06/watchos3_breathe.jpg | no (fetch failed) |
| 4 | The six-circle rebuild animated (open and close, turn) | https://css-tricks.com/wp-content/uploads/2017/11/apple-watch-breathe.gif | no (fetch failed) |
| 5 | Screen recording of Vision Pro Mindfulness, credited to Apple | https://techcrunch.com/wp-content/uploads/2024/01/RPReplay_Final1706721157.gif | no (fetch failed) |
| 6 | Video of the visionOS 2 petals following the breath | https://cdn.macstories.net/mindfunless-1726496477336.mp4 | no (video) |
| 7 | Patent FIG. 3: indicator 318a to 318c and elements 320 (six and eight elements) | https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11069255 | no (PDF page) |
| 8 | Patent FIG. 4: the grow and shrink sequence, indicators 424a to 424e | same PDF | no (PDF page) |
| 9 | Seed of Life line figure | https://mathworld.wolfram.com/images/eps-svg/SeedofLife.svg | no (fetch failed) |
| 10 | A six-petal rosette in a real mosaic floor (Herod's palace) | https://commons.wikimedia.org/wiki/File:Mosaic_floor_from_a_bathhouse_in_Herod%27s_palace_-_Google_Art_Project.jpg | no |
| 11 | Overlapping circles grid figures (rosette families) | https://en.wikipedia.org/wiki/Overlapping_circles_grid | no |
| 12 | YouTube still of the watchOS 3 Breathe demo | https://i.ytimg.com/vi/fHIIxZ0PW9w/maxresdefault.jpg | no |

Drive `references` folder: https://drive.google.com/drive/folders/1amdBgHG9WRPwdpXzqlIZ-6ZpEvr9vV0M (inside `vision_petals`, https://drive.google.com/drive/folders/1uigH3qoNBQw13OJ66xyohICsWYAf7KXb).

## 3. How the real thing behaves, with motion notes

| State | The benchmark | This dash (Our reading unless marked) |
|---|---|---|
| Start-up | The first indicator "rotates with a trailing portion ... less visible than a leading portion" in a preliminary phase, and a small circle appears "at the bottom of a breath" (**Cited**, patent). On Vision Pro the room darkens gradually (**Cited**, Apple Support). | Show is the family's materialise: petal thickness grows from 0 over 200 ms (tokens `durations.materialize`), never opacity. The ignition sweep (350 ms up on `emphasizedDecelerate`, 300 ms down on `standardAccelerate`, first `IsRaceOn` only, from the tokens) opens the flower to d = R and closes it: one breath. |
| Idle | A single circle at the bottom of the breath (**Cited**, patent). | d = 0, every petal on one disc, the closed flower: still. |
| Cruise | Steady breathing at 7 a minute. | d follows rpm and is still at a steady rpm: no breathing of its own. The catalog's Abstract line wants "calm at steady input". |
| Pull to the limiter | The figure grows and turns on the inhale. | d rises to R and the flower turns by 360° ÷ N, ending on its own symmetry (section 4). The beads fill outside in. |
| Shift point and limiter | Follow Your Breathing "held steady, waiting for me to exhale" (**Cited**, MacStories). | d is clamped at R, so the figure holds still: the held breath. Petal rims steady in `shift`; only the beads strobe (section 7). |
| Gear change | Patent: the count can change ("removing variable visual elements"), and a longer session has more elements (**Cited**). | The count changes with the gear: one petal unfolds from its neighbour on an upshift, one folds back into it on a downshift, on the `snap` spring (T 0.102 s, ζ 1). |
| Braking | Exhale: shrink and turn back. | Revs fall, so the flower closes and turns back by the same law. Nothing separate is needed. |
| Drift and lateral G | None. | The two petal depths shear by 2 and 4 px per g on the tokens' parallax spring (ω 12 rad/s); core and gear stay still; the family's light swings up to 25° at 1.5 g. |
| Reverse and neutral | None. | Neutral is the closed disc; reverse is one petal pointing down (the brief). |
| Shut-down | End of session: "less complex" indicator, then a summary (**Cited**, patent; MacRumors: heart rate shown). | The host hides the dash in 1 frame (family rule). No farewell animation. |

Haptic taps on the Watch "begin fast and gradually fade at the top of each breath" (**Cited**, MacRumors). That rhythm gives the beads' idea in section 5.

## 4. The maths

**Model** (the catalog row, checked). Petal i of N has its centre at d(cos φ_i, sin φ_i) with φ_i = 90° + 360° · i ÷ N + ψ, where ψ = (360° ÷ N) · d ÷ R and d = R · clamp((rpm − idle) ÷ (shift − idle), 0, 1), R = 110. `Signals` carries `shift_rpm` = max rpm × the tokens' shift fraction (default 0.97, clamped to 0.5..1, `fh6-kit/src/cluster.rs`).

**Computed values the unit tests can assert** (grid of 0.5 px for the depth counts):

| Test | Value |
|---|---|
| At d = R every petal passes through the centre | distance from each centre to the origin = R |
| Bounding radius d + R | 110 closed, 165 at half, 220 open |
| Unchanged by a turn of 360° ÷ N | the centre set is equal under the turn (checked for N = 7, d = 80) |
| Neighbouring centres at d = R | 2R sin(π ÷ N): N = 2 gives 2R, 3 gives 1.7321R, 4 gives 1.4142R, 6 gives exactly R, 10 gives 0.6180R = R ÷ φ (golden ratio) |
| N = 2 at d = R | the two petals touch only at the centre: no overlap at the shift point (max depth 1) |
| Deepest point at d = R | ⌈N ÷ 2⌉ petals: 2 for N = 3 and 4, 3 for 6, 4 for 8, 5 for 10. Near the centre a point is inside every petal whose direction lies in the open half-plane towards it. |
| Deepest point for d < R | all N, at the centre |
| Share of lit area by depth at d = R | N = 6: 45.3% one petal, 45.3% two, 9.5% three. N = 10: 18.5%, 29.9%, 29.9%, 18.5%, 3.3% for 1 to 5 |
| Below idle, max rpm 0, shift ≤ idle | d = 0, never negative or NaN |
| Full-open turn ψ | 360° ÷ N: 180° in second gear (Apple's rebuild's half turn), 60° in sixth, 36° in tenth |

**Light through the petals, and two corrections to the row.**

1. The brief says T loses "about 4% at each [face], for n = 1.5". The family glass is synthetic sapphire, n_d 1.768, whose face-on reflectance is R₀ = 0.0770 (optics.md §1). **Computed:** per face 0.923; two faces 0.852; an incoherent slab (1 − R) ÷ (1 + R) = 0.857. For crown glass (n 1.5168) R₀ = 0.0422. The row should name the material: with sapphire, T is about 0.85 per petal, not 0.92.
2. "Two petals pass the square of one" is the no-interreflection product. Stokes' pile-of-plates result, in ray optics, is τ_N = τ₁ ÷ (τ₁ + N(1 − τ₁)) (**Cited**, Lu, Miniatura and Englert, arXiv 0907.5557, eq. 4, after Stokes, Proc. R. Soc. 11, 545, 1862). With τ₁ = (1 − R) ÷ (1 + R) this is (1 − R) ÷ (1 + (2N − 1)R) (**Computed**, algebra). For sapphire it gives 0.857, 0.750, 0.666, 0.600, 0.545, 0.500 for N = 1 to 6, against the plain power 0.857, 0.734, 0.629, 0.539, 0.462, 0.396. The test "two petals pass the square of one" holds for the model as written; the row should say MODEL (no interreflections) or adopt Stokes. Our reading: keep the power law for the test, because it is what a Beer-Lambert stack gives, and note the 2 to 21% difference from Stokes in the handover.
3. Absorption: clarity 0.96 over 80 px gives σ = 5.10 × 10⁻⁴ per px; a petal path of 12 to 22 px passes 0.994 to 0.989 (**Computed**). Absorption is almost nothing: the darkening is all Fresnel.

**What the kit does today (Our reading from the code).** A glass layer snapshots the scene once at its start and every piece in it refracts that snapshot (`LayerOptions::glass`, `shape.wgsl` `optic_glass`). So overlapping petals in one layer do not compound: depth k is not produced by drawing k pieces. And pieces in one `Shape::Blob` group merge by smooth union (optics.md §2), which would melt the rosette into one outline: **petals must never share a blob group**, which also keeps this dash apart from `LiquidMorph`. Two routes for T^k:
- **(a) recommended:** draw the light under the flower, then draw each petal's disc with `BlendMode::Multiply` at colour T, so depth k multiplies by T^k exactly, then one glass layer of N separate discs for the rims and the bending. The overlay multiplies sRGB-encoded values; **Computed:** with T = 0.852 the encoded product gives 0.7264, 0.6197, 0.5290, 0.4520, 0.3865 for k = 2 to 6 against linear 0.7259, 0.6185, 0.5269, 0.4489, 0.3825, an error under 0.4 points. The cap at six needs a floor colour drawn under the multiplies, or a check that it never binds where it shows (next point).
- **(b)** one glass layer per depth: correct optics but up to N backdrop copies, against a 0.3 ms budget. Not recommended.
- **Where the cap binds (Computed):** outside the gear core (r > 95) the depth seen never passes 6 once d ≥ 0.35R, for any N. It passes 6 only when the flower is nearly closed in gears 7 to 10 (d ≤ 0.25R: N = 10 shows 7 at d = 0.25R and 10 at d = 0.1R). So the cap mostly guards the closed disc at idle in a high gear.

**The kit's contact model counts one face.** A piece in optical contact with the plane under it reflects only at its top face (optics.md §4, §8), so a single kit petal passes 0.923, not 0.852. Route (a) sets T itself; the board must state which value it uses.

**Layout arithmetic (Computed from the brief).** In the kit's local frame (y down, cockpit envelope 1440 × 560, from −320 to +240 about the anchor), the flower spans −230 to +210 and the arc's top is at −256; beads at y −296 ± 14 sit 26 px above the arc. The arc ends at 8 and 4 o'clock at (±213, +113). The arc clears the open flower by 26 px. If the brief's "up positive" also applied to the beads' y, they would sit at +282 to +310 and leave the envelope, so the brief mixes two conventions (open question 1).

**Upshift drop (Computed).** After an upshift d ÷ R = (k · shift − idle) ÷ (shift − idle) for an rpm ratio k: 0.718 for k 0.75 at shift 7,000 and idle 800; 0.605 for k 0.65; 0.771 for a 12,000 rpm engine at k 0.8; 0.640 for a 4,500 rpm diesel at k 0.7. The flower closes by about a quarter to two fifths, not by half as the brief says.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Petal opening d | `rpm`, `idle_rpm`, `shift_rpm` | verified | d = R · clamp((rpm − idle) ÷ (shift − idle), 0, 1), on the `needle` spring with feed-forward from `rpm_rate` |
| Petal turn ψ | d, gear | verified | (360° ÷ N) · d ÷ R, carried by the `snap` spring across a gear change |
| Petal count N | `gear` | verified | Forward(n) gives n (1 to 10); Neutral the closed disc; Reverse one petal at 270° |
| Light under the flower | `throttle` | verified | brightness floor plus throttle, on the `default` spring |
| Overlap depth | geometry | computed | T^min(k, 6), route (a) |
| Petal rims in `shift` colour | rpm ≥ shift | verified | steady, never flashing |
| Ten beads | `shift_fraction` | verified | fill outside in; strobe from the shift point |
| Arc (tach) | `rpm_fraction` | verified | 240° over the top, numerals from the car's max rpm |
| Speed | `speed`, `unit` | verified | tabular, hysteresis 0.6 |
| G-dot and petal parallax | `g_lat`, `g_long` | verified (smoothed) | `gBall` spring; 2 and 4 px per g |
| Studio light swing | `g_lat` | verified | family rule, 25° at 1.5 g |
| EV detection (one-petal case, section 6) | `cylinders` | decoded, offset confirmed at check C | 0 means an EV; fallback one forward gear with idle 0 |
| Lap summary (item 3 (c) below) | `race_progress` (last_lap, lap_number) | decoded, offset confirmed at check C | the last lap time under the speed for 3 s after each lap line, races only |
| Rumble glints (item 2 (c) below) | `rumble_strip` | decoded, offset confirmed at check C | a bead glint per rumble-strip contact; not recommended (motion off the rev axis) |

**Benchmark parts the 88 fields could now feed.** None of the benchmark's parts below is in the 88 fields: the decoder adds no breath, heart rate or camera data, so the list stands. Two items gain an option from decoded fields (2 (c), 3 (c)); the rest of the dash keeps to the long-verified fields, which is what the Abstract line's "calm at steady input" wants.

**What the game cannot send** (parts of the benchmark with no telemetry), each with a replacement:

1. **Breath-rate guide and the "Inhale" and "Exhale" prompts.** (a) **Recommended:** at the shift point a ghost rim of petal N + 1 appears where it will unfold, steady, so the flower itself says "shift" the way the Watch says "inhale". (b) A nano label on the arc that reads the learned speed per 1,000 rpm of the gear, from speed ÷ rpm, held per gear (derived).
2. **Haptic taps that start fast and fade at the top of the breath.** (a) **Recommended:** the beads are the taps: each new bead lights with a 1-frame glint (tokens `rimFlash` 60%), so the glints come faster as the revs climb and stop at the top, where the strobe takes over. (b) Fine ticks on the arc that thin out towards the shift point. (c) The game's own haptic source: a glint on the nearest bead when a wheel touches a rumble strip (`rumble_strip`, decoded, offset confirmed at check C). Not recommended: it moves the figure for a reason other than the revs.
3. **Heart-rate summary and Mindful Minutes at the end.** (a) **Recommended:** a peak-hold tick on the arc for the last pull's highest rpm, cleared at the next pull. (b) The last 0 to 100 km/h (0 to 60 mph) time from speed and the timestamp, shown in Titillium under the speed when the car stops (derived, labelled). (c) In a race, the lap is the session: the last lap time (`race_progress`, decoded, offset confirmed at check C) under the speed for 3 s after each lap line.
4. **Follow Your Breathing (camera sensing of the breath).** (a) **Recommended:** nothing to fake: the engine is the breath, and the flower follows it exactly. (b) Steer turns the whole flower by up to a quarter of 360° ÷ N, on the `default` spring.
5. **The room darkening and the Environments (beach, moon).** (a) **Recommended:** a bespoke studio panorama for the rims to mirror (section 8), so the flower sits in its own calm room. (b) At a standstill (speed 0, idle) the light under the flower drops to its floor and the flower is closed: the dash dims itself.
6. **Petals that fly out round the viewer (Vision Pro).** (a) **Recommended:** the new petal on an upshift materialises by thickness from its neighbour, so it rises towards the viewer. (b) The far petal depth's parallax doubles for 300 ms after an upshift.
7. **Narration and sound.** (a) **Recommended:** no audio in a visual dash; the guide's role goes to item 1 (a). (b) The Watch's Reflect colours (**Cited**, 9to5Mac, "a fluid, abstract wallpaper flowing between different colors"): under the colour law, hue by depth k from the palette ramp, on the figure only.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| Max rpm 0 or tiny, shift ≤ idle | d = 0 (closed flower), no NaN; the arc shows no numerals |
| Idle near max | the clamp keeps d in [0, R]; the flower snaps fully open on a blip, which is correct |
| 12,000+ rpm and 4,500 rpm diesels | the scale is the car's own; the upshift closes the flower by 23% to 40% (section 4) |
| EV (one gear, idle 0, odd rpm) | known by `cylinders` 0 (decoded, offset confirmed at check C), with one gear and idle 0 as the fallback test; one petal always; d from rpm with idle 0; no gear-change moment |
| Cylinders read but odd (negative, above 16) | ignored: the petal count comes from the gear only, so a wrong cylinder count changes nothing but the EV test, which then falls back |
| No laps (free roam, best_lap and last_lap 0) | the lap summary option (item 3 (c)) shows nothing; a last_lap of 0 is never shown as a time |
| Rumble strip held for long (the rumble option, item 2 (c)) | at most one glint per contact, at least 400 ms apart (under the WCAG 2.3.1 three flashes a second), and none during the limiter strobe, so the beads never flicker from a kerb |
| Reverse | one petal pointing down, d from rpm |
| Neutral | the closed disc: neutral has no petals to part, so a blip in neutral does not open it; only the light under it follows throttle and the beads and arc follow rpm |
| Gear above 10 | `Gear::from_raw` gives Neutral for anything above 10, so N ≤ 10 always |
| mph and km/h | `unit` from `Signals`; numerals tabular |
| hp and kW | not shown by this brief |
| Limiter bounce (about 50 rpm a frame) | d is clamped at R, so the figure is still; rpm digits (none here) would need `Hysteresis`; the beads use the strobe clock, not rpm |
| Crash spikes, huge G | parallax and G-dot clamp at 1.5 g; the light swing is clamped by the family rule |
| Airborne | rpm can spike: d clamps; nothing else reacts |
| Stale or paused stream | the app hides the dash; on unpause no ignition sweep |
| Palette extremes and random palettes | the petal tint leans to `primary` but the overlap darkening is neutral; rims use `shift` only at the shift point |
| Font swap | gear, speed and arc numerals come from the font roles; the core size is fixed |
| Cockpit view only | no hood variant in the catalog |
| Reduced flash | the strobe becomes the 2 Hz breath (40% depth); the bead glints and the ghost-petal fade are left out |
| Colour-only cues and colour vision | the shift point is shown by the held-still open flower, the ghost petal and the beads, not only by the `shift` rim colour |
| Second gear at the shift point | the two petals touch only at the centre, behind the core: expected, not a fault |

## 7. Signature moments

**Limiter.** (A) **Recommended:** only the ten beads strobe (10 Hz, 50% duty, the clock starting lit at the shift point; a 2 Hz breath in gentle mode). The flower is held at d = R, still, with steady `shift` rims and the ghost of petal N + 1: the held breath of MacStories' description. (B) The beads strobe and the petal rims' studio light leans 6° towards the viewer: a steady change. Fusing the beads into one bar is `LiquidGlass`'s moment and is left out.

**Gear change.** (A) **Recommended:** on an upshift a new petal unfolds out of its neighbour (it starts concentric with petal N − 1 and the angles spread from 360° ÷ N to 360° ÷ (N + 1) on `snap`) while the revs fall and the flower closes by a quarter to two fifths; it materialises by thickness. The numeral follows the tokens (outgoing 50 ms to 0.92, incoming from 1.12 on `fast`). (B) The patent's counter-rotation: as the flower closes after a shift it turns back, so an upshift reads as an exhale. (C) A downshift folds the last petal back into its neighbour.

## 8. What it needs from the 4K library

The pre-build (PR #18, `PREBUILD.md`) made **no kit** for this dash: it "reuse[s] the existing `liquid_glass` studio plate (seed 77203)", and its petal lens "is optical glass, which the kit computes from the outline". Reusable now: `assets/clusters/liquid_glass/studio_env.png` (Blender studio, octahedral 4096²) and its ComfyUI panorama, and the kit's analytic studio as fallback. The Saira, Saira Condensed and Titillium Web fonts are in `assets/fonts`.

The cluster prompt asks every dash for at least one bespoke ComfyUI and one bespoke Blender asset, so:

LIBRARY GAP: a bespoke ComfyUI studio panorama, 4096 × 2048, seamless left to right, a calm dim room for the rims to mirror (it replaces the Environments of Vision Pro); ComfyUI SDXL, softened on the cyclorama like the pilot's.
LIBRARY GAP: a bespoke Blender studio environment, octahedral 4096², lit with the family's horizon ring, key strip and far strip, carrying that panorama; Blender Cycles, as the pilot's recipe.
LIBRARY GAP: a Blender bake of one petal blank's normal and thickness maps, 4096², if the board gives the petal anything other than the kit's profile (a slight dimple or lens face); otherwise not needed, because the kit computes the surface from the outline.

## 9. Departures and open questions for the board

1. **The y convention.** The brief measures angles "with up positive" but places the beads at y −296, which only fits the envelope with y down. The board should give every position in the kit's y-down frame and say that φ = 90° points up.
2. **Which T.** Sapphire per petal is 0.852 (two faces) or 0.923 (the kit's one face); the brief's "4% at n = 1.5" is crown glass. Pick one and fix the catalog row.
3. **Darker overlaps, not additive ones.** We follow Apple's patent (B5), not the rebuild's screen blend. The light under the flower is needed, or the glass shows only rims on black.
4. **How big the light under the flower is.** A lit disc of radius 220 is a large lit area for the colour budget; the board should try it at a low floor with throttle on top.
5. **The upshift closes by 25 to 40%, not by half** (section 4).
6. **N = 2 has no overlap at the shift point.** The direction check looks at gears 1 and 2 first, as the brief says.
7. **Petals never share a blob group** (would melt into `LiquidMorph`); and against `ApertureNine` (blades round a centre), `KaleidoscopeMirror` (sectors twice the gear), `VisionSpatialGlass` (stacked panes with parallax) and `LiquidGlass` (drops over a rod), the thumbnail must read as one ring of equal round lenses.
8. **The Watch's turn amount and the shipped overlap look are unconfirmed**; step 1 measures them from Apple's video.

## 10. Sources

**Apple**
- US 11,069,255 B2, "Fluctuating progress indicator", Apple Inc., filed 27 Jan 2017, priority 10 Jun 2016: https://uspto.report/patent/grant/11069255 and https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11069255
- US 11,033,708, "Breathing sequence user interface" (same priority date; the text read held no figure details): https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11033708
- Apple Newsroom, "Apple previews watchOS 3", 13 Jun 2016: https://www.apple.com/newsroom/2016/06/apple-previews-watchos-3/
- Apple Newsroom, watchOS 8, 7 Jun 2021: https://www.apple.com/newsroom/2021/06/watchos-8-brings-new-access-connectivity-and-mindfulness-features-to-apple-watch
- Apple Support, Breathe on Apple Watch: https://support.apple.com/en-jo/guide/watch/apd371dfe3d7/26/watchos/26
- Apple Support, Mindfulness on Apple Vision Pro: https://support.apple.com/guide/apple-vision-pro/devae0eb1c57/visionos

**Press and reviews**
- T. Hardwick, MacRumors, 20 Jun 2016: https://www.macrumors.com/2016/06/20/watchos-3-apple-watch-breathe-app/
- B. Heater, TechCrunch, 31 Jan 2024: https://techcrunch.com/2024/01/31/apple-vision-pros-secret-weapon-mindfulness
- J. Krol (TheStreet), via Blue Mountain Eagle, 12 Feb 2024: https://bluemountaineagle.com/?p=228339 ; excerpt at https://macdailynews.com/2024/02/12/apple-vision-pros-mindfulness-app-is-impressive
- D. Dundee, "visionOS 2: The MacStories Review": https://www.macstories.net/stories/visionos-2-the-macstories-review/6/
- M. Potuck, 9to5Mac, 10 Nov 2021: https://9to5mac.com/2021/11/10/how-mindfulness-app-works-apple-watch-watchos-8/
- Wikipedia, "Mindfulness (Apple)": https://en.wikipedia.org/wiki/Mindfulness_(Apple)

**Rebuilds, geometry and optics**
- G. Graham, "Recreating the Apple Watch Breathe App Animation", CSS-Tricks, 20 Nov 2017: https://css-tricks.com/?p=262519
- Wolfram MathWorld, "Seed of Life": https://mathworld.wolfram.com/SeedofLife.html ; Wikipedia, "Overlapping circles grid": https://en.wikipedia.org/wiki/Overlapping_circles_grid
- Y. Lu, C. Miniatura and B.-G. Englert, "Average transmission probability of a random stack", arXiv 0907.5557 (2009), eq. 4: https://ar5iv.labs.arxiv.org/html/0907.5557 ; G. G. Stokes, "On the intensity of the light reflected from or transmitted through a pile of plates", Proc. R. Soc. 11, 545 (1862): https://scilit.net/publications/e67bf003068afab513ba74795814ef16
- `docs/specs/optics.md` §1 (sapphire R₀ 0.0770), §2 (smooth union), §4 and §8 (contact model, one face), §6 (Beer-Lambert)
