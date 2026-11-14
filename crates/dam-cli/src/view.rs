mod objects;
mod serialize;

use serde::Serialize;

use crate::error::CliError;
use crate::layout::table;
use crate::output::Report;
use crate::paint::Tint;

pub(crate) use objects::{ObjectList, ObjectRow};
pub(crate) use serialize::{display, displays, document, optional_display, optional_wire, wire};

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

    fn notices(&self) -> Vec<String> {
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
        stderr: view
            .warnings()
            .iter()
            .map(|w| format!("warning: {w}"))
            .chain(view.notices().iter().map(|n| format!("dam: {n}")))
            .collect(),
    })
}
