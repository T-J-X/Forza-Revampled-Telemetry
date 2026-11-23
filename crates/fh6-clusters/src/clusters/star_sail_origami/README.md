# Star Sail Origami

Catalog row 43 · `ClusterTheme::StarSailOrigami` (config value `"StarSailOrigami"`) · Sci-Fi · cockpit+hood · build #35.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/star_sail_origami/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/star_sail_origami/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/star_sail_origami.rs`, if it needs bespoke assets.
- `tests/golden/images/StarSailOrigami_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster StarSailOrigami
cargo test -p fh6-clusters star_sail_origami
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters star_sail_origami
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster StarSailOrigami --stats
```
`preview-export --cluster` writes `mockups/StarSailOrigami/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
