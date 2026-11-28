# AMG MBUX Supersport

Catalog row 49 · `ClusterTheme::AmgMbuxSupersport` (config value `"AmgMbuxSupersport"`) · Supercar category · target `recreation` · Modern Digital 2027 family · cockpit.

Status: **scaffold**. `mod.rs` draws the shared under-construction frame from `common::scaffold` until the dash thread replaces it. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/amg_mbux_supersport/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/amg_mbux_supersport/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/amg_mbux_supersport.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/AmgMbuxSupersport_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster AmgMbuxSupersport
cargo test -p fh6-clusters amg_mbux_supersport
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters amg_mbux_supersport
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster AmgMbuxSupersport --stats
```
`preview-export --cluster` writes `mockups/AmgMbuxSupersport/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.

## Design board
Penpot master file `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, page `07 · AmgMbuxSupersport` (`347e5ae2-9e0b-80b1-8009-039ab96ca3ef`). Board v1 (2026-11-28, a 1:1 measure of the real blue Supersport plus the elevated notes):

| Board | ID |
|---|---|
| v1 · 01 idle | `347e5ae2-9e0b-80b1-8009-039d00b23109` |
| v1 · 02 mid cruise | `347e5ae2-9e0b-80b1-8009-039d3565c1ab` |
| v1 · 03 redline strobe | `347e5ae2-9e0b-80b1-8009-039d38b57107` |
| v1 · 04 drift braking | `347e5ae2-9e0b-80b1-8009-039d3c31af94` |
| v1 · 05 motion notes | `347e5ae2-9e0b-80b1-8009-039d8eb1d71c` |

Exports (PNG and SVG) are in the Drive masters folder `amg_mbux_supersport/board_v1/`.
