# Aurora Veil

Catalog row 45 · `ClusterTheme::AuroraVeil` (config value `"AuroraVeil"`) · Sci-Fi · cockpit · build #41.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/aurora_veil/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/aurora_veil/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/aurora_veil.rs`, if it needs bespoke assets.
- `tests/golden/images/AuroraVeil_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster AuroraVeil
cargo test -p fh6-clusters aurora_veil
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters aurora_veil
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster AuroraVeil --stats
```
`preview-export --cluster` writes `mockups/AuroraVeil/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
