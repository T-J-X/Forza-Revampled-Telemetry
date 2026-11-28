# HANDOVER: Rust fix phase D · One typed config

**Chat / branch:** thread "Rust fix D", branch `claude/project-thread-ruv0ss` (PR #6, rebased onto main after fix phase E)  **Date:** 2026-11-27  **Result:** ✅ complete

Fix phase D of the Rust structural assessment of 2026-11-27: findings 2, 3, 5, 12, 13, 14 and 15, the `style_to_theme` move from finding 9 (step 1, already on main as `21b591f`), the keep-or-remove list for the modules finding 10 left to this phase, the Phase B follow-up on `OledDimmerMinAlpha`, and the power unit for every rebuilt dash. A fix phase has no MASTER_PROMPT, so the exit criteria are those findings and the gates.

## Exit criteria
| Criterion | Met? | Evidence |
|---|---|---|
| Finding 2: one typed field per view with one default, old names accepted on load | ✅ | `AppConfig::cockpit_hud_style` and `bonnet_hud_style` are `ClusterTheme` (defaults BoschDdu and Gt7HoodPro, as the shipped file); `chase_hud_style` is `Option<ClusterTheme>` and follows the cockpit when unset. The three `*Typed` copies are retired. Names and aliases parse through `ClusterTheme`'s `FromStr`. An unknown name falls back to that view's default with a warning (it used to pick BoschDdu silently). Tests `styles_load_by_name_or_alias_and_the_chase_follows_the_cockpit`, `blank_styles_mean_the_default_and_a_blank_cockpit_takes_the_old_hud_style`, `a_value_that_does_not_fit_takes_its_default_and_the_rest_still_loads` |
| Finding 3: every config key is wired or removed | ✅ | 43 keys are retired (`config::RETIRED_KEYS`, listed below): a load drops them and a save never writes them. All 82 remaining fields have a reader outside `config.rs`. A save writes 77 keys (was 115). `data/config.json` is rewritten at 7.0.0 in exactly the form a save writes (`data_config_is_what_a_save_writes`), and the file shipped before it loads to the same config (`the_config_shipped_before_7_loads_as_the_one_shipped_now`, which embeds the old file byte for byte) |
| Finding 5: the default driver name is empty, empty means no name, and a cleared name stays cleared | ✅ | `custom_driver_name` defaults to `""`. The logo zone shows the brand name, else the custom name, else nothing; PorscheAnalog's greeting falls back to the brand name, then "DRIVER 1". The pre-7 migration clears the old CJK default once; nothing puts it back. Test `the_old_default_driver_name_is_cleared_and_an_empty_name_stays_empty`. SKILLS.md law 5 already says the pack has no CJK face (fixed earlier), so no font is added |
| Finding 12: closed sets are enums with serde names | ✅ | `SpeedUnit`, `PowerUnit`, `ColorMode` and `ClusterTheme` are enums with `FromStr`, `Display` and serde by name (`serde_str::serde_by_name!`). `Family` is an enum in `fh6-kit` and all 48 registry rows use it. The speed smoother compares an enum each frame, not a string. `AppConfig` holds 9 `String`s (was 25), so an OSD edit clones far less |
| Finding 13: live settings are typed fields resolved when the config changes | ✅ | The seven OSD keys that lived in `extra` (`ShowLogo`, `ShowStats`, `ReducedFlash`, `GlassRefraction`, `ClusterFont`, `PaletteSeed`, `ShiftPointPct`) are typed fields. The render loop and the kit read fields, never `extra` by name. The shift point has one home (`shift_point_pct`, `None` = the tokens); `ShiftStartPct` is retired. Test `the_osd_settings_are_typed_fields`, the OSD's `units_shift_point_and_limiter_rows_edit_typed_fields` |
| Finding 14: the car database is keyed by ordinal, normalised once, and its load returns a `Result` | ✅ | `CarDatabase` holds `HashMap<i32, CarEntry>` and upper-cased, trimmed brand keys; a lookup never allocates. `load` returns `Result<_, CarDatabaseError>` and the app logs the error and the counts. Car profiles are keyed by `i32` too. Test `the_shipped_database_has_a_palette_for_every_car` (693 cars, 37 brands) |
| Finding 15: the seat height migration runs once and the old key is dropped | ✅ | The pre-7 migration copies `CockpitOffsetY` into `DriverSeatHeightM` once, only for a file without `DriverSeatHeightM` (a C# port file; the Rust build always wrote `DriverSeatHeightM`) and only when the old value is not zero. `CockpitOffsetY` is retired, so no later load can overwrite the seat. Tests `an_old_seat_height_migrates_once` (including the reset-to-default case in a 6.0.0 and a 7.0.0 file) and `a_file_the_rust_build_saved_before_7_loads_with_its_osd_settings` |
| Finding 9 (part): `style_to_theme` becomes `impl FromStr for ClusterTheme` | ✅ | Step 1, on main as `21b591f`; fix phase E then removed `fh6-render` |
| Finding 10 (part): keep or remove `shift_guard.rs` and `fh6-win/src/car_brands.rs` | ✅ | Table below: the shift guard is removed, the `car_brands` binding is kept |
| Phase B follow-up: `OledDimmerMinAlpha` has one default and one range | ✅ | The default is `oled_care::DEFAULT_MIN_ALPHA` everywhere; the sanitiser clamps it to 0..=1 and replaces a value that is not a number with the default, the same range the engine uses. Test `the_dimmer_floor_has_one_default_and_one_range` |
| Every rebuilt dash gets the power unit; existing dashes are not edited | ✅ | `PowerUnit` (HP or kW) is a setting of its own and `Signals::power_unit` carries it to every cluster beside `Signals::unit`. The kit README and the cluster prompt tell rebuilds to use it. No dash was edited. Test `signals_carry_the_speed_and_power_units_separately` |
| fmt, clippy `-D warnings`, `cargo test --workspace` with the goldens, parity and the wasm build are green | ✅ | Verification output below |

## What was built
- **`crates/fh6-core/src/config.rs`** (rewritten).
  - One field per setting, grouped as the OSD groups them. `SpeedUnit`, `PowerUnit` and `ColorMode` are enums; the per-view clusters are `ClusterTheme`; `theme(view)` and `set_theme(view, theme)` are the one way to read and write them.
  - A load runs in three stages. First, raw JSON migrations: a file below 7.0.0 gets the one-time fixes (power unit, seat height, the old default name) and is stamped 7.0.0; then blank styles and the old single cluster height are resolved, each FOV key keeps one spelling (the canonical `BonnetFOV` wins over `BonnetFov` in any order), and the retired keys are dropped in place, keeping the order of the rest. Second, a lenient typed parse: the whole object is tried first, and when it fails each key is added in turn and a key that does not fit is dropped with a warning, so one bad value never resets the rest of the file. A safe zone loads margin by margin the same way. Third, the idempotent sanitiser: a number that is not finite takes its default (an f32 overflow would otherwise save as `null`), and settings with a range are clamped, including the six FOVs (`FOV_RANGE`, 40 to 110 degrees, now in `fh6-core` and used by the camera patcher and the OSD slider) and `AmbientGlowIntensity` (0 to 1).
  - Versions: the leading digits of `Version` are its major (`"7-beta"` is 7, a number such as `8` counts and is kept as text). A newer file keeps its version and its keys, retired names included, since a newer build may reuse one. A version this build cannot read (`"v7"`) is kept, not migrated, with a warning. Unknown keys still go to `extra` and are saved back in their order.
  - `serde_json` in `fh6-core` uses `preserve_order`, as the app already did through `fh6-win`, so `fh6-core`'s tests run on the map the app ships.
  - Removed: `CameraViewMode` (duplicated `FitView`), `FontPreset`, `effective_cluster_and_scrim` and `default_driver_name`, none of which had a caller.
- **`crates/fh6-core/src/serde_str.rs`** (new): serde by name for any type with `FromStr` and a `name()`, through a visitor, so a bad name is an `invalid value` error naming the setting.
- **`crates/fh6-core/src/cars.rs`** (rewritten): `CarDatabase` by ordinal with private maps, `CarDatabaseError`, `DEFAULT_BRAND_KEY`; two cars with one ordinal warn. `CarOrdinals` had no caller and is removed (`data/fh6_car_ordinals.json` stays).
- **`crates/fh6-core/src/profiles.rs`**: profiles keyed by `i32`, loaded car by car: a car whose key is not an ordinal or whose profile does not load is skipped with a warning and the others still load (before, one bad value dropped every profile). A style that is not a known name, or not text, is ignored with a warning. Two keys for one ordinal warn. The v1 `CockpitScale` and `BonnetScale` overrides are gone.
- **`crates/fh6-core/src/theme.rs`**: `Default` is derived; serde goes through `FromStr`.
- **`crates/fh6-core/src/shift_guard.rs`**: removed.
- **`crates/fh6-kit`**: `Family` moves from `fh6-clusters` to `src/family.rs`; `Signals::power_unit`; the shift point comes from `shift_point_pct` or the tokens; `Palette::from_config` matches on `ColorMode`.
- **`crates/fh6-clusters`**: the registry's 48 rows use `Family`; the golden harness sets `CustomDriverName` to "TJX", as `preview-export` does.
- **`crates/fh6-osd/src/model.rs`**: every row reads and writes a typed field; the `extra` key helpers are gone.
- **`crates/fh6-app`**: the render loop reads typed fields and picks the cluster with `cfg.theme(view)` directly (the theme cache is gone). The palette cache holds the `Arc<AppConfig>` it was built from with the car ordinal and the `Family`, so a freed config's address can never match a newer one. `resources.rs` logs the car database's load result, with the parse error's cause.
- **`crates/fh6-win/src/camera_patcher.rs`**: uses `fh6_core::config::FOV_RANGE`.
- **`tools/`**: `preview-export`, `bench-render` and `web-preview` follow the API; `preview-export` prints an error's causes.
- **`data/config.json`**: 7.0.0, 77 keys, pretty-printed as a save writes it.
- **Rule files**: `AGENTS.md` (the `fh6-core` and `fh6-win` rows), `docs/kit/README.md` (the power unit), `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` ("Units and settings"), `docs/PROGRESS.md` (state of the code, known defects 14 and 15, decision log).

## Keep or remove
| Item | Decision | Why |
|---|---|---|
| `fh6-core/src/shift_guard.rs` | Removed | No caller, and its key `ManualShiftGuardEnabled` is retired. Git history keeps it if a manual-gearbox guard is ever wanted |
| `fh6-win/src/car_brands.rs` | Kept | The Phase 10 configurator needs a brand list with palettes; the binding is tested. AGENTS.md now says nothing calls it yet |
| `fh6-core::cars::CarOrdinals` | Removed | No caller; the ordinal file stays in `data/` |
| `CameraViewMode`, `FontPreset` | Removed | `FitView` is the view type; the font is `ClusterFont` |
| `fh6-core/src/trail.rs` | Kept (not this phase's call) | The assessment keeps it for WindshieldHud |

### Retired keys (43)
- v1 renderer and duplicate copies: `CockpitScale`, `BonnetScale`, `CockpitStyle`, `BonnetStyle`, `HudStyle` (a blank cockpit style takes it once), `CockpitHudStyleTyped`, `BonnetHudStyleTyped`, `ChaseHudStyleTyped`, `ActiveCameraView`, `ShowBonnetGTWidgets`, `ShowBonnetGtWidgets`, `FontPreset`, `FovDegrees`, `AmbientCowlFeather`.
- C# port settings no v2 code reads: `OsdLiveDynoPreviewWhenPaused`, the four `DriftSync*`, `LockVisibleWhileDriving`, `ShiftLightPattern`, `ShiftStartPct`, `RedlinePct`, `ControllerCameraButton`, `ManualShiftGuardEnabled`, `DriverTagGlitchFx`, `RedlineStrobe`, `NightGlowMode`, `NightGlowIntensity`, `AdasBackdropMode`, `CenterConsoleMapMode`, `CenterConsoleInfotainmentMode`, `RouteActive`, `OemDashDimmerEnabled`, `ModularSidePodsEnabled`, `SteeringHubCapEnabled`, `UseWidgetColorOverrides` and its four colours, `PrimaryPort`.
- Migrated once: `CockpitOffsetY` (into `DriverSeatHeightM`).

A feature that brings one back adds a typed field and takes the key off the list; `no_retired_key_is_a_setting` keeps the two apart.

## Verification output
On the PC (DESKTOP-J891GTB, rustc 1.100.0), in a worktree with its own target folder, the GPU work under `gpu.lock`. Real results, shortened.

Run 3, commit `d03b4f5` (rebased onto main after fix E, with the review fixes):
```
$ cargo fmt --all -- --check
exit 0
$ cargo check --workspace --all-targets
Finished, no warnings
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0, no warnings
$ cargo test --workspace
46 test binaries: 941 passed, 0 failed, 1 ignored (the doctest at crates/fh6-kit/src/fluid.rs:25);
golden_images all pass, golden_telemetry passes
$ parity-dump tests/golden/packets
MISMATCH tests/golden/packets\01.bin ... (all 20), 0/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
Finished
$ cargo run -p web-preview --bin web-preview-build
Finished (release wasm32 build and wasm-bindgen)
$ FH6_WheelDash --game-process fh6-stand-in.exe --bench-seconds 25   (release; mock-udp bmw_m4_cs at 60 Hz)
config: data/config.json 7.0.0, theme=BoschDdu view=Cockpit; no warning while loading config.json or car_brands.json
CPU per frame p50 0.639 ms, p99 0.837 ms, max 1.171 ms (3493 frames)
packet to present p50 0.656 ms, p99 1.377 ms (1460 packets); 139.7 fps presented
```
The parity failure was the tool, not the decoder: `parity-dump` compared the golden's text with its own, and once `fh6-core` turned on `serde_json`'s `preserve_order` its keys came out in insertion order while the goldens are alphabetical (`golden_telemetry`, which compares values, passed). It now compares values (after writing its own through text, so an f32's shortest form compares as written), writes a new golden with sorted keys, and has a test that checks all 20 goldens. The goldens are unchanged.

Run 4, commit `2c79a72` (only `tools/parity-dump/src/main.rs` and the docs changed since run 3, so only its gates ran again):
```
$ cargo fmt --all -- --check
exit 0
$ cargo clippy -p parity-dump --all-targets -- -D warnings
Finished, no warnings
$ cargo test -p parity-dump
3 passed; 0 failed; 0 ignored
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ git status --short
(empty: no golden was written)
```

Run 2, commit `559216c` (before the rebase), which re-rendered the golden:
```
$ FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images porsche_analog
1 passed; rewrote PorscheAnalog_01_idle.png (the greeting) and PorscheAnalog_03_redline_strobe.png
(same size, different bytes); the redline file was restored, and the full run passed against it
$ cargo test --workspace
46 suites: 933 passed, 0 failed, 1 ignored
$ parity-dump tests/golden/packets
20/20 OK
```

Cloud container (rustc 1.97, Linux), before each PC run:
```
$ cargo test -p fh6-core -p fh6-osd -p fh6-input -p parity-dump, cargo test -p fh6-kit -p fh6-clusters --lib
all pass (fh6-core 109 + 7 + 1, fh6-osd 44, fh6-kit 125, fh6-clusters 238, parity-dump 3)
$ parity-dump tests/golden/packets
20/20 OK
$ cargo check / clippy --target x86_64-pc-windows-gnu -p fh6-app -p fh6-win -p screen-capture -p stand-in-game -p wgc-bench --all-targets
no warnings in the files this phase changed
$ cargo build -p fh6-kit -p web-preview --target wasm32-unknown-unknown
Finished
```

## Visuals
One golden changes: `tests/golden/images/PorscheAnalog_01_idle.png`. The idle state is parked in neutral, so the right tube greets the driver. It used to greet the old default name, drawn as "ØG ŦJ" with gaps where the CJK glyphs are missing; the harness now sets the name to "TJX", as every `preview-export` render does, so the golden no longer depends on the default. No other golden changes: the old and new `AppConfig::default()` differ only in retired keys, the version, the power unit (no existing dash reads it) and the driver name.

## My calls
- **Retire, not migrate, `PrimaryPort`.** v2 never read it; the primary port is `UdpPort`.
- **Power unit default HP, and an old file keeps what it showed.** Before 7.0.0 every dash chose the power unit from the speed unit, so the migration sets kW for a km/h file and HP otherwise, unless the file already held "HP" (the old default was "kW", even with MPH, and no dash read it).
- **A misspelt hood style falls back to Gt7HoodPro, a misspelt chase style follows the cockpit.** The view's own default, with a warning, instead of BoschDdu silently.
- **Duplicate spellings keep the first.** `ShowBonnetGTWidgets` and `ShowBonnetGtWidgets` both existed; with the lenient parse the first one in the file wins and the second is dropped with a warning.
- **The golden harness names the driver.** The alternative was a golden that changes whenever the default does.
- **The FOVs are clamped in the config, not only when written.** To the game's own limits (`MinFOV` / `MaxFOV`, 40 to 110), which the patcher and the OSD slider already used, so the file and the OSD show what the game gets.
- **Car profiles load car by car.** A broken profile used to drop every car's fit; now only that car's.
- **`Palette::family` still takes `&str`.** It has about 40 callers in the clusters; `Family::key()` feeds it. Changing the signature belongs to the catalog re-cut.

## Known issues / debt
- PorscheAnalog still reads its five option keys from `extra` by name every frame (`clusters/porsche_analog/mod.rs`, `Options::from_config`). The cluster prompt now says to parse dash-only settings when the config changes; PorscheAnalog's next pass should cache them.
- The OSD's edit path still clones the whole `AppConfig` on the render thread (9 strings and `extra` now, not 25 strings).
- The OSD's power unit row changes nothing on the three dashes that exist today: Bosch DDU and PorscheAnalog still pick power from the speed unit, and Liquid Glass always shows hp. That is this phase's scope (existing dashes are not edited); each one reads `signals.power_unit` when it is rebuilt. Before this phase the row was just as inert.
- Pre-existing, outside this phase: `--cluster <Name>` forces the style in the live config, and the OSD seeds its working copy from that config, so any OSD edit during such a run saves the forced style into all three style keys (`fh6-app/src/render_loop.rs`, where the OSD's edit copy is made).
- Pre-existing: saving `car_profiles.json` drops keys this build does not know (`AppConfig` keeps them in `extra`; `CarProfile` has no such map).
- A file below 7.0.0 that is never saved runs its one-time migration on every load. The result is the same each time, and the first save ends it.
- Only a hand-written file hits this: a blank `CockpitHudStyle` with no `HudStyle` key now means Bosch DDU (the old build fell back to Liquid Glass, its `HudStyle` default).
- Style, left as is: the OSD's `ColourMode` sits beside the core's `ColorMode` in one file; `fh6_kit::cluster` re-exports `SpeedUnit` and `PowerUnit` (a second path, as it did for `SpeedUnit` before); the shipped-database test pins 693 cars and 37 brands, so a data update must update it.
- The gate figures in the cloud are partial by design: GPU tests and the Windows-only crates' tests run only on the PC.

## What the next phases must know
- Read settings from `AppConfig` fields, never from `extra`. A setting only one dash has goes in `extra` under a key named after the dash, parsed when the config changes.
- Adding a setting: a field with `#[serde(default)]` semantics and a default in `impl Default`, a range in `sanitize` if it has one, and a test. A setting that replaces an old key needs a migration in `migrate_raw` gated on the file's major version, and the old key in `RETIRED_KEYS`.
- `CONFIG_VERSION` is 7.0.0. A change that needs a one-time fix bumps the major version and gates the fix on it.
- Speed in `signals.unit`, power in `signals.power_unit`.
- `CarDatabase::car(ordinal)`, `brand_palette(key)` and `car_palette(ordinal)`; `load` returns a `Result`.
- Phase 10's configurator writes the same `config.json`: use `AppConfig`'s serde form and `RETIRED_KEYS`, and read brands through `fh6_win::car_brands`.

## Review findings
Two independent reviewers read the change with the `rust-review` skill and the `rust-best-practices` rules: one the config core (`fh6-core`), one its callers. Each finding and what was done:

| Finding | Done |
|---|---|
| The seat migration copied `CockpitOffsetY` again into a file the old Rust build saved with the seat set back to the default: the bug it was meant to remove | Fixed: only a file without `DriverSeatHeightM` takes the old value. Tests for the reset case in 6.0.0 and 7.0.0 files |
| Which of two spellings won depended on the map type: the app (`preserve_order`) and the tests (sorted map) disagreed, and removing a retired key could move an alias in front | Fixed: one spelling per key in the migration (the canonical one wins), retired keys removed with `retain`, and `fh6-core`'s tests run with `preserve_order`. Tests in both orders and with a retired key |
| One bad margin reset a whole safe zone | Fixed: a safe zone loads margin by margin. Test |
| A numeric or unreadable version was treated as 0, so a newer file was migrated and lost its version | Fixed: leading digits, numbers count, an unreadable version is kept and not migrated. Tests |
| FOVs, camera offsets and the ambient glow could become infinite and save as `null`; the docs said every value was range-checked | Fixed: non-finite values take their defaults, the FOVs and the glow are clamped, and the docs say what is checked. Test |
| One non-text style dropped every car profile | Fixed, and wider: profiles load car by car. Test |
| Duplicate ordinals collapsed silently | Fixed: a warning in the car database and the profiles |
| Retired keys were stripped from newer files too | Fixed: only from files up to 7.0.0. Test |
| Migration key names are hand-typed | Fixed: a test checks every key a migration writes is a saved setting |
| A direct `Deserialize` of `AppConfig` skips the load steps | Doc line on `AppConfig` |
| No fixture for a file the old Rust build saved | Added: its default save plus the seven OSD settings loads to the same config with those settings |
| Car profiles drop unknown keys on save; migrations are never written back; `--cluster` plus an OSD edit saves the forced style; a blank cockpit style without `HudStyle` | Pre-existing or hand-edit only: listed under known issues |
| The power unit row changes nothing on the existing dashes | Out of scope by the phase brief (no dash edits): listed under known issues |
| The car database error was logged without its JSON position | Fixed: the cause is logged; `preview-export` prints the error chain |
| The palette test could not tell a by-ordinal hit from the fallback, and `Palette::from_config` had no test | Fixed: the test checks the colours; a new test covers every colour mode |
| Three OSD assertions could no longer fail | Fixed: the shift point is checked exactly and a save/load round trip replaces the dead checks |
| The palette cache compared a raw pointer to a config that could be freed (pre-existing) | Fixed: the cache holds the `Arc` |
| The theme cache costs as much as the three-arm match it saves | Removed |
| `serde_json` unused in `fh6-osd` | Moved to its dev-dependencies (the round-trip tests use it) |
| `bench-render` used the literal `"liquidGlass"` | Uses `Family::LiquidGlass` |
| Style: `ColourMode` beside `ColorMode`, the kit's re-export of the units, two `pub use config` lines, `ClusterTheme::default()` is not the cockpit default, exact counts in the database test | The two `pub use` lines are merged; the rest is listed under known issues as style |

## Closing review
Against `AGENTS.md` and `SKILLS.md`, yes or no with the evidence:

1. No placeholders or stand-ins, and every config key is live. **Yes.** 82 fields, each with a reader outside `config.rs`; 43 dead keys retired. The power unit row is live in the config and for rebuilt dashes; the existing dashes ignore it by scope (known issues). [AGENTS 2]
2. Verified with real output. **Yes.** PC gates below. [AGENTS 3]
3. The manual-gearbox lock holds. **Yes.** `sanitize_toggle_mask` still strips 0xF000 on load and save (`save_strips_manual_gearbox_buttons_from_the_mask`). [AGENTS 4, SKILLS law 3]
4. UTF-8 without BOM in `data/`. **Yes.** `data/config.json` is written as a save writes it; `save_round_trips_without_bom`. [AGENTS 5]
5. No `unwrap`, `expect` or `panic!` in new non-test code. **Yes.** Reviewed by both reviewers. [AGENTS 6]
6. No new dependency without a decision-log line. **Yes.** No new crate; `fh6-core` enables `serde_json`'s `preserve_order`, which the app already used, and the decision log says so. [AGENTS 6]
7. No per-frame allocation or string work added. **Yes.** The render loop lost its per-frame string compares and `extra` lookups; the live bench is below. [AGENTS 6 and 7, SKILLS laws 8 and 9]
8. Deploy guard. **Yes.** Nothing was copied to the game folder. [AGENTS 8]
9. Public repo identity. **Yes.** Commits by `TJX <TJX@Ngage.com>` with no trailer; no real name or user path in a file. [AGENTS 10]
10. Rust only. **Yes.** [AGENTS 11]
11. Reviews are yes/no, not scores. **Yes.** Two reviewers, findings only, each answered above. [AGENTS 12]
12. Progress posted in the thread. **Yes.** [AGENTS 13]
13. GPU work under `gpu.lock`. **Yes.** Every PC run that rendered held it and released it after. [SKILLS section 5]
14. Fonts: no CJK promise. **Yes.** The default name is empty; law 5 already says the pack has no CJK face. [SKILLS law 5]
15. The rule files match the code. **Yes.** `AGENTS.md`'s `fh6-core` and `fh6-win` rows, the kit README and the cluster prompt are updated; `docs/PROGRESS.md` has the state of the code, the defects and the decisions.
