use super::*;

#[test]
fn in_plain_text_it_names_the_hook_and_how_many_tasks_it_runs_for() {
    let progress = HookProgressOnStandardError::for_format(Format::Human);
    assert_eq!(
        progress.line(HookEvent::TaskPulled, 2).as_deref(),
        Some("dam: running task_pulled for 2 tasks")
    );
}

#[test]
fn under_json_it_says_nothing() {
    let progress = HookProgressOnStandardError::for_format(Format::Json);
    assert_eq!(progress.line(HookEvent::TaskPulled, 2), None);
}
