use dam_domain::{Completion, Event, Object, Order, Path, Problem, When};
use jiff::civil::date;

use super::SelectError;
use super::tests::{filter, subjects};
use crate::config::Config;
use crate::errors::UseCaseError;
use crate::testing::{Scenario, oid, saved, task_in};
use crate::use_cases::agenda::Window;
use crate::use_cases::each::Selector;

fn a_late_task_a_finished_one_and_all_hands() -> Scenario {
    let s = Scenario::default().configured(Config {
        filters: vec![
            saved("today", &[("due", "..today")]),
            saved("allhands", &[("kind", "event"), ("subject", "all hands")]),
            saved("meeting", &[("subject", "all hands")]),
            saved("starting", &[("start", "today")]),
            saved("closed", &[("completed_at", "today")]),
            saved("open", &[("done", "false")]),
        ],
        ..Config::default()
    });
    let mut late = task_in(1, "late", "work");
    late.due = Some(When::Day(date(2026, 9, 17)));
    let mut finished = task_in(2, "finished", "work");
    finished.due = Some(When::Day(date(2026, 9, 17)));
    finished.completion = Completion::Done {
        at: Some("2026-09-18T10:00:00Z".parse().unwrap()),
    };
    s.working(Object::Task(late));
    s.working(Object::Task(finished));
    let day = |d| When::Day(date(2026, 9, d));
    s.working(Object::Event(Event::new(
        oid(3),
        "all hands",
        day(18),
        day(19),
    )));
    s
}

fn listed(s: &Scenario, fields: &[(&str, &str)]) -> Result<Vec<String>, UseCaseError> {
    let out = s
        .dam()
        .list_selected(&filter(fields), &Order::default(), &Path::root(), &[])?;
    let mut names: Vec<String> = subjects(&out).into_iter().map(String::from).collect();
    names.sort();
    Ok(names)
}

#[test]
fn a_typed_flag_narrows_what_the_filter_selects_before_any_default() {
    let s = a_late_task_a_finished_one_and_all_hands();
    assert_eq!(listed(&s, &[("filter", "today")]).unwrap(), ["late"]);
    assert_eq!(
        listed(&s, &[("filter", "today"), ("done", "true")]).unwrap(),
        ["finished"]
    );
    assert_eq!(
        listed(&s, &[("filter", "today"), ("done", "any")]).unwrap(),
        ["finished", "late"]
    );
}

#[test]
fn the_fields_a_filter_names_set_the_defaults_as_if_typed() {
    let s = a_late_task_a_finished_one_and_all_hands();
    assert_eq!(
        listed(&s, &[("filter", "allhands")]).unwrap(),
        ["all hands"]
    );
    assert_eq!(listed(&s, &[("filter", "closed")]).unwrap(), ["finished"]);
    assert_eq!(
        listed(&s, &[("filter", "today,starting")]).unwrap(),
        ["all hands", "late"]
    );
}

#[test]
fn a_typed_value_that_contradicts_the_saved_one_is_the_two_at_once_error() {
    let s = a_late_task_a_finished_one_and_all_hands();
    let Err(UseCaseError::Select(SelectError::Value(e))) =
        listed(&s, &[("filter", "open"), ("done", "true")])
    else {
        panic!("a contradiction was let through");
    };
    assert_eq!(e.field, "done");
    assert!(matches!(e.problem, Problem::TwoAtOnce { .. }));
}

#[test]
fn in_still_asks_what_the_filter_lists_on_its_own() {
    let s = a_late_task_a_finished_one_and_all_hands();
    for (byte, parent) in [(4, 1), (5, 2)] {
        let mut child = task_in(byte, &format!("under {parent}"), "home");
        child.base.parent = Some(oid(parent));
        s.working(Object::Task(child));
    }
    assert_eq!(listed(&s, &[("parent", "in:today")]).unwrap(), ["under 1"]);
}

#[test]
fn done_and_agenda_with_a_filter_work_out_their_own_defaults_once() {
    let s = a_late_task_a_finished_one_and_all_hands();
    let picked = Selector::Filter("today".into(), Path::root(), vec![]);
    assert_eq!(s.dam().select(&picked).unwrap(), [oid(1)]);
    let window = Window {
        from: "2026-09-18T00:00:00Z".parse().unwrap(),
        to: "2026-09-19T00:00:00Z".parse().unwrap(),
    };
    let zone = jiff::tz::TimeZone::UTC;
    let met = s
        .dam()
        .agenda(
            &filter(&[("filter", "meeting")]),
            &Path::root(),
            &[],
            window,
            &zone,
        )
        .unwrap();
    assert_eq!(met.len(), 1);
}
