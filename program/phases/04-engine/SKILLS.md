# Phase 4 skills

| When | Skills / tools |
|---|---|
| Start | `rust-best-practices` (before writing any `.rs`), `engineering:architecture` (ADR 0001) |
| Threads, ArcSwap, channels, Win32 handles | `rust-patterns` |
| Tests (config loading, launcher, `bind_car_brand`, hold state) | `rust-testing` |
| Something misbehaves at runtime | `engineering:debug` |
| Before the handover | `rust-review` on `fh6-gpu`, `fh6-win`, `fh6-app`; then `/code-review`, `/simplify` |
| Docs | `engineering:documentation` (ADR, `docs/perf/engine.md`) |

MCP: **context7**. Before first use, look up `wgpu` (surface creation on Windows / DX12, composite alpha modes, present modes), `windows` 0.58+ (Win32 UI WindowsAndMessaging, DirectComposition, Shell_NotifyIcon, ToolHelp, LibraryLoader) and `raw-window-handle`.
