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
//! - `index.html`: the product page of the dash (no script): the cockpit view, the motion sequence
//!   as a looping reel, every state, the measured badges and the GPU p50/p99.
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
/// Frames timed once the picture is complete; the GPU time reported is their median, so a
/// one-off cost (the first sampling of a new 4K texture, another process on the GPU) does not
/// stand in for the frame time.
const TIMED_FRAMES: usize = 5;

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
    /// When timing, the complete frame is drawn [`TIMED_FRAMES`] times and the median GPU time
    /// is kept.
    fn render(&mut self, layout: &Layout, mut draw: impl FnMut(&mut fh6_kit::Painter<'_>)) -> Result<Shot> {
        let mut last: Option<Shot> = None;
        let mut times = [0.0_f64; TIMED_FRAMES];
        let mut timed = 0;
        for _ in 0..MAX_GLYPH_PASSES + TIMED_FRAMES {
            let missing = {
                draw(&mut self.kit.begin_frame(layout));
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
                match gpu_ms {
                    Some(ms) if timed < TIMED_FRAMES => {
                        times[timed] = ms;
                        timed += 1;
                        if timed == TIMED_FRAMES {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        }
        if self.gpu.has_failed() {
            return Err("wgpu reported an error".into());
        }
        let mut shot = last.ok_or("no frame rendered")?;
        if timed > 0 {
            let times = &mut times[..timed];
            times.sort_by(f64::total_cmp);
            shot.gpu_ms = Some(times[timed / 2]);
        }
        Ok(shot)
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

/// The pixels of the rank-4 figure the colour budget leaves out: only for a Psychedelic or
/// Abstract dash, and only when the dash names its figure zone (CATALOG.md, colour law point 4).
pub fn figure_pixels(cat: &CatalogEntry, cluster: &dyn Cluster, layout: &Layout) -> Option<crate::gallery::PixelRect> {
    if !cat.category.pattern_first() {
        return None;
    }
    let zone = cluster.figure_zone()?;
    Some(crate::gallery::zone_pixels(layout, zone, W, H))
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
    logo: &Logo,
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
        custom_driver_name: crate::DRIVER_NAME.to_string(),
        ..Default::default()
    };
    let palette = Palette::family(tokens, entry.family.key());
    kit.set_palette(palette);
    let fonts = kit.font_choice(None);
    let photo = background(root, "cockpit_with_wheel.jpg").ok();
    let mut sh = Shooter {
        gpu,
        kit,
        target: Offscreen::new(&gpu.device, W, H),
        timing,
    };
    println!(
        "{} ({}, {}, {})",
        cat.display,
        cat.family.label(),
        cat.views.label(),
        theme
    );

    let mut rows = Vec::new();
    for (view, prefix) in views(cat) {
        let layout = Layout::from_config(&cfg, tokens, view, (W, H));
        let crop = envelope_crop(&layout, 48.0);
        let ctx = DriveContext {
            config: &cfg,
            palette: &palette,
            tokens,
            fonts,
            logo,
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
            let figure = figure_pixels(cat, cluster.as_ref(), &layout);
            let badges = Badges::compute(&shot.img, &palette, tokens, &shot.stats, shot.gpu_ms, figure);
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
        logo,
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

/// The look of the page: the tokens of `docs/design/CONFIGURATOR_DIRECTION.md` (the page grows
/// into the catalog the configurator serves). No script; the motion reel is a CSS animation.
const PAGE_CSS: &str = "\
@font-face{font-family:'Titillium Web';src:url('../../assets/fonts/titillium-web/TitilliumWeb-Regular.ttf')}\
@font-face{font-family:'Titillium Web';font-weight:600;src:url('../../assets/fonts/titillium-web/TitilliumWeb-SemiBold.ttf')}\
@font-face{font-family:'Michroma';src:url('../../assets/fonts/michroma/Michroma-Regular.ttf')}\
@font-face{font-family:'Saira';font-weight:100 900;src:url('../../assets/fonts/saira/Saira%5Bwdth,wght%5D.ttf')}\
:root{--page:#05070A;--panel:#0B1018;--line:#1E293B;--ink:#F4F7FB;--muted:#94A3B8;--label:#64748B;--accent:#38BDF8;--ok:#2BE37A;--bad:#FF5A5F}\
*{box-sizing:border-box}\
body{margin:0;background:var(--page);color:var(--ink);font:16px/1.55 'Titillium Web',system-ui,sans-serif}\
a{color:inherit}\
img{display:block;max-width:100%}\
.bar{display:flex;gap:20px;align-items:center;padding:18px 40px;border-bottom:1px solid var(--line)}\
.mark{font:13px 'Michroma',sans-serif;letter-spacing:.32em}\
.crumb{color:var(--muted);font-size:14px}\
.pill{margin-left:auto;border:1px solid var(--line);border-radius:999px;padding:4px 14px;color:var(--accent);font-size:13px;letter-spacing:.08em;text-transform:uppercase}\
main{max-width:1680px;margin:0 auto;padding:0 40px 80px}\
section{margin-top:56px}\
.eyebrow,h2,dt,th{color:var(--label);font-size:12px;font-weight:600;letter-spacing:.24em;text-transform:uppercase}\
h1{font:300 72px/1 'Saira',sans-serif;margin:10px 0 18px;letter-spacing:.01em}\
h2{margin:0 0 6px}\
h3{font:400 22px 'Saira',sans-serif;margin:28px 0 12px}\
.hero{display:grid;grid-template-columns:minmax(320px,5fr) 8fr;gap:48px;align-items:center}\
.lede{color:var(--muted);font-size:18px;max-width:62ch;margin:0}\
.lede code,.note code{font-family:ui-monospace,Consolas,monospace;font-size:.86em;color:var(--ink)}\
.facts{display:flex;flex-wrap:wrap;gap:14px 40px;margin:28px 0 0}\
.facts div{min-width:120px}\
dd{margin:2px 0 0;font:400 28px 'Saira',sans-serif}\
dd.sub{font:14px 'Titillium Web',sans-serif;color:var(--muted)}\
.stage,.reel,figure a{display:block;background:#000;border:1px solid var(--line);border-radius:14px;overflow:hidden}\
.note{color:var(--muted);max-width:80ch;margin:0 0 18px}\
.reel{position:relative}\
.reel img{position:absolute;inset:0;width:100%;opacity:0;animation:reel 4s steps(1,end) infinite;animation-delay:calc(var(--i)*4s/60)}\
.reel img:first-child{position:relative;opacity:1}\
@keyframes reel{0%{opacity:1}1.6667%{opacity:0}100%{opacity:0}}\
@media (prefers-reduced-motion:reduce){.reel img{animation:none}}\
.grid{display:grid;grid-template-columns:repeat(2,1fr);gap:20px}\
figure{margin:0}\
figcaption{display:flex;justify-content:space-between;padding:10px 4px 0;color:var(--muted)}\
figcaption b{color:var(--ink);font-weight:600}\
details{background:var(--panel);border:1px solid var(--line);border-radius:14px;padding:14px 20px}\
summary{cursor:pointer;color:var(--muted)}\
table{border-collapse:collapse;margin-top:12px;width:100%;font-size:14px}\
td,th{border-top:1px solid var(--line);padding:7px 10px;text-align:right}\
td:first-child,th:first-child{text-align:left}\
.ok{color:var(--ok)}.bad{color:var(--bad)}\
@media (max-width:1200px){.hero,.grid{grid-template-columns:1fr}h1{font-size:52px}}";

/// Escapes text for HTML and renders the inline Markdown the dash READMEs use: `[text](target)`
/// keeps its text, `` `code` `` becomes `<code>`, and `**` marks are dropped.
fn inline_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len() + 16);
    let mut rest = md;
    let mut code = false;
    while let Some(c) = rest.chars().next() {
        let mut step = c.len_utf8();
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '`' => {
                out.push_str(if code { "</code>" } else { "<code>" });
                code = !code;
            }
            '*' if !code && rest.starts_with("**") => step = 2,
            '[' if !code => match rest
                .find("](")
                .and_then(|mid| rest[mid..].find(')').map(|end| (mid, mid + end)))
                // A bracket that opens before the link text is not part of the link.
                .filter(|(mid, _)| !rest[1..*mid].contains('['))
            {
                Some((mid, end)) => {
                    out.push_str(&inline_html(&rest[1..mid]));
                    step = end + 1;
                }
                None => out.push('['),
            },
            _ => out.push(c),
        }
        rest = &rest[step..];
    }
    if code {
        out.push_str("</code>");
    }
    out
}

/// The first paragraph under `## What it is` in a dash's README, as HTML.
fn lede(readme: &str) -> Option<String> {
    let para: Vec<&str> = readme
        .lines()
        .skip_while(|l| l.trim() != "## What it is")
        .skip(1)
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .collect();
    (!para.is_empty()).then(|| inline_html(&para.join(" ")))
}

/// A row label such as `cockpit · 02_mid_cruise` as the view and a readable state name:
/// `("Cockpit", "Mid cruise")`.
fn pretty(label: &str) -> (String, String) {
    let capital = |s: &str| {
        let mut c = s.chars();
        c.next()
            .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
    };
    let (view, state) = label.split_once(" · ").unwrap_or(("", label));
    let state = state.trim_start_matches(|c: char| c.is_ascii_digit() || c == '_');
    (capital(view), capital(&state.replace('_', " ")))
}

/// Writes `index.html`: the dash's product page. A hero with the cockpit view and the measured
/// numbers, the motion sequence as a looping reel, every state as a card, and the review data.
fn write_page(
    dir: &Path,
    cat: &CatalogEntry,
    rows: &[Row],
    p50: Option<f64>,
    p99: Option<f64>,
    adapter: &str,
) -> std::io::Result<()> {
    let ms = |v: Option<f64>| v.map_or_else(|| "n/a".to_string(), |v| format!("{v:.3} ms"));
    let readme = fs::read_to_string(Path::new(&cat.spec_path()).with_file_name("README.md")).unwrap_or_default();
    let lede = lede(&readme).unwrap_or_default();
    // The hero is the driver's view of mid cruise when there is one, else the first render.
    let hero = rows
        .iter()
        .find(|r| r.cockpit.is_some() && r.label.ends_with("mid_cruise"))
        .or_else(|| rows.iter().find(|r| r.cockpit.is_some()))
        .and_then(|r| r.cockpit.clone())
        .or_else(|| rows.first().map(|r| r.full.clone()))
        .unwrap_or_default();
    let name = inline_html(cat.display);
    let mut h = String::new();
    let _ = write!(
        h,
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{name} · FH6 WheelDash</title><style>{PAGE_CSS}</style></head><body>\
         <header class=\"bar\"><span class=\"mark\">FH6 WHEELDASH</span>\
         <span class=\"crumb\">Dash catalogue · {category} · catalog row {row}</span><span class=\"pill\">{family}</span></header><main>\
         <section class=\"hero\"><div><p class=\"eyebrow\">{category} · {family} · {views}</p><h1>{name}</h1>\
         <p class=\"lede\">{lede}</p><dl class=\"facts\">\
         <div><dt>GPU at 4K</dt><dd>{p50}</dd><dd class=\"sub\">p99 {p99} · budget 0.5 ms</dd></div>\
         <div><dt>Views</dt><dd>{views}</dd></div>\
         <div><dt>Catalog row</dt><dd>{row}</dd><dd class=\"sub\">{category} · shelf {shelf} · {target}</dd></div>\
         <div><dt>Config value</dt><dd>{theme}</dd></div></dl></div>\
         <a class=\"stage\" href=\"{hero}\"><img src=\"_thumbs/{hero}\" alt=\"{name} as the driver sees it\"></a></section>",
        row = cat.row,
        category = cat.category.label(),
        shelf = cat.shelf,
        target = cat.target.label(),
        family = cat.family.label(),
        views = cat.views.label(),
        theme = cat.theme.name(),
        p50 = ms(p50),
        p99 = ms(p99),
    );
    h.push_str(
        "<section><h2>In motion</h2><p class=\"note\">The review sequence at quarter speed: idle, a pull to the \
         limiter, the limiter strobe, the upshift and the next pull. These are the 60 frames the overlay draws in \
         one second.</p><div class=\"reel\">",
    );
    for i in 0..SEQUENCE_FRAMES {
        let _ = write!(
            h,
            "<img style=\"--i:{i}\" src=\"sequence/frame_{i:02}.png\" alt=\"{}\">",
            if i == 0 { "the motion sequence" } else { "" }
        );
    }
    h.push_str("</div></section><section><h2>States</h2>");
    let mut open_view = String::new();
    for r in rows {
        let (view, state) = pretty(&r.label);
        if view != open_view {
            if !open_view.is_empty() {
                h.push_str("</div>");
            }
            let _ = write!(h, "<h3>{view}</h3><div class=\"grid\">");
            open_view = view;
        }
        let _ = write!(
            h,
            "<figure><a href=\"{full}\"><img src=\"{crop}\" alt=\"{state}\"></a>\
             <figcaption><b>{state}</b><span>{gpu} GPU</span></figcaption></figure>",
            full = r.cockpit.as_deref().unwrap_or(&r.full),
            crop = r.crop,
            gpu = ms(r.badges.gpu_ms),
        );
    }
    if !open_view.is_empty() {
        h.push_str("</div>");
    }
    h.push_str(
        "</section><section><h2>Review data</h2><details><summary>Measured from each render</summary><table>\
         <tr><th>State</th><th>GPU</th><th>CPU</th><th>Draws</th><th>Instances</th><th>Base hue</th><th>White</th>\
         <th>Red</th><th>Black outside</th></tr>",
    );
    for r in rows {
        let b = &r.badges;
        let flag = |ok: bool| if ok { "ok" } else { "bad" };
        let (view, state) = pretty(&r.label);
        let _ = write!(
            h,
            "<tr><td>{view} · {state}</td><td>{gpu}</td><td>{cpu:.3} ms</td><td>{draws}</td><td>{inst}</td>\
             <td class=\"{bf}\">{base}</td><td class=\"{wf}\">{white}</td><td>{red}</td>\
             <td class=\"{of}\">{outside}</td></tr>",
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
        );
    }
    let budget = if !cat.category.pattern_first() {
        ""
    } else if rows.iter().any(|r| r.badges.outside_figure) {
        " Base hue, white and red are measured outside the rank-4 figure, whose hue follows its model \
         (the colour law of CATALOG.md)."
    } else {
        " Flag: this dash names no figure zone (<code>Cluster::figure_zone</code>), so base hue, white and red \
         are measured on the whole frame, figure included."
    };
    let _ = writeln!(
        h,
        "</table></details><p class=\"note\" style=\"margin-top:18px\">Brief: <code>{spec}</code>. Rendered on \
         {adapter} at 3840 × 2160.{budget}</p></section></main></body></html>",
        spec = cat.spec_path(),
        adapter = inline_html(adapter),
    );
    fs::write(dir.join("index.html"), h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readme_text_becomes_safe_html() {
        assert_eq!(inline_html("a < b & c"), "a &lt; b &amp; c");
        assert_eq!(
            inline_html("see [the spec](../x.md), `Kit<T>`"),
            "see the spec, <code>Kit&lt;T&gt;</code>"
        );
        assert_eq!(inline_html("**bold** [not a link"), "bold [not a link");
        assert_eq!(inline_html("`open"), "<code>open</code>");
        assert_eq!(inline_html("[a] then [b](x)"), "[a] then b");
    }

    #[test]
    fn the_lede_is_the_first_paragraph_of_what_it_is() {
        let readme = "# Dash\n\nStatus.\n\n## What it is\nGlass, `OpticalGlass`,\nand light.\n\n## Next\nMore.\n";
        assert_eq!(
            lede(readme).as_deref(),
            Some("Glass, <code>OpticalGlass</code>, and light.")
        );
        assert_eq!(lede("# Dash\n\nNo such section.\n"), None);
    }

    #[test]
    fn row_labels_read_as_names() {
        assert_eq!(
            pretty("cockpit · 02_mid_cruise"),
            ("Cockpit".into(), "Mid cruise".into())
        );
        assert_eq!(
            pretty("hood · 03_redline_strobe"),
            ("Hood".into(), "Redline strobe".into())
        );
        assert_eq!(pretty("plain"), (String::new(), "Plain".into()));
    }
}
