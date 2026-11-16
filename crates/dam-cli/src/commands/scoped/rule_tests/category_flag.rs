use clap::Parser;
use dam_application::ProjectSettings;
use dam_domain::{Categories, Category, Object, Path, Task};

use super::super::{dispatch_in, following};
use super::{inside, task};
use crate::args::{Cli, Command};
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::context;

fn with_effort() -> Standing {
    inside(ProjectSettings {
        categories: Categories::new(vec![Category {
            name: "effort".into(),
            values: vec!["light".into(), "deep".into()],
            exclusive: true,
        }])
        .unwrap(),
        ..ProjectSettings::default()
    })
}

fn command(words: &[&str]) -> Command {
    Cli::try_parse_from(["dam"].iter().chain(words))
        .unwrap()
        .command
}

fn run(ctx: Context, words: &[&str]) -> (Context, Result<(), String>) {
    let command = command(words);
    let ctx = following(ctx, &command, &with_effort());
    let ran = dispatch_in(&ctx, command, with_effort())
        .map(|_| ())
        .map_err(|e| e.to_string());
    (ctx, ran)
}

#[test]
fn new_gives_the_task_the_label_of_the_group() {
    let (ctx, ran) = run(context(), &["new", "crash", "--category", "effort=deep"]);
    ran.unwrap();
    assert!(task(&ctx).base.labels.contains("deep"));
}

#[test]
fn new_refuses_a_label_outside_the_group_naming_its_labels() {
    let (ctx, ran) = run(context(), &["new", "crash", "--category", "effort=huge"]);
    let err = ran.unwrap_err();
    assert!(err.contains("light, deep"), "{err}");
    assert!(ctx.store.all().unwrap().is_empty());
}

#[test]
fn a_second_label_of_an_exclusive_group_is_refused_as_labels_refuses_it() {
    let words = [
        "new",
        "crash",
        "--category",
        "effort=deep",
        "--labels",
        "light",
    ];
    let err = run(context(), &words).1.unwrap_err();
    assert!(err.contains("effort"), "{err}");
}

fn holding_deep() -> Context {
    let ctx = context();
    let mut held = Task::new(dam_domain::Oid::generate(&mut |b| b.fill(3)), "crash");
    held.base.path = Path::parse("app").unwrap();
    held.base.labels.insert("deep".into());
    held.base.labels.insert("bug".into());
    ctx.store.put(&Object::Task(held)).unwrap();
    ctx
}

#[test]
fn edit_none_takes_the_groups_label_off() {
    let ctx = holding_deep();
    let (ctx, ran) = run(ctx, &["edit", "0303", "--category", "effort=none"]);
    ran.unwrap();
    let labels = task(&ctx).base.labels;
    assert!(
        !labels.contains("deep") && labels.contains("bug"),
        "{labels:?}"
    );
}

#[test]
fn edit_none_and_a_label_of_the_group_swap_the_label() {
    let words = [
        "edit",
        "0303",
        "--category",
        "effort=light",
        "--category",
        "effort=none",
    ];
    let (ctx, ran) = run(holding_deep(), &words);
    ran.unwrap();
    let labels = task(&ctx).base.labels;
    assert!(
        labels.contains("light") && !labels.contains("deep"),
        "{labels:?}"
    );
}
