use clap::ArgMatches;
use clap::parser::ValueSource;
use serde_json::{Map, Value};

use crate::error::CliError;

pub(super) fn refuse_flags_beside(
    given: &ArgMatches,
    leaf_given: &ArgMatches,
    leaf_command: &clap::Command,
    fields: &Map<String, Value>,
    takes_meta: bool,
) -> Result<(), CliError> {
    let on_the_line =
        |matches: &ArgMatches, id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);
    if takes_meta && fields.contains_key("meta") && on_the_line(given, "meta") {
        return Err(twice("meta"));
    }
    match leaf_command
        .get_arguments()
        .map(|arg| arg.get_id().as_str())
        .find(|id| on_the_line(leaf_given, id))
    {
        Some(id) if fields.contains_key(id) => Err(twice(id)),
        Some(id) => Err(CliError::Usage(format!(
            "--request takes only global flags beside it; put {id} in the request"
        ))),
        None => Ok(()),
    }
}

fn twice(key: &str) -> CliError {
    CliError::Usage(format!(
        "{key} is in the request and on the command line; give it once"
    ))
}
