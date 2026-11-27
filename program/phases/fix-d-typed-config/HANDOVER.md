# HANDOVER: Rust fix phase D · One typed config

**Chat / branch:** thread "Rust fix D", branch `claude/project-thread-ruv0ss` (built on fix phase E's branch, rebased onto main before the merge)  **Date:** 2026-11-27  **Result:** ✅ complete

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
| Finding 15: the seat height migration runs once and the old key is dropped | ✅ | The pre-7 migration moves `CockpitOffsetY` into `DriverSeatHeightM` once, only when the seat is at its default and the old value is not zero, and `CockpitOffsetY` is retired, so a later reload cannot overwrite the seat. Test `an_old_seat_height_migrates_once` |
| Finding 9 (part): `style_to_theme` becomes `impl FromStr for ClusterTheme` | ✅ | Step 1, on main as `21b591f`; fix phase E then removed `fh6-render` |
| Finding 10 (part): keep or remove `shift_guard.rs` and `fh6-win/src/car_brands.rs` | ✅ | Table below: the shift guard is removed, the `car_brands` binding is kept |
| Phase B follow-up: `OledDimmerMinAlpha` has one default and one range | ✅ | The default is `oled_care::DEFAULT_MIN_ALPHA` everywhere; the sanitiser clamps it to 0..=1 and replaces a value that is not a number with the default, the same range the engine uses. Test `the_dimmer_floor_has_one_default_and_one_range` |
| Every rebuilt dash gets the power unit; existing dashes are not edited | ✅ | `PowerUnit` (HP or kW) is a setting of its own and `Signals::power_unit` carries it to every cluster beside `Signals::unit`. The kit README and the cluster prompt tell rebuilds to use it. No dash was edited. Test `signals_carry_the_speed_and_power_units_separately` |
| fmt, clippy `-D warnings`, `cargo test --workspace` with the goldens, parity and the wasm build are green | ✅ | Verification output below |

## What was built
- **`crates/fh6-core/src/config.rs`** (rewritten).
  - One field per setting, grouped as the OSD groups them. `SpeedUnit`, `PowerUnit` and `ColorMode` are enums; the per-view clusters are `ClusterTheme`; `theme(view)` and `set_theme(view, theme)` are the one way to read and write them.
  - A load runs in three stages. First, raw JSON migrations: a file below 7.0.0 gets the one-time fixes (power unit, seat height, the old default name) and is stamped 7.0.0; then blank styles and the old single cluster height are resolved, and the retired keys are dropped. Second, a lenient typed parse: the whole object is tried first, and when it fails each key is added in turn and a key that does not fit is dropped with a warning, so one bad value never resets the rest of the file. Third, the idempotent sanitiser.
  - A file with a newer version keeps it. Unknown keys still go to `extra` and are saved back.
  - Removed: `CameraViewMode` (duplicated `FitView`), `FontPreset`, `effective_cluster_and_scrim` and `default_driver_name`, none of which had a caller.
- **`crates/fh6-core/src/serde_str.rs`** (new): serde by name for any type with `FromStr` and a `name()`, through a visitor, so a bad name is an `invalid value` error naming the setting.
- **`crates/fh6-core/src/cars.rs`** (rewritten): `CarDatabase` by ordinal with private maps, `CarDatabaseError`, `DEFAULT_BRAND_KEY`. `CarOrdinals` had no caller and is removed (`data/fh6_car_ordinals.json` stays).
- **`crates/fh6-core/src/profiles.rs`**: profiles keyed by `i32`; the per-car style overrides are `Option<ClusterTheme>`, and a bad style or key in the file is skipped with a warning. The v1 `CockpitScale` and `BonnetScale` overrides are gone.
- **`crates/fh6-core/src/theme.rs`**: `Default` is derived; serde goes through `FromStr`.
- **`crates/fh6-core/src/shift_guard.rs`**: removed.
- **`crates/fh6-kit`**: `Family` moves from `fh6-clusters` to `src/family.rs`; `Signals::power_unit`; the shift point comes from `shift_point_pct` or the tokens; `Palette::from_config` matches on `ColorMode`.
- **`crates/fh6-clusters`**: the registry's 48 rows use `Family`; the golden harness sets `CustomDriverName` to "TJX", as `preview-export` does.
- **`crates/fh6-osd/src/model.rs`**: every row reads and writes a typed field; the `extra` key helpers are gone.
- **`crates/fh6-app`**: the render loop reads typed fields and caches the palette by `Family`; `resources.rs` logs the car database's load result.
- **`tools/`**: `preview-export`, `bench-render` and `web-preview` follow the API.
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
On the PC (DESKTOP-J891GTB, rustc 1.100.0), commit `559216c` in a worktree with its own target folder, the GPU tests under `gpu.lock`. Real results, shortened:
```
$ cargo fmt --all -- --check
exit 0
$ cargo check --workspace --all-targets
Finished, no warnings
$ cargo clippy --workspace --all-targets -- -D warnings
exit 0, no warnings
$ FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images porsche_analog
1 passed; rewrote PorscheAnalog_01_idle.png (the greeting) and PorscheAnalog_03_redline_strobe.png
(same size, different bytes); the redline file was restored, and the full run below passes against it
$ cargo test --workspace
46 suites: 933 passed, 0 failed, 1 ignored (the doctest at crates/fh6-kit/src/fluid.rs:25)
$ parity-dump tests/golden/packets
20/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
Finished
$ cargo run -p web-preview --bin web-preview-build
Finished (release wasm32 build and wasm-bindgen)
```

Cloud container (rustc 1.97, Linux), before the PC run:
```
$ cargo test -p fh6-core -p fh6-kit -p fh6-osd -p fh6-input   (CPU tests; the kit's GPU tests need an adapter)
all CPU tests pass
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
- **`Palette::family` still takes `&str`.** It has about 40 callers in the clusters; `Family::key()` feeds it. Changing the signature belongs to the catalog re-cut.

## Known issues / debt
- PorscheAnalog still reads its five option keys from `extra` by name every frame (`clusters/porsche_analog/mod.rs`, `Options::from_config`). The cluster prompt now says to parse dash-only settings when the config changes; PorscheAnalog's next pass should cache them.
- The OSD's edit path still clones the whole `AppConfig` on the render thread (9 strings and `extra` now, not 25 strings).
- The gate figures in the cloud are partial by design: GPU tests and the Windows-only crates' tests run only on the PC.

## What the next phases must know
- Read settings from `AppConfig` fields, never from `extra`. A setting only one dash has goes in `extra` under a key named after the dash, parsed when the config changes.
- Adding a setting: a field with `#[serde(default)]` semantics and a default in `impl Default`, a range in `sanitize` if it has one, and a test. A setting that replaces an old key needs a migration in `migrate_raw` gated on the file's major version, and the old key in `RETIRED_KEYS`.
- `CONFIG_VERSION` is 7.0.0. A change that needs a one-time fix bumps the major version and gates the fix on it.
- Speed in `signals.unit`, power in `signals.power_unit`.
- `CarDatabase::car(ordinal)`, `brand_palette(key)` and `car_palette(ordinal)`; `load` returns a `Result`.
- Phase 10's configurator writes the same `config.json`: use `AppConfig`'s serde form and `RETIRED_KEYS`, and read brands through `fh6_win::car_brands`.

## Closing review
REVIEW_PLACEHOLDER
