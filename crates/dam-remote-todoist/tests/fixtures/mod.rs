// The builders and the fake service both push test binaries share.
#![allow(dead_code)]

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use dam_protocol::{Mutation, WireObject, WireTask};

use crate::loopback;

pub fn sync_body() -> serde_json::Value {
    serde_json::json!({
        "sync_token": "t", "full_sync": true,
        "projects": [{"id": "p1", "name": "Work", "parent_id": null, "is_deleted": false, "is_archived": false, "inbox_project": false}],
        "sections": [],
        "items": [{"id": "i1", "content": "milk", "project_id": "p1", "section_id": null, "priority": 1, "checked": false, "is_deleted": false}]
    })
}

pub fn sync_body_with_two_projects() -> serde_json::Value {
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

/// A key of the shape dam mints: a UUID whose last character is left free for
/// the helper to number a mutation's commands in.
pub fn key(n: u8) -> String {
    format!("0102030a-0b0c-4d0e-8f10-1112131415{n:x}0")
}

pub fn mutation(op: &str, n: u8, remote_id: Option<&str>, object: Option<WireObject>) -> Mutation {
    Mutation {
        op: op.into(),
        oid: format!("{n:x}").repeat(40),
        idempotency_key: key(n),
        remote_id: remote_id.map(str::to_string),
        object,
        fields: vec![],
    }
}

pub fn with_fields(mut m: Mutation, fields: &[&str]) -> Mutation {
    m.fields = fields.iter().map(|f| f.to_string()).collect();
    m
}

pub fn task(oid: &str, path: &str, subject: &str, done: bool) -> WireObject {
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

/// An object whose `due` and `recurrence` are both gone, so an update naming
/// only "due" sends the one field that clears the date.
pub fn task_with_dates_cleared(oid: &str) -> WireObject {
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

/// What the fake service did, as opposed to what it was asked to do.
#[derive(Default)]
pub struct Recorded {
    /// One entry per command actually executed, in order.
    pub executed: Vec<serde_json::Value>,
    /// Every command uuid ever seen, executed or deduplicated.
    pub uuids: HashSet<String>,
    pub issued: u32,
}

impl Recorded {
    pub fn types(&self) -> Vec<&str> {
        self.executed
            .iter()
            .filter_map(|c| c["type"].as_str())
            .collect()
    }

    pub fn command(&self, kind: &str) -> serde_json::Value {
        self.executed
            .iter()
            .find(|c| c["type"] == kind)
            .unwrap_or_else(|| panic!("no {kind} in {:?}", self.types()))
            .clone()
    }
}

/// A stand-in for Todoist's sync endpoint that remembers command uuids the way
/// the documented one does: "Todoist will not execute a command that has same
/// UUID as a previously executed command." `refuse` names a command type it
/// answers with an error object instead of executing.
pub fn todoist(
    tree: serde_json::Value,
    refuse: Option<&'static str>,
) -> (loopback::Loopback, Arc<Mutex<Recorded>>) {
    let state = Arc::new(Mutex::new(Recorded::default()));
    let inner = Arc::clone(&state);
    let server = loopback::serve_with(move |seen| {
        let Some(commands) = seen.body.get("commands").and_then(|c| c.as_array()) else {
            return loopback::Reply::new(200, tree.clone());
        };
        let mut recorded = inner.lock().unwrap();
        let mut status = serde_json::Map::new();
        let mut mapping = serde_json::Map::new();
        for c in commands {
            let uuid = c["uuid"].as_str().unwrap_or_default().to_string();
            let kind = c["type"].as_str().unwrap_or_default();
            if !recorded.uuids.insert(uuid.clone()) {
                status.insert(uuid, "ok".into());
                continue;
            }
            if refuse == Some(kind) {
                status.insert(
                    uuid,
                    serde_json::json!({"error_code": 15, "error": "refused by the test"}),
                );
                continue;
            }
            recorded.executed.push(c.clone());
            status.insert(uuid, "ok".into());
            if let Some(temp) = c["temp_id"].as_str() {
                recorded.issued += 1;
                mapping.insert(temp.to_string(), format!("new{}", recorded.issued).into());
            }
        }
        loopback::Reply::new(
            200,
            serde_json::json!({"sync_status": status, "temp_id_mapping": mapping}),
        )
    });
    (server, state)
}
