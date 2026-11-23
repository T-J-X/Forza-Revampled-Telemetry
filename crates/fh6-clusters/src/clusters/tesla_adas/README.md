# Tesla ADAS

Catalog row 15 · `ClusterTheme::TeslaAdas` (config value `"TeslaAdas"`) · ADAS & HUD · cockpit+hood · build #5 · ★ pattern-setter.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/tesla_adas/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/tesla_adas/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/tesla_adas.rs`, if it needs bespoke assets.
- `tests/golden/images/TeslaAdas_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster TeslaAdas
cargo test -p fh6-clusters tesla_adas
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters tesla_adas
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster TeslaAdas --stats
```
`preview-export --cluster` writes `mockups/TeslaAdas/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
