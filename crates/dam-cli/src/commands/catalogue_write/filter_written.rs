use serde::Serialize;

use crate::view::{Human, View};

#[derive(Serialize)]
pub(super) struct FilterCreated<'a> {
    pub(super) name: &'a str,
    pub(super) query: &'a str,
}

impl View for FilterCreated<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![format!(
            "created filter {}: {}",
            self.name, self.query
        )])
    }
}

#[derive(Serialize)]
pub(super) struct FilterRemoved<'a> {
    pub(super) name: &'a str,
}

impl View for FilterRemoved<'_> {
    fn human(&self) -> Human {
        Human::Lines(vec![format!("removed filter {}", self.name)])
    }
}
