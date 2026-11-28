//! GPU tests of the kit: every primitive, text, glass, bloom and blur render without validation
//! errors and put pixels where they should.
#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "test helpers: a GPU setup or readback that fails should stop the test with its message"
)]

use fh6_core::CarDatabase;
use fh6_gpu::GpuContext;
use fh6_kit::logo::{self, CarLogo, LogoIndex, LogoStyle};
use fh6_kit::math::{vec3, Mat4};
use fh6_kit::mesh::{MeshData, MeshDraw, MeshShading};
use fh6_kit::perspective::Camera25;
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{
    deg, vec2, Affine, Cap, Color, Glass, Glow, Gradient, Kit, LayerOptions, Layout, Logo, Painter, Pool, Rect,
    SegmentedBar, Shadow, Shape, Stroke, TextStyle, TickRing, Tokens, Vec2,
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
fn text_under_the_least_size_is_left_out_but_keeps_its_bounds() {
    let (ctx, mut kit) = setup();
    let font = kit.fonts().role(fh6_kit::FontRole::Display);
    let style = TextStyle::new(font, 160.0).tabular();
    // Cap 160 local units at layout scale 0.5 is 80 target pixels.
    let mut bounds = [Rect::from_xywh(0.0, 0.0, 0.0, 0.0); 3];
    let mut inked = [0usize; 3];
    for (i, least) in [0.0, 60.0, 100.0].into_iter().enumerate() {
        // Twice: the first frame queues the glyphs.
        let mut img = None;
        for _ in 0..2 {
            img = Some(render(&mut kit, &ctx, |p| {
                p.save();
                p.set_min_text_px(least);
                bounds[i] = p.text(vec2(0.0, 100.0), "188", &style);
                p.restore();
                // Restore lifts the limit again.
                p.text(vec2(0.0, -150.0), "1", &style);
            }));
        }
        let img = img.expect("rendered");
        inked[i] = (0..60)
            .flat_map(|x| (0..30).map(move |y| vec2(-180.0 + x as f32 * 6.0, 30.0 + y as f32 * 5.0)))
            .filter(|v| at(&img, *v)[3] > 128)
            .count();
        let after = (0..20)
            .flat_map(|x| (0..20).map(move |y| vec2(-60.0 + x as f32 * 6.0, -220.0 + y as f32 * 6.0)))
            .filter(|v| at(&img, *v)[3] > 128)
            .count();
        assert!(after > 20, "text after the restore draws (least {least}): {after}");
    }
    assert!(
        inked[0] > 250 && inked[1] > 250,
        "over the least size it draws: {inked:?}"
    );
    assert_eq!(inked[2], 0, "under it nothing draws");
    assert_eq!(bounds[0], bounds[2], "the bounds stay the same either way");
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
fn a_blurred_layer_looks_the_same_when_far_draws_widen_the_work_rectangle() {
    // A blurred layer is blurred and composited only round what it drew, so a dot in the far
    // corner, which stretches the work rectangle over most of the target, changes nothing.
    let (ctx, mut kit) = setup();
    let layer = |p: &mut Painter<'_>| {
        p.layer(LayerOptions::blurred(20.0), |p| {
            p.fill(&Shape::rect(Rect::from_xywh(500.0, 200.0, 100.0, 100.0)), Color::WHITE);
        });
    };
    let alone = render_named(&mut kit, &ctx, "blur_alone", layer);
    let widened = render_named(&mut kit, &ctx, "blur_widened", |p| {
        p.fill(&Shape::circle(vec2(-850.0, -450.0), 10.0), Color::WHITE);
        layer(p);
    });
    let mut worst = 0u8;
    for y in 250..H {
        for x in 600..W {
            let (a, b) = (
                alone.pixel(x, y).unwrap_or([0; 4]),
                widened.pixel(x, y).unwrap_or([0; 4]),
            );
            for (ca, cb) in a.iter().zip(b.iter()) {
                worst = worst.max(ca.abs_diff(*cb));
            }
        }
    }
    assert!(worst <= 2, "the far dot changed the blurred layer by {worst}");
    assert!(
        at(&widened, vec2(500.0 - 20.0, 250.0))[3] > 0,
        "blur still spreads the layer"
    );
    assert!(at(&widened, vec2(-850.0, -450.0))[3] > 200, "the far dot is drawn");
}

#[test]
fn a_mesh_in_a_blurred_layer_spreads_the_bounds_as_a_shape_does() {
    // The renderer blurs and composites a layer only inside the bounds its draws recorded, so a
    // mesh has to record the same three sigma of spread a shape does.
    let (ctx, mut kit) = setup();
    let car = kit.upload_mesh(&ctx.device, &ctx.queue, &MeshData::car_proxy(4.6, 1.9, 1.3));
    let layout = Layout::centered(vec2(W as f32 * 0.5, H as f32 * 0.5), 0.5, (W, H));
    let viewport = Rect::from_xywh(-200.0, -100.0, 400.0, 200.0);
    let cam = Camera25::adas(900.0, 900.0, 0.0, 0.0);
    let car = MeshDraw {
        mesh: car,
        model: Mat4::translation(vec3(0.0, 0.0, 3.2)),
        view_proj: cam.view_proj(&viewport),
        eye: cam.eye,
        viewport,
        color: Color::WHITE,
        rim: Color::WHITE,
        light: vec3(0.3, 1.0, -0.4),
        shading: MeshShading::Matcap,
    };
    let mut bounds_in = |layer: LayerOptions| {
        {
            let mut p = kit.begin_frame(&layout);
            p.layer(layer, |p| p.mesh(&car));
        }
        kit.scene().bounds()
    };
    let plain = bounds_in(LayerOptions::default());
    let blurred = bounds_in(LayerOptions::blurred(20.0));
    // The viewport at this layout's half scale, round the centre of the target.
    assert_eq!(plain, Rect::from_xywh(380.0, 220.0, 200.0, 100.0));
    // Sigma 20 in local units is 10 px here: 30 px of spread on every side.
    assert_eq!(blurred, plain.expand(30.0));
}

/// Two pools that overlap a little, as `Kit::set_pools` takes them (target pixels).
const POOLS: [Pool; 2] = [
    Pool::Ellipse {
        center: Vec2 { x: 330.0, y: 220.0 },
        radii: Vec2 { x: 200.0, y: 125.0 },
        alpha: 0.45,
    },
    Pool::RoundRect {
        rect: Rect {
            min: Vec2 { x: 580.0, y: 95.0 },
            max: Vec2 { x: 830.0, y: 395.0 },
        },
        radius: 20.0,
        sigma: 30.0,
        alpha: 0.78,
    },
];

/// The same two pools drawn with the painter, in the test layout's local units (half scale
/// round the centre): a radial gradient on an ellipse and a shadow.
fn paint_pools(p: &mut Painter<'_>) {
    p.save();
    let world = p.current_transform();
    p.set_transform(
        Affine::scale_xy(400.0, 250.0)
            .then(&Affine::translate(vec2(-300.0, -100.0)))
            .then(&world),
    );
    p.fill(
        &Shape::circle(Vec2::ZERO, 1.0),
        Gradient::radial(Vec2::ZERO, 1.0)
            .stop(0.0, Color::BLACK.with_alpha(0.45))
            .stop(0.55, Color::BLACK.with_alpha(0.45 * 0.75))
            .stop(1.0, Color::BLACK.with_alpha(0.0)),
    );
    p.restore();
    p.shadow(
        &Shape::round_rect(Rect::from_xywh(200.0, -350.0, 500.0, 600.0), 40.0),
        Shadow {
            offset: Vec2::ZERO,
            sigma: 60.0,
            spread: 0.0,
            color: Color::BLACK.with_alpha(0.78),
        },
    );
}

/// The largest difference between two renders in any channel of any pixel, and where it is.
fn worst_difference(a: &RgbaImage, b: &RgbaImage) -> (u8, u32, u32) {
    assert_eq!((a.width, a.height), (b.width, b.height));
    let mut worst = (0, 0, 0);
    for y in 0..a.height {
        for x in 0..a.width {
            let (pa, pb) = (a.pixel(x, y).unwrap_or([0; 4]), b.pixel(x, y).unwrap_or([0; 4]));
            let d = pa.iter().zip(&pb).map(|(u, v)| u.abs_diff(*v)).max().unwrap_or(0);
            if d > worst.0 {
                worst = (d, x, y);
            }
        }
    }
    worst
}

#[test]
fn pools_are_the_same_picture_as_painting_them_first() {
    // Opaque, translucent, glowing, blurred and glass content, over the pools and beside them.
    // Translucent light is encoded by the alpha under it and glass reads the alpha of what it
    // bends, so each of these would show a pool that was not truly the scene's first layer.
    let content = |p: &mut Painter<'_>| {
        p.fill(
            &Shape::rect(Rect::from_xywh(-700.0, -160.0, 1500.0, 70.0)),
            Color::rgb_u32(0x38BDF8).with_alpha(0.35),
        );
        p.fill(&Shape::circle(vec2(420.0, 60.0), 70.0), Color::WHITE);
        p.glow(
            &Shape::circle(vec2(-420.0, 120.0), 30.0),
            Color::rgb_u32(0xFF2D3A),
            Glow::new(24.0, 1.0),
        );
        p.layer(LayerOptions::blurred(12.0), |p| {
            p.fill(
                &Shape::rect(Rect::from_xywh(120.0, 150.0, 160.0, 60.0)),
                Color::WHITE.with_alpha(0.6),
            );
        });
        p.layer(LayerOptions::glass(8.0), |p| {
            p.fill(
                &Shape::round_rect(Rect::from_xywh(-560.0, -260.0, 1200.0, 240.0), 48.0),
                Glass::default(),
            );
        });
        // Bloom whose faint halo crosses the ellipse's edge but stays clear of the rectangle's:
        // a painted shadow stops three sigmas out, and faint light shows that edge (see
        // `a_faint_halo_does_not_show_where_a_pool_ends`).
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(&Shape::circle(vec2(-600.0, 300.0), 24.0), Color::WHITE);
        });
    };
    let (ctx, mut kit) = setup();
    let painted = render_named(&mut kit, &ctx, "pools_painted", |p| {
        paint_pools(p);
        content(p);
    });
    kit.set_pools(&POOLS);
    let pooled = render_named(&mut kit, &ctx, "pools_set", content);
    let (worst, x, y) = worst_difference(&painted, &pooled);
    assert!(
        worst <= 2,
        "pools set on the kit differ from pools painted first by {worst} at ({x}, {y}): {:?} painted, {:?} set",
        painted.pixel(x, y),
        pooled.pixel(x, y)
    );
    // The pools are there: black at their core alpha, nothing where neither reaches.
    let core = pooled.pixel(760, 300).unwrap_or([0; 4]);
    assert_eq!(&core[..3], &[0, 0, 0], "a pool adds no colour: {core:?}");
    let want = (POOLS[1].alpha_at(vec2(760.5, 300.5)) * 255.0).round() as u8;
    assert!(want > 190, "deep inside the rectangle: {want}");
    assert!(
        core[3].abs_diff(want) <= 1,
        "{want} deep inside the rectangle: {core:?}"
    );
    assert_eq!(pooled.pixel(40, 500), Some([0; 4]), "clear outside every pool");

    // They stay until they are cleared, and an empty slice clears them.
    let again = render_named(&mut kit, &ctx, "pools_again", content);
    assert_eq!(
        worst_difference(&pooled, &again).0,
        0,
        "pools stay set from frame to frame"
    );
    kit.set_pools(&[]);
    let cleared = render_named(&mut kit, &ctx, "pools_cleared", content);
    assert_eq!(
        cleared.pixel(760, 300),
        Some([0; 4]),
        "a cleared pool leaves nothing behind"
    );
    // A kit that never had pools draws the same frame. Its work targets are smaller, so its
    // blur and bloom pyramids fall on other texels: the same picture within the usual 2.
    let bare = {
        let (ctx, mut fresh) = setup();
        render_named(&mut fresh, &ctx, "pools_never", content)
    };
    let (worst, x, y) = worst_difference(&cleared, &bare);
    assert!(worst <= 2, "cleared pools left {worst} behind at ({x}, {y})");
}

#[test]
fn a_faint_halo_does_not_show_where_a_pool_ends() {
    // The resolve encodes light up to the alpha under it as coverage, so an alpha of 0.001
    // holds back a bloom halo of 3/255 and the halo steps up where that alpha stops. A pool's
    // bounds reach far enough out that nothing is left to hold the halo back.
    let (ctx, mut kit) = setup();
    kit.set_pools(&POOLS[1..]);
    let edge = POOLS[1].bounds().min.x as u32;
    let img = render_named(&mut kit, &ctx, "pools_halo", |p| {
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(&Shape::circle(vec2(-240.0, -50.0), 24.0), Color::WHITE);
        });
    });
    let mut lit = 0;
    for y in 150..340 {
        let (out, inside) = (
            img.pixel(edge - 1, y).unwrap_or([0; 4]),
            img.pixel(edge, y).unwrap_or([0; 4]),
        );
        lit += u32::from(out[0] >= 2);
        let step = out.iter().zip(&inside).map(|(a, b)| a.abs_diff(*b)).max().unwrap_or(0);
        assert!(
            step <= 1,
            "the pool's bounds show at y {y}: {out:?} outside, {inside:?} inside"
        );
    }
    assert!(lit > 20, "the halo reaches the pool's edge ({lit} rows)");
}

#[test]
fn pools_show_under_an_empty_frame_and_match_their_formula() {
    let (ctx, mut kit) = setup();
    kit.set_pools(&POOLS);
    let img = render_named(&mut kit, &ctx, "pools_alone", |_| {});
    let mut worst = 0u8;
    for y in (0..H).step_by(3) {
        for x in (0..W).step_by(3) {
            let centre = vec2(x as f32 + 0.5, y as f32 + 0.5);
            let inside = POOLS.iter().any(|pool| pool.bounds().contains(centre));
            let want = if inside {
                (Pool::combined_alpha(&POOLS, centre) * 255.0).round() as u8
            } else {
                0
            };
            let got = img.pixel(x, y).unwrap_or([0; 4]);
            assert_eq!(&got[..3], &[0, 0, 0], "pools are black at ({x}, {y})");
            worst = worst.max(got[3].abs_diff(want));
        }
    }
    assert!(worst <= 1, "the pool pass is off its formula by {worst}");
    // Both pools reach this pixel: together they are darker than either alone.
    let both = vec2(520.5, 220.5);
    let (a, b) = (POOLS[0].alpha_at(both), POOLS[1].alpha_at(both));
    assert!(a > 0.02 && b > 0.01, "the test pools overlap here: {a} {b}");
    let got = f32::from(img.pixel(520, 220).unwrap_or([0; 4])[3]) / 255.0;
    assert!(
        (got - (a + b * (1.0 - a))).abs() < 0.006,
        "overlap: {got} against {a} and {b}"
    );
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

/// A file loads once: the app builds a cluster again on every view or theme change, and the
/// second build must get the textures it already has, not a second upload of each file.
#[test]
fn a_file_loads_once_until_it_changes() {
    let (ctx, mut kit) = setup();
    let dir = std::env::temp_dir().join(format!("fh6-kit-load-once-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("tile.png");
    let image = |w: u32| RgbaImage {
        width: w,
        height: 2,
        pixels: vec![200; (w * 2 * 4) as usize],
    };
    image(2).save_png(&path).expect("write png");
    let first = kit
        .images_mut()
        .load_file(&ctx.device, &ctx.queue, &path, None)
        .expect("load");
    let count = kit.images().len();
    let again = kit
        .images_mut()
        .load_file(&ctx.device, &ctx.queue, &path, None)
        .expect("load again");
    assert_eq!(first, again, "the same file gives the same image");
    assert_eq!(kit.images().len(), count, "and uploads nothing new");
    // A changed file (here a different length) is read again.
    image(4).save_png(&path).expect("rewrite png");
    let changed = kit
        .images_mut()
        .load_file(&ctx.device, &ctx.queue, &path, None)
        .expect("load changed");
    assert_ne!(changed, first);
    assert_eq!(kit.images().size(changed), Some((4, 2)));
    let _ = std::fs::remove_dir_all(&dir);
}

/// The brand logo path the app takes on a car change: the index names the car's brand, its dial
/// is decoded without a device (the app does that on its worker thread) and uploaded from 512 px
/// down, and the logo zone draws that image instead of the wordmark.
#[test]
fn a_cars_brand_logo_decodes_uploads_and_draws() {
    let (ctx, mut kit) = setup();
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let index = LogoIndex::load(&assets).expect("assets/logos/logos.json");
    let cars = CarDatabase::load(assets.join("car_brands/car_brands.json")).expect("car_brands.json");
    // The 2021 Porsche 911 GT3, and the 2019 911 (992) of the same brand.
    let porsche = index.car_brand(&cars, 3667).logo.expect("the GT3 has a brand logo");
    assert_eq!(index.car_brand(&cars, 3123).logo, Some(porsche), "one brand");
    let chain = index.decode_dial(porsche).expect("the dial decodes");
    let image = kit
        .images_mut()
        .upload_mips(&ctx.device, &ctx.queue, &chain, "PORSCHE")
        .expect("the dial uploads");
    let size = kit.images().size(image).expect("a loaded image has a size");
    assert_eq!(size, chain.size());
    assert!(size.0.max(size.1) <= logo::DIAL_TEXTURE_SIDE, "dial size {size:?}");
    let source = kit.images().source_size(image).expect("and its picture's size");
    assert_eq!(source, chain.source_size(), "the dial file's size, for the layout");
    assert_eq!(
        kit.images().mip_count(image) as usize,
        chain.len(),
        "every level down to 1 x 1"
    );

    let zone = Rect::from_center_size(vec2(0.0, 0.0), vec2(600.0, 200.0));
    let style = LogoStyle::new(kit.font_choice(None).wide);
    let shown = Logo {
        image: Some(image),
        brand_name: "Porsche".into(),
        custom_name: "TJX".into(),
        loading: false,
    };
    let mut used = Rect::from_center_size(vec2(0.0, 0.0), Vec2::ZERO);
    let img = render_named(&mut kit, &ctx, "brand_logo", |p| {
        used = logo::draw(p, zone, &shown, &style);
    });
    assert_eq!(used, logo::fit(zone, source), "the image path, not the wordmark");
    let lit = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|&(x, y)| img.pixel(x, y).is_some_and(|px| px[3] > 128))
        .count();
    assert!(lit > 500, "the logo puts pixels on screen ({lit})");
}

/// A dial uploaded from 512 px down, a slice at a time, draws within 1 of 255 per channel of its
/// full 1024 px chain uploaded a level at a time: the kit picks the mip level from the drawn size, so
/// the dropped level is never sampled, and the zone is laid out by the dial file's size, so a
/// dial with an odd side keeps its shape. Checked in Liquid Glass's 240 x 48 zone at user scale
/// 1.0 and 1.4 (the widest use) and PorscheAnalog's 160 x 20 zone, in 4K pixels; in a 500 px
/// zone, which samples the largest levels (the ones that go up in several slices); and in two
/// very small zones, which sample the smallest. The dials are a wordmark, a roundel, the widest
/// wordmark, one with an odd height (1024 x 95), one whose largest level takes four slices
/// (1024 x 411) and a tall one with an odd width (955 x 1024).
#[test]
fn a_dial_from_512_px_down_draws_the_same_pixels_as_the_full_chain() {
    let (ctx, mut kit) = setup();
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let index = LogoIndex::load(&assets).expect("assets/logos/logos.json");
    let style = LogoStyle::new(kit.font_choice(None).wide);
    for key in ["PORSCHE", "BMW", "BAC", "HENNESSEY", "BMW_M", "ABARTH"] {
        let brand = index
            .resolve(key, "")
            .unwrap_or_else(|| panic!("{key} is in the index"));
        let path = index.dial_path(brand).expect("a dial path");
        let bytes = std::fs::read(&path).expect("the dial file");
        let full = kit
            .images_mut()
            .load_png(&ctx.device, &ctx.queue, &bytes, key)
            .expect("the full chain");
        let capped = index
            .load_dial(kit.images_mut(), &ctx.device, &ctx.queue, brand)
            .expect("the 512 px chain");
        assert_eq!(
            kit.images().mip_count(full),
            kit.images().mip_count(capped) + 1,
            "{key}: only the 1024 px level is dropped"
        );
        assert_eq!(
            kit.images().source_size(capped),
            kit.images().size(full),
            "{key}: laid out by the dial file's size"
        );
        // Zone sizes in 4K pixels; the test layout draws half a pixel per local unit.
        for (w, h) in [
            (240.0_f32, 48.0_f32),
            (336.0, 67.2),
            (160.0, 20.0),
            (500.0, 500.0),
            (40.0, 8.0),
            (10.0, 2.0),
        ] {
            let zone = Rect::from_center_size(vec2(0.0, 0.0), vec2(w * 2.0, h * 2.0));
            let only = |image| Logo {
                image: Some(image),
                ..Logo::default()
            };
            let (a_logo, b_logo) = (only(full), only(capped));
            let a = render_named(&mut kit, &ctx, &format!("dial_full_{key}_{w}"), |p| {
                logo::draw(p, zone, &a_logo, &style);
            });
            let b = render_named(&mut kit, &ctx, &format!("dial_512_{key}_{w}"), |p| {
                logo::draw(p, zone, &b_logo, &style);
            });
            let worst = a
                .pixels
                .iter()
                .zip(&b.pixels)
                .map(|(x, y)| x.abs_diff(*y))
                .max()
                .unwrap_or(0);
            assert!(worst <= 1, "{key} in a {w}x{h} zone: a channel differs by {worst}");
            // The two smallest zones are a few pixels of logo, and only there for their levels.
            if w >= 100.0 {
                assert!(a.covered() > 20, "{key} in a {w}x{h} zone puts pixels on screen");
            }
        }
    }
}

/// The live path, as the app runs it: `CarLogo` asks the worker thread for the car's dial, the
/// zone is empty until it is in (the names are set, marked loading, and draw nothing), and the upload is spread over the frames that poll it, each
/// frame's writes submitted before the next. The dial that shows is the same picture as one
/// decoded and uploaded at once.
#[test]
fn a_cars_logo_loads_on_the_worker_and_uploads_over_frames() {
    let (ctx, mut kit) = setup();
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
    let index = LogoIndex::load(&assets).expect("assets/logos/logos.json");
    let cars = CarDatabase::load(assets.join("car_brands/car_brands.json")).expect("car_brands.json");
    let style = LogoStyle::new(kit.font_choice(None).wide);
    let zone = Rect::from_center_size(vec2(0.0, 0.0), vec2(1000.0, 1000.0));
    let mut logo = CarLogo::start(&index);
    // A 2015 BMW i8 (a square dial, the largest upload), a 2025 BMW M4 CS (an odd height) and a
    // 2017 Abarth 124 Spider (a tall dial, whose rows are not a multiple of 256 bytes).
    for ordinal in [2133, 4122, 2740] {
        let brand = index.car_brand(&cars, ordinal);
        let id = brand.logo.expect("a branded car");
        let name = brand.name.clone();
        let before = kit.images().len();
        logo.show(Some(brand), "TJX");
        assert!(logo.busy(), "{name}: the dial is on its way");
        // The worker is decoding. This thread does the same work before it polls, so that the
        // polls counted below are the upload's and not the wait for the decode.
        let chain = index.decode_dial(id).expect("the dial decodes");
        let waiting = Logo {
            image: None,
            brand_name: name.clone(),
            custom_name: "TJX".into(),
            loading: true,
        };
        assert_eq!(*logo.logo(), waiting, "{name}: the names, marked loading");
        // With the names set and the dial on its way, the zone draws nothing: not the name
        // first and the dial a moment later.
        let empty = render_named(&mut kit, &ctx, &format!("car_logo_loading_{ordinal}"), |p| {
            logo::draw(p, zone, &waiting, &style);
        });
        assert_eq!(empty.covered(), 0, "{name}: nothing in the zone while the dial loads");
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        let mut polls = 0;
        while logo.busy() && std::time::Instant::now() < until {
            assert_eq!(*logo.logo(), waiting, "{name}: the zone is empty until the dial is in");
            assert_eq!(kit.images().len(), before, "{name}: and no image is in the store");
            logo.poll(kit.images_mut(), &ctx.device, &ctx.queue);
            // The frame ends: what this poll wrote goes to the device before the next poll.
            ctx.queue.submit(None);
            polls += 1;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(!logo.busy(), "{name}: loaded within 30 s");
        // No machine copies a megabyte in the 0.05 ms a poll has, so the largest dial always
        // takes more than one frame.
        if chain.byte_len() > 1024 * 1024 {
            assert!(polls > 1, "{name}: the whole dial went up in one poll");
        }
        let live = logo.logo().clone();
        let image = live.image.expect("the dial shows");
        assert_eq!(
            (live.brand_name.as_str(), live.custom_name.as_str()),
            (name.as_str(), "TJX")
        );
        assert_eq!(kit.images().len(), before + 1, "{name}: one image");
        let at_once = index
            .load_dial(kit.images_mut(), &ctx.device, &ctx.queue, id)
            .expect("the dial loads at once");
        assert_eq!(kit.images().size(image), kit.images().size(at_once));
        assert_eq!(kit.images().source_size(image), kit.images().source_size(at_once));
        assert_eq!(kit.images().mip_count(image), kit.images().mip_count(at_once));
        let reference = Logo {
            image: Some(at_once),
            ..Logo::default()
        };
        let a = render_named(&mut kit, &ctx, &format!("car_logo_live_{ordinal}"), |p| {
            logo::draw(p, zone, &live, &style);
        });
        let b = render_named(&mut kit, &ctx, &format!("car_logo_at_once_{ordinal}"), |p| {
            logo::draw(p, zone, &reference, &style);
        });
        assert!(
            a.pixels == b.pixels,
            "{name}: the same pixels as the dial uploaded at once"
        );
        assert!(a.covered() > 500, "{name} puts pixels on screen");
        // The next car of the brand has it at once, with no new image.
        logo.show(Some(index.car_brand(&cars, ordinal)), "TJX");
        assert_eq!(logo.logo().image, Some(image));
        assert!(!logo.busy());
    }
}
