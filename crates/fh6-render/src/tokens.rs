use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct DesignTokens {
    pub colors: Option<HashMap<String, String>>,
    pub grid: Option<f32>,
}

impl Default for DesignTokens {
    fn default() -> Self {
        Self {
            colors: None,
            grid: Some(8.0),
        }
    }
}

pub fn load_tokens(path: &str) -> DesignTokens {
    if let Ok(s) = std::fs::read_to_string(path) {
        if let Ok(t) = serde_json::from_str::<DesignTokens>(&s) {
            return t;
        }
    }
    DesignTokens::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_tokens_fallback() {
        let t = load_tokens("nonexistent.json");
        assert_eq!(t.grid.unwrap_or(0.0), 8.0);
    }
}
