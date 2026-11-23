# Shibuya Chromatic Prisma

Catalog row 26 · `ClusterTheme::ShibuyaChromaticPrisma` (config value `"ShibuyaChromaticPrisma"`) · Sci-Fi · cockpit · build #24.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/shibuya_chromatic_prisma/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/shibuya_chromatic_prisma/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/shibuya_chromatic_prisma.rs`, if it needs bespoke assets.
- `tests/golden/images/ShibuyaChromaticPrisma_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster ShibuyaChromaticPrisma
cargo test -p fh6-clusters shibuya_chromatic_prisma
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters shibuya_chromatic_prisma
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster ShibuyaChromaticPrisma --stats
```
`preview-export --cluster` writes `mockups/ShibuyaChromaticPrisma/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
