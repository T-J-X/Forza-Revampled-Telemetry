//! The OSD's input state machine: browse rows, type a name, or capture a new OSD button.
//!
//! Input is the controller's held buttons plus the press and release edges latched since the
//! last frame, so a tap shorter than a frame still registers and every input shows on the next
//! frame. A, B, X and Y are stripped before anything looks at the buttons.
//!
//! - D-pad up/down moves the focus (wraps on a press, stops at the ends while held); left/right
//!   adjusts. Held directions repeat after 280 ms, every 65 ms, then every 25 ms after 900 ms;
//!   sliders also take bigger steps the longer they are held (×4 after 1.2 s, ×10 after 2.5 s).
//! - LB / RB switch tabs.
//! - View or Menu goes back (closes the OSD). A button that is part of the OSD toggle acts on
//!   release when it was held for less than the hold time, so holding it still closes the OSD
//!   through the hold toggle and never does both.
use crate::fit::CockpitFit;
use crate::keyboard::{KeyOutcome, Keyboard};
use crate::model::{self, Ctx, EditState, RowId, RowKind, Tab};
use crate::remap::{RemapCapture, RemapOutcome};
use fh6_core::config::MASK_MANUAL_LOCK;
use fh6_core::AppConfig;
use fh6_kit::View;
use std::time::Duration;

/// D-pad up.
pub const UP: u16 = 0x0001;
/// D-pad down.
pub const DOWN: u16 = 0x0002;
/// D-pad left.
pub const LEFT: u16 = 0x0004;
/// D-pad right.
pub const RIGHT: u16 = 0x0008;
/// Menu / Start.
pub const MENU: u16 = 0x0010;
/// View / Back.
pub const VIEW: u16 = 0x0020;
/// Left bumper.
pub const LB: u16 = 0x0100;
/// Right bumper.
pub const RB: u16 = 0x0200;
const DPAD: u16 = UP | DOWN | LEFT | RIGHT;

/// First repeat after this long.
pub const REPEAT_DELAY: Duration = Duration::from_millis(280);
/// Repeat interval until [`REPEAT_FAST_AFTER`].
pub const REPEAT_SLOW: Duration = Duration::from_millis(65);
/// Repeat interval after [`REPEAT_FAST_AFTER`].
pub const REPEAT_FAST: Duration = Duration::from_millis(25);
/// When repeats speed up.
pub const REPEAT_FAST_AFTER: Duration = Duration::from_millis(900);

/// One frame of controller input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NavInput {
    /// Buttons held now.
    pub held: u16,
    /// Buttons pressed since the last frame.
    pub pressed: u16,
    /// Buttons released since the last frame.
    pub released: u16,
    /// Monotonic time.
    pub now: Duration,
    /// The OSD toggle mask (`OsdToggleMask`).
    pub toggle_mask: u16,
    /// The OSD hold time (`OsdHoldMs`).
    pub hold_ms: u32,
}

/// What a frame of input did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NavOutput {
    /// The working config changed (save it, swap the live config).
    pub changed: bool,
    /// Close the OSD.
    pub close: bool,
    /// Suspend the OSD hold toggle (typing, capturing, or waiting for buttons to lift).
    pub settings_focus: bool,
    /// Anything visible changed (focus, tab, mode).
    pub moved: bool,
}

/// What the D-pad did this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Fire {
    /// The direction bit, or 0.
    pub dir: u16,
    /// Steps to apply (0 when nothing fired).
    pub count: u32,
    /// True for the press itself (not a repeat).
    pub first: bool,
    /// How long the direction has been held.
    pub held_for: Duration,
}

/// Hold-to-repeat for the D-pad, with acceleration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Repeater {
    dir: u16,
    since: Duration,
    next: Duration,
}

impl Repeater {
    /// Feeds one frame; returns what fired.
    pub fn update(&mut self, held: u16, pressed: u16, now: Duration) -> Fire {
        let fresh = pressed & DPAD;
        if fresh != 0 {
            let dir = [UP, DOWN, LEFT, RIGHT]
                .into_iter()
                .find(|b| fresh & b != 0)
                .unwrap_or(0);
            self.dir = dir;
            self.since = now;
            self.next = now + REPEAT_DELAY;
            if held & dir == 0 {
                // Pressed and released within one frame: one step, no repeat.
                self.dir = 0;
            }
            return Fire {
                dir,
                count: 1,
                first: true,
                held_for: Duration::ZERO,
            };
        }
        if self.dir == 0 || held & self.dir == 0 {
            self.dir = 0;
            return Fire::default();
        }
        let mut count = 0;
        while now >= self.next && count < 6 {
            count += 1;
            let step = if self.next.saturating_sub(self.since) < REPEAT_FAST_AFTER {
                REPEAT_SLOW
            } else {
                REPEAT_FAST
            };
            self.next += step;
        }
        if now >= self.next {
            // Far behind (a stalled frame): drop the backlog instead of jumping.
            self.next = now + REPEAT_FAST;
        }
        Fire {
            dir: self.dir,
            count,
            first: false,
            held_for: now.saturating_sub(self.since),
        }
    }
}

/// Slider step multiplier for a direction held for `held_for`.
#[must_use]
pub fn accel(held_for: Duration) -> i32 {
    match held_for.as_millis() {
        0..=1199 => 1,
        1200..=2499 => 4,
        _ => 10,
    }
}

/// The controller's mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Moving between rows and adjusting them.
    #[default]
    Browse,
    /// The on-screen keyboard (custom name).
    Keyboard,
    /// Capturing a new OSD button.
    Remap,
}

/// A short message shown in the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    /// The text.
    pub text: &'static str,
    /// Shown until.
    pub until: Duration,
}

/// How long notices stay up.
pub const NOTICE_TIME: Duration = Duration::from_millis(2200);

/// The OSD's navigation and editing state.
#[derive(Debug, Clone)]
pub struct OsdController {
    tab: Tab,
    focus: [usize; 5],
    mode: Mode,
    /// Edit state shared with the rows.
    pub edit: EditState,
    keyboard: Keyboard,
    remap: RemapCapture,
    repeater: Repeater,
    back_down: [Option<Duration>; 2],
    release_guard: bool,
    notice: Option<Notice>,
}

impl Default for OsdController {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdController {
    /// A controller on the first tab.
    #[must_use]
    pub fn new() -> Self {
        Self {
            tab: Tab::Cluster,
            focus: [0; 5],
            mode: Mode::Browse,
            edit: EditState::default(),
            keyboard: Keyboard::default(),
            remap: RemapCapture::default(),
            repeater: Repeater::default(),
            back_down: [None; 2],
            release_guard: false,
            notice: None,
        }
    }

    /// Called when the OSD opens: back to browsing, edits the view the camera is in. The tab
    /// and focus are kept from last time.
    pub fn open(&mut self, cfg: &AppConfig, view: View) {
        self.mode = Mode::Browse;
        self.edit.view = view;
        self.edit.load_colours(cfg);
        self.repeater = Repeater::default();
        self.back_down = [None; 2];
        self.release_guard = false;
        self.notice = None;
    }

    /// The active tab.
    #[must_use]
    pub fn tab(&self) -> Tab {
        self.tab
    }

    /// Jumps to a tab (previews, tests).
    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    /// The focused row index on the active tab.
    #[must_use]
    pub fn focus_index(&self) -> usize {
        self.focus[self.tab.index()]
    }

    /// Sets the focused row index on the active tab (previews, tests).
    pub fn set_focus_index(&mut self, i: usize) {
        self.focus[self.tab.index()] = i;
    }

    /// Jumps to `tab` and focuses `row` there (previews, tests). Returns false, focusing the
    /// first row, when the tab does not show that row with this config.
    pub fn focus_row(&mut self, tab: Tab, row: RowId, cfg: &AppConfig, fit: &dyn CockpitFit) -> bool {
        self.tab = tab;
        let rows = model::rows(tab, self.ctx(cfg, fit));
        let found = rows.as_slice().iter().position(|r| *r == row);
        self.set_focus_index(found.unwrap_or(0));
        found.is_some()
    }

    /// The focused row.
    #[must_use]
    pub fn focused_row(&self, cfg: &AppConfig, fit: &dyn CockpitFit) -> Option<RowId> {
        let rows = model::rows(self.tab, self.ctx(cfg, fit));
        rows.get(self.focus_index().min(rows.len().saturating_sub(1)))
    }

    /// The mode.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The keyboard (meaningful in [`Mode::Keyboard`]).
    #[must_use]
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// The capture (meaningful in [`Mode::Remap`]).
    #[must_use]
    pub fn remap(&self) -> &RemapCapture {
        &self.remap
    }

    /// The current notice, if one is showing at `now`.
    #[must_use]
    pub fn notice(&self, now: Duration) -> Option<&'static str> {
        self.notice.filter(|n| now < n.until).map(|n| n.text)
    }

    /// Opens the keyboard on `cfg`'s name (previews, tests).
    pub fn start_keyboard(&mut self, cfg: &AppConfig) {
        self.keyboard.open(&cfg.custom_driver_name);
        self.mode = Mode::Keyboard;
    }

    /// Starts a capture at `now` (previews, tests).
    pub fn start_remap(&mut self, now: Duration) {
        self.remap = RemapCapture::start(now);
        self.mode = Mode::Remap;
    }

    /// The row context for `cfg`.
    #[must_use]
    pub fn ctx<'a>(&'a self, cfg: &'a AppConfig, fit: &'a dyn CockpitFit) -> Ctx<'a> {
        Ctx {
            cfg,
            edit: &self.edit,
            fit,
        }
    }

    fn say(&mut self, text: &'static str, now: Duration) {
        self.notice = Some(Notice {
            text,
            until: now + NOTICE_TIME,
        });
    }

    /// Feeds one frame of input. `cfg` is the working config; edits land in it directly.
    pub fn step(&mut self, input: &NavInput, cfg: &mut AppConfig, fit: &mut dyn CockpitFit) -> NavOutput {
        let held = input.held & !MASK_MANUAL_LOCK;
        let pressed = input.pressed & !MASK_MANUAL_LOCK;
        let released = input.released & !MASK_MANUAL_LOCK;
        let now = input.now;
        let mut out = NavOutput::default();
        match self.mode {
            Mode::Browse => self.browse(input, held, pressed, released, cfg, fit, &mut out),
            Mode::Keyboard => self.type_name(held, pressed, now, cfg, &mut out),
            Mode::Remap => match self.remap.update(held, now) {
                RemapOutcome::Pending => {}
                RemapOutcome::Committed(mask) => {
                    cfg.osd_toggle_mask = mask;
                    self.mode = Mode::Browse;
                    self.release_guard = true;
                    self.say("OSD button saved", now);
                    out.changed = true;
                    out.moved = true;
                }
                RemapOutcome::Cancelled => {
                    self.mode = Mode::Browse;
                    self.release_guard = true;
                    self.say("No change: nothing was held for 1 second", now);
                    out.moved = true;
                }
            },
        }
        if self.release_guard && held & !DPAD == 0 {
            self.release_guard = false;
        }
        out.settings_focus = self.mode != Mode::Browse || self.release_guard;
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn browse(
        &mut self,
        input: &NavInput,
        held: u16,
        pressed: u16,
        released: u16,
        cfg: &mut AppConfig,
        fit: &mut dyn CockpitFit,
        out: &mut NavOutput,
    ) {
        let now = input.now;
        if pressed & LB != 0 {
            self.tab = self.tab.step(-1);
            out.moved = true;
        }
        if pressed & RB != 0 {
            self.tab = self.tab.step(1);
            out.moved = true;
        }
        for (i, bit) in [VIEW, MENU].into_iter().enumerate() {
            let in_toggle = input.toggle_mask & bit != 0;
            if pressed & bit != 0 {
                if in_toggle {
                    self.back_down[i] = Some(now);
                } else {
                    out.close = true;
                }
            }
            if released & bit != 0 {
                if let Some(down) = self.back_down[i].take() {
                    if now.saturating_sub(down) < Duration::from_millis(u64::from(input.hold_ms)) {
                        out.close = true;
                    }
                }
            }
        }
        let fire = self.repeater.update(held, pressed, now);
        if fire.count == 0 {
            return;
        }
        let rows = model::rows(self.tab, self.ctx(cfg, &*fit));
        let n = rows.len();
        if n == 0 {
            return;
        }
        let ti = self.tab.index();
        let cur = self.focus[ti].min(n - 1);
        match fire.dir {
            UP | DOWN => {
                let d: i64 = if fire.dir == UP { -1 } else { 1 };
                let mut f = cur as i64;
                for _ in 0..fire.count {
                    f += d;
                    if fire.first {
                        f = f.rem_euclid(n as i64);
                    } else {
                        f = f.clamp(0, n as i64 - 1);
                    }
                }
                self.focus[ti] = f as usize;
                out.moved = true;
            }
            LEFT | RIGHT => {
                let Some(row) = rows.get(cur) else { return };
                let dir = if fire.dir == LEFT { -1 } else { 1 };
                let kind = model::kind(row, &*fit);
                let (times, mult) = match kind {
                    RowKind::Slider(_) => (1, fire.count as i32 * accel(fire.held_for)),
                    RowKind::Carousel => (fire.count, 1),
                    _ if fire.first => (1, 1),
                    _ => (0, 1),
                };
                for _ in 0..times {
                    let e = model::adjust(row, dir, mult, cfg, &mut self.edit, fit);
                    out.changed |= e.changed;
                    out.moved = true;
                    if e.keyboard {
                        self.start_keyboard(cfg);
                    }
                    if e.remap {
                        self.start_remap(now);
                    }
                    if e.rows {
                        let len = model::rows(self.tab, self.ctx(cfg, &*fit)).len();
                        self.focus[ti] = self.focus[ti].min(len.saturating_sub(1));
                    }
                }
            }
            _ => {}
        }
    }

    fn type_name(&mut self, held: u16, pressed: u16, now: Duration, cfg: &mut AppConfig, out: &mut NavOutput) {
        let fire = self.repeater.update(held, pressed, now);
        for _ in 0..fire.count {
            match fire.dir {
                UP => self.keyboard.move_focus(0, -1),
                DOWN => self.keyboard.move_focus(0, 1),
                LEFT => self.keyboard.move_focus(-1, 0),
                RIGHT => self.keyboard.move_focus(1, 0),
                _ => {}
            }
            out.moved = true;
        }
        let mut done = pressed & MENU != 0;
        if pressed & RB != 0 && self.keyboard.press() == KeyOutcome::Done {
            done = true;
        }
        if pressed & LB != 0 {
            self.keyboard.backspace();
        }
        out.moved |= pressed & (RB | LB | MENU | VIEW) != 0;
        if done {
            let name = self.keyboard.result();
            if cfg.custom_driver_name != name {
                cfg.custom_driver_name = name.to_string();
                out.changed = true;
            }
            self.mode = Mode::Browse;
            self.release_guard = true;
            self.say("Name saved", now);
        } else if pressed & VIEW != 0 {
            self.mode = Mode::Browse;
            self.release_guard = true;
            self.say("Name unchanged", now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fit::ConfigOnlyFit;
    use crate::model::RowId;

    const MS: fn(u64) -> Duration = Duration::from_millis;

    struct Rig {
        ctl: OsdController,
        cfg: AppConfig,
        fit: ConfigOnlyFit,
        held: u16,
    }

    impl Rig {
        fn new() -> Self {
            let cfg = AppConfig::default();
            let mut ctl = OsdController::new();
            ctl.open(&cfg, View::Cockpit);
            Self {
                ctl,
                cfg,
                fit: ConfigOnlyFit,
                held: 0,
            }
        }

        fn frame(&mut self, t: u64, press: u16, release: u16) -> NavOutput {
            self.held = (self.held | press) & !release;
            let input = NavInput {
                held: self.held,
                pressed: press,
                released: release,
                now: MS(t),
                toggle_mask: self.cfg.osd_toggle_mask,
                hold_ms: self.cfg.osd_hold_ms,
            };
            self.ctl.step(&input, &mut self.cfg, &mut self.fit)
        }

        fn tap(&mut self, t: u64, b: u16) -> NavOutput {
            let mut o = self.frame(t, b, 0);
            let r = self.frame(t + 16, 0, b);
            o.changed |= r.changed;
            o.close |= r.close;
            o
        }
    }

    #[test]
    fn bumpers_switch_tabs_and_wrap() {
        let mut r = Rig::new();
        r.tap(0, RB);
        assert_eq!(r.ctl.tab(), Tab::Colour);
        r.tap(100, LB);
        r.tap(200, LB);
        assert_eq!(r.ctl.tab(), Tab::Controls);
    }

    #[test]
    fn up_wraps_on_press_and_clamps_while_held() {
        let mut r = Rig::new();
        r.tap(0, UP);
        assert_eq!(r.ctl.focus_index(), 5, "wrapped to the last Cluster row");
        // Hold down: first press moves, repeats clamp at the end.
        r.frame(100, DOWN, 0);
        assert_eq!(r.ctl.focus_index(), 0, "press wraps from the last row");
        for t in (116..2000).step_by(16) {
            r.frame(t, 0, 0);
        }
        assert_eq!(r.ctl.focus_index(), 5, "repeats stop at the end");
    }

    #[test]
    fn repeat_timing_and_acceleration() {
        let mut rep = Repeater::default();
        assert_eq!(rep.update(RIGHT, RIGHT, MS(0)).count, 1);
        assert_eq!(rep.update(RIGHT, 0, MS(270)).count, 0);
        assert_eq!(rep.update(RIGHT, 0, MS(281)).count, 1);
        assert_eq!(rep.update(RIGHT, 0, MS(346)).count, 1);
        let late = rep.update(RIGHT, 0, MS(1500));
        assert!(late.count >= 1 && late.held_for == MS(1500));
        assert_eq!(rep.update(0, 0, MS(1516)).count, 0, "release stops it");
        assert_eq!(accel(MS(100)), 1);
        assert_eq!(accel(MS(1300)), 4);
        assert_eq!(accel(MS(3000)), 10);
    }

    #[test]
    fn held_slider_accelerates() {
        let mut r = Rig::new();
        assert!(r.ctl.focus_row(Tab::Fit, RowId::ClusterHeight, &r.cfg, &r.fit));
        assert_eq!(r.ctl.focused_row(&r.cfg, &r.fit), Some(RowId::ClusterHeight));
        r.frame(0, RIGHT, 0);
        let one = model::slider_value(RowId::ClusterHeight, r.ctl.ctx(&r.cfg, &r.fit));
        assert_eq!(one, 4.0);
        for t in (16..3000).step_by(16) {
            r.frame(t, 0, 0);
        }
        let v = model::slider_value(RowId::ClusterHeight, r.ctl.ctx(&r.cfg, &r.fit));
        assert_eq!(v, 240.0, "reaches the end within 3 s thanks to acceleration");
    }

    #[test]
    fn view_tap_closes_but_a_hold_does_not_double_fire() {
        let mut r = Rig::new();
        assert!(r.tap(0, VIEW).close);
        // Held past the hold time: the hold toggle closes the OSD, so release is not a back.
        let mut r = Rig::new();
        r.frame(0, VIEW, 0);
        assert!(!r.frame(500, 0, VIEW).close);
        // Menu is not in the default toggle mask: it acts on press.
        let mut r = Rig::new();
        assert!(r.frame(0, MENU, 0).close);
    }

    #[test]
    fn release_of_the_opening_hold_is_ignored() {
        let mut r = Rig::new();
        // View was pressed before the OSD opened (the hold that opened it).
        r.held = VIEW;
        assert!(!r.frame(0, 0, VIEW).close);
    }

    #[test]
    fn abxy_never_does_anything() {
        let mut r = Rig::new();
        let before = r.cfg.clone();
        for (i, b) in [0x1000u16, 0x2000, 0x4000, 0x8000, 0xF000].into_iter().enumerate() {
            let o = r.tap(i as u64 * 100, b);
            assert_eq!(o, NavOutput::default(), "{b:#06x}");
        }
        assert_eq!(r.ctl.tab(), Tab::Cluster);
        assert_eq!(r.ctl.focus_index(), 0);
        assert_eq!(r.cfg, before);
        // ABXY held together with the D-pad: only the D-pad counts.
        r.frame(1000, DOWN | 0x1000, 0);
        assert_eq!(r.ctl.focus_index(), 1);
    }

    #[test]
    fn right_on_custom_name_opens_the_keyboard_and_menu_saves() {
        let mut r = Rig::new();
        assert!(r.ctl.focus_row(Tab::Type, RowId::CustomName, &r.cfg, &r.fit));
        let o = r.tap(0, RIGHT);
        assert!(!o.changed);
        assert_eq!(r.ctl.mode(), Mode::Keyboard);
        assert!(r.frame(100, 0, 0).settings_focus);
        // Clear the name with LB, type "Q" with RB, Menu = done.
        for i in 0..20 {
            r.tap(200 + i * 40, LB);
        }
        r.tap(2000, RB);
        let o = r.tap(2100, MENU);
        assert!(o.changed);
        assert!(!o.close, "Menu is done in the keyboard, not back");
        assert_eq!(r.cfg.custom_driver_name, "Q");
        assert_eq!(r.ctl.mode(), Mode::Browse);
    }

    #[test]
    fn view_cancels_the_keyboard() {
        let mut r = Rig::new();
        let name = r.cfg.custom_driver_name.clone();
        r.ctl.start_keyboard(&r.cfg);
        r.tap(0, RB);
        let o = r.tap(100, VIEW);
        assert!(!o.close && !o.changed);
        assert_eq!(r.cfg.custom_driver_name, name);
    }

    #[test]
    fn remap_saves_a_sanitised_mask_and_guards_the_release() {
        let mut r = Rig::new();
        r.ctl.set_tab(Tab::Controls);
        r.tap(0, RIGHT);
        assert_eq!(r.ctl.mode(), Mode::Remap);
        // Holding ABXY + View + L3: only View + L3 is bound.
        r.frame(100, 0xF000 | VIEW | 0x0040, 0);
        let mut changed = false;
        for t in (116..1300).step_by(16) {
            let o = r.frame(t, 0, 0);
            changed |= o.changed;
            assert!(o.settings_focus, "toggle suspended while capturing");
            assert!(!o.close, "View inside a capture is not back");
        }
        assert!(changed);
        assert_eq!(r.cfg.osd_toggle_mask, VIEW | 0x0040);
        assert!(r.frame(1400, 0, 0).settings_focus, "still held: guard stays");
        let o = r.frame(1416, 0, 0xF000 | VIEW | 0x0040);
        assert!(!o.close, "the release after a capture is not a back");
        assert!(!r.frame(1432, 0, 0).settings_focus);
    }

    #[test]
    fn edit_view_and_carousel_follow_each_other() {
        let mut r = Rig::new();
        r.tap(0, RIGHT); // Edit view: Hood
        assert_eq!(r.ctl.edit.view, View::Hood);
        r.tap(100, DOWN);
        let before = model::theme_for_view(&r.cfg, View::Hood);
        let o = r.tap(200, RIGHT);
        assert!(o.changed);
        assert_eq!(model::theme_for_view(&r.cfg, View::Hood), model::theme_step(before, 1));
    }
}
