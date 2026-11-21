# ADR 0001: GPU overlay presentation with wgpu

**Status:** Accepted
**Date:** 2026-11-20
**Deciders:** Phase 4 (Rust Systems lead), approved under TJX's v2 program pre-approval

## Context
FH6 WheelDash draws instrument clusters over Forza Horizon 6. The overlay must:
- be transparent everywhere except the cluster (OLED-off pixels stay `#000000`, so the game shows through)
- stay on top of the game, never take focus and never take mouse input (click-through)
- put a telemetry packet on screen within one display frame ("0 ms" feel), with under 1 ms CPU per frame at 3840x2160
- be drawn on the GPU, because Phase 5 builds bloom, glass, blur and MSDF text that a CPU rasteriser cannot do in budget

The v1 path (tiny-skia into a CPU buffer, then `UpdateLayeredWindow`) never shipped a window, and at 4K it would copy 33 MB per frame from system memory into DWM.

Forza must run **borderless or windowed-fullscreen**. No overlay of any kind (ours, Steam's, Discord's) can draw over exclusive fullscreen, because DWM is bypassed. The configurator (Phase 10) tells users this.

## Decision
Present with **wgpu 30 on DX12 through a DirectComposition swapchain**, on a click-through, topmost, no-activate window.

1. **Window** (`fh6-win::overlay`):
   - style `WS_POPUP`; extended style `WS_EX_NOREDIRECTIONBITMAP | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`
   - `WS_EX_NOREDIRECTIONBITMAP`: DWM allocates no GDI redirection surface; the only content is our composition visual
   - `WS_EX_LAYERED | WS_EX_TRANSPARENT` (with `SetLayeredWindowAttributes(255, LWA_ALPHA)`): hit-testing falls through to the game, so the window is click-through
   - `WS_EX_NOACTIVATE`: clicks never steal focus from Forza; `WS_EX_TOOLWINDOW`: no taskbar or Alt-Tab entry
   - covers the whole monitor Forza is on (primary by default) at native pixels; the process is per-monitor-v2 DPI aware
2. **Swapchain:** `wgpu::Dx12BackendOptions::presentation_system = Dx12SwapchainKind::DxgiFromVisual`.
   - wgpu 30 then creates the DirectComposition device, a target for our HWND and one visual, and calls `CreateSwapChainForComposition` (verified in `wgpu-hal-30.0.2/src/dx12/{mod.rs,dcomp.rs}`).
   - We do not own any DirectComposition objects ourselves. `SurfaceTargetUnsafe::CompositionVisual` stays available if Phase 5+ needs its own visual tree (for example a second visual for the OSD).
3. **Alpha:** `CompositeAlphaMode::PreMultiplied` (reported as supported for visual targets in `wgpu-hal` `adapter.rs`). Transparent is `(0,0,0,0)`.
4. **Format and colour:** the swapchain is `Bgra8Unorm` (not sRGB) and stores **sRGB-encoded, premultiplied** values, `E(c)·a`, because DWM blends 8-bit SDR content in encoded space. Writing linear premultiplied values into an sRGB view would store `E(c·a)` and make every anti-aliased edge and glass pixel too bright. Phase 4's test pipeline encodes in the shader. Phase 5's kit composites in linear premultiplied RGBA on an offscreen `Rgba16Float` target and resolves to `E(c)·a` in its final pass.
5. **Present mode:** `Mailbox` when the surface offers it (on DXGI this is `Present(0, 0)` on a flip-discard composition swapchain, so DWM takes the newest frame at each vsync), otherwise `Fifo`. `desired_maximum_frame_latency = 1`, and wgpu waits on the frame-latency waitable object before acquiring, so the CPU never runs more than one frame ahead.
6. **Frame loop (no queue):**
   - The UDP receive thread decodes the packet, stamps `Instant::now()`, stores it in an `ArcSwap` and unparks the render thread (`std::thread::Thread::unpark`, a futex-style wake with no lock and no queue).
   - The render thread `park_timeout`s until a packet arrives, or for one refresh interval while the overlay is visible or fading, or 50 ms while hidden. It always loads the latest packet; older packets are simply overwritten.
   - Per frame it writes one small instance buffer and records one render pass. Nothing is read back from the GPU.
7. **Visibility:** hiding never waits for a fade. On `IsRaceOn == 0`, a stale stream (over 250 ms) or Forza losing the foreground, the next frame is presented fully transparent (one frame), then the loop idles. Foreground changes arrive through `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` (immediate), backed by a 10 Hz poll.
8. **Measurement:** each frame records receive-to-present (packet `Instant` to the return of `SurfaceTexture::present`) and CPU frame time (wake to present, excluding the latency-waitable wait). `--stats` draws them; `--bench-seconds N` writes `docs/perf/engine.md`.

## Options considered

### A. wgpu DX12 + DirectComposition visual (chosen)
| Dimension | Assessment |
|---|---|
| Complexity | Low: one backend option; wgpu owns the DirectComposition objects |
| Latency | Lowest available on Windows: flip-model composition swapchain, frame latency 1, presented straight to DWM |
| Transparency | True per-pixel premultiplied alpha |
| Phase 5 fit | Full wgpu feature set (compute, MSAA, offscreen targets); the same kit runs in WebGPU for the configurator |

**Pros:** no CPU copy; GPU-resident end to end; the documented wgpu route for transparent windows.
**Cons:** RenderDoc cannot capture composition swapchains (use PIX, or render offscreen for debugging); DX12 only (fine: the game is Windows/DX12).

### B. wgpu (any backend) into an offscreen texture, read back, `UpdateLayeredWindow` (fallback)
| Dimension | Assessment |
|---|---|
| Complexity | Medium: a readback ring plus a GDI DIB section |
| Latency | **Adds about 1 to 2 frames.** The GPU→CPU map waits for the GPU to finish (one frame of pipelining), then `UpdateLayeredWindow` copies 33 MB at 4K into DWM. Estimate (not measured): at roughly 10 GB/s of memory bandwidth the two 33 MB copies alone cost about 6 to 7 ms of CPU per frame at 3840x2160, which breaks the 1 ms CPU budget |
| Transparency | Premultiplied BGRA, works everywhere since Windows 2000 |

**Pros:** works on any GPU and driver, including Vulkan-only or GL paths, and under remote desktop.
**Cons:** a per-frame CPU readback and copy, which ADR scope forbids without justification; only used if DirectComposition surface creation fails. Kept as the documented fallback; not implemented in Phase 4 because every DX12-capable Windows 10/11 machine (the game's own requirement) supports option A.

### C. `CreateSwapChainForHwnd` (wgpu default `DxgiFromHwnd`)
Opaque only: wgpu reports `CompositeAlphaMode::Opaque` alone for HWND swapchains. Rejected, because it would paint black over the game.

### D. Vulkan or GL with a layered window
Vulkan on Windows has no premultiplied-alpha presentation to DWM; GL needs WGL hacks plus the same readback as B. Rejected.

## Trade-off analysis
Option A is the only route that is both transparent and free of a CPU round-trip. Its one real cost is weaker frame-capture tooling, which offscreen rendering (already required for tests and `preview-export`) covers. Option B is kept as a written fallback for machines where `create_surface` fails; its latency penalty (1 to 2 frames, an estimated several ms of CPU at 4K) is exactly what the "0 ms" goal rules out, so it must never be the default.

The render path has no queue, no mutex and no GPU readback:
- `ArcSwap::load_full` (one atomic increment) replaces the old `Mutex<FH6TelemetryData>`.
- `unpark` replaces polling sleeps (the old UDP loop slept 10 ms on `WouldBlock`, adding up to 10 ms of latency).
- The config hot-reload and XInput threads publish through `ArcSwap` and atomics too.

## Consequences
- Easier: Phase 5 gets a device, a queue, a premultiplied surface and an offscreen target to build on; the same pipeline renders in tests and the bench without a window.
- Harder: debugging presentation needs PIX instead of RenderDoc.
- Users must run Forza borderless or windowed-fullscreen (configurator checklist, Phase 10).
- Revisit if wgpu changes `Dx12SwapchainKind`, or if HDR output is wanted (`Rgba16Float` + scRGB on the composition swapchain).

## Implementation notes (added after the Phase 4 build)
- **One `windows` version.** `wgpu-hal` 30 uses `windows` 0.62, and `gpu-allocator` accepts 0.53 to 0.62. With the workspace on 0.58, Cargo unified `gpu-allocator` onto 0.58 and `wgpu-hal` failed to compile. The workspace therefore moved to `windows` 0.62.
- **Window lifetime.** The surface is created from `fh6_win::OverlayHandle`, a `Send + Sync` raw-window-handle that shares a token with the `OverlayWindow`.
  - While any handle exists, dropping the window leaves the `HWND` alive instead of destroying it under a live swapchain.
  - The app joins the render thread (which drops the surface and handle) before it drops the window.
- **Outside `WM_CLOSE`.** A `WM_CLOSE` sent to the overlay (for example by `taskkill`) posts `WM_QUIT` instead of reaching `DefWindowProc`, so the normal shutdown order runs.
- **Loopback by default.** Telemetry binds `127.0.0.1`, which is what Forza's Data Out targets, and needs no firewall rule. `UdpListenAllInterfaces: true` in `config.json` listens on every interface, for a second PC.
- **Measured** (`docs/perf/engine.md`, 3840x2160 at 120 Hz, RTX 3070 Ti Laptop): receive-to-present well under one frame, CPU per frame around the 1 ms budget. The `present` call and the queue submit are the largest parts.

## Action items
1. [x] `crates/fh6-gpu`: context, overlay surface, offscreen target, quad pipeline.
2. [x] `fh6-win::overlay`: the window above; `fh6-win::foreground`: WinEvent hook plus 10 Hz poll.
3. [x] Render loop with unpark-on-packet and latency stats; `docs/perf/engine.md` from `--bench-seconds`.
4. [ ] Phase 5: linear `Rgba16Float` composition and the `E(c)·a` resolve pass.
5. [ ] Implement fallback B only if a user report shows `create_surface` failing.
6. [ ] Phase 5: cut per-frame CPU further: a persistent mapped staging ring instead of `Queue::write_buffer`, and one encoder per frame shared by every cluster layer.
