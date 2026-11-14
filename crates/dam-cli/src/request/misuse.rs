use std::ffi::OsString;

use clap::CommandFactory;
use clap::error::{ContextKind, ContextValue, ErrorKind};

use crate::args::Cli;
use crate::error::{Misuse, Problem};

const END_OF_FLAGS: &str = "--";

pub(super) fn misuse(error: &clap::Error, line: &[OsString]) -> Option<Misuse> {
    let mut command = Cli::command();
    command.build();
    let words = |kind| shown(error, kind);
    let flag = || words(ContextKind::InvalidArg).find_map(|arg| flag_shown_as(&command, arg));
    let (problem, word) = match error.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => return None,
        ErrorKind::InvalidSubcommand => (
            Problem::UnknownSubcommand,
            words(ContextKind::InvalidSubcommand)
                .next()
                .map(str::to_string),
        ),
        ErrorKind::UnknownArgument => {
            let word = words(ContextKind::InvalidArg).next().unwrap_or_default();
            let problem = if typed_as_a_flag(word, line) {
                Problem::UnknownFlag
            } else {
                Problem::UnexpectedArgument
            };
            (problem, Some(word.to_string()))
        }
        ErrorKind::InvalidValue if words(ContextKind::InvalidValue).all(str::is_empty) => {
            (Problem::MissingValue, flag())
        }
        ErrorKind::InvalidValue | ErrorKind::ValueValidation => (Problem::BadValue, flag()),
        ErrorKind::MissingRequiredArgument | ErrorKind::MissingSubcommand => {
            (Problem::MissingArgument, flag())
        }
        ErrorKind::ArgumentConflict => (
            Problem::Conflict,
            flag().or_else(|| {
                words(ContextKind::PriorArg).find_map(|arg| flag_shown_as(&command, arg))
            }),
        ),
        _ => (Problem::Other, None),
    };
    Some(Misuse {
        message: sentence(error),
        problem,
        word,
    })
}

pub(super) fn sentence(error: &clap::Error) -> String {
    error
        .render()
        .to_string()
        .split("\n\n")
        .next()
        .unwrap_or_default()
        .trim_start_matches("error: ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn shown(error: &clap::Error, kind: ContextKind) -> impl Iterator<Item = &str> {
    let values: &[String] = match error.get(kind) {
        Some(ContextValue::String(value)) => std::slice::from_ref(value),
        Some(ContextValue::Strings(values)) => values,
        _ => &[],
    };
    values.iter().map(String::as_str)
}

fn flag_shown_as(command: &clap::Command, shown: &str) -> Option<String> {
    command
        .get_arguments()
        .find(|arg| arg.to_string() == shown)
        .and_then(|arg| {
            arg.get_long()
                .map(|long| format!("--{long}"))
                .or_else(|| arg.get_short().map(|short| format!("-{short}")))
        })
        .or_else(|| {
            command
                .get_subcommands()
                .find_map(|sub| flag_shown_as(sub, shown))
        })
}

fn typed_as_a_flag(word: &str, line: &[OsString]) -> bool {
    word.starts_with('-')
        && !line
            .iter()
            .skip_while(|typed| *typed != END_OF_FLAGS)
            .skip(1)
            .any(|typed| typed == word)
}
