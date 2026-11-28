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

## Work log
- Step 1 research: `RESEARCH.md` (bdf9817).
- Step 2 board: eight state boards drawn as SVG with text outlined from the repo's fonts (`/mnt/project-files/valkyrie_lmp1/board_v1/`), waiting for the shared Penpot tab.
- Asset recipe written before the Penpot board (923eae1) and checked once on the container's CPU Blender with the procedural carbon (16 samples), to prove the pipeline; those renders were not committed. The real build (ComfyUI forged carbon from the pre-build, Cycles at 256 samples) runs on the cloud pod after the board is in Penpot.

## Shared code touched (allowed small changes)
- `tools/asset-forge/src/hero/mat.rs`: `mat::forged(texture, tile_px)`, and in `hero/bpy.rs` the carbon builder takes a `texture` for forged (chopped-fibre) carbon under a clear coat; without the file it keeps the procedural twill. Any later dash with forged carbon can use it (RevueltoCorsa's brief names forged composite).

## Requests for shared code (not done here)
- Lap, distance-travelled and timing fields in the `fh6-core` decoder, verified against real packets: they would let the brief's predictive lap-delta crown and sector strip come back.
