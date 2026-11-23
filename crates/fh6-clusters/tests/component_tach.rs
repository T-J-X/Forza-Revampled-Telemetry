mod support;

// Visual shot test for the tach variants `arc` (Modern Digital), `bar` (Race Dash) and `chevron`
// (Sci-Fi VFD), each settled on idle, mid, redline and limiter side by side (the bar adds a
// peak-hold row). The shots land in `target/component-shots/tach_*.png`; the pixel checks pin
// the lit extent and the stage colours of each state.
//
// Signals are built directly (rpm, limiter, shift rpm and the derived fractions): the same
// fields `SignalSmoother` fills from the verified packet fields.

use fh6_clusters::common::tach::{TachArc, TachBar, TachChevron};
use fh6_kit::testing::RgbaImage;
use fh6_kit::{vec2, Align, FontRole, Palette, Rect, Signals, TextStyle, Tokens, VAlign, Vec2};

/// Frames each state is settled for.
const FRAMES: usize = 30;
/// Limiter rpm of the test car.
const LIMITER: f32 = 8700.0;
/// Idle rpm.
const IDLE: f32 = 900.0;
/// The four review states: name and engine rpm.
const STATES: [(&str, f32); 4] = [
    ("IDLE", IDLE),
    ("MID", 5200.0),
    ("REDLINE", 8200.0),
    ("LIMITER", 8700.0),
];

/// Signals for a steady `rpm` (no climb, so no feed-forward lead).
fn signals(tokens: &Tokens, rpm: f32) -> Signals {
    let shift = LIMITER * tokens.rev_stages.shift_pct;
    Signals {
        race_on: true,
        rpm,
        max_rpm: LIMITER,
        idle_rpm: IDLE,
        shift_rpm: shift,
        rpm_fraction: (rpm / LIMITER).clamp(0.0, 1.0),
        shift_fraction: rpm / shift,
        ..Signals::default()
    }
}

/// Local (4K, anchor-centred) to shot pixels.
fn to_px(p: Vec2) -> (i64, i64) {
    let s = support::H as f32 / 2160.0;
    (
        (p.x * s + support::W as f32 * 0.5).round() as i64,
        (p.y * s + support::H as f32 * 0.5).round() as i64,
    )
}

/// Lit-pixel count and the mean colour of lit pixels in a local rect.
fn lit(img: &RgbaImage, r: Rect) -> (u32, [f32; 3]) {
    let (x0, y0) = to_px(r.min);
    let (x1, y1) = to_px(r.max);
    let mut n = 0u32;
    let mut sum = [0.0f32; 3];
    for y in y0.max(0)..y1.min(i64::from(img.height)) {
        for x in x0.max(0)..x1.min(i64::from(img.width)) {
            let i = ((y as u32 * img.width + x as u32) * 4) as usize;
            let px = &img.pixels[i..i + 4];
            let (r, g, b, a) = (px[0], px[1], px[2], px[3]);
            if a > 40 && u32::from(r) + u32::from(g) + u32::from(b) > 150 {
                n += 1;
                sum[0] += f32::from(r);
                sum[1] += f32::from(g);
                sum[2] += f32::from(b);
            }
        }
    }
    let k = 1.0 / n.max(1) as f32;
    (n, [sum[0] * k, sum[1] * k, sum[2] * k])
}

/// A state caption in the label ink.
fn caption(p: &mut fh6_kit::Painter<'_>, at: Vec2, text: &str, align: Align, palette: &Palette) {
    let font = p.fonts().role(FontRole::Text);
    let ts = p.tokens().type_scale;
    let style = TextStyle::new(font, ts.micro_label * 1.5)
        .paint(palette.roles.ink_label)
        .tracking(ts.label_tracking)
        .align(align, VAlign::CapMiddle);
    p.text(at, text, &style);
}

// ---------------------------------------------------------------------------------------------
// Arc (Modern Digital).

/// Column centres (local 4K px) of the arc shot.
const ARC_COLUMNS: [f32; 4] = [-1440.0, -480.0, 480.0, 1440.0];
/// Arc centre y, track radius and thickness.
const ARC_Y: f32 = 60.0;
const ARC_R: f32 = 330.0;
const ARC_T: f32 = 24.0;

#[test]
fn tach_arc_modern_digital() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "modernDigital");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();

    let arcs: Vec<TachArc> = STATES
        .iter()
        .map(|&(_, rpm)| {
            let mut arc = TachArc::new(&tokens);
            for _ in 0..FRAMES {
                arc.update(&signals(&tokens, rpm), support::DT);
            }
            arc
        })
        .collect();
    assert!(
        arcs.iter().all(|a| !a.animating()),
        "every arc settled in {FRAMES} frames"
    );
    assert!(
        arcs[3].limiter() > 0.99 && arcs[2].limiter() < 0.01,
        "limiter cue only on the limiter"
    );

    let img = gpu.render(&layout, |p, _| {
        let display = p.fonts().role(FontRole::Display);
        let numeral = p.tokens().type_scale.dial_numeral;
        for ((name, _), (arc, cx)) in STATES.iter().zip(arcs.iter().zip(ARC_COLUMNS)) {
            caption(p, vec2(cx, -560.0), name, Align::Center, &palette);
            let center = vec2(cx, ARC_Y);
            arc.draw(p, center, ARC_R, ARC_T, &palette);
            arc.draw_numerals(p, center, ARC_R - ARC_T - 40.0, display, numeral, &palette);
        }
    });
    let path = support::save_shot("tach_arc", &img);
    assert!(path.exists(), "{}", path.display());

    // A small box on the track centre line at rpm fraction `f` of column `col`.
    let probe = |col: usize, f: f32| {
        let at = Vec2::polar(vec2(ARC_COLUMNS[col], ARC_Y), ARC_R, arcs[col].angle_at(f));
        Rect::from_center_size(at, vec2(ARC_T * 0.6, ARC_T * 0.6))
    };
    // Idle: the fill reaches 10%, the rest of the track is an unlit ghost.
    let (n, _) = lit(&img, probe(0, 0.06));
    assert!(n > 4, "idle fill lit near the start ({n} px)");
    let (n, _) = lit(&img, probe(0, 0.5));
    assert!(n == 0, "idle track unlit at 50% ({n} px)");
    // Mid: primary cyan, unlit above the fill.
    let (n, [r, g, b]) = lit(&img, probe(1, 0.45));
    assert!(
        n > 4 && b > r + 80.0 && g > r + 60.0,
        "mid fill is primary ({n} px, {r} {g} {b})"
    );
    let (n, _) = lit(&img, probe(1, 0.8));
    assert!(n == 0, "mid track unlit at 80% ({n} px)");
    // Redline: the head of the fill is in the warning stage.
    let (n, [r, g, b]) = lit(&img, probe(2, 0.925));
    assert!(
        n > 4 && r > g + 60.0 && r > b + 60.0,
        "redline head is warning red ({n} px, {r} {g} {b})"
    );
    // Limiter: the fill holds the shift colour (violet: red and blue well above the cyan's).
    let (_, [mid_r, _, _]) = lit(&img, probe(1, 0.3));
    let (n, [r, g, b]) = lit(&img, probe(3, 0.3));
    assert!(
        n > 4 && r > mid_r + 80.0 && b > g,
        "limiter fill holds the shift colour ({n} px, {r} {g} {b})"
    );
}

// ---------------------------------------------------------------------------------------------
// Bar (Race Dash).

/// Row centres of the bar shot (four states plus peak hold).
const BAR_ROWS: [f32; 5] = [-760.0, -380.0, 0.0, 380.0, 760.0];
/// Race Dash zone Z3: 1000 px span; the cells are 36 px tall.
const BAR_W: f32 = 1000.0;
const BAR_H: f32 = 36.0;

fn bar_rect(row: usize) -> Rect {
    Rect::from_xywh(-BAR_W * 0.5, BAR_ROWS[row] - BAR_H * 0.5, BAR_W, BAR_H)
}

#[test]
fn tach_bar_race_dash() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "raceDash");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();

    let mut bars: Vec<TachBar> = STATES
        .iter()
        .map(|&(_, rpm)| {
            let mut bar = TachBar::new(&tokens);
            for _ in 0..FRAMES {
                bar.update(&signals(&tokens, rpm), support::DT);
            }
            bar
        })
        .collect();
    // Peak hold: a pull to 8350 rpm, then an upshift drop to 6100 rpm for 12 frames.
    let mut peak = TachBar::new(&tokens).with_peak_hold(&tokens);
    for _ in 0..FRAMES {
        peak.update(&signals(&tokens, 8350.0), support::DT);
    }
    let peak_cell = peak.lit() - 1;
    for _ in 0..12 {
        peak.update(&signals(&tokens, 6100.0), support::DT);
    }
    assert_eq!(
        peak.peak_segment(),
        Some(peak_cell),
        "the peak cell is held after the drop"
    );
    assert!(peak.animating(), "a held peak still animates");
    bars.push(peak);
    assert_eq!(bars[0].lit(), 4, "idle lights 4 of 45");
    assert_eq!(bars[3].lit(), 45, "the limiter lights the whole bar");

    let img = gpu.render(&layout, |p, _| {
        let race = p.fonts().role(FontRole::Race);
        let scale_cap = p.tokens().type_scale.micro_label * 1.5;
        let names = STATES.iter().map(|s| s.0).chain(std::iter::once("PEAK HOLD"));
        for (row, (bar, name)) in bars.iter().zip(names).enumerate() {
            let rect = bar_rect(row);
            caption(
                p,
                vec2(rect.min.x - 60.0, rect.center().y),
                name,
                Align::Right,
                &palette,
            );
            bar.draw(p, rect, &palette);
            bar.draw_scale(p, rect, race, scale_cap, &palette);
        }
    });
    let path = support::save_shot("tach_bar", &img);
    assert!(path.exists(), "{}", path.display());

    let cell = |row: usize, i: u32| {
        let r = bars[row].cell_rect(bar_rect(row), i);
        Rect::from_center_size(r.center(), vec2(r.width() * 0.6, r.height() * 0.6))
    };
    // Idle: 4 cells lit in 72% primary (blue), the fifth a grey lens.
    let (n, [r, _, b]) = lit(&img, cell(0, 2));
    assert!(n > 4 && b > r + 80.0, "idle cell lit in primary ({n} px, r {r} b {b})");
    let (n, _) = lit(&img, cell(0, 5));
    assert!(n == 0, "idle cell 6 unlit ({n} px)");
    // Lit cells are top-lit lenses (a glint at the top edge, a shade toward the bottom), not
    // flat tiles.
    let band = |row: usize, i: u32, from: f32, to: f32| {
        let r = bars[row].cell_rect(bar_rect(row), i);
        Rect::from_xywh(
            r.min.x + r.width() * 0.2,
            r.min.y + r.height() * from,
            r.width() * 0.6,
            r.height() * (to - from),
        )
    };
    let luma = |c: [f32; 3]| c[0] + c[1] + c[2];
    let (_, top) = lit(&img, band(2, 10, 0.08, 0.3));
    let (_, bottom) = lit(&img, band(2, 10, 0.7, 0.92));
    assert!(
        luma(top) > luma(bottom) + 60.0,
        "lit cell is a top-lit lens (top {top:?}, bottom {bottom:?})"
    );
    // Redline: low cells primary, the caution band amber, the head red.
    let (n, [r, _, b]) = lit(&img, cell(2, 20));
    assert!(n > 4 && b > r + 80.0, "redline low cells primary ({n} px, r {r} b {b})");
    let (n, [r, g, b]) = lit(&img, cell(2, 38));
    assert!(
        n > 4 && r > b + 80.0 && g > b + 40.0,
        "caution cell amber ({n} px, {r} {g} {b})"
    );
    let head = bars[2].lit() - 1;
    let (n, [r, g, b]) = lit(&img, cell(2, head));
    assert!(
        n > 4 && r > g + 80.0 && r > b + 80.0,
        "warning head red ({n} px, {r} {g} {b})"
    );
    let (n, _) = lit(&img, cell(2, 44));
    assert!(n == 0, "redline leaves the top cell unlit ({n} px)");
    // Limiter: the whole run is steady ink (neutral white).
    for i in [3, 30, 44] {
        let (n, [r, g, b]) = lit(&img, cell(3, i));
        let spread = r.max(g).max(b) - r.min(g).min(b);
        assert!(
            n > 4 && r.min(g).min(b) > 170.0 && spread < 40.0,
            "limiter cell {i} ink ({n} px, {r} {g} {b})"
        );
    }
    // Peak hold: the held cell is marked, the cells between it and the run are dark.
    let (n, _) = lit(&img, cell(4, peak_cell));
    assert!(n > 4, "peak cell marked ({n} px)");
    let (n, _) = lit(&img, cell(4, peak_cell - 2));
    assert!(n == 0, "cells under the peak are unlit ({n} px)");
}

// ---------------------------------------------------------------------------------------------
// Chevron (Sci-Fi VFD).

/// Row centres of the chevron shot.
const CHEVRON_ROWS: [f32; 4] = [-690.0, -230.0, 230.0, 690.0];
/// The run spans 1100 x 96 px.
const CHEVRON_W: f32 = 1100.0;
const CHEVRON_H: f32 = 96.0;

fn chevron_rect(row: usize) -> Rect {
    Rect::from_xywh(
        -CHEVRON_W * 0.5,
        CHEVRON_ROWS[row] - CHEVRON_H * 0.5,
        CHEVRON_W,
        CHEVRON_H,
    )
}

#[test]
fn tach_chevron_sci_fi() {
    let mut gpu = support::gpu();
    let layout = support::shot_layout();
    let palette = Palette::family(&gpu.tokens, "sciFi");
    gpu.kit.set_palette(palette);
    let tokens = gpu.tokens.clone();

    let runs: Vec<TachChevron> = STATES
        .iter()
        .map(|&(_, rpm)| {
            let mut run = TachChevron::new(&tokens);
            for _ in 0..FRAMES {
                run.update(&signals(&tokens, rpm), support::DT);
            }
            run
        })
        .collect();
    let lit_counts: Vec<u32> = runs.iter().map(TachChevron::lit).collect();
    assert_eq!(lit_counts, [2, 14, 22, 24], "step lighting per state");

    let img = gpu.render(&layout, |p, _| {
        let tech = p.fonts().role(FontRole::Tech);
        let scale_cap = p.tokens().type_scale.micro_label * 1.5;
        for (row, ((name, _), run)) in STATES.iter().zip(&runs).enumerate() {
            let rect = chevron_rect(row);
            caption(
                p,
                vec2(rect.min.x - 60.0, rect.max.y - 20.0),
                name,
                Align::Right,
                &palette,
            );
            run.draw(p, rect, &palette);
            run.draw_scale(p, rect, tech, scale_cap, &palette);
        }
    });
    let path = support::save_shot("tach_chevron", &img);
    assert!(path.exists(), "{}", path.display());

    // A box inside segment `i` of row `row`, near the bottom where every segment is full width.
    let seg = |row: usize, i: u32| {
        let rect = chevron_rect(row);
        let (pts, _) = runs[row].segment(i, rect);
        let w = pts[1].x - pts[0].x;
        let c = vec2((pts[0].x + pts[1].x) * 0.5 + w * 0.1, rect.max.y - 14.0);
        Rect::from_center_size(c, vec2(w * 0.4, 12.0))
    };
    // Idle: two cyan segments lit, the third a 7% ghost (below the lit threshold).
    let (n, [r, g, b]) = lit(&img, seg(0, 1));
    assert!(
        n > 4 && g > r + 80.0 && b > r + 60.0,
        "idle segment cyan ({n} px, {r} {g} {b})"
    );
    let (n, _) = lit(&img, seg(0, 2));
    assert!(n == 0, "idle segment 3 is a ghost ({n} px)");
    // Mid: all 14 cyan lit, the yellow zone ghosted.
    let (n, [r, g, b]) = lit(&img, seg(1, 13));
    assert!(n > 4 && g > r + 80.0, "segment 14 cyan ({n} px, {r} {g} {b})");
    let (n, _) = lit(&img, seg(1, 15));
    assert!(n == 0, "mid yellow zone is a ghost ({n} px)");
    // Redline: yellow zone lit, two magenta lit, the last two ghosted.
    let (n, [r, g, b]) = lit(&img, seg(2, 17));
    assert!(
        n > 4 && r > b + 80.0 && g > b + 80.0,
        "yellow zone ({n} px, {r} {g} {b})"
    );
    let (n, [r, g, b]) = lit(&img, seg(2, 21));
    assert!(n > 4 && r > g + 80.0 && b > g, "magenta zone ({n} px, {r} {g} {b})");
    let (n, _) = lit(&img, seg(2, 23));
    assert!(n == 0, "redline leaves the last segment a ghost ({n} px)");
    // Limiter: the run is fully lit and steady (no cue on the run: the gear bay carries it).
    let (n, [r, g, b]) = lit(&img, seg(3, 23));
    assert!(
        n > 4 && r > g + 80.0 && b > g,
        "limiter last segment magenta ({n} px, {r} {g} {b})"
    );
}

// ---------------------------------------------------------------------------------------------
// True size: one local unit = one target pixel (the 4K panel at scale 1.0), for detail review.

fn settled_arc(tokens: &Tokens, rpm: f32) -> TachArc {
    let mut arc = TachArc::new(tokens);
    for _ in 0..FRAMES {
        arc.update(&signals(tokens, rpm), support::DT);
    }
    arc
}

fn settled_bar(bar: TachBar, tokens: &Tokens, rpm: f32) -> TachBar {
    let mut bar = bar;
    for _ in 0..FRAMES {
        bar.update(&signals(tokens, rpm), support::DT);
    }
    bar
}

fn settled_chevron(tokens: &Tokens, rpm: f32) -> TachChevron {
    let mut run = TachChevron::new(tokens);
    for _ in 0..FRAMES {
        run.update(&signals(tokens, rpm), support::DT);
    }
    run
}

#[test]
fn tach_true_size() {
    let mut gpu = support::gpu();
    let center = vec2(support::W as f32 * 0.5, support::H as f32 * 0.5);
    let layout = fh6_kit::Layout::centered(center, 1.0, (support::W, support::H));
    let modern = Palette::family(&gpu.tokens, "modernDigital");
    let race = Palette::family(&gpu.tokens, "raceDash");
    let sci_fi = Palette::family(&gpu.tokens, "sciFi");
    gpu.kit.set_palette(modern);
    let tokens = gpu.tokens.clone();

    let arc = settled_arc(&tokens, 8200.0);
    let small = [settled_arc(&tokens, 5200.0), settled_arc(&tokens, 8700.0)];
    let bars = [
        settled_bar(TachBar::new(&tokens), &tokens, 8200.0),
        settled_bar(TachBar::new(&tokens), &tokens, 8700.0),
    ];
    let runs = [settled_chevron(&tokens, 8200.0), settled_chevron(&tokens, 8700.0)];

    let img = gpu.render(&layout, |p, _| {
        let display = p.fonts().role(FontRole::Display);
        let race_font = p.fonts().role(FontRole::Race);
        let tech = p.fonts().role(FontRole::Tech);
        let ts = p.tokens().type_scale;
        let arc_center = vec2(-540.0, 40.0);
        arc.draw(p, arc_center, ARC_R, ARC_T, &modern);
        arc.draw_numerals(p, arc_center, ARC_R - ARC_T - 40.0, display, ts.dial_numeral, &modern);
        // The same part at another size, in the Sci-Fi FUI palette.
        for (arc, x) in small.iter().zip([60.0, 560.0]) {
            arc.draw(p, vec2(x, 400.0), 110.0, 12.0, &sci_fi);
        }
        for (bar, y) in bars.iter().zip([-470.0, -350.0]) {
            let rect = Rect::from_xywh(-80.0, y, BAR_W, BAR_H);
            bar.draw(p, rect, &race);
            bar.draw_scale(p, rect, race_font, ts.micro_label * 1.5, &race);
        }
        for (run, y) in runs.iter().zip([-200.0, 0.0]) {
            let rect = Rect::from_xywh(-80.0, y, 1000.0, CHEVRON_H);
            run.draw(p, rect, &sci_fi);
            run.draw_scale(p, rect, tech, ts.micro_label * 1.5, &sci_fi);
        }
    });
    let path = support::save_shot("tach_true_size", &img);
    assert!(path.exists(), "{}", path.display());
    let lit_px = img.pixels.chunks_exact(4).filter(|px| px[3] > 200).count();
    assert!(lit_px > 20_000, "the true-size shot draws ({lit_px} px)");
}
