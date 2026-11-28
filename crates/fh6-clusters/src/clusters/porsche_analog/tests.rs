use super::scale::{shortest_turn, DialScale, Mode, SWEEP};
use super::*;
use fh6_kit::optics::{view_through, Dispersion, Profile};

/// Every corner of `r`.
fn corners(r: Rect) -> [Vec2; 4] {
    [r.min, vec2(r.max.x, r.min.y), r.max, vec2(r.min.x, r.max.y)]
}

#[test]
fn every_part_stays_inside_the_cockpit_envelope_and_the_hood_one_when_scaled() {
    let t = Tokens::default();
    let g = &GEOM;
    let mut parts = vec![Rect::from_center_size(g.tach, Vec2::splat(g.tach_outer * 2.0))];
    for (i, c) in g.tubes.iter().enumerate() {
        let r = if i == 1 || i == 2 { g.side_outer } else { g.far_outer };
        parts.push(Rect::from_center_size(*c, Vec2::splat(r * 2.0)));
    }
    let cockpit = t.layout.cockpit_envelope;
    let hood = t.layout.hood_envelope;
    for r in &parts {
        assert!(
            cockpit.contains(r.min) && cockpit.contains(r.max),
            "{r:?} in {cockpit:?}"
        );
        let s = Rect::from_points(r.min * HOOD_SCALE, r.max * HOOD_SCALE);
        assert!(hood.contains(s.min) && hood.contains(s.max), "{s:?} in {hood:?}");
    }
}

#[test]
fn the_tubes_tuck_behind_their_inner_neighbour() {
    let g = &GEOM;
    let pairs = [
        (g.tach, g.tach_outer, g.tubes[1], g.side_outer),
        (g.tach, g.tach_outer, g.tubes[2], g.side_outer),
        (g.tubes[1], g.side_outer, g.tubes[0], g.far_outer),
        (g.tubes[2], g.side_outer, g.tubes[3], g.far_outer),
    ];
    for (a, ra, b, rb) in pairs {
        let d = (a - b).length();
        // Each tube hides a tenth or more of its neighbour's width, as on the 992.2's Classic
        // view (a tenth to a sixth there), and never its middle.
        let hidden = ra + rb - d;
        assert!(
            hidden >= 0.09 * rb * 2.0,
            "the tubes at {a:?} and {b:?} hide only {hidden} px"
        );
        assert!(d > ra, "the inner tube hides the middle of the outer one");
    }
    for slot in [1, 2] {
        let d = (g.tubes[slot] - g.tach).length();
        // The rev counter covers at most the tips of a side tube's ticks, never its numerals.
        assert!(
            d - g.tach_outer >= g.side_tick_out - 6.0,
            "the rev counter covers a side scale"
        );
        // A side crystal reaches under the rev counter's bezel only (it is drawn before the rev
        // counter), never into its dial.
        assert!(
            d - g.side_open >= g.tach_open + 8.0,
            "a side crystal reaches the rev counter's dial"
        );
    }
}

#[test]
fn the_shift_ring_sits_on_the_bezel_top_and_only_a_short_crest_strobes() {
    let g = &GEOM;
    assert!(g.led_r - g.led_size.y * 0.5 >= g.tach_open + 8.0);
    assert!(g.led_r + g.led_size.y * 0.5 <= g.tach_outer - 5.0);
    // The middle lens is at 12 o'clock and is a crest lens; the strobing band is ≤ 42 px tall.
    assert!(shortest_turn(g.led_angle(LEDS / 2), -FRAC_PI_2).abs() < 1e-4);
    let (mut top, mut bottom) = (f32::MAX, f32::MIN);
    let mut crest = 0;
    for i in 0..LEDS {
        if !g.crest(i) {
            continue;
        }
        crest += 1;
        let a = g.led_angle(i);
        let c = Vec2::polar(g.tach, g.led_r, a);
        // The lens is turned along the ring: its half extent in y.
        let along = Vec2::polar(Vec2::ZERO, g.led_size.x * 0.5, a + FRAC_PI_2);
        let across = Vec2::polar(Vec2::ZERO, g.led_size.y * 0.5, a);
        let half = along.y.abs() + across.y.abs();
        top = top.min(c.y - half);
        bottom = bottom.max(c.y + half);
    }
    assert_eq!(crest, 3, "three lenses strobe");
    // With the lit lenses' glow: the flash audit counts it out to about 7 px a side at this
    // sigma and intensity (36 px measured), so the glow may not grow past that.
    let glow_reach = 7.0;
    assert!(
        LENS_GLOW.0 <= 6.0 && LENS_GLOW.1 <= 0.4,
        "the glow grew: re-run the flash audit"
    );
    let band = bottom - top + 2.0 * glow_reach;
    assert!(band <= 42.0, "the strobing band is {band} px tall with its glow");
}

#[test]
fn on_the_track_face_every_lens_sits_over_the_approach_or_past_the_scale() {
    // No lens may sit beside the low numerals: on the track face each lens lies over at least
    // half the shift rpm, or past the top of the scale (the numeral-free dead zone).
    let g = &GEOM;
    for (mode, max, pct, scale) in scales() {
        if mode != Mode::Track || pct < 0.9 {
            continue;
        }
        for i in 0..LEDS {
            let a = g.led_angle(i);
            // The lens's angle measured along the scale from 0 rpm.
            let along = (a - scale.start).rem_euclid(TAU);
            if along > SWEEP {
                continue;
            }
            let rpm = along / scale.per_rpm();
            assert!(rpm >= max * pct * 0.5, "{max} {pct}: lens {i} sits over {rpm} rpm");
        }
    }
}

/// Every scale the window and the logo must work for: both faces, short-shifting to
/// redline-shifting (the OSD's `ShiftPointPct` is 0.80 to 1.00), small to big limiters.
fn scales() -> Vec<(Mode, f32, f32, DialScale)> {
    let mut v = Vec::new();
    for mode in [Mode::Track, Mode::Classic] {
        for max in [
            4000.0, 5999.0, 6500.0, 7500.0, 8700.0, 8999.0, 9000.0, 12_000.0, 19_999.0,
        ] {
            for pct in [0.80, 0.9, 0.97, 1.0] {
                v.push((mode, max, pct, DialScale::new(mode, max, max * pct)));
            }
        }
    }
    v
}

#[test]
fn the_gear_window_sits_in_the_dead_zone_clear_of_the_hub_and_the_scale() {
    let g = &GEOM;
    for (mode, max, pct, scale) in scales() {
        let c = g.window_centre(&scale);
        let d = (c - g.tach).length();
        // Inside the ticks' outer end, clear of the hub and the needle's tail.
        assert!(
            d + g.porthole_rim <= g.tick_out - 8.0,
            "{mode:?} {max} {pct}: rim reaches R {}",
            d + g.porthole_rim
        );
        assert!(
            d - g.porthole_rim >= g.needle_tail + g.needle_half_w + 1.0,
            "{mode:?} {max} {pct}: tail"
        );
        // It reaches across the scale ring only in the numeral-free sector: its angular
        // half-width fits in half the dead zone, less the needle's 4 % overrun past the top and
        // room for the end numerals (about 8°).
        let half = (g.porthole_rim / d).asin();
        let overrun = SWEEP * 0.04;
        let room = 8f32.to_radians();
        assert!(
            half < (TAU - SWEEP) * 0.5 - overrun - room,
            "{mode:?} {max} {pct}: {} deg",
            half.to_degrees()
        );
        let off = shortest_turn(scale.dead_centre(), (c - g.tach).y.atan2((c - g.tach).x)).abs();
        assert!(off < 1e-4);
        // The "+" cue sits inside the window beside the gear, clear of the gear and of the
        // window's edge. Ink half-widths in Barlow Semi Condensed Bold: a digit up to 0.3 of its
        // cap (the "8"), "10" 0.56 of its smaller cap (proportional "1" and "0"), the "+" 0.3.
        let plus = g.plus_centre(&scale) - c;
        assert!(
            plus.length() + g.plus_cap * 0.5 < g.porthole - 2.0,
            "{mode:?} {max} {pct}: plus edge"
        );
        for half_gear in [g.gear_cap * 0.3, g.gear_cap * TWO_DIGIT_GEAR * 0.56] {
            assert!(plus.x - g.plus_cap * 0.3 > half_gear + 2.0, "plus meets the gear");
        }
    }
}

#[test]
fn a_two_digit_gear_stays_inside_what_the_magnifier_shows() {
    // Through the dome the porthole shows content out to about 63 px from its centre (the
    // review's trace of `view_through` at the rim); "10" at the smaller cap reaches its "1"'s
    // flag about 0.71 cap out from the centre.
    let g = &GEOM;
    let reach = g.gear_cap * TWO_DIGIT_GEAR * 0.71;
    assert!(reach < 60.0, "\"10\" reaches {reach} px");
}

#[test]
fn distance_to_rect_measures_to_the_nearest_edge() {
    let r = Rect::from_center_size(vec2(0.0, 0.0), vec2(20.0, 10.0));
    assert_eq!(distance_to_rect(vec2(3.0, 2.0), r), 0.0);
    assert!((distance_to_rect(vec2(15.0, 0.0), r) - 5.0).abs() < 1e-5);
    assert!((distance_to_rect(vec2(0.0, -9.0), r) - 4.0).abs() < 1e-5);
    // Off a corner: the distance to that corner.
    assert!((distance_to_rect(vec2(13.0, 9.0), r) - 5.0).abs() < 1e-5);
}

#[test]
fn numerals_thin_out_only_where_two_digit_ones_would_meet() {
    // Barlow at the numeral cap: one tabular cell is 0.6 em of a 0.7 cap ratio.
    let cell = GEOM.num_cap / 0.7 * 0.6;
    let r = GEOM.tick_out - GEOM.major.0 - NUM_GAP - GEOM.num_cap * 0.5;
    for (max, step) in [(7500.0, 1), (8700.0, 1), (11_000.0, 1), (14_000.0, 2), (19_999.0, 2)] {
        let s = DialScale::new(Mode::Track, max, max * 0.97);
        let spacing = r * s.per_rpm() * 1000.0;
        assert_eq!(numeral_step(spacing, cell), step, "{max}: spacing {spacing}");
    }
}

#[test]
fn the_logo_zone_sits_across_the_hub_clear_of_the_window_and_the_scale() {
    let g = &GEOM;
    for (mode, max, pct, scale) in scales() {
        let zone = g.logo_zone(&scale);
        let window = g.window_centre(&scale);
        for k in corners(zone) {
            let r = (k - g.tach).length();
            assert!(r < g.scale_in - 4.0, "{mode:?} {max} {pct}: logo corner at R {r}");
        }
        // Every point of the zone and of its legend, edges included, clears the window's rim.
        assert!(
            distance_to_rect(window, zone) > g.porthole_rim + 4.0,
            "{mode:?} {max} {pct}: logo meets the window"
        );
        let legend_box = g.legend_box(zone);
        assert!(
            distance_to_rect(window, legend_box) > g.porthole_rim + 4.0,
            "{mode:?} {max} {pct}: legend meets the window"
        );
        for k in corners(legend_box) {
            let r = (k - g.tach).length();
            assert!(r < g.scale_in - 2.0, "{mode:?} {max} {pct}: legend corner at R {r}");
        }
        // The nearest point of the zone to the hub clears the hub cap.
        let near = vec2(
            g.tach.x.clamp(zone.min.x, zone.max.x),
            g.tach.y.clamp(zone.min.y, zone.max.y),
        );
        assert!(
            (near - g.tach).length() > g.hub_r + 2.0,
            "{mode:?} {max} {pct}: logo meets the hub"
        );
    }
    // On the classic face the logo is above the hub and the window below it, as on the 992.1.
    let classic = DialScale::new(Mode::Classic, 7500.0, 7275.0);
    assert!(g.logo_zone(&classic).center().y < g.tach.y && g.window_centre(&classic).y > g.tach.y);
    // Wherever the logo sits below the hub, the needle never stands over it from half the shift
    // rpm to the limiter, on any scale.
    let needle_clear = |zone: Rect, scale: &DialScale, max: f32, shift: f32| {
        let reach = (zone.max.x - g.tach.x).atan2(zone.min.y - g.tach.y);
        (0..=40).all(|k| {
            let rpm = shift * 0.5 + (max - shift * 0.5) * k as f32 / 40.0;
            shortest_turn(scale.angle(rpm), FRAC_PI_2).abs() > reach
        })
    };
    for (mode, max, pct, scale) in scales() {
        let zone = g.logo_zone(&scale);
        if zone.center().y > g.tach.y {
            assert!(
                needle_clear(zone, &scale, max, max * pct),
                "{mode:?} {max} {pct}: the needle crosses the logo"
            );
        }
    }
    // On the track face with the default shift point the logo sits below the hub for the
    // common limiters, the review car's and the default mock car's (7,600) among them.
    for max in [6000.0, 6500.0, 6800.0, 7500.0, 7600.0, 8600.0, 8700.0, 9000.0, 12_000.0] {
        let track = DialScale::new(Mode::Track, max, max * 0.97);
        assert!(g.logo_zone(&track).center().y > g.tach.y, "{max}: logo above the hub");
    }
}

#[test]
fn the_magnifier_shows_the_gear_larger_and_unbroken() {
    // The porthole's dome over the gear: a contact magnifier (docs/specs/optics.md §4).
    let g = &GEOM;
    let dome = Profile {
        thickness: g.cyclops.0,
        bevel: g.porthole,
        base: g.cyclops.1,
        exponent: 2.0,
    };
    let eta = Dispersion::SAPPHIRE.primaries()[1];
    let near = view_through(&dome, g.porthole - 10.0, vec2(1.0, 0.0), eta);
    let magnified = 10.0 / (10.0 + near.offset.x);
    // A low dome: about 5 % (a taller one read as a glass button stuck on the dial).
    assert!((1.03..=1.10).contains(&magnified), "magnification {magnified}");
    // The gear (about 0.6 cap wide) seen through it stays inside the porthole.
    let seen = g.gear_cap * magnified;
    let half_diag = vec2(seen * 0.3, seen * 0.5).length();
    assert!(
        half_diag < g.porthole - 2.0,
        "seen cap {seen}, half diagonal {half_diag}"
    );
}

#[test]
fn the_side_digits_fit_inside_the_scale_ring() {
    let g = &GEOM;
    // Three tabular cells of 0.6 em; Barlow's cap height is about 0.7 em.
    let em = g.speed_cap / 0.7;
    let half_w = 1.5 * 0.6 * em;
    let corner = vec2(half_w, g.speed_cap * 0.5 + 6.0).length();
    assert!(corner < g.side_scale_in + 2.0, "speed digits reach R {corner}");
    // The pointer stays on the scale ring.
    assert!(g.pointer.0 >= g.side_scale_in && g.pointer.1 < g.side_open);
    assert!(g.far_pointer.1 < g.far_open);
}

#[test]
fn options_parse_from_the_config() {
    let mut cfg = AppConfig::default();
    assert_eq!(Options::from_config(&cfg), Options::default());
    cfg.extra.insert("PorscheAnalogScale".into(), "Classic".into());
    cfg.extra.insert("PorscheAnalogLeft".into(), "g".into());
    cfg.extra.insert("PorscheAnalogRight".into(), "Speed".into());
    cfg.extra.insert("PorscheAnalogFarLeft".into(), "off".into());
    cfg.extra.insert("PorscheAnalogFarRight".into(), "nonsense".into());
    cfg.extra.insert("PorscheAnalogNumerals".into(), "Heritage".into());
    let o = Options::from_config(&cfg);
    assert_eq!(o.mode, Mode::Classic);
    assert_eq!(o.tubes, [Content::Off, Content::GForce, Content::Speed, Content::Boost]);
    assert!(o.heritage);
}

#[test]
fn every_tube_value_parses_in_any_case_with_spaces() {
    // Each value goes into a slot whose default is something else, so a broken arm cannot hide
    // behind the fallback.
    let cases = [
        (" POWER ", Content::Speed, Content::Power),
        ("boost", Content::GForce, Content::Boost),
        ("GForce", Content::Boost, Content::GForce),
        ("G", Content::Power, Content::GForce),
        ("Hidden", Content::Speed, Content::Off),
        ("SPEED", Content::Power, Content::Speed),
        ("Off ", Content::Boost, Content::Off),
    ];
    for (value, default, want) in cases {
        assert_eq!(Content::parse(Some(value), default), want, "{value:?}");
    }
    assert_eq!(Content::parse(None, Content::Boost), Content::Boost);
    assert_eq!(Content::parse(Some("speedo"), Content::Boost), Content::Boost);
}

#[test]
fn the_needle_reaches_from_the_hub_into_the_ticks() {
    let g = &GEOM;
    assert!(g.needle_tip > g.tick_out - g.major.0 && g.needle_tip < g.tick_out);
    assert!(g.needle_tail > g.hub_r);
    // The crystal's flat top covers every tick, so the scale is never bent by its edge.
    assert!(g.tach_open - g.tach_crystal.1 >= g.tick_out);
}

#[test]
fn a_parked_car_greets_its_brand_while_the_dial_loads_never_driver_1() {
    let loading = Logo {
        brand_name: "Porsche".into(),
        custom_name: "TJX".into(),
        loading: true,
        ..Logo::default()
    };
    assert_eq!(tubes::greeting_name("", &loading), "Porsche");
    assert_eq!(
        tubes::greeting_name("  ", &loading),
        "Porsche",
        "a blank name is no name"
    );
    assert_eq!(
        tubes::greeting_name(" Ace ", &loading),
        "Ace",
        "the driver's name comes first"
    );
    assert_eq!(tubes::greeting_name("", &Logo::default()), "DRIVER 1");
}

#[test]
fn the_logo_zone_shows_no_name_while_the_cars_dial_loads() {
    let (car, maker) = (Some(7_u32), Some(1_u32));
    let names = Logo {
        brand_name: "Porsche".into(),
        custom_name: "TJX".into(),
        ..Logo::default()
    };
    let loading = Logo {
        loading: true,
        ..names.clone()
    };
    // While the car's own dial is on its way: a maker car shows the maker's wordmark at once,
    // any other car nothing, and never the name.
    assert_eq!(zone_art(None, maker, &loading), ZoneArt::Image(1));
    assert_eq!(zone_art::<u32>(None, None, &loading), ZoneArt::Nothing);
    assert_eq!(zone_art(car, maker, &names), ZoneArt::Image(7), "the car's dial wins");
    assert_eq!(zone_art(None, maker, &names), ZoneArt::Image(1), "else the maker's");
    assert_eq!(zone_art::<u32>(None, None, &names), ZoneArt::Names, "else the name");
    assert_eq!(zone_art(None, maker, &Logo::default()), ZoneArt::Nothing, "off");
}
