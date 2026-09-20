mod support;

mod loopback;

use std::collections::HashMap;

use std::time::Duration;

use dam_remote_todoist::api::{ApiError, TodoistApi};
use dam_remote_todoist::pull::pull;

#[test]
fn pull_returns_every_live_object_and_names_the_removed_ones() {
    let _guard = support::guard("pull_returns_every_live_object_and_names_the_removed_ones");
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        (
            200,
            serde_json::json!({
                "sync_token": "t1", "full_sync": true,
                "projects": [
                    {"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false},
                    {"id": "p9", "name": "Gone", "parent_id": null, "is_deleted": true, "is_archived": false, "inbox_project": false}
                ],
                "sections": [{"id": "s1", "name": "Now", "project_id": "p1", "is_deleted": false}],
                "items": [
                    {"id": "i1", "content": "milk", "project_id": "p1", "section_id": "s1", "priority": 4, "checked": false, "is_deleted": false},
                    {"id": "i9", "content": "old", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": true}
                ]
            }),
        ),
    );
    let server = loopback::serve(routes);
    let response = pull(&TodoistApi::new(&server.base, "tok")).unwrap();
    let ids: Vec<&str> = response
        .objects
        .iter()
        .filter_map(|o| o.remote_id.as_deref())
        .collect();
    assert_eq!(ids, vec!["p:p1", "s:s1", "i:i1"]);
    assert_eq!(
        response.removed,
        vec!["p:p9".to_string(), "i:i9".to_string()]
    );
    assert!(response.sync.is_none());
    assert_eq!(server.seen.lock().unwrap()[0].path, "/sync");
}

/// The opening full sync is the first request a pull makes, so it is where a
/// rate limit is most likely met. The refusal keeps its class and the retry
/// time Todoist named, rather than reading like any other failed request.
#[test]
fn a_rate_limit_on_the_opening_sync_keeps_its_class_and_retry_time() {
    let _guard = support::guard("a_rate_limit_on_the_opening_sync_keeps_its_class_and_retry_time");
    let server = loopback::serve_with(|_| {
        loopback::Reply::new(429, serde_json::json!({"error": "Rate limit exceeded"}))
            .with_header("Retry-After", "42")
    });
    let err = pull(&TodoistApi::new(&server.base, "tok")).unwrap_err();
    assert!(matches!(err, ApiError::RateLimited { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(Duration::from_secs(42)));
    assert!(err.to_string().contains("42s"), "{err}");
}
