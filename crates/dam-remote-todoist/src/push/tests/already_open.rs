use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

fn reopened(remote_id: &str) -> Vec<String> {
    let api = FakeTodoist::new(sync_body());
    let reopen = with_fields(
        mutation(
            "update",
            1,
            Some(remote_id),
            Some(task(&"1".repeat(40), "Work/", "milk", false)),
        ),
        &["done"],
    );
    let response = push(&api, vec![reopen], &mut |_| {}).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    let recorded = api.recorded.borrow();
    recorded.types().into_iter().map(str::to_string).collect()
}

#[test]
fn a_task_the_tree_shows_open_is_not_sent_an_uncomplete() {
    assert_eq!(reopened("i:i1"), Vec::<String>::new());
}

#[test]
fn a_task_the_tree_leaves_out_as_completed_is_still_sent_an_uncomplete() {
    assert_eq!(reopened("i:i9"), vec!["item_uncomplete"]);
}

#[test]
fn a_project_the_tree_shows_unarchived_is_not_sent_an_unarchive() {
    assert_eq!(reopened("p:p1"), Vec::<String>::new());
}
