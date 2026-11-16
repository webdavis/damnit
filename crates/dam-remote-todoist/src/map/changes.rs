use super::{RemoteId, Tree};

impl Tree {
    pub fn remove(&mut self, gone: &RemoteId) {
        match gone {
            RemoteId::Project(id) => {
                self.projects.remove(id);
            }
            RemoteId::Section(id) => {
                self.sections.remove(id);
            }
            RemoteId::Item(id) => {
                self.items.remove(id);
            }
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

    pub fn rename(&mut self, renamed: &RemoteId, name: &str) {
        let name = name.to_string();
        match renamed {
            RemoteId::Project(id) => self.projects.get_mut(id).map(|p| p.name = name),
            RemoteId::Section(id) => self.sections.get_mut(id).map(|s| s.name = name),
            RemoteId::Item(id) => self.items.get_mut(id).map(|i| i.content = name),
        };
    }
}
