//! GPU tests of optical glass: the shader against the CPU reference in `fh6_kit::optics`, and
//! the conservation check of `docs/specs/optics.md` §8.
#![cfg(not(target_arch = "wasm32"))]

use fh6_gpu::GpuContext;
use fh6_kit::color::srgb_to_linear;
use fh6_kit::optics::{fresnel_dielectric, transmittance, view_through, Dispersion, Profile};
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{
    deg, vec2, Color, Environment, Gradient, Kit, LayerOptions, Layout, OpticalGlass, Painter, Piece, Rect, Shape,
    Tokens, Vec2,
};

const W: u32 = 640;
const H: u32 = 480;

fn setup() -> (GpuContext, Kit) {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
    let kit = Kit::new(&ctx.device, &ctx.queue, Tokens::default()).expect("kit");
    (ctx, kit)
}

/// Renders at one device pixel per local unit, with the origin in the middle of the frame.
fn render(kit: &mut Kit, ctx: &GpuContext, name: &str, draw: impl FnOnce(&mut Painter<'_>)) -> RgbaImage {
    let layout = Layout::centered(vec2(W as f32 * 0.5, H as f32 * 0.5), 1.0, (W, H));
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
            .over([0, 0, 0])
            .save_png(&std::path::Path::new(&dir).join(format!("{name}.png")));
    }
    img
}

/// Linear RGB and alpha of the pixel whose centre is nearest `local`.
fn at(img: &RgbaImage, local: Vec2) -> [f32; 4] {
    let x = (W as f32 * 0.5 + local.x).floor() as u32;
    let y = (H as f32 * 0.5 + local.y).floor() as u32;
    let p = img.pixel(x, y).unwrap_or([0; 4]);
    [
        srgb_to_linear(f32::from(p[0]) / 255.0),
        srgb_to_linear(f32::from(p[1]) / 255.0),
        srgb_to_linear(f32::from(p[2]) / 255.0),
        f32::from(p[3]) / 255.0,
    ]
}

/// Clear sapphire with no light of its own: only what the test adds is seen.
fn dark_sapphire() -> OpticalGlass {
    let mut glass = OpticalGlass::new(&Dispersion::SAPPHIRE);
    glass.light = [0.0; 3];
    glass.env_gain = [0.0; 3];
    glass
}

const FIELD: Rect = Rect {
    min: Vec2 { x: -300.0, y: -220.0 },
    max: Vec2 { x: 300.0, y: 220.0 },
};

#[test]
fn glass_is_invisible_in_a_uniform_field() {
    // Reflected plus transmitted light is all the light there is (optics.md §8): with the same
    // radiance behind the glass and all round it, and no absorption, the glass cannot be seen.
    let (ctx, mut kit) = setup();
    let pieces = [
        Piece::circle(vec2(-150.0, 0.0), 90.0).dome(90.0, 0.0),
        Piece::pill(Rect::from_xywh(-20.0, -40.0, 260.0, 80.0)).section(24.0, 30.0, 10.0),
        Piece::circle(vec2(120.0, 90.0), 50.0).dome(30.0, 6.0),
    ];
    let level = 0.5_f32;
    let img = render(&mut kit, &ctx, "uniform_field", |p| {
        p.fill(
            &Shape::rect(FIELD),
            Color::WHITE.fade(1.0).lerp(Color::BLACK, 1.0 - level),
        );
        let mut glass = OpticalGlass::new(&Dispersion::SAPPHIRE);
        glass.light = [0.0; 3];
        glass.environment = Environment::Uniform;
        glass.env_gain = [level; 3];
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &pieces,
                    smooth: 14.0,
                },
                glass,
            );
        });
    });
    let reference = at(&img, vec2(-280.0, -200.0));
    let mut worst = 0.0_f32;
    for y in (-200..=200).step_by(3) {
        for x in (-290..=290).step_by(3) {
            let px = at(&img, vec2(x as f32, y as f32));
            for c in 0..4 {
                worst = worst.max((px[c] - reference[c]).abs());
            }
        }
    }
    assert!(
        worst <= 0.012,
        "glass shows in a uniform field: worst channel error {worst}"
    );
}

#[test]
fn the_rim_reflects_by_fresnel() {
    // Over black, lit only by a uniform environment of radiance 1, a pixel's value is the
    // Fresnel reflectance at its surface normal. On a hemisphere of radius R the normal at
    // radius r is tilted by asin(r / R).
    let (ctx, mut kit) = setup();
    let radius = 200.0_f32;
    let img = render(&mut kit, &ctx, "fresnel_dome", |p| {
        let mut glass = OpticalGlass::new(&Dispersion::SAPPHIRE);
        glass.light = [0.0; 3];
        glass.environment = Environment::Uniform;
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &[Piece::circle(Vec2::ZERO, radius).dome(radius, 0.0)],
                    smooth: 0.0,
                },
                glass,
            );
        });
    });
    let eta = Dispersion::SAPPHIRE.primaries();
    for r in [0.5_f32, 60.0, 120.0, 160.0, 185.0, 195.0] {
        let cos_i = (1.0 - (r / radius) * (r / radius)).sqrt();
        let got = at(&img, vec2(r, 0.5));
        for c in 0..3 {
            let want = fresnel_dielectric(cos_i, eta[c]);
            assert!(
                (got[c] - want).abs() <= 0.02 + want * 0.04,
                "r = {r}, channel {c}: shader {} vs Fresnel {want}",
                got[c]
            );
        }
    }
    let centre = at(&img, vec2(0.5, 0.5));
    assert!(
        centre[2] > centre[0],
        "blue has the higher index, so it reflects a little more: {centre:?}"
    );
}

#[test]
fn refraction_matches_snell_per_channel() {
    // A horizontal ramp under a dome: the value a pixel shows tells where the refracted ray
    // landed. Compared with the CPU trace, with each channel's own index.
    let (ctx, mut kit) = setup();
    let profile = Profile {
        thickness: 120.0,
        bevel: 180.0,
        base: 20.0,
        exponent: 2.0,
    };
    let radius = 180.0_f32;
    let ramp = |x: f32| ((x - FIELD.min.x) / FIELD.width()).clamp(0.0, 1.0);
    let img = render(&mut kit, &ctx, "refraction_ramp", |p| {
        p.fill(
            &Shape::rect(FIELD),
            Gradient::linear(vec2(FIELD.min.x, 0.0), vec2(FIELD.max.x, 0.0))
                .stop(0.0, Color::BLACK)
                .stop(1.0, Color::WHITE),
        );
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::circle(Vec2::ZERO, radius),
                dark_sapphire().with_profile(profile),
            );
        });
    });
    let eta = Dispersion::SAPPHIRE.primaries();
    for x in [-150.0_f32, -100.0, -40.0, 0.5, 60.0, 120.0, 160.0] {
        let outward = if x >= 0.0 { vec2(1.0, 0.0) } else { vec2(-1.0, 0.0) };
        let got = at(&img, vec2(x, 0.5));
        for c in 0..3 {
            let sight = view_through(&profile, radius - x.abs(), outward, eta[c]);
            let want = (1.0 - sight.reflectance) * ramp(x + sight.offset.x);
            assert!(
                (got[c] - want).abs() <= 0.02,
                "x = {x}, channel {c}: shader {} vs traced {want} (offset {})",
                got[c],
                sight.offset.x
            );
        }
    }
    // Outside the glass the ramp is untouched.
    let outside = at(&img, vec2(250.0, 0.5));
    assert!((outside[1] - ramp(250.0)).abs() <= 0.01, "{outside:?}");
}

#[test]
fn tint_follows_the_path_length() {
    // Over white with a black environment, the flat top shows (1 - F) e^(-sigma (H + b)).
    let (ctx, mut kit) = setup();
    let transmit = [0.30_f32, 0.60, 0.90];
    let thin = Profile {
        thickness: 10.0,
        bevel: 12.0,
        base: 10.0,
        exponent: 2.0,
    };
    let thick = Profile {
        thickness: 30.0,
        bevel: 12.0,
        base: 30.0,
        exponent: 2.0,
    };
    let img = render(&mut kit, &ctx, "tint_depth", |p| {
        p.fill(&Shape::rect(FIELD), Color::WHITE);
        p.layer(LayerOptions::glass(0.0), |p| {
            for (x, profile) in [(-140.0, thin), (140.0, thick)] {
                p.fill(
                    &Shape::round_rect(Rect::from_xywh(x - 100.0, -80.0, 200.0, 160.0), 30.0),
                    dark_sapphire().tinted(transmit, 20.0).with_profile(profile),
                );
            }
        });
    });
    let eta = Dispersion::SAPPHIRE.primaries();
    for (x, depth) in [(-140.0_f32, 20.0_f32), (140.0, 60.0)] {
        let got = at(&img, vec2(x, 0.5));
        for c in 0..3 {
            let sigma = fh6_kit::optics::absorption_coefficient(transmit[c], 20.0);
            let want = (1.0 - fresnel_dielectric(1.0, eta[c])) * transmittance(sigma, depth);
            assert!(
                (got[c] - want).abs() <= 0.02,
                "x = {x}, channel {c}: shader {} vs Beer-Lambert {want}",
                got[c]
            );
        }
    }
    let (a, b) = (at(&img, vec2(-140.0, 0.5)), at(&img, vec2(140.0, 0.5)));
    assert!(
        b[0] < a[0] * 0.2,
        "three times the depth cubes the transmittance: {a:?} {b:?}"
    );
}

#[test]
fn close_pieces_bridge_and_far_pieces_do_not() {
    let (ctx, mut kit) = setup();
    let pieces = [
        Piece::circle(vec2(-60.0, 0.0), 40.0),
        Piece::circle(vec2(60.0, 0.0), 40.0),
    ];
    let mut gap = [0.0_f32; 2];
    for (i, smooth) in [0.0_f32, 30.0].into_iter().enumerate() {
        let img = render(&mut kit, &ctx, &format!("blob_{i}"), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &pieces,
                    smooth,
                },
                Color::WHITE,
            );
        });
        gap[i] = at(&img, vec2(0.5, 0.5))[3];
        assert!(at(&img, vec2(-60.0, 0.5))[3] > 0.99, "each disc is filled");
        assert!(at(&img, vec2(0.5, 120.0))[3] < 0.01, "nothing far from the pieces");
    }
    assert!(
        gap[0] < 0.01 && gap[1] > 0.99,
        "plain union {} vs smooth union {}",
        gap[0],
        gap[1]
    );
}

#[test]
fn highlights_follow_the_light() {
    // The brightest pixel of a dome under the studio and the key light lies towards the light,
    // and turns with it.
    let (ctx, mut kit) = setup();
    let radius = 150.0_f32;
    let mut centroids = Vec::new();
    for azimuth in [deg(-135.0), deg(-45.0)] {
        let img = render(
            &mut kit,
            &ctx,
            &format!("light_{}", azimuth.to_degrees().round()),
            |p| {
                let mut glass = OpticalGlass::new(&Dispersion::SAPPHIRE);
                glass.light = [6.0; 3];
                glass.aim(azimuth, deg(40.0), Vec2::ZERO);
                p.layer(LayerOptions::glass(0.0), |p| {
                    p.fill(
                        &Shape::Blob {
                            pieces: &[Piece::circle(Vec2::ZERO, radius).dome(radius * 0.6, 0.0)],
                            smooth: 0.0,
                        },
                        glass,
                    );
                });
            },
        );
        let (mut sx, mut sy, mut sw) = (0.0_f32, 0.0_f32, 0.0_f32);
        for y in (-150..=150).step_by(2) {
            for x in (-150..=150).step_by(2) {
                let px = at(&img, vec2(x as f32, y as f32));
                let w = (px[0] + px[1] + px[2]).powi(2);
                sx += w * x as f32;
                sy += w * y as f32;
                sw += w;
            }
        }
        assert!(sw > 0.0, "the dome reflects something");
        centroids.push(vec2(sx / sw, sy / sw));
    }
    let (up_left, up_right) = (centroids[0], centroids[1]);
    assert!(up_left.x < -5.0 && up_left.y < -5.0, "lit from up-left: {up_left:?}");
    assert!(up_right.x > 5.0 && up_right.y < -5.0, "lit from up-right: {up_right:?}");
}

#[test]
fn materialise_grows_the_lens_and_never_fades() {
    // At materialise 0 the piece has no thickness: it bends nothing and reflects only its
    // face-on Fresnel share. The backdrop under it is as bright as beside it.
    let (ctx, mut kit) = setup();
    let img = render(&mut kit, &ctx, "materialise_0", |p| {
        p.fill(&Shape::rect(FIELD), Color::WHITE);
        let mut glass = dark_sapphire();
        glass.materialize = 0.0;
        p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(
                &Shape::Blob {
                    pieces: &[Piece::circle(Vec2::ZERO, 120.0).dome(120.0, 30.0)],
                    smooth: 0.0,
                },
                glass,
            );
        });
    });
    let eta = Dispersion::SAPPHIRE.primaries();
    for r in [0.5_f32, 60.0, 110.0] {
        let got = at(&img, vec2(r, 0.5));
        let want = 1.0 - fresnel_dielectric(1.0, eta[1]);
        assert!((got[1] - want).abs() <= 0.02, "r = {r}: {} vs flat {want}", got[1]);
    }
}
