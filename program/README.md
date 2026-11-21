# FH6 WheelDash v2: build program

Every phase runs in its own chat and git worktree, started from `program/phases/<phase>/MASTER_PROMPT.md`. A conductor chat (TJX's main chat) runs the chain:
1. It starts phases once their dependencies are merged.
2. It verifies each handover.
3. It merges the phase branch into `main` and pushes to `origin` as TJX.
4. It starts whatever is unblocked.

## Roadmap

| Phase | Name | Depends on | Runs alongside | Effort | Status |
|---|---|---|---|---|---|
| 1 | Clean Rust repo | none | none | n/a | ✅ done |
| 2 | Program kit (this folder) | 1 | none | n/a | ✅ done |
| 3 | Design system & art direction | 2 | 4 | xhigh | ◐ running |
| 4 | Engine: a working exe | 2 | 3 | xhigh | ✅ merged 2026-11-21 |
| 5 | GPU render kit (wgpu) | 4 | 3, 6 | xhigh | ◐ running |
| 6 | Assets, logos & fonts | 3 | 5 | high | ☐ |
| 7 | Cluster factory: setup chat, then one chat per cluster | 3, 5, 6 | 8, 9 | setup high; pattern-setters (★) xhigh; other clusters high | ☐ |
| 8 | Cockpit fit: dash blocker, camera, seat, FOV | 4, 5 | 7, 9 | high | ☐ |
| 9 | Liquid Glass OSD | 5, 3 | 7, 8 | high | ☐ |
| 10 | Configurator (`index.html`) & release | 7 (pattern-setters at least), 8, 9 | the rest of 7 | high | ☐ |
| 11 | Showcase site on Vercel (optional) | 10 | none | medium | ☐ |

```
2 ─┬─> 3 ──────────┬─> 6 ─┐
   └─> 4 ─> 5 ─────┼──────┼─> 7 (setup, then cluster chats ×N, 3 at a time) ─┐
                   ├──────┴─> 9 ───────────────────────────────────────────────┼─> 10 ─> 11
                   └─> 8 ──────────────────────────────────────────────────────┘
```

## Conductor protocol
1. **Start a phase** with a new chat (in a worktree) whose prompt is: *"You are the Phase N chat. Read `ONBOARDING.md`, then follow `program/phases/<phase>/MASTER_PROMPT.md` exactly."* Cluster chats get the extra lines that `program/phases/07-cluster-factory/CLUSTER_PROMPT.md` asks for.
2. **When a chat ends,** check its handover against the phase exit criteria:
   - `cargo check --workspace` and `cargo test --workspace` (known reds only, as listed in `docs/PROGRESS.md`)
   - the required artefacts exist
   - screenshots or golden renders are present for visual work
3. **If it passes:** merge the branch into `main` (fast-forward or a merge commit as TJX, no co-author trailer), push to `origin`, and tick the roadmap above.
4. **If it fails:** send the chat its concrete gaps and have it continue. After 2 failed rounds, triage like an incident (the `engineering:incident-response` skill): record the cause in the handover, then re-scope or split the phase.
5. **Next:** start every phase whose dependencies are now merged. The cluster factory runs at most 3 cluster chats at once.
6. **If the chain stops by itself,** TJX types `continue` in the conductor chat.

## Pre-approved (TJX, 2026-11-20)
- **Downloads:**
  - crates through cargo (wgpu, resvg and similar)
  - the `wasm32-unknown-unknown` target and `wasm-bindgen-cli`
  - OFL fonts from github.com/google/fonts
  - car logos from Wikimedia Commons and manufacturers' official press kits, shipped in the repo (TJX accepts the trademark risk)
  - CC0 textures from ambientCG and Poly Haven
  - ComfyUI models: SDXL base 1.0 (`stabilityai/stable-diffusion-xl-base-1.0`, about 6.9 GB) and 4x-UltraSharp (about 67 MB)
- **Git:** merging verified phase branches into `main` and pushing to `origin` (public: T-J-X/Forza-Revampled-Telemetry). A GitHub Release in Phase 10.
- **Never without TJX's GO:** copying to the game folder or mirrors, and editing live game files.

## Targets every phase protects
- **"0 ms" feel:**
  - No packet queue; telemetry reaches the screen on the next vsync, within 1 display frame.
  - Under 1 ms CPU per frame and under 0.5 ms GPU per cluster at 3840x2160.
  - The packet-to-present time is measured and logged.
- **Visibility:** hidden within 1 frame when `IsRaceOn == 0` (menus, pause) or when Forza isn't the foreground window.
- **OLED:** off pixels are `#000000` (transparent in the overlay). Never fill the whole screen.
- **Fonts:** a motorsport type default; users can add their own fonts.

## Reasoning effort (Opus)
The conductor sets each chat's effort when it starts it (see the roadmap's Effort column). The rule: **xhigh** where a wrong call is expensive or the work sets the pattern for everything after it (design language, GPU engine, render kit, pattern-setter clusters); **high** for substantial work that follows an established pattern; **medium** for mostly mechanical or low-risk work (showcase site, simple fix-up rounds). A chat that is clearly struggling at its level gets raised one step on its retry round.
