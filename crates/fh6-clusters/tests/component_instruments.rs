mod support;

// Visual shot test for the rank-4/5 instruments: GBall, TyreSlipQuad, SignedBar, AlarmColumn (side
// columns and the bottom row) and DataTile, driven through the four review states (idle, cruise,
// redline, drift-braking) side by side, in the Race Dash look (colour by exception: ink accel bar,
// neutral nominal tyres) and the Modern Digital look (full ramps).
//
// Signals come from the review telemetry through the kit's `SignalSmoother` (the same path the
// host uses). Tyre slip and temperatures are not in the golden packets, so the shot feeds
// synthetic per-state values, as a cluster would after verifying the real fields.
//
// Shots: `target/component-shots/instruments_<family>.png` (the four states at half 4K) and
// `instruments_<family>_detail_{top,bottom}.png` (redline and drift at 1:1 4K pixels).

use fh6_clusters::common::gball::GBall;
use fh6_clusters::common::race::{
    abs_condition, rev_condition, tc_condition, tyre_condition, AlarmColumn, AlarmLamp, AlarmSide, DataTile,
    TileFormat, TileFrame,
};
use fh6_clusters::common::signed_bar::{AccelTone, SignedBar, SignedSource};
use fh6_clusters::common::tyre_slip::{TyreFill, TyreSlipQuad};
use fh6_clusters::common::GearGlyph;
use fh6_clusters::review::STATES;
use fh6_core::AppConfig;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{
    vec2, Align, FontId, FontRole, Gradient, Layout, Painter, Palette, Rect, Shape, SignalSmoother, TextStyle, Tokens,
    VAlign, Vec2,
};

/// Frames each state is settled for.
const FRAMES: usize = 30;
/// Column centres (local 4K px) for idle, cruise, redline, drift.
const COLUMNS: [f32; 4] = [-1440.0, -480.0, 480.0, 1440.0];
/// Gear cap (Race Dash `race` 800 at 236).
const GEAR_CAP: f32 = 236.0;
/// Signed bar centre y and half width (race-dash.md: ±260 px).
const BAR_Y: f32 = -905.0;
const BAR_HALF: f32 = 260.0;
/// Gear centre y.
const GEAR_Y: f32 = -745.0;
/// G-ball centre y and outer-ring radius.
const GBALL_Y: f32 = -350.0;
const GBALL_R: f32 = 200.0;
/// Tyre quad footprint (relative to the column centre).
const QUAD: (f32, f32, f32, f32) = (-130.0, 0.0, 260.0, 360.0);
/// Alarm columns: top y, height, width, and the outer edge offset from the column centre.
const LAMPS_Y: f32 = 470.0;
const LAMPS_H: f32 = 520.0;
const LAMPS_W: f32 = 200.0;
const LAMPS_OUTER: f32 = 440.0;
/// Data tiles: top y, half width and value cap.
const TILE_Y: f32 = 480.0;
const TILE_HALF: f32 = 170.0;
const TILE_CAP: f32 = 64.0;
/// Bottom alarm row: top y and half width.
const ROW_Y: f32 = 850.0;
const ROW_HALF: f32 = 350.0;

/// Synthetic tyre inputs per review state: slip ratios and temperatures (°C), FL FR RL RR.
const TYRES: [([f32; 4], [f32; 4]); 4] = [
    ([0.0, 0.0, 0.0, 0.0], [45.0, 46.0, 44.0, 45.0]),
    ([0.02, 0.02, 0.03, 0.03], [88.0, 90.0, 94.0, 95.0]),
    ([0.03, 0.03, 0.12, 0.11], [96.0, 97.0, 108.0, 109.0]),
    ([0.06, 0.09, 0.24, 0.27], [98.0, 104.0, 118.0, 121.0]),
];

/// The family's variants of the instruments.
#[derive(Clone, Copy)]
struct Look {
    accel: AccelTone,
    tyres: TyreFill,
}

impl Look {
    /// Race Dash colours only by exception (race-dash.md); other families use the full ramps.
    fn of(family: &str) -> Self {
        if family == "raceDash" {
            Look {
                accel: AccelTone::Ink,
                tyres: TyreFill::Exception,
            }
        } else {
            Look {
                accel: AccelTone::Go,
                tyres: TyreFill::Ramp,
            }
        }
    }
}

/// Everything one column shows, settled on one state.
struct Column {
    title: &'static str,
    gball: GBall,
    quad: TyreSlipQuad,
    bar: SignedBar,
    gear: GearGlyph,
    left: AlarmColumn,
    right: AlarmColumn,
    row: AlarmColumn,
    boost: DataTile,
    power: DataTile,
}

fn settle(tokens: &Tokens, look: Look, state_index: usize) -> Column {
    let state = &STATES[state_index];
    let data = state.telemetry();
    let cfg = AppConfig::default();
    let mut smoother = SignalSmoother::new();
    let mut c = Column {
        title: state.name,
        gball: GBall::new(tokens),
        quad: TyreSlipQuad::new(tokens).with_fill(look.tyres),
        bar: SignedBar::new(tokens, SignedSource::LongitudinalG { full_scale_g: 1.0 }).with_accel_tone(look.accel),
        gear: GearGlyph::new(tokens),
        left: AlarmColumn::new(
            tokens,
            AlarmSide::Left,
            &[AlarmLamp::FUEL, AlarmLamp::TC, AlarmLamp::ABS],
        ),
        right: AlarmColumn::new(tokens, AlarmSide::Right, &[AlarmLamp::REV, AlarmLamp::TYRE]),
        // The Race Dash cockpit row: all five lamps under the screen, the limiter labelled LIM.
        row: AlarmColumn::new(
            tokens,
            AlarmSide::Bottom,
            &[
                AlarmLamp::FUEL,
                AlarmLamp::TC,
                AlarmLamp::ABS,
                AlarmLamp::LIM,
                AlarmLamp::TYRE,
            ],
        ),
        boost: DataTile::new("BOOST PSI", TileFormat::Fixed { decimals: 1 }),
        power: DataTile::new("POWER KW", TileFormat::Integer { cells: 4 }).with_frame(TileFrame::Open),
    };
    // The G-ball arrives from the previous review state, so its 500 ms trail shows the path.
    let previous = STATES[state_index.saturating_sub(1)].telemetry();
    for _ in 0..FRAMES {
        let sig = smoother.update(&previous, &cfg, tokens, support::DT);
        c.gball.update(&sig, support::DT);
    }
    let (slip, temps) = TYRES[state_index];
    for _ in 0..FRAMES {
        let sig = smoother.update(&data, &cfg, tokens, support::DT);
        c.gball.update(&sig, support::DT);
        c.quad.update(slip, Some(temps), support::DT);
        c.bar.update(&sig, support::DT);
        c.gear.update(sig.gear, support::DT);
        let rear = (slip[2].abs() + slip[3].abs()) * 0.5;
        let front = slip[0].abs().max(slip[1].abs());
        let (tc, abs) = (tc_condition(rear, sig.throttle), abs_condition(front, sig.brake));
        let (rev, hot) = (rev_condition(&sig), tyre_condition(&temps));
        // FUEL has no verified source yet: it never lights.
        c.left.update(&[false, tc, abs], support::DT);
        c.right.update(&[rev, hot], support::DT);
        c.row.update(&[false, tc, abs, rev, hot], support::DT);
        c.boost.update(Some(data.boost_psi));
        c.power.update(Some(data.power_watts / 1000.0));
    }
    c
}

/// The fonts the shot draws with.
#[derive(Clone, Copy)]
struct Fonts {
    race: FontId,
    text: FontId,
}

/// Draws one column centred on `cx` (local 4K px).
fn draw_column(p: &mut Painter<'_>, c: &Column, cx: f32, palette: &Palette, fonts: Fonts) {
    let ts = p.tokens().type_scale;
    let title = TextStyle::new(fonts.text, ts.micro_label * 1.25)
        .paint(palette.roles.ink_label)
        .tracking(ts.label_tracking)
        .align(Align::Center, VAlign::CapTop);
    p.text(vec2(cx, -1040.0), c.title, &title);

    c.bar.draw(p, vec2(cx, BAR_Y), BAR_HALF, GEAR_CAP, palette);
    c.gear
        .draw(p, vec2(cx, GEAR_Y), fonts.race, GEAR_CAP, palette.roles.ink);

    c.gball
        .draw(p, vec2(cx, GBALL_Y), GBALL_R, palette, fonts.race, fonts.text);

    let quad = Rect::from_xywh(cx + QUAD.0, QUAD.1, QUAD.2, QUAD.3);
    c.quad.draw(p, quad, palette, fonts.race);

    c.left.draw(p, lamps_rect(cx, AlarmSide::Left), palette, fonts.text);
    c.right.draw(p, lamps_rect(cx, AlarmSide::Right), palette, fonts.text);

    let tile_h = DataTile::preferred_height(p.tokens(), TILE_CAP);
    let boost = Rect::from_xywh(cx - TILE_HALF, TILE_Y, TILE_HALF * 2.0, tile_h);
    c.boost.draw(p, boost, TILE_CAP, palette, fonts.race, fonts.text);
    let power = Rect::from_xywh(cx - TILE_HALF, TILE_Y + tile_h + 40.0, TILE_HALF * 2.0, tile_h);
    c.power.draw(p, power, TILE_CAP, palette, fonts.race, fonts.text);

    c.row.draw(
        p,
        Rect::from_xywh(cx - ROW_HALF, ROW_Y, ROW_HALF * 2.0, 100.0),
        palette,
        fonts.text,
    );
}

/// The alarm column rect on `side` of the column centred on `cx`.
fn lamps_rect(cx: f32, side: AlarmSide) -> Rect {
    match side {
        AlarmSide::Right => Rect::from_xywh(cx + LAMPS_OUTER - LAMPS_W, LAMPS_Y, LAMPS_W, LAMPS_H),
        _ => Rect::from_xywh(cx - LAMPS_OUTER, LAMPS_Y, LAMPS_W, LAMPS_H),
    }
}

/// Shot chrome: a hairline between two state columns at `x`, fading at both ends.
fn divider(p: &mut Painter<'_>, x: f32, palette: &Palette) {
    let line = Rect::from_xywh(x - 1.0, -1000.0, 2.0, 1980.0);
    let c = palette.roles.hairline;
    p.fill(
        &Shape::rect(line),
        Gradient::linear(line.at(0.5, 0.0), line.at(0.5, 1.0))
            .stop(0.0, c.fade(0.0))
            .stop(0.15, c.fade(0.6))
            .stop(0.85, c.fade(0.6))
            .stop(1.0, c.fade(0.0)),
    );
}

/// The four settled columns of `family`, with the state assumptions the shots rely on checked.
fn columns(tokens: &Tokens, family: &str) -> Vec<Column> {
    let look = Look::of(family);
    let columns: Vec<Column> = (0..4).map(|i| settle(tokens, look, i)).collect();
    let drift = &columns[3];
    assert!(
        drift.gball.ball().x > 0.9 && drift.gball.ball().y < -0.5,
        "drift ball up-right"
    );
    assert!(
        drift.gball.animating(),
        "the drift trail still shows the swing from the redline state"
    );
    assert!(drift.bar.value() < -0.5, "braking bar grows left");
    assert!(columns[2].bar.value() > 0.35, "acceleration bar grows right");
    assert!(
        drift.left.is_lit(1) && drift.left.is_lit(2),
        "TC and ABS lit while drift-braking"
    );
    assert!(drift.right.is_lit(1) && !drift.right.is_lit(0), "TYRE lit, REV off");
    assert!(columns[2].right.is_lit(0), "REV lit on the limiter");
    assert!(
        columns[2].row.is_lit(3) && drift.row.is_lit(1) && drift.row.is_lit(4),
        "row mirrors the columns"
    );
    assert!(
        !columns[1].left.is_lit(1) && !columns[1].right.is_lit(1),
        "cruise is quiet"
    );
    columns
}

fn fonts(p: &Painter<'_>) -> Fonts {
    Fonts {
        race: p.fonts().role(FontRole::Race),
        text: p.fonts().role(FontRole::Text),
    }
}

/// Renders and saves the four-state overview of `family`; returns it with the tokens it used.
fn shot(family: &str, name: &str) -> (RgbaImage, Tokens) {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, family);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let columns = columns(&tokens, family);

    let img = gpu.render(&layout, |p, _| {
        let f = fonts(p);
        for x in [-960.0, 0.0, 960.0] {
            divider(p, x, &palette);
        }
        for (c, cx) in columns.iter().zip(COLUMNS) {
            draw_column(p, c, cx, &palette, f);
        }
    });
    let path = support::save_shot(name, &img);
    assert!(path.exists(), "{}", path.display());
    (img, tokens)
}

/// Renders the redline and drift columns at 1:1 4K pixels: the top half (bar, gear, G-ball) and
/// the bottom half (tyres, lamps, tiles), for close review.
fn detail(family: &str, name: &str) {
    let mut gpu = support::gpu();
    let palette = Palette::family(&gpu.tokens, family);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let columns = columns(&tokens, family);
    // Local y windows: -1060..20 (top) and -20..1060 (bottom); the columns sit at x ±480.
    for (half, anchor_y) in [("top", 1060.0), ("bottom", 20.0)] {
        let layout = Layout::centered(vec2(960.0, anchor_y), 1.0, (support::W, support::H));
        let img = gpu.render(&layout, |p, _| {
            let f = fonts(p);
            divider(p, 0.0, &palette);
            draw_column(p, &columns[2], -480.0, &palette, f);
            draw_column(p, &columns[3], 480.0, &palette, f);
        });
        let (n, _) = lit(&img, (0, 0, support::W, support::H));
        assert!(n > 2000, "{family} {half}: detail has content ({n} px)");
        let path = support::save_shot(&format!("{name}_{half}"), &img);
        assert!(path.exists(), "{}", path.display());
    }
}

/// Local (4K, anchor-centred) to overview shot pixels.
fn to_px(p: Vec2) -> (u32, u32) {
    let s = support::H as f32 / 2160.0;
    let x = (p.x * s + support::W as f32 * 0.5).round().max(0.0);
    let y = (p.y * s + support::H as f32 * 0.5).round().max(0.0);
    (x as u32, y as u32)
}

/// A local rect in overview shot pixels `(x, y, w, h)`.
fn px_rect(r: Rect) -> (u32, u32, u32, u32) {
    let (x0, y0) = to_px(r.min);
    let (x1, y1) = to_px(r.max);
    (x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0))
}

/// Lit-pixel count and the mean colour of lit pixels in a pixel rect `(x, y, w, h)`.
fn lit(img: &RgbaImage, (x0, y0, w, h): (u32, u32, u32, u32)) -> (u32, [f32; 3]) {
    let mut n = 0u32;
    let mut sum = [0.0f32; 3];
    for y in y0..(y0 + h).min(img.height) {
        for x in x0..(x0 + w).min(img.width) {
            let i = ((y * img.width + x) * 4) as usize;
            let (r, g, b, a) = (img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], img.pixels[i + 3]);
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

fn lit_local(img: &RgbaImage, r: Rect) -> (u32, [f32; 3]) {
    lit(img, px_rect(r))
}

/// Horizontal extent (min, max pixel x) of lit pixels in a local rect, if any.
fn lit_span_x(img: &RgbaImage, r: Rect) -> Option<(u32, u32)> {
    let (x0, y0, w, h) = px_rect(r);
    let mut span: Option<(u32, u32)> = None;
    for y in y0..(y0 + h).min(img.height) {
        for x in x0..(x0 + w).min(img.width) {
            let i = ((y * img.width + x) * 4) as usize;
            let (r, g, b, a) = (img.pixels[i], img.pixels[i + 1], img.pixels[i + 2], img.pixels[i + 3]);
            if a > 40 && u32::from(r) + u32::from(g) + u32::from(b) > 150 {
                span = Some(span.map_or((x, x), |(lo, hi)| (lo.min(x), hi.max(x))));
            }
        }
    }
    span
}

/// Mean RGB (ignoring alpha) of every pixel in a local rect.
fn mean_rgb(img: &RgbaImage, r: Rect) -> [f32; 3] {
    let (x0, y0, w, h) = px_rect(r);
    let mut sum = [0.0f32; 3];
    let mut n = 0u32;
    for y in y0..(y0 + h).min(img.height) {
        for x in x0..(x0 + w).min(img.width) {
            let i = ((y * img.width + x) * 4) as usize;
            for (c, s) in sum.iter_mut().enumerate() {
                *s += f32::from(img.pixels[i + c]);
            }
            n += 1;
        }
    }
    let k = 1.0 / n.max(1) as f32;
    [sum[0] * k, sum[1] * k, sum[2] * k]
}

fn check_pixels(img: &RgbaImage, tokens: &Tokens, family: &str) {
    let look = Look::of(family);
    let drift_x = COLUMNS[3];
    let redline_x = COLUMNS[2];
    let cruise_x = COLUMNS[1];
    let per_g = GBALL_R / 1.5;

    // The drift ball sits up-right of the centre (felt force of a left-hand, braking slide).
    let ball = vec2(drift_x + 0.96 * per_g, GBALL_Y - 0.58 * per_g);
    let (n, _) = lit_local(img, Rect::from_center_size(ball, vec2(24.0, 24.0)));
    assert!(n > 10, "{family}: ball lit at {ball:?} ({n} px)");
    let mirror = vec2(drift_x - 0.96 * per_g, GBALL_Y + 0.58 * per_g);
    let (n, _) = lit_local(img, Rect::from_center_size(mirror, vec2(12.0, 12.0)));
    assert!(n == 0, "{family}: nothing at the mirrored position ({n} px)");

    // The total-g readout ("0.42 G") is measured and centred under the meter: the value and
    // unit together sit within 2 px (1080p) of the column centre.
    let cap = tokens.type_scale.metric_secondary;
    let baseline = GBall::readout_baseline(vec2(redline_x, GBALL_Y), GBALL_R, cap).y;
    let band = Rect::from_xywh(redline_x - 160.0, baseline - cap - 4.0, 320.0, cap + 8.0);
    let (lo, hi) = lit_span_x(img, band).unwrap_or((0, 0));
    let mid = (lo + hi) as f32 * 0.5;
    let (centre, _) = to_px(vec2(redline_x, baseline));
    assert!(
        hi > lo && (mid - centre as f32).abs() <= 2.0,
        "{family}: readout centred ({lo}..{hi} around {centre})"
    );

    // Signed bar: acceleration right (go, or neutral ink on Race Dash), braking left in warning.
    let (n, [r, g, b]) = lit_local(img, Rect::from_xywh(redline_x + 30.0, BAR_Y - 8.0, 70.0, 16.0));
    match look.accel {
        AccelTone::Go => assert!(
            n > 20 && g > r && g > b,
            "{family}: redline bar green ({n} px, {r} {g} {b})"
        ),
        AccelTone::Ink => assert!(
            n > 20 && (r - g).abs() < 20.0 && (g - b).abs() < 20.0,
            "{family}: redline bar neutral ink ({n} px, {r} {g} {b})"
        ),
    }
    let (n, _) = lit_local(img, Rect::from_xywh(redline_x - 100.0, BAR_Y - 8.0, 70.0, 16.0));
    assert!(n == 0, "{family}: redline bar has no left fill ({n} px)");
    let (n, [r, g, _]) = lit_local(img, Rect::from_xywh(drift_x - 100.0, BAR_Y - 8.0, 70.0, 16.0));
    assert!(
        n > 20 && r > g * 1.5,
        "{family}: drift bar left, red ({n} px, r {r} g {g})"
    );

    // Tyre fills: the tyre rects at the corners of the footprint (24% x 30% of it).
    let tyre = |cx: f32, right: bool, rear: bool| {
        let (w, h) = (QUAD.2 * 0.24, QUAD.3 * 0.30);
        let x = if right { cx + QUAD.0 + QUAD.2 - w } else { cx + QUAD.0 };
        let y = if rear {
            QUAD.1 + QUAD.3 * 0.80 - h * 0.5
        } else {
            QUAD.1 + QUAD.3 * 0.20 - h * 0.5
        };
        Rect::from_xywh(x + 8.0, y + 8.0, w - 16.0, h - 16.0)
    };
    // Drift rear-right: overheated, red in both looks.
    let (n, [r, g, b]) = lit_local(img, tyre(drift_x, true, true));
    assert!(
        n > 20 && r > g && r > b,
        "{family}: overheated tyre red ({n} px, {r} {g} {b})"
    );
    // Cruise front-left: optimal, green on the ramp, a neutral lens on Race Dash (dark rubber:
    // nothing bright or coloured, but visibly above OLED black).
    let (n, [r, g, b]) = lit_local(img, tyre(cruise_x, false, false));
    match look.tyres {
        TyreFill::Ramp => assert!(
            n > 20 && g > r && g > b,
            "{family}: optimal tyre green ({n} px, {r} {g} {b})"
        ),
        TyreFill::Exception => {
            let [r, g, b] = mean_rgb(img, tyre(cruise_x, false, false));
            assert!(
                n == 0 && r > 4.0 && (r - g).abs() < 4.0 && (g - b).abs() < 4.0,
                "{family}: optimal tyre a neutral lens ({n} bright px, mean {r} {g} {b})"
            );
        }
    }

    // Temperature labels sit as close to a right-hand tyre as to a left-hand one (cruise reads
    // two digits: no empty leading cell may push the right label outwards).
    let front_y = QUAD.1 + QUAD.3 * 0.20;
    let (tyre_l, _) = to_px(vec2(cruise_x + QUAD.0, front_y));
    let (tyre_r, _) = to_px(vec2(cruise_x + QUAD.0 + QUAD.2, front_y));
    let left = lit_span_x(
        img,
        Rect::from_xywh(cruise_x + QUAD.0 - 200.0, front_y - 30.0, 192.0, 60.0),
    );
    let right = lit_span_x(
        img,
        Rect::from_xywh(cruise_x + QUAD.0 + QUAD.2 + 8.0, front_y - 30.0, 192.0, 60.0),
    );
    assert!(
        left.is_some() && right.is_some(),
        "{family}: both front temperatures drawn ({left:?}, {right:?})"
    );
    let (Some((_, left_hi)), Some((right_lo, _))) = (left, right) else {
        return;
    };
    let (gap_l, gap_r) = (tyre_l as f32 - left_hi as f32, right_lo as f32 - tyre_r as f32);
    assert!(
        (gap_l - gap_r).abs() <= 4.0,
        "{family}: temperature gaps match ({gap_l} px left, {gap_r} px right)"
    );

    // Alarm lamps: the drift TYRE lamp is lit (caution: red and green, little blue), the idle
    // TYRE lamp is a grey lens (nothing above the lit threshold).
    let tyre_lamp = |cx: f32| {
        let rect = lamps_rect(cx, AlarmSide::Right);
        let slot = LAMPS_H / 5.0;
        Rect::from_xywh(rect.max.x - 100.0, rect.min.y + slot * 1.5 - 17.0, 100.0, 34.0)
    };
    let (n, [r, g, b]) = lit_local(img, tyre_lamp(drift_x));
    assert!(
        n > 100 && r > b * 1.5 && g > b,
        "{family}: TYRE lamp lit ({n} px, {r} {g} {b})"
    );
    let (n, _) = lit_local(img, tyre_lamp(COLUMNS[0]));
    assert!(n == 0, "{family}: idle TYRE lamp unlit ({n} px)");

    // Bottom row: lamp i is centred at cx - ROW_HALF + slot * (i + 0.5); LIM lights in the
    // palette shift colour on the limiter, TC in caution while drift-braking.
    let row_lamp = |cx: f32, i: f32| {
        let slot = ROW_HALF * 2.0 / 5.0;
        Rect::from_xywh(cx - ROW_HALF + slot * (i + 0.5) - 45.0, ROW_Y + 4.0, 90.0, 26.0)
    };
    let (n, [r, g, b]) = lit_local(img, row_lamp(drift_x, 1.0));
    assert!(
        n > 100 && r > b * 1.5 && g > b,
        "{family}: row TC lit ({n} px, {r} {g} {b})"
    );
    let (n, [_, g, b]) = lit_local(img, row_lamp(redline_x, 3.0));
    assert!(
        n > 100 && b > g,
        "{family}: row LIM lit in the shift colour ({n} px, g {g} b {b})"
    );
    let (n, _) = lit_local(img, row_lamp(cruise_x, 3.0));
    assert!(n == 0, "{family}: cruise LIM unlit ({n} px)");
}

#[test]
fn instruments_race_dash() {
    let (img, tokens) = shot("raceDash", "instruments_race_dash");
    check_pixels(&img, &tokens, "raceDash");
}

#[test]
fn instruments_modern_digital() {
    let (img, tokens) = shot("modernDigital", "instruments_modern_digital");
    check_pixels(&img, &tokens, "modernDigital");
}

/// The tyre quad paints nothing opaque: over a lit backdrop (a glass plate, or the game itself in
/// the frameless hood view) the wheel arches show the backdrop, not black holes, and the body
/// line still stops short of every tyre.
#[test]
fn tyre_quad_over_the_scene() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "modernDigital");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let mut quad = TyreSlipQuad::new(&tokens);
    for _ in 0..FRAMES {
        quad.update([0.0, 0.02, 0.12, 0.24], Some([82.0, 95.0, 106.0, 118.0]), support::DT);
    }
    // FL tyre: x -260..-135.2, y -324..-108 (24% x 30% of the footprint); arch air 17.5.
    let rect = Rect::from_xywh(-260.0, -360.0, 520.0, 720.0);
    let img = gpu.render(&layout, |p, _| {
        let race = p.fonts().role(FontRole::Race);
        // A stand-in for whatever lies behind the overlay.
        let scene = Rect::from_xywh(-1920.0, -1080.0, 3840.0, 2160.0);
        p.fill(&Shape::rect(scene), palette.roles.ink_muted.fade(0.6));
        quad.draw(p, rect, &palette, race);
    });
    let path = support::save_shot("instruments_tyre_quad_over_scene", &img);
    assert!(path.exists(), "{}", path.display());

    let backdrop = mean_rgb(&img, Rect::from_xywh(-900.0, -40.0, 80.0, 80.0));
    assert!(backdrop.iter().all(|c| *c > 40.0), "backdrop is lit: {backdrop:?}");
    // Outboard of the FL tyre, inside its arch air (no slip ring on FL), above its temperature.
    let arch = mean_rgb(&img, Rect::from_xywh(-273.0, -300.0, 8.0, 40.0));
    for (a, b) in arch.iter().zip(backdrop) {
        assert!((a - b).abs() < 6.0, "arch shows the backdrop: {arch:?} vs {backdrop:?}");
    }
    // The body line (x = -210) shows between the axles, and is clipped where it would cross
    // the translucent tyre.
    let brightest = |r: Rect| {
        let (x0, y0, w, h) = px_rect(r);
        let mut best = 0u32;
        for y in y0..(y0 + h).min(img.height) {
            for x in x0..(x0 + w).min(img.width) {
                let i = ((y * img.width + x) * 4) as usize;
                best = best.max(u32::from(img.pixels[i]) + u32::from(img.pixels[i + 1]) + u32::from(img.pixels[i + 2]));
            }
        }
        best
    };
    let base = (backdrop[0] + backdrop[1] + backdrop[2]) as u32;
    let waist = brightest(Rect::from_xywh(-216.0, -40.0, 12.0, 80.0));
    assert!(
        waist > base + 6,
        "body line visible between the wheels ({waist} vs {base})"
    );
    // One-pixel columns inside the FL tyre: on the line's x, and clear of it and the grooves
    // (the tread shading only varies vertically, so the two match unless a line crosses).
    let through_tyre = mean_rgb(&img, Rect::from_xywh(-211.0, -280.0, 2.0, 130.0));
    let tyre_beside = mean_rgb(&img, Rect::from_xywh(-197.0, -280.0, 2.0, 130.0));
    let (a, b) = (through_tyre.iter().sum::<f32>(), tyre_beside.iter().sum::<f32>());
    assert!(
        (a - b).abs() < 9.0,
        "no body line across the tyre ({through_tyre:?} vs {tyre_beside:?})"
    );
}

#[test]
fn instruments_detail() {
    detail("raceDash", "instruments_race_dash_detail");
    detail("modernDigital", "instruments_modern_digital_detail");
}
