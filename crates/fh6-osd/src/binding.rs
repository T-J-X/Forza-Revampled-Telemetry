//! Two-way binding between OSD UI state and AppConfig, with atomic disk persistence.
use crate::model::{OsdNavAction, UnifiedOsdState};
use fh6_core::config::AppConfig;
use std::path::{Path, PathBuf};

pub struct OsdConfigBinding {
    pub config: AppConfig,
    pub path: Option<PathBuf>,
    pub is_dirty: bool,
    pub auto_save_on_adjust: bool,
}

impl OsdConfigBinding {
    pub fn new(config: AppConfig, path: Option<PathBuf>) -> Self {
        Self {
            config,
            path,
            is_dirty: false,
            auto_save_on_adjust: false,
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let config = AppConfig::load(&path_buf);
        Self::new(config, Some(path_buf))
    }

    /// Handles a navigation action, mutating the state and config.
    /// If an adjustment was made, marks config as dirty and optionally saves.
    pub fn handle_action(&mut self, state: &mut UnifiedOsdState, action: OsdNavAction) -> std::io::Result<bool> {
        let is_adjust = matches!(action, OsdNavAction::AdjustLeft | OsdNavAction::AdjustRight);

        state.handle_action(action, &mut self.config);

        if is_adjust {
            self.is_dirty = true;
            if self.auto_save_on_adjust {
                self.save()?;
            }
            return Ok(true);
        }

        if matches!(action, OsdNavAction::Close) && self.is_dirty {
            self.save()?;
        }

        Ok(false)
    }

    /// Persists current config to disk atomically without UTF-8 BOM.
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(ref path) = self.path {
            self.config.save(path)?;
            self.is_dirty = false;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_way_binding_and_persistence() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join(format!("fh6_osd_test_{}.json", std::process::id()));

        // Initial config
        let initial = AppConfig {
            fov_degrees: 65,
            oem_dash_opaque_pct: 50,
            ..Default::default()
        };
        initial.save(&config_path).expect("save initial");

        let mut binding = OsdConfigBinding::load_from_file(&config_path);
        let mut state = UnifiedOsdState::new();

        // 1. Adjust FOV in Tab 0, Row 0
        state.selected_row = 0;
        binding
            .handle_action(&mut state, OsdNavAction::AdjustRight)
            .expect("handle adjust right");
        assert_eq!(binding.config.fov_degrees, 66);
        assert!(binding.is_dirty);

        // 2. Switch to Tab 1 (CLUSTERS) and adjust Cluster Scale
        binding
            .handle_action(&mut state, OsdNavAction::TabNext)
            .expect("next tab");
        assert_eq!(state.current_tab, 1);
        assert_eq!(state.selected_row, 0);

        // Adjust cluster scale row 3
        state.selected_row = 3;
        binding
            .handle_action(&mut state, OsdNavAction::AdjustRight)
            .expect("adjust scale");
        assert_eq!(binding.config.cluster_scale, 1.02);

        // 3. Save changes
        binding.save().expect("save binding");
        assert!(!binding.is_dirty);

        // 4. Reload from disk and verify byte-level persistence
        let reloaded = AppConfig::load(&config_path);
        assert_eq!(reloaded.fov_degrees, 66);
        assert_eq!(reloaded.cluster_scale, 1.02);

        // Verify UTF-8 no BOM
        let bytes = std::fs::read(&config_path).expect("read bytes");
        assert!(
            bytes.len() < 3 || bytes[0..3] != [0xEF, 0xBB, 0xBF],
            "Config must NOT contain a UTF-8 BOM"
        );

        let _ = std::fs::remove_file(&config_path);
    }
}
