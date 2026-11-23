# Rack Focus

Catalog row 38 · `ClusterTheme::RackFocus` (config value `"RackFocus"`) · Race Dash · cockpit+hood · build #33.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/rack_focus/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/rack_focus/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/rack_focus.rs`, if it needs bespoke assets.
- `tests/golden/images/RackFocus_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster RackFocus
cargo test -p fh6-clusters rack_focus
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters rack_focus
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster RackFocus --stats
```
`preview-export --cluster` writes `mockups/RackFocus/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
