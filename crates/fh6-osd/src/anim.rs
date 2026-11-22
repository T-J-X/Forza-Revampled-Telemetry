//! OSD motion: the open fade, the live cluster's glide into the live zone, the focus ring and
//! tab underline springs, row scrolling and the tab crossfade (tokens springs and durations).
use fh6_kit::{Spring, Tokens, Tween};

/// Animated values, advanced once per frame.
#[derive(Debug, Clone, Copy)]
pub struct OsdAnim {
    /// 0..1 open progress (200 ms).
    pub open: Tween,
    /// Live cluster x in 4K units.
    pub glide: Spring,
    /// Focus ring top in panel content units.
    pub focus_y: Spring,
    /// Focus ring height.
    pub focus_h: Spring,
    /// Tab underline centre x.
    pub underline_x: Spring,
    /// Tab underline width.
    pub underline_w: Spring,
    /// Row scroll offset.
    pub scroll: Spring,
    /// Rows fade in after a tab change (150 ms).
    pub tab_fade: Tween,
    /// Seconds since the OSD opened (pulses).
    pub t: f32,
    primed: bool,
}

/// Where the animated values are heading this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AnimTargets {
    /// Live cluster x target.
    pub glide: f32,
    /// Focus ring top.
    pub focus_y: f32,
    /// Focus ring height.
    pub focus_h: f32,
    /// Underline centre.
    pub underline_x: f32,
    /// Underline width.
    pub underline_w: f32,
    /// Scroll offset.
    pub scroll: f32,
}

impl OsdAnim {
    /// Fresh state; call [`OsdAnim::reset`] when the OSD opens.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            open: Tween::new(0.2),
            glide: Spring::new(tokens.springs.default, 1920.0),
            focus_y: Spring::new(tokens.springs.fast, 0.0),
            focus_h: Spring::new(tokens.springs.fast, 120.0),
            underline_x: Spring::new(tokens.springs.fast, 0.0),
            underline_w: Spring::new(tokens.springs.fast, 0.0),
            scroll: Spring::new(tokens.springs.default, 0.0),
            tab_fade: Tween::new(tokens.durations.crossfade.max(0.05)),
            t: 0.0,
            primed: false,
        }
    }

    /// Restarts the open animation; the cluster glides from `from_x` (its normal anchor).
    pub fn reset(&mut self, from_x: f32) {
        self.open.restart();
        self.glide.reset(from_x);
        self.tab_fade.restart();
        self.t = 0.0;
        self.primed = false;
    }

    /// The tab changed: crossfade the rows.
    pub fn tab_changed(&mut self) {
        self.tab_fade.restart();
    }

    /// Advances by `dt` toward `to`. The first frame after a reset snaps the focus, underline
    /// and scroll so they do not fly in from zero.
    pub fn update(&mut self, to: &AnimTargets, dt: f32) {
        if !self.primed {
            self.focus_y.reset(to.focus_y);
            self.focus_h.reset(to.focus_h);
            self.underline_x.reset(to.underline_x);
            self.underline_w.reset(to.underline_w);
            self.scroll.reset(to.scroll);
            self.primed = true;
        }
        self.open.update(dt);
        self.tab_fade.update(dt);
        self.glide.update(to.glide, dt);
        self.focus_y.update(to.focus_y, dt);
        self.focus_h.update(to.focus_h, dt);
        self.underline_x.update(to.underline_x, dt);
        self.underline_w.update(to.underline_w, dt);
        self.scroll.update(to.scroll, dt);
        self.t += dt;
    }

    /// Snaps everything to its target (previews).
    pub fn settle(&mut self, to: &AnimTargets) {
        for _ in 0..240 {
            self.update(to, 1.0 / 60.0);
        }
    }
}
