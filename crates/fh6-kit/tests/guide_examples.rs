//! Every example in `docs/kit/README.md`, compiled and rendered, so the guide cannot drift from
//! the code. Section numbers match the guide.
#![cfg(not(target_arch = "wasm32"))]

use fh6_gpu::GpuContext;
use fh6_kit::fluid::{Fluid, FluidConfig};
use fh6_kit::mesh::{MeshData, MeshDraw, MeshShading};
use fh6_kit::particles::{Emitter, EmitterConfig};
use fh6_kit::perspective::{Camera25, RoadCurve};
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{
    deg, vec2, Align, BlendMode, Cap, Cluster, Color, FrameInput, Glass, Glow, Gradient, Hysteresis, ImagePaint, Kit,
    LayerOptions, Layout, Painter, Piece, Radii, Rect, SegmentedBar, Shadow, Shape, Smoothed, Spring, Stroke,
    TextStyle, TickRing, Tokens, VAlign, Vec2,
};

// 2. Shapes ------------------------------------------------------------------------------------
fn shapes(p: &mut Painter<'_>) {
    let pal = *p.palette();
    // Rounded rect, per-corner radii.
    let pod = Rect::from_xywh(-200.0, -60.0, 400.0, 120.0);
    p.fill(
        &Shape::round_rect(pod, Radii::new(24.0, 24.0, 8.0, 8.0)),
        pal.roles.ink_ghost,
    );
    // A 1 px specular rim, exactly one device pixel at any scale.
    p.stroke(
        &Shape::round_rect(pod, 24.0),
        pal.roles.rim_specular,
        Stroke::rim_px(1.0),
    );
    // Arc: tach track 135° -> 405°, round caps.
    p.fill(
        &Shape::arc(Vec2::ZERO, 260.0, 18.0, deg(135.0), deg(270.0), Cap::Round),
        pal.roles.ink_ghost,
    );
    // Tick ring: 37 ticks, every 4th major.
    p.fill(
        &Shape::Ticks(TickRing::new(Vec2::ZERO, 300.0, deg(135.0), deg(270.0), 37).major(4, 34.0, 5.0)),
        pal.roles.ink,
    );
    // Tapered needle (tokens: 14 px base, 3 px tip) and its hub.
    p.fill(
        &Shape::needle(Vec2::ZERO, deg(200.0), 240.0, 40.0, 14.0, 3.0),
        pal.primary_text,
    );
    p.fill(&Shape::circle(Vec2::ZERO, 24.0), pal.roles.ink_muted);
    // Polyline and polygon.
    let pts = [vec2(-300.0, 200.0), vec2(-200.0, 150.0), vec2(-100.0, 190.0)];
    p.fill(
        &Shape::Polyline {
            points: &pts,
            width: 6.0,
            closed: false,
        },
        pal.accent,
    );
    p.fill(&Shape::Polygon { points: &pts }, pal.secondary.with_alpha(0.3));
    // Chevron and a segmented LED bar lit to 6.5 of 12.
    p.fill(
        &Shape::Chevron {
            center: vec2(320.0, 0.0),
            size: vec2(40.0, 60.0),
            thickness: 8.0,
            angle: 0.0,
        },
        pal.accent,
    );
    p.fill(
        &Shape::Bar(SegmentedBar {
            rect: Rect::from_xywh(-300.0, 240.0, 600.0, 24.0),
            count: 12,
            gap: 6.0,
            lit: 6.5,
            radius: 4.0,
            unlit: pal.roles.ink_ghost,
            axis: fh6_kit::Axis::Horizontal,
        }),
        pal.roles.go,
    );
    // Glow and drop shadow from the same shape.
    let orb = Shape::circle(vec2(0.0, -200.0), 20.0);
    p.shadow(
        &orb,
        Shadow::new(vec2(4.0, 6.0), 8.0, pal.roles.oled_black.with_alpha(0.6)),
    );
    p.glow(&orb, pal.accent, Glow::new(16.0, 0.6));
    p.fill(&orb, pal.roles.ink);
}

// 3. Paint -------------------------------------------------------------------------------------
fn paints(p: &mut Painter<'_>, image: fh6_kit::ImageId) {
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
        Gradient::radial(Vec2::ZERO, 120.0)
            .stop(0.0, pal.roles.ink)
            .stop(1.0, pal.primary.with_alpha(0.0)),
    );
    p.fill(
        &Shape::ring(Vec2::ZERO, 200.0, 12.0),
        Gradient::conic(Vec2::ZERO, deg(-90.0))
            .stop(0.0, pal.primary)
            .stop(1.0, pal.accent),
    );
    let logo = Rect::from_xywh(-60.0, 100.0, 120.0, 60.0);
    p.fill(
        &Shape::round_rect(logo, 8.0),
        ImagePaint::new(image, logo).tint(pal.roles.ink),
    );
    p.scoped(|p| {
        p.set_blend(BlendMode::Add);
        p.fill(&Shape::circle(vec2(150.0, 0.0), 60.0), pal.accent.with_alpha(0.3));
    });
}

// 4. Light and material ------------------------------------------------------------------------
fn light(p: &mut Painter<'_>) {
    let pal = *p.palette();
    // Light that lives under the glass: the pieces bend it.
    p.fill(
        &Shape::arc(Vec2::ZERO, 150.0, 16.0, deg(135.0), deg(200.0), Cap::Round),
        pal.primary,
    );
    // Optical glass: a dome and a pill that merge like liquid. It bends the scene as it was
    // when its layer began, so draw the layers it should bend before the layer.
    let mut glass = p.tokens().optical_glass(&pal);
    glass.aim(deg(-135.0), deg(40.0), Vec2::ZERO);
    let pieces = [
        Piece::circle(Vec2::ZERO, 190.0).dome(110.0, 12.0),
        Piece::pill(Rect::from_xywh(150.0, -40.0, 320.0, 80.0)).section(18.0, 26.0, 8.0),
    ];
    p.layer(LayerOptions::glass(0.0), |p| {
        p.fill(
            &Shape::Blob {
                pieces: &pieces,
                smooth: 18.0,
            },
            glass,
        );
    });
    // The legacy slab glass, still used by the OSD until it is rebuilt on optical glass.
    let slab = p.tokens().glass(pal.glass_tint);
    p.layer(LayerOptions::glass(18.0), |p| {
        p.fill(
            &Shape::round_rect(Rect::from_xywh(-400.0, 220.0, 800.0, 120.0), 48.0),
            Glass { ..slab },
        );
    });
    // Bloom is earned: only glance ranks 1–3 go in a bloom layer.
    p.layer(LayerOptions::bloom(), |p| {
        p.fill(
            &Shape::arc(Vec2::ZERO, 120.0, 16.0, deg(135.0), deg(200.0), Cap::Round),
            pal.primary,
        );
    });
    // A blurred UI layer.
    p.layer(LayerOptions::blurred(12.0), |p| {
        p.fill(&Shape::rect(Rect::from_xywh(250.0, 50.0, 80.0, 80.0)), pal.secondary);
    });
}

// 5. Text --------------------------------------------------------------------------------------
fn text(p: &mut Painter<'_>, speed: i64) {
    let pal = *p.palette();
    let ts = p.tokens().type_scale;
    let display = p.fonts().role(fh6_kit::FontRole::Display);
    let label_font = p.fonts().role(fh6_kit::FontRole::Text);
    // Hero gear: cap height from the tokens, centred on the cap middle.
    p.text(
        vec2(0.0, -60.0),
        "4",
        &TextStyle::new(display, ts.gear_compact).paint(pal.roles.ink),
    );
    // Tabular speed, 3 fixed cells, ghost 8s.
    let speed_style = TextStyle::new(display, ts.speed_compact).paint(pal.roles.ink).tabular();
    p.number(
        vec2(0.0, 90.0),
        speed,
        3,
        &speed_style,
        Some(pal.roles.ink_ghost.into()),
    );
    // Micro-label: tracked caps in the label ink.
    let label = TextStyle::new(label_font, ts.micro_label)
        .paint(pal.roles.ink_label)
        .tracking(ts.label_tracking)
        .align(Align::Center, VAlign::CapTop);
    p.text(vec2(0.0, 150.0), "MPH", &label);
}

// 6. Animation ---------------------------------------------------------------------------------
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

// 7. Effects -----------------------------------------------------------------------------------
fn effects(p: &mut Painter<'_>, emitter: &mut Emitter, dye: fh6_kit::ImageId, dt: f32) {
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
        p.fill(
            &Shape::Polyline {
                points: &edge[..n],
                width: 3.0,
                closed: false,
            },
            pal.roles.hairline,
        );
    }
    let _ = fh6_kit::noise::curl2(0.3, 0.7, 0.0);
}

// 8. Cluster contract --------------------------------------------------------------------------
struct MyCluster {
    rpm: Spring,
    frac: f32,
}

impl Cluster for MyCluster {
    fn update(&mut self, input: &FrameInput<'_>) {
        let s = input.signals;
        self.frac = self
            .rpm
            .update_tracking(s.rpm_fraction, s.rpm_rate / s.max_rpm.max(1.0), input.dt);
    }
    fn draw(&self, p: &mut Painter<'_>, _layout: &Layout) {
        let pal = *p.palette();
        p.layer(LayerOptions::bloom(), |p| {
            p.fill(
                &Shape::arc(Vec2::ZERO, 260.0, 20.0, deg(135.0), deg(270.0 * self.frac), Cap::Round),
                pal.primary,
            );
        });
    }
    fn animating(&self) -> bool {
        !self.rpm.settled(self.frac, 1e-4)
    }
}

#[test]
fn guide_examples_render() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
    let mut kit = Kit::new(&ctx.device, &ctx.queue, Tokens::default()).expect("kit");
    let logo_png = {
        let mut bytes = Vec::new();
        let mut enc = png::Encoder::new(&mut bytes, 2, 2);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().expect("png header");
        w.write_image_data(&[255; 16]).expect("png data");
        drop(w);
        bytes
    };
    let image = kit
        .images_mut()
        .load_png(&ctx.device, &ctx.queue, &logo_png, "logo")
        .expect("png");
    let fluid = Fluid::new(&ctx.device, 128, 64, FluidConfig::default());
    let dye = kit
        .images_mut()
        .register_view(&ctx.device, fluid.dye_view(), fluid.size());
    let mut emitter = Emitter::new(256).with_config(EmitterConfig::default());
    let car = kit.upload_mesh(&ctx.device, &ctx.queue, &MeshData::car_proxy(4.6, 1.9, 1.3));
    let tokens = kit.tokens().clone();
    let mut needle = Needle::new(&tokens);
    let shown = needle.update(6000.0, 3000.0, 0.4, 118.2, 1.0 / 60.0);
    let mut cluster = MyCluster {
        rpm: Spring::new(tokens.springs.ribbon, 0.0),
        frac: 0.0,
    };
    let _ = &mut cluster;

    let target = Offscreen::new(&ctx.device, 1920, 1080);
    let layout = Layout::centered(vec2(960.0, 540.0), 0.8, (1920, 1080));
    for section in 0..6 {
        {
            let mut p = kit.begin_frame(&layout);
            match section {
                0 => shapes(&mut p),
                1 => paints(&mut p, image),
                2 => light(&mut p),
                3 => text(&mut p, shown),
                4 => effects(&mut p, &mut emitter, dye, 1.0 / 60.0),
                _ => {
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
                    cluster.draw(&mut p, &layout);
                }
            }
        }
        let mut encoder = ctx.device.create_command_encoder(&Default::default());
        let stats = kit.render(
            &ctx.device,
            &ctx.queue,
            &mut encoder,
            target.view(),
            OFFSCREEN_FORMAT,
            (1920, 1080),
            1.0,
        );
        target.copy_out(&mut encoder);
        ctx.queue.submit(Some(encoder.finish()));
        let img = target.read(&ctx.device).expect("readback");
        assert!(!ctx.has_failed(), "section {section}: wgpu error");
        assert_eq!(stats.dropped, 0, "section {section}");
        assert!(img.covered() > 100, "section {section} drew something");
    }
}
