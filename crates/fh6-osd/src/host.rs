//! Glue between a render loop and the OSD: navigation state, motion, the cockpit-fit backend
//! and the carousel's live thumbnail clusters. The overlay and `preview-export` both drive it.
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
use crate::fit::CockpitFit;
use crate::model::{self, Tab};
use crate::nav::{Mode, NavInput, NavOutput, OsdController};
use fh6_core::{AppConfig, ClusterTheme};
use fh6_kit::{wgpu, Cluster, FrameInput, Kit, Layout, Painter, Tokens, View};
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

    /// Sets the font families the Font row offers (from the loaded faces).
    pub fn set_fonts(&mut self, fonts: Vec<String>) {
        self.ctl.edit.fonts = fonts;
    }

    /// The OSD opened: browse mode, editing `view`; the cluster glides from `anchor_x_4k`.
    pub fn open(&mut self, cfg: &AppConfig, view: View, anchor_x_4k: f32) {
        self.ctl.open(cfg, view);
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
        if self.ctl.tab() != self.last_tab {
            self.last_tab = self.ctl.tab();
            self.anim.tab_changed();
        }
        out
    }

    fn carousel_visible(&self) -> bool {
        self.ctl.tab() == Tab::Cluster && self.ctl.mode() == Mode::Browse
    }

    /// Creates the carousel's clusters if the selection moved. Call before `begin_frame`.
    pub fn prepare(
        &mut self,
        kit: &mut Kit,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        assets: Option<&Path>,
        cfg: &AppConfig,
    ) {
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

    /// Draws the OSD (after the backdrop and the live cluster).
    pub fn draw(
        &self,
        p: &mut Painter<'_>,
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
            },
        );
    }

    /// True while something in the OSD still moves.
    #[must_use]
    pub fn animating(&self) -> bool {
        self.anim.open.running() || self.anim.tab_fade.running()
    }
}
