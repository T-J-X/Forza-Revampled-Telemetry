# PENPOT: the design tool (since 2026-11-23)

New design work goes to **Penpot**, through Penpot's official hosted MCP: Phase 7 cluster pages, the Phase 10 configurator, the Phase 11 showcase and any redesign. **Figma is the read-only legacy master** ([FIGMA.md](FIGMA.md)): read it for history, never write to it. The HTML boards in `docs/design/boards/` stay the authoritative pixels for Phases 3 and 9 until they are rebuilt in Penpot.

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

## Limits of the hosted MCP
| Limit | Work-around |
|---|---|
| **The tab must stay open.** The MCP only reaches the file while it is open in a browser tab and connected. Closing or reloading the tab, or the machine sleeping, drops the connection. | Keep the tab open for the whole design session; reconnect with ⋮ → MCP server → Connect. |
| **One live tab per key.** Every chat that uses the key drives the same connected tab, and so shares its active page. | Prefer one Penpot design chat at a time. When chats run in parallel, each `execute_code` call first opens the chat's own page (`await penpot.openPage(...)`) and touches nothing outside it; never assume the active page. |
| **No image import from a local path.** The hosted server has no `import_image` tool, so `execute_code` cannot place `docs/design/heroes/*.png`, `mockups/` renders or `assets/reference/*.jpg`. | TJX drags the PNG into the file by hand (or uses Penpot's image import in the UI); the chat then positions and names it with `execute_code`. |
| **Limited export.** `export_shape` takes one shape ID (or `selection` / `page`) and returns a PNG or SVG to the chat for review, or a shape's raw image fill as PNG. It has no PDF, no scale setting and no batch. | Use it to look at a board. For files kept in the repo, export from the Penpot UI, or keep using `preview-export` and the HTML boards. |

## Master file
Not created yet. TJX creates it in Penpot as **FH6 WheelDash v2 — Design System** and connects it; the first design chat builds the pages below and records here the project name, the file URL (never the key) and the main board IDs.

Penpot shape IDs are UUIDs (`shape.id`). Record board IDs, not names, in catalogs and handovers.

| Page | Contents |
|---|---|
| `01 · System` | Token sets and themes mirrored from `data/figma_tokens.json`, typographies, and the components from [COMPONENTS.md](COMPONENTS.md) |
| `02 · Families & Heroes` | 7 family templates and 28 hero boards (3840×2160, black) holding the PNGs from `docs/design/heroes/` and `docs/design/families/` |
| `03 · OSD · Configurator · Scrim` | The OSD (`docs/design/osd/`), the OSD and configurator directions, and the scrim fit (`docs/design/scrim/`) |
| `07 · <Name>` | Phase 7 cluster pages. The Phase 7 docs define them and record their board IDs in `program/phases/07-cluster-factory/CATALOG.md` |
| `10 · Configurator`, `11 · Showcase` | Phases 10 and 11 |

## Tokens
`data/figma_tokens.json` stays the single source of truth ([TOKENS.md](TOKENS.md)). The file name and the `FigmaTokens` loader name are historical and stay unchanged. Never edit a token in Penpot first.

Penpot has native design tokens (`penpot.library.local.tokens`, a `TokenCatalog`; checked with `penpot_api_info` on 2026-11-23):
- **Token sets** hold the tokens; only active sets affect shapes.
- **Token themes** activate a list of sets. Themes sit in groups, and only one theme per group is active at a time. So the per-family palettes become one set per family, switched by a `family` theme group, instead of the separate Figma variables the 1-mode Starter limit forced.

Mirror the JSON into these sets and themes with `execute_code` after every token change.

## Carried over from Figma
The to-do list that Figma's cap blocked ([FIGMA.md](FIGMA.md), "Blocked") moves here. It is **pending**, for the separate "Build the Penpot design-system master file" session, in this order:
1. Mirror tokens **v3.2** into Penpot token sets and themes.
2. Build the component library on `01 · System` from [COMPONENTS.md](COMPONENTS.md), bound to those tokens.
3. Place the hero and template PNGs on `02 · Families & Heroes` (by hand, see the import limit above).
4. Build `03 · OSD · Configurator · Scrim` from `docs/design/osd/`, `docs/design/directions/`, `docs/design/scrim/` and [boards/osd/index.html](boards/osd/index.html).
