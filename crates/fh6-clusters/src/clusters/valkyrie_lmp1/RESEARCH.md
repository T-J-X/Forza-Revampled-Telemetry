# Valkyrie LMP1: research (step 1)

Target `recreation`: the **Aston Martin Valkyrie** road car's driver display. Read on 2026-11-28 from the pages below (press release, road tests, the encyclopaedia entry, and the review of the maker-licensed replica wheel). Every point is marked with its source; a point marked *not established* was looked for and not found.

## The real display

The Valkyrie has no instrument binnacle. "All the vital signs [are] shown on a single OLED display screen" on the steering wheel, which carries all the switchgear (Aston Martin press release, 2017; repeated by Wards and Goodwood). The wheel is a "detachable oblong" F1-style wheel (Goodwood), in "bespoke carbonfibre" with "a screen at its centre giving you essential information" (evo, 2025). Rear view comes from cameras feeding screens at the base of each A-post (press release; evo, PistonHeads, Top Gear, Car).

What the sources say the screen shows:

| Feature | What the sources say | Source |
|---|---|---|
| Rev counter | "dominated by a huge rendered rev counter with numbers that go up to 12" | PistonHeads |
| Rev counter colour | the maker-licensed replica's "official Valkyrie dashboard interface" shows a blue tachometer, central | TweakTown (Simagic replica, image text) |
| Gear, status | the replica's screen shows the gear and ABS / traction-control indicators | TweakTown (replica) |
| Speed | "my steering wheel read-out shows 187mph": a digital speed on the wheel screen | Car |
| Shift lights | "the blue shift-lights on the steering wheel"; "the first change-up light comes on, well short of the limiter" | evo; PistonHeads |
| Shift-light layout | on the replica, above the screen: "a single light on each side flanks the central partition, which includes another eight LEDs" (ten in all) | TweakTown (replica) |
| Engine | redline 11,100 rpm (evo, Car, Wikipedia; PistonHeads says 11,200); peak power at 10,500 rpm (Wikipedia, evo body text) | as named |
| Drive modes | three: Urban, Sport and Track, on a rocker or scroll controller at the wheel's lower left | Top Gear, evo, PistonHeads, Car |
| Wheel controls | indicators, nose-lift, lights, wipers, start/stop, neutral, launch control, a KERS push button; "dials and switches sit beside the wheel screen" | Top Gear, PistonHeads, Car, Wikipedia |
| Wheel material | carbon fibre; the replica uses "contoured 3D carbon fiber", Alcantara grips and magnesium | evo; TweakTown (replica) |

*Not established:* the screen's size and resolution, the typeface, the exact rev-counter geometry (sweep, where 0 and 12 sit, the red zone), the colours of the gear and speed, whether the shift lights change colour or flash at the limiter, and the weave of the wheel's carbon. No reference picture could be fetched into the container (the replica review's site refused the proxy), so these stay open, and the dash follows the house rules where the sources are silent.

### Sources
- Aston Martin press release, 12 Jul 2017: https://media.astonmartin.com/aston-martin-valkyrie-secrets-of-exterior-and-interior-design-revealed/
- Wards, 13 Jul 2017: https://www.wardsauto.com/interiors-ux/aston-martin-applies-formula-one-formula-to-valkyrie
- Goodwood, 12 Jul 2017: https://www.goodwood.com/grr/road/news/aston-martin-valkyrie--8-things-we-now-know/
- The Drive, 2 Nov 2018: https://www.thedrive.com/news/24648/aston-martin-shows-off-the-valkyries-f1-inspired-cabin-in-new-photos
- evo road test, 2025: https://www.evo.co.uk/aston-martin/valkyrie-coupe/205739/aston-martin-valkyrie-2025-review-700-miles-in-the-worlds-most
- PistonHeads road test, 2023: https://www.pistonheads.com/news/ph-driven/2023-aston-martin-valkyrie--ph-review/46920
- Car road test, 2023: https://carmagazine.co.uk/car-reviews/aston-martin/valkyrie-hypercar
- Top Gear review and interior page, 2023: https://www.topgear.com/car-reviews/aston-martin/valkyrie and https://www.topgear.com/car-reviews/aston-martin/valkyrie/interior
- Wikipedia: https://en.wikipedia.org/wiki/Aston_Martin_Valkyrie
- TweakTown, review of the maker-licensed replica wheel: https://www.tweaktown.com/reviews/11650/simagic-aston-martin-valkyrie-steering-wheel/index.html
- Valkyrie LM (2025), "lighting and a built-in screen" on the wheel: https://www.designboom.com/technology/aston-martin-valkyrie-lm-screen-steering-wheel-racing-hypercar-06-11-2025/

## What makes it recognisable, and what the dash does

The real display is a small screen on a wheel, so by step 1's rule the dash takes its parts, not its shape: a carbon wheel face around a true-black OLED window, the ten blue shift lights in the replica's 1 + 8 + 1 grouping across the top, a big round rev counter numbered 0 to 12 that fills the screen, the gear at its centre, a digital speed, and rotary dials beside the screen.

| Recognisable feature | In the dash |
|---|---|
| Carbon F1-style wheel face, screen at its centre | Matched: a forged-carbon wheel face (Blender, on the pre-built ComfyUI forged carbon) with angled shoulders round a true-black window. The weave is not established; forged carbon follows the brief. |
| "Huge rendered rev counter" numbered to 12 | Matched: a round rev counter that fills the window, numbered in thousands from the car's own limiter (0 to 12 on an 11,100 rpm car), lit arc in `primary`. |
| Blue tachometer | Matched by the family palette: Race Dash's `primary` is blue (#00B4FF). A user palette changes it. |
| Ten blue shift lights, 1 + 8 + 1 | Matched: ten LED lens pods above the window, the eight in a centre partition and one on each shoulder. They fill outside-in in `primary` (blue) and take the `shift` role at the shift point (house rule: the shift colour is a palette role). |
| Gear and speed on the screen | Matched: the gear inside the dial's centre (rank 1), tabular speed beside the dial (rank 2). |
| ABS and TC indicators | Kept as derived lamps (from body slip and G, as BoschDdu derives them), labelled derived. |
| Rotary dials beside the screen | Matched: knurled anodised encoder rings on the shoulders (pre-built Blender knurl), carrying data arcs. |
| Urban / Sport / Track modes | Not in the telemetry. Deliberate change: no mode readout, so nothing claims a mode the game does not send. |

### Deliberate changes from the catalog brief, and why
- **The predictive lap-delta crown, the lap delta readout and the sector strip are dropped.** They need lap, distance and timing fields that `fh6-core` does not decode, and adding them is a shared-code change outside a dash thread (CLUSTER_PROMPT.md, "Edit nothing outside them"). They are also not on the real Valkyrie's screen as the sources describe it. Written in the handover as a request, not done.
- **The perimeter tach becomes the real car's round rev counter** ("numbers that go up to 12").
- **Fifteen LEDs become ten** (the replica's 1 + 8 + 1).
- **TC and ABS intervention arcs, tyre temperatures, fuel and boost** need slip ratio, tyre temperature and fuel, which are not decoded (the Valkyrie is also naturally aspirated, so boost reads nothing). The left encoder shows the pedals (throttle outer, brake inner); the right encoder shows power as a share of the car's measured peak, in `signals.power_unit`.
- **The engineer trace stays** (throttle, brake, steer over the last 4 s, 3 s in the hood view): it is the brief's rank 4 and reads only verified fields.

### House rules that outrank the real display
Rank order (gear and shift lights, then speed, then the tach), OLED black outside the hardware, palette roles instead of fixed colours, the swappable font (`race` Saira Condensed by default), the flash rule (at the limiter only the LED strip strobes at 10 Hz; the dial's red-zone rim holds a steady `shift` and never flashes), and telemetry honesty (nothing shown without a decoded or derived source).

## Second pass, 28 Nov 09:10: reference photos and the texture plan

**Reference photos.** Looked again for a picture of the real wheel screen. Four more pages were read (The Drive, 2 Nov 2018: "a race-inspired display which shows the gauge cluster and most important vehicle status information"; HotHardware, 2018: a "rectangular steering 'wheel' with a large embedded display"; The Week, 12 Jul 2017: "three OLED displays … The first acts as an instrument panel"; designboom, 11 Jun 2025, the Valkyrie LM: "lighting and a built-in screen to show the user their driving information"). None of them describes the screen's layout, colours or typeface, and the proxy refuses the image hosts (designboom, The Drive, Wikimedia Commons), so no photo could be looked at. V15 (the look matches the reference) stays *not assessed*; the open points in "The real display" stay open. The new pages add nothing that changes the board.

**How the new forged carbon goes onto the wheel face.** The wheel face is the only part of this dash that uses a texture; everything else on it is the hero library's procedural materials (anodised partition, knurled encoders, LED caps, cover glass, piano-black wells).

| Step | Plan |
|---|---|
| Which file | The pre-build kit's `forged_carbon_marble` as the manifest lists it when the cloud batch ends. Today that is seed 90002 with the "flat polished panel" prompt (kit commit 06a4eff); the recipe (`tools/asset-forge/src/recipes/valkyrie_lmp1.rs`, `FORGED`) now carries the same prompt, negative and seed, so the cache reuses the file instead of making it again. If the 4090 batch changes any of the three, `FORGED` changes with it before the build runs. |
| Check before use | SHA-256 against the manifest; a 1:1 crop of the 4096² master and a 2 × 2 tile (seams); the batch-1 failure signs are absent: no rock or coal relief, no beige or white specks, no grid. Strands read as flat fibre under resin. |
| Scale | `TILE_PX` 900: one 4096² tile covers 900 local px of the cockpit face (4.5 texels per screen px, so no mip blur at 4K), 630 px in the hood view (scale 0.7). The yoke is 1400 px wide, so about 1.6 repeats across it; with a seamless master no seam shows, and the marble is irregular enough that the repeat does not read. If the strands come out finer than about 6 px on screen, raise `TILE_PX` (fibre that fine turns to noise at seat distance). |
| Material | `mat::forged`: the texture as base colour, a normal from its luminance at low strength, and a clear coat. The strands carry a satin sheen; the resin is glossy. The three key lights (-135°, -160°, -110°) and the G swing between them make the strands glint as the car corners, which is why the plate has three keys. |
| What stays dark | The face must not out-shine the data. Its brightest glint at the rest key stays below the dimmest lit readout (the gear's ink); the cluster already dims the plate through the tint (`BAKED_GAIN` 0.55), and the closing review checks it with `stretch` and `glance`. |
| After the renders | The PC renders `wheel_{cockpit,hood}_k{0,1,2}.png` to Drive `valkyrie_lmp1/build1` with sizes and SHA-256. This thread downloads them, checks the hashes, commits them with their `ASSETS.md`, re-renders the eight states, and runs V16 and V17 on the result. |

No board change came out of this pass, so Penpot stays as v2.

## Third pass, 28 Nov 10:40: how the real car behaves, motion, telemetry and edge cases

### How the real car behaves (driver's view)
| Behaviour | What the sources say | Source | What the dash does with it |
|---|---|---|---|
| Idle | "savagely loud at a 1000rpm idle" | Autocar (autocar.co.uk/node/125694) | At idle (1,000 of 11,100) the first tunnel ring is lit (0.63 > 0.5), so the dial is never dead with the engine running. Keep. |
| Limiter | 11,100 rpm (Aston Martin 2019 powertrain release, Wikipedia, Top Gear: "keep looking down to ensure you don't hit the 11,100rpm limiter"); Hagerty UK's 11,200 is the outlier | astonmartin.com 2019-3-1; topgear.com | The scale top is the car's own `max_rpm` rounded up (12 for 11,100), as the sources' "numbers that go up to 12". Keep. |
| Heat derate | the redline drops "as low as 8,000rpm" when hot (Hagerty UK); "cut its redline when the coolant got too hot" (R&T via Yahoo) | hagerty.co.uk; autos.yahoo.com | FH6 sends no temperatures and no live limiter, so nothing to show. Not faked. |
| Power and torque | engine 1,000 bhp at 10,500 rpm, 740 Nm at 7,000 rpm; system 1,160 bhp at 10,500 rpm, 900 Nm at 6,000 rpm | Aston Martin 2019 | Power peaks just under the limiter, so the right encoder's share arc and the 4-digit readout both peak in the red band. 1,160 hp and 865 kW fit the 4-digit cap. |
| Shift lights | "blue shift-lights"; "progressive illumination of the change-up lights"; the first light comes "well short of the limiter" | evo; R&T via Yahoo; Autocar; Hagerty UK | Matches the family stages (first light at 75% of the shift point), blue `primary`, outside-in fill. Keep. Whether the real lights change colour or flash at the shift point is *not established*; the house shift colour and the 10 Hz limiter strobe stay. |
| Gearbox | Ricardo 7-speed single-clutch AMT: "so slow on the way up … slurs a shift"; "brutal shifts"; "won't upshift automatically"; "can change down through the gears autonomously" | Top Gear; Hagerty US; evo | The gear digit is the rank-1 element and changes the frame the packet does; no animation that could lag a slow real shift. The auto-downshift means gears can drop with no driver input: the digit just follows. |
| Pull-away and reverse | "pulls away electrically, then bleeds the clutch in automatically around 10mph"; "backing up always done electrically" | Top Gear; Hagerty UK; evo | In FH6 the gear and rpm come from the game, so `R` and `N` show as sent. |
| ERS boost | 1.8 kWh Rimac pack, "ERS button for a 143bhp e-boost", "you have to choose when to deploy it" | Top Gear; Hagerty UK; Car ("marked KERS") | Not in the telemetry (no battery or deploy field). Not faked. A user who wants it can read boost-like surges in the power share. |
| Grip | road car "up to 2G of lateral acceleration on the Cup 2 tires"; AMR Pro 3.3 g lateral, 3.5 g braking | R&T via Yahoo; Wikipedia | **Finding:** the friction circle's full scale is 1.5 g (`FC_FULL_G`), so on this car the dot pins to the rim through most fast corners and the information is lost. Plan: full scale 2.5 g with faint rings at 1 g and 2 g (decided on the board after LIBRARY DONE). |
| Speed | 220 mph limited (evo), 224 in its text; AMR Pro 250 mph | evo; Wikipedia | Three digits in either unit (402 km/h max). Fits the 3-cell readout. |
| Wheel | "oblong", "squared-off", detachable, carbon with a screen at its centre; the licensed replica is 320 × 156 mm with a 5-inch screen, 8 buttons, 2 five-way switches, start/stop | Car; Autocar; Luxury London; TweakTown (replica) | The face keeps the oblong yoke with angled shoulders. The replica's 2:1 face proportion (320:156) is close to the yoke's 1400:512. |

*Not found in any source:* rev speed from idle to the limiter, hard cut or bounce at the limiter, shift times, downshift blips, launch rpm and light pattern, the real shift-light count, colour change at the shift point, the screen's mode pages, and any ERS indicator on screen. The Forza Horizon 6 Valkyrie is class R, rear drive, semi-slicks (Dexerto's FH6 car page); no power curve was found.

### Motion notes (what moves, how fast, and why)
| Element | Motion | Timing |
|---|---|---|
| Shift lights | fill outside-in with rpm, all `shift` at the shift point, strobe on the limiter | per frame; 10 Hz, 3 on / 3 off (only flashing element; reduced-flash holds steady) |
| Rev arc and tunnel | arc head follows rpm; rings light back to front up to the limiter, all `shift` on it | per frame, no smoothing (the packet is already 60 Hz) |
| Gear | swaps on the frame the packet changes | instant (a slow real shift must not be made to look slower) |
| Speed | tabular, no roll | per frame |
| Power readout | hysteresis step 5 so it stands still on the limiter | per frame |
| Power share arc | smoothed | tau 0.10 s |
| Friction circle | dot plus a 0.5 s comet of 8 samples | per frame |
| Trace | scrolls left | 4 s window (3 s in the hood view) |
| Wheel face | key light swings with lateral G, the plate crossfades between the three render keys, 4 px parallax | follows the virtual light |
| Lamps | TC and ABS fade in on their derived conditions | `AlarmColumn` timing |
Idle with the car still: nothing moves (checked in the direction check).

### Telemetry mapping
| Field | Drives |
|---|---|
| rpm, max_rpm | rev arc, scale top, numerals, red band, tunnel, shift lights (with the palette's shift point) |
| gear | the gear digit (R, N, 1-10) |
| speed | the speed readout, unit from the config |
| power | power readout (in `power_unit`) and the share arc against this car's peak |
| car_ordinal | resets the power peak |
| throttle, brake | left encoder arcs, trace |
| steer | trace |
| g_lat, g_long | friction circle; g_lat swings the face's light |
| velocity, drivetrain | the derived TC lamp (with slip from `Signals`) |
| brake, g_long | the derived ABS lamp |

### Edge cases to test before the closing review
1. **Housing off** (the new OSD switch): with the face gone, the encoder labels read much dimmer (brightest pixel 46/255 against 117/255 with the face, measured on `03_redline_strobe` crops, `/mnt/project-files/valkyrie_lmp1/housing_on_off.png`). **Cause found:** `bezel::on_screen` turns a design level into linear coverage *over an opaque black body* (its doc says so). The face was that body. With it off, the labels and the encoders' unlit tracks sit over the overlay's transparent pixels, which the desktop compositor blends in encoded space, so the linear coverage shows as itself: on_screen(0.47) = 0.187, about 48/255, as measured. Fix planned (after LIBRARY DONE): parts outside the window use the design level directly as alpha when the face is off, and the same check over a bright game frame (`--backdrop`). The window's contents are unaffected (they sit on its opaque black fill).
2. **2 g and more:** the friction circle pins at 1.5 g (above).
3. **Low-revving cars** (trucks, diesels): scale top clamps at 4; check the numerals and tunnel spacing at 4 and 5.
4. **20,000 rpm cars:** scale top 20, 21 numerals; check numeral crowding on the 240° arc.
5. **Power at 9,999 hp or kW:** clamp holds four digits; negative power on the overrun reads 0.
6. **Car change mid-session:** power peak resets; the share arc starts from the new car's first sample.
7. **Reverse and neutral:** `R` and `N` in the gear digit; the TC lamp stays off when not moving forward.
8. **Packets stop (pause, menu):** the dash holds its last frame and stops animating.
9. **Reduced flash on:** the limiter strobe holds steady `shift`.
10. **Hood view:** every element above stays over the size floors at 0.7 scale.
11. **Front-drive and AWD cars:** the TC lamp's drivetrain condition (no power oversteer lamp on front drive).

## Fourth pass, 28 Nov 10:55: the full FH6 packet (main 60c99c9)

The decoder now reads all 88 fields, including `tyre_slip_ratio`, `tyre_temp`, `torque`, `fuel`, `boost` and `race_progress` (laps, best and last lap, race time, position). Real-game check C still has to confirm their offsets and units, so nothing here ships before it. What the dash should do with them, in order of value:

| New field | Plan | Board change? |
|---|---|---|
| `tyre_slip_ratio` | **The TC and ABS lamps stop being derived.** TC lights when a driven wheel's slip ratio passes about +0.15 under throttle; ABS when any wheel's passes about -0.15 under brake. This is what the real systems react to, so the "derived" label goes and the body-slip and g thresholds go with it. The DASH_SCENARIOS and the matrix row change to `tyre_slip_ratio` (with `drivetrain` for which wheels are driven). | No: same lamps, same place. |
| `tyre_temp` | The brief's tyre tags came back as data: four small corner pips round the friction circle (front left, front right, rear left, rear right), coloured cold / working / hot on the palette's go and warning roles, so the circle shows grip used and the state of the tyres in one place. | Yes, a small one, in Penpot after LIBRARY DONE. |
| `race_progress` | A lap-delta line (current lap against best, in tabular digits) under the trace, shown only while a lap is timed (`current_lap` > 0); in free roam it stays off and the trace keeps the space. The real Valkyrie screen shows no lap timing, so this is a race-mode extra, not a recreation feature. | Yes, in Penpot. |
| `torque` | Not used: power already sits on the right encoder and readout, and two engine-output readouts would compete. | No. |
| `boost` | Not used: the car is naturally aspirated, so it would read zero. | No. |
| `fuel` | Not used: FH6 has no fuel strategy in free roam, and the real screen shows none. | No. |

The rev tunnel and the friction circle (the board v2 replacements for the dropped parts) stay: they are data-driven and they are on the board. The plan above adds to them; it does not bring back the lap-delta crown or the sector strip.
