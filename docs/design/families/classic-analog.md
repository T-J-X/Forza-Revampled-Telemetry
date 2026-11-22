# Family: Classic Analog

**Pattern-setter:** ★ `PorscheAnalog` · **Members:** `PorscheAnalog`, `PaganiHorologerie`, `BugattiTourbillon`, `LfaV10Nurburgring`, `KoenigseggGyro`; new: `RattrapanteSector`, `SapphireMystery` · **Hero:** [`heroes/classic-analog/`](../heroes/classic-analog/) · **Template:** [`classic-analog-template.png`](classic-analog-template.png)

## Language
Horology rendered for a transparent OLED. Every finish is a height or tangent field under **one moving light**: brushed titanium, clous de Paris, sunburst, domed sapphire, dauphine hands whose facets swap brightness as they sweep. Faces are true black; depth comes from stacked layers (bezel, chapter ring, hands, sapphire) and screen-space shadows, never from a lifted face colour. The horology line (Bugatti, Pagani, Koenigsegg) adds skeleton plates, guilloché and complications.

**Is not:** navy or grey dial faces, knurled grey rings, a flat red blade on a red pivot dot, Forza's stock gauge scaled up.

## Layout grammar (cockpit)
| Zone | Element | Geometry |
|---|---|---|
| Z1 | 15 jewels on a flat visor (the strip) | y −318, Ø30, 728 px span, 30 px tall; outside-in |
| Z1 | hub lens + gear | true-black sapphire R 102 at (0, −14); gear `display` 700 cap 150 |
| Z3 | centre tach | bezel 242–262, chapter ring with operating band and red sector, numerals `dial` 600 cap 32 at R 182 |
| Z2 | speed satellite | R 190 at (−452, 16), on the −4 px layer; ticks + track pointer only; digits `display` 700 cap 100, no ghost 8s |
| Z4 | G / pedal satellite | R 190 at (452, 16) |
| Z5 | logo zone | lower dead zone of the tach (never crossed by the needle): crest or Michroma custom name |

The default face is the symmetric 270° sweep. **Track mode** (optional, per car) rotates the scale so the shift rpm stands at 12 o'clock (`layout.classicDial.trackMode`).

## Materials and light
`materials.brushedTitanium` (≤ 24% luminance, two anisotropic lobes facing `light`), `guilloche` (≤ 14 alpha, baked to mip-mapped normals to avoid WRGB moiré), domed sapphire (fresnel to 40/255, crescent 18/255, AR rim `#7B6CFF`), dauphine needle (lit facet ink, shadow facet `#8C929C`, swapped by `n·L`; 6 px lume insert in `primary` with a 2 px black keyline; shadow 3 px away from the light, σ 2, 40%).

## Type and colour
Dial numerals `dial` (Barlow Semi Condensed); gear and speed `display`; wordmark `wide`. `primary` = lume insert and speed pointer (family default `#FF7A1A`, ≥ 25° from warning); `secondary` = logo-zone rule; operating band in lume day at 35%; red only for the red sector, numerals 8–9 and the hub ring from stage 3. Throttle ink, brake warning.

## Motion and signature moments
- **Signature:** the dauphine facet swap during a sweep (light vs shadow facet flip as the needle crosses the light vector), and the hub ring turning warning red only from stage 3.
- **Limiter:** only the jewel visor strobes; the face takes a **steady** red wash that ramps in from 92% of the limiter (LFA precedent), never flashing.
- **Needle:** `needle` spring (T 0.07, ζ 0.75, feed-forward); LFA afterimage above 180°/s.
- **Sweep:** 350 ms up / 300 ms down on the first race start only.

## Do / Don't
- Do keep every static surface ≤ 20% luminance; let the light, not the base colour, make metal read.
- Don't put digits under a needle path; don't use ghost 8s on analog faces.

## Phase 6 assets
Brushed-titanium and black-anodised albedo/roughness, spun and sunburst anisotropy maps, clous-de-Paris / barleycorn normals, sapphire-dome normal map, polished-steel hub matcap, brand logos (dial variant), lume textures.

## References
Porsche 992.2 Classic view and track rev counter, 550 Spyder operating band, Bugatti Tourbillon (Concepto), Pagani Utopia, Lexus LFA meter, Patek/Breguet finishing (RESEARCH.md §4).
