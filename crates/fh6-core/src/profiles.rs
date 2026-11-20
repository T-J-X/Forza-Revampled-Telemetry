use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CarProfile {
    #[serde(default)]
    pub car_ordinal: i32,
    #[serde(default)]
    pub car_name: String,
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
    pub cockpit_offset_y: f32,
    #[serde(default)]
    pub night_ambient_glow_enabled: bool,
    #[serde(default)]
    pub night_ambient_glow_intensity_pct: i32,
}

impl Default for CarProfile {
    fn default() -> Self {
        Self {
            car_ordinal: 0,
            car_name: String::new(),
            cockpit_anchor_x: 1920,
            cockpit_anchor_y: 1835,
            cockpit_scale: 1.48,
            bonnet_anchor_x: 1920,
            bonnet_anchor_y: 1760,
            bonnet_scale: 1.55,
            cockpit_hud_style: "BoschDdu".to_string(),
            bonnet_hud_style: "Gt7HoodPro".to_string(),
            oem_dash_opaque_pct: 100,
            oem_dash_binnacle_offset_y: 0,
            oem_dash_width_scale: 1.0,
            cockpit_offset_y: -0.045,
            night_ambient_glow_enabled: true,
            night_ambient_glow_intensity_pct: 75,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CarProfilesFile {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub profiles: HashMap<String, CarProfile>,
}

impl Default for CarProfilesFile {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            profiles: HashMap::new(),
        }
    }
}

pub struct CarProfileDatabase {
    file_path: String,
    file_data: CarProfilesFile,
}

impl CarProfileDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_ref = path.as_ref();
        let path_str = path_ref.to_string_lossy().to_string();
        let file_data = Self::load_or_create(path_ref);
        Self {
            file_path: path_str,
            file_data,
        }
    }

    fn load_or_create(path: &Path) -> CarProfilesFile {
        if path.exists() {
            if let Ok(s) = fs::read_to_string(path) {
                if let Ok(file) = serde_json::from_str::<CarProfilesFile>(&s) {
                    if !file.profiles.is_empty() {
                        return file;
                    }
                }
            }
        }
        let mut def = CarProfilesFile::default();
        def.profiles.insert(
            "4122".to_string(),
            CarProfile {
                car_ordinal: 4122,
                car_name: "2025 BMW M4 CS".to_string(),
                cockpit_anchor_x: 1920,
                cockpit_anchor_y: 1835,
                cockpit_scale: 1.48,
                bonnet_anchor_x: 1920,
                bonnet_anchor_y: 1760,
                bonnet_scale: 1.55,
                cockpit_hud_style: "LiquidGlass".to_string(),
                bonnet_hud_style: "Gt7HoodPro".to_string(),
                oem_dash_opaque_pct: 100,
                oem_dash_binnacle_offset_y: 0,
                oem_dash_width_scale: 1.0,
                cockpit_offset_y: -0.045,
                night_ambient_glow_enabled: true,
                night_ambient_glow_intensity_pct: 75,
            },
        );
        def.profiles.insert(
            "3595".to_string(),
            CarProfile {
                car_ordinal: 3595,
                car_name: "2020 Ferrari SF90 Stradale".to_string(),
                cockpit_anchor_x: 1920,
                cockpit_anchor_y: 1820,
                cockpit_scale: 1.45,
                bonnet_anchor_x: 1920,
                bonnet_anchor_y: 1760,
                bonnet_scale: 1.55,
                cockpit_hud_style: "BoschDdu".to_string(),
                bonnet_hud_style: "Gt7HoodPro".to_string(),
                oem_dash_opaque_pct: 95,
                oem_dash_binnacle_offset_y: -5,
                oem_dash_width_scale: 1.02,
                cockpit_offset_y: -0.045,
                night_ambient_glow_enabled: true,
                night_ambient_glow_intensity_pct: 75,
            },
        );
        def.profiles.insert(
            "3625".to_string(),
            CarProfile {
                car_ordinal: 3625,
                car_name: "2021 Rimac Nevera".to_string(),
                cockpit_anchor_x: 1920,
                cockpit_anchor_y: 1840,
                cockpit_scale: 1.50,
                bonnet_anchor_x: 1920,
                bonnet_anchor_y: 1760,
                bonnet_scale: 1.55,
                cockpit_hud_style: "TeslaAdas".to_string(),
                bonnet_hud_style: "Gt7HoodPro".to_string(),
                oem_dash_opaque_pct: 100,
                oem_dash_binnacle_offset_y: 0,
                oem_dash_width_scale: 1.0,
                cockpit_offset_y: -0.045,
                night_ambient_glow_enabled: true,
                night_ambient_glow_intensity_pct: 75,
            },
        );
        def.profiles.insert(
            "3781".to_string(),
            CarProfile {
                car_ordinal: 3781,
                car_name: "2023 Porsche 911 GT3 RS".to_string(),
                cockpit_anchor_x: 1920,
                cockpit_anchor_y: 1830,
                cockpit_scale: 1.46,
                bonnet_anchor_x: 1920,
                bonnet_anchor_y: 1760,
                bonnet_scale: 1.55,
                cockpit_hud_style: "PorscheAnalog".to_string(),
                bonnet_hud_style: "Gt7HoodPro".to_string(),
                oem_dash_opaque_pct: 100,
                oem_dash_binnacle_offset_y: 2,
                oem_dash_width_scale: 0.98,
                cockpit_offset_y: -0.045,
                night_ambient_glow_enabled: true,
                night_ambient_glow_intensity_pct: 75,
            },
        );
        def
    }

    pub fn try_get_profile(&self, car_ordinal: i32) -> Option<&CarProfile> {
        let key = car_ordinal.to_string();
        self.file_data.profiles.get(&key)
    }

    pub fn save_or_update_profile(&mut self, car_ordinal: i32, config: &AppConfig, car_name: Option<&str>) {
        if car_ordinal <= 0 {
            return;
        }
        let key = car_ordinal.to_string();
        let default_name = car_name.unwrap_or(&format!("Car Ordinal {}", car_ordinal)).to_string();
        let prof = self
            .file_data
            .profiles
            .entry(key.clone())
            .or_insert_with(|| CarProfile {
                car_ordinal,
                car_name: default_name,
                ..Default::default()
            });
        if let Some(name) = car_name {
            prof.car_name = name.to_string();
        }
        prof.cockpit_anchor_x = config.cockpit_anchor_x;
        prof.cockpit_anchor_y = config.cockpit_anchor_y;
        prof.cockpit_scale = config.cockpit_scale;
        prof.bonnet_anchor_x = config.bonnet_anchor_x;
        prof.bonnet_anchor_y = config.bonnet_anchor_y;
        prof.bonnet_scale = config.bonnet_scale;
        prof.cockpit_hud_style = config.cockpit_hud_style.clone();
        prof.bonnet_hud_style = config.bonnet_hud_style.clone();
        prof.oem_dash_opaque_pct = config.oem_dash_opaque_pct;
        prof.oem_dash_binnacle_offset_y = config.oem_dash_binnacle_offset_y;
        prof.oem_dash_width_scale = config.oem_dash_width_scale;
        prof.cockpit_offset_y = config.cockpit_offset_y;
        prof.night_ambient_glow_enabled = config.night_ambient_glow_enabled;
        prof.night_ambient_glow_intensity_pct = config.night_ambient_glow_intensity_pct;
        let _ = self.persist();
    }

    pub fn persist(&self) -> Result<(), std::io::Error> {
        if self.file_path.is_empty() {
            return Ok(());
        }
        let json = serde_json::to_string(&self.file_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let tmp_path = format!("{}.tmp", self.file_path);
        fs::write(&tmp_path, json)?;
        if std::path::Path::new(&self.file_path).exists() {
            let _ = fs::remove_file(&self.file_path);
        }
        fs::rename(&tmp_path, &self.file_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_profile_values() {
        let p = CarProfile::default();
        assert_eq!(p.cockpit_anchor_x, 1920);
        assert_eq!(p.cockpit_hud_style, "BoschDdu");
        assert!(p.night_ambient_glow_enabled);
    }

    #[test]
    fn database_defaults_contain_expected_ordinals() {
        let db = CarProfileDatabase::new(std::path::Path::new("/tmp/nonexistent.json"));
        assert!(db.try_get_profile(4122).is_some());
        assert!(db.try_get_profile(3595).is_some());
        assert!(db.try_get_profile(3625).is_some());
        assert!(db.try_get_profile(3781).is_some());
    }
}
