//! Golden-image tests: every registered kit cluster, in two review states, rendered on the GPU
//! and compared perceptually with `tests/golden/images/<Cluster>_<state>.png`.
//!
//! Run with `FH6_UPDATE_GOLDEN=1` to (re)write the goldens after an intended visual change;
//! review the new PNGs before committing them.
#![cfg(not(target_arch = "wasm32"))]

use fh6_clusters::registry::REGISTRY;
use fh6_core::{AppConfig, FH6TelemetryData};
use fh6_gpu::GpuContext;
use fh6_kit::testing::{compare, Offscreen, RgbaImage, Tolerance, OFFSCREEN_FORMAT};
use fh6_kit::{FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use std::path::PathBuf;

const W: u32 = 1920;
const H: u32 = 1080;
const DT: f32 = 1.0 / 60.0;

fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/images")
}

/// Cruise and drift/braking review states (tokens `states`), limiter 8700 rpm.
fn state(name: &str) -> FH6TelemetryData {
    let (rpm, gear, mph, thr, brk, glat, glong, slip): (f32, u8, f32, f32, f32, f32, f32, f32) = match name {
        "cruise" => (5200.0, 4, 118.0, 0.42, 0.0, 0.18, 0.08, 1.0),
        _ => (7100.0, 3, 74.0, 0.30, 0.62, -0.96, -0.58, 27.0),
    };
    let v = mph / 2.236_936_3;
    let s = slip.to_radians();
    FH6TelemetryData {
        is_race_on: 1,
        engine_max_rpm: 8700.0,
        engine_idle_rpm: 900.0,
        current_engine_rpm: rpm,
        speed_ms: v,
        velocity_x: v * s.sin(),
        velocity_z: v * s.cos(),
        acceleration_x: glat * 9.80665,
        acceleration_z: glong * 9.80665,
        gear,
        accel: (thr * 255.0) as u8,
        brake: (brk * 255.0) as u8,
        ..Default::default()
    }
}

#[test]
fn kit_clusters_match_their_goldens() {
    let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
    // The goldens bind to the repo tokens, so a token edit shows up as a visual diff.
    let tokens_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/figma_tokens.json");
    let tokens = Tokens::load(&tokens_path).expect("data/figma_tokens.json");
    let mut kit = Kit::new(&ctx.device, &ctx.queue, tokens.clone()).expect("kit");
    let cfg = AppConfig::default();
    let layout = Layout::from_config(&cfg, &tokens, View::Cockpit, (W, H));
    let target = Offscreen::new(&ctx.device, W, H);
    let logo = Logo {
        custom_name: "TJX".into(),
        ..Logo::default()
    };
    let fonts = kit.font_choice(None);
    let update = std::env::var_os("FH6_UPDATE_GOLDEN").is_some();
    let mut failures = Vec::new();
    for entry in REGISTRY {
        for state_name in ["cruise", "drift"] {
            let palette = Palette::family(&tokens, entry.family);
            kit.set_palette(palette);
            let mut cluster = (entry.make)(&mut kit.setup(&ctx.device, &ctx.queue, None));
            let data = state(state_name);
            let mut smoother = SignalSmoother::new();
            for i in 0..=30 {
                let signals = smoother.update(&data, &cfg, &tokens, DT);
                cluster.update(&FrameInput {
                    telemetry: &data,
                    signals: &signals,
                    dt: DT,
                    t: i as f32 * DT,
                    config: &cfg,
                    palette: &palette,
                    tokens: &tokens,
                    fonts,
                    logo: &logo,
                    view: View::Cockpit,
                    reduced_flash: false,
                });
            }
            let mut img = None;
            // Two frames: glyphs first used in frame one are in the atlas by frame two.
            for _ in 0..2 {
                {
                    let mut p = kit.begin_frame(&layout);
                    cluster.draw(&mut p, &layout);
                }
                let mut encoder = ctx.device.create_command_encoder(&Default::default());
                kit.render(
                    &ctx.device,
                    &ctx.queue,
                    &mut encoder,
                    target.view(),
                    OFFSCREEN_FORMAT,
                    (W, H),
                    1.0,
                );
                target.copy_out(&mut encoder);
                ctx.queue.submit(Some(encoder.finish()));
                img = Some(target.read(&ctx.device).expect("readback"));
            }
            assert!(!ctx.has_failed(), "wgpu error rendering {}", entry.theme);
            let img: RgbaImage = img.expect("rendered").crop(560, 700, 800, 380).over([0, 0, 0]);
            let name = format!("{}_{state_name}.png", entry.theme);
            let path = golden_dir().join(&name);
            if update || !path.exists() {
                if !update {
                    failures.push(format!("{name}: golden missing (run with FH6_UPDATE_GOLDEN=1)"));
                    continue;
                }
                img.save_png(&path).expect("write golden");
                continue;
            }
            let golden = RgbaImage::load_png(&path).expect("read golden");
            let diff = compare(&img, &golden, Tolerance::default());
            if !diff.pass {
                let actual = golden_dir().join(format!("{}_{state_name}.actual.png", entry.theme));
                let _ = img.save_png(&actual);
                failures.push(format!(
                    "{name}: {:.2}% of pixels differ (mean dE {:.2}, max {:.1}); wrote {}",
                    diff.fraction_over * 100.0,
                    diff.mean,
                    diff.max,
                    actual.display()
                ));
            }
        }
    }
    assert!(failures.is_empty(), "golden mismatches:\n{}", failures.join("\n"));
}
