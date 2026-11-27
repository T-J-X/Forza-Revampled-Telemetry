# Phase 3: design system & art direction (MASTER PROMPT)

You lead Phase 3 of FH6 WheelDash v2, running in your own chat and worktree. The old 29 clusters look like rough flat rectangles. Your job is to define a visual language so strong that every later cluster chat can build world-class 4K instrument clusters from it. You design; you do not write Rust.

## Read first
1. `ONBOARDING.md`, root `AGENTS.md`, root `SKILLS.md`.
2. This folder's `AGENTS.md` and `SKILLS.md`. Load the skills in the order given.
3. `docs/PROGRESS.md`, `docs/specs/phase-8-materials-and-shaders.md` (the optics and kinematics maths worth keeping), `.claude/skills/fh6-hmi-design-rules/SKILL.md`, and `program/phases/07-cluster-factory/CATALOG.md`.
4. To see what failed, look at `mockups/_thumbs/*/02_mid_cruise.png`. Generate them with `cargo run --release -p preview-export` if they're missing.
5. Reference photos: `assets/reference/cockpit_with_wheel.jpg` (stock cockpit) and `assets/reference/target_wide_cockpit_fov.jpg` (the target low-seat, wide-FOV view).

## Goal
A Penpot master file plus repo docs that define:
- the FH6 WheelDash design system
- every cluster family
- the hero concepts
- the OSD and configurator directions
- the full cluster catalog of 29 rebuilds and at least 15 new next-gen cockpit concepts

## Constraints you design for
- **Canvas:** a 3840x2160 transparent overlay on an LG C1 OLED, drawn over the live cockpit.
  - Off pixels are true black, which shows as fully transparent.
  - Never fill the screen.
  - Glass is alpha 28..64 with crisp 1 px specular rims.
- **Placement:** the cockpit cluster sits in the steering-column binnacle, centred at X = 1920, roughly Y 1700–2000 at default scale. Users can change its height and scale. Hood view has its own lower, wider layout.
- **Glanceability:** under 150 ms, with this rank order:
  1. hero gear plus shift lights / rev-limiter strobe
  2. tabular speed
  3. tach
  4. the visualisation under the speed (ADAS traffic, road map, tyre slip, G)
  5. bezel and micro-labels
- **Renderer:** GPU (wgpu with WGSL shaders), so glow, bloom, blur, gradients, glass fresnel/specular, fluid simulation, particles, 3D meshes and MSDF text are all available. Specify them; don't design around old CPU limits.
  - **Glass:** true background refraction needs a captured game frame and is optional. Every glass design must also look premium with tint, rim and caustic texture alone.
- **Type:** a strongly motorsport default, chosen from OFL fonts only (for example Saira / Saira Condensed, Titillium Web, Orbitron, Chakra Petch, Rajdhani, Michroma, Oxanium, Barlow Condensed). Users may load their own fonts, so layouts must tolerate other faces.
- **Colour:** each cluster has factory (brand palette from `assets/car_brands/car_brands.json`), custom and randomise modes. Define how a palette maps to each cluster's roles: primary, accent, shift, warning, glass tint.
- **Analog dials:** show the car's logo (4K PNG or SVG from Phase 6), or the user's custom name, in a defined logo zone.
- **Motion:** fluid, physical, spring-based. The rev limiter, shift strobe and gear change are signature moments. Define timings.

## Steps
1. **Set up Penpot.**
   - Follow `docs/design/PENPOT.md`: the `penpot` MCP, with the file open in a browser tab and connected.
   - Create the master file "FH6 WheelDash v2 — Design System" and record its file ID and page map in `docs/design/PENPOT.md`.
   - If the Penpot tools are unavailable, use HTML boards instead (`docs/design/boards/*.html`, reviewed in the built-in browser), and say so in the handover.
2. **Research** (`anthropic-skills:deep-research`, fanned out with the Workflow tool, at most 6 agents).
   - Sources:
     - GT7 HUD and meters
     - 2026–27 production clusters: Porsche 911 and Taycan, Lucid Gravity, BMW Panoramic iDrive, MBUX Hyperscreen, CarPlay Ultra in the Aston Martin
     - F1 and LMDh steering-wheel displays and Bosch / AiM / MoTeC dashes
     - analog horology (Pagani, Bugatti, Koenigsegg)
     - sci-fi film and game UI (Oblivion, Tron Legacy, Blade Runner 2049, Cyberpunk 2077, The Expanse)
     - fluid, generative and motion work
   - Synthesise with `design:research-synthesis` into `docs/design/RESEARCH.md`: what makes each one feel premium, with specific techniques.
3. **Design system** (`design:design-system`, `design-superpowers:ds-make`, the `penpot` MCP for token sets and library components).
   - **Tokens:** colour roles, the type scale (including tabular hero digits), stroke and rim widths, radii, glow and bloom levels, glass material parameters, motion curves and durations.
   - Write them to `data/design_tokens.json`. Extend its schema, keep it valid JSON, and document it in `docs/design/TOKENS.md`.
   - **Components:** hero gear, tabular speed, tach (arc, bar, ribbon, needle-dial variants), shift-light rows, rev-limiter strobe, logo zone, micro-labels, glass plate, bezel, ADAS road, G-ball, tyre-slip quad.
4. **Families.** Define at least 5 families (Classic Analog, Modern Digital 2027, Sci-Fi, Fluid / Generative, ADAS & HUD), and add more if research supports it. For each, give a template frame and a written spec in `docs/design/families/<family>.md`.
5. **Hero concepts.** Make 6 at 4K, one per family plus a Liquid Glass flagship. Each shows 4 states: idle, mid cruise, redline strobe, drift / braking.
   - Lay them out in Penpot (`execute_code`, checked with `export_shape`). Use `blender-hero-assets` and `comfyui-hero-textures` for glow, glass and material looks and for depth / ADAS, and write the motion spec in the family doc.
   - Export PNGs to `docs/design/heroes/`.
6. **Catalog.** Rewrite `program/phases/07-cluster-factory/CATALOG.md`:
   - every one of the 29 existing clusters with its family, a new art direction and key references
   - at least **15 new next-gen cockpit concepts** that feel out of this world, each with a one-paragraph brief, a family and references
   - pattern-setters marked (one per family) and a build order
7. **OSD and configurator.** Give one direction frame each (`program/phases/09-osd` and `10-configurator-release` will design them fully).
   - **OSD:** full-screen Liquid Glass, controller-first, with a live cluster view.
   - **Configurator:** a web page with a live preview, colour, font, FOV, seat and height controls, and the Steam launch command.
8. **Scrim fit.** Write a spec for the OEM dash blocker over `assets/reference/cockpit_with_wheel.jpg`: the dome outline, feather widths, and how the cluster sits in it, in `docs/design/SCRIM_FIT.md`.
9. **Critique loop.** Run `design:design-critique` and the `ui-ux-designer` agent on every hero frame. Fix and re-export until each is clearly better than the references, not merely as good. Record the verdicts in `docs/design/CRITIQUE.md`.
10. **Handover:** follow `program/HANDOVER_TEMPLATE.md`. Include Penpot board IDs for the library, the family templates and each hero.

## Exit criteria
- `docs/design/PENPOT.md` holds the file ID and page map, or `docs/design/BOARDS.md` records the HTML-board fallback.
- `docs/design/RESEARCH.md`, `TOKENS.md`, `SCRIM_FIT.md`, `CRITIQUE.md` and `families/*.md` exist, and `data/design_tokens.json` is extended and parses as valid JSON.
- 6 hero concepts × 4 states are exported as 4K PNGs in `docs/design/heroes/`.
- `CATALOG.md` lists 29 rebuilds plus at least 15 new concepts, with families, briefs, references, pattern-setters and build order.
- No Rust changes. Text files are UTF-8 without BOM.

## Out of scope
Writing Rust, and sourcing or rendering final textures (that is Phase 6).
