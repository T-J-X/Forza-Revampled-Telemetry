use super::geom::{Geom, RINGS, SHIFT_PER_SIDE};
use super::hardware::{crescent_band, crescent_polygon};
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
fn every_part_floats_inside_the_cockpit_envelope() {
    let t = Tokens::default();
    let env = t.layout.cockpit_envelope.inset(4.0);
    let g = Geom::COCKPIT;
    for side in [-1.0, 1.0] {
        for k in 0..=20 {
            let p = g.crescent_point(k as f32 / 20.0, 0.0, side);
            assert!(env.contains(p), "crescent at {p:?} leaves the envelope");
        }
        for t in [0.0, 1.0] {
            let a = g.band_angle(t, side);
            let p = g.tunnel + vec2(a.cos(), a.sin()) * (g.band_out + 4.0);
            assert!(env.contains(p), "band end {p:?} leaves the envelope");
        }
    }
    let zones = [
        g.assembly_rect,
        g.crescent_rect,
        g.widget_rect(g.g_widget, g.lens_margin),
        g.widget_rect(g.boost_widget, g.gauge_margin),
        g.logo,
        circle_rect(g.gear, g.gear_cap * 0.6),
        circle_rect(g.stack_row(-1.0), g.stack_cap),
        circle_rect(g.stack_row(1.0), g.stack_cap),
        g.shift_segment(0, -1.0),
        g.shift_segment(0, 1.0),
        circle_rect(g.speed, g.speed_cap * 0.5),
    ];
    for z in zones {
        assert!(inside(env, z), "{z:?} leaves the envelope");
    }
    // The widgets' titles above and their labels below stay inside too.
    for c in [g.g_widget, g.boost_widget] {
        assert!(c.y - g.widget_r - g.title_dy - g.label_cap > env.min.y);
        assert!(c.y + g.widget_r + 29.0 + g.unit_cap < env.max.y);
    }
    // The plates cover their parts: the crescents, the bands and the road.
    for side in [-1.0, 1.0] {
        for k in 0..=20 {
            assert!(g.crescent_rect.contains(g.crescent_point(k as f32 / 20.0, 0.0, side)));
        }
    }
    assert!(g.assembly_rect.max.y >= g.floor_bottom && g.assembly_rect.max.x >= g.band_out + 4.0);
}

#[test]
fn the_tunnel_bands_and_widgets_never_collide() {
    let g = Geom::COCKPIT;
    // The scale numerals (three digits at cap 20 are about 42 px wide) sit on the band, inside
    // the ticks (14 px majors, inward from the rim), as on the real display.
    assert!(g.scale_numeral_r - 22.0 > g.band_in);
    assert!(g.scale_numeral_r + 22.0 < g.band_out - 14.0);
    // The bands clear the crescents, and the widgets clear the bands.
    assert!(g.band_in > g.crescent_r + 6.0);
    for w in [g.g_widget, g.boost_widget] {
        assert!((w - g.tunnel).length() - g.widget_r > g.band_out + 20.0, "{w:?}");
    }
    // The ring lines grow from just outside the light to just inside the crescents.
    let (c0, r0) = g.ring(0);
    assert!((c0 - g.disc).length() + g.disc_r < r0);
    for k in 1..RINGS {
        assert!(g.ring(k).1 > g.ring(k - 1).1);
    }
    let (c1, r1) = g.ring(RINGS - 1);
    assert!((c1 - g.tunnel).length() < 1e-3);
    assert!(r1 < g.crescent_r - g.crescent_w);
    // The horizon cuts the light below its top and the road starts there.
    assert!(g.horizon > g.disc.y - g.disc_r && g.horizon < g.disc.y + g.disc_r);
    assert!(g.floor()[0].y == g.horizon && g.floor_bottom > g.rpm.y + g.rpm_cap);
    // The rpm sits on the road under the horizon, the speed above the speed line.
    assert!(g.rpm.y - g.rpm_cap * 0.5 > g.horizon + 40.0);
    assert!(g.speed.y + g.speed_cap * 0.5 < g.line_y - 4.0);
    // The selector stack's column clears the engaged gear.
    assert!(g.stack.x + g.stack_cap < g.gear.x - g.gear_cap * 0.5);
}

#[test]
fn the_shift_strip_runs_on_the_speed_line_inside_the_crescents() {
    let g = Geom::COCKPIT;
    let inner = g.shift_segment(SHIFT_PER_SIDE - 1, 1.0);
    let outer = g.shift_segment(0, 1.0);
    assert!((inner.min.x - g.shift_inner).abs() < 1e-3, "{inner:?}");
    assert!((outer.max.x - g.shift_outer).abs() < 1e-3, "{outer:?}");
    assert!((outer.center().y - g.line_y).abs() < 1e-3);
    // It leaves room under three digits of the speed (about 120 px).
    assert!(g.shift_inner > 60.0);
    // The crescents' inner edge at the speed line's height is outside the strip.
    let mut nearest = (f32::MAX, 0.0_f32);
    for k in 0..=400 {
        let p = g.crescent_point(k as f32 / 400.0, 1.0, 1.0);
        let d = (p.y - g.line_y).abs();
        if d < nearest.0 {
            nearest = (d, p.x);
        }
    }
    assert!(nearest.1 >= g.shift_outer - 2.0, "inner edge at x {}", nearest.1);
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
    // Flat in the middle: at a quarter it keeps most of its width.
    assert!(g.crescent_width(0.25) > g.crescent_w * 0.85);
    // The right crescent mirrors the left one.
    for t in [0.0, 0.3, 0.7, 1.0] {
        let l = g.crescent_point(t, 0.5, -1.0);
        let r = g.crescent_point(t, 0.5, 1.0);
        assert!((l.x + r.x).abs() < 1e-3 && (l.y - r.y).abs() < 1e-3);
    }
    let mut pts = [Vec2::ZERO; CRESCENT_SAMPLES * 2];
    assert_eq!(crescent_polygon(&g, 0.4, 0.4, -1.0, &mut pts), 0);
    assert_eq!(crescent_band(&g, 0.0, 0.5, 0.1, 1.0, &mut pts), CRESCENT_SAMPLES * 2);
    assert_eq!(crescent_polygon(&g, 0.0, 1.0, 1.0, &mut pts), CRESCENT_SAMPLES * 2);
    assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
}

#[test]
fn the_stack_shows_the_gears_either_side() {
    assert_eq!(neighbours(Gear::Forward(4)), ("5", "3"));
    assert_eq!(neighbours(Gear::Forward(1)), ("2", "N"));
    assert_eq!(neighbours(Gear::Neutral), ("1", "R"));
    assert_eq!(neighbours(Gear::Reverse), ("N", ""));
    assert_eq!(neighbours(Gear::Forward(10)), ("", "9"));
    // A raw byte past 10 never indexes out of the list.
    assert_eq!(neighbours(Gear::Forward(200)), ("", "9"));
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
}
