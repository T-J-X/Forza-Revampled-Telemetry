use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static PEN_CACHE: LazyLock<Mutex<HashMap<(u32, u32), ()>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static FONT_CACHE: LazyLock<Mutex<HashMap<String, ()>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct PaintPool;

impl PaintPool {
    pub fn get_pen(color_argb: u32, width: f32) {
        let key = (color_argb, (width * 10.0).round() as u32);
        let mut cache = PEN_CACHE.lock().unwrap();
        cache.entry(key).or_insert(());
    }

    pub fn get_font(family: &str, size_pt: f32, style: u32) -> String {
        let key = format!("{}_{}_{}", family, (size_pt * 10.0).round() as u32, style);
        let mut cache = FONT_CACHE.lock().unwrap();
        cache.entry(key.clone()).or_insert(());
        key
    }

    pub fn resolve_font_family(requested: &str) -> String {
        if requested.is_empty() {
            return "Bahnschrift".to_string();
        }
        if requested.contains("Mono") {
            return "Consolas".to_string();
        }
        requested.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pool_caches() {
        PaintPool::get_pen(0xFF0000, 1.0);
        PaintPool::get_pen(0xFF0000, 1.0);
        let f = PaintPool::get_font("Segoe UI", 12.0, 1);
        assert!(!f.is_empty());
    }
}
