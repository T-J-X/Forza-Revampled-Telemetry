//! Loads the real `data/config.json` and checks the values the overlay depends on.
use fh6_core::config::AppConfig;
use fh6_core::config_watch::ConfigWatcher;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn data_config() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/config.json")
}

#[test]
fn data_config_loads_real_values() {
    let cfg = AppConfig::try_load(data_config()).expect("data/config.json parses");
    assert_eq!(cfg.version, "6.0.0");
    assert_eq!((cfg.cockpit_anchor_x, cfg.cockpit_anchor_y), (1920, 1835));
    assert_eq!((cfg.bonnet_anchor_x, cfg.bonnet_anchor_y), (1920, 1760));
    assert_eq!(cfg.cockpit_scale, 1.45);
    assert_eq!(cfg.bonnet_scale, 1.55);
    assert_eq!(cfg.current_camera_index, 3);
    assert_eq!(cfg.cockpit_hud_style, "BoschDdu");
    assert_eq!(cfg.bonnet_hud_style, "Gt7HoodPro");
    assert_eq!(cfg.oem_dash_opaque_pct, 100);

    // The all-caps acronym keys that used to load as 0.
    assert_eq!(cfg.wheel_less_cockpit_fov, 80.0);
    assert_eq!(cfg.standard_cockpit_fov, 80.0);
    assert_eq!(cfg.bonnet_fov, 95.0);
    assert_eq!(cfg.chase_near_fov, 62.0);
    assert_eq!(cfg.chase_far_fov, 68.0);
    assert_eq!(cfg.bumper_fov, 78.0);
    assert!(cfg.show_bonnet_gt_widgets);

    // Keys absent from the file take defaults, not zero.
    assert_eq!(cfg.cluster_scale, 1.0);
    assert_eq!(cfg.osd_toggle_mask, 0x0020);
    assert_eq!(cfg.osd_hold_ms, 400);
    assert_eq!(cfg.driver_seat_height_m, -0.045);

    let ports = cfg.telemetry_ports();
    assert_eq!(
        (ports.primary, ports.fallback, ports.forward),
        (8000, Some(5500), Some(5607))
    );

    // "PrimaryPort" is not a field; it must survive a save.
    assert!(cfg.extra.contains_key("PrimaryPort"));
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
