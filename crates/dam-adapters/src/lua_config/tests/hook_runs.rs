use std::time::Duration;

use dam_application::{ConfigFile, FunctionFailure, FunctionFault, History, HookEvent};

use super::filter_support::{FixedHistory, every_field_task};
use super::support::{Folder, config};

const LIMIT: Duration = Duration::from_secs(5);

fn hooked(task_done: &str) -> Folder {
    config(&format!(
        "return {{\n  hook = {{\n    task_done = {task_done},\n  }},\n}}\n"
    ))
}

fn run_within(folder: &Folder, limit: Duration) -> Result<(), FunctionFailure> {
    let loaded = folder.file().limiting_functions_to(limit).load().unwrap();
    let histories = FixedHistory(History::Recorded(vec![]));
    loaded
        .hooks
        .run(HookEvent::TaskDone, &every_field_task(), &histories)
}

fn run(task_done: &str) -> Result<(), FunctionFailure> {
    run_within(&hooked(task_done), LIMIT)
}

#[test]
fn a_hook_that_returns_nothing_succeeds_written_either_way() {
    assert_eq!(run("function(task, dam) end"), Ok(()));
    assert_eq!(
        run("{ run = function() return nil end, on_failure = 'refuse' }"),
        Ok(())
    );
}

#[test]
fn a_hook_is_handed_the_task_and_dam() {
    let folder = Folder::new();
    let marker = folder.marker();
    let folder = folder.with_config(&format!(
        "return {{ hook = {{ task_done = function(task, dam)\n  \
           local file = io.open({marker:?}, 'w')\n  \
           file:write(task.subject, ' ', #dam.history(task))\n  \
           file:close()\n\
         end }} }}\n"
    ));
    assert_eq!(run_within(&folder, LIMIT), Ok(()));
    assert_eq!(std::fs::read_to_string(marker).unwrap(), "report 0");
}

#[test]
fn a_hook_that_returns_a_value_fails_naming_what_came_back() {
    let failure = run("{ run = function() return true end }").unwrap_err();
    assert_eq!(failure.line, Some(3));
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
    assert!(failure.file.ends_with("config.lua"), "{:?}", failure.file);
    assert_eq!(failure.line, Some(4));
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
