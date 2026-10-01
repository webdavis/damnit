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
