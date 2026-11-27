# visual-review: worked commands

Run from the repo root in the Bash tool. The tool is CPU only (no GPU lock), reads PNG and JPEG, composites everything over OLED black, and writes to `mockups/<Name>/review/` unless `--out` says otherwise (`--out` takes a folder, or a file name ending in `.png` for the single-output commands). `cargo run --release -p visual-review -- --help` prints every flag.

```bash
VR="cargo run --release -p visual-review --"
N=LiquidGlass                      # your theme name, as in mockups/<Name>/
```

Coordinates are full-frame 4K px. With `--anchor cockpit|hood|hud` they are local to that view's anchor, which is how a cluster's `Geom` table is written. On a `<state>_crop.png` or a sequence frame, 4K coordinates are mapped onto the crop for you, and the outputs print both. On any other picture (a golden, a reference photo, a board of another size) give `--region` in the picture's own pixels; `--view` and `--anchor` refuse it. The examples use the Liquid Glass pilot's zones; use your own.

Transparency: renders are premultiplied, so the tool keeps their colour as it is. A design-tool export (a Penpot board) has straight alpha: add `--straight`, or its soft edges come out too bright.

## Detail at 1:1

```bash
$VR tiles mockups/$N/02_mid_cruise_cockpit.png                      # the cockpit envelope + 40 px as 1:1 tiles
$VR tiles mockups/$N/hood_02_mid_cruise.png                         # hood_* files get the hood envelope (two tiles: it is wider than 1568 px)
$VR tiles mockups/$N/02_mid_cruise.png --anchor cockpit --region -260,-260,520,420 --zoom 3   # 3x pixels of the gear zone
$VR crop  mockups/$N/02_mid_cruise_cockpit.png --view cockpit --mirror                         # a mirrored copy for the cold look
```

A tile or crop is 1:1 only while it stays within 1568 px a side and 1.15 megapixels; every command prints a note when an output is larger than that, because the Read tool then shows it downscaled.

## The glance gate and the squint view

```bash
for s in 01_idle 02_mid_cruise 03_redline_strobe 04_drift_braking; do
  $VR glance mockups/$N/${s}_cockpit.png --anchor cockpit \
      --zone gear:-120,-150,240,200 --zone speed:-636,-130,340,160 --zone rpm:-560,60,200,60
done
$VR glance mockups/$N/hood_02_mid_cruise.png --anchor hood --zone gear:-110,-112,220,200 --zone speed:-650,-92,340,160
```

Draw each zone box round its glyph with background on every side; the tool says "check the box" when a box is almost all glyph or almost empty. `gear` needs 7:1 and `speed` 4.5:1 (the tokens' blur gate); any other name needs 4.5:1, or add `:3` for a graphic (`--zone needle:...:3`). It writes `<stem>_glance.png` (the 960x540 view: look at it once, as a glance), `<stem>_squint.png` (the hierarchy view) and `<stem>_glance_zones.png` (the boxes and ratios, with a legend). A `<state>_crop.png` or a sequence frame works too: the tool puts it back into a black 4K frame first.

`--over <image or #RRGGBB>` puts a render over a backdrop first (a bright sky or snow for hood rows). It needs a render that keeps its alpha; `preview-export` saves its renders already over black, so today it refuses them.

## Against the before render, the goldens and the board

```bash
$VR diff mockups/_before/$N/02_mid_cruise.png mockups/$N/02_mid_cruise.png --view cockpit
$VR diff tests/golden/images/${N}_02_mid_cruise.png tests/golden/images/${N}_02_mid_cruise.actual.png
git show HEAD:tests/golden/images/${N}_02_mid_cruise.png > mockups/$N/review/golden_head_02_mid_cruise.png
$VR diff mockups/$N/review/golden_head_02_mid_cruise.png tests/golden/images/${N}_02_mid_cruise.png --labels "committed|new"
$VR diff mockups/$N/02_mid_cruise.png mockups/$N/board/02_mid_cruise.png --view cockpit --fit --straight --labels "render|board"
$VR sheet mockups/$N/board/02_mid_cruise.png@cockpit mockups/_before/$N/02_mid_cruise.png@cockpit mockups/$N/02_mid_cruise.png@cockpit \
      --labels "board|before|after" --title "$N: board, before, after" --out mockups/$N/review/board_before_after.png
```

`diff` measures exactly as the golden harness does (OKLab delta-E x100 on a 2x2 box in linear light; within dE 3 on at most 0.5% of pixels), prints the changed area, the largest change and the most-changed cells, and writes a heat map (blue below the threshold, then yellow, orange and magenta from dE 10 and 25), crop pairs of the top cells (`--pairs N`, drawn at 2x when a cell is small), a 50% blend (an onion skin to check that the two line up) and a side by side. Labels name both files, with their folder when the names are the same; `--labels "A|B"` names them yourself.

The goldens are 1920x1080 renders cropped to the envelope plus 24 px (768x329 for the cockpit), not shrunken 4K renders, so compare a golden only with another golden or with its `.actual.png` (the failing test writes it), never with a 4K render. `--fit` scales B to A: give the 4K render as A, so `--view` still applies. Export a Penpot board at 3840x2160 (the frame it is drawn in) so `@cockpit` and `--view` apply to it too.

## Side by side

```bash
$VR sheet mockups/$N/01_idle.png@cockpit mockups/$N/02_mid_cruise.png@cockpit mockups/$N/03_redline_strobe.png@cockpit mockups/$N/04_drift_braking.png@cockpit \
      --labels "idle|cruise|redline|drift" --out mockups/$N/review/states.png
$VR sheet mockups/$N/sequence/frame_{28..35}.png --cols 4 --title "strobe onset" --out mockups/$N/review/strobe.png
$VR sheet mockups/$N/sequence/frame_{39..50}.png --cols 4 --title "gear change" --out mockups/$N/review/upshift.png
$VR sheet mockups/references/$N/front.jpg mockups/$N/02_mid_cruise.png@cockpit --labels "real|render" --out mockups/$N/review/real_vs_render.png
$VR sheet mockups/references/$N/front.jpg mockups/$N/02_mid_cruise.png@cockpit --blur 8 --labels "real|render" --out mockups/$N/review/real_vs_render_coarse.png
```

`file@cockpit`, `@hood`, `@hud` or `@X,Y,W,H` crops one picture (4K coordinates on a full frame or a crop, moved by `--anchor`; the picture's own pixels on anything else); `--view` or `--region` crops every full frame and crop, and says which pictures it left whole. A misspelt suffix is an error, not a file name. Each label shows the scale a picture was drawn at, so a downscaled one is never mistaken for 1:1; pictures are shrunk to fit their column or enlarged in whole steps only, up to 8x. Without `--width` the sheet is made narrow enough to be seen 1:1. `--blur S` blurs every picture by S sheet pixels after scaling, the same for all, for the coarse comparison of masses and light (method step 13).

## Motion and flashing

```bash
$VR flash  mockups/$N/sequence --frames 30-41          # the limiter hold: the sustained strobe
$VR flash  mockups/$N/sequence --frames 30-41 --scale 1.4 --eye-m 0.6   # the largest user scale, closest eye
$VR motion mockups/$N/sequence                         # change per frame, stalls, spikes, the idle
```

`flash` prints the frames it read, the flashing pixels per threshold with their rates, the bright and dark phase lengths in frames (3 on and 3 off is the 10 Hz strobe), the flashing areas (each with the rate most of its pixels flash at and the fastest), the worst 10-degree field against 25% of its area, and the tallest flashing block against the 42 px strip; it writes `flash_map_30-41.png` (yellow general flashes, magenta red ones, the worst field ringed, the largest areas boxed). Read its NOT ASSESSED lines: the 12-frame hold cannot see a cycle slower than 6 Hz (21 frames would cover every rate above 3 Hz), and WCAG's count of more than three flashes inside one second needs a hold of a full second. Run on all 60 frames, digits counting up during the pulls show as flashing: that is data, so use `--frames 30-41` for the strobe. The red rule is WCAG 2.0's `(R - G - B) x 320` (WCAG 2.2's u'v' rule is not implemented).

`motion` labels each frame with its segment (also inside a `--frames` range) and flags a stall (nothing changed during a pull) or a spike (a frame changing far more than its neighbours: check it is an intended stage change). Its idle line is a finding only when a settled idle exists (SKILL.md section 8).

## Colour, black and static parts

```bash
$VR cvd     mockups/$N/03_redline_strobe.png            # protanopia, deuteranopia, tritanopia, luminance only
$VR stretch mockups/$N/02_mid_cruise.png                # the whole frame, values 0-8 of 255 shown as 0-255, and the stray light outside the envelope
$VR stretch mockups/$N/02_mid_cruise.png --view cockpit # the envelope only, small enough to see 1:1
$VR static  mockups/$N/01_idle.png mockups/$N/02_mid_cruise.png mockups/$N/03_redline_strobe.png mockups/$N/04_drift_braking.png mockups/$N/sequence
$VR static  mockups/$N/hood_01_idle.png mockups/$N/hood_02_mid_cruise.png mockups/$N/hood_03_redline_strobe.png mockups/$N/hood_04_drift_braking.png
$VR probe   mockups/$N/02_mid_cruise.png --anchor cockpit --region 400,60,200,60   # a label's brightness, median colour and hue
```

`static` compares each pixel on its own (no smoothing), so an edge next to a moving glyph is not counted as live. Give it the sequence folder with the states: four states alone count the strokes that different readouts share (the "1" of 118 and 152) as static, while the sequence's pull sweeps every readout; from the review sequence it takes frames 10 to 59, because 0 to 9 hold the start-up, where labels still fade in. It lists each static area with its place and p95 brightness and marks the ones over the 0.2 cap: name each of those (a label, a unit, a digit that never changes, such as the last 0 of an rpm readout held by 100) and fix it or say why it stays. `--mask X,Y,W,H` leaves out a zone (the logo, when the rules let it be bright). It answers NOT ASSESSED when no lit pixel is the same in every picture. The sequence is drawn in the first view the dash is designed for, so the other view's run is the four states only: read its area list with the shared-stroke caveat in mind. `stretch` on a full frame counts every pixel that is not off outside the view's envelope plus 40 px: stray light the panel shows.

## Dials and needles

```bash
$VR unwrap   mockups/$N/02_mid_cruise.png --anchor cockpit --center 0,-50 --radius 120,175 --from 120 --to 420
$VR annotate mockups/$N/02_mid_cruise.png --anchor cockpit --region -300,-300,600,520 --protractor 0,-50,170
```

`unwrap` turns a ring into a strip (angle across, outer edge on top) with a degree scale under it: count ticks and LEDs on it, and read the needle's angle as a column (0 degrees at 3 o'clock, clockwise; `--from` and `--to` may pass 360). Positions are continuous: pixel (i, j) has its centre at (i + 0.5, j + 0.5), so a centre that falls on a pixel centre ends in .5. The protractor draws ticks every 5, 15 and 45 degrees round a centre.

## The blind glance read

```bash
$VR blind mockups/$N --seed 11 --anchor cockpit --mask 346,76,240,48    # the Liquid Glass logo zone: mask yours
$VR key                                                                 # the review telemetry of every frame
```

It writes `mockups/blind/11/glance_01.png` to `glance_08.png` (full frames at 1430x804, the logo zone black) and the key to `mockups/$N/review/blind_key_11.txt`; the blind folder never names the dash, and the tool refuses a key inside it. Take the logo zone from your `Geom` table (Liquid Glass: centre (466, 100), 240x48) and check the mask on a `crop --mask` first. Give the reader only the image paths (see method.md, "The cold-reviewer packet"). `key` prints the gear, speed and rpm of all 60 frames and the four states, with the rpm as a share of the 8700 rpm limiter.

## Screenshots for progress posts

```bash
$VR annotate mockups/$N/02_mid_cruise_cockpit.png --view cockpit \
      --box 1800,1685,240,200:"gear 16.8:1 at a glance" --arrow 1500,1600,1700,1700:"rim reads crisp" \
      --note "$N build 3: cruise" --out mockups/$N/review/progress_03.png
```

Boxes, rings and arrows are numbered on the image; their labels go in a legend under it, so nothing covers the pixels (a narrow crop is widened to 480 px so the legend can be read). Send the PNG with SendUserFile and attach it to the thread message.
