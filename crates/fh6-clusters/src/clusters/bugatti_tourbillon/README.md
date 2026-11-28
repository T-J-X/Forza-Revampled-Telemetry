# Bugatti Tourbillon

Catalog row 9 · `ClusterTheme::BugattiTourbillon` (config value `"BugattiTourbillon"`) · Hypercar category · target `recreation` · Classic Analog family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/bugatti_tourbillon/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/bugatti_tourbillon/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/bugatti_tourbillon.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/BugattiTourbillon_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster BugattiTourbillon
cargo test -p fh6-clusters bugatti_tourbillon
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters bugatti_tourbillon
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster BugattiTourbillon --stats
```
`preview-export --cluster` writes `mockups/BugattiTourbillon/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
