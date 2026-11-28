# TeslaAdas assets

These files are built by `cargo run --release -p asset-forge -- build --set tesla_adas` from `tools/asset-forge/src/recipes/tesla_adas.rs`. Don't edit them by hand: change the recipe and rebuild.

The ledger rows are also in `assets/ASSETS.md`. The dash draws a plain fallback for each image, used when `setup.assets` is `None` or the file is missing.

Blender's camera is right-handed and the kit's isn't, so every Blender render is mirrored left-right in post.

| File | Tool | Recipe and seed | Size | Used for | Fallback |
|---|---|---|---|---|---|
| `ego.png` | Blender 5.3 (Cycles 128 spp, OptiX denoise, AgX −0.7 EV) | `tesla_adas::CAR_LIB` + `SHOTS`, seed 152. The lofted fastback in pearl white with a glass roof, a ducktail, a full-width light bar at running level and a white 330 × 165 mm plate. Shot through the chase camera (3.2 m up, 7.2 m behind, 16° down) at yaw −40..40 in steps of 8. Cells are 1024 × 640 at 176 px/m; the car's ground centre sits at (0.5, 0.46) of each cell. | 4096 × 1920 (4 × 3 cells) | The driver's own car in the road scene. The nearest yaw cell is drawn. | A drawn body block with a light bar |
| `ego_glow.png` | Blender 5.3 (96 spp) | The same shots at half size, with a 1.9 × 4.6 m light panel 0.14 m under the floor, the car opaque and the ground pale. Post: luminance becomes alpha with one exposure for all yaws, softened 4×, black point 1.5%, gamma 0.75, feathered. | 2048 × 960 | The shift-stage light under the car, tinted live and added | A soft glow round a ground ellipse |
| `ego_brake.png` | Blender 5.3 (64 spp) | The same shots at half size. Only the tail bar is emissive, with its spill on the bumper and the road. Luminance becomes alpha, gamma 0.8. | 2048 × 960 | The brake light, added in warning red while braking | A red bar on the drawn body |
| `ego_parked.png` | Blender 5.3 (Cycles 256 spp) | The same car from the front left through a 64 mm lens, with a strip light over the centre line and one on each side, running lights on. | 2048 × 1152 | The parked view: the car alone while it stands still | The drawn body block |
| `traffic_sedan.png`, `traffic_sport.png`, `traffic_kei.png` | Blender 5.3 (Cycles 96 spp) | `tesla_adas::CAR_LIB` + `SHOTS`, seed 152, in grey paint. The sedan and the sport body are fastbacks (the sport one with a taller ducktail); the kei is a one-box 3.40 × 1.48 × 1.78 m with upright corner lamps and a yellow plate. Rows are view elevations 3 / 6 / 10 / 16°; columns are headings relative to the view ray, 0..40° in steps of 10 (mirrored live for negative angles). Cells are 336 × 232 at 64 px/m, anchored at (0.5, 0.70). | 1680 × 928 each | The seeded ambient traffic, picked by each car's elevation and heading on screen | A grey block |
| `tunnel_lights.png` | ComfyUI (SDXL base 1.0 + 4x-UltraSharp, dpmpp_2m karras, 32 steps, cfg 6.5, 1536 × 640) | Seed 6006, prompt `NIGHT_PROMPT` in the recipe: the ceiling lamps of a Japanese expressway tunnel. Post: the 6144 × 2560 upscale at 4096 × 1706, its median luminance taken to black and its 99.9th percentile to full, so only the lamps' light is left. | 4096 × 1706 | A build input, not loaded by the dash: every lit car shot hangs it 6 m over the road as a light card (long side along the road, emission × 6). It is what the glass roofs and the paint reflect, and it tints the bodies. | The cars are lit by the studio alone |
| `NotoSansJP-Medium-subset.ttf` | fontTools 4.69 | See "The Japanese label face" below. | 160,696 bytes, 700 glyphs | The Japanese line of each two-language label and the road text | The kit's embedded Noto Sans (the Japanese line is then left out) |
| `NotoSansJP-OFL.txt` | — | The font's licence, copied from the same commit. | — | — | — |

The plates carry three dark green blocks where the number would be. No plate, label or road text holds a readable registration, place name or route number.

Rig for every car shot:
- **Body:** a lofted mesh built by the recipe from station tables, subdivided and smoothed. Clear-coated metallic paint, dark glass, rubber, smoked tail lenses.
- **Lighting:** an overhead softbox, two long strip lights, a cool rim from ahead, and the studio.exr HDRI.
- **Night:** `tunnel_lights.png` as a light card over the road, hidden from the camera.
- **Road:** a dark plane under the car, hidden from the camera. It shades the underside and shows in the paint. The sprites carry no shadow: a baked shadow is black with alpha, and the overlay's 8-bit premultiplied output shows that as a lighter box over the faint road surface.

The road itself (surface, lane lines, reflectors, path, braking marks, road text) is drawn live and is not an asset.

## The Japanese label face

The dash sets its Japanese lines in Noto Sans JP Medium. The full variable font is 9.6 MB, and the kit would read it once per weight if it sat in `assets/fonts/`, so the dash carries its own 160 KB subset and loads it itself.

Source: `NotoSansJP[wght].ttf` (9,589,900 bytes, sha256 `c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f`) from <https://github.com/google/fonts/tree/06bfe0d62a98375cb09c942fa18e68bd32cff057/ofl/notosansjp>, SIL Open Font Licence 1.1.

Made with fontTools 4.69.1 (and skia-pathops for `--remove-overlaps`):

```
python -m fontTools varLib.instancer NotoSansJP-wght.ttf wght=500 --update-name-table --remove-overlaps -o NotoSansJP-Medium.ttf
python -m fontTools.subset NotoSansJP-Medium.ttf --unicodes-file=unicodes.txt --layout-features='kern,palt,vert' --name-IDs='*' --name-languages='*' --notdef-outline --no-hinting --output-file=NotoSansJP-Medium-subset.ttf
```

`unicodes.txt` lists 518 code points: Basic Latin, a few symbols, all hiragana and katakana, CJK punctuation, and 233 kanji for dash and road vocabulary (drive modes, gauges, warnings, road text). The result has sha256 `cfb41a07b0f3b539d4d764271a92aee6e2cd508cc5288def9250d30ccbb56dc5`; the recipe's `tesla_adas/font` job checks the file against its fingerprint and registers it in the ledger, it does not rebuild it.

Licences:
- The Blender renders are MIT (asset-forge).
- The SDXL output is under the CreativeML Open RAIL++-M licence; the asset-forge code that made it is MIT.
- Noto Sans JP is under the SIL Open Font Licence 1.1 (`NotoSansJP-OFL.txt`).
