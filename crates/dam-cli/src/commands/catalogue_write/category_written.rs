use serde::Serialize;

use super::entry::in_project;
use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct CategoryCreated<'a> {
    pub(super) name: &'a str,
    pub(super) values: &'a [String],
    pub(super) exclusive: bool,
    pub(super) file: &'a str,
}

impl View for CategoryCreated<'_> {
    fn human(&self) -> Human {
        let exclusive = if self.exclusive { " (exclusive)" } else { "" };
        Human::Lines(vec![format!(
            "created category {}: {}{exclusive}{}",
            self.name,
            self.values.join(", "),
            in_project(self.file)
        )])
    }
}

#[derive(Serialize)]
pub(super) struct CategoryRemoved<'a> {
    pub(super) name: &'a str,
    pub(super) tasks_keeping_labels: usize,
    pub(super) file: &'a str,
}

impl View for CategoryRemoved<'_> {
    fn human(&self) -> Human {
        let mut lines = vec![format!(
            "removed category {}{}",
            self.name,
            in_project(self.file)
        )];
        match self.tasks_keeping_labels {
            0 => {}
            1 => lines.push("1 task keeps its labels as free labels".into()),
            n => lines.push(format!("{n} tasks keep its labels as free labels")),
        }
        Human::Lines(lines)
    }
}
