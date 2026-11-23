# HANDOVER: Phase 9 · Controller-first Liquid Glass OSD

**Chat / branch:** `claude/phase-9-osd-1fcc97`  **Date:** 2026-11-23  **Result:** ✅ complete (Figma page blocked by the plan; HTML board used as the reference, as in Phase 3)

Main (6ccf761, Phases 6 and 8) is merged into the branch (63b1682) and every gate below ran on the merged tree.

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence (command output, file, screenshot) |
|---|---|---|
| Workspace check, test and clippy are green. | ✅ | Verification output below: check clean, test 366 passed, 0 failed, 1 ignored, clippy `-D warnings` clean, fmt clean, parity 20/20, wasm build |
| The Figma page is done. | ◐ fallback | The remote Figma MCP returned the Starter-plan tool-call cap on the first `use_figma` write (`docs/design/FIGMA.md`). Reference instead: `docs/design/boards/osd/index.html` (every state, real renders) and `docs/design/OSD.md` |
| 4K OSD renders (each tab, plus focus and slider states) are in `mockups/osd/`, with the critique passed. | ✅ | 11 states from `cargo run --release -p preview-export` in `mockups/osd/` (gitignored), 960×540 copies in `docs/design/osd/`. Critique round 1 fail (6 must-fixes, all fixed), round 2 **pass** (`docs/design/OSD.md` § Critique) |
| The live view works in `--demo`, and the perf numbers are in `docs/perf/osd.md`. | ✅ | `FH6_WheelDash --demo --open-osd` shows the live dyno ("Game paused · live dyno"). GPU 1.26–1.32 ms median for the whole OSD at 4K (target < 1.5). CPU p50 0.99 ms, p99 2.24 ms (defect 13) |
| A manual run with screenshots shows the controller toggle, navigation and live-applied changes. | ✅ | `docs/handover-shots/phase9-01…07` (scripted pad `phase9-osd-tour.pad` through the real tracker: hold View to open, LB/RB tabs, D-pad carousel, custom hue, cluster height held to +120 px live, Controls, closed with the new settings); `phase9-08-merged-fit-tab.png` from the merged release build (pack fonts, Camera patch row, per-view cluster height). Earlier run log: "OSD opened mid-race: ESC sent to pause the game" |

## What was built
- **`crates/fh6-osd`** (new crate): `model` (5-tab row model, values, help, `PROFILE_OVERRIDE_NOTE`), `nav` (state machine, hold-to-repeat with acceleration, `focus_row`), `keyboard` (on-screen, D-pad), `remap` (hold 1 s, ABXY and D-pad refused), `pause` (`AutoPause` behind `KeyboardInjector`), `saver` (debounced 350 ms, UTF-8 without BOM), `dyno`, `anim`, `fit` (`CockpitFit` seam, `SCRIM_ROWS`, `ConfigOnlyFit`), `draw` (kit drawing, live backing with the real blocker), `host` (`OsdHost`: carousel thumbnails, `note_profile`, `blocker_focused`).
- **`crates/fh6-app`**: OSD wiring in `render_loop.rs` (open and close edges, working config followed until the saved file reloads, `rows_cfg` vs `cfg`, live blocker preview), `osd_fit.rs` (`PatcherFit` over Phase 8's `CameraPatchHandle` and `InputShared::resync_camera`), `--pad-script` and `--open-osd`.
- **`crates/fh6-input`**: press/release latches (`take_edges`), the hold toggle suspended while the OSD has focus, `PadScript` / `ScriptedPad`.
- **`crates/fh6-win`**: `SendInput` key taps for the auto-pause ESC.
- **`crates/fh6-kit`**: `Kit::set_underlay` (dim in the resolve), `Painter::set_backdrops(false)` (thumbnails), font-pack loading from `assets/fonts/fonts.json` (`Fonts::load_dir`, variable-font instances, deferred glyph preload with `ensure_preloaded`, typographic family names, cached kerning plans).
- **`tools/preview-export`**: the 11 OSD states with GPU timings, `--backdrop PNG`.
- **Docs:** `docs/design/OSD.md`, `docs/design/boards/osd/index.html`, `docs/design/osd/*.png`, `docs/perf/osd.md`, `docs/handover-shots/phase9-*`, PROGRESS, AGENTS.
- **Workspace:** dev-profile `opt-level = 3` for rustybuzz, ttf-parser, fdsm and nalgebra (text tests 34.6 s → 2.1 s). No new dependency.

## Verification output
```
$ cargo check --workspace
warning: `web-preview` (manifest) generated 1 warning   (unused dependency fh6-clusters, as on main)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.55s
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.00s
$ cargo fmt --all -- --check
fmt ok
$ cargo test --workspace   (sum of every "test result" line)
TOTAL passed=366 failed=0 ignored=1
$ cargo run -p parity-dump -- tests/golden/packets
OK tests/golden/packets\19.bin
OK tests/golden/packets\20.bin
20/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
wasm ok
```
GPU per OSD state (`preview-export`, merged branch, pack fonts loaded: 81 faces, 9 roles), median / min ms:
```
01 1.316/1.269  02 1.302/1.298  03 1.271/1.265  04 1.267/1.212  05 1.263/1.177  06 1.297/1.245
07 1.277/1.245  08 1.297/1.287  09 1.287/1.202  10 1.300/1.218  11 1.287/1.253
```
Post-merge scripted run (release build, `--demo`, copy of `config.json`): log "OSD toggled by hold open=true", "OSD opened view=Cockpit", "OSD closed"; the saved copy has `CockpitHeightOffsetPx: 240.0` (held Right on Cluster height) and `CameraPatchAuto: false` (the tour never touches the patch switch).

## Visuals
- `docs/design/boards/osd/index.html` (board), `docs/design/osd/01…11` (renders), full size in `mockups/osd/`.
- `docs/handover-shots/phase9-01-before-open.png` … `phase9-07-closed-new-settings.png`, `phase9-08-merged-fit-tab.png`.

## My calls
- **Figma fallback to the HTML board:** the plan's MCP cap blocks writes; same call as Phase 3.
- **Dim harder (62%) instead of blurring the game:** the overlay never captures the game, so it cannot blur it; feathered pools behind the echo and the live cluster keep text legible.
- **Rows edit the global config; the live view shows the car's profile on top** and an overridden row says so. Per-car editing needs a save target per row and is Phase 10's configurator work.
- **Camera rows follow the patch switch:** "Applies on next launch" when on, "saved in config only" when off, "camera files not found" when the patcher has no install. The Cockpit FOV row shows the wheel-less FOV, which is the value the patcher writes.
- **No OSD while the overlay is hidden from the tray** (and so no ESC): an OSD nobody can see must not pause the game.
- **The dyno keeps the last raced car's profile**, so the live view while paused shows what that car really gets.
- **Dev opt-level for the text crates:** debug text tests were 34.6 s; release behaviour unchanged.
- **Screenshots of the merged build:** only one kept (08). In later runs the desktop app took the z-order over the stand-in mid-run and showed through the glass, so those captures were deleted; the pre-merge set 01–07 still shows the full tour.

## Known issues / debt
- **Pause-menu acceptance shot:** the OSD over the real Forza pause menu needs TJX in the game (deploy guard).
- **Critique carry-overs:** LB/RB chips in the tab row; a live preview of the draft custom name.
- **CPU p99 with the OSD open:** 2.24 ms (defect 13). Paused-only, shares the Phase 4/5 levers.
- **Unverified review items** (defect 14): custom name cleared then restored by the config migration on reload; seat height reverted on reload when a legacy `CockpitOffsetY` remains; a face marked preloaded when the glyph atlas is full; the open-edge ESC judges "racing" from a packet up to 250 ms old. The review's behaviour pass stopped at the usage limit, so that dimension is not covered.
- **CJK:** the pack has no CJK glyphs (defect 15).
- **Torn captures:** `screen-capture` (BitBlt) can tear on a DirectComposition overlay mid-animation; capture on settled frames.

## What the next phases must know
- **Cockpit-fit seam:** `fh6_osd::fit::CockpitFit` (`camera_changed`, `scrim_rows`, `can_resync`, `resync`). The app passes `fh6_app::osd_fit::PatcherFit`; previews and tests use `ConfigOnlyFit`. Blocker rows live in `fh6_osd::SCRIM_ROWS` (9 rows over the `OemDash*` keys with Phase 8's sanitised ranges).
- **Two configs per frame while open:** `rows_cfg` is the global working config (rows, update, draw, prepare, `blocker_focused`); `cfg` is `CarConfig::resolve(rows_cfg, profiles, car_ordinal)` for the live cluster. `OsdHost::note_profile(&rows_cfg, &cfg)` sets the "profile overrides" help and the default-height line. Phase 10 per-car editing should save through `CarProfilesFile::upsert` and can drop the note for rows it saves per car.
- **Font pack API:** `Fonts::load_dir(dir)` reads `fonts.json` (or scans the folder and subfolders), one face per weight in 300/400/500/600 for variable fonts, glyph preload deferred until `ensure_preloaded(FontId)` or `set_role(role, FontId)`. `MAX_FACES` is 160. The family name is the manifest's, else name ID 16, else ID 1.
- **Extra config keys** (in `AppConfig::extra`, `model::KEY_*`): `ShiftPointPct`, `PaletteSeed`, `ClusterFont`, `ShowLogo`, `ShowStats`, `GlassRefraction`, `ReducedFlash`.
- **Manual runs without a controller:** `FH6_WheelDash --demo --pad-script <file> --game-process fh6-stand-in.exe --config <copy>`; `--open-osd` opens it at start. The script goes through the real tracker, so A/B/X/Y are stripped. Start the stand-in first and keep other windows off the screen while capturing.
- **Never** read A/B/X/Y (`MASK_MANUAL_LOCK`); the OSD live view lifts race-off and stale only, never the foreground or tray-hidden rules.
