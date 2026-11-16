use std::time::Duration;

use dam_application::{
    ConfigFile, EditRequest, FunctionFailure, FunctionFault, History, HookEvent,
};

use super::filter_support::{FixedHistory, every_field_task, handed};
use super::support::Folder;

const LIMIT: Duration = Duration::from_secs(5);

fn hooked(task_done: &str) -> Folder {
    super::support::hooked("task_done", task_done)
}

fn run_within(folder: &Folder, limit: Duration) -> Result<Option<EditRequest>, FunctionFailure> {
    let loaded = folder.file().limiting_functions_to(limit).load().unwrap();
    let histories = FixedHistory(History::Recorded(vec![]));
    loaded.hooks.run(
        HookEvent::TaskDone,
        &handed(&every_field_task()),
        &histories,
    )
}

pub(super) fn run(task_done: &str) -> Result<Option<EditRequest>, FunctionFailure> {
    run_within(&hooked(task_done), LIMIT)
}

#[test]
fn a_hook_that_returns_nothing_succeeds_written_either_way() {
    assert_eq!(run("function(task, dam) end"), Ok(None));
    assert_eq!(
        run("{ run = function() return nil end, on_failure = 'refuse' }"),
        Ok(None)
    );
}

#[test]
fn a_hook_is_handed_the_task_and_dam() {
    let folder = Folder::new();
    let marker = folder.marker();
    let folder = folder.with_hook(
        "task_done.lua",
        &format!(
            "return function(task, dam)\n  \
           local file = io.open({marker:?}, 'w')\n  \
           file:write(task.subject, ' ', #dam.history(task))\n  \
           file:close()\n\
         end\n"
        ),
    );
    assert_eq!(run_within(&folder, LIMIT), Ok(None));
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "report 0");
}

#[test]
fn a_hook_that_returns_a_value_fails_naming_what_came_back() {
    let failure = run("{ run = function() return true end }").unwrap_err();
    assert_eq!(failure.line, Some(1));
    assert_eq!(
        failure.fault,
        FunctionFault::Returned {
            returned: "a boolean".into()
        }
    );
}

#[test]
fn a_hook_that_raises_gives_its_line_and_its_message() {
    let failure = run("function(task)\n      error('no bell')\n    end").unwrap_err();
    assert!(
        failure.file.ends_with("hooks/task_done.lua"),
        "{:?}",
        failure.file
    );
    assert_eq!(failure.line, Some(2));
    assert_eq!(
        failure.fault,
        FunctionFault::Raised {
            message: "no bell".into()
        }
    );
}

#[test]
fn a_hook_that_never_returns_is_stopped_at_the_limit() {
    let folder = hooked("function() while true do end end");
    let failure = run_within(&folder, Duration::from_millis(200)).unwrap_err();
    assert_eq!(failure.fault, FunctionFault::RanOver);
}

#[test]
fn a_hook_is_handed_category_overdue_and_blocked_as_match_is() {
    let folder = Folder::new();
    let marker = folder.marker();
    let folder = folder.with_hook(
        "task_done.lua",
        &format!(
            "return function(task, dam)\n  \
           local file = io.open({marker:?}, 'w')\n  \
           file:write(type(task.category), ' ', tostring(task.overdue), ' ', tostring(task.blocked))\n  \
           file:close()\n\
         end\n"
        ),
    );
    assert_eq!(run_within(&folder, LIMIT), Ok(None));
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "table true false");
}
