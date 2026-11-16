use super::Tree;
use crate::api::{Item, Project, Section};

impl Tree {
    pub fn add_project(&mut self, p: Project) {
        self.projects.insert(p.id.clone(), p);
    }

    pub fn add_section(&mut self, s: Section) {
        self.sections.insert(s.id.clone(), s);
    }

    pub fn add_item(&mut self, i: Item) {
        self.items.insert(i.id.clone(), i);
    }
}
