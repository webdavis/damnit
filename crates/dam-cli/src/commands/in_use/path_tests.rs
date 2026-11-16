use dam_application::testing::{allowed, holder, task_in};
use dam_application::{Project, ProjectSettings};
use dam_domain::{Completion, Object, Permission};
use serde_json::json;

use crate::args::{Command, PathArgs, PathCommand};
use crate::commands::scoped::dispatch_in;
use crate::context::Context;
use crate::output::Report;
use crate::project_scope::Standing;
use crate::testing::context;

fn inside_work() -> Standing {
    Standing::Inside(Box::new(Project {
        folder: "/code/work".into(),
        settings: ProjectSettings {
            path: "work".into(),
            ..Default::default()
        },
    }))
}

fn holding_paths() -> Context {
    let ctx = context();
    let mut done = task_in(3, "shipped", "work/cli");
    done.completion = Completion::Done { at: None };
    for task in [
        task_in(1, "parse flags", "work/cli"),
        task_in(2, "readme", "work"),
        done,
        task_in(4, "weed", "home/garden"),
        task_in(5, "inbox", ""),
    ] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

fn path_list(ctx: &Context, standing: Standing, global: bool) -> Report {
    let mut list = PathCommand::List(Default::default());
    let PathCommand::List(args) = &mut list;
    args.scope.global = global;
    dispatch_in(ctx, Command::Path(PathArgs { command: list }), standing).unwrap()
}

#[test]
fn every_path_in_use_and_its_parents_with_what_sits_at_or_under_it() {
    let listed = path_list(&holding_paths(), Standing::Outside, false);
    assert_eq!(
        listed.data,
        json!({"paths": [
            {"path": "home/", "count": 1},
            {"path": "home/garden/", "count": 1},
            {"path": "work/", "count": 3},
            {"path": "work/cli/", "count": 2},
        ]})
    );
    assert_eq!(
        listed.human,
        "home/         1\nhome/garden/  1\nwork/         3\nwork/cli/     2"
    );
}

#[test]
fn inside_a_project_only_its_paths_are_listed_and_global_lists_all() {
    let ctx = holding_paths();
    let inside = path_list(&ctx, inside_work(), false);
    assert_eq!(
        inside.data,
        json!({"paths": [{"path": "work/", "count": 3}, {"path": "work/cli/", "count": 2}]})
    );
    let global = path_list(&ctx, inside_work(), true);
    assert_eq!(global.data["paths"].as_array().unwrap().len(), 4);
}

#[test]
fn a_token_counts_only_the_objects_it_may_read() {
    let mut ctx = holding_paths();
    ctx.access = Box::new(holder(vec![allowed(
        &[Permission::TaskRead],
        Some(&["work/cli"]),
    )]));
    let listed = path_list(&ctx, Standing::Outside, false);
    assert_eq!(
        listed.data,
        json!({"paths": [{"path": "work/", "count": 2}, {"path": "work/cli/", "count": 2}]})
    );
}

#[test]
fn no_path_in_use_says_so() {
    assert_eq!(
        path_list(&context(), Standing::Outside, false).human,
        "no paths in use"
    );
}
