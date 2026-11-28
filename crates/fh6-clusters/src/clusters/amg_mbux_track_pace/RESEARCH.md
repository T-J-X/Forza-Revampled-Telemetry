# AmgMbuxTrackPace: research note

Build #29, catalog row 57, category Supercar, Target `recreation`, family Modern Digital 2027, views cockpit and hood. Pre-researched 2026-11-28 by the thread "Research the upcoming dashes ahead" before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used throughout: **Cited** means read from the linked source on 2026-11-28; **Computed** means worked out here from cited inputs, with the numbers shown so a test can assert them; **Our reading** means a conclusion drawn for this dash, open to the direction check. "Check C" is the real-game capture at the PC that confirms the offsets of the newly decoded packet fields.

Read first: the `Shared AMG MBUX notes` in `amg_mbux_superscreen/RESEARCH.md` (branch `claude/project-thread-aig4a0`) and the Supersport note (`amg_mbux_supersport/RESEARCH.md`, branch `claude/project-thread-yw1el9`). This note does not repeat their palette, type and motion rules; it says where it departs from them.

## Deliberate changes (read first)

The catalog's AMG rules ask every AMG note to open with these.

1. **No flashing area outside the shift strip.** Only the 12-cell upshift bar strobes (10 Hz, 50% duty; 2 Hz breath in reduced flash). Nothing large flashes, including the green and red delta states and the cabin-light lean.
2. **Deltas by role, with a sign and a direction.** Ahead is `go`, behind is `warning`, always with `+` or `−` and an arrow or chevron, never colour alone. The real system uses green and red only (Cited, see 1.6).
3. **No maker's marks.** The logo zone shows the driven car's logo or the custom name. No AMG, Mercedes or star mark is drawn, and no asset carries one.
4. **Palette roles.** Default palette is the shared `amgMbux` family default. TRACK PACE is not a red style in any source read, so red is used only where the real one uses it: the G-meter maxima (Cited, 1.3) and the `warning` delta. The yellow manual `M` uses `primary_text`, as the shared notes say.
5. **A swappable font.** Barlow Semi Condensed (OFL), the AMG family face set by the shared notes, replaces the brief's Saira and Titillium so the five AMG dashes read as one maker's screens. The user's display font replaces the numerals.
6. **No housing slab.** The parts float on OLED black, as for Supersport (its change 6, `bezel::over_clear`).
7. **What the game cannot send is dropped and replaced** (section 5): stored circuits, tyre pressures, oil and transmission temperatures, aero position, video, the AR ghost car and race navigation each get a telemetry-driven replacement.

## 1. The real display

### What it is
The **TRACK PACE menu (display style) of the second-generation MBUX driver display**: the 12.3-inch screen in a visor on the Mercedes-AMG GT (C192), SL (R232), C 43 / C 63 (W206) and the W214-step CLE 53. It is the cluster face of AMG TRACK PACE, a data logger built into MBUX.

### Feature inventory (text sources; the drawn form waits for frame grabs)

| # | Feature | Status and source |
|---|---|---|
| 1.1 | Three parts: **1 G-meter, 2 centre display content (example: gear display), 3 telemetry display (example: selected race track)**. | **Cited**, MY24 AMG C Sedan supplement p. 36 figure legend ([MBUSA PDF](https://Mbusa.Com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20C%20Sedan%20Owners%20Manual.pdf)); same legend in the CLE 53 supplement p. 27 ([MBUSA PDF](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf)) |
| 1.2 | Menu options: G-meter, race track selected, braking and acceleration procedure, gear currently selected. The centre follows the TRACK PACE settings: gear display, braking points, acceleration, quarter-miles, selected race track, current race, recorded route. The menu is "intended only for closed race tracks". | **Cited**, C Sedan supplement p. 36 |
| 1.3 | The G-meter shows "the forces that are exerted on the vehicle occupants both laterally and in the direction of travel"; "The maximum values will be shown in red in the coordinate system." | **Cited**, C Sedan p. 36; CLE p. 27 (wording "will appear in red") |
| 1.4 | Driver display menus on these cars: Classic, Understated, Service, Supersport, Sport, Track Pace (C-Class); press lists five styles Classic, Sport, Supersport, TRACK PACE, Discreet (AMG GT, SL). | **Cited**, C Sedan pp. 32 to 36; press texts in `amg-trackpace-one-superscreen.json` |
| 1.5 | Lap and sector times are shown "on the multimedia display, head-up display and in the instrument cluster". TRACK PACE records more than 80 values (speed, acceleration, steering angle, brake pedal) ten times a second; 40 parameters live on the centre Telemetry screen, up to four at once. | **Cited**, AMG GT Coupé press kit, Aug 2023 ([Mercedes-Benz CH PDF](https://media.mercedes-benz.ch/download/9595ea6c-ddc6-465b-aaac-1d1fc4e7fc57/20230819-en-pi-mercedes-amg-gt-coupeacute.pdf)) |
| 1.6 | Elements turn **green or red** against the reference lap; a reviewer saw "relative sector times in both the driver display and the head-up display", and the ambient light "glows green or red depending on which side of the timeline you're on". | **Cited**, [PistonHeads, 6 Nov 2022](https://www.pistonheads.com/news/blog/ghost-in-the-machine--ph-footnote/46434); ambient light also in the supplements ("lit in red or green depending on delta time") and [T3, 12 Nov 2022](https://www.t3.com/features/i-tried-the-mercedes-amg-track-pace-app-and-it-made-me-faster-around-the-circuit) |
| 1.7 | Time is captured "bis auf die Zehntelsekunde" (to a tenth of a second), and a visualisation shows "auf einen Blick" whether the driver is within the target time. | **Cited**, [mbpassion, 26 Apr 2021](https://mbpassion.de/2021/04/amg-track-pace-nun-als-on-demand-ausstattung/) |
| 1.8 | Timing starts by itself at the start line; sectors are set while recording a track; a different session can be the reference ("Add Recording"); the system notices a left or shortened circuit; 60 tracks pre-stored (T3). | **Cited**, C Sedan pp. 41 to 42; T3; press text in the leads json |
| 1.9 | Drag Race: acceleration, quarter mile, braking (braking in 5 mph / 10 km/h steps to standstill; distance at most one mile). | **Cited**, C Sedan p. 43; AMG CLA 2022 manual (leads json) |
| 1.10 | Head-up display TRACK PACE content: speed and gear, lap and sector times, acceleration and braking, track layout graphics; the GT's HUD "shows cornering angles and braking points". | **Cited**, AMG CLA 2022 manual (leads json); GT press kit |
| 1.11 | Manual mode shows a yellow `M`; an upshift bar appears on the driver's display or HUD in manual mode. | **Cited** in the shared AMG notes and the Supersport note (Mercedes online manual) |
| 1.12 | Earlier generations: first-generation MBUX Supersport put TRACK PACE in the **left-hand section** of the display, with Warm-up, Engine data, SETUP and G-meter on the right; two Drive photos show it (a Nürburgring GP map with tyre temperatures on an A 35, a "Start TRACK PACE" prompt on a GT 63 S 4-Door). | **Cited**, Mercedes online manual, AMG GT 4-Door (leads json); photos listed in section 2 |

**Not in any text source (take from frame grabs, section 2.1):** the drawn form of the G-meter (rings, cross-hair, axis values, dot), how lap time, delta and sectors are laid out, whether the track part draws a map and what it looks like, whether a rev band or speed figure is part of the menu, whether the style follows the drive program's colourway (blue in Comfort/Sport, red in Sport+/Race, as Supersport does), and the typeface. **Our reading** from the Supersport G-force page (gen 2, same screen and same family): the G-meter is likely a polar target with rings at 0.5, 1.0 and 1.5 g, a red dot and red maxima, which the Supersport note measured; step 1 must confirm it on the TRACK PACE menu itself.

**Better target?** None found. TRACK PACE is the only AMG style whose whole job is lap timing, and the catalog reserves the driven-line map for it. The first-generation left-section layout (1.12) is a fallback reference if no gen-2 frame is found.

## 2. Reference photos

No second-generation TRACK PACE cluster photo was found by text search: every gallery and owner thread read shows Supersport, Classic or the centre-screen app. That is the main gap, and why the prebuild says this dash waits for its own frame grabs. The tools here cannot view or save images, so each row is described from its page, caption or file name.

| # | What it shows | Direct image URL (where known) | Source page |
|---|---|---|---|
| R1 | Official Track Race screen of the app (centre display) | https://mbpassion.de/wp-cb4ef-content/uploads/de-amg-track-pace-track-race-xl.jpg | [mbpassion 2021](https://mbpassion.de/2021/04/amg-track-pace-nun-als-on-demand-ausstattung/) |
| R2 | Official Drag Race screen | https://mbpassion.de/wp-cb4ef-content/uploads/de-amg-track-pace-drag-race-xl.jpg | same |
| R3 | Official analysis view (lead image, 1440 × 810) | https://mbpassion.de/wp-cb4ef-content/uploads/de-amg-track-pace-analyse-xl.jpg | same |
| R4 | Official TRACK PACE page image, 1920 × 1080 | https://www.mercedes-amg.com/media/images/145a75e1ed796599b2621ee5cd168fadddd3419b-1920x1080.jpg | [mercedes-amg.com](https://www.mercedes-amg.com/en/amg-track-pace) |
| R5 | Gen 1: A 35 Supersport with TRACK PACE Nürburgring GP map and tyre temperatures | Drive file `gen1_a35_w177_supersport_trackpace_nurburgring_01.jpg` in `amg_mbux_supersport/references` | [reddit r/mercedes_benz](https://www.reddit.com/r/mercedes_benz/comments/f2hksu/anyway_at_all_to_get_this_digital_interface_on_a/) |
| R6 | Gen 1: GT 63 S 4-Door Supersport, "Start TRACK PACE" in the right part | Drive file `gen1_gt63s_4door_x290_supersport_trackpace_idle_01.jpg` (same folder) | [vicariousmag](https://www.vicariousmag.com/road-test-2019-mercedes-amg-gt-63-s-4-door/) |
| R7 | Gen 2: AMG GT 63 Supersport G-force page (polar G target left, speed / gas / brake right) | Drive file `gt63_c192_supersport_gforce_page_01.jpg` (same folder) | [MotorWeek](https://motorweek.org/road-tests/track-tests/2024-mercedes-amg-gt-63-coupe/) |
| R8 | Gen 2: SL 55 Supersport, G-force cross-hair in the centre | Drive file `sl55_r232_supersport_gforce_centre_blue_01.jpg` (same folder) | [CarBuzz](https://carbuzz.com/reviews/driven-2022-mercedes-amg-sl-55-proves-that-you-dont-need-the-63/) |
| R9 | Gen 2: AMG GT 63 Classic style, for type and panel finish | Drive file `gt63_c192_classic_style_compare_01.jpg` (same folder) | [AOL](https://www.aol.com/articles/peek-inside-mercedes-amg-gt63-132500000.html) |
| R10 | Gen 2: SL 63 style selector carousel (shows the menu that holds Track Pace) | Drive file `sl63_r232_style_menu_classic_highlighted_01.jpg` (same folder) | [cars.com](https://www.cars.com/articles/2023-mercedes-amg-sl63-review-dynamic-droptop-coddler-473658/) |
| R11 | Manual figure: Track Pace menu with callouts 1 to 3 | PDF page 36 | [MY24 AMG C Sedan supplement](https://Mbusa.Com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20C%20Sedan%20Owners%20Manual.pdf) |
| R12 | Manual figure: TRACK PACE menu, CLE 53 | PDF page 27 | [MY24 AMG CLE Coupe supplement](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf) |

Drive references folder: https://drive.google.com/drive/folders/1KI9uhfS3Um2DMvHYS2sIybgKC48OnXWx (its `SOURCES.md` lists these rows and the videos below for the PC to fetch).

### 2.1 Frames step 1 must grab
Video candidates (titles from search results; content not opened, so each must be checked): [C 63 S E Performance BOOST strategy demonstration](https://www.youtube.com/watch?v=t4oT2V8OHJE) (gen 2; boost strategy is a TRACK PACE feature, PistonHeads), [Mercedes AMG SL 55 2024](https://www.youtube.com/watch?v=RTo9uos3cZk) (returned for a Track Pace cluster query), [SL 55 display options](https://www.youtube.com/watch?v=w-lHb2vPOv8) (from the Supersport note), [2021 C63s Track Pace demo](https://www.youtube.com/watch?v=q9vsdx52PuE) and [A 35 TRACK PACE](https://www.youtube.com/watch?v=aUu_Tx1TVQ4) (gen 1, lineage), [Track Pace stoplight sprint, NL](https://www.youtube.com/watch?v=Z3GmQFsFp00) (Drag Race). Grab these frames, in this order of value:

1. **F1, the TRACK PACE style at rest, gen 2, full screen, square to the lens.** Measures the three parts' boxes, the G-meter form, the centre gear, the telemetry part, any rev band or speed figure, and the type.
2. **F2, a running lap on track**: current lap time, delta, sector chips, the map with the car marker. Hold two frames: one ahead (green), one behind (red), to read how much area changes colour.
3. **F3, a sector crossing and a lap crossing**: what freezes, for how long (count frames at the video's rate), how the delta enters.
4. **F4, the G-meter while cornering and braking, and after**: dot style, trail if any, how the red maxima are drawn (ticks, a polygon, or numbers).
5. **F5, the selected-track map in the telemetry part**: line weight, start/finish mark, sector marks, car dot, heading-up or north-up.
6. **F6, high revs and an upshift in manual**: whether the upshift bar shows in this style, the `M` tag, the gear animation.
7. **F7, a Drag Race run in the cluster** (acceleration or quarter mile): the timer layout.
8. **F8, the style in the red Sport+/Race colourway**, if the style follows the drive program.
9. **F9, the switch into TRACK PACE** from another style (the transition).
10. **F10, the HUD TRACK PACE content** (for the hood view) and **F11, the cabin light green and red** (hue, how fast it changes).

## 3. How the real thing behaves, with motion notes

Real timing is documented nowhere read; durations below are proposals tied to house springs (shared notes, "Motion timing") until F2 to F9 give real ones.

| State | Real (source) | The dash |
|---|---|---|
| Start-up, style switch | Not documented (F9) | Parts fade in on the house boot timing; the map draws its stored best lap from start to end over 600 ms if one exists, else shows `TRAIL` and dashes. |
| Idle, parked | Timing waits for the start line (1.8) | G dot centred, maxima held, lap column shows `--:--.-`, gear `N` or the gear with `M`. |
| Cruise | Delta "constantly updated" (A 45 test, leads json) | Delta updates at 10 Hz (the logger's rate, 1.5) with 0.05 s hysteresis on the shown tenth, so the figure does not shimmer. |
| Pull to the limiter | Upshift bar in manual (1.11) | Bar fills outside-in from the per-car thresholds; the G dot moves back with longitudinal G. |
| Limiter | Not documented | Only the upshift bar strobes (deliberate change 1). |
| Gear change | Not documented (F6) | `HeroGear` 1.12 → 1.0; a shift tick is stored on the trail (section 7). |
| Braking | G-meter shows occupant force (1.3) | The dot runs forward-down with the `gBall` spring (T 0.168 s, ζ 0.9, `data/design_tokens.json`); a new maximum turns its tick `warning` red and holds. |
| Drift, lateral G | Maxima red (1.3); TRACK PACE logs drift angle ([mercedes-amg.com](https://www.mercedes-amg.com/en/amg-track-pace)) | Dot sideways; a slim slip arc shows β (section 5). |
| Sector and lap crossing | Green or red against the reference (1.6) | The delta swing (section 7). |
| Reverse, neutral | Not documented | `R` in the `warning` role (tokens), `N`; timers pause, the trail keeps logging. |
| Shut-down, menus, pause | Data Out stops in menus, pauses, replays, rewinds and "after finishing a race" (**Cited**, [FH6 Data Out](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation)) | The host hides the dash on a stale stream; the dash keeps its buffers so the best lap survives a pause. |

## 4. The maths

The dash has no row in "Real geometry and maths". It has timing and geometry, and its tests should assert these (all **Computed**, `python3`):

- **Distances.** Quarter mile 402.336 m; eighth mile 201.168 m; 100 km/h = 27.778 m/s; 200 km/h = 55.556 m/s; 60 mph = 26.822 m/s.
- **Crossing interpolation.** At 60 Hz a frame is 16.667 ms; at 1 g the speed changes 0.1634 m/s per frame. A timer that stops on the first frame past a target is late by up to one frame, so each timer interpolates linearly between the two samples that bracket the target: t* = t₀ + (target − x₀)(t₁ − t₀)/(x₁ − x₀). Test: samples (t 0.000 s, v 27.0) and (t 0.016667 s, v 28.0) cross 27.778 at 0.012963 s.
- **Timestamp.** `TimestampMS` is a U32 that "can overflow to 0" (FH6 Data Out): it wraps after 49.71 days, so every difference uses `wrapping_sub`.
- **Braking from 100.** At 0.8, 1.2 and 1.6 g: 3.54, 2.36 and 1.77 s; 49.2, 32.8 and 24.6 m. The real Drag Race steps braking in 10 km/h bands (1.9), so the dash can show the 100 to 0 total and the band splits.
- **Lap buffers.** Sampling the driven line every 5 m, 4,096 samples hold 20.48 km (every FH6 circuit and the Nordschleife length); at 2 m they hold 8.19 km. One buffer of (x, z, t) in f32 is 49,152 bytes; the dash needs two (current and best), fixed size, no allocation.
- **Lap closure** (brief): the line returns within 15 m of its own start and the heading agrees within 30° (cos 30° = 0.8660), after at least 500 m and 20 s so a parked car or a U-turn never closes a lap. At 250 km/h a 60 Hz frame moves 1.157 m, so a 15 m gate is crossed in about 13 frames and cannot be missed.
- **Delta by projection, not by distance.** delta = t_cur − t_ref(p), where p is the current position projected onto the reference polyline, searched in a window of ±32 samples round the last match. Distance travelled alone makes a shorter line look ahead; the projection does not. Test: on a straight reference sampled every 5 m at 50 m/s (0.1 s per sample), a car at x = 102.5 m at t = 2.10 s has t_ref = 2.05 s and delta +0.05 s.
- **Map decimation.** The brief caps the map at 240 points. A uniform stride over a 5 km lap makes 20.8 m chords: a 15 m hairpin loses 4.21 m (sagitta), a 30 m one 1.87 m; over a 10 km lap the 15 m hairpin vanishes. So decimate once at lap close with Douglas-Peucker to 240 points (fixed scratch arrays), never by stride. At 0.25 px per m (a 1,200 m wide circuit in a 300 px box) 1.87 m is 0.47 px.
- **G-meter scale.** With rings at 0.5, 1.0 and 1.5 g and the edge at 2.0 g, a 300 px meter puts the rings at r 37.5, 75 and 112.5 px. g = a ÷ 9.80665.

**Catalog correction:** the brief's Telemetry line says "The game's lap and sector fields are not decoded". Since main 60c99c9 `best_lap`, `last_lap`, `current_lap`, `current_race_time`, `lap_number` and `race_position` are decoded (offset confirmed at check C). The line should read: use the game's lap fields when they are non-zero (a circuit race), and the self-closing trail lap otherwise. There are no sector fields in the packet. Also, `normalized_driving_line` is not the driven-line map (see 5).

## 5. Telemetry mapping

Fields from `crates/fh6-core/src/telemetry.rs` and `Signals` (`crates/fh6-kit/src/cluster.rs`). "Verified" = long-verified against the golden packets; "decoded, check C" = decoded, offset confirmed at check C.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| G dot | `signals.g_lat`, `g_long` (acceleration_x, _z) | verified | ÷ 9.80665, clamp to 2.0 g, `gBall` spring. Sign so the dot shows the force on the occupant (1.3): braking moves it forward; confirm the real direction in F4. |
| G maxima | same | verified | Per-quadrant held peaks after 150 ms; reset on `car_ordinal` change or an OSD reset. |
| Vertical G (optional chip) | acceleration_y (`g_vert`) | verified | TRACK PACE logs vertical acceleration (mercedes-amg.com); a small value under the meter. |
| Slip arc | velocity_x, _z | verified | β = atan2(v_x, v_z), shown only above 3 m/s. |
| Gear and `M` | `signals.gear` | verified | `Gear::label()`; `M` always on. |
| Speed | `signals.speed`, `unit` | verified | Tabular, 0.6 hysteresis. |
| Upshift bar, rev band | `rpm`, `idle_rpm`, `max_rpm`, `shift_rpm` | verified | Per-car, per-gear thresholds, outside-in. |
| Lap time, last, best | `current_lap`, `last_lap`, `best_lap` | decoded, check C | Shown when non-zero ("0.0 if not applicable", FH6 Data Out). Else the dash's own timer from `timestamp_ms`. |
| Lap count, position | `lap_number`, `race_position` | decoded, check C | `L3`, `P2` chips, only in a race. |
| Driven line, map | `position_x`, `_z` | decoded, check C (Y-up to check) | 5 m samples into the fixed buffers; heading-up by the trail's own direction (no yaw needed); `TRAIL` until a lap closes. |
| Delta, sectors | position + timestamp vs the best lap | derived | Projection delta (section 4); three equal-distance sectors of the reference lap. |
| Lap distance | `distance_traveled` | decoded, check C | Fallback: ∫ speed dt. |
| Drag timers | `speed_ms`, `timestamp_ms` | verified | Interpolated crossings; armed when speed < 0.5 m/s. |
| Line coach | `normalized_driving_line` (−127..127) | decoded, check C | **Our reading:** the car's lateral place against the game's driving line; a small bar under the map. May read 0 when the line assist is off. |
| Brake coach | `normalized_ai_brake_difference` (−127..127) | decoded, check C | **Our reading:** the player's braking against the AI's; a two-sided bar, `go` near 0. |
| Tyre quad | `tire_temp_*` | decoded, check C (unit not stated) | The Track Race "Tire temperature" widget (1.12); calibrate per car from the session's range. |

### What the game cannot send
- **Stored circuits, start lines and track outlines** (60 tracks, 1.8). **Replacement (recommended): the self-closing trail lap**, drawn as the map and labelled `TRAIL`, which becomes `LAP` after one closed loop; in a circuit race the game's lap fields time it exactly. Second idea: shift and braking ticks stored on the trail, so the map reads as the driver's own analysis view.
- **Race navigation, braking points and cornering angles** (HUD, 1.10). **Replacement (recommended): braking points from the best lap**, where the brake first passed 0.3, shown as ticks on the map and a "BRAKE 80 m" countdown on the next approach, labelled `REF` (exact, from the driver's own lap). Second: the AI brake difference bar (decoded, check C).
- **AR ghost car on the camera feed.** **Replacement (recommended): the ghost dot** on the map where the best lap was at the same elapsed time (brief).
- **Tyre pressures, oil and transmission temperatures, aero position, suspension data** (mercedes-amg.com). **Replacement (recommended): tyre temperature quad** (decoded). Second: combined slip per wheel as a grip-use quad (decoded, check C), or suspension travel bars.
- **Cabin ambient light green or red** (1.6). **Replacement (recommended): the kit's binnacle wash leans to `go` or `warning` with the delta**, a steady ramp, alpha within the wash's 12 to 35 of 255 (scrim.rs), off with the ambient toggle. It needs a kit hook (the host draws the wash in `primary`).
- **Video, acoustic sector feedback, boost strategy, fuel and clock row.** Dropped. Fuel is decoded (check C) and may fill a thin bar if the board keeps a status row.
- **Sector definitions.** **Replacement (recommended): three equal-distance sectors** of the reference lap. Second: sectors split at the two longest braking zones.

## 6. Edge cases

| Case | The dash does |
|---|---|
| `max_rpm` 0 or tiny | Bar parks dark; timers and G still work. |
| Idle near max, 12,000+ rpm, diesel 4,500 | Thresholds follow the car (TOKENS shift fractions); nothing scales to a fixed 9,000. |
| EV (one gear, odd rpm) | Bar follows rpm fraction; the gear shows `1` or `D` as `Gear::label()` gives. |
| Reverse, neutral, gear above 10 | `R` in `warning`, `N`, raw 11+ is neutral; timers pause in reverse. |
| mph and km/h | Drag targets switch: 0 to 60 mph and 0 to 100 km/h; quarter mile in both. |
| Limiter bounce (about 50 rpm a frame) | Bar uses the 60 rpm backlash of the shared notes. |
| Lap fields zero | Own timer; the game's fields win the moment they are non-zero. |
| Lap fields jump (restart, rewind) | `current_lap` falling by more than 1 s or `lap_number` falling resets the current lap, keeps the best. |
| Free roam, no loop | Map stays `TRAIL` (last 240 samples); timer dashes. |
| Teleport (position jump > 100 m in a frame) | Ends the current lap without a result; the trail restarts (as `trail.rs` does at 100 m). |
| Huge G spike, crash | Peaks taken only after 150 ms; clamp 2.0 g on the dot. |
| Airborne | g near 0; the dot rests at centre; laps keep timing. |
| Stale stream, end of race | Host hides the dash; buffers kept. |
| Car change | Best lap reset (a lap is car-specific); G maxima reset. |
| Palette extremes, random palettes | Delta stays on fixed `go` and `warning`; `primary` may be near either, so sign and arrow carry the meaning. |
| Font swap | Lap times use tabular figures; a proportional user font falls back to fixed advance for the timing column. |
| Cockpit and hood | Hood keeps G-meter, gear, lap time and a delta bar; map dropped (brief). |
| Reduced flash | Bar breath 2 Hz; the delta swing keeps its scale move but no brightness pulse. |
| Colour-vision | Sign, arrow and the chip's position carry ahead or behind. |

## 7. Signature moments

- **Limiter.** (a) **Recommended:** only the 12-cell upshift bar strobes, the gear and G-meter stay still; the real menu's own upshift bar (1.11) is the strip. (b) The bar holds steady `shift` and the G-meter's outer ring warms in `shift` as a steady ramp; still only the bar strobes.
- **Gear change.** (a) **Recommended: the shift tick.** `HeroGear` 1.12 → 1.0, and the trail stores a tick at the shift point, so after a lap the map shows where the driver shifted, the way TRACK PACE analysis shows driving data on the circuit. (b) The G dot's longitudinal dip during the shift is held as a faint 300 ms trace, a real physical signature of the shift.
- **The delta swing** (brief, kept): at a sector or lap crossing the time freezes for 2 s (to confirm in F3), the delta enters with its sign and an arrow, and on a lap the finished lap becomes the reference line if it is faster.

## 8. What it needs from the 4K library

The pre-build made **no kit** for this dash: PREBUILD.md and `kits.rs` (PR #18 branch) list `AmgMbuxTrackPace` among the dashes that "wait for their frame grabs", and there is no `assets/clusters/amg_mbux_track_pace/prebuild/manifest.json`.

Reusable now: the shared MSDF fonts in `assets/fonts/` (Barlow Semi Condensed, Saira, Titillium Web); `antiglare_matte_grain` (computed for SimInputTrace, SimPitWall and MclarenSlimDisplay) if the board puts a screen face behind the telemetry part; Supersport's `horizon_haze` only if the board shares its tunnel, which the sibling rule argues against.

LIBRARY GAP: G-meter ring and lens, Ø300 at 4K, rest key and two G-swing keys (Blender sprite, the brief's Phase 6 asset); made after F1 and F4 fix its form.
LIBRARY GAP: dark panel material behind the telemetry part, 4096² seamless (ComfyUI per the brief, or a computed halftone if F1 shows the gen-2 dotted halftone the Supersport note found on lit surfaces); made after F1.
LIBRARY GAP: 128² blue-noise dither tile for the map line's soft edge and the wash (computed), unless the shared one already exists.

Kit item (shared code, not a library gap): a hook that lets a dash set the binnacle wash colour (for the delta lean), since `scrim::Wash` takes the palette's `primary` from the host.

## 9. Departures and open questions for the board

1. **Font:** Barlow Semi Condensed, not Saira and Titillium (the AMG family rule). Confirm against F1.
2. **Gear rank:** the brief makes the gear rank 1; the real centre content's example is the gear display (1.1), so this matches, but its size is unknown until F1.
3. **Rev band:** the brief adds a slim rev band; no source shows one on TRACK PACE. Keep only if F1 or F6 shows it; the upshift bar alone may be enough.
4. **Colourway:** does TRACK PACE follow the drive program's blue or red (F8)? If yes, the board picks blue to stay apart from Superscreen's red.
5. **Map orientation:** heading-up (brief) or north-up like a stored circuit map (F5).
6. **Telling it apart:** Supersport's G-force page also has a G target left and data right. TRACK PACE is told apart by the timing column and the map; the board should give the map real presence at rank 4.
7. **Sectors:** three equal thirds (recommended) or braking-zone splits.
8. **Free-roam laps:** a Horizon world rarely loops; consider a manual "mark start" in the OSD (a `config.extra` key) as the real system's "Start Record" (1.8).
9. **Coach bars:** keep the line and brake coach only if check C shows the two fields carry data with the assists off.

## 10. Sources

Mercedes material: [MY24 AMG C Sedan supplement](https://Mbusa.Com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20C%20Sedan%20Owners%20Manual.pdf); [MY24 AMG CLE Coupe supplement](https://mbusa.com/content/dam/mb-nafta/us/owners/manuals/2024/2024-owners-manuals/MY24_AMG%20CLE%20Coupe%20Owners%20Manual.pdf); [AMG GT Coupé press kit, Aug 2023](https://media.mercedes-benz.ch/download/9595ea6c-ddc6-465b-aaac-1d1fc4e7fc57/20230819-en-pi-mercedes-amg-gt-coupeacute.pdf); [mercedes-amg.com AMG TRACK PACE](https://www.mercedes-amg.com/en/amg-track-pace); [Mercedes online manual, AMG GT 4-Door](https://moba.i.mercedes-benz.com/baix/cars/257.3_audio_2020_b/en_GB/page/ID_87d5596c493762d8354ae365689e58e4-7f8789aee8f0bf57ac190d2b518cb50b-en-GB.html) (via the leads json).

Reviews: [PistonHeads, Ghost in the machine](https://www.pistonheads.com/news/blog/ghost-in-the-machine--ph-footnote/46434); [T3, Track Pace app](https://www.t3.com/features/i-tried-the-mercedes-amg-track-pace-app-and-it-made-me-faster-around-the-circuit); [mbpassion, on-demand](https://mbpassion.de/2021/04/amg-track-pace-nun-als-on-demand-ausstattung/); [CAR magazine](https://carmagazine.co.uk/car-news/tech/amg-track-pace-app) (paywalled; its "no purple" point rests on the earlier fetch summary, unconfirmed).

Game: [Forza Horizon 6 Data Out documentation](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation).

Repo and leads: `/mnt/project-files/dash-research/amg-trackpace-one-superscreen.json`, `amg-display-styles.json`; Superscreen and Supersport notes on their branches; PREBUILD.md and `kits.rs` on `claude/project-thread-ib67x9`; Supersport Drive `references/sources.md`.

Videos (for frames, not opened): listed in 2.1.
