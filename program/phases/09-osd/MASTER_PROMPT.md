# Phase 9: controller-first Liquid Glass OSD (MASTER PROMPT)

You lead Phase 9 of FH6 WheelDash v2, running in your own chat and worktree. The OSD is the in-game control centre. It is full-screen, controller-first, Liquid Glass inspired, and has a **live view** of the dash, so changes show instantly.

## Read first
`ONBOARDING.md`, root `AGENTS.md` / `SKILLS.md`, this folder's `AGENTS.md` / `SKILLS.md`, the handovers of Phases 3, 4 and 5 (and 8 if it is merged), `docs/design/*` (the OSD direction frame), `docs/kit/README.md`, and `crates/fh6-osd/*`. The menu model in `model.rs` is worth keeping; the renderer is not.

## Required work
1. **Design in Penpot first,** on page `09 · OSD` of the master file, extending the Phase 3 direction.
   - **Layout:** full screen at 4K. A live dash view takes about 60% (the real selected cluster, animated), with a Liquid Glass control column. Tabs are switched with LB/RB.
   - Large, glanceable type in the motorsport default font.
   - **Focus states** that read from the couch on a 55" OLED.
   - Iterate with `design:design-critique` until it's excellent.
2. **Settings,** all live-applied and saved to `config.json`:
   - cluster per view (cockpit, hood, chase), with a carousel of live thumbnails
   - colour mode: factory (brand), custom (a palette editor with D-pad hue/sat/value plus presets) or randomise (press to re-roll, lock to keep)
   - font: pack and user fonts
   - logo on or off, and the custom name (on-screen keyboard, D-pad driven)
   - FOV per view, seat height and wheel offset (through the Phase 8 camera patcher, with the "applies on next launch" notice)
   - cluster height, scale and the scrim settings
   - speed and power units, shift % and the redline strobe style
   - stats overlay on or off
   - the button for opening the OSD (remapper; ABXY never allowed)
3. **Live view:**
   - While the OSD is open and Forza is paused, the live dyno drives the selected cluster: real springs, a sweep through the gears, and the redline strobe.
   - If live telemetry is flowing, show live data instead.
   - Use `LiveDynoGenerator`; it exists but is unused.
4. **Open and close:**
   - Use the unified hold-toggle from Phase 4 (default: hold View 400 ms).
   - On open while racing, send ESC with a real `SendInput` so the game pauses (`osd_auto_pause_game`). On close, resume. Never send ESC when not racing.
   - The overlay's visibility rules allow the OSD while paused.
5. **Navigation:**
   - D-pad moves, LB/RB switch tabs, and View or Menu goes back. Never A/B/X/Y.
   - Hold-to-repeat on D-pad for sliders, with acceleration.
   - Every input gets visual feedback on the next frame.
6. **Glass on the GPU:**
   - Kit glass material, blur behind the panel over the live view, bloom on the focus ring.
   - Under 1.5 ms GPU for the whole OSD at 4K.
7. **Tests:**
   - Navigation state machine.
   - Config round-trip (UTF-8 without BOM).
   - ABXY rejection.
   - SendInput behind a trait with a mock.

## Exit criteria
- Workspace check, test and clippy are green.
- The Penpot page is done.
- 4K OSD renders (each tab, plus focus and slider states) are in `mockups/osd/`, with the critique passed.
- The live view works in `--demo`, and the perf numbers are in `docs/perf/osd.md`.
- A manual run with screenshots shows the controller toggle, navigation and live-applied changes.
