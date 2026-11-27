//! Log setup shared by both binaries: `RUST_LOG`-style filtering (default `info`), written to
//! stderr or appended to a file. Panics on any thread are logged too.
use std::fs::OpenOptions;
use std::path::Path;
use std::sync::{Mutex, Once};
use tracing_subscriber::EnvFilter;

/// Installs the global subscriber and the panic log. Logging failures never stop the app.
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
    log_panics();
}

/// Writes every panic to the log as well as to stderr. A release build has no console, so
/// without this a panic on any thread would leave no trace in the `--log` file. The hook is
/// installed once per process, however often this is called.
fn log_panics() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let thread = std::thread::current();
            tracing::error!(thread = thread.name().unwrap_or("unnamed"), panic = %info, "a thread panicked");
            previous(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::sync::Arc;

    /// Collects what the subscriber writes.
    #[derive(Clone, Default)]
    struct Captured(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let mut buffer = self.0.lock().map_err(|_| io::Error::other("capture lock poisoned"))?;
            buffer.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_panic_on_another_thread_reaches_the_log() {
        let captured = Captured::default();
        let subscriber = {
            let captured = captured.clone();
            tracing_subscriber::fmt()
                .with_ansi(false)
                .with_writer(move || captured.clone())
                .finish()
        };
        // The hook stays for the rest of this test process; all it adds is a log line. A second
        // call must not add a second hook.
        log_panics();
        log_panics();
        let joined = std::thread::Builder::new()
            .name("fh6-render".into())
            .spawn(move || {
                let _log = tracing::subscriber::set_default(subscriber);
                panic!("frame 42 went wrong");
            })
            .expect("the thread starts")
            .join();
        assert!(joined.is_err());
        let bytes = captured.0.lock().expect("capture lock").clone();
        let log = String::from_utf8_lossy(&bytes);
        assert!(log.contains("ERROR"), "logged as an error: {log}");
        assert_eq!(
            log.matches("frame 42 went wrong").count(),
            1,
            "the message, once: {log}"
        );
        assert!(log.contains("fh6-render"), "the thread: {log}");
        assert!(log.contains("logging.rs"), "where it happened: {log}");
    }
}
