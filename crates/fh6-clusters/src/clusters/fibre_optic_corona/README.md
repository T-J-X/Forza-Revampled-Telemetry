# Fibre-Optic Corona

Catalog row 33 · `ClusterTheme::FibreOpticCorona` (config value `"FibreOpticCorona"`) · Liquid Glass · cockpit · build #48.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/fibre_optic_corona/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/fibre_optic_corona/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/fibre_optic_corona.rs`, if it needs bespoke assets.
- `tests/golden/images/FibreOpticCorona_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster FibreOpticCorona
cargo test -p fh6-clusters fibre_optic_corona
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters fibre_optic_corona
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster FibreOpticCorona --stats
```
`preview-export --cluster` writes `mockups/FibreOpticCorona/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
