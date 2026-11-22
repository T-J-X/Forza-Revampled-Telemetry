//! Renders the GPU test cluster offscreen and reports per-frame times.
//!
//! ```text
//! cargo run --release -p bench-render -- [--width 3840] [--height 2160] [--frames 1000]
//! ```
//! Each frame: build the quad batch from a moving telemetry sweep (CPU), upload + record +
//! submit (CPU), then wait for the GPU to finish (an upper bound on GPU time, including the
//! queue round-trip). Prints a Markdown table.
use fh6_gpu::test_cluster::{self, TestClusterInput};
use fh6_gpu::{GpuContext, OffscreenTarget, QuadBatch};
use std::process::ExitCode;
use std::time::{Duration, Instant};

const WARMUP_FRAMES: usize = 60;

fn arg<T: std::str::FromStr>(name: &str, default: T) -> T {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == name)
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// p-th percentile (0..=100) of `samples`, sorted in place.
fn percentile(samples: &mut [Duration], p: f64) -> Duration {
    if samples.is_empty() {
        return Duration::ZERO;
    }
    samples.sort_unstable();
    let rank = ((p / 100.0) * (samples.len() - 1) as f64).round() as usize;
    samples[rank.min(samples.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn run() -> Result<(), String> {
    let width: u32 = arg("--width", 3840);
    let height: u32 = arg("--height", 2160);
    let frames: usize = arg("--frames", 1000).max(1);
    let ctx = GpuContext::headless().map_err(|e| e.to_string())?;
    let mut target = OffscreenTarget::new(&ctx, width, height);
    let mut batch = QuadBatch::new();

    let mut build = Vec::with_capacity(frames);
    let mut encode = Vec::with_capacity(frames);
    let mut gpu = Vec::with_capacity(frames);
    for i in 0..WARMUP_FRAMES + frames {
        let t = i as f32 / 60.0;
        let input = TestClusterInput {
            speed: 60.0 + 60.0 * (t * 0.7).sin(),
            gear: 1 + (i / 90 % 6) as u8,
            rpm: 1000.0 + 6500.0 * (0.5 + 0.5 * (t * 2.0).sin()),
            max_rpm: 8000.0,
            shift_start: 0.72,
            redline: 0.94,
            throttle: 0.5 + 0.5 * (t * 1.3).sin(),
            brake: (0.5 - 0.5 * (t * 1.3).sin()).max(0.0),
            anchor: [width as f32 * 0.5, height as f32 * (1835.0 / 2160.0)],
            scale: height as f32 / 2160.0 * 1.45,
            accent: [0.0, 0.824, 1.0, 1.0],
        };
        let t0 = Instant::now();
        batch.clear();
        test_cluster::build(&mut batch, &input);
        let built = t0.elapsed();
        let (cpu, wait) = target.render(&ctx, 1.0, &batch);
        if i >= WARMUP_FRAMES {
            build.push(built);
            encode.push(cpu);
            gpu.push(wait);
        }
    }

    if ctx.has_failed() {
        return Err("wgpu reported an error or lost the device during the run; the timings are invalid".into());
    }
    let mut total: Vec<Duration> = build.iter().zip(&encode).map(|(a, b)| *a + *b).collect();
    println!("Adapter: {}", ctx.adapter_summary());
    println!(
        "Target: {width}x{height}, {frames} frames after {WARMUP_FRAMES} warm-up, {} quads/frame",
        batch.len()
    );
    println!();
    println!("| Stage | p50 (ms) | p99 (ms) | max (ms) |");
    println!("|---|---|---|---|");
    for (name, samples) in [
        ("Build quad batch (CPU)", &mut build),
        ("Upload + record + submit (CPU)", &mut encode),
        ("CPU total", &mut total),
        ("Submit to GPU idle (GPU upper bound)", &mut gpu),
    ] {
        let p50 = percentile(samples, 50.0);
        let p99 = percentile(samples, 99.0);
        let max = percentile(samples, 100.0);
        println!("| {name} | {:.3} | {:.3} | {:.3} |", ms(p50), ms(p99), ms(max));
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("bench-render: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_picks_ranks() {
        let mut v: Vec<Duration> = (1..=100).map(Duration::from_millis).collect();
        assert_eq!(percentile(&mut v, 50.0), Duration::from_millis(51));
        assert_eq!(percentile(&mut v, 99.0), Duration::from_millis(99));
        assert_eq!(percentile(&mut v, 100.0), Duration::from_millis(100));
        assert_eq!(percentile(&mut [], 50.0), Duration::ZERO);
    }
}
