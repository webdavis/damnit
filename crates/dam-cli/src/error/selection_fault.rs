use dam_application::SelectError;
use dam_domain::{FilterTable, select_field};

use crate::args::selection::flag_of;

mod command_line;
mod fix;
mod sentence;
mod sort_fix;

use command_line::command_line;

#[derive(Debug)]
pub(crate) struct Fault {
    pub(crate) error: SelectError,
    pub(crate) typed: FilterTable,
    pub(crate) sort: Vec<String>,
}

impl Fault {
    pub(crate) fn field(&self) -> &str {
        match &self.error {
            SelectError::Value(e) => &e.field,
            SelectError::Defaults(e) => fix::dropped_or_moved(e),
            SelectError::Sort(_) => "sort",
        }
    }

    pub(crate) fn value(&self) -> String {
        match &self.error {
            SelectError::Value(e) | SelectError::Sort(e) => e.value.clone(),
            SelectError::Defaults(_) => fix::typed_text(&self.typed, self.field()),
        }
    }

    pub(crate) fn shapes(&self) -> Vec<String> {
        if let SelectError::Sort(_) = self.error {
            return vec!["FIELD".into(), "FIELD:desc".into()];
        }
        select_field(self.field()).map_or_else(Vec::new, |f| f.shapes())
    }

    pub(crate) fn command(&self) -> String {
        let sort = sort_fix::sorted(&self.error, &self.sort);
        command_line(&fix::corrected(&self.error, &self.typed), &sort)
    }

    pub(crate) fn conflict(&self) -> bool {
        matches!(self.error, SelectError::Defaults(_))
    }

    pub(crate) fn word(&self) -> String {
        match &self.error {
            SelectError::Value(e) if !self.typed.fields.contains_key(&e.field) => "--any".into(),
            _ => format!("--{}", flag_of(self.field())),
        }
    }

    pub(crate) fn document(&self) -> serde_json::Value {
        serde_json::json!({
            "field": self.field(),
            "value": self.value(),
            "shapes": self.shapes(),
            "command": self.command(),
        })
    }
}

impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sentence = match &self.error {
            SelectError::Value(e) => sentence::of_value(self, &e.problem),
            SelectError::Defaults(e) => sentence::of_defaults(self, e),
            SelectError::Sort(e) => sentence::of_sort(self, &e.problem),
        };
        write!(f, "{sentence}; run {}", self.command())
    }
}

#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod tests;
