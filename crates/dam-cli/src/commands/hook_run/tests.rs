use std::path::PathBuf;

use dam_application::{FunctionFailure, FunctionFault, HookEvent, ListedHookRun};
use serde_json::json;

use super::HookRun;
use crate::view::report;

fn counted(error: Option<FunctionFailure>) -> ListedHookRun {
    ListedHookRun {
        hook: HookEvent::TaskChanged,
        ran: 3,
        changed: 2,
        failed: 1,
        not_reached: 4,
        error,
    }
}

#[test]
fn the_answer_counts_what_the_run_did_on_one_line() {
    let report = report(&HookRun::of(&counted(None))).unwrap();
    assert_eq!(
        report.human,
        "task_changed: 3 ran, 2 changed, 1 failed, 4 not reached"
    );
    assert_eq!(
        report.data,
        json!({"hook": "task_changed", "ran": 3, "changed": 2, "failed": 1, "not_reached": 4, "error": null})
    );
}

#[test]
fn the_answer_names_the_first_error_by_the_hooks_file_and_line() {
    let failure = FunctionFailure {
        file: PathBuf::from("/home/ada/.config/dam/hooks/task_changed.lua"),
        line: Some(3),
        fault: FunctionFault::Raised {
            message: "no bell".into(),
        },
    };
    let report = report(&HookRun::of(&counted(Some(failure)))).unwrap();
    assert_eq!(
        report.data["error"],
        json!({"file": "/home/ada/.config/dam/hooks/task_changed.lua", "line": 3, "message": "no bell"})
    );
}
