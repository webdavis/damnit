use dam_application::{Expected, Misread, PulledHook};

use super::support::{config, in_table};

fn pulled(text: &str) -> Option<PulledHook> {
    config(&format!("return {{ hook = {text} }}"))
        .load()
        .unwrap()
        .task_pulled
}

fn refusal(text: &str) -> Misread {
    match config(&format!("return {{ hook = {text} }}")).refusal() {
        dam_application::ConfigError::Misread(misread) => *misread,
        other => panic!("not a misread: {other:?}"),
    }
}

#[test]
fn the_pulled_hook_is_a_function_or_a_table_that_runs_one() {
    assert_eq!(pulled("{ task_new = function() end }"), None);
    assert_eq!(pulled("{ task_pulled = function() end }"), Some(PulledHook));
    assert_eq!(
        pulled("{ task_pulled = { run = function() end } }"),
        Some(PulledHook)
    );
}

#[test]
fn the_pulled_hook_takes_no_on_failure_because_it_always_warns() {
    assert_eq!(
        refusal("{ task_pulled = { run = function() end, on_failure = 'refuse' } }"),
        Misread::AlwaysWarns {
            key: "on_failure".into(),
            at: in_table("hook.task_pulled", "config.lua"),
        }
    );
}

#[test]
fn a_pulled_hook_table_needs_a_function_to_run() {
    let at = in_table("hook.task_pulled", "config.lua");
    assert_eq!(
        refusal("{ task_pulled = { } }"),
        Misread::Missing {
            key: "run".into(),
            at: at.clone(),
        }
    );
    assert_eq!(
        refusal("{ task_pulled = 'notify' }"),
        Misread::WrongType {
            key: "task_pulled".into(),
            at: in_table("hook", "config.lua"),
            expected: Expected::FunctionOrTable,
        }
    );
}
