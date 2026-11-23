# PENPOT: the design tool (since 2026-11-23)

New design work goes to **Penpot**, through Penpot's official hosted MCP: Phase 7 cluster pages, the Phase 10 configurator, the Phase 11 showcase and any redesign. **Figma is the read-only legacy master** ([FIGMA.md](FIGMA.md)): read it for history, never write to it. The HTML boards in `docs/design/boards/` stay the source of the Phase 3 and 9 pixels; the [master file](#master-file) holds their renders, the token mirror and the component library.

## Why we moved
- The Figma account has a **View seat on a Starter team**. The remote Figma MCP stops at the Starter tool-call cap, and the desktop Dev Mode MCP (`figma-desktop`, `127.0.0.1:3845`) needs a Dev or Full seat, so it always fails ([FIGMA.md](FIGMA.md), "Blocked").
- A third-party Figma "MCP Bridge" plugin was looked at and dropped.
- TJX has an **unlimited Penpot plan**, and Penpot's hosted MCP writes to the open file through the Penpot plugin API (`execute_code`). None of the Figma Starter workarounds carry over: no 3-page limit, no one-Section-per-cluster layout, no single variable mode, no MCP call cap.

## Setup (TJX, once per machine)
1. In Penpot, open **Your account → Integrations → MCP Server** and create a key.
2. Register the server at user scope, in your own terminal:
   ```
   claude mcp add --transport http --scope user penpot "https://design.penpot.app/mcp/stream?userToken=<your key>"
   ```
3. Open the design file in the browser, open the main menu (**⋮** beside the file name) and choose **MCP server → Connect**. It is a top-level entry next to File, Edit and View, not inside File. When it's connected, the toolbar shows a green **● MCP** badge and the submenu offers Disconnect and Manage.
4. **Keep that tab open**, then start a new Claude session (MCP servers load at session start).

**The key is a secret.** A chat never asks for it, and it never goes into a file in this repo or into the repo's `.mcp.json`. User scope keeps it in TJX's own Claude config. If it leaks, revoke it under Integrations and create a new one.

## Using it in a chat
1. `ToolSearch` with `penpot` must list `execute_code`, `high_level_overview`, `penpot_api_info` and `export_shape`. If they are missing, stop and point TJX to the setup above.
2. Call `high_level_overview` **once per chat**, before the first `execute_code`. Look up types and members with `penpot_api_info` rather than assuming an API exists.
3. Write with `execute_code`: JavaScript against the Penpot plugin API, run inside the open tab. It is tool input, not a repo file; never commit these snippets (Rust-only rule).
4. Shapes can only be created or changed on the active page: `await penpot.openPage(page)` before writing to another page.
5. Changes apply asynchronously: `await penpot.waitForLayoutUpdate()` before reading back what you just changed.
6. Check the result with `export_shape`, then critique it (`fh6-hmi-design-rules`, `design:design-critique`).
7. Design to code: `penpot.generateStyle(shapes, { type: "css" })` and `penpot.generateMarkup` (through `execute_code`) give exact values; the Rust side still takes every value from tokens.
8. If `execute_code` answers *"No Penpot instance connected for user token"*, the file tab is closed or not connected. Ask TJX to open the file and choose **⋮ → MCP server → Connect**, then retry. (This is what the tools answered on 2026-11-23, when this doc was written: the MCP was registered and `high_level_overview` worked, but no file tab was connected.)
9. Gotchas found while building the master file (2026-11-23):
   - **Typography tokens:** applied to many texts in one call, some texts keep the old font in the render although the shape reports the token. Apply the library typography first (`typography.applyToText(text)`), `await text.waitForLayoutUpdate()`, then `text.applyToken(token, ["typography"])`, one text at a time.
   - **Token references** must point at a token that already exists: create `family/*` before `core/color` tokens that reference `{palette.*}`. Composite token fields are strings (`blur: "12"`, `fontSizes: "160"`); colours take `#RRGGBBAA` for alpha.
   - `penpot.createPath()` gets a default black stroke: set `strokes = []` unless you want one. Board `strokeWidthTop` and friends throw in this build; draw a 1 px rectangle for a divider.
   - `penpot.library.local.components` lists one entry per variant group. Reach the variants through the container: `container.variants.variantComponents()` and each one's `variantProps`.
   - `export_shape` PNGs flatten alpha in the chat preview, so a low-alpha white (an unlit pod, `rev.unlit`) looks solid white on a transparent board. Export the black section board behind it, or the SVG, to judge alpha.
   - `File.name` is read-only in the plugin API; rename the file in the Penpot UI.

## Limits of the hosted MCP
| Limit | Work-around |
|---|---|
| **The tab must stay open.** The MCP only reaches the file while it is open in a browser tab and connected. Closing or reloading the tab, or the machine sleeping, drops the connection. | Keep the tab open for the whole design session; reconnect with ⋮ → MCP server → Connect. |
| **One live tab per key.** Every chat that uses the key drives the same connected tab, and so shares its active page. | Prefer one Penpot design chat at a time. When chats run in parallel, each `execute_code` call first opens the chat's own page (`await penpot.openPage(...)`) and touches nothing outside it; never assume the active page. |
| **No image import from a local path.** The hosted server has no `import_image` tool, and `fetch()` inside `execute_code` is blocked (NetworkError), so a local file or `localhost` URL can't reach Penpot. | **Committed and pushed PNGs import automatically:** `await penpot.uploadMediaUrl(name, url)` fetches on Penpot's server, so a raw GitHub URL of this public repo pinned to a commit works, for example `https://raw.githubusercontent.com/T-J-X/Forza-Revampled-Telemetry/<sha>/docs/design/osd/09_controls.png`. Put the returned `ImageData` in a fill (`{ fillOpacity: 1, fillImage }`). Every render in the master file came in this way. Only uncommitted files (fresh `mockups/` renders) still need TJX to drag them in by hand. |
| **Limited export.** `export_shape` takes one shape ID (or `selection` / `page`) and returns a PNG or SVG to the chat for review, or a shape's raw image fill as PNG. It has no PDF, no scale setting and no batch. | Use it to look at a board. For files kept in the repo, export from the Penpot UI, or keep using `preview-export` and the HTML boards. |

## Master file
Built on 2026-11-23 by the "Build the Penpot design-system master file" session.

- **File ID:** `e7ca6b81-c0d3-8029-8008-fd87488aab8f`. Its URL is `https://design.penpot.app/#/workspace?team-id=<team>&file-id=e7ca6b81-c0d3-8029-8008-fd87488aab8f` (open it from the Penpot dashboard; the key never goes in a URL or a file).
- **Name:** it was connected as **"New File 1"**, and the plugin API can't rename a file. **TJX renames it to `FH6 WheelDash v2 — Design System`** in the Penpot UI; the ID doesn't change.
- **Saved version:** "Phase 3 master build (tokens v3.2, components, heroes, OSD, scrim)" (File → History).
- **Images:** every render is a locked rectangle named `render · <path>` inside its board, imported with `uploadMediaUrl` from this repo's raw GitHub URLs at commit `71994b2`. The board itself is `#000000`, so a designer can hide the render and draw over it in 4K coordinates.
- **Scale:** every 4K board is 3840×2160 in 4K px. Template renders are 1:1 crops placed where the HTML board's `viewBox` put them; scrim and direction renders are 1920×1080 exports scaled ×2; OSD renders are 960×540 kit renders scaled ×4.

Penpot shape IDs are UUIDs (`shape.id`). Record board IDs, not names, in catalogs and handovers. Page numbers follow the phase that owns the page: 01–04 are the Phase 3 system, 08 the Phase 8 scrim, 09 the Phase 9 OSD, 10 the Phase 10 configurator. Penpot appends new pages at the end; drag a page in the Pages panel to reorder.

| Page | ID | Contents |
|---|---|---|
| `00 · Cover` | `e7ca6b81-c0d3-8029-8008-fd87488aab90` | Cover board `6fe75c5e-bb22-8084-8008-fd8b18014b21` (1920×1080) |
| `01 · Tokens` | `6fe75c5e-bb22-8084-8008-fd8a2bc86eb2` | Reference boards bound to the tokens (table below) |
| `02 · Components` | `6fe75c5e-bb22-8084-8008-fd8a2bc96163` | The 16 components from [COMPONENTS.md](COMPONENTS.md), one black `§` section board each (table below) |
| `03 · Family templates` | `6fe75c5e-bb22-8084-8008-fd8a2bca0c17` | 7 template boards, 3840×2160, with a `Guides · <view> (from tokens)` group: envelope, anchor and safe margins |
| `04 · Heroes` | `6fe75c5e-bb22-8084-8008-fd8a2bcb3427` | 28 hero boards, 3840×2160 (7 families × 4 review states) |
| `08 · Scrim fit` | `6fe75c5e-bb22-8084-8008-fd8a2bcc057a` | 3 scrim-fit boards with the dome core outline (`fh6-core::fit` defaults) and a spec table |
| `09 · OSD` | `6fe75c5e-bb22-8084-8008-fd8a2bcd3335` | 11 OSD states with a `Grid · OSD.md (4K units)` group (shown on OSD 09, hidden on the rest), the Phase 3 OSD direction, and the grid/type and navigation tables |
| `10 · Configurator` | `6fe75c5e-bb22-8084-8008-fd8a2bce3f4e` | The Phase 3 configurator direction (1920×1080); Phase 10 builds here |
| `07 · <Name>` | not created | Phase 7 cluster pages, one per cluster, made by each cluster chat (`program/phases/07-cluster-factory/CLUSTER_PROMPT.md`); board IDs go in `CATALOG.md` |
| `11 · Showcase` | not created | Phase 11 |

### `01 · Tokens` boards
| Board | ID |
|---|---|
| Tokens · colour (core/color + active family) | `6fe75c5e-bb22-8084-8008-fd8b34234f3d` |
| Tokens · family palettes (family/* sets, Family theme) | `6fe75c5e-bb22-8084-8008-fd8b6b2545e5` |
| Tokens · type (core/type typography tokens) | `6fe75c5e-bb22-8084-8008-fd8b6e599453` |
| Tokens · geometry, glow and glass | `6fe75c5e-bb22-8084-8008-fd8bdabc86db` |
| Tokens · motion, layout and review states | `6fe75c5e-bb22-8084-8008-fd8c05f8802b` |

### `02 · Components`
Each component's main lives inside its `§` section board; the components sit under the `FH6` path in the Assets panel. Fills, strokes, glows and type are bound to tokens wherever the value has one (`role.*`, `rev.*`, `thermal.*`, `glass.*`, `glow.*`, `type.*`, `stroke.*`, `radius.*`), so switching the Family or Glass theme recolours the library. Gradients (glass rim, brushed titanium, liquid metal) and per-stage glow colours are literal values copied from the tokens, because Penpot can't bind a gradient stop or a shadow colour to a token.

| Component | Rank | Variants (properties) | Main / variant container ID | Section board ID |
|---|---|---|---|---|
| HeroGear | 1 | 24 (Size hero·compact × Gear N R 1–10) | `6fe75c5e-bb22-8084-8008-fd8c6cce8785` | `6fe75c5e-bb22-8084-8008-fd8c585123bb` |
| Seg7Digit | — | 11 (Value 0–9, blank) | `6fe75c5e-bb22-8084-8008-fd8cc1c26ed9` | `6fe75c5e-bb22-8084-8008-fd8c9c0218ad` |
| TabularSpeed | 2 | 5 (Style hero·compact·light·race·segment) | `6fe75c5e-bb22-8084-8008-fd8d538f0fff` | `6fe75c5e-bb22-8084-8008-fd8d44f96d46` |
| ShiftLights | 1 | 18 (Type race10·modern11·glass16·analog15·beads11·vfdStrip × State unlit·staged·shift) | `6fe75c5e-bb22-8084-8008-fd8d971f5cc1` | `6fe75c5e-bb22-8084-8008-fd8d75d5176e` |
| RevLimiterStrobe | 1 | 3 (Phase on·off·reducedFlash) | `6fe75c5e-bb22-8084-8008-fd8df520d35a` | `6fe75c5e-bb22-8084-8008-fd8def1f407f` |
| MicroLabel | 5 | 4 (Size micro·nano × State label·shyActive) | `6fe75c5e-bb22-8084-8008-fd8df7f93450` | `6fe75c5e-bb22-8084-8008-fd8df5322627` |
| LogoZone | 5 | 4 (Type crest·wordmark·customName·dial) | `6fe75c5e-bb22-8084-8008-fd8e1fb65900` | `6fe75c5e-bb22-8084-8008-fd8e1c108a4b` |
| GlassPlate | 5 | 2 (Shape squircle·pill) | `6fe75c5e-bb22-8084-8008-fd8e23117a8e` | `6fe75c5e-bb22-8084-8008-fd8e1fca34b9` |
| Bezel | 5 | 4 (Type titaniumRing·chamfer45·carbonFrame·frameless) | `6fe75c5e-bb22-8084-8008-fd8e5a91b2b2` | `6fe75c5e-bb22-8084-8008-fd8e57e31c6b` |
| AdasRoad | 4 | 1 | `6fe75c5e-bb22-8084-8008-fd8e8b5b7792` | `6fe75c5e-bb22-8084-8008-fd8e8b12c8b5` |
| GBall | 4 | 1 | `6fe75c5e-bb22-8084-8008-fd8e8d434b62` | `6fe75c5e-bb22-8084-8008-fd8e8d262fed` |
| TyreSlipQuad | 4 | 2 (State optimal·drift) | `6fe75c5e-bb22-8084-8008-fd8e976e850a` | `6fe75c5e-bb22-8084-8008-fd8e8d262fed` |
| SignedBar | 4 | 2 (Direction accel·brake) | `6fe75c5e-bb22-8084-8008-fd8e977cd395` | `6fe75c5e-bb22-8084-8008-fd8e8d262fed` |
| Tach | 3 | 6 (Type arc·bar·ribbon·needleDial·chevron·liquidChannel), all at state.cruise | `6fe75c5e-bb22-8084-8008-fd8f31c8dbc7` | `6fe75c5e-bb22-8084-8008-fd8ed53d2fd3` |
| AlarmColumn | 4–5 | 2 (State unlit·lit) | `6fe75c5e-bb22-8084-8008-fd8f5f9ce9a8` | `6fe75c5e-bb22-8084-8008-fd8f5708b9e0` |
| DataTile | 5 | 1 | `6fe75c5e-bb22-8084-8008-fd8f5fafde24` | `6fe75c5e-bb22-8084-8008-fd8f5708b9e0` |
| LiquidSurface | 4 | 1 | `6fe75c5e-bb22-8084-8008-fd8f6144c288` | `6fe75c5e-bb22-8084-8008-fd8f5708b9e0` |

Where the sources disagree, the library follows `data/figma_tokens.json` and the difference is open for the owner of COMPONENTS.md:
- HeroGear `N` uses `role.inkMuted` (`components.heroGear.neutral`); COMPONENTS.md says ink at 72%.
- ShiftLights uses the COMPONENTS.md variant names; `components.shiftLights.variants` in the JSON still lists the older `linear16`, `outsideIn15`, `converging5Pairs`, `arcJewels`, `ribbonPills16`.
- The AdasRoad camera uses the JSON's raised chase eye (0, 3.4, −6) m, pitch 8°, FOV 33°; COMPONENTS.md still quotes (0, 1.85, −3.4), 15°, 35°.
- Not drawable in Penpot, so left to the kit: frost noise, caustics, fresnel and lensing (GlassPlate), the 2×2 carbon twill texture (Bezel, gradient fallback), anisotropic brushing, the top-left glass crescent and the 45% white core's bloom.

### `03 · Family templates`
| Family | Template board | View |
|---|---|---|
| Classic Analog | `6fe75c5e-bb22-8084-8008-fd8a7e68e181` | cockpit |
| Modern Digital 2027 | `6fe75c5e-bb22-8084-8008-fd8a7e6cd8d6` | hood |
| Race Dash | `6fe75c5e-bb22-8084-8008-fd8a7e71919d` | cockpit |
| Liquid Glass | `6fe75c5e-bb22-8084-8008-fd8a7e751cc2` | cockpit |
| Sci-Fi | `6fe75c5e-bb22-8084-8008-fd8a7e78e33d` | cockpit |
| Fluid / Generative | `6fe75c5e-bb22-8084-8008-fd8a7e7c0866` | cockpit |
| ADAS & HUD | `6fe75c5e-bb22-8084-8008-fd8a7e800a18` | cockpit |

### `04 · Heroes`
| Family | 01 idle | 02 mid cruise | 03 redline strobe | 04 drift / braking |
|---|---|---|---|---|
| Classic Analog | `6fe75c5e-bb22-8084-8008-fd8a46f932d9` | `6fe75c5e-bb22-8084-8008-fd8a4700eaac` | `6fe75c5e-bb22-8084-8008-fd8a47076679` | `6fe75c5e-bb22-8084-8008-fd8a470cf9ee` |
| Modern Digital 2027 | `6fe75c5e-bb22-8084-8008-fd8a4a0c58e0` | `6fe75c5e-bb22-8084-8008-fd8a4a10d215` | `6fe75c5e-bb22-8084-8008-fd8a4a159cdd` | `6fe75c5e-bb22-8084-8008-fd8a4a19a375` |
| Race Dash | `6fe75c5e-bb22-8084-8008-fd8a4c9a7d3e` | `6fe75c5e-bb22-8084-8008-fd8a4c9d541a` | `6fe75c5e-bb22-8084-8008-fd8a4ca15ce3` | `6fe75c5e-bb22-8084-8008-fd8a4ca52c5e` |
| Liquid Glass | `6fe75c5e-bb22-8084-8008-fd8a52404667` | `6fe75c5e-bb22-8084-8008-fd8a524eb60d` | `6fe75c5e-bb22-8084-8008-fd8a525464b8` | `6fe75c5e-bb22-8084-8008-fd8a5259bece` |
| Sci-Fi | `6fe75c5e-bb22-8084-8008-fd8a54ffd9d3` | `6fe75c5e-bb22-8084-8008-fd8a55035dfc` | `6fe75c5e-bb22-8084-8008-fd8a550754be` | `6fe75c5e-bb22-8084-8008-fd8a550b6ca9` |
| Fluid / Generative | `6fe75c5e-bb22-8084-8008-fd8a5784b9f8` | `6fe75c5e-bb22-8084-8008-fd8a57887a00` | `6fe75c5e-bb22-8084-8008-fd8a578c7b87` | `6fe75c5e-bb22-8084-8008-fd8a5791d540` |
| ADAS & HUD | `6fe75c5e-bb22-8084-8008-fd8a5a24bcac` | `6fe75c5e-bb22-8084-8008-fd8a5a28fd61` | `6fe75c5e-bb22-8084-8008-fd8a5a2db9c8` | `6fe75c5e-bb22-8084-8008-fd8a5a31dfb1` |

### `08 · Scrim fit`
| Board | ID |
|---|---|
| Scrim fit · annotated (55%, factory binnacle visible) | `6fe75c5e-bb22-8084-8008-fd8aba28b9a2` |
| Scrim fit · stock cockpit (100%, Liquid Glass mid cruise) | `6fe75c5e-bb22-8084-8008-fd8aba32c967` |
| Scrim fit · low seat, wide FOV (−170 px offset) | `6fe75c5e-bb22-8084-8008-fd8aba39c0d8` |
| Scrim fit · spec (SCRIM_FIT.md §2) | `6fe75c5e-bb22-8084-8008-fd8ac79f375a` |

### `09 · OSD`
| Board | ID |
|---|---|
| OSD 01 · Cluster · carousel | `6fe75c5e-bb22-8084-8008-fd8aeed8fa18` |
| OSD 02 · Cluster · editing the Hood view | `6fe75c5e-bb22-8084-8008-fd8aeee0be9d` |
| OSD 03 · Colour · custom palette | `6fe75c5e-bb22-8084-8008-fd8aeee4d8b2` |
| OSD 04 · Colour · random palette | `6fe75c5e-bb22-8084-8008-fd8aeee886b3` |
| OSD 05 · Type & logo | `6fe75c5e-bb22-8084-8008-fd8aeeece75d` |
| OSD 06 · Fit & camera · FOV | `6fe75c5e-bb22-8084-8008-fd8aeef078b7` |
| OSD 07 · Fit & camera · cluster height | `6fe75c5e-bb22-8084-8008-fd8aeef3fe24` |
| OSD 08 · Fit & camera · dash blocker | `6fe75c5e-bb22-8084-8008-fd8aeef78e1b` |
| OSD 09 · Controls | `6fe75c5e-bb22-8084-8008-fd8aeefb62ed` |
| OSD 10 · On-screen keyboard | `6fe75c5e-bb22-8084-8008-fd8aeeff0834` |
| OSD 11 · Remapper | `6fe75c5e-bb22-8084-8008-fd8aef02ce42` |
| OSD direction (Phase 3, OSD_DIRECTION.md) | `6fe75c5e-bb22-8084-8008-fd8af204c0ef` |
| OSD · grid and type (4K units) | `6fe75c5e-bb22-8084-8008-fd8b0755efba` |
| OSD · navigation | `6fe75c5e-bb22-8084-8008-fd8b088f2686` |

### `10 · Configurator`
| Board | ID |
|---|---|
| Configurator direction (Phase 3, CONFIGURATOR_DIRECTION.md) | `6fe75c5e-bb22-8084-8008-fd8b17357b40` |

## Tokens
`data/figma_tokens.json` stays the single source of truth ([TOKENS.md](TOKENS.md)). The file name and the `FigmaTokens` loader name are historical and stay unchanged. Never edit a token in Penpot first.

Penpot has native design tokens (`penpot.library.local.tokens`, a `TokenCatalog`; checked with `penpot_api_info` on 2026-11-23):
- **Token sets** hold the tokens; only active sets affect shapes.
- **Token themes** activate a list of sets. Themes sit in groups, and only one theme per group is active at a time. So the per-family palettes become one set per family, switched by the `Family` theme group, instead of the separate Figma variables the 1-mode Starter limit forced.

Mirror the JSON into these sets and themes with `execute_code` after every token change.

### The mirror (tokens v3.2, 489 tokens)
| Set | Tokens | Holds |
|---|---|---|
| `core/color` | 69 | `role.*` (primary, secondary, accent, shift and glassTint reference `{palette.*}`; the fixed roles are literal, alpha as `#RRGGBBAA`), `rev.*` (stage1–3 reference go / caution / warning, `rev.unlit`), `input.*`, `thermal.*`, `proximity.*`, `assist.*`, `lume.*`, `adas.*` (with `adas.underglow.*`), `tyreSlip.ramp0–3`, `glass.rim.stop0/38/78/100`, `glass.innerRim`, `glass.arRimTint`, `glass.tintBase`, `material.*` colours |
| `core/type` | 86 | `font.family.*` (the 10 OFL roles), `font.capRatio.*`, `font.weight.w100–w800`, per scale step `font.cap.*` (cap px), `font.size.*` (= cap / capRatio), `font.tracking.*` (px), and the composite `type.*` typography tokens: the 10 scale steps plus `raceGear`, `glassSpeed`, `dialGear`, `fluidNumeral`, `hudNumeric`, `techLabel`, `wordmark`; `type.tabular.*`, `type.ratio.*` |
| `core/space` · `core/stroke` · `core/radius` | 9 · 17 · 9 | the 8 px grid (+ the 24 px lattice), stroke and tick widths and lengths, the needle, radii, squircle exponent, chamfer |
| `core/glow` | 26 | lobe σ and intensity per level, bloom threshold, bloom chain, neon |
| `core/glass` | 36 | tint alpha (0–255), frost, rim width, inner-rim inset, crown sheen, caustic opacity, fresnel, refraction, lensing, crescent, dimming pad, materialise; numeric `material.*` |
| `core/light` · `core/motion` | 8 · 47 | light azimuth and rim; springs (T, ζ), durations, shift lights, limiter strobe, gear change, ignition sweep, visibility, speed hysteresis, idle, afterimage |
| `core/layout` · `core/states` | 36 · 35 | canvas, cockpit / hood / raised-HUD anchors and envelopes, user scale and offset, safe margins, zone minimums, logo zone, classic dial; the 4 review states and the rpm limits |
| `core/components` · `core/accessibility` | 36 · 10 | numeric `components.*` (shift-light pods, ADAS camera, G-ball rings, signed bar, FPM); text floor, blur gate, colour budget, text-safe lift |
| `family/<key>` × 7 | 9 each | `palette.primary / secondary / accent / shift`, `palette.glassTint` (= mix(primary, #0A1220, 0.72)) and the shadow tokens `glow.soft / medium / hot / phosphor` in the family's primary (blur = 2σ, alpha = intensity) |
| `glass/regular` · `glass/tinted` | 1 each | `glass.tint.opacity` 0.17 (44/255) or 0.25 (64/255) |

Each token's description names its `figma_tokens.json` path. Strings with no Penpot token type (easing curves, rules, docs) are on the `01 · Tokens` boards instead.

| Theme group | Theme | ID | Activates |
|---|---|---|---|
| Family | Classic Analog | `6fe75c5e-bb22-8084-8008-fd8a17a27ba4` | every `core/*` set + `family/classicAnalog` |
| Family | Modern Digital 2027 (active) | `6fe75c5e-bb22-8084-8008-fd8a17a9af76` | `core/*` + `family/modernDigital` |
| Family | Race Dash | `6fe75c5e-bb22-8084-8008-fd8a17b11a5f` | `core/*` + `family/raceDash` |
| Family | Liquid Glass | `6fe75c5e-bb22-8084-8008-fd8a17b702cc` | `core/*` + `family/liquidGlass` |
| Family | Sci-Fi | `6fe75c5e-bb22-8084-8008-fd8a17bcc3e4` | `core/*` + `family/sciFi` |
| Family | Fluid / Generative | `6fe75c5e-bb22-8084-8008-fd8a17c2c713` | `core/*` + `family/fluidGenerative` |
| Family | ADAS & HUD | `6fe75c5e-bb22-8084-8008-fd8a17c9f5f7` | `core/*` + `family/adasHud` |
| Glass | Regular (active) | `6fe75c5e-bb22-8084-8008-fd8a17cf846b` | `glass/regular` |
| Glass | Tinted | `6fe75c5e-bb22-8084-8008-fd8a17d0ed6a` | `glass/tinted` |

The local library also holds **95 colours** (`FH6/<group>/…` for the literal core colours, `FH6/Families/<family>/…` for the palettes) and **17 typographies** (`FH6/<step>`, the same set as the `type.*` tokens) for the Assets panel.

**After a token change:** edit `data/figma_tokens.json` first, then in Penpot update the token of the same name (`penpotUtils.findTokenByName(name)`, then set its value) or add it to the set named above, following these rules: font size = cap / capRatio, tracking in px = trackingEm × font size, alpha 0–255 → `#RRGGBBAA`, glow lobes → shadow layers with blur 2σ. Re-check the `01 · Tokens` boards with `export_shape`.

## Carried over from Figma: done
The to-do list that Figma's cap blocked ([FIGMA.md](FIGMA.md), "Blocked") was finished in the master file on 2026-11-23:
1. Tokens **v3.2** mirrored into 22 token sets and 2 theme groups (above). The v3.2 critique colours (shift `#978DFC`, Classic primary `#FF7A1A`, Race Dash shift `#B27CFF`) are in.
2. The component library from [COMPONENTS.md](COMPONENTS.md), bound to those tokens, on `02 · Components`.
3. The 28 hero renders and the 7 template renders placed automatically (`uploadMediaUrl`, see the limits table), on `04 · Heroes` and `03 · Family templates`.
4. The OSD page (`09 · OSD`), the scrim fit (`08 · Scrim fit`) and the configurator direction (`10 · Configurator`).

Left for TJX: rename the file (see [Master file](#master-file)), and delete or keep the empty white 208×902 "Board" (`6fe75c5e-bb22-8084-8008-fd89a6d4d466`) on `00 · Cover`, which the build didn't create (it appeared in the shared tab during the build).
