use super::Tree;

impl Tree {
    pub fn shows_item_open(&self, id: &str) -> bool {
        self.items.get(id).is_some_and(|item| !item.checked)
    }

    pub fn shows_project_unarchived(&self, id: &str) -> bool {
        self.projects
            .get(id)
            .is_some_and(|project| !project.is_archived)
    }
}
