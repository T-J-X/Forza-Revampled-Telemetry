# Liquid Glass OSD rebuild: closing review (fixed checklist)

Written 2026-11-27 at the start of the rebuild, before the first build, and frozen: lines are answered, never edited. Every line is answered **yes** or **no** with its evidence (a command's real output, a file, a render) in the handover. No scores, no partial marks. A no is fixed and the whole list is walked again; a no about something that already happened (the order of work, a lock) cannot be fixed afterwards and is reported in the handover, in the thread and to the coordinator. Sources: AGENTS.md ("AGENTS n"), SKILLS.md ("SKILLS s.n"), sections A and B of the dash checklist in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` adapted to the OSD, the research ([OSD_RESEARCH.md](../../../docs/design/OSD_RESEARCH.md), "P n") and the design ([OSD.md](../../../docs/design/OSD.md)).

**A. Hard rules**
1. `fh6-osd` reads, binds or requires none of A, B, X, Y (`0xF000`), and no hint names them as an OSD control. [AGENTS 4]
2. No placeholder, no `todo!()`, no stub, no hardcoded value standing in for live state. [AGENTS 2]
3. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. [AGENTS 6]
4. `draw`, `update` and the per-frame host calls do no heap allocation, file I/O, logging or locking after warm-up: a search of the changed code for `format!`, `vec!`, `collect`, `to_string`, `String::from`, `Box::new`, `tracing::`, `.lock()` and `std::fs` finds hits only in one-time setup (`prepare`'s first call, `new`) and in tests. [AGENTS 6, SKILLS 3.8]
5. The whole OSD frame is under 1.5 ms GPU at 3840×2160 in every one of the 11 states, from `preview-export --osd` on the PC's GPU, with the table in `docs/perf/osd.md`. [OSD_DIRECTION, AGENTS 7]
6. With the OSD open, the live bench (`--demo --open-osd`) runs without error and its CPU per frame and receive-to-present rows are recorded; the OSD's own render-thread work (navigate, prepare, anim) is no larger than before the rebuild. [AGENTS 7]
7. The OSD adds no path that draws the dash while the app hides it; only the OSD's live view shows while paused. [AGENTS 9]
8. No non-Rust source file was added; bpy lives as Rust strings in the asset-forge recipe. [AGENTS 11]
9. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path or a personal email, and the research note's links were checked for this. [AGENTS 10]
10. Nothing was copied into the game folder or the mirror. [AGENTS 8]
11. Any new dependency has its line in the `docs/PROGRESS.md` decision log. [AGENTS 6]
12. `cargo check --workspace`, `cargo test -p fh6-osd -p preview-export -p asset-forge`, `cargo clippy -p fh6-osd -p preview-export -p asset-forge --all-targets -- -D warnings` and the `rustfmt --check` of the changed files pass, with their real output in the handover. [AGENTS 3]

**B. Rendering laws**
13. Colour is premultiplied linear end to end; alpha is never multiplied twice; transparent is `(0,0,0,0)`. [SKILLS 3.1]
14. Sizes, radii and stroke widths are clamped to at least 1 px, and animated values that drive geometry (spring edges, scroll) cannot go negative or NaN. [SKILLS 3.2]
15. Every textured look has a procedural fallback: the studio environment and the driving-position layers each have a branch that draws without them, and they load once, never in `update` or `draw`. [SKILLS 3.4]
16. Text is MSDF from the kit, and the OSD draws in the user's font pack roles. [SKILLS 3.5]
17. Off pixels are `#000000` (transparent): there is no opaque grey or navy box, no panel and no full-canvas fill (the dim is the kit underlay). [SKILLS 3.6, rule 7]
18. The OSD never reads the wall clock; time comes from the host's monotonic `now` and the frame's `dt`. [SKILLS 3.7]
19. Colours and type come from tokens and palette roles; no hex colour is hardcoded where a token exists. [SKILLS 2]
20. All glass is `OpticalGlass` following `docs/specs/optics.md`; `fh6-osd` no longer uses the legacy `Glass`. [AGENTS 3A, optics.md]

**C. OSD design**
21. There is no slab: no glass and no fill acts as a background panel behind the rows; the only glass is the tab bar, the hint bar, the focus platter, the keyboard's key cap and the notice pill. [P1, P2]
22. The glass is clear (no frost, no tint of its own), mirrors the shared studio, and materialises by thickness on open, never by opacity. [P5 to P7, AGENTS 3A]
23. No two glass pieces overlap in any state or during any move. [P2, L10]
24. All text inside a glass piece sits under its level face, and no text outside a piece comes within 64 px of its edge (no mirrored ghost text in a rim). [P8, design]
25. The focused row or key is the first thing found in the blur-gate glance of every state, without colour alone. [P3, OSD_DIRECTION]
26. Labels and values are at least 30 px cap, help and status at least 26, keycaps and the gearbox line at least 22, section headings 24 in sentence case. [P12, design]
27. Over the bright game backdrop, every label and value keeps at least 4.5:1 contrast against what is behind it. [accessibility-review, T8]
28. Focus and tab moves stretch and settle without a jump or overshoot past the next row, and with `ReducedFlash` they do not stretch; open materialises within 300 ms and close is one frame. [P4, P5, P11]
29. Nothing in the OSD flashes: the energise light is a spring, and hold-to-repeat cannot make it strobe (the WCAG 2.3.1 audit of a held-adjust sequence passes). [P11, SKILLS flash audit]
30. The live cluster stays the strongest picture on screen in every state; the menu never outshines it. [rule 3, intent]
31. The camera rows show the rendered driving position, with the seat moving with seat height and the wheel with its offset, and the vector schematic as the fallback. [design]

**D. Workflow and assets**
32. The order was followed: `OSD_RESEARCH.md`, then the Penpot page, then ComfyUI, then Blender, then the first build; every later Penpot change was followed by an asset pass or the handover says why none was needed. [AGENTS 14]
33. The Penpot page `09 · OSD` shows all 11 states of the rebuild, and its board IDs are in `docs/design/PENPOT.md` and the handover. [AGENTS 14]
34. At least one bespoke ComfyUI asset and one bespoke Blender asset are visibly used, each with a row in `assets/osd/ASSETS.md` (recipe, seed, licence and master file name). [AGENTS 14]
35. All assets are 4K; the compressed runtime files are in git and the masters are in the Drive folder under `osd\`, listed in the handover. [AGENTS 14]
36. Every GPU job held its lane's lock; ComfyUI and Blender ran on the cloud lane through `asset-forge` (or the handover says why not), `cloud down` ended the batch, and the thread's cloud spend is in the handover. [AGENTS 15]
37. The direction check was posted with the first build's render, and its self-review (decision, findings, what was done) is in the handover. [AGENTS 12]
38. The product page `mockups/osd/index.html` exists and was generated by `preview-export`. [plan]
39. The handover holds findings and yes/no answers only, no scores. [AGENTS 12]
40. Every applicable line of the `fh6-visual-review` checklist (V1 to V26) is answered under `## Visual review` in the handover with its evidence, NOT ASSESSED where that skill's section 8 allows; the cold reviewer ran once with the neutral packet. [AGENTS 12]
41. `design:design-critique`, `design:accessibility-review` and `rust-review` ran on the final build, and each finding is fixed or answered in the handover. [AGENTS 12]
