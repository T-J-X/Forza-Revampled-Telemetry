# Valkyrie LMP1: research (step 1)

Target `recreation`: the **Aston Martin Valkyrie** road car's driver display. Read on 2026-11-28 from the pages below (press release, road tests, the encyclopaedia entry, and the review of the maker-licensed replica wheel). Every point is marked with its source; a point marked *not established* was looked for and not found.

## The real display

The Valkyrie has no instrument binnacle. "All the vital signs [are] shown on a single OLED display screen" on the steering wheel, which carries all the switchgear (Aston Martin press release, 2017; repeated by Wards and Goodwood). The wheel is a "detachable oblong" F1-style wheel (Goodwood), in "bespoke carbonfibre" with "a screen at its centre giving you essential information" (evo, 2025). Rear view comes from cameras feeding screens at the base of each A-post (press release; evo, PistonHeads, Top Gear, Car).

What the sources say the screen shows:

| Feature | What the sources say | Source |
|---|---|---|
| Rev counter | "dominated by a huge rendered rev counter with numbers that go up to 12" | PistonHeads |
| Rev counter colour | the maker-licensed replica's "official Valkyrie dashboard interface" shows a blue tachometer, central | TweakTown (Simagic replica, image text) |
| Gear, status | the replica's screen shows the gear and ABS / traction-control indicators | TweakTown (replica) |
| Speed | "my steering wheel read-out shows 187mph": a digital speed on the wheel screen | Car |
| Shift lights | "the blue shift-lights on the steering wheel"; "the first change-up light comes on, well short of the limiter" | evo; PistonHeads |
| Shift-light layout | on the replica, above the screen: "a single light on each side flanks the central partition, which includes another eight LEDs" (ten in all) | TweakTown (replica) |
| Engine | redline 11,100 rpm (evo, Car, Wikipedia; PistonHeads says 11,200); peak power at 10,500 rpm (Wikipedia, evo body text) | as named |
| Drive modes | three: Urban, Sport and Track, on a rocker or scroll controller at the wheel's lower left | Top Gear, evo, PistonHeads, Car |
| Wheel controls | indicators, nose-lift, lights, wipers, start/stop, neutral, launch control, a KERS push button; "dials and switches sit beside the wheel screen" | Top Gear, PistonHeads, Car, Wikipedia |
| Wheel material | carbon fibre; the replica uses "contoured 3D carbon fiber", Alcantara grips and magnesium | evo; TweakTown (replica) |

*Not established:* the screen's size and resolution, the typeface, the exact rev-counter geometry (sweep, where 0 and 12 sit, the red zone), the colours of the gear and speed, whether the shift lights change colour or flash at the limiter, and the weave of the wheel's carbon. No reference picture could be fetched into the container (the replica review's site refused the proxy), so these stay open, and the dash follows the house rules where the sources are silent.

### Sources
- Aston Martin press release, 12 Jul 2017: https://media.astonmartin.com/aston-martin-valkyrie-secrets-of-exterior-and-interior-design-revealed/
- Wards, 13 Jul 2017: https://www.wardsauto.com/interiors-ux/aston-martin-applies-formula-one-formula-to-valkyrie
- Goodwood, 12 Jul 2017: https://www.goodwood.com/grr/road/news/aston-martin-valkyrie--8-things-we-now-know/
- The Drive, 2 Nov 2018: https://www.thedrive.com/news/24648/aston-martin-shows-off-the-valkyries-f1-inspired-cabin-in-new-photos
- evo road test, 2025: https://www.evo.co.uk/aston-martin/valkyrie-coupe/205739/aston-martin-valkyrie-2025-review-700-miles-in-the-worlds-most
- PistonHeads road test, 2023: https://www.pistonheads.com/news/ph-driven/2023-aston-martin-valkyrie--ph-review/46920
- Car road test, 2023: https://carmagazine.co.uk/car-reviews/aston-martin/valkyrie-hypercar
- Top Gear review and interior page, 2023: https://www.topgear.com/car-reviews/aston-martin/valkyrie and https://www.topgear.com/car-reviews/aston-martin/valkyrie/interior
- Wikipedia: https://en.wikipedia.org/wiki/Aston_Martin_Valkyrie
- TweakTown, review of the maker-licensed replica wheel: https://www.tweaktown.com/reviews/11650/simagic-aston-martin-valkyrie-steering-wheel/index.html
- Valkyrie LM (2025), "lighting and a built-in screen" on the wheel: https://www.designboom.com/technology/aston-martin-valkyrie-lm-screen-steering-wheel-racing-hypercar-06-11-2025/

## What makes it recognisable, and what the dash does

The real display is a small screen on a wheel, so by step 1's rule the dash takes its parts, not its shape: a carbon wheel face around a true-black OLED window, the ten blue shift lights in the replica's 1 + 8 + 1 grouping across the top, a big round rev counter numbered 0 to 12 that fills the screen, the gear at its centre, a digital speed, and rotary dials beside the screen.

| Recognisable feature | In the dash |
|---|---|
| Carbon F1-style wheel face, screen at its centre | Matched: a forged-carbon wheel face (Blender, on the pre-built ComfyUI forged carbon) with angled shoulders round a true-black window. The weave is not established; forged carbon follows the brief. |
| "Huge rendered rev counter" numbered to 12 | Matched: a round rev counter that fills the window, numbered in thousands from the car's own limiter (0 to 12 on an 11,100 rpm car), lit arc in `primary`. |
| Blue tachometer | Matched by the family palette: Race Dash's `primary` is blue (#00B4FF). A user palette changes it. |
| Ten blue shift lights, 1 + 8 + 1 | Matched: ten LED lens pods above the window, the eight in a centre partition and one on each shoulder. They fill outside-in in `primary` (blue) and take the `shift` role at the shift point (house rule: the shift colour is a palette role). |
| Gear and speed on the screen | Matched: the gear inside the dial's centre (rank 1), tabular speed beside the dial (rank 2). |
| ABS and TC indicators | Kept as derived lamps (from body slip and G, as BoschDdu derives them), labelled derived. |
| Rotary dials beside the screen | Matched: knurled anodised encoder rings on the shoulders (pre-built Blender knurl), carrying data arcs. |
| Urban / Sport / Track modes | Not in the telemetry. Deliberate change: no mode readout, so nothing claims a mode the game does not send. |

### Deliberate changes from the catalog brief, and why
- **The predictive lap-delta crown, the lap delta readout and the sector strip are dropped.** They need lap, distance and timing fields that `fh6-core` does not decode, and adding them is a shared-code change outside a dash thread (CLUSTER_PROMPT.md, "Edit nothing outside them"). They are also not on the real Valkyrie's screen as the sources describe it. Written in the handover as a request, not done.
- **The perimeter tach becomes the real car's round rev counter** ("numbers that go up to 12").
- **Fifteen LEDs become ten** (the replica's 1 + 8 + 1).
- **TC and ABS intervention arcs, tyre temperatures, fuel and boost** need slip ratio, tyre temperature and fuel, which are not decoded (the Valkyrie is also naturally aspirated, so boost reads nothing). The left encoder shows the pedals (throttle outer, brake inner); the right encoder shows power as a share of the car's measured peak, in `signals.power_unit`.
- **The engineer trace stays** (throttle, brake, steer over the last 4 s, 3 s in the hood view): it is the brief's rank 4 and reads only verified fields.

### House rules that outrank the real display
Rank order (gear and shift lights, then speed, then the tach), OLED black outside the hardware, palette roles instead of fixed colours, the swappable font (`race` Saira Condensed by default), the flash rule (at the limiter only the LED strip strobes at 10 Hz; the dial's red-zone rim holds a steady `shift` and never flashes), and telemetry honesty (nothing shown without a decoded or derived source).
