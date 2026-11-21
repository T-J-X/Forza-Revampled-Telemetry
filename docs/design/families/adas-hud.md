# Family: ADAS & HUD

**Pattern-setter:** ★ `TeslaAdas` · **Members:** `TeslaAdas`, `WindshieldHud`, `ArNavWayfinder`, `F35HelmetHmd` (moved here from Sci-Fi: it shares the flight-path marker, boresight and horizon); new: `GeodesicWell`, `Lightpainter`, `TreadSeismograph` · **Hero:** [`heroes/adas-hud/`](../heroes/adas-hud/) · **Template:** [`adas-hud-template.png`](adas-hud-template.png)

## Language
A state machine, not scenery (Tesla FSD): a grey world, one saturated hue for what is in the path, red only for imminent action, and the ego's state shown on the ego model. HUD members follow collimated aviation symbology: thin monochrome strokes, a boresight and a flight-path marker whose gap reads without numbers.

**Is not:** 1990s flat-shaded boxes, a lit navy road surface, a chase camera that hides the road behind the ego, traffic larger than the ego.

**Honesty rule:** the packet carries no traffic, map or route data. Traffic is a seeded ambient simulation and road ahead is predicted from yaw rate (or the world-position trail); both are styled and labelled as such and never presented as sensor data.

## Layout grammar (cockpit)
| Zone | Element | Geometry |
|---|---|---|
| Z1 | 10 pods (the strip) | 64 × 24, 20 px gaps, 820 px span, y −326 |
| Z3 | rev line | 1240 × 4 px at y −282, redline segment |
| Z1 | frameless gear | `display` 800 cap 164 at (0, −92) on a dimming halo |
| Z2 | light speed | `light` 300 cap 90 at (−360, −92), no ghost 8s |
| Z5 | shy assist chips | TC / ABS / LIM, lit only when active |
| Z4 | 3D road scene | band y −50 … 240, edges dissolve (no viewport box) |

## Scene rules (`components.adasRoad`)
- Camera eye (0, 3.4, −6.0) m, pitch 8°, focal ≈ 560 px; ego at z 3.2 m in the bottom 40% of the band; the lead car fully above the ego roof. (The old eye (0, 1.85, −3.4) in AGENTS.md hid the road; this replaces it.)
- Road: curvature κ = yawRate / max(v, 3 m/s); no lit ground — fading 2 px edge lines, dashes scrolling with speed, a two-tone carpet (0.55 → 0; 35% beyond braking distance).
- Objects: ≤ 6 road users, ≤ 2 saturated; traffic `#8C929C` at alpha 0.70; no ambient car in an adjacent lane nearer than ego + 6 m; a blind-spot car is a ghost body (alpha 0.30) plus a 3-step proximity arc.
- Ego: light neutral body tinted 15% by the brand, opaque, faces lit by the G-driven light, brake bar glows when braking, underglow cyan / amber (assist) / violet (limiter); in a drift the nose points into the bend (heading = path tangent + slip).
- Lead car: in-path blue `#3BA7FF` with FSD corner brackets and a distance label in `aviation` (B612 Mono).

## Motion and signature moments
- **Signature:** in a drift the ego yaws into the bend while the carpet turns amber; HUD members separate the flight-path marker from the boresight by the slip angle (14 px/°, clamped ±35°, hidden below 5 m/s).
- **Limiter:** only the pods strobe; the underglow turns steady violet.
- Proximity and assists change in steps with a 150 ms crossfade.

## Views
`WindshieldHud` uses the raised anchor (1920, 1595) and a feathered dark band behind its crown (BMW Panoramic Vision). Hood variants keep the scene and drop the band.

## Phase 6 assets
Low-poly glTF cars (ego coupé, sport sedan, SUV, supercar; ≤ 5k tris, smooth normals, baked AO) with sprite fallbacks; lane and road decal sheet.

## References
Tesla FSD v12–v14 visualisation (semantic colours, two-tone path, proximity arcs, underglow), Rivian Unreal cluster, Mercedes AR-HUD, BMW Panoramic Vision, F/A-18 and F-35 HUD symbology, Taycan's 3-lane / 6-object cap (RESEARCH.md §1–2).
