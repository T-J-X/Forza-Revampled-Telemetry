# Phase 4 agents

| Lead | Mandate in this phase |
|---|---|
| **Agent 2 · Rust Systems & Graphics** (lead) | Owns the ADR, the `fh6-gpu` crate, the window and message loop, UDP-to-render wiring, XInput, the visibility controller, config and the app shell. Puts every `unsafe` block in `fh6-win` / `fh6-input` / `fh6-gpu`, each with a `// SAFETY:` comment. Allocates nothing per frame. |
| **Agent 1 · HMI/UX** | Defines the visibility rules (when to hide, fade timing) and keeps the ABXY lock and controller ergonomics. |
| **Agent 4 · QA & Release** | Owns the latency and frame-time measurements, the clippy-green bar, the fixed launcher test, the `bind_car_brand` safety test and the manual run checklist with screenshots. |

Latency is the feature here. Any design that adds a queue, a CPU readback per frame or a mutex on the render path needs a written justification in the ADR.
