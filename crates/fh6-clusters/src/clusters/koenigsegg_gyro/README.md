# Koenigsegg Gyro

Catalog row 11 · `ClusterTheme::KoenigseggGyro` (config value `"KoenigseggGyro"`) · Classic Analog · cockpit · build #15.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/koenigsegg_gyro/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/koenigsegg_gyro/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/koenigsegg_gyro.rs`, if it needs bespoke assets.
- `tests/golden/images/KoenigseggGyro_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster KoenigseggGyro
cargo test -p fh6-clusters koenigsegg_gyro
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters koenigsegg_gyro
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster KoenigseggGyro --stats
```
`preview-export --cluster` writes `mockups/KoenigseggGyro/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
