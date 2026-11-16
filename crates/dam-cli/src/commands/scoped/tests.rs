use dam_application::{Project, ProjectSettings};
use dam_domain::{Kind, Object, Path};

use super::dispatch_in;
use crate::args::{Command, NewArgs, ScopeArgs};
use crate::context::Context;
use crate::project_scope::Standing;
use crate::testing::context;

fn inside(path: &str) -> Standing {
    Standing::Inside(Project {
        folder: "/work/damnit".into(),
        settings: ProjectSettings { path: path.into() },
    })
}

pub(super) fn new_args(subject: &str, path: &str) -> NewArgs {
    NewArgs {
        subject: subject.into(),
        kind: Kind::Task,
        event: false,
        path: Path::parse(path).unwrap(),
        parent: None,
        due: None,
        priority: None,
        deadline: None,
        labels: vec![],
        body: String::new(),
        start: None,
        end: None,
        id: None,
        scope: ScopeArgs::default(),
    }
}

fn paths(ctx: &Context) -> Vec<String> {
    let mut all: Vec<String> = ctx
        .store
        .all()
        .unwrap()
        .iter()
        .map(|o: &Object| format!("{}{}", o.base().path, o.base().subject))
        .collect();
    all.sort();
    all
}

#[test]
fn a_task_made_inside_a_project_equals_one_made_outside_with_the_full_path() {
    let (made_inside, made_outside) = (context(), context());
    let command = Command::New(new_args("parse flags", "cli"));
    dispatch_in(&made_inside, command, inside("damnit")).unwrap();
    let command = Command::New(new_args("parse flags", "damnit/cli"));
    dispatch_in(&made_outside, command, Standing::Outside).unwrap();
    assert_eq!(paths(&made_inside), ["damnit/cli/parse flags"]);
    assert_eq!(
        made_inside.store.all().unwrap(),
        made_outside.store.all().unwrap()
    );
}

#[test]
fn a_new_task_with_no_path_lands_at_the_projects_path() {
    let ctx = context();
    dispatch_in(&ctx, Command::New(new_args("x", "")), inside("damnit")).unwrap();
    assert_eq!(paths(&ctx), ["damnit/x"]);
}

#[test]
fn global_reads_the_path_as_it_reads_outside() {
    let ctx = context();
    let mut args = new_args("x", "home");
    args.scope.global = true;
    dispatch_in(&ctx, Command::New(args), inside("damnit")).unwrap();
    assert_eq!(paths(&ctx), ["home/x"]);
}

#[test]
fn a_parent_with_no_path_places_the_task_as_it_does_outside() {
    let ctx = context();
    dispatch_in(&ctx, Command::New(new_args("home", "")), Standing::Outside).unwrap();
    let parent = ctx.store.all().unwrap()[0].oid().short().to_string();
    let mut args = new_args("bins", "");
    args.parent = Some(parent);
    dispatch_in(&ctx, Command::New(args), inside("damnit")).unwrap();
    assert_eq!(paths(&ctx), ["home", "home/bins"]);
}

#[test]
fn a_token_is_checked_on_the_path_the_project_fills_in() {
    use dam_application::testing::{allowed, holder};
    use dam_domain::Permission::{TaskCreate, TaskRead};
    let token = |paths: &[&str]| Context {
        access: Box::new(holder(vec![allowed(&[TaskCreate, TaskRead], Some(paths))])),
        ..context()
    };
    let in_the_project = token(&["damnit"]);
    let made = dispatch_in(
        &in_the_project,
        Command::New(new_args("x", "cli")),
        inside("damnit"),
    );
    assert_eq!(made.unwrap().data["path"], "damnit/cli/");
    let elsewhere = token(&["cli"]);
    let refused = dispatch_in(
        &elsewhere,
        Command::New(new_args("x", "cli")),
        inside("damnit"),
    );
    let refused = refused.unwrap_err();
    assert_eq!(refused.to_string(), "the token does not allow task:create");
    assert!(paths(&elsewhere).is_empty());
}
