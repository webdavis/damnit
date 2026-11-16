use clap::CommandFactory;
use dam_domain::{Object, Oid, Priority, Task};

use crate::args::Cli;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::testing::{context, list_args};

fn run(ctx: &Context, words: &[&str]) -> Result<Report, CliError> {
    super::run_list(ctx, list_args(words))
}

fn subjects(ctx: &Context, words: &[&str]) -> Vec<String> {
    let rows = run(ctx, words).unwrap().data["objects"].clone();
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row["subject"].as_str().unwrap().to_string())
        .collect()
}

fn four_priorities() -> Context {
    let ctx = context();
    for (byte, priority) in [(1, 2), (2, 4), (3, 1), (4, 3)] {
        let mut task = Task::new(Oid::generate(&mut |x: &mut [u8]| x.fill(byte)), "");
        task.base.subject = format!("p{priority}");
        task.priority = Priority::new(priority).unwrap();
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

#[test]
fn sort_limit_and_offset_shape_the_listed_rows() {
    let ctx = four_priorities();
    assert_eq!(
        subjects(&ctx, &["--sort", "priority"]),
        ["p1", "p2", "p3", "p4"]
    );
    assert_eq!(
        subjects(&ctx, &["--sort", "priority:desc", "--limit", "2"]),
        ["p4", "p3"]
    );
    assert_eq!(
        subjects(&ctx, &["--sort", "priority", "--offset", "3"]),
        ["p4"]
    );
}

#[test]
fn a_mistyped_sort_field_names_the_closest_and_the_command_to_run() {
    let err = run(
        &context(),
        &["--labels", "bug", "--sort", "priority,duee:desc"],
    )
    .unwrap_err();
    assert_eq!(err.exit_code(), 2);
    assert_eq!(
        err.to_string(),
        "--sort duee:desc names no field; the closest is due; run dam list --labels bug --sort priority,due:desc"
    );
    let doc = err.document();
    assert_eq!(doc["error"]["usage"]["word"], "--sort");
    assert_eq!(doc["error"]["selection"]["field"], "sort");
    assert_eq!(doc["error"]["selection"]["value"], "duee:desc");
    assert_eq!(doc["error"]["selection"]["shapes"][1], "FIELD:desc");
}

#[test]
fn a_sort_direction_other_than_desc_names_the_shapes() {
    let err = run(&context(), &["--sort", "due:down"]).unwrap_err();
    assert_eq!(
        err.to_string(),
        "--sort cannot be \"due:down\"; it takes FIELD, FIELD:desc; run dam list --sort due"
    );
}

#[test]
fn sort_limit_and_offset_sit_under_a_heading_of_their_own() {
    let mut root = Cli::command();
    root.build();
    let list = root.find_subcommand("list").unwrap();
    for flag in ["sort", "limit", "offset"] {
        let arg = list
            .get_arguments()
            .find(|a| a.get_long() == Some(flag))
            .unwrap_or_else(|| panic!("--{flag} is missing"));
        assert_eq!(arg.get_help_heading(), Some("Order and size"), "--{flag}");
    }
}
