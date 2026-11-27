//! `sheet`: images side by side under their labels (render against board, reference, before and
//! after, the four states, sequence frames).
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Result};
use crate::frame::{Frame, Linear};
use crate::geom::Rect;
use crate::overlay::{Map, Mark, Painter, WHITE};
use std::path::Path;

const GAP: u32 = 12;
const LABEL_H: u32 = 34;
const LABEL_PX: f64 = 22.0;
const TITLE_H: u32 = 46;
const TITLE_PX: f64 = 28.0;
const BACKGROUND: [u8; 3] = [12, 14, 18];
const BORDER: [u8; 3] = [52, 58, 66];
/// Default sheet width: inside Claude's 1:1 view.
const DEFAULT_WIDTH: u32 = 1560;
/// The most a picture is enlarged (in whole steps, so its pixels stay square).
const MAX_ENLARGE: f64 = 8.0;
/// The largest sheet the tool writes, in pixels.
const MAX_SHEET_PIXELS: u64 = 16_000_000;
/// The `--blur` range, in sheet pixels.
const BLUR_RANGE: (f32, f32) = (0.1, 40.0);

/// One picture on a sheet.
#[derive(Debug, Clone)]
pub struct Cell {
    /// The picture.
    pub frame: Frame,
    /// Its label.
    pub label: String,
}

/// The scale that fits a `w` x `h` picture into a cell `cell_w` wide and at most `max_h` tall:
/// shrunk by any amount, enlarged only in whole steps up to 8x.
pub fn fit_scale(w: u32, h: u32, cell_w: u32, max_h: u32) -> f64 {
    let k = (f64::from(cell_w) / f64::from(w.max(1))).min(f64::from(max_h) / f64::from(h.max(1)));
    if k >= 1.0 {
        k.floor().min(MAX_ENLARGE)
    } else {
        k
    }
}

/// `frame` drawn at `k`: whole-step enlargements with nearest pixels, reductions in linear light.
fn scaled(frame: &Frame, k: f64) -> Frame {
    if k >= 1.0 {
        return frame.zoom(k.round() as u32);
    }
    let w = (f64::from(frame.width) * k).round().max(1.0) as u32;
    let h = (f64::from(frame.height) * k).round().max(1.0) as u32;
    frame.resize(w, h)
}

/// Lays `cells` out in `cols` columns across `width` px. Returns the sheet and the scale each
/// cell was drawn at.
///
/// # Errors
/// A usage error when the sheet would be larger than 16 megapixels; overlay errors from the
/// labels.
pub fn compose(
    cells: &[Cell],
    cols: u32,
    width: u32,
    title: Option<&str>,
    painter: &Painter,
) -> Result<(Frame, Vec<f64>)> {
    layout(cells, cols, width, title, None, painter)
}

/// [`compose`], with every picture blurred by `blur` sheet pixels after scaling (a squint view).
///
/// # Errors
/// See [`compose`].
pub fn layout(
    cells: &[Cell],
    cols: u32,
    width: u32,
    title: Option<&str>,
    blur: Option<f32>,
    painter: &Painter,
) -> Result<(Frame, Vec<f64>)> {
    let cols = cols.clamp(1, u32::try_from(cells.len().max(1)).unwrap_or(1));
    let cell_w = (width.saturating_sub(GAP * (cols + 1)) / cols).max(16);
    let max_h = (cell_w * 2).min(cli::VIEW_MAX_SIDE);
    let drawn: Vec<(Frame, f64)> = cells
        .iter()
        .map(|c| {
            let k = fit_scale(c.frame.width, c.frame.height, cell_w, max_h);
            let mut f = scaled(&c.frame, k);
            if let Some(s) = blur {
                f = Linear::from_frame(&f).blur(s).to_frame();
            }
            (f, k)
        })
        .collect();
    let title_h = if title.is_some() { TITLE_H } else { 0 };
    let rows: Vec<&[(Frame, f64)]> = drawn.chunks(cols as usize).collect();
    let height = GAP
        + title_h
        + rows
            .iter()
            .map(|r| LABEL_H + r.iter().map(|(f, _)| f.height).max().unwrap_or(0) + GAP)
            .sum::<u32>();
    let pixels = u64::from(width) * u64::from(height);
    if pixels > MAX_SHEET_PIXELS {
        return Err(usage(format!(
            "the sheet would be {width}x{height} ({:.1} MP, more than 16): give fewer pictures, more --cols or a smaller --width",
            pixels as f64 / 1e6
        )));
    }
    let mut sheet = Frame::filled(width, height, BACKGROUND);
    let mut marks = Vec::new();
    if let Some(t) = title {
        marks.push(Mark::Text {
            x: f64::from(GAP),
            y: f64::from(GAP) + TITLE_PX,
            size: TITLE_PX,
            text: t.to_string(),
            color: WHITE,
        });
    }
    let mut y = GAP + title_h;
    let mut i = 0;
    for row in &rows {
        let row_h = row.iter().map(|(f, _)| f.height).max().unwrap_or(0);
        for (col, (f, k)) in row.iter().enumerate() {
            let x = GAP + (col as u32) * (cell_w + GAP);
            let left = x + cell_w.saturating_sub(f.width) / 2;
            let top = y + LABEL_H;
            sheet.fill(
                Rect::new(
                    i64::from(left) - 1,
                    i64::from(top) - 1,
                    i64::from(f.width) + 2,
                    i64::from(f.height) + 2,
                ),
                BORDER,
            );
            sheet.blit(f, left, top);
            let label = cells.get(i).map(|c| c.label.as_str()).unwrap_or("");
            let scale = if (*k - 1.0).abs() < 1e-6 {
                String::new()
            } else {
                format!("  ({k:.2}x)")
            };
            marks.push(Mark::Text {
                x: f64::from(x),
                y: f64::from(y) + LABEL_PX + 2.0,
                size: LABEL_PX,
                text: fit_label(label, &scale, cell_w),
                color: WHITE,
            });
            i += 1;
        }
        y += LABEL_H + row_h + GAP;
    }
    painter.draw(&mut sheet, &marks, Map::IDENTITY)?;
    Ok((sheet, drawn.into_iter().map(|(_, k)| k).collect()))
}

/// `label` and `scale` shortened to fit a column `cell_w` px wide: the scale is kept and the
/// label's end is cut, marked with an ellipsis.
fn fit_label(label: &str, scale: &str, cell_w: u32) -> String {
    let max = (f64::from(cell_w) / (LABEL_PX * 0.52)).floor().max(4.0) as usize;
    let (n, s) = (label.chars().count(), scale.chars().count());
    if n + s <= max {
        return format!("{label}{scale}");
    }
    let keep = max.saturating_sub(s + 1);
    let cut: String = label.chars().take(keep).collect();
    format!("{cut}\u{2026}{scale}")
}

/// What follows the last `@` of a picture argument.
#[derive(Debug, Clone, PartialEq)]
enum Suffix {
    /// No region: the path is the whole argument.
    None,
    /// `@X,Y,W,H`.
    Rect(Rect),
    /// `@cockpit|hood|hud`.
    View(Rect),
}

/// Splits `path@X,Y,W,H` or `path@cockpit|hood|hud`. A path that has an `@` of its own is kept
/// whole when that file exists.
///
/// # Errors
/// A usage error when the suffix is neither a rectangle nor a view and the whole argument is
/// not a file.
fn split_spec<'a>(spec: &'a str, ctx: &Ctx) -> Result<(&'a str, Suffix)> {
    let Some((path, suffix)) = spec.rsplit_once('@') else {
        return Ok((spec, Suffix::None));
    };
    if let Ok(r) = Rect::parse(suffix) {
        return Ok((path, Suffix::Rect(r)));
    }
    if let Ok(v) = ctx.tokens.view(suffix) {
        return Ok((path, Suffix::View(v.region(cli::VIEW_MARGIN))));
    }
    if Path::new(spec).is_file() {
        return Ok((spec, Suffix::None));
    }
    Err(usage(format!(
        "{spec}: @{suffix} is neither X,Y,W,H nor cockpit, hood or hud"
    )))
}

/// The part of `frame` a picture argument asks for, in the frame's own pixels.
///
/// # Errors
/// A usage error when the region misses the picture, or a view is asked of a picture that is
/// neither a full frame nor a known crop.
fn pick(args: &Args, ctx: &Ctx, spec: &str, frame: &Frame, own: &Suffix, global: Option<Rect>) -> Result<Option<Rect>> {
    let origin = ctx.origin(frame);
    let anchor = cli::anchor(args, ctx)?;
    let r = match (own, origin) {
        (Suffix::Rect(r), Some((ox, oy))) => Some(r.offset(anchor.0 - ox, anchor.1 - oy)),
        // Any other picture: the rectangle is in its own pixels.
        (Suffix::Rect(r), None) => Some(*r),
        (Suffix::View(r), Some((ox, oy))) => Some(r.offset(-ox, -oy)),
        (Suffix::View(_), None) => {
            return Err(usage(format!(
                "{spec}: a view needs a full 3840x2160 frame or a preview-export crop, and this picture is {}x{}",
                frame.width, frame.height
            )))
        }
        (Suffix::None, Some((ox, oy))) => global.map(|r| r.offset(-ox, -oy)),
        (Suffix::None, None) => {
            if global.is_some() {
                out!(
                    "note: {spec} is {}x{}, neither a full frame nor a crop, so --region/--view is not applied to it",
                    frame.width,
                    frame.height
                );
            }
            None
        }
    };
    match r {
        None => Ok(None),
        Some(r) => r.clamp_to(frame.width, frame.height).map(Some).ok_or_else(|| {
            usage(format!(
                "{spec}: region {r} is outside the {}x{} picture",
                frame.width, frame.height
            ))
        }),
    }
}

/// `sheet`: composes the images and prints each one's scale.
///
/// # Errors
/// Usage, image and overlay errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&[
        "--out", "--labels", "--cols", "--width", "--title", "--region", "--view", "--anchor", "--blur",
    ])?;
    let Some(first) = args.pos().first() else {
        return Err(usage("sheet needs at least one image"));
    };
    let blur = match args.one("--blur") {
        None => None,
        Some(_) => {
            let s = args.num("--blur", 0.0f32)?;
            if !(BLUR_RANGE.0..=BLUR_RANGE.1).contains(&s) {
                return Err(usage(format!(
                    "--blur must be {} to {} sheet pixels",
                    BLUR_RANGE.0, BLUR_RANGE.1
                )));
            }
            Some(s)
        }
    };
    let labels: Vec<&str> = args.one("--labels").map(|l| l.split('|').collect()).unwrap_or_default();
    if labels.len() > args.pos().len() {
        warn!(
            "warning: {} labels for {} picture(s); the extra ones are not used",
            labels.len(),
            args.pos().len()
        );
    }
    let global = cli::region(args, ctx)?;
    // With --anchor, the shared --region is already moved by `cli::region`.
    let mut cells = Vec::with_capacity(args.pos().len());
    for (i, spec) in args.pos().iter().enumerate() {
        let (path, own) = split_spec(spec, ctx)?;
        let frame = ctx.load(Path::new(path))?;
        let frame = match pick(args, ctx, spec, &frame, &own, global)? {
            Some(r) => frame
                .crop(r)
                .ok_or_else(|| usage(format!("{spec}: region {r} is outside the picture")))?,
            None => frame,
        };
        let label = labels
            .get(i)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| cli::stem(Path::new(path)));
        cells.push(Cell { frame, label });
    }
    let n = u32::try_from(cells.len()).unwrap_or(u32::MAX);
    let cols = args
        .num("--cols", if n == 4 { 2 } else { n.min(3) })?
        .clamp(1, n.max(1));
    let painter = Painter::new(&ctx.root);
    let title = args.one("--title");
    let (sheet, scales) = if args.one("--width").is_some() {
        let width = args.num("--width", DEFAULT_WIDTH)?.clamp(320, 7680);
        layout(&cells, cols, width, title, blur, &painter)?
    } else {
        // The default keeps the sheet inside Claude's 1:1 view: narrower until it fits.
        let mut width = DEFAULT_WIDTH;
        let mut made = layout(&cells, cols, width, title, blur, &painter)?;
        for _ in 0..4 {
            let px = u64::from(made.0.width) * u64::from(made.0.height);
            if px <= cli::VIEW_MAX_PIXELS && made.0.height <= cli::VIEW_MAX_SIDE {
                break;
            }
            let k = (cli::VIEW_MAX_PIXELS as f64 / px as f64)
                .sqrt()
                .min(f64::from(cli::VIEW_MAX_SIDE) / f64::from(made.0.height));
            width = ((f64::from(width) * k * 0.98) as u32).max(320);
            made = layout(&cells, cols, width, title, blur, &painter)?;
        }
        made
    };
    let (first_path, _) = split_spec(first, ctx)?;
    let dest = cli::out_file(args, ctx, Path::new(first_path), "sheet");
    sheet.save(&dest)?;
    out!(
        "wrote {} ({}x{}, {} picture(s) in {cols} column(s){})",
        dest.display(),
        sheet.width,
        sheet.height,
        cells.len(),
        blur.map(|s| format!(", blurred {s} px")).unwrap_or_default()
    );
    for (c, k) in cells.iter().zip(&scales) {
        out!("  {}: {}x{} drawn at {k:.2}x", c.label, c.frame.width, c.frame.height);
    }
    if let Some(note) = cli::view_note(sheet.width, sheet.height) {
        out!("{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painter() -> Painter {
        Painter::new(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
    }

    #[test]
    fn lays_cells_out_in_rows_under_labels() {
        let cells: Vec<Cell> = (0..4)
            .map(|i| Cell {
                frame: Frame::filled(300, 100, [40 * i, 0, 0]),
                label: format!("state {i}"),
            })
            .collect();
        let (sheet, scales) = compose(&cells, 2, 636, Some("States"), &painter()).expect("composes");
        // Two columns of (636 - 3 * 12) / 2 = 300 px: drawn 1:1.
        assert!(scales.iter().all(|k| (k - 1.0).abs() < 1e-9));
        assert_eq!(sheet.width, 636);
        assert_eq!(sheet.height, GAP + TITLE_H + 2 * (LABEL_H + 100 + GAP));
        // The second cell of the first row starts at x = 12 + 300 + 12.
        let top = GAP + TITLE_H + LABEL_H;
        assert_eq!(sheet.pixel(GAP + 300 + GAP + 5, top + 50), [40, 0, 0]);
        assert_eq!(sheet.pixel(GAP + 5, top + LABEL_H + 100 + GAP + 50), [80, 0, 0]);
    }

    #[test]
    fn pictures_fit_their_cell_and_enlarge_in_whole_steps() {
        // A tall picture is held to twice the cell width.
        assert!((fit_scale(100, 1000, 300, 600) - 0.6).abs() < 1e-9);
        // A small one is enlarged in whole steps: 6x, not 6.25x.
        assert!((fit_scale(48, 20, 300, 600) - 6.0).abs() < 1e-9);
        // Never more than 8x.
        assert!((fit_scale(10, 10, 300, 600) - 8.0).abs() < 1e-9);
        let cells = [Cell {
            frame: Frame::filled(100, 1000, [9, 9, 9]),
            label: "tall".to_string(),
        }];
        let (sheet, _) = compose(&cells, 1, 324, None, &painter()).expect("composes");
        assert_eq!(sheet.height, GAP + LABEL_H + 600 + GAP);
    }

    #[test]
    fn long_labels_are_cut_to_their_column_and_keep_the_scale() {
        assert_eq!(fit_label("idle", "  (0.50x)", 300), "idle  (0.50x)");
        let cut = fit_label("A golden_head/LiquidGlass_02_mid_cruise", "  (2.00x)", 192);
        assert!(cut.ends_with("\u{2026}  (2.00x)"), "{cut}");
        assert!(cut.chars().count() <= (192.0 / (LABEL_PX * 0.52)) as usize);
    }

    #[test]
    fn refuses_a_sheet_over_16_megapixels() {
        let cells: Vec<Cell> = (0..12)
            .map(|_| Cell {
                frame: Frame::filled(800, 800, [0; 3]),
                label: String::new(),
            })
            .collect();
        assert!(compose(&cells, 1, 7680, None, &painter()).is_err());
    }
}
