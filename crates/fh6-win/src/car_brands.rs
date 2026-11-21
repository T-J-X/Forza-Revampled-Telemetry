//! Safe editing of `car_brands.json` (binding a car ordinal to a brand).
//!
//! The real file is `{"Version", "TotalCars", "BrandPalettes": {...}, "Cars": {"247": {
//! "BrandKey", "ModelName", "Manufacturer"}, ...}}`. Editing goes through `serde_json::Value`
//! with key order preserved, so every key this code does not know survives untouched. The
//! file is only replaced (temp file, then rename) after the new text re-parses and still holds
//! every car it held before: a bind can add or replace one car, never shrink the file.
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Why a bind was refused. The file is untouched in every case.
#[derive(Debug, thiserror::Error)]
pub enum BindCarBrandError {
    /// Car ordinals are positive.
    #[error("car ordinal must be greater than 0, got {0}")]
    InvalidOrdinal(i32),
    /// No brand key given.
    #[error("brand key is empty")]
    EmptyBrandKey,
    /// The brand has no palette, so the cluster could not colour it.
    #[error("brand key {0:?} is not in BrandPalettes")]
    UnknownBrand(String),
    /// The file could not be read or written.
    #[error("{action} {path}")]
    Io {
        /// What was being done.
        action: &'static str,
        /// The file involved.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
    /// The file is not JSON.
    #[error("{path} is not valid JSON")]
    Parse {
        /// The file involved.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: serde_json::Error,
    },
    /// The JSON does not have the expected shape; rewriting it could lose data.
    #[error("{path} has no {missing:?} object; refusing to rewrite it")]
    Schema {
        /// The file involved.
        path: PathBuf,
        /// The key that is missing or not an object.
        missing: &'static str,
    },
    /// The rewritten document would hold fewer cars than the original.
    #[error("refusing to write {path}: car count would drop from {before} to {after}")]
    WouldShrink {
        /// The file involved.
        path: PathBuf,
        /// Cars before the edit.
        before: usize,
        /// Cars after the edit.
        after: usize,
    },
}

/// What a successful bind changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindOutcome {
    /// The ordinal key that was written.
    pub ordinal: String,
    /// The brand key as spelled in `BrandPalettes`.
    pub brand_key: String,
    /// True if the ordinal already existed and was replaced.
    pub replaced: bool,
    /// Number of cars after the edit (also written to `TotalCars`).
    pub total_cars: usize,
}

/// A car to bind.
#[derive(Debug, Clone, Copy)]
pub struct CarBinding<'a> {
    /// Forza's `CarOrdinal` from telemetry.
    pub ordinal: i32,
    /// A key of `BrandPalettes` (matched case-insensitively).
    pub brand_key: &'a str,
    /// Display name; empty means `"Car <ordinal>"`.
    pub model_name: &'a str,
    /// Manufacturer; empty means the palette's display name, or the brand key.
    pub manufacturer: &'a str,
}

fn io_err<'p>(action: &'static str, path: &'p Path) -> impl FnOnce(io::Error) -> BindCarBrandError + 'p {
    move |source| BindCarBrandError::Io {
        action,
        path: path.to_path_buf(),
        source,
    }
}

/// Binds `car.ordinal` to `car.brand_key` in the `car_brands.json` at `path`.
///
/// # Errors
/// See [`BindCarBrandError`]. On any error the file is left exactly as it was.
pub fn bind_car_brand(path: &Path, car: CarBinding<'_>) -> Result<BindOutcome, BindCarBrandError> {
    if car.ordinal <= 0 {
        return Err(BindCarBrandError::InvalidOrdinal(car.ordinal));
    }
    let wanted = car.brand_key.trim();
    if wanted.is_empty() {
        return Err(BindCarBrandError::EmptyBrandKey);
    }

    let text = fs::read_to_string(path).map_err(io_err("reading", path))?;
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let mut doc: Value = serde_json::from_str(text).map_err(|source| BindCarBrandError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    let schema = |missing| BindCarBrandError::Schema {
        path: path.to_path_buf(),
        missing,
    };
    let root = doc.as_object_mut().ok_or_else(|| schema("<root>"))?;

    let palettes = root
        .get("BrandPalettes")
        .and_then(Value::as_object)
        .ok_or_else(|| schema("BrandPalettes"))?;
    let (brand_key, display_name) = palettes
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(wanted))
        .map(|(k, v)| {
            let display = v.get("DisplayName").and_then(Value::as_str).unwrap_or(k).to_string();
            (k.clone(), display)
        })
        .ok_or_else(|| BindCarBrandError::UnknownBrand(wanted.to_string()))?;

    let cars = root
        .get_mut("Cars")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| schema("Cars"))?;
    let before = cars.len();
    let ordinal = car.ordinal.to_string();
    let model_name = match car.model_name.trim() {
        "" => format!("Car {ordinal}"),
        name => name.to_string(),
    };
    let manufacturer = match car.manufacturer.trim() {
        "" => display_name,
        m => m.to_string(),
    };
    let mut entry = Map::new();
    entry.insert("BrandKey".into(), Value::from(brand_key.clone()));
    entry.insert("ModelName".into(), Value::from(model_name));
    entry.insert("Manufacturer".into(), Value::from(manufacturer));
    let replaced = cars.insert(ordinal.clone(), Value::Object(entry)).is_some();
    let total_cars = cars.len();
    root.insert("TotalCars".into(), Value::from(total_cars));

    let out = serde_json::to_string_pretty(&doc).map_err(|source| BindCarBrandError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    // Verify the exact bytes about to be written before touching the real file.
    let check: Value = serde_json::from_str(&out).map_err(|source| BindCarBrandError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    let after = check.get("Cars").and_then(Value::as_object).map_or(0, Map::len);
    if after < before || after == 0 {
        return Err(BindCarBrandError::WouldShrink {
            path: path.to_path_buf(),
            before,
            after,
        });
    }

    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, out.as_bytes()).map_err(io_err("writing", &tmp))?;
    if let Err(err) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(io_err("replacing", path)(err));
    }
    Ok(BindOutcome {
        ordinal,
        brand_key,
        replaced,
        total_cars,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn real_file() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/car_brands/car_brands.json")
    }

    fn temp_copy(name: &str) -> PathBuf {
        let dst = std::env::temp_dir().join(format!("fh6_car_brands_{name}_{}.json", std::process::id()));
        fs::copy(real_file(), &dst).expect("copy the real car_brands.json");
        dst
    }

    fn load(path: &Path) -> Value {
        serde_json::from_str(&fs::read_to_string(path).expect("read")).expect("parse")
    }

    fn binding(ordinal: i32, brand: &str) -> CarBinding<'_> {
        CarBinding {
            ordinal,
            brand_key: brand,
            model_name: "",
            manufacturer: "",
        }
    }

    #[test]
    fn bind_on_real_file_adds_one_car_and_keeps_everything_else() {
        let path = temp_copy("add");
        let before = load(&path);
        let cars_before = before["Cars"].as_object().expect("Cars").len();
        let palettes_before = before["BrandPalettes"].clone();
        assert!(cars_before > 600, "the real file has ~693 cars");

        let outcome = bind_car_brand(&path, binding(999_001, "bmw")).expect("bind succeeds");
        assert!(!outcome.replaced);
        assert_eq!(outcome.brand_key, "BMW", "canonical palette key");
        assert_eq!(outcome.total_cars, cars_before + 1);

        let after = load(&path);
        assert_eq!(after["Cars"].as_object().expect("Cars").len(), cars_before + 1);
        assert_eq!(after["TotalCars"], Value::from(cars_before + 1));
        assert_eq!(after["Version"], before["Version"]);
        assert_eq!(after["BrandPalettes"], palettes_before, "palettes untouched");
        assert_eq!(after["Cars"]["999001"]["BrandKey"], "BMW");
        assert_eq!(after["Cars"]["999001"]["ModelName"], "Car 999001");
        for (k, v) in before["Cars"].as_object().expect("Cars") {
            assert_eq!(&after["Cars"][k], v, "car {k} unchanged");
        }
        let keys: Vec<&String> = after.as_object().expect("root").keys().collect();
        assert_eq!(keys, ["Version", "TotalCars", "BrandPalettes", "Cars"], "key order kept");
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_file(path).ok();
    }

    #[test]
    fn rebinding_an_existing_ordinal_replaces_it() {
        let path = temp_copy("replace");
        let outcome = bind_car_brand(
            &path,
            CarBinding {
                ordinal: 247,
                brand_key: "FERRARI",
                model_name: "Test",
                manufacturer: "Ferrari",
            },
        )
        .expect("bind succeeds");
        assert!(outcome.replaced);
        let after = load(&path);
        assert_eq!(after["Cars"]["247"]["BrandKey"], "FERRARI");
        fs::remove_file(path).ok();
    }

    #[test]
    fn legacy_lowercase_file_is_refused_and_untouched() {
        let path = std::env::temp_dir().join(format!("fh6_car_brands_legacy_{}.json", std::process::id()));
        let original = r#"{"cars":{},"total_cars":0}"#;
        fs::write(&path, original).expect("write");
        let err = bind_car_brand(&path, binding(1, "BMW")).expect_err("must refuse");
        assert!(matches!(err, BindCarBrandError::Schema { .. }), "{err}");
        assert_eq!(fs::read_to_string(&path).expect("read"), original);
        fs::remove_file(path).ok();
    }

    #[test]
    fn bad_inputs_never_touch_the_file() {
        let path = temp_copy("bad");
        let original = fs::read(&path).expect("read");
        assert!(matches!(
            bind_car_brand(&path, binding(0, "BMW")),
            Err(BindCarBrandError::InvalidOrdinal(0))
        ));
        assert!(matches!(
            bind_car_brand(&path, binding(5, "  ")),
            Err(BindCarBrandError::EmptyBrandKey)
        ));
        assert!(matches!(
            bind_car_brand(&path, binding(5, "NOT_A_BRAND")),
            Err(BindCarBrandError::UnknownBrand(_))
        ));
        assert_eq!(fs::read(&path).expect("read"), original);
        fs::remove_file(path).ok();
    }

    #[test]
    fn truncated_file_is_refused() {
        let path = std::env::temp_dir().join(format!("fh6_car_brands_trunc_{}.json", std::process::id()));
        fs::write(&path, r#"{"BrandPalettes":{"BMW":{}},"Cars":{"#).expect("write");
        assert!(matches!(
            bind_car_brand(&path, binding(1, "BMW")),
            Err(BindCarBrandError::Parse { .. })
        ));
        fs::remove_file(path).ok();
    }
}
