# Porsche Analog

Catalog row 7 · `ClusterTheme::PorscheAnalog` (config value `"PorscheAnalog"`) · Classic Analog · cockpit · build #3 · ★ pattern-setter.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/porsche_analog/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/porsche_analog/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/porsche_analog.rs`, if it needs bespoke assets.
- `tests/golden/images/PorscheAnalog_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster PorscheAnalog
cargo test -p fh6-clusters porsche_analog
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters porsche_analog
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster PorscheAnalog --stats
```
`preview-export --cluster` writes `mockups/PorscheAnalog/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
