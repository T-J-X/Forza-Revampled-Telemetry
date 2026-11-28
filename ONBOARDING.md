# ONBOARDING: read this first in every chat

You are joining **FH6 WheelDash v2**, a Rust overlay for Forza Horizon 6. Work runs as threads: one thread per phase, and one per dash. A coordinator starts each one, and merges its work into `main` when the job is finished and its checks pass. Each thread reviews and signs off its own work (AGENTS.md rule 12); TJX's own words win when TJX answers. This page is everything a new chat needs before touching the repo.

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
crates/fh6-core      telemetry decoder (golden-tested), UDP engine, config, car/brand data, cockpit fit model
crates/fh6-gpu       wgpu DX12 device and the premultiplied overlay surface
crates/fh6-kit       the GPU render kit every v2 dash draws with: shapes, glass, text, springs, tokens, scrim and wash
crates/fh6-clusters  the catalog (48 rows) and one module per dash
crates/fh6-osd       OSD model and rendering
crates/fh6-input     XInput (ABXY lock)
crates/fh6-win       window and Win32, camera patcher
crates/fh6-app       binaries FH6_WheelDash and FH6_SteamLauncher
tools/               preview-export (4K renders -> mockups/), visual-review (review images of those renders, CPU only),
                     asset-forge (Blender and ComfyUI assets), mock-udp, parity-dump, bench-render, web-preview,
                     wgc-bench, screen-capture, stand-in-game
tests/golden/packets 20 golden telemetry packets with expected JSON. Never edit them to make a test pass.
assets/              textures, logos/, fonts/, car_brands/, clusters/<snake_name>/, reference/ photos
data/                config.json, car profiles, ordinals, design_tokens.json, game camera files
docs/                PROGRESS.md (status), specs/, design/, kit/ (the kit guide), adr/
program/             the build program: roadmap, per-phase MASTER_PROMPT / AGENTS / SKILLS, handovers
```

## 3. Rules that are never broken
1. **Manual gearbox lock.** A `0x1000`, B `0x2000`, X `0x4000` and Y `0x8000` are the gearbox, handbrake and rewind. No code may read, bind or require them. Use the D-pad, LB/RB, View, Menu, L3/R3 or Guide.
2. **Deploy guard.** Never copy anything into the game folder (`E:\SteamLibrary\steamapps\common\ForzaHorizon6\`) or the mirror (`%USERPROFILE%\Downloads\FH6 Ultimate Camera\`), and never edit live game files, without TJX's explicit GO in chat.
3. **Public repo.**
   - Commit as **TJX <TJX@Ngage.com>**; the repo config is already set, so check with `git config user.email`.
   - **Add no `Co-Authored-By` trailer.**
   - Never put a real name, a `C:\Users\<name>` path or a personal email in a file. Use `%USERPROFILE%`.
4. **Verify with real output.** Show the real output of `cargo check`, `cargo test` and `cargo clippy` before calling anything done.
5. **97% rule.** When unsure about an offset, bitmask, coordinate or crate API, look it up (context7 for crates, the golden packets for telemetry). The golden packets are synthetic and cover only the fields `mock-udp` writes; any other offset is unverified until it is checked against the real game. Ask only if it truly blocks.
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
- **ComfyUI portable:** `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable\`. asset-forge starts it when it is not running (`run_nvidia_gpu.bat` starts it by hand); the API is on `127.0.0.1:8188`, and every thread shares it, so never stop it. Installed models: SDXL base 1.0 and refiner, the depth and canny ControlNets and 4x-UltraSharp (`cargo run --release -p asset-forge -- doctor` lists them). A 4096×4096 texture fits the 8 GB card.
- **Two GPU lanes, cloud first:** a rented 24 GB pod that takes two jobs at once (`gpu-cloud.lock`, `gpu-cloud-2.lock`) runs every asset-forge ComfyUI and Blender job (`--lane cloud`), and the local card (`gpu.lock`, one job at a time) keeps `preview-export`, the goldens and the live bench. Every GPU job holds a lock file in `%USERPROFILE%\Documents\Dash_Tools\` while it runs. `SKILLS.md` section 5 lists the GPU jobs (any `cargo test` that renders is one), gives the command that takes a lock, and has the rules and the $50 budget of the paid lane. `docs/cloud-gpu.md` is the guide.
- **4K masters:** the Google Drive desktop app mounts the folder `FH6 WheelDash v2 - 4K Masters` at `G:\My Drive\FH6 WheelDash v2 - 4K Masters\` (check the drive letter). Write masters there, one subfolder per dash plus `shared\`, and it syncs. Git keeps only the compressed runtime assets. The Drive connector can't upload files over about 7 MB.
- **Penpot** (the design tool since 2026-11-23): Penpot's hosted MCP is registered at user scope as the `penpot` server (`execute_code`, `high_level_overview`, `penpot_api_info`, `export_shape`). It only reaches the design file while that file is **open in a browser tab and connected** (main menu ⋮ → MCP server → Connect; a green ● MCP badge shows in the toolbar), so the tab must stay open. Call `high_level_overview` once before the first `execute_code`. Setup, limits and the master file are in `docs/design/PENPOT.md`. Never ask for the MCP key or write it into a file. HTML boards are the fallback.
- **context7 MCP** for crate docs (it needs no API key); the **built-in browser** for viewing `mockups/index.html` and HTML boards, served over loopback (a `file://` page loads without its images; see `SKILLS.md` section 2).

## 5. How a thread works (TJX, 2026-11-27)
1. Read in this order: the approved plan (the coordinator gives you the link) → this file → root `AGENTS.md` → root `SKILLS.md` → your phase's files in `program/phases/<phase>/` (`MASTER_PROMPT.md`, `AGENTS.md`, `SKILLS.md`; a dash thread follows `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`) → `docs/PROGRESS.md` → the `HANDOVER.md` of every phase you depend on.
2. **Open with an introduction** to TJX in your thread: the task, the rule files that apply, and what done looks like.
3. Load the skills your phase's `SKILLS.md` lists, at the step it names. Root `SKILLS.md` says how to call a skill or an MCP tool.
4. Work without check-in questions. Stop for TJX only where the rules ask for TJX: a sign-in, a paid service, a deploy, and anything that cannot be undone. TJX is hands-off on reviews (AGENTS.md rule 12): you run the direction check on design work and the closing review yourself with the review skills, post the results, and carry on. Record your other decisions as "my call" in your handover. The downloads TJX pre-approved are listed in `program/README.md`.
5. Hold the lock of its lane for every GPU job (`SKILLS.md` section 5). Threads can share one checkout: commit as TJX with explicit paths, and never switch its branch. If you push a branch of your own, open a draft pull request with its first push (AGENTS.md rule 17).
6. **Review with a fixed checklist,** line by line against `AGENTS.md` and `SKILLS.md`, with yes/no answers. No scores and no critic loops. A dash uses the checklist in `CLUSTER_PROMPT.md`. Every render, board and motion sequence is reviewed with the `fh6-visual-review` skill and its `visual-review` tool: Claude sees a 4K frame only at about 0.4x, so judge detail on 1:1 tiles, and measure what eyes get wrong.
   Post a short update with a screenshot at each milestone and at least every 20 to 30 minutes (TJX, 2026-11-27); `visual-review annotate` makes it.
7. **Finish:**
   1. Fill in your `HANDOVER.md` from `program/HANDOVER_TEMPLATE.md`.
   2. Update `docs/PROGRESS.md`. A dash thread only appends decision-log rows there (a new dependency or token); the coordinator keeps the status table.
   3. Commit, and report the finished job to the coordinator. Once its checks pass, the coordinator merges it into `main` and pushes.
   4. Post the result and the completed checklist in your thread. Your self-review stands in for TJX's sign-off (AGENTS.md rule 12); where TJX answers later, apply what TJX asks. The thread is never archived.
   5. Last, if your thread pushed a branch, once the coordinator has told you your work is in `main` and pushed: make sure your pull request shows as merged (if it still shows open, follow AGENTS.md rule 17 before closing it; never merge it yourself), then check with `git ls-remote origin` whether your thread's branch is still on GitHub. Do not delete it yourself. Say in your closing post and to the coordinator whether it is still there, with its name and head (AGENTS.md rule 17).
