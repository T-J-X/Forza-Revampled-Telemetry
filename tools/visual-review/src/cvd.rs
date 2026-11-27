//! `cvd`: the frame as people with a colour-vision deficiency see it, and with no colour at all,
//! so a reviewer can check that every colour-coded state (shift stages, braking, proximity,
//! alarms) still reads by brightness, position or count.
//!
//! The matrices are libDaltonLens's (public domain, <https://daltonlens.org>): Viénot, Brettel
//! and Mollon 1999 for protanopia and deuteranopia, and Brettel, Viénot and Mollon 1997 for
//! tritanopia (two half-plane projections, picked by a plane whose normal libDaltonLens gives in
//! linear RGB), all applied in linear sRGB at full severity. Full severity models dichromacy,
//! the strongest case: the common anomalous trichromacies lose less colour than these views
//! show, so a state that reads here reads for them too.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::{luminance, Linear};
use crate::overlay::Painter;
use crate::sheet::{compose, Cell};
use std::path::Path;

/// Viénot 1999, protanopia: linear RGB to simulated linear RGB (row-major).
const VIENOT_PROTAN: [f32; 9] = [0.11238, 0.88762, 0.0, 0.11238, 0.88762, 0.0, 0.00401, -0.00401, 1.0];
/// Viénot 1999, deuteranopia.
const VIENOT_DEUTAN: [f32; 9] = [0.29275, 0.70725, 0.0, 0.29275, 0.70725, 0.0, -0.02234, 0.02234, 1.0];
/// Brettel 1997, tritanopia: the projection on the first half-plane.
const BRETTEL_TRITAN_1: [f32; 9] = [
    1.01277, 0.13548, -0.14826, -0.01243, 0.86812, 0.14431, 0.07589, 0.80500, 0.11911,
];
/// Brettel 1997, tritanopia: the projection on the second half-plane.
const BRETTEL_TRITAN_2: [f32; 9] = [
    0.93678, 0.18979, -0.12657, 0.06154, 0.81526, 0.12320, -0.37562, 1.12767, 0.24796,
];
/// Brettel 1997, tritanopia: the normal of the plane that picks the half-plane, in RGB.
const BRETTEL_TRITAN_NORMAL: [f32; 3] = [0.03901, -0.02788, -0.01113];

/// A colour-vision deficiency.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deficiency {
    /// No L cones (red-weak).
    Protan,
    /// No M cones (green-weak).
    Deutan,
    /// No S cones (blue-weak).
    Tritan,
}

impl Deficiency {
    /// Its label on the sheet.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Protan => "protanopia",
            Self::Deutan => "deuteranopia",
            Self::Tritan => "tritanopia",
        }
    }
}

fn mul(m: &[f32; 9], c: [f32; 3]) -> [f32; 3] {
    [
        m[0] * c[0] + m[1] * c[1] + m[2] * c[2],
        m[3] * c[0] + m[4] * c[1] + m[5] * c[2],
        m[6] * c[0] + m[7] * c[1] + m[8] * c[2],
    ]
}

/// One linear colour as seen with `d`.
pub fn simulate_px(c: [f32; 3], d: Deficiency) -> [f32; 3] {
    let out = match d {
        Deficiency::Protan => mul(&VIENOT_PROTAN, c),
        Deficiency::Deutan => mul(&VIENOT_DEUTAN, c),
        Deficiency::Tritan => {
            let n = BRETTEL_TRITAN_NORMAL;
            let side = c[0] * n[0] + c[1] * n[1] + c[2] * n[2];
            mul(
                if side >= 0.0 {
                    &BRETTEL_TRITAN_1
                } else {
                    &BRETTEL_TRITAN_2
                },
                c,
            )
        }
    };
    [out[0].clamp(0.0, 1.0), out[1].clamp(0.0, 1.0), out[2].clamp(0.0, 1.0)]
}

/// The whole image as seen with `d`.
pub fn simulate(img: &Linear, d: Deficiency) -> Linear {
    Linear {
        width: img.width,
        height: img.height,
        px: img.px.iter().map(|&c| simulate_px(c, d)).collect(),
    }
}

/// Luminance only: what is left when colour carries nothing.
pub fn greyscale(img: &Linear) -> Linear {
    Linear {
        width: img.width,
        height: img.height,
        px: img
            .px
            .iter()
            .map(|&c| {
                let y = luminance(c);
                [y, y, y]
            })
            .collect(),
    }
}

/// `cvd`: writes a five-panel sheet (normal, protanopia, deuteranopia, tritanopia, luminance)
/// and each simulation at 1:1.
///
/// # Errors
/// Usage, image and overlay errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--view", "--region", "--anchor", "--out"])?;
    let [input] = args.pos() else {
        return Err(usage("cvd needs one image"));
    };
    let path = Path::new(input);
    let frame = ctx.load(path)?;
    let region = cli::work_region(args, ctx, &frame, path)?;
    let base = frame.crop(region).ok_or_else(|| {
        usage(format!(
            "region {region} is outside the {}x{} image",
            frame.width, frame.height
        ))
    })?;
    out!("cvd of {input}, region {region} ({}x{})", base.width, base.height);
    if let Some(note) = cli::view_note(base.width, base.height) {
        out!("{note} (the single views; the sheet is smaller)");
    }
    let lin = Linear::from_frame(&base);
    let dir = cli::out_dir(args, ctx, path);
    let stem = cli::stem(path);
    let mut cells = vec![Cell {
        frame: base.clone(),
        label: "normal".to_string(),
    }];
    for d in [Deficiency::Protan, Deficiency::Deutan, Deficiency::Tritan] {
        let f = simulate(&lin, d).to_frame();
        let p = dir.join(format!("{stem}_{}.png", d.label()));
        f.save(&p)?;
        out!("{}: {}", d.label(), p.display());
        cells.push(Cell {
            frame: f,
            label: d.label().to_string(),
        });
    }
    let grey = greyscale(&lin).to_frame();
    let p = dir.join(format!("{stem}_luminance.png"));
    grey.save(&p)?;
    out!("luminance only: {}", p.display());
    cells.push(Cell {
        frame: grey,
        label: "luminance only".to_string(),
    });
    let (sheet, _) = compose(
        &cells,
        3,
        1560,
        Some(&format!("{stem}: colour vision")),
        &Painter::new(&ctx.root),
    )?;
    let p = dir.join(format!("{stem}_cvd_sheet.png"));
    sheet.save(&p)?;
    out!("sheet ({}x{}): {}", sheet.width, sheet.height, p.display());
    if let Some(note) = cli::view_note(sheet.width, sheet.height) {
        out!("{note}");
    }
    out!("check: every colour-coded state still reads by brightness, position or count in each panel");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_and_green_merge_for_protans_and_deutans_but_not_for_tritans() {
        let red = [0.8, 0.05, 0.05];
        let green = [0.05, 0.45, 0.05];
        let dist =
            |a: [f32; 3], b: [f32; 3]| ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        let normal = dist(red, green);
        for d in [Deficiency::Protan, Deficiency::Deutan] {
            let (r, g) = (simulate_px(red, d), simulate_px(green, d));
            assert!(dist(r, g) < normal * 0.5, "{d:?}: {r:?} vs {g:?}");
            // Protan and deutan simulations have no red-green axis left: R equals G.
            assert!((r[0] - r[1]).abs() < 1e-4);
        }
        let (r, g) = (
            simulate_px(red, Deficiency::Tritan),
            simulate_px(green, Deficiency::Tritan),
        );
        assert!(dist(r, g) > normal * 0.5);
    }

    #[test]
    fn neutrals_stay_neutral() {
        for d in [Deficiency::Protan, Deficiency::Deutan, Deficiency::Tritan] {
            for v in [0.0, 0.18, 1.0] {
                let s = simulate_px([v, v, v], d);
                assert!(s.iter().all(|c| (c - v).abs() < 2e-3), "{d:?} {v}: {s:?}");
            }
        }
        let g = greyscale(&Linear {
            width: 1,
            height: 1,
            px: vec![[1.0, 0.0, 0.0]],
        });
        assert!((g.px[0][1] - 0.2126).abs() < 1e-6);
    }
}
