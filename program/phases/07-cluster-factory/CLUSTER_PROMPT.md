# Cluster chat (CLUSTER PROMPT)

The conductor starts you with: *"You are the cluster chat for `<Name>` (catalog row `<n>`). Read `ONBOARDING.md`, then follow `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`."* You own exactly one cluster: its Figma page, its module directory `crates/fh6-clusters/src/clusters/<snake_name>/` and its assets. **Edit nothing outside them.** If a shared helper or the kit needs a change, write it in your handover so the conductor can schedule it.

## Read first
- Root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`.
- Your catalog row and `spec.md`, your family spec in `docs/design/families/`, the matching hero frames in `docs/design/heroes/`, `docs/kit/README.md` and `assets/ASSETS.md`.
- 2 finished clusters from `CATALOG.md` (status ✅) to match quality and conventions.

## The bar
- **It is the cluster.** It must look like it shipped in a 2027 car or a film's hero prop, on a 4K OLED, inside a real cockpit.
- **Instant feel.** Telemetry drives everything live: RPM, speed, gear, throttle and brake, boost, G, slip, yaw and steer as the design needs. Springs carry the motion: no jitter, but no lag either.
- **Signature moments.** The rev limiter / shift strobe and the gear change each need a specific, beautiful behaviour.
- **Cockpit fit.** It sits in the binnacle (`Layout` anchor), respects the user's height and scale, and has a hood variant if the family calls for one.
- **Personalisation.**
  - All colours come from palette roles: factory, custom or random.
  - The font is swappable, with a motorsport default.
  - Analog designs show the car's 4K logo in the logo zone, or the custom name.
- **Never:** an opaque full-screen fill, flat rectangles posing as glow, hardcoded telemetry, block digits, or anything over budget.

## Steps
1. **Design first, in Figma, on your page.** Follow the family template and tokens. Make the 4 states at 4K over the cockpit reference photo, plus the motion notes.
   - For Liquid Glass, fluid, abstract or sci-fi designs, plan the bespoke assets now.
   - Run `design:design-critique` on the frames, and iterate until it clears the bar above.
2. **Bespoke assets** (Blender and ComfyUI via `cargo run -p asset-forge`).
   - Use the Phase 6 recipes and workflows, or add a recipe module for your cluster under `tools/asset-forge/src/recipes/<snake_name>.rs`. That is your directory too.
   - Outputs go to `assets/clusters/<snake_name>/`, with a row in `assets/clusters/<snake_name>/ASSETS.md`.
   - "Real abstract fluidity" means baked Blender liquid flipbooks, flow and normal maps **plus** the kit's real-time GPU fluid driven by telemetry. Combine them.
3. **Build:** implement `Cluster` in your module with the kit and `common/` helpers. Keep `update()` and `draw()` allocation-free after warm-up.
4. **Verify:**
   - `cargo test -p fh6-clusters <name>` (golden images) and `cargo clippy -D warnings`.
   - `preview-export --cluster <Name>`, then compare against your Figma frames side by side, and fix the gaps.
   - Run it live with `--demo` for 60 seconds and watch for jitter.
   - Check the GPU time at 4K is under 0.5 ms (`--stats`).
5. **Critique:** run `design:design-critique` and the `ui-ux-designer` agent on the 4K renders. Iterate until both say it beats the references. Record the verdict.
6. **Handover:** write `crates/fh6-clusters/src/clusters/<snake_name>/HANDOVER.md` from the template, with renders, perf numbers, the asset list and "my calls". Set your `CATALOG.md` row to ✅ in your branch; the conductor resolves that single-line change.

## Exit criteria
- The Figma page has 4 states.
- The module is complete, with no TODOs, and goldens pass.
- 4K renders plus the 60-frame sequence are in `mockups/<Name>/`.
- Live demo verified, with the GPU time under 0.5 ms recorded.
- The critique has passed, with its notes.
- The handover is written.
