use std::ffi::OsString;

use clap::Parser;

use super::Refused;
use super::misuse::misuse;
use crate::args::Cli;
use crate::error::CliError;
use crate::output::Format;

pub(super) fn parse(line: Vec<OsString>) -> Result<Cli, Refused> {
    Cli::try_parse_from(&line).map_err(|error| {
        let format = format_given(&line);
        match misuse(&error, &line) {
            Some(misuse) if format != Format::Human => Refused {
                format,
                error: CliError::CommandLine(misuse),
            },
            _ => error.exit(),
        }
    })
}

fn format_given(line: &[OsString]) -> Format {
    let words: Vec<&str> = line
        .iter()
        .take_while(|word| *word != "--")
        .filter_map(|word| word.to_str())
        .collect();
    let value_at = |at: usize| match words[at] {
        "-o" | "--output" => words.get(at + 1).copied(),
        word => word
            .strip_prefix("--output=")
            .or_else(|| word.strip_prefix("-o"))
            .map(|value| value.strip_prefix('=').unwrap_or(value)),
    };
    (0..words.len())
        .find_map(|at| value_at(at).and_then(|value| Format::chosen(value).ok()))
        .unwrap_or(Format::Human)
}
