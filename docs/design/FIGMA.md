# FIGMA: master file and HTML boards

Phase 3 works in two places:
1. **The Figma master file** holds the token foundations (variables, text styles, effect styles) and the frame skeleton for templates and heroes.
2. **HTML boards** (`docs/design/boards/`) are the rendered source of every visual: pure SVG inside HTML, no script, rendered to the PNGs in `docs/design/`. They are the authoritative pixels until the Figma file is completed (see "Blocked" below).

## Master file
- **Name:** FH6 WheelDash v2 — Design System
- **Key:** `0zOEwnJ8dEj1kQ7QwPGEH0`
- **URL:** https://www.figma.com/design/0zOEwnJ8dEj1kQ7QwPGEH0
- **Plan:** Starter team (one plan, View seat). Two hard limits shaped the file: **3 pages maximum** (organise with Sections) and **1 variable mode per collection** (per-family palettes are separate variables, not modes). The Starter plan also caps Figma MCP tool calls; Phase 3 reached that cap (see "Blocked").
- The legacy keys `ZW8mpHivoqjl8p7eLlvdBi` and `FsEcAnDwfFf340i8zwbBup` (open question in PROGRESS.md) are **not** the v2 master. `data/figma_tokens.json` keeps `meta.figmaFileKey` at `null` on purpose and points here.

## Page map
| Page | ID | Contents |
|---|---|---|
| 01 · System (tokens + components) | `0:1` | Component library (16 components from [COMPONENTS.md](COMPONENTS.md)); build was started and interrupted, see "Blocked" |
| 02 · Families & Heroes | `1:4` | Section **Family templates** `8:2` (7 frames) and Section **Heroes** `8:3` (28 frames, 3840×2160, black) |
| 03 · OSD · Configurator · Scrim · Catalog | `1:5` | Reserved: OSD / configurator direction, scrim fit, and one Section per catalog cluster for Phase 7 (Starter cannot have a page per cluster) |

## Foundations (done)
| Collection | ID | Variables |
|---|---|---|
| Palette | `VariableCollectionId:3:2` | 28 colours: `palette/<family>/<primary·secondary·accent·shift>` for the 7 families |
| Roles | `VariableCollectionId:4:2` | 44 colours: `role/*`, `rev/*` (stages alias go/caution/warning), `input/*`, `thermal/*`, `proximity/*`, `assist/*`, `lume/*`, `adas/*`, `glass/*` |
| Dimensions | `VariableCollectionId:5:2` | 80 numbers: type cap heights and font sizes, strokes, radii, spacing, glass alpha, layout anchors/envelopes, zone limits |
| Motion | `VariableCollectionId:5:83` | 45 numbers: springs (T, ζ), durations, limiter strobe, shift lights, sweep, visibility |

Every variable has explicit scopes, WEB code syntax and a description naming its `figma_tokens.json` path. Note: the variables were created from tokens v3.1; the round-1 critique changes in v3.2 (shift `#978DFC`, Classic primary `#FF7A1A`, Race Dash shift `#B27CFF`) still need to be mirrored.

| Style | ID |
|---|---|
| Text `FH6/gearHero` · `gearCompact` · `speedHero` · `speedCompact` | `S:90a20324…`, `S:03e9b591…`, `S:d0c66fc1…`, `S:70e15431…` |
| Text `FH6/metricPrimary` · `metricSecondary` · `dialNumeral` · `unit` | `S:86e6a6b5…`, `S:5c41cc5e…`, `S:a768efba…`, `S:0d5c7ab7…` |
| Text `FH6/microLabel` · `nano` · `race-gear` · `glass-speed` | `S:dc9c5abf…`, `S:be8a6704…`, `S:2d108b63…`, `S:71358fb2…` |
| Effect `FH6/glow/soft` · `medium` · `hot` · `phosphor` | `S:247907ba…`, `S:1ccd52d5…`, `S:609e98fc…`, `S:5cac49bd…` |
| Effect `FH6/glass/plate` (background blur 24 + 1 px inner rim) | `S:42755e17…` |

Text styles bind `fontSize` to the Dimensions variables and use `leadingTrim: CAP_HEIGHT`. Labels use Titillium Web SemiBold (available in Figma).

## Frame IDs on page `1:4`
| Family | Template | 01 idle | 02 mid cruise | 03 redline strobe | 04 drift / braking |
|---|---|---|---|---|---|
| Classic Analog | `8:4` | `8:5` | `8:6` | `8:7` | `8:8` |
| Modern Digital 2027 | `8:9` | `8:10` | `8:11` | `8:12` | `8:13` |
| Race Dash | `8:14` | `8:15` | `8:16` | `8:17` | `8:18` |
| Liquid Glass | `8:19` | `8:20` | `8:21` | `8:22` | `8:23` |
| Sci-Fi | `8:24` | `8:25` | `8:26` | `8:27` | `8:28` |
| Fluid / Generative | `8:29` | `8:30` | `8:31` | `8:32` | `8:33` |
| ADAS & HUD | `8:34` | `8:35` | `8:36` | `8:37` | `8:38` |

## Blocked: what is left in Figma, and how to finish it
Phase 3 hit the Starter plan's Figma MCP call cap ("You've reached the Figma MCP tool call limit on the Starter plan"), after the foundations and the page-2 skeleton. Still to do, in order:
1. **Place the renders:** each hero PNG `docs/design/heroes/<family>/<state>.png` as an image fill (scale FILL) on the frame ID above; each template PNG `docs/design/families/<family>-template.png` on its template frame (scale FIT). With MCP access this is `upload_assets` with `nodeIds` (28 + 7 uploads); by hand, drag each PNG onto its frame.
2. **Finish the component library on `0:1`** from [COMPONENTS.md](COMPONENTS.md), binding the variables and styles above.
3. **Page `1:5`:** the OSD and configurator direction frames (`docs/design/directions/*.png`), the scrim-fit frames (`docs/design/scrim/*.png`), and one Section per catalog row for Phase 7.
4. **Mirror tokens v3.2** into the Palette and Roles variables.

Unblock by upgrading the Figma plan (Professional removes the 3-page and 1-mode limits and raises the MCP cap) or by waiting for the cap to reset; the conductor or the Phase 7 setup chat can then run steps 1–4.

## HTML boards (authoritative visuals)
| Folder | What | PNG output |
|---|---|---|
| `boards/heroes/` | 28 hero boards (`<family>-<state>.html`) | `docs/design/heroes/<family>/<state>.png` (3840×2160) |
| `boards/families/` | 7 template boards | `docs/design/families/<family>-template.png` |
| `boards/scrim/` | 3 scrim-fit boards over `assets/reference/*.jpg` | `docs/design/scrim/*.png` |
| `boards/directions/` | OSD and configurator directions | `docs/design/directions/*.png` |
| `boards/index.html` | Gallery of everything above | — |

**Render a board to PNG** (headless Edge or Chrome on Windows; fonts load from Google Fonts, so stay online):
```
msedge --headless=new --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --window-size=3840,2160 --allow-file-access-from-files --virtual-time-budget=9000 --screenshot=<out.png> file:///<path to board>.html
```
Use `--window-size=1920,1080` for the template, scrim and direction boards (they scale a 4K viewBox down). The boards were generated by a scratch script kept outside the repo (Rust-only rule); edit the SVG in a board directly, or regenerate from the tokens.
