use super::*;

#[test]
fn it_runs_for_a_created_an_updated_and_a_completed_task() {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    tracked(&s, 2, "water plants");
    pull_objects(
        &s,
        vec![
            incoming("new", "r1"),
            done("water plants", "r2"),
            incoming("brand new", "r3"),
        ],
    );
    assert_eq!(
        calls(&s),
        [
            ("new".to_string(), Some("old".to_string()), "changed"),
            (
                "water plants".to_string(),
                Some("water plants".to_string()),
                "done"
            ),
            ("brand new".to_string(), None, "new"),
        ]
    );
}

#[test]
fn the_answer_counts_what_the_hook_did() {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    let report = pull_objects(&s, vec![incoming("new", "r1"), incoming("b", "r2")]);
    let run = report.task_pulled.unwrap();
    assert_eq!(
        (run.ran, run.changed, run.failed, run.not_reached),
        (2, 0, 0, 0)
    );
}

#[test]
fn the_answer_holds_no_hook_run_when_none_is_set_or_none_ran() {
    let s = pulled_hooked();
    tracked(&s, 1, "old");
    let unchanged = pull_objects(&s, vec![incoming("old", "r1")]);
    assert_eq!(unchanged.task_pulled, None);
    let s = Scenario::default();
    s.store.set_sync_token(&todoist(), Some("s0")).unwrap();
    let unhooked = pull_objects(&s, vec![incoming("new", "r1")]);
    assert_eq!(unhooked.task_pulled, None);
}
