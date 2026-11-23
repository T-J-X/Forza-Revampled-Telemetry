# Neural Cyber Synapse

Catalog row 23 · `ClusterTheme::NeuralCyberSynapse` (config value `"NeuralCyberSynapse"`) · Sci-Fi · cockpit · build #23.

Status: **scaffold** (Phase 7 setup). `mod.rs` draws the shared under-construction frame from `common::scaffold` until the cluster chat replaces it.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/neural_cyber_synapse/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README, `HANDOVER.md`, and any private submodules.
- `assets/clusters/neural_cyber_synapse/` (with its own `ASSETS.md`) and `tools/asset-forge/src/recipes/neural_cyber_synapse.rs`, if it needs bespoke assets.
- `tests/golden/images/NeuralCyberSynapse_*.png` (its goldens).

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster NeuralCyberSynapse
cargo test -p fh6-clusters neural_cyber_synapse
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters neural_cyber_synapse
cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster NeuralCyberSynapse --stats
```
`preview-export --cluster` writes `mockups/NeuralCyberSynapse/`: the 4 review states at 4K (cockpit), 1:1 crops, thumbnails, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges.
