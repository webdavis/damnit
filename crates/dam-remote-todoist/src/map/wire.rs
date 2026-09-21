//! One Todoist project, section or item as the wire object dam reads.

use dam_protocol::{WireObject, WireTask};

use super::{Tree, dam_priority, remote_id};
use crate::api::{Item, Project, Section};

fn wire(remote_id: String, subject: &str, body: &str, path: String, task: WireTask) -> WireObject {
    WireObject {
        oid: String::new(),
        remote_id: Some(remote_id),
        kind: "task".into(),
        subject: subject.to_string(),
        body: body.to_string(),
        path,
        labels: vec![],
        depends: vec![],
        reminders: vec![],
        recurrence: None,
        task: Some(task),
        event: None,
    }
}

fn plain(done: bool) -> WireTask {
    WireTask {
        done,
        completed_at: None,
        priority: 4,
        due: None,
        deadline: None,
        event: None,
    }
}

pub fn project_to_wire(t: &Tree, p: &Project) -> WireObject {
    let path = p
        .parent_id
        .as_deref()
        .and_then(|pid| t.project_path(pid))
        .unwrap_or_default();
    wire(
        remote_id('p', &p.id),
        &p.name,
        "",
        path,
        plain(p.is_archived),
    )
}

pub fn section_to_wire(t: &Tree, s: &Section) -> WireObject {
    let path = t.project_path(&s.project_id).unwrap_or_default();
    wire(remote_id('s', &s.id), &s.name, "", path, plain(false))
}

pub fn item_to_wire(t: &Tree, i: &Item) -> Option<WireObject> {
    let path = t.item_path(i)?;
    let task = WireTask {
        done: i.checked,
        completed_at: None,
        priority: dam_priority(i.priority),
        due: i.due.as_ref().map(|d| d.date.clone()),
        deadline: i.deadline.as_ref().map(|d| d.date.clone()),
        event: None,
    };
    let mut w = wire(
        remote_id('i', &i.id),
        &i.content,
        &i.description,
        path,
        task,
    );
    w.labels = i.labels.clone();
    w.recurrence = i
        .due
        .as_ref()
        .filter(|d| d.is_recurring)
        .map(|d| d.string.clone());
    Some(w)
}
