//! `stretch`: the near-black levels made visible. Values 0 to `max` (8 of 255 by default) are
//! stretched to the full range, which shows haze, the rectangular edge of a blur or bloom target,
//! speckle and faint ink that an OLED panel shows as grey in a dark room. Alphas are linear, so a
//! few percent of ink is far brighter on the panel than its code value suggests.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::Frame;
use crate::geom::Rect;
use std::path::Path;

/// Every channel scaled so that `max` becomes 255 (values above it clip to white).
pub fn stretched(f: &Frame, max: u8) -> Frame {
    let max = u32::from(max.max(1));
    Frame {
        width: f.width,
        height: f.height,
        rgb: f
            .rgb
            .iter()
            .map(|&c| (u32::from(c) * 255 / max).min(255) as u8)
            .collect(),
        alpha: None,
        straight: false,
    }
}

/// Pixels that are off, faint (at most `max` in every channel) and lit.
pub fn levels(f: &Frame, max: u8) -> (u64, u64, u64) {
    let (mut off, mut faint, mut lit) = (0u64, 0u64, 0u64);
    for p in f.rgb.chunks_exact(3) {
        let m = p[0].max(p[1]).max(p[2]);
        if m == 0 {
            off += 1;
        } else if m <= max {
            faint += 1;
        } else {
            lit += 1;
        }
    }
    (off, faint, lit)
}

/// `stretch`: writes the stretched view and counts off, faint and lit pixels.
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--max", "--view", "--region", "--anchor", "--out"])?;
    let [input] = args.pos() else {
        return Err(usage("stretch needs one image"));
    };
    let path = Path::new(input);
    let frame = ctx.load(path)?;
    let max = args.num("--max", 8u8)?;
    if max == 0 {
        return Err(usage("--max must be 1 to 255"));
    }
    // The whole picture unless a region is asked for: haze can sit far outside the envelope.
    let region = if cli::region(args, ctx)?.is_some() {
        cli::work_region(args, ctx, &frame, path)?
    } else {
        Rect::new(0, 0, i64::from(frame.width), i64::from(frame.height))
    };
    let base = frame.crop(region).ok_or_else(|| {
        usage(format!(
            "region {region} is outside the {}x{} image",
            frame.width, frame.height
        ))
    })?;
    let (off, faint, lit) = levels(&base, max);
    let n = (off + faint + lit).max(1) as f64;
    out!(
        "{input} region {region}: {:.2}% off (#000000), {:.2}% faint (1 to {max} of 255), {:.2}% lit",
        off as f64 / n * 100.0,
        faint as f64 / n * 100.0,
        lit as f64 / n * 100.0
    );
    if (frame.width, frame.height) == ctx.tokens.canvas && cli::region(args, ctx)?.is_none() {
        // A full frame: anything not off outside the view's envelope is stray light.
        let inside = ctx.view_for(path).region(cli::VIEW_MARGIN);
        let mut outside = frame.clone();
        outside.fill(inside, [0, 0, 0]);
        let (_, f2, l2) = levels(&outside, max);
        out!(
            "outside the {} envelope plus {} px: {} px not off ({f2} faint, {l2} lit)",
            if cli::stem(path).starts_with("hood_") {
                "hood"
            } else {
                "cockpit"
            },
            cli::VIEW_MARGIN,
            f2 + l2
        );
    }
    let out = stretched(&base, max);
    let dest = cli::out_file(args, ctx, path, &format!("stretch{max}"));
    out.save(&dest)?;
    out!("stretched view (0 to {max} shown as 0 to 255): {}", dest.display());
    out!(
        "check: every non-black pixel belongs to an intended element or its glow (no rectangle edge, haze or speckle)"
    );
    if let Some(note) = cli::view_note(out.width, out.height) {
        out!("{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faint_ink_becomes_visible_and_is_counted() {
        let mut f = Frame::filled(3, 1, [0; 3]);
        f.put(1, 0, [2, 2, 2]);
        f.put(2, 0, [200, 10, 10]);
        let s = stretched(&f, 8);
        assert_eq!(s.pixel(0, 0), [0, 0, 0]);
        assert_eq!(s.pixel(1, 0), [63, 63, 63]);
        assert_eq!(s.pixel(2, 0), [255, 255, 255]);
        assert_eq!(levels(&f, 8), (1, 1, 1));
    }
}
