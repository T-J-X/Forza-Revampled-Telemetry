# LFA V10 Nürburgring

Catalog row 10 · `ClusterTheme::LfaV10Nurburgring` (config value `"LfaV10Nurburgring"`) · Supercar category · target `recreation` · Classic Analog family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/lfa_v10_nurburgring/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/lfa_v10_nurburgring/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/lfa_v10_nurburgring.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/LfaV10Nurburgring_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster LfaV10Nurburgring
cargo test -p fh6-clusters lfa_v10_nurburgring
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters lfa_v10_nurburgring
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster LfaV10Nurburgring --stats
```
`preview-export --cluster` writes `mockups/LfaV10Nurburgring/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
