# gpu-lock

The first-come-first-served queue for the GPU lock files that the threads share (rule: `SKILLS.md` section 5).

TJX, 2026-11-28: "as soon as gpu lock is off the next task that needs the gpu uses it". Before this tool, a thread that found a lock taken tried again later, so the card sat idle between jobs, and a lock left behind by a thread that crashed blocked everyone until somebody asked about it (the coordinator saw that happen around midnight before this tool).

## Where it is

Every checkout and worktree on the PC uses the same copy:

```
%USERPROFILE%\Documents\Dash_Tools\bin\gpu-lock.exe
```

In the Bash tool that is `"$USERPROFILE/Documents/Dash_Tools/bin/gpu-lock.exe"`. If it is missing, build it from any checkout and copy it there (no `gpu-lock` may be running while it is replaced):

```
cargo build --release -p gpu-lock
cp target/release/gpu-lock.exe "$USERPROFILE/Documents/Dash_Tools/bin/"
```

## Using it

One command, holding the lock for exactly as long as the command runs. Start it with the Bash tool's `run_in_background`, because the wait can be longer than the tool's time limit; you are told when it ends:

```
gpu-lock.exe run --thread "<thread name>" --job "<what>" -- cargo run --release -p preview-export -- --cluster <Name>
```

It prints its place in line while it waits, `LOCKED gpu.lock ...` when its turn comes, then the command's own output, and it exits with the command's exit code. `--lane cloud` queues for the cloud pod instead (`gpu-cloud.lock` or `gpu-cloud-2.lock`, whichever is free first), for asset-forge's `--lane cloud` jobs.

Several commands, or Blender through the `Blender` MCP:

```
gpu-lock.exe take --thread "<thread name>" --job "<what>" --minutes 30     (in the background)
gpu-lock.exe wait --thread "<thread name>"                                   (returns with LOCKED)
... the GPU work ...
gpu-lock.exe release --thread "<thread name>"
```

`take` keeps running while it holds, so it is started in the background; `wait` returns as soon as it holds. While a thread holds the lane with `take`, its own `run`s go at once.

Other commands: `gpu-lock.exe status` (who holds each lock and who waits, in order), `gpu-lock.exe cancel --thread "<name>"` (leaves the queue for every run and take of the thread and stops its running jobs), `gpu-lock.exe --help`.

Exit codes: the command's own for `run`; 2 for a wrong command line; 3 when `--wait-minutes` ran out; 4 when it was cancelled or released before or during the job; 5 when the time limit stopped it; 6 when another run's lock file took its slot.

## Time limits and crashes

- A holder whose process dies, whatever killed it, frees the lock at once: the next in line takes it within 0.2 s. A `run` also writes its job's process id (`job_pid`) into the lock file: if only `gpu-lock` is killed and its job lives on, the job keeps the lock until it ends, so two jobs never share the card.
- A holder that finds another run's lock file in its slot stops (a `run` stops its job too) and exits 6.
- A `run` under its own thread's `take` ends at that take's time limit.
- After the PC sleeps, a holder's limit moves on by the time asleep.
- A waiter whose process dies leaves the queue.
- A `run` holds for at most `--minutes` (default 15, at most 240). At the limit, its command and everything the command started are stopped (`taskkill /T`) and the lock is released. Pass more minutes for a long job.
- A `take` holds for at most `--minutes` (default 15, at most 240), then lets go by itself.
- A holder touches its lock file every 10 s as its heartbeat. A lock with no heartbeat for 2 minutes (a hung or frozen holder), or 2 minutes past its limit, is taken over by the next in line, and the old holder is told to let go. Just after the PC wakes from sleep a waiter gives the holders 20 s to beat first.
- A lock file made by hand (the old one-line command, without a `ticket:` line) counts until it is 15 minutes old, then the next in line takes it over.
- If a lock file is deleted while it is held (the old `rm -f`), the holder writes it again within 0.2 s.

Every take, release, time limit and takeover is a line in `Dash_Tools\gpu-queue\log.txt`, with the time to the millisecond, so the gap between one job and the next can be read off.

## How it works

- The lock files are where they always were, in `Dash_Tools\`: `gpu.lock` for the PC's card, `gpu-cloud.lock` and `gpu-cloud-2.lock` for the pod. Their first three lines (`thread`, `started`, `job`) are what asset-forge reads; the queue adds `ticket`, `took_unix`, `until`, `until_unix`, `kind` and `pid`.
- Each lane has a queue folder, `Dash_Tools\gpu-queue\local\` and `...\cloud\`. A run that wants the lane writes a ticket there, `<id>.ticket` (who, what, since when), and keeps `<id>.live` locked for as long as its process lives. The id starts with the time it joined in nanoseconds, so the tickets sort in the order they came.
- Every 0.2 s each waiting run looks at the queue. Tickets whose `.live` file nobody holds any more belong to dead processes, and are removed. Only the first waiting ticket may take a free lock file, which it creates with `create_new`, so two runs can never both get it.
- The process that waits is the process that holds: `run` holds while its command runs, `take` until `release`. Nothing is left behind in the background. That matters on Windows, where a process started by another inherits its open handles: a leftover holder started from the Bash tool would keep the tool's output pipe open, and the tool call would never end.
- `release` and `cancel` ask a run to let go by writing `<id>.release`; the run sees it within 0.2 s.
- Std only: file locks are `std::fs::File::try_lock` (Rust 1.89 and later), so the tool needs no `unsafe` and no new dependency.

Never delete the `gpu-queue` folder or a lock file by hand: the tool repairs both, but a lock file deleted at the wrong moment lets a hand-made lock in.

## Tests

`cargo test -p gpu-lock`: 18 unit tests (one is the heartbeat rule), and 15 tests that start real `gpu-lock` processes on a folder of their own (`GPU_LOCK_DIR`): the order of the queue and the hand-over gap (under a second; about 25 to 50 ms measured), holders and waiters whose process dies, a job whose supervisor alone was killed, a hand-made lock young and old, both time limits, `take`, `wait`, `release` and `cancel`, `status`, a command that cannot start, and the cloud lane's two locks held at once. They need no GPU and take about 5 seconds.
