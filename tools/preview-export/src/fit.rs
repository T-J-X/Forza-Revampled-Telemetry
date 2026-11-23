//! `--fit`: the cockpit fit review (Phase 8). Composites the kit scrim, its lip wash and a
//! cluster over `assets/reference/cockpit_with_wheel.jpg` upscaled to 4K, exactly as the overlay
//! would sit over the game frame:
//! 3 clusters (Liquid Glass and Analog, the two kit clusters so far, plus Analog at user scale
//! 1.25 to check that the dome scales with the cluster)
//! x 3 scrim opacities (0, 50, 100%) x 2 cluster heights (0 and +170 px)
//! -> `mockups/fit/<cluster>_scrim<pct>_h<height>.png` plus a crop around the dome, a thumbnail
//! and a script-free `mockups/fit/index.html`. Two extra frames put Liquid Glass over the
//! low-seat, wide-FOV target view (`target_wide_cockpit_fov.jpg`).
//!
//! It also renders the kit scrim alone at 0/50/100% over black into `Scrim/OemScrim_<pct>pct.png`
//! (replacing the v1 tiny-skia scrim renders).
use super::states::STATES;
use super::{Result, DT, H, W};
use fh6_clusters::registry;
use fh6_core::{AppConfig, ClusterTheme, FitView, ScrimFit};
use fh6_gpu::GpuContext;
use fh6_kit::scrim::{self, Wash};
use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
use fh6_kit::{FrameInput, Kit, Layout, Logo, Palette, SignalSmoother, Tokens, View};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Scrim opacities reviewed.
pub const OPACITIES: [i32; 3] = [0, 50, 100];
/// Cluster heights reviewed (px at 4K, positive raises).
pub const HEIGHTS: [f32; 2] = [0.0, 170.0];

/// Name, cluster and user scale (`ClusterScale`).
const CLUSTERS: [(&str, ClusterTheme, f32); 3] = [
    ("LiquidGlass", ClusterTheme::LiquidGlass, 1.0),
    ("Analog", ClusterTheme::PorscheAnalog, 1.0),
    ("Analog_x1.25", ClusterTheme::PorscheAnalog, 1.25),
];

/// Decodes a reference photo and resizes it to 4K (Catmull-Rom), as opaque RGBA8.
pub(crate) fn background(root: &Path, file: &str) -> Result<RgbaImage> {
    let path = root.join("assets/reference").join(file);
    let img = image::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rgba = img
        .resize_exact(W, H, image::imageops::FilterType::CatmullRom)
        .to_rgba8();
    Ok(RgbaImage {
        width: W,
        height: H,
        pixels: rgba.into_raw(),
    })
}

/// Premultiplied `top` over opaque `bottom`, in encoded space (what DWM does).
pub(crate) fn composite(top: &RgbaImage, bottom: &RgbaImage) -> RgbaImage {
    let mut out = bottom.clone();
    for (o, t) in out.pixels.chunks_exact_mut(4).zip(top.pixels.chunks_exact(4)) {
        let a = f32::from(t[3]) / 255.0;
        for c in 0..3 {
            o[c] = (f32::from(t[c]) + f32::from(o[c]) * (1.0 - a))
                .round()
                .clamp(0.0, 255.0) as u8;
        }
        o[3] = 255;
    }
    out
}

struct Ctx<'a> {
    gpu: &'a GpuContext,
    kit: &'a mut Kit,
    tokens: &'a Tokens,
    target: &'a Offscreen,
    assets: &'a Path,
}

impl Ctx<'_> {
    /// One kit frame: scrim + wash, then (optionally) a kit cluster warmed up on `state`.
    fn frame(&mut self, cfg: &AppConfig, cluster: Option<ClusterTheme>, frames: u32) -> Result<RgbaImage> {
        let layout = Layout::from_config(cfg, self.tokens, View::Cockpit, (W, H));
        let fit = ScrimFit::from_config(cfg, FitView::Cockpit);
        let family = cluster.map_or("liquidGlass", |t| registry::resolve(t).family);
        let palette = Palette::family(self.tokens, family);
        self.kit.set_palette(palette);
        let wash = Wash::from_config(cfg, palette.primary);
        let mut built = match cluster {
            Some(theme) => {
                let entry = registry::resolve(theme);
                let mut setup = self.kit.setup(&self.gpu.device, &self.gpu.queue, Some(self.assets));
                Some((entry.make)(&mut setup))
            }
            None => None,
        };
        if let Some(c) = built.as_mut() {
            let logo = Logo {
                image: None,
                brand_name: String::new(),
                custom_name: cfg.custom_driver_name.clone(),
            };
            let fonts = self.kit.font_choice(None);
            let data = STATES[1].telemetry();
            let mut smoother = SignalSmoother::new();
            let mut t = 0.0;
            for _ in 0..=frames {
                let signals = smoother.update(&data, cfg, self.tokens, DT);
                c.update(&FrameInput {
                    telemetry: &data,
                    signals: &signals,
                    dt: DT,
                    t,
                    config: cfg,
                    palette: &palette,
                    tokens: self.tokens,
                    fonts,
                    logo: &logo,
                    view: View::Cockpit,
                    reduced_flash: false,
                });
                t += DT;
            }
        }
        let mut img = None;
        // Twice: glyphs queued on the first frame appear on the second.
        for _ in 0..2 {
            {
                let mut p = self.kit.begin_frame(&layout);
                scrim::draw_scrim(&mut p, &fit);
                scrim::draw_wash(&mut p, &fit, wash);
                if let Some(c) = built.as_ref() {
                    c.draw(&mut p, &layout);
                }
            }
            let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
            self.kit.render(
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
            img = Some(self.target.read(&self.gpu.device)?);
        }
        if self.gpu.has_failed() {
            return Err("wgpu reported an error in the fit render".into());
        }
        img.ok_or_else(|| "no frame".into())
    }
}

fn config(pct: i32, height: f32, scale: f32) -> AppConfig {
    AppConfig {
        current_camera_index: 3,
        custom_driver_name: "TJX".to_string(),
        oem_dash_opaque_pct: pct,
        cockpit_height_offset_px: height,
        cluster_scale: scale,
        ambient_glow_enabled: true,
        ..AppConfig::default()
    }
}

/// Renders the kit scrim alone over black (the gallery's scrim cards).
pub fn scrims(gpu: &GpuContext, kit: &mut Kit, tokens: &Tokens, root: &Path, out: &Path) -> Result<()> {
    let target = Offscreen::new(&gpu.device, W, H);
    let assets = root.join("assets");
    let mut ctx = Ctx {
        gpu,
        kit,
        tokens,
        target: &target,
        assets: &assets,
    };
    for pct in OPACITIES {
        let img = ctx.frame(&config(pct, 0.0, 1.0), None, 0)?;
        img.over([0, 0, 0])
            .save_png(&out.join(format!("Scrim/OemScrim_{pct}pct.png")))?;
    }
    Ok(())
}

/// Renders, saves (full, crop, thumb) and adds a gallery card.
#[allow(clippy::too_many_arguments)]
fn shoot(
    ctx: &mut Ctx<'_>,
    dir: &Path,
    bg: &RgbaImage,
    cfg: &AppConfig,
    theme: ClusterTheme,
    stem: &str,
    caption: &str,
    frames: u32,
    cards: &mut String,
) -> Result<()> {
    let overlay = ctx.frame(cfg, Some(theme), frames)?;
    let shown = composite(&overlay, bg);
    shown.save_png(&dir.join(format!("{stem}.png")))?;
    let layout = Layout::from_config(cfg, ctx.tokens, View::Cockpit, (W, H));
    let origin = layout.to_target(fh6_kit::vec2(0.0, -75.0));
    let (cx, cy) = (origin.x.max(0.0) as u32, origin.y.max(0.0) as u32);
    let crop = shown.crop(
        cx.saturating_sub(1000),
        cy.saturating_sub(620).min(H - 1125),
        2000,
        1125,
    );
    crop.save_png(&dir.join(format!("{stem}_crop.png")))?;
    crop.downsample(2).save_png(&dir.join(format!("{stem}_thumb.png")))?;
    let _ = write!(
        cards,
        "<figure><a href=\"{stem}.png\"><img src=\"{stem}_thumb.png\" alt=\"{stem}\"></a>\
         <figcaption>{caption}</figcaption></figure>"
    );
    println!("fit {stem}");
    Ok(())
}

/// The full fit review.
pub fn run(gpu: &GpuContext, kit: &mut Kit, tokens: &Tokens, root: &Path, out: &Path, frames: u32) -> Result<()> {
    let bg = background(root, "cockpit_with_wheel.jpg")?;
    let low_seat = background(root, "target_wide_cockpit_fov.jpg")?;
    let dir = out.join("fit");
    fs::create_dir_all(&dir)?;
    let target = Offscreen::new(&gpu.device, W, H);
    let assets = root.join("assets");
    let mut ctx = Ctx {
        gpu,
        kit,
        tokens,
        target: &target,
        assets: &assets,
    };
    let mut cards = String::new();
    for (name, theme, scale) in CLUSTERS {
        for pct in OPACITIES {
            for height in HEIGHTS {
                let cfg = config(pct, height, scale);
                let stem = format!("{name}_scrim{pct:03}_h{:+04}", height as i32);
                let caption = format!("{name} · scrim {pct}% · height {:+} px", height as i32);
                shoot(&mut ctx, &dir, &bg, &cfg, theme, &stem, &caption, frames, &mut cards)?;
            }
        }
    }
    // The low-seat, wide-FOV target view (SCRIM_FIT §5), scrim 100%.
    for height in HEIGHTS {
        let cfg = config(100, height, 1.0);
        let stem = format!("LowSeat_LiquidGlass_scrim100_h{:+04}", height as i32);
        let caption = format!(
            "Low seat target view · Liquid Glass · scrim 100% · height {:+} px",
            height as i32
        );
        shoot(
            &mut ctx,
            &dir,
            &low_seat,
            &cfg,
            ClusterTheme::LiquidGlass,
            &stem,
            &caption,
            frames,
            &mut cards,
        )?;
    }
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>FH6 cockpit fit</title>\
         <style>body{{background:#000;color:#ddd;font:14px Bahnschrift,sans-serif;padding:24px}}\
         main{{display:grid;grid-template-columns:repeat(auto-fill,minmax(480px,1fr));gap:16px}}\
         img{{width:100%;display:block}}figure{{margin:0}}</style></head><body>\
         <h1>Cockpit fit: scrim × cluster × height</h1>\
         <p>Over <code>cockpit_with_wheel.jpg</code> (and the low-seat target view) at 4K. Crops are \
         2000×1125 around the dome centre; click for the full 4K frame.</p>\
         <main>{cards}</main></body></html>"
    );
    fs::write(dir.join("index.html"), html)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_is_premultiplied_over() {
        let top = RgbaImage {
            width: 1,
            height: 1,
            pixels: vec![0, 0, 0, 128],
        };
        let bottom = RgbaImage {
            width: 1,
            height: 1,
            pixels: vec![200, 100, 50, 255],
        };
        assert_eq!(composite(&top, &bottom).pixels, vec![100, 50, 25, 255]);
    }
}
