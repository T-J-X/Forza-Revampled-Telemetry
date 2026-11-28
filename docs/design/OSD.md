# OSD: the Liquid Glass rebuild (2026-11-27)

The controller-first OSD, rebuilt on the optical glass of the rebuilt Liquid Glass pilot (`fh6_kit::OpticalGlass`, [docs/specs/optics.md](../specs/optics.md)). It replaces the Phase 9 look (one tinted, frosted glass slab behind every row), which had the same faults TJX rejected in the first pilot. The research is [OSD_RESEARCH.md](OSD_RESEARCH.md); the Phase 3 direction it still follows for layout and input is [OSD_DIRECTION.md](OSD_DIRECTION.md). The code is `crates/fh6-osd` (`draw.rs`, `anim.rs`, `host.rs`); `preview-export --osd` renders every state into `mockups/osd/` and writes the product page `mockups/osd/index.html`.

Design file: the Penpot page `09 · OSD` ([PENPOT.md](PENPOT.md), board IDs there), redrawn for this rebuild from the kit renders.

## Intent

The OSD is where the driver changes the dash while the game is paused: pick a cluster, colour it, fit it, set the camera. It must feel like the same product as the Liquid Glass cluster beside it, read from a couch on a 55" OLED, and stay out of the way: the live cluster is the picture, the menu is the frame.

- **Goals:** glass that is physically the pilot's material; focus that reads at a glance from 2.2 m; motion that feels liquid; every change previewed live; inside the frame budget.
- **Non-goals:** new settings, new tabs or new inputs (the row model, navigation and the ABXY lock are unchanged); per-car editing (Phase 10).
- **Anti-goals:** any panel, slab or card behind the rows; frost, tint or a painted rim, sheen or fresnel band; glass on glass; a grey or navy box; a menu that outshines the live cluster.

## Approaches

| | Approach | Trade-off | Verdict |
|---|---|---|---|
| A | **Glass is the navigation only.** Rows are light on black; glass marks the tab bar, the hint bar and what has focus. | Follows Apple's own rule (glass is the navigation layer, lists are content) and the pilot's "separate pieces on black". Least glass, so each piece must be excellent. | **Chosen** (my call). |
| B | Every row is its own glass pill. | Lots of glass; text sits on glass everywhere (Apple says lists stay content), the column turns into a stack of capsules, and the GPU pays for 10 to 15 pieces. | Rejected. |
| C | Keep one panel, rebuilt as optical glass. | Still a slab (AGENTS 3A forbids a glass panel), and a flat face over black only shows a rim. | Rejected. |

## Layout (4K units)

- **Live zone (left, about 60%).** Unchanged in content: the real selected cluster at its real height and scale, gliding to the live-zone centre on open; the header ("Live view", the cluster name, the data source and the edited view); the focus echo (the focused row's label, its value at 110 px, a slider mirror, its help line); for camera rows, the driving-position view (below); for cluster height, the dashed default-height line. Feathered black pools behind the echo (45%) and the live cluster (35%).
- **Control column (right, x 2380 to 3700 before the right-align offset).** No panel. A feathered black pool (a shadow of the column's rounded rectangle, σ 90, 78% at the core) keeps it dark over a bright game frame and is invisible on OLED black.
  - **Tab bar:** a glass capsule, x 2436 to 3644, y 228 to 324. The five tab labels sit under its level face; the selected tab is an accent fill under the glass, with its label in full ink. The fill's alpha is set from the accent's luminance so it shows at relative luminance 0.13 (3:1 against the bar's face, and its label still over 4.5:1) with any palette, within 22 to 60%.
  - **Rows:** viewport y 424 to 1736. Label edge x 2488 (`TX`), value edge x 3552 (`VX`). Rows are 116 px (sliders 164, carousel 320, info 124); a focused row with a help line grows to 164 (sliders 220), so the help line's descenders stay 36 px inside the platter's rim. Each kind's content is centred in its row, so on the platter it sits as far from the top rim as from the bottom one. The focused row keeps 52 px clear above and below its platter, and a section heading sits above that space, so headings stay more than 64 px from the platter's edge. The content starts 40 px below the viewport's top, and rows fade only within 40 px of either edge, so nothing is faded before the list scrolls. The list scrolls to keep the focused row near the middle, moved to the nearest place where a whole row starts at the top (with its heading, so a list never opens on a row whose heading has scrolled away), and its end is padded by the same 40 px, so the last row is never faded while all of it shows. A heading shows only with its row, so a list never ends on a heading. The scroll bar, 64 px right of the glass, shows quietly at rest and fully for a second on open, on a tab change and while the list moves.
  - **Scroll bar:** 64 px right of the glass, a quiet-ink track and a thumb in half secondary ink. It shows while the list moves and for 1 s after (and briefly on open), then hides.
  - **Focus platter:** a glass rounded rectangle (radius 44) over the focused row, x 2436 to 3644. Under it the row lights from within (below).
  - **Hint bar:** a glass capsule, x 2436 to 3644, y 1802 to 1890, holding the keycap chips. The gearbox line ("A, B, X and Y stay with your gearbox: the menu never uses them.") sits under it at y 1972, 70 px clear of its edge.
  - **Notice:** a small glass pill at y 140 above the tab bar while a notice shows.
- **Keyboard and remapper** replace the rows. The focused key wears a glass key cap (radius 18) over an accent face at 90%, with its letter dark (`#0A0E12`) and larger; other keys are a hairline outline with their letters in secondary ink. Caps lock shows a dot and the words "Caps on". The remapper names the buttons it accepts, draws its hold bar in the accent over a quiet-ink track, and shows the current binding and the time left.

## The glass

- **Material:** `Tokens::optical_glass` (clear synthetic sapphire, n_d 1.768, exact Fresnel, Snell, Beer-Lambert) with the token clarity and no lean of its own: the OSD's glass holds text. It mirrors the Liquid Glass cluster's studio (`assets/clusters/liquid_glass/studio_env.png`, built by the same `liquid_glass::studio_glass` the cluster uses; the kit's analytic studio when it is missing), leans to the palette's primary colour by the same 40%, and takes the same half share of the key light, so the menu and the cluster read as one material. Refraction is `World` (the captured game when glass refraction is on, else our own layers).
- **Pieces:** every piece is a `Piece` with a level face 14 px above its rim, a 36 px rounded edge (k = 2) and 4 px of base (key caps and the notice: 10 px, 20 px). All text inside a piece sits at least 52 px in from its outline horizontally and at least 36 px vertically, descenders included, which keeps it off the 36 px rounded edge (a bar or a 116 px row has no room for 52 px above and below a 30 px cap), under the level face, where the glass does not bend it. All of it is one glass layer, drawn after everything it bends.
- **Clearance:** nothing outside a piece comes within 64 px of its edge: a rounded edge mirrors what lies beside it (optics.md §7), and closer text showed as a squeezed ghost in the rim (seen in the spike).
- **Light from within:** under the focused platter, the accent rises from 0 through the bottom 120 px (at most half the platter, so a short one keeps its upper part clear) to a flat lit band in the last 24 px. That band's alpha is set from the accent's luminance so it shows at relative luminance 0.15 (over 3:1 against the dark column) with any palette, and while energised it rises to at most 0.19, where full ink still reads at 4:1. The light is the accent's colour with at least 0.10 OKLCH chroma, so a pale accent never turns into a grey field. The glass's rounded edge bends a band of it into a lit rim. It is the only accent field in the column, and it is how focus reads from the couch.
- **No glass on glass:** the platter, the bars and the notice never overlap.

## Type, colour and focus

- Labels and values 30 px cap, help, hints and status 26, keycaps 22, section headings 24 in sentence case ("Field of view", not "FIELD OF VIEW"; a single short word such as "OSD" stays).
- Four inks: full ink for what has focus and for help on the lit platter (secondary ink drops under 4.5:1 there); label ink (86%) for unfocused labels, the header, the hint text, the remapper's instructions and the gearbox line (over 4.5:1 even over the dimmed game's brightest); secondary ink (`inkMuted`) for unfocused values and section headings; quiet ink (21%, at least 3:1 on black as measured on the renders; 17% measured 2.95:1) for unfocused slider fills, chevrons, the scroll and hold tracks and the echo's mirror track.
- Focus is the platter, its inner light, the label going to full ink and the value to the accent. No ring, no bloom glow.
- **Controls:** a slider's knob travels inside its track and is opaque; the focused row's knob is larger, with a dark ring that keeps it apart from its own glow. The unfilled part is an outlined slot on the focused row and a 2 px quiet line on the others, so filled and unfilled differ by shape as well as ink. A tick marks the fresh-config default on every slider but Hue, Saturation and Brightness. The Hue track lies on a dark bed. A switch that is on and focused has a dark knob on the lit pill; on and unfocused, an ink knob on a dimmed pill; off, an outlined pill. Choice rows show both chevrons, in the accent when focused and in quiet ink otherwise.
- **The echo:** the value at 110 px (a name wider than 420 px drops to 84), a unit made of letters at half size in secondary ink on the same baseline, and the slider mirror (the Hue row's as a spectrum). The help line is on the platter only.
- The accent is the palette accent unless it sits within 25° of hue of the warning red, or in the blue-to-violet band (OKLCH hue 250 to 310°, where light under the glass reads as a slate or navy box); then the primary, by the same tests; then sapphire `#38BDF8`. Fills from the accent are set by its luminance (the selected tab, the inner light).

## Motion

- **Open:** the game dims and the content fades in over 200 ms; the glass materialises (thickness from 0) on the same ease; the cluster glides on the `default` spring, and the column slides in 64 px from the right on the same ease (my call: it reads as the menu arriving and costs nothing). Close: one frame.
- **Focus and tab moves stretch like liquid:** the platter's leading edge moves on `fast` and its trailing edge on `expressive`, so it stretches toward the new row and settles; the tab fill does the same left and right. With `ReducedFlash` both edges use `fast` (no stretch).
- **Energise:** while a value is being changed the inner light rises and then decays (a spring, never a strobe, so hold-to-repeat cannot flicker), and the focused knob glows only then. Off with `ReducedFlash`.
- **Parallax:** the key light's tilt follows the platter's height in the column (a few degrees), so the highlights travel as focus moves.
- **No text in a moving rim:** rows take their new heights at once while the platter travels, so a row or heading that a rim crosses hides, fading over the last 16 px inside the platter or the last 36 px outside it (the rim's ghost reach is about 35 px), and returns as the platter settles. At rest every row clears it.
- Scroll on `default`. On a tab change the rows crossfade over 150 ms and the platter and the scroll start on the new tab's row, so the light never travels over a row that does not have focus; the tab fill still slides.

## Driving-position view (camera rows)

On the camera rows of the Fit & camera tab (the three FOV rows, Seat height and Wheel offset) the live zone shows the driving position as rendered hardware, and the focus echo lifts out of its way. The code is `crates/fh6-osd/src/drive.rs`, placed by `draw.rs`.

- **Assets:** asset-forge's set `osd` (`tools/asset-forge/src/recipes/osd.rs`, ledger `assets/osd/ASSETS.md`). A ComfyUI alcantara texture (SDXL with the 4x upscale, a seamless 4096 px master) covers a Blender side cutaway of a carbon bucket seat, the wheel and its column, and the dash. Cycles renders them at 4x the OSD's scale (2200 px per m, 16-bit masters on Drive under `osd\`), box-filtered to five runtime layers: dash, wheel, wheel rim, seat and seat rim. The two rim layers are the same parts lit only by a backlit world, so every silhouette edge carries a white rim the OSD can tint. The studio is low key (near-black upholstery with lit edges), so the picture never outshines the live cluster.
- **Projection:** orthographic, yaw 24°, pitch 8°, 550 px per m, over the car's centre plane: a 957 × 572 view. The recipe and `drive.rs` share these numbers, and each has a test for them. The eye sits at (−0.02, 0, 0.66) m from the seat's H-point.
- **What moves:** the seat layer and the eye rise with Seat height (0.545 px per mm over −100 to +50 mm), and the wheel layer rises with Wheel offset (−80 to +20 mm). The dash stays put. The eye is an accent dot (radius 16) with a dark collar, and the default eye (seat height −45 mm, the low bucket-seat default) is a ring (radius 24, 4 px) wide enough to hold the dot. The FOV fan opens from the eye at the cockpit FOV's angle, out to 240 px: two edges that keep 60% of their strength to the end and an arc that closes it. A legend ("● Your eye ○ Default", 26 px cap, secondary ink) sits under the dash.
- **Focus:** the part the focused row moves lights from its rim. Seat height adds the seat's rim layer and Wheel offset the wheel's, tinted by the accent at 70% plus 30% of the energise light and added on top; the other part keeps a quiet rim (30% secondary ink), so both outlines hold over black. The cockpit FOV row turns the fan to the accent, fills it with a radial fade and doubles its stroke. The hood and chase FOV rows keep the cockpit's fan in secondary ink, edges only, since their cameras are not in this picture.
- **Layout and motion:** the focus echo (label, value, slider mirror) lifts 300 px on its own spring, which snaps on open and springs on row moves. The view's top sits at y 850, centred in the live zone, over a feathered black pool (radii 620 × 390, 45% at full lift). The view fades with the lift, so it leaves as the echo comes down.
- **Clear of the cluster:** a raised or enlarged live cluster would run into the view, so the view shrinks about the middle of its top edge until its lowest point (a lowered seat hangs below it) keeps 64 px above the cluster's envelope, and its pool shrinks with it. Below 60% it fades out instead (`draw::drive_fit`). At the default anchor, height and scale it is full size; with the seat at its lowest it draws at about 96%. Tests check that at the widest FOV (110°) and the highest seat (+50 mm) the fan's tip stays below the lifted echo's slider mirror, that the view keeps 64 px above a raised and enlarged cluster or fades out, and that the legend stays under the dash and clear of the wheel at its lowest offset.
- **Fallback:** without all five files, a line drawing of the same scene in the same projection (the seat's centre line, the wheel's rim and column, the dash's outline) stands in, with the focused part in the accent.
- **Open question:** the game's FOV setting may be vertical or horizontal, and neither is verified. The fan shows the setting's angle as the side view's opening. The real-game check (C) settles it; if the setting turns out to be horizontal, only the fan's angle changes.

## Start-up loading

The studio and the five driving-position layers are files to read and decode (the studio is a 4096 px map), so the app loads them when its kit is ready (`OsdHost::load_assets`), not on the first open; `OsdHost::prepare` loads them only if that was never called. A lost device calls `OsdHost::device_changed`, which forgets everything made for the old kit. A carousel thumbnail leaves out text under 12 px (`Painter::set_min_text_px`), where labels would read as grey bars; its gear, speed and shapes still draw.

## Rows

Unchanged by the rebuild:

| Tab | Rows |
|---|---|
| Cluster | Edit view (Cockpit / Hood / Chase; the live view follows), Cluster (carousel of 5 live thumbnails, every dash in the catalog), Housing (on / off, only for a dash that has a housing: its slab, bezel or wheel face; stored per dash in the config key `Housing`, defaults in `fh6_clusters::housing`; AMG MBUX dashes off, ValkyrieLmp1 on), Speed units, Power units, Shift point (80–100%), Limiter light (10 Hz strobe / Gentle pulse) |
| Colour | Colour mode (Factory / Custom / Random). Factory: brand swatches. Custom: Colour (primary, secondary, accent, shift light), Hue on a spectrum track, Saturation, Brightness, Preset (6). Random: Re-roll, Same palette on every car. Then Glass refraction |
| Type & logo | Font (pack default plus every loaded family), Logo, Custom name (on-screen keyboard, 14 characters) |
| Fit & camera | Cockpit / Hood / Chase FOV (40–110°), Seat height (−100..+50 mm), Wheel offset (−80..+20 mm), Camera patch, Cluster height (±240 px, per view), Cluster scale (70–140%), the nine dash-blocker rows, Resync when the patcher offers it |
| Controls | OSD button (remapper), Hold time (200–1000 ms), Pause game when open, Stats overlay |

## Input

Unchanged: D-pad moves and adjusts (repeat after 280 ms, every 65 ms, every 25 ms after 900 ms; sliders take ×4 steps after 1.2 s and ×10 after 2.5 s), LB/RB switch tabs, View or Menu goes back. A, B, X and Y are stripped before any input is read. Keyboard: RB types, LB deletes, Menu done, View cancel. Remapper: hold the new combination for 1 s (A/B/X/Y and the D-pad ignored, LB or RB alone refused); D-pad left cancels, and it times out 30 s after the held buttons last changed.

## Budget

The whole OSD frame (live cluster included) stays under 1.5 ms GPU at 4K, as Phase 9 set; the rebuild drops the slab's 1320 × 1880 frosted backdrop and adds three or four small pieces. The OSD's own render-thread work stays allocation-free.

## My calls (the thread's, not TJX's)

- Approach A, and the tab bar and hint bar as glass capsules (Apple's floating tab bar and toolbar), with the selection as a fill under the glass rather than a second piece of glass.
- One material with the cluster: the pilot's studio and lean, not a studio of the OSD's own.
- Focus reads by inner light rather than a ring; the accent gradient is the one coloured field in the column.
- Section headings to sentence case at 24 px (Apple's list headers, and 18 px capitals were below every ten-foot minimum).
- The driving-position view as the bespoke Blender and ComfyUI assets: it is the one picture in the menu that is about hardware.
- After the design critique and the accessibility review (2026-11-28): fills from the accent set by its luminance rather than a fixed alpha; four inks, with help in full ink on the lit platter; outlined slots for unfilled tracks rather than a brighter grey that crowds the accent; the focused key as a solid accent face; D-pad left as the remapper's cancel (the D-pad is never part of a binding); the driving view shrinking above a raised cluster rather than moving sideways, where it would cover the column or the game's minimap.
- Left open, for a later layout phase: a text-size setting (WCAG 1.4.4) and larger base sizes for the 26, 24 and 22 px classes. Both move every row and would need the column to widen or the tabs to scroll.
