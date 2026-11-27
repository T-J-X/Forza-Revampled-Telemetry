# fh6-kit: the cluster author's guide

`crates/fh6-kit` is the GPU 2D/2.5D kit every FH6 WheelDash cluster is built with. A cluster gets crisp vector shapes, bloom, Liquid Glass, motorsport typography and spring motion in a few lines, and costs about 0.2–0.3 ms of GPU at 4K (`docs/perf/render-kit.md`).

Every snippet below is compiled and rendered by `crates/fh6-kit/tests/guide_examples.rs` (section numbers match), so the guide cannot drift from the code. Design decisions are in `docs/adr/0002-render-kit.md`. Complete, working examples: the Phase 7 pilot `crates/fh6-clusters/src/clusters/liquid_glass/` (the pattern every cluster follows) and the two Phase 5 showcase samples in `crates/fh6-clusters/src/kit/`.

Contents: [1 Start](#1-start-a-new-cluster) · [2 Shapes](#2-shapes) · [3 Paint](#3-paint) · [4 Light and material](#4-light-and-material) · [5 Text](#5-text) · [6 Animation](#6-animation) · [7 Effects](#7-effects) · [8 Contract, registry, layout](#8-cluster-contract-registry-layout) · [9 Palette and tokens](#9-palette-tokens-logo-zone) · [10 Check your work](#10-check-your-work) · [11 Rules](#11-rules-that-keep-the-budget) · [12 Shared components](#12-shared-components-fh6-clusterscommon) · [13 Lessons from the pilot](#13-lessons-from-the-pilot)

---

## 1. Start a new cluster

1. Read your family spec in `docs/design/families/`, `docs/design/TOKENS.md` and `docs/design/COMPONENTS.md`.
2. Every catalog cluster already has its module, `crates/fh6-clusters/src/clusters/<snake_name>/mod.rs`, drawing the Phase 7 scaffold. Replace the scaffold with your struct that implements [`Cluster`](#8-cluster-contract-registry-layout) and keep `pub fn make`. Your brief is `spec.md` beside it.
3. It is already registered (`registry::REGISTRY`, generated from `catalog::CATALOG`) and has its golden test. Do not edit those shared files.
4. Render it: `cargo run --release -p preview-export -- --cluster <Theme>` (4 states per view at 4K, crops, the cockpit composite, the 60-frame sequence and badges in `mockups/<Theme>/index.html`), then write your goldens (`FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters <snake_name>`).
5. See it live: `FH6_WheelDash --demo --cluster <Theme> --stats` (or set `"CockpitHudStyle": "<Theme>"` in `config.json`), or open the WebGPU page (`cargo run -p web-preview --bin web-preview-build`, then `--bin web-preview-serve`, http://127.0.0.1:5310/?cluster=<Theme>).

**Coordinates.** Draw in **local units = 4K pixels at user scale 1.0, around the anchor** (0, 0). The host sets the transform from the `Layout` (anchor, user scale 0.70–1.40, height offset, target size). Your cluster must fit the view's envelope: cockpit `(-720, -320, 1440, 560)`, hood `(-880, -220, 1760, 400)` (`layout.envelope`). y points **down**; angles are radians, **clockwise from 3 o'clock** (`deg(135.0)` is 7:30, the usual tach start). Use `deg()` for degrees.

**Colours.** Never write a hex value. Every colour comes from `p.palette()` (brand roles and fixed roles) or `p.tokens()`. Colours are linear and premultiplied; `with_alpha`, `fade` and `lerp` keep them correct.

## 2. Shapes

All shapes are analytic signed distances, anti-aliased by exactly one device pixel at any scale, and work with `fill`, `stroke`, `glow` and `shadow`.

```rust
let pal = *p.palette();
// Rounded rect, per-corner radii.
let pod = Rect::from_xywh(-200.0, -60.0, 400.0, 120.0);
p.fill(&Shape::round_rect(pod, Radii::new(24.0, 24.0, 8.0, 8.0)), pal.roles.ink_ghost);
// A 1 px specular rim, exactly one device pixel at any scale.
p.stroke(&Shape::round_rect(pod, 24.0), pal.roles.rim_specular, Stroke::rim_px(1.0));
// Arc: tach track 135° -> 405°, round caps.
p.fill(&Shape::arc(Vec2::ZERO, 260.0, 18.0, deg(135.0), deg(270.0), Cap::Round), pal.roles.ink_ghost);
// Tick ring: 37 ticks, every 4th major.
p.fill(
    &Shape::Ticks(TickRing::new(Vec2::ZERO, 300.0, deg(135.0), deg(270.0), 37).major(4, 34.0, 5.0)),
    pal.roles.ink,
);
// Tapered needle (tokens: 14 px base, 3 px tip) and its hub.
p.fill(&Shape::needle(Vec2::ZERO, deg(200.0), 240.0, 40.0, 14.0, 3.0), pal.primary_text);
p.fill(&Shape::circle(Vec2::ZERO, 24.0), pal.roles.ink_muted);
// Polyline and polygon (points live in your struct or a stack array: no allocation).
let pts = [vec2(-300.0, 200.0), vec2(-200.0, 150.0), vec2(-100.0, 190.0)];
p.fill(&Shape::Polyline { points: &pts, width: 6.0, closed: false }, pal.accent);
p.fill(&Shape::Polygon { points: &pts }, pal.secondary.with_alpha(0.3));
// Chevron and a segmented LED bar lit to 6.5 of 12.
p.fill(&Shape::Chevron { center: vec2(320.0, 0.0), size: vec2(40.0, 60.0), thickness: 8.0, angle: 0.0 }, pal.accent);
p.fill(
    &Shape::Bar(SegmentedBar {
        rect: Rect::from_xywh(-300.0, 240.0, 600.0, 24.0),
        count: 12, gap: 6.0, lit: 6.5, radius: 4.0,
        unlit: pal.roles.ink_ghost,
        axis: fh6_kit::Axis::Horizontal,
    }),
    pal.roles.go,
);
// Glow and drop shadow from the same shape.
let orb = Shape::circle(vec2(0.0, -200.0), 20.0);
p.shadow(&orb, Shadow::new(vec2(4.0, 6.0), 8.0, pal.roles.oled_black.with_alpha(0.6)));
p.glow(&orb, pal.accent, Glow::new(16.0, 0.6));
p.fill(&orb, pal.roles.ink);
```

| Shape | Notes |
|---|---|
| `Shape::rect`, `round_rect(rect, radii)` | `Radii::new(tl, tr, br, bl)` or one radius |
| `circle`, `ring`, `arc(center, r, thickness, start, sweep, Cap)` | negative sweeps run anticlockwise |
| `line(a, b, width)`, `needle(pivot, angle, len, tail, base_w, tip_w)`, `Shape::Line { .. }` | tapered capsules; `Cap::Butt` makes a trapezoid |
| `Shape::Polyline`, `Shape::Polygon` | even-odd fill; cost grows with point count |
| `Shape::Ticks(TickRing)` | one instance for the whole ring |
| `Shape::Chevron`, `Shape::Bar(SegmentedBar)` | a bar's lit cells take the paint **at each cell centre**, so a hard-stop gradient gives per-LED stage colours |
| `Shape::SevenSeg { rect, thickness, mask }` | `shape::seven_seg_mask('8')`; VFD digits |

Strokes: `Stroke::new(w)` (centred), `inner(w)`, `outer(w)` in local units, `Stroke::rim_px(px)` in device pixels. `p.glow(shape, paint, Glow { sigma, intensity, fill_inside })` is a Gaussian of the distance (no extra pass). `p.shadow` uses an erf edge; draw it before the shape. Use the tokens' glow lobes: `p.tokens().glow.hot.lobes()`.

State: `p.save()` / `p.restore()` or `p.scoped(|p| ..)` around `translate`, `scale`, `rotate_about`, `transform`, `set_opacity`, `set_blend`, `clip(rect, radius)`. `p.device_px()` is one device pixel in local units; `p.set_transform(Affine::IDENTITY)` draws in target pixels.

## 3. Paint

```rust
let pal = *p.palette();
let r = Rect::from_xywh(-300.0, -40.0, 600.0, 80.0);
p.fill(
    &Shape::round_rect(r, 40.0),
    Gradient::linear(r.at(0.0, 0.5), r.at(1.0, 0.5))
        .stop(0.0, pal.roles.go)
        .stop(0.85, pal.roles.caution)
        .stop(1.0, pal.roles.warning),
);
p.fill(
    &Shape::circle(Vec2::ZERO, 120.0),
    Gradient::radial(Vec2::ZERO, 120.0).stop(0.0, pal.roles.ink).stop(1.0, pal.primary.with_alpha(0.0)),
);
p.fill(
    &Shape::ring(Vec2::ZERO, 200.0, 12.0),
    Gradient::conic(Vec2::ZERO, deg(-90.0)).stop(0.0, pal.primary).stop(1.0, pal.accent),
);
let logo = Rect::from_xywh(-60.0, 100.0, 120.0, 60.0);
p.fill(&Shape::round_rect(logo, 8.0), ImagePaint::new(image, logo).tint(pal.roles.ink));
p.scoped(|p| {
    p.set_blend(BlendMode::Add);
    p.fill(&Shape::circle(vec2(150.0, 0.0), 60.0), pal.accent.with_alpha(0.3));
});
```

- Gradients: up to 8 stops, `Spread::{Pad, Repeat, Reflect}`, interpolated in linear premultiplied light. Two stops at the same offset make a hard edge. Coordinates are in the shape's local space, so gradients move with transforms.
- Images: load once in your cluster's constructor with `setup.images.load_file(device, queue, path, svg_size)` (`.png`, `.ktx2` (RGBA8 or BC7, optionally zstd), `.svg` through resvg). Images get mips; `ImageWrap::Repeat` tiles. `p.nine_slice(rect, image, insets, uv_insets, tint)` keeps corners.
- Blend modes: `Normal` (default), `Add`, `Screen`, `Multiply`. A change of blend mode or image starts a new draw call: group draws by blend and image.

## 4. Light and material

```rust
let pal = *p.palette();
let tokens_glass = p.tokens().glass(pal.glass_tint);
// Something behind the glass to lens.
p.fill(&Shape::circle(vec2(-150.0, 0.0), 200.0), pal.primary.with_alpha(0.5));
// Glass: draw it first in a backdrop layer.
p.layer(LayerOptions::glass(18.0), |p| {
    p.fill(&Shape::round_rect(Rect::from_xywh(-400.0, -150.0, 800.0, 300.0), 64.0), Glass { ..tokens_glass });
});
// Bloom is earned: only glance ranks 1–3 go in a bloom layer.
p.layer(LayerOptions::bloom(), |p| {
    p.fill(&Shape::arc(Vec2::ZERO, 120.0, 16.0, deg(135.0), deg(200.0), Cap::Round), pal.primary);
});
// A blurred UI layer.
p.layer(LayerOptions::blurred(12.0), |p| {
    p.fill(&Shape::rect(Rect::from_xywh(250.0, 50.0, 80.0, 80.0)), pal.secondary);
});
```

- **Bloom** (`LayerOptions::bloom()` or `Bloom { threshold, intensity }`): everything in the layer writes light into the emissive buffer, bloomed once per frame by a 5-level dual-filter (Kawase) pyramid. Tokens: only shift lights, gear, speed and the tach fill or needle bloom; labels, bezels, tyres and the G-ball never do. Bloom adds about 0.05–0.1 ms of GPU.
- **Glass** (`Glass`, start from `tokens.glass(palette.glass_tint)`): tint, frost (blurred backdrop + noise), self-lensing near the edge, fresnel (IOR 1.52), the 1 px rim gradient (`glass.rim.stops`), the inner rim (2 px inset), the arched crown sheen and an optional caustic image. Set `light_angle` from the virtual light (`common::VirtualLight`) so the rim follows lateral G. Glass samples the scene **as it was when its layer began**: draw it first in a `LayerOptions::glass(frost_sigma)` layer, then draw content on top. `refraction: Refraction::World` also refracts the captured game frame when the user enables it (`"GlassRefraction": true`); otherwise it falls back to your own layers.
- **Blur** (`LayerOptions::blurred(sigma)`): the layer is Gaussian-blurred (on a downsampled pyramid level) and composited.
- Layers nest. Opacity on a layer multiplies its draws.

## 5. Text

```rust
let pal = *p.palette();
let ts = p.tokens().type_scale;
let display = p.fonts().role(fh6_kit::FontRole::Display);
let label_font = p.fonts().role(fh6_kit::FontRole::Text);
// Hero gear: cap height from the tokens, centred on the cap middle.
p.text(vec2(0.0, -60.0), "4", &TextStyle::new(display, ts.gear_compact).paint(pal.roles.ink));
// Tabular speed, 3 fixed cells, ghost 8s.
let speed_style = TextStyle::new(display, ts.speed_compact).paint(pal.roles.ink).tabular();
p.number(vec2(0.0, 90.0), speed, 3, &speed_style, Some(pal.roles.ink_ghost.into()));
// Micro-label: tracked caps in the label ink.
let label = TextStyle::new(label_font, ts.micro_label)
    .paint(pal.roles.ink_label)
    .tracking(ts.label_tracking)
    .align(Align::Center, VAlign::CapTop);
p.text(vec2(0.0, 150.0), "MPH", &label);
```

- Sizes are **cap heights** (tokens `type.scale`: gearHero 200, speedHero 112, metricPrimary 44, microLabel 16, nano 12 minimum). A user font keeps the same optical size.
- Faces are bound to roles from tokens `type.fonts` (`Display` = Saira Condensed, `Text` = Titillium Web, `Dial`, `Wide`, `Tech`, `Fluid`, `Aviation`, `Light`, `Race`). The host loads `assets/fonts/` (Phase 6 pack) and the user `fonts/` folder and calls `Kit::bind_font_roles`; until the pack is installed every role is the embedded Noto Sans. A specific weight: `p.fonts().find("Saira Condensed", 700, false)`.
- `Figures::Tabular` (`.tabular()`) puts every digit in a fixed 0.60 em cell; `p.number` adds ghost 8s and never allocates. Kerning is measured once per face; `tracking` is in em; `weight` (-1..1) and `slant` are synthetic fallbacks when a face is missing.
- `p.text` returns the set bounds; `p.measure` measures without drawing. Characters not yet in the atlas appear on the next frame (generated after the frame, 16 per frame at most).

## 6. Animation

```rust
struct Needle {
    spring: Spring,
    g: Smoothed<f32>,
    shown_speed: Hysteresis,
}

impl Needle {
    fn new(tokens: &Tokens) -> Self {
        Self {
            spring: Spring::new(tokens.springs.needle, 0.0),
            g: Smoothed::new(0.0, 0.06),
            shown_speed: Hysteresis::new(0.0, 0.6),
        }
    }
    fn update(&mut self, rpm: f32, rpm_rate: f32, g_lat: f32, speed: f32, dt: f32) -> i64 {
        self.spring.update_tracking(rpm, rpm_rate, dt);
        self.g.update(g_lat, dt);
        self.shown_speed.update(speed)
    }
}
```

- `Spring` is closed-form (period `T`, damping `zeta`): exact at 60/120/144 Hz, any `dt`. Use the tokens' springs (`needle`, `ribbon`, `snap`, `fast`, `light`, `gBall`, `fluidSlosh`, ...). `update_tracking(target, target_rate, dt)` adds feed-forward so a climbing needle lags less than one 60 Hz frame.
- `Smoothed<T>` (f32, Vec2, Color) for noisy channels; `Hysteresis` for the speed digit (changes only when the value moves by 0.6); `Easing` (tokens' M3 curves, 256-entry LUT); `Tween`; `Strobe::LIMITER` (10 Hz, 50%) and `Strobe::breath` for reduced-flash mode.
- Animate only from `FrameInput::dt` and `t`, never a wall clock. Return `true` from `Cluster::animating` while something still moves without new telemetry, so the host renders between packets.
- Ready-made components with token motion: `fh6_clusters::common::{ShiftLights, GearGlyph, SpeedReadout, VirtualLight}`.

## 7. Effects

```rust
let pal = *p.palette();
emitter.update(dt, vec2(0.0, 100.0));
emitter.draw(p, Some(6.0));
let plate = Rect::from_xywh(-300.0, -200.0, 600.0, 200.0);
p.fill(&Shape::round_rect(plate, 32.0), ImagePaint::new(dye, plate));
// 2.5D road under the speed: the ADAS camera and the curvature model.
let cam = Camera25::adas(900.0, 900.0, 0.0, 120.0);
let road = RoadCurve::from_motion(0.3, 30.0, 0.05);
let mut edge = [Vec2::ZERO; 18];
for offset in [-1.8, 1.8] {
    let n = road.project_line(&cam, offset, 1.8, 44.0, &mut edge);
    p.fill(&Shape::Polyline { points: &edge[..n], width: 3.0, closed: false }, pal.roles.hairline);
}
```

- **Particles** (`particles::Emitter::new(capacity).with_config(EmitterConfig { .. })`): fixed pool, seeded, optional curl-noise advection; `draw` fills circles with an optional glow.
- **Fluid** (`fluid::Fluid::new(device, w, h, FluidConfig::default())`): GPU stable fluids (advection, vorticity, 20 Jacobi iterations). Register its dye once (`setup.images.register_view(device, fluid.dye_view(), fluid.size())`) and paint it like any image; `splat(uv, force, color, radius)` and `step(queue, encoder, dt)` or `step_submit(device, queue, dt)` each frame. 256x128 costs about 0.3 ms of GPU.
- **Noise**: `noise::{simplex2, simplex3, fbm2, curl2}` on the CPU and the same functions in WGSL (`noise::WGSL_NOISE`).
- **2.5D**: `perspective::Camera25` (eye at (0, 1.85, -3.4) m) projects world points; `RoadCurve::from_motion(yaw_rate, speed, slip)` bends the road with `kappa = omega / V` (guarded below 2 m/s).
- **Meshes** (ADAS cars): upload once (`setup.meshes.upload(device, queue, &MeshData::from_glb(bytes)?)`, or `MeshData::car_proxy(l, w, h)` until Phase 6's glTF cars), then each frame:

```rust
let cam = Camera25::adas(900.0, 900.0, 0.0, 0.0);
let viewport = Rect::from_xywh(-400.0, -200.0, 800.0, 400.0);
p.mesh(&MeshDraw {
    mesh: car,
    model: fh6_kit::math::Mat4::translation(fh6_kit::math::vec3(0.0, 0.0, 3.2)),
    view_proj: cam.view_proj(&viewport),
    eye: cam.eye,
    viewport,
    color: p.palette().primary,
    rim: Color::WHITE.with_alpha(0.6),
    light: fh6_kit::math::vec3(0.3, 1.0, -0.4),
    shading: MeshShading::Matcap,
});
```

`cam.view_proj(&viewport)` reproduces `cam.project` exactly, so cars sit on the 2D road you draw with the same camera.

## 8. Cluster contract, registry, layout

```rust
struct MyCluster {
    rpm: Spring,
    frac: f32,
}

impl Cluster for MyCluster {
    fn update(&mut self, input: &FrameInput<'_>) {
        let s = input.signals;
        self.frac = self.rpm.update_tracking(s.rpm_fraction, s.rpm_rate / s.max_rpm.max(1.0), input.dt);
    }
    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = *p.palette();
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(&Shape::arc(Vec2::ZERO, 260.0, 20.0, deg(135.0), deg(270.0 * self.frac), Cap::Round), pal.primary);
        });
    }
    fn animating(&self) -> bool {
        !self.rpm.settled(self.frac, 1e-4)
    }
}
```

- `update(&mut self, &FrameInput)`: `telemetry` (raw packet), `signals` (decoded and smoothed: speed in the user's unit, rpm and its rate, `rpm_fraction`, `shift_fraction`, `Gear`, pedals, smoothed G and slip, yaw rate, position), `dt`, `t`, `config`, `palette`, `tokens`, `fonts` (display / text / wide, with the user's font override), `logo`, `view`, `reduced_flash`.
- `draw(&self, &mut Painter, &Layout)`: record only; no state changes, no allocation.
- Each cluster module exports its constructor, and the registry row (generated once in Phase 7 from `catalog::CATALOG`, never edited by a cluster chat) points at it:

```rust
fn make_my_cluster(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(MyCluster::new(setup)) // load images, meshes and fonts here, once
}
// the row in registry::REGISTRY (already there for every catalog cluster):
ClusterEntry { theme: ClusterTheme::BoschDdu, name: "Bosch DDU", family: "raceDash", make: make_my_cluster },
```

`ClusterTheme` stays the identifier (config values). Every theme has its own row; until its cluster chat builds it, its module draws the Phase 7 scaffold (`common::scaffold`).
- `Layout`: `anchor` (target px), `scale` (target px per local unit), `height_offset`, `view` (`Cockpit`, `Hood`, `Chase`), `envelope` (local), `safe` (target px). Draw in local units; use `layout.view` to adapt (hood: wider, lower, frameless).

## 9. Palette, tokens, logo zone

- `Palette` (in `p.palette()` and `FrameInput::palette`): `primary`, `secondary`, `accent`, `shift`, their text-safe versions `primary_text` / `accent_text` (lifted to at least 7:1 on black, for digits and thin lines), `glass_tint`, the fixed `roles` (`warning`, `caution`, `go`, `ink`, `ink_muted`, `ink_label`, `ink_ghost`, `hairline`, `rim_specular`, `oled_black`) and `rev_stage(fraction)`.
- The host picks the palette from config: factory (the car's brand from `car_brands.json` by ordinal, then `DEFAULT`, then your family), custom (`ColorMode: "Custom"` + hex values), random (`"Random"`, seeded by `PaletteSeed`; harmonious, never within 25° of the warning hue, OLED-safe), or `Palette::family(tokens, "<family>")`.
- `Tokens` (`p.tokens()`): springs, easings, durations, glow lobes, bloom chain, glass, type scale, rev stages, layout anchors and envelopes, family palettes, font roles. Bound from `data/design_tokens.json` (v3.2); missing keys fall back to the v3 values.
- Logo zone: `fh6_kit::logo::draw(p, zone, &logo, &LogoStyle::new(fonts.wide))` fits the brand image with brand-safe padding and optical centring, else sets the brand name, else the custom name, in the wide face (14 characters at most, cut at a word).

## 10. Check your work

| Tool | What it gives |
|---|---|
| `cargo run --release -p preview-export -- --cluster <Theme>` | one cluster's review set in `mockups/<Theme>/`: 4 states per view at 4K, 1:1 crops, the cockpit composite over the reference photo with the scrim, the 60-frame sequence (idle → redline strobe → gear change) with GPU p50 / p99, and `index.html` with the badges |
| `cargo run --release -p preview-export` | `mockups/kit/<Theme>/<state>.png` (4K over OLED black), `<state>_crop.png` (1:1 cluster), `_thumbs/` (box-downsampled 960x540), `index.html` with measured badges: GPU ms, CPU ms, draws, instances, colour budget (base hue ≥ 80%, white ≤ 15%, red ≤ 5%), transparent outside |
| `cargo test -p fh6-clusters <snake_name>` | that cluster's golden test: every view and review state against `tests/golden/images/<Theme>[_hood]_<state>.png` (OKLab ΔE > 3 on at most 0.5% of pixels). `FH6_UPDATE_GOLDEN=1` rewrites them; review the PNGs before committing. `--test golden_images` runs all 48 |
| `cargo run --release -p bench-render -- --frames 600` | CPU and GPU per scenario at 4K |
| `FH6_WheelDash --demo --cluster <Theme> --stats` | the live overlay with synthetic telemetry, that cluster forced in every view, and latency/CPU lines |
| `web-preview` | the same cluster in a browser (WebGPU) |

## 11. Rules that keep the budget

- Budget per cluster: under 0.5 ms GPU at 4K, and keep `update` + `draw` well under 0.1 ms CPU.
- No heap allocation in `update` or `draw`: keep point arrays in your struct or on the stack, format numbers with `p.number`, clone strings only when they change (`clone_from`). The arena never grows after the first frame (tested).
- Fewer passes = less CPU in wgpu's submit (about 10 µs per pass): every `glass` layer adds a backdrop copy and three blur passes; bloom adds 9. Use one glass layer and group bloom draws.
- Group draws by blend mode and image; a change starts a new draw call.
- Strobes only on the shift strip (≤ 42 px tall), 10 Hz max; honour `reduced_flash`.
- Hide is the host's job: never draw "hidden" states yourself.

## 12. Shared components (`fh6-clusters::common`)

Every family composes these instead of re-drawing the same instrument. Each keeps its own animation state (`update`), draws with the kit (`draw`), never allocates after `new`, and takes every colour from the palette. They are shared files: a cluster chat uses them and does not edit them. If one needs a change, say so in your handover. Component shots are written by `cargo test -p fh6-clusters --test component_<group>` to `target/component-shots/`.

| Module | Main types | Use it for |
|---|---|---|
| `scaffold` | `Scaffold` | the placeholder every unbuilt catalog cluster draws (title, family, shift strip, gear, speed, rpm) |
| `glass_plate` | `squircle`, `squircle_point`, `GlassPlate` (`Regular` / `Clear` / `Tinted`), `dimming_pad`, `legibility_halo` | a Liquid Glass slab with the virtual light, the 200 ms materialise ramp, palette-driven rims; the pad under a hero glyph; the hood halo over a bright bonnet |
| `ribbon` | `PathRibbon`, `RibbonStyle` | a tapered light ribbon along an arch or a squircle rim, drawn in chunks (cheap), with a meniscus bead |
| `shift` + `mod.rs` | `ShiftLights`, `PodShape`, `PodStyle`, `draw_pod`, `LimiterCue`, `draw_rail` | outside-in shift lights (odd or even counts) as a strip, at arbitrary centres (`draw_at`) or on an arc; the 10 Hz limiter strobe; a steady limiter ramp for the non-strobing cue |
| `hero_gear` | `HeroGear` (+ `GearGlyph`) | the rank-1 gear with state colours, the 1.12 / 24 px entry, the 1-frame flash and the limiter pulse |
| `speed` | `TabularSpeed` (+ `SpeedReadout`) | 3 fixed cells, the 0.6 hysteresis, optional ghost 8s, the unit label |
| `label` | `micro_label`, `shy`, `metric`, `logo_zone` (`LogoVariant`) | rank-5 labels and the car logo / custom-name zone |
| `tach` | `TachArc`, `TachBar`, `TachChevron`, `LimiterCue`, `ChevronZones` | arc, segmented-bar and chevron tachs with the honest fill model (never shows revs the engine has not got) |
| `needle_dial` | `NeedleDial`, `DialConfig`, `DialScale`, `OperatingBand` | classic analog dials: face, numerals, needle, hub, lume, optional track mode and afterimages |
| `bezel` | `Bezel` (`BezelForm`, `BezelVariant`), `lit_rim`, `octagon` | machined and chamfered bezels lit by the virtual light |
| `adas_road` | `AdasRoad`, `AdasInput`, `curvature`, `ego_yaw` | the perspective ADAS road (κ = ω_y / V, slip, low-speed guards), lane dashes, traffic and blind-spot arcs |
| `gball` | `GBall`, `felt_force` | G meter on the `gBall` spring (felt force, not acceleration) |
| `tyre_slip` | `TyreSlipQuad`, `Corner` | 4-corner tyre slip; tyre slip is **not verified** in telemetry (see the pilot's lessons) |
| `signed_bar` | `SignedBar` | bipolar bars (G, yaw rate, pedal overlap) |
| `race` | `AlarmLamp`, `AlarmColumn`, `DataTile`, `draw_fixed` | race-dash alarm lamps and fixed-point data tiles without allocation |
| `seg7` | `Seg7Digit`, `Seg7Number`, `Seg7Geometry`, `draw_speed` | VFD / LCD seven-segment numerals with phosphor, emission and ghosting |
| `liquid` | `LiquidSurface`, `LiquidChannel`, `LiquidMetal`, `Droplets`, `Underlay` | liquid-metal pools and channels with slosh from G, meniscus and droplets; an optional Phase 6 flipbook or live fluid underlay |

**Rendered hardware (asset-forge hero library).** The components above draw live data. The physical parts under them (bezels, dial faces, hub wells, needles, crystals, housings, leather hoods, LED caps) come from `tools/asset-forge/src/hero/` as Blender renders, not as vectors with a texture tint:

- `Frame::envelope(&frame::COCKPIT)` (or `HOOD`) renders pixel-exact in local 4K px, so a plate is drawn with `ImagePaint { rect }` where `rect` is the frame's rect, at full strength (`BlendMode::Normal`, straight alpha premultiplied on load).
- Draw order: hardware plate → live vectors (ticks, numerals, digits, bars) → needle sprite (`Frame::square`, pointing up, pivot at the centre; rotate it, add a soft offset shadow) → glass layer (crystals rendered alone: reflections only).
- The kit has no normal-map relighting yet, so the plate comes as `beauty_0/1/2.png` at key azimuths −135°, −160°, −110°: crossfade them by `VirtualLight` azimuth. `normal.png`, `height.png`, `ao.png`, `rmh.png` and `emission.png` ship alongside for a future lit-sprite pass and for night glow (Add).
- Proof and reference: `cargo run --release -p asset-forge -- hero-demo` (PorscheAnalog hardware) writes `docs/handover-shots/asset-pipeline-v2-*.png`. Skills: `blender-hero-assets`, `comfyui-hero-textures`.

## 13. Lessons from the pilot

The Liquid Glass pilot (`clusters/liquid_glass/`, its `README.md` and `HANDOVER.md`) set the pattern. What it taught:

- **Module shape.** One `Geom` table per view at the top of `mod.rs` (every size and position in local 4K px), a `Faces` struct resolved once in `new()`, precomputed paths per view, and state that is only springs, smoothers and shared components. A unit test keeps the table inside `layout.envelope` and checks the clearances you care about.
- **Long paths cost GPU.** A `Shape::Polyline` evaluates every segment over its whole bounding box: one 128-point polyline round a 1240 px capsule cost 0.36 ms. Use `PathRibbon` or chunks of at most 8 segments (0.17 ms for the whole pilot afterwards).
- **Overlapping translucent segments show as dots.** Keep segmented strokes opaque (darken, do not fade alpha), or use butt caps for additive streaks.
- **Clips are rectangles with an optional corner radius.** A circle is a square clip with radius = half its side (the pilot clips the gear to its lens that way). Keep content inside other curves by geometry.
- **Bloom is for ranks 1–3, and less is more.** A Gaussian halo outside a glass body reads as a game HUD on OLED black. The critique pushed the pilot to: pipe out of the bloom, the gear blooming only on the limiter, a bead whose size follows the orb intensity law, pill glow sigma 6.
- **One hue per job.** The limiter belongs to the pill strip alone. The pipe stays in the primary hue at the limiter; only the bead goes hot.
- **Glass first means no refraction of the scene behind it.** The capsule opens the frame, so its backdrop is black and the lensing has nothing to bend. Glass reads from the rim, the fresnel band and a thin sheen, so tune those (`Glass::sheen`, `sheen_height`, `fresnel`) per cluster rather than relying on the token defaults.
- **Alphas are linear.** The kit blends in linear light, so a 3% ink fill shows as about 19% grey on screen and a 50% black overlay barely darkens. Pick small alphas for faint fills (1/255 for an unlit lens) and 0.9+ for a backing that must read black.
- **Faces rasterise on first use.** The pack has static weights plus variable Saira at 300 / 400 / 500 / 600 only (`find` returns the nearest). Resolve faces once with `setup.fonts.find(family, weight, italic).unwrap_or(role)`; a new glyph can appear a frame late (the review tools repeat the frame until no glyph is missing). If `input.fonts.display` differs from the display role seen in `new()`, the user chose a font: use it for numerals.
- **Telemetry honesty.** Tyre slip, suspension, pitch / roll, fuel, laps and torque are not verified against `tests/golden/packets`. The pilot uses body slip (`Signals::slip_deg`, from velocity) for grip and integrates speed for distance.
- **Review loop.** `preview-export --cluster <Name>` (crops, cockpit composite, 60-frame sequence, GPU p50 / p99), the goldens, then `design:design-critique` and the `ui-ux-designer` agent on the crops. Read the critique's must-fix list against the family spec; most items are about rank, glow and spacing, not new features.
