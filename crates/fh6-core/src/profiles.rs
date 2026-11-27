//! Per-car profiles (`data/car_profiles.json`): fit overrides keyed by the telemetry's car
//! ordinal. Every field is optional; a missing field inherits the user's `config.json` value.
//!
//! When `PerCarProfileEnabled` is on, the host overlays the profile of the car being driven on
//! the live config ([`CarProfilesFile::effective_config`]) whenever `car_ordinal` changes, so
//! the scrim dome and the cluster placement follow the car automatically.
use crate::config::AppConfig;
use crate::theme::ClusterTheme;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The profile file name, kept next to `config.json`.
pub const PROFILES_FILE_NAME: &str = "car_profiles.json";

/// Why a profiles file could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ProfilesError {
    /// The file could not be read.
    #[error("reading {path}")]
    Read {
        /// The file.
        path: PathBuf,
        /// The I/O error.
        #[source]
        source: io::Error,
    },
    /// The file is not valid profiles JSON.
    #[error("parsing {path}")]
    Parse {
        /// The file.
        path: PathBuf,
        /// The JSON error.
        #[source]
        source: serde_json::Error,
    },
}

/// Overrides for one car. `None` inherits the config value.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CarProfile {
    /// The car ordinal (also the map key).
    pub car_ordinal: i32,
    /// Display name.
    pub car_name: String,

    // Cluster placement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cockpit_anchor_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cockpit_anchor_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cockpit_height_offset_px: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cockpit_cluster_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonnet_anchor_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonnet_anchor_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonnet_height_offset_px: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonnet_cluster_scale: Option<f32>,
    // Styles. A name no cluster has is logged and ignored, so the rest of the file still loads.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "lenient_theme")]
    pub cockpit_hud_style: Option<ClusterTheme>,
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "lenient_theme")]
    pub bonnet_hud_style: Option<ClusterTheme>,

    // Scrim dome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_opaque_pct: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_offset_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_binnacle_offset_y: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_width_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_height_scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_top_arch: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_corner_softness: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_feather_px: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oem_dash_column_void_px: Option<f32>,

    // Ambient wash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub night_ambient_glow_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub night_ambient_glow_intensity_pct: Option<i32>,

    /// Seat height (metres, DriverCam `Param.y`). Recorded only: the game reads its camera
    /// files at start, so a per-car seat cannot change while driving.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cockpit_offset_y: Option<f32>,
}

impl CarProfile {
    /// Writes every set field over `cfg`.
    pub fn apply_to(&self, cfg: &mut AppConfig) {
        fn set<T: Clone>(dst: &mut T, v: &Option<T>) {
            if let Some(v) = v {
                *dst = v.clone();
            }
        }
        set(&mut cfg.cockpit_anchor_x, &self.cockpit_anchor_x);
        set(&mut cfg.cockpit_anchor_y, &self.cockpit_anchor_y);
        set(&mut cfg.cockpit_height_offset_px, &self.cockpit_height_offset_px);
        set(&mut cfg.cockpit_cluster_scale, &self.cockpit_cluster_scale);
        set(&mut cfg.bonnet_anchor_x, &self.bonnet_anchor_x);
        set(&mut cfg.bonnet_anchor_y, &self.bonnet_anchor_y);
        set(&mut cfg.bonnet_height_offset_px, &self.bonnet_height_offset_px);
        set(&mut cfg.bonnet_cluster_scale, &self.bonnet_cluster_scale);
        set(&mut cfg.cockpit_hud_style, &self.cockpit_hud_style);
        set(&mut cfg.bonnet_hud_style, &self.bonnet_hud_style);
        set(&mut cfg.oem_dash_opaque_pct, &self.oem_dash_opaque_pct);
        set(&mut cfg.oem_dash_offset_x, &self.oem_dash_offset_x);
        set(&mut cfg.oem_dash_binnacle_offset_y, &self.oem_dash_binnacle_offset_y);
        set(&mut cfg.oem_dash_width_scale, &self.oem_dash_width_scale);
        set(&mut cfg.oem_dash_height_scale, &self.oem_dash_height_scale);
        set(&mut cfg.oem_dash_top_arch, &self.oem_dash_top_arch);
        set(&mut cfg.oem_dash_corner_softness, &self.oem_dash_corner_softness);
        set(&mut cfg.oem_dash_feather_px, &self.oem_dash_feather_px);
        set(&mut cfg.oem_dash_column_void_px, &self.oem_dash_column_void_px);
        set(&mut cfg.night_ambient_glow_enabled, &self.night_ambient_glow_enabled);
        set(
            &mut cfg.night_ambient_glow_intensity_pct,
            &self.night_ambient_glow_intensity_pct,
        );
    }

    /// A profile holding the fit values of `cfg` (placement and scrim), for "save for this car".
    #[must_use]
    pub fn fit_snapshot(car_ordinal: i32, car_name: &str, cfg: &AppConfig) -> Self {
        Self {
            car_ordinal,
            car_name: car_name.to_string(),
            cockpit_anchor_x: Some(cfg.cockpit_anchor_x),
            cockpit_anchor_y: Some(cfg.cockpit_anchor_y),
            cockpit_height_offset_px: Some(cfg.cockpit_height_offset_px),
            cockpit_cluster_scale: Some(cfg.cockpit_cluster_scale),
            bonnet_anchor_x: Some(cfg.bonnet_anchor_x),
            bonnet_anchor_y: Some(cfg.bonnet_anchor_y),
            bonnet_height_offset_px: Some(cfg.bonnet_height_offset_px),
            bonnet_cluster_scale: Some(cfg.bonnet_cluster_scale),
            oem_dash_opaque_pct: Some(cfg.oem_dash_opaque_pct),
            oem_dash_offset_x: Some(cfg.oem_dash_offset_x),
            oem_dash_binnacle_offset_y: Some(cfg.oem_dash_binnacle_offset_y),
            oem_dash_width_scale: Some(cfg.oem_dash_width_scale),
            oem_dash_height_scale: Some(cfg.oem_dash_height_scale),
            oem_dash_top_arch: Some(cfg.oem_dash_top_arch),
            oem_dash_corner_softness: Some(cfg.oem_dash_corner_softness),
            oem_dash_feather_px: Some(cfg.oem_dash_feather_px),
            oem_dash_column_void_px: Some(cfg.oem_dash_column_void_px),
            ..Self::default()
        }
    }
}

/// The whole `car_profiles.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CarProfilesFile {
    /// Schema version (`2.0.0`: optional overrides).
    pub version: String,
    /// Profiles by car ordinal. A key that is not a positive ordinal is logged and skipped.
    #[serde(deserialize_with = "ordinal_keys")]
    pub profiles: BTreeMap<i32, CarProfile>,
}

impl Default for CarProfilesFile {
    fn default() -> Self {
        Self {
            version: "2.0.0".to_string(),
            profiles: BTreeMap::new(),
        }
    }
}

impl CarProfilesFile {
    /// Parses profiles JSON (a leading BOM is ignored).
    ///
    /// # Errors
    /// The JSON error if the text is not a profiles object.
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(text))
    }

    /// Reads and parses a profiles file.
    ///
    /// # Errors
    /// [`ProfilesError::Read`] or [`ProfilesError::Parse`].
    pub fn try_load(path: &Path) -> Result<Self, ProfilesError> {
        let text = fs::read_to_string(path).map_err(|source| ProfilesError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_str(&text).map_err(|source| ProfilesError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Loads a profiles file; a missing or invalid file gives no profiles.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        match Self::try_load(path) {
            Ok(p) => p,
            Err(ProfilesError::Read { source, .. }) if source.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(err) => {
                tracing::warn!(error = %err, "car profiles not loaded");
                Self::default()
            }
        }
    }

    /// Writes the file atomically, UTF-8 without a BOM.
    ///
    /// # Errors
    /// The I/O error if it cannot be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        crate::atomic_file::write_atomically(path, json.as_bytes())
    }

    /// The profile of `car_ordinal`, if any.
    #[must_use]
    pub fn get(&self, car_ordinal: i32) -> Option<&CarProfile> {
        self.profiles.get(&car_ordinal)
    }

    /// Inserts or replaces a profile.
    pub fn upsert(&mut self, profile: CarProfile) {
        if profile.car_ordinal > 0 {
            self.profiles.insert(profile.car_ordinal, profile);
        }
    }

    /// The config to use while driving `car_ordinal`: `base` with that car's profile applied,
    /// or `None` when profiles are off or the car has none (use `base` as is).
    #[must_use]
    pub fn effective_config(&self, base: &AppConfig, car_ordinal: i32) -> Option<AppConfig> {
        if !base.per_car_profile_enabled {
            return None;
        }
        let profile = self.get(car_ordinal)?;
        let mut cfg = base.clone();
        profile.apply_to(&mut cfg);
        // Re-run the range checks: a profile can hold anything.
        Some(AppConfig::sanitized(cfg))
    }
}

/// A profile style: a cluster name or alias; anything else is logged and gives no override.
fn lenient_theme<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<ClusterTheme>, D::Error> {
    let name = Option::<String>::deserialize(deserializer)?;
    Ok(name.and_then(|name| match name.parse() {
        Ok(theme) => Some(theme),
        Err(_) => {
            if !name.trim().is_empty() {
                tracing::warn!(style = %name, "car profile names no cluster; style ignored");
            }
            None
        }
    }))
}

/// The profiles map: JSON object keys are text, parsed to ordinals once at load.
fn ordinal_keys<'de, D: Deserializer<'de>>(deserializer: D) -> Result<BTreeMap<i32, CarProfile>, D::Error> {
    let raw = BTreeMap::<String, CarProfile>::deserialize(deserializer)?;
    Ok(raw
        .into_iter()
        .filter_map(|(key, profile)| match key.trim().parse::<i32>() {
            Ok(ordinal) if ordinal > 0 => Some((ordinal, profile)),
            _ => {
                tracing::warn!(%key, "car profile key is not an ordinal; profile skipped");
                None
            }
        })
        .collect())
}

/// Shared, lock-free handle to the live profiles (see [`crate::ConfigWatcher`]).
#[derive(Clone, Debug)]
pub struct ProfilesHandle {
    live: Arc<arc_swap::ArcSwap<CarProfilesFile>>,
}

impl ProfilesHandle {
    /// A handle holding fixed profiles.
    #[must_use]
    pub fn fixed(p: CarProfilesFile) -> Self {
        Self {
            live: Arc::new(arc_swap::ArcSwap::from_pointee(p)),
        }
    }

    /// The profiles as of now (no lock, no allocation).
    #[must_use]
    pub fn current(&self) -> Arc<CarProfilesFile> {
        self.live.load_full()
    }

    pub(crate) fn publish(&self, p: CarProfilesFile) {
        self.live.store(Arc::new(p));
    }
}

/// The profiles file that belongs to a config file (same folder).
#[must_use]
pub fn profiles_path_for(config_path: &Path) -> PathBuf {
    config_path
        .parent()
        .map_or_else(|| PathBuf::from(PROFILES_FILE_NAME), |d| d.join(PROFILES_FILE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_inherit_the_config() {
        let file = CarProfilesFile::from_json_str(
            r#"{"Version":"2.0.0","Profiles":{"3595":{"CarOrdinal":3595,"CockpitAnchorY":1820,"OemDashWidthScale":1.1}}}"#,
        )
        .expect("valid");
        let base = AppConfig {
            oem_dash_opaque_pct: 80,
            ..AppConfig::default()
        };
        let eff = file.effective_config(&base, 3595).expect("profile applies");
        assert_eq!(eff.cockpit_anchor_y, 1820);
        assert_eq!(eff.oem_dash_width_scale, 1.1);
        assert_eq!(eff.oem_dash_opaque_pct, 80, "inherited, not zeroed");
        assert_eq!(eff.oem_dash_height_scale, 1.0);
        assert_eq!(eff.cockpit_hud_style, base.cockpit_hud_style);
        assert!(file.effective_config(&base, 4122).is_none(), "no profile for that car");
        assert!(file.effective_config(&base, 0).is_none());
    }

    #[test]
    fn profiles_off_means_no_overlay() {
        let mut file = CarProfilesFile::default();
        file.upsert(CarProfile {
            car_ordinal: 7,
            cockpit_anchor_y: Some(1700),
            ..CarProfile::default()
        });
        let base = AppConfig {
            per_car_profile_enabled: false,
            ..AppConfig::default()
        };
        assert!(file.effective_config(&base, 7).is_none());
    }

    #[test]
    fn profile_values_are_range_checked() {
        let mut file = CarProfilesFile::default();
        file.upsert(CarProfile {
            car_ordinal: 9,
            oem_dash_opaque_pct: Some(400),
            oem_dash_width_scale: Some(0.0),
            ..CarProfile::default()
        });
        let eff = file.effective_config(&AppConfig::default(), 9).expect("applies");
        assert_eq!(eff.oem_dash_opaque_pct, 100);
        assert!(eff.oem_dash_width_scale >= 0.5, "a zero width never collapses the dome");
    }

    #[test]
    fn v1_profiles_still_load() {
        let v1 = r#"{"Version":"1.0.0","Profiles":{"4122":{"CarOrdinal":4122,"CarName":"2025 BMW M4 CS","CockpitAnchorX":1920,"CockpitAnchorY":1835,"CockpitScale":1.48,"CockpitHudStyle":"LiquidGlass","OemDashOpaquePct":100,"OemDashBinnacleOffsetY":0,"OemDashWidthScale":1.0,"CockpitOffsetY":-0.045,"NightAmbientGlowEnabled":true,"NightAmbientGlowIntensityPct":75}}}"#;
        let file = CarProfilesFile::from_json_str(v1).expect("v1 is a subset");
        let p = file.get(4122).expect("present");
        assert_eq!(p.cockpit_hud_style, Some(ClusterTheme::LiquidGlass));
        assert_eq!(p.oem_dash_width_scale, Some(1.0));
    }

    #[test]
    fn shipped_profiles_file_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/car_profiles.json");
        let file = CarProfilesFile::try_load(&path).expect("data/car_profiles.json parses");
        assert!(!file.profiles.is_empty());
        for (k, p) in &file.profiles {
            assert_eq!(*k, p.car_ordinal, "key matches ordinal");
        }
    }

    #[test]
    fn save_round_trip_without_bom() {
        let dir = std::env::temp_dir().join(format!("fh6_profiles_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(PROFILES_FILE_NAME);
        let mut file = CarProfilesFile::default();
        file.upsert(CarProfile::fit_snapshot(3781, "Porsche", &AppConfig::default()));
        file.save(&path).expect("saves");
        let bytes = fs::read(&path).expect("reads");
        assert!(!bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert_eq!(CarProfilesFile::try_load(&path).expect("loads"), file);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_profile_style_overrides_the_view_and_a_bad_one_is_ignored() {
        let file = CarProfilesFile::from_json_str(
            r#"{"Profiles":{"7":{"CarOrdinal":7,"CockpitHudStyle":"porsche","BonnetHudStyle":"NoSuchDash"},
                "x":{"CarOrdinal":8},"-3":{"CarOrdinal":-3}}}"#,
        )
        .expect("one bad style or key does not fail the file");
        assert_eq!(file.profiles.len(), 1, "keys that are not ordinals are skipped");
        let base = AppConfig::default();
        let eff = file.effective_config(&base, 7).expect("profile applies");
        assert_eq!(eff.cockpit_hud_style, ClusterTheme::PorscheAnalog);
        assert_eq!(eff.bonnet_hud_style, base.bonnet_hud_style);
        let json = serde_json::to_string(&file).expect("serialises");
        assert!(
            json.contains(r#""Profiles":{"7":"#),
            "ordinal keys are written as text: {json}"
        );
        assert!(json.contains(r#""CockpitHudStyle":"PorscheAnalog""#), "{json}");
    }
}
