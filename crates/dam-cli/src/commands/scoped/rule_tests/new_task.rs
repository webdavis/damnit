use dam_application::{ProjectSettings, TaskTemplate};
use dam_domain::{Path, Priority};

use super::super::dispatch_in;
use super::super::tests::new_args;
use super::{global, inside, task};
use crate::args::{Command, NewArgs};
use crate::project_scope::Standing;
use crate::testing::{FakeTemplates, context};

fn defaults() -> ProjectSettings {
    ProjectSettings {
        new_task: TaskTemplate {
            labels: Some(vec!["app".into()]),
            priority: Some(Priority::new(2).unwrap()),
            due: Some("2026-09-19".into()),
            deadline: Some("2026-09-30".into()),
            ..TaskTemplate::default()
        },
        ..ProjectSettings::default()
    }
}

#[test]
fn a_task_made_inside_the_project_takes_its_defaults() {
    let ctx = context();
    let made = Command::New(new_args("crash", ""));
    dispatch_in(&ctx, made, inside(defaults())).unwrap();
    let task = task(&ctx);
    assert_eq!(task.base.labels.iter().collect::<Vec<_>>(), ["app"]);
    assert_eq!(task.priority.get(), 2);
    assert_eq!(task.due.map(|d| d.to_text()).as_deref(), Some("2026-09-19"));
    assert_eq!(
        task.deadline.map(|d| d.to_string()).as_deref(),
        Some("2026-09-30")
    );
}

#[test]
fn a_flag_wins_over_the_projects_default() {
    let ctx = context();
    let args = NewArgs {
        labels: vec!["mine".into()],
        priority: Some(Priority::HIGHEST),
        ..new_args("crash", "")
    };
    dispatch_in(&ctx, Command::New(args), inside(defaults())).unwrap();
    let task = task(&ctx);
    assert_eq!(task.base.labels.iter().collect::<Vec<_>>(), ["mine"]);
    assert_eq!(task.priority.get(), 1);
    assert_eq!(task.due.map(|d| d.to_text()).as_deref(), Some("2026-09-19"));
}

#[test]
fn a_template_wins_over_the_projects_default() {
    let template = TaskTemplate {
        labels: Some(vec!["bug".into()]),
        path: Some(Path::parse("bugs").unwrap()),
        ..TaskTemplate::default()
    };
    let ctx = crate::context::Context {
        templates: Box::new(FakeTemplates {
            held: vec![(
                "bug".into(),
                dam_application::TemplateOwner::Project,
                template,
            )],
            looked_up: Default::default(),
        }),
        ..context()
    };
    let args = NewArgs {
        template: Some("bug".into()),
        ..new_args("crash", "")
    };
    dispatch_in(&ctx, Command::New(args), inside(defaults())).unwrap();
    let task = task(&ctx);
    assert_eq!(task.base.labels.iter().collect::<Vec<_>>(), ["bug"]);
    assert_eq!(task.priority.get(), 2);
}

#[test]
fn no_default_applies_outside_the_project_or_with_global_or_project() {
    let mut elsewhere = new_args("crash", "");
    elsewhere.scope.project = Some(Path::parse("other").unwrap());
    for (made, standing) in [
        (Command::New(new_args("crash", "")), Standing::Outside),
        (global(new_args("crash", "")), inside(defaults())),
        (Command::New(elsewhere), inside(defaults())),
    ] {
        let ctx = context();
        dispatch_in(&ctx, made, standing).unwrap();
        let task = task(&ctx);
        assert!(task.base.labels.is_empty());
        assert!(task.due.is_none());
    }
}
