use std::ffi::OsString;

use clap::CommandFactory;
use dam_domain::Oid;
use serde_json::{Map, Value};

use super::with_fields;
use crate::args::{Cli, Command, EditArgs};
use crate::error::CliError;

pub(crate) fn hook_edit_args(task: &Oid, json: &str) -> Result<EditArgs, CliError> {
    let mut fields: Map<String, Value> =
        serde_json::from_str(json).map_err(|why| CliError::Usage(format!("--request: {why}")))?;
    if fields.contains_key("meta") {
        return Err(CliError::Usage(
            "--request: meta is not a field a hook's request takes".into(),
        ));
    }
    fields
        .entry("oid")
        .or_insert_with(|| task.to_string().into());
    let line: Vec<OsString> = ["dam", "edit", "--request", "-"].map(OsString::from).into();
    let given = Cli::command()
        .ignore_errors(true)
        .try_get_matches_from(&line)
        .map_err(|e| CliError::Internal(e.to_string()))?;
    match with_fields(line, &given, &fields)?.command {
        Command::Edit(args) => Ok(args),
        _ => Err(CliError::Internal(
            "a hook's request read as another command".into(),
        )),
    }
}

#[cfg(test)]
mod tests;
