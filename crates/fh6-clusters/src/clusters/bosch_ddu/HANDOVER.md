# HANDOVER: Dash · BoschDdu (catalog row 12)

**Thread / branch:** "Rebuild BoschDdu dash", `claude/project-thread-ovnqwk`, draft PR #8  **Date:** 2026-11-28  **Result:** merged into `main` by this thread (AGENTS.md rule 16); the `MERGED` post in the thread gives the `main` SHA. Closed by self-review: TJX asked to be hands-off (AGENTS.md rule 12); no TJX answer about the design arrived in the thread, so nothing of TJX's is quoted about it. TJX's standing words that shaped the work are quoted where they apply (the cloud lane).

Findings and yes/no answers only; there are no scores in this file. The earlier BoschDdu handover (2026-11-23, the v2 dash this rebuild replaces) is in git history; nothing in it carries over.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| The introduction was posted, and the thread is not archived | yes | the thread's first message (`DASH BoschDdu`); the thread is open |
| `RESEARCH.md` is written, with sources | yes | [RESEARCH.md](RESEARCH.md): 16 features with sources, the match table, deliberate changes A to L, the gaps |
| The design board is the Penpot page `07 · BoschDdu`, 4 states, its IDs in the README and here | yes | page `b568ed3f-64cd-806f-8008-fd94834f5850`; boards under "Penpot" below; [README.md](README.md) |
| A bespoke ComfyUI asset and a bespoke Blender asset are used, with ledger rows and 4K masters on Drive; the before and after pair is here | yes | [assets/clusters/bosch_ddu/ASSETS.md](../../../../../assets/clusters/bosch_ddu/ASSETS.md); "Assets and masters"; "Before and after pair" |
| The module is complete, with no TODOs, and goldens pass | yes | "Verification output" |
| 4K renders, the 60-frame sequence and the product page are in `mockups/BoschDdu/` | yes | written by the final `preview-export` run (gitignored) |
| The live bench report and the GPU time under 0.5 ms are recorded | yes | "Verification output", "Live bench" |
| The direction check was posted and self-reviewed, with its findings here | yes | "Direction check" |
| Every line of the closing review is yes, or a line that could not be fixed afterwards is reported | yes (line 30 reported) | "Closing review" |
| The visual review (V1 to V26) is here, each line with its evidence | yes | "Visual review" |
| The handover is written, the job is reported for the merge, and the product page and checklist are posted | yes | this file; the thread's closing message |

## Penpot
Page `07 · BoschDdu`, `b568ed3f-64cd-806f-8008-fd94834f5850`, in the master file "FH6 WheelDash v2 — Design System". Eight 3840×2160 boards at the top level (read on 2026-11-28 at 07:05 UTC):

| State | This rebuild's board (drawn from the dash's `Geom`, text outlined) | The v2 dash's board (2026-11-23, kept for history) |
|---|---|---|
| 01_idle | `BoschDdu_01_idle` `347e5ae2-9e0b-80b1-8009-036560d2332f` | `BoschDdu · 01_idle` `b568ed3f-64cd-806f-8008-fd951f47f89c` |
| 02_mid_cruise | `BoschDdu_02_mid_cruise` `347e5ae2-9e0b-80b1-8009-036560dcf7e2` | `BoschDdu · 02_mid_cruise` `b568ed3f-64cd-806f-8008-fd95256171ef` |
| 03_redline_strobe | `BoschDdu_03_redline_strobe` `347e5ae2-9e0b-80b1-8009-036560e5a969` | `BoschDdu · 03_redline_strobe` `b568ed3f-64cd-806f-8008-fd9531913497` |
| 04_drift_braking | `BoschDdu_04_drift_braking` `347e5ae2-9e0b-80b1-8009-036560ed8d1a` | `BoschDdu · 04_drift_braking` `b568ed3f-64cd-806f-8008-fd9541fc429d` |

The catalog lists cockpit only, so there are no hood boards. Board v2 (TJX, 2026-11-28: Penpot first, then code), drawn in Penpot before the matching code change (`d320f91`): the horizon band over the panel columns only, as the code draws it, and the boost row with "psi" after the value. Each underscore board's content was replaced in place (the board ids stay); its new top child: 01 `347e5ae2-9e0b-80b1-8009-038de5f8b368`, 02 `347e5ae2-9e0b-80b1-8009-038e090b456b`, 03 `347e5ae2-9e0b-80b1-8009-038e0b748e35`, 04 `347e5ae2-9e0b-80b1-8009-038e0d90ae35`. Exports saved as `mockups/BoschDdu/board/<state>.png` on the PC and in Drive (`bosch_ddu_penpot_board/board_v2_<state>.png`).

## What was built
- `crates/fh6-clusters/src/clusters/bosch_ddu/`
  - `mod.rs`: the zone table (`Geom`, cockpit, and the hood table at `HOOD_SCALE` 0.68 for the parts that clear the size floors), the `BoschDduPanels` option, the brow LEDs (`TopLeds`: 2-6-2 in the palette's stage colours, each lens at its share of the shift rpm, the shift lamp, the middle-six strobe with the outer pairs steady, reduced-flash breathing), the side columns (brake left, body slip right, latched steps), the rev gauge (hairline scale ring, the band inside it with a needle at its head, the cross-fade to ink over the shift point, the red zone and limiter tick), the rpm digits (a 0.35 s window maximum through a limiter bounce, which the band follows too), the gear in its beam, the four quarter panels (PEDALS, G-FORCE, ENGINE, SPEED), the chin's name and logo zones, and 21 unit tests.
  - `hardware.rs`: the Blender housing (three key azimuths, cross-faded by the virtual light that follows G) and its procedural fallback; 1 unit test.
  - `RESEARCH.md`, `README.md`, `spec.md` (unchanged brief), this file.
- `tools/asset-forge/src/recipes/bosch_ddu.rs`: the ComfyUI bead-blast macro (SDXL, seamless, made neutral), the Blender housing scene (bead-blasted anodised body, bonded black glass face, 10 + 5 + 5 smoked lenses, light sensor; the housing's own rig with no softbox over the face and no HDRI), the runtime box filter with a hard fail over the static limits, and the cluster's `ASSETS.md`; 13 tests. The list registrations were already on `main` from the code re-cut.
- `assets/clusters/bosch_ddu/`: `housing_0.png`, `housing_1.png`, `housing_2.png` and `ASSETS.md` (housing build 3). The v2 dash's `carbon_weave.png`, `housing_l110.png` and `housing_l160.png` are removed.
- `tests/golden/images/BoschDdu_*.png`: the four cockpit goldens.
- `docs/TELEMETRY_USAGE_MATRIX.md`: the dash's row, and the v2 dash's four `DASH_SCENARIOS` rows removed; `crates/fh6-clusters/tests/dash_gates.rs`: the same four scenarios removed (the alarm lamps that needed them are gone).
- `docs/handover-shots/cluster-bosch_ddu-*.png`: the before and after pair and the review crops (the v2 dash's shots replaced).

## Telemetry
Every field the dash reads is covered by the golden packets or derived from them (matrix row: `race_on`, `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity`, `power`, `boost`):
- **Read as they are (through `fh6_kit::Signals`):** rpm (current, limiter), gear, speed, throttle, brake, the race flag.
- **Read from the packet:** power (W, shown in the user's power unit) and boost (psi, clamped to −15..60).
- **Derived:** lateral and longitudinal G from acceleration (the kit's `GBall`); body slip from velocity (`Signals::slip_deg`, the right column, one lens per 2.5° up to five); the shift point from the limiter and `ShiftPointPct` (`Signals::shift_rpm`). The limiter is trusted only when `limiter_known`: otherwise the strip stays dark, the red zone, limiter tick and SHIFT figure are not drawn and the scale falls back to its default top.
- **Not read:** clutch, handbrake, drivetrain, tyre temperatures and slip, fuel, laps and every other offset the golden packets do not cover. The v2 dash's derived TC, ABS, LNCH and HB lamps are gone with its alarm row (RESEARCH change F).
- **Held beyond the kit springs (line 7):** the rpm digits and the band show the highest rpm of the last 0.35 s while the rpm sits near the limiter (`PEAK_HOLD_S`), so a limiter bounce (about 50 rpm a frame) neither flickers the digits nor jumps the needle; a gear change or a drop out of the band releases it at once. Speed by the kit's 0.6 rule; power, boost and G by display hysteresis (`G_HYST_CENTI` 3 hundredths). The brow keeps the tokens' 120 rpm shift hysteresis. Gear and speed reach the screen on the next frame.

## Verification output
At the final head. The cloud container runs lavapipe (Linux), the PC runs the Windows toolchain of `AGENTS.md`.

```
cloud container (lavapipe), code of a0cb477 plus the local-only Vulkan backend commit (never pushed):
cargo test -p fh6-clusters --test dash_gates        test result: ok. 10 passed; 0 failed
cargo test -p fh6-clusters --lib bosch_ddu          test result: ok. 22 passed; 0 failed
cargo test -p fh6-clusters --test golden_images     bosch_ddu ok, twice in a row, no golden rewritten
                                                    (the four BoschDdu_*.png byte-identical: 86b8edab… 9d94dc1d… e5a9e20b… 7554e670…)
rustfmt --check mod.rs hardware.rs recipe           clean (only Linux's CRLF newline-style note, which the PC does not print)

PC (RTX 3070 Ti Laptop), a0cb477, preview-export under gpu-lock.exe:
  01_idle            0.219 ms GPU   14 draws   209 instances
  02_mid_cruise      0.278 ms GPU   15 draws   218 instances
  03_redline_strobe  0.282 ms GPU   15 draws   249 instances
  04_drift_braking   0.282 ms GPU   15 draws   257 instances
  sequence: 60 frames, GPU p50 0.280 ms, p99 0.282 ms
```

The golden run in the container also reports 69 other dashes' goldens as mismatched on lavapipe (their goldens were made on other adapters); none is this thread's, and `main` has the same goldens.

The workspace gate (`cargo check --workspace --all-targets`, `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`) cannot build in the Linux container (`fh6-app` and `fh6-input` need the Windows crates), so it runs on the PC on the merge head, the commit that adds this file; its output is in the thread's `READY FOR MERGE BoschDdu` post.

## Live bench
Run on the PC at `a0cb477` under one `gpu-lock.exe` take (announced in the thread first): `mock-udp --scenario bmw_m4_cs --port 8000 --rate-hz 60` streaming, then `FH6_WheelDash --demo --cluster <Name> --stats --bench-seconds 60 --bench-out mockups/BoschDdu/live_bench.md` (and `live_bench_scaffold.md`), both exit 0 with no error. The scaffold is VisionPetals (catalog row 51, still ☐).

| | BoschDdu | VisionPetals (scaffold) |
|---|---|---|
| Frames presented | 5239 (87.3 fps) | 8232 (137.2 fps) |
| New packets shown | 3504 (58.4/s) | 3500 (58.3/s) |
| CPU per frame p50 / p99 | 0.729 / 1.068 ms | 0.714 / 1.081 ms |
| Receive→present p50 / p99 | 0.764 / 1.495 ms | 0.736 / 1.500 ms |
| Receive→next vblank p50 / p99 | 2.447 / 4.134 ms | 2.441 / 4.410 ms |
| Kit work per frame | 3 passes, 15 draws | 3 passes, 11 draws |
| Draw into arena p50 / p99 | 0.039 / 0.073 ms | 0.032 / 0.066 ms |

- The dash's own CPU share: 0.729 − 0.714 = 0.015 ms at p50 (limit 0.3).
- The app's CPU per frame is over 1 ms at p99 in both runs, the scaffold included, so it is the host, not the dash: nearly all of it is upload, record and submit (about 0.42 ms) and the present call (about 0.24 ms). These runs drew on the laptop's own 1920×1080 panel at a 300 Hz refresh, unlike the 4K runs in `docs/perf/engine.md`; BoschDdu also presented fewer frames a second than the scaffold (87 against 137), which the reports do not explain. Reported to the coordinator; no shared code was edited for it.
- 77.1 % (BoschDdu) and 76.8 % (scaffold) of packets were presented before the first vblank.

## Closing review
Answered at the final head. Line 30 (the order of work) is a no that could not be fixed afterwards, reported below, in the thread and to the coordinator. The visual review's V5, V13 and V12 (the base-hue badge) are reported under "Visual review".

| Line | Answer | Evidence |
|---|---|---|
| 1 | yes | the module takes no pad input: no button mask or `buttons` field appears in `bosch_ddu/` |
| 2 | yes | no `todo!()` or stub; every figure comes from telemetry ("Telemetry" above names each field; power and boost are covered by the golden packets; G and slip are derived from covered acceleration and velocity). The SHIFT box stays empty when the car sends no limiter (closing fix, test `limiter_known`) |
| 3 | yes | `expect` appears only in `#[cfg(test)]` code (`mod.rs` from line 1923; `hardware.rs` from 244); no `unwrap()`, `panic!` or `unsafe` in the module or the recipe (rust review, and a search) |
| 4 | yes | the search for `format!`, `vec!`, `collect`, `to_string`, `String::from`, `Box::new`, `tracing::`, `.lock()` and `std::fs` finds `Box::new` in `make` (setup), `format!` in `Sprites::load` (setup, `hardware.rs:84`) and the rest in tests; `warm_frames_do_not_allocate` passes in `dash_gates` (10 passed) |
| 5 | yes | `preview-export` on the PC under `gpu-lock.exe` at `a0cb477` (the code of this head): 01 0.219, 02 0.278, 03 0.282, 04 0.282 ms; sequence p50 0.280, p99 0.282 ms (limit 0.5); cockpit is the only view |
| 6 | yes | live bench at `a0cb477`: CPU per frame p50 0.729 ms with BoschDdu against 0.714 ms with the scaffold VisionPetals (row 51, ☐): the dash's share is 0.015 ms (limit 0.3). The app's p99 is over 1 ms in both runs (1.068 and 1.081 ms), the scaffold too, so it is the host (upload, record and submit about 0.42 ms, the present call about 0.24 ms, on the laptop's 1920×1080 panel at 300 Hz): reported to the coordinator; no shared code was edited for it. Both reports under "Live bench" |
| 7 | yes | rpm, speed and gear reach the screen on the next frame through the kit springs; the rpm digits and band hold a 0.35 s window peak only while the rpm sits near the limiter, and a gear change or a drop out of the band releases it at once ("Telemetry", "Held beyond the kit springs"); this is display hysteresis against a 30 Hz bounce, not a smoothing lag (V9, V21) |
| 8 | yes | the cluster draws only when the host calls `draw`; it has no timer, thread or second surface (the host's hide rules apply unchanged) |
| 9 | yes | no non-Rust source added; the bpy scene lives as Rust strings in `tools/asset-forge/src/recipes/bosch_ddu.rs` |
| 10 | yes | every branch commit is by `TJX <TJX@Ngage.com>` with no trailer (`git log --format='%an %ae%n%b' origin/main..HEAD`); a search of the changed files finds no real name, user path, personal email, legacy build or replaced design tool; the `RESEARCH.md` links are public product and press pages |
| 11 | yes | nothing was copied into the game folder or the mirror |
| 12 | yes | no new dependency |
| 13 | yes | `cargo test -p fh6-clusters bosch_ddu` 22 passed; `dash_gates` 10 passed; `rustfmt --check` of the three files clean; `cargo check` and `cargo clippy -- -D warnings` run on the PC as part of the workspace gate on the merge head ("Verification output"; the container's toolchain is older and cannot build the Windows crates) |
| 14 | yes | the module paints through the kit's premultiplied pipeline only; housing sprites are loaded straight-alpha and premultiplied by the kit; no colour is multiplied by alpha in the module |
| 15 | yes | sizes and strokes go through the kit's 1 px clamps; divisions are guarded (`shift_rpm.max(1.0)`, `(1.0 - start).max(1e-3)`, the luminance divisors; rust review); the G5 hostile-telemetry gate passes with no repairs |
| 16 | yes | `Sprites::load` loads the three housing sprites in `new()`; when one is missing `hardware.rs` draws the procedural housing (the `dash_gates` no-assets run passes) |
| 17 | yes | MSDF text from the kit; speed, rpm and every figure through `draw_fixed` / `p.number` with tabular cells; the figures follow `input.fonts.display` (the user's font), with Saira Semi Condensed 600 as the default (change L) |
| 18 | yes | the screen is true black (`stretch`, `probe`); the only fills are the housing render and the lit elements; no full-canvas fill |
| 19 | yes | the cluster reads no clock and keeps its state in `&mut self`; the goldens passed twice in a row without rewriting (lavapipe, "Verification output") |
| 20 | yes | colours from palette roles and tokens (`accent_print`, `print_ink`, `roles.ink`, `roles.warning`, the stage colours); the only hex in the module is in tests and a doc example |
| 21 | NOT ASSESSED | no glass, reflective or spectral effect: the face is opaque black glass in the Blender render, and the module draws no optical effect |
| 22 | yes | V10: in every state the gear in its beam and the brow lead, then speed, then the arc, then the panels, then housing and labels |
| 23 | yes | V6 over the cockpit composites: gear ≥ 10.4:1, speed ≥ 11.4:1, lit LEDs 7.0 and 9.8:1 at 100 % |
| 24 | yes | the cluster draws round (1920, 1835) inside the cockpit envelope (`stretch`: 0 px outside it plus 40); the catalog lists cockpit only |
| 25 | yes | `RESEARCH.md`: the Bosch Motorsport DDU 10 in the BMW M4 GT3, 16 features with sources, the match table and changes A to L |
| 26 | yes | each part is its own `Geom` zone; the housing is the real DDU 10's, with no backing panel beyond it |
| 27 | yes | V17: the Blender housing (anodised body with the ComfyUI bead-blast in its roughness and colour, the black glass face, smoked lenses) reads as a rendered object at full frame; the grain itself is below what a full frame shows (finding C7) |
| 28 | yes | `sequence/`: the strobe 3 on / 3 off from frame 30 (V22) and the gear change at 42 (V23) |
| 29 | yes | `draw()` takes colours from the palette, the figures from `input.fonts.display`, the chin's logo from the car's `CarLogo` and the name from the user's custom name; no literal colour, font or brand name in `draw()` (the default face is chosen in `new()`) |
| 30 | no (cannot be fixed afterwards) | the order ran `RESEARCH.md` (about 00:10 to 03:10 UTC) → the first build's code and render (03:10 to 03:35, commit `2555e09`) → the ComfyUI bead-blast (about 04:15) → the Penpot boards (05:18, drawn from the code's `Geom`) → the Blender housing (06:45, `9a5e051`). The Penpot tab was queued behind two other dashes, and the thread wrote the code meanwhile instead of waiting; TJX's "Penpot first" rule (07:08, "never ever dont we always make in penpot first then code ?") came later and was followed from then on: board v2 was drawn in Penpot before the matching code change (`d320f91`). The board v2 change touches no asset, so it needed no new ComfyUI or Blender pass. Reported in the thread and to the coordinator |
| 31 | yes | page `b568ed3f-64cd-806f-8008-fd94834f5850`, four boards (cockpit only), ids in `README.md` and under "Penpot" |
| 32 | yes | the bead-blast macro (ComfyUI) and the housing (Blender), each with its row in `assets/clusters/bosch_ddu/ASSETS.md` |
| 33 | yes | the bead-blast master is 4096 px; the housing renders at 2 px per 4K px, box-filtered to 1:1 (880×560 at the 4K layout); runtime files in git, masters in Drive under `bosch_ddu/` ("Assets and masters") |
| 34 | yes | every Blender and ComfyUI job ran on the cloud pod through `asset-forge --lane cloud`, which takes `gpu-cloud.lock` per job and runs `cloud down` after the batch; the PC's `preview-export` and bench ran under `gpu-lock.exe`; cloud spend under "Assets and masters" |
| 35 | yes | `DIRECTION CHECK BoschDdu` posted with the first build's render; two rounds and their findings under "Direction check"; TJX did not answer about the design |
| 36 | yes | `mockups/BoschDdu/index.html`, written by the final `preview-export` run |
| 37 | yes | 60 s with `mock-udp bmw_m4_cs` at 60 Hz, exit 0, no error: 5239 frames, 3504 new packets shown, receive→present p50 0.764 / p99 1.495 ms, CPU per frame p50 0.729 / p99 1.068 ms ("Live bench"); the sequence shows no jitter or flicker (V9, V21, V24) |
| 38 | yes | no scores anywhere in this file |
| 39 | yes | V1 to V26 under "Visual review", each with its evidence; V5 and V13 the cold reviewer's one run, reported; V19 NOT ASSESSED (no refracting glass) |

## Direction check
Two rounds, both on lavapipe renders in the cloud container, both self-reviewed; decision **go with changes** after round 1, **go** after round 2.

### Frozen expectations (written 2026-11-28 about 03:35 UTC, before the first render was opened)
- **Rank order:** 1 the gear in its beam and the brow's shift LEDs; 2 speed (Q4); 3 the rev arc; 4 the pedals, G and engine panels; 5 the housing, labels and chin.
- **Feature inventory, reference → tolerance:**
  - F1 black glass face edge to edge, no visible screen edge → no frame line round the screen
  - F2 arched top, top corners R 120 ± 15, bottom R 44 ± 10, satin aluminium wall visible
  - F3 screen 77 % × 68 % of the face, ± 3 %
  - F4 top 10 LEDs in 2-6-2, horizontal pills, unlit read as grey dashes → count 10, groups visible
  - F5 5 side LEDs per side, all above the face's mid-height
  - F6 round rev gauge over 270°, numerals outside, rpm figure in the disc
  - F7 the gear in a glowing perspective beam at the bottom centre, a trapezoid widening down
  - F8 four quarter panels with caps headers over a rule with an angled end
  - F9 one accent colour on the screen (plus red for brake and limiter, the stage colours on the LEDs)
  - F10 the light sensor on the right border, a small dark dot
- **Key frames:** the limiter at frame 29 or 30, the strobe 30-41 (3 on / 3 off), the gear change at 42.
- **TJX's rejection patterns** a to e (`fh6-visual-review` step 2). Likeliest rejection, guessed before looking: the procedural housing reads as a flat grey slab (c, d) until the Blender render lands.

### Round 1 (03:40 to 05:57 UTC, procedural housing)
- **Cold reviewer** (one `cold-reviewer` run, packet `mockups/blind/4417`, seed 4417). First, second, third: the gear "4", "5200 rpm" with the blue arc, the speed "118" (V4 yes). Maker or display unprompted: "Don't know" (V13 no; stays no). Blind read: gear 8 of 8, speed 8 of 8 within 1, shift lights as each frame shows them; the rpm zone read "high" on frames 37, 36 and 29 where the key says limiter (V5 no; stays no). R1 (Bosch's DDU 10 demonstration page) against the render: same layout in both orders; R2 (a BMW brochure page showing a menu) different in both orders.
- **Its findings and what was done:**
  1. The strobe's dark phase reads as "no shift needed" → fixed: the outer pairs stay lit in the shift lamp while the middle six strobe (flash audit: one area, 3 on / 3 off).
  2. Ghost 8s read as digits ("892") → fixed: no ghost 8s.
  3. The gear's plinth hides the low end of the arc → not fixed: the beam stays 16 px or more clear of the track's ends (unit test); at 900 rpm the lit arc is short because idle is a tenth of the scale.
  4. Small dim labels; SHIFT on its blue box low → part fixed: the box fill removed (4.8:1); the labels stay at the static cap (0.195 relative luminance) against the 4.5:1 floor.
  5. The limiter zone not recognisable → fixed: a 6 px red band from the shift point to the scale's end and a wider limiter tick.
  6. Flat bezel and housing → fixed by the Blender housing (round 2).
  7. The G dot hardly moves → not fixed: the ring is full at 1.5 g, so 0.37 g is a quarter of the radius.
  8. Odd demo numbers (0 hp at 2600 rpm) → not the dash: the review telemetry.
  9. The purple arc close to the blue → not fixed: the palette's shift colour (the user's palette); the LEDs and the red zone carry the state too.
- **Five-viewpoint fan-out** (Rust, house rules, HMI, reference fidelity, accessibility; each finding adversarially verified, findings in the review notes at 9a75acc). The confirmed findings went into `0ee0b26` ("step 5: the review's findings, and a black screen"): the DDU 10's horizon line; the hairline scale ring with the band inside it and a needle at its head; the beam lit at its apex and edges; headers on accent rules that kick up at the gauge; hatched pedal bars; a full G ring; horizontal side lenses (brake left, slip right); 2-6-2 brow colours with each LED at its share of the shift rpm; a shift lamp that is the brow's brightest state; a window maximum for the rpm digits; labels at cap 15 and 4.8:1; every tinted static part under the 0.2 cap for any palette; the hood view limited to what clears the size floors; the fallback housing's sheen off the screen; the screen true black. Not taken as asked, each logged: the token size minimums for gear, speed and the LED row (RESEARCH change B: they do not fit a 7-inch race display's slots); the labels at the static cap where the page prints them near white (change J); the shared `common/` components re-implemented locally (under "Requests for shared code"). One HMI finding (brake shown twice) was refuted by its verifier.
- **`design:design-critique`** (round 1, own context): its findings fed the same commit; decision go with changes.

### Round 2 (06:39 to 06:45 UTC, the render of `346e28d` with Blender housing build 2)
- **Housing:** the Drive masters `beauty_0..2.png`, box-filtered here by the recipe's own `box_filter`: p95 0.019 / 0.016 / 0.023 and screen p99 0.0011 / 0.0016 / 0.0009 (limits 0.2 / 0.004).
- **DC lines:**
  - V1 yes: the expectations above, unchanged.
  - V2 yes: the cold description and rejection patterns at round 1; this round's likeliest rejection (a flat slab, c and d) checked on the cockpit composite: the housing now carries a polished rim with a key highlight and a black glass face.
  - V3 yes: the cold reviewer once (seed 4417), nine findings recorded above. Not rerun.
  - V4 yes: the reviewer named the gear 4, then 5200 rpm and the arc.
  - V6 yes: over `<state>_cockpit.png`: gear 10.4 to 16.4:1, speed 8.1 to 12.7:1, rpm 7.4 to 9.9:1, power 6.7 to 8.5:1, G 4.9 to 5.6:1, lit LEDs 7.0 and 9.6:1. Boost 4.2:1 in `03_redline_strobe` only, carried as not fixed (closing review, A1). Unlit LEDs carry no information.
  - V10 yes: in the squint view the labels and LEDs fade first, the housing rim stays as a faint outline, and the gear 4, the speed 118 and the arc go last.
  - V13 no (stays no): "Don't know".
  - V14 yes: F1 glass surround 0.0006 relative luminance (sRGB about 2) against 0 for the screen, its edge visible only in the stretch view; F2 the arch with R 120 and R 44 (geometry test); F3 624/810 = 77 %, 374/548 = 68 %; F4 10 lenses in 2-6-2 counted on the crop; F5 5 + 5 above mid-height; F6 0 to 9 over 270°, numerals outside, rpm in the disc; F7 the beam ±32 to ±120; F8 four headers; F9 primary only, plus the red zone and brake and the LED stages; F10 the sensor dot on the right border. Changes A to L in `RESEARCH.md`.
  - V15 yes: blurred to the same level (blur 6), the same masses in both orders: the arched LED brow, the centre disc, the beam-lit gear at the foot, four quarter panels, the speed lower right, the ring lower left. The reference housing is matt charcoal; this one shows a brighter polished rim (deliberate: the rim's physical catch-light).
  - V16 yes: board, render and before side by side: every zone in the same place at the same size; the render is richer than both (the lit bloom, the housing render, the beam). The board's full-width horizon line became the band over the panels (round 2, logged).
  - V17 yes: in the full frame: a polished and satin metal rim with its key highlight top left and right, the black glass face, grey lens caps, the dark sensor window.
  - V19 NOT ASSESSED: no refracting glass piece (the face is opaque black glass with a rim highlight; nothing behind it to bend).
  - V20 yes: stretch: 0 px outside the envelope plus 40, the corners off; every non-black pixel is an element, its glow or the glass surround (8/255 or less, the intended face); no rectangle edge or haze.
- **`design:design-critique`** (round 2, own context, findings only):
  1. The gear and the rpm figure lead, the speed is strong; it reads as the DDU's driving page. Works.
  2. "5200" in the disc is nearly as heavy as the speed, so ranks 2 and 3 sit close (the same on the real page). Not fixed (matches the reference).
  3. The panels' band of light reads a little like empty blue boxes because its ends are hard-cut. Minor; open for the closing review (the closing cold reviewer found the same; not fixed, see "Review findings").
  4. The unlit lenses are the brightest hardware at a glance. Fixed at the closing review (darker lenses, housing build 3).
  5. At the limiter the shift lamp is the palette's lavender, near white beside the white arc; the outer pairs steady and the middle strobe keep the state. Not fixed (the user's palette).
- **Decision: go.**

## Visual review
Answered at the final head (the render of `d320f91`, the code `a0cb477` carries, with housing build 3; lavapipe in the cloud container). Images are under `mockups/BoschDdu/review/` (gitignored) unless a path says otherwise; the committed crops are in `docs/handover-shots/`.

| Line | Answer | Evidence |
|---|---|---|
| V1 | yes | "Frozen expectations of the closing round" below, written at 07:08 UTC before housing build 3 or any final render existed; nothing in it was edited afterwards |
| V2 | yes | the cold look of the final `02_mid_cruise_cockpit.png`, `final_02_mirrored.png` and `02_mid_cruise_cockpit_luminance.png`, written at 07:13 UTC before the notes, board or references were reopened (seven plain sentences: the arched display low in the column gap, the 2-6-2 brow and 5 + 5 side lenses in dark grey, the eye's order 4 → 118 → 5200 → arc, every value, one blue accent with white figures and grey print, the rim and lens caps physical and the bands flat; the mirror and luminance views read the same). Likeliest rejection: the bands over G-FORCE and SPEED read as empty boxes (c) and the bead-blast grain is not visible at full frame (b). Patterns checked on named crops: (a) the DDU 10 layout holds on `02_mid_cruise_crop.png`; (b) the grain does not show at full size, the material reads by the rim's highlight and wall; (c) one housing, as on the real unit, no backing panel; the bands stay the open finding; (d) the lit beam and rendered housing; (e) not a glass dash, the face is black in the luminance view |
| V3 | yes | one `cold-reviewer` run, packet `mockups/blind/8361` (seed 8361), its 14 findings recorded under "Review findings" as fixed or not fixed; not rerun |
| V4 | yes | the cold reviewer's first three: the white gear "4" in its blue trapezoid, 5200 rpm, 118 mph |
| V5 | no (stays no) | blind read of seed 8361: gear right on 8 of 8, speed right on 8 of 8, shift lights as each frame shows them; the rpm zone read "high" on the three limiter frames (02, 06, 07), with its own caveat "if the limiter is at the red mark below 9, it would be limiter instead". Changed in the dash since round 1: the red zone now runs from the shift point with a wider limiter tick, and the arc turns to ink over the shift point. The reviewer does not know the car's limiter, so the key's "limiter" was not read from a still |
| V6 | yes | `glance` over `<state>_cockpit.png` (01 / 02 / 03 / 04): gear 10.4 / 16.4 / 15.1 / 14.8:1 (7:1), speed 11.7 / 12.1 / 11.4 / 12.1:1 (4.5:1), rpm 9.7 / 5.7 / 6.3 / 6.9:1, power 8.0 / 7.1 / 5.8 / 6.6:1, boost 6.6 / 6.7 / 6.7 / 5.9:1, G 5.6 / 5.2 / 5.6 / 4.9:1, lit LEDs 7.0:1 (03) and 9.8:1 (04); unlit LEDs carry no information. Boost's zone takes in the black margin right of the value (the same zone over the board v1 layout gives 6.1 to 6.9:1: the margin, not the layout change, takes boost over 4.5:1 in the gate; the layout change gives the value 7.5 px clear of the accent label, against about 5.5) |
| V7 | yes | squint views `01_idle_cockpit_squint.png`, `03_redline_strobe_cockpit_squint.png`, `04_drift_braking_cockpit_squint.png` and the brow of sequence frames 30 (strobe off: the four outer lenses lit) and 33 (on: all ten): idle shows a dark brow; 04's green and amber stages and the red and amber side columns are distinct blobs; 03's lit row is a bright band across the brow; on (ten lit) and off (four lit, a gap in the middle) differ from each other and from idle (none lit) |
| V8 | yes | `Geom` (cockpit): gear cap 132 px, speed 58, rpm 36, power 28, G 26, boost and SHIFT figures 18, rev numerals 20 (all ≥ 17.7 px), gear ≥ 40 px; the unit test of ISO 15008 minimums (`mod.rs`, "every dynamic readout at least 17.7 px cap") passes; the hood table at `HOOD_SCALE` keeps only parts ≥ 17.7 px (test). Thinnest information stroke: the scale ring and needle at 2 px, confirmed on `02_mid_cruise_crop.png` |
| V9 | yes | `flash --frames 30-41` (`flash_map_30-41.png`): area 1 is the strip; area 2 is the G figure (206 px), which is the review data's own decay from 0.60 to 0.16 g over frames 26 to 37, monotonic and not an alternation (`g_26_45` crop); no rpm, speed, gear, power or boost digit is marked (the rpm digits and band hold the 0.35 s peak) |
| V10 | yes | in each squint view the labels, numerals, lenses and chin fade first, the housing rim stays as a faint outline, then the arc and band, and the gear in its beam with the speed go last; the same in all four states |
| V11 | yes | `static` over the plain four states and the sequence: p95 0.194 (cap 0.2). Areas it marks over the cap, each explained: the ticks and numerals at 0.206 to 0.213 (the print sits at the cap, its anti-aliased cores read a hair over); the housing rim's catch-light top left (max 0.925, 1326 px, a thin physical highlight that moves with the virtual light between azimuths); the rpm's last digit (always 0 in the review data); stroke parts shared by changing digits (speed, the boost decimal, the G's leading "0."); the housing's left wall line (0.36, thin). No large near-white area |
| V12 | no, for the colour budget only | red appears only in the red zone from the shift point, the brake column and the brake bar; in every `cvd` panel (`02_mid_cruise_cockpit_cvd_sheet.png`) the shift stages read by count and position, brake and slip by side and count, the red zone by its place from the shift point and the wider limiter tick. The product page's badges: red 2 to 4 % (budget 5 %, yes), white 3 to 4 % (budget 15 %, yes), base hue 5 to 8 % (budget at least 80 %, no): the lit pixels are mostly the housing's neutral metal and the grey print, which the badge counts as lit (request 8). Reported as a rule question, as the LiquidGlass handover did |
| V13 | no (stays no) | quoted: "Car, maker or display: don't know." (seed 8361; round 1 said "Don't know" too). Changed in the dash since: the Blender DDU 10 housing with its 2-6-2 brow and 5 + 5 side lenses. The Bosch marks are left off (RESEARCH change A), which takes away the one unprompted cue |
| V14 | yes | the frozen inventory F1 to F10 with measured values at the direction check (round 2, "Direction check" above), unchanged by the closing fixes: F4 10 lenses in 2-6-2 and F5 5 + 5 counted on `02_mid_cruise_crop.png` after build 3; F9 colours by `probe` (primary blue, the red zone, the LED stages); every other line as measured at round 2; changes A to L in `RESEARCH.md` |
| V15 | yes | the round 2 sheets (blur 6, both orders) hold for the final render: build 3 changed only the lenses' brightness and the boost row only its unit's place, which leave the masses unchanged (the arched brow, the centre disc, the beam at the foot, four quarter panels, the speed lower right, the ring lower left) |
| V16 | yes | board v2 (`BoschDdu_01..04`, children `347e5ae2-…-038de5f8b368`, `…038e090b456b`, `…038e0b748e35`, `…038e0d90ae35`) against the final render: every zone in the same place at the same size, the boost row's value and unit on the right as the board draws them, the horizon band over the panel columns only; the render is richer than the board and the before render (the housing render, the beam's light, the bloom) |
| V17 | yes | full frame and cockpit composite: a satin anodised body with a polished rim catching the key light top left and top right, the black glass face, smoked lens caps, the sensor window; one key light from the virtual light's azimuth |
| V18 | yes | the materials checks that apply (metal, LEDs, light and shadow, motion of light, whole-part checks; glass refraction and carbon dropped: the face is opaque black glass and there is no carbon): the rim's highlight follows the arch and sits on the light's side; the lit lenses glow from inside with a hot centre and a short bloom that stays on the lens; unlit lenses are smoked dark caps (sRGB 41 to 53 on the sides); the key moves between azimuths with lateral G (`02` against `04`), and every part shares it; no part floats without a contact edge |
| V19 | NOT ASSESSED | no refracting glass piece: the face is opaque black glass with a rim highlight, with nothing behind it to bend |
| V20 | yes | `stretch` (`final_stretch_env.png`, `02_mid_cruise_stretch8.png`): 0 px outside the envelope plus 40 on the full frame; the corners off by `probe`; every non-black pixel is an element, its glow, the faint glass surround or the gear glow's spill on the chin (at most 8/255); no rectangle edge, haze or banding step |
| V21 | yes | `motion` (`motion_profile.png`): 0 stalls in the pulls, one spike at frame 10 (the pull starts); the strip's stages change on the frame the rpm crosses each share; the needle and band trail by at most a frame and never overshoot (the band follows the held peak on the limiter); gear and digits change on the data's frame |
| V22 | yes | `flash --frames 30-41`: worst 10° field 5588 of 21526 px allowed (yes), strip height 28 px (yes), red flash 0; area 1 (the middle six, 5588 px) shows 3 on / 3 off at 10 Hz, one equal whole number with no range; area 2 is the G figure, a digit (line 9). NOT ASSESSED by the tool: the reduced-flash mode (not rendered; judged from the code and its unit test, a 2 Hz breath between 0.6 and full that never goes dark) |
| V23 | yes | `final_gear_39_50.png` at 1:1: 3 through frame 41, 4 from frame 42 (the data's frame), never blank or doubled; the 1.12 → 1.0 entry ends within 6 frames (committed as `docs/handover-shots/cluster-bosch_ddu-gear_change_39_50.png`) |
| V24 | yes | each `motion` spike checked: frame 10 is the pull's start (band, needle, digits, throttle bar); frames 29 to 30 the limiter (strip lit), 42 the upshift (gear, band drop, digits); no shimmer, crawling edge, bloom pop or late glyph in the pairs' `diff` |
| V25 | yes | `final_before_after.png` (committed `cluster-bosch_ddu-before_…` and `-after_02_mid_cruise_cockpit.png`): the before's named failure, the v2 dash's flat slab with stuck-on boxes, is gone (one rendered DDU 10 housing with a live screen); nothing the before did better is lost except the size of gear and speed, which shrank on purpose to the DDU 10's proportions (RESEARCH change B) |
| V26 | yes | the four `tests/golden/images/BoschDdu_*.png` were opened in one sheet against the previous goldens (`goldens_sheet`): every difference is the rebuild (against the v2 dash's goldens, `bcfe234`), build 3's darker lenses and board v2's boost row. SHA-256 `86b8edab…`, `9d94dc1d…`, `e5a9e20b…`, `7554e670…` |

### Frozen expectations of the closing round (written 07:08 UTC, before housing build 3 or any final render existed)
Carried unchanged from the direction check: the rank order, F1 to F10 and their tolerances, the key frames and TJX's rejection patterns. Added from the closing findings, before re-rendering:
- E1 the unlit lenses darker than build 2: a lit side LED's lens body at least 3:1 over an unlit one (WCAG 1.4.11), unlit lenses about sRGB 45 to 80.
- E2 the housing still under the static limits (p95 ≤ 0.2, screen p99 ≤ 0.004; the recipe now fails the build otherwise).
- E3 the limiter-hold flash map shows only the strip's strobe (one area, 3 on / 3 off) and the G figure settling; no needle or band area.
- E4 the SHIFT box empty when the limiter is unknown (unit level; the review telemetry always has one).
- E5 boost at 4.2:1 in `03_redline_strobe` stays (not fixed, reason under "Review findings"); every other figure as at the direction check.
Tolerance changes: none.

### The cold reviewer, closing round (one `cold-reviewer` run, packet `mockups/blind/8361`, seed 8361, on the render of `f17087c` with housing build 2)
- First, second, third: the large white gear "4" in a blue trapezoid at the bottom centre, then 5200 rpm, then 118 mph. What it is for: "a driving cluster for racing".
- Car, maker or display: "don't know".
- Glances (it took the limiter as the dial's end at 9, not knowing the car's limiter): gear right on all 8; speed right on all 8; shift lights described as each frame shows them; the rpm zone on the limiter frames (02, 06, 07) read "high", with its own caveat "if the limiter is at the red mark below 9, it would be limiter instead".
- R1 (the DDU 10 demonstration page) against the render: same layout in both orders, "only the content differs"; R2 (a menu page) different in both orders.
- Its findings are under "Review findings".

## Before and after pair
- Before: the v2 dash on `main` before this rebuild, `docs/handover-shots/cluster-bosch_ddu-before_02_mid_cruise_cockpit.png` (kept at the start in `mockups/_before/BoschDdu/`).
- After: `docs/handover-shots/cluster-bosch_ddu-after_02_mid_cruise_cockpit.png` (the final render, lavapipe; SHA-256 `3247e7f9…`). Review crops beside it: the four states' envelope crops (`-01_idle` to `-04_drift_braking`), the engine panel at 3x (`-engine_boost_3x`) and frames 39 to 50 of the gear change (`-gear_change_39_50`).

## Assets and masters
Housing build 3 (the closing review's darker lenses): rendered on the cloud lane from the recipe at `aaa0c80` (a 3 min 10 s pod, about $0.05), the masters copied to Drive (`hero_bosch_ddu_housing/beauty_0..2.png`), box-filtered here by the recipe's own `box_filter` and checked against the PC's SHA-256s. Runtime files: `housing_0.png` `71d5ccb07896a064…`, `housing_1.png` `3b37242f8729717e…`, `housing_2.png` `ec58b1f3759d58d7…`; luminance p95 0.018 / 0.015 / 0.023, screen p99 0.0011 / 0.0016 / 0.0009 (limits 0.2 / 0.004).

| Runtime file (git) | What | Recipe and seed | Licence |
|---|---|---|---|
| (none: rendered into the housing) | ComfyUI bead-blast macro, seamless 4096 px, made neutral | `comfyui:bosch_ddu::BEAD_BLAST`, SDXL base 1.0 (+ refiner), seed 71207 | CreativeML Open RAIL++-M output; MIT (asset-forge) |
| `housing_0.png`, `housing_1.png`, `housing_2.png` (880×560, 1 px per 4K px) | the DDU 10 housing at key azimuths −135°, −160°, −110° | `blender:hero` `bosch_ddu::housing_scene`, Cycles 384 samples, 2 px per 4K px, box-filtered to 1:1 | MIT (asset-forge) |

Masters in `FH6 WheelDash v2 - 4K Masters/bosch_ddu/`: `comfyui/bosch_ddu_bead_blast.png`, `comfyui/bosch_ddu_bead_blast_neutral_a443e36f330a.png`, `hero_bosch_ddu_housing/beauty_0.png`, `beauty_1.png`, `beauty_2.png`, with their `*_preview.png`.

Cloud spend: this thread's builds on the cloud lane, the bead-blast and housing builds 1 to 3, came to well under $1 (build 2 $0.074, build 3 about $0.047); the lane's total was $2.90 of the $50 after build 3. Every pod was started through `asset-forge` and stopped with `cloud down` at the end of its batch.

## Changes from the real display and from the brief
The changes from the real display are `RESEARCH.md` changes A to L: no Bosch marks (A), the gear leads and the family's size minimums are not met in a 7-inch layout (B), honest panels for data the packet carries (C), thresholds from the car (D), the side columns' sources (E), no alarm boxes (F), the 2027 step (G), larger lenses (H), the red zone (I), static print at the cap (J), the strobe from the shift point (K), Saira Semi Condensed 600 for the figures (L).

From the brief (`spec.md`, the v2 brief): the event page (LAST / Δ BEST), the alarm lamp row and the tyre-load quad are gone, because the real DDU 10 driving page has none of them and the packet carries no lap or tyre data; the shift colour is the palette's, not the brief's `#2F6BFF`; the brief's carbon surround is replaced by the DDU 10's anodised body.

## Shared places touched
- `docs/TELEMETRY_USAGE_MATRIX.md`: this dash's row; the four v2 `DASH_SCENARIOS` rows removed and the note that named them reworded.
- `crates/fh6-clusters/tests/dash_gates.rs`: the four v2 BoschDdu scenarios and their `standing()` helper removed (`DASH_SCENARIOS` is empty).
- `docs/handover-shots/cluster-bosch_ddu-*.png`: this dash's own crops.
- No hero-library part, optics effect or decision-log row was added.

## Requests for shared code
1. `fh6_kit::GBall::snap`, so a dash can seat the G ball on a snapped frame without stepping it (BoschDdu steps it locally).
2. A pressure-unit setting (bar or kPa beside psi) in `AppConfig`; boost reads in psi until then.
3. Export the kit's virtual-light `REST_DEG` and `SWING_DEG`, so recipes and dashes stop copying them.
4. Fallback metal tokens (anodised, polished rim) in `data/design_tokens.json` for procedural housings.
5. Zone minimums per family that fit a 7-inch race display (gear, speed and LED row; see RESEARCH change B).
6. Tabular figures aligned on the ink edge rather than the cell edge, for flush-right columns.
7. Bloom that honours the painter's clip, so a lit gear's glow cannot leave its screen.
8. The product page's base-hue badge counts the housing's metal pixels as hue; it should skip the figure and hardware zones.
9. `preview-export` states for a calm (reduced-flash) limiter and for reverse, so both can be reviewed.
10. A `TopLeds`-style 2-6-2 strip in `common/` (the brow logic here is generic).
11. A hero rig option without the overhead softbox and sky, for flat glass faces that must stay black.
12. `reduced_flash` on by default at first run, or a clearer OSD row name ("Limiter: strobe / gentle pulse (photosensitive-safe)") (accessibility review A6).
13. One source for geometry shared by a recipe and its dash (the recipe copies `GEOM` and `HOUSING_FRAME` and checks them against the dash's source text; rust review R9).

## What sibling dashes must know
- A flat black glass face mirrors the hero rig's overhead softbox as a grey veil over the screen. Drop the softbox and the HDRI, light from the side, and measure the sprite's screen p99 (BoschDdu's recipe fails the build over 0.004).
- On the limiter the review rpm alternates 8690/8640: anything that follows live rpm there (digits, a band head, a needle) flickers at 30 Hz in the flash audit. Feed it a held peak.
- If every shift LED goes dark in the strobe's off phase, a still frame reads as "no shift needed". Keep the outer LEDs steady and strobe the middle ones.
- Ghost "888" digits behind a tabular figure are read as real digits in the blind read.
- At the glance gate's blur, thin figure strokes inside a ring lose contrast: Saira Semi Condensed 500 left the G at 4.4:1; 600 gives 4.9:1.
- A static label in `roles.ink` at 0.195 relative luminance reaches about 4.8:1 and stays under the 0.2 static cap.
- Unlit lens caps are the brightest hardware at a glance if their albedo is too high: albedo 0.014 and roughness 0.22 keep them at about sRGB 45 to 80 under the housing rig.

## Review findings
Each review ran once at the closing review. Findings and what was done.

**Cold reviewer** (seed 8361, render of `f17087c`, housing build 2):
1. The unlit top and side lenses are mid-grey and clearly visible → fixed: housing build 3 (lens albedo 0.014, roughness 0.22): an unlit side lens drops from relative luminance 0.055 to 0.091 to 0.022 to 0.036 (sRGB 41 to 53). That the side LEDs are never lit in the glances is the review data (no glance frame brakes or slides).
2. The blue strips over G-FORCE and SPEED look like filler → not fixed: they are the DDU 10's horizon band, which the real page draws as a hard band under the gauge (feature added from the round 1 fidelity review); the board v2 draws it over the panel columns only, as the code does.
3. SHIFT 8440 in grey in a thin box, lower contrast → not fixed: the SHIFT figure is static print at the 0.2 static cap (change J), 4.8:1.
4. No "x1000" label, a small red zone, a thin needle → not fixed: the DDU 10 page has no x1000 label; the red zone runs from the shift point to the scale's end (6 px) with a wider limiter tick; the needle is 2 px at full ink.
5. A flat bezel and plate → fixed after this review: housing build 3 is the render it reviewed with darker lenses; the rim's catch-light and the satin wall read at full frame (V17); the plate is the black glass face of the real unit.
6. The G ring and its dot are unclear → not fixed: the ring is full at 1.5 g, so 0.2 g is a short arc (as at round 1).
7. Small grey labels → not fixed: static print at the cap (change J), 4.8:1 against the 4.5:1 floor.
8. The cluster fills a small area at the bottom centre → not fixed: the cockpit envelope; gear and speed read at a glance (V6).
9. 02 against 06 and 07 at the same values, partly and fully lit → the strobe's off and on phases (V7, V22); intended.
10. The lit colour is a pale lavender, partly and fully lit look alike → not fixed: the shift colour is the user's palette; the outer pairs steady and the middle six strobing carry the state.
11. The arc turns white near the limiter → not fixed: the band cross-fades to ink over the shift point on purpose, so the red zone and the strip carry the warning; the band stays readable against the red zone.
12. Speed, rpm, power and boost do not agree → the review telemetry, not the dash.
13. 0.12 g while stopped in N → the review telemetry.
14. The "N" is light grey → not fixed: neutral is drawn at 72 % of the gear's ink on purpose so it reads as "no gear" (still 10.4:1 at the glance gate).

**`design:design-critique`** (once, on the final render: the full cockpit frame, `02_mid_cruise_crop.png` at 1:1, the four states sheet, the squint views; against "The bar" and the motorsport family):
- C1 The gear in its lit beam leads, then 118, then 5200: the DDU's driving page reads at once. Works.
- C2 The boost row now reads as a value with its unit and the accent label alone on the left. Works.
- C3 The horizon band's hard ends still read a little like empty fills at full frame → not fixed (as cold reviewer 2: the reference's horizon band).
- C4 At the limiter the arc is near white beside lavender LEDs; the danger reads from the strobe and red zone, not the arc → not fixed (deliberate cross-fade; the user's palette).
- C5 The neutral "N" at 72 % separates neutral from a gear without losing glance contrast. Works.
- C6 The rendered housing, the beam's light and the hairline scale ring carry it past a flat widget; the screen's content is flat print, as on the real unit. Works.
- C7 The bead-blast grain is not visible at full frame; the material reads by the rim and wall only → not fixed: at 1 px per 4K px the grain sits under what a full frame resolves; the rim highlight and wall carry the material (V17, V18).

**`design:accessibility-review`** (once, on the render of `f17087c`):
- A1 Boost 4.2:1 at the glance gate in `03_redline_strobe` (WCAG 1.4.3) → fixed: board v2 put the unit after the value (Penpot first, then `d320f91`); the value stands 7.5 px clear of the accent label; boost 5.9 to 6.7:1 in all four states (the measured zone includes the black margin right of the value; the old layout measures 6.1 to 6.9:1 with the same zone, see V6).
- A2 Lit and unlit side LEDs only 2.6:1 apart (1.4.11) → fixed: housing build 3; a lit side lens body (0.399) is now over 5:1 above an unlit one (0.022 to 0.036).
- A3 The G figure's hundredths change at 10 to 30 Hz on the limiter hold → not fixed: the flash map's G area is the review data's own decay (0.60 to 0.16 g over frames 26 to 37), monotonic, under the area threshold; the 3-hundredths hysteresis stops any alternation (V9).
- A4 The needle and red zone flicker at 30 Hz on the limiter → fixed (`f17087c`): the band and needle follow the held peak; the final flash map has no needle or band area.
- A5 The idle dimmer takes the static labels under 4.5:1 → not fixed: the car is parked when the dimmer acts and nothing needs reading.
- A6 Reduced flash is off by default → not this dash: request 12 (a default or a clearer OSD row name).

**`rust-review`** (once, on `mod.rs`, `hardware.rs` and the recipe):
- R1 The band followed the held peak while four comments said it shows live rpm → fixed (`f17087c`): the docs now say the band and digits follow the held peak.
- R2 An over-bright housing sprite was written with only a warning → fixed (`41dce3f`): the recipe measures before writing and fails the build.
- R3 The SHIFT box printed 0 with no limiter → fixed (`41dce3f`): empty when the limiter is unknown, as the strip and red zone.
- R4 A stale gap-detection comment → fixed (`41dce3f`).
- R5 The limiter tick sits on the first thousand at or above the limiter → doc fixed (`41dce3f`): the doc says where it sits; no extra tick (the red zone starts at the shift point).
- R6 `shift_lamp` recomputed per lit LED → not fixed: three luminance evaluations for up to ten LEDs a frame, only over the shift point; the live bench puts the dash's whole CPU share at 0.015 ms, so it buys nothing measurable.
- R7 A manual save/restore pair → fixed (`41dce3f`): `Painter::scoped`.
- R8 `unwrap_or_default` on the fingerprint JSON → fixed (`41dce3f`): the error propagates.
- R9 Geometry duplicated in the recipe and checked by source text → not fixed: asset-forge does not depend on fh6-clusters; request 13.
- R10 Nits (`animating` by value, long doc lines) → fixed (`41dce3f`); the private bool parameters stay (within the tolerance for private helpers).
