# Vision Spatial Glass

Catalog row 2 · `ClusterTheme::VisionSpatialGlass` (config value `"VisionSpatialGlass"`) · Apple Vision category · target `concept` · Liquid Glass family · cockpit.

Status: **in progress** (thread 'Build VisionSpatialGlass dash', PR #20). `mod.rs` draws the dash from the Penpot board; `RESEARCH.md` section 9 lists what the board changed from the brief.

Penpot: page `07 · VisionSpatialGlass` (id `347e5ae2-9e0b-80b1-8009-03925c3ac4cb`); boards (v4) 01 idle `…03a308779d08`, 02 mid cruise `…03a30bc54065`, 03 redline strobe `…03a30fe98825`, 04 drift braking `…03a3149c5f43`, 05 motion notes `…03951ba3137e`, 06 ideas `…0396040b493e`, 07 gear placement `…03a2b6c8da11`, 08 gear change `…03a318879361` (every id starts `347e5ae2-9e0b-80b1-8009-`). Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/vision_spatial_glass/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/vision_spatial_glass/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/vision_spatial_glass.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/VisionSpatialGlass_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster VisionSpatialGlass
cargo test -p fh6-clusters vision_spatial_glass
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters vision_spatial_glass
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster VisionSpatialGlass --stats
```
`preview-export --cluster` writes `mockups/VisionSpatialGlass/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
