# HANDOVER: Cluster · Retro Cyber (catalog row 19, build #6, ★ Sci-Fi pattern-setter)

**Chat / branch:** RetroCyber cluster chat, `claude/cluster-retrocyber-c0db06`  **Date:** 2026-11-23  **Result:** ✅ complete (Penpot page, bespoke Blender + ComfyUI assets, clean critique pass)

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board has 4 states per view | ✅ | Penpot page `07 · RetroCyber` (page ID `6fe75c5e-bb22-8084-8008-fd924684ddca`) in the master file. It holds 8 boards at 3840 × 2160: 4 states × cockpit and hood, each with the steering centreline, the view envelope and the anchor cross, drawn from the same geometry as `mod.rs`. The board IDs are listed under "What the next Sci-Fi chats must know". The exported cockpit redline board matches the render. Also: the zone table at the top of `mod.rs` and `mockups/RetroCyber/index.html`. |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs`; `cargo test -p fh6-clusters retro_cyber`: 7 unit tests + the golden test (8 images) pass |
| 4K renders + 60-frame sequence in `mockups/RetroCyber/` | ✅ | `cargo run --release -p preview-export -- --cluster RetroCyber` |
| Live demo verified, GPU under 0.5 ms recorded | ✅ | GPU p50 0.32 / p99 0.33 ms (sequence); live run below |
| Critique passed, with notes | ✅ | 4 rounds with the `ui-ux-designer` agent, ending in a clean pass that beats the hero reference; see "Critique" |
| Handover written | ✅ | this file |

## What it is
A VFD tube in a 10 px chamfer45 gunmetal housing (1380 × 500 cavity, cut 58; hood 1680 × 336, cut 48). Inside, four layers with felt-G parallax (0 / 1 / 3 / 4 px):
1. **Phosphor:** procedural seven-segment digits (shared `common::seg7`), pods, chevrons, columns and dots. Every unlit element is an opaque 7% ghost of its own phosphor.
2. **Control-grid mesh:** the Phase 6 `assets/scifi/vfd_hex_mesh.png` tiled at 144 px (about a 6 px cell), multiplied at 42%; a procedural triangular lattice when the asset is missing.
3. **Filaments:** five 1 px wires, multiplied at 55%, crossing the strip, chevrons, speed, gear and scope.
4. **Front glass:** a light-driven 1 px rim and a 12 px crescent at 3.5% peak, both facet by facet; never a tint.

Ranks:
1. The octagon-bay gear (Seg7 cap 220, white phosphor, primary ghosts with the `N` diagonal; N at 72%, R in warning; the soft lobe glow) under the 38 px, 15-pod slanted strip (outer 8 primary, 4 secondary, centre 3 accent; all pods `shift` at the limiter).
2. The Seg7 speed (cap 112, ghost 8s, 0.6 hysteresis, phosphor bloom).
3. 24 chevrons (14 / 6 / 4) with the rpm scale; steady at the limiter.
4. THR / BRK / BST columns (go / warning / secondary, 16 steps, 60% level, no glow); a 13 × 5 dot G-scope (felt G, 0.25 g per column, 0.5 g per row, 500 ms trail); 6-step drift columns flanking the bay on the side the tail steps out (5 / 10 / 15 / 20 / 30 / 45°); a two-cell DRIFT readout (from 8°).
5. SLIP / LOCK / SPIN annunciators (lit only when active, 150 ms hold), labels in Chakra Petch 600, the housing with four slotted screws.

Signature moments: the filament-ignition boot, the gear re-strike with the facet lap, and the strip-only limiter (see `README.md`).

## Renders
- `docs/handover-shots/cluster-retro_cyber-02_mid_cruise.png`
- `docs/handover-shots/cluster-retro_cyber-03_redline_strobe.png`
- `docs/handover-shots/cluster-retro_cyber-04_drift_braking.png`
- `docs/handover-shots/cluster-retro_cyber-hood_03_redline_strobe.png`
- `docs/handover-shots/cluster-retro_cyber-hood_04_drift_braking.png`
- `docs/handover-shots/cluster-retro_cyber-assets-before.png` and `docs/handover-shots/cluster-retro_cyber-assets-after.png` (procedural versus Blender housing and ComfyUI grain, 2× zoom)
- `docs/handover-shots/cluster-retro_cyber-upshift_frame42.png` (sequence frame 42: the first facet of the lap and the re-strike)
- Everything else is in `mockups/RetroCyber/` (gitignored; regenerate with the command above).

## Performance (RTX 3070 Ti Laptop, 3840 × 2160, `preview-export`)
| View | GPU |
|---|---|
| Cockpit, 60-frame sequence | p50 0.316 ms, p99 0.331 ms |
| Cockpit states | 0.31–0.33 ms |
| Hood states | 0.27–0.28 ms (the first hood frame after the view switch reads 0.43 ms) |

History: the first build ran 0.85 ms. 0.33 ms of it was the shared `Bezel` Chamfer45 (see "Shared work"); the rest was full-tube glass strokes and a full-cavity mesh. The housing and glass are now drawn facet by facet and the mesh and scanlines cover only the five phosphor zones.

With the bespoke assets: sequence p50 0.321 ms, p99 0.331 ms; 19–25 draw calls (the plate fills are grouped after the halos and ghosts, so the image switches once per group).

Live demo (`FH6_WheelDash --demo --cluster RetroCyber --stats --bench-seconds 60`, release, desktop with no stand-in game, before the assets landed): 3743 frames in 60 s (62.4 fps), stable, no jitter. CPU per frame was p50 1.24 ms, p99 4.86 ms; of that, upload + record + submit was 0.88 ms p50. That is the shared host cost the pilot also recorded (1.44 / 3.66 ms), not the cluster. Demo mode synthesises telemetry in-process, so the packet-to-present series is empty.

## Assets
Recipe: `tools/asset-forge/src/recipes/retro_cyber.rs`. Build it with `cargo run --release -p asset-forge -- build --set retro_cyber` (≈ 95 s: Blender ≈ 20 s, SDXL ≈ 70 s). Outputs are in `assets/clusters/retro_cyber/`, with the ledger in `assets/clusters/retro_cyber/ASSETS.md` (generator, seed, prompt, licence).

| Asset | Tool | What it adds | Fallback |
|---|---|---|---|
| `housing_cockpit_l160.png`, `housing_cockpit_l110.png`, `housing_hood_l160.png`, `housing_hood_l110.png` | **Blender 5.3**, Cycles on the GPU, 96 spp, OptiX denoise, seed 1907 | The chamfer45 housing modelled in 3D: 45° outer bevel, flat face with an engraved groove at 55%, rounded inner lip, four slotted pan-head screws, brushed gunmetal (roughness streaked by stretched noise). Rendered orthographic at 2 px per 4K px with key lights at −160° and −110°. `draw_housing_art` blends the two by the virtual light's azimuth (lateral G), facet by facet over the ring band, and adds the light-driven 1 px rim. | procedural facets (`draw_housing`) |
| `phosphor_grain.png` (1024², tileable) | **ComfyUI**, SDXL base 1.0 + 4x-UltraSharp, seed 1906 | A macro of lit VFD phosphor powder: grey, made seamless, high-passed, 88–100% display level (±6%). It is tiled at 80 local px (2–3 px grains, under the 6 px hex mesh) and tinted by each lit pod, chevron, column and drift step (gain 1.1), so the phosphor has a fine organic grain instead of flat paint. | flat emission with an 85% lower-edge falloff |

Before / after (2× zoom of the top-left corner, redline state): `docs/handover-shots/cluster-retro_cyber-assets-before.png` and `docs/handover-shots/cluster-retro_cyber-assets-after.png`. In the "after", the strip and chevrons show the phosphor grain, and the housing shows the rendered bevel shading, the groove and the screw heads.

Also used: the shared Phase 6 `assets/scifi/vfd_hex_mesh.png` (control grid), with a procedural lattice as fallback, and Chakra Petch 600 from the font pack.

The recipe writes its own ledger and never calls `ctx.image`, so the shared `assets/ASSETS.md`, `assets.json`, `forge.lock.json` and `contact-sheet.html` are untouched. `build` still re-saves them and can reformat them; I reverted that churn. Registering the set took three one-line shared edits:
- `pub mod retro_cyber;` in `recipes/mod.rs`
- the `"retro_cyber"` arm in `sets/mod.rs`
- `"retro_cyber"` in `SETS` in `main.rs`

Other cluster chats will add the same kind of lines, so expect trivial merge conflicts there.

## Critique
- **Round 1** (`ui-ux-designer` on the 4K crops, the composite and the hero frames): **not yet**. Better parts than the hero, weaker restraint. Must-fix: bloom leaking outside the cavity; a white "plastic key" band through every pod and chevron; rank 4 as loud as rank 3; the gear not dominant; a thin bay rim; labels and bezel too bright. All six were applied.
- **Round 2:** "the cockpit states now beat the hero frame", with one remaining hard fail: the strip and chevron bloom still hazed the bezel (the kit's bloom cannot be masked to the cavity). It also asked to stretch the hood chevrons and to tint the gear cyan. Fixed: pods and chevrons left the bloom pass for tight local glows (σ 3.5 / 2.5), and the hood was re-laid with the full-width staircase.
- **Round 3** (after the bespoke Blender and ComfyUI assets): "nothing lights outside the tube… clearly beats the hero frame on hierarchy and material." Two must-fixes:
  1. The grain was too strong and hid the hex mesh. Fixed: the plate was remapped to 88–100% (±6%) and tiled at 80 px (2–3 px grains), and the mesh went to 42% so the regular grid dominates.
  2. The hood annunciators were too small. Fixed: 180 × 36 px lamps with 19 px caps.
- **Final:** "**Yes, it is a clean pass and beats the hero reference.**" Not taken, and accepted by the reviewer as deferred:
  - the cyan gear tint (the brief says the gear is ink with a 45% white core, N at 72%)
  - the ghost `N` diagonal (shared `seg7`)
  - the dead band between the scrim dome and the housing (host layout / Phase 8 fit)

## My calls
- **The boot runs only on an observed 0 → 1.** A cluster made while racing never saw the 0, so it does not boot; this also keeps the goldens and the sequence free of the boot. The boot is covered by unit tests (`boot_runs_only_on_the_first_observed_race_start`, `sectors_ignite_left_to_right_and_settle`); the review harness never sends `IsRaceOn` 0 → 1, so there is no render of it (see "Shared work").
- **Slip is body slip from velocity** (`Signals::slip_deg`), not rear `TireSlipAngle`: tyre slip is not verified against `tests/golden/packets`. It drives the drift columns, the DRIFT readout and SLIP.
- **LOCK and SPIN are inferred from verified fields.** The packet decoder has no `TireSlipRatio` at all. LOCK = brake > 0.6 at over 5 m/s while decelerating less than 0.35 g per unit of brake. SPIN = throttle > 0.6 with more than 12° of body slip.
- **Corner cut 58 px, not 28.** At 28 px the 1380 × 500 tube reads as a rectangle; 58 is the family table's 64 px outer chamfer less the 10 px ring.
- **Pods and chevrons do not bloom.** The display bloom has a fixed pixel radius and cannot be masked, so at any useful level it lit the bezel. Local glows stay inside the cavity; only the speed (seg7 phosphor bloom at 3%) and the gear's soft lobe write to the bloom buffer.
- **The strip's stage colours are the VFD zones**, not `Palette::rev_stage` (which gave green). `ShiftLights` still decides lit, attack, release, strobe and breath.
- **No white core band in pods and chevrons:** flat emission with an 85% lower-edge falloff. The mesh and filaments give them texture.
- **The gear uses the soft lobe all the time** (`Emission::Soft`), not matte off the limiter: round 1 found the matte white gear lost to the blooming cyan speed.
- **The housing is drawn locally**, with the Chamfer45 look (levels, bevel facing, rims, screws), to stay in budget.
- **The Blender housing is a lit render, not a normal/roughness map.** The kit has no normal-mapped 2D shading, so a normal map could not be lit at runtime. Two baked key lights blended by azimuth give real light-driven bevels and cost two texture fills over the 10 px band.
- **The phosphor plate replaces the flat fill only on blocks the cluster draws itself** (pods, chevrons, columns, drift steps). The seven-segment digits are the shared `seg7` and take no paint, so the gear and speed stay smooth phosphor.
- **No blue-noise dither on the crescent.** It peaks at 3.5% and shows no banding.
- **A user display font replaces the labels and scale only.** The digits are the family's procedural segment face (tokens `type.fonts.segment`).
- **The parallax runs phosphor 0, mesh 1, filaments 3, glass 4 px** (the brief lists 0/1/3/4 and the layer order).

## Shared work for the conductor
1. **`common::bezel` Chamfer45 costs 0.33 ms at 4K** on a 1400 × 520 frame. Its two closed 8-segment polylines (body and gunmetal face) and the two full-octagon rim strokes are evaluated over the whole frame's bounding box. Drawing it facet by facet, as `draw_housing` here does, gives the same look for a few µs. Every Sci-Fi and race-hardware sibling will hit this.
2. **The kit cannot mask bloom to a shape.** The brief's "one emissive buffer clipped to 1440 × 560" is not expressible: `p.clip` limits what is written, not the blur. A `LayerOptions` bloom clip (rect or rounded rect) would let VFD strips bloom inside their cavity.
3. **`common::tach::TachChevron::draw` blooms at the full token intensity and paints a white core band.** Inside a VFD cavity both read as neon UI. RetroCyber draws the run itself through the public `segment`, `segment_color` and `lit`. A `ChevronStyle` (bloom share, core on/off) would let siblings keep the component.
4. **The review harness never sends `IsRaceOn` 0 → 1,** so no family's sector boot can be rendered or goldened. A boot sequence (or a sequence variant starting with `is_race_on: 0`) in `review` and `preview-export` would cover the Sci-Fi signature.

### `common::seg7` second review (requested)
It works for RetroCyber; the tests pass and the digits, ghosts, diagonal neutral and re-strike behave as specified. Findings:
1. **`draw_scanlines` does not do what its docs say.** It fills normal-blend `oledBlack` at alpha 34. That only darkens lit phosphor when the cavity under it is opaque. Over the transparent overlay it would darken the game in stripes. Use `BlendMode::Multiply` (identity on black, and it truly multiplies emission), or say in the docs that it needs an opaque cavity.
2. **Scanline phase depends on each call's rect.** The repeating gradient starts at the rect's snapped `min.y`, so two calls over neighbouring rects do not share one grid. Anchor the period to target y = 0 (start the gradient at `floor(min.y / period) * period`). RetroCyber calls it per phosphor zone; its zones never touch, so this is harmless here.
3. **No blank state for `Seg7Number`.** `update` always lights at least the last digit (`0`). The DRIFT readout needed two `Seg7Digit`s to show all-ghost. An `update_blank()` (or `Option<i64>`) would help.
4. **The diagonal glass ghosts at the full 7%,** so a faint `N` shows behind every gear (round 1 critique). An optional lower ghost level for the diagonal would answer that without touching the hero frames.
5. **`Seg7Geometry::bounds` is conservative:** it adds the slant overhang on both sides at every corner, plus half the spacing. That is fine for layout, but a fit test against an octagon needs the segment polygons (as `the_gear_fits_its_octagon_bay` does).

## Known issues / debt
- **The boot has no render** (see shared item 4); it is unit-tested.
- **The hood over a bright bonnet is unproven.** The tube is opaque black, so legibility is safe, but there is no hood-over-photo composite in `preview-export` yet.
- **The scrim dome leaves about 250 px of dark band above the housing** in the cockpit composite. That is the Phase 8 fit / host anchor, not the cluster.

## Penpot
- The page is `07 · RetroCyber`, ID `6fe75c5e-bb22-8084-8008-fd924684ddca`, in the master file. The file has since been renamed "FH6 WheelDash v2 — Design System."; the IDs are unchanged.
- Boards, each 3840 × 2160 and named `<View> · <state>`:
  - Cockpit `01_idle` `…fd9291c36128`, `02_mid_cruise` `…fd929257c331`, `03_redline_strobe` `…fd927d11c9b0`, `04_drift_braking` `…fd9292f018cd`
  - Hood `01_idle` `…fd92935d6982`, `02_mid_cruise` `…fd9293d5f226`, `03_redline_strobe` `…fd92944d0ada`, `04_drift_braking` `…fd9294c26f8e`
  - Every ID has the prefix `6fe75c5e-bb22-8084-8008-`.
- Each board has a `cluster` group: an SVG built from the same zone table, plus the guides.
- Exports of the cockpit redline and hood drift boards were compared with the renders: they match. The fonts fall back to Penpot's default, as Chakra Petch is not installed there.

## What the next Sci-Fi chats must know
- **The VFD dialect pattern:**
  - Opaque cavity; every unlit element is an opaque 7% ghost (`ghost()` in `mod.rs`, the same formula as `seg7`).
  - Mesh, filaments and scanlines multiply, and only over the phosphor zones.
  - Glass is a facet-wise rim and crescent.
  - Housing drawn facet by facet.
  - No bloom except digits.
- **Module shape:** a `Geom` per view with the zone table in the module docs, unit tests for the envelope, octagon-cut clearance and the gear-in-bay fit, plus a GPU-free `Boot` state machine.
