use dam_application::{ConfigError, HookConfig, HookEvent, OnFailure, PulledHook};

use super::support::{Folder, config, hooked};

fn hooks_of(folder: &Folder) -> (Vec<HookConfig>, Option<PulledHook>) {
    let config = folder.load().unwrap();
    (config.hooks, config.task_pulled)
}

#[test]
fn each_event_is_read_from_its_own_file_written_either_way() {
    let folder = hooked("task_new", "function() end")
        .with_hook("task_changed.lua", "return function() end\n")
        .with_hook(
            "task_done.lua",
            "return { run = function() end, on_failure = 'refuse' }\n",
        )
        .with_hook(
            "task_pulled.lua",
            "return { run = function() end, changes = 'working' }\n",
        );
    let (local, pulled) = hooks_of(&folder);
    assert_eq!(
        local,
        [
            HookConfig {
                event: HookEvent::TaskNew,
                on_failure: None,
                file: "hooks/task_new.lua".into(),
            },
            HookConfig {
                event: HookEvent::TaskChanged,
                on_failure: None,
                file: "hooks/task_changed.lua".into(),
            },
            HookConfig {
                event: HookEvent::TaskDone,
                on_failure: Some(OnFailure::Refuse),
                file: "hooks/task_done.lua".into(),
            },
        ]
    );
    assert_eq!(
        pulled.map(|hook| (hook.changes().as_str(), hook.file)),
        Some(("working", "hooks/task_pulled.lua".to_string()))
    );
}

#[test]
fn no_hooks_folder_means_no_hooks() {
    assert_eq!(hooks_of(&config("return {}")), (Vec::new(), None));
}

#[test]
fn a_file_that_is_not_lua_is_not_a_hook() {
    let folder = Folder::new().with_hook("task_new.lua.bak", "not lua");
    assert_eq!(hooks_of(&folder), (Vec::new(), None));
}

#[test]
fn a_lua_file_named_for_no_event_is_refused_by_name() {
    let folder = Folder::new().with_hook("task_neww.lua", "return function() end\n");
    assert_eq!(
        folder.refusal(),
        ConfigError::UnknownHookFile {
            file: "hooks/task_neww.lua".into(),
        }
    );
}

#[test]
fn a_hook_table_in_config_lua_is_refused_naming_where_each_hook_moves() {
    let folder =
        config("return { hook = { task_new = function() end, task_pulled = function() end } }");
    assert_eq!(
        folder.refusal(),
        ConfigError::HookMoved {
            config: "config.lua".into(),
            events: vec!["task_new".into(), "task_pulled".into()],
        }
    );
}

#[test]
fn a_hook_file_that_returns_no_hook_is_refused() {
    let folder = hooked("task_done", "'notify'");
    assert_eq!(
        folder.refusal(),
        ConfigError::NotAHook {
            file: "hooks/task_done.lua".into(),
            returned: "a string".into(),
        }
    );
}

#[test]
fn a_hook_file_that_fails_to_load_names_its_file_and_line_and_no_secret() {
    let folder = Folder::new()
        .with_secrets("return { remote = { todoist = { api_token = 'sekrit-value' } } }")
        .with_config("return { remote = { todoist = { url = 'todoist::' } } }")
        .with_hook("task_new.lua", "local x = 1\nerror('no bell')\n");
    match folder.refusal() {
        ConfigError::StoppedLoading { path, line, reason } => {
            assert!(path.ends_with("hooks/task_new.lua"), "{path:?}");
            assert_eq!(line, Some(2));
            assert!(!format!("{reason:?}").contains("sekrit"), "{reason:?}");
        }
        other => panic!("not a stop: {other:?}"),
    }
}
