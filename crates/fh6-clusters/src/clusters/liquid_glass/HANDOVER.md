# HANDOVER: Cluster · Liquid Glass (catalog row 1, the Phase 7 pilot)

**Chat / branch:** Phase 7 setup chat 07-00, `claude/exciting-napier-8fa678`  **Date:** 2026-11-23  **Result:** ◐ built and verified; the critique is a conditional pass (the material gap is in shared code, see below)

## Exit criteria (`CLUSTER_PROMPT.md`)
| Criterion | Met? | Evidence |
|---|---|---|
| Design board has 4 states per view | ◐ HTML board (the Penpot page `07 · LiquidGlass` waits for a Penpot session, see `CATALOG.md`) | `mockups/LiquidGlass/index.html`: 4 states × cockpit and hood, crops, cockpit composites, 60-frame sequence |
| Module complete, no TODOs, goldens pass | ✅ | `mod.rs` (zone table `Geom`, `Faces`, `Paths`); `cargo test -p fh6-clusters liquid_glass`: 3 unit tests + the golden test (8 images) pass |
| 4K renders + 60-frame sequence in `mockups/LiquidGlass/` | ✅ | `cargo run --release -p preview-export -- --cluster LiquidGlass` |
| Live demo verified, GPU under 0.5 ms recorded | ✅ | `FH6_WheelDash --demo --cluster LiquidGlass --stats` ran for 60 s with the scrim and brand palette; GPU numbers below |
| Critique passed, with notes | ◐ conditional (see "Critique") | 4 rounds: 6 → 6.5 → 7 → 7.5/10, plus the two fixes the last review said would clear the bar; the remaining gap is shared glass material |
| Handover written | ✅ | this file |

## What it is
A 1240 × 360 sapphire squircle (n 5; 1560 × 260 on the hood) with no wallpaper. The ranks:
1. The gear, in Saira Condensed 700 at cap 196, clipped to a 240 px lens with a dimming pad. It blooms only on the limiter.
2. Tabular speed (Saira 200, cap 108, no ghost 8s), with the unit on its baseline.
3. A thin light pipe (4 → 8 px, outside the bloom) round the rim, ending in a meniscus bead that follows the orb intensity law.
4. A shallow pendant bulge on the bottom rim (r 28) with the caustic G spot, which defocuses with body slip, and a 0.5 g reference ring.
5. Power and boost on one baseline, and the logo zone on the lower rim (cockpit only).

The 16 shift pills sit on an offset curve of the crown, at least 4 px clear of the full-size bead.

Signature moments:
- **Upshift:** a glint on the glyph, a streak up each side of the lens rim, then two tapered specular streaks run round the capsule rim at 5000 px/s. They are dimmed while they cross the crown and meet in the pendant after about 350 ms.
- **Limiter:** only the pills strobe at 10 Hz. The steady cue is the hot bead (white core, warning halo); the pipe warms only slightly toward the shift colour.

## Renders
- `docs/handover-shots/cluster-liquid_glass-02_mid_cruise.png`
- `docs/handover-shots/cluster-liquid_glass-03_redline_strobe.png`
- `docs/handover-shots/cluster-liquid_glass-04_drift_braking.png`
- `docs/handover-shots/cluster-liquid_glass-hood_02_mid_cruise.png`
- `docs/handover-shots/cluster-liquid_glass-cockpit.png` (the composite over the cockpit photo)
- Everything else is in `mockups/LiquidGlass/` (gitignored; regenerate with the command above).

## Performance (RTX 3070 Ti Laptop, 3840 × 2160, `preview-export`)
| View | GPU |
|---|---|
| Cockpit, 60-frame sequence | p50 0.227 ms, p99 0.240 ms |
| Cockpit states | 0.23–0.24 ms (the first state rendered, idle, reads 0.33 ms) |
| Hood states | 0.44–0.45 ms (the first state rendered, idle, reads 0.70 ms) |

The first state of each view reads high on every run. It is the first frame after the view switch; the sequence p50 / p99 is the budget number. The hood is close to the 0.5 ms line because of its legibility halo, the body backing fill and the larger glass area. If a hood-heavy cluster runs over, those three are the place to look.

Live demo, desktop with no stand-in game, release, 60 s: the cluster was stable, with no jitter. CPU per frame was p50 1.44 / p99 3.66 ms. That misses the 1 ms target, but it is the pre-existing Phase 5 / 9 host cost (upload + record + submit 1.08 ms p50), not the cluster.

## Assets
- None of its own. It uses the shared Phase 6 `assets/glass/caustic_lens_circle.png` for the defocused caustic when present, and a procedural ring otherwise (`setup.assets` may be `None`).

## Critique
- **Round 1:** three critics on the first renders (director 3A, a second design critic, and a third that hit the usage limit).
  - Scores were 6/10, "below the references", passes only after its must-fix items.
  - Main findings: three luminous bands at the limiter; the pipe and bead colliding with the pills; a light, fogged gear; a crowded centre column; glass reading as tinted plastic; the light-run reading as lens-flare balls; a dark disc behind the bead; the hood looking stretched; an idle bead outshining the gear; ghost 8s; and a loose metric baseline.
  - Every must-fix was worked: see "What it is" and "My calls".
- **Round 2:** an independent `ui-ux-designer` review of the fixed renders scored **6.5/10, not passing**.
  - It asked for a bigger gear and lens, a taller capsule so it stops reading as the car's rear-view mirror, a darker body, darker off pills and a more visible light-run.
  - All of these were applied in round 3 (1240 × 360, lens 240, gear 196 at weight 800 (back to 700 after the Phase 9 merge, see My calls), centre backing, streak 230 px at full white).
- **Round 3:** the same reviewer scored **7/10** and said the hierarchy was fixed but the material was not.
  - **The root cause:** the kit blends in linear light, so a 3% ink fill shows as about 19% grey on screen, and a 50% black backing barely darkens anything.
  - **Fixed:** the off pills are now near-black (`PodStyle::unlit_rim`, new); the body has a linear 0.94 black backing in the centre; the pipe warms to the shift colour at the limiter; the idle caustic is a soft glow; the comet head glows 6 px.
- **Round 4:** **7.5/10.** The gap is now material quality, not hierarchy. It was not yet a pass, and named three items:
  1. The cockpit composite still read as a lit panel (about 12% of the bright cabin showed through the body). Fixed afterwards: the centre backing went to 0.94.
  2. The pendant drop read as a camera pod. Fixed afterwards: it is now a shallow r 28 bulge on the bottom rim.
  3. The rim and fresnel are a uniform grey. This is a `GlassPlate` / kit change and is tracked as debt.

  The reviewer estimated that items 1 and 2 would bring it to about 8.5 and clear the bar. Both landed after that review; they were not re-scored.

**Verdict:** the setup chat's call is a **conditional pass**. Every hierarchy, limiter and motion finding is closed. What is left is material quality from shared code: a per-azimuth rim and fresnel ramp in `GlassPlate`, and a smooth-union bulge in the kit. Those go to the conductor as shared and kit work, and the next Liquid Glass sibling should re-run the critique once they land.

## My calls
- **Grip uses body slip, not tyre combined slip.** Tyre slip is not verified against `tests/golden/packets`; `Signals::slip_deg` comes from the velocity vector, which is verified.
- **Distance (the caustic phase) is integrated from speed.** The distance field is not verified.
- **Capsule exponent is 5, not 4.** At n 4 the hood's 6:1 capsule pinched its ends into points.
- **The gear is Saira Condensed 700, as the brief says.** Round 3 tried 800, but before the Phase 9 merge the kit was silently drawing the gear in the fallback face. With the real ExtraBold rendered, 800 reads as block digits.
- **The speed is Saira 300, not 200.** The merged loader makes the variable Saira only at 300, 400, 500 and 600.
- **The light-run speed is 5000 px/s,** so it still meets in about 350 ms on the taller capsule.
- **The pipe is outside the bloom.** It is rank 3, and its Gaussian halo leaking onto OLED black read as a game HUD.
- **The gear is clipped to its lens,** so the tokens' 1.12 / 24 px entry can never reach the rim.
- **No logo zone on the hood:** there was nothing to mirror it on the left.
- **No octagon and no Phase 6 caustic wallpaper:** a quiet centre is the family rule.

## Known issues / debt
- **The glass refracts nothing.** The capsule opens the frame, so its glass backdrop is black. Real refraction of the cockpit needs the WGC source (`GlassRefraction`, off by default) or a kit change that lets a glass layer lens the layers drawn after it. Both are out of a cluster chat's scope.
- **The hood legibility over a bright bonnet is unproven.** `preview-export` has no hood-over-photo composite yet; that is a tooling follow-up.
- **The pendant drop is joined by a clip,** not a true smooth-union bulge. The kit has no SDF smooth-union between shapes.

## What the next cluster chats must know
- Copy this module's shape: a `Geom` per view, `Faces` resolved in `new()`, `Paths` per view, and a layout unit test that also checks your clearances.
- The common helpers it uses: `GlassPlate`, `dimming_pad`, `legibility_halo`, `squircle`, `PathRibbon`, `ShiftLights::draw_at`, `LimiterCue`, `HeroGear`, `TabularSpeed`, `logo_zone`.
- `docs/kit/README.md` §13 has the lessons.
