//! Badges computed from each render, and the script-free `index.html` gallery.
use fh6_kit::color::{srgb_to_linear, Oklab, Oklch};
use fh6_kit::testing::RgbaImage;
use fh6_kit::{FrameStats, Palette, Tokens};
use std::fmt::Write as _;
use std::path::Path;

/// Numbers measured on one render (nothing here is hardcoded).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Badges {
    /// GPU time of the frame (timestamp queries), when the device supports them.
    pub gpu_ms: Option<f64>,
    /// CPU time of `Kit::render`.
    pub cpu_ms: f64,
    /// Draw calls.
    pub draws: u32,
    /// Shape instances.
    pub instances: u32,
    /// Lit pixels (alpha > 8).
    pub covered: u64,
    /// Share of lit pixels in the palette's base hue (tokens budget: >= 80%).
    pub base_share: f32,
    /// Share of white / near-white lit pixels (budget: <= 15%).
    pub white_share: f32,
    /// Share of red lit pixels (budget: <= 5% except at the limiter).
    pub red_share: f32,
    /// Every pixel outside the cluster is fully transparent (OLED true black).
    pub black_outside: bool,
    /// The shares above leave out the dash's rank-4 figure (a Psychedelic or Abstract dash that
    /// names its figure zone: CATALOG.md, colour law point 4).
    pub outside_figure: bool,
}

/// A pixel rect `[x0, y0, x1, y1)` of the render.
pub type PixelRect = [u32; 4];

/// The pixel rect of a local-px zone under `layout`, clamped to a `w` x `h` render.
#[must_use]
pub fn zone_pixels(layout: &fh6_kit::Layout, zone: fh6_kit::Rect, w: u32, h: u32) -> PixelRect {
    let a = layout.to_target(zone.min);
    let b = layout.to_target(zone.max);
    let clamp = |v: f32, hi: u32| (v.max(0.0) as u32).min(hi);
    [
        clamp(a.x.min(b.x).floor(), w),
        clamp(a.y.min(b.y).floor(), h),
        clamp(a.x.max(b.x).ceil(), w),
        clamp(a.y.max(b.y).ceil(), h),
    ]
}

impl Badges {
    /// Measures `img` (premultiplied sRGB, as presented). The colour shares leave out the pixels
    /// of `figure` when it is given; `covered` counts every lit pixel.
    #[must_use]
    pub fn compute(
        img: &RgbaImage,
        palette: &Palette,
        tokens: &Tokens,
        stats: &FrameStats,
        gpu_ms: Option<f64>,
        figure: Option<PixelRect>,
    ) -> Self {
        let base_h = palette.primary.to_oklch().h;
        let red_h = tokens.roles.warning.to_oklch().h;
        let width = img.width.max(1) as usize;
        let inside = |i: usize| {
            figure.is_some_and(|[x0, y0, x1, y1]| {
                let (x, y) = ((i % width) as u32, (i / width) as u32);
                (x0..x1).contains(&x) && (y0..y1).contains(&y)
            })
        };
        let (mut covered, mut lit, mut base, mut white, mut red) = (0u64, 0u64, 0u64, 0u64, 0u64);
        for (i, p) in img.pixels.chunks_exact(4).enumerate() {
            if p[3] <= 8 {
                continue;
            }
            covered += 1;
            if inside(i) {
                continue;
            }
            lit += 1;
            let a = f32::from(p[3]) / 255.0;
            let lin = |c: u8| srgb_to_linear((f32::from(c) / 255.0 / a).min(1.0));
            let c = Oklab::from_linear_rgb(lin(p[0]), lin(p[1]), lin(p[2])).to_oklch();
            if c.c < 0.04 {
                if c.l > 0.75 {
                    white += 1;
                }
            } else if Oklch::hue_distance(c.h, red_h) < 20.0 && c.c > 0.08 {
                red += 1;
            } else if Oklch::hue_distance(c.h, base_h) < 30.0 {
                base += 1;
            }
        }
        let share = |n: u64| if lit == 0 { 0.0 } else { n as f32 / lit as f32 };
        let corners = [
            (0, 0),
            (img.width - 1, 0),
            (0, img.height - 1),
            (img.width - 1, img.height - 1),
        ];
        Self {
            gpu_ms,
            cpu_ms: stats.cpu.as_secs_f64() * 1000.0,
            draws: stats.draw_calls,
            instances: stats.instances,
            covered,
            base_share: share(base),
            white_share: share(white),
            red_share: share(red),
            black_outside: corners
                .iter()
                .all(|(x, y)| img.pixel(*x, *y).is_some_and(|p| p == [0, 0, 0, 0])),
            outside_figure: figure.is_some(),
        }
    }
}

/// One exported render.
#[derive(Debug, Clone)]
pub struct Render {
    /// Theme identifier.
    pub cluster: String,
    /// Display name.
    pub display: String,
    /// Review state.
    pub state: String,
    /// Path of the 4K PNG relative to the gallery.
    pub full: String,
    /// Path of the thumbnail relative to the gallery.
    pub thumb: String,
    /// Measurements.
    pub badges: Badges,
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Writes `index.html`: no script; full-size views open as plain links.
pub fn write_index(
    out: &Path,
    renders: &[Render],
    osd: &[crate::osd::OsdRender],
    adapter: &str,
) -> std::io::Result<()> {
    let mut h = String::new();
    let _ = write!(
        h,
        r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><title>FH6 WheelDash kit renders</title>
<style>
:root{{color-scheme:dark}}
body{{background:#05070b;color:#e5e7eb;font:14px/1.45 system-ui,Segoe UI,sans-serif;margin:0;padding:24px}}
h1{{margin:0 0 4px;font-size:22px}} h2{{margin:28px 0 10px;font-size:16px;color:#cbd5e1}}
.meta{{color:#94a3b8;margin-bottom:16px}}
.grid{{display:grid;grid-template-columns:repeat(auto-fill,minmax(420px,1fr));gap:14px}}
.card{{background:#0b1220;border:1px solid #1e293b;border-radius:10px;overflow:hidden}}
.card img{{display:block;width:100%;height:auto;background:#000}}
.cap{{padding:8px 10px;font-size:12px;color:#cbd5e1}}
.b{{display:inline-block;margin:2px 4px 0 0;padding:2px 6px;border-radius:6px;background:#111827;border:1px solid #334155;font-size:11px;color:#e2e8f0}}
.ok{{border-color:#14532d;color:#86efac}} .warn{{border-color:#713f12;color:#fde68a}}
</style></head><body>
<h1>FH6 WheelDash: fh6-kit renders</h1>
<div class="meta">3840x2160 per frame, over OLED black. GPU: {}. Badges are measured on each render.</div>
"#,
        esc(adapter)
    );
    let mut current = String::new();
    for r in renders {
        if r.cluster != current {
            if !current.is_empty() {
                h.push_str("</div>\n");
            }
            let _ = writeln!(
                h,
                "<h2>{} <span class=\"b\">{}</span></h2><div class=\"grid\">",
                esc(&r.display),
                esc(&r.cluster)
            );
            current.clone_from(&r.cluster);
        }
        let b = &r.badges;
        let budget = |ok: bool| if ok { "b ok" } else { "b warn" };
        let gpu = b
            .gpu_ms
            .map_or_else(|| "GPU n/a".to_string(), |g| format!("GPU {g:.3} ms"));
        let _ = writeln!(
            h,
            "<div class=\"card\"><a href=\"{full}\"><img src=\"{thumb}\" alt=\"{name} {state}\"></a><div class=\"cap\">{state}<br>\
<span class=\"b\">{gpu}</span><span class=\"b\">CPU {cpu:.3} ms</span><span class=\"b\">{draws} draws</span><span class=\"b\">{inst} instances</span>\
<span class=\"{bcls}\">base hue {base:.0}%</span><span class=\"{wcls}\">white {white:.0}%</span><span class=\"{rcls}\">red {red:.1}%</span>\
<span class=\"{kcls}\">outside {kres}</span>{figure}</div></div>\n",
            full = esc(&r.full),
            thumb = esc(&r.thumb),
            name = esc(&r.cluster),
            state = esc(&r.state),
            cpu = b.cpu_ms,
            draws = b.draws,
            inst = b.instances,
            bcls = budget(b.base_share >= 0.8),
            base = b.base_share * 100.0,
            wcls = budget(b.white_share <= 0.15),
            white = b.white_share * 100.0,
            rcls = budget(b.red_share <= 0.05 || r.state.contains("redline")),
            red = b.red_share * 100.0,
            kcls = budget(b.black_outside),
            kres = if b.black_outside { "#000 transparent" } else { "NOT transparent" },
            figure = if b.outside_figure {
                "<span class=\"b\">hue shares outside the figure</span>"
            } else {
                ""
            },
        );
    }
    if !current.is_empty() {
        h.push_str("</div>\n");
    }
    h.push_str(
        "<h2>OSD (kit, Phase 9)</h2><div class=\"meta\">Whole-frame GPU time (median; live cluster \
and thumbnails included; budget 1.5 ms). Files ending <code>_over_game.png</code> show the frame \
over a game capture when exported with <code>--backdrop</code>.</div><div class=\"grid\">\n",
    );
    for r in osd {
        let gpu = match (r.gpu_ms, r.gpu_min_ms) {
            (Some(m), Some(lo)) => format!("{m:.3} ms (min {lo:.3})"),
            _ => "n/a".to_string(),
        };
        let cls = if r.gpu_min_ms.is_some_and(|g| g < 1.5) {
            "ok"
        } else {
            "warn"
        };
        let _ = write!(
            h,
            "<div class=\"card\"><a href=\"{full}\"><img src=\"{thumb}\" alt=\"{name}\"></a><div class=\"cap\">{cap}<br>\
<span class=\"b {cls}\">GPU {gpu}</span><span class=\"b\">{draws} draws</span></div></div>",
            full = r.full,
            thumb = r.thumb,
            name = r.name,
            cap = esc(r.caption),
            draws = r.draws,
        );
    }
    h.push_str("</div>\n");
    h.push_str(
        "<h2>OEM scrim</h2><div class=\"grid\">\
<div class=\"card\"><a href=\"Scrim/OemScrim_0pct.png\"><img src=\"Scrim/OemScrim_0pct.png\" alt=\"scrim 0%\"></a><div class=\"cap\">Scrim 0%</div></div>\
<div class=\"card\"><a href=\"Scrim/OemScrim_50pct.png\"><img src=\"Scrim/OemScrim_50pct.png\" alt=\"scrim 50%\"></a><div class=\"cap\">Scrim 50%</div></div>\
<div class=\"card\"><a href=\"Scrim/OemScrim_100pct.png\"><img src=\"Scrim/OemScrim_100pct.png\" alt=\"scrim 100%\"></a><div class=\"cap\">Scrim 100%</div></div></div>\n",
    );
    h.push_str("</body></html>\n");
    std::fs::write(out.join("index.html"), h)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Four lit pixels in a row: white at x 0 and 3, saturated red at x 1 and 2.
    fn strip() -> RgbaImage {
        let mut pixels = Vec::new();
        for px in [
            [255, 255, 255, 255],
            [255, 0, 0, 255],
            [255, 0, 0, 255],
            [255, 255, 255, 255],
        ] {
            pixels.extend_from_slice(&px);
        }
        RgbaImage {
            width: 4,
            height: 1,
            pixels,
        }
    }

    #[test]
    fn the_figure_rect_is_left_out_of_the_shares_but_counted_as_lit() {
        let tokens = Tokens::default();
        let palette = Palette::family(&tokens, "fluidGenerative");
        let stats = FrameStats::default();
        let img = strip();
        let whole = Badges::compute(&img, &palette, &tokens, &stats, None, None);
        assert_eq!(whole.covered, 4);
        assert!(whole.red_share > 0.4, "{}", whole.red_share);
        assert!(!whole.outside_figure);
        // [x0, x1) = [1, 3): both red pixels, neither white one.
        let outside = Badges::compute(&img, &palette, &tokens, &stats, None, Some([1, 0, 3, 1]));
        assert_eq!(outside.covered, 4);
        assert!(outside.red_share.abs() < 1e-6, "{}", outside.red_share);
        assert!((outside.white_share - 1.0).abs() < 1e-6, "{}", outside.white_share);
        assert!(outside.outside_figure);
        // One pixel narrower on the right keeps x 2 in the shares.
        let narrow = Badges::compute(&img, &palette, &tokens, &stats, None, Some([1, 0, 2, 1]));
        assert!((narrow.red_share - 1.0 / 3.0).abs() < 1e-6, "{}", narrow.red_share);
    }
}
