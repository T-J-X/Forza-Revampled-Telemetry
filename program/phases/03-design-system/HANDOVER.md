# HANDOVER: Phase 3 · Design system & art direction

**Chat / branch:** `claude/phase-3-design-system-449d2f` (PR T-J-X/Forza-Revampled-Telemetry#1)  **Date:** 2026-11-21  **Result:** ◐ partial — every repo deliverable is done; the Figma file is foundations + frame skeleton only (Starter plan MCP cap)

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence |
|---|---|---|
| `docs/design/FIGMA.md` holds the file key, URL and page map, or records the HTML-board fallback | ✅ | Key `0zOEwnJ8dEj1kQ7QwPGEH0`, URL, 3-page map, foundation and frame node IDs, and the HTML-board fallback with render command ([FIGMA.md](../../../docs/design/FIGMA.md)) |
| `RESEARCH.md`, `TOKENS.md`, `SCRIM_FIT.md`, `CRITIQUE.md` and `families/*.md` exist | ✅ | `docs/design/` (7 family specs + 7 template PNGs) |
| `data/figma_tokens.json` is extended and parses as valid JSON | ✅ | v3.2.0; see Verification output |
| 6 hero concepts × 4 states exported as 4K PNGs in `docs/design/heroes/` | ✅ | **7** × 4 = 28 PNGs, 3840×2160 (one per family; Liquid Glass is the flagship) |
| `CATALOG.md` lists 29 rebuilds + ≥ 15 new concepts, with families, briefs, references, pattern-setters and build order | ✅ | 29 + **19**, ★ per family, build order ([CATALOG.md](../07-cluster-factory/CATALOG.md)) |
| No Rust changes; text files UTF-8 without BOM | ✅ | `git diff main --stat -- '*.rs'` is empty; BOM check below |
| (Step 9) Critique every hero until clearly better than the references; verdicts in CRITIQUE.md | ◐ | 3 rounds, verdicts recorded; final "matches the references" — see Known issues |
| (Step 1/10) Figma file with library, family templates and heroes, node IDs in the handover | ◐ | Foundations + frame skeleton with IDs (FIGMA.md); library and PNG placement blocked (Starter MCP cap; desktop MCP not available) |

## What was built
- `docs/design/RESEARCH.md` — six-domain reference research → 10 testable principles (workflow of 6 researchers + synthesis).
- `data/figma_tokens.json` v3.2 + `docs/design/TOKENS.md` — colour roles and palette modes (factory / custom / random), OFL type scale, strokes, radii, glow (earned bloom), glass, virtual light, materials, motion (closed-form springs, M3 durations, 10 Hz strip strobe), layout and zones, review states, components, 7 families, accessibility.
- `docs/design/COMPONENTS.md` — 16 shared components for `fh6-clusters/src/common/`.
- `docs/design/families/*.md` + `*-template.png` — 7 family specs and template frames.
- `docs/design/heroes/<family>/0{1..4}_*.png` — 28 hero frames; sources in `docs/design/boards/heroes/`.
- `program/phases/07-cluster-factory/CATALOG.md` — 29 rebuilds + 19 new concepts.
- `docs/design/SCRIM_FIT.md` + `docs/design/scrim/*.png` — OEM dash-blocker dome spec over the reference cockpit.
- `docs/design/OSD_DIRECTION.md`, `CONFIGURATOR_DIRECTION.md` + `docs/design/directions/*.png`.
- `docs/design/CRITIQUE.md` — three critique rounds and verdicts.
- `docs/design/FIGMA.md`, `docs/design/boards/index.html` (gallery), `docs/PROGRESS.md` updated.
- Figma `0zOEwnJ8dEj1kQ7QwPGEH0`: 4 variable collections (197 variables), 12 text styles, 5 effect styles, 3 pages, 7 template + 28 hero frames on page `1:4`.

## Verification output
Phase 3 changed no Rust. Real output (2026-11-21, this branch):
```
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 18.97s      (warnings only, as baseline)

$ cargo test --workspace --exclude fh6-app      (grouped "test result" lines)
      9 test result: ok. 0 passed; 0 failed
      1 test result: ok. 1 passed; 0 failed   (x3)
      1 test result: ok. 13 passed; 0 failed
      1 test result: ok. 20 passed; 0 failed
      1 test result: ok. 24 passed; 0 failed
      1 test result: ok. 25 passed; 0 failed
      1 test result: ok. 3 passed; 0 failed
      1 test result: ok. 5 passed; 0 failed
$ cargo test -p fh6-app --bins
test result: ok. 0 passed; 0 failed
test result: ok. 10 passed; 0 failed
=> 103 passed, 0 failed (baseline: 103)

$ cargo test --workspace            (known red, unchanged)
error[E0277]: can't compare `&str` with `str`
error: could not compile `fh6-app` (test "steam_launcher_tests") due to 1 previous error

$ cargo clippy --workspace -- -D warnings   (known red, unchanged)
error: this `if` statement can be collapsed
error: could not compile `fh6-core` (lib) due to 1 previous error

$ node -e "JSON.parse(fs.readFileSync('data/figma_tokens.json'))"
figma_tokens.json valid JSON          first bytes 7b 0a 20 (no BOM)

$ git ls-files | grep -E '\.(py|js|ts|ps1|sh|c|cpp|cs)$' | wc -l
0
```
`fh6-render::tokens::FigmaTokens` reads only optional `colors` and `grid`; v3 adds neither key, so the loader behaves as before. No perf numbers apply (no runtime code).

## Visuals
- Gallery: `docs/design/boards/index.html` (open in the built-in browser via a local server or GitHub).
- Heroes: `docs/design/heroes/{classic-analog,modern-digital,race-dash,liquid-glass,sci-fi,fluid-generative,adas-hud}/`.
- Templates: `docs/design/families/*-template.png`. Scrim: `docs/design/scrim/`. Directions: `docs/design/directions/`.

## My calls
- **Figma fallback in part:** Starter plan → 3 pages (Sections instead of pages), 1 variable mode (per-family palette variables), MCP cap reached; hero and component visuals are HTML boards rendered by headless Edge. Generator kept outside the repo (Rust-only).
- **Seven families, seven heroes:** research supported a separate Race Dash family; one hero per family, Liquid Glass as the flagship (the brief's "6" is exceeded, not missed).
- **Modern Digital hero is the hood view** (`Gt7HoodPro` is the default hood cluster), so the hood layout is shown and specified.
- **Fonts:** Saira Condensed default; Titillium Web for labels (Saira Semi Condensed is not in Figma; Titillium is the F1-heritage OFL face).
- **Colour:** default shift violet `#978DFC`; Classic primary `#FF7A1A` (≥ 25° from warning); Race Dash shift `#B27CFF` (≥ 60° from primary); unlit lights neutral.
- **Strobe:** 10 Hz on the strip only (≤ 42 px); limiter cues elsewhere are steady.
- **ADAS camera raised** to (0, 3.4, −6.0) m (critique showed the AGENTS.md eye hid the road).
- **Classic dial:** symmetric 270° default; shift-at-12 as an optional Track mode.
- **Catalog:** kept 19 concepts, dropped 5 near-duplicates (listed in the catalog).

## Known issues / debt
- **Figma (blocked):** component library on `0:1` unfinished; hero/template PNGs not yet placed on page `1:4`; page `1:5` empty; variables are tokens v3.1 (v3.2 colour changes not mirrored). Steps in FIGMA.md.
- **Critique bar not fully met:** three rounds ran (round 1: 13 critiques by both critics; round 2: 2 of 7 before the usage limit; round 3: lead pass with `design:design-critique` after two more agent runs were cut off). Every hero beats v1; the final verdict is "matches the references" (6.5–7.5/10), not yet "clearly better". The remaining gap is material and motion quality that needs the GPU kit and Phase 6 assets; CRITIQUE.md lists, per hero, what the Phase 7 chat must close. The 7-agent final round can be re-run with the saved workflow `fh6-phase3-critique-final` once the usage window allows.
- **Figma routes both failed** (exact errors in FIGMA.md): the remote MCP is capped on the Starter plan, and the `figma-desktop` server at `127.0.0.1:3845` is not loaded in this session and not listening (likely needs the desktop MCP server switched on and a Dev/Full seat).
- **AGENTS.md** still quotes the old ADAS eye; conductor should update it (PROGRESS.md open question 4).
- **Telemetry not yet decoded by `fh6-core`** that catalog rows rely on: fuel, lap times, race position, torque. Verify offsets against `tests/golden/packets` (97% rule) before Phase 7 uses them.
- Hero frames are SVG concept renders: materials that need the GPU kit (real lensing, live fluid, bloom chain, glTF cars) are approximated.

## What the next phases must know
- **Phase 5 (kit):** implement `light`, `glass`, `glow.emissiveRanks`, keylines (`type.keyline`), dimming pads, smooth-union liquid surface, closed-form springs from `motion.springs`; see TOKENS.md for every number.
- **Phase 6 (assets):** asset lists per family in each family spec and per row in CATALOG.md; fonts: Saira / Saira Condensed, Titillium Web, Barlow Semi Condensed, Michroma, Chakra Petch, Exo 2, B612 Mono (record `tnum` per face).
- **Phase 7 (factory):** pilot `LiquidGlass`; build order in CATALOG.md; shared components in COMPONENTS.md; one Figma Section per cluster on page `1:5` (no per-cluster pages on Starter); hero boards are the visual target.
- **Phase 8 (fit):** SCRIM_FIT.md (dome a 640 × b 470, n 2.6, centre anchor + (0, −75), feather 96/120/140); calibrate per car class in the low-seat view.
- **Phase 9 / 10:** OSD_DIRECTION.md and CONFIGURATOR_DIRECTION.md with their frames.
