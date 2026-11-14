use super::{RemoteId, Tree};

impl Tree {
    pub fn project_and_section_of(&self, container: &RemoteId) -> Option<(String, Option<String>)> {
        match container {
            RemoteId::Item(id) => self
                .items
                .get(id)
                .filter(|i| !i.is_deleted)
                .map(|i| (i.project_id.clone(), i.section_id.clone())),
            RemoteId::Section(id) => self
                .sections
                .get(id)
                .filter(|s| !s.is_deleted)
                .map(|s| (s.project_id.clone(), Some(id.clone()))),
            RemoteId::Project(id) => self
                .projects
                .get(id)
                .filter(|p| !p.is_deleted)
                .map(|_| (id.clone(), None)),
        }
    }
}
