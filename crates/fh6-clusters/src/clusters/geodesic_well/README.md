# Geodesic Well

Catalog row 40 · `ClusterTheme::GeodesicWell` (config value `"GeodesicWell"`) · ADAS & HUD · cockpit+hood · build #34.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/geodesic_well/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/geodesic_well/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/geodesic_well.rs`, if it needs bespoke assets.
- `tests/golden/images/GeodesicWell_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster GeodesicWell
cargo test -p fh6-clusters geodesic_well
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters geodesic_well
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster GeodesicWell --stats
```
`preview-export --cluster` writes `mockups/GeodesicWell/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
