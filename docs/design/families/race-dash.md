# Family: Race Dash

**Pattern-setter:** ★ `BoschDdu` · **Members:** `BoschDdu`, `ValkyrieLmp1`, `Gt7ModularTripod`; new: `RackFocus`, `BionicSpaceframe` · **Hero:** [`heroes/race-dash/`](../heroes/race-dash/) · **Template:** [`race-dash-template.png`](race-dash-template.png)

Added in Phase 3 on the research's recommendation (RESEARCH.md §10): four vendors, F1 and LMDh converge on an architecture no other family owns.

## Language
An honest race display. A machined housing carries a top LED bar with recessed lens pods; a true-black screen shows a giant gear and tabular data grouped by alignment and micro-labels alone. Every lamp has a threshold, a priority and a real telemetry source; warnings outrank shift lights; nominal states are neutral and colour appears only by exception.

**Is not:** a wireframe of outlined tiles, iOS-button pods, permanent decorative alarm lamps, five saturated hues at the limiter.

## Layout grammar (cockpit)
| Zone | Element | Geometry |
|---|---|---|
| Z5 | hooded housing | 1240 wide at the top, 1180 at the bottom, top corners 120, bottom 40, carbon twill, 1 px light-driven rim |
| Z1 | 10 LED lens pods in the bar (the strip) | 88 × 30, 26 px gaps, x −557 + 114·i, y −306 |
| Z3 | segmented rev bar | 45 segments, 1000 px span at y −236, 72% of `primary`, stage colours above 7300 / 8000 rpm |
| Z4 | signed longitudinal-g bar | ±260 px at y −160, ink for accel, warning for braking |
| Z1 | giant gear | `race` 800 cap 236, centred |
| Z2 | speed (left column) | `race` 700 cap 112, tabular, ghost 8s |
| Z4 | lap data (right column), tyre quad under the gear, alarm lamps (bottom row) | hairline dividers at x ±170 fade at both ends |

## Materials and light
Carbon twill (Phase 6 texture, procedural 2×2 fallback) with a diagonal sheen; pods use the LED-lens optics in `components.shiftLights.pod` (2-stop radial, inner rim, top glint). Parallax: housing ±4, screen ±2, data 0.

## Type and colour
`race` (Saira Condensed 700/800) for every value; labels `text` 700 caps at 25–30% of value height. `primary` = rev bar; `shift` = pods at shift and the REV lamp (family default `#B27CFF`, ≥ 60° from primary). At the limiter the lit rev segments turn steady ink so only the strip and REV carry the shift colour.

## Alarm lamps (only with a source)
Left: FUEL (fuel < 10%), TC (rear slip > 0.15 with throttle > 20%), ABS (front slip with brake > 20%). Right: REV (limiter), TYRE (any tyre overheated). No lamp without a packet field behind it; fields `fh6-core` does not decode yet (fuel, lap data) must be verified against `tests/golden/packets` first.

## Motion and signature moments
- **Signature:** the outside-in fill landing on the centre pair above the gear, then the strip strobe at 10 Hz.
- **Gear change:** token motion; the gear is the only large moving element.
- **Event pages:** an event (pit, flag) may swap the rank-4 area (Bosch event pages), never the gear.

## Do / Don't
- Do align data instead of boxing it.
- Don't light green for nominal; don't keep lamps that can never light.

## Phase 6 assets
Carbon weave albedo + normal, anodised-aluminium bar, LED-lens normal, Allen-bolt sprites (optional).

## References
Bosch DDU 10/11, AiM MXG Strada, MoTeC C1xx and SLM-C priorities, Cosworth CCW Mk3, McLaren MCL40 and Porsche 963 wheels (RESEARCH.md §3).
