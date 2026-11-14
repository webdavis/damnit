use super::{RemoteId, Tree};

impl Tree {
    pub fn remove(&mut self, gone: &RemoteId) {
        let id = match gone {
            RemoteId::Project(id) => self.projects.remove(id).map(|p| p.id),
            RemoteId::Section(id) => self.sections.remove(id).map(|s| s.id),
            RemoteId::Item(id) => self.items.remove(id).map(|i| i.id),
        };
        if let Some(id) = id {
            self.added_this_push.forget(&id);
        }
    }

    pub fn move_item(
        &mut self,
        id: &str,
        project_id: &str,
        section_id: Option<&str>,
        parent_id: Option<&str>,
    ) {
        if let Some(item) = self.items.get_mut(id) {
            item.project_id = project_id.to_string();
            item.section_id = section_id.map(str::to_string);
            item.parent_id = parent_id.map(str::to_string);
        }
    }
}
