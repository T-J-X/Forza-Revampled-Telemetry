use std::sync::OnceLock;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CarBrand { pub hex: String }

pub static CAR_BRANDS: OnceLock<serde_json::Value> = OnceLock::new();

/// Manufacturer colour table, embedded at build time so rendering never depends on a file path.
static CAR_BRANDS_JSON: &str = include_str!("../../../../assets/car_brands/car_brands.json");

pub fn load_car_brands() -> &'static serde_json::Value {
    CAR_BRANDS.get_or_init(|| {
        serde_json::from_str(CAR_BRANDS_JSON).unwrap_or(serde_json::Value::Object(Default::default()))
    })
}
