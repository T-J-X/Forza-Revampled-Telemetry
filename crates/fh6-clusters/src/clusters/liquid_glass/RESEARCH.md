# Liquid Glass rebuild: research note

Written 2026-11-27 for the rebuild of the Phase 7 pilot. Every statement marked **Cited** was read from the source named on that date. Statements marked **Our reading** are conclusions drawn for this overlay and are open to the direction check. The optics equations and their sources are in [docs/specs/optics.md](../../../../../docs/specs/optics.md); this note covers the look and the behaviour.

## 1. The bar

Apple's Liquid Glass (iOS 26, CarPlay, visionOS glass) as Apple describes and specifies it, on a 55" 4K OLED, inside a Forza cockpit.

## 2. What Apple's material does

| # | Behaviour | Source |
|---|---|---|
| A1 | **Lensing defines the shape.** Light is bent, shaped and concentrated in real time. Earlier materials scattered light; this one bends it. That is what separates a control from its background while it stays transparent. | Cited: WWDC25 219, Dynamics chapter |
| A2 | **Materialise, don't fade.** Objects appear and disappear by gradually modulating the light bending and lensing. | Cited: WWDC25 219 |
| A3 | **Flex and energise.** On interaction the material flexes like gel and lights from within; the glow starts at the touch, spreads through the element and reaches nearby glass. | Cited: WWDC25 219 |
| A4 | **Morph on one plane.** Controls change shape between contexts and stay on one floating plane. A menu opens as a bubble from the button that caused it. | Cited: WWDC25 219 |
| A5 | **Shapes blend when they come close.** In a glass container, the paths of nearby shapes start to blend as they approach; a larger spacing value makes blending start sooner. Shapes can be unioned into one at rest, and morph into each other in transitions. | Cited: SwiftUI `GlassEffectContainer`; "Applying Liquid Glass to custom views" |
| A6 | **Glass cannot sample other glass.** Pieces that share a container share one sampling region, which keeps them consistent. | Cited: WWDC25 323 |
| A7 | **Highlights follow geometry and travel.** Light sources shine on the material and make highlights that respond to its geometry. When the lights move, light travels round the material and defines its silhouette. Sometimes the lighting follows device motion. | Cited: WWDC25 219, Principles chapter |
| A8 | **Shadows know what is behind them.** Shadow opacity rises over text and falls over a plain light background. | Cited: WWDC25 219 |
| A9 | **Bigger means thicker.** At larger sizes the glass simulates a thicker material: deeper shadows, more pronounced lensing and refraction, softer scattering. | Cited: WWDC25 219, Adaptivity chapter |
| A10 | **It has no colour of its own.** Glass takes its colour from the content behind it. Tint is like stained glass, used sparingly on a primary element; never tint several controls at once. | Cited: HIG Color, "Liquid Glass color" |
| A11 | **Two variants, never mixed.** Regular blurs and adjusts luminosity for legibility. Clear is permanently more transparent, belongs only over rich media, and needs a dark dimming layer (35% opacity) over bright content. | Cited: WWDC25 219; HIG Materials |
| A12 | **No glass on glass.** Elements placed on glass use fills, transparency and vibrancy, so they read as a thin overlay that is part of the material. | Cited: WWDC25 219; "Adopting Liquid Glass" |
| A13 | **Glass is the floating functional layer.** It is not for the content layer or app backgrounds, and it is used sparingly. | Cited: HIG Materials; "Adopting Liquid Glass" |
| A14 | **Shapes are concentric.** Capsules (radius = half the height), fixed radii, and concentric radii (parent radius minus padding). No pinched or flared corners. | Cited: WWDC25 356 |
| A15 | **Accessibility changes the material.** Reduce Transparency makes it frostier. Increase Contrast makes elements mostly black or white with a contrasting border. Reduce Motion lowers effect intensity and turns off the elastic behaviour. | Cited: WWDC25 219 |
| A16 | **visionOS glass adapts to luminance.** It limits the range of background colour so a window stays readable, there is no dark mode, and translucent surfaces are preferred to opaque ones. | Cited: HIG Materials, visionOS section |

## 3. CarPlay Ultra clusters

- **Cited (Apple Newsroom, 2025-05-15):** the cluster shows speedometer, tachometer, fuel and temperature gauges, with maps, media, widgets, ADAS and tyre pressure. Apple and the car maker design themes together; the driver picks from a curated set and changes colours and wallpapers. Apple's own captions describe four looks: thin blue bars on the speedometer and tachometer, multi-coloured bars, thick grey bars, and thick red bars.
- **Cited (Auto Express, 2025-06-10):** six layouts, from a full-screen map or music view to minimalist line gauges; the Aston Martin theme is British Racing Green with a script on the speedometer; a centre widget, set from the steering-wheel controls, shows music, maps or car information.
- **Cited (9to5Mac, 2025-06-30):** the theme and the centre content switch from a steering-wheel touchpad; response is immediate.
- **Not found:** theme names, typography details, or any Apple statement that CarPlay Ultra clusters use Liquid Glass. HIG CarPlay does not mention Liquid Glass or clusters. **Our reading:** the cluster benchmark is the gauge grammar (thin or thick bars, a calm centre, one accent colour); the glass benchmark is iOS 26 and visionOS.

### 3.1 The benchmark, feature by feature

Liquid Glass is a catalog concept with no single real display. It is benchmarked against two real things: the **CarPlay Ultra instrument cluster** (Aston Martin launch, May 2025) for what a cluster shows and how calm it is, and **Apple's Liquid Glass material** (iOS 26, §2) for how the pieces look and move.

| What makes the benchmark recognisable | In this dash |
|---|---|
| Speed and revs as bars, thin or thick, in one accent colour (Apple's four looks) | **Matched** for revs: one thin bar of light under a glass rod, in the palette's primary colour. **Changed** for speed: tabular digits, because rank 2 must read in 150 ms and a bar cannot. |
| A calm centre with one large value | **Matched:** the gear, alone in the centre puck. |
| Themes chosen by the driver, with their colours | **Matched:** every colour is a palette role (factory, custom or random), the numerals follow the user's font, and the logo zone shows the car's logo or the custom name. |
| Wallpaper behind the gauges | **Changed:** none. Off pixels are true black on the OLED, and the game is the only picture behind the dash. |
| Fuel and temperature gauges, maps, media, widgets | **Changed:** not shown. Fuel and temperature are not verified against the golden packets; power and boost are, so they take the secondary pill. Maps and media are not a dash's job in a racing game. |
| ADAS and tyre pressure on the cluster (Apple Newsroom) | **Changed:** not shown. Neither is in the telemetry verified against the golden packets, and the game has no driver assistance to show. |
| Six layouts the driver picks from, and a centre widget set from the wheel (Auto Express) | **Changed:** one layout of separate pieces. Each piece is its own zone in the `Geom` table, so the configurator (Phase 10) can move, hide or swap them; the centre holds the gear (rank 1), not a widget. |
| Where the gauges sit, and the typeface | **Not stated** by Apple's release or Auto Express, and no frame grab was measured (§6), so nothing here is matched to them. **Our design:** speed on the left, the gear in the centre, power and boost on the right; OFL faces only (Saira Condensed, Saira, Titillium Web), because the project ships only OFL fonts. |
| Glass elements that float as separate pieces and merge like liquid (A4, A5, A13) | **Matched:** seven kinds of separate pieces; the cabochons fuse on the limiter and the side capsules fuse with the gear lens under hard braking. |
| Shape defined by lensing and by highlights that move with the device (A1, A7) | **Matched:** every piece is computed optics, and the one light rig moves with lateral and longitudinal G. |
| Glass that adapts light or dark to its content, and blurs it (A10, A11) | **Changed:** one clear variant with no frost, because on OLED a frosted fill reads as a grey panel. |

## 4. Why the first pilot was rejected

From its own handover and code (`mod.rs` before this rebuild, `common/glass_plate.rs`, the kit's `glass_delta`):

| Fault | Cause |
|---|---|
| A dark slab | One 1240 × 360 capsule with a 0.94 black backing fill. Glass used as a background panel breaks A13. |
| Blobs stuck on it | The pendant drop and the bead were clipped onto the slab, not merged into it (the kit had no smooth union). Breaks A5. |
| It bent nothing | The glass was drawn first, so its backdrop was black. With nothing to lens, A1 could not happen. |
| The edge was painted | The rim was a fixed white-to-cyan gradient, the "fresnel" was a ramp over edge distance, and the lensing was a fixed radial warp from the shape's centre. None came from a surface normal, so highlights could not respond to geometry (A7). |
| Thickness did nothing | Tint was one flat alpha. Breaks A9. |
| Wrong material constants | Index 1.52 was labelled sapphire; the dispersion was a fixed 1.5 px red/blue split. See optics.md §11. |

## 5. What we take, and what differs here

Our overlay is not an app over wallpaper. It is light on an OLED over a game, and off pixels must stay off.

| # | Principle for the rebuild | From |
|---|---|---|
| P1 | **Separate floating pieces, one per instrument, on true black.** No backing panel. | A13, rule 7 of `fh6-hmi-design-rules` |
| P2 | **Every piece is a real optical object.** Its top surface is a height profile built from its outline; normals, refraction, Fresnel and highlights all come from that surface (optics.md §2 to §7). | A1, A7 |
| P3 | **Light lives under the glass.** Each piece sits over its own instrument light (the rev light, a shift LED, a numeral), so the bevels always have something to bend. With `GlassRefraction` on, the game's cockpit is bent as well. | A1, A6 |
| P4 | **Pieces merge and split like liquid.** One smooth-union field per group; the blend radius is animated on events (gear change, limiter). | A4, A5 |
| P5 | **Bigger is thicker.** Thickness scales with the piece, which strengthens lensing and deepens the absorption tint. | A9 |
| P6 | **One light, and it travels.** One virtual key light and one environment, both turning with lateral G, so highlights run round every silhouette together. | A7, design decision log 2026-11-21 |
| P7 | **Clear, not coloured.** Near-zero tint; colour comes from the light under the glass and from the car's palette. Only the limiter tints. | A10 |
| P8 | **Numerals are a thin layer of the material, never glass on glass.** Gear and speed sit under the flat face of their piece, where the surface does not bend them. | A12 |
| P9 | **Materialise by ramping thickness.** Show ramps the height profile from 0 over 200 ms, so lensing and highlights grow in. Hide stays 1 frame. | A2 |
| P10 | **One variant.** Clear glass over our own dark field; the absorption term is the dimming layer. | A11 |
| P11 | **Reduced flash and reduced motion are honoured.** `ReducedFlash` already exists; the merge and light-run motion scale down with it. | A15 |

**Differences from Apple that we keep on purpose (our reading):**
- Apple's glass adapts tint and switches light or dark by sampling content. Ours sits over black or over a scrimmed cockpit, so it stays in its dark-field state.
- Apple's glass lights from a touch. Ours energises from telemetry: the gear change and the limiter.
- Apple blurs content under Regular glass. On OLED a frosted fill reads as a grey panel, so frost stays off by default.

## 6. Sources

Apple:
- WWDC25 session 219, "Meet Liquid Glass": https://developer.apple.com/videos/play/wwdc2025/219/
- WWDC25 session 323, "Build a SwiftUI app with the new design": https://developer.apple.com/videos/play/wwdc2025/323/
- WWDC25 session 356, "Get to know the new design system": https://developer.apple.com/videos/play/wwdc2025/356/
- Human Interface Guidelines, Materials: https://developer.apple.com/design/human-interface-guidelines/materials
- Human Interface Guidelines, Color ("Liquid Glass color"): https://developer.apple.com/design/human-interface-guidelines/color
- Human Interface Guidelines, CarPlay: https://developer.apple.com/design/human-interface-guidelines/carplay
- "Adopting Liquid Glass": https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass
- SwiftUI `GlassEffectContainer`: https://developer.apple.com/documentation/swiftui/glasseffectcontainer
- "Applying Liquid Glass to custom views": https://developer.apple.com/documentation/swiftui/applying-liquid-glass-to-custom-views
- Apple Newsroom, "CarPlay Ultra, the next generation of CarPlay, begins rolling out today" (2025-05-15): https://www.apple.com/newsroom/2025/05/carplay-ultra-the-next-generation-of-carplay-begins-rolling-out-today/

Press:
- Auto Express, CarPlay Ultra review (2025-06-10): https://www.autoexpress.co.uk/features/367055/apple-carplay-ultra-review-next-generation-car-smartphone-mirroring
- 9to5Mac, CarPlay Ultra hands-on (2025-06-30): https://9to5mac.com/2025/06/30/carplay-ultra-hands-on-reveals-its-standout-best-new-feature/

Pages that did not give usable text: HIG "Designing for visionOS" has nothing on glass, depth or highlights. No frame grabs were measured in this pass, so no pixel value in the design comes from Apple footage.
