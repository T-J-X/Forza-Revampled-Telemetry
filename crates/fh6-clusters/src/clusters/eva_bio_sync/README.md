# EVA Bio-Sync

Catalog row 21 · `ClusterTheme::EvaBioSync` (config value `"EvaBioSync"`) · Sci-Fi · cockpit · build #22.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/eva_bio_sync/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/eva_bio_sync/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/eva_bio_sync.rs`, if it needs bespoke assets.
- `tests/golden/images/EvaBioSync_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster EvaBioSync
cargo test -p fh6-clusters eva_bio_sync
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters eva_bio_sync
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster EvaBioSync --stats
```
`preview-export --cluster` writes `mockups/EvaBioSync/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
