//! Visual shot of the Seg7Digit VFD component (COMPONENTS.md §15) on OLED black, written to
//! `target/component-shots/seg7.png`.
//!
//! Rows 1–3 sit in an opaque true-black VFD cavity (`materials.vfdCavity`), as in the Sci-Fi
//! clusters, with the 2 px scanlines over rows 1 and 2 only:
//! 1. speed, 3 cells: idle 0, cruise 88, fast 152, 1500 clamped to 999;
//! 2. 4-digit rpm live and at the limiter (accent phosphor on cyan ghosts), boost 14.7 with a
//!    decimal point in the secondary phosphor, a matte (rank 4) lap time with a colon;
//! 3. the white hero gear on the gear bay's diagonal glass with cyan ghosts, matte as HeroGear asks
//!    outside the limiter: the 3 -> 4 re-strike mid-sequence, 4 settled, the diagonal `N` at 72%,
//!    reverse in `warning`, and 5 at the limiter with the soft glow (`Emission::Soft`).
//!
//! Bottom right: the `segment` TabularSpeed convenience at the spec cap of 120, straight on the
//! transparent overlay (no cavity), to show the ghosts hold the same 7% there.
//!
//! A second shot, `seg7_true_size.png`, renders the RetroCyber sizes at 1 target px per 4K px,
//! where the bloom radius (fixed in device px) relates to the digits as it does on the display.
mod support;

use fh6_clusters::common::seg7::{
    draw_scanlines, draw_speed, Emission, Leading, Phosphor, Reveal, Seg7Digit, Seg7Geometry, Seg7Number, Segments,
    Separator, GHOST_LEVEL,
};
use fh6_clusters::common::SpeedReadout;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Gear, Layout, Palette, Rect, Shape, Signals, TextStyle, VAlign, Vec2};

/// Shot scale: target px per local (4K) px.
const SCALE: f32 = support::H as f32 / 2160.0;

fn to_px(p: Vec2) -> (u32, u32) {
    let x = support::W as f32 * 0.5 + p.x * SCALE;
    let y = support::H as f32 * 0.5 + p.y * SCALE;
    (x.round().max(0.0) as u32, y.round().max(0.0) as u32)
}

/// The pixel at the centre of segment `index` of the digit centred at `center`.
fn segment_pixel(img: &RgbaImage, geom: &Seg7Geometry, center: Vec2, index: usize) -> [u8; 4] {
    let pts = geom.segment(center, index);
    let sum = pts.iter().fold(Vec2::ZERO, |s, p| s + *p);
    let (x, y) = to_px(sum * (1.0 / pts.len() as f32));
    img.pixel(x, y).unwrap_or([0; 4])
}

/// The mean green of a 3 x 3 block at the centre of a segment (smooths the scanlines out).
fn segment_green(img: &RgbaImage, geom: &Seg7Geometry, center: Vec2, index: usize) -> f32 {
    let pts = geom.segment(center, index);
    let sum = pts.iter().fold(Vec2::ZERO, |s, p| s + *p);
    let (x, y) = to_px(sum * (1.0 / pts.len() as f32));
    let mut total = 0.0;
    for dy in 0..3 {
        for dx in 0..3 {
            let px = img.pixel(x + dx - 1, y + dy - 1).unwrap_or([0; 4]);
            total += f32::from(px[1]);
        }
    }
    total / 9.0
}

fn signals(speed: f32, rpm: f32, gear: Gear) -> Signals {
    Signals {
        race_on: true,
        speed,
        rpm,
        max_rpm: 8000.0,
        shift_rpm: 7600.0,
        rpm_fraction: rpm / 8000.0,
        shift_fraction: rpm / 7600.0,
        gear,
        ..Signals::default()
    }
}

#[test]
fn seg7_shot() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "sciFi");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let dt = support::DT;

    // Row 1: speed states, 3 cells, through the hysteresis filter like a cluster would.
    let speed_geom = Seg7Geometry::new(200.0);
    let speed_sigs = [
        signals(0.0, 900.0, Gear::Neutral),
        signals(87.7, 3250.0, Gear::Forward(4)),
        signals(152.2, 6100.0, Gear::Forward(5)),
        signals(1500.0, 7980.0, Gear::Forward(6)),
    ];
    let mut speeds = [Seg7Number::<3>::new(&tokens, Reveal::Instant); 4];
    let mut readouts = [SpeedReadout::new(); 4];
    for _ in 0..30 {
        for ((num, readout), sig) in speeds.iter_mut().zip(readouts.iter_mut()).zip(&speed_sigs) {
            readout.update(sig.speed);
            num.update(readout.value(), dt);
        }
    }

    // Row 2: rpm live and at the limiter, boost with a decimal point, a lap time with a colon.
    let rpm_geom = Seg7Geometry::new(170.0);
    let live = signals(87.7, 3250.0, Gear::Forward(4));
    let limit = signals(152.2, 7980.0, Gear::Forward(5));
    let mut rpm_live = Seg7Number::<4>::new(&tokens, Reveal::Instant);
    let mut rpm_limit = Seg7Number::<4>::new(&tokens, Reveal::Instant);
    let mut boost = Seg7Number::<3>::new(&tokens, Reveal::Instant)
        .with_separator(Separator::Decimal { places: 1 })
        .with_phosphor(Phosphor::Secondary);
    let lap_geom = Seg7Geometry::new(130.0);
    let mut lap = Seg7Number::<3>::new(&tokens, Reveal::Instant)
        .with_separator(Separator::Colon { after: 1 })
        .with_leading(Leading::Zeros);
    for _ in 0..30 {
        rpm_live.update(live.rpm.round() as i64, dt);
        rpm_limit.update(limit.rpm.round() as i64, dt);
        boost.update(147, dt);
        lap.update(142, dt);
    }
    // The limiter switches the lit phosphor only; the glass (ghosts) stays cyan.
    if limit.rpm >= limit.shift_rpm {
        rpm_limit.set_phosphor(Phosphor::Accent);
    }

    // Row 3: the white gear on the panel's cyan ghosts.
    let gear_geom = Seg7Geometry::new(300.0);
    let hero_gear = Seg7Digit::new(&tokens, Reveal::Sequence)
        .with_segments(Segments::Diagonal)
        .with_phosphor(Phosphor::Ink)
        .with_ghost(Phosphor::Primary);
    let mut gear_mid = hero_gear;
    let mut gear_set = hero_gear;
    let mut gear_n = hero_gear;
    let mut gear_rev = hero_gear;
    let mut gear_lim = hero_gear;
    gear_rev.set_phosphor(Phosphor::Warning);
    gear_n.set_brightness(0.72);
    for _ in 0..30 {
        gear_mid.update_gear(Gear::Forward(3), dt);
        gear_set.update_gear(live.gear, dt);
        gear_n.update_gear(Gear::Neutral, dt);
        gear_rev.update_gear(Gear::Reverse, dt);
        gear_lim.update_gear(limit.gear, dt);
    }
    // HeroGear (COMPONENTS.md §1): a soft glow only while the limiter is active.
    let gear_emission = |sig: &Signals| {
        if sig.rpm >= sig.shift_rpm {
            Emission::Soft
        } else {
            Emission::Matte
        }
    };
    let (normal, at_limit) = (gear_emission(&live), gear_emission(&limit));
    // Upshift 3 -> 4, 67 ms into the 80 ms re-strike: b, c and f lit, the middle bar still to come.
    for _ in 0..5 {
        gear_mid.update_gear(Gear::Forward(4), dt);
    }
    assert!(gear_mid.animating(), "the gear change is mid-sequence");
    assert!(!gear_set.animating() && !rpm_live.animating());

    let speed_x = [-1380.0, -460.0, 460.0, 1380.0];
    let speed_y = -640.0;
    let row2_x = [-1310.0, -430.0, 380.0, 1210.0];
    let row2_y = -120.0;
    let gear_x = [-1560.0, -1080.0, -600.0, -120.0, 360.0];
    let gear_y = 520.0;
    let bare_speed = vec2(1350.0, 540.0);
    let upper_cavity = Rect::from_xywh(-1900.0, -860.0, 3800.0, 900.0);
    let lower_cavity = Rect::from_xywh(-1900.0, 260.0, 2560.0, 560.0);
    let labels = [
        (vec2(speed_x[0], speed_y + 160.0), "IDLE 0"),
        (vec2(speed_x[1], speed_y + 160.0), "CRUISE 88"),
        (vec2(speed_x[2], speed_y + 160.0), "FAST 152"),
        (vec2(speed_x[3], speed_y + 160.0), "CLAMPED 1500"),
        (vec2(row2_x[0], row2_y + 140.0), "RPM"),
        (vec2(row2_x[1], row2_y + 140.0), "RPM LIMITER"),
        (vec2(row2_x[2], row2_y + 140.0), "BOOST 14.7"),
        (vec2(row2_x[3], row2_y + 140.0), "LAP  MATTE"),
        (vec2(gear_x[0], gear_y + 220.0), "3>4 SEQUENCE"),
        (vec2(gear_x[1], gear_y + 220.0), "GEAR"),
        (vec2(gear_x[2], gear_y + 220.0), "NEUTRAL 72%"),
        (vec2(gear_x[3], gear_y + 220.0), "REVERSE"),
        (vec2(gear_x[4], gear_y + 220.0), "LIMITER GLOW"),
        (vec2(bare_speed.x, gear_y + 220.0), "CAP 120  NO CAVITY"),
    ];

    let img = gpu.render(&layout, |p, _| {
        let pal = *p.palette();
        // The VFD cavities: opaque true black, as the Sci-Fi bezel fills them.
        p.fill(&Shape::rect(upper_cavity), pal.roles.oled_black);
        p.fill(&Shape::rect(lower_cavity), pal.roles.oled_black);
        for (num, x) in speeds.iter().zip(speed_x) {
            num.draw(p, vec2(x, speed_y), &speed_geom, &pal, Emission::Bloom);
        }
        rpm_live.draw(p, vec2(row2_x[0], row2_y), &rpm_geom, &pal, Emission::Bloom);
        rpm_limit.draw(p, vec2(row2_x[1], row2_y), &rpm_geom, &pal, Emission::Bloom);
        boost.draw(p, vec2(row2_x[2], row2_y), &rpm_geom, &pal, Emission::Bloom);
        lap.draw(p, vec2(row2_x[3], row2_y), &lap_geom, &pal, Emission::Matte);
        gear_mid.draw(p, vec2(gear_x[0], gear_y), &gear_geom, &pal, normal);
        gear_set.draw(p, vec2(gear_x[1], gear_y), &gear_geom, &pal, normal);
        gear_n.draw(p, vec2(gear_x[2], gear_y), &gear_geom, &pal, normal);
        gear_rev.draw(p, vec2(gear_x[3], gear_y), &gear_geom, &pal, normal);
        gear_lim.draw(p, vec2(gear_x[4], gear_y), &gear_geom, &pal, at_limit);
        draw_speed(p, bare_speed, 120.0, 88, &pal);
        // The cavity's scanlines over rows 1 and 2 only, so row 3 shows the bare phosphor.
        draw_scanlines(p, upper_cavity, &pal);
        let font = p.fonts().role(FontRole::Tech);
        let ts = p.tokens().type_scale;
        let style = TextStyle::new(font, ts.micro_label * 1.5)
            .paint(pal.roles.ink_label)
            .tracking(ts.label_tracking)
            .align(Align::Center, VAlign::CapTop);
        for (pos, text) in labels {
            p.text(pos, text, &style);
        }
    });
    let path = support::save_shot("seg7", &img);
    let shot = path.display();

    // The ghost level the palette asks for: 7% of the lit phosphor's displayed green.
    let lit_green = f32::from(palette.primary_text.to_srgb8()[1]);
    let ghost_green = GHOST_LEVEL * lit_green;

    // Cruise 88: the leading cell shows only the ghost 8 at 7%, the two digits are lit phosphor.
    let cruise = speeds[1];
    assert_eq!(cruise.value(), 88);
    let c = vec2(speed_x[1], speed_y);
    let ghost = segment_green(&img, &speed_geom, cruise.cell_center(c, &speed_geom, 0), 6);
    assert!(
        ghost > ghost_green * 0.6 && ghost < ghost_green * 2.5,
        "ghost segment is dim phosphor: {ghost} vs {ghost_green} ({shot})"
    );
    let ghost_px = segment_pixel(&img, &speed_geom, cruise.cell_center(c, &speed_geom, 0), 3);
    assert!(ghost_px[0] < ghost_px[1], "ghost keeps the cyan hue: {ghost_px:?}");
    let lit = segment_pixel(&img, &speed_geom, cruise.cell_center(c, &speed_geom, 1), 6);
    assert!(lit[1] > 200, "lit segment: {lit:?}");

    // Idle 0: both leading cells are ghosts, and the one next to the lit 0 is not swallowed by
    // its halo (it still reads as a segment, not as haze).
    let idle = speeds[0];
    assert_eq!(idle.value(), 0);
    let ic = vec2(speed_x[0], speed_y);
    let far = segment_green(&img, &speed_geom, idle.cell_center(ic, &speed_geom, 0), 6);
    let near = segment_green(&img, &speed_geom, idle.cell_center(ic, &speed_geom, 1), 6);
    assert!(far > ghost_green * 0.6, "far ghost visible: {far}");
    assert!(near >= far && near < lit_green * 0.4, "near ghost: {near} (far {far})");

    // The clamped readout shows 999 with every segment of `9` lit (f lit, e unlit).
    let clamped = speeds[3];
    assert_eq!(clamped.value(), 999);
    let cc = clamped.cell_center(vec2(speed_x[3], speed_y), &speed_geom, 2);
    let f = segment_green(&img, &speed_geom, cc, 5);
    let e = segment_green(&img, &speed_geom, cc, 4);
    assert!(f > 200.0, "segment f of 9 lit: {f}");
    assert!(e < f * 0.45, "segment e of 9 ghosted: {e} vs lit {f}");

    // The limiter rpm is magenta (red over green) on cyan ghosts.
    let lc = vec2(row2_x[1], row2_y);
    let lim = segment_pixel(&img, &rpm_geom, rpm_limit.cell_center(lc, &rpm_geom, 0), 0);
    assert!(lim[0] > 150 && lim[0] > lim[1], "accent phosphor: {lim:?}");

    // The settled gear 4 is near-white ink; its segment a is a cyan ghost.
    let g4 = vec2(gear_x[1], gear_y);
    let gear_lit = segment_pixel(&img, &gear_geom, g4, 6);
    assert!(
        gear_lit.iter().take(3).all(|&v| v > 200),
        "white gear core: {gear_lit:?}"
    );
    let gear_ghost = segment_pixel(&img, &gear_geom, g4, 0);
    assert!(
        gear_ghost[1] > gear_ghost[0] && gear_ghost[1] < 60,
        "gear ghost is the cyan panel phosphor: {gear_ghost:?}"
    );

    // Neutral at 72% reads dimmer than the settled gear.
    let gn = vec2(gear_x[2], gear_y);
    let n_lit = segment_pixel(&img, &gear_geom, gn, 1);
    let g4_lit = segment_pixel(&img, &gear_geom, g4, 1);
    assert!(n_lit[0] < g4_lit[0], "N at 72%: {n_lit:?} vs {g4_lit:?}");
    // The hero frames' N: its diagonal crosses the cell middle, over the ghost middle bar.
    let (nx, ny) = to_px(gn);
    let n_mid = img.pixel(nx, ny).unwrap_or([0; 4]);
    assert!(n_mid[0] > 150, "N diagonal lit through the middle: {n_mid:?}");

    // Mid-sequence: b, c and f of the new 4 are lit, the middle bar (last in the order) not yet.
    let gm = vec2(gear_x[0], gear_y);
    assert!(segment_pixel(&img, &gear_geom, gm, 1)[1] > 200, "sequence started");
    assert!(segment_pixel(&img, &gear_geom, gm, 5)[1] > 200, "f lit at 67 ms");
    assert!(
        segment_pixel(&img, &gear_geom, gm, 6)[1] < 120,
        "middle bar lights last"
    );

    // On the bare overlay the ghosts hold the same 7%.
    let bare_geom = Seg7Geometry::new(120.0);
    let bare_cells = Seg7Number::<3>::new(&tokens, Reveal::Instant);
    let bare = segment_green(&img, &bare_geom, bare_cells.cell_center(bare_speed, &bare_geom, 0), 3);
    assert!(
        bare > ghost_green * 0.6 && bare < ghost_green * 2.5,
        "ghost without a cavity: {bare} vs {ghost_green}"
    );

    // OLED black between the cavities: no fill, no haze.
    let (bx, by) = to_px(vec2(-920.0, 150.0));
    let off = img.pixel(bx, by).unwrap_or([255; 4]);
    assert!(off[0] < 4 && off[1] < 4 && off[2] < 4, "off pixels stay black: {off:?}");

    // A meaningful amount of phosphor lit in the speed row.
    let row_top = to_px(vec2(0.0, speed_y - 120.0)).1;
    let row_bottom = to_px(vec2(0.0, speed_y + 120.0)).1;
    let lit_count = (row_top..row_bottom)
        .flat_map(|y| (0..support::W).map(move |x| (x, y)))
        .filter(|&(x, y)| img.pixel(x, y).is_some_and(|px| px[1] > 128))
        .count();
    assert!(lit_count > 2000, "speed row lit pixels: {lit_count}");

    // The soft limiter glow keeps the white 5 crisp: its unlit segment e stays a dim ghost
    // instead of drowning in the halo.
    let g5 = vec2(gear_x[4], gear_y);
    let e5 = segment_pixel(&img, &gear_geom, g5, 4);
    assert!(
        e5[1] < 70 && e5[1] > e5[0],
        "limiter 5's ghost e stays a cyan ghost: {e5:?}"
    );
}

/// Mean RGB of a 3 x 3 block around the local point `p` under `layout`.
fn mean_rgb(img: &RgbaImage, layout: &Layout, p: Vec2) -> [f32; 3] {
    let t = layout.to_target(p);
    let (x, y) = (t.x.round().max(1.0) as u32, t.y.round().max(1.0) as u32);
    let mut sum = [0.0; 3];
    for dy in 0..3 {
        for dx in 0..3 {
            let px = img.pixel(x + dx - 1, y + dy - 1).unwrap_or([0; 4]);
            for (s, v) in sum.iter_mut().zip(px) {
                *s += f32::from(v) / 9.0;
            }
        }
    }
    sum
}

/// The centre of segment `index` of the digit centred at `center`.
fn segment_center(geom: &Seg7Geometry, center: Vec2, index: usize) -> Vec2 {
    let pts = geom.segment(center, index);
    pts.iter().fold(Vec2::ZERO, |s, p| s + *p) * (1.0 / pts.len() as f32)
}

/// The RetroCyber sizes at true 4K scale (sci-fi.md: ghost-888 speed at cap 120, the 236 px bay
/// gear) plus 90 px rpm and boost readouts, in one opaque cavity under the 2 px scanlines.
#[test]
fn seg7_true_size_shot() {
    let mut gpu = support::gpu();
    let center = vec2(support::W as f32 * 0.5, support::H as f32 * 0.5);
    let layout = Layout::centered(center, 1.0, (support::W, support::H));
    let palette = Palette::family(&gpu.tokens, "sciFi");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();
    let dt = support::DT;

    let speed_geom = Seg7Geometry::new(120.0);
    let gear_geom = Seg7Geometry::new(236.0);
    let small_geom = Seg7Geometry::new(90.0);
    let mut speed = Seg7Number::<3>::new(&tokens, Reveal::Instant);
    let mut idle = Seg7Number::<3>::new(&tokens, Reveal::Instant);
    let mut rpm = Seg7Number::<4>::new(&tokens, Reveal::Instant);
    let mut boost = Seg7Number::<3>::new(&tokens, Reveal::Instant)
        .with_separator(Separator::Decimal { places: 1 })
        .with_phosphor(Phosphor::Secondary);
    let gear = Seg7Digit::new(&tokens, Reveal::Sequence)
        .with_segments(Segments::Diagonal)
        .with_phosphor(Phosphor::Ink)
        .with_ghost(Phosphor::Primary);
    let (mut cruise_gear, mut limiter_gear) = (gear, gear);
    let mut readout = SpeedReadout::new();
    for _ in 0..30 {
        readout.update(152.2);
        speed.update(readout.value(), dt);
        idle.update(0, dt);
        rpm.update(7980, dt);
        boost.update(147, dt);
        cruise_gear.update_gear(Gear::Forward(4), dt);
        limiter_gear.update_gear(Gear::Forward(5), dt);
    }
    rpm.set_phosphor(Phosphor::Accent);

    let cavity = Rect::from_xywh(-930.0, -500.0, 1860.0, 1000.0);
    let (speed_at, idle_at) = (vec2(-600.0, -250.0), vec2(-600.0, 220.0));
    let (gear_at, limiter_at) = (vec2(-110.0, -230.0), vec2(-110.0, 230.0));
    let (rpm_at, boost_at) = (vec2(480.0, -250.0), vec2(480.0, 220.0));
    let img = gpu.render(&layout, |p, _| {
        let pal = *p.palette();
        p.fill(&Shape::rect(cavity), pal.roles.oled_black);
        speed.draw(p, speed_at, &speed_geom, &pal, Emission::Bloom);
        idle.draw(p, idle_at, &speed_geom, &pal, Emission::Bloom);
        cruise_gear.draw(p, gear_at, &gear_geom, &pal, Emission::Matte);
        limiter_gear.draw(p, limiter_at, &gear_geom, &pal, Emission::Soft);
        rpm.draw(p, rpm_at, &small_geom, &pal, Emission::Bloom);
        boost.draw(p, boost_at, &small_geom, &pal, Emission::Bloom);
        draw_scanlines(p, cavity, &pal);
    });
    let path = support::save_shot("seg7_true_size", &img);
    let shot = path.display();

    // The leading ghost 8 of 152 at 7% of the phosphor, lit digits at full phosphor.
    let lit_green = f32::from(palette.primary_text.to_srgb8()[1]);
    let ghost_green = GHOST_LEVEL * lit_green;
    let ghost = mean_rgb(
        &img,
        &layout,
        segment_center(&speed_geom, idle.cell_center(idle_at, &speed_geom, 0), 6),
    );
    assert!(
        ghost[1] > ghost_green * 0.5 && ghost[1] < ghost_green * 2.0,
        "ghost 8: {ghost:?} vs {ghost_green} ({shot})"
    );
    let lit = mean_rgb(
        &img,
        &layout,
        segment_center(&speed_geom, speed.cell_center(speed_at, &speed_geom, 1), 0),
    );
    assert!(lit[1] > 180.0, "lit segment a of 5: {lit:?}");

    // Crisp digits: the halo stays short, so the lit 0's counters stay black (the per-segment
    // halos add up inside a digit).
    let zero = idle.cell_center(idle_at, &speed_geom, 2);
    for v in [-0.25, 0.25] {
        let counter = mean_rgb(&img, &layout, speed_geom.map(zero, vec2(0.0, v * speed_geom.height)));
        assert!(counter[1] < 16.0, "the 0's counter at {v}: {counter:?}");
    }

    // Tight VFD halo (tokens glow.phosphor): 40 px above the digits the cavity is black again.
    let above = mean_rgb(
        &img,
        &layout,
        vec2(speed_at.x, speed_at.y - speed_geom.height * 0.5 - 40.0),
    );
    assert!(above[1] < 10.0, "phosphor veil 40 px above the speed: {above:?}");
    // The soft limiter glow fades within the bay: 80 px beside the gear it is gone.
    let edge = gear_geom.bounds(limiter_at).max.x;
    let beside = mean_rgb(&img, &layout, vec2(edge + 80.0, limiter_at.y));
    assert!(beside.iter().all(|&v| v < 12.0), "limiter glow 80 px out: {beside:?}");
}
