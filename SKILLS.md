# SKILLS.md: skills, MCPs and Rust rendering laws

This sits alongside [AGENTS.md](AGENTS.md), which holds the council, the hard rules and the commands. If the two conflict, AGENTS.md and TJX win.

## 1. Which skill when

| Situation | Load first | Notes |
|---|---|---|
| Any new feature, cluster style, OSD change or redesign | `brainstorming` | No code until TJX has approved the design. |
| Writing or reviewing any `.rs` file | `rust-best-practices` | Applies to every crate. |
| Looking up a specific Rust rule: ownership, errors, memory, unsafe, async, API design, performance, anti-patterns | `rust-skills` (265 rules in 26 categories; MIT, by leonardomso, v1.5.1, restored from the earlier setup on 2026-11-27) | Its `SKILL.md` is the index: open only the rule files you need. The repo is edition 2021 and declares MSRV 1.87 (the locked dependencies need about 1.90), so skip a rule that needs a newer edition or compiler. AGENTS.md wins where they differ. |
| Ownership, traits, concurrency (UDP engine, `ArcSwap`, threads) | `rust-patterns` | |
| Adding or fixing tests, golden vectors, property tests | `rust-testing` | Golden packets live in `tests/golden/packets`. |
| Reviewing a crate, module or diff | `rust-review`, with `rust-skills` as the rule reference | Gives a prioritised report with fixes. |
| Porting a reference algorithm (shader maths, kinematics) from another language | `rust-port` | |
| Any drawing: clusters, OSD, scrim, mockups | `fh6-hmi-design-rules` (project skill), then `aesthetic` | The project skill wins on anchors, OLED black and the rank order. |
| Web UI: the configurator page, the showcase site, HTML boards | `frontend-design` (project skill), with `aesthetic` | Page logic stays Rust → WASM (AGENTS.md rule 11). |
| Design-system upkeep and design findings | `design:design-system`, `design:design-critique` (the `design` plugin, enabled) | The `design-superpowers` plugin is not enabled on this account (checked 2026-11-27); don't rely on it. |
| Static concept art, posters, contact sheets | `canvas-design` (user scope) | Outputs go to `mockups/`, never into the repo tree as scripts. |
| Building a custom MCP server for this project | `mcp-builder` (user scope) | Only when a tool has no MCP and plain HTTP from Rust isn't enough. |
| Building any dash | the order in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`: R&D on the web → Penpot → ComfyUI → final 3D in Blender | All assets are 4K. Blender is the last image step. |
| Organic textures and environment plates (the ComfyUI step) | `comfyui-hero-textures` | asset-forge starts ComfyUI itself on `127.0.0.1:8188` when it is not running; never stop that server. Hold the lock of the lane the job runs on (section 5); `--lane cloud` sends the job to the rented pod. |
| Rendered hardware: bezels, dials, needles, crystals (the Blender step) | `blender-hero-assets` | Through `tools/asset-forge`; the `Blender` MCP is for looking and checking. Hold the lock of the lane the job runs on (section 5); `--lane cloud` sends the render to the rented pod. |
| Before merging or pushing a branch | `requesting-code-review`, `/code-review`, `/simplify` | |
| Cleaning up folders and finding duplicates | `file-organizer` | Never touch the deploy targets. |
| Designing, or pulling tokens and layouts from the design file | the `penpot` MCP: `high_level_overview` (once per chat), `penpot_api_info`, `execute_code`, `export_shape`; with `fh6-hmi-design-rules` | Penpot needs its file tab open and connected (`docs/design/PENPOT.md`). |
| Looking at or judging any render, board, screenshot or motion sequence: the direction check, the closing review, a progress post | `fh6-visual-review` (project skill), with `fh6-hmi-design-rules` | The fixed 26-line visual checklist, the procedure that keeps a self-review honest (expectations frozen first, a cold look, one fresh-context reviewer, a blind glance read) and the `visual-review` tool: `cargo run --release -p visual-review -- <command>` (CPU only, no GPU lock; outputs in `mockups/<Name>/review/`). Claude sees a 4K frame only at about 0.4x, so detail is judged on the tool's 1:1 tiles. Findings and yes/no only. |
| Reviewing a dash | the fixed checklist ("Closing review" in `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`), walked by the thread itself with `fh6-visual-review`, `design:design-critique`, `design:accessibility-review` and `rust-review` (steps 5 and 10 there) | TJX is hands-off (AGENTS.md rule 12): the self-review stands in for TJX's direction check and sign-off. No scores and no critic loops: each skill, and the `cold-reviewer` agent that `fh6-visual-review` runs, lists findings once, as input to the checklist; never ask one for a score. |
| Reviewing the OSD or a phase | a fixed yes/no checklist written at the start of the thread from AGENTS.md and SKILLS.md (sections A and B of the dash checklist apply as they are), walked by the thread itself with the same review skills; renders are checked with the `fh6-visual-review` lines that apply | The same rules: a self-review, no scores and no critic loops. |

### How to call them
- **A skill:** call the Skill tool with its name, for example `rust-review`, or `design:design-critique` for a plugin skill. A person types `/<skill-name>`. Project skills live in `.claude/skills/` and are listed to every new session when it starts.
- **An MCP tool:** when a session lists the MCP tools by name only, load them first with ToolSearch, all in one call, then call them. For example `select:mcp__penpot__high_level_overview,mcp__penpot__penpot_api_info,mcp__penpot__execute_code,mcp__penpot__export_shape` or `select:mcp__context7__resolve-library-id,mcp__context7__query-docs`. The Blender tools are named `mcp__Blender__*`. The Google Drive tools carry an account ID in their names, so find them with the ToolSearch query `drive search_files`.
- **ComfyUI and headless Blender** have no MCP tool: run them through `cargo run --release -p asset-forge -- <command>` with the Bash tool. Put `--lane cloud` before the command to run its ComfyUI and Blender work on the rented pod (section 5).
- **An agent:** start it with the Agent tool (named Task in some sessions), once, and ask for findings only. For a visual review the cold reviewer is the project agent `cold-reviewer` (`.claude/agents/cold-reviewer.md`: it reads only the images it is given and gives no verdicts or ratings): give it only the neutral, logo-masked packet that `fh6-visual-review` describes (`references/method.md`). A session that started before that agent was added may not list it: then use `general-purpose` with the agent file's text above the brief. `ui-ux-designer` is for web and app interfaces: its own instructions ask for verdicts, severity ratings and no pure black, so it is not the cold reviewer.
- **The review tool** has no skill of its own: run `cargo run --release -p visual-review -- <command>` from the repo root in the Bash tool. `--help` lists the commands; `fh6-visual-review` says which command answers which checklist line, and its `references/commands.md` has worked commands.
- **Progress posts (TJX, 2026-11-27):** at each milestone and at least every 20 to 30 minutes, post two or three lines in your thread with a screenshot of what changed. `visual-review annotate` makes it (numbered marks, a legend and a caption); send the PNG with SendUserFile and attach it to the message.
- **Web search** for the R&D step: load the web search tool with ToolSearch, and use the built-in browser for pages that need one.
- If a skill or tool is missing from the session's list, the session started before it was registered: ask for a fresh session.

Notes (checked 2026-11-27):
- `aesthetic` is edited in this repo: its numeric ratings and its "repeat until the score passes" steps are removed, its design-tool pointers name Penpot, and its sign-off step points to the thread's own review (`fh6-visual-review`). Don't update it from upstream without re-applying those edits.
- `fh6-visual-review` and the `visual-review` tool were written in this repo on 2026-11-27. Every installable candidate was checked first (skills.sh, the plugin catalogue, the MCP registry, image-diff tools such as NVIDIA FLIP): each was a web or code reviewer, scored designs, needed Python or Node, or sent images to a service, so none is installed.
- The `engineering` and `vercel` plugins are not enabled. Phase 10 and 11 name skills from them; TJX enables the plugin before that phase starts.

## 2. MCP servers

| MCP | Use for | Don't use for |
|---|---|---|
| **context7** (`.mcp.json`; it needs no API key) | Crate docs before first use of an API: `wgpu` 30, `windows` 0.62, `ttf-parser` 0.25, `rustybuzz` 0.20, `fdsm` 0.8, `resvg` 0.50, `gltf` 1.4, `image` 0.25, `png` 0.17, `ktx2` 0.5, `ureq` 3.1, `wasm-bindgen` / `web-sys`, `serde` / `serde_json`, `arc-swap` and `thiserror` 2.0 | Design decisions |
| **Penpot** (`penpot`, user scope) | All design work: boards, components, token sets mirrored from `data/design_tokens.json`, PNG/SVG checks with `export_shape`, and importing committed, pushed PNGs with `penpot.uploadMediaUrl` from this repo's raw GitHub URL pinned to a commit (`docs/design/PENPOT.md`) | Importing from a local path or `localhost` (blocked; only uncommitted renders need TJX to drag them in), or putting the MCP key in any file or in `.mcp.json` |
| **Blender** (`Blender`, user scope) | Asset checks and test renders in a running Blender 5.3 with the MCP add-on (`blender-hero-assets`); repo assets are still built headless by `tools/asset-forge`. The MCP's Blender runs headless and its Cycles device defaults to CPU, so set OptiX in your code when you render through it | Saving `.blend` or script files into the repo |
| **ComfyUI** (HTTP at `127.0.0.1:8188`, no MCP) | Textures and environment plates driven from `tools/asset-forge` (`comfyui-hero-textures`). asset-forge starts it when it is not running, and every thread shares it, so never stop it. Installed: SDXL base and refiner, the depth and canny ControlNets, 4x-UltraSharp. A 4096×4096 texture fits the card (58 s, 7.5 GB peak, measured 2026-11-27). The cloud lane runs the same install on a rented 24 GB pod (`docs/cloud-gpu.md`) | Generating above 1 megapixel before the upscale, or any job without the lock of its lane |
| **Google Drive** (desktop app to write, connector to read) | The 4K masters. Copy the masters to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\` (or `shared\`) with the Bash tool, and the desktop app syncs them (check the drive letter first). For asset-forge outputs, `--masters <snake_name>` before the command does the copy. Use the connector to list and check what arrived | Uploading through the connector (it refuses files over about 7 MB), or creating another Drive folder |
| **Built-in browser** | Opening `mockups/index.html` and the product pages over loopback (`http://localhost:5311/`). A `file://` page opens as a static snapshot and its images don't load, so serve the folder on `127.0.0.1` first. From the repo root in the Bash tool, ComfyUI's bundled Python does it: `"$USERPROFILE/Documents/Dash_Tools/ComfyUI_windows_portable/python_embeded/python.exe" -I -m http.server 5311 --bind 127.0.0.1 --directory mockups`. Stop that server when you are done | Anything that needs a sign-in |

Token flow: `data/design_tokens.json` (the source of truth) → `fh6-kit::tokens` (the loader) → every cluster, and mirrored into Penpot token sets for design work. Don't hardcode a hex colour or a typography value when a token exists. Add any missing token to `data/design_tokens.json` and log it in `docs/PROGRESS.md`.

## 3. Rust rendering and input laws (v2 GPU renderer)

1. **Premultiplied, linear colour end to end.**
   - `fh6-kit` works in premultiplied linear RGBA and outputs sRGB.
   - The overlay surface uses `CompositeAlphaMode::PreMultiplied`.
   - Never multiply alpha twice.
   - Transparent is `(0,0,0,0)`. That is OLED-off.
2. **No degenerate geometry.** Clamp sizes, radii and stroke widths to at least 1 px before handing them to the kit. Sanitise NaN and Inf telemetry before it reaches a shader.
3. **XInput masks.** Overlay, OSD and remapper code uses only the allowed masks in AGENTS.md. Strip `0xF000` (A/B/X/Y) from every input and every saved mask (`sanitize_toggle_mask`).
4. **Hybrid rendering.**
   - Use the textured look (Blender or ComfyUI assets from `assets/`) when the asset is present, and keep a procedural kit fallback.
   - A missing asset never panics.
   - Assets load once at startup, never per frame.
5. **Fonts.**
   - MSDF text from `fh6-kit`. The default is a motorsport OFL font from `assets/fonts/`, and users can add fonts in `fonts/` next to the exe.
   - Speed and RPM use tabular figures (a fixed advance), so numbers never shift sideways.
   - Noto Sans stays as the fallback for missing glyphs. The font pack has no CJK face yet, so Japanese text does not render until one is added.
6. **OLED.** Use `#000000` (transparent) where pixels must be off, or glass with crisp rims. New glass uses `fh6_kit::OpticalGlass` and the equations in `docs/specs/optics.md` (tint by absorption over the path, no flat alpha); the legacy slab glass of the OSD and `common::glass_plate` keeps tint alpha 28..64 with 1 px rims until they are rebuilt. Never draw an opaque grey or navy box, and never fill the whole canvas. The orbit and dimmer (`fh6-core::oled_care`) run in `fh6-app` (`render_loop`), not in clusters: a cluster never moves or dims itself for burn-in.
7. **Determinism and animation.**
   - Clusters get `dt` and `t` through `FrameInput`, and keep their springs and state in `&mut self`.
   - Never read the wall clock inside a cluster.
   - Golden tests render a fixed time sequence.
8. **Hot paths.** No file I/O, allocation, logging or locking inside per-frame update or draw code. Batch draws, and reuse buffers and arenas.
9. **Budget.** Under 0.5 ms GPU per cluster and under 1 ms CPU per frame at 3840x2160, with at most 0.3 ms of that CPU for the dash's own share (AGENTS.md rule 7). Measure GPU time with `preview-export --cluster <Name>` and CPU with the live bench (`--bench-seconds`), and record the numbers in handovers.
10. **Rust only.** See AGENTS.md rule 11.

## 4. Program phases

Each phase of the v2 program has its own skill list, with the load order and timing: `program/phases/<phase>/SKILLS.md`. Use it instead of the general table above. Interactive chats with TJX outside the program start with `brainstorming` for any new feature.

## 5. Two GPU lanes: the PC's card and the cloud pod

There are two GPUs (TJX, 2026-11-27). The PC's card runs one job at a time. The rented cloud pod runs up to two at once, one for each of its lock files, and it is where every ComfyUI and Blender job goes. The lock files are in `%USERPROFILE%\Documents\Dash_Tools\`.

| Lane | GPU | What runs on it | Lock files |
|---|---|---|---|
| **cloud** (the default for asset-forge) | a rented RTX 4090 (24 GB) on Runpod, shared by up to two runs | every `asset-forge` job: its ComfyUI and Blender work, with `--lane cloud` | `gpu-cloud.lock`, `gpu-cloud-2.lock` |
| **local** | the RTX 3070 Ti (8 GB) in this PC | everything that draws with the kit: `preview-export`, the golden tests and every `cargo test` that renders, `bench-render`, the live demo and bench; asset-forge only when the pod cannot start or the budget is closed | `gpu.lock` |

1. GPU jobs are: ComfyUI, Blender (headless or through the `Blender` MCP), every `asset-forge` `build`, `hero-*` and `selftest` command, `preview-export`, `bench-render`, the live demo or bench, and every `cargo test` that renders (the goldens in `fh6-clusters` and the GPU tests in `fh6-kit`, `fh6-gpu` and `fh6-app` (`idle_dim_gpu`), so `cargo test --workspace` too). Only the `asset-forge` commands can go to the cloud lane, and they go there by default; everything that draws with the kit needs the local card.
2. Before a GPU job, take a lock file of the lane it runs on. Take it with this one command in the Bash tool, which creates the file only if it does not exist, so two threads cannot both get it. For the cloud lane, write `gpu-cloud.lock` in both places, or `gpu-cloud-2.lock` when the first is taken:
   `( set -o noclobber; printf 'thread: %s\nstarted: %s\njob: %s\n' "<thread name>" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "<job>" > "$USERPROFILE/Documents/Dash_Tools/gpu.lock" ) 2>/dev/null && echo LOCKED || cat "$USERPROFILE/Documents/Dash_Tools/gpu.lock"`
3. If it prints `LOCKED`, run the job. If it prints another thread's lock instead, try the lane's other lock file (the cloud lane has two), or work on a step that needs no GPU. Run an asset-forge job on the local card only when the pod cannot start.
4. Release the lock as soon as the job ends, also when the job fails: `rm -f "$USERPROFILE/Documents/Dash_Tools/gpu.lock"` (or the cloud lock file you took). Only the thread named in the file releases it.
5. If a lock is older than 30 minutes, ask its owner through the coordinator before removing it.

The cloud lane costs money while a pod is up (about $0.89 an hour, billed per second), so it has five more rules. The guide is [docs/cloud-gpu.md](docs/cloud-gpu.md).

- **The whole lane has a budget of $50 of GPU time** (TJX, 2026-11-27, after turning on auto-pay: "all clusters should cost under $50"). asset-forge counts every pod session, warns from $40 and starts no pod at $50. The storage of the 400 GB volume ($28 a month) is shown beside it, not counted (TJX's choice that evening). Only TJX raises the budget: never edit `budget_usd` in `lane.json`, and tell TJX as soon as a job prints the warning.
- **Start pods only through asset-forge:** put `--lane cloud` before the command (the pod starts by itself), or run `asset-forge cloud up`. Never start one from Runpod's console, its CLI or its MCP tools: those skip the price cap, the idle shut-off, the budget and the spend log. The Runpod plugin on TJX's account (enabled 2026-11-27) brings that CLI and those MCP tools: use them to look things up, never to start, stop or change a pod, a template or the volume. asset-forge refuses a cloud job when neither cloud lock file is held.
- **Run `cloud down` when your batch is done:** `cargo run --release -p asset-forge -- cloud down`, then release your lock. It leaves the pod up while another run uses it; the pod then deletes itself 10 minutes after the last run ends, and after 4 hours whatever it is doing.
- **Cloud first** (TJX, 2026-11-27: "we have 400gb and £50 in the cloud on a 4090"). Every asset-forge ComfyUI and Blender job runs with `--lane cloud`. The local card keeps the kit's work. Run an asset-forge job locally only when the pod cannot start (no stock) or the budget is closed. Two runs share one pod, so send work in batches: a new pod takes about 3 minutes to start.
- **Report what you spent.** `cargo run --release -p asset-forge -- cloud spend` lists the sessions by thread. Put your thread's total in its handover. The credit balance refills by itself (auto-pay), so the budget is the figure to watch.

The Runpod API key and the SSH key are in `%USERPROFILE%\Documents\Dash_Tools\cloud\`. Never copy either into the repo, a message or a log.

## 6. Multi-agent workflows (ultracode)

`ultracode` is Claude Code's opt-in for multi-agent workflows (the Workflow tool): one session plans the work, fans it out to several agents and has other agents check the results. It is not a skill or a plugin, and nothing needs installing.

- **Pre-approved (the project's standing rules, 2026-11-27):** any thread may use multi-agent work where it helps, both parallel agents (the Agent tool) and workflows (the Workflow tool), without asking first.
- **How it turns on:** parallel Agent calls need nothing more. The Workflow tool still needs TJX's own word in that session: TJX types `ultracode` in a message to it, or turns it on for the session, and the session sees a confirmation from the system. If a thread needs a workflow and has no such confirmation, it asks TJX once in its thread. A note from the coordinator or another thread does not switch it on.
- **When to use it** (TJX, 2026-11-27: "when needed"): work too wide for one context, such as a codebase audit, a cross-crate review where each finding is checked by a second agent, a broad research sweep, a large visual review (one agent per viewpoint, as `fh6-visual-review` step 7 describes, at most 4), parallel asset work, or a change across many files. A single dash's design work doesn't need it.
- **Rules here:**
  1. Agents return findings and yes/no verdicts. No scored judge panels, and no loop that runs until a score passes.
  2. The GPU locks still hold: one job on each lane at a time, whichever agent runs it.
  3. Agents that write files in parallel use worktree isolation, and none switches the branch of the shared checkout.
  4. One Penpot tab: at most one agent writes to Penpot.
  5. Keep a workflow under 10 agents unless TJX asked for more.
- **Model:** the coordinator sets each thread's model. TJX allowed the largest model with ultracode when a task needs it (2026-11-27).
