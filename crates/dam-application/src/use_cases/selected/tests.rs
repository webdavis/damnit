use dam_domain::{
    Categories, Category, Completion, DefaultsError, Filter, FilterTable, Object, Path, Problem,
    TableValue, Task, When,
};
use jiff::civil::date;

use super::SelectError;
use crate::config::{Config, FilterConfig, FilterRule};
use crate::errors::{Refusal, UseCaseError};
use crate::testing::{Scenario, event, oid, task_in};

pub(super) fn filter(fields: &[(&str, &str)]) -> Filter {
    let fields = fields
        .iter()
        .map(|(k, v)| (k.to_string(), TableValue::One(v.to_string())))
        .collect();
    Filter::read(&FilterTable {
        fields,
        any: Vec::new(),
    })
    .unwrap()
}

pub(super) fn subjects(objects: &[Object]) -> Vec<&str> {
    objects.iter().map(|o| o.base().subject.as_str()).collect()
}

fn effort_and_a_today_filter() -> Config {
    Config {
        owner: Some("ana".into()),
        categories: Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["deep".into()],
            exclusive: true,
        }])
        .unwrap(),
        filters: vec![FilterConfig {
            name: "today".into(),
            rule: FilterRule::Query("due:today".into()),
        }],
        ..Config::default()
    }
}

fn open_done_due_and_an_event() -> Scenario {
    let s = Scenario::default().configured(effort_and_a_today_filter());
    let mut due = task_in(1, "due", "work");
    due.due = Some(When::Day(date(2026, 9, 18)));
    due.base.labels.insert("deep".into());
    let mut done = task_in(2, "done", "work");
    done.completion = Completion::Done { at: None };
    s.working(Object::Task(due));
    s.working(Object::Task(done));
    s.working(Object::Task(task_in(3, "later", "home")));
    s.working(event(4));
    s
}

fn listed(s: &Scenario, fields: &[(&str, &str)]) -> Result<Vec<String>, UseCaseError> {
    let out = s.dam().list_selected(&filter(fields), &Path::root(), &[])?;
    Ok(subjects(&out).into_iter().map(String::from).collect())
}

#[test]
fn no_condition_lists_the_tasks_that_are_not_done() {
    let s = open_done_due_and_an_event();
    assert_eq!(listed(&s, &[]).unwrap(), ["later", "due"]);
}

#[test]
fn each_condition_narrows_and_any_lifts_a_default() {
    let s = open_done_due_and_an_event();
    assert_eq!(listed(&s, &[("labels", "deep")]).unwrap(), ["due"]);
    assert_eq!(listed(&s, &[("done", "true")]).unwrap(), ["done"]);
    let all = listed(&s, &[("done", "any"), ("kind", "any")]).unwrap();
    assert_eq!(all.len(), 4);
}

#[test]
fn a_place_keeps_what_lies_under_it() {
    let s = open_done_due_and_an_event();
    let out = s
        .dam()
        .list_selected(&filter(&[]), &Path::parse("work").unwrap(), &[])
        .unwrap();
    assert_eq!(subjects(&out), ["due"]);
}

#[test]
fn a_saved_filter_runs_as_it_does_today_and_the_default_still_holds() {
    let s = open_done_due_and_an_event();
    assert_eq!(listed(&s, &[("filter", "today")]).unwrap(), ["due"]);
    assert_eq!(listed(&s, &[("filter", "not:today")]).unwrap(), ["later"]);
}

#[test]
fn a_saved_filter_that_does_not_exist_is_refused_by_name() {
    let s = open_done_due_and_an_event();
    assert_eq!(
        listed(&s, &[("parent", "in:nope")]),
        Err(UseCaseError::Refused(Refusal::NoSuchFilter("nope".into())))
    );
}

#[test]
fn a_category_is_checked_against_the_categories_of_the_place() {
    let s = open_done_due_and_an_event();
    assert_eq!(listed(&s, &[("category", "effort=deep")]).unwrap(), ["due"]);
    let Err(UseCaseError::Select(SelectError::Value(e))) =
        listed(&s, &[("category", "effort=home")])
    else {
        panic!("a label outside its group was let through");
    };
    assert!(matches!(e.problem, Problem::NotInGroup { .. }));
}

#[test]
fn a_kind_that_contradicts_a_field_is_refused() {
    let s = open_done_due_and_an_event();
    assert_eq!(
        listed(&s, &[("kind", "task"), ("start", "today")]),
        Err(UseCaseError::Select(SelectError::Defaults(
            DefaultsError::KindContradicts { field: "start" }
        )))
    );
}

#[test]
fn author_is_the_name_the_row_prints() {
    let s = open_done_due_and_an_event();
    assert_eq!(listed(&s, &[("author", "ana")]).unwrap(), ["later", "due"]);
    assert!(listed(&s, &[("author", "not:ana")]).unwrap().is_empty());
}

#[test]
fn a_task_waiting_on_an_open_task_is_blocked() {
    let s = open_done_due_and_an_event();
    let mut waits = Task::new(oid(5), "waits");
    waits.base.depends.push(oid(1));
    let mut free = Task::new(oid(6), "free");
    free.base.depends.push(oid(2));
    s.working(Object::Task(waits));
    s.working(Object::Task(free));
    assert_eq!(listed(&s, &[("blocked", "true")]).unwrap(), ["waits"]);
}

#[test]
fn a_filter_of_the_place_is_found_first_and_reads_its_paths_from_the_place() {
    let s = open_done_due_and_an_event();
    let here = [FilterConfig {
        name: "today".into(),
        rule: FilterRule::Query("path:/".into()),
    }];
    let work = Path::parse("work").unwrap();
    let out = s
        .dam()
        .list_selected(&filter(&[("filter", "today")]), &work, &here)
        .unwrap();
    assert_eq!(subjects(&out), ["due"]);
}
