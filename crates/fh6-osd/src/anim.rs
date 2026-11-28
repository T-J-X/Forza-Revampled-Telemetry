//! OSD motion: the open ramp, the live cluster's glide into the live zone, the glass pieces'
//! liquid moves (each edge on its own spring), row scrolling, the tab crossfade, the focus
//! echo's lift over the driving position and the focused piece's inner light while a value
//! changes (tokens springs and durations).
use fh6_kit::{Spring, SpringSpec, Tokens, Tween};

/// How long a value change keeps the inner light up before it decays (s). Hold-to-repeat
/// re-arms it every step, so a held adjust stays lit instead of pulsing.
const ENERGISE_HOLD: f32 = 0.18;
/// How long the scroll bar stays up after the rows last scrolled, and after the OSD opens (s).
const SCROLLBAR_HOLD: f32 = 1.0;

/// Animated values, advanced once per frame.
#[derive(Debug, Clone, Copy)]
pub struct OsdAnim {
    /// 0..1 open progress (200 ms): the content fades in and the glass grows in.
    pub open: Tween,
    /// Live cluster x in 4K units.
    pub glide: Spring,
    /// The focus platter's top edge (content units). On a move the leading edge travels on the
    /// `fast` spring and the trailing edge on `expressive`, so the platter stretches toward the
    /// new row and settles like a liquid.
    pub focus_top: Spring,
    /// The focus platter's bottom edge.
    pub focus_bot: Spring,
    /// The selected tab's fill, left edge (4K units).
    pub tab_left: Spring,
    /// The selected tab's fill, right edge.
    pub tab_right: Spring,
    /// The keyboard's glass key cap: left, top, right and bottom edges.
    pub key: [Spring; 4],
    /// Row scroll offset.
    pub scroll: Spring,
    /// Rows fade in after a tab change (150 ms).
    pub tab_fade: Tween,
    /// The focus echo's lift on the camera rows: 0 at rest, 1 raised over the driving position,
    /// which fades in with it.
    pub echo_lift: Spring,
    /// The focused piece's extra inner light while a value changes (0 at rest, 1 lit).
    pub energy: Spring,
    /// Seconds the inner light stays up after the last value change.
    energised: f32,
    /// The scroll bar's opacity: up while the rows scroll and briefly on open, else hidden.
    pub scrollbar: Spring,
    /// Seconds the scroll bar stays up.
    scrolled: f32,
    /// Seconds since the OSD opened (the text cursor's blink).
    pub t: f32,
    primed: bool,
    /// Set by [`OsdAnim::tab_changed`]: the next update puts the platter and the scroll on target.
    snap_focus: bool,
    lead: SpringSpec,
    trail: SpringSpec,
}

/// Where the animated values are heading this frame.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AnimTargets {
    /// Live cluster x target.
    pub glide: f32,
    /// Focus platter top.
    pub focus_top: f32,
    /// Focus platter bottom.
    pub focus_bot: f32,
    /// Selected tab fill, left edge.
    pub tab_left: f32,
    /// Selected tab fill, right edge.
    pub tab_right: f32,
    /// The keyboard's focused key (left, top, right, bottom); ignored while it is closed.
    pub key: [f32; 4],
    /// Scroll offset.
    pub scroll: f32,
    /// The focus echo's lift (1 on a camera row, else 0).
    pub echo_lift: f32,
    /// Calm motion (`ReducedFlash`): no stretch and no energise light.
    pub calm: bool,
}

/// The springs for one pair of edges: the leading one, the trailing one, and calm motion.
#[derive(Debug, Clone, Copy)]
struct Pace {
    lead: SpringSpec,
    trail: SpringSpec,
    calm: bool,
}

/// Moves a pair of edges (`lo` below `hi`) toward their targets. The edge on the side the pair
/// is moving to leads, the other trails; calm motion moves both on the leading spring.
fn edges(lo: &mut Spring, hi: &mut Spring, to: (f32, f32), pace: Pace, dt: f32) {
    let rising = to.0 + to.1 > lo.value() + hi.value();
    let (lo_spec, hi_spec) = match (pace.calm, rising) {
        (true, _) => (pace.lead, pace.lead),
        (false, true) => (pace.trail, pace.lead),
        (false, false) => (pace.lead, pace.trail),
    };
    lo.spec = lo_spec;
    hi.spec = hi_spec;
    lo.update(to.0, dt);
    hi.update(to.1, dt);
}

impl OsdAnim {
    /// Fresh state; call [`OsdAnim::reset`] when the OSD opens.
    #[must_use]
    pub fn new(tokens: &Tokens) -> Self {
        let s = &tokens.springs;
        let edge = |v: f32| Spring::new(s.fast, v);
        Self {
            open: Tween::new(tokens.durations.materialize.max(0.05)),
            glide: Spring::new(s.default, 1920.0),
            focus_top: edge(0.0),
            focus_bot: edge(120.0),
            tab_left: edge(0.0),
            tab_right: edge(0.0),
            key: [edge(0.0); 4],
            scroll: Spring::new(s.default, 0.0),
            tab_fade: Tween::new(tokens.durations.crossfade.max(0.05)),
            echo_lift: Spring::new(s.default, 0.0),
            energy: Spring::new(s.default, 0.0),
            energised: 0.0,
            scrollbar: Spring::new(s.default, 0.0),
            scrolled: 0.0,
            t: 0.0,
            primed: false,
            snap_focus: false,
            lead: s.fast,
            trail: s.expressive,
        }
    }

    /// Restarts the open animation; the cluster glides from `from_x` (its normal anchor).
    pub fn reset(&mut self, from_x: f32) {
        self.open.restart();
        self.glide.reset(from_x);
        self.tab_fade.restart();
        self.energy.reset(0.0);
        self.energised = 0.0;
        self.scrollbar.reset(0.0);
        self.scrolled = SCROLLBAR_HOLD;
        self.t = 0.0;
        self.primed = false;
    }

    /// The tab changed: crossfade the rows, and put the focus platter and the scroll straight
    /// on the new tab's row on the next update, so the light never travels over a row of the
    /// new tab that does not have focus. The tab's own fill still slides, and the scroll bar
    /// shows for a moment, as on open.
    pub fn tab_changed(&mut self) {
        self.tab_fade.restart();
        self.snap_focus = true;
        self.scrolled = SCROLLBAR_HOLD;
    }

    /// A value changed: the focused piece lights up and stays lit while changes keep coming.
    pub fn energise(&mut self) {
        self.energised = ENERGISE_HOLD;
    }

    /// Advances by `dt` toward `to`. The first frame after a reset snaps the glass pieces, the
    /// scroll and the echo's lift so they do not fly in from zero.
    pub fn update(&mut self, to: &AnimTargets, dt: f32) {
        if !self.primed {
            self.focus_top.reset(to.focus_top);
            self.focus_bot.reset(to.focus_bot);
            self.tab_left.reset(to.tab_left);
            self.tab_right.reset(to.tab_right);
            for (spring, target) in self.key.iter_mut().zip(to.key) {
                spring.reset(target);
            }
            self.scroll.reset(to.scroll);
            self.echo_lift.reset(to.echo_lift);
            self.primed = true;
        }
        if self.snap_focus {
            self.focus_top.reset(to.focus_top);
            self.focus_bot.reset(to.focus_bot);
            self.scroll.reset(to.scroll);
            self.snap_focus = false;
        }
        let pace = Pace {
            lead: self.lead,
            trail: self.trail,
            calm: to.calm,
        };
        self.open.update(dt);
        self.tab_fade.update(dt);
        self.glide.update(to.glide, dt);
        edges(
            &mut self.focus_top,
            &mut self.focus_bot,
            (to.focus_top, to.focus_bot),
            pace,
            dt,
        );
        edges(
            &mut self.tab_left,
            &mut self.tab_right,
            (to.tab_left, to.tab_right),
            pace,
            dt,
        );
        let [left, top, right, bottom] = &mut self.key;
        edges(left, right, (to.key[0], to.key[2]), pace, dt);
        edges(top, bottom, (to.key[1], to.key[3]), pace, dt);
        if (to.scroll - self.scroll.value()).abs() > 1.0 {
            self.scrolled = SCROLLBAR_HOLD;
        }
        self.scroll.update(to.scroll, dt);
        let shown = self.scrolled > 0.0;
        self.scrolled = (self.scrolled - dt).max(0.0);
        self.scrollbar.update(if shown { 1.0 } else { 0.0 }, dt);
        self.echo_lift.update(to.echo_lift, dt);
        let lit = self.energised > 0.0 && !to.calm;
        self.energised = (self.energised - dt).max(0.0);
        self.energy.update(if lit { 1.0 } else { 0.0 }, dt);
        self.t += dt;
    }

    /// Snaps everything to its target (previews).
    pub fn settle(&mut self, to: &AnimTargets) {
        for _ in 0..240 {
            self.update(to, 1.0 / 60.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn targets(top: f32) -> AnimTargets {
        AnimTargets {
            focus_top: top,
            focus_bot: top + 116.0,
            ..AnimTargets::default()
        }
    }

    #[test]
    fn a_move_down_stretches_then_settles() {
        let mut a = OsdAnim::new(&Tokens::default());
        a.settle(&targets(0.0));
        let to = targets(116.0);
        let mut widest = 0.0f32;
        for _ in 0..60 {
            a.update(&to, 1.0 / 60.0);
            let h = a.focus_bot.value() - a.focus_top.value();
            widest = widest.max(h);
            assert!(h > 40.0, "the platter never inverts");
        }
        assert!(widest > 130.0, "the leading edge runs ahead: {widest}");
        assert!((a.focus_top.value() - 116.0).abs() < 1.0 && (a.focus_bot.value() - 232.0).abs() < 1.0);
    }

    #[test]
    fn calm_motion_does_not_stretch_or_light_up() {
        let mut a = OsdAnim::new(&Tokens::default());
        a.settle(&targets(0.0));
        let mut to = targets(116.0);
        to.calm = true;
        a.energise();
        for _ in 0..60 {
            a.update(&to, 1.0 / 60.0);
            let h = a.focus_bot.value() - a.focus_top.value();
            assert!((h - 116.0).abs() < 0.5, "both edges move together: {h}");
        }
        assert!(a.energy.value() < 1e-3);
    }

    #[test]
    fn the_echo_lift_snaps_on_open_and_springs_after() {
        let mut a = OsdAnim::new(&Tokens::default());
        a.reset(1920.0);
        let mut to = targets(0.0);
        to.echo_lift = 1.0;
        // The OSD opened on a camera row: lifted from the first frame, not flying up.
        a.update(&to, 1.0 / 60.0);
        assert!((a.echo_lift.value() - 1.0).abs() < 1e-6);
        // Leaving the camera rows: it comes down over a few frames and settles.
        to.echo_lift = 0.0;
        a.update(&to, 1.0 / 60.0);
        let first = a.echo_lift.value();
        assert!(first > 0.5 && first < 1.0, "moving, not snapped: {first}");
        for _ in 0..60 {
            a.update(&to, 1.0 / 60.0);
        }
        assert!(a.echo_lift.value().abs() < 0.01, "settled: {}", a.echo_lift.value());
    }

    #[test]
    fn a_held_adjust_stays_lit_then_decays() {
        let mut a = OsdAnim::new(&Tokens::default());
        let to = targets(0.0);
        a.settle(&to);
        // Hold-to-repeat: a step every 65 ms for a second.
        let mut low_after_rise = f32::MAX;
        for frame in 0..60 {
            if frame % 4 == 0 {
                a.energise();
            }
            a.update(&to, 1.0 / 60.0);
            if frame > 20 {
                low_after_rise = low_after_rise.min(a.energy.value());
            }
        }
        assert!(low_after_rise > 0.9, "no pulsing while held: {low_after_rise}");
        for _ in 0..60 {
            a.update(&to, 1.0 / 60.0);
        }
        assert!(a.energy.value() < 0.05, "decays after release");
    }

    #[test]
    fn the_scroll_bar_shows_on_open_and_while_scrolling_then_hides() {
        let mut a = OsdAnim::new(&Tokens::default());
        a.reset(1920.0);
        let mut to = targets(0.0);
        for _ in 0..20 {
            a.update(&to, 1.0 / 60.0);
        }
        assert!(a.scrollbar.value() > 0.5, "flashes on open: {}", a.scrollbar.value());
        for _ in 0..120 {
            a.update(&to, 1.0 / 60.0);
        }
        assert!(a.scrollbar.value() < 0.01, "hidden at rest: {}", a.scrollbar.value());
        to.scroll = 400.0;
        for _ in 0..20 {
            a.update(&to, 1.0 / 60.0);
        }
        assert!(a.scrollbar.value() > 0.5, "up while scrolling: {}", a.scrollbar.value());
    }

    #[test]
    fn a_tab_change_puts_the_platter_on_the_new_row_at_once() {
        let mut a = OsdAnim::new(&Tokens::default());
        let mut to = targets(0.0);
        a.settle(&to);
        to.focus_top = 600.0;
        to.focus_bot = 716.0;
        to.scroll = 200.0;
        a.tab_changed();
        a.update(&to, 1.0 / 60.0);
        assert!((a.focus_top.value() - 600.0).abs() < 1e-3 && (a.focus_bot.value() - 716.0).abs() < 1e-3);
        assert!((a.scroll.value() - 200.0).abs() < 1e-3);
        // A move within the tab still travels.
        to.focus_top = 900.0;
        to.focus_bot = 1016.0;
        a.update(&to, 1.0 / 60.0);
        assert!(a.focus_top.value() < 899.0, "travels: {}", a.focus_top.value());
    }
}
