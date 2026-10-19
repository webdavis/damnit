use std::fmt::Display;

use dam_adapters::to_wire;
use dam_domain::Object;
use serde::{Serialize, Serializer};

use crate::error::CliError;
use crate::output::{Report, object_cells};

pub(crate) enum Human {
    Table(Vec<Vec<String>>),
    Lines(Vec<String>),
}

pub(crate) trait View: Serialize {
    fn human(&self) -> Human;
}

pub(crate) fn report(view: &impl View) -> Result<Report, CliError> {
    let lines = match view.human() {
        Human::Table(rows) => rows.iter().map(|cells| row(cells)).collect(),
        Human::Lines(lines) => lines,
    };
    Ok(Report {
        human: lines.join("\n"),
        data: serde_json::to_value(view).map_err(|e| CliError::Internal(e.to_string()))?,
    })
}

pub(crate) fn row(cells: &[String]) -> String {
    cells.join("  ")
}

#[derive(Serialize)]
pub(crate) struct ObjectList<'a> {
    #[serde(serialize_with = "wires")]
    pub(crate) objects: &'a [Object],
}

impl View for ObjectList<'_> {
    fn human(&self) -> Human {
        Human::Table(self.objects.iter().map(object_cells).collect())
    }
}

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct ObjectRow<'a>(#[serde(serialize_with = "wire")] pub(crate) &'a Object);

impl View for ObjectRow<'_> {
    fn human(&self) -> Human {
        Human::Table(vec![object_cells(self.0)])
    }
}

pub(crate) fn display<T: Display, S: Serializer>(value: &T, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(value)
}

pub(crate) fn optional_display<T: Display, S: Serializer>(
    value: &Option<T>,
    s: S,
) -> Result<S::Ok, S::Error> {
    value.as_ref().map(ToString::to_string).serialize(s)
}

pub(crate) fn displays<T: Display, S: Serializer>(values: &&[T], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(values.iter().map(ToString::to_string))
}

pub(crate) fn wire<S: Serializer>(object: &&Object, s: S) -> Result<S::Ok, S::Error> {
    to_wire(object, None).serialize(s)
}

pub(crate) fn optional_wire<S: Serializer>(
    object: &Option<&Object>,
    s: S,
) -> Result<S::Ok, S::Error> {
    object.map(|o| to_wire(o, None)).serialize(s)
}

fn wires<S: Serializer>(objects: &&[Object], s: S) -> Result<S::Ok, S::Error> {
    s.collect_seq(objects.iter().map(|o| to_wire(o, None)))
}
