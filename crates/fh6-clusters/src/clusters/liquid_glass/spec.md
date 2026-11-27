# Liquid Glass (`LiquidGlass`): brief

The brief of the rebuild (thread 1 of the v2 rebuild plan, 2026-11-27). It replaces the brief of the first pilot, which TJX rejected: "replace the dark slab with blobs by separate floating glass pieces that read as glass because of how they bend and catch light". The research behind it is in [RESEARCH.md](RESEARCH.md); the equations are in [docs/specs/optics.md](../../../../../docs/specs/optics.md); the board is Penpot page `07 · LiquidGlass`.

- **Family:** Liquid Glass (Design Council 3A) · **View:** cockpit and hood · **Catalog row:** 1

**The idea.** Separate pieces of clear glass float over black, one per instrument, and each is a real optical object. The light lives under the glass.

| Rank | Piece | What it shows |
|---|---|---|
| 1 | 16 cabochons on an arc above the rod | the shift lights: one light under each, filling from the outside in |
| 1 | Gear puck, centre | the gear (Saira Condensed 700) under its flat face, with a thin ring light |
| 2 | Speed pill, left | three tabular digits and the unit |
| 3 | Rev rod with a bead | a ribbon of liquid light from idle to the limiter; the bead rides its head |
| 4 | G lens, under the puck | the G light: a point when the tyres grip, a ring when the car slides |
| 5 | Info pill, right; foot row | power and boost; the rpm readout and the logo zone |

- **Glass:** synthetic sapphire (`tokens.glass.optics`), clear, no frost, no painted rim. Flat-faced pieces have a rounded edge about 20 to 26 px wide, so their highlights are thin lines; the small pieces are domes.
- **Light rig:** one studio for every piece (a horizon ring, a key strip, a far strip). Lateral G swings it up to 25° and acceleration leans it 6° per g.
- **Signature moments:** on the limiter only the cabochon strip strobes (10 Hz), the 16 cabochons fuse into one bar and the bead turns white with a warning halo. On an upshift the gear puck flexes and the ring light runs once round it. On show the glass grows in over 200 ms and the side pills bud out of the puck.
- **Telemetry:** rpm (current, idle, limiter) with the tokens' shift point → the ribbon, the bead and the 16 cabochons. Gear → the glyph and the upshift moment. Speed → the tabular field (0.6 hysteresis). Acceleration x and z → the light rig and the G light (felt force). Body slip from velocity → the G light's focus. Power and boost → the info pill. All are native fields; nothing is synthesised.
- **Assets:** `studio_env.png` (the studio the glass mirrors: a Blender render with a ComfyUI panorama on its cyclorama) and `liquid_light.png` (the caustic in the ribbon, ray traced). Both are optional at run time.
- **Budget:** under 0.5 ms GPU at 4K with one glass layer; the first build measures 0.34 ms.
- **References:** Apple's Liquid Glass (WWDC25 sessions 219, 323 and 356; the HIG on materials) and the CarPlay Ultra instrument cluster. See RESEARCH.md §3.1 for what is matched and what is changed.
