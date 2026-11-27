# Porsche Analog

Catalog row 7 · `ClusterTheme::PorscheAnalog` (config value `"PorscheAnalog"`) · Classic Analog · cockpit · build #3 · ★ pattern-setter, and the reference for rendered physical hardware.

**What it is:** the Porsche 911 (992) five-tube cluster as it looks on the 992.2's Classic view: the rev counter in the middle with the speed and G tubes tucked behind it and the power and boost tubes tucked behind those. The rev counter uses Porsche's rotated Track scale (the shift rpm at 12 o'clock), the gear sits in a porthole under a sapphire magnifier in the numeral-free sector, and the GT3-style shift lights are set into the bezel. Research and sources: [`RESEARCH.md`](RESEARCH.md). Board: Penpot page `07 · PorscheAnalog`.

## Files
- `mod.rs`: the zone table (`Geom`), the options, the cluster (`update`, `draw`) and the rev counter's drawing.
- `scale.rs`: the 240° scale, classic or turned so the shift rpm stands at 12.
- `hardware.rs`: the Blender sprites (each rendered at several light azimuths and cross-faded by the virtual light) and the procedural fallbacks.
- `tubes.rs`: what the four other tubes show (speed, G, power, boost), the power and boost scale tops, and the driver greeting.
- `readout.rs`: digital readouts that refresh at 5 Hz with hysteresis, and the needle's hold through a bounce off the limiter.
- `tests.rs`: geometry, clearances, the tubes' tuck, the strobe band, the lens placement, the gear window and the logo zone on every scale, the magnifier's optics and the options.
- `spec.md` (the catalog brief), `RESEARCH.md`, `HANDOVER.md`.
- Assets: `assets/clusters/porsche_analog/` with its `ASSETS.md`, built by `tools/asset-forge/src/recipes/porsche_analog.rs`.
- Goldens: `tests/golden/images/PorscheAnalog_*.png`.

## Options (config extra keys, hot-reloaded)
| Key | Values | Default |
|---|---|---|
| `PorscheAnalogScale` | `Track` (the shift rpm at 12 o'clock), `Classic` (0 at 8 o'clock) | `Track` |
| `PorscheAnalogFarLeft`, `PorscheAnalogLeft`, `PorscheAnalogRight`, `PorscheAnalogFarRight` | `Speed`, `G`, `Power`, `Boost`, `Off` | `Power`, `Speed`, `G`, `Boost` |
| `PorscheAnalogNumerals` | `White`, `Heritage` (the 356 green) | `White` |

Colours come from the palette (factory, custom or random), the numerals follow the user's display font, and the logo zone shows the car's logo, else the Porsche wordmark from `assets/logos/` on a Porsche, else the brand or custom name; it stays empty when `ShowLogo` is off. The right tube greets the driver by the config's driver name while the car is parked in neutral.

## Commands
```
cargo run --release -p asset-forge -- --lane cloud --masters porsche_analog build --set porsche_analog
cargo run --release -p asset-forge -- cloud down
cargo run --release -p preview-export -- --cluster PorscheAnalog
cargo test -p fh6-clusters porsche_analog
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters porsche_analog
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster PorscheAnalog --stats
```
The asset build, the renders, the goldens and the demo are GPU jobs: hold the lock of the lane you run on (`SKILLS.md` section 5).
