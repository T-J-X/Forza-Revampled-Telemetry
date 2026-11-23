# HANDOVER: Phase 7 setup · Cluster factory (chat 07-00)

**Chat / branch:** `claude/exciting-napier-8fa678`  **Date:** 2026-11-23  **Result:** ✅ complete. The pilot's critique is a conditional pass, and the design tool moved to Penpot mid-phase (pages not made yet: this session has no Penpot tools).

## Exit criteria
| Criterion (MASTER_PROMPT) | Met? | Evidence |
|---|---|---|
| Every catalog entry has a module that compiles, is registered and appears in `preview-export` | ✅ | 48 modules in `crates/fh6-clusters/src/clusters/<snake>/` (`mod.rs`, `spec.md`, `README.md`). `registry::REGISTRY` has 48 rows (tests `every_theme_has_its_own_row`, `rows_follow_the_catalog`). `preview-export` renders all 48 × 4 states. |
| New concepts have `ClusterTheme` variants, appended only | ✅ | 19 variants, 29–47, in `fh6-core/src/theme.rs` (test `numbering_is_dense_and_stable`) |
| Shared helpers in `common/` | ✅ | 17 modules, see `docs/kit/README.md` §12; component shot tests `tests/component_{tach,classic,adas,instruments,seg7,liquid}.rs` |
| `preview-export --cluster <Name>` | ✅ | writes `mockups/<Name>/`: 4 states per view, crops, cockpit composites, thumbnails, the 60-frame sequence, `index.html`, and GPU p50 / p99 |
| `cargo test -p fh6-clusters <name>` runs its golden test | ✅ | `tests/golden_images.rs`: one test per catalog row (49 tests including the coverage check), 260 goldens |
| `FH6_WheelDash --demo --cluster <Name>` | ✅ | `--cluster` in `fh6-app` args; `ConfigHandle::force_cluster_style` survives hot reloads (test `a_forced_style_survives_every_publish`); verified live with LiquidGlass |
| Catalog ordered, first 3 `next` | ✅ | `CATALOG.md` build-order table: LiquidGlass ✅ (pilot), then Gt7HoodPro, PorscheAnalog and BoschDdu `next` |
| Design pages with IDs | ◐ pending (Penpot) | Figma never worked (Starter-plan call cap, `figma-desktop` refused). On 2026-11-23 TJX switched to Penpot (unlimited). This session started before the `penpot` MCP was registered, so the Penpot master file and the 48 `07 · <Name>` pages are being built by the "Build the Penpot design-system master file" session. **Pending from that session:** the file ID and the row → page / board ID table (it records them in `docs/design/PENPOT.md` on its branch); fill the CATALOG "Design board" column from it. `CATALOG.md` "Design boards (Penpot)" and the Phase 7 prompts are switched to Penpot; the HTML board `mockups/<Name>/index.html` stays in use alongside. |
| Pilot to the `CLUSTER_PROMPT.md` exit bar | ✅ / critique ◐ | `clusters/liquid_glass/HANDOVER.md`: module, goldens, renders, sequence, live demo and GPU numbers are all done. Four critique rounds went 6 → 7.5/10 with every hierarchy, limiter and motion finding closed; the remaining gap is glass material in shared code (see Known issues). |
| `cargo test` / `clippy -D warnings` green | ✅ | see "Verification output" |

## What was built
- **Catalog as data:** `crates/fh6-clusters/src/catalog.rs` (48 `CatalogEntry` rows: family, kind, views, build order, pattern-setter, spec path; `find` by theme, snake or display name).
- **Registry:** `registry.rs`, generated once from the catalog, with `resolve` falling back to LiquidGlass.
- **48 module directories,** each drawing `common::scaffold::Scaffold` until its cluster chat replaces it. The 29 v1 tiny-skia clusters moved to `legacy/` (`git mv`), behind `LegacyClusterRenderer`.
- **Review harness:** `review.rs` (the 4 states, a 60-frame sequence of idle → pull → limiter strobe → upshift → pull, and a `Driver` that feeds a cluster like the overlay does), `tests/support/` (a shared GPU, crops to the envelope, the glyph-complete render loop) and `tests/golden_images.rs`.
- **Shared components in `common/`:**
  - `glass_plate`, `ribbon`, `shift`, `hero_gear`, `speed`, `label`, `scaffold`
  - `tach`, `needle_dial`, `bezel`, `adas_road`, `gball`, `tyre_slip`, `signed_bar`, `race`, `seg7`, `liquid`
  - Each was built, then independently reviewed and fixed. The review agents for `adas_road`, `seg7` and `liquid` hit the usage limit before reviewing. Those three compile, pass their unit and shot tests and are clippy-clean, but they have not had the second review.
- **Tooling:**
  - `preview-export --cluster` (`tools/preview-export/src/cluster.rs`); a WARN-level tracing subscriber so wgpu validation errors show
  - `web-preview ?cluster=<Theme>`
  - bench-render scenario names
- **Kit fixes:**
  - Font loading: superseded in the merge by Phase 9's loader, plus the deferred-face fix described under Known issues.
  - `Scene::missing_glyph_count`.
  - The glass-first backdrop is cleared with render passes instead of `clear_texture` (that needs the `CLEAR_TEXTURE` feature).
- **`fh6-core`:** `ClusterTheme::from_name`; Phase 9's `name()` / dispatch refactor adopted verbatim, so the merge is mechanical; the `ConfigHandle` forced style.
- **Docs:**
  - `docs/kit/README.md` §1 rewritten, plus new §12 Shared components and §13 Lessons from the pilot
  - `CLUSTER_PROMPT.md` rewritten (ownership, board fallback, commands, lessons)
  - `CATALOG.md` regenerated (index with Module / Board / Status columns, build order with waves)
  - `AGENTS.md` rows and commands
- **The pilot:** `clusters/liquid_glass/` (module, README, HANDOVER, 8 goldens) and `docs/handover-shots/cluster-liquid_glass-*.png`.

## Verification output
```
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] (only the web-preview manifest note, as on main)
$ cargo test --workspace
    597 passed, 0 failed, 1 ignored, on the tree merged with main 71994b2 (fh6-clusters lib 168; golden_images 49 incl. 260 goldens; component shots 6 suites; Phase 9 OSD suites)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile (clean, no warnings)
$ cargo fmt --all -- --check
    exit 0
$ cargo run -p parity-dump -- tests/golden/packets
    20/20 OK
$ cargo build -p fh6-kit -p fh6-clusters --target wasm32-unknown-unknown
    Finished
```
preview-export `--cluster LiquidGlass`: cockpit sequence GPU p50 0.227 ms / p99 0.238 ms, cockpit states 0.23–0.24 ms, hood states 0.44–0.45 ms (RTX 3070 Ti Laptop, 3840 × 2160).

## Visuals
- `docs/handover-shots/cluster-liquid_glass-{02_mid_cruise,03_redline_strobe,04_drift_braking,hood_02_mid_cruise,cockpit}.png`
- `mockups/<Name>/index.html` for any cluster after `preview-export --cluster <Name>`; `mockups/index.html` for all 48 (gitignored).
- Component shots: `target/component-shots/` after `cargo test -p fh6-clusters --test component_<group>`.

## My calls
- **v1 clusters moved to `legacy/`.** `clusters/<x>.rs` and `clusters/<x>/mod.rs` cannot coexist; the v1 code stays reachable behind `LegacyClusterRenderer` until each rebuild lands.
- **Adopted Phase 9's `theme.rs` `name()` and `dispatch.rs` verbatim,** then appended the 19 variants. This keeps the merge mechanical (see the merge notes).
- **`--cluster` forces the style through `ConfigHandle`** (a `OnceLock` applied on every publish), so hot reloads cannot undo it.
- **Faces are rasterised on first use** (Phase 9's deferred preload, fixed so `find` faces work). A new face's glyphs appear one frame late, so the review tools repeat a frame until no glyph is missing (`Scene::missing_glyph_count`).
- **The dev profile builds `rustybuzz`, `ttf-parser`, `fdsm`, `png`, `miniz_oxide` and `fdeflate` at opt-level 3,** so the golden tests load fonts in seconds instead of minutes.
- **Goldens cover every catalog row now** (the scaffold renders). Each cluster chat overwrites only its own. Total size is 27 MB in `tests/golden/images/` (260 PNGs).
- **Design tool:** Phase 7 docs now point at Penpot (one page per cluster). Until the pages exist, the board is the HTML board plus the zone table in each `mod.rs`.
- **`ShiftLights` accepts even counts** (the 16-pill strip lights the two centre pods together). `PodStyle` gained `unlit_rim`, defaulting to the old 0.12, so other clusters are unchanged.
- **The pilot's design calls** are in its own `HANDOVER.md`.

## Known issues / debt
- **Pilot glass material (shared / kit work for the conductor to schedule):**
  - `GlassPlate` needs a per-azimuth rim (100% within ±35° of the light, ≤8% opposite) and a tinted fresnel ramp.
  - The kit needs a smooth-union between shapes (the pendant bulge) and self-lensing of layers drawn after a glass layer (glass-first clusters refract nothing).
- **Second reviews missing:** `common/adas_road.rs`, `seg7.rs` and `liquid.rs` were built but their second review was cut by the usage limit. The first cluster chat that uses one of them should look harder.
- **Main is merged in** (Phase 9, `71994b2`):
  - **Conflicts resolved:** `Cargo.toml` (dev opt-levels united), `theme.rs` (both lists), `preview-export` (`--cluster` and `--backdrop` together), `AGENTS.md` and `PROGRESS.md`.
  - **`text.rs` is Phase 9's loader** (fonts.json manifest, variable instances, deferred preload), which replaces the Phase 7 lazy loader.
  - **The OSD's `theme_label`** gained the 19 new themes.
- **Kit fix made in the merge (`fh6-kit/src/text.rs`).** Phase 9's deferred preload had a bug: a face found with `Fonts::find` (not bound to a role) borrowed the fallback face's glyphs forever, so it was never rasterised. Every cluster that picks its own faces drew them in the fallback font, and tabular figures were lost.
  - `glyph()` now falls back only for preloaded faces.
  - `generate_missing` preloads a deferred face on first use and never blanks a glyph that the preload added.
  - A regression test sits in `load_dir_reads_the_font_pack_with_a_deferred_preload`.
  - The OSD binds roles, so it was not affected.
  - All 260 goldens were regenerated afterwards.
- **Per-car profiles can still override `--cluster`:** a car profile's `CockpitHudStyle` wins when that car loads. Fine for the demo (car ordinal 0); worth a guard if it bites.
- **Live CPU per frame** is p50 1.44 / p99 3.66 ms on a desktop without the stand-in game, against the 1 ms target. This is pre-existing host cost (upload + record + submit) and is not caused by the clusters.
- **`tools/web-preview` manifest note** ("unused dependency `fh6-clusters`" on native builds): it is used only on wasm32. It is a warning and is on main too.
- **`preview-export` has no hood-over-photo composite,** so hood legibility over a bright bonnet is unproven for every hood cluster.
- **The first state of each view reads high in `preview-export`** (the first frame after a view switch). Use the sequence p50 / p99 as the budget number.

## What the next phases must know
- **Cluster chats own exactly their directory, goldens, assets and their own `CATALOG.md` status cell.**
  - `catalog.rs`, `registry.rs`, `ClusterTheme`, `common/`, the golden harness, the kit and the tools are frozen for them.
  - Shared changes go through their handover.
- **Commands (from the repo root):**
  - `cargo run --release -p preview-export -- --cluster <Name>`
  - `FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters <snake>`, then `cargo test -p fh6-clusters <snake>`
  - `cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster <Name> --stats`
- **Next three:** Gt7HoodPro (hood default; tach, G-ball and tyre-slip helpers), PorscheAnalog (needle dial and bezel), BoschDdu (race, seg7 and tach bar).
- **Read `docs/kit/README.md` §12–13 and the pilot first.** The two most expensive lessons:
  - Long polylines cost GPU (draw paths in chunks).
  - Alphas are linear (3% reads as 19% grey).
