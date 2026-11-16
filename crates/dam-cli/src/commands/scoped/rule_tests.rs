mod listing;
mod new_task;
mod required;

use dam_application::{Project, ProjectSettings};
use dam_domain::Object;

use crate::args::{Command, NewArgs};
use crate::context::Context;
use crate::project_scope::Standing;

pub(super) fn inside(settings: ProjectSettings) -> Standing {
    Standing::Inside(Project {
        folder: "/work/app".into(),
        settings: ProjectSettings {
            path: "app".into(),
            ..settings
        },
    })
}

pub(super) fn global(args: NewArgs) -> Command {
    let mut args = args;
    args.scope.global = true;
    Command::New(args)
}

pub(super) fn task(ctx: &Context) -> dam_domain::Task {
    match ctx.store.all().unwrap().pop() {
        Some(Object::Task(task)) => task,
        other => panic!("{other:?}"),
    }
}
