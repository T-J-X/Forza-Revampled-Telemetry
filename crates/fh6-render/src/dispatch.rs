//! Dispatch logic for camera view routing and theme resolution.
use fh6_core::{AppConfig, ClusterTheme};

/// Short and legacy names accepted besides each theme's own name.
const ALIASES: &[(&str, ClusterTheme)] = &[
    ("bosch", ClusterTheme::BoschDdu),
    ("drift", ClusterTheme::DriftGlass),
    ("porsche", ClusterTheme::PorscheAnalog),
    ("analog", ClusterTheme::PorscheAnalog),
    ("windshield", ClusterTheme::WindshieldHud),
    ("tesla", ClusterTheme::TeslaAdas),
    ("retro", ClusterTheme::RetroCyber),
    ("gt7hood", ClusterTheme::Gt7HoodPro),
    ("hood", ClusterTheme::Gt7HoodPro),
    ("amg", ClusterTheme::AmgMbuxSuperscreen),
    ("mbux", ClusterTheme::AmgMbuxSuperscreen),
    ("lfa", ClusterTheme::LfaV10Nurburgring),
    ("pagani horologerie", ClusterTheme::PaganiHorologerie),
    ("pagani", ClusterTheme::PaganiHorologerie),
    ("f35helmet hmd", ClusterTheme::F35HelmetHmd),
    ("f35", ClusterTheme::F35HelmetHmd),
    ("helmet", ClusterTheme::F35HelmetHmd),
    ("bugatti", ClusterTheme::BugattiTourbillon),
    ("eva", ClusterTheme::EvaBioSync),
    ("koenigsegg", ClusterTheme::KoenigseggGyro),
    ("valkyrie", ClusterTheme::ValkyrieLmp1),
    ("synthwave", ClusterTheme::SynthwaveOutrun),
    ("outrun", ClusterTheme::SynthwaveOutrun),
    ("chrono", ClusterTheme::ChronoMultiverse),
];

/// The theme a config style name selects: any theme's own name (case-insensitive) or a short
/// alias; unknown names select `BoschDdu`. Allocation-free.
pub fn style_to_theme(s: &str) -> ClusterTheme {
    let name = s.trim();
    if let Some(t) = ClusterTheme::all().iter().find(|t| t.name().eq_ignore_ascii_case(name)) {
        return *t;
    }
    ALIASES
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
        .map_or(ClusterTheme::BoschDdu, |(_, t)| *t)
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
