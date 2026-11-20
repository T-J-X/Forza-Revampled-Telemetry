# Phase 7 setup: cluster factory (MASTER PROMPT, chat 07-00)

You lead the setup chat for the cluster factory, running in your own chat and worktree. After you, the conductor starts **one chat per cluster** (at most 3 at once), each following `CLUSTER_PROMPT.md`. Your job is to make those chats fast, consistent and free of merge conflicts.

## Read first
`ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md`, `SKILLS.md`, `CATALOG.md` and `CLUSTER_PROMPT.md`, plus the handovers of Phases 3, 5 and 6, `docs/kit/README.md`, `docs/design/*` and `assets/ASSETS.md`.

## Required work
1. **Scaffold every catalog entry.**
   - Give each one a module directory `crates/fh6-clusters/src/clusters/<snake_name>/` containing `mod.rs` (a `Cluster` impl that draws a tasteful "under construction" frame using the kit, never an opaque fill), `spec.md` (copied from the catalog brief) and `README.md`.
   - Add a `ClusterTheme` variant for each **new** concept. Append only; keep existing numeric values.
   - Generate the registry table once so later cluster chats edit only their own directory. Nobody edits shared files after this.
2. **Shared factory helpers:** if 3 or more catalog entries need the same component (hero gear, shift-light row, tabular speed block, logo zone, tach variants), build it once in `crates/fh6-clusters/src/common/`, from the Phase 3 components and the kit. Cluster chats may compose these components but must not fork them.
3. **Per-cluster tooling:**
   - `cargo run -p preview-export -- --cluster <Name>` renders the 4 states at 4K plus thumbnails and a short animated sequence (PNG frames, 60 frames: idle → redline strobe → gear change) to `mockups/<Name>/`.
   - `cargo test -p fh6-clusters <name>` runs its golden-image test.
   - `cargo run -p fh6-app --bin FH6_WheelDash -- --demo --cluster <Name>` shows it live with the dyno.
4. **Figma pages:** in the Phase 3 master file, create one empty page per cluster named `07 · <Name>`, holding a 3840x2160 frame with the binnacle guide. Record the node IDs in `CATALOG.md`.
5. **Order the catalog:**
   1. one pattern-setter per family (Phase 3 marks them)
   2. the remaining 29 rebuilds
   3. the new next-gen concepts

   Then mark the first 3 as `next`.
6. **Pilot:** build the first pattern-setter yourself, end to end, following `CLUSTER_PROMPT.md`. It proves the loop. Fix anything in the prompt, helpers or tooling that slowed you down.

## Exit criteria
- Every catalog entry has a module that compiles and is registered, and appears in `preview-export`.
- `cargo test` / `clippy -D warnings` are green.
- The per-cluster commands work.
- The pilot cluster is done to the `CLUSTER_PROMPT.md` exit bar.
- `CATALOG.md` has the order, the Figma node IDs and the status column.
