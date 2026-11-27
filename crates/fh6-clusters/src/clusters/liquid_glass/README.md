# Liquid Glass

Catalog row 1 · `ClusterTheme::LiquidGlass` (config value `"LiquidGlass"`, the default cockpit theme) · Liquid Glass family · cockpit and hood.

Status: **rebuilt on optical glass, second build** (2026-11-27). TJX's answer to the direction check is open; the verification, the handover and the sign-off follow it. The first pilot (one dark capsule with a painted rim) was rejected on 2026-11-27 and is gone; [RESEARCH.md](RESEARCH.md) §4 says why.

## What it is
Separate drops of clear glass float over black, and every one is a lens: 16 cabochons (the shift lights), a graduated glass rod with a bead (the rev ribbon), a dome that magnifies the gear, two shallow capsules (speed; power and boost) and a small lens (the G light). Nothing is painted to look like glass. Each piece is `fh6_kit::OpticalGlass`: a surface built from its outline, lit by one studio rig and computed per pixel from the equations in [docs/specs/optics.md](../../../../../docs/specs/optics.md). On black a piece shows as the thin lines where its edge mirrors a light, and it bends whatever sits under it: the gear is drawn at cap 156 and seen about 1.15 times larger. The glass is liquid too: an upshift sends a ripple across the gear dome, the limiter fuses the cabochons, and hard braking makes the capsules reach for the dome.

## How the module is laid out
| Part | Where | Notes |
|---|---|---|
| Zone table per view | the module doc, then `Geom`, `COCKPIT`, `HOOD` in `mod.rs` | every size and position in local 4K px, with each piece's glass section; unit tests keep it inside the view envelope and keep the pieces apart |
| Faces | `Faces` | resolved once in `new()`; a user display font replaces the numerals |
| State | `LiquidGlass` | springs, tweens, the shared components and this frame's glass material |
| `update` | `impl Cluster` | copies signals, advances the springs, aims the light rig; no allocation |
| `draw` | `draw_cores`, `draw_rev`, `draw_shift`, `draw_gear`, `draw_readouts`, `draw_g`, `draw_glass` | a black core under each piece that holds numerals, then every light, then all the glass in one layer |

Shared components it composes (`crate::common`): `ShiftLights`, `LimiterCue`, `HeroGear`, `TabularSpeed`, `VirtualLight`, `logo_zone`.

## Files this cluster owns
- `crates/fh6-clusters/src/clusters/liquid_glass/`
- `tests/golden/images/LiquidGlass_*.png` and `LiquidGlass_hood_*.png`
- `assets/clusters/liquid_glass/` and its ledger `ASSETS.md`: `studio_env.png` (the studio the glass mirrors: Blender, with a ComfyUI panorama on its cyclorama) and `liquid_light.png` (the caustic in the rev ribbon, ray traced). Without them the cluster uses the kit's analytic studio and a plain ribbon.
- `tools/asset-forge/src/recipes/liquid_glass.rs` (builds both assets; the 4K masters go to the Drive folder `liquid_glass\`)
- Penpot page `07 · LiquidGlass`

## Commands
```
cargo run --release -p preview-export -- --cluster LiquidGlass
cargo test -p fh6-clusters liquid_glass
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters liquid_glass
cargo run --release -p asset-forge -- build --set liquid_glass
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster LiquidGlass --stats
```
The renders, the goldens, the live demo and an asset build that finds a stale ComfyUI or Blender step are GPU jobs: hold the GPU lock (`SKILLS.md` section 5).
