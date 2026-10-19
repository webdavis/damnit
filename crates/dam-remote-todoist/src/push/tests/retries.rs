use dam_protocol::{WireObject, WireTask};

use super::fixtures::{mutation, sync_body, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

const EVERY_FIELD: [&str; 9] = [
    "subject",
    "body",
    "path",
    "labels",
    "done",
    "priority",
    "due",
    "deadline",
    "recurrence",
];

fn as_pulled(oid: &str, path: &str, subject: &str) -> WireObject {
    WireObject {
        oid: oid.into(),
        remote_id: None,
        kind: "task".into(),
        subject: subject.into(),
        body: String::new(),
        path: path.into(),
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done: false,
            completed_at: None,
            priority: 4,
            due: None,
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

#[test]
fn a_retry_declaring_every_field_with_only_the_name_differing_is_ok() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![with_fields(
        mutation(
            "update",
            4,
            Some("p:p1"),
            Some(as_pulled(&"4".repeat(40), "", "Chores")),
        ),
        &EVERY_FIELD,
    )];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    assert_eq!(
        api.recorded.borrow().command("project_update")["args"],
        serde_json::json!({"id": "p1", "name": "Chores"})
    );
}

#[test]
fn a_path_change_among_every_declared_field_is_refused_for_the_path_only_until_reverted() {
    let api = FakeTodoist::new(sync_body());
    let moved = with_fields(
        mutation(
            "update",
            5,
            Some("p:p1"),
            Some(as_pulled(&"5".repeat(40), "Home/", "Work")),
        ),
        &EVERY_FIELD,
    );
    let response = push(&api, vec![moved]).unwrap();
    assert!(!response.results[0].ok());
    assert_eq!(
        response.results[0].why(),
        Some(
            "a Todoist project carries only its name and whether it is archived; path did not change"
        )
    );
    let reverted = with_fields(
        mutation(
            "update",
            6,
            Some("p:p1"),
            Some(as_pulled(&"5".repeat(40), "", "Work")),
        ),
        &EVERY_FIELD,
    );
    let response = push(&api, vec![reverted]).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
}

#[test]
fn an_uncarried_field_counts_as_changed_only_when_it_differs_from_the_pulled_value() {
    let api = FakeTodoist::new(sync_body());
    let mut dated = as_pulled(&"7".repeat(40), "", "Work");
    dated.task.as_mut().unwrap().due = Some("2026-09-25".into());
    let mutations = vec![
        with_fields(
            mutation(
                "update",
                7,
                Some("p:p1"),
                Some(as_pulled(&"8".repeat(40), "", "Work")),
            ),
            &EVERY_FIELD,
        ),
        with_fields(
            mutation("update", 8, Some("p:p1"), Some(dated)),
            &EVERY_FIELD,
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok(), "{:?}", response.results[0].why());
    assert!(!response.results[1].ok());
    assert_eq!(
        response.results[1].why(),
        Some(
            "a Todoist project carries only its name and whether it is archived; due did not change"
        )
    );
}
