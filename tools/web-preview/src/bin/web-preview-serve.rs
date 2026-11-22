//! Serves `tools/web-preview/www/` on <http://127.0.0.1:5310/> (loopback only) with the MIME
//! types WebAssembly needs. A development helper, not part of the overlay.
#[cfg(not(target_arch = "wasm32"))]
fn main() -> std::io::Result<()> {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::path::{Component, Path, PathBuf};
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("www");
    let port: u16 = std::env::args().nth(1).and_then(|p| p.parse().ok()).unwrap_or(5310);
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!("web-preview: http://127.0.0.1:{port}/  (Ctrl+C to stop)");
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        let mut line = String::new();
        if BufReader::new(&stream).read_line(&mut line).is_err() {
            continue;
        }
        let path = line.split_whitespace().nth(1).unwrap_or("/");
        let path = path.split('?').next().unwrap_or("/");
        let rel = if path == "/" {
            "index.html"
        } else {
            path.trim_start_matches('/')
        };
        let safe = Path::new(rel).components().all(|c| matches!(c, Component::Normal(_)));
        let file = root.join(rel);
        let (status, body, mime) = match (safe, std::fs::read(&file)) {
            (true, Ok(body)) => {
                let mime = match file.extension().and_then(|e| e.to_str()) {
                    Some("html") => "text/html; charset=utf-8",
                    Some("js") => "text/javascript",
                    Some("wasm") => "application/wasm",
                    Some("css") => "text/css",
                    Some("png") => "image/png",
                    _ => "application/octet-stream",
                };
                ("200 OK", body, mime)
            }
            _ => ("404 Not Found", b"not found".to_vec(), "text/plain"),
        };
        let header = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {mime}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(&body);
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {}
