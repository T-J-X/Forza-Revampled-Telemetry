# LFA V10 Nürburgring

Catalog row 10 · `ClusterTheme::LfaV10Nurburgring` (config value `"LfaV10Nurburgring"`) · Classic Analog · cockpit · build #12.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/lfa_v10_nurburgring/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/lfa_v10_nurburgring/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/lfa_v10_nurburgring.rs`, if it needs bespoke assets.
- `tests/golden/images/LfaV10Nurburgring_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster LfaV10Nurburgring
cargo test -p fh6-clusters lfa_v10_nurburgring
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters lfa_v10_nurburgring
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster LfaV10Nurburgring --stats
```
`preview-export --cluster` writes `mockups/LfaV10Nurburgring/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
