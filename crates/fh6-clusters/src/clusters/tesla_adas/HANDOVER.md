# HANDOVER: Cluster · Tesla ADAS (catalog row 15, build #5, ★ ADAS & HUD pattern-setter)

**Chat / branch:** TeslaAdas cluster chat, `claude/cluster-teslaadas-2bf6d5`
**Date:** 2026-11-23
**Result:** ✅ built, verified, and through the "materials must dominate" pass.
- **Critique:** round 7, 8.5/10, verdict "physical and rendered". It is a clean pass under the conductor's ruling on the Rivian clause (below). No cluster-level must-fix is left.

## Decided (conductor): out of scope, OLED-black rule
The critic noted that Rivian R1 "renders a fully lit world with environment reflections; this is rendered objects on OLED black". The conductor ruled the lit-environment part out of scope: off pixels must be true black `#000000` on the OLED (AGENTS.md and the HMI rules), so a fully lit 3D world is ruled out by design.
- Rivian is judged only on the quality of the rendered objects, their lighting and their motion.
- The other references:
  - Taycan: beaten.
  - Tesla FSD v14: beaten as a cluster, with car quality matched.
  - Lucid Gravity: on par to beaten.
- Round 7 therefore counts as a clean pass, and the CATALOG cell is ✅.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board, 4 states per view | ✅ | **Penpot** page `07 · TeslaAdas` (id `b568ed3f-64cd-806f-8008-fd974581b17e`, 10 boards; listed below). |
| The render outclasses the board | ✅ | Board export `docs/handover-shots/cluster-tesla_adas-penpot_board-02_mid_cruise.png` (flat shapes) next to `…materials_after-02_mid_cruise_cockpit.png` (rendered cars, lenses, asphalt). |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs` and `scene.rs`. The whole `fh6-clusters` suite passes: 186 unit tests and 49 goldens, including the 8 `TeslaAdas_*` goldens, which were regenerated. |
| 4K renders and 60-frame sequence | ✅ | `cargo run --release -p preview-export -- --cluster TeslaAdas` |
| Live demo, GPU under 0.5 ms | ✅ | Every state ≤ 0.311 ms. Sequence p50 0.188 ms, p99 0.214 ms. Live 60 s: 4766 frames, CPU p50 0.58 ms. |
| Bespoke Blender and ComfyUI assets, visibly used | ✅ | 4 Blender sets (car sprites, LED caps, underglow pools, lamp masks) and 1 ComfyUI plate (asphalt). See Assets. |
| Full-frame before / after | ✅ | `docs/handover-shots/cluster-tesla_adas-materials_{before,after}-{02_mid_cruise,03_redline_strobe,04_drift_braking}_cockpit.png` (2560 × 1440, same states). |
| Critique: "physical and rendered, or vector UI?" | ✅ "physical and rendered" | Round 7, 8.5/10 (below). |
| Handover | ✅ | this file |

Penpot boards on `07 · TeslaAdas`:
- cockpit 4K `…fd9748994766` and hood 4K `…fd974b250b27`;
- cockpit 01–04: `…fd974d1f7478`, `…fd97507a4f18`, `…fd9752dde16f`, `…fd975547f921`;
- hood 01–04: `…fd9758448962`, `…fd975b01ea09`, `…fd975e70d1be`, `…fd97646a9b18`.

Every id starts with `b568ed3f-64cd-806f-8008-`.

The board text-centring pass I had planned was not done. The shared Penpot connection's current page belonged to another chat (`07 · GlyphDotMatrix`), and the rules forbid writing unless `currentPage.name` is mine. The boards are complete; only some labels are off-centre.

## Kit fix (merged)
The conductor's kit fix is in: da34dc7, which re-binds vertex slot 0 after a mesh draw. Main is merged into this branch, the blurred-layer guard is deleted, and the goldens are regenerated. The meshes are now only the fallback; with every asset present, the cluster draws no mesh at all.

## Assets (`assets/clusters/tesla_adas/`, recipe `tools/asset-forge/src/recipes/tesla_adas.rs`, rows in `ASSETS.md`)
Build them with:

```
cargo run --release -p asset-forge -- build --set tesla_adas
```

| Asset | Tool, seed | Replaces | Before → after |
|---|---|---|---|
| `ego_{cockpit,hood}.png` | Blender `CARS`, seed 152. The Phase 6 ego glb, subdivided once, with clear-coated metallic paint, dark glass, rubber and smoked lenses. Lit by a softbox, two strip lights, a cool rim and studio.exr, over a shadow catcher. Rendered through each exact chase camera at 11 yaws. | The flat clay matcap mesh | Full-frame pairs 02 and 04: the ego goes from clay to a lit car with reflections, glass and tyres. |
| `ego_lamps_{cockpit,hood}.png` | Blender `CARS` lamp pass | The unlit red mesh quad and a detached 2D spill | 04: the brake light now sits on the lenses, bloomed. |
| `traffic_{sedan,suv,supercar}.png` | Blender `CARS`. A 4 elevation × 5 relative-heading grid, mirrored for negative angles, with baked shadows. | Clay meshes and 2D shadow ellipses | 02 and 04: the SUV and lead car are rendered bodies. |
| `led_caps.png` | Blender `CAPS`, seed 153. A pill lens with a bevelled rim: smoked acrylic when off, a back-lit diffuser in Standard view when on. | Vector pill outlines | 02: smoked lenses replace hairline outlines. 03 and 04: lit lenses with a hot core, bevel and specular. |
| `underglow_pool_{cockpit,hood}.png` | Blender `POOL`, seed 151 (from round 4) | A Gaussian | **Fixed this round:** the atlas was mirrored relative to the kit camera, because Blender is right-handed. Every chase-camera render is now flipped in post. |
| `asphalt.png` | ComfyUI SDXL + 4x, seed 1507 | A 1% gradient | Raised to 6% ink so the grain reads. Glow-feathered 10 px at the edges, and the bottom 20% fades out. |

All assets keep a procedural fallback: `setup.assets = None` gives the clay meshes, the vector pods and a Gaussian pool. The horizon haze is procedural, two half-ellipses at 6%. The round-3 ComfyUI haze plate stays dropped.

## What it is
A frameless diorama in the column gap. The zone table is at the top of `mod.rs`. Changes since the 8/10 build are in bold.

- **Z1 pods:** 10 **Blender LED caps** at y −296 (60 × 22, pitch 82).
  - Lit caps take the colour from the shared `ShiftLights::color_of`, mixed 25% toward white (10% at the limiter), and the off cap's bevel and specular are added on top.
  - The glow is sigma 4 at 0.40, with no bloom, so neighbours never merge.
  - Only the pods strobe.
- **Z3 rev ribbon:** 6 px (3 px in the hood), primary → accent, with a white comet. It takes caution and warning stages from the pod rpm.
- **Z1 gear:** Saira Condensed 800 in a pill, with n±1 ghosts and the 24 px roll; entry scale cancelled.
- **Z2 speed / Z5 rpm and chips:** unchanged.
- **Z4 scene** (`scene.rs`): the chase camera and band as before. Drawn in this order:
  1. horizon haze;
  2. **feathered asphalt**;
  3. the underglow pool: 0.22, dipping 75% in a drift;
  4. carpet;
  5. lanes;
  6. stop line;
  7. arcs;
  8. **studio-lit sprites, far to near** (the ego among the traffic);
  9. FSD brackets.
- **Sprite mechanics:** each sprite is anchored at its car's projected ground centre and scaled by that point's depth (`fx / (depth · S)`).
  - **Ego:** the nearest yaw cell. Roll is a screen rotation about the ground centre; dive is the lift plus a vertical squash. The lamp mask is added and bloomed while braking.
  - **Traffic:** the nearest elevation and heading cell, opaque. Blending two cells read as a double exposure.

## Performance (RTX 3070 Ti Laptop, 3840 × 2160, `preview-export`, quiet GPU)
| | GPU |
|---|---|
| 60-frame sequence | **p50 0.188 ms, p99 0.214 ms** |
| Cockpit states | 0.22 (idle), 0.24 (cruise), 0.31 (redline), 0.25 (drift) |
| Hood states | 0.23, 0.24, 0.25, 0.26 |

**Live, 60 s** (`--demo --cluster TeslaAdas --stats --bench-seconds 60`): 4766 frames, 0 validation errors.
- CPU per frame: p50 0.58 ms, p99 1.36 ms. Upload + record + submit: p50 0.37 ms.
- The p99 is over 1 ms because other chats were building on this machine.

**Bench note:** without `--bench-out`, the bench overwrites the shared `docs/perf/engine.md`. I reverted it twice; it is not in this branch.

## Critique (`ui-ux-designer`, 7 rounds)
- **Rounds 1–3:** 6 → 7 → 8/10 on layout, motion and colour. All fixed; details are in git history (c5f7b62).
- **Round 4: 7/10.**
  - Findings: a grey pool rectangle, a yaw-blind pool, the haze bar, a wireframe road, no contact shadows, a muddy SUV.
  - All fixed: the black point, the yaw atlas, the haze bar dropped, the asphalt plate, contact shadows, a lighter SUV.
- **Round 5 (materials bar): 7.5/10, fail.**
  - The cars and caps clear the bar.
  - Still vector-like: the haze ellipse, the hard road edges, double-exposed traffic, a neon-rim underglow, a detached brake glow, the drift pool merging with the carpet, and pods merging.
  - All seven fixed.
- **Round 6: 8/10.** One blocker, the lit caps flattening into fills. Fixed: the Standard-view diffuser plus the additive bevel and specular. The haze was halved.
- **Round 7: 8.5/10, "physical and rendered".**
  - No cluster-level must-fix is left.
  - Its two nice-to-haves (a saturated limiter core, the pool at 0.22) are applied.
  - The critic marked a strict FAIL only on Rivian R1's lit environment, which the conductor ruled out of scope (OLED-black rule), so round 7 is a clean pass.

## My calls
- **Sprites rather than meshes for the cars.** The kit mesh pass is one colour per mesh with one light, no textures and no MSAA, so it cannot show paint, glass or reflections. Pre-rendered Blender sprites through the exact cluster cameras give studio lighting at a fraction of the cost: sequence p50 fell from about 0.29 to 0.19 ms. The meshes stay as the fallback.
- **The nearest cell, no crossfade** for both the ego and the traffic. Blending two sprites made a visible double image. The steps are 10°, and the springs make the switch brief. Finer ego yaw steps (5°) would cost another 17 MB of VRAM per view.
- **Mirroring:** Blender's camera is right-handed and the kit's view is not, so a Blender render through the same camera is mirrored. Every chase-camera render is flipped in post. This was a real bug in the round-4 pool atlas.
- **The traffic rig uses the cockpit eye height and elevation bins** for both views. Scale comes from the depth, so the hood view reuses it without a second render set.
- **VRAM:** the textures total about 60 MB (ego 2 × 9 MB, traffic 3 × 6 MB, pools 2 × 12 MB, plus small files).
- The earlier calls still hold (git history, c5f7b62):
  - curvature from `a_lat`, dive from measured deceleration;
  - SLIP and LIM chips only;
  - number-free brackets;
  - no red blind-spot arcs;
  - the private `scene.rs`.

## Requests for the conductor (shared code)
1. **Kit mesh pass:** add per-material sub-meshes, textured or matcap shading, and MSAA. These would let live 3D replace the sprites (true continuous yaw, steer angle). The sprites are the workaround.
2. **`common::shift::draw_pod`:** a lit pod's glow erases the neighbouring unlit outlines. This cluster no longer uses `draw_pod` when the caps asset is present.
3. **`common::adas_road` review** (its second review). Defects:
   - It draws a distance number on the synthetic lead car (`draw_lead_marker`): fake data.
   - Its curvature comes from `Signals::yaw_rate`, whose sign is unverified. `a_lat / v²` is verified (`scene::path_curvature`).
   - It has no measured-deceleration stop line, only a fixed 9 m/s² step.
   - Its proximity arcs show at standstill and turn full red alongside.
   - Its 15-point arc polylines exceed the 8-segment rule.
   - It draws procedural hulls, not the glTF cars or sprites.
   - Folding in `tesla_adas/scene.rs` (curvature, stop line, sprites, wake, speed-gated arcs) would give the family one scene.
4. **asset-forge, shared files:**
   - I registered the `tesla_adas` set: one line each in `recipes/mod.rs`, `sets/mod.rs` and `main.rs` `SETS`. This was needed to build through the pipeline; the merge with main kept every set.
   - The ledger's `FORGE_DIRS` does not cover `assets/clusters/`. **Don't simply add it:** `sweep` would then delete un-ledgered cluster files. It needs a per-cluster allowlist.
5. **`fh6-app` bench:** its default `--bench-out docs/perf/engine.md` makes every cluster chat's live run dirty a shared file. Default it to `target/`.
6. **Family spec `adas-hud.md`:** in-path colour (lead tint plus brackets), no red blind-spot arcs, no lead distance label, and studio-lit sprites as the car standard.

## What the next ADAS & HUD chats must know
- Reuse the `CARS` rig. It takes a shot list (glb, eye height, depth, pitch, scale and anchor) and reproduces `Camera25` exactly; just mirror in post. Anchor sprites at the projected ground centre and scale them by `fx / (depth · S)`.
- Lit lenses: render the diffuser in Standard view (AgX flattens it), then add the off-state render on top to keep the bevel and specular.
- Anything big and soft on black (haze, pools) reads as a shape on the OLED. Keep it under about 6–20% and fade it on every side before any clip edge.
- Renders in `docs/handover-shots/cluster-tesla_adas-*`:
  - the full-frame materials before/after pairs;
  - the hood drift;
  - the 03 and 04 crops;
  - upshift frame 43;
  - the Penpot board export.

  The rest is in `mockups/TeslaAdas/` (gitignored).
