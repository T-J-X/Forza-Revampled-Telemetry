//! `diff`: where two renders differ, measured exactly as the golden harness measures it
//! (`fh6_kit::testing::compare`: both over black, a 2x2 box in linear light, OKLab distance x100),
//! with a heat map and the cells that changed most.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::{linear_lut, luminance, Frame};
use crate::geom::Rect;
use crate::overlay::{CYAN, MAGENTA, YELLOW};
use crate::sheet::{compose, Cell};
use fh6_kit::color::{linear_to_srgb, Oklab};
use std::path::Path;

/// The golden harness's tolerance (`fh6_kit::testing::Tolerance::default`).
const GOLDEN_DELTA: f32 = 3.0;
/// See [`GOLDEN_DELTA`].
const GOLDEN_FRACTION: f32 = 0.005;
const ORANGE: [u8; 3] = [255, 128, 0];
/// The grid the change is summarised on.
const GRID: (u32, u32) = (16, 9);

/// The OKLab colour of every pixel after the harness's 2x2 box.
pub fn lab_field(f: &Frame) -> Vec<Oklab> {
    let lut = linear_lut();
    let (w, h) = (f.width, f.height);
    let mut out = Vec::with_capacity(w as usize * h as usize);
    for y in 0..h {
        for x in 0..w {
            let mut acc = [0.0f32; 3];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                let p = f.pixel((x + dx).min(w - 1), (y + dy).min(h - 1));
                acc[0] += lut[usize::from(p[0])];
                acc[1] += lut[usize::from(p[1])];
                acc[2] += lut[usize::from(p[2])];
            }
            out.push(Oklab::from_linear_rgb(acc[0] / 4.0, acc[1] / 4.0, acc[2] / 4.0));
        }
    }
    out
}

/// OKLab distance x100, the harness's measure.
pub fn distance(p: Oklab, q: Oklab) -> f32 {
    ((p.l - q.l).powi(2) + (p.a - q.a).powi(2) + (p.b - q.b).powi(2)).sqrt() * 100.0
}

/// Per-pixel OKLab distance x100 between two frames of the same size.
pub fn delta_e(a: &Frame, b: &Frame) -> Vec<f32> {
    lab_field(a)
        .iter()
        .zip(lab_field(b))
        .map(|(p, q)| distance(*p, q))
        .collect()
}

/// What a diff found.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// Mean distance over all pixels.
    pub mean: f32,
    /// Largest distance.
    pub max: f32,
    /// Where the largest distance is.
    pub max_at: (u32, u32),
    /// Pixels above the threshold.
    pub over: u64,
    /// Their share.
    pub fraction: f32,
    /// The box round them.
    pub bbox: Option<Rect>,
    /// Grid cells with more than 1% of their pixels over, most changed first.
    pub cells: Vec<(Rect, f32)>,
}

impl Stats {
    /// Summarises `map` (`w` x `h`) at `delta`.
    pub fn of(map: &[f32], w: u32, h: u32, delta: f32) -> Self {
        let (mut sum, mut max, mut over, mut max_i) = (0.0f64, 0.0f32, 0u64, 0usize);
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
        let (cw, ch) = (w.div_ceil(GRID.0).max(1), h.div_ceil(GRID.1).max(1));
        let (gx, gy) = (w.div_ceil(cw), h.div_ceil(ch));
        let mut grid = vec![0u64; (gx * gy) as usize];
        let xy = |i: usize| {
            (
                (i as u64 % u64::from(w.max(1))) as u32,
                (i as u64 / u64::from(w.max(1))) as u32,
            )
        };
        for (i, &d) in map.iter().enumerate() {
            sum += f64::from(d);
            if d > max {
                max = d;
                max_i = i;
            }
            if d > delta {
                over += 1;
                let (x, y) = xy(i);
                (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                if let Some(c) = grid.get_mut(((y / ch) * gx + x / cw) as usize) {
                    *c += 1;
                }
            }
        }
        let n = f64::from(w) * f64::from(h);
        let mut cells: Vec<(Rect, f32)> = grid
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| {
                let (cx, cy) = (i as u32 % gx, i as u32 / gx);
                let r =
                    Rect::new(i64::from(cx * cw), i64::from(cy * ch), i64::from(cw), i64::from(ch)).clamp_to(w, h)?;
                let share = c as f32 / (r.w * r.h) as f32;
                (share > 0.01).then_some((r, share))
            })
            .collect();
        cells.sort_by(|a, b| b.1.total_cmp(&a.1));
        Self {
            mean: (sum / n.max(1.0)) as f32,
            max,
            max_at: xy(max_i),
            over,
            fraction: (over as f64 / n.max(1.0)) as f32,
            bbox: (over > 0).then(|| {
                Rect::new(
                    i64::from(x0),
                    i64::from(y0),
                    i64::from(x1 - x0 + 1),
                    i64::from(y1 - y0 + 1),
                )
            }),
            cells,
        }
    }
}

/// The heat map: `a` as dim grey, with changed pixels in steps (blue under the threshold, then
/// yellow, orange and magenta from dE 10 and 25).
pub fn heat(a: &Frame, map: &[f32], delta: f32) -> Frame {
    let lut = linear_lut();
    let mut out = Frame::filled(a.width, a.height, [0; 3]);
    for ((dst, src), &d) in out.rgb.chunks_exact_mut(3).zip(a.rgb.chunks_exact(3)).zip(map) {
        let c = if d > 25.0 {
            MAGENTA
        } else if d > 10.0 {
            ORANGE
        } else if d > delta {
            YELLOW
        } else if d > 1.0 {
            [CYAN[0] / 3, CYAN[1] / 3, CYAN[2] / 2]
        } else {
            let y = luminance([
                lut[usize::from(src[0])],
                lut[usize::from(src[1])],
                lut[usize::from(src[2])],
            ]);
            let g = (linear_to_srgb(y) * 255.0 * 0.35).round() as u8;
            [g, g, g]
        };
        dst.copy_from_slice(&c);
    }
    out
}

/// A and B mixed half and half in linear light: an onion-skin overlay to check that two
/// pictures line up.
pub fn blend(a: &Frame, b: &Frame) -> Frame {
    let lut = linear_lut();
    let rgb = a
        .rgb
        .iter()
        .zip(&b.rgb)
        .map(|(&x, &y)| {
            let v = (lut[usize::from(x)] + lut[usize::from(y)]) * 0.5;
            (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8
        })
        .collect();
    Frame {
        width: a.width,
        height: a.height,
        rgb,
        alpha: None,
        straight: false,
    }
}

/// `diff`: prints the change and writes the heat map (and a side-by-side when it fits).
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&[
        "--out",
        "--region",
        "--view",
        "--anchor",
        "--delta",
        "--max-fraction",
        "--fit",
        "--pairs",
        "--labels",
    ])?;
    let [a_in, b_in] = args.pos() else {
        return Err(usage("diff needs two images: A B"));
    };
    let delta = args.num("--delta", GOLDEN_DELTA)?;
    let max_fraction = args.num("--max-fraction", GOLDEN_FRACTION)?;
    let pairs = args.num("--pairs", 3usize)?;
    if !delta.is_finite() || delta < 0.0 {
        return Err(usage("--delta must be a number of at least 0"));
    }
    if !max_fraction.is_finite() || !(0.0..=1.0).contains(&max_fraction) {
        return Err(usage("--max-fraction must be 0 to 1"));
    }
    if pairs > 16 {
        return Err(usage("--pairs must be 0 to 16"));
    }
    let (pa, pb) = (Path::new(a_in), Path::new(b_in));
    // Labels name both pictures, with their folder when the file names are the same.
    let named = |p: &Path| {
        let parent = p
            .parent()
            .and_then(Path::file_name)
            .map(|n| n.to_string_lossy().into_owned());
        match parent {
            Some(dir) if cli::stem(pa) == cli::stem(pb) => format!("{dir}/{}", cli::stem(p)),
            _ => cli::stem(p),
        }
    };
    let given: Vec<String> = args
        .one("--labels")
        .map(|l| l.split('|').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    let defaults = [format!("A {}", named(pa)), format!("B {}", named(pb))];
    let label = |i: usize| {
        given
            .get(i)
            .filter(|s| !s.is_empty())
            .or_else(|| defaults.get(i))
            .cloned()
            .unwrap_or_default()
    };
    let a = ctx.load(pa)?;
    let mut b = ctx.load(pb)?;
    if (a.width, a.height) != (b.width, b.height) {
        if !args.has("--fit") {
            return Err(usage(format!(
                "{a_in} is {}x{} but {b_in} is {}x{}: pass --fit to scale B to A (give the render as A)",
                a.width, a.height, b.width, b.height
            )));
        }
        out!(
            "note: {b_in} scaled from {}x{} to {}x{}",
            b.width,
            b.height,
            a.width,
            a.height
        );
        b = b.resize(a.width, a.height);
    }
    let (fox, foy) = ctx.origin(&a).unwrap_or((0, 0));
    let (a, b, origin) = if cli::region(args, ctx)?.is_some() {
        let r = cli::work_region(args, ctx, &a, pa)?;
        let ca = a.crop(r).ok_or_else(|| usage("region is outside the image"))?;
        let cb = b.crop(r).ok_or_else(|| usage("region is outside the image"))?;
        (ca, cb, (r.x + fox, r.y + foy))
    } else {
        (a, b, (fox, foy))
    };
    let map = delta_e(&a, &b);
    let s = Stats::of(&map, a.width, a.height, delta);
    let shift = |r: Rect| r.offset(origin.0, origin.1);
    out!(
        "diff {a_in} vs {b_in} ({}x{} from {},{})",
        a.width,
        a.height,
        origin.0,
        origin.1
    );
    out!(
        "mean dE {:.2}, max dE {:.1}; {} px ({:.2}%) over dE {delta}",
        s.mean,
        s.max,
        s.over,
        s.fraction * 100.0
    );
    let within = s.fraction <= max_fraction;
    out!(
        "within dE {delta} on at most {:.2}% of pixels (the golden tolerance at the defaults): {}",
        max_fraction * 100.0,
        if within { "yes" } else { "no" }
    );
    match s.bbox {
        Some(r) => out!(
            "changed area: {}; largest change dE {:.1} at {},{}",
            shift(r),
            s.max,
            i64::from(s.max_at.0) + origin.0,
            i64::from(s.max_at.1) + origin.1
        ),
        None => out!("changed area: none"),
    }
    for (r, share) in s.cells.iter().take(8) {
        out!("  cell {}: {:.1}% over", shift(*r), share * 100.0);
    }
    let dir = cli::out_dir(args, ctx, pa);
    let clean = |s: String| s.replace(['/', '\\', ' '], "-");
    let name = clean(format!("{}_vs_{}", named(pa), named(pb)));
    let h = heat(&a, &map, delta);
    let mix = blend(&a, &b);
    let blend_png = dir.join(format!("{name}_blend.png"));
    mix.save(&blend_png)?;
    out!(
        "50% blend of A and B (an overlay for alignment): {}",
        blend_png.display()
    );
    if let Some(note) = cli::view_note(mix.width, mix.height) {
        out!("{note}");
    }
    let heat_png = dir.join(format!("{name}_heat.png"));
    h.save(&heat_png)?;
    out!("heat map: {}", heat_png.display());
    if let Some(note) = cli::view_note(h.width, h.height) {
        out!("{note}");
        // An overview that fits both limits of the 1:1 view.
        let (w, hh) = (f64::from(h.width), f64::from(h.height));
        let k = (f64::from(cli::VIEW_MAX_SIDE - 32) / w.max(hh))
            .min((cli::VIEW_MAX_PIXELS as f64 / (w * hh)).sqrt())
            .min(1.0);
        let small = h.resize(((w * k).floor() as u32).max(1), ((hh * k).floor() as u32).max(1));
        let p = dir.join(format!("{name}_heat_overview.png"));
        small.save(&p)?;
        out!("heat overview ({}x{}): {}", small.width, small.height, p.display());
    }
    // Crop pairs of the most-changed cells at full resolution (2x when a cell is small).
    let painter = crate::overlay::Painter::new(&ctx.root);
    for (k, (cell, share)) in s.cells.iter().take(pairs).enumerate() {
        let Some(g) = cell.grow(24).clamp_to(a.width, a.height) else {
            continue;
        };
        let (Some(ca), Some(cb), Some(ch)) = (a.crop(g), b.crop(g), h.crop(g)) else {
            continue;
        };
        let z = if g.w * 3 + 48 < 780 { 2 } else { 1 };
        let cells = [
            Cell {
                frame: ca.zoom(z),
                label: label(0),
            },
            Cell {
                frame: cb.zoom(z),
                label: label(1),
            },
            Cell {
                frame: ch.zoom(z),
                label: format!("dE over {delta}"),
            },
        ];
        // The place and the share go in the title, so every label fits its column.
        let width = (u32::try_from(g.w).unwrap_or(1) * z * 3 + 48).clamp(640, 1560);
        let title = format!("{} ({:.0}% over)", shift(g), share * 100.0);
        let (pair, _) = compose(&cells, 3, width, Some(&title), &painter)?;
        let p = dir.join(format!("{name}_pair{}.png", k + 1));
        pair.save(&p)?;
        out!("crop pair {} ({}x): {}", k + 1, z, p.display());
        if let Some(note) = cli::view_note(pair.width, pair.height) {
            out!("{note}");
        }
    }
    if a.width <= 2400 {
        let cells = [
            Cell {
                frame: a,
                label: label(0),
            },
            Cell {
                frame: b,
                label: label(1),
            },
            Cell {
                frame: h,
                label: format!("dE over {delta}"),
            },
        ];
        let (side, _) = compose(&cells, 3, 1560, None, &painter)?;
        let p = dir.join(format!("{name}_side.png"));
        side.save(&p)?;
        out!("side by side: {}", p.display());
        if let Some(note) = cli::view_note(side.width, side.height) {
            out!("{note}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_kit::testing::{compare, RgbaImage, Tolerance};

    /// A deterministic noisy image (LCG).
    fn noisy(w: u32, h: u32, seed: u32) -> Frame {
        let mut s = seed;
        let mut rgb = Vec::with_capacity((w * h * 3) as usize);
        for _ in 0..w * h * 3 {
            s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            rgb.push((s >> 24) as u8);
        }
        Frame {
            width: w,
            height: h,
            rgb,
            alpha: None,
            straight: false,
        }
    }

    fn rgba(f: &Frame) -> RgbaImage {
        RgbaImage {
            width: f.width,
            height: f.height,
            pixels: f.rgb.chunks_exact(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
        }
    }

    #[test]
    fn matches_the_golden_harness_measure() {
        let a = noisy(37, 23, 1);
        let mut b = a.clone();
        for i in (0..b.rgb.len()).step_by(7) {
            b.rgb[i] = b.rgb[i].wrapping_add(9);
        }
        let ours = Stats::of(&delta_e(&a, &b), a.width, a.height, GOLDEN_DELTA);
        let kit = compare(&rgba(&a), &rgba(&b), Tolerance::default());
        assert!((ours.mean - kit.mean).abs() < 1e-4, "{} vs {}", ours.mean, kit.mean);
        assert!((ours.max - kit.max).abs() < 1e-4);
        assert!((ours.fraction - kit.fraction_over).abs() < 1e-6);
        assert_eq!(ours.fraction <= GOLDEN_FRACTION, kit.pass);
    }

    #[test]
    fn finds_where_a_change_is() {
        let a = Frame::filled(64, 48, [0; 3]);
        let mut b = a.clone();
        b.fill(Rect::new(40, 10, 8, 6), [255, 255, 255]);
        let map = delta_e(&a, &b);
        let s = Stats::of(&map, 64, 48, GOLDEN_DELTA);
        // The 2x2 box spreads the edge by one pixel up and left.
        assert_eq!(s.bbox, Some(Rect::new(39, 9, 9, 7)));
        assert!(s.cells.first().is_some_and(|(r, _)| r.x <= 40 && r.right() >= 40));
        assert!(Stats::of(&delta_e(&a, &a), 64, 48, GOLDEN_DELTA).bbox.is_none());
        let h = heat(&a, &map, GOLDEN_DELTA);
        assert_eq!(h.pixel(44, 12), MAGENTA);
        assert_eq!(h.pixel(5, 5), [0, 0, 0]);
    }
}
