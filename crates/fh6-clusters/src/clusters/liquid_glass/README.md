# Liquid Glass

Catalog row 1 · `ClusterTheme::LiquidGlass` (config value `"LiquidGlass"`, the default cockpit theme) · Liquid Glass · cockpit+hood · build #1 · ★ pattern-setter.

Status: **built** by the Phase 7 setup chat as the pilot. See [HANDOVER.md](HANDOVER.md) for renders, numbers, the critique and "my calls", and [spec.md](spec.md) for the brief.

## What it is
One sapphire capsule (a 1240 × 360 squircle, n 5) with no wallpaper. The gear sits under a domed lens, the speed floats on the left, a thin light pipe runs round the rim from lower-left over the crown to lower-right with a meniscus bead at the rpm, 16 glass pills on a curve under the crown are the shift lights, and a pendant glass drop under the centre holds the caustic G spot.

## How the module is laid out (the pattern for every cluster)
| Part | Where | Notes |
|---|---|---|
| Zone table per view | `Geom`, `COCKPIT`, `HOOD` in `mod.rs` | every size and position in local 4K px; a unit test keeps it inside `layout.envelope` |
| Faces | `Faces` | resolved once in `new()`; a user display font replaces the numerals |
| Paths | `Paths` | the light pipe and the two light-run halves, sampled once per view |
| State | `LiquidGlass` | only springs, smoothers and the shared components |
| `update` | `impl Cluster` | copies signals, advances components; no allocation |
| `draw` | `impl Cluster`, `draw_spot`, `draw_run` | glass first, then quiet structure, one bloom group for ranks 1–3, then labels |

Shared components it composes (`crate::common`): `GlassPlate` (squircle, dimming pad, hood halo), `PathRibbon` (light pipe, bead), `ShiftLights` with the pod optics, `LimiterCue`, `HeroGear`, `TabularSpeed`, `logo_zone`.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/liquid_glass/`
- `tests/golden/images/LiquidGlass_*.png` and `LiquidGlass_hood_*.png`
- `assets/clusters/liquid_glass/` (none needed: it uses the shared `assets/glass/caustic_lens_circle.png` when present and a procedural ring otherwise)

## Commands
```
cargo run --release -p preview-export -- --cluster LiquidGlass
cargo test -p fh6-clusters liquid_glass
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters liquid_glass
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster LiquidGlass --stats
```
