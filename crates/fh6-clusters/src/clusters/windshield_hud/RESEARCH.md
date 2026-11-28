# WindshieldHud: research note

Build #27, catalog row 16, category Apple Vision, Target `concept`, family ADAS & HUD, view raised HUD (anchor (1920, 1595), `CLUSTER_PROMPT.md` closing review line 24; `docs/design/TOKENS.md` Layout). Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on the linked page on 2026-11-28 (the page is named). **Computed** means worked out here from cited or repo numbers with `python3 -I`, inputs shown. **Our reading** is a conclusion or proposal for this dash, open to the direction check. Anything not confirmed says "unconfirmed" and what step 1 must check.

Family rules carried over from the finished `TeslaAdas` note (`crates/fh6-clusters/src/clusters/tesla_adas/RESEARCH.md`) and `docs/design/families/adas-hud.md`: a grey world with one saturated hue for what is in the path and red only for imminent action; the road ahead is predicted from the car's own motion and drawn as such, never as map or sensor data; no fake traffic warnings (blue or red on simulated objects would claim a sensor reading); stop point at d = v² ÷ (2a) from measured deceleration; every readout live from frame 1, no start-up show; the driven car's logo or custom name, never a maker's mark; the swappable font.

## 1. The benchmark

**One benchmark: the Mercedes-Benz S-Class (W223) augmented-reality head-up display**, the brief's own reference. What makes it recognisable:

| # | Feature | Label and source |
|---|---|---|
| B1 | A wide, flat virtual window, 10° wide and 5° tall, focused about 10 m ahead of the driver, "a monitor with a 77-inch diagonal". | Cited: Mercedes S-Class DIGITAL press text, reprinted by [TechPlugged](https://techplugged.com/meet-the-s-class-digital-my-mbux-mercedes-benz-user-experience/) |
| B2 | That checks: 10° × 5° at 10 m is 1.750 m × 0.873 m, diagonal 1.956 m = 77.0 in. | Computed |
| B3 | The image is made by "individual mirrors and a highly efficient light source", the technology of cinema projectors (a micromirror device). | Cited: the same press text |
| B4 | Content is registered on the road, not on the glass: animated turn arrows are "virtually and precisely projected onto the road lane". | Cited: S-Class press kit, as summarised from [Automotive World](https://www.automotiveworld.com/news/the-new-mercedes-benz-s-class-automotive-luxury-experienced-in-a-completely-new-way-2/) |
| B5 | In use: "glowing lights under the car in front" to hold a safe distance, red lines at the edge in corners, flashing arrows. | Cited: [SlashGear, 29 Jul 2020](https://www.slashgear.com/watch-the-2021-mercedes-benz-s-class-augmented-reality-hud-in-action-29631150/) (describes Mercedes' teaser video) |
| B6 | No plate, no frame: the symbols are light only, floating over the scene. | Our reading of B1 to B5 and of every HUD photo listed in section 2 |

**A better-documented structure for the same idea** (not a different Target): the two-plane AR-HUD that Volkswagen (ID.3, ID.4) and Audi (Q4 e-tron) ship. It is the clearest published description of what a civil AR-HUD shows, and the dash should borrow its two planes:

- **Far plane:** about 10 m away, diagonal about 1.8 m, holding assist and navigation content placed on the world, stabilised against the car's own motion. Cited: [Volkswagen Newsroom, 17 Dec 2020](https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730). 1.8 m at 10 m is 10.29° diagonal (Computed). Audi gives "around 70 inches", which is 10.16° (Computed), and says content can sit "considerably further ahead" than 10 m. Cited: [audi.com, Q4 e-tron AR-HUD, 14 Apr 2021](https://www.audi.com/en/electric-suvs-in-the-premium-compact-segment-the-audi-q4-e-tron-and-the-q4-sportback-e-tron-until-2026-13887/augmented-reality-head-up-display-13897).
- **Near plane:** "a flat band under the large far-range window" about 3 m away, with the speed, road signs and assist and navigation symbols as static displays (both pages, Cited).

Other real HUDs, for the board (all Cited on the pages named):

- **BMW Panoramic Vision (Neue Klasse, iX3 and i3):** projects onto "a dark-tinted, nano-coated strip" along the whole bottom of the windscreen, key driving information on the driver's side above the wheel, images that "appear in 3D"; the optional BMW 3D Head-Up Display just above it projects navigation and automated-driving information "with spatial depth, looking almost as if it is printed on the road" ([BMW Group, PreDrive BMW iX3, 11 Jun 2025](https://www.bmwgroup.com/en/news/general/2025/predrive-bmw-ix3.html)). It runs pillar to pillar onto a black printed surface, fixed widgets on the left in place of the cluster ([BMWBLOG, 23 Mar 2026](https://www.bmwblog.com/2026/03/23/bmw-explains-why-the-new-cars-have-a-wide-upper-display/)). **The dark band is a real precedent for this dash's dimming pad** (Our reading).
- **Panasonic AR HUD (CES 2021):** field of view "beyond 10 by 4 degrees" at a virtual image distance of 10 m or more, eye tracking, tilted image planes, an image-stability algorithm, updates in under 300 ms, laser holography from Envisics, static near-field cluster content plus a far AR plane ([Auganix, 11 Jan 2021](https://www.auganix.org/panasonic-automotive-introduces-its-new-augmented-reality-hud-at-ces-2021/)).
- **Envisics holographic HUD:** computes holograms "hundreds of times a second"; GM called the Lyriq system "a second plane of graphics" ([Design News, 10 Mar 2023](https://www.designnews.com/automotive-engineering/envisics-holographic-hud-technology-is-ready-for-prime-time)).
- **The DMD behind such HUDs:** TI's automotive DLP5530-Q1 is a 0.55-inch DMD of 1152 × 576 ([TI, interior display DMDs](https://www.ti.com/dlp-chip/automotive/display/overview.html)). Over 10° × 5° that is 115.2 px per degree, 0.52 arcmin per pixel (Computed). Whether Mercedes uses this part is unconfirmed.
- **Aviation, for the stroke grammar:** collimated image "perceived as existing at or near optical infinity"; boresight versus flight-path vector; an "acceleration indicator or energy cue" beside the flight-path symbol, above it when accelerating; conformal 1:1 scaling; boresight accuracy about ±7 mrad ([Wikipedia, "Head-up display"](https://en.wikipedia.org/wiki/Head-up_display)). The pitch ladder and the winged flight-path marker belong to the sibling `F35HelmetHmd`, so this dash keeps the civil-car grammar and uses only the energy cue idea (Our reading).

**What the visual review measures against (Our reading):** thin light symbols with no plate over a real road; one wide far window plus a narrow near band under it; content that sits on the road (perspective, shrinking with distance) rather than on a screen; very few colours, with red only for an imminent event.

## 2. Reference photos

| # | What it shows | Direct image URL | Source page |
|---|---|---|---|
| 1 | F/A-18 HUD gun symbology, thin green strokes over sky | https://upload.wikimedia.org/wikipedia/commons/a/ae/F-18_HUD_gun_symbology.jpeg | [Commons](https://commons.wikimedia.org/wiki/File:F-18_HUD_gun_symbology.jpeg) |
| 2 | Labelled HUD symbology diagram (boresight, flight path, horizon) | https://upload.wikimedia.org/wikipedia/commons/b/ba/Heads_up_display_symbology.png | [Commons](https://commons.wikimedia.org/wiki/File:Heads_up_display_symbology.png) |
| 3 | Carrier-deck HUD view (file title "Hud on the cat"; check which aircraft) | https://upload.wikimedia.org/wikipedia/commons/4/4f/Hud_on_the_cat.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Hud_on_the_cat.jpg) |
| 4 | BMW E60 5 Series windscreen HUD, early car HUD | https://upload.wikimedia.org/wikipedia/commons/f/f3/E60hud.JPG | [Commons](https://commons.wikimedia.org/wiki/File:E60hud.JPG) |
| 5 | Toyota Prius windscreen HUD | https://upload.wikimedia.org/wikipedia/commons/1/17/Head_Up_Display_of_a_Toyota_Prius.JPG | [Commons](https://commons.wikimedia.org/wiki/File:Head_Up_Display_of_a_Toyota_Prius.JPG) |
| 6 | Mazda CX-9 Active Driving Display with sign recognition | https://upload.wikimedia.org/wikipedia/commons/1/17/Mazda_cx-9_active_driving_display_with_traffic_sign_recongnition.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Mazda_cx-9_active_driving_display_with_traffic_sign_recongnition.jpg) |
| 7 | Lexus night-vision HUD | https://upload.wikimedia.org/wikipedia/commons/7/75/Lexus_night_vision_HUD_full.jpg | [Commons](https://commons.wikimedia.org/wiki/File:Lexus_night_vision_HUD_full.jpg) |
| 8 | Synthetic-vision primary flight display (road-like perspective terrain) | https://upload.wikimedia.org/wikipedia/commons/b/b1/Synthetic_Vision.JPG | [Commons](https://commons.wikimedia.org/wiki/File:Synthetic_Vision.JPG) |
| 9 | PZL TS-11 Iskra HUD: combiner glass and collimating lens | https://upload.wikimedia.org/wikipedia/commons/3/3b/PZL_TS-11F_Iskra_%28HUD%29.jpg | [Commons](https://commons.wikimedia.org/wiki/File:PZL_TS-11F_Iskra_(HUD).jpg) |
| 10 | C-130J co-pilot HUD over a runway | https://upload.wikimedia.org/wikipedia/commons/a/aa/C-130J_Co_Pilot%27s_Head-up_display.jpg | [Commons](https://commons.wikimedia.org/wiki/File:C-130J_Co_Pilot%27s_Head-up_display.jpg) |
| 11 | Audi Q4 e-tron AR-HUD working principle and system figures | unknown (page images) | [Audi Technology Portal](https://www.audi-technology-portal.de/en/electrics-electronics/driver-assistant-systems/audi-q4-e-tron-ar-hud-en) |
| 12 | VW ID. AR-HUD lead photo and footage (far window and near band) | unknown | [Volkswagen Newsroom](https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730) |
| 13 | Ghost image versus wedge windscreen, Figure 1 | unknown | [SPIE Newsroom](https://www.spie.org/news/6610-simultaneous-head-up-display-windshield-wedge-angle-and-layer-thickness-measurements) |
| 14 | Mercedes S-Class AR-HUD teaser (glow under the lead car, arrows) | unknown (video) | [SlashGear](https://www.slashgear.com/watch-the-2021-mercedes-benz-s-class-augmented-reality-hud-in-action-29631150/) |
| 15 | BMW Panoramic Vision band and 3D HUD | unknown (page images) | [BMW Group PreDrive iX3](https://www.bmwgroup.com/en/news/general/2025/predrive-bmw-ix3.html) |

Drive `references` folder (SOURCES.md lists every URL for the PC to fetch): https://drive.google.com/drive/folders/10sqCEtvdW1nt2gETAxoqjmy4LmxPXH-q. No image bytes were saved by this cloud run (no tool here saves images); the PC fetches the direct URLs.

## 3. How the real thing behaves, with motion notes

- **Start-up / greeting.** No source read describes an AR-HUD start-up animation (unconfirmed; step 1 checks the Mercedes and VW videos). The dash keeps the house rule: live from frame 1, show over 200 ms (TOKENS.md).
- **Idle and cruise.** The near band is static (VW, Audi, Cited). The far content is stabilised against the car's motion: Audi predicts the car's movement, estimates vertical motion from camera, radar and ESC sensors, and applies "shake compensation" a few milliseconds before output, rendering at 60 frames per second (Cited, audi.com). VW: "Less is more" (Cited). Our reading: at steady cruise nothing on the far plane moves except what the road ahead implies.
- **Approach behaviour.** VW: an advance notice on the road, then three arrows at the junction that grow as you approach while "their textures fade in order to ensure a clear view of the road" (Cited). Audi's "drone" flies ahead to the next point of action, disappears on straight road and reappears in good time (Cited). Our reading: content scales with true perspective as distance closes and thins out near the car.
- **Pull to the limiter, limiter, shift.** Real car AR-HUDs show no shift light; AMG's HUD carries an upshift bar and "Protection from reaching the overrevving range" in manual mode (lead, not opened: `dash-research/amg-display-styles.json`, AMG supplement legend). The house rule wins: only the stave row strobes (10 Hz at 50% duty, 3 on and 3 off frames at 60 Hz), 2 Hz breath in gentle mode.
- **Braking.** VW shows a red warning when dangerously close to the car ahead with the assists off; Audi a red marking plus a warning symbol when attention is needed (Cited). The dash has no car ahead: braking becomes the predicted stop point (section 5).
- **Lane edges.** Lane departure: VW draws the boundary line orange; Audi superimposes a red line on the real marking; Travel Assist shows two green lines (Cited). Mercedes shows red edge lines in corners (SlashGear, Cited).
- **Drift / lateral G.** No car HUD shows slip. The brief's β arrow follows the aviation idea that the flight path differs from where the nose points (Wikipedia, Cited).
- **Reverse / neutral / parked.** Unconfirmed for the AR window; real HUDs usually hide AR content when stopped (step 1 checks a manual). Our reading: below 3 m/s the ribbon prediction fades out over 120 ms; gear and speed stay.
- **Brightness.** A HUD must vary its luminance over a very wide range: ambient up to 10,000 cd/m² in daylight and 10⁻³ cd/m² on a moonless night, needing about 10,000:1 dimming; the HUD contrast ratio (L_display + L_ambient) ÷ L_ambient should be 1.15 to 1.5, below 1.15 the image may be undetectable, above 4 uncomfortable (Cited: [Analog Devices design note](https://www.analog.com/en/resources/design-notes/come-rain-or-come-shine1--automotive-headsup-displays-must-work-everyday.html), after NHTSA 1995). At 10,000 cd/m² ambient that is 1,500 to 5,000 cd/m² of display (Computed).
- **Ghosting.** The inner and outer windscreen surfaces each reflect the image, giving a virtual and a ghost image; to remove it their angular separation must be under about 0.2 mrad, the eye's resolution, so windscreens carry a wedged PVB interlayer (a measured HUD windscreen had 0.588 mrad), optimal for one eye height only (Cited: [SPIE Newsroom, 29 Jul 2016](https://www.spie.org/news/6610-simultaneous-head-up-display-windshield-wedge-angle-and-layer-thickness-measurements); glass n 1.518, acoustic layer 1.476 there). Polarised sunglasses can hide a HUD (AMG lead above, not opened).
- **Shut-down.** Hide in 0 ms (AGENTS rule 9, TOKENS.md); no real-car source needed.

## 4. The maths and physics

The dash has no row in "Real geometry and maths". It does model four figures; the tests can assert these (all Computed).

| Model | Formula | Values to assert |
|---|---|---|
| Crown arc | chord c = 960, sag s = 16: R = (c²/4 + s²) ÷ 2s | R = 7208.0 px; subtends 7.637°; 21 staves at 48.0 px pitch, 40 px gaps; row height 22 + 16 = 38 px (under the 42 px strobe cap) |
| Stop point | d = v² ÷ (2a), t = v ÷ a, only while decelerating | 118 mph (52.75 m/s) at 1.0 g: 141.9 m, 5.38 s; 152 mph (67.95 m/s) at 1.0 g: 235.4 m, 6.93 s; at 1.3 g: 109.1 m and 181.1 m |
| Lookahead arc | s = 3 s × v; κ = ω_y ÷ max(v, 3) (decoded, check C) or κ = a_lat ÷ v² (verified fallback, as TeslaAdas); offset (1 − cos κs) ÷ κ | cruise: s = 158.3 m; 0.18 g gives κ = 6.344e−4 /m (R 1576 m, yaw rate 1.92°/s) and a 7.94 m offset at 3 s |
| Ground perspective | a road point at z sits atan(h ÷ z) under the horizon; on screen y = f h ÷ z (the `SynthwaveOutrun` row's law) | h = 1.2 m: 21.80° at 3 m, 6.84° at 10 m, 3.43° at 20 m, 1.375° at 50 m, 0.435° at 158 m |
| Two-plane parallax | parallax ∝ 1 ÷ distance | 3 m plane ÷ 10 m plane = 3.333; if the near plane moves 4 px per g, the far plane moves 1.20 px and the horizon (infinity) 0 |
| Ghost (optional) | ghost ÷ primary ≈ (1 − R_s)², R_s Fresnel at the inner surface, n 1.52 | 60°: R_s 0.1834, ratio 0.667; 65°: 0.2357, 0.584. At 128 px/degree, 0.2 mrad = 1.47 px, 0.5 mrad = 3.67 px, 1 mrad = 7.33 px |

**Catalog correction (contrast, for the lead).** The brief asks the dimming pad (alpha ≤ 150) to let the gear pass 7:1 and the speed 4.5:1 in the blur gate. The pad passes 41.18% of the background, so with white symbols (WCAG contrast, as `tools/visual-review/src/frame.rs` uses) the gear reaches 7:1 only over backgrounds of relative luminance ≤ 0.243 (sRGB about 136) and the speed 4.5:1 only up to 0.445 (sRGB about 178). Over white sky the white gear gets 2.27:1. To reach 7:1 over Y 1.0 the darkening behind the glyph must be alpha ≥ 230; over Y 0.6, 212; over Y 0.3, 170 (Computed). So either the cap rises behind the gear and speed only (a tight glyph halo, BMW's dark strip being the precedent), or the gate is specified against named backdrops. See open question 1.

Other numbers: 8% of the 1280 × 480 window is 49,152 px; a rough lit-area estimate of the brief's parts (staves 3,696, horizon 1,280, core 3,840, gear about 5,300, speed about 5,000, ribbon about 8,000) is about 27,000 px, 4.4% (Computed, estimate). At 128 px per degree the 1280 × 480 window is 10° × 3.75°, a wider aspect than Mercedes' 2 : 1 (Computed).

**The game's camera is not the HUD's optics (Our reading).** In the cockpit view a true 10° conformal window would be 480 px wide at a 70° horizontal field of view, 336 px at 90° and 258 px at 105° (f = 1920 ÷ tan(FOV ÷ 2), Computed). The game's FOV is the user's setting and is not in the packet, so the dash cannot register symbols on the game's road. It draws its own world plane (the ribbon) and says so: the far plane is a HUD world, not the game world.

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Crown staves (rank 1) | `rpm`, `idle_rpm`, `shift_rpm` per gear | verified | outside-in, 11 steps (10 pairs, centre last) on per-gear thresholds |
| Horizon rpm core (rank 3) | `rpm_fraction` | verified | 3 px core grows centre-out to both ends at the shift rpm |
| Gear and reticle (rank 1) | `gear` | verified | R, N, 1 to 10; ticks snap 8 px in at shift |
| Speed (rank 2) | `speed`, `unit` | verified | hysteresis 0.6 units |
| Lookahead length | `speed_ms` | verified | 3 s × v, fade from 1.5 s to 3 s |
| κ arc ahead | `yaw_rate` (angular_velocity_y) | decoded, offset confirmed at check C | κ = ω ÷ max(v, 3); fallback a_lat ÷ v² from verified `g_lat` |
| Trail behind (120 nodes) | `position` x, z (bytes 244 and 252) | decoded, offset confirmed at check C | `fh6_core::trail` every 2 m (240 m); fallback: dead-reckoned from speed and κ |
| Stepped brake zones on the trail | `brake` sampled per node | verified (trail needs a brake byte per node) | accent < 20%, caution < 60%, warning ≥ 60% |
| Predicted stop bar | `g_long` (acceleration_z), `speed_ms`, `brake` | verified | d = v² ÷ 2a while braking and a > 0.1 g |
| β slip arrow and label | `velocity` x, z | verified | β = atan2(v_x, v_z), hidden below 5 m/s |
| Ribbon counter-roll | `orientation` roll | decoded, offset confirmed at check C | fallback: roll estimated as 0 (ribbon level) |
| Parallax and light azimuth | `g_lat`, `g_long` | verified | `parallax` spring ω 12 rad/s, near 4 px/g, far 1.2 px/g |
| Edge-line flash on a kerb | `rumble_strip` per wheel | decoded, offset confirmed at check C | left or right wheels on a strip light that side's path edge, steady, 150 ms crossfade |
| Brake-earlier cue | `driving_line` AI brake difference | decoded; meaning and sign unconfirmed (check C) | caution tint on the stop bar when the game says brake harder |

**What the game cannot send** (each with a replacement idea the telemetry drives; recommended one first).

1. **Navigation arrows, the drone, route and turn distance.** (a) **Recommended: the energy drone.** A small open chevron flies on the far plane at the point the car will reach in 1.5 s along the κ arc, banked by κ, and rises or sinks by the measured longitudinal G (the aviation energy cue, above when accelerating). (b) The near band shows "distance to stop" in metres while braking (the Audi turn-distance slot), and the 3 s lookahead in metres otherwise, labelled as predicted.
2. **Lane lines, lane departure, Travel Assist lines.** (a) **Recommended: the car's own path edges**, two 1 px lines one car width (1.9 m, our figure) apart along the κ arc, grey, fading with distance. (b) The kerb flash above: a wheel on a rumble strip lights that edge in `caution`, the honest version of VW's orange departure line.
3. **The glow under the car ahead (Mercedes), the lead-vehicle stripe (VW), proximity warnings.** (a) **Recommended: the stop glow.** The light Mercedes puts under the car ahead goes where your own car will stop: a soft bar on the ribbon at v² ÷ 2a, `accent` under light braking, `caution` past 0.6 g, `warning` only past 1.0 g. (b) No traffic at all: the family's ambient traffic is wrong on a HUD (it would read as sensor data on the windscreen).
4. **Speed-limit sign and set speed.** (a) **Recommended: the gear's speed band**, a bracket beside the speed from the live per-gear speed ÷ rpm table (`[f32; 10]`), showing the speed at the shift rpm in this gear. (b) A predicted speed in 1 s, v + a × 1 s, as a small ghost caret on the speed.
5. **Assist icons, warnings, telltales.** (a) **Recommended: shy chips** (family rule) for slip: TRACTION when any `tyre_combined_slip` exceeds 1 (decoded, check C; fallback rpm rate against speed rate), LIM on the limiter. (b) Leave the space dark.
6. **Range, battery, fuel.** Fuel is decoded (check C), so (a) **recommended:** a 1 px fuel line on the near band, no number. (b) A range figure from fuel and `distance_traveled`, labelled MODEL. Range itself is not in the packet.
7. **Phone, media, the BMW widgets, time, outside temperature.** (a) **Recommended: nothing.** The HUD's virtue is "less is more" (VW, Cited). (b) Lap and race time in the near band from `race_progress` (decoded, check C), shown only in a race.

Rejected for sibling overlap: gates and a remembered driving-line ribbon (`ArNavWayfinder`), pitch ladder and winged flight-path marker (`F35HelmetHmd`), clouds over the sky (a HUD never draws over the real sky; `AmgMbuxSupersport` owns the horizon haze).

## 6. Edge cases

- **Max rpm 0 or tiny, idle ≥ max:** staves and core empty, gear and speed still drawn; guard every division (telemetry.rs says both can happen).
- **Idle near max, 12,000+ rpm engines, 4,500 rpm diesels:** thresholds come from the car's own idle, shift and per-gear landing rpm, so the 11 steps always span the usable band.
- **EVs:** single gear, odd rpm: the crown follows rpm as sent; if rpm stays flat the crown follows power ÷ peak power seen this session (one stored f32), labelled on the board.
- **Reverse, neutral, gear above 10:** R and N in the reticle; 11+ shown as the number; the ribbon fades out in reverse.
- **mph and km/h, hp and kW:** the shown unit follows `unit`; lookahead and stop distance stay in metres (or feet with mph, open question 5).
- **Limiter bounce (about 50 rpm a frame):** stave thresholds use 100 rpm hysteresis; the strobe keys off a 150 ms limiter latch, not each frame.
- **Spikes and crashes (huge G):** parallax and the stop bar clamp at 2 g; κ clamps to |κ| ≤ 0.05 /m (R 20 m); a trail jump over 100 m resets (already in `trail.rs`).
- **Airborne:** v_y large and |a_z| small: no stop bar; the ribbon holds its last κ and fades after 0.5 s.
- **Stale or paused stream:** the app hides the dash; on resume no sweep.
- **Raised view and height offset:** with +240 px the anchor is at y 1355; the crown at about −300 px lands near y 1055, mid-screen, so the near band must never cover the centre of the road (open question 2).
- **Palettes:** random or dark primaries can vanish on dark asphalt and bright ones on sky: the stroke keeps a 1 px dark keyline (from the pad) in every palette.
- **Font swap:** gear and speed take the user's display font; B612 Mono stays the default for the speed and the β label.
- **Reduced flash:** the stave strobe becomes the 2 Hz breath; the kerb flash becomes a steady tint.
- **Colour only:** the stop bar's level also shows as its thickness (1, 2, 3 px); the kerb edge also thickens; β has a sign and an arrow.
- **Colour vision:** caution and warning are told apart by step thickness and position as well as hue; `visual-review` colour-vision views at step 1.

## 7. Signature moments

**Limiter and shift.**
- (a) **Recommended: the crown closes** (the brief): staves fill outside-in while the horizon core grows centre-out; they meet at the shift rpm and the crown becomes one closed arc in the shift colour, steady. On the limiter only the 38 px stave row strobes 10 Hz; horizon and reticle hold.
- (b) **The focus pull** (from the ghosting physics): through the shift stages the crown alone carries a faint ghost (ratio about 0.6 of its own alpha, capped at 25%) offset by up to 6 px, shrinking to 0 at the shift rpm, so the crown "snaps sharp" exactly at the shift point, as a wedge windscreen fuses the two images. Never on digits.

**Gear change.**
- (a) **Recommended: the reticle breathes.** The tokens' gear swap (old glyph 0.92 and fade in 50 ms, new from 1.12 with a 24 px slide) on the near plane, while the four reticle ticks, pulled 8 px in at the shift, spring back out on the `fast` spring, and the crown empties to the new gear's landing rpm over the 150 ms crossfade.
- (b) **The drone hops:** on an upshift the energy drone jumps forward by the distance covered in 0.5 s and settles back on the `fast` spring, as Audi's drone flies ahead (Cited behaviour).

## 8. What it needs from the 4K library

- **Its kit:** none. PREBUILD.md and `kits.rs` (PR #18 branch) list WindshieldHud among the dashes that are "procedural only by their briefs", with nothing layout-independent to make, and no `assets/clusters/windshield_hud/prebuild/manifest.json` exists.
- **Shared parts it reuses:** `assets/glass/blue_noise_128.png` (on main) to dither the pad feather; the repo fonts B612 Mono, Saira Condensed and Titillium Web (MSDF atlases built by the dash); the kit's feathered shapes (`fh6_kit::pool`) and the scrim's dithered smoothstep feather as a model for the pad.
- **Not used:** the `tesla_adas` car sprites (no traffic, no ego body on a HUD), `amg_mbux_supersport/prebuild/computed/horizon_haze.png` (no clouds over the sky).

LIBRARY GAP: review backdrop set for the raised HUD, five 3840 × 2160 cockpit frames (noon sky, overcast, sunset into the sun, night street, tunnel exit) so the blur gate and the pad can be judged over a real windscreen; PC capture from FH6, or ComfyUI stand-ins where capture is not possible.

Code gaps for the handover (not library): `Layout` has no raised view (`View` is Cockpit, Hood, Chase) and `LayoutTokens::raised_anchor` is not read anywhere, so the dash would offset itself by −240 px or the kit adds the view; `fh6_core::trail` stores only x and z and `get_history_nodes` returns a `Vec` (an allocation each call).

## 9. Departures and open questions for the board

1. **Pad contrast.** Keep the 150 cap for the wide pad, add a tight halo up to about 230 behind gear and speed only, or judge against the backdrop set? (Section 4 numbers.)
2. **Two planes.** Recommended: near band (3 m, gear and speed) and far plane (10 m, crown, horizon, ribbon), as VW, Audi and Panasonic do. This changes the brief's single stack and its parallax (near 4 px/g, far 1.2 px/g, horizon 0).
3. **The ribbon is a HUD world, not the game road.** The game's FOV is unknown, so nothing claims registration; say it on the board.
4. **Colour.** Real AR-HUDs are full colour (DMD, LCD); aviation HUDs are monochrome. Recommended: palette primary for strokes, accent, caution and warning only for the stop bar and kerb flash, red only past 1.0 g. Whether aviation green is a preset is open (no source read on green phosphor; unconfirmed).
5. **Units of distance:** metres always, or feet with mph?
6. **"Dashed κ prediction 3 s ahead" against "faded out by 1.5 s"** in the brief: read as solid to 1.5 s, dashed and fading from 1.5 s to 3 s; confirm.
7. **The 1280 × 480 window** inside the 1440 × 560 envelope: keep 8 : 3, or match Mercedes' 2 : 1 (1120 × 560)?
8. **Ghost realism (section 7b):** optional, off by default? Real premium HUDs remove it.
9. **AI brake difference:** use it only after check C confirms its meaning and sign.

## 10. Sources

Benchmark and car HUDs
- Mercedes S-Class DIGITAL press text (reprint): https://techplugged.com/meet-the-s-class-digital-my-mbux-mercedes-benz-user-experience/
- S-Class press kit (summary): https://www.automotiveworld.com/news/the-new-mercedes-benz-s-class-automotive-luxury-experienced-in-a-completely-new-way-2/
- SlashGear, S-Class AR-HUD in action: https://www.slashgear.com/watch-the-2021-mercedes-benz-s-class-augmented-reality-hud-in-action-29631150/
- Volkswagen Newsroom, AR head-up display: https://www.volkswagen-newsroom.com/en/press-releases/from-the-luxury-class-to-the-compact-segment-the-augmented-reality-head-up-display-6730
- audi.com, Q4 e-tron AR-HUD: https://www.audi.com/en/electric-suvs-in-the-premium-compact-segment-the-audi-q4-e-tron-and-the-q4-sportback-e-tron-until-2026-13887/augmented-reality-head-up-display-13897
- Audi Technology Portal: https://www.audi-technology-portal.de/en/electrics-electronics/driver-assistant-systems/audi-q4-e-tron-ar-hud-en
- BMW Group, PreDrive BMW iX3: https://www.bmwgroup.com/en/news/general/2025/predrive-bmw-ix3.html
- BMWBLOG, Panoramic Vision: https://www.bmwblog.com/2026/03/23/bmw-explains-why-the-new-cars-have-a-wide-upper-display/
- Auganix, Panasonic AR HUD at CES 2021: https://www.auganix.org/panasonic-automotive-introduces-its-new-augmented-reality-hud-at-ces-2021/
- Design News, Envisics holographic HUD: https://www.designnews.com/automotive-engineering/envisics-holographic-hud-technology-is-ready-for-prime-time
- TI, automotive interior display DMDs: https://www.ti.com/dlp-chip/automotive/display/overview.html

Optics, luminance, aviation, type
- Analog Devices, HUD dimming and contrast: https://www.analog.com/en/resources/design-notes/come-rain-or-come-shine1--automotive-headsup-displays-must-work-everyday.html
- SPIE Newsroom, HUD windscreen wedge and ghosting: https://www.spie.org/news/6610-simultaneous-head-up-display-windshield-wedge-angle-and-layer-thickness-measurements
- Wikipedia, Head-up display: https://en.wikipedia.org/wiki/Head-up_display
- Google Fonts, B612 Mono (designed for Airbus cockpit screens by Intactile DESIGN and ENAC): https://fonts.google.com/specimen/B612+Mono/about

Images: the Wikimedia Commons pages in section 2.

Leads not opened (budget or access): IEEE Spectrum "Head-Up Car Displays Coming in 2024" (https://spectrum.ieee.org/augmented-reality-car-hud); Mercedes owner's manual AR-HUD pages (moba.i.mercedes-benz.com returned 404); TI white paper on solar loading in AR HUDs (linked from the TI page); NHTSA 1995 HUD human-factors review; the AMG HUD legend in `dash-research/amg-display-styles.json`.

Repo: `CATALOG.md` (brief, Apple Vision, Flash and telemetry), `CLUSTER_PROMPT.md`, `docs/design/TOKENS.md`, `docs/design/families/adas-hud.md`, `docs/TELEMETRY_USAGE_MATRIX.md`, `crates/fh6-core/src/telemetry.rs` and `trail.rs`, `crates/fh6-kit/src/cluster.rs` and `tokens.rs`, `tools/visual-review/src/frame.rs`, `tesla_adas/RESEARCH.md`, PR #18 `PREBUILD.md` and `kits.rs`.
