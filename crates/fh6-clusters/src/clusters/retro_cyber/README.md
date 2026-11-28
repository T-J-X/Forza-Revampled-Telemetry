# Retro Cyber

Catalog row 19 · `ClusterTheme::RetroCyber` (config value `"RetroCyber"`) · Sci-Fi · cockpit+hood · build #6 · ★ pattern-setter (VFD dialect).

**What it is:** the Toyota Supra A70 "Panoramic Digital Meter" (1986 to 1993), the full-width vacuum-fluorescent panel, as a 2027 production revival. One long smoked lens, stepped like the A70's panel (a low strip across the width with a raised part over the centre), covers bare VFD windows on black, separated by a thin printed line under them. From left to right: pedal columns (where the A70 has its fuel window), the centre window (a horizontal tach whose bars rise to the right, a red bracket over the red zone that is also the shift light, big blue-green seven-segment speed, the gear, and lateral G where the A70 has its trip), a slip-angle bar (where it has its temperature bar), and power and boost as slanted bars (where it has oil and turbo). Research, sources and the deliberate changes: [`RESEARCH.md`](RESEARCH.md). Board: Penpot page `07 · RetroCyber` (see `HANDOVER.md`).

## Signature moments
- **Ignition:** on the first observed `IsRaceOn` 0 → 1 after launch, the filaments warm for 150 ms, then 8 sectors light left to right (40 ms each) as an all-segments self-test and settle to ghost. Segments switch, they never fade. Gear and speed are live from frame 1.
- **Shift light:** the red bracket over the red zone fills from the shift point, then strobes magenta on the limiter (10 Hz, or the reduced-flash breath); the gear keeps its white phosphor and takes a soft glow, so it holds its 7:1 glance contrast.
- **Glass:** the kit's optical lens (clear, smoked by absorption, crisp rims) shows the studio's reflections, and its light swings with lateral G.

## GPU budget
The PC timed each layer (RTX 3070 Ti Laptop, 28 Nov): the surround's full-panel fills, the lens and the speed's bloom were the costs. So the surround's art, face, rims and the lens black are drawn on twelve rectangles that cover only its ring (`Geom::ring_cover`, about 17% of the panel; a test checks every ring pixel of both views), and the speed and gear glow with local halos (`Emission::Halo`, `Emission::SoftHalo`): no frame of this dash blooms, so the work rectangle never grows by the bloom margin. The grid mesh and the scanlines go only where phosphor can light (`Geom::mesh_zones`: the side windows, the tach band in six steps along its curve, the speed and gear cells with their halos), not over the centre window's black. The lens glass stays whole: the kit draws glass over its full bounds even under a clip.

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
