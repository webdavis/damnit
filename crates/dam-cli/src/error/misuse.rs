use serde::Serialize;

use super::CliError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Problem {
    UnknownSubcommand,
    UnknownFlag,
    UnexpectedArgument,
    MissingValue,
    BadValue,
    MissingArgument,
    Conflict,
    Other,
}

#[derive(Debug)]
pub(crate) struct Misuse {
    pub(crate) message: String,
    pub(crate) problem: Problem,
    pub(crate) word: Option<String>,
}

pub(super) fn usage_part(error: &CliError) -> Option<serde_json::Value> {
    let (problem, word) = match error {
        CliError::CommandLine(misuse) => (misuse.problem, misuse.word.as_deref()),
        _ if error.kind() == super::KIND_USAGE => (Problem::Other, None),
        _ => return None,
    };
    Some(serde_json::json!({ "problem": problem, "word": word }))
}
