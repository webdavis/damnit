mod loopback;

use std::collections::HashMap;

use dam_remote_todoist::api::{ApiError, TodoistApi};

#[test]
fn sync_all_sends_the_token_and_decodes_the_three_resources() {
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        (
            200,
            serde_json::json!({
                "sync_token": "t1", "full_sync": true,
                "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
                "sections": [{"id": "s1", "name": "Now", "project_id": "p1", "is_deleted": false}],
                "items": [{"id": "i1", "content": "milk", "description": "", "project_id": "p1", "section_id": "s1", "parent_id": null, "priority": 4, "due": {"date": "2026-09-25", "is_recurring": false, "string": "Sep 25"}, "deadline": null, "labels": ["errand"], "checked": false, "is_deleted": false, "child_order": 1}]
            }),
        ),
    );
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let sync = api.sync_all().unwrap();
    assert_eq!(sync.projects[0].name, "Work");
    assert_eq!(sync.sections[0].project_id, "p1");
    assert_eq!(sync.items[0].due.as_ref().unwrap().date, "2026-09-25");
    assert_eq!(sync.items[0].priority, 4);
    let seen = server.seen.lock().unwrap();
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer tok"));
    assert_eq!(seen[0].body["sync_token"], "*");
    assert_eq!(
        seen[0].body["resource_types"],
        serde_json::json!(["projects", "sections", "items"])
    );
}

/// Writes go to the same endpoint as the read, as a form field holding the
/// JSON array of commands, and every verdict comes back keyed by command uuid.
#[test]
fn a_write_sends_its_commands_as_a_form_field_and_reads_each_verdict() {
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        (
            200,
            serde_json::json!({
                "sync_status": {"u-0": "ok", "u-1": {"error": "task not found", "error_code": 15}},
                "temp_id_mapping": {"t-1": "i9"}
            }),
        ),
    );
    let server = loopback::serve(routes);
    let written = TodoistApi::new(&server.base, "tok")
        .sync_commands(&[
            serde_json::json!({"type": "item_add", "uuid": "u-0", "temp_id": "t-1", "args": {}}),
            serde_json::json!({"type": "item_close", "uuid": "u-1", "args": {}}),
        ])
        .unwrap();
    assert_eq!(written.temp_id_mapping.get("t-1").unwrap(), "i9");
    assert_eq!(written.accepted("u-0"), Ok(()));
    assert_eq!(written.accepted("u-1"), Err("task not found".to_string()));
    assert!(written.accepted("u-2").is_err(), "no verdict is no success");

    let seen = server.seen.lock().unwrap();
    assert_eq!(seen[0].authorization.as_deref(), Some("Bearer tok"));
    assert_eq!(seen[0].body["commands"][0]["type"], "item_add");
    assert_eq!(seen[0].body["commands"][1]["uuid"], "u-1");
}

/// Neither a rejected request nor one that never reaches the server may render the token.
#[test]
fn an_http_failure_and_a_transport_failure_never_show_the_token() {
    let sentinel = "sentinel-token";
    let mut routes = HashMap::new();
    routes.insert(
        "POST /tasks",
        (403, serde_json::json!({"error": "forbidden"})),
    );
    let server = loopback::serve(routes);
    let http_err = TodoistApi::new(&server.base, sentinel)
        .post("/tasks", &serde_json::json!({"content": "x"}))
        .unwrap_err();
    match &http_err {
        ApiError::Http { status, body } => {
            assert_eq!(*status, 403);
            assert!(body.contains("forbidden"));
        }
        other => panic!("{other:?}"),
    }
    assert!(!http_err.to_string().contains(sentinel));

    let transport_err = TodoistApi::new("http://127.0.0.1:1", sentinel)
        .post("/tasks", &serde_json::json!({}))
        .unwrap_err();
    assert!(matches!(transport_err, ApiError::Transport(_)));
    assert!(!transport_err.to_string().contains(sentinel));
}

/// A hostile or merely large upstream error page is copied into dam's stored
/// notices and printed by `dam status`, so the helper bounds it first.
#[test]
fn a_huge_error_body_is_bounded_to_one_short_line() {
    let mut routes = HashMap::new();
    let huge = format!("{}\n{}", "x".repeat(100 * 1024), "tail");
    routes.insert("POST /tasks", (500, serde_json::json!({ "error": huge })));
    let server = loopback::serve(routes);
    let err = TodoistApi::new(&server.base, "tok")
        .post("/tasks", &serde_json::json!({}))
        .unwrap_err();
    let ApiError::Http { body, .. } = &err else {
        panic!("{err:?}")
    };
    assert!(
        body.chars().count() <= dam_remote_todoist::api::MAX_ERROR_BODY,
        "{} characters reached dam",
        body.chars().count()
    );
    assert!(!body.contains('\n'), "a stored notice stays one line");
    assert!(body.ends_with('…'), "the cut is marked: {body:?}");
}

/// Todoist answers 429 with a `Retry-After` header when it is rate limiting,
/// and dam has to be able to tell that apart from any other refusal.
#[test]
fn a_rate_limit_is_its_own_outcome_and_carries_the_retry_time() {
    let server = loopback::serve_with(|seen| {
        let reply = loopback::Reply::new(
            429,
            serde_json::json!({"error": "Rate limit exceeded", "error_extra": {"retry_after": 30}}),
        );
        if seen.path == "/with-header" {
            reply.with_header("Retry-After", "30")
        } else {
            reply
        }
    });
    let api = TodoistApi::new(&server.base, "tok");

    let with = api
        .post("/with-header", &serde_json::json!({}))
        .unwrap_err();
    assert_eq!(
        with.retry_after(),
        Some(std::time::Duration::from_secs(30)),
        "{with:?}"
    );
    assert!(matches!(with, ApiError::RateLimited { .. }), "{with:?}");
    assert!(with.to_string().contains("30s"), "{with}");

    let without = api.post("/no-header", &serde_json::json!({})).unwrap_err();
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
