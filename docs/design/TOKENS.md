# TOKENS: the FH6 WheelDash v2 design tokens

`data/design_tokens.json` is the single source of truth for colour, type, geometry, light, glass and motion. Penpot mirrors it as token sets (see [PENPOT.md](PENPOT.md)). `fh6-render::tokens` / `fh6-kit` read it at startup. No cluster hardcodes a hex value, a size or a timing that has a token.

- **Version:** 3.1.0 (schema v3, 2026-11-20). v1 keys are kept for the current tiny-skia code.
- **Units:** every size is in **4K pixels at cluster scale 1.0**. Multiply by the user scale (0.70–1.40). The one exception: specular rims are always exactly 1 px, snapped to the pixel grid.
- **Alpha:** 0–255, unless a key ends in `Pct`, `Opacity` or is written as a 0–1 fraction in a gradient stop.
- **Colour space:** hex values are sRGB. `fh6-kit` converts them to linear premultiplied RGBA before blending (SKILLS.md law 1).
- **Why these values:** [RESEARCH.md](RESEARCH.md) §8–10. Values marked "est." there are starting points to verify on the C1.

## Top-level map

| Key | v | What it holds | Read by |
|---|---|---|---|
| `$schema`, `meta` | 1→3 | version, display, units, research basis; `penpotFileId` names the Penpot master file (see PENPOT.md) | tooling |
| `typography` | 1 | **legacy** tier sizes in pt; font stacks updated to OFL faces | current tiny-skia code only |
| `sharedSemantics` | 1 | **legacy** rev-stage and input colours | current code only |
| `colorRoles` | 3 | the roles every cluster colours itself with, plus rev stages, inputs, thermal, proximity, assists, lume, ADAS colours and the colour budget | kit, clusters, OSD |
| `paletteModes` | 3 | factory / custom / random palette rules and the 7 family default palettes | OSD, configurator, clusters |
| `type` | 3 | OFL font roles, cap-height type scale, ratios, tabular rules, user-font rules | kit text, clusters |
| `space`, `stroke`, `radius` | 3 | 8 px grid, stroke and tick widths, radii, chamfer | kit, clusters |
| `glow` | 3 | bloom lobes per level, the emissive-rank rule, bloom chain, neon | kit post-process |
| `glass` | 3 | the Liquid Glass material | kit material, OSD |
| `light` | 3 | the one virtual light per cluster, driven by lateral G | kit, clusters |
| `materials` | 3 | named looks with Phase 6 textures and procedural fallbacks | kit, Phase 6 |
| `motion` | 3 | closed-form springs, M3 durations, easings, shift lights, limiter strobe, gear change, boot, sweep, visibility, idle | clusters, app |
| `layout` | 3 | anchors, envelopes, user scale/offset, zones, logo zone, depth layers, density tiers, classic dial faces | clusters, app, scrim |
| `states` | 3 | the 4 reference telemetry states used by every mockup and golden render | preview-export, tests |
| `components` | 3 | shared component parameters (anatomy in [COMPONENTS.md](COMPONENTS.md)) | `fh6-clusters/src/common/` |
| `families` | 3 | the 7 families, their spec file, pattern-setter and hero folder | Phase 7 |
| `accessibility` | 3 | reduced flash, minimum text, colour independence, blur gate, flash audit | QA, OSD |
| `clusters` | 1 | **legacy** per-cluster v1 layouts and palettes | current code; Phase 7 replaces each entry |

The current Rust loader (`fh6-render::tokens::DesignTokens`) reads only optional `colors` and `grid` keys and ignores the rest, so the extension is backward compatible. v3 deliberately does **not** use those two key names.

## Colour

### Roles (`colorRoles.roles`)
| Role | Source | Use |
|---|---|---|
| `primary` | palette | hero accents, tach fill, needle, ego-car tint |
| `secondary` | palette | trim, logo-zone ring, secondary arcs |
| `accent` | palette | leading edges, meniscus orb, focus, active segment |
| `shift` | palette | the limiter strobe and the final shift stage |
| `warning` `#FF2D3A` | fixed | redline sector, brake, alarms. Never replaced by a brand colour |
| `caution` `#FFB020` | fixed | caution stage, assists active, slip |
| `go` `#2BE37A` | fixed | first shift stage, throttle |
| `glassTint` | derived | `mix(primary, #0A1220, 0.72)` at `glass.tintAlpha` |
| `ink`, `inkMuted` (158), `inkLabel` (120), `inkGhost` (16) | fixed `#F4F7FB` | digits, units, labels, ghost 8s and unlit segments |
| `hairline` (46), `rimSpecular` (216) | fixed white | 1 px tracks and the top of every rim |
| `oledBlack` | fixed `#000000` | off pixels, VFD cavities, the scrim core |

**Text-safe lift** (`derivedTextSafe`): any role drawn as text or a thin line on black is converted to OKLCH and lifted to L ≥ 0.70 (chroma ≤ 0.26) so it keeps ≥ 7:1 on black. BMW's `#0066B1`, for example, is lifted before it is used for digits; its fills keep the true brand value.

### Rev stages (`colorRoles.revStages`)
Pods light on **per-car, per-gear thresholds** from the car profile; the fractions below are the fallback, as fractions of `shiftRpm = limiterRpm × shiftPct` (default 0.97). Fill is **outside-in**, landing on the centre pod above the gear. Attack is 0 ms (same frame).

| Stage | From | Colour |
|---|---|---|
| stage1 | 0.75 | `go` `#2BE37A` |
| stage2 | 0.85 | `caution` `#FFB020` |
| stage3 | 0.93 | `warning` `#FF2D3A` |
| shift | 1.00 | palette `shift` (default `#7C5CFF`; brands whose `ShiftFlashHex` is white strobe white) |

Unlit pods are **neutral** (white at alpha 22 or less), never a dim version of their stage colour: colour appears only when something is lit. Off hysteresis is 120 rpm.

### Other semantic sets
- `thermal`: 4 discrete steps (cold `#3B82F6` < 70 °C, optimal `#2BE37A` < 100, hot `#FFB020` < 112, overheated `#FF2D3A`), crossfaded over 150 ms. Never a continuous hue sweep.
- `proximity`: 3 steps grey → amber → red. `assists`: on / off / intervening (GT7 code).
- `lume`: Super-LumiNova night and day colours for analog markers.
- `adas`: neutral object grey at 70%, in-path blue `#3BA7FF`, imminent red, cones; at most 2 saturated objects, 6 road users and 3 lanes.
- `budget`: base hue ≥ 80% of lit pixels, white ≤ 15%, red ≤ 5% (except at the limiter); red is never structural. Checked with a lit-pixel histogram per mockup.

### Palette modes (`paletteModes`)
- **Factory:** `assets/car_brands/car_brands.json` → `BrandPalettes[brand]`: `PrimaryHex → primary`, `SecondaryHex → secondary`, `AccentHex → accent`, `ShiftFlashHex → shift`. Unknown brand → `DEFAULT`, then the family default.
- **Custom:** the user sets primary, accent and shift; secondary defaults to primary rotated −30° at L − 0.12.
- **Random:** a re-roll in OKLCH (primary L 0.66–0.80, C 0.14–0.24; accent at +30/−30/+150/180°; secondary the complement at low chroma; shift the highest-contrast of white, accent, `#7C5CFF`, `#00E5FF`). A primary hue within 15° of the warning hue is rejected, so red never means two things. A lock keeps the roll per cluster.
- **Family defaults:** one palette per family (`familyDefaults`), used by every mockup in `docs/design/heroes/`.

## Type (`type`)
Sizes are **cap heights**. Font size = `capPx / capRatio` of the active face, so a user font keeps the same optical size.

| Role | Face (OFL) | Use |
|---|---|---|
| `display` | Saira Condensed 300–800 | hero gear, speed, primary metrics: the motorsport default |
| `text` | Titillium Web 400/600/700 | labels, units, micro-labels, OSD text |
| `light` | Saira 100–600 | floating light numerals (Liquid Glass, ADAS) |
| `dial` | Barlow Semi Condensed 500–700 | analog dial numerals (DIN-like) |
| `wide` | Michroma | wordmarks, the custom name |
| `tech` | Chakra Petch 500–700 | Sci-Fi labels |
| `fluid` | Exo 2 200–800 | Fluid / Generative numerals |
| `aviation` | B612 Mono | HUD numerics (designed by Airbus for cockpit displays) |
| `race` | Saira Condensed 700/800 | Race Dash gear and values |
| `segment` | procedural | VFD 7-segment digits drawn as SDF segments, not a font |
| `fallback` | Noto Sans | missing glyphs, including JP |

Scale (cap px): gearHero 200, gearCompact 150, speedHero 112, speedCompact 84, metricPrimary 44, metricSecondary 30, dialNumeral 34, unit 20, microLabel 16, nano 12 (the floor). Micro-labels are uppercase with +16% tracking at `inkLabel`. Ratios (`type.ratios`): speed is 0.40–0.56 of the gear, labels 0.22–0.30 of their value.

**Tabular digits:** every number sits in fixed cells of `0.60 em`, centred, whatever the face. Speed is a fixed 3-cell field; unused leading cells draw a ghost `8` in `inkGhost`, so the width never changes from 0 to 999. Phase 6 records which faces ship `tnum`; the kit forces fixed advances either way.

**User fonts:** a user font replaces `display` (and optionally `text`). Layouts survive because sizes are cap-height based, digits use fixed cells, labels shrink to 85% then elide, and vertical centring uses cap height. Faces with a cap ratio outside 0.60–0.78 are clamped.

## Geometry
- `space`: 8 px grid (steps 8 … 128). The 24 px Oblivion lattice is 3 units.
- `stroke`: rim and hairline 1, xs 2, s 3, m 4, l 6, xl 10; ticks minor 2 / mid 3 / major 5 with lengths 14 / 22 / 34; needle 14 px base to 3 px tip at 92% of the radius.
- `radius`: xs 6 … xl 64, pill, squircle exponent 4.0, chamfer 45° / 28 px.

## Light, glow and glass
- **`light`:** one virtual light per cluster. Azimuth `135° + 25° × clamp(gLat / 1.5)`, elevation 40°, on a critical spring (T 0.15 s). It drives every 1 px rim (peak 60–80% white, exponent 24–48), every facet and every brushed finish. In a steady 1 g corner the highlight must move at least 15°.
- **`glow`:** levels `soft`, `medium`, `hot`, `phosphor` as Gaussian lobes (σ in px, intensity). **Bloom is earned:** only glance ranks 1–3 (shift lights, gear, speed, tach fill or needle) write to the emissive buffer. Labels, bezels, tyres and the G-ball never glow. Bloom chain: 8 levels from 256 px, threshold 0.7, knee 0.7, intensity 0.8; radius ≤ 96 px for the budget.
- **`glass`:** tint alpha 28 / 44 / 64 (regular up to 44, tinted 56–64, one variant per cluster); frost noise (1.4 px, 3.5%); 1 px rim gradient `#FFFFFF 92% → #38BDF8 59% → #0284C7 27% → #0EA5E9 49%` top to bottom (accent may replace the cyan); 1 px inner rim inset 2 px at alpha 40; a crown sheen under the top edge (18% of the height, sag 44 px, alpha 76 → 26 → 0) that shifts with lateral G; caustic texture (Phase 6 asset, screen blend at 0.32, procedural fallback); fresnel (IOR 1.52, R0 0.0426); self-lensing of our own layers (24–40 px band); a dimming pad behind gear and speed (alpha ≤ 140 × (1 − scrim %)). Glass **materialises** over 200 ms by ramping lensing and rim, never opacity; hiding is 1 frame. Separation comes from rims and inner light, never from grey shadows.
- **`materials`:** brushed titanium, guilloché (clous de Paris, barleycorn, wave, sunburst), enamel dial, carbon twill, VFD cavity (true black, mesh, 2 px scanlines inside the cavity only), liquid metal (environment bands + smooth-union surface + specular), sapphire.

## Motion (`motion`)
Clusters animate from `FrameInput` `dt`, never the wall clock. Springs are **closed-form** in (period T, damping ζ), exact at 60/120/144 Hz and allocation-free.

| Spring | T (s) | ζ | Use |
|---|---|---|---|
| `needle` | 0.070 | 0.75 | live needle, with velocity feed-forward (lag ≤ 16.7 ms) |
| `ribbon` | 0.070 | 0.90 | bar / ribbon tach fill, feed-forward |
| `snap` | 0.102 | 1.0 | small state snaps |
| `fast` | 0.168 | 0.9 | incoming gear glyph |
| `default` / `slow` / `expressive` | 0.237 / 0.363 / 0.322 | 0.9 / 0.9 / 0.8 | panels, OSD |
| `light` | 0.150 | 1.0 | virtual light azimuth |
| `gBall` | 0.168 | 0.9 | G-ball |
| `parallax` | ω 12 rad/s | — | depth layers 0 / 2 / 4 px per g |
| `fluidSlosh` | 0.62 | 0.35 | liquid tilt, deliberately under-damped, settles at zero input |

- **Durations** sit on the Material 3 50 ms scale (50 … 1000). Named: micro 50, short 100, crossfade 150, materialize 200, medium 300, long 500, boot ≤ 1000. Easings (`emphasized`, `emphasizedDecelerate`, `standardAccelerate`) are baked to 256-entry LUTs at load.
- **Shift lights:** 0 ms attack, release ≤ 50 ms.
- **Limiter strobe:** 10 Hz, 50% duty (3 on / 3 off frames at 60 Hz, 6 / 6 at 120 Hz), **on the shift strip only** (≤ 42 px tall at scale 1.0, ≤ 59 px at 1.40). Nothing taller flashes faster than 3 Hz; no large saturated-red area ever flashes; red washes are steady ramps. The hero gear may pulse at 2 Hz after 0.5 s on the limiter. Reduced-flash mode swaps every strobe for a 2 Hz sine breath.
- **Gear change:** the old glyph scales to 0.92 and fades in 50 ms; the new glyph enters from 1.12 with a 24 px slide (up for an upshift) on the `fast` spring; a 1-frame rim flash at 60%. Each family adds one flourish (for example, glow running round the glass rims at 4000 px/s).
- **Ignition sweep:** 350 ms up, 300 ms down, only on the first `IsRaceOn` 0 → 1 after launch, never on unpause, skippable. Boot ≤ 1 s; gear and speed are live from frame 1.
- **Visibility:** hide in 0 ms (within one frame, AGENTS rule 9), show over 200 ms.
- **Idle is still:** with constant telemetry nothing moves outside telemetry-bound regions; static structure stays ≤ 20% luminance.
- **Speed display:** the shown integer changes only when `|v − shown| ≥ 0.6`, so the last digit never flickers.
- **Afterimage:** LFA-style ghost needles (up to 4, `[f32; 4]` ring buffer) above 180°/s.

## Layout (`layout`)
| View | Anchor | Envelope (local x, y, w, h) |
|---|---|---|
| Cockpit `[3]` | (1920, 1835) | −720, −320, 1440, 560 |
| Hood `[4]` | (1920, 1760) | −880, −220, 1760, 400; scrim forced to 0% |
| Raised HUD (`WindshieldHud`) | (1920, 1595) | +240 px up the sightline |

- User scale 0.70–1.40 and height offset ±240 px; safe margins 40 px bottom, 160 px sides.
- **Zones (rank order):** Z1 gear + shift strip (gear cap ≥ 150, strip span ≥ 720, strobe strip ≤ 42 px), Z2 speed (cap ≥ 84, ≤ 420 px from the gear), Z3 tach, Z4 visualisation directly under speed and gear, Z5 bezel and labels.
- **Depth:** at least 3 layers (hero 0, tach ±2, bezel ±4 px of G parallax).
- **Density tiers:** full, race (ranks 1–3 plus alarms), zen (gear, speed, shift lights).
- **Logo zone:** crest ≤ 96 px, or wordmark ≤ 220 × 44 px; the dial variant is mono white at alpha 215 with a 1 px engraved shadow. Without a logo, the custom name is set in `wide` caps at cap 18 with +30% tracking (≤ 14 characters).
- **Classic dial:** default symmetric 270° face; optional per-car **Track mode** rotates the scale so the shift rpm stands at 12 o'clock; the operating band (peak torque → peak power) is a lume-green band on the chapter ring.

## Review states (`states`)
Every mockup, golden render and critique uses these. `limiterRpm` 8700, `maxRpm` 9000, `idleRpm` 900.

| State | rpm | gear | mph | thr | brk | gLat | gLong | slip |
|---|---|---|---|---|---|---|---|---|
| idle | 900 | N | 0 | 0 | 0 | 0 | 0 | 0° |
| cruise | 5200 | 4 | 118 | 0.42 | 0 | 0.18 | 0.08 | 1° |
| redline (strobe phase on) | 8680 | 5 | 152 | 1.00 | 0 | 0.05 | 0.42 | 0° |
| drift / braking | 7100 | 3 | 74 | 0.30 | 0.62 | −0.96 | −0.58 | 27° |

G-ball convention: the ball shows the force the driver feels (a ball in a bowl), i.e. `−acceleration`.

## Accessibility (`accessibility`)
Reduced flash (nothing above 3 Hz), text floor 12 px cap, colour independence (every stage change is also a count or position change), a **blur gate** in CI (at 960×540 and σ 1.5 the gear keeps ≥ 7:1 and the speed ≥ 4.5:1), an automated WCAG 2.3.1 flash-area audit over `preview-export` frames, and a Tinted glass toggle.

## Changing tokens
1. Edit `data/design_tokens.json` (UTF-8, no BOM, valid JSON; `node -e` or `cargo test` must parse it).
2. Mirror the change into the Penpot token sets with `execute_code` (PENPOT.md, "Tokens").
3. Log the change in `docs/PROGRESS.md` and, if it changes a rule, here.
