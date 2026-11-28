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

No frame grabs were measured in this pass, so no pixel value in the design comes from Apple or car-maker footage.
