use super::{RemoteId, Tree, one_path_segment};

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

    pub fn rename(&mut self, renamed: &RemoteId, name: &str) {
        let was = self.path_inside(renamed);
        let name = name.to_string();
        match renamed {
            RemoteId::Project(id) => self.projects.get_mut(id).map(|p| p.name = name),
            RemoteId::Section(id) => self.sections.get_mut(id).map(|s| s.name = name),
            RemoteId::Item(id) => self.items.get_mut(id).map(|i| i.content = name),
        };
        if let (Some(was), Some(now)) = (was, self.path_inside(renamed)) {
            self.added_this_push.moved_from_to(&was, &now);
        }
    }

    fn path_inside(&self, container: &RemoteId) -> Option<String> {
        match container {
            RemoteId::Project(id) => self.project_path(id),
            RemoteId::Section(id) => self.section_path(id),
            RemoteId::Item(id) => {
                let item = self.items.get(id)?;
                let place = self.item_path(item)?;
                Some(format!("{place}{}/", one_path_segment(&item.content, id)))
            }
        }
    }
}
