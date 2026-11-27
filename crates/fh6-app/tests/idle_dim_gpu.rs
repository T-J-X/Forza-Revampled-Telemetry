//! The OLED idle dim on a real GPU: `render_loop::dim_dash` drawn with the kit and read back.
//!
//! This test needs a GPU (or WARP), so running it is a GPU job (SKILLS.md section 5).
use fh6_app::render_loop::dim_dash;
use fh6_gpu::GpuContext;
use fh6_kit::color::srgb_to_linear;
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{vec2, Color, Kit, LayerOptions, Layout, Rect, Shape, Tokens, Vec2};

const SIZE: (u32, u32) = (960, 540);
/// The default `OledDimmerMinAlpha`.
const DIM: f32 = 0.35;

/// One 8-bit premultiplied pixel as (light, coverage): its green channel decoded to linear
/// light, and its alpha.
fn light(pixel: [u8; 4]) -> (f32, f32) {
    let alpha = f32::from(pixel[3]) / 255.0;
    if alpha == 0.0 {
        return (0.0, 0.0);
    }
    (srgb_to_linear(f32::from(pixel[1]) / 255.0 / alpha), alpha)
}

#[test]
fn idle_dim_turns_down_the_light_and_keeps_the_coverage() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
    let mut kit = Kit::new(&ctx.device, &ctx.queue, Tokens::default()).expect("the kit starts");
    let layout = Layout::centered(vec2(480.0, 270.0), 0.5, SIZE);
    // Everything but the lamp on the far right is inside the dimmed area.
    let area = Rect::from_points(vec2(-900.0, -500.0), vec2(650.0, 500.0));
    let rect = |x, y, w, h| Shape::rect(Rect::from_xywh(x, y, w, h));
    let mut shot = |dim: f32, bloom: bool| {
        kit.set_bloom(bloom);
        {
            let mut p = kit.begin_frame(&layout);
            // A black scrim, an opaque grey plate, a lamp on the plate, a dark shade over half
            // the lamp, a half-clear pane over nothing, a lamp that blooms and a lamp outside
            // the dimmed area.
            p.fill(&rect(-800.0, -150.0, 200.0, 300.0), Color::BLACK);
            p.fill(&rect(-500.0, -150.0, 400.0, 300.0), Color::rgb_u32(0x80_80_80));
            p.fill(&rect(-400.0, -60.0, 200.0, 120.0), Color::WHITE);
            p.fill(&rect(-300.0, -60.0, 100.0, 120.0), Color::BLACK.with_alpha(0.8));
            p.fill(&rect(0.0, -150.0, 200.0, 300.0), Color::WHITE.with_alpha(0.5));
            p.layer(LayerOptions::bloom(), |p| {
                p.fill(&rect(300.0, -150.0, 300.0, 300.0), Color::WHITE);
            });
            p.fill(&rect(700.0, 250.0, 150.0, 150.0), Color::WHITE);
            dim_dash(&mut p, area, dim);
        }
        let target = Offscreen::new(&ctx.device, SIZE.0, SIZE.1);
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        kit.render(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            target.view(),
            OFFSCREEN_FORMAT,
            SIZE,
            1.0,
        );
        target.copy_out(&mut encoder);
        ctx.queue.submit(Some(encoder.finish()));
        target.read(&ctx.device).expect("the frame reads back")
    };
    // The render loop draws a dimmed frame without bloom; `glowing` is what bloom would add.
    let (bright, dimmed, glowing) = (shot(1.0, false), shot(DIM, false), shot(DIM, true));
    assert!(!ctx.has_failed(), "wgpu reported a validation error");
    let at = |local: Vec2| {
        let px = layout.to_target(local);
        (px.x as u32, px.y as u32)
    };
    let left = srgb_to_linear(DIM);

    // Each row: what is there, its place in local units, whether it gives off light.
    for (what, local, lit) in [
        ("the scrim", vec2(-700.0, 0.0), false),
        ("the plate", vec2(-450.0, 100.0), true),
        ("the lamp", vec2(-350.0, 0.0), true),
        ("the shaded half of the lamp", vec2(-250.0, 0.0), true),
        ("the half-clear pane", vec2(100.0, 0.0), true),
        ("the lamp that blooms", vec2(450.0, 0.0), true),
        ("nothing", vec2(-100.0, -300.0), false),
    ] {
        let (x, y) = at(local);
        let (before, alpha_before) = light(bright.pixel(x, y).expect("inside the frame"));
        let (after, alpha_after) = light(dimmed.pixel(x, y).expect("inside the frame"));
        assert!(
            (alpha_after - alpha_before).abs() < 0.01,
            "{what}: coverage went from {alpha_before} to {alpha_after}"
        );
        if lit {
            assert!(before > 0.02, "{what} should be lit, got {before}");
            let share = after / before;
            assert!(
                (share / left - 1.0).abs() < 0.1,
                "{what}: {share} of its light is left, expected {left}"
            );
        } else {
            assert_eq!((before, after), (0.0, 0.0), "{what} gives off no light");
        }
    }

    // White lands where white at 35% opacity would: 0.35 x 255 on screen.
    let (x, y) = at(vec2(-350.0, 0.0));
    let lamp = dimmed.pixel(x, y).expect("inside the frame");
    assert!(lamp[1].abs_diff(89) <= 1, "the dimmed lamp reads {lamp:?}");

    // Outside the area nothing changes.
    let (x, y) = at(vec2(775.0, 325.0));
    assert_eq!(dimmed.pixel(x, y), bright.pixel(x, y), "the lamp outside the area");

    // Why a dimmed frame has no bloom: the dim cannot reach it, so with bloom the lamp that
    // blooms would stay several times brighter than the rest of the dash.
    let (x, y) = at(vec2(450.0, 0.0));
    let (with_bloom, _) = light(glowing.pixel(x, y).expect("inside the frame"));
    assert!(
        with_bloom > 2.0 * left,
        "bloom was dimmed after all ({with_bloom}): the render loop can keep it on"
    );
}
