use dam_domain::{Completion, Event, Filter, FilterTable, Object, Path, TableValue, Task, When};
use jiff::civil::date;
use jiff::tz::TimeZone;

use super::{Agenda, Window};
use crate::testing::{Scenario, oid};

fn due(n: u8, subject: &str, when: When) -> Object {
    let mut t = Task::new(oid(n), subject);
    t.due = Some(when);
    Object::Task(t)
}

fn on(day: i8) -> When {
    When::Day(date(2026, 9, day))
}

fn meeting(n: u8, subject: &str, start: &str, end: &str) -> Object {
    let at = |t: &str| When::At(t.parse::<jiff::Zoned>().unwrap());
    Object::Event(Event::new(oid(n), subject, at(start), at(end)))
}

fn kind(word: &str) -> Filter {
    Filter::read(&FilterTable {
        fields: [("kind".to_string(), TableValue::One(word.into()))].into(),
        any: Vec::new(),
    })
    .unwrap()
}

fn two_days() -> Window {
    Window {
        from: "2026-09-25T00:00:00Z".parse().unwrap(),
        to: "2026-09-27T00:00:00Z".parse().unwrap(),
    }
}

fn agenda(s: &Scenario, filter: &Filter, zone: &TimeZone) -> Agenda {
    s.dam()
        .agenda(filter, &Path::root(), &[], two_days(), zone)
        .unwrap()
}

fn task_days(found: &Agenda) -> Vec<(String, String)> {
    found
        .tasks
        .iter()
        .map(|d| (d.task.base.subject.clone(), d.day.to_string()))
        .collect()
}

fn store() -> Scenario {
    let s = Scenario::default();
    s.working(due(1, "late", on(26)));
    s.working(due(2, "overdue", on(24)));
    s.working(due(3, "first", on(25)));
    s.working(due(4, "after", on(27)));
    s.working(Object::Task(Task::new(oid(5), "undated")));
    let mut done = Task::new(oid(6), "finished");
    done.due = Some(on(25));
    done.completion = Completion::Done { at: None };
    s.working(Object::Task(done));
    s.working(meeting(
        7,
        "review",
        "2026-09-25T14:00:00+00:00[UTC]",
        "2026-09-25T15:00:00+00:00[UTC]",
    ));
    s
}

#[test]
fn kind_any_lists_the_open_tasks_due_on_a_day_of_the_window_by_day_beside_the_events() {
    let found = agenda(&store(), &kind("any"), &TimeZone::UTC);
    assert_eq!(
        task_days(&found),
        [
            ("first".into(), "2026-09-25".into()),
            ("late".into(), "2026-09-26".into())
        ]
    );
    let events: Vec<_> = found.events.iter().map(|s| s.day.to_string()).collect();
    assert_eq!(events, ["2026-09-25"]);
}

#[test]
fn kind_task_lists_the_tasks_alone_and_no_kind_lists_the_events_alone() {
    let s = store();
    let tasks = agenda(&s, &kind("task"), &TimeZone::UTC);
    assert!(tasks.events.is_empty());
    assert_eq!(tasks.tasks.len(), 2);
    let events = agenda(&s, &Filter::default(), &TimeZone::UTC);
    assert!(events.tasks.is_empty());
    assert_eq!(events.events.len(), 1);
}

#[test]
fn a_timed_due_falls_on_its_day_in_the_persons_zone() {
    let s = Scenario::default();
    let at = "2026-09-27T02:00:00+00:00[UTC]"
        .parse::<jiff::Zoned>()
        .unwrap();
    s.working(due(1, "evening call", When::At(at)));
    let west = TimeZone::fixed(jiff::tz::offset(-4));
    assert_eq!(
        task_days(&agenda(&s, &kind("task"), &west)),
        [("evening call".into(), "2026-09-26".into())]
    );
    assert!(agenda(&s, &kind("task"), &TimeZone::UTC).tasks.is_empty());
}

#[test]
fn the_window_names_its_first_and_last_day_in_the_persons_zone() {
    let days = two_days().days(&TimeZone::UTC);
    assert_eq!(days, (date(2026, 9, 25), date(2026, 9, 26)));
    let west = TimeZone::fixed(jiff::tz::offset(-4));
    assert_eq!(
        two_days().days(&west),
        (date(2026, 9, 24), date(2026, 9, 26))
    );
}
