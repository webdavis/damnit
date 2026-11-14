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
    let flags: Vec<&OsString> = line.iter().take_while(|word| *word != "--").collect();
    let given = |flag: &str| flags.iter().any(|word| *word == flag);
    Format::of(given("--json"), given("--toon"))
}
