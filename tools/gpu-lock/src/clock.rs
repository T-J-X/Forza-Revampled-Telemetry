//! Wall-clock times for the lock files, the messages and the log, in UTC, without a date crate.

use std::time::{SystemTime, UNIX_EPOCH};

/// Milliseconds since the Unix epoch (0 for a clock set before 1970).
#[must_use]
pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
}

/// Seconds since the Unix epoch.
#[must_use]
pub fn now_secs() -> u64 {
    now_ms() / 1000
}

/// `2026-11-28T06:20:00Z` for a time in seconds since the epoch.
#[must_use]
pub fn utc(secs: u64) -> String {
    let (date, rest) = split(secs);
    format!("{date}T{:02}:{:02}:{:02}Z", rest / 3600, rest % 3600 / 60, rest % 60)
}

/// `2026-11-28T06:20:00.153Z` for a time in milliseconds since the epoch, for the log.
#[must_use]
pub fn utc_ms(ms: u64) -> String {
    let (date, rest) = split(ms / 1000);
    format!(
        "{date}T{:02}:{:02}:{:02}.{:03}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60,
        ms % 1000
    )
}

/// `06:20 UTC`, for messages.
#[must_use]
pub fn hhmm(secs: u64) -> String {
    let rest = secs % 86_400;
    format!("{:02}:{:02} UTC", rest / 3600, rest % 3600 / 60)
}

/// A span for messages: `12 s`, `4 min` or `1 h 05 min`.
#[must_use]
pub fn span(secs: u64) -> String {
    if secs < 60 {
        format!("{secs} s")
    } else if secs < 3600 {
        format!("{} min", secs / 60)
    } else {
        format!("{} h {:02} min", secs / 3600, secs % 3600 / 60)
    }
}

/// The date (`YYYY-MM-DD`) of a time in seconds, and the seconds since its midnight.
fn split(secs: u64) -> (String, u64) {
    let days = i64::try_from(secs / 86_400).unwrap_or(0);
    let (year, month, day) = civil_from_days(days);
    (format!("{year:04}-{month:02}-{day:02}"), secs % 86_400)
}

/// Year, month and day of a count of days since 1970-01-01 (Howard Hinnant's `civil_from_days`).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era = (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_are_written_in_utc() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc(1_795_846_800), "2026-11-28T06:20:00Z");
        assert_eq!(utc(1_798_761_599), "2026-12-31T23:59:59Z");
        assert_eq!(utc_ms(1_795_846_800_153), "2026-11-28T06:20:00.153Z");
        assert_eq!(utc_ms(1_500), "1970-01-01T00:00:01.500Z");
        assert_eq!(hhmm(1_795_846_800), "06:20 UTC");
    }

    #[test]
    fn spans_read_as_seconds_minutes_or_hours() {
        assert_eq!(span(0), "0 s");
        assert_eq!(span(59), "59 s");
        assert_eq!(span(60), "1 min");
        assert_eq!(span(3599), "59 min");
        assert_eq!(span(3600), "1 h 00 min");
        assert_eq!(span(3 * 3600 + 5 * 60 + 7), "3 h 05 min");
    }

    #[test]
    fn the_clock_is_after_the_year_2020() {
        assert!(now_secs() > 1_577_836_800);
        assert!(now_ms() / 1000 >= now_secs() - 1);
    }
}
