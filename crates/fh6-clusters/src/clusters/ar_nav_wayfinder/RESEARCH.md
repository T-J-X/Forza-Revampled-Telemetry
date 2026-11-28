# ArNavWayfinder: research note

Build #36, catalog row 17, category Apple Vision, Target `concept`, family ADAS & HUD, views cockpit and hood (cockpit anchor (1920, 1835), envelope −720, −320, 1440 × 560; hood anchor (1920, 1760), envelope −880, −220, 1760 × 400; `docs/design/TOKENS.md` Layout). Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on the linked page on 2026-11-28 (the page is named; "via summary" means read through one WebFetch summary because the page is paywalled). **Computed** means worked out here with `python3 -I`, inputs shown. **Our reading** is a conclusion or proposal for this dash, open to the direction check. Anything not confirmed says "unconfirmed" and what step 1 must check.

Family rules carried over from the finished `TeslaAdas` note and from `WindshieldHud/RESEARCH.md`: a grey world with one saturated hue for what is in the path and red only for imminent action; the road ahead is predicted or remembered from the car's own motion and drawn as such, never as map or sensor data; no fake traffic; every readout live from frame 1; the driven car's logo or custom name; the swappable font.

## 1. The benchmark

**One benchmark: Mercedes-Benz MBUX Augmented Reality navigation** (first in the 2018 A-Class, W177), the brief's own reference. It is a screen, not a head-up display, which suits a binnacle dash: a first-person picture of the road ahead with guidance graphics placed on that road. What makes it recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | A video image of the surroundings from the front camera is enriched with navigation information: arrows or house numbers are "automatically" placed straight into the picture on the media display, to help find a house number or the right side street. | Cited: [mbpassion.de, 13 Feb 2018](https://mbpassion.de/2018/02/neue-details-zur-augmented-reality-funktion-der-a-klasse/) (German, Daimler press image) |
| B2 | The view appears "as you approach an intersection"; a directional arrow "with subtly shifting colors"; house numbers "projected" onto the properties you pass. | Cited: [The Drive, 2019 A-Class MBUX test](https://www.thedrive.com/tech/23816/daddy-mbux-testing-mercedes-benzs-new-infotainment-system-in-the-2019-a-class) |
| B3 | A live feed with "an arrow at the exact junction to turn at". | Cited: [Auto Express, MBUX explained](https://www.autoexpress.co.uk/mercedes/353561/mbux-mercedes-infotainment-technology-explained) |
| B4 | In use: a large blue chevron along the bottom edge moves forward toward the junction and hovers at the turn; road number or street name when known; the swooping arrow is "a little slow" at tight roundabouts and complex junctions; it works best at simple junctions. | Cited via summary: [CAR magazine, "Merc's augmented reality nav: does it work?"](https://carmagazine.co.uk/car-news/tech/mercedes-augmented-reality-navigation-how-does-it-work). This confirms the brief's "judged slow". |
| B5 | The S-Class AR-HUD takes the same idea onto the windscreen: it highlights "the correct turning point in a complex intersection", projects house numbers, and its projector is a mirror matrix of 1.3 million mirrors. | Cited: [Mercedes-Benz NZ magazine, 19 Oct 2020](https://www.mercedes-benz.co.nz/passengercars/brand/mercedes-me-magazine/innovation/articles/augmented-reality.html). The HUD belongs to `WindshieldHud`. |

**What the visual review measures against (Our reading):** a first-person road with a true vanishing point; guidance symbols that lie on the road in perspective, shrink with distance and bend with the road; one saturated guidance hue (Mercedes blue in the real thing, the palette's accent here); labels pinned to places in the world, not to the screen.

**The other real AR navigation, for the board:**
- **Volkswagen ID.3 / ID.4 AR-HUD:** a far window about 10 m out (diagonal about 1.8 m) holds turn arrows and assist content; a near band about 3 m out holds static speed and signs. Approaching a turn, "an advance notification on the road level, and then three arrows located at the junction. The closer the driver gets to the junction, the larger the arrows become. At the same time, their textures fade." The design rule is "Less is more"; displays are stabilised against the car's movement. Cited: [Volkswagen Newsroom, 17 Dec 2020](https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730).
- **Hyundai / Genesis:** the G80 concept at CES 2019 carried WayRay holographic AR navigation over the whole windscreen, with turns, highlighted objects and, as a stated possibility, "the perfect racing line during track days" (Cited: [Motor Authority via ABC 33/40, 9 Jan 2019](https://abc3340.com/news/auto-matters/hyundai-turns-the-windshield-into-an-augmented-reality-nav-system)). In production, the 2024 Genesis G80's 14.5-inch navigation "includes augmented reality" (Cited: [CarExpert spec sheet](https://www.carexpert.com.au/genesis/g80/2024/luxury/technology)); how it looks is unconfirmed.
- **Apple Maps AR walking guidance (iOS 15 on):** raise the iPhone, it scans the buildings around to fix "a highly accurate position", and step-by-step directions appear "in the context of the real world" (Cited: [Apple Newsroom, 27 Sep 2021](https://www.apple.com/newsroom/2021/09/apple-maps-introduces-new-ways-to-explore-major-cities-in-3d/); [MacRumors](https://www.macrumors.com/how-to/ios-ar-walking-directions-maps/)). It shows "big arrows and street names", and "the navigation arrows cast realistic shadows and glow as you approach them" (Cited: [iDownloadBlog, updated 23 Mar 2026](https://www.idownloadblog.com/2023/08/07/apple-maps-ar-walking-directions-tutorial/)). **The Apple lesson for this category (Our reading): localise first, then draw.** Apple refuses to draw until it knows where you are; this dash draws remembered road only once it has matched the car to its memory.
- **visionOS world anchoring:** an ARKit `WorldAnchor` is "a fixed location in a person's surroundings", and "ARKit persists world anchor UUIDs and transforms across multiple runs of your app"; each anchor reports `isTracked` (Cited: [Apple Developer, WorldAnchor](https://developer.apple.com/documentation/arkit/worldanchor)). This dash's gates are world anchors in the game's coordinates, with the same tracked or not-tracked honesty (section 5).
- **Tunnel and pathway in the sky (aviation):** an "out-the-window" perspective view with a tunnel the pilot flies through; pilots kept good accuracy with minimal training (Cited: [Stanford GPS Lab, after Barrows and Powell 1999](https://gps.stanford.edu/research/early-gpspnt-research/gps-tunnel-sky/)). NASA Langley's workshops (1994 to 1996) named display clutter and obscuring hazards as the main concerns (Cited: [NTRS 20030032531](https://ntrs.nasa.gov/citations/20030032531)). This is where the brake-gate arches come from.
- **Chevron alignment signs:** placed on the outside of a curve, "in line with and approximately at a right angle to approaching traffic", spaced "so that several delineators or chevrons are always visible" (Cited: [FHWA, 2009 MUTCD appendix](https://highways.dot.gov/safety/local-rural/roadway-departure-safety-manual-local-rural-road-owners/appendix-b-2009-mutcd); [TxDOT field book](https://www.txdot.gov/manuals/trf/sfb/markers_and_delineation/dilineators-i1003914/delineator_and_chevron_spacing-i1004004.html)).
- **The game's own guidance:** FH6's racing line is green where you can hold speed, yellow where a turn approaches and you should slow, red where you must brake; a "braking only" setting shows it only before turns (Cited, community guide: [Ludo, FH6 racing line](https://www.ludo.guide/guide/forza-horizon-6/racing-line); step 1 confirms in game). It matters because the packet carries two bytes about that line (section 5).

**Better target?** None found. MBUX AR navigation stays the benchmark; the Apple Maps behaviour (localise, then draw; arrows that glow as you near them) is the category's flavour.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | A-Class MBUX AR navigation: camera view, arrow, house numbers | unknown (page image) | [mbpassion.de](https://mbpassion.de/2018/02/neue-details-zur-augmented-reality-funktion-der-a-klasse/) |
| 2 | Mercedes AR nav in use: blue chevron hovering at the turn | unknown (paywalled) | [CAR magazine](https://carmagazine.co.uk/car-news/tech/mercedes-augmented-reality-navigation-how-does-it-work) |
| 3 | S-Class AR-HUD article images | unknown | [Mercedes-Benz NZ](https://www.mercedes-benz.co.nz/passengercars/brand/mercedes-me-magazine/innovation/articles/augmented-reality.html) |
| 4 | VW ID. AR-HUD lead photo and footage (three arrows at the junction) | unknown | [Volkswagen Newsroom](https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730) |
| 5 | Genesis G80 concept, holographic AR navigation | unknown | [ABC 33/40](https://abc3340.com/news/auto-matters/hyundai-turns-the-windshield-into-an-augmented-reality-nav-system) |
| 6 | Apple Maps AR walking guidance | unknown (gallery) | [Apple Newsroom](https://www.apple.com/newsroom/2021/09/apple-maps-introduces-new-ways-to-explore-major-cities-in-3d/) |
| 7 | Apple Maps AR arrows with shadows, street names | unknown | [iDownloadBlog](https://www.idownloadblog.com/2023/08/07/apple-maps-ar-walking-directions-tutorial/) |
| 8 | Synthetic-vision flight display, perspective path | https://upload.wikimedia.org/wikipedia/commons/b/b1/Synthetic_Vision.JPG | [Commons](https://commons.wikimedia.org/wiki/File:Synthetic_Vision.JPG) |
| 9 | Tunnel-in-the-sky prototype (paper figures) | unknown | [Stanford GPS Lab](https://gps.stanford.edu/research/early-gpspnt-research/gps-tunnel-sky/) |
| 10 | Pathway-in-the-sky workshop figures | unknown (PDF) | [NTRS](https://ntrs.nasa.gov/citations/20030032531) |
| 11 | Chevron signs on the outside of a real bend | https://upload.wikimedia.org/wikipedia/commons/8/88/A44_round_the_bend_-_geograph.org.uk_-_759047.jpg | [Commons](https://commons.wikimedia.org/wiki/File:A44_round_the_bend_-_geograph.org.uk_-_759047.jpg) |
| 12 | Chevron alignment sign artwork | https://upload.wikimedia.org/wikipedia/commons/4/44/AU_road_sign_D4-6-R.svg | [Commons](https://commons.wikimedia.org/wiki/File:AU_road_sign_D4-6-R.svg) |
| 13 | Curve marker sign artwork | https://upload.wikimedia.org/wikipedia/commons/7/7a/Singapore_road_sign_-_Warning_-_Curve_marker_-_Left.svg | [Commons](https://commons.wikimedia.org/wiki/File:Singapore_road_sign_-_Warning_-_Curve_marker_-_Left.svg) |
| 14 | MUTCD warning signs for a turn (page Figure 5) | unknown | [FHWA](https://highways.dot.gov/safety/local-rural/roadway-departure-safety-manual-local-rural-road-owners/appendix-b-2009-mutcd) |

Drive `references` folder (SOURCES.md lists every URL for the PC to fetch): https://drive.google.com/drive/folders/1Azk2Sdqnw5v4jZ5iPAUrvkY_6UX63sux. No image bytes were saved by this cloud run; the PC fetches the four direct URLs and step 1 grabs frames from the Mercedes and VW footage.

## 3. How the real thing behaves, with motion notes

- **Start-up.** MBUX shows the camera view only near a junction (B2, B4); no source describes a start-up animation (unconfirmed). Our reading: live from frame 1 (TOKENS), the rails and chevrons empty, the remembered road drawn only once localised; on the very first drive of a session only the κ arc shows.
- **Approach to a manoeuvre.** Mercedes: the chevron runs forward to the junction and hovers there (B4). VW: an advance notice on the road, then three arrows at the junction that grow as you near and fade their texture to keep the road clear. Apple: arrows glow as you approach. Our reading: a gate arch is a thin outline far off, gains a soft glow inside 100 m, and its inner area thins as it passes over the eye, so it never blocks the gear.
- **Idle and cruise.** VW stabilises its display against the car's motion (Cited). Our reading: on a charted straight only the ribbon dashes scroll at true ground speed; nothing else moves.
- **Latency.** The one documented complaint is that Mercedes' arrow is slow at complex junctions (B4). Ours is never delayed: gates are placed in world coordinates and projected each frame, with no animation between positions.
- **Pull to the limiter, limiter, gear change.** Real AR navigation has no shift light. House rule: only the chevron strip strobes (10 Hz, 50% duty; 2 Hz breath in gentle mode); the rails hold a steady full shift fill; the gear change follows TOKENS (old glyph 0.92 and fade in 50 ms, new from 1.12 with a 24 px slide on `fast`).
- **Braking.** The game's racing line turns red where you must brake (section 1). Our reading: a gate that the car is reaching too fast (section 4) takes `caution`, and `warning` only if the stop distance at 1 g already exceeds the distance left.
- **Drift and lateral G.** No AR navigation shows slip. Our reading: the ribbon stays on the road (it is the remembered line), while the whole road view yaws by β = atan2(v_x, v_z) about the vanishing point, so in a drift the road swings across the gear the way the real road swings across the windscreen; clamp ±25°.
- **Reverse and neutral.** Our reading: in reverse the memory stops writing, the gates hide and the rails stay; neutral changes nothing.
- **Shut-down.** Hide in 0 ms (AGENTS rule 9).

## 4. The maths and physics

No row in "Real geometry and maths" for this dash. It still models a camera, a memory and a gate rule; the tests can assert these (all Computed).

| Model | Formula | Values to assert |
|---|---|---|
| Camera | pinhole, f = (W ÷ 2) ÷ tan(FOV ÷ 2); a road point at depth z sits y = f h ÷ z below the vanishing point, lateral X at x = f X ÷ z | cockpit W 1440, FOV 40°: f = 1978.18 px; vertical FOV of the 560 px envelope 16.11° |
| Near road | bottom edge y +240 is 380 px under the VP (y −140) | z_near = 5.726 m at h 1.1 m; the rails' bottom corners (±720) are ±2.084 m, a 4.168 m road (about one lane plus margin) |
| Ribbon and arches | width f × 1.6 ÷ z; 1.2 m arch height f × 1.2 ÷ z | ribbon 316.5 px at 10 m, 63.3 at 50 m, 12.7 at 250 m; arch 237.4, 47.5, 9.5 px; the 250 m gate sits only 8.7 px under the VP |
| Rails as tach | parametrised by screen length L = √(720² + 380²) = 814.1 px each | if each rail stops where y = −40 (100 px under the VP, clear of the gear), the usable length is 599.9 px and spans road from 5.73 m to 21.76 m |
| Hood camera | W 1760, FOV 40° | f = 2417.78 px; with the VP at y −100 the near road is 9.50 m and ±3.46 m wide |
| Memory | 65,536 samples every 2 m | covers 131.1 km; 16 B per sample is 1.00 MiB, 20 B 1.25 MiB, 24 B 1.50 MiB |
| Localisation | nearest sample within ±64 samples (±128 m) of the last match, heading match cos Δψ ≥ cos 30° = 0.8660 | a crossing road at 90° never matches; a bridge over the road is separated by y |
| Gate warning | reach rule d_stop = v² ÷ (2a) | at 1 g: 30 m/s 45.9 m, 50 m/s 127.5 m, 70 m/s 249.8 m, so the brief's 250 m ghost lead is under one stop distance at 70 m/s |
| Ghost lead | 250 m fixed against 4 s × v | 20 m/s: 12.5 s or 80 m; 50 m/s: 5.0 s or 200 m; 70 m/s: 3.57 s or 280 m. Our reading: lead = max(150 m, 4 s × v) |
| Chevron spacing law | delineator spacing S = 3√(R − 50) in feet reproduces TxDOT table 8-3 (5730 ft: 226.1 against 225; 1433: 111.6 against 110; 573: 68.6 against 70; 101: 21.4 against 20) | metric: R 30 m gives S 6.36 m; 100 m, 15.25 m; 500 m, 36.47 m (for the optional world chevrons, section 5) |
| κ arc (uncharted) | κ = ω_y ÷ max(v, 3) (decoded) or a_lat ÷ v² (verified) | 0.5 g at 50 m/s: κ = 1.961e−3 /m, R 509.9 m |

**Catalog corrections (for the lead):**
1. The Risk line says "Lap number, lap times and distance travelled are not decoded yet". Since main 60c99c9 they are decoded (`lap_number`, `best_lap`, `current_lap`, `distance_traveled`, offsets confirmed at check C), as the `AmgMbuxTrackPace` note also reports. The line should read: use the game's lap fields when a race is on, else the memory's own timing.
2. The memory is "≈1 MB" only at 16 B a sample. The fields the brief lists plus the timestamp that a delta needs come to 20 B packed (x, z as f32; y as i16 decimetres; speed u16 at 0.01 m/s; time u32 ms; brake, throttle, gear, heading u8), 1.25 MiB, or 1.50 MiB with a 4-byte hash-chain link. Either quantise further or write ≈1.5 MB.
3. The ghost numeral in `inkGhost` (alpha 16) has a contrast of 1.10:1 on black, so it cannot carry a cue; `inkLabel` (120) gives 4.49:1 and `inkMuted` (158) 7.35:1 (WCAG, TOKENS ink #F4F7FB). Recommend `inkMuted` at 60% of the hero's cap.
4. If y −140 is the hero gear's centre, its cap 170 top is at y −225 and the chevron row's bottom at y −273: a 48 px gap, so a cap 90 ghost "just beyond the hero gear" cannot sit above it. Place the ghost beside the gear on the horizon, or shrink it.

## 5. Telemetry mapping

`Signals` carries `position` but not orientation, the driving line or the race fields; the dash reads those from `FrameInput.telemetry` (`FH6TelemetryData`).

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Chevron strip (Z1, ten 48 × 38 at y −292, 31.1 px gaps over 760 px) | `rpm`, `idle_rpm`, per-gear shift thresholds | verified | outside-in, five per side pointing inward; strobe 10 Hz at the limiter |
| Rails (Z3) | rpm fraction | verified | screen-length fill on the `ribbon` spring from the bottom corners to y −40 |
| Hero gear | `gear` | verified | R, N, 1 to 10, 11+ as the number |
| Speed | `speed`, unit | verified | 0.6 hysteresis; on the horizon at left, ≤ 420 px from the gear |
| Memory writes | `position` x, y, z | decoded, offset confirmed at check C | every 2 m of travel, only when not airborne, not rewinding, in a forward gear |
| Heading | `orientation` yaw | decoded, offset confirmed at check C (units and zero direction unconfirmed) | fallback: atan2(Δx, Δz) from the last two samples, exact enough at 2 m |
| Localisation and remembered road | memory, position, heading | derived | nearest sample in the spatial hash, ±64-sample window, heading ±30°; draw only while matched (the `isTracked` idea) |
| Road rise and fall | recorded y | derived | the ribbon climbs or dips by Δy ahead, so crests read before they arrive (Our reading) |
| Gates | recorded brake onset (brake crosses 0.2 after ≥ 1 s below 0.05) | verified input, derived place | dashed after one pass, solid when a second pass agrees within 4 m |
| Apex diamonds | recorded speed minimum between gate and throttle pick-up | derived | one per corner |
| Ribbon tint | recorded throttle and brake | verified | `accent` on throttle, `caution` on brake, `inkMuted` coasting |
| Target-gear ghost and ticks | recorded gear at the gate (same `car_ordinal` only) vs live gear | verified (gear), decoded (car ordinal) | one tick per downshift still needed |
| Gate distance | arc length along the memory, 2 m a sample | derived | metres, or feet with mph (open question) |
| Lap delta | `best_lap`, `current_lap`, `lap_number` in a race; else memory time at the matched sample against the best pass | decoded, check C; fallback exact from the timestamp | Saira Condensed tabular, `go` / `warning` with a sign |
| κ arc when uncharted | `yaw_rate` | decoded, check C | κ = ω ÷ max(v, 3); fallback a_lat ÷ v² |
| Horizon counter-tilt | `orientation` roll | decoded, check C | minus a 2 s baseline, ×2, clamp ±4°; fallback level |
| Drift swing | `velocity` x, z | verified | road view yaws by β about the VP, hidden below 5 m/s |
| Game line offset | `normalized_driving_line` (s8) | decoded; meaning unconfirmed | Our reading: the car's lateral place against the game's racing line. The near end of the ribbon slides by it; hidden while it reads 0 |
| Brake-now cue | `normalized_ai_brake_difference` (s8) | decoded; meaning and sign unconfirmed | Our reading: how much harder than the AI you should brake here; tints the next gate `caution` |
| Rail flash | `rumble_strip` per wheel | decoded, check C | left or right wheels on a kerb light that rail's outer 1 px, steady, 150 ms crossfade |
| Parallax and light | `g_lat`, `g_long` | verified | near symbols 4 px per g, gates by 1 ÷ z, horizon 0 |

**The driving line is one byte, not a path (Cited fact, Our reading of its use).** The official field list gives only `S8 NormalizedDrivingLine` and `S8 NormalizedAIBrakeDifference`, with no comment on their meaning ([Forza Motorsport Data Out documentation](https://support.forza.net/hc/en-us/articles/21742934024211-Forza-Motorsport-Data-Out-Documentation)); open-source parsers read them the same way and say nothing more. So the packet cannot draw the line ahead. The path ahead must be our own memory (charted road), our own prediction (the κ arc), or the honest fallback of nothing. Check C must log both bytes with the racing line set to full, braking only and off, in a race and in free roam.

**What the game cannot send** (each with replacements; the recommended one first).

1. **Route, turn arrows, the hovering chevron.** (a) **Recommended: the remembered bend.** On charted road the ribbon bends ahead along the memory for the next 250 m, and the next gate's arch stands where you braked last time: guidance from your own driving, never from a route. (b) The game's own colour logic: once check C confirms the AI brake difference, the next gate turns `caution` when the game says brake harder, the in-dash echo of the red racing line. (c) Uncharted: only the fading κ arc.
2. **House numbers and street names pinned to buildings (MBUX, Apple Maps).** (a) **Recommended: corner labels anchored in the world.** Each solid gate carries a small Titillium plate with the gear and entry speed of your best pass through it, pinned to the gate and shrinking with it, like Apple's street names on buildings. (b) Countdown boards in the world at 100 m and 50 m before each gate, 1 px outlines with the number, as circuit brake markers (the brief's reference; marker sizes unconfirmed).
3. **Distance to turn, ETA.** (a) **Recommended: next-gate distance and the lap delta** (brief). (b) Time to gate in seconds at the current speed, which does not change with units.
4. **Lane lines and lane departure.** (a) **Recommended: the rails are the road edges and the tach** (brief). (b) The rumble-strip flash on the rail of the side that is on a kerb.
5. **Speed-limit and advisory signs.** (a) **Recommended: the advisory plate.** Under each solid gate, the apex speed of your best pass in a small square plate, after the MUTCD advisory speed plaque that supplements a curve sign (Cited, FHWA page). (b) Nothing; the gear ghost already says how hard the corner is.
6. **Other traffic, pedestrians, hazards.** (a) **Recommended: none.** A first-person view with invented cars would read as sensor data. (b) The best pass as a pace light: a short bright band on the ribbon where you were on your best pass at the same elapsed time, running ahead or behind you in the world (`AmgMbuxTrackPace` has the same ghost as a map dot; this is its in-world form).
7. **Map, compass, points of interest.** (a) **Recommended: nothing.** VW's "Less is more". (b) Optional world chevrons on the outside of a remembered bend, spaced by S = 3√(R − 50) ft with R from the memory, so the bend reads like a real signed curve.
8. **The camera image itself.** (a) **Recommended: no picture of the road.** Lines on OLED black, per the family (no lit ground). (b) A ground plane at ≤ 6% luminance that exists only inside the rails, dithered with the blue-noise tile.

Rejected for sibling overlap: a raised windscreen position, crown staves and the stop bar (`WindshieldHud`); an ego car, a chase camera and FSD slowing chevrons in the path (`TeslaAdas`); a top-down driven-line map and a "BRAKE 80 m" text countdown (`AmgMbuxTrackPace`, which keeps braking points as map ticks); a pitch ladder and flight-path marker (`F35HelmetHmd`).

## 6. Edge cases

- **Max rpm 0 or tiny, idle ≥ max:** chevrons and rails empty, gear and speed drawn, memory still runs; guard every division.
- **Idle near max, 12,000+ rpm, 4,500 rpm diesels:** thresholds from the car's idle, shift and per-gear landing rpm.
- **EVs (single gear, odd rpm):** the ghost and ticks need gears, so with one forward gear the ghost shows the best-pass entry speed instead; the rails follow power ÷ peak power seen this session if rpm stays flat.
- **Reverse, neutral, gear above 10:** reverse stops writes and hides gates; 11+ as the number; the ghost never asks for a gear the car does not have (clamp to the highest gear seen).
- **mph and km/h, hp and kW:** speed follows the unit; distances in metres unless open question 5 says feet with mph.
- **Limiter bounce (about 50 rpm a frame):** 100 rpm hysteresis on the chevron thresholds; the strobe keys off a 150 ms limiter latch.
- **First drive, empty memory:** κ arc only, labelled `UNCHARTED` in Titillium cap 16; gates appear on the second pass of a road.
- **Fast travel, respawn, reset to road:** a position jump over 100 m in one frame drops the match, writes nothing for that step, and re-localises.
- **Rewind:** the timestamp keeps counting but the position jumps back along the recorded road; detect travel against the recorded heading or a backward jump and suppress writes until 1 s after it ends, so a rewound corner never writes a second gate.
- **Crossings, bridges, a road driven the other way:** the heading test rejects crossings and the opposite direction (stored as its own samples); y separates a bridge from the road under it.
- **Ring full:** the oldest samples are overwritten; each hash bucket stores the sample's write generation so a stale link is skipped; best-lap samples are pinned and never overwritten.
- **Car change mid-session:** gates and the road stay (they belong to the road); gear ghosts only for passes with the same `car_ordinal`.
- **Spikes, crashes, huge G:** parallax clamps at 2 g; a crash (speed drop over 15 m/s in 0.2 s) marks the pass as not a best.
- **Airborne:** v_y large and all suspension extended (decoded): no writes, the road holds, the horizon does not tilt.
- **Stale or paused stream:** the app hides the dash; on resume no sweep and the match restarts.
- **Palette extremes:** the ribbon, gates and rails keep a 1 px dark keyline so bright and dark primaries both read on black; red stays reserved for `warning`.
- **Font swap:** gear and speed take the user's display font; plates stay Titillium.
- **Cockpit and hood:** the hood is wider and shorter (1760 × 400): same camera law, VP near y −100, chevrons at the top edge (about y −206, as TeslaAdas's hood pods), labels shrink.
- **Reduced flash:** the strip strobe becomes the 2 Hz breath; the 1-frame merge flash is dropped.
- **Colour-only cues and colour vision:** a gate's state also shows as dashed or solid and as stroke weight (1, 2, 3 px); the delta has a sign; the ghost has its ticks.

## 7. Signature moments

**Limiter and shift.**
- (a) **Recommended: the brief's strip.** Chevrons fill outside-in toward the gear; at the shift point all ten point at it and the rails reach their stop at full shift colour, steady. On the limiter only the 38 px chevron row strobes 10 Hz.
- (b) **The tightening bend.** As the chevrons fill, their spacing closes from 31 px to about 16 px around the centre pair, the way chevron spacing shrinks as a curve tightens (TxDOT tables), so the strip reads as a bend closing in on the gear; the strobe rule is unchanged.

**Gear change.**
- (a) **Recommended: the target-gear ghost merge** (brief): each downshift steps the hero toward the ghost and removes one tick; when they match the two merge with the TOKENS rim flash (1 frame at 60%), and passing the gate lays its arch flat onto the ribbon while the next arch rises from the VP over 200 ms (`materialize`).
- (b) **The road lunges.** On an upshift the rails drop to the new gear's landing rpm while the whole road view dollies forward by the distance of 0.25 s of travel on the `fast` spring, so the change is felt as surge in the world, not only as a digit.
- (c) **The gate glows.** From Apple's arrows that glow as you approach: the next gate's glow steps up at 100 m, 50 m and at the gate, steady, never flashing.

## 8. What it needs from the 4K library

- **Its kit:** none. PREBUILD.md on the PR #18 branch lists ArNavWayfinder among the dashes that are "procedural only by their briefs"; `tools/asset-forge/src/recipes/ar_nav_wayfinder.rs` is registered and empty; no `assets/clusters/ar_nav_wayfinder/prebuild/manifest.json` exists.
- **Shared parts it reuses:** `assets/glass/blue_noise_128.png` (on main) to dither the gate glows and the optional ground; the repo fonts `saira-condensed`, `saira` and `titillium-web` (MSDF atlases built by the dash's own font step); the kit's feathered shapes and closed-form springs.
- **Not used:** the `tesla_adas` car sprites (no ego, no traffic), any glass (the category note says no glass panes).

No LIBRARY GAP: every part is a stroke, an SDF or a triangle strip drawn at run time.

Code and test needs for the handover (not library): a fixed-capacity road memory and spatial hash in `fh6-core` or the dash module (preallocated at start, no per-frame allocation; `fh6_core::trail` stores only x and z, 120 nodes, and returns a `Vec`); a recorded telemetry fixture with two passes over one road and one rewind, so the goldens can show charted gates (the review states have no history); a `dash_gates` probe for the race fields and the driving-line bytes.

## 9. Departures and open questions for the board

1. **Ghost colour and place** (section 4, corrections 3 and 4): `inkMuted` at about cap 100, beside the gear on the horizon, instead of `inkGhost` cap 90 above it.
2. **Ghost lead distance:** max(150 m, 4 s × v) instead of a fixed 250 m, so it is useful at 20 m/s and in time at 70 m/s.
3. **Drift swing of the whole road view by β:** keep it (it is the dash's slip cue) or leave the road fixed and slip only the ribbon?
4. **The driving-line bytes:** use them only after check C confirms meaning, sign and whether FH6 sends them with the assist off.
5. **Units of distance:** metres always, or feet with mph?
6. **Persist the memory across sessions** (the WorldAnchor idea): write it at session end with `fh6-core`'s atomic file, so roads driven yesterday are charted today. About 1.5 MB on disk; off by default?
7. **Hood layout:** the brief gives no hood geometry; recommended VP y −100, chevrons y −206, rails to the 1760 px corners.
8. **Gate thresholds:** brake onset 0.2 after 1 s below 0.05, and two passes within 4 m; check against real driving at step 1.
9. **Pace light (section 5, 6b):** optional, or left to `AmgMbuxTrackPace`?

## 10. Sources

Benchmark and real AR navigation
- mbpassion.de, A-Class MBUX AR details: https://mbpassion.de/2018/02/neue-details-zur-augmented-reality-funktion-der-a-klasse/
- The Drive, MBUX in the 2019 A-Class: https://www.thedrive.com/tech/23816/daddy-mbux-testing-mercedes-benzs-new-infotainment-system-in-the-2019-a-class
- Auto Express, MBUX explained: https://www.autoexpress.co.uk/mercedes/353561/mbux-mercedes-infotainment-technology-explained
- CAR magazine, Mercedes AR navigation (via summary): https://carmagazine.co.uk/car-news/tech/mercedes-augmented-reality-navigation-how-does-it-work
- Mercedes-Benz NZ magazine, augmented reality: https://www.mercedes-benz.co.nz/passengercars/brand/mercedes-me-magazine/innovation/articles/augmented-reality.html
- Volkswagen Newsroom, AR head-up display: https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730
- ABC 33/40 (Motor Authority), Genesis G80 holographic AR navigation: https://abc3340.com/news/auto-matters/hyundai-turns-the-windshield-into-an-augmented-reality-nav-system
- CarExpert, 2024 Genesis G80 technology: https://www.carexpert.com.au/genesis/g80/2024/luxury/technology

Apple
- Apple Newsroom, Apple Maps in iOS 15: https://www.apple.com/newsroom/2021/09/apple-maps-introduces-new-ways-to-explore-major-cities-in-3d/
- MacRumors, AR walking directions: https://www.macrumors.com/how-to/ios-ar-walking-directions-maps/
- iDownloadBlog, AR walking directions tutorial: https://www.idownloadblog.com/2023/08/07/apple-maps-ar-walking-directions-tutorial/
- Apple Developer, ARKit WorldAnchor: https://developer.apple.com/documentation/arkit/worldanchor

Aviation, roads, game
- Stanford GPS Lab, Tunnel in the Sky (Barrows and Powell): https://gps.stanford.edu/research/early-gpspnt-research/gps-tunnel-sky/
- NASA NTRS, pathway-in-the-sky workshops (Parrish, NASA/CP-2003-212164): https://ntrs.nasa.gov/citations/20030032531
- FHWA, 2009 MUTCD standards appendix: https://highways.dot.gov/safety/local-rural/roadway-departure-safety-manual-local-rural-road-owners/appendix-b-2009-mutcd
- TxDOT, delineator and chevron spacing: https://www.txdot.gov/manuals/trf/sfb/markers_and_delineation/dilineators-i1003914/delineator_and_chevron_spacing-i1004004.html
- Forza Motorsport Data Out documentation: https://support.forza.net/hc/en-us/articles/21742934024211-Forza-Motorsport-Data-Out-Documentation
- Forza Horizon 6 Data Out documentation: https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation
- Ludo, FH6 racing line (community guide): https://www.ludo.guide/guide/forza-horizon-6/racing-line

Images: the Wikimedia Commons pages in section 2.

Leads not opened: Mercedes owner's manual AR pages (moba.i.mercedes-benz.com returned 404); the retired Forza forums (closed July 2026), where the driving-line bytes may have been explained; Barrows and Powell's tunnel papers (PDF figures); circuit brake-marker board sizes.

Repo: `CATALOG.md` (brief, Apple Vision, Flash and telemetry, siblings), `CLUSTER_PROMPT.md`, `docs/design/TOKENS.md`, `docs/design/families/adas-hud.md`, `docs/TELEMETRY_USAGE_MATRIX.md`, `crates/fh6-core/src/telemetry.rs` and `trail.rs`, `crates/fh6-kit/src/cluster.rs`, `tesla_adas/RESEARCH.md`, `windshield_hud/RESEARCH.md`, `amg_mbux_track_pace/RESEARCH.md`, PR #18 `PREBUILD.md`.
