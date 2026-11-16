use super::{Categories, Category};

fn overlap(a: &Category, b: &Category) -> bool {
    a.name == b.name || a.values.iter().any(|v| b.values.contains(v))
}

impl Categories {
    pub fn within(&self, project: &Categories) -> Categories {
        let kept = self
            .0
            .iter()
            .filter(|yours| !project.0.iter().any(|theirs| overlap(yours, theirs)));
        Categories(project.0.iter().chain(kept).cloned().collect())
    }

    pub fn set_aside_by<'a>(&'a self, project: &'a Categories) -> Vec<(&'a str, &'a str)> {
        self.0
            .iter()
            .filter_map(|yours| {
                project
                    .0
                    .iter()
                    .find(|theirs| overlap(yours, theirs))
                    .map(|theirs| (yours.name.as_str(), theirs.name.as_str()))
            })
            .collect()
    }
}
