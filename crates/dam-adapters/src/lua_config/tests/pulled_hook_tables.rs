use dam_application::{ConfigError, Expected, Misread, PulledChanges, PulledHook};

use super::hook_tables::{in_file, misread};
use super::support::hooked;

fn pulled(hook: &str) -> Option<PulledHook> {
    hooked("task_pulled", hook).load().unwrap().task_pulled
}

#[test]
fn the_pulled_hook_is_a_function_or_a_table_that_runs_one() {
    let file = || "hooks/task_pulled.lua".to_string();
    assert_eq!(
        hooked("task_new", "function() end")
            .load()
            .unwrap()
            .task_pulled,
        None
    );
    assert_eq!(
        pulled("function() end"),
        Some(PulledHook {
            changes: None,
            file: file(),
        })
    );
    assert_eq!(
        pulled("{ run = function() end }"),
        Some(PulledHook {
            changes: None,
            file: file(),
        })
    );
}

#[test]
fn the_pulled_hook_takes_no_on_failure_because_it_always_warns() {
    assert_eq!(
        misread(
            "task_pulled",
            "{ run = function() end, on_failure = 'refuse' }"
        ),
        Misread::AlwaysWarns {
            key: "on_failure".into(),
            at: in_file("task_pulled"),
        }
    );
}

#[test]
fn a_pulled_hook_table_needs_a_function_to_run() {
    assert_eq!(
        misread("task_pulled", "{ }"),
        Misread::Missing {
            key: "run".into(),
            at: in_file("task_pulled"),
        }
    );
    assert_eq!(
        hooked("task_pulled", "'notify'").refusal(),
        ConfigError::NotAHook {
            file: "hooks/task_pulled.lua".into(),
            returned: "a string".into(),
        }
    );
}

#[test]
fn the_pulled_hook_reads_where_its_changes_go() {
    let changes = |written: &str| {
        pulled(&format!(
            "{{ run = function() end, changes = '{written}' }}"
        ))
        .unwrap()
        .changes
    };
    assert_eq!(changes("committed"), Some(PulledChanges::Committed));
    assert_eq!(changes("working"), Some(PulledChanges::Working));
    assert_eq!(
        misread(
            "task_pulled",
            "{ run = function() end, changes = 'staged' }"
        ),
        Misread::WrongType {
            key: "changes".into(),
            at: in_file("task_pulled"),
            expected: Expected::CommittedOrWorking,
        }
    );
}

#[test]
fn only_the_pulled_hook_takes_changes() {
    assert_eq!(
        misread("task_done", "{ run = function() end, changes = 'working' }"),
        Misread::UnknownKey {
            key: "changes".into(),
            at: in_file("task_done"),
        }
    );
}
