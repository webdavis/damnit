use dam_domain::paths_in_use;
use serde::Serialize;

use crate::args::{PathArgs, PathCommand};
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::view::{Human, View, report};

#[derive(Serialize)]
struct PathList {
    paths: Vec<PathRow>,
}

#[derive(Serialize)]
struct PathRow {
    path: String,
    count: usize,
}

impl View for PathList {
    fn human(&self) -> Human {
        let rows = self.paths.iter();
        let rows = rows.map(|p| vec![p.path.clone(), p.count.to_string()]);
        Human::list(rows.collect(), "no paths in use")
    }
}

pub(crate) fn run_path(ctx: &Context, args: PathArgs) -> Result<Report, CliError> {
    let PathCommand::List(list) = args.command;
    let objects = ctx.dam().list_within(&list.place)?;
    let paths = paths_in_use(&objects, &list.place).into_iter();
    let paths = paths.map(|(path, count)| PathRow {
        path: path.to_string(),
        count,
    });
    report(&PathList {
        paths: paths.collect(),
    })
}
