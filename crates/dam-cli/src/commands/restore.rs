use dam_application::Restored;
use dam_domain::Oid;
use serde::Serialize;

use crate::args::RestoreArgs;
use crate::commands::each::report_preview;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct RestoreView<'a> {
    #[serde(skip)]
    results: &'a [Restored],
    restored: Vec<String>,
    unchanged: Vec<String>,
}

impl View for RestoreView<'_> {
    fn human(&self) -> Human {
        Human::Lines(self.results.iter().map(line).collect())
    }
}

pub(crate) fn run(ctx: &Context, args: RestoreArgs) -> Result<Report, CliError> {
    let oids = args
        .oids
        .iter()
        .map(|text| ctx.dam().resolve_object(text))
        .collect::<Result<Vec<Oid>, _>>()?;
    if args.preview {
        let rehearsed = ctx.dam().rehearse(|dam| dam.restore(&oids))?;
        return report_preview(ctx, rehearsed.changes, None);
    }
    let restored = ctx.dam().restore(&oids)?;
    let ids = |changed: bool| {
        restored
            .iter()
            .filter(|r| r.changed == changed)
            .map(|r| r.oid.to_string())
            .collect::<Vec<String>>()
    };
    report(&RestoreView {
        results: &restored,
        restored: ids(true),
        unchanged: ids(false),
    })
}

fn line(r: &Restored) -> String {
    match r.changed {
        true => format!("restored  {}", r.oid.short()),
        false => format!("{} already matches the last commit", r.oid.short()),
    }
}

#[cfg(test)]
mod preview_tests;
#[cfg(test)]
mod tests;
