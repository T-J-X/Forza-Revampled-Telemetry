# Tesla ADAS

Catalog row 15 · `ClusterTheme::TeslaAdas` (config value `"TeslaAdas"`) · ADAS & HUD · cockpit+hood · build #5 · ★ pattern-setter.

Status: **built** (Phase 7 cluster chat, 2026-11-23). `mod.rs` is the cluster (zone table at the top, top zone: pods, rev ribbon, rolling gear pill, speed, rpm, chips); `scene.rs` is the Z4 road diorama (glTF clay cars through the kit mesh pass, carpet, lanes, stop line, blind-spot arcs, underglow, seeded traffic). `HANDOVER.md` has the renders, perf, critique and the shared-code requests.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/tesla_adas/`: `mod.rs` (the `Cluster` impl and `make`), `scene.rs` (the private road scene), `spec.md` (the brief), this README and `HANDOVER.md`.
- `assets/clusters/tesla_adas/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/tesla_adas.rs`, if it needs bespoke assets.
- `tests/golden/images/TeslaAdas_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster TeslaAdas
cargo test -p fh6-clusters tesla_adas
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters tesla_adas
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster TeslaAdas --stats
```
`preview-export --cluster` writes `mockups/TeslaAdas/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
