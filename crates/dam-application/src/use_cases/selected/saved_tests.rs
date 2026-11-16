use dam_domain::{Event, Object, Order, Path, When};
use jiff::civil::date;

use super::tests::{effort_and_a_today_filter, filter, open_done_due_and_an_event, subjects};
use crate::errors::{Refusal, UseCaseError};
use crate::service::Dam;
use crate::testing::{Scenario, oid, saved, task_in};
use crate::use_cases::each::Selector;

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
