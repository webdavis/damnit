use serde::Serialize;

use super::entry::in_project;
use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct FilterCreated<'a> {
    pub(super) name: &'a str,
    pub(super) table: serde_json::Value,
    #[serde(skip)]
    pub(super) shown: String,
    pub(super) file: &'a str,
}

impl View for FilterCreated<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![format!(
            "created filter {}: {}{}",
            self.name,
            self.shown,
            in_project(self.file)
        )])
    }
}

#[derive(Serialize)]
pub(super) struct FilterRemoved<'a> {
    pub(super) name: &'a str,
    pub(super) file: &'a str,
}

impl View for FilterRemoved<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![format!(
            "removed filter {}{}",
            self.name,
            in_project(self.file)
        )])
    }
}
