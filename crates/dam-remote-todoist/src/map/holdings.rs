use super::{RemoteId, Tree};

impl Tree {
    pub fn what_it_still_holds(&self, container: &RemoteId) -> Option<String> {
        let live_items = || self.items.values().filter(|i| !i.is_deleted);
        let counts = match container {
            RemoteId::Item(id) => [
                live_items()
                    .filter(|i| i.parent_id.as_ref() == Some(id))
                    .count(),
                0,
                0,
            ],
            RemoteId::Section(id) => [
                live_items()
                    .filter(|i| i.section_id.as_ref() == Some(id))
                    .count(),
                0,
                0,
            ],
            RemoteId::Project(id) => [
                live_items().filter(|i| &i.project_id == id).count(),
                self.sections
                    .values()
                    .filter(|s| !s.is_deleted && &s.project_id == id)
                    .count(),
                self.projects
                    .values()
                    .filter(|p| !p.is_deleted && p.parent_id.as_ref() == Some(id))
                    .count(),
            ],
        };
        let named: Vec<String> = counts
            .into_iter()
            .zip(["task", "section", "project"])
            .filter(|(n, _)| *n > 0)
            .map(|(n, what)| format!("{n} {what}{}", if n == 1 { "" } else { "s" }))
            .collect();
        (!named.is_empty()).then(|| named.join(", "))
    }
}
