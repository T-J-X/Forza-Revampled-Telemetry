//! Glue between a render loop and the OSD: navigation state, motion, the cockpit-fit backend
//! and the carousel's live thumbnail clusters. The overlay and `preview-export` both drive it.
//!
//! Once, when the kit is ready: [`OsdHost::load_assets`], so the first open does not decode the
//! studio and the driving-position layers mid-session. After the kit is replaced (a lost
//! device), [`OsdHost::device_changed`] and then `load_assets` again.
//!
//! Per frame, while the OSD is open:
//! 1. [`OsdHost::step`] with the controller input (edits land in the working config).
//! 2. [`OsdHost::prepare`] before `Kit::begin_frame` (creates thumbnail clusters on a
//!    carousel change; nothing on other frames).
//! 3. [`OsdHost::update`] with the frame input the live cluster got.
//! 4. `kit.begin_frame(&host.live_layout(..))`, then [`draw::backdrop`], the live cluster,
//!    then [`OsdHost::draw`].
use crate::anim::OsdAnim;
use crate::draw::{self, DrawInput, Frame, LiveStatus, Thumb};
use crate::drive::{self, DriveImages};
use crate::fit::CockpitFit;
use crate::model::{self, Tab};
use crate::nav::{Mode, NavInput, NavOutput, OsdController};
use fh6_clusters::clusters::liquid_glass;
use fh6_core::{AppConfig, ClusterTheme};
use fh6_kit::{wgpu, Cluster, FrameInput, ImageId, Kit, Layout, Painter, Tokens, View};
use std::path::Path;
use std::time::Duration;

/// Thumbnail clusters kept alive (the five shown plus a few recent ones).
const THUMB_CACHE: usize = 9;

/// The OSD as a render loop sees it.
pub struct OsdHost {
    /// Navigation and editing state.
    pub ctl: OsdController,
    /// Motion.
    pub anim: OsdAnim,
    fit: Box<dyn CockpitFit>,
    thumbs: Vec<(ClusterTheme, Box<dyn Cluster>)>,
    order: [Option<ClusterTheme>; 5],
    last_tab: Tab,
    /// The studio the OSD's glass mirrors (the Liquid Glass cluster's, so both look like one
    /// material); `None` falls back to the kit's analytic studio.
    env: Option<ImageId>,
    /// The driving position's rendered layers; `None` draws its line drawing.
    drive: Option<DriveImages>,
    /// The studio and the driving position have been loaded ([`OsdHost::load_assets`]), found
    /// or not: a missing file is not looked for again every frame.
    loaded: bool,
}

impl std::fmt::Debug for OsdHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OsdHost")
            .field("tab", &self.ctl.tab())
            .field("mode", &self.ctl.mode())
            .field("thumbs", &self.thumbs.len())
            .finish_non_exhaustive()
    }
}

impl OsdHost {
    /// A closed OSD using `fit` for camera changes and the scrim rows.
    #[must_use]
    pub fn new(tokens: &Tokens, fit: Box<dyn CockpitFit>) -> Self {
        Self {
            ctl: OsdController::new(),
            anim: OsdAnim::new(tokens),
            fit,
            thumbs: Vec::with_capacity(THUMB_CACHE + 1),
            order: [None; 5],
            last_tab: Tab::Cluster,
            env: None,
            drive: None,
            loaded: false,
        }
    }

    /// The cockpit-fit backend.
    #[must_use]
    pub fn fit(&self) -> &dyn CockpitFit {
        &*self.fit
    }

    /// Jumps to `tab` and focuses `row` there (previews, tests); false when the tab does not
    /// show that row with this config.
    pub fn focus_row(&mut self, tab: Tab, row: crate::model::RowId, cfg: &AppConfig) -> bool {
        self.ctl.focus_row(tab, row, cfg, &*self.fit)
    }

    /// Whether a dash-blocker row has focus (the live view then shows the real blocker under
    /// the cluster, see [`draw::live_backing`]).
    #[must_use]
    pub fn blocker_focused(&self, cfg: &AppConfig) -> bool {
        matches!(
            self.ctl.focused_row(cfg, &*self.fit),
            Some(crate::model::RowId::Scrim(_))
        )
    }

    /// Tells the OSD what the live view really shows. `base` is the config being edited (the
    /// global `config.json`); `effective` is `base` with the current car's profile applied
    /// (pass `base` twice when there is none). When the focused row reads differently in the
    /// two, its help line says the car's profile overrides it. No allocation.
    pub fn note_profile(&mut self, base: &AppConfig, effective: &AppConfig) {
        let fit = &*self.fit;
        let overridden = self.ctl.focused_row(base, fit).is_some_and(|row| {
            let (mut global, mut car) = (model::Line::new(), model::Line::new());
            model::format_value(row, self.ctl.ctx(base, fit), &mut global);
            model::format_value(row, self.ctl.ctx(effective, fit), &mut car);
            global.as_str() != car.as_str() || model::toggle_value(row, base) != model::toggle_value(row, effective)
        });
        self.ctl.edit.profile_override = overridden;
        self.ctl.edit.live_anchor_y = Some(match self.ctl.edit.view {
            View::Cockpit => effective.cockpit_anchor_y,
            View::Hood => effective.bonnet_anchor_y,
            View::Chase => effective.chase_anchor_y,
        } as f32);
    }

    /// Sets the font families the Font row offers (from the loaded faces).
    pub fn set_fonts(&mut self, fonts: Vec<String>) {
        self.ctl.edit.fonts = fonts;
    }

    /// The OSD opened: browse mode, editing `view`; the cluster glides from `anchor_x_4k`.
    pub fn open(&mut self, cfg: &AppConfig, view: View, anchor_x_4k: f32) {
        self.ctl.open(cfg, view);
        // The camera rows say from the start whether the camera patch will write them.
        self.ctl.edit.camera = self.fit.camera_changed(cfg);
        self.anim.reset(anchor_x_4k);
        self.last_tab = self.ctl.tab();
    }

    /// The view the live cluster should show (the one being edited).
    #[must_use]
    pub fn edit_view(&self) -> View {
        self.ctl.edit.view
    }

    /// Feeds one frame of controller input; edits land in `cfg`.
    pub fn step(&mut self, input: &NavInput, cfg: &mut AppConfig) -> NavOutput {
        let out = self.ctl.step(input, cfg, &mut *self.fit);
        if out.changed {
            self.anim.energise();
        }
        if self.ctl.tab() != self.last_tab {
            self.last_tab = self.ctl.tab();
            self.anim.tab_changed();
        }
        out
    }

    fn carousel_visible(&self) -> bool {
        self.ctl.tab() == Tab::Cluster && self.ctl.mode() == Mode::Browse
    }

    /// Loads the studio the glass mirrors and the driving position's layers, once. They are
    /// files to read and decode (the studio is a 4096 px map), so a render loop calls this when
    /// its kit is ready and the first open stays smooth; [`OsdHost::prepare`] loads them if this
    /// was never called.
    pub fn load_assets(&mut self, kit: &mut Kit, device: &wgpu::Device, queue: &wgpu::Queue, assets: Option<&Path>) {
        if self.loaded {
            return;
        }
        self.loaded = true;
        let mut setup = kit.setup(device, queue, assets);
        self.env = liquid_glass::load_studio(&mut setup);
        self.drive = drive::load(&mut setup);
    }

    /// Forgets everything that was made for a kit: the carousel's clusters, the studio and the
    /// driving position's layers. Call when the kit is replaced (a lost device): their images
    /// and fonts belong to the old one. The next [`OsdHost::load_assets`] or
    /// [`OsdHost::prepare`] makes them again.
    pub fn device_changed(&mut self) {
        self.thumbs.clear();
        self.order = [None; 5];
        self.env = None;
        self.drive = None;
        self.loaded = false;
    }

    /// Creates the carousel's clusters if the selection moved, and loads the OSD's assets if
    /// [`OsdHost::load_assets`] has not. Call before `begin_frame`.
    pub fn prepare(
        &mut self,
        kit: &mut Kit,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: Option<&Path>,
        cfg: &AppConfig,
    ) {
        self.load_assets(kit, device, queue, assets);
        if !self.carousel_visible() {
            return;
        }
        let selected = model::theme_for_view(cfg, self.ctl.edit.view);
        let mut order = [None; 5];
        for (i, slot) in order.iter_mut().enumerate() {
            *slot = Some(model::theme_step(selected, i as i32 - 2));
        }
        if order == self.order {
            return;
        }
        self.order = order;
        for theme in order.into_iter().flatten() {
            if !self.thumbs.iter().any(|(t, _)| *t == theme) {
                let cluster = {
                    let mut setup = kit.setup(device, queue, assets);
                    fh6_clusters::registry::create(theme, &mut setup)
                };
                self.thumbs.push((theme, cluster));
            }
        }
        while self.thumbs.len() > THUMB_CACHE {
            match self.thumbs.iter().position(|(t, _)| !order.contains(&Some(*t))) {
                Some(i) => {
                    self.thumbs.remove(i);
                }
                None => break,
            }
        }
    }

    /// Advances motion and the thumbnail clusters with the frame the live cluster got.
    pub fn update(&mut self, input: &FrameInput<'_>, kit: &Kit, frame: Frame, cfg: &AppConfig) {
        let to = draw::targets(&self.ctl, cfg, &*self.fit, kit.fonts(), frame);
        self.anim.update(&to, input.dt);
        if self.carousel_visible() {
            for (theme, cluster) in &mut self.thumbs {
                if self.order.contains(&Some(*theme)) {
                    cluster.update(input);
                }
            }
        }
    }

    /// Snaps motion to its targets and warms the thumbnails (previews).
    pub fn settle(&mut self, input: &FrameInput<'_>, kit: &Kit, frame: Frame, cfg: &AppConfig) {
        let to = draw::targets(&self.ctl, cfg, &*self.fit, kit.fonts(), frame);
        self.anim.settle(&to);
        for (_, cluster) in &mut self.thumbs {
            for _ in 0..30 {
                cluster.update(input);
            }
        }
    }

    /// The layout the live cluster is drawn with while the OSD is open.
    #[must_use]
    pub fn live_layout(&self, base: &Layout, frame: Frame) -> Layout {
        draw::live_layout(base, &self.anim, frame)
    }

    /// The soft black pools under the OSD for this frame. The host passes them to
    /// `Kit::set_pools` before it renders, and an empty slice once the OSD is closed. `layout`
    /// is the live cluster's ([`OsdHost::live_layout`]); `cfg` is the config being edited (the
    /// driving position's pool follows its seat height); `live_pool` is false while the host
    /// draws the real dash blocker under the cluster instead ([`draw::live_backing`]).
    #[must_use]
    pub fn pools(&self, layout: &Layout, frame: Frame, cfg: &AppConfig, live_pool: bool) -> draw::Pools {
        draw::pools(&self.anim, frame, layout, cfg, live_pool)
    }

    /// Draws the OSD (after the backdrop and the live cluster). `layout` is the live cluster's
    /// ([`OsdHost::live_layout`]). Its black pools are not part of this: see
    /// [`OsdHost::pools`].
    #[allow(clippy::too_many_arguments, reason = "the host's per-frame inputs, each one plain")]
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
        layout: &Layout,
        cfg: &AppConfig,
        frame: Frame,
        now: Duration,
        live_name: &'static str,
        status: LiveStatus,
    ) {
        let mut thumbs: [Option<Thumb<'_>>; 5] = [None; 5];
        if self.carousel_visible() {
            for (slot, theme) in thumbs.iter_mut().zip(self.order) {
                let Some(theme) = theme else { continue };
                *slot = self.thumbs.iter().find(|(t, _)| *t == theme).map(|(t, c)| Thumb {
                    theme: *t,
                    cluster: &**c,
                });
            }
        }
        draw::ui(
            p,
            &DrawInput {
                ctl: &self.ctl,
                cfg,
                fit: &*self.fit,
                anim: &self.anim,
                frame,
                now,
                live_name,
                status,
                thumbs: &thumbs,
                env: self.env,
                drive: self.drive,
                live: *layout,
            },
        );
    }

    /// True while something in the OSD still moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.anim.open.running() || self.anim.tab_fade.running()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::{ConfigOnlyFit, SCRIM_ROWS};
    use crate::model::{RowId, PROFILE_OVERRIDE_NOTE};

    #[test]
    fn a_row_the_car_profile_overrides_says_so() {
        let mut host = OsdHost::new(&Tokens::default(), Box::new(ConfigOnlyFit));
        let base = AppConfig::default();
        host.open(&base, View::Cockpit, base.cockpit_anchor_x as f32);
        // The car's profile sets its own blocker opacity and binnacle anchor.
        let car = AppConfig {
            oem_dash_opaque_pct: 95,
            cockpit_anchor_y: 1820,
            ..base.clone()
        };
        let opacity = SCRIM_ROWS
            .iter()
            .position(|r| r.label == "Blocker strength")
            .map(|i| RowId::Scrim(i as u8));
        let opacity = opacity.expect("opacity row");
        assert!(host.focus_row(Tab::Fit, opacity, &base));

        host.note_profile(&base, &car);
        assert_eq!(model::help(opacity, &host.ctl.edit), Some(PROFILE_OVERRIDE_NOTE));
        assert_eq!(
            host.ctl.edit.live_anchor_y,
            Some(1820.0),
            "the default-height line follows the car"
        );

        // A row the profile leaves alone keeps its own help; no profile, no note.
        assert!(host.focus_row(Tab::Fit, RowId::ClusterScale, &base));
        host.note_profile(&base, &car);
        assert_eq!(model::help(RowId::ClusterScale, &host.ctl.edit), Some("Applies now"));
        assert!(host.focus_row(Tab::Fit, opacity, &base));
        host.note_profile(&base, &base);
        assert_ne!(model::help(opacity, &host.ctl.edit), Some(PROFILE_OVERRIDE_NOTE));
        assert_eq!(host.ctl.edit.live_anchor_y, Some(1835.0));

        // A toggle the profile flips is caught too.
        let auto = AppConfig {
            camera_patch_auto: true,
            ..base.clone()
        };
        assert!(host.focus_row(Tab::Fit, RowId::CameraPatch, &base));
        host.note_profile(&base, &auto);
        assert_eq!(
            model::help(RowId::CameraPatch, &host.ctl.edit),
            Some(PROFILE_OVERRIDE_NOTE)
        );
    }
}

/// The OSD on wgpu's no-op device (no GPU, no lock): it loads, updates and records frames as on
/// a real one.
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod gates {
    use super::*;
    use crate::fit::ConfigOnlyFit;
    use crate::model::RowId;
    use crate::nav::{LB, RIGHT, VIEW};
    use crate::LiveDynoGenerator;
    use fh6_core::ColorMode;
    use fh6_kit::testing::{Offscreen, RgbaImage, OFFSCREEN_FORMAT};
    use fh6_kit::{Logo, Palette, SignalSmoother};
    use std::future::Future;
    use std::path::PathBuf;
    use std::pin::pin;
    use std::task::{Context, Poll, Waker};
    use std::time::Duration;

    const TARGET: (u32, u32) = (3840, 2160);
    const DT: f32 = 1.0 / 60.0;

    fn block_on<F: Future>(f: F) -> F::Output {
        let mut f = pin!(f);
        let mut cx = Context::from_waker(Waker::noop());
        loop {
            if let Poll::Ready(v) = f.as_mut().poll(&mut cx) {
                return v;
            }
            std::thread::yield_now();
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    struct NoopKit {
        device: wgpu::Device,
        queue: wgpu::Queue,
        kit: Kit,
        tokens: Tokens,
        target: Offscreen,
    }

    impl NoopKit {
        fn new() -> Self {
            let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
            desc.backends = wgpu::Backends::NOOP;
            desc.backend_options.noop.enable = true;
            let instance = wgpu::Instance::new(desc);
            let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("wgpu no-op adapter (is the `noop` feature on?)");
            let (device, queue) =
                block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).expect("no-op device");
            let tokens = Tokens::load(&repo_root().join("data/design_tokens.json")).expect("tokens");
            let mut kit = Kit::new(&device, &queue, tokens.clone()).expect("kit");
            kit.fonts_mut()
                .load_dir(&repo_root().join("assets/fonts"))
                .expect("fonts");
            kit.bind_font_roles();
            let target = Offscreen::new(&device, TARGET.0, TARGET.1);
            Self {
                device,
                queue,
                kit,
                tokens,
                target,
            }
        }

        /// Renders the recorded frame (the no-op device discards it), which generates the
        /// glyphs the frame missed.
        fn render(&mut self) {
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
            self.kit.render(
                &self.device,
                &self.queue,
                &mut encoder,
                self.target.view(),
                OFFSCREEN_FORMAT,
                TARGET,
                1.0,
            );
            self.queue.submit([encoder.finish()]);
            self.kit.frame_submitted();
        }
    }

    /// An assets folder that holds small stand-ins for the files the OSD loads: the code paths
    /// are those of the real files, without decoding a 4096 px studio map in a debug build.
    fn stand_in_assets(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fh6-osd-gates-{}-{name}", std::process::id()));
        let image = RgbaImage {
            width: 4,
            height: 4,
            pixels: vec![128; 64],
        };
        let files = [
            "clusters/liquid_glass/studio_env.png",
            "osd/drive_dash.png",
            "osd/drive_wheel.png",
            "osd/drive_wheel_rim.png",
            "osd/drive_seat.png",
            "osd/drive_seat_rim.png",
        ];
        for file in files {
            let path = dir.join(file);
            std::fs::create_dir_all(path.parent().expect("a folder")).expect("temp folder");
            image.save_png(&path).expect("stand-in image");
        }
        dir
    }

    fn press(host: &mut OsdHost, cfg: &mut AppConfig, t_ms: u64, held: u16, pressed: u16) {
        let input = NavInput {
            held,
            pressed,
            released: 0,
            now: Duration::from_millis(t_ms),
            toggle_mask: cfg.osd_toggle_mask,
            hold_ms: cfg.osd_hold_ms,
        };
        host.step(&input, cfg);
    }

    type State = (&'static str, fn(&mut OsdHost, &mut AppConfig) -> Duration);

    /// One state per kind of row and per mode: what `preview-export --osd` renders.
    const STATES: [State; 9] = [
        ("the carousel", |h, c| {
            assert!(h.focus_row(Tab::Cluster, RowId::Cluster, c));
            Duration::ZERO
        }),
        ("a hue slider", |h, c| {
            c.color_mode = ColorMode::Custom;
            assert!(h.focus_row(Tab::Colour, RowId::Hue, c));
            Duration::ZERO
        }),
        ("the font stepper", |h, c| {
            assert!(h.focus_row(Tab::Type, RowId::Font, c));
            Duration::ZERO
        }),
        ("the cockpit FOV and the driving position", |h, c| {
            assert!(h.focus_row(Tab::Fit, RowId::FovCockpit, c));
            Duration::ZERO
        }),
        ("the seat height", |h, c| {
            c.driver_seat_height_m = 0.020;
            assert!(h.focus_row(Tab::Fit, RowId::SeatHeight, c));
            Duration::ZERO
        }),
        ("the cluster height and its default line", |h, c| {
            c.cockpit_height_offset_px = 64.0;
            assert!(h.focus_row(Tab::Fit, RowId::ClusterHeight, c));
            Duration::ZERO
        }),
        ("a dash-blocker row", |h, c| {
            assert!(h.focus_row(Tab::Fit, RowId::Scrim(1), c));
            Duration::ZERO
        }),
        ("the keyboard", |h, c| {
            assert!(h.focus_row(Tab::Type, RowId::CustomName, c));
            h.ctl.start_keyboard(c);
            press(h, c, 0, 0, LB);
            press(h, c, 20, RIGHT, RIGHT);
            Duration::from_millis(100)
        }),
        ("the remapper mid-capture", |h, c| {
            assert!(h.focus_row(Tab::Controls, RowId::OsdButton, c));
            h.ctl.start_remap(Duration::ZERO);
            press(h, c, 0, VIEW | 0x0040, VIEW | 0x0040);
            press(h, c, 600, VIEW | 0x0040, 0);
            Duration::from_millis(600)
        }),
    ];

    /// Frames before the measured ones: the scene's buffers reach their size, the thumbnail
    /// clusters exist, every glyph is in the atlas and the springs have moved.
    const WARM_FRAMES: u32 = 24;
    const MEASURED_FRAMES: u32 = 8;

    /// Heap allocations of the OSD's own share of a warm frame, at their worst over
    /// [`MEASURED_FRAMES`]: `prepare`, `update`, the pools, and `draw` into a begun frame.
    fn warm_allocations(nk: &mut NoopKit, assets: Option<&Path>, state: &State) -> u64 {
        let mut cfg = AppConfig::default();
        let mut host = OsdHost::new(&nk.tokens, Box::new(ConfigOnlyFit));
        host.set_fonts(nk.kit.fonts().families());
        host.open(&cfg, View::Cockpit, cfg.cockpit_anchor_x as f32);
        let now = (state.1)(&mut host, &mut cfg);
        let view = host.edit_view();
        let frame = Frame::new(TARGET);
        let base = Layout::from_config(&cfg, &nk.tokens, view, TARGET);
        let palette = Palette::family(&nk.tokens, "liquidGlass");
        nk.kit.set_palette(palette);
        let logo = Logo::default();
        let fonts = nk.kit.font_choice(None);
        let mut smoother = SignalSmoother::new();
        let mut dyno = LiveDynoGenerator::new();
        let mut worst = 0;
        for i in 0..WARM_FRAMES + MEASURED_FRAMES {
            let data = dyno.tick(DT);
            let signals = smoother.update(&data, &cfg, &nk.tokens, DT);
            let input = FrameInput {
                telemetry: &data,
                signals: &signals,
                dt: DT,
                t: i as f32 * DT,
                config: &cfg,
                palette: &palette,
                tokens: &nk.tokens,
                fonts,
                logo: &logo,
                view,
                reduced_flash: cfg.reduced_flash,
            };
            let info = allocation_counter::measure(|| {
                host.prepare(&mut nk.kit, &nk.device, &nk.queue, assets, &cfg);
                host.update(&input, &nk.kit, frame, &cfg);
                let layout = host.live_layout(&base, frame);
                std::hint::black_box(host.pools(&layout, frame, &cfg, true).as_slice());
                let mut p = nk.kit.begin_frame(&layout);
                draw::live_backing(&mut p, None);
                host.draw(&mut p, &layout, &cfg, frame, now, "Live", LiveStatus::Dyno);
            });
            if i >= WARM_FRAMES {
                worst = worst.max(info.count_total);
            }
            nk.render();
        }
        assert_eq!(
            nk.kit.scene().dropped(),
            0,
            "{}: the frame fits the kit's limits",
            state.0
        );
        assert_eq!(
            nk.kit.scene().missing_glyph_count(),
            0,
            "{}: every glyph is ready",
            state.0
        );
        worst
    }

    #[test]
    fn a_warm_osd_frame_allocates_nothing() {
        let mut nk = NoopKit::new();
        let dir = stand_in_assets("frames");
        let mut failures = Vec::new();
        // With the rendered layers and the studio map, and with neither (the line drawing and
        // the kit's analytic studio).
        for assets in [Some(dir.as_path()), None] {
            for state in &STATES {
                let count = warm_allocations(&mut nk, assets, state);
                if count > 0 {
                    let with = if assets.is_some() { "with" } else { "without" };
                    failures.push(format!("{} ({with} assets): {count} allocations", state.0));
                }
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
        assert!(failures.is_empty(), "allocating OSD frames:\n{}", failures.join("\n"));
    }

    /// The gate above is only as good as the counter: it must see an allocation on this thread.
    #[test]
    fn the_allocation_counter_sees_allocations() {
        let info = allocation_counter::measure(|| {
            std::hint::black_box(vec![0u8; 64]);
        });
        assert!(info.count_total >= 1, "allocation-counter is not the global allocator");
    }

    #[test]
    fn a_new_kit_gets_its_own_assets_and_thumbnails() {
        let dir = stand_in_assets("device");
        let assets = Some(dir.as_path());
        let cfg = AppConfig::default();
        let mut old = NoopKit::new();
        let mut host = OsdHost::new(&old.tokens, Box::new(ConfigOnlyFit));
        host.open(&cfg, View::Cockpit, cfg.cockpit_anchor_x as f32);
        assert!(host.focus_row(Tab::Cluster, RowId::Cluster, &cfg));

        // At start-up, before the OSD has ever opened.
        host.load_assets(&mut old.kit, &old.device, &old.queue, assets);
        let loaded = (host.env, host.drive);
        assert!(loaded.0.is_some() && loaded.1.is_some(), "the stand-in files load");
        // `prepare` finds them loaded and reads nothing again; it builds the carousel.
        host.prepare(&mut old.kit, &old.device, &old.queue, assets, &cfg);
        assert_eq!((host.env, host.drive), loaded);
        assert_eq!(host.thumbs.len(), 5);

        // The device is lost: nothing made for the old kit is kept.
        host.device_changed();
        assert!(host.env.is_none() && host.drive.is_none() && !host.loaded);
        assert!(host.thumbs.is_empty() && host.order == [None; 5]);

        // The new kit gets its own images and its own carousel.
        let mut new = NoopKit::new();
        host.prepare(&mut new.kit, &new.device, &new.queue, assets, &cfg);
        let env = host.env.expect("the studio loads into the new kit");
        let drive = host.drive.expect("the layers load into the new kit");
        for image in [
            env,
            drive.dash,
            drive.wheel,
            drive.wheel_rim,
            drive.seat,
            drive.seat_rim,
        ] {
            assert_eq!(new.kit.images().size(image), Some((4, 4)));
        }
        assert_eq!(host.thumbs.len(), 5);

        // Without an assets folder nothing loads, and nothing is looked for again each frame.
        host.device_changed();
        host.load_assets(&mut new.kit, &new.device, &new.queue, None);
        assert!(host.loaded && host.env.is_none() && host.drive.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
