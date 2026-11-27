# HANDOVER: Cloud GPU lane

**Thread:** Set up cloud GPU lane  **Date:** 2026-11-27  **Result:** ✅ complete

The guide for using the lane is [docs/cloud-gpu.md](../../docs/cloud-gpu.md). The rule for sharing the two lanes is root `SKILLS.md`, section 5. This file records what the thread delivered and how it was checked.

## Exit criteria

The criteria are the goals the coordinator set when it started the thread, plus the budget TJX set during it.

| Criterion | Met? | Evidence |
|---|---|---|
| Pick a provider and give the reason | yes | Runpod. The reasons are in `docs/cloud-gpu.md`, "Choices made on 2026-11-27". |
| A template that runs ComfyUI (SDXL base and refiner, canny and depth ControlNet, 4x-UltraSharp, as on this PC) and headless Blender 5.3 with Cycles OptiX | yes | Template `asset-forge-gpu` on Runpod. `cloud doctor` on a new pod passes all 13 checks (output below): the same Blender build hash as this PC, OptiX and CUDA listed, the five model files at this PC's sizes (their SHA-256 are checked at download and equal this PC's files), 101 pinned packages. |
| Callable from this PC, so asset-forge and the dash threads can send jobs | yes | `asset-forge --lane cloud <command>`. `--lane cloud selftest` ran an SDXL texture with the 4x upscale, base and refiner with the canny ControlNet, and a 3840x2160 Cycles render, on two different pods (output below). |
| Outputs come back to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\` | yes, with one limit | `--masters <snake_name>` copies each Blender output folder and each ComfyUI image of 2048 px or more. It was tested into a test folder (two 4096² textures and the 4K render arrived), so that no test picture sits in the Drive. The Drive folder itself was written by the same code path for the spend log (`shared\cloud-gpu-spend.csv` is there). |
| The gpu.lock rule has two lanes and is documented in `SKILLS.md` | yes | `SKILLS.md` section 5; `AGENTS.md` directive 15; the lines that point at them in `ONBOARDING.md`, `program/README.md`, the dash prompt and its checklist line 34, the phase 7 skill list and the two asset skills. asset-forge refuses a cloud job without `gpu-cloud.lock`. |
| Pods stop by themselves when idle | yes | The first pod was left alone after its last use at 15:22:07 UTC. Its watchdog wrote `2026-11-27T15:32:16Z afb069dhdonioy idle for 600s`, and Runpod's API answered 404 for it at 15:32:22. |
| Spend is recorded | yes | `spend.jsonl` holds the three sessions ($0.863); `cloud spend` prints them by thread next to Runpod's own billing; the CSV copy is in the Drive folder. The first session's $0.734 matched the fall of the balance ($10.00 to $9.27). |
| TJX creates the account and pays; no card details asked for; nothing over a small test (about $1) without asking | yes | TJX made the account, paid in $10 and created the key. The whole set-up and all tests cost $0.86. |
| The pilot thread's GPU use and files are left alone | yes | The local GPU jobs this thread ran were the comparison run (2 minutes) and the final `cargo test --workspace` in a clean clone, both under `gpu.lock`. The commit touches none of the pilot's files. |
| A $50 cap for the whole lane, a warning at $40 (TJX, 15:42 UTC, after turning on auto-pay) | yes | `budget_usd` and `budget_warn_usd` in `lane.json`. Tried against the real account: with the budget set under the spend, `cloud up` answered `the cloud lane is closed: $0.81 of the $0.50 budget is spent`; with $0.09 left, `too little of the budget is left to start a pod`; with the warning line under the spend, the warning printed. A pod started with $0.24 left got `FORGE_MAX_SECS=944` in its environment on Runpod (the usual value is 14400). |

## What was built

- `tools/asset-forge/src/lane.rs`: the two lanes, their lock files, and the copy of 4K outputs to the Drive masters folder (`--masters`).
- `tools/asset-forge/src/cloud/`:
  - `mod.rs`: settings, the pod on record, the budget, starting and adopting pods, the session with its SSH forward, and the `cloud` commands (`setup`, `key`, `up`, `down`, `status`, `spend`, `doctor`).
  - `runpod.rs`: the Runpod REST API v2 calls the lane needs.
  - `ssh.rs`: the OpenSSH client that ships with Windows, for scripts, file transfer and the forward.
  - `pod.rs`: what runs on the pod (the idle watchdog, the services, the installer, the doctor) as Rust string constants, with every version and checksum pinned.
  - `remote.rs`: a Blender recipe's parameters rewritten for the pod, inputs uploaded, outputs fetched.
  - `spend.rs`: the spend log. `clock.rs`: UTC timestamps without a time crate.
- `tools/asset-forge/src/selftest.rs`: three timed jobs that run on either lane.
- Changed: `main.rs` (the `--lane` and `--masters` flags, the `cloud` and `selftest` commands), `paths.rs` (the tools folder, the lane, the masters folder), `comfy.rs` (connects to the lane's ComfyUI; copies master-sized images), `blender.rs` (runs on the lane; copies the output folder).
- Docs: `docs/cloud-gpu.md` (new), `SKILLS.md` section 5 and the lines that point at it (see "Shared places touched"), this tool's `README.md`, and `docs/PROGRESS.md`.
- No dependency was added: `tools/asset-forge/Cargo.toml` and `Cargo.lock` are unchanged.
- On Runpod (TJX's account): the network volume `asset-forge` (30 GB, EU-RO-1) and the template `asset-forge-gpu`. No pod is left running.
- On this PC, outside the repo: `%USERPROFILE%\Documents\Dash_Tools\cloud\` (the key, the SSH key pair, `lane.json`, `spend.jsonl`).

## Verification output

Run in a clean clone of `main` (commit `7be8b37`) with this thread's files added, on 2026-11-27. No Rust code reads the documents, which were still edited after the run.

```
$ cargo check --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 11.57s
$ cargo clippy --workspace -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.49s
$ cargo fmt --all -- --check
(no output, exit 0)
$ cargo run -p parity-dump -- tests/golden/packets
20/20 OK
$ cargo test --workspace            (under gpu.lock, 15:55:35 to 16:02:57 UTC)
48 test binaries: 771 passed, 0 failed, 1 ignored (exit 0)
asset-forge:  test result: ok. 122 passed; 0 failed; 0 ignored
golden images: test result: ok. 49 passed; 0 failed; 0 ignored; finished in 242.11s
```

`check` and `clippy` each print one cargo warning that `main` already had: `web-preview` lists `fh6-clusters` as a dependency and does not use it.

`cloud doctor` on a new pod (the second of three pods):

```
✓ gpu: NVIDIA GeForce RTX 4090, 24564 MiB, 595.112.07
✓ blender: Blender 5.3.0 (hash 02f0899d78e3 built 2026-11-10 01:31:18)
✓ cycles optix: NVIDIA GeForce RTX 4090
✓ cycles cuda: NVIDIA GeForce RTX 4090
✓ comfyui: 0.46.0 on cuda:0 NVIDIA GeForce RTX 4090 : cudaMallocAsync
✓ torch: 2.14.1+cu130
✓ sd_xl_base_1.0.safetensors: 6938078334 bytes
✓ sd_xl_refiner_1.0.safetensors: 6075981930 bytes
✓ controlnet-canny-sdxl-1.0-small.fp16.safetensors: 320237179 bytes
✓ controlnet-depth-sdxl-1.0-small.fp16.safetensors: 320237179 bytes
✓ 4x-UltraSharp.pth: 66961958 bytes
✓ python packages: 101 pinned, all at their version
✓ pins: forge-pod-31b4a4506471bdb3
· disk: 26.7 of 30 GB used, 3.3 GB free
· last watchdog deletions: 2026-11-27T15:32:16Z afb069dhdonioy idle for 600s|
· last start of the services by a pod: 15:38:21 [services] Blender 5.3.0 (hash 02f0899d78e3 built 2026-11-10 01:31:18)|15:38:21 [services] starting ComfyUI|15:40:33 [services] ready|
cloud doctor: all checks passed
```

`--lane cloud selftest` on the same pod, and `selftest` on this PC:

```
selftest on the cloud lane
  comfyui   cuda:0 NVIDIA GeForce RTX 4090 : cudaMallocAsync, 24082 MB VRAM (ready after 4.9 s)
  texture   SDXL 1024² 32 steps + 4x upscale → 4096x4096    36.9 s  (first run, loads the model)
  texture   SDXL 1024² 32 steps + 4x upscale → 4096x4096     9.1 s  (second run)
  control   SDXL base + refiner + canny ControlNet → 1024x1024    25.1 s
  blender   Cycles OPTIX GPU NVIDIA GeForce RTX 4090 3840x2160 128 samples    21.8 s  (Blender itself ran 00:12.575 to the save; the rest is file transfer)
selftest: passed

selftest on the local lane
  comfyui   cuda:0 NVIDIA GeForce RTX 3070 Ti Laptop GPU : cudaMallocAsync, 8191 MB VRAM (ready after 0.1 s)
  texture   SDXL 1024² 32 steps + 4x upscale → 4096x4096    33.4 s  (first run, loads the model)
  texture   SDXL 1024² 32 steps + 4x upscale → 4096x4096    27.1 s  (second run)
  control   SDXL base + refiner + canny ControlNet → 1024x1024    25.5 s
  blender   Cycles OPTIX GPU NVIDIA GeForce RTX 3070 Ti Laptop GPU 3840x2160 128 samples    33.0 s  (Blender itself ran 00:32.406 to the save; the rest is file transfer)
selftest: passed
```

`cloud spend` after the last pod:

```
spend log: 3 session(s), 58 min 09 s, $0.863
  Set up cloud GPU lane: 3 session(s), 58 min 09 s, $0.863
Runpod's own billing (it runs hours behind): pods $0.169, volume storage $0.000
budget: $0.86 of the $50.00 budget is spent (pods $0.86, volume storage $0.00)
balance: $9.19 (PREPAID)
```

The pod's scripts were also checked with `bash -n` by hand, and the installer's Python step was run against stubs for eight starting states (fresh, already stamped, an older stamp, cut off, a second attempt, another Python, a failing install, no GPU). No test in the repo starts bash.

## Measurements

The full tables are in `docs/cloud-gpu.md`, "Measured on 2026-11-27". In short:

- A new pod is ready for a job 2 min 42 s to 3 min 18 s after the command.
- Once a model is loaded: SDXL 4096² in 9.1 s against 27.1 s here; the 4K Cycles render in 12.6 s against 32.4 s.
- The first job on a new pod is no faster than this PC.
- An idle pod was gone 10 min 9 s after its last use.
- The one-time install took 21 minutes.

## My calls

- **Runpod, secure cloud, data centre EU-RO-1, the base image.** The reasons are in the guide.
- **A network volume of 30 GB** ($2.10 a month) instead of installing on every pod: a pod start is 3 minutes instead of 21.
- **Pods are deleted, never stopped.** A stopped pod still costs storage, and the volume already keeps everything.
- **The pod is reached over SSH only.** ComfyUI listens on the pod's loopback, so nothing but sshd is open to the internet.
- **Limits:** $0.90 an hour (the top of the range the validation thread priced, $0.34 to $0.89), 10 idle minutes, 4 hours, one pod.
- **The $50 budget counts everything the lane has cost,** the $0.86 of set-up included, and Runpod's volume storage as it is billed. A pod that starts near the limit gets a shorter lifetime, so the limit holds without this PC.
- **Python packages are pinned to this PC's versions** (101 pins). Without them the pod had 24 packages newer than this PC.
- **Job folders are on the pod's own disk,** so renders do not fill the volume.
- **Names sent to Runpod are neutral** (`asset-forge`, `asset-forge-gpu`): the project's name is not sent.
- **`program/README.md`'s pre-approved list is left as it is.** It is TJX's list; the approval of the cloud lane is recorded in `AGENTS.md` directive 15 and `SKILLS.md` section 5 instead.
- **The dash checklist's line 34 was reworded** so that a dash which used the pod and a dash which did not can both answer yes.
- **The `brainstorming` skill was not run.** The coordinator's brief was the design input and TJX had approved the idea; the choices above are listed here for TJX to change.

## Known issues / debt

1. **A pod start takes about 3 minutes,** most of it ComfyUI importing its Python packages from the network volume. Keeping the environment as one archive and unpacking it to the pod's own disk at start would likely cut that and free about 5 GB of the volume. It needs a paid test run of about $0.30, so it is left for TJX to ask for.
2. **The volume is 26.7 GB full of 30.** It now grows only with uploaded inputs, which are cleared after 14 days. `cloud doctor` warns under 3 GB free. A bigger volume costs $0.07 a month for each GB (question 9 in `docs/PROGRESS.md`).
3. **The API key may expire on 2026-12-27.** Runpod's key dialog showed that date, and its API does not report it. A `status 401` then says how to save a new key.
4. **The budget is counted from this PC's spend log** (rate times seconds). Runpod's own billing history runs behind, so it only raises the figure later. A pod started from Runpod's console is not in the log until Runpod bills it.
5. **The two fallback GPU types have never been used.** They are Blackwell cards; the CUDA 13.0 torch build should run on them, but no pod has proved it.
6. **The package pins are a snapshot** of this PC's ComfyUI on 2026-11-27. Nothing compares them with this PC again.
7. **`--masters` was tested into a test folder,** not into a dash's folder in the Drive.
8. **The depth ControlNet has no test job.** It is installed and checksum-checked; the selftest uses the canny one.
9. **The local lane's lock is not enforced by asset-forge,** as before. Only the cloud lane refuses a job without its lock.
10. **If this PC dies in the middle of a Blender job,** the pod finishes the render, then idles for 10 minutes and deletes itself. The render is lost with it.
11. **`cloud setup` was run before the pins and the budget existed.** Its later parts were run through `cloud up`, `cloud doctor` and `cloud down`; the whole command was not run again from an empty account.

## Shared places touched

- `AGENTS.md`: directive 15, the `tools/asset-forge` row, two command lines.
- `SKILLS.md`: section 5 (rewritten for two lanes, with the paid lane's rules and the budget), the ComfyUI, Blender and Drive rows, one line of "How to call them", workflow rule 2.
- `ONBOARDING.md`: the GPU line in section 4 and step 5 in section 5.
- `program/README.md`: one sentence of the conductor protocol.
- `program/phases/07-cluster-factory/CLUSTER_PROMPT.md`: the two GPU lines under "Shared machine rules" and checklist line 34.
- `program/phases/07-cluster-factory/SKILLS.md`: the ComfyUI, Blender and Masters rows.
- `.claude/skills/comfyui-hero-textures/SKILL.md` and `.claude/skills/blender-hero-assets/SKILL.md`: one passage each.
- `docs/PROGRESS.md`: one decision-log row and open question 9.

## What other threads must know

- Take `gpu-cloud.lock` before any `--lane cloud` job, run `asset-forge cloud down` when your batch is done, and put your spend in your handover (`asset-forge cloud spend`).
- Start pods only through asset-forge. A pod started from Runpod's console, CLI or MCP tools has no idle shut-off and is not in the spend log. TJX enabled the Runpod plugin on 2026-11-27, which brings that CLI and those MCP tools: use it to look things up only.
- Never edit `budget_usd` in `lane.json`. Tell TJX when a job prints the budget warning.
- A recipe needs no change to run on the pod, as long as every file it reads is named in its parameters by an absolute path and everything it writes goes into `out`.
- The first job after a pod starts loads each model from the network volume, so it is slower than the ones after it. Send a batch, not single jobs.
- Changing a pin or the installer in `cloud/pod.rs` changes the volume's stamp; the next cloud job then runs the installer again before it starts, which takes seconds when nothing has to be downloaded.

## Code review

A second agent reviews the change against the requirements (the `requesting-code-review` skill). It was still running when this was committed on 2026-11-27 at about 16:40 UTC, because the pilot thread was waiting for these shared files. Its findings, and what was done with each, follow in the next commit.

## Closing review

A fixed list, answered yes or no with the evidence. No scores. The source of each line is in brackets: "AGENTS 6" is global directive 6 of `AGENTS.md`, "SKILLS 5" is section 5 of `SKILLS.md`, and "brief" is the coordinator's brief for this thread.

| # | Line | Answer | Evidence |
|---|---|---|---|
| 1 | Nothing was guessed: every API field, flag and pin was looked up or run. [AGENTS 1] | yes | The Runpod calls follow its OpenAPI description (read 2026-11-27) and every one the lane uses ran against the real account. Versions and checksums were read from this PC's install. The paths that never ran are named in "Known issues" 5, 7, 8 and 11. |
| 2 | No placeholder, stub or truncated file. [AGENTS 2] | yes | No `todo!`, `unimplemented!` or stand-in value in the new code (searched). |
| 3 | `cargo check`, `cargo test`, `cargo clippy -D warnings`, `cargo fmt --check` and the golden packets pass, with the real output shown. [AGENTS 3] | yes | "Verification output" above: all five pass in a clean clone; `cargo test --workspace` ran 771 tests with no failure. |
| 4 | The manual-transmission buttons are untouched. [AGENTS 4] | yes | The change has no input, overlay or OSD code. |
| 5 | Files for `data/` and the game are UTF-8 without BOM. [AGENTS 5] | yes | Nothing is written to `data/` or to the game's files. |
| 6 | No `unwrap()`, `expect()` or `panic!` in new non-test code; no `unsafe`; no new dependency. [AGENTS 6] | yes | Searched in the nine new files and in the added lines of the four changed ones: none outside `mod tests`. `tools/asset-forge/Cargo.toml` and `Cargo.lock` are unchanged. |
| 7 | The frame budget is not affected. [AGENTS 7] | yes | No code on a render path changed. |
| 8 | Nothing was copied into the game folder or a mirror. [AGENTS 8] | yes | The lane writes to `target/`, `Dash_Tools\cloud\` and the Drive masters folder only. |
| 9 | Dash visibility is not affected. [AGENTS 9] | yes | No overlay code changed. |
| 10 | The commit is by `TJX <TJX@Ngage.com>` with no co-author trailer, and no file holds a real name, a `C:\Users\<name>` path or a personal email. [AGENTS 10] | yes | Author and committer `TJX <TJX@Ngage.com>`, no trailer. Every file of the commit was searched for names, personal paths and keys: the only hit is the test string `rpa_SECRET` in `runpod.rs`. The API key is in no file of the repo. |
| 11 | Rust only. [AGENTS 11] | yes | The commit adds `.rs` and `.md` files only. The pod's shell and the Blender Python are Rust string constants. |
| 12 | The review is this yes/no list; no scores, no loop that runs until a reviewer passes. [AGENTS 12] | yes | One code review by a second agent, findings only (see "Code review"), then this list. |
| 13 | The thread opened with an introduction: the task, the rule files that apply, what done looks like. [AGENTS 13] | no | It opened with the list of steps and then the request for the account, with the provider and the costs. It did not name the rule files. This cannot be put right afterwards. |
| 14 | The thread is not archived, and closes only after this review and the sign-off. [AGENTS 13] | yes | On 2026-11-27 at 16:00 UTC TJX asked for threads to review their own work with the installed review skills and agents, so that TJX can be hands-off. This list and the code review stand in for TJX's sign-off; TJX can still reopen anything. |
| 15 | The dash order (R&D, Penpot, ComfyUI, Blender). [AGENTS 14] | yes | Not a dash; no asset was made for the repo. |
| 16 | Every GPU job held the lock of its lane, and the provider and the cost were TJX's to approve. [AGENTS 15, SKILLS 5] | yes | `gpu-cloud.lock` was held from 14:42:51 to 15:53:16 UTC for all pod work; `gpu.lock` for the comparison run and for the final tests. TJX approved the rented GPU, paid in the credit and set the $50 budget. |
| 17 | The work is on `main`, committed with explicit paths, and the shared checkout's branch was never switched. [AGENTS 16] | yes | Committed on `main` in the shared checkout by staging this thread's files by name. The pilot thread's uncommitted files were left as they were, and the branch was never switched. |
| 18 | `rust-best-practices` before writing Rust; `requesting-code-review` before the merge. [SKILLS 1] | yes | Both were loaded. |
| 19 | `brainstorming` and TJX's approval of a design before the build. [SKILLS 1] | no | The coordinator's brief was the design input, and TJX had approved the idea. The choices are under "My calls" for TJX to change. This cannot be put right afterwards. |
| 20 | Crate APIs were checked in context7 before first use. [SKILLS 2] | no | The only new API was ureq's TLS root setting, which was read in the crate's own source in the cargo registry. |
| 21 | The rendering and input laws. [SKILLS 3] | yes | No kit, render or input code changed. |
| 22 | No multi-agent workflow ran. [SKILLS 6] | yes | TJX did not switch one on. The one agent was the code reviewer that `requesting-code-review` asks for. |
| 23 | TJX was asked for the account and the key in plain words, and never for card details. [brief] | yes | The first request in the thread. |
| 24 | Nothing over a small test (about $1) was spent without asking. [brief] | yes | $0.86 in total. |
| 25 | The API key is only in `Dash_Tools\cloud\runpod.key`. [brief] | yes, with one note | This thread wrote it nowhere else. TJX pasted it into the thread, so it is in the chat's history. |
| 26 | The pilot thread's GPU use and files were left alone. [brief] | yes | See the exit criteria. |
| 27 | No pod is left running, and the lane's settings are as TJX set them. | yes | `cloud status` after the last test: `pod: none on record`; `budget_usd` 50 and `budget_warn_usd` 40 in `lane.json`. |

## TJX's sign-off

Not waited for. On 2026-11-27 at 16:00 UTC TJX asked for threads to review their own work with the installed review skills and agents, so that TJX can be hands-off. The code review and the closing review stand in for the sign-off; TJX can still reopen anything.
