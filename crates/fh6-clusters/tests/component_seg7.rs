//! Visual shot of the Seg7Digit VFD component (COMPONENTS.md §15) on OLED black: speed states
//! (idle, cruise, fast, clamped), 4-digit rpm (live and at the limiter in the accent phosphor), a
//! decimal and a colon readout, the hero gear mid-sequence and settled, reverse, and the
//! `segment` TabularSpeed convenience at the spec cap of 120. Written to
//! `target/component-shots/seg7.png`.
mod support;

use fh6_clusters::common::seg7::{
    draw_scanlines, draw_speed, Emission, Leading, Phosphor, Reveal, Seg7Digit, Seg7Geometry, Seg7Number, Separator,
};
use fh6_clusters::common::SpeedReadout;
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Gear, Palette, Rect, Signals, TextStyle, VAlign, Vec2};

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

fn gear_char(gear: Gear) -> char {
    match gear {
        Gear::Reverse => 'r',
        Gear::Neutral => 'n',
        Gear::Forward(n) => char::from_digit(u32::from(n % 10), 10).unwrap_or('-'),
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

    // Row 1: speed states, 3 cells.
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
    let mut rpm_live = Seg7Number::<4>::new(&tokens, Reveal::Instant);
    let mut rpm_limit = Seg7Number::<4>::new(&tokens, Reveal::Instant).with_phosphor(Phosphor::Accent);
    let mut boost = Seg7Number::<3>::new(&tokens, Reveal::Instant)
        .with_separator(Separator::Decimal { places: 1 })
        .with_phosphor(Phosphor::Secondary);
    let lap_geom = Seg7Geometry::new(130.0);
    let mut lap = Seg7Number::<3>::new(&tokens, Reveal::Instant)
        .with_separator(Separator::Colon { after: 1 })
        .with_leading(Leading::Zeros);
    let live = signals(87.7, 3250.0, Gear::Forward(4));
    let limit = signals(152.2, 7980.0, Gear::Forward(5));
    for _ in 0..30 {
        rpm_live.update(live.rpm.round() as i64, dt);
        rpm_limit.update(limit.rpm.round() as i64, dt);
        boost.update(147, dt);
        lap.update(142, dt);
    }

    // Row 3: the gear (white ink) mid-sequence and settled, reverse, and the limiter cue.
    let gear_geom = Seg7Geometry::new(300.0);
    let mut gear_mid = Seg7Digit::new(&tokens, Reveal::Sequence).with_phosphor(Phosphor::Ink);
    let mut gear_set = Seg7Digit::new(&tokens, Reveal::Sequence).with_phosphor(Phosphor::Ink);
    let mut gear_rev = Seg7Digit::new(&tokens, Reveal::Sequence).with_phosphor(Phosphor::Warning);
    let mut gear_lim = Seg7Digit::new(&tokens, Reveal::Sequence).with_phosphor(Phosphor::Accent);
    for _ in 0..30 {
        gear_mid.update_char(gear_char(Gear::Forward(3)), dt);
        gear_set.update_char(gear_char(live.gear), dt);
        gear_rev.update_char(gear_char(Gear::Reverse), dt);
        gear_lim.update_char(gear_char(limit.gear), dt);
    }
    // Upshift 3 -> 4: 33 ms into the 80 ms re-strike (b and c lit, f and g still to come).
    for _ in 0..3 {
        gear_mid.update_char(gear_char(Gear::Forward(4)), dt);
    }
    assert!(gear_mid.animating(), "the gear change is mid-sequence");
    assert!(!gear_set.animating() && !rpm_live.animating());

    let speed_x = [-1380.0, -460.0, 460.0, 1380.0];
    let speed_y = -640.0;
    let row2_x = [-1310.0, -430.0, 380.0, 1210.0];
    let row2_y = -120.0;
    let gear_x = [-1500.0, -950.0, -400.0, 150.0];
    let gear_y = 520.0;
    let small_speed = vec2(1150.0, 560.0);
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
        (vec2(gear_x[2], gear_y + 220.0), "REVERSE"),
        (vec2(gear_x[3], gear_y + 220.0), "LIMITER"),
        (vec2(small_speed.x, gear_y + 220.0), "SPEED CAP 120"),
    ];

    let img = gpu.render(&layout, |p, _| {
        let pal = *p.palette();
        for (num, x) in speeds.iter().zip(speed_x) {
            num.draw(p, vec2(x, speed_y), &speed_geom, &pal, Emission::Bloom);
        }
        rpm_live.draw(p, vec2(row2_x[0], row2_y), &rpm_geom, &pal, Emission::Bloom);
        rpm_limit.draw(p, vec2(row2_x[1], row2_y), &rpm_geom, &pal, Emission::Bloom);
        boost.draw(p, vec2(row2_x[2], row2_y), &rpm_geom, &pal, Emission::Bloom);
        lap.draw(p, vec2(row2_x[3], row2_y), &lap_geom, &pal, Emission::Matte);
        gear_mid.draw(p, vec2(gear_x[0], gear_y), &gear_geom, &pal, Emission::Bloom);
        gear_set.draw(p, vec2(gear_x[1], gear_y), &gear_geom, &pal, Emission::Bloom);
        gear_rev.draw(p, vec2(gear_x[2], gear_y), &gear_geom, &pal, Emission::Bloom);
        gear_lim.draw(p, vec2(gear_x[3], gear_y), &gear_geom, &pal, Emission::Bloom);
        draw_speed(p, small_speed, 120.0, 88, &pal);
        // The cavity's scanlines over rows 1 and 2 only, so row 3 shows the bare phosphor.
        draw_scanlines(p, Rect::from_xywh(-1900.0, -820.0, 3800.0, 860.0), &pal);
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

    // Cruise 88: the leading cell shows only the ghost 8, the two digits are lit phosphor.
    let cruise = speeds[1];
    assert_eq!(cruise.value(), 88);
    let c = vec2(speed_x[1], speed_y);
    let ghost = segment_pixel(&img, &speed_geom, cruise.cell_center(c, &speed_geom, 0), 6);
    let lit = segment_pixel(&img, &speed_geom, cruise.cell_center(c, &speed_geom, 1), 6);
    assert!(
        (6..=60).contains(&ghost[1]),
        "ghost segment is dim phosphor: {ghost:?} ({})",
        path.display()
    );
    assert!(ghost[0] < ghost[1], "ghost keeps the cyan hue: {ghost:?}");
    assert!(lit[1] > 150 && lit[1] > 3 * ghost[1], "lit segment: {lit:?}");

    // The clamped readout shows 999 with every segment of `9` lit (f lit, e unlit).
    let clamped = speeds[3];
    assert_eq!(clamped.value(), 999);
    let cc = clamped.cell_center(vec2(speed_x[3], speed_y), &speed_geom, 2);
    assert!(segment_pixel(&img, &speed_geom, cc, 5)[1] > 150, "segment f of 9 lit");
    assert!(
        segment_pixel(&img, &speed_geom, cc, 4)[1] < 60,
        "segment e of 9 ghosted"
    );

    // The limiter rpm is magenta: red over green.
    let lim = segment_pixel(
        &img,
        &rpm_geom,
        rpm_limit.cell_center(vec2(row2_x[1], row2_y), &rpm_geom, 0),
        0,
    );
    assert!(lim[0] > 150 && lim[0] > lim[1], "accent phosphor: {lim:?}");

    // The settled gear 4 is near-white ink; its segment a is a ghost.
    let g4 = vec2(gear_x[1], gear_y);
    let gear_lit = segment_pixel(&img, &gear_geom, g4, 6);
    assert!(
        gear_lit.iter().take(3).all(|&v| v > 180),
        "white gear core: {gear_lit:?}"
    );
    let gear_ghost = segment_pixel(&img, &gear_geom, g4, 0);
    assert!(gear_ghost[1] < 60, "gear ghost segment: {gear_ghost:?}");

    // Mid-sequence: segment b of the new 4 is lit, the middle bar (last in the order) not yet.
    let gm = vec2(gear_x[0], gear_y);
    assert!(segment_pixel(&img, &gear_geom, gm, 1)[1] > 180, "sequence started");
    assert!(
        segment_pixel(&img, &gear_geom, gm, 6)[1] < 120,
        "middle bar lights last"
    );

    // OLED black between the readouts: no fill, no haze.
    let (bx, by) = to_px(vec2(-920.0, 230.0));
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
}
