mod confirm;
mod requests;
mod schema;
mod verbs;

use clap::Parser;
use dam_domain::{Object, Oid, Path, Task};

use crate::args::Cli;
use crate::commands::dispatch_a_verb_that_opens_the_store;
use crate::context::Context;
use crate::error::CliError;
use crate::output::Report;
use crate::testing::context;

fn filed(ctx: &Context, byte: u8, path: &str) -> Oid {
    let oid = Oid::generate(&mut |x: &mut [u8]| x.fill(byte));
    let mut task = Task::new(oid.clone(), format!("t{byte}"));
    task.base.path = Path::parse(path).unwrap();
    ctx.store.put(&Object::Task(task)).unwrap();
    oid
}

fn three_tasks() -> (Context, Vec<String>) {
    let ctx = context();
    let oids = [(1, "work"), (2, "work"), (3, "home")]
        .map(|(byte, path)| filed(&ctx, byte, path).as_str().to_string());
    (ctx, oids.into())
}

fn dam(ctx: &Context, words: &[&str]) -> Result<Report, CliError> {
    let line = std::iter::once("dam").chain(words.iter().copied());
    let cli = Cli::try_parse_from(line).map_err(|e| CliError::Usage(e.to_string()))?;
    dispatch_a_verb_that_opens_the_store(ctx, cli.command)
}

fn last_line(report: &Report) -> &str {
    report.human.lines().last().unwrap()
}
