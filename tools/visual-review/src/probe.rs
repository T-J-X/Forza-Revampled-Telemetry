//! `probe`: numbers for one region of an image (true black, luminance spread, the median lit
//! colour and the dominant hues).
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::{linear_lut, luminance, Frame};
use crate::geom::Rect;
use fh6_kit::color::{linear_to_srgb, Oklab, Oklch};
use std::path::Path;

/// Chroma above which a pixel counts as coloured.
const CHROMA_MIN: f32 = 0.04;
/// A pixel is lit above this relative luminance.
const LIT: f32 = 0.002;

/// The numbers `probe` prints.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    /// Pixels measured.
    pub px: u64,
    /// Exactly `#000000` (OLED off).
    pub black: u64,
    /// Covered pixels (alpha above 0), when the source had alpha.
    pub covered: Option<u64>,
    /// Relative luminance at the 5th, 50th and 95th percentiles, and the maximum.
    pub luma: [f32; 4],
    /// Lit pixels (relative luminance above 0.002).
    pub lit: u64,
    /// The median OKLab of the lit pixels (per axis), as OKLCH; `None` when nothing is lit.
    pub median_lit: Option<Oklch>,
    /// Hue bins of 30 degrees and their share of the coloured pixels, largest first.
    pub hues: Vec<(u32, f32)>,
}

/// The median of `v` (sorted in place).
fn median(v: &mut [f32]) -> Option<f32> {
    v.sort_by(f32::total_cmp);
    v.get(v.len() / 2).copied()
}

/// Measures `r` of `f`.
pub fn probe(f: &Frame, r: Rect) -> Option<Probe> {
    let r = r.clamp_to(f.width, f.height)?;
    let (x, y, w, h) = r.parts();
    let lut = linear_lut();
    let mut ys = Vec::with_capacity(w as usize * h as usize);
    let (mut black, mut covered, mut coloured) = (0u64, 0u64, 0u64);
    let mut bins = [0u64; 12];
    let (mut ls, mut as_, mut bs) = (Vec::new(), Vec::new(), Vec::new());
    for yy in y..y + h {
        for xx in x..x + w {
            let p = f.pixel(xx, yy);
            if p == [0, 0, 0] {
                black += 1;
            }
            if let Some(a) = &f.alpha {
                if a.get(yy as usize * f.width as usize + xx as usize)
                    .is_some_and(|v| *v > 0)
                {
                    covered += 1;
                }
            }
            let c = [lut[usize::from(p[0])], lut[usize::from(p[1])], lut[usize::from(p[2])]];
            let luma = luminance(c);
            ys.push(luma);
            let lab = Oklab::from_linear_rgb(c[0], c[1], c[2]);
            if luma > LIT {
                ls.push(lab.l);
                as_.push(lab.a);
                bs.push(lab.b);
            }
            let lch = lab.to_oklch();
            if lch.c > CHROMA_MIN {
                coloured += 1;
                if let Some(b) = bins.get_mut((lch.h.rem_euclid(360.0) / 30.0) as usize % 12) {
                    *b += 1;
                }
            }
        }
    }
    ys.sort_by(f32::total_cmp);
    let at = |q: f32| {
        ys.get(((ys.len().saturating_sub(1)) as f32 * q).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    let mut hues: Vec<(u32, f32)> = bins
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| (i as u32 * 30, *n as f32 / coloured.max(1) as f32))
        .collect();
    hues.sort_by(|a, b| b.1.total_cmp(&a.1));
    let lit = ls.len() as u64;
    let median_lit = match (median(&mut ls), median(&mut as_), median(&mut bs)) {
        (Some(l), Some(a), Some(b)) => Some(Oklab { l, a, b }.to_oklch()),
        _ => None,
    };
    Some(Probe {
        px: u64::from(w) * u64::from(h),
        black,
        covered: f.alpha.as_ref().map(|_| covered),
        luma: [at(0.05), at(0.5), at(0.95), ys.last().copied().unwrap_or(0.0)],
        lit,
        median_lit,
        hues,
    })
}

/// `#RRGGBB` of an OKLCH colour, clipped into sRGB.
fn hex(c: Oklch) -> String {
    let (r, g, b) = c.to_linear_rgb_clipped();
    let byte = |v: f32| (linear_to_srgb(v.clamp(0.0, 1.0)) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", byte(r), byte(g), byte(b))
}

/// `probe`: prints the region's numbers.
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--region", "--view", "--anchor"])?;
    let [input] = args.pos() else {
        return Err(usage("probe needs one image"));
    };
    let path = Path::new(input);
    let f = ctx.load(path)?;
    let r = cli::work_region(args, ctx, &f, path)?;
    let p = probe(&f, r).ok_or_else(|| usage(format!("region {r} is outside the {}x{} image", f.width, f.height)))?;
    let pct = |n: u64| n as f64 / p.px.max(1) as f64 * 100.0;
    out!(
        "probe {input} region {r}{}: {} px{}",
        cli::in_4k(ctx, &f, r),
        p.px,
        if f.straight {
            " (straight alpha, multiplied over black)"
        } else {
            ""
        }
    );
    out!("  exactly #000000 (off): {:.2}%", pct(p.black));
    if let Some(c) = p.covered {
        out!("  covered (alpha above 0): {:.2}%", pct(c));
    }
    out!(
        "  relative luminance p5 {:.4}, median {:.4}, p95 {:.4}, max {:.4}",
        p.luma[0],
        p.luma[1],
        p.luma[2],
        p.luma[3]
    );
    match p.median_lit {
        Some(c) => out!(
            "  lit pixels {:.2}%; their median colour OKLCH L {:.3} C {:.3} h {:.0} ({})",
            pct(p.lit),
            c.l,
            c.c,
            c.h.rem_euclid(360.0),
            hex(c)
        ),
        None => out!("  lit pixels: none"),
    }
    let hues: Vec<String> = p
        .hues
        .iter()
        .take(3)
        .map(|(h, s)| format!("{h}-{} deg {:.0}%", h + 30, s * 100.0))
        .collect();
    out!(
        "  coloured-pixel hues (OKLCH): {}",
        if hues.is_empty() {
            "none".to_string()
        } else {
            hues.join(", ")
        }
    );
    if (f.width, f.height) == ctx.tokens.canvas {
        let corners = [(0, 0), (f.width - 1, 0), (0, f.height - 1), (f.width - 1, f.height - 1)];
        let off = corners.iter().all(|(x, y)| f.pixel(*x, *y) == [0, 0, 0]);
        out!("  the frame's corners are off: {}", if off { "yes" } else { "no" });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_black_luminance_and_hue() {
        let mut f = Frame::filled(10, 10, [0; 3]);
        f.fill(Rect::new(0, 0, 10, 2), [255, 0, 0]);
        let p = probe(&f, Rect::new(0, 0, 10, 10)).expect("inside");
        assert_eq!(p.px, 100);
        assert_eq!(p.black, 80);
        assert_eq!(p.lit, 20);
        assert!((p.luma[3] - 0.2126).abs() < 1e-3);
        assert_eq!(p.hues.first().map(|h| h.0), Some(0));
        // The median lit colour is the red itself.
        let m = p.median_lit.expect("lit");
        assert_eq!(hex(m), "#FF0000");
        assert!(probe(&f, Rect::new(50, 50, 5, 5)).is_none());
        let dark = probe(&Frame::filled(4, 4, [0; 3]), Rect::new(0, 0, 4, 4)).expect("inside");
        assert!(dark.median_lit.is_none());
    }
}
