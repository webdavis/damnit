use dam_application::testing::{allowed, holder, task_in};
use dam_application::{Project, ProjectSettings};
use dam_domain::{Completion, Object, Permission};
use serde_json::json;

use crate::args::{Command, LabelArgs, LabelCommand};
use crate::commands::scoped::dispatch_in;
use crate::context::Context;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::testing::context;

fn inside_damnit() -> Standing {
    Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit".into(),
            ..Default::default()
        },
    }))
}

fn labelled(byte: u8, path: &str, labels: &[&str]) -> Object {
    let mut task = task_in(byte, "a task", path);
    task.base.labels = labels.iter().map(|l| l.to_string()).collect();
    Object::Task(task)
}

fn holding_labels() -> Context {
    let ctx = context();
    let mut done = labelled(3, "damnit/cli", &["urgent"]);
    if let Object::Task(task) = &mut done {
        task.completion = Completion::Done { at: None };
    }
    for object in [
        labelled(1, "damnit", &["urgent", "bug"]),
        labelled(2, "home", &["urgent", "garden"]),
        done,
    ] {
        ctx.store.put(&object).unwrap();
    }
    ctx
}

fn label_list(ctx: &Context, standing: Standing, global: bool) -> Report {
    let mut list = LabelCommand::List(Default::default());
    let LabelCommand::List(args) = &mut list;
    args.scope.global = global;
    let command = Command::Label(LabelArgs { command: list });
    dispatch_in(ctx, command, standing).unwrap()
}

#[test]
fn every_label_in_use_with_its_count_sorted_by_name_done_tasks_included() {
    let listed = label_list(&holding_labels(), Standing::Outside, false);
    assert_eq!(
        listed.data,
        json!({"labels": [
            {"name": "bug", "count": 1},
            {"name": "garden", "count": 1},
            {"name": "urgent", "count": 3},
        ]})
    );
    assert_eq!(listed.human, "bug     1\ngarden  1\nurgent  3");
}

#[test]
fn inside_a_project_only_its_objects_count_and_global_counts_all() {
    let ctx = holding_labels();
    let inside = label_list(&ctx, inside_damnit(), false);
    assert_eq!(
        inside.data,
        json!({"labels": [{"name": "bug", "count": 1}, {"name": "urgent", "count": 2}]})
    );
    let global = label_list(&ctx, inside_damnit(), true);
    assert_eq!(global.data["labels"].as_array().unwrap().len(), 3);
}

#[test]
fn a_token_counts_only_the_objects_it_may_read() {
    let mut ctx = holding_labels();
    ctx.access = Box::new(holder(vec![allowed(
        &[Permission::TaskRead],
        Some(&["home"]),
    )]));
    let listed = label_list(&ctx, Standing::Outside, false);
    assert_eq!(
        listed.data,
        json!({"labels": [{"name": "garden", "count": 1}, {"name": "urgent", "count": 1}]})
    );
}

#[test]
fn no_label_in_use_says_so() {
    assert_eq!(
        label_list(&context(), Standing::Outside, false).human,
        "no labels in use"
    );
}
