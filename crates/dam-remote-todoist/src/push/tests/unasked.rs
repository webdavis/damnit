use dam_protocol::Verdict;

use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::{FakeTodoist, answered};
use crate::push::push;

fn unasked(verdict: &Verdict) -> bool {
    match verdict {
        Verdict::Refused { unasked, .. } => *unasked,
        other => panic!("expected a refusal: {other:?}"),
    }
}

#[test]
fn a_refusal_made_while_planning_says_todoist_was_not_asked() {
    let api = FakeTodoist::new(sync_body());
    let create = mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Nowhere/", "eggs", false)),
    );
    let response = push(&api, vec![create], &mut |_| {}).unwrap();
    assert!(unasked(&response.results[0].verdict));
    assert_eq!(api.recorded.borrow().writes, 0);
}

#[test]
fn a_write_todoist_turned_away_whole_says_todoist_gave_no_verdict() {
    for (status, why) in [(401, "unauthorized"), (400, "bad request")] {
        let api = FakeTodoist::new(sync_body()).failing_write(1, answered(status, why));
        let update = with_fields(
            mutation(
                "update",
                1,
                Some("i:i1"),
                Some(task(&"1".repeat(40), "Work/", "oat", false)),
            ),
            &["subject"],
        );
        let response = push(&api, vec![update], &mut |_| {}).unwrap();
        assert!(unasked(&response.results[0].verdict), "{status}");
    }
}

#[test]
fn a_command_todoist_turned_away_carries_no_mark() {
    let api = FakeTodoist::new(sync_body()).refusing("item_update");
    let update = with_fields(
        mutation(
            "update",
            1,
            Some("i:i1"),
            Some(task(&"1".repeat(40), "Work/", "oat", false)),
        ),
        &["subject"],
    );
    let response = push(&api, vec![update], &mut |_| {}).unwrap();
    assert!(!unasked(&response.results[0].verdict));
}

#[test]
fn a_plan_that_sent_commands_and_then_refused_carries_no_mark() {
    let api = FakeTodoist::new(sync_body());
    let rename_and_move = with_fields(
        mutation(
            "update",
            3,
            Some("p:p1"),
            Some(task(&"3".repeat(40), "Home/", "Chores", false)),
        ),
        &["subject", "path"],
    );
    let response = push(&api, vec![rename_and_move], &mut |_| {}).unwrap();
    assert_eq!(api.recorded.borrow().types(), vec!["project_update"]);
    assert!(!unasked(&response.results[0].verdict));
}

#[test]
fn a_container_refusal_that_sent_nothing_says_todoist_was_not_asked() {
    let api = FakeTodoist::new(sync_body());
    let move_only = with_fields(
        mutation(
            "update",
            1,
            Some("p:p1"),
            Some(task(&"1".repeat(40), "Home/", "Work", false)),
        ),
        &["path"],
    );
    let response = push(&api, vec![move_only], &mut |_| {}).unwrap();
    assert!(api.recorded.borrow().executed.is_empty());
    assert!(unasked(&response.results[0].verdict));
}

#[test]
fn a_removal_the_tree_shows_would_take_contents_says_todoist_was_not_asked() {
    let api = FakeTodoist::new(sync_body());
    let removal = mutation("delete", 1, Some("p:p1"), None);
    let response = push(&api, vec![removal], &mut |_| {}).unwrap();
    assert_eq!(api.recorded.borrow().writes, 0);
    assert!(unasked(&response.results[0].verdict));
}
