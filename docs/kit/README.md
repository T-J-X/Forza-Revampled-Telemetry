# fh6-kit: the cluster author's guide

`crates/fh6-kit` is the GPU 2D/2.5D kit every FH6 WheelDash cluster is built with. A cluster gets crisp vector shapes, bloom, Liquid Glass, motorsport typography and spring motion in a few lines, and costs about 0.2–0.3 ms of GPU at 4K (`docs/perf/render-kit.md`).

Every snippet below is compiled and rendered by `crates/fh6-kit/tests/guide_examples.rs` (section numbers match), so the guide cannot drift from the code. Design decisions are in `docs/adr/0002-render-kit.md`; the two showcase clusters in `crates/fh6-clusters/src/kit/` are complete, working examples.

Contents: [1 Start](#1-start-a-new-cluster) · [2 Shapes](#2-shapes) · [3 Paint](#3-paint) · [4 Light and material](#4-light-and-material) · [5 Text](#5-text) · [6 Animation](#6-animation) · [7 Effects](#7-effects) · [8 Contract, registry, layout](#8-cluster-contract-registry-layout) · [9 Palette and tokens](#9-palette-tokens-logo-zone) · [10 Check your work](#10-check-your-work) · [11 Rules](#11-rules-that-keep-the-budget)

---

## 1. Start a new cluster

1. Read your family spec in `docs/design/families/`, `docs/design/TOKENS.md` and `docs/design/COMPONENTS.md`.
2. Create `crates/fh6-clusters/src/kit/<name>.rs` with a struct that implements [`Cluster`](#8-cluster-contract-registry-layout).
3. Register it: one row in `crates/fh6-clusters/src/registry.rs` (`REGISTRY`).
4. Render it: `cargo run --release -p preview-export` (4K PNGs, thumbnails, measured badges in `mockups/index.html`), then add goldens (`FH6_UPDATE_GOLDEN=1 cargo test -p fh6-clusters --test golden_images`).
5. See it live: set `"CockpitHudStyle": "<Theme>"` in `config.json` and run `FH6_WheelDash --demo`, or open the WebGPU page (`cargo run -p web-preview --bin web-preview-build`, then `--bin web-preview-serve`, http://127.0.0.1:5310/).

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
- Register in `crates/fh6-clusters/src/registry.rs`:

```rust
fn make_my_cluster(setup: &mut Setup<'_>) -> Box<dyn Cluster> {
    Box::new(MyCluster::new(setup)) // load images, meshes and fonts here, once
}
// in REGISTRY:
ClusterEntry { theme: ClusterTheme::BoschDdu, name: "Bosch DDU", family: "raceDash", make: make_my_cluster },
```

`ClusterTheme` stays the identifier (config values). Themes without a kit implementation resolve to their family's sample.
- `Layout`: `anchor` (target px), `scale` (target px per local unit), `height_offset`, `view` (`Cockpit`, `Hood`, `Chase`), `envelope` (local), `safe` (target px). Draw in local units; use `layout.view` to adapt (hood: wider, lower, frameless).

## 9. Palette, tokens, logo zone

- `Palette` (in `p.palette()` and `FrameInput::palette`): `primary`, `secondary`, `accent`, `shift`, their text-safe versions `primary_text` / `accent_text` (lifted to at least 7:1 on black, for digits and thin lines), `glass_tint`, the fixed `roles` (`warning`, `caution`, `go`, `ink`, `ink_muted`, `ink_label`, `ink_ghost`, `hairline`, `rim_specular`, `oled_black`) and `rev_stage(fraction)`.
- The host picks the palette from config: factory (the car's brand from `car_brands.json` by ordinal, then `DEFAULT`, then your family), custom (`ColorMode: "Custom"` + hex values), random (`"Random"`, seeded by `PaletteSeed`; harmonious, never within 25° of the warning hue, OLED-safe), or `Palette::family(tokens, "<family>")`.
- `Tokens` (`p.tokens()`): springs, easings, durations, glow lobes, bloom chain, glass, type scale, rev stages, layout anchors and envelopes, family palettes, font roles. Bound from `data/figma_tokens.json` (v3.2); missing keys fall back to the v3 values.
- Logo zone: `fh6_kit::logo::draw(p, zone, &logo, &LogoStyle::new(fonts.wide))` fits the brand image with brand-safe padding and optical centring, else sets the brand name, else the custom name, in the wide face (14 characters at most, cut at a word).

## 10. Check your work

| Tool | What it gives |
|---|---|
| `cargo run --release -p preview-export` | `mockups/kit/<Theme>/<state>.png` (4K over OLED black), `<state>_crop.png` (1:1 cluster), `_thumbs/` (box-downsampled 960x540), `index.html` with measured badges: GPU ms, CPU ms, draws, instances, colour budget (base hue ≥ 80%, white ≤ 15%, red ≤ 5%), transparent outside |
| `cargo test -p fh6-clusters --test golden_images` | perceptual comparison with `tests/golden/images/` (OKLab ΔE > 3 on at most 0.5% of pixels). `FH6_UPDATE_GOLDEN=1` rewrites them; review the PNGs before committing |
| `cargo run --release -p bench-render -- --frames 600` | CPU and GPU per scenario at 4K |
| `FH6_WheelDash --demo --stats` | the live overlay with synthetic telemetry and latency/CPU lines |
| `web-preview` | the same cluster in a browser (WebGPU) |

## 11. Rules that keep the budget

- Budget per cluster: under 0.5 ms GPU at 4K, and keep `update` + `draw` well under 0.1 ms CPU.
- No heap allocation in `update` or `draw`: keep point arrays in your struct or on the stack, format numbers with `p.number`, clone strings only when they change (`clone_from`). The arena never grows after the first frame (tested).
- Fewer passes = less CPU in wgpu's submit (about 10 µs per pass): every `glass` layer adds a backdrop copy and three blur passes; bloom adds 9. Use one glass layer and group bloom draws.
- Group draws by blend mode and image; a change starts a new draw call.
- Strobes only on the shift strip (≤ 42 px tall), 10 Hz max; honour `reduced_flash`.
- Hide is the host's job: never draw "hidden" states yourself.
