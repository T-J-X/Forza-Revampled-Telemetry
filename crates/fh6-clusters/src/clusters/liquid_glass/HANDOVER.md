# HANDOVER: Dash · Liquid Glass (catalog row 1)

**Thread / branch:** thread 1 of the v2 rebuild plan, on `main` (shared checkout)  **Date:** 2026-11-27  **Result:** ◐ in progress. The second build (every piece a lens) is done; TJX's answer to it is open.

This is a draft. It replaces the handover of the first pilot, which TJX rejected on 2026-11-27 (that file is in the git history). The sections marked *pending* are filled in after the direction check.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| The introduction was posted, and the thread is not archived | yes | the thread's first message |
| `RESEARCH.md` is written, with sources | yes | [RESEARCH.md](RESEARCH.md) (16 Apple behaviours with sources, the CarPlay Ultra facts, the benchmark table in §3.1) |
| The design board is the Penpot page `07 · LiquidGlass`, 4 states per view | yes | page `b568ed3f-64cd-806f-8008-fd9deaa27e48`; board IDs under "What sibling dashes must know" |
| One bespoke ComfyUI asset and one bespoke Blender asset, used, with ledger rows and masters on Drive | yes | [assets/clusters/liquid_glass/ASSETS.md](../../../../../assets/clusters/liquid_glass/ASSETS.md) |
| The module is complete, with no TODOs, and goldens pass | first build | `cargo test --workspace`: every suite ok, 49 goldens |
| 4K renders, the 60-frame sequence and the product page | first build | `mockups/LiquidGlass/` |
| GPU under 0.5 ms; the live bench | yes (second build) | sequence p50 0.344 ms, p99 0.411 ms; 60 s live bench below |
| The direction check got a go from TJX | *pending* | first render posted 2026-11-27; TJX's reply and the second render are under "TJX's direction check" |
| Every line of the closing review is yes | *pending* | |
| TJX has signed off | *pending* | |

## What was built
- **Optics spec:** [docs/specs/optics.md](../../../../../docs/specs/optics.md) replaces card P8-1: the surface profile, exact Fresnel, Snell refraction through a piece, dispersion at the sRGB primaries, Beer-Lambert absorption, the GGX key light, the studio, the pixel footprint filter, display-light compositing, and the corrected sci-fi equations. One test per equation (§13).
- **Kit (`crates/fh6-kit`):** `optics.rs` (the CPU reference), `OpticalGlass` and `Environment` (`paint.rs`), `Shape::Blob` with `Piece` and the smooth union (`shape.rs`), the glass path in `shaders/shape.wgsl`, `tokens.glass.optics`, HDR KTX2 loading, an anisotropic sampler for environment maps, tight bounds for arcs, and a faster PNG loader. GPU tests in `tests/glass_gpu.rs`.
- **Dash:** `mod.rs` (new), `spec.md`, `README.md`, `RESEARCH.md`, this file, and the 8 goldens.
- **Assets:** `tools/asset-forge/src/recipes/liquid_glass.rs` and `assets/clusters/liquid_glass/`.

## Verification output
First build, 2026-11-27 (`preview-export --cluster LiquidGlass`, RTX 3070 Ti Laptop, 3840 × 2160):
```
  01_idle                   0.325 ms GPU   13 draws    50 instances
  02_mid_cruise             0.332 ms GPU   15 draws    56 instances
  03_redline_strobe         0.371 ms GPU   15 draws    76 instances
  04_drift_braking          0.334 ms GPU   15 draws    62 instances
  hood_01_idle              0.297 ms GPU   13 draws    50 instances
  hood_02_mid_cruise        0.304 ms GPU   15 draws    56 instances
  hood_03_redline_strobe    0.340 ms GPU   15 draws    76 instances
  hood_04_drift_braking     0.316 ms GPU   15 draws    62 instances
  sequence: 60 frames, GPU p50 0.336 ms, p99 0.378 ms
```

Live bench of the second build, 2026-11-27 (`FH6_WheelDash --demo --cluster <Name> --stats --bench-seconds 60`, `mock-udp` at 60 Hz, 120 Hz display, no errors). Reports: `mockups/LiquidGlass/live_bench.md` and `live_bench_scaffold.md`.

| Run | Frames | Receive to present p50 / p99 (ms) | CPU per frame p50 / p99 (ms) |
|---|---|---|---|
| LiquidGlass | 6138 | 0.811 / 2.594 | 0.751 / 1.309 |
| Scaffold (`VisionSpatialGlass`, row 8) | 6202 | 0.794 / 2.405 | 0.727 / 1.502 |

The dash's own CPU share is 0.751 − 0.727 = 0.024 ms (limit 0.3). The app's p99 is over 1 ms in both runs: known defect 10 in `docs/PROGRESS.md`, not this dash.

The final `cargo check`, `cargo test`, `cargo clippy`, `rustfmt` and live-bench output goes here at the close.

## TJX's direction check
- **First render** (flat-faced pieces with thin rims), posted 2026-11-27 with the full-frame cockpit render of `02_mid_cruise`, its close-up and the Penpot board.
- **TJX, 2026-11-27 15:18Z, in the project chat:** "exactly we want 2027 future not anything less just modern in all fairness". This thread read it as go with changes (the direction is right, the render is only modern) and asked TJX to confirm that reading.
- **Second render**, posted 2026-11-27: every piece is a lens. The gear sits under a dome that magnifies it, an upshift sends a ripple across it, the capsules are shallow domes, the rev rod is graduated, hard braking fuses the pieces, and the studio light leans to the car's colour. Answer: *pending*.

## Before and after pair
*Pending.* The "before" (the first pilot, rendered from `main` before the rebuild) is kept in `mockups/_before/LiquidGlass/02_mid_cruise_cockpit.png`.

## Assets and masters
See the ledger [assets/clusters/liquid_glass/ASSETS.md](../../../../../assets/clusters/liquid_glass/ASSETS.md) for recipe, seed and licence.

| Runtime file | Made by | Master in the Drive folder `liquid_glass\` |
|---|---|---|
| `studio_env.png` (4096², octahedral, white = radiance 16) | Blender (Cycles, six cube faces), with the ComfyUI studio panorama on its cyclorama | `studio_px.exr` … `studio_nz.exr`; `studio_plate_master.png` |
| `liquid_light.png` (4096², tileable, white = 4 × the mean) | ray traced in the recipe (Snell's law at a rippled water surface) | `liquid_light_master.png` |

## My calls
- **The liquid-light tile is ray traced, not generated.** The SDXL caustic came back as four mismatched quadrants and was not a caustic. The recipe now traces 151 million rays through a rippled water surface, which is exact and tiles by construction. ComfyUI's asset in this dash is the studio panorama.
- **A studio of thin lights.** The first render had one broad softbox and wide rounded edges, and read as glossy black buttons. The studio is now a horizon ring and two thin strips, so each piece shows a thin line along its whole outline.
- **Every piece is a dome, and the gear is really magnified.** A drop of liquid is a dome, and a dome is a lens. The gear is drawn at cap 156 and seen about 1.15 times larger through 96 px of sapphire. Numerals stay where the surface tilts less than 22°, so no reflection crosses them (unit-tested from the profile).
- **The upshift ripple is held under 5°.** The first try (26°) shredded the numeral for a third of a second. At 0.5 px over a 40 px wavelength its edges move 3 px: a wobble, and the gear never loses its shape.
- **No colour splitting.** Sapphire this thin separates red from blue by under a pixel (optics.md §5), so the dash does not show it, and it is not faked.
- **No pool of light under the pieces.** It turned the faces navy, which rule 7 of the design rules forbids. The studio light leans to the car's colour instead, so the rims carry it.
- **A black core under each piece that holds numerals.** Smoked glass would dim the numerals as much as the picture behind them, so it gains no contrast. A core keeps the numerals readable over a bright cabin or bonnet and leaves the rim clear.
- **No caustic plate under the pieces and no relief-mapped crystal.** A caustic needs a diffuse surface to land on, and the pieces sit on an emissive black display.
- **The environment map is 8-bit sRGB with white = radiance 16**, so it stays an optimised PNG; the HDR masters are the EXR cube faces.

## Shared places touched
- `crates/fh6-kit` (the plan's kit step): see "What was built". Three changes help every dash: arcs now have tight bounds (a shallow arc of a large circle used to cover the whole screen and cost 0.5 ms here), PNG textures load several times faster (opaque and flat texels skip the colour arithmetic; pixels are identical), and `docs/specs/optics.md` is the reference for any glass or reflective effect.
- `data/design_tokens.json`: the `glass.optics` block; `arRimTint` corrected to the computed coating colour.
- `docs/specs/phase-8-materials-and-shaders.md`: card P8-1 withdrawn in favour of `optics.md`.
- `tools/asset-forge`: `main.rs`, `sets/mod.rs` and `recipes/mod.rs` gain the `liquid_glass` set.

## Requests for shared code
*Pending the close.* So far: the rule files still describe the first pilot's look (`AGENTS.md` Agent 3A, `SKILLS.md` law 6, `fh6-hmi-design-rules` rule 7 and its `LiquidGlass` brief); this thread repoints them at the close.

## Closing review
*Pending.*

## What sibling dashes must know
- Penpot: page `07 · LiquidGlass` (`b568ed3f-64cd-806f-8008-fd9deaa27e48`). Boards: cockpit 01 `791061e3-930c-8090-8009-02957d24347b`, 02 `791061e3-930c-8090-8009-02960076ab73`, 03 `791061e3-930c-8090-8009-029629233a70`, 04 `791061e3-930c-8090-8009-02965b083da7`; hood 01 `791061e3-930c-8090-8009-02968ca40fbd`, 02 `791061e3-930c-8090-8009-0296a85d0c9f`, 03 `791061e3-930c-8090-8009-0296ae2e1f90`, 04 `791061e3-930c-8090-8009-0296b582d923`; notes `791061e3-930c-8090-8009-02975286a87c`.
- Glass is `tokens.optical_glass(&palette)` on a `Shape::Blob` of `Piece`s, in one `LayerOptions::glass(0.0)` layer, drawn after the light it bends. `docs/kit/README.md` §4 has the recipe.

## TJX's sign-off
*Pending.*
