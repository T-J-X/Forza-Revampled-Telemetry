//! `motion`: how much the picture changes from one frame to the next across a sequence. A pull
//! that changes nothing for a frame is a stall (the dash lagging its data); a single frame that
//! changes far more than its neighbours is a spike to check (an intended stage change, or a
//! glitch); an idle that keeps changing is not still.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::diff::{distance, lab_field};
use crate::error::{usage, Result};
use crate::flash::{file_name, frame_paths, frame_range};
use crate::frame::Frame;
use crate::geom::Rect;
use crate::overlay::{Map, Mark, Painter, CYAN, MAGENTA, WHITE, YELLOW};
use fh6_kit::color::Oklab;
use std::path::PathBuf;

/// A pixel counts as changed above this OKLab distance (x100): the golden tolerance.
const CHANGED: f32 = 3.0;
/// A frame changes "far more than its neighbours" (a spike) above this multiple of their median.
const POP: f32 = 3.0;

/// The segments of the review sequence (`fh6_clusters::review::sequence_frame`).
const SEGMENTS: [(u32, u32, &str); 5] = [
    (0, 9, "idle"),
    (10, 29, "pull in 3rd"),
    (30, 41, "limiter hold"),
    (42, 44, "upshift"),
    (45, 59, "pull in 4th"),
];

/// The change from the previous frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    /// Share of pixels that changed.
    pub changed: f32,
    /// Mean distance over all pixels.
    pub mean: f32,
}

/// The change from `p` to `cur` (OKLab fields of the same size).
fn step(p: &[Oklab], cur: &[Oklab]) -> Step {
    let (mut changed, mut sum) = (0u64, 0.0f64);
    for (a, b) in p.iter().zip(cur) {
        let d = distance(*a, *b);
        sum += f64::from(d);
        if d > CHANGED {
            changed += 1;
        }
    }
    let n = cur.len().max(1) as f64;
    Step {
        changed: (changed as f64 / n) as f32,
        mean: (sum / n) as f32,
    }
}

/// The change between each frame and the one before it (`run` streams the same steps).
#[cfg(test)]
pub fn profile(frames: &[Frame]) -> Vec<Step> {
    let mut out = Vec::with_capacity(frames.len().saturating_sub(1));
    let mut prev = frames.first().map(lab_field);
    for f in frames.iter().skip(1) {
        let cur = lab_field(f);
        if let Some(p) = &prev {
            out.push(step(p, &cur));
        }
        prev = Some(cur);
    }
    out
}

fn segment(i: u32, review: bool) -> &'static str {
    if !review {
        return "";
    }
    SEGMENTS
        .iter()
        .find(|(a, b, _)| (*a..=*b).contains(&i))
        .map_or("", |s| s.2)
}

/// The median of `v`.
fn median(mut v: Vec<f32>) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f32::total_cmp);
    v[v.len() / 2]
}

/// `motion`: prints the change per frame, flags stalls and pops, and draws the profile.
///
/// # Errors
/// Usage, image and overlay errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--frames", "--out"])?;
    let mut paths = frame_paths(args.pos())?;
    // The review sequence's segments are named by frame number, also inside a --frames range.
    let review = paths.len() == 60;
    let range = args.one("--frames");
    let first = if let Some(r) = range {
        let (a, b) = frame_range(r, paths.len())?;
        paths = paths.get(a..=b).map(<[PathBuf]>::to_vec).unwrap_or_default();
        a
    } else {
        0
    };
    let (Some(first_path), Some(last_path)) = (paths.first(), paths.last()) else {
        return Err(usage("motion needs a sequence of at least two frames"));
    };
    if paths.len() < 2 {
        return Err(usage("motion needs a sequence of at least two frames"));
    }
    out!(
        "frames: {} .. {} ({} files)",
        file_name(first_path),
        file_name(last_path),
        paths.len()
    );
    // One frame in memory at a time; only the first says how transparency is read.
    let head = ctx.load(first_path)?;
    let size = (head.width, head.height);
    let mut prev = lab_field(&head);
    let mut steps = Vec::with_capacity(paths.len() - 1);
    for p in paths.iter().skip(1) {
        let f = Frame::load(p, ctx.straight)?;
        if (f.width, f.height) != size {
            return Err(usage(format!(
                "{} is {}x{}, the first frame was {}x{}: every frame of the sequence must have the same size",
                p.display(),
                f.width,
                f.height,
                size.0,
                size.1
            )));
        }
        let cur = lab_field(&f);
        steps.push(step(&prev, &cur));
        prev = cur;
    }
    out!(
        "motion over {} frames (change from the previous frame; a pixel changes above dE {CHANGED})",
        paths.len()
    );
    let mut flags = Vec::new();
    for (k, s) in steps.iter().enumerate() {
        let i = (first + k + 1) as u32;
        let seg = segment(i, review);
        let lo = k.saturating_sub(3);
        let hi = (k + 4).min(steps.len());
        let around: Vec<f32> = steps[lo..hi]
            .iter()
            .enumerate()
            .filter(|(j, _)| lo + j != k)
            .map(|(_, s)| s.changed)
            .collect();
        // Without neighbours (a two-frame range) nothing can stand out from them.
        let has_neighbours = !around.is_empty();
        let med = median(around);
        let mut note = String::new();
        // A frame that changes while its neighbours are still is the plainest glitch of all.
        let spike = has_neighbours && s.changed > 0.002 && (med == 0.0 || s.changed > med * POP);
        if seg.starts_with("pull") && s.changed == 0.0 {
            note = "  <- stall: nothing moved while the data changed".to_string();
            flags.push((i, "stall"));
        } else if spike && seg == "limiter hold" {
            note = "  (strobe edge)".to_string();
        } else if spike && seg == "upshift" {
            note = "  (gear change)".to_string();
        } else if spike {
            let size = if med == 0.0 {
                "changes while its neighbours are still".to_string()
            } else {
                format!("{:.1}x its neighbours", s.changed / med)
            };
            note = format!("  <- spike: {size}; check it is an intended event (a shift stage) and not a glitch");
            flags.push((i, "spike"));
        }
        out!(
            "  frame {i:>2}{}: {:>6.2}% changed, mean dE {:.2}{note}",
            if seg.is_empty() {
                String::new()
            } else {
                format!(" ({seg})")
            },
            s.changed * 100.0,
            s.mean
        );
    }
    if review && first == 0 && steps.len() >= 9 {
        // Steps 5..9 are the changes into frames 6 to 9.
        let still = steps.iter().skip(5).take(4).all(|s| s.changed < 0.001);
        out!(
            "idle frames 6-9 are still (under 0.1% changed each): {}",
            if still {
                "yes"
            } else {
                "no: something still moves at idle. The sequence starts from a fresh dash, so this can be a start-up sweep or a spring settling; look at frames 5-9 to say which"
            }
        );
    } else if review {
        let why = if first != 0 {
            "the range does not start at frame 0"
        } else {
            "the range ends before frame 9"
        };
        out!("idle stillness: NOT ASSESSED ({why})");
    }
    out!(
        "stalls: {}, spikes to check: {}",
        flags.iter().filter(|f| f.1 == "stall").count(),
        flags.iter().filter(|f| f.1 == "spike").count()
    );

    // The profile as a bar chart.
    let (w, h, base) = (1500u32, 420u32, 360u32);
    let mut chart = Frame::filled(w, h, [12, 14, 18]);
    let n = steps.len().max(1) as u32;
    let bw = ((w - 80) / n).max(2);
    let top = steps.iter().map(|s| s.changed).fold(0.0f32, f32::max).max(0.01);
    let mut marks = vec![Mark::Text {
        x: 40.0,
        y: 34.0,
        size: 22.0,
        text: format!("share of pixels changed per frame (top {:.1}%)", top * 100.0),
        color: WHITE,
    }];
    for (k, s) in steps.iter().enumerate() {
        let i = (first + k + 1) as u32;
        let bh = ((s.changed / top) * (base - 60) as f32).round().max(1.0) as i64;
        let x = 40 + i64::from(k as u32 * bw);
        let flagged = flags.iter().any(|f| f.0 == i);
        let color = if flagged {
            MAGENTA
        } else if segment(i, review) == "limiter hold" {
            YELLOW
        } else {
            CYAN
        };
        chart.fill(
            Rect::new(x, i64::from(base) - bh, i64::from(bw.saturating_sub(2).max(1)), bh),
            color,
        );
        if k % 5 == 4 || k == 0 {
            marks.push(Mark::Text {
                x: x as f64,
                y: f64::from(base) + 30.0,
                size: 18.0,
                text: i.to_string(),
                color: WHITE,
            });
        }
    }
    Painter::new(&ctx.root).draw(&mut chart, &marks, Map::IDENTITY)?;
    let name = match range {
        Some(r) => format!("motion_profile_{}.png", r.replace(' ', "")),
        None => "motion_profile.png".to_string(),
    };
    let p = cli::out_named(args, ctx, first_path, &name);
    chart.save(&p)?;
    out!("profile: {} (magenta: flagged; yellow: limiter hold)", p.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_moving_bar_changes_every_frame_and_a_still_one_does_not() {
        let frames: Vec<Frame> = (0..6)
            .map(|i| {
                let mut f = Frame::filled(40, 10, [0; 3]);
                let x = if i < 3 { 5 + i * 5 } else { 15 };
                f.fill(Rect::new(x, 2, 4, 6), [255, 255, 255]);
                f
            })
            .collect();
        let p = profile(&frames);
        assert_eq!(p.len(), 5);
        assert!(p[0].changed > 0.0 && p[1].changed > 0.0);
        assert_eq!(p[3].changed, 0.0);
        assert_eq!(p[4].changed, 0.0);
        assert_eq!(segment(35, true), "limiter hold");
        assert_eq!(segment(35, false), "");
        assert_eq!(median(vec![3.0, 1.0, 2.0]), 2.0);
    }
}
