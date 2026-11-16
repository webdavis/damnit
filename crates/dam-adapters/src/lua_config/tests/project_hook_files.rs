use dam_application::{History, HookEvent};

use super::filter_support::{FixedHistory, every_field_task, handed};
use super::support::Folder;
use crate::ProjectHookFolder;

fn project_with(task_done: &str) -> Folder {
    let folder = Folder::new();
    let hooks = folder.path("app").join(".damhooks");
    std::fs::create_dir_all(&hooks).unwrap();
    std::fs::write(hooks.join("task_done.lua"), task_done).unwrap();
    folder
}

#[test]
fn a_projects_hook_is_named_by_its_file_under_damhooks() {
    let folder = project_with("return function() end");
    let loaded = folder
        .file()
        .project_hooks(
            &ProjectHookFolder::read(&folder.path("app")).unwrap(),
            "Ada",
        )
        .unwrap();
    assert_eq!(loaded.hooks[0].file, ".damhooks/task_done.lua");
}

#[test]
fn a_failing_projects_hook_names_its_file_under_damhooks() {
    let folder = project_with("return function() error('no') end");
    let loaded = folder
        .file()
        .project_hooks(
            &ProjectHookFolder::read(&folder.path("app")).unwrap(),
            "Ada",
        )
        .unwrap();
    let histories = FixedHistory(History::Recorded(vec![]));
    let failure = loaded
        .run
        .run(
            HookEvent::TaskDone,
            &handed(&every_field_task()),
            &histories,
        )
        .unwrap_err();
    assert!(
        failure.file.ends_with("app/.damhooks/task_done.lua"),
        "{failure:?}"
    );
}

#[test]
fn an_edit_request_from_a_projects_hook_names_its_file_under_damhooks() {
    let folder = project_with("return function() return { labels = { 'x' } } end");
    let loaded = folder
        .file()
        .project_hooks(
            &ProjectHookFolder::read(&folder.path("app")).unwrap(),
            "Ada",
        )
        .unwrap();
    let histories = FixedHistory(History::Recorded(vec![]));
    let request = loaded
        .run
        .run(
            HookEvent::TaskDone,
            &handed(&every_field_task()),
            &histories,
        )
        .unwrap()
        .unwrap();
    assert!(
        request.file.ends_with("app/.damhooks/task_done.lua"),
        "{request:?}"
    );
}
