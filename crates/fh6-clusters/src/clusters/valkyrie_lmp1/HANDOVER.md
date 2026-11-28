# ValkyrieLmp1: handover (draft, in progress)

Build #9, catalog row 13 · Hypercar · target `recreation` · Race Dash family · cockpit and hood. Thread "Build ValkyrieLmp1 dash", branch `claude/project-thread-rck9qi`, draft PR #19.

## Frozen expectations (fh6-visual-review step 1)

Written on 2026-11-28 before the first build was rendered. Later answers never edit this section; a changed tolerance is logged under it with its reason.

### Rank order
1. Gear (inside the rev counter, `race` 800, cap 176) and the ten shift lights above the window.
2. Tabular speed left of the dial (cap 112, ghost 8s at 2 to 3%).
3. The round rev counter: the lit arc in `primary`, numbered 0 to 12 in thousands.
4. Power readout and the 4 s engineer trace on the right; the two shoulder encoders (pedals left, power share right).
5. The forged-carbon wheel face, the window rim, the partition, labels, the TC and ABS lamps when dark.

### Feature inventory of the real display, with tolerances (from RESEARCH.md)
| # | Feature | Reference | Tolerance |
|---|---|---|---|
| F1 | A carbon wheel face around a black screen | evo, the replica | the face reads as carbon at full frame without zoom; the window is true black inside |
| F2 | A round rev counter that dominates the screen | PistonHeads | the dial is the largest element in the window; its diameter is at least 40% of the window width |
| F3 | Numbered to 12 (thousands) | PistonHeads | numerals 0 to 12 on an 11,100 rpm car, each legible on a 1:1 crop |
| F4 | Blue rev counter | the replica | the lit arc's hue in the blue band (probe hue 190° to 215° with the factory palette) |
| F5 | Ten shift lights, 1 + 8 + 1, above the screen | the replica | exactly 10 lenses counted on the strip crop; eight in one partition, one on each shoulder |
| F6 | Blue shift lights | evo | lit pods below the shift point in `primary` (blue); the `shift` role at and above the shift point |
| F7 | First change-up light well short of the limiter | PistonHeads | the first pods light by about 8,600 rpm on an 11,100 rpm car |
| F8 | Gear at the screen's centre | the replica | the gear's centre within 10 px of the dial's centre line x = 0 |
| F9 | A digital speed | Car | tabular speed readable on every state |
| F10 | ABS and TC indicators | the replica | two lamps, dark unless their derived condition holds |
| F11 | Rotary dials beside the screen | Wikipedia, the replica | two knurled rings on the shoulders, read as machined metal at full frame |

### Key frames
- The limiter is reached at frame 29 or 30; the strobe runs over frames 30 to 41 (only the LED strip flashes, 10 Hz, equal on and off phases; the dial's red-zone rim holds a steady `shift`).
- The gear changes 3 to 4 at frame 42; the new gear shows on the first frame after the data changes.
- Idle frames 0 to 9 settle with no sweep the data does not drive.

### TJX's rejection patterns, to check against named crops
- (a) Looks nothing like the real car's display.
- (b) Faint textures on vector shapes; materials that show only at 2x zoom.
- (c) One slab with blobs stuck on, or a backing panel the real display does not have. Here the wheel face is the real display's own hardware (the Valkyrie's screen sits in its carbon wheel); it must read as that, not as a generic panel.
- (d) Flat modern widgets instead of the 2027 bar.
- (e) Glass dashes only: not applicable (no glass layer; the cover glass is part of the Blender render).

### Tolerance changes logged after freezing
- 2026-11-28, board v2 (TJX's rule in CLUSTER_PROMPT.md "The bar": replace what the game does not send): the rev tunnel and the friction circle were added, the window widened from ±440 to ±470, the dial went from r 204 to r 190 (diameter 380, still at least 40% of the 940 px window, F2), and speed went from cap 112 to cap 104 so that three tabular cells fit between the window edge and the dial.
- 2026-11-28, direction check: the gear went from cap 176 to cap 168 (centre y 12), so it clears the RPM x1000 label by 16 px. It is still the largest glyph.

## Direction check (2026-11-28, first build of board v2, procedural wheel face)
Renders: `mockups/ValkyrieLmp1/` (preview-export on lavapipe). Fresh-eyes reviewer packet: `mockups/blind/913/` (A.png and 8 glance frames, no logo zone, no reference photos).

Cold description (step 2), written before the board or notes were opened: the frame shows a dark angled wheel face with a row of 8 rectangular lenses in a partition and one more on each shoulder. In the middle is a black screen holding a blue 240° arc numbered 0 to 9, with the white gear "4" at its centre and faint concentric rings behind it. "118 MPH" sits to the left, and "POWER 104 HP" with a small three-line trace to the right. Under the speed is a small crosshair circle with a dot; under the gear are the TC and ABS pills. Two round encoders sit outside the screen, with a green arc on the left and a blue arc on the right. The eye goes gear, then speed, then arc, then power. The face reads as flat grey, not as a material.

Rejection patterns: (a) a real Valkyrie owner would see the 1+8+1 blue lights and the round counter, but no reference photo could be fetched, so this is NOT ASSESSED; (b) no faint textures (the face is flat until the renders arrive); (c) the wheel face is the real display's own hardware, but in procedural form it reads as a flat slab; this is a no until the Blender face lands; (d) the arc, tunnel and pods read as instrument hardware, not flat widgets; (e) not applicable.

Fresh-eyes reviewer (once, seed 913). Its findings and what was done:
- It noticed first the gear, then the speed, then the arc.
- It named the source as "don't know".
- Blind read: every gear right; every speed within 1. Zones were right on 5 of 8 frames; it missed high on frames 25, 53 and 23 (92%, 80% and 83% of the limiter), reading the boundary as mid. **Fixed:** a `caution` band now runs outside the arc from rev stage 2 to the shift point, so the approach to the limiter reads on the dial.
- The gear's glow washed over the RPM label (frame 11). **Fixed:** the gear bloom is cut to 50% and the gear is cap 168 with a 16 px gap (`review/window_1to1.png`).
- Ghost 8s read as a smudge, and the unit sits under the 3-cell field. **Not fixed:** this is the house TabularSpeed (ghost 3% on screen, fixed cells).
- The shift lights are blue only. **Not fixed:** deliberate; the real car's lights are blue (evo), and `shift` takes over at the shift point.
- Small labels. **Not fixed:** they are rank 5 at cap 16 and carry no dynamic value.
- The encoders have no ticks and muddy maroon rings, and the housing is flat. **Not fixed yet:** this is the procedural face; the Blender knurled rings and forged carbon replace it once the reworked texture lands.
- The TC/ABS pills are unclear when dark, and the trace is sparse at a steady state. **Not fixed:** they are dark by design, and the trace draws only the data.

Measured (step 4):
- `glance` gear 11.3 to 15.2:1 and speed 16.6 to 17.4:1 in all four cockpit states; hood gear 9.4 to 13.0:1 and speed 13.5 to 14.7:1: yes.
- `flash --frames 30-41`: only the 10 lenses flash, at 10 Hz with 3 on / 3 off. The worst field is 8274 of 21526 px². The tallest block is 34 px: yes. One 17×11 px area at (169, 77) local is the lit arc's head following the rpm bouncing off the limiter (data). The power digits flickered at first; **fixed** by feeding the readout the packet value through 5-unit hysteresis.
- `motion`: idle frames 1 to 9 at 0.00% change; frame 10 is the pull starting.
- `stretch`: 0 px lit outside the envelope plus 40 px.
- `probe`: the arc's lit colour is #04618B (HSV hue 199°, F4 blue band).

DC lines:
- V1 yes (expectations frozen above before the first render).
- V2 yes (cold description and rejection patterns above).
- V3 yes (one fresh-eyes reviewer; findings recorded above).
- V4 yes (gear first).
- V6 yes (glance figures above).
- V10 yes (squint and luminance view: the labels and the face fade first, then the gear).
- V13 **no** ("don't know"). The Valkyrie wheel OLED is obscure, and the reviewer had no reference. Logged as a line that cannot be fixed by the dash; the recognisable parts (1+8+1 blue lights, round counter numbered from the limiter, gear at the centre, encoders beside the screen) are all in.
- V14 yes for layout, geometry and colour: F2 dial 380 px against the 376 px minimum; F3 numerals 0 to ceil(limiter) (0 to 9 on the 8,700 rpm review car, 0 to 12 on 11,100); F4 hue 199°; F5 10 lenses counted by `flash` (8 in the partition, 2 on the shoulders); F8 gear x = dial x = 0; F9 speed in every state; F10 two lamps; F7 first pods at stage 1 (0.75 of the shift rpm: 8,076 rpm on an 11,100 rpm car, under the 8,600 tolerance).
- V15 NOT ASSESSED (no reference photo could be fetched; the proxy refused the replica review's images).
- V16 zones yes (`review/board_vs_render.png`: every board zone in place). "Clearly richer than the board": **no** until the Blender face replaces the procedural one.
- V17 **no**, for the same reason (the materials are flat).
- V19 not applicable (no glass layer).
- V20 yes.

Decision: **go with changes**. The concept holds. Changes: the Blender wheel face (forged carbon, knurled encoders, LED caps) once the reworked texture lands, then V16 and V17 are walked again at the closing review.

## Work log
- Step 1 research: `RESEARCH.md` (bdf9817).
- Step 2 board: eight state boards drawn as SVG with text outlined from the repo's fonts (`/mnt/project-files/valkyrie_lmp1/board_v1/`), waiting for the shared Penpot tab.
- Asset recipe written before the Penpot board (923eae1) and checked once on the container's CPU Blender with the procedural carbon (16 samples), to prove the pipeline; those renders were not committed. The real build (ComfyUI forged carbon from the pre-build, Cycles at 256 samples) runs on the cloud pod after the board is in Penpot.

## Shared code touched (allowed small changes)
- `tools/asset-forge/src/hero/mat.rs`: `mat::forged(texture, tile_px)`, and in `hero/bpy.rs` the carbon builder takes a `texture` for forged (chopped-fibre) carbon under a clear coat; without the file it keeps the procedural twill. Any later dash with forged carbon can use it (RevueltoCorsa's brief names forged composite).

## Requests for shared code (not done here)
- Lap, distance-travelled and timing fields in the `fh6-core` decoder, verified against real packets: they would let the brief's predictive lap-delta crown and sector strip come back.
