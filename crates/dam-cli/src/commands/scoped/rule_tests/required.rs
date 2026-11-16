use dam_application::testing::{allowed, holder};
use dam_application::{ProjectSettings, Refusal, RequiredField, TaskTemplate, UseCaseError};
use dam_domain::{Kind, Permission};

use super::super::dispatch_in;
use super::super::tests::new_args;
use super::{global, inside, task};
use crate::args::{Command, NewArgs};
use crate::error::CliError;
use crate::project_scope::Standing;
use crate::testing::context;

fn requiring(required: Vec<RequiredField>) -> Standing {
    inside(ProjectSettings {
        required,
        ..ProjectSettings::default()
    })
}

fn refusal(required: RequiredField) -> String {
    let made = Command::New(new_args("crash", ""));
    match dispatch_in(&context(), made, requiring(vec![required])) {
        Err(CliError::Usage(text)) => text,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_task_without_a_required_field_is_refused_naming_it_and_its_flag() {
    assert_eq!(
        refusal(RequiredField::Due),
        "a task in this project needs a due date; give it with --due"
    );
    assert_eq!(
        refusal(RequiredField::Labels),
        "a task in this project needs a label; give it with --labels"
    );
    assert_eq!(
        refusal(RequiredField::Priority),
        "a task in this project needs a priority; give it with --priority"
    );
    assert_eq!(
        refusal(RequiredField::Body),
        "a task in this project needs a body; give it with --body"
    );
    assert_eq!(
        refusal(RequiredField::Deadline),
        "a task in this project needs a deadline; give it with --deadline"
    );
}

#[test]
fn a_refused_task_is_never_stored() {
    let ctx = context();
    let made = Command::New(new_args("crash", ""));
    assert!(dispatch_in(&ctx, made, requiring(vec![RequiredField::Due])).is_err());
    assert!(ctx.store.all().unwrap().is_empty());
}

#[test]
fn a_flag_or_the_projects_default_meets_the_rule() {
    let ctx = context();
    let args = NewArgs {
        due: Some("2026-09-19".into()),
        ..new_args("crash", "")
    };
    let standing = inside(ProjectSettings {
        required: vec![RequiredField::Due, RequiredField::Labels],
        new_task: TaskTemplate {
            labels: Some(vec!["app".into()]),
            ..TaskTemplate::default()
        },
        ..ProjectSettings::default()
    });
    dispatch_in(&ctx, Command::New(args), standing).unwrap();
    assert_eq!(task(&ctx).base.labels.iter().collect::<Vec<_>>(), ["app"]);
}

#[test]
fn nothing_is_required_outside_the_project_with_global_or_of_an_event() {
    let event = NewArgs {
        kind: Kind::Event,
        start: Some("2026-09-19T10:00".into()),
        end: Some("2026-09-19T11:00".into()),
        ..new_args("standup", "")
    };
    for (made, standing) in [
        (Command::New(new_args("crash", "")), Standing::Outside),
        (
            global(new_args("crash", "")),
            requiring(vec![RequiredField::Due]),
        ),
        (Command::New(event), requiring(vec![RequiredField::Due])),
    ] {
        dispatch_in(&context(), made, standing).unwrap();
    }
}

fn token(permissions: &[Permission], paths: Option<&str>) -> crate::context::Context {
    let paths = paths.map(|path| [path]);
    crate::context::Context {
        access: Box::new(holder(vec![allowed(
            permissions,
            paths.as_ref().map(|p| &p[..]),
        )])),
        ..context()
    }
}

#[test]
fn a_token_is_refused_for_its_permission_first_then_told_only_the_field() {
    let required = || requiring(vec![RequiredField::Due]);
    let made = || Command::New(new_args("crash", ""));
    let sees_nothing = dispatch_in(&token(&[Permission::TaskRead], None), made(), required());
    assert!(matches!(
        sees_nothing,
        Err(CliError::UseCase(UseCaseError::Refused(
            Refusal::NotPermitted(_)
        )))
    ));
    for ctx in [
        token(&[Permission::TaskCreate], Some("app")),
        token(&[Permission::TaskCreate, Permission::TaskRead], None),
    ] {
        let Err(CliError::Usage(text)) = dispatch_in(&ctx, made(), required()) else {
            panic!("not refused")
        };
        assert_eq!(
            text,
            "a task in this project needs a due date; give it with --due"
        );
    }
}
