# The cloud GPU lane

A rented 24 GB card that runs the same ComfyUI and the same Blender as this PC. Every asset-forge ComfyUI and Blender job runs there, two runs at a time on one pod, and the local RTX 3070 Ti keeps the kit's work: `preview-export`, the goldens and the live bench (TJX, 2026-11-27: cloud first). TJX approved the lane on 2026-11-27. The rule for sharing the two lanes is in [SKILLS.md](../SKILLS.md), section 5.

## In one minute

```
# queue for a cloud lock with gpu-lock (SKILLS.md, section 5): it holds gpu-cloud.lock or gpu-cloud-2.lock
# while the job runs. gpu-lock.exe is "$USERPROFILE/Documents/Dash_Tools/bin/gpu-lock.exe"; start it in the background.
gpu-lock.exe run --lane cloud --thread "<thread name>" --job "<set>" -- cargo run --release -p asset-forge -- --lane cloud build --set <snake_name>
gpu-lock.exe run --lane cloud --thread "<thread name>" --job "<set>" -- cargo run --release -p asset-forge -- --lane cloud --masters <snake_name> build --set <snake_name>
cargo run --release -p asset-forge -- cloud down      # when your batch is done (it needs no lock)
```

- `--lane cloud` sends the run's ComfyUI and Blender work to the pod. Everything else stays on this PC, and the results land in the same folders as a local run. A recipe needs no change.
- `--masters <snake_name>` also copies the files in each Blender job's output folder (not its sub-folders) and each ComfyUI image of 2048 px or more to `G:\My Drive\FH6 WheelDash v2 - 4K Masters\<snake_name>\`. It works on both lanes.
- The pod starts by itself when a job needs it, and is ready about 3 minutes later (measured below). It is deleted by `cloud down` when no other run uses it, or by itself 10 minutes after the last run ends.
- Two runs can use the pod at once, for example two dash threads: the second joins the pod the first one started. Only one run at a time starts or installs a pod; a second one waits for it.
- Only `asset-forge` jobs can run there. `preview-export`, the golden tests and everything else that draws with the kit need the local card.

## What it costs

| Item | Price | When |
|---|---|---|
| RTX 4090 pod (first choice) | $0.89 an hour, billed per second | only while a pod is up |
| RTX PRO 4500 Blackwell, 32 GB (fallback) | $0.72 an hour | when no 4090 is free |
| RTX PRO 4000 Blackwell, 24 GB (fallback) | $0.57 an hour | when neither is free |
| 400 GB network volume | $28 a month | always, until it is deleted, also while the lane is closed; shown beside the budget, not counted |

The 4090's price is what the three test pods were billed. The other prices are Runpod's secure-cloud list prices on 2026-11-27, and neither fallback card has been used yet.

TJX turned on auto-pay on 2026-11-27. The credit balance refills by itself, so it no longer limits what can be spent. The budget below does.

## The limits

Five things keep the cost down, and none of them depends on a thread remembering:

1. **A budget for the whole lane: $50 of GPU time** (TJX, 2026-11-27: "all clusters should cost under $50"). It is `budget_usd` in `lane.json`.
   - From $40 (`budget_warn_usd`) every cloud job prints how much is spent.
   - At $50 no pod starts. A pod that is still up then is deleted by the next cloud job, `cloud up` or `cloud down`.
   - A pod that starts close to $50 gets only the lifetime the rest of the budget pays for at the price cap. Its own watchdog enforces that, from a deadline set when the pod is created. So pod time cannot take the total past $50 even if this PC is off. When the rest pays for less than 10 minutes, no pod starts.
   - What counts: pod time, that is the spend log plus the pod that is up now, or Runpod's own billing for pods when that is higher. The volume's storage is shown beside it and not counted: TJX chose that on 2026-11-27, after growing the volume to 400 GB (`budget_counts_storage` in `lane.json`).
   - The volume keeps costing $28 a month, also after the lane closes, until it is deleted.
   - If a line of the spend log is damaged, the budget cannot be counted, and no pod starts until the line is fixed or removed.
   - Only TJX raises the budget. A thread never edits `budget_usd`.
   - TJX will not raise it (2026-11-28: "when we run out we will work here instead only"). Once it is spent, the lane stays closed and every job runs on the local lane: queue for `gpu.lock` with `gpu-lock run` and run the same command without `--lane cloud`.
2. **No pod over the price cap.** `max_usd_per_hour` in `lane.json` is $0.90. `cloud setup` lists only GPU types at or under it, and a pod that comes back dearer is deleted at once and booked. A pod whose answer gives no price is booked at the cap.
3. **Idle shut-off.** A watchdog inside the pod deletes it after `idle_minutes` (10) without use. "Use" is a command from this PC, a beat from an open asset-forge run, a running Blender, or a busy GPU. The run sends a beat every 15 seconds, and the pod stops counting them a minute after the last one. So a PC that sleeps, crashes or loses its network frees the pod about 11 minutes later. The watchdog is part of the pod's start command, so it works with this PC off.
4. **Lifetime cap.** The same watchdog deletes the pod at `max_hours` (4), whatever it is doing, or earlier when the budget is nearly used up. A second loop that needs nothing but the clock enforces this too, in case the first one hangs. Nothing in the watchdog waits without a time limit. asset-forge replaces a pod that is within 30 minutes of the cap before it starts a job on it.
5. **One pod, shared.** Up to two runs on this PC use the same pod at once; each marks itself in `runs\` for as long as it uses the pod. Only one run at a time starts, installs or replaces a pod (`start.lock`), and the others wait for it. A pod another run uses is never deleted or replaced under it. A run adopts a pod of its own that it finds running with no record, and deletes and books any other one.

All five hold only for pods that asset-forge starts. The Runpod plugin on TJX's account (enabled 2026-11-27) can start pods too, through Runpod's CLI and MCP tools. Such a pod has no watchdog and no price cap, and it reaches the budget only when Runpod bills it, hours later. So the plugin is for looking things up, and pods are started through asset-forge only (SKILLS.md, section 5).

## Commands

| Command | What it does |
|---|---|
| `asset-forge cloud setup [--dry-run]` | One-time set-up: picks the GPU types and the data centre from live stock, creates the volume and the template, installs everything on a first pod, checks it and deletes the pod. Safe to run again. `--dry-run` prints the choices and creates nothing. |
| `asset-forge cloud up` | Starts the pod now and waits until it is ready. |
| `asset-forge cloud down` | Deletes the pod, and any other pod of the lane, and writes each session to the spend log. While another run still uses the pod, it leaves the pod up; the pod then deletes itself 10 minutes after the last run ends. |
| `asset-forge cloud status` | The balance, the limits, the pod that is up with its cost so far, and how much of the budget is spent. A pod that has ended is booked first. |
| `asset-forge cloud spend` | The spend log by thread, Runpod's own billing, the budget and the balance. A pod that has ended is booked first. |
| `asset-forge cloud doctor` | Checks the pod like `doctor` checks this PC: GPU, Blender, Cycles devices, ComfyUI, torch, every model file, every pinned Python package and the disk. |
| `asset-forge cloud key` | Asks for a new Runpod API key (paste it and press Enter) and saves it once Runpod has accepted it. The key never goes on the command line, where the shell's history and `cargo run` would keep it. |
| `asset-forge [--lane cloud] selftest` | Three fixed jobs with timings, on either lane: an SDXL texture with the 4x upscale, base and refiner with the canny ControlNet, and a 3840x2160 glass render. |

`up`, `setup`, `doctor` and every `--lane cloud` job need a cloud lock file (`gpu-cloud.lock` or `gpu-cloud-2.lock`): run them through `gpu-lock run --lane cloud` (SKILLS.md, section 5). `down`, `status`, `spend` and `key` do not.

## Measured on 2026-11-27

Starting and stopping:

| What | Time |
|---|---|
| The one-time install on the empty volume | 21 min (13 of them writing the Python environment) |
| A new pod, from the command to ready for a job | 2 min 42 s and 3 min 18 s (two pods); 5 min 21 s for the first pod after the installer changed |
| Of that: until the container runs | about 25 s |
| Of that: ComfyUI's own start from the network volume | 2 min 12 s to 3 min 31 s |
| Connecting to a pod that is already up | 4 to 5 s |
| An idle pod deleting itself | 10 min 9 s after its last use |

The same three jobs on both lanes (`asset-forge selftest`). The cloud column gives the fastest and the slowest of three pods; pods on different hosts differ.

| Job | Local, RTX 3070 Ti 8 GB | Cloud, RTX 4090 24 GB |
|---|---|---|
| SDXL 1024², 32 steps, 4x upscale to 4096²: first run, loads the checkpoint | 33.4 s | 36.9 to 51.3 s |
| The same, second run | 27.1 s | 9.1 to 11.8 s |
| SDXL base and refiner with the canny ControlNet, 1024²: first use of both models | 25.5 s | 25.1 to 31.3 s |
| Cycles on OptiX, 3840x2160, 128 samples, glass plate: Blender's own clock at the save | 32.4 s | 12.6 to 17.0 s |
| The same, with the 4K PNG copied back to this PC | 33.0 s | 21.8 to 27.1 s |

What the numbers say:

- Once a model is loaded, the pod is 2.3 to 3 times as fast on SDXL, and 1.9 to 2.6 times as fast in Cycles.
- The first job on a new pod is no faster than this PC, because each model is read from the network volume first.
- So the lane pays off for batches and for heavy renders. One small job is quicker on the local card, which needs no 3-minute start.
- The other two gains are not in the table: the lane runs at the same time as the local card, and it has 24 GB where the local card has 8.

## How it works

- **One pod, made on demand.** `asset-forge` asks Runpod's REST API for a pod from the template `asset-forge-gpu`, on the first GPU type in `lane.json` that has room. The pod mounts the network volume at `/workspace`.
- **The volume holds everything that takes long to install:** Blender 5.3.0, ComfyUI v0.46.0 with its Python 3.13 environment (torch 2.14.1, the CUDA 13.0 build, as on this PC), and the five model files. A new pod only starts ComfyUI.
- **What "the same as this PC" means, exactly.**
  - Blender and the five models are downloads checked against a SHA-256. The model checksums equal those of the files in this PC's ComfyUI install, and Blender reports the same build hash.
  - ComfyUI is the same commit. torch, torchvision and torchaudio are the same versions and the same CUDA 13.0 build.
  - ComfyUI's other Python packages are pinned by version in `cloud/pod.rs`: 79 that this PC also has, at this PC's versions as read on 2026-11-27 (`PINS`), and 22 that only a Linux install has, at the versions of the first pod (`PINS_LINUX`). ComfyUI's own `requirements.txt` leaves most of them open, so without the pins a disk installed on another day gets other versions. The first pod did: 24 packages came out newer than this PC's, `kornia` and `transformers` among them.
  - `cloud doctor` compares every package on the pod with the pins.
  - The pins are a snapshot. If this PC's ComfyUI is updated, the two drift apart until the pins are read again; nothing checks this PC's packages.
  - The GPU is a different card, so the same recipe and seed can still differ in small numbers between the lanes.
- **Nothing on the pod is public.** The only open port is SSH, with a key that never leaves this PC. ComfyUI listens on the pod's loopback and is reached through an SSH forward on `127.0.0.1:8189` (the local ComfyUI keeps 8188). The same SSH connection carries the run's beats.
- **ComfyUI jobs** go through that forward and use the same HTTP calls as the local lane.
- **Blender jobs** are rewritten for the pod: the recipe's `out` folder becomes a folder on the pod, every input file the parameters name is uploaded, Blender runs with the same script and flags, and the output folder is copied back into the local `out`.
  - Job folders are on the pod's own disk, one folder for each run (`/root/forge-jobs/run<process id>/<job>`), and go with the pod.
  - An uploaded input is kept on the volume for 14 days under a name made from its content, so a texture that many renders use is sent once.
  - A recipe works unchanged as long as every file it reads is named in its parameters by an absolute path, and everything it writes goes into `out`. A parameter that names a path on this PC with nothing behind it stops the job here, so no path of this PC is sent to the pod.
- **Two runs share the pod.** Each run has a forward port of its own (8189, or another free one), its own job folders, and its tag in the names of the images it uploads to ComfyUI. Their ComfyUI jobs take turns in ComfyUI's one queue, and their Blender renders run side by side. ComfyUI leaves 6 GB of the card free (`--reserve-vram 6`), and does not unload its models after each job, so the other run need not load them again. A Blender job waits, for up to 10 minutes, until 6 GB of the card is free. The files ComfyUI writes stay on the pod's own disk.
- **The pod's scripts are Rust string constants** in `tools/asset-forge/src/cloud/pod.rs`, like the Blender recipes. The repo holds no shell files.
- **A changed pin updates the volume by itself.** The volume carries a stamp of the installer's text. When a pin or the installer changes, the next cloud job runs the installer first; it replaces only what changed. A pod starts ComfyUI by itself only on a disk with the current stamp, and the installer and the pod's own start never run at the same time.

## Where things live

Everything personal is outside the repo, in `%USERPROFILE%\Documents\Dash_Tools\cloud\`:

| File | What it is |
|---|---|
| `runpod.key` | The API key. `RUNPOD_API_KEY` in the environment takes precedence. Never copy it into the repo, a message or a log. |
| `id_ed25519`, `id_ed25519.pub` | The SSH key pair for pods, made by `cloud setup`. |
| `lane.json` | The settings: data centre, volume, GPU types in order, price cap, idle minutes, lifetime hours, image, and the budget. Plain JSON. The budget is TJX's to change. |
| `state.json` | The pod that is up, until its session is booked. |
| `start.lock` | Held by the run that is starting, installing or replacing the pod; the others wait for it. Windows releases it when that run ends, also after a crash. |
| `runs\<process id>.lock` | One for each run that uses the pod, held while it does. A file whose run has ended is removed by the next count. |
| `spend.jsonl` | One line per finished session: thread, job, GPU, start, end, seconds and dollars. |

The spend log is also written as `shared\cloud-gpu-spend.csv` in the Drive masters folder, so it can be read from another machine.

Two things about the numbers in the spend log:

- A session's dollars are the hourly rate times the seconds the pod was up. For the first pod that gave $0.734, and the balance fell by the same amount.
- A pod that ended without `cloud down` is booked up to its last use plus the idle time, or at Runpod's bill for it when that is higher. Its last use is saved before and after every job, so a run that was killed is counted too.
- A pod is booked once, however many commands find it.
- Runpod's own billing history runs behind. `cloud spend` prints it next to the log. It showed $0.00 for more than an hour after the first pod started, and $0.17 of the $0.86 after 70 minutes.

## Choices made on 2026-11-27 (my call, for TJX to change)

- **Runpod** over Vast.ai: fixed prices, per-second billing, and an API that lets asset-forge start and delete pods itself. Vast.ai is cheaper per hour but is a marketplace of private hosts.
- **Secure cloud**, not community cloud: community pods are not guaranteed a public address, and the lane needs one for SSH. Network volumes are secure-cloud only as well.
- **Data centre EU-RO-1 (Romania):** TJX is in the UK, and on that day it was the only European site with network volumes that had the RTX 4090 and two fallback cards in stock. The volume ties the lane to this data centre.
- **Runpod's base image** (`runpod/base:2.1.1-ubuntu2404`, 0.7 GB) rather than its PyTorch image (14 GB): the Python environment is on the volume, so the image only needs sshd, and a small image starts faster on a host that has not cached it.
- **Names sent to Runpod are neutral** (`asset-forge`, `asset-forge-gpu`), and so is the client name in requests.
- **The budget counts every pod session**, the set-up included, not only the dash threads' jobs.
- **One shared pod rather than a second pod** for two runs at once: a second pod would add its own start (about 3 minutes) and idle tail (10 minutes, about $0.15) to every batch, and one 24 GB card has room for a ComfyUI job and a Blender render side by side. A second pod can follow if two heavy renders keep waiting for each other.

TJX's own choices on 2026-11-27: the volume was created at 30 GB, grown to 100 GB at TJX's request ("100"), and then grown to 400 GB by TJX in Runpod's console. The budget counts GPU time only, and storage is shown beside it. Every ComfyUI and Blender job runs on the pod ("we have 400gb and £50 in the cloud on a 4090").

## When something goes wrong

| What you see | What it means | What to do |
|---|---|---|
| `the cloud lane is not locked` | No cloud lock file is held. | Run the job through `gpu-lock run --lane cloud` (SKILLS.md, section 5). |
| `the cloud lane is closed: ... of the ... budget is spent` | The budget is used up. | Run this job and every later one on the local lane (queue for `gpu.lock` with `gpu-lock run`, and drop `--lane cloud`), and say in your thread that the lane is closed. TJX keeps the budget as it is (2026-11-28). |
| `too little of the budget is left to start a pod` | The rest of the budget pays for less than 10 minutes. | The same. |
| `no pod could be started in EU-RO-1` | None of the GPU types has room right now. | Run the job on the local lane, or try again later. |
| `the Runpod balance is too low` | The credit ran out and auto-pay did not refill it. | Tell TJX. |
| `waiting for another run to finish starting the pod` | Another run is starting, installing or replacing the pod. | Nothing: the run goes on once the pod is ready, and shares it. |
| `waiting for 6000 MiB of free GPU memory` | The other run's job is using the card. | Nothing: the render starts once there is room, after 10 minutes at the latest. |
| `the pod ends in ... and another run is using it` | The shared pod is minutes from its lifetime cap. | Run the job again once that pod has ended; the next run starts a new one. |
| `... has ... line(s) that are not a session` | The spend log is damaged, so the budget cannot be counted. | Fix or remove the named lines in `spend.jsonl`. |
| `cloud doctor` fails on `pins` | The volume was installed by an older asset-forge. | Run any cloud job: it brings the volume up to date first. |
| `cloud doctor` fails on `python packages` | A package on the pod is not at its pinned version, although the installer's stamp is current: something changed the environment by hand. | Tell TJX. The line names the packages. The installer brings them back the next time any pin in `cloud/pod.rs` changes. |
| `cloud doctor` says the disk is low | Under 3 GB of the volume is free. | Tell TJX: a bigger volume costs $0.07 a month for each GB, and Runpod can grow a volume but never shrink it. |
| `status 401` from Runpod | The API key is wrong or has expired. Runpod's key dialog defaults to a 30-day expiry, so the key made on 2026-11-27 may stop on 2026-12-27. | TJX makes a new key on Runpod's Credentials page; save it with `asset-forge cloud key`, which asks for it. |
| A pod is up and nobody is using it | A run was interrupted. | `asset-forge cloud down`. Without it, the pod deletes itself after 10 idle minutes. |

If the balance reaches $0, Runpod stops the pods and keeps the volume. If it stays at $0, Runpod may delete the volume in the end. Nothing of ours is lost with it: `asset-forge cloud setup` builds it again in about 21 minutes for about $0.35.

To remove the lane completely: `asset-forge cloud down`, then delete the volume `asset-forge` and the template `asset-forge-gpu` in Runpod's console. After that nothing is billed.
