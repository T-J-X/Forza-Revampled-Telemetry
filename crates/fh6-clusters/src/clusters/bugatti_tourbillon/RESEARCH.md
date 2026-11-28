# BugattiTourbillon: research note

Build #25, catalog row 9, category Hypercar, Target `recreation`, family Classic Analog, view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

**Labels.** **Cited** means the fact was read on the named page on 2026-11-28 (the link is given). **Computed** means it was worked out from cited inputs with `python3`, and the inputs and numbers are shown. **Our reading** is a conclusion or idea drawn for this dash, open to the direction check. "Unconfirmed" marks a point that no page opened today settles; it says what step 1 must check. The leads file `hypercar-clusters.json` was written from memory and was used only as a list of things to look up.

**No photo was viewed.** No tool in this session can open image bytes, so every statement about how the cluster looks comes from text written by people who saw it. Step 1 must look at the reference photos (section 2) and the Bugatti video before the board, especially for the open questions in section 9.

## 1. The real display

### 1.1 What the sources agree on

| # | Feature | Label and source |
|---|---|---|
| R1 | **Three round instruments**: a large centre dial with a smaller satellite on each side. Concepto's project manager, quoted by Faulhaber: "a large circular display in the center with two smaller satellite displays to the left and right", likened to a wristwatch with several complications. | Cited: [Faulhaber story via TimesTech](https://timestech.in/precise-information-in-the-cockpit-with-faulhaber-stepper-motors/); also [Motor1 724009](https://www.motor1.com/news/724009/bugatti-tourbillon-instrument-cluster-details/), [Time+Tide](https://timeandtidewatches.com/bugatti-tourbillon-instrument-cluster/) |
| R2 | **Centre dial = speed and rpm on concentric hands.** "The center dial includes both the speedometer on the outer rim and the tachometer inside of it" (Jalopnik). A second, also central, hand serves as the tachometer, and "the speedometer reaches a baffling 550km/h" (Oracle of Time). | Cited: [Jalopnik](https://www.jalopnik.com/i-must-twist-the-bugatti-tourbillons-fixed-hub-steering-1851551706/); [Oracle of Time](https://oracleoftime.com/conceptog-haute-horology-bugatti-tourbillon-hyper-car/) |
| R3 | **A small digital window in the centre dial**: "a small digital display at the bottom for the speed and gear selection" (Motor1); "a small rectangular concession to digitality in the speedo-cum-rpm display" (Oracle). | Cited: Motor1 724009; Oracle of Time. Position conflicts, see 1.2 |
| R4 | **Right satellite = power**, shown separately for the V16 and the electric motors ("horsepower outputs produced [by] the V16 engine and electric motors respectively", SJX); "an evocative engine power distribution readout on the right, including the hybrid powertrain data" (Oracle). Time+Tide adds battery status here. | Cited: [SJX](https://watchesbysjx.com/2024/06/bugatti-tourbillon-hybrid-hypercar.html); Oracle; Time+Tide |
| R5 | **Left satellite = temperature, fuel and battery**, three display elements carried by four hands. | Cited: Motor1 724009 (temperature, fuel, battery); Faulhaber story ("four motors are housed in the left satellite with its three display elements"); Time+Tide ("three sub-dials, also like a chronograph") |
| R6 | **Eight hands, eight stepper motors**, one motor per hand: FAULHABER AM0820 and AM1020 (8 and 10 mm). The four left-satellite motors drive their hands through a worm gear only; the other four drive "a more complex transmission that is visible through sapphire glass". The control holds a stable position when the value does not change. Rated −30 to +120 °C. | Cited: Faulhaber story. **Computed:** 4 left + 4 others; the 4 others are most likely speed and rpm (centre) and the two power hands (right). Our reading, unconfirmed |
| R7 | **The visible gear trains are the hands' real drive.** "The twin dials are driven by an intricate assembly of gears and racks that are visible in a computer-generated publicity video"; "The racks and pinions are reminiscent of strikework in a striking watch." | Cited: SJX (and its dismantled-gauge photo, section 2 #3) |
| R8 | **Skeletonised titanium, sapphire and ruby.** "A skeletonized work of art crafted from titanium, sapphire and ruby"; functional rubies are bearing jewels, "not an aesthetic detail"; "Sapphire crystal, skeletonized structures and hand-finished needles". Open-worked bridges, large jewels, bevelling (anglage) on the bridges. | Cited: [Bugatti, horology release 2025](https://newsroom.bugatti.com/press-releases/bugatti-an-enduring-and-remarable-tale-in-the-world-of-horology); [Bugatti, haute horlogerie release 2026](https://newsroom.bugatti.com/press-releases/haute-horlogerie-to-the-bugatti-tourbillon); SJX; Oracle |
| R9 | **Numerals**: on the inner aluminium disc they are laser-cut and filled with translucent ceramic, so light can pass through them. Car and Driver: "Milled sapphire crystal glass covers raised numbers and physical needles." | Cited: Oracle of Time (seen at Concepto); [Car and Driver via Yahoo](https://autos.yahoo.com/bugatti-tourbillons-gauge-cluster-unlike-201900837.html) |
| R10 | **Case**: "milled aluminum casing, elegantly skeletonized composition and crystal-housed display" (Bugatti); "set within a sculpted aluminium frame", "a polished aluminium frame" (Oracle). | Cited: [Bugatti interior release 2025](https://newsroom.bugatti.com/press-releases/bugatti-tourbillon-interior); Oracle |
| R11 | **Lighting**: "numerous LEDs are integrated in the cluster" so it is always readable; it sits in direct sun with a single ventilation opening. LEDs and PCBs had to be built into a watchmaking assembly. | Cited: Faulhaber story; Bugatti 2026 release |
| R12 | **Finishes offered**: Clous de Paris, radial guilloché, tapestry (tapisserie), engine-turned patterns that refer to Bugatti's history, aventurine, mother-of-pearl on request, stone dials, diamond-set parts considered. Chosen from physical samples. | Cited: Bugatti 2026 release; [Motor1 787157](https://www.motor1.com/news/787157/bugatti-tourbillon-interior/); Oracle |
| R13 | **Fixed hub**: the cluster stays fixed while the rim turns round it; the rim's spokes are at 12 and 6, each a pair of aluminium bars reaching round behind the cluster to the back of the hub; button clusters at 9 and 3, flat bottom. | Cited: [Bugatti launch release 2024](https://newsroom.bugatti.com/press-releases/the-bugatti-tourbillon-an-automotive-icon-pour-leternite); Jalopnik |
| R14 | **Mass and precision**: more than 600 parts, largest tolerance 50 µm, smallest 5 µm, 700 g. | Cited: Bugatti launch release 2024; repeated by Jalopnik, Motor1 and MotorBox |
| R15 | **Engine facts the scales need**: 8.3-litre naturally aspirated V16, 9,000 rpm redline, eight-speed dual-clutch gearbox, 1,000 hp engine plus 800 hp from three electric motors; the V16 drives the rear wheels only, the front motors the front. | Cited: Bugatti launch release; [Top Gear first ride](https://www.topgear.com/car-reviews/bugatti/tourbillon-passenger/first-drive) (rear-only V16, EV and combustion-only modes) |

### 1.2 Where the sources disagree (step 1 settles from the photos and the video)
- **Part count and mass.** 600+ (Bugatti 2024 and 2025, Faulhaber); 650+ (Bugatti 2026, Motor1 2026, The Drive 2026); "some 2,000 mechanical components" and 600 g (Top Gear, 2026, quoting the passenger ride). Use "more than 600" with Bugatti's 2024 figures in any copy. **The task brief's "0.5 to 50 micron" is wrong: Bugatti gives 5 to 50 µm** (largest 50, smallest 5).
- **What the centre dial's second hand shows.** Jalopnik, SJX and Oracle say tachometer. Time+Tide says a short "hour hand" points to the current gear and flies back like a flyback chronograph on each shift. **Our reading:** Time+Tide most likely saw the rpm hand drop on an upshift (SJX: "the tachometer needle bounces up or down almost like a retrograde hand"). Unconfirmed: look for a gear scale on the centre dial.
- **Where the digital window is.** Motor1: at the bottom, speed and gear. Time+Tide: at 3 o'clock, speed. Unconfirmed.
- **Right dial contents.** Power for V16 and electric (SJX, Oracle) or power plus battery (Time+Tide). The two right hands are the visible rack-driven pair (R6, R7).
- **No source mentions a tourbillon cage, shift lights, a red zone colour or the face and hand colours.** The car is named after the complication, but nothing read today puts a rotating cage in the cluster. The brief's Ø210 cage is our idea, not the car's (see 9).

### 1.3 Better real target
The brief draws one Ø520 dial with no satellites and "no centre hand". For a `recreation` the real display leads (CATALOG "Order of authority"; CLUSTER_PROMPT step 1), and the real thing is three dials with eight real hands. **Our reading:** rebuild the three-dial architecture (centre speed and rpm with the digital window, right power pair, left three-element satellite) and keep the brief's good ideas (jewel strip, G-driven light on anglage, movement that is still at idle) as how those dials are dressed and lit. This is the main question for the board (9, item 1).

## 2. Reference photos

Image content is taken from captions and alt text; none was viewed. All are Bugatti press images unless noted.

| # | What it shows (per caption) | Direct image URL | Source page |
|---|---|---|---|
| 1 | Cluster close-up | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-cluster-closeup.jpg | SJX |
| 2 | Fixed-hub wheel with the cluster above the hub | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-cluster-closeup-1.jpg | SJX |
| 3 | A gauge dismantled: racks and pinions like strikework | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-gauge-dismantled.jpg | SJX |
| 4 | Cluster dismantled, second view | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-cluster-dismantled-2.jpg | SJX |
| 5 | Cabin and steering column | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-cabin-steering-column.jpg | SJX |
| 6 | Back of the steering column (spokes behind the cluster) | https://watchesbysjx.com/wp-content/uploads/2024/06/bugatti-tourbillon-back-steering-column.jpg | SJX |
| 7 | The instruments | https://timeandtidewatches.com/wp-content/uploads/2024/06/bugatti-tourbillon-instruments.jpg | Time+Tide |
| 8 | Cockpit interior | https://timeandtidewatches.com/wp-content/uploads/2024/06/bugatti-tourbillon-interior.jpg | Time+Tide |
| 9 | Cluster, documentary episode still (2026) | https://bugatti-newsroom.imgix.net/698d8c77fcc6f41e3fecef85/02%202602%20BUGATTI-Tourbillon-BTS-Ep-13.jpg | Bugatti 2026 release |
| 10 | Cluster being hand-assembled at Concepto | https://bugatti-newsroom.imgix.net/698d8c99f169951a5573c5a4/03%202602%20BUGATTI-Tourbillon-BTS-Ep-13.jpg | Bugatti 2026 release |
| 11 | "The instrument cluster had to be a mechanical work of art" | https://bugatti-newsroom.imgix.net/698d8cf7fcc6f41e3fecef89/07%202602%20BUGATTI-Tourbillon-BTS-Ep-13.jpg | Bugatti 2026 release |
| 12 | Concepto workshop and parts (gallery) | https://oracleoftime.com/wp-content/uploads/2025/10/Concepto-Bugatti-1.jpg | Oracle of Time |
| 13 | Concepto gallery | https://oracleoftime.com/wp-content/uploads/2025/10/Concepto-Bugatti-5.jpg | Oracle of Time |
| 14 | Concepto gallery | https://oracleoftime.com/wp-content/uploads/2025/10/Concepto-Bugatti-6.jpg | Oracle of Time |
| 15 | Concepto gallery | https://oracleoftime.com/wp-content/uploads/2025/10/Concepto-Bugatti-9.jpg | Oracle of Time |

Videos: Bugatti's 'A New Era' episode on the cluster (February 2026, The Drive names it "Analogue Fascination"), on the Bugatti YouTube channel (https://www.youtube.com/user/bugattisocial); and the 2024 computer-generated launch video of the gear trains that SJX describes. Step 1 finds both and takes frame grabs of the hands moving.

Drive `references` folder: https://drive.google.com/drive/folders/1VYgiiOSChCkI7oYRlXsvDS8oL5nUUBnJ (holds `SOURCES.md`; the PC fetches the images listed there).

## 3. How the real thing behaves

| State | What the sources say | Label |
|---|---|---|
| Start-up | The engine starts with a pull lever, "a pull to start and a push to cease". The stepper motors "must start up without delay" on a freezing day. No source describes a needle sweep or a light-up sequence. | Cited (launch release; Top Gear; Faulhaber). Sweep unconfirmed: check the video |
| Idle | Each hand is held by its stepper control "or holds a stable position if the value remains unchanged". Because the gears are the hands' drive (R7), the movement is still when the values are still. | Cited (Faulhaber, SJX); "movement still at idle" is Our reading |
| Cruise | The two speed and rpm hands turn slowly over their scales; nothing animates on its own. The digital window shows speed (and gear, per Motor1). | Our reading from R2, R3, R6 |
| Pull to the limiter | The rpm hand climbs to the 9,000 rpm redline. The power hands climb together, V16 and electric shown apart. No shift lights are mentioned. | Cited redline (launch release); rest Our reading; shift lights unconfirmed |
| Gear change | The DCT shifts and "the tachometer needle bounces up or down almost like a retrograde hand". Time+Tide calls it a flyback. The instrument "responds instantly". | Cited (SJX; Time+Tide) |
| Braking | Speed hand falls; nothing else is reported. | Our reading |
| Drift, lateral G | The car is set up to drift in combustion-only mode (rear drive). No G display is reported. | Cited (Top Gear); display Our reading |
| Reverse, neutral | Reverse shows the camera on the raised centre screen in 2 s. The cluster's R and N indication is not reported. | Cited (launch release); cluster unconfirmed |
| EV mode | The V16 can be switched off and the car runs on electric. **Our reading:** the rpm hand then rests and only the electric power hand moves; unconfirmed. | Cited (Top Gear) for the mode |
| Shut-down | Push the lever in. Hand return not reported. | Cited lever; rest unconfirmed |

Motion notes for the dash (Our reading): the real hands are geared steppers, so they move fast and stop dead without overshoot. That matches the family `needle` spring (T 0.07, ζ 0.75) only loosely: give the hands a slightly stiffer, near-critical spring, and make every visible gear and rack turn by the exact ratio of its hand (section 4), so the movement and the reading can never disagree.

## 4. The maths

No row in "Real geometry and maths". The modelled figures are the gear kinematics, the brief's road-driven cage, and the wheel radius.

- **Gear chain** (textbook kinematics): a wheel train turns at ω_out = ω_in · Π(z_driver ÷ z_driven); a rack moves x = r_pinion · θ. Tests assert that each drawn wheel's angle is the hand angle times its ratio, and that meshing wheels turn opposite ways.
- **Stepper resolution.** The AM0820 has an 18° step angle ([RS listing](https://uk.rs-online.com/web/p/stepper-motors/9211347)), so 20 full steps per motor turn. **Computed:** a 270° speed scale to 550 km/h is 0.49° per km/h; the real cluster gears the motor down to reach that. The dash draws continuous angles and needs no step quantisation.
- **Wheel radius** from the launch release tyres: **Computed** front 285/35 R20 r = 0.3538 m, rear 345/30 R21 r = 0.3702 m.
- **The brief's cage at wheel rps ÷ 16, Computed** with r = 0.370 m:

| Speed | Wheel ω | Cage | Per frame at 60 fps |
|---|---|---|---|
| 10 km/h | 7.5 rad/s | 0.075 rps (27°/s) | 0.45° |
| 100 km/h | 75.0 rad/s | 0.746 rps (269°/s) | 4.5° |
| 200 km/h | 150 rad/s | 1.49 rps | 9.0° |
| 445 km/h | 334 rad/s | 3.32 rps (1,196°/s) | 19.9° |

- **Aliasing limit, Computed:** a part with z identical teeth or arms looks like it turns backwards above (360 ÷ z ÷ 2)° per frame. At 60 fps: 12 teeth 2.5 rps, 30 teeth 1.0 rps, 60 teeth 0.5 rps; a three-arm cage 10 rps. So the cage itself never aliases, but any toothed wheel geared up from it, or a fine-toothed pinion behind a fast hand, will. Tests assert that no drawn wheel exceeds its own limit (fade the teeth to a blur ring above it).
- **Tourbillon facts for the idea:** patented by Breguet on 26 June 1801; the carriage "is released and locked with each vibration of the balance", so a real cage advances in small beats; carriages commonly turn once per minute; the flying (one-side supported) tourbillon dates from 1920. Cited: [Wikipedia, Tourbillon](https://en.wikipedia.org/wiki/Tourbillon).
- **Wheel-speed units.** Unconfirmed whether `wheel_rotation_speed_*` is rad/s. Check: at near-zero slip ratio, r_eff = speed ÷ ω should land near 0.3 to 0.4 m; if it lands near 0.05 m the field is in rps. Learn r_eff live with that test.

## 5. Telemetry mapping

Signal keys are the Inputs table of `docs/TELEMETRY_USAGE_MATRIX.md`. "Verified" = long-verified against the golden packets; "Decoded" = decoded, offset confirmed at check C.

| Visual part | Input | Status | Transform |
|---|---|---|---|
| Centre outer hand (speed) | `speed` | Verified | km/h or mph; scale end per car (round up the car's top speed seen, or a fixed 0 to 400/250), angle linear |
| Centre inner hand (rpm) | `rpm`, `idle_rpm`, `max_rpm` | Verified | linear over 0 to the next whole thousand above max rpm; shift rpm from the car profile, fallback limiter × 0.97 |
| Digital window | `speed`, `gear` | Verified | tabular digits; R, N, 1 to 10+ |
| Right pair, hand A | `power` | Verified | kW or hp, scale from the peak seen per car |
| Right pair, hand B | `torque` | Decoded | fallback torque = power × 60 ÷ (2π rpm) |
| Left element 1 | `fuel` | Decoded | 0 to 1 power-reserve hand |
| Left element 2 (two hands) | `tyre_temp` | Decoded | front and rear axle means (or hottest and coolest of four) |
| Left element 3 | `boost` (turbo cars) or `g_lat` | Verified | boost hand; otherwise a lateral-G hand |
| Gear trains and racks | the hands above | derived | angle = hand angle × ratio (section 4) |
| Brief's cage | `wheel_speed`, `drivetrain` | Decoded | driven wheels' mean (FWD front, RWD rear, AWD all) ÷ 16; fallback from `speed` ÷ r_eff |
| Wheelspin cue | `wheel_speed` vs `speed`, or `tyre_slip_ratio` | Decoded | driven wheel ω·r_eff − speed |
| Light on the anglage, parallax | `g_lat`, `g_long` | Verified | light azimuth, ±2 to 4 px |
| Jewel strip | `rpm` + thresholds | Verified | outside-in or left-right fill; limiter strobe |
| Gear-change beat | `gear` edge, `clutch` | Verified / Decoded | edge on gear; clutch confirms |

**What the game cannot send, and its replacement** (one recommended in each):
1. **Electric power and the V16/electric split** (right dial). **Bold idea (recommended): a torque and power pair**, two hands on the right racked together like the real twin rack, so their gap shows where in the band the engine is. Second: power against its per-car peak hold, a lagging accent hairline.
2. **Battery charge** (left). **Recommended: a fuel power-reserve hand**, the watch complication that it most resembles (fuel is decoded). Second: for EVs (no fuel use), distance travelled this session as a 'going train' counter.
3. **Engine temperatures** (left). **Recommended: tyre temperatures on two small hands, front and rear**, the only real temperatures in the packet. Second: brake-heat MODEL from brake × speed, labelled MODEL.
4. **The deployable screen, camera, phone and media.** Not part of the cluster; dropped, nothing to replace.
5. **Warning lamps and messages.** Dropped. **Idea:** the jewel strip's two end jewels glow steady warning on a stale stream only (the app hides the dash anyway, so mostly unused).

## 6. Edge cases
- **Max rpm 0 or tiny:** hold the rpm hand at rest, scale 0 to 8 and no strip until max rpm > 1,500. **Idle near max:** clamp the shift rpm to at least idle + 10% of the range.
- **12,000+ rpm engines:** scale numerals in thousands up to 13 or more; the inner tach ring must fit two-digit numerals. **Diesels (≈4,500):** scale 0 to 5, half-thousand ticks.
- **EVs:** no idle, odd rpm and a single gear. Follow the real car's EV mode: the rpm hand may track motor rpm on its own scale; the cage still turns with the wheels; fuel becomes the distance counter.
- **Reverse:** R in the window in `warning`; cage turns backwards (wheel speed is signed or speed sign from `velocity`). **Neutral:** N at 72% ink, cage freewheels with the wheels. **Gear above 10:** two digits in the window.
- **Units:** mph scale end 250 or per car; hp and kW per user setting on the power dial.
- **Limiter bounce of ≈50 rpm a frame:** hand on its spring, strip thresholds with 150 rpm hysteresis, window digits do not show rpm.
- **Crashes, huge G:** clamp light azimuth swing and parallax; hands keep spring limits. **Airborne:** driven wheels overspeed, so the cage races ahead of the ground index (that is the brief's wheelspin cue working, not a bug); cap its drawn rate by section 4.
- **Stale or paused stream:** the app hides the dash. **Palette extremes and random palettes:** jewels and lume take the accent; never ruby red as structure.
- **Font swap:** gear and speed digits follow the user's font; raised numerals on the dials too, kept on MSDF.
- **Reduced-flash mode:** strip breathes at 2 Hz. **Colour-vision:** strip fill length, the hand position and the window digits carry every cue; colour never alone.
- **Cockpit only:** no hood variant is listed.

## 7. Signature moments

**Limiter (only the shift strip strobes, 10 Hz at 50% duty, 2 Hz breath in gentle mode).**
- A (recommended): the brief's 11 jewels set in the top bridge strobe; the rpm hand's lume tip burns steady `shift` past the redline. The real car shows no shift lights, so the strip is the one house addition, dressed as jewel bearings because the real cluster's rubies are functional bearings (R8).
- B: the strip as a row of jewels along the rim between the 12 o'clock spoke pair positions (R13), echoing the wheel's 12 o'clock mark.

**Gear change.**
- A (recommended): **the retrograde drop**, as the real tach does (SJX). On an upshift the rpm hand flies back on the fast spring, and every gear in its train visibly spins the other way at its ratio, then the window numeral changes. On a downshift it jumps up. The movement and the reading are one mechanism.
- B: the brief's escapement beat: the cage holds 50 ms, then jumps 30° as the numeral rises, grounded in the real carriage locking and releasing with each beat (section 4).
- C: the digital window rolls its gear digit like a date disc. Keep for the window only, to stay apart from `PaganiHorologerie`'s grande-date jump.

## 8. What it needs from the 4K library

Kit `bugatti_tourbillon` (PR #18 branch, `tools/asset-forge/src/prebuild/kits.rs`, `assets/clusters/bugatti_tourbillon/prebuild/manifest.json`):
- Swatch `brushed_steel_linear` (Blender, 4096² seamless, with normal and rough/metal/height): the bridges' base, 20% luminance at most.
- Sprite `jewel_lens_30` (Blender, 42 px at 1:1, three G-swing keys): the strip jewels and the jewel bearings, tinted at run time.
- Deferred by the kit to the thread: Côtes de Genève stripes; the Ø210 cage glTF.

Shared parts it can reuse: `clous_de_paris_p12` and `rose_guilloche_p24` swatches and `polished_steel_matcap` (PaganiHorologerie kit) for the Clous de Paris and radial guilloché finishes (R12) and the hands; `brushed_titanium` (KoenigseggGyro) for titanium bridges (R8); `spun_titanium` (LfaV10Nurburgring) and `spun_titanium_arc_rail` (RattrapanteSector) for the chapter rings; `dial_ring_480` (T50Floodlit) as a bezel study.

LIBRARY GAP: Côtes de Genève stripe normal and height tile, 4096² seamless, for the bridges; needs a guilloché 'stripes' mode in the hero library (Blender, asset-forge).
LIBRARY GAP: tapisserie (tapestry) guilloché tile, 4096² seamless, one of Bugatti's listed dial finishes (R12) (Blender, asset-forge guilloché mode).
LIBRARY GAP: engine-turned (perlage, overlapping circular grain) tile, 4096², the finish Bugatti says refers to its prewar cars (Blender).
LIBRARY GAP: aventurine dial texture, 4096² seamless with a seeded sparkle mask, the gem-stone dial option (ComfyUI SDXL for the stone, sparkle mask computed).
LIBRARY GAP: milled aluminium (fly-cut, circular machining marks) swatch, 4096², for the case and the inner numeral disc (R9, R10); the case shape is layout work, the surface is not (Blender).
LIBRARY GAP: a parametric watch wheel, pinion and rack recipe (crossed spokes, true involute teeth, anglage chamfer, jewelled arbor) in asset-forge so the gear trains and racks of R7 can be made at any size after the board (Blender geometry nodes).

## 9. Departures and open questions for the board
1. **One dial or three.** The brief's single Ø520 dial with no satellites departs from the real three-dial cluster. Recommended: three dials (1.3); if the board keeps one dial, list it as a departure and say why.
2. **Centre hand.** The brief says no centre hand and uses a peripheral carriage. The real dial has two concentric central hands (R2). Recommended: real hands; the carriage could stay as the rpm indication only if the board keeps one dial.
3. **The cage.** No source puts a tourbillon in the cluster. It is a house idea; if kept, keep it small, behind the window, and driven by the wheels as the brief says.
4. **Colours, numerals, typeface, red zone.** Unconfirmed from text; take them from photos 1, 7, 9. Ceramic-filled backlit numerals (R9) suggest lit numerals on a dark disc.
5. **Window position** (bottom or 3 o'clock) and what it shows: from the photos.
6. **Distinct from siblings:** `PaganiHorologerie` is two equal dials, a date window and guilloché; `KoenigseggGyro` a rotating ring; `RattrapanteSector` a fan with a split hand. This dash is a large round centre dial with concentric hands and two satellites, with gear trains and racks visibly driving the hands.
7. **Luminance.** The brief's movement at ≤20% and ≤12% of lit area stays; the real cluster's titanium can read through the G light only.

## 10. Sources

**Bugatti**
- https://newsroom.bugatti.com/press-releases/the-bugatti-tourbillon-an-automotive-icon-pour-leternite (launch, 21 June 2024)
- https://newsroom.bugatti.com/press-releases/bugatti-an-enduring-and-remarable-tale-in-the-world-of-horology (14 Feb 2025)
- https://newsroom.bugatti.com/press-releases/bugatti-tourbillon-interior (18 Dec 2025)
- https://newsroom.bugatti.com/press-releases/haute-horlogerie-to-the-bugatti-tourbillon (12 Feb 2026)
- https://www.youtube.com/user/bugattisocial ('A New Era' episodes; not opened)

**Makers and suppliers**
- https://timestech.in/precise-information-in-the-cockpit-with-faulhaber-stepper-motors/ (FAULHABER case story, Feb 2025)
- https://uk.rs-online.com/web/p/stepper-motors/9211347 (AM0820 step angle, from the listing title)

**Press**
- https://www.motor1.com/news/724009/bugatti-tourbillon-instrument-cluster-details/ (Motor1, A. Alaniz, 2024)
- https://www.motor1.com/news/787157/bugatti-tourbillon-interior/ (Motor1, 2026)
- https://www.jalopnik.com/i-must-twist-the-bugatti-tourbillons-fixed-hub-steering-1851551706/ (Jalopnik, D. Golson, 2024)
- https://autos.yahoo.com/bugatti-tourbillons-gauge-cluster-unlike-201900837.html (Car and Driver, 2024)
- https://www.thedrive.com/news/new-bugatti-tourbillon-1800-hp-v16-hybrid-coolest-gauge-cluster-ever (The Drive, 2024)
- https://www.thedrive.com/news/watch-how-bugatti-crafts-its-gauge-clusters-like-a-rolex (The Drive, 2026)
- https://www.topgear.com/car-reviews/bugatti/tourbillon-passenger/first-drive (Top Gear, Oct 2026)
- https://www.motorbox.com/auto/magazine/auto-novita/bugatti-tourbillon-foto-e-video-della-strumentazione (MotorBox)

**Watch press**
- https://watchesbysjx.com/2024/06/bugatti-tourbillon-hybrid-hypercar.html (SJX, D. Ichim and JX Su, 2024)
- https://timeandtidewatches.com/bugatti-tourbillon-instrument-cluster/ (Time+Tide, J. Weiss, 2024)
- https://oracleoftime.com/conceptog-haute-horology-bugatti-tourbillon-hyper-car/ (Oracle of Time, T. Svaboe, Oct 2025)

**Horology**
- https://en.wikipedia.org/wiki/Tourbillon

**Repo**
- `program/phases/07-cluster-factory/CATALOG.md` (brief, category, flash rules); `docs/design/families/classic-analog.md`; `docs/TELEMETRY_USAGE_MATRIX.md`; `docs/design/RESEARCH.md` §4; PR #18 branch `origin/claude/project-thread-ib67x9` (`PREBUILD.md`, `kits.rs`, the kit manifest).
