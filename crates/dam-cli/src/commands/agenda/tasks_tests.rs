use dam_domain::{Object, Oid, Task, When};
use jiff::civil::date;

use super::run_agenda;
use super::tests::{standup, timed};
use crate::context::Context;
use crate::output::Report;
use crate::testing::{agenda_args, context};

fn due(fill: u8, subject: &str, day: i8) -> Object {
    let mut t = Task::new(Oid::generate(&mut |b: &mut [u8]| b.fill(fill)), subject);
    t.due = Some(When::Day(date(2026, 9, day)));
    t.base.labels.insert("home".into());
    Object::Task(t)
}

fn two_days_of_events_and_tasks() -> Context {
    let ctx = context();
    ctx.store.put(&Object::Event(standup())).unwrap();
    let next = timed(
        3,
        "next",
        "2026-09-19T09:00:00+00:00[UTC]",
        "2026-09-19T09:30:00+00:00[UTC]",
    );
    for object in [
        next,
        due(4, "groceries", 19),
        due(5, "rent", 18),
        due(6, "taxes", 17),
    ] {
        ctx.store.put(&object).unwrap();
    }
    ctx
}

const NAMES: [&str; 4] = ["standup", "rent", "next", "groceries"];

fn agenda(ctx: &Context, words: &[&str]) -> Report {
    run_agenda(ctx, agenda_args(words)).unwrap()
}

fn subjects(report: &Report, key: &str) -> Vec<String> {
    let rows = report.data[key].as_array().unwrap();
    rows.iter()
        .map(|r| r["subject"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn kind_any_lists_the_tasks_due_on_each_day_of_the_window_beside_the_events() {
    let ctx = two_days_of_events_and_tasks();
    let both = agenda(
        &ctx,
        &["--kind", "any", "--from", "today", "--to", "today+1"],
    );
    assert_eq!(subjects(&both, "events"), ["standup", "next"]);
    assert_eq!(subjects(&both, "tasks"), ["rent", "groceries"]);
    assert_eq!(both.data["tasks"][0]["kind"], "task");
    assert_eq!(both.data["tasks"][0]["overdue"], false);
}

#[test]
fn the_text_lists_each_day_with_its_events_first_and_then_its_tasks_as_list_rows() {
    let ctx = two_days_of_events_and_tasks();
    let both = agenda(
        &ctx,
        &["--kind", "any", "--from", "today", "--to", "today+1"],
    );
    let subjects: Vec<&str> = both
        .human
        .lines()
        .filter_map(|l| NAMES.into_iter().find(|n| l.contains(n)))
        .collect();
    assert_eq!(subjects, ["standup", "rent", "next", "groceries"]);
    let rent = both.human.lines().nth(1).unwrap();
    assert!(rent.contains("p4") && rent.contains("2026-09-18"), "{rent}");
    assert!(rent.ends_with("rent  [home]"), "{rent}");
}

#[test]
fn kind_task_lists_only_the_tasks_and_no_flag_only_the_events() {
    let ctx = two_days_of_events_and_tasks();
    let tasks = agenda(
        &ctx,
        &["--kind", "task", "--from", "today", "--to", "today+1"],
    );
    assert_eq!(subjects(&tasks, "events"), Vec::<String>::new());
    assert_eq!(subjects(&tasks, "tasks"), ["rent", "groceries"]);
    let events = agenda(&ctx, &["--from", "today", "--to", "today+1"]);
    assert_eq!(events.data.get("tasks"), None);
    assert_eq!(subjects(&events, "events"), ["standup", "next"]);
}

#[test]
fn a_task_field_lists_the_tasks_it_selects_in_the_window() {
    let ctx = two_days_of_events_and_tasks();
    let found = agenda(
        &ctx,
        &["--labels", "home", "--due", "today", "--to", "week"],
    );
    assert_eq!(subjects(&found, "tasks"), ["rent"]);
}

#[test]
fn an_empty_window_says_which_kinds_it_looked_for() {
    let ctx = context();
    let text = |words: &[&str]| agenda(&ctx, words).human;
    assert_eq!(text(&[]), "no events in the window");
    assert_eq!(text(&["--kind", "task"]), "no tasks due in the window");
    assert_eq!(
        text(&["--kind", "any"]),
        "no events or tasks due in the window"
    );
}
