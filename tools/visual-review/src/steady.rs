//! `static`: the parts that look the same in every state (bezels, labels, logo, housing). They
//! must stay dimmer than the live data, and static structure stays at or below 20% relative
//! luminance (RESEARCH principle 9, the OLED-care cap): a static bright area is both glare and an
//! image-retention risk on the panel.
//!
//! Four states alone are not enough: different readouts share strokes (the "1" of 118 and 152,
//! the bowl of 0 and 8), so those pixels look the same in all four. Pass the motion sequence as
//! well (a folder): its pull sweeps every readout through many values, and a pixel counts as
//! static only when it never changes in any picture.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::components::{label, Bounds};
use crate::diff::distance;
use crate::error::{file_err, usage, Result};
use crate::flash::{file_name, natural_cmp};
use crate::frame::{linear_lut, luminance, Frame};
use crate::geom::Rect;
use crate::overlay::{Map, Mark, Painter, CYAN, YELLOW};
use fh6_kit::color::Oklab;
use std::path::{Path, PathBuf};

/// A pixel is "the same" in two pictures below this OKLab distance (x100).
const SAME: f32 = 2.0;
/// A pixel is lit above this relative luminance.
const LIT: f32 = 0.002;
/// RESEARCH principle 9: static structure stays at or below this relative luminance.
const STATIC_CAP: f32 = 0.20;
/// Gaps closed when static pixels are grouped into areas.
const AREA_GAP: u32 = 8;
/// Static areas smaller than this are left out of the list (stray edge pixels).
const AREA_MIN_PX: u64 = 24;

/// What `static` measured.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Lit pixels that are the same in every picture.
    pub static_px: u64,
    /// Lit pixels that change.
    pub live_px: u64,
    /// The 95th percentile and the maximum relative luminance of the static pixels, `None` when
    /// there are none.
    pub static_luma: Option<(f32, f32)>,
    /// The same for the live pixels.
    pub live_luma: Option<(f32, f32)>,
    /// Which pixels are static.
    pub mask: Vec<bool>,
    /// The static areas, largest first.
    pub areas: Vec<Area>,
}

/// One area of static pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Area {
    /// The box round it, in the measured picture's pixels.
    pub rect: Rect,
    /// Its static pixels.
    pub px: u64,
    /// The 95th percentile and the maximum relative luminance of them.
    pub luma: (f32, f32),
}

fn p95_max(mut v: Vec<f32>) -> Option<(f32, f32)> {
    v.sort_by(f32::total_cmp);
    let last = *v.last()?;
    let p = v
        .get(((v.len() - 1) as f32 * 0.95).round() as usize)
        .copied()
        .unwrap_or(last);
    Some((p, last))
}

/// The running state of the measure: each pixel's first colour, whether it has stayed the same,
/// and its brightest and darkest luminance.
#[derive(Debug, Clone)]
pub struct Tracker {
    width: u32,
    height: u32,
    base: Vec<Oklab>,
    same: Vec<bool>,
    peak: Vec<f32>,
    low: Vec<f32>,
    lut: [f32; 256],
}

impl Tracker {
    /// Starts at `first`.
    pub fn new(first: &Frame) -> Self {
        let lut = linear_lut();
        let n = first.width as usize * first.height as usize;
        let mut t = Self {
            width: first.width,
            height: first.height,
            base: Vec::with_capacity(n),
            same: vec![true; n],
            peak: Vec::with_capacity(n),
            low: Vec::with_capacity(n),
            lut,
        };
        for p in first.rgb.chunks_exact(3) {
            let (lab, y) = t.pixel(p);
            t.base.push(lab);
            t.peak.push(y);
            t.low.push(y);
        }
        t
    }

    /// Each pixel on its own (no filtering: a pixel next to a moving edge is judged on itself).
    fn pixel(&self, p: &[u8]) -> (Oklab, f32) {
        let lut = &self.lut;
        let c = [lut[usize::from(p[0])], lut[usize::from(p[1])], lut[usize::from(p[2])]];
        (Oklab::from_linear_rgb(c[0], c[1], c[2]), luminance(c))
    }

    /// Takes in the next picture.
    ///
    /// # Errors
    /// A usage error when its size differs from the first picture's.
    pub fn feed(&mut self, f: &Frame) -> Result<()> {
        if (f.width, f.height) != (self.width, self.height) {
            return Err(usage(format!(
                "a picture is {}x{} where the first was {}x{}: give pictures of one view (all cockpit or all hood_*)",
                f.width, f.height, self.width, self.height
            )));
        }
        for (i, p) in f.rgb.chunks_exact(3).enumerate() {
            let (lab, y) = self.pixel(p);
            if let (Some(base), Some(same), Some(peak), Some(low)) = (
                self.base.get(i),
                self.same.get_mut(i),
                self.peak.get_mut(i),
                self.low.get_mut(i),
            ) {
                *same &= distance(lab, *base) < SAME;
                *peak = peak.max(y);
                *low = low.min(y);
            }
        }
        Ok(())
    }

    /// The report so far.
    pub fn report(&self) -> Report {
        let n = self.base.len();
        let mut mask = vec![false; n];
        let (mut st, mut live) = (Vec::new(), Vec::new());
        for (i, m) in mask.iter_mut().enumerate() {
            let (peak, low, same) = (
                self.peak.get(i).copied().unwrap_or(0.0),
                self.low.get(i).copied().unwrap_or(0.0),
                self.same.get(i).copied().unwrap_or(false),
            );
            if peak <= LIT {
                continue;
            }
            if same && low > LIT {
                *m = true;
                st.push(peak);
            } else {
                live.push(peak);
            }
        }
        let (ids, count) = label(&mask, self.width, self.height, AREA_GAP);
        let mut gathered: Vec<(Bounds, Vec<f32>)> = vec![(Bounds::default(), Vec::new()); count];
        let wu = self.width as usize;
        for (i, &id) in ids.iter().enumerate().filter(|(_, id)| **id != u32::MAX) {
            if let Some((b, ys)) = gathered.get_mut(id as usize) {
                b.add((i % wu) as u32, (i / wu) as u32);
                ys.push(self.peak.get(i).copied().unwrap_or(0.0));
            }
        }
        let mut areas: Vec<Area> = gathered
            .into_iter()
            .filter(|(_, ys)| ys.len() as u64 >= AREA_MIN_PX)
            .filter_map(|(b, ys)| {
                let px = ys.len() as u64;
                Some(Area {
                    rect: b.rect()?,
                    px,
                    luma: p95_max(ys)?,
                })
            })
            .collect();
        areas.sort_by_key(|a| std::cmp::Reverse(a.px));
        Report {
            static_px: st.len() as u64,
            live_px: live.len() as u64,
            static_luma: p95_max(st),
            live_luma: p95_max(live),
            mask,
            areas,
        }
    }
}

/// Splits the lit pixels of `frames` (same size) into static and live.
#[cfg(test)]
pub fn measure(frames: &[Frame]) -> Option<Report> {
    let mut t = Tracker::new(frames.first()?);
    for f in frames.iter().skip(1) {
        t.feed(f).ok()?;
    }
    Some(t.report())
}

/// The review sequence's first frame after the idle, which holds the dash's start-up.
const AFTER_STARTUP: usize = 10;

/// Whether `v` (sorted) is the 60-frame review sequence (`frame_00.png` to `frame_59.png`).
fn is_review_sequence(v: &[PathBuf]) -> bool {
    v.len() == 60
        && v.iter()
            .enumerate()
            .all(|(i, p)| file_name(p) == format!("frame_{i:02}.png"))
}

/// The pictures: files as given, folders as every PNG in them in counting order. The review
/// sequence's folder gives frames 10 to 59: frames 0 to 9 hold the start-up, where labels that
/// are static afterwards still fade in.
///
/// # Errors
/// A file error for a missing file or an unreadable or empty folder.
fn inputs(args: &Args) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for a in args.pos() {
        let p = Path::new(a);
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(p)
                .map_err(|e| file_err(p, e))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|q| q.is_file() && q.extension().is_some_and(|x| x.eq_ignore_ascii_case("png")))
                .collect();
            if v.is_empty() {
                return Err(file_err(p, "holds no PNG files"));
            }
            v.sort_by(|a, b| natural_cmp(&file_name(a), &file_name(b)));
            if is_review_sequence(&v) {
                out!(
                    "note: {} is the review sequence: frames {AFTER_STARTUP} to 59 are used (0 to 9 hold the start-up)",
                    p.display()
                );
                v.drain(..AFTER_STARTUP);
            }
            out.extend(v);
        } else if p.is_file() {
            out.push(p.to_path_buf());
        } else {
            return Err(file_err(p, "not found"));
        }
    }
    Ok(out)
}

/// The region every picture is measured over, in 4K coordinates: `--region` or `--view`, else
/// the envelope plus 40 px of the first picture's view. `None` for pictures of unknown place.
fn region_4k(args: &Args, ctx: &Ctx, first: &Frame, path: &Path) -> Result<Option<Rect>> {
    if let Some(r) = cli::region(args, ctx)? {
        return Ok(Some(r));
    }
    Ok(match ctx.origin(first) {
        Some((0, 0)) => Some(ctx.view_for(path).region(cli::VIEW_MARGIN)),
        Some(o) => {
            let hood = ctx.tokens.hood.region(cli::CROP_MARGIN);
            let view = if (hood.x, hood.y) == o {
                ctx.tokens.hood
            } else {
                ctx.tokens.cockpit
            };
            Some(view.region(cli::VIEW_MARGIN))
        }
        None => None,
    })
}

/// `static`: prints the static and live luminance, lists the static areas and draws them.
///
/// # Errors
/// Usage, file, image and overlay errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--view", "--region", "--anchor", "--mask", "--out"])?;
    let paths = inputs(args)?;
    let (Some(first_path), true) = (paths.first(), paths.len() >= 2) else {
        return Err(usage(
            "static needs two or more renders of different states, and best the sequence folder too: <state>.png ... mockups/<Name>/sequence",
        ));
    };
    let head = ctx.load(first_path)?;
    let area = region_4k(args, ctx, &head, first_path)?;
    // Every picture cut to the same 4K region, masked, and fed one at a time.
    let cut = |f: Frame, p: &Path| -> Result<(Frame, Rect)> {
        let origin = ctx.origin(&f);
        let mut f = f;
        for m in cli::masks(args, ctx, origin.unwrap_or((0, 0)))? {
            f.fill(m, [0, 0, 0]);
        }
        let r = match (area, origin) {
            (Some(r4), Some((ox, oy))) => r4.offset(-ox, -oy),
            (Some(r), None) if args.one("--view").is_none() && args.one("--anchor").is_none() => r,
            (Some(_), None) => {
                return Err(usage(format!(
                    "{} is {}x{}: --view and --anchor need full frames or preview-export crops",
                    p.display(),
                    f.width,
                    f.height
                )))
            }
            (None, _) => Rect::new(0, 0, i64::from(f.width), i64::from(f.height)),
        };
        let r = r
            .clamp_to(f.width, f.height)
            .ok_or_else(|| usage(format!("{}: the region misses the picture", p.display())))?;
        let c = f
            .crop(r)
            .ok_or_else(|| usage(format!("{}: the region misses the picture", p.display())))?;
        Ok((c, r))
    };
    let (base, base_rect) = cut(head, first_path)?;
    let origin = match area {
        Some(r) => (r.x, r.y),
        None => (base_rect.x, base_rect.y),
    };
    let mut tracker = Tracker::new(&base);
    for p in paths.iter().skip(1) {
        let (f, _) = cut(Frame::load(p, ctx.straight)?, p)?;
        tracker.feed(&f).map_err(|e| match e {
            crate::error::Error::Usage(m) => usage(format!("{}: {m}", p.display())),
            other => other,
        })?;
    }
    let rep = tracker.report();
    let place = |r: Rect| {
        if area.is_some() {
            r.offset(origin.0, origin.1)
        } else {
            r
        }
    };
    out!(
        "static over {} pictures ({} .. {}), region {}{}",
        paths.len(),
        file_name(first_path),
        paths.last().map(|p| file_name(p)).unwrap_or_default(),
        place(Rect::new(0, 0, i64::from(base.width), i64::from(base.height))),
        if area.is_some() { " (4K)" } else { "" }
    );
    if paths.len() <= 8 {
        out!(
            "note: with only {} pictures, different readouts that share strokes look static; add the sequence folder (mockups/<Name>/sequence) so every readout sweeps",
            paths.len()
        );
    }
    out!(
        "static lit pixels (the same in every picture): {}; live lit pixels: {}",
        rep.static_px,
        rep.live_px
    );
    let fmt = |l: Option<(f32, f32)>| l.map_or("none".to_string(), |(p, m)| format!("p95 {p:.3}, max {m:.3}"));
    out!(
        "relative luminance: static {}; live {}",
        fmt(rep.static_luma),
        fmt(rep.live_luma)
    );
    for (i, a) in rep.areas.iter().take(10).enumerate() {
        out!(
            "  static area {}: {} px within {}, p95 {:.3}, max {:.3}{}",
            i + 1,
            a.px,
            place(a.rect),
            a.luma.0,
            a.luma.1,
            if a.luma.0 > STATIC_CAP {
                "  <- over the 0.2 cap"
            } else {
                ""
            }
        );
    }
    if rep.areas.len() > 10 {
        out!("  ({} smaller areas not listed)", rep.areas.len() - 10);
    }
    match (rep.static_luma, rep.live_luma) {
        (None, _) => out!("static structure: NOT ASSESSED (no lit pixel is the same in every picture)"),
        (Some((sp, _)), live) => {
            out!(
                "static structure stays at or below {STATIC_CAP} (p95): {}",
                if sp <= STATIC_CAP { "yes" } else { "no" }
            );
            match live {
                None => {
                    out!("static parts dimmer than the live data: NOT ASSESSED (nothing changes between the pictures)")
                }
                Some((lp, _)) => out!(
                    "the static parts are dimmer than the live data (p95): {}",
                    if sp <= lp { "yes" } else { "no" }
                ),
            }
        }
    }
    let mut map = base.dimmed(0.35);
    for (px, m) in map.rgb.chunks_exact_mut(3).zip(&rep.mask) {
        if *m {
            px.copy_from_slice(&CYAN);
        }
    }
    let mut marks: Vec<Mark> = rep
        .areas
        .iter()
        .take(6)
        .map(|a| Mark::Box {
            rect: a.rect.grow(4),
            label: format!("static {} px, p95 {:.3}", a.px, a.luma.0),
            color: if a.luma.0 > STATIC_CAP { YELLOW } else { CYAN },
        })
        .collect();
    marks.push(Mark::Caption(format!(
        "static parts (the same in all {} pictures) in cyan; yellow boxes are over the 0.2 cap",
        paths.len()
    )));
    let map = Painter::new(&ctx.root).annotated(&map, &marks, Map::IDENTITY)?;
    let dest = cli::out_file(args, ctx, first_path, "static");
    map.save(&dest)?;
    out!("static map: {}", dest.display());
    if let Some(note) = cli::view_note(map.width, map.height) {
        out!("{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fixed_bezel_is_static_and_a_changing_digit_is_live() {
        let frames: Vec<Frame> = (0..3)
            .map(|i| {
                let mut f = Frame::filled(20, 10, [0; 3]);
                f.fill(Rect::new(0, 0, 20, 2), [60, 60, 60]);
                f.fill(Rect::new(5 + i * 3, 5, 2, 3), [255, 255, 255]);
                f
            })
            .collect();
        let r = measure(&frames).expect("frames");
        assert_eq!(r.static_px, 40);
        assert!(r.live_px >= 6);
        let (sp, lp) = (r.static_luma.expect("static").0, r.live_luma.expect("live").0);
        assert!(sp < lp);
        assert!(sp <= STATIC_CAP);
        // The bezel is one area.
        assert_eq!(r.areas.len(), 1);
        assert_eq!(r.areas[0].rect, Rect::new(0, 0, 20, 2));
    }

    #[test]
    fn a_shared_stroke_is_static_in_four_states_but_live_once_the_readout_sweeps() {
        // A digit stroke lit in the four states, dark in one sweep frame.
        let lit = |on: bool| {
            let mut f = Frame::filled(12, 12, [0; 3]);
            if on {
                f.fill(Rect::new(4, 0, 3, 12), [255, 255, 255]);
            }
            f
        };
        let states: Vec<Frame> = (0..4).map(|_| lit(true)).collect();
        assert!(measure(&states).expect("frames").static_px > 0);
        let mut with_sweep = states.clone();
        with_sweep.push(lit(false));
        let r = measure(&with_sweep).expect("frames");
        assert_eq!(r.static_px, 0);
        assert!(r.static_luma.is_none());
    }

    #[test]
    fn nothing_lit_is_not_assessed() {
        let frames = vec![Frame::filled(4, 4, [0; 3]); 2];
        let r = measure(&frames).expect("frames");
        assert_eq!((r.static_px, r.live_px), (0, 0));
        assert!(r.static_luma.is_none() && r.live_luma.is_none());
        assert!(r.areas.is_empty());
    }
}
