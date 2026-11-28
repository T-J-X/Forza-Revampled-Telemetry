//! Shared GPU harness for the integration tests (goldens and component shots): one headless
//! device and kit per test binary, bound to the repo tokens and the Phase 6 font pack.
#![allow(dead_code)]

use fh6_clusters::review::{DriveContext, Driver, ReviewState};
use fh6_core::AppConfig;
use fh6_gpu::GpuContext;
use fh6_kit::cluster::ClusterEntry;
use fh6_kit::testing::{compare, Offscreen, RgbaImage, Tolerance, OFFSCREEN_FORMAT};
use fh6_kit::{Kit, Layout, Logo, Painter, Palette, Rect, Tokens, View};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

/// Golden and shot resolution (half of 4K; layouts scale with the target height).
pub const W: u32 = 1920;
/// See [`W`].
pub const H: u32 = 1080;
/// Frame time used to settle clusters.
pub const DT: f32 = 1.0 / 60.0;

/// The repository root.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// `tests/golden/images/`.
pub fn golden_dir() -> PathBuf {
    repo_root().join("tests/golden/images")
}

/// The device, kit and tokens shared by every test in one binary.
pub struct Gpu {
    /// Headless DX12 (or WARP) device.
    pub ctx: GpuContext,
    /// The kit, fonts loaded.
    pub kit: Kit,
    /// Repo tokens.
    pub tokens: Tokens,
    /// `assets/`.
    pub assets: PathBuf,
}

static GPU: OnceLock<Mutex<Gpu>> = OnceLock::new();

/// Locks the shared GPU (tests in one binary run one at a time on it).
pub fn gpu() -> MutexGuard<'static, Gpu> {
    let cell = GPU.get_or_init(|| {
        let ctx = GpuContext::headless().expect("a DX12 adapter or WARP");
        // The goldens bind to the repo tokens, so a token edit shows up as a visual diff.
        let tokens = Tokens::load(&repo_root().join("data/design_tokens.json")).expect("data/design_tokens.json");
        let mut kit = Kit::new(&ctx.device, &ctx.queue, tokens.clone()).expect("kit");
        let assets = repo_root().join("assets");
        let fonts = assets.join("fonts");
        if fonts.is_dir() {
            kit.fonts_mut().load_dir(&fonts).expect("assets/fonts");
            kit.bind_font_roles();
        }
        Mutex::new(Gpu {
            ctx,
            kit,
            tokens,
            assets,
        })
    });
    // A failed test poisons the lock; the GPU state is still usable for the next test.
    cell.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Gpu {
    /// Renders whatever `draw` records under `layout` into a `W` x `H` image (repeating the frame
    /// until every glyph is in the atlas).
    pub fn render(&mut self, layout: &Layout, mut draw: impl FnMut(&mut Painter<'_>, &Layout)) -> RgbaImage {
        let target = Offscreen::new(&self.ctx.device, layout.target.0, layout.target.1);
        let mut img = None;
        for _ in 0..16 {
            let missing = {
                draw(&mut self.kit.begin_frame(layout), layout);
                self.kit.scene().missing_glyph_count()
            };
            let mut encoder = self.ctx.device.create_command_encoder(&Default::default());
            self.kit.render(
                &self.ctx.device,
                &self.ctx.queue,
                &mut encoder,
                target.view(),
                OFFSCREEN_FORMAT,
                layout.target,
                1.0,
            );
            target.copy_out(&mut encoder);
            self.ctx.queue.submit(Some(encoder.finish()));
            img = Some(target.read(&self.ctx.device).expect("readback"));
            if missing == 0 {
                break;
            }
        }
        assert!(!self.ctx.has_failed(), "wgpu reported an error");
        img.expect("rendered")
    }

    /// Builds `entry`'s cluster, settles it on `state` in `view` and renders it at `W` x `H`,
    /// cropped to the view envelope plus a margin.
    pub fn render_cluster(&mut self, entry: &ClusterEntry, view: View, state: &ReviewState) -> RgbaImage {
        // A set driver name, so PorscheAnalog's parked greeting does not follow the default.
        let cfg = AppConfig {
            custom_driver_name: "TJX".into(),
            ..AppConfig::default()
        };
        let layout = Layout::from_config(&cfg, &self.tokens, view, (W, H));
        let palette = Palette::family(&self.tokens, entry.family.key());
        self.kit.set_palette(palette);
        let assets = self.assets.clone();
        let mut cluster = {
            let mut setup = self.kit.setup(&self.ctx.device, &self.ctx.queue, Some(&assets));
            (entry.make)(&mut setup)
        };
        let logo = Logo {
            custom_name: "TJX".into(),
            ..Logo::default()
        };
        let tokens = self.tokens.clone();
        let ctx = DriveContext {
            config: &cfg,
            palette: &palette,
            tokens: &tokens,
            fonts: self.kit.font_choice(None),
            logo: &logo,
            view,
            reduced_flash: false,
        };
        Driver::new().settle(cluster.as_mut(), state, &ctx, 30, DT);
        let img = self.render(&layout, |p, l| cluster.draw(p, l));
        let crop = envelope_crop(&layout, 24.0);
        img.crop(crop.0, crop.1, crop.2, crop.3).over([0, 0, 0])
    }
}

/// The layout's envelope in target pixels, grown by `margin`, clamped to the target.
pub fn envelope_crop(layout: &Layout, margin: f32) -> (u32, u32, u32, u32) {
    let env = layout.envelope;
    let a = layout.to_target(env.min);
    let b = layout.to_target(env.max);
    let r = Rect::from_points(a, b);
    let x0 = (r.min.x - margin).max(0.0).floor() as u32;
    let y0 = (r.min.y - margin).max(0.0).floor() as u32;
    let x1 = (r.max.x + margin).min(layout.target.0 as f32).ceil() as u32;
    let y1 = (r.max.y + margin).min(layout.target.1 as f32).ceil() as u32;
    (x0, y0, x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1))
}

/// Compares `img` with `tests/golden/images/<name>.png`; with `FH6_UPDATE_GOLDEN` set (or no
/// golden yet while updating) it writes the golden instead. Returns the failure text, if any.
pub fn check_golden(name: &str, img: &RgbaImage) -> Option<String> {
    let path = golden_dir().join(format!("{name}.png"));
    let update = std::env::var_os("FH6_UPDATE_GOLDEN").is_some();
    if update {
        return img
            .save_png(&path)
            .err()
            .map(|e| format!("{name}: writing the golden failed: {e}"));
    }
    if !path.exists() {
        return Some(format!("{name}: golden missing (run with FH6_UPDATE_GOLDEN=1)"));
    }
    let golden = match RgbaImage::load_png(&path) {
        Ok(g) => g,
        Err(e) => return Some(format!("{name}: reading the golden failed: {e}")),
    };
    let diff = compare(img, &golden, Tolerance::default());
    if diff.pass {
        let _ = std::fs::remove_file(golden_dir().join(format!("{name}.actual.png")));
        return None;
    }
    let actual = golden_dir().join(format!("{name}.actual.png"));
    let _ = img.save_png(&actual);
    Some(format!(
        "{name}: {:.2}% of pixels differ (mean dE {:.2}, max {:.1}); wrote {}",
        diff.fraction_over * 100.0,
        diff.mean,
        diff.max,
        actual.display()
    ))
}

/// Writes a component shot to `target/component-shots/<name>.png` for visual review and returns
/// its path.
pub fn save_shot(name: &str, img: &RgbaImage) -> PathBuf {
    let dir = repo_root().join("target/component-shots");
    let path = dir.join(format!("{name}.png"));
    img.over([0, 0, 0]).save_png(&path).expect("write shot");
    path
}

/// A centred layout for component shots: anchor in the middle of a `W` x `H` target at 4K
/// authoring scale (local units = 4K px).
pub fn shot_layout() -> Layout {
    Layout::centered(fh6_kit::vec2(W as f32 * 0.5, H as f32 * 0.5), H as f32 / 2160.0, (W, H))
}

/// True if `path` exists (helper for tests that want an optional asset).
pub fn exists(path: &Path) -> bool {
    path.exists()
}
