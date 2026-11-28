# Tesla ADAS

Catalog row 15 · `ClusterTheme::TeslaAdas` (config value `"TeslaAdas"`) · ADAS & HUD · cockpit + hood · build #5 · ★ pattern-setter for the road-scene dashes.

**What it is:** the driver display of the Tesla Model S and Model X (2021 on) in manual driving with Track Mode on, as a Japan-market car shows it on an expressway at night. The road scene fills the middle with the car seen from a fixed chase camera, the left card holds Track Mode's G-meter, and the right column holds the gear, the speed and Tesla's R · N · D row. Labels are in two languages (Japanese over English), the road is a three-lane expressway that keeps left, and under braking the lane shows slow-down dashes and a stop bar with 止まれ. Research, sources and every change from the real display: [`RESEARCH.md`](RESEARCH.md). Board: Penpot page `07 · TeslaAdas` (file `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, page `b568ed3f-64cd-806f-8008-fd974581b17e`; the eight board IDs are in [`HANDOVER.md`](HANDOVER.md)).

**Real telemetry (2026-11-28 rework, the full FH6 Data Out list).** Beyond rpm, gear, speed, brake, G, body slip and power, the scene reads: the game's driving line (`NormalizedDrivingLine`), which the blue path merges onto ahead of the nose; the AI brake difference (`NormalizedAIBrakeDifference`), which turns the path ahead caution, then warning red, as Forza's braking line does; the rumble-strip flags, which light the ego lane's line on that side solid red (Tesla's lane departure warning); the puddle flags, which leave a soft wet patch on the road that scrolls away; each tyre's combined slip, which glows caution to red under that wheel past the edge of grip (1.0); and the lap data, which turns the card's TRACK MODE label into Track Mode's lap timer (lap time, lap, position) while a race runs. With all of these at zero the frame is bit-identical to the earlier build. The signs and scales of the two driving-line fields and the lap number's base are not on the Data Out page: they are constants in `scene.rs` and `LapTimer`, confirmed at real-game check C.

The traffic and the lanes are a seeded ambient simulation. The game sends no data about other cars or lane markings, so nothing in them is presented as a sensor reading.

## Files
- `mod.rs`: the zone tables (`Geom`, one per view), the cluster (`update`, `draw`), the shift pills, the left card and the right column, and the tests.
- `scene.rs`: the road scene: the fixed camera, the lane paint, the path band, the reflector posts, the stop bar and its text, the seeded traffic, and the car images with their procedural fallbacks.
- `spec.md` (the catalog brief), `RESEARCH.md`, `HANDOVER.md`.
- Assets: `assets/clusters/tesla_adas/` with its `ASSETS.md`, built by `tools/asset-forge/src/recipes/tesla_adas.rs`.
- Goldens: `tests/golden/images/TeslaAdas_*.png`.

## Options
None of its own. Colours come from the palette (factory, custom or random), the speed and power units follow the config, and the logo zone shows the car's logo, else the brand or custom name; it stays empty when `ShowLogo` is off.

## Commands
```
cargo run --release -p asset-forge -- --lane cloud --masters tesla_adas build --set tesla_adas
cargo run --release -p asset-forge -- cloud down
cargo run --release -p preview-export -- --cluster TeslaAdas
cargo test -p fh6-clusters tesla_adas
FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters tesla_adas
cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster TeslaAdas --stats
```
The asset build, the renders, the goldens and the demo are GPU jobs: hold the lock of the lane you run on (`SKILLS.md` section 5).
