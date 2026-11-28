# Retro Cyber

Catalog row 19 · `ClusterTheme::RetroCyber` (config value `"RetroCyber"`) · Sci-Fi · cockpit+hood · build #6 · ★ pattern-setter (VFD dialect).

**What it is:** the Toyota Supra A70 "Panoramic Digital Meter" (1986 to 1993), the full-width vacuum-fluorescent panel, as a 2027 production revival. One long smoked lens covers separate VFD windows on black. From left to right: pedal columns (where the A70 has its fuel window), the centre window (a horizontal tach whose bars rise to the right, a red bracket over the red zone that is also the shift light, big blue-green seven-segment speed, the gear, and lateral G where the A70 has its trip), a slip-angle bar (where it has its temperature bar), and power and boost as slanted bars (where it has oil and turbo). Research, sources and the deliberate changes: [`RESEARCH.md`](RESEARCH.md). Board: Penpot page `07 · RetroCyber` (see `HANDOVER.md`).

## Signature moments
- **Ignition:** on the first observed `IsRaceOn` 0 → 1 after launch, the filaments warm for 150 ms, then 8 sectors light left to right (40 ms each) as an all-segments self-test and settle to ghost. Segments switch, they never fade. Gear and speed are live from frame 1.
- **Shift light:** the red bracket over the red zone fills from the shift point, then strobes magenta on the limiter (10 Hz, or the reduced-flash breath); the gear turns magenta with it.
- **Glass:** the kit's optical lens (clear, smoked by absorption, crisp rims) shows the studio's reflections, and its light swings with lateral G.

## Files
- `mod.rs`: the zone tables for both views (`Geom`), the tach scale, the gauges, the boot, the cluster (`update`, `draw`) and its tests.
- `spec.md` (the catalog brief, now superseded by `RESEARCH.md` where they differ), `RESEARCH.md`, `HANDOVER.md`.
- Assets: `assets/clusters/retro_cyber/` with its `ASSETS.md`, built by `tools/asset-forge/src/recipes/retro_cyber.rs`: the Blender surround (two key lights per view, cross-faded by the virtual light) over a ComfyUI graphite grain, and the ComfyUI phosphor plate. Each has a procedural fallback; the shared `assets/scifi/vfd_hex_mesh.png` gives the grid mesh.
- Goldens: `tests/golden/images/RetroCyber_*.png` and `RetroCyber_hood_*.png`.

Colours come from the palette (factory, custom or random); the logo zone under the tach legend shows the car's logo, else the Toyota wordmark on a Toyota, else the brand or custom name, and stays empty when `ShowLogo` is off. Power reads in the user's unit (hp or kW), speed in km/h or mph.

## Commands
```
cargo run --release -p asset-forge -- --lane cloud --masters retro_cyber build --set retro_cyber
cargo run --release -p asset-forge -- cloud down
cargo run --release -p preview-export -- --cluster RetroCyber
cargo test -p fh6-clusters retro_cyber
cargo test -p fh6-clusters --test dash_gates
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters retro_cyber
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster RetroCyber --stats
```
The asset build, the renders, the goldens and the demo are GPU jobs: hold the lock of the lane you run on (`SKILLS.md` section 5).
