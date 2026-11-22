# CRITIQUE: hero concepts, three rounds

Every hero frame went through `design:design-critique` and the `ui-ux-designer` agent, measured against the bar in the MASTER_PROMPT: *clearly better than the references, not merely as good*, with the glance rank order, the 150 ms target and the OLED rules. Critics read 1:1 crops of the 4K frames, the v1 thumbnails, RESEARCH.md §8/§12 and the tokens.

**Bottom line (honest):** all seven heroes clearly beat v1 in every round, and three rounds of fixes closed most structural problems (rank order, OLED black, strobe scope, colour budget, flat boxes). No critic has yet rated any hero "clearly better than the references". The final assessment is **"matches the references"** (7–7.5/10): the composition, hierarchy and behaviour now stand with GT7, Porsche, Bosch and Tesla, while the remaining gap is mostly material and light quality that SVG concept frames cannot show (true lensing, the bloom chain, live fluid, glTF cars). The "beats references" bar therefore moves to each Phase 7 cluster chat, judged on GPU renders; this file lists what each must close.

## Round 1 (both critics, 13 of 14 passes)
Scores out of 10. P1 = blocks the bar, P2 = visible weakness. The design-critique pass for Classic Analog was cut off by a usage limit.

| Hero | design-critique | ui-ux-designer | Verdict (both) | Beats v1 | Rank order | OLED rules | Main P1s |
|---|---|---|---|---|---|---|---|
| Liquid Glass | 5.5 | 5 | below references | yes | no | no | full-plate grey caustic "marble", ribbon owned the crown, thin speed failing the blur gate, flat octagon "stop sign", three glowing outlines at the limiter, static light |
| Modern Digital | 5 | 5 | below references | yes | no | no | SimHub-template look, no signature, candy tyre pills, app-icon gear tile, strobe and bloom scope, dim shift colour, hood legibility unproven |
| Classic Analog | — | 5.5 | below references | yes | no | no | navy dial faces, red as structure, crowded speed satellite, flat blade needle, flat bright bezels |
| Race Dash | 4 | 5 | below references | yes | no | yes | wireframe of outlined tiles, iOS-button pods, five hues at the limiter, panel outside the scrim dome, decorative alarm lamps |
| Sci-Fi | 5.5 | 5 | below references | yes | no | no | speed as big as the gear, DRIFT text colliding with the bay, hue-coded glowing bargraphs, flat framed rectangle, unstaged strip |
| Fluid / Generative | 5 | 6 | below references | yes | no | no | pool crossing the gear, weak light speed, satin-pipe "chrome", static bright torus, beads too short a span |
| ADAS & HUD | 4 | 5 | below references | yes | no | no | camera hid the road behind the ego, 1990s flat-shaded boxes, traffic larger than the ego, lit navy road, small gear, strobe halo |

**Applied after round 1** (all seven heroes): one virtual light per cluster moving with lateral G on every rim, facet and crescent; 0/2/4 px parallax layers; dimming pads and glyph keylines; strobes confined to the shift strip; only ranks 1–3 glow; unlit lights neutral; throttle in ink, red only for brake and alarms; gear/speed ratios brought into the token range. Per hero: edge-band caustics and a U light pipe (Glass); the "Apex lock" twin wings fusing at the shift point and a glass pod (Modern); true-black faces, dauphine needle with lume, flat jewel visor, quiet bezels (Classic); hooded housing, recessed pods, no tile boxes, sourced lamps (Race); staged strip, bigger bay, drift columns, quiet bargraphs, 10 px lit bezel (Sci-Fi); dark-studio chrome, knocked-out gear, flat bead strip (Fluid); raised camera, no lit ground, shaded opaque cars, nose into the bend (ADAS). Tokens moved to 3.2.0 with these rules.

## Round 2 (design-critique; 2 of 7 completed, the rest cut off by a usage limit)
| Hero | Score | Verdict | Round-1 P1s | Remaining P1s |
|---|---|---|---|---|
| Race Dash | 6 | below references | 4 fixed, 4 partial | rim too faint and lost in drift; pods still glossy buttons, no machined bar; invisible cavity lip, no cover glass, checker "carbon"; gear 42% of the panel and far below the strip; red at 18% of lit area in drift |
| Sci-Fi | 6.5 | **matches references** | 3 fixed, 3 partial | no depth stack (flat lensless rectangle); neutral rendered as a 7-seg Π |

**Applied after round 2:** Race Dash — per-tow lit twill, an anodised LED bar with fresnel lens pods and crescent glints, a lit cavity lip and cover-glass crescent, gear raised to 47% of the panel, ink g-bar and tyre numerals, fixed-cell lap times, quiet idle, labels at microLabel. Sci-Fi — a shouldered visor housing the strip, light-shaded facets, a smoked cover lens, a starburst gear cell so N reads, white gear at the limiter, strip and staircase on the same thresholds, scanlines masked to lit phosphor, inactive annunciators off.

## Round 3 (final, lead critique with `design:design-critique`)
The 7-agent final round was cut off by the usage limit twice, so the Phase 3 lead ran the final pass on 2×2 state sheets of the current frames, against the round-1/round-2 findings. It can be re-run as a workflow later (`fh6-phase3-critique-final`, resumable).

| Hero | Score | Verdict | What now works | What the Phase 7 chat must still close (P1 → P2) |
|---|---|---|---|---|
| Liquid Glass | 7 | matches references | quiet centre; pills own the crown; U light pipe with a scaled meniscus; violet strip and hot meniscus at the limiter; light and sheen move with G | GPU lensing of our own layers so the glass reads as glass, not tint; prism must refract (dispersion) rather than shade; caustic band reads as smoke at small sizes → use the Phase 6 caustic texture |
| Classic Analog | 7.5 | matches references | true-black faces; dauphine facet swap and lume; clear hub gear; quiet track-pointer speed; steady red wash at the limiter | real anisotropic titanium and sapphire normals (Phase 6); the jewel visor floats ~40 px above the bezel → tie it to the bezel or the scrim lip; Track mode needs per-car shift data |
| Modern Digital | 7 | matches references | Apex lock reads instantly; diamonds strobe alone; tyres alarm by exception; keylines make every glyph hood-proof | pod still reads partly as a tile on black → prove it over a bright hood frame with the kit's glass; comet heads at idle could rest dimmer |
| Race Dash | 7 | matches references | hardware LED bar with lens pods; giant gear; colour only by exception; fits the scrim dome | carbon needs the Phase 6 normal map (pattern reads coarse); verify fuel and lap fields in the packet before shipping their lamps and tiles |
| Sci-Fi | 7 | matches references | staged strip; N/R starburst; drift columns are unique; stepped visor breaks the rectangle; off pixels stay off | the sector boot and event glitch (the family signature) exist only in the spec → show them in motion; gear bay flare on gear change |
| Fluid / Generative | 6.5 | matches references (lower end) | dark mercury with a softbox streak; gear knocked out of the pool; bold speed; beads clear of the numerals | the column must read as liquid in motion: live smooth-union surface, slosh and droplets from the GPU fluid; mercury is very dark at mid rpm → raise the specular streak, not the base |
| ADAS & HUD | 6.5 | matches references (lower end) | the road ahead is visible; semantic colours; nose into the bend; ghost blind-spot car; frameless big gear | low-poly glTF cars with wrap lighting and fresnel (Phase 6) to replace the box stand-ins; ego reads as a bright box → matte albedo and baked AO |

## Rules the critiques added to the system
Recorded in tokens 3.2.0 and TOKENS.md: shift colour lifted to `#978DFC` (≥ 7:1); Classic primary `#FF7A1A` (≥ 25° from warning); Race Dash shift `#B27CFF` (≥ 60° from primary); ghost 8s by family; glass caustic masked to a rim band and frozen at idle; glyph keylines; ADAS eye (0, 3.4, −6.0) m; traffic and blind-spot rules; Z2 distance 460 px for analog satellites.

## Sources
- Round 1: 13 structured critiques (two critics per hero) from the `fh6-phase3-hero-critique` workflow.
- Round 2: `design:design-critique` agents for Race Dash and Sci-Fi.
- Round 3: lead critique on the final frames in `docs/design/heroes/`.
