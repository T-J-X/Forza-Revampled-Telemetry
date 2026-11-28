# Bosch DDU

Catalog row 12 · `ClusterTheme::BoschDdu` (config value `"BoschDdu"`) · Race Dash · cockpit · build #4 · ★ pattern-setter.

**What it is:** the Bosch Motorsport DDU 10 as BMW fits it to the M4 GT3: one black glass face edge to edge in a bead-blasted anodised aluminium body with an arched top, ten shift LEDs across the brow in groups of 2, 6 and 2, five LEDs down each side, and a 7" screen with the round rev gauge in the middle, the gear in a beam of light under it and four quarter panels. It carries no Bosch marks: the chin shows the car's logo and the driver's name. Research and sources: [`RESEARCH.md`](RESEARCH.md). Board: Penpot page `07 · BoschDdu`.

## Files
- `mod.rs`: the zone table (`Geom`), the option, the brow LEDs' logic (`TopLeds`), the cluster (`update`, `draw`) and every lit part.
- `hardware.rs`: the Blender housing renders (three key-light azimuths, cross-faded by the virtual light) and the procedural fallback.
- `spec.md` (the catalog brief; where it differs from the real display, `RESEARCH.md` wins), `RESEARCH.md`, `HANDOVER.md`.
- Assets: `assets/clusters/bosch_ddu/` with its `ASSETS.md`, built by `tools/asset-forge/src/recipes/bosch_ddu.rs`.
- Goldens: `tests/golden/images/BoschDdu_*.png`.

## Option (config extra key, hot-reloaded)
| Key | Values | Default |
|---|---|---|
| `BoschDduPanels` | four entries, comma-separated, for the panels left top, left bottom, right top, right bottom: `Pedals`, `G`, `Engine`, `Speed`, `Off` | `Pedals,G,Engine,Speed` |

Colours come from the palette (factory, custom or random): the screen uses `primary` as its one accent, the brow LEDs the rev stages and `shift`. The figures follow the user's display font. The logo zone shows the car's logo, else the brand or custom name. Power is shown in the user's power unit; boost follows the speed unit (psi with mph, bar with km/h).

## Commands
```
cargo run --release -p asset-forge -- --lane cloud --masters bosch_ddu build --set bosch_ddu
cargo run --release -p asset-forge -- cloud down
cargo run --release -p preview-export -- --cluster BoschDdu
cargo test -p fh6-clusters bosch_ddu
cargo test -p fh6-clusters --test dash_gates
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters bosch_ddu
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster BoschDdu --stats
```
The asset build, the renders, the goldens and the demo are GPU jobs: hold the lock of the lane you run on (`SKILLS.md` section 5). The dash gates run on the CPU.
