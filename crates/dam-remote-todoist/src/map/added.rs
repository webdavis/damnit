use std::collections::HashMap;

use super::Tree;
use crate::api::{Item, Project, Section};

#[derive(Default)]
pub(super) struct AddedThisPush {
    pub(super) project_by_path: HashMap<String, String>,
    pub(super) section_by_path: HashMap<String, String>,
    pub(super) item_by_place_and_content: HashMap<(String, String), String>,
}

impl Tree {
    pub fn add_project(&mut self, p: Project) {
        let id = p.id.clone();
        self.projects.insert(id.clone(), p);
        if let Some(path) = self.project_path(&id) {
            self.added_this_push.project_by_path.insert(path, id);
        }
    }

    pub fn add_section(&mut self, s: Section) {
        let id = s.id.clone();
        self.sections.insert(id.clone(), s);
        if let Some(path) = self.section_path(&id) {
            self.added_this_push.section_by_path.insert(path, id);
        }
    }

    pub fn add_item(&mut self, i: Item) {
        if let Some(place) = self.item_path(&i) {
            self.added_this_push
                .item_by_place_and_content
                .insert((place, i.content.clone()), i.id.clone());
        }
        self.items.insert(i.id.clone(), i);
    }
}
