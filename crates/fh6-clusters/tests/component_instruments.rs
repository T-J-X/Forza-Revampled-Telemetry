mod support;

// Visual shot test for the rank-4/5 instruments: GBall, TyreSlipQuad, SignedBar, AlarmColumn and
// DataTile, driven through the four review states (idle, cruise, redline, drift-braking) side by
// side, in the Race Dash and Modern Digital family palettes.
//
// Signals come from the review telemetry through the kit's `SignalSmoother` (the same path the
// host uses). Tyre slip and temperatures are not in the golden packets, so the shot feeds
// synthetic per-state values, as a cluster would after verifying the real fields.

use fh6_clusters::common::gball::GBall;
use fh6_clusters::common::race::{
    abs_condition, rev_condition, tc_condition, tyre_condition, AlarmColumn, AlarmLamp, AlarmSide, DataTile,
    TileFormat, TileFrame,
};
use fh6_clusters::common::signed_bar::{SignedBar, SignedSource};
use fh6_clusters::common::tyre_slip::TyreSlipQuad;
use fh6_clusters::common::GearGlyph;
use fh6_clusters::review::STATES;
use fh6_core::AppConfig;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Palette, Rect, SignalSmoother, TextStyle, VAlign, Vec2};

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
/// Data tile value cap.
const TILE_CAP: f32 = 64.0;

/// Synthetic tyre inputs per review state: slip ratios and temperatures (°C), FL FR RL RR.
const TYRES: [([f32; 4], [f32; 4]); 4] = [
    ([0.0, 0.0, 0.0, 0.0], [45.0, 46.0, 44.0, 45.0]),
    ([0.02, 0.02, 0.03, 0.03], [88.0, 90.0, 94.0, 95.0]),
    ([0.03, 0.03, 0.12, 0.11], [96.0, 97.0, 108.0, 109.0]),
    ([0.06, 0.09, 0.24, 0.27], [98.0, 104.0, 118.0, 121.0]),
];

/// Everything one column shows, settled on one state.
struct Column {
    title: &'static str,
    gball: GBall,
    quad: TyreSlipQuad,
    bar: SignedBar,
    gear: GearGlyph,
    left: AlarmColumn,
    right: AlarmColumn,
    boost: DataTile,
    power: DataTile,
}

fn settle(tokens: &fh6_kit::Tokens, state_index: usize) -> Column {
    let state = &STATES[state_index];
    let data = state.telemetry();
    let cfg = AppConfig::default();
    let mut smoother = SignalSmoother::new();
    let mut c = Column {
        title: state.name,
        gball: GBall::new(tokens),
        quad: TyreSlipQuad::new(tokens),
        bar: SignedBar::new(tokens, SignedSource::LongitudinalG { full_scale_g: 1.0 }),
        gear: GearGlyph::new(tokens),
        left: AlarmColumn::new(tokens, AlarmSide::Left, &[AlarmLamp::FUEL, AlarmLamp::TC, AlarmLamp::ABS]),
        right: AlarmColumn::new(tokens, AlarmSide::Right, &[AlarmLamp::REV, AlarmLamp::TYRE]),
        boost: DataTile::new("BOOST PSI", TileFormat::Fixed { decimals: 1 }),
        power: DataTile::new("POWER KW", TileFormat::Integer { cells: 4 }).with_frame(TileFrame::Open),
    };
    let (slip, temps) = TYRES[state_index];
    for _ in 0..FRAMES {
        let sig = smoother.update(&data, &cfg, tokens, support::DT);
        c.gball.update(&sig, support::DT);
        c.quad.update(slip, Some(temps), support::DT);
        c.bar.update(&sig, support::DT);
        c.gear.update(sig.gear, support::DT);
        let rear = (slip[2].abs() + slip[3].abs()) * 0.5;
        let front = slip[0].abs().max(slip[1].abs());
        // FUEL has no verified source yet: it never lights.
        c.left.update(
            &[false, tc_condition(rear, sig.throttle), abs_condition(front, sig.brake)],
            support::DT,
        );
        c.right.update(&[rev_condition(&sig), tyre_condition(&temps)], support::DT);
        c.boost.update(Some(data.boost_psi));
        c.power.update(Some(data.power_watts / 1000.0));
    }
    c
}

/// Local (4K, anchor-centred) to shot pixels.
fn to_px(p: Vec2) -> (i64, i64) {
    let s = support::H as f32 / 2160.0;
    (
        (p.x * s + support::W as f32 * 0.5).round() as i64,
        (p.y * s + support::H as f32 * 0.5).round() as i64,
    )
}

/// Lit-pixel count and the mean colour of lit pixels in a local rect.
fn lit(img: &RgbaImage, r: Rect) -> (u32, [f32; 3]) {
    let (x0, y0) = to_px(r.min);
    let (x1, y1) = to_px(r.max);
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

fn shot(family: &str, name: &str) -> RgbaImage {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, family);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let columns: Vec<Column> = (0..4).map(|i| settle(&tokens, i)).collect();

    // State assumptions the shot relies on.
    let drift = &columns[3];
    assert!(drift.gball.ball().x > 0.9 && drift.gball.ball().y < -0.5, "drift ball up-right");
    assert!(drift.bar.value() < -0.5, "braking bar grows left");
    assert!(columns[2].bar.value() > 0.35, "acceleration bar grows right");
    assert!(drift.left.is_lit(1) && drift.left.is_lit(2), "TC and ABS lit while drift-braking");
    assert!(drift.right.is_lit(1) && !drift.right.is_lit(0), "TYRE lit, REV off");
    assert!(columns[2].right.is_lit(0), "REV lit on the limiter");
    assert!(!columns[1].left.is_lit(1) && !columns[1].right.is_lit(1), "cruise is quiet");

    let img = gpu.render(&layout, |p, _| {
        let race = p.fonts().role(FontRole::Race);
        let text = p.fonts().role(FontRole::Text);
        let ts = p.tokens().type_scale;
        for (c, cx) in columns.iter().zip(COLUMNS) {
            let title = TextStyle::new(text, ts.micro_label * 1.25)
                .paint(palette.roles.ink_label)
                .tracking(ts.label_tracking)
                .align(Align::Center, VAlign::CapTop);
            p.text(vec2(cx, -1040.0), c.title, &title);

            c.bar.draw(p, vec2(cx, BAR_Y), BAR_HALF, GEAR_CAP, &palette);
            c.gear.draw(p, vec2(cx, GEAR_Y), race, GEAR_CAP, palette.roles.ink);

            c.gball.draw(p, vec2(cx, GBALL_Y), GBALL_R, &palette, race, text);

            let quad = Rect::from_xywh(cx + QUAD.0, QUAD.1, QUAD.2, QUAD.3);
            c.quad.draw(p, quad, &palette, race);

            c.left.draw(p, Rect::from_xywh(cx - 470.0, 470.0, 200.0, 520.0), &palette, text);
            c.right.draw(p, Rect::from_xywh(cx + 270.0, 470.0, 200.0, 520.0), &palette, text);

            let tile_h = DataTile::preferred_height(p.tokens(), TILE_CAP);
            let boost = Rect::from_xywh(cx - 170.0, 480.0, 340.0, tile_h);
            c.boost.draw(p, boost, TILE_CAP, &palette, race, text);
            let power = Rect::from_xywh(cx - 170.0, 480.0 + tile_h + 40.0, 340.0, tile_h);
            c.power.draw(p, power, TILE_CAP, &palette, race, text);
        }
    });
    let path = support::save_shot(name, &img);
    assert!(path.exists(), "{}", path.display());
    img
}

fn check_pixels(img: &RgbaImage, family: &str) {
    let drift_x = COLUMNS[3];
    let redline_x = COLUMNS[2];
    let per_g = GBALL_R / 1.5;

    // The drift ball sits up-right of the centre (felt force of a left-hand, braking slide).
    let ball = vec2(drift_x + 0.96 * per_g, GBALL_Y - 0.58 * per_g);
    let (n, _) = lit(img, Rect::from_center_size(ball, vec2(24.0, 24.0)));
    assert!(n > 10, "{family}: ball lit at {ball:?} ({n} px)");
    let mirror = vec2(drift_x - 0.96 * per_g, GBALL_Y + 0.58 * per_g);
    let (n, _) = lit(img, Rect::from_center_size(mirror, vec2(12.0, 12.0)));
    assert!(n == 0, "{family}: nothing at the mirrored position ({n} px)");

    // Signed bar: acceleration right in `go` (green), braking left in `warning` (red).
    let (n, [r, g, _]) = lit(img, Rect::from_xywh(redline_x + 30.0, BAR_Y - 8.0, 70.0, 16.0));
    assert!(n > 20 && g > r, "{family}: redline bar right, green ({n} px, r {r} g {g})");
    let (n, _) = lit(img, Rect::from_xywh(redline_x - 100.0, BAR_Y - 8.0, 70.0, 16.0));
    assert!(n == 0, "{family}: redline bar has no left fill ({n} px)");
    let (n, [r, g, _]) = lit(img, Rect::from_xywh(drift_x - 100.0, BAR_Y - 8.0, 70.0, 16.0));
    assert!(n > 20 && r > g * 1.5, "{family}: drift bar left, red ({n} px, r {r} g {g})");

    // Rear-right tyre in the drift column: overheated (red) fill.
    let rr = Rect::from_xywh(drift_x + 130.0 - 0.24 * 260.0 + 8.0, 360.0 - 72.0 - 54.0 + 8.0, 40.0, 60.0);
    let (n, [r, g, b]) = lit(img, rr);
    assert!(n > 20 && r > g && r > b, "{family}: overheated tyre red ({n} px, {r} {g} {b})");

    // Alarm lamps: the drift TYRE lamp is lit (caution: red and green, little blue), the idle
    // TYRE lamp is a grey lens (nothing above the lit threshold).
    let tyre_lamp = |cx: f32| {
        let slot = 520.0 / 5.0;
        Rect::from_xywh(cx + 470.0 - 100.0, 470.0 + slot * 1.5 - 17.0, 100.0, 34.0)
    };
    let (n, [r, g, b]) = lit(img, tyre_lamp(drift_x));
    assert!(n > 100 && r > b * 1.5 && g > b, "{family}: TYRE lamp lit ({n} px, {r} {g} {b})");
    let (n, _) = lit(img, tyre_lamp(COLUMNS[0]));
    assert!(n == 0, "{family}: idle TYRE lamp unlit ({n} px)");
}

#[test]
fn instruments_race_dash() {
    let img = shot("raceDash", "instruments_race_dash");
    check_pixels(&img, "raceDash");
}

#[test]
fn instruments_modern_digital() {
    let img = shot("modernDigital", "instruments_modern_digital");
    check_pixels(&img, "modernDigital");
}
