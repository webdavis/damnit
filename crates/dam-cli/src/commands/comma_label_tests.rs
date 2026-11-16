use clap::Parser;
use dam_domain::{Categories, Category, Object, Oid, Task};

use crate::args::{Cli, Command};
use crate::context::Context;
use crate::request::hook_edit_args;
use crate::testing::context;

fn command(argv: &[&str]) -> Command {
    Cli::try_parse_from(argv).unwrap().command
}

fn typed(ctx: &Context, argv: &[&str]) -> Result<String, String> {
    let report = match command(argv) {
        Command::New(a) => super::new::run(ctx, a),
        Command::Edit(a) => super::edit::run(ctx, a),
        Command::Ls(a) => super::ls::run_ls(ctx, a),
        _ => panic!("not a command these tests type"),
    };
    report.map(|r| r.human).map_err(|e| e.to_string())
}

fn oid() -> Oid {
    Oid::generate(&mut |x: &mut [u8]| x.fill(7))
}

fn labels_of(ctx: &Context) -> Vec<String> {
    ctx.store.all().unwrap()[0]
        .base()
        .labels
        .iter()
        .cloned()
        .collect()
}

fn holding(label: &str) -> Context {
    let ctx = context();
    let mut task = Task::new(oid(), "milk");
    task.base.labels.insert(label.into());
    ctx.store.put(&Object::Task(task)).unwrap();
    ctx
}

#[test]
fn new_with_a_comma_list_saves_two_labels() {
    let ctx = context();
    typed(&ctx, &["dam", "new", "x", "--labels", "small,big"]).unwrap();
    assert_eq!(labels_of(&ctx), ["big", "small"]);
}

#[test]
fn an_exclusive_category_refuses_two_of_its_labels_typed_as_one_list() {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![Category {
        name: "size".into(),
        values: vec!["small".into(), "big".into()],
        exclusive: true,
    }])
    .unwrap();
    let refused = typed(&ctx, &["dam", "new", "x", "--labels", "small,big"]).unwrap_err();
    assert!(refused.contains("size"), "{refused}");
    assert!(ctx.store.all().unwrap().is_empty());
}

#[test]
fn edit_with_a_comma_list_adds_two_labels() {
    let ctx = holding("errand");
    typed(&ctx, &["dam", "edit", oid().as_str(), "--labels", "a, b"]).unwrap();
    assert_eq!(labels_of(&ctx), ["a", "b", "errand"]);
}

#[test]
fn a_stored_label_with_a_comma_survives_an_edit_of_another_field_and_shows_in_ls() {
    let ctx = holding("small,big");
    typed(
        &ctx,
        &["dam", "edit", oid().as_str(), "--subject", "oat milk"],
    )
    .unwrap();
    assert_eq!(labels_of(&ctx), ["small,big"]);
    let listed = typed(&ctx, &["dam", "ls"]).unwrap();
    assert!(listed.contains("small,big"), "{listed}");
}

#[test]
fn a_request_document_removes_a_stored_label_with_a_comma() {
    let ctx = holding("small,big");
    let args = hook_edit_args(&oid(), r#"{"unlabels": ["small,big"]}"#).unwrap();
    super::edit::run(&ctx, args).unwrap();
    assert!(labels_of(&ctx).is_empty());
}
