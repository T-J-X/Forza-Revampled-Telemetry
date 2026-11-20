//! Dispatch logic for camera view routing and theme resolution.
use fh6_core::{AppConfig, ClusterTheme};

fn style_to_theme(s: &str) -> ClusterTheme {
    let name = s.trim().to_lowercase();
    match name.as_str() {
        "boschddu" | "bosch" => ClusterTheme::BoschDdu,
        "driftglass" | "drift" => ClusterTheme::DriftGlass,
        "porscheanalog" | "porsche" | "analog" => ClusterTheme::PorscheAnalog,
        "liquidglass" => ClusterTheme::LiquidGlass,
        "windshieldhud" | "windshield" => ClusterTheme::WindshieldHud,
        "teslaadas" | "tesla" => ClusterTheme::TeslaAdas,
        "retrocyber" | "retro" => ClusterTheme::RetroCyber,
        "gt7hoodpro" | "gt7hood" | "hood" => ClusterTheme::Gt7HoodPro,
        "amgmbuxsuperscreen" | "amg" | "mbux" => ClusterTheme::AmgMbuxSuperscreen,
        "lfav10nurburgring" | "lfa" => ClusterTheme::LfaV10Nurburgring,
        "pagani horologerie" | "pagani" => ClusterTheme::PaganiHorologerie,
        "f35helmet hmd" | "f35" | "helmet" => ClusterTheme::F35HelmetHmd,
        "bugattitourbillon" | "bugatti" => ClusterTheme::BugattiTourbillon,
        "evabiosync" | "eva" => ClusterTheme::EvaBioSync,
        "koenigsegggyro" | "koenigsegg" => ClusterTheme::KoenigseggGyro,
        "valkyrielmp1" | "valkyrie" => ClusterTheme::ValkyrieLmp1,
        "synthwaveoutrun" | "synthwave" | "outrun" => ClusterTheme::SynthwaveOutrun,
        "chronomultiverse" | "chrono" => ClusterTheme::ChronoMultiverse,
        _ => ClusterTheme::BoschDdu,
    }
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
