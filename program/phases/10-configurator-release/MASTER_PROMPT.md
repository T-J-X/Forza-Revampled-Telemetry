# Phase 10: configurator (`index.html`) & release (MASTER PROMPT)

You lead Phase 10 of FH6 WheelDash v2, running in your own chat and worktree. **The set-and-forget experience:** the user opens the configurator once, picks and tunes their dash with a pixel-identical live preview, copies one Steam launch option, and from then on the overlay starts and stops with the game. Then you ship the release.

## Read first
`ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`, the handovers of Phases 3, 4, 5, 8 and 9 and the finished cluster handovers, `docs/design/*` (the configurator direction frame), `docs/kit/README.md`, `tools/web-preview/` (Phase 5), and `crates/fh6-app/src/main.rs` (the widget server on 127.0.0.1:5301).

## Required work
1. **Design in Penpot first** on page `10 · Configurator` of the Penpot master file (`docs/design/PENPOT.md`). It is a dark, premium, motorsport web app that matches the OSD's design language. Review it against a fixed yes/no checklist, and run the direction check (a self-review, AGENTS.md rule 12) before you build. No scores and no critic loops (AGENTS.md rule 12).
2. **Served by the exe** at `http://127.0.0.1:5301/`:
   - The HTML shell, the WASM renderer and the assets are embedded in the exe (pure Rust, for example `include_bytes!` or `rust-embed`).
   - Opened from the tray, or with `FH6_WheelDash.exe --configure`.
   - **Rust only:** page logic in Rust compiled to WASM (`web-sys`, a WebGPU canvas). HTML and CSS are the only non-Rust parts, and the wasm-bindgen JS loader is a build artefact.
   - Replace the old inline widget page.
3. **Live preview:** the real `fh6-kit` and cluster code running in WebGPU, the same renderer as the overlay.
   - The user picks a background: transparent over the cockpit reference photo, or plain black.
   - **Hold Space** to rev with spring physics, and the rev limiter and strobe fire at the redline.
   - **Keys 1–9** select a gear (N with 0 and R with `-` are optional), with the gear-change animation.
   - The preview runs on a local dyno with no round trip to the exe, so it reacts on the next frame.
4. **Controls,** all written to `config.json` over a small JSON API (`GET`/`PUT /api/config`). The overlay hot-reloads (Phase 4) and reflects each change immediately.
   - **Cluster pickers:** a cluster per view, through a gallery of live thumbnails.
   - **Colours:** factory (brand by car: search and pick a car or brand), custom (colour pickers per palette role) or randomise (with re-roll and lock).
   - **Fonts:** the bundled pack, or drop-in upload. A dropped font is copied to the user `fonts/` folder.
   - **Name and logo:** logo on or off, and the custom name.
   - **Camera:** FOV per view, seat height and wheel offset (Phase 8 patcher, showing "applies when the game is closed or restarted").
   - **Layout and scrim:** cluster height and scale, and the dash blocker opacity and fit, previewed over the cockpit photo.
   - **Other:** units, shift % and the redline strobe.
5. **Set and forget, a step-by-step panel:**
   1. **Forza data:** in Forza, set Settings → HUD → Data Out to ON, IP `127.0.0.1`, port `8000`. Show it with a screenshot-style diagram.
   2. **Steam launch option:** a copy button for `"<actual install dir>\FH6_SteamLauncher.exe" %command%`, with the install path filled in from the exe's own location.
   3. **Display mode:** borderless or windowed fullscreen, so the overlay can show.
   4. **Self-test:** a status light shows "telemetry received" live when the game sends data.
6. **Release:**
   - A `cargo build --release` profile (LTO, `codegen-units=1`, `strip`, `panic=abort` where safe).
   - A portable zip containing `FH6_WheelDash.exe`, `FH6_SteamLauncher.exe`, `config.json`, `assets/` (if not embedded), `fonts/`, `README.txt` and `LICENSE`.
   - `docs/RELEASE.md` checklist, made with `engineering:deploy-checklist`. The `engineering` plugin is not enabled (checked 2026-11-27): ask TJX to enable it, or write the checklist from Agent 4's deploy rules in `AGENTS.md`.
   - A GitHub Release (tag `v2.0.0`) with the zip, created through the GitHub web UI or the API using the user's existing git credentials. If neither works, prepare everything and leave the publish step for TJX.
   - **Deploy guard:** never copy into the game folder or the mirrors without TJX's GO.

## Exit criteria
- Workspace check, test and clippy are green.
- The configurator works in the built-in browser against the running exe: preview, Space and 1–9, every control, and config persisting and hot-reloading into the overlay. Screenshots are attached.
- A pixel comparison of the WebGPU preview against `preview-export` for 3 clusters is within tolerance.
- The release zip builds and the release checklist passes.
- An end-to-end run is recorded, with `mock-udp` standing in for the game:
  1. Launching through the launcher with a stand-in game process starts the overlay.
  2. Closing the stand-in process makes the overlay exit.
