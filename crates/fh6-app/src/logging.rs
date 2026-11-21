//! Log setup shared by both binaries: `RUST_LOG`-style filtering (default `info`), written to
//! stderr or appended to a file.
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;

/// Installs the global subscriber. Logging failures never stop the app.
pub fn init(log_file: Option<&Path>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter).with_target(false);
    let file = log_file.and_then(|path| {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|err| eprintln!("log file {} not opened: {err}", path.display()))
            .ok()
    });
    let result = match file {
        Some(file) => builder.with_ansi(false).with_writer(Mutex::new(file)).try_init(),
        None => builder.with_writer(std::io::stderr).try_init(),
    };
    if let Err(err) = result {
        eprintln!("logging not initialised: {err}");
    }
}
