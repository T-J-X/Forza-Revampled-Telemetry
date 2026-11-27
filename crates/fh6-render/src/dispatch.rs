//! Dispatch logic for camera view routing and theme resolution.
use fh6_core::{AppConfig, ClusterTheme};

/// The theme a config style name selects: any theme's own name (case-insensitive) or a short
/// alias, as [`ClusterTheme`]'s `FromStr` parses it; unknown names select `BoschDdu`.
/// Allocation-free.
pub fn style_to_theme(s: &str) -> ClusterTheme {
    s.parse().unwrap_or(ClusterTheme::BoschDdu)
}

pub struct DispatchContext {
    pub camera_index: i32,
    pub use_scrim: bool,
    pub theme: ClusterTheme,
}

pub fn resolve_dispatch(config: &AppConfig) -> DispatchContext {
    let cam = config.current_camera_index;
    let is_cockpit = cam == 3;
    let is_bonnet = cam == 4;

    let style_name = if is_bonnet {
        config.bonnet_hud_style.trim()
    } else if is_cockpit {
        if !config.cockpit_hud_style.trim().is_empty() {
            config.cockpit_hud_style.as_str()
        } else if !config.hud_style.trim().is_empty() {
            config.hud_style.as_str()
        } else {
            "BoschDdu"
        }
    } else {
        "BoschDdu"
    };

    let theme = style_to_theme(style_name);
    let use_scrim = is_cockpit && config.oem_dash_opaque_pct > 0;

    DispatchContext {
        camera_index: cam,
        use_scrim,
        theme,
    }
}

pub fn should_draw_scrim(ctx: &DispatchContext, config: &AppConfig) -> bool {
    ctx.camera_index == 3 && config.oem_dash_opaque_pct > 0
}

pub fn bonnet_scrim_forced_zero(ctx: &DispatchContext) -> bool {
    ctx.camera_index == 4
}

#[cfg(test)]
mod tests {
    use super::*;
    use fh6_core::AppConfig;

    #[test]
    fn cockpit_routing_uses_cockpit_hud_style() {
        let cfg = AppConfig {
            current_camera_index: 3,
            cockpit_hud_style: "LiquidGlass".into(),
            oem_dash_opaque_pct: 50,
            ..Default::default()
        };
        let ctx = resolve_dispatch(&cfg);
        assert_eq!(ctx.theme, ClusterTheme::LiquidGlass);
        assert!(ctx.use_scrim);
        assert!(should_draw_scrim(&ctx, &cfg));
    }

    #[test]
    fn bonnet_routing_forces_scrim_zero() {
        let cfg = AppConfig {
            current_camera_index: 4,
            bonnet_hud_style: "Gt7HoodPro".into(),
            oem_dash_opaque_pct: 80,
            ..Default::default()
        };
        let ctx = resolve_dispatch(&cfg);
        assert_eq!(ctx.theme, ClusterTheme::Gt7HoodPro);
        assert!(!ctx.use_scrim);
        assert!(bonnet_scrim_forced_zero(&ctx));
        assert!(!should_draw_scrim(&ctx, &cfg));
    }

    #[test]
    fn fallback_to_bosch() {
        let cfg = AppConfig {
            current_camera_index: 3,
            ..Default::default()
        };
        let ctx = resolve_dispatch(&cfg);
        assert_eq!(ctx.theme, ClusterTheme::BoschDdu);
    }
}
