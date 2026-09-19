pub type Date = jiff::civil::Date;
pub type Timestamp = jiff::Timestamp;

/// A point on the calendar: a whole day, or an instant in a named zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum When {
    Day(Date),
    At(jiff::Zoned),
}

/// `When::parse_human` failed to read the given text as a date.
#[derive(Debug, PartialEq, Eq)]
pub struct WhenError(pub String);

impl std::fmt::Display for WhenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WhenError {}

impl When {
    pub fn date(&self) -> Date {
        match self {
            When::Day(d) => *d,
            When::At(z) => z.date(),
        }
    }

    pub fn is_all_day(&self) -> bool {
        matches!(self, When::Day(_))
    }

    /// `today`, `tomorrow`, `YYYY-MM-DD`, `YYYY-MM-DDTHH:MM[:SS]` in `tz`, or a full zoned timestamp.
    ///
    /// `Date` and `DateTime` both parse a string carrying more than they need (dropping any
    /// trailing time or zone), so the text shape picks the parser instead of trying each in
    /// turn: a `[zone]` suffix means a full zoned timestamp, a bare `T` means a local time, and
    /// anything else is a plain date.
    pub fn parse_human(
        text: &str,
        today: Date,
        tz: &jiff::tz::TimeZone,
    ) -> Result<When, WhenError> {
        let text = text.trim();
        match text {
            "today" => return Ok(When::Day(today)),
            "tomorrow" => {
                return today
                    .tomorrow()
                    .map(When::Day)
                    .map_err(|e| WhenError(e.to_string()));
            }
            _ => {}
        }
        if text.contains('[') {
            return text
                .parse::<jiff::Zoned>()
                .map(When::At)
                .map_err(|e| hint(text, &e));
        }
        if text.contains('T') {
            return text
                .parse::<jiff::civil::DateTime>()
                .map_err(|e| hint(text, &e))?
                .to_zoned(tz.clone())
                .map(When::At)
                .map_err(|e| hint(text, &e));
        }
        text.parse::<Date>()
            .map(When::Day)
            .map_err(|e| hint(text, &e))
    }

    /// `2026-09-25` or the zoned form; `parse_human` reads both back.
    pub fn to_text(&self) -> String {
        match self {
            When::Day(d) => d.to_string(),
            When::At(z) => z.to_string(),
        }
    }
}

/// The same friendly hint on every `parse_human` failure, whatever shape the
/// text was read as, with jiff's own reason appended.
fn hint(text: &str, cause: &dyn std::fmt::Display) -> WhenError {
    WhenError(format!(
        "cannot read {text:?} as a date; use today, tomorrow, YYYY-MM-DD or YYYY-MM-DDTHH:MM: {cause}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn a_day_is_all_day_and_its_date_is_itself() {
        let w = When::Day(date(2026, 9, 25));
        assert!(w.is_all_day());
        assert_eq!(w.date(), date(2026, 9, 25));
    }

    #[test]
    fn an_instant_reports_its_local_date() {
        let z = date(2026, 9, 25)
            .at(14, 0, 0, 0)
            .in_tz("America/Denver")
            .unwrap();
        let w = When::At(z);
        assert!(!w.is_all_day());
        assert_eq!(w.date(), date(2026, 9, 25));
    }

    #[test]
    fn parse_human_reads_relative_days_dates_and_local_times() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        assert_eq!(
            When::parse_human("today", today, &tz).unwrap(),
            When::Day(today)
        );
        assert_eq!(
            When::parse_human("tomorrow", today, &tz).unwrap(),
            When::Day(date(2026, 9, 19))
        );
        assert_eq!(
            When::parse_human("2026-09-25", today, &tz).unwrap(),
            When::Day(date(2026, 9, 25))
        );
        let at = When::parse_human("2026-09-25T14:00", today, &tz).unwrap();
        assert_eq!(at.date(), date(2026, 9, 25));
        assert!(!at.is_all_day());
        assert!(When::parse_human("next tuesday", today, &tz).is_err());
    }

    #[test]
    fn to_text_round_trips_through_parse_human() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        for text in ["2026-09-25", "2026-09-25T14:00"] {
            let w = When::parse_human(text, today, &tz).unwrap();
            assert_eq!(When::parse_human(&w.to_text(), today, &tz).unwrap(), w);
        }
    }

    #[test]
    fn parse_human_reads_a_full_zoned_timestamp_as_an_instant() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        let at = When::parse_human("2026-09-25T09:00:00[America/Denver]", today, &tz).unwrap();
        assert!(!at.is_all_day());
        assert!(matches!(at, When::At(_)));
    }

    #[test]
    fn a_malformed_plain_date_gets_the_friendly_hint() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        let err = When::parse_human("next tuesday", today, &tz).unwrap_err();
        assert!(err.0.contains("cannot read"), "{}", err.0);
    }

    #[test]
    fn a_malformed_local_time_gets_the_friendly_hint() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        let err = When::parse_human("2026-13-45T09:00", today, &tz).unwrap_err();
        assert!(err.0.contains("cannot read"), "{}", err.0);
    }

    #[test]
    fn a_malformed_zoned_timestamp_gets_the_friendly_hint() {
        let today = date(2026, 9, 18);
        let tz = jiff::tz::TimeZone::UTC;
        let err = When::parse_human("2026-09-25T09:00[Nowhere/Nope]", today, &tz).unwrap_err();
        assert!(err.0.contains("cannot read"), "{}", err.0);
    }
}
