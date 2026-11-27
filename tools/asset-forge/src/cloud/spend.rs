//! The spend log: one line of JSON per finished pod session in `spend.jsonl`, next to the
//! tools and outside git. The amounts are this PC's estimate (hourly rate times the seconds
//! the pod was up); Runpod's billing history is the authority and `cloud spend` prints both.
use super::clock;
use crate::error::{io_err, ForgeError, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::path::Path;

/// One finished pod session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// The pod's id.
    pub pod_id: String,
    /// The GPU type it ran on.
    pub gpu: String,
    /// The data centre.
    pub data_center: String,
    /// The hourly compute rate in USD.
    pub usd_per_hour: f64,
    /// When the pod was created (UTC, RFC 3339).
    pub started: String,
    /// When it ended (UTC, RFC 3339).
    pub ended: String,
    /// Seconds between the two.
    pub seconds: u64,
    /// Rate times time, in USD.
    pub usd: f64,
    /// The thread that held the cloud lock when the pod was started.
    pub thread: String,
    /// The job named in that lock.
    pub job: String,
    /// How the session ended: `cloud down`, the pod's watchdog, or a recycle.
    pub how: String,
}

impl Session {
    /// A session from its start and end in Unix seconds.
    #[must_use]
    pub fn close(open: &super::State, ended: u64, how: &str) -> Self {
        let seconds = ended.saturating_sub(open.started);
        Self {
            pod_id: open.pod_id.clone(),
            gpu: open.gpu.clone(),
            data_center: open.data_center.clone(),
            usd_per_hour: open.usd_per_hour,
            started: clock::rfc3339(open.started),
            ended: clock::rfc3339(ended),
            seconds,
            usd: cost(open.usd_per_hour, seconds),
            thread: open.thread.clone(),
            job: open.job.clone(),
            how: how.to_owned(),
        }
    }
}

/// USD for `seconds` at `usd_per_hour`, rounded to a tenth of a cent.
#[must_use]
pub fn cost(usd_per_hour: f64, seconds: u64) -> f64 {
    (usd_per_hour * seconds as f64 / 3600.0 * 1000.0).round() / 1000.0
}

/// Adds a session to the log.
pub fn append(log: &Path, session: &Session) -> Result<()> {
    if let Some(dir) = log.parent() {
        std::fs::create_dir_all(dir).map_err(|e| io_err(dir, e))?;
    }
    let line = serde_json::to_string(session).map_err(|source| ForgeError::Json {
        context: log.display().to_string(),
        source,
    })?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .map_err(|e| io_err(log, e))?;
    writeln!(file, "{line}").map_err(|e| io_err(log, e))
}

/// Every session in the log, oldest first. A missing log is an empty one; a damaged line is
/// skipped.
#[must_use]
pub fn load(log: &Path) -> Vec<Session> {
    std::fs::read_to_string(log)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// Total USD and total seconds over `sessions`.
#[must_use]
pub fn total(sessions: &[Session]) -> (f64, u64) {
    // A fold from +0.0: the sum of an empty list of floats is -0.0, which prints as "$-0.000".
    let usd = sessions.iter().fold(0.0_f64, |sum, s| sum + s.usd);
    (
        (usd * 1000.0).round() / 1000.0,
        sessions.iter().map(|s| s.seconds).sum(),
    )
}

fn csv_field(text: &str) -> String {
    if text.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", text.replace('"', "\"\""))
    } else {
        text.to_owned()
    }
}

/// The log as CSV with a header and a total row, for a spreadsheet.
#[must_use]
pub fn csv(sessions: &[Session]) -> String {
    let mut out =
        String::from("started_utc,ended_utc,minutes,usd,usd_per_hour,gpu,data_center,thread,job,ended_by,pod_id\n");
    for s in sessions {
        let fields = [
            s.started.clone(),
            s.ended.clone(),
            format!("{:.1}", s.seconds as f64 / 60.0),
            format!("{:.3}", s.usd),
            format!("{:.2}", s.usd_per_hour),
            csv_field(&s.gpu),
            csv_field(&s.data_center),
            csv_field(&s.thread),
            csv_field(&s.job),
            csv_field(&s.how),
            csv_field(&s.pod_id),
        ];
        out.push_str(&fields.join(","));
        out.push('\n');
    }
    let (usd, seconds) = total(sessions);
    out.push_str(&format!("total,,{:.1},{usd:.3},,,,,,,\n", seconds as f64 / 60.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> crate::cloud::State {
        crate::cloud::State {
            pod_id: "p1".into(),
            gpu: "NVIDIA GeForce RTX 4090".into(),
            data_center: "EU-RO-1".into(),
            usd_per_hour: 0.69,
            started: 1_795_787_495,
            touched: 1_795_787_495,
            thread: "LiquidGlass, pilot".into(),
            job: "hero plates".into(),
            max_secs: 0,
        }
    }

    #[test]
    fn cost_is_rate_times_time() {
        assert!((cost(0.69, 3600) - 0.69).abs() < 1e-9);
        assert!((cost(0.69, 600) - 0.115).abs() < 1e-9);
        assert!(cost(0.89, 0).abs() < 1e-9);
    }

    #[test]
    fn a_session_closes_with_its_length_and_cost() {
        let s = Session::close(&open(), 1_795_787_495 + 1800, "cloud down");
        assert_eq!(s.seconds, 1800);
        assert!((s.usd - 0.345).abs() < 1e-9);
        assert_eq!(s.started, "2026-11-27T13:51:35Z");
        assert_eq!(s.ended, "2026-11-27T14:21:35Z");
        // A clock that went backwards never gives a negative session.
        assert_eq!(Session::close(&open(), 5, "cloud down").seconds, 0);
    }

    #[test]
    fn the_log_round_trips_and_skips_damage() {
        let dir = std::env::temp_dir().join("forge_spend_log");
        let _ = std::fs::remove_dir_all(&dir);
        let log = dir.join("spend.jsonl");
        assert!(load(&log).is_empty());
        let a = Session::close(&open(), 1_795_787_495 + 600, "cloud down");
        let b = Session::close(&open(), 1_795_787_495 + 1200, "the pod's watchdog");
        append(&log, &a).unwrap();
        std::fs::OpenOptions::new()
            .append(true)
            .open(&log)
            .unwrap()
            .write_all(b"not json\n")
            .unwrap();
        append(&log, &b).unwrap();
        let back = load(&log);
        assert_eq!(back, vec![a, b]);
        let (usd, seconds) = total(&back);
        assert_eq!(seconds, 1800);
        assert!((usd - 0.345).abs() < 1e-9);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn an_empty_log_totals_to_a_plain_zero() {
        let (usd, seconds) = total(&[]);
        assert_eq!(seconds, 0);
        assert_eq!(format!("{usd:.3}"), "0.000");
        assert!(csv(&[]).ends_with(
            "total,,0.0,0.000,,,,,,,
"
        ));
    }

    #[test]
    fn csv_quotes_commas_and_ends_with_the_total() {
        let s = Session::close(&open(), 1_795_787_495 + 600, "cloud down");
        let text = csv(&[s]);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("started_utc,ended_utc,minutes,usd,"));
        assert!(lines[1].contains("\"LiquidGlass, pilot\""), "{}", lines[1]);
        assert!(lines[1].contains(",10.0,0.115,0.69,"), "{}", lines[1]);
        assert_eq!(lines[2], "total,,10.0,0.115,,,,,,,");
    }
}
