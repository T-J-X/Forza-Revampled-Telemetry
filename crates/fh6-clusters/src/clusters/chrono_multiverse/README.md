# Chrono Multiverse

Catalog row 29 · `ClusterTheme::ChronoMultiverse` (config value `"ChronoMultiverse"`) · Fluid / Generative · cockpit+hood · build #29.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/chrono_multiverse/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/chrono_multiverse/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/chrono_multiverse.rs`, if it needs bespoke assets.
- `tests/golden/images/ChronoMultiverse_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster ChronoMultiverse
cargo test -p fh6-clusters chrono_multiverse
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters chrono_multiverse
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster ChronoMultiverse --stats
```
`preview-export --cluster` writes `mockups/ChronoMultiverse/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
