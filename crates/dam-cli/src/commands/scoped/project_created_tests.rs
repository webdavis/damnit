use dam_application::testing::task_in;
use dam_domain::{Object, Path};

use super::dispatch_in;
use super::tests::new_args;
use crate::args::Command;
use crate::project_scope::Standing;
use crate::testing::context;

fn with_project(subject: &str, project: &str) -> Command {
    let mut args = new_args(subject, "");
    args.scope.project = Some(Path::parse(project).unwrap());
    Command::New(args)
}

#[test]
fn new_with_project_creates_the_missing_project_task_and_says_so() {
    let ctx = context();
    let made = dispatch_in(&ctx, with_project("x", "pns/hooks/"), Standing::Outside).unwrap();
    assert_eq!(made.data["path"], "pns/hooks/");
    assert_eq!(made.data["project_created"]["path"], "pns/hooks/");
    let created = made.data["project_created"]["oid"].as_str().unwrap();
    assert_eq!(
        made.standard_error_lines,
        [format!(
            "created task {} to hold the project pns/hooks/",
            dam_domain::Oid::parse(created).unwrap().short()
        )]
    );
    assert_eq!(ctx.store.all().unwrap().len(), 2);
}

#[test]
fn new_with_project_already_held_creates_nothing_more() {
    let ctx = context();
    ctx.store
        .put(&Object::Task(task_in(9, "hooks", "pns")))
        .unwrap();
    let made = dispatch_in(&ctx, with_project("x", "pns/hooks/"), Standing::Outside).unwrap();
    assert!(made.data.get("project_created").is_none());
    assert!(made.standard_error_lines.is_empty());
    assert_eq!(ctx.store.all().unwrap().len(), 2);
}
