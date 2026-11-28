---
name: fh6-visual-review
description: How an FH6 WheelDash thread reviews its own renders now that TJX is hands-off (2026-11-27). The fixed yes/no visual checklist, the bias-proof procedure for the direction check, the closing review and every progress screenshot, and the visual-review Rust tool (1:1 tiles and zooms, the blur-gate glance and contrast test, diffs with crop pairs against goldens, before renders and Penpot boards, comparison sheets, the WCAG 2.3.1 flash audit, motion profile, colour-vision views, near-black stretch, static-parts check, blind glance read, dial unwrap, protractor and annotated screenshots). Use whenever a thread reviews, critiques, compares, signs off or posts a render, board or motion sequence of a dash, the OSD or the scrim. Findings and yes/no only, never scores.
---

# FH6 visual review

TJX is hands-off (2026-11-27). Every thread reviews its own renders at the direction check and at the closing review, posts what it found with screenshots, and carries on without waiting for TJX. This skill is how. It checks against `fh6-hmi-design-rules` (load that too) and keeps AGENTS.md rule 12: findings and yes/no answers, no scores. TJX can still step in at any time; TJX's own words then win.

## 1. When

| Moment | Run | Answer |
|---|---|---|
| Direction check (`CLUSTER_PROMPT.md` step 5), on the first real render | steps 1 to 6 of section 5 | the lines marked **DC** in section 6 |
| Closing review (step 10) | all of section 5, ending with the six-lead review panel ([references/panel.md](references/panel.md)) | every line of section 6, then the panel's findings under `## Review panel` |
| A progress post: each milestone, and at least every 20 to 30 minutes of work | `annotate` a crop of what changed (section 7) | none |
| OSD, scrim or phase visuals | section 5 | the lines that apply; mark the rest NOT ASSESSED with the reason |

## 2. Rules

1. **No scores.** No numbers out of N, grades, ratings, percent matches, "N of M passed" tallies or verdict panels. No loop that reruns a reviewer or rewords its prompt until it says yes. Measurements (contrast ratios, delta-E, px²) are evidence against a fixed threshold, never something to push up.
2. **Once per review point.** Each finding source runs once at the direction check and once at the closing review. Fix every no, then walk the whole checklist again: that is the fixed checklist, not a critic loop. After a fix, re-capture and re-check only what the fix touched, then walk the list. V4, V5 and V13 are the cold reviewer's and blind reader's answers from their one run at that review point and are never asked again: a no there is fixed in the dash where it can be (with the tool's evidence of the fix), stays no, and is reported as a line that cannot be fixed afterwards (the closing-review preamble of `CLUSTER_PROMPT.md`).
3. **Look before you know.** Freeze the expectations (step 1) before you open this round's renders, and describe what you see (step 2) before you open your notes, the spec, the board or the references. Expectation fills gaps; it is why people miss their own typos.
4. **Independence.** You built it, and models favour their own work. The independent reviewer is a fresh-context agent: the project agent `cold-reviewer` (`.claude/agents/cold-reviewer.md`, Read only, no verdicts or ratings), started with the Agent tool, that sees only neutral-named, logo-masked images and never your notes. A session started before that agent existed may not list it: then start `general-purpose` with the same brief and the agent file's text at its top. `design:design-critique` runs in your own context, so it is a second opinion, not an independent one.
5. **See pixels 1:1.** Claude's Read shows an image 1:1 only up to 1568 px a side and about 1.15 megapixels (newer models allow more; assume this limit). A 3840x2160 frame arrives at about 0.37x to 0.41x: treat it as the glance view, never as evidence of detail. Judge rims, glyph edges, grain and banding on 1:1 tiles; use 3x zooms only to find a cause. If something reads only at 3x, the answer to "does it read" is no.
6. **Measure what eyes get wrong.** Counts (ticks, LEDs, tows), angles (needles) and colours come from `unwrap`, the protractor and `probe`, not from a downscaled frame.
7. **Ask same/different questions both ways round** (left and right swapped). An answer that flips is no conclusion: settle it with a measurement.
8. **NOT ASSESSED is never yes.** A check that could not run (a missing input, a part the dash does not have) is written NOT ASSESSED with the reason. A DC line NOT ASSESSED for a missing input blocks a "go"; a line for a part the dash does not have (V19 on a dash without glass) is NOT ASSESSED and blocks nothing.
9. **Local only.** Never upload a render or a reference to an online checker, font finder or service.
10. **GPU.** `visual-review` runs on the CPU: no lock. A new render from `preview-export` is a GPU job: hold the lock (root `SKILLS.md` section 5).

## 3. The review set

`cargo run --release -p preview-export -- --cluster <Name>` (a GPU job) writes `mockups/<Name>/`:
- `<state>.png`: the full 3840x2160 frame over black, for `01_idle`, `02_mid_cruise`, `03_redline_strobe` and `04_drift_braking`; `hood_<state>.png` for the hood view.
- `<state>_cockpit.png`: the cockpit view as the driver sees it, over the in-game cockpit photo with the scrim.
- `<state>_crop.png`: the envelope at 1:1 (cockpit 1536x656: read 1:1; hood 1856x496: tile it).
- `sequence/frame_00.png` to `frame_59.png`: one second at 60 fps as envelope crops. Idle 0-9 (the dash starts fresh, so its start-up sweep or settling runs here), pull in 3rd 10-29, limiter hold 30-41 (the strobe), upshift to 4th 42-44, pull in 4th 45-59.
- `index.html`: the product page, with the measured colour badges.

Also: the before render kept at the start (`mockups/_before/<Name>/`); the goldens `tests/golden/images/<Name>[_hood]_<state>.png` (1920x1080 renders cropped to the envelope plus 24 px, 768x329 for the cockpit, so they are compared only with other goldens: a failing test writes `.actual.png` beside them, and `git show HEAD:<path>` gives the committed one); the Penpot board export, saved as a 3840x2160 PNG at `mockups/<Name>/board/<state>.png` (Penpot's own export, which TJX or a session with the Penpot UI makes; the `penpot` MCP's `export_shape` only shows a board in the chat, so without a saved file the board overlay is answered from that view beside the render, and section 8 says so); and the reference photos of the real display from `RESEARCH.md` (look at them in the built-in browser; a copy you keep stays in `mockups/references/<Name>/`, which git ignores). Prefer frontal press images for geometry; owner photos are for materials and behaviour.

The answer key is the review telemetry: the states in `docs/design/TOKENS.md` ("Review states") and the sequence in `fh6_clusters::review::sequence_frame`. `visual-review key` prints it for every frame, and `blind` writes the key of each draw.

## 4. The tool

`cargo run --release -p visual-review -- <command> ...` (CPU only). Outputs go to `mockups/<Name>/review/` (git ignores `mockups/`). `--anchor cockpit|hood|hud` makes every coordinate local to that view's anchor, so you can paste numbers from your `Geom` table; on a `<state>_crop.png` or a sequence frame, 4K coordinates are mapped onto the crop. `--view cockpit|hood|hud` picks the envelope plus 40 px. `--straight` reads a design-tool export's straight alpha. Every command says when an output is too big to be seen 1:1. Worked commands for a dash: [references/commands.md](references/commands.md).

| Command | What it gives | Lines |
|---|---|---|
| `tiles` | 1:1 tiles of the envelope, or `--zoom 3` pixel zooms | detail everywhere |
| `crop` | one region; `--zoom`, `--mirror`, `--mask` (black out the logo zone) | V2, V3 |
| `glance` | the tokens' blur gate (960x540, sigma 1.5) and a squint view, with WCAG contrast per `--zone` against the tokens' minimums | V6, V7, V10 |
| `diff` | delta-E exactly as the golden harness measures it: heat map, changed cells, crop pairs, a 50% blend, side by side; `--fit` scales B to A (give the render as A) | V16, V24 to V26 |
| `sheet` | pictures side by side under labels (`file@cockpit`, `file@X,Y,W,H`); `--blur` for the coarse comparison | V15, V16 |
| `flash` | WCAG 2.3.1 general and red flash audit: rates, on and off phases, flashing areas, the worst 10-degree field, the strip height, and what the sequence is too short to show; `--frames 30-41` for the limiter hold | V9, V22 |
| `motion` | change per frame: stalls in the pulls, spikes to check, whether the idle settles | V21, V24 |
| `cvd` | protanopia, deuteranopia, tritanopia and luminance-only views | V12 |
| `stretch` | the near-black levels (0 to 8 of 255) stretched to full range, and stray light outside the envelope | V20 |
| `static` | the parts that are the same in every state, and how bright they are | V11 |
| `blind` | 8 seeded, neutral-named frames in `mockups/blind/<seed>/` (a new folder each time), the logo zone masked with `--mask`, the key with the rpm zones kept apart | V5 |
| `key` | the review telemetry of all 60 frames and the four states | V5, V21, V23 |
| `unwrap` | a dial ring unrolled into a strip (angle across, radius down) | V14 |
| `annotate` | numbered boxes, rings, arrows, a protractor and a caption; labels go in a legend | V14, posts |
| `probe` | true-black share, luminance spread, the median lit colour and hues of a region | V6, V11, V20 |

## 5. Procedure

Steps 1 to 6 are the direction check; all nine are the closing review. Write as you go into the handover (section 7).

1. **Freeze.** Before opening this round's renders, write the expectations: the rank order; the feature inventory of the real display with a tolerance per line (from `RESEARCH.md`); the key frames (the limiter reached at frame 29 or 30, strobe 30-41, gear change at 42); and TJX's rejection patterns (step 2). Later answers never edit these. If a tolerance must change, log the change and why before re-checking.
2. **Cold look, then the failure first.** Open the full-frame cockpit composite of `02_mid_cruise` (`hood_02_mid_cruise.png` for a hood-only dash, which has no cockpit composite), a mirrored copy (`crop --mirror`) and the luminance-only view (`cvd`). Write 5 to 10 plain sentences: what is there, the order your eye visits it, every value you can read, the colours, and which parts look like lit physical objects and which look flat. No evaluative words. Then assume TJX rejected it and write the likeliest reason, and check each recorded rejection against a named crop: (a) it looks nothing like the real car's display; (b) faint textures on vector shapes, or materials that show only at 2x zoom; (c) one slab with blobs stuck on, or a backing panel the real display does not have; (d) flat modern widgets instead of TJX's "2027 future" bar; (e) murky, tinted or frosted glass on a glass dash.
3. **Cold reviewer, once.** Build the packet and brief the `cold-reviewer` agent exactly as [references/method.md](references/method.md) says: neutral file names, the logo zone masked, the display facts, fixed questions in a fixed order (five-second test, unprompted recognition, the blind glance read, then the references), findings and yes/no only. Record each finding as fixed (with the new evidence) or not fixed (with the reason). Never rerun it or reword its prompt to get a different answer.
4. **Measure.** Run the tool (section 4; worked commands in references/commands.md): `glance` on all four states of every view the catalog lists, `flash --frames 30-41`, `motion`, `static`, `stretch`, `cvd`, `diff` against the before render (and, at the closing review, the goldens: a first build has none yet), and `unwrap` or the protractor for dials and needles.
5. **The real display and the board.** Fill the frozen feature inventory with measured render values next to the reference values. Compare reference and render blurred to the same coarse level first (masses and light), then 1:1 detail pairs, each question asked in both orders. Lay the board export over the render (`sheet`, `diff --fit`): every zone where the board and the `Geom` table put it.
6. **Materials, light and glass.** Walk the checks in [references/materials.md](references/materials.md) that apply to the parts your dash has; drop the others before you start (say which).
7. **Second opinions, once each.** `design:design-critique` (Skill tool) on the full-frame render, the cockpit tile, the glance zones image, the states sheet and the reference sheet, with the context "an instrument cluster for a 55-inch 4K OLED in a car cockpit, recreating <the real display>; judge it against "The bar" of `CLUSTER_PROMPT.md` and <your family spec>; findings only, no score". `design:accessibility-review` limited to contrast (WCAG 1.4.3, 1.4.11), use of colour (1.4.1), flashing (2.3.1, with the reduced-flash mode judged from the code, because it is not rendered) and motion that no data drives; keyboard and screen-reader items do not apply. Then, at the closing review of a dash, **the six-lead review panel** (mandatory, TJS 2026-11-28): six fresh reviewers at once, one per lead of `program/phases/07-cluster-factory/AGENTS.md` (the family's design director, HMI/UX, the research scout, the technical artist, Rust systems and QA), and one checker for their findings, exactly as [references/panel.md](references/panel.md) says; it is the last step before the merge (`CLUSTER_PROMPT.md` line 40), and it may also run at the direction check (pre-approved). Findings feed section 6; they are not verdicts.
8. **Answer section 6.** Each line yes, no or NOT ASSESSED, with its evidence (an image path under `mockups/<Name>/review/` or a command's output line). Fix every no, re-capture, then walk the whole list again.
9. **Record and post** (section 7).

## 6. The checklist

Fixed lines, written V1 to V26 in the handover (the tool table's "Lines" column uses these numbers). **DC** = answered at the direction check too. The source of each line is in brackets; "method N" and "materials" are this skill's references, "tokens" is `data/design_tokens.json`.

**A. Process**
1. **DC** The expectations were frozen before this round's renders were opened. [method 1]
2. **DC** A cold description of the composite, its mirror and its luminance view was written before the notes, spec, board or references were opened, and each of TJX's rejection patterns was checked against a named crop. [method 2, 3]
3. **DC** One fresh-context reviewer got the neutral, logo-masked packet once, and each of its findings is recorded as fixed or not fixed, with evidence. [method 4]

**B. Glance and legibility**
4. **DC** Five-second test: the cold reviewer named the gear or the shift light among the first two things it noticed. [method 5; fh6-hmi-design-rules 3]
5. Blind glance read (`blind`, 8 seeded frames): the reader got the gear and the rpm zone right on every frame (both are in the key), the speed right to within 1 on every frame that shows it, and the shift lights as that frame shows them (dark, partly lit or all lit: a still shows the strobe only in its on or off phase, so check the frame's lights on a crop; the strobe itself is lines 7 and 22). [method 7]
6. **DC** `glance` prints yes for the gear (7:1) and the speed (4.5:1) in all four states, over `<state>_cockpit.png` for cockpit rows and `hood_<state>.png` for hood rows; every other zone that carries information (lit shift lights, the needle or ribbon head, data labels) reaches 4.5:1, or 3:1 for a graphic. [tokens accessibility.blurGate; WCAG 1.4.3, 1.4.11]
7. In the squint view, each shift-light stage and the strobe's on and off frames are plainly different from each other and from idle. [method 10]
8. Every dynamic readout has a cap height of at least 17.7 px (ISO 15008's 20 arcmin at 0.9 m), and the gear and shift symbol at least 40 px in the cockpit view and 36 px in the hood view (readable without looking down); every stroke that carries information is at least 1 px at full contrast. Sizes come from your `Geom` table, confirmed on a 1:1 crop. [standards]
9. No readout flickers on the limiter: the limiter-hold flash map marks no digits (a value that alternates every frame needs display hysteresis, like the speed's 0.6 rule). [tokens motion.speedDisplay]

**C. Hierarchy and colour**
10. **DC** Glance view to squint view, the elements fade in reverse rank order: labels and bezel first, gear and shift light last, in every state and view. [fh6-hmi-design-rules 3; method 8]
11. `static` over the four states and the sequence: the parts that never change are dimmer than the live data, and at most 20% relative luminance (p95); every static area it marks over the cap (it lists all of them first) is named and fixed, or explained in the handover; no large near-white area. [RESEARCH principle 9; method 25]
12. Colour means state: red only at the redline, braking and alarms; the product page's colour badges are within budget; and in every `cvd` panel each colour-coded state still reads by brightness, position or count. [tokens colorRoles.budget, accessibility.colourIndependence]

**D. The real display and the board**
13. **DC** Unprompted and with the logo masked, the cold reviewer named the target maker or display (for a concept dash, the real display it is benchmarked against). Its words are quoted. [method 6]
14. **DC** Every line of the frozen feature inventory has a measured render value beside the reference value (angles from `unwrap` or the protractor, counts from the unrolled strip, colours from `probe`), marked matched or logged as a deliberate change. At the direction check: the layout, geometry and colour lines. [method 12; CLUSTER_PROMPT line 25]
15. **DC** Blurred to the same coarse level, the reference and the render show the same arrangement of masses and light, with the same answer in both left/right orders. [method 13]
16. **DC** On the board-against-render sheet and overlay, every board zone sits in the same place at the same size, and the render looks clearly richer than the board and the before render. [method 14, 15; CLUSTER_PROMPT asset rules]

**E. Materials, light and glass**
17. **DC** Without zooming (full frame and cockpit composite), every physical part's material can be named, and each part shows highlights that fit its shape and one key light. [materials; CLUSTER_PROMPT line 27]
18. Every check in references/materials.md that applies to the dash's parts is yes.
19. **DC** (glass dashes) The glass bends what is behind it, stays clear over black (only thin rim lines, no tint, frost or texture inside), has crisp rims whose width follows the piece's size, moves its highlights between `02_mid_cruise` and `04_drift_braking`, never sits on other glass, and merges without seams. [docs/specs/optics.md; materials glass checks]
20. **DC** `stretch`: every non-black pixel belongs to an intended element or its glow (no rectangle edge, haze or speckle), and on the full frame its count of pixels that are not off outside the envelope plus 40 px is 0 (faint ones included); `probe` reports the frame's corners off; dark gradients show no banding steps. [SKILLS 3.6; method 22, 23; CLUSTER_PROMPT line 24]

**F. Motion**
21. In each sequence frame, discrete signals (shift stages, strobe, gear, digits) change on the frame the data changes; needles and ribbons trail by at most one frame and never overshoot the data; `motion` reports no stall in the pulls. [method 17; AGENTS 7]
22. `flash --frames 30-41` prints yes for the worst 10-degree field and for the strip height; every area of the shift strip's lights (it lists every area) shows one equal whole number of frames on and off, with no range (its "(3 on / 3 off)", where "2-3 on" would mean uneven phases); and every other flashing area is explained: a digit is line 9, anything else is a finding. What it marks NOT ASSESSED goes into section 8's list. [WCAG 2.3.1; tokens motion.limiterStrobe; method 18, 19]
23. Gear change (frames 39 to 50 at 1:1): the new gear is unambiguous from the first frame after the data changes, no frame shows a blank or doubled gear, and any morph ends within 6 frames. [method 20]
24. Every region that changes between consecutive frames is explained by the data (each `motion` spike checked, `diff` of the pair where needed): no shimmer, crawling aliasing, bloom popping or late glyphs. [method 21]

**G. Regressions**
25. Before and after (`diff` with crop pairs): the before's named main failure is gone, and nothing the before did better is lost. [method 26]
26. Every golden PNG was opened and checked before it was accepted, and every difference from the previous goldens is an intended change. [method 27]

## 7. Record, decide and post

- **Direction check.** After the review (not before it), post `DIRECTION CHECK <Name>` in your thread once, with the cockpit render (the 4K hood render for a hood-only dash), the annotated glance view, the board-against-render sheet (without a saved board file: the Penpot page name, the board ID and the `export_shape` view), the reference side by side, two or three lines on the idea and the references it follows, and the DC lines answered. Decide it yourself: **go** (every DC line that applies to the dash is yes); **go with changes** (each no has a fix that keeps the concept; list them, then do them); **no-go**, a redesign and a new check (line 13, 14, 15 or 16 is no for the concept itself, or line 4, 6 or 10 cannot be fixed without changing it). Record the DC answers and the decision under `## Direction check` in the handover, and carry on without waiting. If TJX answers in the thread, quote TJX: TJX's words win.
- **Closing review.** Every line under `## Visual review` in the handover, each yes, no or NOT ASSESSED with its evidence; then `CLUSTER_PROMPT.md`'s closing checklist (its line 39 points here); then the six-lead review panel, recorded under `## Review panel` and posted as `REVIEW PANEL <Name>` before the merge (its line 40). Copy the crops you want kept to `docs/handover-shots/cluster-<snake_name>-*.png`.
- **Progress posts (TJX, 2026-11-27).** At each milestone (research done, board drawn, each asset batch, first build, direction check, each fix round, closing review) and at least every 20 to 30 minutes of work, post two or three lines with a screenshot of what changed: `annotate` a crop with a `--note` caption and numbered `--box` findings, send the PNG with SendUserFile, and attach it to the message. The first-build post comes after step 1 has frozen the expectations: attach the render with a caption only, and keep its findings for the `DIRECTION CHECK` post.

## 8. What the review set cannot show

Mark these NOT ASSESSED with the reason, or capture them separately:
- Glass bending the real game frame: the cockpit composite lays the cluster over the photo with plain alpha, so refraction is judged on the cluster's own layers.
- A fully blind recognition test: the cold reviewer is a subagent, and subagents receive the project's `AGENTS.md`, which describes each family's look (the Liquid Glass family names Apple's). So V4 and V13 are not fully blind: the handover says so, and V13 counts only a real maker or display the reviewer names from the image, not a family style it could have read.
- The board overlay without a saved board file: when no 3840x2160 board PNG could be saved, V16 is answered from `export_shape`'s view of the board beside the render, and the handover says so.
- Hood legibility over a bright scene: `preview-export` saves its renders already over black, so `glance --over` cannot put them over a backdrop yet (it refuses an opaque render). A render that keeps its alpha is a shared-tool request.
- Whether the idle is still: the sequence's idle frames hold the start-up animation, so `motion`'s idle line is evidence only once a settled idle pair exists.
- The light swing in the sequence: lateral G is held at 0.12 g there, so judge the light's swing on `02_mid_cruise` against `04_drift_braking`.
- Slow flashing and WCAG's one-second count: the limiter hold is 12 frames, so `flash` sees cycles of 6 Hz and faster only (21 frames would cover everything above 3 Hz) and cannot count flashes inside a second. A longer hold is a shared-tool request; until then the strobe's rate comes from the tokens and its phases line.
- WCAG 2.2's red-flash rule (a u'v' chromaticity change above 0.2): `flash` applies WCAG 2.0's `(R - G - B) x 320` rule.
- Five-second-plus flashing, reduced-flash mode, the show (materialise) transition and night or day panel brightness: none is rendered.
