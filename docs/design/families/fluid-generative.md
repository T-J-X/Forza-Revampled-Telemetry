# Family: Fluid / Generative

**Pattern-setter:** ★ `KineticLiquidMetal` · **Members:** `KineticLiquidMetal`, `CymaticResonanceField`, `ChronoMultiverse`; new: `IsochromaticLoadPath`, `AbyssalShearBloom`, `HarmonographDrive` · **Hero:** [`heroes/fluid-generative/`](../heroes/fluid-generative/) · **Template:** [`fluid-generative-template.png`](fluid-generative-template.png)

## Language
Live physics that the car drives, always **under** the readouts. Liquid metal, ferrofluid, Chladni plates and flow fields respond to G, throttle and slip, and come to rest when the input stops (idle is still). The material is dark: a mercury surface against a dark studio environment, lit by one hard softbox line and a sharp specular from the G-driven light, so lit area stays low on the OLED.

**Is not:** a satin aluminium pipe, a screensaver, animation that moves at idle, fluid crossing the gear glyph.

## Layout grammar (cockpit, KineticLiquidMetal)
| Zone | Element | Geometry |
|---|---|---|
| Z1 | 11 droplet beads (the strip) | y ≈ −306 (shallow arc), Ø30, pitch 72, 40 px tall |
| Z1 | gear well | true black R 152 at (0, −40), 1 px light-driven rim; gear `fluid` 800 cap 200 |
| Z3 | liquid channel tach | 240° glass tube r 176–222 around the well, metaball column r ≈ 16 |
| Z4 | tilting mercury pool | rest level well-centre + 96, tilt = 0.7 × atan(−gLat / (1 + |gLong|)), ≤ 22° |
| Z2 | speed | `fluid` 600 cap 104 at baseline 228 under the well, on a dimming pad, no ghost 8s |
| Z4 | pedal tubes · g / slip readouts | x −470/−420 and x 440 |
| Z5 | slipstream lines | 9 flow lines bending with G at ≤ 6% alpha; absent at idle |

## Materials and light
`materials.liquidMetal`: dark studio LUT (floor `#06080B`, softbox line `#F2F7FF` 3–4 px, falloff, chrome horizon `#0A0C10`, lower band), smooth-union surface (`k` 8–12 px), Blinn-Phong specular (exponent ≈ 120) from `light`, fresnel kicker in `primary` at the grazing edge. Live GPU fluid (Phase 5) clipped to the cluster rectangle, with Phase 6 flipbooks as the budget fallback. The gear is knocked out of the pool with a black 8 px keyline.

## Type and colour
Numerals `fluid` (Exo 2); labels `text`. `primary` tints the fresnel kicker and slipstream; `shift` (family default `#FF5A1F`) = beads at shift and a steady hot glow under the redline end of the channel at the limiter. Throttle ink, brake warning.

## Motion and signature moments
- **Signature:** limiter droplets detach from the meniscus and merge back (smooth union) toward the gear well; the pool sloshes on the under-damped `fluidSlosh` spring (T 0.62, ζ 0.35) in corners.
- **Limiter:** only the beads strobe; the channel end glows steadily.
- **Idle:** surfaces settle to rest; slipstream lines vanish.

## Do / Don't
- Do keep fluid behind and below glyphs; cap pool luminance inside the gear box at 45%.
- Don't let near-white exceed 15% of lit pixels (thin streaks, not chrome slabs).

## Phase 6 assets
Liquid-metal and ferrofluid flipbooks (64 frames, 1024² cells) with matching flow and normal maps, Chladni plates, studio environment LUT, curl-noise textures.

## References
WebGL Fluid Simulation, GPU Gems ch. 38 stable fluids, curl noise (Bridson), Chladni patterns, smooth-union SDFs (Quilez), BMW Digital Art Mode (RESEARCH.md §6).
