# Arc Ladder Cascade

Catalog row 44 · `ClusterTheme::ArcLadderCascade` (config value `"ArcLadderCascade"`) · Sci-Fi · cockpit · build #46.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/arc_ladder_cascade/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/arc_ladder_cascade/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/arc_ladder_cascade.rs`, if it needs bespoke assets.
- `tests/golden/images/ArcLadderCascade_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster ArcLadderCascade
cargo test -p fh6-clusters arc_ladder_cascade
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters arc_ladder_cascade
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster ArcLadderCascade --stats
```
`preview-export --cluster` writes `mockups/ArcLadderCascade/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
