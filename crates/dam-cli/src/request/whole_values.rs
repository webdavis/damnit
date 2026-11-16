use std::ffi::OsString;

use clap::{CommandFactory, FromArgMatches};

use crate::args::Cli;

pub(super) fn parsed(line: Vec<OsString>) -> Result<Cli, clap::Error> {
    taking_each_value_whole(Cli::command())
        .try_get_matches_from(line)
        .and_then(|matches| Cli::from_arg_matches(&matches))
}

fn taking_each_value_whole(command: clap::Command) -> clap::Command {
    command
        .mut_args(|arg| match arg.get_value_delimiter() {
            Some(_) => arg
                .value_delimiter(None)
                .value_parser(clap::value_parser!(String)),
            None => arg,
        })
        .mut_subcommands(taking_each_value_whole)
}

#[cfg(test)]
mod whole_labels_tests;
