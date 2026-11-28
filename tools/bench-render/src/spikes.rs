//! `--glass-spikes`: where the GPU time of a glass layer goes, frame by frame.
//!
//! A RetroCyber-sized panel (four panel fills, a moving bar) is drawn at 3840x2160 in six
//! variants that take the glass layer apart: no glass; one optical-glass lens in a glass layer;
//! the same lens with backdrops suppressed (the optics shader, no copy, no new segment); the
//! glass layer with a plain fill instead of glass (the copy and segment, no optics shader); the
//! lens drawn only on its rim band under four clips; and the whole lens drawn as nine clipped
//! tiles. Each runs in three pacings: back to back, with a 4K readback after every frame (as
//! `preview-export` times), and with an 8 ms pause after every frame (as the 120 Hz overlay
//! leaves the GPU idle). Variants and pacings are interleaved in blocks so clock and thermal
//! drift fall on all of them alike.
//!
//! Every frame records the kit's pass timeline (`Kit::set_gpu_timeline`). A spike is a frame
//! more than [`SPIKE_MS`] over its variant's median; each spike is put down to the interval
//! (a pass or the gap before it) that grew the most over that interval's own median. The report
//! goes to stdout and, with `--out`, to a Markdown file; `--csv` writes every frame.
use crate::{Result, H, W};
use fh6_gpu::GpuContext;
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{
    vec2, Color, GpuPassKind, GpuPassTime, Kit, LayerOptions, Layout, Painter, Palette, Rect, Shape, Tokens, View,
};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

/// A frame this much over its variant's median GPU time is a spike (ms).
const SPIKE_MS: f64 = 0.15;
/// Frames per block before the next variant and pacing take over.
const BLOCK: usize = 50;
/// Frames per variant and pacing drawn and thrown away before any block is timed.
const WARMUP: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Variant {
    NoGlass,
    Glass,
    GlassNoBackdrop,
    CopyOnly,
    RimBands,
    NineTiles,
}

impl Variant {
    const ALL: [Self; 6] = [
        Self::NoGlass,
        Self::Glass,
        Self::GlassNoBackdrop,
        Self::CopyOnly,
        Self::RimBands,
        Self::NineTiles,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::NoGlass => "no glass",
            Self::Glass => "glass lens",
            Self::GlassNoBackdrop => "glass lens, backdrops off",
            Self::CopyOnly => "glass layer, plain fill",
            Self::RimBands => "glass lens, rim band only (4 clips)",
            Self::NineTiles => "glass lens as 9 clipped tiles",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Pacing {
    BackToBack,
    Readback,
    Paced8ms,
}

impl Pacing {
    const ALL: [Self; 3] = [Self::BackToBack, Self::Readback, Self::Paced8ms];

    fn name(self) -> &'static str {
        match self {
            Self::BackToBack => "back to back",
            Self::Readback => "4K readback each frame",
            Self::Paced8ms => "8 ms idle each frame",
        }
    }
}

/// One timed frame.
struct Frame {
    total_ms: f64,
    /// `(interval name, ms)`: each pass, and the gap before each pass after the first.
    intervals: Vec<(String, f64)>,
}

fn intervals(passes: &[GpuPassTime]) -> Vec<(String, f64)> {
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    let mut out = Vec::with_capacity(passes.len() * 2);
    let mut last_end: Option<Duration> = None;
    for p in passes {
        let name = if p.kind == GpuPassKind::Resolve {
            p.kind.name().to_string()
        } else {
            format!("{} #{}", p.kind.name(), p.segment)
        };
        if let Some(e) = last_end {
            out.push((format!("gap before {name}"), ms(p.start.saturating_sub(e))));
        }
        out.push((name, ms(p.end.saturating_sub(p.start))));
        last_end = Some(p.end);
    }
    out
}

fn median(v: &mut [f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn quantile(v: &mut [f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() - 1) as f64 * q).round() as usize]
}

/// The panel and lens rectangles (cockpit layout units).
const PANEL: Rect = Rect {
    min: fh6_kit::Vec2 { x: -620.0, y: -230.0 },
    max: fh6_kit::Vec2 { x: 620.0, y: 230.0 },
};
const RIM: f32 = 48.0;

fn draw(p: &mut Painter<'_>, variant: Variant, frame: usize, lens: fh6_kit::OpticalGlass) {
    // Four panel-sized fills, as a surround, two light images and a lens body would be.
    for (i, c) in [0x0A0C10_u32, 0x101418, 0x06080A, 0x000000].into_iter().enumerate() {
        let r = PANEL.expand(12.0 - i as f32 * 4.0);
        p.fill(&Shape::round_rect(r, 28.0), Color::rgb_u32(c).with_alpha(0.9));
    }
    // A moving bar, so the instances change every frame as a tach does.
    let t = (frame as f32 * 0.013).fract();
    p.fill(
        &Shape::rect(Rect::from_xywh(-560.0, -120.0, 1120.0 * t, 60.0)),
        Color::rgb_u32(0x38F0C8),
    );
    p.fill(
        &Shape::rect(Rect::from_xywh(-200.0, 0.0, 400.0, 160.0)),
        Color::rgb_u32(0xF0F0F0),
    );
    let lens_shape = Shape::round_rect(PANEL, 28.0);
    match variant {
        Variant::NoGlass => {}
        Variant::Glass => p.layer(LayerOptions::glass(0.0), |p| p.fill(&lens_shape, lens)),
        Variant::GlassNoBackdrop => p.scoped(|p| {
            p.set_backdrops(false);
            p.layer(LayerOptions::glass(0.0), |p| p.fill(&lens_shape, lens));
        }),
        Variant::CopyOnly => p.layer(LayerOptions::glass(0.0), |p| {
            p.fill(&lens_shape, Color::rgb_u32(0x202830).with_alpha(0.05));
        }),
        Variant::RimBands | Variant::NineTiles => {
            let (x0, x1) = (PANEL.min.x - 2.0, PANEL.max.x + 2.0);
            let (y0, y1) = (PANEL.min.y - 2.0, PANEL.max.y + 2.0);
            let xs = [x0, PANEL.min.x + RIM, PANEL.max.x - RIM, x1];
            let ys = [y0, PANEL.min.y + RIM, PANEL.max.y - RIM, y1];
            p.layer(LayerOptions::glass(0.0), |p| {
                for i in 0..9 {
                    let (c, r) = (i % 3, i / 3);
                    if variant == Variant::RimBands && c == 1 && r == 1 {
                        continue;
                    }
                    let tile = Rect::from_points(vec2(xs[c], ys[r]), vec2(xs[c + 1], ys[r + 1]));
                    p.scoped(|p| {
                        p.clip(tile, 0.0);
                        p.fill(&lens_shape, lens);
                    });
                }
            });
        }
    }
}

/// Runs the spike bench and writes its report.
pub fn run(frames: usize, out: Option<&Path>, csv: Option<&Path>) -> Result<()> {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tokens = Tokens::load(&root.join("data/design_tokens.json")).unwrap_or_default();
    let ctx = GpuContext::headless()?;
    let adapter = ctx.adapter_summary().to_string();
    println!("GPU: {adapter}");
    let mut kit = Kit::new(&ctx.device, &ctx.queue, tokens.clone())?;
    if !kit.set_gpu_timeline(true) {
        return Err("this adapter has no TIMESTAMP_QUERY".into());
    }
    let family = fh6_clusters::registry::resolve(fh6_core::ClusterTheme::RetroCyber).family;
    let palette = Palette::family(&tokens, family.key());
    kit.set_palette(palette);
    let mut lens = tokens.optical_glass(&palette);
    lens.env_gain = lens.env_gain.map(|e| e * 0.04);
    lens.roughness = lens.roughness.min(0.04);
    let target = Offscreen::new(&ctx.device, W, H);
    let layout = Layout::from_config(&fh6_core::AppConfig::default(), &tokens, View::Cockpit, (W, H));

    let mut runs: BTreeMap<(Variant, Pacing), Vec<Frame>> = BTreeMap::new();
    let mut work: BTreeMap<Variant, u64> = BTreeMap::new();
    let blocks = frames.div_ceil(BLOCK);
    let mut frame_no = 0usize;
    let started = Instant::now();
    for block in 0..=blocks {
        for pacing in Pacing::ALL {
            for variant in Variant::ALL {
                // Block 0 is the warm-up, timed by nobody.
                let count = if block == 0 { WARMUP } else { BLOCK };
                for _ in 0..count {
                    frame_no += 1;
                    {
                        let mut p = kit.begin_frame(&layout);
                        draw(&mut p, variant, frame_no, lens);
                    }
                    let mut encoder = ctx.device.create_command_encoder(&Default::default());
                    let stats = kit.render(
                        &ctx.device,
                        &ctx.queue,
                        &mut encoder,
                        target.view(),
                        OFFSCREEN_FORMAT,
                        (W, H),
                        1.0,
                    );
                    if pacing == Pacing::Readback {
                        target.copy_out(&mut encoder);
                    }
                    ctx.queue.submit(Some(encoder.finish()));
                    kit.frame_submitted();
                    if pacing == Pacing::Readback {
                        let _ = target.read(&ctx.device)?;
                    }
                    // One readback: the frame runs from the first pass's start to the resolve's end.
                    let passes = kit.gpu_timeline(&ctx.device);
                    let total = passes.as_ref().and_then(|p| p.last()).map(|p| p.end);
                    if pacing == Pacing::Paced8ms {
                        std::thread::sleep(Duration::from_millis(8));
                    }
                    if block == 0 {
                        continue;
                    }
                    work.insert(variant, stats.work_pixels);
                    if let (Some(total), Some(passes)) = (total, passes) {
                        runs.entry((variant, pacing)).or_default().push(Frame {
                            total_ms: total.as_secs_f64() * 1e3,
                            intervals: intervals(&passes),
                        });
                    }
                }
            }
        }
        if ctx.has_failed() {
            return Err("wgpu reported an error".into());
        }
    }
    println!("{frame_no} frames in {:.1} s", started.elapsed().as_secs_f32());

    let mut report = String::new();
    let _ = writeln!(
        report,
        "## Glass layer spike bench (generated by `bench-render --glass-spikes`)\n"
    );
    let _ = writeln!(
        report,
        "3840x2160, adapter {adapter}, {} frames per variant and pacing in blocks of {BLOCK}, \
interleaved. GPU times from the kit's pass timeline, in ms. A spike is a frame more than {SPIKE_MS} ms \
over its row's median; \"spikes put down to\" names the interval that grew the most over its own median \
in each spike.\n",
        blocks * BLOCK
    );
    let _ = writeln!(
        report,
        "| Variant | Pacing | work px | p50 | p90 | p99 | max | spikes | spikes put down to |"
    );
    let _ = writeln!(report, "|---|---|---|---|---|---|---|---|---|");
    let mut detail = String::new();
    for ((variant, pacing), frames) in &runs {
        let mut totals: Vec<f64> = frames.iter().map(|f| f.total_ms).collect();
        let p50 = median(&mut totals);
        let p90 = quantile(&mut totals, 0.90);
        let p99 = quantile(&mut totals, 0.99);
        let max = totals.last().copied().unwrap_or(f64::NAN);
        // Each interval's median over the row.
        let mut per: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
        for f in frames {
            for (n, v) in &f.intervals {
                per.entry(n.as_str()).or_default().push(*v);
            }
        }
        let medians: BTreeMap<&str, f64> = per.iter_mut().map(|(n, v)| (*n, median(v))).collect();
        let mut blame: BTreeMap<String, (usize, f64)> = BTreeMap::new();
        let mut spikes = 0;
        for f in frames.iter().filter(|f| f.total_ms > p50 + SPIKE_MS) {
            spikes += 1;
            let worst = f
                .intervals
                .iter()
                .map(|(n, v)| (n, v - medians.get(n.as_str()).copied().unwrap_or(0.0)))
                .max_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((n, grew)) = worst {
                let e = blame.entry(n.clone()).or_default();
                e.0 += 1;
                e.1 += grew;
            }
        }
        let blamed = blame
            .iter()
            .map(|(n, (c, grew))| format!("{n}: {c} (+{:.3} ms avg)", grew / *c as f64))
            .collect::<Vec<_>>()
            .join("; ");
        let _ = writeln!(
            report,
            "| {} | {} | {} | {p50:.3} | {p90:.3} | {p99:.3} | {max:.3} | {spikes} | {} |",
            variant.name(),
            pacing.name(),
            work.get(variant).copied().unwrap_or(0),
            if blamed.is_empty() { "-" } else { &blamed }
        );
        let _ = write!(detail, "| {} | {} |", variant.name(), pacing.name());
        for (n, m) in &medians {
            let _ = write!(detail, " {n} {m:.3};");
        }
        let _ = writeln!(detail, " |");
    }
    let _ = writeln!(report, "\nMedian of each interval (ms):\n");
    let _ = writeln!(report, "| Variant | Pacing | intervals |\n|---|---|---|");
    report.push_str(&detail);
    println!("{report}");
    if let Some(path) = out {
        std::fs::write(path, &report)?;
        println!("wrote {}", path.display());
    }
    if let Some(path) = csv {
        let mut s = String::from("variant,pacing,frame,total_ms,intervals\n");
        for ((variant, pacing), frames) in &runs {
            for (i, f) in frames.iter().enumerate() {
                let iv = f
                    .intervals
                    .iter()
                    .map(|(n, v)| format!("{n}={v:.4}"))
                    .collect::<Vec<_>>()
                    .join(";");
                let _ = writeln!(s, "{},{},{i},{:.4},{iv}", variant.name(), pacing.name(), f.total_ms);
            }
        }
        std::fs::write(path, s)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
