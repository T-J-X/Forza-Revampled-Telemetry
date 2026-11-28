//! Controller tracker: the RB camera ring, the OSD hold toggle, OSD tab switching and the
//! Guide edge, all with the manual-gearbox lock (A/B/X/Y are stripped before anything looks
//! at the buttons).
use crate::guide::GuideEdgeDetector;
use crate::hold_state::{HoldConfig, HoldToggle};
use crate::xinput::{XInputState, XINPUT_GAMEPAD_GUIDE};

pub const XINPUT_GAMEPAD_DPAD_UP: u16 = 0x0001;
pub const XINPUT_GAMEPAD_DPAD_DOWN: u16 = 0x0002;
pub const XINPUT_GAMEPAD_DPAD_LEFT: u16 = 0x0004;
pub const XINPUT_GAMEPAD_DPAD_RIGHT: u16 = 0x0008;
pub const XINPUT_GAMEPAD_START: u16 = 0x0010;
pub const XINPUT_GAMEPAD_BACK: u16 = 0x0020;
pub const XINPUT_GAMEPAD_LEFT_THUMB: u16 = 0x0040;
pub const XINPUT_GAMEPAD_RIGHT_THUMB: u16 = 0x0080;
pub const XINPUT_GAMEPAD_LEFT_SHOULDER: u16 = 0x0100;
pub const XINPUT_GAMEPAD_RIGHT_SHOULDER: u16 = 0x0200;
pub const XINPUT_GAMEPAD_A: u16 = 0x1000;
pub const XINPUT_GAMEPAD_B: u16 = 0x2000;
pub const XINPUT_GAMEPAD_X: u16 = 0x4000;
pub const XINPUT_GAMEPAD_Y: u16 = 0x8000;

/// A/B/X/Y: upshift, downshift, handbrake, rewind. Never read.
pub use fh6_core::config::MASK_MANUAL_LOCK;
pub const MASK_DPAD: u16 = 0x000F;
pub const MASK_VIEW_MENU: u16 = 0x0030;
pub const MASK_LB_RB: u16 = 0x0300;
pub const MASK_L3_R3: u16 = 0x00C0;

/// Number of camera positions in the game's RB ring.
pub const CAMERA_RING_LEN: usize = 6;
/// Ignore a second RB or LB edge within this many ms.
const SHOULDER_DEBOUNCE_MS: u64 = 150;
/// Ignore a second L3+R3 chord within this many ms.
const STUDIO_DEBOUNCE_MS: u64 = 400;
/// How often an empty controller slot is probed again.
const EMPTY_SLOT_PROBE_MS: u64 = 1000;
/// RB only cycles the camera once racing has run this long (the game ignores RB on resume).
const RESUME_GUARD_MS: f64 = 150.0;

/// The camera resync chord: hold LB + RB this long and the camera index is set to
/// `camera_index` (the camera the game is on after the chord; `ResyncCameraIndex`, default 3 =
/// cockpit). Never A/B/X/Y.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResyncConfig {
    /// Index the ring is set to (0..6).
    pub camera_index: usize,
    /// Hold time in ms (`ResyncHoldMs`, 500..=3000).
    pub hold_ms: u64,
}

impl Default for ResyncConfig {
    fn default() -> Self {
        Self {
            camera_index: 3,
            hold_ms: 1000,
        }
    }
}

impl ResyncConfig {
    /// From `ResyncCameraIndex` and `ResyncHoldMs`.
    pub fn from_config(cfg: &fh6_core::AppConfig) -> Self {
        Self {
            camera_index: usize::try_from(cfg.resync_camera_index.clamp(0, CAMERA_RING_LEN as i32 - 1)).unwrap_or(3),
            hold_ms: u64::from(cfg.resync_hold_ms.clamp(
                fh6_core::config::RESYNC_HOLD_MS_MIN,
                fh6_core::config::RESYNC_HOLD_MS_MAX,
            )),
        }
    }
}

/// Something that reports controller states per slot (0..=3).
pub trait PadSource {
    /// The state of `slot`, or `None` when nothing is connected there.
    fn poll_slot(&mut self, slot: usize) -> Option<XInputState>;
}

/// What the tracker needs to know about the world for one poll.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PollContext {
    /// Monotonic time in ms (any epoch).
    pub now_ms: u64,
    /// Racing with a fresh telemetry stream.
    pub driving_active: bool,
    /// How long racing has been running, in ms.
    pub ms_since_resume: f64,
    /// Demo mode: allow the camera ring without telemetry.
    pub demo_mode: bool,
    /// A text field or remapper has focus: the OSD hold toggle is suspended.
    pub settings_focused: bool,
}

/// What happened during one poll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InputEvents {
    /// RB advanced the camera ring.
    pub camera_changed: bool,
    /// LB + RB held for the resync time set the camera index (`ResyncConfig`).
    pub camera_resynced: bool,
    /// The hold toggle opened or closed the OSD.
    pub osd_toggled: bool,
    /// LB while the OSD is open.
    pub tab_prev: bool,
    /// RB while the OSD is open.
    pub tab_next: bool,
    /// L3+R3 chord.
    pub studio_toggled: bool,
    /// Guide button pressed (only when `XInputGetStateEx` is available).
    pub guide_pressed: bool,
}

impl InputEvents {
    /// True if anything happened.
    pub fn any(&self) -> bool {
        *self != Self::default()
    }
}

/// Edge detection and debouncing over up to four controllers.
pub struct InputControllerTracker<S: PadSource> {
    /// Whether the OSD is open. The app may also set this (tray "Open OSD").
    pub is_osd_open: bool,
    prev_buttons: [u16; 4],
    latest_buttons: u16,
    connected: bool,
    slot_live: [bool; 4],
    next_probe_ms: [u64; 4],
    hold: HoldToggle,
    guide: GuideEdgeDetector,
    last_shoulder_ms: Option<u64>,
    last_studio_ms: Option<u64>,
    l3r3_held: bool,
    resync: ResyncConfig,
    chord_since: Option<u64>,
    chord_fired: bool,
    pad_source: S,
}

impl<S: PadSource> std::fmt::Debug for InputControllerTracker<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InputControllerTracker")
            .field("is_osd_open", &self.is_osd_open)
            .field("latest_buttons", &self.latest_buttons)
            .finish_non_exhaustive()
    }
}

fn debounced(last: Option<u64>, now: u64, window: u64) -> bool {
    last.is_none_or(|t| now.saturating_sub(t) >= window)
}

impl<S: PadSource> InputControllerTracker<S> {
    /// A tracker reading from `pad_source`, with the OSD hold toggle set by `hold`.
    pub fn new(pad_source: S, hold: HoldConfig) -> Self {
        Self {
            is_osd_open: false,
            prev_buttons: [0; 4],
            latest_buttons: 0,
            connected: false,
            slot_live: [false; 4],
            next_probe_ms: [0; 4],
            hold: HoldToggle::new(hold),
            guide: GuideEdgeDetector::new(true),
            last_shoulder_ms: None,
            last_studio_ms: None,
            l3r3_held: false,
            resync: ResyncConfig::default(),
            chord_since: None,
            chord_fired: false,
            pad_source,
        }
    }

    /// Replaces the hold toggle settings (config hot reload).
    pub fn set_hold_config(&mut self, hold: HoldConfig) {
        self.hold.set_config(hold);
    }

    /// Replaces the resync chord settings (config hot reload).
    pub fn set_resync_config(&mut self, resync: ResyncConfig) {
        self.resync = resync;
    }

    /// The A/B/X/Y-stripped buttons of every connected pad, OR-ed together.
    pub fn buttons(&self) -> u16 {
        self.latest_buttons
    }

    /// True if any controller answered the last poll.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    fn safe_buttons(raw: u16) -> u16 {
        raw & !MASK_MANUAL_LOCK
    }

    /// Polls all four slots once. `camera_index` is advanced on an RB edge when allowed.
    pub fn poll(&mut self, camera_index: &mut usize, ctx: &PollContext) -> InputEvents {
        let mut events = InputEvents::default();
        let mut combined = 0u16;
        let mut rising = 0u16;
        let mut connected = false;
        let now = ctx.now_ms;
        for (slot, prev) in self.prev_buttons.iter_mut().enumerate() {
            // XInputGetState on an empty slot is slow; probe empty slots once a second only.
            let probe_due = self.slot_live[slot] || now >= self.next_probe_ms[slot];
            let state = if probe_due {
                self.pad_source.poll_slot(slot)
            } else {
                None
            };
            if probe_due {
                self.slot_live[slot] = state.is_some();
                if state.is_none() {
                    self.next_probe_ms[slot] = now.saturating_add(EMPTY_SLOT_PROBE_MS);
                }
            }
            let safe = match state {
                Some(state) => {
                    connected = true;
                    Self::safe_buttons(state.Gamepad.wButtons)
                }
                None => 0,
            };
            rising |= safe & !*prev;
            combined |= safe;
            *prev = safe;
        }
        self.latest_buttons = combined;
        self.connected = connected;

        if !ctx.settings_focused && self.hold.update(combined, now) {
            self.is_osd_open = !self.is_osd_open;
            events.osd_toggled = true;
        }

        let l3r3 = (combined & MASK_L3_R3) == MASK_L3_R3;
        if l3r3 && !self.l3r3_held && debounced(self.last_studio_ms, now, STUDIO_DEBOUNCE_MS) {
            self.last_studio_ms = Some(now);
            events.studio_toggled = true;
        }
        self.l3r3_held = l3r3;

        let shoulder_ok = debounced(self.last_shoulder_ms, now, SHOULDER_DEBOUNCE_MS);
        if self.is_osd_open {
            if shoulder_ok && rising & XINPUT_GAMEPAD_LEFT_SHOULDER != 0 {
                self.last_shoulder_ms = Some(now);
                events.tab_prev = true;
            } else if shoulder_ok && rising & XINPUT_GAMEPAD_RIGHT_SHOULDER != 0 {
                self.last_shoulder_ms = Some(now);
                events.tab_next = true;
            }
        } else if rising & XINPUT_GAMEPAD_RIGHT_SHOULDER != 0 && shoulder_ok {
            let guard = (ctx.driving_active && ctx.ms_since_resume >= RESUME_GUARD_MS) || ctx.demo_mode;
            if guard {
                *camera_index = (*camera_index + 1) % CAMERA_RING_LEN;
                self.last_shoulder_ms = Some(now);
                events.camera_changed = true;
            }
        }

        // Resync chord: LB + RB held (OSD closed). The RB edge that starts it still advances the
        // ring like the game does; once the hold completes the index is set outright, once per
        // hold.
        let chord = combined & MASK_LB_RB == MASK_LB_RB;
        if chord && !self.is_osd_open {
            let since = *self.chord_since.get_or_insert(now);
            if !self.chord_fired && now.saturating_sub(since) >= self.resync.hold_ms {
                self.chord_fired = true;
                *camera_index = self.resync.camera_index % CAMERA_RING_LEN;
                events.camera_resynced = true;
            }
        } else {
            self.chord_since = None;
            self.chord_fired = false;
        }

        events.guide_pressed = self.guide.update(combined & XINPUT_GAMEPAD_GUIDE != 0);
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xinput::XInputGamepad;
    use std::collections::VecDeque;

    /// Reports one scripted state per poll on slot 0; other slots are empty.
    struct FakePad {
        states: VecDeque<u16>,
        current: Option<u16>,
    }

    impl FakePad {
        fn new(states: &[u16]) -> Self {
            Self {
                states: states.iter().copied().collect(),
                current: None,
            }
        }
    }

    impl PadSource for FakePad {
        fn poll_slot(&mut self, slot: usize) -> Option<XInputState> {
            if slot == 0 {
                self.current = self.states.pop_front();
            }
            match (slot, self.current) {
                (0, Some(buttons)) => Some(XInputState {
                    Gamepad: XInputGamepad {
                        wButtons: buttons,
                        ..XInputGamepad::default()
                    },
                    ..XInputState::default()
                }),
                _ => None,
            }
        }
    }

    fn driving(now_ms: u64) -> PollContext {
        PollContext {
            now_ms,
            driving_active: true,
            ms_since_resume: 1000.0,
            ..PollContext::default()
        }
    }

    #[test]
    fn rb_rising_edge_advances_camera_ring() {
        let mut t =
            InputControllerTracker::new(FakePad::new(&[0, XINPUT_GAMEPAD_RIGHT_SHOULDER]), HoldConfig::default());
        let mut cam = 3usize;
        t.poll(&mut cam, &driving(0));
        let ev = t.poll(&mut cam, &driving(16));
        assert!(ev.camera_changed);
        assert_eq!(cam, 4, "cockpit [3] -> hood [4]");
    }

    #[test]
    fn camera_ring_wraps() {
        let mut t = InputControllerTracker::new(FakePad::new(&[XINPUT_GAMEPAD_RIGHT_SHOULDER]), HoldConfig::default());
        let mut cam = 5usize;
        t.poll(&mut cam, &driving(0));
        assert_eq!(cam, 0);
    }

    #[test]
    fn rb_ignored_when_not_driving() {
        let mut t = InputControllerTracker::new(FakePad::new(&[XINPUT_GAMEPAD_RIGHT_SHOULDER]), HoldConfig::default());
        let mut cam = 3usize;
        let ev = t.poll(&mut cam, &PollContext::default());
        assert!(!ev.camera_changed);
        assert_eq!(cam, 3);
    }

    #[test]
    fn rb_with_osd_open_switches_tab_only() {
        let mut t = InputControllerTracker::new(FakePad::new(&[XINPUT_GAMEPAD_RIGHT_SHOULDER]), HoldConfig::default());
        t.is_osd_open = true;
        let mut cam = 3usize;
        let ev = t.poll(&mut cam, &driving(0));
        assert!(ev.tab_next);
        assert_eq!(cam, 3);
    }

    #[test]
    fn manual_lock_buttons_are_never_seen() {
        let all_abxy = XINPUT_GAMEPAD_A | XINPUT_GAMEPAD_B | XINPUT_GAMEPAD_X | XINPUT_GAMEPAD_Y;
        let mut t = InputControllerTracker::new(
            FakePad::new(&[all_abxy | XINPUT_GAMEPAD_RIGHT_SHOULDER]),
            HoldConfig {
                toggle_mask: 0xF000,
                hold_ms: 0,
            },
        );
        let mut cam = 3usize;
        t.poll(&mut cam, &driving(0));
        assert_eq!(t.buttons() & MASK_MANUAL_LOCK, 0);
        assert_ne!(t.buttons() & XINPUT_GAMEPAD_RIGHT_SHOULDER, 0);
    }

    #[test]
    fn view_hold_toggles_osd_from_config() {
        let held = [XINPUT_GAMEPAD_BACK; 3];
        let mut t = InputControllerTracker::new(
            FakePad::new(&held),
            HoldConfig {
                toggle_mask: 0x0020,
                hold_ms: 400,
            },
        );
        let mut cam = 3usize;
        assert!(!t.poll(&mut cam, &driving(0)).osd_toggled);
        assert!(!t.poll(&mut cam, &driving(200)).osd_toggled);
        assert!(t.poll(&mut cam, &driving(400)).osd_toggled);
        assert!(t.is_osd_open);
    }

    #[test]
    fn view_menu_chord_alone_does_not_use_abxy() {
        // Holding A (handbrake) for a long time must never toggle the OSD.
        let mut t = InputControllerTracker::new(FakePad::new(&[XINPUT_GAMEPAD_A; 4]), HoldConfig::default());
        let mut cam = 3usize;
        for now in [0, 500, 1000, 2000] {
            assert!(!t.poll(&mut cam, &driving(now)).osd_toggled);
        }
    }

    #[test]
    fn empty_slots_are_probed_once_a_second() {
        struct CountingPad {
            calls: [u32; 4],
        }
        impl PadSource for CountingPad {
            fn poll_slot(&mut self, slot: usize) -> Option<XInputState> {
                self.calls[slot] += 1;
                (slot == 0).then(XInputState::default)
            }
        }
        let mut t = InputControllerTracker::new(CountingPad { calls: [0; 4] }, HoldConfig::default());
        let mut cam = 3usize;
        for i in 0..125u64 {
            t.poll(&mut cam, &driving(i * 8));
        }
        assert_eq!(t.pad_source.calls[0], 125, "a connected pad is read every poll");
        assert_eq!(t.pad_source.calls[1], 1, "an empty slot is probed once in 1 s");
    }

    #[test]
    fn holding_lb_rb_resyncs_the_camera_once() {
        const LB_RB: u16 = XINPUT_GAMEPAD_LEFT_SHOULDER | XINPUT_GAMEPAD_RIGHT_SHOULDER;
        let mut t = InputControllerTracker::new(
            FakePad::new(&[XINPUT_GAMEPAD_LEFT_SHOULDER, LB_RB, LB_RB, LB_RB, LB_RB, 0]),
            HoldConfig::default(),
        );
        let mut cam = 0usize;
        assert!(!t.poll(&mut cam, &driving(0)).camera_resynced);
        let ev = t.poll(&mut cam, &driving(100));
        assert!(
            ev.camera_changed && !ev.camera_resynced,
            "the RB edge still mirrors the game"
        );
        assert_eq!(cam, 1);
        assert!(!t.poll(&mut cam, &driving(600)).camera_resynced, "not held long enough");
        let ev = t.poll(&mut cam, &driving(1100));
        assert!(ev.camera_resynced);
        assert_eq!(cam, 3, "default ResyncCameraIndex = cockpit");
        cam = 5;
        assert!(!t.poll(&mut cam, &driving(2500)).camera_resynced, "once per hold");
        assert_eq!(cam, 5);
        assert!(!t.poll(&mut cam, &driving(2600)).camera_resynced);
    }

    #[test]
    fn resync_follows_config_and_ignores_abxy_and_the_open_osd() {
        const LB_RB: u16 = XINPUT_GAMEPAD_LEFT_SHOULDER | XINPUT_GAMEPAD_RIGHT_SHOULDER;
        let cfg = fh6_core::AppConfig {
            resync_camera_index: 4,
            resync_hold_ms: 500,
            ..fh6_core::AppConfig::default()
        };
        let abxy = XINPUT_GAMEPAD_A | XINPUT_GAMEPAD_B | XINPUT_GAMEPAD_X | XINPUT_GAMEPAD_Y;
        let mut t = InputControllerTracker::new(FakePad::new(&[LB_RB | abxy, LB_RB | abxy]), HoldConfig::default());
        t.set_resync_config(ResyncConfig::from_config(&cfg));
        let mut cam = 0usize;
        t.poll(&mut cam, &driving(0));
        assert!(t.poll(&mut cam, &driving(500)).camera_resynced);
        assert_eq!(cam, 4);

        let mut t = InputControllerTracker::new(FakePad::new(&[LB_RB, LB_RB]), HoldConfig::default());
        t.is_osd_open = true;
        let mut cam = 0usize;
        t.poll(&mut cam, &driving(0));
        assert!(
            !t.poll(&mut cam, &driving(2000)).camera_resynced,
            "LB/RB switch tabs in the OSD"
        );
        assert_eq!(cam, 0);

        let mut t = InputControllerTracker::new(FakePad::new(&[abxy, abxy]), HoldConfig::default());
        t.poll(&mut cam, &driving(0));
        assert!(
            !t.poll(&mut cam, &driving(2000)).camera_resynced,
            "A/B/X/Y never resync"
        );
    }

    #[test]
    fn resync_config_clamps() {
        let cfg = fh6_core::AppConfig {
            resync_camera_index: 99,
            resync_hold_ms: 10,
            ..fh6_core::AppConfig::default()
        };
        let r = ResyncConfig::from_config(&cfg);
        assert_eq!(r.camera_index, 5);
        assert_eq!(r.hold_ms, 500);
    }

    #[test]
    fn bonnet_index_maps_to_bonnet_view() {
        use fh6_core::FitView;
        assert_eq!(FitView::from_camera_index(4), FitView::Hood);
    }
}
