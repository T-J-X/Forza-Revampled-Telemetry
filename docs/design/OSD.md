# OSD: the shipped design (Phase 9)

The controller-first Liquid Glass OSD, as built in `crates/fh6-osd` and drawn with the render kit. It extends the Phase 3 direction ([OSD_DIRECTION.md](OSD_DIRECTION.md)). The board with every state is [boards/osd/index.html](boards/osd/index.html); the images in [osd/](osd/) are real 4K kit renders (960×540 downsamples) from `cargo run --release -p preview-export` (full size in `mockups/osd/`).

Design file: the OSD page is `09 · OSD` in the Penpot master file ([PENPOT.md](PENPOT.md), board IDs there). The HTML board stays alongside as the rendered reference.

## Layout
- **Live zone (left, about 60%).** The real selected cluster at its real height and scale, gliding from its normal anchor to the live-zone centre (`(2380 + extra width) / 2`) when the OSD opens. Header: "LIVE VIEW", the cluster name, and the data source ("Game paused · live dyno" or "Live telemetry") with the edited view. Under the header, a **focus echo**: the focused row's label and value in 110 px type, a slider mirror, the row's help line. Camera rows add a side-view schematic (eye at seat height, FOV wedge, wheel); cluster height adds a dashed default-height line.
- **Panel (right).** Glass at x 2380–3700, y 140–2020, radius 56, right-aligned on wider screens. Five tabs: Cluster · Colour · Type & logo · Fit & camera · Controls. Rows are 116 px (sliders 164, carousel 340, info 124); the focused row grows to 148 px (sliders 208) when it has a help line, so the help sits inside the focus ring. Rows scroll on a spring and fade at the viewport edges. Hint bar and the gearbox line at the bottom.
- **Dim.** The game is dimmed 62% through the kit underlay. The direction asked for about 45% plus a blur; the overlay cannot blur a game it does not capture, so it dims harder and adds feathered black pools behind the echo (45%) and the live cluster (35%).

## Rows
| Tab | Rows |
|---|---|
| Cluster | Edit view (Cockpit / Hood / Chase; the live view follows), Cluster (carousel of 5 live thumbnails, all 29 themes), Speed units, Power units, Shift point (80–100%), Limiter light (10 Hz strobe / Gentle pulse) |
| Colour | Colour mode (Factory / Custom / Random). Factory: brand swatches. Custom: Colour (primary, secondary, accent, shift light), Hue on a spectrum track, Saturation, Brightness, Preset (6). Random: Re-roll, Same palette on every car. Then Glass refraction |
| Type & logo | Font (pack default plus every loaded family), Logo, Custom name (on-screen keyboard, 14 characters) |
| Fit & camera | Cockpit / Hood / Chase FOV (40–110°; Cockpit edits the wheel-less FOV the patcher writes), Seat height (−100..+50 mm), Wheel offset (−80..+20 mm), Camera patch (the `CameraPatchAuto` switch), Cluster height (±240 px, per view: the view being edited), Cluster scale (70–140%), the nine Phase 8 dash-blocker rows (opacity, width, height, crown arch, corner softness, left/right, up/down, edge feather, steering-column gap), Resync when the patcher offers it. Camera rows say "Applies on next launch (camera file)" when the patch is on, "Camera patch is off: saved in config only" when it is off, and that the camera files were not found when the patcher has no install |
| Controls | OSD button (remapper), Hold time (200–1000 ms), Pause game when open, Stats overlay |

## Type, colour and focus
- Labels and values 30 cap; help, hints and status 26 cap; keycaps 22; section headings 18 caps at +24% tracking. Secondary text uses `inkMuted` (no `inkLabel` text in the OSD).
- One left text edge (2488) and one value edge (3552); Choice rows always show both chevrons.
- Focus: a white 0.07 lift, a 3 px accent rim in a bloom layer and a σ 14 glow. The accent is the palette accent unless it sits within 25° of hue of the warning red; then the primary, then sapphire `#38BDF8`.
- Panel glass: tint alpha 64, frost 0.9, backdrop σ 28, crown sheen 28/255 that ends above the tabs, OLED black 0.62 underneath.

## Input
D-pad moves and adjusts (repeat after 280 ms, every 65 ms, every 25 ms after 900 ms; sliders take ×4 steps after 1.2 s and ×10 after 2.5 s), LB/RB switch tabs, View or Menu goes back. A, B, X and Y are stripped before any input is read. Keyboard: RB types, LB deletes, Menu done, View cancel. Remapper: hold the new combination for 1 s (A/B/X/Y and the D-pad ignored, LB or RB alone refused, 10 s timeout).

## Motion
Open 200 ms (fade and a 64 px slide), cluster glide on the `default` spring, focus ring and tab underline on `fast`, scroll on `default`, a 150 ms crossfade on a tab change.

## Critique
Round 1 (independent `ui-ux-designer` review of the renders): fail, with six must-fixes. Text below the 26 px floor, the backdrop on bright frames, rows hard-clipped by the scroll viewport, a ragged value edge, the crown sheen behind the header, and the missing font pack. All six were fixed or waived (fonts: see PROGRESS). Round 2: one must-fix, the help line overlapping the focus ring on focused rows. The focused row now grows to hold it (above). Re-review of 02, 06 and 07: **pass** for the Phase 9 exit. Carried over (not blocking): a check over the real pause menu, LB/RB chips in the tab row, a live preview of the draft name (the blocker preview is done, below). See `program/phases/09-osd/HANDOVER.md`.

## After the merge with Phases 6 and 8
- **Fonts.** The OSD draws in the Phase 6 motorsport pack (`assets/fonts/fonts.json`, per-family folders, variable fonts instanced at 300/400/500/600). The Phase 9 font waiver is closed.
- **Live blocker preview.** While a dash-blocker row has focus, the live view draws the real Phase 8 dome and wash (`fh6_kit::scrim`) under the cluster, so every blocker change shows on the cluster it frames.
- **Car profiles.** The rows show and edit the global `config.json`; the live view shows what the current car really gets (its `car_profiles.json` overlay on top). When the focused row reads differently for this car, its help line says "This car's profile overrides this setting", and the default-height line follows the car's anchor. Editing per-car values is Phase 10.
