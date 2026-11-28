# Valkyrie LMP1

Catalog row 13 · `ClusterTheme::ValkyrieLmp1` (config value `"ValkyrieLmp1"`) · Hypercar category · target `recreation` · Race Dash family · cockpit+hood.

Status: **in progress** (thread "Build ValkyrieLmp1 dash"). `mod.rs` draws the Valkyrie wheel OLED: ten blue shift lights (1 + 8 + 1), a round rev counter numbered from the limiter with the gear at its centre, the rev tunnel, speed, the friction circle, power, the engineer trace, derived TC and ABS lamps and two shoulder encoders. Research in `RESEARCH.md`, review and decisions in `HANDOVER.md`.

Penpot: page `07 · ValkyrieLmp1` (`0642f537-df59-8061-8008-fdd5a2ae4179`). Board v2 (current), row at y 13200: cockpit `01_idle` `347e5ae2-9e0b-80b1-8009-039897e91af7`, `02_mid_cruise` `…0398991a6b5d`, `03_redline_strobe` `…039899b6b71b`, `04_drift_braking` `…03989a5bc8ef`; hood `01` `…03989afee6b2`, `02` `…03989bb2ef63`, `03` `…03989c549c44`, `04` `…03989d129eac`; ideas and replacements `…03989dbcbdef`. Board v1 (superseded) is kept on the same page. Its place in the build queue is in [CATALOG.md](../../../../../program/phases/07-cluster-factory/CATALOG.md#build-order), the only place that records it.

## Files this dash owns
- `crates/fh6-clusters/src/clusters/valkyrie_lmp1/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/valkyrie_lmp1/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/valkyrie_lmp1.rs` (registered, empty until the dash needs bespoke assets).
- `tests/golden/images/ValkyrieLmp1_*.png` (its goldens) and its row in `docs/TELEMETRY_USAGE_MATRIX.md`.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the coordinator in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster ValkyrieLmp1
cargo test -p fh6-clusters valkyrie_lmp1
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters valkyrie_lmp1
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster ValkyrieLmp1 --stats
```
`preview-export --cluster` writes `mockups/ValkyrieLmp1/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
