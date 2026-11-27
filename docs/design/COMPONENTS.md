# COMPONENTS: the shared cluster parts

Every cluster is built from these parts. Phase 7 builds each one **once** in `crates/fh6-clusters/src/common/` on the `fh6-kit` primitives; cluster chats compose them and never fork them. Parameters live in `data/design_tokens.json → components` (and the token groups named below). The Penpot library on page `02 · Components` mirrors them (IDs in [PENPOT.md](PENPOT.md)); the hero frames in `docs/design/heroes/` show them in use.

Conventions for every component:
- **Coordinates** are 4K px at scale 1.0, relative to the cluster anchor.
- **Glance rank** (1 highest … 5) decides size, contrast and whether the part may write to the emissive (bloom) buffer: ranks 1–3 only.
- **Colour** comes only from `colorRoles` (palette roles plus fixed semantics). Unlit parts are neutral; colour appears when something is lit.
- **Motion** uses `motion.*` springs and durations from `FrameInput.dt`; no wall clock, no allocation in `update()` / `draw()`.
- **No flat rectangles:** every surface has at least one of a 1 px light-driven rim, a gradient with a reason, a material texture, or depth parallax.

## 1. HeroGear (rank 1)
- **Anatomy:** the gear glyph; optional holder (hub lens, prism, pod, octagon bay, pill, well) defined by the family.
- **Type:** `type.scale.gearHero` (cap 200) or `gearCompact` (cap 150, the minimum); Race Dash uses `race` 800 at 228–250 cap (45–55% of the panel).
- **States:** `1..10` in `ink`; `N` in `ink` at 72%; `R` in `warning`. Never hidden.
- **Motion:** `motion.gearChange` (out 50 ms to 0.92; in from 1.12 with a 24 px slide on the `fast` spring; 1-frame 60% rim flash) plus one family flourish. 2 Hz pulse allowed after 0.5 s on the limiter.
- **Emissive:** yes (rank 1), soft glow only while the limiter is active.
- **Families:** all. **Penpot:** `HeroGear` set (size × value).

## 2. TabularSpeed (rank 2)
- **Anatomy:** 3 fixed digit cells (`type.tabular.advanceEm` 0.60), ghost `8` in unused leading cells only (`inkGhost`), unit label (`type.scale.unit`, uppercase, `inkMuted`).
- **Rules:** width never changes from 0 to 999; within 420 px of the gear; displayed value changes with 0.6-unit hysteresis; mph or km/h from config.
- **Variants:** `speedHero` (cap 112, `display` 600), `speedCompact` (84), `light` (Saira 200 for glass and ADAS), `segment` (VFD 7-seg, all segments ghosted), `race` (700).
- **Emissive:** no glow by default; may glow in VFD (phosphor).

## 3. Tach (rank 3)
| Variant | Geometry | Fill / motion | Used by |
|---|---|---|---|
| `arc` | 270° ring (start 135°), redline sector from 92% of max | gradient fill, `ribbon` spring, leading glow | Modern, Sci-Fi FUI |
| `bar` | segmented linear bar (Race: 45 segments, 5 px gaps) | step lighting, stage colours above 7300 / 8000 rpm | Race Dash |
| `ribbon` | variable-width arch (7 → 24 px, power 1.6) or a light pipe along a glass outline | gradient primary → white, white comet / meniscus orb at the tip | Modern, Liquid Glass |
| `needleDial` | dial face, chapter ring, ticks every 250/500/1000, numerals at `dialNumeral`, operating band, redline sector; lance needle 14 → 3 px with a lit ridge (light flank / dark flank) and a white lume core | `needle` spring with feed-forward; screen-space drop shadow (3 px, σ ≈ height, 40%) | Classic Analog |
| `chevron` | 24 stepped chevron segments rising left to right (14 cyan, 6 yellow, 4 magenta) | step lighting; ghost segments at 7% | Sci-Fi VFD |
| `liquidChannel` | 240° glass tube, metaball liquid column with meniscus and droplets | `fluidSlosh` spring, smooth-union surface, specular from `light` | Fluid |
- **Track mode** (`layout.classicDial.trackMode`): optional per-car rotation so the shift rpm stands at 12 o'clock.
- **Afterimage:** optional LFA-style ghost needles above 180°/s (`motion.afterimage`).

## 4. ShiftLights (rank 1)
- **Pod optics** (`components.shiftLights.pod`): rounded rect, gap 30% of width, radius 35% of height; lit = 2-stop radial (centre 40% toward white, edge full hue) + 1 px inner rim at 50% white + glow; unlit = white at alpha ≤ 22 with a 12% rim (a grey lens, no colour).
- **Variants:** `race10` (10 top pods, optionally 5-per-side alarm columns), `modern11` (5 converging diamond pairs + centre), `glass16` (16 pills), `analog15` (15 jewels on an arc above the dial), `beads11` (liquid droplets), `vfdStrip` (segmented strobe strip).
- **Fill:** per-car, per-gear thresholds from the car profile (fallback `colorRoles.revStages`), **outside-in**, 0 ms attack, ≤ 50 ms release, 120 rpm off-hysteresis; landing on the centre pod above the gear. At `shiftRpm` all pods take the `shift` colour.
- **Size rule:** the row that strobes is ≤ 42 px tall at scale 1.0. Span ≥ 720 px.

## 5. RevLimiterStrobe (rank 1, a behaviour)
- 10 Hz, 50% duty (3/3 frames at 60 Hz, 6/6 at 120 Hz), on the shift row only. Optional 2 Hz gear pulse after 0.5 s. Each family adds **one steady** limiter cue (a red wash ramp, a lit rim, a hot meniscus, a magenta octagon) that never flashes.
- Reduced flash: 2 Hz sine breath at 40% depth everywhere.
- Mockups show the **on** phase in `03_redline_strobe`.

## 6. LogoZone (rank 5)
- Crest ≤ 96 px or wordmark ≤ 220 × 44 px (`layout.logoZone`). Dial variant: mono white at alpha 215 with a 1 px engraved shadow. Fallback: the custom name in `wide` (Michroma) caps, cap 18 (dial: 11–12), +28–30% tracking, ≤ 14 characters, shrink-to-fit.
- **Placement:** inside analog dials where the needle passes least (Classic: the lower dead zone of the tach); optional elsewhere.
- **Phase 6:** a 4K logo per brand (full colour, mono white, dial variant) in `assets/logos/`.

## 7. MicroLabel (rank 5)
- `type.scale.microLabel` (cap 16) or `nano` (12, the floor): `text` font (Titillium Web 600/700), uppercase, +16–30% tracking, `inkLabel`. "Shy": state labels (TC, ABS, LIM) show their colour only while active.
- Never glows. Shrinks to 85% then elides with a user font.

## 8. GlassPlate (rank 5 material)
- Squircle (exponent 4–4.2) or pill. Tint (`glass.tintAlpha`, regular ≤ 44), dimming pad behind gear/speed, frost, caustic texture (or procedural contour caustics), crown sheen shifted by G, fresnel edge band, 1 px gradient rim + 1 px inner rim, top-left crescent driven by `light`.
- Materialises in 200 ms (lensing and rim ramp); hides in 1 frame. One glass variant per cluster.

## 9. Bezel (rank 5)
| Variant | Build |
|---|---|
| `titaniumRing` | conic brushed banding (two anisotropic lobes facing `light`), 1 px top-lit outer rim, inverted inner rim |
| `chamfer45` | thin gunmetal frame with 45° corners, lit upper-left chamfer at 55%, shadowed lower-right at 18%, rationed screws |
| `carbonFrame` | 2×2 twill (12 px) ring with a diagonal sheen and Allen bolts, 1 px rim |
| `frameless` | rims and content falloff only (Lucid / Modern / ADAS) |
- A bezel is a **thin ring around** a cavity, never a filled plate. Cavities are `oledBlack`.

## 10. AdasRoad (rank 4)
- Camera: eye (0, 1.85, −3.4) m, pitch ≈ 15° down, vertical FOV 35°, near plane 0.1 m; ego at z = 3.2 m in the lower third.
- Road: curvature κ = yawRate / max(v, 3 m/s); 3 lanes of 3.6 m; solid outer edges, dashed inner lines scrolling with speed; matte ground at very low luminance; edges dissolve with masks (no viewport box).
- Path carpet: two-tone, alpha 0.55 at the ego → 0 at the horizon; beyond braking distance v²/(2·9 m/s²) at 35%.
- Objects: ≤ 6 road users, ≤ 2 saturated; neutral grey traffic, in-path lead in `adas.inPath` with FSD corner brackets and distance; 3-step blind-spot arcs; ego = light neutral body tinted by the brand, brake lights glow when braking, yaw = slip in a drift, underglow cyan / amber (assist) / violet (limiter).
- Traffic is a seeded ambient simulation (the telemetry has no traffic data); it is labelled as such in the family spec and never presented as sensor data.
- Phase 6: low-poly glTF cars (≤ 5k tris) per class; procedural box-cars are the fallback.

## 11. GBall (rank 4)
- Rings at 0.5 / 1.0 / 1.5 g (1.0 g brighter, 1.5 g dashed), crosshair at hairline alpha, ball = felt force (`−acceleration`) on the `gBall` spring, trail of the last 500 ms as fading dots, total-g readout. No glow.

## 12. TyreSlipQuad (rank 4)
- Four tyres in a car footprint (hairline body outline), fill = discrete thermal step with a vertical sheen, 1 px rim, temperature beside each tyre, slip outline amber (> 0.08 slip ratio) or red (> 0.18). No glow.

## 13. SignedBar (rank 4)
- F1 MGU-K style bar above the gear: grows from the centre (right = accel / deploy in `go`, left = braking in `warning`), 8–12% of the gear height, `fast` spring, fades out near zero.

## 14. AlarmColumn and DataTile (Race Dash, ranks 4–5)
- Alarm lamps (5 per side), warnings outrank shift; lit = caution (or the shift colour for LIM) with glow, unlit = grey lens. Labels shy.
- Data tile: 1 px hairline outline (no fill), label 25–30% of the value height, value right-aligned in `race` tabular figures.

## 15. Seg7Digit (Sci-Fi VFD)
- Slanted 8°, chamfered segments, 10.5% stroke, gap 18% of stroke; unlit segments at 7% of the phosphor colour; lit segments = phosphor glow + 45% white core. Drawn as SDF segments by the kit (no font).

## 16. LiquidSurface (Fluid)
- Metaball column / pool: smooth-union (`k` 8–12 px), environment banding (`materials.liquidMetal`), specular from `light`, surface tilt `atan(−gLat / (1 + |gLong|))` on the `fluidSlosh` spring; droplets detach at the limiter. Live GPU fluid (Phase 5) or Phase 6 flipbooks underneath, clipped to the cluster rectangle.
