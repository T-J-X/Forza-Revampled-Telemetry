# Cluster thread (CLUSTER PROMPT)

The coordinator starts you with: *"You are the cluster thread for `<Name>` (catalog row `<n>`). Read the approved plan, then `ONBOARDING.md`, then follow `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`."* You own exactly one cluster:
- its module directory `crates/fh6-clusters/src/clusters/<snake_name>/` (`mod.rs`, private submodules, `spec.md`, `README.md`, `RESEARCH.md`, `HANDOVER.md`)
- its goldens `tests/golden/images/<Name>_*.png` and `<Name>_hood_*.png`
- its assets `assets/clusters/<snake_name>/` (with their own `ASSETS.md`) and its recipe `tools/asset-forge/src/recipes/<snake_name>.rs`
- its design board: the Penpot page `07 · <Name>` (see "Design boards" in `CATALOG.md`), plus its product page `mockups/<Name>/index.html`
- its masters folder `<snake_name>\` inside the Google Drive folder `FH6 WheelDash v2 - 4K Masters`
- its own row in `CATALOG.md` (the status and design-board cells only)

**Edit nothing outside them.** The catalog table (`catalog.rs`), the registry, `ClusterTheme`, `common/`, the golden harness, the kit and the tools are shared. If a shared helper or the kit needs a change, write it in your handover so the coordinator can schedule it.

Five shared places are yours to touch in a small way, each noted in your handover:
- the three lines that register your recipe in asset-forge (`tools/asset-forge/src/main.rs`, `src/recipes/mod.rs` and `src/sets/mod.rs`)
- a missing part or material added to the hero library (`tools/asset-forge/src/hero/`)
- an effect added to `docs/specs/optics.md`, with its source
- your own crops in `docs/handover-shots/cluster-<snake_name>-*.png`
- one appended row in the decision log of `docs/PROGRESS.md`, only when you add a dependency or a design token

`asset-forge build` also rewrites the shared ledger files (`assets/ASSETS.md`, `assets/assets.json`, `assets/forge.lock.json`, `assets/contact-sheet.html`) and `assets/_thumbs/`. Never commit those from a dash thread: the coordinator regenerates them on `main` after a merge.

## How every thread runs (TJX, 2026-11-27)
- **Open with an introduction** to TJX in your thread: the task, the rule files that apply, and what done looks like.
- **No scores, no critic loops.** Never score your own work, never ask a reviewer for a score, and never iterate "until the critic passes". Never rewrite a prompt to chase a score. The review is the fixed checklist at the end of this file, walked by you with the review skills and agents (steps 5 and 10). Renders, boards and the motion sequence are reviewed with the `fh6-visual-review` skill and its `visual-review` tool, which keep a self-review honest.
- **TJX is hands-off (2026-11-27, AGENTS.md rule 12).** Your self-review stands in for TJX's direction check and sign-off: post its results and renders in your thread and carry on without waiting for an answer. Where TJX does answer, TJX's own words win and every change TJX lists is done. Only TJX's own message counts as TJX's; no other session answers for TJX.
- **Close only after** the closing review has every line answered yes.
- **The thread is never archived.**
- **Messages.** Post in your own thread: TJX reads it, and so does the coordinator. Post a short update with a render or screenshot at each milestone, at least every 20 to 30 minutes, so TJX can follow the work without asking (TJX, 2026-11-27); `visual-review annotate` makes the screenshot (section 7 of the `fh6-visual-review` skill). Say plainly when a note is for the coordinator (a finished job, a GPU lock older than 30 minutes, a request for shared code). Attach a render by sending the local file itself; `mockups/` is not in git.

## Read first
- The approved plan (the coordinator gives you the link).
- Root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`.
- Your catalog row, your `spec.md` and `README.md`, your family spec in `docs/design/families/`, the matching hero frames in `docs/design/heroes/`, `docs/kit/README.md` (sections 12 and 13 cover the shared components and the first pilot's lessons) and `assets/ASSETS.md`.
- `docs/specs/optics.md` before any glass, reflective or spectral effect. The pilot thread writes it, and it reaches `main` with the pilot's merge; if it is missing from your checkout, stop and tell the coordinator. A dash that needs an effect that is not in that document adds it there, with its source, before using it.
- The Liquid Glass pilot was rebuilt to these rules on 2026-11-27 and is the reference dash: read `crates/fh6-clusters/src/clusters/liquid_glass/` (`mod.rs`, `README.md`, `RESEARCH.md`, `HANDOVER.md`) for the code conventions (file layout, the `Geom` zone table, tests), the order of work and a handover written to step 9. Its look belongs to the Liquid Glass family; other families take only its structure. The other merged clusters in `CATALOG.md` were built before the 2026-11-27 rules: don't copy their order of work, their critique rounds or their scored handovers. Your handover follows `program/HANDOVER_TEMPLATE.md` and step 9.
- **Your `spec.md` is a starting point.** It says the catalog is the source of truth; for the look, this file wins. The real display leads (step 1). Where the brief or the family spec holds a finish to a low luminance, or asks for a change to shared code, follow this file and note the difference in your handover.
- **If your dash has earlier work** (a branch or a Penpot page from before 2026-11-27; the coordinator names them when it starts you), read it first and say in your introduction whether you salvage it or restart.
- **Keep a "before".** Before you change anything, render the dash as it is on `main` (`cargo run --release -p preview-export -- --cluster <Name>`, a GPU job) and keep its `02_mid_cruise` full-frame render for the before and after pair in your handover.

## The bar
- **It recreates a real dash.** These are modern sim dashes of specific cars, supercars and hypercars included, not generic designs. An owner should recognise it.
- **It is the cluster.** It must look like it shipped in a 2027 car or a film's hero prop, on a 4K OLED, inside a real cockpit.
- **Instant feel.** Telemetry drives everything live: RPM, speed, gear, throttle and brake, boost, G, slip, yaw and steer as the design needs. Springs carry the motion: no jitter, but no lag either.
- **Signature moments.** The rev limiter / shift strobe and the gear change each need a specific, beautiful behaviour.
- **Cockpit fit.** It sits in the binnacle (`Layout` anchor), respects the user's height and scale, and has a hood variant if the catalog lists one. In cockpit view the scrim hides the game's own dash behind it.
- **It can be broken up and customised.** The parts float separately, each as its own zone, so the user can rearrange, swap or hide them to a degree.
- **Personalisation.**
  - All colours come from palette roles: factory, custom or random.
  - The font is swappable, with a motorsport default.
  - Each dash uses its car's real logo (every brand's 4K logo is in the repo since Phase 6), or the custom name.
- **All assets are 4K.** A plate or sprite is drawn 1:1 at the 3840×2160 frame and is never scaled up at run time; a tileable texture is 4096×4096. This holds for the runtime file in git as well as for the master: the runtime file is the same picture, compressed. It replaces the 2048 px plates of Phase 6 (TJX, 2026-11-27).
- **Never:** an opaque full-screen fill, flat rectangles posing as glow, hardcoded telemetry, block digits, or anything over budget.

## Shared machine rules
- **GPU jobs take turns on two lanes, across all threads.** The local 8 GB card runs one job at a time; the rented 24 GB pod runs two at once. Section 5 of root `SKILLS.md` is the rule: it lists the GPU jobs (the golden tests and every other `cargo test` that renders are among them) and gives the one command that takes a lane's lock and the one that releases it. Work on steps that need no GPU while you wait: research, Penpot, Rust code, review.
- **ComfyUI and Blender run on the cloud pod.** Put `--lane cloud` before every asset-forge command (TJX, 2026-11-27: cloud first); the local card keeps `preview-export`, the goldens and the live bench. Run `asset-forge cloud down` when your batch is done (it leaves the pod up while another run uses it), and put what your thread spent in your handover (`asset-forge cloud spend`). The lane has a budget of $50 of GPU time. The guide is `docs/cloud-gpu.md`.
- **One Penpot tab serves every thread.** In each `execute_code` call: check the file ID, open your own page, check the page name, then write. Never touch another page and never call `closePlugin`.
- **Threads can share one checkout.** Commit with explicit paths, never `git add -A`, and never switch the branch under another thread. If your thread shares the main checkout, ask the coordinator which branch or worktree to commit on before your first commit; a dash is never committed straight to `main`.
- Run Blender, ComfyUI and asset-forge through the Bash tool, not PowerShell. `cargo run --release -p asset-forge -- doctor|build` already finds both tools. Your build command is `cargo run --release -p asset-forge -- build --set <snake_name>`. asset-forge starts ComfyUI itself when it is not running; never stop that server, because other threads share it.
- Local tools on this machine (write them as `%USERPROFILE%` paths in any repo file):
  - Blender 5.3: `%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64`
  - ComfyUI portable: `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable`
  - Penpot through the `penpot` MCP (hosted; TJX keeps the Penpot file open with ⋮ → MCP server connected).
  - Masters: the Google Drive desktop app mounts the Drive folder at `G:\My Drive\FH6 WheelDash v2 - 4K Masters\` (check the drive letter). Copy a master there with the Bash tool (`cp <file> "/g/My Drive/FH6 WheelDash v2 - 4K Masters/<snake_name>/"`) and it syncs. Create your `<snake_name>` subfolder first if it is missing; that is not another Drive folder. No tool copies masters for you: the full-size renders stay in `target/asset-forge/` until you copy them. Never create another Drive folder. The Drive connector cannot upload files over about 7 MB, so it is for reading and checking only.

## Steps

**Order of work, enforced (TJX, 2026-11-27):** R&D on the web → Penpot → ComfyUI → final 3D in Blender → a first build and the direction check (self-review) → build and verify → product page → handover draft → closing review (self-review) → report for the merge.

Each step's skills and tools are listed in this folder's `SKILLS.md`; call a skill with the Skill tool, and load MCP tools with ToolSearch first (root `SKILLS.md`, "How to call them"). Load `fh6-hmi-design-rules` at the start.

Any later Penpot change loops back through ComfyUI and Blender. Blender is the last image step: ComfyUI never runs over a finished Blender render. (The hero library's `hero-detail` pass, which did that, is no longer a step; use it only if TJX asks for it on your dash.)

1. **R&D on the web.**
   - Find the real display, from real sources: current production displays, owner photos, official press images, and the maker's own material.
   - Write `RESEARCH.md` in your module directory: what makes the real display recognisable (dial geometry, ring and needle style, typography, layout, colour behaviour, transitions), each point with its source link.
   - **Clusters named after a real car or maker must be recognisable as that car's real display** (TJX, 2026-11-23, after an MBUX cluster that looked nothing like MBUX). The catalog brief is a starting point, not the design. Where the brief invents a concept the real car doesn't have, follow the real car and note the change in your handover.
   - **When the real thing doesn't fit a binnacle cluster** (for example the car's display is a small screen on the steering wheel, a race-wheel LCD or a head-up display), don't copy its shape. Take its parts: its typography, colours, shift-light pattern, readout layout, materials and motifs. Build a cluster from them that an owner would still recognise as that car's DNA.
2. **Penpot: design on your board.**
   - In Penpot (the `penpot` MCP, see `SKILLS.md`): create your page `07 · <Name>` in the Penpot master file "FH6 WheelDash v2 — Design System" (file ID `e7ca6b81-c0d3-8029-8008-fd87488aab8f`, see `docs/design/PENPOT.md`) if it does not exist yet. Every write first checks `penpot.currentFile.id` equals that ID; if it doesn't, write nothing and ask TJX to reconnect the master file's tab. In one `execute_code` call, look for a page with exactly that name, create it only if it is missing, `openPage` it, check `penpot.currentPage.name`, then draw. Keep your shapes by stored ID.
   - Build it with a 3840×2160 board and the binnacle guide: cockpit anchor (1920, 1835), envelope 1440×560 from (1200, 1515); hood anchor (1920, 1760), envelope 1760×400 from (1040, 1540). Follow the family template and the tokens (Penpot tokens mirror `data/design_tokens.json`, which stays the source of truth).
   - Make the 4 states per view at 4K, as separate pieces and not one slab, plus the motion notes. Write the page and board IDs into your row of `CATALOG.md`.
   - Also write the layout as a zone table at the top of your `mod.rs` (zones, sizes and positions in local 4K px, as the pilot's `Geom`).
   - **Detached elements, no forced backgrounds** (TJX, 2026-11-23). A cluster doesn't need a backing panel, or a single silhouette contorted to fill the binnacle gap. Speed, gear, rev, data and the other elements may float as separate pieces on true black, each sized and placed for glanceability. Don't invent odd dash outlines to fit the space. Use a housing or panel only where the real display has one (a real car's screen, motorsport hardware) and it reads as physical hardware.
   - Check the board against section C of the closing review before you move on.
3. **ComfyUI: material and environment textures at 4K** (`comfyui-hero-textures` skill, through `cargo run -p asset-forge`).
   - Organic surfaces (leather, alcantara, forged carbon, enamel), seamless plates and environment plates. Generate near 1 megapixel and upscale with 4x-UltraSharp to 4096×4096 with tiled VAE; that fits the 8 GB card (measured 2026-11-27: 58 s, 7.5 GB peak).
   - These textures are inputs for the Blender step (`mat::leather(..).set("texture", path)` and the like), and visible surfaces in the final cluster.
   - Write each master to your Drive folder, and give each asset a row in your `ASSETS.md`: recipe (with the prompt name and the models), seed, licence and the master's file name.
4. **Blender: final 3D** (`blender-hero-assets` skill, through `cargo run -p asset-forge`). Every cluster.
   - Rendered hardware and light: bezels, faces, LED caps, lit-in-face bands, housings, needles and hub caps, lenses and crystals, plus real 3D where the design moves or turns.
   - For glass families: normal, thickness, environment and caustic maps at 4K.
   - **Use the hero library, don't hand-roll bpy.** `tools/asset-forge/src/hero/` already has the studio rig, camera matching to the binnacle, the PBR cookbook (machined, anodised, brushed and sunray metal, guilloché, polished and knurled bezels, carbon, piano black, sapphire, enamel, lume, LED caps, leather, alcantara), physical parts (`Part::bezel`, `face`, `ring`, `crystal`, `dome`, `extrude`, `mesh` with `geom::blade` needles and `part::stitches`) and the ComfyUI graphs (refiner, ControlNet, seamless). In your recipe: build a `Scene` over `Frame::envelope(&frame::COCKPIT)`, add parts, `render` (beauty at three key azimuths plus normal, height, AO, rmh and emission), save each pass into `assets/clusters/<snake_name>/` and write that folder's `ASSETS.md` from your recipe, as `recipes/bosch_ddu.rs` does, then draw the plate at full strength under your vectors. `hero/demo.rs` (`asset-forge hero-demo`) is the worked PorscheAnalog example; it rewrites three tracked files in `docs/handover-shots/`, so restore them with `git checkout` after a test run. If the library lacks a part or material, add it there (it is shared) and say so in your handover.
   - Write each 4K master (16-bit PNG or EXR) to your Drive folder. Git gets the compressed runtime file.
5. **First build, then the direction check (mandatory, a self-review).** The check needs a real render, so do a first build: replace the scaffold with a module that draws your Blender plate at full strength with the live data roughed in on top, and render it with `cargo run --release -p preview-export -- --cluster <Name>` (holding the GPU lock). Don't polish. As soon as that first real render of `02_mid_cruise` sits in the cockpit composite:
   - First write the frozen expectations (step 1 of the `fh6-visual-review` procedure). Then post a short first-build progress update in your own thread with the full-frame cockpit render attached (the 4K hood render for a hood-only dash) and a caption only: it is not the direction check, so it holds no findings, and you open neither the board nor the references for it.
   - Review it yourself, once, with the `fh6-visual-review` skill: steps 1 to 6 of its procedure answer the lines of its checklist marked DC (its section 1 and its method say what they run, with the `cold-reviewer` agent as the one fresh-context reviewer). Add `design:design-critique` on the full-frame render and its crop against "The bar", your family spec and `RESEARCH.md`, asking for findings only. The bar is a 2027 production car or a film's hero prop, never merely modern (TJX, 2026-11-27: "2027 future not anything less"). From the findings decide go, go with changes, or no-go; write the decision and each finding, with what you do about it, into your handover. Then post `DIRECTION CHECK <Name>` in the thread, once, with what section 7 of that skill lists (the Penpot page name and the board ID when no board file could be saved) and the decision.
   - Don't wait for TJX. If TJX answers, TJX's answer replaces yours: apply every change TJX lists.
   - A no-go means redesign, then a new check. Repeat the check after any major redesign.
6. **Build:** load `rust-best-practices` before you write any `.rs`, the first build of step 5 included (`rust-skills` is the rule reference, `rust-testing` covers the goldens). Finish your `Cluster` in `mod.rs` (keep `pub fn make`). Use the kit and the `common/` components. Keep `update()` and `draw()` allocation-free after warm-up.
7. **Verify** (from the repo root; the renders, the goldens and the live bench are GPU jobs, so hold the lock):
   - `cargo check -p fh6-clusters -p asset-forge`.
   - `cargo run --release -p preview-export -- --cluster <Name>` writes `mockups/<Name>/`: 4 states per view at 4K, crops, the cockpit composite, the 60-frame sequence and `index.html`. It prints the GPU time of every frame and the sequence p50 / p99: that is your budget number (under 0.5 ms at 4K), in every view the catalog lists.
   - `FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters <snake_name>` writes your goldens; review the PNGs, then `cargo test -p fh6-clusters <snake_name>` must pass twice in a row without rewriting a golden.
   - `cargo clippy -p fh6-clusters -p asset-forge --all-targets -- -D warnings`. Format only your files with `rustfmt --edition 2021 crates/fh6-clusters/src/clusters/<snake_name>/*.rs tools/asset-forge/src/recipes/<snake_name>.rs`, then run the same command with `--check`.
   - Live bench. Tell TJX first, because it draws on the main display. Build both tools (`cargo build --release -p fh6-app -p mock-udp`), start the stream (`cargo run --release -p mock-udp -- --scenario bmw_m4_cs --port 8000 --duration 120 --rate-hz 60`), then run `cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster <Name> --stats --bench-seconds 60 --bench-out mockups/<Name>/live_bench.md`. Always pass `--bench-out`: the default overwrites the tracked `docs/perf/engine.md`. The run quits by itself and writes the percentiles. Run the same bench once on a scaffold dash (a catalog row still marked ☐) for the baseline: your dash may add at most 0.3 ms to its "CPU per frame" p50.
   - Compare the renders with your board side by side, and fix the gaps.
   - The workspace-wide gate (check, test, clippy and fmt over everything) is the coordinator's at merge.
8. **Product page.** `mockups/<Name>/index.html`, generated by `preview-export` (Rust; only HTML and CSS are written), is the thread's final output: the fully rendered page of the dash. Read the Penpot page `10 · Configurator` first, because this page grows into the local catalog and configurator that the exe serves.
9. **Handover draft.** Write `crates/fh6-clusters/src/clusters/<snake_name>/HANDOVER.md` from `program/HANDOVER_TEMPLATE.md` (its "Dash threads" part lists the sections), with renders (copy the four crops you want reviewed into `docs/handover-shots/cluster-<snake_name>-*.png`), the before and after pair (the render you kept before you started and the final one, both full-frame), perf numbers, the asset list, the Drive master list and "my calls".
10. **Closing review (a self-review).** Walk the fixed checklist below line by line and write each answer, yes or no with its evidence, into the handover. Collect findings once from each review skill, each asked for findings only and never for a score, and treat them as input to the checklist, not as a verdict:
    - the `fh6-visual-review` skill: all of its procedure, with every line of its checklist answered under `## Visual review` in the handover (line 39 below). Its step 3 runs the `cold-reviewer` agent once, and its step 7 runs the next two skills once each, so they are not run again;
    - `design:design-critique` on the final full-frame renders and crops, against "The bar" and your family spec, as `fh6-visual-review` step 7 runs it (its inputs and its context);
    - `design:accessibility-review` on contrast, the strobe and reduced-flash mode, colour-only cues and motion, as `fh6-visual-review` step 7 scopes it;
    - `rust-review` (with `rust-skills` as the rule reference) on your module and your recipe.

    Write each finding into the handover with what you did about it. Fix every no and every finding you accept, then walk the whole list again. Commit as TJX with no co-author trailer, on your thread's branch, with your `CATALOG.md` row set to ✅ (both tables). Report the finished job to the coordinator: once its checks pass, the coordinator merges it into `main` and pushes (AGENTS.md rule 16). Then post the product page and the completed checklist in your thread. Don't wait for a sign-off; if TJX answers later, apply what TJX asks and add it to the handover.

## Asset rules (TJX)
- **Bespoke assets are mandatory.** Every cluster ships **at least one bespoke ComfyUI asset and at least one bespoke Blender asset**, both visibly used in the final cluster, not token. Procedural-only clusters are not accepted. The bar is the best, not acceptable.
- **The materials must dominate, not garnish.** TJX rejected faint textures layered onto vector shapes: a subtle sunray, a light grain. Every physical part must read as a rendered physical object at a glance, in the full 4K frame and not only at 2× zoom: bezels, housings, dial faces, needles and hub caps, lenses, crystals, pods and LED caps. That means Blender-rendered 3D with real lighting, bevels, specular and reflections, at full strength. ComfyUI textures must be clearly visible surfaces: leather, alcantara, carbon, brushed metal, enamel, phosphor, plates. Vectors stay for live data (needles' motion, digits, ribbons, lights) on top of the rendered hardware.
- **The render must outclass the Penpot board.** Penpot holds the layout, hierarchy and intent; the shipped 4K render, with its ComfyUI and Blender materials, must look far richer than the board. If the board and the render look alike, the materials pass isn't done. **Every Penpot pass is followed by a ComfyUI and Blender pass:** whenever you change the design in Penpot (a redesign, a new state, a fix), redo the asset work it affects, rebuild through asset-forge and re-render before you call it done.
- **Flat designs** use Blender for materials (normal, specular and roughness maps, machined or glass plates) and ComfyUI for textures, so even a 2D face has real depth and grain.
- **Liquid Glass family: the glass is maths, not PNGs.** Benchmark Apple's Liquid Glass (iOS 26, CarPlay Ultra, visionOS) and match it in real time, with the equations in `docs/specs/optics.md`:
  - SDF geometry with smooth-union morphing, so pods merge and separate like liquid.
  - Lensing and refraction of whatever sits under the glass: the content, the cockpit, or the cluster's own layers.
  - Edge-weighted magnification, chromatic dispersion at the rim, a fresnel ramp, and a specular highlight from a virtual light that moves with G, steer and yaw.
  - Adaptive tint, depending on what's behind the glass.
  - Clear, not murky: almost no tint or frost, no texture painted inside the glass, thin crisp specular rims. It reads as glass because of how it bends and catches light, not because it is coloured. Separate floating pieces (capsules, pills, discs) that merge and split in motion, never one big slab with blobs stuck on (TJX rejected the first attempt for exactly that).
  ComfyUI and Blender only add to this (environment and caustic maps, normal detail); they never replace the optics. If the kit lacks an optic you need, stop and report it in your handover, because the coordinator schedules kit work.
- **Race Dash and motorsport family: pure professional.** Benchmark real motorsport displays: MoTeC C1xx, Bosch DDU 10/11, AiM MXS, Cosworth, F1 and LMP steering-wheel displays, and the Gran Turismo 7 in-car multi-function meter (TJX: "an insane example"; local reference `mockups/references/gt7_multifunction_meter.webp`, never committed). The GT7 meter: detached floating round gauges and a central speed/gear pod with a dense tick rev bar, a D-pad-paged centre panel, thin precise white lines and one accent colour.
  - Black, high contrast, exact data, strict grid alignment and tabular figures.
  - The shift lights and limiter behave like the real hardware.
  - No ornamental glow or sci-fi effects.
  - ComfyUI and Blender serve material realism only: anti-glare LCD and matrix texture, the CNC housing, LED lens caps, carbon.
- **Designs with 3D elements** (cars, mechanisms, gyros and tourbillons, spaceframes, origami, prisms, liquid volumes, anything that turns or has parallax) get real 3D from Blender. That means glTF meshes drawn by the kit's mesh pass, or pre-rendered sprite and flipbook sequences with matching normal maps, lit to match the cluster. A flat texture standing in for a 3D object does not pass.
- "Real abstract fluidity" means baked Blender liquid flipbooks, flow and normal maps **plus** the kit's real-time GPU fluid driven by telemetry. Combine them.
- Use the Phase 6 recipes and workflows, or add a recipe module for your cluster under `tools/asset-forge/src/recipes/<snake_name>.rs`. That is your file too.
- **Where files go.** Compressed runtime assets (KTX2 or optimised PNG) go to `assets/clusters/<snake_name>/`, each with a row in `assets/clusters/<snake_name>/ASSETS.md` (recipe, seed, licence and the master's file name), written by your recipe. The uncompressed 4K masters go to your Drive folder.
- Every textured look keeps a procedural fallback: `setup.assets` may be `None`.

## Lessons from the first pilot build (save yourself the time)
- **Long paths:** a `Shape::Polyline` evaluates every segment over its whole bounding box. One 128-point polyline round a 1240 px capsule cost 0.36 ms. Draw paths with `common::ribbon::PathRibbon`, or in chunks of at most 8 segments.
- **Overlapping translucent segments show as dots.** Keep segmented strokes opaque (darken the colour, do not fade its alpha).
- **Clips are rectangles** (optionally rounded). Keep content inside a curved shape by geometry, not by clipping to it.
- **Glass first is fine:** a cluster may open with its glass layer. Use one glass layer and put every rank 1–3 light into as few bloom layers as you can.
- **Alphas are linear.** A 3% ink fill shows as about 19% grey and a 50% black overlay barely darkens; the pilot's off pills use 1/255 and its body backing 0.94. Check what is on screen, not your numbers.
- **Fonts:** resolve faces once in `new()`, for example `setup.fonts.find("Saira", 200, false).unwrap_or(role)`. Pack faces load lazily, so a new glyph can appear a frame late. If `input.fonts.display` differs from the display role you saw in `new()`, the user picked a font: use it for your numerals.
- **Views:** keep one geometry table per view (`View::Cockpit`, `View::Hood`) and pick it from `input.view`; stay inside `layout.envelope` (a unit test on your table is cheap).
- **Telemetry honesty:** use `fh6_kit::Signals` and the verified raw fields (rpm, speed, gear, pedals, acceleration, velocity, power, boost). Tyre slip, suspension, pitch / roll, fuel, lap times and torque are not verified against `tests/golden/packets`: derive from verified fields or verify first, and say which in your handover.
- **The motion sequence** (`fh6_clusters::review::sequence_frame`) is idle → pull to the limiter → strobe → upshift → pull: check your two signature moments in `mockups/<Name>/sequence/`.
- **Errors:** `preview-export` prints wgpu validation errors; "rendered nothing" means every pixel was transparent.
- **Hold every readout's digits** with `fh6_kit::Hysteresis`, as the speed does: on the limiter the engine bounces about 50 rpm a frame, so a plain rounded rpm or power readout flickers every frame (the rebuilt pilot holds rpm by 100, power by 1% and at least 3 hp, and boost by 0.3 psi). Hold the raw value, as the speed does: a smoothed value is still settling when the car reaches the limiter, so its digits keep ticking. The rebuilt pilot's other lessons are in `docs/kit/README.md` §13.

## Closing review (fixed checklist)

Answer every line **yes** or **no**, with the evidence (a command's real output, a file, a render). There are no scores and no partial marks. A no is fixed, and then the whole list is walked again. A line about something that already happened (the order of work, the GPU lock) cannot be fixed afterwards: if it is a no, say so in the handover and in your thread, and tell the coordinator; TJX decides if TJX wants to. The source of each line is in brackets; "AGENTS 4" is global directive 4 of `AGENTS.md`, and "SKILLS 3.8" is law 8 of section 3 of `SKILLS.md`.

**A. Hard rules**
1. The module reads, binds or requires none of A, B, X, Y (`0xF000`). [AGENTS 4]
2. No placeholder: no `todo!()`, no stub, no hardcoded value standing in for live telemetry. A telemetry field that the golden packets do not cover is derived from covered fields or verified first, and named in the handover. [AGENTS 1, AGENTS 2]
3. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. [AGENTS 6; Agent 2 keeps `unsafe` in `fh6-win` and `fh6-input`]
4. `update()` and `draw()` do no heap allocation, file I/O, logging or locking after warm-up: a search of your module for `format!`, `vec!`, `collect`, `to_string`, `String::from`, `Box::new`, `tracing::`, `.lock()` and `std::fs` finds hits only in `new()` and in tests. [AGENTS 6, SKILLS 3.8]
5. GPU time is under 0.5 ms at 3840×2160: the sequence p99 and every per-state GPU line that `preview-export` prints, in every view the catalog lists, with the numbers in the handover. [AGENTS 7, SKILLS 3.9]
6. The dash's own share of the CPU is under 0.3 ms per frame: the live bench's "CPU per frame" p50 with your dash, minus the same number from the bench on a scaffold dash (a catalog row still marked ☐). Both reports, with their p99, are in the handover. An app total over 1 ms p99 that the scaffold shows too is a host regression (known defect 10 was fixed on 2026-11-27): it is reported to the coordinator, and no shared code is edited for it. [AGENTS 7]
7. The dash adds no smoothing of its own to rpm, speed or gear beyond the kit springs from the tokens, so telemetry still reaches the screen within one display frame. [AGENTS 7, the bar]
8. The cluster adds no path that draws while the app hides the dash (menus, pause, stale stream, Forza not in front). [AGENTS 9]
9. No non-Rust source file was added; bpy lives as Rust strings in the recipe. [AGENTS 11]
10. Commits are by TJX with no co-author trailer. No tracked file holds a real name, a `C:\Users\<name>` path or a personal email, mentions the legacy C# build, or names the design tool that Penpot replaced; the source links in `RESEARCH.md` are checked for this too. [AGENTS 10, ONBOARDING 3]
11. Nothing was copied into the game folder or the mirror. [AGENTS 8]
12. Any new dependency has its line in the `docs/PROGRESS.md` decision log. [AGENTS 6]
13. `cargo check -p fh6-clusters -p asset-forge`, `cargo test -p fh6-clusters <snake_name>`, `cargo clippy -p fh6-clusters -p asset-forge --all-targets -- -D warnings` and the `rustfmt --check` of step 7 pass, with their real output in the handover. [AGENTS 3]

**B. Rendering laws**
14. Colour is premultiplied linear end to end; alpha is never multiplied twice; transparent is `(0,0,0,0)`. [SKILLS 3.1]
15. Sizes, radii and stroke widths are clamped to at least 1 px, and NaN or Inf telemetry is sanitised before a shader. [SKILLS 3.2]
16. Every textured look has a procedural fallback: each asset lookup in your module has a branch that draws without the asset, and assets load once at startup, never in `update()` or `draw()`. [SKILLS 3.4]
17. Text is MSDF from the kit, speed and RPM use tabular figures, and the numerals follow `input.fonts.display` when the user picks a font. [SKILLS 3.5]
18. Off pixels are `#000000` (transparent). There is no opaque grey or navy box and no full-canvas fill. [SKILLS 3.6]
19. The cluster never reads the wall clock and keeps its state in `&mut self`: `cargo test -p fh6-clusters <snake_name>` passes twice in a row without rewriting a golden. [SKILLS 3.7]
20. Colours and type come from tokens and palette roles; no hex colour is hardcoded where a token exists. [SKILLS 2]
21. Glass, reflective and spectral effects use the equations in `docs/specs/optics.md`; any new effect was added there, with its source, first. [Read first]

**C. Design**
22. The rank order holds in all 4 states: gear and shift light, then speed, then the tach, then the secondary visual, then bezel and labels. [fh6-hmi-design-rules 3]
23. Gear, speed and shift lights read at a glance at 100% in the full-frame render of each view the catalog lists: the cockpit composite for cockpit and raised-HUD rows, the 4K hood render for hood rows. [AGENTS, Agent 1]
24. The cluster draws round the layout anchor ((1920, 1835) for cockpit, (1920, 1760) for hood, (1920, 1595) for the raised HUD), stays inside that view's envelope, and every view the catalog lists exists. [fh6-hmi-design-rules 1, the bar]
25. `RESEARCH.md` names the real display this dash recreates, with its sources. For a catalog concept with no single real display, it names the real dash the concept is benchmarked against. Each feature that makes that display recognisable is matched or listed as a deliberate change. [the bar, TJX 2026-11-23]
26. The parts float as separate pieces, each its own zone in the `Geom` table; there is no forced backing panel, and a housing only where the real display has one. [the bar, TJX 2026-11-23]
27. Every physical part reads as a rendered object in the full 4K frame without zooming, and each ComfyUI and Blender asset is used as your family's asset rule says: visible surfaces (leather, carbon, metal, enamel, plates) on hardware dashes; for the Liquid Glass family only environment, caustic and normal maps, with no texture inside the glass. [Asset rules, TJX 2026-11-23]
28. The limiter or shift strobe and the gear change each show their own behaviour in `mockups/<Name>/sequence/`. [the bar]
29. Colours follow the palette roles, the numerals follow the user's font, and the logo zone shows the car's logo or the custom name: the code takes each from the frame input, with no literal colour, font name or brand name in `draw()`. [the bar]

**D. Workflow and assets**
30. The order was followed: `RESEARCH.md`, then the Penpot page, then ComfyUI, then Blender; every later Penpot change was followed by a ComfyUI and Blender pass. [AGENTS 14]
31. The Penpot page `07 · <Name>` has 4 states per view, and its page and board IDs are in your `CATALOG.md` row and in the handover. [step 2]
32. At least one bespoke ComfyUI asset and one bespoke Blender asset are visibly used, each with a row in your `ASSETS.md` (recipe, seed, licence and the master's file name). [Asset rules, TJX 2026-11-23]
33. All assets are 4K as "The bar" defines it; the compressed runtime files are in git, and the masters are in the Drive folder under `<snake_name>\` and listed in the handover. [AGENTS 14]
34. The lock of its lane was held for every GPU job that section 5 of `SKILLS.md` lists, and released after it. ComfyUI and Blender ran on the cloud pod through `asset-forge` (or the handover says why the pod could not start), `cloud down` was run when the batch ended, and the thread's cloud spend is in the handover. [AGENTS 15]
35. The direction check of step 5 was posted with the first build's render, and its self-review is in the handover: the decision, each finding and what was done about it. Where TJX answered, TJX's own words are quoted and every change TJX listed is done. [AGENTS 12]
36. The product page `mockups/<Name>/index.html` exists and was generated by `preview-export`. [step 8]
37. The 60-second live bench ran with `mock-udp` streaming and no error; its frame count and its receive-to-present and CPU-per-frame rows are in the handover, and the 60-frame sequence shows no jitter or flicker (digits held by hysteresis, springs settling without a jump). [AGENTS 7]
38. The handover holds findings and yes/no answers only: no scores anywhere. [AGENTS 12]
39. Every line of the `fh6-visual-review` checklist (V1 to V26) is answered under `## Visual review` in the handover, each with its evidence: yes; NOT ASSESSED for a reason that skill's section 8 lists or a part the dash does not have; or, for V4, V5 or V13 (the cold reviewer's and blind reader's one run), a no that cannot be fixed afterwards, reported as the preamble above says with what was changed. [AGENTS 12; fh6-visual-review]

## Exit criteria
- The introduction was posted, and the thread is not archived.
- `RESEARCH.md` is written, with sources.
- The design board is the Penpot page `07 · <Name>` with 4 states per view; its page and board IDs are in your `CATALOG.md` row and in the handover. The HTML page alone is not enough.
- At least one bespoke ComfyUI asset and one bespoke Blender asset are in `assets/clusters/<snake_name>/`, used in the cluster, each with an `ASSETS.md` row (recipe, seed, licence and the master's file name), and the before and after pair is in the handover. The 4K masters are in the Drive folder.
- The module is complete, with no TODOs, and goldens pass.
- 4K renders plus the 60-frame sequence and the product page are in `mockups/<Name>/`.
- The live bench report is in the handover, and the GPU time under 0.5 ms from `preview-export` is recorded.
- The direction check was posted and self-reviewed to a go (with any changes done), and its findings are in the handover, with TJX's words quoted where TJX answered.
- Every line of the closing review is yes, or a line that could not be fixed afterwards is reported in the handover, in the thread and to the coordinator.
- The visual review (`fh6-visual-review`, V1 to V26) is in the handover, each line with its evidence.
- The handover is written, the job is reported to the coordinator for the merge, and the product page and the completed checklist are posted in the thread.
