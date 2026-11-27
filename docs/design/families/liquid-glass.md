# Family: Liquid Glass

**Pattern-setter:** ★ `LiquidGlass` (the default cockpit theme and the design flagship), rebuilt on 2026-11-27 · **Members:** `LiquidGlass`, `VisionSpatialGlass`; new: `LumenEtch`, `VernierLoupe`, `DichroicLouvre`, `FibreOpticCorona` · **Reference build:** [`crates/fh6-clusters/src/clusters/liquid_glass/`](../../../crates/fh6-clusters/src/clusters/liquid_glass/) (its `README.md`, `RESEARCH.md` and `HANDOVER.md`) · **Optics:** [`docs/specs/optics.md`](../../specs/optics.md) · **Also owns:** the OSD material ([OSD_DIRECTION.md](../OSD_DIRECTION.md)), which still uses the legacy slab glass until it is rebuilt.

The hero frames in [`heroes/liquid-glass/`](../heroes/liquid-glass/) and the template image show the rejected first pilot (one capsule). Use the reference build's renders instead.

## Language
Separate drops of clear glass float over black, one per instrument, and every drop is a real optical object: the kit's `OpticalGlass` builds a surface from the piece's outline and computes, per pixel, the exact Fresnel reflection of one studio, Snell refraction of what lies under the piece and absorption along the path (optics.md). Nothing is painted to look like glass.

On OLED black, clear glass shows only where its edges mirror a light; studio photographers call this dark-field lighting. So the family rests on four things:
- **Light lives under the glass.** Shift lights, the rev ribbon and the numerals are drawn first; the glass bends them.
- **A dome is a lens.** Pieces are domes, as drops of liquid are. A dome over a numeral really magnifies it (the pilot's gear is seen about 1.15 times larger).
- **Thin rims from one studio.** The studio is a horizon ring and two thin strips, so every outline shows as one thin line plus a shorter one inside it. The light rig turns 25° per g of lateral G, so all the highlights travel together; the key light also leans 6° per g of acceleration, while the environment stays level so its dark cone stays on the viewer.
- **Liquid behaviour.** Pieces merge and split by smooth union (optics.md §2): they fuse on events and materialise in place on show, and a surface can ripple.

**Is not:** a slab with blobs on it, a frosted or tinted panel, a painted rim, sheen or fresnel band, a texture inside the glass, a caustic wallpaper, or colour splitting that the glass's thickness cannot produce.

## Layout grammar
- One piece per instrument, each its own zone in the `Geom` table, with its rise and base. No backing panel.
- Numerals sit where their lens tilts less than 22°. The studio leaves a 44° cone round the viewer dark (a face tilted θ mirrors the direction 2θ off the view axis), so faces tilted under 22° mirror black and no reflection crosses a numeral. The pilot unit-tests this from the profile, and its recipe from the studio's lights.
- Pieces at rest stand at least 3 blend lengths apart, so drawing them separately does not show. A smooth union bulges by k ln(1 + e^(−gap/k)), and two pieces touch at k = gap / (2 ln 2).
- The strobing shift strip is at most 42 px tall.
- See the pilot's zone table (the top of its `mod.rs`) for a worked cockpit and hood layout.

## Materials and light
- Token block `glass.optics`: synthetic sapphire (n_d 1.768), clarity 0.96 over 80 px, a slight lean to the palette's primary, no frost. Start from `tokens.optical_glass(&palette)`.
- Glass over numerals keeps the clarity but drops the hue lean (the pilot's `clear` material), so white stays white through 100 px of glass.
- Environment: a studio map rendered in Blender (key strip, far strip, horizon ring, and a cyclorama carrying a softened ComfyUI panorama), with the kit's analytic studio as the fallback. The rig's light colour leans 40% to the palette's primary, like a cabin's ambient light.
- A black core (alpha 0.92) under each piece that holds numerals keeps them readable over a bright cabin or bonnet. Smoked glass would dim the numerals as much as the picture behind them.
- Dispersion is real but under a pixel at these thicknesses; do not fake it.

## Type and colour
- Gear `display` 700 (Saira Condensed); speed Saira 500 with tabular figures; values Saira 400; labels and units Titillium Web 600 caps, at least 22 px.
- Brightness follows the rank: the gear is the brightest thing on the dash, the lit shift lights bloom, the rev ribbon burns at half strength with a full-strength head, secondary data (power, boost, rpm) is at 86% of the ink, and the parts that never change (labels, units, the logo) stay at or under the tokens' 20% luminance cap for static parts (`motion.idle.staticLumaMaxPct`), which still leaves a label 4.8:1 on black. Dim them by their light (`Color::intensity`), not their alpha: the overlay blends coverage in encoded space, so a label at 22% alpha on nothing shows at about 3%.
- Palette roles: `primary` is the ribbon and the light rig's lean, `shift` is the limiter strobe (default violet `#978DFC`), `warning` is the redline and the hot bead. A numeral's colour never changes on the limiter.

## Motion and signature moments
- **Limiter:** from the shift point only the shift strip strobes (10 Hz), on a clock that starts at the shift point so its first 50 ms are lit, and on the limiter its pieces fuse into one bar of glass. The steady cue is the bead: its light turns white with a warning halo. Every readout holds its digits by hysteresis (`fh6_kit::Hysteresis`, as the speed does), because the engine bounces about 50 rpm a frame on the limiter and a plain rounded readout flickers.
- **Upshift:** a ripple runs out through the rim of the gear lens and wobbles its reflections; the numeral stays still for the glance that confirms the shift.
- **Braking:** the side pieces reach for the centre piece and fuse with it, on the tokens' slosh spring.
- **Show:** materialise over 200 ms by growing the thickness, never the opacity. **Hide:** 1 frame (the host).
- **Reduced flash:** the strobe becomes a breath, and the ripple and other transient light are left out.

## Views
Cockpit and hood. On the hood the scrim is off, so the black cores do the work of keeping numerals readable over a bright bonnet.

## Do / Don't
- Do draw every light first and all the glass in one `LayerOptions::glass(0.0)` layer: it bends the scene as it was when the layer began, and one layer keeps the pass count low.
- Do use `Color::intensity` for segmented light (opaque, so joints do not double up) and radial gradients for halos on black (`p.glow` stops at 3 sigma, which shows as a square edge).
- Don't let the ribbon or the bead outshine the gear; don't strobe anything but the shift strip.

## Assets
The family's ComfyUI and Blender assets are environment, caustic and normal maps only (CLUSTER_PROMPT asset rules): no texture inside the glass. The pilot's are the Blender studio map with a ComfyUI panorama on its cyclorama, and a caustic ray traced in its recipe (SDXL could not draw one that tiles).

## References
Apple Liquid Glass (WWDC25 sessions 219, 323 and 356; lensing, the travelling highlight, materialise), CarPlay Ultra themes, Lucid Clearview (a floating OLED), and *Light: Science & Magic* chapter 7 on lighting glass (bright-field and dark-field). See the pilot's `RESEARCH.md`.
