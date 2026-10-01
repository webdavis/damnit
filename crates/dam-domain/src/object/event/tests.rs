use jiff::civil::date;
use jiff::tz::TimeZone;

use super::*;
use crate::{Oid, When};

fn oid() -> Oid {
    Oid::generate(&mut |b: &mut [u8]| b.fill(7))
}

fn timed() -> Event {
    let start = "2026-09-25T14:00:00-04:00[America/New_York]"
        .parse()
        .unwrap();
    let end = "2026-09-25T14:30:00-04:00[America/New_York]"
        .parse()
        .unwrap();
    Event::new(oid(), "standup", When::At(start), When::At(end))
}

fn attendee(is_self: bool, response: ResponseStatus) -> Attendee {
    Attendee {
        email: "x@example.com".into(),
        response,
        is_self,
    }
}

#[test]
fn a_timed_span_is_its_own_two_instants_whatever_zone_is_given() {
    let (start, end) = timed().span(&TimeZone::UTC).unwrap();
    assert_eq!(
        (start.as_second(), end.as_second()),
        (1_790_359_200, 1_790_361_000)
    );
}

#[test]
fn an_all_day_span_runs_midnight_to_midnight_in_the_zone_given() {
    let day = Event::new(
        oid(),
        "offsite",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    );
    let (start, end) = day.span(&TimeZone::fixed(jiff::tz::offset(-4))).unwrap();
    assert_eq!(
        (start.as_second(), end.as_second()),
        (1_790_308_800, 1_790_395_200)
    );
}

#[test]
fn an_all_day_span_on_a_spring_forward_day_is_twenty_three_hours() {
    let zone = TimeZone::posix("EST5EDT,M3.2.0,M11.1.0").unwrap();
    let day = Event::new(
        oid(),
        "dst",
        When::Day(date(2026, 3, 8)),
        When::Day(date(2026, 3, 9)),
    );
    let (start, end) = day.span(&zone).unwrap();
    assert_eq!(end.as_second() - start.as_second(), 23 * 3600);
}

#[test]
fn a_confirmed_busy_event_holds_time_and_so_does_a_tentative_one() {
    assert!(timed().holds_time());
    let mut tentative = timed();
    tentative.status = EventStatus::Tentative;
    tentative.attendees = vec![attendee(true, ResponseStatus::NeedsAction)];
    assert!(tentative.holds_time());
}

#[test]
fn a_cancelled_or_free_event_holds_no_time() {
    let mut cancelled = timed();
    cancelled.status = EventStatus::Cancelled;
    assert!(!cancelled.holds_time());
    let mut free = timed();
    free.transparency = Transparency::Free;
    assert!(!free.holds_time());
}

#[test]
fn an_event_the_calendars_own_attendee_declined_holds_no_time() {
    let mut declined = timed();
    declined.attendees = vec![
        attendee(false, ResponseStatus::Accepted),
        attendee(true, ResponseStatus::Declined),
    ];
    assert!(!declined.holds_time());
    let mut someone_else = timed();
    someone_else.attendees = vec![attendee(false, ResponseStatus::Declined)];
    assert!(someone_else.holds_time(), "only the owner's answer counts");
}
