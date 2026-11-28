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
