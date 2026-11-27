# Cluster thread (CLUSTER PROMPT)

The coordinator starts you with: *"You are the cluster thread for `<Name>` (catalog row `<n>`). Read the approved plan, then `ONBOARDING.md`, then follow `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`."* You own exactly one cluster:
- its module directory `crates/fh6-clusters/src/clusters/<snake_name>/` (`mod.rs`, private submodules, `spec.md`, `README.md`, `RESEARCH.md`, `HANDOVER.md`)
- its goldens `tests/golden/images/<Name>_*.png` and `<Name>_hood_*.png`
- its assets `assets/clusters/<snake_name>/` and its recipe `tools/asset-forge/src/recipes/<snake_name>.rs`
- its design board: the Penpot page `07 · <Name>` (see "Design boards" in `CATALOG.md`), plus its product page `mockups/<Name>/index.html`
- its masters folder `<snake_name>\` inside the Google Drive folder `FH6 WheelDash v2 - 4K Masters`
- its own row in `CATALOG.md` (the status cell only)

**Edit nothing outside them.** The catalog table (`catalog.rs`), the registry, `ClusterTheme`, `common/`, the golden harness, the kit and the tools are shared. If a shared helper or the kit needs a change, write it in your handover so the coordinator can schedule it.

## How every thread runs (TJX, 2026-11-27)
- **Open with an introduction** to TJX in your thread: the task, the rule files that apply, and what done looks like.
- **No scores, no critic loops.** Never score your own work, never ask a reviewer for a score, and never iterate "until the critic passes". Never rewrite a prompt to chase a score. The review is the fixed checklist at the end of this file plus TJX's judgement.
- **Close only after** the closing review has every line answered yes, and TJX has signed off in the thread. Only TJX's own message counts; no other session answers for TJX.
- **The thread is never archived.**

## Read first
- The approved plan (the coordinator gives you the link).
- Root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`.
- Your catalog row, your `spec.md` and `README.md`, your family spec in `docs/design/families/`, the matching hero frames in `docs/design/heroes/`, `docs/kit/README.md` (sections 12 and 13 cover the shared components and the first pilot's lessons) and `assets/ASSETS.md`.
- `docs/specs/optics.md` (written by the pilot thread) before any glass, reflective or spectral effect. A cluster that needs an effect that is not in that document adds it there, with its source, before using it.
- The Liquid Glass pilot is being rebuilt. Until TJX signs the rebuild off, read `crates/fh6-clusters/src/clusters/liquid_glass/` for module conventions only (file layout, the `Geom` zone table, tests, the handover shape), not for its look: TJX rejected the first version. Also read one finished cluster from `CATALOG.md` (status ✅) to match conventions.

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
- **All assets are 4K.** Full-frame plates are pixel-exact at the 3840×2160 frame; tileable textures are 4096×4096.
- **Never:** an opaque full-screen fill, flat rectangles posing as glow, hardcoded telemetry, block digits, or anything over budget.

## Shared machine rules
- **One GPU job at a time, across all threads** (one 8 GB card). Before any GPU job (ComfyUI, Blender, `asset-forge`, `preview-export`, the live demo), create `%USERPROFILE%\Documents\Dash_Tools\gpu.lock` holding your thread name, the start time (UTC) and the job. Delete it as soon as the job ends. If the file exists, wait. If it is older than 30 minutes, ask its owner (through the coordinator) before removing it. Work on steps that need no GPU while you wait: research, Penpot, Rust code, review.
- **A cloud GPU is not used** unless TJX has approved the provider and the cost.
- **One Penpot tab serves every thread.** In each `execute_code` call: check the file ID, open your own page, check the page name, then write. Never touch another page and never call `closePlugin`.
- **Threads can share one checkout.** Commit with explicit paths, never `git add -A`, and never switch the branch under another thread.
- Run Blender, ComfyUI and asset-forge through the Bash tool, not PowerShell. `cargo run --release -p asset-forge -- doctor|build` already finds both tools.
- Local tools on this machine (write them as `%USERPROFILE%` paths in any repo file):
  - Blender 5.3: `%USERPROFILE%\Documents\Dash_Tools\blender-5.3.0-windows-x64`
  - ComfyUI portable: `%USERPROFILE%\Documents\Dash_Tools\ComfyUI_windows_portable`
  - Penpot through the `penpot` MCP (hosted; TJX keeps the Penpot file open with ⋮ → MCP server connected).
  - Masters: the Google Drive desktop app mounts the Drive folder at `G:\My Drive\FH6 WheelDash v2 - 4K Masters\` (check the drive letter). Write a file there and it syncs. Never create another Drive folder. The Drive connector cannot upload files over about 7 MB, so it is for reading and checking only.

## Steps

**Order of work, enforced (TJX, 2026-11-27):** R&D on the web → Penpot → ComfyUI → final 3D in Blender → direction check with TJX → build and verify → product page → closing review → TJX's sign-off.

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
   - Write each master to your Drive folder, and record the prompt name, seed and models in your `ASSETS.md` row.
4. **Blender: final 3D** (`blender-hero-assets` skill, through `cargo run -p asset-forge`). Every cluster.
   - Rendered hardware and light: bezels, faces, LED caps, lit-in-face bands, housings, needles and hub caps, lenses and crystals, plus real 3D where the design moves or turns.
   - For glass families: normal, thickness, environment and caustic maps at 4K.
   - **Use the hero library, don't hand-roll bpy.** `tools/asset-forge/src/hero/` already has the studio rig, camera matching to the binnacle, the PBR cookbook (machined, anodised, brushed and sunray metal, guilloché, polished and knurled bezels, carbon, piano black, sapphire, enamel, lume, LED caps, leather, alcantara), physical parts (`Part::bezel`, `face`, `ring`, `crystal`, `dome`, `extrude`, `mesh` with `geom::blade` needles and `part::stitches`) and the ComfyUI graphs (refiner, ControlNet, seamless). In your recipe: build a `Scene` over `Frame::envelope(&frame::COCKPIT)`, add parts, `render` (beauty at three key azimuths plus normal, height, AO, rmh and emission), save each pass with `ctx.image`, draw the plate at full strength under your vectors. `hero/demo.rs` (`asset-forge hero-demo`) is the worked PorscheAnalog example; it rewrites three tracked files in `docs/handover-shots/`, so restore them with `git checkout` after a test run. If the library lacks a part or material, add it there (it is shared) and say so in your handover.
   - Write each 4K master (16-bit PNG or EXR) to your Drive folder. Git gets the compressed runtime file.
5. **Direction check with TJX (mandatory).** As soon as the first real render of `02_mid_cruise` sits in the cockpit composite, and **before** any polish, post one message to TJX in your own thread: `DIRECTION CHECK <Name>`, the full-frame cockpit render and the Penpot board export attached, then two or three lines on the idea and the references it follows. TJX answers go, go with changes, or no-go; apply every change TJX lists. Until TJX answers, carry on only with work that the answer can't invalidate (module plumbing, tests, telemetry). A no-go means redesign, then a new check. Repeat the check after any major redesign.
6. **Build:** replace the scaffold in your `mod.rs` with your `Cluster` (keep `pub fn make`). Use the kit and the `common/` components. Keep `update()` and `draw()` allocation-free after warm-up.
7. **Verify** (from the repo root, holding the GPU lock for the GPU jobs):
   - `cargo run --release -p preview-export -- --cluster <Name>` writes `mockups/<Name>/`: 4 states per view at 4K, crops, the cockpit composite, the 60-frame sequence and `index.html`. It prints the GPU time of every frame and the sequence p50 / p99: that is your budget number (under 0.5 ms at 4K).
   - `FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters <snake_name>` writes your goldens; review the PNGs, then `cargo test -p fh6-clusters <snake_name>` must pass.
   - `cargo clippy -p fh6-clusters --all-targets -- -D warnings`, and format only your files: `rustfmt --edition 2021 crates/fh6-clusters/src/clusters/<snake_name>/*.rs`.
   - Live: `cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster <Name> --stats` for 60 seconds; watch for jitter and read the CPU and latency lines.
   - Compare the renders with your board side by side, and fix the gaps.
8. **Product page.** `mockups/<Name>/index.html`, generated by `preview-export` (Rust; only HTML and CSS are written), is the thread's final output: the fully rendered page of the dash. Read the Penpot page `10 · Configurator` first, because this page grows into the local catalog and configurator that the exe serves.
9. **Closing review.** Walk the fixed checklist below line by line and answer each line yes or no with its evidence. Fix every no, then walk the whole list again. You may run `design:design-critique` or the `ui-ux-designer` agent once to collect findings; ask for findings only, never a score, and treat them as input to the checklist, not as a verdict.
10. **Handover and sign-off.** Write `crates/fh6-clusters/src/clusters/<snake_name>/HANDOVER.md` from `program/HANDOVER_TEMPLATE.md`, with renders (copy the four crops you want reviewed into `docs/handover-shots/cluster-<snake_name>-*.png`), perf numbers, the asset list, the Drive master list, the completed checklist and "my calls". Set your `CATALOG.md` row to ✅ in your branch (both tables). Commit as TJX with no co-author trailer. Report the finished job to the coordinator: once its checks pass, the coordinator merges it into `main` and pushes (TJX's standing rule, 2026-11-27). Then post the product page and the completed checklist to TJX and ask for the sign-off; the thread closes only on it.

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
- **Where files go.** Compressed runtime assets (KTX2 or optimised PNG) go to `assets/clusters/<snake_name>/`, with a row in `assets/clusters/<snake_name>/ASSETS.md` (recipe, seed, licence, and the name of the master). The uncompressed 4K masters go to your Drive folder.
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

## Closing review (fixed checklist)

Answer every line **yes** or **no**, with the evidence (a command's real output, a file, a render). There are no scores and no partial marks. A no is fixed, and then the whole list is walked again. The source of each line is in brackets.

**A. Hard rules**
1. The module reads, binds or requires none of A, B, X, Y (`0xF000`). [AGENTS 4]
2. No placeholder: no `todo!()`, no stub, no hardcoded value standing in for live telemetry; unverified telemetry fields are derived or verified, and named in the handover. [AGENTS 2]
3. No `unwrap()`, `expect()` or `panic!` in new non-test code, and no new `unsafe`. [AGENTS 6]
4. `update()` and `draw()` do no heap allocation, file I/O, logging or locking after warm-up. [AGENTS 6, SKILLS 3.8]
5. GPU time is under 0.5 ms at 3840×2160 (sequence p99) and CPU under 1 ms per frame, with both numbers in the handover. [AGENTS 7, SKILLS 3.9]
6. The cluster adds no path that draws while the app hides the dash (menus, pause, stale stream, Forza not in front). [AGENTS 9]
7. No non-Rust source file was added; bpy lives as Rust strings in the recipe. [AGENTS 11]
8. Commits are by TJX with no co-author trailer; no real name, `C:\Users\<name>` path or personal email is in any file. [AGENTS 10]
9. Nothing was copied into the game folder or the mirror. [AGENTS 8]
10. Any new dependency has its line in the `docs/PROGRESS.md` decision log. [AGENTS 6]
11. `cargo check`, `cargo test -p fh6-clusters <snake_name>`, `cargo clippy -D warnings` and the format check pass, with their real output in the handover. [AGENTS 3]

**B. Rendering laws**
12. Colour is premultiplied linear end to end; alpha is never multiplied twice; transparent is `(0,0,0,0)`. [SKILLS 3.1]
13. Sizes, radii and stroke widths are clamped to at least 1 px, and NaN or Inf telemetry is sanitised before a shader. [SKILLS 3.2]
14. Every textured look has a procedural fallback: the cluster renders with `setup.assets = None`, and assets load once at startup. [SKILLS 3.4]
15. Text is MSDF from the kit, speed and RPM use tabular figures, and a user font is honoured. [SKILLS 3.5]
16. Off pixels are `#000000` (transparent). There is no opaque grey or navy box and no full-canvas fill. [SKILLS 3.6]
17. The cluster never reads the wall clock; its state lives in `&mut self`; the goldens render the fixed sequence. [SKILLS 3.7]
18. Colours and type come from tokens and palette roles; no hex colour is hardcoded where a token exists. [SKILLS 2]
19. Glass, reflective and spectral effects use the equations in `docs/specs/optics.md`; any new effect was added there, with its source, first. [plan]

**C. Design**
20. The rank order holds in all 4 states: gear and shift light, then speed, then the tach, then the secondary visual, then bezel and labels. [fh6-hmi-design-rules 3]
21. Gear, speed and shift lights read at a glance in the full-frame cockpit composite at 100%. [AGENTS, Agent 1]
22. The anchors are (1920, 1835) for cockpit and (1920, 1760) for hood, everything stays inside `layout.envelope`, and the hood variant exists if the catalog lists one. [fh6-hmi-design-rules 1]
23. `RESEARCH.md` names the real display and its sources, and each feature that makes it recognisable is matched or listed as a deliberate change. [plan, TJX 2026-11-23]
24. The parts float as separate pieces, each its own zone in the `Geom` table; there is no forced backing panel, and a housing only where the real display has one. [plan, TJX 2026-11-23]
25. Every physical part reads as a rendered object in the full 4K frame without zooming, and the ComfyUI textures are clearly visible surfaces. [TJX 2026-11-23]
26. The limiter or shift strobe and the gear change each show their own behaviour in `mockups/<Name>/sequence/`. [the bar]
27. Colours follow the palette roles (factory, custom, random), the font swaps, and the car's logo or the custom name shows where the design has a logo zone. [the bar]

**D. Workflow and assets**
28. The order was followed: `RESEARCH.md`, then the Penpot page, then ComfyUI, then Blender; every later Penpot change was followed by a ComfyUI and Blender pass. [plan]
29. The Penpot page `07 · <Name>` has 4 states per view, and its page ID is in the handover. [plan]
30. At least one bespoke ComfyUI asset and one bespoke Blender asset are visibly used, each with an `ASSETS.md` row (recipe, seed, licence). [TJX 2026-11-23]
31. All assets are 4K; the compressed runtime files are in git, and the masters are in the Drive folder under `<snake_name>\` and listed in the handover. [plan]
32. The GPU lock was held for every GPU job and released after it. [plan]
33. TJX's own answer to the direction check (go, or go with changes) is quoted in the handover, and every change TJX listed is done. [plan]
34. The product page `mockups/<Name>/index.html` exists and was generated by `preview-export`. [plan]
35. The 60-second live demo ran with no jitter seen, and its CPU and latency lines are in the handover. [AGENTS 7]
36. The handover holds findings and yes/no answers only: no scores anywhere. [TJX 2026-11-27]

## Exit criteria
- The introduction was posted, and the thread is not archived.
- `RESEARCH.md` is written, with sources.
- The design board is the Penpot page `07 · <Name>` with 4 states per view (its page ID is in the handover); the HTML page alone is not enough.
- At least one bespoke ComfyUI asset and one bespoke Blender asset are in `assets/clusters/<snake_name>/`, used in the cluster, with ASSETS rows (recipe plus seed) and before and after crops in the handover. The 4K masters are in the Drive folder.
- The module is complete, with no TODOs, and goldens pass.
- 4K renders plus the 60-frame sequence and the product page are in `mockups/<Name>/`.
- Live demo verified, with the GPU time under 0.5 ms recorded.
- The direction check got a go from TJX, quoted in the handover, with any listed changes done.
- Every line of the closing review is yes.
- The handover is written, and TJX has signed off in the thread.
