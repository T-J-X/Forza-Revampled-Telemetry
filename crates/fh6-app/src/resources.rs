//! Files the render kit needs at startup: design tokens, the car/brand palette database, the
//! `assets/` folder (Phase 6 fonts and textures) and the user's `fonts/` folder beside the exe.
use fh6_core::CarDatabase;
use fh6_kit::Tokens;
use std::path::{Path, PathBuf};

/// Tokens file name (in `data/` in the repo, beside `config.json` when deployed).
const TOKENS_FILE: &str = "figma_tokens.json";
/// Marker file that identifies an `assets/` folder.
const BRANDS_FILE: &str = "car_brands/car_brands.json";

/// Startup resources for the kit.
#[derive(Debug, Clone)]
pub struct KitResources {
    /// Bound design tokens (defaults when the file is missing or invalid).
    pub tokens: Tokens,
    /// Brand palettes by car ordinal.
    pub cars: CarDatabase,
    /// The `assets/` folder, when found.
    pub assets: Option<PathBuf>,
    /// The user's `fonts/` folder beside the exe, when it exists.
    pub user_fonts: Option<PathBuf>,
}

impl KitResources {
    /// Resolves everything relative to the config file, the exe folder and the working folder.
    #[must_use]
    pub fn resolve(config_path: &Path) -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf));
        let cwd = std::env::current_dir().ok();
        let starts: Vec<PathBuf> = [config_path.parent().map(Path::to_path_buf), exe_dir.clone(), cwd]
            .into_iter()
            .flatten()
            .collect();
        let tokens_path = find_upwards(&starts, |d| {
            [d.join(TOKENS_FILE), d.join("data").join(TOKENS_FILE)]
                .into_iter()
                .find(|p| p.is_file())
        });
        let tokens = match tokens_path.as_deref().map(Tokens::load) {
            Some(Ok(t)) => {
                tracing::info!(version = %t.version, "design tokens loaded");
                t
            }
            Some(Err(err)) => {
                tracing::warn!(error = %err, "design tokens unreadable; using built-in defaults");
                Tokens::default()
            }
            None => {
                tracing::warn!("figma_tokens.json not found; using built-in defaults");
                Tokens::default()
            }
        };
        let assets = find_upwards(&starts, |d| {
            let a = d.join("assets");
            a.join(BRANDS_FILE).is_file().then_some(a)
        });
        let cars = assets
            .as_ref()
            .map(|a| CarDatabase::load(a.join(BRANDS_FILE)))
            .unwrap_or_default();
        let user_fonts = exe_dir.map(|d| d.join("fonts")).filter(|d| d.is_dir());
        Self {
            tokens,
            cars,
            assets,
            user_fonts,
        }
    }
}

fn find_upwards(starts: &[PathBuf], probe: impl Fn(&Path) -> Option<PathBuf>) -> Option<PathBuf> {
    starts.iter().flat_map(|s| s.ancestors()).find_map(probe)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_resources_resolve_from_the_data_folder() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let r = KitResources::resolve(&repo.join("data/config.json"));
        assert!(r.assets.is_some(), "assets/ found from data/config.json");
        assert!(!r.cars.brand_palettes.is_empty());
        assert!(!r.tokens.family_defaults.is_empty());
    }
}
