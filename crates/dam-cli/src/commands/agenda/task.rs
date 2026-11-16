use dam_application::Due;
use dam_domain::{Object, weekday_text};
use jiff::tz::TimeZone;
use serde::Serialize;
use serde_json::Value;

use crate::output::object_cells;
use crate::view::{document, stamp_worked_out};

#[derive(Serialize)]
#[serde(transparent)]
pub(super) struct AgendaTask {
    row: Value,
    #[serde(skip)]
    cells: Vec<String>,
}

impl AgendaTask {
    pub(super) fn new(due: &Due, zone: &TimeZone) -> AgendaTask {
        let object = Object::Task(due.task.clone());
        let mut row = serde_json::to_value(document(&object)).unwrap_or_default();
        stamp_worked_out(&mut row, &due.worked_out);
        if let Some(fields) = row.as_object_mut() {
            fields.insert("day".into(), due.day.to_string().into());
            fields.insert("weekday".into(), weekday_text(due.day.weekday()).into());
        }
        let mut cells = object_cells(&object, zone);
        cells.insert(3, String::new());
        AgendaTask { row, cells }
    }

    pub(super) fn cells(&self) -> Vec<String> {
        self.cells.clone()
    }
}
