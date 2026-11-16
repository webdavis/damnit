use dam_application::NewTask;
use dam_domain::Path;

use super::{context, folders, here, subjects_and_paths, trash_project_file};
use crate::args::InitArgs;
use crate::commands::init::run;

#[test]
fn a_path_an_object_already_holds_creates_no_task() {
    let (ctx, folders) = (context(), folders());
    run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    trash_project_file(&folders.project);
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    assert_eq!(subjects_and_paths(&ctx).len(), 1);
    assert_eq!(report.data["task"], serde_json::Value::Null);
    assert_eq!(report.data["tasks_already_there"], 0);
}

#[test]
fn joining_a_place_that_holds_tasks_says_how_many_and_how_to_use_another() {
    let (ctx, folders) = (context(), folders());
    for subject in ["damnit", "damnit/fix the roof", "damnit/paint"] {
        let (path, subject) = subject
            .rsplit_once('/')
            .map_or(("", subject), |(p, s)| (p, s));
        ctx.dam()
            .new_task(NewTask {
                parent: None,
                subject: subject.into(),
                path: Path::root().join(path).unwrap_or_else(|_| Path::root()),
                priority: Default::default(),
                due: None,
                deadline: None,
                labels: Default::default(),
                body: String::new(),
                id: None,
            })
            .unwrap();
    }
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    let file = folders.project.join(".dam.lua");
    assert_eq!(report.data["tasks_already_there"], 2);
    assert_eq!(
        report.human,
        [
            format!("wrote {} with path damnit/", file.display()),
            format!(
                "damnit/ was already there, with 2 tasks under it; this folder now shares that place. To use another place, change path in {}",
                file.display()
            ),
        ]
        .join("\n")
    );
}

#[test]
fn a_new_place_has_no_count_of_tasks_already_there() {
    let (ctx, folders) = (context(), folders());
    let report = run(
        &ctx,
        InitArgs { path: None },
        here(&folders, &folders.project),
    )
    .unwrap();
    assert_eq!(report.data["tasks_already_there"], serde_json::Value::Null);
}
