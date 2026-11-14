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
        self.keeping_what_was_added_beneath(&RemoteId::Item(id.to_string()), |tree| {
            if let Some(item) = tree.items.get_mut(id) {
                item.project_id = project_id.to_string();
                item.section_id = section_id.map(str::to_string);
                item.parent_id = parent_id.map(str::to_string);
            }
            if let Some(place) = tree.items.get(id).and_then(|i| tree.item_path(i)) {
                tree.added_this_push.item_now_at(id, place);
            }
        });
    }

    pub fn rename(&mut self, renamed: &RemoteId, name: &str) {
        let name = name.to_string();
        self.keeping_what_was_added_beneath(renamed, |tree| {
            match renamed {
                RemoteId::Project(id) => tree.projects.get_mut(id).map(|p| p.name = name),
                RemoteId::Section(id) => tree.sections.get_mut(id).map(|s| s.name = name),
                RemoteId::Item(id) => tree.items.get_mut(id).map(|i| i.content = name),
            };
        });
    }

    fn keeping_what_was_added_beneath(
        &mut self,
        changed: &RemoteId,
        change: impl FnOnce(&mut Tree),
    ) {
        let was = self.path_inside(changed);
        change(self);
        if let (Some(was), Some(now)) = (was, self.path_inside(changed)) {
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
