# HANDOVER: Cluster · Bosch DDU (catalog row 12, ★ Race Dash pattern-setter)

**Chat / branch:** cluster chat `claude/cluster-boschddu-3653d6`  **Date:** 2026-11-23  **Result:** ✅ complete. The final independent critique gave a clean **PASS at 8.5/10**.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board has 4 states | ✅ | Penpot page `07 · BoschDdu` (page ID `b568ed3f-64cd-806f-8008-fd94834f5850`) has four 3840×2160 boards with the cockpit binnacle guides (envelope and anchor); IDs below. The HTML board is `mockups/BoschDdu/index.html`. |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs` (zone table, `Geom`, `Faces`, event page, derived lamps, load quad); 4 unit tests and the golden test pass |
| 4K renders + 60-frame sequence | ✅ | `cargo run --release -p preview-export -- --cluster BoschDdu` |
| Live demo, GPU under 0.5 ms | ✅ | see "Performance" |
| Critique passed, with notes | ✅ | 5 rounds with two independent `ui-ux-designer` critics, using the design-critique structure: 6 → 7.5 → 8 (conditional) → 7 (fresh critic, not pass) → **8.5 PASS** |
| Bespoke Blender + ComfyUI assets, visibly used | ✅ | see "Assets"; before / after crops below |
| Handover written | ✅ | this file |

Penpot boards: `BoschDdu · 01_idle` `b568ed3f-64cd-806f-8008-fd951f47f89c`, `02_mid_cruise` `…fd95256171ef`, `03_redline_strobe` `…fd9531913497`, `04_drift_braking` `…fd9541fc429d`.

The boards predate the last polish round: lens outlines removed, the `×1000` unit, `TYRE LOAD`, and the SemiBold gear. The Penpot tab was suspended when I tried to sync them ("no heartbeat"); the renders are the reference.

## What it is
An honest race display in a machined carbon housing.

**Housing.** An open-bottom U (1400 × 540 from y −306, top corners r 96): a 14 px carbon face, a 28 px 45° chamfer into a true-black cavity, and four Allen cap screws. The legs dissolve into the steering column over the last 96 px.
- The housing is the **Blender render**: clear-coated carbon built on the **ComfyUI** fibre macro.
- There is one render per key-light azimuth (−160° and −110°). They are crossfaded by the virtual light, so lateral G swings the highlights.
- The housing parallaxes ±4 px, snapped to whole pixels.
- Without the files, a procedural housing takes over: twill, facet-lit chamfer, rims and bolts.
- The bonded AR cover glass adds a 7% tinted 1 px rim and an 18/255 crescent above the light bar.

Ranks:
1. **Pods and gear.**
   - Ten 84 × 36 LEDs behind microprism lenses in a recessed light bar. There are no outlines: each lens is a two-stop radial, 5 px concentric prism rings and a top glint. A lit lens has a 6 px glow at 25% and bloom at 40% of the token intensity.
   - Under them, the gear in Saira Condensed **600** at cap 240. It is crisp, with no bloom.
2. **Speed** on the left in Saira Condensed 700 at cap 112, tabular, with ghost 8s at 2% on screen and the unit under it.
3. **The 45-segment rev bar,** numbered in thousands from the car's limiter, with a `×1000` unit at its end.
   - The cells are drawn here from `TachBar`'s fill model, flat and without bloom.
   - Nominal cells are `primary` at 72%, pulled 35% toward the label grey. The caution and warning stages keep their full colour.
4. **Rank 4:**
   - The long-G bar, drawn from `SignedBar`'s spring: a 4 px track, ±1 g ticks, and a flat 8 px fill (ink when accelerating, `warning` when braking), 36 px above the gear.
   - The **tyre-load quad** under the gear.
   - The boost and power tiles on the right (`DataTile`, open frame; kW under km/h).
5. **Six alarm lamps,** drawn from `AlarmColumn` state: TC, ABS and SLIP on the left; LIM, HB and LNCH on the right. They are flat lenses with no glow. Unlit labels sit at 35% on screen.

Signature moments:
- **Shift lights.** Pods light outside-in in the frame each stage threshold is crossed, landing on the centre pair over the gear. At the shift point all ten take `shift`. At the limiter only the strip strobes (10 Hz, `ShiftLights`), and the rev cells turn steady ink.
- **Pod spill (the steady cue).** A non-emissive reflection of each pod on the upper chamfer: one hot spot per pod at 35% peak, ramping in over 150 ms. It never flashes.
- **Shift event page.** On every upshift the right tiles swap for 4 s:
  - `SHIFT RPM` is the peak rpm in the 250 ms before the gear byte changed.
  - `LATE BY RPM` or `EARLY BY RPM` shows how far that was from the shift point. It turns `caution` outside ±150 rpm and is never red.
  - The swap steps the old page out, then the new one in (3 × 25 ms each), so two pages never overlap.

## Telemetry honesty
- **Decoded and golden-tested:** rpm (current, limiter), the shift point, gear, speed, longitudinal and lateral G, throttle, brake, clutch, handbrake, body slip (from velocity), `drivetrain_type`, power, boost.
- **Not decoded, so not shown:** tyre temperatures, slip ratios, combined slip, fuel, lap data, race position, distance. The brief's FUEL and TYRE lamps, the thermal quad and the LAST / Δ BEST page all wait on the decoder.
- **Derived (my calls; thresholds proposed):**
  - **TC:** power-oversteer onset. Throttle > 20%, body slip > 3° and growing faster than 8°/s, speed > 4 m/s, not front-wheel drive.
  - **ABS:** threshold braking. Brake > 30% and deceleration ≥ 0.85 g.
  - **SLIP:** body slip ≥ 10°.
  - **LIM:** rpm ≥ 98.5% of the limiter.
  - **HB:** handbrake > 10%.
  - **LNCH:** launch armed. Standing, in gear, throttle > 50% with the brake, handbrake or clutch held.
  - **Tyre load:** weight transfer from the smoothed G (0.18 per g front/rear, 0.30 per g left/right; load conserved). Nominal load shows 20% on screen, a loaded corner 60%, and a lifting (< 0.25) or overloaded (> 1.9) wheel turns `caution`.

## Assets (`assets/clusters/bosch_ddu/`, own ledger `ASSETS.md`)
Recipe: `tools/asset-forge/src/recipes/bosch_ddu.rs`. Build with `cargo run --release -p asset-forge -- build --set bosch_ddu` (through Bash).

| File | Tool | What it adds |
|---|---|---|
| `carbon_weave.png` (1024², tileable) | **ComfyUI** (SDXL base + 4x-UltraSharp, seed 1203): a macro of a raw carbon-fibre tow | The filament detail. SDXL draws filaments well but not a regular weave (the first try was ribbed resin), so the recipe lays the macro into an exact 2×2 twill: 16 tows per tile, crowned tows, 30% grooves, over / under dips, filaments streaked along each tow, horizontal tows 20% brighter so the diagonal reads. It feeds the Blender material's tow colour, roughness and the bump under the clear coat. |
| `housing_l160.png`, `housing_l110.png` (2824 × 1092, 2 px per 4K px) | **Blender 5.3** Cycles (seed 1204, 128 samples) | The housing modelled in 3D: outer wall, rounded edge, flat carbon face, rounded lip, the deep 45° chamfer and four machined Allen cap screws. Clear-coated carbon (coat roughness 0.05) under a studio HDRI and a key light 35° above the plane, one render per azimuth. The cluster dims it to 55% linear so the static structure stays under 20% on screen. |

Before / after, the same crop of the housing at 1:1:
- `docs/handover-shots/cluster-bosch_ddu-housing_before.png`: the procedural fallback.
- `docs/handover-shots/cluster-bosch_ddu-housing_after.png`: the Blender housing on the ComfyUI weave.
- `docs/handover-shots/cluster-bosch_ddu-carbon_weave_crop.png`: the weave tile.

The Phase 6 `assets/classic/carbon_twill_lit.png` is now used only by the procedural fallback.

## Renders
- `docs/handover-shots/cluster-bosch_ddu-01_idle.png`, `-02_mid_cruise.png`, `-03_redline_strobe.png`, `-04_drift_braking.png` (1:1 crops)
- `docs/handover-shots/cluster-bosch_ddu-event_page.png` (sequence frame 50)
- `docs/handover-shots/cluster-bosch_ddu-cockpit.png` (the composite, cropped to the lower centre and scaled to 1280 × 720)
- Everything else is in `mockups/BoschDdu/` (gitignored).

## Performance (RTX 3070 Ti Laptop, 3840 × 2160)
| Where | GPU |
|---|---|
| `preview-export`, 60-frame sequence | **p50 0.168 ms, p99 0.177 ms** |
| `preview-export`, states | 0.12–0.26 ms (the strobe state, with the pod bloom, is the highest) |
| The procedural fallback (no assets), sequence | p50 0.249 ms, p99 0.266 ms |

The first build drew the housing as five full-area fills and cost 0.513 ms. Drawing it as small band pieces, then using the baked image, cut that away.

**Live demo:** `FH6_WheelDash --demo --cluster BoschDdu --stats --bench-seconds 60` (release, with the baked assets, desktop, no stand-in game, synthetic demo data, 120 Hz display) ran 60 s with no errors and 3795 frames.
- **Building the quad batch** (this cluster's `update` + `draw` recording): p50 0.007 ms, p99 0.059 ms.
- **CPU per frame:** p50 1.08 ms, p99 3.78 ms. Most of it is the host's upload + record + submit (0.76 ms p50), the pre-existing cost the pilot also recorded.

I read the numbers; I did not watch the overlay window for jitter.

## Critique
- **Rounds 1–3** (first critic): 6 → 7.5 → 8/10. The fixes:
  - real twill;
  - dark unlit parts;
  - no lens outlines;
  - the event page in caution only, early / late;
  - a per-pod spill;
  - the G-bar spacing;
  - readable tyres;
  - the crescent above the pods;
  - seamless band joints.
- **Round 4** (a fresh critic, not anchored on the earlier rounds): 7/10, not pass. Must-fixes:
  - the pods read as iOS buttons;
  - the glossy saturated rev bar;
  - the weave read as brick, not twill;
  - an MSDF notch in the Bold "N";
  - the load quad was too bright and muddy.

  All five were fixed. To do it, the module draws the rev cells, the G bar and the lamps itself from the shared components' state, without their bloom and glow.
- **Round 5:** **8.5/10, PASS.** "The glance order now holds in every state… colour shows up only by exception, and each hue does one job." Its one polish note, the slivers in unloaded tyres, is fixed. A lifting wheel now fills its cell in `caution`, and fills under a quarter of the cell are left out.
- **Conductor's professional-hardware bar** (MoTeC / Bosch / AiM / Cosworth): black, high contrast, exact tabular data on a strict grid, real shift-light and limiter behaviour, no ornamental glow. The cluster has no sci-fi effects. Its only light effects are the LED pods, the pod reflection on the chamfer, and an 18/255 cover-glass crescent.

## Shared change requests (for the conductor)
1. **`common/tach.rs` `TachBar`:** a no-bloom option and a flat-cell option, so siblings needn't re-draw the cells as BoschDdu does.
2. **`common/race.rs` `AlarmColumn`:**
   - a glow opt-out, and the unlit rim from the palette (0.12 linear reads as a grey button);
   - make the lamp `level` public, so a locally drawn lamp can fade out like the shared one. BoschDdu's lamps switch on and off.
3. **`common/signed_bar.rs` `SignedBar`:** a flat 8 px square-ended fill, a short notch and ±1 g ticks (what BoschDdu draws locally).
4. **`common/shift.rs` `draw_pod`:** a lens style with no inner rim and adjustable bloom (what BoschDdu draws locally).
5. **Kit text:** the MSDF atlas for Saira Condensed **Bold** shows a notch where the "N" diagonal meets its stem (01_idle at 1:1). SemiBold is clean. Check the atlas range and corner colouring.
6. **Decoder (`fh6-core::telemetry`):** tyre temperatures, slip ratios, combined slip, fuel, laps, position, distance, all golden-verified.
7. **Per-gear shift rpm** in `Signals` (the brief wants per-car, per-gear thresholds).
8. **`docs/kit/README.md` §13:** two lessons.
   - Band pieces must meet on hard clips at whole pixels; round any translation that moves them.
   - A baked image drawn through the same band pieces costs less than procedural layers.
9. **asset-forge (shared files touched, one line each):**
   - `main.rs` `SETS` gains `"bosch_ddu"` (rustfmt then split the list onto lines);
   - `recipes/mod.rs` gains `pub mod bosch_ddu;`;
   - `sets/mod.rs` gains the `"bosch_ddu"` arm.

   These mirror the RetroCyber and Gt7HoodPro chats' changes. The forge also rewrote the shared `assets/ASSETS.md` / `assets.json` / `contact-sheet.html` / `forge.lock.json`, mislabelling my files "v1 project texture". I reverted those four; the cluster's own ledger is authoritative.

## My calls
- **Gear in SemiBold (600), not Bold:** the Bold "N" has the MSDF notch.
- **No bloom on the gear or the rev bar:** rank 1 and rank 3 read as a crisp TFT, and the bloom read as a game HUD.
- **The pod spill peaks at 35%, not 60%:** at 60% the critique read it as an emitter.
- **Event page:** shift rpm and early / late, instead of LAST / Δ BEST (lap data is not decoded).
- **Lamps:** TC, ABS, SLIP | LIM, HB, LNCH, each with a source today.
- **Tyre-load quad instead of the thermal quad** (no tyre temperatures).
- **Housing renders dimmed to 55% linear** so the static structure stays under 20% on screen.
- **Ghost 8s at 2% on screen:** brighter, "74" read as "874".
- **The shift colour is the palette's** (family default `#B27CFF`); the brief's `#2F6BFF` is not in the tokens.
- **Hood:** the catalog lists cockpit only. Forced into the hood view, the layout draws at 0.7 scale, which fits the hood envelope (unit-tested).
- **`common/seg7.rs` was not used:** the DDU is a TFT, so I did not review it.

## Known issues / debt
- The shared change requests above.
- The Penpot boards lag the last polish (see the top).
- The derived lamp thresholds are proposals; tune them from a logged drive.
- Where the cluster sits in the cockpit composite is the host `Layout`, not this module.

## What the next Race Dash chats must know
- These pieces are reusable; copy them, or ask the conductor to promote them into `common/race.rs`:
  - the band-piece housing (`Geom::band`, `band_clips`, `clipped`);
  - the baked-housing crossfade;
  - the local lens / cell / G-bar drawing;
  - the derived lamp conditions;
  - `EventPage::levels`;
  - the tyre-load quad.
- The asset recipe pattern for a housing: ComfyUI material macro → exact pattern in Rust → Blender 3D housing per key light.
- Alphas are linear: use `bezel::on_screen(level)` for every "as seen on screen" level.
