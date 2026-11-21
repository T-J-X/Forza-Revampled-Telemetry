//! `AppConfig`: the serde model of `config.json`, its defaults, its location and its sanitising.
//!
//! The file uses PascalCase keys (`CockpitAnchorX`) plus a few all-caps acronyms
//! (`StandardCockpitFOV`, `ShowBonnetGTWidgets`) that need explicit renames. Any key missing
//! from the file takes its value from [`AppConfig::default`], never zero, because the whole
//! struct is `#[serde(default)]`. Unknown keys are kept in [`AppConfig::extra`] and written back.
use crate::theme::ClusterTheme;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The file name the overlay looks for next to its executable.
pub const CONFIG_FILE_NAME: &str = "config.json";

/// Default primary telemetry port (Forza "Data Out" target).
pub const DEFAULT_UDP_PORT: u16 = 8000;
/// Default fallback telemetry port.
pub const DEFAULT_FALLBACK_PORT: u16 = 5500;
/// Default port that received packets are forwarded to, for other telemetry tools.
pub const DEFAULT_FORWARD_PORT: u16 = 5607;

/// The A/B/X/Y bits: the manual gearbox, handbrake and rewind. Never bindable.
pub const MASK_MANUAL_LOCK: u16 = 0xF000;
/// Shortest allowed OSD hold time (ms).
pub const OSD_HOLD_MS_MIN: u32 = 200;
/// Longest allowed OSD hold time (ms).
pub const OSD_HOLD_MS_MAX: u32 = 1000;

/// Why a config file could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read.
    #[error("reading {path}")]
    Read {
        /// The file that failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The file is not valid config JSON.
    #[error("parsing {path}")]
    Parse {
        /// The file that failed.
        path: PathBuf,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
}

/// Every user setting of the overlay, as stored in `config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AppConfig {
    pub version: String,

    // Window & layout anchors (4K reference coordinates, 3840x2160).
    pub cockpit_anchor_x: i32,
    pub cockpit_anchor_y: i32,
    pub cockpit_scale: f32,
    pub bonnet_anchor_x: i32,
    pub bonnet_anchor_y: i32,
    pub bonnet_scale: f32,

    // Camera & styles
    pub current_camera_index: i32,
    pub cockpit_style: String,
    pub bonnet_style: String,
    pub hud_style: String,
    pub cockpit_hud_style: String,
    pub bonnet_hud_style: String,
    pub oem_dash_opaque_pct: i32,
    pub oem_dash_binnacle_offset_y: i32,
    pub oem_dash_width_scale: f32,
    pub osd_auto_pause_game: bool,
    pub osd_live_dyno_preview_when_paused: bool,
    pub drift_sync_mode: String,
    pub drift_sync_sensitivity: f32,
    pub drift_sync_smoothing: f32,
    pub drift_sync_max_offset_px: f32,
    pub lock_visible_while_driving: bool,

    // CARTEK shift pod
    pub shift_light_pattern: String,
    pub shift_start_pct: f32,
    pub redline_pct: f32,

    // Controller
    pub controller_camera_button: String,

    // Units & features
    pub speed_unit: String,
    #[serde(rename = "ShowBonnetGTWidgets", alias = "ShowBonnetGtWidgets")]
    pub show_bonnet_gt_widgets: bool,
    pub demo_mode: bool,

    // Per-car / OLED / shift guard
    pub per_car_profile_enabled: bool,
    pub oled_pixel_orbit_enabled: bool,
    pub oled_idle_dimmer_enabled: bool,
    pub oled_dimmer_min_alpha: f32,
    pub manual_shift_guard_enabled: bool,
    pub custom_driver_name: String,
    pub driver_tag_glitch_fx: bool,
    pub power_unit: String,
    pub redline_strobe: bool,

    // Night glow
    pub night_glow_mode: String,
    pub night_glow_intensity: f32,
    pub night_ambient_glow_enabled: bool,
    pub night_ambient_glow_intensity_pct: i32,

    // Modular cockpit
    pub adas_backdrop_mode: i32,
    pub center_console_map_mode: i32,
    pub center_console_infotainment_mode: i32,
    pub route_active: bool,
    pub oem_dash_dimmer_enabled: bool,
    pub modular_side_pods_enabled: bool,
    pub steering_hub_cap_enabled: bool,

    // Telemetry ports
    pub udp_port: i32,
    pub fallback_port: i32,
    pub forward_port: i32,
    /// Listen for telemetry on all network interfaces (Forza running on another PC). Off by
    /// default: loopback needs no firewall rule.
    pub udp_listen_all_interfaces: bool,

    // Brand & theme
    pub color_mode: String,
    pub custom_brand_key: String,
    pub primary_hex: String,
    pub secondary_hex: String,
    pub accent_hex: String,
    pub shift_flash_hex: String,

    // Widget overrides
    pub use_widget_color_overrides: bool,
    pub speed_color_hex: String,
    pub gear_color_hex: String,
    pub g_force_color_hex: String,
    pub suspension_color_hex: String,

    // FOV & offsets
    #[serde(rename = "WheelLessCockpitFOV", alias = "WheelLessCockpitFov")]
    pub wheel_less_cockpit_fov: f32,
    #[serde(rename = "StandardCockpitFOV", alias = "StandardCockpitFov")]
    pub standard_cockpit_fov: f32,
    #[serde(rename = "BonnetFOV", alias = "BonnetFov")]
    pub bonnet_fov: f32,
    #[serde(rename = "ChaseNearFOV", alias = "ChaseNearFov")]
    pub chase_near_fov: f32,
    #[serde(rename = "ChaseFarFOV", alias = "ChaseFarFov")]
    pub chase_far_fov: f32,
    #[serde(rename = "BumperFOV", alias = "BumperFov")]
    pub bumper_fov: f32,
    pub cockpit_offset_x: f32,
    pub cockpit_offset_y: f32,
    pub cockpit_offset_z: f32,
    pub hood_offset_y: f32,
    pub hood_offset_z: f32,
    pub hood_pitch_offset: f32,

    // Pre-05A fields
    pub active_camera_view: CameraViewMode,
    pub cockpit_hud_style_typed: ClusterTheme,
    pub bonnet_hud_style_typed: ClusterTheme,
    pub chase_hud_style_typed: ClusterTheme,
    pub driver_seat_height_m: f32,
    pub steering_wheel_offset_y_m: f32,
    pub fov_degrees: i32,
    pub cluster_scale: f32,
    pub font_preset: FontPreset,
    pub ambient_glow_enabled: bool,
    pub ambient_glow_intensity: f32,
    pub ambient_cowl_feather: f32,

    pub osd_toggle_mask: u16,
    pub osd_hold_ms: u32,
    pub guide_button_opens_widget: bool,

    /// Keys this version does not know, preserved on save.
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: "6.0.0".to_string(),
            cockpit_anchor_x: 1920,
            cockpit_anchor_y: 1835,
            cockpit_scale: 1.45,
            bonnet_anchor_x: 1920,
            bonnet_anchor_y: 1760,
            bonnet_scale: 1.55,
            current_camera_index: 3,
            cockpit_style: "Digital".to_string(),
            bonnet_style: "Digital".to_string(),
            hud_style: "LiquidGlass".to_string(),
            cockpit_hud_style: "BoschDdu".to_string(),
            bonnet_hud_style: "Gt7HoodPro".to_string(),
            oem_dash_opaque_pct: 100,
            oem_dash_binnacle_offset_y: 0,
            oem_dash_width_scale: 1.0,
            osd_auto_pause_game: true,
            osd_live_dyno_preview_when_paused: true,
            drift_sync_mode: "Off".to_string(),
            drift_sync_sensitivity: 1.0,
            drift_sync_smoothing: 0.2,
            drift_sync_max_offset_px: 400.0,
            lock_visible_while_driving: false,
            shift_light_pattern: "Converging".to_string(),
            shift_start_pct: 0.72,
            redline_pct: 0.94,
            controller_camera_button: "RB".to_string(),
            speed_unit: "MPH".to_string(),
            show_bonnet_gt_widgets: true,
            demo_mode: false,
            per_car_profile_enabled: true,
            oled_pixel_orbit_enabled: true,
            oled_idle_dimmer_enabled: true,
            oled_dimmer_min_alpha: 0.35,
            manual_shift_guard_enabled: true,
            custom_driver_name: default_driver_name(),
            driver_tag_glitch_fx: false,
            power_unit: "kW".to_string(),
            redline_strobe: false,
            night_glow_mode: "Off".to_string(),
            night_glow_intensity: 0.35,
            night_ambient_glow_enabled: false,
            night_ambient_glow_intensity_pct: 0,
            adas_backdrop_mode: 0,
            center_console_map_mode: 0,
            center_console_infotainment_mode: 0,
            route_active: false,
            oem_dash_dimmer_enabled: false,
            modular_side_pods_enabled: false,
            steering_hub_cap_enabled: false,
            udp_port: i32::from(DEFAULT_UDP_PORT),
            fallback_port: i32::from(DEFAULT_FALLBACK_PORT),
            forward_port: i32::from(DEFAULT_FORWARD_PORT),
            udp_listen_all_interfaces: false,
            color_mode: "AutoBrand".to_string(),
            custom_brand_key: "DEFAULT".to_string(),
            primary_hex: "#00D2FF".to_string(),
            secondary_hex: "#FFB300".to_string(),
            accent_hex: "#00E5FF".to_string(),
            shift_flash_hex: "#FFFFFF".to_string(),
            use_widget_color_overrides: false,
            speed_color_hex: "#FFFFFF".to_string(),
            gear_color_hex: "#FFFFFF".to_string(),
            g_force_color_hex: "#00E5FF".to_string(),
            suspension_color_hex: "#76FF03".to_string(),
            wheel_less_cockpit_fov: 80.0,
            standard_cockpit_fov: 80.0,
            bonnet_fov: 95.0,
            chase_near_fov: 62.0,
            chase_far_fov: 68.0,
            bumper_fov: 78.0,
            cockpit_offset_x: 0.0,
            cockpit_offset_y: -0.045,
            cockpit_offset_z: -0.5,
            hood_offset_y: 0.0,
            hood_offset_z: 0.0,
            hood_pitch_offset: 0.04,
            active_camera_view: CameraViewMode::default(),
            cockpit_hud_style_typed: ClusterTheme::default(),
            bonnet_hud_style_typed: ClusterTheme::Gt7HoodPro,
            chase_hud_style_typed: ClusterTheme::BoschDdu,
            driver_seat_height_m: -0.045,
            steering_wheel_offset_y_m: -0.035,
            fov_degrees: 65,
            cluster_scale: 1.0,
            font_preset: FontPreset::default(),
            ambient_glow_enabled: true,
            ambient_glow_intensity: 0.25,
            ambient_cowl_feather: 0.40,
            osd_toggle_mask: default_osd_toggle_mask(),
            osd_hold_ms: default_osd_hold_ms(),
            guide_button_opens_widget: default_guide_button_osd(),
            extra: serde_json::Map::new(),
        }
    }
}

/// Telemetry ports resolved from config: always valid, never zero for the primary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TelemetryPorts {
    /// The port Forza sends to.
    pub primary: u16,
    /// A second listening port, if different from the primary.
    pub fallback: Option<u16>,
    /// Where each received packet is re-sent on 127.0.0.1, if enabled and not a listening port.
    pub forward: Option<u16>,
    /// Listen on every interface (Forza on another PC) instead of loopback only.
    pub all_interfaces: bool,
}

impl AppConfig {
    /// The cluster theme and scrim opacity (0..=1) for the active camera view.
    pub fn effective_cluster_and_scrim(&self) -> (ClusterTheme, f32) {
        match self.active_camera_view {
            CameraViewMode::Cockpit => (self.cockpit_hud_style_typed, self.oem_dash_opaque_pct as f32 / 100.0),
            CameraViewMode::Bonnet => (self.bonnet_hud_style_typed, 0.0),
            CameraViewMode::Chase => (self.chase_hud_style_typed, 0.0),
        }
    }

    /// The listening and forwarding ports, with invalid values replaced by the defaults.
    pub fn telemetry_ports(&self) -> TelemetryPorts {
        let primary = valid_port(self.udp_port).unwrap_or(DEFAULT_UDP_PORT);
        let fallback = valid_port(self.fallback_port).filter(|&p| p != primary);
        let forward = valid_port(self.forward_port).filter(|&p| p != primary && Some(p) != fallback);
        TelemetryPorts {
            primary,
            fallback,
            forward,
            all_interfaces: self.udp_listen_all_interfaces,
        }
    }

    /// Reads and parses a config file, then applies migrations and sanitising.
    ///
    /// # Errors
    /// [`ConfigError::Read`] if the file cannot be read, [`ConfigError::Parse`] if it is not
    /// valid config JSON. A UTF-8 BOM at the start is tolerated.
    pub fn try_load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_str(&text).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Parses config JSON (a leading BOM is ignored), then migrates and sanitises it.
    ///
    /// # Errors
    /// Returns the JSON error if the text is not a valid config object.
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut cfg: AppConfig = serde_json::from_str(text)?;
        cfg.apply_migrations();
        Ok(cfg)
    }

    /// Loads a config file, falling back to [`AppConfig::default`] if it is missing or invalid.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match Self::try_load(path) {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::warn!(error = %err, "config not loaded, using defaults");
                Self::default()
            }
        }
    }

    /// Writes the config atomically (temp file, then rename), as UTF-8 without a BOM.
    ///
    /// # Errors
    /// Returns the I/O error if the temp file cannot be written or renamed.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let json = serde_json::to_string_pretty(self).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, json)?;
        // `rename` replaces the destination atomically on NTFS (MoveFileEx with REPLACE_EXISTING).
        fs::rename(tmp, path)
    }

    fn apply_migrations(&mut self) {
        let defaults = AppConfig::default();
        if (5200..=5300).contains(&self.fallback_port) {
            self.fallback_port = i32::from(DEFAULT_FALLBACK_PORT);
        }
        if valid_port(self.udp_port).is_none() {
            self.udp_port = defaults.udp_port;
        }
        if self.oem_dash_width_scale <= 0.01 {
            self.oem_dash_width_scale = 1.0;
        }
        if !(self.cockpit_scale.is_finite() && self.cockpit_scale > 0.05) {
            self.cockpit_scale = defaults.cockpit_scale;
        }
        if !(self.bonnet_scale.is_finite() && self.bonnet_scale > 0.05) {
            self.bonnet_scale = defaults.bonnet_scale;
        }
        if !(self.cluster_scale.is_finite() && self.cluster_scale > 0.05) {
            self.cluster_scale = defaults.cluster_scale;
        }
        self.oem_dash_opaque_pct = self.oem_dash_opaque_pct.clamp(0, 100);
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
        if !(0..6).contains(&self.current_camera_index) {
            self.current_camera_index = defaults.current_camera_index;
        }
        if self.cockpit_hud_style.trim().is_empty() {
            self.cockpit_hud_style = if self.hud_style.trim().is_empty() {
                "BoschDdu".to_string()
            } else {
                self.hud_style.clone()
            };
        }
        if self.bonnet_hud_style.trim().is_empty() {
            self.bonnet_hud_style = "Gt7HoodPro".to_string();
        }
        if self.custom_driver_name.trim().is_empty() {
            self.custom_driver_name = default_driver_name();
        }
        self.osd_toggle_mask = sanitize_toggle_mask(self.osd_toggle_mask);
        self.osd_hold_ms = self.osd_hold_ms.clamp(OSD_HOLD_MS_MIN, OSD_HOLD_MS_MAX);
        // Migrate the legacy cockpit_offset_y (metres) into driver_seat_height_m while the
        // latter is still at its default.
        if (self.driver_seat_height_m - defaults.driver_seat_height_m).abs() < f32::EPSILON
            && self.cockpit_offset_y.abs() > f32::EPSILON
        {
            self.driver_seat_height_m = self.cockpit_offset_y.clamp(-0.100, 0.050);
        }
    }
}

fn valid_port(raw: i32) -> Option<u16> {
    u16::try_from(raw).ok().filter(|&p| p != 0)
}

/// Finds `config.json`: next to the executable first, then `data/config.json` in the exe's
/// folder or any parent (a `cargo run` from the workspace), then the same search from the
/// working directory. Returns the exe-side path (which may not exist yet) if nothing is found.
pub fn resolve_config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    let cwd = std::env::current_dir().ok();
    resolve_config_path_from(exe_dir.as_deref(), cwd.as_deref())
}

/// [`resolve_config_path`] with explicit start folders, for tests.
pub fn resolve_config_path_from(exe_dir: Option<&Path>, cwd: Option<&Path>) -> PathBuf {
    if let Some(dir) = exe_dir {
        let beside = dir.join(CONFIG_FILE_NAME);
        if beside.is_file() {
            return beside;
        }
    }
    for start in [exe_dir, cwd].into_iter().flatten() {
        for dir in start.ancestors() {
            let candidate = dir.join("data").join(CONFIG_FILE_NAME);
            if candidate.is_file() {
                return candidate;
            }
        }
    }
    exe_dir
        .or(cwd)
        .map(|d| d.join(CONFIG_FILE_NAME))
        .unwrap_or_else(|| PathBuf::from(CONFIG_FILE_NAME))
}

fn default_driver_name() -> String {
    "ØG『死』ŦJ乂".to_string()
}

/// Default OSD toggle button: View/Back.
pub fn default_osd_toggle_mask() -> u16 {
    0x0020
}

/// Default OSD hold time in milliseconds.
pub fn default_osd_hold_ms() -> u32 {
    400
}

/// Whether the Guide button opens the widget by default.
pub fn default_guide_button_osd() -> bool {
    true
}

/// Strips A/B/X/Y from a toggle mask; an empty result falls back to View (`0x0020`).
pub fn sanitize_toggle_mask(mask: u16) -> u16 {
    let sanitized = mask & !MASK_MANUAL_LOCK;
    if sanitized == 0 {
        default_osd_toggle_mask()
    } else {
        sanitized
    }
}

/// Which family of camera the game is in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CameraViewMode {
    /// Camera index 3: wheel-less cockpit.
    #[default]
    Cockpit,
    /// Camera index 4: hood.
    Bonnet,
    /// Any other index.
    Chase,
}

impl CameraViewMode {
    /// Maps the game's camera ring index (0..6) to a view family.
    pub fn from_index(index: i32) -> Self {
        match index {
            3 => CameraViewMode::Cockpit,
            4 => CameraViewMode::Bonnet,
            _ => CameraViewMode::Chase,
        }
    }
}

/// Typography presets offered by the OSD.
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
        assert_eq!(sanitize_toggle_mask(0x0020), 0x0020);
    }

    #[test]
    fn test_remapper_rejects_abxy_bits() {
        assert_eq!(sanitize_toggle_mask(0x1000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x2000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x4000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x8000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0xF000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x0020 | 0x1000), 0x0020);
        assert_eq!(sanitize_toggle_mask(0x0030 | 0x8000), 0x0030);
        assert_eq!(sanitize_toggle_mask(0x0040), 0x0040);
    }

    #[test]
    fn empty_object_takes_defaults_not_zero() {
        let cfg = AppConfig::from_json_str("{}").expect("empty object is valid");
        assert_eq!(cfg, {
            let mut d = AppConfig::default();
            d.apply_migrations();
            d
        });
        assert_eq!(cfg.cockpit_anchor_x, 1920);
        assert!(cfg.cluster_scale > 0.0);
        assert!(cfg.standard_cockpit_fov > 0.0);
    }

    #[test]
    fn acronym_keys_round_trip() {
        let cfg = AppConfig::from_json_str(r#"{"StandardCockpitFOV":71,"ShowBonnetGTWidgets":false}"#)
            .expect("valid json");
        assert_eq!(cfg.standard_cockpit_fov, 71.0);
        assert!(!cfg.show_bonnet_gt_widgets);
        let json = serde_json::to_string(&cfg).expect("serialises");
        assert!(json.contains("\"StandardCockpitFOV\":71"), "{json}");
        assert!(json.contains("\"ShowBonnetGTWidgets\":false"), "{json}");
        assert!(!json.contains("StandardCockpitFov"), "{json}");
    }

    #[test]
    fn unknown_keys_are_preserved() {
        let cfg = AppConfig::from_json_str(r#"{"PrimaryPort":8000,"FutureKey":"x"}"#).expect("valid json");
        assert_eq!(cfg.extra.get("FutureKey"), Some(&serde_json::Value::from("x")));
        let json = serde_json::to_string(&cfg).expect("serialises");
        assert!(json.contains("\"FutureKey\":\"x\""));
    }

    #[test]
    fn bom_is_tolerated() {
        let cfg = AppConfig::from_json_str("\u{feff}{\"CockpitAnchorY\":1800}").expect("valid json");
        assert_eq!(cfg.cockpit_anchor_y, 1800);
    }

    #[test]
    fn zero_and_invalid_values_are_sanitised() {
        let cfg = AppConfig::from_json_str(
            r#"{"UdpPort":0,"CockpitScale":0,"ClusterScale":-1,"OsdToggleMask":4096,"OsdHoldMs":5,"CurrentCameraIndex":9}"#,
        )
        .expect("valid json");
        assert_eq!(cfg.udp_port, 8000);
        assert_eq!(cfg.cockpit_scale, 1.45);
        assert_eq!(cfg.cluster_scale, 1.0);
        assert_eq!(cfg.osd_toggle_mask, 0x0020);
        assert_eq!(cfg.osd_hold_ms, OSD_HOLD_MS_MIN);
        assert_eq!(cfg.current_camera_index, 3);
    }

    #[test]
    fn telemetry_ports_skip_duplicates_and_zero() {
        let cfg = AppConfig {
            udp_port: 8000,
            fallback_port: 8000,
            forward_port: 0,
            ..AppConfig::default()
        };
        assert_eq!(
            cfg.telemetry_ports(),
            TelemetryPorts {
                primary: 8000,
                fallback: None,
                forward: None,
                all_interfaces: false
            }
        );
        let ports = AppConfig::default().telemetry_ports();
        assert_eq!(ports.primary, 8000);
        assert_eq!(ports.fallback, Some(5500));
        assert_eq!(ports.forward, Some(5607));
    }

    #[test]
    fn resolve_prefers_file_beside_exe() {
        let root = std::env::temp_dir().join(format!("fh6_cfg_resolve_{}", std::process::id()));
        let exe_dir = root.join("target").join("debug");
        fs::create_dir_all(&exe_dir).expect("create temp dirs");
        fs::create_dir_all(root.join("data")).expect("create data dir");
        fs::write(root.join("data").join(CONFIG_FILE_NAME), "{}").expect("write dev config");

        assert_eq!(
            resolve_config_path_from(Some(&exe_dir), None),
            root.join("data").join(CONFIG_FILE_NAME),
            "falls back to data/config.json in a parent"
        );
        fs::write(exe_dir.join(CONFIG_FILE_NAME), "{}").expect("write exe config");
        assert_eq!(
            resolve_config_path_from(Some(&exe_dir), None),
            exe_dir.join(CONFIG_FILE_NAME),
            "prefers config.json beside the exe"
        );
        fs::remove_dir_all(&root).ok();
    }
}
