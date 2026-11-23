# Lumen Etch

Catalog row 30 · `ClusterTheme::LumenEtch` (config value `"LumenEtch"`) · Liquid Glass · cockpit · build #43.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/lumen_etch/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/lumen_etch/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/lumen_etch.rs`, if it needs bespoke assets.
- `tests/golden/images/LumenEtch_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster LumenEtch
cargo test -p fh6-clusters lumen_etch
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters lumen_etch
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster LumenEtch --stats
```
`preview-export --cluster` writes `mockups/LumenEtch/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
