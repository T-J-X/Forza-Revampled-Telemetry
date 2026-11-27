//! Numbered marks over a frame: an SVG overlay rasterised by resvg (pure Rust) with the repo's
//! OFL fonts and composited onto the frame. Labels go in a legend strip under the image, so they
//! never cover the pixels under review or collide with each other. Also the `annotate` command.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{usage, Error, Result};
use crate::frame::Frame;
use crate::geom::Rect;
use resvg::{tiny_skia, usvg};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

/// Findings and passes.
pub const YELLOW: [u8; 3] = [255, 214, 10];
/// Failures.
pub const MAGENTA: [u8; 3] = [255, 64, 200];
/// A second layer (sub-threshold changes).
pub const CYAN: [u8; 3] = [64, 220, 255];
/// Plain text.
pub const WHITE: [u8; 3] = [240, 244, 248];
/// The legend strip's background.
const LEGEND_BG: [u8; 3] = [12, 14, 18];
/// The narrowest output that carries a legend or a caption.
const MIN_TEXT_WIDTH: u32 = 480;

/// The label faces, from the repo's OFL font pack.
const FONT_FILES: [&str; 3] = [
    "assets/fonts/titillium-web/TitilliumWeb-SemiBold.ttf",
    "assets/fonts/titillium-web/TitilliumWeb-Bold.ttf",
    "assets/fonts/b612-mono/B612Mono-Bold.ttf",
];
const FONT: &str = "Titillium Web";

/// Something drawn over a frame, in source pixels (see [`Map`]).
#[derive(Debug, Clone, PartialEq)]
pub enum Mark {
    /// A numbered box; its label goes in the legend.
    Box {
        /// The box.
        rect: Rect,
        /// Its label.
        label: String,
        /// Stroke colour.
        color: [u8; 3],
    },
    /// A numbered circle; its label goes in the legend.
    Ring {
        /// Centre x.
        cx: f64,
        /// Centre y.
        cy: f64,
        /// Radius.
        r: f64,
        /// Its label.
        label: String,
        /// Stroke colour.
        color: [u8; 3],
    },
    /// A numbered arrow pointing at `to`, numbered at `from`; its label goes in the legend.
    Arrow {
        /// Tail.
        from: (f64, f64),
        /// Head.
        to: (f64, f64),
        /// Its label.
        label: String,
        /// Colour.
        color: [u8; 3],
    },
    /// Text with a dark halo at its baseline, in output pixels (not mapped).
    Text {
        /// Left edge.
        x: f64,
        /// Baseline.
        y: f64,
        /// Font size in output pixels.
        size: f64,
        /// The text.
        text: String,
        /// Colour.
        color: [u8; 3],
    },
    /// A caption band across the top.
    Caption(String),
    /// A protractor round a dial centre: ticks every 5, 15 and 45 degrees, labelled every 45
    /// (screen degrees: 0 at 3 o'clock, clockwise), for reading a needle's angle.
    Protractor {
        /// Centre x.
        cx: f64,
        /// Centre y.
        cy: f64,
        /// Radius of the scale.
        r: f64,
        /// Colour.
        color: [u8; 3],
    },
}

impl Mark {
    /// The label and colour of a numbered mark.
    fn numbered(&self) -> Option<(&str, [u8; 3])> {
        match self {
            Self::Box { label, color, .. } | Self::Ring { label, color, .. } | Self::Arrow { label, color, .. } => {
                Some((label.as_str(), *color))
            }
            Self::Text { .. } | Self::Caption(_) | Self::Protractor { .. } => None,
        }
    }
}

/// Maps mark coordinates to the output: `(v - origin) * k`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Map {
    /// Source x shown at the output's left edge.
    pub ox: f64,
    /// Source y shown at the output's top edge.
    pub oy: f64,
    /// Output pixels per source pixel.
    pub k: f64,
}

impl Map {
    /// Marks already in output pixels.
    pub const IDENTITY: Self = Self {
        ox: 0.0,
        oy: 0.0,
        k: 1.0,
    };

    fn x(self, v: f64) -> f64 {
        (v - self.ox) * self.k
    }

    fn y(self, v: f64) -> f64 {
        (v - self.oy) * self.k
    }
}

/// One legend line: the mark's number (on its first line), colour and text.
#[derive(Debug, Clone, PartialEq)]
struct Line {
    n: Option<usize>,
    color: [u8; 3],
    text: String,
}

/// Draws marks with the repo's fonts.
pub struct Painter {
    opt: usvg::Options<'static>,
}

impl Painter {
    /// Loads the label fonts from `root`. A missing font only loses the text: shapes still draw.
    pub fn new(root: &Path) -> Self {
        let mut opt = usvg::Options::default();
        let db = Arc::make_mut(&mut opt.fontdb);
        for f in FONT_FILES {
            match std::fs::read(root.join(f)) {
                Ok(bytes) => db.load_font_data(bytes),
                Err(e) => warn!("warning: {f}: {e}; labels may be drawn without text"),
            }
        }
        Self { opt }
    }

    /// Draws `marks` onto `frame` in place (numbers only, no legend).
    ///
    /// # Errors
    /// [`Error::Overlay`] when the generated SVG fails to parse.
    pub fn draw(&self, frame: &mut Frame, marks: &[Mark], map: Map) -> Result<()> {
        if marks.is_empty() || frame.width == 0 || frame.height == 0 {
            return Ok(());
        }
        let svg = svg(frame.width, 0.0, frame.height, frame.height, marks, map, &[]);
        self.render(frame, &svg)
    }

    /// `frame` with `marks` drawn on it, a caption strip above it (for a [`Mark::Caption`]) and
    /// a legend strip of their numbered labels underneath, so no text covers the picture.
    ///
    /// # Errors
    /// [`Error::Overlay`] when the generated SVG fails to parse.
    pub fn annotated(&self, frame: &Frame, marks: &[Mark], map: Map) -> Result<Frame> {
        // A legend or caption needs room to be read, however narrow the picture is.
        let has_text = marks
            .iter()
            .any(|m| matches!(m, Mark::Caption(_)) || m.numbered().is_some_and(|(label, _)| !label.trim().is_empty()));
        let width = if has_text {
            frame.width.max(MIN_TEXT_WIDTH)
        } else {
            frame.width
        };
        let font = font_px(width, frame.height);
        let band = caption_band(marks, width, font).ceil() as u32;
        let max_chars = ((f64::from(width) - font * 3.0) / (font * 0.52)).max(8.0) as usize;
        let mut lines = Vec::new();
        let mut n = 0;
        for (label, color) in marks.iter().filter_map(Mark::numbered) {
            n += 1;
            for (i, text) in wrap(label, max_chars).into_iter().enumerate() {
                lines.push(Line {
                    n: (i == 0).then_some(n),
                    color,
                    text,
                });
            }
        }
        let legend_h = if lines.is_empty() {
            0
        } else {
            (lines.len() as f64 * font * 1.4 + font * 0.9).ceil() as u32
        };
        let mut out = Frame::filled(width, band + frame.height + legend_h, LEGEND_BG);
        out.blit(frame, 0, band);
        if marks.is_empty() || out.width == 0 || out.height == 0 {
            return Ok(out);
        }
        // The picture sits under the caption strip, so every mark moves down with it.
        let below = Map {
            oy: map.oy - f64::from(band) / map.k,
            ..map
        };
        let svg = svg(width, f64::from(band), frame.height, out.height, marks, below, &lines);
        self.render(&mut out, &svg)?;
        Ok(out)
    }

    fn render(&self, frame: &mut Frame, svg: &str) -> Result<()> {
        let tree = usvg::Tree::from_str(svg, &self.opt).map_err(|e| Error::Overlay(e.to_string()))?;
        let mut pixmap = tiny_skia::Pixmap::new(frame.width, frame.height)
            .ok_or_else(|| Error::Overlay(format!("cannot draw on a {}x{} frame", frame.width, frame.height)))?;
        resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
        over(frame, pixmap.data());
        Ok(())
    }
}

/// The label size for an image of `w` x `h`.
fn font_px(w: u32, h: u32) -> f64 {
    (f64::from(w.max(h)) / 60.0).clamp(16.0, 44.0)
}

/// Greedy word wrap at `max` characters (long words are split).
fn wrap(text: &str, max: usize) -> Vec<String> {
    let max = max.max(4);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        let mut word: Vec<char> = word.chars().collect();
        while word.len() > max {
            if !cur.is_empty() {
                lines.push(std::mem::take(&mut cur));
            }
            lines.push(word.drain(..max).collect());
        }
        let word: String = word.into_iter().collect();
        if word.is_empty() {
            continue;
        }
        if !cur.is_empty() && cur.chars().count() + 1 + word.chars().count() > max {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(&word);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", c[0], c[1], c[2])
}

/// Text made safe for SVG: markup characters escaped, and characters XML 1.0 does not allow
/// (control characters from a pasted label) dropped.
fn esc(s: &str) -> String {
    s.chars()
        .filter(|&c| matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{FFFE}' && c != '\u{FFFF}'))
        .collect::<String>()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Text with a dark halo, so it reads over anything.
fn text(s: &mut String, x: f64, y: f64, size: f64, color: [u8; 3], body: &str) {
    let _ = write!(
        s,
        r##"<text x="{x:.1}" y="{y:.1}" font-family="{FONT}" font-weight="600" font-size="{size:.1}" fill="{fill}" stroke="#000000" stroke-opacity="0.9" stroke-width="{halo:.1}" stroke-linejoin="round" paint-order="stroke">{body}</text>"##,
        fill = hex(color),
        halo = size * 0.26,
        body = esc(body),
    );
}

/// The number disc of mark `n`, centred at `(cx, cy)`.
fn disc(s: &mut String, n: usize, cx: f64, cy: f64, color: [u8; 3], font: f64) {
    let r = font * 0.62;
    let _ = write!(
        s,
        r##"<circle cx="{cx:.1}" cy="{cy:.1}" r="{r:.1}" fill="{fill}" stroke="#000000" stroke-width="2"/>"##,
        fill = hex(color),
    );
    let _ = write!(
        s,
        r##"<text x="{cx:.1}" y="{ty:.1}" font-family="{FONT}" font-weight="700" font-size="{fs:.1}" text-anchor="middle" fill="#000000">{n}</text>"##,
        ty = cy + font * 0.3,
        fs = font * 0.8,
    );
}

/// The number disc at a mark's top-left corner `at`: just outside it when there is room above
/// (below the caption band, which ends at `top`), else just inside it; moved right past any
/// disc already `placed`, so close marks keep readable numbers.
fn tag(s: &mut String, n: usize, at: (f64, f64), color: [u8; 3], font: f64, top: f64, placed: &mut Vec<(f64, f64)>) {
    let r = font * 0.62;
    let cy = if at.1 - 2.0 * r - 4.0 >= top {
        at.1 - r - 4.0
    } else {
        at.1 + r + 4.0
    };
    let mut cx = at.0 + r;
    for _ in 0..=placed.len() {
        let clash = placed.iter().any(|&(px, py)| (px - cx).hypot(py - cy) < 2.0 * r + 1.0);
        if !clash {
            break;
        }
        cx += 2.0 * r + 2.0;
    }
    placed.push((cx, cy));
    disc(s, n, cx, cy, color, font);
}

/// Stroke drawn twice: dark underneath, colour on top.
fn outlined(s: &mut String, shape: &str, color: [u8; 3], line: f64) {
    let _ = write!(
        s,
        r##"<{shape} fill="none" stroke="#000000" stroke-opacity="0.85" stroke-width="{:.1}" stroke-linecap="round"/>"##,
        line + 3.0
    );
    let _ = write!(
        s,
        r#"<{shape} fill="none" stroke="{}" stroke-width="{line:.1}" stroke-linecap="round"/>"#,
        hex(color)
    );
}

/// The rows of a caption at `font` px across `w` px.
fn caption_rows(body: &str, w: u32, font: f64) -> Vec<String> {
    let max_chars = ((f64::from(w) - font * 1.2) / (font * 0.52)).max(8.0) as usize;
    wrap(body, max_chars)
}

/// The height of a caption band of `rows` rows.
fn band_of(rows: usize, font: f64) -> f64 {
    font * (0.6 + 1.3 * rows.max(1) as f64)
}

/// The height of the caption band `marks` ask for across `w` px (0 without a caption).
fn caption_band(marks: &[Mark], w: u32, font: f64) -> f64 {
    marks
        .iter()
        .find_map(|m| match m {
            Mark::Caption(body) => Some(band_of(caption_rows(body, w, font).len(), font)),
            _ => None,
        })
        .unwrap_or(0.0)
}

/// The overlay as an SVG document: the caption band across the top, marks over the image
/// (`w` x `image_h`, starting `image_top` px down) and the legend `lines` in the strip below it
/// (down to `total_h`).
fn svg(w: u32, image_top: f64, image_h: u32, total_h: u32, marks: &[Mark], map: Map, lines: &[Line]) -> String {
    let font = font_px(w, image_h);
    let line = (font / 9.0).max(2.0);
    let mut s = String::new();
    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{total_h}" viewBox="0 0 {w} {total_h}">"#
    );
    // Number discs keep clear of the caption band and of each other.
    let top = caption_band(marks, w, font);
    let mut placed = Vec::new();
    let mut n = 0;
    for m in marks {
        match m {
            Mark::Box { rect, color, .. } => {
                n += 1;
                let (x, y) = (map.x(rect.x as f64), map.y(rect.y as f64));
                let (bw, bh) = (rect.w as f64 * map.k, rect.h as f64 * map.k);
                outlined(
                    &mut s,
                    &format!(r#"rect x="{x:.1}" y="{y:.1}" width="{bw:.1}" height="{bh:.1}""#),
                    *color,
                    line,
                );
                tag(&mut s, n, (x, y), *color, font, top, &mut placed);
            }
            Mark::Ring { cx, cy, r, color, .. } => {
                n += 1;
                let (x, y, rr) = (map.x(*cx), map.y(*cy), r * map.k);
                outlined(
                    &mut s,
                    &format!(r#"circle cx="{x:.1}" cy="{y:.1}" r="{rr:.1}""#),
                    *color,
                    line,
                );
                let d = rr * std::f64::consts::FRAC_1_SQRT_2;
                tag(&mut s, n, (x - d, y - d), *color, font, top, &mut placed);
            }
            Mark::Arrow { from, to, color, .. } => {
                n += 1;
                let (x1, y1, x2, y2) = (map.x(from.0), map.y(from.1), map.x(to.0), map.y(to.1));
                let ang = (y2 - y1).atan2(x2 - x1);
                let head = font * 0.7;
                let (ax, ay) = (x2 - head * (ang - 0.45).cos(), y2 - head * (ang - 0.45).sin());
                let (bx, by) = (x2 - head * (ang + 0.45).cos(), y2 - head * (ang + 0.45).sin());
                outlined(
                    &mut s,
                    &format!(r#"line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}""#),
                    *color,
                    line,
                );
                let _ = write!(
                    s,
                    r##"<polygon points="{x2:.1},{y2:.1} {ax:.1},{ay:.1} {bx:.1},{by:.1}" fill="{}" stroke="#000000" stroke-width="1.5"/>"##,
                    hex(*color)
                );
                disc(&mut s, n, x1, y1, *color, font);
            }
            Mark::Text {
                x,
                y,
                size,
                text: body,
                color,
            } => text(&mut s, *x, *y, *size, *color, body),
            Mark::Caption(body) => {
                let rows = caption_rows(body, w, font);
                let band = band_of(rows.len(), font);
                let _ = write!(
                    s,
                    r##"<rect x="0" y="0" width="{w}" height="{band:.1}" fill="#000000" fill-opacity="0.72"/>"##
                );
                for (i, row) in rows.iter().enumerate() {
                    text(&mut s, font * 0.6, font * (1.3 + 1.3 * i as f64), font, WHITE, row);
                }
            }
            Mark::Protractor { cx, cy, r, color } => {
                let (x, y, rr) = (map.x(*cx), map.y(*cy), r * map.k);
                outlined(
                    &mut s,
                    &format!(r#"circle cx="{x:.1}" cy="{y:.1}" r="{rr:.1}""#),
                    *color,
                    1.0,
                );
                for deg in (0..360).step_by(5) {
                    let len = if deg % 45 == 0 {
                        0.12
                    } else if deg % 15 == 0 {
                        0.07
                    } else {
                        0.035
                    } * rr;
                    let (sn, cs) = f64::from(deg).to_radians().sin_cos();
                    let (x1, y1) = (x + rr * cs, y + rr * sn);
                    let (x2, y2) = (x + (rr - len) * cs, y + (rr - len) * sn);
                    let _ = write!(
                        s,
                        r#"<line x1="{x1:.1}" y1="{y1:.1}" x2="{x2:.1}" y2="{y2:.1}" stroke="{}" stroke-width="1.2"/>"#,
                        hex(*color)
                    );
                    if deg % 45 == 0 {
                        let (lx, ly) = (x + (rr + font * 0.9) * cs, y + (rr + font * 0.9) * sn + font * 0.35);
                        let _ = write!(
                            s,
                            r##"<text x="{lx:.1}" y="{ly:.1}" font-family="{FONT}" font-weight="600" font-size="{fs:.1}" text-anchor="middle" fill="{fill}" stroke="#000000" stroke-width="{halo:.1}" paint-order="stroke">{deg}</text>"##,
                            fs = font * 0.8,
                            fill = hex(*color),
                            halo = font * 0.2,
                        );
                    }
                }
            }
        }
    }
    let step = font * 1.4;
    for (i, l) in lines.iter().enumerate() {
        let base = image_top + f64::from(image_h) + font * 0.45 + step * (i as f64 + 1.0) - font * 0.35;
        if let Some(num) = l.n {
            disc(&mut s, num, font * 1.0, base - font * 0.33, l.color, font);
        }
        text(&mut s, font * 2.0, base, font, WHITE, &l.text);
    }
    s.push_str("</svg>");
    s
}

/// Premultiplied RGBA8 `px` over the frame.
fn over(frame: &mut Frame, px: &[u8]) {
    for (d, s) in frame.rgb.chunks_exact_mut(3).zip(px.chunks_exact(4)) {
        let a = u32::from(s[3]);
        if a == 0 {
            continue;
        }
        for (dc, &sc) in d.iter_mut().zip(&s[..3]) {
            *dc = (u32::from(sc) + u32::from(*dc) * (255 - a) / 255).min(255) as u8;
        }
    }
}

/// `#RRGGBB`, given to `flag`.
///
/// # Errors
/// A usage error naming `flag` for anything else.
pub fn parse_color(s: &str, flag: &str) -> Result<[u8; 3]> {
    let bad = || usage(format!("{flag}: {s:?} is not #RRGGBB"));
    let h = s.strip_prefix('#').ok_or_else(bad)?;
    if h.len() != 6 || !h.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(bad());
    }
    let byte = |i: usize| u8::from_str_radix(h.get(i..i + 2).unwrap_or(""), 16).map_err(|_| bad());
    Ok([byte(0)?, byte(2)?, byte(4)?])
}

/// Whether a mark touches the source rectangle `area` (a caption always does).
fn touches(m: &Mark, area: Rect) -> bool {
    let (x0, y0, x1, y1) = (area.x as f64, area.y as f64, area.right() as f64, area.bottom() as f64);
    let hit = |l: f64, t: f64, r: f64, b: f64| r >= x0 && l <= x1 && b >= y0 && t <= y1;
    match m {
        Mark::Box { rect, .. } => hit(rect.x as f64, rect.y as f64, rect.right() as f64, rect.bottom() as f64),
        Mark::Ring { cx, cy, r, .. } | Mark::Protractor { cx, cy, r, .. } => hit(cx - r, cy - r, cx + r, cy + r),
        Mark::Arrow { from, to, .. } => hit(from.0.min(to.0), from.1.min(to.1), from.0.max(to.0), from.1.max(to.1)),
        Mark::Text { .. } | Mark::Caption(_) => true,
    }
}

/// `annotate`: numbered boxes, rings, arrows and a caption over an image (or a crop of it), with
/// their labels in a legend underneath.
///
/// # Errors
/// Usage, image and overlay errors.
pub fn annotate(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&[
        "--box",
        "--ring",
        "--arrow",
        "--protractor",
        "--note",
        "--region",
        "--view",
        "--anchor",
        "--zoom",
        "--color",
        "--out",
    ])?;
    let [input] = args.pos() else {
        return Err(usage("annotate needs one image"));
    };
    let path = Path::new(input);
    let frame = ctx.load(path)?;
    if args.one("--anchor").is_some() && ctx.origin(&frame).is_none() {
        return Err(usage(format!(
            "{input} is {}x{}: --anchor needs a full 3840x2160 frame or a preview-export crop; give the marks in the picture's own pixels",
            frame.width, frame.height
        )));
    }
    let color = args
        .one("--color")
        .map(|c| parse_color(c, "--color"))
        .transpose()?
        .unwrap_or(YELLOW);
    let (ax, ay) = cli::anchor(args, ctx)?;
    let (fx, fy) = (ax as f64, ay as f64);
    let mut marks = Vec::new();
    for spec in args.all("--box") {
        let (r, label) = spec.split_once(':').unwrap_or((spec, ""));
        marks.push(Mark::Box {
            rect: Rect::parse(r)?.offset(ax, ay),
            label: label.to_string(),
            color,
        });
    }
    for spec in args.all("--ring") {
        let (v, label) = spec.split_once(':').unwrap_or((spec, ""));
        let v = cli::numbers(v, 3, "--ring CX,CY,R")?;
        marks.push(Mark::Ring {
            cx: v[0] + fx,
            cy: v[1] + fy,
            r: v[2].abs(),
            label: label.to_string(),
            color,
        });
    }
    for spec in args.all("--arrow") {
        let (v, label) = spec.split_once(':').unwrap_or((spec, ""));
        let v = cli::numbers(v, 4, "--arrow X1,Y1,X2,Y2")?;
        marks.push(Mark::Arrow {
            from: (v[0] + fx, v[1] + fy),
            to: (v[2] + fx, v[3] + fy),
            label: label.to_string(),
            color,
        });
    }
    for spec in args.all("--protractor") {
        let v = cli::numbers(spec, 3, "--protractor CX,CY,R")?;
        marks.push(Mark::Protractor {
            cx: v[0] + fx,
            cy: v[1] + fy,
            r: v[2].abs(),
            color: CYAN,
        });
    }
    if let Some(note) = args.one("--note") {
        marks.push(Mark::Caption(note.to_string()));
    }
    if marks.is_empty() {
        return Err(usage(
            "annotate needs at least one --box, --ring, --arrow, --protractor or --note",
        ));
    }
    let zoom = cli::zoom(args)?;
    // Marks are in 4K frame coordinates; a crop's own pixels start at its origin.
    let (fox, foy) = ctx.origin(&frame).unwrap_or((0, 0));
    let whole = Rect::new(0, 0, i64::from(frame.width), i64::from(frame.height));
    let area = if cli::region(args, ctx)?.is_some() {
        cli::work_region(args, ctx, &frame, path)?
    } else {
        whole
    };
    cli::check_size(area.w, area.h, zoom)?;
    let base = frame
        .crop(area)
        .ok_or_else(|| usage(format!("region {area} is outside the image")))?;
    let shown = area.offset(fox, foy);
    for (i, m) in marks.iter().enumerate() {
        if !touches(m, shown) {
            warn!(
                "warning: mark {} lies outside the picture ({shown} in frame coordinates); check its numbers and --anchor",
                i + 1
            );
        }
    }
    let out = Painter::new(&ctx.root).annotated(
        &base.zoom(zoom),
        &marks,
        Map {
            ox: shown.x as f64,
            oy: shown.y as f64,
            k: f64::from(zoom),
        },
    )?;
    let dest = cli::out_file(args, ctx, path, "annotated");
    out.save(&dest)?;
    out!(
        "wrote {} ({}x{}, {} marks)",
        dest.display(),
        out.width,
        out.height,
        marks.len()
    );
    if let Some(note) = cli::view_note(out.width, out.height) {
        out!("{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn marks_draw_on_the_image_and_labels_go_in_a_legend() {
        let f = Frame::filled(400, 200, [0; 3]);
        let marks = [
            Mark::Box {
                rect: Rect::new(40, 60, 120, 80),
                label: "gear <reads> & stays sharp at a glance in every state of the sequence".to_string(),
                color: YELLOW,
            },
            Mark::Ring {
                cx: 300.0,
                cy: 100.0,
                r: 40.0,
                label: String::new(),
                color: MAGENTA,
            },
            Mark::Arrow {
                from: (200.0, 180.0),
                to: (250.0, 120.0),
                label: "rim".to_string(),
                color: CYAN,
            },
            Mark::Caption("DIRECTION CHECK".to_string()),
        ];
        let out = Painter::new(&root())
            .annotated(&f, &marks, Map::IDENTITY)
            .expect("draws");
        // Widened to the narrowest width a legend can be read at.
        assert_eq!(out.width, MIN_TEXT_WIDTH);
        assert!(out.height > 200, "a legend strip was added below the image");
        // The caption has its own strip above the picture, so the picture and every mark move
        // down by the strip's height: the box's yellow stroke runs along its top edge there.
        let band = caption_band(&marks, MIN_TEXT_WIDTH, font_px(MIN_TEXT_WIDTH, 200)).ceil() as u32;
        assert!(band > 0);
        let edge = out.pixel(100, 60 + band);
        assert!(edge[0] > 200 && edge[1] > 150 && edge[2] < 80, "box edge is {edge:?}");
        // The picture's top-left pixel is under the strip, not covered by it.
        assert_eq!(out.pixel(0, band), [0, 0, 0]);
        // The caption and the legend light text pixels.
        let bright = |y0: u32, y1: u32| {
            (0..400)
                .flat_map(|x| (y0..y1).map(move |y| (x, y)))
                .filter(|&(x, y)| out.pixel(x, y)[0] > 150)
                .count()
        };
        assert!(bright(0, 30) > 20, "caption text");
        assert!(bright(200, out.height) > 50, "legend text");
        // Without labels there is no legend.
        let plain = Painter::new(&root())
            .annotated(&f, &marks[1..2], Map::IDENTITY)
            .expect("draws");
        assert_eq!((plain.width, plain.height), (400, 200));
    }

    #[test]
    fn map_moves_and_scales_marks() {
        let mut f = Frame::filled(200, 200, [0; 3]);
        let marks = [Mark::Box {
            rect: Rect::new(1050, 1050, 20, 20),
            label: String::new(),
            color: YELLOW,
        }];
        let map = Map {
            ox: 1000.0,
            oy: 1000.0,
            k: 2.0,
        };
        Painter::new(&root()).draw(&mut f, &marks, map).expect("draws");
        assert!(f.pixel(120, 100)[0] > 200, "the box lands at (100..140, 100..140)");
    }

    #[test]
    fn wraps_colours_and_escapes() {
        assert_eq!(wrap("one two three four", 9), ["one two", "three", "four"]);
        assert_eq!(wrap("abcdefghij", 4), ["abcd", "efgh", "ij"]);
        assert!(wrap("   ", 10).is_empty());
        assert_eq!(parse_color("#FFD60A", "--color").ok(), Some([255, 214, 10]));
        for bad in ["FFD60A", "#FFF", "#GG0000", "#+1+2+3"] {
            assert!(parse_color(bad, "--color").is_err(), "{bad:?}");
        }
        assert_eq!(esc("a\u{1}b\u{1b}c"), "abc");
        assert!(touches(
            &Mark::Box {
                rect: Rect::new(10, 10, 5, 5),
                label: String::new(),
                color: YELLOW
            },
            Rect::new(0, 0, 20, 20)
        ));
        assert!(!touches(
            &Mark::Box {
                rect: Rect::new(100, 100, 5, 5),
                label: String::new(),
                color: YELLOW
            },
            Rect::new(0, 0, 20, 20)
        ));
        assert_eq!(esc(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
    }
}
