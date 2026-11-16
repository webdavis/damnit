use dam_application::ProjectSettings;
use dam_application::testing::task_in;
use dam_domain::Object;

use super::super::dispatch_in;
use super::inside;
use crate::args::Command;
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::{context, list_args};

fn list(words: &[&str], global: bool) -> Command {
    let mut args = list_args(words);
    args.scope.global = global;
    Command::List(args)
}

fn listing_urgent() -> Standing {
    inside(ProjectSettings {
        ls: Some("p1".into()),
        ..ProjectSettings::default()
    })
}

fn holding() -> Context {
    let ctx = context();
    let mut urgent = task_in(2, "urgent", "app");
    urgent.priority = dam_domain::Priority::HIGHEST;
    let mut elsewhere = task_in(3, "urgent elsewhere", "home");
    elsewhere.priority = dam_domain::Priority::HIGHEST;
    for task in [task_in(1, "calm", "app"), urgent, elsewhere] {
        ctx.store.put(&Object::Task(task)).unwrap();
    }
    ctx
}

fn subjects(ctx: &Context, command: Command, standing: Standing) -> Vec<String> {
    let listed = dispatch_in(ctx, command, standing).unwrap();
    let mut subjects: Vec<String> = listed.data["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["subject"].as_str().unwrap().to_string())
        .collect();
    subjects.sort();
    subjects
}

#[test]
fn list_without_a_selection_flag_shows_the_projects_listing_narrowed_to_it() {
    let ctx = holding();
    assert_eq!(
        subjects(&ctx, list(&[], false), listing_urgent()),
        ["urgent"]
    );
}

#[test]
fn a_selection_flag_replaces_the_projects_listing() {
    let ctx = holding();
    assert_eq!(
        subjects(&ctx, list(&["--priority", "4"], false), listing_urgent()),
        ["calm"]
    );
}

#[test]
fn global_ignores_the_projects_listing() {
    let ctx = holding();
    assert_eq!(
        subjects(&ctx, list(&[], true), listing_urgent()),
        ["calm", "urgent", "urgent elsewhere"]
    );
}

fn with_filter(name: &str, query: &str) -> dam_application::FilterConfig {
    dam_application::FilterConfig {
        name: name.into(),
        rule: dam_application::FilterRule::Query(query.into()),
    }
}

#[test]
fn the_projects_filter_is_found_before_a_personal_one_of_the_same_name() {
    let mut ctx = holding();
    ctx.config.filters = vec![with_filter("hot", "p4")];
    let standing = || {
        inside(ProjectSettings {
            filters: vec![with_filter("hot", "p1")],
            ..ProjectSettings::default()
        })
    };
    assert_eq!(
        subjects(&ctx, list(&["--filter", "hot"], false), standing()),
        ["urgent"]
    );
    assert_eq!(
        subjects(&ctx, list(&["--filter", "hot"], true), standing()),
        ["calm"]
    );
}

#[test]
fn an_output_flag_keeps_the_projects_listing() {
    let ctx = holding();
    let args = crate::testing::list_args(&["-o", "json"]);
    assert_eq!(
        subjects(&ctx, Command::List(args), listing_urgent()),
        ["urgent"]
    );
}
