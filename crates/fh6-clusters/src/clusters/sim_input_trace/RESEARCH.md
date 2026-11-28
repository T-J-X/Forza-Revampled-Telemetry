# SimInputTrace: research note

Build #31, catalog row 59, Motorsport category, Target `concept`, Race Dash family, views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read from the linked source on 2026-11-28 (for open-source tools, from their published code). **Computed** means worked out here from cited or repo values, with the inputs and numbers shown (`python3 -I`). **Our reading** is a conclusion drawn for this dash, open to the direction check. "Unconfirmed" marks what step 1 must still check.

## 1. The benchmark

**Benchmark: the input-trace overlay that sim racers run over iRacing, measured on the open-source irdashies "Input" widget** (MIT licence, React and Canvas, iRacing plus Le Mans Ultimate beta). The closed tools (RaceLab "Input Telemetry", the overlay most seen on streams, [simracingcockpit.gg](https://simracingcockpit.gg/iracing-overlay-racelabapps/)) show the same object but publish no geometry; irdashies publishes its drawing code, so the visual review can measure against exact defaults. Two more open tools cross-check it: SimTrace (Rust, egui; ACC, AMS2, iRacing) and Marble Trace (Rust and Tauri; iRacing). MoTeC i2 is the professional reading of the same traces. **Our reading:** no single overlay owns the look; the genre's shared grammar below is what makes it recognisable, and irdashies is the measurable instance of it.

What makes the object recognisable (all **Cited** unless marked):

| # | Feature | Source |
|---|---|---|
| B1 | **One horizontal strip:** trace panel, then live pedal bars, then a large gear with speed under it, side by side. In the default layout the trace has flex weight 4 and every other element weight 1, so with bars and gear the trace takes 4/6 of the width. | irdashies `Input/layout.ts` (`weightOf`), `InputContainer.tsx`; [README "Input trace"](https://raw.githubusercontent.com/tariknz/irdashies/main/README.md) and its screenshot (ref 1) |
| B2 | **Newest sample at the right edge, history scrolls left.** SimTrace places each point by its timestamp ("so rendering stays correct" whatever the frame rate) over a window of 10 s by default. irdashies keeps a ring of samples instead (default 400, range 50 to 1000), one per telemetry update. | SimTrace `renderer/trace_graph.rs`, `config/settings.rs`; irdashies `InputTrace/inputTraceDrawing.ts` |
| B3 | **Colour code: throttle green, brake red, clutch blue, steering light grey.** irdashies uses its theme's green, red, blue and slate 300; VirtualCoach uses the same three pedal colours; SimTrace defaults throttle `#00FF00`, brake `#FF0000`, clutch `#AA44FF`; Le Mans Ultimate's own HUD puts "red and green bars" for brake and throttle beside the speedo. | irdashies `inputTraceDrawing.ts`; [VirtualCoach Inputs](https://vc-docs.pages.dev/overlays/inputs/); SimTrace settings; [LMU HUD guide](https://guide.lemansultimate.com/hc/en-gb/articles/13201287548687-Understanding-the-HUD-Heads-Up-Display) |
| B4 | **Throttle and brake share one 0 to 100% axis, overlaid.** irdashies maps every pedal to the same y scale (domain −0.05 to 1.05, so 0 and 100% sit just inside the frame). A data engineer prefers the same: overlaid "both [are] plotted at a larger scale", and "only one of them is going to be active at a time". | irdashies; [HP Academy, Braking technique](https://www.hpacademy.com/courses/professional-motorsport-data-analysis/braking-performance-braking-technique) 00:29 to 00:52 |
| B5 | **Steering is a line about the centre**, not from the floor: irdashies draws it at mid-height, full lock at the edges. | irdashies `steeringLine` |
| B6 | **A light grid:** dashed horizontals at 0, 25, 50, 75 and 100% (1 px, `#666`, dash 2/2) in irdashies; vertical lines every 2 s for a window over 5 s, every 1 s otherwise, in SimTrace. | irdashies `drawGrid`; SimTrace `draw_grid` |
| B7 | **Smooth, thick strokes:** 3 px default on a 400 × 100 logical canvas (range 1 to 10), drawn through a B-spline (`curveBasis`), so 1/255 steps never show as stairs. | irdashies |
| B8 | **Live bars beside the trace** in the order clutch, brake, throttle, each 1:4, its value in percent above it (grey at 0, white otherwise), filling from the bottom with a 50 ms ease-out. Marble Trace lights a bar's end stops "once a pedal is touched and when it is fully pressed". | irdashies `InputBar.tsx`; [Marble Trace README](https://raw.githubusercontent.com/mvoof/Marble-Trace/main/README.md) |
| B9 | **Assist activity painted onto the trace, not lamps:** irdashies draws ABS as a yellow stroke 1.67 times thicker over the brake trace, or a 30% yellow fill under it; SimTrace recolours brake segments `#00BBFF` while trail braking (steering above 5°) and `#FF44AA` for ABS while cornering, and flags TC in `#FFCC00`. | irdashies; SimTrace settings and `draw_brake_trace` |
| B10 | **Gear numbers on the time axis.** irdashies' Lap Trace prints the gear (5, 4, 3, 4) along the bottom at each change under a vertical cursor (ref 4). | irdashies README image `laptrace.png` |
| B11 | **The professional tool stacks the same channels.** MoTeC i2's Compare worksheet shows "speed, rpm, gear, brake, and throttle information in a number of line graphs", and the Driver worksheet adds "steer angle and longitudinal/lateral g forces". | [Coach Dave Academy, MoTeC in ACC](https://coachdaveacademy.com/tutorials/how-to-use-motec-data-in-assetto-corsa-competizione/) |

**Our reading, the recognisable core:** a dark strip in which a green line and a red area alternate along one baseline, a pale steering line wanders about the middle, the shapes slide left at constant speed, and the live pedal bars and a big gear sit at the right edge where the lines are born. That is the silhouette the review compares at thumbnail size.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | irdashies Input widget: trace with dashed grid, red brake, green throttle, grey steer; bars 100/26/49; gear 3 over 158 km/h; wheel | https://raw.githubusercontent.com/tariknz/irdashies/main/docs/assets/input.png | [irdashies README](https://raw.githubusercontent.com/tariknz/irdashies/main/README.md) |
| 2 | Marble Trace Input Trace: green and red arcs, white steer line, blue clutch on the floor, three bars with end stops, wheel with speed and gear | https://raw.githubusercontent.com/mvoof/Marble-Trace/main/site/assets/widgets/input-trace.png | [Marble Trace README](https://raw.githubusercontent.com/mvoof/Marble-Trace/main/README.md) |
| 3 | Marble Trace RPM Lights: a long row of square LEDs, green then amber, unlit grey | https://raw.githubusercontent.com/mvoof/Marble-Trace/main/site/assets/widgets/rpm-lights.png | same |
| 4 | irdashies Lap Trace: gear numbers on the distance axis, a vertical cursor, throttle and brake steps | https://raw.githubusercontent.com/tariknz/irdashies/main/docs/assets/laptrace.png | irdashies README |
| 5 | irdashies Shift Light widget | https://raw.githubusercontent.com/tariknz/irdashies/main/docs/assets/shiftlight.png | irdashies README |
| 6 | SimTrace demo (frame 150 of 300): brake turning cyan while trail braking, throttle blips during downshifts, brake by steer phase plot, bars and gear ring | https://raw.githubusercontent.com/rgracey/SimTrace/main/docs/media/SimTrace.gif | [SimTrace README](https://raw.githubusercontent.com/rgracey/SimTrace/main/README.md) |
| 7 | MoTeC i2 Compare and Driver worksheets in ACC (page figures) | page only | [Coach Dave Academy](https://coachdaveacademy.com/tutorials/how-to-use-motec-data-in-assetto-corsa-competizione/) |
| 8 | Le Mans Ultimate speedo with brake and throttle bars and three shift LEDs (page figure) | page only | [LMU HUD guide](https://guide.lemansultimate.com/hc/en-gb/articles/13201287548687-Understanding-the-HUD-Heads-Up-Display) |
| 9 | VirtualCoach input overlay: bright driver bars beside dark reference bars, steering ring | page only | [VirtualCoach Inputs](https://vc-docs.pages.dev/overlays/inputs/) |
| 10 | Brake trace phases on a real logger (video figure) | page only | [HP Academy, Braking technique](https://www.hpacademy.com/courses/professional-motorsport-data-analysis/braking-performance-braking-technique) |

Drive `references` folder: https://drive.google.com/drive/folders/1nN_hXAx0nuLtMF6kjJrl-rXeVRQM6wNA (refs 1 to 6 saved as image files, ref 6 as the extracted frame; refs 7 to 10 listed in `SOURCES.md` for the PC to capture).

## 3. How the real thing behaves, with motion notes

The benchmark is a logger, so its "states" are the shapes the driver writes into it. Everything below that is not cited is **Our reading** for this dash.

| State | What the overlays do (cited) | What SimInputTrace does |
|---|---|---|
| Start-up | irdashies fills its ring with zeros (steer at centre), so it opens on flat lines; it can hide until the car is on track. | The window opens empty: baseline, grid and labels only. Lines are born at the right edge as packets arrive; no invented history. LEDs dark. |
| Idle | Flat lines at 0, steer centred. | Same. No `COAST` at standstill (needs speed above about 5 m/s, a proposed threshold). The only motion is the scroll itself, which at idle carries nothing but grid lines: freeze the grid's scroll while speed is 0 and every input is 0, so an idle dash is still. |
| Cruise | Partial throttle is a wavy green line. | Same, at rank-4 luminance. |
| Pull to the limiter | Throttle pinned at the top; shift lights fill. | Throttle flat at 100%; the 16 LEDs fill left to right; the rev bar follows rpm; time at the limiter is hatched on the rev row (brief). |
| Limiter | Overlays may flash a background (irdashies' optional shift flash). | Only the LED row strobes, 10 Hz, 50% duty, release at most 50 ms (TOKENS `motion.limiterStrobe`); nothing in the trace window flashes. |
| Gear change | Lap Trace prints the gear at the change (B10). | A shift mark drops into the trace in the same frame (section 7). |
| Braking | Brake rises fast to a peak at "the beginning of the stop", bleeds off, then a second bleed "in proportion to the steering input" (trail braking); the gap after the throttle lift is coasting to minimise. SimTrace shows throttle blips on downshifts as narrow green spikes inside the brake area (ref 6). | Brake as a filled area so the build, peak and bleed read as a shape; the throttle-off to brake-on gap measured (section 7). |
| Drift and lateral G | Counter-steer shows as "steering input reverses direction" spikes. | The steer line crosses the centre; handbrake ticks on the baseline (after check C); optional longitudinal-G trace. |
| Reverse and neutral | Not described. | Gear `R` or `N`; traces continue; a shift mark is labelled `R` or `N`. |
| Shut-down, pause, rewind | FH6 sends no data "during menus, pauses, replays, rewinds, or after finishing a race" ([FH6 Data Out documentation](https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation)), and the host hides the dash on a stale stream (AGENTS.md directive 9). | The time base stops with the stream. When packets return, a thin break mark is placed where the stream stopped and the lines are not joined across it (a rewind starts a new timeline). |

Motion rules (**Our reading**, from the brief and TOKENS): the scroll is the only continuous motion and runs at exactly 125 px/s, sub-pixel smooth at the render rate (section 4); LEDs use the shift-light token attack 0 ms and release at most 50 ms; the gear uses the token gear-change motion and is the only large moving element; shift marks appear in one frame with no animation of their own, since they already scroll.

## 4. The maths and the time base

The catalog has no maths row for this dash. Its model is the time base of the trace, and its unit tests can assert the numbers below without a GPU.

**FH6 sends a packet "at a rate equal to the game's frame rate"** (**Cited**, FH6 Data Out documentation). So neither the render frame nor the packet is a fixed clock, and the brief's "sampled once a frame into fixed ring buffers of 512 values" does not hold:

| Case | Samples in 8 s | Fits 512? |
|---|---|---|
| Game at 30 / 60 fps (packets) | 240 / 480 | yes |
| Game at 120 / 144 fps (packets) | 960 / 1152 | no |
| Render at 120 / 144 Hz (one sample a frame) | 960 / 1152 | no |

(**Computed**, samples = rate x 8.) A per-frame sampler also stretches or squeezes the time axis when the frame rate changes, which breaks "a fixed 125 px a second".

**Recommended time base (Computed):** resample onto a fixed grid of **64 Hz bins** (15.625 ms each). 512 bins hold exactly 8 s, and one bin is 1000 / 512 = 1.953125 px at 125 px/s. Each packet writes its bin (latest value wins; a bin with no packet repeats the previous one, which happens for 4 of every 64 bins at 60 fps). Bin index comes from the host's monotonic time `FrameInput.t`, not from `TimestampMS`, which "can overflow to 0" (**Cited**). The window is drawn at x = right edge − (t_now − t_bin) x 125, so it slides by fractions of a pixel every frame instead of stepping 1.95 px at 64 Hz.

**Catalog correction (brief and spec.md):** "sampled once a frame" should read "resampled by time into 64 Hz bins". For the hood, the brief's 1500 px window at the same 125 px/s spans 12 s = 768 bins, more than 512 (**Computed**): either the hood scrolls faster (1500 / 8 = 187.5 px/s, 2.93 px a bin) or the ring grows to 1024 bins (6 channels x 1024 x 4 bytes = 24,576 bytes, still fixed and allocation-free). Recommended: one 1024-bin ring and the same 125 px/s in both views, so the motion is identical (open question 2).

Other values for tests (**Computed**):
- Pedal bytes are 0 to 255 and steer −127 to 127 (**Cited**, FH6 doc). One pedal step is 170 / 255 = 0.667 px in the 170 px window (0.43 px in the 110 px hood window); one steer step 85 / 127 = 0.669 px. The bytes never stair-step visibly, so no spline is needed for smoothness; a straight segment per bin is honest.
- LED row: 16 caps of 32 px on 780 px gives a pitch of (780 − 32) / 15 = 49.867 px and gaps of 17.867 px. The row meets `shiftRowSpanMinPx` 720 and the strip stays under `strobeStripMaxPx` 42.
- LED thresholds, if spread evenly from the stage-1 share (0.75) to the shift point (`shift_pct` 0.97 x max rpm): step = 0.25 x 0.97 x max / 15. Max 7,000: first LED 5,092, shift 6,790, step 113.2 rpm. Max 12,000: 8,730 to 11,640, step 194.0. Diesel 4,500: 3,274 to 4,365, step 72.8, which is smaller than the 120 rpm off-hysteresis (ratio 1.65), so a falling pull holds an LED longer than a rising one: correct, not a bug (the AmgOneF1 note found the same).
- Kit cost: one 128-point polyline cost 0.36 ms (CLUSTER_PROMPT, "Long paths"). Three naive 512-point polylines would cost about 3 x 512 / 128 x 0.36 = 4.3 ms, fourteen times the brief's 0.3 ms estimate. Each trace is single-valued in x, so it is a true height field: brake as one thin column per bin from the baseline, throttle and steer as one short segment per bin pair (511 segments, each with a tiny bounding box) or chunks of 8 (64 chunks a trace). Measure both before anything else is built (brief).
- Coast gap: time from the last bin with throttle above 2% to the first with brake above 2% (2% is 5/255, proposed), in whole bins of 15.625 ms; a test feeds a 12-bin gap and expects 187.5 ms, shown rounded to 10 ms as `190 ms`.

## 5. Telemetry mapping

"Verified" = long-verified against the golden packets. "Decoded, check C" = decoded from the 88 fields, offset confirmed at check C (the real-game check at the PC).

| Visual part | Input (packet field) | Status | Transform |
|---|---|---|---|
| Throttle trace (solid 3 px line) | `accel` | verified | /255, binned at 64 Hz |
| Brake trace (filled area) | `brake` | verified | /255 |
| Steering trace (1.5 px line about the centre) | `steer` | verified | /127, clamp ±1; it is the input, not a wheel angle in degrees |
| Live pedal bars and steering tick at the right edge | `accel`, `brake`, `steer` | verified | latest packet, no smoothing beyond one packet (Gt7HoodPro uses a 30 ms tau for its steering dot) |
| Longitudinal-G fourth trace (optional) | `acceleration_z` | verified | /9.81, ±1.5 g on the half-height |
| 16 LEDs, rev bar, its numerals | `current_engine_rpm`, `engine_max_rpm`, `engine_idle_rpm` | verified | `Signals::shift_rpm`, per-gear landing rpm learned live (`[f32; 10]`) as the brief's per-gear thresholds |
| Limiter hatching on the rev row | rpm, max rpm | verified | on at rpm ≥ shift rpm, off 120 rpm under it (the shared tach rule) |
| Gear numeral and shift marks | `gear` | verified | `Gear` enum: 0 = R, 1 to 10, else N |
| Tabular speed | `speed_ms` | verified | display unit, 0.6 hysteresis |
| `COAST` on the baseline | `accel`, `brake`, `speed_ms` | verified | both pedals under 2% and speed above 5 m/s |
| Coast-gap figure | `accel`, `brake` | verified | section 4 |
| Trail-brake styling of the brake area | `brake`, `steer` | verified | brake above 2% while abs(steer) above 0.08 (about 10/127, proposed); SimTrace's 5° rule has no degrees here |
| Clutch trace | `clutch` | decoded, check C | /255; joins after check C (brief); unknown whether FH6's automatic clutch moves it |
| Handbrake ticks | `hand_brake` | decoded, check C | a tick where it exceeds 50% |
| `LOCK` overlay on the brake trace | `tire_slip_ratio_fl/fr` | decoded, check C | abs(ratio) > 1 (FH6 doc: "> 1.0 means loss of grip") while brake above 2% |
| `SPIN` overlay on the throttle trace | `tire_slip_ratio_rl/rr` (driven axle by `drivetrain_type`) | decoded, check C | abs(ratio) > 1 while throttle above 20% |
| Kerb ticks under the baseline | `wheel_on_rumble_strip_*` | decoded, check C | a short tick per bin with any wheel on a kerb |
| Torque, if a power row is ever added | `torque` | decoded, check C | fallback power x 60 / (2π rpm) |

### What the game cannot send
Each part of the benchmark that FH6 cannot drive, its **bold replacement** (recommended one marked R), never faked; anything modelled carries `MODEL`.

1. **The sims' ABS-active flag** (irdashies' yellow overlay, SimTrace's orange). FH6 has no ABS or TC flag. **R: `LOCK` painted on the brake area where a front tyre's slip ratio passes 1** (decoded, check C), drawn as a dotted edge rather than a colour so it reads without colour vision. Second: a `SPIN` dotted edge on the throttle line from the driven axle's slip ratio.
2. **Raw pedal input versus the sim's processed input** (irdashies option). FH6 sends one value per pedal. **R: drop it; the trace shows what FH6 sends.** Second: none needed.
3. **The coach or reference lap** (VirtualCoach's dark reference bars, MoTeC lap overlays). There is no reference lap in the packet. **R: in races only, a ghost of your previous lap**: the last lap's throttle and brake stored as bytes by distance into the lap (`distance_traveled`, `lap_number`, decoded, check C), a 2-minute lap at 64 Hz is 7,680 bins x 2 bytes, about 15 KB fixed; drawn dim and dashed behind the live lines, hidden in free roam. Second: **`normalized_ai_brake_difference`** as a thin tick row under the brake trace, showing early or late against the game's braking line (decoded; its meaning must be learned at check C before it is drawn, MODEL until then).
4. **Track position, corner names and distance axis** (MoTeC distance mode, irdashies corner feedback, SimTrace track strip). FH6 has no track ordinal. **R: kerb ticks from the rumble-strip flags** under the baseline, so corners announce themselves where the car touched them. Second: an **apex speed chip**, the minimum speed between a brake application and the next full throttle (Marble Trace shows entry, apex and exit speed chips), derived exactly from `speed_ms`.
5. **Steering angle in degrees and a drawn steering wheel** (irdashies wheel styles, Marble Trace silhouettes). `steer` is a normalised input with no lock angle. **R: a steering tick on a ±100% rail at the right edge, no degrees.** Second: if a wheel is wanted, it turns by a fixed 450° per full input, labelled MODEL.
6. **Brake pressure in bar** (MoTeC). Only pedal percent exists. **R: the brake axis is labelled percent.** No second needed.

**User-changeable parts** (the owner's wish for customisation, applied here; **Our reading**): trace colours through palette roles (throttle `go`, brake `warning`, steering `secondary`, clutch `primary` as proposed defaults); LED stage colours through `go`, `warning` and `shift`; trace weights fixed (they carry meaning); window length 4 / 8 / 12 s and the fourth trace on or off as `config.extra` settings; the numeral font through the swappable `display` font; the anodised edge finish fixed by the asset rule.

## 6. Edge cases

| Case | What the dash does |
|---|---|
| Max rpm 0 or tiny | LEDs and rev bar dark, numerals hidden; traces, gear and speed still run (they do not need rpm). |
| Idle near max | Stage thresholds clamp at or above idle + 1 step; if shift rpm ≤ idle, LEDs stay dark. |
| 12,000+ rpm and diesels at 4,500 | Scale from the car's max rpm; numerals every 1,000 (or 2,000 above 12,000); diesel step under the hysteresis is accepted (section 4). |
| EVs | One gear, so no shift marks: the trace window is unaffected. The rev bar shows motor rpm from the packet; no idle shoulder. |
| Reverse, neutral, gear above 10 | `R` and `N` glyphs; a byte above 10 decodes to N (the `Gear` rule); marks labelled the same. |
| mph and km/h, hp and kW | Speed in `Signals::unit`; no power figure on this dash. |
| Limiter bounce (about 50 rpm a frame) | LED hysteresis 120 rpm; limiter hatch keeps its state through the bounce; rev-bar digits do not exist, so no digit flicker. |
| Spikes and crashes (huge G) | The optional G trace clamps at ±1.5 g with a clipped cap; inputs cannot spike beyond 0 to 100%. |
| Airborne | Nothing special: inputs still trace; LOCK and SPIN can fire falsely in the air, so require `speed_ms` > 5 and any suspension compression above 0.05 (decoded, check C) before drawing them. |
| Stale, paused or rewound stream | Host hides the dash; the time base stops; on resume a break mark, no joined line (section 3). |
| Palette extremes, random palettes | Trace roles must stay apart: if throttle and brake roles fall within 60° of hue, fall back to style alone (solid line versus filled area), which already separates them. |
| Font swap | Gear and speed use the `display` font with a tabular-figures check; labels `THR`, `BRK`, `STR` stay in `text`. |
| Cockpit and hood | Cockpit per the brief; hood keeps the same scroll speed (section 4) and needs its own LED row (open question 3). |
| Reduced flash | The LED strobe becomes the 2 Hz breath; nothing else changes because nothing else flashes. |
| Colour-only cues and colour vision | Throttle solid 3 px line, brake filled area, steering 1.5 px line about the centre, clutch dashed (proposed), each labelled at the right edge (brief); LOCK and SPIN are dotted edges; trail braking is a hatch on the brake area, not SimTrace's recolour. |

## 7. Signature moments

**Limiter / shift strobe** (only the shift strip strobes: 10 Hz at 50% duty, 2 Hz breath in gentle mode):
- **L-A (recommended): sim-wheel fill, then the strobe, with the hatch as the steady cue.** The 16 LEDs fill left to right in three bands, `go`, then `warning`, then `shift` (the brief's green, red, blue; F1 rows run "green → red → blue/purple", **Cited** via `docs/design/RESEARCH.md` from evo India; the Race Dash `shift` default is violet `#B27CFF`). At shift rpm all 16 take `shift`; at the limiter the row strobes. The family's one steady limiter cue is the hatching that grows on the rev row in real time while the limiter holds, so the driver later reads how long they sat on it.
- L-B: outside-in fill onto the centre pair (the Race Dash family signature). It keeps the family line but loses the sim-wheel read that is this dash's point.

**Gear change:**
- **G-A (recommended): the shift mark** (brief). In the frame the gear changes, a 1 px vertical line drops through the trace window with the new gear's number above it (precedent: irdashies' Lap Trace, B10), then scrolls with the data. Upshift and downshift marks differ by a small notch at the top (up or down), not by colour.
- G-B: a **shift-quality notch**: the mark's foot carries a tick when the upshift came after the limiter (late) or before the first `shift` LED (early), derived from rpm at the change. It turns the window into the lap coach the brief describes, still steady and colourless.
- G-C: the gear numeral rolls (old out up, new in from below) on the token gear motion. Allowed for the hero gear only; the trace never animates.

**Coast gap** (the brief's `COAST`, sharpened by HP Academy's first braking phase): where the throttle leaves and the brake arrives, a thin bracket spans the gap on the baseline with its length, for example `190 ms`, then scrolls away. With overlap (left-foot braking) the bracket inverts and reads `OVERLAP`.

## 8. What it needs from the 4K library

The pre-build kit for this dash (PR #18 branch `claude/project-thread-ib67x9`, `tools/asset-forge/src/prebuild/kits.rs` "sim_input_trace", `assets/clusters/sim_input_trace/prebuild/manifest.json`):
- **Swatch `dark_anodised_carrier`** (Blender, 4096² seamless, with normal and rough/metal/height): the LED carrier and the trace window's frame edge.
- **Sprites `shift_led_32_off` (three keys: rest −135°, swing −160° and −110°) and `shift_led_32_on` (rest key only)**, 40 px at 1:1 with collar and cap: the 16 LEDs, tinted `go` / `warning` / `shift` by the kit. SimPitWall uses the same design.
- **Computed `antiglare_matte_grain`** (4096² master, Worley 22 px cells, luminance ±4%): labelled micro-grain over the screen face.

Shared parts to reuse: `assets/glass/blue_noise_128.png` (dither for the brake area's soft fill and the hatch); fonts already in `assets/fonts` (`saira-condensed`, `chakra-petch`, `titillium-web`).

Still needed:
- LIBRARY GAP: anodised frame-edge profile strip for the trace window, about 4096 × 64 seamless along its length, at the rest key and two light azimuths (Blender), so the window edge is rendered hardware rather than a vector line; the brief names it and the kit has only the flat swatch.
- LIBRARY GAP: one bespoke ComfyUI texture: a 4096² seamless macro of etched anti-glare cover glass (an organic sparkle pattern, SDXL plus refiner), because CLUSTER_PROMPT requires a visible bespoke ComfyUI asset and every kit item here is Blender or computed.
- LIBRARY GAP (only if `On` sprites need the swing keys like `Off`): `shift_led_32_on` at the two G-swing keys (Blender), since the manifest has the rest key only.

Layout-dependent (not gaps): the LED carrier bar and the trace window's housing, sized by the board.

## 9. Departures and open questions for the board

1. **Time base.** Resample by time into 64 Hz bins, not once a frame (section 4). Catalog and spec.md wording to fix.
2. **Hood window length.** Same 125 px/s with a 1024-bin ring (12 s shown), or 8 s at 187.5 px/s. Recommended the first.
3. **Hood shift strip.** The brief's hood view is "the trace window alone ... with gear and speed at its right end", which has no shift strip, yet the limiter rule needs one. Proposed: the 16-LED row above the 110 px window (the 400 px hood envelope fits it). Also check that gear (cap 150 minimum) and speed fit the 260 px left beside a 1500 px window in a 1760 px envelope.
4. **Trace colours.** The genre's green / red / blue / grey maps to `go` / `warning` / `primary` / `secondary`. `warning` is the tokens' red stage (`#FF2D3A`), so the brake area is the largest red surface on the dash: it must sit at rank-4 luminance (a translucent fill under a brighter edge) and never bloom (brief).
5. **No smoothing spline.** irdashies uses a B-spline; at 0.67 px per byte step a straight segment per bin is already smooth and shows the true data (Our reading).
6. **LED colour order.** Green, red, blue is the brief's and the F1 benchmark's; sim tools vary (Marble Trace defaults green then amber; Fanatec's app sets "the color of each LED"). Kept, with the palette roles. Count and banding (for example 6 / 6 / 4) to set on the board.
7. **Idle stillness.** Freeze the grid scroll at a standstill with no input, so an idle dash does not move.
8. **Assist overlays.** `LOCK` and `SPIN` come from slip ratio, not ABS or TC flags; they are labelled, dotted and wait for check C.
9. **Previous-lap ghost and AI-brake ticks** wait for check C and race mode; both are optional and dim.
10. **Telling it apart.** Against `SimPitWall` (g-g diagram, learned power curve, channel table: what the car does) this dash has no diagram, no table and no curve; its shift marks live in the trace, never on the rev bar, where SimPitWall puts its upshift marker. Against `RackFocus` (focal planes and defocus) nothing here blurs. Against `BoschDdu` (DDU housing, round gauge, beam) there is no dial. The thumbnail read is a wide horizontal strip of scrolling lines.
11. **Benchmark choice.** If step 1 can read RaceLab's input overlay from owner captures, compare it with irdashies and say whether the review should measure the more widely seen tool instead.

## 10. Sources

Benchmark tools (code and docs):
- irdashies README: https://raw.githubusercontent.com/tariknz/irdashies/main/README.md
- irdashies `inputTraceDrawing.ts`: https://raw.githubusercontent.com/tariknz/irdashies/main/src/frontend/components/Input/InputTrace/inputTraceDrawing.ts
- irdashies `InputBar.tsx`, `InputContainer.tsx`, `layout.ts`: https://raw.githubusercontent.com/tariknz/irdashies/main/src/frontend/components/Input/ (same folder)
- SimTrace README, `config/settings.rs`, `renderer/trace_graph.rs`: https://raw.githubusercontent.com/rgracey/SimTrace/main/README.md
- Marble Trace README: https://raw.githubusercontent.com/mvoof/Marble-Trace/main/README.md
- VirtualCoach Inputs: https://vc-docs.pages.dev/overlays/inputs/
- RaceLab overview: https://simracingcockpit.gg/iracing-overlay-racelabapps/

Sims and loggers:
- Le Mans Ultimate HUD guide: https://guide.lemansultimate.com/hc/en-gb/articles/13201287548687-Understanding-the-HUD-Heads-Up-Display
- Coach Dave Academy, MoTeC in ACC: https://coachdaveacademy.com/tutorials/how-to-use-motec-data-in-assetto-corsa-competizione/
- MoTeC i2 V1.1.4 Feature Guide (auto-centre time/distance graphs): https://www.motec.com.au/hessian/uploads/i2_V1_1_4_Feature_Guide_0d03b00fa8.pdf
- HP Academy, Braking technique: https://www.hpacademy.com/courses/professional-motorsport-data-analysis/braking-performance-braking-technique
- Fanatec, Steering input telemetry: https://www.fanatec.com/us/en/explorer/products/steering-wheel/steering-input-telemetry/
- Fanatec, Customizing RevLEDs: https://www.fanatec.com/uk/en/explorer/products/steering-wheel/customizing-revleds-for-fanatec-steering-wheels/

Telemetry:
- Forza Horizon 6 Data Out documentation: https://support.forza.net/hc/en-us/articles/51744149102611-Forza-Horizon-6-Data-Out-Documentation

Repo: `program/phases/07-cluster-factory/CATALOG.md` (brief, Motorsport row, Flash and telemetry), `CLUSTER_PROMPT.md` ("The bar", "Long paths"), `docs/design/families/race-dash.md`, `data/design_tokens.json` (`revStages`, `limiterStrobe`, `layout.zones`), `crates/fh6-kit/src/cluster.rs` (`Signals`, `Gear`, `shift_pct` 0.97), `crates/fh6-clusters/src/clusters/bosch_ddu/RESEARCH.md`, `docs/design/RESEARCH.md` (F1 LED order, evo India), PR #18 `PREBUILD.md` and `kits.rs`.
