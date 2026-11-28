# Hakosuka S20

Catalog row 64 · `ClusterTheme::HakosukaS20` (config value `"HakosukaS20"`) · Retro category · target `inspired` · Classic Analog family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/hakosuka_s20/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/hakosuka_s20/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/hakosuka_s20.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/HakosukaS20_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster HakosukaS20
cargo test -p fh6-clusters hakosuka_s20
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters hakosuka_s20
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster HakosukaS20 --stats
```
`preview-export --cluster` writes `mockups/HakosukaS20/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
