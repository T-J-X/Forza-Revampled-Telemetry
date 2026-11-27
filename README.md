# FH6 WheelDash

A Rust overlay for Forza Horizon 6. It reads the game's UDP telemetry and draws 4K instrument clusters on a click-through layered window. It also covers the in-game dash with a domed OLED-black scrim, adds a gamepad-driven OSD for live tuning, and patches the game's camera FOV and seat height. It is built for a 55" LG C1 OLED at 3840x2160.

## Build and test

```
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Rust 1.80+ (developed on 1.100, MSVC). Windows is needed for `fh6-win`, `fh6-input` and `fh6-app`. The other crates build on any OS.

## Tools

| Command | What it does |
|---|---|
| `cargo run --release -p preview-export` | Renders every cluster, scrim and OSD state to `mockups/` (gitignored), with an `index.html` gallery |
| `cargo run -p mock-udp -- --scenario bmw_m4_cs --port 8000` | Sends synthetic telemetry so you can drive the overlay without the game |
| `cargo run -p parity-dump -- tests/golden/packets` | Checks the packet decoder against the 20 golden packets |
| `cargo run -p fh6-app --bin FH6_WheelDash` | Runs the overlay |

## Layout

```
crates/   fh6-core  fh6-render  fh6-clusters  fh6-osd  fh6-input  fh6-win  fh6-app
tools/    preview-export  mock-udp  parity-dump  bench-render
tests/golden/packets/   324-byte telemetry captures + expected JSON
assets/   textures, car brand colours, reference photos
data/     config.json, car profiles, ordinals, design tokens, camera files
docs/     PROGRESS.md (status and backlog), specs/
```

## Working on it

[AGENTS.md](AGENTS.md) holds the rules and the specialist council, [SKILLS.md](SKILLS.md) says which skill or MCP to use when, and [docs/PROGRESS.md](docs/PROGRESS.md) shows where things stand.

Controller rule: A, B, X and Y belong to the manual gearbox and are never used by the overlay.
