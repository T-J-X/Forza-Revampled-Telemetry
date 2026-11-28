# GT7 Modular Tripod

Catalog row 14 · `ClusterTheme::Gt7ModularTripod` (config value `"Gt7ModularTripod"`) · Motorsport category · target `recreation` · Race Dash family · cockpit+hood.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/gt7_modular_tripod/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/gt7_modular_tripod/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/gt7_modular_tripod.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/Gt7ModularTripod_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Gt7ModularTripod
cargo test -p fh6-clusters gt7_modular_tripod
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt7_modular_tripod
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt7ModularTripod --stats
```
`preview-export --cluster` writes `mockups/Gt7ModularTripod/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
