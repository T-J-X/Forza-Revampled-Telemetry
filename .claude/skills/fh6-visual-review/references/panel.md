# The six-lead review panel

TJS, 2026-11-28: "where is the 6 review panel gone for every dash so I don't need to babysit". Every dash runs this panel at its closing review (`CLUSTER_PROMPT.md` step 10), on the final build and after the handover draft, before it merges (`AGENTS.md` rule 16). At the direction check it may run too (pre-approved), on the first build.

## Who

Six fresh-context reviewers, one for each lead in `program/phases/07-cluster-factory/AGENTS.md`, and then one checker: seven agents in all, which root `SKILLS.md` section 6 pre-approves. Start the six at once, as six parallel Agent calls in one message (`general-purpose`), or as one Workflow where the session has the opt-in. Each one sees the dash's files and the rules: this is an expert panel, not the blind read (the `cold-reviewer` stays the one blind reviewer).

| # | Reviewer | Looks at | Answers | Loads |
|---|---|---|---|---|
| 1 | **Design director** (Agent 3: 3A for Liquid Glass and spatial glass, 3B for classic analog, motorsport and GT7, 3C for sci-fi, retro, synthwave, psychedelic and abstract, 3D for ADAS, HUD and AR) | the four states in each view, the cockpit composite, the product page, the sequence's signature moments, the Penpot page, the family spec, "The bar" | closing lines 22, 26 to 29, 31; V10, V15 to V19 | `design:design-critique`, `fh6-hmi-design-rules` |
| 2 | **HMI/UX** (Agent 1) | the glance and squint views and their zones, the binnacle fit, the hood view, the logo zone, the blind read's answers | closing lines 22, 23, 24; V4 to V12 | `design:accessibility-review`, `fh6-hmi-design-rules` |
| 3 | **Research scout** (Agent 0) | `RESEARCH.md` and its sources, the reference photos, the frozen feature inventory against the render | closing line 25; V13 to V16 | web search, only to check a cited source |
| 4 | **Technical artist** (Agent 5) | `assets/clusters/<snake_name>/` and its `ASSETS.md`, the recipe, the Drive masters list, the materials at glance scale | closing lines 27, 30, 32, 33; V17, V18 (`references/materials.md`) | `blender-hero-assets`, `comfyui-hero-textures` |
| 5 | **Rust systems and graphics** (Agent 2) | the module (`mod.rs` and its submodules), the recipe, the perf numbers, the `dash_gates` output | closing lines 1 to 21 | `rust-review` with `rust-skills` |
| 6 | **QA** (Agent 4) | the handover's evidence: goldens, perf, live bench, workspace gate, the answers to every checklist line | closing lines 5, 6, 13, 34 to 39; V21 to V26 | none |

## The brief

Give each reviewer this, with the brackets filled in and nothing else changed:

> You are the [reviewer] on a six-reviewer panel for the dash [Name] (catalog row [n], family [family]) in the FH6 WheelDash repo at [repo root]. Read `AGENTS.md`, `SKILLS.md`, `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` and `program/phases/07-cluster-factory/AGENTS.md`, and your lead's section of `AGENTS.md`. Review only your lead's mandate: [mandate from the table above].
> Look at: the module `crates/fh6-clusters/src/clusters/[snake_name]/` (`RESEARCH.md`, `spec.md`, `README.md`, the `HANDOVER.md` draft), `mockups/[Name]/` (the states, the cockpit composites, the crops, `sequence/`, `review/`), `assets/clusters/[snake_name]/`, `tools/asset-forge/src/recipes/[snake_name].rs` and `docs/handover-shots/cluster-[snake_name]-*.png`.
> 1. Answer each of these lines yes or no, with its evidence (a file and line, an image path and the place in it, or a command and its output): [closing lines and V lines from the table].
> 2. Then list every other finding in your mandate: what is wrong, where, the evidence, and the smallest fix.
> Findings and yes/no answers only: no scores, ratings, grades, severity labels and no overall verdict. Look once; you will not be asked again. Judge detail on 1:1 crops (`cargo run --release -p visual-review -- tiles` or `crop`), never on a downscaled 4K frame. Read-only: edit, stage and commit nothing, and run no GPU job (no `preview-export`, goldens or app); `visual-review`, `cargo check`, `cargo clippy` and `cargo test -p fh6-clusters --test dash_gates` are allowed. If you find nothing, say "nothing found".

## The checker

When all six have answered, start one more fresh agent with every finding and answer, and the same paths. For each, it opens the evidence (the file, the image, or the command again) and says confirmed or not confirmed, with its reason. It adds a finding of its own only where a reviewer's evidence shows something else wrong.

## After the panel

- Fix every confirmed finding you accept, and write why for one you don't. Re-capture and re-check only what a fix touched, then walk the closing checklist again.
- Never run the panel again to get a different answer, and never reword a brief: that is the critic loop `AGENTS.md` rule 12 forbids. A redesign after a no-go is a new review point, with a new panel.
- Write `## Review panel` in the handover: for each reviewer, its yes/no answers and findings (short), the checker's answer to each, and what was done.
- Post `REVIEW PANEL <Name>` in your thread: how many findings, how many confirmed, and whether each was fixed or reported. Then merge (`AGENTS.md` rule 16).
