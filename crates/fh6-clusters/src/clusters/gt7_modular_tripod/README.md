# GT7 Modular Tripod

Catalog row 14 · `ClusterTheme::Gt7ModularTripod` (config value `"Gt7ModularTripod"`) · Race Dash · cockpit+hood · build #17.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/gt7_modular_tripod/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/gt7_modular_tripod/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/gt7_modular_tripod.rs`, if it needs bespoke assets.
- `tests/golden/images/Gt7ModularTripod_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Gt7ModularTripod
cargo test -p fh6-clusters gt7_modular_tripod
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt7_modular_tripod
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt7ModularTripod --stats
```
`preview-export --cluster` writes `mockups/Gt7ModularTripod/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
