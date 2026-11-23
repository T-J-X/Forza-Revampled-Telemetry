# GT7 Hood Pro

Catalog row 3 · `ClusterTheme::Gt7HoodPro` (config value `"Gt7HoodPro"`) · Modern Digital 2027 · hood · build #2 · ★ pattern-setter.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/gt7_hood_pro/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/gt7_hood_pro/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/gt7_hood_pro.rs`, if it needs bespoke assets.
- `tests/golden/images/Gt7HoodPro_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Gt7HoodPro
cargo test -p fh6-clusters gt7_hood_pro
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt7_hood_pro
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt7HoodPro --stats
```
`preview-export --cluster` writes `mockups/Gt7HoodPro/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
