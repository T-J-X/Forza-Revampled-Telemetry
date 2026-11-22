# RESEARCH: what makes a cluster feel premium

Phase 3 reference synthesis for FH6 WheelDash v2, 2026-11-20. Method: `design:research-synthesis` (evidence, then themes, then principles), adapted from user interviews to visual-reference research. Six parallel researcher passes fed it.

**Executive summary**
1. Premium is discipline first: colour only for state, one accent, sparse symbols over the moving scene, an idle that holds still (Tesla, GT7, Porsche, Oblivion, The Expanse).
2. Race hardware shares one lamp layout: 10 RGB shift pods on top, alarm lamps at the sides, per-gear thresholds, warnings over shift. No vendor publishes a flash rate, so WCAG 2.3.1 sets our strobe limits.
3. Materials read as real when light comes from geometry and moves (Apple's travelling specular, a faceted hand that swaps brightness as it sweeps). Smoothed G is our light driver.
4. Depth comes from stacked layers (skeleton plates, domed sapphire, a light blade under a lens), never a box with a shadow. Our glass lenses only our own layers, so tint, rim and caustic carry it.
5. Makers publish almost no fonts, hex values or timings: every est. needs a frame grab, and every timing is a proposal on Material 3 tokens and closed-form springs.

## How to read this

- **Cited**: a source states it; the URL is in the row. "(snippet)" = only a search excerpt was seen; "(summary)" = a search summary across pages. Verify both before quoting.
- **est.**: estimated from memory of published images or footage. No frame was measured in any pass, so every est. is a low-confidence starting value for a frame-grab check.
- **prop.**: our proposal for the overlay, not a claim about the reference. **derived**: arithmetic from cited inputs.
- **Fonts**: unless a source names a face, the face given is the closest OFL match (est.).
- **Coverage**: all six researcher notes arrived; none failed. Pages that returned 403, 404 or a paywall are flagged in their rows.
- **Display constants** (derived): the 55-inch panel is 1.218 m wide, so 1 px = 0.317 mm; at 0.8 m, 1° ≈ 44 px.

---

## 1. Racing-game HUDs, ADAS visualisation and aviation HUDs

### Gran Turismo 7 (Polyphony Digital)
**Premium because:** one compact meter where the gear is the largest glyph, red means only "limit" or "intervening", and HUD density is the user's choice.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Shift indicator lights as revs near the red zone | Position not stated in text | Cited: [GT7 manual, Race Screen](https://gran-turismo.com/us/gt7/manual/race/02) | The top-crown pod row (section 3) |
| 3-state assist colour code | On white, off grey, intervening red | Cited: same page | TC/ABS/STAB micro-labels `#E8ECF2` / `#5A606B` / `#FF3B30` (prop. hex) |
| HUD density setting, including "Race Info Only" | Strips the lower HUD graphics | Cited: [GT7 manual, Display](https://gran-turismo.com/us/gt7/manual/drivingoption/05); [Gamepur](https://gamepur.com/?p=309437) | Density tiers Full / Race / Zen as one token |
| Meter proportions and face | Speed cap 0.5–0.6× gear; thin segmented rev strip; white ≈90% on a dark plate; squared wide tabular face, identity unknown | est. | Modern Digital ratios; test Titillium Web SemiBold and Saira SemiExpanded against frame grabs |
| Rev bar only approximates the best shift point for most cars | Community claim | Cited: [GTPlanet telemetry thread](https://www.gtplanet.net/forum/threads/overview-of-gt7-telemetry-software.418011/post-14567882) | Shift thresholds come from per-car data |

### ACC, Le Mans Ultimate and Forza
**Premium because:** ACC's tyre colours are the sim-racing lingua franca, and serious sims treat the shift point as per-car data. Forza is not the bar.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| 4-step tyre temperature | Blue cold, green optimum, orange hot, red overheated | Cited: [Traxion.gg](https://traxion.gg/why-tyre-pressures-are-key-to-winning-in-asetto-corsa-competizione/) | Discrete steps with a 150 ms crossfade (prop.); bands per compound |
| Per-car shift points from third-party tools and LMU LED vendors; ACC's own bar and third-party bars disagree on colour | — | Cited: [Sidekick](https://www.overtake.gg/downloads/sidekick.11007/); [Daniel Newman Racing](https://www.danielnewmanracing.com/change-logs); [Race Element](https://www.overtake.gg/downloads/race-element.50578/updates?page=6) | `car_profiles.json` thresholds with a telemetry-redline fallback; our tokens set colour |
| LMU: six core HUD elements, each toggleable | — | Cited: [gameplay.tips](https://gameplay.tips/guides/le-mans-ultimate-the-in-game-hud-guide.html) | Every rank 4–5 element can be toggled |
| Forza Motorsport (2023): HUD changes not planned for cost (undated reply); players built SimHub tyre HUDs. No sourced FH5 HUD spec | — | Cited: [Forza forums](https://forums.forza.net/t/forza-ui-multiplayer-qol-changes-sample-picture-inside/719086); [SimHub thread](https://forums.forza.net/t/tire-temp-wear-hud-for-simhub/643080) | Tyre state is an unmet need we can fill |

### Tesla FSD visualisation
**Premium because:** it is a state machine, not scenery: a grey world, one saturated hue for objects in the planned path, red only for imminent action, and ego state shown on the ego model.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Two-tone path line (since Beta 10.8) | Darker blue where it will accelerate; faded where it will stop | Cited: [Not a Tesla App reference](https://www.notateslaapp.com/tesla-reference/636/all-tesla-fsd-visualizations-and-what-they-mean.) | 64-segment path carpet, alpha 0.55 at ego → 0 at horizon; beyond d = v²/(2·a_brake), 35% of near alpha (prop.) |
| Object semantics | Grey default, dark grey lead, blue in or entering path, red for immediate action; cones orange | Cited: same | Neutral `#8C929C` at 70%, accent `#3BA7FF` (both est.); at most 2 saturated objects (prop.) |
| Proximity arcs | Grey → yellow → red as distance closes | Cited: same | 3-step blind-spot arcs, 150 ms crossfade (prop.) |
| Vector lanes (10.11); vehicle models stretched to measured size (10.10.2) | — | Cited: same | Lanes and rims as SDF; one ego mesh scaled per class |
| Blue underglow shows FSD is driving (v14.1) | — | Cited: [Tesla Oracle](https://www.teslaoracle.com/2025/10/11/tesla-improves-fsd-v14-driving-visualization-to-indicate-autopilot-in-use-video/) | Additive SDF ellipse, Gaussian σ ≈ 0.3 car width: cyan nominal, amber TC/ABS, magenta limiter (prop.) |
| Chase camera | 20–30° down-pitch, ego in lower third, vertical FOV 30–40°, untextured matte ground | est. | Our eye (0, 1.85 m, −3.4 m) with vertical FOV ≈ 35° (prop.) |

### 3D-scene and AR peers (Rivian, Mercedes AR-HUD)
**Premium because:** symbology over a moving world stays large, sparse and few-coloured.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Rivian builds its display in Unreal, the car model reacting live to sensors; Gen 2 swaps an Autonomy view with Maps | — | Cited: [Unreal spotlight](https://www.unrealengine.com/en-US/spotlights/rivian-s-r1t-features-state-of-the-art-hmi-designed-developed-and-deployed-in-unreal-engine); [Rivian 2025.46](https://stories.rivian.com/software-update-2025-46) | Ego glTF reacts to brake, yaw and slip; ADAS ↔ road map share one rank-4 slot |
| Mercedes AR-HUD | 10° × 5° window, image at ≈10 m, 77-inch equivalent, TI DMD | Cited: [MBUSA S-Class release](https://media.mbusa.com/releases/the-new-mercedes-benz-s-class-2021) | Sparse, large symbols over the live cockpit |
| AR navigation: large blue floating arrows, judged slow at roundabouts | — | Cited: [CAR](https://carmagazine.co.uk/car-news/tech/mercedes-augmented-reality-navigation-how-does-it-work) (paywall, snippet) | Graphics must never lag the scene |

### Collimated aviation HUD (F/A-18, F-16, F-35 HMD)
**Premium because:** thin monochrome strokes float on the combiner, and the boresight/flight-path pair shows a difference as a gap you read without numbers.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Collimated image at infinity, monochrome green | Phosphor `#3CFF7A` | Cited: [Wikipedia, HUD](https://en.wikipedia.org/wiki/Head-up_display); hex est. | HUD mode: one hue, amber `#FFB020` cautions (prop.) |
| Flight-path marker against a fixed boresight; FPM is conformal | — | Cited: Wikipedia; [NASA NTRS](https://ntrs.nasa.gov/api/citations/19900008316/downloads/19900008316.pdf) | Drift FPM offset by β = atan2(v_lat, v_long), ≈14 px/°, clamped ±35°, hidden below 5 m/s with a 120 ms fade (prop.) |
| Fixed tapes (speed left, altitude right); luminance adjustable from glare to night | — | Cited: Wikipedia | Fixed slots per rank; one brightness scalar |
| Asymmetric pitch ladder improves bank recognition; pilots preferred a full-width horizon (no measured gain) | — | Cited: [Human Factors Society 1989](https://journals.sagepub.com/doi/ref/10.1177/154193128903302008); [DTIC ADA331767](https://apps.dtic.mil/sti/tr/pdf/ADA331767.pdf) (summary) | Signed bars: positive solid, negative dashed; 1 px full-width crown hairline as rev-bar baseline |
| Stroke glyphs, no fills | 2 px core, 1 px SDF AA, 6–8 px halo at 25% | est. (convention); values prop. | HUD-mode material |

## 2. Production clusters 2025–27 and Apple Liquid Glass

### Apple Liquid Glass (iOS 26 / visionOS)
**Premium because:** separation comes from lensing and from light that moves with the geometry, not from frost or opacity.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Lensing (bending light), not scattering, defines the material; bigger elements act as thicker glass | — | Cited: [WWDC25 "Meet Liquid Glass"](https://developer.apple.com/videos/play/wwdc2025/219/) | We cannot read game pixels, so we lens our own layers: `uv += n·k·(1 − smoothstep(0, w, d))²`, w 24–40 px, k scaled by size (prop.) |
| Highlights come from environment lights, follow geometry, travel round the silhouette, can follow device motion | — | Cited: WWDC25; [Apple Newsroom](https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/) | Light vector L from smoothed G (critical spring ≈150 ms); rim `pow(max(dot(n, L), 0), 24..48)`, peak 60–80% white (prop.) |
| Clear variant needs a dimming layer and bold bright content; local dimming on small items; never mix Regular and Clear | — | Cited: WWDC25 | Feathered black pad behind gear and speed, alpha ∝ (1 − scrim %) (prop.); one variant per cluster |
| Elements materialize by ramping lensing, not opacity | — | Cited: WWDC25 | Show: 200 ms lensing and rim ramp (prop.). Hide: 1 frame (directive 9) |
| Touch glow starts at the contact point and spreads to nearby glass | — | Cited: WWDC25 | On upshift, glow leaves the gear glyph and runs along rims at ≈4000 px/s (prop.) |
| iOS 26.1 "Tinted" raises opacity and contrast; accessibility modes frost the glass, add borders or cut motion | — | Cited: [MacRumors](https://www.macrumors.com/2025/10/20/ios-26-1-liquid-glass-toggle/); [TidBITS](https://tidbits.com/2025/10/21/ios-26-1-to-add-optional-opacity-to-liquid-glass/); WWDC25 | Tinted setting = top of alpha 28..64; OSD accessibility toggles |

### Porsche 911 (992.2) and Taycan facelift
**Premium because:** heritage geometry rendered digitally, with the scale rotated so the needle stands vertical at the shift point, and a single contrast colour.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| 12.6-inch curved cluster, up to 7 designs; "Classic" view has five round dials around a centre rev counter; white-on-black faces (est.) | — | Cited: [Porsche Newsroom, 911 Interior](https://newsroom.porsche.com/en/press-kits/911/Interior.html); [Top Gear](https://www.topgear.com/car-reviews/porsche/911-0/interior) | Classic Analog layout grammar |
| Rev-counter mode rotates the scale so the needle is at 12 o'clock at shift speed | — | Cited: Porsche Newsroom | θ = θ_top + (rpm − rpm_shift)·sweep / rpm_range, per car (prop.) |
| GTS contrast colour (Carmine Red or Slate Grey Neo) only on the Sport Chrono and rev counter | — | Cited: Porsche Newsroom | One contrast hue, on the tach only |
| Porsche Next brand face, three weights | — | Cited: [Porsche Design System](https://designsystem.porsche.com/v4/must-know/typography/) | Not OFL; use Titillium Web or Saira SemiBold with `tnum` (est. match) |
| Taycan 3D assistance view: up to 3 lanes and 6 road users | — | Cited: [Taycan press kit](https://newsroom.porsche.com/en/press-kits/taycan/Die-Driver-Experience-und-die-Connectivity.html) | Hard cap for our ADAS scene |

### Lucid Gravity Clearview Cockpit
**Premium because:** a bezel-free OLED that seems to float high in the sightline, plus a "Zen" theme. That is our exact situation.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Curved 34-inch 6K OLED that seems to float above the dash, mounted high for minimal eye movement | — | Cited: [Lucid](https://lucidmotors.com/stories/introducing-lucid-gravity-touring); (summary) [Top Gear](https://www.topgear.com/car-reviews/usa/lucid/gravity-us/interior), [Motor1](https://www.motor1.com/reviews/756565/2025-lucid-gravity-first-drive/) | No full frame; edges come from rims and content falloff |
| Zen theme shows only speed, odometer and turn signals; the turn signal brings up a blind-spot camera. Top Gear criticises the UI's cognitive load | — | Cited (summary): [Electrek](https://electrek.co/2026/06/23/lucid-gravity-2026-review/); Top Gear | Zen density tier; proximity arcs only on events |

### BMW Panoramic iDrive (Neue Klasse iX3)
**Premium because:** one dark horizontal band is the information surface, and an upper HUD layer is choreographed with it.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Panoramic Vision runs A-pillar to A-pillar, projected on a black print band | — | Cited: [BMW Group Newsroom](https://newsroom.bmwgroup.com/americas/new-bmw-panoramic-idrive-revolutionizes-vehicle-operation) | One feathered dark band behind the `WindshieldHud` crown: alpha 0.55–0.75, 40–60 px feather (prop.) |
| Key information in the driver section; optional 3D HUD above takes alerts while the band takes detail; "shy-tech" buttons light only when available | — | Cited: same | Urgent cues (strobe, braking zone) upper, context lower; micro-labels appear only while active |
| Greeting spreads driver → passenger; My Modes change content, colour, light and sound together | ≈600 ms, ease-out cubic | Cited: [BMW iX3 premiere](https://newsroom.bmwgroup.com/global/the-start-of-a-new-era-the-new-bmw-ix3); [BMW iX3 world premiere](https://newsroom.bmwgroup.com/global/world-premiere-bmw-ix3-en); timing prop. | Boot reveal spreads outward from the gear; a theme is one token set across all layers |

### Mercedes MBUX Hyperscreen and CLA Superscreen
**Premium because:** parametric styles, including a needle described as a "digital laser sword in a glass lens", and a game engine for 3D.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| "Sporty" became a 3D performance bar (Drive, Accelerate, Charge) | — | Cited (summary): [Mercedes-Benz Canada](https://media.mercedes-benz.ca/releases/release-c6ffb9918e70966e2c6964edd30527f4-the-eqs-the-interior-design) (404 on fetch) | One bar, hue by phase: cyan power, white coast, amber braking (prop.) |
| "Classic" twin dials as a light blade inside a glass lens | Needle capsule 3–4 px, HDR core ≈2.0, 2-pass bloom | Cited (summary): [Hyperscreen page](https://group.mercedes-benz.com/innovation/digitalisation/connectivity/mbux-hyperscreen.html) (403); values prop. | SDF capsule under a convex glass disc with a fresnel rim |
| CLA runs Unity real-time 3D (≈3 s init, vendor claim) and merges 3D surroundings with route guidance in one view | — | Cited: [Unity](https://unity.com/resources/next-gen-car-experiences-mercedes-cla); (summary) [T3](https://www.t3.com/auto/vehicle-tech/mbux-superscreen-on-the-mercedes-benz-cla-shooting-brake) | One rank-4 scene; measure and cache our pipeline cold start |

### Apple CarPlay Ultra (Aston Martin, May 2025)
**Premium because:** each cluster theme is a parametric matrix (bar weight, colour, wallpaper) co-designed with the carmaker.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Curated themes with user colours and wallpaper; sample themes use thin blue, multi-coloured, thick grey or thick red bars | — | Cited: [Apple Newsroom](https://www.apple.com/newsroom/2025/05/carplay-ultra-the-next-generation-of-carplay-begins-rolling-out-today/) (incl. captions) | `ClusterTheme` tokens: `arc_weight` {thin 6 px, thick 18 px}, `accent`, `wallpaper` {none, caustic, carbon}, `multi_segment` (prop.) |
| The Aston build felt too generic to one reviewer; SF Pro is not OFL | Inter, Barlow Condensed digits (est. match) | Cited: [InsideEVs](https://insideevs.com/news/759736/carplay-ultra-aston-martin-launch/) | Brand identity must survive the system look |

## 3. Motorsport displays

### F1 2026 (McLaren MCL40) and Porsche 963
**Premium because:** each pixel answers one racing question, and a signed bar vanishes when it has nothing to say.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Separate MGU-K bar above the gear: deploy grows centre → right, harvest centre → left, shrinks when derating, disappears when idle | — | Cited: [Autosport](https://www.autosport.com/f1/news/how-mclaren-has-tweaked-its-steering-wheel-to-maximise-f1s-2026-hybrid/10798146/) | Signed bar above the gear for longitudinal G, 8–12% of gear height, spring ≈120 ms, fades out near zero (est.) |
| Shift LEDs light progressively | ≈15 LEDs, green → red → blue/purple; gear 35–50% of screen height | Cited: [evo India](https://www.evoindia.com/article/f1-steering-wheel); values est. | Pod row (below) |
| 963: shift and warning lights across the top; colour screen shows messages, lap times and gear | — | Cited: [Motorsport.com](https://www.motorsport.com/imsa/news/porsche-963-driving-manual-daytona/10412256/) | Shift and alarm lamps share one crown band |

### Bosch DDU 10 / 11, AiM MXG Strada, MoTeC, Cosworth CCW
**Premium because:** a fixed, honest lamp layout where every LED has a threshold, a priority and an intensity.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Bosch DDU 10 / DDU 11 | 800×480; 10 top + 5 per side RGB LEDs (DDU 11: 8 top); 6-step brightness; event-triggered page changes | Cited: [Bosch DDU 10](https://www.bosch-motorsport.com/products/displays/display-ddu-10/); [DDU 11](https://www.bosch-motorsport.com/products/displays/display-ddu-11/) (a retailer counts 13 LEDs) | 10 top shift pods, side alarm columns; an event swaps the rank-4 slot |
| AiM MXG Strada: 10 shift + 8 alarm RGB LEDs; per-LED rpm threshold by gear; predictive mode greens the bar when gaining, reds it when losing | 7 in, 700 cd/m² | Cited: [AiM MXG Strada](https://www.aim-sportline.com/en/products/mxg-strada); [AiM shift lights](https://aim-sportline.com/us/products/mx-series/shift-lights-and-alarms.htm) | Per-gear stage tables from `EngineMaxRpm` and `Gear` (first pod ≈75–80% of shift rpm, est.); delta mode below the first threshold |
| MoTeC: colour, function and intensity programmable per LED; SLM-C ranks warnings over shift stages. Cosworth Mk3: 10 shift + 6 alarm | — | Cited: [MoTeC D175](https://www.motec.com.au/products/D175); [SLM-C PDF](https://cdn.shopify.com/s/files/1/0332/8881/4637/files/CDS18122_SLM-C_Club_Shift_Light_Module.pdf); [Cosworth Mk3](https://www.cosworth.com/motorsport/products/ccw-mk3/) | Intensity is a uniform separate from hue (dimmer, OLED ABL); alarm slots outrank shift slots |
| Stage colours and limiter | Green `#00E040` → amber `#FFC400` → red `#FF1E1E` → whole row blue `#2060FF` or purple `#A040FF`; row flashes 4–10 Hz, ≈50% duty; unlit pods show as grey lenses | est. | Stage tokens; ghost pods at alpha 16–24; strobe per principle 1 |
| Pod optics | Rounded-rect pods, gap ≈30% of pod width, radius ≈35% of pod height; 2-stop radial (core 40% toward white, edge full hue); 1 px inner rim at ≈50% white; bloom on lit pods only | est. (LED-lens look); values prop. | SDF pod material for every shift row |
| Race page and type | Gear 45–55% of panel height, white on black; labels 25–30% of value height; bold condensed tabular grotesque | est. | Race Dash type scale (section 9) |

### Flash and peripheral-perception standards
**Premium because** (here, "safe because"): a strobe that respects human limits reads as engineered, not flashy.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| WCAG 2.3.1 | ≤3 flashes in any 1 s unless under the general and red thresholds; general flash = ≥10% luminance change; red = R/(R+G+B) ≥ 0.8; area ≤0.006 sr (≈25% of any 10° field) | Cited: [W3C](https://www.w3.org/WAI/WCAG21/Understanding/three-flashes-or-below-threshold.html) | Only a thin strip may strobe above 3 Hz |
| SAE J1133 allows 1–4 Hz warning flash; beacon detection at 1 Hz matched 4 Hz, intensity mattered more | — | Cited: [SAE J1133](https://saemobilus.sae.org/content/J1133_201106); (summary) [core.ac.uk](https://core.ac.uk/download/76992917.pdf) | Large-area alerts pulse at 1–3 Hz; contrast beats rate |
| Peripheral warnings can capture attention better than ones near the speedometer | — | Cited (abstract): [Behavior Research Methods](https://link.springer.com/content/pdf/10.3758/s13428-010-0054-8.pdf) | Rank-1 strobe on the top crown |
| Strip geometry | At 0.9 m a 10° field is ≈497 px across; 25% ≈48 k px². At 0.6 m: ≈331 px and ≈21.5 k px² | derived | A strip ≤60 px tall passes down to 0.6 m (331 × 60 ≈ 19.9 k px²) |

## 4. Analog horology and hypercar analog

### Bugatti Tourbillon (2024)
**Premium because:** real mechanism visible through the dial, on a fixed hub the wheel never hides.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Built by Swiss watchmakers Concepto: 600+ or 650+ parts (Bugatti gives both), ruby jewels, hand-finished needles; finishes include Clous de Paris, radial guilloché, aventurine | — | Cited: [Bugatti](https://newsroom.bugatti.com/press-releases/haute-horlogerie-to-the-bugatti-tourbillon); [Bugatti](https://newsroom.bugatti.com/press-releases/bugatti-an-enduring-and-remarable-tale-in-the-world-of-horology) | Finish menu for the horology line |
| Centre dial for speed and rpm with a small digital speed and gear low inside; satellites left and right | — | Cited: [Motor1](https://www.motor1.com/news/724009/bugatti-tourbillon-instrument-cluster-details/) | One hero dial plus 2 satellites |
| 4-layer skeleton stack | Gear plate `#8A8E93`, linearly brushed; bridges with a 2–3 px anglage band; cut-away ring; hands and glass; 1–4 px G parallax | est. | Depth without boxes |
| Live gear train; ruby jewels | Gears turn with rpm; jewels 6–10 px, radial `#5A0A14` → `#C0182B`, 2 px dot at 70% white | est. | Skeleton layer kept at 10–20% luminance for OLED care |

### Pagani Utopia and Koenigsegg Jesko
**Premium because:** machined metal and exposed movements (Pagani), and content that stays level while its frame rotates (Koenigsegg).

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Utopia dials are analog and reveal parts of their movements; small screen between two dials | — | Cited (snippet): [Motor1](https://www.motor1.com/news/735286/pagani-utopia-interior-technology-analysis/); [CarExpert](https://carexpert.com.au/car-news/pagani-utopia-revealed) | SDF apertures with 1 px chamfer, 50% white lit side and 15% shadow side (est.) |
| One car seen with a black anodised aluminium cluster | Base `#1A1B1E`, spun grain, 3 px polished 45° chamfer | Cited (snippet): [Hagerty](https://www.hagerty.com/media/?p=252194); values est. | Machined bezel material |
| Jesko: a gyroscope keeps the wheel display's key information upright; 4 corner icons stay level | — | Cited: [The Drive](https://www.thedrive.com/news/28249/300-mph-koenigsegg-jesko-hypercar-boasts-worlds-first-gyroscopic-gauge-cluster) | A roll-stabilised horizon in a sub-dial, spring ω ≈10 rad/s (est.); roll field to be verified against golden packets |

### Lexus LFA meter
**Premium because:** the instrument was made faster than any mechanism, and it behaves differently at idle and at speed.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| TFT tach because no needle could follow idle → 9,000 rpm in 0.6 s; Polyphony Digital helped design it | — | Cited: [evo](https://www.evo.co.uk/lexus/lfa/202607/lexus-lfa-rev-counter-art-of-speed) | The live needle must never trail telemetry |
| The main dial glides across the display on a menu call | Spring ω ≈12 rad/s, ≈400 ms | Cited: evo; values est. | Dial glide when the OSD opens |
| Sport mode: bigger, bolder numerals; face turns white; redline moves to the top. Near redline the dial is bathed in red | Red wash `#FF1E1E` from 92% of limiter rpm, up to 30% alpha | Cited: evo; values est. | Mode-driven type weight; tint, never a white face on OLED |
| Raw needle at idle; smoothed with an afterimage at speed | Raw below 90°/s; afterimage wedge 60% → 0 and up to 4 ghost needles above 180°/s | Cited (snippet; page 404): [Lexus mag](https://mag.lexus.co.uk/gauging-interest-with-the-lexus-is-f-sport-meter); values est. | Fixed `[f32; 4]` ring buffer, no allocation |

### Porsche centre tachometer (550 Spyder to 992)
**Premium because:** 70 years of putting rpm dead centre, with an operating band that tells you where the engine wants to be.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| 550 Spyder (1953): light-green numerals on black, white needle, green band 4,000–6,000 rpm, red zone above | — | Cited: [Porsche Newsroom](https://newsroom.porsche.com/en/2026/history/porsche-icon-christophorus-417-tachometer-41697.html) | Band 4% of R, `#7FD67A` at 35% from peak-torque to peak-power rpm; red `#E0262B` past redline (est.) |
| Numerals | `#A8E6A0` | est. | Saira Condensed SemiBold or Barlow Condensed Medium, `tnum` verified |

### Watch-dial finishing, faked in a shader
**Premium because:** every finish is a height or tangent field under a moving light. Even the hub: 2-stop radial `#2A2C30` → `#9AA0A8` with a 1 px inner rim at 60% white (est.).

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Clous de Paris: crossing grooves forming pyramids | 45° UV, period 10–14 px; h = min(\|fract(u/P) − .5\|, \|fract(v/P) − .5\|) | Cited (snippet): [Monochrome](https://monochrome-watches.com/technical-perspective-the-art-of-guilloche-dials-examples-of-guilloche-patterns-comparison-stamped-cnc-dials); maths derived | Bake to a mip-mapped normal map at load (kills WRGB moiré) |
| Sunburst: rays brushed centre to edge; anisotropy maps store strength (R) and rotation (G) | Radial tangent; aniso GGX α 0.6 along / 0.05 across; cheap form base + k·\|sin(θ − φ_L)\|^24 | Cited (snippet): [Patek Philippe](https://patek.com/en/company/savoir-faire/attention-to-details/dials); [Kitware](https://www.kitware.com/pbr-journey-part-2-anisotropy-model-with-vtk/); values est. | Dial faces; spun (tangential) grain for bezels |
| Super-LumiNova C3 glows green (515 nm), BGW9 blue-green (485 nm) | Night `#7DFF8A` / `#5FD8FF`; day `#E3EBC4` / `#F1F3EE`; bloom 6 px at 25% | Cited (snippet): [Wikipedia](https://en.wikipedia.org/wiki/Super-LumiNova); hex est. | Lume tokens |
| AR-coated sapphire reflects blue or purple at an angle, slightly brown face-on; domes reflect more | Crescent 18/255; fresnel (r/R)^4 to 40/255; rim tint `#7B6CFF`; refraction uv·(1 − 0.03r²) | Cited (snippet): [Fratello](https://www.fratellowatches.com/how-watches-work-antireflective-coatings-in-watches/); values est. | Domed glass stack inside alpha 28..64 |
| Dauphine facet swap; hand shadows constant in screen space | Facet normals (±0.7, 0, 1); light 135° / 40°; shadow 3 px (tach), 6 px (upper layer), σ ≈ h, 40% | derived; values est. | Signature analog moment |

## 5. Sci-fi film and game UI (FUI)

### Oblivion (2013): GMUNK, Jake Sargeant, Joseph Chan and team
**Premium because:** one dot grid holds everything, greeble is rationed, and the Bubbleship UI is "a hologram embedded within the spherical glass cockpit" ([GMUNK](https://gmunk.com/OBLIVION-GFX)).

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Function and minimalism; one graphic language; greeble controlled; one bright palette | — | Cited: [GMUNK](https://gmunk.com/OBLIVION-GFX) | One language per family |
| A dot grid to hang every element on; screens have startup and turn-off animations | 24 px pitch at 4K; 2×2 px dots at alpha 0.08, shown only at boot and in the OSD | Cited: [HUDS+GUIS](https://www.hudsandguis.com/2013/05/02/oblivion-interface-design/); [Pushing Pixels](https://www.pushing-pixels.org/?p=9032); values prop. | Layout lattice; boot and shutdown states |
| Line weights and palette | 1 px at 1080p (≈2 px at 4K); near-white `#EEF6FA`, accent `#8FD9F2`, alerts `#FF7A3D` | est. | Hairline structure |

### Tron: Legacy (2010) and Tron: Ares (2025)
**Premium because:** procedural line work under a strict rulebook, with depth from layered reliefs, and a boot that assembles sector by sector.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Custom procedural tools (Josh Nimoy, GMUNK, David Lewandowski at Digital Domain); real commands, no gimmicks | — | Cited: [Gizmodo](https://gizmodo.com/a-look-at-how-tron-legacys-awesome-software-art-was-mad-5788616); [Core77](https://www.core77.com/posts/18933/josh-nimoy-on-the-secrets-behind-the-special-effects-in-tron-legacy-18933) | Generated geometry, not stock greeble |
| Organic forms under a strict rulebook; depth from layered reliefs and orbiting data; boot dissolves "bit by bit", then sectors fill | — | Cited: [GMUNK, TRON](https://gmunk.com/TRON-Legacy) | Sector boot |
| Three-term neon light | Core near-white; halo `exp(−d/6 px)`; bloom `exp(−d/24 px)` at 0.12; cyan `#6FE9FF` vs orange `#FF9A2E` | est.; radii prop. | Neon SDF material, bloom only on ranks 1–3 |
| Tron: Ares symbol designed to deploy across many UIs | — | Cited: [Prologue](https://prologue.com/portfolio-item/tronares/) | Brand mark as a system element |

### Blade Runner 2049 (2017): Territory Studio
**Premium because:** material register carries status (degraded analogue for K, pure geometry for Wallace), and every screen is a 3-state machine.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| K's spinner: warping, ghosting, colour loss, glitch. Wallace: minimal black-and-white geometry. Morgue zoom steps through lens positions | Ghost copies 3–6 px at 20–30%; Wallace `#F4F4F0` hairlines | Cited: [Territory Studio](https://territorystudio.com/project/blade-runner-2049/); values est. | Degradation is a register or an event, never default; stepped, not smooth, transitions |
| Each screen has initial, action and loop states; legibility tested by scaling and blurring | — | Cited: [Engadget](https://www.engadget.com/2017-10-20-designing-the-technology-of-blade-runner-2049.html) | Blur gate in CI (principle 4) |

### Cyberpunk 2077 (CD Projekt Red)
**Premium because:** a stable UI that tears only when something happens, and squared 1980s dash numerals.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Mostly text HUD in an off-red lo-fi look; car dashes use thick square 1980s-style figures | Salmon `#FF5F57`, cyan `#5EF2FF` | Cited: [HUDS+GUIS](https://www.hudsandguis.com/home/2019/cyberpunk-2077); hex est. | Sci-Fi VFD styles |
| Mods replace the shipped UI face (Rajdhani, inferred) for legibility | — | Cited (indirect): [Nexus Mods 18623](https://www.nexusmods.com/cyberpunk2077/mods/18623) | Rajdhani 600–700 for short labels only; Chakra Petch or Orbitron for digits |
| Glitch as an event | RGB split 2–6 px at 1080p for 100–200 ms, slice displacement | est. | Event-only, 80–120 ms, never on the hero gear (prop.) |

### The Expanse and concept-car UIs
**Premium because:** colour as a faction code, no decoration-only elements, and a car that wakes up as a ritual.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Mars red, Earth blue, Belter orange-brown; red found hard to use because it grabs the eye; no giant flashing alert boxes (Rhys Yorke) | — | Cited: [Pushing Pixels](https://www.pushing-pixels.org/?p=18324) | Brand hue is the structure "faction"; red is reserved |
| Vision AVTR interior comes alive when a hand rests on the controller; Peugeot's 360° Halo Cluster was judged more impact than function | — | Cited: [CES](https://www.ces.tech/articles/daimler-presents-an-avatar-inspired-concept-car-at-ces-2020/); (snippet) [Driving.co.uk](https://driving.co.uk/news/inception-concept-hints-at-what-peugeots-will-look-like-from-2025) | Breath-in on `IsRaceOn` 0 → 1; spectacle must not cost function |

## 6. Fluid, generative and motion

### Real-time fluid and generative fields
**Premium because:** live physics that responds to the car, at a cost the frame budget can carry when clipped to the binnacle.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| WebGL Fluid Simulation defaults (MIT) and relief shading | Sim 128, dye 1024, pressure ×0.8 warm start, 20 Jacobi, curl 30; bloom 8 levels from 256, threshold 0.6, knee 0.7, intensity 0.8; `color *= clamp(dot(n, z) + 0.7, 0.7, 1)` | Cited: [script.js](https://raw.githubusercontent.com/PavelDoGreat/WebGL-Fluid-Simulation/master/script.js) | Clipped sim: RG16F velocity, 512 dye, 12–16 Jacobi in shared memory (est.), forced by G splats; relief as fake depth for one fetch |
| GPU Gems ch. 38: pressure needs 40–80 Jacobi from zero; no-slip and Neumann boundaries | — | Cited: [NVIDIA](https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu) | Keep the warm start; use `exp(−d·dt)` dissipation (derived) |
| Bandwidth wall | Full 4K RGBA16F read+write ≈133 MB ≈0.27 ms; an 1800×600 clip ≈0.035 ms | est. | Clip every full-resolution pass to the cluster rectangle |
| Curl noise is divergence-free, so particles do not clump | — | Cited: [SIGGRAPH 2007 paper](https://www.dneg.com/wp-content/uploads/2022/11/2007.pdf) | 32 k additive particles behind the speed, velocity scaled by speed |
| Chladni plate | u = cos(nπx)cos(mπy) − cos(mπx)cos(nπy), m ≠ n | Cited: [luciopaiva/chladni](https://www.github.com/luciopaiva/chladni) | (n, m) stepped by rpm band, 120 ms crossfade, `fwidth` AA (prop.) |
| Reaction-diffusion (Gray-Scott) | D_A 1.0, D_B 0.5, f 0.055, k 0.062 | Cited: [Karl Sims](https://www.karlsims.com/rd.html) | Too slow for live data; OSD backgrounds only |
| Smooth union | `h = max(k − \|a − b\|, 0)/k; d = min(a, b) − h²k/4` | Cited: [iquilezles.org](https://iquilezles.org/articles/smin) (via ports) | Limiter droplets merging into the gear pod, k 8–12 px (prop.) |
| BMW Digital Art Mode (artist Cao Fei); Mercedes CLA shares one Unity 3D scene across screens | — | Cited: [BMW press](https://www.press.bmwgroup.com/usa/article/detail/T0363542EN_US/bmw-first-to-bring-digital-art-into-vehicles-artist-cao-fei-creates-digital-art-mode); [Unity](https://unity.com/resources/next-gen-car-experiences-mercedes-cla) | Generative layers are a selectable mode, never the readout |

### Springs, easing, sweeps and gear changes
**Premium because:** motion with physical character that is exact at any frame rate and never trails the data.

| Technique | Values | Source or est. | How we use it on the GPU overlay |
|---|---|---|---|
| Apple `Spring` | duration 0.5, bounce 0.3 → stiffness 157.9, damping 17.6; k = m(2π/T)², ζ = 1 − bounce | Cited: [Apple docs](https://developer.apple.com/documentation/swiftui/spring.md); mapping derived | Springs in (T, ζ) |
| Material 3 durations and easings | 50, 100, 150, 200; 250–400; 450–600; 700–1000 ms. Emphasized (0.2, 0, 0, 1); emphasized decelerate (0.05, 0.7, 0.1, 1); standard accelerate (0.3, 0, 1, 1) | Cited: [MotionTokens.kt](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/MotionTokens.kt) | Our duration scale; easings as 256-entry LUTs baked at load |
| M3 springs (ζ / k) | Standard spatial 0.9 / 1400, 700, 300 → T 0.168, 0.237, 0.363 s; expressive default 0.8 / 380 → 0.322 s; effects 1.0 / 3800 → 0.102 s | Cited: [StandardMotionTokens.kt](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt); [ExpressiveMotionTokens.kt](https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt); periods derived | Spring presets |
| Closed-form spring update | Exact for ζ < 1 and ζ = 1; ramp lag 2ζ/ω (16.7 ms at T 70 ms, ζ 0.75) | derived | Allocation-free, identical at 60/120/144 Hz; velocity feed-forward removes lag |
| MoTeC SLM-C priorities; BMW S 1000 RR flash in Hz with separate ON and OFF rpm | — | Cited: [SLM-C PDF](https://cdn.shopify.com/s/files/1/0332/8881/4637/files/CDS18122_SLM-C_Club_Shift_Light_Module.pdf); [S 1000 RR manual](https://www.manualslib.com/manual/745085/Bmw-S-1000-Rr.html?page=87) | 100–150 rpm OFF hysteresis (est.) |
| Startup sweep is a self-test of 2–3 s | — | Cited (weak): [NHTSA TSB](https://static.nhtsa.gov/odi/tsbs/2020/MC-10179211-0001.pdf); [ASE guide](https://open-exam-prep.com/study-guides/ase-t6/gauges-warning-driver-info/instrument-clusters-datalink-gauges) | Ours: 350 ms up, 300 ms down inside a ≤1 s boot (prop.) |
| Gear change | Old glyph to 0.92 and out in 50 ms; new 1.12 → 1.0 on T 0.168 s, ζ 0.9; 1-frame rim flash at 60% | prop. | Rank-1 signature moment |

---

## 7. Cross-cutting themes

Counts give how many of the six research domains supply evidence.

1. **Colour is state, never decoration.** Tesla's grey world and in-path blue; GT7's white/grey/red assists; Porsche's single contrast hue on the tach; the LFA turning red only near the redline; Oblivion's one accent; The Expanse finding red too eye-grabbing for structure. (5 of 6.)
2. **Discrete steps beat continuous ramps in the periphery.** AiM per-LED thresholds, MoTeC stages, Bosch's 6 brightness steps, ACC's 4 tyre steps, Tesla's 3 proximity steps, BR2049's stepped lens zoom. (3 of 6.)
3. **Rank 1 lives in its own top band, apart from alarms.** 10 shift LEDs on top at Bosch, AiM and Cosworth; the 963's top light bar; MoTeC warnings outranking shift; peripheral warnings out-capturing central ones; pilots preferring a full-width horizon; BMW's upper HUD for alerts. (3 of 6.)
4. **Light comes from geometry and moves.** Apple's travelling, motion-linked specular; the dauphine facet swap; anisotropic sunburst; Koenigsegg's level-keeping content. Telemetry G replaces device motion. (2 of 6.)
5. **Depth comes from layered stacks, not boxes.** Tron's reliefs, Oblivion's hologram in glass, the Bugatti skeleton, the domed sapphire, Apple's thicker glass for bigger elements, Mercedes' blade under a lens. (3 of 6.)
6. **Restraint over a moving scene.** Mercedes' 10° × 5° AR window, GT7's Race Info Only, Oblivion's greeble control, The Expanse's no-decoration rule, Lucid's Zen theme and cognitive-load critique, the Taycan object cap, BMW shy-tech. (3 of 6.)
7. **Motion is a state machine, and idle is still.** Territory's initial/action/loop, Cyberpunk's glitch as an event, Apple's materialize, Lucid's camera on signal, Bosch event pages, McLaren's bar vanishing at idle. On OLED, stillness is also burn-in care. (3 of 6.)
8. **The scale belongs to the car.** Porsche's shift-at-12, AiM per-gear thresholds, ACC/LMU per-car tools, the GT7 bar called an approximation, the 550 Spyder's operating band, the LFA moving its redline. (4 of 6.)
9. **Instruments must be faster than mechanics.** The LFA went digital because no needle could keep up; Mercedes' AR arrows were judged slow; analytic springs are exact at any frame rate. (3 of 6.)
10. **Black is the canvas.** BMW's black band, Lucid's floating OLED, Apple's dimming layer, HUD symbols on a combiner, Tron on pure black. On our C1 off pixels are transparent, so grain or fill over them is an error. (3 of 6.)

## 8. Design principles for FH6 WheelDash v2

1. **Shift lights fill outside-in and strobe only as a strip.** Pods light on per-car, per-gear thresholds in the same frame (0 ms attack), green → amber → red, ending at the centre pod above the gear. At the shift rpm the whole row takes the `shift` colour; at the limiter it strobes at 10 Hz, 50% duty (3 on / 3 off frames at 60 Hz, 6 / 6 at 120 Hz; derived), release ≤50 ms. Test: a frame-stepped capture shows the outer pods first and a 100 ms ±1 frame period.
2. **Nothing taller than the strip flashes faster than 3 Hz.** The strobing strip is ≤42 px tall at scale 1.0 (≤59 px at the 1.40 maximum), which passes the WCAG area test down to a 0.6 m eye distance. The hero gear may pulse at 2 Hz only after 0.5 s on the limiter; no large saturated-red area ever flashes. Test: an automated flash-area audit over `preview-export` frames.
3. **Colour budget by lit area, and state in steps.** Brand or base hue ≥80% of lit area, white ≤15%, red ≤5% except at the limiter; red is never structural; at most 2 saturated objects in the ADAS scene. Tyre (4 steps), proximity (3) and assists (3) change with a 150 ms crossfade. Test: a lit-pixel histogram per mockup state, and no intermediate hue held longer than 150 ms.
4. **Hero legibility survives blur.** Downscaled to 960×540 and blurred at σ 1.5, the gear holds ≥7:1 contrast and the speed ≥4.5:1 against what is behind them. Digits use fixed advances, so speed width is constant from 0 to 999. Test: a CI gate on every mockup.
5. **Light moves with the car.** One virtual light per cluster, azimuth φ_L = 135° + 25°·clamp(G_lat / 1.5 g) through a ≈150 ms critical spring, drives every rim (1 px, peak 60–80% white), facet and anisotropic finish. Test: in a steady 1 g corner the highlight moves ≥15°.
6. **Bloom is earned.** Only ranks 1–3 write to the emissive buffer that feeds bloom; labels and bezels never glow. Test: with pods, tach and gear masked, the emissive buffer is empty.
7. **At least 3 depth layers and no flat rectangle.** Hero at 0 px, tach ±2 px, bezel ±4 px of G parallax (spring ω 12 rad/s). Every glass element has a 1 px rim, a fresnel term and one of caustic, lensing or crescent, within alpha 28..64. Test: a review checklist plus a parallax capture.
8. **Show slowly, hide instantly, never delay telemetry.** Boot is ≤1 s and skippable; gear and speed are live from frame 1; the sweep plays only on the first `IsRaceOn` 0 → 1 after launch, never on unpause. Hide takes 1 frame. Test: packet-to-present on the first race frame equals steady state.
9. **Idle is still.** With constant telemetry, nothing moves outside telemetry-bound regions; static structure stays ≤20% luminance (est. OLED-care cap); fluid, particles and glitch are forced by telemetry or events and settle at zero input. Test: zero frame difference outside telemetry regions at idle.
10. **The car sets the scale, and motion never trails.** Thresholds, redline and operating band come from the car profile; Classic dials put the shift rpm at 12 o'clock. Durations sit on the 50 ms M3 scale, springs are closed-form (T, ζ), and the live needle uses T ≤70 ms with velocity feed-forward. Test: three cars each put the needle vertical at their own shift rpm, and on a `mock-udp` rpm ramp the needle lags ≤16.7 ms.

## 9. Implications for tokens

- **Colour roles**: `base` (brand structure hue), `primary` (near-white `#E8ECF2`, prop.), `neutral` (`#8C929C` at 70%, est.), `off` (`#5A606B`, prop.), `accent` (in-path / lead, `#3BA7FF`, est.), `caution` (`#FFB020`, prop.), `critical` (`#FF3B30` UI, `#FF1E1E` race red, est.), `glassTint`, `ghostPod`. Stage roles `stageGreen`, `stageAmber`, `stageRed`, `shift`: the legacy tokens (`#00E676`, `#FFB300`, `#FF1744`, white strobe) sit close to the race est. values; keep them until a C1 test says otherwise.
- **Type scale**: gear 45–55% of cluster height; speed 40–50% of gear; labels ≈25% of value height, caps, +4–8% tracking (all est.). Faces by role: gear Saira Condensed 800 or Teko 600; speed Barlow Condensed 700 or Saira Condensed 700; labels Titillium Web 600; VFD digits Chakra Petch or Orbitron; VFD labels Rajdhani 600–700; glass and OSD Inter. Bahnschrift is a host font, not OFL, and must not be a default. Verify `tnum` per face and force fixed MSDF advances when it is missing.
- **Strokes**: 1 px rims only; 2 px hairlines; 3 px minor ticks; 5 px major ticks; 8–10 px needle or ribbon core (prop.). Rims peak 60–80% white with exponent 24–48; chamfers 50% lit, 15% shadow (est.).
- **Glow and bloom**: neon halo 6 px, bloom 24 px at 0.12; bloom chain 8 levels from 256 px, threshold 0.6–0.8, knee 0.7, intensity 0.8, emissive buffer only; lume and HUD halos 6–8 px at 25%.
- **Glass**: alpha 28..64, Tinted at the top; lensing w 24–40 px; fresnel (r/R)^4 to 40/255; crescent 18/255; AR rim tint `#7B6CFF`; dimming pad ∝ (1 − scrim %); blue-noise dither on every low-alpha gradient; one variant per cluster.
- **Motion**: durations on the M3 scale; easings as LUTs; springs (T, ζ): `snap` 0.102 / 1.0, `fast` 0.168 / 0.9, `default` 0.237 / 0.9, `slow` 0.363 / 0.9, `expressive` 0.322 / 0.8, `needle` 0.07 / 0.75 with feed-forward, `light` ≈0.15 critical, `parallax` ω 12 rad/s. Crossfade 150 ms; materialize 200 ms; boot ≤1 s; strobe 10 Hz strip, ≤3 Hz elsewhere.
- **Grid**: the token draft's 8 px (4K) unit holds the 24 px Oblivion lattice as 3 units.

## 10. Implications for families

- **Classic Analog** (`PorscheAnalog`, `LfaV10Nurburgring`; horology line `BugattiTourbillon`, `PaganiHorologerie`, `KoenigseggGyro`): shift-at-12 scale, operating band, dauphine facets, screen-space shadows, sunburst face with spun sub-dials, domed sapphire, slew-dependent needle; the horology line adds skeleton parallax, guilloché, ruby jewels and a gyro horizon. Signature: the facet swap during a sweep.
- **Modern Digital 2027** (`AmgMbuxSuperscreen`, `DriftGlass`, `GlyphDotMatrix`, `Gt7HoodPro`): CarPlay-style parametric arcs, one contrast hue, Mercedes phase-coloured bar, BMW black band, shy micro-labels, Zen tier. Signature: the ribbon locking vertical at the shift point.
- **Liquid Glass** (`LiquidGlass`, `VisionSpatialGlass`, the OSD): self-lensing, G-driven specular, materialize on show, event-origin glow, dimming pad, Tinted toggle, caustic texture. Signature: light running round the gear pod on an upshift.
- **Sci-Fi** (`RetroCyber`, `SynthwaveOutrun` and the other concept themes): dot lattice, three-term neon, sector boot, 3-state machine, event-only glitch, brand hue as faction code. Signature: the sector-by-sector boot.
- **Fluid / Generative** (`KineticLiquidMetal`, `CymaticResonanceField`, `ChronoMultiverse`): clipped fluid forced by G, relief shading, curl-noise slipstream, rpm-stepped Chladni modes, smooth-union droplets, always under the readouts. Signature: limiter droplets merging into the gear pod.
- **ADAS & HUD** (`TeslaAdas`, `WindshieldHud`, `ArNavWayfinder`): semantic scene colours, two-tone carpet, ego underglow, object cap, 3-step arcs, BMW two-layer split. The aviation research argues for moving `F35HelmetHmd` here from Sci-Fi, since it shares the FPM, boresight and horizon. Signature: the FPM separating from the boresight in a drift.
- **Proposed seventh family: Race Dash** (`BoschDdu`, `ValkyrieLmp1`, `Gt7ModularTripod`). Four vendors, F1 and LMDh converge on an architecture no other family owns: top shift row plus side alarm lamps, warnings over shift, event page swaps, a signed centre-out bar, lap-delta mode, intensity separate from hue, bold condensed white-on-black type. It differs from Modern Digital in density, semantics and audience. Signature: the outside-in fill landing above the gear, then the strip strobe.
- **Not families**: horology is a line in Classic Analog (same dial grammar, other materials); aviation is a mode of ADAS & HUD.

## 11. Assets Phase 6 must make

- **Textures**: Liquid Glass caustic lens at 4K with a normal/refraction pair; carbon weave with normal map (Race Dash); brushed titanium and black anodised aluminium albedo/roughness; a 64×64 or 128×128 blue-noise dither tile; the 4K brand-logo set with small dial variants.
- **Normal and tangent maps**: Clous de Paris (P ≈ 12 px), barleycorn (N ≈ 24), waves; sunburst (radial) and spun (tangential) maps with strength in R and rotation in G; sapphire dome; bezel chamfers. Each keeps its procedural compute-at-load fallback.
- **Flipbooks**: liquid-metal and ferrofluid loops with matching flow and normal maps, as the budget fallback for live fluid; an optional glitch slice mask.
- **Meshes (glTF)**: one ego car scaled per class (sport sedan, SUV, supercar) with sprite fallbacks; a tourbillon gear-train plate and bridges, or an SDF recipe for them.
- **Fonts**: MSDF atlases for Saira and Saira Condensed, Teko, Barlow Condensed, Titillium Web, Chakra Petch, Orbitron, Rajdhani and Inter, each with a recorded `tnum` check and its OFL licence in the asset ledger.

## 12. Anti-patterns: what makes a cluster look cheap

- **Bloom on everything, one line weight everywhere**: no hierarchy (est.); compare Tron's three-term light and Oblivion's heavy hero strokes.
- **Three or more saturated hues at equal weight, or red as structure**: The Expanse's designer found red hard to use; Tesla keeps the world grey (cited).
- **Greeble, fake scrolling numbers, gimmicks**: rejected by the Oblivion and Tron designers (cited).
- **Giant flashing alert boxes and large red flashes**: The Expanse rules; WCAG 2.3.1 (cited).
- **Continuous hue sweeps for state**: every state-signalling reference uses steps (cited).
- **Always-on scanlines, aberration or grain over off pixels**: lights pixels that should be transparent (est.); BR2049 and Cyberpunk use degradation only as a register or event (cited).
- **Display faces for labels**: players replace Cyberpunk's squared UI face for legibility (cited, indirect).
- **Fixed-percentage shift points**: the GT7 bar is called an approximation (community, cited).
- **Opaque boxes, full frames, white faces on OLED**: Lucid floats (cited); a white face is a burn-in risk (est.).
- **Mixed glass variants, frost instead of lensing; soft-spring live needles; sweeps on every unpause**: Apple's rules and the LFA precedent (cited), plus our 1-frame rule.
- **Too much at once**: the Lucid cognitive-load critique (summary); the Inception Halo read as impact over function (snippet).

## 13. Open questions

1. What is the real eye-to-panel distance? Principle 2 assumes ≥0.6 m.
2. Is the `shift` colour white (current token) or blue/purple (race convention, est.)? Is outside-in or left-to-right the default fill?
3. Does the overlay present at 60 or 120 Hz? At 50% duty, 10 Hz and 7.5 Hz give whole-frame on/off phases at both rates; 8 Hz does not (derived).
4. Does the car profile carry shift, peak-torque and peak-power rpm? Is per-gear data feasible?
5. Is roll available and trustworthy in the 324-byte packet for the gyro horizon? Check against the golden packets (97% rule). Likewise, can a live lap delta be built from packet fields, or does it need a recorded reference lap?
6. Which candidate OFL faces actually ship `tnum`?
7. Every est. value (GT7, Tesla camera, F1 LEDs, Bosch proportions, FUI palettes) needs frame grabs, and every fluid, particle and bloom cost needs `bench-render`.
8. Is a captured-frame lensing path worth its latency? (Optional per the Phase 3 brief.)
9. Research gaps: Ferrari, Red Bull, Cadillac and 499P wheels; the Quadra Turbo-R dash; the Tron: Ares in-world UI; Polestar, Genesis and Chinese EV clusters; MIL-STD-1787 line widths; vendor blink-rate defaults (manuals exceeded fetch limits); Apple's spring preset values.

## 14. Sources

**Game HUDs, ADAS, aviation**
- https://gran-turismo.com/us/gt7/manual/race/02
- https://gran-turismo.com/us/gt7/manual/drivingoption/05
- https://gamepur.com/?p=309437
- https://www.gtplanet.net/forum/threads/overview-of-gt7-telemetry-software.418011/post-14567882
- https://gtplanet.net/tag/cedec
- https://www.gtplanet.net/display-tech-is-finally-catching-up-to-gran-turismo-7-20260731/ (403, not read)
- https://forums.forza.net/t/forza-ui-multiplayer-qol-changes-sample-picture-inside/719086
- https://forums.forza.net/t/tire-temp-wear-hud-for-simhub/643080
- https://guide.lemansultimate.com/hc/en-gb/articles/13201287548687-Understanding-the-HUD-Heads-Up-Display (403, snippet)
- https://gameplay.tips/guides/le-mans-ultimate-the-in-game-hud-guide.html
- https://overtake.gg/news/comments/76869
- https://www.danielnewmanracing.com/change-logs
- https://traxion.gg/why-tyre-pressures-are-key-to-winning-in-asetto-corsa-competizione/
- https://www.overtake.gg/downloads/sidekick.11007/
- https://www.overtake.gg/downloads/race-element.50578/updates?page=6
- https://www.notateslaapp.com/tesla-reference/636/all-tesla-fsd-visualizations-and-what-they-mean.
- https://www.notateslaapp.com/news/3008/tesla-to-switch-to-unreal-engine-for-improved-fsd-visualizationsk
- https://www.teslaoracle.com/2025/10/11/tesla-improves-fsd-v14-driving-visualization-to-indicate-autopilot-in-use-video/
- https://www.unrealengine.com/en-US/spotlights/rivian-s-r1t-features-state-of-the-art-hmi-designed-developed-and-deployed-in-unreal-engine
- https://www.unrealengine.com/en-US/spotlights/rivian-brings-adventurous-spirit-to-new-display-ui-powered-by-unreal-engine (403)
- https://stories.rivian.com/software-update-2025-46
- https://media.mbusa.com/releases/the-new-mercedes-benz-s-class-2021
- https://carmagazine.co.uk/car-news/tech/mercedes-augmented-reality-navigation-how-does-it-work (paywall, snippet)
- https://www.mbscottsdale.com/blog/how-to-use-route-guidance-with-augmented-reality/amp/?p=15177
- https://moba.i.mercedes-benz.com/baix/cars/223.1_mbux-premium_2020_a/en_GB/page/ID_dae06f278d69960cac190d2b58651e91-1e2495618d69960cac190d2b02b931ab-en-GB.html
- https://www.press.bmwgroup.com/asia/article/detail/T0458175EN/the-new-bmw-ix3-now-available-in-singapore
- https://insideevs.com/news/481597/xpeng-surrounding-reality-display-navigation-system/
- https://www.autonomousvehicleinternational.com/news/adas/xpeng-p7i-to-launch-in-china-with-xngp-architecture-enabling-l4-adas-functions.html
- https://carnewschina.com/2025/06/05/huawei-and-xpeng-unite-to-launch-industry-leading-ar-hud-for-new-xpeng-g7-suv/ (headline only)
- https://en.wikipedia.org/wiki/Head-up_display
- https://journals.sagepub.com/doi/ref/10.1177/154193128903302008
- https://apps.dtic.mil/sti/tr/pdf/ADA331767.pdf (not opened)
- https://f14.manuals.heatblur.se/f14bu/systems/vdig_r/vertical_display_indicator_group_replacement.html
- https://ntrs.nasa.gov/api/citations/19900008316/downloads/19900008316.pdf

**Production clusters and Liquid Glass**
- https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/
- https://developer.apple.com/videos/play/wwdc2025/219/
- https://www.macrumors.com/2025/10/20/ios-26-1-liquid-glass-toggle/
- https://tidbits.com/2025/10/21/ios-26-1-to-add-optional-opacity-to-liquid-glass/
- https://techcrunch.com/?p=3059677
- https://www.apple.com/newsroom/2025/05/carplay-ultra-the-next-generation-of-carplay-begins-rolling-out-today/
- https://www.macrumors.com/2025/05/15/apples-carplay-ultra-now-available/
- https://9to5mac.com/?p=1001421
- https://insideevs.com/news/759736/carplay-ultra-aston-martin-launch/
- https://newsroom.porsche.com/en/press-kits/911/Interior.html
- https://www.topgear.com/car-reviews/porsche/911-0/interior
- https://designsystem.porsche.com/v4/must-know/typography/
- https://newsroom.porsche.com/en/press-kits/taycan/Die-Driver-Experience-und-die-Connectivity.html
- https://paultan.org/2024/02/07/2025-porsche-taycan-facelift-range-now-up-to-678-km-320-kw-dc-fast-charging-up-to-952-ps-on-turbo-s
- https://lucidmotors.com/stories/introducing-lucid-gravity-touring
- https://www.topgear.com/car-reviews/usa/lucid/gravity-us/interior
- https://www.motor1.com/reviews/756565/2025-lucid-gravity-first-drive/
- https://www.cars.com/articles/clear-pull-2025-lucid-gravity-suv-debuts-474403
- https://electrek.co/2026/06/23/lucid-gravity-2026-review/
- https://newsroom.bmwgroup.com/americas/new-bmw-panoramic-idrive-revolutionizes-vehicle-operation
- https://newsroom.bmwgroup.com/global/world-premiere-bmw-ix3-en
- https://newsroom.bmwgroup.com/global/the-start-of-a-new-era-the-new-bmw-ix3
- https://media.mercedes-benz.ca/releases/release-c6ffb9918e70966e2c6964edd30527f4-the-eqs-the-interior-design (404 on fetch)
- https://group.mercedes-benz.com/innovation/digitalisation/connectivity/mbux-hyperscreen.html (403)
- https://www.t3.com/auto/vehicle-tech/mbux-superscreen-on-the-mercedes-benz-cla-shooting-brake
- https://unity.com/resources/next-gen-car-experiences-mercedes-cla
- https://group.mercedes-benz.com/innovation/product-innovation/technology/ces-2024.html
- https://www.cardesignnews.com/cars/lighting-and-screens-define-new-mercedes-cla/459568

**Motorsport and flash standards**
- https://www.autosport.com/f1/news/how-mclaren-has-tweaked-its-steering-wheel-to-maximise-f1s-2026-hybrid/10798146/
- https://www.mclaren.com/racing/formula-1/2026/what-is-the-technical-specification-of-our-2026-formula-1/
- https://f1chronicle.com/f1-driver-inputs-steering-wheel-2026
- https://www.evoindia.com/article/f1-steering-wheel
- https://mintlify.com/TracingInsights/26Regs/technical/electronics (unverified)
- https://www.motorsport.com/imsa/news/porsche-963-driving-manual-daytona/10412256/
- https://www.bosch-motorsport.com/products/displays/display-ddu-10/
- https://www.bosch-motorsport.com/products/displays/display-ddu-11/
- https://www.bosch-motorsport.com/content/downloads/raceparts/Resources/pdf/Data%20Sheet_323420299_Display_DDU_11.pdf
- https://xtramotorsport.com/product/bosch-motorsport-ddu11-display/
- https://www.aim-sportline.com/en/products/mxg-strada
- https://aim-sportline.com/us/products/mxg-strada/index.htm
- https://aim-sportline.com/us/products/mx-series/shift-lights-and-alarms.htm
- https://www.aim-sportline.com/download/doc/eng/mx1.2-1.3_strada/MX1.2+1.3_Strada_user_guide_eng.pdf
- https://www.motec.com.au/hessian/uploads/C127_User_Manual_da85a94ff5.pdf (404 on fetch)
- https://motec.com.au/products/C127%20-%20OBSOLETE
- https://emsinc-tn.com/product/motec-c127-7-color-display/
- https://www.sourcesensors.com/news/motec-launches-new-full-colour-dash-displays/
- https://www.motec.com.au/products/C1212%20Race%20Display%20Kit
- https://www.motec.com.au/products/D175
- https://www.motec.com.au/products/SLM
- https://cdn.shopify.com/s/files/1/0332/8881/4637/files/CDS18122_SLM-C_Club_Shift_Light_Module.pdf
- https://www.cosworth.com/media/gnppbdda/ccw-mk2_product-information-sheet.pdf
- https://cosworth.com/motorsport/products/ccw-mk2
- https://www.cosworth.com/motorsport/products/ccw-mk3/
- https://saemobilus.sae.org/content/J1133_201106
- https://stacks.cdc.gov/view/cdc/202242/cdc_202242_DS1.pdf
- https://mutcd.fhwa.dot.gov/resources/interpretations/4_09_64.htm
- https://core.ac.uk/download/76992917.pdf
- https://link.springer.com/content/pdf/10.3758/s13428-010-0054-8.pdf
- https://www.w3.org/WAI/WCAG21/Understanding/three-flashes-or-below-threshold.html
- https://www.manualslib.com/manual/745085/Bmw-S-1000-Rr.html?page=87
- https://docs.beamng.com/modding/vehicle/vehicle_system/controller/gauges/shiftlights/

**Horology and hypercar analog**
- https://newsroom.bugatti.com/press-releases/haute-horlogerie-to-the-bugatti-tourbillon
- https://newsroom.bugatti.com/press-releases/bugatti-an-enduring-and-remarable-tale-in-the-world-of-horology
- https://www.motor1.com/news/724009/bugatti-tourbillon-instrument-cluster-details/
- https://timeandtidewatches.com/?p=229697
- https://www.motor1.com/news/735286/pagani-utopia-interior-technology-analysis/
- https://carexpert.com.au/car-news/pagani-utopia-revealed
- https://www.hagerty.com/media/?p=252194
- https://pagani.com/app/uploads/2022/09/Pagani-Utopia_Contents_ENG.pdf (HTTP 429, not read)
- https://www.thedrive.com/news/28249/300-mph-koenigsegg-jesko-hypercar-boasts-worlds-first-gyroscopic-gauge-cluster
- https://www.qt.io/press/qt-brings-digital-cockpit-to-koenigsegg-hypercars
- https://thesupercarblog.com/koenigsegg-cc850-is-a-modern-take-on-the-cc8s-that-started-it-all/
- https://www.evo.co.uk/lexus/lfa/202607/lexus-lfa-rev-counter-art-of-speed
- https://www.topgear.com/car-news/big-reads/archives-top-gears-first-lexus-lfa-drive
- https://mag.lexus.co.uk/gauging-interest-with-the-lexus-is-f-sport-meter (404 on fetch)
- https://newsroom.porsche.com/en/2026/history/porsche-icon-christophorus-417-tachometer-41697.html
- https://jalopnik.com/how-the-porsche-911s-interior-and-iconic-five-gauge-clu-1832057073
- https://www.motor1.com/news/721359/new-porsche-911-no-analog-tachometer/
- https://patek.com/en/company/savoir-faire/attention-to-details/dials
- https://monochrome-watches.com/technical-perspective-the-art-of-guilloche-dials-examples-of-guilloche-patterns-comparison-stamped-cnc-dials
- https://timeandtidewatches.com/watch-finishing-techniques/
- https://en.wikipedia.org/wiki/Super-LumiNova
- https://www.rctritec.com/en/superluminova/colour-settings-according-to-customer-requirements
- https://www.fratellowatches.com/how-watches-work-antireflective-coatings-in-watches/
- https://www.gnomonwatches.com/blogs/news/anti-reflective-sapphire-crystal
- https://www.kitware.com/pbr-journey-part-2-anisotropy-model-with-vtk/
- https://forums.autodesk.com/t5/arnold-general-rendering-forum/generative-radial-anisotrophic-material-brushed-metal/m-p/12862354
- https://manuals.keyshot.com/keyshot2024/manual/brushed-radial.html
- https://help.autodesk.com/cloudhelp/JPN/AR-Cinema4D/files/ci-tutorials/ci-shading/arnold_for_cinema_4d_ci_Shading_ci_Anisotropic_Brushed_Metal_html.html

**Sci-fi FUI**
- https://gmunk.com/OBLIVION-GFX
- https://www.hudsandguis.com/2013/05/02/oblivion-interface-design/
- https://www.pushing-pixels.org/?p=9032
- https://gmunk.com/TRON-Legacy
- https://gizmodo.com/a-look-at-how-tron-legacys-awesome-software-art-was-mad-5788616
- https://www.core77.com/posts/18933/josh-nimoy-on-the-secrets-behind-the-special-effects-in-tron-legacy-18933
- https://prologue.com/portfolio-item/tronares/
- https://imaginaryforces.com/project/tron-ares
- https://www.fantasylandnews.com/?p=23074
- https://territorystudio.com/project/blade-runner-2049/
- https://www.engadget.com/2017-10-20-designing-the-technology-of-blade-runner-2049.html
- https://architosh.com/2018/01/territory-studio-cinema-4d-powers-screen-visuals-in-blade-runner-2049
- https://www.hudsandguis.com/home/2019/cyberpunk-2077
- https://www.nexusmods.com/cyberpunk2077/mods/18623
- https://www.nexusmods.com/cyberpunk2077/mods/18078
- https://medium.com/super-jump/a-ux-analysis-of-cyberpunk-2077s-hud-f74afe6b9961
- https://www.behance.net/gallery/133185623/Cyberpunk-2077User-Interface-%28Part-2%29 (403, snippet)
- https://www.pushing-pixels.org/?p=18324
- https://www.hudsandguis.com/home/2021/theexpanse
- https://www.ces.tech/articles/daimler-presents-an-avatar-inspired-concept-car-at-ces-2020/
- https://www.businesswire.com/news/home/20200106006083/en/Inspired-future-Mercedes-Benz-VISION-AVTR (403, snippet)
- https://group.mercedes-benz.com/innovation/product-innovation/design/vision-avtr-bci.html
- https://www.media.stellantis.com/uk-en/peugeot/press/peugeot-reveals-inception-concept
- https://www.media.stellantis.com/me-en/peugeot/press/vivatech-2024-peugeot-designs-the-future-of-driving-pleasure-with-hypersquare
- https://driving.co.uk/news/inception-concept-hints-at-what-peugeots-will-look-like-from-2025

**Fluid, generative and motion**
- https://raw.githubusercontent.com/PavelDoGreat/WebGL-Fluid-Simulation/master/script.js
- https://developer.nvidia.com/gpugems/gpugems/part-vi-beyond-triangles/chapter-38-fast-fluid-dynamics-simulation-gpu
- https://www.karlsims.com/rd.html
- https://history.siggraph.org/?p=115355
- https://www.dneg.com/wp-content/uploads/2022/11/2007.pdf
- https://www.github.com/luciopaiva/chladni
- https://wenhaoquestion.github.io/chladni-plate-explorer
- https://editor.p5js.org/pattvira/sketches/5LAASlRY-
- https://phys.libretexts.org/Bookshelves/Waves_and_Acoustics/The_Physics_of_Waves_(Goergi)/11%3A_Two_and_Three_Dimensions/11.03%3A_Chladni_Plates
- https://iquilezles.org/articles/smin
- https://github.com/glslify/glsl-smooth-min
- https://www.press.bmwgroup.com/usa/article/detail/T0363542EN_US/bmw-first-to-bring-digital-art-into-vehicles-artist-cao-fei-creates-digital-art-mode
- https://www.press.bmwgroup.com/united-kingdom/article/detail/T0363620EN_GB/the-bmw-group-at-the-consumer-electronics-show-ces-2022
- https://www.press.bmwgroup.com/global/article/detail/T0447111EN/new-bmw-panoramic-idrive-revolutionises-vehicle-operation-visionary-technology-package-for-the-neue
- https://www.bmwgroup.com/en/news/general/2025/newidrive.html
- https://media.mbusa.com/releases/release-ebe78e1e0abb0f8a2f173a4032054126-mercedes-benz-heralds-a-new-era-for-the-user-interface-with-human-like-virtual-assistant-powered-by-generative-ai
- https://developer.apple.com/documentation/swiftui/spring.md
- https://developer.apple.com/fr/videos/play/wwdc2023/10158/
- https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/MotionTokens.kt
- https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/StandardMotionTokens.kt
- https://raw.githubusercontent.com/androidx/androidx/androidx-main/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/tokens/ExpressiveMotionTokens.kt
- https://static.nhtsa.gov/odi/tsbs/2020/MC-10179211-0001.pdf
- https://open-exam-prep.com/study-guides/ase-t6/gauges-warning-driver-info/instrument-clusters-datalink-gauges
- https://electro-tech-online.com/threads/identification-of-stepper-motor-how-can-i-run-it-from-a-speedometer.101453
