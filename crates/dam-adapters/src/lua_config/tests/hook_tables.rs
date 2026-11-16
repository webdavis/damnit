use dam_application::{ConfigError, Expected, HookConfig, HookEvent, Misread, OnFailure, Place};

use super::support::hooked;

pub(super) fn misread(event: &str, hook: &str) -> Misread {
    match hooked(event, hook).refusal() {
        ConfigError::Misread(misread) => *misread,
        other => panic!("not a misread: {other:?}"),
    }
}

pub(super) fn in_file(event: &str) -> Place {
    Place {
        file: format!("hooks/{event}.lua"),
        table: None,
    }
}

#[test]
fn a_hook_table_names_how_its_failure_is_handled() {
    let hooks = |on_failure: &str| {
        hooked(
            "task_changed",
            &format!("{{ run = function() end, on_failure = '{on_failure}' }}"),
        )
        .load()
        .unwrap()
        .hooks
    };
    assert_eq!(
        hooks("warn"),
        [HookConfig {
            event: HookEvent::TaskChanged,
            on_failure: Some(OnFailure::Warn),
            file: "hooks/task_changed.lua".into(),
        }]
    );
    assert_eq!(hooks("refuse")[0].on_failure, Some(OnFailure::Refuse));
}

#[test]
fn a_hook_table_takes_only_run_and_on_failure() {
    let at = in_file("task_done");
    assert_eq!(
        misread("task_done", "{ run = function() end, loud = true }"),
        Misread::UnknownKey {
            key: "loud".into(),
            at: at.clone(),
        }
    );
    assert_eq!(
        misread("task_done", "{ on_failure = 'warn' }"),
        Misread::Missing {
            key: "run".into(),
            at: at.clone(),
        }
    );
    assert_eq!(
        misread("task_done", "{ run = 'notify' }"),
        Misread::WrongType {
            key: "run".into(),
            at: at.clone(),
            expected: Expected::Function,
        }
    );
    assert_eq!(
        misread(
            "task_done",
            "{ run = function() end, on_failure = 'ignore' }"
        ),
        Misread::WrongType {
            key: "on_failure".into(),
            at,
            expected: Expected::WarnOrRefuse,
        }
    );
}
