# F35HelmetHmd: research note

Build #28, catalog row 18, category Sci-Fi, Target `recreation`, family ADAS & HUD (pattern-setter `TeslaAdas`), views cockpit and hood. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels used throughout. **Cited** means read on 2026-11-28 from the page linked. **Computed** means worked out here with `python3 -I` from cited or brief values; the inputs and outputs are given so step 1 can re-run them. **Our reading** is a conclusion drawn for this dash, open to the direction check. Only public, unclassified material was used: maker pages, the program office page, press, public-domain US Navy training manuals and NASA/USAF research reports. Nothing here describes weapons or sensor modes beyond what those pages print, and the dash draws only generic HUD conventions (the brief's own rule).

## 1. The real display

**What this dash recreates: the "virtual HUD" page of the F-35 Gen III Helmet Mounted Display System (Collins Aerospace and Elbit Systems), as the pilot sees it when looking forward.** The F-35 has no physical HUD; the helmet draws HUD symbology on the visor, and when the pilot looks forward a fixed "virtual HUD" snaps into place ahead of the nose.

| # | Feature | Label and source |
|---|---|---|
| R1 | **No HUD glass at all.** "The F-35 is the first modern fighter to use an HMD to the exclusion of a HUD." The HMD projects "two identical images onto the visor, one for each eye, focused at infinity", carrying "HUD vector symbology as well as sensor video". | Cited, [F-35 JPO, HMDS](https://www.jsf.mil/hmds) |
| R2 | **Field of view 30 by 40 degrees, binocular, 100 percent overlap**; the HUD, the HMD and visor-projected night vision are "fully integrated". The engineering helmet of 2007 had 30 by 50 degrees before the continuous visor. | Cited, [Collins Aerospace product page](https://www.collinsaerospace.com/what-we-do/Military-And-Defense/Displays-And-Controls/Airborne/Helmet-Mounted-Displays/F-35-Gen-Iii-Helmet-Mounted-Display-System); [FlightGlobal, 17 Apr 2007](https://www.flightglobal.com/fixed-wing/2007/04/f-35-takes-off-with-helmet-display/) |
| R3 | **The virtual HUD is world-fixed ahead of the nose.** It "snaps into place digitally when the pilot looks forward" and "snaps over to the right as the pilot turns their head". | Cited, [TWZ, SRVL video, 2019](https://www.twz.com/29569/check-out-this-f-35-helmet-display-video-of-a-rolling-landing-on-the-hms-queen-elizabeth); [TWZ, Navy test videos, 2020](https://www.twz.com/12297/navy-presents-new-f-35-helmet-display-videos-and-flight-test-dangers) |
| R4 | **Layout of the primary flight page.** On Lockheed's published simulated view: airspeed (625 kt) "displayed in a rectangle on the left", groundspeed "GS 786" just below it, an attitude indicator (pitch and roll against the horizon) in the centre, a heading scale at top centre (279 to 290), altitude "14,000" in a rectangle on the right. "The F-35 helmet-display layout and symbols are similar to those of F-22 and F-16 head-up displays." | Cited, [Air & Space, "Super Helmet", Sept 2017, gallery image 1 of 4](https://www.smithsonianmag.com/air-space-magazine/super-helmet-180964342/) |
| R5 | **Boxes, not tapes.** R4 describes boxed digital airspeed and altitude, as in the F/A-18 and T-45 HUDs ("KIAS and barometric altitude digital data boxes"); the F-16 HUD has moving tapes. No public source read shows a moving tape on the F-35 virtual HUD. | Cited for the T-45, [NAVAIR P-1259 p. 9-4](https://navyflightmanuals.tpub.com/P-1259/Velocity-Vector-Relationship-To-Aircraft-Waterline-141.htm); Our reading for the F-35 (unconfirmed: step 1 checks frame grabs of the TWZ and Air & Space images) |
| R6 | **Flight path marker with an on-speed bracket and an acceleration caret.** "Like a classic HUD, the HMD shows the pilot a flight path marker (or velocity vector), with a bracket to indicate if the aircraft is 'on speed'... a caret moves up or down in reference to the flight path marker to give an acceleration-deceleration cue." | Cited, F. Colucci, "Tailored to Trap", Avionics, 1 Dec 2012, quoted on [F-16.net forum](https://www.f-16.net/forum/viewtopic.php?p=241959) (original page gone; step 1 should find an archive copy) |
| R7 | **Monochrome green.** The visor shows two "green spots" (the exit pupils); the LCD display shows a "green glow" in low light because its backlight lights the whole frame; OLED displays "avoid this effect by only illuminating the active pixels". The glow was too bright "even when set on the lowest level". OLED helmets went to the Navy and Marine Corps from 2020. | Cited, [Code One, "Lightning Sight", 2009](https://www.codeonemagazine.com/u2_article.html?item_id=12); [Penn State EOC, OLED HMD (quotes GAO-18-321)](https://eoc.psu.edu/project/oled-helmet-mounted-display-improves-performance-and-cost-for-f-35/); [Air Force Times, 8 Dec 2021](https://www.militarytimes.com/news/your-military/2021/12/08/f-35-helmets-that-fix-green-glow-are-on-their-way-but-not-to-the-air-force/) |
| R8 | **Stability matters.** Program issues in 2012 included "jitter" (lack of stability of the displayed symbology), latency and alignment, since mitigated. | Cited, Senate testimony of 24 Apr 2013, quoted on [F-16.net forum](https://www.f-16.net/forum/viewtopic.php?p=241959) |
| R9 | **Why green.** HUDs "typically use green light for their display symbology because the human eye is most sensitive to these wavelengths"; symbology is collimated and conformal (1:1 with the world). | Cited, [Radiant Vision Systems, "Quality Considerations for Aviation HUDs", 2018](https://www.radiantvisionsystems.com/blog/quality-considerations-aviation-head-displays-huds) |
| R10 | **Contents named by the maker side:** airspeed, altitude, rate of climb and weapon aiming, plus alerts. The helmet also overlays DAS infrared video and night-vision camera video under the symbology. | Cited, Code One (R7); JPO (R1) |

**Generic HUD conventions the F-35 page shares (public standards and manuals):**

| # | Convention | Label and source |
|---|---|---|
| C1 | **Pitch ladder:** positive pitch solid, negative pitch dashed; rungs "become more acute as pitch angle increases, and they always point at the horizon"; the horizon is "a larger, unlabeled bar". | Cited, [NAVAIR P-1259 p. 4-4 and 4-5](https://navyflightmanuals.tpub.com/P-1259/Hud-Navigation-Symbology-85.htm) |
| C2 | Ladder variants tested by USAF/NASA: tapered, straight or bent ("V", each rung rotated by half its angle); labels "slightly above and inboard from the end"; minus signs below the horizon; 5 degree spacing (10 degrees beyond ±30 in compressed ladders). The climb-dive marker is "a winged and tailed circle". | Cited, [NASA TM-4665, TRISTAR I, Newman et al., 1995](https://ntrs.nasa.gov/api/citations/19950017610/downloads/19950017610.pdf) |
| C3 | **Velocity vector limits:** T-45 limits it to ±7 degrees laterally and +3/−11 degrees vertically; beyond the limit it is caged and a "ghost velocity vector" shows the true path; it "will flash when display limits are exceeded". It is deflected "for drift and/or side-slip". | Cited, NAVAIR P-1259 p. 4-4 and 9-4 |
| C4 | **Heading scale:** a moving 30 degree wide scale with digits every 10 degrees; turning right moves it right to left. Bank scale marks 0, 5, 15, 30, 45 degrees. | Cited, [NAVAIR P-1259 p. 4-5](https://navyflightmanuals.tpub.com/P-1259/Bank-Scale-And-Pointer-86.htm) |
| C5 | **Digit steps:** airspeed in 1 kt steps; barometric altitude in 20 ft steps; vertical speed in 10 ft/min steps. | Cited, [NAVAIR P-1259 p. 4-5 and 4-6](https://navyflightmanuals.tpub.com/P-1259/Vertical-Speed-87.htm) |
| C6 | **F-16 practice (simulator documentation, a lead only):** current G at upper left, maximum G latched on the left and reset by the pilot, scales switchable off leaving digital readouts, heading scale moves below the speed and altitude scales once the gear is up. | Our reading of the open-source FlightGear F-16 manual, [NikolaiVChr/f16 wiki, HUD](https://github.com/NikolaiVChr/f16/wiki/HUD); unconfirmed against a real -1 manual |
| C7 | MIL-STD-1787 sets "symbol geometry, font, recommended dimensions, and mechanizations" for USAF displays, and leaves out colour, brightness and the dynamic behaviour. Its dimensions were not readable for free (shop pages only, DTIC was under maintenance). | Cited, [FAS summary of MIL-STD-1787A](https://man.fas.org/dod-101/sys/ac/equip/mil-std-1787.htm); line widths and character heights in mrad are **unconfirmed** |
| C8 | **Typeface:** B612 was designed 2010 to 2012 for Airbus cockpit screens (Intactile DESIGN with ENAC and Université de Toulouse III) and opened in 2017. It is a civil cockpit face, not the F-35's stroke font, whose face is not public. | Cited, [Google Fonts, B612 Mono](https://fonts.google.com/specimen/B612+Mono/about) |

**Our reading:** the recognisable F-35 page is sparse: two boxed numbers at the sides, a heading scale on top, a winged flight path marker over a thin ladder in the middle, all in one green on nothing. That matches the brief, with one correction: the brief's left speed tape and right rpm tape are F-16 conventions, and the F-35 page shows boxes (R4, R5). The better real target is unchanged; the recreation should lead with the boxes (section 9, D1).

## 2. Reference photos

All Wikimedia files are US federal government works (public domain). Press images are listed by page for the PC to fetch and are used as reference only, never shipped.

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | The F-35 HMDS helmet, visor and optics (USMC) | https://upload.wikimedia.org/wikipedia/commons/9/9b/F-35_Helmet_Mounted_Display_System.jpg | [Commons](https://commons.wikimedia.org/wiki/File:F-35_Helmet_Mounted_Display_System.jpg) |
| 2 | Small crop of the same helmet | https://upload.wikimedia.org/wikipedia/commons/0/0c/F-35_HMDS.jpg | Commons, same category |
| 3 | Lockheed simulated pilot's view: airspeed box left, GS below, attitude centre, heading top, altitude box right (R4) | not known; gallery image 1 of 4 | [Air & Space, "Super Helmet"](https://www.smithsonianmag.com/air-space-magazine/super-helmet-180964342/) |
| 4 | Real HMD video, F-35B rolling landing, virtual HUD and STOVL symbology | embedded video | [TWZ 29569](https://www.twz.com/29569/check-out-this-f-35-helmet-display-video-of-a-rolling-landing-on-the-hms-queen-elizabeth) |
| 5 | Real HMD night video and GIFs, virtual HUD snapping with head turns, monochrome night picture | embedded GIFs and the Flight Test Safety Committee video | [TWZ 12297](https://www.twz.com/12297/navy-presents-new-f-35-helmet-display-videos-and-flight-test-dangers) |
| 6 | A-7E HUD symbology drawing, labelled (US Navy, 1975): the classic ladder, FPM and readouts | https://upload.wikimedia.org/wikipedia/commons/b/ba/Heads_up_display_symbology.png | [Commons](https://commons.wikimedia.org/wiki/File:Heads_up_display_symbology.png) |
| 7 | F/A-18 HUD gun symbology photo: boxed airspeed and altitude, stroke weight | https://upload.wikimedia.org/wikipedia/commons/a/ae/F-18_HUD_gun_symbology.jpeg | Commons, Category Head-up displays in aircraft |
| 8 | F-15E LANTIRN: green stroke symbology over infrared video (the nearest public analogue to HMD over DAS) | https://upload.wikimedia.org/wikipedia/commons/3/3a/F-15E_LANTIRN_IR_HUD_image.jpg | Commons, same category |
| 9 | F-14A in an F/A-18 HUD, 1988: green glow on a real combiner, line widths against the sky | https://upload.wikimedia.org/wikipedia/commons/1/17/F-14A_Tomcat_in_head-up_display_c1988.jpg | Commons, same category |
| 10 | Eurofighter HUD during evasive manoeuvres: rolled ladder | https://upload.wikimedia.org/wikipedia/commons/e/eb/Eurofighter_HUD_Evasive_maneuvers.jpg | Commons, same category |
| 11 | F/A-18C pilot behind the HUD combiner, carrier deck | https://upload.wikimedia.org/wikipedia/commons/7/7a/US_Navy_030528-N-0295M-007_A_pilot_looks_through_his_Heads-Up_Display_%28HUD%29_while_sitting_in_his_F-A-18C_Hornet_aboard_the_aircraft_carrier_USS_Constellation_%28CV_64%29.jpg | Commons, same category |
| 12 | Declassified F-16 HUD video of an Auto-GCAS recovery: how the ladder, FPM and readouts move in a steep dive and roll-out (motion reference) | https://upload.wikimedia.org/wikipedia/commons/5/52/Auto-GCAS_Saves_Unconscious_F-16_Pilot%E2%80%94Declassified_USAF_Footage.webm | Commons, same category |
| 13 | NASA TM-4665 figures 7 to 9: climb-dive marker and the tapered, straight and bent ladder variants | PDF figures 7, 8, 9 | [NTRS 19950017610](https://ntrs.nasa.gov/api/citations/19950017610/downloads/19950017610.pdf) |
| 14 | B612 Mono specimen | specimen page | [Google Fonts](https://fonts.google.com/specimen/B612+Mono/about) |

Drive `references` folder: [f35_helmet_hmd/references](https://drive.google.com/drive/folders/1dm7hCqjKcMTlh98BuBOBtl-_U7sQOW93), with `SOURCES.md` listing every item above. No image bytes were saved by the cloud research; the PC fetches the URLs.

## 3. How the real thing behaves, with motion notes

| State | Real behaviour | Dash motion (Our reading) |
|---|---|---|
| Start-up | Not documented publicly. The symbology appears once the helmet is aligned; the magnetic tracker needs no pre-flight alignment (Code One). | No show: the tokens want every readout live from frame 1 and the dash rebuilds on view changes. At most a 120 ms fade-in of the stroke layer. |
| Idle, parked | Readouts static, FPM hidden or at boresight at zero speed (an aircraft at rest has no velocity vector). | FPM hidden below 5 m/s (brief), boresight and ladder drawn, speed box `0`. Nothing moves. |
| Cruise | Ladder and horizon conformal and stable (R8: jitter was a defect); heading scale slides right to left in a right turn (C4). | Springs on roll and pitch, critically damped; heading tape at 20 px per degree (Computed: a 30 degree window across 600 px). |
| Pull to the limiter | Aircraft equivalent: the acceleration caret climbs above the FPM wing (R6). | Lock line fills outside-in; an acceleration caret beside the FPM wing moves with longitudinal G. |
| Limiter / shift | No aircraft equivalent; a HUD flashes a symbol only at a limit (C3, the bank pointer flashes past 47.5 degrees). | Only the lock line and the box's 4 px top bar strobe 10 Hz, 50% duty (house rule); everything else steady. |
| Gear change | None. | Brackets open and the gear digit cuts (section 7). |
| Braking | Caret drops below the wing (R6). | Caret below the wing, length by deceleration; no colour change (one hue). |
| Drift, lateral G | FPM deflected by drift and sideslip, limited and caged at the field edge with a ghost (C3). | FPM slides by β at 14 px per degree with a 1 px leash; at the window edge it cages and a dashed ghost marks the true β (section 9, D3). |
| Reverse, neutral | None. | `R` or `N` in the gear brackets; in reverse the FPM is hidden (velocity points backwards, the marker would leave the window). |
| Airborne | Ladder pitches with the jet. | Ladder and FPM follow the measured pitch and v_y; G box shows near 0 G. |
| Shut-down | Display off with the helmet. | Host hides the dash on a stale stream (AGENTS directive 9). |

No public source gives symbol animation timings; every timing above is ours and the step-1 thread can measure frame steps in reference 4, 5 and 12.

## 4. The maths

The dash has no row in "Real geometry and maths". The mappings are kinematics only, and the values the unit tests can assert are in section 5 (Computed).

One check of the brief against itself (**Computed**): the brief clamps the FPM at ±35 degrees of β at 14 px per degree, which is ±490 px, but its attitude window is 560 px wide (±280 px). The marker would leave the window at β = 20 degrees. Either the clamp becomes ±20 degrees, or the marker cages at the window edge (the real HUD's own answer, C3). The family spec carries the same ±35 degree figure. This is not a maths-row error; it goes to the board as D3 and the lead may want the brief and family spec to say which.

## 5. Telemetry mapping

Raw fields come from `FrameInput.telemetry` (`FH6TelemetryData`); smoothed ones from `Signals`.

| Visual part | Input | State | Transform |
|---|---|---|---|
| Lock line, 12 dashes | `rpm`, `max_rpm`, shift rpm | verified | per-car, per-gear thresholds; outside-in fill |
| Gear in brackets | `gear` | verified | `Gear::from_raw`; R, N, 1 to 10 |
| Speed box (left, rank 2) | `speed` | verified | display unit from `AppConfig`; integer changes only when the difference is 0.6 or more (TOKENS) |
| Line under the speed box (the "GS" slot) | `g_lat`, `g_long` | verified | `G 1.2` = √(g_lat² + g_long²) to one decimal; session max-G latch beside it |
| rpm box (right, the altitude slot) | `rpm` | verified | digits in 50 rpm steps with 100 rpm hysteresis (Our reading, after C5's quantised altitude) |
| Line under the rpm box | `position` y | decoded, offset confirmed at check C | `ALT 312` in m or ft; MODEL-free, it is the game's world height. Fallback: hide |
| Pitch ladder offset | `orientation` pitch minus a 2 s baseline | decoded, offset confirmed at check C | ×4, 56 px per real degree (Computed: 14 × 4); fallback: ladder fixed at the horizon |
| Ladder rotation | `orientation` roll | decoded, offset confirmed at check C | ×2, sprung; fallback: level |
| FPM lateral | local `velocity` x, z | verified | β = atan2(v_x, v_z); 14 px per degree. Computed: (5, 20) m/s gives 14.04 degrees, 196.5 px; (−3, 25) gives −6.84 degrees, −95.8 px; (10, 10) gives 45 degrees, caged |
| FPM vertical | local `velocity` y, z | verified | atan2(v_y, v_z) ×4 at 14 px per degree; Computed: (0.5, 30) gives 0.955 degrees, 53.5 px |
| Acceleration caret | `g_long` | verified | caret offset from the FPM wing, 40 px per G (Our reading), clamped ±1.5 G |
| On-speed bracket, as on-shift bracket | `rpm`, shift rpm | verified | the F-35 bracket for "on speed" (R6) becomes a bracket on the rpm box that closes at the shift rpm |
| Heading tape (shy) | `orientation` yaw | decoded, offset confirmed at check C | heading = yaw in degrees mod 360 (Computed: π/2 gives 90.0; sign to confirm at check C); fallback: integrate `yaw_rate` from the timestamp, relative heading only |
| Lap clock, bottom right | `race_progress` current_lap | decoded, offset confirmed at check C | m:ss.t; fallback: a session timer from the timestamp |
| Brake caret weight | `brake` | verified | caret thickens 2 to 3 px with brake |

**What the game cannot send** (only what is truly absent from the 88 fields), each with replacement ideas; the recommended one is marked.

| Real part | Replacement idea |
|---|---|
| Targeting: diamonds for possible targets, circles for designated ones (R4), weapon aiming (R10) | **A. Recommended: a "steering diamond" on the racing line**, from `driving_line` (normalised driving line, decoded, offset confirmed at check C): a small diamond above the horizon that slides left or right by the car's offset from the game's line and closes into a circle when the car sits on it. It is a real reading, drawn in target grammar. B. A brake cue chevron from `normalized_ai_brake_difference` (same check). |
| DAS infrared and night-vision video under the symbology (R1, R10) | **A. Recommended: a conformal horizon line across the whole width**, beyond the attitude window, at alpha 40, carrying ground-stabilised ticks that stream toward the boresight with speed (speed and yaw rate only): the "see-through" world as strokes, still at rest. B. The shared `horizon_haze` band (section 8) below the horizon at a few percent luminance, the TJX cloud-and-horizon idea, only in the hood view. |
| Mach and angle of attack | **A. Recommended: a "car AoA" readout,** α = pitch-free angle between body and velocity, atan2(v_y, v_z) in tenths of a degree, as a small `α 0.4` under the FPM; verified fields. B. Drop. |
| Vertical speed, barometric and radar altitude | **A. Recommended:** world height from `position` y (above) and its rate as a climb caret on the rpm box (decoded). B. Drop the radar altitude; the game has no terrain height. |
| Groundspeed, wind | The speed box already shows ground speed; the "GS" line holds G (above). |
| Waypoint steering, time-to-go, datalink tracks, bullseye | **A. Recommended: lap data in the same grammar:** `LAP 3  P2` (lap number and race position, decoded) where the steerpoint text sits; hidden outside races. B. Distance travelled in km or mi. |
| Fuel and bingo | `fuel` is decoded (check C): **A. Recommended:** a shy `FUEL` cue at 10% and below, steady, never blinking (the real one blinks; house rule). B. Drop. |
| Warnings (FLY UP, LIMIT, master caution) | **A. Recommended:** a steady boxed `LIMIT` cue when the rpm sits on the limiter, in the shift colour, beside the steady `SHIFT` cue. B. `SPLASH` for 600 ms on a smashable hit (`smashable`, decoded), event only. |

## 6. Edge cases

- **max_rpm 0 or tiny:** `Signals` clamps it to 1; the lock line and rpm box hide below 1,000 rpm of range, the gear and speed stay.
- **Idle near max:** thresholds start from idle; if max minus idle is under 1,000 rpm the lock line fills on rpm fraction only.
- **12,000 rpm and above:** rpm box needs five digits in a fixed cell; 50 rpm steps still read. **Diesel 4,500:** fine.
- **EV (single gear, odd rpm):** gear `1` steady; lock line still follows rpm; if `cylinders` is 0 the shift cue still works from rpm.
- **Reverse:** FPM hidden, β not shown; gear `R`. **Neutral:** `N`, FPM still live (coasting).
- **Gear above 10:** two digits fit the brackets at cap 170 (Saira Condensed is narrow).
- **mph and km/h, hp and kW:** speed unit from `AppConfig`; altitude follows the speed unit (m or ft); power is not shown.
- **Limiter bounce of about 50 rpm a frame:** rpm box hysteresis of 100 rpm; lock-line dash edges use 2% hysteresis.
- **Spikes and crashes (huge G):** G readout clamps at 9.9 with a 150 ms median; the max-G latch ignores samples that jump more than 3 G in one frame (Our reading).
- **Airborne:** v_y large, FPM vertical cages at the window edge; ladder follows pitch; nothing flashes.
- **Stale or paused stream:** the host hides the dash.
- **Palette extremes and random palettes:** the stroke hue is the text-safe primary lifted to OKLCH L ≥ 0.80. Computed: `#3CFF7A` has OKLCH L 0.879, C 0.232, h 148.8, and 15.8:1 on black (9.1:1 at alpha 140); a darker green such as `#20C040` (L 0.706) fails the brief's L floor and must be lifted.
- **Font swap:** the user font replaces `display` (gear); the `aviation` role (B612 Mono) carries the boxes and stays a fixed-cell mono so digits never shift.
- **Cockpit and hood:** hood moves the boxes to ±560 and shrinks the attitude window to 760 × 150 (brief); at 150 px tall only ±1 rung fits (Computed: 56 px per degree).
- **Reduced-flash mode:** lock line breathes at 2 Hz; the `LIMIT` cue stays steady.
- **Colour-only cues and colour vision:** the display is one hue by design, so every state already has a shape: brackets closed or open, caret above or below, dashes filled or empty, a boxed word.

## 7. Signature moments

**Shift and limiter.**
- **A. Recommended (the brief's, kept):** the lock line fills outside-in; at the shift rpm the four corner brackets snap into a closed box round the gear (80 ms, ease-out) and a boxed `SHIFT` lights steady above it; at the limiter only the lock line and the box's 4 px top bar strobe 10 Hz. Grounded in R6: the F-35's own on-speed bracket is a bracket that closes on a target value.
- **B.** The on-shift bracket on the rpm box closes round the digits instead of the gear brackets, so the gear stays still; less visible at a glance.

**Gear change.**
- **A. Recommended:** brackets spring open 12 px outward and back (120 ms), the digit cuts with no roll (HUD digits step, C5), the lock line empties from the centre outward on the rpm drop.
- **B.** A one-frame "cage" of the FPM to the boresight on the shift, recalling the caged velocity vector (C3); rejected as default because it hides the real β at the moment of a shift.

## 8. What it needs from the 4K library

The pre-build (PR #18, `PREBUILD.md` and `kits.rs` on `origin/claude/project-thread-ib67x9`) lists `F35HelmetHmd` as **procedural only, no kit**; there is no `assets/clusters/f35_helmet_hmd/prebuild/manifest.json`. The B612 Mono and Saira Condensed TTFs are in `assets/fonts/`, and `fh6-kit` generates MSDF glyphs at load (`text.rs`), so the brief's atlases are not a gap.

Shared parts it can reuse: the computed `horizon_haze` band (seamless 4096 × 1024 luminance, seed 110003) from the `AmgMbuxSupersport` kit, for replacement B of the DAS video.

The asset rule (one bespoke ComfyUI and one bespoke Blender asset per dash, CLUSTER_PROMPT "Asset rules") meets a display that is strokes on black. Layout-independent parts that would serve it:

LIBRARY GAP: HMD stroke glow profile, a 256 × 1 16-bit linear look-up table of the halo falloff round a 2 px stroke (6 px reach, the brief's ranks 1 to 3 halo), computed on the CPU, so every stroke and glyph glows the same and no flat rectangle poses as glow; asset-forge `prebuild` computed map.
LIBRARY GAP: visor inner-surface reflection plate, 3840 × 1080 at 1:1, a very faint curved polycarbonate highlight with no symbology, alpha ≤ 12, rendered in Blender (glossy dielectric, studio rig) as the dash's bespoke Blender asset; used only in the hood view behind the strokes, never as a painted rim.
LIBRARY GAP: night cloud-deck band, 4096 × 1024 seamless, very low luminance, made in ComfyUI (SDXL base and refiner), as the dash's bespoke ComfyUI asset under the conformal horizon if replacement B is chosen; the `horizon_haze` band is the fallback.

## 9. Departures and open questions for the board

1. **D1. Boxes before tapes.** The F-35 page shows boxed airspeed and altitude (R4, R5). Recommend a boxed speed (left) and a boxed rpm (right, the altitude slot), and make the brief's two tapes optional shy layers; this also answers the brief's density risk. If the board keeps an rpm tape (rank 3 needs an analog rpm read), it should be a short tape beside the box, and the lock line carries the analog read either way.
2. **D2. Heading below, not on top.** Top centre holds the gear (rank 1), where the real page has its heading scale. Move the heading tape under the attitude window, as the F-16 does once the gear is up (C6).
3. **D3. FPM limit.** The ±35 degree clamp exceeds the window (section 4). Recommend caging at the window edge (±20 degrees) with a dashed ghost marking the true β, never a flash (C3 flashes; house rule).
4. **D4. One hue, no glow wash.** The real LCD helmet glows green over the whole frame (R7). The dash recreates the OLED helmet: only lit strokes, true black elsewhere, which is also what the OLED panel and the scrim want.
5. **D5. Numerals honest under exaggeration.** The ladder is ×4, so rung labels give real car degrees (1, 2) at 56 px spacing, not 5 and 10.
6. **D6. Font.** B612 Mono is a civil Airbus face; the real stroke font is not public. Keep B612 Mono in the `aviation` role and say so in the handover.
7. **D7. No maker or program marks.** No Lockheed, Collins or Elbit name or logo; the logo zone shows the driven car's logo, small, under the rpm box.
8. **D8. Customisable without losing the look** (TJS's "change needle colour" idea, carried over): stroke colour (palette primary; default the 'phosphor' preset `#3CFF7A`), stroke weight 1.5 to 3 px (`config.extra`, default 2 px), halo on or off (default on, ranks 1 to 3), ladder exaggeration ×2 to ×6 (default ×4), heading tape and lap clock shy or off, gear font (`display` role, default Saira Condensed 700), box font fixed (`aviation`).
9. **Open:** whether the asset rule applies in full to a stroke display; the three gaps in section 8 are offered so it can.

## 10. Sources

**F-35 HMDS, makers and program**
- Collins Aerospace, F-35 Gen III HMDS: https://www.collinsaerospace.com/what-we-do/Military-And-Defense/Displays-And-Controls/Airborne/Helmet-Mounted-Displays/F-35-Gen-Iii-Helmet-Mounted-Display-System
- F-35 Joint Program Office, HMDS: https://www.jsf.mil/hmds
- Elbit America (now Twenty-Six Defense), F-35 HMDS page (marketing only): https://www.elbitamerica.com/f-35-hmds-product
- Code One (Lockheed Martin), S. Carroll, "Lightning Sight", 2009: https://www.codeonemagazine.com/u2_article.html?item_id=12
- FlightGlobal, "F-35 takes off with helmet display", 17 Apr 2007: https://www.flightglobal.com/fixed-wing/2007/04/f-35-takes-off-with-helmet-display/

**Press and released video**
- Air & Space, R. Mola, "Super Helmet", Sept 2017: https://www.smithsonianmag.com/air-space-magazine/super-helmet-180964342/
- TWZ, T. Rogoway, F-35 HMD rolling landing video, 2019: https://www.twz.com/29569/check-out-this-f-35-helmet-display-video-of-a-rolling-landing-on-the-hms-queen-elizabeth
- TWZ, T. Rogoway, Navy F-35 helmet display videos, 2020: https://www.twz.com/12297/navy-presents-new-f-35-helmet-display-videos-and-flight-test-dangers
- Air Force Times, R. S. Cohen, green glow, 8 Dec 2021: https://www.militarytimes.com/news/your-military/2021/12/08/f-35-helmets-that-fix-green-glow-are-on-their-way-but-not-to-the-air-force/
- Penn State EOC, OLED display for the F-35 HMD: https://eoc.psu.edu/project/oled-helmet-mounted-display-improves-performance-and-cost-for-f-35/
- F-16.net forum excerpts of Avionics "Tailored to Trap" (2012) and Senate testimony (2013): https://www.f-16.net/forum/viewtopic.php?p=241959

**HUD symbology standards, manuals and research**
- FAS summary of MIL-STD-1787A: https://man.fas.org/dod-101/sys/ac/equip/mil-std-1787.htm
- NASA TM-4665, R. L. Newman et al., TRISTAR I, 1995: https://ntrs.nasa.gov/api/citations/19950017610/downloads/19950017610.pdf
- NAVAIR P-1259 (T-45C) HUD pages: https://navyflightmanuals.tpub.com/P-1259/Hud-Navigation-Symbology-85.htm, https://navyflightmanuals.tpub.com/P-1259/Bank-Scale-And-Pointer-86.htm, https://navyflightmanuals.tpub.com/P-1259/Vertical-Speed-87.htm, https://navyflightmanuals.tpub.com/P-1259/Velocity-Vector-Relationship-To-Aircraft-Waterline-141.htm
- Radiant Vision Systems, A. Corning, HUD quality considerations, 2018: https://www.radiantvisionsystems.com/blog/quality-considerations-aviation-head-displays-huds
- FlightGear F-16 HUD manual (lead only): https://github.com/NikolaiVChr/f16/wiki/HUD
- L. M. Roust, "Evaluation of head-up display formats for the F/A-18 Hornet", NPS thesis, 1989 (opened, not read in full; step 1 reads it): https://archive.org/stream/evaluationofhead00rous/evaluationofhead00rous_djvu.txt

**Typeface**
- Google Fonts, B612 Mono: https://fonts.google.com/specimen/B612+Mono/about

**Images**
- Wikimedia Commons, Category Head-up displays in aircraft: https://commons.wikimedia.org/wiki/Category:Head-up_displays_in_aircraft
- Wikimedia Commons, Category Helmet mounted displays: https://commons.wikimedia.org/wiki/Category:Helmet_mounted_displays

**Not reachable on 2026-11-28 (step 1 retries):** Aviation Today 2010, 2015 and 2018 F-35 helmet articles (moved, 404); NATO STO HFM-350 educational note (404); DTIC ADA331767 and other DTIC reports (site maintenance). WebFetch's shared hourly budget ran out after one call, so the NASA report was read once by WebFetch and the rest in the PC browser.
