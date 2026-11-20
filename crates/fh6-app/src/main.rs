#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod single_instance;
mod tray;
mod process_watcher;
mod remapper;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use fh6_core::{AppConfig, FH6TelemetryData};
use single_instance::{SingleInstanceGuard, SingleInstanceError};
use tray::TrayIcon;
use process_watcher::{ProcessWatcher, enumerate_processes, GameState};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let auto_exit = args.iter().any(|a| a == "--auto-exit");
    let open_widget = args.iter().any(|a| a == "--open-widget");

    // Single instance guard
    let _guard = match SingleInstanceGuard::new() {
        Ok(g) => g,
        Err(SingleInstanceError::AlreadyRunning) => {
            if open_widget {
                launch_widget();
            }
            std::process::exit(0);
        }
        Err(_) => std::process::exit(1),
    };

    // Load config
    let config_path = "config.json";
    let _config = AppConfig::load(config_path);

    // System tray
    let mut tray = TrayIcon::new("FH6 WheelDash");
    let _ = tray.add();

    // Telemetry shared state
    let telemetry = Arc::new(Mutex::new(FH6TelemetryData::default()));
    let telemetry_clone = telemetry.clone();
    thread::spawn(move || {
        start_widget_server(telemetry_clone);
    });

    // Process watcher
    let mut watcher = ProcessWatcher::new("ForzaHorizon6.exe", 2000);
    let telemetry_for_watcher = telemetry.clone();
    let tray_for_watcher = tray; // move into thread? We'll clone tooltip via Arc? Simplify: spawn thread that polls and updates tray via interior mutability.
    // For simplicity, keep tray updates in main loop
    if auto_exit {
        loop {
            let procs = enumerate_processes();
            if let Some(state) = watcher.check(&procs) {
                match state {
                    GameState::Stopped => {
                        // update tooltip
                        // tray.update_tooltip("Game exited");
                        break;
                    }
                    GameState::Started => {
                        // tray.update_tooltip("Game started");
                    }
                }
            }
            thread::sleep(Duration::from_millis(watcher.poll_interval().as_millis() as u64));
            if !process_exists_stub("ForzaHorizon6.exe") {
                thread::sleep(Duration::from_millis(1500));
                break;
            }
        }
    } else {
        // Background watcher thread
        let watcher_handle = thread::spawn(move || {
            let mut w = ProcessWatcher::new("ForzaHorizon6.exe", 2000);
            loop {
                let procs = enumerate_processes();
                if let Some(state) = w.check(&procs) {
                    // Log state change
                    match state {
                        GameState::Started => {
                            eprintln!("GameStarted");
                        }
                        GameState::Stopped => {
                            eprintln!("GameExited");
                        }
                    }
                }
                thread::sleep(Duration::from_millis(w.poll_interval().as_millis() as u64));
            }
        });
        let _ = watcher_handle;
        // Keep main thread alive
        loop {
            thread::sleep(Duration::from_secs(60));
        }
    }
}

fn process_exists_stub(_name: &str) -> bool {
    false
}

fn launch_widget() {
    let _ = std::process::Command::new("msedge.exe")
        .args(&["--app", "http://127.0.0.1:5301/widget", "--window-size=420,680"])
        .spawn();
}

fn start_widget_server(telemetry: Arc<Mutex<FH6TelemetryData>>) {
    if let Ok(listener) = TcpListener::bind("127.0.0.1:5301") {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let tel = telemetry.clone();
                thread::spawn(move || handle_client(&mut stream, &tel));
            }
        }
    }
}

fn handle_client(stream: &mut TcpStream, telemetry: &Arc<Mutex<FH6TelemetryData>>) {
    let mut buf = [0u8; 1024];
    let _ = stream.read(&mut buf);
    let request = String::from_utf8_lossy(&buf);
    let path = if request.contains("GET /api/state") {
        "/api/state"
    } else {
        "/widget"
    };
    let response = if path == "/api/state" {
        let data = telemetry.lock().unwrap();
        let json = serde_json::to_string(&*data).unwrap_or_else(|_| "{}".to_string());
        format_response(200, "application/json", &json)
    } else {
        let html = r#"<!DOCTYPE html><html><head><title>FH6 Widget</title><style>body{background:#0b0f19;color:#fff;font-family:sans-serif}</style></head><body><h1>ØG『死』ŦJ乂</h1><div id="telemetry"></div><script>setInterval(()=>fetch('/api/state').then(r=>r.json()).then(d=>{document.getElementById('telemetry').innerText=`RPM:${d.current_engine_rpm} Speed:${d.speed_mph} Gear:${d.gear}`}),100)</script></body></html>"#;
        format_response(200, "text/html", html)
    };
    let _ = stream.write_all(response.as_bytes());
}

fn format_response(code: u16, content_type: &str, body: &str) -> String {
    format!("HTTP/1.1 {} OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", code, content_type, body.len(), body)
}
