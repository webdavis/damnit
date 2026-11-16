use dam_application::testing::task_in;
use dam_application::{Project, ProjectSettings};
use dam_domain::Object;

use super::dispatch_in;
use crate::args::Command;
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::{context, list_args};

fn inside() -> Standing {
    Standing::Inside(Box::new(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings {
            path: "damnit".into(),
            ..Default::default()
        },
    }))
}

fn list(words: &[&str], global: bool) -> Command {
    let mut args = list_args(words);
    args.scope.global = global;
    Command::List(args)
}

fn holding_a_project_and_more() -> Context {
    let ctx = context();
    for task in [
        task_in(1, "damnit", ""),
        task_in(2, "parse flags", "damnit/cli"),
        task_in(3, "readme", "damnit"),
        task_in(4, "other cli", "cli"),
    ] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

#[test]
fn list_inside_a_project_equals_a_path_search_outside() {
    let ctx = holding_a_project_and_more();
    let inside_the_project = dispatch_in(&ctx, list(&[], false), inside()).unwrap();
    let outside =
        dispatch_in(&ctx, list(&["--path", "damnit/"], false), Standing::Outside).unwrap();
    assert_eq!(inside_the_project.data, outside.data);
    assert_eq!(
        inside_the_project.data["objects"].as_array().unwrap().len(),
        2
    );
}

#[test]
fn a_typed_path_starts_from_the_project_and_listings_print_full_paths() {
    let ctx = holding_a_project_and_more();
    let listed = dispatch_in(&ctx, list(&["--path", "cli/"], false), inside()).unwrap();
    let objects = listed.data["objects"].as_array().unwrap();
    assert_eq!(objects.len(), 1);
    assert_eq!(objects[0]["path"], "damnit/cli/");
}

#[test]
fn global_lists_everything_and_reads_a_path_from_the_root() {
    let ctx = holding_a_project_and_more();
    let everything = dispatch_in(&ctx, list(&[], true), inside()).unwrap();
    assert_eq!(everything.data["objects"].as_array().unwrap().len(), 4);
    let cli = dispatch_in(&ctx, list(&["--path", "cli/"], true), inside()).unwrap();
    assert_eq!(cli.data["objects"][0]["subject"], "other cli");
}
