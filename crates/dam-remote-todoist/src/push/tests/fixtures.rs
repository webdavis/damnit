use dam_protocol::{Mutation, WireObject, WireTask};

pub(crate) fn sync_body() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

pub(crate) fn sync_body_with_two_projects() -> serde_json::Value {
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

pub(crate) fn key_with_its_last_character_free(n: u8) -> String {
    format!("0102030a-0b0c-4d0e-8f10-1112131415{n:x}0")
}

pub(crate) fn mutation(
    op: &str,
    n: u8,
    remote_id: Option<&str>,
    object: Option<WireObject>,
) -> Mutation {
    Mutation {
        op: op.into(),
        oid: format!("{n:x}").repeat(40),
        idempotency_key: key_with_its_last_character_free(n),
        remote_id: remote_id.map(str::to_string),
        object,
        fields: vec![],
    }
}

pub(crate) fn with_fields(mut m: Mutation, fields: &[&str]) -> Mutation {
    m.fields = fields.iter().map(|f| f.to_string()).collect();
    m
}

pub(crate) fn task(oid: &str, path: &str, subject: &str, done: bool) -> WireObject {
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
            completed_at: None,
            priority: 1,
            due: Some("2026-09-25".into()),
            deadline: None,
            event: None,
        }),
        event: None,
    }
}

pub(crate) fn task_with_dates_cleared(oid: &str) -> WireObject {
    let mut t = task(oid, "Work/", "unused", false);
    t.task = Some(WireTask {
        done: false,
        completed_at: None,
        priority: 1,
        due: None,
        deadline: None,
        event: None,
    });
    t
}
