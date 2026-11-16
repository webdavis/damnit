use dam_application::{ConfigError, HookEvent, HookFile, HookSetting, OnFailure, PulledChanges};

use super::support::{Folder, config};

fn listed(folder: &Folder) -> Vec<HookFile> {
    folder.file().hook_files().unwrap()
}

#[test]
fn each_hook_file_is_listed_in_event_order_with_its_setting() {
    let folder = Folder::new()
        .with_hook(
            "task_pulled.lua",
            "return { run = function() end, changes = 'working' }\n",
        )
        .with_hook("task_done.lua", "return function() end\n")
        .with_hook(
            "task_new.lua",
            "return { run = function() end, on_failure = 'refuse' }\n",
        );
    assert_eq!(
        listed(&folder),
        [
            HookFile {
                file: "hooks/task_new.lua".into(),
                event: Some(HookEvent::TaskNew),
                read: Ok(HookSetting::OnFailure(Some(OnFailure::Refuse))),
            },
            HookFile {
                file: "hooks/task_done.lua".into(),
                event: Some(HookEvent::TaskDone),
                read: Ok(HookSetting::OnFailure(None)),
            },
            HookFile {
                file: "hooks/task_pulled.lua".into(),
                event: Some(HookEvent::TaskPulled),
                read: Ok(HookSetting::Changes(Some(PulledChanges::Working))),
            },
        ]
    );
}

#[test]
fn a_hook_that_does_not_load_is_listed_with_why_beside_those_that_do() {
    let folder = Folder::new()
        .with_hook("task_new.lua", "return function() end\n")
        .with_hook("task_done.lua", "return 'notify'\n")
        .with_hook("task_neww.lua", "return function() end\n");
    assert_eq!(
        listed(&folder),
        [
            HookFile {
                file: "hooks/task_new.lua".into(),
                event: Some(HookEvent::TaskNew),
                read: Ok(HookSetting::OnFailure(None)),
            },
            HookFile {
                file: "hooks/task_done.lua".into(),
                event: Some(HookEvent::TaskDone),
                read: Err(ConfigError::NotAHook {
                    file: "hooks/task_done.lua".into(),
                    returned: "a string".into(),
                }),
            },
            HookFile {
                file: "hooks/task_neww.lua".into(),
                event: None,
                read: Err(ConfigError::UnknownHookFile {
                    file: "hooks/task_neww.lua".into(),
                }),
            },
        ]
    );
}

#[test]
fn a_broken_config_lua_does_not_hide_the_hooks() {
    let folder = config("return {").with_hook("task_new.lua", "return function() end\n");
    assert_eq!(listed(&folder).len(), 1);
}

#[test]
fn no_hooks_folder_lists_nothing() {
    assert_eq!(listed(&config("return {}")), []);
}
