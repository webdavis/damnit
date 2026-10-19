use super::fixtures::{mutation, sync_body, task, with_fields};
use crate::api::fake::FakeTodoist;
use crate::push::push;

#[test]
fn a_project_path_change_and_a_section_done_change_are_refused_and_send_nothing() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![
        with_fields(
            mutation(
                "update",
                1,
                Some("p:p1"),
                Some(task(&"1".repeat(40), "Home/", "Work", false)),
            ),
            &["path"],
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("s:s1"),
                Some(task(&"2".repeat(40), "Work/", "Now", true)),
            ),
            &["done"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert_eq!(response.results.len(), 2);
    assert!(!response.results[0].ok());
    assert_eq!(
        response.results[0].why(),
        Some(
            "a Todoist project carries only its name and whether it is archived; path did not change"
        )
    );
    assert!(!response.results[1].ok());
    assert_eq!(
        response.results[1].why(),
        Some("a Todoist section carries only its name; done did not change")
    );
    assert!(api.recorded.borrow().executed.is_empty());
}

#[test]
fn a_rename_with_a_path_change_sends_the_rename_and_is_refused_for_the_path_only() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![with_fields(
        mutation(
            "update",
            3,
            Some("p:p1"),
            Some(task(&"3".repeat(40), "Home/", "Chores", false)),
        ),
        &["subject", "path"],
    )];
    let response = push(&api, mutations).unwrap();
    assert!(!response.results[0].ok());
    assert_eq!(
        response.results[0].why(),
        Some(
            "a Todoist project carries only its name and whether it is archived; path did not change"
        )
    );
    let recorded = api.recorded.borrow();
    assert_eq!(recorded.types(), vec!["project_update"]);
    assert_eq!(
        recorded.command("project_update")["args"],
        serde_json::json!({"id": "p1", "name": "Chores"})
    );
}

#[test]
fn a_hostile_remote_id_is_refused_and_never_reaches_a_command() {
    let api = FakeTodoist::new(sync_body());
    let mutations = vec![
        mutation("delete", 1, Some("i:../../projects/p1"), None),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1?force=true"),
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    for result in &response.results {
        assert!(!result.ok(), "{result:?}");
        assert!(
            result.why().unwrap().contains("unusable Todoist id"),
            "{result:?}"
        );
    }
    assert!(api.recorded.borrow().executed.is_empty());
}
