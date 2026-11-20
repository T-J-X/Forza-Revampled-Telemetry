use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct BrandPalette {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub primary_hex: String,
    #[serde(default)]
    pub secondary_hex: String,
    #[serde(default)]
    pub accent_hex: String,
    #[serde(default)]
    pub shift_flash_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CarEntry {
    #[serde(default)]
    pub brand_key: String,
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub manufacturer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CarDatabase {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub total_cars: i32,
    #[serde(default)]
    pub brand_palettes: HashMap<String, BrandPalette>,
    #[serde(default)]
    pub cars: HashMap<String, CarEntry>,
}

impl CarDatabase {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if path.exists() {
            if let Ok(s) = fs::read_to_string(path) {
                if let Ok(db) = serde_json::from_str::<CarDatabase>(&s) {
                    return db;
                }
            }
        }
        Self::default()
    }

    pub fn get_brand_palette(&self, brand_key: &str) -> Option<&BrandPalette> {
        let key = brand_key.to_ascii_lowercase();
        self.brand_palettes
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == key)
            .map(|(_, v)| v)
    }

    pub fn get_car_entry(&self, ordinal: &str) -> Option<&CarEntry> {
        let key = ordinal.trim();
        self.cars.get(key)
    }

    pub fn get_brand_palette_for_ordinal(&self, ordinal: &str) -> Option<&BrandPalette> {
        self.get_car_entry(ordinal)
            .and_then(|e| self.get_brand_palette(&e.brand_key))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CarOrdinals {
    #[serde(flatten)]
    pub map: HashMap<String, String>,
}

impl CarOrdinals {
    pub fn load<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();
        if path.exists() {
            if let Ok(s) = fs::read_to_string(path) {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&s) {
                    return CarOrdinals { map };
                }
            }
        }
        Self { map: HashMap::new() }
    }

    pub fn ordinal_for_name(&self, name: &str) -> Option<&str> {
        let key = name.to_ascii_lowercase();
        self.map
            .iter()
            .find(|(k, _)| k.to_ascii_lowercase() == key)
            .map(|(_, v)| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_palette_lookup_case_insensitive() {
        let mut db = CarDatabase::default();
        db.brand_palettes.insert(
            "FERRARI".to_string(),
            BrandPalette {
                display_name: "Ferrari".to_string(),
                primary_hex: "#D40000".to_string(),
                secondary_hex: "#FFCC00".to_string(),
                accent_hex: "#FF2800".to_string(),
                shift_flash_hex: "#FFFFFF".to_string(),
            },
        );
        assert!(db.get_brand_palette("ferrari").is_some());
        assert!(db.get_brand_palette("FERRARI").is_some());
        assert!(db.get_brand_palette("Ferrari").is_some());
    }
}
