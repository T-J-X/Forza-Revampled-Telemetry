# Windshield HUD

Catalog row 16 · `ClusterTheme::WindshieldHud` (config value `"WindshieldHud"`) · ADAS & HUD · raised HUD · build #18.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/windshield_hud/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/windshield_hud/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/windshield_hud.rs`, if it needs bespoke assets.
- `tests/golden/images/WindshieldHud_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster WindshieldHud
cargo test -p fh6-clusters windshield_hud
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters windshield_hud
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster WindshieldHud --stats
```
`preview-export --cluster` writes `mockups/WindshieldHud/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
