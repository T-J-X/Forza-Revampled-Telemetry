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
visible instance with its paint and clip, every point, draw run, layer segment and mesh draw, bit for
bit). A probe builds the dash fresh, feeds the scenario for 8 updates, switches the input's probe
value in for 20 and back out for 10, and fingerprints four frames: the first and last with the
value, the first after it and the last. It compares them with a fresh build fed the scenario
without the value, so an input that only shows while it changes (a start banner on `race_on`, a
peak reset on a new car) counts as read. The test also checks that two fresh builds fed the same
packets draw the same frames.

Rebuilt on 2026-11-27 in Phase G from what the gate observes. It replaces the hand-audited v1
snapshot of 2026-11-19, which covered the v1 tiny-skia clusters.

## How a dash plugs in

- A dash phase that rebuilds a dash updates its row in the same commit. On a mismatch the test
  prints the observed row; copy it only after checking that each change is one the dash means
  to make (the module docs list the telemetry a dash claims).
- An input the dash reads only together with another (a lamp with two conditions, a rate) needs
  a scenario where the other conditions hold: add it to `DASH_SCENARIOS` in the test. No dash needs
  one today (the earlier Bosch DDU's four alarm lamps were the first).
- `car_ordinal` reaches every dash through the palette and the logo's brand name, which the host
  picks (`fh6-app` `render_loop.rs`, `ensure_palette` and `Logo.brand_name`); the gate gives each
  dash a fixed palette and the default logo, so a row lists `car_ordinal` only when the dash reads
  it itself. `race_on` and a stale stream hide the dash
  in the host (AGENTS.md directive 9); a row lists `race_on` only when the dash draws differently
  for it.

## Inputs

The probe values are in the test's `INPUTS` table; a discrete input gets several (gear 2, reverse
and neutral; rear and all-wheel drive; throttle full and a quarter; steer both ways). Each field of
`FH6TelemetryData` belongs to exactly one input and each probe value changes exactly its input's
fields (`every_telemetry_field_has_exactly_one_input`), so a new field fails the test until it has
a column. In a field list, `_fr` repeats the previous field's stem.

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

Every dash is probed in five common scenarios: cruise (the mid-cruise review state), idle, redline
and drift braking (the review states in `fh6_clusters::review::STATES`), and a lift off (four
frames at the limiter, then cruise, so a peak or a falling reading differs from the live one). Each
has the fields a review state leaves at zero (timestamp, suspension, car ordinal) set to a
plausible reading, the timestamp counting up and the position moving along the velocity. A dash is
probed in each view the catalog designs it for (cockpit, hood or both; a raised HUD draws in the
cockpit layout).

Dash scenarios (`DASH_SCENARIOS`):

| Dash | Scenario | Makes reachable |
|---|---|---|

None today: every dash's inputs are reachable in the common scenarios.

## Dashes

Six dashes have their own code today (Liquid Glass, Bonnet Line, Porsche Analog, Bosch DDU,
Tesla ADAS, Retro Cyber). The other 68 (42 from the first catalog, the 24 added on 2026-11-28
and the two liquid-metal dashes added later that day, at the end of the table) draw the shared scaffold (`common::scaffold`), which reads rpm, the
limiter, gear and speed.

| Theme | Dash | Reads |
|---|---|---|
| `LiquidGlass` | Liquid Glass | `race_on`, `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `g_lat`, `g_long`, `velocity`, `power`, `boost` |
| `VisionSpatialGlass` | Vision Spatial Glass | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt7HoodPro` | Bonnet Line | `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `steer`, `g_lat`, `g_long`, `velocity`, `boost` |
| `AmgMbuxSuperscreen` | AMG MBUX Superscreen | `rpm`, `max_rpm`, `gear`, `speed` |
| `DriftGlass` | Drift Glass | `rpm`, `max_rpm`, `gear`, `speed` |
| `GlyphDotMatrix` | Glyph Dot Matrix | `rpm`, `max_rpm`, `gear`, `speed` |
| `PorscheAnalog` | Porsche Analog | `race_on`, `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `car_ordinal`, `power`, `boost` |
| `PaganiHorologerie` | Pagani Horologerie | `rpm`, `max_rpm`, `gear`, `speed` |
| `BugattiTourbillon` | Bugatti Tourbillon | `rpm`, `max_rpm`, `gear`, `speed` |
| `LfaV10Nurburgring` | LFA V10 Nürburgring | `rpm`, `max_rpm`, `gear`, `speed` |
| `KoenigseggGyro` | Koenigsegg Gyro | `rpm`, `max_rpm`, `gear`, `speed` |
| `BoschDdu` | Bosch DDU | `race_on`, `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity`, `power`, `boost` |
| `ValkyrieLmp1` | Valkyrie LMP1 | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt7ModularTripod` | GT7 Modular Tripod | `rpm`, `max_rpm`, `gear`, `speed` |
| `TeslaAdas` | Tesla ADAS | `rpm`, `max_rpm`, `idle_rpm`, `gear`, `speed`, `brake`, `g_lat`, `g_long`, `velocity`, `power` |
| `WindshieldHud` | Windshield HUD | `rpm`, `max_rpm`, `gear`, `speed` |
| `ArNavWayfinder` | AR Nav Wayfinder | `rpm`, `max_rpm`, `gear`, `speed` |
| `F35HelmetHmd` | F-35 Helmet HMD | `rpm`, `max_rpm`, `gear`, `speed` |
| `RetroCyber` | Retro Cyber | `race_on`, `rpm`, `max_rpm`, `gear`, `speed`, `throttle`, `brake`, `g_lat`, `g_long`, `velocity`, `car_ordinal`, `power`, `boost` |
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
| `AmgMbuxSupersport` | AMG MBUX Supersport | `rpm`, `max_rpm`, `gear`, `speed` |
| `SkylineR34Mfd` | Skyline R34 MFD | `rpm`, `max_rpm`, `gear`, `speed` |
| `VisionPetals` | Vision Petals | `rpm`, `max_rpm`, `gear`, `speed` |
| `AmgOneF1` | AMG ONE F1 | `rpm`, `max_rpm`, `gear`, `speed` |
| `CarplayIsland` | CarPlay Island | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt3RsApogee` | GT3 RS Apogee | `rpm`, `max_rpm`, `gear`, `speed` |
| `Gt7CockpitPro` | GT7 Cockpit Pro | `rpm`, `max_rpm`, `gear`, `speed` |
| `LiquidMorph` | Liquid Morph | `rpm`, `max_rpm`, `gear`, `speed` |
| `AmgMbuxTrackPace` | AMG MBUX Track Pace | `rpm`, `max_rpm`, `gear`, `speed` |
| `GtrNismoMfd` | GT-R Nismo MFD | `rpm`, `max_rpm`, `gear`, `speed` |
| `SimInputTrace` | Sim Input Trace | `rpm`, `max_rpm`, `gear`, `speed` |
| `LorenzButterfly` | Lorenz Butterfly | `rpm`, `max_rpm`, `gear`, `speed` |
| `AmgMbuxClassic` | AMG MBUX Classic | `rpm`, `max_rpm`, `gear`, `speed` |
| `SuminagashiMarble` | Suminagashi Marble | `rpm`, `max_rpm`, `gear`, `speed` |
| `T50Floodlit` | T.50 Floodlit | `rpm`, `max_rpm`, `gear`, `speed` |
| `HakosukaS20` | Hakosuka S20 | `rpm`, `max_rpm`, `gear`, `speed` |
| `SimPitWall` | Sim Pit Wall | `rpm`, `max_rpm`, `gear`, `speed` |
| `KaleidoscopeMirror` | Kaleidoscope Mirror | `rpm`, `max_rpm`, `gear`, `speed` |
| `RevueltoCorsa` | Revuelto Corsa | `rpm`, `max_rpm`, `gear`, `speed` |
| `ZNismoSport` | Z Nismo Sport | `rpm`, `max_rpm`, `gear`, `speed` |
| `GoldenAngle` | Golden Angle | `rpm`, `max_rpm`, `gear`, `speed` |
| `MclarenSlimDisplay` | McLaren Slim Display | `rpm`, `max_rpm`, `gear`, `speed` |
| `HopfFibration` | Hopf Fibration | `rpm`, `max_rpm`, `gear`, `speed` |
| `HyperbolicTiles` | Hyperbolic Tiles | `rpm`, `max_rpm`, `gear`, `speed` |
| `OilSlickChrome` | Oil-Slick Chrome | `rpm`, `max_rpm`, `gear`, `speed` |
| `FaradayMandala` | Faraday Mandala | `rpm`, `max_rpm`, `gear`, `speed` |
