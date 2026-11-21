//! A tiny loopback HTTP server: `GET /api/state` returns the latest telemetry as JSON, and
//! `GET /` a server-rendered status page (no scripts). The Phase 10 configurator replaces it.
use fh6_core::TelemetryReader;
use std::fmt::Write as _;
use std::io::{self, BufRead, BufReader, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

/// The loopback port shared with the Phase 10 configurator.
pub const PORT: u16 = 5301;

/// The status page URL.
pub fn url() -> String {
    format!("http://127.0.0.1:{PORT}/")
}

/// Binds 127.0.0.1:5301 and serves on a background thread.
///
/// # Errors
/// Returns the I/O error if the port cannot be bound or the thread cannot start.
pub fn spawn(telemetry: TelemetryReader) -> io::Result<()> {
    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT))?;
    thread::Builder::new().name("fh6-widget-http".into()).spawn(move || {
        for stream in listener.incoming().flatten() {
            if let Err(err) = handle(stream, &telemetry) {
                tracing::debug!(error = %err, "widget request failed");
            }
        }
    })?;
    Ok(())
}

fn handle(mut stream: TcpStream, telemetry: &TelemetryReader) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    let mut request_line = String::new();
    BufReader::new(&stream).read_line(&mut request_line)?;
    let path = request_line.split_whitespace().nth(1).unwrap_or("/");
    let frame = telemetry.latest();
    let (content_type, body) = if path.starts_with("/api/state") {
        let json = serde_json::to_string(&frame.data).unwrap_or_else(|_| "{}".to_string());
        ("application/json", json)
    } else {
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
             <tr><td>RPM</td><td>{:.0} / {:.0}</td></tr></table>\
             <p>JSON: <a href=\"/api/state\">/api/state</a>. The configurator arrives in Phase 10.</p></body></html>",
            d.is_race_on,
            d.speed_mph(),
            d.gear_string(),
            d.current_engine_rpm,
            d.engine_max_rpm,
        );
        ("text/html; charset=utf-8", html)
    };
    write!(
        stream,
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body.as_bytes())
}
