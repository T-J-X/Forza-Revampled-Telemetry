# Branch and pull request cleanup log

How a branch on GitHub (`origin`, T-J-X/Forza-Revampled-Telemetry) gets deleted, and what was there before each cleanup, so that anything deleted can be put back. AGENTS.md rule 17 is the rule this file serves. The first three sections stand; after them each cleanup has its own dated section, newest first.

## Who can delete a branch

**Not a thread on its own.** Claude's cloud sessions reach GitHub through a proxy that lets a push create or update a branch and refuses every branch deletion with `HTTP 403` (the "GitHub proxy" part of https://code.claude.com/docs/en/cloud-environments). The GitHub tools a thread has do not delete branches either. A Remote Control session on the PC has the PC's own GitHub sign-in, but its permission check denied the delete as destructive before git ran (2026-11-28). Both are guardrails, not faults. Do not retry a refused delete, and do not look for another route.

Two can:

1. **GitHub itself**, through the repository setting "Automatically delete head branches" (Settings, General, Pull Requests), which has been on since 2026-11-28. It is TJX's to change. A thread can read it: `gh api repos/T-J-X/Forza-Revampled-Telemetry --jq .delete_branch_on_merge` prints `true` (04:58 UTC on 2026-11-28). With it on, GitHub deletes a pull request's head branch when the pull request merges. Its documentation does not say whether that covers the way work is merged here: a push of the branch's commits to `main`, which GitHub then marks as merged (an "indirect merge"). It does: the first merge after the setting went on, pull request #12 at 05:08 UTC on 2026-11-28, was a push to `main` from a Claude cloud session, and GitHub deleted its branch within seconds (see that day's section). Public reports say a push made by GitHub Actions' own token may not trigger it. GitHub ties a branch to a merge only through its pull request, which is why rule 17 gives every pushed branch one.
2. **TJX, signed in on GitHub.** The link `https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=<code>`, with `<code>` a part of the name that no other branch name contains (`git ls-remote --heads origin | grep -c <code>` prints 1; for a `claude/project-thread-...` name that is its last six characters), shows that one branch and its bin. Press the bin only on the row that shows the full name. The page of a merged pull request has a "Delete branch" button. Neither button has a lease: it deletes whatever the branch points at when it is pressed, so whoever presses first sees that the head is still the checked one (`git ls-remote origin refs/heads/<branch>`, or 0 commits ahead of `main` on that row). A thread that pushes to its branch again tells the coordinator, who takes the link back. At a terminal with TJX's own GitHub sign-in, step 5 deletes with a lease. TJX can also have a thread press those buttons in the browser of the Claude desktop app, where TJX is signed in to GitHub (step 6). That is TJX's delete, not the thread's: a thread does it only when TJX has told that thread to, and only for the branches TJX named. No thread opens that browser to delete a branch on its own.

## The check before a branch is deleted

The coordinator runs steps 1 to 4 before it passes TJX the link above. A delete after a single merge is passed to TJX in a message that holds the branch's full name, its checked head, its pull request number and the link, and needs no section here; a cleanup of several branches gets a dated section.

1. Work in a full clone: if `git rev-parse --is-shallow-repository` prints `true`, run `git fetch --unshallow origin`. Then `git fetch --prune origin`.
2. Take the branch's head from `git ls-remote origin`. `git merge-base --is-ancestor <head> origin/main` exits 0, so every commit on the branch is already in `main`. If it exits 1 because the work reached `main` rebased or squashed, run `git merge-tree --write-tree origin/main <head>`: when it prints the same id as `git rev-parse origin/main^{tree}`, merging the branch would change nothing in `main`, so nothing is missing. Any other id, or exit 1, means the branch holds work `main` lacks, and it stays.
3. No open pull request has the branch as its head or its base. Deleting the head branch of an open pull request closes it.
4. The branch's thread has closed.
5. Only TJX, at a terminal with their own GitHub sign-in: delete with a lease, so that the delete goes through only if the branch still points at the head that was checked: `git push --force-with-lease=refs/heads/<branch>:<head> origin :refs/heads/<branch>`. A plain `git push origin --delete <branch>` would also delete commits pushed since the check.
6. Only a thread that TJX has told to, in the browser of the Claude desktop app: open the branch's link and, just before the click, see that the page lists that one branch and no other, that GitHub counts it 0 commits ahead of `main`, that it has no open pull request, and that `git ls-remote origin` still gives the head that was checked. The "Delete branch" button has no lease, so this look is the lease. Take one branch at a time, and see with `git ls-remote origin` that it is gone before the next.

A branch is kept when `main` lacks any of its commits, or when its thread is still at work, even if it is merged. If unsure, keep it and tell the coordinator. With "Automatically delete head branches" on, GitHub may delete the branch of a thread that still has an item left once its pull request merges. That is expected and loses nothing, since the branch is then in `main`: the thread's next push creates the branch again, and it opens a new draft pull request for it.

## Restoring a branch

A deleted branch whose head is in `main`'s history comes back from a full clone with

```
git push origin <head>:refs/heads/<branch>
```

with the head and the branch name from that cleanup's table, from the coordinator's message to TJX, or from `refs/pull/<number>/head`. A thread can run this, because a push that creates a branch is allowed. GitHub also keeps every pull request's head as `refs/pull/<number>/head`, and a merged pull request whose branch was deleted shows a "Restore branch" button. The button works from that pull request's own head, so for `claude/project-thread-ruv0ss` use #6, not #2. A checkout that still lists a deleted branch under `origin/` drops it with `git fetch --prune origin`. Local branches and worktrees are not touched by any of this.

## 2026-11-28

TJX found GitHub cluttered with the branches of finished threads and asked for a cleanup that breaks nothing. Until now nothing deleted a thread's branch after its merge.

**Snapshot:** `git ls-remote origin` at 03:52 UTC, with `main` at `fd13cf5`. There were no tags. Seven threads were at work: Rust fix phases F and G, the Liquid Glass OSD, and the TeslaAdas, BoschDdu, RetroCyber and Gt7HoodPro rebuilds (Gt7HoodPro had not pushed a branch yet). Their branches keep moving, and a branch that moves or appears after the snapshot is not part of this cleanup.

**Result:** six branches are deleted, the setting is on, and nothing else changed. GitHub went from 16 branches to 10. Its 12 pull requests stayed as they were: 5 merged, and 7 open drafts of threads at work.

- 03:56 UTC: the seven leased deletes from the cleanup thread's cloud session were refused with `HTTP 403`.
- 04:20 UTC: the cloud GPU lane's thread came back and pushed one more commit (`7648f02`) to `claude/project-thread-obs70c` and to `main` at the same time. That branch is in use again, so it leaves this cleanup.
- 04:21 UTC: TJX had said to force it, so a Remote Control session on the PC tried the leased deletes with the PC's own GitHub sign-in. Its permission check denied the command as destructive before git ran. The command still named the old head `8aebf2b` for `claude/project-thread-obs70c`, so the lease in step 5 would have refused that one, where a plain delete would have removed a branch its thread had just pushed to.
- 04:24 UTC: TJX was asked to press the links and turn the setting on.
- 04:25 UTC: TJX answered "yes sign in here and do that", choosing to sign in to GitHub in the browser of the Claude desktop app and to have this cleanup's thread press the buttons there. At 04:38 UTC TJX was signed in and said to go ahead.
- 04:40 to 04:41 UTC: the thread deleted the six branches below there, one at a time, each with the "Delete branch" button on its own filtered page (step 6). Before each click the page listed that one branch, 0 commits ahead of `main` and with no open pull request, and `git ls-remote origin` gave the head in the table. After each click `git ls-remote origin` no longer listed the branch. It still lists `refs/pull/1/head`, `refs/pull/2/head`, `refs/pull/4/head` and `refs/pull/6/head` at their old commits.
- 04:42 UTC: the thread ticked "Automatically delete head branches" in the repository settings, and the box was still ticked after a reload.
- 05:08:51 UTC: the thread pushed this cleanup's branch to `main` (`8ac3aff`). GitHub marked pull request #12 merged at 05:08:53 and deleted `claude/project-thread-5xdfjx` by itself; `git ls-remote origin` no longer listed it at 05:08:57. So the setting works for the way work is merged here. GitHub then held 9 branches.

The six, each with the link that showed it:

- `claude/phase-3-design-system-449d2f`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=449d2f
- `claude/project-thread-euvhdf`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=euvhdf
- `claude/project-thread-ixh90u`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=ixh90u
- `claude/project-thread-muqx44`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=muqx44
- `claude/project-thread-ruv0ss`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=ruv0ss
- `claude/project-thread-urfb8s`: https://github.com/T-J-X/Forza-Revampled-Telemetry/branches/all?query=urfb8s

Between the snapshot and the deletes two branches appeared, `claude/project-thread-h7qahw` (the Gt7HoodPro rebuild) and `claude/project-thread-5xdfjx` (this cleanup), and three pull requests were opened (#10 for the Gt7HoodPro rebuild, #11 for the Liquid Glass OSD, #12 for this cleanup). That is why 16 branches and 12 pull requests stood before the first delete.

Left on GitHub: `main` and nine thread branches. Seven belong to threads at work, each with an open draft pull request (`191tnq`, `5xdfjx`, `czxgs6`, `h7qahw`, `ovnqwk`, `q6crd5`, `r828zb`). Two are fully in `main` and wait for the next sweep: `claude/project-thread-obs70c` (cloud GPU lane, no pull request, its thread was at work at 04:20 UTC) and `claude/project-thread-ocd2bw` (Rust fix G, #5 merged, its thread has one item left).

### Branches before: 14

| Branch | Head | Work | Pull request | Commits `main` lacks | Decision |
|---|---|---|---|---|---|
| `main` | `fd13cf50ea15f903b8567a70a98639d825de62a9` | | | | keep |
| `claude/phase-3-design-system-449d2f` | `2d8200a90a69f561ccaa231dcfd1cdb273654f64` | Phase 3, design system and art direction | #1, merged | 0 | delete |
| `claude/project-thread-euvhdf` | `a8ef6a9b129f94b9ece48f346d864362c6f996be` | Rust fix phase E | #4, merged | 0 | delete |
| `claude/project-thread-ixh90u` | `0ab6687ab28663da6ccc1cefe78c3158dfababb4` | Tool setup: Penpot as the only design tool | none | 0 | delete |
| `claude/project-thread-muqx44` | `e4d9ee8439bd12a6bc3055f41d572b97643549e8` | Host CPU phase | none | 0 | delete |
| `claude/project-thread-obs70c` | `8aebf2b8210b18c5c245ee6b4a92d3d66c1fba92` | Cloud GPU lane | none | 0 | delete at the snapshot; keep since about 04:20 UTC, when its thread pushed new work |
| `claude/project-thread-ruv0ss` | `8212145877d4921659038f3d7fab1ff64cceb4cd` | Rust fix phase D | #2 and #6, merged | 0 | delete |
| `claude/project-thread-urfb8s` | `2f85973a374da84c0f7426b161693d9f8f6a852d` | PorscheAnalog rebuild | none | 0 | delete |
| `claude/project-thread-ocd2bw` | `fd13cf50ea15f903b8567a70a98639d825de62a9` | Rust fix phase G | #5, merged | 0 | keep: merged, but its thread has one item left (it waits for fix F) |
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
