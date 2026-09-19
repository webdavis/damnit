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

#[test]
fn delete_hits_the_path_with_the_method() {
    let mut routes = HashMap::new();
    routes.insert("DELETE /tasks/i1", (204, serde_json::Value::Null));
    let server = loopback::serve(routes);
    TodoistApi::new(&server.base, "tok")
        .delete("/tasks/i1")
        .unwrap();
    assert_eq!(server.seen.lock().unwrap()[0].method, "DELETE");
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
