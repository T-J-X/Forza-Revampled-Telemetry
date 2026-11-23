# Glyph Dot Matrix

Catalog row 6 · `ClusterTheme::GlyphDotMatrix` (config value `"GlyphDotMatrix"`) · Modern Digital 2027 · cockpit · build #11.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/glyph_dot_matrix/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/glyph_dot_matrix/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/glyph_dot_matrix.rs`, if it needs bespoke assets.
- `tests/golden/images/GlyphDotMatrix_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster GlyphDotMatrix
cargo test -p fh6-clusters glyph_dot_matrix
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters glyph_dot_matrix
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster GlyphDotMatrix --stats
```
`preview-export --cluster` writes `mockups/GlyphDotMatrix/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
