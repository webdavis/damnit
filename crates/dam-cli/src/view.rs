use std::fmt::Display;

use dam_adapters::to_wire;
use dam_domain::{Object, When};
use jiff::tz::TimeZone;
use serde::{Serialize, Serializer};

use crate::error::CliError;
use crate::layout::table;
use crate::output::{Report, object_cells};
use crate::paint::Tint;

pub(crate) enum Human {
    Table(Vec<Vec<String>>),
    Lines(Vec<String>),
    Mixed(Vec<Line>),
}

pub(crate) enum Line {
    Row(String),
    Whole(String),
    Removed(String),
    Added(String),
}

impl Human {
    pub(crate) fn list(rows: Vec<Vec<String>>, empty: &str) -> Human {
        if rows.is_empty() {
            Human::Lines(vec![empty.to_string()])
        } else {
            Human::Table(rows)
        }
    }
}

pub(crate) trait View: Serialize {
    fn human(&self) -> Human;

    fn warnings(&self) -> Vec<String> {
        Vec::new()
    }
}

pub(crate) fn report(view: &impl View) -> Result<Report, CliError> {
    let lines: Vec<(String, bool, Tint)> = match view.human() {
        Human::Table(rows) => table(&rows)
            .into_iter()
            .map(|l| (l, true, Tint::Plain))
            .collect(),
        Human::Lines(lines) => lines.into_iter().map(|l| (l, false, Tint::Plain)).collect(),
        Human::Mixed(lines) => lines
            .into_iter()
            .map(|line| match line {
                Line::Row(l) => (l, true, Tint::Plain),
                Line::Whole(l) => (l, false, Tint::Plain),
                Line::Removed(l) => (l, true, Tint::Removed),
                Line::Added(l) => (l, true, Tint::Added),
            })
            .collect(),
    };
    Ok(Report {
        human: lines
            .iter()
            .map(|(l, ..)| l.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        data: serde_json::to_value(view).map_err(|e| CliError::Internal(e.to_string()))?,
        cut: lines.iter().map(|(_, cut, _)| *cut).collect(),
        tints: lines.iter().map(|(.., tint)| *tint).collect(),
        warnings: view.warnings(),
    })
}

#[derive(Serialize)]
pub(crate) struct ObjectList<'a> {
    #[serde(serialize_with = "wires")]
    pub(crate) objects: &'a [Object],
    #[serde(skip)]
    pub(crate) authors: &'a [&'a str],
    #[serde(skip)]
    pub(crate) zone: &'a TimeZone,
}

impl View for ObjectList<'_> {
    fn human(&self) -> Human {
        Human::list(
            self.objects
                .iter()
                .zip(self.authors)
                .map(|(o, author)| {
                    let mut cells = object_cells(o, self.zone);
                    cells.insert(4, author.to_string());
                    cells
                })
                .collect(),
            "no objects match",
        )
    }
}

#[derive(Serialize)]
#[serde(transparent)]
pub(crate) struct ObjectRow<'a>(
    #[serde(serialize_with = "wire")] pub(crate) &'a Object,
    #[serde(skip)] pub(crate) &'a TimeZone,
);

impl View for ObjectRow<'_> {
    fn human(&self) -> Human {
        Human::Table(vec![object_cells(self.0, self.1)])
    }
}

pub(crate) fn display<T: Display, S: Serializer>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

pub(crate) fn optional_display<T: Display, S: Serializer>(
    value: &Option<T>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    value
        .as_ref()
        .map(ToString::to_string)
        .serialize(serializer)
}

pub(crate) fn displays<T: Display, S: Serializer>(
    values: &&[T],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(values.iter().map(ToString::to_string))
}

pub(crate) fn document(object: &Object) -> impl Serialize + use<> {
    let mut document = to_wire(object, None);
    if let (Some(task), Some(t)) = (document.task.as_mut(), object.as_task()) {
        task.due = t.due.as_ref().map(When::to_rfc3339);
    }
    if let (Some(event), Some(e)) = (document.event.as_mut(), object.as_event()) {
        event.start = e.start.to_rfc3339();
        event.end = e.end.to_rfc3339();
    }
    document
}

pub(crate) fn wire<S: Serializer>(object: &&Object, serializer: S) -> Result<S::Ok, S::Error> {
    document(object).serialize(serializer)
}

pub(crate) fn optional_wire<S: Serializer>(
    object: &Option<&Object>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    object.map(document).serialize(serializer)
}

fn wires<S: Serializer>(objects: &&[Object], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_seq(objects.iter().map(document))
}
