//! `blind`: frames for a blind glance read. It draws frames at random (seeded, never
//! hand-picked) from a dash's 60-frame review sequence and its four state renders, puts each
//! back into a full black frame, scales it to 1430x804 (inside the 1.15 megapixels every Claude
//! model sees 1:1) and saves it under a neutral name in `mockups/blind/<seed>/`, a folder that
//! does not name the dash. The answer key, the review telemetry of each frame, goes into the
//! dash's own review folder, where the blind reader is never pointed.
//!
//! The key restates `fh6_clusters::review` (the four states and `sequence_frame`) so the tool
//! builds even while a dash module is mid-edit in a shared checkout; a test proves every value
//! matches the cluster crate. `key` prints it for every frame.
use crate::args::Args;
use crate::cli::{self, Ctx};
use crate::error::{file_err, usage, Result};
use crate::frame::Frame;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// The size the blind reader sees: within every model's 1:1 view (1568 px a side, 1.15 MP).
const BLIND_SIZE: (u32, u32) = (1430, 804);
/// Frames in the review sequence.
const SEQUENCE_FRAMES: u32 = 60;
/// Neutral in the telemetry gear byte.
const NEUTRAL: u8 = 11;
/// The limiter of every review state and frame (`EngineMaxRpm`).
const LIMITER_RPM: f32 = 8700.0;
/// The review states: file stem, gear, mph and rpm (`fh6_clusters::review::STATES`).
const STATES: [(&str, u8, f32, f32); 4] = [
    ("01_idle", NEUTRAL, 0.0, 900.0),
    ("02_mid_cruise", 4, 118.0, 5200.0),
    ("03_redline_strobe", 5, 152.0, 8680.0),
    ("04_drift_braking", 3, 74.0, 7100.0),
];

/// Gear, mph and rpm of frame `i` of the review sequence (`fh6_clusters::review::sequence_frame`).
fn sequence_key(i: u32) -> (u8, f32, f32) {
    let i = i.min(SEQUENCE_FRAMES - 1) as f32;
    let smooth = |a: f32, b: f32, t: f32| {
        let t = t.clamp(0.0, 1.0);
        a + (b - a) * t * t * (3.0 - 2.0 * t)
    };
    if i < 10.0 {
        (NEUTRAL, 0.0, 900.0)
    } else if i < 30.0 {
        let t = (i - 10.0) / 19.0;
        (3, smooth(38.0, 92.0, t), smooth(2600.0, 8680.0, t))
    } else if i < 42.0 {
        let bounce = if (i as u32).is_multiple_of(2) { 8690.0 } else { 8640.0 };
        (3, 92.5, bounce)
    } else if i < 45.0 {
        (4, 93.0, smooth(8600.0, 6420.0, (i - 42.0) / 2.0))
    } else {
        let t = (i - 45.0) / 14.0;
        (4, smooth(93.0, 103.0, t), smooth(6420.0, 7350.0, t))
    }
}

/// One item that can be drawn: a sequence frame or a state render.
#[derive(Debug, Clone, PartialEq)]
struct Item {
    path: PathBuf,
    what: String,
    gear: u8,
    mph: f32,
    rpm: f32,
    /// The sequence crop's origin in the full frame (`None` for a full-frame state render).
    origin: Option<(i64, i64)>,
}

fn gear_name(g: u8) -> String {
    match g {
        0 => "R".to_string(),
        NEUTRAL => "N".to_string(),
        n => n.to_string(),
    }
}

fn segment(i: u32) -> &'static str {
    match i {
        0..=9 => "idle",
        10..=29 => "pull in 3rd",
        30..=41 => "limiter hold",
        42..=44 => "upshift",
        _ => "pull in 4th",
    }
}

/// The rpm zones the blind reader names, by share of the limiter: below 50% low, below 80% mid,
/// below 97% high, else the limiter.
const ZONES: [(f32, &str); 3] = [(0.50, "low"), (0.80, "mid"), (0.97, "high")];

/// The rpm zone of `rpm`.
fn zone(rpm: f32) -> &'static str {
    let share = rpm / LIMITER_RPM;
    ZONES
        .iter()
        .find(|(limit, _)| share < *limit)
        .map_or("limiter", |z| z.1)
}

/// The question the blind reader answers for each frame.
const READER_QUESTION: &str = "for each frame, after one look and without looking again: the gear, the speed, the shift lights (dark, partly lit or all lit) and the rpm zone (low below 50% of the limiter, mid below 80%, high below 97%, limiter above)";

/// One line of the key: gear, speed, rpm, its share of the limiter and its zone.
fn key_line(what: &str, gear: u8, mph: f32, rpm: f32) -> String {
    format!(
        "{what}: gear {}, {mph:.1} mph, {rpm:.0} rpm ({:.0}% of the {LIMITER_RPM:.0} rpm limiter, zone {})",
        gear_name(gear),
        rpm / LIMITER_RPM * 100.0,
        zone(rpm)
    )
}

/// Picks `count` distinct indices below `n` with a seeded LCG.
pub fn draw(n: usize, count: usize, seed: u64) -> Vec<usize> {
    let mut s = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut out = Vec::with_capacity(count.min(n));
    while out.len() < count.min(n) {
        s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let i = ((s >> 33) % n as u64) as usize;
        if !out.contains(&i) {
            out.push(i);
        }
    }
    out
}

/// `path` made absolute against the working folder, for comparing places.
fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// `key`: the review telemetry of every sequence frame and state (the answer key).
///
/// # Errors
/// A usage error for any argument.
pub fn key(args: &Args) -> Result<()> {
    args.known(&[])?;
    if !args.pos().is_empty() {
        return Err(usage("key takes no arguments"));
    }
    out!("review telemetry (fh6_clusters::review), limiter {LIMITER_RPM:.0} rpm:");
    for i in 0..SEQUENCE_FRAMES {
        let (gear, mph, rpm) = sequence_key(i);
        out!(
            "  {}",
            key_line(&format!("frame_{i:02} ({})", segment(i)), gear, mph, rpm)
        );
    }
    for (name, gear, mph, rpm) in STATES {
        out!("  {}", key_line(&format!("state {name}"), gear, mph, rpm));
    }
    Ok(())
}

/// `blind`: writes the neutral-named frames and the answer key.
///
/// # Errors
/// Usage, file and image errors.
pub fn run(args: &Args, ctx: &Ctx) -> Result<()> {
    args.known(&["--count", "--seed", "--out", "--key", "--mask", "--anchor"])?;
    let [dir] = args.pos() else {
        return Err(usage("blind needs the dash's mockups folder, e.g. mockups/LiquidGlass"));
    };
    let dir = Path::new(dir);
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .ok_or_else(|| usage("blind needs the dash's mockups folder, e.g. mockups/LiquidGlass"))?;
    let seq = dir.join("sequence");
    let first = seq.join("frame_00.png");
    if !first.is_file() {
        return Err(usage(format!(
            "{} is missing: run preview-export --cluster first",
            first.display()
        )));
    }
    let probe = Frame::load(&first, ctx.straight)?;
    let origin = ctx.origin(&probe).filter(|o| *o != (0, 0)).ok_or_else(|| {
        usage(format!(
            "sequence frames are {}x{}, which matches neither the cockpit nor the hood envelope plus {} px",
            probe.width,
            probe.height,
            cli::CROP_MARGIN
        ))
    })?;
    let hood = ctx.tokens.hood.region(cli::CROP_MARGIN);
    let prefix = if (hood.x, hood.y) == origin { "hood_" } else { "" };
    let mut items = Vec::new();
    for i in 0..SEQUENCE_FRAMES {
        let (gear, mph, rpm) = sequence_key(i);
        items.push(Item {
            path: seq.join(format!("frame_{i:02}.png")),
            what: format!("sequence frame {i:02} ({})", segment(i)),
            gear,
            mph,
            rpm,
            origin: Some(origin),
        });
    }
    for (state, gear, mph, rpm) in STATES {
        items.push(Item {
            path: dir.join(format!("{prefix}{state}.png")),
            what: format!("state {prefix}{state}"),
            gear,
            mph,
            rpm,
            origin: None,
        });
    }
    let count = args.num("--count", 8usize)?.clamp(1, items.len());
    let seed = args.num("--seed", 1u64)?;
    let out = args
        .one("--out")
        .map_or_else(|| ctx.mockups.join("blind").join(seed.to_string()), PathBuf::from);
    let key_path = args.one("--key").map_or_else(
        || {
            ctx.mockups
                .join(&name)
                .join("review")
                .join(format!("blind_key_{seed}.txt"))
        },
        PathBuf::from,
    );
    let (out_abs, key_abs, dash_abs) = (absolute(&out), absolute(&key_path), absolute(dir));
    if key_abs.starts_with(&out_abs) {
        return Err(usage(format!(
            "the key {} would sit inside the blind folder {}, where the reader looks",
            key_path.display(),
            out.display()
        )));
    }
    if out_abs.starts_with(&dash_abs) {
        return Err(usage(format!(
            "the blind folder {} is inside {}, so its path would name the dash",
            out.display(),
            dir.display()
        )));
    }
    // Threads share mockups/: a folder that already holds anything may be another thread's
    // packet, or this one's from an earlier draw, so it is never reused.
    if std::fs::read_dir(&out).is_ok_and(|mut entries| entries.next().is_some()) {
        return Err(usage(format!(
            "{} already holds files (another packet, or an earlier draw): pick a seed whose folder does not exist yet",
            out.display()
        )));
    }
    // Read and prepare every drawn frame before anything is written.
    let masks = cli::masks(args, ctx, (0, 0))?;
    let mut prepared = Vec::with_capacity(count);
    for (k, idx) in draw(items.len(), count, seed).into_iter().enumerate() {
        let Some(item) = items.get(idx) else {
            continue;
        };
        let src = Frame::load(&item.path, ctx.straight)?;
        let mut full = match item.origin {
            Some((x, y)) => {
                let mut f = Frame::filled(ctx.tokens.canvas.0, ctx.tokens.canvas.1, [0; 3]);
                f.blit(&src, u32::try_from(x).unwrap_or(0), u32::try_from(y).unwrap_or(0));
                f
            }
            None => src,
        };
        for m in &masks {
            full.fill(*m, [0, 0, 0]);
        }
        let file = format!("glance_{:02}.png", k + 1);
        prepared.push((file, full.resize(BLIND_SIZE.0, BLIND_SIZE.1), item));
    }
    let mut key = format!(
        "blind glance read of {name}: seed {seed}, {count} frames, shown at {}x{}\n",
        BLIND_SIZE.0, BLIND_SIZE.1
    );
    let mut shown = Vec::with_capacity(prepared.len());
    for (file, frame, item) in &prepared {
        let dest = out.join(file);
        frame.save(&dest)?;
        let _ = writeln!(key, "{file} = {}", key_line(&item.what, item.gear, item.mph, item.rpm));
        shown.push(dest);
    }
    if let Some(parent) = key_path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| file_err(parent, e))?;
    }
    std::fs::write(&key_path, key).map_err(|e| file_err(&key_path, e))?;
    out!("blind frames (give these paths, and nothing else about the dash, to the reader):");
    for p in &shown {
        out!("  {}", p.display());
    }
    out!("answer key (never show it to the reader): {}", key_path.display());
    out!("ask {READER_QUESTION}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_draw_is_seeded_distinct_and_bounded() {
        let a = draw(64, 8, 7);
        assert_eq!(a.len(), 8);
        assert_eq!(a, draw(64, 8, 7));
        assert_ne!(a, draw(64, 8, 8));
        let mut s = a.clone();
        s.sort_unstable();
        s.dedup();
        assert_eq!(s.len(), 8);
        assert!(a.iter().all(|&i| i < 64));
        assert_eq!(draw(3, 10, 1).len(), 3);
    }

    /// Miles per hour in one metre per second.
    const MPH_PER_MS: f32 = 2.236_936_3;

    #[test]
    fn the_key_matches_the_review_telemetry_of_the_cluster_crate() {
        use fh6_clusters::review;
        assert_eq!(review::SEQUENCE_FRAMES, SEQUENCE_FRAMES);
        assert_eq!(review::NEUTRAL, NEUTRAL);
        assert!((review::LIMITER_RPM - LIMITER_RPM).abs() < 1e-6);
        for i in 0..SEQUENCE_FRAMES {
            let t = review::sequence_frame(i);
            let (gear, mph, rpm) = sequence_key(i);
            assert_eq!(gear, t.gear, "frame {i}");
            assert!((mph - t.speed_ms * MPH_PER_MS).abs() < 0.01, "frame {i}: {mph} mph");
            assert!((rpm - t.current_engine_rpm).abs() < 0.01, "frame {i}: {rpm} rpm");
            assert!((t.engine_max_rpm - LIMITER_RPM).abs() < 1e-6);
        }
        for ((name, gear, mph, rpm), s) in STATES.iter().zip(review::STATES.iter()) {
            assert_eq!((*name, *gear), (s.name, s.gear));
            assert!((mph - s.mph).abs() < 1e-6 && (rpm - s.rpm).abs() < 1e-6, "{name}");
        }
    }

    #[test]
    fn names_segments_and_key_lines() {
        assert_eq!(gear_name(NEUTRAL), "N");
        assert_eq!(gear_name(0), "R");
        assert_eq!(gear_name(4), "4");
        assert_eq!(segment(35), "limiter hold");
        assert_eq!(sequence_key(35), (3, 92.5, 8640.0));
        assert_eq!(
            key_line("x", 3, 92.5, 8700.0),
            "x: gear 3, 92.5 mph, 8700 rpm (100% of the 8700 rpm limiter, zone limiter)"
        );
        assert_eq!(
            (zone(900.0), zone(5200.0), zone(7100.0), zone(8640.0)),
            ("low", "mid", "high", "limiter")
        );
        // The blind size stays inside the 1:1 view.
        assert!(u64::from(BLIND_SIZE.0) * u64::from(BLIND_SIZE.1) <= cli::VIEW_MAX_PIXELS);
    }
}
