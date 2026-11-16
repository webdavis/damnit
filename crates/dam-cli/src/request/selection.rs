mod read;

use std::ffi::OsString;

use clap::{ArgMatches, CommandFactory};
use dam_domain::select_field;
use serde_json::{Map, Value};

use super::{command_at, leaf, with_fields};
use crate::args::{Cli, Command};
use crate::error::CliError;

const SELECTION_KEYS: [&str; 4] = ["any", "sort", "limit", "offset"];

pub(super) fn with_selection(
    line: Vec<OsString>,
    given: &ArgMatches,
    fields: Map<String, Value>,
) -> Result<Cli, CliError> {
    let (words, _) = leaf(given);
    let list = command_at(Cli::command(), &words);
    let (passed, selection): (Map<String, Value>, Map<String, Value>) = fields
        .into_iter()
        .partition(|(key, _)| passes_through(&list, key));
    let mut cli = with_fields(line, given, &passed)?;
    let Command::List(args) = &mut cli.command else {
        return Err(CliError::Internal(
            "a list request read as another command".into(),
        ));
    };
    args.selection.typed = read::table(&selection)?;
    args.order = read::order(&selection)?;
    Ok(cli)
}

fn passes_through(list: &clap::Command, key: &str) -> bool {
    !SELECTION_KEYS.contains(&key)
        && select_field(key).is_none()
        && list.get_arguments().any(|arg| arg.get_id() == key)
}

#[cfg(test)]
mod schema_tests;
#[cfg(test)]
mod tests;
