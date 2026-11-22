//! Measures the optional Windows.Graphics.Capture window capture (the Liquid Glass
//! `refraction_source`): frames delivered, CPU cost of copy + map + convert, achieved fps.
//!
//! ```text
//! cargo run --release -p wgc-bench -- --process fh6-stand-in.exe
//!     [--hwnd N] [--seconds 5] [--region X,Y,W,H] [--max W,H] [--rate-hz 120] [--repaint] [--blocking]
//! ```
//! `--region` is in the window's client pixels (default `1300,1560,1240,560`, the cockpit
//! cluster zone at 4K); `--max` is the downscale target (default half the region). WGC only
//! delivers a frame when the window is recomposed, so a static window such as the stand-in
//! needs `--repaint`, which invalidates it before every poll; the real game presents on its own.
//! `--blocking` reads each frame back in the same call (`Map` waits for the GPU) instead of the
//! default pipelined readback, which delivers a frame one poll after it arrives.
use fh6_win::wgc::{self, CaptureFrame, CaptureRegion, Readback, WindowCapture};
use std::process::ExitCode;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
struct Args {
    process: Option<String>,
    hwnd: Option<isize>,
    seconds: f64,
    region: CaptureRegion,
    max: Option<(u32, u32)>,
    rate_hz: f64,
    repaint: bool,
    blocking: bool,
}

fn parse_list(value: &str, count: usize) -> Option<Vec<i64>> {
    let parts: Vec<i64> = value.split(',').map(|p| p.trim().parse().ok()).collect::<Option<_>>()?;
    (parts.len() == count).then_some(parts)
}

fn parse_hwnd(value: &str) -> Option<isize> {
    match value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")) {
        Some(hex) => isize::from_str_radix(hex, 16).ok(),
        None => value.parse().ok(),
    }
}

fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Args, String> {
    let mut parsed = Args {
        process: None,
        hwnd: None,
        seconds: 5.0,
        region: CaptureRegion {
            x: 1300,
            y: 1560,
            width: 1240,
            height: 560,
        },
        max: None,
        rate_hz: 120.0,
        repaint: false,
        blocking: false,
    };
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
        match arg.as_str() {
            "--process" => parsed.process = Some(value("--process")?),
            "--hwnd" => {
                let v = value("--hwnd")?;
                parsed.hwnd = Some(parse_hwnd(&v).ok_or(format!("bad --hwnd {v:?}"))?);
            }
            "--seconds" => {
                let v = value("--seconds")?;
                parsed.seconds = v
                    .parse::<f64>()
                    .ok()
                    .filter(|s| *s > 0.0 && s.is_finite())
                    .ok_or(format!("bad --seconds {v:?}"))?;
            }
            "--rate-hz" => {
                let v = value("--rate-hz")?;
                parsed.rate_hz = v
                    .parse::<f64>()
                    .ok()
                    .filter(|s| *s >= 1.0 && s.is_finite())
                    .ok_or(format!("bad --rate-hz {v:?}"))?;
            }
            "--region" => {
                let v = value("--region")?;
                let p = parse_list(&v, 4).ok_or(format!("bad --region {v:?}, expected X,Y,W,H"))?;
                let bad = |e: std::num::TryFromIntError| format!("bad --region {v:?}: {e}");
                parsed.region = CaptureRegion {
                    x: i32::try_from(p[0]).map_err(bad)?,
                    y: i32::try_from(p[1]).map_err(bad)?,
                    width: u32::try_from(p[2]).map_err(bad)?,
                    height: u32::try_from(p[3]).map_err(bad)?,
                };
            }
            "--max" => {
                let v = value("--max")?;
                let p = parse_list(&v, 2).ok_or(format!("bad --max {v:?}, expected W,H"))?;
                let bad = |e: std::num::TryFromIntError| format!("bad --max {v:?}: {e}");
                parsed.max = Some((u32::try_from(p[0]).map_err(bad)?, u32::try_from(p[1]).map_err(bad)?));
            }
            "--repaint" => parsed.repaint = true,
            "--blocking" => parsed.blocking = true,
            other => return Err(format!("unknown argument {other:?}")),
        }
    }
    if parsed.process.is_none() == parsed.hwnd.is_none() {
        return Err("pass exactly one of --process <exe name> or --hwnd <n>".into());
    }
    Ok(parsed)
}

/// The `p` quantile (0..=1) of sorted samples, nearest rank.
fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let idx = ((sorted.len() - 1) as f64 * p.clamp(0.0, 1.0)).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn mean_rgba(pixels: &[u8]) -> [u8; 4] {
    let mut sum = [0u64; 4];
    let mut n = 0u64;
    for px in pixels.chunks_exact(4) {
        for (s, &c) in sum.iter_mut().zip(px) {
            *s += u64::from(c);
        }
        n += 1;
    }
    let n = n.max(1);
    sum.map(|s| u8::try_from(s / n).unwrap_or(u8::MAX))
}

fn run(args: &Args) -> Result<(), String> {
    fh6_win::monitor::enable_per_monitor_dpi_awareness();
    if !wgc::is_supported() {
        return Err("Windows.Graphics.Capture is not supported on this system".into());
    }
    let hwnd = match (&args.process, args.hwnd) {
        (_, Some(h)) => h,
        (Some(exe), None) => wgc::find_process_window(exe).ok_or(format!("no visible window of {exe}"))?,
        (None, None) => return Err("no target".into()),
    };
    let readback = if args.blocking {
        Readback::Blocking
    } else {
        Readback::Pipelined
    };
    let max = args
        .max
        .unwrap_or(((args.region.width / 2).max(1), (args.region.height / 2).max(1)));
    println!(
        "target hwnd {hwnd:#x}, region {},{} {}x{} (client px), max {}x{}, poll {} Hz for {} s, {:?} readback{}",
        args.region.x,
        args.region.y,
        args.region.width,
        args.region.height,
        max.0,
        max.1,
        args.rate_hz,
        args.seconds,
        readback,
        if args.repaint {
            ", repainting the target each poll"
        } else {
            ""
        }
    );

    let setup = Instant::now();
    let mut capture = WindowCapture::start(hwnd, args.region, max).map_err(|e| format!("start: {e}"))?;
    capture.set_readback(readback);
    let setup_time = setup.elapsed();
    let mut frame = CaptureFrame::with_capacity(max.0, max.1);

    // Room for one sample per poll; never reallocated in the loop.
    let polls_expected = (args.seconds * args.rate_hz).ceil() as usize + 16;
    let mut copy_costs: Vec<Duration> = Vec::with_capacity(polls_expected);
    let mut call_costs: Vec<Duration> = Vec::with_capacity(polls_expected);
    let period = Duration::from_secs_f64(1.0 / args.rate_hz);
    let start = Instant::now();
    let deadline = start + Duration::from_secs_f64(args.seconds);
    let mut next = start;
    let mut polls = 0u64;
    let mut first_frame: Option<Duration> = None;
    let mut last_frame_at = start;
    while Instant::now() < deadline {
        if args.repaint {
            wgc::request_repaint(hwnd);
        }
        let call = Instant::now();
        let got = capture.latest(&mut frame).map_err(|e| format!("latest: {e}"))?;
        let call_time = call.elapsed();
        polls += 1;
        if call_costs.len() < call_costs.capacity() {
            call_costs.push(call_time);
        }
        if got && copy_costs.len() < copy_costs.capacity() {
            copy_costs.push(frame.cpu);
            first_frame.get_or_insert(call.duration_since(start));
            last_frame_at = call;
        }
        next += period;
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else {
            next = now;
        }
    }
    let elapsed = start.elapsed();
    drop(capture);

    let frames = copy_costs.len();
    copy_costs.sort_unstable();
    call_costs.sort_unstable();
    // fps over the span the frames actually arrived in (excludes start-up before frame 1).
    let span = first_frame.map_or(Duration::ZERO, |f| last_frame_at.duration_since(start + f));
    let fps_span = if frames > 1 && !span.is_zero() {
        (frames - 1) as f64 / span.as_secs_f64()
    } else {
        0.0
    };
    println!("setup (device, item, pool, session): {:.2} ms", ms(setup_time));
    println!(
        "polls {polls} in {:.2} s ({:.1} Hz), frames captured {frames}, first frame after {:.2} ms",
        elapsed.as_secs_f64(),
        polls as f64 / elapsed.as_secs_f64(),
        first_frame.map_or(f64::NAN, ms)
    );
    println!(
        "achieved fps {:.1} (frames / run time), {:.1} (between first and last frame)",
        frames as f64 / elapsed.as_secs_f64(),
        fps_span
    );
    println!(
        "copy+map+convert CPU per delivered frame: p50 {:.3} ms, p99 {:.3} ms, max {:.3} ms",
        ms(percentile(&copy_costs, 0.5)),
        ms(percentile(&copy_costs, 0.99)),
        ms(percentile(&copy_costs, 1.0))
    );
    println!(
        "whole latest() call, every poll: p50 {:.3} ms, p99 {:.3} ms, max {:.3} ms",
        ms(percentile(&call_costs, 0.5)),
        ms(percentile(&call_costs, 0.99)),
        ms(percentile(&call_costs, 1.0))
    );
    if frames > 0 {
        let [r, g, b, a] = mean_rgba(&frame.pixels);
        println!(
            "last frame {}x{} RGBA, mean colour #{r:02x}{g:02x}{b:02x} alpha {a}, timestamp {:.3} s",
            frame.width,
            frame.height,
            frame.timestamp.as_secs_f64()
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("wgc-bench: {err}");
            eprintln!(
                "usage: wgc-bench --process <exe> | --hwnd <n> [--seconds 5] [--region X,Y,W,H] [--max W,H] [--rate-hz 120] [--repaint] [--blocking]"
            );
            return ExitCode::FAILURE;
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("wgc-bench: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Result<Args, String> {
        parse_args(list.iter().map(|s| (*s).to_owned()))
    }

    #[test]
    fn parses_a_full_command_line() {
        let a = args(&[
            "--process",
            "fh6-stand-in.exe",
            "--seconds",
            "2.5",
            "--region",
            "1,2,30,40",
            "--max",
            "15,20",
            "--repaint",
        ])
        .expect("valid");
        assert_eq!(a.process.as_deref(), Some("fh6-stand-in.exe"));
        assert_eq!(a.seconds, 2.5);
        assert_eq!(
            a.region,
            CaptureRegion {
                x: 1,
                y: 2,
                width: 30,
                height: 40
            }
        );
        assert_eq!(a.max, Some((15, 20)));
        assert!(a.repaint);
    }

    #[test]
    fn needs_exactly_one_target() {
        assert!(args(&[]).is_err());
        assert!(args(&["--process", "a.exe", "--hwnd", "5"]).is_err());
        assert_eq!(args(&["--hwnd", "0x1A"]).expect("hex").hwnd, Some(26));
        assert!(args(&["--hwnd", "5", "--region", "1,2,3"]).is_err());
        assert!(args(&["--hwnd", "5", "--seconds", "0"]).is_err());
    }

    #[test]
    fn percentiles_use_nearest_rank() {
        let s: Vec<Duration> = (1..=100).map(Duration::from_millis).collect();
        assert_eq!(percentile(&s, 0.5), Duration::from_millis(51));
        assert_eq!(percentile(&s, 0.99), Duration::from_millis(99));
        assert_eq!(percentile(&[], 0.5), Duration::ZERO);
    }
}
