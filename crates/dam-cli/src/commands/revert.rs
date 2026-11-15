use dam_application::Fields;
use serde::Serialize;

use crate::args::RevertArgs;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

pub(crate) fn run(ctx: &Context, args: RevertArgs) -> Result<Report, CliError> {
    let reverted = ctx.dam().revert(&args.commit, &args.oids)?;
    report(&RevertView {
        reverted: reverted.reverted.iter().map(Row::from).collect(),
        kept: reverted.kept.iter().map(Row::from).collect(),
    })
}

#[derive(Serialize)]
struct RevertView {
    reverted: Vec<Row>,
    kept: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    oid: String,
    fields: Vec<&'static str>,
    #[serde(skip)]
    short: String,
}

impl From<&Fields> for Row {
    fn from(f: &Fields) -> Row {
        Row {
            oid: f.oid.to_string(),
            fields: f.fields.iter().map(|field| field.as_str()).collect(),
            short: f.oid.short().to_string(),
        }
    }
}

impl View for RevertView {
    fn human(&self) -> Human {
        let reverted = self.reverted.iter().map(|r| r.cells("reverted", ""));
        let kept = self.kept.iter().map(|r| r.cells("kept", "changed since: "));
        Human::list(reverted.chain(kept).collect(), "nothing to revert")
    }
}

impl Row {
    fn cells(&self, word: &str, why: &str) -> Vec<String> {
        vec![
            word.to_string(),
            self.short.clone(),
            format!("{why}{}", self.fields.join(", ")),
        ]
    }
}
