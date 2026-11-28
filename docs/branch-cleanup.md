# Branch and pull request cleanup log

What was on GitHub (`origin`, T-J-X/Forza-Revampled-Telemetry) before each cleanup, so that anything deleted can be put back. AGENTS.md rule 17 keeps it tidy between cleanups. Each cleanup gets its own dated section, newest first.

## 2026-11-28

TJX found GitHub cluttered with the branches of finished threads and asked for a cleanup that breaks nothing. Until now nothing deleted a thread's branch after its merge.

**Snapshot:** `git ls-remote origin` at 03:52 UTC, with `main` at `fd13cf5`. There were no tags. Seven threads were at work: Rust fix phases F and G, the Liquid Glass OSD, and the TeslaAdas, BoschDdu, RetroCyber and Gt7HoodPro rebuilds (Gt7HoodPro had not pushed a branch yet). Their branches keep moving, and a branch that moves or appears after the snapshot is not part of this cleanup.

**Status:** the seven deletes follow the commit that adds this section. A "Result" part is added here when they are done.

### Branches before: 14

| Branch | Head | Work | Pull request | Commits `main` lacks | Decision |
|---|---|---|---|---|---|
| `main` | `fd13cf50ea15f903b8567a70a98639d825de62a9` | | | | keep |
| `claude/phase-3-design-system-449d2f` | `2d8200a90a69f561ccaa231dcfd1cdb273654f64` | Phase 3, design system and art direction | #1, merged | 0 | delete |
| `claude/project-thread-euvhdf` | `a8ef6a9b129f94b9ece48f346d864362c6f996be` | Rust fix phase E | #4, merged | 0 | delete |
| `claude/project-thread-ixh90u` | `0ab6687ab28663da6ccc1cefe78c3158dfababb4` | Tool setup: Penpot as the only design tool | none | 0 | delete |
| `claude/project-thread-muqx44` | `e4d9ee8439bd12a6bc3055f41d572b97643549e8` | Host CPU phase | none | 0 | delete |
| `claude/project-thread-obs70c` | `8aebf2b8210b18c5c245ee6b4a92d3d66c1fba92` | Cloud GPU lane | none | 0 | delete |
| `claude/project-thread-ruv0ss` | `8212145877d4921659038f3d7fab1ff64cceb4cd` | Rust fix phase D | #2 and #6, merged | 0 | delete |
| `claude/project-thread-urfb8s` | `2f85973a374da84c0f7426b161693d9f8f6a852d` | PorscheAnalog rebuild | none | 0 | delete |
| `claude/project-thread-ocd2bw` | `fd13cf50ea15f903b8567a70a98639d825de62a9` | Rust fix phase G | #5, merged | 0 | keep: merged, but its thread has one item left (it waits for fix F); the thread deletes it when it closes |
| `claude/project-thread-czxgs6` | `44565a2ba6647675894a8d47a3f1127843107fe5` | Liquid Glass OSD rebuild, at work | none | 19 | keep |
| `claude/project-thread-q6crd5` | `3d9f02c94deb4887e4deeff1a508ccb163efb969` | Rust fix phase F, at work | #3, open draft | 9 | keep |
| `claude/project-thread-r828zb` | `cfce4957d57d6c6543897a143286c3fefefdf7e8` | TeslaAdas rebuild, at work | #7, open draft | 2 | keep |
| `claude/project-thread-ovnqwk` | `9a75acc5e0deb9ba87c05926c52663788daacf1c` | BoschDdu rebuild, at work | #8, open draft | 5 | keep |
| `claude/project-thread-191tnq` | `0ee979c4eb56493b093181f57fa8ea58490957ce` | RetroCyber rebuild, at work | #9, open draft | 6 | keep |

"Work" is read from the commits on the branch. "Commits `main` lacks" is `git rev-list --count origin/main..origin/<branch>`.

### Pull requests before: 9

Every one has `main` as its base.

| PR | Title | State | Head branch | Head | Decision |
|---|---|---|---|---|---|
| #1 | Phase 3: design system & art direction (hero concepts, tokens v3, catalog) | merged 2026-11-22 | `claude/phase-3-design-system-449d2f` | `2d8200a90a69f561ccaa231dcfd1cdb273654f64` | leave |
| #2 | Rust fix D: one typed config | merged 2026-11-27 | `claude/project-thread-ruv0ss` | `21b591fe4a0a76968c4f581c566c147293d6a5ca` | leave |
| #3 | Rust fix F: load the brand logos | open draft | `claude/project-thread-q6crd5` | `3d9f02c94deb4887e4deeff1a508ccb163efb969` | leave, at work |
| #4 | Rust fix E: remove the v1 code | merged 2026-11-27 | `claude/project-thread-euvhdf` | `a8ef6a9b129f94b9ece48f346d864362c6f996be` | leave |
| #5 | Rust fix G: gates that mean something | merged 2026-11-28 | `claude/project-thread-ocd2bw` | `fd13cf50ea15f903b8567a70a98639d825de62a9` | leave |
| #6 | Rust fix D, step 2: one typed config | merged 2026-11-28 | `claude/project-thread-ruv0ss` | `8212145877d4921659038f3d7fab1ff64cceb4cd` | leave |
| #7 | TeslaAdas: rebuild the dash from the ground up (catalog row 15) | open draft | `claude/project-thread-r828zb` | `cfce4957d57d6c6543897a143286c3fefefdf7e8` | leave, at work |
| #8 | BoschDdu rebuild: the Bosch DDU 10 of the BMW M4 GT3 | open draft | `claude/project-thread-ovnqwk` | `9a75acc5e0deb9ba87c05926c52663788daacf1c` | leave, at work |
| #9 | RetroCyber: rebuild as the Toyota Supra A70 Panoramic Digital Meter | open draft | `claude/project-thread-191tnq` | `0ee979c4eb56493b093181f57fa8ea58490957ce` | leave, at work |

No open pull request was already merged or superseded, so none is closed. The four open ones belong to threads at work.

### The check before each delete

1. Work in a full clone: if `git rev-parse --is-shallow-repository` prints `true`, run `git fetch --unshallow origin`. Then `git fetch --prune origin`.
2. Take the branch's head from `git ls-remote origin`. `git merge-base --is-ancestor <head> origin/main` exits 0, so every commit on the branch is already in `main`.
3. No open pull request has the branch as its head or its base. Deleting the head branch of an open pull request closes it.
4. The branch's thread has closed, or it is your own thread's branch and its work is on `main`.
5. Delete with a lease, so that the delete goes through only if the branch still points at the head that was checked: `git push --force-with-lease=refs/heads/<branch>:<head> origin :refs/heads/<branch>`. A plain `git push origin --delete <branch>` would also delete commits pushed since the check.

A branch is kept when `main` lacks any of its commits, or when its thread is still at work, even if it is merged. If unsure, keep it and tell the coordinator.

### Restoring a branch

The seven heads marked "delete" are all commits in `main`'s history, so from a full clone:

```
git push origin <head>:refs/heads/<branch>
```

with the head and the branch name from the table. GitHub also keeps every pull request's head as `refs/pull/<number>/head`, and a merged pull request whose branch was deleted shows a "Restore branch" button. The button works from that pull request's own head, so for `claude/project-thread-ruv0ss` use #6, not #2. A checkout that still lists a deleted branch under `origin/` drops it with `git fetch --prune origin`. Local branches and worktrees are not touched by any of this.
