//! The car database (`assets/car_brands/car_brands.json`): each car's brand by telemetry ordinal,
//! and each brand's palette.
//!
//! Keys are normalised once, at load: brand keys are trimmed and upper-cased, and car keys are
//! parsed to the ordinal. A lookup never allocates.
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The brand key of the fallback palette.
pub const DEFAULT_BRAND_KEY: &str = "DEFAULT";

/// Why the car database could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum CarDatabaseError {
    /// The file could not be read.
    #[error("reading {path}")]
    Read {
        /// The file that failed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The file is not a car database.
    #[error("parsing {path}")]
    Parse {
        /// The file that failed.
        path: PathBuf,
        /// The underlying JSON error.
        #[source]
        source: serde_json::Error,
    },
}

/// A brand's colours, as `#RRGGBB` text.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct BrandPalette {
    /// The brand's name as shown to the user.
    pub display_name: String,
    /// Primary colour.
    pub primary_hex: String,
    /// Secondary colour.
    pub secondary_hex: String,
    /// Accent colour.
    pub accent_hex: String,
    /// Shift-light colour.
    pub shift_flash_hex: String,
}

/// One car.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct CarEntry {
    /// Its brand's key in the palettes (trimmed and upper-cased by [`CarDatabase`]).
    pub brand_key: String,
    /// The model name.
    pub model_name: String,
    /// The maker's name.
    pub manufacturer: String,
}

/// Cars by ordinal and brand palettes by key. Build it with [`CarDatabase::load`] or
/// [`CarDatabase::from_json_str`]; the default is empty.
#[derive(Debug, Clone, Default)]
pub struct CarDatabase {
    version: String,
    palettes: HashMap<String, BrandPalette>,
    cars: HashMap<i32, CarEntry>,
}

/// The file as written. Ordered maps, so that key clashes resolve and log the same way every
/// time.
#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase", default)]
struct RawDatabase {
    version: String,
    brand_palettes: BTreeMap<String, BrandPalette>,
    cars: BTreeMap<String, CarEntry>,
}

impl CarDatabase {
    /// Reads and parses a car database file.
    ///
    /// # Errors
    ///
    /// [`CarDatabaseError::Read`] if the file cannot be read, [`CarDatabaseError::Parse`] if it
    /// is not a car database.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CarDatabaseError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| CarDatabaseError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_json_str(&text).map_err(|source| CarDatabaseError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    /// Parses car database JSON (a leading BOM is ignored). A car whose key is not an ordinal is
    /// skipped with a warning.
    ///
    /// # Errors
    ///
    /// The JSON error if the text is not a car database.
    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        let raw: RawDatabase = serde_json::from_str(text.strip_prefix('\u{feff}').unwrap_or(text))?;
        let mut palettes = HashMap::with_capacity(raw.brand_palettes.len());
        for (key, palette) in raw.brand_palettes {
            let key = normalise_brand_key(&key);
            if palettes.insert(key.clone(), palette).is_some() {
                tracing::warn!(brand = %key, "two brand palettes share a key; the later one is kept");
            }
        }
        let mut cars = HashMap::with_capacity(raw.cars.len());
        for (key, mut car) in raw.cars {
            let Ok(ordinal) = key.trim().parse::<i32>() else {
                tracing::warn!(%key, "car key is not an ordinal; car skipped");
                continue;
            };
            car.brand_key = normalise_brand_key(&car.brand_key);
            cars.insert(ordinal, car);
        }
        Ok(Self {
            version: raw.version,
            palettes,
            cars,
        })
    }

    /// The file's version text.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// The car with telemetry ordinal `ordinal`.
    #[must_use]
    pub fn car(&self, ordinal: i32) -> Option<&CarEntry> {
        self.cars.get(&ordinal)
    }

    /// The palette of brand `key`, matched case-insensitively with surrounding spaces ignored.
    #[must_use]
    pub fn brand_palette(&self, key: &str) -> Option<&BrandPalette> {
        let key = key.trim();
        self.palettes.get(key).or_else(|| {
            self.palettes
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, palette)| palette)
        })
    }

    /// The palette of the brand of the car with ordinal `ordinal`.
    #[must_use]
    pub fn car_palette(&self, ordinal: i32) -> Option<&BrandPalette> {
        self.car(ordinal).and_then(|car| self.brand_palette(&car.brand_key))
    }

    /// Every car with its ordinal, in no particular order.
    pub fn cars(&self) -> impl Iterator<Item = (i32, &CarEntry)> + '_ {
        self.cars.iter().map(|(&ordinal, car)| (ordinal, car))
    }

    /// The number of cars.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cars.len()
    }

    /// True when the database holds no cars.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cars.is_empty()
    }

    /// The number of brand palettes.
    #[must_use]
    pub fn brand_count(&self) -> usize {
        self.palettes.len()
    }
}

fn normalise_brand_key(key: &str) -> String {
    key.trim().to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SMALL: &str = r##"{
        "Version": "1.0",
        "BrandPalettes": {
            "FERRARI": {"DisplayName": "Ferrari", "PrimaryHex": "#D40000"},
            "DEFAULT": {"DisplayName": "Default", "PrimaryHex": "#00D2FF"}
        },
        "Cars": {
            "3595": {"BrandKey": " ferrari ", "ModelName": "SF90"},
            "12": {"BrandKey": "NOPE"},
            "not-a-number": {"BrandKey": "FERRARI"}
        }
    }"##;

    fn small() -> CarDatabase {
        CarDatabase::from_json_str(SMALL).expect("valid")
    }

    #[test]
    fn brand_palette_lookup_is_case_insensitive_and_trimmed() {
        let db = small();
        for key in ["ferrari", "FERRARI", "Ferrari", "  Ferrari "] {
            assert_eq!(
                db.brand_palette(key).map(|p| p.display_name.as_str()),
                Some("Ferrari"),
                "{key:?}"
            );
        }
        assert!(db.brand_palette("lada").is_none());
    }

    #[test]
    fn cars_are_keyed_by_ordinal_with_normalised_brand_keys() {
        let db = small();
        let car = db.car(3595).expect("present");
        assert_eq!(car.brand_key, "FERRARI");
        assert_eq!(car.model_name, "SF90");
        assert_eq!(db.car_palette(3595).map(|p| p.primary_hex.as_str()), Some("#D40000"));
        assert!(db.car_palette(12).is_none(), "brand without a palette");
        assert!(db.car(4122).is_none());
        assert_eq!(db.len(), 2, "the key that is not an ordinal is skipped");
        assert_eq!(db.brand_count(), 2);
        assert_eq!(db.version(), "1.0");
    }

    #[test]
    fn bom_and_missing_sections_are_tolerated() {
        let db = CarDatabase::from_json_str("\u{feff}{}").expect("valid");
        assert!(db.is_empty());
        assert_eq!(db.brand_count(), 0);
    }

    #[test]
    fn a_missing_file_is_an_error_that_names_it() {
        let path = std::env::temp_dir().join("fh6_no_such_car_brands.json");
        let err = CarDatabase::load(&path).expect_err("missing");
        assert!(matches!(err, CarDatabaseError::Read { .. }));
        assert!(err.to_string().contains("fh6_no_such_car_brands.json"), "{err}");
    }

    #[test]
    fn the_shipped_database_has_a_palette_for_every_car() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/car_brands/car_brands.json");
        let db = CarDatabase::load(&path).expect("assets/car_brands/car_brands.json loads");
        assert_eq!(db.len(), 693);
        assert_eq!(db.brand_count(), 37);
        assert!(db.brand_palette(DEFAULT_BRAND_KEY).is_some());
        for (ordinal, car) in db.cars() {
            assert!(
                db.brand_palette(&car.brand_key).is_some(),
                "car {ordinal} brand {:?}",
                car.brand_key
            );
        }
    }
}
