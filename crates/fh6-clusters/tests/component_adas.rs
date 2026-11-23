mod support;

// Visual shot test for AdasRoad (COMPONENTS §10): the ADAS & HUD road scene in four states side
// by side (idle, cruise on a gentle bend with traffic, limiter with a car in the blind spot,
// drift-braking in a left-hand bend), plus a native-scale shot of the family's real scene band
// (1000 x 290 local px at 4K density) for line weights and label legibility.
//
// The road users are the component's seeded ambient simulation (the telemetry has no traffic);
// yaw rate, slip and lateral G are synthetic per-state values, as a cluster would feed them after
// verifying the real fields.

use fh6_clusters::common::adas_road::{AdasInput, AdasRoad, Underglow, EGO_Z};
use fh6_kit::math::vec3;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Layout, Palette, Rect, Signals, TextStyle, VAlign, Vec2};

/// Frames each state is settled for (the review harness settles 30).
const FRAMES: usize = 30;
/// The family palette.
const FAMILY: &str = "adasHud";
/// The overview panels (local 4K px): 3.4:1 like the family band, enlarged for review.
const PANEL_W: f32 = 1720.0;
const PANEL_H: f32 = 506.0;
const PANEL_X: [f32; 2] = [-1800.0, 80.0];
const PANEL_Y: [f32; 2] = [-930.0, 120.0];
/// The family's real scene band (adas-hud.md: y -50 .. 240, about 1000 px wide).
const BAND: (f32, f32, f32, f32) = (-500.0, -50.0, 1000.0, 290.0);

/// One state of the scene.
struct Case {
    title: &'static str,
    sig: Signals,
    underglow: Option<Underglow>,
    seed: u64,
}

fn cases() -> [Case; 4] {
    let base = Signals {
        race_on: true,
        max_rpm: 8000.0,
        idle_rpm: 900.0,
        shift_rpm: 7600.0,
        ..Signals::default()
    };
    [
        Case {
            title: "IDLE",
            sig: Signals {
                rpm: 900.0,
                shift_fraction: 0.12,
                ..base
            },
            underglow: None,
            seed: 7,
        },
        Case {
            // 31 m/s on a 260 m right-hand bend: kappa = 0.12 / 31.
            title: "CRUISE  BEND RIGHT  TRAFFIC",
            sig: Signals {
                speed_ms: 31.0,
                rpm: 4800.0,
                shift_fraction: 0.63,
                throttle: 0.35,
                yaw_rate: 0.12,
                g_lat: 0.38,
                slip_deg: 1.5,
                ..base
            },
            underglow: None,
            seed: 7,
        },
        Case {
            // Straight at 52 m/s on the limiter; the seed is picked so a car sits alongside.
            title: "LIMITER  BLIND SPOT",
            sig: Signals {
                speed_ms: 52.0,
                rpm: 7900.0,
                shift_fraction: 1.04,
                throttle: 1.0,
                yaw_rate: 0.0,
                ..base
            },
            underglow: None,
            seed: 0,
        },
        Case {
            // 20 m/s, 40 m left-hand bend, 27 degrees of slip, braking with an assist intervening.
            title: "DRIFT  BRAKING  ASSIST",
            sig: Signals {
                speed_ms: 20.0,
                rpm: 5600.0,
                shift_fraction: 0.74,
                brake: 0.85,
                yaw_rate: -0.5,
                g_lat: -1.0,
                slip_deg: 27.0,
                ..base
            },
            underglow: Some(Underglow::Assist),
            seed: 7,
        },
    ]
}

/// The input of a case.
fn input(c: &Case) -> AdasInput {
    let input = AdasInput::from_signals(&c.sig);
    c.underglow.map_or(input, |u| input.with_underglow(u))
}

/// A scene settled on `c` for [`FRAMES`] frames.
fn settle(tokens: &fh6_kit::Tokens, c: &Case, seed: u64) -> AdasRoad {
    let mut road = AdasRoad::new(tokens, seed);
    let input = input(c);
    for _ in 0..FRAMES {
        road.update(&input, support::DT);
    }
    road
}

/// The first seed from `c.seed` that puts a car alongside the ego (proximity step 2 or more), so
/// the limiter panel shows the blind-spot arc; deterministic.
fn blind_spot_seed(tokens: &fh6_kit::Tokens, c: &Case) -> u64 {
    (c.seed..c.seed + 256)
        .find(|&s| settle(tokens, c, s).blind_spot().iter().any(|&b| b >= 2))
        .unwrap_or(c.seed)
}

fn panel(i: usize) -> Rect {
    Rect::from_xywh(PANEL_X[i % 2], PANEL_Y[i / 2], PANEL_W, PANEL_H)
}

/// Pixels in a local rect (under `layout`) that satisfy `pred` on RGB.
fn count(img: &RgbaImage, layout: &Layout, r: Rect, pred: impl Fn(f32, f32, f32) -> bool) -> u32 {
    let a = layout.to_target(r.min);
    let b = layout.to_target(r.max);
    let (x0, y0) = (a.x.max(0.0).floor() as u32, a.y.max(0.0).floor() as u32);
    let (x1, y1) = ((b.x.ceil() as u32).min(img.width), (b.y.ceil() as u32).min(img.height));
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = ((y * img.width + x) * 4) as usize;
            let px = &img.pixels[i..i + 3];
            if pred(f32::from(px[0]), f32::from(px[1]), f32::from(px[2])) {
                n += 1;
            }
        }
    }
    n
}

/// Visible at all on OLED black.
fn lit(r: f32, g: f32, b: f32) -> bool {
    r + g + b > 24.0
}

/// The ego's ground point in a viewport (the road's lateral offset at the ego is < 0.2 m).
fn ego_point(vp: &Rect) -> Vec2 {
    AdasRoad::camera(vp)
        .project(vec3(0.0, 0.0, EGO_Z))
        .unwrap_or_else(|| vp.center())
}

#[test]
fn adas_road_states() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, FAMILY);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let cases = cases();
    let limiter_seed = blind_spot_seed(&tokens, &cases[2]);
    let roads: Vec<AdasRoad> = cases
        .iter()
        .enumerate()
        .map(|(i, c)| settle(&tokens, c, if i == 2 { limiter_seed } else { c.seed }))
        .collect();

    // State assumptions the shot relies on.
    assert!(
        !roads[0].animating() || roads[0].curvature().abs() < 1e-6,
        "idle is still"
    );
    assert!((roads[1].curvature() - 0.12 / 31.0).abs() < 4e-4, "cruise bends right");
    assert!(roads[1].lead_gap().is_some(), "cruise has an in-path lead car");
    assert!(
        roads[2].blind_spot().iter().any(|&b| b >= 2),
        "limiter: a car alongside"
    );
    assert!(
        roads[3].curvature() < -0.02,
        "drift bends left: {}",
        roads[3].curvature()
    );

    let img = gpu.render(&layout, |p, _| {
        let text = p.fonts().role(FontRole::Text);
        let ts = p.tokens().type_scale;
        let title = TextStyle::new(text, ts.micro_label * 1.6)
            .paint(palette.roles.ink_label)
            .tracking(ts.label_tracking)
            .align(Align::Left, VAlign::CapTop);
        let note = TextStyle::new(text, ts.micro_label * 1.2)
            .paint(palette.roles.ink_muted)
            .tracking(ts.label_tracking)
            .align(Align::Right, VAlign::CapTop);
        for (i, (c, road)) in cases.iter().zip(&roads).enumerate() {
            let vp = panel(i);
            p.text(vec2(vp.min.x + 40.0, vp.min.y - 80.0), c.title, &title);
            p.text(vec2(vp.max.x - 40.0, vp.min.y - 76.0), "SIMULATED TRAFFIC", &note);
            road.draw(p, vp, &palette);
        }
    });
    let path = support::save_shot("adas_road", &img);
    assert!(path.exists(), "{}", path.display());

    // Nothing outside the panels and their title rows: the scene dissolves, no viewport box.
    let any = |r: Rect| count(&img, &layout, r, lit);
    for i in 0..4 {
        let vp = panel(i);
        let below = Rect::from_xywh(vp.min.x, vp.max.y + 2.0, vp.width(), 120.0);
        let (n_in, n_below) = (any(vp), any(below));
        assert!(n_in > 2000, "panel {i} draws a scene ({n_in} px)");
        if i < 2 {
            assert!(n_below == 0, "panel {i} leaks below its viewport ({n_below} px)");
        }
    }
    let gutter = Rect::from_xywh(
        PANEL_X[0] + PANEL_W + 4.0,
        -900.0,
        PANEL_X[1] - PANEL_X[0] - PANEL_W - 8.0,
        1500.0,
    );
    assert_eq!(any(gutter), 0, "nothing between the panels");

    // The ground beside the ego: the underglow's colour (primary blue, violet limiter, amber
    // assist), sampled on both sides of the footprint.
    let beside = |i: usize| {
        let vp = panel(i);
        let e = ego_point(&vp);
        let w = vp.height() * 0.32;
        [
            Rect::from_xywh(e.x - w, e.y - 30.0, w * 0.35, 40.0),
            Rect::from_xywh(e.x + w * 0.65, e.y - 30.0, w * 0.35, 40.0),
        ]
    };
    let glow = |i: usize, pred: &dyn Fn(f32, f32, f32) -> bool| -> u32 {
        beside(i).iter().map(|r| count(&img, &layout, *r, pred)).sum()
    };
    let blue = |r: f32, g: f32, b: f32| b > 40.0 && b > r * 1.6 && b > g * 1.1;
    let violet = |r: f32, g: f32, b: f32| b > 40.0 && r > g * 1.2 && b > g * 1.2;
    let amber = |r: f32, g: f32, b: f32| r > 40.0 && r > b * 1.8 && g > b * 1.2;
    let n = glow(1, &blue);
    assert!(n > 200, "cruise underglow is the primary blue ({n} px)");
    let n = glow(2, &violet);
    assert!(n > 200, "limiter underglow is violet ({n} px)");
    let n = glow(3, &amber);
    assert!(n > 200, "assist underglow is amber ({n} px)");
    assert_eq!(glow(1, &amber), 0, "no amber in the cruise underglow");

    // Brake lights: bright red on the ego only while braking.
    let ego_box = |i: usize| {
        let vp = panel(i);
        let e = ego_point(&vp);
        Rect::from_xywh(e.x - 220.0, e.y - 150.0, 440.0, 170.0)
    };
    // Bright red: the running light (22% intensity) stays below this, the braking bar above.
    let red = |r: f32, g: f32, b: f32| r > 170.0 && r > g * 2.5 && r > b * 2.2;
    let n = count(&img, &layout, ego_box(3), red);
    assert!(n > 60, "drift brake bar glows ({n} px)");
    let n = count(&img, &layout, ego_box(1), red);
    assert!(n == 0, "no brake glow while cruising ({n} px)");

    // The carpet ahead of the ego: blue at cruise, amber in the drift.
    let ahead = |i: usize| {
        let vp = panel(i);
        let e = ego_point(&vp);
        Rect::from_xywh(
            e.x - 260.0,
            vp.min.y + vp.height() * 0.30,
            520.0,
            e.y - 150.0 - (vp.min.y + vp.height() * 0.30),
        )
    };
    let n = count(&img, &layout, ahead(1), blue);
    assert!(n > 100, "cruise carpet blue ({n} px)");
    let n = count(&img, &layout, ahead(3), amber);
    assert!(n > 100, "drift carpet amber ({n} px)");

    // Idle shows no warning colour at all.
    let n = count(&img, &layout, panel(0), red) + count(&img, &layout, panel(0), amber);
    assert_eq!(n, 0, "idle is calm");
}

#[test]
fn adas_road_native_band() {
    let mut gpu = support::gpu();
    // Native 4K pixel density: one local unit per target pixel.
    let layout = Layout::centered(
        vec2(support::W as f32 * 0.5, support::H as f32 * 0.5),
        1.0,
        (support::W, support::H),
    );
    let palette = Palette::family(&gpu.tokens, FAMILY);
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let cases = cases();
    let shown = [&cases[1], &cases[3]];
    let roads: Vec<AdasRoad> = shown.iter().map(|c| settle(&tokens, c, c.seed)).collect();
    let bands = [
        Rect::from_xywh(BAND.0, -440.0, BAND.2, BAND.3),
        Rect::from_xywh(BAND.0, 80.0, BAND.2, BAND.3),
    ];
    let img = gpu.render(&layout, |p, _| {
        for (road, band) in roads.iter().zip(bands) {
            road.draw(p, band, &palette);
        }
    });
    let path = support::save_shot("adas_road_native", &img);
    assert!(path.exists(), "{}", path.display());
    for (i, band) in bands.iter().enumerate() {
        let n = count(&img, &layout, *band, lit);
        assert!(n > 1500, "band {i} draws a scene ({n} px)");
        let outside = Rect::from_xywh(band.min.x, band.max.y + 2.0, band.width(), 60.0);
        assert_eq!(count(&img, &layout, outside, lit), 0, "band {i} stays in its viewport");
    }
}
