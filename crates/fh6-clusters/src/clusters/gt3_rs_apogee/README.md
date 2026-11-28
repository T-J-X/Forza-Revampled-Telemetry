# GT3 RS Apogee

Catalog row 54 · `ClusterTheme::Gt3RsApogee` (config value `"Gt3RsApogee"`) · Supercar category · target `inspired` · Classic Analog family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/gt3_rs_apogee/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/gt3_rs_apogee/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/gt3_rs_apogee.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/Gt3RsApogee_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Gt3RsApogee
cargo test -p fh6-clusters gt3_rs_apogee
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt3_rs_apogee
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt3RsApogee --stats
```
`preview-export --cluster` writes `mockups/Gt3RsApogee/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
