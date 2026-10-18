mod support;

mod fixtures;
mod loopback;

use dam_remote_todoist::api::TodoistApi;
use dam_remote_todoist::push::push;
use fixtures::{mutation, sync_body, task, todoist, with_fields};

#[test]
fn a_project_path_change_and_a_section_done_change_are_refused_and_send_nothing() {
    let _guard = support::guard(
        "a_project_path_change_and_a_section_done_change_are_refused_and_send_nothing",
    );
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
    assert!(!response.results[0].ok);
    assert_eq!(
        response.results[0].why.as_deref(),
        Some(
            "a Todoist project carries only its name and whether it is archived; path did not change"
        )
    );
    assert!(!response.results[1].ok);
    assert_eq!(
        response.results[1].why.as_deref(),
        Some("a Todoist section carries only its name; done did not change")
    );
    assert!(recorded.lock().unwrap().executed.is_empty());
}

#[test]
fn a_rename_with_a_path_change_sends_the_rename_and_is_refused_for_the_path_only() {
    let _guard = support::guard(
        "a_rename_with_a_path_change_sends_the_rename_and_is_refused_for_the_path_only",
    );
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
    assert!(!response.results[0].ok);
    assert_eq!(
        response.results[0].why.as_deref(),
        Some(
            "a Todoist project carries only its name and whether it is archived; path did not change"
        )
    );
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["project_update"]);
    assert_eq!(
        recorded.command("project_update")["args"],
        serde_json::json!({"id": "p1", "name": "Chores"})
    );
}
