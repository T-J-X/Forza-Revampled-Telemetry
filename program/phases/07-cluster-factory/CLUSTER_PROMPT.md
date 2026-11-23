# Cluster chat (CLUSTER PROMPT)

The conductor starts you with: *"You are the cluster chat for `<Name>` (catalog row `<n>`). Read `ONBOARDING.md`, then follow `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`."* You own exactly one cluster:
- its module directory `crates/fh6-clusters/src/clusters/<snake_name>/` (`mod.rs`, private submodules, `spec.md`, `README.md`, `HANDOVER.md`)
- its goldens `tests/golden/images/<Name>_*.png` and `<Name>_hood_*.png`
- its assets `assets/clusters/<snake_name>/` and its recipe `tools/asset-forge/src/recipes/<snake_name>.rs`
- its design board (a Figma Section if the Figma MCP answers, otherwise the HTML board; see "Design boards" in `CATALOG.md`)
- its own row in `CATALOG.md` (the status cell only)

**Edit nothing outside them.** The catalog table (`catalog.rs`), the registry, `ClusterTheme`, `common/`, the golden harness, the kit and the tools are shared. If a shared helper or the kit needs a change, write it in your handover so the conductor can schedule it.

## Read first
- Root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`.
- Your catalog row, your `spec.md` and `README.md`, your family spec in `docs/design/families/`, the matching hero frames in `docs/design/heroes/`, `docs/kit/README.md` (sections 12 and 13 cover the shared components and the pilot's lessons) and `assets/ASSETS.md`.
- The pilot, `crates/fh6-clusters/src/clusters/liquid_glass/` (module and `HANDOVER.md`), and one more finished cluster from `CATALOG.md` (status ✅) once there is one, to match quality and conventions.

## The bar
- **It is the cluster.** It must look like it shipped in a 2027 car or a film's hero prop, on a 4K OLED, inside a real cockpit.
- **Instant feel.** Telemetry drives everything live: RPM, speed, gear, throttle and brake, boost, G, slip, yaw and steer as the design needs. Springs carry the motion: no jitter, but no lag either.
- **Signature moments.** The rev limiter / shift strobe and the gear change each need a specific, beautiful behaviour.
- **Cockpit fit.** It sits in the binnacle (`Layout` anchor), respects the user's height and scale, and has a hood variant if the catalog lists one.
- **Personalisation.**
  - All colours come from palette roles: factory, custom or random.
  - The font is swappable, with a motorsport default.
  - Analog designs show the car's 4K logo in the logo zone, or the custom name.
- **Never:** an opaque full-screen fill, flat rectangles posing as glow, hardcoded telemetry, block digits, or anything over budget.

## Steps
1. **Design first, on your board.**
   - With Figma: follow the family template and tokens on your Section. Make the 4 states at 4K over the cockpit reference photo, plus the motion notes.
   - Without Figma (the current state, see `CATALOG.md`): write the layout as a zone table at the top of your `mod.rs` (zones, sizes and positions in local 4K px, as the pilot's `Geom`), then design in code against `mockups/<Name>/index.html`.
   - For Liquid Glass, fluid, abstract or sci-fi designs, plan the bespoke assets now.
   - Run `design:design-critique` on the frames, and iterate until it clears the bar above.
2. **Bespoke assets** (Blender and ComfyUI via `cargo run -p asset-forge`).
   - Local tools on this machine (write them as `%USERPROFILE%` paths in any repo file):
     - Blender 5.3: `%USERPROFILE%DocumentsDash_Toolslender-5.3.0-windows-x64`
     - ComfyUI portable: `%USERPROFILE%DocumentsDash_ToolsComfyUI_windows_portable`
     - Figma Desktop with the `figma-desktop` MCP, when it is open (otherwise use the HTML board).
   - Use the Phase 6 recipes and workflows, or add a recipe module for your cluster under `tools/asset-forge/src/recipes/<snake_name>.rs`. That is your file too.
   - Outputs go to `assets/clusters/<snake_name>/`, with a row in `assets/clusters/<snake_name>/ASSETS.md`.
   - "Real abstract fluidity" means baked Blender liquid flipbooks, flow and normal maps **plus** the kit's real-time GPU fluid driven by telemetry. Combine them.
   - Every textured look keeps a procedural fallback: `setup.assets` may be `None`.
3. **Build:** replace the scaffold in your `mod.rs` with your `Cluster` (keep `pub fn make`). Use the kit and the `common/` components. Keep `update()` and `draw()` allocation-free after warm-up.
4. **Verify** (from the repo root):
   - `cargo run --release -p preview-export -- --cluster <Name>` writes `mockups/<Name>/`: 4 states per view at 4K, crops, the cockpit composite, the 60-frame sequence and `index.html`. It prints the GPU time of every frame and the sequence p50 / p99: that is your budget number (under 0.5 ms at 4K).
   - `FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters <snake_name>` writes your goldens; review the PNGs, then `cargo test -p fh6-clusters <snake_name>` must pass.
   - `cargo clippy -p fh6-clusters --all-targets -- -D warnings`, and format only your files: `rustfmt --edition 2021 crates/fh6-clusters/src/clusters/<snake_name>/*.rs`.
   - Live: `cargo run --release -p fh6-app --bin FH6_WheelDash -- --demo --cluster <Name> --stats` for 60 seconds; watch for jitter and read the CPU and latency lines.
   - Compare the renders with your board side by side, and fix the gaps.
5. **Critique:** run `design:design-critique` and the `ui-ux-designer` agent on the 4K renders. Iterate until both say it beats the references. Record the verdict.
6. **Handover:** write `crates/fh6-clusters/src/clusters/<snake_name>/HANDOVER.md` from `program/HANDOVER_TEMPLATE.md`, with renders (copy the four crops you want reviewed into `docs/handover-shots/cluster-<snake_name>-*.png`), perf numbers, the asset list and "my calls". Set your `CATALOG.md` row to ✅ in your branch (both tables); the conductor resolves that single-line change. Commit as TJX with no co-author trailer; do not push or merge.

## Lessons from the pilot (save yourself the time)
- **Long paths:** a `Shape::Polyline` evaluates every segment over its whole bounding box. One 128-point polyline round a 1240 px capsule cost 0.36 ms. Draw paths with `common::ribbon::PathRibbon`, or in chunks of at most 8 segments.
- **Overlapping translucent segments show as dots.** Keep segmented strokes opaque (darken the colour, do not fade its alpha).
- **Clips are rectangles** (optionally rounded). Keep content inside a curved shape by geometry, not by clipping to it.
- **Glass first is fine:** a cluster may open with its glass layer. Use one glass layer and put every rank 1–3 light into as few bloom layers as you can.
- **Alphas are linear.** A 3% ink fill shows as about 19% grey and a 50% black overlay barely darkens; the pilot's off pills use 1/255 and its body backing 0.94. The critique will measure what is on screen, not your numbers.
- **Fonts:** resolve faces once in `new()`, for example `setup.fonts.find("Saira", 200, false).unwrap_or(role)`. Pack faces load lazily, so a new glyph can appear a frame late. If `input.fonts.display` differs from the display role you saw in `new()`, the user picked a font: use it for your numerals.
- **Views:** keep one geometry table per view (`View::Cockpit`, `View::Hood`) and pick it from `input.view`; stay inside `layout.envelope` (a unit test on your table is cheap).
- **Telemetry honesty:** use `fh6_kit::Signals` and the verified raw fields (rpm, speed, gear, pedals, acceleration, velocity, power, boost). Tyre slip, suspension, pitch / roll, fuel, lap times and torque are not verified against `tests/golden/packets`: derive from verified fields or verify first, and say which in your handover.
- **The motion sequence** (`fh6_clusters::review::sequence_frame`) is idle → pull to the limiter → strobe → upshift → pull: check your two signature moments in `mockups/<Name>/sequence/`.
- **Errors:** `preview-export` prints wgpu validation errors; "rendered nothing" means every pixel was transparent.

## Exit criteria
- The design board has 4 states (per view).
- The module is complete, with no TODOs, and goldens pass.
- 4K renders plus the 60-frame sequence are in `mockups/<Name>/`.
- Live demo verified, with the GPU time under 0.5 ms recorded.
- The critique has passed, with its notes.
- The handover is written.
