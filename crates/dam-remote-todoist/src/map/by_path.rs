use super::Tree;

impl Tree {
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
