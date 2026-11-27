//! GPU tests of the kit: every primitive, text, glass, bloom and blur render without validation
//! errors and put pixels where they should.
#![cfg(not(target_arch = "wasm32"))]

use fh6_gpu::GpuContext;
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{
    deg, vec2, Cap, Color, Glass, Glow, Gradient, Kit, LayerOptions, Layout, Painter, Rect, SegmentedBar, Shadow,
    Shape, Stroke, TextStyle, TickRing, Tokens, Vec2,
};

const W: u32 = 960;
const H: u32 = 540;

fn render(kit: &mut Kit, ctx: &GpuContext, draw: impl FnOnce(&mut Painter<'_>)) -> RgbaImage {
    render_named(kit, ctx, "frame", draw)
}

/// Renders and, when `FH6_KIT_DUMP` names a folder, saves `<name>.png` there for inspection.
fn render_named(kit: &mut Kit, ctx: &GpuContext, name: &str, draw: impl FnOnce(&mut Painter<'_>)) -> RgbaImage {
    let layout = Layout::centered(vec2(W as f32 * 0.5, H as f32 * 0.5), 0.5, (W, H));
    {
        let mut p = kit.begin_frame(&layout);
        draw(&mut p);
    }
    let target = Offscreen::new(&ctx.device, W, H);
    let mut encoder = ctx.device.create_command_encoder(&Default::default());
    let stats = kit.render(
        &ctx.device,
        &ctx.queue,
        &mut encoder,
        target.view(),
        OFFSCREEN_FORMAT,
        (W, H),
        1.0,
    );
    target.copy_out(&mut encoder);
    ctx.queue.submit(Some(encoder.finish()));
    let img = target.read(&ctx.device).expect("readback");
    assert!(!ctx.has_failed(), "wgpu reported a validation error");
    assert_eq!(stats.dropped, 0);
    if let Ok(dir) = std::env::var("FH6_KIT_DUMP") {
        let _ = img
            .over([24, 24, 28])
            .save_png(&std::path::Path::new(&dir).join(format!("{name}.png")));
    }
    img
}

fn setup() -> (GpuContext, Kit) {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
    let kit = Kit::new(&ctx.device, &ctx.queue, Tokens::default()).expect("kit");
    (ctx, kit)
}

/// Pixel at local coordinates (layout scale 0.5 around the centre).
fn at(img: &RgbaImage, local: Vec2) -> [u8; 4] {
    let x = (W as f32 * 0.5 + local.x * 0.5) as u32;
    let y = (H as f32 * 0.5 + local.y * 0.5) as u32;
    img.pixel(x, y).unwrap_or([0; 4])
}

#[test]
fn shapes_render_where_expected() {
    let (ctx, mut kit) = setup();
    let pts = [
        vec2(-700.0, 300.0),
        vec2(-600.0, 200.0),
        vec2(-500.0, 300.0),
        vec2(-400.0, 200.0),
    ];
    let tri = [vec2(500.0, 200.0), vec2(700.0, 200.0), vec2(600.0, 350.0)];
    let img = render_named(&mut kit, &ctx, "shapes", |p| {
        let white = Color::WHITE;
        p.fill(
            &Shape::round_rect(Rect::from_xywh(-800.0, -400.0, 300.0, 200.0), 40.0),
            white,
        );
        p.fill(&Shape::circle(vec2(0.0, -300.0), 80.0), Color::rgb_u32(0xFF2D3A));
        p.fill(
            &Shape::arc(vec2(400.0, -300.0), 90.0, 20.0, deg(180.0), deg(180.0), Cap::Round),
            white,
        );
        p.fill(&Shape::needle(vec2(-200.0, 50.0), 0.0, 200.0, 20.0, 14.0, 3.0), white);
        p.fill(
            &Shape::Polyline {
                points: &pts,
                width: 12.0,
                closed: false,
            },
            white,
        );
        p.fill(&Shape::Polygon { points: &tri }, white);
        p.fill(
            &Shape::Ticks(TickRing::new(vec2(200.0, 150.0), 120.0, deg(135.0), deg(270.0), 28).major(3, 30.0, 6.0)),
            white,
        );
        p.fill(
            &Shape::Chevron {
                center: vec2(0.0, 300.0),
                size: vec2(60.0, 80.0),
                thickness: 10.0,
                angle: 0.0,
            },
            white,
        );
        p.fill(
            &Shape::Bar(SegmentedBar {
                rect: Rect::from_xywh(-300.0, 380.0, 600.0, 30.0),
                count: 10,
                gap: 8.0,
                lit: 4.5,
                radius: 4.0,
                unlit: Color::WHITE.with_alpha8(22),
                axis: fh6_kit::Axis::Horizontal,
            }),
            Color::rgb_u32(0x2BE37A),
        );
        p.fill(
            &Shape::SevenSeg {
                rect: Rect::from_xywh(-700.0, -100.0, 60.0, 100.0),
                thickness: 12.0,
                mask: fh6_kit::shape::seven_seg_mask('8'),
            },
            white,
        );
        p.stroke(&Shape::circle(vec2(0.0, -300.0), 100.0), white, Stroke::rim_px(1.0));
    });
    assert_eq!(at(&img, vec2(-650.0, -300.0))[3], 255, "rounded rect centre is opaque");
    assert_eq!(at(&img, vec2(-799.0, -399.0))[3], 0, "rounded corner is cut");
    let red = at(&img, vec2(0.0, -300.0));
    assert!(
        red[0] > 240 && red[1] < 80 && red[3] == 255,
        "circle is #FF2D3A, got {red:?}"
    );
    assert!(at(&img, vec2(400.0, -390.0))[3] > 200, "arc top (12 o'clock) is drawn");
    assert_eq!(at(&img, vec2(400.0, -210.0))[3], 0, "arc bottom half is not drawn");
    assert!(at(&img, vec2(-100.0, 50.0))[3] > 200, "needle body");
    assert!(at(&img, vec2(600.0, 250.0))[3] > 200, "polygon interior");
    assert!(at(&img, vec2(-280.0, 395.0))[1] > 150, "first bar segment lit green");
    let unlit = at(&img, vec2(270.0, 395.0));
    assert!(
        unlit[3] > 0 && unlit[3] < 60,
        "last bar segment unlit ghost, got {unlit:?}"
    );
    assert!(at(&img, vec2(-670.0, -95.0))[3] > 200, "seven-seg top segment");
    assert!(img.covered() > 10_000);
}

#[test]
fn gradients_and_text() {
    let (ctx, mut kit) = setup();
    let font = kit.fonts().role(fh6_kit::FontRole::Display);
    let img = render_named(&mut kit, &ctx, "text", |p| {
        let g = Gradient::linear(vec2(-600.0, 0.0), vec2(600.0, 0.0))
            .stop(0.0, Color::rgb_u32(0xFF0000))
            .stop(1.0, Color::rgb_u32(0x0000FF));
        p.fill(&Shape::rect(Rect::from_xywh(-600.0, -400.0, 1200.0, 100.0)), g);
        let style = TextStyle::new(font, 160.0).tabular();
        p.text(vec2(0.0, 100.0), "188", &style);
    });
    let left = at(&img, vec2(-580.0, -350.0));
    let right = at(&img, vec2(580.0, -350.0));
    assert!(left[0] > 200 && left[2] < 60, "left is red: {left:?}");
    assert!(right[2] > 200 && right[0] < 60, "right is blue: {right:?}");
    // The text "188" (cap 160 around y = 100) covers a good share of its box, and nothing above.
    let in_box = (0..60)
        .flat_map(|i| (0..30).map(move |j| vec2(-180.0 + i as f32 * 6.0, 30.0 + j as f32 * 5.0)))
        .filter(|v| at(&img, *v)[3] > 128)
        .count();
    assert!(in_box > 250, "glyph coverage in the text box: {in_box}");
    assert_eq!(at(&img, vec2(0.0, -100.0))[3], 0, "no text above the cap line");
}

#[test]
fn glass_bloom_blur_and_effects() {
    let (ctx, mut kit) = setup();
    let img = render_named(&mut kit, &ctx, "glass", |p| {
        // Something behind the glass.
        p.fill(
            &Shape::rect(Rect::from_xywh(-500.0, -150.0, 1000.0, 60.0)),
            Color::rgb_u32(0x38BDF8),
        );
        p.layer(LayerOptions::glass(16.0), |p| {
            p.fill(
                &Shape::round_rect(Rect::from_xywh(-400.0, -300.0, 800.0, 400.0), 48.0),
                Glass::default(),
            );
        });
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(&Shape::circle(vec2(0.0, 250.0), 30.0), Color::WHITE);
        });
        p.layer(LayerOptions::blurred(20.0), |p| {
            p.fill(&Shape::rect(Rect::from_xywh(500.0, 200.0, 100.0, 100.0)), Color::WHITE);
        });
        p.glow(
            &Shape::circle(vec2(-600.0, 250.0), 30.0),
            Color::rgb_u32(0xFF2D3A),
            Glow::new(20.0, 1.0),
        );
        p.shadow(
            &Shape::round_rect(Rect::from_xywh(-800.0, -400.0, 100.0, 100.0), 10.0),
            Shadow::new(vec2(10.0, 10.0), 12.0, Color::BLACK.with_alpha(0.8)),
        );
    });
    let glass_mid = at(&img, vec2(0.0, -280.0));
    assert!(glass_mid[3] > 20, "glass tint covers its interior: {glass_mid:?}");
    let rim = at(&img, vec2(0.0, -299.0));
    assert!(rim[0] > glass_mid[0], "the top rim is brighter than the interior");
    let halo = at(&img, vec2(0.0, 250.0 - 60.0));
    assert!(halo[0] > 0, "bloom spreads light beyond the circle: {halo:?}");
    let blurred_edge = at(&img, vec2(500.0 - 20.0, 250.0));
    assert!(blurred_edge[3] > 0, "blur spreads the layer: {blurred_edge:?}");
    assert!(at(&img, vec2(-600.0, 250.0 - 50.0))[0] > 0, "outer glow");
    assert!(at(&img, vec2(-690.0, -290.0))[3] > 0, "shadow offset beyond the shape");
}

#[test]
fn warm_frames_do_not_grow_the_arena() {
    let (ctx, mut kit) = setup();
    let font = kit.fonts().role(fh6_kit::FontRole::Text);
    let draw = |p: &mut Painter<'_>| {
        for i in 0..200 {
            let a = i as f32 * 0.03;
            p.fill(&Shape::circle(Vec2::from_angle(a) * 300.0, 8.0), Color::WHITE);
        }
        p.number(vec2(0.0, 0.0), 118, 3, &TextStyle::new(font, 80.0), None);
    };
    render(&mut kit, &ctx, draw);
    let before = kit.scene().capacities();
    for _ in 0..5 {
        render(&mut kit, &ctx, draw);
    }
    assert_eq!(kit.scene().capacities(), before);
}

#[test]
fn recycled_upload_staging_draws_every_frame_exactly_like_write_buffer() {
    // Kit `a` hands its staging slots back after each submit (the live app's path), except on
    // frames 5 to 8: by frame 8 all three slots are out, so frames 8 and 9 fall back to
    // `Queue::write_buffer`, and frame 10 is a slot frame right after fallback frames. Kit `b`
    // never hands back, so from its fourth frame on it always falls back. Each frame moves the
    // shapes, so a slot that carried stale bytes would show as a difference.
    let (ctx, mut a) = setup();
    let mut b = Kit::new(&ctx.device, &ctx.queue, Tokens::default()).expect("kit");
    let font = a.fonts().role(fh6_kit::FontRole::Text);
    let mut previous: Option<RgbaImage> = None;
    for frame in 0..16u32 {
        let draw = |p: &mut Painter<'_>| {
            let shift = frame as f32 * 37.0;
            for i in 0..40 {
                let at = vec2(-700.0 + shift + i as f32 * 30.0, -200.0 + (i % 7) as f32 * 40.0);
                p.fill(&Shape::circle(at, 12.0), Color::rgb_u32(0x38BDF8));
            }
            p.number(
                vec2(0.0, 250.0),
                100 + i64::from(frame),
                3,
                &TextStyle::new(font, 80.0),
                None,
            );
        };
        let got = render(&mut a, &ctx, draw);
        if !(5..=8).contains(&frame) {
            a.frame_submitted();
        }
        let want = render(&mut b, &ctx, draw);
        assert!(got == want, "frame {frame}: the recycled upload drew something else");
        if let Some(prev) = &previous {
            assert!(prev != &got, "frame {frame} repeats the previous frame");
        }
        previous = Some(got);
    }
}

#[test]
fn dome_scrim_is_opaque_black_core_with_feather_and_void() {
    let (ctx, mut kit) = setup();
    // The image spans local +-960 x +-540, so this dome is the SCRIM_FIT shape at about 60%.
    let dome = fh6_kit::Dome {
        center: vec2(0.0, 0.0),
        semi_axes: vec2(400.0, 300.0),
        exponent_top: 2.6,
        exponent_bottom: 2.6,
        feather_top: 80.0,
        feather_side: 100.0,
        feather_bottom: 120.0,
        void_half_width: 60.0,
        void_top: 150.0,
        void_feather: 12.0,
    };
    let img = render_named(&mut kit, &ctx, "dome", |p| {
        p.fill(&Shape::Dome(dome), Color::BLACK);
    });
    let core = at(&img, vec2(0.0, 0.0));
    assert_eq!(core, [0, 0, 0, 255], "OLED core is pure black at alpha 255");
    assert_eq!(at(&img, vec2(330.0, 60.0))[3], 255, "still core near the sides");
    // Top edge at -300; half-way through the 80 feather is about half alpha.
    let mid = at(&img, vec2(0.0, -340.0))[3];
    assert!((90..=165).contains(&mid), "feather mid alpha {mid}");
    assert_eq!(at(&img, vec2(0.0, -390.0))[3], 0, "outside the feather");
    assert_eq!(at(&img, vec2(-900.0, 0.0))[3], 0, "A-pillar region clear");
    assert!(at(&img, vec2(0.0, 270.0))[3] < 8, "column void open");
    assert_eq!(at(&img, vec2(200.0, 250.0))[3], 255, "void is narrow");
    // Monotonic fall-off along the top (no banding steps back up beyond dither).
    let mut prev = 255i32;
    for i in 0..60 {
        let a = i32::from(at(&img, vec2(0.0, -296.0 - i as f32 * 2.0))[3]);
        assert!(a <= prev + 2, "alpha rises at step {i}: {prev} -> {a}");
        prev = a;
    }
}
