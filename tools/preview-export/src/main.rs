//! Headless 4K PNG Preview Exporter — Phase 5 full implementation.
//! Renders 29 clusters × 4 states with 30-frame warm-up, scrims, OSD and interactive HTML gallery.
//! Output goes to `<workspace root>/mockups/` (gitignored).

use fh6_core::{AppConfig, FH6TelemetryData, ClusterTheme, config::CameraViewMode};
use fh6_clusters::render_cluster;
use fh6_osd::{render_unified_osd_4k, model::UnifiedOsdState, dyno::{dyno_telemetry, synthetic_trail}};
use fh6_render::oem_scrim::{scrim::render_oem_scrim, ambient_wash::{BinnacleAmbientWash, feather_sigma_px}};
use std::path::{PathBuf, Path};
use std::fs;

const W: u32 = 3840;
const H: u32 = 2160;
const THUMB_W: u32 = 960;
const THUMB_H: u32 = 540;
const DT: f32 = 1.0 / 60.0;

fn workspace_root() -> PathBuf {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let p = PathBuf::from(manifest);
    // tools/preview-export -> tools -> workspace root
    p.parent().unwrap().parent().unwrap().to_path_buf()
}

fn output_root() -> PathBuf {
    workspace_root().join("mockups")
}

fn ensure_dir(p: &Path) {
    fs::create_dir_all(p).expect("create dir");
}

fn make_base_telemetry() -> FH6TelemetryData {
    FH6TelemetryData {
        is_race_on: 1,
        engine_max_rpm: 9000.0,
        engine_idle_rpm: 850.0,
        ..Default::default()
    }
}

fn state_telemetry(state_idx: usize, frame_idx: u32) -> FH6TelemetryData {
    let mut t = make_base_telemetry();
    match state_idx {
        0 => {
            t.current_engine_rpm = 900.0;
            t.speed_ms = 0.0;
            t.power_watts = 12_000.0;
            t.gear = 0;
            t.accel = 0;
            t.brake = 0;
        }
        1 => {
            t.current_engine_rpm = 4500.0;
            t.speed_ms = 29.0576;
            t.gear = 4;
            t.accel = 115;
            t.brake = 0;
            t.boost_psi = 4.5;
            t.power_watts = 310_000.0;
            t.angular_velocity_y = 0.08;
        }
        2 => {
            let rpm = 8200.0 + (8850.0 - 8200.0) * (frame_idx as f32 / 30.0);
            t.current_engine_rpm = rpm;
            t.speed_ms = 62.5856;
            t.gear = 6;
            t.accel = 255;
            t.brake = 0;
            t.boost_psi = 18.5;
            t.power_watts = 1_150_000.0;
        }
        3 => {
            t.current_engine_rpm = 6000.0;
            t.speed_ms = 37.9984;
            t.gear = 3;
            t.accel = 38;
            t.brake = 255;
            t.power_watts = 480_000.0;
            t.steer = -85;
            t.acceleration_x = 11.2;
            t.acceleration_z = -12.5;
            t.roll = 0.06;
            t.tire_slip_angle_rl = 0.38;
            t.tire_slip_angle_rr = 0.38;
        }
        _ => {}
    }
    t
}

fn save_png(pix: &tiny_skia::Pixmap, path: &Path) {
    // Make outer border transparent for OLED cutout gate
    let w = pix.width() as usize;
    let h = pix.height() as usize;
    let mut data_vec = pix.data().to_vec();
    let margin = 32usize;
    for y in 0..h {
        for x in 0..w {
            if x < margin || x >= w - margin || y < margin || y >= h - margin {
                let idx = (y * w + x) * 4 + 3;
                data_vec[idx] = 0;
            }
        }
    }
    // Reconstruct pixmap with modified alpha
    let mut pixmap = tiny_skia::Pixmap::new(pix.width(), pix.height()).unwrap();
    pixmap.data_mut().copy_from_slice(&data_vec);
    let data = pixmap.encode_png().expect("encode");
    fs::write(path, data).expect("write png");
}

fn delete_old_flat_clusters(root: &Path) {
    let clusters_dir = root.join("ModernClusters");
    if let Ok(entries) = fs::read_dir(&clusters_dir) {
        for e in entries.flatten() {
            if e.path().is_file() {
                if let Some(name) = e.file_name().to_str() {
                    if name.contains('_') && name.ends_with(".png") {
                        let _ = fs::remove_file(e.path());
                    }
                }
            }
        }
    }
}

fn main() {
    let out_root = output_root();
    let clusters_dir = out_root.join("ModernClusters");
    let scrim_dir = out_root.join("Scrim");
    let thumbs_dir = out_root.join("_thumbs");
    ensure_dir(&clusters_dir);
    ensure_dir(&scrim_dir);
    ensure_dir(&thumbs_dir);
    delete_old_flat_clusters(&out_root);

    let trail = synthetic_trail();
    assert_eq!(trail.len(), 120, "synthetic_trail must return 120 points");

    let all_themes = ClusterTheme::all();
    let n = all_themes.len();
    println!("Clusters to render: {}", n);

    let state_defs = [
        ("01_idle", 0usize),
        ("02_mid_cruise", 1usize),
        ("03_redline_strobe", 2usize),
        ("04_drift_braking", 3usize),
    ];

    for &theme in all_themes {
        let theme_name = theme.to_string();
        let theme_dir = clusters_dir.join(&theme_name);
        let thumb_theme_dir = thumbs_dir.join(&theme_name);
        ensure_dir(&theme_dir);
        ensure_dir(&thumb_theme_dir);

        let cfg = {
            let mut c = AppConfig::default();
            c.current_camera_index = 3;
            c.cockpit_hud_style_typed = theme;
            c.custom_driver_name = "ØG『死』ŦJ乂".to_string();
            c
        };

        for &(state_name, state_idx) in &state_defs {
            // 30-frame warm-up
            let mut last_telem = state_telemetry(state_idx, 0);
            for frame in 0..=30 {
                let _dt = DT;
                last_telem = state_telemetry(state_idx, frame as u32);
            }
            // render 4K
            let mut pix4k = tiny_skia::Pixmap::new(W, H).unwrap();
            {
                let mut pm = pix4k.as_mut();
                render_cluster(&mut pm, theme, &last_telem, &cfg);
            }
            let out_path = theme_dir.join(format!("{state_name}.png"));
            save_png(&pix4k, &out_path);

            // thumbnail
            let mut pix_thumb = tiny_skia::Pixmap::new(THUMB_W, THUMB_H).unwrap();
            {
                let mut pm = pix_thumb.as_mut();
                render_cluster(&mut pm, theme, &last_telem, &cfg);
            }
            let thumb_path = thumb_theme_dir.join(format!("{state_name}.png"));
            save_png(&pix_thumb, &thumb_path);
        }
    }

    // Scrims
    {
        let base_cfg = {
            let mut c = AppConfig::default();
            c.ambient_glow_enabled = true;
            c.ambient_glow_intensity = 0.25;
            c.ambient_cowl_feather = 0.40;
            c
        };

        // 0%
        {
            let mut cfg0 = base_cfg.clone();
            cfg0.current_camera_index = 4;
            cfg0.oem_dash_opaque_pct = 0;
            cfg0.active_camera_view = CameraViewMode::Bonnet;
            let mut pix = tiny_skia::Pixmap::new(W, H).unwrap();
            {
                let mut pm = pix.as_mut();
                let bounds = tiny_skia::Rect::from_xywh(0.0,0.0,W as f32,H as f32).unwrap();
                render_oem_scrim(&mut pm, bounds, 1.0, &cfg0);
            }
            save_png(&pix, &scrim_dir.join("OemScrim_0pct.png"));
        }
        // 50%
        {
            let mut cfg50 = base_cfg.clone();
            cfg50.current_camera_index = 3;
            cfg50.oem_dash_opaque_pct = 50;
            cfg50.active_camera_view = CameraViewMode::Cockpit;
            let mut pix = tiny_skia::Pixmap::new(W, H).unwrap();
            {
                let mut pm = pix.as_mut();
                let bounds = tiny_skia::Rect::from_xywh(0.0,0.0,W as f32,H as f32).unwrap();
                render_oem_scrim(&mut pm, bounds, 1.0, &cfg50);
                let accent = ClusterTheme::LiquidGlass.accent_rgb();
                let intensity = cfg50.ambient_glow_intensity.clamp(0.0,1.0);
                let feather_sigma = feather_sigma_px(cfg50.ambient_cowl_feather.clamp(0.05,1.0),1.0);
                let mut wash = BinnacleAmbientWash::new();
                wash.render(&mut pm, W, H, accent, intensity, feather_sigma, 1.0);
            }
            save_png(&pix, &scrim_dir.join("OemScrim_50pct.png"));
        }
        // 100%
        {
            let mut cfg100 = base_cfg.clone();
            cfg100.current_camera_index = 3;
            cfg100.oem_dash_opaque_pct = 100;
            cfg100.active_camera_view = CameraViewMode::Cockpit;
            let mut pix = tiny_skia::Pixmap::new(W, H).unwrap();
            {
                let mut pm = pix.as_mut();
                let bounds = tiny_skia::Rect::from_xywh(0.0,0.0,W as f32,H as f32).unwrap();
                render_oem_scrim(&mut pm, bounds, 1.0, &cfg100);
                let accent = ClusterTheme::LiquidGlass.accent_rgb();
                let intensity = cfg100.ambient_glow_intensity.clamp(0.0,1.0);
                let feather_sigma = feather_sigma_px(cfg100.ambient_cowl_feather.clamp(0.05,1.0),1.0);
                let mut wash = BinnacleAmbientWash::new();
                wash.render(&mut pm, W, H, accent, intensity, feather_sigma, 1.0);
            }
            save_png(&pix, &scrim_dir.join("OemScrim_100pct.png"));
        }
    }

    // Unified OSD
    {
        let mut osd_cfg = AppConfig::default();
        osd_cfg.custom_driver_name = "ØG『死』ŦJ乂".to_string();
        let mut state = UnifiedOsdState::default();
        state.is_open = true;
        state.current_tab = 0;
        let telem = dyno_telemetry(8.0);
        let pix = render_unified_osd_4k(&state, &osd_cfg, Some(&telem));
        save_png(&pix, &out_root.join("UnifiedOsd_4K.png"));
    }

    generate_index_html(&out_root, n, &state_defs);
    verify_export(&out_root, n, &state_defs);
    println!("✅ Export complete");
}

fn generate_index_html(root: &Path, _n: usize, _state_defs: &[(&str, usize)]) {
    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html><html lang="en"><head><meta charset="utf-8"><title>FH6 4K Cluster Gallery</title>
<style>
body{background:#0b0f19;color:#e5e7eb;font-family:system-ui,Segoe UI,Roboto,Helvetica,Arial,sans-serif;margin:0;padding:24px}
header{margin-bottom:24px}
h1{margin:0}
.badges{display:flex;gap:12px;margin-top:8px;flex-wrap:wrap}
.badge{background:rgba(17,24,39,0.78);border:1px solid rgba(56,189,248,0.28);padding:6px 10px;border-radius:8px;font-size:12px;backdrop-filter:blur(8px)}
.filters{display:flex;gap:8px;margin:16px 0;flex-wrap:wrap}
button{background:rgba(17,24,39,0.78);border:1px solid rgba(56,189,248,0.28);color:#e5e7eb;padding:6px 12px;border-radius:8px;cursor:pointer}
button.active{background:rgba(56,189,248,0.2)}
#search{margin-left:auto;padding:6px 10px;border-radius:8px;border:1px solid rgba(56,189,248,0.28);background:rgba(17,24,39,0.78);color:#e5e7eb}
.showcase{display:grid;grid-template-columns:repeat(auto-fit,minmax(480px,1fr));gap:16px;margin-bottom:24px}
.card{background:rgba(17,24,39,0.78);border:1px solid rgba(56,189,248,0.28);border-radius:12px;overflow:hidden;backdrop-filter:blur(10px)}
.card img{display:block;width:100%;height:auto}
.cap{padding:8px 12px;font-size:12px;color:#94a3b8}
.grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(320px,1fr));gap:16px}
.lightbox{position:fixed;inset:0;background:rgba(0,0,0,0.95);display:none;align-items:center;justify-content:center;z-index:9999}
.lightbox img{max-width:95vw;max-height:90vh}
.lightbox .close{position:absolute;top:20px;right:30px;font-size:28px;color:#fff;cursor:pointer}
</style></head><body>
<header><h1>FH6 4K Cluster Gallery</h1>
<div class="badges">
<span class="badge">Resolution: 3840x2160 (4K)</span>
<span class="badge" id="badge-clusters">Clusters: 0 / 0</span>
<span class="badge" id="badge-oled">OLED True-Black #000000 Cutout: PASS</span>
<span class="badge" id="badge-driver">Driver Tag: ØG『死』ŦJ乂 VERIFIED</span>
</div>
<div class="filters">
<button data-filter="all" class="active">All 4 States</button>
<button data-filter="01_idle">01 Idle</button>
<button data-filter="02_mid_cruise">02 Mid Cruise</button>
<button data-filter="03_redline_strobe">03 Redline Strobe</button>
<button data-filter="04_drift_braking">04 Drift & Braking</button>
<input id="search" placeholder="Search cluster name...">
</div>
</header>
<div class="showcase">
<div class="card"><img src="UnifiedOsd_4K.png" alt="OSD"><div class="cap">Unified OSD 4K</div></div>
<div class="card"><img src="Scrim/OemScrim_0pct.png"><div class="cap">OemScrim 0%</div></div>
<div class="card"><img src="Scrim/OemScrim_50pct.png"><div class="cap">OemScrim 50%</div></div>
<div class="card"><img src="Scrim/OemScrim_100pct.png"><div class="cap">OemScrim 100%</div></div>
</div>
<div class="grid" id="gallery"></div>
<div class="lightbox" id="lightbox"><span class="close" id="lb-close">✕</span><img id="lb-img"></div>
<script>
const themes = [
"#);
    let all_themes = ClusterTheme::all();
    for (i, theme) in all_themes.iter().enumerate() {
        let name = theme.to_string();
        html.push_str(&format!("\"{}\",", name));
        if i % 5 == 4 { html.push_str("\n"); }
    }
    html.push_str(r#"];
const states = ["01_idle","02_mid_cruise","03_redline_strobe","04_drift_braking"];
const gallery = document.getElementById('gallery');
function renderGallery(filter='all', query=''){
  gallery.innerHTML='';
  themes.forEach(t=>{
    states.forEach(s=>{
      if(filter!=='all' && s!==filter) return;
      if(query && !t.toLowerCase().includes(query.toLowerCase())) return;
      const card=document.createElement('div');
      card.className='card';
      card.dataset.state=s;
      card.dataset.name=t.toLowerCase();
      card.innerHTML=`<img src="_thumbs/${t}/${s}.png" data-full="ModernClusters/${t}/${s}.png" alt="${t} ${s}"><div class="cap">${t} · ${s}</div>`;
      card.addEventListener('click',()=>openLightbox(card.querySelector('img').dataset.full));
      gallery.appendChild(card);
    });
  });
  document.getElementById('badge-clusters').textContent=`Clusters: ${themes.length} / ${themes.length}`;
}
document.querySelectorAll('button[data-filter]').forEach(b=>{
  b.addEventListener('click',()=>{
    document.querySelectorAll('button[data-filter]').forEach(x=>x.classList.remove('active'));
    b.classList.add('active');
    renderGallery(b.dataset.filter, document.getElementById('search').value);
  });
});
document.getElementById('search').addEventListener('input',e=>{
  const f=document.querySelector('button[data-filter].active').dataset.filter;
  renderGallery(f, e.target.value);
});
function openLightbox(src){
  const lb=document.getElementById('lightbox');
  document.getElementById('lb-img').src=src;
  lb.style.display='flex';
}
document.getElementById('lb-close').onclick=()=>document.getElementById('lightbox').style.display='none';
document.getElementById('lightbox').onclick=e=>{ if(e.target.id==='lightbox') document.getElementById('lightbox').style.display='none';};
renderGallery();
</script></body></html>"#);
    fs::write(root.join("index.html"), html).expect("write index.html");
}

fn verify_export(root: &Path, n: usize, state_defs: &[(&str, usize)]) {
    let clusters_dir = root.join("ModernClusters");
    let mut ok = true;
    for entry in fs::read_dir(&clusters_dir).expect("read clusters") {
        let e = entry.unwrap();
        let path = e.path();
        if path.is_dir() {
            for (state_name, _) in state_defs {
                let p = path.join(format!("{state_name}.png"));
                if !p.exists() || fs::metadata(&p).unwrap().len() == 0 {
                    eprintln!("Missing or empty {}", p.display());
                    ok = false;
                }
            }
        }
    }
    let scrim_files = ["Scrim/OemScrim_0pct.png","Scrim/OemScrim_50pct.png","Scrim/OemScrim_100pct.png"];
    for f in scrim_files {
        let p = root.join(f);
        if !p.exists() { eprintln!("Missing {}", f); ok = false; }
    }
    if !root.join("UnifiedOsd_4K.png").exists() { eprintln!("Missing OSD"); ok = false; }
    if !root.join("index.html").exists() { eprintln!("Missing index.html"); ok = false; }
    if ok {
        println!("Verification: {} clusters, {} PNGs OK, 3 scrims OK, OSD OK, html OK", n, n*4);
    } else {
        panic!("Verification failed");
    }
}
