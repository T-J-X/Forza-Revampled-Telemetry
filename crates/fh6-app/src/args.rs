//! Command-line options of `FH6_WheelDash`.
use fh6_core::ClusterTheme;
use std::path::PathBuf;

/// The game's process image name.
pub const GAME_EXE: &str = "ForzaHorizon6.exe";

/// Usage text for `--help` and argument errors.
pub const USAGE: &str = "FH6_WheelDash [options]
  --auto-exit           quit when the game process exits (used by FH6_SteamLauncher)
  --demo                show the overlay regardless of race state and focus (synthetic data if no packets)
  --cluster NAME        show this catalog cluster in every view for this run (e.g. LiquidGlass)
  --stats              draw receive-to-present latency and CPU frame time on the overlay
  --bench-seconds N     run N seconds, then write latency/frame-time percentiles and quit
  --bench-out PATH      where --bench-seconds writes its report (default docs/perf/engine.md)
  --game-process NAME   foreground process that counts as the game (default ForzaHorizon6.exe)
  --config PATH         use this config.json instead of searching for one
  --log PATH            append logs to PATH (default: stderr)
  --open-widget         open the live state page (also when another instance is running)
  --help                print this text";

/// Parsed options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppArgs {
    /// Quit when the game process exits after having been seen.
    pub auto_exit: bool,
    /// Bypass the visibility rules; synthesise telemetry when none arrives.
    pub demo: bool,
    /// A cluster forced in every view for this run (the config file is not changed).
    pub cluster: Option<ClusterTheme>,
    /// Draw the latency stats on the overlay.
    pub stats: bool,
    /// Benchmark duration, if benchmarking.
    pub bench_seconds: Option<u64>,
    /// Benchmark report path.
    pub bench_out: PathBuf,
    /// Process name that counts as the game for focus checks.
    pub game_process: String,
    /// Explicit config path.
    pub config: Option<PathBuf>,
    /// Log file.
    pub log: Option<PathBuf>,
    /// Open the live state page.
    pub open_widget: bool,
}

impl Default for AppArgs {
    fn default() -> Self {
        Self {
            auto_exit: false,
            demo: false,
            cluster: None,
            stats: false,
            bench_seconds: None,
            bench_out: PathBuf::from("docs/perf/engine.md"),
            game_process: GAME_EXE.to_string(),
            config: None,
            log: None,
            open_widget: false,
        }
    }
}

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Run the overlay.
    Run(AppArgs),
    /// Print usage and exit.
    Help,
}

/// Why the command line was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ArgsError {
    /// An option needs a value that was not given.
    #[error("{0} needs a value")]
    MissingValue(&'static str),
    /// A value could not be parsed.
    #[error("invalid value {value:?} for {option}")]
    InvalidValue {
        /// The option.
        option: &'static str,
        /// What was given.
        value: String,
    },
    /// An option this program does not know.
    #[error("unknown option {0:?}")]
    Unknown(String),
}

/// Parses the arguments after the program name.
///
/// # Errors
/// [`ArgsError`] for unknown options, missing values or unparsable numbers.
pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, ArgsError> {
    let mut out = AppArgs::default();
    let mut it = args.into_iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--auto-exit" => out.auto_exit = true,
            "--demo" => out.demo = true,
            "--stats" => out.stats = true,
            "--open-widget" => out.open_widget = true,
            "--help" | "-h" | "/?" => return Ok(Command::Help),
            "--bench-seconds" => {
                let v = it.next().ok_or(ArgsError::MissingValue("--bench-seconds"))?;
                let secs: u64 = v.parse().ok().filter(|&s| s > 0).ok_or(ArgsError::InvalidValue {
                    option: "--bench-seconds",
                    value: v.clone(),
                })?;
                out.bench_seconds = Some(secs);
            }
            "--bench-out" => out.bench_out = it.next().ok_or(ArgsError::MissingValue("--bench-out"))?.into(),
            "--game-process" => {
                let v = it.next().ok_or(ArgsError::MissingValue("--game-process"))?;
                if v.trim().is_empty() {
                    return Err(ArgsError::InvalidValue {
                        option: "--game-process",
                        value: v,
                    });
                }
                out.game_process = v;
            }
            "--config" => out.config = Some(it.next().ok_or(ArgsError::MissingValue("--config"))?.into()),
            "--log" => out.log = Some(it.next().ok_or(ArgsError::MissingValue("--log"))?.into()),
            "--cluster" => {
                let v = it.next().ok_or(ArgsError::MissingValue("--cluster"))?;
                out.cluster = Some(ClusterTheme::from_name(&v).ok_or(ArgsError::InvalidValue {
                    option: "--cluster",
                    value: v.clone(),
                })?);
            }
            _ => return Err(ArgsError::Unknown(arg)),
        }
    }
    Ok(Command::Run(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(args: &[&str]) -> Result<Command, ArgsError> {
        parse(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_args_is_the_default_run() {
        assert_eq!(run(&[]), Ok(Command::Run(AppArgs::default())));
    }

    #[test]
    fn flags_and_values() {
        let Ok(Command::Run(a)) = run(&[
            "--auto-exit",
            "--stats",
            "--demo",
            "--bench-seconds",
            "20",
            "--game-process",
            "notepad.exe",
        ]) else {
            panic!("parse failed");
        };
        assert!(a.auto_exit && a.stats && a.demo);
        assert_eq!(a.bench_seconds, Some(20));
        assert_eq!(a.game_process, "notepad.exe");
    }

    #[test]
    fn errors_are_reported() {
        assert_eq!(
            run(&["--bench-seconds"]),
            Err(ArgsError::MissingValue("--bench-seconds"))
        );
        assert!(matches!(
            run(&["--bench-seconds", "0"]),
            Err(ArgsError::InvalidValue { .. })
        ));
        assert_eq!(run(&["--frobnicate"]), Err(ArgsError::Unknown("--frobnicate".into())));
        assert_eq!(run(&["--help"]), Ok(Command::Help));
    }

    #[test]
    fn cluster_takes_a_catalog_theme_name() {
        let Ok(Command::Run(a)) = run(&["--demo", "--cluster", "vernierloupe"]) else {
            panic!("parse failed");
        };
        assert_eq!(a.cluster, Some(ClusterTheme::VernierLoupe));
        assert_eq!(run(&["--cluster"]), Err(ArgsError::MissingValue("--cluster")));
        assert!(matches!(
            run(&["--cluster", "NoSuchCluster"]),
            Err(ArgsError::InvalidValue {
                option: "--cluster",
                ..
            })
        ));
    }
}
