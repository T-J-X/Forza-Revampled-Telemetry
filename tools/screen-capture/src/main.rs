//! Saves a PNG of the desktop (or a region of it), for the manual run checklist.
//!
//! ```text
//! cargo run -p screen-capture -- --out docs/handover-shots/overlay.png
//!     [--region X,Y,W,H] [--delay-ms N] [--half]
//! ```
//! `--half` box-filters the image to half size to keep 4K screenshots small.
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

struct Args {
    out: PathBuf,
    region: Option<(i32, i32, u32, u32)>,
    delay: Duration,
    half: bool,
}

fn parse_args() -> Result<Args, String> {
    let mut out = None;
    let mut region = None;
    let mut delay = Duration::ZERO;
    let mut half = false;
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--out" => out = it.next().map(PathBuf::from),
            "--delay-ms" => {
                let ms: u64 = it.next().and_then(|v| v.parse().ok()).ok_or("--delay-ms needs a number")?;
                delay = Duration::from_millis(ms);
            }
            "--region" => {
                let v = it.next().ok_or("--region needs X,Y,W,H")?;
                let parts: Vec<i64> = v.split(',').filter_map(|p| p.trim().parse().ok()).collect();
                let [x, y, w, h] = parts[..] else {
                    return Err(format!("bad --region {v:?}, expected X,Y,W,H"));
                };
                region = Some((
                    i32::try_from(x).map_err(|e| e.to_string())?,
                    i32::try_from(y).map_err(|e| e.to_string())?,
                    u32::try_from(w).map_err(|e| e.to_string())?,
                    u32::try_from(h).map_err(|e| e.to_string())?,
                ));
            }
            "--half" => half = true,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    Ok(Args {
        out: out.ok_or("--out <file.png> is required")?,
        region,
        delay,
        half,
    })
}

fn halve(width: u32, height: u32, rgba: &[u8]) -> (u32, u32, Vec<u8>) {
    let (hw, hh) = ((width / 2).max(1), (height / 2).max(1));
    let mut out = Vec::with_capacity((hw * hh * 4) as usize);
    let at = |x: u32, y: u32, c: usize| u32::from(rgba[((y.min(height - 1) * width + x.min(width - 1)) * 4) as usize + c]);
    for y in 0..hh {
        for x in 0..hw {
            for c in 0..4 {
                let sum = at(2 * x, 2 * y, c) + at(2 * x + 1, 2 * y, c) + at(2 * x, 2 * y + 1, c) + at(2 * x + 1, 2 * y + 1, c);
                out.push(u8::try_from(sum / 4).unwrap_or(255));
            }
        }
    }
    (hw, hh, out)
}

fn run() -> Result<(), String> {
    let args = parse_args()?;
    fh6_win::monitor::enable_per_monitor_dpi_awareness();
    std::thread::sleep(args.delay);
    let (x, y, w, h) = match args.region {
        Some(r) => r,
        None => {
            let m = fh6_win::monitor::primary_monitor_rect().ok_or("no primary monitor")?;
            (m.x, m.y, m.width, m.height)
        }
    };
    let shot = fh6_win::capture::capture_screen(x, y, w, h).map_err(|e| e.to_string())?;
    let (width, height, pixels) = if args.half {
        halve(shot.width, shot.height, &shot.rgba)
    } else {
        (shot.width, shot.height, shot.rgba)
    };
    if let Some(dir) = args.out.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    let file = File::create(&args.out).map_err(|e| format!("creating {}: {e}", args.out.display()))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&pixels).map_err(|e| e.to_string())?;
    println!("saved {} ({width}x{height}) from region {x},{y} {w}x{h}", args.out.display());
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("screen-capture: {err}");
            ExitCode::FAILURE
        }
    }
}
