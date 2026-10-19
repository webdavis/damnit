use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

#[test]
fn a_resent_mutation_repeats_its_uuids_and_creates_nothing_twice() {
    let api = FakeTodoist::new(sync_body());
    let first = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let again = first.clone();

    let one = push(&api, first).unwrap();
    assert!(one.results[0].ok());
    let two = push(&api, again).unwrap();

    let recorded = api.recorded.borrow();
    assert_eq!(
        recorded
            .types()
            .iter()
            .filter(|t| **t == "item_add")
            .count(),
        1,
        "the resend was deduplicated: {:?}",
        recorded.types()
    );
    assert_eq!(
        recorded.uuids_seen.len(),
        1,
        "the resend repeated the uuid rather than minting one"
    );
    assert!(
        !two.results[0].ok(),
        "a resend dam cannot learn an id from is not reported as pushed"
    );
}

#[test]
fn the_commands_of_one_mutation_have_distinct_uuids() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", true)),
    )];
    assert!(push(&api, mutations).unwrap().results[0].ok());
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.types(), vec!["item_add", "item_close"]);
    assert_eq!(recorded.uuids_seen.len(), 2);
}

#[test]
fn a_refused_command_fails_only_its_own_mutation() {
    let api = FakeTodoist::new(sync_body()).refusing("item_update");
    let mutations = vec![
        mutation(
            "create",
            1,
            None,
            Some(task(&"1".repeat(40), "Work/", "eggs", false)),
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        mutation("delete", 3, Some("p:p1"), None),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok());
    assert!(!response.results[1].ok());
    assert_eq!(response.results[1].why(), Some("refused by the test"));
    assert!(response.results[2].ok());
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.types(), vec!["item_add", "project_delete"]);
}

#[test]
fn a_mutation_is_pushed_only_when_every_one_of_its_commands_is_accepted() {
    let api = FakeTodoist::new(sync_body()).refusing("item_close");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", true)),
    )];
    let response = push(&api, mutations).unwrap();
    assert!(!response.results[0].ok());
    assert_eq!(response.results[0].why(), Some("refused by the test"));
    assert_eq!(api.recorded.borrow().types(), vec!["item_add"]);
}
