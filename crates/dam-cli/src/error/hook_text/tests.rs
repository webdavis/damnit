use std::path::PathBuf;

use dam_application::{FunctionFailure, FunctionFault, HookEvent, HookFailure};

use super::HookText;
use crate::commands::hook_error_row::HookErrorRow;

const ODD: &str = "no\u{1b}[2J\rwater\nhere";

fn failure(file: &str, fault: FunctionFault) -> HookFailure {
    HookFailure {
        event: HookEvent::TaskNew,
        failure: FunctionFailure {
            file: PathBuf::from(file),
            line: Some(1),
            fault,
        },
    }
}

fn every_fault() -> [FunctionFault; 3] {
    [
        FunctionFault::Raised {
            message: ODD.into(),
        },
        FunctionFault::Returned {
            returned: ODD.into(),
        },
        FunctionFault::Requested { why: ODD.into() },
    ]
}

#[test]
fn a_projects_hook_never_prints_a_control_character_it_raised() {
    for fault in every_fault() {
        let theirs = failure("/work/app/.damhooks/task_new.lua", fault);
        let line = HookText(&theirs).to_string();
        assert!(!line.chars().any(char::is_control), "{line:?}");
        assert!(
            line.contains("no\u{fffd}[2J\u{fffd}water\u{fffd}here"),
            "{line:?}"
        );
        let row = HookErrorRow::of(&theirs.failure).line();
        assert!(!row.chars().any(char::is_control), "{row:?}");
    }
}

#[test]
fn a_persons_own_hook_prints_what_it_raised_as_it_is() {
    let mine = failure(
        "/home/ada/.config/dam/hooks/task_new.lua",
        FunctionFault::Raised {
            message: ODD.into(),
        },
    );
    assert!(HookText(&mine).to_string().ends_with(ODD));
}
