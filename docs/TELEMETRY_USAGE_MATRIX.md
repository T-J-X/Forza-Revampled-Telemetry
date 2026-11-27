# Telemetry usage matrix

Which telemetry inputs change each dash's frame. This file is the declaration that gates G2 and G3
check (`docs/PROGRESS.md`): `crates/fh6-clusters/tests/dash_gates.rs` builds every dash in the
registry on wgpu's no-op backend (no GPU and no GPU lock), probes each input below in each
scenario, and fails when:

- a listed input does not change the dash's frame (G2: a claim the dash does not keep),
- an input that is not listed does change it (G3: the matrix is out of date),
- a registry dash has no row, a row names a dash or an input that does not exist, or the
  Inputs table here differs from the test's `INPUTS` table.

"Changes the frame" means the scene fingerprint differs (`Scene::fingerprint` in `fh6-kit`: every
instance, paint, clip, point, draw and mesh the GPU would draw, bit for bit), after 21 updates
with the probe applied, compared with a fresh build of the same dash fed the same scenario
without it. The test also checks that two fresh builds fed the same packets draw the same frame.

Rebuilt on 2026-11-27 in Phase G from what the gate observes. It replaces the hand-audited v1
snapshot of 2026-11-19, which covered the v1 tiny-skia clusters.

## How a dash plugs in

- A dash phase that rebuilds a dash updates its row in the same commit. On a mismatch the test
  prints the observed row; copy it only after checking that each change is one the dash means
  to make (the module docs list the telemetry a dash claims).
- An input the dash reads only together with another (a lamp with two conditions, a rate) needs
  a scenario where the other conditions hold: add it to `DASH_SCENARIOS` in the test. Bosch DDU's
  four are the example.
- `car_ordinal` reaches every dash through the palette, which the host picks
  (`fh6-app` `render_loop.rs`, `ensure_palette`); the gate gives each dash a fixed palette, so a row
  lists `car_ordinal` only when the dash reads it itself. `race_on` and a stale stream hide the dash
  in the host (AGENTS.md directive 9); a row lists `race_on` only when the dash draws differently
  for it.

## Inputs

The probe values are in the test's `INPUTS` table. Together the inputs cover every field of
`FH6TelemetryData`, and the test fails to compile when a field is added without one.

| Key | Packet fields |
|---|---|
| `race_on` | is_race_on |
| `timestamp` | timestamp_ms |
| `rpm` | current_engine_rpm |
| `max_rpm` | engine_max_rpm |
| `idle_rpm` | engine_idle_rpm |
| `gear` | gear |
| `speed` | speed_ms |
| `throttle` | accel |
| `brake` | brake |
| `clutch` | clutch |
| `handbrake` | hand_brake |
| `steer` | steer |
| `g_lat` | acceleration_x |
| `g_long` | acceleration_z |
| `g_vert` | acceleration_y |
| `velocity` | velocity_x, velocity_y, velocity_z |
| `yaw_rate` | angular_velocity_y |
| `orientation` | yaw, pitch, roll |
| `tyre_slip` | tire_slip_angle_fl, _fr, _rl, _rr |
| `suspension` | normalized_suspension_travel_fl, _fr, _rl, _rr |
| `suspension_m` | suspension_travel_meters_fl, _fr, _rl, _rr |
| `car_ordinal` | car_ordinal |
| `drivetrain` | drivetrain_type |
| `position` | position_x, position_y, position_z |
| `power` | power_watts |
| `boost` | boost_psi |

## Scenarios

Every dash is probed in five: cruise (the mid-cruise review state), idle, redline and drift
braking (the review states in `fh6_clusters::review::STATES`), each with the fields a review state
leaves at zero (timestamp, suspension, car ordinal) set to a plausible reading.

Dash scenarios (`DASH_SCENARIOS`):

| Dash | Scenario | Makes reachable |
|---|---|---|
| Bosch DDU | launch, nothing held | `brake`, `clutch` (LNCH: standing, throttle over 50% and something held) |
| Bosch DDU | launch, brake held, throttle off | `throttle` (LNCH) |
| Bosch DDU | threshold braking, brake off | `brake` (ABS: brake over 30% at 0.85 g deceleration) |
| Bosch DDU | power oversteer, front drive | `drivetrain` (TC: throttle and growing body slip, not front drive) |

## Dashes

Six dashes have their own code today (Liquid Glass, GT7 Hood Pro, Porsche Analog, Bosch DDU,
Tesla ADAS, Retro Cyber). The other 42 draw the shared scaffold (`common::scaffold`), which reads
rpm, the limiter, gear and speed.

| Theme | Dash | Reads |
|---|---|---|
| `LiquidGlass` | Liquid Glass | `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `g_lat`, `g_long`, `velocity`, `power`, `boost` |
| `VisionSpatialGlass` | Vision Spatial Glass | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt7HoodPro` | GT7 Hood Pro | `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity` |
| `AmgMbuxSuperscreen` | AMG MBUX Superscreen | `rpm`, `max_rpm`, `gear`, `speed` |
| `DriftGlass` | Drift Glass | `rpm`, `max_rpm`, `gear`, `speed` |
| `GlyphDotMatrix` | Glyph Dot Matrix | `rpm`, `max_rpm`, `gear`, `speed` |
| `PorscheAnalog` | Porsche Analog | `race_on`, `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `power`, `boost` |
| `PaganiHorologerie` | Pagani Horologerie | `rpm`, `max_rpm`, `gear`, `speed` |
| `BugattiTourbillon` | Bugatti Tourbillon | `rpm`, `max_rpm`, `gear`, `speed` |
| `LfaV10Nurburgring` | LFA V10 Nürburgring | `rpm`, `max_rpm`, `gear`, `speed` |
| `KoenigseggGyro` | Koenigsegg Gyro | `rpm`, `max_rpm`, `gear`, `speed` |
| `BoschDdu` | Bosch DDU | `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `clutch`, `handbrake`, `g_lat`, `g_long`, `velocity`, `drivetrain`, `power`, `boost` |
| `ValkyrieLmp1` | Valkyrie LMP1 | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt7ModularTripod` | GT7 Modular Tripod | `rpm`, `max_rpm`, `gear`, `speed` |
| `TeslaAdas` | Tesla ADAS | `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `brake`, `g_lat`, `g_long`, `velocity` |
| `WindshieldHud` | Windshield HUD | `rpm`, `max_rpm`, `gear`, `speed` |
| `ArNavWayfinder` | AR Nav Wayfinder | `rpm`, `max_rpm`, `gear`, `speed` |
| `F35HelmetHmd` | F-35 Helmet HMD | `rpm`, `max_rpm`, `gear`, `speed` |
| `RetroCyber` | Retro Cyber | `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity`, `boost` |
| `SynthwaveOutrun` | Synthwave Outrun | `rpm`, `max_rpm`, `gear`, `speed` |
| `EvaBioSync` | EVA Bio-Sync | `rpm`, `max_rpm`, `gear`, `speed` |
| `ChronoEventHorizon` | Chrono Event Horizon | `rpm`, `max_rpm`, `gear`, `speed` |
| `NeuralCyberSynapse` | Neural Cyber Synapse | `rpm`, `max_rpm`, `gear`, `speed` |
| `SolarisWarpCore` | Solaris Warp Core | `rpm`, `max_rpm`, `gear`, `speed` |
| `AeroPlasmaBlade` | Aero Plasma Blade | `rpm`, `max_rpm`, `gear`, `speed` |
| `ShibuyaChromaticPrisma` | Shibuya Chromatic Prisma | `rpm`, `max_rpm`, `gear`, `speed` |
| `KineticLiquidMetal` | Kinetic Liquid Metal | `rpm`, `max_rpm`, `gear`, `speed` |
| `CymaticResonanceField` | Cymatic Resonance Field | `rpm`, `max_rpm`, `gear`, `speed` |
| `ChronoMultiverse` | Chrono Multiverse | `rpm`, `max_rpm`, `gear`, `speed` |
| `LumenEtch` | Lumen Etch | `rpm`, `max_rpm`, `gear`, `speed` |
| `VernierLoupe` | Vernier Loupe | `rpm`, `max_rpm`, `gear`, `speed` |
| `DichroicLouvre` | Dichroic Louvre | `rpm`, `max_rpm`, `gear`, `speed` |
| `FibreOpticCorona` | Fibre-Optic Corona | `rpm`, `max_rpm`, `gear`, `speed` |
| `ApertureNine` | Aperture Nine | `rpm`, `max_rpm`, `gear`, `speed` |
| `AeroSmokeTunnel` | Aero Smoke Tunnel | `rpm`, `max_rpm`, `gear`, `speed` |
| `RattrapanteSector` | Rattrapante Sector | `rpm`, `max_rpm`, `gear`, `speed` |
| `SapphireMystery` | Sapphire Mystery | `rpm`, `max_rpm`, `gear`, `speed` |
| `RackFocus` | Rack Focus | `rpm`, `max_rpm`, `gear`, `speed` |
| `BionicSpaceframe` | Bionic Spaceframe | `rpm`, `max_rpm`, `gear`, `speed` |
| `GeodesicWell` | Geodesic Well | `rpm`, `max_rpm`, `gear`, `speed` |
| `Lightpainter` | Lightpainter | `rpm`, `max_rpm`, `gear`, `speed` |
| `TreadSeismograph` | Tread Seismograph | `rpm`, `max_rpm`, `gear`, `speed` |
| `StarSailOrigami` | Star Sail Origami | `rpm`, `max_rpm`, `gear`, `speed` |
| `ArcLadderCascade` | Arc Ladder Cascade | `rpm`, `max_rpm`, `gear`, `speed` |
| `AuroraVeil` | Aurora Veil | `rpm`, `max_rpm`, `gear`, `speed` |
| `IsochromaticLoadPath` | Isochromatic Load Path | `rpm`, `max_rpm`, `gear`, `speed` |
| `AbyssalShearBloom` | Abyssal Shear Bloom | `rpm`, `max_rpm`, `gear`, `speed` |
| `HarmonographDrive` | Harmonograph Drive | `rpm`, `max_rpm`, `gear`, `speed` |
