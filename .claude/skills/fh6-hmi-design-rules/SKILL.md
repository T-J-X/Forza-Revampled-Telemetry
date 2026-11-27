---
name: fh6-hmi-design-rules
description: Design rules for the FH6 WheelDash instrument clusters, Unified OSD and OEM dash blackout scrim on an LG C1 4K OLED. Use for any task that draws, restyles or reviews a cluster, the OSD, the OEM scrim, or exports or critiques mockups.
---

# FH6 HMI design rules

## The 7 rules
1. **Change the least that answers the problem.** Keep the anchors at `(1920, 1835)` for Cockpit and `(1920, 1760)` for Hood, and keep the working 324-byte telemetry decoder.
2. **A new element copies its nearest sibling.** Match typography, corner radii and 1 px specular borders so everything reads as one OEM motorsport design system.
3. **One thing leads.** Rank order:
   1. Hero gear plus the shift light / rev-limiter strobe
   2. Zero-jitter tabular speed (`000` / `888` ghost grid)
   3. Tach arc, light ribbon or chevron bar
   4. The visualisation under the speedo: TeslaAdas 3D traffic, the WindshieldHud road-map ribbon, Gt7HoodPro tyre slip and G
   5. Bezel, catchlights and micro-labels
4. **Things sit beside what they are about.** The road or map visual goes directly under the speedometer and gear pod.
5. **Use the lines already there.** Align to the steering-column centreline `X = 1920` and to symmetric vanishing points.
6. **The same thing looks the same.** Manufacturer colours come from `assets/car_brands/car_brands.json` (AutoBrand), across clusters, shift lights, the ego car and the OSD.
7. **LG C1 OLED first.** Use true `#000000` for off pixels (VFD cavities, 100% OEM blackout), or ultra-translucent glass at alpha 28..64 with crisp 1 px rims. Never a muddy grey or navy opaque box, and never a full-canvas fill.

## Glanceability
Hero gear, speed and shift lights must read in under 150 ms in peripheral vision at 220+ MPH.

## Views
- **Cockpit `[3]` (wheel-less):** the cluster sits in the steering-column gap, and the OEM dash is covered by the domed scrim (`OemDashOpaquePct`, 0–100%).
- **Hood `[4]`:** `BonnetHudStyle` (default `Gt7HoodPro`), with the scrim forced to 0%.
- **Camera ring:** RB cycles `(i + 1) % 6`.
- **OSD:** hold View `0x0020` for 400 ms by default; remappable. It is never bound to A, B, X or Y.

## OEM scrim
A domed binnacle shape, not a rectangle. The inner core is 100% `#000000` at alpha 255 and covers about 80% of the area behind the cluster. A feathered skirt fades from alpha 255 to 0, and the steering-column gap is masked out.

## Cluster briefs
- `BoschDdu`: carbon weave, CARTEK 16-LED pod
- `LiquidGlass`: CarPlay Ultra 18–24% sapphire, rim `#FFFFFF→#38BDF8`, light-ribbon tach and meniscus orb
- `WindshieldHud`: collimated +240 px, live road-map ribbon
- `TeslaAdas`: 3D road, ego car plus 3–4 traffic cars, FSD target boxes, blind-spot arcs
- `RetroCyber`: 45° chamfer, `#000` VFD, 2 px scanlines, 24-segment arc (14 cyan, 6 yellow, 4 magenta)
- `PorscheAnalog`: 992 GT3 dial
- `DriftGlass`: smoked glass and a G vector
- `Gt7HoodPro`: frameless swept tach, 4-corner tyre slip
- `AmgMbuxSuperscreen`: MBUX with a 3D map
- `LfaV10Nurburgring`: 10K titanium ring
- `PaganiHorologerie`: Swiss chronograph
- `F35HelmetHmd`: aviation HUD with articulated horizon
- `BugattiTourbillon`: coaxial skeleton
- `EvaBioSync`: NERV hex bars
- `KoenigseggGyro`: gyro horizon
- `ValkyrieLmp1`: FIA 15-LED
- `SynthwaveOutrun`: sun and grid
- `ChronoMultiverse`: warping ferrofluid
- `ChronoEventHorizon`, `NeuralCyberSynapse`, `Gt7ModularTripod`, `ArNavWayfinder`, `SolarisWarpCore`, `VisionSpatialGlass`, `GlyphDotMatrix`, `AeroPlasmaBlade`, `CymaticResonanceField`, `KineticLiquidMetal`, `ShibuyaChromaticPrisma`: see `docs/specs/phase-8-materials-and-shaders.md`

The full list of 29 themes is in `crates/fh6-core/src/theme.rs`.

## Hybrid rendering
Composite the cached PNG from `assets/` when it is present, and always have a procedural fallback; never fail on a missing asset. Take colours and typography from `data/design_tokens.json`, not hardcoded hex values.

## Reference images
- `assets/reference/cockpit_with_wheel.jpg`: stock in-car cockpit, the background for the in-cockpit compositor
- `assets/reference/target_wide_cockpit_fov.jpg`: the target wide FOV and low seat
