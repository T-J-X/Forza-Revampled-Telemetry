# GT7 Hood Pro (`Gt7HoodPro`): brief

The brief of the rebuild (thread "Rebuild Gt7HoodPro dash", 2026-11-28). It replaces the catalog brief of Phase 7's setup (the gull-wing horizon with twin ribbons, in the git history), which invented a concept the real display does not have. The rule since 2026-11-23 is that a dash named after a real display must be recognisable as that display, so this one follows Gran Turismo 7. The research is in [RESEARCH.md](RESEARCH.md); the board is Penpot page `07 · Gt7HoodPro`.

- **Family:** Modern Digital 2027 (Design Council 3B), spec [docs/design/families/modern-digital.md](../../../../../docs/design/families/modern-digital.md) · **Kind:** rebuild · ★ pattern-setter · **View:** hood · **Catalog row:** 3

**The idea.** GT7 shows a compact meter in its Bonnet View, the view our hood camera matches: one low band across the bottom of the screen, floating over the bonnet with no panel. This dash is that band, part for part, rebuilt as 2027 production hardware: rendered black glass, black chrome, smoked lenses, forged carbon and red enamel, lit by the cluster's virtual light.

**The parts** (each its own zone round the hood anchor; the numbers are `geom::HOOD`):
- **Rank 1:** the gear (cap 88) right of a thin divider on the centre line; the rev bar's lit run (95 LEDs over 760 px) with GT7's rev alert; the red suggested-gear tile beside the gear.
- **Rank 2:** the tabular speed (cap 58), right-aligned to the divider, with its unit under it.
- **Rank 3:** the rev bar as a tach: where the head LED is.
- **Rank 4:** the steering dot above the bar, the brake and throttle bars either side, the tyre diagram (load, by weight transfer), the boost gauge with the trip box under it, and the slide and limiter telltales.
- **Rank 5:** the unit, the label, the divider, and the car's logo or the custom name at the right end.

**Signature moments.**
- *The rev alert.* From the warning stage the whole lit run takes the `shift` colour at once. At the shift point the run strobes (the tokens' limiter strobe, on the strip only) and the red tile pops up naming the next gear.
- *The hand-over.* When the driver takes the suggested gear, the tile brightens for one frame and folds shut towards the gear in 100 ms, while the new gear rises with the tokens' gear motion.

**Telemetry.** rpm (current and limiter) and the tokens' shift point: the LED run, the alert, the strobe, the tile and the limiter lamp. Gear: the gear and the tile's digit. Speed: the tabular speed and the trip. Throttle and brake: the pedal bars. Steer: the dot. Acceleration: the tyre loads and the virtual light. Velocity: body slip, for the slide lamp. Boost: the gauge. Nothing GT7 shows that the verified telemetry lacks is invented: no tyre heat or wear, fuel, assists or braking suggestion.

**Where it leaves GT7 on purpose** (RESEARCH.md, "Deliberate changes"): the suggested gear is an upshift prompt at the shift point, not the driving line's downshift; two telltales, not two grids; the band is 1760 px wide, not 1916; the steering dot is not red; and the car's logo sits where GT7 shows its multi-function display.

**Risk.** Over a bright bonnet or sky, white digits can lose their edge. Every glyph, the divider and the logo carry a black keyline, as GT7's digits carry a dark edge; there is no dimming pad or box. Perf: three sprites of 1760 × 240 px cross-faded and up to about 140 SDF shapes, 95 of them the LEDs, whose lit run is the only bloom layer; the budget is 0.5 ms of GPU time at 4K.
