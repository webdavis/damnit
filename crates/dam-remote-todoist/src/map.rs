use std::collections::{HashMap, HashSet};

use crate::api::{Item, Project, Section, SyncResponse};

/// A Todoist name as exactly one dam path segment: `/` cannot introduce an
/// extra level, and a blank name still names something.
fn segment(name: &str, id: &str) -> String {
    let clean = name.replace('/', "-");
    if clean.trim().is_empty() {
        format!("untitled-{id}")
    } else {
        clean
    }
}

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
        Some(format!("{parent}{}/", segment(&p.name, &p.id)))
    }

    pub fn section_path(&self, id: &str) -> Option<String> {
        let s = self.sections.get(id)?;
        Some(format!(
            "{}{}/",
            self.project_path(&s.project_id)?,
            segment(&s.name, &s.id)
        ))
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
                segment(&parent.content, &parent.id)
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

    pub fn add_project(&mut self, p: Project) {
        self.projects.insert(p.id.clone(), p);
    }

    pub fn add_section(&mut self, s: Section) {
        self.sections.insert(s.id.clone(), s);
    }

    pub fn add_item(&mut self, i: Item) {
        self.items.insert(i.id.clone(), i);
    }

    /// True when a project, section or item is already rooted at `path`, so a
    /// depth-1 object with something under it becomes a section, not a task.
    pub fn has_children_at(&self, path: &str) -> bool {
        self.projects.values().any(|p| {
            !p.is_deleted
                && p.parent_id
                    .as_deref()
                    .and_then(|id| self.project_path(id))
                    .as_deref()
                    == Some(path)
        }) || self
            .sections
            .values()
            .any(|s| !s.is_deleted && self.project_path(&s.project_id).as_deref() == Some(path))
            || self
                .items
                .values()
                .any(|i| !i.is_deleted && self.item_path(i).as_deref() == Some(path))
    }
}

mod identity;
mod wire;

pub use identity::{api_priority, dam_priority, remote_id, split_remote_id};
pub use wire::{item_to_wire, project_to_wire, section_to_wire};

#[cfg(test)]
mod tests;
