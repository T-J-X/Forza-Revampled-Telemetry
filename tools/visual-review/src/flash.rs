//! `flash`: a WCAG 2.3.1 audit over a frame sequence (the 60-frame review sequence that
//! `preview-export --cluster` writes, 1:1 envelope crops at 60 fps).
//!
//! Per pixel it follows two quantities through the frames, as the working definitions in the
//! notes to WCAG's general and red flash thresholds do:
//! - general: relative luminance; a transition is a change of 0.1 or more where the darker state
//!   is below 0.8 (WCAG 2.0 to 2.2);
//! - red: `(R - G - B) x 320` on linear values (negatives set to 0); a transition is a change of
//!   more than 20 where one of the two states is a saturated red (`R / (R + G + B) >= 0.8`). This
//!   is WCAG 2.0's working definition, which PEAT still uses; WCAG 2.2 replaced it with a CIE 1976
//!   u'v' difference above 0.2 (from ISO 9241-391), which this tool does not implement.
//!
//! A flash is a pair of opposing transitions. Two rates are reported:
//! - **sustained**: a pixel whose shortest full cycle (three alternating transitions) is quicker
//!   than a third of a second flashes faster than 3 Hz for as long as the state lasts. The review
//!   sequence holds only 12 frames on the limiter, so this is the test that sees the strobe; it
//!   cannot see a cycle longer than the sequence (`fps / (frames - 2)`), and says so;
//! - **count**: more than three flashes (seven transitions) inside one second, WCAG's literal
//!   test, which needs a sequence that holds the state for a full second.
//!
//! The area test is WCAG's: flashing above the limit may cover at most 25% of any 10-degree
//! field (0.006 sr) at the viewing distance, evaluated as a disk on the panel's pixel grid over
//! the pixels that flash by either threshold, and the repo's own rule that nothing taller than
//! the shift strip flashes faster than 3 Hz (gaps of up to 6 px are closed when heights are
//! measured, so a stack of thin bars counts as one block).
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::components::{label, Bounds};
use crate::error::{file_err, usage, Result};
use crate::frame::{linear_lut, luminance, Frame};
use crate::geom::{angle_px, Rect};
use crate::overlay::{Map, Mark, Painter, CYAN, MAGENTA, WHITE, YELLOW};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

/// General flash: the smallest luminance change that counts.
const GENERAL_STEP: f32 = 0.10;
/// General flash: the darker state must be below this.
const GENERAL_DARK_MAX: f32 = 0.80;
/// Red flash: the scale on `R - G - B`.
const RED_SCALE: f32 = 320.0;
/// Red flash: the change must exceed this.
const RED_STEP: f32 = 20.0;
/// Red flash: a saturated red has at least this share of `R + G + B`.
const RED_SATURATION: f32 = 0.8;
/// More than three flashes in a second: seven transitions.
const BURST: u32 = 7;
/// Flashing faster than this fails.
const MAX_HZ: f64 = 3.0;
/// The field the area test uses.
const FIELD_DEG: f64 = 10.0;
/// The share of that field flashing may cover.
const FIELD_SHARE: f64 = 0.25;
/// Gaps up to this many pixels are closed when flashing areas and heights are measured.
const GAP_CLOSE: u32 = 6;
/// The most frames per second accepted.
const MAX_FPS: u16 = 1000;
/// Flashing areas boxed on the map (the text lists every area).
const AREAS_BOXED: usize = 8;

/// Which threshold a tracker follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    General,
    Red,
}

/// One state of a pixel: the tracked value and whether it is a saturated red.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Sample {
    v: f32,
    sat: bool,
}

impl Kind {
    /// Whether going between `a` and `b` is a transition.
    fn transition(self, a: Sample, b: Sample) -> bool {
        match self {
            Self::General => (a.v - b.v).abs() >= GENERAL_STEP && a.v.min(b.v) < GENERAL_DARK_MAX,
            Self::Red => (a.v - b.v).abs() > RED_STEP && (a.sat || b.sat),
        }
    }
}

/// The transitions of one pixel for one threshold.
#[derive(Debug, Clone, Copy)]
struct Tracker {
    lo: Sample,
    hi: Sample,
    peak: Sample,
    dir: i8,
    times: [u16; BURST as usize],
    n: u32,
    min_period: u16,
    burst: bool,
    /// Every complete bright and dark phase it showed, as ranges.
    phases: Phases,
}

/// The bright and dark phase lengths a pixel showed, each as (shortest, longest) in frames; a
/// longest of 0 means that phase was never seen complete.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Phases {
    /// Bright phases.
    pub on: (u16, u16),
    /// Dark phases.
    pub off: (u16, u16),
}

impl Phases {
    /// Both phases were seen complete at least once.
    fn seen(self) -> bool {
        self.on.1 > 0 && self.off.1 > 0
    }

    /// Takes in one complete phase of `len` frames.
    fn add(range: &mut (u16, u16), len: u16) {
        *range = if range.1 == 0 {
            (len, len)
        } else {
            (range.0.min(len), range.1.max(len))
        };
    }
}

impl std::fmt::Display for Phases {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let part = |(a, b): (u16, u16)| {
            if a == b {
                a.to_string()
            } else {
                format!("{a}-{b}")
            }
        };
        write!(f, "{} on / {} off", part(self.on), part(self.off))
    }
}

impl Tracker {
    fn new(s: Sample) -> Self {
        Self {
            lo: s,
            hi: s,
            peak: s,
            dir: 0,
            times: [0; BURST as usize],
            n: 0,
            min_period: u16::MAX,
            burst: false,
            phases: Phases::default(),
        }
    }

    /// Records a transition at frame `f` heading `dir`.
    fn mark(&mut self, f: u16, dir: i8, s: Sample, fps: u16) {
        let slot = |i: u32| (i % BURST) as usize;
        if self.n >= 1 {
            // Transitions alternate, so the one before went the other way: this ends a phase.
            let len = f.saturating_sub(self.times[slot(self.n - 1)]);
            if dir < 0 {
                Phases::add(&mut self.phases.on, len);
            } else {
                Phases::add(&mut self.phases.off, len);
            }
        }
        if self.n >= 2 {
            let prev = self.times[slot(self.n - 2)];
            self.min_period = self.min_period.min(f.saturating_sub(prev));
        }
        if self.n + 1 >= BURST && f.saturating_sub(self.times[slot(self.n + 1 - BURST)]) < fps {
            self.burst = true;
        }
        self.times[slot(self.n)] = f;
        self.n += 1;
        self.dir = dir;
        self.peak = s;
    }

    /// Feeds the state at frame `f`.
    fn feed(&mut self, kind: Kind, f: u16, s: Sample, fps: u16) {
        match self.dir {
            0 => {
                if s.v > self.hi.v {
                    self.hi = s;
                }
                if s.v < self.lo.v {
                    self.lo = s;
                }
                if s.v > self.lo.v && kind.transition(self.lo, s) {
                    self.mark(f, 1, s, fps);
                } else if s.v < self.hi.v && kind.transition(self.hi, s) {
                    self.mark(f, -1, s, fps);
                }
            }
            1 => {
                if s.v >= self.peak.v {
                    self.peak = s;
                } else if kind.transition(self.peak, s) {
                    self.mark(f, -1, s, fps);
                }
            }
            _ => {
                if s.v <= self.peak.v {
                    self.peak = s;
                } else if kind.transition(self.peak, s) {
                    self.mark(f, 1, s, fps);
                }
            }
        }
    }

    /// The fastest sustained rate seen, in Hz.
    fn hz(&self, fps: u16) -> Option<f64> {
        (self.min_period != u16::MAX && self.min_period > 0).then(|| f64::from(fps) / f64::from(self.min_period))
    }
}

/// The audit state over a sequence of equal-sized frames.
#[derive(Debug)]
pub struct Audit {
    width: u32,
    height: u32,
    fps: u16,
    frames: u16,
    general: Vec<Tracker>,
    red: Vec<Tracker>,
    brightest: Vec<u8>,
    lut: [f32; 256],
}

/// The samples of one frame's pixel.
fn samples(lut: &[f32; 256], p: &[u8]) -> (Sample, Sample) {
    let (r, g, b) = (lut[usize::from(p[0])], lut[usize::from(p[1])], lut[usize::from(p[2])]);
    let sum = r + g + b;
    let sat = sum > 0.0 && r / sum >= RED_SATURATION;
    (
        Sample {
            v: luminance([r, g, b]),
            sat,
        },
        Sample {
            v: ((r - g - b) * RED_SCALE).max(0.0),
            sat,
        },
    )
}

/// Rates above 3 Hz and how many pixels flash at each, most common first.
fn rates_of(hz: &[Option<f64>]) -> Vec<(f64, u64)> {
    let mut rates: Vec<(f64, u64)> = Vec::new();
    for h in hz.iter().flatten().filter(|h| **h > MAX_HZ) {
        match rates.iter_mut().find(|(r, _)| (r - h).abs() < 1e-6) {
            Some((_, n)) => *n += 1,
            None => rates.push((*h, 1)),
        }
    }
    rates.sort_by(|a, b| b.1.cmp(&a.1).then(b.0.total_cmp(&a.0)));
    rates
}

impl Audit {
    /// Starts an audit at `first`.
    pub fn new(first: &Frame, fps: u16) -> Self {
        let lut = linear_lut();
        let (mut general, mut red) = (Vec::new(), Vec::new());
        for p in first.rgb.chunks_exact(3) {
            let (g, r) = samples(&lut, p);
            general.push(Tracker::new(g));
            red.push(Tracker::new(r));
        }
        Self {
            width: first.width,
            height: first.height,
            fps: fps.max(1),
            frames: 1,
            general,
            red,
            brightest: first.rgb.clone(),
            lut,
        }
    }

    /// Feeds the next frame.
    ///
    /// # Errors
    /// A usage error when its size differs from the first frame's.
    pub fn feed(&mut self, frame: &Frame) -> Result<()> {
        if (frame.width, frame.height) != (self.width, self.height) {
            return Err(usage(format!(
                "frame {} is {}x{}, the first was {}x{}",
                self.frames, frame.width, frame.height, self.width, self.height
            )));
        }
        let f = self.frames;
        let fps = self.fps;
        for (((p, g), r), best) in frame
            .rgb
            .chunks_exact(3)
            .zip(self.general.iter_mut())
            .zip(self.red.iter_mut())
            .zip(self.brightest.chunks_exact_mut(3))
        {
            let (gs, rs) = samples(&self.lut, p);
            g.feed(Kind::General, f, gs, fps);
            r.feed(Kind::Red, f, rs, fps);
            for (b, &c) in best.iter_mut().zip(p) {
                *b = (*b).max(c);
            }
        }
        self.frames = self.frames.saturating_add(1);
        Ok(())
    }

    /// The findings at `eye_m` metres and user cluster scale `scale`.
    pub fn report(&self, eye_m: f64, scale: f64) -> Report {
        let field = angle_px(FIELD_DEG, eye_m) / scale.max(0.1);
        let limit = std::f64::consts::PI * (field * 0.5).powi(2) * FIELD_SHARE;
        let (w, h) = (self.width, self.height);
        let fast = |hz: &Option<f64>| hz.is_some_and(|h| h > MAX_HZ);
        let layer = |trackers: &[Tracker]| {
            let hz: Vec<Option<f64>> = trackers.iter().map(|t| t.hz(self.fps)).collect();
            let mask: Vec<bool> = hz.iter().map(fast).collect();
            let burst = trackers.iter().filter(|t| t.burst).count() as u64;
            Layer::of(mask, w, h, field, rates_of(&hz), burst)
        };
        // Either threshold: the fastest of the two rates of each pixel.
        let either: Vec<Option<f64>> = self
            .general
            .iter()
            .zip(&self.red)
            .map(|(g, r)| match (g.hz(self.fps), r.hz(self.fps)) {
                (Some(a), Some(b)) => Some(a.max(b)),
                (a, b) => a.or(b),
            })
            .collect();
        let union_mask: Vec<bool> = either.iter().map(fast).collect();
        let count_mask: Vec<bool> = self
            .general
            .iter()
            .zip(&self.red)
            .map(|(g, r)| g.burst || r.burst)
            .collect();
        let count_px = count_mask.iter().filter(|m| **m).count() as u64;
        // Each flashing pixel's range of bright and dark phase lengths (unseen when not flashing).
        let pixel_phase: Vec<Phases> = self
            .general
            .iter()
            .zip(&self.red)
            .zip(&union_mask)
            .map(|((g, r), m)| {
                if !m {
                    return Phases::default();
                }
                let t = if fast(&g.hz(self.fps)) { g } else { r };
                t.phases
            })
            .collect();
        let phases = phase_counts(pixel_phase.iter().copied());
        let hz_values: Vec<f64> = either.iter().map(|h| h.unwrap_or(0.0)).collect();
        let regions = regions(&union_mask, w, h, &hz_values, &pixel_phase);
        let measured = u32::from(self.frames).saturating_sub(2).max(1);
        Report {
            frames: self.frames,
            width: w,
            height: h,
            fps: self.fps,
            field_px: field,
            limit_px2: limit,
            general: layer(&self.general),
            red: layer(&self.red),
            union: Layer::of(union_mask, w, h, field, rates_of(&either), count_px),
            count: Layer::of(count_mask, w, h, field, Vec::new(), count_px),
            floor_hz: f64::from(self.fps) / f64::from(measured),
            phases,
            regions,
        }
    }

    /// The per-pixel brightest frame, for the map image.
    fn brightest(&self) -> Frame {
        Frame {
            width: self.width,
            height: self.height,
            rgb: self.brightest.clone(),
            alpha: None,
            straight: false,
        }
    }
}

/// Flashing pixels of one threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct Layer {
    /// Which pixels flash faster than 3 Hz.
    pub mask: Vec<bool>,
    /// How many.
    pub px: u64,
    /// The box round them.
    pub bbox: Option<Rect>,
    /// The tallest vertical extent of them, with gaps of up to 6 px closed.
    pub tallest: u32,
    /// The most of them inside one 10-degree field, and that field's centre.
    pub worst: (u64, i64, i64),
    /// Rates above 3 Hz and how many pixels flash at each, most common first.
    pub rates: Vec<(f64, u64)>,
    /// Pixels with more than three flashes inside one second.
    pub burst: u64,
}

impl Layer {
    fn of(mask: Vec<bool>, w: u32, h: u32, field: f64, rates: Vec<(f64, u64)>, burst: u64) -> Self {
        let px = mask.iter().filter(|m| **m).count() as u64;
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
        for (i, _) in mask.iter().enumerate().filter(|(_, m)| **m) {
            let (x, y) = ((i % w as usize) as u32, (i / w as usize) as u32);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let bbox = (px > 0).then(|| {
            Rect::new(
                i64::from(x0),
                i64::from(y0),
                i64::from(x1 - x0 + 1),
                i64::from(y1 - y0 + 1),
            )
        });
        let tallest = tallest_run(&mask, w, h);
        let worst = bbox.map_or((0, 0, 0), |b| max_in_disk(&mask, w, h, field, b));
        Self {
            mask,
            px,
            bbox,
            tallest,
            worst,
            rates,
            burst,
        }
    }
}

/// The tallest vertical extent of `mask` in any column, bridging gaps of up to 6 px.
pub fn tallest_run(mask: &[bool], w: u32, h: u32) -> u32 {
    let mut best = 0u32;
    for x in 0..w as usize {
        let (mut start, mut last) = (None::<u32>, 0u32);
        for y in 0..h {
            if !mask.get(y as usize * w as usize + x).copied().unwrap_or(false) {
                continue;
            }
            match start {
                Some(_) if y - last - 1 <= GAP_CLOSE => {}
                _ => start = Some(y),
            }
            last = y;
            if let Some(s) = start {
                best = best.max(y - s + 1);
            }
        }
    }
    best
}

/// One flashing area.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    /// The box round its flashing pixels.
    pub rect: Rect,
    /// How many pixels flash.
    pub px: u64,
    /// The fastest rate among them, Hz.
    pub hz: f64,
    /// The rate most of them flash at, Hz.
    pub common_hz: f64,
    /// The ranges of bright and dark phase lengths most of them show, in frames.
    pub phase: Option<Phases>,
}

/// What [`regions`] gathers for one area.
#[derive(Debug, Clone, Default)]
struct Gather {
    bounds: Bounds,
    px: u64,
    rates: Vec<(f64, u64)>,
    phases: Vec<Phases>,
}

/// How many pixels show each set of phase ranges, most common first; pixels that never
/// showed both phases complete are left out.
fn phase_counts(phases: impl Iterator<Item = Phases>) -> Vec<(Phases, u64)> {
    let mut out: Vec<(Phases, u64)> = Vec::new();
    for p in phases.filter(|p| p.seen()) {
        match out.iter_mut().find(|(q, _)| *q == p) {
            Some((_, n)) => *n += 1,
            None => out.push((p, 1)),
        }
    }
    out.sort_by_key(|p| std::cmp::Reverse(p.1));
    out
}

/// The flashing areas of `mask`, with gaps of up to 6 px closed, largest first. `hz` and
/// `phase` give each pixel's rate and phase lengths.
pub fn regions(mask: &[bool], w: u32, h: u32, hz: &[f64], phase: &[Phases]) -> Vec<Region> {
    let (ids, count) = label(mask, w, h, GAP_CLOSE);
    let wu = w as usize;
    let mut found = vec![Gather::default(); count];
    for (i, &id) in ids.iter().enumerate().filter(|(_, id)| **id != u32::MAX) {
        let Some(r) = found.get_mut(id as usize) else {
            continue;
        };
        r.bounds.add((i % wu) as u32, (i / wu) as u32);
        r.px += 1;
        let v = hz.get(i).copied().unwrap_or(0.0);
        match r.rates.iter_mut().find(|(h, _)| (h - v).abs() < 1e-6) {
            Some((_, n)) => *n += 1,
            None => r.rates.push((v, 1)),
        }
        r.phases.push(phase.get(i).copied().unwrap_or_default());
    }
    let mut out: Vec<Region> = found
        .into_iter()
        .filter_map(|g| Some((g.bounds.rect()?, g)))
        .map(|(rect, g)| Region {
            rect,
            px: g.px,
            hz: g.rates.iter().map(|r| r.0).fold(0.0, f64::max),
            common_hz: g
                .rates
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1).then(b.0.total_cmp(&a.0)))
                .map_or(0.0, |r| r.0),
            phase: phase_counts(g.phases.into_iter()).first().map(|p| p.0),
        })
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(r.px));
    out
}

/// The most `true` pixels of `mask` inside any disk of `diameter` px, searched over centres
/// round `bbox`, as `(count, cx, cy)`.
pub fn max_in_disk(mask: &[bool], w: u32, h: u32, diameter: f64, bbox: Rect) -> (u64, i64, i64) {
    let (wi, hi) = (i64::from(w), i64::from(h));
    let r = (diameter * 0.5).max(0.5);
    let ri = r.floor() as i64;
    // Row prefix sums: pre[y * (w + 1) + x] = trues in row y before column x.
    let stride = w as usize + 1;
    let mut pre = vec![0u32; stride * h as usize];
    for y in 0..h as usize {
        for x in 0..w as usize {
            let v = u32::from(mask.get(y * w as usize + x).copied().unwrap_or(false));
            pre[y * stride + x + 1] = pre[y * stride + x] + v;
        }
    }
    let half: Vec<i64> = (-ri..=ri)
        .map(|dy| (r * r - (dy * dy) as f64).max(0.0).sqrt().floor() as i64)
        .collect();
    let mut best = (0u64, bbox.x, bbox.y);
    let (cx0, cx1) = ((bbox.x - ri).max(0), (bbox.right() + ri).min(wi));
    let (cy0, cy1) = ((bbox.y - ri).max(0), (bbox.bottom() + ri).min(hi));
    for cy in cy0..cy1 {
        for cx in cx0..cx1 {
            let mut n = 0u64;
            for (j, hw) in half.iter().enumerate() {
                let y = cy + j as i64 - ri;
                if y < 0 || y >= hi {
                    continue;
                }
                let (a, b) = ((cx - hw).max(0), (cx + hw + 1).min(wi));
                if b > a {
                    let row = y as usize * stride;
                    n += u64::from(pre[row + b as usize] - pre[row + a as usize]);
                }
            }
            if n > best.0 {
                best = (n, cx, cy);
            }
        }
    }
    best
}

/// The audit's findings.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// Frames read.
    pub frames: u16,
    /// Frame width.
    pub width: u32,
    /// Frame height.
    pub height: u32,
    /// Frames per second.
    pub fps: u16,
    /// The 10-degree field's diameter in frame pixels.
    pub field_px: f64,
    /// 25% of its area.
    pub limit_px2: f64,
    /// General flashes.
    pub general: Layer,
    /// Red flashes.
    pub red: Layer,
    /// Flashing by either threshold: what the area and height verdicts use.
    pub union: Layer,
    /// More than three flashes inside one second, by either threshold.
    pub count: Layer,
    /// The slowest rate the sustained test can see: `fps / (frames - 2)`.
    pub floor_hz: f64,
    /// The ranges of bright and dark phase lengths of the flashing pixels in frames, most
    /// common first, with how many pixels show each.
    pub phases: Vec<(Phases, u64)>,
    /// The flashing areas, largest first.
    pub regions: Vec<Region>,
}

/// Compares file names the way people count: `frame_9` before `frame_10`.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    fn digits(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
        let mut s = String::new();
        while let Some(&c) = it.peek() {
            if !c.is_ascii_digit() {
                break;
            }
            s.push(c);
            it.next();
        }
        s
    }
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let (m, n) = (digits(&mut x), digits(&mut y));
                let (mt, nt) = (m.trim_start_matches('0'), n.trim_start_matches('0'));
                let o = mt
                    .len()
                    .cmp(&nt.len())
                    .then_with(|| mt.cmp(nt))
                    .then_with(|| m.len().cmp(&n.len()));
                if o != Ordering::Equal {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                if c != d {
                    return c.cmp(&d);
                }
                x.next();
                y.next();
            }
        }
    }
}

/// The file name of `p`, for sorting and printing.
pub fn file_name(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The sequence's frames: every PNG in a folder (in counting order), or the files given.
///
/// # Errors
/// A file error when the folder cannot be read or holds no PNG, or a file is missing.
pub fn frame_paths(inputs: &[String]) -> Result<Vec<PathBuf>> {
    if let [one] = inputs {
        let p = Path::new(one);
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(p)
                .map_err(|e| file_err(p, e))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("png")))
                .collect();
            if v.is_empty() {
                return Err(file_err(p, "holds no PNG frames"));
            }
            v.sort_by(|a, b| natural_cmp(&file_name(a), &file_name(b)));
            return Ok(v);
        }
    }
    for i in inputs {
        let p = Path::new(i);
        if !p.is_file() {
            return Err(file_err(
                p,
                "not found: give the sequence folder (mockups/<Name>/sequence) or the frame files",
            ));
        }
    }
    Ok(inputs.iter().map(PathBuf::from).collect())
}

fn yes(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

/// `A-B`: an inclusive range of frame indices.
///
/// # Errors
/// A usage error for anything else, or a range outside `len` frames.
pub fn frame_range(s: &str, len: usize) -> Result<(usize, usize)> {
    let bad = || {
        usage(format!(
            "--frames {s:?}: expected FIRST-LAST, two frame indices below {len}"
        ))
    };
    let (a, b) = s.split_once('-').ok_or_else(bad)?;
    let a: usize = a.trim().parse().map_err(|_| bad())?;
    let b: usize = b.trim().parse().map_err(|_| bad())?;
    if a >= b || b >= len {
        return Err(bad());
    }
    Ok((a, b))
}

/// The review sequence's limiter hold (`fh6_clusters::review::sequence_frame`: frames 30-41).
const LIMITER_HOLD: &str = "30-41";

/// The fewest frames that let the sustained test see every rate above 3 Hz at `fps`.
fn frames_for_3hz(fps: u16) -> u32 {
    // The slowest failing cycle is the longest whole number of frames under fps / 3.
    let slowest = (f64::from(fps) / MAX_HZ).ceil() as u32 - 1;
    slowest.max(1) + 2
}

/// `flash`: audits the sequence and writes the flash map.
///
/// # Errors
/// Usage and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--fps", "--eye-m", "--scale", "--frames", "--out"])?;
    let mut paths = frame_paths(args.pos())?;
    let range = args.one("--frames");
    if let Some(r) = range {
        let (a, b) = frame_range(r, paths.len())?;
        paths = paths.get(a..=b).map(<[PathBuf]>::to_vec).unwrap_or_default();
    } else if paths.len() == 60 {
        out!(
            "note: this looks like the 60-frame review sequence; speed, rpm and gear change in frames 10-29 and 42-59, so counting digits show up as flashing there. For the strobe test pass --frames {LIMITER_HOLD} (the limiter hold)."
        );
    }
    let (Some(first_path), Some(last_path)) = (paths.first(), paths.last()) else {
        return Err(usage("flash needs a sequence of frames"));
    };
    if paths.len() < 3 {
        return Err(usage(
            "flash needs a sequence: a folder of frames (mockups/<Name>/sequence) or at least three frame files",
        ));
    }
    let fps = args.num("--fps", 60u16)?;
    let eye_m = args.num("--eye-m", 0.6f64)?;
    let scale = args.num("--scale", 1.0f64)?;
    if !(0.2..=3.0).contains(&eye_m) || !(0.5..=2.0).contains(&scale) || !(1..=MAX_FPS).contains(&fps) {
        return Err(usage(format!(
            "--eye-m must be 0.2 to 3.0, --scale 0.5 to 2.0 and --fps 1 to {MAX_FPS}"
        )));
    }
    if paths.len() > usize::from(u16::MAX) {
        return Err(usage("flash reads at most 65535 frames"));
    }
    out!(
        "frames: {} .. {} ({} files)",
        file_name(first_path),
        file_name(last_path),
        paths.len()
    );
    // Only the first frame says how transparency is read.
    let first = ctx.load(first_path)?;
    let mut audit = Audit::new(&first, fps);
    for p in paths.iter().skip(1) {
        audit.feed(&Frame::load(p, ctx.straight)?)?;
    }
    let rep = audit.report(eye_m, scale);
    // Positions print in 4K frame pixels when the frames' place in the frame is known.
    let origin = ctx.origin(&first);
    let place = |r: Rect| match origin {
        Some((ox, oy)) => format!("4K {}", r.offset(ox, oy)),
        None => format!("{r} (the frames' own pixels)"),
    };
    if origin.is_none() {
        out!(
            "note: {}x{} is neither a full frame nor a preview-export crop, so positions are in the frames' own pixels",
            first.width,
            first.height
        );
    }
    let strip = ctx.tokens.strip_max_px;
    out!(
        "flash audit: {} frames of {}x{} at {} fps; eye {eye_m:.2} m, cluster scale {scale:.2}; a 10-degree field is {:.0} px across, 25% of it is {:.0} px2",
        rep.frames, rep.width, rep.height, rep.fps, rep.field_px, rep.limit_px2
    );
    for (name, l) in [("general", &rep.general), ("red", &rep.red)] {
        let rates: Vec<String> = l
            .rates
            .iter()
            .take(3)
            .map(|(hz, n)| format!("{hz:.1} Hz in {n} px"))
            .collect();
        out!(
            "{name} flashes faster than 3 Hz: {} px{}{}",
            l.px,
            l.bbox.map_or(String::new(), |b| format!(" within {}", place(b))),
            if rates.is_empty() {
                String::new()
            } else {
                format!(" ({})", rates.join(", "))
            }
        );
    }
    let phases: Vec<String> = rep
        .phases
        .iter()
        .take(3)
        .map(|(p, n)| {
            let ms = |f: u16| f64::from(f) * 1000.0 / f64::from(rep.fps);
            if p.on.0 == p.on.1 && p.off.0 == p.off.1 {
                format!("{p} ({:.0} ms / {:.0} ms) in {n} px", ms(p.on.0), ms(p.off.0))
            } else {
                format!("{p} (uneven) in {n} px")
            }
        })
        .collect();
    if !phases.is_empty() {
        out!(
            "phases of the flashing pixels, in frames (a range means the phases were uneven): {}",
            phases.join("; ")
        );
    }
    let area_text = |r: &Region| {
        format!(
            "{} px within {}, most at {:.1} Hz{}, fastest {:.1} Hz",
            r.px,
            place(r.rect),
            r.common_hz,
            r.phase.map(|p| format!(" ({p})")).unwrap_or_default(),
            r.hz
        )
    };
    // Every area is listed; the largest are boxed on the map with the same numbers.
    for (i, r) in rep.regions.iter().enumerate() {
        out!("  area {}: {}", i + 1, area_text(r));
    }
    if rep.regions.len() > AREAS_BOXED {
        out!("  (areas 1 to {AREAS_BOXED} are boxed on the map; the rest are listed only)");
    }
    let area_ok = (rep.union.worst.0 as f64) <= rep.limit_px2;
    out!(
        "flashing faster than 3 Hz (either threshold) covers at most 25% of any 10-degree field: {} (worst field holds {} px2 of {:.0})",
        yes(area_ok),
        rep.union.worst.0,
        rep.limit_px2
    );
    let tallest = rep.union.tallest;
    let strip_ok = f64::from(tallest) <= strip;
    out!(
        "flashing faster than 3 Hz is no taller than the shift strip ({strip:.0} px at scale 1.0): {} (tallest {tallest} px, gaps up to {GAP_CLOSE} px closed)",
        yes(strip_ok)
    );
    let need = frames_for_3hz(rep.fps);
    let slow_seen = u32::from(rep.frames) >= need;
    if slow_seen {
        out!("rates seen: every rate above 3 Hz");
    } else {
        out!(
            "rates seen: {:.1} Hz and faster. Rates from 3 to {:.1} Hz: NOT ASSESSED (a cycle that slow needs {need} frames; this sequence has {})",
            rep.floor_hz,
            rep.floor_hz,
            rep.frames
        );
    }
    let count_assessed = u32::from(rep.frames) >= u32::from(rep.fps);
    let count_ok = (rep.count.worst.0 as f64) <= rep.limit_px2;
    if count_assessed {
        out!(
            "more than three flashes inside one second (WCAG count) on at most 25% of any 10-degree field: {} ({} px flash that often; worst field holds {} px2)",
            yes(count_ok),
            rep.count.px,
            rep.count.worst.0
        );
    } else {
        out!(
            "more than three flashes inside one second (WCAG count): NOT ASSESSED ({} frames is under one second at {} fps; {} px already flash more than three times within these {} frames)",
            rep.frames,
            rep.fps,
            rep.count.px,
            rep.frames
        );
    }
    let pass = area_ok && strip_ok && (!count_assessed || count_ok);
    if slow_seen && count_assessed {
        out!("flash audit: {}", yes(pass));
    } else {
        out!(
            "flash audit of what this sequence can show: {}; the parts marked NOT ASSESSED are not answered",
            yes(pass)
        );
    }

    let mut map = audit.brightest().dimmed(0.4);
    for (i, px) in map.rgb.chunks_exact_mut(3).enumerate() {
        let red = rep.red.mask.get(i).copied().unwrap_or(false);
        let general = rep.general.mask.get(i).copied().unwrap_or(false);
        if red {
            px.copy_from_slice(&MAGENTA);
        } else if general {
            px.copy_from_slice(&YELLOW);
        }
    }
    // The listed areas first, so mark k on the map is area k in the text; the ring last.
    let mut marks: Vec<Mark> = rep
        .regions
        .iter()
        .take(AREAS_BOXED)
        .enumerate()
        .map(|(i, r)| Mark::Box {
            rect: r.rect.grow(4),
            label: format!("area {}: {}", i + 1, area_text(r)),
            color: CYAN,
        })
        .collect();
    if rep.union.px > 0 {
        marks.push(Mark::Ring {
            cx: rep.union.worst.1 as f64,
            cy: rep.union.worst.2 as f64,
            r: rep.field_px * 0.5,
            label: format!(
                "worst 10-degree field: {} of {:.0} px2 allowed",
                rep.union.worst.0, rep.limit_px2
            ),
            color: WHITE,
        });
    }
    marks.push(Mark::Caption(if marks.is_empty() {
        "no pixel flashes faster than 3 Hz".to_string()
    } else {
        format!("flashing faster than 3 Hz: yellow general, magenta red; eye {eye_m:.2} m, scale {scale:.2}")
    }));
    let map = Painter::new(&ctx.root).annotated(&map, &marks, Map::IDENTITY)?;
    let name = match range {
        Some(r) => format!("flash_map_{}.png", r.replace(' ', "")),
        None => "flash_map.png".to_string(),
    };
    let dest = cli::out_named(args, ctx, first_path, &name);
    map.save(&dest)?;
    out!("flash map: {}", dest.display());
    if let Some(note) = cli::view_note(map.width, map.height) {
        out!("{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 60 frames of `w` x `h`; `lit(frame, x, y)` gives each pixel's colour.
    fn sequence(w: u32, h: u32, lit: impl Fn(u16, u32, u32) -> [u8; 3]) -> Vec<Frame> {
        (0..60u16)
            .map(|f| {
                let mut fr = Frame::filled(w, h, [0; 3]);
                for y in 0..h {
                    for x in 0..w {
                        fr.put(x, y, lit(f, x, y));
                    }
                }
                fr
            })
            .collect()
    }

    fn audit(frames: &[Frame]) -> Audit {
        let mut a = Audit::new(&frames[0], 60);
        for f in &frames[1..] {
            a.feed(f).expect("same size");
        }
        a
    }

    /// The limiter strobe of the review sequence: frames 30-41, 3 on / 3 off (10 Hz).
    fn strobe(f: u16) -> bool {
        (30..42).contains(&f) && ((f - 30) / 3).is_multiple_of(2)
    }

    #[test]
    fn a_thin_strip_strobing_at_10_hz_is_found_and_passes() {
        let frames = sequence(600, 120, |f, x, y| {
            if (40..80).contains(&y) && (50..550).contains(&x) && strobe(f) {
                [255, 255, 255]
            } else {
                [0, 0, 0]
            }
        });
        let rep = audit(&frames).report(0.6, 1.0);
        assert_eq!(rep.general.px, 40 * 500);
        assert_eq!(rep.general.tallest, 40);
        assert_eq!(rep.general.bbox, Some(Rect::new(50, 40, 500, 40)));
        assert!(rep
            .general
            .rates
            .first()
            .is_some_and(|(hz, _)| (hz - 10.0).abs() < 1e-6));
        // A 40 px strip covers at most 331 x 40 px of a 10-degree field at 0.6 m.
        assert!(rep.union.worst.0 as f64 <= rep.limit_px2);
        assert!(rep.union.worst.0 >= 40 * 320);
        // Two cycles in 0.2 s is not more than three flashes in a second.
        assert_eq!(rep.general.burst, 0);
        // White is not red.
        assert_eq!(rep.red.px, 0);
        // 3 frames on, 3 off, in one area.
        let even = Phases {
            on: (3, 3),
            off: (3, 3),
        };
        assert_eq!(rep.phases.first().map(|p| p.0), Some(even));
        assert_eq!(rep.regions.len(), 1);
        assert_eq!(rep.regions[0].rect, Rect::new(50, 40, 500, 40));
        assert_eq!(rep.regions[0].phase, Some(even));
        assert!((rep.regions[0].common_hz - 10.0).abs() < 1e-9);
        assert_eq!(even.to_string(), "3 on / 3 off");
    }

    #[test]
    fn an_uneven_strobe_shows_its_phase_ranges() {
        // On 30-32, off 33-35, on 36-37, off 38-41, on 42-44: phases of 3, 2 and 3 on, 3 and 4 off.
        let lit = |f: u16| (30..33).contains(&f) || (36..38).contains(&f) || (42..45).contains(&f);
        let frames = sequence(8, 8, |f, _, _| if lit(f) { [255; 3] } else { [0; 3] });
        let rep = audit(&frames).report(0.6, 1.0);
        let p = rep.regions.first().and_then(|r| r.phase).expect("a flashing area");
        assert_eq!(
            p,
            Phases {
                on: (2, 3),
                off: (3, 4)
            }
        );
        assert_eq!(p.to_string(), "2-3 on / 3-4 off");
    }

    #[test]
    fn the_limiter_hold_alone_sees_the_strobe_and_says_what_it_cannot_see() {
        let frames = sequence(40, 20, |f, _, y| if y < 10 && strobe(f) { [255; 3] } else { [0; 3] });
        let hold = &frames[30..42];
        let rep = audit(hold).report(0.6, 1.0);
        assert_eq!(rep.frames, 12);
        assert_eq!(rep.union.px, 400);
        // 12 frames see cycles of up to 10 frames: 6 Hz and faster.
        assert!((rep.floor_hz - 6.0).abs() < 1e-9);
        assert!(u32::from(rep.frames) < frames_for_3hz(60));
        assert_eq!(frames_for_3hz(60), 21);
    }

    #[test]
    fn stacked_bars_with_small_gaps_count_as_one_block() {
        // Three 10 px bars with 4 px gaps: 38 px tall once the gaps are closed.
        let frames = sequence(60, 60, |f, _, y| {
            let bar = (5..15).contains(&y) || (19..29).contains(&y) || (33..43).contains(&y);
            if bar && strobe(f) {
                [255; 3]
            } else {
                [0; 3]
            }
        });
        let rep = audit(&frames).report(0.6, 1.0);
        assert_eq!(rep.union.tallest, 38);
        assert_eq!(rep.regions.len(), 1);
    }

    #[test]
    fn a_large_flashing_block_fails_the_area_test() {
        let frames = sequence(500, 400, |f, x, y| {
            if (50..450).contains(&x) && (50..350).contains(&y) && strobe(f) {
                [200, 200, 200]
            } else {
                [0, 0, 0]
            }
        });
        let rep = audit(&frames).report(0.6, 1.0);
        assert!(rep.union.worst.0 as f64 > rep.limit_px2);
        assert_eq!(rep.union.tallest, 300);
    }

    #[test]
    fn red_flashes_are_tracked_apart_from_luminance() {
        let frames = sequence(
            80,
            60,
            |f, x, _| if x < 40 && strobe(f) { [255, 0, 0] } else { [0, 0, 0] },
        );
        let rep = audit(&frames).report(0.6, 1.0);
        assert_eq!(rep.red.px, 40 * 60);
        // Pure red is also a luminance flash (0.2126 is above the 0.1 step).
        assert_eq!(rep.general.px, 40 * 60);
        assert_eq!(rep.union.px, 40 * 60);
    }

    #[test]
    fn a_slow_fade_and_a_single_blink_are_not_flashing() {
        let frames = sequence(10, 10, |f, x, _| {
            if x < 5 {
                let v = (f32::from(f) / 59.0 * 255.0) as u8;
                [v, v, v]
            } else if (42..44).contains(&f) {
                [255, 255, 255]
            } else {
                [0, 0, 0]
            }
        });
        let rep = audit(&frames).report(0.6, 1.0);
        assert_eq!(rep.general.px, 0, "rates {:?}", rep.general.rates);
        assert!(rep.regions.is_empty());
    }

    #[test]
    fn a_second_of_strobing_trips_the_wcag_count() {
        let frames = sequence(4, 4, |f, _, _| {
            if (f / 3).is_multiple_of(2) {
                [255, 255, 255]
            } else {
                [0, 0, 0]
            }
        });
        let rep = audit(&frames).report(0.6, 1.0);
        assert_eq!(rep.general.burst, 16);
        assert_eq!(rep.general.px, 16);
        assert_eq!(rep.count.px, 16);
    }

    #[test]
    fn frame_ranges_parse_and_names_sort_by_number() {
        assert_eq!(frame_range("30-41", 60).ok(), Some((30, 41)));
        for bad in ["41-30", "30-60", "30", "a-b", "5-5"] {
            assert!(frame_range(bad, 60).is_err(), "{bad:?} should be rejected");
        }
        let mut names = vec!["frame_10.png", "frame_9.png", "frame_02.png", "frame_1.png"];
        names.sort_by(|a, b| natural_cmp(a, b));
        assert_eq!(names, ["frame_1.png", "frame_02.png", "frame_9.png", "frame_10.png"]);
        assert!(frame_paths(&["no/such/folder".to_string()]).is_err());
    }

    #[test]
    fn the_disk_search_counts_exactly() {
        let (w, h) = (100u32, 100u32);
        let mut mask = vec![false; (w * h) as usize];
        for y in 40..60 {
            for x in 30..70 {
                mask[(y * w + x) as usize] = true;
            }
        }
        // A disk wide enough to hold the whole 40x20 block.
        let (n, _, _) = max_in_disk(&mask, w, h, 200.0, Rect::new(30, 40, 40, 20));
        assert_eq!(n, 800);
        // A small disk holds at most its own area.
        let (m, _, _) = max_in_disk(&mask, w, h, 10.0, Rect::new(30, 40, 40, 20));
        assert!(m > 60 && m <= 81, "{m}");
    }
}
