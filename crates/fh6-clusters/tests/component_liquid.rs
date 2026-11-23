mod support;

// Visual shot test for the Fluid family's liquid parts: the `liquidChannel` tach
// (`LiquidChannel`) around the gear well and the tilting mercury pool (`LiquidSurface`) inside it,
// settled on idle, a right-hand corner at cruise (lateral G tilts the pool and the meniscus) and
// the limiter (droplets detach from the meniscus and hop off the pool, the redline end glows).
// A fourth column repeats the limiter over the Phase 6 `mercury_pool_beauty_8x8` flipbook when
// the asset is present. The shots land in `target/component-shots/liquid_*.png`; the pixel
// checks pin the fill extent, the tilt direction, the limiter glow and the droplets.
//
// Signals are built directly (rpm, limiter, shift rpm, the derived fractions and the smoothed
// G): the same fields `SignalSmoother` fills from the verified packet fields.

use fh6_clusters::common::liquid::{ChannelGeometry, LiquidChannel, LiquidSurface, PoolGeometry, Underlay};
use fh6_clusters::common::VirtualLight;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Layout, Palette, Rect, Signals, TextStyle, Tokens, VAlign, Vec2};

/// Frames each state is settled for.
const FRAMES: usize = 30;
/// Limiter rpm of the test car.
const LIMITER: f32 = 8000.0;
/// Idle rpm.
const IDLE: f32 = 900.0;
/// Family palette.
const FAMILY: &str = "fluidGenerative";

/// One review state: caption, rpm, lateral and longitudinal G.
#[derive(Debug, Clone, Copy)]
struct State {
    name: &'static str,
    rpm: f32,
    g_lat: f32,
    g_long: f32,
}

/// Idle, a right-hand corner at cruise, the limiter.
const STATES: [State; 3] = [
    State {
        name: "IDLE",
        rpm: IDLE,
        g_lat: 0.0,
        g_long: 0.0,
    },
    State {
        name: "CRUISE  0.6 G RIGHT",
        rpm: 4600.0,
        g_lat: 0.6,
        g_long: 0.1,
    },
    State {
        name: "LIMITER",
        rpm: LIMITER,
        g_lat: 0.0,
        g_long: 0.0,
    },
];

/// Signals for a steady state (no climb, so no feed-forward lead).
fn signals(tokens: &Tokens, s: State) -> Signals {
    let shift = LIMITER * tokens.rev_stages.shift_pct;
    Signals {
        race_on: true,
        rpm: s.rpm,
        max_rpm: LIMITER,
        idle_rpm: IDLE,
        shift_rpm: shift,
        rpm_fraction: (s.rpm / LIMITER).clamp(0.0, 1.0),
        shift_fraction: s.rpm / shift,
        g_lat: s.g_lat,
        g_long: s.g_long,
        ..Signals::default()
    }
}

/// The three liquid parts settled on one state.
struct Settled {
    channel: LiquidChannel,
    pool: LiquidSurface,
    light: VirtualLight,
}

fn settle(tokens: &Tokens, s: State) -> Settled {
    let mut channel = LiquidChannel::new(tokens);
    let mut pool = LiquidSurface::new(tokens);
    let mut light = VirtualLight::new(tokens);
    let sig = signals(tokens, s);
    for _ in 0..FRAMES {
        channel.update(&sig, support::DT);
        pool.update(&sig, support::DT);
        light.update(sig.g_lat, support::DT);
    }
    Settled { channel, pool, light }
}

/// Local (anchor-centred) to shot pixels at `scale` target px per local unit.
fn to_px(p: Vec2, scale: f32) -> (i64, i64) {
    (
        (p.x * scale + support::W as f32 * 0.5).round() as i64,
        (p.y * scale + support::H as f32 * 0.5).round() as i64,
    )
}

/// Bright-pixel count and the mean colour of bright pixels in a local rect.
fn lit(img: &RgbaImage, r: Rect, scale: f32) -> (u32, [f32; 3]) {
    let (x0, y0) = to_px(r.min, scale);
    let (x1, y1) = to_px(r.max, scale);
    let mut n = 0u32;
    let mut sum = [0.0f32; 3];
    for y in y0.max(0)..y1.min(i64::from(img.height)) {
        for x in x0.max(0)..x1.min(i64::from(img.width)) {
            let i = ((y as u32 * img.width + x as u32) * 4) as usize;
            let px = &img.pixels[i..i + 4];
            let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
            if a > 40 && u32::from(r) + u32::from(g) + u32::from(b) > 150 {
                n += 1;
                sum[0] += f32::from(r);
                sum[1] += f32::from(g);
                sum[2] += f32::from(b);
            }
        }
    }
    let k = 1.0 / n.max(1) as f32;
    (n, [sum[0] * k, sum[1] * k, sum[2] * k])
}

/// Non-black pixel count (any channel above 12) in a local rect.
fn coverage(img: &RgbaImage, r: Rect, scale: f32) -> u32 {
    let (x0, y0) = to_px(r.min, scale);
    let (x1, y1) = to_px(r.max, scale);
    let mut n = 0u32;
    for y in y0.max(0)..y1.min(i64::from(img.height)) {
        for x in x0.max(0)..x1.min(i64::from(img.width)) {
            let i = ((y as u32 * img.width + x as u32) * 4) as usize;
            if img.pixels[i..i + 3].iter().any(|&c| c > 12) {
                n += 1;
            }
        }
    }
    n
}

/// The local y of the first bright pixel scanning down column `x` from `from_y` to `to_y` (the
/// pool's surface line: kicker and softbox), if any.
fn surface_y(img: &RgbaImage, x: f32, from_y: f32, to_y: f32, scale: f32) -> Option<f32> {
    let (px, y0) = to_px(vec2(x, from_y), scale);
    let (_, y1) = to_px(vec2(x, to_y), scale);
    if px < 0 || px >= i64::from(img.width) {
        return None;
    }
    (y0.max(0)..y1.min(i64::from(img.height))).find_map(|y| {
        let i = ((y as u32 * img.width + px as u32) * 4) as usize;
        let sum: u32 = img.pixels[i..i + 3].iter().map(|&c| u32::from(c)).sum();
        (sum > 150).then(|| (y as f32 - support::H as f32 * 0.5) / scale)
    })
}

/// A state caption in the label ink.
fn caption(p: &mut fh6_kit::Painter<'_>, at: Vec2, text: &str, cap: f32, palette: &Palette) {
    let font = p.fonts().role(FontRole::Text);
    let tracking = p.tokens().type_scale.label_tracking;
    let style = TextStyle::new(font, cap)
        .paint(palette.roles.ink_label)
        .tracking(tracking)
        .align(Align::Center, VAlign::CapMiddle);
    p.text(at, text, &style);
}

/// The family composite (the pool in the gear well, the channel around it) at `size` x the
/// KineticLiquidMetal geometry around `center`.
fn geometry(center: Vec2, size: f32) -> (ChannelGeometry, PoolGeometry) {
    let fam = ChannelGeometry::fluid_family();
    let channel = ChannelGeometry::new(center, fam.inner * size, fam.outer * size).with_column(fam.column * size);
    // Gear well R 152, rest level well centre + 96 (family spec Z1/Z4).
    let pool = PoolGeometry::well(center, 152.0 * size, 96.0 * size);
    (channel, pool)
}

/// Draws one composite: pool, channel and its scale.
fn draw_composite(
    p: &mut fh6_kit::Painter<'_>,
    part: &Settled,
    center: Vec2,
    size: f32,
    palette: &Palette,
    underlay: Option<&Underlay>,
) {
    let (channel, pool) = geometry(center, size);
    let light = part.light.angle();
    part.pool.draw(p, &pool, palette, light, underlay);
    let clip = Rect::from_center_size(center, Vec2::splat(channel.outer * 2.0 + 240.0 * size));
    part.channel.draw(p, &channel, clip, palette, light, underlay);
    let font = p.fonts().role(FontRole::Fluid);
    let cap = p.tokens().type_scale.micro_label * 1.4 * size;
    part.channel.draw_scale(p, &channel, font, cap, palette);
}

// ---------------------------------------------------------------------------------------------
// The states side by side (4K authoring units on a half-resolution target).

/// Column centres (local 4K px) and the composite's scale over the family geometry.
const COLUMNS: [f32; 4] = [-1440.0, -480.0, 480.0, 1440.0];
const STATE_Y: f32 = 80.0;
const SIZE: f32 = 1.55;
/// Half-resolution shot: target px per local unit.
const HALF: f32 = 0.5;

#[test]
fn liquid_states() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, FAMILY);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();

    let parts: Vec<Settled> = STATES.iter().map(|&s| settle(&tokens, s)).collect();
    let limiter_flip = settle(&tokens, STATES[2]);
    // The Phase 6 mercury flipbook, when the asset pack is present (the parts never need it).
    let atlas = gpu.assets.join("fluid/mercury_pool_beauty_8x8.png");
    let underlay = if support::exists(&atlas) {
        let g = &mut *gpu;
        let id = g.kit.images_mut().load_file(&g.ctx.device, &g.ctx.queue, &atlas, None);
        id.ok().map(|id| Underlay::flipbook(id, 8, 8).with_opacity(0.45))
    } else {
        None
    };

    // Telemetry state of each part.
    let [idle, cruise, limiter] = [&parts[0], &parts[1], &parts[2]];
    assert!(idle.pool.tilt().abs() < 1e-4 && idle.channel.tilt().abs() < 1e-4, "idle is level");
    assert!(cruise.pool.tilt() > 0.1, "a right turn raises the left side: {}", cruise.pool.tilt());
    assert!(cruise.channel.tilt() > 0.1, "the meniscus leans with the pool");
    assert!(limiter.channel.limiter() > 0.95, "hot glow at the limiter");
    assert!(limiter.channel.droplet_count() > 0, "droplets leave the meniscus");
    assert!(limiter.pool.droplet_count() > 0, "droplets hop off the pool");
    assert!(idle.channel.droplet_count() == 0 && idle.pool.droplet_count() == 0);
    assert!(!idle.pool.animating(), "the idle pool never moved");
    // Idle comes to rest: two more seconds of idle and nothing animates.
    let (mut ch, mut pool) = (idle.channel, idle.pool);
    let sig = signals(&tokens, STATES[0]);
    for _ in 0..120 {
        ch.update(&sig, support::DT);
        pool.update(&sig, support::DT);
    }
    assert!(!ch.animating() && !pool.animating(), "idle is still");

    let img = gpu.render(&layout, |p, _| {
        let names = STATES
            .iter()
            .map(|s| s.name)
            .chain(std::iter::once(if underlay.is_some() {
                "LIMITER  FLIPBOOK UNDERLAY"
            } else {
                "LIMITER  (NO FLIPBOOK ASSET)"
            }));
        let all = parts.iter().chain(std::iter::once(&limiter_flip));
        for (i, ((name, part), cx)) in names.zip(all).zip(COLUMNS).enumerate() {
            caption(p, vec2(cx, -620.0), name, 36.0, &palette);
            let u = if i == 3 { underlay.as_ref() } else { None };
            draw_composite(p, part, vec2(cx, STATE_Y), SIZE, &palette, u);
        }
    });
    let path = support::save_shot("liquid_states", &img);
    assert!(path.exists(), "{}", path.display());

    // The column at rpm fraction `f` of column `col`: a box across the whole bore.
    let probe = |col: usize, f: f32| {
        let (ch, _) = geometry(vec2(COLUMNS[col], STATE_Y), SIZE);
        let at = Vec2::polar(ch.center, ch.radius(), parts[col.min(2)].channel.angle_at(f));
        Rect::from_center_size(at, Vec2::splat(ch.half_bore() * 1.6))
    };
    // Cruise: the column fills to 57.5%: liquid (non-black) at 30%, empty glass at 85%.
    let filled = coverage(&img, probe(1, 0.3), HALF);
    let empty = coverage(&img, probe(1, 0.85), HALF);
    assert!(
        filled > 3 * empty.max(1),
        "cruise column filled at 30% ({filled} px) and empty at 85% ({empty} px)"
    );
    // Idle: a bead rests in the closed end; the tube is empty at 50%.
    let idle_mid = coverage(&img, probe(0, 0.5), HALF);
    assert!(filled > 3 * idle_mid.max(1), "idle tube empty at 50% ({idle_mid} px)");
    // Limiter: the redline end glows in `shift` (orange: red well above blue).
    let (n, [r, g, b]) = lit(&img, probe(2, 0.96), HALF);
    assert!(n > 4 && r > b + 80.0 && r > g, "limiter end glows hot ({n} px, {r} {g} {b})");
    let (n, [r, _, b]) = lit(&img, probe(1, 0.96), HALF);
    assert!(n == 0 || b + 40.0 > r, "no hot glow off the limiter ({n} px, r {r} b {b})");

    // Cruise pool: the surface line rises on the left in a right turn.
    let (_, pool) = geometry(vec2(COLUMNS[1], STATE_Y), SIZE);
    let half = pool.rect.width() * 0.5;
    let (top, bottom) = (pool.rect.min.y + 0.1 * half, pool.rect.max.y);
    let left = surface_y(&img, COLUMNS[1] - 0.45 * half, top, bottom, HALF);
    let right = surface_y(&img, COLUMNS[1] + 0.45 * half, top, bottom, HALF);
    match (left, right) {
        (Some(l), Some(r)) => assert!(l + 20.0 < r, "left surface above the right: {l} vs {r}"),
        other => panic!("pool surface not found: {other:?}"),
    }
    // Idle pool: level (both sides within a few px).
    let (_, pool) = geometry(vec2(COLUMNS[0], STATE_Y), SIZE);
    let half = pool.rect.width() * 0.5;
    let (top, bottom) = (pool.rect.min.y + 0.1 * half, pool.rect.max.y);
    let left = surface_y(&img, COLUMNS[0] - 0.45 * half, top, bottom, HALF);
    let right = surface_y(&img, COLUMNS[0] + 0.45 * half, top, bottom, HALF);
    match (left, right) {
        (Some(l), Some(r)) => assert!((l - r).abs() < 6.0, "idle surface level: {l} vs {r}"),
        other => panic!("idle pool surface not found: {other:?}"),
    }

    // The family rule: thin streaks, not chrome slabs. Near-white stays under 15% of the
    // lit pixels in each procedural composite.
    for (col, cx) in COLUMNS.iter().take(3).enumerate() {
        let r = Rect::from_center_size(vec2(*cx, STATE_Y), Vec2::splat(900.0));
        let (x0, y0) = to_px(r.min, HALF);
        let (x1, y1) = to_px(r.max, HALF);
        let (mut on, mut white) = (0u32, 0u32);
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y as u32 * img.width + x as u32) * 4) as usize;
                let px = &img.pixels[i..i + 3];
                if px.iter().any(|&c| c > 12) {
                    on += 1;
                }
                if px.iter().all(|&c| c > 225) {
                    white += 1;
                }
            }
        }
        let share = white as f32 / on.max(1) as f32;
        assert!(share < 0.15, "column {col}: near-white {:.1}% of lit pixels", share * 100.0);
    }
}

// ---------------------------------------------------------------------------------------------
// True size: one local unit = one target pixel (the 4K panel at scale 1.0), at the family
// geometry, for detail review.

const TRUE_COLUMNS: [f32; 3] = [-620.0, 0.0, 620.0];
const TRUE_Y: f32 = 40.0;

#[test]
fn liquid_true_size() {
    let mut gpu = support::gpu();
    let center = vec2(support::W as f32 * 0.5, support::H as f32 * 0.5);
    let layout = Layout::centered(center, 1.0, (support::W, support::H));
    let palette = Palette::family(&gpu.tokens, FAMILY);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let parts: Vec<Settled> = STATES.iter().map(|&s| settle(&tokens, s)).collect();

    let img = gpu.render(&layout, |p, _| {
        for ((state, part), cx) in STATES.iter().zip(&parts).zip(TRUE_COLUMNS) {
            caption(p, vec2(cx, -400.0), state.name, 18.0, &palette);
            draw_composite(p, part, vec2(cx, TRUE_Y), 1.0, &palette, None);
        }
    });
    let path = support::save_shot("liquid_true_size", &img);
    assert!(path.exists(), "{}", path.display());

    // Every composite draws, and the liquid stays inside its envelope (the clip rect).
    for (cx, part) in TRUE_COLUMNS.iter().zip(&parts) {
        let inside = Rect::from_center_size(vec2(*cx, TRUE_Y), Vec2::splat(560.0));
        assert!(coverage(&img, inside, 1.0) > 2_000, "composite at {cx} draws");
        // Droplets fly inside the composite.
        let (ch, _) = geometry(vec2(*cx, TRUE_Y), 1.0);
        for (c, r) in part.channel.droplets(&ch) {
            assert!(r > 0.0 && (c - ch.center).length() < ch.outer + 120.0, "droplet at {c:?}");
        }
    }
}

#[test]
fn zz_debug_droplets() {
    let tokens = support::gpu().tokens.clone();
    let part = settle(&tokens, STATES[2]);
    let ch = ChannelGeometry::fluid_family().at(Vec2::ZERO);
    for (c, r) in part.channel.droplets(&ch) {
        eprintln!("channel drop {c:?} r {r}");
    }
    eprintln!("light {}", part.light.angle().to_degrees());
}
