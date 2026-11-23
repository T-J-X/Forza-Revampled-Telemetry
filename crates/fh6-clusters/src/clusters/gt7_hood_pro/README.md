# GT7 Hood Pro

Catalog row 3 · `ClusterTheme::Gt7HoodPro` (config value `"Gt7HoodPro"`) · Modern Digital 2027 · hood · build #2 · ★ pattern-setter.

Status: **built** (Phase 7 cluster chat, see `HANDOVER.md`). A frameless gull-wing horizon: twin mirrored light ribbons climb from the wingtips to a crest over the gear and fuse at the car's shift rpm (the "apex fuse"), 11 converging diamonds strobe at the limiter, the gear sits in a sculpted OLED-black glass pod with four slip capsules on its corners, the speed below, the G puck right and THR / BRK left. Hood view first (no scrim), with a cockpit geometry for forced views.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/gt7_hood_pro/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/gt7_hood_pro/` (`pod_lens.png` from Blender, `ribbon_grain.png` from ComfyUI, with `ASSETS.md`) and `tools/asset-forge/src/recipes/gt7_hood_pro.rs` (`cargo run --release -p asset-forge -- build --set gt7_hood_pro`). Both assets are optional at runtime.
- `tests/golden/images/Gt7HoodPro_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster Gt7HoodPro
cargo test -p fh6-clusters gt7_hood_pro
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt7_hood_pro
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt7HoodPro --stats
cargo run --release -p asset-forge -- build --set gt7_hood_pro
```
`preview-export --cluster` writes `mockups/Gt7HoodPro/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
