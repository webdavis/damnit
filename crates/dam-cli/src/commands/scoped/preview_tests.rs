use dam_application::{ProjectSettings, RequiredField, TaskTemplate};
use dam_domain::Path;

use super::dispatch_in;
use super::rule_tests::inside;
use super::tests::new_args;
use crate::args::{Command, NewArgs};
use crate::error::CliError;
use crate::project_scope::Standing;
use crate::testing::context;

fn previewed(args: NewArgs) -> Command {
    Command::New(NewArgs {
        preview: true,
        ..args
    })
}

#[test]
fn new_preview_answers_the_one_create_and_writes_nothing() {
    let ctx = context();
    let made = dispatch_in(&ctx, previewed(new_args("milk", "home")), Standing::Outside).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
    assert_eq!(made.data["preview"], true);
    assert_eq!(made.data["changes"][0]["op"], "create");
    assert_eq!(made.data["changes"][0]["subject"], "milk");
    assert!(
        made.human
            .trim_end()
            .ends_with("preview: 1 of 1 would change, nothing written")
    );
}

#[test]
fn new_preview_with_project_says_it_would_create_the_project_task_and_creates_none() {
    let ctx = context();
    let mut args = new_args("x", "");
    args.scope.project = Some(Path::parse("pns/hooks/").unwrap());
    let made = dispatch_in(&ctx, previewed(args), Standing::Outside).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
    let subjects: Vec<&str> = made.data["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["subject"].as_str().unwrap())
        .collect();
    assert_eq!(subjects.len(), 2);
    assert!(subjects.contains(&"hooks") && subjects.contains(&"x"));
}

#[test]
fn new_preview_inside_a_project_applies_its_rules() {
    let ctx = context();
    let standing = inside(ProjectSettings {
        new_task: TaskTemplate {
            labels: Some(vec!["app".into()]),
            ..TaskTemplate::default()
        },
        ..ProjectSettings::default()
    });
    let made = dispatch_in(&ctx, previewed(new_args("x", "")), standing).unwrap();
    assert!(ctx.store.all().unwrap().is_empty());
    assert_eq!(made.data["changes"][0]["path"], "app/");
    assert_eq!(made.data["changes"][0]["labels"][0], "app");
}

#[test]
fn new_preview_refuses_as_the_run_would() {
    let ctx = context();
    let standing = inside(ProjectSettings {
        required: vec![RequiredField::Due],
        ..ProjectSettings::default()
    });
    assert!(matches!(
        dispatch_in(&ctx, previewed(new_args("x", "")), standing),
        Err(CliError::Usage(_))
    ));
}
