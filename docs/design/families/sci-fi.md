# Family: Sci-Fi

**Pattern-setter:** ★ `RetroCyber` · **Members:** `RetroCyber`, `SynthwaveOutrun`, `EvaBioSync`, `ChronoEventHorizon`, `NeuralCyberSynapse`, `SolarisWarpCore`, `AeroPlasmaBlade`, `ShibuyaChromaticPrisma`; new: `StarSailOrigami`, `ArcLadderCascade`, `AuroraVeil` · **Hero:** [`heroes/sci-fi/`](../heroes/sci-fi/) · **Template:** [`sci-fi-template.png`](sci-fi-template.png)

## Language
Two dialects share one rulebook:
- **VFD / retro-futurist** (RetroCyber, SynthwaveOutrun): a true-black vacuum-fluorescent cavity, phosphor segments with ghosts, a control-grid mesh and scanlines inside the cavity only, behind a thin 45° chamfered bezel.
- **FUI / film** (the other members and new concepts): hairline structure on a dot lattice, procedural geometry, a three-term neon light (core, 6 px halo, 24 px bloom at 0.12), brand hue as a "faction" colour.

Both follow Oblivion and Tron's discipline: one graphic language, rationed greeble, glitch and aberration only as events, and the screen as a 3-state machine (initial, action, loop).

**Is not:** a flat framed rectangle, always-on scanlines over off pixels, rainbow bargraphs, cyan-magenta-yellow at equal weight everywhere.

## Layout grammar (cockpit, RetroCyber)
| Zone | Element | Geometry |
|---|---|---|
| Z5 | chamfer45 bezel | 1380 × 540 at (−690, −300), 64 px chamfers, 10 px total |
| Z1 | 24-pod VFD strip (the strip) | y −270, 42 px pods, 8 px gaps, 28 px tall; outside-in cyan → yellow → magenta |
| Z3 | 24-segment chevron staircase | 14 cyan / 6 yellow / 4 magenta, steady |
| Z1 | octagon gear bay | 296 px flat-to-flat at (0, 70); 7-seg gear 236 px |
| Z4 | drift columns | the bay's brackets become 6-segment slip columns (5/10/15/20/30/45°) on the side the tail steps out, plus a mini 7-seg readout |
| Z2 | ghost-888 speed | 7-seg cap 120 at (−600, 0) |
| Z4 | bargraphs | 3 × 16 steps, phosphor cyan at alpha 140, no glow |
| Z5 | annunciators | shift, slip, abs, tcs: lit only when active |

## Materials and light
`materials.vfdCavity` (true black, mesh alpha 10, 2 px scanlines at alpha 34, cavity only), `glow.phosphor` for lit segments (tight core, short halo), ghost segments at 7%. Bezel faces ≤ 20% luminance with a light-driven 1 px rim and lit inner lip.

## Type and colour
Labels `tech` (Chakra Petch); digits are procedural 7-segment SDFs (`type.fonts.segment`), slanted 8°. Roles: `primary` = phosphor cyan, `secondary` = caution yellow, `accent` = magenta (shift and limiter only). Magenta is reserved for the strip, the top tach zone and the gear bay at the limiter.

## Motion and signature moments
- **Signature:** the **sector boot**: the cavity powers up sector by sector (≤ 1 s, skippable), gear and speed live from frame 1.
- **Gear change:** segments of the new digit light in sequence over 80 ms; the bay rim flashes once.
- **Limiter:** only the 28 px strip strobes; the gear bay turns steady magenta.
- **Glitch:** event-only (80–120 ms RGB split on a crash or a missed shift), never on the hero gear.

## Do / Don't
- Do keep everything outside the cavity off.
- Don't hue-code bargraphs; don't glow ranks 4–5.

## Phase 6 assets
VFD phosphor mask and mesh, scanline mask, hologram grid sheets, warp/energy plates for FUI members, chromatic prism plates (Shibuya), accretion-disk plates (ChronoEventHorizon).

## References
Cyberpunk 2077 dashes (squared 1980s figures), Oblivion (dot grid, rationed greeble), Tron: Legacy (procedural, sector boot), Blade Runner 2049 (3-state screens, stepped transitions), The Expanse (colour as faction, no flashing boxes) (RESEARCH.md §5).
