# ONBOARDING: read this first in every chat

You are joining **FH6 WheelDash v2**, a Rust overlay for Forza Horizon 6. Work runs as a chain of chats: one chat per phase, and one per cluster in the cluster factory. A conductor chat starts each one, verifies what it delivers, and merges it. This page is everything a new chat needs before touching the repo.

## 1. What we are building
**The overlay:**
- The widest and most beautiful set of 4K instrument clusters for Forza Horizon 6, mainly for **cockpit view**: classic analog, modern digital (GT7 standard, but 2027), out-of-this-world sci-fi, fluid animation, and a Tesla-style ADAS view with live traffic.
- It is driven by Forza's UDP telemetry (324 bytes, little-endian, port 8000).

**Around it:**
- **The OEM dash blocker:** a domed, OLED-black scrim that hides the game's own gauges so ours sit cleanly in the binnacle.
- **The OSD:** a full-screen, controller-first Liquid Glass menu with a live view. It sets the cluster, colours (factory, custom or random), font, FOV, seat height and cluster height.
- **`index.html`:** a set-and-forget configurator served by the exe. It has a live WebGPU preview (hold Space for throttle, press 1–9 for gears) and the Steam launch option to copy.

**Quality bar:**
- Response time must feel like 0 ms.
- The dash is never visible in menus or when paused.
- The look must rival or beat production 2027 clusters. Anything that looks like flat rectangles is a failure.

## 2. Repo map
```
crates/fh6-core      telemetry decoder (golden-tested), UDP engine, config, car/brand data, rev limiter, trail
crates/fh6-render    render layer (moving to wgpu in Phase 5), scrim and ambient wash, tokens
crates/fh6-clusters  one module per cluster
crates/fh6-osd       OSD model and rendering
crates/fh6-input     XInput (ABXY lock)
crates/fh6-win       window and Win32, camera patcher
crates/fh6-app       binaries FH6_WheelDash and FH6_SteamLauncher
tools/               preview-export (4K mockups -> mockups/), mock-udp, parity-dump, bench-render
tests/golden/packets 20 telemetry captures with expected JSON. Never edit them to make a test pass.
assets/              textures, car_brands/, reference/ photos; logos/ and fonts/ arrive in Phase 6
data/                config.json, car profiles, ordinals, figma_tokens.json, game camera files
docs/                PROGRESS.md (status), specs/, design/ (from Phase 3)
program/             this build program: roadmap, per-phase MASTER_PROMPT / AGENTS / SKILLS, handovers
```

## 3. Rules that are never broken
1. **Manual gearbox lock.** A `0x1000`, B `0x2000`, X `0x4000` and Y `0x8000` are the gearbox, handbrake and rewind. No code may read, bind or require them. Use the D-pad, LB/RB, View, Menu, L3/R3 or Guide.
2. **Deploy guard.** Never copy anything into the game folder (`E:\SteamLibrary\steamapps\common\ForzaHorizon6\`) or the mirror (`%USERPROFILE%\Downloads\FH6 Ultimate Camera\`), and never edit live game files, without TJX's explicit GO in chat.
3. **Public repo.**
   - Commit as **TJX <TJX@Ngage.com>**; the repo config is already set, so check with `git config user.email`.
   - **Add no `Co-Authored-By` trailer.**
   - Never put a real name, a `C:\Users\<name>` path or a personal email in a file. Use `%USERPROFILE%`.
4. **Verify with real output.** Show the real output of `cargo check`, `cargo test` and `cargo clippy` before calling anything done.
5. **97% rule.** When unsure about an offset, bitmask, coordinate or crate API, look it up (context7 for crates, the golden packets for telemetry). Ask only if it truly blocks.
6. **No placeholders.** Never present a stub, a hardcoded stand-in for live telemetry, or an opaque full-screen fill as finished work.
7. **No mention of the legacy C# build** in repo files.
8. **Rust only.** The repo contains no C, C++, C#, Python, JS, TS, PowerShell or shell source files.
   - The app and every tool are Rust: lightweight, and the reason we can hit "0 ms".
   - **Blender:** it only accepts Python, so `tools/asset-forge` (Rust) holds its bpy snippets as Rust string constants and passes them to Blender when it runs.
   - **ComfyUI:** driven over HTTP from Rust. Workflow JSON is data, not code.
   - **Web pages:** their logic is Rust compiled to WASM. The JS loader that `wasm-bindgen` generates is a build artefact and is never committed.
   - **Dependencies:** a crate that compiles C internally is allowed only with a decision-log entry. Prefer pure-Rust crates.

## 4. Machine and tools
- Windows 11, rustc and cargo 1.100 (MSVC), GPU RTX 3070 Ti Laptop (8 GB).
- Display: LG C1 55" 4K OLED at 3840x2160.
- **Blender 5.3:** `%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64\blender.exe`. Run headless with `blender.exe -b -P script.py`.
- **ComfyUI portable:** `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable\`. Start it with `run_nvidia_gpu.bat`; the API is on `127.0.0.1:8188`. It has no models until Phase 6 installs them.
- **Penpot** (the design tool since 2026-11-23): Penpot's hosted MCP is registered at user scope as the `penpot` server (`execute_code`, `high_level_overview`, `penpot_api_info`, `export_shape`). It only reaches the design file while that file is **open in a browser tab and connected** (File → MCP Server → Connect), so the tab must stay open. Call `high_level_overview` once before the first `execute_code`. Setup, limits and the master file are in `docs/design/PENPOT.md`. Never ask for the MCP key or write it into a file. HTML boards are the fallback.
- **Figma** is the read-only legacy master (`docs/design/FIGMA.md`): read it for history, never write to it. Its Starter plan and View seat block writes.
- **context7 MCP** for crate docs; the **built-in browser** for viewing `mockups/index.html` and HTML boards.

## 5. How a chat works
1. Read in this order: this file → root `AGENTS.md` → root `SKILLS.md` → your `program/phases/<phase>/MASTER_PROMPT.md`, `AGENTS.md` and `SKILLS.md` → `docs/PROGRESS.md` → the `HANDOVER.md` of every phase you depend on.
2. Load the skills your phase's SKILLS.md lists, in order, at the moment it says.
3. Work autonomously. TJX pre-approved the whole program on 2026-11-20, including the downloads listed in `program/README.md`. Don't stop to ask unless the 97% rule truly blocks you; record decisions as "my call" in your handover.
4. Commit early and often on your branch, as TJX.
5. **Finish:**
   1. Fill in `program/phases/<phase>/HANDOVER.md` from `program/HANDOVER_TEMPLATE.md`.
   2. Update `docs/PROGRESS.md`.
   3. Commit.
   4. End with a short report.
   The conductor verifies, merges and starts the next chat.
