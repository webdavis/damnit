//! Events as intervals over a window, each with whether it holds time: the
//! reading a scheduler takes busy times from.

use dam_domain::{Event, Object, Timestamp};

use crate::config::Config;
use crate::errors::UseCaseError;
use crate::ports::{Clock, ObjectRepository};
use crate::use_cases::list::list;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Window {
    pub from: Timestamp,
    pub to: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Scheduled {
    pub event: Event,
    pub start: Timestamp,
    pub end: Timestamp,
    pub busy: bool,
}

/// Every event the query admits whose span overlaps the window (start before
/// `to`, end after `from`), busy or not, ordered by start and then oid. Tasks
/// are never listed.
pub fn agenda(
    objects: &dyn ObjectRepository,
    clock: &dyn Clock,
    config: &Config,
    query: Option<&str>,
    window: Window,
    zone: &jiff::tz::TimeZone,
) -> Result<Vec<Scheduled>, UseCaseError> {
    let mut found: Vec<Scheduled> = list(objects, clock, config, query)?
        .into_iter()
        .filter_map(|o| match o {
            Object::Event(event) => {
                let (start, end) = event.span(zone)?;
                (start < window.to && end > window.from).then(|| Scheduled {
                    busy: event.holds_time(),
                    event,
                    start,
                    end,
                })
            }
            Object::Task(_) => None,
        })
        .collect();
    found.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| a.event.base.oid.cmp(&b.event.base.oid))
    });
    Ok(found)
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;
    use jiff::tz::TimeZone;

    use super::*;
    use crate::testing::prelude::*;
    use crate::testing::{FixedClock, MemoryStore, oid};
    use dam_domain::{EventStatus, Object, Path, Task, When};

    fn at(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn event(n: u8, subject: &str, start: &str, end: &str) -> Event {
        let zoned = |t: &str| When::At(at(t).to_zoned(TimeZone::UTC));
        Event::new(oid(n), subject, zoned(start), zoned(end))
    }

    fn run(store: &MemoryStore, query: Option<&str>, from: &str, to: &str) -> Vec<Scheduled> {
        agenda(
            store,
            &FixedClock(date(2026, 9, 25)),
            &Config::default(),
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
        let store = MemoryStore::new();
        store
            .put(&Object::Event(event(
                1,
                "running",
                "2026-09-25T09:00:00Z",
                "2026-09-25T11:00:00Z",
            )))
            .unwrap();
        let found = run(&store, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z");
        assert_eq!(subjects(&found), vec!["running"]);
        assert_eq!(
            (found[0].start, found[0].end, found[0].busy),
            (at("2026-09-25T09:00:00Z"), at("2026-09-25T11:00:00Z"), true)
        );
    }

    #[test]
    fn the_window_is_half_open_on_both_sides() {
        let store = MemoryStore::new();
        store
            .put(&Object::Event(event(
                1,
                "ended at open",
                "2026-09-25T08:00:00Z",
                "2026-09-25T10:00:00Z",
            )))
            .unwrap();
        store
            .put(&Object::Event(event(
                2,
                "starts at close",
                "2026-09-26T10:00:00Z",
                "2026-09-26T11:00:00Z",
            )))
            .unwrap();
        assert!(run(&store, None, "2026-09-25T10:00:00Z", "2026-09-26T10:00:00Z").is_empty());
    }

    #[test]
    fn events_are_ordered_by_start_and_a_cancelled_one_is_listed_as_not_busy() {
        let store = MemoryStore::new();
        let mut gone = event(1, "gone", "2026-09-25T15:00:00Z", "2026-09-25T16:00:00Z");
        gone.status = EventStatus::Cancelled;
        store.put(&Object::Event(gone)).unwrap();
        store
            .put(&Object::Event(event(
                2,
                "first",
                "2026-09-25T12:00:00Z",
                "2026-09-25T13:00:00Z",
            )))
            .unwrap();
        let found = run(&store, None, "2026-09-25T00:00:00Z", "2026-09-26T00:00:00Z");
        assert_eq!(subjects(&found), vec!["first", "gone"]);
        assert!(found[0].busy && !found[1].busy);
    }

    #[test]
    fn tasks_are_never_listed_and_a_query_narrows_the_events() {
        let store = MemoryStore::new();
        let mut due = Task::new(oid(3), "a task due now");
        due.due = Some(When::Day(date(2026, 9, 25)));
        store.put(&Object::Task(due)).unwrap();
        let mut work = event(4, "work", "2026-09-25T12:00:00Z", "2026-09-25T13:00:00Z");
        work.base.path = Path::parse("work").unwrap();
        store.put(&Object::Event(work)).unwrap();
        store
            .put(&Object::Event(event(
                5,
                "home",
                "2026-09-25T12:00:00Z",
                "2026-09-25T13:00:00Z",
            )))
            .unwrap();
        assert_eq!(
            subjects(&run(
                &store,
                None,
                "2026-09-25T00:00:00Z",
                "2026-09-26T00:00:00Z"
            )),
            vec!["work", "home"]
        );
        assert_eq!(
            subjects(&run(
                &store,
                Some("path:work/"),
                "2026-09-25T00:00:00Z",
                "2026-09-26T00:00:00Z"
            )),
            vec!["work"]
        );
    }

    /// An all-day event is placed by the zone the reader passes, midnight to
    /// midnight there.
    #[test]
    fn an_all_day_event_runs_midnight_to_midnight_in_the_zone_given() {
        let store = MemoryStore::new();
        let day = Event::new(
            oid(6),
            "offsite",
            When::Day(date(2026, 9, 25)),
            When::Day(date(2026, 9, 26)),
        );
        store.put(&Object::Event(day)).unwrap();
        let window = Window {
            from: at("2026-09-25T00:00:00Z"),
            to: at("2026-09-27T00:00:00Z"),
        };
        let zone = TimeZone::fixed(jiff::tz::offset(-4));
        let found = agenda(
            &store,
            &FixedClock(date(2026, 9, 25)),
            &Config::default(),
            None,
            window,
            &zone,
        )
        .unwrap();
        assert_eq!(
            (found[0].start, found[0].end),
            (at("2026-09-25T04:00:00Z"), at("2026-09-26T04:00:00Z"))
        );
    }
}
