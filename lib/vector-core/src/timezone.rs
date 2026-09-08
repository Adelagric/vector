use std::{error::Error, fmt};

use vrl::compiler::TimeZone;

/// An error returned when the system local time zone cannot be loaded.
#[derive(Debug, Eq, PartialEq)]
pub struct LocalTimeZoneError {
    source: String,
}

impl fmt::Display for LocalTimeZoneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Unable to load the system local time zone: {}. Set `timezone` explicitly to a valid IANA time zone or configure system time zone data.",
            self.source
        )
    }
}

impl Error for LocalTimeZoneError {}

/// Verifies that a configured time zone can be loaded.
///
/// Named time zones are embedded in the binary and require no system lookup.
/// The system local time zone is loaded fallibly so that callers do not inherit
/// Chrono's silent fallback to UTC.
///
/// # Errors
///
/// Returns an error when `timezone` is local and the system local time zone
/// cannot be loaded.
pub fn validate_timezone(timezone: TimeZone) -> Result<(), LocalTimeZoneError> {
    validate_timezone_with(timezone, || {
        jiff::tz::TimeZone::try_system()
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

fn validate_timezone_with(
    timezone: TimeZone,
    validate_local: impl FnOnce() -> Result<(), String>,
) -> Result<(), LocalTimeZoneError> {
    if !matches!(timezone, TimeZone::Local) {
        return Ok(());
    }

    validate_local().map_err(|source| LocalTimeZoneError { source })
}

#[cfg(test)]
mod tests {
    use chrono_tz::Tz;

    use super::*;

    #[test]
    fn named_timezone_does_not_load_system_timezone() {
        validate_timezone_with(TimeZone::Named(Tz::UTC), || {
            panic!("named time zones do not need system timezone data")
        })
        .unwrap();
    }

    #[test]
    fn unavailable_local_timezone_is_rejected() {
        let error = validate_timezone_with(TimeZone::Local, || {
            Err("timezone data is unavailable".into())
        })
        .unwrap_err();

        assert_eq!(
            error,
            LocalTimeZoneError {
                source: "timezone data is unavailable".into()
            }
        );
        assert_eq!(
            error.to_string(),
            "Unable to load the system local time zone: timezone data is unavailable. Set `timezone` explicitly to a valid IANA time zone or configure system time zone data."
        );
    }
}
