//! Utilities for working with times and durations.

use chrono::TimeDelta;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use std::time::Duration;

/**
Convert a `Duration` to a human readable string if possible.
*/
pub fn human_readable_duration(duration: Duration) -> Result<String> {
    let timedelta = TimeDelta::from_std(duration)?;
    human_readable_timedelta(timedelta)
}

/// Convert a `TimeDelta` into a human readable string if possible.
fn human_readable_timedelta(mut timedelta: TimeDelta) -> Result<String> {
    // Output string to build.
    let mut output = String::new();
    // Whether we have already added any actual numbers to the output string.
    let mut numbers_added = false;

    // Prepend with single `-` if negative time.
    if timedelta.abs() != timedelta {
        output.push('-');
        timedelta = timedelta.abs();
    }

    for (count_fn, time_unit, truncate_fn) in [
        (
            TimeDelta::num_weeks as fn(&TimeDelta) -> i64,
            "w",
            TimeDelta::try_weeks as fn(i64) -> Option<TimeDelta>,
        ),
        (
            TimeDelta::num_days as fn(&TimeDelta) -> i64,
            "d",
            TimeDelta::try_days as fn(i64) -> Option<TimeDelta>,
        ),
        (
            TimeDelta::num_hours as fn(&TimeDelta) -> i64,
            "h",
            TimeDelta::try_hours as fn(i64) -> Option<TimeDelta>,
        ),
        (
            TimeDelta::num_minutes as fn(&TimeDelta) -> i64,
            "m",
            TimeDelta::try_minutes as fn(i64) -> Option<TimeDelta>,
        ),
        (
            TimeDelta::num_seconds as fn(&TimeDelta) -> i64,
            "s",
            TimeDelta::try_seconds as fn(i64) -> Option<TimeDelta>,
        ),
        (
            TimeDelta::num_milliseconds as fn(&TimeDelta) -> i64,
            "ms",
            TimeDelta::try_milliseconds as fn(i64) -> Option<TimeDelta>,
        ),
    ] {
        let count = count_fn(&timedelta);
        // Don't prepend e.g. `0w 0d 0h ...`
        if count == 0 {
            continue;
        }

        // Don't give millisecond precision unless we're <1s
        if time_unit == "ms" && numbers_added {
            break;
        }

        // If there are already values in the output string, add a space.
        if numbers_added {
            output.push(' ');
        }

        numbers_added = true;
        output.push_str(&count.to_string());
        output.push_str(time_unit);

        // Remove the amount we just added to the output string (we want `1m 10s` not `1m 70s`).
        timedelta -= truncate_fn(count).ok_or_else(|| eyre!("Failed to truncate {time_unit}"))?;
    }

    // Add microseconds if we took less than 1ms.
    if !numbers_added
        && let Some(microseconds) = timedelta.num_microseconds()
        && microseconds != 0
    {
        numbers_added = true;
        output.push_str(&microseconds.to_string());
        output.push_str("µs");
    }

    // Add nanoseconds if we took less than 1µs.
    if !numbers_added && let Some(nanoseconds) = timedelta.num_nanoseconds() {
        output.push_str(&nanoseconds.to_string());
        output.push_str("ns");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::utils::time::human_readable_duration;
    use crate::utils::time::human_readable_timedelta;
    use chrono::TimeDelta;
    use color_eyre::Result;
    use std::time::Duration;
    use testutils::ensure_eq;

    /// Number of seconds in a minute.
    const MINUTES: u64 = 60;
    /// Number of seconds in an hour.
    const HOURS: u64 = MINUTES * 60;
    /// Number of seconds in an day.
    const DAYS: u64 = HOURS * 24;
    /// Number of seconds in an week.
    const WEEKS: u64 = DAYS * 7;

    #[test]
    fn test_human_readable_duration() -> Result<()> {
        // Check each time unit type.
        ensure_eq!("0ns", human_readable_duration(Duration::from_nanos(0))?);
        ensure_eq!("5ns", human_readable_duration(Duration::from_nanos(5))?);
        ensure_eq!("5µs", human_readable_duration(Duration::from_nanos(5999))?);
        ensure_eq!("5ms", human_readable_duration(Duration::from_micros(5678))?);
        ensure_eq!("10s", human_readable_duration(Duration::from_secs(10))?);
        ensure_eq!("5m", human_readable_duration(Duration::from_secs(300))?);
        ensure_eq!(
            "6h",
            human_readable_duration(Duration::from_secs(6 * HOURS))?
        );
        ensure_eq!(
            "5d",
            human_readable_duration(Duration::from_secs(5 * DAYS))?
        );
        ensure_eq!(
            "1w",
            human_readable_duration(Duration::from_secs(7 * DAYS))?
        );
        ensure_eq!(
            "4w",
            human_readable_duration(Duration::from_secs(4 * WEEKS))?
        );

        // minutes + millis/micros/nanos: only print minutes.
        ensure_eq!(
            "5m",
            human_readable_duration(Duration::from_nanos(300_123_456_789))?
        );

        ensure_eq!(
            "5m 12s",
            human_readable_duration(Duration::from_nanos(312_123_456_789))?
        );

        ensure_eq!(
            "17m 59s",
            human_readable_duration(Duration::from_secs(1079))?
        );

        // Weeks + s, s should be printed
        ensure_eq!(
            "28w 20s",
            human_readable_duration(Duration::from_secs(28 * WEEKS) + Duration::from_secs(20))?,
        );

        // Weeks + ms, ms should be ignored
        ensure_eq!(
            "28w",
            human_readable_duration(Duration::from_secs(28 * WEEKS) + Duration::from_millis(543))?,
        );

        ensure_eq!(
            "5w 2d 5s",
            human_readable_duration(
                Duration::from_secs(5 * WEEKS + 2 * DAYS) + Duration::from_secs(5)
            )?,
        );

        ensure_eq!(
            "5w 2d 4h 59m 50s",
            human_readable_duration(Duration::from_secs(
                5 * WEEKS + 2 * DAYS + 4 * HOURS + 59 * MINUTES + 50
            ))?,
        );

        Ok(())
    }

    #[test]
    fn test_human_readable_timedelta() -> Result<()> {
        ensure_eq!(
            "-2m 14s",
            human_readable_timedelta(TimeDelta::seconds(-134_i64))?
        );
        ensure_eq!(
            "-2ns",
            human_readable_timedelta(TimeDelta::nanoseconds(-2_i64))?
        );
        Ok(())
    }
}
