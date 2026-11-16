use crate::testing::{context, list_args};
use dam_application::testing::saved;
use dam_domain::{Categories, Category, Object, Oid, Task, When};
use jiff::civil::date;

use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;

fn run(ctx: &Context, words: &[&str]) -> Result<Report, CliError> {
    super::run_list(ctx, list_args(words))
}

fn count(ctx: &Context, words: &[&str]) -> usize {
    run(ctx, words).unwrap().data["objects"]
        .as_array()
        .unwrap()
        .len()
}

fn today_and_later() -> Context {
    let mut ctx = context();
    let mut due = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(1)), "today");
    due.due = Some(When::Day(date(2026, 9, 18)));
    ctx.store.put(&Object::Task(due)).unwrap();
    let later = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(2)), "later");
    ctx.store.put(&Object::Task(later)).unwrap();
    ctx.config.filters.push(saved("now", &[("due", "today")]));
    ctx
}

#[test]
fn list_narrows_by_flags_and_by_saved_filter() {
    let ctx = today_and_later();
    assert_eq!(count(&ctx, &[]), 2);
    assert_eq!(count(&ctx, &["--due", "today"]), 1);
    let saved = run(&ctx, &["--filter", "now"]).unwrap();
    assert_eq!(saved.human.lines().count(), 1);
    assert!(saved.human.contains("today"));
}

#[test]
fn author_lists_by_the_name_each_row_prints() {
    let ctx = today_and_later();
    assert_eq!(count(&ctx, &["--author", "owner"]), 2);
    assert_eq!(count(&ctx, &["--author", "not:owner"]), 0);
}

#[test]
fn a_label_outside_its_group_exits_2_naming_the_groups_labels_and_a_command() {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["deep".into()],
        exclusive: true,
    }])
    .unwrap();
    let err = run(&ctx, &["--category", "effort=home"]).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert_eq!(
        err.to_string(),
        "--category effort=home: the group effort holds only deep; run dam list --category effort=deep"
    );
}

#[test]
fn an_undeclared_group_exits_2_and_names_it() {
    let err = run(&context(), &["--category", "mood=happy"]).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(err.to_string().contains("mood"), "{err}");
}

#[test]
fn a_bad_value_is_refused_before_the_store_is_read() {
    let err = run(&context(), &["--priority", "9"]).unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert!(
        err.to_string().ends_with("run dam list --priority 1"),
        "{err}"
    );
}
