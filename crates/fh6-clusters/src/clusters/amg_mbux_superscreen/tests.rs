use super::geom::{Geom, RINGS, SHIFT_PER_SIDE};
use super::hardware;
use super::*;
use fh6_kit::Tokens;
use std::path::Path;

/// The repository's tokens (the AMG palette lives only there, not in `Tokens::default`).
fn tokens() -> Tokens {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/design_tokens.json");
    Tokens::load(&file).expect("data/design_tokens.json loads")
}

fn inside(outer: Rect, r: Rect) -> bool {
    outer.contains(r.min) && outer.contains(r.max)
}

fn circle_rect(c: Vec2, r: f32) -> Rect {
    Rect::from_center_size(c, Vec2::splat(r * 2.0))
}

#[test]
fn the_unit_sits_inside_the_cockpit_envelope_and_every_zone_on_its_glass() {
    let t = Tokens::default();
    let g = Geom::COCKPIT;
    assert!(inside(t.layout.cockpit_envelope, g.housing), "{:?}", g.housing);
    assert!(inside(g.housing, g.screen));
    // The bezel is even all round.
    let bezel = g.screen.min.x - g.housing.min.x;
    assert!((g.screen.min.y - g.housing.min.y - bezel).abs() < 1e-3);
    assert!((g.housing.max.x - g.screen.max.x - bezel).abs() < 1e-3);
    assert!((g.housing.max.y - g.screen.max.y - bezel).abs() < 1e-3);
    // The real screen is 2.67:1; the glass here stays close to it (the cockpit gap is lower).
    let aspect = g.screen.width() / g.screen.height();
    assert!((2.6..=2.75).contains(&aspect), "aspect {aspect}");
    for side in [-1.0, 1.0] {
        for k in 0..=20 {
            let p = g.crescent_point(k as f32 / 20.0, 0.0, side);
            assert!(g.screen.inset(8.0).contains(p), "crescent at {p:?} leaves the glass");
        }
    }
    let zones = [
        circle_rect(g.g_widget, g.widget_r),
        circle_rect(g.boost_widget, g.widget_r),
        g.logo,
        g.transmission,
        g.shift_segment(0, -1.0),
        g.shift_segment(0, 1.0),
    ];
    for z in zones {
        assert!(inside(g.screen.inset(8.0), z), "{z:?} leaves the glass");
    }
    // The widgets' titles and labels (30 px beyond the ring, cap 22) stay on the glass too.
    for c in [g.g_widget, g.boost_widget] {
        assert!(c.y - g.widget_r - 30.0 - g.label_cap > g.screen.min.y);
        assert!(c.y + g.widget_r + 30.0 + g.label_cap < g.screen.max.y);
    }
}

#[test]
fn the_crescents_scales_and_widgets_never_collide() {
    let g = Geom::COCKPIT;
    // The side scales' numerals (three digits at cap 22 are about 46 px wide) sit between the
    // crescents and the ticks (18 px majors, inward), as on the real display.
    assert!(g.scale_numeral_r - 24.0 > g.crescent_r + 6.0);
    assert!(g.scale_numeral_r + 24.0 < g.scale_r - 18.0);
    // Their ends stay on the glass.
    for t in [0.0, 1.0] {
        let p = g.scale_point(t, g.scale_r + 12.0, -1.0);
        assert!(g.screen.inset(4.0).contains(p), "scale end {p:?}");
    }
    // Their numerals stay clear of the widgets.
    for side in [-1.0, 1.0] {
        let w = if side < 0.0 { g.g_widget } else { g.boost_widget };
        for k in 0..=4 {
            let p = g.scale_point(k as f32 / 4.0, g.scale_numeral_r, side);
            let d = (p - w).length();
            assert!(d > g.widget_r + 40.0, "numeral {k} at {p:?} is {d} px from a widget");
        }
    }
    // The widgets sit outside the scales.
    assert!((g.g_widget - g.tunnel).length() - g.widget_r > g.scale_r + 40.0);
    // The disc sits inside the innermost ring, which sits inside the crescents.
    let (c, r) = g.ring(RINGS - 1);
    assert!((c - g.disc).length() + g.disc_r <= r + 1e-3);
    let (c0, r0) = g.ring(0);
    assert!((c0 - g.tunnel).length() + r0 < g.crescent_r - g.crescent_w);
}

#[test]
fn the_shift_strip_runs_along_the_top_clear_of_the_speed_and_the_scales() {
    let g = Geom::COCKPIT;
    let inner = g.shift_segment(SHIFT_PER_SIDE - 1, 1.0);
    let outer = g.shift_segment(0, 1.0);
    assert!((inner.min.x - g.shift_inner).abs() < 1e-3, "{inner:?}");
    assert!((outer.max.x - g.shift_outer).abs() < 1e-3, "{outer:?}");
    // The strip runs along the glass's top edge, above the speed, the crescents' tips and the
    // side scales' top ends, and leaves room over three digits of the speed (about 130 px).
    let top = g.shift_y + g.shift_h * 0.5;
    assert!(
        top + 12.0 < g.speed.y - g.speed_cap * 0.5,
        "strip {top}, speed {:?}",
        g.speed
    );
    assert!(top + 12.0 < g.crescent_point(1.0, 0.0, -1.0).y);
    assert!(top + 12.0 < g.scale_point(1.0, g.scale_r, -1.0).y);
    assert!(g.shift_inner > 65.0 + 24.0);
    assert!(g.shift_y - g.shift_h * 0.5 > g.screen.min.y + 4.0);
    // Segments are mirrored.
    for i in 0..SHIFT_PER_SIDE {
        let l = g.shift_segment(i, -1.0);
        let r = g.shift_segment(i, 1.0);
        assert!((l.min.x + r.max.x).abs() < 1e-3 && (l.max.x + r.min.x).abs() < 1e-3);
    }
}

#[test]
fn the_crescents_fill_bottom_to_top_and_taper_to_their_tips() {
    let g = Geom::COCKPIT;
    let bottom = g.crescent_point(0.0, 0.0, -1.0);
    let top = g.crescent_point(1.0, 0.0, -1.0);
    assert!(bottom.y > top.y, "fill runs upwards");
    assert!(bottom.x < 0.0 && top.x < 0.0, "the left crescent is on the left");
    assert!(g.crescent_width(0.0) < 1e-3 && g.crescent_width(1.0) < 1e-3);
    assert!((g.crescent_width(0.5) - g.crescent_w).abs() < 1e-3);
    // The right crescent mirrors the left one.
    for t in [0.0, 0.3, 0.7, 1.0] {
        let l = g.crescent_point(t, 0.5, -1.0);
        let r = g.crescent_point(t, 0.5, 1.0);
        assert!((l.x + r.x).abs() < 1e-3 && (l.y - r.y).abs() < 1e-3);
    }
    let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
    assert_eq!(crescent_polygon(&g, 0.4, 0.4, -1.0, &mut pts), 0);
    assert_eq!(crescent_polygon(&g, 0.0, 1.0, 1.0, &mut pts), CRESCENT_SAMPLES * 2);
    assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
}

#[test]
fn the_default_palette_is_the_amg_red_and_only_the_shift_is_white() {
    let pal = own_palette(&tokens());
    let [r, g, b, _] = pal.primary.to_srgb8();
    assert!(r > 200 && g < 40 && b < 60, "primary {r} {g} {b}");
    let [r, g, b, _] = pal.shift.to_srgb8();
    assert!(r > 240 && g > 240 && b > 240, "shift {r} {g} {b}");
}

#[test]
fn the_scales_grow_by_whole_steps_and_ignore_bad_values() {
    assert_eq!(grown(SCALE_STEP, 100.0), SCALE_STEP);
    assert_eq!(grown(SCALE_STEP, SCALE_STEP * 0.96), SCALE_STEP * 2.0);
    assert_eq!(grown(SCALE_STEP * 2.0, SCALE_STEP * 3.5), SCALE_STEP * 4.0);
    assert_eq!(grown(SCALE_STEP, f32::NAN), SCALE_STEP);
    assert_eq!(grown(SCALE_STEP, f32::INFINITY), SCALE_STEP);
    // Bounded even for a huge value.
    assert!(grown(SCALE_STEP, 1e12) <= SCALE_STEP * 65.0);
}

#[test]
fn the_units_follow_the_users_choice() {
    assert_eq!(speed_unit(SpeedUnit::Kmh), "km/h");
    assert_eq!(speed_unit(SpeedUnit::Mph), "mph");
    assert_eq!(power_unit(PowerUnit::Kw), "kW");
    assert_eq!(torque_unit(PowerUnit::Kw), "Nm");
    assert_eq!(power_unit(PowerUnit::Hp), "hp");
    assert_eq!(torque_unit(PowerUnit::Hp), "lb-ft");
}

#[test]
fn static_parts_stay_under_the_static_light_cap() {
    let pal = own_palette(&tokens());
    let c = static_ink(pal.roles.ink, STATIC_LUMA);
    let [r, g, b, _] = c.to_srgb8();
    let lin = |v: u8| {
        let v = f32::from(v) / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let luma = 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
    assert!(luma <= 0.20 + 0.01, "static ink luma {luma}");
    // The procedural tunnel rings dim with depth and never go dark or non-finite.
    let levels = hardware::fallback_levels();
    assert!(levels.windows(2).all(|w| w[1] < w[0]));
    assert!(levels.iter().all(|l| l.is_finite() && *l > 0.0));
}
