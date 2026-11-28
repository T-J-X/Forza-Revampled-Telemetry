# Vision Spatial Glass: research note

Written 2026-11-28 for build #10 (catalog row 2, Apple Vision category, target `concept`, Liquid Glass family). Statements marked **Cited** were read from the source named, on that date. Statements marked **Our reading** are conclusions for this overlay. The material itself (what Apple's Liquid Glass does, and why the first Liquid Glass pilot was rejected) is covered in the pilot's [`RESEARCH.md`](../liquid_glass/RESEARCH.md) §2 and §4; this note does not repeat it. The optics are in [`docs/specs/optics.md`](../../../../../docs/specs/optics.md).

## 1. The benchmark

This is a catalog concept with no single real car display. Following "Target" in `CATALOG.md`, it is benchmarked against one real thing: **a visionOS window with its ornaments**, as Apple's Human Interface Guidelines and WWDC23 session 10076 describe them. That is the real product whose grammar the brief uses: a main glass window, smaller glass pieces floating in planes in front of it, depth read from parallax and light rather than from drawn shadows. Apple's Liquid Glass (WWDC25 session 219) gives the material, as it does for the pilot.

The brief's car references were checked as well (§4). One of them does not say what the brief says it does.

## 2. What makes a visionOS window and its ornaments recognisable

| # | Feature | Source |
|---|---|---|
| V1 | **Ornaments float in front of their window.** An ornament sits "in a plane that's parallel to its associated window and slightly in front of it along the z-axis". Apple gives no number for the offset. | Cited: HIG, Ornaments |
| V2 | **An ornament overlaps the window's edge.** WWDC23 10076 (third-party notes of the session): bottom ornaments sit outside the window "with depth", and overlap its bottom edge by 20 points. | Cited: WWDC Notes, wwdc23/10076 |
| V3 | **An ornament is no wider than its window.** "Aim to keep an ornament's width the same or narrower than the width of the associated window." | Cited: HIG, Ornaments |
| V4 | **Everything is glass, nothing is opaque.** Windows are made of the glass material; avoid opaque windows. Ornament backgrounds are glass by default. | Cited: HIG, Ornaments; WWDC Notes 10076 |
| V5 | **Don't stack lighter materials.** Stacking glass on glass costs contrast. | Cited: WWDC Notes 10076 |
| V6 | **Depth communicates hierarchy, and is conveyed by the system's light.** Reflections, shadows and colour temperature give depth; a sheet makes its window recede along the z-axis. | Cited: HIG, Spatial layout |
| V7 | **Text does not get depth.** "In general, avoid adding depth to text. Text that appears to hover above its background is difficult to read." | Cited: HIG, Spatial layout |
| V8 | **Few depth levels.** People refocus their eyes for each difference in depth, so keep depth changes few and purposeful. | Cited: HIG, Spatial layout |
| V9 | **Concentric corners.** Inner corner radius plus padding is the outer corner radius. | Cited: WWDC Notes 10076; also WWDC25 356 (pilot A14) |
| V10 | **Vibrancy on glass.** Text and symbols on glass use vibrancy in three levels (primary, secondary, tertiary), not colour. | Cited: WWDC Notes 10076 |
| V11 | **Specular highlights respond to geometry and to motion.** "In some cases, the lighting responds to device motion." | Cited: WWDC25 219; Apple Newsroom, 2025-06-09 |
| V12 | **Bigger glass is thicker.** Larger pieces cast deeper shadows and show more pronounced lensing and refraction. | Cited: WWDC25 219 |
| V13 | **Glass lights from within on input.** The material "illuminates from within as a form of feedback". | Cited: WWDC25 219 |
| V14 | **Materialise, don't fade** (pilot A2). | Cited: WWDC25 219 |

### 2.1 The benchmark, feature by feature

| Feature of the benchmark | In this dash |
|---|---|
| V1 ornaments in front of their window | **Matched.** Four depth layers: the two wing panes (back), the front pane (the "window"), the attitude puck (an ornament under the front pane) and the shift blade (an ornament above it, frontmost). Depth is read from parallax: lateral G shears the layers by a per-layer amount, nearest most. |
| V2 ornament overlaps the window's edge | **Matched** for the puck: it overlaps the front pane's bottom edge by 20 px. **Changed** for the blade: it floats clear of the front pane's top edge, because the blade strobes and must never cross the gear. |
| V3 ornament no wider than its window | **Changed on purpose.** The blade is 900 px over a 480 px front pane, because it is rank 1 (shift lights must read in peripheral vision) and spans the wings, which belong to the same window group. The puck keeps the rule. |
| V4 all glass, nothing opaque | **Matched.** Every pane, the blade and the puck are the kit's clear optical glass. The only opaque fill is the 0.92 black core under the gear and speed (family rule, for legibility over a bright cabin). |
| V5 no stacked glass | **Matched.** Pane overlap is kept at or under 12% of a wing's area, and inside one glass layer glass never bends glass (`docs/kit/README.md` §4). |
| V6 depth from light | **Matched.** One G-driven virtual light reaches every rim; the front pane recedes on an upshift (§5). No drop shadows are drawn, because on OLED black a shadow is invisible. |
| V7 no depth on text | **Matched.** Every digit sits flat under the untilted front pane, where its lens tilts under 22°. The wings carry only the rev ribbon and nano labels. |
| V8 few depth levels | **Matched.** Four levels, the minimum that the brief's triptych plus two ornaments needs. |
| V9 concentric corners | **Matched.** The black core's radius is the front pane's radius minus its inset. |
| V10 vibrancy, not colour | **Matched** by brightness ranks: labels at the static-part cap, values at 86% ink, gear at full ink. Colour is only the palette's primary (ribbon), shift (blade) and warning (redline). |
| V11 highlights move with motion | **Matched.** The key light's azimuth follows lateral G (`common::VirtualLight`), and the rig leans with felt force. |
| V12 bigger is thicker | **Matched.** The front pane is the thickest piece, the wings thinner, the blade and puck thinnest. |
| V13 lights from within | **Matched** by the Z-hop upshift glint (§5). |
| V14 materialise | **Matched.** Panes materialise back to front over 200 ms by growing thickness, never opacity. |
| A visionOS window is a rectangle seen face-on | **Changed.** The wings are toed in 14° and drawn through a true homography (far edge 12% shorter), as the brief asks. This is the dash's one departure from Apple's face-on windows, and it is what makes it a binnacle and not a floating app. |

## 3. The material

As the pilot: clear synthetic sapphire, n_d 1.768, no frost, one studio, dark-field lighting (`docs/specs/optics.md` §1 to §8, the pilot's RESEARCH §2 and §5). Two points of the brief are overruled by the optics, as its category note says:
- **"Tint 36" on the front pane.** optics.md §6: tint is never a flat alpha. The front pane is clear (the pilot's `clear` material, no hue lean), and its black core does the legibility work.
- **"1.5 px chromatic split at the rim" of the refracted ribbon.** optics.md §5: real dispersion through a sapphire edge this thin is well under a pixel, and the fixed 1.5 px split was the non-physical fault of the first pilot (§11). The refraction offset is what the bevel really gives (1.46 b at the rim, §4), and the split is whatever the per-channel index gives.

## 4. The brief's car references

| Reference | What the sources say | What we take |
|---|---|---|
| **BMW Panoramic iDrive** (Neue Klasse, CES 2025) | Panoramic Vision projects information "from A-pillar to A-pillar onto a black printed surface in the lower section of the windscreen"; the driving information sits on the left, above the wheel, and up to six widgets can be moved there. The optional 3D Head-Up Display sits above it, and the two together "create a 3D effect". | **Our reading:** two information planes at different depths, the nearer one holding the driving data, the farther one the context. Taken as the front pane (data) over the wings (rev context). The black print band itself is not copied. |
| **Porsche Taycan curved cluster** | A curved, freestanding 16.8-inch screen with three freely configurable round instruments; the curve nods to the 1973 911 G-series; small touch fields sit at its edges. The sources describe no angled side zones. | **Correction to the brief:** the "angled side zones" are not in the Taycan sources. The toed-in wings stay because the brief asks for them, as our own design, and the Taycan reference is reduced to "a freestanding display wider than the wheel, whose sides curve toward the driver". |
| **visionOS window stacking** | §2. | The benchmark. |
| **Apple Liquid Glass lensing** | The pilot's RESEARCH §2. | The material. |

## 5. Signature moments and motion (the brief, checked)

- **Z-hop upshift.** A 1-frame glint on the far wing's rim, hopping forward one layer per 40 ms (wing, front pane, blade); the front pane recedes 6 px in depth on the fast spring and the new gear enters from 1.12. **Our reading:** this is V6 and V13 together, depth shown by light moving through the layers. "Recedes 6 px in depth" is drawn as a 6 px shrink of the pane about its centre (depth is scale on a spatial display, HIG Spatial layout: a farther window is smaller), and the numeral stays still for the glance (family rule).
- **Parallax shear.** The brief's 0 / 2 / 4 / 6 px per g. **Our reading:** 6 px is about 0.8 mm on a 55-inch 4K panel, too small to read as depth from 1.5 m. Proposed for the board: 0 / 6 / 12 / 18 px per g (wings, front pane, puck, blade), on a spring of ω 12 rad/s, clamped at 1.5 g. The board and the direction check confirm it.
- **Limiter.** Only the 38 px blade strobes at 10 Hz, on its own clock from the shift point (pilot lesson); the wings' outer rims hold a steady shift-colour line, which is also the cue a still frame shows.
- **Show.** Back to front over 200 ms (V14).

## 6. Telemetry, checked against `tests/golden/packets`

The golden packets cover only the race flag, timestamp, the three RPMs, acceleration, velocity, car ordinal, drivetrain, speed, power, boost, throttle, brake, gear and steer (AGENTS.md rule 1). The brief's fields, one by one:

| Brief | Verified? | What the dash reads |
|---|---|---|
| rpm, idle and max rpm, car-profile shift point → ribbon and the 15 blade pods | Yes | `Signals::rpm`, `rpm_fraction`, `shift_fraction` |
| gear, speed → front pane | Yes | `Signals::gear`, `speed`, `unit` |
| acceleration x/z (felt force) → parallax, light azimuth, bead on the puck | Yes | `Signals::g_lat`, `g_long` (smoothed acceleration), as `common::gball::felt_force` |
| pitch and roll → puck tilt | **No** (not in the golden packets) | The brief's fallback: tilt from smoothed felt force. Nothing reads pitch or roll. |
| yaw rate → wing toe-in ±2° | **No** (`Signals::yaw_rate` is `angular_velocity_y`, not in the golden packets) | Derived from verified fields: in a steady turn a_lat = v ω, so ω = a_lat / max(v, 3 m/s). |
| brake → caution step on the puck rim | Yes | `Signals::brake` |
| tyre combined slip → bead halo | **No** (tyre slip is not verified) | Body slip angle from velocity, `Signals::slip_deg`, as the pilot does. |

So the dash reads only verified fields and two quantities derived from them, and the brief's "native fields only" holds in that sense: nothing is invented.

## 7. Risks found in this pass

- **The kit has no quadrilateral glass outline** (`fh6_kit::shape::Outline` is circle, capsule and arc). A wing pane seen through a homography is a trapezoid with rounded corners. Proposed: build it as the smooth union of a few rounded slices whose heights follow the homography (the union of neighbouring slices is smooth to every order, optics.md §2), and project its ribbon and labels with the same homography on the CPU. If that shows steps or costs too much, the handover asks the coordinator for a convex-polygon outline in the kit.
- **Muddy grey stack** (the brief's own risk): overlap ≤ 12%, no glass on glass, and digits only under the untilted front pane.

## 9. After the Penpot board: what was dropped and what replaced it

TJS rejected the first flat outline sheet ("This is ass") and set the rule now in CLUSTER_PROMPT "The bar": parts the game cannot drive are dropped, and the space goes to bold, telemetry-driven ideas. Board v2 (Penpot page `07 · VisionSpatialGlass`) and the code follow it.

| Brief | Board v2 and the code | Why |
|---|---|---|
| A 10 px light ribbon crossing both wings, refracted behind the pane | **Rev tunnel:** 16 rounded rings receding to a vanishing point (0, −45) behind the pane; they fill back to front with rpm, so the limiter is the ring nearest the driver. Rings past the shift point take the shift colour, and on the limiter the whole lit tunnel holds it. The wings are glass over the tunnel, so they bend it, as the brief wanted the ribbon bent | TJS's own idea ("the tunnel from the back to front being the rev limiter"); depth is the category's whole subject |
| No background | **Dim visionOS sky:** a horizon through the vanishing point that tilts 4° per g of felt lateral force, a floor glow under it and faint clouds over it | TJS ("subtle clouds and horizon tunnel background"); the horizon is an attitude cue from verified G |
| Pitch and roll → puck tilt | Puck tilt from felt G (a gimbal ring rolled by lateral force and foreshortened by longitudinal force) | Pitch and roll are unverified in the golden packets (section 6) |
| Yaw rate → wing toe ±2° | Yaw derived as lateral G over speed (a = vω), from 3 m/s | `angular_velocity_y` is unverified |
| Tyre combined slip → bead halo | Body slip (`Signals::slip_deg`) | Per-tyre slip is unverified |
| Front pane 480 × 330, gear cap 180, speed cap 96 | Pane 560 × 396, gear cap 200, speed cap 88 (Saira 400, no ghost 8s) | The board's proportions; the pane carries every digit, untilted |
| Wings 420 × 200 by homography | Tapered glass capsules from x ±250 to ±700, half height 95 inside and 120 outside | The kit has no quad glass outline; a tapered capsule is one SDF piece |
| Parallax 0/2/4/6 px per g | 0/6/12/18 px per g (wings, pane, puck, blade), tunnel and sky −6 | Section 5: 2 px is below a glance |
| Wings carry only nano labels | The tunnel's rpm numerals (one per 1000 rpm, just inside their ring, red from the shift point) sit on both wings over the glass, with "RPM ×1000", the speed unit and the logo zone | The numerals name the rings; drawn over the glass, no rim bends them |
| Tint 36, 1.5 px chromatic split | Clear sapphire, dispersion sub-pixel | `docs/specs/optics.md` |

Board and code coordinates: the zone table in `mod.rs`.

## 10. Deeper pass while the 4K library finishes (28 Nov, after board v4)

Research only: nothing here is drawn yet. Each finding ends in a plan line for after LIBRARY DONE.

### 10.1 How the real thing behaves (Apple's own words, checked)

| Apple says | Source | Where the dash stands | Plan |
|---|---|---|---|
| Light sources make highlights that follow the geometry; on interaction the lights *move*, so light travels round the material and draws its outline; sometimes they follow device motion | WWDC25 219 | `VirtualLight` swings with felt lateral G, and the Z-hop glint runs wings → pane → blade on each upshift | Keep. Check at the closing review that the highlight visibly moves between `02` and `04` (V19) |
| Glass shows scale and place through specular reflections and shadows | HIG Windows | Rims and the pane crescent (board v3) carry the light; no drop shadows (OLED black has nothing to cast on) | Keep: a shadow on black is invisible, so depth goes to light and parallax |
| Shadow opacity rises over text and falls over light backgrounds | WWDC25 219 | The black cores under the pane and the puck are exactly this: a denser shadow where the digits sit | Keep; the core's alpha (0.92) is the "over text" case |
| Clear glass never adapts and needs a dimming layer; use it only over rich content when the content above is bold and bright | WWDC25 219 | Our glass is the Clear variant over the tunnel, with the cores as its dimming layer, and the digits on it are bold white: Apple's own condition for Clear | Keep. Never mix in a frosted (Regular) piece |
| Larger glass shapes cast deeper shadows, lens more and scatter light more softly | WWDC25 219 | The wings (r 95–120), the pane (r 72) and the puck (r 54) share one lens strength | After the library: scale the lens depth with each piece's size (wings strongest, puck weakest) and check V19 again |
| Glass materializes by modulating lensing, never by fading | WWDC25 219 | The 200 ms show grows each piece's thickness back to front | Keep (V14) |
| Concentric corners: inner radius + padding = outer radius | WWDC23 10076 | Core radius = pane radius 72 − inset 22 = 50 | Already true |
| Ornaments overlap the window by 20 pt at the bottom (of a 1280 × 720 pt window) | WWDC23 10076 | The puck overlaps the pane by 34 px (2.8% of 720 pt scales to 11 px on our 396 px pane); the blade floats 22 px *above* the pane with no overlap | Penpot after the library: try the blade overlapping the pane's top by about 11 px, as a visionOS top ornament does, and the puck at 11 px; keep whichever reads better at a glance |
| Text on glass is white; colour goes in backgrounds, not labels; body text is semibold | createwithswift; WWDC23 10076 | Lit wing numerals are their ring's hue mixed halfway to white; the gear and speed are white | Keep the half-white mix; never draw a fully saturated numeral |
| A window turns more transparent while it is being moved | createwithswift | Not used | Rejected: the stack shears with G all the time in a race, and fading the glass then would cost legibility exactly when the driver is busy |

### 10.2 Reference photos

On Drive (TJS's 08:53 yes), never in the repo: R1 Apple Newsroom visionOS window over a room, R2 WWDC25 Liquid Glass hero (folder `4K Masters/vision_spatial_glass/references`). Still wanted, as sources only (the container's proxy blocks image downloads, so the PC fetches them later if needed):
- WWDC25 219 frames at 7:20 (lensing on a bright background) and 16:50 (Clear over media with its dimming layer): the "glass over the tunnel" case.
- HIG Windows page figures of a window with its bottom ornament: the overlap above.
- WWDC23 10076 frames of the eye-focus highlight on an icon: the size of a highlight against its shape.

### 10.3 Motion notes (every animation, its trigger and its calm form)

| Motion | Trigger (telemetry) | Timing | Reduced-flash (`calm`) form |
|---|---|---|---|
| Show | `race_on` turns true | 200 ms, back to front | Same (no flash) |
| Tunnel fill | rpm through `backlash` (60 rpm band) | Spring tracking `rpm_rate` | Same |
| Shift-colour heat | pods past the shift point (`LimiterCue`, 0.25 s) | Eases in | Same, no strobe |
| Pod strobe | limiter held | 10 Hz, 3 frames on / 3 off | Steady lit strip |
| Z-hop glints | upshift | wings 0, pane 40, blade 80 ms; each 20 ms | Off |
| Warp ring | upshift | 220 ms, far to near on k² | Off |
| Pane recede | upshift | 6 px over 200 ms | Off |
| Gear entry | any gear change | Shared `HeroGear` entry | Shared calm form |
| Parallax shear | felt lateral G | Spring ω 12 rad/s, clamped at 1.5 g | Same (driven by data) |
| Horizon tilt | felt lateral G | 4° per g on the same spring | Same |
| Wing toe-in | derived yaw = a_lat / v (from 3 m/s) | ±2° | Same |
| Streaks | speed | 0.019 rail-lengths per m/s per s | Same |
| Cloud drift | speed | 0.4 px/s per m/s | Same |
| Floor glow | throttle (0.9) / brake (0.6) | Smoothed | Same |
| Bead and halo | felt G, body slip | 40 px per g, clamped at 40 px | Same |

Rule check: every motion above is driven by data or by a one-off event in the data; none loops on its own clock except the limiter strobe, which has a calm form (AGENTS rule 7, WCAG 2.3.1).

### 10.4 Telemetry mapping

| Field | Units | Drives | If missing or not finite |
|---|---|---|---|
| `race_on` | flag | Starts the show | Stays as drawn |
| `rpm`, `idle_rpm`, `max_rpm`, `rpm_rate` | rpm, rpm/s | Tunnel fill, head ring, lit wing numerals | Span floored at 1 rpm; rpm held |
| `shift_rpm` / `shift_fraction` | rpm | Shift colour from that ring on, pods | From the car profile |
| `gear` | R, N, 1–10 | Hero gear; upshift → glints, warp, recede | Last gear kept |
| `speed`, `unit` | mph or km/h | Speed digits and unit; streak speed; cloud drift | Streaks stop below 0.5 m/s or when not finite |
| `throttle`, `brake` | 0–1 | Floor glow; brake → three caution arcs on the puck | Taken as 0 |
| `g_lat`, `g_long` | g (smoothed acceleration) | Shear, horizon tilt, light angle, bead, derived yaw | Taken as 0 |
| `slip_deg` | degrees (body slip from velocity) | Bead halo | Taken as 0 |
| `velocity` | m/s | Body slip (via `slip_deg`) | — |

Not read: pitch, roll, `yaw_rate` and per-tyre slip (unverified, section 6); `WheelInPuddle` (not decoded yet).

### 10.5 Edge cases, checked against the code

| Case | What happens now | Finding | Plan |
|---|---|---|---|
| Reverse | Gear shows R; streaks still run *towards* the driver | `speed` is a magnitude, so going backwards looks like going forwards | After the library: when the gear is R, run the streaks and clouds backwards (towards the vanishing point). Cheap and true to the data |
| R → 1 | Counts as an upshift, so the warp plays | Reads as "drive engaged", which is fine | Keep |
| Neutral while rolling | N at the anchor, streaks keep running | Correct | — |
| Paused or in a menu (`race_on` false) | Streaks keep flowing at the last speed if the game keeps sending it | A paused game should look paused | Freeze the flow and the cloud drift while `race_on` is false |
| km/h | Speed up to 3 digits; "KM/H" sits to the right of the speed cells | ~132 px from centre, well inside the pane's clear 256 px | Add a golden-free unit test that the km/h label fits |
| 10th gear | "10" at cap 236 is about 280 px wide | Inside the 512 px clear pane | Add to the digit test |
| Low-revving car (800–4500 rpm) | Measured: 3 and 4 fit, at x 352 and 557 | Sparse but readable | Accept |
| High-revving car (1000–18 000 rpm) | Measured: 11 to 17 fit, 40 to 51 px apart | **Two-digit numerals overlap**: "11" at cap 36 is about 43 px wide, and `numeral_fits` checks the pane, not the gap to the neighbour | Before goldens: thin the numerals to every 2nd (or 5th) when their pitch is under 1.2 × the numeral cap, and add a test for an 18 000 rpm car |
| `max_rpm` ≤ `idle_rpm` (menus, bad profile) | Span floored at 1 rpm | No division by zero | — |
| Limiter held in calm mode | Strip lit steady, no strobe, no glints, no warp | Correct | — |
| Speed above 999 | Shared `TabularSpeed` clamps | — | — |

### 10.6 Depth cues: is the tunnel's ring spacing "real"?

**The geometry.** Rings equally spaced in depth project at scale 1/z. With the nearest ring at 1 and the farthest at `RING_FAR` 0.18 (z from 1 to 5.6), true perspective puts the ring scales at 0.18, 0.19, 0.20, 0.22, 0.23, 0.25, 0.27, 0.29, 0.32, 0.35, 0.40, 0.45, 0.52, 0.62, 0.77, 1.00. Our curve (`RING_FAR + (1 − RING_FAR)·k^1.6`) gives 0.18, 0.19, 0.21, 0.24, 0.28, 0.32, 0.37, 0.42, 0.48, 0.54, 0.61, 0.68, 0.75, 0.83, 0.91, 1.00 (chart: `build2/ring_spacing.png` in the project files).

**What that means.** The pane's half width is 0.40 of the nearest ring, so in true perspective 11 of the 16 rings (everything below about 70% of the rev range) would sit behind the pane (our curve hides 7), and the last two rings would take a third of the whole width. The tunnel would read as "dark, dark, dark, BANG" instead of a rev scale. Our curve spaces the rings so each 1/15 of the rev range is a visible step, and leaves the depth to the other cues.

**The other cues, checked** (the pictorial cues in the CUNY and Hanover perception texts):

| Cue | In the dash |
|---|---|
| Relative size and linear perspective | Rings shrink to one vanishing point; the rails draw the converging lines |
| Texture gradient ("finer in the distance") | Ring stroke 6 px near to 2 px far |
| Atmospheric perspective ("hazy, bluish" far away) | Depth fog: far rings at 55% (`FOG_FAR`), near rings tinted toward `NEAR_TINT` ice white |
| Occlusion | The pane and wings sit in front of the tunnel and bend it |
| Motion parallax | The tunnel and sky shear −6 px per g, the glass layers +6 to +18: opposite directions, so the gap between them reads as depth |
| Motion in depth | The warp ring and the streaks accelerate as they near (k² and the ring curve), as a passing light does |

**From the 3D cluster studies** (Broy et al., AutomotiveUI 2014 and 2015, stereoscopic clusters): putting content in front raised perceived urgency, colour mattered more than depth for urgency, depth made the display read as "clearly structured", and task time and errors did not change. Jitter from shear was a complaint. **Our reading:** the limiter as the ring *nearest* the driver, in the shift colour, is the urgent-in-front pattern, with colour doing most of the work; the parallax must stay smooth (it rides a spring on smoothed G, and V24's motion check covers jitter at the closing review).

**Plan.** Keep the curve. If the depth reads weak at the closing review, the cheap lever is a steeper curve only for the outer three rings (closer to 1/z at the near end, where it costs no rev resolution), not a different law.

### 10.7 The plan for LIBRARY DONE, in order

1. **Code fixes found here** (no assets needed, so first): thin the wing numerals when their pitch is under 1.2 × the numeral cap, with an 18 000 rpm test (10.5); streaks and clouds run backwards in R and freeze while `race_on` is false (10.5); a km/h and a 10th-gear fit test.
2. **Penpot trials** (one board, two strips): the blade overlapping the pane's top by about 11 px and the puck's overlap cut to about 11 px (10.1). Keep the version that reads better in the blind glance.
3. **Lens depth by size** (10.1): wings strongest, puck weakest; check V19 on `02` and `04`.
4. **Library assets**: swap `cabin_env.png` for the library's 4K cabin/sky plate if it has a better match, and pull any glass-edge or caustic texture it offers into the rims; no new ComfyUI job unless the library lacks one.
5. **Goldens twice** on lavapipe, then `dash_gates` and the workspace gates.
6. **PC**: GPU timing and the live CPU bench on the real card (targets 0.3 ms CPU, 0.5 ms GPU).
7. **Closing review**: V1–V26 with a new blind read, accessibility review and rust review; product page.
8. **Merge** under rule 16 and post MERGED here and in the chain thread.

### 10.8 The newly decoded fields (main 60c99c9): what the dash could take

The decoder now reads all 88 FH6 fields. None is confirmed until real-game check C has checked its offset and units, so each line below is a plan for after C, with today's fallback kept.

| Field | Would drive | Today | Plan |
|---|---|---|---|
| `tyre_combined_slip` | The bead halo on the puck, the brief's original signal (the largest of the four tyres) | Body slip from velocity | Switch after C; keep body slip as the fallback when the values are not finite |
| `orientation` (pitch, roll) | The puck's gimbal tilt and the horizon, from the car's real attitude instead of felt G | Felt G | Switch after C. The horizon then tilts with real roll on banked corners, which felt G cannot show |
| `yaw_rate` | The wings' toe-in, directly | Derived a_lat / v | Switch after C; the derivation stays as the fallback below 3 m/s |
| `puddle` (0/1 per wheel) | **New idea: rain on the glass.** When any wheel is in water, a few refracted droplets bead on the wings and slide outward with felt G, then dry off over 2 s | — | Board idea for Penpot after LIBRARY DONE (droplets as small glass domes, one `Piece::circle(...).dome(...)` each, so the same optics bend the tunnel through them). Pairs with [[fh6-weather-telemetry]]: there is no weather field, so the puddle flag is the only wet signal |
| `rumble_strip` / `surface_rumble` | A 1 px shiver of the three nearest rings while a wheel is on a kerb | — | Small and data-driven (passes AGENTS rule 7); try it, drop it if the glance read gets worse |
| `torque`, `power`, `boost`, `fuel`, lap and race data | — | — | Not taken: the cockpit stack stays rpm, gear and speed first; more numbers would muddy the pane |

Note for the merge: main's matrix row for this dash still lists only `rpm`, `max_rpm`, `gear`, `speed`; this branch's row (with `race_on`, `idle_rpm`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity`) replaces it, and `dash_gates` decides the final list.

## 8. Sources

Apple:
- Human Interface Guidelines, Ornaments: https://developer.apple.com/design/human-interface-guidelines/ornaments
- Human Interface Guidelines, Spatial layout: https://developer.apple.com/design/human-interface-guidelines/spatial-layout
- WWDC23 session 10076, "Design for spatial user interfaces" (third-party notes): https://wwdcnotes.com/notes/wwdc23/10076
- WWDC25 session 219, "Meet Liquid Glass": https://developer.apple.com/videos/play/wwdc2025/219/
- Apple Newsroom, "Apple introduces a delightful and elegant new software design" (2025-06-09): https://www.apple.com/newsroom/2025/06/apple-introduces-a-delightful-and-elegant-new-software-design/

Cars:
- BMW Group, "New BMW Panoramic iDrive" (CES 2025): https://www.press.bmwgroup.com/asia/article/detail/T0447361EN/new-bmw-panoramic-idrive-revolutionises-vehicle-operation-visionary-technology-package-for-the-neue-klasse-celebrates-world-premiere-at-ces-2025
- Porsche Newsroom, Taycan interior design: https://newsroom.porsche.com/en/products/taycan/interior-design-18552.html
- Christophorus 392 (2019), Taycan interior: https://christophorus.porsche.com/en/2019/392/taycan-design-interior-e-mobility.html

- Human Interface Guidelines, Windows: https://developer.apple.com/design/human-interface-guidelines/windows
- WWDC24 session 10152, "Create custom hover effects in visionOS": https://developer.apple.com/videos/play/wwdc2024/10152/
- Create with Swift, "Ensuring interface legibility and contrast in visionOS": https://www.createwithswift.com/ensuring-interface-legibility-and-contrast-in-visionos

Depth and 3D clusters:
- Broy et al., AutomotiveUI 2015 (stereoscopic cluster, real-world study): https://www.medien.ifi.lmu.de/pubdb/publications/pub/broy2015autoui/broy2015autoui.pdf
- Broy et al., AutomotiveUI 2014 (stereoscopic cluster, driving simulator): https://www.medien.ifi.lmu.de/pubdb/publications/pub/broy20143D/broy20143D.pdf
- CUNY Sensation and Perception, chapter 7, Depth perception: https://pressbooks.cuny.edu/sensationandperception/?p=389
- Hanover College ISLE, Monocular depth cues: https://isle.hanover.edu/Ch07DepthSize/Ch07MonocDepth.html

No frame grabs were measured in this pass, so no pixel value in the design comes from Apple or car-maker footage.
