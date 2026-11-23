---
name: blender-hero-assets
description: Hero-grade Blender renders for FH6 WheelDash clusters through asset-forge's hero library - studio light rigs, camera matching to the 4K binnacle, the PBR material cookbook (machined, anodised, brushed and sunray metal, guilloché, polished and knurled bezels, carbon, piano black, sapphire, enamel, lume, LED caps, leather, alcantara), Cycles settings, colour management, render passes, G-swing light variants, sprites and glTF. Use whenever a cluster needs rendered physical hardware (bezels, dials, needles, crystals, housings, surrounds) or any Blender asset, and whenever a render "looks like vector UI" or the materials are invisible.
---

# Blender hero assets

The rule TJX set: **the rendered hardware dominates the frame.** Penpot boards are layout and
intent only; the render must look far richer than the board. Vectors are for live data
(ticks, numerals, needles' angle, digits, bars); everything physical is a full-strength
rendered plate.

Code lives in `tools/asset-forge/src/hero/` (Rust; bpy is a Rust string constant in
`hero/bpy.rs`). Never write `.py` files into the repo. Run Blender through `asset-forge` with
the **Bash** tool, never PowerShell. One 8 GB RTX 3070 Ti is shared with other chats: one GPU
job at a time, and `Scene::render` already retries once on failure.

## Why earlier attempts were invisible (don't repeat these)

1. **Luminance maps added at 10–12 % over vector shapes.** That is a tint, not a material. Render
   the part itself and composite it at 100 % (`BlendMode::Normal`, premultiplied).
2. **Top-down ortho of a flat face reflects only what is straight above it.** With only a 40°
   key, metal faces render black. The hero rig carries a broad overhead "sky" softbox; keep it.
3. **AgX/Filmic tone curves** grey out exact colours. Beauty plates use the **Standard** view.
4. **Tiny bevels.** A machined edge needs a fillet of ≥ 0.6 px and a 45° chamfer of ≈ 0.28 × the
   bezel width to catch a highlight at 4K. `BezelSpec::ring` has these proportions built in.

## The workflow

```rust
use crate::hero::{frame::{self, Frame}, geom::BezelSpec, mat::{self, metal}, part::Part, Scene};
let mut s = Scene::new("my_cluster_hw", Frame::envelope(&frame::COCKPIT));
s.add(Part::face("tach_face", [0.0, -14.0], 0.0, 228.0, mat::sunray([0.30, 0.31, 0.33])));
s.add(Part::bezel("tach_bezel", [0.0, -14.0], 0.0, BezelSpec::ring(226.0, 18.0), [
    mat::polished(metal::TITANIUM), mat::spun(metal::TITANIUM),
    mat::brushed(metal::GUNMETAL), mat::knurled(metal::TITANIUM, 5.0, false)]));
let out = s.render(&ctx.paths, &dir)?; // beauty_0..2.png, normal/height/depth/ao/rmh/emission
```

Then save the PNGs through `ctx.image(...)` so the ledger records them, and draw them in the
cluster with `ImagePaint { rect }` where `rect` = the frame's rect in local px.

Reference implementation: `hero/demo.rs` (PorscheAnalog proof:
`cargo run --release -p asset-forge -- hero-demo [--preview]`).

## Camera matching (`hero/frame.rs`)

- Units are **local 4K px** round the anchor, x right, y **down**. Blender gets `PX = 0.01` BU per
  px with y flipped; the library does the conversion — never convert by hand.
- Cockpit anchor (1920, 1835), envelope (−720, −320, 1440, 560). Hood anchor (1920, 1760),
  envelope (−880, −220, 1760, 400). `Frame::envelope(&frame::COCKPIT)` renders the whole cluster.
- **Ortho** (default) is pixel-exact: `ortho_scale = res_x / scale × PX`, so a plate drops into
  `ImagePaint { rect }` with no fitting. Use it for everything the kit composites.
- **Persp** (`.persp(lens_mm, tilt_deg)`) only for hero stills and turntables; the z = 0 plane
  stays registered at tilt 0, proud parts get parallax.
- Sprites (needles, LED caps): `Frame::square(0, 0, size)`, pivot at the image centre, pointing
  **up** (−y). The kit rotates them.
- `.scaled(0.5)` for previews: same coverage, half the pixels.

## Light rigs (`hero/rig.rs`)

- Azimuth = degrees clockwise from 3 o'clock on screen (the kit's `VirtualLight` convention);
  −135° = upper-left = the kit's rest key. Elevation in degrees above the dial plane.
- Power is calibrated: power 1.0 straight above a white diffuse plane ≈ 0.8 linear, at any
  distance (energy = power × 10 × d²). Think in stops, not watts.
- `Rig::hero(span, key_az)`: hard key (el 38°, power 2.6), overhead sky softbox (el 76°, 1.6 ×
  span), second smaller sky opposite (sheen gradient), cool low rim strip opposite the key, warm
  low kicker at +95°, dim `studio.exr` HDRI (camera-invisible) for curved reflections.
- `Rig::raking(span, az)`: one low light for relief proofs (guilloché, leather grain).
- **G swing:** the kit has no normal-map relighting. `Scene::swing` (default −160°, −110°)
  renders extra beauties with every light turned; the cluster crossfades `beauty_0/1/2` by
  `VirtualLight` azimuth (RetroCyber's pattern). Normal/rmh passes are rendered for the day the
  kit gets a lit-sprite pass.
- Lights are invisible to camera; nothing but the hardware ever lands in the alpha.

## Material cookbook (`hero/mat.rs`)

Colours are **linear**. Metals use measured F0 (`mat::metal::{ALUMINIUM, TITANIUM, CHROME,
STEEL, NICKEL, GOLD, COPPER, BLACK_CHROME, GUNMETAL}`), never eyeballed greys. Patterns are
procedural in object space round the part's own centre — no UVs.

| Builder | Use | Key settings |
|---|---|---|
| `machined_alu()` | housings, bead-blasted parts | rough 0.28, isotropic grain bump |
| `anodised(tint)` | coloured alloy (red calipers, blue trims) | metal + tinted coat (Coat Tint), coat rough 0.18 |
| `brushed(base)` | linear grain plates | anisotropic 0.85, tangent +x, streak noise |
| `sunray(base)` | dial faces | radial tangent, aniso 0.95, rough 0.22 — **full strength, it is the dial** |
| `spun(base)` | lathe-turned bezel tops | concentric tangent, aniso 0.9 |
| `polished(base)` | chamfers, needles, hubs | rough 0.05 + 0.02 variation |
| `knurled(base, pitch, reeded)` | bezel outer chamfer, knobs | diamond / reeded bump round the axis |
| `guilloche(base, "clous"/"rose"/"wave", pitch)` | horology dials | bump depth 0.2 × pitch |
| `carbon(tow)` | 2×2 twill | tangent switches per tow, metallic 0.6, mirror coat |
| `piano_black()` | visors, wells, trims | albedo 0.004, coat rough 0 |
| `sapphire()` | crystals | IOR 1.77, 120 nm thin film at IOR 1.38 (AR sheen) |
| `glass(tint)` | lenses | IOR 1.5 |
| `enamel(color)` | grand-feu dials, badges | coat + subsurface 0.15 |
| `lume(color, glow)` | needle inserts, indices | emission strength `glow` (0 = daylight) |
| `led_cap(color, glow)` | shift-light jewels | PMMA IOR 1.49, transmission 0.85, emission |
| `leather(color, grain)` | hoods, surrounds | Voronoi crease grain + sheen; `.set("texture", path)` for a ComfyUI plate |
| `alcantara(color)` | suede trims | rough 0.92, sheen 1.0 |
| `matte`, `ink`, `emissive` | rubber, printed ink, backlit graphics | |

`hero-demo` renders `docs/handover-shots/asset-pipeline-v2-materials.png`, one puck per
builder — check a new material there first.

## Parts (`hero/part.rs`, `hero/geom.rs`)

`Part::bezel` (4 slots: inner chamfer, top, walls, outer chamfer), `face`, `chaptered_face`,
`ring`, `crystal`, `dome` (hub caps, jewels), `extrude` (housings, leather hoods; outline in
absolute px, pillow bevel), `mesh` (needles via `geom::blade`, stitching via
`part::stitches`). Lathe profiles take fillets (`Profile::round`); shapes from
`Outline::rrect` or `Outline::circle_union` (smooth union of gauge pods — a binnacle shroud).

Stack heights in px above the dial plane: hood top ≈ −14, faces 0, wells/bearings 0.6, bezels
0..10, crystals 10–14. Keep crystals in a **separate glass scene** (reflections only, drawn by
the kit after the needle), as `hero-demo` does.

## Cycles and colour

- `gpu_cycles`: OptiX → CUDA → CPU, OptiX denoise, 16 bounces, caustics on, indirect clamp 20.
- Samples: 384 for 4K beauty, 64 for `--preview`, 16 for data passes (no denoise).
- View transform **Standard**, look None, exposure 0; never AgX for plates that must hit exact
  colours. Display sRGB.
- Film transparent + `film_transparent_glass` (roughness ≤ 0.1): glass over nothing stays clear,
  so crystals contribute only their reflections.
- PNG output is **straight (unassociated) alpha**, 16-bit, sRGB-encoded. The kit premultiplies on
  load; never premultiply twice.

## Passes (`Passes`)

| File | Content | Use |
|---|---|---|
| `beauty_0.png` | rest key (−135°) | the plate |
| `beauty_1/2.png` | swing keys | crossfade by lateral G |
| `normal.png` | tangent normal, OpenGL (+G = up the screen) | future relight, ComfyUI |
| `height.png` | 16-bit height (range from the parts) | parallax, bump |
| `depth.png` | 8-bit near = white on black | ControlNet depth |
| `ao.png` | ambient occlusion | contact darkening under vectors |
| `rmh.png` | roughness, metallic, height | kit material masks |
| `emission.png` | lights off, emitters only | Add layer for lume / LEDs at night |

## Baking, sequences and glTF

- **Bake** = render the ortho data pass; there is no UV unwrap step because every pattern is
  object-space procedural. For a tiling texture, render a `Frame::rect` over a flat part and run
  `img::make_seamless`.
- **Flipbooks / turntables**: render N scenes with `rot` stepped (`Part::rotated`) into
  `frame_###`, then `img::atlas`; check `img::loop_seam_score`.
- **glTF** for the kit's mesh pass: build the parts the same way, then export with
  `bpy.ops.export_scene.gltf(export_format='GLB', export_apply=True)` from a recipe body (see
  `recipes/adas.rs`). Keep triangle budgets in the ledger QA line.

## Done means

- The hardware reads at arm's length on the full-frame shot, with no zoom.
- Before/after and Penpot-vs-render images in `docs/handover-shots/`.
- Every output goes through `ctx.image` so `assets/ASSETS.md` records recipe, seed and licence.
