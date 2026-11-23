# Liquid Glass

Catalog row 1 · `ClusterTheme::LiquidGlass` (config value `"LiquidGlass"`) · Liquid Glass · cockpit+hood · build #1 · ★ pattern-setter.

Status: built by the Phase 7 setup chat as the pilot; see [HANDOVER.md](HANDOVER.md).

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/liquid_glass/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/liquid_glass/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/liquid_glass.rs`, if it needs bespoke assets.
- `tests/golden/images/LiquidGlass_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster LiquidGlass
cargo test -p fh6-clusters liquid_glass
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters liquid_glass
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster LiquidGlass --stats
```
`preview-export --cluster` writes `mockups/LiquidGlass/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
