# Kit liquid metal: reference swatches

Renders of the kit's liquid-metal paint (`OpticalGlass::metal`, `docs/specs/optics.md` §2 standing waves and §3 conductors), for the Penpot library page `Kit · Liquid metal`. Each is a disc 1520 px across (radius 760 at 4K, user scale 1) on a transparent 1600 px square, drawn by the kit renderer itself (software Vulkan, 28 Nov 2026): the studio environment, the key light at rest (azimuth -135°, 40° up), a shallow meniscus profile (thickness 18, bevel 90, exponent 2), and the pinned still zone `still(70, 60)` on every waved swatch. Waves are `StandingWaves::symmetric(centre, k, count, amplitude, 0.2)`.

| File | Metal | Waves | Wavelength (px) | Amplitude (px) |
|---|---|---|---|---|
| `mercury_still.png` | `Conductor::mercury()` | none | | |
| `mercury_2_waves_squares.png` | mercury | 2 at 90° (squares) | 64 | 3.0 |
| `mercury_3_waves_hexagons.png` | mercury | 3 at 120° (hexagons) | 72 | 3.0 |
| `mercury_3_waves_fine.png` | mercury | 3 at 120° | 36 | 1.4 |
| `mercury_6_waves_twelvefold.png` | mercury | 6 at 30° (twelve-fold) | 80 | 2.2 |
| `mercury_6_waves_coarse.png` | mercury | 6 at 30° | 140 | 3.6 |
| `chromium_still.png` | `Conductor::chromium()` | none | | |
| `chromium_3_waves_hexagons.png` | chromium | 3 at 120° | 72 | 3.0 |
| `chromium_6_waves_twelvefold.png` | chromium | 6 at 30° | 80 | 2.2 |

They are references for layout and look, not runtime assets: a dash draws the surface live, with its own wavenumber, amplitudes and phases every frame. The light and the environment a dash uses change what the metal mirrors; on black, a metal shows only what it reflects.

## In Penpot

Master file `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, page `00 · Kit · Liquid metal` (`347e5ae2-9e0b-80b1-8009-03ab8d7699dd`), added 28 Nov 2026. Images come from this folder at commit `266f471`.

- Overview board `Kit · Liquid metal — overview` (3840×2160): `347e5ae2-9e0b-80b1-8009-03abb25d9481`. The nine discs as instances at 560 px, with captions and the API note.
- Components, local library path `Kit / Liquid metal`: each a 1600×1600 black board holding one locked render rectangle at 1:1 (constraints "scale", so a smaller instance scales its disc). The main boards sit below the overview at y = 2600.

| Component | Component ID | Main board ID |
|---|---|---|
| mercury_still | `347e5ae2-9e0b-80b1-8009-03aba2532abe` | `347e5ae2-9e0b-80b1-8009-03aba244e694` |
| mercury_2_waves_squares | `347e5ae2-9e0b-80b1-8009-03aba25d53bb` | `347e5ae2-9e0b-80b1-8009-03aba25470a2` |
| mercury_3_waves_hexagons | `347e5ae2-9e0b-80b1-8009-03aba26614c4` | `347e5ae2-9e0b-80b1-8009-03aba25e6023` |
| mercury_3_waves_fine | `347e5ae2-9e0b-80b1-8009-03aba26dabb2` | `347e5ae2-9e0b-80b1-8009-03aba266d85e` |
| mercury_6_waves_twelvefold | `347e5ae2-9e0b-80b1-8009-03aba2753b11` | `347e5ae2-9e0b-80b1-8009-03aba26e04ee` |
| mercury_6_waves_coarse | `347e5ae2-9e0b-80b1-8009-03aba27fa6a7` | `347e5ae2-9e0b-80b1-8009-03aba2760532` |
| chromium_still | `347e5ae2-9e0b-80b1-8009-03aba286fbf7` | `347e5ae2-9e0b-80b1-8009-03aba2808ebf` |
| chromium_3_waves_hexagons | `347e5ae2-9e0b-80b1-8009-03aba28ed7ca` | `347e5ae2-9e0b-80b1-8009-03aba28758e3` |
| chromium_6_waves_twelvefold | `347e5ae2-9e0b-80b1-8009-03aba296501c` | `347e5ae2-9e0b-80b1-8009-03aba28fbd09` |

The page sits at the end of the file (the plugin API cannot reorder pages); drag it beside `00 · Asset library` in the Pages panel once that page exists.
