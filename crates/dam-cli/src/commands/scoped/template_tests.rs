use std::cell::RefCell;
use std::rc::Rc;

use dam_application::{Project, ProjectSettings, TaskTemplate, TemplateOwner};
use dam_domain::{Object, Path, Priority};

use super::dispatch_in;
use super::tests::new_args;
use crate::args::{Command, NewArgs};
use crate::context::Context;
use crate::error::CliError;
use crate::project_scope::Standing;
use crate::testing::{FakeTemplates, context};

fn bug() -> TaskTemplate {
    TaskTemplate {
        body: Some("Steps:".into()),
        labels: Some(vec!["bug".into()]),
        priority: Some(Priority::new(2).unwrap()),
        due: Some("tomorrow".into()),
        deadline: Some("2026-09-30".into()),
        path: Some(Path::parse("bugs").unwrap()),
    }
}

type LookedUp = Rc<RefCell<Vec<String>>>;

fn holding(templates: Vec<(&str, TemplateOwner, TaskTemplate)>) -> Context {
    watched(templates).0
}

fn watched(templates: Vec<(&str, TemplateOwner, TaskTemplate)>) -> (Context, LookedUp) {
    let looked_up = LookedUp::default();
    let held = templates
        .into_iter()
        .map(|(name, owner, template)| (name.to_string(), owner, template))
        .collect();
    let ctx = Context {
        templates: Box::new(FakeTemplates {
            held,
            looked_up: looked_up.clone(),
        }),
        ..context()
    };
    (ctx, looked_up)
}

fn from(template: &str, args: NewArgs) -> Command {
    Command::New(NewArgs {
        template: Some(template.into()),
        ..args
    })
}

fn task(ctx: &Context) -> dam_domain::Task {
    match ctx.store.all().unwrap().pop() {
        Some(Object::Task(task)) => task,
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_new_task_starts_from_each_field_of_its_template() {
    let ctx = holding(vec![("bug", TemplateOwner::Person, bug())]);
    dispatch_in(&ctx, from("bug", new_args("crash", "")), Standing::Outside).unwrap();
    let task = task(&ctx);
    assert_eq!(task.base.subject, "crash");
    assert_eq!(task.base.body, "Steps:");
    assert!(task.base.labels.contains("bug"));
    assert_eq!(task.priority.get(), 2);
    assert_eq!(task.due.map(|d| d.to_text()).as_deref(), Some("2026-09-19"));
    assert_eq!(
        task.deadline.map(|d| d.to_string()).as_deref(),
        Some("2026-09-30")
    );
    assert_eq!(task.base.path.as_str(), "bugs/");
}

#[test]
fn each_flag_given_replaces_the_templates_value() {
    let ctx = holding(vec![("bug", TemplateOwner::Person, bug())]);
    let args = NewArgs {
        body: "mine".into(),
        labels: vec!["urgent".into()],
        priority: Some(Priority::HIGHEST),
        due: Some("2026-10-01".into()),
        deadline: Some("2026-10-02".into()),
        ..new_args("crash", "inbox")
    };
    dispatch_in(&ctx, from("bug", args), Standing::Outside).unwrap();
    let task = task(&ctx);
    assert_eq!(task.base.body, "mine");
    assert_eq!(task.base.labels.iter().collect::<Vec<_>>(), ["urgent"]);
    assert_eq!(task.priority.get(), 1);
    assert_eq!(task.due.map(|d| d.to_text()).as_deref(), Some("2026-10-01"));
    assert_eq!(
        task.deadline.map(|d| d.to_string()).as_deref(),
        Some("2026-10-02")
    );
    assert_eq!(task.base.path.as_str(), "inbox/");
}

#[test]
fn a_templates_path_starts_from_the_project() {
    let ctx = holding(vec![("bug", TemplateOwner::Project, bug())]);
    let inside = Standing::Inside(Project {
        folder: "/work/app".into(),
        settings: ProjectSettings { path: "app".into() },
    });
    dispatch_in(&ctx, from("bug", new_args("crash", "")), inside).unwrap();
    assert_eq!(task(&ctx).base.path.as_str(), "app/bugs/");
}

#[test]
fn an_unknown_name_lists_the_templates_that_exist() {
    let ctx = holding(vec![
        ("fix", TemplateOwner::Project, bug()),
        ("chore", TemplateOwner::Person, bug()),
    ]);
    let Err(CliError::Usage(text)) =
        dispatch_in(&ctx, from("bug", new_args("x", "")), Standing::Outside)
    else {
        panic!("not refused")
    };
    assert_eq!(
        text,
        "no template named bug; the templates are fix (the project's) and chore (yours)"
    );
}

#[test]
fn a_name_that_leaves_the_folder_is_refused_before_any_lookup() {
    let (ctx, looked_up) = watched(vec![]);
    let refused = dispatch_in(&ctx, from("../../x", new_args("x", "")), Standing::Outside);
    assert!(matches!(refused, Err(CliError::Usage(_))));
    assert!(looked_up.borrow().is_empty());
}

mod tokens;
