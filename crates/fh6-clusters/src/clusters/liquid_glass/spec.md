# Liquid Glass (`LiquidGlass`): brief

The brief of the rebuild (thread 1 of the v2 rebuild plan, 2026-11-27). It replaces the brief of the first pilot, which TJX rejected: "replace the dark slab with blobs by separate floating glass pieces that read as glass because of how they bend and catch light". The research behind it is in [RESEARCH.md](RESEARCH.md); the equations are in [docs/specs/optics.md](../../../../../docs/specs/optics.md); the board is Penpot page `07 · LiquidGlass`.

- **Family:** Liquid Glass (Design Council 3A) · **View:** cockpit and hood · **Catalog row:** 1

**The idea.** Separate pieces of clear glass float over black, one per instrument, and each is a real optical object. The light lives under the glass.

| Rank | Piece | What it shows |
|---|---|---|
| 1 | 16 cabochons on an arc above the rod | the shift lights: one light under each, filling from the outside in |
| 1 | Gear lens, centre | the gear (Saira Condensed 700) under a dome that magnifies it about 1.15 times |
| 2 | Speed capsule, left | three tabular digits and the unit, under a shallow dome |
| 3 | Rev rod with a bead | a ribbon of liquid light from idle to the limiter, with a graduation every 1000 rpm; the bead rides its head and magnifies the mark under it |
| 4 | G lens, under the gear lens | the G light: a point when the tyres grip, a ring when the car slides |
| 4 | Info capsule, right; the foot row's left | power, boost and the rpm readout: secondary data at 86% of the ink, below the speed |
| 5 | Labels, units; the foot row's right | the labels and units, and the logo or the custom name. They never change, so they stay at 19% luminance, under the tokens' 20% cap for static parts, and still 4.8:1 on black; the light is dimmed, not the coverage, because the overlay blends coverage in encoded space |

- **Glass:** synthetic sapphire (`tokens.glass.optics`), clear, no frost, no painted rim. Every piece is a dome, as a drop of liquid is: the gear lens rises 72 px over a 160 px radius, the speed capsule 30 px and the info capsule 24 px (cockpit sizes). Numerals stay where the surface tilts less than 22°, so no reflection crosses them.
- **Light rig:** one studio for every piece (a horizon ring, a key strip, a far strip), leaning to the car's primary colour as a cabin's ambient lighting does. Lateral G swings it up to 25°; acceleration leans the key light 6° per g, and the environment stays level so its dark cone stays on the viewer.
- **Signature moments:** from the shift point only the cabochon strip strobes (10 Hz), on its own clock that starts lit, so the strip never goes dark at the moment to shift, and its dark phase keeps a fifth of the light, so a still of the limiter never reads as off; on the limiter the 16 cabochons fuse into one bar and the bead turns white with a warning halo. On an upshift, once the new gear has settled (about 100 ms), a ripple runs out through the gear lens's rim and wobbles its reflections while the numeral stays still, and a lap of light runs once round the lens (both left out in reduced-flash mode). Under hard braking the capsules reach for the gear lens and fuse with it through necks of glass. On show the numerals are live from the first frame and only the glass grows in over 200 ms, each piece in place.
- **Telemetry:** rpm (current, idle, limiter) with the tokens' shift point → the ribbon, the bead and the 16 cabochons. Gear → the glyph and the upshift moment. Speed → the tabular field (0.6 hysteresis). Acceleration x and z → the light rig and the G light (felt force). Body slip from velocity → the G light's focus. Power and boost → the info capsule. The rpm, power and boost digits come straight from the telemetry and hold still until it moves by 100 rpm, 1% of the power (at least 3 hp) or 0.3 psi, so they do not flicker while the engine bounces on the limiter, and a real change shows on the next frame. All are native fields; nothing is synthesised.
- **Assets:** `studio_env.png` (the studio the glass mirrors: a Blender render with a ComfyUI panorama on its cyclorama) and `liquid_light.png` (the caustic in the ribbon, ray traced). Both are optional at run time.
- **Budget:** under 0.5 ms GPU at 4K with one glass layer; the build measures 0.34 ms.
- **References:** Apple's Liquid Glass (WWDC25 sessions 219, 323 and 356; the HIG on materials) and the CarPlay Ultra instrument cluster. See RESEARCH.md §3.1 for what is matched and what is changed.
