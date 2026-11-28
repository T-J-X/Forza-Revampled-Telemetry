//! Loads the real `data/config.json` and checks the values the overlay depends on.
use fh6_core::config::{AppConfig, PowerUnit, SpeedUnit, CONFIG_VERSION, RETIRED_KEYS};
use fh6_core::config_watch::ConfigWatcher;
use fh6_core::ClusterTheme;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn data_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/config.json")
}

/// `data/config.json` as shipped before 7.0.0, byte for byte.
const SHIPPED_6_0_0: &str = r##"{"Version":"6.0.0","CockpitAnchorX":1920,"CockpitAnchorY":1835,"CockpitScale":1.45,"BonnetAnchorX":1920,"BonnetAnchorY":1760,"BonnetScale":1.55,"CurrentCameraIndex":3,"CockpitStyle":"Digital","BonnetStyle":"Digital","HudStyle":"LiquidGlass","CockpitHudStyle":"BoschDdu","BonnetHudStyle":"Gt7HoodPro","OemDashOpaquePct":100,"OemDashBinnacleOffsetY":0,"OemDashWidthScale":1,"OsdAutoPauseGame":true,"OsdLiveDynoPreviewWhenPaused":true,"DriftSyncMode":"Off","DriftSyncSensitivity":1,"DriftSyncSmoothing":0.2,"DriftSyncMaxOffsetPx":400,"LockVisibleWhileDriving":false,"ShiftLightPattern":"Converging","ShiftStartPct":0.72,"RedlinePct":0.94,"ControllerCameraButton":"RB","SpeedUnit":"MPH","ShowBonnetGTWidgets":true,"DemoMode":false,"PerCarProfileEnabled":true,"OledPixelOrbitEnabled":true,"OledIdleDimmerEnabled":true,"OledDimmerMinAlpha":0.35,"ManualShiftGuardEnabled":true,"NightGlowMode":"Off","NightGlowIntensity":0.35,"UdpPort":8000,"FallbackPort":5500,"ForwardPort":5607,"PrimaryPort":8000,"ColorMode":"AutoBrand","CustomBrandKey":"DEFAULT","PrimaryHex":"#00D2FF","SecondaryHex":"#FFB300","AccentHex":"#00E5FF","ShiftFlashHex":"#FFFFFF","UseWidgetColorOverrides":false,"SpeedColorHex":"#FFFFFF","GearColorHex":"#FFFFFF","GForceColorHex":"#00E5FF","SuspensionColorHex":"#76FF03","WheelLessCockpitFOV":80,"StandardCockpitFOV":80,"BonnetFOV":95,"ChaseNearFOV":62,"ChaseFarFOV":68,"BumperFOV":78,"CockpitOffsetX":0,"CockpitOffsetY":-0.045,"CockpitOffsetZ":-0.5,"HoodOffsetY":0,"HoodOffsetZ":0,"HoodPitchOffset":0.04}"##;

/// A file the Rust build saved before 7.0.0: its default config as written by the OSD (every
/// struct field, the typed style copies, the CJK default name, `PowerUnit` "kW" with MPH), plus the
/// seven OSD settings that lived in the untyped map, set away from their defaults.
const SAVED_BY_RUST_6_0_0: &str = r##"{"Version":"6.0.0","CockpitAnchorX":1920,"CockpitAnchorY":1835,"CockpitScale":1.45,"BonnetAnchorX":1920,"BonnetAnchorY":1760,"BonnetScale":1.55,"CurrentCameraIndex":3,"CockpitStyle":"Digital","BonnetStyle":"Digital","HudStyle":"LiquidGlass","CockpitHudStyle":"BoschDdu","BonnetHudStyle":"Gt7HoodPro","OemDashOpaquePct":100,"OemDashBinnacleOffsetY":0,"OemDashWidthScale":1.0,"OsdAutoPauseGame":true,"OsdLiveDynoPreviewWhenPaused":true,"DriftSyncMode":"Off","DriftSyncSensitivity":1.0,"DriftSyncSmoothing":0.2,"DriftSyncMaxOffsetPx":400.0,"LockVisibleWhileDriving":false,"ShiftLightPattern":"Converging","ShiftStartPct":0.72,"RedlinePct":0.94,"ControllerCameraButton":"RB","SpeedUnit":"MPH","ShowBonnetGTWidgets":true,"DemoMode":false,"PerCarProfileEnabled":true,"OledPixelOrbitEnabled":true,"OledIdleDimmerEnabled":true,"OledDimmerMinAlpha":0.35,"ManualShiftGuardEnabled":true,"CustomDriverName":"ØG『死』ŦJ乂","DriverTagGlitchFx":false,"PowerUnit":"kW","RedlineStrobe":false,"NightGlowMode":"Off","NightGlowIntensity":0.35,"NightAmbientGlowEnabled":false,"NightAmbientGlowIntensityPct":0,"AdasBackdropMode":0,"CenterConsoleMapMode":0,"CenterConsoleInfotainmentMode":0,"RouteActive":false,"OemDashDimmerEnabled":false,"ModularSidePodsEnabled":false,"SteeringHubCapEnabled":false,"UdpPort":8000,"FallbackPort":5500,"ForwardPort":5607,"UdpListenAllInterfaces":false,"ColorMode":"AutoBrand","CustomBrandKey":"DEFAULT","PrimaryHex":"#00D2FF","SecondaryHex":"#FFB300","AccentHex":"#00E5FF","ShiftFlashHex":"#FFFFFF","UseWidgetColorOverrides":false,"SpeedColorHex":"#FFFFFF","GearColorHex":"#FFFFFF","GForceColorHex":"#00E5FF","SuspensionColorHex":"#76FF03","WheelLessCockpitFOV":80.0,"StandardCockpitFOV":80.0,"BonnetFOV":95.0,"ChaseNearFOV":62.0,"ChaseFarFOV":68.0,"BumperFOV":78.0,"CockpitOffsetX":0.0,"CockpitOffsetY":-0.045,"CockpitOffsetZ":-0.5,"HoodOffsetY":0.0,"HoodOffsetZ":0.0,"HoodPitchOffset":0.04,"ActiveCameraView":"Cockpit","CockpitHudStyleTyped":"LiquidGlass","BonnetHudStyleTyped":"Gt7HoodPro","ChaseHudStyleTyped":"BoschDdu","DriverSeatHeightM":-0.045,"SteeringWheelOffsetYM":-0.035,"FovDegrees":65,"ClusterScale":1.0,"FontPreset":"BahnschriftLight","AmbientGlowEnabled":true,"AmbientGlowIntensity":0.25,"AmbientCowlFeather":0.4,"OsdToggleMask":32,"OsdHoldMs":400,"GuideButtonOpensWidget":true,"OemDashOffsetX":0,"OemDashHeightScale":1.0,"OemDashTopArch":0.7,"OemDashCornerSoftness":0.7,"OemDashFeatherPx":96.0,"OemDashColumnVoidPx":220.0,"ChaseAnchorX":1920,"ChaseAnchorY":1900,"CockpitHeightOffsetPx":0.0,"BonnetHeightOffsetPx":0.0,"ChaseHeightOffsetPx":0.0,"CockpitClusterScale":1.0,"BonnetClusterScale":1.0,"ChaseClusterScale":0.8,"CockpitSafeZone":{"SidePx":160.0,"TopPx":0.0,"BottomPx":40.0},"BonnetSafeZone":{"SidePx":160.0,"TopPx":0.0,"BottomPx":40.0},"ChaseSafeZone":{"SidePx":160.0,"TopPx":0.0,"BottomPx":40.0},"ChaseHudEnabled":true,"ChaseHudStyle":"","ResyncCameraIndex":3,"ResyncHoldMs":1000,"CameraPatchAuto":false,"GameInstallDir":"","ShowLogo":false,"ShowStats":true,"ReducedFlash":true,"GlassRefraction":true,"ClusterFont":"Saira","PaletteSeed":4242,"ShiftPointPct":0.96}"##;

#[test]
fn data_config_loads_real_values() {
    let cfg = AppConfig::try_load(data_config()).expect("data/config.json parses");
    assert_eq!(cfg.version, CONFIG_VERSION);
    assert_eq!((cfg.cockpit_anchor_x, cfg.cockpit_anchor_y), (1920, 1835));
    assert_eq!((cfg.bonnet_anchor_x, cfg.bonnet_anchor_y), (1920, 1760));
    assert_eq!(cfg.current_camera_index, 3);
    assert_eq!(cfg.cockpit_hud_style, ClusterTheme::BoschDdu);
    assert_eq!(cfg.bonnet_hud_style, ClusterTheme::Gt7HoodPro);
    assert_eq!(cfg.oem_dash_opaque_pct, 100);
    assert_eq!((cfg.speed_unit, cfg.power_unit), (SpeedUnit::Mph, PowerUnit::Hp));

    // The all-caps acronym keys that used to load as 0.
    assert_eq!(cfg.wheel_less_cockpit_fov, 80.0);
    assert_eq!(cfg.standard_cockpit_fov, 80.0);
    assert_eq!(cfg.bonnet_fov, 95.0);
    assert_eq!(cfg.chase_near_fov, 62.0);
    assert_eq!(cfg.chase_far_fov, 68.0);
    assert_eq!(cfg.bumper_fov, 78.0);

    assert_eq!(cfg.cluster_scale, 1.0);
    assert_eq!(cfg.osd_toggle_mask, 0x0020);
    assert_eq!(cfg.osd_hold_ms, 400);
    assert_eq!(cfg.driver_seat_height_m, -0.045);

    let ports = cfg.telemetry_ports();
    assert_eq!(
        (ports.primary, ports.fallback, ports.forward),
        (8000, Some(5500), Some(5607))
    );
    assert!(cfg.extra.is_empty(), "every shipped key is a setting: {:?}", cfg.extra);
}

#[test]
fn data_config_is_what_a_save_writes() {
    // Git may check the file out with CRLF line endings on Windows.
    let text = std::fs::read_to_string(data_config())
        .expect("data/config.json reads")
        .replace("\r\n", "\n");
    let cfg = AppConfig::from_json_str(&text).expect("data/config.json parses");
    let saved = serde_json::to_string_pretty(&cfg).expect("serialises");
    assert_eq!(
        text.trim_end(),
        saved,
        "data/config.json is not in the form AppConfig::save writes"
    );
}

#[test]
fn the_config_shipped_before_7_loads_as_the_one_shipped_now() {
    let old = AppConfig::from_json_str(SHIPPED_6_0_0).expect("the 6.0.0 file parses");
    let now = AppConfig::try_load(data_config()).expect("data/config.json parses");
    assert_eq!(old, now);
    let saved = serde_json::to_string(&old).expect("serialises");
    for key in RETIRED_KEYS {
        assert!(!saved.contains(&format!("\"{key}\"")), "{key} survived a save");
    }
}

#[test]
fn a_file_the_rust_build_saved_before_7_loads_with_its_osd_settings() {
    let cfg = AppConfig::from_json_str(SAVED_BY_RUST_6_0_0).expect("the old Rust file parses");
    let expected = AppConfig {
        show_logo: false,
        show_stats: true,
        reduced_flash: true,
        glass_refraction: true,
        cluster_font: Some("Saira".into()),
        palette_seed: Some(4242),
        shift_point_pct: Some(0.96),
        ..AppConfig::default()
    };
    assert_eq!(cfg, expected, "only the OSD settings differ from the defaults");
    let saved = serde_json::to_string(&cfg).expect("serialises");
    for key in RETIRED_KEYS {
        assert!(!saved.contains(&format!("\"{key}\"")), "{key} survived a save");
    }
    assert_eq!(AppConfig::from_json_str(&saved).expect("reloads"), cfg);
}

#[test]
fn save_round_trips_without_bom() {
    let cfg = AppConfig::try_load(data_config()).expect("data/config.json parses");
    let out = std::env::temp_dir().join(format!("fh6_cfg_roundtrip_{}.json", std::process::id()));
    cfg.save(&out).expect("save");
    let bytes = std::fs::read(&out).expect("read back");
    assert_ne!(&bytes[..3], b"\xEF\xBB\xBF", "no UTF-8 BOM");
    let again = AppConfig::try_load(&out).expect("reload");
    assert_eq!(again, cfg);
    std::fs::remove_file(out).ok();
}

#[test]
fn save_strips_manual_gearbox_buttons_from_the_mask() {
    // A/B/X/Y (0xF000) are the manual gearbox; they must never be written to disk.
    let cfg = AppConfig {
        osd_toggle_mask: 0x1000 | 0x8000 | 0x0020,
        ..AppConfig::default()
    };
    let out = std::env::temp_dir().join(format!("fh6_cfg_mask_{}.json", std::process::id()));
    cfg.save(&out).expect("save");
    // Read the raw file: loading would sanitise again and hide a missing save-side strip.
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out).expect("read back")).expect("json");
    assert_eq!(raw["OsdToggleMask"], 0x0020);
    std::fs::remove_file(out).ok();
}

#[test]
fn watcher_hot_reloads_on_change() {
    let path = std::env::temp_dir().join(format!("fh6_cfg_watch_{}.json", std::process::id()));
    std::fs::write(&path, r#"{"CockpitAnchorY":1835}"#).expect("write initial");

    let changes = Arc::new(AtomicUsize::new(0));
    let notify = {
        let changes = Arc::clone(&changes);
        Arc::new(move || {
            changes.fetch_add(1, Ordering::SeqCst);
        })
    };
    let watcher = ConfigWatcher::spawn(path.clone(), Duration::from_millis(20), notify).expect("spawn watcher");
    assert_eq!(watcher.current().cockpit_anchor_y, 1835);

    // A half-written file must not replace the good config.
    std::fs::write(&path, r#"{"CockpitAnchorY":"#).expect("write broken");
    std::thread::sleep(Duration::from_millis(120));
    assert_eq!(watcher.current().cockpit_anchor_y, 1835);

    std::fs::write(&path, r#"{"CockpitAnchorY":1700, "UdpPort": 8001}"#).expect("write update");
    let deadline = Instant::now() + Duration::from_secs(3);
    while watcher.current().cockpit_anchor_y != 1700 && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(watcher.current().cockpit_anchor_y, 1700);
    assert_eq!(watcher.current().udp_port, 8001);
    assert!(changes.load(Ordering::SeqCst) >= 1);

    drop(watcher);
    std::fs::remove_file(path).ok();
}
