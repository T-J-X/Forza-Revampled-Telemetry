//! `AppConfig`: the typed model of `config.json`, with its defaults, migrations, location and
//! range checks.
//!
//! The file uses PascalCase keys (`CockpitAnchorX`) plus a few all-caps acronyms
//! (`StandardCockpitFOV`) that need explicit renames. [`AppConfig::from_json_str`] loads it in
//! three steps:
//!
//! 1. **Migrate** the raw JSON object: one-time fixes for files written before
//!    [`CONFIG_VERSION`], the old `ClusterHeightOffsetPx`, one spelling per key (the canonical one
//!    wins over an alias such as `BonnetFov`), and the removal of the [`RETIRED_KEYS`] that nothing
//!    reads any more. A file whose version this build cannot read is not migrated.
//! 2. **Type it leniently:** a value that does not fit its setting, such as text where a number
//!    belongs or a style no cluster has, is dropped with a warning and takes its default. The rest
//!    of the file still loads.
//! 3. **Sanitise:** a number that is not finite takes its default, and a setting with a range is
//!    clamped to it ([`AppConfig::sanitized`]).
//!
//! A key missing from the file takes its value from [`AppConfig::default`], never zero. Keys this
//! version does not know (a dash's own settings, a newer build's) are kept in
//! [`AppConfig::extra`] and written back.
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::fit::FitView;
use crate::oled_care::DEFAULT_MIN_ALPHA;
use crate::serde_str::serde_by_name;
use crate::theme::ClusterTheme;

/// The file name the overlay looks for next to its executable.
pub const CONFIG_FILE_NAME: &str = "config.json";

/// The config format this build writes. A file with an older major version gets the one-time
/// migrations once; the next save records this version, so they never run again.
pub const CONFIG_VERSION: &str = "7.0.0";
/// The major part of [`CONFIG_VERSION`].
const CONFIG_MAJOR: u64 = 7;

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
/// Largest cluster height offset either way (px at 4K).
pub const HEIGHT_OFFSET_MAX_PX: f32 = 240.0;
/// Shortest allowed LB+RB resync hold (ms).
pub const RESYNC_HOLD_MS_MIN: u32 = 500;
/// Longest allowed LB+RB resync hold (ms).
pub const RESYNC_HOLD_MS_MAX: u32 = 3000;
/// Lowest OSD shift point (fraction of the limiter rpm).
pub const SHIFT_POINT_MIN: f32 = 0.80;
/// Highest OSD shift point (fraction of the limiter rpm).
pub const SHIFT_POINT_MAX: f32 = 1.0;
/// The game's field of view limits in degrees (`MinFOV` / `MaxFOV` in `CameraSettings.ini`), for
/// every camera view. The camera patcher writes within them.
pub const FOV_RANGE: (f32, f32) = (40.0, 110.0);

/// Default seat height (metres, DriverCam `Param.y`).
const DEFAULT_SEAT_HEIGHT_M: f32 = -0.045;
/// The range the old `CockpitOffsetY` seat value is clamped to when it is migrated.
const SEAT_HEIGHT_RANGE_M: (f32, f32) = (-0.100, 0.050);

/// The driver name every config defaulted to before 7.0.0. The font pack has no CJK face, so it
/// cannot be drawn; a file from before 7.0.0 that still holds it gets an empty name.
const OLD_DEFAULT_DRIVER_NAME: &str = "ØG『死』ŦJ乂";

/// Settings that also load from a second spelling: (canonical key, alias). When a file has both,
/// the canonical one wins.
const KEY_ALIASES: [(&str, &str); 6] = [
    ("WheelLessCockpitFOV", "WheelLessCockpitFov"),
    ("StandardCockpitFOV", "StandardCockpitFov"),
    ("BonnetFOV", "BonnetFov"),
    ("ChaseNearFOV", "ChaseNearFov"),
    ("ChaseFarFOV", "ChaseFarFov"),
    ("BumperFOV", "BumperFov"),
];

/// Keys earlier versions wrote that nothing reads any more. They are dropped on load, so the next
/// save leaves them out. `docs/PROGRESS.md` (fix D) records why each one went.
pub const RETIRED_KEYS: &[&str] = &[
    // v1 renderer scales and styles, and the typed copies of the style names.
    "CockpitScale",
    "BonnetScale",
    "CockpitStyle",
    "BonnetStyle",
    "HudStyle",
    "CockpitHudStyleTyped",
    "BonnetHudStyleTyped",
    "ChaseHudStyleTyped",
    "ActiveCameraView",
    "ShowBonnetGTWidgets",
    "ShowBonnetGtWidgets",
    "FontPreset",
    "FovDegrees",
    "AmbientCowlFeather",
    // C# port settings no v2 code reads.
    "OsdLiveDynoPreviewWhenPaused",
    "DriftSyncMode",
    "DriftSyncSensitivity",
    "DriftSyncSmoothing",
    "DriftSyncMaxOffsetPx",
    "LockVisibleWhileDriving",
    "ShiftLightPattern",
    "ShiftStartPct",
    "RedlinePct",
    "ControllerCameraButton",
    "ManualShiftGuardEnabled",
    "DriverTagGlitchFx",
    "RedlineStrobe",
    "NightGlowMode",
    "NightGlowIntensity",
    "AdasBackdropMode",
    "CenterConsoleMapMode",
    "CenterConsoleInfotainmentMode",
    "RouteActive",
    "OemDashDimmerEnabled",
    "ModularSidePodsEnabled",
    "SteeringHubCapEnabled",
    "UseWidgetColorOverrides",
    "SpeedColorHex",
    "GearColorHex",
    "GForceColorHex",
    "SuspensionColorHex",
    "PrimaryPort",
    // Migrated into `DriverSeatHeightM` by the pre-7.0.0 migration.
    "CockpitOffsetY",
];

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
    /// The file is not a JSON object.
    #[error("parsing {path}")]
    Parse {
        /// The file that failed.
        path: PathBuf,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
}

/// The error the setting enums' [`FromStr`] returns for a name that is none of their options. It
/// holds no copy of the input, so a failed parse never allocates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ParseSettingError {
    setting: &'static str,
}

impl ParseSettingError {
    /// The setting the name was for (`"speed unit"`).
    #[must_use]
    pub fn setting(&self) -> &'static str {
        self.setting
    }
}

impl fmt::Display for ParseSettingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown {}", self.setting)
    }
}

impl std::error::Error for ParseSettingError {}

/// Speed display units (`SpeedUnit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpeedUnit {
    /// Miles per hour (`"MPH"`).
    #[default]
    Mph,
    /// Kilometres per hour (`"KMH"`).
    Kmh,
}

impl SpeedUnit {
    /// The config name (`"MPH"`, `"KMH"`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            SpeedUnit::Mph => "MPH",
            SpeedUnit::Kmh => "KMH",
        }
    }

    /// The label a dash prints (`"MPH"`, `"KM/H"`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            SpeedUnit::Mph => "MPH",
            SpeedUnit::Kmh => "KM/H",
        }
    }
}

impl FromStr for SpeedUnit {
    type Err = ParseSettingError;

    /// `mph`, or `kmh`, `kph` or `km/h`, in any case and with surrounding spaces ignored.
    ///
    /// # Errors
    ///
    /// [`ParseSettingError`] for any other name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("mph") {
            Ok(SpeedUnit::Mph)
        } else if ["kmh", "kph", "km/h"].iter().any(|n| s.eq_ignore_ascii_case(n)) {
            Ok(SpeedUnit::Kmh)
        } else {
            Err(ParseSettingError { setting: "speed unit" })
        }
    }
}

impl fmt::Display for SpeedUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

serde_by_name!(SpeedUnit, "a speed unit (MPH or KMH)");

/// Engine power display units (`PowerUnit`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PowerUnit {
    /// Mechanical horsepower (`"HP"`).
    #[default]
    Hp,
    /// Kilowatts (`"kW"`).
    Kw,
}

impl PowerUnit {
    /// The config name (`"HP"`, `"kW"`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            PowerUnit::Hp => "HP",
            PowerUnit::Kw => "kW",
        }
    }

    /// The label a dash prints (`"HP"`, `"kW"`).
    #[must_use]
    pub fn label(self) -> &'static str {
        self.name()
    }
}

impl FromStr for PowerUnit {
    type Err = ParseSettingError;

    /// `hp` or `kw`, in any case and with surrounding spaces ignored.
    ///
    /// # Errors
    ///
    /// [`ParseSettingError`] for any other name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.eq_ignore_ascii_case("hp") {
            Ok(PowerUnit::Hp)
        } else if s.eq_ignore_ascii_case("kw") {
            Ok(PowerUnit::Kw)
        } else {
            Err(ParseSettingError { setting: "power unit" })
        }
    }
}

impl fmt::Display for PowerUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

serde_by_name!(PowerUnit, "a power unit (HP or kW)");

/// Where a dash's colours come from (`ColorMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColorMode {
    /// The brand palette of the car being driven, else the dash's own (`"AutoBrand"`, which the
    /// OSD calls Factory).
    #[default]
    AutoBrand,
    /// The palette of the brand in `CustomBrandKey` (`"Brand"`).
    Brand,
    /// The hex colours `PrimaryHex`, `SecondaryHex`, `AccentHex` and `ShiftFlashHex`
    /// (`"Custom"`).
    Custom,
    /// A seeded random palette (`"Random"`, seeded by `PaletteSeed`).
    Random,
}

impl ColorMode {
    /// The config name (`"AutoBrand"`, `"Brand"`, `"Custom"`, `"Random"`).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            ColorMode::AutoBrand => "AutoBrand",
            ColorMode::Brand => "Brand",
            ColorMode::Custom => "Custom",
            ColorMode::Random => "Random",
        }
    }
}

impl FromStr for ColorMode {
    type Err = ParseSettingError;

    /// A mode's config name, or `factory` for [`ColorMode::AutoBrand`], in any case and with
    /// surrounding spaces ignored.
    ///
    /// # Errors
    ///
    /// [`ParseSettingError`] for any other name.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        [
            ColorMode::AutoBrand,
            ColorMode::Brand,
            ColorMode::Custom,
            ColorMode::Random,
        ]
        .into_iter()
        .find(|m| m.name().eq_ignore_ascii_case(s))
        .or_else(|| s.eq_ignore_ascii_case("factory").then_some(ColorMode::AutoBrand))
        .ok_or(ParseSettingError { setting: "colour mode" })
    }
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

serde_by_name!(ColorMode, "a colour mode (AutoBrand, Brand, Custom or Random)");

/// Every user setting of the overlay, as stored in `config.json`.
///
/// Load it with [`AppConfig::try_load`], [`AppConfig::load`] or [`AppConfig::from_json_str`].
/// Deserializing it directly skips the migrations, the lenient parse and the range checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct AppConfig {
    /// The config format version ([`CONFIG_VERSION`] once this build has loaded the file).
    pub version: String,

    // Cluster placement per view (4K reference px; see `docs/design/SCRIM_FIT.md`).
    /// Cockpit `[3]` cluster anchor, x.
    pub cockpit_anchor_x: i32,
    /// Cockpit `[3]` cluster anchor, y.
    pub cockpit_anchor_y: i32,
    /// Hood `[4]` cluster anchor, x.
    pub bonnet_anchor_x: i32,
    /// Hood `[4]` cluster anchor, y.
    pub bonnet_anchor_y: i32,
    /// Chase-camera floating HUD anchor, x.
    pub chase_anchor_x: i32,
    /// Chase-camera floating HUD anchor, y.
    pub chase_anchor_y: i32,
    /// Cockpit cluster height in px (positive raises; ±240).
    pub cockpit_height_offset_px: f32,
    /// Hood cluster height in px (positive raises; ±240).
    pub bonnet_height_offset_px: f32,
    /// Chase HUD height in px (positive raises; ±240).
    pub chase_height_offset_px: f32,
    /// Overall cluster scale, multiplied by the per-view scale; the product is clamped to
    /// 0.70–1.40.
    pub cluster_scale: f32,
    /// Cockpit scale (0.5–2.0).
    pub cockpit_cluster_scale: f32,
    /// Hood scale (0.5–2.0).
    pub bonnet_cluster_scale: f32,
    /// Chase HUD scale (0.5–2.0).
    pub chase_cluster_scale: f32,
    /// Margins the cockpit cluster's envelope is clamped into.
    pub cockpit_safe_zone: SafeZone,
    /// Margins the hood cluster's envelope is clamped into.
    pub bonnet_safe_zone: SafeZone,
    /// Margins the chase HUD's envelope is clamped into.
    pub chase_safe_zone: SafeZone,
    /// Show a floating HUD in the chase cameras.
    pub chase_hud_enabled: bool,

    // Camera ring and clusters.
    /// The camera ring index the game starts in (0..6; 3 cockpit, 4 hood).
    pub current_camera_index: i32,
    /// The cockpit `[3]` cluster.
    pub cockpit_hud_style: ClusterTheme,
    /// The hood `[4]` cluster.
    pub bonnet_hud_style: ClusterTheme,
    /// The chase HUD's cluster; absent follows the cockpit ([`AppConfig::theme`]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chase_hud_style: Option<ClusterTheme>,
    /// The camera index a resync (OSD item or LB+RB hold) declares the game to be in.
    pub resync_camera_index: i32,
    /// How long LB+RB must be held to resync (ms).
    pub resync_hold_ms: u32,

    // The dome scrim that hides the OEM dash (4K px at user scale 1.0).
    /// Core opacity of the scrim in the cockpit (0..=100; the hood and chase force 0).
    pub oem_dash_opaque_pct: i32,
    /// Vertical dome offset (px, ±600).
    pub oem_dash_binnacle_offset_y: i32,
    /// Dome width multiplier (0.5–1.6).
    pub oem_dash_width_scale: f32,
    /// Horizontal dome offset from the steering-column line (px, + right, ±600).
    pub oem_dash_offset_x: i32,
    /// Dome height multiplier (semi-axis 470 px at 1.0; 0.5–1.6).
    pub oem_dash_height_scale: f32,
    /// Top arch: 0 = flat crown (exponent 4), 1 = round arch (exponent 2). Default 0.7 (2.6).
    pub oem_dash_top_arch: f32,
    /// Lower corners: 0 = square (exponent 4), 1 = round (exponent 2). Default 0.7 (2.6).
    pub oem_dash_corner_softness: f32,
    /// Feather at the top edge in px (sides x1.25, bottom corners x1.46).
    pub oem_dash_feather_px: f32,
    /// Width of the steering-column void cut below the cluster in px (0 = no void).
    pub oem_dash_column_void_px: f32,

    // The ambient wash on the binnacle lip.
    /// Draw the ambient wash.
    pub ambient_glow_enabled: bool,
    /// Wash strength (0..=1).
    pub ambient_glow_intensity: f32,
    /// Raise the wash at night.
    pub night_ambient_glow_enabled: bool,
    /// Night wash strength (0..=100).
    pub night_ambient_glow_intensity_pct: i32,

    // Readouts, colours and the dash's own options.
    /// Speed display units.
    pub speed_unit: SpeedUnit,
    /// Engine power display units.
    pub power_unit: PowerUnit,
    /// Where the dash's colours come from.
    pub color_mode: ColorMode,
    /// The brand whose palette [`ColorMode::Brand`] uses (`car_brands.json` key).
    pub custom_brand_key: String,
    /// [`ColorMode::Custom`] primary colour (`#RRGGBB`).
    pub primary_hex: String,
    /// [`ColorMode::Custom`] secondary colour.
    pub secondary_hex: String,
    /// [`ColorMode::Custom`] accent colour.
    pub accent_hex: String,
    /// [`ColorMode::Custom`] shift-light colour.
    pub shift_flash_hex: String,
    /// The [`ColorMode::Random`] seed; absent seeds by the car ordinal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub palette_seed: Option<u64>,
    /// The driver name a dash may show; empty shows none.
    pub custom_driver_name: String,
    /// Show the brand logo zone.
    pub show_logo: bool,
    /// Show the frame-time overlay.
    pub show_stats: bool,
    /// The font family the dashes use; absent keeps each dash's own.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_font: Option<String>,
    /// The OSD's shift point (fraction of the limiter, 0.80–1.00); absent uses the tokens'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shift_point_pct: Option<f32>,
    /// Tame the shift-light flash (no full-field strobe).
    pub reduced_flash: bool,
    /// Bend the captured game frame through glass pieces (screen capture; off by default).
    pub glass_refraction: bool,

    // OSD and controller.
    /// Pause the game while the OSD is open.
    pub osd_auto_pause_game: bool,
    /// The buttons that open the OSD when held (XInput bits; A/B/X/Y are never allowed).
    pub osd_toggle_mask: u16,
    /// How long the toggle must be held (ms).
    pub osd_hold_ms: u32,
    /// The Guide button opens the OSD too.
    pub guide_button_opens_widget: bool,

    // OLED care.
    /// Move the dash by a pixel or two over time.
    pub oled_pixel_orbit_enabled: bool,
    /// Dim the dash while the car sits still.
    pub oled_idle_dimmer_enabled: bool,
    /// The idle dimmer's lowest opacity (0..=1).
    pub oled_dimmer_min_alpha: f32,

    // Per-car profiles and demo.
    /// Overlay `car_profiles.json` on these settings for the car being driven.
    pub per_car_profile_enabled: bool,
    /// Drive the dash from recorded telemetry instead of the game.
    pub demo_mode: bool,

    // Telemetry ports.
    /// The port Forza's Data Out sends to.
    pub udp_port: i32,
    /// A second listening port (0 = none).
    pub fallback_port: i32,
    /// Where each packet is re-sent on 127.0.0.1 for other tools (0 = none).
    pub forward_port: i32,
    /// Listen for telemetry on all network interfaces (Forza running on another PC). Off by
    /// default: loopback needs no firewall rule.
    pub udp_listen_all_interfaces: bool,

    // The game's camera files: FOV per camera and the cockpit and hood offsets.
    /// Wheel-less cockpit `[3]` FOV (degrees).
    #[serde(rename = "WheelLessCockpitFOV", alias = "WheelLessCockpitFov")]
    pub wheel_less_cockpit_fov: f32,
    /// Standard cockpit FOV (degrees).
    #[serde(rename = "StandardCockpitFOV", alias = "StandardCockpitFov")]
    pub standard_cockpit_fov: f32,
    /// Hood `[4]` FOV (degrees).
    #[serde(rename = "BonnetFOV", alias = "BonnetFov")]
    pub bonnet_fov: f32,
    /// Near chase FOV (degrees).
    #[serde(rename = "ChaseNearFOV", alias = "ChaseNearFov")]
    pub chase_near_fov: f32,
    /// Far chase FOV (degrees).
    #[serde(rename = "ChaseFarFOV", alias = "ChaseFarFov")]
    pub chase_far_fov: f32,
    /// Bumper FOV (degrees).
    #[serde(rename = "BumperFOV", alias = "BumperFov")]
    pub bumper_fov: f32,
    /// Cockpit camera offset, x (metres).
    pub cockpit_offset_x: f32,
    /// Cockpit camera offset, z (metres).
    pub cockpit_offset_z: f32,
    /// Hood camera offset, y (metres).
    pub hood_offset_y: f32,
    /// Hood camera offset, z (metres).
    pub hood_offset_z: f32,
    /// Hood camera pitch offset.
    pub hood_pitch_offset: f32,
    /// Seat height (metres, DriverCam `Param.y`).
    pub driver_seat_height_m: f32,
    /// Steering wheel height offset (metres).
    pub steering_wheel_offset_y_m: f32,
    /// Write the camera files automatically when seat or FOV settings change (queued while the
    /// game runs). Off by default: the OSD and configurator apply explicitly.
    pub camera_patch_auto: bool,
    /// Game install folder override; empty = find it through Steam's `libraryfolders.vdf`.
    pub game_install_dir: String,

    /// Keys this version does not know, preserved on save. A dash keeps its own settings here
    /// (`PorscheAnalogScale`).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: CONFIG_VERSION.to_string(),
            cockpit_anchor_x: 1920,
            cockpit_anchor_y: 1835,
            bonnet_anchor_x: 1920,
            bonnet_anchor_y: 1760,
            chase_anchor_x: 1920,
            chase_anchor_y: 1900,
            cockpit_height_offset_px: 0.0,
            bonnet_height_offset_px: 0.0,
            chase_height_offset_px: 0.0,
            cluster_scale: 1.0,
            cockpit_cluster_scale: 1.0,
            bonnet_cluster_scale: 1.0,
            chase_cluster_scale: 0.8,
            cockpit_safe_zone: SafeZone::default(),
            bonnet_safe_zone: SafeZone::default(),
            chase_safe_zone: SafeZone::default(),
            chase_hud_enabled: true,
            current_camera_index: 3,
            cockpit_hud_style: ClusterTheme::BoschDdu,
            bonnet_hud_style: ClusterTheme::Gt7HoodPro,
            chase_hud_style: None,
            resync_camera_index: 3,
            resync_hold_ms: 1000,
            oem_dash_opaque_pct: 100,
            oem_dash_binnacle_offset_y: 0,
            oem_dash_width_scale: 1.0,
            oem_dash_offset_x: 0,
            oem_dash_height_scale: 1.0,
            oem_dash_top_arch: 0.7,
            oem_dash_corner_softness: 0.7,
            oem_dash_feather_px: 96.0,
            oem_dash_column_void_px: 220.0,
            ambient_glow_enabled: true,
            ambient_glow_intensity: 0.25,
            night_ambient_glow_enabled: false,
            night_ambient_glow_intensity_pct: 0,
            speed_unit: SpeedUnit::Mph,
            power_unit: PowerUnit::Hp,
            color_mode: ColorMode::AutoBrand,
            custom_brand_key: "DEFAULT".to_string(),
            primary_hex: "#00D2FF".to_string(),
            secondary_hex: "#FFB300".to_string(),
            accent_hex: "#00E5FF".to_string(),
            shift_flash_hex: "#FFFFFF".to_string(),
            palette_seed: None,
            custom_driver_name: String::new(),
            show_logo: true,
            show_stats: false,
            cluster_font: None,
            shift_point_pct: None,
            reduced_flash: false,
            glass_refraction: false,
            osd_auto_pause_game: true,
            osd_toggle_mask: default_osd_toggle_mask(),
            osd_hold_ms: default_osd_hold_ms(),
            guide_button_opens_widget: default_guide_button_osd(),
            oled_pixel_orbit_enabled: true,
            oled_idle_dimmer_enabled: true,
            oled_dimmer_min_alpha: DEFAULT_MIN_ALPHA,
            per_car_profile_enabled: true,
            demo_mode: false,
            udp_port: i32::from(DEFAULT_UDP_PORT),
            fallback_port: i32::from(DEFAULT_FALLBACK_PORT),
            forward_port: i32::from(DEFAULT_FORWARD_PORT),
            udp_listen_all_interfaces: false,
            wheel_less_cockpit_fov: 80.0,
            standard_cockpit_fov: 80.0,
            bonnet_fov: 95.0,
            chase_near_fov: 62.0,
            chase_far_fov: 68.0,
            bumper_fov: 78.0,
            cockpit_offset_x: 0.0,
            cockpit_offset_z: -0.5,
            hood_offset_y: 0.0,
            hood_offset_z: 0.0,
            hood_pitch_offset: 0.04,
            driver_seat_height_m: DEFAULT_SEAT_HEIGHT_M,
            steering_wheel_offset_y_m: -0.035,
            camera_patch_auto: false,
            game_install_dir: String::new(),
            extra: Map::new(),
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
    /// The cluster shown in `view`. The chase HUD follows the cockpit unless it has its own.
    #[must_use]
    pub fn theme(&self, view: FitView) -> ClusterTheme {
        match view {
            FitView::Cockpit => self.cockpit_hud_style,
            FitView::Hood => self.bonnet_hud_style,
            FitView::Chase => self.chase_hud_style.unwrap_or(self.cockpit_hud_style),
        }
    }

    /// Shows `theme` in `view`. Setting the chase HUD gives it its own cluster.
    pub fn set_theme(&mut self, view: FitView, theme: ClusterTheme) {
        match view {
            FitView::Cockpit => self.cockpit_hud_style = theme,
            FitView::Hood => self.bonnet_hud_style = theme,
            FitView::Chase => self.chase_hud_style = Some(theme),
        }
    }

    /// The listening and forwarding ports, with invalid values replaced by the defaults.
    #[must_use]
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

    /// Reads and parses a config file, then migrates and sanitises it.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Read`] if the file cannot be read, [`ConfigError::Parse`] if it is not a
    /// JSON object. A UTF-8 BOM at the start is tolerated.
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

    /// Parses config JSON (a leading BOM is ignored), then migrates, types and sanitises it as the
    /// module documentation describes. A value that does not fit its setting is logged and takes
    /// its default.
    ///
    /// # Errors
    ///
    /// The JSON error if the text is not a JSON object.
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut raw: Map<String, Value> = serde_json::from_str(text)?;
        migrate_raw(&mut raw);
        let mut cfg = deserialize_lenient(raw);
        cfg.sanitize();
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
    ///
    /// The I/O error if the temp file cannot be written or renamed.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        // A/B/X/Y are stripped from every saved mask, not only on load (manual gearbox lock).
        let mut clean = self.clone();
        clean.osd_toggle_mask = sanitize_toggle_mask(clean.osd_toggle_mask);
        let json = serde_json::to_string_pretty(&clean).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        crate::atomic_file::write_atomically(path.as_ref(), json.as_bytes())
    }

    /// `cfg` with every range check applied (for configs built in code, such as a per-car
    /// overlay). Applying it twice changes nothing more.
    #[must_use]
    pub fn sanitized(mut cfg: AppConfig) -> AppConfig {
        cfg.sanitize();
        cfg
    }

    /// Step 3 of a load: a number that is not finite takes its default, and a setting with a range
    /// is clamped to it. Idempotent.
    fn sanitize(&mut self) {
        let d = AppConfig::default();
        let fix = |v: &mut f32, default: f32, lo: f32, hi: f32| {
            *v = if v.is_finite() { v.clamp(lo, hi) } else { default };
        };

        // Kept from the C# port, which reset fallback ports in 5200..=5300; the reason was not
        // recorded.
        if (5200..=5300).contains(&self.fallback_port) {
            self.fallback_port = d.fallback_port;
        }
        if valid_port(self.udp_port).is_none() {
            self.udp_port = d.udp_port;
        }
        if !(0..6).contains(&self.current_camera_index) {
            self.current_camera_index = d.current_camera_index;
        }
        if !(0..6).contains(&self.resync_camera_index) {
            self.resync_camera_index = d.resync_camera_index;
        }
        self.resync_hold_ms = self.resync_hold_ms.clamp(RESYNC_HOLD_MS_MIN, RESYNC_HOLD_MS_MAX);
        self.osd_toggle_mask = sanitize_toggle_mask(self.osd_toggle_mask);
        self.osd_hold_ms = self.osd_hold_ms.clamp(OSD_HOLD_MS_MIN, OSD_HOLD_MS_MAX);

        // Placement.
        if !(self.cluster_scale.is_finite() && self.cluster_scale > 0.05) {
            self.cluster_scale = d.cluster_scale;
        }
        for (h, default) in [
            (&mut self.cockpit_height_offset_px, d.cockpit_height_offset_px),
            (&mut self.bonnet_height_offset_px, d.bonnet_height_offset_px),
            (&mut self.chase_height_offset_px, d.chase_height_offset_px),
        ] {
            fix(h, default, -HEIGHT_OFFSET_MAX_PX, HEIGHT_OFFSET_MAX_PX);
        }
        for (s, default) in [
            (&mut self.cockpit_cluster_scale, d.cockpit_cluster_scale),
            (&mut self.bonnet_cluster_scale, d.bonnet_cluster_scale),
            (&mut self.chase_cluster_scale, d.chase_cluster_scale),
        ] {
            fix(s, default, 0.5, 2.0);
        }
        for z in [
            &mut self.cockpit_safe_zone,
            &mut self.bonnet_safe_zone,
            &mut self.chase_safe_zone,
        ] {
            z.sanitize();
        }

        // Scrim. A width of (about) zero means "unset", not "as narrow as allowed".
        if self.oem_dash_width_scale <= 0.01 {
            self.oem_dash_width_scale = d.oem_dash_width_scale;
        }
        self.oem_dash_opaque_pct = self.oem_dash_opaque_pct.clamp(0, 100);
        self.oem_dash_offset_x = self.oem_dash_offset_x.clamp(-600, 600);
        self.oem_dash_binnacle_offset_y = self.oem_dash_binnacle_offset_y.clamp(-600, 600);
        fix(&mut self.oem_dash_width_scale, d.oem_dash_width_scale, 0.5, 1.6);
        fix(&mut self.oem_dash_height_scale, d.oem_dash_height_scale, 0.5, 1.6);
        fix(&mut self.oem_dash_top_arch, d.oem_dash_top_arch, 0.0, 1.0);
        fix(&mut self.oem_dash_corner_softness, d.oem_dash_corner_softness, 0.0, 1.0);
        fix(&mut self.oem_dash_feather_px, d.oem_dash_feather_px, 8.0, 240.0);
        fix(&mut self.oem_dash_column_void_px, d.oem_dash_column_void_px, 0.0, 600.0);

        // Camera. The patcher keeps each offset inside the game's own limits; here a value that is
        // not a number (or too large for an f32, which would save as null) takes its default.
        for (fov, default) in [
            (&mut self.wheel_less_cockpit_fov, d.wheel_less_cockpit_fov),
            (&mut self.standard_cockpit_fov, d.standard_cockpit_fov),
            (&mut self.bonnet_fov, d.bonnet_fov),
            (&mut self.chase_near_fov, d.chase_near_fov),
            (&mut self.chase_far_fov, d.chase_far_fov),
            (&mut self.bumper_fov, d.bumper_fov),
        ] {
            fix(fov, default, FOV_RANGE.0, FOV_RANGE.1);
        }
        for (v, default) in [
            (&mut self.cockpit_offset_x, d.cockpit_offset_x),
            (&mut self.cockpit_offset_z, d.cockpit_offset_z),
            (&mut self.hood_offset_y, d.hood_offset_y),
            (&mut self.hood_offset_z, d.hood_offset_z),
            (&mut self.hood_pitch_offset, d.hood_pitch_offset),
            (&mut self.driver_seat_height_m, d.driver_seat_height_m),
            (&mut self.steering_wheel_offset_y_m, d.steering_wheel_offset_y_m),
        ] {
            if !v.is_finite() {
                *v = default;
            }
        }

        // Wash, OLED care and the dash options.
        fix(&mut self.ambient_glow_intensity, d.ambient_glow_intensity, 0.0, 1.0);
        self.night_ambient_glow_intensity_pct = self.night_ambient_glow_intensity_pct.clamp(0, 100);
        fix(&mut self.oled_dimmer_min_alpha, d.oled_dimmer_min_alpha, 0.0, 1.0);
        self.shift_point_pct = self
            .shift_point_pct
            .filter(|v| v.is_finite())
            .map(|v| v.clamp(SHIFT_POINT_MIN, SHIFT_POINT_MAX));
        if self.cluster_font.as_deref().is_some_and(|f| f.trim().is_empty()) {
            self.cluster_font = None;
        }
    }
}

/// Step 1 of a load: fixes on the raw object, before any value is typed.
fn migrate_raw(raw: &mut Map<String, Value>) {
    // A number is a version too (`"Version": 8`); keep it, as text, through the typed parse.
    if let Some(Value::Number(n)) = raw.get("Version") {
        let text = n.to_string();
        raw.insert("Version".to_owned(), Value::from(text));
    }
    let major = file_major(raw);
    match major {
        Some(major) if major < CONFIG_MAJOR => {
            migrate_before_7(raw);
            raw.insert("Version".to_owned(), Value::from(CONFIG_VERSION));
        }
        Some(_) => {}
        None => tracing::warn!(
            version = ?raw.get("Version"),
            "config version not understood; the file is loaded without migrations"
        ),
    }
    migrate_blank_styles(raw);
    migrate_cluster_height(raw);
    migrate_key_spellings(raw);
    // A newer build may use one of these names again, so only files up to this version lose them.
    if major.is_some_and(|major| major <= CONFIG_MAJOR) {
        raw.retain(|key, _| !RETIRED_KEYS.contains(&key.as_str()));
    }
}

/// The file's major version: 0 when it has none (files from before versioning), `None` when it
/// has one this build cannot read.
fn file_major(raw: &Map<String, Value>) -> Option<u64> {
    match raw.get("Version") {
        None | Some(Value::Null) => Some(0),
        Some(Value::String(version)) => version_major(version),
        Some(_) => None,
    }
}

/// The major part of a version such as `"7.0.0"` or `"7-beta"`: its leading digits.
fn version_major(version: &str) -> Option<u64> {
    let version = version.trim();
    let end = version.find(|c: char| !c.is_ascii_digit()).unwrap_or(version.len());
    version[..end].parse().ok()
}

/// One-time fixes for a file written before 7.0.0.
fn migrate_before_7(raw: &mut Map<String, Value>) {
    // `PowerUnit` changed nothing before 7.0.0: the dashes showed hp with MPH and kW with km/h.
    // Keep what the user saw, unless the file holds a choice other than the old default "kW".
    let power_was_default = raw
        .get("PowerUnit")
        .and_then(Value::as_str)
        .is_none_or(|p| p.parse() == Ok(PowerUnit::Kw));
    if power_was_default {
        let speed = raw
            .get("SpeedUnit")
            .and_then(Value::as_str)
            .and_then(|s| s.parse().ok());
        let power = if speed == Some(SpeedUnit::Kmh) {
            PowerUnit::Kw
        } else {
            PowerUnit::Hp
        };
        raw.insert("PowerUnit".to_owned(), Value::from(power.name()));
    }

    // The C# port kept the seat height in `CockpitOffsetY`. The Rust build before 7.0.0 copied it
    // into `DriverSeatHeightM` on every load while the seat was at its default, and wrote only
    // `DriverSeatHeightM`, so a file that has `DriverSeatHeightM` already holds the user's seat
    // (setting it back to the default used to bring the old value back). Only a file without it
    // takes the old value, once; `CockpitOffsetY` is retired.
    if let (false, Some(old)) = (
        raw.contains_key("DriverSeatHeightM"),
        raw.get("CockpitOffsetY").and_then(Value::as_f64),
    ) {
        let old = old as f32;
        if old.is_finite() && old.abs() > f32::EPSILON {
            let (lo, hi) = SEAT_HEIGHT_RANGE_M;
            raw.insert(
                "DriverSeatHeightM".to_owned(),
                Value::from(f64::from(old.clamp(lo, hi))),
            );
        }
    }

    if raw.get("CustomDriverName").and_then(Value::as_str) == Some(OLD_DEFAULT_DRIVER_NAME) {
        raw.insert("CustomDriverName".to_owned(), Value::from(""));
    }
}

/// A blank style name used to mean "the default": `HudStyle` for the cockpit, and the cockpit's
/// cluster for the chase HUD. Styles are typed now and have no blank value, so a blank name is
/// replaced or removed before the styles are parsed.
fn migrate_blank_styles(raw: &mut Map<String, Value>) {
    fn non_blank(v: &Value) -> bool {
        v.as_str().is_some_and(|s| !s.trim().is_empty())
    }
    let blank = |raw: &Map<String, Value>, key: &str| raw.get(key).is_some_and(|v| v.is_string() && !non_blank(v));
    if blank(raw, "CockpitHudStyle") {
        match raw.get("HudStyle").filter(|v| non_blank(v)).cloned() {
            Some(style) => {
                raw.insert("CockpitHudStyle".to_owned(), style);
            }
            None => {
                raw.shift_remove("CockpitHudStyle");
            }
        }
    }
    for key in ["BonnetHudStyle", "ChaseHudStyle"] {
        if blank(raw, key) {
            raw.shift_remove(key);
        }
    }
}

/// Phase 8 replaced the one `ClusterHeightOffsetPx` with a height per view. A per-view height
/// that is absent or zero takes the old value.
fn migrate_cluster_height(raw: &mut Map<String, Value>) {
    let Some(old) = raw.shift_remove("ClusterHeightOffsetPx").and_then(|v| v.as_f64()) else {
        return;
    };
    for key in ["CockpitHeightOffsetPx", "BonnetHeightOffsetPx", "ChaseHeightOffsetPx"] {
        let unset = raw
            .get(key)
            .and_then(Value::as_f64)
            .is_none_or(|h| (h as f32).abs() < f32::EPSILON);
        if unset {
            raw.insert(key.to_owned(), Value::from(old));
        }
    }
}

/// One spelling per setting, so that the typed parse never sees two: the canonical key wins, and a
/// lone alias is renamed to it.
fn migrate_key_spellings(raw: &mut Map<String, Value>) {
    for (canonical, alias) in KEY_ALIASES {
        let Some(value) = raw.shift_remove(alias) else {
            continue;
        };
        if raw.contains_key(canonical) {
            tracing::warn!(%alias, %canonical, value = ?value, "config has two spellings of one key; the second is dropped");
        } else {
            raw.insert(canonical.to_owned(), value);
        }
    }
}

/// Step 2 of a load: types every value. A value that does not fit its setting is dropped with a
/// warning, so that it takes its default and the rest of the file still loads.
fn deserialize_lenient(raw: Map<String, Value>) -> AppConfig {
    let err = match AppConfig::deserialize(&raw) {
        Ok(cfg) => return cfg,
        Err(err) => err,
    };
    tracing::warn!(error = %err, "config has values that do not fit their settings");
    // Only for a damaged file: add the keys back one at a time and drop each one that fails.
    let mut cfg = AppConfig::default();
    let mut kept = Map::new();
    for (key, value) in raw {
        kept.insert(key.clone(), value);
        match AppConfig::deserialize(&kept) {
            Ok(good) => cfg = good,
            Err(err) => {
                let dropped = kept.shift_remove(&key);
                tracing::warn!(%key, value = ?dropped, error = %err, "config value dropped, using the default");
            }
        }
    }
    cfg
}

/// Margins (px at 4K) a cluster's envelope must stay inside.
///
/// It loads margin by margin: a margin that is not a number takes its default with a warning and
/// the others keep their values.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct SafeZone {
    /// Left and right margin.
    pub side_px: f32,
    /// Top margin.
    pub top_px: f32,
    /// Bottom margin.
    pub bottom_px: f32,
}

impl Default for SafeZone {
    /// The tokens' safe zone: 160 px at the sides, 40 px at the bottom.
    fn default() -> Self {
        Self {
            side_px: 160.0,
            top_px: 0.0,
            bottom_px: 40.0,
        }
    }
}

impl<'de> Deserialize<'de> for SafeZone {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = Map::<String, Value>::deserialize(deserializer)?;
        let mut zone = Self::default();
        for (key, value) in &raw {
            let margin = match key.as_str() {
                "SidePx" => &mut zone.side_px,
                "TopPx" => &mut zone.top_px,
                "BottomPx" => &mut zone.bottom_px,
                _ => continue,
            };
            match value.as_f64() {
                Some(v) => *margin = v as f32,
                None => tracing::warn!(%key, value = ?value, "safe zone margin is not a number; using the default"),
            }
        }
        Ok(zone)
    }
}

impl SafeZone {
    fn sanitize(&mut self) {
        let d = Self::default();
        for (v, dv) in [
            (&mut self.side_px, d.side_px),
            (&mut self.top_px, d.top_px),
            (&mut self.bottom_px, d.bottom_px),
        ] {
            *v = if v.is_finite() { v.clamp(0.0, 800.0) } else { dv };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load(json: &str) -> AppConfig {
        AppConfig::from_json_str(json).expect("valid json")
    }

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
    fn the_version_constants_agree() {
        assert_eq!(version_major(CONFIG_VERSION), Some(CONFIG_MAJOR));
        assert_eq!(AppConfig::default().version, CONFIG_VERSION);
        assert_eq!(version_major(" 12.1 "), Some(12));
        assert_eq!(version_major("7-beta"), Some(7));
        assert_eq!(version_major("abc"), None);
        assert_eq!(version_major("18446744073709551616.0.0"), None);
    }

    #[test]
    fn defaults_are_already_in_range() {
        let d = AppConfig::default();
        assert_eq!(AppConfig::sanitized(d.clone()), d);
    }

    #[test]
    fn empty_object_takes_defaults_not_zero() {
        let cfg = load("{}");
        assert_eq!(cfg, AppConfig::default());
        assert_eq!(cfg.cockpit_anchor_x, 1920);
        assert!(cfg.cluster_scale > 0.0);
        assert!(cfg.standard_cockpit_fov > 0.0);
    }

    #[test]
    fn acronym_keys_round_trip() {
        let cfg = load(r#"{"StandardCockpitFOV":71,"BonnetFov":90}"#);
        assert_eq!(cfg.standard_cockpit_fov, 71.0);
        assert_eq!(cfg.bonnet_fov, 90.0, "the old spelling still loads");
        let json = serde_json::to_string(&cfg).expect("serialises");
        assert!(json.contains("\"StandardCockpitFOV\":71"), "{json}");
        assert!(json.contains("\"BonnetFOV\":90"), "{json}");
        assert!(!json.contains("StandardCockpitFov"), "{json}");
    }

    #[test]
    fn unknown_keys_are_preserved() {
        let cfg = load(r#"{"PorscheAnalogScale":"Classic","FutureKey":"x"}"#);
        assert_eq!(cfg.extra.get("FutureKey"), Some(&Value::from("x")));
        let json = serde_json::to_string(&cfg).expect("serialises");
        assert!(json.contains("\"FutureKey\":\"x\""));
        assert!(json.contains("\"PorscheAnalogScale\":\"Classic\""));
    }

    #[test]
    fn bom_is_tolerated() {
        assert_eq!(load("\u{feff}{\"CockpitAnchorY\":1800}").cockpit_anchor_y, 1800);
    }

    #[test]
    fn a_file_that_is_not_an_object_is_an_error() {
        assert!(AppConfig::from_json_str("[1,2]").is_err());
        assert!(AppConfig::from_json_str("{\"CockpitAnchorY\":").is_err());
    }

    #[test]
    fn zero_and_invalid_values_are_sanitised() {
        let cfg = load(
            r#"{"UdpPort":0,"ClusterScale":-1,"OsdToggleMask":4096,"OsdHoldMs":5,"CurrentCameraIndex":9,
                "OledDimmerMinAlpha":3.5,"ShiftPointPct":0.1,"ClusterFont":"  ","OemDashWidthScale":0}"#,
        );
        assert_eq!(cfg.udp_port, 8000);
        assert_eq!(cfg.cluster_scale, 1.0);
        assert_eq!(cfg.osd_toggle_mask, 0x0020);
        assert_eq!(cfg.osd_hold_ms, OSD_HOLD_MS_MIN);
        assert_eq!(cfg.current_camera_index, 3);
        assert_eq!(cfg.oled_dimmer_min_alpha, 1.0);
        assert_eq!(cfg.shift_point_pct, Some(SHIFT_POINT_MIN));
        assert_eq!(cfg.cluster_font, None);
        assert_eq!(cfg.oem_dash_width_scale, 1.0);
    }

    #[test]
    fn the_dimmer_floor_has_one_default_and_one_range() {
        assert_eq!(AppConfig::default().oled_dimmer_min_alpha, DEFAULT_MIN_ALPHA);
        assert_eq!(load(r#"{"OledDimmerMinAlpha":-0.2}"#).oled_dimmer_min_alpha, 0.0);
        let nan = AppConfig {
            oled_dimmer_min_alpha: f32::NAN,
            ..AppConfig::default()
        };
        assert_eq!(AppConfig::sanitized(nan).oled_dimmer_min_alpha, DEFAULT_MIN_ALPHA);
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
    fn styles_load_by_name_or_alias_and_the_chase_follows_the_cockpit() {
        let cfg = load(r#"{"CockpitHudStyle":"liquidglass","BonnetHudStyle":"hood"}"#);
        assert_eq!(cfg.theme(FitView::Cockpit), ClusterTheme::LiquidGlass);
        assert_eq!(cfg.theme(FitView::Hood), ClusterTheme::Gt7HoodPro);
        assert_eq!(cfg.chase_hud_style, None);
        assert_eq!(cfg.theme(FitView::Chase), ClusterTheme::LiquidGlass);
        let json = serde_json::to_string(&cfg).expect("serialises");
        assert!(json.contains("\"CockpitHudStyle\":\"LiquidGlass\""), "{json}");
        assert!(!json.contains("ChaseHudStyle"), "absent stays absent: {json}");

        let mut cfg = cfg;
        cfg.set_theme(FitView::Chase, ClusterTheme::RetroCyber);
        assert_eq!(cfg.theme(FitView::Chase), ClusterTheme::RetroCyber);
        assert_eq!(cfg.theme(FitView::Cockpit), ClusterTheme::LiquidGlass);
    }

    #[test]
    fn blank_styles_mean_the_default_and_a_blank_cockpit_takes_the_old_hud_style() {
        let cfg = load(r#"{"CockpitHudStyle":"","HudStyle":"PorscheAnalog","BonnetHudStyle":" ","ChaseHudStyle":""}"#);
        assert_eq!(cfg.cockpit_hud_style, ClusterTheme::PorscheAnalog);
        assert_eq!(cfg.bonnet_hud_style, ClusterTheme::Gt7HoodPro);
        assert_eq!(cfg.chase_hud_style, None);
        assert!(!cfg.extra.contains_key("HudStyle"), "retired");

        let no_fallback = load(r#"{"CockpitHudStyle":"","HudStyle":""}"#);
        assert_eq!(no_fallback.cockpit_hud_style, ClusterTheme::BoschDdu);
        let present_wins = load(r#"{"CockpitHudStyle":"RetroCyber","HudStyle":"PorscheAnalog"}"#);
        assert_eq!(present_wins.cockpit_hud_style, ClusterTheme::RetroCyber);
    }

    #[test]
    fn a_value_that_does_not_fit_takes_its_default_and_the_rest_still_loads() {
        let cfg = load(
            r#"{"CockpitHudStyle":"NoSuchDash","CockpitAnchorY":"high","SpeedUnit":"furlongs",
                "CockpitAnchorX":1700,"BonnetHudStyle":"LiquidGlass","UdpPort":8100}"#,
        );
        assert_eq!(cfg.cockpit_hud_style, ClusterTheme::BoschDdu);
        assert_eq!(cfg.cockpit_anchor_y, 1835);
        assert_eq!(cfg.speed_unit, SpeedUnit::Mph);
        assert_eq!(cfg.cockpit_anchor_x, 1700);
        assert_eq!(cfg.bonnet_hud_style, ClusterTheme::LiquidGlass);
        assert_eq!(cfg.udp_port, 8100);
        assert!(
            !cfg.extra.contains_key("CockpitHudStyle"),
            "a dropped value is not kept as unknown"
        );
    }

    #[test]
    fn the_canonical_spelling_wins_over_an_alias_in_any_order() {
        for text in [
            r#"{"BonnetFOV":91,"BonnetFov":99,"CockpitAnchorX":1700}"#,
            r#"{"BonnetFov":99,"BonnetFOV":91,"CockpitAnchorX":1700}"#,
            // Removing a retired key must not move the alias in front.
            r#"{"Version":"7.0.0","CockpitScale":1,"BonnetFOV":91,"BonnetFov":99,"CockpitAnchorX":1700}"#,
        ] {
            let cfg = load(text);
            assert_eq!(cfg.bonnet_fov, 91.0, "{text}");
            assert_eq!(cfg.cockpit_anchor_x, 1700, "{text}");
        }
        // A lone alias still loads.
        assert_eq!(load(r#"{"BumperFov":70}"#).bumper_fov, 70.0);
    }

    #[test]
    fn unknown_keys_keep_their_order_on_save() {
        let cfg = load(r#"{"Version":"7.0.0","Zeta":1,"CockpitScale":1,"Alpha":2,"Mid":3,"Last":4}"#);
        let keys: Vec<&str> = cfg.extra.keys().map(String::as_str).collect();
        assert_eq!(keys, ["Zeta", "Alpha", "Mid", "Last"]);
    }

    #[test]
    fn one_bad_safe_zone_margin_keeps_the_others() {
        let cfg = load(r#"{"CockpitSafeZone":{"SidePx":"wide","TopPx":12,"BottomPx":60}}"#);
        let d = SafeZone::default();
        assert_eq!(
            cfg.cockpit_safe_zone,
            SafeZone {
                side_px: d.side_px,
                top_px: 12.0,
                bottom_px: 60.0
            }
        );
        // A missing margin takes its default too.
        let cfg = load(r#"{"BonnetSafeZone":{"TopPx":8}}"#);
        assert_eq!(cfg.bonnet_safe_zone, SafeZone { top_px: 8.0, ..d });
    }

    #[test]
    fn numbers_too_large_for_the_setting_take_their_default() {
        let cfg = load(r#"{"WheelLessCockpitFOV":1e300,"AmbientGlowIntensity":1e300,"HoodPitchOffset":-1e300}"#);
        let d = AppConfig::default();
        assert_eq!(cfg.wheel_less_cockpit_fov, d.wheel_less_cockpit_fov);
        assert_eq!(cfg.ambient_glow_intensity, d.ambient_glow_intensity);
        assert_eq!(cfg.hood_pitch_offset, d.hood_pitch_offset);
        // In range otherwise, and a save never writes null.
        let cfg = load(r#"{"BonnetFOV":200,"AmbientGlowIntensity":3}"#);
        assert_eq!((cfg.bonnet_fov, cfg.ambient_glow_intensity), (FOV_RANGE.1, 1.0));
        let saved = serde_json::to_string(&cfg).expect("serialises");
        assert!(!saved.contains("null"), "{saved}");
    }

    #[test]
    fn retired_keys_are_dropped_and_not_saved() {
        let mut raw = Map::new();
        for key in RETIRED_KEYS {
            raw.insert((*key).to_owned(), Value::from(1));
        }
        let cfg = load(&Value::Object(raw).to_string());
        let json = serde_json::to_string(&cfg).expect("serialises");
        for key in RETIRED_KEYS {
            assert!(!json.contains(&format!("\"{key}\"")), "{key} was saved");
        }
        assert!(cfg.extra.is_empty(), "{:?}", cfg.extra);
    }

    #[test]
    fn no_retired_key_is_a_setting() {
        let json = serde_json::to_value(AppConfig::default()).expect("serialises");
        for key in RETIRED_KEYS {
            assert!(json.get(*key).is_none(), "{key} is both retired and a setting");
        }
    }

    #[test]
    fn an_old_seat_height_migrates_once() {
        let old = r#"{"Version":"6.0.0","CockpitOffsetY":-0.08}"#;
        let cfg = load(old);
        assert_eq!(cfg.driver_seat_height_m, -0.08);
        assert_eq!(cfg.version, CONFIG_VERSION);
        assert!(!cfg.extra.contains_key("CockpitOffsetY"));
        // The old value is clamped to the seat range.
        assert_eq!(load(r#"{"CockpitOffsetY":-0.5}"#).driver_seat_height_m, -0.100);
        // A seat the user set is never overwritten.
        let set = load(r#"{"Version":"6.0.0","CockpitOffsetY":-0.08,"DriverSeatHeightM":0.01}"#);
        assert_eq!(set.driver_seat_height_m, 0.01);
        // The bug this fixes: the user sets the seat back to the default, the file still holds
        // the old key, and the next load brought the old value back. The Rust build before 7.0.0
        // always wrote `DriverSeatHeightM`, so its files keep their seat, and so does a 7.0.0 file.
        for version in ["6.0.0", "7.0.0"] {
            let text = format!(r#"{{"Version":"{version}","CockpitOffsetY":-0.08,"DriverSeatHeightM":-0.045}}"#);
            assert_eq!(load(&text).driver_seat_height_m, DEFAULT_SEAT_HEIGHT_M, "{version}");
        }
        // After one save nothing is left to migrate.
        let saved = serde_json::to_string(&cfg).expect("serialises");
        assert!(!saved.contains("CockpitOffsetY"), "{saved}");
        assert_eq!(load(&saved), cfg);
    }

    #[test]
    fn the_old_default_driver_name_is_cleared_and_an_empty_name_stays_empty() {
        let old = format!(r#"{{"Version":"6.0.0","CustomDriverName":"{OLD_DEFAULT_DRIVER_NAME}"}}"#);
        assert_eq!(load(&old).custom_driver_name, "");
        assert_eq!(load(r#"{"CustomDriverName":""}"#).custom_driver_name, "");
        assert_eq!(AppConfig::default().custom_driver_name, "");
        assert_eq!(load(r#"{"CustomDriverName":"TJX"}"#).custom_driver_name, "TJX");
        let kept = format!(r#"{{"Version":"7.0.0","CustomDriverName":"{OLD_DEFAULT_DRIVER_NAME}"}}"#);
        assert_eq!(
            load(&kept).custom_driver_name,
            OLD_DEFAULT_DRIVER_NAME,
            "a name typed in a 7.0.0 file is the user's"
        );
    }

    #[test]
    fn an_old_file_keeps_the_power_unit_it_showed() {
        let mph = load(r#"{"Version":"6.0.0","SpeedUnit":"MPH","PowerUnit":"kW"}"#);
        assert_eq!(mph.power_unit, PowerUnit::Hp);
        let kmh = load(r#"{"Version":"6.0.0","SpeedUnit":"KPH"}"#);
        assert_eq!((kmh.speed_unit, kmh.power_unit), (SpeedUnit::Kmh, PowerUnit::Kw));
        let chosen = load(r#"{"Version":"6.0.0","SpeedUnit":"KMH","PowerUnit":"HP"}"#);
        assert_eq!(chosen.power_unit, PowerUnit::Hp, "a non-default choice is kept");
        let new = load(r#"{"Version":"7.0.0","SpeedUnit":"MPH","PowerUnit":"kW"}"#);
        assert_eq!(new.power_unit, PowerUnit::Kw, "7.0.0 files are taken as written");
    }

    #[test]
    fn a_newer_file_keeps_its_version_and_its_keys() {
        assert_eq!(load(r#"{"Version":"8.1.0"}"#).version, "8.1.0");
        // A newer build may reuse a retired name, and no migration runs.
        let cfg = load(r#"{"Version":"8.0.0","FontPreset":"Inter","PowerUnit":"kW"}"#);
        assert_eq!(cfg.extra.get("FontPreset"), Some(&Value::from("Inter")));
        assert_eq!(cfg.power_unit, PowerUnit::Kw);
        // A numeric version counts too, and is kept as text.
        let cfg = load(r#"{"Version":8,"PowerUnit":"kW"}"#);
        assert_eq!((cfg.version.as_str(), cfg.power_unit), ("8", PowerUnit::Kw));
    }

    #[test]
    fn a_version_this_build_cannot_read_is_kept_and_not_migrated() {
        let old_name = format!(r#"{{"Version":"v7","CustomDriverName":"{OLD_DEFAULT_DRIVER_NAME}"}}"#);
        let cfg = load(&old_name);
        assert_eq!(cfg.version, "v7");
        assert_eq!(cfg.custom_driver_name, OLD_DEFAULT_DRIVER_NAME);
        // "7-beta" reads as 7: no migration either.
        let cfg = load(r#"{"Version":"7-beta","PowerUnit":"kW"}"#);
        assert_eq!((cfg.version.as_str(), cfg.power_unit), ("7-beta", PowerUnit::Kw));
    }

    #[test]
    fn every_key_a_migration_writes_is_a_setting() {
        let cfg = AppConfig {
            chase_hud_style: Some(ClusterTheme::BoschDdu),
            ..AppConfig::default()
        };
        let saved = serde_json::to_value(&cfg).expect("serialises");
        let mut keys = vec![
            "Version",
            "PowerUnit",
            "SpeedUnit",
            "DriverSeatHeightM",
            "CustomDriverName",
            "CockpitHudStyle",
            "BonnetHudStyle",
            "ChaseHudStyle",
            "CockpitHeightOffsetPx",
            "BonnetHeightOffsetPx",
            "ChaseHeightOffsetPx",
        ];
        keys.extend(KEY_ALIASES.iter().map(|(canonical, _)| *canonical));
        for key in keys {
            assert!(saved.get(key).is_some(), "{key} is not a saved setting");
        }
        for (_, alias) in KEY_ALIASES {
            assert!(saved.get(alias).is_none(), "{alias} is saved");
        }
    }

    #[test]
    fn the_old_cluster_height_fills_the_unset_views() {
        let cfg = load(r#"{"ClusterHeightOffsetPx":-30,"BonnetHeightOffsetPx":12}"#);
        assert_eq!(cfg.cockpit_height_offset_px, -30.0);
        assert_eq!(cfg.bonnet_height_offset_px, 12.0);
        assert_eq!(cfg.chase_height_offset_px, -30.0);
        assert!(!cfg.extra.contains_key("ClusterHeightOffsetPx"));
    }

    #[test]
    fn the_osd_settings_are_typed_fields() {
        let cfg = load(
            r#"{"ReducedFlash":true,"GlassRefraction":true,"PaletteSeed":4242,"ClusterFont":"Inter",
                "ShowLogo":false,"ShowStats":true,"ShiftPointPct":0.9,"ColorMode":"random"}"#,
        );
        assert!(cfg.reduced_flash && cfg.glass_refraction && cfg.show_stats && !cfg.show_logo);
        assert_eq!(cfg.palette_seed, Some(4242));
        assert_eq!(cfg.cluster_font.as_deref(), Some("Inter"));
        assert_eq!(cfg.shift_point_pct, Some(0.9));
        assert_eq!(cfg.color_mode, ColorMode::Random);
        assert!(cfg.extra.is_empty(), "{:?}", cfg.extra);
        let d = AppConfig::default();
        assert!(d.show_logo && !d.show_stats && !d.reduced_flash && !d.glass_refraction);
    }

    #[test]
    fn setting_names_parse_and_print() {
        for u in [SpeedUnit::Mph, SpeedUnit::Kmh] {
            assert_eq!(u.name().parse(), Ok(u));
            assert_eq!(u.to_string(), u.name());
        }
        assert_eq!(" km/h ".parse(), Ok(SpeedUnit::Kmh));
        assert_eq!(SpeedUnit::Kmh.label(), "KM/H");
        for p in [PowerUnit::Hp, PowerUnit::Kw] {
            assert_eq!(p.name().parse(), Ok(p));
        }
        assert_eq!("KW".parse(), Ok(PowerUnit::Kw));
        for m in [
            ColorMode::AutoBrand,
            ColorMode::Brand,
            ColorMode::Custom,
            ColorMode::Random,
        ] {
            assert_eq!(m.name().to_lowercase().parse(), Ok(m));
        }
        assert_eq!("Factory".parse(), Ok(ColorMode::AutoBrand));
        let err = "furlongs".parse::<SpeedUnit>().expect_err("not a unit");
        assert_eq!(err.setting(), "speed unit");
        assert_eq!(err.to_string(), "unknown speed unit");
    }

    #[test]
    fn a_load_save_load_cycle_is_stable() {
        let cfg = load(
            r#"{"Version":"6.0.0","CockpitHudStyle":"bosch","ChaseHudStyle":"RetroCyber","SpeedUnit":"kmh",
                "PaletteSeed":7,"ShiftPointPct":0.95,"CockpitOffsetY":-0.06,"PorscheAnalogScale":"Classic"}"#,
        );
        let saved = serde_json::to_string_pretty(&cfg).expect("serialises");
        let again = load(&saved);
        assert_eq!(again, cfg);
        assert_eq!(serde_json::to_string_pretty(&again).expect("serialises"), saved);
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
