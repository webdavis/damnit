use clap::CommandFactory;

use crate::args::{Cli, Command, ConfigArgs, SyncArgs};
use crate::error::CliError;

pub(crate) fn print(group: &str) -> Result<(), CliError> {
    let mut root = Cli::command();
    root.build();
    let printed = root
        .find_subcommand_mut(group)
        .ok_or_else(|| CliError::Internal(format!("no command group named {group}")))?
        .print_help();
    match printed {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => Err(CliError::Io(e.to_string())),
        _ => Ok(()),
    }
}

pub(crate) fn bare(command: &Command) -> Option<&'static str> {
    match command {
        Command::Config(ConfigArgs { command: None }) => Some("config"),
        Command::Sync(SyncArgs { command: None }) => Some("sync"),
        _ => None,
    }
}
