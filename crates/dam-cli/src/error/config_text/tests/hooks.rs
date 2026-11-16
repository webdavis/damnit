use dam_application::ConfigError;

use super::text;

#[test]
fn a_hook_table_in_config_lua_names_the_file_each_hook_moves_to() {
    assert_eq!(
        text(ConfigError::HookMoved {
            config: "config.lua".into(),
            events: vec!["task_new".into(), "task_pulled".into()],
        }),
        "config: config.lua: hook.task_new moves to hooks/task_new.lua, hook.task_pulled moves to \
         hooks/task_pulled.lua; put each hook's function or table after return in its own file \
         beside config.lua, remove the hook table, then run dam config validate"
    );
}

#[test]
fn a_hook_file_named_for_no_event_names_the_four_it_may_be() {
    assert_eq!(
        text(ConfigError::UnknownHookFile {
            file: "hooks/task_neww.lua".into(),
        }),
        "config: hooks/task_neww.lua names no event; a hook file is named task_new.lua, \
         task_changed.lua, task_done.lua or task_pulled.lua"
    );
}

#[test]
fn a_hook_file_that_returns_no_hook_says_what_it_returned() {
    assert_eq!(
        text(ConfigError::NotAHook {
            file: "hooks/task_done.lua".into(),
            returned: "a string".into(),
        }),
        "config: hooks/task_done.lua must return a function, or a table holding one as run, and it \
         returned a string"
    );
}
