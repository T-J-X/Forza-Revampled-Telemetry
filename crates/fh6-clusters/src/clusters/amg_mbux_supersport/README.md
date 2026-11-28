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

Board v2 (2026-11-28, current): thick fire jets shot from the tunnel-end mesh, the real blue brightness, subtler hills and gear, a bolder speed (TJS 09:12).

| Board | ID |
|---|---|
| v2 · 01 idle | `347e5ae2-9e0b-80b1-8009-03a02ede3ed6` |
| v2 · 02 mid cruise | `347e5ae2-9e0b-80b1-8009-03a0327b599f` |
| v2 · 03 redline strobe | `347e5ae2-9e0b-80b1-8009-03a03648412d` |
| v2 · 04 drift braking | `347e5ae2-9e0b-80b1-8009-03a03a859dc7` |

Board v3 (2026-11-28 09:55, current): no upshift strip, flame edges on the rev line and the floor edge, the speed in front with an extrude and shadow, the gear set back in the haze (TJS 09:52). Open item for the next pass: the real flame is a smooth plasma ribbon that swells and pinches, while v3 draws it as a spiky edge (RESEARCH.md, "Board v3 notes", item 2).

| Board | ID |
|---|---|
| v3 · 01 idle | `347e5ae2-9e0b-80b1-8009-03a8ee595be1` |
| v3 · 02 mid cruise | `347e5ae2-9e0b-80b1-8009-03a8f3c502fe` |
| v3 · 03 redline strobe | `347e5ae2-9e0b-80b1-8009-03a8f93273e6` |
| v3 · 04 drift braking | `347e5ae2-9e0b-80b1-8009-03a8ff2f2124` |

Exports (PNG and SVG) are in the Drive masters folder `amg_mbux_supersport/board_v1/`, `board_v2/` and `board_v3/`.
