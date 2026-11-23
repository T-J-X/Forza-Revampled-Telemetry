# Bosch DDU

Catalog row 12 · `ClusterTheme::BoschDdu` (config value `"BoschDdu"`) · Race Dash · cockpit · build #4 · ★ pattern-setter.

Status: **built** (Phase 7 cluster chat; final critique 8.5/10, PASS). Bespoke assets: `assets/clusters/bosch_ddu/` (Blender housing on a ComfyUI carbon weave). See `HANDOVER.md` for what it is, the derived lamps, perf and the shared change requests.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/bosch_ddu/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/bosch_ddu/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/bosch_ddu.rs`, if it needs bespoke assets.
- `tests/golden/images/BoschDdu_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster BoschDdu
cargo test -p fh6-clusters bosch_ddu
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters bosch_ddu
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster BoschDdu --stats
```
`preview-export --cluster` writes `mockups/BoschDdu/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
