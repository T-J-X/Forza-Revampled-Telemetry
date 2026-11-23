# Vernier Loupe

Catalog row 31 · `ClusterTheme::VernierLoupe` (config value `"VernierLoupe"`) · Liquid Glass · cockpit+hood · build #30.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/vernier_loupe/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/vernier_loupe/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/vernier_loupe.rs`, if it needs bespoke assets.
- `tests/golden/images/VernierLoupe_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster VernierLoupe
cargo test -p fh6-clusters vernier_loupe
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters vernier_loupe
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster VernierLoupe --stats
```
`preview-export --cluster` writes `mockups/VernierLoupe/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
