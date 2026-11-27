# HANDOVER: Phase <N> · <name>

**Chat / branch:** <branch name>  **Date:** <YYYY-MM-DD>  **Result:** ✅ complete / ◐ partial / ❌ blocked

## Exit criteria
| Criterion (copied from the MASTER_PROMPT) | Met? | Evidence (command output, file, screenshot) |
|---|---|---|

## What was built
<short list of files, modules and assets added or changed>

## Verification output
```
<paste the real tail of cargo check / cargo test / cargo clippy and any perf numbers>
```

## Visuals
<paths to renders or screenshots, e.g. mockups/<Cluster>/..., docs/design/...>

## My calls
<decisions made without asking, each with a one-line reason>

## Known issues / debt
<anything not done, with the reason>

## What the next phases must know
<APIs, file locations, gotchas, Penpot board IDs>

## Dash threads
A dash handover lives in the dash's own module directory. Title it `# HANDOVER: Dash · <Name> (catalog row <n>)`, copy the exit criteria from `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` (not from a MASTER_PROMPT), and add these sections. Findings and yes/no answers only: no scores.

- `## Closing review`: every line of the checklist, each yes or no with its evidence.
- `## Direction check`: the self-review of the first build (the decision, each finding and what was done about it), and TJX's answer quoted where TJX gave one, with each change TJX listed and where it was done.
- `## Before and after pair`: the full-frame render kept before the work started, and the final one.
- `## Assets and masters`: the runtime files with recipe, seed and licence; the master file names in the Drive folder.
- `## Changes from the real display and from the brief`.
- `## Shared places touched` and `## Requests for shared code`.
- `## What sibling dashes must know` (in place of "What the next phases must know").
- `## Review findings`: the findings of the closing review's skills and agents (step 10), each with what was done about it. The self-review stands in for TJX's sign-off (AGENTS.md rule 12); anything TJX says later is added here by a later commit.
