# CarplayIsland: research note

Build #18, catalog row 53, category Apple Vision, Target `inspired`, family Liquid Glass, view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Telemetry updated 2026-11-28 for the 88-field decoder (main 60c99c9).

Labels: **Cited** means read on 2026-11-28 from the source linked (for videos, from Apple's own transcript page). **Computed** means worked out here from cited or repo values, with the inputs and numbers shown (`python3 -I` with numpy and scipy). **Our reading** is a conclusion drawn for this dash, open to the direction check. "Unconfirmed" marks what step 1 must still check. No image bytes could be fetched in this pass (every image host returned an error), so nothing below is measured from a picture; the web budget then ran out, so the Aston Martin press release and the 2026 maker list were not re-read.

## 1. The real display: parts kept, measured where sources allow, and departures

The real target is the **Aston Martin CarPlay Ultra instrument cluster** (first shipped 15 May 2025 in the DBX707, then the core range), with the gauge grammar of Apple's CarPlay design system and the motion of iOS (Dynamic Island, springs). Research found no better real target: it is still the only shipping CarPlay Ultra cluster as far as these sources reach.

### 1.1 Inventory of the real display

| # | Feature | Label and source |
|---|---|---|
| R1 | CarPlay Ultra "provides content for all the driver's screens, including the instrument cluster", with "speedometer, tachometer, fuel gauge, temperature gauge, and more"; a "curated set of instrument cluster themes", colours and wallpapers changeable, themes "crafted in close collaboration between Apple and the automaker's design team". | Cited: Apple Newsroom, 15 May 2025 |
| R2 | Apple's own captions of four theme photos: "thin blue bars on the speedometer and tachometer", "multi-colored bars for various measurements", "thick gray bars", "thick red bars". | Cited: Apple Newsroom image alt texts |
| R3 | Aston Martin calls its choices "classic twin dial or innovative 'Power Bar' themes", with "colours, textures and patterns including gradients, landscapes and topographical options for each theme"; tyre-pressure alerts "appear as pop-ups in situ". | Cited: Aston Martin brand story |
| R4 | The Aston theme: "two dials, in understated hues, offset by a white font with an additional (configurable) panel between them"; the reviewer would "prefer a needle to the rise and fall of a coloured band". So the Aston dials show the value as a **filled arc, not a needle**. | Cited: PistonHeads hands-on |
| R5 | The Aston theme is "dressed in British Racing Green" with a "'Made in Great Britain' script on the speedometer"; Auto Express counts **six layouts**, "from a full-screen display for music or sat-nav maps, to minimalist line gauges, reminiscent of the Modular face on the Apple Watch". Top Gear names three Aston ones: the "racing green twin roundel set-up that mimics analogue dials", "colour horizontal lines (more of an Apple-style look)" and a full-screen map "with speed and engine revs minimised at the bottom". | Cited: Auto Express; Top Gear |
| R6 | "Analog speed and rev counters on the left and right sides. In between the dials is an 'info square'"; the map layout puts "indicators, fuel level, and speed" on "a translucent strip" at the base. | Cited: AppleInsider |
| R7 | The centre square changes with up and down swipes on the wheel's thumb pad (maps, audio, tyre pressure, trip); left and right swipes change layout, "much like flipping between Apple Watch faces". Colours are free, layouts fixed. | Cited: Top Gear; TechRadar ("fairly static no matter what overall instrument theme") |
| R8 | Gauge geometry is the maker's: "Circular gauges let you decide the arc of the gauge too, as well as its start and end points"; "The number of ticks, their length, style and their position are all customizable. You can even have multiple sets of ticks"; stroke width up to "all the way to the center"; flat colours, several colours per gauge, gradients; a modern needle "with a little bit of an homage to a classic gauge needle" or a true analogue needle. | Cited: WWDC24 10112, Gauge customization |
| R9 | Type is Apple's SF variable family: weight, width and "the softness of the corners" on continuous axes. Reviewers saw it as "slightly more sans-serif" and more legible than Aston's own. | Cited: WWDC24 10112; SlashGear; Wallpaper |
| R10 | The speedometer is "always paired with a fuel gauge, or state of charge gauge"; speed and fuel are mandatory, the rest optional. Cruise target is "a dot", set speed "as text adjacent to that dot"; a speed limiter "shows as a simple line". Space is reserved "for road signage as well as alternate units for speed". Low fuel orange, critical red. | Cited: WWDC24 10112 |
| R11 | The tachometer carries "current gear for manual drive modes, redline, and arrows for recommended gear changes"; in a circular gauge the centre "transmission state" can be swapped for an image or trim badge. | Cited: WWDC24 10112 |
| R12 | Dynamic content always has a reserved space; it "can be stacked behind non-critical gauge elements" (real depth order); it is the default place for notifications, two at once. | Cited: WWDC24 10112 |
| R13 | Gauges are Local UI drawn by the car (OpenGL renderer plus an asset package of images and behaviour scripts sent from the iPhone at pairing), so they are low latency, survive a dropped link and are "ready to show" when the door opens. Telltales are the car's Overlay UI, frontmost. | Cited: WWDC24 10111 |
| R14 | "The RPM gauge gets updated with a single value and an associated state. The state is set to Normal most of the time, but if the driver manages to exceed the maximum engine speed, it's up to you to update both the value and the state." No shift light is described anywhere. | Cited: WWDC24 10111 |
| R15 | "Any smoothing or adjustments are executed by the vehicle before forwarding to the UI. The next generation of CarPlay simply shows exactly what you send." Speed comes in three units at once; the finest is used "for smooth gauge animations". | Cited: WWDC24 10111 |
| R16 | iOS 26: "the new Liquid Glass design extends to the CarPlay interface", and the iOS 26 features reach CarPlay Ultra; no source says the Aston cluster dials themselves became glass. | Cited: MacRumors, 13 Jun 2025. Our reading: the glass is the dash's own step from Apple's language, not a copy of the cluster |
| R17 | Aston's 12.3 in driver display is flat. | Cited: TechRadar |

### 1.2 Kept, measured, and departures

Kept from the real display: two round dials on a dark field with a panel between them (R4, R6); the value drawn as a **rising coloured band, no needle** (R4), which is the brief's "lit arc"; white numerals (R4); the speed on the left and revs on the right (R6); a calm centre panel whose content is a widget (R7, R12); free colour from palette roles with a fixed layout (R7); a maker-set tick ring (R8); the gear in the round gauge's centre or the panel (R11); alternate speed units (R10); the speedometer paired with a fuel gauge (R10), now drawn from the decoded `fuel` field (§5).

Measured: nothing could be measured from a picture in this pass. **Our reading** of the brief's geometry against the binnacle (Computed from the brief's numbers and the cockpit envelope 1440 × 560 from (1200, 1515) about the anchor (1920, 1835), so local x ±720 and y −320 to +240): the dials reach x ±590, the island's top edge is at y −310 (10 px inside the envelope), the widgets' bottom at y +230 (10 px inside). The island's bottom (−270) clears the dial tops (−210) by 60 px and the pane top (−190) by 80 px. The pane (x ±220) overlaps each dial's inner edge (x ±190) by 30 px, as the brief says. Step 1 measures the real dial diameter to gap ratio from the Apple Newsroom photo (§2, ref 01) and compares.

Departures (each listed again in §9): no needle option is used; the temperature gauge reads the tyres, because engine temperature is not in the packet (the fuel gauge is no longer a departure, §5); the green Aston livery and the "Made in Great Britain" script are not copied (palette roles and the car's own logo instead); no wallpaper (OLED black); SF type is replaced by OFL Saira and Titillium Web; the cluster's flat panel becomes three depths of clear glass; the island is the dash's own part, from the iPhone's Dynamic Island, and has no counterpart in the real cluster.

## 2. Reference photos

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 01 | Aston twin dials with the centre square (Apple's caption: speedometer and tachometer) | https://www.apple.com/newsroom/images/2025/05/carplay-ultra-the-next-generation-of-carplay-begins-rolling-out-today/article/Apple-CarPlay-Ultra-cluster-ADAS_big.jpg.large.jpg | no (fetch error) |
| 02 | Widgets in the cluster | .../article/Apple-CarPlay-Ultra-cluster-widgets_big.jpg.large.jpg (same folder) | no |
| 03 | Map layout | .../article/Apple-CarPlay-Ultra-cluster-Maps_big.jpg.large.jpg | no |
| 04 | Tyre pressure card in the centre | .../article/Apple-CarPlay-Ultra-cluster-TPMS_big.jpg.large.jpg | no |
| 05 | Theme: thin blue bars | .../article/Apple-CarPlay-Ultra-instrument-cluster-themes-01_big.jpg.large.jpg | no |
| 06 | Theme: multi-coloured bars | .../themes-02_big.jpg.large.jpg | no |
| 07 | Theme: thick grey bars | .../themes-03_big.jpg.large.jpg | no |
| 08 | Theme: thick red bars | .../themes-04_big.jpg.large.jpg | no (fetch error) |
| 09 | Aston official: cluster replacement | https://www.astonmartin.com/-/media/brand-stories/carplay-2/final-imagery/lft---instrument-cluster-replacement.jpg | no (fetch error) |
| 10 | Aston official: widget support | https://www.astonmartin.com/-/media/brand-stories/carplay-2/final-imagery/lft---widget-support.jpg | no |
| 11 | Aston official: multi-instrument scroll | https://www.astonmartin.com/-/media/brand-stories/carplay-2/final-imagery/ferrite-scrollable-desktop---multi-instrument-new.jpg | no |
| 12 | Owner-style photo, "traditional gauge cluster" | https://www.slashgear.com/img/gallery/i-tested-apples-new-carplay-ultra-for-a-week-and-its-not-what-i-expected/turn-on-log-in-drop-out-1753204080.jpg | no (fetch error) |
| 13 | Press photo, instrument panel (Aston supplied) | https://images.cars.com/cldstatic/wp-content/uploads/apple-carplay-ultra-15-interior-instrument-panel-oem.jpg | no (fetch error) |
| 14 | "Apple CarPlay dials in the DBX707" | https://cdn.mos.cms.futurecdn.net/QhPzyyvyhYtU8xqNYurH8S.jpg | no (fetch error) |
| 15 | Dynamic Island compact and expanded figures | https://developer.apple.com/design/human-interface-guidelines/live-activities (figures "type-compact" and "type-expanded") | no |

Drive `references` folder: https://drive.google.com/drive/folders/1FB3gSZOuDNq9G6oLS40sK_6z-x_okXvj (holds `SOURCES.md`; no images, so the PC fetches refs 01, 05 to 08 and 12 first). Apple's page also offers `Images-of-Apple-CarPlay-Ultra.zip` in the same article folder.

## 3. How the real thing behaves, with motion notes

| State | What the real display does | Label and source | For this dash |
|---|---|---|---|
| Start-up | "Greeted by an Aston and CarPlay logo on the instrument cluster"; the gauges "initiated already in the Ultra font" on unlock; the cluster boots first. Content is ready "when the door opens". | Cited: Top Gear; SlashGear; AppleInsider; WWDC24 10111 | Show: the pieces materialise over 200 ms by growing thickness (family rule), far to near. The logo zone shows the car's logo, never an Apple mark |
| Idle | Not described. | Unconfirmed | Island at its 200 px pill, eight dim seat dots, arcs at idle |
| Cruise | "As we start driving, the speed increases, the gauge animates." The band rises and falls. | Cited: WWDC24 10111; PistonHeads | Speed arc and tabular digits; widgets live |
| Pull to the limiter | Band rises; "redline" and "arrows for recommended gear changes" on the tach. | Cited: WWDC24 10112 | Island stretches seat by seat (§7) |
| Limiter | The car sets the rpm state past maximum engine speed; no flash or shift light is documented. | Cited: WWDC24 10111 | Only the island's lamps strobe; arcs hold steady `shift` |
| Gear change | Gear shown on the tach for manual modes, transmission state in a round gauge's centre. No transition described. | Cited: WWDC24 10112 | §7 |
| Braking | Nothing on the cluster. | Not found | Brake ring in the left widget |
| Lateral G | Nothing on the cluster. Liquid Glass highlights "sometimes" follow device motion. | Cited: pilot RESEARCH A7 (WWDC25 219) | Parallax 0, 2, 4 px per g (tokens `motion.springs.parallax`, ω 12) and the light rig's swing |
| Reverse and neutral | Rear camera is the car's punch-through UI, shown "quickly" even before the phone connects. | Cited: WWDC24 10111 | No camera (§5) |
| Layout change | "Seamless layout transitions and animations", matched frame by frame "at 60 frames per second. Or even faster". | Cited: WWDC24 10111 | Not used: one layout |
| Shut-down | Not described. | Unconfirmed | Host hides the dash in one frame (family rule) |

**Dynamic Island motion (the island's model).** It is "a singular system layer that can organically shape shift", "designed to feel like a living organism, with a deliberate elasticity"; it "can grow horizontally to what's called the Compact view, and also bloom into a larger Expanded view"; content stays "concentric" with its shape (Cited: WWDC23 10194). Live Activity animations last at most two seconds, and a layout change should move existing elements to new places rather than remove and re-add them (Cited: HIG Live Activities). Dynamic Island backgrounds are black and opaque, with a tinted key line in dark mode (Cited: same). Sizes: compact leading and trailing 62.33 × 36.67 pt each on a 430 pt wide phone, expanded 408 pt wide and 84 to 160 pt tall (Cited: HIG Specifications). **Computed:** the expanded island spans 408 ÷ 430 = 95% of the phone's width; the dash's full island spans 800 ÷ 1440 = 56% of the envelope, so it reads as a capsule, not a bar.

**Apple's springs.** A spring is set by perceptual duration d and bounce; stiffness = (2π ÷ d)², and "when bounce is 0, we get this smooth curve" (critically damped); on retargeting "a spring animation uses the velocity it had when it was retargeted as the initial velocity" (Cited: WWDC23 10158). **Computed:** ω = 2π ÷ d, so the brief's ω = 14 rad/s is Apple's `spring(duration: 0.449, bounce: 0)`; SwiftUI's example duration 0.5 s would be ω = 12.57 rad/s.

## 4. The maths

Model (catalog row): critically damped spring for the island's half-width, x(t) = x_T + (c₁ + c₂t)e^(−ωt), c₁ = x₀ − x_T, c₂ = v₀ + ωc₁, ω = 14 rad/s. Refraction at the pane's rim per optics.md §4.

**Computed checks (values the unit tests assert):**
- From rest, (1 + u)e^(−u) = 0.05 at u = 4.74386, so t₅% = 0.33885 s. The catalog's 4.7439 ÷ ω = 0.3388 s **checks; no correction**. Within 2%: 0.4167 s.
- From rest the peak speed of a step Δ is Δω ÷ e at t = 1 ÷ ω = 71.4 ms: 3,090 px/s for the full 600 px stretch.
- Overshoot with v₀ ≠ 0 happens when the edge is already moving towards a new, near target faster than ω|c₁|. Example: width 300 px moving inwards at 3,000 px/s when the target drops to 200 px: c₁ = 100, c₂ = −1,600, the edge passes 200 and reaches 182.5 px. So the clamp to [200, 800] is needed, as the row says.
- **Seat geometry.** Ten seats over 760 px (centres) gives a pitch of 760 ÷ 9 = 84.44 px. For n lit seats a side, with the capsule's end radius 20 px round the last seat, the half-width is h(n) = 20 + (n − 0.5) × 84.44: 62.2, 146.7, 231.1, 315.6 and 400.0 px. So n = 5 is exactly the 800 px island, and the 200 px resting pill (half 100) already covers seat 1 each side: the brief's numbers are consistent.
- **Lag (a finding).** A critically damped spring following a steady ramp trails it by 2 ÷ ω = 0.143 s (8.6 frames at 60 Hz). In a pull at 3,000 rpm/s that is 429 rpm, which can be two seats. Because the brief says the seats follow the glass, the rank-1 shift lights would light about 143 ms late, against CLUSTER_PROMPT closing line 7 (telemetry on screen within one frame). Fix options in §9 Q1; the recommended one feeds the spring target forward by `rpm_rate` × 2 ÷ ω (the `Signals.rpm_rate` field exists for needle feed-forward, and the tokens' `needle` spring uses it).
- **Rim refraction.** At the rim a piece displaces what is under it by b√(η² − 1) = 1.46 b for sapphire (optics.md §4, tested there). With a pane base of b px, ticks seen through the pane's rim shift by 1.46 b towards the pane's inside; b is a board choice (for b = 12 px, 17.5 px).
- **Island to pane fusing (if used, §7 idea L2).** Two outlines 80 px apart touch at k = 80 ÷ (2 ln 2) = 57.7 px (optics.md §2); at rest the family needs a gap of at least 3k, so k at rest ≤ 26.7 px.
- **Boost unit.** The brief's "0.05 bar" threshold is 0.725 psi; the telemetry field is `boost_psi`.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Speed arc and digits (left dial) | `speed`, `unit` | verified | arc 0 to the car's learned top speed or a round scale; digits through `Hysteresis` |
| Alternate unit line (R10) | `speed_ms` | verified | the other unit, small, under the digits |
| Rev arc and tick ring (right dial) | `rpm`, `idle_rpm`, `max_rpm` | verified | arc from idle to max; scale from the driven car's max |
| Island width and seats | `rpm`, `shift_rpm`, `rpm_rate` | verified, derived | seat count from the per-gear window; half-width h(n) on the ω 14 spring, target fed forward |
| Limiter strobe | rpm at or past `shift_rpm`, or bouncing at `max_rpm` | verified | island lamps only, 10 Hz 50%; 2 Hz breath gentle |
| Gear numeral in the pane | `gear` | verified | R, N, 1 to 10+ |
| Outer ring | `power` | verified | power ÷ highest power seen this session (one `f32`) |
| Middle ring | `boost`, else `throttle` | verified | boost ÷ session max; throttle while boost < 0.725 psi since start, labelled THR |
| Inner ring | `brake` | verified | 0 to 1 |
| G-dot widget | `g_lat`, `g_long` | verified | `gBall` spring (T 0.168 s, ζ 0.9) |
| Depth parallax and light rig | `g_lat`, `g_long` | verified | 0, 2, 4 px per g; rig swing per family |
| Fuel arc (inner track of the speed dial, R10) | `fuel` | decoded, offset confirmed at check C | the arc shows the tank share; the page states no unit or range, so check C reads the value on a full tank and after a long drive before the arc assumes 0 to 1; `caution` below the low mark and `warning` below the critical mark (R10's orange and red; Our reading: 15% and 5% until the board sets them); fallback the ENERGY arc (below) when fuel is not sent |
| Temperature gauge (short arc on the rev dial) | `tyre_temp` | decoded, offset confirmed at check C | the hottest of the four tyres, labelled TYRE; the page states no unit (Forza titles have sent Fahrenheit), so the scale ends come from check C readings on cold and hot tyres, and digits, if any, convert by `unit` only after that; fallback the LOAD arc (below) |
| Trip page distance | `race_progress` (distance_traveled) | decoded, offset confirmed at check C | session distance = distance_traveled minus its value at session start; fallback ∫speed dt |
| Tyre card corners (second page of the G-dot widget) | `tyre_temp` | decoded, offset confirmed at check C | four corner values on the plan outline of the car, in the place of the real tyre card (R7) |
| Best-lap toast | `race_progress` (best_lap) | decoded, offset confirmed at check C | a toast when best_lap drops to a new value; never outside a race |
| EV detection | `cylinders` | decoded, offset confirmed at check C | 0 means an EV (fuel arc labelled as charge, §6); fallback the rpm heuristic of §6 |

**Shown for real now** (moved out of "What the game cannot send" on 2026-11-28, since the decoder reads all 88 fields):

- **Fuel gauge** (mandatory in CarPlay, R10). ★ **Real fuel arc** from `fuel` on the speed dial's inner track, as in the table above, so the speedometer is paired with a fuel gauge exactly as Apple requires. Second (kept as the fallback when `fuel` is not sent, §6): **Energy spent**, a fuel-style arc filling with ∫power dt since the session started, in kWh or hp·h by `power_unit` (derived exactly; it is crank work, so the label says ENERGY, never FUEL). Third: a distance arc (distance_traveled, or ∫speed dt) reading like a trip gauge.
- **Temperature gauge.** The sources do not name what the real gauge measures (Our reading: engine temperature, which the packet does not carry), so this is a real temperature but a different one. ★ **Tyre temperature arc** from `tyre_temp`, labelled TYRE so it never reads as an engine gauge. Second: **Load duty**, the share of the last 60 s spent above 90% throttle, as the same short arc (derived exactly from throttle and the timestamp; rises with hard driving, labelled LOAD).

**What the game cannot send** (each with a replacement the telemetry drives; ★ marks the recommendation):

1. **Engine temperature** (coolant or oil, if that is what the real gauge shows): not in the 88 fields. The TYRE arc above takes the place; nothing is labelled as engine heat.
2. *(Fuel moved to "Shown for real now" above.)*
3. **Map in the centre square.** ★ **Speed trace as a route line**: the last 20 s of speed drawn as one thick rounded line in the pane behind the gear, in the manner of a Maps route, with a dot at "now" (derived). Second: the G-trace (friction circle with a 500 ms tail, tokens `gBall.trailMs`) as the pane's background.
4. **Now Playing and media.** ★ **Gear as the track**: a scrubber under the gear whose playhead is rpm within the gear's window (idle or downshift point to shift point) and whose elapsed time is time in gear, with "GEAR 3" as the title (derived). Second: the engine's firing frequency as a level meter, rpm ÷ 60 × cylinders ÷ 2 Hz for a four-stroke, from `cylinders` (decoded, offset confirmed at check C); rpm ÷ 60 as the crank frequency when cylinders is 0 or not sent.
5. **Tyre pressure card** (R7). Pressures are not in the 88 fields, but tyre temperatures are. ★ The **G-dot widget** takes its place (already in the brief), framed by a plan outline of a car so it reads where the tyre card did, with the four tyre temperatures from `tyre_temp` (decoded, offset confirmed at check C) at its corners or on its second page, labelled TEMP, never PSI or BAR. Second: four corner bars of suspension travel from `suspension` (decoded, offset confirmed at check C), with MODEL load transfer from g_lat and g_long (labelled MODEL) as their fallback.
6. **Trip computer.** ★ **Session trip**: distance (from distance_traveled, decoded, offset confirmed at check C; ∫speed dt as the fallback), time, average and top speed, and a 0 to 100 km/h (or 0 to 60 mph) timer from speed and the timestamp, in the right widget's second page (derived). In a race the page also shows lap, last lap and best lap from `race_progress` (decoded, offset confirmed at check C). Second: per-gear top speed learned live (`[f32; 10]`).
7. **Notifications and pop-ups** (R12). ★ **Event toasts in the pane**, Live Activity style, at most one at a time, under 2 s: a new top speed, a best 0 to 100 time, a new best lap (`race_progress`, decoded, offset confirmed at check C, races only), "limiter" count (derived, rare, never flashing). Second: the island itself blooms for an event; rejected for now because the island is rank 1 and must mean revs only.
8. **Cruise dot and limiter line** (R10). ★ **Top-speed dot**: the session's highest speed as a dot on the speed arc with its value beside it, using Apple's cruise grammar (derived). Second: a line on the speed arc at this gear's learned speed at the shift point ("you will shift here").
9. **Road signage and drive mode.** ★ Dropped; the alternate-unit line takes the reserved space. Second: the gear's learned speed at shift as the "sign".
10. **ADAS view and rear camera.** ★ In reverse the pane shows the reversing speed large and the island stays a dim pill (no lamps can light below idle). Second: none; no picture of the road is sent.
11. **Range.** ★ **Time to shift**: (shift_rpm − rpm) ÷ rpm_rate while it is between 0 and 3 s, as a small countdown (derived prediction, labelled EST). Second: a fuel range derived from `fuel` and distance_traveled (both decoded, offset confirmed at check C): the distance covered per unit of fuel used this session, times the fuel left, labelled EST and shown only once fuel has fallen by a measurable step; the game gives no range of its own.
12. **Wallpapers** (gradients, landscapes, topographical, R3). ★ True black; the glass's studio reflection is the only "wallpaper". Second: a topographic field of thin isolines at the static-part luminance cap whose relief is the session's speed trace; risky beside `VisionSpatialGlass`'s tunnel and clouds, so only if the board wants it.

## 6. Edge cases

- **Max rpm 0 or tiny:** `max_rpm` is clamped to 1 in `SignalSmoother`; the dash hides the rev arc and island lamps when max_rpm < 1,000 and keeps the speed and gear.
- **Idle near max** (window under 500 rpm): the island uses fewer seats (the window divides into what fits) and never lights more than it can stage; the strobe still starts at `shift_rpm`.
- **12,000+ rpm and diesels at 4,500:** numerals every 1,000 or 2,000 rpm, chosen so 6 to 9 labels fit the arc; the island window follows `shift_rpm`, so it works the same.
- **EVs** (one gear, idle 0 or odd rpm): an EV is known by `cylinders` 0 (decoded, offset confirmed at check C), with "`idle_rpm` 0 or rpm never settles" as the fallback test; the rev arc then starts at 0; with no shifts the island shows power share instead of revs only if step 1 confirms EVs reach `shift_rpm`; otherwise it stays a pill and the limiter strobe is the only cue. The fuel arc becomes a charge arc (Apple's "state of charge gauge", R10) only if check C shows `fuel` moving on an EV; otherwise it is hidden on an EV.
- **Cylinders 0 or odd values:** 0 is an EV (above); the firing-frequency meter falls back to crank Hz when cylinders is 0, negative or above 16, and a rotary or two-stroke would make the four-stroke formula wrong, so that meter stays a second option.
- **Fuel 0 or missing:** if `fuel` reads exactly 0 for the first 2 s with race on, or never changes from 0, treat it as not sent and show the ENERGY arc in its place (labelled ENERGY), never a red empty tank. A real reading below the critical mark turns the arc `warning`, steady, never flashing.
- **Fuel that never drops:** whether FH6 uses fuel at all is unconfirmed; a constant reading is shown as it is (a full tank is true data), and the derived range stays hidden until fuel has fallen by a measurable step.
- **Tyre temperatures before warm-up:** cold tyres sit at the cold end of the TYRE arc in the neutral role, with no warning colour; the hot end alone takes `warning`. All four at 0 is read as not sent: the arc falls back to LOAD.
- **No laps** (free roam, or `lap_number` and `current_lap` both 0): the lap rows of the trip page and the best-lap toast are hidden; the session trip carries on. A best_lap of 0 means no lap yet, never a 0:00.000 record.
- **Reverse and neutral:** R and N in the pane; no lamps; the speed digits stay positive.
- **Gear above 10:** two digits must fit the pane's flat top (Saira Condensed 700, cap 170); the per-gear table holds 10 and gears above share the last slot.
- **mph and km/h, hp and kW:** from `unit` and `power_unit`; the alternate-unit line shows the other speed unit.
- **Limiter bounce of about 50 rpm a frame:** digits through `Hysteresis`; the seat count gets a band of half a seat so the island does not shiver; the strobe clock starts at the shift point with its first 50 ms lit (family rule).
- **Spikes and crashes (huge G):** parallax and the G-dot clamp at 1.5 g (tokens `gBall.ringsG`); the island clamp holds 200 to 800 px whatever the spring does.
- **Airborne:** G near zero, revs flare: the island may reach the shift stage; this is true data and stays.
- **Stale or paused stream:** the app hides the dash; on return the springs restart from the last value, not from zero.
- **Palette extremes and random palettes:** lamps and arcs take `primary` and `shift`; clear glass carries no hue lean over numerals; lamp intensity is checked against black at the darkest random primary so seats stay visible.
- **Font swap:** the gear must fit the pane's flat top (where the lens tilts under 22°) at the user's font's widest digits; the speed keeps fixed digit advance.
- **Cockpit only:** no hood variant is listed.
- **Reduced flash:** the strobe becomes the 2 Hz breath at 40% depth (tokens `reducedFlash`); the upshift ripple is left out.
- **Colour-only cues and colour vision:** the limiter is also shown by shape (all ten seats lit, island at full width), never by colour alone; the throttle fallback ring has a text label.

## 7. Signature moments

**Limiter / shift strobe** (only the island's lamps strobe, 10 Hz at 50%, 2 Hz breath gentle):
- **L1 ★ Brief's island.** Glass stretches seat by seat to 800 px; at the shift point all ten are lit and strobe; both dial arcs hold steady `shift`. Grounded in the Dynamic Island growing "horizontally" (WWDC23 10194) and in R14 (the car, not the phone, owns the over-limit state). With the feed-forward of §4 so the lamps are not late.
- **L2 Fuse to the pane.** On the limiter the island's blend length rises until a neck of glass reaches down to the pane (k from about 26 to 58 px over 150 ms), the family's "pieces fuse into one bar" rule turned vertical. Nothing flashes but the lamps. Risk: the neck crosses the gap above the gear; check it never covers the numeral.
- **L3 Recommended-shift chevron.** At the shift point a drawn chevron (our own glyph, not an Apple symbol) appears between the two centre seats, from Apple's "arrows for recommended gear changes" (R11). Steady, not strobing; pairs with L1.

**Gear change:**
- **G1 ★ Let go and ripple.** On an upshift the island returns to the pill in one motion on the ω 14 spring (velocity kept, WWDC23 10158), and a ripple runs out through the pane's rim while the numeral stays still (family rule). The new gear arrives with the tokens' `gearChange` incoming spring.
- **G2 Numeric roll.** The gear rolls up (upshift) or down (downshift) as Apple's numeric content transition does (WWDC23 10194). Conflicts with the family's "numeral stays still", so only if the board shows it reads faster.
- **G3 Swoop.** A droplet of glass leaves the island and drops into the pane, from the Dynamic Island "swoops up" language (WWDC23 10194), reversed. Rich, but two moving glass pieces at the moment of the shift may be one too many.

## 8. What it needs from the 4K library

The pre-build made **no kit** for this dash: `kits.rs` and PREBUILD.md list `CarplayIsland` among dashes that "reuse the existing `liquid_glass` studio plate (seed 77203)". Parts it can reuse:
- `assets/clusters/liquid_glass/studio_env.png` (Blender studio, octahedral, with the ComfyUI plate seed 77203 on its cyclorama): what every glass rim mirrors.
- `assets/glass/blue_noise_128.png`: dither for the arcs' gradients.
- Fonts in `assets/fonts` (Saira variable for 300, Saira Condensed Bold 700, Titillium Web SemiBold 600); the kit builds MSDF glyphs at load (`fh6-kit/src/text.rs`), so no atlas files are needed.
- The glass itself is computed by `OpticalGlass` (optics.md); the brief's "LiquidGlass sapphire normal map" does not exist in the rebuilt pilot, and the legacy baked normals and fixed-IOR caustics in `assets/glass/` belong to the rejected slab glass and are not used.

The cluster prompt requires at least one bespoke ComfyUI asset and one bespoke Blender asset per dash; the brief has none.

LIBRARY GAP: a bespoke Blender studio environment for this dash, 4096² octahedral (and the six cube faces), Cycles, with one long thin horizontal key strip wide enough that the island's whole top rim mirrors it as one continuous line at every width, plus the family's horizon ring; why: the island's stretch must read by its moving rim highlight on black; tool: Blender via asset-forge (`hero` studio rig).
LIBRARY GAP: a bespoke ComfyUI cyclorama plate for that studio, 4096 × 2048 seamless panorama of a dark car cabin at night with a few warm practical lights; why: the bespoke ComfyUI asset the prompt requires, seen only as faint detail in the rims; tool: ComfyUI SDXL via asset-forge.

## 9. Departures and open questions for the board

1. **Q1 Seats behind the glass.** The brief's "seats follow the glass" makes the shift lights about 143 ms late in a pull (§4). Options: (a) ★ feed the spring target forward by rpm_rate × 2 ÷ ω; (b) seats light from rpm at once and the glass chases them; (c) raise ω (ω = 28 halves the lag but feels less like Apple's 0.45 s spring).
2. **No needle.** The Aston theme uses a rising band (R4), so the arc is right; an analogue needle (R8) is a later option only.
3. **The panel between the dials** is real (R6) but flat; here it becomes a glass squircle at middle depth. Its corner radius should be concentric with the widgets under it (HIG, WWDC23 10194).
4. **Which dial holds the gear.** The real round gauge's centre holds transmission state (R11); the brief puts the gear in the pane. Keep the pane (rank 1 to 2 spacing of 391 px); leave the tach centre empty or for the logo.
5. **Activity rings.** The three-ring widget borrows Apple Watch's ring grammar; colours must come from palette roles, not Apple's red, green and blue, and no Apple marks appear anywhere (brief's Risk line).
6. **Green livery and script.** Not copied: the dash is for any car, and the logo zone shows the driven car's logo.
7. **Depth order.** Apple stacks dynamic content behind non-critical gauge parts (R12); the board can place the widgets so the dial ticks pass in front of their edges, which would prove depth without parallax.
8. **Six layouts versus one.** The real cluster swaps layouts by swipe (R5, R7); the dash has one layout of separate zones, which the configurator can rearrange.
9. **Island height.** The family caps a strobing strip at 42 px; the island stays 40 px and never "blooms" taller.
10. **Sibling check.** `AmgMbuxClassic` also has two round dials; the glass pane, the island and the rings must make this dash's thumbnail different.
11. **Fuel and temperature gauges (updated 2026-11-28).** The fuel gauge is no longer a departure: Apple's mandatory speed and fuel pair (R10) is kept with the decoded `fuel` field, and ENERGY is only its fallback. The temperature gauge stays a departure in what it measures: it reads the tyres (TYRE), because engine temperature is not in the packet. The board places both short arcs so they do not crowd the island or the pane.

## 10. Sources

Apple:
- Apple Newsroom, "CarPlay Ultra, the next generation of CarPlay, begins rolling out today", 15 May 2025: https://www.apple.com/newsroom/2025/05/carplay-ultra-the-next-generation-of-carplay-begins-rolling-out-today/
- WWDC24 10112, "Say hello to the next generation of CarPlay design system": https://developer.apple.com/videos/play/wwdc2024/10112/
- WWDC24 10111, "Meet the next generation of CarPlay architecture": https://developer.apple.com/wwdc24/10111/
- WWDC23 10194, "Design dynamic Live Activities": https://developer.apple.com/videos/play/wwdc2023/10194
- WWDC23 10158, "Animate with springs": https://developer.apple.com/videos/play/wwdc2023/10158/
- HIG, Live Activities: https://developer.apple.com/design/human-interface-guidelines/live-activities
- WWDC25 219, 323, 356 and the HIG Materials page, as cited in `crates/fh6-clusters/src/clusters/liquid_glass/RESEARCH.md`

Aston Martin:
- "A world first as Aston Martin debuts next-generation Apple CarPlay Ultra": https://www.astonmartin.com/en/our-world/brand-stories/aston-martin-and-apple-carplay-ultra
- Press release (not re-read, budget): https://www.automotiveworld.com/news-releases/aston-martin-debuts-apple-carplay-ultra-for-iphone-users/

Reviews and hands-on:
- Top Gear: https://www.topgear.com/car-news/tech/new-apple-carplay-ultra-tested-hands-every-new-feature
- PistonHeads: https://www.pistonheads.com/news/ph-britishcars/hands-on-with-apple-carplay-ultra/49948
- Auto Express: https://www.autoexpress.co.uk/features/367055/apple-carplay-ultra-review-next-generation-car-smartphone-mirroring
- AppleInsider: https://appleinsider.com/articles/25/05/15/carplay-ultras-first-trial-reveals-a-deeply-integrated-apple-like-experience
- TechRadar: https://www.techradar.com/vehicle-tech/hybrid-electric-vehicles/ive-tried-apple-carplay-ultra-it-fixes-everything-thats-irritating-about-carplay-but-theres-a-catch
- SlashGear (a week with it): https://www.slashgear.com/1919080/new-apple-carplay-ultra-hands-on-test-first-impressions
- Motor1: https://www.motor1.com/reviews/764192/apple-carplay-ultra-review/
- 9to5Mac: https://9to5mac.com/2025/06/30/carplay-ultra-hands-on-reveals-its-standout-best-new-feature/
- Cars.com: https://www.cars.com/articles/hands-on-with-apple-carplay-ultra-the-new-killer-app-512357
- Wallpaper: https://www.wallpaper.com/transportation/the-next-generation-apple-carplay-ultra-makes-its-debut-in-aston-martins-dbx707
- MacRumors on iOS 26 CarPlay: https://www.macrumors.com/2025/06/13/everything-new-for-carplay-on-ios-26/
- Videos to watch at step 1 (Top Gear and Aston Martin hands-on, collected by iDownloadBlog): https://www.idownloadblog.com/2025/05/15/carplay-ultra-hands-on-video-aston-martin-top-gear/
- Maker status in 2026 (titles seen in search, not read): https://macrumors.com/2026/08/19/apple-says-carplay-ultra-coming-to-these-vehicles/

Repo: `docs/specs/optics.md` §2, §4; `data/design_tokens.json` (`motion.springs`, `reducedFlash`, `gearChange`); `crates/fh6-kit/src/cluster.rs` (`Signals`); `docs/design/families/liquid-glass.md`; earlier web check `/mnt/project-files/dash-research/webcheck/CarplayIsland.md` (leads only).
