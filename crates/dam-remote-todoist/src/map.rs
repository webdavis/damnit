use std::collections::{HashMap, HashSet};

use dam_protocol::{WireObject, WireTask};

use crate::api::{Item, Project, Section, SyncResponse};

pub struct Tree {
    projects: HashMap<String, Project>,
    sections: HashMap<String, Section>,
    items: HashMap<String, Item>,
}

impl Tree {
    pub fn from_sync(sync: &SyncResponse) -> Tree {
        Tree {
            projects: sync
                .projects
                .iter()
                .map(|p| (p.id.clone(), p.clone()))
                .collect(),
            sections: sync
                .sections
                .iter()
                .map(|s| (s.id.clone(), s.clone()))
                .collect(),
            items: sync
                .items
                .iter()
                .map(|i| (i.id.clone(), i.clone()))
                .collect(),
        }
    }

    pub fn project_path(&self, id: &str) -> Option<String> {
        self.project_path_from(id, &mut HashSet::new())
    }

    /// `visited` catches a parent cycle in the untrusted sync response: a
    /// repeated id ends the walk with `None` instead of recursing forever.
    fn project_path_from(&self, id: &str, visited: &mut HashSet<String>) -> Option<String> {
        if !visited.insert(id.to_string()) {
            return None;
        }
        let p = self.projects.get(id)?;
        let parent = match &p.parent_id {
            Some(pid) => self.project_path_from(pid, visited)?,
            None => String::new(),
        };
        Some(format!("{parent}{}/", p.name))
    }

    pub fn section_path(&self, id: &str) -> Option<String> {
        let s = self.sections.get(id)?;
        Some(format!("{}{}/", self.project_path(&s.project_id)?, s.name))
    }

    pub fn item_path(&self, item: &Item) -> Option<String> {
        self.item_path_from(item, &mut HashSet::new())
    }

    /// Same cycle guard as `project_path_from`, keyed by item id.
    fn item_path_from(&self, item: &Item, visited: &mut HashSet<String>) -> Option<String> {
        if !visited.insert(item.id.clone()) {
            return None;
        }
        if let Some(parent) = item.parent_id.as_deref().and_then(|p| self.items.get(p)) {
            return Some(format!(
                "{}{}/",
                self.item_path_from(parent, visited)?,
                parent.content
            ));
        }
        match &item.section_id {
            Some(s) => self.section_path(s),
            None => self.project_path(&item.project_id),
        }
    }

    pub fn project_id_for(&self, path: &str) -> Option<&str> {
        self.projects
            .values()
            .find(|p| !p.is_deleted && self.project_path(&p.id).as_deref() == Some(path))
            .map(|p| p.id.as_str())
    }

    pub fn section_id_for(&self, path: &str) -> Option<&str> {
        self.sections
            .values()
            .find(|s| !s.is_deleted && self.section_path(&s.id).as_deref() == Some(path))
            .map(|s| s.id.as_str())
    }

    pub fn item_id_for(&self, path: &str, content: &str) -> Option<&str> {
        self.items
            .values()
            .find(|i| {
                !i.is_deleted && i.content == content && self.item_path(i).as_deref() == Some(path)
            })
            .map(|i| i.id.as_str())
    }
}

pub fn remote_id(kind: char, id: &str) -> String {
    format!("{kind}:{id}")
}

pub fn split_remote_id(text: &str) -> Option<(char, &str)> {
    let (kind, id) = text.split_once(':')?;
    let mut chars = kind.chars();
    match (chars.next(), chars.next()) {
        (Some(k @ ('p' | 's' | 'i')), None) => Some((k, id)),
        _ => None,
    }
}

pub fn api_priority(dam: u8) -> u8 {
    5u8.saturating_sub(dam.clamp(1, 4))
}

pub fn dam_priority(api: u8) -> u8 {
    5u8.saturating_sub(api.clamp(1, 4))
}

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

#[cfg(test)]
mod tests;
