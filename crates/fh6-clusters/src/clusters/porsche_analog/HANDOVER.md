# HANDOVER: Dash · PorscheAnalog (catalog row 7)

**Thread / branch:** "Rebuild PorscheAnalog hardware dash", `claude/project-thread-urfb8s`, merged into `main` (AGENTS.md rule 16 and the project's standing rule)  **Date:** 2026-11-27  **Result:** ✅ complete. Closed by self-review: TJX asked to be hands-off (AGENTS.md rule 12); no TJX answer arrived in the thread, so nothing of TJX's is quoted.

Findings and yes/no answers only; there are no scores in this file.

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| The introduction was posted, and the thread is not archived | yes | the thread's first message; the thread is open |
| `RESEARCH.md` is written, with sources | yes | [RESEARCH.md](RESEARCH.md): 17 features with sources, the match table, deliberate changes A to L, the gaps |
| The design board is the Penpot page `07 · PorscheAnalog`, 4 states, its IDs in the catalog row and here | yes | page `b568ed3f-64cd-806f-8008-fd946dce7013`, boards 01 `…fd946df67f24`, 02 `…fd946ee9892c`, 03 `…fd946f74f160`, 04 `…fd946ff7365f` (prefix `b568ed3f-64cd-806f-8008-`); CATALOG.md row 7; exports in `mockups/PorscheAnalog/board/` |
| A bespoke ComfyUI asset and a bespoke Blender asset are used, with ledger rows and 4K masters on Drive; the before and after pair is here | yes | [assets/clusters/porsche_analog/ASSETS.md](../../../../../assets/clusters/porsche_analog/ASSETS.md); "Assets and masters"; "Before and after pair" |
| The module is complete, with no TODOs, and goldens pass | yes | "Verification output" |
| 4K renders, the 60-frame sequence and the product page are in `mockups/PorscheAnalog/` | yes | written by the final `preview-export` run (gitignored) |
| The live bench report and the GPU time under 0.5 ms are recorded | yes | "Verification output", "Live bench" |
| The direction check was posted and self-reviewed (go with changes, changes done), with its findings here | yes | "Direction check" |
| Every line of the closing review is yes, or a line that could not be fixed afterwards is reported | yes | "Closing review": lines 30 and 34 are a no about past work, and line 39 is a no: V4 is a no from the one cold-reviewer run (what changed is listed), and V6 (the small static labels, a rule question), V8 (the "+" cue at cap 24) and V15 (the track face against the 992.1 photo) are noes this dash cannot change without changing the design. Each is reported in the thread and to the coordinator |
| The visual review (V1 to V26) is here, each line with its evidence | yes | "Visual review" |
| The handover is written, the job is reported for the merge, and the product page and checklist are posted | yes | this file; the thread's closing message |

## What was built
- `crates/fh6-clusters/src/clusters/porsche_analog/`
  - `mod.rs`: the zone table (`Geom`), the options, the cluster (`update`, `draw`) and the rev counter: scale, logo zone, the gear window under the cyclops, the 11-lens shift ring, the needle and the illuminated hub.
  - `scale.rs`: the 240° face, Track (the shift rpm at 12 o'clock, the default) or Classic, and the top rule (the first whole thousand at least 500 rpm past the limiter).
  - `tubes.rs`: the four other tubes (speed, G, power, boost; each swappable or hidden), the power and boost scale tops, the outer gauges' end values and the driver greeting.
  - `hardware.rs`: the Blender sprites (three light azimuths cross-faded by the virtual light; the needles at eight azimuths, picked by their angle against the light) and the tubes' procedural fallback; the hub, window rim, needle and pointer fallbacks are drawn where those parts are (`mod.rs`, `tubes.rs`).
  - `readout.rs`: digital readouts that refresh at 5 Hz with hysteresis, and the needle's hold through a bounce off the limiter.
  - `tests.rs` plus the submodules' own tests: 41 unit tests (geometry, the tubes' tuck, the strobe band with its glow, the lens placement, the gear window with a two-digit gear, the logo zone on 72 scales with the needle kept off it, the magnifier's optics, options and every tube value, numeral spacing, the readouts, the limiter hold and its release, the scale tops and the scale's clamp, the outer gauges' end values and value fit, the digit layout, the sprite frames against the zone table, the pointer's scaling).
  - `RESEARCH.md`, `README.md`, this file.
- `tools/asset-forge/src/recipes/porsche_analog.rs`: the ComfyUI dial paint (SDXL, made neutral and evened out locally), the Blender tubes, hub, window rim and needles, the runtime box filter, previews and the cluster's `ASSETS.md`; registered in `tools/asset-forge/src/{main.rs, sets/mod.rs, recipes/mod.rs}` (one line each).
- `assets/clusters/porsche_analog/`: 31 runtime PNGs and `ASSETS.md` (build 6).
- `tests/golden/images/PorscheAnalog_*.png`: the four goldens.
- `docs/handover-shots/cluster-porsche_analog-*.png`: the before and after pair and the review crops.

## Telemetry
Every field the dash reads is covered by the golden packets or derived from them:
- **Read as they are (through `fh6_kit::Signals`):** rpm (current, limiter), gear, speed, throttle, brake, the race flag, the car ordinal.
- **Read from the packet:** power (W) and boost (psi).
- **Derived:** lateral and longitudinal G from acceleration (the kit's smoothing); the rpm rate from rpm; the shift point from the limiter and `ShiftPointPct`.
- **Not read:** idle rpm, tyre slip, temperatures, fuel, suspension and every other offset the golden packets do not cover.
- **Held beyond the kit springs (line 7):** the needle's limiter hold (`readout::LimiterHold`): within 200 rpm of the limiter a drop under 200 rpm holds the last peak for 0.1 s, then it sinks at 400 rpm/s; a gear change, a drop of 200 rpm or more, or a lift out of the band releases it at once. For 0.15 s after a gear change the needle runs on its spring without the feed-forward (the rpm steps there, and the kit's rate still carries the old slope), and comes to rest on the rpm rather than swinging past it. The shift ring stays over the shift point until the rpm falls 120 rpm under it (the tokens' hysteresis). The digital readouts' 5 Hz refresh applies to power, boost and G only; rpm, speed and gear reach the screen on the next frame.

## Verification output
Final run: `main` at `af87bf4` (the pilot's close and the host CPU phase) plus this thread's changes, in the shared checkout with this thread's own target folder (`target/porsche_analog_review`), under the local `gpu.lock`. The lints and tests below also ran on an exact copy of the commit's tree (`git archive` of the commit, its own target folder), so nothing in them leans on another thread's uncommitted work.

`cargo run --release -p preview-export -- --cluster PorscheAnalog` (GPU time is the median of 5 frames per shot):
```
Porsche Analog (Classic Analog, cockpit, PorscheAnalog)
  01_idle                   0.308 ms GPU   28 draws   256 instances
  02_mid_cruise             0.389 ms GPU   36 draws   280 instances
  03_redline_strobe         0.393 ms GPU   36 draws   304 instances
  04_drift_braking          0.391 ms GPU   36 draws   288 instances
  sequence: 60 frames, GPU p50 0.392 ms, p99 0.394 ms
```
(21:39 UTC, the final code. The side crystals' own glass layer, drawn before the rev counter so its bezel covers them, and the contact shadows add a little over the closing round's 0.38 ms; setting the cyclops into the crystal's layer took one glass layer away again.)
Goldens and tests:
```
cargo test -p fh6-clusters --lib porsche_analog               (shared checkout, and the commit's tree)
  running 41 tests
  test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 200 filtered out
cargo test -p asset-forge porsche_analog                      (the commit's tree)
  test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 155 filtered out
FH6_UPDATE_GOLDEN=1 cargo test --release -p fh6-clusters --test golden_images porsche_analog   (21:36 UTC)
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out
cargo test --release -p fh6-clusters --test golden_images porsche_analog      (21:36 and 21:39, twice each)
  test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out
```
Each new golden was opened and compared with the previous ones (`visual-review diff`, `mockups/PorscheAnalog/review/goldens_diff/`): the changes are the tucked side tubes, the graphite bezels, the window and gear, the hub, the logo zone (the custom name below the hub, as the review data has no Porsche brand) and the pedal arcs; the rev counter's scale and numerals are unchanged.
Lints and format:
```
cargo check -p fh6-clusters -p asset-forge                    (the commit's tree)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 9.53s
cargo clippy -p fh6-clusters -p asset-forge --all-targets -- -D warnings      (the commit's tree)
    Checking fh6-render v0.1.0
    Checking fh6-clusters v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.02s
rustfmt --edition 2021 --check crates/fh6-clusters/src/clusters/porsche_analog/*.rs tools/asset-forge/src/recipes/porsche_analog.rs
    (no output: clean)
```

## Live bench
`mock-udp --scenario bmw_m4_cs --port 8000 --duration 100` streaming at 60 Hz, then `FH6_WheelDash --demo --cluster <Name> --stats --bench-seconds 60 --bench-out …` (reports in `mockups/PorscheAnalog/live_bench_run3*.md`; the bench was announced in the thread at 18:30 UTC).

| Run (20:20 to 20:23 UTC) | Frames | Packets shown | CPU per frame p50 / p99 (ms) | Receive → present p50 / p99 (ms) |
|---|---|---|---|---|
| PorscheAnalog | 8,071 | 3,514 | 0.748 / 1.494 | 0.830 / 2.176 |
| LfaV10Nurburgring (scaffold, ☐) | 8,194 | 3,511 | 0.771 / 1.575 | 0.843 / 2.421 |

These runs used the code before the final check's fixes, which only add a few comparisons per frame and remove one glass layer, so they bound the final cost from above. The dash's own share of the CPU: 0.748 − 0.771 = −0.023 ms, under 0.3 ms (the two runs are within noise of each other; runs 1 and 2 earlier in the day gave −0.32 and −0.04 ms). The app total's p99 over 1 ms shows on the scaffold too: known defect 10 in `docs/PROGRESS.md`, reported to the coordinator, no shared code edited. Receive to present stays under one 300 Hz frame at p50.

## Closing review
| # | Answer | Evidence |
|---|---|---|
| 1 | yes | no read of `0xF000` bits: the module reads no buttons at all (`grep` finds no `buttons` or `0x1000`…`0x8000`) |
| 2 | yes | no `todo!`, stub or hard-coded telemetry; every field under "Telemetry" |
| 3 | yes | no `unwrap`, `expect` or `panic!` outside tests; no `unsafe` |
| 4 | yes | `format!`, `vec!`, `collect`, `to_string`, `String::from`, `Box::new`, `tracing::`, `.lock()`, `std::fs`: three hits, all on the setup path called from `new()` (`make`'s `Box::new`, the sprite names' `format!` and the load warning in `hardware.rs`). `update()` copies the logo and the driver's name with `clone_from` only when they change (a new car's brand or an edited name), never per frame |
| 5 | yes | GPU per state 0.31 to 0.39 ms and sequence p99 0.394 ms, under 0.5 ms ("Verification output"); cockpit only, as the catalog lists |
| 6 | yes | −0.023 ms ("Live bench"); both reports with their p99 are above |
| 7 | yes | rpm, speed and gear reach the screen on the next frame; the limiter hold, the ring's 120 rpm hysteresis and the needle's feed-forward pause after a shift are bounded as "Telemetry" says |
| 8 | yes | the cluster draws only when the host calls `draw`; it adds no path of its own; the OSD's dyno frames (race flag off) teach it no peaks, and the first frame of a race-flag change or after a hidden gap takes the car's state as it is |
| 9 | yes | no non-Rust source; bpy is Rust strings in the hero library |
| 10 | yes | commits by `TJX <TJX@Ngage.com>` with no co-author trailer; no real name, user path or personal email in any file; `RESEARCH.md` links checked |
| 11 | yes | nothing copied to the game folder or the mirror |
| 12 | yes | no new dependency |
| 13 | yes | "Verification output" |
| 14 | yes | the kit's paints only; no alpha multiplied twice (the content opacity is applied once, by layer) |
| 15 | yes | every stroke ≥ 1 px (`width.max(1.0)`), every telemetry value sanitised (`is_finite` checks in `update`, `readout`, `tubes`) |
| 16 | yes | each sprite lookup has a fallback branch (`hardware::fallback_tube`, vector needle, ring rim, painted hub); assets load in `new()` only |
| 17 | yes | MSDF text from the kit; speed and readouts tabular; the gear, speed and dial numerals follow `input.fonts.display` and come back to the dash's faces when it is cleared |
| 18 | yes | off pixels transparent; no box or full-canvas fill (`stretch`, corners off) |
| 19 | yes | no wall clock (the strobe runs on its own clock in `update`); `cargo test -p fh6-clusters porsche_analog` passed twice without rewriting a golden |
| 20 | yes | colours from palette roles and tokens (the sprites' white tint is `ImagePaint`'s identity). Type: the dial and label faces are the tokens' families (Barlow Semi Condensed 600, Titillium Web 600); the gear and speed use Barlow Semi Condensed in place of the display role's Saira Condensed, logged as `RESEARCH.md` change M; the user's display font replaces all of them. The maker name (`MAKER`) is compared in `update`, never drawn as text |
| 21 | yes | the crystals and the magnifier use `fh6_kit::OpticalGlass` (`docs/specs/optics.md`); no new effect |
| 22 | yes | glance and squint in all 4 states (V6, V10): the gear and the lenses lead, then the speed, the tach, the secondary tubes, the bezels and labels |
| 23 | yes | gear 9.5 to 12.9:1 and speed 9.1 to 9.6:1 in the cockpit composites of all four states (V6) |
| 24 | yes | drawn round (1920, 1835); every part inside the cockpit envelope (test `every_part_stays_inside_the_cockpit_envelope…`); the catalog lists cockpit only |
| 25 | yes | `RESEARCH.md` names the 992.1 and 992.2 clusters with sources; 17 features matched or logged (changes A to L) |
| 26 | yes | five separate tubes, each its own `Geom` zone, on black; no backing panel (the real 992 has none) |
| 27 | yes | every tube, bezel, needle, hub and window rim is a Blender render drawn at full strength; the ComfyUI paint is the dials' surface (hardware-dash rule) |
| 28 | yes | `sequence/`: the strobe on frames 27 to 41 with its own clock, the gear change at 42 with the rim flash and the dark shift ring (V21 to V23) |
| 29 | yes | palette roles, the user's font, and the logo zone from `input.logo` (the maker's wordmark only on the maker's cars, nothing when the logo is off); no literal colour, font or brand in `draw()` |
| 30 | no (past work) | the order was R&D → Penpot → ComfyUI → Blender for builds 1 to 6, but the Penpot boards were redrawn after builds 5 and 6 rather than before them (the window and bezel changes came from the closing reviewers), so those two Blender passes preceded their board change. Reported in the thread and to the coordinator |
| 31 | yes | four boards, IDs above and in the catalog row |
| 32 | yes | `ASSETS.md`: the ComfyUI dial paint (seed 70701) and the Blender tubes, hub, window, needles (recipe, licence, master names) |
| 33 | yes | 4096 px ComfyUI master and Blender renders at 2 px per 4K px; masters listed under "Assets and masters" |
| 34 | no (past work) | every Blender and ComfyUI build held its lane's lock and the cloud builds ran `cloud down`; but one golden test run at 18:13 UTC ran for about 8 s without the local GPU lock (it was meant as a CPU test). Cloud spend: 4 sessions, 29 min, $0.431. Reported in the thread and to the coordinator |
| 35 | yes | `DIRECTION CHECK PorscheAnalog` posted 18:03 UTC with the render; "Direction check" below |
| 36 | yes | `mockups/PorscheAnalog/index.html`, written by `preview-export` |
| 37 | yes | "Live bench"; the sequence shows no jitter (V9, V21, V24) |
| 38 | yes | findings and yes/no only |
| 39 | **no** | "Visual review": V4 is a no from the one cold-reviewer run (allowed, with what changed). V6 (the small static labels under 4.5:1 in the blur gate, the rule question the pilot also reported), V8 (the "+" cue at cap 24 inside the 70 px porthole) and V15 (the track face against the 992.1 photo, a deliberate change) are noes this dash cannot change without changing its design. All four are reported in the thread and to the coordinator |

## Direction check

### Frozen expectations (written 2026-11-27 17:22 UTC, before the first render with the Blender assets was opened)

Rank order (fh6-hmi-design-rules 3): (1) the gear in its porthole and the shift lenses on the limiter, (2) the speed digits in the left tube, (3) the rev counter's needle and scale, (4) the G tube, power and boost, (5) bezels, labels, the logo and "1/min × 1000".

Feature inventory of the real display (from `RESEARCH.md`), each with the tolerance the render must meet:

| # | Feature | Reference | Tolerance |
|---|---|---|---|
| 1 | Five round tubes, the rev counter largest in the middle, each outer tube tucked behind its neighbour | 992.2 GT3 Touring and Turbo S Classic views | all five present; every overlap visible, 10 to 100 px |
| 2 | Rev counter scale sweep | about 240° (30° per 1,000 on an 8,000 face) | 240° ± 5° |
| 3 | Shift rpm at 12 o'clock on the rotated face | 992.2 rev-counter mode | 8,439 rpm at −90° ± 1° |
| 4 | A thick bar every 1,000 rpm, fine ticks between | 992.1 press photo | 10 bars on a 9,000 face, 100 rpm minors |
| 5 | Upright white numerals, none red | every official photo | numerals 0 to 9 upright, white |
| 6 | Red band on the outer edge from the maximum engine speed | QSG; GT3 page | red from 8,700 to 9,000, red minor ticks only there |
| 7 | Slim needle on a polished domed hub cap with a short tail | Carrera 4S press photo; 992.2 model pages | tail visible past the cap; needle in the palette primary |
| 8 | A darker inner zone inside the numerals | Carrera 4S press photo | inner disc visibly darker than the scale ring |
| 9 | "1/min × 1000" on the face | Carrera 4S, Turbo S | present, legible at 1:1 |
| 10 | Model lettering (here the logo zone) | GT3 / GTS / Turbo S | the Porsche wordmark from the repo, upright |
| 11 | Readout in the numeral-free sector | rotated GTS dial | the gear porthole centred in the dead zone |
| 12 | Coloured shift bars either side of the rev counter | GT3 | 15 lenses, filling from both sides to 12 |
| 13 | "+" beside the gear at the shift point | Carrera gear shift assist | shown on the limiter, hidden below the shift point |
| 14 | Speedometer in the left tube | 992.1 left display | 0 to 220 mph scale, pointer, digits |
| 15 | G-Force circular plot | Car & Info menu | dot, rings, throttle and brake |
| 16 | Driver greeting | "Hallo FAHRER 1" | idle state shows HELLO and the name |

Key frames of the sequence: the limiter is reached at frame 29 or 30, the strobe runs 30 to 41, the gear changes at 42.

TJX's rejection patterns to check against named crops: (a) it looks nothing like the real car's display; (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real display does not have; (d) flat modern widgets instead of the "2027 future" bar; (e) murky, tinted or frosted glass.

### Cold look (written 2026-11-27 17:42 UTC from `02_mid_cruise_cockpit.png` at full frame, before the board, the notes or the spec were opened)

Deviation, recorded honestly: the four reference screenshots (`mockups/references/PorscheAnalog/`) were collected at 17:33 to 17:39 UTC, before this render existed, so this look was not blind to them.

1. Five round dials sit in a row low in the black binnacle space, bottom centre: a large middle dial, a medium dial either side, and two small outer dials partly hidden behind those.
2. The eye goes first to the large white "118" in the left medium dial, then to the white "4" in a small round window at 3 o'clock on the middle dial, then to the orange needle pointing to 9 o'clock.
3. The middle dial's numerals run clockwise from 0 at about half past four, through 1, 2 and 3 along the bottom, 5 at 9 o'clock, to 8 at the top and 9 at about 1 o'clock, where a short red arc sits; the numeral-free gap is on the right, where the "4" window is.
4. The left dial is a speedometer, 0 to 220, with "118" and "MPH" in the middle and an orange pointer near 120.
5. The right dial holds concentric rings, an orange dot right of centre, a green arc at lower left and "THR 0.20 G BRK" along the bottom.
6. The outer left dial reads "104 HP" and the outer right "5.6 PSI", both small.
7. On the middle dial a small "PORSCHE" wordmark and "1/min × 1000" sit left of the hub; a ring of dark lens shapes runs round the bezel, unlit.
8. Colours: white numerals and ticks, orange needles, pointers and dot, one red arc, one green arc, on near-black faces inside grey and silver rings.
9. The bezel rings carry thin bright highlights and read as metal; the gear window has a bright rim; the faces read as dark matte paint; at this scale the needles read as flat strokes.
10. The cluster spans about 37 % of the frame width, inside the black scrim; the in-game car's own HUD (map at bottom left, speed at bottom right) shows outside it.

Luminance-only view (`review/02_mid_cruise_cockpit_luminance.png`): the white "118", the "4" and the numerals carry the most light; the orange needle drops to a mid grey, no brighter than the scale's minor ticks; the G dot and the green throttle arc are mid grey; the bezel highlights are the brightest thin lines.

**Premortem (likeliest reason TJX rejects it):** "It is too small and too dark to feel like a 911: five generic dials floating in black, with the low numerals upside down along the bottom, so it reads as a gauge pack rather than Porsche's cluster."

Rejection patterns against named crops (`02_mid_cruise_crop.png`, `03_redline_strobe_crop.png`): (a) no: the blind reviewer named a Porsche 911 992 GT3 from the five overlapping tubes and the scale; (b) no: the dial paint's grain is not meant to read, and every material reads without zooming (bezels, paint, crystal), but the hub cap reads as a dark square (fixed below); (c) no: five separate tubes on black, as on the 992.2's screen, no slab or backing panel; (d) partly: the outer tubes' few ticks and the thin G rings read flat (finding 16 below); (e) not a glass dash: the crystal and the magnifier stay clear, but the magnifier and its bright rim read as "a glass button stuck on" (finding 13, fixed below).

### Cold reviewer (one `ui-ux-designer` run at 17:53 to 18:02 UTC, packet in `mockups/blind/`: A.png, glance_01–08 at seed 7, R1–R3)

Caveat: the agent's context carries the project's `AGENTS.md`, which names the catalog's dashes, so its recognition answer is not fully blind.

- Five-second test: "first the central tach with the bright '4', second '118 MPH', third the G-meter's orange dot and green arc".
- Unprompted recognition: "Porsche 911, probably the 992 GT3. The clues are the five overlapping round dials and the central tach labelled '1/min × 1000' that ends at 9,000."
- Blind glance read against `blind_key.txt`: the gear right on every frame; the speed within 1 mph on every frame; the shift-light state wrong on the strobe's off-phase frames (35 and 39, read as "filling"; frame 31, the on phase, read as full).
- References: same five-tube arrangement as R1 and R3 in both orders; the rev counter's face differs from R2 (rotated scale, no window under the hub).

Its findings, each with what happens to it (fix round 1 unless it says not fixed):

| # | Finding | Outcome |
|---|---|---|
| 1 | Black box left of the hub | Not a defect: the packet's logo mask |
| 2 | "HELLO, TJX" drawn over the live G plot while moving | Fixed: the greeting and the G plot take turns, never both at once |
| 3 | Shift lenses light beside the low numerals | Fixed: 11 lenses on the upper arc only (±65° round 12), over the approach to the shift point and the red zone |
| 4 | Limiter barely differs from nearly full | Fixed: on the limiter every flank lens turns steady white and the crest strobes |
| 5 | Haze over dark lenses | Fixed: the lenses glow tightly and no longer feed bloom |
| 6 | The red zone is a hook | Fixed: the scale runs to the thousand at least 500 rpm above the limiter (8,700 → 10, as on the GT3's real face) |
| 7 | 0.12 G at a standstill | Not fixed: the review telemetry holds 0.12 g lateral at idle; the dash shows its input |
| 8 | 118 mph at 5,200 rpm vs the sequence's numbers | Not fixed: the review states and the sequence are separate data |
| 9 | "THR0.20 GBRK" | Fixed: the labels move to the arcs' top ends and the value drops between the rings |
| 10 | Ticks and pointers touch numerals and values | Fixed: outer values moved onto the clear face, speed numerals moved in, a true minus sign |
| 11 | Labels 9 to 11 px | Fixed: every label at least the tokens' 12 px floor |
| 12 | The "+" too small | Fixed: larger "+" |
| 13 | The gear lens reads as a glass button | Fixed: no bloom fringe on the gear, a lower magnifier dome |
| 14 | Unlit lenses look like punched holes | Partly: 11 lenses instead of 15 |
| 15 | Dark square hub | Fixed then: satin silver cap under the full light (black satin since build 6) |
| 16 | Faces flat, bezels read as glossy plastic | Fixed in part: machined bezel tops (graphite titanium since build 6) |
| 17 | Rotated scale | Not fixed, deliberate: the 992.2's rev-counter mode puts the shift point at 12; `PorscheAnalogScale` = `Classic` gives the 992.1 face |
| 18 | Gear and speed not under the hub | Not fixed, deliberate (`RESEARCH.md` change B) |
| 19 | No full speedometer needle | Not fixed, deliberate: a centre needle would cross the rank-2 digits |
| 20 | Throttle left, brake right is the opposite of the pedals | Fixed: brake left, throttle right |
| 21 | LED ring not in the photos | Not fixed, deliberate (`RESEARCH.md` change F: the GT3's shift bars, set into the bezel) |
| 22 | No telltales, fuel or cowl | Not fixed, deliberate (`RESEARCH.md` change D) |

### Measurements (17:45 to 18:00 UTC)

- `glance`, cockpit, all four states: gear 8.4, 10.8, 10.0, 10.2 :1 (needs 7); speed 10.2, 10.8, 10.6, 10.6 :1 (needs 4.5); lit crest lens (03) 8.7:1; lit green lens (04) 7.5:1; needle (04) 5.1:1 (needs 3).
- Squint (`02_mid_cruise_cockpit_squint.png`): the gear and the speed digits last, the numerals next, the bezels and labels fade first.
- `flash --frames 30-41`: **no**. The strobe band is 44 px tall with its bloom (limit 42); the needle buzzes red at 30 Hz with the limiter bounce; the power, boost and G digits change every frame. Fixes: lenses out of bloom, the needle holds the bounce's peak, digital readouts refresh at 5 Hz with hysteresis.
- `static`: static p95 0.162 (≤ 0.2) and dimmer than the live data: yes. `stretch`: corners off; the only faint spread is the lenses' bloom halo.
- `unwrap` (`review/tach_unwrap.png`): 0 rpm at 45°, 9,000 at 285° (sweep 240°), shift rpm at 270° (12 o'clock), red from 277°, needle at 183.75° = 5,203 rpm (data 5,200); ticks every 100 rpm.

### Tolerance changes (logged before re-checking)

Inventory lines 4 and 6 assumed a 9,000 face (the next thousand above 8,700). After finding 6 the scale rule changes to the thousand at least 500 rpm above the limiter, so the review car gets a 10,000 face: 11 bars, red from 8,700 to 10,000. Reason: Porsche's GT3 (limit 9,000) reads to 10 with a red band from 9 (R3), and a 300 rpm band reads as a hook. Line 3 stays: the shift rpm at 12 o'clock.

Inventory line 12 assumed 15 lenses round the top. After finding 3 it becomes 11 lenses within 65° of 12 o'clock, filling from both ends to 12 (`RESEARCH.md` change F). Reason: a lens beside the low numerals reads as running against the scale.

Inventory line 10 (logged at the closing round, 19:50 UTC): the logo zone shows the Porsche wordmark on a Porsche, else the brand or the custom name, and sits below the hub on the track face (`RESEARCH.md` change I). Reason: the code reviewer found the wordmark printed on every car and with the logo turned off, and two reviewers found it under the needle at the shift point.

### DC answers and decision

| Line | Answer | Evidence |
|---|---|---|
| 1 | yes | expectations 17:22 UTC, render opened 17:41 |
| 2 | yes, with the deviation above | cold look 17:42 UTC; patterns (a) to (e) above |
| 3 | yes | reviewer table above |
| 4 | yes | "first the central tach with the bright '4'" |
| 6 | yes | glance numbers above |
| 10 | yes | squint view |
| 13 | yes | "Porsche 911, probably the 992 GT3" |
| 14 | yes | inventory: lines 1 to 3, 5, 7 to 16 matched or logged; 4 and 6 re-checked after the tolerance change |
| 15 | yes | `review/real_vs_render_AB.png` and `_BA.png`: same five-tube arrangement in both orders |
| 16 | yes | `review/board_before_render.png`: every board zone in place; the render is richer than the board and the before |
| 17 | no → fix | the hub cap reads as a dark square |
| 19 | not a glass dash; crystal and magnifier clear | `02_mid_cruise_crop.png` |
| 20 | yes | `review/02_mid_cruise_stretch8.png`, corners off |

**Decision: go with changes** (18:05 UTC). The concept holds (recognised blind, gear first, contrast passes); fix round 1 is the twelve fixes in the table above. Posted in the thread at 18:03 UTC.

## Visual review

### Frozen expectations of the closing round (written 18:46 UTC, before the final render existed)

The direction check's frozen inventory stands, with the logged tolerance changes: line 4 is 11 bars on a 10,000 face with 100 rpm minors; line 6 is red from 8,700 to 10,000; line 12 is 11 lenses within 65° of 12 o'clock filling from both ends. Added for this round: the limiter shows every flank lens steady white with only the three crest lenses strobing (3 frames on, 3 off); the needle holds still through the limiter bounce (frames 30 to 41); power, boost and G digits change at most once in frames 30 to 41; the gear stays inside its porthole while it slides in (frames 42 to 44); the outer gauges show their scale's end values. Key frames as before (limiter from frame 29 or 30, strobe 30 to 41, gear change at 42). The rejection patterns (a) to (e) are re-checked on named crops.

### Closing-round cold look (19:20 UTC, `02_mid_cruise_cockpit.png`, its mirror `review/final_02_mirrored.png` and `review/02_mid_cruise_cockpit_luminance.png`)

Not blind: this thread had seen the references and its own notes in the direction check.

1. Five round tubes sit low in the black binnacle, the largest in the middle, the outer pair tucked behind the side pair.
2. The eye goes to "118" first, then the "4" in its round window at 3 o'clock, then the orange needle at 9 o'clock.
3. The middle dial runs from 0 at about 5 o'clock clockwise round the bottom and the left to 8 near 12 and 10 at about 1 o'clock; a red band runs from just past 12 to the 10.
4. "PORSCHE" and "1/min × 1000" sit above the hub; the hub is a small bright silver dome.
5. The left tube reads 0 to 220 with "118 MPH" in the middle; the right tube holds rings, an orange dot, "BRK" and "THR" at the top, "0.20 G" at the bottom and a green arc on the right.
6. The outer tubes read "104 HP" (scale 0 to 300) and "5.6 PSI" (scale −15 to 20).
7. Colours: white numerals and digits, orange needle, pointers and dot, one red band, one green arc, near-black faces, grey titanium rims with bright edges.
8. In the luminance view the digits and numerals carry the most light, the needle drops to a mid grey, and the red band reads as a grey band at the scale's end.
9. Bezels, hub and window rim read as metal; the faces as matte paint; the needle as a flat lacquered blade.

**Premortem:** "the speed in the left tube reads before the gear, and the rims look like aftermarket chrome pods rather than a 911's dark instrument rings." Both became fixes (the larger gear window, the graphite bezels).

Rejection patterns against the final crops (`02_mid_cruise_crop.png`, `03_redline_strobe_crop.png`, `review/z_02_window_hub_x3.png`): (a) no: the cold reviewer named a Porsche 911 again; (b) no: bezels, chamfers, hub, window rim and dial paint read at 1:1 without zooming; (c) no: five separate tubes, no panel; (d) no: rendered hardware under computed glass, with the plumb-line shift ring and the turning scale; (e) no: the crystals and the cyclops stay clear, the cyclops's glint now under the gear's ink.

### The cold reviewer, closing round (one `ui-ux-designer` run, 19:18 to 19:26 UTC; packet `mockups/blind/`: A.png, glance_01–08 at seed 8, R1–R3)

- Five-second test: "Noticed first: the centre rev counter (its size, white numerals and orange needle). Second: the '118' speed. Third: the '4' in its dome."
- Recognition: "Porsche 911. The clues are five overlapping round dials with the rev counter in the centre and the '1/min × 1000' legend."
- Blind read (seed 8, key `mockups/PorscheAnalog/review/blind_key_8.txt`), reader's answer beside the key:

| Frame | Reader | Key |
|---|---|---|
| glance_01 (seq 06) | N / 0 mph / off | N / 0.0 mph / 900 rpm, off |
| glance_02 (seq 46) | 4 / 93 / filling | 4 / 93.1 / 6,434 rpm, first stage |
| glance_03 (seq 00) | N / 0 / off | N / 0.0 / 900 rpm, off |
| glance_04 (seq 47) | 4 / 93 / filling | 4 / 93.6 / 6,472 rpm, first stage |
| glance_05 (seq 20) | 3 / 67 / off | 3 / 67.1 / 5,880 rpm, off |
| glance_06 (seq 25) | 3 / 86 / filling | 3 / 85.8 / 7,985 rpm, filling |
| glance_07 (seq 15) | 3 / 47 / off | 3 / 47.3 / 3,642 rpm, off |
| glance_08 (seq 11) | 3 / 38 / off | 3 / 38.4 / 2,649 rpm, off |

Its findings and what happened to each are under "Review findings".

### Checklist

| Line | Answer | Evidence |
|---|---|---|
| V1 | yes | closing expectations 18:46 UTC, before the final render existed |
| V2 | yes | closing cold look 19:20 UTC (not blind, said above); patterns (a) to (e) on named crops above |
| V3 | yes | one closing run; each finding with its outcome under "Review findings" |
| V4 | **no** (cannot be fixed afterwards) | the reviewer noticed the rev counter, then the speed, then the gear third. What changed: the gear window grew (porthole R 53 → 70, gear cap 74 → 96, about 101 through the low magnifier), the "+" moved inside it, the cyclops's glint was dimmed below the gear's ink, and Neutral draws at full ink; the gear is now the largest figure in the rev counter (`review/z_02_window_hub_x3.png`) |
| V5 | yes | the blind read above: gear and shift state right on every frame, speed within 1 mph on every frame |
| V6 | **no** for the small labels (rule question); yes for everything else | final `glance` over the cockpit composites, zones with their surround: gear 9.5 / 12.9 / 11.7 / 11.9 :1; speed 9.1 / 9.6 / 9.6 / 9.6 :1; HP 4.5 / 5.5 / 5.1 / 5.2; PSI 5.1 / 5.6 / 5.4 / 4.8; G value 5.8 / 5.7 / 4.6 (01 shows the greeting); needle 3.2:1; lit crest lens 11.1:1; lit green lens 7.8:1; pedal arcs 3.2 and 3.5:1 (04), 3.5:1 (02); G dot 3.3 and 3.5:1 (graphics need 3). The static unit labels (MPH, HP, PSI, G, BRK, THR, "1/min × 1000", 12 to 16 px) read 7 to 8:1 unblurred but fall under 4.5:1 in the 960×540 blur: the same conflict the pilot reported between V6 and the 12 px label floor, reported to the coordinator as a rule question |
| V7 | yes | `review/<state>_cockpit_squint.png` and `review/final_shift_point.png`: dark, staged (green, amber, red), full white with the crest strobing, and its off phase each differ from idle |
| V8 | **no** for the "+" cue; yes for everything else | `Geom`: speed cap 66, gear cap 96 (cockpit ≥ 40; "10" at 73); the readouts cap 26 (`G_VALUE`, `FAR_VALUE` in `tubes.rs`) and 46 (side-tube power and boost), every dynamic readout ≥ 17.7 px; the shift signal is the 11-lens ring (lenses 36 × 12 px across 445 px); every stroke ≥ 1 px at full contrast (1:1 crops). The "+" upshift cue is cap 24: a 40 px "+" does not fit beside the gear inside the 70 px porthole, and outside it the reviewers read it as floating, so it stays as the Carrera's small secondary cue |
| V9 | yes | `flash --frames 30-41`: one flashing area, the crest strobe; no digit and no needle flashes (`review/flash_map_30-41.png`) |
| V10 | yes | glance to squint: labels and bezels fade first, the gear, the lit lenses and the speed last, in all four states |
| V11 | yes | `static`: p95 at or below 0.2 and dimmer than the live data (`review/01_idle_static.png`) |
| V12 | yes | red only on the redline band, the red ticks and the lenses' last stage; green only on the lenses' first stage (the pedal arcs now use white ink); `cvd`: every state reads by count and position (`review/02_mid_cruise_cockpit_cvd_sheet.png`) |
| V13 | yes | "Porsche 911", quoted above (both rounds) |
| V14 | yes | the inventory measured: 11 bars on the 10,000 face, sweep 240°, shift rpm at 270° (12 o'clock), red from the limiter, needle within 1° of the data (`review/tach_unwrap_final.png`); lines 1, 7, 10 and 12 logged as changes; 11 lenses counted |
| V15 | **no** for R2, deliberate | blurred, the render and R1 / R3 show the same five-tube arrangement in both orders; against R2 (the 992.1 rev counter alone) the masses differ, because the track face and the window in the dead zone are deliberate changes (`RESEARCH.md` changes B and the track face) |
| V16 | yes | `review/final_board_vs_render.png`: every board zone in place at its size; the render is richer than the board and the before |
| V17 | yes | at full frame the bezels read as dark machined metal with polished chamfers, the hub as a black satin cap, the needles as lacquer, the faces as matte paint, the crystals as glass; every highlight follows the one key light |
| V18 | yes | materials checks that apply (metal bezels, paint, lacquer, glass; no leather, carbon or enamel here): one key light, highlights move with the light swing between 02 and 04, no material only at zoom, and no glass piece stacked on another (the cyclops is set into the crystal, V19) |
| V19 | yes | crystals and cyclops clear over black, crisp rims, highlights move between 02 and 04 (`review/light_swing.png`), the magnifier enlarges the gear about 5 % (test `the_magnifier_shows_the_gear_larger_and_unbroken`). No glass sits on glass: the cyclops is set into the rev counter's crystal (one glass layer, the lens in its own footprint, both refracting the dial), and the side crystals sit under the rev counter's bezel |
| V20 | yes | `stretch` (`review/02_mid_cruise_stretch8.png`, `03_…`): only intended elements and their glow; corners off |
| V21 | yes | discrete signals change on their data frame (the stages at 26, the full white and "+" at 27 when the rpm passes 8,439, the gear at 42); the needle trails the data by under a frame, and after the upshift it comes to rest on the rpm without passing it (`review/final_after_upshift.png`: no feed-forward for 0.15 s after a gear change, and no swing past the rpm while it chases the step); `motion`: no stall in the pulls. The idle line is NOT ASSESSED: the sequence's idle frames hold the start-up materialise (section 8) |
| V22 | yes | `flash --frames 30-41`: "flash audit of what this sequence can show: yes", 3 on / 3 off, 36 px tall (limit 42), only the shift strip flashes |
| V23 | yes | frames 41 to 48 (`review/final_upshift.png`, `review/final_gear_entry.png`): the new gear from the first frame, never blank or doubled, settled by frame 46; the ring and the "+" are dark at 42, and from 43, once the rpm is under the shift point, the ring shows its stage colours again (the "+" stays dark, as the rpm stays under the shift point) |
| V24 | yes | the `motion` spike at 27 is the shift point (full white, "+", strobe on); 30, 33, 36 and 39 are strobe edges; 42 is the gear change; from 43 the needle's moves follow the data |
| V25 | yes | before and after (`docs/handover-shots/cluster-porsche_analog-before_02_mid_cruise_cockpit.png`, `…-after_02_mid_cruise_cockpit.png`): the before's flat vector dial with a jewel visor is gone; nothing it did better is lost |
| V26 | yes | each new golden opened and compared with the previous ones (`visual-review diff`): every change is the window, the bezels, the logo, the tubes' tuck and the pedal arcs |

### What the review set could not show (fh6-visual-review section 8)
- Glass bending the real game frame: NOT ASSESSED (the composite lays the cluster over the photo with plain alpha).
- Hood legibility: not applicable (the catalog lists cockpit only).
- Whether the idle is still: NOT ASSESSED (the sequence starts fresh).
- The light swing in the sequence: judged on `02` against `04` instead.
- Five-second-plus flashing, reduced-flash mode, the show transition and panel brightness: NOT ASSESSED (not rendered).

## Before and after pair
- Before: [cluster-porsche_analog-before_02_mid_cruise_cockpit.png](../../../../../docs/handover-shots/cluster-porsche_analog-before_02_mid_cruise_cockpit.png) (the dash on `main` before this thread, a flat vector dial with a jewel visor).
- After: [cluster-porsche_analog-after_02_mid_cruise_cockpit.png](../../../../../docs/handover-shots/cluster-porsche_analog-after_02_mid_cruise_cockpit.png).
- Crops: `cluster-porsche_analog-01_idle.png` to `-04_drift_braking.png`, `-shift_point.png`, `-upshift.png`.

## Assets and masters
Runtime files (31, in git) with recipe, seed and licence: [assets/clusters/porsche_analog/ASSETS.md](../../../../../assets/clusters/porsche_analog/ASSETS.md). The ComfyUI dial paint (SDXL, seed 70701, made neutral and evened out locally) is rendered into every dial face; the Blender tubes, hub, window rim (three key azimuths each), needle and pointer (eight azimuths each) are drawn at full strength.

Masters in the Drive folder `FH6 WheelDash v2 - 4K Masters/porsche_analog/`: `comfyui/hero_porsche_analog_dial_grain_seam_00001_.png` and `_00002_.png`; `textures_porsche_analog_dial_grain_preview.png` and the neutral grain previews; one folder per Blender job (`hero_porsche_analog_tach`, `_side`, `_far`, `_hub`, `_window`, `_needle`, `_pointer`), each with `beauty_<k>.png`, `normal.png`, `height.png`, `depth.png`, `data.exr` (and `ao.png`, `ao.exr` for the tubes); a `*_preview.png` beside each for the tracker.

Cloud GPU: 4 sessions on the Runpod 4090 (builds 3 to 6), 29 min, $0.431 (`asset-forge cloud spend`); each ended with `cloud down`.

## Changes from the real display and from the brief
From `RESEARCH.md`, "Deliberate changes and why":
- A. The outer and right tubes show power, boost and G (the telemetry has no volts, oil or coolant).
- B. The gear is the hero, in a larger window under a cyclops.
- C. The 2027 step: rendered hardware, physically based glass, the scale that turns for each car.
- D. No speed-limit sign, telltales or fuel (no data).
- E. The brief's two-rate scale is replaced by Porsche's linear rotated scale.
- F. The brief's jewel visor is replaced by the GT3's shift bars in the bezel, 11 lenses round 12.
- G. The scale's top is the first whole thousand at least 500 rpm past the limiter.
- H. Small readouts refresh like a car display; the needle holds a limiter bounce; the power and boost scales re-print only off the throttle.
- I. The logo sits below the hub on the track face for the common limiters, clear of the needle at the shift point; where the window sits low it goes above, where the 992.2 has its badge.
- J. The outer tubes keep their size (0.44 of the rev counter), to fit the envelope.
- K. Graphite bezels and a black satin hub (as the 992.1); needles in the palette's primary (white with the Heritage numerals).
- L. Sizes against the tokens: gear cap 96 rather than 150, speed in its own tube about 510 px from the gear; the G parallax replaced by the light swing.
- M. One grotesque for every figure: the gear and speed in Barlow Semi Condensed like the dial, not the display role's Saira Condensed.

## Shared places touched
- `tools/asset-forge/src/main.rs` (the set name), `tools/asset-forge/src/sets/mod.rs` (the set's dispatch) and `tools/asset-forge/src/recipes/mod.rs` (`pub mod porsche_analog;`): one line each.
- `program/phases/07-cluster-factory/CATALOG.md`: this dash's two rows only (✅ and the Penpot IDs).
- `docs/handover-shots/cluster-porsche_analog-*.png`.
- Nothing in `fh6-kit`, `common/`, the hero library, `docs/specs/optics.md`, `docs/PROGRESS.md` or the shared ledgers (`assets/ASSETS.md`, `assets/assets.json`, `assets/forge.lock.json`, `assets/contact-sheet.html`), which every build rewrites and this thread restored each time.

## Requests for shared code
1. **Asset ledger:** the shared ledger's scan lists a cluster's own set (`assets/clusters/<snake>/*.png`) under "Files not made by asset-forge" as "v1 project texture"; it should read the cluster's own `ASSETS.md`.
2. **Hero materials:** a dielectric or leather material cannot set its specular level or IOR, so a black paint's reflectance can only be tamed by dimming the whole rig. A `spec` (or `ior`) key on those materials would let a dial be lit at full power.
3. **Hero textures:** `make_texture` checks the seam but not the content. A recipe whose prompt names an object (a dial) gets that object drawn into the texture. A cheap guard (an edge or text detector) would have caught build 1.
4. **visual-review `blind --mask`:** the mask is a solid black box, which the cold reviewer reported as a defect in both rounds; a blurred or local-colour fill would mask the logo without adding a finding.
5. **visual-review `glance` zones:** a zone drawn tight round a glyph measures the glyph's own blurred halo as "dark" (01's N read 6.8:1 in a tight box, 9.5:1 with its surround). The skill could say that a zone includes the element's surround.
6. **Live bench:** the scaffold baseline and the dash run minutes apart on a shared PC. An interleaved A/B mode (alternating dashes within one run) would give a steadier difference.
7. **`HeroGear::snap`:** this dash was built while the pilot's `snap` was not on `main`, so it settles the glyph's entry motion in 30 small steps before a snapped frame (`GEAR_SETTLE_STEPS`). `snap` landed with the pilot's close (`a40b019`) just before this merge; a small follow-up can replace the loop with it.
8. **Setup checklist:** the cockpit composite shows the game's own speed and gear gauge at the lower right; the configurator's checklist could tell the driver to hide Forza's HUD gauges (HMI reviewer's note).
9. **The kit's file cache in `Images::load_file`:** done, it landed with the pilot's close (`a40b019`). This dash loads 31 sprites in `new()`, and the app builds the cluster again on every view change, so the cache keeps those changes from decoding and uploading them again.
10. **`common::ShiftLights` release:** the level decays towards 0 by a fraction each frame and never reaches it (at 58 Hz or more it sticks at the smallest subnormal), so the outermost pods stay lit and `animating()` stays true after the rpm falls under the first stage. Snapping the level to the target once it is within a small step would fix every dash that uses it; this dash works round it.
11. **`fh6_kit::logo` wordmark floor:** the wordmark shrinks to fit its zone with no lower limit, so long brand names drop under the 12 px floor in narrow zones (here 160 × 20). A floor with a narrower fallback face, or eliding, belongs in the kit.
12. **A two-image mix paint in the kit:** cross-fading two renders of a sprite with normal blending over-counts the coverage of their anti-aliased edges, so needle edges pop when the render pair changes. A paint that mixes two images before compositing would make the light swing exact.
13. **A CPU `Setup` for cluster tests:** `Cluster::new` needs a wgpu device, so `update()`'s rules (the strobe clock, the upshift quiet, the ring's hysteresis, the scale tops) can only be tested through renders. A `Setup` that loads no GPU resources would let every dash unit-test its state machine.

## What sibling dashes must know
- **SDXL draws what the prompt names.** Name the material, never the object it belongs to, and list the object in the negative prompt.
- **Black paint under the hero rig renders grey** (about 5 % luminance) because the hot key's off-specular lobe alone gives a black dielectric about 2 %. Dim the rig for parts in shade (the binnacle), and keep needles and other lit parts on the full rig.
- **A polished dome in a top-down ortho render mirrors only the square sky softbox** and reads as a dark square; satin silver reads as a ball bearing. A satin black-chrome cap (roughness 0.3) under the full rig reads as a hub.
- **Light metal bezels read as aftermarket chrome** next to a real 911; a dark graphite top (albedo about 0.12, roughness 0.24) with polished chamfers reads as OEM.
- **Needles turn under a fixed light**: render them at eight azimuths and pick the render whose light matches the needle's angle against the virtual light (`hardware.rs`, `turned`).
- **Numerals by their own width**: place each numeral so its box stays a fixed gap inside the ticks (`tucked` in `mod.rs`).
- **Strobe flash audit**: lit lenses that feed bloom spread the strobe past the 42 px strip; a tight per-lens glow without bloom keeps it inside. Run the strobe on its own clock from the frame the shift point is reached, so its first frame is lit.
- **Limiter bounce**: the review sequence alternates 8,690 and 8,640 rpm every frame; a needle with feed-forward buzzes red at 30 Hz unless it holds the bounce, and the hold must end at once when the rpm leaves the band.
- **A cluster is rebuilt on every view change**: take the car's state as it is on the first frame (needle, gear, greeting, scale), never animate into it; and don't fade printed parts with the materialise ramp (it ramps lensing only).
- **The upshift "quiet"**: after an upshift, keep the shift ring and "+" dark only until the rpm first falls below the shift point; a time window alone also darkens a quick pull from neutral.
- **Learn the scale peaks only in a race**: the OSD's dyno frames carry the last car's ordinal and a dyno's power.
- **Penpot's plugin tab sleeps** within a minute or two in the background; split board builds into small calls (`buildA`, `place8` in chunks, `finishB`), expect `finishB` to time out but complete, and re-place large glyphs after the board's fonts load (a text placed before its font loaded lands off by its own size).

## Review findings
Each finding source ran once at the closing round (findings only, no scores); the outcome is what was done about it. "Build 6" is the last cloud build (graphite bezels, black hub).

### Cold reviewer (closing round)
| # | Finding | Outcome |
|---|---|---|
| 1 | A black bar on the face | Not a defect: the packet's logo mask (request 4) |
| 2 | Scale starts lower right; 8 at 12 o'clock; the idle needle covers the "1" | Not fixed, deliberate: the track face (`PorscheAnalogScale` = `Classic` gives the 992.1 face) |
| 3 | Unlit lenses read as a notched chrome ring; the red zone is not centred on the light row | Partly: the graphite bezel makes the unlit lenses read as smoked lenses in a dark ring; the light row centres on the shift point and the red band starts at the limiter, by design |
| 4 | The gear dome reads as a plastic button; the gear no larger than the speed; a stub of needle tail | Fixed: larger window and gear, the "+" inside it, the glint dimmed; the tail is the needle's own (as on the 992) |
| 5 | Mirror-chrome rims look like aftermarket pods | Fixed: graphite-titanium bezels (build 6) |
| 6 | Grainy faces; flat orange needles on a chrome ball | Partly: black satin hub with the needle's light round it; the paint's fine grain stays (it is the dial's surface) |
| 7 | Outer tubes half hidden; small end values; "−15" reads "−18" | Fixed: each end value sits past its end of the arc by half its own width and 6 px, with a true minus (test `end_values_clear_the_end_ticks_and_the_resting_pointer`); the side tubes moved in, so more of the outer tubes shows |
| 8 | The G meter reads as an app widget; BRK and THR far from the arcs | Partly: pedal arcs in the dial's ink at rank-4 strength; the plot stays a plot (the 992's Car & Info G view is one) |
| 9 | Labels 11 to 14 px | Not fixed: every label at the tokens' 12 px floor (V6 rule question) |
| 10 | The speed pointer is a short tick; speed and gear on different dials | Not fixed, deliberate (change B; the 992's own layout) |
| 11 | A soft black box over the composite's edges | Not this dash's: the shared OEM scrim of the composite |
| 12 | The first stage is two small dots far apart | Not fixed, deliberate: the lenses fill from both ends to 12, as the GT3's bars do |
| 13 | The light threshold looks inconsistent (glance 05 against 02) | Not a defect: 5,880 rpm is under the first stage (6,329 rpm), 6,434 over it |
| 14 | glance_03 dimmed grey; 0.12 G at a standstill | Fixed: printed parts no longer fade with the materialise ramp; the 0.12 G is the review telemetry |
| 15 | "HELLO, TJX" while moving at 38 mph | Fixed: the greeting goes on the first moving frame (`review/final_greeting.png`) |
| 16 | The same readouts at different speeds | Not a defect: the 5 Hz refresh (change H) |
| 17 | Racing readouts where the 911 has volts, oil and water | Not fixed, deliberate (change A) |

### HMI and glance (fan-out viewpoint)
| # | Finding | Outcome |
|---|---|---|
| 1 | The bright cue comes a frame after the needle stands at 12 | Fixed: the strobe runs on its own clock, lit on its first frame; full white and "+" at frame 27, the first frame over 8,439 rpm (`review/final_shift_point.png`) |
| 2 | Frame 42 shows the new gear under white lenses and "+" | Fixed: the ring and "+" stay dark after an upshift until the rpm falls below the shift point, for at most 0.3 s (`review/final_upshift.png`) |
| 3 | The G tube's arcs reuse the shift colours at full strength | Fixed: the arcs use the dial's white ink at 0.62, 6 px wide |
| 4 | The cyclops streak and the hub outshine the gear | Fixed: the magnifier's light at 0.45; black satin hub (build 6) |
| 5 | Idle N is the faintest large figure | Fixed: N at full ink; the greeting's name at 0.8 |
| 6 | The strobe opens on its dark phase | Fixed: its own clock, lit first; the 10 Hz rate is the tokens' (`motion.limiterStrobe`) |
| 7 | The speed outweighs the gear | Fixed: gear cap 74 → 96 under the magnifier |
| 8 | The power tube re-ranges mid-pull and its pointer falls | Fixed: the tops change only off the throttle |
| 9 | End ticks run into their numerals | Fixed: end values past the arc ends by half their width and 6 px (the first fix left 3-cell values on the ticks; the final check caught it) |
| 10 | Labels below 16 to 20 arcmin | Not fixed: the tokens' 12 px floor (rule question) |
| 11 | The needle crosses the wordmark at the shift point | Fixed: the logo below the hub (change I) |
| 12 | Green lenses glow while braking | Not fixed: the lenses show rpm, as the real shift lights do |
| 13 | The cluster sits low in the dome | Not this dash's: the anchor is set in AGENTS.md |
| 14 | The game's own gauge in the composite | Request 8 |

### Code QA (fan-out viewpoint)
| # | Finding | Outcome |
|---|---|---|
| 1 | A rebuild mid-race replays a gear change and a needle sweep | Fixed: `fresh` first frame (needle, pointers, G, greeting reset; gear settled before the first draw) |
| 2 | The start-up sweep replays after the OSD | Fixed: the sweep is removed |
| 3 | Dyno frames teach the power peak and turn the scale | Fixed: peaks only in a race; a race-flag edge snaps the scale and the whole state, and resets the power and boost smoothers so nothing of the dyno's power reaches the peak |
| 4 | The strobe phased on time since build | Fixed: its own clock |
| 5 | Outer G and speed values skip the readouts | Fixed: `g_digits` and the tabular speed |
| 6 | The G dot clamped per axis | Fixed: clamped by length |
| 7 | `ShowLogo: false` still draws the wordmark; the greeting loses the name | Fixed: nothing when the logo is off; the greeting reads the config's driver name |
| 8 | Clearing a user font keeps the role's font | Fixed: `own_faces` come back |
| 9 | The materialise ramp fades the scale, gear and digits | Fixed: printed parts and data at full strength; only the glass materialises |
| 10 | No NaN guard on the ring's rpm | Fixed: the ring is fed the sanitised rpm |
| 11 | The greeting has no length limit | Fixed: shrinks to fit and clips to the tube's opening |
| 12 | `clone_from` on the logo allocates | Kept: only when the logo changes, never per frame (line 4) |
| 13 | Gear and speed caps below the tokens' | Logged: change L |
| 14 | Gear to speed 508 px; lens span 445 px | Logged: change L (the 992 layout) |
| 15 | The zone table's "+" cap | Fixed: the table no longer states it |
| 16 | `LimiterCue` drawn nowhere | Fixed: removed |
| 17 | km/h power end not a round kW | Fixed: tops in whole 50 kW and 0.5 bar steps |
| 18 | The logo flips sides mid-turn | Fixed: its side follows the car's scale, not the turning one |
| 19 | `fixed()` decimals past 3 | Fixed: clamped once |
| 20 | The speed field re-centres at 10 and 100 mph | Kept: centred, as the 992's own readout |
| 21 | Very high limiters lose the red band | Kept: TOP_MAX 20,000; no Forza car reaches it |
| 22 | The hold outlives a lift out of the band | Fixed: the hold ends when the rpm leaves the band (test `a_lift_out_of_the_band_ends_the_hold_at_once`) |
| 23 | The hold assumes a bounce within 200 rpm | Kept: needs a real-game capture (noted for the real-game run) |
| 24 | Hood sizes below 12 px | Not applicable: the catalog lists cockpit only |
| 25 | A zero limiter paints the scale red | Fixed: no red band without a limiter |
| 26 | No test drives `update()` | Partly: the pure parts are unit-tested; `update()` needs a GPU `Setup`, so the goldens and the sequence review cover it |
| 27 | No frame-budget numbers | Fixed: "Verification output" and "Live bench" |
| 28 | The strobe-band test ignores the glow | Fixed: the test adds the glow's reach (7 px a side, which the flash audit's 36 px band matches) and pins the glow's sigma and intensity |
| 29 | Classic face: lenses beside 2 and 3 | Kept: the lenses are bezel hardware; the Classic face is an option |
| 30 | The ring waits for the needle | Kept: the plumb line; feed-forward keeps the lag under a frame |
| 31 | The tokens' parallax dropped | Logged: change L |
| 32 | Recipe and sprite sizes kept in step by hand | Kept: header sizes checked by eye (512, 384, 232, 48, 164, 24×236, 16×38) |
| 33 | `build()` does not check job outputs | Request for the hero library (noted) |
| 34 | The neutral grain's cache ignored spread and window | Fixed: both are in the fingerprint |
| 35 | Static review only | Fixed: this file's verification output |

### Reference fidelity (fan-out viewpoint)
| # | Finding | Outcome |
|---|---|---|
| 1 | The side tubes only touch the rev counter | Fixed: 36 px behind its bezel (a tenth of each tube), with contact shadows |
| 2 | The outer tubes are small | Logged: change J |
| 3 | The red zone is a continuous band | Kept: the band reads at a glance (DC finding 6); the red ticks stay too |
| 4 | The bezels are the brightest ring | Fixed: graphite bezels (build 6) |
| 5 | No dark tick track | Kept: the dial's ring structure is the 992.1's raised scale ring |
| 6 | Orange needles | Logged: change K (palette primary; white with Heritage) |
| 7 | Numerals taller and narrower than Porsche's | Kept: Barlow Semi Condensed is the closest open face (feature 15) |
| 8 | The wordmark where the GT3 shows its model | Kept: the telemetry carries no model name; the logo zone shows the maker or the custom name |
| 9 | The speed tube copies the 992.1 | Kept: the rank-2 digits need the centre |
| 10 | The power scale re-labels mid-pull | Fixed |
| 11 | The references are squeezed 1.67:1 | Noted: measurements taken along one axis |
| 12 | No reference shows the track mode, the cyclops or bezel lights | Noted in `RESEARCH.md` "Gaps" |

### Classic Analog director (fan-out viewpoint)
| # | Finding | Outcome |
|---|---|---|
| 1 | The cyclops glint outshines the digit | Fixed |
| 2 | Tubes touch; no contact shadow | Fixed |
| 3 | Bezels read as grey plastic with white pinstripes | Fixed in part: graphite spun tops (build 6); the chamfers stay polished |
| 4 | Chrome-ball hub that never lights | Fixed: black satin cap with the needle's light round it |
| 5 | The power scale changes mid-pull | Fixed |
| 6 | The speed pointer's root reaches the numerals | Kept: a pointer crosses its numerals, as on the real dial; the digits carry the reading |
| 7 | End labels touch ticks | Fixed (in the final round: end values placed by their own width) |
| 8 | The speed outweighs the gear | Fixed |
| 9 | Lenses read as flat blocks; the limiter fuses them into one arc | Kept: the steady white flanks are the limiter's signature (change F) |
| 10 | The needle is one flat orange | Kept: the rendered lacquer's ridge catches the light near its rest azimuth |
| 11 | The dial paint is two flat greys with a hard step | Kept: the raised scale ring's edge |
| 12 | The tabular "1" leaves gaps ("1 1 8") | Kept: the tabular rule for rank-2 speed |
| 13 | The brightest pixels are glints | Fixed in part: cyclops dimmed, hub black; chamfer glints still clip (mirror reflections of the key) |
| 14 | The G tube is app graphics; colours repeat | Fixed in part: arcs in white ink |
| 15 | The legend runs into the porthole | Fixed: the logo and legend below the hub, clear of the window (test) |
| 16 | The "+" floats outside the window | Fixed: inside the window |
| 17 | The wordmark under the needle at the shift | Fixed |
| 18 | N dimmer than "0" and "TJX" | Fixed |
| 19 | A faint hairline outside every bezel | Kept: the stretched view (`review/z_stretch_left.png`) shows it is the bezel's own polished black-chrome outer chamfer catching the key, parted from the spun top by the edge between them, at levels 10 to 18 of 255: part of the rendered bezel, not a separate element |
| 20 | Low in the dome; the boost bezel meets the dashboard | Not this dash's: the anchor |

### `design:design-critique` (closing round)
| Finding | Outcome |
|---|---|
| The five overlapping tubes read as the 992.2; rims, hub and lenses read as hardware | Kept |
| The speed draws the eye almost as much as the gear | Fixed: the larger gear window |
| BRK and THR sit at the arcs' top ends, away from where they start filling | Kept: the labels name the arcs at their ends, clear of the values |
| The tach numerals hold back behind the gear; type and colour consistent | Kept |
| The needle reads flat at 1:1 | Kept: the rendered ridge (see the director's 10) |

### `design:accessibility-review` (closing round; contrast, colour and flashing only)
| Finding | Outcome |
|---|---|
| Gear and speed pass 1.4.3 with a wide margin | Kept |
| Small readouts and unit labels under 4.5:1 in the blur gate (7:1 or more unblurred) | Fixed for the readouts (heavier, larger; now 4.5 to 5.8:1); labels reported as the rule question |
| Colour-coded states also read by count and position (1.4.1) | Kept |
| The strobe stays under the 2.3.1 limits; reduced-flash mode not rendered | Kept; reduced-flash NOT ASSESSED |

### `rust-review` (with `rust-skills`)
| Finding | Outcome |
|---|---|
| A limiter of 0 made every rpm "near" the limiter | Fixed: a limiter of 200 rpm or less counts as none |
| Pedantic casts on bounded values | Kept: every cast is on a clamped value |
| `DialScale` passed by reference | Kept: it is small and `Copy`; either way works |
| The options re-read from the config every frame | Kept: a few map lookups, no allocation; it gives hot reload |

### Final independent check (after the closing fixes)
Before the merge, four fresh reviewers read the final module and this file through four lenses (the update state machine, the draw path, the handover's claims, the tests), and one skeptic per finding tried to refute it (43 agents, 20:42 to 21:22 UTC, read-only). 39 findings, 37 survived, 2 were refuted. Each surviving one and what was done:

| # | Finding | Outcome |
|---|---|---|
| 1 | The shared ring's release never reaches zero, so the two end lenses linger lit and `animating()` never settles | Fixed here: the lenses go dark at once below the first stage (`ring_lit`), and the ring counts as animating only while lit; the shared fix is request 10 |
| 2 | Closing the OSD replays a gear change from the dyno's gear (rim flash, slide, the upshift quiet over the shift point), and the needle, pointer, G dot and greeting spring from the dyno's values | Fixed: a race-flag change snaps the whole state, as the first frame does |
| 3 | A car change or the OSD close lets the smoothed power of the last source into the new peak | Fixed: the power and boost smoothers and the readouts restart from the current sample on a snap or a car change |
| 4 | No hysteresis at the shift point: an rpm hovering there flashes the whole ring | Fixed: 120 rpm off-hysteresis (the tokens' value) |
| 5 | The power and boost tops stay at the floor through the first pull after a build | Kept: the gauge pegs at its end on that pull and the scale is printed at the first lift (re-ranging mid-pull is what the reviewers rejected); recorded in the code |
| 6 | A unit change plots the old unit's speed on the new face for a frame | Fixed: the pointer resets on a unit change |
| 7 | With no limiter known the ring reads "over" and strobes at idle | Fixed: the ring needs a known limiter |
| 8 | The upshift frame keeps the needle held on the limiter (a 90 rpm drop looks like a bounce) | Fixed: a gear change releases the hold (test `a_gear_change_ends_the_hold_at_once`) |
| 9 | "Shown again after a menu or a pause" cannot happen: the host skips updates while hidden | Fixed: the comments say what happens; a frame after a hidden gap (`dt` ≥ 0.09 s) snaps the state |
| 10 | Gear "10" does not fit the magnifier's view and meets the "+" | Fixed: two-character gears at 0.76 of the cap (tests for both) |
| 11 | 3-cell end values still sit on the end ticks and the resting pointer | Fixed: placed past the end by half their width and 6 px; wide centre values fit their visible span (tests) |
| 12 | Long brand names in the logo zone shrink under the 12 px floor | Kept: the kit's wordmark has no floor; request 11 |
| 13 | Tach numerals run together on faces to 16,000 or more | Fixed: every other numeral where two-digit ones would meet (test) |
| 14 | Sprite cross-fades double edge coverage, and needle edges pop every 45° | Kept: a few edge pixels, during motion; request 12 (a two-image mix paint in the kit) |
| 15 | The porthole's depth gradient changes no pixel | Fixed: removed |
| 16 | After the upshift the needle dips under the data by about 470 rpm (the feed-forward with a lagging rate) | Fixed: no feed-forward for 0.15 s after a gear change, and in that window the needle comes to rest on the rpm instead of swinging past it (the tokens' spring alone rang 55 rpm past it) |
| 17, 31 | The magnifier is about 1.05×, not 1.15×; the gear is seen at about 101 px | Fixed: the numbers corrected here and in `RESEARCH.md`; the test bounds narrowed to 1.03 to 1.10 |
| 18 | Closing line 39 answered "yes, with the noes" | Fixed: line 39 is a no, with the noes listed and reported |
| 19 | The cyclops was glass on glass | Fixed: set into the crystal in one glass layer |
| 20 | V8 misstated the readout caps and left out the "+" (cap 24) | Fixed: V8 corrected; the "+" is a reported no |
| 21 | V23 said the "+" lights again at frame 43 | Fixed: only the ring relights there |
| 22 | QA 28 marked fixed though its test ignored the glow | Fixed: the test adds the glow's reach |
| 23 | HMI 2 left out the quiet's 0.3 s cap | Fixed |
| 24 | Telemetry listed idle rpm and said everything comes through `Signals` | Fixed |
| 25 | Closing line 20 said the maker name is the only literal | Fixed: the faces and change M recorded |
| 26 | "What was built" put every fallback in `hardware.rs` | Fixed |
| 27 | On the track face the logo goes above the hub for common limiters (the mock's 7,600 among them) | Fixed for the common limiters: a narrower zone a little lower; the needle test now runs on all 72 scales |
| 28 | The bouncing-readout test passed without its hysteresis | Fixed: samples that round to the next digit |
| 29 | The dead-zone test never reached the clamp | Fixed: samples past the top and below zero |
| 30 | The pointer test copied the formula | Fixed: `pointer_rect` is pure and tested against both blades |
| 32 | The window tests sized the gear as one glyph | Fixed (with 10) |
| 33 | `distance_to_rect` had no direct test | Fixed |
| 34 | `Content::parse` untested for most values | Fixed: every value, case and spaces |
| 35 | The sprite frames were never checked against the zone table | Fixed: test `the_sprite_frames_match_the_zone_table` |
| 36 | The rank-1 rules in `update()` have no test | Partly: their pure parts are tested; `update()` itself needs a GPU `Setup` (request 13) |
| 37 | `fixed()` and `tucked()` untested | Fixed for `fixed()` (`Digits` is pure and tested); `tucked()` keeps its Painter (one cell width) |

Refuted: `animating()` never settling on a NaN rpm (every `Signals` producer sanitises it), and the track-scale test's sweep check restating `end()` (other assertions in the same test cover the sweep).
