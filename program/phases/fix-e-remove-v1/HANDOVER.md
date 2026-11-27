# HANDOVER: Fix phase E · Remove v1

**Chat / branch:** thread "Rust fix E", branch `claude/project-thread-euvhdf` (draft PR #4), on main `21b591f`  **Date:** 2026-11-27  **Result:** ✅ complete

Fix phase E of the Rust structural assessment of 2026-11-27, "Remove v1": findings 8, 9, 11 and 19, and `rev_limiter.rs` from finding 10, plus the asset ledger's "v1" label from the audit extras. A fix phase has no MASTER_PROMPT, so the exit criteria are those findings, the scope the coordinator set (E's own files; D, F and G ran in parallel threads) and the gates.

## Exit criteria
| Criterion | Met? | Evidence |
|---|---|---|
| The v1 clusters, `draw_ext`, the v1 half of `fh6-clusters/src/lib.rs` and their tests are gone (findings 8 and 9) | ✅ | `crates/fh6-clusters/src/legacy/` (31 files), `draw_ext.rs` and `tests/telemetry_sensitivity.rs` deleted; `lib.rs` is the module list and its doc |
| The `fh6-render` crate is gone, after D moved `style_to_theme` | ✅ | D landed `ClusterTheme`'s `FromStr` (`21b591f`); the crate, its workspace member, the `tiny-skia` workspace dependency and `fh6-osd`'s manifest line are deleted |
| `fh6-gpu`'s Phase 4 quad pipeline and test cluster are gone, `StackStr` lives on (finding 11) | ✅ | `quad.rs`, `shaders/quad.wgsl`, `test_cluster.rs`, `seven_seg.rs`, `offscreen.rs` and `tests/offscreen_render.rs` deleted; `OverlaySurface` builds no quad renderer; `StackStr` is `fh6_kit::StackStr` |
| `rev_limiter.rs` (finding 10) and `preview-export --legacy` are gone | ✅ | Deleted; `--legacy` is now an unknown argument |
| The three unused dependencies are fixed (finding 19) | ✅ | `fh6-app` and `preview-export` no longer name `fh6-render` (nor `preview-export` `tiny-skia`); `web-preview` takes `fh6-kit` and `fh6-clusters` on wasm32 only, where `src/web.rs` uses them. Also gone: `fh6-gpu`'s `bytemuck`, `fh6-clusters`' `fh6-render` and `tiny-skia`. `tiny-skia` 0.11 is out of the lockfile (resvg's 0.12 is the only copy) |
| The ledger stops calling per-cluster sets "v1" | ✅ | `ledger::other_origin` labels `assets/clusters/*`; `other_files` skips files that have a record; two tests |
| Nothing that runs behaves differently | ✅ | Every test that went was in a deleted file; no remaining test changed its result (cloud and PC runs below); the goldens pass; live run below |
| Docs no longer point at deleted code | ✅ | Rule files and live docs updated (list below); history left as it was |
| fmt, check, clippy `-D warnings`, `cargo test --workspace` with the goldens, parity and the wasm build pass on the PC | ✅ | "Verification output" |

## What was built
Removed (about 10,700 lines):
- **`crates/fh6-clusters`**: `src/legacy/` (the 29 tiny-skia clusters and their `mod.rs`), `src/draw_ext.rs`, the v1 half of `src/lib.rs` (`LegacyClusterRenderer` and its `*Renderer` structs, `create_cluster`, `render_cluster`, and the v1 wash with its tests), `tests/telemetry_sensitivity.rs`; the `fh6-render` and `tiny-skia` dependencies.
- **`crates/fh6-render`**: the whole crate (theme dispatch, layout kit, tokens, text, asset cache, paint pool, the CPU scrim, mount and wash twins).
- **`crates/fh6-gpu`**: `quad.rs`, `shaders/quad.wgsl`, `test_cluster.rs`, `seven_seg.rs`, `offscreen.rs`, `tests/offscreen_render.rs`; `GpuError::Readback` (only the offscreen target produced it); the quad renderer field and the quad `render` method of `OverlaySurface`; the `bytemuck` dependency.
- **`crates/fh6-core`**: `rev_limiter.rs`.
- **`tools/preview-export`**: `--legacy` and its `mod legacy`; the `fh6-render` and `tiny-skia` dependencies.

Changed:
- **`crates/fh6-kit/src/stack_str.rs`** (new): `StackStr`, moved unchanged from `fh6-gpu::seven_seg` with `#[must_use]` on `new` and `as_str`, and its two tests (one new: a failed write keeps the text before it, `clear` empties). Exported as `fh6_kit::StackStr`; `fh6-app`'s stats text imports it from there.
- **`crates/fh6-gpu/src/lib.rs`, `surface.rs`, `Cargo.toml`**: the crate is the device and the overlay surface; the doc points offscreen work at `fh6_kit::testing`.
- **`tools/preview-export/src/gallery.rs`**: `write_index` lost its `legacy` argument and the v1 section; the scrim heading no longer says "(v1 renderer until Phase 8)", because the scrim renders have come from the kit since Phase 8.
- **`tools/web-preview/Cargo.toml`**: `fh6-kit` and `fh6-clusters` in the wasm32 target table, with a comment.
- **`tools/asset-forge/src/ledger.rs`**: files under `assets/clusters/` are "Per-cluster set built by that cluster's asset-forge recipe" with the licence "As listed in the folder's `ASSETS.md`" (it was "v1 project texture from the initial import" and MIT, which is wrong for SDXL outputs); a file with a record is no longer listed again in the other-files table, which is now headed "Files without a record in this ledger" (TeslaAdas's 12 files were listed twice). `assets/ASSETS.md` is the shared ledger, so it was not regenerated here: the next `asset-forge build` writes the new labels.
- **Manifests**: root `Cargo.toml` (member and workspace dependency), `fh6-app`, `fh6-clusters`, `fh6-gpu`, `fh6-osd`, `preview-export`, `web-preview`; `Cargo.lock`.
- **Rule files and docs**: `AGENTS.md` (the v2 renderer line, the `fh6-core`, `fh6-clusters`, `fh6-osd`, `fh6-gpu`, `preview-export` and `web-preview` rows, the `fh6-render` row removed, the render-code rule names `fh6-kit`), `ONBOARDING.md` and `README.md` (repo maps), `SKILLS.md` (context7 row, token flow), `docs/PROGRESS.md` (state of the code, known defects 3, 5 to 8 and 11, backlog P5 and G2, four decision-log rows), `docs/design/TOKENS.md`, `docs/design/SCRIM_FIT.md`, `program/phases/07-cluster-factory/CATALOG.md` (where to read a v1 cluster now), `docs/specs/phase-8-materials-and-shaders.md` (items 3 and 4), `data/design_tokens.json` (`meta.legacyKeys` note only).

## Verification output
### On the PC (rustc 1.100.0, clippy 0.1.100), commit `b123a71`
In an isolated clone under the checkout's `target\fix-e\` with its own target folder; the tests, `preview-export` and the live run held `gpu.lock` (taken 23:00:31 UTC, after fix F released it, and removed after the live run). Each command's last line as the PC session reported it:
```
$ cargo fmt --all -- --check
exit 0
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 10s
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.00s     (no lints)
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 59.44s
$ cargo check -p web-preview --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 24.13s
$ cargo test --workspace --no-fail-fast
exit 0; 46 'test result' lines, all ok: 911 passed, 0 failed, 1 ignored (the fluid.rs doctest)
golden_images: 49 passed in 98.85 s; golden_telemetry passed
$ cargo run -p preview-export -- --out target\fix-e\mockups
Export complete: 192 kit renders     (index.html written, 106,693 bytes)
```

### Final run on the PC, commit `e629575` (the review fixes and this handover)
The same clone and target folder, at `e629575` with a clean `git status`. The full test run held `gpu.lock` (taken 23:29:52 UTC, after fix D's run released it, and removed by the run's exit trap):
```
$ cargo fmt --all -- --check
exit 0
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.68s
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.43s     (no lints)
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.05s
$ cargo check -p web-preview --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.30s
$ cargo test --workspace --no-fail-fast
exit 0; 46 'test result' lines, all ok: 911 passed, 0 failed, 1 ignored (the fluid.rs doctest)
golden_images: 49 passed in 97.08 s
```

### Live run (release build, the stand-in game window, `mock-udp --scenario bmw_m4_cs` at 60 Hz, `--stats`, 25 s)
Bosch DDU, cockpit view, 1920x1080 at 300 Hz, on `b123a71`:

| Measure | p50 | p99 |
|---|---|---|
| CPU per frame | 0.674 ms | 0.901 ms |
| Packet to present | 0.685 ms | 1.463 ms |

3,462 frames presented, 1,463 packets shown; the report says both targets are met and "--stats overlay drawn: yes"; the app exited with code 0. The host CPU phase measured p99 0.78–0.88 ms on three dashes, so this is within the usual spread. The screen capture taken during the run shows only the stand-in window (see "Visuals").


### Cloud checks (Linux container, rustc 1.97; Windows code cross-checked)
```
$ cargo fmt --all -- --check --config newline_style=Unix
exit 0
$ cargo check --workspace --all-targets --exclude asset-forge --target x86_64-pc-windows-msvc
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.65s
$ cargo clippy --workspace --all-targets --exclude asset-forge --target x86_64-pc-windows-msvc -- -D warnings -A clippy::needless_range_loop
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.22s
$ cargo clippy -p asset-forge --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.75s
$ cargo check -p web-preview --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.28s
$ cargo build -p fh6-kit --target wasm32-unknown-unknown
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.77s
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ cargo test --workspace --no-fail-fast --exclude fh6-app --exclude screen-capture --exclude stand-in-game --exclude wgc-bench
passed 755 failed 88 ignored 1   (main 21e8a06 in the same container: 788 / 91 / 1)
```
The 88 failures are the container's known reds, the same tests as on main: the GPU tests (no adapter here), four `asset-forge` `cloud::remote` tests and one `fh6-win` camera test. `-A clippy::needless_range_loop` skips one lint that clippy 1.97 raises in `liquid_glass/mod.rs:790` on main as well; the PC's clippy does not raise it.

Test by test against main: the 44 tests that went are all in deleted files (`fh6-render`'s 22, `fh6-gpu`'s quad, seven-segment, test-cluster and offscreen tests, `rev_limiter`'s 4, the v1 wash's 3 and `telemetry_sensitivity_all_clusters`). Added: the two `StackStr` tests, the two ledger tests and D's four `theme` tests. No test changed its result.

## Visuals
No images are committed. A phase that changes nothing drawn has no visual review; the one drawn text it touches is the `--stats` lines (`StackStr` moved unchanged), and the live bench reports them drawn. The desktop capture of the live run (`screen-capture`, on the PC under `target\fix-e\evidence\`) shows only the stand-in window, without the overlay; why is not known (the PC session's guess, not verified: GDI capture does not see the DirectComposition swapchain). The progress cards posted in the thread summarise the cloud checks.

## My calls
1. **Two commits, one landing.** Step 1 (everything but `fh6-render`) was ready before D's theme move; D landed it at 22:34 UTC (`21b591f`), so step 2 deletes the crate and both land together.
2. **`web-preview`'s `fh6-kit` and `fh6-clusters` move to the wasm32 table instead of going.** The assessment read them as unused, but the browser page (`src/web.rs`, wasm32 only) uses both; on the native build they were unused, which was cargo's warning.
3. **`StackStr` goes to `fh6-kit`, not `fh6-app`.** The kit is where frame-path text lives, and a cluster may want it; the code moved unchanged.
4. **`GpuError::Readback` goes with `offscreen.rs`,** its only producer. Every user of `fh6-gpu` is in this workspace and compiles.
5. **The ledger's new label sends the reader to the per-cluster `ASSETS.md` for the licence,** because those sets mix MIT and SDXL (CreativeML Open RAIL++-M) outputs.
6. **History stays as it was:** finished-phase handovers and MASTER_PROMPTs, ADRs, the decision log's earlier rows and the older "State of the code" sections. Live docs that sent a reader to deleted code were changed, with `git show 21e8a06:crates/fh6-clusters/src/legacy/<snake_name>.rs` as the way to read a v1 cluster.
7. **No assets deleted.** The root v1 textures (`assets/*.png`, `assets/textures/4k/`) are now loaded by nothing, but deleting them changes the shared ledger; known defect 7 now says so.
8. **The v1 token keys stay** (`typography`, `sharedSemantics`, `clusters.*`): no code reads them, they are reference for the rebuilds, and the note in `meta.legacyKeys` and `TOKENS.md` now says that.
9. **`shift_guard` and `trail` stay.** They are the rest of finding 10, outside E's scope.

## Known issues / debt
- **The root v1 textures and the token keys that name them** (known defect 7): delete both copies, `glass.caustic.asset` and the legacy `clusters.*.assetBindings` in an asset pass that rebuilds the ledger; `assets/gt7_hood_pro_plate.png` is the one the Gt7HoodPro handover asked to retire.
- **No telemetry-sensitivity test for the kit clusters** (backlog G2): the v1 test went with the v1 clusters; today the four golden states per cluster are the only check.
- **`assets/ASSETS.md` still shows the old labels** until the next `asset-forge build`.
- **Clippy 1.97 (the cloud container's) raises `needless_range_loop` at `crates/fh6-clusters/src/clusters/liquid_glass/mod.rs:790` and `crates/fh6-clusters/tests/component_classic.rs:110`** (on main too; the PC's 1.100 raises neither). Not E's files.
- **On a Linux `cargo check`, `fh6-input` warns that `XInputStateEx` is never constructed** (it is used only on Windows). Pre-existing.

## What the next phases must know
- **A v1 cluster's source:** `git show 21e8a06:crates/fh6-clusters/src/legacy/<snake_name>.rs`; `fh6-render` is at `21b591f:crates/fh6-render/`.
- **Offscreen rendering** for tests, benches and tools is `fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT}`; `OverlaySurface::render_with` is the only live presentation path.
- **`fh6_kit::StackStr<N>`** formats numbers on a frame path without allocating (`write!`, then `as_str`).
- **Style names** parse with `ClusterTheme`'s `FromStr` (fix D); nothing else maps names to themes.
- **For G:** `FH6TelemetryData::normalized_rpm` has no caller left.
- **For D:** the comment at `crates/fh6-core/src/profiles.rs:67` ("kept for the legacy renderers") is stale; no renderer reads those multipliers.
- **For the coordinator:** run `cargo run -p asset-forge -- sheet` on the Windows checkout (no GPU; it rewrites only `assets/ASSETS.md`) and commit it when no dash thread is building assets, so the ledger shows the new labels. `assets/assets.json` still records `assets/clusters/tesla_adas/horizon_haze.png`, which no longer exists; a `build --set tesla_adas` clears it.

## Closing review
A fixed list, written from the coordinator's brief, `AGENTS.md` and `SKILLS.md`. Each line is yes or no, with its evidence. No scores.

How it was checked: a `rust-review` pass on the diff, then an independent agent that was given the code and the scope (not this thread's conclusions) and rebuilt base and head from clean exports. Their findings and what was done are under "Review findings".

**A. The brief**
1. Findings 8 and 9: the v1 clusters, `draw_ext`, the v1 half of `fh6-clusters/src/lib.rs` and their tests are gone. **Yes.** `git show --stat` of the two commits; `lib.rs` is the module list.
2. `fh6-render` is gone, after D moved its last caller, and nothing names it. **Yes.** D's `21b591f` is the base; `git grep fh6_render` finds nothing, and `"fh6-render"` survives only as the render thread's name.
3. Finding 11: the quad pipeline and the test cluster are gone, and `StackStr` lives on unchanged. **Yes.** The independent review compared the code with the deleted `seven_seg.rs` version: identical apart from `#[must_use]` and the docs.
4. `rev_limiter` and `preview-export --legacy` are gone. **Yes.** `--legacy` now stops with "unknown argument".
5. Finding 19: the unused dependencies are gone and nothing was added. **Yes.** `cargo tree -e features`, base against head, for every trimmed crate: identical except that `fh6-render`, `tiny-skia` 0.11.4 and `tiny-skia-path` 0.11.4 are gone; `Cargo.lock` holds one `tiny-skia` (0.12).
6. The ledger labels per-cluster sets correctly and lists a recorded file once. **Yes.** `ledger::tests`, three tests; both new ones fail on the old logic (independent review).
7. E kept to its files and told D, F and G before merging. **Yes.** Messages at 22:48–22:50 UTC; F and D answered that they rebase onto E.
8. Docs no longer point at deleted code, and history is left as it was. **Yes.** The independent review's `git grep` for every deleted file, module, type, function, flag and crate; its hits are fixed or are the history listed in "My calls".

**B. Hard rules**
9. No code reads, binds or requires A, B, X or Y. **Yes.** No input code changed. [AGENTS 4]
10. No placeholder, `todo!()` or stub added. **Yes.** [AGENTS 1, 2]
11. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. **Yes.** `stack_str.rs` and `ledger.rs` use them only in tests. [AGENTS 6]
12. The per-frame path adds no heap allocation, file I/O, logging or locking. **Yes.** `render_loop.rs` changes one import; `StackStr` is the same code. [AGENTS 7]
13. The frame budget holds. **Yes.** No draw code changed; the live bench measured CPU per frame p50 0.674 ms and p99 0.901 ms. [AGENTS 7]
14. Telemetry still reaches the screen on the next frame. **Yes.** The loop is unchanged. [AGENTS 3]
15. No new path draws while the app hides the dash. **Yes.** [AGENTS 9]
16. No non-Rust source file was added. **Yes.** A shader was deleted; the additions are Rust, Markdown and JSON text. [AGENTS 11]
17. Commits are by TJX with no co-author trailer; no tracked file holds a real name, a `C:\Users\<name>` path, a personal email or a mention of the legacy build. **Yes.** `git log --format='%an <%ae> | %(trailers:only)'`; a search of the diff found none.
18. Nothing was copied into the game folder or the mirror. **Yes.** [AGENTS 8]
19. fmt, check, clippy `-D warnings`, `cargo test --workspace` with the goldens, parity 20/20 and the wasm builds pass on the tree that lands. **Yes.** "Verification output".

**C. Rendering laws**
20. Colour stays premultiplied linear end to end. **Yes.** No shader or blend state was added; `quad.wgsl` was deleted.
21. Off pixels stay `#000000`, with no full-canvas fill. **Yes.** Nothing drawn changed; the goldens pass.
22. Nothing new reads a clock. **Yes.**

**D. Thread rules**
23. The thread opened with an introduction and is not archived. **Yes.** [AGENTS 13]
24. The GPU lock was held for every GPU job and released after it. **Yes.** No GPU job ran in the cloud; on the PC the lock was taken at 23:00:31 UTC for the tests, `preview-export` and the live run and removed afterwards, and again at 23:29:52 UTC for the final test run, removed when it ended. [SKILLS 5]
25. A `rust-review` pass and an independent review ran, and their findings are applied or listed. **Yes.** "Review findings".
26. The review is yes/no only, with no scores anywhere. **Yes.** [AGENTS 12]
27. The handover follows `program/HANDOVER_TEMPLATE.md`, and `docs/PROGRESS.md`, `AGENTS.md` and `SKILLS.md` match the code. **Yes.**
28. The shared checkout's branch was never switched; the PC run used an isolated clone. **Yes.**

## Review findings
`rust-review` (diff only; changed lines are Rust library and tool code):
- Consider: `#[must_use]` on `StackStr::new` and `as_str` (clippy pedantic). Applied.
- Consider: one `use` per module path in `render_loop.rs` (`StackStr` joined the `fh6_kit::{..}` list). Applied.
- Consider: the ledger test reused a fixed temp folder without clearing it. Applied (it clears it first).
- Consider: `StackStr` could implement `Display`. Not applied: no caller needs it, and the move stays a move.
- Consider: removing `GpuError::Readback` would break an outside user. No action: every user is in the workspace.

Independent review (head `b123a71` against `21b591f`, clean exports rebuilt on both):
1. The new decision-log row cited this handover before it was committed. It lands in the same merge.
2. The committed `assets/ASSETS.md` still shows the old labels and the TeslaAdas duplicates. Left: it is the shared ledger. The decision-log row says it catches up on the next `asset-forge sheet` (no GPU) or `build`; the follow-up is below.
3. The Phase 8 spec (a heading, `render_oem_scrim`, `BinnacleAmbientWash`) and the backlog bullet "`AssetCache` blitting" named deleted code. Fixed.
4. `Kind::Rebuild`'s doc said the 29 v1 themes are rebuilt; 6 are. Fixed ("which Phase 7 rebuilds on the kit").
5. Output that changed beyond the deletion: the gallery's scrim heading, the ledger's other-files heading and labels, `OverlaySurface` no longer building a quad renderer (cheaper start, frames unchanged), `GpuError::Readback`. All intended; listed under "What was built".
6. The decision-log row said about 9,300 lines (step 1 alone). Fixed to about 10,700.
7. The ledger's doc said "a file asset-forge did not make", and a per-cluster `ASSETS.md` row pointed at itself for its licence. Fixed: the doc says "a file that has no record in this ledger", and a per-cluster ledger gets its own label and MIT; the test covers it.
8. Known defect 7 implied `AssetCache` loaded the root textures (nothing called it) and missed that the Phase 8 spec names them. Reworded.
9. The Liquid Glass handover's follow-up still said the v1 renderer is wired in `lib.rs` and the sensitivity test. A dated note says both are gone.
10. `crates/fh6-core/src/profiles.rs:67` says its scale multipliers are "kept for the legacy renderers", which never read them (pre-existing). Left for D, whose phase owns `profiles.rs`.
11. `docs/design/TOKENS.md` gave the tokens version as 3.1.0; the file is 3.2.0 (pre-existing, on a line this diff edits). Fixed.
12. `assets/assets.json` records `assets/clusters/tesla_adas/horizon_haze.png`, which is neither in git nor on disk (pre-existing). Listed below.
13. Clippy 1.97 raises a second `needless_range_loop`, in `crates/fh6-clusters/tests/component_classic.rs:110` (pre-existing; 1.100 does not). Listed under "Known issues".
14. The `StackStr` test name promised more than it tested: a `write!` with several arguments keeps the pieces written before the one that did not fit. The test now checks that case, under a new name, and the doc says so.
15. History (the Phase 7 setup handover, PROGRESS's runtime audit, `docs/TELEMETRY_USAGE_MATRIX.md`'s dated snapshot, a quoted build line in the PorscheAnalog handover). Left as history.

## Sign-off
TJX asked on 2026-11-27 (16:00 UTC) for threads to review their own work with the installed review skills and agents, and not to wait for a sign-off. This closing review stands in for it.
