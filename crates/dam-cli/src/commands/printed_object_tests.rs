use clap::Parser;
use dam_domain::{Categories, Category};
use serde_json::{Value, json};

use crate::args::{Cli, Command, ShowArgs};
use crate::context::Context;
use crate::testing::context;

fn with_effort() -> Context {
    let mut ctx = context();
    ctx.config.categories = Categories::new(vec![Category {
        name: "effort".into(),
        values: vec!["deep".into(), "light".into()],
        exclusive: true,
    }])
    .unwrap();
    ctx
}

fn command(words: &[&str]) -> Command {
    Cli::try_parse_from(["dam"].iter().chain(words))
        .unwrap()
        .command
}

fn printed(ctx: &Context, words: &[&str]) -> Value {
    let report = match command(words) {
        Command::New(args) => super::new::run(ctx, args),
        Command::Edit(args) => super::edit::run(ctx, args),
        other => panic!("{other:?}"),
    };
    report.unwrap().data
}

fn assert_worked_out(data: &Value, category: Value, overdue: bool) {
    assert_eq!(data["category"], category, "{data}");
    assert_eq!(data["overdue"], overdue, "{data}");
    assert!(
        data["days_until_due"].as_i64().is_some_and(|days| days < 0),
        "{data}"
    );
    assert!(data.get("blocked").is_none(), "{data}");
    assert!(data.get("children_open").is_none(), "{data}");
}

#[test]
fn new_edit_and_show_print_the_category_and_overdue_they_know_without_reading_the_store() {
    let ctx = with_effort();
    let made = printed(
        &ctx,
        &["new", "late", "--due", "2000-01-01", "--labels", "deep"],
    );
    assert_worked_out(&made, json!({"effort": "deep"}), true);
    let oid = made["oid"].as_str().unwrap().to_string();
    let edited = printed(&ctx, &["edit", &oid, "--unlabels", "deep"]);
    assert_worked_out(&edited, json!({}), true);
    let shown = super::show::run_show(&ctx, ShowArgs { id: oid })
        .unwrap()
        .data;
    assert_worked_out(&shown, json!({}), true);
}
