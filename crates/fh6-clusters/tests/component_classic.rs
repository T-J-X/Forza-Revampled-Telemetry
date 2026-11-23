mod support;

// Visual shot test for the Classic Analog parts: the `needleDial` tach (COMPONENTS.md §3) inside a
// `titaniumRing` bezel at idle, on an LFA-style blip, on the limiter and in Track mode, plus every
// bezel variant (COMPONENTS.md §9) on circular and rounded-rect cavities, in the `classicAnalog`
// palette.
//
// The dials are driven through `update()` with constructed `Signals` (rpm, rpm rate, limiter and
// shift rpm: all verified telemetry fields), 30 frames at 1/60 s.

use fh6_clusters::common::bezel::{Bezel, BezelForm, BezelVariant};
use fh6_clusters::common::needle_dial::{Afterimage, DialConfig, DialMode, Lume, NeedleDial, OperatingBand};
use fh6_clusters::common::VirtualLight;
use fh6_kit::logo::{self, LogoStyle};
use fh6_kit::testing::RgbaImage;
use fh6_kit::{deg, vec2, Align, FontRole, Logo, Palette, Rect, Signals, TextStyle, Tokens, VAlign, Vec2};

/// Limiter of the shot car (tokens `states.limiterRpm`).
const LIMITER: f32 = 8700.0;
/// Face radius and titanium ring width (family layout: bezel 242–262).
const FACE_R: f32 = 242.0;
/// See [`FACE_R`].
const RING_W: f32 = 20.0;
/// Dial row centre y and column x (local 4K px).
const DIAL_Y: f32 = -400.0;
/// See [`DIAL_Y`].
const DIAL_X: [f32; 4] = [-1395.0, -465.0, 465.0, 1395.0];
/// Bezel row centre y and column x.
const BEZEL_Y: f32 = 560.0;
/// See [`BEZEL_Y`].
const BEZEL_X: [f32; 6] = [-1600.0, -960.0, -320.0, 320.0, 960.0, 1600.0];
/// Rounded-rect cavity size for the bezel row.
const CAVITY: Vec2 = Vec2::new(400.0, 240.0);
/// Circular cavity radius for the bezel row.
const CAVITY_R: f32 = 170.0;
/// Frameless demo: the rule's y and width, and the falloff depth.
const RULE_Y: f32 = BEZEL_Y + CAVITY.y * 0.5 - 30.0;
/// See [`RULE_Y`].
const RULE_W: f32 = 6.0;
/// See [`RULE_Y`].
const FALLOFF: f32 = 48.0;

/// The car's signals at `rpm`, climbing at `rate` rpm/s.
fn signals(tokens: &Tokens, rpm: f32, rate: f32, g_lat: f32) -> Signals {
    let shift_rpm = LIMITER * tokens.rev_stages.shift_pct;
    Signals {
        race_on: true,
        rpm,
        rpm_rate: rate,
        max_rpm: LIMITER,
        idle_rpm: 900.0,
        shift_rpm,
        rpm_fraction: (rpm / LIMITER).clamp(0.0, 1.0),
        shift_fraction: rpm / shift_rpm,
        g_lat,
        ..Signals::default()
    }
}

/// One dial settled on a state.
struct DialShot {
    title: &'static str,
    dial: NeedleDial,
    light: VirtualLight,
}

/// Settles `config` on a steady `rpm` for 30 frames, then (for a blip) climbs at `blip` rpm/s for
/// the last 8 frames.
fn settle(tokens: &Tokens, title: &'static str, config: DialConfig, rpm: f32, blip: f32, g_lat: f32) -> DialShot {
    let mut dial = NeedleDial::new(tokens, config);
    let mut light = VirtualLight::new(tokens);
    let mut r = rpm;
    for frame in 0..30 {
        let rate = if blip > 0.0 && frame >= 22 { blip } else { 0.0 };
        r += rate * support::DT;
        dial.update(&signals(tokens, r, rate, g_lat), support::DT);
        light.update(g_lat, support::DT);
    }
    DialShot { title, dial, light }
}

/// Local (4K, anchor-centred) to shot pixels.
fn to_px(p: Vec2) -> (i64, i64) {
    let s = support::H as f32 / 2160.0;
    (
        (p.x * s + support::W as f32 * 0.5).round() as i64,
        (p.y * s + support::H as f32 * 0.5).round() as i64,
    )
}

/// Mean premultiplied colour (equal to the colour over OLED black) of the pixels whose local
/// position satisfies `inside`, scanning the local rect `r`.
fn mean(img: &RgbaImage, r: Rect, inside: impl Fn(Vec2) -> bool) -> [f32; 3] {
    let (x0, y0) = to_px(r.min);
    let (x1, y1) = to_px(r.max);
    let s = 2160.0 / support::H as f32;
    let mut n = 0u32;
    let mut sum = [0.0f32; 3];
    for y in y0.max(0)..y1.min(i64::from(img.height)) {
        for x in x0.max(0)..x1.min(i64::from(img.width)) {
            let local = vec2(
                (x as f32 + 0.5 - support::W as f32 * 0.5) * s,
                (y as f32 + 0.5 - support::H as f32 * 0.5) * s,
            );
            if !inside(local) {
                continue;
            }
            let i = ((y as u32 * img.width + x as u32) * 4) as usize;
            n += 1;
            for c in 0..3 {
                sum[c] += f32::from(img.pixels[i + c]);
            }
        }
    }
    let k = 1.0 / n.max(1) as f32;
    [sum[0] * k, sum[1] * k, sum[2] * k]
}

/// Mean colour of an annular sector around `center` (radii and angles in local px / radians).
fn sector(img: &RgbaImage, center: Vec2, r0: f32, r1: f32, a: f32, half: f32) -> [f32; 3] {
    let bounds = Rect::from_center_size(center, Vec2::splat(r1 * 2.0));
    mean(img, bounds, |p| {
        let d = p - center;
        let r = d.length();
        let da = (d.y.atan2(d.x) - a + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        r >= r0 && r <= r1 && da.abs() <= half
    })
}

fn luma(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

#[test]
fn classic_needle_dial_and_bezels() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "classicAnalog");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let shift = LIMITER * tokens.rev_stages.shift_pct;

    let band = Some(OperatingBand::new(0.55, 0.85));
    let symmetric = DialConfig {
        band,
        ..DialConfig::default()
    };
    let dials = [
        settle(&tokens, "IDLE 900", symmetric, 900.0, 0.0, 0.0),
        settle(
            &tokens,
            "BLIP 24000 RPM/S . LFA",
            DialConfig {
                afterimage: Afterimage::Lfa,
                ..symmetric
            },
            3500.0,
            24_000.0,
            0.0,
        ),
        settle(&tokens, "LIMITER 8700", symmetric, LIMITER, 0.0, 0.0),
        settle(
            &tokens,
            "TRACK MODE . 1.0 G",
            DialConfig {
                mode: DialMode::Track,
                lume: Lume::Primary,
                ..symmetric
            },
            shift,
            0.0,
            1.0,
        ),
    ];
    let satellite = settle(&tokens, "", DialConfig::default(), 4200.0, 0.0, 0.0);

    // State assumptions the shot relies on.
    assert!(!dials[0].dial.animating(), "idle has settled");
    assert!(dials[1].dial.animating(), "the blip still moves (ghosts)");
    assert!(dials[2].dial.hub_heat() > 0.99, "hub ring hot on the limiter");
    assert!(dials[2].dial.wash() > 0.99, "steady red wash on the limiter");
    assert!(
        dials[0].dial.wash() == 0.0 && dials[0].dial.hub_heat() < 1e-3,
        "idle is cold"
    );
    let track_angle = dials[3].dial.needle_angle().rem_euclid(std::f32::consts::TAU);
    assert!(
        (track_angle - deg(270.0)).abs() < deg(1.0),
        "track needle at 12 ({track_angle})"
    );

    let titanium = Bezel::new(BezelVariant::TitaniumRing, RING_W);
    let logo_data = Logo {
        custom_name: "TJX".into(),
        ..Logo::default()
    };
    let rest_light = VirtualLight::new(&tokens).angle();
    let rect_at = |x: f32| Rect::from_center_size(vec2(x, BEZEL_Y), CAVITY);
    let bezels = [
        (
            "CHAMFER45 . RECT",
            Bezel::new(BezelVariant::Chamfer45, 16.0),
            BezelForm::round_rect(rect_at(BEZEL_X[0]), 34.0),
        ),
        (
            "CHAMFER45 . ROUND",
            Bezel::new(BezelVariant::Chamfer45, 16.0),
            BezelForm::circle(vec2(BEZEL_X[1], BEZEL_Y), CAVITY_R),
        ),
        (
            "CARBON . SATELLITE",
            Bezel::new(BezelVariant::CarbonFrame { weave: None }, 24.0),
            BezelForm::circle(vec2(BEZEL_X[2], BEZEL_Y), CAVITY_R),
        ),
        (
            "CARBON . RECT",
            Bezel::new(BezelVariant::CarbonFrame { weave: None }, 22.0),
            BezelForm::round_rect(rect_at(BEZEL_X[3]), 30.0),
        ),
        (
            "TITANIUM . RECT",
            Bezel::new(BezelVariant::TitaniumRing, 18.0),
            BezelForm::round_rect(rect_at(BEZEL_X[4]), 40.0),
        ),
        (
            "FRAMELESS . FALLOFF",
            Bezel::new(BezelVariant::Frameless, 0.0),
            BezelForm::round_rect(rect_at(BEZEL_X[5]), 40.0),
        ),
    ];

    let img = gpu.render(&layout, |p, _| {
        let text = p.fonts().role(FontRole::Text);
        let wide = p.fonts().role(FontRole::Wide);
        let display = p.fonts().role(FontRole::Display);
        let ts = p.tokens().type_scale;
        let title = TextStyle::new(text, ts.micro_label * 1.25)
            .paint(palette.roles.ink_label)
            .tracking(ts.label_tracking)
            .align(Align::Center, VAlign::CapTop);
        let logo_style = LogoStyle {
            tint: palette.roles.ink.with_alpha(215.0 / 255.0),
            text: palette.roles.ink.with_alpha(215.0 / 255.0).into(),
            cap: 12.0,
            ..LogoStyle::new(wide)
        };

        for (shot, x) in dials.iter().zip(DIAL_X) {
            let c = vec2(x, DIAL_Y);
            let light = shot.light.angle();
            p.text(vec2(x, DIAL_Y - FACE_R - RING_W - 70.0), shot.title, &title);
            let form = BezelForm::circle(c, FACE_R);
            titanium.fill_cavity(p, form, &palette);
            titanium.draw(p, form, &palette, light);
            shot.dial.draw_face(p, c, FACE_R, &palette, light);
            logo::draw(p, shot.dial.logo_zone(c, FACE_R), &logo_data, &logo_style);
            shot.dial.draw_needle(p, c, FACE_R, &palette, light);
        }

        for (i, ((name, bezel, form), x)) in bezels.iter().zip(BEZEL_X).enumerate() {
            p.text(vec2(x, BEZEL_Y - 250.0), name, &title);
            bezel.fill_cavity(p, *form, &palette);
            bezel.draw(p, *form, &palette, rest_light);
            match i {
                // The carbon ring around a satellite dial: the same part at R 170.
                2 => satellite
                    .dial
                    .draw(p, form.center(), CAVITY_R, &palette, satellite.light.angle()),
                // Frameless: content that runs into the cavity edge and dissolves there.
                5 => {
                    let speed = TextStyle::new(display, 150.0)
                        .paint(palette.roles.ink)
                        .align(Align::Center, VAlign::CapMiddle);
                    p.text(form.center(), "118", &speed);
                    // A rule from edge to edge (its round caps end just inside the rounded
                    // cavity edge).
                    let r = rect_at(x);
                    p.fill(
                        &fh6_kit::Shape::line(vec2(r.min.x + RULE_W, RULE_Y), vec2(r.max.x - RULE_W, RULE_Y), RULE_W),
                        palette.primary,
                    );
                    Bezel::draw_falloff(p, *form, &palette, FALLOFF);
                }
                _ => {}
            }
        }
    });
    let path = support::save_shot("classic_needle_dial_bezels", &img);
    assert!(path.exists(), "{}", path.display());

    let light = rest_light;

    // Needles: the idle needle points at 900 rpm, the limiter needle at 8700 rpm.
    for (i, rpm) in [(0usize, 900.0f32), (2, LIMITER)] {
        let c = vec2(DIAL_X[i], DIAL_Y);
        let a = dials[i].dial.scale().angle(rpm);
        let on = luma(sector(&img, c, 80.0, 180.0, a, deg(3.0)));
        let off = luma(sector(&img, c, 80.0, 120.0, a + deg(90.0), deg(3.0)));
        assert!(
            on > 60.0 && on > off * 3.0,
            "dial {i}: needle lit at {rpm} rpm ({on} vs {off})"
        );
    }

    // The steady red wash on the limiter: the face between hub and numerals turns red; idle stays
    // true black.
    // Probe between the 5 and 6 numerals (5500 rpm), inside the numeral ring.
    let face_probe = |i: usize| sector(&img, vec2(DIAL_X[i], DIAL_Y), 120.0, 145.0, deg(300.0), deg(4.0));
    let hot = face_probe(2);
    let cold = face_probe(0);
    assert!(hot[0] > hot[1] + 8.0, "limiter wash is red ({hot:?})");
    assert!(cold.iter().all(|v| *v < 16.0), "idle face is near black ({cold:?})");

    // Hub ring: warning on the limiter, neutral at idle (ring at hub + 4 px), probed below the
    // hub where neither needle nor tail passes.
    let hub = FACE_R * 0.10 + 4.0;
    let ring = |i: usize| {
        sector(
            &img,
            vec2(DIAL_X[i], DIAL_Y),
            hub - 2.0,
            hub + 2.0,
            deg(100.0),
            deg(30.0),
        )
    };
    let (hot, cold) = (ring(2), ring(0));
    assert!(hot[0] > hot[1] * 1.6, "hub ring hot ({hot:?})");
    assert!((cold[0] - cold[1]).abs() < 12.0, "hub ring neutral at idle ({cold:?})");

    // Titanium lobes face the light: the ring toward the light is brighter than the trough 90°
    // away.
    let c = vec2(DIAL_X[0], DIAL_Y);
    let lit = luma(sector(&img, c, FACE_R + 3.0, FACE_R + RING_W - 3.0, light, deg(6.0)));
    let trough = luma(sector(
        &img,
        c,
        FACE_R + 3.0,
        FACE_R + RING_W - 3.0,
        light + deg(90.0),
        deg(6.0),
    ));
    assert!(
        lit > trough * 1.5 && lit > 20.0,
        "titanium lobe toward the light ({lit} vs {trough})"
    );

    // The cavity is a cavity: black at the centre of an empty bezel.
    let cavity = mean(
        &img,
        Rect::from_center_size(vec2(BEZEL_X[4], BEZEL_Y), vec2(80.0, 80.0)),
        |_| true,
    );
    assert!(
        cavity.iter().all(|v| *v < 2.0),
        "titanium rect cavity is OLED black ({cavity:?})"
    );

    // Chamfer: the upper-left 45° facet is lit, the lower-right one shadowed.
    let r = rect_at(BEZEL_X[0]);
    let d = std::f32::consts::FRAC_1_SQRT_2;
    let facet = |corner: Vec2, dir: Vec2| {
        // Facet midpoint: the cut's midpoint pushed out to the middle of the outer bevel band.
        let at = corner + dir * (16.0 * 0.81 - 34.0 * d);
        mean(&img, Rect::from_center_size(at, vec2(6.0, 6.0)), |_| true)
    };
    let upper_left = luma(facet(vec2(r.min.x, r.min.y), vec2(-d, -d)));
    let lower_right = luma(facet(vec2(r.max.x, r.max.y), vec2(d, d)));
    assert!(
        upper_left > lower_right * 1.8,
        "chamfer facets ({upper_left} vs {lower_right})"
    );

    // The logo zone holds the wordmark and the needle never reaches it.
    let zone = dials[2].dial.logo_zone(vec2(DIAL_X[2], DIAL_Y), FACE_R);
    let mark = luma(mean(&img, zone, |_| true));
    assert!(mark > 1.0, "wordmark drawn in the logo zone ({mark})");

    // Frameless falloff: the rule dissolves to near black at the cavity edge (even on screen,
    // not just in linear light), while the middle of the rule still reads.
    let r = rect_at(BEZEL_X[5]);
    let rule = |x: f32| {
        luma(mean(
            &img,
            Rect::from_center_size(vec2(x, RULE_Y), vec2(6.0, 6.0)),
            |_| true,
        ))
    };
    let (edge, inside, middle) = (
        rule(r.min.x + 6.0),
        rule(r.min.x + FALLOFF * 0.5),
        rule(r.center().x + 60.0),
    );
    assert!(middle > 40.0, "the rule reads inside the cavity ({middle})");
    assert!(
        edge < middle * 0.15 && edge < inside && inside < middle,
        "falloff ramps to black at the edge ({edge} < {inside} < {middle})"
    );
}

/// A 2x close-up of the materials: the limiter dial and the blip's afterimage at R 460 in
/// titanium, a large chamfer frame, carbon on a circle and a rounded rect, titanium on a rounded
/// rect.
#[test]
fn classic_detail() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "classicAnalog");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let config = DialConfig {
        band: Some(OperatingBand::new(0.55, 0.85)),
        afterimage: Afterimage::Lfa,
        ..DialConfig::default()
    };
    let limiter = settle(&tokens, "", config, LIMITER, 0.0, 0.0);
    let blip = settle(&tokens, "", config, 3000.0, 24_000.0, -0.6);
    let rest = VirtualLight::new(&tokens).angle();

    const R: f32 = 460.0;
    const W: f32 = 38.0;
    let dials = [(vec2(-1380.0, -500.0), &limiter), (vec2(-260.0, -500.0), &blip)];
    let titanium = Bezel::new(BezelVariant::TitaniumRing, W);
    let chamfer_form = BezelForm::round_rect(Rect::from_center_size(vec2(1140.0, -500.0), vec2(820.0, 560.0)), 70.0);
    let chamfer = Bezel::new(BezelVariant::Chamfer45, 30.0);
    let carbon_round = BezelForm::circle(vec2(-1380.0, 620.0), 300.0);
    let carbon_rect = BezelForm::round_rect(Rect::from_center_size(vec2(-180.0, 620.0), vec2(760.0, 420.0)), 56.0);
    let carbon = Bezel::new(BezelVariant::CarbonFrame { weave: None }, 44.0);
    let ti_rect = BezelForm::round_rect(Rect::from_center_size(vec2(1140.0, 620.0), vec2(760.0, 420.0)), 72.0);
    let ti = Bezel::new(BezelVariant::TitaniumRing, 34.0);

    let img = gpu.render(&layout, |p, _| {
        for (c, shot) in dials {
            let light = shot.light.angle();
            let form = BezelForm::circle(c, R);
            titanium.fill_cavity(p, form, &palette);
            titanium.draw(p, form, &palette, light);
            shot.dial.draw(p, c, R, &palette, light);
        }
        for (bezel, form) in [
            (chamfer, chamfer_form),
            (carbon, carbon_round),
            (carbon, carbon_rect),
            (ti, ti_rect),
        ] {
            bezel.fill_cavity(p, form, &palette);
            bezel.draw(p, form, &palette, rest);
        }
    });
    let path = support::save_shot("classic_detail", &img);
    assert!(path.exists(), "{}", path.display());

    // Every bezel ring carries light somewhere along it, and its cavity stays OLED black.
    for (name, form, w) in [
        ("chamfer", chamfer_form, 30.0),
        ("carbon round", carbon_round, 44.0),
        ("carbon rect", carbon_rect, 44.0),
        ("titanium rect", ti_rect, 34.0),
    ] {
        let top = form.grown(w * 0.5).bounds().min.y;
        let band = Rect::from_center_size(vec2(form.center().x, top), vec2(120.0, 8.0));
        let lit_side = luma(mean(&img, band, |_| true));
        assert!(lit_side > 4.0, "{name}: the top of the ring carries light ({lit_side})");
        let cavity = mean(&img, Rect::from_center_size(form.center(), vec2(60.0, 60.0)), |_| true);
        assert!(
            cavity.iter().all(|v| *v < 2.0),
            "{name}: cavity is OLED black ({cavity:?})"
        );
    }
}
