use dam_application::{FunctionFailure, FunctionFault, HookEvent, HookFailure};

use super::{HookWarningsOnStandardError, warning};
use crate::output::Format;

#[test]
fn a_failed_pulled_hook_warns_where_it_failed_and_that_the_pull_is_kept() {
    let failure = HookFailure {
        event: HookEvent::TaskPulled,
        failure: FunctionFailure {
            file: "/home/config.lua".into(),
            line: Some(3),
            fault: FunctionFault::Raised {
                message: "no bell".into(),
            },
        },
    };
    assert_eq!(
        warning(&failure),
        "warning: hook task_pulled failed at /home/config.lua:3: no bell; the pull is kept"
    );
}

#[test]
fn a_failed_change_hook_warns_that_the_change_stands() {
    for event in [
        HookEvent::TaskNew,
        HookEvent::TaskChanged,
        HookEvent::TaskDone,
    ] {
        let failure = HookFailure {
            event,
            failure: FunctionFailure {
                file: "/home/config.lua".into(),
                line: None,
                fault: FunctionFault::Raised {
                    message: "no bell".into(),
                },
            },
        };
        assert!(
            warning(&failure).ends_with("no bell; the change stands"),
            "{}",
            warning(&failure)
        );
    }
}

#[test]
fn under_quiet_it_warns_of_nothing_but_says_it_otherwise() {
    let failure = HookFailure {
        event: HookEvent::TaskDone,
        failure: FunctionFailure {
            file: "/home/config.lua".into(),
            line: None,
            fault: FunctionFault::Raised {
                message: "no bell".into(),
            },
        },
    };
    let quiet = HookWarningsOnStandardError::for_format(Format::Quiet);
    assert_eq!(quiet.line(&failure), None);
    let json = HookWarningsOnStandardError::for_format(Format::Json);
    assert_eq!(json.line(&failure), Some(warning(&failure)));
}
