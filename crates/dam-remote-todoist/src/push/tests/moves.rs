use super::fixtures::{mutation, sync_body_with_two_projects, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

#[test]
fn a_path_change_moves_the_task_through_the_move_command() {
    let api = FakeTodoist::new(sync_body_with_two_projects());
    let mutations = vec![with_fields(
        mutation(
            "update",
            9,
            Some("i:i1"),
            Some(task(&"9".repeat(40), "Home/", "milk", false)),
        ),
        &["path"],
    )];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.types(), vec!["item_move"]);
    assert_eq!(
        recorded.command("item_move")["args"],
        serde_json::json!({"id": "i1", "project_id": "p2"})
    );
}

#[test]
fn a_path_and_subject_change_moves_first_then_updates_the_content() {
    let api = FakeTodoist::new(sync_body_with_two_projects());
    let mutations = vec![with_fields(
        mutation(
            "update",
            8,
            Some("i:i1"),
            Some(task(&"8".repeat(40), "Home/", "oat", false)),
        ),
        &["path", "subject"],
    )];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    let recorded = api.recorded.borrow();
    assert_eq!(
        recorded.types(),
        vec!["item_move", "item_update"],
        "move takes exactly one container, so it goes on its own"
    );
    assert_eq!(
        recorded.command("item_move")["args"],
        serde_json::json!({"id": "i1", "project_id": "p2"})
    );
    assert_eq!(recorded.command("item_update")["args"]["content"], "oat");
}
