use dam_application::{History, HookEvent};

use super::filter_support::{FixedHistory, every_field_task, run_filter};
use super::support::Folder;

const SEEN: &str = "seen";

fn run_task_done(folder: &Folder) {
    let histories = FixedHistory(History::Recorded(vec![]));
    folder
        .loaded()
        .hooks
        .run(HookEvent::TaskDone, &every_field_task(), &histories)
        .unwrap();
}

fn hooked(before: &str, body: &str) -> Folder {
    let folder = Folder::new();
    let seen = folder.path(SEEN);
    let body = body.replace("SEEN", &format!("{seen:?}"));
    folder.with_config(&format!(
        "{before}\nreturn {{ hook = {{ task_done = function()\n{body}\nend }} }}\n"
    ))
}

#[test]
fn a_command_os_execute_starts_in_a_hook_carries_the_event() {
    let folder = hooked("", "os.execute('printf %s \"$DAM_HOOK\" > ' .. SEEN)");
    run_task_done(&folder);
    assert_eq!(folder.text(SEEN), "task_done");
}

#[test]
fn a_command_io_popen_starts_in_a_hook_carries_the_event_and_hands_back_luas_handle() {
    let folder = hooked(
        "",
        "local started = io.popen('printf %s \"$DAM_HOOK\"')\n\
         local read = started:read('a')\n\
         started:close()\n\
         local file = io.open(SEEN, 'w')\n\
         file:write(read)\n\
         file:close()",
    );
    run_task_done(&folder);
    assert_eq!(folder.text(SEEN), "task_done");
}

#[test]
fn os_execute_saved_while_the_config_loads_still_marks_what_a_hook_starts() {
    let folder = hooked(
        "local execute = os.execute",
        "execute('printf %s \"$DAM_HOOK\" > ' .. SEEN)",
    );
    run_task_done(&folder);
    assert_eq!(folder.text(SEEN), "task_done");
}

#[test]
fn os_execute_with_no_command_still_answers_whether_a_shell_exists() {
    let folder = hooked(
        "",
        "local file = io.open(SEEN, 'w')\nfile:write(tostring(os.execute()))\nfile:close()",
    );
    run_task_done(&folder);
    assert_eq!(folder.text(SEEN), "true");
}

#[test]
fn a_command_a_filter_starts_carries_no_event() {
    let dir = tempfile::tempdir().unwrap();
    let seen = dir.path().join(SEEN);
    let body = format!("os.execute('printf %s \"${{DAM_HOOK-none}}\" > ' .. {seen:?})");
    run_filter(&body, &[every_field_task()]).unwrap();
    assert_eq!(std::fs::read_to_string(seen).unwrap(), "none");
}

#[test]
fn a_command_started_after_a_hook_returns_carries_no_event() {
    let folder = Folder::new();
    let seen = folder.path(SEEN);
    let folder = folder.with_config(&format!(
        "return {{\n  hook = {{ task_done = function() end }},\n  \
           filter = {{ probe = {{ match = function()\n    \
             os.execute('printf %s \"${{DAM_HOOK-none}}\" > ' .. {seen:?})\n  \
           end }} }},\n}}\n"
    ));
    let loaded = folder.loaded();
    let histories = FixedHistory(History::Recorded(vec![]));
    let task = every_field_task();
    loaded
        .hooks
        .run(HookEvent::TaskDone, &task, &histories)
        .unwrap();
    loaded
        .filter_functions
        .keeps("probe", &[task], &histories)
        .unwrap();
    assert_eq!(folder.text(SEEN), "none");
}
