use std::collections::HashMap;

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
        let p = self.projects.get(id)?;
        let parent = match &p.parent_id {
            Some(pid) => self.project_path(pid)?,
            None => String::new(),
        };
        Some(format!("{parent}{}/", p.name))
    }

    pub fn section_path(&self, id: &str) -> Option<String> {
        let s = self.sections.get(id)?;
        Some(format!("{}{}/", self.project_path(&s.project_id)?, s.name))
    }

    pub fn item_path(&self, item: &Item) -> Option<String> {
        if let Some(parent) = item.parent_id.as_deref().and_then(|p| self.items.get(p)) {
            return Some(format!("{}{}/", self.item_path(parent)?, parent.content));
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
mod tests {
    use super::*;
    use crate::api::{Due, Item, Project, Section, SyncResponse};

    fn sync() -> SyncResponse {
        SyncResponse {
            projects: vec![
                Project {
                    id: "p1".into(),
                    name: "Work".into(),
                    parent_id: None,
                    is_deleted: false,
                    is_archived: false,
                    inbox_project: false,
                },
                Project {
                    id: "p2".into(),
                    name: "Client".into(),
                    parent_id: Some("p1".into()),
                    is_deleted: false,
                    is_archived: true,
                    inbox_project: false,
                },
            ],
            sections: vec![Section {
                id: "s1".into(),
                name: "Now".into(),
                project_id: "p1".into(),
                is_deleted: false,
            }],
            items: vec![
                Item {
                    id: "i1".into(),
                    content: "milk".into(),
                    description: "2%".into(),
                    project_id: "p1".into(),
                    section_id: Some("s1".into()),
                    parent_id: None,
                    priority: 4,
                    due: Some(Due {
                        date: "2026-09-25".into(),
                        is_recurring: true,
                        string: "every week".into(),
                    }),
                    deadline: None,
                    labels: vec!["errand".into()],
                    checked: false,
                    is_deleted: false,
                },
                Item {
                    id: "i2".into(),
                    content: "oat".into(),
                    description: String::new(),
                    project_id: "p1".into(),
                    section_id: Some("s1".into()),
                    parent_id: Some("i1".into()),
                    priority: 1,
                    due: None,
                    deadline: None,
                    labels: vec![],
                    checked: true,
                    is_deleted: false,
                },
                Item {
                    id: "i3".into(),
                    content: "loose".into(),
                    description: String::new(),
                    project_id: "p1".into(),
                    section_id: None,
                    parent_id: None,
                    priority: 2,
                    due: None,
                    deadline: None,
                    labels: vec![],
                    checked: false,
                    is_deleted: false,
                },
            ],
        }
    }

    #[test]
    fn paths_follow_project_section_and_parent_names() {
        let t = Tree::from_sync(&sync());
        assert_eq!(t.project_path("p1").as_deref(), Some("Work/"));
        assert_eq!(t.project_path("p2").as_deref(), Some("Work/Client/"));
        assert_eq!(t.section_path("s1").as_deref(), Some("Work/Now/"));
        let items = &sync().items;
        assert_eq!(t.item_path(&items[0]).as_deref(), Some("Work/Now/"));
        assert_eq!(t.item_path(&items[1]).as_deref(), Some("Work/Now/milk/"));
        assert_eq!(t.item_path(&items[2]).as_deref(), Some("Work/"));
    }

    #[test]
    fn wire_objects_carry_kind_fields_and_remote_ids() {
        let s = sync();
        let t = Tree::from_sync(&s);
        let p = project_to_wire(&t, &s.projects[1]);
        assert_eq!(p.remote_id.as_deref(), Some("p:p2"));
        assert_eq!(p.path, "Work/");
        assert_eq!(p.subject, "Client");
        assert!(p.task.as_ref().unwrap().done, "archived is done");
        let i = item_to_wire(&t, &s.items[0]).unwrap();
        assert_eq!(i.remote_id.as_deref(), Some("i:i1"));
        assert_eq!(i.subject, "milk");
        assert_eq!(i.body, "2%");
        assert_eq!(i.labels, vec!["errand".to_string()]);
        assert_eq!(i.task.as_ref().unwrap().priority, 1, "API 4 is p1");
        assert_eq!(i.task.as_ref().unwrap().due.as_deref(), Some("2026-09-25"));
        assert_eq!(i.recurrence.as_deref(), Some("every week"));
        let child = item_to_wire(&t, &s.items[1]).unwrap();
        assert!(child.task.as_ref().unwrap().done);
        assert_eq!(child.task.as_ref().unwrap().priority, 4);
    }

    #[test]
    fn reverse_lookups_find_ids_by_path() {
        let t = Tree::from_sync(&sync());
        assert_eq!(t.project_id_for("Work/"), Some("p1"));
        assert_eq!(t.project_id_for("Work/Client/"), Some("p2"));
        assert_eq!(t.section_id_for("Work/Now/"), Some("s1"));
        assert_eq!(t.item_id_for("Work/Now/", "milk"), Some("i1"));
        assert!(t.project_id_for("Nope/").is_none());
    }

    #[test]
    fn priorities_map_both_ways() {
        assert_eq!(api_priority(1), 4);
        assert_eq!(api_priority(4), 1);
        assert_eq!(dam_priority(4), 1);
        assert_eq!(dam_priority(0), 4);
        assert_eq!(dam_priority(9), 1);
        assert_eq!(split_remote_id("i:abc"), Some(('i', "abc")));
        assert_eq!(split_remote_id("abc"), None);
    }
}
