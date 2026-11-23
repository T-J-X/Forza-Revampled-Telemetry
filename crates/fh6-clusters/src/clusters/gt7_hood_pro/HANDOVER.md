# HANDOVER: Cluster · GT7 Hood Pro (catalog row 3, ★ Modern Digital 2027 pattern-setter, hood default)

**Chat / branch:** cluster chat Gt7HoodPro, `claude/cluster-gt7hoodpro-7f9e27`  **Date:** 2026-11-23  **Result:** ✅ built, verified and passed critique (9/10, no blockers)

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board has 4 states per view | ✅ Penpot + HTML board | Penpot page `07 · Gt7HoodPro` (page `6fe75c5e-bb22-8084-8008-fd908bb325c8`): 4 hood state boards at 3840×2160 with the binnacle guides (hood and cockpit envelopes, both anchors) plus a motion and material notes board. HTML board: `mockups/Gt7HoodPro/index.html` |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs` (zone table, `Geom` per view, `Faces`, `Wings`); `cargo test -p fh6-clusters gt7_hood_pro`: 3 unit tests and the golden test (4 hood images) pass |
| 4K renders + 60-frame sequence in `mockups/Gt7HoodPro/` | ✅ | `cargo run --release -p preview-export -- --cluster Gt7HoodPro` |
| Live demo verified, GPU under 0.5 ms recorded | ✅ | `FH6_WheelDash --demo --cluster Gt7HoodPro --stats --bench-seconds 60`: 3789 frames at 3840×2160; the GPU numbers are below |
| Critique passed, with notes | ✅ clean pass | 4 rounds by an independent `ui-ux-designer` reviewer: 6.5 → 8 → 8.5 → **9 (pass)**; see "Critique" |
| Bespoke Blender + ComfyUI assets, used in the frame | ✅ | `assets/clusters/gt7_hood_pro/pod_lens.png` (Blender) and `ribbon_grain.png` (ComfyUI), recipe `tools/asset-forge/src/recipes/gt7_hood_pro.rs`; before / after below |
| Handover written | ✅ | this file |

## What it is
A frameless "gull-wing horizon" hung from one 1 px hairline. Each wing is a circular arc, level at the tip and rising steeply into a kinked crest. Two mirrored light ribbons (7 → 24 px, power 1.6, primary → white core) climb from the wingtips (±820, −60) to the crest (0, −146) over the gear. Idle sits at the tips and the car's shift rpm at the crest.

The ranks:
1. **Shift strip and gear.**
   - Eleven converging diamonds, 28 px (the centre one 34), sit above the crest.
   - The gear (Saira Condensed 800, cap 140) sits in an OLED-black squircle pod, 232 × 184. The pod's face is the Blender glass lens, with a light-driven specular crescent.
2. **Speed.** Saira Condensed 600, cap 80, centred for 1–3 digits, with a black keyline on the hood.
3. **The twin ribbons,** each led by a comet orb (the `common::ribbon` meniscus bead, sized by the orb intensity law). The ComfyUI light grain flows through them toward the crest, faster with the revs.
   - The left wing carries the rpm scale: Titillium Web 600 numerals every 1000 rpm, ticks every 500.
   - The right wing runs at 70% with ticks only.
4. **Car and forces.**
   - Four slip capsules on the pod's corners make the pod read as the car seen from above.
   - The G puck (`common::GBall`) sits on the right, the THR / BRK capsules on the left.

Signature moments:
- **Apex fuse.** The comets race inward. Within 2r + 10 px a neck grows between them. At the shift rpm:
  - the white-hot inner fill retracts;
  - the inner 45% of both wings turns `shift` (85%, in its own bloom);
  - the cores shrink;
  - an 8 px blade rises from the crest to the centre diamond in 60 ms.

  All of it is steady.
- **Limiter.** Only the diamonds strobe, on the lock's own clock: 100 ms full on arrival, then 10 Hz between 100% and 25%, so the strip never vanishes. Reduced-flash mode gets the shared 2 Hz breath.
- **Upshift.** The lock falls (the blade splits) and the comets drop outward on the ribbon spring with feed-forward. The gear does the tokens' 1.12 → 1.0 entry, clipped 16 px inside the pod, while a 60° light streak runs over the pod rim (reversed on a downshift).
- **Light.** Lateral G swings the virtual light ±25° on the pod crescent and rims, and moves the mid and front planes 2 and 4 px (parallax).

Hood legibility (no scrim in this view; checked over a bright sky and a road, see Visuals):
- a blurred dark channel under the lit span of each wing (0.38), and a faint trace under the unlit span (0.10);
- 8-way black keylines on every text run (speed, numerals, MPH, THR / BRK);
- soft rounded-rect pads (0.30, 32 px feather, clipped to the envelope) under the speed, G and pedal groups;
- dark lenses behind lit diamonds (0.5), unlit diamonds (0.2) and the slip capsules (0.5).

## Verification output
```
cargo check --workspace                                   Finished (web-preview manifest warning is pre-existing)
cargo clippy --workspace --all-targets -- -D warnings     Finished, no warnings from this branch
rustfmt --edition 2021 --check <own .rs files>            ok
cargo test -p fh6-clusters gt7_hood_pro                   3 passed; golden_images gt7_hood_pro ... ok
cargo test -p asset-forge                                 12 passed
cargo test --workspace                                    every suite ok, 0 failed (golden_images: 49 passed)
```

## Performance (RTX 3070 Ti Laptop, 3840 × 2160)
| What | Number |
|---|---|
| `preview-export` 60-frame sequence, hood (final) | GPU p50 0.269–0.272 ms, p99 0.289–0.296 ms (two runs) |
| Hood states | 0.25–0.31 ms. Single-frame spikes of 1.3–2.6 ms landed on a different state in each run while the shared GPU was 36% busy with the other cluster chats, so they are contention, not the cluster |
| Before the hood-legibility layers | p50 0.17 ms. The two blurred channel layers and the pad shadows cost about 0.13 ms |
| Live demo, 60 s, `--stats` | 3789 frames presented (63 fps on a synthetic stream). The cluster's own build step: p50 0.007 ms, p99 0.049 ms. Host CPU per frame: p50 0.96 ms, p99 4.25 ms (the shared upload / record / submit cost the pilot also reports, not this cluster) |

The first draft's hairline was one arc of a ~4000 px circle, and the kit evaluated it over a full-screen quad: 0.88 ms. It is now 8-segment polyline chunks (0.17 ms).

## Bespoke assets (asset-forge set `gt7_hood_pro`)
Run `cargo run --release -p asset-forge -- build --set gt7_hood_pro`. Rows (recipe + seed) are in `assets/clusters/gt7_hood_pro/ASSETS.md`. Both are optional at runtime, and the procedural look is the fallback.

| Asset | Tool | What it adds |
|---|---|---|
| `pod_lens.png` (928 × 736, 4× the pod) | Blender 5.3, recipe `POD_LENS`, seed 7201 | The pod as a sculpted black-glass lens: a squircle slab with a 24-segment rolled bevel in a satin clear-coated black dielectric, a key strip from the upper left (the resting virtual light) and a cool rim strip from the lower right. Cycles 256 spp, top-down orthographic, face floor crushed to OLED black. It turns the flat black pod into hardware: the bevel catches the light. Before / after: `mockups/Gt7HoodPro/assets/before_after_pod_lens_2x.png` |
| `ribbon_grain.png` (1024², seamless, luminance) | ComfyUI SDXL base 1.0 + 4x-UltraSharp, seed 7301 | Fine horizontal light filaments, scrolled through both ribbons (body dimmed 38%, filaments added back), tinted by the palette and `shift` on the locked arch. It flows toward the crest at 80 + 900 × fill px/s, so the wings read as tubes of moving light rather than flat strokes. Before / after: `mockups/Gt7HoodPro/assets/before_after_ribbon_grain_2x.png` |

Full-frame before / after (top procedural, bottom with assets): `mockups/Gt7HoodPro/assets/before_after_02_mid_cruise.png`, `before_after_03_redline_strobe.png`.

Recipe review notes:
- The first lens render mirrored an overhead fill light as a grey square on the face, so the fill was removed.
- The satin face lifted to slate; a black floor of 0.1 in the recipe's Rust post-processing fixed that.
- The first grain pass was invisible: additive light on a near-white ribbon saturates. The grain is now drawn last, as dim + add.

## Visuals
- `docs/handover-shots/cluster-gt7_hood_pro-02_mid_cruise.png`, `-03_redline_strobe.png`, `-04_drift_braking.png` (1:1 crops over OLED black)
- `docs/handover-shots/cluster-gt7_hood_pro-03_redline_sky.png`, `-02_mid_cruise_road.png` (hood view over the bright daylight backdrops)
- `docs/handover-shots/cluster-gt7_hood_pro-assets_pod_lens.png`, `-assets_ribbon_grain.png` (before / after)
- Everything else is in `mockups/Gt7HoodPro/` (gitignored): the review states, `sequence/`, `assets/` and `legibility/`.
- Penpot: page `07 · Gt7HoodPro` (ID above). Its boards are `Hood · 01 idle` `6fe75c5e-bb22-8084-8008-fd908bea20e6`, `Hood · 02 mid cruise` `…fd908c5751ab`, `Hood · 03 redline strobe (apex fuse + lock)` `…fd908caab823`, `Hood · 04 drift braking` `…fd908cf8020b`, and `Motion + material notes` `…fd90a1a0b8f2`. Penpot falls back from Saira and Titillium to a serif, and opacity overlaps show as dots on the SVG-imported ribbons. Both are board-only artefacts; the renders are the pixels of record.

## Critique
Four rounds by one independent `ui-ux-designer` reviewer, judging the PNGs (it measured what is on screen, not intentions).
- **Round 1: 6.5, not a pass.**
  - The diamond glow read as navy plates.
  - The strip was dark at the fuse.
  - The arch never changed colour.
  - Pink lobes at the crest: the ribbon spring overshot into the ribbon's warning stop.
  - The gear entry overflowed the pod.
  - Hood legibility was unproven.
  - All fixed.
- **Round 2: 8.**
  - The arch tint was buried under the white bloom.
  - The strobe started on its off phase.
  - The pod's radial pad formed a dome over the apex.
  - Speed off-centre without ghosts.
  - All fixed (lock retracts the white fill, the strobe runs on the lock's clock, the pod shadow pad, centred digits).
- **Round 3: 8.5.** It cleared the bar on black and road. Over sky:
  - the unlit channel out-contrasted the lit tach;
  - the lit cues washed out;
  - the unlit lenses were too heavy;
  - the radial pads read as grime.

  All fixed: lit-span channel, lit-diamond lenses, 0.2 unlit lenses, rounded-rect text pads.
- **Round 4: 9/10, pass, no blockers.**
  - "Beats GT7" (the apex lock is a better shift cue than GT7's bar).
  - "Matches McLaren progressive LEDs".
  - "Now matches Porsche shift-at-12", including over bright sky.
  - "Beats Lucid on true black."
  - Polish applied afterwards: text pads 0.30 → 0.20 (every text run also has its keyline).
  - Left for later: the shared `GBall` crosshair still has the SimHub silhouette (a family-level redesign of a shared component).
  - The reviewer's caveat: the legibility proof uses a synthetic daylight scene, so confirm it with one in-game hood capture over a sunlit road.

The pod lens moved the score less than the hierarchy fixes, by the reviewer's own account. It adds the rolled, lit bevel that makes the pod read as hardware.

## Telemetry
All inputs are native and verified: rpm (current, idle, limiter), the shift rpm (limiter × the shift point, or the OSD's `ShiftPointPct`), gear, speed, throttle, brake, smoothed acceleration (G puck, light, parallax) and body slip (from velocity).

**Not available:** tyre temperatures, slip ratios, lap number, current and best lap, and the per-gear thresholds. `fh6-core::telemetry` doesn't decode them and `tests/golden/packets` doesn't carry them. So:
- the slip capsules use body slip β: rears β, fronts β / 2, outline amber > 8° and red > 18°, tint by exception;
- there are no temperature numerals;
- the lap line is omitted.

When the decoder gains verified tyre and lap fields, feed them to the existing `TyreSlipQuad::update(slip, Some(temps), dt)` call and add the shy lap line above the pedal capsules.

## My calls
- **Gull-wing geometry: tips at y −60, not −20.** At −20 the wings passed 5–9 px from the pod corners, and the comet orb would touch the pod. The crest is at −146, not −150, which keeps the centre diamond inside the 400 px hood envelope. The pod is 232 × 184 with gear cap 140 and speed cap 80, not 250 × 210 / 150 / 84: the brief's stack is taller than the envelope. A unit test checks every clearance.
- **Wings are circular arcs** (`PathRibbon::arch`): level tangent at the tip, which gives the gull-wing kink at the crest without a new shared path type.
- **No ghost 8s.** The kit draws very-low-alpha text far brighter than its linear alpha: even 1/255 rendered as a clearly readable grey "8". The field is as wide as the digits, so the speed stays centred.
- **Slip from body slip** (see Telemetry).
- **The diamond optics are drawn here, not with `common::shift::draw_pod`.** The shared pod glow read as navy plates. The ShiftLights logic (fill, release, reduced flash) is still shared; only the limiter strobe phase runs on the lock's clock.
- **The lock owns the moment:** violet inner arch, blade, full strip. The comets shrink and the white fill retracts while locked.
- **Hood legibility layers cost about 0.13 ms of GPU.** They stay because the brief's risk ("thin ribbons and the speed can fail the blur gate") was real in the daylight composites.
- **A cockpit geometry exists** (tips ±680, crest −206, the cockpit envelope) for `--cluster` forced views. The catalog lists hood only, so its goldens are hood only.
- **The shared asset ledger isn't committed.** `asset-forge build` rewrites `assets/ASSETS.md`, `assets.json`, `contact-sheet.html` and `forge.lock.json`, which every cluster chat touches. I committed only my files and my own `assets/clusters/gt7_hood_pro/ASSETS.md`. The conductor can regenerate the shared ledger with `asset-forge build --set gt7_hood_pro` or `asset-forge sheet` after merging.

## Known issues / debt
- **The pads cut faint marks into the ribbon bloom on pure black.** Dark shapes drawn before a bloom layer still dim its halo. This is visible only as faint shadows in the OLED-black renders, at the pads and as the channel's "tube casing". Over a real scene they don't show. It's a kit question: bloom composite order.
- **Penpot boards are approximations:** SVG imports with fallback fonts.

## Shared changes requested (for the conductor)
1. **`tools/asset-forge` registration.** Three one-line edits are in this branch so `--set gt7_hood_pro` exists:
   - `recipes/mod.rs`: `pub mod gt7_hood_pro;`
   - `sets/mod.rs`: the match arm
   - `main.rs`: the `SETS` list and the usage string

   Every cluster with assets will touch the same lines, so a generic per-cluster recipe registry would avoid the conflicts.
2. **`preview-export --cluster` should write transparent hood renders, or take `--backdrop`, for hood clusters.** I checked legibility with a scratch renderer (outside the repo) that composites over a synthetic daylight scene; the composites are in `mockups/Gt7HoodPro/legibility/`. This should be a shared tool.
3. **Kit: low-alpha text.** A glyph at linear alpha 1/255 renders as a readable grey. Ghost 8s across the catalog depend on this.
4. **`common::shift::draw_pod`:** its glow reads as a dark plate on OLED. The steeper additive halo here (35% → 12% → 3% → 0.6% → 0 over 1.15 × size) could replace it.
5. **`common::tyre_slip`:** expose the crossfaded fill colour and `is_left` / `is_front` (both private), so clusters that place capsules themselves can reuse them.
6. **Retire `assets/gt7_hood_pro_plate.png`** (the v1 opaque plate, per the brief) once the legacy v1 cluster is gone. It isn't in this chat's files.

## What the next Modern Digital chats must know
- Copy the module shape: a `Geom` per view with the zone table in the module doc, `Wings` (`PathRibbon::arch` pairs plus sampled hairline points), and the layout and clearance unit tests.
- **Shared helpers proven here:**
  - `PathRibbon` (arch, fill spring with feed-forward, meniscus bead)
  - `ShiftLights` (logic) and `LimiterCue`
  - `HeroGear`
  - `GBall` (`draw_meter` + `draw_readout` at your own position)
  - `TyreSlipQuad` (state only)
  - `VirtualLight`
  - `squircle` / `squircle_point`
- **Lessons:**
  - clamp a ribbon's drawn fraction below 1.0, or the spring overshoot paints the warning stop;
  - a huge-radius `Shape::arc` costs a full-screen quad;
  - grain on bright light needs dim + add;
  - check the hood over a bright backdrop: OLED-black renders hide every legibility failure.
