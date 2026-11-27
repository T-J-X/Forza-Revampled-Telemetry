//! `tiles` and `crop`: pieces of a render small enough for Claude to see 1:1.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::geom::Rect;
use std::path::Path;

/// Default largest tile: inside Claude's 1:1 view (1568 px a side, 1.15 MP).
const DEFAULT_MAX: &str = "1400x800";
/// The smallest tile side, in source pixels.
const MIN_TILE: u32 = 16;

/// Splits `r` into the fewest equal tiles no larger than `max_w` x `max_h`, as
/// `(row, column, rect)`.
pub fn split(r: Rect, max_w: u32, max_h: u32) -> Vec<(u32, u32, Rect)> {
    let (_, _, w, h) = r.parts();
    let cols = w.div_ceil(max_w.max(16)).max(1);
    let rows = h.div_ceil(max_h.max(16)).max(1);
    let (tw, th) = (w.div_ceil(cols), h.div_ceil(rows));
    let mut out = Vec::with_capacity((rows * cols) as usize);
    for row in 0..rows {
        for col in 0..cols {
            let x = r.x + i64::from(col * tw);
            let y = r.y + i64::from(row * th);
            let cw = i64::from(tw).min(r.right() - x);
            let ch = i64::from(th).min(r.bottom() - y);
            if cw > 0 && ch > 0 {
                out.push((row, col, Rect::new(x, y, cw, ch)));
            }
        }
    }
    out
}

/// `tiles`: cuts a region into 1:1 tiles (optionally zoomed) and prints where each one sits.
///
/// # Errors
/// Usage and image errors.
pub fn tiles(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--view", "--region", "--anchor", "--zoom", "--max", "--out"])?;
    let [input] = args.pos() else {
        return Err(usage("tiles needs one image"));
    };
    let path = Path::new(input);
    let zoom = cli::zoom(args)?;
    let (max_w, max_h) = cli::size(args.one("--max").unwrap_or(DEFAULT_MAX), "--max")?;
    if max_w / zoom < MIN_TILE || max_h / zoom < MIN_TILE {
        return Err(usage(format!(
            "--max {max_w}x{max_h} at --zoom {zoom} leaves tiles under {MIN_TILE} px of source: give at least {}x{}",
            MIN_TILE * zoom,
            MIN_TILE * zoom
        )));
    }
    let frame = ctx.load(path)?;
    let region = cli::work_region(args, ctx, &frame, path)?;
    let pieces = split(region, max_w / zoom, max_h / zoom);
    let dir = cli::out_dir(args, ctx, path);
    let stem = cli::stem(path);
    let suffix = if zoom > 1 { format!("_x{zoom}") } else { String::new() };
    out!(
        "tiles of {input} ({}x{}): region {region}{}, {} tile(s) at {zoom}x",
        frame.width,
        frame.height,
        cli::in_4k(ctx, &frame, region),
        pieces.len()
    );
    for (row, col, rect) in &pieces {
        let Some(tile) = frame.crop(*rect) else {
            continue;
        };
        let tile = tile.zoom(zoom);
        let dest = dir.join(format!("{stem}_tile_r{row}c{col}{suffix}.png"));
        tile.save(&dest)?;
        out!(
            "  {} = source {rect}{} ({}x{})",
            dest.display(),
            cli::in_4k(ctx, &frame, *rect),
            tile.width,
            tile.height
        );
        if let Some(note) = cli::view_note(tile.width, tile.height) {
            out!("  {note} (lower --max)");
        }
    }
    Ok(())
}

/// `crop`: one region, optionally zoomed.
///
/// # Errors
/// Usage and image errors.
pub fn crop(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&[
        "--view", "--region", "--anchor", "--zoom", "--mirror", "--mask", "--out",
    ])?;
    let [input] = args.pos() else {
        return Err(usage("crop needs one image"));
    };
    let path = Path::new(input);
    if cli::region(args, ctx)?.is_none() {
        return Err(usage("crop needs --region X,Y,W,H or --view cockpit|hood|hud"));
    }
    let zoom = cli::zoom(args)?;
    let mut frame = ctx.load(path)?;
    let origin = ctx.origin(&frame).unwrap_or((0, 0));
    for m in cli::masks(args, ctx, origin)? {
        frame.fill(m, [0, 0, 0]);
    }
    let region = cli::work_region(args, ctx, &frame, path)?;
    cli::check_size(region.w, region.h, zoom)?;
    let mut out = frame
        .crop(region)
        .ok_or_else(|| usage(format!("region {region} is outside the image")))?
        .zoom(zoom);
    let mut zs = if zoom > 1 { format!("_x{zoom}") } else { String::new() };
    if args.has("--mirror") {
        out = out.mirrored();
        zs.push_str("_mirrored");
    }
    let dest = cli::out_file(
        args,
        ctx,
        path,
        &format!("crop_{}_{}_{}x{}{zs}", region.x, region.y, region.w, region.h),
    );
    out.save(&dest)?;
    out!(
        "wrote {} = source {region}{} at {zoom}x ({}x{})",
        dest.display(),
        cli::in_4k(ctx, &frame, region),
        out.width,
        out.height
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
    fn splits_into_the_fewest_tiles_that_fit() {
        // The cockpit envelope plus 40 px (1520x640) fits one 1400x800 tile? No: 1520 > 1400.
        let t = split(Rect::new(1160, 1475, 1520, 640), 1400, 800);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].2, Rect::new(1160, 1475, 760, 640));
        assert_eq!(t[1].2, Rect::new(1920, 1475, 760, 640));
        // Tiles cover the region exactly.
        let area: i64 = t.iter().map(|(_, _, r)| r.w * r.h).sum();
        assert_eq!(area, 1520 * 640);
        // At 3x zoom each tile covers at most 466x266 source px.
        let z = split(Rect::new(0, 0, 1520, 640), 1400 / 3, 800 / 3);
        assert!(z.iter().all(|(_, _, r)| r.w <= 466 && r.h <= 266));
        let area: i64 = z.iter().map(|(_, _, r)| r.w * r.h).sum();
        assert_eq!(area, 1520 * 640);
    }

    #[test]
    fn a_small_region_is_one_tile() {
        let t = split(Rect::new(5, 6, 100, 50), 1400, 800);
        assert_eq!(t, vec![(0, 0, Rect::new(5, 6, 100, 50))]);
    }
}
