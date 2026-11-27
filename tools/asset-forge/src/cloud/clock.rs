//! UTC time for the spend log: Unix seconds to and from RFC 3339 text, with no time crate.
//! The calendar maths is the standard days-from-civil algorithm (proleptic Gregorian).
use std::time::{SystemTime, UNIX_EPOCH};

/// Seconds in a day.
const DAY: u64 = 86_400;

/// Now, in Unix seconds.
#[must_use]
pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// Days since 1970-01-01 for a calendar date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The calendar date for a count of days since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// `2026-11-27T13:51:35Z` for Unix seconds.
#[must_use]
pub fn rfc3339(secs: u64) -> String {
    let (year, month, day) = civil_from_days(i64::try_from(secs / DAY).unwrap_or(0));
    let rest = secs % DAY;
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Unix seconds for RFC 3339 text such as `2026-11-27T13:51:35Z`, `...35.123Z` or
/// `...35+01:00`. Returns `None` for anything else.
#[must_use]
pub fn parse_rfc3339(text: &str) -> Option<u64> {
    let text = text.trim();
    let (date, time) = text.split_once(['T', 't', ' '])?;
    let mut ymd = date.splitn(3, '-').map(str::parse::<i64>);
    let (year, month, day) = (ymd.next()?.ok()?, ymd.next()?.ok()?, ymd.next()?.ok()?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Split the clock from the zone: `Z`, `+hh:mm` or `-hh:mm`.
    let zone_at = time.find(['Z', 'z', '+', '-']).unwrap_or(time.len());
    let (clock, zone) = time.split_at(zone_at);
    let clock = clock.split_once('.').map_or(clock, |(whole, _fraction)| whole);
    let mut hms = clock.splitn(3, ':').map(str::parse::<i64>);
    let (hour, minute, second) = (hms.next()?.ok()?, hms.next()?.ok()?, hms.next()?.ok()?);
    if !(0..24).contains(&hour) || !(0..60).contains(&minute) || !(0..=60).contains(&second) {
        return None;
    }
    let offset = match zone {
        "" | "Z" | "z" => 0,
        _ => {
            let sign = if zone.starts_with('-') { -1 } else { 1 };
            let (h, m) = zone[1..].split_once(':')?;
            sign * (h.parse::<i64>().ok()? * 3600 + m.parse::<i64>().ok()? * 60)
        }
    };
    let total = days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second - offset;
    u64::try_from(total).ok()
}

/// `1 h 02 min` or `4 min 05 s` for a span in seconds.
#[must_use]
pub fn span(secs: u64) -> String {
    if secs >= 3600 {
        format!("{} h {:02} min", secs / 3600, secs % 3600 / 60)
    } else {
        format!("{} min {:02} s", secs / 60, secs % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_known_instants() {
        assert_eq!(rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339(1_795_787_495), "2026-11-27T13:51:35Z");
    }

    #[test]
    fn parses_what_it_formats_across_leap_years() {
        for secs in [0, 86_399, 951_782_400, 1_709_251_199, 1_795_787_495, 4_102_444_800] {
            assert_eq!(parse_rfc3339(&rfc3339(secs)), Some(secs), "{secs}");
        }
    }

    #[test]
    fn parses_fractions_and_offsets() {
        assert_eq!(parse_rfc3339("2026-11-27T13:51:35.482Z"), Some(1_795_787_495));
        assert_eq!(parse_rfc3339("2026-11-27T14:51:35+01:00"), Some(1_795_787_495));
        assert_eq!(parse_rfc3339("2026-11-27T05:51:35-08:00"), Some(1_795_787_495));
    }

    #[test]
    fn rejects_text_that_is_not_a_timestamp() {
        for bad in [
            "",
            "yesterday",
            "2026-13-01T00:00:00Z",
            "2026-11-27",
            "2026-11-27T25:00:00Z",
        ] {
            assert_eq!(parse_rfc3339(bad), None, "{bad}");
        }
    }

    #[test]
    fn spans_read_naturally() {
        assert_eq!(span(245), "4 min 05 s");
        assert_eq!(span(3725), "1 h 02 min");
    }
}
