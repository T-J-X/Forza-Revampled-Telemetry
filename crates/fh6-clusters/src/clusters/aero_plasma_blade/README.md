# Aero Plasma Blade

Catalog row 25 · `ClusterTheme::AeroPlasmaBlade` (config value `"AeroPlasmaBlade"`) · Sci-Fi · hood · build #25.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/aero_plasma_blade/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/aero_plasma_blade/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/aero_plasma_blade.rs`, if it needs bespoke assets.
- `tests/golden/images/AeroPlasmaBlade_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster AeroPlasmaBlade
cargo test -p fh6-clusters aero_plasma_blade
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters aero_plasma_blade
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster AeroPlasmaBlade --stats
```
`preview-export --cluster` writes `mockups/AeroPlasmaBlade/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
