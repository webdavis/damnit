mod words;

pub type Date = jiff::civil::Date;
pub type Timestamp = jiff::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum When {
    Day(Date),
    At(jiff::Zoned),
}

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

    pub fn instant(&self, zone: &jiff::tz::TimeZone) -> Option<Timestamp> {
        match self {
            When::At(z) => Some(z.timestamp()),
            When::Day(d) => d.to_zoned(zone.clone()).ok().map(|z| z.timestamp()),
        }
    }

    pub fn parse_human(
        text: &str,
        today: Date,
        tz: &jiff::tz::TimeZone,
    ) -> Result<When, WhenError> {
        let text = text.trim();
        if let Some(day) = words::day(text, today) {
            return Ok(When::Day(day));
        }
        let names_a_zone = text.contains('[');
        let names_a_local_time = text.contains('T');
        if names_a_zone {
            return text
                .parse::<jiff::Zoned>()
                .map(When::At)
                .map_err(|e| friendly_hint_with_cause(text, &e));
        }
        if names_a_local_time {
            return text
                .parse::<jiff::civil::DateTime>()
                .map_err(|e| friendly_hint_with_cause(text, &e))?
                .to_zoned(tz.clone())
                .map(When::At)
                .map_err(|e| friendly_hint_with_cause(text, &e));
        }
        text.parse::<Date>()
            .map(When::Day)
            .map_err(|e| friendly_hint_with_cause(text, &e))
    }

    pub fn to_text(&self) -> String {
        match self {
            When::Day(d) => d.to_string(),
            When::At(z) => z.to_string(),
        }
    }
}

fn friendly_hint_with_cause(text: &str, cause: &dyn std::fmt::Display) -> WhenError {
    WhenError(format!(
        "cannot read {text:?} as a date; use {}, YYYY-MM-DD or YYYY-MM-DDTHH:MM: {cause}",
        words::ACCEPTED_FORMS
    ))
}

#[cfg(test)]
mod tests;
