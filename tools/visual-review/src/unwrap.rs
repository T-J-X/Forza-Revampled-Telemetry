//! `unwrap`: a dial ring unrolled into a straight strip (angle across, radius down), so ticks,
//! numerals and LEDs can be counted and the needle's angle read from pixel columns instead of
//! guessed from a curve. Angles are screen degrees: 0 at 3 o'clock, increasing clockwise (the
//! tokens' `layout.classicDial` convention). Positions are continuous pixel coordinates: pixel
//! `(i, j)` covers `[i, i + 1) x [j, j + 1)`, so its centre is `(i + 0.5, j + 0.5)`.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::Frame;
use crate::geom::Rect;
use crate::overlay::{Map, Mark, Painter, WHITE};
use std::path::Path;

/// The strip's label band.
const BAND: u32 = 40;
/// The most columns per degree.
const MAX_PPD: f64 = 40.0;
/// The largest outer radius and ring thickness, in pixels.
const MAX_RADIUS: f64 = 4000.0;
/// See [`MAX_RADIUS`].
const MAX_THICKNESS: f64 = 2160.0;
/// The largest strip, in pixels.
const MAX_STRIP_PIXELS: f64 = 16_000_000.0;

/// Bilinear sample of `f` at the continuous position `(x, y)` (black outside).
fn sample(f: &Frame, x: f64, y: f64) -> [u8; 3] {
    // Pixel centres sit at half-pixel positions.
    let (x, y) = (x - 0.5, y - 0.5);
    let (x0, y0) = (x.floor(), y.floor());
    let (tx, ty) = (x - x0, y - y0);
    let px = |dx: f64, dy: f64| -> [f64; 3] {
        let (xx, yy) = (x0 + dx, y0 + dy);
        if xx < 0.0 || yy < 0.0 || xx >= f64::from(f.width) || yy >= f64::from(f.height) {
            return [0.0; 3];
        }
        let p = f.pixel(xx as u32, yy as u32);
        [f64::from(p[0]), f64::from(p[1]), f64::from(p[2])]
    };
    let (a, b, c, d) = (px(0.0, 0.0), px(1.0, 0.0), px(0.0, 1.0), px(1.0, 1.0));
    let mut out = [0u8; 3];
    for (k, o) in out.iter_mut().enumerate() {
        let top = a[k] + (b[k] - a[k]) * tx;
        let bottom = c[k] + (d[k] - c[k]) * tx;
        *o = (top + (bottom - top) * ty).round().clamp(0.0, 255.0) as u8;
    }
    out
}

/// The ring between radii `r0` and `r1` round `(cx, cy)`, from `from` to `to` degrees, at `ppd`
/// columns per degree. Column `x` is the angle `from + x / ppd`; row `y` is the radius `r1 - y`
/// (the top row is the outer radius).
pub fn unrolled(f: &Frame, (cx, cy): (f64, f64), (r0, r1): (f64, f64), (from, to): (f64, f64), ppd: f64) -> Frame {
    let w = ((to - from) * ppd).round().max(1.0) as u32;
    let h = (r1 - r0).round().max(1.0) as u32;
    let mut out = Frame::filled(w, h, [0; 3]);
    for x in 0..w {
        let ang = (from + f64::from(x) / ppd).to_radians();
        let (s, c) = ang.sin_cos();
        for y in 0..h {
            let r = r1 - f64::from(y);
            out.put(x, y, sample(f, cx + r * c, cy + r * s));
        }
    }
    out
}

/// `unwrap`: writes the strip with an angle scale under it.
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--center", "--radius", "--from", "--to", "--ppd", "--anchor", "--out"])?;
    let [input] = args.pos() else {
        return Err(usage("unwrap needs one image"));
    };
    let path = Path::new(input);
    let c = cli::numbers(
        args.one("--center")
            .ok_or_else(|| usage("unwrap needs --center CX,CY"))?,
        2,
        "--center CX,CY",
    )?;
    let r = cli::numbers(
        args.one("--radius")
            .ok_or_else(|| usage("unwrap needs --radius INNER,OUTER"))?,
        2,
        "--radius INNER,OUTER",
    )?;
    let (r0, r1) = (r[0].min(r[1]).max(0.0), r[0].max(r[1]));
    if r1 - r0 < 2.0 {
        return Err(usage("--radius: the ring must be at least 2 px thick"));
    }
    if r1 > MAX_RADIUS || r1 - r0 > MAX_THICKNESS {
        return Err(usage(format!(
            "--radius: the outer radius may be at most {MAX_RADIUS:.0} px and the ring at most {MAX_THICKNESS:.0} px thick"
        )));
    }
    let from = args.num("--from", 0.0f64)?;
    let to = args.num("--to", from + 360.0)?;
    if !from.is_finite() || !to.is_finite() || !(to > from && to - from <= 720.0) || from.abs() > 7200.0 {
        return Err(usage("--to must be above --from, by at most 720 degrees"));
    }
    // About one column per pixel of arc along the outer edge, 2 to 40 per degree.
    let ppd = args.num("--ppd", (r1 * std::f64::consts::PI / 180.0).clamp(2.0, MAX_PPD))?;
    if !(0.5..=MAX_PPD).contains(&ppd) {
        return Err(usage("--ppd must be 0.5 to 40 columns per degree"));
    }
    if (to - from) * ppd * (r1 - r0) > MAX_STRIP_PIXELS {
        return Err(usage(
            "the strip would be over 16 megapixels: narrow --from/--to, lower --ppd or thin the ring",
        ));
    }
    let frame = ctx.load(path)?;
    let (ax, ay) = cli::anchor(args, ctx)?;
    let (ox, oy) = match ctx.origin(&frame) {
        Some(o) => o,
        None if args.one("--anchor").is_some() => {
            return Err(usage(format!(
                "{input} is {}x{}: --anchor needs a full 3840x2160 frame or a preview-export crop; give --center in the picture's own pixels",
                frame.width, frame.height
            )))
        }
        None => (0, 0),
    };
    // The centre in 4K coordinates, and in the picture's own pixels.
    let centre = (c[0] + ax as f64, c[1] + ay as f64);
    let local = (centre.0 - ox as f64, centre.1 - oy as f64);
    let strip = unrolled(&frame, local, (r0, r1), (from, to), ppd);
    let mut out = Frame::filled(strip.width, strip.height + BAND, [12, 14, 18]);
    out.blit(&strip, 0, 0);
    let mut marks = Vec::new();
    // Ticks every 10 degrees, counted in whole steps so no rounding creeps in.
    let (first, last) = ((from / 10.0).ceil() as i64, (to / 10.0).floor() as i64);
    for step in first..=last {
        let deg = (step * 10) as f64;
        let x = ((deg - from) * ppd).round();
        let major = step.rem_euclid(3) == 0;
        out.fill(
            Rect::new(x as i64, i64::from(strip.height), 1, if major { 14 } else { 7 }),
            WHITE,
        );
        if major {
            marks.push(Mark::Text {
                x: x + 3.0,
                y: f64::from(strip.height) + 32.0,
                size: 16.0,
                text: format!("{}", (step * 10).rem_euclid(360)),
                color: WHITE,
            });
        }
    }
    Painter::new(&ctx.root).draw(&mut out, &marks, Map::IDENTITY)?;
    let dest = cli::out_file(args, ctx, path, &format!("unwrap_{:.0}_{:.0}", centre.0, centre.1));
    out.save(&dest)?;
    out!(
        "wrote {} ({}x{}): ring {r0:.0}-{r1:.0} px round {:.1},{:.1}, {from:.0} to {to:.0} degrees at {ppd:.2} columns per degree; top row = outer edge",
        dest.display(),
        out.width,
        out.height,
        centre.0,
        centre.1
    );
    out!(
        "column x is the angle {from:.0} + x / {ppd:.2} degrees (0 = 3 o'clock, clockwise); row y is the radius {r1:.0} - y"
    );
    if let Some(note) = cli::view_note(out.width, out.height) {
        out!("{note} (narrow it with --from and --to)");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_radial_line_becomes_one_column() {
        // A white spoke one pixel wide straight down from the centre (90 degrees in screen space).
        // Pixel column 100 is centred at x = 100.5.
        let mut f = Frame::filled(201, 201, [0; 3]);
        for y in 130..190 {
            f.put(100, y, [255, 255, 255]);
        }
        let strip = unrolled(&f, (100.5, 100.5), (40.0, 80.0), (0.0, 180.0), 2.0);
        assert_eq!((strip.width, strip.height), (360, 40));
        // Column for 90 degrees: x = 180. Its middle rows are white; 0 degrees is black.
        assert_eq!(strip.pixel(180, 20), [255, 255, 255]);
        assert_eq!(strip.pixel(10, 20), [0, 0, 0]);
        // Half a pixel off the spoke's centre reads half its light.
        assert!((120..=135).contains(&sample(&f, 101.0, 150.5)[0]));
    }
}
