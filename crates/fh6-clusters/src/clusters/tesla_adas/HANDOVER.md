# HANDOVER: Cluster · Tesla ADAS (catalog row 15, build #5, ★ ADAS & HUD pattern-setter)

**Chat / branch:** TeslaAdas cluster chat, `claude/cluster-teslaadas-2bf6d5`  **Date:** 2026-11-23  **Result:** ✅ built, verified, critique passed (8/10). One kit bug blocks the hood budget (see **Read first**).

## Read first: kit renderer bug (urgent, shared)
After a mesh draw, `fh6-kit`'s shape pass never re-binds vertex slot 0 (the shape instances).
- **Where:** `Meshes::record` binds the mesh's vertices to slot 0, and `Renderer` (`renderer.rs`, the `DrawCmd::Mesh` arm) resets the pipeline and bind groups but not slot 0.
- **What breaks:** every 2D shape recorded after a mesh in the same segment.
  - In `preview-export` those shapes are silently lost.
  - In the live overlay they fail wgpu validation (*"Instance N extends beyond limit 0 imposed by the buffer in slot 0"*), and the device is dropped and recreated every second.
- **Reproduce:** run `FH6_WheelDash --demo --cluster TeslaAdas --stats` without this cluster's guard. 0 frames are presented, because the host draws the `--stats` text after the cluster. The OSD live view, or any host draw after a mesh cluster, would hit the same failure.
- **The fix (one line):** in the `DrawCmd::Mesh` arm, after `self.meshes.record(...)`, add `pass.set_vertex_buffer(0, self.upload.slice(offs.instances..));`.
- I asked the conductor for permission to make it; no answer had arrived before this handover, so the kit is untouched.

What this cluster does meanwhile, in `mod.rs` `draw`:
1. Every 2D draw comes first, and the meshes come last.
2. The tail-light bar is an unlit mesh quad, so it can sit on top of the ego.
3. The draw ends with a **guard**: a 1 px, alpha-1/255 dot in a `LayerOptions::blurred(0.5)` layer. That forces a segment break, so the host's next pass re-binds slot 0.
   - It costs about 0.06 ms in the cockpit and about 0.14 ms in the hood.
   - **Delete that block once the kit fix lands.** The hood then returns under budget (see Performance).

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board, 4 states per view | ◐ HTML board | `mockups/TeslaAdas/index.html` (4 states × cockpit and hood, crops, cockpit composite, 60-frame sequence). The Penpot page `07 · TeslaAdas` doesn't exist yet: `docs/design/PENPOT.md` on main says "not created". The conductor's rule was to use it if it exists and never create duplicates, so I used the fallback. |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs` (zone table, `Geom` per view, `Faces`) and `scene.rs`. 8 unit tests plus the golden test (8 images) pass. |
| 4K renders and 60-frame sequence | ✅ | `cargo run --release -p preview-export -- --cluster TeslaAdas` |
| Live demo, GPU under 0.5 ms recorded | ◐ | 60 s live with `--stats` and the guard: 2861 frames, 0 validation errors. GPU: cockpit sequence p50 0.333 ms; hood redline 0.60 ms with the guard (0.46 without). |
| Critique passed, with notes | ✅ | `ui-ux-designer` scored it 6 → 7 → **8/10, pass**, over 3 rounds (below). |
| Handover written | ✅ | this file |

## What it is
A frameless clay-and-light diorama in the column gap. The full zone table is at the top of `mod.rs`.
- **Z1 pods:** 10 pill pods at y −296 (60 × 22, pitch 82), filling outside-in. Only these strobe at the limiter (10 Hz, `ShiftLights`).
- **Z3 rev ribbon:** 6 px wide (3 px in the hood), ±640. It runs primary → accent, with a 12 px white comet at its head, a hairline track and a dim redline share. It warms with the pods: caution from the first amber pod's rpm, warning from the first red pod's.
- **Z1 gear:** Saira Condensed 800 at cap 170, in a 1 px vertical pill (±72 × −258..−30) with n+1 above and n−1 below as Titillium ghosts.
  - On a change the shared `GearGlyph` rolls the glyph 24 px; its 1.12 entry scale is cancelled inside the pill.
  - The ghosts roll 24 px, stay hidden until the glyph settles, then fade in.
  - On a limiter held past 0.5 s the gear pulses at 2 Hz (bloom).
- **Z2 speed:** Saira 300 at cap 112, with no ghost 8s. The unit is right-aligned under the digits.
- **Z5 right side:** rpm in Saira 400, in steps of 50, with shy chips: SLIP (a latched drift) and LIM (the shift point).
- **Z4 scene** (`scene.rs`): a 1440 × 260 band from y −20, chase camera eye (0, 3.4, −6), 8° pitch, focal 440.
  - **Ground:** faint, at most 6% luminance, between the outer edges.
  - **Path carpet:** two-tone, 0.40 at the ego, stepping to 35% beyond the stop point.
  - **Lanes:** perspective-tapered edges and dashes, dashes scrolling at true ground speed, dissolving towards the horizon, the sides (120 px) and the band edges.
  - **Cars:** the Phase 6 glTF clay cars through the kit mesh pass (matcap, opaque, darkened rather than translucent).
    - The ego is a brand-pearl twin: light neutral, 15% brand tint, an accent-tinted rim.
    - The lead car is light clay with a 20% tint, marked only by number-free FSD brackets.
    - Traffic is `#8C929C` at 0.70; blind-spot ghosts sit at 0.30.
  - **Blind-spot arcs:** 3 steps (primary 40%, caution 70%, caution 90%), shown only above 3–6 m/s.
  - **Underglow:** a Gaussian at 0.40, sigma 22% of the car width. Primary normally, amber in a drift, the limiter violet (palette `shift`) on the limiter.
- **Hood:** pods at y −208, gear cap 96, speed cap 84, a legibility halo behind the top zone, and the scene 1760 × 180 from y 0 (eye (0, 2.8, −6.5), focal 400).

Signature moments:
- **Brake dive.** In the frame the brake passes 5%:
  - the ego's tail bar lights warning red (from a smoked lens), with a glow spilling round the body;
  - the body pitches 4°/g of measured deceleration (clamped 4° down, 1.5° squat, `snap` spring) and rolls 2.5°/g;
  - a 3 px caution line drops across the three lanes at `v² / (2 · max(−a_long, 2 m/s²))`, with its distance in Titillium cap 16 left of the line, hidden beyond 120 m;
  - the carpet beyond the line falls to 35%.
- **Upshift wake.** One 250 ms band (accent mixed 55% toward white, 70% alpha with a crisp 0.9 edge) climbs the carpet evenly in screen space, with alpha only and no bloom. Meanwhile the pill rolls.
- **Limiter.** Only the pods strobe; the underglow holds a steady violet and the gear pulses after 0.5 s.
- **Drift.** The ego yaws into the bend by the slip angle; the carpet and underglow turn amber and SLIP lights.

## Telemetry (verified fields only)
- rpm, idle, limiter and the tokens' shift point → pods, ribbon and the ribbon's colour stages.
- Gear → pill and ghosts.
- Speed → readout, dash scroll, traffic speed, the arcs' speed gate and the stop line.
- Brake → lamps, dive trigger and stop line.
- **Longitudinal acceleration** (verified `Acceleration` z) → dive and stop distance. This replaces the brief's pitch, which is not verified.
- **Lateral acceleration** (verified `Acceleration` x; its sign matches `Steer` in packets 08 and 19) → path curvature `κ = a_lat / max(v, 3)²`, plus roll and the light.
- Body slip (from the verified velocity) → ego yaw, drift and SLIP.
- Car ordinal → factory palette → ego tint.
- **Not used**, because they are not in the packet or not verified: tyre slip ratio (the decoder has none, only slip angles), suspension, pitch / roll, wheel rotation and tyre temperatures. See My calls.

## Performance (RTX 3070 Ti Laptop, 3840 × 2160, `preview-export`, guard included)
| | GPU |
|---|---|
| Cockpit, 60-frame sequence | **p50 0.333 ms, p99 0.349 ms** (0.275 / 0.294 before the guard) |
| Cockpit states | 0.24 (idle, cruise), 0.35 (drift), 0.38 (redline) |
| Hood states | 0.29 (idle, cruise), 0.40 (drift), **0.60 (redline)**: 0.46 without the guard |

Five other cluster chats were rendering on the same GPU, so single states are noisy: one run read hood redline at 0.92 and hood drift at 1.12. The 60-frame p50 / p99 was steady across 5 runs.

**Live, 60 s, `--demo --cluster TeslaAdas --stats`:** 2861 frames, 0 validation errors. CPU per frame was p50 1.69 ms, p99 9.1 ms, with upload + record + submit at p50 1.23 ms. This is the known host cost the pilot recorded (p50 1.44); the p99 includes the parallel builds on this machine.

## Critique (`ui-ux-designer`, 3 rounds)
- **Round 1: 6/10.** Findings:
  - the gear roll spilled out of the pill;
  - the wake was invisible;
  - there was too much blue (lead car, carpet and underglow all brand blue);
  - red arcs showed at standstill;
  - the ribbon was thin and lost its red;
  - MPH floated away from the digits;
  - the scene was crammed under the pill;
  - the stop label collided with the brackets.

  All of these were fixed (see What it is).
- **Round 2: 7/10.** Still open: the gear clip, red arcs while cruising, and red starting too late on the ribbon. All three were fixed: the entry scale is cancelled, red is reserved, and the ribbon follows the pod stages.
- **Round 3: 8/10, pass.** Its last note (the top arc step too loud) was applied: no glow, caution at 90%.
- **Open, shared (not this cluster):** in `common::shift::draw_pod`, a lit pod's glow is drawn over the neighbouring unlit pods' 1 px outlines, which then disappear (04, frame 44). The reviewer asked that this be fixed before wave 1 closes.

## My calls
- **A private `scene.rs` instead of `common::adas_road::AdasRoad`.** The brief needs things `AdasRoad` doesn't do: glTF meshes, number-free brackets, the measured stop line, dive and roll, and the wake. `common/` is frozen. `scene.rs` reuses `adas_road`'s `curvature`, `ego_yaw`, `EGO_Z`, `LANE_WIDTH`, `MAX_DT` and `EYE`, and copies and trims its traffic, carpet, lane and arc code. It is the candidate to fold back into `common::adas_road` (see Requests).
- **Curvature from lateral acceleration, not `angular_velocity_y`.** `a_lat` is verified and is the true path curvature; the yaw rate's sign is not verified, and in a drift yaw rate ≠ path curvature × v. The review harness synthesises both consistently, so the mockups are unchanged in meaning.
- **Dive from measured deceleration, not `pitch`.** Pitch isn't verified, and deceleration has no 2 s baseline to drift.
- **Chips are SLIP and LIM only.** The brief's LOCK needs tyre slip ratios, which the decoder doesn't have. SLIP uses the latched body-slip drift (on at 10° and 5 m/s, off below 7° or 4 m/s).
- **Not built:** front-wheel steer angle, the tyre sidewall stripe, per-corner ride height and the thermal steps. The glb loader flattens every node into one mesh with one colour, so wheels and material slots are not separable. The other inputs aren't verified.
- **The lead car is light clay with a 20% brand tint, not in-path blue.** This departs from the family spec on the critique's advice: one saturated hue (the path) gives the scene a hierarchy, and the brackets mark the lead.
- **The blind-spot steps never turn red** (primary 40%, then caution 70%, then caution 90%). Red is kept for the brake bar and the redline. The arcs show only when moving (fading in from 3 to 6 m/s).
- **The carpet is 0.40 at the ego** (spec 0.55) and the **underglow 0.40 with sigma 22%** (spec 0.55 and 30%), from the critique.
- **No ghost 8s.** At `ink_ghost` the leading 8 read as a digit ("874"); the family rule is also no ghosts.
- **The ribbon is 6 px in the cockpit** (brief 4 px), because the critique judged 4 px about 1.3 mm on the 55". The hood keeps 3 px.
- **The cockpit scene band is 1440 × 260 from y −20** (brief 1440 × 280), with focal 440 and the horizon at 8.5%, so the far road and the lead car clear the gear pill. The hood camera is lower (eye (0, 2.8, −6.5), focal 400) so the ego and its underglow fit the 180 px band. The brief's "25° pitch" left no visible road in 180 px.
- **The wake runs in screen space, not in depth.** In depth it spent its first frames under the ego and the rest as a sliver at the horizon.
- **The gear pill holds the glyph without the shared 1.12 entry scale**, cancelled with a local copy of `GearGlyph`'s `fast` spring, so the 24 px roll stays inside the pill.
- **The traffic seed is fixed** (`scene::SEED`). It has 4 road users (the lead car plus 3 adjacent), never parks a solid car beside the ego, and shows no numbers.
- **Assets:** none of my own. It uses the shared Phase 6 `assets/adas/{ego_coupe,sport_sedan,suv,supercar}.glb` and falls back to `MeshData::car_proxy` when `setup.assets` is `None` or a glb fails to load. No recipe module was needed.

## Requests for the conductor (shared code)
1. **Kit, urgent:** re-bind vertex slot 0 after a mesh draw (Read first). Then delete the guard block at the end of `TeslaAdas::draw`.
2. **Kit:** the glb loader flattens all nodes and materials into one mesh and one colour. Expose per-material sub-meshes, or at least a per-node split, for the tail and head lamps, glass, tyres and separable wheels. That enables the brief's steer angle and lamp materials.
3. **Kit:** the mesh pass has no MSAA (the brief asked for 4× in Z4). The car silhouettes alias at 1 px.
4. **`common::shift::draw_pod`:** a lit pod's glow erases the neighbouring unlit outlines (critique, all clusters).
5. **`common::adas_road` review** (I'm its first real user). Defects found:
   - It draws a distance number on the synthetic lead car (`draw_lead_marker`). The catalog brief calls that fake data; the family spec still asks for it. One of the two should change, and the component needs a number-free option.
   - Curvature comes from `Signals::yaw_rate`, whose sign is not verified. `a_lat / v²` is verified and is the path curvature (see `scene::path_curvature`).
   - It has no measured-deceleration stop line, only a fixed 9 m/s² step.
   - Its proximity arcs show at standstill and go full red alongside.
   - Its 15-point arc polylines exceed the pilot's 8-segment rule.
   - It draws procedural hulls rather than the glTF meshes.
   - Folding `tesla_adas/scene.rs`'s pieces in (`path_curvature`, `stop_distance`, the mesh cars, the wake, the speed-gated arcs) would give the family one scene.
6. **Family spec `adas-hud.md`:** update the in-path colour rule (lead car tint versus brackets), the blind-spot colours (no red), and drop the lead distance label, to match this pattern-setter.

## What the next ADAS & HUD chats must know
- Draw all 2D first and the meshes last until request 1 lands. Then you can drop the guard and order freely.
- `scene::SceneGeom` (band, eye, pitch, focal, horizon) is the per-view camera table. Keep the ego, its underglow and the lead car inside the band, and check the hood separately: 180 px is very short.
- Linear alphas: the carpet at 0.40 still reads saturated. Keep traffic opaque but darkened, because translucent meshes show their back faces.
- Renders: `docs/handover-shots/cluster-tesla_adas-{03_redline_strobe,04_drift_braking,hood_02_mid_cruise,05_brake_dive,upshift_frame43,cockpit}.png`. The rest is in `mockups/TeslaAdas/` (gitignored; regenerate with the command above).
