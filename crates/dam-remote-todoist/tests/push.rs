mod support;

mod fixtures;
mod loopback;

use dam_remote_todoist::api::{ApiError, TodoistApi};
use dam_remote_todoist::push::push;
use fixtures::{
    mutation, sync_body, sync_body_with_two_projects, task, task_with_dates_cleared, todoist,
    with_fields,
};

#[test]
fn creates_updates_and_deletes_become_the_right_commands() {
    let _guard = support::guard("creates_updates_and_deletes_become_the_right_commands");
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
    assert!(response.results[0].ok);
    assert_eq!(response.results[0].remote_id.as_deref(), Some("i:new1"));
    assert!(response.results[1].ok, "{:?}", response.results[1].why);
    assert!(response.results[2].ok);
    assert!(!response.results[3].ok);
    assert!(response.results[3].why.as_deref().unwrap().contains("Nope"));
    assert!(response.results[4].ok);
    assert!(response.results[5].ok);

    let recorded = recorded.lock().unwrap();
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
fn a_path_change_moves_the_task_through_the_move_command() {
    let _guard = support::guard("a_path_change_moves_the_task_through_the_move_command");
    let (server, recorded) = todoist(sync_body_with_two_projects(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_move"]);
    assert_eq!(
        recorded.command("item_move")["args"],
        serde_json::json!({"id": "i1", "project_id": "p2"})
    );
}

#[test]
fn a_path_and_subject_change_moves_first_then_updates_the_content() {
    let _guard = support::guard("a_path_and_subject_change_moves_first_then_updates_the_content");
    let (server, recorded) = todoist(sync_body_with_two_projects(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let recorded = recorded.lock().unwrap();
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

/// The stored `remote_id` is Todoist's own string on the way back out. One
/// carrying path syntax must never reach a command.
#[test]
fn a_hostile_remote_id_is_refused_and_never_reaches_a_command() {
    let _guard = support::guard("a_hostile_remote_id_is_refused_and_never_reaches_a_command");
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
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
        assert!(!result.ok, "{result:?}");
        assert!(
            result
                .why
                .as_deref()
                .unwrap()
                .contains("unusable Todoist id"),
            "{result:?}"
        );
    }
    assert!(recorded.lock().unwrap().executed.is_empty());
}

/// A rate limit is not one mutation's problem. Reporting it per mutation would
/// leave dam marking the rest of the batch pushed while Todoist took none of
/// it, so the push stops and says how long to wait.
#[test]
fn a_rate_limit_stops_the_push_and_names_the_retry_time() {
    let _guard = support::guard("a_rate_limit_stops_the_push_and_names_the_retry_time");
    let server = loopback::serve_with(|seen| {
        if seen.body.get("commands").is_none() {
            return loopback::Reply::new(200, sync_body());
        }
        loopback::Reply::new(429, serde_json::json!({"error": "Rate limit exceeded"}))
            .with_header("Retry-After", "42")
    });
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let err = push(&api, mutations).unwrap_err();
    assert!(matches!(err, ApiError::RateLimited { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(42)));
    assert!(err.to_string().contains("42s"), "{err}");
}
