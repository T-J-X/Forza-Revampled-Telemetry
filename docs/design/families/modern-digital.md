# Family: Modern Digital 2027

**Pattern-setter:** ★ `Gt7HoodPro` (the default hood theme) · **Members:** `Gt7HoodPro`, `AmgMbuxSuperscreen`, `DriftGlass`, `GlyphDotMatrix`; new: `ApertureNine`, `AeroSmokeTunnel` · **Hero:** [`heroes/modern-digital/`](../heroes/modern-digital/) (hood view) · **Template:** [`modern-digital-template.png`](modern-digital-template.png)

## Language
GT7's discipline (one compact meter, the gear the largest glyph, red only for limits) rebuilt as 2027 production hardware: parametric arcs like CarPlay Ultra, one contrast hue like the Porsche tach, shy micro-labels like BMW's shy-tech. Frameless: rims, glyph keylines and content fall-off define the edges. The scale belongs to the car: the arch is mapped to the **car's shift rpm**, not a fixed 9000.

**Is not:** a SimHub template (thin arc + rounded-square gear box + candy tyre pills + crosshair), glossy gradient slabs, a hue on every widget.

## Layout grammar (hood anchor (1920, 1760); cockpit variants use the cockpit anchor)
| Zone | Element | Geometry |
|---|---|---|
| Z1 | 11 converging diamonds (the strip) | y −262, 34 px (centre 42), pitch 72, 720 px span |
| Z1 | glass gear pod | squircle 216 × 224 at (0, −8); gear `display` 700 cap 180 |
| Z2 | speed | `display` 600 cap 90 under the pod, baseline 214, ghost 8s on |
| Z3 | twin-wing ribbon arch | chord 1180, sagitta 78, apex (0, −150); width 6 → 20 px toward the apex |
| Z4 | tyre quad · G meter | (−700, 40) and (700, 40) |
| Z5 | micro-labels, drift readout | drift readout above the strip only while sliding |

## Materials and light
Glass pod per `glass.*` (tint alpha 44, light-driven rim, inner rim, crescent). Arch top hairline lit by `light`. **Hood legibility:** every glyph gets a black keyline (`type.keyline`, 6/18 px; gear 8/24 px) plus a dimming pad limited to the centre stack. No scrim in hood view.

## Type and colour
Gear and speed `display`; labels `text` caps. `primary` = wing fill (fading from 35% at the ends), stage colours at the head, `shift` = diamonds at shift (default `#978DFC`). Tyres: quiet ink outlines when optimal; thermal step colour only off-optimal; slip outline amber (> 0.08) or white (> 0.18). Throttle ink, brake warning, G-ball ink.

## Motion and signature moments
- **Signature, "Apex lock":** both wings fill from their outer ends and **fuse at the apex exactly at the car's shift rpm**; an axis-lock blade (3 px, white) lights above the apex at fusion. Two comet heads converge outside-in, mirroring the diamonds.
- **Limiter:** only the diamonds strobe; the fused arch holds the shift colour steadily at 85%.
- **Gear change:** token motion plus a light sweep across the pod rim.

## Views
Hood is primary (`layout.hood`, scrim 0%). Cockpit members (`AmgMbuxSuperscreen`, `DriftGlass`, `GlyphDotMatrix`) use the cockpit envelope with the same grammar.

## Do / Don't
- Do map every scale to the car profile (shift, redline).
- Don't use red for structure; don't light tyres when nothing is wrong (alarm by exception).

## Phase 6 assets
Mostly procedural. Blue-noise dither, glass pod normal map, MSDF atlases with an outer distance band wide enough for keylines.

## References
GT7 meters and Race Info Only, CarPlay Ultra parametric arcs, Porsche single contrast hue, MBUX phase-coloured bar, BMW shy-tech, Bosch/MoTeC alarm-by-exception (RESEARCH.md §1–3).
