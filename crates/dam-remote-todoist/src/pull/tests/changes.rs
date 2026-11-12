use serde_json::{Value, json};

use super::super::pull;
use crate::api::fake::FakeTodoist;
use dam_protocol::{PullResponse, WireObject};

fn listing(items: Value) -> Value {
    json!({
        "sync_token": "t1", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work"}],
        "sections": [],
        "items": items
    })
}

fn changes(items: Value) -> Value {
    json!({"sync_token": "t2", "full_sync": false, "items": items})
}

fn open_item(id: &str, content: &str) -> Value {
    json!({"id": id, "content": content, "project_id": "p1", "checked": false})
}

fn object<'a>(response: &'a PullResponse, remote_id: &str) -> &'a WireObject {
    let mut found = response
        .objects
        .iter()
        .filter(|o| o.remote_id.as_deref() == Some(remote_id));
    let first = found.next().unwrap_or_else(|| panic!("no {remote_id}"));
    assert!(found.next().is_none(), "{remote_id} came twice");
    first
}

#[test]
fn a_task_completed_in_the_app_since_the_token_comes_in_done_with_its_time() {
    let todoist = FakeTodoist::new(listing(json!([open_item("i2", "eggs")]))).with_changes_since(
        "t0",
        changes(
            json!([{"id": "i1", "content": "milk", "project_id": "p1", "checked": true,
            "completed_at": "2026-11-12T09:15:22Z"}]),
        ),
    );
    let response = pull(&todoist, Some("t0")).unwrap();
    let task = object(&response, "i:i1").task.clone().unwrap();
    assert!(task.done);
    assert_eq!(task.completed_at.as_deref(), Some("2026-11-12T09:15:22Z"));
    assert!(!object(&response, "i:i2").task.as_ref().unwrap().done);
    assert_eq!(response.sync.as_deref(), Some("t2"));
    let recorded = todoist.recorded.borrow();
    assert_eq!(recorded.changes_asked_since, vec!["t0".to_string()]);
    assert_eq!(recorded.reads, 1);
}

#[test]
fn a_task_section_or_project_deleted_in_the_app_is_named_removed() {
    let todoist = FakeTodoist::new(listing(json!([]))).with_changes_since(
        "t0",
        json!({"sync_token": "t2", "full_sync": false,
            "projects": [{"id": "p9", "name": "Old", "is_deleted": true}],
            "sections": [{"id": "s9", "name": "Later", "project_id": "p1", "is_deleted": true}],
            "items": [{"id": "i1", "content": "milk", "project_id": "p1", "is_deleted": true}]}),
    );
    let response = pull(&todoist, Some("t0")).unwrap();
    assert_eq!(response.removed, vec!["p:p9", "s:s9", "i:i1"]);
    assert!(response.objects.iter().all(|o| o.subject == "Work"));
}

#[test]
fn a_task_reopened_in_the_app_comes_in_open_once() {
    let todoist = FakeTodoist::new(listing(json!([open_item("i1", "milk")])))
        .with_changes_since("t0", changes(json!([open_item("i1", "milk")])));
    let response = pull(&todoist, Some("t0")).unwrap();
    let task = object(&response, "i:i1").task.clone().unwrap();
    assert!(!task.done);
    assert_eq!(task.completed_at, None);
}

#[test]
fn a_recurring_task_completed_in_the_app_stays_open_on_its_next_date() {
    let next = json!([{"id": "i1", "content": "water", "project_id": "p1", "checked": false,
        "due": {"date": "2026-11-13", "is_recurring": true, "string": "every day"}}]);
    let todoist = FakeTodoist::new(listing(next.clone())).with_changes_since("t0", changes(next));
    let response = pull(&todoist, Some("t0")).unwrap();
    let water = object(&response, "i:i1");
    let task = water.task.clone().unwrap();
    assert!(!task.done);
    assert_eq!(task.due.as_deref(), Some("2026-11-13"));
    assert_eq!(water.recurrence.as_deref(), Some("every day"));
}

#[test]
fn a_completed_task_under_a_completed_parent_is_still_placed() {
    let todoist = FakeTodoist::new(listing(json!([]))).with_changes_since(
        "t0",
        changes(json!([
            {"id": "i1", "content": "trip", "project_id": "p1", "checked": true},
            {"id": "i2", "content": "pack", "project_id": "p1", "parent_id": "i1", "checked": true}
        ])),
    );
    let response = pull(&todoist, Some("t0")).unwrap();
    assert_eq!(object(&response, "i:i2").path, "Work/trip/");
    assert!(response.rejected.is_empty(), "{:?}", response.rejected);
}

#[test]
fn a_token_todoist_answers_with_a_full_sync_reads_as_a_pull_with_no_token() {
    let todoist = FakeTodoist::new(listing(json!([open_item("i1", "stale")]))).with_changes_since(
        "lost",
        json!({"sync_token": "t3", "full_sync": true,
            "projects": [{"id": "p1", "name": "Work"}],
            "items": [open_item("i5", "fresh")]}),
    );
    let response = pull(&todoist, Some("lost")).unwrap();
    let ids: Vec<&str> = response
        .objects
        .iter()
        .filter_map(|o| o.remote_id.as_deref())
        .collect();
    assert_eq!(ids, vec!["p:p1", "i:i5"]);
    assert_eq!(response.sync.as_deref(), Some("t3"));
}

#[test]
fn a_change_the_listing_already_holds_is_answered_once() {
    let todoist = FakeTodoist::new(listing(json!([open_item("i1", "milk")])))
        .with_changes_since("t0", changes(json!([open_item("i1", "milk")])));
    let first = pull(&todoist, Some("t0")).unwrap();
    let again = pull(&todoist, Some("t0")).unwrap();
    assert_eq!(first, again);
    assert_eq!(object(&again, "i:i1").subject, "milk");
}
