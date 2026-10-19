use super::fixtures::{mutation, sync_body, task, task_with_dates_cleared, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

#[test]
fn creates_updates_and_deletes_become_the_right_commands() {
    let api = FakeTodoist::new(sync_body());
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
                Some(task(&"2".repeat(40), "Work/", "milk", true)),
            ),
            &["done", "subject"],
        ),
        mutation("delete", 3, Some("p:p1"), None),
        mutation(
            "create",
            4,
            None,
            Some(task(&"4".repeat(40), "Nope/", "lost", false)),
        ),
        with_fields(
            mutation(
                "update",
                5,
                Some("i:i3"),
                Some(task_with_dates_cleared(&"5".repeat(40))),
            ),
            &["due"],
        ),
        with_fields(
            mutation(
                "update",
                6,
                Some("i:i4"),
                Some(task_with_dates_cleared(&"6".repeat(40))),
            ),
            &["deadline"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert_eq!(response.results.len(), 6);
    assert!(response.results[0].ok());
    assert_eq!(response.results[0].remote_id(), Some("i:new1"));
    assert!(response.results[1].ok(), "{:?}", response.results[1].why());
    assert!(response.results[2].ok());
    assert!(!response.results[3].ok());
    assert!(response.results[3].why().unwrap().contains("Nope"));
    assert!(response.results[4].ok());
    assert!(response.results[5].ok());

    let recorded = api.recorded.borrow();
    let add = recorded.command("item_add");
    assert_eq!(add["args"]["content"], "eggs");
    assert_eq!(add["args"]["project_id"], "p1");
    assert_eq!(add["args"]["priority"], 4, "dam p1 is API 4");
    assert_eq!(
        add["args"]["due"],
        serde_json::json!({"date": "2026-09-25"})
    );
    assert_eq!(add["args"]["labels"], serde_json::json!(["errand"]));
    assert!(
        add["args"].get("deadline").is_none(),
        "a create with no deadline omits the field rather than clearing one"
    );
    let update = recorded.command("item_update");
    assert_eq!(update["args"]["id"], "i1");
    assert_eq!(update["args"]["content"], "milk");
    assert!(
        update["args"].get("priority").is_none(),
        "only changed fields are sent"
    );
    assert!(recorded.types().contains(&"item_close"));
    assert!(recorded.types().contains(&"project_delete"));
    let cleared: Vec<&serde_json::Value> = recorded
        .executed
        .iter()
        .filter(|c| c["type"] == "item_update")
        .collect();
    assert_eq!(
        cleared[1]["args"],
        serde_json::json!({"id": "i3", "due": null})
    );
    assert_eq!(
        cleared[2]["args"],
        serde_json::json!({"id": "i4", "deadline": null})
    );
}

#[test]
fn a_depth_one_create_becomes_a_section_when_its_child_arrives_later_in_the_same_push() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![
        mutation(
            "create",
            1,
            None,
            Some(task(&"1".repeat(40), "Work/", "Later", false)),
        ),
        mutation(
            "create",
            2,
            None,
            Some(task(&"2".repeat(40), "Work/Later/", "first step", false)),
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(
        response.results.iter().all(|r| r.ok()),
        "{:?}",
        response.results
    );
    assert_eq!(response.results[0].remote_id(), Some("s:new1"));
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.types(), vec!["section_add", "item_add"]);
    assert_eq!(recorded.command("item_add")["args"]["section_id"], "new1");
}

#[test]
fn clearing_done_becomes_the_uncomplete_command() {
    let api = FakeTodoist::new(sync_body());
    let response = push(
        &api,
        vec![with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["done"],
        )],
    )
    .unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.command("item_uncomplete")["args"]["id"], "i1");
    assert!(!recorded.types().contains(&"item_close"));
}
