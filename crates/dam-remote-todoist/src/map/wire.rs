use dam_protocol::{WireObject, WireTask};

use super::{RemoteId, Tree, dam_due, dam_priority};
use crate::api::{Item, Project, Section};
use crate::rule_words::from_todoist;

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
        parent: None,
        container_remote_id: None,
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
        estimate: None,
        focus: None,
    }
}

pub fn project_to_wire(t: &Tree, p: &Project) -> WireObject {
    let path = p
        .parent_id
        .as_deref()
        .and_then(|pid| t.project_path(pid))
        .unwrap_or_default();
    let mut w = wire(
        RemoteId::Project(p.id.clone()).to_string(),
        &p.name,
        "",
        path,
        plain(p.is_archived),
    );
    w.container_remote_id = p
        .parent_id
        .clone()
        .map(|id| RemoteId::Project(id).to_string());
    w
}

pub fn section_to_wire(t: &Tree, s: &Section) -> WireObject {
    let path = t.project_path(&s.project_id).unwrap_or_default();
    let mut w = wire(
        RemoteId::Section(s.id.clone()).to_string(),
        &s.name,
        "",
        path,
        plain(false),
    );
    w.container_remote_id = Some(RemoteId::Project(s.project_id.clone()).to_string());
    w
}

pub fn item_to_wire(t: &Tree, i: &Item) -> Option<WireObject> {
    let path = t.item_path(i)?;
    let task = WireTask {
        done: i.checked,
        completed_at: i.completed_at.clone().filter(|_| i.checked),
        priority: dam_priority(i.priority),
        due: i
            .due
            .as_ref()
            .map(|d| dam_due(d, &jiff::tz::TimeZone::system())),
        deadline: i.deadline.as_ref().map(|d| d.date.clone()),
        event: None,
        estimate: None,
        focus: None,
    };
    let mut w = wire(
        RemoteId::Item(i.id.clone()).to_string(),
        &i.content,
        &i.description,
        path,
        task,
    );
    w.labels = i.labels.clone();
    w.container_remote_id = Some(nearest_container(i).to_string());
    w.recurrence = i
        .due
        .as_ref()
        .filter(|d| d.is_recurring)
        .map(|d| from_todoist(&d.string));
    Some(w)
}

fn nearest_container(i: &Item) -> RemoteId {
    match (&i.parent_id, &i.section_id) {
        (Some(parent), _) => RemoteId::Item(parent.clone()),
        (None, Some(section)) => RemoteId::Section(section.clone()),
        (None, None) => RemoteId::Project(i.project_id.clone()),
    }
}
