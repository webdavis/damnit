mod loopback;

use std::collections::HashMap;

use dam_protocol::{Mutation, WireObject, WireTask};
use dam_remote_todoist::api::TodoistApi;
use dam_remote_todoist::push::push;

fn sync_body() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

fn sync_body_with_two_projects() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [
            {"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false},
            {"id": "p2", "name": "Home", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}
        ],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

fn task(oid: &str, path: &str, subject: &str, done: bool) -> WireObject {
    WireObject {
        oid: oid.into(),
        remote_id: None,
        kind: "task".into(),
        subject: subject.into(),
        body: "b".into(),
        path: path.into(),
        labels: vec!["errand".into()],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done,
            priority: 1,
            due: Some("2026-09-25".into()),
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

/// An object whose `due` is gone and whose `recurrence` is gone, with only "due" in the
/// changed-fields list, so the update sends nothing but the field that clears the date.
fn task_with_due_cleared(oid: &str) -> WireObject {
    WireObject {
        oid: oid.into(),
        remote_id: None,
        kind: "task".into(),
        subject: "unused".into(),
        body: "b".into(),
        path: "Work/".into(),
        labels: vec!["errand".into()],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done: false,
            priority: 1,
            due: None,
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

/// Same shape, but the cleared field is the deadline, not the due date.
fn task_with_deadline_cleared(oid: &str) -> WireObject {
    WireObject {
        oid: oid.into(),
        remote_id: None,
        kind: "task".into(),
        subject: "unused".into(),
        body: "b".into(),
        path: "Work/".into(),
        labels: vec!["errand".into()],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(WireTask {
            done: false,
            priority: 1,
            due: None,
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

#[test]
fn creates_updates_and_deletes_become_the_right_requests() {
    let mut routes = HashMap::new();
    routes.insert("POST /sync", (200, sync_body()));
    routes.insert("POST /tasks", (200, serde_json::json!({"id": "i2"})));
    routes.insert("POST /tasks/i1", (200, serde_json::json!({"id": "i1"})));
    routes.insert("POST /tasks/i1/close", (204, serde_json::Value::Null));
    routes.insert("DELETE /projects/p1", (204, serde_json::Value::Null));
    routes.insert("POST /tasks/i3", (200, serde_json::json!({"id": "i3"})));
    routes.insert("POST /tasks/i4", (200, serde_json::json!({"id": "i4"})));
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![
        Mutation {
            op: "create".into(),
            oid: "1".repeat(40),
            remote_id: None,
            object: Some(task(&"1".repeat(40), "Work/", "eggs", false)),
            fields: vec![],
        },
        Mutation {
            op: "update".into(),
            oid: "2".repeat(40),
            remote_id: Some("i:i1".into()),
            object: Some(task(&"2".repeat(40), "Work/", "milk", true)),
            fields: vec!["done".into(), "subject".into()],
        },
        Mutation {
            op: "delete".into(),
            oid: "3".repeat(40),
            remote_id: Some("p:p1".into()),
            object: None,
            fields: vec![],
        },
        Mutation {
            op: "create".into(),
            oid: "4".repeat(40),
            remote_id: None,
            object: Some(task(&"4".repeat(40), "Nope/", "lost", false)),
            fields: vec![],
        },
        Mutation {
            op: "update".into(),
            oid: "5".repeat(40),
            remote_id: Some("i:i3".into()),
            object: Some(task_with_due_cleared(&"5".repeat(40))),
            fields: vec!["due".into()],
        },
        Mutation {
            op: "update".into(),
            oid: "6".repeat(40),
            remote_id: Some("i:i4".into()),
            object: Some(task_with_deadline_cleared(&"6".repeat(40))),
            fields: vec!["deadline".into()],
        },
    ];
    let response = push(&api, mutations).unwrap();
    assert_eq!(response.results.len(), 6);
    assert!(response.results[0].ok);
    assert_eq!(response.results[0].remote_id.as_deref(), Some("i:i2"));
    assert!(response.results[1].ok);
    assert!(response.results[2].ok);
    assert!(!response.results[3].ok);
    assert!(response.results[3].why.as_deref().unwrap().contains("Nope"));
    assert!(response.results[4].ok);
    assert!(response.results[5].ok);
    let seen = server.seen.lock().unwrap();
    let create = seen
        .iter()
        .find(|s| s.path == "/tasks" && s.method == "POST")
        .unwrap();
    assert_eq!(create.body["content"], "eggs");
    assert_eq!(create.body["project_id"], "p1");
    assert_eq!(create.body["priority"], 4, "dam p1 is API 4");
    assert_eq!(create.body["due_date"], "2026-09-25");
    assert_eq!(create.body["labels"], serde_json::json!(["errand"]));
    assert!(
        create.body.get("deadline_date").is_none(),
        "a create with no deadline omits the field rather than sending null"
    );
    let update = seen
        .iter()
        .find(|s| s.path == "/tasks/i1" && s.method == "POST")
        .unwrap();
    assert_eq!(update.body["content"], "milk");
    assert!(
        update.body.get("priority").is_none(),
        "only changed fields are sent"
    );
    assert!(seen.iter().any(|s| s.path == "/tasks/i1/close"));
    assert!(
        seen.iter()
            .any(|s| s.path == "/projects/p1" && s.method == "DELETE")
    );
    let cleared = seen
        .iter()
        .find(|s| s.path == "/tasks/i3" && s.method == "POST")
        .unwrap();
    assert_eq!(cleared.body, serde_json::json!({"due_string": "no date"}));
    let deadline_cleared = seen
        .iter()
        .find(|s| s.path == "/tasks/i4" && s.method == "POST")
        .unwrap();
    assert_eq!(
        deadline_cleared.body,
        serde_json::json!({"deadline_date": null})
    );
}

#[test]
fn a_path_change_moves_the_task_through_the_move_endpoint() {
    let mut routes = HashMap::new();
    routes.insert("POST /sync", (200, sync_body_with_two_projects()));
    routes.insert(
        "POST /tasks/i1/move",
        (200, serde_json::json!({"id": "i1"})),
    );
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![Mutation {
        op: "update".into(),
        oid: "9".repeat(40),
        remote_id: Some("i:i1".into()),
        object: Some(task(&"9".repeat(40), "Home/", "milk", false)),
        fields: vec!["path".into()],
    }];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let seen = server.seen.lock().unwrap();
    let mv = seen
        .iter()
        .find(|s| s.path == "/tasks/i1/move" && s.method == "POST")
        .unwrap();
    assert_eq!(mv.body, serde_json::json!({"project_id": "p2"}));
    assert!(
        !seen
            .iter()
            .any(|s| s.path == "/tasks/i1" && s.method == "POST"),
        "a path-only change sends no plain update, only the move"
    );
}

#[test]
fn a_path_and_subject_change_moves_first_then_updates_the_content() {
    let mut routes = HashMap::new();
    routes.insert("POST /sync", (200, sync_body_with_two_projects()));
    routes.insert(
        "POST /tasks/i1/move",
        (200, serde_json::json!({"id": "i1"})),
    );
    routes.insert("POST /tasks/i1", (200, serde_json::json!({"id": "i1"})));
    let server = loopback::serve(routes);
    let api = TodoistApi::new(&server.base, "tok");
    let mutations = vec![Mutation {
        op: "update".into(),
        oid: "8".repeat(40),
        remote_id: Some("i:i1".into()),
        object: Some(task(&"8".repeat(40), "Home/", "oat", false)),
        fields: vec!["path".into(), "subject".into()],
    }];
    let response = push(&api, mutations).unwrap();
    assert!(response.results[0].ok, "{:?}", response.results[0].why);
    let seen = server.seen.lock().unwrap();
    let task_calls: Vec<&loopback::Seen> = seen
        .iter()
        .filter(|s| s.path.starts_with("/tasks/i1"))
        .collect();
    assert_eq!(
        task_calls.len(),
        2,
        "one move and one update, nothing else: {task_calls:?}"
    );
    assert_eq!(task_calls[0].method, "POST");
    assert_eq!(task_calls[0].path, "/tasks/i1/move");
    assert_eq!(
        task_calls[0].body,
        serde_json::json!({"project_id": "p2"}),
        "move takes exactly one of project_id, section_id or parent_id"
    );
    assert_eq!(task_calls[1].method, "POST");
    assert_eq!(task_calls[1].path, "/tasks/i1");
    assert_eq!(task_calls[1].body["content"], "oat");
}
