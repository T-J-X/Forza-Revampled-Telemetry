# Hyperbolic Tiles

Catalog row 72 · `ClusterTheme::HyperbolicTiles` (config value `"HyperbolicTiles"`) · Abstract category · target `concept` · Fluid / Generative family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/hyperbolic_tiles/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/hyperbolic_tiles/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/hyperbolic_tiles.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/HyperbolicTiles_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster HyperbolicTiles
cargo test -p fh6-clusters hyperbolic_tiles
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters hyperbolic_tiles
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster HyperbolicTiles --stats
```
`preview-export --cluster` writes `mockups/HyperbolicTiles/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
