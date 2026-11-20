//! AppConfig – byte-compatible serde loader for config.json
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::theme::ClusterTheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AppConfig {
    #[serde(default)]
    pub version: String,

    // Window & Layout Anchors
    #[serde(default)]
    pub cockpit_anchor_x: i32,
    #[serde(default)]
    pub cockpit_anchor_y: i32,
    #[serde(default)]
    pub cockpit_scale: f32,
    #[serde(default)]
    pub bonnet_anchor_x: i32,
    #[serde(default)]
    pub bonnet_anchor_y: i32,
    #[serde(default)]
    pub bonnet_scale: f32,

    // Camera & Styles
    #[serde(default)]
    pub current_camera_index: i32,
    #[serde(default)]
    pub cockpit_style: String,
    #[serde(default)]
    pub bonnet_style: String,
    #[serde(default)]
    pub hud_style: String,
    #[serde(default)]
    pub cockpit_hud_style: String,
    #[serde(default)]
    pub bonnet_hud_style: String,
    #[serde(default)]
    pub oem_dash_opaque_pct: i32,
    #[serde(default)]
    pub oem_dash_binnacle_offset_y: i32,
    #[serde(default)]
    pub oem_dash_width_scale: f32,
    #[serde(default)]
    pub osd_auto_pause_game: bool,
    #[serde(default)]
    pub osd_live_dyno_preview_when_paused: bool,
    #[serde(default)]
    pub drift_sync_mode: String,
    #[serde(default)]
    pub drift_sync_sensitivity: f32,
    #[serde(default)]
    pub drift_sync_smoothing: f32,
    #[serde(default)]
    pub drift_sync_max_offset_px: f32,
    #[serde(default)]
    pub lock_visible_while_driving: bool,

    // CARTEK Shift Pod
    #[serde(default)]
    pub shift_light_pattern: String,
    #[serde(default)]
    pub shift_start_pct: f32,
    #[serde(default)]
    pub redline_pct: f32,

    // Controller
    #[serde(default)]
    pub controller_camera_button: String,

    // Units & Features
    #[serde(default)]
    pub speed_unit: String,
    #[serde(default)]
    pub show_bonnet_gt_widgets: bool,
    #[serde(default)]
    pub demo_mode: bool,

    // Per-Car / OLED / Shift Guard
    #[serde(default)]
    pub per_car_profile_enabled: bool,
    #[serde(default)]
    pub oled_pixel_orbit_enabled: bool,
    #[serde(default)]
    pub oled_idle_dimmer_enabled: bool,
    #[serde(default)]
    pub oled_dimmer_min_alpha: f32,
    #[serde(default)]
    pub manual_shift_guard_enabled: bool,
    #[serde(default = "default_driver_name")]
    pub custom_driver_name: String,
    #[serde(default)]
    pub driver_tag_glitch_fx: bool,
    #[serde(default)]
    pub power_unit: String,
    #[serde(default)]
    pub redline_strobe: bool,

    // Night Glow
    #[serde(default)]
    pub night_glow_mode: String,
    #[serde(default)]
    pub night_glow_intensity: f32,
    #[serde(default)]
    pub night_ambient_glow_enabled: bool,
    #[serde(default)]
    pub night_ambient_glow_intensity_pct: i32,

    // Modular Cockpit
    #[serde(default)]
    pub adas_backdrop_mode: i32,
    #[serde(default)]
    pub center_console_map_mode: i32,
    #[serde(default)]
    pub center_console_infotainment_mode: i32,
    #[serde(default)]
    pub route_active: bool,
    #[serde(default)]
    pub oem_dash_dimmer_enabled: bool,
    #[serde(default)]
    pub modular_side_pods_enabled: bool,
    #[serde(default)]
    pub steering_hub_cap_enabled: bool,

    // Ports
    #[serde(default)]
    pub udp_port: i32,
    #[serde(default)]
    pub fallback_port: i32,
    #[serde(default)]
    pub forward_port: i32,

    // Brand & Theme
    #[serde(default)]
    pub color_mode: String,
    #[serde(default)]
    pub custom_brand_key: String,
    #[serde(default)]
    pub primary_hex: String,
    #[serde(default)]
    pub secondary_hex: String,
    #[serde(default)]
    pub accent_hex: String,
    #[serde(default)]
    pub shift_flash_hex: String,

    // Widget overrides
    #[serde(default)]
    pub use_widget_color_overrides: bool,
    #[serde(default)]
    pub speed_color_hex: String,
    #[serde(default)]
    pub gear_color_hex: String,
    #[serde(default)]
    pub g_force_color_hex: String,
    #[serde(default)]
    pub suspension_color_hex: String,

    // FOV & offsets
    #[serde(default)]
    pub wheel_less_cockpit_fov: f32,
    #[serde(default)]
    pub standard_cockpit_fov: f32,
    #[serde(default)]
    pub bonnet_fov: f32,
    #[serde(default)]
    pub chase_near_fov: f32,
    #[serde(default)]
    pub chase_far_fov: f32,
    #[serde(default)]
    pub bumper_fov: f32,
    #[serde(default)]
    pub cockpit_offset_x: f32,
    #[serde(default)]
    pub cockpit_offset_y: f32,
    #[serde(default)]
    pub cockpit_offset_z: f32,
    #[serde(default)]
    pub hood_offset_y: f32,
    #[serde(default)]
    pub hood_offset_z: f32,
    #[serde(default)]
    pub hood_pitch_offset: f32,

    // New Phase Pre-05A fields
    #[serde(default)]
    pub active_camera_view: CameraViewMode,
    #[serde(default)]
    pub cockpit_hud_style_typed: ClusterTheme,
    #[serde(default)]
    pub bonnet_hud_style_typed: ClusterTheme,
    #[serde(default)]
    pub chase_hud_style_typed: ClusterTheme,
    #[serde(default)]
    pub driver_seat_height_m: f32,
    #[serde(default)]
    pub steering_wheel_offset_y_m: f32,
    #[serde(default)]
    pub fov_degrees: i32,
    #[serde(default)]
    pub cluster_scale: f32,
    #[serde(default)]
    pub font_preset: FontPreset,
    #[serde(default)]
    pub ambient_glow_enabled: bool,
    #[serde(default)]
    pub ambient_glow_intensity: f32,
    #[serde(default)]
    pub ambient_cowl_feather: f32,

    #[serde(default = "default_osd_toggle_mask")]
    pub osd_toggle_mask: u16,
    #[serde(default = "default_osd_hold_ms")]
    pub osd_hold_ms: u32,
    #[serde(default = "default_guide_button_osd")]
    pub guide_button_opens_widget: bool,

    #[serde(flatten)]
    #[serde(default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
        version: String::new(),
        cockpit_anchor_x: 0,
        cockpit_anchor_y: 0,
        cockpit_scale: 0.0,
        bonnet_anchor_x: 0,
        bonnet_anchor_y: 0,
        bonnet_scale: 0.0,
        current_camera_index: 0,
        cockpit_style: String::new(),
        bonnet_style: String::new(),
        hud_style: String::new(),
        cockpit_hud_style: String::new(),
        bonnet_hud_style: String::new(),
        oem_dash_opaque_pct: 0,
        oem_dash_binnacle_offset_y: 0,
        oem_dash_width_scale: 0.0,
        osd_auto_pause_game: false,
        osd_live_dyno_preview_when_paused: false,
        drift_sync_mode: String::new(),
        drift_sync_sensitivity: 0.0,
        drift_sync_smoothing: 0.0,
        drift_sync_max_offset_px: 0.0,
        lock_visible_while_driving: false,
        shift_light_pattern: String::new(),
        shift_start_pct: 0.0,
        redline_pct: 0.0,
        controller_camera_button: String::new(),
        speed_unit: String::new(),
        show_bonnet_gt_widgets: false,
        demo_mode: false,
        per_car_profile_enabled: false,
        oled_pixel_orbit_enabled: false,
        oled_idle_dimmer_enabled: false,
        oled_dimmer_min_alpha: 0.0,
        manual_shift_guard_enabled: false,
        custom_driver_name: String::new(),
        driver_tag_glitch_fx: false,
        power_unit: String::new(),
        redline_strobe: false,
        night_glow_mode: String::new(),
        night_glow_intensity: 0.0,
        night_ambient_glow_enabled: false,
        night_ambient_glow_intensity_pct: 0,
        adas_backdrop_mode: 0,
        center_console_map_mode: 0,
        center_console_infotainment_mode: 0,
        route_active: false,
        oem_dash_dimmer_enabled: false,
        modular_side_pods_enabled: false,
        steering_hub_cap_enabled: false,
        udp_port: 0,
        fallback_port: 0,
        forward_port: 0,
        color_mode: String::new(),
        custom_brand_key: String::new(),
        primary_hex: String::new(),
        secondary_hex: String::new(),
        accent_hex: String::new(),
        shift_flash_hex: String::new(),
        use_widget_color_overrides: false,
        speed_color_hex: String::new(),
        gear_color_hex: String::new(),
        g_force_color_hex: String::new(),
        suspension_color_hex: String::new(),
        wheel_less_cockpit_fov: 0.0,
        standard_cockpit_fov: 0.0,
        bonnet_fov: 0.0,
        chase_near_fov: 0.0,
        chase_far_fov: 0.0,
        bumper_fov: 0.0,
        cockpit_offset_x: 0.0,
        cockpit_offset_y: 0.0,
        cockpit_offset_z: 0.0,
        hood_offset_y: 0.0,
        hood_offset_z: 0.0,
        hood_pitch_offset: 0.0,
        active_camera_view: Default::default(),
        cockpit_hud_style_typed: Default::default(),
        bonnet_hud_style_typed: ClusterTheme::Gt7HoodPro,
        chase_hud_style_typed: ClusterTheme::BoschDdu,
        driver_seat_height_m: -0.045f32,
        steering_wheel_offset_y_m: -0.035f32,
        fov_degrees: 65,
        cluster_scale: 1.00f32,
        font_preset: Default::default(),
        ambient_glow_enabled: true,
        ambient_glow_intensity: 0.25f32,
        ambient_cowl_feather: 0.40f32,
        osd_toggle_mask: default_osd_toggle_mask(),
        osd_hold_ms: default_osd_hold_ms(),
        guide_button_opens_widget: default_guide_button_osd(),
        extra: serde_json::Map::new(),
        }
    }
}

impl AppConfig {
    pub fn effective_cluster_and_scrim(&self) -> (ClusterTheme, f32) {
        let view = self.active_camera_view;
        match view {
            CameraViewMode::Cockpit => (self.cockpit_hud_style_typed, self.oem_dash_opaque_pct as f32 / 100.0),
            CameraViewMode::Bonnet => (self.bonnet_hud_style_typed, 0.0),
            CameraViewMode::Chase => (self.chase_hud_style_typed, 0.0),
        }
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if path.exists() {
            if let Ok(s) = fs::read_to_string(path) {
                if let Ok(mut cfg) = serde_json::from_str::<AppConfig>(&s) {
                    cfg.apply_migrations();
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp = path.with_extension("tmp");
        fs::write(&tmp, json)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        fs::rename(tmp, path)
    }

    fn apply_migrations(&mut self) {
        if (5200..=5300).contains(&self.fallback_port) {
            self.fallback_port = 5500;
        }
        if self.oem_dash_width_scale <= 0.01 {
            self.oem_dash_width_scale = 1.0;
        }
        self.night_ambient_glow_intensity_pct = self.night_ambient_glow_intensity_pct.clamp(0, 100);
        if !(0..=2).contains(&self.adas_backdrop_mode) {
            self.adas_backdrop_mode = 0;
        }
        if !(0..=3).contains(&self.center_console_map_mode) {
            self.center_console_map_mode = 0;
        }
        if !(0..=4).contains(&self.center_console_infotainment_mode) {
            self.center_console_infotainment_mode = 0;
        }
        if self.cockpit_hud_style.trim().is_empty() {
            self.cockpit_hud_style = if !self.hud_style.trim().is_empty() {
                self.hud_style.clone()
            } else {
                "BoschDdu".to_string()
            };
        }
        if self.bonnet_hud_style.trim().is_empty() {
            self.bonnet_hud_style = "Gt7HoodPro".to_string();
        }
        if self.custom_driver_name.trim().is_empty() {
            self.custom_driver_name = "ØG『死』ŦJ乂".to_string();
        }
        // Migrate legacy cockpit_offset_y to driver_seat_height_m if new field is at default
        if (self.driver_seat_height_m - (-0.045)).abs() < f32::EPSILON {
            if self.cockpit_offset_y.abs() > f32::EPSILON {
                // cockpit_offset_y is in meters already (legacy)
                let migrated = self.cockpit_offset_y.clamp(-0.100, 0.050);
                self.driver_seat_height_m = migrated;
            }
        }
    }
}

fn default_driver_name() -> String {
    "ØG『死』ŦJ乂".to_string()
}

pub fn default_osd_toggle_mask() -> u16 {
    0x0020
}

pub fn default_osd_hold_ms() -> u32 {
    400
}

pub fn default_guide_button_osd() -> bool {
    true
}

pub fn sanitize_toggle_mask(mask: u16) -> u16 {
    const ABXY_MASK: u16 = 0xF000;
    let sanitized = mask & !ABXY_MASK;
    if sanitized == 0 {
        0x0020
    } else {
        sanitized
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CameraViewMode {
    #[default]
    Cockpit,   // camera_index == 3
    Bonnet,    // camera_index == 4
    Chase,     // any other index
}
impl CameraViewMode {
    pub fn from_index(index: i32) -> Self {
        match index {
            3 => CameraViewMode::Cockpit,
            4 => CameraViewMode::Bonnet,
            _ => CameraViewMode::Chase,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FontPreset {
    #[default]
    BahnschriftLight,
    SfPro,
    Tabular,
    VfdMono,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_select_hold_0x0020_and_tap_ignore() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.osd_toggle_mask, 0x0020);
        assert_eq!(cfg.osd_hold_ms, 400);
        // sanitize should keep 0x0020
        assert_eq!(sanitize_toggle_mask(0x0020), 0x0020);
    }

    #[test]
    fn test_remapper_rejects_abxy_bits() {
        // ABXY bits should be stripped
        assert_eq!(sanitize_toggle_mask(0x1000), 0x0020); // A only -> fallback
        assert_eq!(sanitize_toggle_mask(0x2000), 0x0020); // B only
        assert_eq!(sanitize_toggle_mask(0x4000), 0x0020); // X only
        assert_eq!(sanitize_toggle_mask(0x8000), 0x0020); // Y only
        assert_eq!(sanitize_toggle_mask(0xF000), 0x0020); // all ABXY -> fallback
        // Mixed safe + ABXY
        assert_eq!(sanitize_toggle_mask(0x0020 | 0x1000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x0030 | 0x8000), 0x0030);
        // Safe mask unchanged
        assert_eq!(sanitize_toggle_mask(0x0040), 0x0040);
    }
}
