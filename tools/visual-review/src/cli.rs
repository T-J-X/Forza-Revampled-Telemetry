//! What every command shares: the context, loading, regions on full frames and crops, zoom,
//! numbers and output paths.
use crate::args::Args;
use crate::error::{usage, Result};
use crate::frame::Frame;
use crate::geom::Rect;
use crate::tokens::Tokens;
use std::path::{Path, PathBuf};

/// Pixels added round a view's envelope when `--view` picks a region.
pub const VIEW_MARGIN: i64 = 40;

/// The margin `preview-export` adds round the envelope in `<state>_crop.png` and in the
/// sequence frames (`tools/preview-export/src/cluster.rs`).
pub const CROP_MARGIN: i64 = 48;

/// The largest image Claude's Read tool shows 1:1: at most 1568 px a side and about 1.15
/// megapixels. Anything bigger is downscaled before the model sees it.
pub const VIEW_MAX_SIDE: u32 = 1568;
/// See [`VIEW_MAX_SIDE`].
pub const VIEW_MAX_PIXELS: u64 = 1_150_000;

/// The repo root, its tokens and the settings every command shares.
#[derive(Debug, Clone)]
pub struct Ctx {
    /// The repo root (fonts and tokens are read from here).
    pub root: PathBuf,
    /// `data/design_tokens.json`.
    pub tokens: Tokens,
    /// `mockups/`, relative when the tool runs from the repo root.
    pub mockups: PathBuf,
    /// `mockups/review/`: where outputs go when no dash can be told from the input.
    pub review_dir: PathBuf,
    /// `--straight`: inputs with transparency have straight alpha (a design-tool export).
    pub straight: bool,
}

impl Ctx {
    /// The repo this binary was built from.
    ///
    /// # Errors
    /// When the tokens cannot be read.
    pub fn new(straight: bool) -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
        let tokens = Tokens::load(&root)?;
        let at_root = match (
            std::env::current_dir().and_then(std::fs::canonicalize),
            std::fs::canonicalize(&root),
        ) {
            (Ok(cwd), Ok(r)) => cwd == r,
            _ => false,
        };
        let mockups = if at_root {
            PathBuf::from("mockups")
        } else {
            root.join("mockups")
        };
        Ok(Self {
            review_dir: mockups.join("review"),
            root,
            tokens,
            mockups,
            straight,
        })
    }

    /// Reads an image as the panel shows it. Says once how a file with transparency is read.
    ///
    /// # Errors
    /// An image error when the file cannot be read.
    pub fn load(&self, path: &Path) -> Result<Frame> {
        let f = Frame::load(path, self.straight)?;
        if f.alpha.is_some() {
            out!(
                "note: {} has transparency, read as {} alpha{}",
                path.display(),
                if self.straight { "straight" } else { "premultiplied" },
                if self.straight {
                    ""
                } else {
                    " (pass --straight for a design-tool export)"
                }
            );
        }
        Ok(f)
    }

    /// Where `frame` sits in the 3840x2160 canvas: (0, 0) for a full frame, the envelope minus
    /// [`CROP_MARGIN`] for a `<state>_crop.png` or sequence frame, `None` for anything else.
    pub fn origin(&self, frame: &Frame) -> Option<(i64, i64)> {
        if (frame.width, frame.height) == self.tokens.canvas {
            return Some((0, 0));
        }
        let size = (i64::from(frame.width), i64::from(frame.height));
        [self.tokens.cockpit, self.tokens.hood]
            .iter()
            .map(|v| v.region(CROP_MARGIN))
            .find(|r| (r.w, r.h) == size)
            .map(|r| (r.x, r.y))
    }

    /// The view a full frame shows by its file name: the hood envelope for `hood_*` files, the
    /// cockpit one otherwise.
    pub fn view_for(&self, path: &Path) -> crate::tokens::View {
        if stem(path).starts_with("hood_") {
            self.tokens.hood
        } else {
            self.tokens.cockpit
        }
    }
}

/// The `--anchor` offset: with it, every coordinate flag is local to that view's anchor.
///
/// # Errors
/// A usage error for an unknown view.
pub fn anchor(args: &Args, ctx: &Ctx) -> Result<(i64, i64)> {
    match args.one("--anchor") {
        None => Ok((0, 0)),
        Some(v) => Ok(ctx.tokens.view(v)?.anchor),
    }
}

/// The region from `--region` (moved by `--anchor`) or `--view` (the envelope plus 40 px), in
/// full-frame 4K coordinates.
///
/// # Errors
/// A usage error for a bad rectangle or view.
pub fn region(args: &Args, ctx: &Ctx) -> Result<Option<Rect>> {
    if let Some(r) = args.one("--region") {
        let (ax, ay) = anchor(args, ctx)?;
        return Ok(Some(Rect::parse(r)?.offset(ax, ay)));
    }
    if let Some(v) = args.one("--view") {
        return Ok(Some(ctx.tokens.view(v)?.region(VIEW_MARGIN)));
    }
    Ok(None)
}

/// The region a command works on, in `frame`'s own pixels, clamped to it: `--region` or
/// `--view` (4K coordinates, mapped onto a crop), else the view envelope plus 40 px of a full
/// frame (the hood one for `hood_*` files), else the whole image. Prints the region it chose.
///
/// # Errors
/// A usage error when the region misses the image, or names 4K coordinates on an image that
/// is neither a full frame nor a known crop.
pub fn work_region(args: &Args, ctx: &Ctx, frame: &Frame, path: &Path) -> Result<Rect> {
    let whole = Rect::new(0, 0, i64::from(frame.width), i64::from(frame.height));
    let r = match (region(args, ctx)?, ctx.origin(frame)) {
        (Some(r), Some((ox, oy))) => r.offset(-ox, -oy),
        (Some(r), None) if args.one("--anchor").is_none() && args.one("--view").is_none() => r,
        (Some(_), None) => {
            return Err(usage(format!(
                "{} is {}x{}: --view and --anchor need a full 3840x2160 frame or a preview-export crop (1536x656 cockpit, 1856x496 hood); give --region in the picture's own pixels",
                path.display(),
                frame.width,
                frame.height
            )))
        }
        (None, Some((0, 0))) => ctx.view_for(path).region(VIEW_MARGIN),
        (None, _) => whole,
    };
    r.clamp_to(frame.width, frame.height).ok_or_else(|| {
        usage(format!(
            "region {r} is outside the {}x{} image {}",
            frame.width,
            frame.height,
            path.display()
        ))
    })
}

/// Where `r` (in `frame`'s own pixels) sits in the 4K canvas, for printing: empty for a full
/// frame or a picture of unknown place, else ` (4K X,Y WxH)`.
pub fn in_4k(ctx: &Ctx, frame: &Frame, r: Rect) -> String {
    match ctx.origin(frame) {
        Some((0, 0)) | None => String::new(),
        Some((x, y)) => format!(" (4K {})", r.offset(x, y)),
    }
}

/// Every `--mask X,Y,W,H` (moved by `--anchor`) in `frame`'s own pixels: areas to black out,
/// such as the logo zone before a blind review.
///
/// # Errors
/// A usage error for a bad rectangle or view.
pub fn masks(args: &Args, ctx: &Ctx, origin: (i64, i64)) -> Result<Vec<Rect>> {
    let (ax, ay) = anchor(args, ctx)?;
    args.all("--mask")
        .into_iter()
        .map(|m| Rect::parse(m).map(|r| r.offset(ax - origin.0, ay - origin.1)))
        .collect()
}

/// `--over BACKDROP`: an image (scaled to `w` x `h`) or a `#RRGGBB` colour to composite a render
/// over, for example a bright sky or snow scene for a hood view. `None` without the flag.
///
/// # Errors
/// A usage error for a bad colour, or an image error for an unreadable file.
pub fn backdrop(args: &Args, ctx: &Ctx, w: u32, h: u32) -> Result<Option<Frame>> {
    let Some(v) = args.one("--over") else {
        return Ok(None);
    };
    if v.starts_with('#') {
        let c = crate::overlay::parse_color(v, "--over")?;
        return Ok(Some(Frame::filled(w, h, c)));
    }
    let f = ctx.load(Path::new(v))?;
    Ok(Some(f.resize(w, h)))
}

/// `--zoom`, 1 to 8 (default 1).
///
/// # Errors
/// A usage error outside that range.
pub fn zoom(args: &Args) -> Result<u32> {
    let z = args.num("--zoom", 1u32)?;
    if (1..=8).contains(&z) {
        Ok(z)
    } else {
        Err(usage("--zoom must be 1 to 8"))
    }
}

/// `n` comma-separated numbers.
///
/// # Errors
/// A usage error naming `what` when there are not exactly `n` numbers.
pub fn numbers(s: &str, n: usize, what: &str) -> Result<Vec<f64>> {
    let v: Vec<f64> = s
        .split(',')
        .map(|p| p.trim().parse::<f64>())
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| usage(format!("{what}: {s:?} is not {n} comma-separated numbers")))?;
    if v.len() == n && v.iter().all(|x| x.is_finite() && x.abs() <= 1e6) {
        Ok(v)
    } else {
        Err(usage(format!(
            "{what}: {s:?} is not {n} comma-separated numbers within a million"
        )))
    }
}

/// `WxH`.
///
/// # Errors
/// A usage error for anything else.
pub fn size(s: &str, what: &str) -> Result<(u32, u32)> {
    let bad = || usage(format!("{what}: {s:?} is not WxH"));
    let (w, h) = s.split_once(['x', 'X']).ok_or_else(bad)?;
    let w: u32 = w.trim().parse().map_err(|_| bad())?;
    let h: u32 = h.trim().parse().map_err(|_| bad())?;
    if w == 0 || h == 0 {
        return Err(bad());
    }
    Ok((w, h))
}

/// The file name of `path` without its extension.
pub fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string())
}

/// The dash an input belongs to, by its place under `mockups/` (`<Name>/...`,
/// `_before/<Name>/...`, `references/<Name>/...`) or, for a golden, by its file name
/// (`<Name>_<state>.png`) when `mockups/<Name>/` exists.
fn dash_of(ctx: &Ctx, input: &Path) -> Option<String> {
    let parent = input
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if let (Ok(p), Ok(m)) = (std::fs::canonicalize(parent), std::fs::canonicalize(&ctx.mockups)) {
        if let Ok(rel) = p.strip_prefix(&m) {
            let parts: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            return match parts.first().map(String::as_str) {
                None | Some("review" | "blind") => None,
                Some("_before" | "references") => parts.get(1).cloned(),
                Some(name) => Some(name.to_string()),
            };
        }
    }
    let theme = stem(input).split('_').next().map(str::to_string)?;
    ctx.mockups.join(&theme).is_dir().then_some(theme)
}

/// Where a command writes: `--out`, else `mockups/<Name>/review/` for an input that belongs to
/// a dash (see [`dash_of`]), else `mockups/review/`. Nothing is written into a tracked folder by
/// default.
pub fn out_dir(args: &Args, ctx: &Ctx, input: &Path) -> PathBuf {
    if let Some(d) = args.one("--out") {
        return PathBuf::from(d);
    }
    match dash_of(ctx, input) {
        Some(name) => ctx.mockups.join(name).join("review"),
        None => ctx.review_dir.clone(),
    }
}

/// The file a single-output command writes: `--out` (a `.png` name, or a folder to write the
/// default name into), else `<out dir>/<stem>_<suffix>.png`.
pub fn out_file(args: &Args, ctx: &Ctx, input: &Path, suffix: &str) -> PathBuf {
    let name = format!("{}_{suffix}.png", stem(input));
    match args.one("--out") {
        Some(f) if Path::new(f).is_dir() || f.ends_with(['/', '\\']) => PathBuf::from(f).join(name),
        Some(f) => PathBuf::from(f),
        None => out_dir(&Args::default(), ctx, input).join(name),
    }
}

/// A note when an output is too big for Claude to see 1:1.
pub fn view_note(w: u32, h: u32) -> Option<String> {
    let big = w > VIEW_MAX_SIDE || h > VIEW_MAX_SIDE || u64::from(w) * u64::from(h) > VIEW_MAX_PIXELS;
    big.then(|| {
        format!(
            "note: {w}x{h} is larger than Claude's 1:1 view ({VIEW_MAX_SIDE} px a side, 1.15 MP), so the Read tool shows it downscaled; use tiles or a crop to see pixels"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Ctx {
        Ctx::new(false).expect("tokens")
    }

    #[test]
    fn numbers_and_sizes_parse() {
        assert_eq!(numbers("1, 2.5,-3", 3, "t").ok(), Some(vec![1.0, 2.5, -3.0]));
        assert!(numbers("1,2", 3, "t").is_err());
        assert!(numbers("1,x,3", 3, "t").is_err());
        assert!(numbers("1,2,1e10", 3, "t").is_err());
        assert_eq!(size("1400x800", "t").ok(), Some((1400, 800)));
        assert!(size("1400", "t").is_err() && size("0x5", "t").is_err());
    }

    #[test]
    fn view_note_flags_images_claude_downscales() {
        assert!(view_note(1520, 640).is_none());
        assert!(view_note(1600, 400).is_some());
        assert!(view_note(1200, 1000).is_some());
        assert_eq!(
            stem(Path::new("mockups/LiquidGlass/02_mid_cruise.png")),
            "02_mid_cruise"
        );
    }

    #[test]
    fn crops_know_where_they_sit_in_the_canvas() {
        let c = ctx();
        assert_eq!(c.origin(&Frame::filled(3840, 2160, [0; 3])), Some((0, 0)));
        assert_eq!(c.origin(&Frame::filled(1536, 656, [0; 3])), Some((1152, 1467)));
        assert_eq!(c.origin(&Frame::filled(1856, 496, [0; 3])), Some((992, 1492)));
        assert_eq!(c.origin(&Frame::filled(100, 100, [0; 3])), None);
        assert_eq!(c.view_for(Path::new("x/hood_02_mid_cruise.png")), c.tokens.hood);
        assert_eq!(c.view_for(Path::new("x/02_mid_cruise.png")), c.tokens.cockpit);
    }

    #[test]
    fn an_anchor_region_maps_onto_a_crop() {
        let c = ctx();
        let args = Args::parse(
            ["--anchor", "cockpit", "--region", "-120,-150,240,200"]
                .iter()
                .map(|s| (*s).to_string()),
            &[],
        )
        .expect("parses");
        let crop = Frame::filled(1536, 656, [0; 3]);
        let r = work_region(&args, &c, &crop, Path::new("frame_42.png")).expect("inside");
        // Full-frame (1800, 1685) minus the crop origin (1152, 1467).
        assert_eq!(r, Rect::new(648, 218, 240, 200));
        let other = Frame::filled(100, 100, [0; 3]);
        assert!(work_region(&args, &c, &other, Path::new("photo.png")).is_err());
    }
}
