//! `--cluster <Name>`: the review set a Phase 7 cluster chat iterates on, in `mockups/<Theme>/`.
//!
//! For one catalog cluster (theme name, module name or display name) it writes:
//! - `<state>.png` (and `hood_<state>.png` when the catalog lists a hood view): the 4 review
//!   states at 3840x2160 over OLED black, warmed up through `update` exactly as the overlay runs;
//! - `<state>_crop.png`: a 1:1 crop of the view envelope;
//! - `<state>_cockpit.png`: the cockpit view over `assets/reference/cockpit_with_wheel.jpg` with
//!   the OEM scrim (100%) and wash, as the driver sees it;
//! - `_thumbs/*.png`: 960x540 box-downsamples;
//! - `sequence/frame_NN.png`: the 60-frame motion sequence (idle → redline strobe → gear change,
//!   `fh6_clusters::review::sequence_frame`) as 1:1 envelope crops, with GPU time per frame;
//! - `index.html`: a script-free page with every image, the measured badges and the GPU p50/p99.
use crate::fit::{background, composite};
use crate::gallery::Badges;
use crate::{Result, DT, H, THUMB_FACTOR, W};
use fh6_clusters::catalog::{self, CatalogEntry};
use fh6_clusters::registry;
use fh6_clusters::review::{sequence_frame, DriveContext, Driver, SEQUENCE_DT, SEQUENCE_FRAMES, STATES};
use fh6_core::{AppConfig, FitView, ScrimFit};
use fh6_gpu::GpuContext;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{Cluster, FrameStats, Kit, Layout, Logo, Palette, Tokens, View};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Most frames a render is repeated while glyphs are still being added to the atlas.
const MAX_GLYPH_PASSES: usize = 16;

/// The GPU, kit and target for one export.
struct Shooter<'a> {
    gpu: &'a GpuContext,
    kit: &'a mut Kit,
    target: Offscreen,
    timing: bool,
}

/// One rendered frame.
struct Shot {
    img: RgbaImage,
    stats: FrameStats,
    gpu_ms: Option<f64>,
}

impl Shooter<'_> {
    /// Renders `draw` under `layout`, repeating the frame until every glyph is in the atlas.
    fn render(&mut self, layout: &Layout, mut draw: impl FnMut(&mut fh6_kit::Painter<'_>)) -> Result<Shot> {
        let mut last = None;
        for _ in 0..MAX_GLYPH_PASSES {
            let missing = {
                let mut p = self.kit.begin_frame(layout);
                draw(&mut p);
                drop(p);
                self.kit.scene().missing_glyph_count()
            };
            let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
            let stats = self.kit.render(
                &self.gpu.device,
                &self.gpu.queue,
                &mut encoder,
                self.target.view(),
                OFFSCREEN_FORMAT,
                (W, H),
                1.0,
            );
            self.target.copy_out(&mut encoder);
            self.gpu.queue.submit(Some(encoder.finish()));
            let img = self.target.read(&self.gpu.device)?;
            let gpu_ms = if self.timing {
                self.kit.gpu_time(&self.gpu.device).map(|d| d.as_secs_f64() * 1000.0)
            } else {
                None
            };
            last = Some(Shot { img, stats, gpu_ms });
            if missing == 0 {
                break;
            }
        }
        if self.gpu.has_failed() {
            return Err("wgpu reported an error".into());
        }
        last.ok_or_else(|| "no frame rendered".into())
    }

    /// A fresh cluster for `entry`.
    fn build(&mut self, entry: &fh6_kit::cluster::ClusterEntry, assets: &Path) -> Box<dyn Cluster> {
        let mut setup = self.kit.setup(&self.gpu.device, &self.gpu.queue, Some(assets));
        (entry.make)(&mut setup)
    }
}

/// One image row in the page.
struct Row {
    label: String,
    full: String,
    crop: String,
    cockpit: Option<String>,
    badges: Badges,
}

/// The views to export for a catalog row (raised HUDs draw in the cockpit layout).
fn views(e: &CatalogEntry) -> Vec<(View, &'static str)> {
    let mut v = Vec::new();
    if e.views.cockpit || e.views.raised_hud {
        v.push((View::Cockpit, ""));
    }
    if e.views.hood {
        v.push((View::Hood, "hood_"));
    }
    v
}

/// The layout's envelope in target pixels, grown by `margin`.
fn envelope_crop(layout: &Layout, margin: f32) -> (u32, u32, u32, u32) {
    let a = layout.to_target(layout.envelope.min);
    let b = layout.to_target(layout.envelope.max);
    let x0 = (a.x.min(b.x) - margin).max(0.0).floor() as u32;
    let y0 = (a.y.min(b.y) - margin).max(0.0).floor() as u32;
    let x1 = (a.x.max(b.x) + margin).min(W as f32).ceil() as u32;
    let y1 = (a.y.max(b.y) + margin).min(H as f32).ceil() as u32;
    (x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1))
}

fn percentile(sorted: &[f64], q: f64) -> Option<f64> {
    if sorted.is_empty() {
        return None;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted.get(i).copied()
}

/// Runs the per-cluster export for `name`.
#[allow(clippy::too_many_arguments)]
pub fn run(
    gpu: &GpuContext,
    kit: &mut Kit,
    tokens: &Tokens,
    root: &Path,
    out: &Path,
    name: &str,
    frames: u32,
    timing: bool,
) -> Result<()> {
    let Some(cat) = catalog::find(name) else {
        let mut names = String::new();
        for e in &catalog::CATALOG {
            let _ = write!(names, " {}", e.theme);
        }
        return Err(format!("unknown cluster {name:?}; one of:{names}").into());
    };
    let entry = registry::entry(cat.theme).ok_or_else(|| format!("{} is not registered", cat.theme))?;
    let theme = cat.theme.to_string();
    let dir = out.join(&theme);
    fs::create_dir_all(dir.join("_thumbs"))?;
    fs::create_dir_all(dir.join("sequence"))?;
    let assets = root.join("assets");
    let cfg = AppConfig {
        current_camera_index: 3,
        custom_driver_name: "TJX".to_string(),
        ..Default::default()
    };
    let logo = Logo {
        image: None,
        brand_name: String::new(),
        custom_name: cfg.custom_driver_name.clone(),
    };
    let palette = Palette::family(tokens, entry.family);
    kit.set_palette(palette);
    let fonts = kit.font_choice(None);
    let photo = background(root, "cockpit_with_wheel.jpg").ok();
    let mut sh = Shooter {
        gpu,
        kit,
        target: Offscreen::new(&gpu.device, W, H),
        timing,
    };
    println!("{} ({}, {}, {})", cat.display, cat.family.label(), cat.views.label(), theme);

    let mut rows = Vec::new();
    for (view, prefix) in views(cat) {
        let layout = Layout::from_config(&cfg, tokens, view, (W, H));
        let crop = envelope_crop(&layout, 48.0);
        let ctx = DriveContext {
            config: &cfg,
            palette: &palette,
            tokens,
            fonts,
            logo: &logo,
            view,
            reduced_flash: false,
        };
        for state in &STATES {
            let started = Instant::now();
            let mut cluster = sh.build(entry, &assets);
            Driver::new().settle(cluster.as_mut(), state, &ctx, frames, DT);
            let shot = sh.render(&layout, |p| cluster.draw(p, &layout))?;
            let stem = format!("{prefix}{}", state.name);
            let shown = shot.img.over([0, 0, 0]);
            shown.save_png(&dir.join(format!("{stem}.png")))?;
            shown
                .downsample(THUMB_FACTOR)
                .save_png(&dir.join("_thumbs").join(format!("{stem}.png")))?;
            shown
                .crop(crop.0, crop.1, crop.2, crop.3)
                .save_png(&dir.join(format!("{stem}_crop.png")))?;
            let mut cockpit = None;
            if let (View::Cockpit, Some(bg)) = (view, photo.as_ref()) {
                // The driver's view: OEM scrim at 100% and the wash under the cluster.
                let fit = ScrimFit::from_config(&cfg, FitView::Cockpit);
                let wash = Wash::from_config(&cfg, palette.primary);
                let over = sh.render(&layout, |p| {
                    scrim::draw_scrim(p, &fit);
                    scrim::draw_wash(p, &fit, wash);
                    cluster.draw(p, &layout);
                })?;
                let file = format!("{stem}_cockpit.png");
                let shown = composite(&over.img, bg);
                shown.save_png(&dir.join(&file))?;
                shown
                    .downsample(THUMB_FACTOR)
                    .save_png(&dir.join("_thumbs").join(&file))?;
                cockpit = Some(file);
            }
            let badges = Badges::compute(&shot.img, &palette, tokens, &shot.stats, shot.gpu_ms);
            println!(
                "  {stem:<24} {:>6.3} ms GPU  {:>3} draws  {:>4} instances  ({:.1} s)",
                shot.gpu_ms.unwrap_or(f64::NAN),
                shot.stats.draw_calls,
                shot.stats.instances,
                started.elapsed().as_secs_f32()
            );
            if badges.covered == 0 {
                return Err(format!("{theme} {stem} rendered nothing").into());
            }
            rows.push(Row {
                label: format!(
                    "{} · {}",
                    if view == View::Hood { "hood" } else { "cockpit" },
                    state.name
                ),
                full: format!("{stem}.png"),
                crop: format!("{stem}_crop.png"),
                cockpit,
                badges,
            });
        }
    }

    // The motion sequence, in the first view the cluster is designed for.
    let (seq_view, _) = views(cat).first().copied().unwrap_or((View::Cockpit, ""));
    let layout = Layout::from_config(&cfg, tokens, seq_view, (W, H));
    let crop = envelope_crop(&layout, 48.0);
    let ctx = DriveContext {
        config: &cfg,
        palette: &palette,
        tokens,
        fonts,
        logo: &logo,
        view: seq_view,
        reduced_flash: false,
    };
    let mut cluster = sh.build(entry, &assets);
    let mut driver = Driver::new();
    let mut gpu_times = Vec::with_capacity(SEQUENCE_FRAMES as usize);
    for i in 0..SEQUENCE_FRAMES {
        driver.step(cluster.as_mut(), &sequence_frame(i), &ctx, SEQUENCE_DT);
        let shot = sh.render(&layout, |p| cluster.draw(p, &layout))?;
        if let Some(ms) = shot.gpu_ms {
            gpu_times.push(ms);
        }
        shot.img
            .over([0, 0, 0])
            .crop(crop.0, crop.1, crop.2, crop.3)
            .save_png(&dir.join("sequence").join(format!("frame_{i:02}.png")))?;
    }
    gpu_times.sort_by(f64::total_cmp);
    let p50 = percentile(&gpu_times, 0.5);
    let p99 = percentile(&gpu_times, 0.99);
    println!(
        "  sequence: {SEQUENCE_FRAMES} frames, GPU p50 {:.3} ms, p99 {:.3} ms",
        p50.unwrap_or(f64::NAN),
        p99.unwrap_or(f64::NAN)
    );
    write_page(&dir, cat, &rows, p50, p99, gpu.adapter_summary().to_string().as_str())?;
    println!("Cluster review written to {}", dir.display());
    Ok(())
}

fn pct(v: f32) -> String {
    format!("{:.0}%", v * 100.0)
}

fn write_page(
    dir: &Path,
    cat: &CatalogEntry,
    rows: &[Row],
    p50: Option<f64>,
    p99: Option<f64>,
    adapter: &str,
) -> std::io::Result<()> {
    let ms = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |v| format!("{v:.3} ms"));
    let mut h = String::new();
    let _ = write!(
        h,
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{name} review</title>\
         <style>body{{background:#000;color:#cbd5e1;font:14px system-ui,sans-serif;margin:24px}}\
         h1{{font-weight:500;letter-spacing:.04em}}figure{{margin:0 0 28px}}img{{max-width:100%;display:block;border:1px solid #1e293b}}\
         .row{{display:grid;grid-template-columns:1fr 1fr;gap:12px}}table{{border-collapse:collapse;margin:8px 0}}\
         td,th{{border:1px solid #1e293b;padding:3px 8px;text-align:right}}.bad{{color:#ff5a5f}}.ok{{color:#2be37a}}\
         .seq{{display:grid;grid-template-columns:repeat(6,1fr);gap:4px}}.seq img{{border:0}}</style></head><body>\
         <h1>{name} <small>· {family} · {views} · build #{order}</small></h1>\
         <p>Brief: <code>{spec}</code>. GPU: {adapter}. Sequence GPU p50 <b>{p50}</b>, p99 <b>{p99}</b> (budget 0.5 ms).</p>",
        name = cat.display,
        family = cat.family.label(),
        views = cat.views.label(),
        order = cat.build_order,
        spec = cat.spec_path(),
        p50 = ms(p50),
        p99 = ms(p99),
    );
    for r in rows {
        let b = &r.badges;
        let flag = |ok: bool| if ok { "ok" } else { "bad" };
        let _ = write!(
            h,
            "<figure><h3>{label}</h3><table><tr><th>GPU</th><th>CPU</th><th>draws</th><th>instances</th>\
             <th>base hue</th><th>white</th><th>red</th><th>black outside</th></tr><tr><td>{gpu}</td><td>{cpu:.3} ms</td>\
             <td>{draws}</td><td>{inst}</td><td class=\"{bf}\">{base}</td><td class=\"{wf}\">{white}</td>\
             <td>{red}</td><td class=\"{of}\">{outside}</td></tr></table><div class=\"row\"><a href=\"{full}\"><img src=\"{crop}\" alt=\"{label} crop\"></a>",
            label = r.label,
            gpu = ms(b.gpu_ms),
            cpu = b.cpu_ms,
            draws = b.draws,
            inst = b.instances,
            base = pct(b.base_share),
            white = pct(b.white_share),
            red = pct(b.red_share),
            outside = b.black_outside,
            bf = flag(b.base_share >= 0.8),
            wf = flag(b.white_share <= 0.15),
            of = flag(b.black_outside),
            full = r.full,
            crop = r.crop,
        );
        if let Some(c) = &r.cockpit {
            let _ = write!(h, "<a href=\"{c}\"><img src=\"_thumbs/{c}\" alt=\"{} in the cockpit\"></a>", r.label);
        }
        h.push_str("</div></figure>");
    }
    h.push_str("<h2>Sequence: idle → redline strobe → gear change (60 frames, 60 Hz)</h2><div class=\"seq\">");
    for i in 0..SEQUENCE_FRAMES {
        let _ = write!(h, "<img src=\"sequence/frame_{i:02}.png\" alt=\"frame {i}\">");
    }
    h.push_str("</div></body></html>\n");
    fs::write(dir.join("index.html"), h)
}
