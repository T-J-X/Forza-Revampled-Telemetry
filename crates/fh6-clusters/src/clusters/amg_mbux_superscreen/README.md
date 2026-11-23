# AMG MBUX Superscreen

Catalog row 4 · `ClusterTheme::AmgMbuxSuperscreen` (config value `"AmgMbuxSuperscreen"`) · Modern Digital 2027 · cockpit · build #9.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/amg_mbux_superscreen/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/amg_mbux_superscreen/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/amg_mbux_superscreen.rs`, if it needs bespoke assets.
- `tests/golden/images/AmgMbuxSuperscreen_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster AmgMbuxSuperscreen
cargo test -p fh6-clusters amg_mbux_superscreen
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters amg_mbux_superscreen
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster AmgMbuxSuperscreen --stats
```
`preview-export --cluster` writes `mockups/AmgMbuxSuperscreen/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
