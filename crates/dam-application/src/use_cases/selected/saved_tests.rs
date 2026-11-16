use dam_domain::{Event, Object, Order, Path, When};
use jiff::civil::date;

use super::tests::{effort_and_a_today_filter, filter, open_done_due_and_an_event, subjects};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{Scenario, oid, saved, task_in};
use crate::use_cases::each::Selector;
use crate::use_cases::selected::BeyondPulled;

fn configured(filters: Vec<crate::config::FilterConfig>) -> Scenario {
    let s = open_done_due_and_an_event();
    let mut config = effort_and_a_today_filter();
    config.filters.extend(filters);
    s.configured(config)
}

fn listed(dam: &Dam, fields: &[(&str, &str)]) -> Result<Vec<String>, UseCaseError> {
    let out = dam.list_selected(&filter(fields), &Order::default(), &Path::root(), &[])?;
    Ok(subjects(&out).into_iter().map(String::from).collect())
}

#[test]
fn a_filter_that_reaches_itself_is_refused_naming_the_chain() {
    let s = configured(vec![
        saved("mine", &[("filter", "theirs")]),
        saved("theirs", &[("parent", "in:mine")]),
    ]);
    assert_eq!(
        listed(&s.dam(), &[("filter", "mine")]),
        Err(UseCaseError::Refused(Refusal::FilterReachesItself(vec![
            "mine".into(),
            "theirs".into(),
            "mine".into()
        ])))
    );
}

#[test]
fn a_filter_may_name_another_filter() {
    let s = configured(vec![saved(
        "soon",
        &[("filter", "today"), ("path", "work/")],
    )]);
    assert_eq!(listed(&s.dam(), &[("filter", "soon")]).unwrap(), ["due"]);
}

#[test]
fn in_lists_what_points_at_an_object_the_filter_lists() {
    let s = configured(vec![]);
    let mut child = task_in(5, "child", "home");
    child.base.parent = Some(oid(1));
    s.working(Object::Task(child));
    assert_eq!(
        listed(&s.dam(), &[("parent", "in:today")]).unwrap(),
        ["child"]
    );
}

#[test]
fn on_lists_what_falls_on_a_day_an_event_of_the_filter_takes_place() {
    let s = configured(vec![saved("standup", &[("kind", "event")])]);
    let standup = Event::new(
        oid(9),
        "standup",
        When::Day(date(2026, 9, 18)),
        When::Day(date(2026, 9, 19)),
    );
    s.working(Object::Event(standup));
    assert_eq!(listed(&s.dam(), &[("due", "on:standup")]).unwrap(), ["due"]);
}

#[test]
fn a_personal_filter_run_inside_a_project_reads_its_paths_from_the_root() {
    let s = configured(vec![saved("work", &[("path", "work/")])]);
    let selector = Selector::Filter("work".into(), Path::parse("work").unwrap(), vec![]);
    assert_eq!(s.dam().select(&selector).unwrap(), [oid(1)]);
    let elsewhere = Selector::Filter("work".into(), Path::parse("home").unwrap(), vec![]);
    assert!(s.dam().select(&elsewhere).unwrap().is_empty());
}

#[test]
fn a_projects_filter_is_found_first_and_reads_its_paths_from_the_project() {
    let s = configured(vec![]);
    let here = vec![saved("today", &[("path", "/")])];
    let selector = Selector::Filter("today".into(), Path::parse("work").unwrap(), here);
    assert_eq!(s.dam().select(&selector).unwrap(), [oid(1)]);
}

#[test]
fn a_function_narrows_what_the_fields_list() {
    let mut s = configured(vec![crate::testing::saved_function(
        "mine",
        &[("path", "work/")],
    )]);
    s.fixture.filter_functions = crate::testing::FakeFilterFunctions::keeping(&["due", "later"]);
    assert_eq!(listed(&s.dam(), &[("filter", "mine")]).unwrap(), ["due"]);
}

fn noted(dam: &Dam) -> Option<BeyondPulled> {
    let on = filter(&[("due", "on:standup")]);
    dam.list_selected_noting(&on, &Order::default(), &Path::root(), &[])
        .unwrap()
        .1
}

fn due_late(s: &Scenario, byte: u8, path: &str) {
    let mut late = task_in(byte, "late", path);
    late.due = Some(When::Day(date(2026, 10, 1)));
    s.working(Object::Task(late));
}

#[test]
fn on_counts_what_it_could_not_place_past_the_events_dam_holds() {
    let s = configured(vec![saved("standup", &[("kind", "event")])]);
    let standup = Event::new(
        oid(9),
        "standup",
        When::Day(date(2026, 9, 18)),
        When::Day(date(2026, 9, 19)),
    );
    s.working(Object::Event(standup));
    assert_eq!(noted(&s.dam()), None);
    due_late(&s, 7, "work");
    due_late(&s, 8, "home");
    let beyond = BeyondPulled {
        filters: vec!["standup".into()],
        fields: vec!["due"],
        held: Some((date(2026, 1, 1), date(2026, 9, 18))),
        tasks: 2,
        events: 0,
    };
    assert_eq!(noted(&s.dam()), Some(beyond.clone()));
    let token = crate::testing::holder(vec![crate::testing::allowed(
        &[
            dam_domain::Permission::TaskRead,
            dam_domain::Permission::EventRead,
        ],
        Some(&["home"]),
    )]);
    let dam = Dam {
        access: &token,
        ..s.dam()
    };
    assert_eq!(noted(&dam).map(|b| b.tasks), Some(1));
}
