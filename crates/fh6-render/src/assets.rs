use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

static CACHE: LazyLock<Mutex<HashMap<String, Vec<u8>>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

pub struct AssetCache;

impl AssetCache {
    pub fn get_or_load(relative_path: &str) -> Option<Vec<u8>> {
        if relative_path.is_empty() {
            return None;
        }
        let mut cache = CACHE.lock().unwrap();
        if let Some(data) = cache.get(relative_path) {
            return Some(data.clone());
        }
        if let Ok(data) = std::fs::read(relative_path) {
            cache.insert(relative_path.to_string(), data.clone());
            return Some(data);
        }
        None
    }

    pub fn set(relative_path: &str, data: Vec<u8>) {
        if relative_path.is_empty() {
            return;
        }
        let mut cache = CACHE.lock().unwrap();
        cache.insert(relative_path.to_string(), data);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn asset_cache_empty_path() {
        assert!(AssetCache::get_or_load("").is_none());
    }
}
