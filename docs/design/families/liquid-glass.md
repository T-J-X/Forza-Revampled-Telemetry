# Family: Liquid Glass

**Pattern-setter:** ★ `LiquidGlass` (the default cockpit theme and the design flagship) · **Members:** `LiquidGlass`, `VisionSpatialGlass`; new: `LumenEtch`, `VernierLoupe`, `DichroicLouvre`, `FibreOpticCorona` · **Hero:** [`heroes/liquid-glass/`](../heroes/liquid-glass/) · **Template:** [`liquid-glass-template.png`](liquid-glass-template.png) · **Also owns:** the OSD material ([OSD_DIRECTION.md](../OSD_DIRECTION.md)).

## Language
One piece of sapphire floating in the binnacle. Separation comes from **lensing and moving light**, never from frost or opacity (Apple WWDC25). On our OLED the glass can only lens our own layers, so the look rests on four things: a tint at alpha 28–44, a light-driven 1 px rim, a crown sheen and crescent that move with lateral G, and caustic light confined to the rim band. The centre of the glass is quiet so the gear and speed own it.

**Is not:** a frosted grey slab, marble or smoke wallpaper, a white ring that blooms, or three glowing outlines at the limiter.

## Layout grammar (cockpit, local px from the anchor)
| Zone | Element | Geometry |
|---|---|---|
| Z5 | glass capsule | squircle n 4.2, centre (0, −25), 1280 × 430 |
| Z1 | 16 glass pills (the strip) | y −210, 56 × 20, gap 14, 1110 px span; owns the crown |
| Z1 | crystal prism + gear | octagon R 152 at (0, 18); gear `display` 700 cap 196 |
| Z2 | floating speed | `light` 300 cap 104 at (−395, 18); dimming pad behind; no ghost 8s |
| Z3 | light-pipe ribbon tach | inset 34 px from the rim, a U from the upper-left down through the bottom to the upper-right; redline = last 10% |
| Z4 | pedal capsules + chips | liquid capsules at x 218/262; chips 244 × 52 at x 322 |

## Materials and light
- Tokens: `glass.*` (tint, frost, rim stops, inner rim, crown sheen, caustic, fresnel, crescent, dimming pad), `light.*`.
- **Caustic:** Phase 6 texture or the procedural contour caustic, masked to a 48–140 px rim band, knocked out behind gear and speed, opacity ≤ 0.12 (×2.4 under load), frozen at idle, phase from integrated speed.
- **Prism facets:** alpha `0.03 + 0.24·max(n·L, 0)^4`; lit edges get a 1.5 px chromatic split (`#FF5A7A` / `#38BDF8`).
- **Depth:** capsule ±4 px, ribbon and pills ±2 px, gear and speed 0 (parallax per g).

## Type and colour
- Gear `display` 700; speed `light` 300; chips `light` 400; labels `text` 600 caps.
- Palette roles: `primary` = ribbon and orb tint, `accent` = orb core, `shift` = pill strobe (default violet `#978DFC`), `glassTint` = `mix(primary, #0A1220, 0.72)`. Throttle is ink, brake is warning.

## Motion and signature moments
- **Signature (upshift):** the glow leaves the gear glyph and runs round the rims at 4000 px/s (`motion.gearChange.upshiftGlow`).
- **Limiter:** only the pills strobe (10 Hz). The one steady cue is the **hot meniscus**: the orb core turns white with a red halo and the rim brightens steadily to 85% (no bloom on the rim).
- **Show:** materialise over 200 ms by ramping lensing and rim, never opacity. **Hide:** 1 frame.
- Orb intensity `0.35 + 0.65·rpm/shiftRpm`; `glowSoft` below stage 1, `glowMedium` from stage 3.

## Views
Cockpit only by default. A hood variant may drop the capsule and keep pills, prism and ribbon with glyph keylines (see Modern Digital).

## Do / Don't
- Do keep one glass variant per cluster (regular or tinted), never mixed.
- Do keep the centre clean: nothing textured within 60 px of a hero glyph.
- Don't let the ribbon or orb outshine the gear; don't strobe anything but the pills.

## Phase 6 assets
Caustic lens (4K, tileable) plus normal/refraction pair, blue-noise dither tile, glass normal maps (rounded rect, pill, squircle), fresnel rim mask, droplet/meniscus normals.

## References
Apple Liquid Glass (lensing, travelling highlight, materialise), CarPlay Ultra themes, Lucid Clearview (floating OLED), BMW Panoramic iDrive (one dark band), Mercedes "light blade in a glass lens" (RESEARCH.md §2).
