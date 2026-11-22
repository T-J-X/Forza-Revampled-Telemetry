//! A tiny loopback HTTP server: `GET /api/state` returns the latest telemetry as JSON, and
//! `GET /` a server-rendered status page (no scripts). The Phase 10 configurator replaces it.
//!
//! Only requests whose `Host` header names this loopback address are answered, so a web page
//! cannot read the API through DNS rebinding. Each connection gets its own short-lived thread.
use fh6_core::{CameraPatchHandle, TelemetryReader};
use std::fmt::Write as _;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

/// The loopback port shared with the Phase 10 configurator.
pub const PORT: u16 = 5301;

/// Most header lines read from one request.
const MAX_HEADER_LINES: usize = 64;

/// The status page URL.
pub fn url() -> String {
    format!("http://127.0.0.1:{PORT}/")
}

/// True if a `Host` header value names this server on loopback.
pub fn is_allowed_host(host: &str) -> bool {
    let host = host.trim();
    let expected = [format!("127.0.0.1:{PORT}"), format!("localhost:{PORT}")];
    expected.iter().any(|e| host.eq_ignore_ascii_case(e))
}

/// Binds 127.0.0.1:5301 and serves on a background thread.
///
/// # Errors
/// Returns the I/O error if the port cannot be bound or the thread cannot start.
pub fn spawn(telemetry: TelemetryReader, camera: CameraPatchHandle) -> io::Result<()> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT))?;
    thread::Builder::new().name("fh6-widget-http".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            let telemetry = telemetry.clone();
            let camera = camera.clone();
            let spawned = thread::Builder::new().name("fh6-widget-conn".into()).spawn(move || {
                if let Err(err) = handle(stream, &telemetry, &camera) {
                    tracing::debug!(error = %err, "widget request failed");
                }
            });
            if let Err(err) = spawned {
                tracing::debug!(error = %err, "widget connection thread not started");
            }
        }
    })?;
    Ok(())
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &str) -> io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body.as_bytes())
}

fn handle(mut stream: TcpStream, telemetry: &TelemetryReader, camera: &CameraPatchHandle) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(500)))?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut host_ok = false;
    for _ in 0..MAX_HEADER_LINES {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("host") {
                host_ok = is_allowed_host(value);
            }
        }
    }
    if !host_ok {
        return respond(&mut stream, "403 Forbidden", "text/plain", "forbidden host");
    }

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    if path.starts_with("/api/camera") {
        return respond(
            &mut stream,
            "200 OK",
            "application/json",
            &camera_json(&camera.status()),
        );
    }
    let frame = telemetry.latest();
    if path.starts_with("/api/state") {
        let json = serde_json::to_string(&frame.data).unwrap_or_else(|_| "{}".to_string());
        return respond(&mut stream, "200 OK", "application/json", &json);
    }
    let d = &frame.data;
    let age = frame
        .age(Instant::now())
        .map_or_else(|| "never".to_string(), |a| format!("{} ms ago", a.as_millis()));
    let mut html = String::with_capacity(1024);
    let _ = write!(
        html,
        "<!doctype html><html><head><meta charset=\"utf-8\"><meta http-equiv=\"refresh\" content=\"1\">\
         <title>FH6 WheelDash</title><style>body{{background:#000;color:#fff;font:16px Bahnschrift,sans-serif;padding:24px}}\
         td{{padding:2px 12px}}</style></head><body><h1>FH6 WheelDash</h1><table>\
         <tr><td>Last packet</td><td>{age}</td></tr><tr><td>Race on</td><td>{}</td></tr>\
         <tr><td>Speed</td><td>{:.0} mph</td></tr><tr><td>Gear</td><td>{}</td></tr>\
         <tr><td>RPM</td><td>{:.0} / {:.0}</td></tr><tr><td>Camera files</td><td>{}</td></tr></table>\
         <p>JSON: <a href=\"/api/state\">/api/state</a>, <a href=\"/api/camera\">/api/camera</a>. The configurator arrives in Phase 10.</p></body></html>",
        d.is_race_on,
        d.speed_mph(),
        d.gear_string(),
        d.current_engine_rpm,
        d.engine_max_rpm,
        html_escape(camera.status().state.message()),
    );
    respond(&mut stream, "200 OK", "text/html; charset=utf-8", &html)
}

/// The camera patch status as JSON: `{"state","message","restartNeeded","installDir","backupDir"}`.
pub fn camera_json(s: &fh6_core::CameraPatchStatus) -> String {
    let state = match &s.state {
        fh6_core::CameraPatchState::Off => "off",
        fh6_core::CameraPatchState::NoInstall => "noInstall",
        fh6_core::CameraPatchState::UpToDate => "upToDate",
        fh6_core::CameraPatchState::Applied => "applied",
        fh6_core::CameraPatchState::QueuedUntilGameExits => "queued",
        fh6_core::CameraPatchState::Restored => "restored",
        fh6_core::CameraPatchState::Failed(_) => "failed",
    };
    serde_json::json!({
        "state": state,
        "message": s.state.message(),
        "restartNeeded": s.state.restart_needed(),
        "installDir": s.install_dir,
        "backupDir": s.backup_dir,
    })
    .to_string()
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_loopback_hosts_are_allowed() {
        assert!(is_allowed_host(" 127.0.0.1:5301\r\n"));
        assert!(is_allowed_host("LOCALHOST:5301"));
        assert!(!is_allowed_host("evil.example:5301"));
        assert!(!is_allowed_host("127.0.0.1"));
        assert!(!is_allowed_host(""));
    }
}
