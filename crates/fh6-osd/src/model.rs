//! OSD data model, row definitions, and gamepad navigation.
use fh6_core::config::AppConfig;

pub const COCKPIT_STYLES: &[&str] = &[
    "BoschDdu",
    "DriftGlass",
    "PorscheAnalog",
    "LiquidGlass",
    "WindshieldHud",
    "TeslaAdas",
    "RetroCyber",
    "Gt7HoodPro",
    "AmgMbuxSuperscreen",
    "LfaV10Nurburgring",
    "PaganiHorologerie",
    "F35HelmetHmd",
    "BugattiTourbillon",
    "EvaBioSync",
    "KoenigseggGyro",
    "ValkyrieLmp1",
    "SynthwaveOutrun",
    "ChronoMultiverse",
    "ChronoEventHorizon",
    "NeuralCyberSynapse",
    "Gt7ModularTripod",
    "ArNavWayfinder",
    "SolarisWarpCore",
    "VisionSpatialGlass",
    "GlyphDotMatrix",
    "AeroPlasmaBlade",
    "CymaticResonanceField",
    "KineticLiquidMetal",
    "ShibuyaChromaticPrisma",
];

pub const BONNET_STYLES: &[&str] = &[
    "Gt7HoodPro",
    "DriftGlass",
    "WindshieldHud",
    "F35HelmetHmd",
    "LiquidGlass",
    "TeslaAdas",
    "AmgMbuxSuperscreen",
    "ValkyrieLmp1",
];

pub const SHIFT_PATTERNS: &[&str] = &["Converging", "GT3 F1 Sweep", "Progressive"];
pub const DRIFT_MODES: &[&str] = &["Column Track", "Subtle Parallax", "Off"];
pub const TAB_NAMES: &[&str] = &[
    "\u{300e}\u{8996}\u{300f} VIEW",       // 『視』 VIEW
    "\u{300e}\u{5100}\u{300f} CLUSTERS",   // 『儀』 CLUSTERS
    "\u{300e}\u{5149}\u{300f} OPTICS",     // 『光』 OPTICS
    "\u{300e}\u{8abf}\u{300f} TELEMETRY",  // 『調』 TELEMETRY
];

/// D-pad typematic stepping for Left/Right holds.
/// Instant step on rising edge, 280 ms initial delay, 65 ms repeat, 25 ms after 900 ms continuous hold.
/// Returns number of steps to apply given hold duration in ms from first press.
pub fn dpad_typematic_steps(hold_ms: u64) -> i32 {
    if hold_ms == 0 {
        return 1;
    }
    if hold_ms < 280 {
        return 1;
    }
    if hold_ms <= 900 {
        return 1 + ((hold_ms - 280) / 65) as i32;
    }
    // After 900ms, switch to 25ms repeat
    let steps_up_to_900 = 1 + ((900 - 280) / 65) as i32;
    let extra = (hold_ms - 900) / 25;
    steps_up_to_900 + extra as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsdTab {
    DisplaysStyles = 0,
    SeatingDimmer = 1,
    ShiftTelemetry = 2,
    Calibration = 3,
}

impl OsdTab {
    pub fn from_index(index: usize) -> Self {
        match index % 4 {
            0 => Self::DisplaysStyles,
            1 => Self::SeatingDimmer,
            2 => Self::ShiftTelemetry,
            _ => Self::Calibration,
        }
    }

    pub fn to_index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsdNavAction {
    TabNext,
    TabPrev,
    RowUp,
    RowDown,
    AdjustLeft,
    AdjustRight,
    Close,
}

/// Parses gamepad buttons into an OsdNavAction.
/// Strictly enforces T-INPUT-LOCK: A (0x1000), B (0x2000), X (0x4000), Y (0x8000) are NEVER read.
pub fn parse_gamepad_action(raw_buttons: u16, prev_buttons: u16) -> Option<OsdNavAction> {
    // 1. Manual Lock check: if A/B/X/Y (0xF000) are pressed, strictly REJECT completely!
    if (raw_buttons & 0xF000) != 0 {
        return None;
    }
    let safe_raw = raw_buttons;
    let safe_prev = prev_buttons & !0xF000;
    let pressed = safe_raw & !safe_prev;

    // View + Menu (0x0020 + 0x0010 = 0x0030) or single Menu toggles/closes
    if (safe_raw & 0x0030) == 0x0030 {
        return Some(OsdNavAction::Close);
    }

    // Bumpers switch tabs (LB=0x0100, RB=0x0200)
    if (pressed & 0x0100) != 0 {
        return Some(OsdNavAction::TabPrev);
    }
    if (pressed & 0x0200) != 0 {
        return Some(OsdNavAction::TabNext);
    }

    // D-Pad navigation
    if (pressed & 0x0001) != 0 {
        return Some(OsdNavAction::RowUp);
    }
    if (pressed & 0x0002) != 0 {
        return Some(OsdNavAction::RowDown);
    }
    if (pressed & 0x0004) != 0 {
        return Some(OsdNavAction::AdjustLeft);
    }
    if (pressed & 0x0008) != 0 {
        return Some(OsdNavAction::AdjustRight);
    }

    None
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnifiedOsdState {
    pub is_open: bool,
    pub current_tab: usize,
    pub selected_row: usize,
    pub seat_height_changed: bool,
}

impl UnifiedOsdState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_rows_for_tab(tab: usize) -> usize {
        match tab % 4 {
            0 => 4,
            1 => 5,
            2 => 4,
            3 => 5,
            _ => 10,
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = (self.current_tab + 1) % 4;
        self.selected_row = 0;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = (self.current_tab + 3) % 4;
        self.selected_row = 0;
    }

    pub fn row_up(&mut self) {
        let max_rows = Self::max_rows_for_tab(self.current_tab);
        if max_rows == 0 {
            self.selected_row = 0;
            return;
        }
        if self.selected_row == 0 {
            self.selected_row = max_rows - 1;
        } else {
            self.selected_row -= 1;
        }
    }

    pub fn row_down(&mut self) {
        let max_rows = Self::max_rows_for_tab(self.current_tab);
        if max_rows == 0 {
            self.selected_row = 0;
            return;
        }
        self.selected_row = (self.selected_row + 1) % max_rows;
    }

    pub fn handle_action(&mut self, action: OsdNavAction, config: &mut AppConfig) {
        match action {
            OsdNavAction::TabNext => self.next_tab(),
            OsdNavAction::TabPrev => self.prev_tab(),
            OsdNavAction::RowUp => self.row_up(),
            OsdNavAction::RowDown => self.row_down(),
            OsdNavAction::AdjustLeft => self.adjust_value(-1, config),
            OsdNavAction::AdjustRight => self.adjust_value(1, config),
            OsdNavAction::Close => self.is_open = false,
        }
    }

    pub fn adjust_value(&mut self, dir: i32, config: &mut AppConfig) {
        match self.current_tab {
            0 => self.adjust_tab0_row(self.selected_row, dir, config),
            1 => self.adjust_tab1_row(self.selected_row, dir, config),
            2 => self.adjust_tab2_row(self.selected_row, dir, config),
            3 => self.adjust_tab3_row(self.selected_row, dir, config),
            _ => {}
        }
    }

    fn adjust_tab0_row(&mut self, row: usize, dir: i32, config: &mut AppConfig) {
        match row {
            0 => {
                // Field of View
                let v = config.fov_degrees + dir;
                config.fov_degrees = v.clamp(45, 110);
                // Push to camera patcher atomics immediately - simulate by updating legacy fov fields
                config.standard_cockpit_fov = config.fov_degrees as f32;
            }
            1 => {
                // Seat Height
                let v = config.driver_seat_height_m + dir as f32 * 0.005;
                config.driver_seat_height_m = v.clamp(-0.100, 0.050);
            }
            2 => {
                // Wheel Offset
                let v = config.steering_wheel_offset_y_m + dir as f32 * 0.005;
                config.steering_wheel_offset_y_m = v.clamp(-0.080, 0.020);
            }
            3 => {
                // Camera View cycle
                use fh6_core::config::CameraViewMode;
                let cur = config.active_camera_view;
                let next = match cur {
                    CameraViewMode::Cockpit => if dir > 0 { CameraViewMode::Bonnet } else { CameraViewMode::Chase },
                    CameraViewMode::Bonnet => if dir > 0 { CameraViewMode::Chase } else { CameraViewMode::Cockpit },
                    CameraViewMode::Chase => if dir > 0 { CameraViewMode::Cockpit } else { CameraViewMode::Bonnet },
                };
                config.active_camera_view = next;
                // Sync legacy index
                config.current_camera_index = match next {
                    CameraViewMode::Cockpit => 3,
                    CameraViewMode::Bonnet => 4,
                    CameraViewMode::Chase => 0,
                };
            }
            _ => {}
        }
    }

    fn adjust_tab1_row(&mut self, row: usize, dir: i32, config: &mut AppConfig) {
        use fh6_core::theme::ClusterTheme;
        use fh6_core::config::FontPreset;
        match row {
            0 => {
                let all = ClusterTheme::all();
                let idx = all.iter().position(|&t| t == config.cockpit_hud_style_typed).unwrap_or(0);
                let len = all.len() as i32;
                let next = (((idx as i32 + dir) % len + len) % len) as usize;
                config.cockpit_hud_style_typed = all[next];
            }
            1 => {
                let all = ClusterTheme::all();
                let idx = all.iter().position(|&t| t == config.bonnet_hud_style_typed).unwrap_or(0);
                let len = all.len() as i32;
                let next = (((idx as i32 + dir) % len + len) % len) as usize;
                config.bonnet_hud_style_typed = all[next];
            }
            2 => {
                let all = ClusterTheme::all();
                let idx = all.iter().position(|&t| t == config.chase_hud_style_typed).unwrap_or(0);
                let len = all.len() as i32;
                let next = (((idx as i32 + dir) % len + len) % len) as usize;
                config.chase_hud_style_typed = all[next];
            }
            3 => {
                let v = config.cluster_scale + dir as f32 * 0.02;
                config.cluster_scale = v.clamp(0.70, 1.40);
            }
            4 => {
                let variants = [FontPreset::BahnschriftLight, FontPreset::SfPro, FontPreset::Tabular, FontPreset::VfdMono];
                let idx = variants.iter().position(|&p| p == config.font_preset).unwrap_or(0);
                let len = variants.len() as i32;
                let next = (((idx as i32 + dir) % len + len) % len) as usize;
                config.font_preset = variants[next];
            }
            _ => {}
        }
    }

    fn adjust_tab2_row(&mut self, row: usize, dir: i32, config: &mut AppConfig) {
        match row {
            0 => {
                config.ambient_glow_enabled = !config.ambient_glow_enabled;
            }
            1 => {
                let v = config.ambient_glow_intensity + dir as f32 * 0.05;
                config.ambient_glow_intensity = v.clamp(0.0, 1.0);
            }
            2 => {
                let v = config.ambient_cowl_feather + dir as f32 * 0.05;
                config.ambient_cowl_feather = v.clamp(0.05, 1.0);
            }
            3 => {
                config.oem_dash_opaque_pct = (config.oem_dash_opaque_pct + dir * 5).clamp(0, 100);
            }
            _ => {}
        }
    }

    fn adjust_tab3_row(&mut self, row: usize, dir: i32, config: &mut AppConfig) {
        match row {
            0 => {
                config.driver_tag_glitch_fx = !config.driver_tag_glitch_fx;
            }
            1 => {
                config.speed_unit = if config.speed_unit.eq_ignore_ascii_case("KMH") {
                    "MPH".to_string()
                } else {
                    "KMH".to_string()
                };
            }
            2 => {
                config.power_unit = if config.power_unit.eq_ignore_ascii_case("HP") {
                    "kW".to_string()
                } else {
                    "HP".to_string()
                };
            }
            3 => {
                let v = config.shift_start_pct + dir as f32;
                config.shift_start_pct = v.clamp(0.0, 100.0);
            }
            4 => {
                config.redline_strobe = !config.redline_strobe;
            }
            _ => {}
        }
    }
}

pub fn get_row_label(tab: usize, row: usize) -> &'static str {
    match tab {
        0 => match row {
            0 => "Field of View",
            1 => "Seat Height",
            2 => "Wheel Offset",
            3 => "Camera View",
            _ => "Row",
        },
        1 => match row {
            0 => "Cockpit HUD",
            1 => "Bonnet HUD",
            2 => "Chase HUD",
            3 => "Cluster Scale",
            4 => "Font",
            _ => "Row",
        },
        2 => match row {
            0 => "Ambient Glow",
            1 => "Glow Intensity",
            2 => "Cowl Feather",
            3 => "Scrim Opacity",
            _ => "Row",
        },
        3 => match row {
            0 => "Driver Tag Glitch",
            1 => "Speed Units",
            2 => "Power Units",
            3 => "Shift Point %",
            4 => "Redline Strobe",
            _ => "Row",
        },
        _ => "Row",
    }
}

pub fn get_row_value(tab: usize, row: usize, config: &AppConfig) -> String {
    use fh6_core::config::CameraViewMode;
    match tab {
        0 => match row {
            0 => format!("{}°", config.fov_degrees),
            1 => format!("{:.3}m", config.driver_seat_height_m),
            2 => format!("{:.3}m", config.steering_wheel_offset_y_m),
            3 => {
                let s = match config.active_camera_view {
                    CameraViewMode::Cockpit => "Cockpit",
                    CameraViewMode::Bonnet => "Bonnet",
                    CameraViewMode::Chase => "Chase",
                };
                s.to_string()
            }
            _ => "—".to_string(),
        },
        1 => match row {
            0 => format!("{:?}", config.cockpit_hud_style_typed),
            1 => format!("{:?}", config.bonnet_hud_style_typed),
            2 => format!("{:?}", config.chase_hud_style_typed),
            3 => format!("{:.2}x", config.cluster_scale),
            4 => format!("{:?}", config.font_preset),
            _ => "—".to_string(),
        },
        2 => match row {
            0 => if config.ambient_glow_enabled { "ON" } else { "OFF" }.to_string(),
            1 => format!("{:.2}", config.ambient_glow_intensity),
            2 => format!("{:.2}", config.ambient_cowl_feather),
            3 => format!("{}%", config.oem_dash_opaque_pct),
            _ => "—".to_string(),
        },
        3 => match row {
            0 => if config.driver_tag_glitch_fx { "ON" } else { "OFF" }.to_string(),
            1 => {
                if config.speed_unit.is_empty() { "MPH".to_string() } else { config.speed_unit.clone() }
            }
            2 => {
                if config.power_unit.is_empty() { "kW".to_string() } else { config.power_unit.clone() }
            }
            3 => format!("{}%", config.shift_start_pct.round() as i32),
            4 => if config.redline_strobe { "ON" } else { "OFF" }.to_string(),
            _ => "—".to_string(),
        },
        _ => "—".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_and_row_navigation() {
        let mut state = UnifiedOsdState::new();
        assert_eq!(state.current_tab, 0);
        assert_eq!(state.selected_row, 0);

        state.row_down();
        assert_eq!(state.selected_row, 1);

        state.row_up();
        assert_eq!(state.selected_row, 0);

        state.row_up(); // wraps to last
        assert_eq!(state.selected_row, 3);

        state.next_tab();
        assert_eq!(state.current_tab, 1);
        assert_eq!(state.selected_row, 0);

        state.prev_tab();
        assert_eq!(state.current_tab, 0);
    }

    #[test]
    fn test_t_input_lock_no_abxy_handled() {
        // Manual-gearbox lock: A (0x1000), B (0x2000), X (0x4000), Y (0x8000)
        let abxy_buttons = [0x1000u16, 0x2000, 0x4000, 0x8000];
        for btn in abxy_buttons {
            let act = parse_gamepad_action(btn, 0);
            assert_eq!(
                act, None,
                "Manual lock button 0x{:04X} must NOT trigger any action",
                btn
            );
        }

        // Even if combined with D-Pad or bumpers, ABXY must be completely ignored
        let act_a_plus_up = parse_gamepad_action(0x1000 | 0x0001, 0);
        assert_eq!(
            act_a_plus_up, None,
            "A button pressed along with D-Pad Up must not register"
        );

        let act_b_plus_rb = parse_gamepad_action(0x2000 | 0x0200, 0);
        assert_eq!(act_b_plus_rb, None, "B button pressed along with RB must not register");
    }

    #[test]
    fn test_adjust_values() {
        let mut state = UnifiedOsdState::new();
        let mut config = AppConfig::default();
        config.fov_degrees = 65;
        config.oem_dash_opaque_pct = 50;

        // Tab 0, Row 0: FOV
        state.selected_row = 0;
        state.adjust_value(1, &mut config);
        assert_eq!(config.fov_degrees, 66);

        // Tab 2, Row 3: Scrim Opacity %
        state.current_tab = 2;
        state.selected_row = 3;
        state.adjust_value(1, &mut config);
        assert_eq!(config.oem_dash_opaque_pct, 55);
        state.adjust_value(-2, &mut config);
        assert_eq!(config.oem_dash_opaque_pct, 45);
    }

    #[test]
    fn test_abxy_controller_lock_invariant() {
        let buttons = [0x1000u16, 0x2000, 0x4000, 0x8000];
        for btn in buttons {
            let act = parse_gamepad_action(btn, 0);
            assert_eq!(act, None, "ABXY must not trigger action");
        }
    }

    #[test]
    fn test_modal_rb_gate_and_bonnet_scrim_override() {
        let mut cfg = AppConfig::default();
        cfg.active_camera_view = fh6_core::config::CameraViewMode::Bonnet;
        cfg.bonnet_hud_style_typed = fh6_core::theme::ClusterTheme::Gt7HoodPro;
        cfg.oem_dash_opaque_pct = 100;
        let (theme, scrim) = cfg.effective_cluster_and_scrim();
        assert_eq!(theme, fh6_core::theme::ClusterTheme::Gt7HoodPro);
        assert_eq!(scrim, 0.0);
    }

    #[test]
    fn test_dpad_typematic_timing() {
        use super::dpad_typematic_steps;
        assert_eq!(dpad_typematic_steps(0), 1);
        assert_eq!(dpad_typematic_steps(100), 1);
        assert_eq!(dpad_typematic_steps(279), 1);
        assert_eq!(dpad_typematic_steps(280), 1);
        assert_eq!(dpad_typematic_steps(344), 1);
        assert_eq!(dpad_typematic_steps(345), 2);
        // At 900ms, steps = 1 + floor((900-280)/65) = 1 + 9 =10
        assert_eq!(dpad_typematic_steps(900), 10);
        // After 900ms, 25ms repeat
        assert_eq!(dpad_typematic_steps(925), 11);
        assert_eq!(dpad_typematic_steps(950), 12);
        assert_eq!(dpad_typematic_steps(975), 13);
    }
}
