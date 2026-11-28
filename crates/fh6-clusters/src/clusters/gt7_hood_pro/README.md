# GT7 Hood Pro

Catalog row 3 · `ClusterTheme::Gt7HoodPro` (config value `"Gt7HoodPro"`, the default hood theme) · Modern Digital 2027 · hood · ★ pattern-setter.

Status: **rebuilt as Gran Turismo 7's Bonnet View meter** (2026-11-28, thread "Rebuild Gt7HoodPro dash"). The order of work, the self-review and the numbers are in [HANDOVER.md](HANDOVER.md). The 2026-11-23 build (two mirrored light ribbons under a gull-wing hairline) is gone: it was a concept of its own, and a dash named after a real display has to be recognisable as that display ([RESEARCH.md](RESEARCH.md)).

## What it is
One low band of separate parts floating over the bonnet, with no panel behind it, laid out as GT7 lays out its compact meter: a straight rev bar of 95 LEDs on the centre line with the steering dot above it; under it the speed, a thin divider and the gear; a red key tile beside the gear that names the gear to take; a pedal bar either side; and on the left the tyre diagram, a small round gauge with the trip box under it. GT7 draws flat graphics. Here the parts are rendered hardware (one Blender plate, lit by the cluster's virtual light): a black-glass light bar on a black-chrome plinth, smoked LED lenses, a forged-carbon car chip, a polished boost scale, black-glass pedal slots and a red enamel key.

The two signature moments are GT7's rev alert and the hand-over of the gear. Near the shift point the whole lit run takes the `shift` colour at once; at the shift point it strobes and the red tile pops up with the next gear. When the driver takes that gear, the tile brightens for one frame and folds shut towards the gear while the new gear rises into place.

## How the module is laid out
| Part | Where | Notes |
|---|---|---|
| Zone table | the module doc of `mod.rs`, then `Geom` and `HOOD` in `geom.rs` | every size and position in local 4K px round the hood anchor; `fit` scales the band into another view's envelope; tests keep the zones inside the envelope and apart, and one test reads the asset recipe and fails if the two drift |
| Hardware | `hardware.rs` | `Sprites` (the plate and the tile at three light azimuths), `light_weights` (the cross-fade), `fallback_plate` and `fallback_tile` (drawn when a file is missing), and the two lamp icons |
| Faces | `Faces` in `mod.rs` | resolved once in `new()`; a user display font replaces the numerals |
| State | `Gt7HoodPro`, `RevBar`, `Tile` | the LED run with its backlash and strobe clock, the tile's pop, flash and fold, the shared components and the smoothed inputs |
| `update` | `impl Cluster` | copies the signals, advances the run, the tile, the steering dot, the lamps and the trip; no allocation |
| `draw` | `draw_rev`, `draw_steer`, `draw_pedals`, `draw_tyres`, `draw_boost`, `draw_trip`, `draw_lamps`, `draw_readout`, `draw_tile`, `draw_logo` | the plate first at full strength, then the live fills in its black-glass channels, then the readouts; every glyph, the divider and the logo carry a black keyline, and nothing sits on a box |

Shared components it composes (`crate::common`): `HeroGear`, `TabularSpeed`, `VirtualLight` and `gball::felt_force`. Its own helpers: `RevBar` (GT7's alert: the whole run changes colour, and the strip strobes on its own clock from the shift point so it starts lit and keeps a fifth of its light in the dark phase), `Tile` (the suggested gear), `backlash` (the head LED holds still while the engine bounces on the limiter), `tyre_load` (weight transfer), `static_ink` (the static cap, by light and not by coverage) and the keyline helpers.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/gt7_hood_pro/`
- `tests/golden/images/Gt7HoodPro_hood_*.png`
- `assets/clusters/gt7_hood_pro/` and its ledger `ASSETS.md`: `plate_0.png` to `plate_2.png` and `suggest_0.png` to `suggest_2.png` (Blender, at the light's rest azimuth and its ±25° swing). The forged-carbon texture (ComfyUI) is rendered into the plate and has no runtime file. Without the files the cluster draws its procedural hardware.
- `tools/asset-forge/src/recipes/gt7_hood_pro.rs` (builds all of them; the 4K masters go to the Drive folder `gt7_hood_pro\`)
- Penpot page `07 · Gt7HoodPro`

## Commands
```
cargo run --release -p preview-export -- --cluster Gt7HoodPro
cargo test -p fh6-clusters gt7_hood_pro
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters gt7_hood_pro
cargo run --release -p asset-forge -- --lane cloud --masters gt7_hood_pro build --set gt7_hood_pro
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster Gt7HoodPro --stats
```
The renders, the goldens, the live demo and the asset build are GPU jobs: hold the lock of the lane they run on (`SKILLS.md` section 5).
