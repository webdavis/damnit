use std::collections::{HashMap, HashSet};

use dam_protocol::WireObject;

use crate::api::{Item, Project, Section, SyncResponse};
use added::AddedThisPush;

fn one_path_segment(name: &str, id: &str) -> String {
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
    added_this_push: AddedThisPush,
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
            added_this_push: AddedThisPush::default(),
        }
    }

    pub fn project_path(&self, id: &str) -> Option<String> {
        self.project_path_unless_cycle(id, &mut HashSet::new())
    }

    fn project_path_unless_cycle(
        &self,
        id: &str,
        ids_on_this_walk: &mut HashSet<String>,
    ) -> Option<String> {
        if !ids_on_this_walk.insert(id.to_string()) {
            return None;
        }
        let p = self.projects.get(id)?;
        let parent = match &p.parent_id {
            Some(pid) => self.project_path_unless_cycle(pid, ids_on_this_walk)?,
            None => String::new(),
        };
        Some(format!("{parent}{}/", one_path_segment(&p.name, &p.id)))
    }

    pub fn section_path(&self, id: &str) -> Option<String> {
        let s = self.sections.get(id)?;
        Some(format!(
            "{}{}/",
            self.project_path(&s.project_id)?,
            one_path_segment(&s.name, &s.id)
        ))
    }

    pub fn item_path(&self, item: &Item) -> Option<String> {
        self.item_path_unless_cycle(item, &mut HashSet::new())
    }

    fn item_path_unless_cycle(
        &self,
        item: &Item,
        ids_on_this_walk: &mut HashSet<String>,
    ) -> Option<String> {
        if !ids_on_this_walk.insert(item.id.clone()) {
            return None;
        }
        if let Some(parent) = item.parent_id.as_deref().and_then(|p| self.items.get(p)) {
            return Some(format!(
                "{}{}/",
                self.item_path_unless_cycle(parent, ids_on_this_walk)?,
                one_path_segment(&parent.content, &parent.id)
            ));
        }
        match &item.section_id {
            Some(s) => self.section_path(s),
            None => self.project_path(&item.project_id),
        }
    }

    pub fn held_wire(&self, remote: &RemoteId) -> Option<WireObject> {
        match remote {
            RemoteId::Project(id) => self.projects.get(id).map(|p| project_to_wire(self, p)),
            RemoteId::Section(id) => self.sections.get(id).map(|s| section_to_wire(self, s)),
            RemoteId::Item(id) => self.items.get(id).and_then(|i| item_to_wire(self, i)),
        }
    }
}

mod added;
mod by_id;
mod by_path;
mod changes;
mod holdings;
mod identity;
mod open;
mod wire;

pub use identity::{RemoteId, api_priority, dam_priority};
pub use wire::{item_to_wire, project_to_wire, section_to_wire};

#[cfg(test)]
mod tests;
