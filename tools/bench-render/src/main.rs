//! Offscreen benchmark of the render kit at 3840x2160.
//!
//! Scenarios: the two showcase clusters (bloom on and off), glass with and without a
//! refraction source, the fluid solver on, and an empty frame. For each it reports, over N
//! frames after warm-up:
//! - CPU `update` + `draw` (recording into the arena);
//! - CPU `Kit::render` (packing, the single upload, pass recording);
//! - GPU time from timestamp queries around the whole frame.
//!
//! `--frames N` (default 600), `--out FILE` writes the Markdown report
//! (e.g. `docs/perf/render-kit.md`).

use fh6_core::{AppConfig, ClusterTheme, FH6TelemetryData};
use fh6_gpu::GpuContext;
use fh6_kit::fluid::{Fluid, FluidConfig};
use fh6_kit::testing::{Offscreen, OFFSCREEN_FORMAT};
use fh6_kit::{
    vec2, Cluster, Color, FrameInput, Glass, ImagePaint, Kit, LayerOptions, Layout, Logo, Palette, Rect, Refraction,
    Shape, SignalSmoother, Tokens, View,
};
use std::error::Error;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::Instant;

const W: u32 = 3840;
const H: u32 = 2160;
const WARMUP: usize = 60;
const DT: f32 = 1.0 / 120.0;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Stats {
    p50: f64,
    p99: f64,
    max: f64,
}

fn stats(samples: &mut [f64]) -> Stats {
    if samples.is_empty() {
        return Stats {
            p50: f64::NAN,
            p99: f64::NAN,
            max: f64::NAN,
        };
    }
    samples.sort_by(f64::total_cmp);
    let at = |q: f64| samples[((samples.len() - 1) as f64 * q).round() as usize];
    Stats {
        p50: at(0.50),
        p99: at(0.99),
        max: samples[samples.len() - 1],
    }
}

#[derive(Debug, Clone)]
struct Row {
    name: String,
    record: Stats,
    render: Stats,
    gpu: Stats,
    draws: u32,
    instances: u32,
    work: u64,
}

/// What a scenario draws on top of (or instead of) a cluster.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Extra {
    None,
    /// A glass plate over a refraction source texture.
    GlassWorld,
    /// A glass plate with no refraction source (own layers only).
    GlassOwn,
    /// The fluid solver stepping each frame, its dye drawn in a plate.
    Fluid,
    /// Nothing at all.
    Empty,
}

struct Scenario {
    name: &'static str,
    theme: Option<ClusterTheme>,
    bloom: bool,
    extra: Extra,
}

fn telemetry(frame: usize) -> FH6TelemetryData {
    // A rev sweep through the gears, so springs, strobes and digits keep moving.
    let phase = (frame as f32 * DT * 0.7).fract();
    FH6TelemetryData {
        is_race_on: 1,
        engine_max_rpm: 8700.0,
        engine_idle_rpm: 900.0,
        current_engine_rpm: 2500.0 + 6400.0 * phase,
        speed_ms: 20.0 + 40.0 * phase,
        gear: 2 + (frame / 200 % 5) as u8,
        accel: 230,
        acceleration_x: 6.0 * (frame as f32 * 0.01).sin(),
        ..Default::default()
    }
}

fn main() -> Result<()> {
    let mut frames = 600usize;
    let mut out: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--frames" => frames = args.next().ok_or("--frames needs a value")?.parse()?,
            "--out" => out = Some(PathBuf::from(args.next().ok_or("--out needs a value")?)),
            other => return Err(format!("unknown argument {other:?}").into()),
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let tokens = Tokens::load(&root.join("data/figma_tokens.json")).unwrap_or_default();
    let ctx = GpuContext::headless()?;
    let adapter = ctx.adapter_summary().to_string();
    println!("GPU: {adapter}");
    let mut kit = Kit::new(&ctx.device, &ctx.queue, tokens.clone())?;
    if !kit.set_gpu_timing(true) {
        println!("note: this adapter has no TIMESTAMP_QUERY; GPU column is n/a");
    }
    let target = Offscreen::new(&ctx.device, W, H);
    let cfg = AppConfig::default();
    let layout = Layout::from_config(&cfg, &tokens, View::Cockpit, (W, H));
    let logo = Logo {
        custom_name: "TJX".into(),
        ..Logo::default()
    };
    let fonts = kit.font_choice(None);

    // A stand-in "game frame" for the refraction source: 1920x840, half the cluster region.
    let world_size = (1920u32, 840u32);
    let world_pixels: Vec<u8> = (0..world_size.0 * world_size.1)
        .flat_map(|i| {
            let (x, y) = (i % world_size.0, i / world_size.0);
            [(x / 7 % 256) as u8, (y / 3 % 256) as u8, ((x ^ y) % 256) as u8, 255]
        })
        .collect();
    let world_tex = ctx.device.create_texture(&fh6_kit::wgpu::TextureDescriptor {
        label: Some("bench world"),
        size: fh6_kit::wgpu::Extent3d {
            width: world_size.0,
            height: world_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: fh6_kit::wgpu::TextureDimension::D2,
        format: fh6_kit::wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: fh6_kit::wgpu::TextureUsages::TEXTURE_BINDING | fh6_kit::wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let world_view = world_tex.create_view(&Default::default());
    let world_rect = layout.transform().apply_rect(&layout.envelope.expand(48.0));

    let mut fluid = Fluid::new(&ctx.device, 256, 128, FluidConfig::default());
    let dye = kit
        .images_mut()
        .register_view(&ctx.device, fluid.dye_view(), fluid.size());

    let scenarios = [
        Scenario {
            name: "Liquid Glass sample (bloom on)",
            theme: Some(ClusterTheme::LiquidGlass),
            bloom: true,
            extra: Extra::None,
        },
        Scenario {
            name: "Liquid Glass sample (bloom off)",
            theme: Some(ClusterTheme::LiquidGlass),
            bloom: false,
            extra: Extra::None,
        },
        Scenario {
            name: "Analog sample (bloom on)",
            theme: Some(ClusterTheme::PorscheAnalog),
            bloom: true,
            extra: Extra::None,
        },
        Scenario {
            name: "Analog sample (bloom off)",
            theme: Some(ClusterTheme::PorscheAnalog),
            bloom: false,
            extra: Extra::None,
        },
        Scenario {
            name: "Glass plate, own layers only",
            theme: None,
            bloom: true,
            extra: Extra::GlassOwn,
        },
        Scenario {
            name: "Glass plate + refraction source",
            theme: None,
            bloom: true,
            extra: Extra::GlassWorld,
        },
        Scenario {
            name: "Analog sample + fluid (256x128, 20 Jacobi)",
            theme: Some(ClusterTheme::PorscheAnalog),
            bloom: true,
            extra: Extra::Fluid,
        },
        Scenario {
            name: "Empty frame (hidden)",
            theme: None,
            bloom: true,
            extra: Extra::Empty,
        },
    ];

    let mut rows = Vec::new();
    let mut fluid_note: Option<Stats> = None;
    for sc in &scenarios {
        kit.set_bloom(sc.bloom);
        kit.set_refraction_source(if sc.extra == Extra::GlassWorld {
            ctx.queue.write_texture(
                fh6_kit::wgpu::TexelCopyTextureInfo {
                    texture: &world_tex,
                    mip_level: 0,
                    origin: fh6_kit::wgpu::Origin3d::ZERO,
                    aspect: fh6_kit::wgpu::TextureAspect::All,
                },
                &world_pixels,
                fh6_kit::wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(world_size.0 * 4),
                    rows_per_image: Some(world_size.1),
                },
                fh6_kit::wgpu::Extent3d {
                    width: world_size.0,
                    height: world_size.1,
                    depth_or_array_layers: 1,
                },
            );
            Some((world_view.clone(), world_rect))
        } else {
            None
        });
        let entry = sc.theme.map(fh6_clusters::registry::resolve);
        let palette = Palette::family(&tokens, entry.map_or("liquidGlass", |e| e.family));
        kit.set_palette(palette);
        let mut cluster: Option<Box<dyn Cluster>> =
            entry.map(|e| (e.make)(&mut kit.setup(&ctx.device, &ctx.queue, None)));
        let mut smoother = SignalSmoother::new();
        let (mut rec, mut ren, mut gpu) = (Vec::new(), Vec::new(), Vec::new());
        let mut fluid_ms = Vec::new();
        let mut sub = Vec::new();
        let mut last = fh6_kit::FrameStats::default();
        for f in 0..WARMUP + frames {
            let t0 = Instant::now();
            let data = telemetry(f);
            let signals = smoother.update(&data, &cfg, &tokens, DT);
            if let Some(c) = cluster.as_mut() {
                c.update(&FrameInput {
                    telemetry: &data,
                    signals: &signals,
                    dt: DT,
                    t: f as f32 * DT,
                    config: &cfg,
                    palette: &palette,
                    tokens: &tokens,
                    fonts,
                    logo: &logo,
                    view: View::Cockpit,
                    reduced_flash: false,
                });
            }
            {
                let mut p = kit.begin_frame(&layout);
                if let Some(c) = cluster.as_ref() {
                    c.draw(&mut p, &layout);
                }
                let plate = Rect::from_xywh(-520.0, -250.0, 1040.0, 486.0);
                match sc.extra {
                    Extra::GlassOwn | Extra::GlassWorld => {
                        p.fill(
                            &Shape::circle(vec2(-200.0, 0.0), 220.0),
                            palette.primary.with_alpha(0.6),
                        );
                        p.layer(LayerOptions::glass(18.0), |p| {
                            p.fill(
                                &Shape::round_rect(plate, 64.0),
                                Glass {
                                    refraction: Refraction::World,
                                    ..tokens.glass(palette.glass_tint)
                                },
                            );
                        });
                    }
                    Extra::Fluid => {
                        let fr = Rect::from_xywh(-500.0, -240.0, 1000.0, 460.0);
                        p.fill(&Shape::round_rect(fr, 48.0), ImagePaint::new(dye, fr));
                    }
                    Extra::None | Extra::Empty => {}
                }
            }
            let t_recorded = Instant::now();
            if sc.extra == Extra::Fluid {
                let a = f as f32 * 0.05;
                fluid.splat(
                    vec2(0.5 + 0.3 * a.cos(), 0.5 + 0.3 * a.sin()),
                    vec2(-a.sin(), a.cos()) * 0.6,
                    Color::rgb_u32(0x38BDF8),
                    0.06,
                );
                // The solver in its own submission, timed to GPU idle (an upper bound).
                let tf = Instant::now();
                fluid.step_submit(&ctx.device, &ctx.queue, DT);
                let _ = ctx.device.poll(fh6_kit::wgpu::PollType::Wait {
                    submission_index: None,
                    timeout: Some(std::time::Duration::from_secs(5)),
                });
                if f >= WARMUP {
                    fluid_ms.push(tf.elapsed().as_secs_f64() * 1e3);
                }
            }
            let t1 = Instant::now();
            let mut encoder = ctx.device.create_command_encoder(&Default::default());
            last = kit.render(
                &ctx.device,
                &ctx.queue,
                &mut encoder,
                target.view(),
                OFFSCREEN_FORMAT,
                (W, H),
                1.0,
            );
            let t2 = Instant::now();
            ctx.queue.submit(Some(encoder.finish()));
            let t3 = Instant::now();
            let g = kit.gpu_time(&ctx.device);
            if f >= WARMUP {
                rec.push((t_recorded - t0).as_secs_f64() * 1e3);
                ren.push((t2 - t1).as_secs_f64() * 1e3);
                sub.push((t3 - t2).as_secs_f64() * 1e3);
                if let Some(g) = g {
                    gpu.push(g.as_secs_f64() * 1e3);
                }
            }
        }
        let s_sub = stats(&mut sub);
        println!(
            "  finish + submit: p50 {:.3} p99 {:.3} ms, {} passes",
            s_sub.p50, s_sub.p99, last.passes
        );
        if !fluid_ms.is_empty() {
            let s = stats(&mut fluid_ms);
            println!(
                "  fluid step alone (submit to GPU idle): p50 {:.3} p99 {:.3} ms",
                s.p50, s.p99
            );
            fluid_note = Some(s);
        }
        if ctx.has_failed() {
            return Err(format!("wgpu error in scenario {}", sc.name).into());
        }
        let row = Row {
            name: sc.name.to_string(),
            record: stats(&mut rec),
            render: stats(&mut ren),
            gpu: stats(&mut gpu),
            draws: last.draw_calls,
            instances: last.instances,
            work: last.work_pixels,
        };
        println!(
            "{:<46} record p50 {:.3} p99 {:.3} | render p50 {:.3} p99 {:.3} | GPU p50 {:.3} p99 {:.3} ms | {} draws",
            row.name,
            row.record.p50,
            row.record.p99,
            row.render.p50,
            row.render.p99,
            row.gpu.p50,
            row.gpu.p99,
            row.draws
        );
        rows.push(row);
        drop(cluster.take());
    }

    let mut report = report(&adapter, frames, &rows);
    if let Some(s) = fluid_note {
        let _ = writeln!(
            report,
            "
Fluid solver step alone (256x128, 20 Jacobi iterations, vorticity), its own submission timed from submit to GPU idle (an upper bound that includes submission overhead): p50 {:.3} ms, p99 {:.3} ms. It is not inside the GPU column above.",
            s.p50, s.p99
        );
    }
    if let Some(path) = out {
        std::fs::write(&path, report)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}

fn report(adapter: &str, frames: usize, rows: &[Row]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "## Offscreen kit benchmark (generated by `bench-render`)\n");
    let _ = writeln!(
        s,
        "3840x2160, {frames} frames per scenario after {WARMUP} warm-up frames, adapter: {adapter}. Times in ms. \
`record` = cluster `update` + `draw` into the arena; `render` = `Kit::render` (pack, one upload, pass recording); \
GPU = timestamp queries around the whole frame (all passes, including the resolve).\n"
    );
    let _ = writeln!(
        s,
        "| Scenario | record p50 / p99 | render p50 / p99 | GPU p50 / p99 / max | draws | instances | work px |"
    );
    let _ = writeln!(s, "|---|---|---|---|---|---|---|");
    for r in rows {
        let _ = writeln!(
            s,
            "| {} | {:.3} / {:.3} | {:.3} / {:.3} | {:.3} / {:.3} / {:.3} | {} | {} | {} |",
            r.name,
            r.record.p50,
            r.record.p99,
            r.render.p50,
            r.render.p99,
            r.gpu.p50,
            r.gpu.p99,
            r.gpu.max,
            r.draws,
            r.instances,
            r.work
        );
    }
    s
}
