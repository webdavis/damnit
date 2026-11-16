use dam_domain::labels_in_use;
use serde::Serialize;

use crate::args::{LabelArgs, LabelCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct LabelList {
    labels: Vec<LabelRow>,
}

#[derive(Serialize)]
struct LabelRow {
    name: String,
    count: usize,
}

impl View for LabelList {
    fn human(&self) -> Human {
        let rows = self.labels.iter();
        let rows = rows.map(|l| vec![l.name.clone(), l.count.to_string()]);
        Human::list(rows.collect(), "no labels in use")
    }
}

pub(super) fn run_label(ctx: &Context, args: LabelArgs) -> Result<Report, CliError> {
    let LabelCommand::List(list) = args.command;
    let objects = ctx.dam().list_within(&list.place)?;
    let labels = labels_in_use(&objects).into_iter();
    let labels = labels.map(|(name, count)| LabelRow { name, count });
    report(&LabelList {
        labels: labels.collect(),
    })
}

#[cfg(test)]
mod label_tests;
