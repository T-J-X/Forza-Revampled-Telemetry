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
  - **Tab bar:** a glass capsule, x 2436 to 3644, y 228 to 324. The five tab labels sit under its level face; the selected tab is a soft accent fill under the glass (22%), with its label in full ink.
  - **Rows:** viewport y 424 to 1736. Label edge x 2488 (`TX`), value edge x 3552 (`VX`). Rows are 116 px (sliders 164, carousel 340, info 124); a focused row with a help line grows to 148 (sliders 208). Rows fade at the viewport edges.
  - **Focus platter:** a glass rounded rectangle (radius 44) over the focused row, x 2436 to 3644. Under it the row lights from within (below).
  - **Hint bar:** a glass capsule, x 2436 to 3644, y 1802 to 1890, holding the keycap chips. The gearbox line ("A, B, X and Y stay with your gearbox: the menu never uses them.") sits under it at y 1972, 70 px clear of its edge.
  - **Notice:** a small glass pill at y 140 above the tab bar while a notice shows.
- **Keyboard and remapper** replace the rows. The focused key wears a glass key cap (radius 18) and lights from within; other keys are a hairline outline only.

## The glass

- **Material:** `Tokens::optical_glass` (clear synthetic sapphire, n_d 1.768, exact Fresnel, Snell, Beer-Lambert) with the token clarity and no lean of its own: the OSD's glass holds text. It mirrors the Liquid Glass cluster's studio (`assets/clusters/liquid_glass/studio_env.png`, loaded once by `OsdHost::prepare`; the kit's analytic studio when it is missing), leans to the palette's primary colour by the same 40%, and takes the same half share of the key light, so the menu and the cluster read as one material. Refraction is `World` (the captured game when glass refraction is on, else our own layers).
- **Pieces:** every piece is a `Piece` with a level face 14 px above its rim, a 36 px rounded edge (k = 2) and 4 px of base (key caps and the notice: 10 px, 20 px). All text inside a piece sits at least 52 px in from its outline, under the level face, where the glass does not bend it. All of it is one glass layer, drawn after everything it bends.
- **Clearance:** nothing outside a piece comes within 64 px of its edge: a rounded edge mirrors what lies beside it (optics.md §7), and closer text showed as a squeezed ghost in the rim (seen in the spike).
- **Light from within:** under the focused platter or key, an accent gradient rises from 0 at the top to 20% at the floor. The glass's rounded edge bends a band of it into a lit rim. This is the only accent field in the column, and it is how focus reads from the couch.
- **No glass on glass:** the platter, the bars and the notice never overlap.

## Type, colour and focus

- Labels and values 30 px cap, help, hints and status 26, keycaps 22, section headings 24 in sentence case ("Field of view", not "FIELD OF VIEW"; a single short word such as "OSD" stays). Secondary text is `inkMuted`.
- Focus is the platter, its inner light, the label going to full ink and the value to the accent. No ring, no bloom glow.
- The accent is the palette accent unless it sits within 25° of hue of the warning red; then the primary, then sapphire `#38BDF8`.

## Motion

- **Open:** the game dims and the content fades in over 200 ms; the glass materialises (thickness from 0) on the same ease; the cluster glides on the `default` spring. No slide. Close: one frame.
- **Focus and tab moves stretch like liquid:** the platter's leading edge moves on `fast` and its trailing edge on `expressive`, so it stretches toward the new row and settles; the tab fill does the same left and right. With `ReducedFlash` both edges use `fast` (no stretch).
- **Energise:** while a value is being changed the inner light rises and then decays (a spring, never a strobe, so hold-to-repeat cannot flicker). Off with `ReducedFlash`.
- **Parallax:** the key light's tilt follows the platter's height in the column (a few degrees), so the highlights travel as focus moves.
- Scroll on `default`; a 150 ms crossfade of the rows on a tab change.

## Driving-position view (camera rows)

On the camera rows of the Fit & camera tab (the three FOV rows, Seat height and Wheel offset) the live zone shows the driving position as rendered hardware, and the focus echo lifts out of its way. The code is `crates/fh6-osd/src/drive.rs`, placed by `draw.rs`.

- **Assets:** asset-forge's set `osd` (`tools/asset-forge/src/recipes/osd.rs`, ledger `assets/osd/ASSETS.md`). A ComfyUI alcantara texture (SDXL with the 4x upscale, a seamless 4096 px master) covers a Blender side cutaway of a carbon bucket seat, the wheel and its column, and the dash. Cycles renders them at 4x the OSD's scale (2200 px per m, 16-bit masters on Drive under `osd\`), box-filtered to five runtime layers: dash, wheel, wheel rim, seat and seat rim. The two rim layers are the same parts lit only by a backlit world, so every silhouette edge carries a white rim the OSD can tint. The studio is low key (near-black upholstery with lit edges), so the picture never outshines the live cluster.
- **Projection:** orthographic, yaw 24°, pitch 8°, 550 px per m, over the car's centre plane: a 957 × 572 view. The recipe and `drive.rs` share these numbers, and each has a test for them. The eye sits at (−0.02, 0, 0.66) m from the seat's H-point.
- **What moves:** the seat layer and the eye rise with Seat height (0.545 px per mm over −100 to +50 mm), and the wheel layer rises with Wheel offset (−80 to +20 mm). The dash stays put. The eye is an accent dot with a dark collar, and the default eye (seat height −45 mm, the low bucket-seat default) is a ring wide enough to hold the dot. The FOV fan opens from the eye at the cockpit FOV's angle: a radial fade out to 240 px, with its two edges in ink that fades along them. A legend ("● Your eye ○ Default", 26 px cap, secondary ink) sits under the dash.
- **Focus:** the part the focused row moves lights from its rim. Seat height adds the seat's rim layer and Wheel offset the wheel's, tinted by the accent at 70% plus 30% of the energise light and added on top. The cockpit FOV row turns the fan to the accent. The hood and chase FOV rows keep the cockpit's fan in secondary ink, since their cameras are not in this picture.
- **Layout and motion:** the focus echo (label, value, slider mirror, help line) lifts 300 px on its own spring, which snaps on open and springs on row moves. The view's top sits at y 850, centred in the live zone, over a feathered black pool (radii 620 × 390, 45% at full lift). The view fades with the lift, so it leaves as the echo comes down. Tests check that at the widest FOV (110°) and the highest seat (+50 mm) the fan's tip stays below the lifted echo's help line, and that the legend stays under the dash and clear of the wheel at its lowest offset.
- **Fallback:** without all five files, a line drawing of the same scene in the same projection (the seat's centre line, the wheel's rim and column, the dash's outline) stands in, with the focused part in the accent.
- **Open question:** the game's FOV setting may be vertical or horizontal, and neither is verified. The fan shows the setting's angle as the side view's opening. The real-game check (C) settles it; if the setting turns out to be horizontal, only the fan's angle changes.

## Rows

Unchanged by the rebuild:

| Tab | Rows |
|---|---|
| Cluster | Edit view (Cockpit / Hood / Chase; the live view follows), Cluster (carousel of 5 live thumbnails, all 29 themes), Speed units, Power units, Shift point (80–100%), Limiter light (10 Hz strobe / Gentle pulse) |
| Colour | Colour mode (Factory / Custom / Random). Factory: brand swatches. Custom: Colour (primary, secondary, accent, shift light), Hue on a spectrum track, Saturation, Brightness, Preset (6). Random: Re-roll, Same palette on every car. Then Glass refraction |
| Type & logo | Font (pack default plus every loaded family), Logo, Custom name (on-screen keyboard, 14 characters) |
| Fit & camera | Cockpit / Hood / Chase FOV (40–110°), Seat height (−100..+50 mm), Wheel offset (−80..+20 mm), Camera patch, Cluster height (±240 px, per view), Cluster scale (70–140%), the nine dash-blocker rows, Resync when the patcher offers it |
| Controls | OSD button (remapper), Hold time (200–1000 ms), Pause game when open, Stats overlay |

## Input

Unchanged: D-pad moves and adjusts (repeat after 280 ms, every 65 ms, every 25 ms after 900 ms; sliders take ×4 steps after 1.2 s and ×10 after 2.5 s), LB/RB switch tabs, View or Menu goes back. A, B, X and Y are stripped before any input is read. Keyboard: RB types, LB deletes, Menu done, View cancel. Remapper: hold the new combination for 1 s (A/B/X/Y and the D-pad ignored, LB or RB alone refused, 10 s timeout).

## Budget

The whole OSD frame (live cluster included) stays under 1.5 ms GPU at 4K, as Phase 9 set; the rebuild drops the slab's 1320 × 1880 frosted backdrop and adds three or four small pieces. The OSD's own render-thread work stays allocation-free.

## My calls (the thread's, not TJX's)

- Approach A, and the tab bar and hint bar as glass capsules (Apple's floating tab bar and toolbar), with the selection as a fill under the glass rather than a second piece of glass.
- One material with the cluster: the pilot's studio and lean, not a studio of the OSD's own.
- Focus reads by inner light rather than a ring; the accent gradient is the one coloured field in the column.
- Section headings to sentence case at 24 px (Apple's list headers, and 18 px capitals were below every ten-foot minimum).
- The driving-position view as the bespoke Blender and ComfyUI assets: it is the one picture in the menu that is about hardware.
