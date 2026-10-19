use dam_helper::testing;
use std::collections::HashMap;

use dam_remote_todoist::api::{ApiError, TodoistApi, TodoistSync};

#[test]
fn read_all_sends_the_token_and_decodes_the_three_resources() {
    let _guard = testing::guard("read_all_sends_the_token_and_decodes_the_three_resources");
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        vec![testing::Reply::json(
            200,
            serde_json::json!({
                "sync_token": "t1", "full_sync": true,
                "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
                "sections": [{"id": "s1", "name": "Now", "project_id": "p1", "is_deleted": false}],
                "items": [{"id": "i1", "content": "milk", "description": "", "project_id": "p1", "section_id": "s1", "parent_id": null, "priority": 4, "due": {"date": "2026-09-25", "is_recurring": false, "string": "Sep 25"}, "deadline": null, "labels": ["errand"], "checked": false, "is_deleted": false, "child_order": 1}]
            }),
        )],
    );
    let server = testing::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let sync = api.read_all().unwrap();
    assert_eq!(sync.projects[0].name, "Work");
    assert_eq!(sync.sections[0].project_id, "p1");
    assert_eq!(sync.items[0].due.as_ref().unwrap().date, "2026-09-25");
    assert_eq!(sync.items[0].priority, 4);
    let seen = server.seen.lock().unwrap();
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer tok"));
    assert_eq!(seen[0].json()["sync_token"], "*");
    assert_eq!(
        seen[0].json()["resource_types"],
        serde_json::json!(["projects", "sections", "items"])
    );
}

#[test]
fn a_write_sends_its_commands_as_a_form_field_and_reads_each_verdict() {
    let _guard =
        testing::guard("a_write_sends_its_commands_as_a_form_field_and_reads_each_verdict");
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        vec![testing::Reply::json(
            200,
            serde_json::json!({
                "sync_status": {"u-0": "ok", "u-1": {"error": "task not found", "error_code": 15}},
                "temp_id_mapping": {"t-1": "i9"}
            }),
        )],
    );
    let server = testing::serve(routes);
    let written = TodoistApi::new(&server.base, "tok")
        .write(&[
            serde_json::json!({"type": "item_add", "uuid": "u-0", "temp_id": "t-1", "args": {}}),
            serde_json::json!({"type": "item_close", "uuid": "u-1", "args": {}}),
        ])
        .unwrap();
    assert_eq!(written.issued_id_by_temp_id.get("t-1").unwrap(), "i9");
    assert_eq!(written.accepted("u-0"), Ok(()));
    assert_eq!(written.accepted("u-1"), Err("task not found".to_string()));
    assert!(written.accepted("u-2").is_err(), "no verdict is no success");

    let seen = server.seen.lock().unwrap();
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer tok"));
    assert_eq!(seen[0].json()["commands"][0]["type"], "item_add");
    assert_eq!(seen[0].json()["commands"][1]["uuid"], "u-1");
}

#[test]
fn an_http_failure_and_a_transport_failure_never_show_the_token() {
    let _guard = testing::guard("an_http_failure_and_a_transport_failure_never_show_the_token");
    let sentinel = "sentinel-token";
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        vec![testing::Reply::json(
            403,
            serde_json::json!({"error": "forbidden"}),
        )],
    );
    let server = testing::serve(routes);
    let http_err = TodoistApi::new(&server.base, sentinel)
        .read_all()
        .unwrap_err();
    match &http_err {
        ApiError::Http { status, body, .. } => {
            assert_eq!(*status, 403);
            assert!(body.contains("forbidden"));
        }
        other => panic!("{other:?}"),
    }
    assert!(!http_err.to_string().contains(sentinel));

    let transport_err = TodoistApi::new("http://127.0.0.1:1", sentinel)
        .read_all()
        .unwrap_err();
    assert!(matches!(transport_err, ApiError::Transport { .. }));
    assert!(!transport_err.to_string().contains(sentinel));
}

#[test]
fn a_huge_error_body_is_bounded_to_one_short_line_before_it_becomes_a_notice() {
    let _guard =
        testing::guard("a_huge_error_body_is_bounded_to_one_short_line_before_it_becomes_a_notice");
    let mut routes = HashMap::new();
    let huge = format!("{}\n{}", "x".repeat(100 * 1024), "tail");
    routes.insert(
        "POST /sync",
        vec![testing::Reply::json(
            500,
            serde_json::json!({ "error": huge }),
        )],
    );
    let server = testing::serve(routes);
    let err = TodoistApi::new(&server.base, "tok").read_all().unwrap_err();
    let ApiError::Http { body, .. } = &err else {
        panic!("{err:?}")
    };
    assert!(
        body.chars().count() <= dam_remote_todoist::api::MAX_ERROR_BODY_CHARS,
        "{} characters reached dam",
        body.chars().count()
    );
    assert!(!body.contains('\n'), "a stored notice stays one line");
    assert!(body.ends_with('…'), "the cut is marked: {body:?}");
}

#[test]
fn a_rate_limit_is_its_own_outcome_and_carries_the_retry_time() {
    let _guard = testing::guard("a_rate_limit_is_its_own_outcome_and_carries_the_retry_time");
    let calls = std::sync::Arc::new(std::sync::Mutex::new(0u32));
    let counted = std::sync::Arc::clone(&calls);
    let server = testing::serve_with(move |_| {
        let mut n = counted.lock().unwrap();
        *n += 1;
        let reply = testing::Reply::json(
            429,
            serde_json::json!({"error": "Rate limit exceeded", "error_extra": {"retry_after": 30}}),
        );
        if *n == 1 {
            reply.with_header("Retry-After", "30")
        } else {
            reply
        }
    });
    let api = TodoistApi::new(&server.base, "tok");

    let with = api.read_all().unwrap_err();
    assert_eq!(
        with.retry_after(),
        Some(std::time::Duration::from_secs(30)),
        "{with:?}"
    );
    assert!(matches!(with, ApiError::RateLimited { .. }), "{with:?}");
    assert!(with.to_string().contains("30s"), "{with}");

    let without = api.read_all().unwrap_err();
    assert!(
        matches!(without, ApiError::RateLimited { .. }),
        "{without:?}"
    );
    assert_eq!(without.retry_after(), None);
    assert!(
        without.to_string().contains("rate limit"),
        "the reason is named even with no retry time: {without}"
    );
}

#[test]
fn a_redirected_write_is_not_followed_and_the_other_host_sees_nothing() {
    let _guard =
        testing::guard("a_redirected_write_is_not_followed_and_the_other_host_sees_nothing");
    let elsewhere = testing::serve(HashMap::new());
    let target = format!("{}/sync", elsewhere.base);
    let todoist = testing::serve_with(move |_| {
        testing::Reply::json(307, serde_json::json!({})).with_header("Location", target.clone())
    });
    let api = TodoistApi::new(&todoist.base, "tok");
    let err = api
        .write(&[serde_json::json!({"type": "item_add", "uuid": "u-0", "args": {}})])
        .unwrap_err();
    assert!(matches!(err, ApiError::Http { status: 307, .. }), "{err:?}");
    assert_eq!(todoist.seen.lock().unwrap().len(), 1);
    assert!(elsewhere.seen.lock().unwrap().is_empty());
}
