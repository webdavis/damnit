//! The remote's address names the calendars it reads: `gcal::` is the
//! primary calendar, `gcal::primary,team@group.calendar.google.com` two.

pub fn calendars(address: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in address
        .split(',')
        .map(str::trim)
        .filter(|id| !id.is_empty())
    {
        if !out.iter().any(|seen| seen == id) {
            out.push(id.to_string());
        }
    }
    if out.is_empty() {
        out.push("primary".into());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_address_reads_the_primary_calendar() {
        assert_eq!(calendars(""), vec!["primary"]);
        assert_eq!(calendars(" , "), vec!["primary"]);
    }

    #[test]
    fn named_calendars_are_read_in_order_once_each() {
        assert_eq!(
            calendars(
                "primary, team@group.calendar.google.com,primary,en.usa#holiday@group.v.calendar.google.com"
            ),
            vec![
                "primary",
                "team@group.calendar.google.com",
                "en.usa#holiday@group.v.calendar.google.com"
            ]
        );
    }
}
