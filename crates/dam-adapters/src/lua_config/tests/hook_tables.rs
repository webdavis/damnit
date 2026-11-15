use dam_application::{Expected, HookConfig, HookEvent, Misread, OnFailure};

use super::support::{config, in_table};

fn hooks(text: &str) -> Vec<HookConfig> {
    config(&format!("return {{ hook = {text} }}"))
        .load()
        .unwrap()
        .hooks
}

fn refusal(text: &str) -> Misread {
    match config(&format!("return {{ hook = {text} }}")).refusal() {
        dam_application::ConfigError::Misread(misread) => *misread,
        other => panic!("not a misread: {other:?}"),
    }
}

#[test]
fn a_hook_is_a_function_or_a_table_that_runs_one() {
    assert_eq!(
        hooks(
            "{ task_new = function() end, \
               task_done = { run = function() end, on_failure = 'refuse' }, \
               task_changed = { run = function() end, on_failure = 'warn' } }"
        ),
        [
            HookConfig {
                event: HookEvent::TaskNew,
                on_failure: None,
            },
            HookConfig {
                event: HookEvent::TaskChanged,
                on_failure: Some(OnFailure::Warn),
            },
            HookConfig {
                event: HookEvent::TaskDone,
                on_failure: Some(OnFailure::Refuse),
            },
        ]
    );
}

#[test]
fn an_event_dam_does_not_know_is_refused() {
    assert_eq!(
        refusal("{ task_moved = function() end }"),
        Misread::UnknownKey {
            key: "task_moved".into(),
            at: in_table("hook", "config.lua"),
        }
    );
}

#[test]
fn a_hook_that_is_neither_a_function_nor_a_table_is_refused() {
    assert_eq!(
        refusal("{ task_new = 'notify' }"),
        Misread::WrongType {
            key: "task_new".into(),
            at: in_table("hook", "config.lua"),
            expected: Expected::FunctionOrTable,
        }
    );
}

#[test]
fn a_hook_table_takes_only_run_and_on_failure() {
    let at = in_table("hook.task_done", "config.lua");
    assert_eq!(
        refusal("{ task_done = { run = function() end, loud = true } }"),
        Misread::UnknownKey {
            key: "loud".into(),
            at: at.clone(),
        }
    );
    assert_eq!(
        refusal("{ task_done = { on_failure = 'warn' } }"),
        Misread::Missing {
            key: "run".into(),
            at: at.clone(),
        }
    );
    assert_eq!(
        refusal("{ task_done = { run = 'notify' } }"),
        Misread::WrongType {
            key: "run".into(),
            at: at.clone(),
            expected: Expected::Function,
        }
    );
    assert_eq!(
        refusal("{ task_done = { run = function() end, on_failure = 'ignore' } }"),
        Misread::WrongType {
            key: "on_failure".into(),
            at,
            expected: Expected::WarnOrRefuse,
        }
    );
}
