# Harmonograph Drive

Catalog row 48 · `ClusterTheme::HarmonographDrive` (config value `"HarmonographDrive"`) · Fluid / Generative · cockpit · build #47.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/harmonograph_drive/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/harmonograph_drive/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/harmonograph_drive.rs`, if it needs bespoke assets.
- `tests/golden/images/HarmonographDrive_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster HarmonographDrive
cargo test -p fh6-clusters harmonograph_drive
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters harmonograph_drive
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster HarmonographDrive --stats
```
`preview-export --cluster` writes `mockups/HarmonographDrive/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
