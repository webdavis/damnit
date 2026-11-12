use dam_helper::testing;
use std::collections::HashMap;

use dam_remote_todoist::api::{TodoistApi, TodoistSync};

#[test]
fn read_changes_sends_the_stored_token_and_reads_whether_todoist_answered_in_full() {
    let _guard = testing::guard(
        "read_changes_sends_the_stored_token_and_reads_whether_todoist_answered_in_full",
    );
    let mut routes = HashMap::new();
    routes.insert(
        "POST /sync",
        vec![testing::Reply::json(
            200,
            serde_json::json!({
                "sync_token": "t2", "full_sync": false,
                "items": [{"id": "i1", "content": "milk", "project_id": "p1", "checked": true, "completed_at": "2026-11-12T09:15:22Z", "is_deleted": false}]
            }),
        )],
    );
    let server = testing::serve(routes);
    let changes = TodoistApi::new(&server.base, "tok")
        .read_changes("t1")
        .unwrap();
    assert_eq!(changes.sync_token.as_deref(), Some("t2"));
    assert!(!changes.full_sync);
    assert!(changes.items[0].checked);
    let seen = server.seen.lock().unwrap();
    assert_eq!(seen[0].json()["sync_token"], "t1");
}
