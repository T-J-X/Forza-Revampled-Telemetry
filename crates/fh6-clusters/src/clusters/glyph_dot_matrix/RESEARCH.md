# GlyphDotMatrix: research note

Build #21, catalog row 6, category Retro, Target `concept`, family Modern Digital 2027, view cockpit only. Pre-researched 2026-11-28 by the thread 'Research the upcoming dashes ahead' before this dash's own thread started. The dash thread checks every point at step 1 and extends it. Facts, maths and ideas only: the layout is designed on the Penpot board after LIBRARY DONE.

Labels: **Cited** means read on 2026-11-28 from the source linked. **Computed** means worked out here from cited or brief values, with the inputs and numbers shown (Python 3 with numpy). **Our reading** is a conclusion or proposal for this dash, open to the direction check. "Unconfirmed" marks a lead that step 1 must still check. The shared web-fetch budget ran out early in this run, so pages were read through GitHub (the Nothing developer kit, the Ndot specimens, two LED datasheets), the PC's built-in browser (Engadget, Beebom, The Drive) and two fetches (Fonts In Use, Hagerty). Pages found but not read are listed in section 10 as leads for step 1.

**Earlier work.** `git log --all -- crates/fh6-clusters/src/clusters/glyph_dot_matrix` shows only three commits, none of them design work: `33dcc78` (2026-11-22, the Phase 7 scaffold), `33580f3` (2026-11-27, a fix phase that carried the scaffold) and `2d1f114` (2026-11-28, the code re-cut). No branch name contains `glyph` or `dot`. The real earlier work is the **v1 tiny-skia dash** `crates/fh6-clusters/src/clusters/glyph_dot_matrix.rs` (205 lines, from the initial import `f1b3a3c`, removed by `33dcc78`; its goldens went in `e629575`, "Remove v1"). It drew exposed PCB traces, a field of registration crosses, five perimeter "glyph strips", a circular dot-matrix tach of rings and stations with an inner power ring, a 5×7 dot gear, 7×9 dot speed digits and "OP-1 synthesizer" readout modules with a torque figure (`hp * 5252 / rpm`, imperial; an earlier version used `hp * 0.5`, flagged in `docs/specs/phase-8-materials-and-shaders.md`). AGENTS.md also points to a later copy at `21e8a06:crates/fh6-clusters/src/legacy/glyph_dot_matrix.rs`; that commit is not in this clone, so step 1 checks it. The catalog's Risk line records why it failed: rings and a canvas-wide grid dominated and the gear was tiny. `2e4d190` (2026-11-23) marked the dash started in the old table, but no Phase 7 branch or commit followed. Step 1 must look for a Penpot page `07 · GlyphDotMatrix` from before 2026-11-27 (this run had no Penpot access).

## 1. The benchmark

**The one benchmark: the Glyph Matrix on the back of the Nothing Phone (3)**, read through its official developer kit. The category note says the phone is "a benchmark for the dots, not the subject": the review measures the dot optics against it, while the subject stays the LED dot-matrix display of the 1980s (see 1.2).

### 1.1 What makes the Glyph Matrix recognisable

| # | Feature | Label and source |
|---|---|---|
| B1 | A 25 × 25 addressable grid on Phone (3) (13 × 13 on Phone (4a) Pro); a frame is a 25 × 25 integer array. | **Cited**, GlyphMatrix Developer Kit README, "Supported Devices" and `setMatrixFrame(int[])` |
| B2 | Only a disc of the grid is populated. Counting the kit's LED-allocation figure gives 489 lit positions and 136 empty corners; row widths from top to bottom are 7, 11, 15, 17, 19, 21, 21, 23, 23, then seven rows of 25, then the mirror. | **Computed** from `image/23111_25111_LED_allocation.svg` (625 drawn cells: 487 white plus 2 inside yellow markers, 134 at 10% opacity plus 2 outside yellow markers) |
| B3 | The mask is exactly "every cell whose centre lies within 12.5 pitches of the centre cell": that rule gives 489 and reproduces every row width; 12.25 gives 481 and 12.6 gives 497. π × 12.5² = 490.9. | **Computed** |
| B4 | Each object has a brightness of 0 to 255 (0 is LED off); objects also take position, rotation, scale 0 to 200 and transparency. A frame stacks at most three objects, one per layer (top, middle, low). | **Cited**, README, `GlyphMatrixObject.Builder` and `GlyphMatrixFrame.Builder` |
| B5 | Text is a first-class object (`setText`), so the matrix is read as a character display as well as a picture. | **Cited**, README |
| B6 | Toys respond to the back button (short press cycles, long press sends "change") and may use the gyroscope and accelerometer; Nothing names a "Magic 8 Ball" and a "Leveler" toy. An always-on toy is woken once a minute (`EVENT_AOD`). | **Cited**, README, "User Interaction with Glyph Button" and "Toy with AOD capability" |
| B7 | The house typeface Ndot was designed in 2021 by Colophon "for exclusive use by tech company Nothing". | **Cited**, Fonts In Use, Ndot |
| B8 | The Nothing wordmark sits on a 5-row grid of round dots: horizontal pitch 30.7 px with 30 px dots (dots touch along a row), vertical pitch 46.2 px (cells 1 : 1.5). Ndot 57 is a 5 × 7 face whose dots also touch within strokes. | **Computed** by pixel runs on the specimen `Ndot-55a.png` in the fan repository xeji01/nothingfont (a lead, not a maker source); Ndot 57 chart in the same repository |
| B9 | "The matrix is made of 489 LEDs"; it shows "monochromatic images", replaces the flashing light strips of earlier Nothing phones, and is driven by a button under the glass back. Uses named: notifications, weather and battery widgets, a stopwatch, a "low-res mirror", a camera countdown, games. | **Cited**, Engadget hands-on (1 July 2025) |
| B10 | "489 micro LEDs" acting as a secondary screen; nine toys at the time of writing (digital clock, battery, solar clock, Glyph mirror, spin the bottle, rock paper scissors, Magic 8 Ball, Leveller, stopwatch); a volume animation; manual or auto brightness; always-on toys "with reduced brightness"; two sound engines, "Diode" and "Chemical", each with its own animations. | **Cited**, Beebom Glyph Matrix guide |
| B11 | The physical diameter and dot pitch of the matrix. | Unconfirmed: no source read here gives them; step 1 looks for a teardown or the spec sheet |

**Our reading of B1 to B8:** the Glyph look is (a) round dots, never squares; (b) off positions read as dark glass, with no glowing grid; (c) grey levels per dot (a 256-step image, not on/off); (d) text and pictograms that are clearly built from the same dots; (e) a hard circular silhouette made of whole dots. The dash keeps (a) to (d). It does not copy the disc as a big circle: a 25 × 25 disc would bring back the canvas-wide grid that failed v1. It may use the disc at small size (the class badge or the operating-point cluster).

### 1.2 The era technology for the Retro line

The Retro category asks for "one named instrument or display technology of its era, drawn as that technology really looks". The named technology here is **the LED dot-matrix display: 5 × 7 and 8 × 8 modules and the matrix scoreboards built from them**.

| # | Fact | Label and source |
|---|---|---|
| E1 | A common 8 × 8 module (1088AS) is 32 × 32 × 8 mm with 3 mm red dots (625 to 630 nm), a black face and a white body, wired row-cathode, column-anode, 20 mA per LED. | **Cited**, 1088AS datasheet (hosted on GitHub, d2r2/go-max7219/docs) |
| E2 | Its pitch is 32 ÷ 8 = 4 mm, so dot ÷ pitch = 0.75 and the lit fraction of the face is π(0.75)² ÷ 4 = 0.44. | **Computed** |
| E3 | Such modules are multiplexed: a driver (MAX7219) scans 8 digits at 500 to 1,300 Hz, typically 800 Hz, and sets brightness in 16 PWM steps (maximum 31/32 or 15/16 duty). | **Cited**, MAX7219/MAX7221 datasheet, electrical characteristics and Table 7 |
| E4 | 5 × 7 character modules are sold in 17.2 mm and 30.5 mm heights (Kingbright TA07/TC07 and TA12/TC12 families). | **Cited** from the RS listing titles; dot diameter and pitch unconfirmed (the datasheets were not read) |
| E5b | The 1987 Corvette C4 cluster is LCD under a polarising film, with "bar-graph speedometers", a Eurostile typeface and words rather than icons; it is hard to read in sun and shows "typical, old LCD ghosting". | **Cited**, The Drive, "C4 Corvette Start-Up Sequence Proves Digital Dashes Used to Be Sick" |
| E5 | Period production cars rarely used LED dot matrices. The Lagonda Series 2 (prototype October 1976) used "graphic and digital solid-state displays", then gas plasma, CRT and vacuum fluorescent; the Corvette C4 had "three LCDs as standard"; the 300ZX (Z31) a "colourful bar chart"; the Citroën BX 19 Digit a "digital and graphic display of the speed". | **Cited**, Hagerty UK, "12 favourite retro digital dashboards, volume one" |

**Our reading:** the 1980s car dashes the lead suggested (Lagonda, C4) are LED-segment, plasma, LCD and VFD, not dot matrices, so they are period mood, not the technology to draw. The true dot matrices of that decade were scoreboards, destination signs and character modules. Two consequences: the dash borrows the **scoreboard** grammar (big character cells, a dot bar, a dot graph) rather than any car's layout, and it stays clearly apart from `RetroCyber`, which recreates the Supra A70's vacuum-fluorescent meter (segments, cyan phosphor, a filter glass). Better real target: none found that beats the Glyph Matrix for dot optics; if step 1 wants a period anchor for the review sheet, an 8 × 8 or 5 × 7 LED module photographed lit and unlit is the one to add.

### 1.3 The period hardware it is faithful to

**Named hardware:** the Hewlett-Packard HDSP-2000 family of 5 × 7 LED character displays (four characters per package, red glass on a ceramic base) and the epoxy 5 × 7 and 8 × 8 dot-matrix modules of the same era, seen through the contrast filters HP specified for them. A scoreboard or message board of the 1980s is the same parts at large scale, so the dash reads as one panel of such a board.

| # | Property | What the real hardware does | Label and source |
|---|---|---|---|
| P1 | Package and glass | HDSP-2010 and -2030 are "among the few displays ever made to use a red glass and ceramic package"; "most similar displays of this era are manufactured with much cheaper red epoxy". The family came in red, yellow and green. | **Cited**, Industrial Alchemy, HP HDSP-2xxx series |
| P2 | Face | A dot-matrix package "uses a dark ceramic substrate"; indoor displays are "designed for low reflectance" so unlit elements match the package. The 1088AS has a black face and white body. | **Cited**, HP/Agilent AN 1015; 1088AS datasheet |
| P3 | Cell | 5 × 7 dots, usually separated by one blank row and column so the real cell is 6 × 8. | **Cited**, Wikipedia, Dot-matrix display |
| P4 | Pitch | 1088AS: 4 mm pitch, 3 mm dots, dot ÷ pitch 0.75 (E1, E2). HDSP-2000 dot size and pitch unconfirmed (datasheet PDF could not be read; step 1 reads the HP 1986 Optoelectronics catalogue on bitsavers). | **Cited** and **Computed**; unconfirmed for HP |
| P5 | Colour | LEDs "were primarily red in color for the first decade"; in the late 1980s AlInGaP gave efficient red and amber; the "green" of the time "was hardly green at all, mostly yellow"; efficient blue arrived only in the late 1980s. AN 1015 names standard red at 655 nm, high-efficiency red 635 nm, yellow 583 nm and green 565 to 569 nm. | **Cited**, Wikipedia, LED display (History); AN 1015 |
| P6 | Filter over the face | Dim rooms: a red long-pass filter (cut-off 600 to 630 nm, 60 to 75% transmission for 655 nm red); moderate: 35 to 50%; above 200 footcandles a neutral grey at 18 to 25%. Grey attenuates ambient light twice and emitted light once. Circular polariser (10 to 12%) to kill specular reflections; a 45° louvred filter "similarly to a venetian blind" narrows the view to 60°. Untextured filters are tilted forward so reflections miss the viewer. Wavelength filters "are not recommended for sunlight ambients". | **Cited**, AN 1015 |
| P7 | Persistence | No phosphor and no liquid crystal: the dot follows its drive current, so there is no afterglow (unlike VFD phosphor) and no LCD smear (the C4's "typical, old LCD ghosting", E5b). Grey comes only from PWM (16 steps on the MAX7219). | **Our reading** from E3 and E5b; a rise-time figure is unconfirmed |
| P8 | Ghosting | Multiplexed LED matrices show a faint "ghost" of lit dots in the next scanned row or column when the driver does not blank between rows; TI and a US patent treat it as a known fault ("downside" and "upside" ghosting). | **Cited** as titles only (TI SLVAFM7 contents; US 8,659,514 title); the look is **Our reading** |
| P9 | Night | The display is emissive; at night only brightness changes, set by PWM duty (MAX7219: 16 steps) or by auto-brightness (the Glyph Matrix offers auto or manual brightness, and always-on toys run "with reduced brightness"). Under a red filter the unlit dots stay near-black and the lit dots bloom slightly in the filter and lens. | **Cited** E3, B10; the bloom is **Our reading** |

**Our reading: what "realistic" means here.** (1) The face is near-black ceramic or epoxy behind a tinted filter, not pure OLED black: unlit dots are visible as darker, slightly domed wells only where the cell or track exists. (2) Lit dots are one saturated LED colour with a hot core and a soft halo inside the lens, never a flat disc. (3) The filter adds one faint specular sheet, tilted, as AN 1015 prescribes, moving with the virtual light. (4) No afterglow and no smear; optional faint ghost dots at about 3% in the row below lit dots, off by default (it is a fault). (5) At night only the lit level drops; the filter and wells stay.

### 1.4 What the user can change, and the period default

| Part | Maps to | Period-correct default |
|---|---|---|
| Lit dot colour (gear, speed, bar) | palette `primary` (`primary_text` for small cells) | high-efficiency red, about 635 nm (AN 1015); approximate sRGB `#FF3A1A` (**Our reading**; step 1 converts 635 nm with the CIE 1931 tables) |
| Shift super-dots and strobe | palette `shift` | yellow 583 nm, approximate `#FFC21A`, the period's second LED colour |
| Dyno power dots | palette `accent` at ≤ 30% luminance | same red, dimmer |
| Dyno torque ring dots | palette `secondary` | amber (late-1980s AlInGaP), approximate `#FF8A1A` |
| Unlit wells | role `ink_ghost` | the brief's inkGhost |
| Filter tint | `config.extra` `GlyphDotMatrixFilter`: `red`, `grey`, `amber`, `none` | `red` long-pass (P6) |
| Filter louvre and polariser sheen | `config.extra` `GlyphDotMatrixLouvre` (bool) | on, faint |
| Ghost dots (multiplex fault) | `config.extra` `GlyphDotMatrixGhost` (bool) | off |
| Dot shape ratio | `config.extra` `GlyphDotMatrixDot`: `module` (0.75) or `ndot` (touching) | `module` (0.75, 1088AS) |
| Micro-labels | font role (the user's swappable font) | Titillium Web 600 (the brief); the dot glyphs stay bitmaps |
| Brightness and night level | the host's global brightness and `reduced_flash` | full; reduced flash swaps the strobe for the 2 Hz breath |

Random and custom palettes stay allowed (house rule); a non-LED colour such as blue is a user choice, not the default (blue LEDs were not efficient until the late 1980s, P5).

## 2. Reference photos

| # | What it shows | URL | Saved to Drive |
|---|---|---|---|
| 1 | Glyph Matrix LED allocation, Phone (3) 25 × 25 and Phone (4a) Pro 13 × 13 | https://github.com/Nothing-Developer-Programme/GlyphMatrix-Developer-Kit/blob/main/image/23111_25111_LED_allocation.svg | yes, `ref_01_glyph_matrix_led_allocation.svg` |
| 2 | Nothing wordmark on its 5-row dot grid, with the Ndot alphabet | https://github.com/xeji01/nothingfont/blob/main/assets/Ndot-55a.png | yes, `ref_02_ndot_wordmark_grid.png` |
| 3 | Ndot 57 full character set (5 × 7, touching dots) | https://github.com/xeji01/nothingfont/blob/main/assets/Ndot-57glyph.png | yes, `ref_03_ndot57_character_set.png` |
| 4 | Ndot 55 text sample | https://github.com/xeji01/nothingfont/blob/main/assets/comparisonb.png | yes, `ref_04_ndot55_text_sample.png` |
| 5 | 1088AS 8 × 8 LED matrix: schematic, data and product photo (black face, round dots) | https://raw.githubusercontent.com/d2r2/go-max7219/master/docs/LED8x8_1088AS.pdf | yes, page 1 as `ref_05_1088as_8x8_led_matrix_datasheet.png` |
| 6 | Nothing Phone (3) back with the Glyph Matrix lit (official) | https://intl.nothing.tech/products/phone-3 | no (fetch budget) |
| 7 | Glyph Matrix close-ups and toys in use | https://www.engadget.com/mobile/smartphones/nothing-phone-3-hands-on-dot-matrix-glyph-flagship-phone-173019742.html | no |
| 8 | Glyph Matrix guide with toy screenshots | https://gadgets.beebom.com/guides/nothing-phone-3-glyph-matrix-guide | no |
| 9 | Kingbright TA07 5 × 7 module, lit | https://uk.rs-online.com/web/p/led-displays/1451359 | no |
| 10 | Corvette C4 cluster start-up (period mood) | https://www.thedrive.com/news/c4-corvette-start-up-sequence-proves-digital-dashes-used-to-be-sick | no |
| 11 | Lagonda Series 2 dash (period mood) | https://www.hagerty.co.uk/articles/lists/12-favourite-retro-digital-dashboards-volume-one/ | no |
| 12 | Oblivion screen graphics (GMUNK), the brief's layout reference | https://gmunk.com/OBLIVION-GFX | no |
| 13 | Stadium scoreboard history (matrix boards) | https://www.athleticbusiness.com/facilities/stadium-arena/article/15152114/how-scoreboards-and-video-displays-have-evolved | no |
| 14 | HP HDSP-2010 5 × 7 LED display on a breadboard (red glass package) | https://raw.githubusercontent.com/anachrocomputer/testHDSP2000/master/HDSP2010breadboard.jpg (page https://github.com/anachrocomputer/testHDSP2000) | no, URL for the PC |
| 15 | HP HDSP-2010 close-up, lit dots behind red glass | https://raw.githubusercontent.com/anachrocomputer/testHDSP2000/master/HDSP2010display.jpg | no, URL for the PC |
| 16 | HP HDSP-2xxx packages and die photos | https://www.industrialalchemy.org/articleview.php?item=750 (thumbnails on the page) | no |
| 17 | HP/Agilent AN 1015, contrast filter figures and tables | https://www.hpmemoryproject.org/an/pdf/an_1015.pdf | no |

Drive `references` folder: https://drive.google.com/drive/folders/1KDjkcKpIUOs6W8Q3WuyzNPlhBq_1gl8L (`G:\My Drive\FH6 WheelDash v2 - 4K Masters\glyph_dot_matrix\references\`), with `SOURCES.md`.

## 3. How the real thing behaves, with motion notes

The benchmark is not a car display, so these are the behaviours of LED matrices and the Glyph Matrix, then what the dash does in each state.

- **Switching.** An LED has no thermal lag: a dot is on or off within one scan, and grey comes from PWM duty (16 steps on the MAX7219, 256 levels per object on the Glyph Matrix) (**Cited** E3, B4). **Our reading:** dots never fade like filaments; a transition is a step in brightness, optionally across a few frames, never a blur or a slide of the dot itself.
- **Scanning.** Multiplexing at about 800 Hz is invisible to the eye (**Cited** E3). Our reading: no scan-line or flicker effect is drawn; it would read as a fault and cost flash budget.
- **Motion on a matrix is quantised.** Content moves by whole dots (marquee, raster wipes), and rotation and scale on the Glyph Matrix resample onto the grid (**Cited** B4 for the parameters; the resampling is **Our reading**). The dash moves light between dots, never the dots.
- **Contrast with LCD.** The C4's LCD ghosts and lags (**Cited** E5b); an LED matrix does neither. Our reading: the dash must feel instant; the only softness is the 1 to 2 dot leading edge.
- **Idle on the phone.** Always-on toys run at reduced brightness and are woken once a minute (**Cited** B6, B10). Our reading: the dash's idle dims nothing by itself (the house rule says idle is still), but the ghost dots sit at the phone's "reduced" level, well under the lit level.
- **Start-up / greeting.** Unconfirmed for the phone. Proposal: the 11 super-dots and the 64 × 3 bar light outside-in in one 400 ms raster, the gear cell draws `N` row by row, then idle. Shut-down is the host hiding the dash.
- **Idle.** Static apart from telemetry: the operating point sits at idle on the dyno, ghost 8s at inkGhost, no breathing (house rule: idle is still).
- **Cruise.** Bar fill tracks rpm by whole dots with a 1 to 2 dot soft leading edge (a brightness ramp, not a sub-dot position); the dyno shows what has been learned.
- **Pull to the limiter.** Super-dots fill outside-in by shift fraction; the operating-point cluster climbs the learned curve; past the learned peak-power column the bar dots beyond it drop to 60%.
- **Limiter.** Only the super-dot row strobes, 10 Hz at 50% duty (3 on, 3 off frames at 60 Hz; 6 and 6 at 120 Hz, **Computed**); 2 Hz breath in gentle mode.
- **Gear change.** The brief's 50 ms re-raster is 3 frames at 60 Hz and 6 at 120 Hz (**Computed**): the new glyph is complete from frame 1, rows step 0.6 to 1.0 from top to bottom.
- **Braking.** No dot behaviour in the benchmark. Proposal in section 5.
- **Drift / lateral G.** The Leveler toy (**Cited** B6) is the real precedent for a matrix reading an accelerometer. The brief moves the dot sheen azimuth and a 0, 2 or 4 px parallax with lateral G.
- **Reverse / neutral.** Glyphs `R` and `N` in the gear cell (original bitmaps).

## 4. The maths

No row in "Real geometry and maths". The dash does use one derived quantity and one learned table, so the unit tests can assert these:

- **Torque.** Since main `60c99c9` the decoder reads the game's `torque` field (Data Out offset 264; the page states no unit, Forza Motorsport sends N·m; `crates/fh6-core/src/telemetry.rs`), decoded, offset confirmed at check C. Fallback and cross-check from the long-verified power: τ = P × 60 ÷ (2π × rpm), P in W, τ in N·m. 300 kW at 6,000 rpm gives 477.5 N·m (**Computed**). A unit test feeds a packet with both and asserts the two agree within 2% when rpm > 1,000 (Our reading: the tolerance is a guess; check C sets it). Guard rpm below 300: no derived sample. The v1 imperial form `hp × 5252 ÷ rpm` gives lb·ft and must not return.
- **Where the curves cross.** On a dyno sheet with hp and lb·ft on one numeric axis the curves cross at 33,000 ÷ 2π = 5,252 rpm; with kW and N·m they cross at 60,000 ÷ 2π = 9,549 rpm (**Computed**). Our reading: plot power and torque each normalised to its own learned peak, so no false crossing appears and units do not change the drawing.
- **Bin width.** 64 bins from idle to max rpm: 800 to 8,000 gives 112.5 rpm; 1,000 to 12,000 gives 171.9; 700 to 4,500 (diesel) gives 59.4; 0 to 16,000 (EV-like) gives 250 (**Computed**).
- **Samples per bin.** At 60 Hz, rpm rising at 3,000, 6,000, 10,000 and 15,000 rpm/s moves 50, 100, 167 and 250 rpm a frame (**Computed**), so fast first-gear pulls skip bins on narrow tables. Fill every bin between the previous and current sample (linear in rpm) so the curve has no holes. Test: a sweep at 15,000 rpm/s over 800 to 8,000 leaves no empty bin.
- **Learned shift hint (optional, derived).** With per-gear speed ÷ rpm learned live (allowed derived value), the best upshift rpm r* in gear g is where P(r*) = P(r* × k_{g+1} ÷ k_g), k = speed ÷ rpm. More honest than "peak power" for cars with flat curves; label it learned, not MODEL.
- **Dot geometry checks (Computed from the brief).** Gear cell 5 × 7 on 30 px: 150 × 210 px. Speed: 3 cells of 5 × 7 on 16 px with one blank column between cells, 17 columns, 272 px wide, cap 7 × 16 = 112. Bar 64 × 18 = 1,152 px, inside the 1,440 px envelope. Dot ÷ pitch: brief gear ≥ 24 ÷ 30 = 0.80 (lit fraction 0.50); 1088AS 0.75 (0.44); Ndot about 1.0 along a row (0.79).

## 5. Telemetry mapping

| Visual part | Input | Status | Transform |
|---|---|---|---|
| 11 super-dots (shift strip) | `rpm`, `shift_rpm` | verified | outside-in count by `shift_fraction`; strobe at limiter |
| 64 × 3 dot bar | `rpm`, `idle_rpm`, `max_rpm` | verified | (rpm − idle) ÷ (max − idle) × 64, soft 1 to 2 dot edge |
| Gear glyph 5 × 7 | `gear` | verified | original const bitmaps for R, N, 1 to 10, then "10+" handling (section 6) |
| Speed 3 cells | `speed`, `unit` | verified | rounded, hysteresis 0.5 unit; ghost 8s |
| Dyno power dots | `power`, `throttle`, `rpm` | verified | at throttle ≥ 0.95, bin max of P (ignore P ≤ 0) |
| Dyno torque ring dots | `torque` | decoded, offset confirmed at check C | bin max at throttle ≥ 0.95; fallback τ = P × 60 ÷ (2π rpm) from verified power |
| Operating point 2 × 2 | `rpm`, `power` | verified | current bin and normalised P |
| Peak crown, 60% dim beyond | learned table | derived | argmax of power bins |
| Boost shy column | `boost` | verified | psi or bar by unit setting; hidden when 0 for NA cars |
| Sheen azimuth, parallax | `g_lat` | verified | azimuth = atan2(g_lat, 1); 0, 2, 4 px steps |
| Dyno reset | `car_ordinal` | verified | clear both [f32; 64] on change |
| Class badge (shy) | `car_class`, `car_performance_index` | decoded, offset confirmed at check C | PI number as 3 dot digits; the class index's mapping to letters is unverified, so show the letter only after check C |
| Cylinder count under the dyno (shy) | `num_cylinders` | decoded, offset confirmed at check C | a row of n dots (0 for EVs), the dyno sheet's "engine" line; fallback: hide |
| Drivetrain word (shy) | `drivetrain` | verified field | FWD, RWD, AWD as a 3-letter dot word |
| Brake under-row | `brake` | verified | 64 × 1 row under the bar fills from the right |

The brief's "car ordinal / class / PI → dyno reset and a shy class badge" holds since main `60c99c9` (class and PI are decoded); the letter mapping of the class index is the one open point.

**What the game cannot send** (only what is absent from the 88 Data Out fields: the phone functions of the benchmark):

- **Phone notifications, clock, battery, timer toys.** Replacement A (**recommended**): a "pull counter" scoreboard: a small dot tally of full-throttle pulls that have added data to the dyno this session, so the viewer sees the curve being learned. B: a session timer built from timestamp, shown only when stopped.
- **Weather and volume widgets.** A (**recommended**): a "volume" bar reborn as a throttle meter, a 9-dot column that fills with `throttle` like the phone's volume animation. B: a fuel column from the decoded `fuel` field (decoded, offset confirmed at check C), hidden when the game sends 0.
- **Solar clock and digital clock toys.** A (**recommended**): a scoreboard clock that is real race data: `current_lap` or `current_race_time` in small dot digits while a race runs (decoded, offset confirmed at check C), and nothing in free roam. B: gear-by-gear "innings", a row of 10 small dot columns, each the best 0-to-limit time of that gear learned from speed and timestamp.
- **Coolant and oil lamps of a period dash.** Truly absent. A (**recommended**): leave them out and give the space to the dyno. B: a lateral-G "Leveller" (the Glyph toy precedent, B10): one bright dot sliding in a 9-dot track in whole-dot steps.
- **Glyph mirror, spin the bottle, 8 Ball and button toys.** No equivalent; nothing drawn.

## 6. Edge cases

- **Max rpm 0 or tiny, idle near or above max.** Bar and bins disabled when max − idle < 500 rpm; the dash shows gear and speed only; no division by zero (decoder note: `max_rpm` can be 0, idle can exceed it).
- **12,000+ rpm engines.** 171.9 rpm bins; fill between samples. **Diesels (4,500).** 59.4 rpm bins, fine.
- **EVs.** No idle, one gear, rpm may reach 16,000+: use idle 0, still 64 bins; gear cell shows `1` or `D` per the house gear rule (step 1 checks `Gear`); `num_cylinders` is presumably 0 (unconfirmed, check C), so the cylinder row hides. Power and torque are real, so the dyno works and is most interesting here.
- **Decoded fields not yet confirmed (torque, class, PI, cylinders, fuel, lap data).** Each has a fallback or hides: torque falls back to the power derivation, the badge and cylinder row hide when the value is 0 or out of range (PI outside 100 to 999, cylinders outside 0 to 16).
- **Reverse, neutral.** `R`, `N` glyphs; dyno learning paused (power in reverse is not a pull).
- **Gear above 10.** Two-digit gears do not fit one 5 × 7 cell at 30 px: use two cells at 22 px pitch for 11+ (check against cap rule) or clamp the learned table index at 10 as the brief's `[f32; 10]` already does.
- **mph and km/h, hp and kW.** Speed by `unit`, power readouts by `power_unit` (Signals keeps them separate). The dyno is unitless after normalisation.
- **Limiter bounce of about 50 rpm a frame.** Hysteresis of one dot on the bar and 100 rpm on the super-dot threshold; the strobe latches for 150 ms.
- **Spikes and crashes.** Clamp G before the sheen; cap power samples at 1.5 × the running 95th percentile so a crash spike cannot crown a false peak.
- **Airborne.** Wheels free, rpm flares at full throttle with low power: the max-per-bin rule ignores low values, so the curve is safe. Add a check that P > 0.25 × current bin value before replacing, to keep a flare from shifting the operating point oddly (Our reading).
- **Stale or paused stream.** The host hides the dash. On return the dyno keeps its data for the same car ordinal.
- **Palette extremes and random palettes.** Dot core 40% toward white keeps low-luminance palettes visible; the 1 px rim at 35% gives shape on near-white palettes. Test with black-on-black and white primaries.
- **Font swap.** The gear and speed are dot bitmaps, not the font; only the Titillium Web micro-labels follow the user's font.
- **Reduced-flash mode.** Super-dots breathe at 2 Hz; gear re-raster drops to a single-frame swap.
- **Colour only.** Shift stages are also a count of lit super-dots; the peak-power dimming is also a 3-dot crown position.
- **OLED subpixels.** On a 65-inch 4K panel one pixel is 0.375 mm, so a 30 px pitch is 11.2 mm and a 16 px pitch 6.0 mm; at 0.75 m a 16 px pitch subtends 27.5 arcminutes (**Computed**). Dots that small must be drawn with fwidth antialiasing on even pitches, as the brief says, to avoid WRGB moiré.

## 7. Signature moments

**Limiter / shift strobe** (only the super-dot row strobes).
1. **Recommended: scoreboard flash.** All 11 super-dots at 10 Hz, 50% duty, in the shift colour, with each dot's 1 px rim staying lit at 35% during off frames, so the row never vanishes (a real LED cannot do this, but it reads as the lens catching light).
2. **Converging pair.** Outside-in fill completes, then the strobe runs in phase on all 11; in gentle mode the breath peaks from the centre dot outwards over 500 ms.

**Gear change.**
1. **Recommended: the brief's re-raster** (new glyph present from frame 1, rows 0.6 to 1.0 top to bottom in 50 ms), plus the dyno operating point snapping to its new bin with no tween, which is how a matrix moves.
2. **Scoreboard roll.** The old glyph scrolls up one dot row per frame while the new rises from below, 7 frames at 120 Hz; reject if it costs the "present from frame 1" rule.
3. **Marquee tick.** A single bright column sweeps across the gear cell left to right in 3 frames, the Glyph-style "loading" gesture; quieter than 2.

## 8. What it needs from the 4K library

The pre-build (PR #18, branch `claude/project-thread-ib67x9`) made **no kit** for this dash: PREBUILD.md lists GlyphDotMatrix under "Dashes with nothing to pre-build" as "procedural only by their briefs", and `kits.rs` has no `glyph_dot_matrix` entry. No `assets/clusters/glyph_dot_matrix/prebuild/manifest.json` exists.

That collides with `CLUSTER_PROMPT.md`, "Asset rules": every cluster ships at least one bespoke ComfyUI and one bespoke Blender asset, and "procedural-only clusters are not accepted". The dash therefore needs physical parts that are independent of the layout:

- Library check against the Retro list (1.3): the library has none of the period dot-matrix parts (red glass or epoxy face, dark ceramic wells, contrast filters, louvres, lit-dot profile); the only near part is the LED cap below.
- Reusable now: `shift_led_32_off` / `shift_led_32_on` (SimPitWall and SimInputTrace kits), 32 px LED caps; close to the 30 px super-dots. Either the brief moves super-dots to 32 px or the sprite is re-run at 30 px. `antiglare_matte_grain` (SimPitWall) for a screen-face grain at 4% luminance or less.

LIBRARY GAP: LED dot lens sprite, domed diffused lens in a black well, off and on keys, at 30 px (super-dots and gear) and 16 / 18 px (speed and bar), 2 px per 4K px master; the physical "LED" each SDF dot sits in; Blender.
LIBRARY GAP: black dot-matrix module face swatch, 4096² seamless, matte black epoxy with recessed round wells at 30 px pitch (variants at 16 and 18 px), normal and roughness passes; the unlit-cell surface inside glyph cells and bar tracks (never a full field); Blender.
LIBRARY GAP: contrast filter sheet, 4096² seamless, in four tints (red long-pass, neutral grey 20%, amber, clear), untextured with very faint micro-scratches, plus its specular sheen map for a forward-tilted sheet (AN 1015); the cover over every dot field; computed (exact transmission per tint) with a ComfyUI scratch layer.
LIBRARY GAP: 45° louvred filter swatch, 4096² seamless, fine black louvres seen face-on at very low contrast with an off-axis darkening table; the "venetian blind" film of AN 1015, optional; Blender.
LIBRARY GAP: lit LED dot core and halo profile table, 256 × 1, per filter tint, measured shape of a domed 3 mm LED behind a filter (hot core, soft halo, no bloom past the well); the realistic lit dot; Blender render of one lit lens, reduced to a radial table.

## 9. Departures and open questions for the board

1. Benchmark is the Glyph Matrix for dot optics, but the subject is the 1980s LED matrix. Does the board show the subject strongly enough to pass the Retro line beside `RetroCyber` (VFD) and `SynthwaveOutrun`? Reason: category check.
2. Brief dots (0.80 of pitch) sit between a real LED module (0.75) and Ndot (about 1.0 along a row). Pick one ratio per element on the board. Reason: B8, E2.
3. Class badge: PI digits now, class letter after check C confirms the index mapping. Reason: telemetry truth (main `60c99c9`).
4. Procedural-only brief versus mandatory bespoke assets: the three LIBRARY GAP parts resolve it. Reason: asset rules.
5. Super-dots 30 px versus the existing 32 px LED cap sprite. Reason: reuse.
6. Gear above 10 in one 5 × 7 cell. Reason: edge case.
7. "Oblivion dot-lattice layout (GMUNK)" is a layout reference only; no dot-lattice field outside cells. Reason: v1 failure.
8. Dyno: power and torque each normalised to their own peak (no false crossing). Reason: section 4.
9. Period default colours (red dots, yellow shift, amber torque) versus the family's `shift` default `#978DFC` (violet). Reason: Retro realism (P5) against the family spec; the board decides which the factory palette uses.
10. Ghost dots: a real fault, off by default; keep it as a setting only if it reads as realism, not as a bug. Reason: P8.

## 10. Sources

**Read in full (GitHub):**
- GlyphMatrix Developer Kit README and LED allocation figure: https://github.com/Nothing-Developer-Programme/GlyphMatrix-Developer-Kit
- Ndot specimens (fan repository, not a maker source): https://github.com/xeji01/nothingfont
- 1088AS 8 × 8 LED matrix datasheet: https://raw.githubusercontent.com/d2r2/go-max7219/master/docs/LED8x8_1088AS.pdf
- MAX7219/MAX7221 datasheet: https://raw.githubusercontent.com/d2r2/go-max7219/master/docs/MAX7219-MAX7221.pdf

**Read (web):**
- Fonts In Use, Ndot: https://www.fontsinuse.com/typefaces/233367/ndot
- Engadget, Nothing Phone 3 hands-on (built-in browser): https://www.engadget.com/mobile/smartphones/nothing-phone-3-hands-on-dot-matrix-glyph-flagship-phone-173019742.html
- Beebom, Glyph Matrix guide (built-in browser): https://gadgets.beebom.com/guides/nothing-phone-3-glyph-matrix-guide
- The Drive, C4 start-up (built-in browser): https://www.thedrive.com/news/c4-corvette-start-up-sequence-proves-digital-dashes-used-to-be-sick
- Hagerty UK, retro digital dashboards: https://www.hagerty.co.uk/articles/lists/12-favourite-retro-digital-dashboards-volume-one/

- Industrial Alchemy, HP HDSP-2xxx (built-in browser): https://www.industrialalchemy.org/articleview.php?item=750
- HP/Agilent AN 1015, contrast enhancement for LED displays: https://www.hpmemoryproject.org/an/pdf/an_1015.pdf
- Wikipedia, Dot-matrix display (built-in browser): https://en.wikipedia.org/wiki/Dot-matrix_display
- Wikipedia, LED display (built-in browser): https://en.wikipedia.org/wiki/LED_display
- testHDSP2000 README (GitHub): https://github.com/anachrocomputer/testHDSP2000

**Found, not read (step 1 reads them):**
- HP 1986 Optoelectronics Designers Catalog (HDSP-2000 dot size and pitch): https://ftp.mirrorservice.org/sites/www.bitsavers.org/components/hp/1986_Optoelectronics_Designers_Catalog.pdf.pdf
- TI SLVAFM7, LED matrix ghosting (contents only loaded): https://www.ti.com/document-viewer/lit/html/SLVAFM7/GUID-B869C3DD-46F3-4A47-871B-9634E02BFFB9
- US 8,659,514, LED matrix ghost image prevention: https://uspto.report/patent/grant/8659514
- Nothing Phone (3) product page (loaded, almost no text): https://intl.nothing.tech/products/phone-3
- GSMArena hands-on (blocked by a bot wall): https://www.gsmarena.com/nothing_phone_3_handson-news-68454.php
- Android Authority, Glyph Matrix: https://www.androidauthority.com/nothing-phone-3-glyph-matrix-3572917
- Nothing community, Ndot57: https://nothing.community/de/d/104-ndot57-the-nothing-typeface
- Kingbright TA07 listing: https://uk.rs-online.com/web/p/led-displays/1451359
- Wikipedia, Aston Martin Lagonda: https://en.wikipedia.org/wiki/Aston_Martin_Lagonda
- Athletic Business, scoreboard history: https://www.athleticbusiness.com/facilities/stadium-arena/article/15152114/how-scoreboards-and-video-displays-have-evolved
- GMUNK, Oblivion GFX: https://gmunk.com/OBLIVION-GFX

**Repository:** `program/phases/07-cluster-factory/CATALOG.md` (brief, Retro row, Flash and telemetry); `CLUSTER_PROMPT.md` (The bar, Asset rules); `docs/design/families/modern-digital.md`; `crates/fh6-core/src/telemetry.rs` (as of main `60c99c9`, all 88 fields); `crates/fh6-kit/src/cluster.rs` (`Signals`); `docs/TELEMETRY_USAGE_MATRIX.md`; PREBUILD.md and `kits.rs` on `origin/claude/project-thread-ib67x9`; v1 code at `f1b3a3c`.
