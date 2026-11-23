# Retro Cyber

Catalog row 19 · `ClusterTheme::RetroCyber` (config value `"RetroCyber"`) · Sci-Fi · cockpit+hood · build #6 · ★ pattern-setter (VFD dialect).

Status: **built** (Phase 7). A vacuum-fluorescent tube in a chamfer45 gunmetal housing: phosphor seven-segment gear and speed with 7% ghosts, a 15-pod shift strip, a 24-chevron staircase, rank-4 columns, a dot G-scope, drift columns and annunciators. The hex control-grid mesh, five filaments and 2 px scanlines only multiply lit phosphor, so the cavity stays `#000000`. The brief is `spec.md`; the zone table and the full description are at the top of `mod.rs`; the handover is `HANDOVER.md`.

## Signature moments
- **Filament ignition boot** on the first observed `IsRaceOn` 0 → 1 after launch: filaments warm for 150 ms, then 8 sectors ignite left to right (40 ms each) as an all-segments self-test and settle to ghost (≈ 0.52 s). Gear and speed are live from frame 1.
- **Gear change:** the digit re-strikes over 80 ms while a 1-frame 60% flash laps the octagon's 8 facets in 50 ms.
- **Limiter:** only the strip strobes (10 Hz, or the reduced-flash breath); the octagon rim holds steady accent; the gear pulses at 2 Hz after 0.5 s.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/retro_cyber/`: `mod.rs` (the `Cluster` impl and `make`), `spec.md` (the brief), this README and `HANDOVER.md`.
- `tests/golden/images/RetroCyber_*.png` and `RetroCyber_hood_*.png` (its goldens).
- `assets/clusters/retro_cyber/` (ledger `ASSETS.md`) and its recipe `tools/asset-forge/src/recipes/retro_cyber.rs`: the Blender-rendered chamfer housing (two key lights per view) and the ComfyUI phosphor-grain plate. Build with `cargo run --release -p asset-forge -- build --set retro_cyber`. Each has a procedural fallback; the shared Phase 6 `assets/scifi/vfd_hex_mesh.png` gives the control grid.

Everything else (catalog, registry, theme enum, `common/`, the golden harness, the tools) is shared: ask the conductor in your handover.

## Commands
```
cargo run --release -p preview-export -- --cluster RetroCyber
cargo test -p fh6-clusters retro_cyber
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters retro_cyber
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster RetroCyber --stats
```
`preview-export --cluster` writes `mockups/RetroCyber/`: the 4 review states at 4K (cockpit and hood), 1:1 crops, thumbnails, the cockpit composite, a 60-frame sequence (idle → redline strobe → gear change) and an `index.html` with GPU and colour-budget badges. GPU at 4K: sequence p50 0.32 ms, p99 0.33 ms. Design board: Penpot page `07 · RetroCyber` (see `HANDOVER.md`).
