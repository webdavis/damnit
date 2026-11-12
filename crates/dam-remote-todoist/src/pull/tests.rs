use super::*;
use crate::api::fake::{FakeTodoist, rate_limited};

#[test]
fn pull_returns_every_live_object_and_names_the_removed_ones() {
    let todoist = FakeTodoist::new(serde_json::json!({
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
    }));
    let response = pull(&todoist, None).unwrap();
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
    assert_eq!(response.sync.as_deref(), Some("t1"));
    assert!(todoist.recorded.borrow().changes_asked_since.is_empty());
    assert_eq!(todoist.recorded.borrow().reads, 1);
}

#[test]
fn an_item_whose_parent_chain_loops_is_reported_and_the_rest_land() {
    let todoist = FakeTodoist::new(serde_json::json!({
        "sync_token": "t1", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [],
        "items": [
            {"id": "i1", "content": "a", "project_id": "p1", "parent_id": "i2", "priority": 1, "checked": false, "is_deleted": false},
            {"id": "i2", "content": "b", "project_id": "p1", "parent_id": "i1", "priority": 1, "checked": false, "is_deleted": false},
            {"id": "i3", "content": "milk", "project_id": "p1", "priority": 1, "checked": false, "is_deleted": false}
        ]
    }));
    let response = pull(&todoist, None).unwrap();
    let ids: Vec<&str> = response
        .objects
        .iter()
        .filter_map(|o| o.remote_id.as_deref())
        .collect();
    assert_eq!(ids, vec!["p:p1", "i:i3"]);
    let rejected: Vec<(&str, &str)> = response
        .rejected
        .iter()
        .map(|r| (r.remote_id.as_str(), r.why.as_str()))
        .collect();
    let why = "its parent chain loops or its project or section is missing";
    assert_eq!(rejected, vec![("i:i1", why), ("i:i2", why)]);
}

#[test]
fn a_rate_limit_on_the_opening_sync_keeps_its_class_and_retry_time() {
    let todoist = FakeTodoist::default().failing_read(rate_limited(42));
    let err = pull(&todoist, None).unwrap_err();
    assert!(matches!(err, ApiError::RateLimited { .. }), "{err:?}");
    assert_eq!(err.retry_after(), Some(std::time::Duration::from_secs(42)));
    assert!(err.to_string().contains("42s"), "{err}");
}

#[test]
fn a_task_completed_in_the_app_carries_its_completion_time() {
    let todoist = FakeTodoist::new(serde_json::json!({
        "projects": [{"id": "p1", "name": "Work"}],
        "items": [
            {"id": "i1", "content": "milk", "project_id": "p1", "checked": true, "completed_at": "2026-11-12T09:15:22.123456Z"},
            {"id": "i2", "content": "eggs", "project_id": "p1", "checked": false, "completed_at": null}
        ]
    }));
    let response = pull(&todoist, None).unwrap();
    let completed: Vec<Option<&str>> = response.objects[1..]
        .iter()
        .map(|o| o.task.as_ref().unwrap().completed_at.as_deref())
        .collect();
    assert_eq!(completed, vec![Some("2026-11-12T09:15:22.123456Z"), None]);
}

mod changes;
