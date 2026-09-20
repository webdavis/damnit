mod support;

mod fixtures;
mod loopback;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dam_remote_todoist::api::{ApiError, TodoistApi};
use dam_remote_todoist::push::push;
use fixtures::{mutation, sync_body, task, todoist, with_fields};

/// Finding 1.1: the request left, the answer never arrived, dam resent. The
/// resent mutation carries the key it carried the first time, so the service
/// recognises it and creates nothing twice.
#[test]
fn a_resent_mutation_repeats_its_uuids_and_creates_nothing_twice() {
    let _guard = support::guard("a_resent_mutation_repeats_its_uuids_and_creates_nothing_twice");
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let first = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let again = first.clone();

    let one = push(&api, first).unwrap();
    assert!(one.results[0].ok);
    let two = push(&api, again).unwrap();

    let recorded = recorded.lock().unwrap();
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
        recorded.uuids.len(),
        1,
        "the resend repeated the uuid rather than minting one"
    );
    assert!(
        !two.results[0].ok,
        "a resend dam cannot learn an id from is not reported as pushed"
    );
}

/// Every command of one mutation gets its own uuid, or the second is dropped
/// as a duplicate of the first.
#[test]
fn the_commands_of_one_mutation_have_distinct_uuids() {
    let _guard = support::guard("the_commands_of_one_mutation_have_distinct_uuids");
    let (server, recorded) = todoist(sync_body(), None);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", true)),
    )];
    assert!(push(&api, mutations).unwrap().results[0].ok);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_add", "item_close"]);
    assert_eq!(recorded.uuids.len(), 2);
}

/// One command's error is that mutation's failure and nobody else's.
#[test]
fn a_refused_command_fails_only_its_own_mutation() {
    let _guard = support::guard("a_refused_command_fails_only_its_own_mutation");
    let (server, recorded) = todoist(sync_body(), Some("item_update"));
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
                Some(task(&"2".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        mutation("delete", 3, Some("p:p1"), None),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok);
    assert!(!response.results[1].ok);
    assert_eq!(
        response.results[1].why.as_deref(),
        Some("refused by the test")
    );
    assert!(response.results[2].ok);
    let recorded = recorded.lock().unwrap();
    assert_eq!(recorded.types(), vec!["item_add", "project_delete"]);
}

/// A failure reading the tree is not a per-mutation result: no mutation was
/// attempted, so there is nothing to report against one.
#[test]
fn a_failure_reading_the_tree_stops_the_push_before_any_mutation() {
    let _guard = support::guard("a_failure_reading_the_tree_stops_the_push_before_any_mutation");
    let mut routes = HashMap::new();
    routes.insert("POST /sync", (500, serde_json::json!({"error": "boom"})));
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![mutation(
        "create",
        1,
        None,
        Some(task(&"1".repeat(40), "Work/", "eggs", false)),
    )];
    let err = push(&api, mutations).unwrap_err();
    assert!(
        matches!(err, ApiError::Http { status: 500, .. }),
        "an upstream status reaches the caller as itself: {err:?}"
    );
    assert_eq!(
        server.seen.lock().unwrap().len(),
        1,
        "nothing was attempted after the read failed"
    );
}

/// The other half of the same decision. Once mutations are being applied, a
/// failure belongs to the mutation it happened on: the ones Todoist already
/// took are reported as taken, and dam marks exactly those pushed. That is
/// only safe because a resend repeats its key, so anything dam sends again is
/// deduplicated rather than duplicated.
#[test]
fn a_failure_part_way_through_belongs_to_its_own_mutation() {
    let _guard = support::guard("a_failure_part_way_through_belongs_to_its_own_mutation");
    let writes = Arc::new(Mutex::new(0u32));
    let counted = Arc::clone(&writes);
    let server = loopback::serve_with(move |seen| {
        if seen.body.get("commands").is_none() {
            return loopback::Reply::new(200, sync_body());
        }
        let mut n = counted.lock().unwrap();
        *n += 1;
        if *n == 2 {
            return loopback::Reply::new(500, serde_json::json!({"error": "upstream is down"}));
        }
        let uuids: Vec<String> = seen.body["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["uuid"].as_str().unwrap_or_default().to_string())
            .collect();
        let status: serde_json::Map<String, serde_json::Value> = uuids
            .into_iter()
            .map(|u| (u, serde_json::Value::from("ok")))
            .collect();
        loopback::Reply::new(
            200,
            serde_json::json!({"sync_status": status, "temp_id_mapping": {}}),
        )
    });
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        with_fields(
            mutation(
                "update",
                1,
                Some("i:i1"),
                Some(task(&"1".repeat(40), "Work/", "milk", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                2,
                Some("i:i1"),
                Some(task(&"2".repeat(40), "Work/", "oat", false)),
            ),
            &["subject"],
        ),
        with_fields(
            mutation(
                "update",
                3,
                Some("i:i1"),
                Some(task(&"3".repeat(40), "Work/", "soy", false)),
            ),
            &["subject"],
        ),
    ];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok);
    assert!(!response.results[1].ok);
    assert!(response.results[1].why.as_deref().unwrap().contains("500"));
    assert!(
        response.results[2].ok,
        "the batch goes on after one mutation's failure"
    );
}
