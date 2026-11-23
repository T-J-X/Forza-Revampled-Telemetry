# Lightpainter

Catalog row 41 · `ClusterTheme::Lightpainter` (config value `"Lightpainter"`) · ADAS & HUD · cockpit+hood · build #40.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/lightpainter/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/lightpainter/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/lightpainter.rs`, if it needs bespoke assets.
- `tests/golden/images/Lightpainter_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Lightpainter
cargo test -p fh6-clusters lightpainter
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters lightpainter
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Lightpainter --stats
```
`preview-export --cluster` writes `mockups/Lightpainter/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
