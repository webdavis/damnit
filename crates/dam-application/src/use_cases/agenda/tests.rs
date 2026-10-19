use jiff::civil::date;
use jiff::tz::TimeZone;

use super::*;
use crate::testing::{FixedClock, Scenario, oid};
use dam_domain::{EventStatus, Object, Path, Task, When};

fn at(text: &str) -> Timestamp {
    text.parse().unwrap()
}

fn event(n: u8, subject: &str, start: &str, end: &str) -> Event {
    let zoned = |t: &str| When::At(at(t).to_zoned(TimeZone::UTC));
    Event::new(oid(n), subject, zoned(start), zoned(end))
}

fn on_the_25th() -> Scenario {
    let mut s = Scenario::default();
    s.fixture.clock = FixedClock(date(2026, 9, 25));
    s
}

fn run(s: &Scenario, query: Option<&str>, from: &str, to: &str) -> Vec<Scheduled> {
    s.dam()
        .agenda(
            query,
            Window {
                from: at(from),
                to: at(to),
            },
            &TimeZone::UTC,
        )
        .unwrap()
}

fn subjects(found: &[Scheduled]) -> Vec<&str> {
    found
        .iter()
        .map(|s| s.event.base.subject.as_str())
        .collect()
}

#[test]
fn an_event_in_progress_when_the_window_opens_is_listed() {
    let s = on_the_25th();
    s.working(Object::Event(event(
        1,
        "running",
        "2026-09-25T09:00:00Z",
        "2026-09-25T11:00:00Z",
    )));
    let found = run(&s, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z");
    assert_eq!(subjects(&found), vec!["running"]);
    assert_eq!(
        (found[0].start, found[0].end, found[0].busy),
        (at("2026-09-25T09:00:00Z"), at("2026-09-25T11:00:00Z"), true)
    );
}

#[test]
fn the_window_is_half_open_on_both_sides() {
    let s = on_the_25th();
    s.working(Object::Event(event(
        1,
        "ended at open",
        "2026-09-25T08:00:00Z",
        "2026-09-25T10:00:00Z",
    )));
    s.working(Object::Event(event(
        2,
        "starts at close",
        "2026-09-26T10:00:00Z",
        "2026-09-26T11:00:00Z",
    )));
    assert!(run(&s, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z").is_empty());
}

#[test]
fn events_are_ordered_by_start_and_a_cancelled_one_is_listed_as_not_busy() {
    let s = on_the_25th();
    let mut gone = event(1, "gone", "2026-09-25T15:00:00Z", "2026-09-25T16:00:00Z");
    gone.status = EventStatus::Cancelled;
    s.working(Object::Event(gone));
    s.working(Object::Event(event(
        2,
        "first",
        "2026-09-25T12:00:00Z",
        "2026-09-25T13:00:00Z",
    )));
    let found = run(&s, None, "2026-09-25T00:00:00Z", "2026-09-26T00:00:00Z");
    assert_eq!(subjects(&found), vec!["first", "gone"]);
    assert!(found[0].busy && !found[1].busy);
}

#[test]
fn tasks_are_never_listed_and_a_query_narrows_the_events() {
    let s = on_the_25th();
    let mut due = Task::new(oid(3), "a task due now");
    due.due = Some(When::Day(date(2026, 9, 25)));
    s.working(Object::Task(due));
    let mut work = event(4, "work", "2026-09-25T12:00:00Z", "2026-09-25T13:00:00Z");
    work.base.path = Path::parse("work").unwrap();
    s.working(Object::Event(work));
    s.working(Object::Event(event(
        5,
        "home",
        "2026-09-25T12:00:00Z",
        "2026-09-25T13:00:00Z",
    )));
    assert_eq!(
        subjects(&run(
            &s,
            None,
            "2026-09-25T00:00:00Z",
            "2026-09-26T00:00:00Z"
        )),
        vec!["work", "home"]
    );
    assert_eq!(
        subjects(&run(
            &s,
            Some("path:work/"),
            "2026-09-25T00:00:00Z",
            "2026-09-26T00:00:00Z"
        )),
        vec!["work"]
    );
}

#[test]
fn an_all_day_event_runs_midnight_to_midnight_in_the_zone_given() {
    let s = on_the_25th();
    let day = Event::new(
        oid(6),
        "offsite",
        When::Day(date(2026, 9, 25)),
        When::Day(date(2026, 9, 26)),
    );
    s.working(Object::Event(day));
    let window = Window {
        from: at("2026-09-25T00:00:00Z"),
        to: at("2026-09-27T00:00:00Z"),
    };
    let zone = TimeZone::fixed(jiff::tz::offset(-4));
    let found = s.dam().agenda(None, window, &zone).unwrap();
    assert_eq!(
        (found[0].start, found[0].end),
        (at("2026-09-25T04:00:00Z"), at("2026-09-26T04:00:00Z"))
    );
}
